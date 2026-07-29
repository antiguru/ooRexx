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

//! The text conversions both grammars need, ported from the runtime services
//! the C++ parser reaches into rather than from either parser.
//!
//! `TraceSetting::parseTraceSetting` decides whether an operand is a usable
//! `TRACE` option. `RexxString::numberString` decides whether one is a number at
//! all, and `requestNumber` converts one to an integer under a precision. The
//! instruction grammar needs all three for `TRACE` and the directive grammar
//! needs all three for `::CONSTANT`, `::ANNOTATE` and `::OPTIONS`, so they
//! belong to neither module.
//!
//! # Number ACCEPTANCE is not this crate's rule, and must not be
//!
//! `is_number` and the first step of `whole_number` both go through
//! `rexx_num::Number::parse`. That is the only implementation of "is this text a
//! Rexx number" in the workspace, and it must stay the only one.
//!
//! The C++ layers it the same way. `LanguageParser` and `DirectiveParser` never
//! implement number syntax, they call `RexxString::numberString()`, which is the
//! numeric runtime's. So the dependency edge from this crate to `rexx-num`
//! mirrors the interpreter's own.
//!
//! This is not a preference. An earlier version of this module carried its own
//! acceptance rule and reintroduced a defect `rexx-num` had already found, fixed
//! and pinned: blanks are legal between a sign and its digits, so
//! `trace "+ 9"` is a skip count of 9 and `::options digits "+ 9"` is rc 0, and
//! the private rule rejected both. `rexx-num`'s `signblank` case set, 2,320
//! cases, exists for exactly that defect, and `Number::parse` is verified across
//! 128,368 differential cases. Two ports of one rule drift. One does not.
//!
//! # What is still local, and why
//!
//! `whole_number`'s second half is `NumberString::numberValue`
//! (`NumberStringClass.cpp:588`), which rounds an accepted number to a precision
//! and asks whether the result is an integer that fits. That is a conversion and
//! not a syntax, and it lives nowhere else in the workspace.
//!
//! It needs the mantissa and exponent, and `Number` keeps those `pub(crate)`, so
//! `decompose` re-walks the text to recover them. **`decompose` is a valuation
//! and not a second acceptance rule**: it only ever runs on text
//! `Number::parse` has already accepted, and
//! `tests::the_local_walk_accepts_exactly_what_rexx_num_accepts` pins that it
//! agrees with `Number::parse` over `rexx-num`'s own `signblank` shapes, so it
//! cannot start disagreeing without a test failing.
//!
//! By rights `numberValue` belongs in `rexx-num` beside `Number::parse`, which
//! already has the `round_to` it partly duplicates. It is here because this task
//! was scoped not to touch that crate. `Number::format` is NOT a usable
//! substitute for it, and measurably so: `::options digits "1e18"` is Error 26.5
//! at nineteen digits, while Rexx's display rule puts an adjusted exponent equal
//! to `DIGITS` in plain notation, so a `format`-based check would accept it.

use rexx_num::Number;

/// The precision `::OPTIONS DIGITS`, `::OPTIONS FUZZ` and every internal
/// overflow check convert under.
///
/// `Numerics::ARGUMENT_DIGITS` (`Numerics.hpp:90`), 18 on a 64-bit build and 9
/// on a 32-bit one. The platform dependence is reproduced because it is
/// observable: measured on this 64-bit build,
/// `::options digits 123456789012345678` is rc 0 and `1234567890123456789` is
/// Error 26.5, so the boundary sits at eighteen.
pub(crate) const ARGUMENT_DIGITS: usize = 18;

/// A number's mantissa and exponent, recovered from text `Number::parse` has
/// already accepted.
///
/// `digits` holds one value 0..=9 per digit, most significant first, with
/// leading zeros stripped and trailing zeros KEPT, which is what
/// `NumberString::numberDigits` holds. Empty means the value is zero, because
/// every digit was a zero. `exponent` is the power of ten the last digit sits
/// at, so the value is `digits * 10^exponent` with the sign applied.
struct Decomposed {
    negative: bool,
    digits: Vec<u8>,
    exponent: i64,
}

