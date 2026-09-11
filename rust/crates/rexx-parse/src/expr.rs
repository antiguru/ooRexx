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

//! The expression grammar: hand-written recursive descent, per D10.

use std::ops::Range;

use crate::ast::{CallTarget, Expr, ExprKind, PrefixOp};
use crate::token::{
    Operator, ParseCtx, ParseError, SymbolClass, SymbolId, Tag, Token, TokenCursor, TokenKind,
};

/// How many levels of nested expression the parser will descend before
/// raising `11.1`, "Insufficient control stack space; cannot continue
/// execution", instead of recursing one level deeper.
pub const MAX_EXPR_DEPTH: u32 = 50_000;

/// The shallowest native stack overflow measured for any recursion
/// `MAX_EXPR_DEPTH` guards, on the 512 MiB thread, debug: 88,800 levels of
/// grouping parentheses parsed and 89,000 aborted, against 91,948 and 92,337
/// for nested calls.
const MEASURED_NATIVE_CLIFF: u32 = 88_800;

const _: () = assert!(
    MAX_EXPR_DEPTH < MEASURED_NATIVE_CLIFF,
    "MAX_EXPR_DEPTH must stay below the measured native cliff, or the counter never fires \
     and a deep expression aborts the process instead of raising 11.1"
);

/// Dyadic operator precedence, ported from `RexxToken::precedence`
/// (`Token.cpp:111`).
fn precedence(op: Operator) -> u8 {
    match op {
        Operator::Backslash => 8,
        Operator::Power => 7,
        Operator::Multiply | Operator::Divide | Operator::IntDiv | Operator::Remainder => 6,
        Operator::Plus | Operator::Subtract => 5,
        Operator::Abuttal | Operator::Concatenate | Operator::Blank => 4,
        Operator::Equal
        | Operator::BackslashEqual
        | Operator::GreaterThan
        | Operator::BackslashGreaterThan
        | Operator::LessThan
        | Operator::BackslashLessThan
        | Operator::GreaterThanEqual
        | Operator::LessThanEqual
        | Operator::StrictEqual
        | Operator::StrictBackslashEqual
        | Operator::StrictGreaterThan
        | Operator::StrictBackslashGreaterThan
        | Operator::StrictLessThan
        | Operator::StrictBackslashLessThan
        | Operator::StrictGreaterThanEqual
        | Operator::StrictLessThanEqual
        | Operator::LessThanGreaterThan
        | Operator::GreaterThanLessThan => 3,
        Operator::And => 2,
        Operator::Or | Operator::Xor => 1,
    }
}

/// Which tokens end the expression being parsed.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) struct Terminators(u32);

// Tasks 3.6 and 3.7 pick a set per instruction. Only `EOC`, `RIGHT` and
// `SQRIGHT` have a caller inside the grammar itself.
impl Terminators {
    /// End of clause. Carried for fidelity with `TERM_EOC` and never read:
    /// `RexxToken::isTerminator` returns true for an end of clause and for a
    /// comma whatever the flags say (`Token.cpp:194`-`201`).
    pub(crate) const EOC: Terminators = Terminators(0x0000_0001);
    pub(crate) const RIGHT: Terminators = Terminators(0x0000_0002);
    pub(crate) const SQRIGHT: Terminators = Terminators(0x0000_0004);
    pub(super) const TO: Terminators = Terminators(0x0000_0008);
    const BY: Terminators = Terminators(0x0000_0010);
    const FOR: Terminators = Terminators(0x0000_0020);
    /// `TERM_WHILE` picks up both `WHILE` and `UNTIL` (`Token.cpp:252`).
    const WHILE: Terminators = Terminators(0x0000_0040);
    const WITH: Terminators = Terminators(0x0000_0100);
    const THEN: Terminators = Terminators(0x0000_0200);
    /// Gates every keyword check. Without it a symbol never terminates,
    /// however it is spelled.
    pub(super) const KEYWORD: Terminators = Terminators(0x1000_0000);

    /// `TERM_CONTROL`: a `DO`/`LOOP` control expression.
    pub(crate) const CONTROL: Terminators =
        Terminators(Self::KEYWORD.0 | Self::TO.0 | Self::BY.0 | Self::FOR.0 | Self::WHILE.0);
    /// `TERM_COND`: a `WHILE`/`UNTIL` conditional.
    pub(crate) const COND: Terminators = Terminators(Self::KEYWORD.0 | Self::WHILE.0);
    /// `TERM_OVER`: a `DO ... OVER` collection expression. `OVER` itself is
    /// not a terminator: `isTerminator` has no case for it.
    pub(crate) const OVER: Terminators = Terminators(Self::KEYWORD.0 | Self::FOR.0 | Self::WHILE.0);
    /// `TERM_IF`: an `IF` or `WHEN` condition.
    pub(crate) const IF: Terminators = Terminators(Self::KEYWORD.0 | Self::THEN.0);
    /// `PARSE VALUE`'s source expression, which stops at `WITH`.
    pub(crate) const PARSE_WITH: Terminators = Terminators(Self::KEYWORD.0 | Self::WITH.0);

