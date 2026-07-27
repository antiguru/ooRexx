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

//! Multiplication.
//!
//! Ported from `NumberString::Multiply` (`NumberStringMath2.cpp:106`).

use crate::Number;

impl Number {
    pub fn mul(&self, other: &Number, digits: u32) -> Number {
        // checkNumber truncates an over-long operand to DIGITS + 1 and does
        // NOT round it. Rounding operands here instead would turn
        // `2 * 1.5` at DIGITS 1 into `2 * 2` = 4, where the answer is 3.
        let left = self.truncated_to(digits as usize + 1);
        let right = other.truncated_to(digits as usize + 1);

        if left.is_zero() || right.is_zero() {
            return Number::zero();
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

        let exponent = left.exponent + right.exponent + extra as i32;
        let negative = left.negative != right.negative;
        let raw = Number { negative, digits: kept, exponent };
        let rounded = raw.round_to(digits);
        Number::assemble(rounded.negative, rounded.digits, rounded.exponent)
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
