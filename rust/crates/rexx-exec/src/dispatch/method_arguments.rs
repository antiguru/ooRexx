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

//! The method-argument parsers the primitive methods share: the conversions
//! of `runtime/MethodArguments.hpp`, and the precision they convert at.

use super::{
    Failure, Interp, ObjRef, Raised, required_string_argument, required_string_named_argument,
};

/// `optionalLengthArgument` (`runtime/MethodArguments.hpp:327`): `None` for
/// an omitted argument, and anything that is not a non-negative whole number
/// in range is 93.923.
pub(super) fn optional_length_argument(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    index: usize,
) -> Result<Option<usize>, Failure> {
    match whole_method_argument(interp, args, index, Raised::invalid_length)? {
        Some(size) if size >= 0 => Ok(Some(usize_or_refuse(
            interp,
            args,
            index,
            size,
            Raised::invalid_length,
        )?)),
        Some(_) => Err(refuse_method_argument(
            interp,
            args,
            index,
            Raised::invalid_length,
        )),
        None => Ok(None),
    }
}

/// `lengthArgument` (`runtime/MethodArguments.hpp`): [`optional_length_argument`]
/// with an omitted argument 93.903 -- measured, oracle rc 163:
/// `.MutableBuffer~new('abc')~setBufferSize` reports `argument 1 is required`.
pub(super) fn required_length_argument(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    index: usize,
) -> Result<usize, Failure> {
    match optional_length_argument(interp, args, index)? {
        Some(size) => Ok(size),
        None => Err(Raised::missing_method_argument(index + 1).into()),
    }
}

/// `optionalPositionArgument` (`runtime/MethodArguments.hpp:387`): `None` for
/// an omitted argument, and anything that is not a positive whole number in
/// range is 93.924.
pub(super) fn optional_position_argument(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    index: usize,
) -> Result<Option<usize>, Failure> {
    match whole_method_argument(interp, args, index, Raised::invalid_position)? {
        Some(position) if position > 0 => Ok(Some(usize_or_refuse(
            interp,
            args,
            index,
            position,
            Raised::invalid_position,
        )?)),
        Some(_) => Err(refuse_method_argument(
            interp,
            args,
            index,
            Raised::invalid_position,
        )),
        None => Ok(None),
    }
}

/// `positionArgument` (`runtime/MethodArguments.hpp`): [`optional_position_argument`]
/// with an omitted argument 93.903 -- measured, oracle rc 163:
/// `.MutableBuffer~new('abcabc')~substr` reports `argument 1 is required`.
pub(super) fn required_position_argument(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    index: usize,
) -> Result<usize, Failure> {
    match optional_position_argument(interp, args, index)? {
        Some(position) => Ok(position),
        None => Err(Raised::missing_method_argument(index + 1).into()),
    }
}

/// `stringArgument` by position (`runtime/MethodArguments.hpp:136`), the
/// bytes copied into the lent result buffer: omitted is 93.903 and a value
/// without a string value is 88.909.
pub(super) fn string_method_argument(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    index: usize,
) -> Result<Vec<u8>, Failure> {
    let Some(value) = args.get(index).copied().flatten() else {
        return Err(Raised::missing_method_argument(index + 1).into());
    };
    let text = required_string_argument(interp, value, index + 1)?;
    let mut bytes = interp.take_result_buffer();
    bytes.extend_from_slice(&interp.to_text(text));
    Ok(bytes)
}

/// `optionalPadArgument` (`classes/StringClassUtil.cpp:261`): `None` for an
/// omitted argument, 88.909 for one without a string value, and 93.922 for a
/// string that is not one byte -- measured, oracle rc 163:
/// `.MutableBuffer~new('abcabc')~substr(1, 2, 'xx')` reports `found "xx"`.
pub(super) fn pad_method_argument(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    index: usize,
) -> Result<Option<u8>, Failure> {
    let Some(value) = args.get(index).copied().flatten() else {
        return Ok(None);
    };
    let text = required_string_argument(interp, value, index + 1)?;
    let pad = match interp.to_text(text).as_ref() {
        [byte] => Some(*byte),
        _ => None,
    };
    match pad {
        Some(byte) => Ok(Some(byte)),
        None => Err(Raised::incorrect_pad(&interp.string_value_text(value)).into()),
    }
}

