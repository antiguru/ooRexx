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

//! The conversion builtins: `B2X`, `BITAND`, `BITOR`, `BITXOR`, `C2D`, `C2X`,
//! `D2C`, `D2X`, `X2B`, `X2C`, `X2D` and `XRANGE`.
//! ```text
//! x2c('414')       0414        an odd first group is padded
//! x2c('4 1424')    041424      residue 1, then a group of 4
//! x2c('414 2434')  04142434    residue 1, then a group of 4
//! x2c('414 243')   93.976      residue 1, then a group of 3
//! b2x('101 0000')  50          residue 3, then a group of 4
//! ```
//! ```text
//!             no length    ,1      ,2
//! c2d('80'x)  128          -128    128
//! x2d('80')   128          0       -128
//! ```

use rexx_core::ObjRef;

use super::{
    Args, arg, buffer, fresh_buffer, length_of, optional_string, pad_byte, required_string,
    whole_number,
};
use crate::Interp;
use crate::error::{Failure, Notation, Raised};

/// The two bytes that may separate the groups of a hexadecimal or binary
/// string: `RexxString::ch_SPACE` and `ch_TAB`.
fn is_blank(byte: u8) -> bool {
    byte == b' ' || byte == b'\t'
}

/// The hexadecimal digits the oracle writes, which are upper case.
const HEX_DIGITS: &[u8; 16] = b"0123456789ABCDEF";

/// The value of one hexadecimal digit, in either case.
fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

/// The value of one digit of `notation`, or `None` if the byte is not one.
fn digit_value(notation: Notation, byte: u8) -> Option<u8> {
    match notation {
        Notation::Hex => hex_value(byte),
        Notation::Binary => match byte {
            b'0' => Some(0),
            b'1' => Some(1),
            _ => None,
        },
    }
}

/// How many digits of `notation` make one group.
fn modulus(notation: Notation) -> usize {
    match notation {
        Notation::Hex => 2,
        Notation::Binary => 4,
    }
}

/// Checks `text` against `notation`'s character set and grouping rule,
/// answering how many digits it holds.
fn validate_grouped(text: &[u8], notation: Notation) -> Result<usize, Failure> {
    if is_blank(text[0]) {
        return Err(Raised::misplaced_whitespace(notation, 1).into());
    }

    let modulus = modulus(notation);
    let mut count = 0usize;
    let mut residue = 0usize;
    let mut space_found = false;
    // The 1-based position of the most recently seen whitespace byte, which is
    // what the trailing-whitespace error names -- the *last* of a run, not the
    // first.
    let mut space_position = 0usize;
    let mut last = 0u8;

    for (index, &byte) in text.iter().enumerate() {
        last = byte;
        if digit_value(notation, byte).is_some() {
            count += 1;
        } else if is_blank(byte) {
            space_position = index + 1;
            if space_found {
                if residue != count % modulus {
                    return Err(Raised::invalid_grouping(notation).into());
                }
            } else {
                residue = count % modulus;
                space_found = true;
            }
        } else {
            return Err(Raised::invalid_digit(notation, byte).into());
        }
    }

    if is_blank(last) {
        return Err(Raised::misplaced_whitespace(notation, space_position).into());
    }
    if space_found && count % modulus != residue {
        return Err(Raised::invalid_grouping(notation).into());
    }
    Ok(count)
}

/// Every digit of `text`, with the whitespace between groups dropped.
fn digits_of(text: &[u8], notation: Notation) -> impl Iterator<Item = u8> + '_ {
    text.iter()
        .filter_map(move |&byte| digit_value(notation, byte))
}

/// Packs a validated hexadecimal string into bytes, two nibbles at a time.
fn pack_hex(text: &[u8]) -> Result<Vec<u8>, Failure> {
    if text.is_empty() {
        return Ok(Vec::new());
    }
    let nibbles = validate_grouped(text, Notation::Hex)?;
    let mut out = fresh_buffer(nibbles.div_ceil(2))?;
    let mut digits = digits_of(text, Notation::Hex);
    let mut remaining = nibbles;
    if remaining % 2 == 1 {
        out.push(digits.next().expect("the scan counted this digit"));
        remaining -= 1;
    }
    while remaining > 0 {
        let high = digits.next().expect("the scan counted this digit");
        let low = digits.next().expect("the scan counted this digit");
        out.push(high << 4 | low);
        remaining -= 2;
    }
    Ok(out)
}

