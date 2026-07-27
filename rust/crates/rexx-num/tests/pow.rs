use rexx_num::{ArithError, Number};

fn n(s: &str) -> Number {
    Number::parse(s).unwrap()
}
fn pow(a: &str, b: &str, d: u32) -> Result<String, u16> {
    n(a).pow(&n(b), d).map(|r| r.format(d)).map_err(ArithError::code)
}

#[test]
fn ordinary_powers() {
    assert_eq!(pow("2", "10", 9).unwrap(), "1024");
    assert_eq!(pow("2", "0", 9).unwrap(), "1");
    assert_eq!(pow("2", "-3", 9).unwrap(), "0.125");
    assert_eq!(pow("-2", "3", 9).unwrap(), "-8");
    assert_eq!(pow("-2", "2", 9).unwrap(), "4");
    assert_eq!(pow("10", "20", 9).unwrap(), "1E+20");
}

#[test]
fn zero_to_a_power_has_its_own_rules() {
    // Rexx defines 0**0 as 1, though mathematically it is undefined.
    assert_eq!(pow("0", "0", 9).unwrap(), "1");
    assert_eq!(pow("0", "5", 9).unwrap(), "0");
    // Zero to a negative power underflows rather than being infinite.
    assert_eq!(pow("0", "-1", 9), Err(42));
}

#[test]
fn the_exponent_is_rounded_to_digits_before_being_required_to_be_whole() {
    // At DIGITS 9, 2.5 is not whole -- error 26.
    assert_eq!(pow("2", "2.5", 9), Err(26));
    // At DIGITS 1 it rounds to 3, so this is 2**3.
    assert_eq!(pow("2", "2.5", 1).unwrap(), "8");
    assert_eq!(pow("3", "2.5", 1).unwrap(), "3E+1");
}

#[test]
fn an_exponent_too_wide_for_digits_is_error_26_not_an_overflow() {
    // 1e10 needs eleven digits, so at DIGITS 9 it is not a usable whole
    // number at all ...
    assert_eq!(pow("2", "1e10", 9), Err(26));
    // ... while at DIGITS 15 it is, and the failure moves to the result.
    assert_eq!(pow("2", "1e10", 15), Err(42));
    // 999999999 fits in nine digits, and the result is representable.
    assert_eq!(pow("2", "999999999", 9).unwrap(), "2.306488E+301029995");
    assert_eq!(pow("2", "1000000000", 9), Err(26));
}

#[test]
fn the_base_is_truncated_to_digits_plus_one_before_the_computation() {
    // 123456789 truncates to 1.2e8 at DIGITS 1, so this is 1.44e16 -> 1E+16.
    assert_eq!(pow("123456789", "2", 1).unwrap(), "1E+16");
}
