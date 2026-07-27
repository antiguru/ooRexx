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

//! Addition and subtraction.
//!
//! Both align the operands on their least significant digit, work on plain
//! digit vectors, and then round to `DIGITS`.
//!
//! The subtle part is *when* leading zeros are stripped. A borrow out of the
//! top of a subtraction, or an absent carry out of an addition, leaves a
//! leading zero in the raw result, and the interpreter counts that zero
//! toward the result's digit count and rounds **before** normalising it away.
//! That is observable:
//!
//! ```text
//! DIGITS 9:   1e9 - 1  ->  1.00000000E+9    not 999999999
//! DIGITS 10:  1e9 - 1  ->  999999999
//! ```
//!
//! At `DIGITS 9` the raw result is `0999999999` -- ten digits -- so rounding
//! to nine discards a `9` and carries all the way up to `1000000000`. At
//! `DIGITS 10` the same ten digits fit and the exact answer survives.
//! Stripping the zero first, which is the obvious thing to do, gets every
//! such case wrong.

use crate::{ArithError, Number};

impl Number {
    /// Extends the digit vector downward so both operands share an exponent.
    fn aligned_to(&self, exponent: i32) -> Vec<u8> {
        let pad = (self.exponent - exponent).max(0) as usize;
        let mut digits = self.digits.clone();
        digits.extend(std::iter::repeat_n(0u8, pad));
        digits
    }

    pub fn add(&self, other: &Number, digits: u32) -> Result<Number, ArithError> {
        self.add_signed(other, false, digits)
    }

    pub fn sub(&self, other: &Number, digits: u32) -> Result<Number, ArithError> {
        self.add_signed(other, true, digits)
    }

    fn add_signed(
        &self,
        other: &Number,
        negate_other: bool,
        digits: u32,
    ) -> Result<Number, ArithError> {
        let left_negative = self.negative;
        let right_negative = other.negative != negate_other;

        // Operands longer than DIGITS are truncated to DIGITS + 1 working
        // digits first. (The interpreter also raises LOSTDIGITS here; that
        // condition belongs to Task 2.8.)
        let max_length = digits as usize + 1;
        let left = self.truncated_to(max_length);
        let right = other.truncated_to(max_length);

        let min_exp = left.exponent.min(right.exponent);
        let adjusted_left_exp = (left.exponent - min_exp) as usize;
        let adjusted_right_exp = (right.exponent - min_exp) as usize;
        let left_len = left.digits.len();
        let right_len = right.digits.len();
        let digits_usize = digits as usize;

        // Fast paths, ported from NumberString::addSub. Each returns one
        // operand essentially untouched, and they are not an optimisation:
        // going through the general path produces a leading zero that
        // rounding then turns into a different number. `0 + 123456789` is
        // 123456789, but computing it yields `0123456789`, which rounds to
        // 123456790.
        let fast = if left.is_zero() {
            Some((&right, right_negative))
        } else if right.is_zero() {
            Some((&left, left_negative))
        } else if adjusted_left_exp + left_len > right_len + digits_usize {
            // The right number is too far below the left to reach any digit
            // the result will keep.
            Some((&left, left_negative))
        } else if adjusted_right_exp + right_len > left_len + digits_usize {
            Some((&right, right_negative))
        } else {
            None
        };
        if let Some((value, negative)) = fast {
            let mut result = value.round_to(digits);
            result.negative = negative && !result.is_zero();
            return result.check_range();
        }

        // Alignment adjustment, ported from addSub. When the two operands
        // together span more than the working precision, the less significant
        // one is shortened from its low end so the pair fits. Skipping this
        // is not merely imprecise -- it changes which digit the rounding
        // decision sees, so `12.3400 - 9.999999995` at DIGITS 3 comes out as
        // 2.3 rather than 2.4.
        let mut left_digits = left.digits.clone();
        let mut right_digits = right.digits.clone();
        let mut left_exp = left.exponent;
        let mut right_exp = right.exponent;
        let adjusted_left_exp = adjusted_left_exp as i64;
        let adjusted_right_exp = adjusted_right_exp as i64;
        let max_len = max_length as i64;

        let adjusted_left_digits = left_digits.len() as i64 + adjusted_left_exp - max_len;
        let adjusted_right_digits = right_digits.len() as i64 + adjusted_right_exp - max_len;
        if adjusted_left_digits > 0 || adjusted_right_digits > 0 {
            let mut adjust = adjusted_left_digits.max(adjusted_right_digits);

            // Exactly one of the adjusted exponents is non-zero: the more
            // significant operand's. The other operand is the one shortened.
            //
            // The C++ also decrements the adjusted exponent here, because it
            // feeds the digit-copy loops further down. This port re-derives
            // the alignment from the shortened operands instead, so that
            // bookkeeping would be dead -- but a port of the multiply or
            // divide paths should not assume the same.
            if adjusted_left_exp != 0 {
                let taken = adjust.min(adjusted_left_exp);
                drop_low_digits(&mut right_digits, taken);
                right_exp = right_exp.saturating_add(taken as i32);
                adjust -= taken;
            } else if adjusted_right_exp != 0 {
                let taken = adjust.min(adjusted_right_exp);
                drop_low_digits(&mut left_digits, taken);
                left_exp = left_exp.saturating_add(taken as i32);
                adjust -= taken;
            }

            if adjust != 0 {
                drop_low_digits(&mut left_digits, adjust);
                left_exp = left_exp.saturating_add(adjust as i32);
                drop_low_digits(&mut right_digits, adjust);
                right_exp = right_exp.saturating_add(adjust as i32);
            }
        }

        let left = Number { negative: left_negative, digits: left_digits, exponent: left_exp };
        let right = Number { negative: right_negative, digits: right_digits, exponent: right_exp };
        let min_exp = left.exponent.min(right.exponent);
        let a = left.aligned_to(min_exp);
        let b = right.aligned_to(min_exp);

        let (raw_digits, negative) = if left_negative == right_negative {
            (add_magnitudes(&a, &b), left_negative)
        } else {
            match compare_magnitudes(&a, &b) {
                std::cmp::Ordering::Equal => return Ok(Number::zero()),
                std::cmp::Ordering::Greater => (sub_magnitudes(&a, &b), left_negative),
                std::cmp::Ordering::Less => (sub_magnitudes(&b, &a), right_negative),
            }
        };

        // Deliberately not `assemble` here: that strips leading zeros, and a
        // zero left by a borrow or an absent carry has to still be present
        // for `round_to` to count it. Normalising happens inside `round_to`,
        // after the rounding decision has been made.
        let raw = Number { negative, digits: raw_digits, exponent: min_exp };
        let rounded = raw.round_to(digits);
        Number::assemble(rounded.negative, rounded.digits, rounded.exponent).check_range()
    }