/// `optionalOptionArgument` with a valid set (`classes/StringClassUtil.cpp:341`):
/// `None` for an omitted argument, 88.909 for one without a string value,
/// and 93.915 for the null string or a first byte outside `valid`, compared
/// case-folded; the `0x00` byte is admitted as the builtin's is -- measured,
/// oracle rc 163: `.MutableBuffer~new('abcabc')~verify('a', 'X')` reports
/// `must be one of "MN"; found "X"`, and rc 0: `~verify('abc', '00'x)` is `1`.
pub(super) fn option_method_argument(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    index: usize,
    valid: &str,
) -> Result<Option<u8>, Failure> {
    let Some(value) = args.get(index).copied().flatten() else {
        return Ok(None);
    };
    let text = required_string_argument(interp, value, index + 1)?;
    let letter = interp
        .to_text(text)
        .first()
        .map(|byte| byte.to_ascii_uppercase());
    match letter {
        Some(letter) if letter == 0 || valid.as_bytes().contains(&letter) => Ok(Some(letter)),
        _ => Err(
            Raised::method_option_not_recognised(valid, &interp.string_value_text(value)).into(),
        ),
    }
}

/// `optionalStringArgument` by position (`runtime/MethodArguments.hpp:186`):
/// the null string for an omitted argument, and 88.909 for a value without a
/// string value -- measured, oracle rc 0:
/// `.MutableBuffer~new('abcdef')~translate('ABC')` answers six blanks, the
/// empty input table reading each byte as its own index.
pub(super) fn optional_string_method_argument(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    index: usize,
) -> Result<Vec<u8>, Failure> {
    let Some(value) = args.get(index).copied().flatten() else {
        return Ok(Vec::new());
    };
    let text = required_string_argument(interp, value, index + 1)?;
    Ok(interp.to_text(text).into_owned())
}

/// [`optional_string_method_argument`] where an omitted argument has to stay
/// distinguishable from the null string, which is
/// `StringUtil::makearray`'s own `separator != OREF_NULL` test
/// (`classes/support/StringUtil.cpp:552`): `None` for an omitted argument,
/// and 88.909 for a value without a string value.
pub(super) fn optional_string_or_none_argument(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    index: usize,
) -> Result<Option<Vec<u8>>, Failure> {
    let Some(value) = args.get(index).copied().flatten() else {
        return Ok(None);
    };
    let text = required_string_argument(interp, value, index + 1)?;
    Ok(Some(interp.to_text(text).into_owned()))
}

/// `nonNegativeArgument` (`classes/StringClassUtil.cpp:167`): `None` for an
/// omitted argument, and anything that is not a non-negative whole number in
/// range is 93.906 -- measured, oracle rc 163:
/// `.MutableBuffer~new('abcdef')~insert('a', -1)` reports `Method argument 2
/// must be zero or a positive whole number; found "-1"`.
pub(super) fn optional_non_negative_argument(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    index: usize,
) -> Result<Option<usize>, Failure> {
    let raise = move |found: &[u8]| Raised::argument_not_non_negative(index + 1, found);
    match whole_method_argument(interp, args, index, raise)? {
        Some(value) if value >= 0 => Ok(Some(usize_or_refuse(interp, args, index, value, raise)?)),
        Some(_) => Err(refuse_method_argument(interp, args, index, raise)),
        None => Ok(None),
    }
}

/// `stringArgument`'s named overload (`runtime/MethodArguments.hpp:161`), the
/// bytes copied into the lent result buffer: omitted is 88.901 and a value
/// without a string value is 88.909.
pub(super) fn named_string_argument(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    index: usize,
    argument: &'static str,
) -> Result<Vec<u8>, Failure> {
    let Some(value) = args.get(index).copied().flatten() else {
        return Err(Raised::missing_named_argument(argument).into());
    };
    let text = required_string_named_argument(interp, value, argument)?;
    let mut bytes = interp.take_result_buffer();
    bytes.extend_from_slice(&interp.to_text(text));
    Ok(bytes)
}

