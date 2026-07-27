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

//! Rexx decimal arithmetic.
//!
//! A Rexx number is a string that happens to be numeric, and the round trip
//! through string form is observable everywhere -- so this is a decimal
//! representation carrying its own digits, not a binary float.
//!
//! The behaviour reproduced here was measured against ooRexx 5.3.0 rather
//! than taken from the standard; where they differ, the interpreter wins.
//! See `rust/corpus/num/` for the programs that pin it.

mod addsub;
mod compare;
mod muldiv;
mod pow;
pub use compare::{compare, CompareOp};
pub use muldiv::DivOp;
mod settings;
pub use settings::{Form, Settings, SettingsError};

/// Rexx's default `NUMERIC DIGITS`.
pub const DEFAULT_DIGITS: u32 = 9;

/// The largest adjusted exponent a Rexx number may have. Beyond this a
/// literal will not convert (error 41) and an arithmetic result overflows
/// (error 42). `Numerics.hpp:113`.
pub const MAX_EXPONENT: i32 = 999_999_999;
pub const MIN_EXPONENT: i32 = -999_999_999;

/// What arithmetic can fail with, carrying the interpreter's error numbers.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum ArithError {
    /// Result exponent outside +/- MAX_EXPONENT. Error 42.
    Overflow,
    /// Error 42.
    DivideByZero,
    /// A `%` or `//` result too wide to be whole at this DIGITS. Error 26.
    NotWholeNumber,
}

impl ArithError {
    pub fn code(self) -> u16 {
        match self {
            ArithError::Overflow | ArithError::DivideByZero => 42,
            ArithError::NotWholeNumber => 26,
        }
    }
}

/// A decimal number: `digits * 10^exponent`, with a sign.
///
/// `digits` keeps trailing zeros, because Rexx does: `1.50 + 0` displays as
/// `1.50`, not `1.5`. Normalising them away would be the single easiest way
/// to break conformance across most numeric output.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Number {
    pub(crate) negative: bool,
    /// Most significant first, each value 0..=9. Never empty. Has no leading
    /// zero unless the value is zero, in which case it is exactly `[0]`.
    pub(crate) digits: Vec<u8>,
    pub(crate) exponent: i32,
}

impl Number {
    /// The canonical zero. Every spelling of zero collapses to this: the
    /// oracle prints `0` for `-0`, `0.0` and `00.00` alike.
    pub fn zero() -> Self {
        Number { negative: false, digits: vec![0], exponent: 0 }
    }

    pub fn is_zero(&self) -> bool {
        self.digits.iter().all(|d| *d == 0)
    }

    /// Parses a Rexx number, or `None` if the string is not one.
    ///
    /// Accepts surrounding whitespace, an optional sign, digits with an
    /// optional decimal point, and an optional exponent. Rejects everything
    /// else -- notably a bare sign, a bare exponent marker, and hex literals,
    /// which are strings in Rexx rather than numbers.
    pub fn parse(text: &str) -> Option<Self> {
        let s = text.trim();
        if s.is_empty() {
            return None;
        }
        let bytes = s.as_bytes();
        let mut i = 0;

        let negative = match bytes[i] {
            b'+' => {
                i += 1;
                false
            }
            b'-' => {
                i += 1;
                true
            }
            _ => false,
        };

        let mut digits: Vec<u8> = Vec::new();
        let mut seen_digit = false;
        let mut decimals: i32 = 0;
        let mut seen_point = false;

        while i < bytes.len() {
            match bytes[i] {
                b'0'..=b'9' => {
                    seen_digit = true;
                    digits.push(bytes[i] - b'0');
                    if seen_point {
                        decimals += 1;
                    }
                    i += 1;
                }
                b'.' if !seen_point => {
                    seen_point = true;
                    i += 1;
                }
                _ => break,
            }
        }
        if !seen_digit {
            return None;
        }

        let mut exponent = -decimals;
        let mut written_exponent: Option<i64> = None;
        if i < bytes.len() && (bytes[i] == b'e' || bytes[i] == b'E') {
            i += 1;
            let exp_negative = match bytes.get(i) {
                Some(b'+') => {
                    i += 1;
                    false
                }
                Some(b'-') => {
                    i += 1;
                    true
                }
                _ => false,
            };
            let start = i;
            let mut value: i64 = 0;
            while i < bytes.len() && bytes[i].is_ascii_digit() {
                value = value.saturating_mul(10).saturating_add((bytes[i] - b'0') as i64);
                i += 1;
            }
            if i == start {
                return None; // "1e", "1e+"
            }
            let signed = if exp_negative { -value } else { value };
            // The exponent as *written* must itself be in range, separately
            // from the range check on the assembled number. Without this,
            // `.235468758140e1000000000` looks fine once the twelve decimal
            // places are folded in, but the interpreter never gets that far.
            // Zero is exempt: `0e1000000996` is simply 0.
            written_exponent = Some(signed);
            exponent = exponent.saturating_add(signed.clamp(
                MIN_EXPONENT as i64 * 2,
                MAX_EXPONENT as i64 * 2,
            ) as i32);
        }

        if i != bytes.len() {
            return None; // trailing junk: "1.2.3", "1 2", "0x1f"
        }

        let assembled = Self::assemble(negative, digits, exponent);
        // A literal outside the representable range is not a number at all:
        // the interpreter reports error 41 rather than an overflow. Zero is
        // exempt from both checks -- it has no magnitude to be out of range.
        if !assembled.is_zero() {
            if let Some(written) = written_exponent
                && !(MIN_EXPONENT as i64..=MAX_EXPONENT as i64).contains(&written)
            {
                return None;
            }
            if !assembled.in_range() {
                return None;
            }
        }
        Some(assembled)
    }