    /// Shortens an over-long operand to the working precision, as
    /// `addSub` and `checkNumber` both do. Truncation, not rounding.
    pub(crate) fn truncated_to(&self, max_length: usize) -> Number {
        if self.digits.len() <= max_length {
            return self.clone();
        }
        let dropped = self.digits.len() - max_length;
        Number {
            negative: self.negative,
            digits: self.digits[..max_length].to_vec(),
            exponent: self.exponent + dropped as i32,
        }
    }
}

/// Compares two digit vectors by value, ignoring leading zeros.
fn compare_magnitudes(a: &[u8], b: &[u8]) -> std::cmp::Ordering {
    let a_start = a.iter().take_while(|d| **d == 0).count();
    let b_start = b.iter().take_while(|d| **d == 0).count();
    let (a, b) = (&a[a_start..], &b[b_start..]);
    a.len().cmp(&b.len()).then_with(|| a.cmp(b))
}

/// Sum of two aligned digit vectors.
///
/// A carry digit is emitted only when there actually is a carry. Emitting an
/// unconditional leading slot would be wrong: the zero counts toward the
/// result length, and at small DIGITS settings rounding then keeps the zero
/// and throws the real digits away -- `1 + 1` at DIGITS 1 comes out as 0.
/// Subtraction is different: its leading zero is a real digit produced by the
/// borrow, and must be kept.
fn add_magnitudes(a: &[u8], b: &[u8]) -> Vec<u8> {
    let n = a.len().max(b.len());
    let mut out = vec![0u8; n];
    let mut carry = 0u8;
    for i in 0..n {
        let x = a.len().checked_sub(i + 1).map_or(0, |k| a[k]);
        let y = b.len().checked_sub(i + 1).map_or(0, |k| b[k]);
        let sum = x + y + carry;
        out[n - 1 - i] = sum % 10;
        carry = sum / 10;
    }
    if carry > 0 {
        out.insert(0, carry);
    }
    out
}

/// `a - b`, where `a >= b` by magnitude.
fn sub_magnitudes(a: &[u8], b: &[u8]) -> Vec<u8> {
    let n = a.len();
    let mut out = vec![0u8; n];
    let mut borrow = 0i8;
    for i in 0..n {
        let x = a[n - 1 - i] as i8;
        let y = b.len().checked_sub(i + 1).map_or(0, |k| b[k] as i8);
        let mut d = x - y - borrow;
        if d < 0 {
            d += 10;
            borrow = 1;
        } else {
            borrow = 0;
        }
        out[n - 1 - i] = d as u8;
    }
    out
}

/// Removes `count` digits from the low end, as the C++ does by walking the
/// end pointer backwards. Never empties the vector.
fn drop_low_digits(digits: &mut Vec<u8>, count: i64) {
    let count = count.max(0) as usize;
    let keep = digits.len().saturating_sub(count).max(1);
    digits.truncate(keep);
}
