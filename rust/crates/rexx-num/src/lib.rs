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

use std::borrow::Cow;

mod addsub;
mod compare;
mod digits;
mod muldiv;
mod pow;
mod rounding;

pub use compare::{
    CompareOp, compare, compare_bytes, compare_decoded, compare_numbers, compare_strings,
};
pub(crate) use digits::Digits;
pub use muldiv::DivOp;
mod settings;
pub use settings::{Form, Settings, SettingsError};
mod format;
pub use format::FormatError;

/// Rexx's default `NUMERIC DIGITS`.
pub const DEFAULT_DIGITS: u64 = 9;

/// The largest adjusted exponent a Rexx number may have. Beyond this a
/// literal will not convert (error 41) and an arithmetic result overflows
/// (error 42). `Numerics.hpp:113`.
pub const MAX_EXPONENT: i32 = 999_999_999;
pub const MIN_EXPONENT: i32 = -999_999_999;

/// The precision an argument is converted to a machine integer under, and the
/// width of that integer in decimal digits.
pub const ARGUMENT_DIGITS: usize = 18;

/// What arithmetic can fail with, carrying the interpreter's error numbers.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ArithError {
    /// Result exponent above `MAX_EXPONENT`, from `mul`/`div`/`add`/`sub`/
    /// `pow`'s shared final range check (`check_range`, below). Error
    /// 42.901; `additional()` is `[adjusted_exponent, "9"]`.
    Overflow { adjusted_exponent: i32 },
    /// Result exponent below `MIN_EXPONENT`, same call sites as `Overflow`.
    /// Error 42.902; `additional()` is `[exponent, "9"]` -- the *raw*
    /// exponent, not the adjusted one `Overflow` uses.
    Underflow { exponent: i32 },
    /// Zero raised to a negative power: an underflow, not infinity. Error
    /// 42.903, no substitution. Raised only by `pow.rs`.
    ZeroToNegativePower,
    /// `**`'s own upfront magnitude check, refusing a hopeless computation
    /// before attempting it. Error 42.001; `additional()` is `[base, "**",
    /// exponent]`. Raised only by `pow.rs`.
    PowerOverflow { base: Number, exponent: Number },
    /// A zero divisor, for `/`, `%`, or `//` alike. Error 42.003, no
    /// substitution.
    DivideByZero,
    /// Three `checked_add`/`checked_sub` guards in `muldiv.rs`, believed
    /// unreachable (both operand exponents are already bounded to
    /// +/-`MAX_EXPONENT`, so their sum/difference cannot overflow `i32` in
    /// practice). No substitution, because there is no valid exponent left
    /// to report if this ever does fire -- that is the failure itself.
    /// Falls back to the bare 42.000 text, which the interpreter itself
    /// never emits (grepped: no `reportException` call uses bare
    /// `Error_Overflow`).
    ExponentComputationOverflow,
    /// `%`'s quotient needs more digits than DIGITS allows. Error 26.011,
    /// no substitution. Raised only by `muldiv.rs`'s `div`.
    IntegerDivideNotWhole,
    /// `//`'s quotient needs more digits than DIGITS allows. Error 26.012,
    /// no substitution. Raised only by `muldiv.rs`'s `div`.
    RemainderNotWhole,
    /// `**`'s exponent, after rounding to DIGITS, is not a whole number.
    /// Error 26.008; `additional()` is `[exponent]`. Raised only by
    /// `pow.rs`.
    PowerExponentNotWhole { exponent: Number },
    /// An up-front working-storage reservation sized by DIGITS failed, so a
    /// huge -- and, since the u64 widening, perfectly legal -- DIGITS fails
    /// here before any arithmetic starts. Error 5, no substitution.
    SystemResources,
}

impl ArithError {
    /// The major number a trapped Rexx program sees in `RC` (`condition('o')
    /// ~code` is this pair's major half). For the *sub*-number too -- what
    /// distinguishes 42.3 from 42.901 and 42.902, all of which are major 42
    /// -- see [`ArithError::sub_code`], which also returns this same major
    /// alongside it.
    pub fn code(self) -> u16 {
        match self {
            ArithError::Overflow { .. }
            | ArithError::Underflow { .. }
            | ArithError::ZeroToNegativePower
            | ArithError::PowerOverflow { .. }
            | ArithError::DivideByZero
            | ArithError::ExponentComputationOverflow => 42,
            ArithError::IntegerDivideNotWhole
            | ArithError::RemainderNotWhole
            | ArithError::PowerExponentNotWhole { .. } => 26,
            ArithError::SystemResources => 5,
        }
    }

