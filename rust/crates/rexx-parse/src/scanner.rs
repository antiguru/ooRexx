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

//! Turning retained source into a token vector.
//!
//! Ported from `interpreter/parser/Scanner.cpp`: `locateToken`,
//! `sourceNextToken`, `scanSymbol`, `scanLiteral`, `scanComment`,
//! `nextSpecial`, `packHexLiteral` and `packBinaryLiteral`. The port is
//! structural on purpose. The scanner's hard cases are not independent rules
//! that can be reimplemented one at a time; they interact, and the
//! interactions are where the behaviour lives. `nextSpecial` skipping blanks
//! and comments is the clearest example: it makes `1 = = 1.0` a strict
//! comparison, and so do `1 =/*c*/= 1.0` and a `=` continued onto the next
//! line with `=-`. All three were measured, and all three fall out of the
//! structure rather than being special-cased.
//!
//! Two departures from the C++ are deliberate and both follow from producing
//! the whole vector up front rather than pulling one clause at a time.
//!
//! * A clause terminator is never emitted twice in a row and never for an
//!   empty final clause, so a blank line or a stray `;;` produces no tokens.
//!   This is the effect of `nextClause`'s null-clause skipping
//!   (`LanguageParser.cpp:1009`) without reproducing the separate
//!   `CLAUSEEND_EOL`/`CLAUSEEND_EOF` subclasses, which nothing in this phase
//!   needs to tell apart.
//! * The interpreter interleaves scanning with parsing, so a scan error later
//!   in the file is never reached if an earlier clause fails to parse.
//!   Measured: a file whose line 1 is `say )` and whose line 3 is
//!   `x = 'unclosed` reports 37.2 on line 1, where an eager scan reports 6.2
//!   on line 3. Both are errors and both are refusals; only the number
//!   differs.

use std::ops::Range;

use crate::token::{
    Keywords, Operator, ParseError, SymbolClass, SymbolId, SymbolTable, Tag, Token, TokenKind,
};
use crate::{ProgramSource, SourceKind};

/// The longest symbol the interpreter accepts
/// (`LanguageParser::MAX_SYMBOL_LENGTH`). Measured: a 250-character name
/// works, 251 raises error 30.1.
const MAX_SYMBOL_LENGTH: usize = 250;

/// The end marker a `::RESOURCE` directive uses when it names none
/// (`GlobalNames::DEFAULT_RESOURCE_END`).
const DEFAULT_RESOURCE_END: &[u8] = b"::END";

/// One `::RESOURCE` body, copied verbatim rather than tokenised.
///
/// The body is not Rexx and must not be scanned as Rexx. Measured: a resource
/// holding `this is 'unmatched and /* unclosed` gets rc 0 from `rexxc`, so a
/// scanner that tokenised it would invent errors 6.2 and 6.1 that the
/// interpreter does not raise.
///
/// The scanner establishes only the body's extent, which is all it has to do
/// to avoid tokenising it. A directive parser still owes the rest of
/// `resourceDirective` (`DirectiveParser.cpp:2266`): keying the package's
/// resource table by the *upcased* name even though the end marker is not
/// upcased when it comes from a literal, rejecting a duplicate name with
/// `Error_Translation_duplicate_resource`, and rejecting a malformed
/// directive, which is error 25.926 and is why a malformed one leaves no
/// `ResourceBody` here at all.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ResourceBody {
    /// Index in `Scanned::tokens` of the `::` that opened the directive
    /// clause, which is how a directive parser finds its own body.
    pub directive: usize,
    /// Byte range of each body line in the retained source, terminators
    /// excluded, in order. The terminating marker line is not included.
    pub lines: Vec<Range<usize>>,
}

/// Everything one pass of the scanner produced.
///
/// `scan` returns the tables because it owns interning; it cannot borrow one
/// it is still filling.
#[derive(Debug)]
pub struct Scanned {
    /// Every token, in source order.
    ///
    /// Three invariants a clause splitter may rely on. No two clause
    /// terminators are adjacent, and none is first, so every `Eoc` closes a
    /// clause that holds at least one token. If there is any token at all the
    /// last one is an `Eoc`, because end of file terminates the final clause.
    /// A program with no clauses, whether empty, all blank lines, all
    /// comments or only semicolons, produces no tokens at all rather than a
    /// lone terminator.
    pub tokens: Vec<Token>,
    pub symbols: SymbolTable,
    pub keywords: Keywords,
    /// The `::RESOURCE` bodies, in source order. Empty for almost every
    /// program.
    pub resources: Vec<ResourceBody>,
}

/// Tokenises `source`.
///
/// A clause terminator is emitted for each `;`, each uncontinued line end and
/// for end of file, with consecutive terminators collapsed.
///
/// Whether this is a program or `INTERPRET` text is `source.kind()`, decided
/// when the source was built, so there is no way to construct one and scan it
/// as the other.
pub fn scan(source: &ProgramSource) -> Result<Scanned, ParseError> {
    let mut symbols = SymbolTable::default();
    let keywords = Keywords::new(&mut symbols);
    let mut scanner = Scanner::new(source, symbols);
    scanner.run()?;
    Ok(Scanned {
        tokens: scanner.tokens,
        symbols: scanner.symbols,
        keywords,
        resources: scanner.resources,
    })
}

