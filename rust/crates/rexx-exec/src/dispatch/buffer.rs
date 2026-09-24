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

//! `MutableBuffer`'s primitive methods, and the argument and byte-search
//! helpers the other primitive methods share.

use super::method_arguments::{
    named_pad_argument, named_position_argument, named_string_argument, option_method_argument,
    optional_length_argument, optional_named_length_argument, optional_non_negative_argument,
    optional_position_argument, optional_string_method_argument, optional_string_or_none_argument,
    pad_method_argument, refuse_method_argument, required_length_argument,
    required_position_argument, string_method_argument, whole_method_argument,
};
use super::{
    Arity, BehaviourId, Body, BufferState, Cleared, Failure, INIT, Interp, Loud, NativeMethod,
    ObjRef, Object, Raised, class_receiver, new_instance, required_string_argument,
};

/// `MutableBuffer`'s instance methods. Chained into `ObjectModel::build`
/// beside [`super::NATIVE_METHODS`].
pub(super) static NATIVE_METHODS: &[(&str, &str, Arity, NativeMethod)] = &[
    // `MutableBuffer`'s rows in `memory/Setup.cpp:1416`-`:1479`, each at its
    // declared count.
    (
        "MutableBuffer",
        "[]",
        Arity::Fixed(2),
        native_mutable_buffer_brackets,
    ),
    (
        "MutableBuffer",
        "[]=",
        Arity::Fixed(3),
        native_mutable_buffer_bracketsequal,
    ),
    (
        "MutableBuffer",
        "APPEND",
        Arity::Counted,
        native_mutable_buffer_append,
    ),
    (
        "MutableBuffer",
        "CASELESSCHANGESTR",
        Arity::Fixed(3),
        native_mutable_buffer_caselesschangestr,
    ),
    (
        "MutableBuffer",
        "CASELESSCONTAINS",
        Arity::Fixed(3),
        native_mutable_buffer_caselesscontains,
    ),
    (
        "MutableBuffer",
        "CASELESSCONTAINSWORD",
        Arity::Fixed(2),
        native_mutable_buffer_caselesscontainsword,
    ),
    (
        "MutableBuffer",
        "CASELESSCOUNTSTR",
        Arity::Fixed(1),
        native_mutable_buffer_caselesscountstr,
    ),
    (
        "MutableBuffer",
        "CASELESSENDSWITH",
        Arity::Fixed(1),
        native_mutable_buffer_caselessendswith,
    ),
    (
        "MutableBuffer",
        "CASELESSLASTPOS",
        Arity::Fixed(3),
        native_mutable_buffer_caselesslastpos,
    ),
    (
        "MutableBuffer",
        "CASELESSMATCH",
        Arity::Fixed(4),
        native_mutable_buffer_caselessmatch,
    ),
    (
        "MutableBuffer",
        "CASELESSMATCHCHAR",
        Arity::Fixed(2),
        native_mutable_buffer_caselessmatchchar,
    ),
    (
        "MutableBuffer",
        "CASELESSPOS",
        Arity::Fixed(3),
        native_mutable_buffer_caselesspos,
    ),
    (
        "MutableBuffer",
        "CASELESSSTARTSWITH",
        Arity::Fixed(1),
        native_mutable_buffer_caselessstartswith,
    ),
    (
        "MutableBuffer",
        "CASELESSWORDPOS",
        Arity::Fixed(2),
        native_mutable_buffer_caselesswordpos,
    ),
    (
        "MutableBuffer",
        "CHANGESTR",
        Arity::Fixed(3),
        native_mutable_buffer_changestr,
    ),
    (
        "MutableBuffer",
        "CONTAINS",
        Arity::Fixed(3),
        native_mutable_buffer_contains,
    ),
    (
        "MutableBuffer",
        "CONTAINSWORD",
        Arity::Fixed(2),
        native_mutable_buffer_containsword,
    ),
    (
        "MutableBuffer",
        "COUNTSTR",
        Arity::Fixed(1),
        native_mutable_buffer_countstr,
    ),
    (
        "MutableBuffer",
        "DELETE",
        Arity::Fixed(2),
        native_mutable_buffer_delete,
    ),
    (
        "MutableBuffer",
        "DELSTR",
        Arity::Fixed(2),
        native_mutable_buffer_delstr,
    ),
    (
        "MutableBuffer",
        "DELWORD",
        Arity::Fixed(2),
        native_mutable_buffer_delword,
    ),
    (
        "MutableBuffer",
        "ENDSWITH",
        Arity::Fixed(1),
        native_mutable_buffer_endswith,
    ),
    (
        "MutableBuffer",
        "GETBUFFERSIZE",
        Arity::Fixed(0),
        native_mutable_buffer_getbuffersize,
    ),
    (
        "MutableBuffer",
        "INSERT",
        Arity::Fixed(4),
        native_mutable_buffer_insert,
    ),
    (
        "MutableBuffer",
        "LASTPOS",
        Arity::Fixed(3),
        native_mutable_buffer_lastpos,
    ),
    (
        "MutableBuffer",
        "LENGTH",
        Arity::Fixed(0),
        native_mutable_buffer_length,
    ),
    (
        "MutableBuffer",
        "LOWER",
        Arity::Fixed(2),
        native_mutable_buffer_lower,
    ),
    (
        "MutableBuffer",
        "MAKEARRAY",
        Arity::Fixed(1),
        native_mutable_buffer_makearray,
    ),
    (
        "MutableBuffer",
        "MAKESTRING",
        Arity::Fixed(0),
        native_mutable_buffer_makestring,
    ),
    (
        "MutableBuffer",
        "MATCH",
        Arity::Fixed(4),
        native_mutable_buffer_match,
    ),
    (
        "MutableBuffer",
        "MATCHCHAR",
        Arity::Fixed(2),
        native_mutable_buffer_matchchar,
    ),
    (
        "MutableBuffer",
        "OVERLAY",
        Arity::Fixed(4),
        native_mutable_buffer_overlay,
    ),
    (
        "MutableBuffer",
        "POS",
        Arity::Fixed(3),
        native_mutable_buffer_pos,
    ),
    (
        "MutableBuffer",
        "REPLACEAT",
        Arity::Fixed(4),
        native_mutable_buffer_replaceat,
    ),
    (
        "MutableBuffer",
        "SETBUFFERSIZE",
        Arity::Fixed(1),
        native_mutable_buffer_setbuffersize,
    ),
    (
        "MutableBuffer",
        "SETTEXT",
        Arity::Fixed(1),
        native_mutable_buffer_settext,
    ),
    (
        "MutableBuffer",
        "SPACE",
        Arity::Fixed(2),
        native_mutable_buffer_space,
    ),
    (
        "MutableBuffer",
        "STARTSWITH",
        Arity::Fixed(1),
        native_mutable_buffer_startswith,
    ),
    (
        "MutableBuffer",
        "STRING",
        Arity::Fixed(0),
        native_mutable_buffer_string,
    ),
    (
        "MutableBuffer",
        "SUBCHAR",
        Arity::Fixed(1),
        native_mutable_buffer_subchar,
    ),
    (
        "MutableBuffer",
        "SUBSTR",
        Arity::Fixed(3),
        native_mutable_buffer_substr,
    ),
    (
        "MutableBuffer",
        "SUBWORD",
        Arity::Fixed(2),
        native_mutable_buffer_subword,
    ),
    (
        "MutableBuffer",
        "SUBWORDS",
        Arity::Fixed(2),
        native_mutable_buffer_subwords,
    ),
    (
        "MutableBuffer",
        "TRANSLATE",
        Arity::Fixed(5),
        native_mutable_buffer_translate,
    ),
    (
        "MutableBuffer",
        "UPPER",
        Arity::Fixed(2),
        native_mutable_buffer_upper,
    ),
    (
        "MutableBuffer",
        "VERIFY",
        Arity::Fixed(4),
        native_mutable_buffer_verify,
    ),
    (
        "MutableBuffer",
        "WORD",
        Arity::Fixed(1),
        native_mutable_buffer_word,
    ),
    (
        "MutableBuffer",
        "WORDINDEX",
        Arity::Fixed(1),
        native_mutable_buffer_wordindex,
    ),
    (
        "MutableBuffer",
        "WORDLENGTH",
        Arity::Fixed(1),
        native_mutable_buffer_wordlength,
    ),
    (
        "MutableBuffer",
        "WORDPOS",
        Arity::Fixed(2),
        native_mutable_buffer_wordpos,
    ),
    (
        "MutableBuffer",
        "WORDS",
        Arity::Fixed(0),
        native_mutable_buffer_words,
    ),
];

