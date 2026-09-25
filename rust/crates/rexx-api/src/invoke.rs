/*----------------------------------------------------------------------------*/
/*                                                                            */
/* Copyright (c) 2026 Rexx Language Association. All rights reserved.          */
/*                                                                            */
/* This program and the accompanying materials are made available under       */
/* the terms of the Common Public License v1.0 which accompanies this         */
/* distribution. A copy is also available at the following address:           */
/* https://www.oorexx.org/license.html                                        */
/*                                                                            */
/*----------------------------------------------------------------------------*/

//! The two-call protocol: ask the stub for its signature, then call it with
//! the arguments that signature declares.
//!
//! `NativeActivation::run` (`interpreter/execution/NativeActivation.cpp:1264-1310`)
//! is the reference.

use rexx_core::ObjRef;

use crate::ffi::{CallContext, MethodContext};
use crate::layout::ValueDescriptor;
use crate::load::{NativeMethodEntry, NativeRoutineEntry, ROUTINE_CLASSIC_STYLE};
use crate::values::{
    self, ARGUMENT_TERMINATOR, Activation, Converted, Failure, ResultRead, Value, Written,
};

/// `NativeActivation::MaxNativeArguments`
/// (`interpreter/execution/NativeActivation.hpp:209`), the length of the
/// descriptor array, whose first element is the return value.
pub const MAX_NATIVE_ARGUMENTS: usize = 16;

/// Run the native method `entry` against `arguments`.
///
/// `context` is the method context the extension is handed. `arguments` is
/// the Rexx argument list, in which `None` is a position the caller left out.
/// The answer is the object the extension returned, `None` where the oracle
/// answers `OREF_NULL`.
///
/// The result is read whatever the call did with it: an extension that raises
/// a condition returns normally and still writes element zero
/// (`interpreter/api/ThreadContextStubs.cpp:1863-1885`). A condition it
/// raised is left on `cx` for the caller to raise once the call has returned,
/// which is where `NativeActivation::checkConditions`
/// (`interpreter/execution/NativeActivation.cpp:1787`) raises it.
///
/// # Errors
/// Whatever converting an argument or the result refuses;
/// [`Failure::Signature`] for a signature the descriptor array cannot hold,
/// or for a parameter code the table does not know that is given an argument
/// or carries the optional bit; [`Failure::MissingArgument`] for a parameter
/// that takes an argument, is not optional and was given none, whether or not
/// the table knows its code; [`Failure::ResultSignature`] for
/// a return code it does not know or does not convert back;
/// [`Failure::TooManyArguments`] for arguments the signature does not consume,
/// where no parameter takes the argument list; [`Failure::UnfilledSlot`] for the
/// first interface member the extension reached that this phase has not
/// written, which also forgets any condition the extension raised.
///
/// # Panics
/// If the caller holds `cx`'s conversion state across this call.
pub fn method(
    entry: &NativeMethodEntry,
    context: &MethodContext<'_>,
    cx: &Activation<'_>,
    arguments: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let signature = signature(entry, context)?;
    run(&signature, cx, arguments, |descriptors, result| {
        entry.call(context, descriptors, result)
    })
}

/// Run the native routine `entry` against `arguments`, as [`method`] runs a
/// method: `NativeActivation::callNativeRoutine`
/// (`interpreter/execution/NativeActivation.cpp:1379`) shares
/// `processArguments` with the method call.
///
/// # Errors
/// [`method`]'s, and [`Failure::ClassicStyle`] for a
/// `ROUTINE_CLASSIC_STYLE` row, which is refused before its stub is entered.
///
/// # Panics
/// As [`method`].
pub fn routine(
    entry: &NativeRoutineEntry,
    context: &CallContext<'_>,
    cx: &Activation<'_>,
    arguments: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if entry.style == ROUTINE_CLASSIC_STYLE {
        return Err(Failure::ClassicStyle);
    }
    let signature = bounded(|limit| entry.signature(context, limit))?;
    run(&signature, cx, arguments, |descriptors, result| {
        entry.call(context, descriptors, result)
    })
}