// ---- the bit builtins ----

/// The body `BITAND`, `BITOR` and `BITXOR` share.
fn bit_operation(
    interp: &mut Interp,
    name: &'static [u8],
    args: Args<'_>,
    operation: fn(u8, u8) -> u8,
    default_pad: u8,
) -> Result<ObjRef, Failure> {
    let first = required_string(interp, args, 1);
    // An omitted second string is the null string rather than a repeat of the
    // first: measured, `c2x(bitand('ffff'x))` is `FFFF` -- every byte reaches
    // the pad path and the default pad leaves it alone.
    let second = optional_string(interp, args, 2).unwrap_or_default();
    let pad = pad_byte(interp, name, args, 3)?.unwrap_or(default_pad);
    let out = bit_operation_over(interp, &first, &second, pad, operation)?;
    Ok(interp.text_built(out))
}

/// One bit operation once both strings and the pad are in hand, shared with
/// `String~bitAnd`, `~bitOr` and `~bitXor`.
pub(crate) fn bit_operation_over(
    interp: &Interp,
    first: &[u8],
    second: &[u8],
    pad: u8,
    operation: fn(u8, u8) -> u8,
) -> Result<Vec<u8>, Failure> {
    let (long, short) = if first.len() <= second.len() {
        (second, first)
    } else {
        (first, second)
    };
    let mut out = buffer(interp, long.len())?;
    out.extend_from_slice(long);
    for (index, byte) in out.iter_mut().enumerate() {
        *byte = match short.get(index) {
            Some(other) => operation(*byte, *other),
            None => operation(*byte, pad),
        };
    }
    Ok(out)
}

/// The default pad of each bit operation, which is its own identity byte.
pub(crate) const BITAND_PAD: u8 = 0xff;
/// See [`BITAND_PAD`].
pub(crate) const BITOR_PAD: u8 = 0x00;

/// `BITAND(string1 [,string2] [,pad])`, whose default pad is `'ff'x`.
pub(crate) fn bitand(
    interp: &mut Interp,
    name: &'static [u8],
    args: Args<'_>,
) -> Result<ObjRef, Failure> {
    bit_operation(interp, name, args, |a, b| a & b, BITAND_PAD)
}

/// `BITOR(string1 [,string2] [,pad])`, whose default pad is `'00'x`.
pub(crate) fn bitor(
    interp: &mut Interp,
    name: &'static [u8],
    args: Args<'_>,
) -> Result<ObjRef, Failure> {
    bit_operation(interp, name, args, |a, b| a | b, BITOR_PAD)
}

/// `BITXOR(string1 [,string2] [,pad])`, whose default pad is `'00'x`.
pub(crate) fn bitxor(
    interp: &mut Interp,
    name: &'static [u8],
    args: Args<'_>,
) -> Result<ObjRef, Failure> {
    bit_operation(interp, name, args, |a, b| a ^ b, BITOR_PAD)
}

// ---- the transliterating four ----

/// `C2X(string)`: each byte as two upper-case hexadecimal digits.
pub(crate) fn c2x(
    interp: &mut Interp,
    _name: &'static [u8],
    args: Args<'_>,
) -> Result<ObjRef, Failure> {
    let string = required_string(interp, args, 1);
    let out = c2x_bytes(interp, &string)?;
    Ok(interp.text_built(out))
}

/// `C2X`'s answer once its argument is read, shared with `String~c2x`.
pub(crate) fn c2x_bytes(interp: &Interp, string: &[u8]) -> Result<Vec<u8>, Failure> {
    // Saturating rather than checked: a length that doubles past `usize` can
    // only be refused, and `buffer` is what refuses it.
    let mut out = buffer(interp, string.len().saturating_mul(2))?;
    for byte in string {
        out.push(HEX_DIGITS[usize::from(byte >> 4)]);
        out.push(HEX_DIGITS[usize::from(byte & 0x0f)]);
    }
    Ok(out)
}

/// `X2C(string)`: hexadecimal digits packed into bytes.
pub(crate) fn x2c(
    interp: &mut Interp,
    _name: &'static [u8],
    args: Args<'_>,
) -> Result<ObjRef, Failure> {
    let string = required_string(interp, args, 1);
    let packed = x2c_bytes(&string)?;
    Ok(interp.text_built(packed))
}

