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
//!
//! # A hexadecimal or binary string is grouped, and the rule is a residue
//!
//! `StringUtil::validateGroupedSet` (`classes/support/StringUtil.cpp`) is the
//! one scanner behind `X2C`, `X2B`, `B2X` and `X2D`, and its rule is not "each
//! group is a whole number of bytes". It keeps a **running total** of the
//! digits seen so far. At the first run of whitespace it records
//! `total % modulus` as the *residue*; at every later run of whitespace, and
//! once more at the end of the string, the running total must be congruent to
//! that same residue. `modulus` is 2 for hexadecimal and 4 for binary.
//!
//! Because the total is cumulative, that is the same thing as saying the first
//! group fixes the remainder and every group after it must be an exact
//! multiple -- and the first group is left-padded rather than rejected.
//! Measured:
//!
//! ```text
//! x2c('414')       0414        an odd first group is padded
//! x2c('4 1424')    041424      residue 1, then a group of 4
//! x2c('414 2434')  04142434    residue 1, then a group of 4
//! x2c('414 243')   93.976      residue 1, then a group of 3
//! b2x('101 0000')  50          residue 3, then a group of 4
//! ```
//!
//! Whitespace here is **blank and horizontal tab and nothing else** --
//! `RexxString::ch_SPACE` and `ch_TAB`, the same two bytes that separate
//! words. Measured: `x2c('41'||'09'x||'42')` is `4142` while
//! `x2c('41'||'0a'x||'42')` is 93.933, the invalid-character error, and every
//! byte at or above `0x80` is invalid too. Whitespace at either *end* is
//! 93.931/93.932 rather than a grouping error, and the position it names is
//! the last byte of a trailing run: `x2c('41 42  ')` names position 7.
//!
//! # `X2C` and `X2B` disagree about an odd number of nibbles
//!
//! `X2C` packs, so it rounds up to a whole byte and pads the top nibble with
//! zero; `X2B` expands, so it emits exactly four bits per nibble and pads
//! nothing. Measured: `x2c('414')` is `'0414'x` and `x2b('414')` is
//! `010000010100`, twelve bits. `B2X` is `X2C`-shaped in this respect -- it
//! pads the first group up to four bits, so `b2x('1')` is `1` and
//! `b2x('101 0000')` is `50`.
//!
//! # `NUMERIC DIGITS` bounds the value, and which end it bounds differs
//!
//! `C2D`/`X2D` are bounded on **output** (93.936/93.935) and `D2C`/`D2X` on
//! **input** (93.929/93.928). Measured under one setting, so the asymmetry is
//! visible in one program: at `DIGITS 9`, `c2d(copies('00'x,10)||'01'x)` is
//! `1` from eleven bytes while `c2d('ffffffff'x)` is an error from four. On
//! the other side, at `DIGITS 3`, `d2x('000123')` is `7B` while `d2x('1E3')`
//! is an error -- the count is of the value's digits, not the text's.
//!
//! The digit count a value is measured by is `exponent + significand length`,
//! with leading zeros removed and trailing zeros kept, which is the number of
//! decimal digits its integer form is written with.
//!
//! # A length argument is a right-aligned window, and it turns the read signed
//!
//! For `C2D` and `X2D` a length shorter than the value truncates from the
//! **left**, silently, and the top bit of what survives becomes a sign bit.
//! Measured, and the two builtins disagree with each other because `X2D`
//! counts the length in hexadecimal digits where `C2D` counts bytes:
//!
//! ```text
//!             no length    ,1      ,2
//! c2d('80'x)  128          -128    128
//! x2d('80')   128          0       -128
//! ```
//!
//! `x2d('80',1)` is `0` rather than `8` because the surviving half-byte has
//! its top nibble masked off *after* the sign test, which for an odd length
//! looks at bit `0x08` rather than `0x80`.
//!
//! For `D2C` and `D2X` the length is the width of the *result*, padded on the
//! left with `0` for a non-negative value and `F` for a negative one, and
//! truncated from the left when it is too small: `d2x(4096,2)` is `00`. A
//! negative value with no length at all is 93.927.
//!
//! # `XRANGE` is variadic over pairs, and one of them swallows the others
//!
//! Each iteration consumes either a class name (one argument) or a
//! start/end pair (two), and the pieces are concatenated -- `xrange('a','b',
//! 'c','d')` is `abcd`. Argument 1 of a pair may be a class name **or** a
//! single character (40.28); argument 2 may be a single character only
//! (40.23).
//!
//! The oracle finishes early whenever it reaches a start/end pair with two or
//! fewer arguments in the whole call, and that early return **discards
//! whatever a preceding class name contributed**. Measured:
//! `xrange('digit','z')` is 134 bytes, `'z'` through `'ff'x`, with no digits
//! in front of them, while the three-argument `xrange('digit','z','q')` does
//! include them. Two class names never reach that path, so
//! `xrange('upper','lower')` is the full alphabet.
//!
//! The `cntrl` class begins with a NUL and is 33 bytes long, so its table is
//! a byte slice with an explicit length rather than anything a C string could
//! carry.

use rexx_core::ObjRef;

use super::{arg, buffer, length_of, optional_string, pad_byte, required_string, whole_number};
use crate::Interp;
use crate::error::{Failure, Notation, Raised};

