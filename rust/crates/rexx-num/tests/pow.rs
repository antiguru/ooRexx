use rexx_num::{ArithError, Number};

fn n(s: &str) -> Number {
    Number::parse(s).unwrap()
}
fn pow(a: &str, b: &str, d: u64) -> Result<String, u16> {
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

#[test]
fn a_reciprocal_out_of_range_at_working_precision_is_not_an_overflow() {
    // At working precision the reciprocal's last digit sits below the
    // exponent floor; only the final rounding has to be representable.
    // The general division would range-check the intermediate and fail.
    assert_eq!(pow("730361.1e999999992", "-1", 2).unwrap(), "1.4E-999999998");
}

#[test]
fn an_exponent_fits_within_digits_beyond_i32_max() {
    // `digits` is a bare u64 parameter here, not the `Settings`-bounded
    // value the interpreter would ever pass. Narrowing it for the "does the
    // exponent fit within `digits`" check wraps negative above the signed
    // maximum (i32 for the original defect, i64 for a careless u64 port),
    // which used to reject every exponent outright regardless of whether it
    // actually fit. Zero as the base takes the cheap early-out in `pow`, so
    // this stays fast even at these `digits` values.
    assert_eq!(pow("0", "5", 3_000_000_000).unwrap(), "0");
    assert_eq!(pow("0", "7", u64::from(u32::MAX)).unwrap(), "0");
    assert_eq!(pow("0", "7", u64::MAX).unwrap(), "0");
}

#[test]
fn the_reciprocal_is_rounded_exactly_once() {
    // The positive power's last working digits feed an unrounded quotient,
    // which the tail rounds in a single step. Routing the reciprocal
    // through the general division rounds twice and lands on ...547.
    assert_eq!(pow("129720.468", "-23", 7).unwrap(), "2.516546E-118");
}
