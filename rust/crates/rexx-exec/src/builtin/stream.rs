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

//! The stream builtins. Each resolves its first argument to a `Stream` through
//! the running activation's table and sends the message named after itself
//! with the arguments that followed, positionally, so an omitted middle
//! argument arrives omitted (`BuiltinFunctions.cpp:2126-2570`).

use rexx_core::ObjRef;

use super::{Args, arg, optional_string, required_string};
use crate::error::Raised;
use crate::{Failure, Interp};

/// Resolves `args`' first argument and sends `message` the rest of them.
/// `added` is the oracle's own `&added`: whether a miss may leave the stream
/// in the table.
fn send_to_stream(
    interp: &mut Interp,
    args: Args<'_>,
    message: &[u8],
    input: bool,
    added: bool,
) -> Result<ObjRef, Failure> {
    let name = match arg(args, 1) {
        Some(value) => interp.to_text(value).into_owned(),
        None => Vec::new(),
    };
    let stream = interp.resolve_stream(&name, input, added)?;
    // **Guarded**: `values_from` slices from its argument, so asking for the
    // tail of a call that wrote no positions at all panics rather than
    // answering an empty slice, and an omitted name is a call with none.
    let rest: Vec<Option<ObjRef>> = if args.len() < 2 {
        Vec::new()
    } else {
        args.values_from(2).to_vec()
    };
    let caller = interp.caller();
    interp
        .send_message(stream, message, None, &rest, caller)?
        .ok_or_else(|| Failure::from(Raised::no_result(message)))
}

/// `LINEIN(name, [line], [count])`.
pub(crate) fn linein(
    interp: &mut Interp,
    _name: &'static [u8],
    args: Args<'_>,
) -> Result<ObjRef, Failure> {
    send_to_stream(interp, args, b"LINEIN", true, true)
}

/// `CHARIN(name, [start], [length])`.
pub(crate) fn charin(
    interp: &mut Interp,
    _name: &'static [u8],
    args: Args<'_>,
) -> Result<ObjRef, Failure> {
    send_to_stream(interp, args, b"CHARIN", true, true)
}

/// `CHAROUT(name, [string], [start])`.
pub(crate) fn charout(
    interp: &mut Interp,
    _name: &'static [u8],
    args: Args<'_>,
) -> Result<ObjRef, Failure> {
    send_to_stream(interp, args, b"CHAROUT", false, true)
}

/// `CHARS(name)`.
pub(crate) fn chars(
    interp: &mut Interp,
    _name: &'static [u8],
    args: Args<'_>,
) -> Result<ObjRef, Failure> {
    send_to_stream(interp, args, b"CHARS", true, true)
}

/// `LINEOUT(name, [string], [line])`. **With neither a string nor a line the
/// entry is dropped after the send**, which is what closes the stream and
/// what makes the next `LINEOUT` to that name reopen it (`:2277`).
pub(crate) fn lineout(
    interp: &mut Interp,
    _name: &'static [u8],
    args: Args<'_>,
) -> Result<ObjRef, Failure> {
    let answer = send_to_stream(interp, args, b"LINEOUT", false, true)?;
    if args.len() < 2 {
        let name = match arg(args, 1) {
            Some(value) => interp.to_text(value).into_owned(),
            None => Vec::new(),
        };
        interp.forget_stream(&name);
    }
    Ok(answer)
}

/// `LINES(name, [option])`. **The builtin's default is `NORMAL` where the
/// method's is `COUNT`**, and the builtin squashes a count to 1 or 0 itself
/// (`:2348-2391`), so `lines(f)` answers whether a line is there rather than
/// how many are left.
pub(crate) fn lines(
    interp: &mut Interp,
    name: &'static [u8],
    args: Args<'_>,
) -> Result<ObjRef, Failure> {
    let option = match arg(args, 2) {
        None => b'N',
        Some(value) => match interp.to_text(value).first().copied() {
            Some(byte) if byte.eq_ignore_ascii_case(&b'C') => b'C',
            Some(byte) if byte.eq_ignore_ascii_case(&b'N') => b'N',
            _ => {
                let shown = required_string(interp, args, 2);
                return Err(Raised::argument_not_in_list(name, 2, "CN", &shown).into());
            }
        },
    };
    let stream_name = match arg(args, 1) {
        Some(value) => interp.to_text(value).into_owned(),
        None => Vec::new(),
    };
    let stream = interp.resolve_stream(&stream_name, true, true)?;
    let argument = interp.text(if option == b'C' { b"C" } else { b"N" });
    let caller = interp.caller();
    let answer = interp
        .send_message(stream, b"LINES", None, &[Some(argument)], caller)?
        .ok_or_else(|| Failure::from(Raised::no_result(b"LINES")))?;
    Ok(answer)
}

