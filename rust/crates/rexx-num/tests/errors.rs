//! `ArithError::message` -- the generated-table wiring for `lib.rs`'s error
//! type. `DivideByZero` is fully resolvable and checked against the exact
//! text `build/bin/rexx` prints for `1 / 0`. `Overflow` and `NotWholeNumber`
//! each collapse more than one interpreter sub-message onto a single unit
//! variant (see the doc comment on `ArithError::message`), so these two only
//! pin the generic fallback text against the generated table directly --
//! there is no single live probe that would confirm them.

use rexx_num::{ArithError, DivOp, Number};

fn n(s: &str) -> Number {
    Number::parse(s).unwrap()
}

#[test]
fn divide_by_zero_message_is_42_003() {
    // rust/crates/rexx-num/tests/../../../../interpreter -- provoked with
    // `signal on syntax name oops; r = 1 / 0`, which prints exactly this.
    let err = n("1").div(&n("0"), 9, DivOp::Divide).unwrap_err();
    assert_eq!(err, ArithError::DivideByZero);
    assert_eq!(err.message(), "Arithmetic overflow; divisor must not be zero.");
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
fn overflow_message_falls_back_to_the_bare_major_text() {
    // A mul overflow (42.901) and a pow zero-to-negative-power underflow
    // (42.903) both raise the same `ArithError::Overflow` from this crate's
    // API (see `pow.rs`/`muldiv.rs`, outside this task's scope), so
    // `message()` cannot pick either sub-message honestly and reports the
    // bare 42.000 text instead.
    let mul_overflow = n("9e999999999").mul(&n("9e999999999"), 9).unwrap_err();
    assert_eq!(mul_overflow, ArithError::Overflow);
    let pow_underflow = n("0").pow(&n("-1"), 9).unwrap_err();
    assert_eq!(pow_underflow, ArithError::Overflow);
    assert_eq!(mul_overflow.message(), "Arithmetic overflow/underflow.");
    assert_eq!(pow_underflow.message(), mul_overflow.message());
}

#[test]
fn not_whole_number_message_falls_back_to_the_bare_major_text() {
    // `%`'s not-whole-number case (26.011) and `**`'s (26.008) both raise
    // the same `ArithError::NotWholeNumber`, for the same reason as above.
    let intdiv = n("123456").div(&n("2"), 3, DivOp::IntegerDivide).unwrap_err();
    assert_eq!(intdiv, ArithError::NotWholeNumber);
    let pow = n("2").pow(&n("2.5"), 9).unwrap_err();
    assert_eq!(pow, ArithError::NotWholeNumber);
    assert_eq!(intdiv.message(), "Invalid whole number.");
    assert_eq!(pow.message(), intdiv.message());
}