/// `MutableBuffer~new(string, size, ...)`: an instance carrying the string
/// and the capacities `MutableBuffer::newRexx` derives from its arguments
/// (`classes/MutableBufferClass.cpp:90`): `default_size` is the second
/// argument or `DEFAULT_BUFFER_LENGTH`, and the capacity is that or the
/// string's length, whichever is larger.
pub(super) fn native_mutable_buffer_new(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    let mut initial = interp.take_result_buffer();
    if let Some(text) = args.first().copied().flatten() {
        let text = required_string_argument(interp, text, 1)?;
        initial.extend_from_slice(&interp.to_text(text));
    }
    let default_size = optional_length_argument(interp, args, 1)?.unwrap_or(BUFFER_DEFAULT_LENGTH);
    let capacity = default_size.max(initial.len());
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(capacity)
        .map_err(|_| Failure::from(Raised::system_resources()))?;
    bytes.extend_from_slice(&initial);
    interp.give_result_buffer(initial);
    let object = new_instance(interp, class)?;
    let Some(Object {
        body: Body::Instance { native, .. },
        ..
    }) = interp.heap.get_mut(object)
    else {
        unreachable!("new_instance allocates a Body::Instance")
    };
    *native = Some(Box::new(rexx_core::NativeState::Buffer(BufferState {
        bytes,
        capacity,
        default_size,
    })));
    let caller = interp.caller();
    let kept = args.len().saturating_sub(2);
    let rest: Vec<Option<ObjRef>> = args.iter().take(kept).copied().collect();
    interp.send_message(object, INIT, None, &rest, caller)?;
    Ok(Some(object))
}

/// `MutableBuffer::DEFAULT_BUFFER_LENGTH` (`classes/MutableBufferClass.hpp:150`).
const BUFFER_DEFAULT_LENGTH: usize = 256;

/// The receiver's buffer state, or [`Loud::native_method`] for a receiver
/// that carries none.
fn buffer_state<'a>(
    interp: &'a Interp,
    receiver: ObjRef,
    name: &[u8],
) -> Result<&'a BufferState, Failure> {
    interp
        .buffer(receiver)
        .ok_or_else(|| Loud::native_method(name, "MutableBuffer").into())
}

/// [`buffer_state`] for a method that changes the contents.
fn buffer_state_mut<'a>(
    interp: &'a mut Interp,
    receiver: ObjRef,
    name: &[u8],
) -> Result<&'a mut BufferState, Failure> {
    interp
        .buffer_mut(receiver)
        .ok_or_else(|| Loud::native_method(name, "MutableBuffer").into())
}

/// `MutableBuffer::lengthRexx` (`classes/MutableBufferClass.cpp:310`).
fn native_mutable_buffer_length(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let length = buffer_state(interp, receiver, b"LENGTH")?.bytes.len();
    Ok(Some(interp.counted(length)))
}

/// `MutableBuffer::getBufferSize` (`classes/MutableBufferClass.hpp:91`).
fn native_mutable_buffer_getbuffersize(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let capacity = buffer_state(interp, receiver, b"GETBUFFERSIZE")?.capacity;
    Ok(Some(interp.counted(capacity)))
}

/// `MutableBuffer~string`, `RexxObject::makeStringRexx` reaching
/// `MutableBuffer::stringValue` (`classes/MutableBufferClass.cpp:740`): a
/// fresh string of the contents.
fn native_mutable_buffer_string(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let state = buffer_state(interp, receiver, b"STRING")?;
    let mut out = interp.take_result_buffer();
    out.extend_from_slice(&state.bytes);
    Ok(Some(interp.text_built(out)))
}

/// `MutableBuffer~makeString`, `RexxObject::makeStringRexx`
/// (`classes/ObjectClass.cpp:2846`) reaching `MutableBuffer::makeString`
/// (`classes/MutableBufferClass.cpp:717`): a fresh string of the contents.
/// `Setup.cpp:1446` and `:1473` bind that one C++ entry under both `String`
/// and `makeString`.
fn native_mutable_buffer_makestring(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let state = buffer_state(interp, receiver, b"MAKESTRING")?;
    let mut out = interp.take_result_buffer();
    out.extend_from_slice(&state.bytes);
    Ok(Some(interp.text_built(out)))
}

/// `MutableBuffer::endsWithRexx` (`classes/MutableBufferClass.cpp:1569`).
fn native_mutable_buffer_endswith(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let needle = named_string_argument(interp, args, 0, "match")?;
    let state = buffer_state(interp, receiver, b"ENDSWITH")?;
    let answer = ends_with(&state.bytes, &needle, <[u8]>::eq);
    interp.give_result_buffer(needle);
    Ok(Some(interp.counted(usize::from(answer))))
}

/// `MutableBuffer::caselessEndsWithRexx` (`classes/MutableBufferClass.cpp:1590`):
/// `ENDSWITH`'s region compare, folded, an empty `match` likewise `0`.
fn native_mutable_buffer_caselessendswith(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let needle = named_string_argument(interp, args, 0, "match")?;
    let state = buffer_state(interp, receiver, b"CASELESSENDSWITH")?;
    let answer = ends_with(&state.bytes, &needle, crate::builtin::string::caseless_eq);
    interp.give_result_buffer(needle);
    Ok(Some(interp.counted(usize::from(answer))))
}

/// `MutableBuffer::appendRexx` (`classes/MutableBufferClass.cpp:323`): every
/// argument a required string, appended in turn; answers the receiver.
fn native_mutable_buffer_append(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if args.is_empty() {
        return Err(Raised::missing_method_argument(1).into());
    }
    for (index, argument) in args.iter().enumerate() {
        let Some(argument) = *argument else {
            return Err(Raised::missing_method_argument(index + 1).into());
        };
        let argument = required_string_argument(interp, argument, index + 1)?;
        let mut piece = interp.take_result_buffer();
        piece.extend_from_slice(&interp.to_text(argument));
        let state = buffer_state_mut(interp, receiver, b"APPEND")?;
        state
            .ensure_capacity(piece.len())
            .map_err(|_| Failure::from(Raised::system_resources()))?;
        state.bytes.extend_from_slice(&piece);
        interp.give_result_buffer(piece);
    }
    Ok(Some(receiver))
}

