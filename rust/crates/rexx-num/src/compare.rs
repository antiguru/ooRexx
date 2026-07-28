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

//! Rexx comparison operators.
//!
//! Ported from `NumberString::comp` (`NumberStringClass.cpp:3194`), the
//! `equal`/`isGreaterThan`/... family around it (`:3336` on), and their
//! `RexxString` counterparts (`StringClass.cpp:753` `comp`, `:795`
//! `stringComp`, `:920` `primitiveStrictComp`) -- the string side is what
//! actually fires here, because every operand in this crate's harness is a
//! plain parsed string, never a `NumberString` produced by prior arithmetic.
//!
//! Two families:
//!
//! - **Numeric** (`=`, `<`, `>`, `<=`, `>=`, `\=`, `<>`, `><`): converts both
//!   operands to `Number` first. If either fails to convert, falls back to
//!   string comparison rather than erroring -- `"abc" = 1` is a legal
//!   (false) comparison, not a syntax error.
//! - **Strict** (`==`, `\==`, `<<`, `>>`, `<<=`, `>>=`): always a byte
//!   comparison of the operand text, never numeric, never blank-trimmed.

use std::cmp::Ordering;

use crate::{ArithError, Number};

/// The twelve Rexx comparison operators. `\=`, `<>` and `><` all mean
/// `NotEqual`; the interpreter's operator table (`StringClass.cpp:2391-2410`)
/// literally repeats the same method pointer for all three tokens.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum CompareOp {
    Equal,
    NotEqual,
    Greater,
    Less,
    GreaterEqual,
    LessEqual,
    StrictEqual,
    StrictNotEqual,
    StrictGreater,
    StrictLess,
    StrictGreaterEqual,
    StrictLessEqual,
}

impl CompareOp {
    fn is_strict(self) -> bool {
        use CompareOp::*;
        matches!(
            self,
            StrictEqual
                | StrictNotEqual
                | StrictGreater
                | StrictLess
                | StrictGreaterEqual
                | StrictLessEqual
        )
    }

    /// Turns an already-computed `Ordering` into the boolean this operator
    /// asks for. Shared by both families: numeric and strict comparisons
    /// differ in how the `Ordering` is produced, not in how it is read.
    fn holds(self, ord: Ordering) -> bool {
        use CompareOp::*;
        match self {
            Equal | StrictEqual => ord.is_eq(),
            NotEqual | StrictNotEqual => ord.is_ne(),
            Greater | StrictGreater => ord.is_gt(),
            Less | StrictLess => ord.is_lt(),
            GreaterEqual | StrictGreaterEqual => ord.is_ge(),
            LessEqual | StrictLessEqual => ord.is_le(),
        }
    }
}

/// Evaluates a Rexx comparison between two operand strings, as the
/// interpreter would when both operands are ordinary strings (parsed from
/// source, `PARSE`, or similar) rather than the result of prior arithmetic.
///
/// `digits` and `fuzz` are `NUMERIC DIGITS`/`NUMERIC FUZZ`; `fuzz` is ignored
/// for the strict operators, exactly as `NumberString::strictComp` takes no
/// fuzz parameter at all.
pub fn compare(
    a: &str,
    b: &str,
    digits: u64,
    fuzz: u64,
    op: CompareOp,
) -> Result<bool, ArithError> {
    if op.is_strict() {
        // `primitiveIsEqual`/`primitiveStrictComp` are both a plain
        // shorter-prefix-then-length compare, with no blank stripping in
        // either direction -- `" 1" == "1"` is false because the lengths
        // differ, full stop. Rust's slice `Ord` already implements exactly
        // that (shared prefix decides; a tie is broken by length), so there
        // is nothing to hand-roll here.
        return Ok(op.holds(a.as_bytes().cmp(b.as_bytes())));
    }

    let ord = match (Number::parse(a), Number::parse(b)) {
        (Some(na), Some(nb)) => numeric_order(&na, &nb, digits, fuzz)?,
        // `RexxString::comp`: if either side doesn't convert, this drops to
        // `stringComp` -- not an error. `NumberString::comp` does the same
        // thing symmetrically (`stringValue()->stringComp(...)`) when only
        // its own right-hand argument fails to convert.
        _ => string_order(a, b),
    };
    Ok(op.holds(ord))
}