/// `positionArgument`'s named overload (`classes/StringClassUtil.cpp:225`):
/// omitted is 88.901 and anything that is not a positive whole number in
/// range is 88.912.
pub(super) fn named_position_argument(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    index: usize,
    argument: &'static str,
) -> Result<usize, Failure> {
    let raise = move |found: &[u8]| Raised::named_argument_invalid_position(argument, found);
    match whole_method_argument(interp, args, index, raise)? {
        Some(position) if position > 0 => usize_or_refuse(interp, args, index, position, raise),
        Some(_) => Err(refuse_method_argument(interp, args, index, raise)),
        None => Err(Raised::missing_named_argument(argument).into()),
    }
}

/// `optionalLengthArgument`'s named overload
/// (`runtime/MethodArguments.hpp:344`): `None` for an omitted argument, and
/// anything that is not a non-negative whole number in range is 88.911.
pub(super) fn optional_named_length_argument(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    index: usize,
    argument: &'static str,
) -> Result<Option<usize>, Failure> {
    let raise = move |found: &[u8]| Raised::named_argument_invalid_length(argument, found);
    match whole_method_argument(interp, args, index, raise)? {
        Some(size) if size >= 0 => Ok(Some(usize_or_refuse(interp, args, index, size, raise)?)),
        Some(_) => Err(refuse_method_argument(interp, args, index, raise)),
        None => Ok(None),
    }
}

/// `optionalPadArgument`'s named overload
/// (`classes/StringClassUtil.cpp:275`): `None` for an omitted argument,
/// 88.909 for one without a string value, and 88.910 for a string that is not
/// one byte.
pub(super) fn named_pad_argument(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    index: usize,
    argument: &'static str,
) -> Result<Option<u8>, Failure> {
    let Some(value) = args.get(index).copied().flatten() else {
        return Ok(None);
    };
    let text = required_string_named_argument(interp, value, argument)?;
    let pad = match interp.to_text(text).as_ref() {
        [byte] => Some(*byte),
        _ => None,
    };
    match pad {
        Some(byte) => Ok(Some(byte)),
        None => Err(
            Raised::named_argument_invalid_pad(argument, &interp.string_value_text(value)).into(),
        ),
    }
}

/// One `optionalPositionArgument`/`optionalLengthArgument` conversion: the
/// argument at `index` as a whole number, `None` for an omitted or absent
/// position, and `raise`'s own condition for anything that is not one.
pub(super) fn whole_method_argument(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    index: usize,
    raise: impl Fn(&[u8]) -> Raised + Copy,
) -> Result<Option<i64>, Failure> {
    let Some(Some(value)) = args.get(index).copied() else {
        return Ok(None);
    };
    match interp.to_number(value) {
        Ok(number) => match number.whole_value(METHOD_ARGUMENT_DIGITS) {
            Some(whole) => Ok(Some(whole)),
            None => Err(refuse_method_argument(interp, args, index, raise)),
        },
        Err(_) => Err(refuse_method_argument(interp, args, index, raise)),
    }
}

/// [`whole_method_argument`]'s refusal, split out because the range checks
/// after it raise the identical condition about the identical object.
pub(super) fn refuse_method_argument(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    index: usize,
    raise: impl Fn(&[u8]) -> Raised,
) -> Failure {
    let found = match args.get(index).copied().flatten() {
        Some(value) => interp.string_value_text(value),
        None => Vec::new(),
    };
    raise(&found).into()
}

/// A converted argument narrowed to a `usize`, or the same refusal a bad
/// range gets. `i64` values wider than a `usize` cannot arise on the
/// platforms this builds for, and answering the refusal rather than
/// truncating is what keeps that from being an assumption.
pub(super) fn usize_or_refuse(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
    index: usize,
    value: i64,
    raise: impl Fn(&[u8]) -> Raised,
) -> Result<usize, Failure> {
    usize::try_from(value).map_err(|_| refuse_method_argument(interp, args, index, raise))
}

/// `Numerics::ARGUMENT_DIGITS`, the precision every method-argument
/// conversion runs at -- `builtin.rs`'s own constant of the same value, kept
/// separate because the two layers reach it through different call paths and
/// a shared one would tie them together for no reason beyond the number
/// agreeing today.
const METHOD_ARGUMENT_DIGITS: usize = 18;