/// `MutableBuffer::mydelete` (`classes/MutableBufferClass.cpp:647`): the
/// position defaults to 1 and the length to the rest of the contents, and a
/// position past them deletes nothing; answers the receiver.
fn buffer_delete(
    interp: &mut Interp,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
    name: &[u8],
) -> Result<Option<ObjRef>, Failure> {
    let (begin, range) = delete_arguments(interp, args)?;
    let state = buffer_state_mut(interp, receiver, name)?;
    crate::builtin::string::delete_range(&mut state.bytes, begin, range);
    Ok(Some(receiver))
}

/// `MutableBuffer~delStr`, [`buffer_delete`].
fn native_mutable_buffer_delstr(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    buffer_delete(interp, receiver, args, b"DELSTR")
}

/// `MutableBuffer~delete`, [`buffer_delete`].
fn native_mutable_buffer_delete(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    buffer_delete(interp, receiver, args, b"DELETE")
}

/// `MutableBuffer::setBufferSize` (`classes/MutableBufferClass.cpp:679`),
/// whose rule [`BufferState::set_buffer_size`] carries; answers the receiver.
fn native_mutable_buffer_setbuffersize(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let size = required_length_argument(interp, args, 0)?;
    let state = buffer_state_mut(interp, receiver, b"SETBUFFERSIZE")?;
    state
        .set_buffer_size(size)
        .map_err(|_| Failure::from(Raised::system_resources()))?;
    Ok(Some(receiver))
}

/// `MutableBuffer::setTextRexx` (`classes/MutableBufferClass.cpp:355`)
/// reaching `setText` (`:369`): the contents become the argument's; answers
/// the receiver.
fn native_mutable_buffer_settext(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let new = string_method_argument(interp, args, 0)?;
    let state = buffer_state_mut(interp, receiver, b"SETTEXT")?;
    state.bytes.clear();
    buffer_capacity(state, new.len())?;
    state.bytes.extend_from_slice(&new);
    interp.give_result_buffer(new);
    Ok(Some(receiver))
}

/// `substr`'s arguments: a 0-based start, an optional length, and a pad
/// defaulting to a blank.
pub(super) fn substr_arguments(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
) -> Result<(usize, Option<usize>, u8), Failure> {
    let start = required_position_argument(interp, args, 0)? - 1;
    let length = optional_length_argument(interp, args, 1)?;
    let pad = pad_method_argument(interp, args, 2)?.unwrap_or(b' ');
    Ok((start, length, pad))
}

/// The optional length `C2D`, `D2C`, `D2X` and `X2D` take, as methods.
pub(super) fn conversion_length_argument(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
) -> Result<Option<usize>, Failure> {
    optional_length_argument(interp, args, 0)
}

/// A count argument that is zero or a positive whole number, answered as the
/// `i64` the numeric cores take.
pub(super) fn count_method_argument(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    index: usize,
) -> Result<Option<i64>, Failure> {
    let raise = move |found: &[u8]| Raised::argument_not_non_negative(index + 1, found);
    match whole_method_argument(interp, args, index, raise)? {
        Some(value) if value >= 0 => Ok(Some(value)),
        Some(_) => Err(refuse_method_argument(interp, args, index, raise)),
        None => Ok(None),
    }
}

/// `BITAND`/`BITOR`/`BITXOR`'s optional operand and optional pad.
pub(super) fn bit_arguments(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
) -> Result<(Vec<u8>, Option<u8>), Failure> {
    let other = optional_string_method_argument(interp, args, 0)?;
    let pad = pad_method_argument(interp, args, 1)?;
    Ok((other, pad))
}

/// `DATATYPE`'s option letter, or `None` when it is omitted.
pub(super) fn datatype_option_argument(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
) -> Result<Option<u8>, Failure> {
    let Some(value) = args.first().copied().flatten() else {
        return Ok(None);
    };
    let text = required_string_argument(interp, value, 1)?;
    let letter = interp
        .to_text(text)
        .first()
        .copied()
        .unwrap_or(0)
        .to_ascii_uppercase();
    Ok(Some(letter))
}

/// `STRIP`'s option and character set.
pub(super) fn strip_arguments(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
) -> Result<(u8, Option<Vec<u8>>), Failure> {
    let option = option_method_argument(interp, args, 0, "BLT")?.unwrap_or(b'B');
    let set = optional_string_or_none_argument(interp, args, 1)?;
    Ok((option, set))
}

/// `EQUALS`'s and `CASELESSEQUALS`'s one operand, as its **string value**.
pub(super) fn equals_argument(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
) -> Result<Vec<u8>, Failure> {
    let Some(value) = args.first().copied().flatten() else {
        return Err(Raised::missing_method_argument(1).into());
    };
    Ok(interp.string_value_text(value))
}

/// `COMPARETO`'s and `CASELESSCOMPARETO`'s three arguments.
pub(super) fn compare_to_arguments(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
) -> Result<(Vec<u8>, usize, Option<usize>), Failure> {
    let other = string_method_argument(interp, args, 0)?;
    let start = optional_position_argument(interp, args, 1)?.unwrap_or(1);
    let length = optional_length_argument(interp, args, 2)?;
    Ok((other, start, length))
}

/// `ABBREV`'s candidate and its minimum length.
pub(super) fn abbrev_arguments(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
) -> Result<(Vec<u8>, Option<usize>), Failure> {
    let info = string_method_argument(interp, args, 0)?;
    let minimum = optional_length_argument(interp, args, 1)?;
    Ok((info, minimum))
}

/// `COMPARE`'s other string and its pad.
pub(super) fn compare_arguments(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
) -> Result<(Vec<u8>, u8), Failure> {
    let other = string_method_argument(interp, args, 0)?;
    let pad = pad_method_argument(interp, args, 1)?.unwrap_or(b' ');
    Ok((other, pad))
}

/// A width and a pad: `CENTER`/`CENTRE`/`LEFT`/`RIGHT`'s two arguments as
/// methods.
pub(super) fn pad_arguments(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
) -> Result<(usize, u8), Failure> {
    let width = required_length_argument(interp, args, 0)?;
    let pad = pad_method_argument(interp, args, 1)?.unwrap_or(b' ');
    Ok((width, pad))
}

/// `COPIES`'s count: `nonNegativeArgument` (`classes/StringClassMisc.cpp:288`),
/// required.
pub(super) fn copies_argument(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
) -> Result<usize, Failure> {
    match optional_non_negative_argument(interp, args, 0)? {
        Some(count) => Ok(count),
        None => Err(Raised::missing_method_argument(1).into()),
    }
}

/// `verify`'s arguments: the reference set, the `M`/`N` option defaulting to
/// `N`, a 0-based start, and an optional range.
pub(super) fn verify_arguments(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
) -> Result<(Vec<u8>, u8, usize, Option<usize>), Failure> {
    let reference = string_method_argument(interp, args, 0)?;
    let option = option_method_argument(interp, args, 1, "MN")?.unwrap_or(b'N');
    let start = optional_position_argument(interp, args, 2)?.unwrap_or(1) - 1;
    let range = optional_length_argument(interp, args, 3)?;
    Ok((reference, option, start, range))
}

/// `MutableBuffer::substr` (`classes/MutableBufferClass.cpp:770`),
/// `StringUtil::substr`'s padding form.
fn native_mutable_buffer_substr(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let (start, length, pad) = substr_arguments(interp, args)?;
    let state = buffer_state(interp, receiver, b"SUBSTR")?;
    let mut out = interp.take_result_buffer();
    crate::builtin::string::substr_bytes(&mut out, &state.bytes, start, length, pad)?;
    Ok(Some(interp.text_built(out)))
}