/// The two bytes that may separate the groups of a hexadecimal or binary
/// string: `RexxString::ch_SPACE` and `ch_TAB`.
///
/// Measured rather than taken from the C++ alone:
/// `x2c('41'||'09'x||'42')` converts, and every other byte below `0x20` --
/// including a newline and a NUL -- is 93.933.
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
///
/// Mirrors `StringUtil::validateGroupedSet`, whose residue rule the module doc
/// writes out. `text` is never the null string: every caller answers that
/// case before getting here, which is also what the oracle does -- its own
/// scanner reads the first byte before it looks at the length.
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
///
/// `StringUtil::packHex`. **The odd nibble is taken first**, which is what
/// makes `x2c('414')` the two bytes `04 14` rather than `41 40`.
fn pack_hex(text: &[u8]) -> Result<Vec<u8>, Failure> {
    if text.is_empty() {
        return Ok(Vec::new());
    }
    let nibbles = validate_grouped(text, Notation::Hex)?;
    let mut out = buffer(nibbles.div_ceil(2))?;
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
///
/// The result is as long as the **longer** argument, with the shorter one
/// combined into its front and `pad` into its tail. Measured, the argument
/// order does not matter: `c2x(bitand('00'x,'ffff'x))` is `00FF`, the same as
/// the reversed call.
///
/// **`pad` is the operation's identity element when the call supplies none**,
/// so the longer string's tail passes through unchanged -- which is why a
/// default of `'00'x` for `BITAND` is wrong. Measured:
/// `c2x(bitand('ffff'x,'00'x))` is `00FF`, one byte combined and one
/// surviving, against `0000` when the call supplies a `'00'x` pad of its own.
fn bit_operation(
    interp: &mut Interp,
    name: &'static [u8],
    args: &[Option<ObjRef>],
    operation: fn(u8, u8) -> u8,
    default_pad: u8,
) -> Result<ObjRef, Failure> {
    let first = required_string(interp, args, 1);
    // An omitted second string is the null string rather than a repeat of the
    // first: measured, `c2x(bitand('ffff'x))` is `FFFF` -- every byte reaches
    // the pad path and the default pad leaves it alone.
    let second = optional_string(interp, args, 2).unwrap_or_default();
    let pad = pad_byte(interp, name, args, 3)?.unwrap_or(default_pad);

    let (long, short) = if first.len() <= second.len() {
        (&second, &first)
    } else {
        (&first, &second)
    };
    let mut out = buffer(long.len())?;
    out.extend_from_slice(long);
    for (index, byte) in out.iter_mut().enumerate() {
        *byte = match short.get(index) {
            Some(other) => operation(*byte, *other),
            None => operation(*byte, pad),
        };
    }
    Ok(interp.text_owned(out))
}

/// `BITAND(string1 [,string2] [,pad])`, whose default pad is `'ff'x`.
pub(crate) fn bitand(
    interp: &mut Interp,
    name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    bit_operation(interp, name, args, |a, b| a & b, 0xff)
}

/// `BITOR(string1 [,string2] [,pad])`, whose default pad is `'00'x`.
pub(crate) fn bitor(
    interp: &mut Interp,
    name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    bit_operation(interp, name, args, |a, b| a | b, 0x00)
}

/// `BITXOR(string1 [,string2] [,pad])`, whose default pad is `'00'x`.
pub(crate) fn bitxor(
    interp: &mut Interp,
    name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    bit_operation(interp, name, args, |a, b| a ^ b, 0x00)
}

// ---- the transliterating four ----

/// `C2X(string)`: each byte as two upper-case hexadecimal digits.
pub(crate) fn c2x(
    interp: &mut Interp,
    _name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    let string = required_string(interp, args, 1);
    // Saturating rather than checked: a length that doubles past `usize` can
    // only be refused, and `buffer` is what refuses it.
    let mut out = buffer(string.len().saturating_mul(2))?;
    for byte in string {
        out.push(HEX_DIGITS[usize::from(byte >> 4)]);
        out.push(HEX_DIGITS[usize::from(byte & 0x0f)]);
    }
    Ok(interp.text_owned(out))
}

/// `X2C(string)`: hexadecimal digits packed into bytes.
pub(crate) fn x2c(
    interp: &mut Interp,
    _name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    let string = required_string(interp, args, 1);
    let packed = pack_hex(&string)?;
    Ok(interp.text_owned(packed))
}

/// `X2B(string)`: four `0`/`1` bytes per hexadecimal digit, with no padding.
pub(crate) fn x2b(
    interp: &mut Interp,
    _name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    let string = required_string(interp, args, 1);
    if string.is_empty() {
        return Ok(interp.text(b""));
    }
    let nibbles = validate_grouped(&string, Notation::Hex)?;
    let mut out = buffer(nibbles.saturating_mul(4))?;
    for value in digits_of(&string, Notation::Hex) {
        for bit in (0..4).rev() {
            out.push(b'0' + ((value >> bit) & 1));
        }
    }
    Ok(interp.text_owned(out))
}

/// `B2X(string)`: one hexadecimal digit per four bits, with the first group
/// padded up to four.
pub(crate) fn b2x(
    interp: &mut Interp,
    _name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    let string = required_string(interp, args, 1);
    if string.is_empty() {
        return Ok(interp.text(b""));
    }
    let bits = validate_grouped(&string, Notation::Binary)?;
    let mut out = buffer(bits.div_ceil(4))?;
    let mut digits = digits_of(&string, Notation::Binary);
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
    Ok(interp.text_owned(out))
}

// ---- the numeric four ----

/// The slack the oracle adds to a conversion's working buffer,
/// `NumberString::OVERFLOWSPACE`.
///
/// It is reproduced because the buffer is what refuses an absurd length: the
/// allocation is asked for before any digit is computed, so
/// `d2x(1,123456789012345678)` is Error 5 at rc 251 rather than a very long
/// wait.
const OVERFLOW_SPACE: usize = 2;

/// The `NUMERIC DIGITS` in force at the call.
fn current_digits(interp: &Interp) -> u64 {
    interp.activation().settings.digits()
}

/// Multiplies a little-endian base-`radix` accumulator by `factor` and adds
/// `addend`.
///
/// One helper for both directions: `C2D`/`X2D` accumulate base ten by sixteens
/// and `D2C`/`D2X` accumulate base sixteen by tens, which is
/// `NumberString::multiplyBaseTen`/`addToBaseTen` and their base-sixteen
/// twins.
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
///
/// The oracle's own count is `accumulator - highDigit`, and it starts at 1
/// with nothing accumulated -- adding a zero digit never moves the high-water
/// mark. That is what makes `d2x(0)` the one-character `0` rather than the
/// null string, and what lets `numeric digits 1 ; c2d('0000000000'x)` answer
/// `0` from ten bytes.
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
///
/// `character` is `C2D`: the argument is already bytes. Otherwise it is
/// hexadecimal text, packed first, and the length argument counts hexadecimal
/// digits rather than bytes.
fn x2d_c2d(
    interp: &mut Interp,
    name: &'static [u8],
    args: &[Option<ObjRef>],
    character: bool,
) -> Result<ObjRef, Failure> {
    let digits = current_digits(interp);
    let string = required_string(interp, args, 1);
    let requested = whole_number(interp, name, args, 2)?;
    // The length's range check runs before anything looks at the string,
    // which is the one ordering a program can see: measured, `x2d('ZZ',-1)`
    // is 93.923 where `x2d('ZZ',4)` is the invalid-character 93.933.
    let requested = match requested {
        Some(value) => Some(length_of(value)?),
        None => None,
    };
    // A length of zero answers zero without validating anything at all --
    // measured, `x2d('zz',0)` is `0`, not an error.
    let result_size = requested.unwrap_or(string.len());
    if result_size == 0 {
        return Ok(interp.text(b"0"));
    }

    let packed;
    let bytes: &[u8] = if character {
        &string
    } else {
        packed = pack_hex(&string)?;
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
    let mut window = buffer(bytes.len() - window_start)?;
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
    Ok(interp.text_owned(rendered))
}

/// `C2D(string [,n])`: the argument's bytes read as a binary integer.
pub(crate) fn c2d(
    interp: &mut Interp,
    name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    x2d_c2d(interp, name, args, true)
}

/// `X2D(string [,n])`: the argument's hexadecimal digits read as an integer.
pub(crate) fn x2d(
    interp: &mut Interp,
    name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    x2d_c2d(interp, name, args, false)
}

/// The pieces `D2X` and `D2C` need out of their first argument: a sign, the
/// significand most significant first, and the power of ten its last digit
/// stands for.
///
/// Leading zeros are dropped and trailing zeros kept, which is the oracle's
/// own `NumberString` shape -- measured at `DIGITS 3`, where `d2x('000123')`
/// converts and `d2x('12300')` does not.
struct Decimal {
    negative: bool,
    /// Values `0..=9`. Exactly `[0]` when the value is zero, whatever the
    /// argument's spelling of zero was.
    significand: Vec<u8>,
    exponent: i64,
}

/// Splits the rendered text of a value already known to be a number.
///
/// **Acceptance is [`Interp::to_number`]'s, not this function's**, and the
/// split is done on the text afterwards for the same reason the oracle does
/// it that way: `D2X`'s argument arrives as a string, and the oracle builds
/// its `NumberString` from exactly these bytes. Nothing here has to reject
/// anything, so the grammar it walks is only the shape a number's text can
/// take -- blanks, a sign, digits around at most one point, and an exponent.
/// `a_scan_takes_apart_every_text_the_number_parser_accepts` is what holds
/// the two to that: this may be looser than the parser, never tighter.
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
///
/// The setting is part of the question rather than a bound on the answer, and
/// a program can see that: measured, `d2x('1.4')` is `1` at `DIGITS 1` and an
/// error at `DIGITS 2`, because one digit of precision drops the `4` and it
/// is below the rounding threshold.
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
///
/// `character` is `D2C`, which asks for twice as many hexadecimal digits and
/// packs them at the end.
fn d2x_d2c(
    interp: &mut Interp,
    name: &'static [u8],
    args: &[Option<ObjRef>],
    character: bool,
) -> Result<ObjRef, Failure> {
    let digits = current_digits(interp);
    let subject = arg(args, 1).expect("check_arity admitted this required argument");
    let numeric = interp.to_number(subject).is_ok();
    let text = interp.to_text(subject).into_owned();
    let requested = whole_number(interp, name, args, 2)?;

    let not_whole = |found: &[u8]| -> Failure {
        if character {
            Raised::d2c_value_not_whole(found).into()
        } else {
            Raised::d2x_value_not_whole(found).into()
        }
    };

    // The value's own check comes first, ahead of the length's range check:
    // measured, `d2c('abc',-1)` is 93.929 where `d2x('1.5',-1)` -- a real
    // number with a bad length -- is 93.923.
    let Some(value) = scan_decimal(&text).filter(|_| numeric) else {
        return Err(not_whole(&text));
    };
    let requested = match requested {
        Some(length) => Some(length_of(length)?),
        None => None,
    };

    // The digit count of the value's integer form, which is what the setting
    // bounds. Widened because the exponent is the argument's own and can name
    // any power of ten a number may carry.
    let written = i128::from(value.exponent) + i128::try_from(value.significand.len()).unwrap_or(0);
    if written > i128::from(digits) {
        return Err(not_whole(&text));
    }
    if has_significant_decimals(&value, digits) {
        return Err(not_whole(&text));
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
    drop(buffer(working.saturating_add(OVERFLOW_SPACE))?);

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
    let mut out = buffer(result_size)?;
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
        return Ok(interp.text_owned(packed));
    }
    Ok(interp.text_owned(out))
}

/// `D2X(number [,n])`: a whole number as hexadecimal digits.
pub(crate) fn d2x(
    interp: &mut Interp,
    name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    d2x_d2c(interp, name, args, false)
}

/// `D2C(number [,n])`: a whole number as bytes.
pub(crate) fn d2c(
    interp: &mut Interp,
    name: &'static [u8],
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    d2x_d2c(interp, name, args, true)
}

// ---- XRANGE ----

/// The twelve POSIX character classes `XRANGE` names, in the byte order the
/// oracle returns them.
///
/// **Byte slices with explicit lengths, not C strings**: `CNTRL` begins with
/// a NUL, and the oracle takes its length as `1 + strlen(class + 1)` for
/// exactly that reason. Measured, `length(xrange('cntrl'))` is 33 and it
/// begins `00010203`.
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
    args: &[Option<ObjRef>],
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
            // A call that is nothing but one class name answers it directly.
            if count == 1 {
                return Ok(interp.text(table));
            }
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
            let mut out = buffer(length)?;
            Piece::Range(start, length).write(&mut out);
            return Ok(interp.text_owned(out));
        }
        pieces.push(Piece::Range(start, length));
    }

    let total: usize = pieces.iter().map(Piece::length).sum();
    let mut out = buffer(total)?;
    for piece in &pieces {
        piece.write(&mut out);
    }
    Ok(interp.text_owned(out))
}

