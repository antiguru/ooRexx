use rexx_num::{ArithError, DivOp, Number};

fn n(s: &str) -> Number {
    Number::parse(s).unwrap()
}
fn mul(a: &str, b: &str, d: u64) -> String {
    n(a).mul(&n(b), d).unwrap().format(d)
}
fn div(a: &str, b: &str, d: u64, op: DivOp) -> Result<String, u16> {
    n(a).div(&n(b), d, op).map(|r| r.format(d)).map_err(ArithError::code)
}

#[test]
fn multiplication_truncates_operands_rather_than_rounding_them() {
    // Rounding 1.5 to DIGITS 1 first would give 2 * 2 = 4.
    assert_eq!(mul("2", "1.5", 1), "3");
    assert_eq!(mul("1", "1", 1), "1");
    assert_eq!(mul("7", "8", 9), "56");
    assert_eq!(mul("0.5", "0.5", 9), "0.25");
    assert_eq!(mul("-3", "4", 9), "-12");
    assert_eq!(mul("123456789", "0", 9), "0");
}

#[test]
fn division_stops_when_it_comes_out_even() {
    assert_eq!(div("1", "1", 9, DivOp::Divide).unwrap(), "1");
    assert_eq!(div("1", "2", 9, DivOp::Divide).unwrap(), "0.5");
    assert_eq!(div("1", "0.1", 9, DivOp::Divide).unwrap(), "10");
}

#[test]
fn division_strips_trailing_zeros_where_the_other_operators_keep_them() {
    assert_eq!(div("1", "7.7", 3, DivOp::Divide).unwrap(), "0.13");
    assert_eq!(div("1", "999999999", 3, DivOp::Divide).unwrap(), "1E-9");
    // the remainder keeps them
    assert_eq!(div("100", "6.66666665", 3, DivOp::Remainder).unwrap(), "0.010");
}

#[test]
fn recurring_quotients_fill_the_digits_setting() {
    assert_eq!(div("1", "3", 9, DivOp::Divide).unwrap(), "0.333333333");
    assert_eq!(div("2", "3", 9, DivOp::Divide).unwrap(), "0.666666667");
    assert_eq!(div("1", "3", 3, DivOp::Divide).unwrap(), "0.333");
}

#[test]
fn integer_divide_and_remainder_take_the_sign_of_the_dividend() {
    assert_eq!(div("7", "3", 9, DivOp::IntegerDivide).unwrap(), "2");
    assert_eq!(div("7", "3", 9, DivOp::Remainder).unwrap(), "1");
    assert_eq!(div("-7", "3", 9, DivOp::IntegerDivide).unwrap(), "-2");
    assert_eq!(div("-7", "3", 9, DivOp::Remainder).unwrap(), "-1");
    assert_eq!(div("7", "-3", 9, DivOp::Remainder).unwrap(), "1");
}

#[test]
fn dividing_by_zero_is_error_42_for_all_three_operators() {
    for op in [DivOp::Divide, DivOp::IntegerDivide, DivOp::Remainder] {
        assert_eq!(div("7", "0", 9, op), Err(42));
    }
}

#[test]
fn a_quotient_too_wide_to_be_whole_is_error_26() {
    // 1 % 1e-9 would need ten digits at DIGITS 9.
    assert_eq!(div("1", "1e-9", 9, DivOp::IntegerDivide), Err(26));
    assert_eq!(div("1", "1e-9", 9, DivOp::Remainder), Err(26));
    // but / is fine with it
    assert_eq!(div("1", "1e-9", 9, DivOp::Divide).unwrap(), "1E+9");
}

#[test]
fn a_result_outside_the_representable_range_is_error_42() {
    assert_eq!(
        n("1e999999999").mul(&n("1e999999999"), 9).map_err(ArithError::code),
        Err(42)
    );
    // and the operation must not panic on the way there
    assert_eq!(
        n("9e999999999").mul(&n("9e999999999"), 9).map_err(ArithError::code),
        Err(42)
    );
}

#[test]
fn a_literal_outside_the_representable_range_is_not_a_number_at_all() {
    // Rejected at parse: the interpreter reports error 41, not an overflow.
    assert!(Number::parse("1e1000000000").is_none());
    assert!(Number::parse("1e-1000000000").is_none());
    // The top end is judged on the most significant digit ...
    assert!(Number::parse("123456789e999999999").is_none());
    assert!(Number::parse("123456789e999999991").is_some());
    // ... and the bottom end on the least significant one.
    assert!(Number::parse(".96329e-999999995").is_none());
    assert!(Number::parse("96329e-999999999").is_some());
    // The exponent as written is checked too, before decimals fold in.
    assert!(Number::parse(".235468758140e1000000000").is_none());
    // Zero is exempt from all of it: it has no magnitude.
    assert_eq!(Number::parse("0e1000000996").unwrap().format(9), "0");
    assert_eq!(Number::parse("-0e-1000000246").unwrap().format(9), "0");
}
