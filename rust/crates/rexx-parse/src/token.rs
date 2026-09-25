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

//! Tokens, symbol interning, the keyword tables, and the context every
//! `parse_*` function is handed.

use std::ops::Range;

mod cursor;
mod keywords;
mod symbols;

pub(crate) use cursor::{ParseCtx, TokenCursor};
pub use keywords::{KeywordSet, Keywords};
pub use symbols::{SymbolId, SymbolTable};

/// A parse-time error, identified the way the interpreter identifies it: a
/// major number and a sub-number, as in `13.1` or `99.943`.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ParseError {
    /// The major error number, e.g. 13 for `Error 13: Invalid character in
    /// program`.
    pub code: u16,
    /// The sub-number, e.g. 1 for `Error 13.1`. Zero means the major error
    /// with no sub-code.
    pub sub: u16,
    /// The byte offset the error is *reported against*, which is the start of
    /// the clause being translated and not the offending character. The name
    /// reads like the latter; it is not.
    pub byte: usize,
}

impl ParseError {
    /// An error reported against the clause starting at `byte`.
    pub fn new(code: u16, sub: u16, byte: usize) -> Self {
        ParseError { code, sub, byte }
    }
}

/// An operator's identity, mirroring the operator range of `TokenSubclass`
/// (`Token.hpp:110`-`141`) in that order.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Operator {
    Plus,
    Subtract,
    Multiply,
    Divide,
    IntDiv,
    Remainder,
    Power,
    Abuttal,
    Concatenate,
    Blank,
    Equal,
    BackslashEqual,
    GreaterThan,
    BackslashGreaterThan,
    LessThan,
    BackslashLessThan,
    GreaterThanEqual,
    LessThanEqual,
    StrictEqual,
    StrictBackslashEqual,
    StrictGreaterThan,
    StrictBackslashGreaterThan,
    StrictLessThan,
    StrictBackslashLessThan,
    StrictGreaterThanEqual,
    StrictLessThanEqual,
    LessThanGreaterThan,
    GreaterThanLessThan,
    And,
    Or,
    Xor,
    Backslash,
}

impl Operator {
    /// The canonical source spelling.
    pub fn spelling(self) -> &'static str {
        match self {
            Operator::Plus => "+",
            Operator::Subtract => "-",
            Operator::Multiply => "*",
            Operator::Divide => "/",
            Operator::IntDiv => "%",
            Operator::Remainder => "//",
            Operator::Power => "**",
            Operator::Abuttal => "",
            Operator::Concatenate => "||",
            Operator::Blank => " ",
            Operator::Equal => "=",
            Operator::BackslashEqual => "\\=",
            Operator::GreaterThan => ">",
            Operator::BackslashGreaterThan => "\\>",
            Operator::LessThan => "<",
            Operator::BackslashLessThan => "\\<",
            Operator::GreaterThanEqual => ">=",
            Operator::LessThanEqual => "<=",
            Operator::StrictEqual => "==",
            Operator::StrictBackslashEqual => "\\==",
            Operator::StrictGreaterThan => ">>",
            Operator::StrictBackslashGreaterThan => "\\>>",
            Operator::StrictLessThan => "<<",
            Operator::StrictBackslashLessThan => "\\<<",
            Operator::StrictGreaterThanEqual => ">>=",
            Operator::StrictLessThanEqual => "<<=",
            Operator::LessThanGreaterThan => "<>",
            Operator::GreaterThanLessThan => "><",
            Operator::And => "&",
            Operator::Or => "|",
            Operator::Xor => "&&",
            Operator::Backslash => "\\",
        }
    }
}

/// What kind of thing a symbol names, from `scanSymbol`'s classification
/// (`Scanner.cpp:1527`-`1593`).
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum SymbolClass {
    /// A lone `.`, the placeholder in a `PARSE` template (`SYMBOL_DUMMY`).
    Dummy,
    /// Starts with a digit, or starts with `.` and scanned as a number: a
    /// literal value, never a variable (`SYMBOL_CONSTANT`).
    Constant,
    /// Starts with `.` and is not a number, e.g. `.true` or `.array`
    /// (`SYMBOL_DOTSYMBOL`).
    DotSymbol,
    /// A simple variable name, no periods (`SYMBOL_VARIABLE`).
    Variable,
    /// One period, at the end, e.g. `stem.` (`SYMBOL_STEM`).
    Stem,
    /// A period that is not the only one or not at the end, e.g. `stem.i.j`
    /// (`SYMBOL_COMPOUND`).
    Compound,
}

