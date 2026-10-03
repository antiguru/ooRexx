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

use crate::ffi::{CallContext, Contexts, MethodContext, RedirectorContext};
use crate::layout::ValueDescriptor;
use crate::load::{
    CommandHandler, Hook, Library, NativeMethodEntry, NativeRoutineEntry, ROUTINE_CLASSIC_STYLE,
};
use crate::redirect::Redirector;
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
/// The halves are [`prepare`], [`call_method`] and [`finish`], with
/// [`Host::between_halves`](crate::values::Host::between_halves) run between
/// each where [`Activation::between_halves`] says so.
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
    let mut native = NativeCall::empty();
    prepare(&mut native, &signature(entry, context)?, cx, arguments)?;
    if cx.between_halves() {
        cx.conversion().host.between_halves();
    }
    let completion = call_method(&mut native, entry, context);
    if cx.between_halves() {
        cx.conversion().host.between_halves();
    }
    finish(cx, completion)
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
    let mut native = NativeCall::empty();
    prepare(
        &mut native,
        &routine_signature(entry, context)?,
        cx,
        arguments,
    )?;
    if cx.between_halves() {
        cx.conversion().host.between_halves();
    }
    let completion = call_routine(&mut native, entry, context);
    if cx.between_halves() {
        cx.conversion().host.between_halves();
    }
    finish(cx, completion)
}

/// Run `library`'s `hook`, where its package entry declares one, with the
/// thread context `contexts` links.
///
/// A condition the hook raised is left on `cx`, as [`method`] leaves one.
///
/// # Errors
/// [`Failure::UnfilledSlot`] for the first interface member the hook reached
/// that this phase has not written, which also forgets any condition it
/// raised.
pub fn hook(
    library: &Library,
    hook: Hook,
    contexts: &Contexts<'_, '_>,
    cx: &Activation<'_>,
) -> Result<(), Failure> {
    let ((), refused) = crate::layout::recording_refusals(|| library.run_hook(hook, contexts));
    if let Some(entry) = refused {
        cx.clear_pending();
        return Err(Failure::UnfilledSlot { entry });
    }
    Ok(())
}

/// Run the command handler `handler` for `command`, issued to the environment
/// `address`, with the exit context `contexts` links and, for a redirecting
/// handler, `redirector` (`ContextCommandHandlerDispatcher::run` and its
/// redirecting sibling, `interpreter/concurrency/CommandHandler.cpp:231-295`).
///
/// The answer is the object the handler returned, `None` for a null or a
/// handle this call does not hold. A condition it raised, a `Throw` member's
/// included, is left on `cx`, as [`method`] leaves one.
///
/// # Errors
/// [`Failure::UnfilledSlot`] for the first interface member the handler
/// reached that this phase has not written, which also forgets any condition
/// it raised.
pub fn command(
    handler: &CommandHandler,
    contexts: &mut Contexts<'_, '_>,
    cx: &Activation<'_>,
    address: ObjRef,
    command: ObjRef,
    redirector: &Redirector,
) -> Result<Option<ObjRef>, Failure> {
    let (address, command) = {
        let mut conversion = cx.conversion();
        let locals = conversion.host.locals();
        (locals.register(address), locals.register(command))
    };
    let exit = contexts.exit();
    let mut io = RedirectorContext::new(redirector);
    let (answered, refused) = crate::layout::recording_refusals(|| {
        handler.call(&exit, address.cast(), command.cast(), &mut io)
    });
    if let Some(entry) = refused {
        cx.clear_pending();
        return Err(Failure::UnfilledSlot { entry });
    }
    Ok(cx.conversion().host.resolve(answered))
}

/// A native call's declared result type and the descriptor array its stub is
/// handed, which [`prepare`] fills in place.
///
/// It holds raw words only, never an `ObjRef`, so that it is `Send`.
pub struct NativeCall {
    returns: u16,
    descriptors: [ValueDescriptor; MAX_NATIVE_ARGUMENTS],
}

