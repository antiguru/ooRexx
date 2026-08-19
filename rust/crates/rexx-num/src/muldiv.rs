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

use crate::{ArithError, Digits, Number};

/// `NumberString::FAST_BUFFER` (`NumberStringClass.hpp:414`): below this
/// working size the C++ divides in stack buffers and can never fail an
/// allocation, so the reservation probe in `div` is skipped the same way.
const FAST_BUFFER: u64 = 48;

impl Number {
    pub fn mul(&self, other: &Number, digits: u64) -> Result<Number, ArithError> {
        // checkNumber truncates an over-long operand to DIGITS + 1 and does
        // NOT round it. Rounding operands here instead would turn
        // `2 * 1.5` at DIGITS 1 into `2 * 2` = 4, where the answer is 3.
        let left = self.truncated_to(crate::working_length(digits));
        let right = other.truncated_to(crate::working_length(digits));

        if left.is_zero() || right.is_zero() {
            return Ok(Number::zero());
        }

        if let Some(exact) = exact_integer_product(&left, &right, digits) {
            return Ok(exact);
        }
        Number::mul_long(&left, &right, digits)
    }

    /// `left * right` by long multiplication over the digit vectors, which is
    /// what [`Number::mul`] runs wherever [`exact_integer_product`] declines.
    ///
    /// Both operands arrive already truncated to the working length and
    /// already known non-zero, because `mul` decides both before choosing
    /// between the two routes.
    #[inline(always)]
    fn mul_long(left: &Number, right: &Number, digits: u64) -> Result<Number, ArithError> {
        let product = mul_magnitudes(&left.digits, &right.digits);
        // Saturated: a bare `digits` past usize means "keep everything",
        // which the length test below then decides.
        let digits_usize = usize::try_from(digits).unwrap_or(usize::MAX);

        // Anything beyond DIGITS + 1 digits is dropped from the low end and
        // its count folded into the exponent; the extra digit is what the
        // final rounding then looks at.
        let (kept, extra) = if product.len() > digits_usize {
            let keep = digits_usize + 1;
            (
                Digits::from_slice(&product[..keep.min(product.len())]),
                product.len() - keep,
            )
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
        let raw = Number {
            negative,
            digits: kept,
            exponent,
        };
        let rounded = raw.into_round(digits);
        let result = Number::assemble(rounded.negative, rounded.digits, rounded.exponent);
        result.check_range()?;
        Ok(result)
    }
}

/// The exact product, when both operands are plain integers whose digits and
/// product an `i64` holds, and the product needs no rounding at `digits`.
///
/// **Every step the general path takes is the identity under those
/// conditions**, which is what makes this a shortcut rather than a second
/// implementation: neither operand is longer than the working length, so
/// `truncated_to` borrows; the exact product is no longer than `digits`, so
/// nothing is dropped into the exponent and `into_round` returns it
/// unchanged; and it carries no leading zero for `assemble` to strip. What is
/// skipped is the `Vec<u16>` accumulator and the digit-by-digit long
/// multiplication over it.
///
/// The product's own width is either `width` or one less, so testing `width`
/// declines a product that would in fact have fitted. That is deliberate: the
/// cases it turns away cost the general path, and deciding them exactly would
/// cost every case the leading-digit product it takes to know.
///
/// `Number::div`'s remainder is where this pays. It multiplies the integer
/// quotient by the divisor at a working precision it inflates well past the
/// setting in force, so that product is exact by construction.
fn exact_integer_product(left: &Number, right: &Number, digits: u64) -> Option<Number> {
    if left.exponent != 0 || right.exponent != 0 {
        return None;
    }
    let width = left.digits.len() + right.digits.len();
    // Eighteen digits is the widest product an `i64` holds whatever its
    // digits are, so this bounds the folds below as well as the multiply.
    if width > 18 || width as u64 > digits {
        return None;
    }
    let fold = |number: &Number| {
        number
            .digits
            .iter()
            .fold(0i64, |acc, digit| acc * 10 + i64::from(*digit))
    };
    let mut product = Number::from_i64(fold(left) * fold(right));
    product.negative = left.negative != right.negative;
    Some(product)
}

/// Exact product of two digit vectors, most significant first.
///
/// Leading zeros are stripped. The C++ derives its accumulator length from a
/// pointer to the first significant digit, so a product that does not carry
/// into the top position simply has one fewer digit -- unlike subtraction,
/// where the zero left by a borrow is a real digit and must be counted.
/// Keeping it here makes `1 * 1` round to 0 at DIGITS 1.
fn mul_magnitudes(a: &[u8], b: &[u8]) -> Digits {
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
    pub fn div(&self, other: &Number, digits: u64, op: DivOp) -> Result<Number, ArithError> {
        if other.is_zero() {
            return Err(ArithError::DivideByZero);
        }
        if self.is_zero() {
            return Ok(Number::zero());
        }

        let left = self.truncated_to(crate::working_length(digits));
        let right = other.truncated_to(crate::working_length(digits));
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
                    let mut r = left.into_owned();
                    r.negative = self.negative && !r.is_zero();
                    r
                }
            });
        }

        // Division's working storage is sized by DIGITS itself rather than by
        // its operands: `NumberString::Division`
        // allocates `3 * ((digits + 1) * 2 + 1)` bytes up front, before the
        // first quotient digit (`NumberStringMath2.cpp:401-417`), and a
        // failed allocation is error 5, "System resources exhausted"
        // (`RexxMemory.cpp:1266`). That allocation *is* the whole bound --
        // there is no fixed threshold. Confirmed against `build/bin/rexx`:
        // `4.0 / 2` and `123456.0 % 2` both fail with 5.0 at DIGITS
        // 999999999999999999 even though their results are tiny, because the
        // buffer is sized before anyone looks at the operands; `1 / 7`
        // computes at DIGITS 1e10 but is 5.0 at 1e11 on the same machine
        // (the boundary is what malloc will grant, so it moves with the
        // machine). Mirrored here with a fallible reservation of the same
        // request, so the same boundary falls out of the same cause -- and,
        // like the C++ (`FAST_BUFFER`), only past a small-size cutoff, so
        // ordinary divisions never pay for an allocation probe. This sits
        // after the `calc_exp < 0` early returns above because the C++
        // allocates after its equivalent ones: a `%`/`//` whose quotient
        // has no integer part never reaches the allocation at all.
        //
        // Division is NOT the only operation the interpreter sizes this way.
        // `addSub` requests `2D+1` (`NumberStringMath.cpp:808`), `Multiply`
        // `2(D+1)+1` (`NumberStringMath2.cpp:146`), and power `2(2(D+e+1)+1)`
        // with `e` the exponent's digit count (`NumberStringMath2.cpp:902`),
        // all past the same `FAST_BUFFER` cutoff. Those three have no
        // reservation here, so `+`, `-`, `*` and `**` return a result where
        // the interpreter raises error 5. That is a recorded deviation rather
        // than an oversight: error 5 is resource exhaustion, the boundary is
        // whatever malloc grants and so is machine-dependent, and in the
        // middle of the range the interpreter is OOM-killed rather than
        // raising 5 at all. `phase-2-gate.md` carries the reasoning.
        // Reproducing it would have to mirror the per-operation request sizes
        // above and the zero-operand short-circuits that precede them,
        // because the oracle accepts `'2.0'+0` at the same DIGITS that makes
        // `'2.0'+3` fail.
        //
        // A reservation must NOT go inside `sub` or `mul` themselves:
        // `compare` calls `sub`, the `%`/`//` remainder tail calls both at
        // inflated precision, and the oracle's comparison at max DIGITS
        // succeeds because `NumberString::comp`'s fast path never allocates
        // by DIGITS. Any future fix belongs at the operator entry points.
        let total_digits = digits.saturating_add(1).saturating_mul(2).saturating_add(1);
        if total_digits > FAST_BUFFER {
            let request = usize::try_from(total_digits.saturating_mul(3)).unwrap_or(usize::MAX);
            let mut probe: Vec<u8> = Vec::new();
            probe
                .try_reserve_exact(request)
                .map_err(|_| ArithError::SystemResources)?;
            // The reservation is the whole point of the buffer, and nothing
            // reads it afterwards, so the optimiser is entitled to delete the
            // allocation and with it the failure this arm exists to report.
            // It does: under `lto = "fat"` and without this line, `numeric
            // digits 999999999999999999; r = 4.0 / 2` answers `2` instead of
            // raising error 5, while the same build without LTO raises it.
            // `black_box` forces the vector to exist, which forces the
            // allocation to have been attempted.
            std::hint::black_box(&probe);
        }

        // Long-divide the digit strings, generating one more digit than
        // DIGITS so the final rounding has something to look at.
        let want = crate::working_length(digits);
        let (mut q, shift) = long_divide(&left.digits, &right.digits, want);

        // value = q * 10^(left.exponent - right.exponent - shift). Same
        // believed-unreachable guard as the two above.
        let q_exp = left
            .exponent
            .checked_sub(right.exponent)
            .and_then(|e| e.checked_sub(shift))
            .ok_or(ArithError::ExponentComputationOverflow)?;

        if op == DivOp::Divide {
            let raw = Number {
                negative,
                digits: q,
                exponent: q_exp,
            };
            // The range check goes after rounding but BEFORE the trailing
            // zeros come off. Those zeros are significant to the check: a
            // quotient of 1.0120 sits one power of ten lower than the 1.012
            // it prints as, and that is the difference between representable
            // and not.
            let mut rounded = raw.into_round(digits);
            rounded.check_range()?;
            // Division strips trailing zeros; `1 / 7.7` at DIGITS 3 is 0.13,
            // not 0.130. Addition and the remainder operators do the
            // opposite and keep them -- `1.50 + 0.50` is 2.00 and
            // `100 // 6.66666665` is 0.010 -- because there the zeros come
            // from the operands rather than from a generated quotient.
            while rounded.digits.len() > 1 && *rounded.digits.last().unwrap() == 0 {
                rounded.digits.pop();
                rounded.exponent += 1;
            }
            let result = Number::assemble(rounded.negative, rounded.digits, rounded.exponent);
            result.check_range()?;
            return Ok(result);
        }

        // For % and //, keep only the integer part of the quotient.
        if q_exp < 0 {
            let drop = (-q_exp) as usize;
            if drop >= q.len() {
                q = Digits::single(0);
            } else {
                q.truncate(q.len() - drop);
            }
        }
        let int_digits = Number::assemble(negative, q, q_exp.max(0));
        // Compared in i64, with `digits` saturated into it rather than
        // narrowed: `digits` is a bare unbounded u64 (and even the
        // `Settings`-bounded value legitimately reaches 10^18 - 1 now), so
        // an `as i32`/`as i64` narrowing wraps negative and makes every
        // non-zero quotient look too wide -- the same silent-narrowing
        // defect `format` and `as_whole` each fixed before this. Saturation
        // is exact: the left side is a digit count plus an exponent,
        // nowhere near i64::MAX.
        let digits_i64 = i64::try_from(digits).unwrap_or(i64::MAX);
        if !int_digits.is_zero()
            && int_digits.digits.len() as i64 + i64::from(int_digits.exponent) > digits_i64
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
                // Carried in u64, saturating: the old `as u32` narrowing of
                // this sum truncated the working precision for large
                // `digits` -- the same class as the comparison fix above,
                // on the arithmetic side.
                let exact_extra =
                    (left.digits.len() + right.digits.len() + int_digits.digits.len() + 10) as u64;
                let exact = digits.saturating_add(exact_extra);
                let product = int_digits.mul(&right, exact)?;
                let mut r = left.sub(&product, exact)?;
                r.negative = self.negative && !r.is_zero();
                r.into_round(digits)
            }
        })
    }
}