/// `MutableBuffer::brackets` (`classes/MutableBufferClass.cpp:787`),
/// `StringUtil::substr`'s two-argument form: the length defaults to one
/// byte, is capped at the end of the contents, and nothing pads -- measured,
/// oracle rc 0: `.MutableBuffer~new('abcabc')[2]` is `b` and `[5, 10]` is `bc`.
fn native_mutable_buffer_brackets(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let start = required_position_argument(interp, args, 0)? - 1;
    let length = optional_length_argument(interp, args, 1)?.unwrap_or(1);
    let state = buffer_state(interp, receiver, b"[]")?;
    let capped = length.min(state.bytes.len().saturating_sub(start));
    let mut out = interp.take_result_buffer();
    crate::builtin::string::substr_bytes(&mut out, &state.bytes, start, Some(capped), b' ')?;
    Ok(Some(interp.text_built(out)))
}

/// `StringUtil::posRexx`'s argument handling: the needle, a 1-based start
/// defaulting to the first byte, and an optional range.
pub(super) fn forward_search_arguments(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
) -> Result<(Vec<u8>, usize, Option<usize>), Failure> {
    let needle = string_method_argument(interp, args, 0)?;
    let start = optional_position_argument(interp, args, 1)?.unwrap_or(1);
    let range = optional_length_argument(interp, args, 2)?;
    Ok((needle, start, range))
}

/// [`forward_search_arguments`]' search: the 1-based position of `needle`, or
/// 0, with an omitted range reaching the end of `haystack`.
pub(super) fn forward_search(
    haystack: &[u8],
    needle: &[u8],
    start: usize,
    range: Option<usize>,
    scan: fn(&[u8], &[u8], usize, usize) -> usize,
) -> usize {
    let range = range.unwrap_or(haystack.len().saturating_sub(start) + 1);
    scan(haystack, needle, start - 1, range)
}

/// `StringUtil::lastPosRexx`'s argument handling: the needle, and a start and
/// range that each default to the receiver's whole length rather than to a
/// fixed number, so both stay `None` here.
pub(super) fn backward_search_arguments(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
) -> Result<(Vec<u8>, Option<usize>, Option<usize>), Failure> {
    let needle = string_method_argument(interp, args, 0)?;
    let start = optional_position_argument(interp, args, 1)?;
    let range = optional_length_argument(interp, args, 2)?;
    Ok((needle, start, range))
}

/// [`backward_search_arguments`]' search, with both defaults taken from
/// `haystack`.
pub(super) fn backward_search(
    haystack: &[u8],
    needle: &[u8],
    start: Option<usize>,
    range: Option<usize>,
    scan: fn(&[u8], &[u8], usize, usize) -> usize,
) -> usize {
    let length = haystack.len();
    scan(
        haystack,
        needle,
        start.unwrap_or(length),
        range.unwrap_or(length),
    )
}

/// [`forward_search`] over a buffer's contents, for the `MutableBuffer` rows
/// that answer a position: `POS` and `CONTAINS`
/// (`classes/MutableBufferClass.cpp:803`, `:819`) and their caseless twins
/// (`:851`, `:869`).
fn buffer_pos(
    interp: &mut Interp,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
    name: &[u8],
    scan: fn(&[u8], &[u8], usize, usize) -> usize,
) -> Result<usize, Failure> {
    let (needle, start, range) = forward_search_arguments(interp, args)?;
    let state = buffer_state(interp, receiver, name)?;
    let found = forward_search(&state.bytes, &needle, start, range, scan);
    interp.give_result_buffer(needle);
    Ok(found)
}

/// `MutableBuffer::posRexx` (`classes/MutableBufferClass.cpp:803`).
fn native_mutable_buffer_pos(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let found = buffer_pos(
        interp,
        receiver,
        args,
        b"POS",
        crate::builtin::string::find_forward,
    )?;
    Ok(Some(interp.counted(found)))
}

/// `MutableBuffer::containsRexx` (`classes/MutableBufferClass.cpp:819`).
fn native_mutable_buffer_contains(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let found = buffer_pos(
        interp,
        receiver,
        args,
        b"CONTAINS",
        crate::builtin::string::find_forward,
    )?;
    Ok(Some(interp.counted(usize::from(found > 0))))
}

/// `MutableBuffer::caselessPos` (`classes/MutableBufferClass.cpp:851`).
fn native_mutable_buffer_caselesspos(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let found = buffer_pos(
        interp,
        receiver,
        args,
        b"CASELESSPOS",
        crate::builtin::string::caseless_find_forward,
    )?;
    Ok(Some(interp.counted(found)))
}

/// `MutableBuffer::caselessContains` (`classes/MutableBufferClass.cpp:869`).
fn native_mutable_buffer_caselesscontains(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let found = buffer_pos(
        interp,
        receiver,
        args,
        b"CASELESSCONTAINS",
        crate::builtin::string::caseless_find_forward,
    )?;
    Ok(Some(interp.counted(usize::from(found > 0))))
}

/// `MutableBuffer::lastPos` (`classes/MutableBufferClass.cpp:835`) through
/// `StringUtil::lastPosRexx`: the start and the range each default to the
/// whole length.
fn native_mutable_buffer_lastpos(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let (needle, start, range) = backward_search_arguments(interp, args)?;
    let state = buffer_state(interp, receiver, b"LASTPOS")?;
    let found = backward_search(
        &state.bytes,
        &needle,
        start,
        range,
        crate::builtin::string::find_backward,
    );
    interp.give_result_buffer(needle);
    Ok(Some(interp.counted(found)))
}

/// `MutableBuffer::caselessLastPos` (`classes/MutableBufferClass.cpp:890`):
/// `LASTPOS`'s scan and defaults, the compare folded.
fn native_mutable_buffer_caselesslastpos(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let (needle, start, range) = backward_search_arguments(interp, args)?;
    let state = buffer_state(interp, receiver, b"CASELESSLASTPOS")?;
    let found = backward_search(
        &state.bytes,
        &needle,
        start,
        range,
        crate::builtin::string::caseless_find_backward,
    );
    interp.give_result_buffer(needle);
    Ok(Some(interp.counted(found)))
}

/// `MutableBuffer::countStrRexx` (`classes/MutableBufferClass.cpp:940`).
fn native_mutable_buffer_countstr(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let needle = string_method_argument(interp, args, 0)?;
    let state = buffer_state(interp, receiver, b"COUNTSTR")?;
    let count = crate::builtin::string::count_occurrences(&state.bytes, &needle, usize::MAX);
    interp.give_result_buffer(needle);
    Ok(Some(interp.counted(count)))
}

/// `MutableBuffer::caselessCountStrRexx` (`classes/MutableBufferClass.cpp:955`).
fn native_mutable_buffer_caselesscountstr(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let needle = string_method_argument(interp, args, 0)?;
    let state = buffer_state(interp, receiver, b"CASELESSCOUNTSTR")?;
    let count =
        crate::builtin::string::caseless_count_occurrences(&state.bytes, &needle, usize::MAX);
    interp.give_result_buffer(needle);
    Ok(Some(interp.counted(count)))
}

/// `MutableBuffer::verify` (`classes/MutableBufferClass.cpp:1744`) through
/// `StringUtil::verify`, which builds an integer on every path -- a start past
/// the end included, where the builtin answers text.
fn native_mutable_buffer_verify(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let (reference, option, start, range) = verify_arguments(interp, args)?;
    let state = buffer_state(interp, receiver, b"VERIFY")?;
    let answer =
        crate::builtin::string::verify_bytes(&state.bytes, &reference, option, start, range);
    interp.give_result_buffer(reference);
    Ok(Some(interp.counted(answer)))
}