    /// True when every digit of the number lies within 10^±`MAX_EXPONENT`.
    ///
    /// The two ends are tested against different exponents, which is the same
    /// asymmetry the display thresholds use. The most significant digit sits
    /// at the *adjusted* exponent and must not exceed the maximum; the least
    /// significant sits at the *raw* exponent and must not fall below the
    /// minimum. Testing one exponent at both ends accepts numbers the
    /// interpreter rejects -- `123456789e999999999` at the top,
    /// `.96329e-999999995` at the bottom.
    pub(crate) fn in_range(&self) -> bool {
        self.adjusted_exponent() <= MAX_EXPONENT && self.exponent >= MIN_EXPONENT
    }

    /// Fails with `Overflow` when an arithmetic result has run outside the
    /// representable range, which the interpreter reports as error 42.
    pub(crate) fn check_range(self) -> Result<Self, ArithError> {
        if self.is_zero() || self.in_range() {
            Ok(self)
        } else {
            Err(ArithError::Overflow)
        }
    }

    /// Strips leading zeros and collapses any zero to the canonical form.
    pub(crate) fn assemble(negative: bool, mut digits: Vec<u8>, exponent: i32) -> Self {
        if digits.iter().all(|d| *d == 0) {
            return Number::zero();
        }
        let lead = digits.iter().take_while(|d| **d == 0).count();
        digits.drain(..lead);
        Number { negative, digits, exponent }
    }

    /// The power of ten of the most significant digit. This is what the
    /// display thresholds are expressed in terms of.
    fn adjusted_exponent(&self) -> i32 {
        self.exponent.saturating_add(self.digits.len() as i32 - 1)
    }

    /// Rounds to at most `digits` significant digits, half-up.
    ///
    /// Rounding is an arithmetic operation, not a display one -- it happens at
    /// the `DIGITS` boundary when a result is produced. It is exposed here
    /// because every operator needs it.
    pub fn round_to(&self, digits: u32) -> Self {
        let keep = digits as usize;
        if keep == 0 || self.digits.len() <= keep || self.is_zero() {
            return self.clone();
        }
        let dropped = self.digits.len() - keep;
        let mut kept: Vec<u8> = self.digits[..keep].to_vec();
        let mut exponent = self.exponent + dropped as i32;

        if self.digits[keep] >= 5 {
            // Propagate the carry; an all-nines mantissa grows a digit and
            // sheds the one it no longer has room for.
            let mut i = keep;
            loop {
                if i == 0 {
                    kept.insert(0, 1);
                    kept.pop();
                    exponent += 1;
                    break;
                }
                i -= 1;
                if kept[i] == 9 {
                    kept[i] = 0;
                } else {
                    kept[i] += 1;
                    break;
                }
            }
        }
        Self::assemble(self.negative, kept, exponent)
    }

    /// Renders the number as the interpreter would at this `DIGITS` setting.
    ///
    /// Rounds first, because that is what an arithmetic result does before it
    /// is ever seen, then chooses plain or exponential form. The two
    /// thresholds are asymmetric in *two* ways, and the second one is easy to
    /// miss:
    ///
    /// - exponential when the **adjusted** exponent is `>= digits`
    /// - exponential when the **raw** exponent is `<= -(2 * digits + 1)`
    ///
    /// The adjusted exponent is that of the most significant digit; the raw
    /// exponent is that of the least significant. They coincide only for a
    /// single-digit mantissa, which is exactly why probing with `1eN` values
    /// alone suggests both sides use the adjusted one. They do not:
    /// `1e-18` prints in plain form while `10e-19` -- the same value -- prints
    /// as `1.0E-18`, because their raw exponents are -18 and -19.
    ///
    /// Found by differentially testing 1,674 inputs against the interpreter;
    /// 78 disagreed, all of them here.
    pub fn format(&self, digits: u32) -> String {
        let n = self.round_to(digits);
        if n.is_zero() {
            return "0".to_string();
        }
        let adjusted = n.adjusted_exponent();
        let sign = if n.negative { "-" } else { "" };
        let d: String = n.digits.iter().map(|x| (b'0' + x) as char).collect();

        if adjusted >= digits as i32 || n.exponent <= -(2 * digits as i32 + 1) {
            let mantissa = if d.len() == 1 {
                d
            } else {
                format!("{}.{}", &d[..1], &d[1..])
            };
            let e_sign = if adjusted < 0 { '-' } else { '+' };
            return format!("{sign}{mantissa}E{e_sign}{}", adjusted.abs());
        }

        if n.exponent >= 0 {
            return format!("{sign}{d}{}", "0".repeat(n.exponent as usize));
        }
        let point = n.digits.len() as i32 + n.exponent;
        if point > 0 {
            let point = point as usize;
            format!("{sign}{}.{}", &d[..point], &d[point..])
        } else {
            format!("{sign}0.{}{d}", "0".repeat((-point) as usize))
        }
    }
}

impl std::fmt::Display for Number {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.format(DEFAULT_DIGITS))
    }
}
