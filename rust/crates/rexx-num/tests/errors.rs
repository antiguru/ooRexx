//! `ArithError::message`/`additional` -- the generated-table wiring for
//! `lib.rs`'s error type. Every sub-message and substitution *value* is
//! confirmed against `build/bin/rexx`, except where noted: `PowerOverflow`
//! and `PowerExponentNotWhole` substitute the base/exponent as originally
//! written in Rexx source, which this crate's `Number` cannot reproduce
//! exactly (see `ArithError::message`'s doc comment) -- those two tests pin
//! the closest achievable approximation, not a byte-exact match.

use rexx_num::{ArithError, DivOp, Number};

fn n(s: &str) -> Number {
    Number::parse(s).unwrap()
}

#[test]
fn divide_by_zero_message_is_42_003() {
    // Provoked with `signal on syntax name oops; r = 1 / 0`.
    let err = n("1").div(&n("0"), 9, DivOp::Divide).unwrap_err();
    assert_eq!(err, ArithError::DivideByZero);
    assert_eq!(err.message(), "Arithmetic overflow; divisor must not be zero.");
    assert_eq!(err.additional(), Vec::<String>::new());
}

#[test]
fn divide_by_zero_message_is_the_same_for_all_three_operators() {
    // `NumberStringMath2.cpp:355` reports `Error_Overflow_zero` before
    // branching on `/`, `%`, or `//`, so all three share this crate's single
    // `DivideByZero` variant and therefore the same message.
    for op in [DivOp::Divide, DivOp::IntegerDivide, DivOp::Remainder] {
        assert_eq!(
            n("1").div(&n("0"), 9, op).unwrap_err().message(),
            "Arithmetic overflow; divisor must not be zero."
        );
    }
}

#[test]
fn general_overflow_message_is_42_901_substituting_the_adjusted_exponent_and_the_constant_9() {
    // Provoked with `numeric digits 9; r = 9e999999999 * 9e999999999`.
    let err = n("9e999999999").mul(&n("9e999999999"), 9).unwrap_err();
    assert!(matches!(err, ArithError::Overflow { adjusted_exponent: 1999999999 }));
    assert_eq!(err.additional(), vec!["1999999999", "9"]);
    assert_eq!(err.message(), "Arithmetic overflow; exponent (\"1999999999\") exceeds 9 digits.");
}

#[test]
fn general_overflow_message_uses_9_regardless_of_the_active_digits_setting() {
    // &2 is `Numerics::DEFAULT_DIGITS`, a fixed C++ constant, not the active
    // `NUMERIC DIGITS` -- confirmed by provoking the same overflow at DIGITS
    // 9 and DIGITS 15 and getting back identical text ("...exceeds 9
    // digits.") either way.
    let err = n("9e999999999").mul(&n("9e999999999"), 15).unwrap_err();
    assert_eq!(err.additional()[1], "9");
    assert_eq!(err.message(), "Arithmetic overflow; exponent (\"1999999999\") exceeds 9 digits.");
}

#[test]
fn general_underflow_message_is_42_902_substituting_the_raw_exponent_not_the_adjusted_one() {
    // Provoked with `numeric digits 9; r = 1e-999999990 / 1e20`.
    let err = n("1e-999999990").div(&n("1e20"), 9, DivOp::Divide).unwrap_err();
    assert!(matches!(err, ArithError::Underflow { exponent: -1000000010 }));
    assert_eq!(err.additional(), vec!["-1000000010", "9"]);
    assert_eq!(err.message(), "Arithmetic underflow; exponent (\"-1000000010\") exceeds 9 digits.");
}

#[test]
fn zero_to_a_negative_power_message_is_42_903_no_substitution() {
    // Provoked with `r = 0 ** -1`.
    let err = n("0").pow(&n("-1"), 9).unwrap_err();
    assert_eq!(err, ArithError::ZeroToNegativePower);
    assert_eq!(err.additional(), Vec::<String>::new());
    assert_eq!(err.message(), "Arithmetic underflow; zero raised to a negative power.");
}

#[test]
fn power_magnitude_precheck_message_is_42_001() {
    // Provoked with `r = 100 ** 999999999`.
    let err = n("100").pow(&n("999999999"), 9).unwrap_err();
    assert!(matches!(err, ArithError::PowerOverflow { .. }));
    assert_eq!(err.additional(), vec!["100", "**", "999999999"]);
    assert_eq!(err.message(), "Arithmetic overflow detected at:  \"100**999999999\".");
}