/// `MutableBuffer::subWord` (`classes/MutableBufferClass.cpp:1758`).
fn native_mutable_buffer_subword(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let position = required_position_argument(interp, args, 0)?;
    let count = optional_length_argument(interp, args, 1)?;
    let state = buffer_state(interp, receiver, b"SUBWORD")?;
    let found = crate::builtin::word::subword_range(&state.bytes, position, count);
    let mut out = interp.take_result_buffer();
    out.extend_from_slice(&state.bytes[found]);
    Ok(Some(interp.text_built(out)))
}

/// A fresh `Array` holding one string per element of `pieces`.
pub(super) fn array_of_texts(interp: &mut Interp, pieces: Vec<Vec<u8>>) -> Result<ObjRef, Failure> {
    let frame = interp.roots.push_frame();
    let mut slots = Vec::with_capacity(pieces.len());
    for piece in pieces {
        let value = interp.text_built(piece);
        interp.roots.push_temp(value);
        slots.push(Some(value));
    }
    let object = interp.alloc_with(
        BehaviourId::ARRAY,
        Body::Array {
            dimensions: None,
            slots,
        },
    );
    interp.roots.pop_frame(frame);
    interp.roots.push_temp(object);
    let caller = interp.caller();
    interp.send_message(object, INIT, None, &[], caller)?;
    Ok(object)
}

/// `MutableBuffer::subWords` (`classes/MutableBufferClass.cpp:1778`) over
/// `StringUtil::subWords` (`classes/support/StringUtil.cpp:1405`): an
/// **`Array`** of the words from `position`, at most `count` of them.
fn native_mutable_buffer_subwords(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let position = optional_position_argument(interp, args, 0)?.unwrap_or(1);
    let count = optional_length_argument(interp, args, 1)?.unwrap_or(usize::MAX);
    let state = buffer_state(interp, receiver, b"SUBWORDS")?;
    let words: Vec<Vec<u8>> = crate::builtin::word::word_slices(&state.bytes)
        .into_iter()
        .skip(position - 1)
        .take(count)
        .map(<[u8]>::to_vec)
        .collect();
    Ok(Some(array_of_texts(interp, words)?))
}

/// `MutableBuffer::makeArrayRexx` (`classes/MutableBufferClass.cpp:927`) over
/// `StringUtil::makearray` (`classes/support/StringUtil.cpp:545`): an
/// `Array` of the contents split on line ends, or on `separator` where one is
/// given.
fn native_mutable_buffer_makearray(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let separator = optional_string_or_none_argument(interp, args, 0)?;
    let state = buffer_state(interp, receiver, b"MAKEARRAY")?;
    let pieces = match &separator {
        Some(separator) => crate::builtin::string::split_slices(&state.bytes, separator),
        None => crate::builtin::string::line_slices(&state.bytes),
    };
    let pieces: Vec<Vec<u8>> = pieces.into_iter().map(<[u8]>::to_vec).collect();
    Ok(Some(array_of_texts(interp, pieces)?))
}

/// `MutableBuffer::word` (`classes/MutableBufferClass.cpp:1791`).
fn native_mutable_buffer_word(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let position = required_position_argument(interp, args, 0)?;
    let state = buffer_state(interp, receiver, b"WORD")?;
    let found = crate::builtin::word::word_range(&state.bytes, position).unwrap_or(0..0);
    let mut out = interp.take_result_buffer();
    out.extend_from_slice(&state.bytes[found]);
    Ok(Some(interp.text_built(out)))
}

/// `MutableBuffer::wordIndex` (`classes/MutableBufferClass.cpp:1805`).
fn native_mutable_buffer_wordindex(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let position = required_position_argument(interp, args, 0)?;
    let state = buffer_state(interp, receiver, b"WORDINDEX")?;
    let index =
        crate::builtin::word::word_range(&state.bytes, position).map_or(0, |word| word.start + 1);
    Ok(Some(interp.counted(index)))
}

/// `MutableBuffer::wordLength` (`classes/MutableBufferClass.cpp:1820`).
fn native_mutable_buffer_wordlength(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let position = required_position_argument(interp, args, 0)?;
    let state = buffer_state(interp, receiver, b"WORDLENGTH")?;
    let length =
        crate::builtin::word::word_range(&state.bytes, position).map_or(0, |word| word.len());
    Ok(Some(interp.counted(length)))
}

/// `MutableBuffer::words` (`classes/MutableBufferClass.cpp:1830`).
fn native_mutable_buffer_words(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let count = crate::builtin::word::word_count(&buffer_state(interp, receiver, b"WORDS")?.bytes);
    Ok(Some(interp.counted(count)))
}

/// `StringUtil::wordPos`'s argument handling: the phrase, and a 1-based start
/// defaulting to the first word.
pub(super) fn wordpos_arguments(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
) -> Result<(Vec<u8>, usize), Failure> {
    let phrase = string_method_argument(interp, args, 0)?;
    let start = optional_position_argument(interp, args, 1)?.unwrap_or(1);
    Ok((phrase, start))
}

/// [`wordpos_arguments`]' search over a buffer's contents, shared by
/// `WORDPOS` and `CONTAINSWORD` (`classes/MutableBufferClass.cpp:1845`,
/// `:1859`) and their caseless twins (`:1873`, `:1887`).
fn buffer_wordpos(
    interp: &mut Interp,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
    name: &[u8],
    scan: fn(&[u8], &[u8], usize) -> usize,
) -> Result<usize, Failure> {
    let (phrase, start) = wordpos_arguments(interp, args)?;
    let state = buffer_state(interp, receiver, name)?;
    let found = scan(&phrase, &state.bytes, start);
    interp.give_result_buffer(phrase);
    Ok(found)
}

/// `MutableBuffer::wordPos` (`classes/MutableBufferClass.cpp:1845`).
fn native_mutable_buffer_wordpos(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let found = buffer_wordpos(
        interp,
        receiver,
        args,
        b"WORDPOS",
        crate::builtin::word::wordpos_bytes,
    )?;
    Ok(Some(interp.counted(found)))
}

/// `MutableBuffer::containsWord` (`classes/MutableBufferClass.cpp:1859`).
fn native_mutable_buffer_containsword(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let found = buffer_wordpos(
        interp,
        receiver,
        args,
        b"CONTAINSWORD",
        crate::builtin::word::wordpos_bytes,
    )?;
    Ok(Some(interp.counted(usize::from(found > 0))))
}

/// `MutableBuffer::caselessWordPos` (`classes/MutableBufferClass.cpp:1873`).
fn native_mutable_buffer_caselesswordpos(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let found = buffer_wordpos(
        interp,
        receiver,
        args,
        b"CASELESSWORDPOS",
        crate::builtin::word::caseless_wordpos_bytes,
    )?;
    Ok(Some(interp.counted(found)))
}

/// `MutableBuffer::caselessContainsWord` (`classes/MutableBufferClass.cpp:1887`).
fn native_mutable_buffer_caselesscontainsword(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let found = buffer_wordpos(
        interp,
        receiver,
        args,
        b"CASELESSCONTAINSWORD",
        crate::builtin::word::caseless_wordpos_bytes,
    )?;
    Ok(Some(interp.counted(usize::from(found > 0))))
}

/// `STARTSWITH` and its caseless twin over any bytes: an empty `match`
/// answers `0` on both, which is why this is not `slice::starts_with`.
pub(super) fn starts_with(
    haystack: &[u8],
    needle: &[u8],
    matches: fn(&[u8], &[u8]) -> bool,
) -> bool {
    !needle.is_empty()
        && haystack
            .get(..needle.len())
            .is_some_and(|front| matches(front, needle))
}

