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

use crate::muldiv::strip_leading;
use crate::{ArithError, MAX_EXPONENT, Number};

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

        // Square and multiply, high bit first, exactly as the interpreter
        // sequences it: the accumulator starts as the base itself, which
        // consumes the power's leading 1-bit, and each remaining bit squares
        // the accumulator and multiplies the base back in when set. The
        // order matters: it changes where the intermediate roundings fall,
        // and the reciprocal below exposes the accumulator's last working
        // digits, not just the rounded result.
        let mut acc = left.clone();
        let top = 63 - power.leading_zeros();
        for i in (0..top).rev() {
            acc = acc.mul(&acc, work)?;
            if (power >> i) & 1 == 1 {
                acc = acc.mul(&left, work)?;
            }
        }

        if negative_power {
            // The reciprocal has its own division, not the general one: it
            // neither rounds nor range-checks its quotient. Both are
            // observable -- the only rounding the reciprocal ever sees is the
            // final one below, where going through `div` would round twice
            // and flip a knife-edge last digit; and a reciprocal near the
            // exponent limit is out of range at working precision until that
            // same rounding brings it back.
            acc = divide_power(&acc, work);
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

/// The reciprocal `1 / accum` for a negative power, ported from
/// `NumberString::dividePower` (`NumberStringMath2.cpp:1059`).
///
/// The interpreter keeps this separate from its general division, and the
/// differences are observable: it long-divides 1 by the accumulator to at
/// most `digits + 1` quotient digits and stops there, with no rounding, no
/// trailing-zero stripping and no range check of its own -- all of that is
/// left to `pow`'s tail.
fn divide_power(accum: &Number, digits: u32) -> Number {
    debug_assert!(!accum.is_zero(), "pow never inverts a zero accumulator");
    let divisor = &accum.digits;
    let n = divisor.len();

    // The dividend is a 1 padded with zeros to the divisor's length.
    let mut left: Vec<u8> = vec![0; n];
    left[0] = 1;

    // The expected exponent of the result's last digit; every extension of
    // the dividend below moves it down one. Carried wide because the C++
    // works in a 64-bit wholenumber_t.
    let mut calc_exp = -(accum.exponent as i64) - n as i64 + 1;

    // Digit guesses divide by the divisor's first two digits plus one, so a
    // guess is either correct or errs low, never high.
    let mut div_char = divisor[0] as i32 * 10;
    if n > 1 {
        div_char += divisor[1] as i32;
    }
    div_char += 1;

    let mut result: Vec<u8> = Vec::new();
    let mut this_digit: i32 = 0;

    // The outer loop yields one quotient digit per pass; the inner loop
    // builds that digit by accumulating guesses until the remainder drops
    // below the divisor.
    'outer: loop {
        loop {
            let multiplier;
            if left.len() == n {
                // Equal lengths: a direct comparison tells us where we are.
                match left[..].cmp(&divisor[..]) {
                    // The remainder is smaller: this digit is complete.
                    std::cmp::Ordering::Less => break,
                    // Exactly equal: the current digit is one too small.
                    // Adjust it and the entire division is done.
                    std::cmp::Ordering::Equal => {
                        result.push((this_digit + 1) as u8);
                        break 'outer;
                    }
                    std::cmp::Ordering::Greater => multiplier = left[0] as i32,
                }
            } else if left.len() > n {
                // The remainder is longer, so it has at least two digits.
                multiplier = left[0] as i32 * 10 + left[1] as i32;
            } else {
                break;
            }

            // A zero guess gets wrapped to 1.
            let m = (multiplier * 10 / div_char).max(1);
            this_digit += m;
            subtract_multiple(&mut left, divisor, m);
            strip_leading(&mut left);
        }

        // A non-zero digit always joins the result; zeros only follow a
        // previous non-zero.
        if !result.is_empty() || this_digit != 0 {
            result.push(this_digit as u8);
            this_digit = 0;
            // Done once the remainder is zero or the result has grown to
            // digits + 1, leaving one digit for the caller's rounding.
            if left[0] == 0 || result.len() > digits as usize {
                break;
            }
        }
        // Reduced to exactly zero before the first significant digit.
        if left.len() == 1 && left[0] == 0 {
            break;
        }
        calc_exp -= 1;
        left.push(0);
    }

    // An exponent beyond i32 cannot be in range anyway; saturate and let the
    // caller's range check reject it.
    let exponent = calc_exp.clamp(i32::MIN as i64, i32::MAX as i64) as i32;
    Number { negative: accum.negative, digits: result, exponent }
}

/// `left -= m * divisor`, aligned at the low-order ends. The guess `m` is
/// correct or low, never high, so the result cannot go negative. Ported from
/// `NumberString::subtractDivisor` (`NumberStringMath2.cpp:224`).
fn subtract_multiple(left: &mut [u8], divisor: &[u8], m: i32) {
    let mut carry: i32 = 0;
    for i in 0..left.len() {
        let pos = left.len() - 1 - i;
        let sub = divisor.len().checked_sub(i + 1).map_or(0, |k| divisor[k] as i32 * m);
        let mut v = carry + left[pos] as i32 - sub;
        if v < 0 {
            // A single digit product can leave a deficit as large as 81, so
            // the borrow out can span two positions.
            v += 100;
            carry = v / 10 - 10;
            v %= 10;
        } else {
            carry = 0;
        }
        left[pos] = v as u8;
    }
    debug_assert_eq!(carry, 0, "an under-guess never drives the remainder negative");
}
