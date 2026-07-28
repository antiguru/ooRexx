use rexx_num::{Form, Settings, SettingsError};

#[test]
fn the_defaults_are_the_ones_the_interpreter_starts_with() {
    let s = Settings::default();
    assert_eq!(s.digits(), 9);
    assert_eq!(s.fuzz(), 0);
    assert_eq!(s.form(), Form::Scientific);
}

#[test]
fn digits_must_be_a_positive_whole_number() {
    let mut s = Settings::default();
    assert!(matches!(
        s.set_digits_str("0"),
        Err(SettingsError::DigitsNotWhole { .. })
    ));
    assert!(matches!(
        s.set_digits_str("-1"),
        Err(SettingsError::DigitsNotWhole { .. })
    ));
    assert!(matches!(
        s.set_digits_str("1.5"),
        Err(SettingsError::DigitsNotWhole { .. })
    ));
    assert!(matches!(
        s.set_digits_str("abc"),
        Err(SettingsError::DigitsNotWhole { .. })
    ));
    assert!(s.set_digits_str("1").is_ok());
    // Any spelling of a whole number that fits the DIGITS in force works,
    // including exponential ones -- but each success moves the boundary the
    // *next* value is judged against: from the 1 just set, 1e3 is four
    // positions too wide, while from the default 9 it is fine.
    assert!(matches!(
        s.set_digits_str("1e3"),
        Err(SettingsError::DigitsNotWhole { .. })
    ));
    let mut s = Settings::default();
    assert!(s.set_digits_str("1e3").is_ok());
    assert_eq!(s.digits(), 1000);
}

#[test]
fn a_new_digits_value_must_fit_the_digits_currently_in_force() {
    // The rule is relative, not a fixed cap -- indistinguishable from one
    // when probed only from the default DIGITS 9, which is exactly how the
    // fixed-cap version survived review. Every row here was confirmed
    // against `build/bin/rexx` from a *non-default* starting DIGITS.
    let mut s = Settings::default();
    s.set_digits_str("3").unwrap();
    assert!(matches!(
        s.set_digits_str("1000"),
        Err(SettingsError::DigitsNotWhole { .. })
    ));
    assert!(matches!(
        s.set_digits_str("12345"),
        Err(SettingsError::DigitsNotWhole { .. })
    ));
    assert!(s.set_digits_str("999").is_ok());

    // Positions count, not significant digits: 10 is 1E1, one mantissa
    // digit, and still two positions wide.
    let mut s = Settings::default();
    s.set_digits_str("1").unwrap();
    assert!(matches!(
        s.set_digits_str("10"),
        Err(SettingsError::DigitsNotWhole { .. })
    ));
    let mut s = Settings::default();
    s.set_digits_str("2").unwrap();
    assert!(matches!(
        s.set_digits_str("1e2"),
        Err(SettingsError::DigitsNotWhole { .. })
    ));

    // At DIGITS 9 the boundary happens to sit at 999999999 -- fresh default
    // for each, because a successful set moves the boundary itself.
    let mut s = Settings::default();
    assert!(s.set_digits_str("999999999").is_ok());
    let mut s = Settings::default();
    assert!(matches!(
        s.set_digits_str("1000000000"),
        Err(SettingsError::DigitsNotWhole { .. })
    ));

    // ... but from DIGITS 10 the very same value is legal, which is what
    // separates the real rule from the fixed cap.
    let mut s = Settings::default();
    s.set_digits_str("10").unwrap();
    assert!(s.set_digits_str("1000000000").is_ok());
    assert_eq!(s.digits(), 1_000_000_000);
}

#[test]
fn digits_ranges_over_the_full_u64_width_up_to_max_wholenumber() {
    // From DIGITS 10, a value past u32 is legal -- the stored setting has to
    // be wide enough to hold it (this is the row that forced the u64
    // widening).
    let mut s = Settings::default();
    s.set_digits_str("10").unwrap();
    assert!(s.set_digits_str("4294967296").is_ok());
    assert_eq!(s.digits(), 4_294_967_296);

    // The absolute ceiling is `Numerics::MAX_WHOLENUMBER`, 10^18 - 1:
    // reachable exactly from DIGITS 18 ...
    let mut s = Settings::default();
    s.set_digits_str("18").unwrap();
    assert!(s.set_digits_str("999999999999999999").is_ok());
    assert_eq!(s.digits(), 999_999_999_999_999_999);

    // ... and 10^18 stays out even from a DIGITS with room for 30 positions.
    let mut s = Settings::default();
    s.set_digits_str("30").unwrap();
    assert!(matches!(
        s.set_digits_str("1000000000000000000"),
        Err(SettingsError::DigitsNotWhole { .. })
    ));
    assert!(s.set_digits_str("999999999999999999").is_ok());
}