    /// The `(major, sub)` pair identifying this failure's exact entry in
    /// the generated message table -- `code()` only ever exposes `major`
    /// (the interpreter number a trapped Rexx program sees in `RC`), so
    /// `message`/`additional` need this to pick the right table row, and so
    /// does any other caller that has to report the sub-number a trapped
    /// `SYNTAX` condition would carry (`condition('o')~code`'s minor half).
    pub fn sub_code(&self) -> (u16, u16) {
        match self {
            ArithError::Overflow { .. } => (42, 901),
            ArithError::Underflow { .. } => (42, 902),
            ArithError::ZeroToNegativePower => (42, 903),
            ArithError::PowerOverflow { .. } => (42, 1),
            ArithError::DivideByZero => (42, 3),
            ArithError::ExponentComputationOverflow => (42, 0),
            ArithError::IntegerDivideNotWhole => (26, 11),
            ArithError::RemainderNotWhole => (26, 12),
            ArithError::PowerExponentNotWhole { .. } => (26, 8),
            // The bare major: `Error_System_resources` is message number 5
            // itself, entered in the table as sub 0.
            ArithError::SystemResources => (5, 0),
        }
    }

    /// The substitution values in the interpreter's own order -- what
    /// `condition('o')~additional` returns for this failure: `[5, 10]` for
    /// a `FuzzNotBelowDigits`-shaped 33.001, `[]` (an *empty* array, not
    /// absent) for a no-substitution message like `DivideByZero`.
    pub fn additional(&self) -> Vec<String> {
        match self {
            ArithError::Overflow { adjusted_exponent } => {
                vec![adjusted_exponent.to_string(), "9".to_string()]
            }
            ArithError::Underflow { exponent } => vec![exponent.to_string(), "9".to_string()],
            ArithError::PowerOverflow { base, exponent } => {
                vec![
                    full_precision(base),
                    "**".to_string(),
                    full_precision(exponent),
                ]
            }
            ArithError::PowerExponentNotWhole { exponent } => vec![full_precision(exponent)],
            ArithError::ZeroToNegativePower
            | ArithError::DivideByZero
            | ArithError::ExponentComputationOverflow
            | ArithError::IntegerDivideNotWhole
            | ArithError::RemainderNotWhole
            | ArithError::SystemResources => vec![],
        }
    }

    /// The interpreter's message text for this failure, rendered from the
    /// generated table on demand -- every sub-message verified against
    /// `build/bin/rexx`.
    pub fn message(&self) -> String {
        let (major, sub) = self.sub_code();
        let subs = self.additional();
        let refs: Vec<&str> = subs.iter().map(String::as_str).collect();
        error_text(major, sub, &refs)
    }
}

/// Renders `n` using every digit it stores, rather than the `DEFAULT_DIGITS`
/// (9) `Number::format`/`Display` uses. Only for `ArithError` substitutions
/// that echo an operand back -- see `ArithError::message`'s doc comment for
/// why even this cannot be byte-exact against the interpreter in general
/// (it fixes needless rounding, not a `Number`'s already-lost original
/// spelling: no `+` after `E`, leading zeros, and so on).
fn full_precision(n: &Number) -> String {
    n.format(n.digits.len() as u64)
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

/// [`Number::whole_value`] for a value a caller already holds as an `i64`,
/// where that answer needs no [`Number`] built to reach it.
pub fn whole_i64(value: i64, digits: usize) -> Option<i64> {
    (value.unsigned_abs() <= max_value_for_digits(digits) as u64).then_some(value)
}

/// `NumberString::createUnsignedValue` (`NumberStringClass.cpp:788`): the first
/// `length` digits, plus a carry, scaled by `10^exponent`.
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

/// The `digits + 1` working length every operator truncates its operands to,
/// as a `usize` for slicing. Saturates instead of wrapping: `digits` is a
/// bare `u64` at the public boundary, and a value near `u64::MAX` must mean
/// "keep everything", not a wrapped-around tiny precision -- the same
/// silent-narrowing class `format`, `as_whole` and `div` each had to fix
/// individually.
pub(crate) fn working_length(digits: u64) -> usize {
    usize::try_from(digits.saturating_add(1)).unwrap_or(usize::MAX)
}

/// Fills `&1`, `&2`, … placeholders in a generated-table message with
/// `subs`, in the order the interpreter's own substitution positions use.
/// `rexx-inventory`'s table keeps them literal -- see its build script's
/// `<Sub position="N"/>` rendering rule -- so filling them in is this
/// crate's job, not the generator's.
pub(crate) fn substitute(text: &str, subs: &[&str]) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.char_indices().peekable();
    while let Some((i, c)) = chars.next() {
        if c != '&' {
            out.push(c);
            continue;
        }
        let digits_start = i + c.len_utf8();
        let mut digits_end = digits_start;
        while let Some(&(j, d)) = chars.peek() {
            if !d.is_ascii_digit() {
                break;
            }
            digits_end = j + d.len_utf8();
            chars.next();
        }
        match text[digits_start..digits_end].parse::<usize>() {
            Ok(n) if n >= 1 && n <= subs.len() => out.push_str(subs[n - 1]),
            // No digits at all, or a position this call didn't supply a
            // substitution for: pass the `&` and whatever digits followed
            // it through unchanged rather than silently dropping them.
            _ => {
                out.push('&');
                out.push_str(&text[digits_start..digits_end]);
            }
        }
    }
    out
}

