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

//! The context every `parse_*` function is handed, and the cursor over one
//! clause's tokens.

use std::cell::RefCell;
use std::ops::Range;

use crate::ProgramSource;
use crate::scanner::ResourceBody;
use crate::selector::SelectorTable;

use super::{Keywords, SymbolTable, Tag, Token};

/// Everything a `parse_*` function needs that is not the clause it is
/// parsing.
pub(crate) struct ParseCtx<'a> {
    /// Read by the instruction parser for `SourceKind`, which decides whether
    /// a label is error 47.1. The expression grammar needs only the tokens.
    pub(crate) source: &'a ProgramSource,
    pub(crate) tokens: &'a [Token],
    /// Read-only by the time parsing starts: `scan` has already interned every
    /// symbol in the program. Tasks 3.6 and 3.7 need it to compare a clause's
    /// first symbol against the pre-interned keyword ids, and Task 3.6 needs it
    /// to recover a label's spelling when it builds `Program::labels`.
    pub(crate) symbols: &'a SymbolTable,
    /// The message names this parse has interned, which is what
    /// `ExprKind::Message` holds one of.
    pub(crate) selectors: &'a RefCell<SelectorTable>,
    /// Every reserved *spelling* this parser recognises, pre-interned by `scan`
    /// before it reads any source, so their ids are fixed and every keyword
    /// test is an integer comparison. Keywords are NOT reserved words, so this
    /// is only ever consulted positionally.
    pub(crate) keywords: &'a Keywords,
    /// The `::RESOURCE` bodies `scan` copied out, keyed by the index of the
    /// `::` token that opened each directive.
    pub(crate) resources: &'a [ResourceBody],
}

/// A position inside one clause's token range, not inside the whole vector,
/// so an expression parser cannot walk off the end of its clause.
pub(crate) struct TokenCursor {
    /// Index range into `ParseCtx::tokens` that this cursor may visit.
    range: Range<usize>,
    /// Next index to yield; always inside `range` or equal to `range.end`.
    pos: usize,
}

impl TokenCursor {
    /// The instruction parser builds one of these per clause, from
    /// `Clause::tokens`; the expression grammar is handed one already built.
    pub(crate) fn new(range: Range<usize>) -> Self {
        Self {
            pos: range.start,
            range,
        }
    }

    /// Index of the next token, or None at the end of the range.
    pub(crate) fn peek(&self) -> Option<usize> {
        (self.pos < self.range.end).then_some(self.pos)
    }

    /// Yield the next token index and step past it. Deliberately not called
    /// `next`: `clippy::should_implement_trait` fires on an inherent `next`
    /// with this signature, and the phase gate runs clippy with `-D warnings`.
    pub(crate) fn advance(&mut self) -> Option<usize> {
        let i = self.peek()?;
        self.pos += 1;
        Some(i)
    }

    /// `nextReal` without consuming: index of the next token that is not a
    /// blank.
    pub(crate) fn peek_real(&self, tokens: &[Token]) -> Option<usize> {
        let mut i = self.peek()?;
        while i < self.range.end && tokens[i].kind.tag() == Tag::Blank {
            i += 1;
        }
        (i < self.range.end).then_some(i)
    }

    /// `nextReal`: the next token that is not a blank, consumed.
    pub(crate) fn advance_real(&mut self, tokens: &[Token]) -> Option<usize> {
        let i = self.peek_real(tokens)?;
        self.pos = i + 1;
        Some(i)
    }

    pub(crate) fn position(&self) -> usize {
        self.pos
    }

    /// Index of the first token this cursor may visit, whatever it has already
    /// yielded.
    pub(crate) fn start(&self) -> usize {
        self.range.start
    }

    /// One past the last token index this cursor may visit.
    pub(crate) fn end(&self) -> usize {
        self.range.end
    }
}