/// `X2C`'s answer once its argument is read, shared with `String~x2c`.
pub(crate) fn x2c_bytes(string: &[u8]) -> Result<Vec<u8>, Failure> {
    pack_hex(string)
}

/// `X2B(string)`: four `0`/`1` bytes per hexadecimal digit, with no padding.
pub(crate) fn x2b(
    interp: &mut Interp,
    _name: &'static [u8],
    args: Args<'_>,
) -> Result<ObjRef, Failure> {
    let string = required_string(interp, args, 1);
    let out = x2b_bytes(interp, &string)?;
    Ok(interp.text_built(out))
}

/// `X2B`'s answer once its argument is read, shared with `String~x2b`.
pub(crate) fn x2b_bytes(interp: &Interp, string: &[u8]) -> Result<Vec<u8>, Failure> {
    if string.is_empty() {
        return Ok(Vec::new());
    }
    let nibbles = validate_grouped(string, Notation::Hex)?;
    let mut out = buffer(interp, nibbles.saturating_mul(4))?;
    for value in digits_of(string, Notation::Hex) {
        for bit in (0..4).rev() {
            out.push(b'0' + ((value >> bit) & 1));
        }
    }
    Ok(out)
}

/// `B2X(string)`: one hexadecimal digit per four bits, with the first group
/// padded up to four.
pub(crate) fn b2x(
    interp: &mut Interp,
    _name: &'static [u8],
    args: Args<'_>,
) -> Result<ObjRef, Failure> {
    let string = required_string(interp, args, 1);
    let out = b2x_bytes(interp, &string)?;
    Ok(interp.text_built(out))
}
/// RFC 2045's alphabet, which is the one with `+` and `/` rather than the
/// URL-safe pair: measured, `'-w=='~decodeBase64` is 93.962 and `'+w=='` is
/// not.
const BASE64: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/// The value `byte` stands for in [`BASE64`], or `None` if it stands for
/// nothing. `=` is not a digit and answers `None` here; only
/// [`decode_base64_bytes`] knows where it is allowed.
fn base64_digit(byte: u8) -> Option<u8> {
    match byte {
        b'A'..=b'Z' => Some(byte - b'A'),
        b'a'..=b'z' => Some(byte - b'a' + 26),
        b'0'..=b'9' => Some(byte - b'0' + 52),
        b'+' => Some(62),
        b'/' => Some(63),
        _ => None,
    }
}

/// `String~encodeBase64`, `RexxString::encodeBase64`
/// (`classes/StringClassConversion.cpp:90`).
pub(crate) fn encode_base64_bytes(source: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(source.len().div_ceil(3) * 4);
    for group in source.chunks(3) {
        let held = [
            group[0],
            group.get(1).copied().unwrap_or(0),
            group.get(2).copied().unwrap_or(0),
        ];
        out.push(BASE64[usize::from(held[0] >> 2)]);
        out.push(BASE64[usize::from(((held[0] & 0x03) << 4) | (held[1] >> 4))]);
        out.push(match group.len() {
            1 => b'=',
            _ => BASE64[usize::from(((held[1] & 0x0f) << 2) | (held[2] >> 6))],
        });
        out.push(match group.len() {
            3 => BASE64[usize::from(held[2] & 0x3f)],
            _ => b'=',
        });
    }
    out
}

/// `String~decodeBase64`, `RexxString::decodeBase64` (`:153`). `None` is
/// 93.962, which is the method's only refusal.
pub(crate) fn decode_base64_bytes(source: &[u8]) -> Option<Vec<u8>> {
    if !source.len().is_multiple_of(4) {
        return None;
    }
    let quartets = source.len() / 4;
    let mut out = Vec::with_capacity(quartets * 3);
    for (index, group) in source.as_chunks::<4>().0.iter().enumerate() {
        let last = index + 1 == quartets;
        let mut accumulated: u32 = 0;
        let mut taken: usize = 0;
        for (position, &byte) in group.iter().enumerate() {
            let Some(value) = base64_digit(byte) else {
                if byte == b'=' && last && (position == 3 || (position == 2 && group[3] == b'=')) {
                    break;
                }
                return None;
            };
            accumulated = (accumulated << 6) | u32::from(value);
            taken += 1;
        }
        // Left-align what was taken into the low three bytes, so the digits
        // sit where they would had the quartet been full.
        accumulated <<= 6 * (4 - taken) as u32;
        let bytes = accumulated.to_be_bytes();
        let emitted = taken.saturating_sub(1);
        out.extend_from_slice(&bytes[1..1 + emitted]);
    }
    Some(out)
}

