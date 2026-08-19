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
//! The public entry points all reach the one `numeric_order`/`string_order`
//! pair below rather than each carrying its own copy of the rule.
//! [`compare`] takes `&str`, for a caller that already has one (kept
//! working exactly as before -- `rexx-parse`'s differential harness calls
//! it); [`compare_bytes`] takes `&[u8]`, because a Rexx string is a *byte*
//! string that need not be valid UTF-8 (D14; `reverse('ää')` is the
//! standing example), which `&str` cannot carry; and [`compare_decoded`]
//! additionally accepts an already-parsed `Number` per side, for a caller
//! sitting on one already (`rexx-exec`'s `Body::Text::num` cache exists
//! precisely so a string is not reparsed on every comparison, and
//! comparison is the operation that asks "is this a number?" most often).
//! [`compare_numbers`] and [`compare_strings`] are `compare_decoded`'s own
//! arms, exposed for a caller that has already settled which arm applies;
//! each says on itself what that saves and what it costs to be wrong about.
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
        return Ok(compare_strings(a, b, op));
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

    match (a_number, b_number) {
        (Some(na), Some(nb)) => Ok(op.holds(numeric_order(na, nb, digits, fuzz)?)),
        // `RexxString::comp`: if either side doesn't convert, this drops to
        // `stringComp` -- not an error. `NumberString::comp` does the same
        // thing symmetrically (`stringValue()->stringComp(...)`) when only
        // its own right-hand argument fails to convert.
        _ => Ok(compare_strings(a, b, op)),
    }
}

/// [`compare_decoded`]'s two byte-comparing arms alone: the strict family,
/// and the fallback a non-strict operator takes when an operand does not
/// convert.
///
/// **Not a shortcut past [`compare_decoded`], but the same arms called
/// directly**, on [`compare_numbers`]'s own terms and for the same reason:
/// that function's strict return and its `_` arm are this call and nothing
/// else, so a caller choosing between them is choosing where the decision is
/// made rather than what the answer is.
///
/// **The entry point for a caller that already knows an operand does not
/// convert**, which [`compare_decoded`] cannot be told: `None` there means
/// "parse it from the bytes", so a caller passing `None` for an operand whose
/// parse it has already attempted and lost buys that same parse a second
/// time. `Interp::compare_values` is where that showed up.
///
/// The two families differ in one thing and it is not the operator: a strict
/// comparison compares the spellings byte for byte, where the fallback strips
/// leading blanks ([`string_order`]). `primitiveIsEqual`/
/// `primitiveStrictComp` are both a plain shorter-prefix-then-length compare,
/// with no blank stripping in either direction -- `" 1" == "1"` is false
/// because the lengths differ, full stop. Rust's slice `Ord` already
/// implements exactly that (shared prefix decides; a tie is broken by
/// length), so there is nothing to hand-roll for it.
pub fn compare_strings(a: &[u8], b: &[u8], op: CompareOp) -> bool {
    if op.is_strict() {
        op.holds(a.cmp(b))
    } else {
        op.holds(string_order(a, b))
    }
}

/// [`compare_decoded`]'s numeric arm alone: both operands already parsed, and
/// **no bytes at all**.
///
/// **The bytes are what this exists to avoid asking for.** [`compare_decoded`]
/// takes them because it owes a string fallback when either side does not
/// convert -- but a caller that has already parsed both sides knows that arm
/// is unreachable, and rendering an operand to text for a parameter nobody
/// reads is what `Interp::compare_values` used to do on every comparison.
/// Measured on `samples/rexxcps.rex`: of 2,520,002 comparisons that reached
/// that rendering, 1,680,001 had both operands parsed and so discarded both
/// renderings.
///
/// **Not a shortcut past [`compare_decoded`], but the same arm called
/// directly**, which is what makes the two impossible to disagree: that
/// function's `(Some, Some)` case is this call and nothing else, so a caller
/// choosing between them is choosing where the bytes are produced rather than
/// what the answer is.
///
/// The strict family never comes here. It compares spellings rather than
/// values -- `"007"` and `"7"` parse alike and are not strictly equal -- so it
/// has no numeric arm to lift out.
pub fn compare_numbers(
    a: &Number,
    b: &Number,
    digits: u64,
    fuzz: u64,
    op: CompareOp,
) -> Result<bool, ArithError> {
    debug_assert!(
        !op.is_strict(),
        "a strict comparison compares spellings and has no numeric arm"
    );
    Ok(op.holds(numeric_order(a, b, digits, fuzz)?))
}

