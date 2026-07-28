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

//! Multiplication and division.
//!
//! Ported from `NumberString::Multiply` (`NumberStringMath2.cpp:106`) and
//! `NumberString::Division` (`:331`), which serves `/`, `%` and `//`.

use crate::{ArithError, Number};

impl Number {
    pub fn mul(&self, other: &Number, digits: u32) -> Result<Number, ArithError> {
        // checkNumber truncates an over-long operand to DIGITS + 1 and does
        // NOT round it. Rounding operands here instead would turn
        // `2 * 1.5` at DIGITS 1 into `2 * 2` = 4, where the answer is 3.
        let left = self.truncated_to(digits as usize + 1);
        let right = other.truncated_to(digits as usize + 1);

        if left.is_zero() || right.is_zero() {
            return Ok(Number::zero());
        }

        let product = mul_magnitudes(&left.digits, &right.digits);
        let digits_usize = digits as usize;

        // Anything beyond DIGITS + 1 digits is dropped from the low end and
        // its count folded into the exponent; the extra digit is what the
        // final rounding then looks at.
        let (kept, extra) = if product.len() > digits_usize {
            let keep = digits_usize + 1;
            (product[..keep.min(product.len())].to_vec(), product.len() - keep)
        } else {
            (product, 0)
        };

        // Checked, not wrapping: two operands near the exponent limit
        // multiply to something well outside i32. See
        // `ArithError::ExponentComputationOverflow`'s doc comment for why
        // this is believed unreachable, and what it falls back to if it
        // ever isn't.
        let exponent = left
            .exponent
            .checked_add(right.exponent)
            .and_then(|e| e.checked_add(extra as i32))
            .ok_or(ArithError::ExponentComputationOverflow)?;
        let negative = left.negative != right.negative;
        let raw = Number { negative, digits: kept, exponent };
        let rounded = raw.round_to(digits);
        Number::assemble(rounded.negative, rounded.digits, rounded.exponent).check_range()
    }
}

/// Exact product of two digit vectors, most significant first.
///
/// Leading zeros are stripped. The C++ derives its accumulator length from a
/// pointer to the first significant digit, so a product that does not carry
/// into the top position simply has one fewer digit -- unlike subtraction,
/// where the zero left by a borrow is a real digit and must be counted.
/// Keeping it here makes `1 * 1` round to 0 at DIGITS 1.
fn mul_magnitudes(a: &[u8], b: &[u8]) -> Vec<u8> {
    let mut out = vec![0u16; a.len() + b.len()];
    for (i, x) in a.iter().rev().enumerate() {
        for (j, y) in b.iter().rev().enumerate() {
            out[a.len() + b.len() - 1 - (i + j)] += (*x as u16) * (*y as u16);
        }
    }
    let mut carry = 0u16;
    for slot in out.iter_mut().rev() {
        let v = *slot + carry;
        *slot = v % 10;
        carry = v / 10;
    }
    debug_assert_eq!(carry, 0, "the output vector is wide enough for any product");
    let lead = out.iter().take_while(|d| **d == 0).count();
    let lead = lead.min(out.len() - 1);
    out[lead..].iter().map(|d| *d as u8).collect()
}

/// Which division the caller wants. All three share one algorithm in the
/// interpreter, differing in when they stop and what they return.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum DivOp {
    /// `/`
    Divide,
    /// `%` -- the integer part of the quotient.
    IntegerDivide,
    /// `//` -- the residue left after an integer divide.
    Remainder,
}

impl Number {
    pub fn div(&self, other: &Number, digits: u32, op: DivOp) -> Result<Number, ArithError> {
        if other.is_zero() {
            return Err(ArithError::DivideByZero);
        }
        if self.is_zero() {
            return Ok(Number::zero());
        }

        let left = self.truncated_to(digits as usize + 1);
        let right = other.truncated_to(digits as usize + 1);
        let negative = left.negative != right.negative;

        // The interpreter's estimate of where the quotient's first digit
        // lands, from the operand exponents and lengths. Same believed-
        // unreachable guard as `mul`'s above.
        let calc_exp = left
            .exponent
            .checked_sub(right.exponent)
            .and_then(|e| e.checked_add(left.digits.len() as i32 - right.digits.len() as i32))
            .ok_or(ArithError::ExponentComputationOverflow)?;

        // A quotient below 1 has no integer part, so % is zero and // is the
        // left operand unchanged.
        if calc_exp < 0 && op != DivOp::Divide {
            return Ok(match op {
                DivOp::IntegerDivide => Number::zero(),
                _ => {
                    let mut r = left.clone();
                    r.negative = self.negative && !r.is_zero();
                    r
                }
            });
        }

        // Long-divide the digit strings, generating one more digit than
        // DIGITS so the final rounding has something to look at.
        let want = digits as usize + 1;
        let (mut q, rem, shift) = long_divide(&left.digits, &right.digits, want);

        // value = q * 10^(left.exponent - right.exponent - shift). Same
        // believed-unreachable guard as the two above.
        let q_exp = left
            .exponent
            .checked_sub(right.exponent)
            .and_then(|e| e.checked_sub(shift))
            .ok_or(ArithError::ExponentComputationOverflow)?;

        if op == DivOp::Divide {
            let raw = Number { negative, digits: q, exponent: q_exp };
            // The range check goes after rounding but BEFORE the trailing
            // zeros come off. Those zeros are significant to the check: a
            // quotient of 1.0120 sits one power of ten lower than the 1.012
            // it prints as, and that is the difference between representable
            // and not.
            let mut rounded = raw.round_to(digits).check_range()?;
            // Division strips trailing zeros; `1 / 7.7` at DIGITS 3 is 0.13,
            // not 0.130. Addition and the remainder operators do the
            // opposite and keep them -- `1.50 + 0.50` is 2.00 and
            // `100 // 6.66666665` is 0.010 -- because there the zeros come
            // from the operands rather than from a generated quotient.
            while rounded.digits.len() > 1 && *rounded.digits.last().unwrap() == 0 {
                rounded.digits.pop();
                rounded.exponent += 1;
            }
            return Number::assemble(rounded.negative, rounded.digits, rounded.exponent).check_range();
        }

        // For % and //, keep only the integer part of the quotient.
        if q_exp < 0 {
            let drop = (-q_exp) as usize;
            if drop >= q.len() {
                q = vec![0];
            } else {
                q.truncate(q.len() - drop);
            }
        }
        let int_digits = Number::assemble(negative, q, q_exp.max(0));
        if !int_digits.is_zero() && int_digits.digits.len() as i32 + int_digits.exponent > digits as i32
        {
            // Only `%`/`//` reach here (`Divide` already returned above),
            // and the interpreter reports each with its own, substitution-
            // free text -- confirmed with `123456 % 2` and `123456 // 2`
            // at DIGITS 3.
            return Err(if op == DivOp::IntegerDivide {
                ArithError::IntegerDivideNotWhole
            } else {
                ArithError::RemainderNotWhole
            });
        }

        Ok(match op {
            DivOp::IntegerDivide => int_digits,
            _ => {
                // remainder = left - (left % right) * right. The intermediate
                // must be computed at enough precision to be exact, or a large
                // dividend loses the low digits that ARE the remainder.
                let _ = rem;
                let exact = (left.digits.len()
                    + right.digits.len()
                    + int_digits.digits.len()
                    + digits as usize
                    + 10) as u32;
                let product = int_digits.mul(&right, exact)?;
                let mut r = left.sub(&product, exact)?;
                r.negative = self.negative && !r.is_zero();
                r.round_to(digits)
            }
        })
    }
}

