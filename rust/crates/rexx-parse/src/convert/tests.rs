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

//! Every case here is a `build/bin/rexxc` measurement, reached through
//! `::OPTIONS DIGITS` and `::OPTIONS FUZZ` for the eighteen-digit conversion,
//! through `TRACE` for the nine-digit one, and through `::CONSTANT` for
//! `is_number`.

use super::*;

/// The `TRACE` skip count's precision, `number_digits()` at its default.
const TRACE_DIGITS: usize = 9;

#[test]
fn a_whole_number_may_be_written_any_way_rexx_writes_one() {
    // Measured through `::options digits`, all rc 0.
    assert_eq!(whole_number(b"12", ARGUMENT_DIGITS), Some(12));
    assert_eq!(whole_number(b"1e2", ARGUMENT_DIGITS), Some(100));
    assert_eq!(whole_number(b"0009", ARGUMENT_DIGITS), Some(9));
    assert_eq!(whole_number(b"9.0", ARGUMENT_DIGITS), Some(9));
    assert_eq!(whole_number(b"9.", ARGUMENT_DIGITS), Some(9));
    assert_eq!(whole_number(b"+9", ARGUMENT_DIGITS), Some(9));
}

#[test]
fn a_number_that_is_not_whole_is_not_a_whole_number() {
    // Measured through `::options digits`, all Error 26.5.
    assert_eq!(whole_number(b"9.5", ARGUMENT_DIGITS), None);
    assert_eq!(whole_number(b"1e-2", ARGUMENT_DIGITS), None);
    assert_eq!(whole_number(b".9", ARGUMENT_DIGITS), None);
    assert_eq!(whole_number(b"", ARGUMENT_DIGITS), None);
    assert_eq!(whole_number(b"abc", ARGUMENT_DIGITS), None);
    assert_eq!(whole_number(b"0.4", ARGUMENT_DIGITS), None);
}

/// The blank rule, which this module got wrong twice before getting it right.
///
/// The first version stripped nothing, so `trace " 9 "` raised 24.1. The second
/// stripped the ends but not the blanks a sign may be followed by, and the probe
/// that was meant to check it used `"- 9"` -- which really is Error 26.5 for
/// `DIGITS`, but because `-9 < 1` fails the RANGE check, not because a sign blank
/// fails the number check. **`"+ 9"` is the discriminating input**, and it is
/// rc 0. Both are asserted so the wrong reason cannot pass again.
#[test]
fn blanks_may_surround_a_number_and_follow_its_sign_but_not_sit_inside_one() {
    // Measured: `::options digits " 9 "` and the tab-padded spelling are rc 0.
    assert_eq!(whole_number(b" 9 ", ARGUMENT_DIGITS), Some(9));
    assert_eq!(whole_number(b"\t9\t", ARGUMENT_DIGITS), Some(9));
    assert_eq!(whole_number(b" 9", ARGUMENT_DIGITS), Some(9));
    assert_eq!(whole_number(b"9 ", ARGUMENT_DIGITS), Some(9));
    // Measured: `::options digits "+ 9"`, `"+  9"` and the tab spelling are all
    // rc 0, and `trace "+ 9"` and `trace "- 9"` are rc 0 too.
    assert_eq!(whole_number(b"+ 9", ARGUMENT_DIGITS), Some(9));
    assert_eq!(whole_number(b"+  9", ARGUMENT_DIGITS), Some(9));
    assert_eq!(whole_number(b"+\t9", ARGUMENT_DIGITS), Some(9));
    assert_eq!(whole_number(b"  +   9  ", ARGUMENT_DIGITS), Some(9));
    // `- 9` is the number -9. `::OPTIONS DIGITS` rejects it, but for its range
    // and not for its blank, which is why this asserts the VALUE while the two
    // directive-level rejections are asserted in `directive/tests.rs`.
    assert_eq!(whole_number(b"- 9", ARGUMENT_DIGITS), Some(-9));
    assert_eq!(whole_number(b"- .5", ARGUMENT_DIGITS), None);
    assert!(is_number(b"- .5"));
    // Measured: `"9 5"` and `"1 e2"` are Error 26.5 and are not numbers at all.
    assert_eq!(whole_number(b"9 5", ARGUMENT_DIGITS), None);
    assert_eq!(whole_number(b"1 e2", ARGUMENT_DIGITS), None);
    assert!(!is_number(b"9 5"));
    assert!(!is_number(b"1 e2"));
    assert!(!is_number(b"+ "));
    assert!(!is_number(b"+"));
    assert!(!is_number(b"++ 3"));
    assert!(!is_number(b"+ - 3"));
    assert!(!is_number(b"3e 2"));
}