/// The 19 token classes of `TokenClass` (`Token.hpp:77`), in that order.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TokenKind {
    /// `TOKEN_NULL`. Never produced.
    Null,
    /// A blank that is an operator, under the two-sided rule in `scan`.
    Blank,
    /// A symbol. Carries the interned upcased spelling, not the source text:
    /// the identity is the `SymbolId` and the occurrence is `Token::span`,
    /// and neither substitutes for the other.
    Symbol {
        id: SymbolId,
        class: SymbolClass,
    },
    /// A quoted literal, carrying its *decoded* value: doubled quotes
    /// collapsed, and a `'…'x` or `'…'b` suffix already packed to bytes. This
    /// is the one token kind that cannot be a slice of its own span, because
    /// `'it''s'` has the value `it's`.
    Literal {
        value: Box<[u8]>,
    },
    Operator(Operator),
    /// A clause terminator: `;`, an uncontinued line end, or end of file.
    Eoc,
    Comma,
    /// `TOKEN_PREFIX`. Never produced.
    Prefix,
    LeftParen,
    RightParen,
    /// `TOKEN_POINT`. Never produced.
    Point,
    Colon,
    Tilde,
    DTilde,
    LeftBracket,
    RightBracket,
    DColon,
    /// `TOKEN_CONTINUE`. Never produced: a `,` or `-` continuation is
    /// resolved inside `locateToken` and becomes a blank or nothing.
    Continue,
    /// An operator immediately followed by `=`, e.g. `+=`, which the
    /// interpreter treats as an assignment shortcut rather than as an
    /// operator (`Token.cpp:95`).
    Assignment(Operator),
}

/// `TokenKind` without its payloads, for asserting token *shape*.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Tag {
    Null,
    Blank,
    Symbol,
    Literal,
    Operator,
    Eoc,
    Comma,
    Prefix,
    LeftParen,
    RightParen,
    Point,
    Colon,
    Tilde,
    DTilde,
    LeftBracket,
    RightBracket,
    DColon,
    Continue,
    Assignment,
}

impl TokenKind {
    /// This kind with its payloads dropped.
    pub fn tag(&self) -> Tag {
        match self {
            TokenKind::Null => Tag::Null,
            TokenKind::Blank => Tag::Blank,
            TokenKind::Symbol { .. } => Tag::Symbol,
            TokenKind::Literal { .. } => Tag::Literal,
            TokenKind::Operator(_) => Tag::Operator,
            TokenKind::Eoc => Tag::Eoc,
            TokenKind::Comma => Tag::Comma,
            TokenKind::Prefix => Tag::Prefix,
            TokenKind::LeftParen => Tag::LeftParen,
            TokenKind::RightParen => Tag::RightParen,
            TokenKind::Point => Tag::Point,
            TokenKind::Colon => Tag::Colon,
            TokenKind::Tilde => Tag::Tilde,
            TokenKind::DTilde => Tag::DTilde,
            TokenKind::LeftBracket => Tag::LeftBracket,
            TokenKind::RightBracket => Tag::RightBracket,
            TokenKind::DColon => Tag::DColon,
            TokenKind::Continue => Tag::Continue,
            TokenKind::Assignment(_) => Tag::Assignment,
        }
    }

    /// Whether a blank *following* this token can be significant:
    /// `RexxToken::isBlankSignificant()` (`Token.hpp:595`). One half of the
    /// two-sided rule; the other half looks at what comes next.
    pub fn makes_blank_significant(&self) -> bool {
        matches!(
            self,
            TokenKind::Symbol { .. }
                | TokenKind::Literal { .. }
                | TokenKind::RightParen
                | TokenKind::RightBracket
        )
    }
}

/// One token: what it is, and where it came from.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Range<usize>,
}

#[cfg(test)]
mod tests;
