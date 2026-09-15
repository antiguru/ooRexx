//! `Number::int64_value` and `Number::unsigned_int64_value`, the conversions a
//! native argument declared as a C integer goes through.
//!
//! Every expectation was measured on the oracle through `orxmethod`'s echo
//! methods (`TestInt64Arg`, `TestUint64Arg` and the narrower ones, which range
//! check what these answer).

use rexx_num::{DIGITS64, Number};

fn signed(text: &str) -> Option<i64> {
    Number::parse(text)?.int64_value(DIGITS64)
}

fn unsigned(text: &str) -> Option<u64> {
    Number::parse(text)?.unsigned_int64_value(DIGITS64)
}

#[test]
fn a_whole_number_converts_however_it_is_written() {
    for (text, value) in [
        ("0", 0),
        ("-0", 0),
        ("0.0", 0),
        ("1.0", 1),
        ("1E2", 100),
        ("1e+2", 100),
        (" 12 ", 12),
        ("+5", 5),
        ("10E-1", 1),
        ("5.", 5),
        ("-2147483649", -2_147_483_649),
    ] {
        assert_eq!(signed(text), Some(value), "int64 {text}");
    }
    for (text, value) in [("0", 0), ("-0", 0), ("1.0", 1), ("1E2", 100), (" 12 ", 12)] {
        assert_eq!(unsigned(text), Some(value), "uint64 {text}");
    }
}

#[test]
fn a_fraction_is_not_whole() {
    for text in ["1.5", "2.50", "1E-1", ".5"] {
        assert_eq!(signed(text), None, "int64 {text}");
        assert_eq!(unsigned(text), None, "uint64 {text}");
    }
}

#[test]
fn the_ends_of_each_range() {
    assert_eq!(signed("9223372036854775807"), Some(i64::MAX));
    assert_eq!(signed("9223372036854775808"), None);
    assert_eq!(signed("-9223372036854775808"), Some(i64::MIN));
    assert_eq!(signed("-9223372036854775809"), None);
    assert_eq!(signed("9223372036854775807.0"), Some(i64::MAX));
    assert_eq!(unsigned("18446744073709551615"), Some(u64::MAX));
    assert_eq!(unsigned("18446744073709551615.0"), Some(u64::MAX));
    assert_eq!(unsigned("18446744073709551616"), None);
    assert_eq!(unsigned("1E19"), Some(10_000_000_000_000_000_000));
    assert_eq!(unsigned("1E20"), None);
    assert_eq!(unsigned("-1"), None);
}

/// The smallest `i64` converts only on the path that allows one past the
/// largest, which a decimal point leaves.
#[test]
fn the_smallest_value_with_a_decimal_point_does_not_convert() {
    assert_eq!(signed("-9223372036854775808.0"), None);
}

/// Past twenty digits the digits are cut and the first one cut carries.
#[test]
fn a_number_wider_than_twenty_digits_rounds_before_it_converts() {
    assert_eq!(
        unsigned("1234567890123456789.95"),
        Some(1_234_567_890_123_456_790)
    );
    assert_eq!(unsigned("1234567890123456789.45"), None);
    assert_eq!(
        unsigned("9999999999999999999.9999"),
        Some(10_000_000_000_000_000_000)
    );
    assert_eq!(signed("9999999999999999999.9999"), None);
    assert_eq!(
        signed("999999999999999999.9999"),
        Some(1_000_000_000_000_000_000)
    );
    assert_eq!(signed("1.99999999999999999995"), Some(2));
    assert_eq!(signed("1.9999999999999999995"), None);
    assert_eq!(
        signed("2147483646.9999999999999999999999"),
        Some(2_147_483_647)
    );
    assert_eq!(signed("2147483647.4999999999999999999999"), None);
    assert_eq!(
        unsigned("4294967295.00000000000000000001"),
        Some(4_294_967_295)
    );
    assert_eq!(signed("1.00000000000000000000001"), Some(1));
    assert_eq!(signed("19999999999999999999.5"), None);
}

/// A value whose every digit is cut and carries is one, and the sign is not
/// applied to it.
#[test]
fn a_carry_past_every_digit_is_one_whatever_the_sign() {
    assert_eq!(signed("0.999999999999999999999"), Some(1));
    assert_eq!(signed("-0.999999999999999999999"), Some(1));
    assert_eq!(unsigned("0.999999999999999999999"), Some(1));
    assert_eq!(unsigned("-0.999999999999999999999"), None);
}

/// The overflow test refuses a step only when it answers less than the value
/// before it, and a multiplication by ten can wrap to more.
#[test]
fn a_multiplication_that_wraps_upward_is_not_seen() {
    assert_eq!(
        unsigned("30000000000000000000"),
        Some(11_553_255_926_290_448_384)
    );
    assert_eq!(
        signed("21000000000000000000"),
        Some(2_553_255_926_290_448_384)
    );
    assert_eq!(
        unsigned("21000000000000000000"),
        Some(2_553_255_926_290_448_384)
    );
    // The control: a wrap to less is seen.
    assert_eq!(unsigned("99999999999999999999"), None);
    assert_eq!(unsigned("18446744073709551616"), None);
}
