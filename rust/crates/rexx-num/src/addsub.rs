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
//! digit vectors, and then round to `DIGITS`. Rounding is applied to the
//! result, not to the operands: `NUMERIC DIGITS 1; 0.5 + 0.4` is `0.9`, not
//! `0 + 0`.

use crate::Number;

/// Digits and exponent of a value, as a borrowed view for the alignment code.
struct Aligned {
    digits: Vec<u8>,
    exponent: i32,
}

impl Number {
    /// Extends the digit vector downward so both operands share an exponent.
    fn aligned_to(&self, exponent: i32) -> Aligned {
        let pad = (self.exponent - exponent).max(0) as usize;
        let mut digits = self.digits.clone();
        digits.extend(std::iter::repeat_n(0u8, pad));
        Aligned { digits, exponent }
    }

    pub fn add(&self, other: &Number, digits: u32) -> Number {
        self.add_signed(other, false, digits)
    }

    pub fn sub(&self, other: &Number, digits: u32) -> Number {
        self.add_signed(other, true, digits)
    }

    fn add_signed(&self, other: &Number, negate_other: bool, digits: u32) -> Number {
        let other_negative = other.negative != negate_other;

        // Working at full precision and rounding once at the end is what
        // matches the interpreter; rounding the operands first loses digits
        // the result is entitled to.
        let exponent = self.exponent.min(other.exponent);
        let a = self.aligned_to(exponent);
        let b = other.aligned_to(exponent);

        let (digits_out, negative) = if self.negative == other_negative {
            (add_magnitudes(&a.digits, &b.digits), self.negative)
        } else {
            match compare_magnitudes(&a.digits, &b.digits) {
                std::cmp::Ordering::Equal => return Number::zero(),
                std::cmp::Ordering::Greater => {
                    (sub_magnitudes(&a.digits, &b.digits), self.negative)
                }
                std::cmp::Ordering::Less => {
                    (sub_magnitudes(&b.digits, &a.digits), other_negative)
                }
            }
        };

        Number::assemble(negative, digits_out, exponent).round_to(digits)
    }
}

/// Compares two digit vectors by value, ignoring leading zeros.
fn compare_magnitudes(a: &[u8], b: &[u8]) -> std::cmp::Ordering {
    let a_start = a.iter().take_while(|d| **d == 0).count();
    let b_start = b.iter().take_while(|d| **d == 0).count();
    let (a, b) = (&a[a_start..], &b[b_start..]);
    a.len().cmp(&b.len()).then_with(|| a.cmp(b))
}

fn add_magnitudes(a: &[u8], b: &[u8]) -> Vec<u8> {
    let n = a.len().max(b.len());
    let mut out = vec![0u8; n + 1];
    let mut carry = 0u8;
    for i in 0..n {
        let x = a.len().checked_sub(i + 1).map_or(0, |k| a[k]);
        let y = b.len().checked_sub(i + 1).map_or(0, |k| b[k]);
        let sum = x + y + carry;
        out[n - i] = sum % 10;
        carry = sum / 10;
    }
    out[0] = carry;
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