    fn has(self, flag: Terminators) -> bool {
        self.0 & flag.0 != 0
    }

    /// The union of two sets. Every real set is a named constant, so this
    /// exists only for the test that builds one the C++ never would.
    #[cfg(test)]
    pub(super) const fn with(self, flag: Terminators) -> Terminators {
        Terminators(self.0 | flag.0)
    }
}

// Positions in the `SUB_KEYWORDS` table, which `KeywordSet::index_of` returns.
// An entry's position is its meaning, so these are indices and not spellings.
// `tests::sub_keyword_indices_still_name_the_right_spellings` pins each one.
const SUBKEY_BY: usize = 5;
const SUBKEY_FOR: usize = 17;
const SUBKEY_THEN: usize = 41;
const SUBKEY_TO: usize = 42;
const SUBKEY_UNTIL: usize = 44;
const SUBKEY_WHILE: usize = 48;
const SUBKEY_WITH: usize = 49;

/// Parses one required expression, stopping at `term`.
pub(crate) fn parse_expr(
    ctx: &ParseCtx,
    cursor: &mut TokenCursor,
    term: Terminators,
    missing: u16,
) -> Result<Expr, ParseError> {
    let mut parser = Parser::new(ctx, cursor);
    match parser.expression(term)? {
        Some(expr) => Ok(expr),
        None => Err(parser.error(35, missing)),
    }
}

/// Parses an expression that may be absent, stopping at `term`.
pub(crate) fn parse_expression(
    ctx: &ParseCtx,
    cursor: &mut TokenCursor,
    term: Terminators,
) -> Result<Option<Expr>, ParseError> {
    Parser::new(ctx, cursor).expression(term)
}

/// Parses a conditional, where a comma-separated list is a logical AND.
pub(crate) fn parse_logical(
    ctx: &ParseCtx,
    cursor: &mut TokenCursor,
    term: Terminators,
    missing: u16,
) -> Result<Expr, ParseError> {
    Parser::new(ctx, cursor).logical(term, missing)
}

/// Parses a `WHEN`'s clause inside `SELECT CASE`: one or more values to compare
/// against the `SELECT`'s own expression.
pub(crate) fn parse_case_when_list(
    ctx: &ParseCtx,
    cursor: &mut TokenCursor,
    term: Terminators,
) -> Result<Vec<Expr>, ParseError> {
    Parser::new(ctx, cursor).comma_list(term, 934)
}

/// Parses an argument list up to `closer`, which is consumed.
pub(crate) fn parse_arg_list(
    ctx: &ParseCtx,
    cursor: &mut TokenCursor,
    closer: Option<Tag>,
) -> Result<Vec<Option<Expr>>, ParseError> {
    Parser::new(ctx, cursor).arg_list(closer)
}

/// Parses the expression inside a `(`, whose `(` is already consumed, and
/// consumes the `)`.
pub(crate) fn parse_paren_expression(
    ctx: &ParseCtx,
    cursor: &mut TokenCursor,
) -> Result<Option<Expr>, ParseError> {
    let mut parser = Parser::new(ctx, cursor);
    let inner = parser.subexpression(Terminators::RIGHT)?;
    if parser.peek_tag() != Some(Tag::RightParen) {
        return Err(parser.unmatched(false));
    }
    parser.advance();
    Ok(inner)
}

/// Parses the restricted expression form that `RAISE`, `FORWARD`, `USE ARG`
/// defaults and `ADDRESS ... WITH` accept.
pub(crate) fn parse_constant_expression(
    ctx: &ParseCtx,
    cursor: &mut TokenCursor,
) -> Result<Option<Expr>, ParseError> {
    let mut parser = Parser::new(ctx, cursor);
    parser.skip_blanks();
    let Some(token) = parser.peek() else {
        return Ok(None);
    };
    let span = token.span.clone();
    match &token.kind {
        TokenKind::Literal { value } => {
            let value = value.clone();
            parser.advance();
            Ok(Some(Expr::new(ExprKind::Literal(value), span)))
        }
        // `isConstant`: a symbol that is not a variable, a stem or a
        // compound, so a number, a lone period, or a dot symbol.
        TokenKind::Symbol { id, class }
            if !matches!(
                class,
                SymbolClass::Variable | SymbolClass::Stem | SymbolClass::Compound
            ) =>
        {
            let kind = symbol_kind(*id, *class);
            parser.advance();
            Ok(Some(Expr::new(kind, span)))
        }
        TokenKind::LeftParen => {
            parser.advance();
            // A comma list is allowed in here, which is why this is
            // `parseFullSubExpression` and not `parseSubExpression`.
            let inner = parser.full_subexpression(Terminators::RIGHT)?;
            if parser.peek_tag() != Some(Tag::RightParen) {
                return Err(parser.unmatched(false));
            }
            parser.advance();
            Ok(inner)
        }
        _ => Err(parser.error(35, 1)),
    }
}

