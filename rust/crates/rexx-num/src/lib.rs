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
pub use compare::{CompareOp, compare, compare_bytes, compare_decoded};
pub use muldiv::DivOp;
mod settings;
pub use settings::{Form, Settings, SettingsError};
mod format;
pub use format::FormatError;

/// Rexx's default `NUMERIC DIGITS`.
///
/// `u64` like every `digits` parameter in this crate: the legal range for a
/// DIGITS setting runs to `Numerics::MAX_WHOLENUMBER` (10^18 - 1 on 64-bit,
/// see `settings.rs`), which no narrower width holds.
pub const DEFAULT_DIGITS: u64 = 9;

/// The largest adjusted exponent a Rexx number may have. Beyond this a
/// literal will not convert (error 41) and an arithmetic result overflows
/// (error 42). `Numerics.hpp:113`.
pub const MAX_EXPONENT: i32 = 999_999_999;
pub const MIN_EXPONENT: i32 = -999_999_999;

/// The precision an argument is converted to a machine integer under, and the
/// width of that integer in decimal digits.
///
/// `Numerics::ARGUMENT_DIGITS` (`Numerics.hpp:90`), 18 on a 64-bit build and 9
/// on a 32-bit one. The platform dependence is observable and so is reproduced:
/// measured on a 64-bit build, `::OPTIONS DIGITS 123456789012345678` is rc 0
/// and `1234567890123456789` is Error 26.5, so the boundary sits at eighteen.
pub const ARGUMENT_DIGITS: usize = 18;

/// What arithmetic can fail with, carrying the interpreter's error numbers.
///
/// Each variant carries its substitution *values*, typed naturally, rather
/// than pre-rendered text: `message()` renders from the generated table on
/// demand, and `additional()` exposes those same values in the interpreter's
/// own order -- what `condition('o')~additional` would return for this
/// failure. That is not a style choice: a Rexx program that reads
/// `condition('o')~additional` directly needs the raw values back, and they
/// cannot be recovered from spliced text once it has been joined into a
/// sentence.
///
/// Originally two of these (`Overflow`, `NotWholeNumber`) were bare unit
/// variants covering several distinct C++ sub-messages at once, because
/// `muldiv.rs`/`pow.rs` were off limits when the message table was first
/// wired up. Now every raise site has its own variant -- see `message`'s
/// doc comment for the one case (`PowerOverflow`/`PowerExponentNotWhole`)
/// whose *values* still cannot be made byte-exact, and why.
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
    ///
    /// Currently raised only by `muldiv.rs`'s `div`, whose reservation site
    /// documents the interpreter mechanics this mirrors. `+`, `-`, `*` and
    /// `**` allocate by DIGITS in the interpreter too and do NOT raise this
    /// yet; that gap is a recorded deviation, see `phase-2-gate.md`.
    SystemResources,
}

