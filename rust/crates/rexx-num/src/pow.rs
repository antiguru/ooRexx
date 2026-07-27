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

//! The `**` operator.
//!
//! Ported from `NumberString::power` (`NumberStringMath2.cpp:811`).

use crate::{ArithError, DivOp, MAX_EXPONENT, Number};

impl Number {
    /// The value one.
    pub fn one() -> Number {
        Number::parse("1").expect("1 is a number")
    }

    /// Interprets this number as a whole number expressible within `digits`
    /// significant digits, which is what `**` requires of its exponent.
    ///
    /// The value is **rounded to `digits` first**, and only then required to
    /// be whole. That is why `2 ** 2.5` is error 26 at DIGITS 9 but `8` at
    /// DIGITS 1, where 2.5 rounds to 3.
    ///
    /// `2 ** 1e10` is error 26 at DIGITS 9 -- ten thousand million needs
    /// eleven digits -- but succeeds at DIGITS 15, where it then fails with
    /// error 42 because the *result* is out of range.
    fn as_whole(&self, digits: u32) -> Option<i64> {
        let rounded = self.round_to(digits);
        let self_ = &rounded;
        if self_.is_zero() {
            return Some(0);
        }
        if self_.digits.len() as i32 + self_.exponent > digits as i32 {
            return None;
        }
        if self_.exponent < 0 {
            // Any digit past the decimal point makes it non-whole.
            let frac = (-self_.exponent) as usize;
            if frac >= self_.digits.len() || self_.digits[self_.digits.len() - frac..].iter().any(|d| *d != 0)
            {
                return None;
            }
        }
        let mut value: i64 = 0;
        for d in &self_.digits {
            value = value.checked_mul(10)?.checked_add(*d as i64)?;
        }
        for _ in 0..self_.exponent.max(0) {
            value = value.checked_mul(10)?;
        }
        for _ in 0..(-self_.exponent).max(0) {
            value /= 10;
        }
        Some(if self_.negative { -value } else { value })
    }

    pub fn pow(&self, exponent: &Number, digits: u32) -> Result<Number, ArithError> {
        let power = exponent.as_whole(digits).ok_or(ArithError::NotWholeNumber)?;
        let negative_power = power < 0;
        let power = power.unsigned_abs();

        let left = self.truncated_to(digits as usize + 1);

        if left.is_zero() {
            // Zero to a negative power is an underflow, not infinity.
            if negative_power {
                return Err(ArithError::Overflow);
            }
            // Rexx defines 0**0 as 1, though mathematically it is undefined.
            return Ok(if power == 0 { Number::one() } else { Number::zero() });
        }

        // The magnitude of the result is knowable up front, so a hopeless
        // computation is refused before it is attempted.
        let magnitude = (left.adjusted_exponent().unsigned_abs() as u64).saturating_mul(power);
        if magnitude > MAX_EXPONENT as u64 {
            return Err(ArithError::Overflow);
        }

        if power == 0 {
            return Ok(Number::one());
        }

        // Working precision is raised by the number of decimal digits in the
        // exponent, plus one, so the repeated squaring does not accumulate
        // visible error.
        let extra = power.to_string().len() as u32;
        let work = digits + extra + 1;

        // Square and multiply, low bit first. The order matters: it changes
        // where the intermediate roundings fall, and on knife-edge cases
        // that changes the last digit of the result.
        let mut acc = Number::one();
        let mut base = left.clone();
        let mut p = power;
        while p > 0 {
            if p & 1 == 1 {
                acc = acc.mul(&base, work)?;
            }
            p >>= 1;
            if p > 0 {
                base = base.mul(&base, work)?;
            }
        }

        if negative_power {
            // Carried at extra precision and left unchecked: the reciprocal
            // of a value near the exponent limit is itself out of range
            // until the final rounding brings it back.
            acc = Number::one().div_unchecked(&acc, work + 2, DivOp::Divide)?;
        }

        let mut result = acc.round_to(digits).check_range()?;
        // Trailing zeros come off, by the same rule division uses.
        while result.digits.len() > 1 && *result.digits.last().unwrap() == 0 {
            result.digits.pop();
            result.exponent += 1;
        }
        Ok(Number::assemble(result.negative, result.digits, result.exponent))
    }
}
