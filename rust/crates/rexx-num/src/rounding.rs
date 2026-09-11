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

//! `FLOOR`, `CEILING` and `ROUND`, and the wholeness test `MODULO` gates on.

use crate::{Digits, Number};

/// Which neighbouring integer a value between two of them is taken to.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Toward {
    Lower,
    Higher,
    Nearest,
}

impl Number {
    /// Implements `~FLOOR`: the largest integer not exceeding this value.
    pub fn floor(&self, digits: u64) -> String {
        self.to_integer(digits, Toward::Lower).trunc(digits, 0)
    }

    /// Implements `~CEILING`: the smallest integer not below this value.
    pub fn ceiling(&self, digits: u64) -> String {
        self.to_integer(digits, Toward::Higher).trunc(digits, 0)
    }

    /// Implements `~ROUND`: the nearest integer, halves away from zero.
    pub fn round(&self, digits: u64) -> String {
        self.to_integer(digits, Toward::Nearest).trunc(digits, 0)
    }

    /// Whether this value is a whole number at `digits` precision, the test
    /// `~MODULO` refuses its target on.
    pub fn is_integer(&self, digits: u64) -> bool {
        if self.is_zero() || self.exponent == 0 {
            return true;
        }
        let adjusted = self.digits.len() as i64 + i64::from(self.exponent);
        // Saturated rather than cast: a `digits` past i64 can only mean "no
        // bound", and a bare cast would wrap it into a negative one.
        if adjusted > i64::try_from(digits).unwrap_or(i64::MAX) || adjusted <= 0 {
            return false;
        }
        if self.exponent > 0 {
            return true;
        }
        self.digits[adjusted as usize..].iter().all(|d| *d == 0)
    }

    /// The neighbouring integer `toward` names, as a `Number` whose decimals
    /// are still present but no longer significant -- the caller's `trunc`
    /// discards them.
    fn to_integer(&self, digits: u64, toward: Toward) -> Number {
        let value = self.round_to(digits);
        if value.is_zero() {
            return Number::zero();
        }
        let integer_digits = value.digits.len() as i32 + value.exponent;
        let step = match toward {
            Toward::Lower => value.negative && value.has_nonzero_decimals(),
            Toward::Higher => !value.negative && value.has_nonzero_decimals(),
            // A value whose first decimal is not even in the digit string
            // has a zero there, so it can only round to zero.
            Toward::Nearest if value.exponent >= 0 => false,
            Toward::Nearest if integer_digits < 0 => return Number::zero(),
            Toward::Nearest => value.digits[integer_digits as usize] >= 5,
        };
        if !step {
            return value.into_owned();
        }
        if integer_digits <= 0 {
            return Number::from_i64(if value.negative { -1 } else { 1 });
        }
        let mut kept = Digits::from_slice(&value.digits[..integer_digits as usize]);
        let mut exponent = 0;
        let mut i = integer_digits as usize;
        loop {
            if i == 0 {
                // Every kept digit was a nine, so the carry runs off the top
                // and the value becomes a one followed by the same count of
                // zeros.
                kept.insert_front(1);
                kept.pop();
                exponent = 1;
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
        Number::assemble(value.negative, kept, exponent)
    }

    /// Whether any digit to the right of the decimal point is non-zero.
    fn has_nonzero_decimals(&self) -> bool {
        if self.exponent >= 0 {
            return false;
        }
        let decimals = self.digits.len().min(self.exponent.unsigned_abs() as usize);
        self.digits[self.digits.len() - decimals..]
            .iter()
            .any(|d| *d != 0)
    }
}