#[cfg(test)]
mod tests {
    use super::super::dispatch;
    use super::{HEX_DIGITS, hex_value};
    use crate::plan::{BodyKey, ProgramId};
    use crate::{Activation, Interp, error::Failure, error::Raised};
    use rexx_parse::parse_program;
    use std::rc::Rc;

    /// An interpreter with a live top-level activation at `NUMERIC DIGITS
    /// digits`.
    ///
    /// The activation is what the `DIGITS`-sensitive four read their setting
    /// from, so unlike the other builtin families these tests cannot run
    /// against a bare `Interp`. The program it activates is a `NOP`: nothing
    /// here executes an instruction, and only the settings on the frame
    /// matter.
    fn interp_at(digits: &str) -> Interp {
        let mut interp = Interp::new();
        let program = Rc::new(parse_program(b"nop".to_vec()).expect("a NOP program parses"));
        let id = ProgramId(interp.programs.len());
        interp.programs.push(Rc::clone(&program));
        let plan = interp.plan_for(
            BodyKey {
                program: id,
                directive: None,
            },
            &program.main,
            &program.symbols,
        );
        let frame = interp.roots.push_slots(plan.len());
        let activation = interp.next_activation_id();
        interp
            .activations
            .push(Activation::new(activation, program, plan, frame));
        interp
            .activation_mut()
            .settings
            .set_digits_str(digits)
            .expect("a legal DIGITS setting");
        interp
    }

    /// Runs `name` over `arguments` at `digits`, each `None` standing for an
    /// omitted interior position, and answers the result's own bytes.
    ///
    /// Goes through [`dispatch`] rather than calling the implementation
    /// directly, so every case here also exercises the arity check and the
    /// name lookup that a real call would.
    fn call_at(digits: &str, name: &[u8], arguments: &[Option<&[u8]>]) -> Result<Vec<u8>, Failure> {
        let mut interp = interp_at(digits);
        let args: Vec<_> = arguments
            .iter()
            .map(|argument| argument.map(|bytes| interp.text(bytes)))
            .collect();
        let result = dispatch(&mut interp, name, &args).expect("a builtin name")?;
        Ok(interp.to_text(result).into_owned())
    }

    /// [`call_at`] at the default precision.
    fn call(name: &[u8], arguments: &[Option<&[u8]>]) -> Result<Vec<u8>, Failure> {
        call_at("9", name, arguments)
    }

    /// [`call`], for the cases whose answer is the bytes and nothing else.
    fn answer(name: &[u8], arguments: &[&[u8]]) -> Vec<u8> {
        let arguments: Vec<_> = arguments.iter().map(|bytes| Some(*bytes)).collect();
        call(name, &arguments).expect("this call succeeds")
    }

    /// [`answer`] at a chosen precision.
    fn answer_at(digits: &str, name: &[u8], arguments: &[&[u8]]) -> Vec<u8> {
        let arguments: Vec<_> = arguments.iter().map(|bytes| Some(*bytes)).collect();
        call_at(digits, name, &arguments).expect("this call succeeds")
    }

    /// The `(major, sub)` and substitutions of the condition `name` raises.
    fn raised(name: &[u8], arguments: &[Option<&[u8]>]) -> (u16, u16, Vec<Vec<u8>>) {
        raised_at("9", name, arguments)
    }

    /// [`raised`] at a chosen precision.
    fn raised_at(
        digits: &str,
        name: &[u8],
        arguments: &[Option<&[u8]>],
    ) -> (u16, u16, Vec<Vec<u8>>) {
        let failure = call_at(digits, name, arguments).expect_err("this call raises");
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        (raised.number, raised.sub, raised.additional)
    }

    /// The alphabet every case set below draws from: the null string, a byte
    /// at or above `0x80`, a control byte, a NUL, and the two that separate
    /// groups.
    ///
    /// Named rather than written out at each site so a reader can see what
    /// the coverage claim rests on, and so a case set that quietly narrowed
    /// to printable ASCII would be visible.
    const BYTE_ALPHABET: &[&[u8]] = &[
        b"",
        b"a",
        b"AB",
        &[0x00],
        &[0x01],
        &[0x09],
        &[0x20],
        &[0x7f],
        &[0x80],
        &[0xff],
        &[0x00, 0xff, 0x7f, 0x80],
        &[0x61, 0x00, 0xe9],
    ];

    /// `C2X` writes two upper-case digits per byte over the whole byte range,
    /// and `X2C` reverses it.
    ///
    /// The round trip is what makes the pair's agreement checkable without a
    /// second table of expected bytes, and the sweep is all 256 values
    /// because a rule stated for "printable characters" agrees with this one
    /// everywhere a printable test could look.
    #[test]
    fn c2x_and_x2c_are_inverse_over_every_byte() {
        for byte in 0..=u8::MAX {
            let hex = answer(b"C2X", &[&[byte]]);
            let expected = format!("{byte:02X}").into_bytes();
            assert_eq!(hex, expected, "byte {byte:#04x} did not render");
            assert_eq!(answer(b"X2C", &[&hex]), vec![byte]);
            // Lower case on the way in, upper case on the way out.
            let lowered: Vec<u8> = hex.iter().map(u8::to_ascii_lowercase).collect();
            assert_eq!(answer(b"X2C", &[&lowered]), vec![byte]);
        }
        for subject in BYTE_ALPHABET {
            let hex = answer(b"C2X", &[subject]);
            assert_eq!(hex.len(), subject.len() * 2);
            assert_eq!(&answer(b"X2C", &[&hex])[..], *subject);
        }
        assert_eq!(answer(b"C2X", &[b""]), b"");
        assert_eq!(answer(b"X2C", &[b""]), b"");
        assert_eq!(answer(b"C2X", &[b"Ab"]), b"4162");
        assert_eq!(answer(b"X2C", &[b"616263"]), b"abc");
    }

    /// The grouping rule: the first group fixes a residue and every later one
    /// must be an exact multiple, with the first group left-padded.
    ///
    /// Both moduli, because 2 and 4 are separate constants in the same
    /// scanner and a hexadecimal-only test cannot see a binary one that
    /// disagrees. Each refusal is paired with the neighbouring string that
    /// converts, so the rule is pinned to the group sizes rather than to the
    /// presence of a blank.
    #[test]
    fn a_grouped_string_carries_its_residue_to_the_end() {
        assert_eq!(answer(b"X2C", &[b"414"]), b"\x04\x14");
        assert_eq!(answer(b"X2C", &[b"4 1424"]), b"\x04\x14\x24");
        assert_eq!(answer(b"X2C", &[b"414 2434"]), b"\x04\x14\x24\x34");
        assert_eq!(answer(b"X2C", &[b"41 42 43"]), b"ABC");
        assert_eq!(answer(b"X2C", &[b"4\t1424"]), b"\x04\x14\x24");
        assert_eq!(raised(b"X2C", &[Some(b"414 243")]), (93, 976, Vec::new()));
        assert_eq!(raised(b"X2C", &[Some(b"4 4")]), (93, 976, Vec::new()));
        assert_eq!(raised(b"X2C", &[Some(b"44 4")]), (93, 976, Vec::new()));

        assert_eq!(answer(b"B2X", &[b"101 0000"]), b"50");
        assert_eq!(answer(b"B2X", &[b"1 0000"]), b"10");
        assert_eq!(answer(b"B2X", &[b"1 0000 0011"]), b"103");
        assert_eq!(answer(b"B2X", &[b"11 0000 0011"]), b"303");
        assert_eq!(answer(b"B2X", &[b"1\t0000"]), b"10");
        assert_eq!(raised(b"B2X", &[Some(b"101 000")]), (93, 977, Vec::new()));
        assert_eq!(raised(b"B2X", &[Some(b"10 10")]), (93, 977, Vec::new()));
        assert_eq!(raised(b"B2X", &[Some(b"1 0 0000")]), (93, 977, Vec::new()));
    }