/// `B2X`'s answer once its argument is read, shared with `String~b2x`.
pub(crate) fn b2x_bytes(interp: &Interp, string: &[u8]) -> Result<Vec<u8>, Failure> {
    if string.is_empty() {
        return Ok(Vec::new());
    }
    let bits = validate_grouped(string, Notation::Binary)?;
    let mut out = buffer(interp, bits.div_ceil(4))?;
    let mut digits = digits_of(string, Notation::Binary);
    let mut remaining = bits;
    while remaining > 0 {
        // The leading group is short when the bit count is not a multiple of
        // four, and the zeros it is short by are added on the left.
        let take = if remaining % 4 == 0 { 4 } else { remaining % 4 };
        let mut value = 0u8;
        for _ in 0..take {
            value = value << 1 | digits.next().expect("the scan counted this digit");
        }
        out.push(HEX_DIGITS[usize::from(value)]);
        remaining -= take;
    }
    Ok(out)
}

// ---- the numeric four ----

/// The slack the oracle adds to a conversion's working buffer,
/// `NumberString::OVERFLOWSPACE`.
const OVERFLOW_SPACE: usize = 2;

/// The `NUMERIC DIGITS` in force at the call.
fn current_digits(interp: &Interp) -> u64 {
    interp.activation().settings.digits()
}

/// Multiplies a little-endian base-`radix` accumulator by `factor` and adds
/// `addend`.
fn shift_in(accumulator: &mut Vec<u8>, radix: u32, factor: u32, addend: u32) {
    let mut carry = addend;
    for digit in accumulator.iter_mut() {
        let value = u32::from(*digit) * factor + carry;
        *digit = u8::try_from(value % radix).expect("a remainder is below the radix");
        carry = value / radix;
    }
    while carry > 0 {
        accumulator.push(u8::try_from(carry % radix).expect("a remainder is below the radix"));
        carry /= radix;
    }
}

/// How many digits an accumulator holds, counting an empty one as one.
fn digit_count(accumulator: &[u8]) -> usize {
    accumulator.len().max(1)
}

/// Renders a little-endian decimal accumulator, with a sign if negative.
fn render_decimal(accumulator: &[u8], negative: bool) -> Vec<u8> {
    let mut out = Vec::with_capacity(digit_count(accumulator) + usize::from(negative));
    if negative {
        out.push(b'-');
    }
    if accumulator.is_empty() {
        out.push(b'0');
    } else {
        out.extend(accumulator.iter().rev().map(|digit| b'0' + digit));
    }
    out
}

/// The body `C2D` and `X2D` share, `RexxString::x2dC2d`.
fn x2d_c2d(
    interp: &mut Interp,
    name: &'static [u8],
    args: Args<'_>,
    character: bool,
) -> Result<ObjRef, Failure> {
    let string = required_string(interp, args, 1);
    let requested = whole_number(interp, name, args, 2)?;
    // The length's range check runs before anything looks at the string,
    // which is the one ordering a program can see: measured, `x2d('ZZ',-1)`
    // is 93.923 where `x2d('ZZ',4)` is the invalid-character 93.933. The
    // method form gets that ordering by reading its own length before it
    // calls the core below.
    let requested = match requested {
        Some(value) => Some(length_of(value)?),
        None => None,
    };
    x2d_c2d_over(interp, &string, requested, character)
}

