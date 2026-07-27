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
        // multiply to something well outside i32.
        let exponent = left
            .exponent
            .checked_add(right.exponent)
            .and_then(|e| e.checked_add(extra as i32))
            .ok_or(ArithError::Overflow)?;
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
        self.div_inner(other, digits, op, true)
    }

    /// As `div`, but without the range check on the quotient. Used for the
    /// reciprocal inside `**`, where the intermediate is carried at extra
    /// precision and only the final result has to be representable.
    pub(crate) fn div_unchecked(
        &self,
        other: &Number,
        digits: u32,
        op: DivOp,
    ) -> Result<Number, ArithError> {
        self.div_inner(other, digits, op, false)
    }

    fn div_inner(
        &self,
        other: &Number,
        digits: u32,
        op: DivOp,
        range_check: bool,
    ) -> Result<Number, ArithError> {
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
        // lands, from the operand exponents and lengths.
        let calc_exp = left
            .exponent
            .checked_sub(right.exponent)
            .and_then(|e| e.checked_add(left.digits.len() as i32 - right.digits.len() as i32))
            .ok_or(ArithError::Overflow)?;

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

        // value = q * 10^(left.exponent - right.exponent - shift)
        let q_exp = left
            .exponent
            .checked_sub(right.exponent)
            .and_then(|e| e.checked_sub(shift))
            .ok_or(ArithError::Overflow)?;

        if op == DivOp::Divide {
            let raw = Number { negative, digits: q, exponent: q_exp };
            // The range check goes after rounding but BEFORE the trailing
            // zeros come off. Those zeros are significant to the check: a
            // quotient of 1.0120 sits one power of ten lower than the 1.012
            // it prints as, and that is the difference between representable
            // and not.
            let mut rounded = raw.round_to(digits);
            if range_check {
                rounded = rounded.check_range()?;
            }
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
            return Err(ArithError::NotWholeNumber);
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
    let mut rem: Vec<u8> = Vec::new();
    let mut q: Vec<u8> = Vec::new();
    let mut shift = 0i32;
    let mut i = 0usize;
    // Feed digits of the numerator, then zeros, taking one quotient digit
    // per step once the quotient has started.
    while q.len() < want {
        rem.push(if i < n.len() { n[i] } else { 0 });
        if i >= n.len() {
            shift += 1;
        }
        i += 1;
        strip_leading(&mut rem);
        let mut count = 0u8;
        while cmp_digits(&rem, d) != std::cmp::Ordering::Less {
            sub_in_place(&mut rem, d);
            strip_leading(&mut rem);
            count += 1;
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
        if rem.iter().all(|x| *x == 0) && i >= n.len() {
            break;
        }
    }
    (q, rem, shift)
}

fn strip_leading(v: &mut Vec<u8>) {
    let lead = v.iter().take_while(|d| **d == 0).count();
    let lead = lead.min(v.len().saturating_sub(1));
    v.drain(..lead);
}

fn cmp_digits(a: &[u8], b: &[u8]) -> std::cmp::Ordering {
    let a_start = a.iter().take_while(|d| **d == 0).count().min(a.len().saturating_sub(1));
    let b_start = b.iter().take_while(|d| **d == 0).count().min(b.len().saturating_sub(1));
    let (a, b) = (&a[a_start..], &b[b_start..]);
    a.len().cmp(&b.len()).then_with(|| a.cmp(b))
}

/// `a -= b`, where `a >= b`.
fn sub_in_place(a: &mut [u8], b: &[u8]) {
    let mut borrow = 0i8;
    for i in 0..a.len() {
        let ai = a.len() - 1 - i;
        let x = a[ai] as i8;
        let y = b.len().checked_sub(i + 1).map_or(0, |k| b[k] as i8);
        let mut v = x - y - borrow;
        if v < 0 {
            v += 10;
            borrow = 1;
        } else {
            borrow = 0;
        }
        a[ai] = v as u8;
    }
}