/// `STREAM(name, [operation], [command])`. The operation is read by its first
/// letter only: `S` (the default) sends `STATE`, `D` sends `DESCRIPTION`, and
/// `C` sends `COMMAND` with the third argument. **The arity depends on the
/// operation** -- `S` and `D` refuse a third argument and `C` requires one --
/// so those two checks live here rather than in the row's flat min and max.
pub(crate) fn stream(
    interp: &mut Interp,
    name: &'static [u8],
    args: Args<'_>,
) -> Result<ObjRef, Failure> {
    let stream_name = required_string(interp, args, 1);
    // Measured: only the empty string is refused. A blank is a name and
    // answers UNKNOWN, and every other reader takes an empty name as the
    // default input or output rather than an error.
    if stream_name.is_empty() {
        return Err(Raised::invalid_stream_name(&stream_name).into());
    }
    let option = optional_string(interp, args, 2);
    let letter = match option.as_deref() {
        None => b'S',
        Some(text) => match text.first().copied() {
            Some(byte) if byte.eq_ignore_ascii_case(&b'S') => b'S',
            Some(byte) if byte.eq_ignore_ascii_case(&b'D') => b'D',
            Some(byte) if byte.eq_ignore_ascii_case(&b'C') => b'C',
            _ => return Err(Raised::argument_not_in_list(name, 2, "SDC", text).into()),
        },
    };
    if letter == b'C' {
        if args.len() < 3 {
            return Err(Raised::not_enough_arguments(name, 3).into());
        }
    } else if args.len() > 2 {
        return Err(Raised::too_many_arguments(name, 2).into());
    }
    let command = if letter == b'C' {
        Some(required_string(interp, args, 3))
    } else {
        None
    };
    // `&added`: STATE and DESCRIPTION never populate the table, and neither
    // does a command that is not an OPEN, CLOSE or SEEK -- which is why
    // `stream(n,'S')` on an unknown name leaves nothing behind.
    let touches = command.as_deref().is_some_and(repositions);
    // The direction is unreachable here and measured so: `stream('')` and
    // `stream('','S')` are both 40.27 above, so the empty name never gets
    // this far and no default stream is ever chosen for `STREAM`.
    let stream = interp.resolve_stream(&stream_name, true, touches)?;
    let caller = interp.caller();
    let answer = match letter {
        b'S' => interp.send_message(stream, b"STATE", None, &[], caller)?,
        b'D' => interp.send_message(stream, b"DESCRIPTION", None, &[], caller)?,
        _ => {
            let text = command.clone().unwrap_or_default();
            let argument = interp.text_built(text);
            interp.send_message(stream, b"COMMAND", None, &[Some(argument)], caller)?
        }
    };
    let answer = answer.ok_or_else(|| Failure::from(Raised::no_result(b"COMMAND")))?;
    // A CLOSE drops the entry, and an OPEN drops it when the answer is not
    // `READY:`. The word scan is over the whole upper-cased command, so
    // `query open` takes the OPEN branch -- the oracle's own quirk.
    if let Some(command) = command.as_deref() {
        let upper = command.to_ascii_uppercase();
        let words: Vec<&[u8]> = upper.split(|byte| byte.is_ascii_whitespace()).collect();
        let closed = words.contains(&&b"CLOSE"[..]);
        let failed_open =
            words.contains(&&b"OPEN"[..]) && interp.to_text(answer).as_ref() != b"READY:";
        if closed || failed_open {
            interp.forget_stream(&stream_name);
        }
    }
    Ok(answer)
}

/// Whether a `STREAM` command is one the table follows: the oracle scans the
/// whole upper-cased string for the words rather than reading the first one.
fn repositions(command: &[u8]) -> bool {
    let upper = command.to_ascii_uppercase();
    upper
        .split(|byte| byte.is_ascii_whitespace())
        .any(|word| word == b"OPEN" || word == b"CLOSE" || word == b"SEEK")
}