    /// Whitespace is blank and tab; every other byte below `0x20`, and every
    /// byte at or above `0x80`, is an invalid digit.
    ///
    /// Swept over the whole byte range rather than at a handful of plausible
    /// separators, because the rules this one could wrongly be -- "any
    /// `isspace`", "every byte below `0x21`" -- differ from it at bytes
    /// nobody would write down.
    #[test]
    fn only_blank_and_tab_separate_groups() {
        for byte in 0..=u8::MAX {
            let subject = [b'4', b'1', byte, b'4', b'2'];
            let result = call(b"X2C", &[Some(&subject)]);
            if byte == b' ' || byte == b'\t' {
                assert_eq!(result.expect("a separator converts"), b"AB");
            } else if hex_value(byte).is_some() {
                assert!(result.is_ok(), "byte {byte:#04x} is a hexadecimal digit");
            } else {
                let failure = result.expect_err("this byte is not a hexadecimal digit");
                let Failure::Raised(raised) = failure else {
                    panic!("expected Raised");
                };
                assert_eq!((raised.number, raised.sub), (93, 933));
                assert_eq!(raised.additional, vec![vec![byte]]);
            }
        }
        // The binary twin names its own sub-code and carries the byte too.
        assert_eq!(
            raised(b"B2X", &[Some(b"1012")]),
            (93, 934, vec![b"2".to_vec()])
        );
        assert_eq!(
            raised(b"B2X", &[Some(&[b'1', 0xff])]),
            (93, 934, vec![vec![0xff]])
        );
        assert_eq!(answer(b"B2X", &[b"1\t0000"]), b"10");
    }

    /// Whitespace at either end is the misplaced-whitespace error, and the
    /// position it names is the *last* byte of a trailing run.
    #[test]
    fn whitespace_at_either_end_names_its_position() {
        assert_eq!(
            raised(b"X2C", &[Some(b" 4142")]),
            (93, 931, vec![b"1".to_vec()])
        );
        assert_eq!(
            raised(b"X2C", &[Some(b"4142 ")]),
            (93, 931, vec![b"5".to_vec()])
        );
        assert_eq!(
            raised(b"X2C", &[Some(b"41 42  ")]),
            (93, 931, vec![b"7".to_vec()])
        );
        assert_eq!(
            raised(b"X2C", &[Some(b"4142\t")]),
            (93, 931, vec![b"5".to_vec()])
        );
        assert_eq!(
            raised(b"X2D", &[Some(b" ")]),
            (93, 931, vec![b"1".to_vec()])
        );
        assert_eq!(
            raised(b"B2X", &[Some(b" 1010")]),
            (93, 932, vec![b"1".to_vec()])
        );
        assert_eq!(
            raised(b"B2X", &[Some(b"1010 ")]),
            (93, 932, vec![b"5".to_vec()])
        );
        // The adjacent successes: the same strings with the outer whitespace
        // removed convert, so the refusal is about position and not content.
        assert_eq!(answer(b"X2C", &[b"4142"]), b"AB");
        assert_eq!(answer(b"B2X", &[b"1010"]), b"A");
    }

    /// `X2C` pads an odd number of nibbles up to a byte and `X2B` does not
    /// pad at all, which is the one place the two disagree.
    #[test]
    fn x2c_pads_an_odd_nibble_where_x2b_does_not() {
        assert_eq!(answer(b"X2C", &[b"4"]), b"\x04");
        assert_eq!(answer(b"X2C", &[b"414"]), b"\x04\x14");
        assert_eq!(answer(b"X2B", &[b"4"]), b"0100");
        assert_eq!(answer(b"X2B", &[b"414"]), b"010000010100");
        assert_eq!(answer(b"X2B", &[b"c3"]), b"11000011");
        assert_eq!(answer(b"X2B", &[b"4 1424"]), b"01000001010000100100");
        assert_eq!(answer(b"X2B", &[b""]), b"");
        // `B2X` is `X2C`-shaped: the leading group is padded up to four bits.
        assert_eq!(answer(b"B2X", &[b"1"]), b"1");
        assert_eq!(answer(b"B2X", &[b"11"]), b"3");
        assert_eq!(answer(b"B2X", &[b"100"]), b"4");
        assert_eq!(answer(b"B2X", &[b"11000011"]), b"C3");
        assert_eq!(answer(b"B2X", &[b"0000"]), b"0");
        assert_eq!(answer(b"B2X", &[b""]), b"");
        // Every nibble value, both ways round.
        for value in 0..16u8 {
            let hex = [HEX_DIGITS[usize::from(value)]];
            let bits: Vec<u8> = (0..4).rev().map(|b| b'0' + ((value >> b) & 1)).collect();
            assert_eq!(answer(b"X2B", &[&hex]), bits);
            assert_eq!(answer(b"B2X", &[&bits]), hex);
        }
    }

    /// The bit builtins pass the longer string's tail through when no pad is
    /// supplied, and combine it with the pad when one is.
    ///
    /// The pad axis is crossed with both orders of unequal lengths and with
    /// the null string, because an implementation that defaulted `BITAND`'s
    /// pad to `'00'x` is right for every equal-length case and wrong for
    /// every other one.
    #[test]
    fn an_omitted_pad_leaves_the_longer_strings_tail_alone() {
        assert_eq!(answer(b"BITAND", &[b"\xff\xff", b"\x00"]), b"\x00\xff");
        assert_eq!(answer(b"BITAND", &[b"\x00", b"\xff\xff"]), b"\x00\xff");
        assert_eq!(
            answer(b"BITAND", &[b"\xff\xff", b"\x00", b"\x00"]),
            b"\x00\x00"
        );
        assert_eq!(answer(b"BITOR", &[b"\xff\xff", b"\x00"]), b"\xff\xff");
        assert_eq!(answer(b"BITOR", &[b"\x00\x00", b"\xff"]), b"\xff\x00");
        assert_eq!(answer(b"BITXOR", &[b"\xff\xff", b"\x00"]), b"\xff\xff");
        assert_eq!(answer(b"BITXOR", &[b"\x00\x00", b"\xff"]), b"\xff\x00");
        // One argument: every byte reaches the pad path.
        assert_eq!(answer(b"BITAND", &[b"\xff\xff"]), b"\xff\xff");
        assert_eq!(answer(b"BITOR", &[b"\x00\x00"]), b"\x00\x00");
        assert_eq!(answer(b"BITXOR", &[b"\x00\x00"]), b"\x00\x00");
        assert_eq!(answer(b"BITAND", &[b"abc"]), b"abc");
        // A null second string is the same thing as no second string.
        for name in [b"BITAND".as_slice(), b"BITOR", b"BITXOR"] {
            assert_eq!(answer(name, &[b"abc", b""]), b"abc");
            assert_eq!(answer(name, &[b"", b"abc"]), b"abc");
            assert_eq!(answer(name, &[b"", b""]), b"");
            assert_eq!(answer(name, &[b""]), b"");
        }
        // The documented examples, which pin the operations themselves.
        assert_eq!(answer(b"BITAND", &[b"cat", b"DOG"]), b"@AD");
        assert_eq!(answer(b"BITOR", &[b"cat", b"DOG"]), b"gow");
        assert_eq!(answer(b"BITXOR", &[b"cat", b"   "]), b"CAT");
        // The pad crossed with the tail, over the byte alphabet.
        for pad in [0x00u8, 0xff, 0x0f, 0x80, 0x01] {
            for subject in BYTE_ALPHABET {
                let and = answer(b"BITAND", &[subject, b"", &[pad]]);
                let or = answer(b"BITOR", &[subject, b"", &[pad]]);
                let xor = answer(b"BITXOR", &[subject, b"", &[pad]]);
                let expect = |op: fn(u8, u8) -> u8| -> Vec<u8> {
                    subject.iter().map(|byte| op(*byte, pad)).collect()
                };
                assert_eq!(and, expect(|a, b| a & b));
                assert_eq!(or, expect(|a, b| a | b));
                assert_eq!(xor, expect(|a, b| a ^ b));
            }
        }
        // An interior omission past the minimum is legal and reaches the pad.
        assert_eq!(
            call(b"BITAND", &[Some(b"\xff\xff"), None, Some(b"\x00")])
                .expect("an omitted second string is legal"),
            b"\x00\x00"
        );
    }