/// `Number::parse_bytes`, named locally for the call sites above.
///
/// A Rexx number's characters are ASCII by definition (`rexx-core`'s
/// `NotNumeric` doc comment makes the same point for the same reason), so
/// invalid UTF-8 can never be one; the parser refuses it as the malformed
/// ASCII it also refuses, and there is no third outcome to invent here.
fn parse_bytes(bytes: &[u8]) -> Option<Number> {
    Number::parse_bytes(bytes)
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
/// Path 3's fast case is a pure optimisation, and [`magnitude_order`] is it:
/// comparing two digit arrays that already fit the working precision gives
/// the same answer as subtracting them, because no rounding can occur either
/// way. `the_two_routes_agree_wherever_both_apply` holds the two against
/// each other rather than trusting that sentence.
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

    let working_digits = digits.saturating_sub(fuzz);

    // Both operands share a sign, so ordering them by magnitude orders them
    // by value once that sign is applied -- a larger magnitude is the larger
    // number when both are positive and the smaller when both are negative.
    if let Some(magnitude) = magnitude_order(a, b, working_digits) {
        return Ok(if sign_a < 0 {
            magnitude.reverse()
        } else {
            magnitude
        });
    }

    // Same non-zero sign: subtracting can only shrink or preserve magnitude
    // relative to the larger operand, which is already within range, so this
    // cannot itself overflow.
    let diff = a.sub(b, working_digits)?;
    Ok(if diff.is_zero() {
        Ordering::Equal
    } else if diff.negative {
        Ordering::Less
    } else {
        Ordering::Greater
    })
}

/// Orders two operands by magnitude without subtracting them, or `None` when
/// the pair does not fit the working precision and only the subtraction
/// decides.
///
/// Ported from the shortcut inside `NumberString::comp`
/// (`NumberStringClass.cpp:3230-3319`). Aligning both operands on the lower
/// of the two exponents gives each an *adjusted length* -- its digit count
/// plus the distance its exponent sits above that floor -- which is how many
/// digits it would occupy in the aligned subtraction. When both fit the
/// working precision, that subtraction would round nothing away, so the
/// digits decide the answer on their own.
///
/// The alignment also makes the digit arrays directly comparable: equal
/// adjusted lengths mean the most significant digits line up at index zero,
/// so a shared-prefix compare is the whole ordering, and a difference in
/// digit count past that prefix only matters where the extra digits are not
/// all zero. That last clause is what keeps `1.5` and `1.50` equal.
///
/// **This does not truncate its operands, and the subtraction route does.**
/// The C++ does not either, and it cannot matter: an operand longer than the
/// working precision has an adjusted length above it too, which is exactly
/// the case handed to the subtraction.
fn magnitude_order(a: &Number, b: &Number, working_digits: u64) -> Option<Ordering> {
    // In i64 like the C++'s `wholenumber_t`, and saturating for `working_
    // digits`: a `DIGITS` setting legitimately reaches 10^18 - 1, which no
    // narrower comparison holds, and the adjusted lengths below span the
    // whole exponent range twice over.
    let min_exponent = i64::from(a.exponent.min(b.exponent));
    let adjusted = |n: &Number| i64::from(n.exponent) - min_exponent + n.digits.len() as i64;
    let (adjusted_a, adjusted_b) = (adjusted(a), adjusted(b));
    let limit = i64::try_from(working_digits).unwrap_or(i64::MAX);
    if adjusted_a > limit || adjusted_b > limit {
        return None;
    }
    if adjusted_a != adjusted_b {
        return Some(adjusted_a.cmp(&adjusted_b));
    }

    let (digits_a, digits_b) = (a.digits.as_slice(), b.digits.as_slice());
    let shared = digits_a.len().min(digits_b.len());
    let prefix = digit_prefix_order(&digits_a[..shared], &digits_b[..shared]);
    if prefix != Ordering::Equal {
        return Some(prefix);
    }
    // The prefix decided nothing, so the longer operand is larger only if it
    // carries a non-zero digit past it. At most one of these tails is
    // non-empty, the adjusted lengths being equal.
    Some(if digits_a[shared..].iter().any(|digit| *digit != 0) {
        Ordering::Greater
    } else if digits_b[shared..].iter().any(|digit| *digit != 0) {
        Ordering::Less
    } else {
        Ordering::Equal
    })
}