/// `LanguageParser::isSymbolCharacter` (`LanguageParser.hpp:415`): whether
/// `byte` may appear in a symbol.
///
/// The C++ reads this out of `characterTable` (`Scanner.cpp:60`), whose
/// non-zero entry for a byte is that byte *upcased*. Nothing here needs the
/// upcased value, because `SymbolTable::intern` upcases what it is given and
/// `to_ascii_uppercase` agrees with the table over every byte the table
/// admits. The table is zero for every byte from 0x80 to 0xFF, which is why a
/// symbol is always ASCII.
fn is_symbol_char(byte: u8) -> bool {
    matches!(byte,
        b'!' | b'.' | b'?' | b'_' | b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z')
}

/// The line scanning starts on: 2 when a program opens with a `#!` line,
/// 1 otherwise.
///
/// `BufferProgramSource::buildDescriptors` (`ProgramSource.cpp:448`) sets
/// `firstLine` and `LanguageParser::translate` positions there
/// (`LanguageParser.cpp:764`). The line is skipped by the parser but kept by
/// `SOURCELINE`, so this belongs to the scan and not to the line index. Found
/// by differential testing: 494 of 790 files under `ootest/` and `samples/`
/// open with `#!/usr/bin/env rexx`, and without this every one of them is
/// error 13.1 on line 1 here and rc 0 under `rexxc`.
///
/// An `INTERPRET` does not get the skip. `ArrayProgramSource::setup`
/// (`ProgramSource.cpp:594`) guards it with `interpretAdjust == 0`, and
/// measured, `interpret "#! nothing here"` is error 13.1 on `#` ('23'X) while
/// the identical text as line 1 of a file is accepted and the program runs on.
fn first_line(source: &ProgramSource) -> usize {
    if source.kind() == SourceKind::Program
        && source.line(1).is_some_and(|line| line.starts_with(b"#!"))
    {
        2
    } else {
        1
    }
}

/// What `locateToken` found, folding the C++'s `CharacterClass` return and
/// its out-parameter into one value.
enum Located {
    /// A character that starts a token. The scan position is on it.
    Normal(u8),
    /// A blank in a context where a blank may be an operator. The scan
    /// position is on it.
    SignificantBlank,
    ClauseEof,
    ClauseEol,
}

/// The numeric-symbol state machine of `scanSymbol` (`Scanner.cpp:1220`).
#[derive(Copy, Clone, PartialEq, Eq)]
enum ExpState {
    Start,
    Excluded,
    Digit,
    SPoint,
    Point,
    E,
    ESign,
    EDigit,
}

/// Which flavour of literal `scanLiteral` found, before its value is decoded.
#[derive(Copy, Clone, PartialEq, Eq)]
enum LiteralKind {
    String,
    Hex,
    Bin,
}

struct Scanner<'a> {
    source: &'a ProgramSource,
    /// The current line's bytes, terminator excluded: the C++'s `current`,
    /// with `line.len()` standing in for `currentLength`.
    line: &'a [u8],
    /// Absolute offset of `line[0]` in the retained source, which is what
    /// turns every in-line offset into a token span.
    line_start: usize,
    /// 1-based. Greater than `line_count()` means there are no lines left.
    line_number: usize,
    /// Offset within `line`.
    line_offset: usize,
    /// End of the last line's content. Stands in for end of file: the only
    /// window onto the retained text is one line at a time, deliberately, so
    /// that the line terminator rules live in `ProgramSource` alone.
    text_end: usize,

    symbols: SymbolTable,
    tokens: Vec<Token>,
    resources: Vec<ResourceBody>,

    /// `RESOURCE` and `END`, interned up front so the `::RESOURCE` test is an
    /// integer comparison. Interning is idempotent, so these are the same ids
    /// the keyword tables already hold.
    resource_id: SymbolId,
    end_id: SymbolId,

    /// Whether the token just produced makes a following blank significant.
    /// False at a clause start, which is what the C++ gets by passing a null
    /// previous token to `sourceNextToken`.
    prev_blank_significant: bool,
    /// Byte offset a scanner error is reported against: the current clause's
    /// start, or the position scanning began if the clause has no tokens yet.
    clause_start: usize,
    /// Whether the current clause has produced a token yet.
    clause_started: bool,
    /// Index in `tokens` of the current clause's first token.
    clause_first: usize,
}

impl<'a> Scanner<'a> {
    fn new(source: &'a ProgramSource, mut symbols: SymbolTable) -> Self {
        let text_end = source.line_span(source.line_count()).map_or(0, |s| s.end);
        let resource_id = symbols.intern("RESOURCE");
        let end_id = symbols.intern("END");
        let mut scanner = Scanner {
            source,
            line: &[],
            line_start: 0,
            line_number: 1,
            line_offset: 0,
            text_end,
            symbols,
            tokens: Vec::new(),
            resources: Vec::new(),
            resource_id,
            end_id,
            prev_blank_significant: false,
            clause_start: 0,
            clause_started: false,
            clause_first: 0,
        };
        scanner.position(first_line(source), 0);
        scanner
    }