/// `RexxString::numberString()`: the number `text` denotes, or `None` if it
/// denotes none.
///
/// Non-UTF-8 input is not a number, which is right rather than convenient: a
/// symbol cannot hold a non-ASCII byte at all, because
/// `LanguageParser::characterTable` is zero for every byte from 0x80 to 0xFF,
/// and a literal that holds one is not a number either.
fn number(text: &[u8]) -> Option<Number> {
    Number::parse(std::str::from_utf8(text).ok()?)
}

/// Whether `text` is a Rexx number at all, without asking what its value is.
///
/// `RexxString::numberString() != OREF_NULL`, which is the test the `::CONSTANT`
/// and `::ANNOTATE` signed forms make on the sign concatenated with the symbol
/// that followed it. Measured against `build/bin/rexxc`: `::CONSTANT c -.5`,
/// `-1e2` and `-5.` are all rc 0 while `-5x` and `-1e` are Error 19.916, so a
/// constant symbol is not necessarily a number.
///
/// The exponent limits come with the delegation rather than being restated here,
/// and they are observable in two independent places. Measured:
/// `::CONSTANT c -1e999999999` is rc 0 and `-1e1000000000` is 19.916, which is
/// the limit on the exponent AS WRITTEN; `-9e999999999` is rc 0 and
/// `-99e999999999` is 19.916, which is the limit on the ADJUSTED exponent. A
/// rule that checked only one of the two would pass one pair and fail the other.
pub(crate) fn is_number(text: &[u8]) -> bool {
    number(text).is_some()
}

/// The value of `text` as a whole number under `digits` precision, or `None` if
/// it has none.
///
/// `RexxString::requestNumber(result, digits)`, which is
/// `NumberString::numberValue` (`NumberStringClass.cpp:588`). The number is
/// ROUNDED to `digits` significant digits first and only then asked whether it
/// is an integer, which is why a fraction can survive the conversion. Measured
/// through `TRACE`, whose fallback to an option string makes each step visible:
///
/// * `trace 1e2` is rc 0 and means 100, and `trace 123456789` is rc 0.
/// * `trace 1234567890` is Error 24.1 at ten digits, and `trace 1e9` is 24.1
///   too, because its value needs ten even though its text holds two.
/// * `trace 1.5` and `trace 1e-2` are 24.1 because neither is whole.
/// * `trace 999999999.4` is **rc 0**: ten digits truncate to nine and the
///   dropped `4` does not round up, so the value is 999999999.
/// * `trace "1.0000000001"` is **rc 0**: eleven digits truncate to nine and
///   every surviving decimal is a zero, so the value is 1.
/// * `trace "0.9999999999"` is **rc 0**: the dropped digit rounds up, and a
///   carry over all-nine decimals gives 1.
/// * `trace "999999999.6"` is 24.1, because that carry makes the value ten
///   digits wide.
/// * `trace "99999999.6"` is 24.1, because nine digits do not exceed the
///   precision, so nothing is rounded and the `6` simply is not whole.
///
/// `digits` is the caller's precision, because the two callers convert under
/// different ones: `TRACE` uses the parse-time `NUMERIC DIGITS` and
/// `::OPTIONS DIGITS` uses `ARGUMENT_DIGITS`. Measured, the boundary really does
/// differ: `::options digits 123456789012345678` is rc 0 at eighteen digits
/// where `trace 1234567890` is already 24.1 at ten.
pub(crate) fn whole_number(text: &[u8], digits: usize) -> Option<i64> {
    // Acceptance first, and it is not this crate's rule. Everything below runs
    // on a number.
    number(text)?;
    let value = decompose(text)?;
    let sign: i64 = if value.negative { -1 } else { 1 };
    // `isZero()`: every spelling of zero converts to zero whatever the exponent
    // says.
    if value.digits.is_empty() {
        return Some(0);
    }

    let max = max_value_for_digits(digits);
    let precision = i64::try_from(digits).ok()?;
    let mut length = i64::try_from(value.digits.len()).ok()?;
    let mut exponent = value.exponent;

    // The common case: no more digits than the precision, and nothing after the
    // decimal point.
    if length <= precision && exponent >= 0 {
        return Some(unsigned_value(&value.digits, length, false, exponent, max)? * sign);
    }

    // `checkIntegerDigits` (`NumberStringClass.cpp:937`). Round to the
    // precision, then require every surviving decimal to be a zero, or a nine
    // when the rounding carried.
    let mut carry = false;
    if length > precision {
        exponent += length - precision;
        length = precision;
        if value.digits[digits] >= 5 {
            carry = true;
        }
    }
    if exponent < 0 {
        let mut decimal_pos = -exponent;
        let mut compare = 0u8;
        if carry {
            // A carry adds one to the right-most digit, so a decimal position
            // beyond the digits means at least one padding zero, and no carry
            // can turn that into an integer.
            if decimal_pos > length {
                return None;
            }
            compare = 9;
        }
        let data: &[u8] = if decimal_pos >= length {
            // The decimal point sits left of every digit, so all of them are
            // decimals.
            decimal_pos = length;
            &value.digits
        } else {
            &value.digits[usize::try_from(length + exponent).ok()?..]
        };
        for &digit in data.iter().take(usize::try_from(decimal_pos).ok()?) {
            if digit != compare {
                return None;
            }
        }
    }

    // The point now sits left of the first digit, so the value is whatever the
    // carry contributed and nothing else. The C++ does NOT apply the sign here,
    // and that is reproduced rather than corrected: `numberValue` returns
    // `carry ? 1 : 0` with no `* numberSign`.
    if -exponent >= length {
        return Some(i64::from(carry));
    }

    let converted = if exponent < 0 {
        unsigned_value(&value.digits, length + exponent, carry, 0, max)?
    } else {
        unsigned_value(&value.digits, length, carry, exponent, max)?
    };
    Some(converted * sign)
}