/// The widest divisor [`short_divide`] can carry in a machine word.
///
/// Its running remainder is always below the divisor, and one more numerator
/// digit takes it to `remainder * 10 + digit`. With the divisor under
/// `10^18` that value stays under `10^19`, which a `u64` holds -- `u64::MAX`
/// is above `1.8 * 10^19`. Nineteen would not fit.
///
/// The bound is on the *divisor*, and `Number::div` truncates its operands to
/// `working_length(digits)`, so every division at the default `NUMERIC
/// DIGITS 9` is inside it whatever its operands look like.
const SHORT_DIVISOR_DIGITS: usize = 18;

/// Divides two digit strings, returning `want` quotient digits, the residue,
/// and how many powers of ten the quotient was scaled by.
///
/// Both routes below produce the same digits; which one runs is decided by
/// whether the divisor fits a machine word. That agreement is asserted rather
/// than argued: `divide_route_tests` runs the two against each other over a
/// grid of numerators, divisors and widths.
fn long_divide(n: &[u8], d: &[u8], want: usize) -> (Digits, i32) {
    if d.len() <= SHORT_DIVISOR_DIGITS {
        let divisor = d
            .iter()
            .fold(0u64, |acc, digit| acc * 10 + u64::from(*digit));
        // A zero divisor never arrives -- `Number::div` answers `DivideByZero`
        // before this -- and neither route terminates on one, so the test
        // routes it to the behaviour that was already here rather than to a
        // new division by zero.
        if divisor != 0 {
            return short_divide(n, divisor, d.len(), want);
        }
    }
    wide_divide(n, d, want)
}

