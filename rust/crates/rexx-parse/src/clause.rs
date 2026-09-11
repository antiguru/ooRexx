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

use std::ops::Range;

use crate::token::{ParseCtx, ParseError, Tag, Token};

/// One clause: the tokens it holds, and the source text `TRACE` prints for it.
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
    pub(crate) span: Range<usize>,
    /// The label's own token range, when the clause is `name:`.
    pub(crate) label: Option<Range<usize>>,
}

/// Splits `tokens` into clauses.
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
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) enum PendingThen {
    If,
    When,
}

/// The clause list being parsed, and a position in it.
pub(crate) struct ClauseCursor {
    clauses: Vec<Clause>,
    /// Next index in `clauses`, used only when `pending` is None.
    pos: usize,
    /// The remainder of a clause that `split_before` ended early. Yielded
    /// ahead of `clauses[pos]`. Not necessarily contiguous with the clause it
    /// was split from: see `split_before`.
    pending: Option<Clause>,
    /// Set when the clause just parsed was an `IF` or `WHEN` whose `THEN` has
    /// not been seen yet, with the byte its own clause started at. Read and
    /// cleared by the next clause's parse.
    /// ```text
    /// 1  nop
    /// 2  if 1 = 1
    /// 3
    /// 4
    /// 5  nop
    ///    Error 18 running ... line 5:  THEN expected.
    ///    Error 18.1:  IF instruction on line 2 requires matching THEN clause.
    /// ```
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
    pub(crate) fn expect_then(&mut self, which: PendingThen, byte: usize) {
        self.pending_then = Some((which, byte));
    }

    /// Whether a `THEN` is expected next, clearing the expectation.
    pub(crate) fn take_expected_then(&mut self) -> Option<(PendingThen, usize)> {
        self.pending_then.take()
    }

    /// End the current clause at byte `end_at`, and re-present tokens `at..`
    /// as the next clause starting at token `at`'s own start byte.
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