#[cfg(test)]
mod substitute_tests {
    use super::substitute;

    #[test]
    fn fills_placeholders_in_order() {
        assert_eq!(substitute("&1 and &2", &["a", "b"]), "a and b");
    }

    #[test]
    fn a_substitution_value_shaped_like_a_placeholder_does_not_get_rewritten_again() {
        // The bug a sequential `str::replace` implementation has: filling
        // &1 with the literal text "&2" must not make the *original* &2
        // placeholder -- or the newly-inserted "&2" -- collide.
        assert_eq!(substitute("… &1 … &2", &["&2", "X"]), "… &2 … X");
    }

    #[test]
    fn an_unsupplied_or_malformed_placeholder_passes_through_unchanged() {
        assert_eq!(substitute("&1 &9 &", &["a"]), "a &9 &");
    }
}

/// Looks up `major.sub` in the generated message table and fills its
/// placeholders from `subs`. Every `(major, sub)` this crate passes is a
/// literal confirmed against `build/bin/rexx`; a lookup failure here would
/// mean the table changed under a verified mapping, not something to
/// recover from at runtime.
pub(crate) fn error_text(major: u16, sub: u16, subs: &[&str]) -> String {
    let m = rexx_inventory::errors::lookup(major, sub)
        .unwrap_or_else(|| panic!("no interpreter message for {major}.{sub:03}"));
    substitute(m.text, subs)
}

/// A decimal number: `digits * 10^exponent`, with a sign.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Number {
    pub(crate) negative: bool,
    /// Most significant first, each value 0..=9. Never empty. Has no leading
    /// zero unless the value is zero, in which case it is exactly `[0]`.
    pub(crate) digits: Digits,
    pub(crate) exponent: i32,
}

/// A `Number` at 40 bytes is what `rexx-core`'s `Body::Num` absorbs without
/// `Body` growing, and `Body`'s width is every arena slot's width. Measured
/// either side of the inline digit buffer landing: `Number` 32 then 40, with
/// `Body` 80 and `Slot` 96 both times.
const _: () = assert!(size_of::<Number>() <= 40);

impl Number {
    /// The canonical zero. Every spelling of zero collapses to this: the
    /// oracle prints `0` for `-0`, `0.0` and `00.00` alike.
    #[inline(always)]
    pub const fn zero() -> Self {
        Number {
            negative: false,
            digits: Digits::single(0),
            exponent: 0,
        }
    }

    #[inline(always)]
    /// How many significant digits this number stores, trailing zeros
    /// included -- `NumberString`'s own `digitsCount`
    /// (`classes/NumberStringClass.hpp`), which is what the interpreter
    /// compares against `DIGITS` to decide that an operand loses digits.
    pub fn digit_count(&self) -> usize {
        self.digits.len()
    }

    pub fn is_zero(&self) -> bool {
        self.digits.iter().all(|d| *d == 0)
    }