/// [`long_divide`] for a divisor a `u64` holds, which is the schoolbook short
/// division: one hardware divide per quotient digit, and no working storage
/// at all.
///
/// **The digit it produces is the digit the wide path produces**, because
/// that path's inner loop is subtracting the divisor out of the running
/// remainder until what is left is smaller -- which is the quotient and the
/// residue of exactly this division. The remainder enters each step below the
/// divisor, so `remainder * 10 + digit` is below ten times it and the digit
/// is a digit.
///
/// The stopping rules are the wide path's, restated against a `u64`
/// remainder: a quotient that has not started yet swallows a zero digit, a
/// remainder of zero with the numerator exhausted ends the division early so
/// that `1 / 1` is `1` rather than a padded `1.00000000`, and `shift` counts
/// the zeros appended past the numerator's own digits.
fn short_divide(n: &[u8], divisor: u64, divisor_len: usize, want: usize) -> (Digits, i32) {
    let mut remainder = 0u64;
    let mut q = Digits::new();
    let mut shift = 0i32;
    let mut i = 0usize;
    // Saturating, for the reason the wide path's own copy of this bound
    // gives: `want` is itself saturated by `working_length`.
    let exhausted = n.len().saturating_add(want).saturating_add(divisor_len);

    while q.len() < want {
        let next = if i < n.len() {
            n[i]
        } else {
            shift += 1;
            0
        };
        i += 1;
        remainder = remainder * 10 + u64::from(next);
        let digit = (remainder / divisor) as u8;
        remainder %= divisor;
        if q.is_empty() && digit == 0 {
            // Not yet reached the first significant quotient digit.
            if i > exhausted {
                break;
            }
            continue;
        }
        q.push(digit);
        if remainder == 0 && i >= n.len() {
            break;
        }
    }
    (q, shift)
}