    /// A length argument is a right-aligned window that truncates from the
    /// left, and the top of what survives is a sign bit.
    ///
    /// `C2D` and `X2D` are asserted side by side because they disagree: the
    /// same `80` is `-128` at `C2D`'s length 1 and `0` at `X2D`'s, since one
    /// counts bytes and the other nibbles.
    #[test]
    fn a_length_makes_the_read_a_signed_window() {
        assert_eq!(answer(b"C2D", &[b"\x80"]), b"128");
        assert_eq!(answer(b"C2D", &[b"\x80", b"1"]), b"-128");
        assert_eq!(answer(b"C2D", &[b"\x80", b"2"]), b"128");
        assert_eq!(answer(b"X2D", &[b"80"]), b"128");
        assert_eq!(answer(b"X2D", &[b"80", b"1"]), b"0");
        assert_eq!(answer(b"X2D", &[b"80", b"2"]), b"-128");

        assert_eq!(answer(b"C2D", &[b"\x01\x02\x03\x04", b"2"]), b"772");
        assert_eq!(answer(b"C2D", &[b"\x7f", b"1"]), b"127");
        assert_eq!(answer(b"C2D", &[b"\xff\x00", b"2"]), b"-256");
        assert_eq!(answer(b"C2D", &[b"\xff\x00", b"1"]), b"0");
        assert_eq!(answer(b"C2D", &[b"\x00\x80", b"2"]), b"128");
        assert_eq!(answer(b"C2D", &[b"\x01", b"5"]), b"1");
        assert_eq!(answer(b"C2D", &[b"\xff", b"5"]), b"255");

        assert_eq!(answer(b"X2D", &[b"8f", b"1"]), b"-1");
        assert_eq!(answer(b"X2D", &[b"18f", b"1"]), b"-1");
        assert_eq!(answer(b"X2D", &[b"18f", b"2"]), b"-113");
        assert_eq!(answer(b"X2D", &[b"18f", b"3"]), b"399");
        assert_eq!(answer(b"X2D", &[b"ff00", b"2"]), b"0");
        assert_eq!(answer(b"X2D", &[b"ff00", b"3"]), b"-256");
        assert_eq!(answer(b"X2D", &[b"ff00", b"4"]), b"-256");
        assert_eq!(answer(b"X2D", &[b"ff00", b"5"]), b"65280");
        assert_eq!(answer(b"X2D", &[b"f", b"2"]), b"15");
        assert_eq!(answer(b"X2D", &[b"ff", b"5"]), b"255");

        // A length of zero answers zero without validating anything, which is
        // the one shape that separates the shortcut from the scan.
        assert_eq!(answer(b"C2D", &[b"abc", b"0"]), b"0");
        assert_eq!(answer(b"X2D", &[b"zz", b"0"]), b"0");
        assert_eq!(answer(b"C2D", &[b""]), b"0");
        assert_eq!(answer(b"X2D", &[b""]), b"0");
        assert_eq!(raised(b"X2D", &[Some(b"zz"), Some(b"4")]).1, 933);
        // The adjacent legality: an omitted length past the minimum.
        assert_eq!(
            call(b"C2D", &[Some(b"\xff"), None]).expect("an omitted length is legal"),
            b"255"
        );
    }

    /// A length is the width of a `D2X`/`D2C` result, padded on the left with
    /// `0` or `F` and truncated on the left when it is too small.
    #[test]
    fn a_d2x_length_pads_and_truncates_on_the_left() {
        assert_eq!(answer(b"D2X", &[b"1"]), b"1");
        assert_eq!(answer(b"D2X", &[b"255"]), b"FF");
        assert_eq!(answer(b"D2X", &[b"255", b"1"]), b"F");
        assert_eq!(answer(b"D2X", &[b"255", b"5"]), b"000FF");
        assert_eq!(answer(b"D2X", &[b"4096", b"2"]), b"00");
        assert_eq!(answer(b"D2X", &[b"-1", b"3"]), b"FFF");
        assert_eq!(answer(b"D2X", &[b"-255", b"3"]), b"F01");
        assert_eq!(answer(b"D2X", &[b"-1", b"1"]), b"F");
        assert_eq!(answer(b"D2X", &[b"-16", b"1"]), b"0");
        assert_eq!(answer(b"D2X", &[b"-16", b"2"]), b"F0");
        assert_eq!(answer(b"D2X", &[b"0"]), b"0");
        assert_eq!(answer(b"D2X", &[b"0", b"3"]), b"000");
        assert_eq!(answer(b"D2X", &[b"0", b"0"]), b"");
        assert_eq!(answer(b"D2X", &[b"255", b"0"]), b"");

        assert_eq!(answer(b"D2C", &[b"1"]), b"\x01");
        assert_eq!(answer(b"D2C", &[b"0"]), b"\x00");
        assert_eq!(answer(b"D2C", &[b"256"]), b"\x01\x00");
        assert_eq!(answer(b"D2C", &[b"16706"]), b"AB");
        assert_eq!(answer(b"D2C", &[b"255", b"1"]), b"\xff");
        assert_eq!(answer(b"D2C", &[b"255", b"3"]), b"\x00\x00\xff");
        assert_eq!(answer(b"D2C", &[b"-1", b"1"]), b"\xff");
        assert_eq!(answer(b"D2C", &[b"-1", b"3"]), b"\xff\xff\xff");
        assert_eq!(answer(b"D2C", &[b"-65536", b"4"]), b"\xff\xff\x00\x00");
        assert_eq!(answer(b"D2C", &[b"0", b"3"]), b"\x00\x00\x00");
        assert_eq!(answer(b"D2C", &[b"255", b"0"]), b"");

        // A negative value with no length at all, both of them.
        assert_eq!(raised(b"D2X", &[Some(b"-1")]), (93, 927, Vec::new()));
        assert_eq!(raised(b"D2C", &[Some(b"-1")]), (93, 927, Vec::new()));
        // The adjacent success is the same value with any length at all.
        assert_eq!(answer(b"D2X", &[b"-1", b"1"]), b"F");
        // And a non-negative value needs none.
        assert_eq!(answer(b"D2X", &[b"1"]), b"1");
        assert_eq!(
            call(b"D2X", &[Some(b"1"), None]).expect("an omitted length is legal"),
            b"1"
        );
    }

    /// `NUMERIC DIGITS` bounds `C2D`/`X2D` on the result and `D2C`/`D2X` on
    /// the value, and the four sub-codes are distinct.
    ///
    /// Crossed with the length argument, since the window decides what the
    /// result is and therefore whether it fits: measured, `c2d('ff'x,1)` is
    /// `-1` at `DIGITS 1` where `c2d('7f'x,1)` is 127 and does not fit.
    #[test]
    fn the_precision_bounds_the_result_one_way_and_the_value_the_other() {
        assert_eq!(answer_at("9", b"C2D", &[b"\x00\x00\x00\x00\x00\x01"]), b"1");
        assert_eq!(
            raised_at("9", b"C2D", &[Some(b"\xff\xff\xff\xff")]),
            (93, 936, vec![b"9".to_vec()])
        );
        assert_eq!(answer_at("1", b"C2D", &[b"\xff", b"1"]), b"-1");
        assert_eq!(
            raised_at("1", b"C2D", &[Some(b"\x7f"), Some(b"1")]),
            (93, 936, vec![b"1".to_vec()])
        );
        assert_eq!(answer_at("1", b"C2D", &[b"\x00"]), b"0");
        assert_eq!(answer_at("1", b"C2D", &[b"\x00\x00\x00\x00\x00"]), b"0");
        assert_eq!(answer_at("3", b"C2D", &[b"\xff"]), b"255");
        assert_eq!(
            raised_at("3", b"C2D", &[Some(b"\xff\xff")]),
            (93, 936, vec![b"3".to_vec()])
        );
        assert_eq!(answer_at("3", b"X2D", &[b"ff"]), b"255");
        assert_eq!(
            raised_at("3", b"X2D", &[Some(b"ffff")]),
            (93, 935, vec![b"3".to_vec()])
        );
        assert_eq!(answer_at("1", b"X2D", &[b"ff", b"2"]), b"-1");
        assert_eq!(
            raised_at("1", b"X2D", &[Some(b"8f"), Some(b"2")]),
            (93, 935, vec![b"1".to_vec()])
        );

        assert_eq!(answer_at("3", b"D2X", &[b"000123"]), b"7B");
        assert_eq!(answer_at("3", b"D2X", &[b"999"]), b"3E7");
        assert_eq!(answer_at("3", b"D2X", &[b"1E2"]), b"64");
        assert_eq!(
            raised_at("3", b"D2X", &[Some(b"1000")]),
            (93, 928, vec![b"1000".to_vec()])
        );
        assert_eq!(
            raised_at("3", b"D2X", &[Some(b"1E3")]),
            (93, 928, vec![b"1E3".to_vec()])
        );
        assert_eq!(
            raised_at("3", b"D2X", &[Some(b"12300")]),
            (93, 928, vec![b"12300".to_vec()])
        );
        assert_eq!(
            raised_at("3", b"D2C", &[Some(b"12300")]),
            (93, 929, vec![b"12300".to_vec()])
        );
        assert_eq!(answer_at("5", b"D2X", &[b"12345"]), b"3039");
        assert_eq!(answer_at("9", b"D2X", &[b"1E8"]), b"5F5E100");
        assert_eq!(
            raised_at("9", b"D2X", &[Some(b"1E9")]),
            (93, 928, vec![b"1E9".to_vec()])
        );
    }