/// The conversion ROUNDS to the precision and only then asks whether the result
/// is an integer, so a fraction can survive it. Every case measured through
/// `TRACE` under the default nine digits.
#[test]
fn the_conversion_rounds_to_the_precision_before_asking_if_it_is_whole() {
    // rc 0: ten digits truncate to nine, the dropped 4 does not carry.
    assert_eq!(
        whole_number(b"999999999.4", TRACE_DIGITS),
        Some(999_999_999)
    );
    // rc 0: eleven digits truncate to nine and every surviving decimal is zero.
    assert_eq!(whole_number(b"1.0000000001", TRACE_DIGITS), Some(1));
    // rc 0: the dropped digit carries, and a carry over all-nine decimals is 1.
    assert_eq!(whole_number(b"0.9999999999", TRACE_DIGITS), Some(1));
    // 24.1: that carry makes the value ten digits wide.
    assert_eq!(whole_number(b"999999999.6", TRACE_DIGITS), None);
    // 24.1: nine digits do not exceed the precision, so nothing is rounded and
    // the 6 simply is not whole. This is the control that separates rounding
    // from truncation.
    assert_eq!(whole_number(b"99999999.6", TRACE_DIGITS), None);
    // The same shapes under eighteen digits, both measured rc 0 through
    // `::options digits`.
    assert_eq!(
        whole_number(b"999999999999999999.4", ARGUMENT_DIGITS),
        Some(999_999_999_999_999_999)
    );
    assert_eq!(
        whole_number(b"1.0000000000000000001", ARGUMENT_DIGITS),
        Some(1)
    );
}

/// The precision is the caller's, and the two callers really do differ.
#[test]
fn the_digit_limit_is_the_precision_the_caller_passed() {
    // Measured: `::options digits 123456789012345678` is rc 0 at eighteen digits
    // and `1234567890123456789` is Error 26.5 at nineteen.
    assert_eq!(
        whole_number(b"123456789012345678", ARGUMENT_DIGITS),
        Some(123_456_789_012_345_678)
    );
    assert_eq!(whole_number(b"1234567890123456789", ARGUMENT_DIGITS), None);
    assert_eq!(whole_number(b"1000000000000000000", ARGUMENT_DIGITS), None);
    // Measured: `trace 123456789` is rc 0 and `trace 1234567890` is Error 24.1.
    assert_eq!(whole_number(b"123456789", TRACE_DIGITS), Some(123_456_789));
    assert_eq!(whole_number(b"1234567890", TRACE_DIGITS), None);
    // The limit is on the VALUE's width and not the text's. Measured:
    // `::options digits 1E18` is 26.5 at nineteen digits while `1e17` is rc 0 at
    // eighteen, and `trace 1e9` is 24.1 at ten.
    assert_eq!(whole_number(b"1E18", ARGUMENT_DIGITS), None);
    assert_eq!(
        whole_number(b"1e17", ARGUMENT_DIGITS),
        Some(100_000_000_000_000_000)
    );
    assert_eq!(whole_number(b"1e9", TRACE_DIGITS), None);
    assert_eq!(whole_number(b"1e8", TRACE_DIGITS), Some(100_000_000));
}

#[test]
fn zero_is_zero_however_it_is_spelled() {
    assert_eq!(whole_number(b"0", ARGUMENT_DIGITS), Some(0));
    assert_eq!(whole_number(b"0.000", ARGUMENT_DIGITS), Some(0));
    assert_eq!(whole_number(b"-0", ARGUMENT_DIGITS), Some(0));
    assert_eq!(whole_number(b"0e9999", ARGUMENT_DIGITS), Some(0));
    assert_eq!(whole_number(b"- 0", ARGUMENT_DIGITS), Some(0));
}

#[test]
fn a_negative_whole_number_keeps_its_sign() {
    assert_eq!(whole_number(b"-9", TRACE_DIGITS), Some(-9));
    assert_eq!(whole_number(b"-1e2", TRACE_DIGITS), Some(-100));
    assert_eq!(
        whole_number(b"-999999999.4", TRACE_DIGITS),
        Some(-999_999_999)
    );
}

/// `is_number` accepts every number, whole or not, and rejects everything else.
/// Measured through `::CONSTANT`, whose signed form makes the distinction
/// visible.
#[test]
fn is_number_accepts_a_fraction_where_whole_number_does_not() {
    // Measured: `::constant c -.5`, `-1e2` and `-5.` are all rc 0.
    assert!(is_number(b"-.5"));
    assert!(is_number(b"-1e2"));
    assert!(is_number(b"-5."));
    assert_eq!(whole_number(b"-.5", ARGUMENT_DIGITS), None);
    // Measured: `::constant c -5x` and `-1e` are both Error 19.916.
    assert!(!is_number(b"-5x"));
    assert!(!is_number(b"-1e"));
    assert!(!is_number(b""));
    assert!(!is_number(b"."));
    // Not UTF-8, so not a number. A literal may hold such a byte, a number may
    // not.
    assert!(!is_number(&[b'1', 0xC3]));
}

