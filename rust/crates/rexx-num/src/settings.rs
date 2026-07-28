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
    /// Not a positive whole number (`DIGITS`), or negative (`FUZZ`). Carries
    /// the interpreter's exact message text -- DIGITS and FUZZ get
    /// different sub-messages for the same code (26.005 vs 26.006, confirmed
    /// by provoking `NUMERIC DIGITS abc` and `NUMERIC FUZZ -1`), which the
    /// shared variant name alone can't tell apart, so each raise site below
    /// renders its own.
    NotWholeNumber(String),
    /// `FUZZ` would not be strictly less than `DIGITS`. Raised whichever of
    /// the two is the one being set. Error 33.001.
    FuzzNotBelowDigits(String),
    /// A `FORM` that is neither `SCIENTIFIC` nor `ENGINEERING`. Error 25.011.
    InvalidForm(String),
}

impl SettingsError {
    /// The interpreter's error number. These are part of the contract, not an
    /// implementation detail -- programs trap on them.
    pub fn code(&self) -> u16 {
        match self {
            SettingsError::InvalidForm(_) => 25,
            SettingsError::NotWholeNumber(_) => 26,
            SettingsError::FuzzNotBelowDigits(_) => 33,
        }
    }

    /// The interpreter's exact message text, substitutions already filled.
    pub fn message(&self) -> &str {
        match self {
            SettingsError::NotWholeNumber(m)
            | SettingsError::FuzzNotBelowDigits(m)
            | SettingsError::InvalidForm(m) => m,
        }
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
        // 26.005 substitutes the raw text the caller passed, unmodified --
        // confirmed with a lowercase variable (`x = "abc"`) that the
        // interpreter echoes back without uppercasing it.
        let not_whole = || SettingsError::NotWholeNumber(crate::error_text(26, 5, &[text]));
        let value = whole_number(text).ok_or_else(not_whole)?;
        if value < 1 || value > crate::MAX_EXPONENT as i64 {
            return Err(not_whole());
        }
        let value = u32::try_from(value).map_err(|_| not_whole())?;
        if value <= self.fuzz {
            // 33.001 substitutes the DIGITS/FUZZ pair the setting would end
            // up with, not necessarily either's stored value: the candidate
            // here (not yet committed to `self.digits`) and the unchanged
            // `self.fuzz` -- confirmed with `NUMERIC FUZZ 5` then `NUMERIC
            // DIGITS 3`, which reports "(\"3\") ... (\"5\")".
            return Err(SettingsError::FuzzNotBelowDigits(crate::error_text(
                33,
                1,
                &[&value.to_string(), &self.fuzz.to_string()],
            )));
        }
        self.digits = value;
        Ok(())
    }

    pub fn set_fuzz_str(&mut self, text: &str) -> Result<(), SettingsError> {
        let not_whole = || SettingsError::NotWholeNumber(crate::error_text(26, 6, &[text]));
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
            return Err(SettingsError::FuzzNotBelowDigits(crate::error_text(
                33,
                1,
                &[&self.digits.to_string(), &value.to_string()],
            )));
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
            // 25.011 substitutes the raw text too, same rule as 26.005/.006.
            _ => return Err(SettingsError::InvalidForm(crate::error_text(25, 11, &[text]))),
        };
        Ok(())
    }
}