/// [`starts_with`] from the other end.
pub(super) fn ends_with(haystack: &[u8], needle: &[u8], matches: fn(&[u8], &[u8]) -> bool) -> bool {
    !needle.is_empty()
        && haystack
            .len()
            .checked_sub(needle.len())
            .is_some_and(|at| matches(&haystack[at..], needle))
}

/// `MutableBuffer::startsWithRexx` (`classes/MutableBufferClass.cpp:1541`):
/// `ENDSWITH`'s twin, an empty `match` likewise `0`.
fn native_mutable_buffer_startswith(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let needle = named_string_argument(interp, args, 0, "match")?;
    let state = buffer_state(interp, receiver, b"STARTSWITH")?;
    let answer = starts_with(&state.bytes, &needle, <[u8]>::eq);
    interp.give_result_buffer(needle);
    Ok(Some(interp.counted(usize::from(answer))))
}

/// `MutableBuffer::caselessStartsWithRexx` (`classes/MutableBufferClass.cpp:1555`):
/// `STARTSWITH`'s region compare, folded, an empty `match` likewise `0`.
fn native_mutable_buffer_caselessstartswith(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let needle = named_string_argument(interp, args, 0, "match")?;
    let state = buffer_state(interp, receiver, b"CASELESSSTARTSWITH")?;
    let answer = starts_with(&state.bytes, &needle, crate::builtin::string::caseless_eq);
    interp.give_result_buffer(needle);
    Ok(Some(interp.counted(usize::from(answer))))
}

/// `MutableBuffer::match` (`classes/MutableBufferClass.cpp:1460`): a start
/// past the contents is `0` before `other` is looked at -- measured, oracle
/// rc 0: `.MutableBuffer~new('abcabc')~match(99, .nil)` is `0`.
fn native_mutable_buffer_match(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let start = required_position_argument(interp, args, 0)?;
    if start > buffer_state(interp, receiver, b"MATCH")?.bytes.len() {
        return Ok(Some(interp.counted(0)));
    }
    let other = string_method_argument(interp, args, 1)?;
    let answer = match_region(interp, receiver, args, start, &other, b"MATCH", <[u8]>::eq);
    interp.give_result_buffer(other);
    Ok(Some(interp.counted(usize::from(answer?))))
}

/// `MutableBuffer::caselessMatch` (`classes/MutableBufferClass.cpp:1505`):
/// `MATCH`'s scan, `primitiveCaselessMatch` (`:1642`) the only difference,
/// and the same order -- measured, oracle rc 0:
/// `.MutableBuffer~new('aBcaBc')~caselessMatch(99, .nil)` is `0`.
fn native_mutable_buffer_caselessmatch(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let start = required_position_argument(interp, args, 0)?;
    if start
        > buffer_state(interp, receiver, b"CASELESSMATCH")?
            .bytes
            .len()
    {
        return Ok(Some(interp.counted(0)));
    }
    let other = string_method_argument(interp, args, 1)?;
    let answer = match_region(
        interp,
        receiver,
        args,
        start,
        &other,
        b"CASELESSMATCH",
        crate::builtin::string::caseless_eq,
    );
    interp.give_result_buffer(other);
    Ok(Some(interp.counted(usize::from(answer?))))
}

/// The rest of `match`'s arguments once `start` is inside the receiver: an
/// explicit offset past `other` is `0` before the length is looked at, and a
/// length past `other` is `0` -- measured, oracle rc 0: `~match(1, 'abc', 4,
/// -1)` is `0`. `None` is those two answers; `Some` is the region of `other`
/// to compare.
pub(super) fn match_region_arguments(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    other_len: usize,
) -> Result<Option<(usize, usize)>, Failure> {
    let offset = optional_position_argument(interp, args, 2)?;
    if offset.is_some_and(|offset| offset > other_len) {
        return Ok(None);
    }
    let offset = offset.unwrap_or(1);
    let length = optional_length_argument(interp, args, 3)?.unwrap_or(other_len + 1 - offset);
    if length == 0 || offset.saturating_add(length) - 1 > other_len {
        return Ok(None);
    }
    Ok(Some((offset, length)))
}

/// [`match_region_arguments`]' comparison, once the receiver's bytes are in
/// hand: `primitiveMatch` (`classes/MutableBufferClass.cpp:1615`) compares the
/// two regions, and `matches` is what separates the two spellings.
pub(super) fn match_region_over(
    haystack: &[u8],
    start: usize,
    other: &[u8],
    region: Option<(usize, usize)>,
    matches: fn(&[u8], &[u8]) -> bool,
) -> bool {
    let Some((offset, length)) = region else {
        return false;
    };
    haystack
        .get(start - 1..(start - 1).saturating_add(length))
        .is_some_and(|region| matches(region, &other[offset - 1..offset - 1 + length]))
}

/// [`match_region_arguments`] and [`match_region_over`] over a buffer's
/// contents.
fn match_region(
    interp: &mut Interp,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
    start: usize,
    other: &[u8],
    name: &[u8],
    matches: fn(&[u8], &[u8]) -> bool,
) -> Result<bool, Failure> {
    let region = match_region_arguments(interp, args, other.len())?;
    let state = buffer_state(interp, receiver, name)?;
    Ok(match_region_over(
        &state.bytes,
        start,
        other,
        region,
        matches,
    ))
}

/// `MutableBuffer::matchChar` (`classes/MutableBufferClass.cpp:1668`): a
/// position past the contents is `0` before the set is looked at -- measured,
/// oracle rc 0: `.MutableBuffer~new('abcabc')~matchChar(99, .nil)` is `0`.
fn native_mutable_buffer_matchchar(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let position = required_position_argument(interp, args, 0)?;
    if position > buffer_state(interp, receiver, b"MATCHCHAR")?.bytes.len() {
        return Ok(Some(interp.counted(0)));
    }
    let set = string_method_argument(interp, args, 1)?;
    let state = buffer_state(interp, receiver, b"MATCHCHAR")?;
    let answer = state
        .bytes
        .get(position - 1)
        .is_some_and(|byte| set.contains(byte));
    interp.give_result_buffer(set);
    Ok(Some(interp.counted(usize::from(answer))))
}

/// `MutableBuffer::caselessMatchChar` (`classes/MutableBufferClass.cpp:1705`):
/// `MATCHCHAR`'s scan with both sides folded, and the same order -- measured,
/// oracle rc 0: `.MutableBuffer~new('aBcaBc')~caselessMatchChar(99, .nil)` is
/// `0`.
fn native_mutable_buffer_caselessmatchchar(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let position = required_position_argument(interp, args, 0)?;
    if position
        > buffer_state(interp, receiver, b"CASELESSMATCHCHAR")?
            .bytes
            .len()
    {
        return Ok(Some(interp.counted(0)));
    }
    let set = string_method_argument(interp, args, 1)?;
    let state = buffer_state(interp, receiver, b"CASELESSMATCHCHAR")?;
    let answer = state.bytes.get(position - 1).is_some_and(|byte| {
        let byte = byte.to_ascii_uppercase();
        set.iter().any(|member| member.to_ascii_uppercase() == byte)
    });
    interp.give_result_buffer(set);
    Ok(Some(interp.counted(usize::from(answer))))
}

/// `MutableBuffer::subchar` (`classes/MutableBufferClass.cpp:914`): the one
/// byte at the position, or the null string past the end.
fn native_mutable_buffer_subchar(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let position = required_position_argument(interp, args, 0)?;
    let state = buffer_state(interp, receiver, b"SUBCHAR")?;
    let mut out = interp.take_result_buffer();
    out.extend(state.bytes.get(position - 1));
    Ok(Some(interp.text_built(out)))
}