impl ArithError {
    /// The major number a trapped Rexx program sees in `RC` (`condition('o')
    /// ~code` is this pair's major half). For the *sub*-number too -- what
    /// distinguishes 42.3 from 42.901 and 42.902, all of which are major 42
    /// -- see [`ArithError::sub_code`], which also returns this same major
    /// alongside it.
    ///
    /// Kept even though `sub_code` alone could answer both: this predates
    /// `sub_code`, several callers only ever want the major (`bin/muldiv
    /// .rs`/`bin/addsub.rs`'s harnesses render it as `<E{major}>` and never
    /// look at the sub-number at all), and forcing every one of them to
    /// destructure a pair for a value they discard would not make the code
    /// clearer. A caller that needs *both* should call `sub_code()` once
    /// rather than this and `sub_code()` separately -- calling both parses
    /// the same `match` twice for one answer.
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
    ///
    /// Public because a caller outside this crate needs exactly this and
    /// has no other way to get it: `rexx-exec`'s `From<ArithError> for
    /// Raised` (Task 7) has to carry the sub-number into the condition it
    /// raises, and the alternative -- hand-copying this `match` a second
    /// time in `rexx-exec` -- is precisely the divergent second copy this
    /// workspace's own rule against duplicating `rexx-num` logic (see
    /// `compare.rs`'s module doc for the same rule applied to comparison)
    /// exists to prevent. Kept as one function returning the pair, not
    /// split into a `sub()` beside `code()`: a caller wanting both calls
    /// this once and destructures it, which is both the natural way to use
    /// a `(major, sub)` fact that is always decided together (see this
    /// `match`, and `message`/`additional`'s own callers) and already this
    /// crate's own internal usage (`message`, below).
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
    ///
    /// `PowerOverflow`/`PowerExponentNotWhole` render `base`/`exponent` at
    /// their own full stored precision (`digits.len()` significant digits,
    /// via `full_precision`) rather than this crate's usual 9-digit
    /// default -- see `message`'s doc comment for why even that is not
    /// exact in general.
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
    ///
    /// `DivideByZero` (42.003), `IntegerDivideNotWhole` (26.011),
    /// `RemainderNotWhole` (26.012), and `ZeroToNegativePower` (42.903) take
    /// no substitution and are exact: confirmed with `1 / 0`, `123456 % 2`
    /// and `123456 // 2` at DIGITS 3, and `0 ** -1`. `Overflow`/`Underflow`
    /// are exact too -- confirmed with a mul overflow and a div underflow,
    /// and separately that &2 stays the literal `"9"` at DIGITS 9, 15, and
    /// 20 alike (`Numerics::DEFAULT_DIGITS` is a fixed C++ constant, not the
    /// active `NUMERIC DIGITS`).
    ///
    /// `PowerOverflow` (42.001, "...detected at: \"BASE**EXP\".") and
    /// `PowerExponentNotWhole` (26.008, "...found \"EXP\".") substitute the
    /// base and/or exponent **as originally written in the Rexx source**,
    /// not this crate's canonical rendering of them -- confirmed two ways:
    /// `1e10 ** 200000000000` reports the base as `"1E10"` (no `+`, the
    /// literal's own spelling) where this crate's `Number::format` would
    /// print `"1E+10"`; and `123.456789012345678 ** 999999999` at DIGITS 15
    /// reports the base at its full 18-digit original precision, not
    /// rounded to the active DIGITS. A `Number` has already discarded that
    /// original spelling by the time `pow.rs` sees it (`Number::parse`
    /// normalises sign/digits/exponent and nothing else survives), and
    /// nothing in this crate's scope threads the source text through, so
    /// `additional()` renders the base/exponent at full stored precision
    /// instead -- closer than the 9-digit default, but still provably wrong
    /// whenever the original had a leading zero, no `+` after `E`, a
    /// different exponent-marker case, or other spelling `Number` does not
    /// preserve. This is a limitation of the *value*, not just its text --
    /// there is no exact `Number`/text to hand back through `additional()`
    /// either, because the exact one was never kept.
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

/// `NumberString::createUnsignedValue` (`NumberStringClass.cpp:788`): the first
/// `length` digits, plus a carry, scaled by `10^exponent`.
///
/// Every overflow path returns `None`, which is the C++'s `false`. The width
/// pre-check is against `ARGUMENT_DIGITS` and not against the caller's
/// precision, which is what the C++ tests, and it is an early-out rather than
/// the deciding check: the `> max` test below rejects everything it would.
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
///
/// A single left-to-right pass over `text`, copying `subs` in without ever
/// re-scanning what was just copied. Doing this with `str::replace` per
/// placeholder instead (as an earlier version did) re-scans the whole
/// string on every call, so a substitution value that itself contains `&2`
/// gets mangled by the very next replacement: `substitute("… &1 … &2",
/// &["&2", "X"])` should read `"… &2 … X"`, but sequential replacement first
/// turns `&1` into the literal text `&2`, then turns *both* the original
/// `&2` and that just-inserted one into `X`. This crate's own arithmetic
/// substitutions now echo operand text back (`ArithError`'s `PowerOverflow`/
/// `PowerExponentNotWhole`), so this is not merely theoretical.
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
        Number {
            negative: false,
            digits: vec![0],
            exponent: 0,
        }
    }

    pub fn is_zero(&self) -> bool {
        self.digits.iter().all(|d| *d == 0)
    }

    /// Parses a Rexx number, or `None` if the string is not one.
    ///
    /// Accepts surrounding blanks, an optional sign (itself followed by
    /// optional blanks), digits with an optional decimal point, and an
    /// optional exponent. Rejects everything else -- notably a bare sign, a
    /// bare exponent marker, and hex literals, which are strings in Rexx
    /// rather than numbers.
    ///
    /// Blank handling mirrors `NumberString::parseNumber`, the state machine
    /// at `NumberStringClass.cpp:2586` validated by `NumberStringBuilder::
    /// finish` at `:2519`, which is the conversion that governs arithmetic
    /// operands. (`numberStringScan` at `:1264-1296` agrees on every
    /// blank-bearing shape, but it is a separate validity pre-check and so is
    /// the wrong reference for this port.) A blank is a space or a
    /// tab -- those two bytes, not Unicode whitespace -- and blanks are
    /// legal at either end and between a sign and its first digit, nowhere
    /// else. Confirmed against `build/bin/rexx`: `'+ 3'`, `'  +   3  '`,
    /// `'+ .5'` and tab variants all convert, while `'+ 3 e2'`, `'3 4'`,
    /// `'3e 2'` and a LF/VT/FF/CR anywhere are all error 41.
    pub fn parse(text: &str) -> Option<Self> {
        fn is_blank(byte: u8) -> bool {
            byte == b' ' || byte == b'\t'
        }
        let bytes = text.as_bytes();
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

    /// Fails with `Overflow`/`Underflow` when an arithmetic result has run
    /// outside the representable range, which the interpreter reports as
    /// error 42.
    ///
    /// Mirrors `NumberStringBase::checkOverflow` (`NumberStringClass.cpp:316`)
    /// exactly, including its priority (the upper bound is checked first).
    /// See `ArithError::Overflow`/`Underflow`'s doc comments for the &2 ==
    /// `"9"` detail and the adjusted-vs-raw exponent distinction between the
    /// two -- this only needs to pick the variant and hand it the one field
    /// each carries; the message rendering that used to happen here now
    /// happens on demand, in `ArithError::message`.
    pub(crate) fn check_range(self) -> Result<Self, ArithError> {
        if self.is_zero() || self.in_range() {
            return Ok(self);
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
    pub(crate) fn assemble(negative: bool, mut digits: Vec<u8>, exponent: i32) -> Self {
        if digits.iter().all(|d| *d == 0) {
            return Number::zero();
        }
        let lead = digits.iter().take_while(|d| **d == 0).count();
        digits.drain(..lead);
        Number {
            negative,
            digits,
            exponent,
        }
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
    /// The value as a machine integer under `digits` precision, or `None` if it
    /// has none.
    ///
    /// `NumberString::numberValue` (`NumberStringClass.cpp:588`), which is what
    /// `RexxString::requestNumber` forwards to. The number is ROUNDED to
    /// `digits` significant digits first and only then asked whether it is an
    /// integer, so a fraction can survive the conversion. That is the part a
    /// caller writing its own will get wrong, and one did. Measured through
    /// `TRACE`, whose fallback to an option string makes each step visible:
    ///
    /// * `trace 999999999.4` is rc 0, a skip count of 999999999, because ten
    ///   digits truncate to nine and the dropped `4` does not round up.
    /// * `trace "1.0000000001"` is rc 0 and means 1: eleven digits truncate to
    ///   nine and every surviving decimal is a zero.
    /// * `trace "0.9999999999"` is rc 0 and means 1: the dropped digit rounds
    ///   up, and a carry over all-nine decimals leaves 1.
    /// * `trace "999999999.6"` is Error 24.1, because that carry makes the value
    ///   ten digits wide.
    /// * `trace "99999999.6"` is 24.1, because nine digits do not exceed the
    ///   precision, so nothing is rounded and the `6` simply is not whole.
    ///
    /// The width limit is on the VALUE and not on the text. Measured:
    /// `trace 1e8` is rc 0 and `trace 1e9` is 24.1, because the second needs ten
    /// digits though its text holds two.
    ///
    /// `digits` is the caller's precision, and callers really do differ:
    /// `TRACE` converts under the parse-time `NUMERIC DIGITS` while
    /// `::OPTIONS DIGITS` converts under `ARGUMENT_DIGITS`.
    ///
    /// This does not duplicate `round_to`, though it rounds. `round_to` returns
    /// a `Number` and keeps arbitrary precision; this asks the separate question
    /// of whether the rounded value is an integer a machine word holds, and
    /// `checkIntegerDigits`'s carry rule is what makes the two different.
    ///
    /// That rule gives the digits two separate jobs, and it is easy to give them
    /// one. The FIRST DROPPED digit decides only whether there is a carry, and
    /// nothing else. The KEPT digits then decide whether the value is whole, and
    /// what they must equal depends on that carry: every surviving decimal must be
    /// a `0` normally but a `9` when the carry set, because only an all-nines tail
    /// can absorb the +1 and leave zeros. So the dropped digit never appears in
    /// the wholeness test and the kept digits never decide the carry. Measured,
    /// with identical kept digits and different dropped ones coming out opposite
    /// ways: `trace "0.9999999994"` is 24.1 and `trace "0.99999999999"` is rc 0.
    /// `tests/whole.rs` carries the rest of the pairs.
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

    /// because every operator needs it.
    ///
    /// `digits == 0` is a deliberate no-op sentinel, not an accident: there
    /// is no `NUMERIC DIGITS 0`, and the caller that produces a zero here --
    /// `compare` with `digits == fuzz`, whose working precision is their
    /// difference -- needs "no rounding at all" rather than "round to
    /// nothing". Reachable from the public `compare(a, b, d, d, op)` entry
    /// point, so the sentinel is load-bearing.
    pub fn round_to(&self, digits: u64) -> Self {
        // Saturated, not truncated: a bare `digits` past usize can only mean
        // "keep everything", which the length test below then decides.
        let keep = usize::try_from(digits).unwrap_or(usize::MAX);
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
