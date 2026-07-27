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

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum SettingsError {
    /// Not a positive whole number (`DIGITS`), or negative (`FUZZ`).
    NotWholeNumber,
    /// `FUZZ` would not be strictly less than `DIGITS`. Raised whichever of
    /// the two is the one being set.
    FuzzNotBelowDigits,
    /// A `FORM` that is neither `SCIENTIFIC` nor `ENGINEERING`.
    InvalidForm,
}

impl SettingsError {
    /// The interpreter's error number. These are part of the contract, not an
    /// implementation detail -- programs trap on them.
    pub fn code(self) -> u16 {
        match self {
            SettingsError::InvalidForm => 25,
            SettingsError::NotWholeNumber => 26,
            SettingsError::FuzzNotBelowDigits => 33,
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
        let value = whole_number(text).ok_or(SettingsError::NotWholeNumber)?;
        if value < 1 || value > crate::MAX_EXPONENT as i64 {
            return Err(SettingsError::NotWholeNumber);
        }
        let value = u32::try_from(value).map_err(|_| SettingsError::NotWholeNumber)?;
        if value <= self.fuzz {
            return Err(SettingsError::FuzzNotBelowDigits);
        }
        self.digits = value;
        Ok(())
    }

    pub fn set_fuzz_str(&mut self, text: &str) -> Result<(), SettingsError> {
        let value = whole_number(text).ok_or(SettingsError::NotWholeNumber)?;
        if value < 0 {
            return Err(SettingsError::NotWholeNumber);
        }
        let value = u32::try_from(value).map_err(|_| SettingsError::NotWholeNumber)?;
        if value >= self.digits {
            return Err(SettingsError::FuzzNotBelowDigits);
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
            _ => return Err(SettingsError::InvalidForm),
        };
        Ok(())
    }
}
