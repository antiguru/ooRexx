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
//!
//! This module is the parser's shared vocabulary rather than only the token
//! type: `ParseError`, `SymbolTable`, `Keywords`, `ParseCtx` and
//! `TokenCursor` all live here because every later parsing task names them
//! and none of them belongs to one task alone.

use std::borrow::Cow;
use std::collections::HashMap;
use std::ops::Range;

use crate::ProgramSource;

/// A parse-time error, identified the way the interpreter identifies it: a
/// major number and a sub-number, as in `13.1` or `99.943`.
///
/// Minimal on purpose. The message table, the substitution values and the
/// mapping from `byte` to a reported line belong to Task 3.8; every task
/// from the scanner on returns this type, so it exists now.
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
    ///
    /// The interpreter reports a syntax error against the clause: measured, a
    /// clause `say 1,` continued onto a line holding `'unclosed` reports error
    /// 6.2 on line 1, and the same holds for 6.1, 13.1 and 15.3. Task 3.8
    /// resolves this with `ProgramSource::line_of`.
    ///
    /// If Task 3.8 ever fills `subs`, it will need the offending position as a
    /// *second* field rather than by redefining this one, because several
    /// messages quote the offending text while still being reported against
    /// the clause: 13.1 quotes `"ä" ('C3A4'X)` and 15.3 quotes `found "g"`.
    pub byte: usize,
    /// The message substitution values.
    ///
    /// Always empty in this phase. Parse errors are not reproduced 1:1 here:
    /// the number and sub-number are gated, the message text and its
    /// substitutions are deliberately not. Task 3.8 owns filling this.
    pub subs: Vec<String>,
}

impl ParseError {
    /// An error with no substitutions, which is every error this phase
    /// raises.
    pub fn new(code: u16, sub: u16, byte: usize) -> Self {
        ParseError {
            code,
            sub,
            byte,
            subs: Vec::new(),
        }
    }
}

/// A symbol's identity: the upcased spelling, interned. Two symbols with the
/// same `SymbolId` name the same variable, method or label.
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct SymbolId(u32);

/// Interns upcased symbol spellings. Owned by `ProgramSource`'s parse, handed
/// to `Program` so Phase 4 can resolve a `SymbolId` back to text for error
/// messages and `SIGNAL`'s label lookup.
///
/// What this buys, measured with the scanner rather than estimated:
/// `CoreClasses.orx` holds 8,118 symbol occurrences over 526 distinct upcased
/// symbols, and `StreamClasses.orx` 2,121 over 273. So this replaces ten
/// thousand short-string allocations with that many hash probes plus a few
/// hundred `Box<str>`. It also turns keyword recognition and variable lookup
/// into integer comparisons.
#[derive(Default, Debug)]
pub struct SymbolTable {
    by_name: HashMap<Box<str>, SymbolId>,
    names: Vec<Box<str>>,
}