    /// The number `value` denotes, built directly.
    pub fn from_i64(value: i64) -> Self {
        if value == 0 {
            return Number::zero();
        }
        // `unsigned_abs`, not `-value`: `i64::MIN` has no positive form.
        let mut magnitude = value.unsigned_abs();
        // An `i64` is at most nineteen decimal digits, which is one of the two
        // bounds `INLINE_DIGITS` is chosen above, so this never allocates.
        let len = magnitude.ilog10() as usize + 1;
        let mut digits = Digits::zeros(len);
        // **The slice is taken once.** `Digits` reaches `[u8]` through
        // `DerefMut`, so writing `digits[cursor]` inside the loop re-matches
        // the storage arm and bounds-checks the length for every digit --
        // which costs more than the reversing pass it was meant to replace.
        // Measured per call over a harness that hashes each result, so the
        // differences rather than the absolutes are the claim: a one-digit
        // value costs 102.0 instructions built by pushing and reversing,
        // 105.0 written in place through a fresh index per digit, and 88.0
        // written in place through this slice.
        let out = &mut digits[..];
        let mut cursor = len;
        while cursor > 0 {
            cursor -= 1;
            out[cursor] = (magnitude % 10) as u8;
            magnitude /= 10;
        }
        Number {
            negative: value < 0,
            digits,
            exponent: 0,
        }
    }

    /// The integer this is already written as, at `digits` precision, or
    /// `None` when it is not written that way.
    pub fn plain_integer(&self, digits: u64) -> Option<i64> {
        let exponent = u64::try_from(self.exponent).ok()?;
        let width = self.digits.len() as u64 + exponent;
        // 19 caps the multiply loop below before `digits` -- which a program
        // may set to billions -- can turn a rejection into a long one.
        if width > digits || width > 19 {
            return None;
        }
        let mut value: i64 = 0;
        for digit in &self.digits {
            value = value.checked_mul(10)?.checked_add(i64::from(*digit))?;
        }
        for _ in 0..exponent {
            value = value.checked_mul(10)?;
        }
        Some(if self.negative { -value } else { value })
    }

    /// The integer this **renders** as at `digits` precision, or `None` when
    /// its rendering is not a plain run of decimal digits.
    pub fn rendered_integer(&self, digits: u64) -> Option<i64> {
        // `round_to`'s three no-op cases, tested rather than taken: a
        // sentinel `digits`, a value already inside the precision, and a
        // zero all leave the number alone.
        let keep = usize::try_from(digits).unwrap_or(usize::MAX);
        let rounds = keep != 0 && self.digits.len() > keep && !self.is_zero();

        // What rounding does to the shape. `round_to` holds the digit count
        // at `keep` and lets the exponent absorb both the dropped digits and
        // an all-nines carry, so the rounded value's length and exponent are
        // known without rounding anything.
        let round_up = rounds && self.digits[keep] >= 5;
        let grew = round_up && self.digits[..keep].iter().all(|d| *d == 9);
        let (length, exponent) = if rounds {
            (
                keep,
                i64::from(self.exponent) + (self.digits.len() - keep) as i64 + i64::from(grew),
            )
        } else {
            (self.digits.len(), i64::from(self.exponent))
        };

        // The two form questions, in `format_with`'s own order. `digits` is
        // saturated into i64 rather than narrowed, for the reason `format`
        // gives at its own comparison.
        let adjusted = exponent + length as i64 - 1;
        let trigger = i64::try_from(digits).unwrap_or(i64::MAX);
        let plain = if adjusted >= trigger
            || (adjusted < 0 && exponent.abs() > trigger.saturating_mul(2))
        {
            // Exponential: the mantissa carries a point unless it is one
            // digit wide, and an `E` follows unless the exponent to be
            // displayed is exactly zero, which is never written.
            length == 1 && adjusted == 0
        } else {
            // Plain: a point is written exactly when digits sit to the right
            // of it, which is what a negative exponent means.
            exponent >= 0
        };
        if !plain {
            return None;
        }

        if length as i64 + exponent > 19 {
            // Wider than any `i64`, so the parse this replaces failed -- with
            // one exception, a run of zeros, which is zero at any width. That
            // arm answers for a shape the canonical zero (one digit, exponent
            // zero) never has, and it is written rather than argued away so
            // that this is a function of the shape alone.
            return self.digits[..length].iter().all(|d| *d == 0).then_some(0);
        }
        // At most 19 digits and at most 19 trailing zeros from here, so the
        // accumulation cannot leave i128 and every `i64` -- `i64::MIN`
        // included, which has no positive form -- is reached exactly.
        let mut value: i128 = 0;
        for &digit in &self.digits[..length] {
            value = value * 10 + i128::from(digit);
        }
        if grew {
            // The all-nines carry: `round_to` writes a `1` and then zeros,
            // and the extra position is already in `exponent` above.
            value = 10i128.pow(length as u32 - 1);
        } else if round_up {
            value += 1;
        }
        for _ in 0..exponent {
            value *= 10;
        }
        if self.negative {
            value = -value;
        }
        i64::try_from(value).ok()
    }

