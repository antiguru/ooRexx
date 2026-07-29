/*----------------------------------------------------------------------------*/
/*                                                                            */
/* Copyright (c) 2026 Rexx Language Association. All rights reserved.          */
/*                                                                            */
/* This program and the accompanying materials are made available under       */
/* the terms of the Common Public License v1.0 which accompanies this         */
/* distribution. A copy is also available at the following address:           */
/* https://www.oorexx.org/license.html                                        */
/*                                                                            */
/*----------------------------------------------------------------------------*/

//! Every case here is a `build/bin/rexxc` measurement, reached through
//! `::OPTIONS DIGITS` for the eighteen-digit conversion, through `TRACE` for
//! the nine-digit one, and through `::CONSTANT` for `is_number`.

use super::*;

/// `Numerics::ARGUMENT_DIGITS` on a 64-bit build, which is what
/// `::OPTIONS DIGITS` and `::OPTIONS FUZZ` convert under.
const ARGUMENT_DIGITS: usize = 18;

/// The `TRACE` skip count's precision, `number_digits()` at its default.
const TRACE_DIGITS: usize = 9;

#[test]
fn a_whole_number_may_be_written_any_way_rexx_writes_one() {
    // Measured through `::options digits`, all rc 0.
    assert_eq!(whole_number(b"12", ARGUMENT_DIGITS), Some(12));
    assert_eq!(whole_number(b"1e2", ARGUMENT_DIGITS), Some(100));
    assert_eq!(whole_number(b"0009", ARGUMENT_DIGITS), Some(9));
    assert_eq!(whole_number(b"9.0", ARGUMENT_DIGITS), Some(9));
    assert_eq!(whole_number(b"9.", ARGUMENT_DIGITS), Some(9));
    assert_eq!(whole_number(b"+9", ARGUMENT_DIGITS), Some(9));
}

#[test]
fn a_number_that_is_not_whole_is_not_a_whole_number() {
    // Measured through `::options digits`, all Error 26.5.
    assert_eq!(whole_number(b"9.5", ARGUMENT_DIGITS), None);
    assert_eq!(whole_number(b"1e-2", ARGUMENT_DIGITS), None);
    assert_eq!(whole_number(b".9", ARGUMENT_DIGITS), None);
    assert_eq!(whole_number(b"", ARGUMENT_DIGITS), None);
    assert_eq!(whole_number(b"abc", ARGUMENT_DIGITS), None);
}

/// The blank rule is the one this module fixed: 3.6's version did not strip
/// them, so `trace " 9 "` raised 24.1 where the oracle accepts it as a skip
/// count. Both directions are asserted, because stripping everywhere would be
/// just as wrong as stripping nowhere.
#[test]
fn blanks_may_surround_a_number_but_not_sit_inside_one() {
    // Measured: `::options digits " 9 "` and the tab-padded spelling are rc 0.
    assert_eq!(whole_number(b" 9 ", ARGUMENT_DIGITS), Some(9));
    assert_eq!(whole_number(b"\t9\t", ARGUMENT_DIGITS), Some(9));
    assert_eq!(whole_number(b" 9", ARGUMENT_DIGITS), Some(9));
    assert_eq!(whole_number(b"9 ", ARGUMENT_DIGITS), Some(9));
    // Measured: `"- 9"`, `"9 5"` and `"1 e2"` are all Error 26.5.
    assert_eq!(whole_number(b"- 9", ARGUMENT_DIGITS), None);
    assert_eq!(whole_number(b"9 5", ARGUMENT_DIGITS), None);
    assert_eq!(whole_number(b"1 e2", ARGUMENT_DIGITS), None);
}

/// The precision is the caller's, and the two callers really do differ.
#[test]
fn the_digit_limit_is_the_precision_the_caller_passed() {
    // Measured: `::options digits 123456789012345678` is rc 0 at eighteen
    // digits and `1234567890123456789` is Error 26.5 at nineteen.
    assert_eq!(
        whole_number(b"123456789012345678", ARGUMENT_DIGITS),
        Some(123_456_789_012_345_678)
    );
    assert_eq!(whole_number(b"1234567890123456789", ARGUMENT_DIGITS), None);
    // Measured: `trace 123456789` is rc 0 and `trace 1234567890` is Error 24.1.
    assert_eq!(whole_number(b"123456789", TRACE_DIGITS), Some(123_456_789));
    assert_eq!(whole_number(b"1234567890", TRACE_DIGITS), None);
    // Measured: `::options digits 1E18` is Error 26.5, because the value needs
    // nineteen digits even though its text holds four.
    assert_eq!(whole_number(b"1E18", ARGUMENT_DIGITS), None);
}

#[test]
fn zero_is_zero_however_it_is_spelled() {
    assert_eq!(whole_number(b"0", ARGUMENT_DIGITS), Some(0));
    assert_eq!(whole_number(b"0.000", ARGUMENT_DIGITS), Some(0));
    assert_eq!(whole_number(b"-0", ARGUMENT_DIGITS), Some(0));
    assert_eq!(whole_number(b"0e9999", ARGUMENT_DIGITS), Some(0));
}

#[test]
fn a_negative_whole_number_keeps_its_sign() {
    assert_eq!(whole_number(b"-9", TRACE_DIGITS), Some(-9));
    assert_eq!(whole_number(b"-1e2", TRACE_DIGITS), Some(-100));
}

#[test]
fn a_trace_setting_is_any_number_of_question_marks_and_one_letter() {
    // Measured: all rc 0.
    assert!(check_trace_setting(b"").is_ok());
    assert!(check_trace_setting(b"r").is_ok());
    assert!(check_trace_setting(b"?r").is_ok());
    assert!(check_trace_setting(b"??r").is_ok());
    assert!(check_trace_setting(b"results").is_ok());
    assert!(check_trace_setting(b"?").is_ok());
    // Measured: `trace zzz` is Error 24.1.
    assert!(check_trace_setting(b"zzz").is_err());
    assert!(check_trace_setting(b"?z").is_err());
}
