use rexx_num::Number;

/// Every pair here was produced by the oracle: `say "[" || v || "] -> [" || (v + 0) || "]"`
/// under `NUMERIC DIGITS 9`. See rust/corpus/num/canonical_form.rex.
const CANONICAL: &[(&str, &str)] = &[
    ("1", "1"),
    ("1.0", "1.0"),
    ("01", "1"),
    ("1.", "1"),
    (".5", "0.5"),
    ("+5", "5"),
    ("-0", "0"),
    ("0.0", "0"),
    ("0.00", "0"),
    ("1e5", "100000"),
    ("1E+5", "100000"),
    ("1e-5", "0.00001"),
    (" 7 ", "7"),
    ("1.50", "1.50"),
    ("000.500", "0.500"),
    ("1e0", "1"),
    ("-1.50", "-1.50"),
    ("00.00", "0"),
    ("+0.0", "0"),
    ("12.3400", "12.3400"),
];

#[test]
fn parsing_then_displaying_reproduces_the_oracles_canonical_form() {
    for (input, expected) in CANONICAL {
        let n = Number::parse(input).unwrap_or_else(|| panic!("{input:?} should parse"));
        assert_eq!(&n.format(9), expected, "input {input:?}");
    }
}

#[test]
fn trailing_zeros_after_a_decimal_point_are_significant() {
    // 1.50 and 1.5 are equal in value but not in form, and Rexx shows the
    // difference. This is the property a naive normalisation would destroy.
    assert_eq!(Number::parse("1.50").unwrap().format(9), "1.50");
    assert_eq!(Number::parse("1.5").unwrap().format(9), "1.5");
}

#[test]
fn every_spelling_of_zero_collapses_to_a_single_form() {
    for z in ["0", "-0", "+0", "0.0", "00.00", "0e10", "-0.000"] {
        assert_eq!(Number::parse(z).unwrap().format(9), "0", "input {z:?}");
    }
}

#[test]
fn exponential_form_is_used_past_the_measured_thresholds() {
    // positive: adjusted exponent >= DIGITS
    assert_eq!(Number::parse("1e8").unwrap().format(9), "100000000");
    assert_eq!(Number::parse("1e9").unwrap().format(9), "1E+9");
    // negative: adjusted exponent <= -(2 * DIGITS + 1)
    assert_eq!(Number::parse("1e-18").unwrap().format(9), "0.000000000000000001");
    assert_eq!(Number::parse("1e-19").unwrap().format(9), "1E-19");
    // and the thresholds move with DIGITS
    assert_eq!(Number::parse("1e2").unwrap().format(3), "100");
    assert_eq!(Number::parse("1e3").unwrap().format(3), "1E+3");
    assert_eq!(Number::parse("1e-6").unwrap().format(3), "0.000001");
    assert_eq!(Number::parse("1e-7").unwrap().format(3), "1E-7");
}

#[test]
fn a_multi_digit_mantissa_keeps_its_point_in_exponential_form() {
    assert_eq!(Number::parse("1234567890").unwrap().format(9), "1.23456789E+9");
    assert_eq!(Number::parse("-1234567890").unwrap().format(9), "-1.23456789E+9");
}

#[test]
fn things_that_are_not_numbers_are_rejected() {
    for bad in ["abc", "", "   ", "1e", "1.2.3", "--1", "1 2", "+", "-", "e5", "1e+", "0x1f"] {
        assert!(Number::parse(bad).is_none(), "{bad:?} should not parse");
    }
}

#[test]
fn format_does_not_overflow_at_extreme_digits() {
    // `digits` is a bare u64 here, not the `Settings`-bounded value the
    // interpreter would ever pass, so `format` itself has to stay well
    // defined for the whole range. `2 * digits` narrowed to i32 overflows
    // above 1073741823 and, before this was fixed, panicked in debug and
    // silently picked the wrong display form in release; the u64 widening
    // adds the same hazard at 2^63, so the top of the new range is pinned
    // here too.
    assert_eq!(Number::parse("1e-30").unwrap().format(2147483647), "0.000000000000000000000000000001");
    assert_eq!(
        Number::parse("1e-30").unwrap().format(u64::from(u32::MAX)),
        "0.000000000000000000000000000001"
    );
    assert_eq!(Number::parse("123456789").unwrap().format(u64::from(u32::MAX)), "123456789");
    assert_eq!(Number::parse("1e-30").unwrap().format(u64::MAX), "0.000000000000000000000000000001");
    assert_eq!(Number::parse("123456789").unwrap().format(u64::MAX), "123456789");
}

#[test]
fn the_negative_threshold_is_on_the_raw_exponent_not_the_adjusted_one() {
    // Same value, two spellings, two different display forms. 1e-18 has raw
    // exponent -18 and prints plain; 10e-19 has raw exponent -19 and prints
    // exponential. A rule written in terms of the adjusted exponent gets both
    // of these wrong in the same direction, and a probe using only single
    // digit mantissas cannot tell the two rules apart.
    assert_eq!(Number::parse("1e-18").unwrap().format(9), "0.000000000000000001");
    assert_eq!(Number::parse("10e-19").unwrap().format(9), "1.0E-18");
    assert_eq!(Number::parse("1.0e-18").unwrap().format(9), "1.0E-18");
    assert_eq!(Number::parse("123e-19").unwrap().format(9), "1.23E-17");
    assert_eq!(Number::parse("1000e-19").unwrap().format(9), "1.000E-16");
}
