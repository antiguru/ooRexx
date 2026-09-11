//! `Number::floor`, `ceiling`, `round` and `is_integer`.

use rexx_num::{DEFAULT_DIGITS, Number};

fn floor(text: &str) -> String {
    Number::parse(text).expect("numeric").floor(DEFAULT_DIGITS)
}

fn ceiling(text: &str) -> String {
    Number::parse(text)
        .expect("numeric")
        .ceiling(DEFAULT_DIGITS)
}

fn round(text: &str) -> String {
    Number::parse(text).expect("numeric").round(DEFAULT_DIGITS)
}

fn whole(text: &str, digits: u64) -> bool {
    Number::parse(text).expect("numeric").is_integer(digits)
}

#[test]
fn a_fraction_goes_to_the_integer_each_direction_names() {
    assert_eq!(floor("1.5"), "1");
    assert_eq!(ceiling("1.5"), "2");
    assert_eq!(floor("-1.5"), "-2");
    assert_eq!(ceiling("-1.5"), "-1");
}

/// The direction is about the value, not the magnitude, so each of the two
/// is the plain truncation on one side of zero and a step on the other.
#[test]
fn the_side_that_truncates_is_the_side_the_direction_points_away_from() {
    assert_eq!(floor("1.9"), "1");
    assert_eq!(ceiling("-1.9"), "-1");
    assert_eq!(floor("-1.1"), "-2");
    assert_eq!(ceiling("1.1"), "2");
}

/// Trailing zero decimals are decimals, and are not a reason to step.
#[test]
fn a_value_with_only_zero_decimals_is_already_whole() {
    assert_eq!(floor("2.0"), "2");
    assert_eq!(ceiling("2.00"), "2");
    assert_eq!(floor("-2.0"), "-2");
    assert_eq!(ceiling("-2.00"), "-2");
    assert_eq!(round("-2.000"), "-2");
}

#[test]
fn every_spelling_of_zero_rounds_to_the_one_zero() {
    assert_eq!(floor("0"), "0");
    assert_eq!(ceiling("0.000"), "0");
    assert_eq!(round("-0.0"), "0");
}

/// Halves go away from zero. The interpreter's own comment calls this
/// `floor(number + .5)`, which would send `-2.5` to `-2`.
#[test]
fn a_half_goes_away_from_zero() {
    assert_eq!(round("0.5"), "1");
    assert_eq!(round("-0.5"), "-1");
    assert_eq!(round("2.5"), "3");
    assert_eq!(round("-2.5"), "-3");
    assert_eq!(round("3.5"), "4");
}

#[test]
fn below_a_half_goes_to_zero_without_a_sign() {
    assert_eq!(round("0.4"), "0");
    assert_eq!(round("-0.4"), "0");
    assert_eq!(round("0.05"), "0");
    assert_eq!(round("-0.05"), "0");
}

/// A value whose magnitude is under one has no integer digits at all, and the
/// step lands on a bare one rather than on a carry.
#[test]
fn a_magnitude_under_one_steps_to_one() {
    assert_eq!(floor("-0.999999999"), "-1");
    assert_eq!(ceiling("0.999999999"), "1");
    assert_eq!(round("0.999999999"), "1");
    assert_eq!(floor("0.999999999"), "0");
    assert_eq!(floor("1E-100"), "0");
    assert_eq!(ceiling("1E-100"), "1");
    assert_eq!(round("1E-100"), "0");
    assert_eq!(floor("-1E-100"), "-1");
    assert_eq!(ceiling("-1E-100"), "0");
    assert_eq!(round("-1E-100"), "0");
}

/// The carry can run off the top of the integer digits.
#[test]
fn a_step_may_carry_past_every_digit() {
    assert_eq!(ceiling("99.5"), "100");
    assert_eq!(floor("-99.5"), "-100");
    assert_eq!(round("99.5"), "100");
    assert_eq!(ceiling("9.5"), "10");
    assert_eq!(round("999999999.5"), "1000000000");
}

/// The value is rounded to `digits` BEFORE the direction is applied, so the
/// answer is a function of the rounded value. At DIGITS 9 `123456789.5`
/// becomes `123456790`, which has no decimals left for `floor` to take.
#[test]
fn the_value_is_rounded_to_the_precision_first() {
    assert_eq!(round("123456789.5"), "123456790");
    assert_eq!(floor("123456789.5"), "123456790");
    assert_eq!(Number::parse("1234").expect("numeric").floor(3), "1230");
    assert_eq!(Number::parse("1236.7").expect("numeric").round(3), "1240");
    assert_eq!(Number::parse("1236.7").expect("numeric").ceiling(3), "1240");
}

/// Never exponential, however wide -- these render through `TRUNC`.
#[test]
fn a_large_value_renders_in_full() {
    assert_eq!(floor("1E9"), "1000000000");
    assert_eq!(floor("1E20"), "100000000000000000000");
    assert_eq!(round("1E20"), "100000000000000000000");
    assert_eq!(floor("1E100").len(), 101);
}

/// Measured through `~modulo`: an answer means whole, 93.940 means not.
#[test]
fn a_value_with_no_exponent_is_whole_however_long_it_is() {
    assert!(whole("1234567890", DEFAULT_DIGITS));
    assert!(whole("12345678901", DEFAULT_DIGITS));
    assert!(whole("7", DEFAULT_DIGITS));
    assert!(whole("-7", DEFAULT_DIGITS));
}

/// The bound is on the adjusted length, and it is the precision the value was
/// created under. `1E8` is whole at DIGITS 9 and `1E9` is not.
#[test]
fn an_exponent_is_bounded_by_the_precision() {
    assert!(whole("1E8", DEFAULT_DIGITS));
    assert!(!whole("1E9", DEFAULT_DIGITS));
    assert!(!whole("1E10", DEFAULT_DIGITS));
    assert!(!whole("1E20", DEFAULT_DIGITS));
    assert!(whole("1E9", 10));
}

#[test]
fn a_value_with_decimals_is_whole_only_if_they_are_all_zero() {
    assert!(whole("10.0", DEFAULT_DIGITS));
    assert!(whole("1.0E1", DEFAULT_DIGITS));
    assert!(!whole("1.5", DEFAULT_DIGITS));
    assert!(!whole("0.5", DEFAULT_DIGITS));
    assert!(whole("0.0", DEFAULT_DIGITS));
    assert!(whole("0", DEFAULT_DIGITS));
}
