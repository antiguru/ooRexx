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

//! `message`, `line`, and how the inserts reach the message.

use rexx_inventory::errors::{E7_1, E8_2, E13_1, E25_901, E36_901, E99_943};

use crate::token::ParseError;
use crate::{ProgramSource, SourceKind};

#[test]
fn a_sub_message_that_needs_no_substitution_is_the_message() {
    // `build/bin/rexxc` on a file holding `nop` then `else nop` prints, verbatim:
    //     2 *-* else nop
    //     Error 8 running <file> line 2:  Unexpected THEN or ELSE.
    //     Error 8.2:  ELSE has no corresponding THEN clause.
    assert_eq!(
        ParseError::new(8, 2, 0).message(),
        b"ELSE has no corresponding THEN clause."
    );
}

#[test]
fn a_second_sub_message_that_needs_no_substitution_is_also_the_message() {
    assert_eq!(
        ParseError::new(10, 1, 0).message(),
        b"END has no corresponding DO, LOOP, or SELECT."
    );
}

#[test]
fn a_token_insert_fills_the_sub_message() {
    // `rexxc` on `::class c junk` prints, verbatim:
    //     1 *-* ::class c junk
    //     Error 25 running <file> line 1:  Invalid subkeyword found.
    //     Error 25.901:  Unknown keyword on ::CLASS directive; found "JUNK".
    assert_eq!(
        ParseError::raise(E25_901, 0, [b"JUNK".to_vec()]).message(),
        br#"Unknown keyword on ::CLASS directive; found "JUNK"."#
    );
}

#[test]
fn a_line_insert_fills_the_sub_message() {
    // `rexxc` on `nop`, blank, `select`, blank, `end` prints, verbatim:
    //     3 *-* select
    //     Error 7 running <file> line 3:  WHEN or OTHERWISE expected.
    //     Error 7.1:  SELECT on line 3 requires WHEN.
    assert_eq!(
        ParseError::raise(E7_1, 0, [b"3".to_vec()]).message(),
        b"SELECT on line 3 requires WHEN."
    );
}

#[test]
fn two_inserts_fill_in_order() {
    // `rexxc` on `r = (a` prints, verbatim:
    //     Error 36.901:  Left parenthesis "(" in position 5 on line 1 requires a
    //     corresponding right parenthesis ")".
    assert_eq!(
        ParseError::raise(E36_901, 0, [b"5".to_vec(), b"1".to_vec()]).message(),
        br#"Left parenthesis "(" in position 5 on line 1 requires a corresponding right parenthesis ")"."#
    );
}

#[test]
fn a_message_raised_with_no_inserts_never_falls_back_to_the_major() {
    // The oracle's `messageSubstitution` leaves a marker it has no value for.
    assert_eq!(
        ParseError::new(25, 901, 0).message(),
        br#"Unknown keyword on ::CLASS directive; found "&1"."#
    );
}

#[test]
fn an_insert_is_not_substituted_again() {
    assert_eq!(
        ParseError::raise(E99_943, 0, [b"&2".to_vec(), b"&1".to_vec()]).message(),
        br#"Missing ::RESOURCE end marker "&2" for resource "&1"."#
    );
}

#[test]
fn an_insert_reaches_the_message_unconverted() {
    // The oracle's 13.1 for a `'80'x` source byte prints that byte raw.
    assert_eq!(
        ParseError::raise(E13_1, 0, [vec![0x80], b"80".to_vec()]).message(),
        b"Incorrect character in program \"\x80\" ('80'X)."
    );
}

#[test]
fn a_message_with_no_inserts_carries_none() {
    let error = ParseError::raise(E8_2, 4, []);
    assert!(error.inserts().is_empty());
    assert_eq!(error, ParseError::new(8, 2, 4));
}

#[test]
fn the_inserts_read_back_in_order() {
    let error = ParseError::raise(E36_901, 0, [b"5".to_vec(), b"1".to_vec()]);
    assert_eq!(error.inserts(), [b"5".to_vec(), b"1".to_vec()]);
    assert_eq!((error.code, error.sub), (36, 901));
}

#[test]
fn a_parse_error_is_one_word() {
    assert_eq!(std::mem::size_of::<ParseError>(), 8);
    assert_eq!(std::mem::size_of::<Result<(), ParseError>>(), 8);
}

/// Three lines of five bytes each plus a terminator, so line 1 is bytes 0-4,
/// its `\n` is byte 5, line 2 is bytes 6-10 with its `\n` at 11, and line 3 is
/// bytes 12-16 with its `\n` at 17.
fn three_lines() -> ProgramSource {
    ProgramSource::new(b"say 1\nsay 2\nsay 3\n".to_vec(), SourceKind::Program)
}

#[test]
fn the_reported_line_is_the_line_the_clause_byte_sits_on() {
    let source = three_lines();
    assert_eq!(ParseError::new(35, 1, 0).line(&source), 1);
    assert_eq!(ParseError::new(35, 1, 6).line(&source), 2);
    assert_eq!(ParseError::new(35, 1, 12).line(&source), 3);
}

#[test]
fn a_byte_on_a_terminator_belongs_to_the_line_that_terminator_ends() {
    // The only assertion in this file that an off-by-one in either direction
    // cannot also satisfy: every byte in the test above sits well inside its
    // line, so `line_of(byte + 1)` answers all three of them correctly.
    let source = three_lines();
    assert_eq!(ParseError::new(35, 1, 5).line(&source), 1);
    assert_eq!(ParseError::new(35, 1, 11).line(&source), 2);
}

#[test]
fn a_byte_past_the_end_clamps_to_the_last_line() {
    // `line_of`'s contract, restated here because a diagnostic must never be the
    // thing that crashes.
    let source = three_lines();
    assert_eq!(ParseError::new(35, 1, 17).line(&source), 3);
    assert_eq!(ParseError::new(35, 1, 999).line(&source), 3);
}

#[test]
fn display_names_the_number_the_sub_number_and_the_message() {
    assert_eq!(
        ParseError::new(8, 2, 0).to_string(),
        "8.2: ELSE has no corresponding THEN clause."
    );
}

#[test]
fn display_does_not_zero_pad_a_three_digit_sub_number() {
    // `25.901` is how the interpreter prints it, even though the table stores the
    // sub as 901 and its own XML spells it 901 inside a `<Subcode>` that pads
    // two-digit ones to three.
    assert_eq!(
        ParseError::raise(E25_901, 0, [b"JUNK".to_vec()]).to_string(),
        r#"25.901: Unknown keyword on ::CLASS directive; found "JUNK"."#
    );
}

#[test]
fn display_decodes_a_non_utf8_insert_lossily() {
    assert_eq!(
        ParseError::raise(E13_1, 0, [vec![0x80], b"80".to_vec()]).to_string(),
        "13.1: Incorrect character in program \"\u{fffd}\" ('80'X)."
    );
}