/// `Numerics::maxValueForDigits` (`Numerics.hpp:160`): `10^digits - 1`, capped
/// at the platform's integer width.
fn max_value_for_digits(digits: usize) -> i64 {
    let capped = digits.min(ARGUMENT_DIGITS);
    let mut max: i64 = 0;
    for _ in 0..capped {
        max = max * 10 + 9;
    }
    max
}

/// `NumberString::createUnsignedValue` (`NumberStringClass.cpp:788`): the first
/// `length` digits, plus a carry, scaled by `10^exponent`.
///
/// Every overflow path returns `None`, which is the C++'s `false`. The width
/// pre-check is against `ARGUMENT_DIGITS` and not against the caller's
/// precision, which is what the C++ tests.
fn unsigned_value(digits: &[u8], length: i64, carry: bool, exponent: i64, max: i64) -> Option<i64> {
    if exponent + length > i64::try_from(ARGUMENT_DIGITS).ok()? {
        return None;
    }
    let mut number: i64 = 0;
    for &digit in digits.iter().take(usize::try_from(length).ok()?) {
        number = number.checked_mul(10)?.checked_add(i64::from(digit))?;
    }
    if carry {
        number = number.checked_add(1)?;
    }
    for _ in 0..exponent {
        number = number.checked_mul(10)?;
    }
    if number > max {
        return None;
    }
    Some(number)
}

