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

use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Range;

use crate::ProgramSource;
use crate::scanner::ResourceBody;
use crate::selector::SelectorTable;

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

/// A symbol's identity: the upcased spelling, interned. Two symbols with the
/// same `SymbolId` name the same variable, method or label.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct SymbolId(u32);

impl SymbolId {
    /// This id as an index into the `SymbolTable` that interned it: **dense,
    /// zero-based, and assigned in interning order**, so `n` distinct symbols
    /// occupy exactly `0..n` and `SymbolTable::len` is the length a caller
    /// needs to size a `Vec`.
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

/// Interns upcased symbol spellings. Owned by `ProgramSource`'s parse, handed
/// to `Program` so Phase 4 can resolve a `SymbolId` back to text for error
/// messages and `SIGNAL`'s label lookup.
#[derive(Default, Debug)]
pub struct SymbolTable {
    by_name: HashMap<Box<str>, SymbolId>,
    names: Vec<Box<str>>,
}

impl SymbolTable {
    /// Intern `text`, upcasing it. Returns the same id for every spelling that
    /// differs only in case.
    pub fn intern(&mut self, text: &str) -> SymbolId {
        // Cow, not Box<str>, because `Box<str>: From<&str>` copies: building
        // the key eagerly would allocate on the lookup path even when the
        // symbol is already interned, which is the common case by an order of
        // magnitude. Borrow when the text is already upper, allocate only to
        // upcase, and allocate the owned key only on a genuine miss.
        let key: Cow<'_, str> = if text.bytes().any(|b| b.is_ascii_lowercase()) {
            Cow::Owned(text.to_ascii_uppercase())
        } else {
            Cow::Borrowed(text)
        };
        if let Some(&id) = self.by_name.get(key.as_ref()) {
            return id;
        }
        let id = SymbolId(u32::try_from(self.names.len()).expect("symbols fit u32"));
        let owned: Box<str> = key.into_owned().into();
        self.names.push(owned.clone());
        self.by_name.insert(owned, id);
        id
    }

    /// The upcased spelling. Panics on an id from a different table, which is
    /// a parser bug rather than a source error.
    pub fn name(&self, id: SymbolId) -> &str {
        &self.names[id.0 as usize]
    }

    /// How many distinct symbols are interned.
    pub fn len(&self) -> usize {
        self.names.len()
    }