/// Parses a term that is a message send, or reports that the clause does not
/// start with one.
pub(crate) fn parse_message_term(
    ctx: &ParseCtx,
    cursor: &mut TokenCursor,
) -> Result<Option<Expr>, ParseError> {
    Parser::new(ctx, cursor).message_term()
}

/// Parses a `PARSE` or `USE ARG` assignment target: a message term, or a
/// variable.
pub(crate) fn parse_variable_or_message_term(
    ctx: &ParseCtx,
    cursor: &mut TokenCursor,
) -> Result<Option<Expr>, ParseError> {
    // The message-term attempt runs on a cursor of its own, because it may
    // consume tokens and come back empty and the fallback below has to see the
    // first token again. The C++ resets its position for the same reason.
    let mut trial = TokenCursor::new(cursor.start()..cursor.end());
    while trial.position() < cursor.position() {
        trial.advance();
    }
    if let Some(term) = Parser::new(ctx, &mut trial).message_term()? {
        *cursor = trial;
        // The C++ converts the message into an assignment message here. That
        // rewrite belongs to whoever executes it: the name it would append
        // `=` to is `ExprKind::Message::name`, and the instruction node
        // already records that this term is an assignment target.
        return Ok(Some(term));
    }
    let mut parser = Parser::new(ctx, cursor);
    parser.skip_blanks();
    let Some(token) = parser.peek() else {
        return Ok(None);
    };
    let span = token.span.clone();
    let TokenKind::Symbol { id, class } = &token.kind else {
        return Ok(None);
    };
    let (id, class) = (*id, *class);
    need_variable(ctx, id, class, parser.clause_byte)?;
    parser.advance();
    Ok(Some(Expr::new(symbol_kind(id, class), span)))
}

/// Raises the error a non-variable symbol gets where a variable was required.
pub(crate) fn need_variable(
    ctx: &ParseCtx,
    id: SymbolId,
    class: SymbolClass,
    byte: usize,
) -> Result<(), ParseError> {
    if matches!(
        class,
        SymbolClass::Variable | SymbolClass::Stem | SymbolClass::Compound
    ) {
        return Ok(());
    }
    // `Error_Invalid_variable_period` is 31.3 and
    // `Error_Invalid_variable_number` is 31.2.
    let sub = if ctx.symbols.name(id).starts_with('.') {
        3
    } else {
        2
    };
    Err(ParseError::new(31, sub, byte))
}

/// One expression parse in progress.
struct Parser<'a, 'c> {
    ctx: &'a ParseCtx<'a>,
    cursor: &'c mut TokenCursor,
    /// The byte offset every error is reported against.
    clause_byte: usize,
    /// How many levels of nested expression are currently open: a grouping
    /// `(...)` through `subterm`'s `TokenKind::LeftParen` arm, plus an
    /// argument list through `arg_list`, counted together against
    /// `MAX_EXPR_DEPTH` before descending one level further, so either raises
    /// `11.1` instead of exhausting the native stack.
    expr_depth: u32,
}

impl<'a, 'c> Parser<'a, 'c> {
    fn new(ctx: &'a ParseCtx<'a>, cursor: &'c mut TokenCursor) -> Self {
        let clause_byte = ctx
            .tokens
            .get(cursor.start())
            .map_or(0, |token| token.span.start);
        Parser {
            ctx,
            cursor,
            clause_byte,
            expr_depth: 0,
        }
    }

    fn error(&self, code: u16, sub: u16) -> ParseError {
        ParseError::new(code, sub, self.clause_byte)
    }

    /// Error 36's two sub-numbers, for an opener with no matching closer.
    fn unmatched(&self, bracket: bool) -> ParseError {
        self.error(36, if bracket { 902 } else { 901 })
    }

