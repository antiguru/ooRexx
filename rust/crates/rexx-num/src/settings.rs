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

//! The `NUMERIC` settings, and the errors that changing them can raise.

use crate::Number;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Form {
    Scientific,
    Engineering,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum SettingsError {
    /// `DIGITS` was not a positive whole number *expressible within the
    /// DIGITS currently in force* -- not-a-number, zero, negative,
    /// fractional, and too-many-digits all land here alike. Error 26.005;
    /// `additional()` is `[found]`. Confirmed with `NUMERIC DIGITS abc`, and
    /// with `NUMERIC DIGITS 1000` at DIGITS 3.
    DigitsNotWhole { found: String },
    /// `FUZZ` was negative, not a whole number, or not expressible within
    /// the DIGITS currently in force -- the same code as `DigitsNotWhole`
    /// (26) but a different sub-message (26.006 vs 26.005), which is exactly
    /// why this is its own variant rather than one shared `NotWholeNumber`
    /// with no way to tell the two apart. `additional()` is `[found]`.
    /// Confirmed with `NUMERIC FUZZ -1`, and with `NUMERIC FUZZ 12345` at
    /// DIGITS 3 (26.006, *not* 33.001: the conversion fails before the
    /// fuzz-below-digits comparison is ever reached).
    FuzzNotWhole { found: String },
    /// `FUZZ` would not be strictly less than `DIGITS`. Raised whichever of
    /// the two is the one being set. Error 33.001; `additional()` is
    /// `[digits, fuzz]` -- the pending pair the setting would end up with
    /// (the just-rejected candidate for whichever is being set, the
    /// unchanged stored value for the other), not necessarily either's
    /// current field on `self`.
    FuzzNotBelowDigits { digits: u64, fuzz: u64 },
    /// A `FORM` that is neither `SCIENTIFIC` nor `ENGINEERING`. Error 25.011;
    /// `additional()` is `[found]`.
    InvalidForm { found: String },
}

impl SettingsError {
    /// The interpreter's error number. These are part of the contract, not an
    /// implementation detail -- programs trap on them.
    pub fn code(&self) -> u16 {
        match self {
            SettingsError::InvalidForm { .. } => 25,
            SettingsError::DigitsNotWhole { .. } | SettingsError::FuzzNotWhole { .. } => 26,
            SettingsError::FuzzNotBelowDigits { .. } => 33,
        }
    }

    /// The `(major, sub)` pair identifying this failure's exact table row --
    /// see `ArithError::sub_code`'s doc comment for why `code()` alone
    /// isn't enough.
    fn sub_code(&self) -> (u16, u16) {
        match self {
            SettingsError::InvalidForm { .. } => (25, 11),
            SettingsError::DigitsNotWhole { .. } => (26, 5),
            SettingsError::FuzzNotWhole { .. } => (26, 6),
            SettingsError::FuzzNotBelowDigits { .. } => (33, 1),
        }
    }

    /// The substitution values in the interpreter's own order -- what
    /// `condition('o')~additional` would return for this failure.
    pub fn additional(&self) -> Vec<String> {
        match self {
            SettingsError::DigitsNotWhole { found }
            | SettingsError::FuzzNotWhole { found }
            | SettingsError::InvalidForm { found } => vec![found.clone()],
            SettingsError::FuzzNotBelowDigits { digits, fuzz } => {
                vec![digits.to_string(), fuzz.to_string()]
            }
        }
    }

    /// The interpreter's message text, rendered from the generated table on
    /// demand.
    pub fn message(&self) -> String {
        let (major, sub) = self.sub_code();
        let subs = self.additional();
        let refs: Vec<&str> = subs.iter().map(String::as_str).collect();
        crate::error_text(major, sub, &refs)
    }
}

/// `Numerics::MAX_WHOLENUMBER` (`Numerics.hpp:86`): the largest value any
/// whole-number conversion can produce on 64-bit, and therefore the absolute
/// ceiling on `DIGITS` itself (confirmed: `NUMERIC DIGITS
/// 999999999999999999` succeeds from DIGITS 18, and 10^18 fails even from
/// DIGITS 30). Unrelated to `crate::MAX_EXPONENT`, which bounds a number's
/// *exponent* -- conflating the two was exactly the defect this rule
/// replaces.
const MAX_WHOLENUMBER: u64 = 999_999_999_999_999_999;

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Settings {
    digits: u64,
    fuzz: u64,
    form: Form,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            digits: crate::DEFAULT_DIGITS,
            fuzz: 0,
            form: Form::Scientific,
        }
    }
}