/// A mutator's rebuilt contents written back over the receiver's own, the
/// capacity already raised for them.
fn replace_buffer_contents(state: &mut BufferState, built: &[u8]) {
    state.bytes.clear();
    state.bytes.extend_from_slice(built);
}

/// [`BufferState::ensure_capacity`]'s refusal, the oracle's 5.1.
fn buffer_capacity(state: &mut BufferState, added: usize) -> Result<(), Failure> {
    state
        .ensure_capacity(added)
        .map_err(|_| Failure::from(Raised::system_resources()))
}

/// `changeStr`'s arguments: needle, replacement, and a count defaulting to
/// every occurrence.
pub(super) fn changestr_arguments(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
) -> Result<(Vec<u8>, Vec<u8>, usize), Failure> {
    let needle = string_method_argument(interp, args, 0)?;
    let replacement = string_method_argument(interp, args, 1)?;
    let limit = optional_non_negative_argument(interp, args, 2)?.unwrap_or(usize::MAX);
    Ok((needle, replacement, limit))
}

/// `space`'s arguments: the gap between words, defaulting to one, and a pad.
pub(super) fn space_arguments(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
) -> Result<(usize, u8), Failure> {
    let gap = optional_length_argument(interp, args, 0)?.unwrap_or(1);
    let pad = pad_method_argument(interp, args, 1)?.unwrap_or(b' ');
    Ok((gap, pad))
}

/// The `(start, range)` a case shift takes, from `first` onwards.
pub(super) fn case_shift_arguments(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    first: usize,
) -> Result<(usize, Option<usize>), Failure> {
    let start = optional_position_argument(interp, args, first)?.unwrap_or(1) - 1;
    let range = optional_length_argument(interp, args, first + 1)?;
    Ok((start, range))
}

/// `translate`'s arguments: the output table, the input table, a pad, a
/// 0-based start and an optional range.
pub(super) fn translate_arguments(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
) -> Result<(Vec<u8>, Vec<u8>, u8, usize, Option<usize>), Failure> {
    let out_table = optional_string_method_argument(interp, args, 0)?;
    let in_table = optional_string_method_argument(interp, args, 1)?;
    let pad = pad_method_argument(interp, args, 2)?.unwrap_or(b' ');
    let start = optional_position_argument(interp, args, 3)?.unwrap_or(1) - 1;
    let range = optional_length_argument(interp, args, 4)?;
    Ok((out_table, in_table, pad, start, range))
}

/// An empty input table means "every byte", which the core spells `None`.
pub(super) fn translate_in_table(in_table: &[u8]) -> Option<&[u8]> {
    if in_table.is_empty() {
        None
    } else {
        Some(in_table)
    }
}

/// `insert`'s arguments: the string, a 0-based begin defaulting to the front,
/// an optional length and a pad.
pub(super) fn insert_arguments(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
) -> Result<(Vec<u8>, usize, Option<usize>, u8), Failure> {
    let new = string_method_argument(interp, args, 0)?;
    let begin = optional_non_negative_argument(interp, args, 1)?.unwrap_or(0);
    let length = optional_length_argument(interp, args, 2)?;
    let pad = pad_method_argument(interp, args, 3)?.unwrap_or(b' ');
    Ok((new, begin, length, pad))
}

/// `overlay`'s arguments, which are [`insert_arguments`]' with a 1-based
/// position in place of the non-negative offset.
pub(super) fn overlay_arguments(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
) -> Result<(Vec<u8>, usize, Option<usize>, u8), Failure> {
    let new = string_method_argument(interp, args, 0)?;
    let begin = optional_position_argument(interp, args, 1)?.unwrap_or(1) - 1;
    let length = optional_length_argument(interp, args, 2)?;
    let pad = pad_method_argument(interp, args, 3)?.unwrap_or(b' ');
    Ok((new, begin, length, pad))
}

/// `delStr`'s arguments: a 0-based begin defaulting to the front and an
/// optional range.
pub(super) fn delete_arguments(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
) -> Result<(usize, Option<usize>), Failure> {
    let begin = optional_position_argument(interp, args, 0)?.unwrap_or(1) - 1;
    let range = optional_length_argument(interp, args, 1)?;
    Ok((begin, range))
}

/// `delWord`'s arguments: a required 1-based word position and an optional
/// count.
pub(super) fn delword_arguments(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
) -> Result<(usize, Option<usize>), Failure> {
    let position = required_position_argument(interp, args, 0)?;
    let count = optional_length_argument(interp, args, 1)?;
    Ok((position, count))
}

/// How much of the receiver `replaceAt` overwrites, and how long the answer
/// is: `(replaced, final_length)`.
pub(super) fn replace_at_plan(
    contents: usize,
    begin: usize,
    length: Option<usize>,
    new_len: usize,
) -> (usize, usize) {
    let mut replaced = length.unwrap_or(new_len);
    if begin > contents {
        replaced = 0;
    } else if begin.saturating_add(replaced) > contents {
        replaced = contents - begin;
    }
    let kept = if begin > contents { begin } else { contents };
    (replaced, kept - replaced + new_len)
}

/// [`replace_at_plan`]'s assembly: the front padded out to `begin`, then
/// `new`, then whatever the replacement did not cover.
pub(super) fn replace_at_bytes(
    out: &mut Vec<u8>,
    bytes: &[u8],
    new: &[u8],
    begin: usize,
    replaced: usize,
    pad: u8,
) -> Result<(), Failure> {
    crate::builtin::string::substr_bytes(out, bytes, 0, Some(begin), pad)?;
    out.extend_from_slice(new);
    out.extend_from_slice(&bytes[(begin + replaced).min(bytes.len())..]);
    Ok(())
}

/// `MutableBuffer::insert` (`classes/MutableBufferClass.cpp:420`): the
/// position is a 0-based count defaulting to 0, a position past the contents
/// pads the gap, and the length pads the insertion; answers the receiver.
fn native_mutable_buffer_insert(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let (new, begin, length, pad) = insert_arguments(interp, args)?;
    let mut out = interp.take_result_buffer();
    let state = buffer_state_mut(interp, receiver, b"INSERT")?;
    let insert_length = length.unwrap_or(new.len());
    let added = insert_length.saturating_add(begin.saturating_sub(state.bytes.len()));
    buffer_capacity(state, added)?;
    crate::builtin::string::insert_bytes(&mut out, &state.bytes, &new, begin, length, pad)?;
    replace_buffer_contents(state, &out);
    interp.give_result_buffer(out);
    Ok(Some(receiver))
}

/// `MutableBuffer::overlay` (`classes/MutableBufferClass.cpp:493`): the
/// position is 1-based and defaults to 1, and the capacity is raised for the
/// position plus the overlay length rather than for the result; answers the
/// receiver.
fn native_mutable_buffer_overlay(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let (new, begin, length, pad) = overlay_arguments(interp, args)?;
    let mut out = interp.take_result_buffer();
    let state = buffer_state_mut(interp, receiver, b"OVERLAY")?;
    let overlay_length = length.unwrap_or(new.len());
    buffer_capacity(state, begin.saturating_add(overlay_length))?;
    crate::builtin::string::overlay_bytes(&mut out, &state.bytes, &new, begin, length, pad)?;
    replace_buffer_contents(state, &out);
    interp.give_result_buffer(out);
    Ok(Some(receiver))
}