/// [`long_divide`] for a divisor too wide for a machine word: guess a
/// multiple from the divisor's leading digits, subtract it out, and repeat.
fn wide_divide(n: &[u8], d: &[u8], want: usize) -> (Digits, i32) {
    // The live remainder is `rem[start..]`: leading zeros are skipped by
    // advancing `start` instead of draining them out, which cost a memmove
    // on every subtraction pass. The dead prefix stays zero, so slicing from
    // `start` is always the whole value.
    let mut rem: Vec<u8> = Vec::new();
    let mut start = 0usize;
    // The quotient becomes the result's digits, so it is built in the same
    // representation they are; the working remainder above is scratch that
    // never leaves this function and stays a plain vector.
    let mut q = Digits::new();
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
            // Saturating: `want` is itself saturated (`working_length`), so
            // this bound must stay huge rather than wrap -- producer-side
            // saturation needs matching consumer-side arithmetic, or the
            // sum overflows in debug at exactly the values the saturation
            // was added for.
            if i > n.len().saturating_add(want).saturating_add(d.len()) {
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
    // The residue is not returned: its one caller discarded it, because a
    // remainder good enough to report has to be recomputed at exact
    // precision rather than read off the division (see `DivOp::Remainder`
    // above). Building it cost a `split_off`, which allocates.
    //
    // **`rem` stays a `Vec` rather than becoming a `Digits`.** The inline type
    // saves the allocations, and costs more than they are worth: `push` and
    // `as_mut_slice` branch on which arm holds the digits, and both sit inside
    // the per-digit inner loop where a vector hands out a pointer instead.
    (q, shift)
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
        let sub = divisor
            .len()
            .checked_sub(i + 1)
            .map_or(0, |k| divisor[k] as i32 * m);
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
    debug_assert_eq!(
        carry, 0,
        "an under-guess never drives the remainder negative"
    );
}

#[cfg(test)]
mod mul_shortcut_tests {
    use super::exact_integer_product;
    use crate::Number;

    /// Spellings either side of what the shortcut admits: plain integers, a
    /// value carrying an exponent, one with a fractional part, and widths
    /// around the `i64` bound.
    fn population() -> Vec<Number> {
        [
            "1",
            "-1",
            "7",
            "-7",
            "9",
            "10",
            "-10",
            "99",
            "100",
            "1E+2",
            "-1E+2",
            "1.5",
            "-1.5",
            "0.001",
            "12345",
            "999999999",
            "1000000000",
            "123456789",
            "123456789012345678",
            "1234567890123456789",
        ]
        .iter()
        .map(|text| Number::parse(text).expect("a spelling in the population parses"))
        .collect()
    }

    /// Wherever the shortcut answers, it answers what the long multiplication
    /// answers.
    ///
    /// This is the whole of its licence: it exists to skip work, so any
    /// disagreement is a defect in it rather than a second opinion. The
    /// precisions span both sides of every operand width in the population,
    /// because what the shortcut may take is decided against the precision
    /// and not against the operands alone.
    #[test]
    fn the_integer_shortcut_answers_what_the_long_multiplication_answers() {
        let population = population();
        let mut taken = 0;
        for left in &population {
            for right in &population {
                for digits in [1u64, 2, 3, 5, 9, 10, 18, 20, 40] {
                    let working = crate::working_length(digits);
                    let left = left.truncated_to(working);
                    let right = right.truncated_to(working);
                    if left.is_zero() || right.is_zero() {
                        continue;
                    }
                    let Some(shortcut) = exact_integer_product(&left, &right, digits) else {
                        continue;
                    };
                    taken += 1;
                    assert_eq!(
                        shortcut,
                        Number::mul_long(&left, &right, digits)
                            .expect("the long path answers for an in-range product"),
                        "{} * {} at DIGITS {digits}",
                        left.format(40),
                        right.format(40)
                    );
                }
            }
        }
        // A shortcut that declined everything would satisfy the loop above
        // without saying anything.
        assert!(taken > 0, "the shortcut never fired");
    }

    /// The shortcut declines every operand it must, so that the assertion
    /// above is a statement about a real boundary rather than about an empty
    /// set on one side of it.
    #[test]
    fn the_integer_shortcut_declines_what_it_cannot_answer_exactly() {
        let ten = Number::parse("10").expect("ten");
        let scaled = Number::parse("1E+2").expect("a scaled integer");
        let fraction = Number::parse("1.5").expect("a fraction");
        let wide = Number::parse("1234567890123456789").expect("nineteen digits");

        // A product wider than the precision has to be rounded, which the
        // shortcut does not do. `10 * 10` is decided on the summed width of
        // four rather than on the three digits `100` actually takes, so
        // `DIGITS 3` is declined although its product would have fitted --
        // the conservatism the shortcut's own comment describes, pinned here
        // so that narrowing it later is a visible change rather than a quiet
        // one.
        assert!(exact_integer_product(&ten, &ten, 2).is_none());
        assert!(exact_integer_product(&ten, &ten, 3).is_none());
        assert!(exact_integer_product(&ten, &ten, 4).is_some());

        // An operand whose digits do not sit at the units place.
        assert!(exact_integer_product(&scaled, &ten, 40).is_none());
        assert!(exact_integer_product(&fraction, &ten, 40).is_none());

        // A product past what an `i64` holds.
        assert!(exact_integer_product(&wide, &wide, 40).is_none());
    }
}

#[cfg(test)]
mod divide_route_tests {
    use super::{SHORT_DIVISOR_DIGITS, short_divide, wide_divide};
    use crate::{DivOp, Number};

    /// Digit strings with a non-zero leading digit, which is what a canonical
    /// `Number` hands `long_divide` and what both routes are written against.
    fn strings() -> Vec<Vec<u8>> {
        let seeds: &[&[u8]] = &[
            &[1],
            &[3],
            &[7],
            &[9],
            &[1, 0],
            &[1, 2],
            &[5, 0, 0],
            &[9, 9],
            &[9, 9, 9, 9, 9, 9, 9, 9, 9],
            &[1, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            &[1, 2, 3, 4, 5, 6, 7, 8, 9],
            &[7, 0, 0, 0, 0, 0, 1],
            &[9, 8, 7, 6, 5, 4, 3, 2, 1, 0, 1, 2, 3, 4, 5, 6, 7, 8],
            &[1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            &[9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9, 9],
            &[
                1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5, 6, 7, 8, 9, 1, 2, 3, 4, 5,
            ],
        ];
        seeds.iter().map(|s| s.to_vec()).collect()
    }

    /// The two routes answer the same quotient and the same scaling.
    ///
    /// This is the whole licence for having two: the short one exists to skip
    /// the guess-and-subtract, so any disagreement is a defect in it rather
    /// than a second opinion. `want` spans both sides of every numerator
    /// width in the population, because where the division stops decides how
    /// many digits either route emits.
    #[test]
    fn a_short_divisor_divides_the_same_way_the_wide_path_does() {
        let population = strings();
        let mut checked = 0usize;
        let mut ended_early = 0usize;
        let mut ran_past_the_numerator = 0usize;

        for n in &population {
            for d in &population {
                if d.len() > SHORT_DIVISOR_DIGITS {
                    continue;
                }
                let divisor = d
                    .iter()
                    .fold(0u64, |acc, digit| acc * 10 + u64::from(*digit));
                for want in [1usize, 2, 3, 9, 10, 21, 25, 40] {
                    let short = short_divide(n, divisor, d.len(), want);
                    let wide = wide_divide(n, d, want);
                    assert_eq!(
                        short, wide,
                        "{n:?} / {d:?} to {want} digit(s): short {short:?}, wide {wide:?}"
                    );
                    checked += 1;
                    if short.0.len() < want {
                        ended_early += 1;
                    }
                    if short.1 > 0 {
                        ran_past_the_numerator += 1;
                    }
                }
            }
        }

        // Floors, so a grid that stopped generating cases -- or one that only
        // ever ran the division to its full width -- fails rather than passes
        // empty. Both other exits are where the two routes keep separate
        // bookkeeping and so are where they could part company.
        assert!(checked > 500, "only {checked} case(s)");
        assert!(ended_early > 0, "no case stopped before `want` digits");
        assert!(ran_past_the_numerator > 0, "no case scaled the quotient");
    }

    /// The constant is the widest divisor whose remainder survives taking one
    /// more numerator digit, and one digit wider it would not.
    ///
    /// `short_divide`'s remainder is below the divisor, so the value it forms
    /// is at most `(divisor - 1) * 10 + 9`; the bound is checked against the
    /// widest divisor of each width rather than argued about.
    #[test]
    fn the_short_divisor_width_is_the_one_a_machine_word_holds() {
        let widest = |width: u32| 10u128.pow(width) - 1;
        let step = |divisor: u128| (divisor - 1) * 10 + 9;
        assert!(step(widest(SHORT_DIVISOR_DIGITS as u32)) <= u128::from(u64::MAX));
        assert!(step(widest(SHORT_DIVISOR_DIGITS as u32 + 1)) > u128::from(u64::MAX));
    }

    /// Either side of the width where the route changes, through the operator
    /// itself, so the boundary is exercised as a division and not only as a
    /// property of the constant.
    ///
    /// The two quotients are the oracle's own, measured at `NUMERIC DIGITS
    /// 20`, and they differ in magnitude as well as in digits -- the pair is
    /// one divisor digit apart, so a route that dropped or gained a scaling
    /// step would show here rather than in a last-digit disagreement.
    #[test]
    fn a_divisor_either_side_of_the_route_boundary_divides_correctly() {
        let short = "999999999999999999";
        let wide = "9999999999999999999";
        assert_eq!(short.len(), SHORT_DIVISOR_DIGITS);
        for (text, quotient) in [
            (short, "1000000000000000001"),
            (wide, "100000000000000000.01"),
        ] {
            let divisor = Number::parse(text).expect("a literal divisor");
            let dividend =
                Number::parse("999999999999999999999999999999999999").expect("a literal");
            assert_eq!(
                dividend
                    .div(&divisor, 20, DivOp::Divide)
                    .expect("in range")
                    .format(20),
                quotient
            );
        }
        // The same pair as an exact division, where the quotient terminates
        // rather than being rounded, so the early exit is what ends it.
        for text in [short, wide] {
            let divisor = Number::parse(text).expect("a literal divisor");
            let dividend = divisor
                .mul(&Number::parse("7").expect("seven"), 40)
                .expect("in range");
            assert_eq!(
                dividend
                    .div(&divisor, 40, DivOp::Divide)
                    .expect("in range")
                    .format(40),
                "7"
            );
        }
    }
}
