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

//! What a `ParseError` reports: its message, from the generated table, and the
//! physical line it is reported on.
//!
//! The type itself stays in `token.rs`, where Task 3.3 put it and said why.
//! This module is its completion, which is a separate concern: `token.rs` is
//! the parser's shared vocabulary and this is the only place that knows the
//! message table exists.
//!
//! # What this phase reproduces, and what it does not
//!
//! Gated: the major number, the sub-number, and the reported line. Those are
//! differentially tested against `build/bin/rexxc` over the corpus in
//! `rust/corpus/errors/parse-errors.tsv`, in both directions.
//!
//! Not reproduced: the **sub-message's substitution values**, and therefore the
//! second line the interpreter prints. `rexxc` answers with two lines, a major
//! and a sub:
//!
//! ```text
//! Error 7 running select.rex line 3:  WHEN or OTHERWISE expected.
//! Error 7.1:  SELECT on line 3 requires WHEN.
//! ```
//!
//! `message` produces the first of those. It is an *observable* deviation and
//! not an unobservable one, because a trapped syntax error hands the program
//! `ERRORTEXT`, `MESSAGE` and `ADDITIONAL`, so a Rexx program reading those
//! would see the difference. That was a scope decision, recorded in the phase
//! plan's exit gate. `rexx-num`'s runtime errors are untouched by it: their
//! numbers, message text and `ADDITIONAL` values stay byte-exact.
//!
//! # Why `ParseError` carries no substitution values
//!
//! It used to carry `subs: Vec<String>`, which every construction site filled
//! with an empty vector. A field nothing sets reads as a contract, so Task 3.8
//! had to either fill it or remove it. It removed it, and the measurement is
//! the argument.
//!
//! Of the **200** distinct `(major, sub)` pairs the crate's own tests reach,
//! **92** have a substitution in their sub-message. They need three kinds of
//! value:
//! the offending token's text (about sixty of them, the `found "&1"` family),
//! the line of the construct that is still open (7.1, 7.2, 10.002-10.007,
//! 14.x, 18.1, 18.2), and a keyword's own spelling (19.925, 20.929, 25.927,
//! 35.935, 49.002). Only one needs something out of scope outright: 36.901's
//! `&1` is a byte offset within a line, which this phase does not produce.
//!
//! So filling is *possible*. It is not cheap: `syntaxError` in the C++ is handed
//! the offending token at each of its call sites, and this parser's roughly two
//! hundred raise sites are not, so every one would have to name its own
//! offender. And this phase does not gate substitution values, which means all
//! two hundred would land unverified under a gate that cannot see them wrong. A
//! value that looks right and is wrong is worse than one that is absent. The
//! field went.
//!
//! What is owed, and to whom: Phase 4 has to answer `condition('o')~additional`
//! for a trapped syntax error, and that is where the values become observable
//! from Rexx rather than only from a message. Measured through a `signal on
//! syntax` trap, `interpret "x: nop"` hands the program `additional=X` where this
//! phase would hand it nothing. Phase 4 will need them for real, with a
//! differential test per substitution, and `token.rs`'s note on `byte` still
//! records the second field that job needs.

use crate::ProgramSource;
use crate::token::ParseError;

impl ParseError {
    /// The interpreter's message text for this error, rendered from the
    /// generated table.
    ///
    /// Never contains an unfilled `&1`-style substitution placeholder, which is
    /// what decides between the two rows the table holds for an error like
    /// `7.1`: the sub-message when the sub-message needs no substitution, and
    /// the major's own text when it does. **92 of the 200** distinct errors the
    /// crate's own tests reach fall on the second branch -- 193 pairs from the
    /// 557 translation-error rows of `corpus/errors/parse-errors.tsv` plus the
    /// seven `INTERPRET`-only pairs, which no corpus row can hold.
    ///
    /// The major's text is always available and always complete. Measured over
    /// the whole generated table: of its 704 rows exactly one with sub 0 carries
    /// a placeholder, `101.000`, and 101 is a runtime error this parser cannot
    /// raise. `tests/errors.rs` asserts the property for every error the corpus
    /// actually reaches rather than relying on that count.
    pub fn message(&self) -> String {
        // The sub-message row first, because it is the specific one, and the
        // major's row only when the specific one would leave a placeholder
        // visible. Filling the placeholder is not an option here -- see the
        // module's note on scope -- and a user-facing message reading
        // `found "&1"` would be worse than the generic one that is true.
        let specific = row(self.code, self.sub);
        if let Some(text) = specific.filter(|text| !has_placeholder(text)) {
            return text.to_string();
        }
        row(self.code, 0)
            .unwrap_or_else(|| panic!("no interpreter message for error {}", self.code))
            .to_string()
    }

    /// The 1-based physical line this error is reported on.
    ///
    /// `byte` is the clause's start, not the offending token's, so this is the
    /// line the interpreter's MAIN message names. It is not always the line of
    /// the text that is wrong, and the interpreter is not consistent about
    /// which it uses: 7.1 reports the `SELECT`'s own line, 7.2 the offending
    /// clause's, and the 10.x family the `END`'s. All three are measured.
    ///
    /// `source` must be the source this error came out of. Nothing here can
    /// check that, because a `ParseError` deliberately does not borrow the
    /// source it was raised against -- both `parse_program` and
    /// `parse_interpret` hand the source back to the caller on success and
    /// would not be able to if it did.
    pub fn line(&self, source: &ProgramSource) -> usize {
        source.line_of(self.byte)
    }
}

impl std::fmt::Display for ParseError {
    /// `13.1: Invalid character in program.`
    ///
    /// Not the interpreter's own format, which needs the program's name and the
    /// reported line and so belongs to whoever holds them. This is for a
    /// `Result` that reaches a `main` or an assertion message.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}: {}", self.code, self.sub, self.message())
    }
}

impl std::error::Error for ParseError {}

/// The generated table's text for one `(major, sub)` pair, or `None` when the
/// table has no such row.
fn row(code: u16, sub: u16) -> Option<&'static str> {
    rexx_inventory::errors::lookup(code, sub).map(|message| message.text)
}

/// Whether `text` still holds a `&1`-style substitution placeholder.
///
/// `rexx-inventory` renders `<Sub position="N"/>` as the literal `&N` and
/// leaves filling it to its caller, so this is the check for "the table row is
/// a template, not a finished sentence". A bare `&` with no digits after it is
/// not a placeholder: nothing would fill it.
fn has_placeholder(text: &str) -> bool {
    let mut rest = text;
    while let Some(at) = rest.find('&') {
        rest = &rest[at + 1..];
        if rest.starts_with(|c: char| c.is_ascii_digit()) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests;