/// `Numerics::maxValueForDigits` (`Numerics.hpp:160`): the largest unsigned
/// whole number expressible in `digits` significant positions, capped at
/// `MAX_WHOLENUMBER`'s eighteen.
fn max_value_for_digits(digits: u64) -> u64 {
    if digits >= 18 {
        MAX_WHOLENUMBER
    } else {
        // `digits < 18`, so the narrowing is lossless and the power fits.
        10u64.pow(digits as u32) - 1
    }
}

/// Converts `text` the way the interpreter converts a NUMERIC DIGITS/FUZZ
/// operand: `requestUnsignedNumber(setting, number_digits())`
/// (`NumericInstruction.cpp:103`/`:139`), i.e. an unsigned whole number
/// **within the DIGITS setting currently in force** -- there is no fixed
/// cap. The value is rounded at that DIGITS boundary first (half-up on the
/// first dropped digit), every decimal digit that remains must reduce to
/// zero -- literal zeros, or all nines consumed by the carry -- and the
/// result must not exceed `max_value_for_digits(digits)`.
fn unsigned_whole_number(text: &str, digits: u64) -> Option<u64> {
    let n = Number::parse(text)?;
    if n.is_zero() {
        return Some(0);
    }
    if n.negative {
        return None;
    }

    let mut exponent = i64::from(n.exponent);
    let mut length = n.digits.len();
    let mut carry = false;
    // More stored digits than the DIGITS in force: the value is rounded at
    // that boundary, and the first dropped digit decides the carry.
    if length as u64 > digits {
        let keep = digits as usize; // < length, so lossless
        exponent += (length - keep) as i64;
        carry = n.digits[keep] >= 5;
        length = keep;
    }
    let kept = &n.digits[..length];
    let max_value = max_value_for_digits(digits);

    if exponent >= 0 {
        return create_unsigned_value(kept, carry, exponent, max_value);
    }

    // Some kept digits sit below the decimal point. They must all reduce to
    // zero for the value to be whole.
    let decimals = (-exponent) as u64;
    if carry && decimals > length as u64 {
        // The carry lands below the lowest stored digit; the padding zeros
        // in between can never ripple it up to the integer part.
        return None;
    }
    let required = if carry { 9 } else { 0 };
    let checked = decimals.min(length as u64) as usize; // <= length: lossless
    if kept[length - checked..].iter().any(|d| *d != required) {
        return None;
    }
    if decimals >= length as u64 {
        // Nothing left of the point: the value truncates to 0, or to 1 when
        // the all-nines carry ripples all the way up.
        return Some(u64::from(carry));
    }
    create_unsigned_value(&kept[..length - decimals as usize], carry, 0, max_value)
}

/// Mirrors `NumberString::createUnsignedValue` (`NumberStringClass.cpp:788`):
/// accumulate the integer digits, add the rounding carry, scale by the
/// exponent's trailing zeros, and reject anything past `max_value`. The
/// checked u64 arithmetic stands in for the C++ wrap tests -- and bounds the
/// scaling loop the same way, since any nonzero value overflows u64 within
/// twenty scalings however large `exponent` is.
fn create_unsigned_value(digits: &[u8], carry: bool, exponent: i64, max_value: u64) -> Option<u64> {
    let mut value: u64 = 0;
    for &d in digits {
        value = value.checked_mul(10)?.checked_add(u64::from(d))?;
    }
    if carry {
        value = value.checked_add(1)?;
    }
    for _ in 0..exponent {
        value = value.checked_mul(10)?;
    }
    (value <= max_value).then_some(value)
}

impl Settings {
    pub fn digits(&self) -> u64 {
        self.digits
    }

    pub fn fuzz(&self) -> u64 {
        self.fuzz
    }

    pub fn form(&self) -> Form {
        self.form
    }