    /// A value with decimals converts when the decimals are insignificant
    /// *within the current precision*, which makes the same argument legal at
    /// one setting and an error at the next.
    ///
    /// The pair the rule turns on: `1.4` is `1` at `DIGITS 1`, because one
    /// digit of precision drops the `4` and `4` is below the rounding
    /// threshold, and an error at `DIGITS 2`, where the `4` is inside the
    /// precision and is not a zero.
    #[test]
    fn decimals_are_significant_relative_to_the_precision() {
        assert_eq!(answer_at("1", b"D2X", &[b"1.4"]), b"1");
        assert_eq!(
            raised_at("2", b"D2X", &[Some(b"1.4")]),
            (93, 928, vec![b"1.4".to_vec()])
        );
        assert_eq!(
            raised_at("1", b"D2X", &[Some(b"1.6")]),
            (93, 928, vec![b"1.6".to_vec()])
        );
        assert_eq!(answer_at("2", b"D2X", &[b"1.04"]), b"1");
        assert_eq!(
            raised_at("3", b"D2X", &[Some(b"1.04")]),
            (93, 928, vec![b"1.04".to_vec()])
        );
        assert_eq!(answer(b"D2X", &[b"1.0"]), b"1");
        assert_eq!(answer(b"D2X", &[b"1.0000000000004"]), b"1");
        assert_eq!(
            raised(b"D2X", &[Some(b"1.50")]),
            (93, 928, vec![b"1.50".to_vec()])
        );
        assert_eq!(answer(b"D2X", &[b"1200E-2"]), b"C");
        assert_eq!(
            raised(b"D2X", &[Some(b"1234E-2")]),
            (93, 928, vec![b"1234E-2".to_vec()])
        );
        assert_eq!(answer(b"D2X", &[b"1.23E4"]), b"300C");
        assert_eq!(answer(b"D2X", &[b"12.00"]), b"C");
        // Every spelling of zero is the same value, and it is whole.
        for zero in [b"0".as_slice(), b"0.0", b"0.00000", b"-0.0", b"0E5"] {
            assert_eq!(answer_at("1", b"D2X", &[zero]), b"0");
        }
        // A non-zero magnitude below one tenth is never whole, whatever the
        // precision.
        for tiny in [
            b"0.5".as_slice(),
            b"0.01",
            b"1E-9",
            b"1E-100",
            b"0.0000000001",
            b"123E-4",
        ] {
            assert_eq!(raised_at("1", b"D2X", &[Some(tiny)]).1, 928);
            assert_eq!(raised_at("20", b"D2X", &[Some(tiny)]).1, 928);
        }
        // The generous spellings the conversion does accept.
        assert_eq!(answer(b"D2X", &[b"  12  "]), b"C");
        assert_eq!(answer(b"D2X", &[b"+12"]), b"C");
        assert_eq!(answer(b"D2X", &[b"- 12", b"4"]), b"FFF4");
        assert_eq!(answer(b"D2X", &[b"\t12\t"]), b"C");
    }

    /// The scan `D2X`/`D2C` split their argument with accepts every text the
    /// crate's own number parser does.
    ///
    /// The two are separate readers of the same grammar -- one decides
    /// whether the argument is a number at all, the other takes it apart --
    /// and a text the parser accepts and the scan does not would be a
    /// silently wrong *error* where the oracle converts. Nothing else in the
    /// module makes them agree, so this asserts it.
    ///
    /// One direction only, and deliberately: the scan is reached solely for a
    /// text the parser has already accepted, so it is free to be looser. It
    /// is, in exactly one respect -- it does not range-check the assembled
    /// exponent, so `1E999999999999` is a number to it and not to the parser.
    #[test]
    fn a_scan_takes_apart_every_text_the_number_parser_accepts() {
        let subjects: &[&[u8]] = &[
            b"",
            b"0",
            b"1",
            b"-1",
            b"+1",
            b"1.5",
            b".5",
            b"5.",
            b"1e2",
            b"1E+2",
            b"1E-2",
            b"1e",
            b"1e+",
            b"1.2.3",
            b"1 2",
            b"0x1f",
            b" 12 ",
            b"\t12\t",
            b"+ 3",
            b"  +   3  ",
            b"+ .5",
            b"- 12",
            b"+",
            b"-",
            b".",
            b"abc",
            b"1E999999999999",
            b"1E-999999999999",
            b"000123",
            b"12300",
            b"0.0",
            b"1\n2",
            &[0xff],
            &[b'1', 0x00],
            &[b'1', 0x80],
        ];
        let mut interp = interp_at("9");
        for subject in subjects {
            let value = interp.text(subject);
            let parsed = interp.to_number(value).is_ok();
            let scanned = super::scan_decimal(subject).is_some();
            assert!(
                scanned || !parsed,
                "{:?} is a number the scan cannot take apart",
                String::from_utf8_lossy(subject)
            );
        }
    }

    /// `XRANGE`'s twelve class tables, including the one that begins with a
    /// NUL.
    ///
    /// `cntrl` is asserted by its whole content rather than by its length
    /// alone: a table built as a C string would be empty, and one built by
    /// concatenating `00`..`1f` without the `7f` would be 32 bytes, so both
    /// mistakes have to be visible.
    #[test]
    fn every_character_class_is_the_oracles_own_table() {
        let mut cntrl: Vec<u8> = (0x00..=0x1fu8).collect();
        cntrl.push(0x7f);
        assert_eq!(cntrl.len(), 33);
        assert_eq!(answer(b"XRANGE", &[b"cntrl"]), cntrl);
        assert_eq!(answer(b"XRANGE", &[b"CNTRL"]), cntrl);
        assert_eq!(answer(b"XRANGE", &[b"CnTrL"]), cntrl);

        assert_eq!(answer(b"XRANGE", &[b"digit"]), b"0123456789");
        assert_eq!(answer(b"XRANGE", &[b"blank"]), b"\t ");
        assert_eq!(answer(b"XRANGE", &[b"space"]), b"\t\n\x0b\x0c\r ");
        assert_eq!(
            answer(b"XRANGE", &[b"upper"]),
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZ"
        );
        assert_eq!(
            answer(b"XRANGE", &[b"lower"]),
            b"abcdefghijklmnopqrstuvwxyz"
        );
        assert_eq!(
            answer(b"XRANGE", &[b"alpha"]),
            b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz"
        );
        assert_eq!(
            answer(b"XRANGE", &[b"alnum"]),
            b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz"
        );
        assert_eq!(answer(b"XRANGE", &[b"xdigit"]), b"0123456789ABCDEFabcdef");
        assert_eq!(
            answer(b"XRANGE", &[b"punct"]),
            b"!\"#$%&'()*+,-./:;<=>?@[\\]^_`{|}~"
        );
        // `graph` and `print` differ by exactly the leading blank.
        let graph = answer(b"XRANGE", &[b"graph"]);
        let print = answer(b"XRANGE", &[b"print"]);
        assert_eq!(graph.len(), 94);
        assert_eq!(print.len(), 95);
        assert_eq!(print[0], b' ');
        assert_eq!(&print[1..], &graph[..]);
        // Every table is in ascending byte order, which is the property the
        // oracle's own comment claims for all twelve.
        for (name, _) in super::CHARACTER_CLASSES {
            let table = answer(b"XRANGE", &[name]);
            assert!(
                table.windows(2).all(|pair| pair[0] < pair[1]),
                "{} is not in ascending byte order",
                String::from_utf8_lossy(name)
            );
        }
    }