/// `X2D`'s and `C2D`'s answer once the string and the length are read, shared
/// with `String~x2d` and `String~c2d`.
pub(crate) fn x2d_c2d_over(
    interp: &mut Interp,
    string: &[u8],
    requested: Option<usize>,
    character: bool,
) -> Result<ObjRef, Failure> {
    let digits = current_digits(interp);
    // A length of zero answers zero without validating anything at all --
    // measured, `x2d('zz',0)` is `0`, not an error.
    let result_size = requested.unwrap_or(string.len());
    if result_size == 0 {
        return Ok(interp.integer_text(b"0".to_vec(), digits));
    }

    let packed;
    let bytes: &[u8] = if character {
        string
    } else {
        packed = pack_hex(string)?;
        &packed
    };

    // The window the length selects, its sign, and whether the top nibble of
    // that window has to be masked off afterwards.
    let mut window_start = 0usize;
    let mut negative = false;
    let mut odd_nibble = false;
    if let Some(length) = requested {
        let size = if character {
            length
        } else {
            // A hexadecimal length counts nibbles, so an odd one keeps half a
            // byte and moves the sign bit from 0x80 to 0x08.
            odd_nibble = length % 2 != 0;
            length / 2 + usize::from(odd_nibble)
        };
        if size <= bytes.len() {
            window_start = bytes.len() - size;
            let top = bytes[window_start];
            negative = if odd_nibble {
                top & 0x08 != 0
            } else {
                top & 0x80 != 0
            };
        } else {
            // Nothing was truncated, so nothing is masked either.
            odd_nibble = false;
        }
    }

    // A copy, because both the negation and the mask write into it. The
    // oracle copies here too, and for the same reason.
    let mut window = buffer(interp, bytes.len() - window_start)?;
    window.extend_from_slice(&bytes[window_start..]);
    if negative {
        for byte in window.iter_mut() {
            *byte ^= 0xff;
        }
        for byte in window.iter_mut().rev() {
            *byte = byte.wrapping_add(1);
            if *byte != 0 {
                break;
            }
        }
    }
    // After the negation, not before: measured, `x2d('18f',1)` is `-1`, which
    // needs the complement of `8f` masked down to `1` rather than the mask
    // applied to `8f` first.
    if odd_nibble && let Some(top) = window.first_mut() {
        *top &= 0x0f;
    }

    // The oracle asks for this buffer before it accumulates anything, so an
    // absurd `NUMERIC DIGITS` is refused rather than computed.
    drop(buffer(
        interp,
        usize::try_from(digits)
            .unwrap_or(usize::MAX)
            .saturating_add(OVERFLOW_SPACE + 1),
    )?);

    let mut accumulator: Vec<u8> = Vec::new();
    for byte in window {
        for nibble in [byte >> 4, byte & 0x0f] {
            shift_in(&mut accumulator, 10, 16, u32::from(nibble));
            if u64::try_from(digit_count(&accumulator)).unwrap_or(u64::MAX) > digits {
                return Err(if character {
                    Raised::c2d_result_too_large(digits).into()
                } else {
                    Raised::x2d_result_too_large(digits).into()
                });
            }
        }
    }
    let rendered = render_decimal(&accumulator, negative);
    Ok(interp.integer_text(rendered, digits))
}

/// `C2D(string [,n])`: the argument's bytes read as a binary integer.
pub(crate) fn c2d(
    interp: &mut Interp,
    name: &'static [u8],
    args: Args<'_>,
) -> Result<ObjRef, Failure> {
    x2d_c2d(interp, name, args, true)
}

/// `X2D(string [,n])`: the argument's hexadecimal digits read as an integer.
pub(crate) fn x2d(
    interp: &mut Interp,
    name: &'static [u8],
    args: Args<'_>,
) -> Result<ObjRef, Failure> {
    x2d_c2d(interp, name, args, false)
}

/// The pieces `D2X` and `D2C` need out of their first argument: a sign, the
/// significand most significant first, and the power of ten its last digit
/// stands for.
struct Decimal {
    negative: bool,
    /// Values `0..=9`. Exactly `[0]` when the value is zero, whatever the
    /// argument's spelling of zero was.
    significand: Vec<u8>,
    exponent: i64,
}

