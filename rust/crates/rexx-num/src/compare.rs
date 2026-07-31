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
//!
//! Three public entry points, all reaching the one `numeric_order`/
//! `string_order` pair below rather than each carrying its own copy of the
//! rule: [`compare`] takes `&str`, for a caller that already has one (kept
//! working exactly as before -- `rexx-parse`'s differential harness calls
//! it); [`compare_bytes`] takes `&[u8]`, because a Rexx string is a *byte*
//! string that need not be valid UTF-8 (D14; `reverse('ää')` is the
//! standing example), which `&str` cannot carry; and [`compare_decoded`]
//! additionally accepts an already-parsed `Number` per side, for a caller
//! sitting on one already (`rexx-exec`'s `Body::Text::num` cache exists
//! precisely so a string is not reparsed on every comparison, and
//! comparison is the operation that asks "is this a number?" most often).
//! A hand-written second copy of `string_order` for the byte-slice path
//! would be exactly the divergence this module's own header warns against.

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
///
/// Kept exactly as it was for the callers that already have a `&str` --
/// `rexx-parse`'s differential harness among them -- and now a thin call
/// into [`compare_decoded`], which is the one place the comparison rule
/// itself lives.
pub fn compare(
    a: &str,
    b: &str,
    digits: u64,
    fuzz: u64,
    op: CompareOp,
) -> Result<bool, ArithError> {
    compare_decoded(a.as_bytes(), None, b.as_bytes(), None, digits, fuzz, op)
}

/// [`compare`]'s byte-slice twin, for a caller holding a Rexx value's actual
/// bytes rather than an already-checked `&str` -- see this module's header
/// for why `&str` cannot carry one. Reduces to [`compare_decoded`] with
/// nothing pre-parsed, exactly as [`compare`] does.
pub fn compare_bytes(
    a: &[u8],
    b: &[u8],
    digits: u64,
    fuzz: u64,
    op: CompareOp,
) -> Result<bool, ArithError> {
    compare_decoded(a, None, b, None, digits, fuzz, op)
}

/// [`compare_bytes`], but for a caller that already knows one or both
/// operands' parsed `Number` -- passing it in skips reparsing `bytes` for
/// the numeric family, which is the whole reason this entry point exists
/// rather than only [`compare_bytes`]. `None` means "parse from the bytes if
/// a numeric operator needs a value", which is what both `compare` and
/// `compare_bytes` pass for every operand.
///
/// `bytes` is still required even when `number` is `Some`: the strict
/// family and the non-numeric string fallback both compare the operand's
/// own text, not a value derived from it, and a `Number` does not carry its
/// original spelling (`"007"` and `"7"` parse to the same `Number` but do
/// not strict-compare equal).
pub fn compare_decoded(
    a: &[u8],
    a_number: Option<&Number>,
    b: &[u8],
    b_number: Option<&Number>,
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
        return Ok(op.holds(a.cmp(b)));
    }

    // Parses only when the caller did not already hand in a `Number`, and
    // only once per side, so a caller that already has one never pays for a
    // second parse of the same bytes -- the entire point of this entry
    // point over `compare_bytes`. The two `Option<Number>` locals exist to
    // give a freshly-parsed value somewhere to live long enough to borrow;
    // when `a_number`/`b_number` is already `Some`, neither is touched and
    // nothing is cloned.
    let mut parsed_a = None;
    let mut parsed_b = None;
    let a_number = a_number.or_else(|| {
        parsed_a = parse_bytes(a);
        parsed_a.as_ref()
    });
    let b_number = b_number.or_else(|| {
        parsed_b = parse_bytes(b);
        parsed_b.as_ref()
    });

    let ord = match (a_number, b_number) {
        (Some(na), Some(nb)) => numeric_order(na, nb, digits, fuzz)?,
        // `RexxString::comp`: if either side doesn't convert, this drops to
        // `stringComp` -- not an error. `NumberString::comp` does the same
        // thing symmetrically (`stringValue()->stringComp(...)`) when only
        // its own right-hand argument fails to convert.
        _ => string_order(a, b),
    };
    Ok(op.holds(ord))
}

/// `Number::parse` over bytes that may not be valid UTF-8. A Rexx number's
/// characters are ASCII by definition (`rexx-core`'s `NotNumeric` doc
/// comment makes the same point for the same reason), so invalid UTF-8 can
/// never be one and is treated as a parse failure exactly like malformed
/// ASCII text already is -- there is no third outcome to invent here.
fn parse_bytes(bytes: &[u8]) -> Option<Number> {
    std::str::from_utf8(bytes).ok().and_then(Number::parse)
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
///
/// Takes `&[u8]` rather than `&str`: the C++ this is ported from never
/// assumed UTF-8 either, comparing raw operand bytes, and nothing below
/// decodes a character at any point -- only single blank/tab/space byte
/// values are ever inspected. Measured against the oracle with a
/// deliberately invalid-UTF-8 operand (a lone `'C3'x`) to confirm this: the
/// leading-blank rule strips a real blank byte in front of it exactly as it
/// does for any other operand, with no UTF-8 validity requirement anywhere
/// in the actual comparison.
fn string_order(a: &[u8], b: &[u8]) -> Ordering {
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

    let a = skip_leading_blanks(a);
    let b = skip_leading_blanks(b);
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