/// The half of the protocol both calls share once the signature is in hand:
/// `processArguments`, then `call`, then `valueToObject`.
fn run(
    signature: &[u16],
    cx: &Activation<'_>,
    arguments: &[Option<ObjRef>],
    call: impl FnOnce(
        &mut [ValueDescriptor; MAX_NATIVE_ARGUMENTS],
        Option<ResultRead>,
    ) -> Option<Written>,
) -> Result<Option<ObjRef>, Failure> {
    let returns = signature.first().copied().unwrap_or(ARGUMENT_TERMINATOR);

    let mut descriptors: [ValueDescriptor; MAX_NATIVE_ARGUMENTS] = std::array::from_fn(|_| empty());
    // The result word as declared, optional bit and all
    // (`NativeActivation.cpp:228`).
    descriptors[0].r#type = returns;

    // The oracle's `inputIndex`: a special argument fills a descriptor
    // without consuming one of these, which is why the position an error
    // reports counts only the arguments (`NativeActivation.cpp:327`).
    let mut input = 0;
    // The oracle's `usedArglist` (`NativeActivation.cpp:311`).
    let mut takes_list = false;
    for (output, declared) in signature.iter().copied().enumerate().skip(1) {
        let consumes = values::consumes_argument(declared);
        let argument = if consumes {
            arguments.get(input).copied().flatten()
        } else {
            None
        };
        // **The conversion state is taken for one argument and given back**,
        // never held across the call below: the context that call hands the
        // extension reaches this same state.
        let converted = values::to_native(&mut cx.conversion(), declared, argument, input + 1)?;
        descriptors[output] = values::descriptor(declared, converted);
        if consumes {
            input += 1;
        }
        takes_list |= values::takes_argument_list(declared);
    }
    if input < arguments.len() && !takes_list {
        return Err(Failure::TooManyArguments { expected: input });
    }

    let (written, refused) =
        crate::layout::recording_refusals(|| call(&mut descriptors, values::result_read(returns)));
    if let Some(entry) = refused {
        // Ahead of the result and of any condition the extension raised
        // during the call, before the refused member or after it.
        cx.clear_pending();
        return Err(Failure::UnfilledSlot { entry });
    }

    if returns == ARGUMENT_TERMINATOR {
        return Ok(None);
    }
    let value = match written.ok_or(Failure::ResultSignature)? {
        Written::Member(value) => value,
        Written::Text(None) => Value::CString(std::ptr::null()),
        Written::Text(Some(bytes)) => Value::CString(cx.conversion().strings.intern(&bytes)),
    };
    values::from_native(&mut cx.conversion(), returns, value)
}

/// The types `entry` declares: its return type, then its parameters.
///
/// Nothing is published on `context` and no argument is read, so this answers
/// the same thing whatever the caller is holding.
///
/// # Errors
/// [`Failure::Signature`] where the stub publishes no array or one longer
/// than [`MAX_NATIVE_ARGUMENTS`] leaves room for.
pub fn signature(
    entry: &NativeMethodEntry,
    context: &MethodContext<'_>,
) -> Result<Vec<u16>, Failure> {
    bounded(|limit| entry.signature(context, limit))
}

/// The signature `read` answers when it may read at most `limit` words.
fn bounded(read: impl FnOnce(usize) -> Option<Vec<u16>>) -> Result<Vec<u16>, Failure> {
    // The words are the return type, the parameters the array has room for,
    // and the terminator, so a signature that fits is one word longer than
    // the array. Reading no further is this crate's form of the oracle's
    // bound check (`NativeActivation.cpp:238-241`).
    read(MAX_NATIVE_ARGUMENTS + 1).ok_or(Failure::Signature)
}

/// A descriptor holding nothing, which is what the oracle's `type` of zero
/// and zeroed value word describe (`NativeActivation.cpp:228-229`).
fn empty() -> ValueDescriptor {
    values::descriptor(
        ARGUMENT_TERMINATOR,
        Converted {
            value: Value::Omitted,
            flags: 0,
        },
    )
}

#[cfg(test)]
mod tests;