/// Divides two digit strings, returning `want` quotient digits, the residue,
/// and how many powers of ten the quotient was scaled by.
fn long_divide(n: &[u8], d: &[u8], want: usize) -> (Vec<u8>, Vec<u8>, i32) {
    // The live remainder is `rem[start..]`: leading zeros are skipped by
    // advancing `start` instead of draining them out, which cost a memmove
    // on every subtraction pass. The dead prefix stays zero, so slicing from
    // `start` is always the whole value.
    let mut rem: Vec<u8> = Vec::new();
    let mut start = 0usize;
    let mut q: Vec<u8> = Vec::new();
    let mut shift = 0i32;
    let mut i = 0usize;

    // Digit guesses divide by the divisor's first two digits plus one, so a
    // guess is either correct or errs low, never high -- the estimate the
    // interpreter's `Division` uses, shared with `divide_power`.
    let mut div_char = d[0] as i32 * 10;
    if d.len() > 1 {
        div_char += d[1] as i32;
    }
    div_char += 1;

    // Feed digits of the numerator, then zeros, taking one quotient digit
    // per step once the quotient has started.
    while q.len() < want {
        rem.push(if i < n.len() { n[i] } else { 0 });
        if i >= n.len() {
            shift += 1;
        }
        i += 1;
        while rem.len() - start > 1 && rem[start] == 0 {
            start += 1;
        }
        // The digit accumulates from under-guesses until the remainder drops
        // below the divisor. The remainder enters each step below ten times
        // the divisor, so the total never exceeds 9.
        let mut count = 0u8;
        loop {
            let cur = &rem[start..];
            let multiplier = if cur.len() == d.len() {
                match cur.cmp(d) {
                    // The remainder is smaller: this digit is complete.
                    std::cmp::Ordering::Less => break,
                    // Exactly equal: one last subtraction empties it.
                    std::cmp::Ordering::Equal => {
                        count += 1;
                        rem.clear();
                        rem.push(0);
                        start = 0;
                        break;
                    }
                    std::cmp::Ordering::Greater => cur[0] as i32,
                }
            } else if cur.len() > d.len() {
                // The remainder is longer, so it has at least two digits.
                cur[0] as i32 * 10 + cur[1] as i32
            } else {
                break;
            };
            // A zero guess gets wrapped to 1.
            let m = (multiplier * 10 / div_char).max(1);
            count += m as u8;
            subtract_multiple(&mut rem[start..], d, m);
            while rem.len() - start > 1 && rem[start] == 0 {
                start += 1;
            }
        }
        if q.is_empty() && count == 0 {
            // Not yet reached the first significant quotient digit.
            if i > n.len() + want + d.len() {
                break;
            }
            continue;
        }
        q.push(count);
        // The interpreter stops as soon as the division comes out even
        // rather than padding to the full width, so 1 / 1 is `1` and not
        // `1.00000000`.
        if rem[start..].iter().all(|x| *x == 0) && i >= n.len() {
            break;
        }
    }
    (q, rem.split_off(start), shift)
}

pub(crate) fn strip_leading(v: &mut Vec<u8>) {
    let lead = v.iter().take_while(|d| **d == 0).count();
    let lead = lead.min(v.len().saturating_sub(1));
    v.drain(..lead);
}

/// `left -= m * divisor`, aligned at the low-order ends. The guess `m` is
/// correct or low, never high, so the result cannot go negative. Ported from
/// `NumberString::subtractDivisor` (`NumberStringMath2.cpp:224`); shared by
/// `long_divide` and `pow`'s `divide_power`.
pub(crate) fn subtract_multiple(left: &mut [u8], divisor: &[u8], m: i32) {
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