impl SymbolTable {
    /// Intern `text`, upcasing it. Returns the same id for every spelling that
    /// differs only in case.
    ///
    /// `to_ascii_uppercase` is byte-identical to the interpreter's
    /// `translateChar` over everything this can receive, and the reason is
    /// `LanguageParser::characterTable` (`Scanner.cpp:60`): it maps only `!`,
    /// `.`, `0`-`9`, `?`, `A`-`Z`, `_` and `a`-`z`, and is **zero for every byte
    /// from 0x80 to 0xFF**. A non-ASCII byte therefore cannot be part of a
    /// symbol at all -- `bäc = 2` is a parse-time error 13.1, `Incorrect
    /// character in program "ä" ('C3A4'X)`. This matters because the scanner
    /// works over bytes so that a UTF-8 sequence survives a round trip, which
    /// is true of literals and comments and must not be read as licence to
    /// admit non-ASCII into a symbol, where it would silently under-upcase.
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
///
/// Two of these are never scanned: `Abuttal`, which the parser synthesises
/// where two terms sit side by side with nothing between them, and `Blank`,
/// which is the subclass the C++ gives `TOKEN_BLANK` and which
/// `TokenKind::Blank` carries here instead. `Concatenate` is both, scanned
/// from `||` and synthesised. All three are listed because a later task's
/// precedence table is indexed by this enum.
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
    ///
    /// Canonical because two spellings can scan to one operator: `\`, `0xAA`
    /// and `0xAC` all give `Backslash`, so this returns the ASCII one rather
    /// than the source bytes. Recover the source bytes from the token's span
    /// where the distinction matters.
    ///
    /// `Abuttal` has no spelling at all and gives the empty string, because
    /// the parser synthesises it where two terms sit side by side with nothing
    /// between them.
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
///
/// The C++ additionally tags a pure-integer constant that fits the platform's
/// integer width with `INTEGER_CONSTANT` (`Scanner.cpp:1546`). That flag only
/// selects an internal number representation (`LanguageParser.cpp:2371`
/// builds an integer object instead of a string plus number-string) and has
/// no observable effect, so it is not reproduced. Reproducing it would also
/// mean reproducing a platform-dependent digit limit, 9 on a 32-bit build and
/// 18 on a 64-bit one.
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
///
/// `Null`, `Prefix`, `Point` and `Continue` are never produced. They are not
/// produced by the C++ either: a grep over `interpreter/` finds
/// `TOKEN_PREFIX`, `TOKEN_POINT`, `TOKEN_CONTINUE` and `TOKEN_NULL` only in
/// the enum declaration itself. They are listed so that this enum is a
/// faithful mirror, and so that a reader comparing the two files does not go
/// looking for a class that was dropped.
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
///
/// One variant per `TokenKind` variant. Assert shape with `Tag` and identity
/// with the `SymbolId` separately, because a test that asserts both at once
/// cannot say which failed.
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
///
/// The span is a byte range into the retained source. It is kept for every
/// token including a symbol, because the source spelling is observable even
/// though the identity is upcased: `sourceline(1)` on `abc = 1` returns
/// `abc = 1`, and `trace r` on `aBc = 2` prints `aBc = 2`.
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
    ///
    /// Linear over at most 50 `SymbolId`s, which is a handful of `u32`
    /// comparisons in cache and needs no ordering. Do NOT sort this and do not
    /// binary-search it: an entry's position IS its meaning to the caller.
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
///
/// One table per C++ table, in `KeywordConstants.cpp` order: 35 keyword
/// instructions, 50 `subKeywords`, 12 `conditionKeywords`, 10
/// `parseOptions`, 9 `directives`, 40 `subDirectives`. They are separate
/// because the same spelling means different things in different positions:
/// `VALUE` is a `parseOptions` entry and a sub-keyword of several
/// instructions, and nothing may conflate them.
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
    ///
    /// This runs per `SymbolTable`, so an `INTERPRET` in a loop pays the whole
    /// set every iteration and `Program::symbols` always carries names that
    /// never appear in the source. Both are accepted: the alternative is a
    /// shared table, and a shared table cannot hand out stable ids per
    /// program.
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
///
/// A `Clause` holds a range into the token vector, so a function given only a
/// clause cannot reach the tokens; this bundles them with the two tables that
/// every instruction and directive parser consults.
///
/// Crate-internal: nothing above the parser names it. Phase 4 consumes the
/// AST, not the token stream it was built from.
pub(crate) struct ParseCtx<'a> {
    /// Read by the instruction parser for `SourceKind`, which decides whether
    /// a label is error 47.1. The expression grammar needs only the tokens.
    pub(crate) source: &'a ProgramSource,
    pub(crate) tokens: &'a [Token],
    /// Read-only by the time parsing starts: `scan` has already interned every
    /// symbol in the program. Tasks 3.6 and 3.7 need it to compare a clause's
    /// first symbol against the pre-interned keyword ids, and Task 3.6 needs it
    /// to recover a label's spelling when it builds `Program::labels`.
    ///
    /// Not for error substitutions: this phase does not reproduce them.
    ///
    /// Read-only is not quite enough for the expression grammar, and Task 3.5
    /// worked around it rather than widening this: a message name taken from a
    /// literal, as in `a~'length'`, has to be upcased and would have to be
    /// interned, and `scan` never saw it as a symbol. `ExprKind::Message`
    /// therefore carries bytes where every other name carries a `SymbolId`.
    pub(crate) symbols: &'a SymbolTable,
    /// Every reserved *spelling* this parser recognises, pre-interned by `scan`
    /// before it reads any source, so their ids are fixed and every keyword
    /// test is an integer comparison. Keywords are NOT reserved words, so this
    /// is only ever consulted positionally.
    pub(crate) keywords: &'a Keywords,
}

/// A position inside one clause's token range, not inside the whole vector,
/// so an expression parser cannot walk off the end of its clause.
///
/// Crate-internal, for the same reason as `ParseCtx`.
///
/// Forward only. The C++ consumes a token and calls `previousToken` to put it
/// back; the grammar here peeks and only then consumes, so it never rewinds.
/// A `back` method existed and was removed once the expression grammar showed
/// it had no caller, and Tasks 3.6 and 3.7 are the same style.
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

    pub(crate) fn position(&self) -> usize {
        self.pos
    }

    /// Index of the first token this cursor may visit, whatever it has already
    /// yielded.
    ///
    /// A cursor is built from one clause's token range, so this is the clause's
    /// first token, which is what a parse error is reported against.
    pub(crate) fn start(&self) -> usize {
        self.range.start
    }
}

#[cfg(test)]
mod tests;