    pub fn set_digits_str(&mut self, text: &str) -> Result<(), SettingsError> {
        // `found` is the raw text the caller passed, unmodified -- confirmed
        // with a lowercase variable (`x = "abc"`) that the interpreter
        // echoes back without uppercasing it.
        let not_whole = || SettingsError::DigitsNotWhole {
            found: text.to_string(),
        };
        // The candidate is judged against the DIGITS *currently in force*:
        // `numeric digits 1000000000` is 26.005 at DIGITS 9 and legal at
        // DIGITS 10. There is no fixed cap short of `MAX_WHOLENUMBER`.
        let value = unsigned_whole_number(text, self.digits).ok_or_else(not_whole)?;
        self.set_digits_checked(value, not_whole)
    }

    /// `::OPTIONS DIGITS n`, whose operand the directive parser has already
    /// converted with `Numerics::ARGUMENT_DIGITS` rather than with the
    /// setting in force.
    pub fn set_digits(&mut self, value: u64) -> Result<(), SettingsError> {
        self.set_digits_checked(value, || SettingsError::DigitsNotWhole {
            found: value.to_string(),
        })
    }

    /// `NUMERIC DIGITS` with no operand: the package default, and it makes
    /// the same FUZZ check the operand form makes.
    pub fn reset_digits(&mut self, default: u64) -> Result<(), SettingsError> {
        if default <= self.fuzz {
            return Err(SettingsError::FuzzNotBelowDigits {
                digits: default,
                fuzz: self.fuzz,
            });
        }
        self.digits = default;
        Ok(())
    }

    /// The half the two share. `not_whole` is a closure so that the text form
    /// reports the bytes the program wrote -- `numeric digits 0.0` names
    /// `"0.0"` and not the `0` it converted to.
    fn set_digits_checked(
        &mut self,
        value: u64,
        not_whole: impl Fn() -> SettingsError,
    ) -> Result<(), SettingsError> {
        if value < 1 {
            return Err(not_whole());
        }
        if value <= self.fuzz {
            // The pending DIGITS/FUZZ pair, not necessarily either's stored
            // value: the candidate here (not yet committed to `self.digits`)
            // and the unchanged `self.fuzz` -- confirmed with `NUMERIC FUZZ
            // 5` then `NUMERIC DIGITS 3`, which reports "(\"3\") ...
            // (\"5\")".
            return Err(SettingsError::FuzzNotBelowDigits {
                digits: value,
                fuzz: self.fuzz,
            });
        }
        self.digits = value;
        Ok(())
    }

    pub fn set_fuzz_str(&mut self, text: &str) -> Result<(), SettingsError> {
        let not_whole = || SettingsError::FuzzNotWhole {
            found: text.to_string(),
        };
        // Same conversion as DIGITS, against the same in-force DIGITS
        // setting -- so `numeric fuzz 12345` at DIGITS 3 is 26.006, not
        // 33.001: the conversion fails before the comparison below is ever
        // reached. (Negatives fail the conversion too; there is no separate
        // sign check.)
        let value = unsigned_whole_number(text, self.digits).ok_or_else(not_whole)?;
        self.set_fuzz_checked(value)
    }

    /// `::OPTIONS FUZZ n`, the sibling of [`Settings::set_digits`] and
    /// converted the same way (`DirectiveParser.cpp:1046`).
    pub fn set_fuzz(&mut self, value: u64) -> Result<(), SettingsError> {
        self.set_fuzz_checked(value)
    }

    /// `NUMERIC FUZZ` with no operand: the package default, checked against
    /// the DIGITS in force.
    pub fn reset_fuzz(&mut self, default: u64) -> Result<(), SettingsError> {
        if default >= self.digits {
            return Err(SettingsError::FuzzNotBelowDigits {
                digits: self.digits,
                fuzz: default,
            });
        }
        self.fuzz = default;
        Ok(())
    }

    /// The half the two share. No `not_whole` closure here: the check below
    /// reports numbers rather than the text it was given.
    fn set_fuzz_checked(&mut self, value: u64) -> Result<(), SettingsError> {
        if value >= self.digits {
            // Same substitution rule as `set_digits_str`, mirrored: the
            // unchanged `self.digits` and the rejected candidate fuzz --
            // confirmed with `NUMERIC DIGITS 5` then `NUMERIC FUZZ 10`,
            // which reports "(\"5\") ... (\"10\")".
            return Err(SettingsError::FuzzNotBelowDigits {
                digits: self.digits,
                fuzz: value,
            });
        }
        self.fuzz = value;
        Ok(())
    }