impl NativeCall {
    /// A call with every descriptor empty.
    #[inline(always)]
    pub fn empty() -> NativeCall {
        NativeCall {
            returns: ARGUMENT_TERMINATOR,
            descriptors: std::array::from_fn(|_| empty()),
        }
    }
}

/// What a native call left behind for [`finish`]: the declared result type,
/// what the stub wrote into element zero, and the first interface member it
/// reached that this phase has not written.
///
/// It holds raw words only, never an `ObjRef`, so that it is `Send`; the
/// `const` block below asserts that for it and for [`NativeCall`]. A
/// condition the stub raised is held by the host, in the call's native frame
/// (or, for a host without a surface, by the call's [`Activation`]).
pub struct Completion {
    returns: u16,
    written: Option<Written>,
    refused: Option<&'static str>,
}

const _: () = {
    const fn require_send<T: Send>() {}
    require_send::<NativeCall>();
    require_send::<Completion>();
};

/// `processArguments`: `arguments` converted into `native`'s descriptors as
/// `signature` declares them. `native` is filled in place rather than
/// answered, which keeps the descriptor array from being copied.
///
/// # Errors
/// As [`method`], for everything refused before the stub is entered.
///
/// # Panics
/// If the caller holds `cx`'s conversion state across this call.
#[inline(always)]
pub fn prepare(
    native: &mut NativeCall,
    signature: &[u16],
    cx: &Activation<'_>,
    arguments: &[Option<ObjRef>],
) -> Result<(), Failure> {
    let returns = signature.first().copied().unwrap_or(ARGUMENT_TERMINATOR);
    native.returns = returns;
    let descriptors = &mut native.descriptors;
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
        // never held across the call: the context the call hands the
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
    Ok(())
}

/// `call` for the native method `entry`.
#[inline(always)]
pub fn call_method(
    native: &mut NativeCall,
    entry: &NativeMethodEntry,
    context: &MethodContext<'_>,
) -> Completion {
    call(native, |descriptors, result| {
        entry.call(context, descriptors, result)
    })
}

/// `call` for the native routine `entry`.
#[inline(always)]
pub fn call_routine(
    native: &mut NativeCall,
    entry: &NativeRoutineEntry,
    context: &CallContext<'_>,
) -> Completion {
    call(native, |descriptors, result| {
        entry.call(context, descriptors, result)
    })
}

/// Runs `stub` over `native`'s descriptors, recording the first unwritten
/// member it reaches.
#[inline(always)]
fn call(
    native: &mut NativeCall,
    stub: impl FnOnce(
        &mut [ValueDescriptor; MAX_NATIVE_ARGUMENTS],
        Option<ResultRead>,
    ) -> Option<Written>,
) -> Completion {
    let returns = native.returns;
    let (written, refused) = crate::layout::recording_refusals(|| {
        stub(&mut native.descriptors, values::result_read(returns))
    });
    Completion {
        returns,
        written,
        refused,
    }
}

/// `valueToObject`: the object `completion`'s result describes, resolved in
/// `cx`.
///
/// # Errors
/// [`Failure::UnfilledSlot`] where the call reached an unwritten member, which
/// also forgets any condition it raised; otherwise as [`method`] for the
/// result.
///
/// # Panics
/// If the caller holds `cx`'s conversion state across this call.
#[inline(always)]
pub fn finish(cx: &Activation<'_>, completion: Completion) -> Result<Option<ObjRef>, Failure> {
    let Completion {
        returns,
        written,
        refused,
    } = completion;
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

/// The types the routine `entry` declares, as [`signature`] answers a
/// method's.
///
/// # Errors
/// [`Failure::ClassicStyle`] for a `ROUTINE_CLASSIC_STYLE` row, whose stub
/// is not entered; otherwise as [`signature`].
pub fn routine_signature(
    entry: &NativeRoutineEntry,
    context: &CallContext<'_>,
) -> Result<Vec<u16>, Failure> {
    if entry.style == ROUTINE_CLASSIC_STYLE {
        return Err(Failure::ClassicStyle);
    }
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