#[test]
fn power_magnitude_precheck_message_renders_full_stored_precision_not_the_9_digit_default() {
    // Provoked with `numeric digits 15; r = 123456789012345678 ** 999999999`
    // -- the interpreter's own text keeps every one of the base's 18 digits
    // (not rounded to the active DIGITS, and not truncated to this crate's
    // usual 9-digit default rendering either). `Number` has already lost
    // the base's *original spelling* (there is no exponential form or
    // leading zero here to expose that), so this is the closest achievable
    // match, not a claim of exactness in general -- see
    // `ArithError::message`'s doc comment.
    let base = n("123456789012345678");
    let err = base.pow(&n("999999999"), 15).unwrap_err();
    assert_eq!(err.additional()[0], "123456789012345678");
    assert_eq!(
        err.message(),
        "Arithmetic overflow detected at:  \"123456789012345678**999999999\"."
    );
}

#[test]
fn integer_divide_not_whole_message_is_26_011_no_substitution() {
    // Provoked with `numeric digits 3; r = 123456 % 2`.
    let err = n("123456").div(&n("2"), 3, DivOp::IntegerDivide).unwrap_err();
    assert_eq!(err, ArithError::IntegerDivideNotWhole);
    assert_eq!(err.additional(), Vec::<String>::new());
    assert_eq!(err.message(), "Result of % operation did not result in a whole number.");
}

#[test]
fn remainder_not_whole_message_is_26_012_no_substitution() {
    // Provoked with `numeric digits 3; r = 123456 // 2`.
    let err = n("123456").div(&n("2"), 3, DivOp::Remainder).unwrap_err();
    assert_eq!(err, ArithError::RemainderNotWhole);
    assert_eq!(err.additional(), Vec::<String>::new());
    assert_eq!(err.message(), "Result of // operation did not result in a whole number.");
}

#[test]
fn power_exponent_not_whole_message_is_26_008_substituting_the_exponent() {
    // Provoked with `r = 2 ** 2.5`.
    let err = n("2").pow(&n("2.5"), 9).unwrap_err();
    assert!(matches!(err, ArithError::PowerExponentNotWhole { .. }));
    assert_eq!(err.additional(), vec!["2.5"]);
    assert_eq!(
        err.message(),
        "Operand to the right of the power operator (**) must be a whole number; found \"2.5\"."
    );
}

#[test]
fn power_exponent_not_whole_message_substitutes_the_original_exponent_not_the_rounded_one() {
    // Provoked with `numeric digits 3; r = 2 ** 1.23456` -- the message
    // shows the full "1.23456", not "1.23" (what `as_whole` rounds to
    // before checking wholeness).
    let err = n("2").pow(&n("1.23456"), 3).unwrap_err();
    assert_eq!(err.additional(), vec!["1.23456"]);
    assert_eq!(
        err.message(),
        "Operand to the right of the power operator (**) must be a whole number; found \"1.23456\"."
    );
}

#[test]
fn code_still_matches_every_variant_after_the_split() {
    assert_eq!(n("1").div(&n("0"), 9, DivOp::Divide).unwrap_err().code(), 42);
    assert_eq!(n("9e999999999").mul(&n("9e999999999"), 9).unwrap_err().code(), 42);
    assert_eq!(n("1e-999999990").div(&n("1e20"), 9, DivOp::Divide).unwrap_err().code(), 42);
    assert_eq!(n("0").pow(&n("-1"), 9).unwrap_err().code(), 42);
    assert_eq!(n("100").pow(&n("999999999"), 9).unwrap_err().code(), 42);
    assert_eq!(n("123456").div(&n("2"), 3, DivOp::IntegerDivide).unwrap_err().code(), 26);
    assert_eq!(n("123456").div(&n("2"), 3, DivOp::Remainder).unwrap_err().code(), 26);
    assert_eq!(n("2").pow(&n("2.5"), 9).unwrap_err().code(), 26);
}

#[test]
fn additional_and_message_agree_on_every_placeholder() {
    // additional()'s values, joined into message()'s own text, must appear
    // there verbatim -- the two are not allowed to drift apart. Covers
    // every variant that carries substitutions.
    let cases: Vec<ArithError> = vec![
        n("9e999999999").mul(&n("9e999999999"), 9).unwrap_err(),
        n("1e-999999990").div(&n("1e20"), 9, DivOp::Divide).unwrap_err(),
        n("100").pow(&n("999999999"), 9).unwrap_err(),
        n("2").pow(&n("2.5"), 9).unwrap_err(),
    ];
    for err in cases {
        for sub in err.additional() {
            assert!(err.message().contains(&sub), "{sub:?} missing from {:?}", err.message());
        }
    }
}
