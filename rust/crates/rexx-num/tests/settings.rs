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
    assert!(matches!(s.set_digits_str("0"), Err(SettingsError::NotWholeNumber(_))));
    assert!(matches!(s.set_digits_str("-1"), Err(SettingsError::NotWholeNumber(_))));
    assert!(matches!(s.set_digits_str("1.5"), Err(SettingsError::NotWholeNumber(_))));
    assert!(matches!(s.set_digits_str("abc"), Err(SettingsError::NotWholeNumber(_))));
    assert!(s.set_digits_str("1").is_ok());
    // any expression yielding a whole number is fine, including 1e3
    assert!(s.set_digits_str("1e3").is_ok());
    assert_eq!(s.digits(), 1000);
}

#[test]
fn digits_is_capped_at_max_exponent() {
    let mut s = Settings::default();
    assert!(s.set_digits_str("999999999").is_ok());
    assert_eq!(s.digits(), 999_999_999);

    let mut s = Settings::default();
    assert!(matches!(s.set_digits_str("1000000000"), Err(SettingsError::NotWholeNumber(_))));
    // Values that would not even fit a u32, let alone the cap, fail the
    // same way rather than a different one.
    assert!(matches!(s.set_digits_str("2147483647"), Err(SettingsError::NotWholeNumber(_))));
    assert!(matches!(s.set_digits_str("4294967296"), Err(SettingsError::NotWholeNumber(_))));
}

#[test]
fn fuzz_must_be_non_negative() {
    let mut s = Settings::default();
    assert!(matches!(s.set_fuzz_str("-1"), Err(SettingsError::NotWholeNumber(_))));
    assert!(s.set_fuzz_str("0").is_ok());
}

#[test]
fn fuzz_must_stay_below_digits_whichever_of_the_two_is_being_set() {
    let mut s = Settings::default();
    s.set_digits_str("5").unwrap();
    assert!(matches!(s.set_fuzz_str("5"), Err(SettingsError::FuzzNotBelowDigits(_))));
    assert!(s.set_fuzz_str("4").is_ok());
    // and lowering digits to meet fuzz fails the same way
    assert!(matches!(s.set_digits_str("4"), Err(SettingsError::FuzzNotBelowDigits(_))));
}

#[test]
fn form_accepts_only_the_two_spellings_case_insensitively() {
    let mut s = Settings::default();
    assert!(s.set_form_str("ENGINEERING").is_ok());
    assert_eq!(s.form(), Form::Engineering);
    assert!(s.set_form_str("scientific").is_ok());
    assert_eq!(s.form(), Form::Scientific);
    assert!(matches!(s.set_form_str("BOGUS"), Err(SettingsError::InvalidForm(_))));
}

#[test]
fn each_error_carries_the_interpreters_number() {
    assert_eq!(Settings::default().set_digits_str("abc").unwrap_err().code(), 26);
    assert_eq!(Settings::default().set_fuzz_str("-1").unwrap_err().code(), 26);
    let mut s = Settings::default();
    s.set_digits_str("5").unwrap();
    assert_eq!(s.set_fuzz_str("5").unwrap_err().code(), 33);
    assert_eq!(Settings::default().set_form_str("BOGUS").unwrap_err().code(), 25);
}

// ---- message text, each confirmed against `build/bin/rexx` ----------------

#[test]
fn digits_not_whole_number_message_is_26_005() {
    let err = Settings::default().set_digits_str("abc").unwrap_err();
    assert_eq!(err.message(), "DIGITS value must be a positive whole number; found \"abc\".");
}

#[test]
fn fuzz_not_whole_number_message_is_26_006() {
    let err = Settings::default().set_fuzz_str("-1").unwrap_err();
    assert_eq!(err.message(), "FUZZ value must be zero or a positive whole number; found \"-1\".");
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