    /// The same magnitude with the sign cleared, which is what `ABS` needs
    /// and nothing here could otherwise express: `negative` is private to
    /// this crate.
    pub fn abs(&self) -> Self {
        Number {
            negative: false,
            digits: self.digits.clone(),
            exponent: self.exponent,
        }
    }

    /// `-1`, `0` or `1`, which is what `SIGN` answers.
    pub fn signum(&self) -> i8 {
        if self.is_zero() {
            0
        } else if self.negative {
            -1
        } else {
            1
        }
    }

    /// Parses a Rexx number, or `None` if the string is not one.
    pub fn parse(text: &str) -> Option<Self> {
        Self::parse_bytes(text.as_bytes())
    }

    /// [`parse`], over the bytes a Rexx string actually is.
    pub fn parse_bytes(bytes: &[u8]) -> Option<Self> {
        fn is_blank(byte: u8) -> bool {
            byte == b' ' || byte == b'\t'
        }
        let mut i = 0;
        while i < bytes.len() && is_blank(bytes[i]) {
            i += 1;
        }
        if i == bytes.len() {
            return None;
        }

        let signed = bytes[i] == b'+' || bytes[i] == b'-';
        let negative = bytes[i] == b'-';
        if signed {
            i += 1;
            // Blanks are allowed between the sign and the digits -- the sign
            // branch of `numberStringScan` has its own skip loop. Without a
            // sign there is nothing to skip: the leading skip above already
            // ran, so a blank here is simply not part of a number.
            while i < bytes.len() && is_blank(bytes[i]) {
                i += 1;
            }
        }

        let mut digits = Digits::new();
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
                value = value
                    .saturating_mul(10)
                    .saturating_add((bytes[i] - b'0') as i64);
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
            exponent = exponent.saturating_add(
                signed.clamp(MIN_EXPONENT as i64 * 2, MAX_EXPONENT as i64 * 2) as i32,
            );
        }

        // Trailing blanks are legal; anything else after them is not.
        while i < bytes.len() && is_blank(bytes[i]) {
            i += 1;
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
    #[inline(always)]
    pub(crate) fn in_range(&self) -> bool {
        self.adjusted_exponent() <= MAX_EXPONENT && self.exponent >= MIN_EXPONENT
    }

    /// Fails with `Overflow`/`Underflow` when an arithmetic result has run
    /// outside the representable range, which the interpreter reports as
    /// error 42.
    #[inline(always)]
    pub(crate) fn check_range(&self) -> Result<(), ArithError> {
        if self.is_zero() || self.in_range() {
            return Ok(());
        }
        if self.adjusted_exponent() > MAX_EXPONENT {
            Err(ArithError::Overflow {
                adjusted_exponent: self.adjusted_exponent(),
            })
        } else {
            Err(ArithError::Underflow {
                exponent: self.exponent,
            })
        }
    }

    /// Strips leading zeros and collapses any zero to the canonical form.
    #[inline(always)]
    pub(crate) fn assemble(negative: bool, mut digits: Digits, exponent: i32) -> Self {
        let lead = digits.iter().take_while(|d| **d == 0).count();
        if lead == digits.len() {
            return Number::zero();
        }
        digits.drop_front(lead);
        Number {
            negative,
            digits,
            exponent,
        }
    }

    /// The power of ten of the most significant digit. This is what the
    /// display thresholds are expressed in terms of.
    #[inline(always)]
    fn adjusted_exponent(&self) -> i32 {
        self.exponent.saturating_add(self.digits.len() as i32 - 1)
    }

    /// The value as a machine integer under `digits` precision, or `None` if it
    /// has none.
    pub fn whole_value(&self, digits: usize) -> Option<i64> {
        // `isZero()`: every spelling of zero converts to zero whatever the
        // exponent says.
        if self.is_zero() {
            return Some(0);
        }
        let sign: i64 = if self.negative { -1 } else { 1 };
        let max = max_value_for_digits(digits);
        let precision = i64::try_from(digits).ok()?;
        let mut length = i64::try_from(self.digits.len()).ok()?;
        let mut exponent = i64::from(self.exponent);

        // The common case: no more digits than the precision, and nothing after
        // the decimal point.
        if length <= precision && exponent >= 0 {
            return Some(unsigned_value(&self.digits, length, false, exponent, max)? * sign);
        }

        // `checkIntegerDigits` (`NumberStringClass.cpp:937`). Round to the
        // precision, then require every surviving decimal to be a zero, or a
        // nine when the rounding carried.
        let mut carry = false;
        if length > precision {
            exponent += length - precision;
            length = precision;
            if self.digits[digits] >= 5 {
                carry = true;
            }
        }
        if exponent < 0 {
            let mut decimal_pos = -exponent;
            let mut compare = 0u8;
            if carry {
                // A carry adds one to the right-most digit, so a decimal
                // position beyond the digits means at least one padding zero,
                // and no carry can turn that into an integer.
                if decimal_pos > length {
                    return None;
                }
                compare = 9;
            }
            let data: &[u8] = if decimal_pos >= length {
                // The decimal point sits left of every digit, so all of them are
                // decimals.
                decimal_pos = length;
                &self.digits
            } else {
                &self.digits[usize::try_from(length + exponent).ok()?..]
            };
            for &digit in data.iter().take(usize::try_from(decimal_pos).ok()?) {
                if digit != compare {
                    return None;
                }
            }
        }

        // The point now sits left of the first digit, so the value is whatever
        // the carry contributed and nothing else. The C++ does NOT apply the
        // sign here, and that is reproduced rather than corrected: `numberValue`
        // returns `carry ? 1 : 0` with no `* numberSign`. It is unobservable
        // through the only caller that can reach it, because a numeric `TRACE`
        // is rejected at RUN time with error 24.901, "Numeric TRACE requests are
        // valid only from interactive debugging", whatever value the parse
        // produced.
        if -exponent >= length {
            return Some(i64::from(carry));
        }

        let converted = if exponent < 0 {
            unsigned_value(&self.digits, length + exponent, carry, 0, max)?
        } else {
            unsigned_value(&self.digits, length, carry, exponent, max)?
        };
        Some(converted * sign)
    }

    /// Rounds to at most `digits` significant digits, half-up.
    pub fn round_to(&self, digits: u64) -> Cow<'_, Self> {
        match self.round_change(digits) {
            None => Cow::Borrowed(self),
            Some(rounded) => Cow::Owned(rounded),
        }
    }