/// Splits the rendered text of a value already known to be a number.
fn scan_decimal(text: &[u8]) -> Option<Decimal> {
    let mut index = 0usize;
    while index < text.len() && is_blank(text[index]) {
        index += 1;
    }
    let mut negative = false;
    if index < text.len() && (text[index] == b'+' || text[index] == b'-') {
        negative = text[index] == b'-';
        index += 1;
        while index < text.len() && is_blank(text[index]) {
            index += 1;
        }
    }

    let mut significand: Vec<u8> = Vec::new();
    let mut decimals = 0i64;
    let mut seen_point = false;
    let mut seen_digit = false;
    while index < text.len() {
        match text[index] {
            byte @ b'0'..=b'9' => {
                seen_digit = true;
                // Leading zeros are not digits of the value, so they never
                // enter the significand at all.
                if !(significand.is_empty() && byte == b'0') {
                    significand.push(byte - b'0');
                }
                if seen_point {
                    decimals += 1;
                }
                index += 1;
            }
            b'.' if !seen_point => {
                seen_point = true;
                index += 1;
            }
            _ => break,
        }
    }
    if !seen_digit {
        return None;
    }

    let mut exponent = -decimals;
    if index < text.len() && (text[index] == b'e' || text[index] == b'E') {
        index += 1;
        let mut exponent_negative = false;
        if index < text.len() && (text[index] == b'+' || text[index] == b'-') {
            exponent_negative = text[index] == b'-';
            index += 1;
        }
        let start = index;
        let mut written = 0i64;
        while index < text.len() && text[index].is_ascii_digit() {
            written = written
                .saturating_mul(10)
                .saturating_add(i64::from(text[index] - b'0'));
            index += 1;
        }
        if index == start {
            return None;
        }
        if exponent_negative {
            written = -written;
        }
        exponent = exponent.saturating_add(written);
    }

    while index < text.len() && is_blank(text[index]) {
        index += 1;
    }
    if index != text.len() {
        return None;
    }

    // Every spelling of zero is the same value, and the oracle's canonical
    // one has a single digit and no exponent -- which is what makes `d2x(0)`
    // and `d2x('0.00000')` both `0`.
    if significand.iter().all(|digit| *digit == 0) {
        return Some(Decimal {
            negative: false,
            significand: vec![0],
            exponent: 0,
        });
    }
    Some(Decimal {
        negative,
        significand,
        exponent,
    })
}

/// Whether `value` has a non-zero decimal within `digits` significant digits,
/// `NumberString::hasSignificantDecimals`.
fn has_significant_decimals(value: &Decimal, digits: u64) -> bool {
    if value.exponent >= 0 {
        return false;
    }
    let significand = &value.significand;
    let start = i64::try_from(significand.len())
        .unwrap_or(i64::MAX)
        .saturating_add(value.exponent);
    if start < 0 {
        // The oracle's scan pointer runs off the front of the digit array
        // here. Every value that reaches it is a non-zero magnitude below
        // 0.1, which no rounding makes whole, and the oracle refuses each one
        // measured -- `d2x('0.01')` at `DIGITS 1`, `d2x('1E-100')` and
        // `d2x('0.0000000001')` are all 93.928.
        return true;
    }
    let mut index = usize::try_from(start).expect("start is not negative");
    let limit = usize::try_from(digits).unwrap_or(usize::MAX);
    let mut remaining = -value.exponent;
    while remaining > 0 && index < limit {
        if significand.get(index).copied().unwrap_or(0) != 0 {
            return true;
        }
        index += 1;
        remaining -= 1;
    }
    // The precision ran out first, so the digit it stopped on decides by
    // whether it would round up into the digits already checked.
    remaining > 0 && significand.get(index).copied().unwrap_or(0) >= 5
}

/// The body `D2X` and `D2C` share, `NumberString::d2xD2c`.
fn d2x_d2c(
    interp: &mut Interp,
    name: &'static [u8],
    args: Args<'_>,
    character: bool,
) -> Result<ObjRef, Failure> {
    let subject = arg(args, 1).expect("check_arity admitted this required argument");
    let numeric = interp.to_number(subject).is_ok();
    let text = interp.to_text(subject).into_owned();
    // **The length's type check runs here and its range check inside**, which
    // is the ordering `the_call_layer_is_checked_before_the_operation_layer`
    // pins: measured, `d2x('x','abc')` is the length's 40.12 while
    // `d2x('x',-1)` is the value's 93.928. The method form has neither half
    // out here -- for it the value beats the length outright.
    let raw = whole_number(interp, name, args, 2)?;
    d2x_d2c_over(interp, &text, numeric, character, move |_| match raw {
        Some(length) => Ok(Some(length_of(length)?)),
        None => Ok(None),
    })
}

