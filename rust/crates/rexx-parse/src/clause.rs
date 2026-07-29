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

use crate::token::{ParseCtx, ParseError, Tag, Token};

/// One clause: the tokens it holds, and the source text `TRACE` prints for it.
///
/// `tokens` and `span` move independently and neither is derivable from the
/// other. An instruction that ends mid-clause moves the *next* clause's token
/// range forward while narrowing its own `span` end, and the two adjustments
/// are separate, so bytes between them belong to no clause at all.
///
/// Crate-internal: a clause is scaffolding for building instructions, and its
/// span is copied into the instruction it produces, so nothing above the parser
/// names it.
///
/// Task 3.6 is the first non-test reader of all three fields.
#[derive(Clone, Debug)]
pub(crate) struct Clause {
    /// Index range into the `ParseCtx::tokens` slice, terminating token
    /// excluded. That terminator is an `Eoc` for an ordinary clause and a
    /// `Colon` for a label clause.
    pub(crate) tokens: Range<usize>,
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
    pub(crate) span: Range<usize>,
    /// The label's own token range, when the clause is `name:`.
    pub(crate) label: Option<Range<usize>>,
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
pub(crate) fn split_clauses(tokens: &[Token]) -> Result<Vec<Clause>, ParseError> {
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

/// Which instruction is still waiting for its `THEN`, and where it is.
///
/// This is the whole of the control stack that the instruction parser keeps,
/// and it exists because `THEN` is the one keyword whose legality the C++
/// decides in `nextInstruction` itself: a `THEN` reached there is always error
/// 8.1, because a real one is consumed by `translateBlock` while it finishes
/// the `IF` (`LanguageParser.cpp:1329`-`1360`). One entry is therefore enough
/// to keep both directions right, and the two variants are needed because the
/// missing-`THEN` error names which instruction wanted it: measured, `if 1 = 1`
/// followed by `nop` is 18.1 and a `WHEN` in the same shape is 18.2.
///
/// Every other block-structure decision needs the full stack and belongs to
/// the task that assembles the instruction chain.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) enum PendingThen {
    If,
    When,
}

/// The clause list being parsed, and a position in it.
///
/// Owns the list rather than borrowing a slice, because an instruction that
/// ends mid-clause has to re-present the remainder as a clause of its own and
/// that remainder is not an element of the list. See `split_before`.
pub(crate) struct ClauseCursor {
    clauses: Vec<Clause>,
    /// Next index in `clauses`, used only when `pending` is None.
    pos: usize,
    /// The remainder of a clause that `split_before` ended early. Yielded
    /// ahead of `clauses[pos]`. Not necessarily contiguous with the clause it
    /// was split from: see `split_before`.
    pending: Option<Clause>,
    /// Set when the clause just parsed was an `IF` or `WHEN` whose `THEN` has
    /// not been seen yet, with the byte its clause started at. Read and cleared
    /// by the next clause's parse.
    ///
    /// The byte is carried because the missing-`THEN` error is reported against
    /// the `IF`, not against the clause that should have held the `THEN`:
    /// `syntaxError(Error_Then_expected_if, instruction)` takes the
    /// INSTRUCTION's location (`LanguageParser.cpp:1341`). Measured, with the
    /// two clauses four lines apart: `nop` / `if 1 = 1` / `nop` / `nop` / `nop`
    /// reports 18.1 on line 2, and moving the `IF` to line 4 moves the report
    /// to line 4.
    pending_then: Option<(PendingThen, usize)>,
}

impl ClauseCursor {
    pub(crate) fn new(clauses: Vec<Clause>) -> Self {
        Self {
            clauses,
            pos: 0,
            pending: None,
            pending_then: None,
        }
    }

    /// The clause being parsed, without consuming it.
    pub(crate) fn peek(&self) -> Option<&Clause> {
        self.pending.as_ref().or_else(|| self.clauses.get(self.pos))
    }

    /// Consume and return the clause being parsed.
    pub(crate) fn next_clause(&mut self) -> Option<Clause> {
        if let Some(c) = self.pending.take() {
            return Some(c);
        }
        let c = self.clauses.get(self.pos)?.clone();
        self.pos += 1;
        Some(c)
    }

    /// Record that the clause just parsed was an `IF` or `WHEN` whose `THEN`
    /// is still to come, whether on this line or the next.
    ///
    /// `byte` is that instruction's own clause start, which is where a missing
    /// `THEN` is reported.
    pub(crate) fn expect_then(&mut self, which: PendingThen, byte: usize) {
        self.pending_then = Some((which, byte));
    }

    /// Whether a `THEN` is expected next, clearing the expectation.
    ///
    /// Called once per clause parsed, so that the expectation lasts exactly
    /// one clause: that is what makes a `THEN` anywhere else error 8.1.
    pub(crate) fn take_expected_then(&mut self) -> Option<(PendingThen, usize)> {
        self.pending_then.take()
    }

    /// End the current clause at byte `end_at`, and re-present tokens `at..`
    /// as the next clause starting at token `at`'s own start byte.
    ///
    /// **This is not a partition, and that is the whole point.** The oracle
    /// makes two independent adjustments with a gap between them, so bytes
    /// between `end_at` and the next clause's start belong to NO clause. Two
    /// positions are required. A single cut point cannot reproduce the
    /// interpreter, and one that tried would be wrong on one side or the other.
    ///
    /// Callers pass `end_at` as follows:
    ///
    /// * `IF`/`WHEN` pass the START byte of whatever token ended the
    ///   condition, so the condition clause keeps its trailing blanks.
    ///   `RexxInstructionIf` does `setEnd(...)` from that token's start
    ///   (`IfInstruction.cpp:58`-`66`). Measured both spellings:
    ///   `if 1 = 1   then    say "a"` traces the condition as `if 1 = 1   `
    ///   with all three blanks, and `if 1 = 1;` with `then` on the next line
    ///   traces as `if 1 = 1` WITHOUT its semicolon, where `nop;` traces with
    ///   one. Only the first spelling reaches this function, because the
    ///   second leaves nothing to re-present.
    /// * `THEN`/`ELSE`/`OTHERWISE` pass their own keyword token's END byte,
    ///   so the keyword clause carries no blank on either side.
    ///   `RexxInstructionThen` takes the token's whole location
    ///   (`ThenInstruction.cpp:76`). `RexxClause::trim` (`Clause.cpp:138`)
    ///   moves only the start, which is why the two ends move separately.
    ///
    /// Measured, for `if 1 = 1   then    say "a"` under `trace r`: the
    /// condition clause keeps all THREE trailing blanks, `then` carries none
    /// on either side despite four following it, and `say "a"` starts at `say`
    /// with zero leading blanks. The four blanks after `then` are in no clause.
    ///
    /// Panics if `at` is outside the current clause's token range, or if
    /// `end_at` is outside the current clause's byte span. Both are parser
    /// bugs rather than source errors.
    pub(crate) fn split_before(&mut self, ctx: &ParseCtx, at: usize, end_at: usize) -> Clause {
        let cur = self
            .next_clause()
            .expect("split_before with no current clause");
        assert!(cur.tokens.contains(&at), "split_before outside the clause");
        assert!(
            cur.span.contains(&end_at) || end_at == cur.span.end,
            "split_before end byte outside the clause"
        );
        self.pending = Some(Clause {
            tokens: at..cur.tokens.end,
            span: ctx.tokens[at].span.start..cur.span.end,
            label: None,
        });
        Clause {
            tokens: cur.tokens.start..at,
            span: cur.span.start..end_at,
            label: cur.label,
        }
    }
}

#[cfg(test)]
mod tests;