/// `MutableBuffer::replaceAt` (`classes/MutableBufferClass.cpp:570`): the
/// range from the 1-based position is excised and the replacement spliced in
/// whole, so the contents shift where the two lengths differ; answers the
/// receiver.
fn buffer_replace_at(
    interp: &mut Interp,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
    name: &[u8],
) -> Result<Option<ObjRef>, Failure> {
    let new = named_string_argument(interp, args, 0, "new")?;
    let begin = named_position_argument(interp, args, 1, "position")? - 1;
    let length = optional_named_length_argument(interp, args, 2, "length")?;
    let pad = named_pad_argument(interp, args, 3, "pad")?.unwrap_or(b' ');
    let mut out = interp.take_result_buffer();
    let state = buffer_state_mut(interp, receiver, name)?;
    let (replaced, final_length) = replace_at_plan(state.bytes.len(), begin, length, new.len());
    buffer_capacity(state, final_length)?;
    out.try_reserve(final_length)
        .map_err(|_| Failure::from(Raised::system_resources()))?;
    replace_at_bytes(&mut out, &state.bytes, &new, begin, replaced, pad)?;
    replace_buffer_contents(state, &out);
    interp.give_result_buffer(out);
    Ok(Some(receiver))
}

/// `MutableBuffer~replaceAt`, [`buffer_replace_at`].
fn native_mutable_buffer_replaceat(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    buffer_replace_at(interp, receiver, args, b"REPLACEAT")
}

/// `MutableBuffer~'[]='`, [`buffer_replace_at`].
fn native_mutable_buffer_bracketsequal(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    buffer_replace_at(interp, receiver, args, b"[]=")
}

/// `MutableBuffer::changeStr` (`classes/MutableBufferClass.cpp:971`): answers
/// the receiver, and raises the capacity only on the branch where the
/// replacement is longer than the needle (`:1081`).
fn native_mutable_buffer_changestr(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let (needle, replacement, limit) = changestr_arguments(interp, args)?;
    let mut out = interp.take_result_buffer();
    let state = buffer_state_mut(interp, receiver, b"CHANGESTR")?;
    if !needle.is_empty() && limit > 0 && replacement.len() > needle.len() {
        let matches = crate::builtin::string::count_occurrences(&state.bytes, &needle, limit);
        if matches > 0 {
            let growth = matches.saturating_mul(replacement.len() - needle.len());
            let result_length = state.bytes.len().saturating_add(growth);
            buffer_capacity(state, result_length)?;
        }
    }
    crate::builtin::string::changestr_bytes(&mut out, &state.bytes, &needle, &replacement, limit)?;
    replace_buffer_contents(state, &out);
    interp.give_result_buffer(out);
    Ok(Some(receiver))
}

/// `MutableBuffer::caselessChangeStr` (`classes/MutableBufferClass.cpp:1136`):
/// `CHANGESTR`'s three length branches with the search folded, and the same
/// capacity rule -- only the growing branch calls `ensureCapacity`, and what
/// it passes is the result's own length. Measured, oracle rc 0:
/// `.MutableBuffer~new('aBc', 10)~caselessChangeStr('B', copies('q', 40))`
/// reads `42 45`, where a buffer built at the 256 default reads `404 512`.
fn native_mutable_buffer_caselesschangestr(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let (needle, replacement, limit) = changestr_arguments(interp, args)?;
    let mut out = interp.take_result_buffer();
    let state = buffer_state_mut(interp, receiver, b"CASELESSCHANGESTR")?;
    if !needle.is_empty() && limit > 0 && replacement.len() > needle.len() {
        let matches =
            crate::builtin::string::caseless_count_occurrences(&state.bytes, &needle, limit);
        if matches > 0 {
            let growth = matches.saturating_mul(replacement.len() - needle.len());
            let result_length = state.bytes.len().saturating_add(growth);
            buffer_capacity(state, result_length)?;
        }
    }
    crate::builtin::string::caseless_changestr_bytes(
        &mut out,
        &state.bytes,
        &needle,
        &replacement,
        limit,
    )?;
    replace_buffer_contents(state, &out);
    interp.give_result_buffer(out);
    Ok(Some(receiver))
}

/// `MutableBuffer::upper` and `::lower` (`classes/MutableBufferClass.cpp:1341`,
/// `:1301`), and `::translate`'s no-table form: the bytes within the range
/// case-shifted in place, with `first` the argument index the position is
/// read from; answers the receiver.
fn buffer_case_shift(
    interp: &mut Interp,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
    first: usize,
    name: &[u8],
    shift: fn(&u8) -> u8,
) -> Result<Option<ObjRef>, Failure> {
    let (start, range) = case_shift_arguments(interp, args, first)?;
    let state = buffer_state_mut(interp, receiver, name)?;
    crate::builtin::string::case_shift_bytes(&mut state.bytes, start, range, shift);
    Ok(Some(receiver))
}

/// `MutableBuffer~upper`, [`buffer_case_shift`].
fn native_mutable_buffer_upper(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    buffer_case_shift(interp, receiver, args, 0, b"UPPER", u8::to_ascii_uppercase)
}

/// `MutableBuffer~lower`, [`buffer_case_shift`].
fn native_mutable_buffer_lower(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    buffer_case_shift(interp, receiver, args, 0, b"LOWER", u8::to_ascii_lowercase)
}

/// `MutableBuffer::translate` (`classes/MutableBufferClass.cpp:1382`): each
/// byte within the range that the input table holds -- or every byte, read as
/// its own index, where that table is the null string -- becomes the output
/// table's byte at that index, or the pad past its end; answers the receiver.
fn native_mutable_buffer_translate(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if args.iter().take(3).all(Option::is_none) {
        return buffer_case_shift(
            interp,
            receiver,
            args,
            3,
            b"TRANSLATE",
            u8::to_ascii_uppercase,
        );
    }
    let (out_table, in_table, pad, start, range) = translate_arguments(interp, args)?;
    let state = buffer_state_mut(interp, receiver, b"TRANSLATE")?;
    crate::builtin::string::translate_bytes(
        &mut state.bytes,
        &out_table,
        translate_in_table(&in_table),
        pad,
        start,
        range,
    );
    Ok(Some(receiver))
}

/// `MutableBuffer::space` (`classes/MutableBufferClass.cpp:1962`): the words
/// rejoined by the pad, and the capacity raised from the single-blank form's
/// length rather than from the original (`:2031`, `:2038`); answers the
/// receiver.
fn native_mutable_buffer_space(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let (gap, pad) = space_arguments(interp, args)?;
    let mut out = interp.take_result_buffer();
    let state = buffer_state_mut(interp, receiver, b"SPACE")?;
    crate::builtin::string::space_bytes(&mut out, &state.bytes, gap, pad)?;
    let gaps = crate::builtin::word::word_count(&state.bytes).saturating_sub(1);
    let growth = gaps.saturating_mul(gap.saturating_sub(1));
    state.bytes.truncate(out.len() - growth);
    buffer_capacity(state, growth)?;
    replace_buffer_contents(state, &out);
    interp.give_result_buffer(out);
    Ok(Some(receiver))
}

/// `MutableBuffer::delWord` (`classes/MutableBufferClass.cpp:1901`): the words
/// from the 1-based position, with the blanks after the last of them, removed
/// in place; answers the receiver.
fn native_mutable_buffer_delword(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let (position, count) = delword_arguments(interp, args)?;
    let state = buffer_state_mut(interp, receiver, b"DELWORD")?;
    crate::builtin::word::delword_bytes(&mut state.bytes, position, count);
    Ok(Some(receiver))
}