/// `D2X`'s and `D2C`'s answer once the value is in hand, shared with
/// `String~d2x` and `String~d2c`.
pub(crate) fn d2x_d2c_over(
    interp: &mut Interp,
    text: &[u8],
    numeric: bool,
    character: bool,
    length: impl FnOnce(&mut Interp) -> Result<Option<usize>, Failure>,
) -> Result<ObjRef, Failure> {
    let digits = current_digits(interp);

    let not_whole = |found: &[u8]| -> Failure {
        if character {
            Raised::d2c_value_not_whole(found).into()
        } else {
            Raised::d2x_value_not_whole(found).into()
        }
    };

    let Some(value) = scan_decimal(text).filter(|_| numeric) else {
        return Err(not_whole(text));
    };
    let requested = length(interp)?;

    // The digit count of the value's integer form, which is what the setting
    // bounds. Widened because the exponent is the argument's own and can name
    // any power of ten a number may carry.
    let written = i128::from(value.exponent) + i128::try_from(value.significand.len()).unwrap_or(0);
    if written > i128::from(digits) {
        return Err(not_whole(text));
    }
    if has_significant_decimals(&value, digits) {
        return Err(not_whole(text));
    }
    if value.negative && requested.is_none() {
        return Err(Raised::length_required_for_negative().into());
    }

    // `D2C` builds twice as many hexadecimal digits as its length asks for
    // bytes, and the working buffer is sized from whichever of that and the
    // precision is larger -- which is where an absurd length is refused.
    let result_size = requested.map(|length| {
        if character {
            length.saturating_mul(2)
        } else {
            length
        }
    });
    let working = match result_size {
        Some(size) => size.max(usize::try_from(digits).unwrap_or(usize::MAX)),
        None => usize::try_from(digits).unwrap_or(usize::MAX),
    };
    drop(buffer(interp, working.saturating_add(OVERFLOW_SPACE))?);

    // Only the integer digits are accumulated; a fraction that got this far is
    // all zeros within the precision and contributes nothing.
    let integer_digits = if value.exponent < 0 {
        usize::try_from(
            i64::try_from(value.significand.len())
                .unwrap_or(i64::MAX)
                .saturating_add(value.exponent),
        )
        .unwrap_or(0)
    } else {
        value.significand.len()
    };
    let mut accumulator: Vec<u8> = Vec::new();
    for digit in value.significand.iter().take(integer_digits) {
        shift_in(&mut accumulator, 16, 10, u32::from(*digit));
    }
    for _ in 0..value.exponent.max(0) {
        shift_in(&mut accumulator, 16, 10, 0);
    }

    let mut pad = b'0';
    if value.negative {
        pad = b'F';
        // The oracle's own in-place negation: subtract one, borrowing through
        // the low-order zeros, then complement every nibble. The accumulator
        // is least significant first, so that borrow walks forwards.
        let mut index = 0usize;
        while accumulator.get(index).copied() == Some(0) {
            accumulator[index] = 0x0f;
            index += 1;
        }
        if let Some(digit) = accumulator.get_mut(index) {
            *digit -= 1;
        }
        for digit in accumulator.iter_mut() {
            *digit ^= 0x0f;
        }
    }

    let hex_length = digit_count(&accumulator);
    let result_size = result_size.unwrap_or(hex_length);
    let mut out = buffer(interp, result_size)?;
    // Padded on the left, or truncated on the left when the length asks for
    // fewer digits than the value has: measured, `d2x(4096,2)` is `00`.
    for _ in hex_length..result_size {
        out.push(pad);
    }
    for index in (0..hex_length.min(result_size)).rev() {
        let digit = accumulator.get(index).copied().unwrap_or(0);
        out.push(HEX_DIGITS[usize::from(digit)]);
    }

    if character {
        let packed = pack_hex(&out)?;
        return Ok(interp.text_built(packed));
    }
    Ok(interp.text_built(out))
}

/// `D2X(number [,n])`: a whole number as hexadecimal digits.
pub(crate) fn d2x(
    interp: &mut Interp,
    name: &'static [u8],
    args: Args<'_>,
) -> Result<ObjRef, Failure> {
    d2x_d2c(interp, name, args, false)
}

/// `D2C(number [,n])`: a whole number as bytes.
pub(crate) fn d2c(
    interp: &mut Interp,
    name: &'static [u8],
    args: Args<'_>,
) -> Result<ObjRef, Failure> {
    d2x_d2c(interp, name, args, true)
}

