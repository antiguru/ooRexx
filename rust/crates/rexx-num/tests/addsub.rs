use rexx_num::Number;

fn add(a: &str, b: &str, digits: u32) -> String {
    Number::parse(a)
        .unwrap()
        .add(&Number::parse(b).unwrap(), digits)
        .unwrap()
        .format(digits)
}
fn sub(a: &str, b: &str, digits: u32) -> String {
    Number::parse(a)
        .unwrap()
        .sub(&Number::parse(b).unwrap(), digits)
        .unwrap()
        .format(digits)
}

#[test]
fn ordinary_arithmetic() {
    assert_eq!(add("1", "1", 9), "2");
    assert_eq!(sub("7", "3", 9), "4");
    assert_eq!(sub("3", "7", 9), "-4");
    assert_eq!(add("0.1", "0.2", 9), "0.3");
    assert_eq!(sub("2.0", "2.0", 9), "0");
}

#[test]
fn trailing_zeros_propagate_through_the_result() {
    assert_eq!(add("1.50", "0.50", 9), "2.00");
    assert_eq!(add("1.5", "0.5", 9), "2.0");
}

#[test]
fn a_borrow_leaves_a_leading_zero_that_rounding_must_count() {
    // The raw result of 1e9 - 1 is 0999999999: ten digits. At DIGITS 9 that
    // rounds up to 1000000000; at DIGITS 10 it fits and stays exact.
    assert_eq!(sub("1e9", "1", 9), "1.00000000E+9");
    assert_eq!(sub("1e9", "1", 10), "999999999");
    assert_eq!(sub("1e8", "1", 9), "99999999");
}

#[test]
fn addition_emits_a_carry_digit_only_when_there_is_a_carry() {
    // Prepending an unconditional zero slot makes 1 + 1 round to 0 at DIGITS 1.
    assert_eq!(add("1", "1", 1), "2");
    assert_eq!(add("1", "5", 1), "6");
    assert_eq!(add("999999999", "1", 9), "1.00000000E+9");
}

#[test]
fn an_operand_too_small_to_reach_the_result_is_discarded() {
    assert_eq!(add("1", "1e-9", 9), "1.00000000");
    assert_eq!(add("1", "1e-10", 9), "1");
    assert_eq!(add("123456789", "0.1", 9), "123456789");
    assert_eq!(add("12345678", "0.1", 9), "12345678.1");
}

#[test]
fn adding_zero_returns_the_other_operand_canonicalised() {
    assert_eq!(add("0", "1e9", 9), "1E+9");
    assert_eq!(add("0", "123456789", 9), "123456789");
    assert_eq!(add("1000000000", "0", 9), "1.00000000E+9");
}

#[test]
fn alignment_adjustment_changes_which_digit_rounding_sees() {
    // Without the adjustment block this is 2.3.
    assert_eq!(sub("12.3400", "9.999999995", 3), "2.4");
    assert_eq!(sub("9.999999995", "12.3400", 3), "-2.4");
}
