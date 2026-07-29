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

//! `message`, `line` and the placeholder rule that decides between the two
//! table rows an error has.
//!
//! Every message text asserted here was printed by `build/bin/rexxc`, not read
//! out of the XML: the point of the table being generated is that the text comes
//! from the C++ tree, so a test that quoted the XML back at itself would pass
//! against a table that renders it wrongly.

use crate::token::ParseError;
use crate::{ProgramSource, SourceKind};

use super::has_placeholder;

#[test]
fn a_sub_message_that_needs_no_substitution_is_the_message() {
    // `build/bin/rexxc` on a file holding `nop` then `else nop` prints, verbatim:
    //     2 *-* else nop
    //     Error 8 running <file> line 2:  Unexpected THEN or ELSE.
    //     Error 8.2:  ELSE has no corresponding THEN clause.
    // so the sub-message is the specific one and needs nothing filled in.
    assert_eq!(
        ParseError::new(8, 2, 0).message(),
        "ELSE has no corresponding THEN clause."
    );
}

#[test]
fn a_second_sub_message_that_needs_no_substitution_is_also_the_message() {
    // A separate test rather than a second assertion above, because a shared
    // `#[test]` hid a wrong expectation twice in this task: the first assertion
    // fails, the run stops, and the second is never evaluated.
    //
    // `rexxc` on `nop` then a blank line then `end` prints, verbatim:
    //     3 *-* end
    //     Error 10 running <file> line 3:  Unexpected or unmatched END.
    //     Error 10.1:  END has no corresponding DO, LOOP, or SELECT.
    assert_eq!(
        ParseError::new(10, 1, 0).message(),
        "END has no corresponding DO, LOOP, or SELECT."
    );
}

#[test]
fn a_sub_message_that_would_substitute_a_token_falls_back_to_the_majors_text() {
    // `rexxc` on `::class c junk` prints, verbatim:
    //     1 *-* ::class c junk
    //     Error 25 running <file> line 1:  Invalid subkeyword found.
    //     Error 25.901:  Unknown keyword on ::CLASS directive; found "JUNK".
    // The sub-message's `&1` is the offending token, which this phase does not
    // carry, so the major's own line is what comes out -- and it is byte for byte
    // what the oracle's first line says.
    assert_eq!(
        ParseError::new(25, 901, 0).message(),
        "Invalid subkeyword found."
    );
}

#[test]
fn a_sub_message_that_would_substitute_a_line_falls_back_to_the_majors_text() {
    // The open-construct-line family. `rexxc` on `nop`, blank, `select`, blank,
    // `end` prints, verbatim:
    //     3 *-* select
    //     Error 7 running <file> line 3:  WHEN or OTHERWISE expected.
    //     Error 7.1:  SELECT on line 3 requires WHEN.
    assert_eq!(
        ParseError::new(7, 1, 0).message(),
        "WHEN or OTHERWISE expected."
    );
}

#[test]
fn the_one_substitution_this_phase_will_not_produce_falls_back_too() {
    // 36.901's `&1` is a byte offset within a line, which the brief rules out.
    // `rexxc` on `r = (a` prints, verbatim:
    //     1 *-* r = (a
    //     Error 36 running <file> line 1:  Unmatched "(" or "[" in expression.
    //     Error 36.901:  Left parenthesis "(" in position 5 on line 1 requires a
    //     corresponding right parenthesis ")".
    assert_eq!(
        ParseError::new(36, 901, 0).message(),
        r#"Unmatched "(" or "[" in expression."#
    );
}

#[test]
fn no_message_the_parser_can_produce_holds_a_placeholder() {
    // The property, not an instance of it: whatever branch `message` takes, an
    // `&1` must never reach a user. Checked over every row of the generated
    // table, so a table update that moves a substitution into a major's own
    // text fails here rather than in whatever prints it.
    for message in rexx_inventory::errors::MESSAGES {
        if MAJORS_THAT_SUBSTITUTE.contains(&message.major) {
            continue;
        }
        let error = ParseError::new(message.major, message.sub, 0);
        // A major with no sub-0 row would panic in `message`. None of the majors
        // in the table lacks one, which is asserted separately below.
        assert!(
            !has_placeholder(&error.message()),
            "{}.{:03} renders {:?}",
            message.major,
            message.sub,
            error.message()
        );
    }
}

/// The majors whose own sub-0 text substitutes, so the fallback cannot make
/// them placeholder-free. There is exactly one, and it is not a translation
/// error, so nothing this crate raises reaches the exception. The test below is
/// what keeps that true.
const MAJORS_THAT_SUBSTITUTE: &[u16] = &[101];

#[test]
fn only_a_runtime_major_has_a_substitution_in_its_own_text() {
    let found: Vec<u16> = rexx_inventory::errors::MESSAGES
        .iter()
        .filter(|m| m.sub == 0 && has_placeholder(m.text))
        .map(|m| m.major)
        .collect();
    // 101.000 is `Error &1 running &2, line &3:.`, a wrapper the interpreter
    // fills in when it reports any error at all. It is not raised as an error
    // itself and it is not a translation error, so exempting it does not exempt
    // anything this parser produces. A second major appearing here would mean
    // the exemption list above has to be re-argued, not extended.
    assert_eq!(found, vec![101]);
}

#[test]
fn every_major_in_the_table_has_a_sub_zero_row_to_fall_back_to() {
    // `message`'s fallback panics without one, so this is the precondition for
    // the test above rather than a separate property.
    let mut missing = Vec::new();
    for message in rexx_inventory::errors::MESSAGES {
        if rexx_inventory::errors::lookup(message.major, 0).is_none() {
            missing.push(message.major);
        }
    }
    missing.dedup();
    assert!(missing.is_empty(), "majors with no sub-0 row: {missing:?}");
}

#[test]
fn the_placeholder_test_finds_a_digit_after_the_ampersand() {
    assert!(has_placeholder("found &1."));
    assert!(has_placeholder("&12 and &2"));
    assert!(has_placeholder("trailing &9"));
}

#[test]
fn the_placeholder_test_ignores_an_ampersand_with_nothing_fillable_after_it() {
    // A `&` with no digit after it is text, not a template. No table row is
    // spelled this way today, which is why the property is asserted on the
    // function rather than only through the table.
    assert!(!has_placeholder("A & B"));
    assert!(!has_placeholder("&"));
    assert!(!has_placeholder("&x"));
    assert!(!has_placeholder(""));
}

#[test]
fn the_placeholder_test_does_not_stop_at_the_first_ampersand() {
    assert!(has_placeholder("A & B &2"));
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
        ParseError::new(25, 901, 0).to_string(),
        "25.901: Invalid subkeyword found."
    );
}
