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

//! Cutting the token vector into clauses.
//!
//! Ported from `LanguageParser::nextClause` (`LanguageParser.cpp:1009`) for the
//! terminator rules, and from the label arm of
//! `LanguageParser::nextInstruction` (`InstructionParser.cpp:150`-`176`) for the
//! rule that a label's colon ends a clause.
//!
//! The label rule is implemented here even though the C++ implements it one
//! layer up, in the instruction parser. That is possible because a label is
//! recognisable from the token stream alone: a symbol or a literal, immediately
//! followed by a colon, at the start of a clause. `THEN`, `ELSE` and
//! `OTHERWISE` also end a clause mid-line in the C++, and those cannot move
//! down here, because only the instruction parser knows whether a `THEN` token
//! is the `THEN` of an open `IF` or a variable named `then`. They stay with the
//! instruction parser and narrow a clause that this function produced.

use std::ops::Range;

use crate::token::{ParseError, Tag, Token};

/// One clause: the tokens it holds, and the source text `TRACE` prints for it.
///
/// `tokens` and `span` move independently and neither is derivable from the
/// other. An instruction that ends mid-clause moves the *next* clause's token
/// range forward while narrowing its own `span` end, and the two adjustments
/// are separate, so bytes between them belong to no clause at all.
#[derive(Clone, Debug)]
pub struct Clause {
    /// Index range into the `ParseCtx::tokens` slice, terminating token
    /// excluded. That terminator is an `Eoc` for an ordinary clause and a
    /// `Colon` for a label clause.
    pub tokens: Range<usize>,
    /// Byte range in the retained source: from the start of the first token to
    /// the END of the terminating token. An explicit `;` is therefore inside the
    /// span. For an end of line the span stops at the last byte of the line's
    /// content, excluding the line terminator.
    ///
    /// Measured against `build/bin/rexx` with `trace r`, which prints exactly
    /// these bytes: `nop;` traces with its semicolon, `here:` with its colon,
    /// `say 1 ;` with the blank before the semicolon, `say 1;   ` *without* the
    /// blanks after it, and `say 1 -- trailing comment` with the whole comment,
    /// because there the terminator is the line end rather than the `--`.
    pub span: Range<usize>,
    /// The label's own token range, when the clause is `name:`.
    pub label: Option<Range<usize>>,
}

/// Splits `tokens` into clauses.
///
/// A clause ends at a `;`, at an uncontinued line end, or at end of file, all
/// three of which reach here as an `Eoc`. A label's `:` ends a clause too.
/// The `Eoc` and the `:` are terminators and belong to no clause's `tokens`,
/// but both are inside the clause's `span`.
///
/// Expects the token vector `scan` produces, whose invariants are stated on
/// `Scanned::tokens`: no `Eoc` is first, no two are adjacent, and the last
/// token is one. A slice that ends mid-clause instead ends the final clause at
/// its last token.
///
/// The `Result` is part of the interface every later parsing stage shares. No
/// input reaches an error return here: the terminator rules cannot fail, and
/// the one label error the C++ raises, error 47.1 for a label in `INTERPRET`
/// text, needs the source kind that this function is not given.
pub fn split_clauses(tokens: &[Token]) -> Result<Vec<Clause>, ParseError> {
    let mut clauses = Vec::new();
    let mut index = 0;

    while index < tokens.len() {
        // The scanner never emits a terminator first or two in a row, so this
        // position holds a real token and the clause is not empty.
        let terminator = (index..tokens.len()).find(|&i| tokens[i].kind.tag() == Tag::Eoc);
        let (limit, span_end) = match terminator {
            Some(i) => (i, tokens[i].span.end),
            // Unterminated, which `scan` does not produce. The last token's own
            // end is then the best available clause end.
            None => (tokens.len(), tokens[tokens.len() - 1].span.end),
        };

        // Peel labels off the front. `a: b: nop` is three clauses, so this
        // repeats rather than testing once.
        let mut start = index;
        while start < limit {
            // A blank never sits between a label and its colon: a blank is
            // only a token when the next real character starts a symbol, a
            // literal, a `(` or a `[`, and `:` is none of those. So the colon
            // is at `start + 1` exactly, which is also where the C++ looks,
            // with `nextToken` rather than `nextReal`.
            let labelled = start + 1 < limit
                && matches!(tokens[start].kind.tag(), Tag::Symbol | Tag::Literal)
                && tokens[start + 1].kind.tag() == Tag::Colon;
            if !labelled {
                clauses.push(Clause {
                    tokens: start..limit,
                    span: tokens[start].span.start..span_end,
                    label: None,
                });
                break;
            }
            // The label clause's span always stops at the colon, even when the
            // colon is the last token before a `;` and so does not itself
            // terminate anything. `labelNew` (`InstructionParser.cpp:2809`)
            // sets the end unconditionally, and measured, `here: ; nop` traces
            // as `here:` then `nop`.
            clauses.push(Clause {
                tokens: start..start + 1,
                span: tokens[start].span.start..tokens[start + 1].span.end,
                label: Some(start..start + 1),
            });
            start += 2;
        }

        index = limit + 1;
    }

    Ok(clauses)
}