#[test]
fn the_candidate_is_rounded_at_the_digits_in_force_before_the_check() {
    // The conversion is `requestUnsignedNumber` at the current DIGITS, so a
    // fractional spelling that rounds clean is accepted -- all confirmed
    // against `build/bin/rexx`.
    let mut s = Settings::default();
    s.set_digits_str("4").unwrap();
    assert!(s.set_digits_str("999.9999").is_ok()); // carry through the nines
    assert_eq!(s.digits(), 1000);

    // The same carry one position wider fails: 999.6 rounds to 1000, four
    // positions at DIGITS 3.
    let mut s = Settings::default();
    s.set_digits_str("3").unwrap();
    assert!(matches!(
        s.set_digits_str("999.6"),
        Err(SettingsError::DigitsNotWhole { .. })
    ));
    assert!(s.set_digits_str("999.4").is_ok()); // rounds down to 999
    assert_eq!(s.digits(), 999);

    // The carry can ripple up from entirely below the decimal point.
    let mut s = Settings::default();
    assert!(s.set_digits_str("0.99999999995").is_ok());
    assert_eq!(s.digits(), 1);
    // A fraction that does not reduce to zero stays an error, rounded or not.
    let mut s = Settings::default();
    assert!(matches!(
        s.set_digits_str("10.5"),
        Err(SettingsError::DigitsNotWhole { .. })
    ));
    assert!(s.set_digits_str("10.0").is_ok());
}

#[test]
fn fuzz_conversion_uses_the_digits_in_force_and_reports_26_006() {
    // `numeric fuzz 12345` at DIGITS 3 is 26.006 -- the conversion fails
    // before the fuzz-below-digits comparison is reached, so 33.001 (what a
    // conversion that wrongly succeeds would report) never fires. Confirmed
    // against `build/bin/rexx`.
    let mut s = Settings::default();
    s.set_digits_str("3").unwrap();
    assert!(matches!(
        s.set_fuzz_str("12345"),
        Err(SettingsError::FuzzNotWhole { .. })
    ));
    assert!(matches!(
        s.set_fuzz_str("1000"),
        Err(SettingsError::FuzzNotWhole { .. })
    ));
    // A value that *does* convert but is not below DIGITS is the 33.001.
    assert!(matches!(
        s.set_fuzz_str("12"),
        Err(SettingsError::FuzzNotBelowDigits { .. })
    ));

    // FUZZ spans the same widened range: digits - 1 at the very top.
    let mut s = Settings::default();
    s.set_digits_str("18").unwrap();
    s.set_digits_str("999999999999999999").unwrap();
    assert!(s.set_fuzz_str("999999999999999998").is_ok());
    assert_eq!(s.fuzz(), 999_999_999_999_999_998);
}

#[test]
fn fuzz_must_be_non_negative() {
    let mut s = Settings::default();
    assert!(matches!(
        s.set_fuzz_str("-1"),
        Err(SettingsError::FuzzNotWhole { .. })
    ));
    assert!(s.set_fuzz_str("0").is_ok());
}

#[test]
fn fuzz_must_stay_below_digits_whichever_of_the_two_is_being_set() {
    let mut s = Settings::default();
    s.set_digits_str("5").unwrap();
    assert!(matches!(
        s.set_fuzz_str("5"),
        Err(SettingsError::FuzzNotBelowDigits { .. })
    ));
    assert!(s.set_fuzz_str("4").is_ok());
    // and lowering digits to meet fuzz fails the same way
    assert!(matches!(
        s.set_digits_str("4"),
        Err(SettingsError::FuzzNotBelowDigits { .. })
    ));
}

#[test]
fn form_accepts_exactly_the_two_uppercase_spellings_and_nothing_else() {
    // The runtime VALUE path -- what a `&str` setter models -- is
    // case-sensitive, does not trim, and takes no abbreviations; only the
    // source-keyword form looks case-insensitive, and only because the
    // tokenizer uppercases it first. All confirmed against `build/bin/rexx`
    // with `numeric form value '...'`.
    let mut s = Settings::default();
    assert!(s.set_form_str("ENGINEERING").is_ok());
    assert_eq!(s.form(), Form::Engineering);
    assert!(s.set_form_str("SCIENTIFIC").is_ok());
    assert_eq!(s.form(), Form::Scientific);
    for rejected in [
        "scientific",
        "engineering",
        "Engineering",
        "ENG",
        " ENGINEERING",
        "BOGUS",
    ] {
        assert!(
            matches!(
                s.set_form_str(rejected),
                Err(SettingsError::InvalidForm { .. })
            ),
            "{rejected:?} must be error 25.011"
        );
    }
    // The rejections above must not have moved the setting.
    assert_eq!(s.form(), Form::Scientific);
}

