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
//!
//! Constraints and error numbers were measured against ooRexx 5.3.0; see
//! `rust/corpus/num/settings.rex`.

use crate::Number;

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Form {
    Scientific,
    Engineering,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum SettingsError {
    /// `DIGITS` was not a positive whole number. Error 26.005; `additional()`
    /// is `[found]`. Confirmed with `NUMERIC DIGITS abc`.
    DigitsNotWhole { found: String },
    /// `FUZZ` was negative, or not a whole number -- the same code as
    /// `DigitsNotWhole` (26) but a different sub-message (26.006 vs 26.005),
    /// which is exactly why this is its own variant rather than one shared
    /// `NotWholeNumber` with no way to tell the two apart. `additional()` is
    /// `[found]`. Confirmed with `NUMERIC FUZZ -1`.
    FuzzNotWhole { found: String },
    /// `FUZZ` would not be strictly less than `DIGITS`. Raised whichever of
    /// the two is the one being set. Error 33.001; `additional()` is
    /// `[digits, fuzz]` -- the pending pair the setting would end up with
    /// (the just-rejected candidate for whichever is being set, the
    /// unchanged stored value for the other), not necessarily either's
    /// current field on `self`.
    FuzzNotBelowDigits { digits: u32, fuzz: u32 },
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

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Settings {
    digits: u32,
    fuzz: u32,
    form: Form,
}

impl Default for Settings {
    fn default() -> Self {
        Settings { digits: crate::DEFAULT_DIGITS, fuzz: 0, form: Form::Scientific }
    }
}

/// Accepts anything that is a whole number, including exponential spellings:
/// `NUMERIC DIGITS 1e3` sets 1000. Rejects fractions and non-numbers.
fn whole_number(text: &str) -> Option<i64> {
    let n = Number::parse(text)?;
    // Format with far more digits than any settings value could need, so the
    // check sees the number itself rather than a rounded rendering of it.
    const NO_ROUNDING: u32 = 1_000;
    let plain = n.format(NO_ROUNDING);
    if plain.contains(['.', 'E']) {
        return None;
    }
    plain.parse::<i64>().ok()
}

impl Settings {
    pub fn digits(&self) -> u32 {
        self.digits
    }

    pub fn fuzz(&self) -> u32 {
        self.fuzz
    }

    pub fn form(&self) -> Form {
        self.form
    }

    pub fn set_digits_str(&mut self, text: &str) -> Result<(), SettingsError> {
        // `found` is the raw text the caller passed, unmodified -- confirmed
        // with a lowercase variable (`x = "abc"`) that the interpreter
        // echoes back without uppercasing it.
        let not_whole = || SettingsError::DigitsNotWhole { found: text.to_string() };
        let value = whole_number(text).ok_or_else(not_whole)?;
        if value < 1 || value > crate::MAX_EXPONENT as i64 {
            return Err(not_whole());
        }
        let value = u32::try_from(value).map_err(|_| not_whole())?;
        if value <= self.fuzz {
            // The pending DIGITS/FUZZ pair, not necessarily either's stored
            // value: the candidate here (not yet committed to `self.digits`)
            // and the unchanged `self.fuzz` -- confirmed with `NUMERIC FUZZ
            // 5` then `NUMERIC DIGITS 3`, which reports "(\"3\") ...
            // (\"5\")".
            return Err(SettingsError::FuzzNotBelowDigits { digits: value, fuzz: self.fuzz });
        }
        self.digits = value;
        Ok(())
    }

    pub fn set_fuzz_str(&mut self, text: &str) -> Result<(), SettingsError> {
        let not_whole = || SettingsError::FuzzNotWhole { found: text.to_string() };
        let value = whole_number(text).ok_or_else(not_whole)?;
        if value < 0 {
            return Err(not_whole());
        }
        let value = u32::try_from(value).map_err(|_| not_whole())?;
        if value >= self.digits {
            // Same substitution rule as `set_digits_str`, mirrored: the
            // unchanged `self.digits` and the rejected candidate fuzz --
            // confirmed with `NUMERIC DIGITS 5` then `NUMERIC FUZZ 10`,
            // which reports "(\"5\") ... (\"10\")".
            return Err(SettingsError::FuzzNotBelowDigits { digits: self.digits, fuzz: value });
        }
        self.fuzz = value;
        Ok(())
    }

    pub fn set_form_str(&mut self, text: &str) -> Result<(), SettingsError> {
        // The interpreter accepts any unambiguous case; these are the only
        // two spellings it takes.
        self.form = match text.to_ascii_uppercase().as_str() {
            "SCIENTIFIC" => Form::Scientific,
            "ENGINEERING" => Form::Engineering,
            // `found` substitutes the raw text too, same rule as DIGITS/FUZZ.
            _ => return Err(SettingsError::InvalidForm { found: text.to_string() }),
        };
        Ok(())
    }
}