/// Recovers the mantissa and exponent of a number `Number::parse` accepted.
///
/// **Not an acceptance rule.** It returns `None` only for shapes it cannot
/// decompose, and every caller has already gated on `Number::parse`.
/// `tests::the_local_walk_accepts_exactly_what_rexx_num_accepts` pins that the
/// two agree anyway, over `rexx-num`'s own `signblank` shapes, so this cannot
/// start disagreeing without a test failing.
///
/// A blank is a space or a tab, and blanks are legal at either end and between a
/// sign and its first digit, nowhere else. That is `parseNumber`'s
/// `NUMBER_SIGN_WHITESPACE` state, which `rexx-num/src/lib.rs:389` documents and
/// which its `signblank` case set covers.
fn decompose(text: &[u8]) -> Option<Decomposed> {
    fn is_blank(byte: u8) -> bool {
        byte == b' ' || byte == b'\t'
    }
    let mut rest = text;
    while let Some((&byte, tail)) = rest.split_first()
        && is_blank(byte)
    {
        rest = tail;
    }
    while let Some((&byte, head)) = rest.split_last()
        && is_blank(byte)
    {
        rest = head;
    }

    let mut negative = false;
    if let Some((&sign, tail)) = rest.split_first()
        && (sign == b'+' || sign == b'-')
    {
        negative = sign == b'-';
        rest = tail;
        // The blanks a sign may be followed by. Without a sign there is nothing
        // to skip: the leading loop above already ran, so a blank here is simply
        // not part of a number.
        while let Some((&byte, tail)) = rest.split_first()
            && is_blank(byte)
        {
            rest = tail;
        }
    }

    let integer_len = rest
        .iter()
        .position(|byte| !byte.is_ascii_digit())
        .unwrap_or(rest.len());
    let integer = &rest[..integer_len];
    rest = &rest[integer_len..];
    let mut fraction: &[u8] = b"";
    if rest.first() == Some(&b'.') {
        rest = &rest[1..];
        let len = rest
            .iter()
            .position(|byte| !byte.is_ascii_digit())
            .unwrap_or(rest.len());
        fraction = &rest[..len];
        rest = &rest[len..];
    }
    if integer.is_empty() && fraction.is_empty() {
        return None;
    }
    let mut exponent: i64 = 0;
    if matches!(rest.first(), Some(b'e' | b'E')) {
        rest = &rest[1..];
        let mut exponent_negative = false;
        if let Some((&sign, tail)) = rest.split_first()
            && (sign == b'+' || sign == b'-')
        {
            exponent_negative = sign == b'-';
            rest = tail;
        }
        if rest.is_empty() || !rest.iter().all(u8::is_ascii_digit) {
            return None;
        }
        let written = std::str::from_utf8(rest).ok()?;
        exponent = written.parse::<i64>().ok()?;
        if exponent_negative {
            exponent = -exponent;
        }
        rest = b"";
    }
    if !rest.is_empty() {
        return None;
    }

    // The mantissa's digits, with the decimal point folded into the exponent.
    let mut mantissa: Vec<u8> = Vec::with_capacity(integer.len() + fraction.len());
    mantissa.extend(integer.iter().map(|byte| byte - b'0'));
    mantissa.extend(fraction.iter().map(|byte| byte - b'0'));
    exponent -= i64::try_from(fraction.len()).ok()?;
    let Some(first) = mantissa.iter().position(|&digit| digit != 0) else {
        // Every digit is a zero, so the value is zero however the exponent
        // reads.
        return Some(Decomposed {
            negative,
            digits: Vec::new(),
            exponent: 0,
        });
    };
    mantissa.drain(..first);
    Some(Decomposed {
        negative,
        digits: mantissa,
        exponent,
    })
}

/// Whether `text` is a usable `TRACE` option string.
///
/// `TraceSetting::parseTraceSetting` (`TraceSetting.cpp:135`) reads any number
/// of leading `?` toggles and then exactly ONE more character, ignoring
/// everything after it, which is why `trace results` works: only the `R`
/// matters. An empty string is the normal setting. Measured: `trace r`,
/// `trace ?r`, `trace ??r`, `trace results` and `trace ''` are all rc 0, while
/// `trace zzz` is Error 24.1.
pub(crate) fn check_trace_setting(text: &[u8]) -> Result<(), ()> {
    for &byte in text {
        if byte == b'?' {
            continue;
        }
        return match byte.to_ascii_uppercase() {
            b'A' | b'C' | b'L' | b'E' | b'F' | b'N' | b'O' | b'R' | b'I' => Ok(()),
            _ => Err(()),
        };
    }
    Ok(())
}

#[cfg(test)]
mod tests;