/// The ordering of two equal-length digit runs, most significant first.
///
/// The same answer `<[u8]>::cmp` gives for equal lengths, which is the only
/// shape it is handed -- `magnitude_order` slices both operands to their
/// shared length first. `the_two_routes_agree_wherever_both_apply` is what
/// holds it to that: measured, reversing the comparison here, and cutting it
/// to the first digit, each redden that test.
///
/// **What differs is how the answer is reached.** The slice comparison
/// calls libc's `memcmp` through the PLT, and its vectorised body is bought
/// with a call, an argument setup and an alignment prologue -- against a run
/// [`magnitude_order`] has already bounded by the working precision, which is
/// nine digits under Rexx's default `NUMERIC DIGITS`. Measured on
/// `samples/rexxcps.rex`, `memcmp` reached from this comparison carried 9.16%
/// of `numeric_order`'s own samples.
fn digit_prefix_order(a: &[u8], b: &[u8]) -> Ordering {
    for (digit_a, digit_b) in a.iter().zip(b) {
        if digit_a != digit_b {
            return digit_a.cmp(digit_b);
        }
    }
    Ordering::Equal
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Spellings, not values. The two routes can only disagree on a pair
    /// whose digit arrays differ while their values do not -- a trailing
    /// zero, a shifted exponent, one operand shorter than the other -- and a
    /// grid of distinct values would never produce those.
    const SPELLINGS: &[&str] = &[
        "1",
        "1.0",
        "1.00",
        "01",
        "1.5",
        "1.50",
        "1.500",
        "15",
        "150",
        "1.5e1",
        "0.15",
        "0.150",
        "2",
        "2.0",
        "9",
        "9.9",
        "10",
        "100",
        "1e2",
        "1.0e2",
        "99",
        "123456789",
        "1234567890",
        "999999999",
        "0.000015",
        "1.0000001",
        "1.0000002",
        "1e-9",
        "1e9",
    ];

    /// The subtraction this exists to avoid, kept whole as the reference.
    fn subtraction_order(a: &Number, b: &Number, working_digits: u64) -> Ordering {
        let diff = a.sub(b, working_digits).expect("the grid stays in range");
        if diff.is_zero() {
            Ordering::Equal
        } else if diff.negative {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    }

    /// `magnitude_order` answers what subtracting answers, on every pair
    /// where it answers at all.
    ///
    /// The taken/skipped counts are asserted, not printed: an implementation
    /// that returned `None` for everything would satisfy the equality above
    /// vacuously, and one that answered everything would be claiming the
    /// subtraction is never needed. Both are refused here.
    #[test]
    fn the_two_routes_agree_wherever_both_apply() {
        let mut taken = 0usize;
        let mut skipped = 0usize;
        for digits in [1u64, 2, 3, 9, 20] {
            for fuzz in [0u64, 1, 2] {
                let working = digits.saturating_sub(fuzz);
                for a_text in SPELLINGS {
                    for b_text in SPELLINGS {
                        for negative in [false, true] {
                            let mut a = Number::parse(a_text).expect("a spelling");
                            let mut b = Number::parse(b_text).expect("a spelling");
                            a.negative = negative;
                            b.negative = negative;
                            let reference = subtraction_order(&a, &b, working);
                            match magnitude_order(&a, &b, working) {
                                Some(magnitude) => {
                                    taken += 1;
                                    let fast = if negative {
                                        magnitude.reverse()
                                    } else {
                                        magnitude
                                    };
                                    assert_eq!(
                                        fast, reference,
                                        "{a_text} vs {b_text}, negative {negative}, \
                                         digits {digits}, fuzz {fuzz}"
                                    );
                                }
                                None => skipped += 1,
                            }
                        }
                    }
                }
            }
        }
        assert!(
            taken > 0,
            "the fast route never fired, so nothing was compared"
        );
        assert!(
            skipped > 0,
            "the fast route fired everywhere, so the grid never reached the subtraction"
        );
    }

    /// The pairs the fast route must call equal despite differing digit
    /// arrays, and the neighbouring ones it must not.
    ///
    /// Trailing zeros are the whole difficulty: `1.5` and `1.50` are the same
    /// value with different digit counts, and the tail scan is what tells
    /// them apart from `1.5` and `1.51`, which are not.
    #[test]
    fn a_trailing_zero_is_not_a_difference_but_a_trailing_digit_is() {
        let order = |a: &str, b: &str| {
            magnitude_order(
                &Number::parse(a).expect("a spelling"),
                &Number::parse(b).expect("a spelling"),
                9,
            )
        };
        assert_eq!(order("1.5", "1.50"), Some(Ordering::Equal));
        assert_eq!(order("1.50", "1.5"), Some(Ordering::Equal));
        assert_eq!(order("1.5", "1.51"), Some(Ordering::Less));
        assert_eq!(order("1.51", "1.5"), Some(Ordering::Greater));
        assert_eq!(order("15", "1.5"), Some(Ordering::Greater));
        assert_eq!(order("1.5", "15"), Some(Ordering::Less));
        assert_eq!(order("1", "1.00000000"), Some(Ordering::Equal));
        // One zero further and the pair spans ten digits, which is past the
        // working precision, so the shortcut declines it rather than
        // answering -- the zeros are counted before they are read.
        assert_eq!(order("1", "1.000000000"), None);
    }

    /// An operand wider than the working precision goes to the subtraction,
    /// which is the case the shortcut is not allowed to answer.
    #[test]
    fn an_operand_past_the_working_precision_is_left_to_the_subtraction() {
        let wide = Number::parse("1234567890").expect("a spelling");
        let narrow = Number::parse("1").expect("a spelling");
        assert_eq!(magnitude_order(&wide, &narrow, 9), None);
        assert_eq!(magnitude_order(&narrow, &wide, 9), None);
        // The adjacent success: the same pair fits once the precision covers
        // the wider operand, and then the shortcut does answer.
        assert_eq!(magnitude_order(&wide, &narrow, 10), Some(Ordering::Greater));
    }
}