    /// `XRANGE` is variadic over pairs, and the wrap-around is on the byte.
    #[test]
    fn xrange_concatenates_ranges_and_wraps_at_the_top() {
        assert_eq!(answer(b"XRANGE", &[b"a", b"e"]), b"abcde");
        assert_eq!(answer(b"XRANGE", &[b"a", b"a"]), b"a");
        assert_eq!(answer(b"XRANGE", &[b"a", b"b", b"c", b"d"]), b"abcd");
        assert_eq!(
            answer(b"XRANGE", &[b"a", b"b", b"c", b"d", b"e", b"f", b"g", b"h"]),
            b"abcdefgh"
        );
        assert_eq!(answer(b"XRANGE", &[b"\x7f", b"\x80"]), b"\x7f\x80");
        assert_eq!(answer(b"XRANGE", &[b"\xff", b"\x00"]), b"\xff\x00");
        assert_eq!(answer(b"XRANGE", &[b"\xfe", b"\x01"]), b"\xfe\xff\x00\x01");
        assert_eq!(answer(b"XRANGE", &[b"\xff", b"\xff"]), b"\xff");
        assert_eq!(answer(b"XRANGE", &[b"\x80", b"\x7f"]).len(), 256);

        let all: Vec<u8> = (0..=u8::MAX).collect();
        assert_eq!(call(b"XRANGE", &[]).expect("no arguments is legal"), all);
        assert_eq!(answer(b"XRANGE", &[b"\x00", b"\xff"]), all);
        // An omitted end is `'ff'x` and an omitted start is `'00'x`.
        assert_eq!(answer(b"XRANGE", &[b"\x00"]), all);
        assert_eq!(
            call(b"XRANGE", &[None, Some(b"\x04")]).expect("an omitted start is legal"),
            b"\x00\x01\x02\x03\x04"
        );
        assert_eq!(
            call(b"XRANGE", &[Some(b"a"), None, Some(b"c"), Some(b"d")])
                .expect("an omitted end is legal"),
            {
                let mut expected: Vec<u8> = (b'a'..=0xff).collect();
                expected.extend_from_slice(b"cd");
                expected
            }
        );
    }

    /// A class name and a start/end pair mix, and the oracle's own early
    /// return discards a class when the whole call is two arguments.
    ///
    /// The three-argument form is asserted beside the two-argument one,
    /// because that is the only pair that shows the discard is about the
    /// argument count rather than about class names in general.
    #[test]
    fn a_two_argument_call_ending_in_a_range_drops_a_preceding_class() {
        let from_z: Vec<u8> = (b'z'..=0xff).collect();
        assert_eq!(answer(b"XRANGE", &[b"digit", b"z"]), from_z);
        assert_eq!(answer(b"XRANGE", &[b"cntrl", b"z"]), from_z);
        assert_eq!(from_z.len(), 134);

        // Add a third argument and the class comes back.
        let mut expected = b"0123456789".to_vec();
        expected.extend((b'z'..=0xff).chain(0x00..=b'q'));
        assert_eq!(answer(b"XRANGE", &[b"digit", b"z", b"q"]), expected);

        // Two class names never reach the early return at all.
        let mut alpha = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ".to_vec();
        alpha.extend_from_slice(b"abcdefghijklmnopqrstuvwxyz");
        assert_eq!(answer(b"XRANGE", &[b"upper", b"lower"]), alpha);
        let mut alnum = b"0123456789".to_vec();
        alnum.extend_from_slice(&alpha);
        assert_eq!(answer(b"XRANGE", &[b"digit", b"Alpha"]), alnum);
        // A class after a range, and a range after a class, with three or
        // more arguments.
        let mut ab_digits = b"ab".to_vec();
        ab_digits.extend_from_slice(b"0123456789");
        assert_eq!(answer(b"XRANGE", &[b"a", b"b", b"digit"]), ab_digits);
        let mut upper_a_z = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ".to_vec();
        upper_a_z.extend_from_slice(b"abcdefghijklmnopqrstuvwxyz");
        assert_eq!(answer(b"XRANGE", &[b"upper", b"a", b"z"]), upper_a_z);
        // One class on its own answers the class -- and *only* when the call
        // really is one argument long, which is why `xrange('digit',)` is
        // ten bytes: the parser drops the trailing omission before the count
        // is taken.
        assert_eq!(answer(b"XRANGE", &[b"digit"]), b"0123456789");
        assert_eq!(
            call(b"XRANGE", &[Some(b"digit"), None])
                .expect("a second position that is present but omitted is legal")
                .len(),
            256,
            "an omitted second argument is a start/end pair, not a missing one"
        );
    }

    /// `XRANGE`'s two argument positions take different things, and say so
    /// with different sub-codes.
    #[test]
    fn xrange_names_the_position_that_is_wrong_and_what_it_wanted() {
        assert_eq!(
            raised(b"XRANGE", &[Some(b"zork")]),
            (
                40,
                28,
                vec![b"XRANGE".to_vec(), b"1".to_vec(), b"zork".to_vec()]
            )
        );
        assert_eq!(
            raised(b"XRANGE", &[Some(b"")]),
            (40, 28, vec![b"XRANGE".to_vec(), b"1".to_vec(), Vec::new()])
        );
        assert_eq!(
            raised(b"XRANGE", &[Some(b"c1"), Some(b"c1")]),
            (
                40,
                28,
                vec![b"XRANGE".to_vec(), b"1".to_vec(), b"c1".to_vec()]
            )
        );
        assert_eq!(
            raised(b"XRANGE", &[Some(b"a"), Some(b"zz")]),
            (
                40,
                23,
                vec![b"XRANGE".to_vec(), b"2".to_vec(), b"zz".to_vec()]
            )
        );
        assert_eq!(
            raised(b"XRANGE", &[Some(b"a"), Some(b"")]),
            (40, 23, vec![b"XRANGE".to_vec(), b"2".to_vec(), Vec::new()])
        );
        // A class name is not a legal *end*, which is what makes the two
        // positions genuinely different rather than the same check twice.
        assert_eq!(
            raised(b"XRANGE", &[Some(b"a"), Some(b"upper")]),
            (
                40,
                23,
                vec![b"XRANGE".to_vec(), b"2".to_vec(), b"upper".to_vec()]
            )
        );
        // The position the message names is the call's own, past the first
        // pair.
        assert_eq!(
            raised(b"XRANGE", &[Some(b"a"), Some(b"b"), Some(b"zork")]),
            (
                40,
                28,
                vec![b"XRANGE".to_vec(), b"3".to_vec(), b"zork".to_vec()]
            )
        );
        assert_eq!(
            raised(
                b"XRANGE",
                &[Some(b"a"), Some(b"b"), Some(b"c"), Some(b"zz")]
            ),
            (
                40,
                23,
                vec![b"XRANGE".to_vec(), b"4".to_vec(), b"zz".to_vec()]
            )
        );
    }