    /// [`round_to`] for a value the caller already owns, where "unchanged"
    /// can hand the value straight back instead of copying it.
    pub fn into_round(self, digits: u64) -> Self {
        self.round_change(digits).unwrap_or(self)
    }

    /// The rounded value, or `None` when rounding would return `self`
    /// unchanged. Both public spellings above are this plus an ownership
    /// decision, so the rule lives in one place.
    fn round_change(&self, digits: u64) -> Option<Self> {
        // Saturated, not truncated: a bare `digits` past usize can only mean
        // "keep everything", which the length test below then decides.
        let keep = usize::try_from(digits).unwrap_or(usize::MAX);
        if keep == 0 || self.digits.len() <= keep || self.is_zero() {
            return None;
        }
        let dropped = self.digits.len() - keep;
        let mut kept = Digits::from_slice(&self.digits[..keep]);
        let mut exponent = self.exponent + dropped as i32;

        if self.digits[keep] >= 5 {
            // Propagate the carry; an all-nines mantissa grows a digit and
            // sheds the one it no longer has room for.
            let mut i = keep;
            loop {
                if i == 0 {
                    kept.insert_front(1);
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
        Some(Self::assemble(self.negative, kept, exponent))
    }

    /// Renders the number as the interpreter would at this `DIGITS` setting.
    pub fn format(&self, digits: u64) -> String {
        let n = self.round_to(digits);
        if n.is_zero() {
            return "0".to_string();
        }
        let adjusted = n.adjusted_exponent();
        let sign = if n.negative { "-" } else { "" };
        let d: String = n.digits.iter().map(|x| (b'0' + x) as char).collect();

        // Compared in i64, with `digits` *saturated* into it rather than
        // narrowed: `digits` is a bare u64 here (not bounded by `Settings`,
        // which is the caller most external code goes through), and both an
        // i32 narrowing (the original defect, wrong past 2^31) and a u64 ->
        // i64 `as` cast (wrong past 2^63) silently pick the wrong display
        // form in release. Saturation is exact: the exponents this is
        // compared against stay within +/-`MAX_EXPONENT`, so i64::MAX
        // decides every comparison the true value would. The doubling then
        // saturates too, for the same reason.
        let digits = i64::try_from(digits).unwrap_or(i64::MAX);
        let low_threshold = digits.saturating_mul(2).saturating_add(1);
        if adjusted as i64 >= digits || n.exponent as i64 <= -low_threshold {
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