// ---- XRANGE ----

/// The twelve POSIX character classes `XRANGE` names, in the byte order the
/// oracle returns them.
const CHARACTER_CLASSES: &[(&[u8], &[u8])] = &[
    (
        b"alnum",
        b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz",
    ),
    (
        b"alpha",
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz",
    ),
    (b"blank", b"\t "),
    (
        b"cntrl",
        &[
            0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d,
            0x0e, 0x0f, 0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b,
            0x1c, 0x1d, 0x1e, 0x1f, 0x7f,
        ],
    ),
    (b"digit", b"0123456789"),
    (
        b"graph",
        b"!\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~",
    ),
    (b"lower", b"abcdefghijklmnopqrstuvwxyz"),
    (
        b"print",
        b" !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~",
    ),
    (b"punct", b"!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~"),
    (b"space", b"\t\n\x0b\x0c\r "),
    (b"upper", b"ABCDEFGHIJKLMNOPQRSTUVWXYZ"),
    (b"xdigit", b"0123456789ABCDEFabcdef"),
];

/// The table `name` selects, matched without regard to case.
fn character_class(name: &[u8]) -> Option<&'static [u8]> {
    CHARACTER_CLASSES
        .iter()
        .find(|(class, _)| class.eq_ignore_ascii_case(name))
        .map(|(_, table)| *table)
}

/// One contribution to an `XRANGE` result.
enum Piece {
    Class(&'static [u8]),
    /// A run of `length` bytes from `start`, wrapping at `0xff`.
    Range(u8, usize),
}

impl Piece {
    fn length(&self) -> usize {
        match self {
            Piece::Class(table) => table.len(),
            Piece::Range(_, length) => *length,
        }
    }

    fn write(&self, out: &mut Vec<u8>) {
        match self {
            Piece::Class(table) => out.extend_from_slice(table),
            Piece::Range(start, length) => {
                let mut byte = *start;
                for _ in 0..*length {
                    out.push(byte);
                    byte = byte.wrapping_add(1);
                }
            }
        }
    }
}

/// `XRANGE([start] [,end] ...)`: ranges and character classes, concatenated.
pub(crate) fn xrange(
    interp: &mut Interp,
    name: &'static [u8],
    args: Args<'_>,
) -> Result<ObjRef, Failure> {
    let count = args.len();
    let mut pieces: Vec<Piece> = Vec::new();
    let mut position = 0usize;

    // Entered once even with no arguments at all, which is what makes
    // `xrange()` the whole 256-byte range rather than the null string.
    while position == 0 || position < count {
        position += 1;
        let first = optional_string(interp, args, position);
        if let Some(text) = &first
            && text.len() != 1
        {
            let Some(table) = character_class(text) else {
                return Err(Raised::argument_not_a_pad_or_class_name(name, position, text).into());
            };
            // The oracle answers a lone class name from here without running
            // its second pass. That shortcut is not reproduced: it is an
            // allocation the two-pass path makes anyway, and unlike the
            // start/end shortcut below it changes no answer -- a one-argument
            // call leaves the loop immediately and builds the same bytes.
            pieces.push(Piece::Class(table));
            continue;
        }
        let start = first.map_or(0u8, |text| text[0]);

        position += 1;
        let end = match optional_string(interp, args, position) {
            Some(text) => match text.as_slice() {
                [byte] => *byte,
                _ => return Err(Raised::argument_not_a_pad(name, position, &text).into()),
            },
            None => 0xff,
        };
        let length = usize::from(end.wrapping_sub(start)) + 1;

        // The oracle's own early return, and it does not add what came
        // before: measured, `xrange('digit','z')` is 134 bytes with no digits
        // among them, where the three-argument form keeps them.
        if count <= 2 {
            let mut out = buffer(interp, length)?;
            Piece::Range(start, length).write(&mut out);
            return Ok(interp.text_built(out));
        }
        pieces.push(Piece::Range(start, length));
    }

    let total: usize = pieces.iter().map(Piece::length).sum();
    let mut out = buffer(interp, total)?;
    for piece in &pieces {
        piece.write(&mut out);
    }
    Ok(interp.text_built(out))
}

#[cfg(test)]
mod tests;