    pub fn set_form_str(&mut self, text: &str) -> Result<(), SettingsError> {
        // Only the two exact uppercase spellings: the runtime VALUE path --
        // which a `&str` setter models -- does no uppercasing, no trimming,
        // and no abbreviation. `engineering`, `Engineering`, `ENG`, and
        // `' ENGINEERING'` are all 25.011, confirmed against
        // `build/bin/rexx`. The *keyword* form written in source only looks
        // case-insensitive because the tokenizer uppercases the token
        // before the instruction ever sees it.
        self.form = match text {
            "SCIENTIFIC" => Form::Scientific,
            "ENGINEERING" => Form::Engineering,
            // `found` substitutes the raw text too, same rule as DIGITS/FUZZ.
            _ => {
                return Err(SettingsError::InvalidForm {
                    found: text.to_string(),
                });
            }
        };
        Ok(())
    }

    /// `::OPTIONS FORM ENGINEERING`/`SCIENTIFIC` and `NUMERIC FORM`'s reset
    /// to the package default, both of which name the setting rather than
    /// spell it.
    pub fn set_form(&mut self, form: Form) {
        self.form = form;
    }
}

#[cfg(test)]
mod tests {
    use super::{Form, Settings, SettingsError};

    /// The reset refuses exactly what the operand form refuses, and names the
    /// same two values -- the default it wanted to store and the FUZZ in force.
    #[test]
    fn the_digits_reset_refuses_what_the_operand_form_refuses() {
        let mut settings = Settings::default();
        settings
            .set_digits_str("30")
            .expect("30 is legal at DIGITS 9");
        settings.set_fuzz_str("20").expect("20 is below DIGITS 30");

        let mut spelled_out = settings.clone();
        assert_eq!(
            spelled_out.set_digits_str("9"),
            Err(SettingsError::FuzzNotBelowDigits {
                digits: 9,
                fuzz: 20
            }),
            "the operand form still refuses the value the reset stores"
        );
        assert_eq!(spelled_out.digits(), 30, "a refused set changes nothing");

        assert_eq!(
            settings.reset_digits(crate::DEFAULT_DIGITS),
            Err(SettingsError::FuzzNotBelowDigits {
                digits: 9,
                fuzz: 20
            }),
            "the reset makes the same check, naming the default it could not store"
        );
        assert_eq!(
            (settings.digits(), settings.fuzz()),
            (30, 20),
            "a refused reset changes nothing"
        );
    }

    /// Each no-operand form writes its own setting and leaves the other two.
    /// Measured across a sweep of starting DIGITS, FUZZ and FORM: `numeric
    /// digits` writes only DIGITS, `numeric fuzz` only FUZZ, and `numeric
    /// form` only FORM.
    #[test]
    fn each_reset_leaves_the_settings_it_does_not_own() {
        let start = || {
            let mut settings = Settings::default();
            settings
                .set_digits_str("30")
                .expect("30 is legal at DIGITS 9");
            settings.set_fuzz_str("20").expect("20 is below DIGITS 30");
            settings
                .set_form_str("ENGINEERING")
                .expect("a spelling the setter accepts");
            settings
        };

        // DIGITS' reset cannot run from this state -- the default sits under
        // the FUZZ in force -- so the pair it would have to leave alone is
        // measured from a state where it succeeds.
        let mut digits_reset = start();
        digits_reset
            .set_fuzz_str("5")
            .expect("5 is below DIGITS 30");
        digits_reset
            .reset_digits(crate::DEFAULT_DIGITS)
            .expect("9 exceeds a FUZZ of 5");
        assert_eq!(
            (
                digits_reset.digits(),
                digits_reset.fuzz(),
                digits_reset.form()
            ),
            (9, 5, Form::Engineering)
        );

        let mut fuzz_reset = start();
        fuzz_reset.reset_fuzz(0).expect("0 is below any DIGITS");
        assert_eq!(
            (fuzz_reset.digits(), fuzz_reset.fuzz(), fuzz_reset.form()),
            (30, 0, Form::Engineering)
        );

        let mut form_reset = start();
        form_reset
            .set_form_str("SCIENTIFIC")
            .expect("a spelling the setter accepts");
        assert_eq!(
            (form_reset.digits(), form_reset.fuzz(), form_reset.form()),
            (30, 20, Form::Scientific)
        );
    }
}