#[test]
fn each_error_carries_the_interpreters_number() {
    assert_eq!(
        Settings::default()
            .set_digits_str("abc")
            .unwrap_err()
            .code(),
        26
    );
    assert_eq!(
        Settings::default().set_fuzz_str("-1").unwrap_err().code(),
        26
    );
    let mut s = Settings::default();
    s.set_digits_str("5").unwrap();
    assert_eq!(s.set_fuzz_str("5").unwrap_err().code(), 33);
    assert_eq!(
        Settings::default()
            .set_form_str("BOGUS")
            .unwrap_err()
            .code(),
        25
    );
}

// ---- message text, each confirmed against `build/bin/rexx` ----------------

#[test]
fn digits_not_whole_number_message_is_26_005() {
    let err = Settings::default().set_digits_str("abc").unwrap_err();
    assert_eq!(
        err.message(),
        "DIGITS value must be a positive whole number; found \"abc\"."
    );
}

#[test]
fn fuzz_not_whole_number_message_is_26_006() {
    let err = Settings::default().set_fuzz_str("-1").unwrap_err();
    assert_eq!(
        err.message(),
        "FUZZ value must be zero or a positive whole number; found \"-1\"."
    );
}

#[test]
fn invalid_form_message_is_25_011() {
    let err = Settings::default().set_form_str("bogus form").unwrap_err();
    assert_eq!(
        err.message(),
        "NUMERIC FORM must be followed by one of the keywords SCIENTIFIC or ENGINEERING; found \"bogus form\"."
    );
}

#[test]
fn fuzz_not_below_digits_message_substitutes_the_pending_pair_not_the_stored_one() {
    // Setting FUZZ too high: DIGITS is unchanged (still 5), FUZZ shows the
    // rejected candidate (10) -- confirmed with `NUMERIC DIGITS 5` then
    // `NUMERIC FUZZ 10`.
    let mut s = Settings::default();
    s.set_digits_str("5").unwrap();
    let err = s.set_fuzz_str("10").unwrap_err();
    assert_eq!(
        err.message(),
        "Value of NUMERIC DIGITS (\"5\") must exceed value of NUMERIC FUZZ (\"10\")."
    );

    // Setting DIGITS too low: FUZZ is unchanged (still 5), DIGITS shows the
    // rejected candidate (3) -- confirmed with `NUMERIC FUZZ 5` then
    // `NUMERIC DIGITS 3`.
    let mut s = Settings::default();
    s.set_fuzz_str("5").unwrap();
    let err = s.set_digits_str("3").unwrap_err();
    assert_eq!(
        err.message(),
        "Value of NUMERIC DIGITS (\"3\") must exceed value of NUMERIC FUZZ (\"5\")."
    );
}

// ---- additional(): the raw substitution values, in interpreter order ------

#[test]
fn additional_carries_the_raw_found_text_for_the_not_whole_errors() {
    assert_eq!(
        Settings::default()
            .set_digits_str("abc")
            .unwrap_err()
            .additional(),
        vec!["abc"]
    );
    assert_eq!(
        Settings::default()
            .set_fuzz_str("-1")
            .unwrap_err()
            .additional(),
        vec!["-1"]
    );
    assert_eq!(
        Settings::default()
            .set_form_str("bogus form")
            .unwrap_err()
            .additional(),
        vec!["bogus form"]
    );
}

#[test]
fn additional_carries_the_pending_digits_fuzz_pair_as_two_values() {
    let mut s = Settings::default();
    s.set_digits_str("5").unwrap();
    assert_eq!(
        s.set_fuzz_str("10").unwrap_err().additional(),
        vec!["5", "10"]
    );
}

#[test]
fn additional_and_message_agree_on_every_placeholder() {
    // additional()'s values, substituted into message()'s own text, must
    // reproduce message() exactly -- the two are not allowed to drift.
    let err = Settings::default().set_digits_str("abc").unwrap_err();
    let subs = err.additional();
    assert!(err.message().contains(&subs[0]));
}
