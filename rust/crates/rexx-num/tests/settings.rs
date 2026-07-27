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
    assert_eq!(s.set_digits_str("0"), Err(SettingsError::NotWholeNumber));
    assert_eq!(s.set_digits_str("-1"), Err(SettingsError::NotWholeNumber));
    assert_eq!(s.set_digits_str("1.5"), Err(SettingsError::NotWholeNumber));
    assert_eq!(s.set_digits_str("abc"), Err(SettingsError::NotWholeNumber));
    assert!(s.set_digits_str("1").is_ok());
    // any expression yielding a whole number is fine, including 1e3
    assert!(s.set_digits_str("1e3").is_ok());
    assert_eq!(s.digits(), 1000);
}

#[test]
fn fuzz_must_be_non_negative() {
    let mut s = Settings::default();
    assert_eq!(s.set_fuzz_str("-1"), Err(SettingsError::NotWholeNumber));
    assert!(s.set_fuzz_str("0").is_ok());
}

#[test]
fn fuzz_must_stay_below_digits_whichever_of_the_two_is_being_set() {
    let mut s = Settings::default();
    s.set_digits_str("5").unwrap();
    assert_eq!(s.set_fuzz_str("5"), Err(SettingsError::FuzzNotBelowDigits));
    assert!(s.set_fuzz_str("4").is_ok());
    // and lowering digits to meet fuzz fails the same way
    assert_eq!(s.set_digits_str("4"), Err(SettingsError::FuzzNotBelowDigits));
}

#[test]
fn form_accepts_only_the_two_spellings_case_insensitively() {
    let mut s = Settings::default();
    assert!(s.set_form_str("ENGINEERING").is_ok());
    assert_eq!(s.form(), Form::Engineering);
    assert!(s.set_form_str("scientific").is_ok());
    assert_eq!(s.form(), Form::Scientific);
    assert_eq!(s.set_form_str("BOGUS"), Err(SettingsError::InvalidForm));
}

#[test]
fn each_error_carries_the_interpreters_number() {
    assert_eq!(SettingsError::NotWholeNumber.code(), 26);
    assert_eq!(SettingsError::FuzzNotBelowDigits.code(), 33);
    assert_eq!(SettingsError::InvalidForm.code(), 25);
}