/// The exponent limits, in both of the two independent places they show up.
///
/// These come from `Number::parse` rather than from anything here, and that is
/// the point: a private acceptance rule had neither of them.
#[test]
fn an_exponent_out_of_range_is_not_a_number() {
    // The limit on the exponent AS WRITTEN. Measured through `::constant`:
    // `-1e999999999` is rc 0 and `-1e1000000000` is Error 19.916.
    assert!(is_number(b"-1e999999999"));
    assert!(!is_number(b"-1e1000000000"));
    assert!(is_number(b"-1e-999999999"));
    assert!(!is_number(b"-1e-1000000000"));
    // The limit on the ADJUSTED exponent, a different check on the same number.
    // Measured: `-9e999999999` is rc 0 and `-99e999999999` is 19.916, so the
    // boundary is not the written nine-nines.
    assert!(is_number(b"-9e999999999"));
    assert!(!is_number(b"-99e999999999"));
    // A number too wide for any precision is still a number, which is why
    // `is_number` must not go through `whole_number`.
    assert!(is_number(b"1234567890123456789012345678901234567890"));
    assert_eq!(
        whole_number(b"1234567890123456789012345678901234567890", ARGUMENT_DIGITS),
        None
    );
}

/// The local valuation walk never fails where `rexx-num` accepted, and over the
/// blank shapes it agrees exactly.
///
/// `decompose` exists only to recover a mantissa and an exponent that `Number`
/// keeps private, and it runs only on text `Number::parse` has already accepted.
/// So the invariant that protects `whole_number` is one-directional: acceptance
/// must imply decomposability, or `whole_number` would silently answer `None`
/// for a number.
///
/// The converse is deliberately NOT required, and one shape shows why:
/// `decompose` accepts `-1e1000000000` because it is syntactically a number,
/// while `Number::parse` rejects it because the exponent is out of range. Range
/// is an acceptance question, and `decompose` does not answer acceptance
/// questions. Requiring equality there would push the exponent limits back into
/// this crate, which is the whole thing this module is arranged to avoid.
///
/// Over `SIGNBLANK_SHAPES` the two DO agree exactly, and that is asserted
/// separately: no shape there is out of range, so any disagreement would be a
/// real divergence in the blank rule, which is the rule that broke.
#[test]
fn the_local_walk_never_fails_where_rexx_num_accepted() {
    for shape in SIGNBLANK_SHAPES.iter().chain(CALLER_SHAPES) {
        if Number::parse(shape).is_some() {
            assert!(
                decompose(shape.as_bytes()).is_some(),
                "Number::parse accepted {shape:?} and the local walk could not \
                 decompose it"
            );
        }
    }
    // And over the blank shapes alone, in both directions.
    for shape in SIGNBLANK_SHAPES {
        assert_eq!(
            decompose(shape.as_bytes()).is_some(),
            Number::parse(shape).is_some(),
            "the local walk and Number::parse disagree on {shape:?}"
        );
    }
}

/// `SIGNBLANK_A` from `rexx-num/tests/gen-curated-sets.py:151`, the operand list
/// of its 2,320-case signblank set. None of these is out of range, so the two
/// implementations must agree on every one in both directions.
const SIGNBLANK_SHAPES: &[&str] = &[
    "+ 3",
    "- 3",
    "+  3",
    "-   3",
    "  + 3  ",
    "+ .5",
    "- .5",
    "+ 3.",
    "+ 0",
    "- 0",
    "+ 1e2",
    "- 1e-2",
    "+ 12345678901",
    "+\t3",
    "\t+ 3\t",
    "3\t",
    "+ 3.14",
    "+ 999999999",
    "- 0.000001",
    "+ 3 e2",
    "3 4",
    "+ - 3",
    "++ 3",
    "3e 2",
    "3e+ 2",
    "+ ",
    "+",
    "- .",
    "+ abc",
];

/// The shapes this crate's own callers reach, including two whose exponent is out
/// of range and which therefore separate acceptance from decomposition.
const CALLER_SHAPES: &[&str] = &[
    "",
    "9",
    "9.",
    ".9",
    "0",
    "0.000",
    "-0",
    "0e9999",
    "1e2",
    "1E18",
    "1e17",
    "9.5",
    "1e-2",
    "abc",
    "-5x",
    "-1e",
    ".",
    "999999999.4",
    "1.0000000001",
    "0.9999999999",
    "999999999.6",
    "99999999.6",
    "1234567890123456789",
    "123456789012345678",
    "-1e999999999",
    "-1e1000000000",
    "-99e999999999",
    "-9e999999999",
    " 9 ",
    "\t9\t",
    "0009",
    "+9",
    "9 5",
    "1 e2",
    "1.2.3",
    "0x1f",
    "1e+2",
    "1e-0",
    "--3",
    "3-",
    "3.4.5",
];

#[test]
fn a_trace_setting_is_any_number_of_question_marks_and_one_letter() {
    // Measured: all rc 0.
    assert!(check_trace_setting(b"").is_ok());
    assert!(check_trace_setting(b"r").is_ok());
    assert!(check_trace_setting(b"?r").is_ok());
    assert!(check_trace_setting(b"??r").is_ok());
    assert!(check_trace_setting(b"results").is_ok());
    assert!(check_trace_setting(b"?").is_ok());
    // Measured: `trace zzz` is Error 24.1.
    assert!(check_trace_setting(b"zzz").is_err());
    assert!(check_trace_setting(b"?z").is_err());
}