    /// Whether nothing is interned yet.
    pub fn is_empty(&self) -> bool {
        self.names.is_empty()
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

/// One table: the interned spellings, in the order the C++ table lists them,
/// so a hit yields that table's own index and the caller maps the index to its
/// own enum.
#[derive(Debug)]
pub struct KeywordSet {
    ids: Vec<SymbolId>,
}

impl KeywordSet {
    /// Interns every spelling in `names`, keeping their order.
    fn new(symbols: &mut SymbolTable, names: &[&str]) -> Self {
        KeywordSet {
            ids: names.iter().map(|n| symbols.intern(n)).collect(),
        }
    }

    /// The table index of `id`, or `None` if `id` is not in this set.
    pub fn index_of(&self, id: SymbolId) -> Option<usize> {
        self.ids.iter().position(|&k| k == id)
    }

    /// How many spellings this table holds.
    pub fn len(&self) -> usize {
        self.ids.len()
    }

    /// Whether this table is empty, which no real table is.
    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }
}

/// The pre-interned spelling tables. Built by `scan` before it reads any
/// source, so a keyword test never hashes a string.
#[derive(Debug)]
pub struct Keywords {
    pub instructions: KeywordSet,
    pub sub_keywords: KeywordSet,
    pub conditions: KeywordSet,
    pub parse_options: KeywordSet,
    pub directives: KeywordSet,
    pub sub_directives: KeywordSet,
}

impl Keywords {
    /// Interns all six tables into `symbols`.
    pub fn new(symbols: &mut SymbolTable) -> Self {
        Keywords {
            instructions: KeywordSet::new(symbols, &INSTRUCTIONS),
            sub_keywords: KeywordSet::new(symbols, &SUB_KEYWORDS),
            conditions: KeywordSet::new(symbols, &CONDITIONS),
            parse_options: KeywordSet::new(symbols, &PARSE_OPTIONS),
            directives: KeywordSet::new(symbols, &DIRECTIVES),
            sub_directives: KeywordSet::new(symbols, &SUB_DIRECTIVES),
        }
    }
}

// The six tables, copied from `KeywordConstants.cpp` without reordering. The
// C++ keeps them in ASCII order so it can binary-search them; here the order
// is load-bearing for a different reason, since `index_of` returns a position
// that the caller maps to its own enum.

const INSTRUCTIONS: [&str; 35] = [
    "ADDRESS",
    "ARG",
    "CALL",
    "DO",
    "DROP",
    "ELSE",
    "END",
    "EXIT",
    "EXPOSE",
    "FORWARD",
    "GUARD",
    "IF",
    "INTERPRET",
    "ITERATE",
    "LEAVE",
    "LOOP",
    "NOP",
    "NUMERIC",
    "OPTIONS",
    "OTHERWISE",
    "PARSE",
    "PROCEDURE",
    "PULL",
    "PUSH",
    "QUEUE",
    "RAISE",
    "REPLY",
    "RETURN",
    "SAY",
    "SELECT",
    "SIGNAL",
    "THEN",
    "TRACE",
    "USE",
    "WHEN",
];

const SUB_KEYWORDS: [&str; 50] = [
    "ADDITIONAL",
    "APPEND",
    "ARG",
    "ARGUMENTS",
    "ARRAY",
    "BY",
    "CASE",
    "CLASS",
    "CONTINUE",
    "COUNTER",
    "DESCRIPTION",
    "DIGITS",
    "ENGINEERING",
    "ERROR",
    "EXIT",
    "EXPOSE",
    "FALSE",
    "FOR",
    "FOREVER",
    "FORM",
    "FUZZ",
    "INDEX",
    "INHERIT",
    "INPUT",
    "ITEM",
    "LABEL",
    "LOCAL",
    "MESSAGE",
    "NAME",
    "NOINHERIT",
    "NORMAL",
    "OFF",
    "ON",
    "OUTPUT",
    "OVER",
    "REPLACE",
    "RETURN",
    "SCIENTIFIC",
    "STEM",
    "STREAM",
    "STRICT",
    "THEN",
    "TO",
    "TRUE",
    "UNTIL",
    "USING",
    "VALUE",
    "WHEN",
    "WHILE",
    "WITH",
];

const CONDITIONS: [&str; 12] = [
    "ANY",
    "ERROR",
    "FAILURE",
    "HALT",
    "LOSTDIGITS",
    "NOMETHOD",
    "NOSTRING",
    "NOTREADY",
    "NOVALUE",
    "PROPAGATE",
    "SYNTAX",
    "USER",
];

const PARSE_OPTIONS: [&str; 10] = [
    "ARG", "CASELESS", "LINEIN", "LOWER", "PULL", "SOURCE", "UPPER", "VALUE", "VAR", "VERSION",
];

const DIRECTIVES: [&str; 9] = [
    "ANNOTATE",
    "ATTRIBUTE",
    "CLASS",
    "CONSTANT",
    "METHOD",
    "OPTIONS",
    "REQUIRES",
    "RESOURCE",
    "ROUTINE",
];

const SUB_DIRECTIVES: [&str; 40] = [
    "ABSTRACT",
    "ALL",
    "ATTRIBUTE",
    "CLASS",
    "CONDITION",
    "CONSTANT",
    "DELEGATE",
    "DIGITS",
    "END",
    "ERROR",
    "EXTERNAL",
    "FAILURE",
    "FORM",
    "FUZZ",
    "GET",
    "GUARDED",
    "INHERIT",
    "LIBRARY",
    "LOSTDIGITS",
    "METACLASS",
    "METHOD",
    "MIXINCLASS",
    "NAMESPACE",
    "NOPROLOG",
    "NOSTRING",
    "NOTREADY",
    "NOVALUE",
    "NUMERIC",
    "PACKAGE",
    "PRIVATE",
    "PROLOG",
    "PROTECTED",
    "PUBLIC",
    "ROUTINE",
    "SET",
    "SUBCLASS",
    "SYNTAX",
    "TRACE",
    "UNGUARDED",
    "UNPROTECTED",
];

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

#[cfg(test)]
mod tests;
