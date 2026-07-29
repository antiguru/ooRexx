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
//! `::CONSTANT` for `is_number` and through `::OPTIONS DIGITS` and `TRACE` for
//! `whole_number`.
//!
//! This module owns two delegations and one rule of its own, so the tests are
//! split the same way. The number ACCEPTANCE rule is `rexx_num::Number::parse`'s
//! and the CONVERSION is `Number::whole_value`'s, both tested exhaustively in
//! that crate; what is tested here is that this crate reaches them, that the
//! byte-to-`str` boundary between them behaves, and the `TRACE` setting rule,
//! which is genuinely local.

use super::*;

/// The precision a `TRACE` skip count converts under, which is the one thing this
/// crate decides about the conversion: `number_digits()` at its default. The
/// instruction grammar names it too, at its own call site.
const TRACE_DIGITS: usize = 9;

/// The two delegations are wired, and each answers the question this crate asks
/// of it.
///
/// Deliberately thin. Every boundary case for acceptance lives in `rexx-num`'s
/// `parse.rs` and its twelve differential sets, and every one for the conversion
/// lives in its `whole.rs`. Restating them here would be the same duplication in
/// the tests that the code just stopped having, and it would rot the same way.
#[test]
fn the_number_rules_are_reached_and_not_reimplemented() {
    // Acceptance: a number, whole or not, and something that is not one.
    // Measured through `::constant`: `-.5` is rc 0 and `-5x` is Error 19.916.
    assert!(is_number(b"-.5"));
    assert!(!is_number(b"-5x"));
    // The rule this crate would have got wrong on its own, and did: a blank
    // between the sign and its digits. Measured: `trace "+ 9"` is rc 0.
    assert!(is_number(b"+ 9"));
    assert_eq!(whole_number(b"+ 9", ARGUMENT_DIGITS), Some(9));
    // And the exponent limits, which arrive with the delegation rather than
    // being restated. Measured: `::constant c -1e999999999` is rc 0 and
    // `-1e1000000000` is 19.916.
    assert!(is_number(b"-1e999999999"));
    assert!(!is_number(b"-1e1000000000"));

    // Conversion: whole, not whole, and the rounding rule that separates them.
    // Measured through `trace`: rc 0, 24.1, rc 0.
    assert_eq!(whole_number(b"12", ARGUMENT_DIGITS), Some(12));
    assert_eq!(whole_number(b"9.5", ARGUMENT_DIGITS), None);
    assert_eq!(
        whole_number(b"999999999.4", TRACE_DIGITS),
        Some(999_999_999)
    );
    // The precision is the caller's, which is the one thing this crate chooses.
    assert_eq!(whole_number(b"1e8", TRACE_DIGITS), Some(100_000_000));
    assert_eq!(whole_number(b"1e9", TRACE_DIGITS), None);
    // A number too wide for any precision is still a number, so the two
    // delegations cannot be collapsed into one.
    assert!(is_number(b"1234567890123456789012345678901234567890"));
    assert_eq!(
        whole_number(b"1234567890123456789012345678901234567890", ARGUMENT_DIGITS),
        None
    );
}

/// The one thing between this crate and `rexx-num` that is this crate's own: the
/// operands arrive as bytes and `Number::parse` takes a `&str`.
///
/// A literal may hold a non-UTF-8 byte, and such a literal is not a number.
/// That is right rather than convenient: a symbol cannot hold a non-ASCII byte at
/// all, because `LanguageParser::characterTable` is zero for every byte from 0x80
/// to 0xFF, and a literal that holds one is not a number either.
#[test]
fn a_non_utf8_operand_is_not_a_number() {
    assert!(!is_number(&[b'1', 0xC3]));
    assert_eq!(whole_number(&[b'1', 0xC3], ARGUMENT_DIGITS), None);
    // The control: the same leading byte on its own is a number.
    assert!(is_number(b"1"));
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