    /// The next token, or `None` at the end of the clause.
    fn peek(&self) -> Option<&'a Token> {
        self.cursor.peek().map(|i| &self.ctx.tokens[i])
    }

    fn peek_tag(&self) -> Option<Tag> {
        self.peek().map(|token| token.kind.tag())
    }

    fn advance(&mut self) -> Option<&'a Token> {
        self.cursor.advance().map(|i| &self.ctx.tokens[i])
    }

    /// Steps past any blank operator, which is what `nextReal` followed by
    /// `previousToken` leaves behind.
    fn skip_blanks(&mut self) {
        while self.peek_tag() == Some(Tag::Blank) {
            self.cursor.advance();
        }
    }

    /// The next token that is not a blank, without consuming anything.
    fn peek_real(&self) -> Option<&'a Token> {
        let mut i = self.cursor.peek()?;
        while i < self.ctx.tokens.len() && self.ctx.tokens[i].kind.tag() == Tag::Blank {
            i += 1;
        }
        // The blank run cannot reach past the clause, because the scanner never
        // emits a blank before a clause terminator, so this index is still
        // inside the cursor's range.
        self.ctx.tokens.get(i)
    }

    /// `RexxToken::isTerminator` (`Token.cpp:189`). `None` is an end of
    /// clause, which always terminates.
    fn is_terminator(&self, token: Option<&Token>, term: Terminators) -> bool {
        let Some(token) = token else { return true };
        match &token.kind {
            TokenKind::Eoc | TokenKind::Comma => true,
            TokenKind::RightParen => term.has(Terminators::RIGHT),
            TokenKind::RightBracket => term.has(Terminators::SQRIGHT),
            // Only a simple variable can be a keyword terminator, so `TO.`
            // and `1` never stop an expression however the flags are set.
            TokenKind::Symbol {
                id,
                class: SymbolClass::Variable,
            } if term.has(Terminators::KEYWORD) => {
                match self.ctx.keywords.sub_keywords.index_of(*id) {
                    Some(SUBKEY_TO) => term.has(Terminators::TO),
                    Some(SUBKEY_BY) => term.has(Terminators::BY),
                    Some(SUBKEY_FOR) => term.has(Terminators::FOR),
                    Some(SUBKEY_WHILE | SUBKEY_UNTIL) => term.has(Terminators::WHILE),
                    Some(SUBKEY_WITH) => term.has(Terminators::WITH),
                    Some(SUBKEY_THEN) => term.has(Terminators::THEN),
                    _ => false,
                }
            }
            _ => false,
        }
    }

    /// The byte extent of tokens `from` up to the cursor's current position.
    fn extent(&self, from: usize) -> Range<usize> {
        let to = self.cursor.position();
        let start = self
            .ctx
            .tokens
            .get(from)
            .map_or(self.clause_byte, |token| token.span.start);
        let end = if to > from {
            self.ctx.tokens[to - 1].span.end
        } else {
            start
        };
        start..end
    }

    /// `parseExpression`: skip to the first real token, then parse a full
    /// subexpression, where commas build a list.
    fn expression(&mut self, term: Terminators) -> Result<Option<Expr>, ParseError> {
        self.skip_blanks();
        self.full_subexpression(term)
    }

    /// `parseFullSubExpression` (`LanguageParser.cpp:2753`): one or more
    /// comma-separated subexpressions, which become an array-building list
    /// when there is more than one.
    fn full_subexpression(&mut self, term: Terminators) -> Result<Option<Expr>, ParseError> {
        let from = self.cursor.position();
        let mut parts: Vec<Option<Expr>> = Vec::new();
        loop {
            parts.push(self.subexpression(term)?);
            if self.peek_tag() == Some(Tag::Comma) {
                self.advance();
            } else {
                break;
            }
        }
        if parts.len() == 1 {
            // A single subexpression is itself, absent or not.
            return Ok(parts.pop().expect("just checked the length"));
        }
        // Trailing omitted elements are kept here, unlike in an argument list:
        // measured, `(1,)~size` is 2 and `(1,,)~size` is 3, where `f(1,)`
        // passes one argument. `parseFullSubExpression` returns `total` where
        // `parseArgList` returns `realcount`.
        let extent = self.extent(from);
        Ok(Some(Expr::new(ExprKind::List(parts), extent)))
    }

    /// `parseLogical` (`LanguageParser.cpp:4264`).
    fn logical(&mut self, term: Terminators, missing: u16) -> Result<Expr, ParseError> {
        let from = {
            self.skip_blanks();
            self.cursor.position()
        };
        let mut parts = self.comma_list(term, missing)?;
        if parts.len() == 1 {
            return Ok(parts.pop().expect("just checked the length"));
        }
        let extent = self.extent(from);
        Ok(Expr::new(ExprKind::Logical(parts), extent))
    }

    /// The comma-separated loop that `parseLogical` (`LanguageParser.cpp:4264`)
    /// and `parseCaseWhenList` (`:3168`) share.
    fn comma_list(&mut self, term: Terminators, missing: u16) -> Result<Vec<Expr>, ParseError> {
        self.skip_blanks();
        let mut parts: Vec<Expr> = Vec::new();
        loop {
            let Some(part) = self.subexpression(term)? else {
                return Err(self.error(35, missing));
            };
            parts.push(part);
            if self.peek_tag() == Some(Tag::Comma) {
                self.advance();
            } else {
                break;
            }
        }
        Ok(parts)
    }

    /// `parseSubExpression` (`LanguageParser.cpp:2812`): a term, then dyadic
    /// operators by precedence. A comma always terminates.
    fn subexpression(&mut self, term: Terminators) -> Result<Option<Expr>, ParseError> {
        let Some(left) = self.message_subterm(term)? else {
            return Ok(None);
        };
        self.binary_rest(left, 1, term).map(Some)
    }

    /// The precedence loop.
    fn binary_rest(
        &mut self,
        mut left: Expr,
        min_prec: u8,
        term: Terminators,
    ) -> Result<Expr, ParseError> {
        loop {
            let Some(token) = self.peek() else {
                return Ok(left);
            };
            if self.is_terminator(Some(token), term) {
                return Ok(left);
            }
            let op = match &token.kind {
                // A term where an operator was expected is an abuttal. The
                // C++ synthesises a zero-length operator token and pushes the
                // term back (`LanguageParser.cpp:2878`), so nothing is
                // consumed here either.
                TokenKind::Symbol { .. } | TokenKind::Literal { .. } | TokenKind::LeftParen => {
                    Operator::Abuttal
                }
                TokenKind::Blank => {
                    // A blank next to a terminator is not an operator, which
                    // is what stops `do i = 1 to 3 while x` concatenating `3`
                    // with `WHILE`. The C++ consumes the blank even then,
                    // because `nextReal` moved past it, so the blanks are
                    // skipped rather than left for the caller.
                    if self.is_terminator(self.peek_real(), term) {
                        self.skip_blanks();
                        return Ok(left);
                    }
                    Operator::Blank
                }
                // `\` is prefix-only. This is why `a \(1 = 2)` is 35.1 rather
                // than a concatenation.
                TokenKind::Operator(Operator::Backslash) => return Err(self.error(35, 1)),
                TokenKind::Operator(op) => *op,
                // The C++'s TILDE, DTILDE and SQLEFT cases, which reattach the
                // message to the term already parsed. Reached whenever the left
                // term did not come back through `message_subterm`'s own
                // cascade loop, which is what a prefix `>` or `<` does:
                // `variable_reference_term` returns straight to the caller, so
                // the `~` in `>a~b` arrives here. Measured, `r = >a~b` and
                // `r = >a[1]` both translate.
                TokenKind::Tilde | TokenKind::DTilde | TokenKind::LeftBracket => {
                    left = self.cascade(left, term)?;
                    continue;
                }
                TokenKind::RightParen => return Err(self.error(37, 2)),
                TokenKind::RightBracket => return Err(self.error(37, 901)),
                // `+=` and friends are assignments, not operators.
                TokenKind::Assignment(_) => return Err(self.error(35, 1)),
                _ => return Err(self.error(35, 1)),
            };
            let prec = precedence(op);
            if prec < min_prec {
                return Ok(left);
            }
            if op != Operator::Abuttal {
                self.advance();
            }
            let Some(right) = self.message_subterm(term)? else {
                return Err(self.error(35, 1));
            };
            let right = self.binary_rest(right, prec + 1, term)?;
            left = Expr::binary(op, left, right);
        }
    }

    /// `parseMessageSubterm` (`LanguageParser.cpp:3617`): prefix operators,
    /// then a subterm with every message send that follows it.
    fn message_subterm(&mut self, term: Terminators) -> Result<Option<Expr>, ParseError> {
        let Some(token) = self.peek() else {
            return Ok(None);
        };
        if self.is_terminator(Some(token), term) {
            return Ok(None);
        }
        if let TokenKind::Operator(op) = token.kind {
            let op_span = token.span.clone();
            let from = self.cursor.position();
            let prefix = match op {
                Operator::Plus => PrefixOp::Plus,
                Operator::Subtract => PrefixOp::Minus,
                Operator::Backslash => PrefixOp::Not,
                // A prefix `>` or `<` is not an operator: it takes a
                // reference to a variable.
                Operator::LessThan | Operator::GreaterThan => {
                    self.advance();
                    return self.variable_reference_term(from).map(Some);
                }
                _ => return Err(self.error(35, 1)),
            };
            self.advance();
            // Recursing on the whole subterm rather than on an atom is what
            // puts prefix operators above `**`: `-2 ** 2` is `(-2) ** 2`,
            // which is 4. It also means a chain works, though `--` is a line
            // comment so `- -2` needs the blank.
            let Some(operand) = self.message_subterm(term)? else {
                return Err(self.error(35, 901));
            };
            return Ok(Some(Expr::new(
                ExprKind::Prefix {
                    op: prefix,
                    operand: Box::new(operand),
                },
                op_span,
            )));
        }
        let Some(atom) = self.subterm(term)? else {
            return Ok(None);
        };
        // A message cascade is part of the same expression term, and the C++
        // parses its message names with `TERM_EOC` rather than the caller's
        // set (`LanguageParser.cpp:3701`).
        self.cascade(atom, Terminators::EOC).map(Some)
    }

    /// Applies every `~`, `~~` and `[` that follows a term.
    fn cascade(&mut self, mut target: Expr, term: Terminators) -> Result<Expr, ParseError> {
        loop {
            match self.peek_tag() {
                Some(Tag::LeftBracket) => {
                    let from = self.cursor.position();
                    self.advance();
                    target = self.collection_message(target, from)?;
                }
                Some(tag @ (Tag::Tilde | Tag::DTilde)) => {
                    let from = self.cursor.position();
                    self.advance();
                    target = self.message(target, tag == Tag::DTilde, from, term)?;
                }
                _ => return Ok(target),
            }
        }
    }

    /// `parseCollectionMessage` (`LanguageParser.cpp:3309`): `target[args]`,
    /// which is the `[]` message.
    fn collection_message(&mut self, target: Expr, from: usize) -> Result<Expr, ParseError> {
        let args = self.arg_list(Some(Tag::RightBracket))?;
        let extent = self.extent(from);
        // Interned like a written name, where the oracle reaches for
        // `GlobalNames::BRACKETS` (`parser/LanguageParser.cpp:3320`-`:3321`):
        // its pool of pre-made names is a second pool this crate does not
        // have, and one pool that holds the spelling however it was written
        // is the same answer for a reader that compares selectors.
        let name = self.ctx.selectors.borrow_mut().intern(b"[]");
        Ok(Expr::new(
            ExprKind::Message {
                target: Box::new(target),
                name,
                super_class: None,
                args,
                cascade: false,
            },
            extent,
        ))
    }

    /// `parseMessage` (`LanguageParser.cpp:3369`): the message name, an
    /// optional `:superclass` override, and an optional argument list.
    fn message(
        &mut self,
        target: Expr,
        cascade: bool,
        from: usize,
        term: Terminators,
    ) -> Result<Expr, ParseError> {
        // No blank is ever skipped here, and the C++ does not skip one either:
        // it uses `nextToken`. A blank is only a token when the next real
        // character starts a symbol, a literal, `(` or `[`, and the previous
        // token was a symbol, a literal, `)` or `]`, so a `~` can have a
        // blank on neither side. Measured, `"abc" ~ length` is 3.
        let name = match self.peek() {
            Some(token) if !self.is_terminator(Some(token), term) => match &token.kind {
                // Already upcased by the scanner.
                TokenKind::Symbol { id, .. } => self.ctx.symbols.name(*id).as_bytes().to_vec(),
                // `parseMessage` upcases a literal name too, with
                // `RexxString::upper`, which is `Utilities::toUpper` per byte
                // and so upcases ASCII only (`Utilities.hpp:52`). Measured,
                // `"abc"~'length'`, `"abc"~'LENGTH'` and `"abc"~"lEnGtH"` all
                // give 3.
                TokenKind::Literal { value } => {
                    value.iter().map(|b| b.to_ascii_uppercase()).collect()
                }
                // `a~[3]` is 19.909: a bracket is not a message name.
                _ => return Err(self.error(19, 909)),
            },
            _ => return Err(self.error(19, 909)),
        };
        // Interned upcased, which is where `parseMessage` interns it:
        // `messagename = commonString(messagename->upper())`
        // (`parser/LanguageParser.cpp:3391`).
        let name = self.ctx.selectors.borrow_mut().intern(&name);
        self.advance();

        let mut super_class = None;
        if self.peek_tag() == Some(Tag::Colon) {
            self.advance();
            super_class = Some(Box::new(self.super_class_term(term)?));
        }

        // Only a parenthesis directly abutted to the name is an argument
        // list. `a~m (1)` is a blank concatenation of `a~m` and `(1)`,
        // because the blank before `(` is a token.
        let args = if self.peek_tag() == Some(Tag::LeftParen) {
            self.advance();
            self.arg_list(Some(Tag::RightParen))?
        } else {
            Vec::new()
        };

        let extent = self.extent(from);
        Ok(Expr::new(
            ExprKind::Message {
                target: Box::new(target),
                name,
                super_class,
                args,
                cascade,
            },
            extent,
        ))
    }

    /// The `:superclass` override's term, which must pass
    /// `isVariableOrDot` (`Token.hpp:576`): a simple variable, a stem, a
    /// compound variable or a dot symbol.
    fn super_class_term(&mut self, term: Terminators) -> Result<Expr, ParseError> {
        let Some(token) = self.peek() else {
            return Err(self.error(20, 917));
        };
        if self.is_terminator(Some(token), term) {
            return Err(self.error(20, 917));
        }
        let span = token.span.clone();
        // `isVariableOrDot` reads only the subclass, so a literal token cannot
        // pass it whatever it holds, and this match on `Symbol` is the same
        // gate.
        let TokenKind::Symbol { id, class } = &token.kind else {
            return Err(self.error(20, 917));
        };
        if !matches!(
            class,
            SymbolClass::Variable
                | SymbolClass::Stem
                | SymbolClass::Compound
                | SymbolClass::DotSymbol
        ) {
            return Err(self.error(20, 917));
        }
        let kind = symbol_kind(*id, *class);
        self.advance();
        Ok(Expr::new(kind, span))
    }

    /// `parseVariableReferenceTerm` (`LanguageParser.cpp:3719`): a prefix `>`
    /// or `<` on a simple variable or a stem.
    fn variable_reference_term(&mut self, from: usize) -> Result<Expr, ParseError> {
        self.skip_blanks();
        let Some(token) = self.peek() else {
            return Err(self.error(20, 930));
        };
        let span = token.span.clone();
        let kind = match &token.kind {
            TokenKind::Symbol {
                id,
                class: SymbolClass::Variable,
            } => ExprKind::Variable(*id),
            TokenKind::Symbol {
                id,
                class: SymbolClass::Stem,
            } => ExprKind::Stem(*id),
            _ => return Err(self.error(20, 930)),
        };
        self.advance();
        let inner = Expr::new(kind, span);
        let extent = self.extent(from);
        Ok(Expr::new(
            ExprKind::VariableReference(Box::new(inner)),
            extent,
        ))
    }

    /// `parseArgList` (`LanguageParser.cpp:3083`): comma-separated arguments
    /// up to `closer`, which is consumed.
    fn arg_list(&mut self, closer: Option<Tag>) -> Result<Vec<Option<Expr>>, ParseError> {
        // Task 3d: an argument list re-enters expression parsing, so
        // `f(f(f(...)))` and `a[a[a[...]]]` recurse once per nesting level
        // exactly as grouping parentheses do, through different code and off
        // the same stack. Counted against the same budget for the reason
        // `MAX_EXPR_DEPTH`'s doc gives.
        if self.expr_depth >= MAX_EXPR_DEPTH {
            return Err(self.error(11, 1));
        }
        self.expr_depth += 1;
        let args = self.arg_list_inner(closer);
        // Decremented before the `?` in the caller's chain, so an error path
        // unwinds the count as faithfully as a success path. `arg_list` has
        // several early returns inside, which is why the body is a separate
        // function rather than this being a bracketed block.
        self.expr_depth -= 1;
        args
    }

    fn arg_list_inner(&mut self, closer: Option<Tag>) -> Result<Vec<Option<Expr>>, ParseError> {
        let bracket = closer == Some(Tag::RightBracket);
        let term = match closer {
            Some(Tag::RightBracket) => Terminators::SQRIGHT,
            Some(_) => Terminators::RIGHT,
            None => Terminators::EOC,
        };
        // Skips the blank a `CALL` keyword leaves before its first argument.
        self.skip_blanks();
        let mut args: Vec<Option<Expr>> = Vec::new();
        let mut real = 0;
        loop {
            let arg = self.subexpression(term)?;
            if arg.is_some() {
                real = args.len() + 1;
            }
            args.push(arg);
            if self.peek_tag() == Some(Tag::Comma) {
                self.advance();
                continue;
            }
            break;
        }
        if let Some(closer) = closer {
            if self.peek_tag() != Some(closer) {
                return Err(self.unmatched(bracket));
            }
            self.advance();
        }
        args.truncate(real);
        Ok(args)
    }

    /// `parseMessageTerm` (`LanguageParser.cpp:3500`): a subterm with at least
    /// one `~`, `~~` or `[` applied to it.
    fn message_term(&mut self) -> Result<Option<Expr>, ParseError> {
        let Some(token) = self.peek() else {
            return Ok(None);
        };
        if self.is_terminator(Some(token), Terminators::EOC) {
            return Ok(None);
        }
        // The C++'s fast path, and it exists to avoid allocating a variable
        // slot for every keyword instruction name in every code block. It is
        // reproduced because it also decides the answer: a simple variable
        // followed by neither a message operator nor a `(` is rejected here
        // rather than parsed and thrown away.
        if matches!(
            &token.kind,
            TokenKind::Symbol {
                class: SymbolClass::Variable,
                ..
            }
        ) {
            let next = self.ctx.tokens.get(self.cursor.position() + 1);
            let follows = next.map(|token| token.kind.tag());
            if !matches!(
                follows,
                Some(Tag::Tilde | Tag::DTilde | Tag::LeftBracket | Tag::LeftParen)
            ) {
                return Ok(None);
            }
        }
        let Some(start) = self.subterm(Terminators::EOC)? else {
            return Ok(None);
        };
        // Only a term that actually took a message is one. `f(1)` parses as a
        // call here and then comes back `None`, which is why a bare `f(1)`
        // clause is a command.
        let mut applied = false;
        let mut target = start;
        loop {
            let from = self.cursor.position();
            match self.peek_tag() {
                Some(Tag::LeftBracket) => {
                    self.advance();
                    target = self.collection_message(target, from)?;
                }
                Some(tag @ (Tag::Tilde | Tag::DTilde)) => {
                    self.advance();
                    target = self.message(target, tag == Tag::DTilde, from, Terminators::EOC)?;
                }
                _ => break,
            }
            applied = true;
        }
        Ok(applied.then_some(target))
    }

    /// `parseSubTerm` (`LanguageParser.cpp:3757`): the atoms.
    fn subterm(&mut self, term: Terminators) -> Result<Option<Expr>, ParseError> {
        let Some(token) = self.peek() else {
            return Ok(None);
        };
        if self.is_terminator(Some(token), term) {
            return Ok(None);
        }
        let from = self.cursor.position();
        let span = token.span.clone();
        match &token.kind {
            TokenKind::LeftParen => {
                self.advance();
                // Task 3c: one more level would recurse past MAX_EXPR_DEPTH,
                // so raise the same condition the oracle raises somewhere
                // past its own, differently-sized cliff, rather than
                // descending into `full_subexpression` at all. See
                // `MAX_EXPR_DEPTH`'s own doc comment for the two measured
                // cliffs this sits between.
                if self.expr_depth >= MAX_EXPR_DEPTH {
                    return Err(self.error(11, 1));
                }
                self.expr_depth += 1;
                // The enclosing terminators are dropped, because the brackets
                // disambiguate, and a comma list is allowed again in here.
                let inner = self.full_subexpression(Terminators::RIGHT);
                self.expr_depth -= 1;
                let Some(inner) = inner? else {
                    // `()` is 35.1.
                    return Err(self.error(35, 1));
                };
                if self.peek_tag() != Some(Tag::RightParen) {
                    return Err(self.unmatched(false));
                }
                self.advance();
                // The parenthesised expression is returned unchanged, so
                // there is no node for the parentheses and the span still
                // covers only what built the node. See `ast`'s module docs.
                Ok(Some(inner))
            }
            TokenKind::Symbol { id, class } => {
                let (id, class) = (*id, *class);
                self.advance();
                match self.peek_tag() {
                    Some(Tag::LeftParen) => {
                        self.advance();
                        let args = self.arg_list(Some(Tag::RightParen))?;
                        let extent = self.extent(from);
                        Ok(Some(Expr::new(
                            ExprKind::Call {
                                target: CallTarget::Symbol(id),
                                args,
                            },
                            extent,
                        )))
                    }
                    Some(Tag::Colon) => {
                        self.advance();
                        self.qualified_symbol(id, from).map(Some)
                    }
                    _ => Ok(Some(Expr::new(symbol_kind(id, class), span))),
                }
            }
            TokenKind::Literal { value } => {
                let value = value.clone();
                self.advance();
                if self.peek_tag() == Some(Tag::LeftParen) {
                    self.advance();
                    let args = self.arg_list(Some(Tag::RightParen))?;
                    let extent = self.extent(from);
                    return Ok(Some(Expr::new(
                        ExprKind::Call {
                            target: CallTarget::Literal(value),
                            args,
                        },
                        extent,
                    )));
                }
                Ok(Some(Expr::new(ExprKind::Literal(value), span)))
            }
            TokenKind::Operator(op) => match op {
                // Not this function's business: `message_subterm` handles a
                // prefix operator, so report nothing found rather than an
                // error. Unreachable from there, mirrored from the C++.
                Operator::Plus | Operator::Subtract | Operator::Backslash => Ok(None),
                Operator::LessThan | Operator::GreaterThan => {
                    self.advance();
                    self.variable_reference_term(from).map(Some)
                }
                _ => Err(self.error(35, 1)),
            },
            TokenKind::RightParen => Err(self.error(37, 2)),
            TokenKind::RightBracket => Err(self.error(37, 901)),
            // 37.1, which cannot be reached: a comma always terminates, so
            // the terminator check above caught it.
            TokenKind::Comma => Err(self.error(37, 1)),
            // A blank, a colon, a `::` or a `[` where a term was expected.
            // `a [1]` is 35.1 for exactly this reason.
            _ => Err(self.error(35, 1)),
        }
    }

    /// `parseQualifiedSymbol` (`LanguageParser.cpp:3261`): `ns:name`, either a
    /// qualified call or a class lookup.
    fn qualified_symbol(&mut self, namespace: SymbolId, from: usize) -> Result<Expr, ParseError> {
        let Some(token) = self.peek() else {
            return Err(self.error(20, 923));
        };
        let TokenKind::Symbol { id: name, .. } = token.kind else {
            return Err(self.error(20, 923));
        };
        self.advance();
        // Only an immediately following parenthesis makes this a call.
        if self.peek_tag() == Some(Tag::LeftParen) {
            self.advance();
            let args = self.arg_list(Some(Tag::RightParen))?;
            let extent = self.extent(from);
            return Ok(Expr::new(
                ExprKind::QualifiedCall {
                    namespace,
                    name,
                    args,
                },
                extent,
            ));
        }
        let extent = self.extent(from);
        Ok(Expr::new(
            ExprKind::ClassResolver { namespace, name },
            extent,
        ))
    }
}

/// The leaf node a symbol token stands for, from its scanned class.
pub(crate) fn symbol_kind(id: SymbolId, class: SymbolClass) -> ExprKind {
    match class {
        SymbolClass::Dummy | SymbolClass::Constant => ExprKind::Constant(id),
        SymbolClass::Variable => ExprKind::Variable(id),
        SymbolClass::Stem => ExprKind::Stem(id),
        SymbolClass::Compound => ExprKind::Compound(id),
        SymbolClass::DotSymbol => ExprKind::DotVariable(id),
    }
}

#[cfg(test)]
mod differential;
#[cfg(test)]
mod tests;