    /// Every conversion of an argument runs before every check on its
    /// content, and the two families answer at different exit codes.
    #[test]
    fn the_call_layer_is_checked_before_the_operation_layer() {
        // The *length*'s type error beats the value's content error.
        assert_eq!(
            raised(b"D2C", &[Some(b"abc"), Some(b"def")]),
            (
                40,
                12,
                vec![b"D2C".to_vec(), b"2".to_vec(), b"def".to_vec()]
            )
        );
        assert_eq!(
            raised(b"X2D", &[Some(b"ZZ"), Some(b"zz")]),
            (40, 12, vec![b"X2D".to_vec(), b"2".to_vec(), b"zz".to_vec()])
        );
        assert_eq!(
            raised(b"D2X", &[Some(b"abc"), Some(b"zz")]),
            (40, 12, vec![b"D2X".to_vec(), b"2".to_vec(), b"zz".to_vec()])
        );
        // With the length made legal, the content error the 40.12 was hiding.
        assert_eq!(raised(b"X2D", &[Some(b"ZZ"), Some(b"4")]).1, 933);
        assert_eq!(raised(b"D2C", &[Some(b"abc"), Some(b"1")]).1, 929);

        // A negative length is 93.923 -- and for `D2X`/`D2C` it loses to the
        // value's own check, where for `C2D`/`X2D` it wins over the string's.
        assert_eq!(
            raised(b"C2D", &[Some(b"abc"), Some(b"-1")]),
            (93, 923, vec![b"-1".to_vec()])
        );
        assert_eq!(
            raised(b"X2D", &[Some(b"ZZ"), Some(b"-1")]),
            (93, 923, vec![b"-1".to_vec()])
        );
        assert_eq!(
            raised(b"D2X", &[Some(b"1.5"), Some(b"-1")]),
            (93, 923, vec![b"-1".to_vec()])
        );
        assert_eq!(
            raised(b"D2X", &[Some(b"-1"), Some(b"-1")]),
            (93, 923, vec![b"-1".to_vec()])
        );
        assert_eq!(
            raised(b"D2C", &[Some(b"abc"), Some(b"-1")]),
            (93, 929, vec![b"abc".to_vec()])
        );
        assert_eq!(
            raised(b"D2X", &[Some(b"abc"), Some(b"-1")]),
            (93, 928, vec![b"abc".to_vec()])
        );
        assert_eq!(
            raised_at("3", b"D2X", &[Some(b"1000"), Some(b"-1")]),
            (93, 923, vec![b"-1".to_vec()])
        );
        // A pad's own type error, at the position the call spells it.
        assert_eq!(
            raised(b"BITAND", &[Some(b"ab"), Some(b"cd"), Some(b"xx")]),
            (
                40,
                23,
                vec![b"BITAND".to_vec(), b"3".to_vec(), b"xx".to_vec()]
            )
        );
        // A value needing more than the argument precision is not a whole
        // number, however it is spelled.
        assert_eq!(
            answer(b"C2D", &[b"abc", b"1.0000000000000000000004"]),
            b"99"
        );
        assert_eq!(
            raised(b"C2D", &[Some(b"abc"), Some(b"1E18")]),
            (
                40,
                12,
                vec![b"C2D".to_vec(), b"2".to_vec(), b"1E18".to_vec()]
            )
        );
    }

    /// A substitution carries the argument's own bytes, and a byte at or
    /// above `0x80` survives to the report where a control byte becomes `?`.
    #[test]
    fn a_substitution_carries_bytes_and_the_report_makes_them_displayable() {
        assert_eq!(
            raised(b"X2C", &[Some(&[b'4', b'1', 0xff])]),
            (93, 933, vec![vec![0xff]])
        );
        assert_eq!(
            raised(b"X2C", &[Some(&[b'4', b'1', 0x01])]),
            (93, 933, vec![vec![0x01]])
        );
        assert_eq!(
            raised(b"X2C", &[Some(&[b'4', b'1', 0x00])]),
            (93, 933, vec![vec![0x00]])
        );
        assert_eq!(
            raised(b"XRANGE", &[Some(&[0xe9, 0xe9])]),
            (
                40,
                28,
                vec![b"XRANGE".to_vec(), b"1".to_vec(), vec![0xe9, 0xe9]]
            )
        );

        let site = crate::error::ClauseSite {
            sites: &[],
            path: "/p.rex",
        };
        let control = Raised::invalid_digit(crate::error::Notation::Hex, 0x01);
        assert!(
            control.report(&site).windows(3).any(|w| w == *b"\"?\""),
            "a control byte must reach the report as a question mark"
        );
        let high = Raised::invalid_digit(crate::error::Notation::Hex, 0xe9);
        assert!(
            high.report(&site)
                .windows(3)
                .any(|w| w == [b'"', 0xe9, b'"']),
            "a byte at or above 0x80 must reach the report unchanged"
        );
    }

    /// The arity rows, at both ends and at the interior omission each one
    /// admits.
    #[test]
    fn the_arity_rows_are_the_oracles_own() {
        for (name, min, max) in [
            (b"B2X".as_slice(), 1usize, 1usize),
            (b"BITAND", 1, 3),
            (b"BITOR", 1, 3),
            (b"BITXOR", 1, 3),
            (b"C2D", 1, 2),
            (b"C2X", 1, 1),
            (b"D2C", 1, 2),
            (b"D2X", 1, 2),
            (b"X2B", 1, 1),
            (b"X2C", 1, 1),
            (b"X2D", 1, 2),
        ] {
            let short: Vec<Option<&[u8]>> = vec![Some(b"1"); min - 1];
            assert_eq!(
                raised(name, &short),
                (40, 3, vec![name.to_vec(), min.to_string().into_bytes()]),
                "{} did not name its minimum",
                String::from_utf8_lossy(name)
            );
            let long: Vec<Option<&[u8]>> = vec![Some(b"1"); max + 1];
            assert_eq!(
                raised(name, &long),
                (40, 4, vec![name.to_vec(), max.to_string().into_bytes()]),
                "{} did not name its maximum",
                String::from_utf8_lossy(name)
            );
            // An omission in the one required position is 40.5 -- but only
            // where a second argument fits at all. A row with a maximum of 1
            // answers 40.4 for the same call, since the maximum is checked
            // first: measured, `b2x(,'x')` is 40.4 and `c2d(,'1')` is 40.5.
            let expected = if max >= 2 { 5 } else { 4 };
            let substitutions = if max >= 2 {
                b"1".to_vec()
            } else {
                max.to_string().into_bytes()
            };
            assert_eq!(
                raised(name, &[None, Some(b"1")]),
                (40, expected, vec![name.to_vec(), substitutions]),
                "{} answered the wrong sub-code for an omitted first argument",
                String::from_utf8_lossy(name)
            );
        }
        // `XRANGE` has neither end: a minimum of 0 and no maximum at all, so
        // no argument list is ever too long and none is ever too short.
        assert_eq!(
            call(b"XRANGE", &[]).expect("no arguments is legal").len(),
            256
        );
        assert_eq!(
            call(b"XRANGE", &[Some(b"a".as_slice()); 12])
                .expect("twelve arguments are not too many")
                .len(),
            6
        );
        assert_eq!(
            call(b"XRANGE", &[None, Some(b"\x00")]).expect("an omitted first is legal"),
            b"\x00"
        );
    }

    /// Every result is text, so a later `NUMERIC DIGITS` cannot reshape it.
    ///
    /// The mutation this catches is building `C2D`'s or `X2D`'s answer
    /// through `Interp::number` under the settings in force, which would
    /// render one million as `1E+6` for a caller that later drops to
    /// `DIGITS 1`. Only the numeric pair can show it, since the others never
    /// produce anything a `DIGITS` setting could reshape.
    #[test]
    fn a_converted_number_is_text_that_no_later_digits_setting_reshapes() {
        for (name, argument, expected) in [
            (
                b"C2D".as_slice(),
                b"\x0f\x42\x40".as_slice(),
                b"1000000".as_slice(),
            ),
            (b"X2D", b"0f4240", b"1000000"),
            (b"D2X", b"1000000", b"F4240"),
        ] {
            // A fresh interpreter per case, because the drop to `DIGITS 1`
            // below cannot be undone: a candidate setting is judged against
            // the precision in force, so `12` is not a whole number at 1.
            let mut interp = interp_at("12");
            let value = interp.text(argument);
            let result = dispatch(&mut interp, name, &[Some(value)])
                .expect("a builtin name")
                .expect("this call succeeds");
            interp
                .activation_mut()
                .settings
                .set_digits_str("1")
                .expect("a legal DIGITS setting");
            assert_eq!(
                interp.to_text(result).into_owned(),
                expected,
                "{} was reshaped by a later precision",
                String::from_utf8_lossy(name)
            );
        }
    }

    /// A result sized from an argument is refused rather than aborting the
    /// process.
    #[test]
    fn a_result_too_large_to_allocate_raises_the_oracles_error_5() {
        for name in [b"D2X".as_slice(), b"D2C"] {
            let failure = call(name, &[Some(b"1"), Some(b"123456789012345678")])
                .expect_err("that length cannot be allocated");
            let Failure::Raised(raised) = failure else {
                panic!("expected Raised, got {failure:?}");
            };
            assert_eq!((raised.number, raised.sub), (5, 0));
        }
        // The adjacent success: a length that is merely large is honoured.
        assert_eq!(answer(b"D2X", &[b"1", b"1000"]).len(), 1000);
        // And a length larger than the string is not a size at all for the
        // pair that reads rather than writes.
        assert_eq!(answer(b"C2D", &[b"\xff", b"123456789012345678"]), b"255");
    }
}