    // Scanning position primitives, mirroring `LanguageParser.hpp:152`-`162`.

    /// `LanguageParser::position`: move to a line and offset, loading that
    /// line. A line number past the end leaves an empty line at `text_end`,
    /// which is how `moreLines` and `moreChars` both go false.
    fn position(&mut self, line: usize, offset: usize) {
        self.line_number = line;
        self.line_offset = offset;
        match (self.source.line_span(line), self.source.line(line)) {
            (Some(span), Some(bytes)) => {
                self.line_start = span.start;
                self.line = bytes;
            }
            _ => {
                self.line_start = self.text_end;
                self.line = &[];
            }
        }
    }

    fn next_line(&mut self) {
        self.position(self.line_number + 1, 0);
    }

    fn more_lines(&self) -> bool {
        self.line_number <= self.source.line_count()
    }

    fn more_chars(&self) -> bool {
        self.line_offset < self.line.len()
    }

    /// The byte at the scan position. Panics unless `more_chars`, which every
    /// caller has checked.
    fn get_char(&self) -> u8 {
        self.line[self.line_offset]
    }

    /// The byte after the scan position, or `None` at the end of the line.
    ///
    /// The C++ `followingChar` reads one byte past the line's content when
    /// the scan position is on the last character, so it sees the line
    /// terminator. `None` stands in for that: the three call sites test for
    /// `-`, `*` and symbol characters, and a terminator is none of those.
    fn following_char(&self) -> Option<u8> {
        self.line.get(self.line_offset + 1).copied()
    }

    fn step_position(&mut self) {
        self.line_offset += 1;
    }

    /// `truncateLine`: discard the rest of the line, which is how a `--`
    /// comment ends a clause.
    fn truncate_line(&mut self) {
        self.line_offset = self.line.len();
    }

    /// The scan position as an offset into the retained source.
    fn absolute(&self) -> usize {
        self.line_start + self.line_offset
    }

    // The scanner proper.

    fn run(&mut self) -> Result<(), ParseError> {
        while let Some(token) = self.source_next_token()? {
            self.prev_blank_significant = token.kind.makes_blank_significant();
            if token.kind.tag() == Tag::Eoc {
                if self.emit_eoc(token) {
                    self.scan_resource_if_directive()?;
                    self.clause_started = false;
                    self.clause_first = self.tokens.len();
                }
            } else {
                if !self.clause_started {
                    self.clause_started = true;
                    self.clause_start = token.span.start;
                    self.clause_first = self.tokens.len();
                }
                self.tokens.push(token);
            }
        }
        // End of file is a clause terminator too, subject to the same
        // collapsing, which is what keeps a trailing blank line or a trailing
        // `;` from producing an empty clause.
        let end = self.text_end..self.text_end;
        self.emit_eoc(Token {
            kind: TokenKind::Eoc,
            span: end,
        });
        Ok(())
    }

    /// Push a clause terminator unless it would be the first token or would
    /// follow another one. Returns whether it was pushed.
    fn emit_eoc(&mut self, token: Token) -> bool {
        match self.tokens.last() {
            None => false,
            Some(last) if last.kind.tag() == Tag::Eoc => false,
            Some(_) => {
                self.tokens.push(token);
                true
            }
        }
    }

    /// `LanguageParser::locateToken`: step over insignificant blanks,
    /// comments and line continuations, and report what stopped the scan.
    fn locate_token(&mut self, blanks_significant: bool) -> Result<Located, ParseError> {
        if !self.more_lines() {
            return Ok(Located::ClauseEof);
        } else if !self.more_chars() {
            return Ok(Located::ClauseEol);
        }

        while self.more_chars() {
            let inch = self.get_char();
            if inch == b' ' || inch == b'\t' {
                if blanks_significant {
                    return Ok(Located::SignificantBlank);
                }
                self.step_position();
            }
            // A `,` or `-` is a continuation only if nothing but blanks and
            // comments follows it on the line, so this has to look ahead and
            // be ready to back up. `--` is a line comment and is checked
            // first.
            else if inch == b',' || inch == b'-' {
                if inch == b'-' && self.following_char() == Some(b'-') {
                    self.truncate_line();
                    return Ok(Located::ClauseEol);
                }

                let start_offset = self.line_offset;
                let start_line = self.line_number;
                self.step_position();

                loop {
                    if !self.more_chars() {
                        // A continuation is functionally a blank, so if blanks
                        // are significant we are done here. Measured:
                        // `say "a"-` then `"b"` prints `a b`, while
                        // `say "a"||-` then `"b"` prints `ab`, because there
                        // the previous token is `||`.
                        if self.more_lines() {
                            self.next_line();
                            if blanks_significant {
                                return Ok(Located::SignificantBlank);
                            }
                        }
                        break;
                    }

                    let inch2 = self.get_char();
                    if inch2 == b' ' || inch2 == b'\t' {
                        self.step_position();
                        continue;
                    }
                    if inch2 == b'/' && self.following_char() == Some(b'*') {
                        // This may step over one or more lines. The
                        // continuation still holds.
                        self.scan_comment()?;
                        continue;
                    }
                    if inch2 == b'-' && self.following_char() == Some(b'-') {
                        self.truncate_line();
                        continue;
                    }

                    self.position(start_line, start_offset);
                    return Ok(Located::Normal(inch));
                }
            } else if inch == b'/' && self.following_char() == Some(b'*') {
                self.scan_comment()?;
            } else {
                return Ok(Located::Normal(inch));
            }
        }

        Ok(Located::ClauseEol)
    }

