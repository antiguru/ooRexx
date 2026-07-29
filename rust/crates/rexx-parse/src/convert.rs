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
//! `RexxString::numberString` and `RexxString::requestNumber` decide whether an
//! operand is a number, and `TraceSetting::parseTraceSetting` decides whether
//! one is a usable `TRACE` option. The instruction grammar needs all three for
//! `TRACE` and the directive grammar needs all three for `::CONSTANT`,
//! `::ANNOTATE` and `::OPTIONS`, so they belong to neither module.

/// How a Rexx number's text decomposes: the sign, the significant digits with
/// the decimal point folded out, and the power of ten they are scaled by.
///
/// `value == (if negative { -1 } else { 1 }) * digits * 10^exponent`, with
/// `digits` holding no leading zero. An all-zero mantissa gives an empty
/// `digits` and a zero `exponent`, because every spelling of zero has the same
/// value and none of them has a significant digit.
struct Number {
    negative: bool,
    digits: Vec<u8>,
    exponent: i64,
}

/// The value of `text` as a whole number, or `None` if it is not one.
///
/// `RexxString::requestNumber(result, digits)`: the text must be a Rexx number
/// whose value is an integer expressible in at most `digits` digits. Measured
/// against `build/bin/rexxc` through `TRACE`, whose fallback makes the boundary
/// visible: `trace 1e2` is rc 0 and means 100, `trace 123456789` is rc 0,
/// `trace 1234567890` is Error 24.1 at ten digits, `trace 1e20` is 24.1 because
/// the value needs 21, and `trace 1.5` and `trace 1e-2` are 24.1 because
/// neither is whole.
///
/// `digits` is the caller's precision, because the two callers convert under
/// different ones: `TRACE` uses the parse-time `NUMERIC DIGITS` and
/// `::OPTIONS DIGITS` uses `Numerics::ARGUMENT_DIGITS`. Measured, the boundary
/// really does differ: `::options digits 123456789012345678` is rc 0 at
/// eighteen digits where `trace 1234567890` is already 24.1 at ten.
pub(crate) fn whole_number(text: &[u8], digits: usize) -> Option<i64> {
    let number = scan_number(text)?;
    let mut mantissa = number.digits;
    let mut exponent = number.exponent;
    // A negative exponent is only whole if the digits it would move past the
    // point are all zeros.
    while exponent < 0 {
        if mantissa.last() != Some(&b'0') {
            return None;
        }
        mantissa.pop();
        exponent += 1;
    }
    let width = mantissa.len() + usize::try_from(exponent).ok()?;
    if width > digits {
        return None;
    }
    let mut value: i64 = 0;
    for &digit in &mantissa {
        value = value
            .checked_mul(10)?
            .checked_add(i64::from(digit - b'0'))?;
    }
    for _ in 0..exponent {
        value = value.checked_mul(10)?;
    }
    Some(if number.negative { -value } else { value })
}

/// Decomposes `text` as a Rexx number, or `None` if it is not one.
///
/// Leading and trailing blanks and tabs are stripped, because a Rexx number may
/// carry them: measured, `::options digits " 9 "` and the tab-padded spelling
/// are both rc 0, while `"- 9"`, `"9 5"` and `"1 e2"` are Error 26.5, so the
/// blanks may surround the number and not sit inside it.
fn scan_number(text: &[u8]) -> Option<Number> {
    let mut rest = text;
    while let Some((&byte, tail)) = rest.split_first()
        && (byte == b' ' || byte == b'\t')
    {
        rest = tail;
    }
    while let Some((&byte, head)) = rest.split_last()
        && (byte == b' ' || byte == b'\t')
    {
        rest = head;
    }

    let mut negative = false;
    if let Some((&sign, tail)) = rest.split_first()
        && (sign == b'+' || sign == b'-')
    {
        negative = sign == b'-';
        rest = tail;
    }
    let integer_len = rest
        .iter()
        .position(|b| !b.is_ascii_digit())
        .unwrap_or(rest.len());
    let integer = &rest[..integer_len];
    rest = &rest[integer_len..];
    let mut fraction: &[u8] = b"";
    if rest.first() == Some(&b'.') {
        rest = &rest[1..];
        let len = rest
            .iter()
            .position(|b| !b.is_ascii_digit())
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
        if rest.is_empty() || !rest.iter().all(|b| b.is_ascii_digit()) {
            return None;
        }
        let digits = std::str::from_utf8(rest).ok()?;
        exponent = digits.parse::<i64>().ok()?;
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
    mantissa.extend_from_slice(integer);
    mantissa.extend_from_slice(fraction);
    exponent -= i64::try_from(fraction.len()).ok()?;
    let Some(first) = mantissa.iter().position(|&b| b != b'0') else {
        // Every digit is a zero, so the value is zero however the exponent
        // reads.
        return Some(Number {
            negative,
            digits: Vec::new(),
            exponent: 0,
        });
    };
    mantissa.drain(..first);
    Some(Number {
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