/// Numeric ordering per `NumberString::comp` (`NumberStringClass.cpp:3194`).
///
/// The C++ takes three paths depending on the operands' signs:
///
/// 1. Different (non-zero) signs: decided by sign alone, no computation.
/// 2. Both zero: equal (every spelling of zero is the same value).
/// 3. Same non-zero sign: either a direct digit-array compare (when both
///    operands, aligned to a shared exponent, fit within `digits - fuzz`
///    digits) or, failing that, an actual subtraction at `digits - fuzz`
///    whose sign is the answer.
///
/// Path 3's fast case is a pure optimisation: comparing two digit arrays
/// that already fit the working precision is provably the same answer as
/// subtracting them, because no rounding could occur either way. So rather
/// than port the digit-array memcmp separately, this always takes the
/// subtraction route for same-sign operands -- it is the general case the
/// fast path is short-circuiting, not a different rule.
///
/// Path 1 is not a mere optimisation, though, and must stay separate: it is
/// the reason two enormous, opposite-signed, individually in-range operands
/// (each within `MAX_EXPONENT`) never overflow when compared, even though
/// adding their magnitudes together (what a same-sign subtraction of them
/// would do) could. The interpreter never attempts that computation for
/// opposite signs; this must not either.
fn numeric_order(a: &Number, b: &Number, digits: u64, fuzz: u64) -> Result<Ordering, ArithError> {
    let sign = |n: &Number| {
        if n.is_zero() {
            0
        } else if n.negative {
            -1
        } else {
            1
        }
    };
    let (sign_a, sign_b) = (sign(a), sign(b));
    if sign_a != sign_b {
        return Ok(sign_a.cmp(&sign_b));
    }
    if sign_a == 0 {
        return Ok(Ordering::Equal);
    }

    // Same non-zero sign: subtracting can only shrink or preserve magnitude
    // relative to the larger operand, which is already within range, so this
    // cannot itself overflow.
    let working_digits = digits.saturating_sub(fuzz);
    let diff = a.sub(b, working_digits)?;
    Ok(if diff.is_zero() {
        Ordering::Equal
    } else if diff.negative {
        Ordering::Less
    } else {
        Ordering::Greater
    })
}

/// String fallback per `RexxString::stringComp` (`StringClass.cpp:795`).
///
/// Strips *leading* blanks and tabs from both operands (not trailing), then
/// compares byte-for-byte up to the shorter length. If that shared prefix
/// matches but the operands differ in length, the longer one is only equal
/// if the rest of it is blank/tab too -- otherwise the first non-blank
/// leftover byte is compared against a literal space to decide the order.
/// This is how non-strict `=` still treats `"1"` and `"1  "` as equal.
fn string_order(a: &str, b: &str) -> Ordering {
    fn is_blank(byte: u8) -> bool {
        byte == b' ' || byte == b'\t'
    }
    fn skip_leading_blanks(s: &[u8]) -> &[u8] {
        let lead = s.iter().take_while(|b| is_blank(**b)).count();
        &s[lead..]
    }
    /// Ordering contributed by one side's leftover tail once the shared
    /// prefix has compared equal: blank/tab throughout is a tie, otherwise
    /// the first non-blank byte is compared against a space, with `flip`
    /// negating the result for the side that had the *shorter* string.
    fn tail_order(tail: &[u8], flip: bool) -> Ordering {
        for &byte in tail {
            if !is_blank(byte) {
                let ord = byte.cmp(&b' ');
                return if flip { ord.reverse() } else { ord };
            }
        }
        Ordering::Equal
    }

    let a = skip_leading_blanks(a.as_bytes());
    let b = skip_leading_blanks(b.as_bytes());
    let shared = a.len().min(b.len());
    match a[..shared].cmp(&b[..shared]) {
        Ordering::Equal => {
            if a.len() > shared {
                tail_order(&a[shared..], false)
            } else {
                tail_order(&b[shared..], true)
            }
        }
        other => other,
    }
}