    /// `LanguageParser::scanComment`: step over a `/* */` comment, which
    /// nests and may span lines.
    ///
    /// The C++ also remembers the comment's opening line, solely for the
    /// error 6.1 substitution `Unmatched comment delimiter ("/*") on line N`;
    /// this phase does not produce substitutions.
    fn scan_comment(&mut self) -> Result<(), ParseError> {
        let mut level = 1;
        self.step_position();
        self.step_position();

        while level > 0 {
            if !self.more_chars() {
                self.next_line();
                if !self.more_lines() {
                    return Err(ParseError::new(6, 1, self.clause_start));
                }
                continue;
            }

            let inch = self.get_char();
            self.step_position();
            if inch == b'*' && self.more_chars() && self.get_char() == b'/' {
                self.step_position();
                level -= 1;
            } else if inch == b'/' && self.more_chars() && self.get_char() == b'*' {
                self.step_position();
                level += 1;
            }
        }
        Ok(())
    }

    /// `LanguageParser::nextSpecial`: consume the next character if it is
    /// `target`, extending `end` to cover it.
    ///
    /// Blanks are not significant here and comments are stepped over, which
    /// is why `1 = = 1.0`, `1 =/*c*/= 1.0` and a `=-` continuation all scan
    /// as `==`. All three measured against `build/bin/rexx`, which prints 0
    /// for each and 1 for `1 = 1.0`.
    fn next_special(&mut self, target: u8, end: &mut usize) -> Result<bool, ParseError> {
        match self.locate_token(false)? {
            Located::Normal(found) if found == target => {
                self.step_position();
                *end = self.absolute();
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    /// An operator that may instead be an assignment shortcut, as `+` is in
    /// `x += 1` (`RexxToken::checkAssignment`, `Token.cpp:95`).
    fn check_assignment(&mut self, op: Operator, end: &mut usize) -> Result<TokenKind, ParseError> {
        if self.next_special(b'=', end)? {
            Ok(TokenKind::Assignment(op))
        } else {
            Ok(TokenKind::Operator(op))
        }
    }

    /// `LanguageParser::sourceNextToken`: the next token, or `None` at end of
    /// file.
    fn source_next_token(&mut self) -> Result<Option<Token>, ParseError> {
        // While the clause has no tokens, the clause's reported position is
        // wherever scanning began, so that an error inside a comment or an
        // unterminated literal that starts the clause is reported there.
        if !self.clause_started {
            self.clause_start = self.absolute();
        }

        loop {
            let blanks_significant = self.prev_blank_significant;
            let located = self.locate_token(blanks_significant)?;
            let start = self.absolute();

            let inch = match located {
                Located::ClauseEof => return Ok(None),
                Located::ClauseEol => {
                    // `locateToken` only reports a line end with the position
                    // at the line's end, so the terminator's span is empty
                    // there.
                    debug_assert_eq!(self.line_offset, self.line.len());
                    self.next_line();
                    return Ok(Some(Token {
                        kind: TokenKind::Eoc,
                        span: start..start,
                    }));
                }
                Located::SignificantBlank => {
                    // Both sides have to agree. The left side is the previous
                    // token, already known; the right side is whatever the
                    // next real character starts. Measured: `abs (2.5)`
                    // prints `ABS 2.5` and `abs(2.5)` prints `2.5`.
                    let next = self.locate_token(false)?;
                    let significant = match next {
                        Located::Normal(c) => {
                            is_symbol_char(c) || c == b'"' || c == b'\'' || c == b'(' || c == b'['
                        }
                        _ => false,
                    };
                    if significant {
                        return Ok(Some(Token {
                            kind: TokenKind::Blank,
                            span: start..start + 1,
                        }));
                    }
                    // Not significant after all. Try again from here.
                    continue;
                }
                Located::Normal(inch) => inch,
            };

            if is_symbol_char(inch) {
                return Ok(Some(self.scan_symbol()?));
            }
            if inch == b'\'' || inch == b'"' {
                return Ok(Some(self.scan_literal()?));
            }

            // Some other special character. Step past it first, because most
            // of these need to look at what follows.
            self.step_position();
            let mut end = start + 1;
            let kind = match inch {
                b')' => TokenKind::RightParen,
                b']' => TokenKind::RightBracket,
                b'(' => TokenKind::LeftParen,
                b'[' => TokenKind::LeftBracket,
                // Continuation commas are resolved in `locateToken`, so a
                // comma here is an argument or template separator.
                b',' => TokenKind::Comma,
                b';' => TokenKind::Eoc,
                b':' => {
                    if self.next_special(b':', &mut end)? {
                        TokenKind::DColon
                    } else {
                        TokenKind::Colon
                    }
                }
                b'~' => {
                    if self.next_special(b'~', &mut end)? {
                        TokenKind::DTilde
                    } else {
                        TokenKind::Tilde
                    }
                }
                b'+' => self.check_assignment(Operator::Plus, &mut end)?,
                b'-' => self.check_assignment(Operator::Subtract, &mut end)?,
                b'%' => self.check_assignment(Operator::IntDiv, &mut end)?,
                b'/' => {
                    if self.next_special(b'/', &mut end)? {
                        self.check_assignment(Operator::Remainder, &mut end)?
                    } else {
                        self.check_assignment(Operator::Divide, &mut end)?
                    }
                }
                b'*' => {
                    if self.next_special(b'*', &mut end)? {
                        self.check_assignment(Operator::Power, &mut end)?
                    } else {
                        self.check_assignment(Operator::Multiply, &mut end)?
                    }
                }
                b'&' => {
                    if self.next_special(b'&', &mut end)? {
                        self.check_assignment(Operator::Xor, &mut end)?
                    } else {
                        self.check_assignment(Operator::And, &mut end)?
                    }
                }
                b'|' => {
                    if self.next_special(b'|', &mut end)? {
                        self.check_assignment(Operator::Concatenate, &mut end)?
                    } else {
                        self.check_assignment(Operator::Or, &mut end)?
                    }
                }
                // `=` is not an assignment shortcut: it is the assignment.
                b'=' => {
                    if self.next_special(b'=', &mut end)? {
                        TokenKind::Operator(Operator::StrictEqual)
                    } else {
                        TokenKind::Operator(Operator::Equal)
                    }
                }
                b'<' => {
                    let op = if self.next_special(b'<', &mut end)? {
                        if self.next_special(b'=', &mut end)? {
                            Operator::StrictLessThanEqual
                        } else {
                            Operator::StrictLessThan
                        }
                    } else if self.next_special(b'=', &mut end)? {
                        Operator::LessThanEqual
                    } else if self.next_special(b'>', &mut end)? {
                        Operator::LessThanGreaterThan
                    } else {
                        Operator::LessThan
                    };
                    TokenKind::Operator(op)
                }
                b'>' => {
                    let op = if self.next_special(b'>', &mut end)? {
                        if self.next_special(b'=', &mut end)? {
                            Operator::StrictGreaterThanEqual
                        } else {
                            Operator::StrictGreaterThan
                        }
                    } else if self.next_special(b'=', &mut end)? {
                        Operator::GreaterThanEqual
                    } else if self.next_special(b'<', &mut end)? {
                        Operator::GreaterThanLessThan
                    } else {
                        Operator::GreaterThan
                    };
                    TokenKind::Operator(op)
                }
                // `\`, and the two code-page logical-not bytes the
                // interpreter accepts as alternatives for it.
                b'\\' | 0xAA | 0xAC => {
                    let op = if self.next_special(b'=', &mut end)? {
                        if self.next_special(b'=', &mut end)? {
                            Operator::StrictBackslashEqual
                        } else {
                            Operator::BackslashEqual
                        }
                    } else if self.next_special(b'>', &mut end)? {
                        if self.next_special(b'>', &mut end)? {
                            Operator::StrictBackslashGreaterThan
                        } else {
                            Operator::BackslashGreaterThan
                        }
                    } else if self.next_special(b'<', &mut end)? {
                        if self.next_special(b'<', &mut end)? {
                            Operator::StrictBackslashLessThan
                        } else {
                            Operator::BackslashLessThan
                        }
                    } else {
                        Operator::Backslash
                    };
                    TokenKind::Operator(op)
                }
                // Anything else cannot appear in a program. Every byte from
                // 0x80 up lands here unless it is inside a literal or a
                // comment, which is why `bäc = 2` is error 13.1.
                _ => return Err(ParseError::new(13, 1, self.clause_start)),
            };

            return Ok(Some(Token {
                kind,
                span: start..end,
            }));
        }
    }

    /// `LanguageParser::scanSymbol`: consume a symbol and classify it.
    ///
    /// The state machine decides whether the symbol is a number, which also
    /// decides whether an `E` may be followed by a sign that belongs to the
    /// symbol. Measured: `say 1e+5` prints `1E+5`, while with `y = 5`,
    /// `say 1e+y` fails with `Nonnumeric value ("1E") used in arithmetic
    /// operation`, so the symbol there is `1E` and the `+` is an operator.
    fn scan_symbol(&mut self) -> Result<Token, ParseError> {
        let mut state = ExpState::Start;
        // Position of an exponent sign, kept so the scan can back up over it
        // if no digits follow.
        let mut eoffset: Option<usize> = None;
        let start = self.line_offset;
        let span_start = self.absolute();
        let mut dot_count = 0usize;

        let mut inch = self.get_char();
        loop {
            if inch == b'.' {
                dot_count += 1;
            }

            state = match state {
                ExpState::Start => {
                    if inch.is_ascii_digit() {
                        ExpState::Digit
                    } else if inch == b'.' {
                        ExpState::SPoint
                    } else {
                        ExpState::Excluded
                    }
                }
                ExpState::Digit => {
                    if inch == b'.' {
                        ExpState::Point
                    } else if inch == b'E' || inch == b'e' {
                        ExpState::E
                    } else if !inch.is_ascii_digit() {
                        ExpState::Excluded
                    } else {
                        ExpState::Digit
                    }
                }
                ExpState::SPoint => {
                    if inch.is_ascii_digit() {
                        ExpState::Point
                    } else {
                        ExpState::Excluded
                    }
                }
                ExpState::Point => {
                    if inch == b'E' || inch == b'e' {
                        ExpState::E
                    } else if !inch.is_ascii_digit() {
                        ExpState::Excluded
                    } else {
                        ExpState::Point
                    }
                }
                // The sign case is handled at the end of the loop, because a
                // sign either continues the symbol or terminates it.
                ExpState::E => {
                    if inch.is_ascii_digit() {
                        ExpState::EDigit
                    } else {
                        ExpState::E
                    }
                }
                ExpState::ESign => {
                    if inch.is_ascii_digit() {
                        ExpState::EDigit
                    } else {
                        ExpState::Excluded
                    }
                }
                ExpState::EDigit => {
                    if inch.is_ascii_digit() {
                        ExpState::EDigit
                    } else {
                        ExpState::Excluded
                    }
                }
                ExpState::Excluded => ExpState::Excluded,
            };

            self.step_position();

            // Stepped past an exponent sign but found no exponent: the sign
            // was not part of the symbol after all.
            if let Some(sign) = eoffset
                && state == ExpState::Excluded
            {
                self.line_offset = sign;
                break;
            }

            if !self.more_chars() {
                break;
            }

            inch = self.get_char();
            if is_symbol_char(inch) {
                continue;
            }

            if state == ExpState::E && (inch == b'+' || inch == b'-') {
                // The C++ guards this with `haveNextChar()`, which tests the
                // same thing as `moreChars()` and so is always true here
                // (`LanguageParser.hpp:159`-`160`). A sign at the end of the
                // line therefore falls through to reading the line
                // terminator, which is not a symbol character, and backs up.
                let sign = self.line_offset;
                eoffset = Some(sign);
                self.step_position();
                state = ExpState::ESign;
                match self.more_chars().then(|| self.get_char()) {
                    Some(next) if is_symbol_char(next) => {
                        inch = next;
                        continue;
                    }
                    _ => {
                        state = ExpState::Excluded;
                        self.line_offset = sign;
                        break;
                    }
                }
            } else {
                // A non-symbol character ends the symbol, whatever state the
                // number machine is in.
                break;
            }
        }

        let length = self.line_offset - start;
        let bytes = &self.line[start..self.line_offset];

        if length > MAX_SYMBOL_LENGTH {
            return Err(ParseError::new(30, 1, self.clause_start));
        }

        // Infallible: `characterTable` admits only ASCII, and the one byte a
        // symbol can hold that it does not admit is an exponent sign, also
        // ASCII.
        let text = std::str::from_utf8(bytes)
            .expect("a symbol's bytes are ASCII: characterTable is zero for 0x80 to 0xFF");
        let id = self.symbols.intern(text);

        // Classification works on the source bytes because `translateChar`
        // only upcases: `.` and the digits are unchanged by it.
        let first = bytes[0];
        let class = if length == 1 && first == b'.' {
            SymbolClass::Dummy
        } else if first.is_ascii_digit() {
            SymbolClass::Constant
        } else if first == b'.' {
            if state == ExpState::Excluded {
                SymbolClass::DotSymbol
            } else {
                SymbolClass::Constant
            }
        } else if dot_count == 0 {
            SymbolClass::Variable
        } else if dot_count == 1 && bytes[length - 1] == b'.' {
            SymbolClass::Stem
        } else {
            SymbolClass::Compound
        };

        Ok(Token {
            kind: TokenKind::Symbol { id, class },
            span: span_start..self.absolute(),
        })
    }

    /// `LanguageParser::scanLiteral`: consume a quoted literal and decode its
    /// value.
    ///
    /// The value is not a slice of the source: doubled delimiters collapse to
    /// one, and a `'…'x` or `'…'b` suffix packs the text down to bytes.
    fn scan_literal(&mut self) -> Result<Token, ParseError> {
        let span_start = self.absolute();
        let delimiter = self.get_char();
        // Just past the opening quote.
        let start = self.line_offset + 1;
        let mut doubled = 0usize;
        let mut kind = LiteralKind::String;

        let literal_end = loop {
            // First time round this steps over the opening quote.
            self.step_position();

            if !self.more_chars() {
                // A literal never spans lines, so running out of characters
                // is the end of it. Measured: 6.2 for `'`, 6.3 for `"`.
                let sub = if delimiter == b'\'' { 2 } else { 3 };
                return Err(ParseError::new(6, sub, self.clause_start));
            }

            if self.get_char() == delimiter {
                let end = self.line_offset - 1;
                self.step_position();
                if !self.more_chars() || self.get_char() != delimiter {
                    break end;
                }
                // A doubled delimiter: still inside the literal, and the
                // value will be one byte shorter than the text.
                doubled += 1;
            }
        };

        // A trailing `x` or `b` marks a hex or binary literal, but only if it
        // is not the start of a longer symbol.
        if self.more_chars() {
            let inch = self.get_char();
            let marker = match inch {
                b'x' | b'X' => Some(LiteralKind::Hex),
                b'b' | b'B' => Some(LiteralKind::Bin),
                _ => None,
            };
            if let Some(marker) = marker
                && !self.following_char().is_some_and(is_symbol_char)
            {
                self.step_position();
                kind = marker;
            }
        }

        // `literal_end` indexes the last character and `start` the first, so
        // an empty literal has `literal_end + 1 == start`.
        let length = literal_end + 1 - start;
        let data = &self.line[start..start + length];

        let value: Vec<u8> = match kind {
            LiteralKind::Hex => pack_hex_literal(data, self.clause_start)?,
            LiteralKind::Bin => pack_binary_literal(data, self.clause_start)?,
            LiteralKind::String if doubled == 0 => data.to_vec(),
            LiteralKind::String => {
                let mut out = Vec::with_capacity(length - doubled);
                let mut j = 0;
                while out.len() < length - doubled {
                    let byte = data[j];
                    if byte == delimiter {
                        // Step one extra over the second of the pair.
                        j += 1;
                    }
                    out.push(byte);
                    j += 1;
                }
                out
            }
        };

        Ok(Token {
            kind: TokenKind::Literal {
                value: value.into_boxed_slice(),
            },
            span: span_start..self.absolute(),
        })
    }

    /// If the clause that just closed is a well-formed `::RESOURCE`
    /// directive, copy its body verbatim up to the end marker.
    ///
    /// A malformed one, `::resource data junk`, is left alone: the C++
    /// rejects it in the directive parser before it ever reads a line, so the
    /// lines that follow are ordinary Rexx there too.
    fn scan_resource_if_directive(&mut self) -> Result<(), ParseError> {
        let eoc = self.tokens.len() - 1;
        // This runs for every clause in the program, so it tests the cheapest
        // discriminator first and allocates nothing for the overwhelming
        // majority that are not directives. A clause's first token is never a
        // blank, because a blank at a clause start is not significant.
        if self.tokens[self.clause_first].kind.tag() != Tag::DColon {
            return Ok(());
        }

        // `nextReal` skips blanks, and `::RESOURCE DATA` has a significant
        // blank in it, so the shape has to be matched over the real tokens.
        // Either shape has at most five, so this needs no growth.
        let mut real = [0usize; 5];
        let mut count = 0;
        for index in self.clause_first..eoc {
            if self.tokens[index].kind.tag() == Tag::Blank {
                continue;
            }
            if count == real.len() {
                return Ok(());
            }
            real[count] = index;
            count += 1;
        }
        if count != 3 && count != 5 {
            return Ok(());
        }
        match self.tokens[real[1]].kind {
            TokenKind::Symbol { id, .. } if id == self.resource_id => {}
            _ => return Ok(()),
        }
        // The resource name, which the scanner does not otherwise need.
        if self.token_value(real[2]).is_none() {
            return Ok(());
        }

        let marker = if count == 5 {
            // The only sub-keyword a `::RESOURCE` accepts is `END`, and it
            // must be a symbol rather than a literal.
            match self.tokens[real[3]].kind {
                TokenKind::Symbol { id, .. } if id == self.end_id => {}
                _ => return Ok(()),
            }
            match self.token_value(real[4]) {
                Some(value) => value,
                None => return Ok(()),
            }
        } else {
            DEFAULT_RESOURCE_END.to_vec()
        };

        // `conditionalNextLine`: if the clause ended at a `;` rather than at a
        // line end, the rest of that line is neither parsed nor part of the
        // body. Measured: `::resource data; say 'x'` gets rc 0 from `rexxc`
        // and yields a one-line resource, so the `say` is simply skipped.
        if self.line_offset != 0 {
            self.next_line();
        }

        let directive = real[0];
        let mut lines = Vec::new();
        loop {
            if !self.more_lines() {
                // Without this the rest of the program would be swallowed.
                // Measured: error 99.943, reported against the directive.
                return Err(ParseError::new(99, 943, self.tokens[directive].span.start));
            }
            if self.check_marker(&marker) {
                self.next_line();
                break;
            }
            lines.push(self.line_start..self.line_start + self.line.len());
            self.next_line();
        }

        self.resources.push(ResourceBody { directive, lines });
        Ok(())
    }

    /// A token's value the way `RexxToken::value()` gives it: upcased for a
    /// symbol, verbatim for a literal, and nothing for anything else.
    fn token_value(&self, index: usize) -> Option<Vec<u8>> {
        match &self.tokens[index].kind {
            TokenKind::Symbol { id, .. } => Some(self.symbols.name(*id).as_bytes().to_vec()),
            TokenKind::Literal { value } => Some(value.to_vec()),
            _ => None,
        }
    }

    /// `LanguageParser::checkMarker`: whether the current line *begins* with
    /// `marker`.
    ///
    /// A prefix match, not an equality test. Measured: with
    /// `::resource d2 end 'STOP'`, a body line `STOPPING? no, prefix match`
    /// ends the resource.
    fn check_marker(&self, marker: &[u8]) -> bool {
        marker.len() <= self.line.len() && &self.line[..marker.len()] == marker
    }
}

/// `LanguageParser::packHexLiteral`: validate a hex literal's grouping and
/// pack it down to bytes.
///
/// Whitespace may separate groups but not sit at either end, and every group
/// after the first must hold an even number of digits. The first group may be
/// odd, which is what makes `'a'x` legal.
fn pack_hex_literal(data: &[u8], clause_start: usize) -> Result<Vec<u8>, ParseError> {
    if data.is_empty() {
        return Ok(Vec::new());
    }

    // Right to left, because that is what makes the group boundaries easy to
    // check: the odd group, if any, is the leftmost one.
    let mut group_count = 0usize;
    let mut nibble_count = 0usize;
    for i in (1..=data.len()).rev() {
        let byte = data[i - 1];
        if byte == b' ' || byte == b'\t' {
            if i == 1 || i == data.len() {
                return Err(ParseError::new(15, 1, clause_start));
            } else if !group_count.is_multiple_of(2) {
                return Err(ParseError::new(15, 5, clause_start));
            }
            group_count = 0;
        } else {
            group_count += 1;
            nibble_count += 1;
        }
    }

    let mut nibbles_this_byte = 2 - (nibble_count % 2);
    let character_count = nibble_count / 2 + (nibble_count % 2);
    let mut out = Vec::with_capacity(character_count);
    let mut pos = 0usize;

    for _ in 0..character_count {
        let mut byte = 0u8;
        while data[pos] == b' ' || data[pos] == b'\t' {
            pos += 1;
        }
        for _ in 0..nibbles_this_byte {
            // In range because the validation above accounted for every
            // non-blank character and rejected trailing whitespace.
            let nibble = data[pos];
            pos += 1;
            let value = match nibble {
                b'0'..=b'9' => nibble - b'0',
                b'a'..=b'f' => nibble - b'a' + 10,
                b'A'..=b'F' => nibble - b'A' + 10,
                _ => return Err(ParseError::new(15, 3, clause_start)),
            };
            byte = (byte << 4) + value;
        }
        // Only the first byte can be short.
        nibbles_this_byte = 2;
        out.push(byte);
    }

    Ok(out)
}

/// `LanguageParser::packBinaryLiteral`: validate a binary literal's grouping
/// and pack it down to bytes.
///
/// As for hex, but the groups are four bits wide and the first byte may hold
/// fewer than eight.
fn pack_binary_literal(data: &[u8], clause_start: usize) -> Result<Vec<u8>, ParseError> {
    if data.is_empty() {
        return Ok(Vec::new());
    }

    let mut group_count = 0usize;
    let mut bit_count = 0usize;
    for i in (1..=data.len()).rev() {
        let byte = data[i - 1];
        if byte == b' ' || byte == b'\t' {
            if i == 1 || i == data.len() {
                return Err(ParseError::new(15, 2, clause_start));
            } else if !group_count.is_multiple_of(4) {
                return Err(ParseError::new(15, 6, clause_start));
            }
            group_count = 0;
        } else {
            group_count += 1;
            bit_count += 1;
        }
    }

    let mut bits_this_byte = bit_count % 8;
    let character_count = bit_count / 8 + usize::from(bits_this_byte != 0);
    if bits_this_byte == 0 {
        bits_this_byte = 8;
    }

    let mut out = Vec::with_capacity(character_count);
    let mut pos = 0usize;

    for _ in 0..character_count {
        let mut byte = 0u8;
        for _ in 0..bits_this_byte {
            // In range for the same reason as in the hex packer.
            let mut bit = data[pos];
            pos += 1;
            while bit == b' ' || bit == b'\t' {
                bit = data[pos];
                pos += 1;
            }
            byte <<= 1;
            if bit == b'1' {
                byte += 1;
            } else if bit != b'0' {
                return Err(ParseError::new(15, 4, clause_start));
            }
        }
        bits_this_byte = 8;
        out.push(byte);
    }

    Ok(out)
}
