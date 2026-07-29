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

//! The instruction grammar: one clause in, one instruction out.
//!
//! Ported from `LanguageParser::nextInstruction` (`InstructionParser.cpp:122`)
//! and the constructors below it.
//!
//! # Keywords are not reserved words
//!
//! Every one of the 35 instruction keywords is a legal variable name. `if = 2`
//! assigns, `say if` prints 2, and `if if = 2 then say if` does both in one
//! clause. A symbol is a keyword only by POSITION: the first token of a clause
//! that is not a label and not an assignment. That is why recognition lives
//! here rather than in the scanner, which cannot know a token's position in
//! its clause, and why `ParseCtx::keywords` is consulted at exactly one place
//! per table.
//!
//! `RexxToken::keyword()` (`KeywordConstants.cpp:510`) resolves against the
//! upcased spelling of any symbol token, whatever its class, so `end.` and
//! `if.1` do not match: their spellings are `END.` and `IF.1`. Recognition
//! here is the same test, as an integer comparison against a pre-interned id.
//!
//! # What this does not decide
//!
//! The C++ splits instruction parsing in two. `nextInstruction` handles one
//! clause and knows nothing about blocks; `translateBlock`
//! (`LanguageParser.cpp:1176`) walks the whole body with a control stack and
//! raises every misplaced-block error. This module is the first half. It
//! carries one bit of the second half, `ClauseCursor`'s pending `THEN`,
//! because `THEN` is the one keyword `nextInstruction` itself rejects, and
//! nothing else. `parse_instructions` lists what the chain assembler owes.

use std::ops::Range;

use crate::ast::{
    ControlExpr, Controlled, Expr, Instruction, InstructionKind, Loop, LoopConditional, LoopKind,
};
use crate::clause::{Clause, ClauseCursor, PendingThen, split_clauses};
use crate::expr::{
    Terminators, need_variable, parse_expr, parse_expression, parse_logical, parse_message_term,
    symbol_kind,
};
use crate::source::SourceKind;
use crate::token::{Operator, ParseCtx, ParseError, SymbolId, Tag, Token, TokenCursor, TokenKind};

// Positions in the `INSTRUCTIONS` table, which `KeywordSet::index_of`
// returns. An entry's position is its meaning, so these are indices and not
// spellings, and `tests::keyword_indices_still_name_their_own_spellings` pins
// every one against the table.
const KW_DO: usize = 3;
const KW_ELSE: usize = 5;
const KW_END: usize = 6;
const KW_IF: usize = 11;
const KW_ITERATE: usize = 13;
const KW_LEAVE: usize = 14;
const KW_LOOP: usize = 15;
const KW_NOP: usize = 16;
const KW_OTHERWISE: usize = 19;
const KW_SELECT: usize = 29;
const KW_THEN: usize = 31;
const KW_WHEN: usize = 34;

// Positions in the `SUB_KEYWORDS` table, pinned the same way by
// `tests::keyword_indices_still_name_their_own_spellings`.
const SUB_BY: usize = 5;
const SUB_CASE: usize = 6;
const SUB_COUNTER: usize = 9;
const SUB_FOR: usize = 17;
const SUB_FOREVER: usize = 18;
const SUB_INDEX: usize = 21;
const SUB_ITEM: usize = 24;
const SUB_LABEL: usize = 25;
const SUB_OVER: usize = 34;
const SUB_TO: usize = 42;
const SUB_UNTIL: usize = 44;
const SUB_WHILE: usize = 48;
const SUB_WITH: usize = 49;

/// The error the C++ raises where its own `switch` has no case left.
///
/// `reportException(Error_Interpretation_switch, ...)` appears at five points
/// inside the loop grammar, each after an expression whose terminator set
/// admits only the keywords the switch already handles, so none is reachable.
/// They are reproduced as the same error rather than as a panic, because a
/// panic on source input would be worse than an odd number, and left in place
/// rather than dropped so that a future change to a terminator set surfaces
/// here instead of silently taking a wrong branch.
const UNREACHABLE_SWITCH: (u16, u16) = (49, 2);

/// Parses every instruction of one code body.
///
/// Stops at the end of the token vector or at a `::` clause, which starts a
/// directive and ends the body exactly as `nextInstruction` returning null
/// does (`InstructionParser.cpp:129`-`135`).
///
/// # What this does NOT do, and what owes it
///
/// This is a loop, not `translateBlock`. It builds no control stack, so it
/// wires nothing together and raises none of the errors that need to know
/// which block is open. The task that assembles the instruction chain owes all
/// of the following, and every one is a `translateBlock` error in the C++
/// rather than a `nextInstruction` one:
///
/// * 7.2, an instruction other than `WHEN`/`OTHERWISE`/`END` inside a `SELECT`.
/// * 8.2, an `ELSE` with no `THEN` above it.
/// * 9.1 and 9.2, a `WHEN` or `OTHERWISE` outside a `SELECT`.
/// * 10.1, 10.2 and 10.3, an `END` with no block, or one closing a `THEN` or
///   an `ELSE`.
/// * 14.x, an unclosed `DO`, `SELECT`, `THEN` or `ELSE` at the end of a body.
/// * The misplaced-label errors, which depend on the open block.
/// * 99.907 and 99.910, `EXPOSE` and `USE LOCAL` not being the first
///   instruction, which read `lastInstruction`.
/// * The chain indices themselves: which instruction an `IF` skips to, which
///   block an `END` closes, which `SELECT` a `WHEN` belongs to.
/// * 35.934 in place of 35.929 for a `WHEN` inside `SELECT CASE`, whose parse
///   needs the enclosing block to pick `parseCaseWhenList`. The hook is
///   threaded already: the sub-number is an argument at one call site.
///
/// 18.1 and 18.2 are raised here and in `parse_instruction`, because the one
/// bit of state they need is the one bit this module carries.
#[allow(dead_code)] // deleted by Task 3.7b
pub(crate) fn parse_instructions(ctx: &ParseCtx) -> Result<Vec<Instruction>, ParseError> {
    let mut cursor = ClauseCursor::new(split_clauses(ctx.tokens)?);
    let mut out: Vec<Instruction> = Vec::new();
    while let Some(clause) = cursor.peek() {
        if ctx.tokens[clause.tokens.start].kind.tag() == Tag::DColon {
            break;
        }
        out.push(parse_instruction(ctx, &mut cursor)?);
    }
    // An IF or WHEN whose THEN never arrived, because the body ended first.
    // `translateBlock` raises this from the failed `nextClause()`
    // (`LanguageParser.cpp:1341`), against the IF's own location.
    if let Some(which) = cursor.take_expected_then() {
        let byte = out.last().map_or(0, |last| last.clause_span.start);
        return Err(ParseError::new(18, missing_then_sub(which), byte));
    }
    Ok(out)
}

/// The sub-number of the missing-`THEN` error, which names the instruction
/// that wanted it: `Error_Then_expected_if` is 18.1 and
/// `Error_Then_expected_when` is 18.2.
fn missing_then_sub(which: PendingThen) -> u16 {
    match which {
        PendingThen::If => 1,
        PendingThen::When => 2,
    }
}

/// Parses the clause the cursor is sitting on, advancing it.
///
/// Panics on an exhausted cursor and on a clause whose first token is `::`,
/// neither of which is an instruction. `parse_instructions` filters both.
#[allow(dead_code)] // deleted by Task 3.7b
pub(crate) fn parse_instruction(
    ctx: &ParseCtx,
    cursor: &mut ClauseCursor,
) -> Result<Instruction, ParseError> {
    let clause = cursor
        .peek()
        .expect("parse_instruction on an exhausted cursor")
        .clone();
    let mut parser = Inst::new(ctx, clause);

    // `translateBlock` looks for the THEN itself, before `nextInstruction`
    // sees the clause at all (`LanguageParser.cpp:1345`-`1352`), so this test
    // comes ahead of the label and assignment tests rather than inside the
    // keyword dispatch: measured, `if 1 = 1` followed by `then = 7` is an
    // error rather than an assignment.
    if let Some(which) = cursor.take_expected_then() {
        // A label clause is excluded, which is the one place this diverges
        // from the C++. There the label is still one clause with whatever
        // follows the colon, so a label spelled THEN becomes the THEN and the
        // leftover `:` then fails with 35.1; Task 3.4 has already split the
        // colon off here, leaving nothing that can fail. Both reject
        // `if 1 = 1` followed by `then: nop`; the number differs.
        if parser.clause.label.is_some() || parser.first_keyword() != Some(KW_THEN) {
            return Err(parser.error(18, missing_then_sub(which)));
        }
        let end_at = parser.keyword_end();
        parser.next_real();
        return Ok(parser.finish_split(cursor, InstructionKind::Then, end_at));
    }

    parser.dispatch(cursor)
}

/// One clause's parse in progress.
struct Inst<'a> {
    ctx: &'a ParseCtx<'a>,
    /// Position inside `clause.tokens`.
    cursor: TokenCursor,
    clause: Clause,
    /// The byte offset every error is reported against.
    ///
    /// The start of the clause, not of the offending token: `syntaxError`
    /// reports against `clauseLocation` even when it is handed a token, and
    /// Task 3.8 turns this into a line number.
    clause_byte: usize,
}

impl<'a> Inst<'a> {
    fn new(ctx: &'a ParseCtx<'a>, clause: Clause) -> Self {
        let clause_byte = ctx
            .tokens
            .get(clause.tokens.start)
            .map_or(clause.span.start, |token| token.span.start);
        Inst {
            ctx,
            cursor: TokenCursor::new(clause.tokens.clone()),
            clause,
            clause_byte,
        }
    }

    fn error(&self, code: u16, sub: u16) -> ParseError {
        ParseError::new(code, sub, self.clause_byte)
    }

    /// Index of the next token that is not a blank, without consuming.
    ///
    /// `None` is the end of the clause, which is the C++'s `TOKEN_EOC`: the
    /// terminating token is outside `Clause::tokens`, so running out of tokens
    /// is exactly `isEndOfClause()`.
    fn peek_real_index(&self) -> Option<usize> {
        let mut i = self.cursor.peek()?;
        while i < self.clause.tokens.end && self.ctx.tokens[i].kind.tag() == Tag::Blank {
            i += 1;
        }
        (i < self.clause.tokens.end).then_some(i)
    }

    /// `nextReal` without consuming.
    fn peek_real(&self) -> Option<&'a Token> {
        self.peek_real_index().map(|i| &self.ctx.tokens[i])
    }

    /// Index of the `n`th token that is not a blank, counting the next one as
    /// zero, without consuming.
    ///
    /// This is the whole of what `markPosition`/`resetPosition` bought the C++
    /// in `createLoop`: it looks two real tokens ahead to tell `DO name = expr`
    /// from `DO name OVER expr` from `DO expr`, then consumes according to what
    /// it found.
    fn nth_real_index(&self, n: usize) -> Option<usize> {
        let mut i = self.cursor.peek()?;
        let mut seen = 0;
        loop {
            while i < self.clause.tokens.end && self.ctx.tokens[i].kind.tag() == Tag::Blank {
                i += 1;
            }
            if i >= self.clause.tokens.end {
                return None;
            }
            if seen == n {
                return Some(i);
            }
            seen += 1;
            i += 1;
        }
    }

    fn nth_real(&self, n: usize) -> Option<&'a Token> {
        self.nth_real_index(n).map(|i| &self.ctx.tokens[i])
    }

    /// `nextReal`: the next token that is not a blank, consumed.
    fn next_real(&mut self) -> Option<&'a Token> {
        let i = self.peek_real_index()?;
        self.seek(i + 1);
        Some(&self.ctx.tokens[i])
    }

    /// `nextToken` without consuming: the very next token, blank or not, and
    /// only if it is inside this clause.
    fn peek_token(&self, ahead: usize) -> Option<&'a Token> {
        let i = self.cursor.position() + ahead;
        (i < self.clause.tokens.end).then(|| &self.ctx.tokens[i])
    }

    /// Steps the cursor to token index `to`.
    ///
    /// The C++ would `resetPosition` backwards; nothing here ever does, so
    /// this only moves forward and there is no `TokenCursor::back` to call.
    fn seek(&mut self, to: usize) {
        while self.cursor.position() < to {
            self.cursor.advance();
        }
    }

    /// A second cursor over the same clause, at the same position.
    ///
    /// This is what replaces `markPosition`/`resetPosition`: a caller that may
    /// want to un-parse something parses it on one of these and keeps it only
    /// on success. The range is the whole clause, so an error raised through it
    /// still reports against the clause's first byte.
    fn trial(&self) -> TokenCursor {
        let mut trial = TokenCursor::new(self.clause.tokens.clone());
        while trial.position() < self.cursor.position() {
            trial.advance();
        }
        trial
    }

    fn at_end(&self) -> bool {
        self.peek_real_index().is_none()
    }

    /// `requiredEndOfClause`: nothing may follow.
    fn required_end(&mut self, code: u16, sub: u16) -> Result<(), ParseError> {
        if self.at_end() {
            return Ok(());
        }
        Err(self.error(code, sub))
    }

    /// The instruction-keyword index of the clause's next real token.
    fn first_keyword(&self) -> Option<usize> {
        match &self.peek_real()?.kind {
            TokenKind::Symbol { id, .. } => self.ctx.keywords.instructions.index_of(*id),
            _ => None,
        }
    }

    /// The end byte of the keyword token the cursor is sitting on.
    fn keyword_end(&self) -> usize {
        self.peek_real()
            .expect("a keyword token selected this arm")
            .span
            .end
    }

    // ---- expression grammar, on this clause's cursor ----

    fn expr(&mut self, term: Terminators, missing: u16) -> Result<Expr, ParseError> {
        parse_expr(self.ctx, &mut self.cursor, term, missing)
    }

    fn opt_expr(&mut self, term: Terminators) -> Result<Option<Expr>, ParseError> {
        parse_expression(self.ctx, &mut self.cursor, term)
    }

    fn logical(&mut self, term: Terminators, missing: u16) -> Result<Expr, ParseError> {
        parse_logical(self.ctx, &mut self.cursor, term, missing)
    }

    // ---- finishing a clause ----

    /// The byte an `IF`'s or `WHEN`'s clause span ends at.
    ///
    /// Rule 2 ends an ordinary clause at the END of its terminating token, so
    /// `nop;` traces with its semicolon. `RexxInstructionIf` instead sets the
    /// end from the START of whatever token ended the condition
    /// (`IfInstruction.cpp:64`), so both spellings lose bytes: measured,
    /// `if 1 = 1   then    say "a"` keeps all three blanks before `then`, and
    /// `if 1 = 1;` with `then` on the next line traces as `if 1 = 1` WITHOUT
    /// its semicolon.
    fn condition_end(&self) -> usize {
        let terminator = self.peek_real_index().unwrap_or(self.clause.tokens.end);
        self.ctx
            .tokens
            .get(terminator)
            .map_or(self.clause.span.end, |token| token.span.start)
    }

    /// Consumes the clause whole and builds the instruction.
    fn finish(self, cursor: &mut ClauseCursor, kind: InstructionKind) -> Instruction {
        let clause = cursor.next_clause().expect("the clause being parsed");
        Instruction {
            kind,
            clause_span: clause.span,
        }
    }

    /// Consumes the clause with its span end moved to `end_at`, re-presenting
    /// whatever the cursor has not reached as the next clause.
    ///
    /// The span end is narrowed even when nothing follows, because the two
    /// adjustments are independent: `if 1 = 1;` loses its semicolon whether or
    /// not a `THEN` shares the line, and a `THEN` at the end of a line loses
    /// the blanks after it.
    fn finish_split(
        self,
        cursor: &mut ClauseCursor,
        kind: InstructionKind,
        end_at: usize,
    ) -> Instruction {
        let clause = match self.peek_real_index() {
            Some(at) => cursor.split_before(self.ctx, at, end_at),
            None => {
                let mut clause = cursor.next_clause().expect("the clause being parsed");
                clause.span.end = end_at;
                clause
            }
        };
        Instruction {
            kind,
            clause_span: clause.span,
        }
    }

    // ---- dispatch ----

    /// `nextInstruction`'s progression: label, assignment, message term,
    /// keyword, command.
    fn dispatch(mut self, cursor: &mut ClauseCursor) -> Result<Instruction, ParseError> {
        if let Some(label) = self.clause.label.clone() {
            let kind = self.label(label)?;
            return Ok(self.finish(cursor, kind));
        }
        if let Some(kind) = self.assignment()? {
            return Ok(self.finish(cursor, kind));
        }
        if let Some(kind) = self.message()? {
            return Ok(self.finish(cursor, kind));
        }
        match self.first_keyword() {
            Some(index) => self.keyword(cursor, index),
            // Not a keyword, so the whole clause is a command.
            None => self.command(cursor),
        }
    }

    /// `labelNew` (`InstructionParser.cpp:2792`).
    ///
    /// Task 3.4 already ended the clause at the colon, so nothing is trimmed
    /// here. Error 47.1 is raised at the point the C++ raises it, from
    /// `isInterpret()` (`InstructionParser.cpp:155`): measured,
    /// `interpret "here: nop"` is rc 47 with `found "HERE"`.
    fn label(&self, label: Range<usize>) -> Result<InstructionKind, ParseError> {
        if self.ctx.source.kind() == SourceKind::Interpret {
            return Err(self.error(47, 1));
        }
        // A label may be a literal too, `"here": nop`. Both spellings reach
        // `addLabel` through `token->value()`, which is why the name is bytes
        // rather than a `SymbolId`: a literal was never seen as a symbol, so
        // it is not in the read-only symbol table.
        let name = match &self.ctx.tokens[label.start].kind {
            TokenKind::Symbol { id, .. } => Box::from(self.ctx.symbols.name(*id).as_bytes()),
            TokenKind::Literal { value } => value.clone(),
            other => panic!("a label token is a symbol or a literal, not {other:?}"),
        };
        Ok(InstructionKind::Label { name })
    }

    /// The `symbol = expr` and `symbol (op)= expr` forms
    /// (`InstructionParser.cpp:180`-`196`, `assignmentNew`, `assignmentOpNew`).
    ///
    /// Recognised from the first two tokens with no blank skipped, which is
    /// what the C++ does and is safe: a blank is only a token when the next
    /// real character starts a symbol, a literal, a `(` or a `[`, so no blank
    /// can sit before an `=`.
    fn assignment(&mut self) -> Result<Option<InstructionKind>, ParseError> {
        let Some(first) = self.peek_token(0) else {
            return Ok(None);
        };
        let TokenKind::Symbol { id, class } = first.kind else {
            return Ok(None);
        };
        let target_span = first.span.clone();
        let Some(second) = self.peek_token(1) else {
            return Ok(None);
        };
        let op = match &second.kind {
            // `symbol == expr` is not an assignment and not an expression
            // either: it is rejected outright.
            TokenKind::Operator(Operator::StrictEqual) => return Err(self.error(35, 1)),
            TokenKind::Operator(Operator::Equal) => None,
            TokenKind::Assignment(op) => Some(*op),
            _ => return Ok(None),
        };
        need_variable(self.ctx, id, class, self.clause_byte)?;
        self.seek(self.cursor.position() + 2);
        let value = self.expr(Terminators::EOC, 918)?;
        // `assignmentOpNew` expands `a += b` into `a = a + b` at parse time,
        // building the binary node itself, so there is one instruction form
        // and not two.
        let value = match op {
            None => value,
            Some(op) => Expr::binary(op, Expr::new(symbol_kind(id, class), target_span), value),
        };
        Ok(Some(InstructionKind::Assignment { target: id, value }))
    }

    /// The four message-term forms (`InstructionParser.cpp:207`-`250`).
    ///
    /// Parsed on a trial cursor, because a term with no message applied is not
    /// a message instruction and the C++ resets its position when that
    /// happens. `f(1)` and `"echo hi"` both come back here as commands.
    fn message(&mut self) -> Result<Option<InstructionKind>, ParseError> {
        let mut trial = self.trial();
        let Some(term) = parse_message_term(self.ctx, &mut trial)? else {
            return Ok(None);
        };
        // A term came back, so the trial's position is the real one.
        self.cursor = trial;
        let op = match self.peek_token(0).map(|token| &token.kind) {
            // The whole clause was the message send.
            None => {
                return Ok(Some(InstructionKind::Message { term, value: None }));
            }
            Some(TokenKind::Operator(Operator::StrictEqual)) => return Err(self.error(35, 1)),
            Some(TokenKind::Operator(Operator::Equal)) => None,
            Some(TokenKind::Assignment(op)) => Some(*op),
            // Anything else means this was not a message instruction after
            // all, so the clause goes back to keyword processing with its
            // position reset, which is what `nextInstruction` does by falling
            // through to `firstToken()`.
            _ => {
                self.cursor = TokenCursor::new(self.clause.tokens.clone());
                return Ok(None);
            }
        };
        self.cursor.advance();
        let value = self.expr(Terminators::EOC, 1)?;
        let value = match op {
            None => value,
            // `messageAssignmentOpNew` keeps a copy of the message term as the
            // left operand and turns the original into the assignment.
            Some(op) => Expr::binary(op, term.clone(), value),
        };
        Ok(Some(InstructionKind::Message {
            term,
            value: Some(value),
        }))
    }

    /// `commandNew` (`InstructionParser.cpp:1241`): the fallback, and not an
    /// exotic one. A bare `"echo hi"` clause is a command dispatched through
    /// the current `ADDRESS`.
    fn command(mut self, cursor: &mut ClauseCursor) -> Result<Instruction, ParseError> {
        let expression = self.opt_expr(Terminators::EOC)?;
        Ok(self.finish(cursor, InstructionKind::Command { expression }))
    }

    /// The keyword instructions.
    fn keyword(
        mut self,
        cursor: &mut ClauseCursor,
        index: usize,
    ) -> Result<Instruction, ParseError> {
        let keyword_end = self.keyword_end();
        match index {
            KW_NOP => {
                self.next_real();
                // `nopNew`: nothing may follow.
                self.required_end(21, 901)?;
                Ok(self.finish(cursor, InstructionKind::Nop))
            }
            KW_IF => {
                self.next_real();
                self.if_instruction(cursor, PendingThen::If)
            }
            KW_WHEN => {
                self.next_real();
                self.if_instruction(cursor, PendingThen::When)
            }
            // `elseNew` and `otherwiseNew` parse nothing at all. Both take
            // their location from the keyword token and let `translateBlock`
            // trim whatever shares the line.
            KW_ELSE => {
                self.next_real();
                Ok(self.finish_split(cursor, InstructionKind::Else, keyword_end))
            }
            KW_OTHERWISE => {
                self.next_real();
                Ok(self.finish_split(cursor, InstructionKind::Otherwise, keyword_end))
            }
            KW_END => {
                self.next_real();
                let name = self.end_name()?;
                Ok(self.finish(cursor, InstructionKind::End { name }))
            }
            // `createLoop(false)` and `createLoop(true)`. Bare `DO` is a
            // block and bare `LOOP` is `LOOP FOREVER`, which is the only
            // difference between the two keywords.
            KW_DO => {
                self.next_real();
                let body = self.create_loop(false)?;
                Ok(self.finish(cursor, InstructionKind::Do(Box::new(body))))
            }
            KW_LOOP => {
                self.next_real();
                let body = self.create_loop(true)?;
                Ok(self.finish(cursor, InstructionKind::Loop(Box::new(body))))
            }
            KW_SELECT => {
                self.next_real();
                let kind = self.select()?;
                Ok(self.finish(cursor, kind))
            }
            // `leaveNew` (`InstructionParser.cpp:2822`) builds both, and both
            // parse bare: measured, `leave` alone is rc 0 under rexxc and
            // Error 28.1 only at run time, because `RexxActivation.cpp:1214`
            // raises it. A parser that rejected it would diverge on every
            // program holding an unreachable LEAVE.
            KW_LEAVE => {
                self.next_real();
                let name = self.block_name(907)?;
                Ok(self.finish(cursor, InstructionKind::Leave { name }))
            }
            KW_ITERATE => {
                self.next_real();
                let name = self.block_name(908)?;
                Ok(self.finish(cursor, InstructionKind::Iterate { name }))
            }
            // A THEN that reaches the keyword dispatch is always misplaced,
            // because a real one is consumed above. `nextInstruction`'s own
            // arm is the same unconditional error.
            KW_THEN => Err(self.error(8, 1)),
            // Not implemented in this commit; the keyword falls through to the
            // command fallback until its family lands.
            _ => self.command(cursor),
        }
    }

    /// `endNew` (`InstructionParser.cpp:2246`): an optional block name.
    fn end_name(&mut self) -> Result<Option<SymbolId>, ParseError> {
        let Some(token) = self.next_real() else {
            return Ok(None);
        };
        let TokenKind::Symbol { id, .. } = token.kind else {
            return Err(self.error(20, 909));
        };
        self.required_end(21, 909)?;
        Ok(Some(id))
    }

    /// `createLoop` (`InstructionParser.cpp:1994`), for both `DO` and `LOOP`.
    ///
    /// The two keywords share every form and differ in one place: bare `DO` is
    /// a block and bare `LOOP` is `LOOP FOREVER`.
    fn create_loop(&mut self, is_loop: bool) -> Result<Loop, ParseError> {
        let mut label = None;
        let mut counter = None;

        // `LABEL name` and `COUNTER name` come first, in either order, and each
        // only once. The loop stops on anything else, including a second
        // occurrence of one already seen, which is how `do label a label b`
        // becomes a DO whose count expression is `label b`.
        while let Some(token) = self.peek_real() {
            if token.kind.tag() != Tag::Symbol {
                break;
            }
            let which = match self.sub_keyword(token) {
                Some(SUB_LABEL) if label.is_none() => SUB_LABEL,
                Some(SUB_COUNTER) if counter.is_none() => SUB_COUNTER,
                _ => break,
            };
            let name = self.nth_real(1);
            match name.map(|token| &token.kind) {
                Some(TokenKind::Symbol { id, class }) => {
                    let (id, class) = (*id, *class);
                    if which == SUB_LABEL {
                        label = Some(id);
                    } else {
                        need_variable(self.ctx, id, class, self.clause_byte)?;
                        counter = Some(id);
                    }
                    let consumed = self.nth_real_index(1).expect("just matched it");
                    self.seek(consumed + 1);
                    if label.is_some() && counter.is_some() {
                        break;
                    }
                }
                // `do label = 1 to 2` is a controlled loop over a variable
                // named LABEL, not a labelled loop. Any `=` triggers this,
                // even one that is part of a larger operator, which is why
                // `==` is an expression error rather than a controlled loop.
                Some(TokenKind::Operator(Operator::Equal)) => {
                    let control = self.controlled(label, counter, token)?;
                    return Ok(control);
                }
                Some(TokenKind::Operator(Operator::StrictEqual)) => {
                    return Err(self.error(35, 1));
                }
                // `Error_Symbol_expected_LABEL` is 20.918 and
                // `Error_Symbol_expected_counter` is 20.934.
                _ => {
                    return Err(self.error(20, if which == SUB_LABEL { 918 } else { 934 }));
                }
            }
        }

        // Just the keyword and its options.
        if self.at_end() {
            if is_loop {
                return self.plain_loop(label, counter, LoopKind::Forever);
            }
            // `newSimpleDo` takes no counter, because a block does not iterate.
            if counter.is_some() {
                return Err(self.error(27, 905));
            }
            return Ok(Loop {
                label,
                counter,
                kind: LoopKind::Simple,
                conditional: None,
            });
        }

        let Some(first) = self.peek_real() else {
            unreachable!("at_end was just false")
        };
        if first.kind.tag() != Tag::Symbol {
            // `DO expr` where the expression does not start with a symbol.
            return self.count_loop(label, counter);
        }
        let second = self.nth_real(1);
        match second.map(|token| &token.kind) {
            Some(TokenKind::Operator(Operator::StrictEqual)) => Err(self.error(35, 1)),
            Some(TokenKind::Operator(Operator::Equal)) => self.controlled(label, counter, first),
            _ => {
                // `DO name OVER expr`. This test comes BEFORE the WITH one, so
                // `do with over x` is a DO OVER whose control variable is named
                // WITH: measured, rc 0 under rexxc.
                if second.and_then(|token| self.sub_keyword(token)) == Some(SUB_OVER) {
                    return self.do_over(label, counter, first);
                }
                if self.sub_keyword(first) == Some(SUB_WITH)
                    && matches!(
                        second.and_then(|token| self.sub_keyword(token)),
                        Some(SUB_INDEX | SUB_ITEM)
                    )
                {
                    // Step past WITH; the INDEX or ITEM keyword starts the
                    // options.
                    self.next_real();
                    return self.do_with(label, counter);
                }
                match self.sub_keyword(first) {
                    Some(SUB_FOREVER) => {
                        self.next_real();
                        // `Error_Invalid_do_forever` is 27.901, and it is the
                        // one reachable use of `parseLoopConditional`'s error
                        // argument: measured, `do forever x` is rc 229.
                        self.plain_loop(label, counter, LoopKind::Forever)
                    }
                    // `DO WHILE` and `DO UNTIL` are a FOREVER loop with the
                    // conditional attached, which is what `newLoopWhile` and
                    // `newLoopUntil` build.
                    Some(SUB_WHILE | SUB_UNTIL) => {
                        self.plain_loop(label, counter, LoopKind::Forever)
                    }
                    // Not a loop keyword, so this is `DO expr`.
                    _ => self.count_loop(label, counter),
                }
            }
        }
    }

    /// `DO FOREVER`, `DO WHILE` and `DO UNTIL`, which differ only in the
    /// conditional that follows.
    ///
    /// `parseForeverLoop` (`InstructionParser.cpp:1860`) and the two `createLoop`
    /// arms below it all reach `parseLoopConditional` with the cursor on the
    /// keyword, so one function covers the three.
    fn plain_loop(
        &mut self,
        label: Option<SymbolId>,
        counter: Option<SymbolId>,
        kind: LoopKind,
    ) -> Result<Loop, ParseError> {
        let conditional = self.loop_conditional((27, 901))?;
        Ok(Loop {
            label,
            counter,
            kind,
            conditional,
        })
    }

    /// `parseCountLoop` (`InstructionParser.cpp:1916`): `DO expr`, with an
    /// optional trailing conditional.
    fn count_loop(
        &mut self,
        label: Option<SymbolId>,
        counter: Option<SymbolId>,
    ) -> Result<Loop, ParseError> {
        let count = self.opt_expr(Terminators::COND)?;
        let conditional = self.loop_conditional(UNREACHABLE_SWITCH)?;
        Ok(Loop {
            label,
            counter,
            kind: LoopKind::Count(count),
            conditional,
        })
    }

    /// `newControlledLoop` (`InstructionParser.cpp:1265`):
    /// `DO i = initial TO t BY b FOR f`.
    ///
    /// The cursor is on the control variable and the `=` follows it. The three
    /// keyword expressions may come in any order and each only once, and the
    /// order is recorded because it is the evaluation order.
    fn controlled(
        &mut self,
        label: Option<SymbolId>,
        counter: Option<SymbolId>,
        name: &Token,
    ) -> Result<Loop, ParseError> {
        let TokenKind::Symbol { id, class } = name.kind else {
            unreachable!("a controlled loop's control token is a symbol")
        };
        need_variable(self.ctx, id, class, self.clause_byte)?;
        // Step past the control variable and the `=`.
        let equals = self.nth_real_index(1).expect("the `=` that got us here");
        self.seek(equals + 1);

        let initial = self.expr(Terminators::CONTROL, 904)?;
        let mut control = Controlled {
            control: id,
            initial,
            to: None,
            by: None,
            for_count: None,
            order: Vec::new(),
        };
        let mut conditional = None;
        while let Some(token) = self.peek_real() {
            let (slot, missing, entry) = match self.sub_keyword(token) {
                Some(SUB_BY) => (&mut control.by, 905, ControlExpr::By),
                Some(SUB_TO) => (&mut control.to, 906, ControlExpr::To),
                Some(SUB_FOR) => (&mut control.for_count, 907, ControlExpr::For),
                Some(SUB_WHILE | SUB_UNTIL) => {
                    // `parseLoopConditional` allows nothing after itself, so
                    // this ends the clause.
                    conditional = self.loop_conditional(UNREACHABLE_SWITCH)?;
                    break;
                }
                _ => return Err(self.error(UNREACHABLE_SWITCH.0, UNREACHABLE_SWITCH.1)),
            };
            if slot.is_some() {
                // `Error_Invalid_do_duplicate`. Measured: `do i = 1 to 3 to 4`
                // is rc 229, Error 27.902.
                return Err(self.error(27, 902));
            }
            self.next_real();
            let value = self.expr(Terminators::CONTROL, missing)?;
            match entry {
                ControlExpr::By => control.by = Some(value),
                ControlExpr::To => control.to = Some(value),
                ControlExpr::For => control.for_count = Some(value),
            }
            control.order.push(entry);
        }
        Ok(Loop {
            // With no LABEL clause the control variable's name is the loop's
            // name, which is what `LEAVE i` matches against.
            label: label.or(Some(id)),
            counter,
            kind: LoopKind::Controlled(Box::new(control)),
            conditional,
        })
    }

    /// `newDoOverLoop` (`InstructionParser.cpp:1432`): `DO name OVER expr`,
    /// with an optional `FOR` and an optional conditional.
    fn do_over(
        &mut self,
        label: Option<SymbolId>,
        counter: Option<SymbolId>,
        name: &Token,
    ) -> Result<Loop, ParseError> {
        let TokenKind::Symbol { id, class } = name.kind else {
            unreachable!("a DO OVER control token is a symbol")
        };
        need_variable(self.ctx, id, class, self.clause_byte)?;
        // Step past the control variable and the OVER keyword.
        let over = self.nth_real_index(1).expect("the OVER that got us here");
        self.seek(over + 1);
        let target = self.expr(Terminators::OVER, 911)?;
        let (for_count, conditional) = self.for_and_conditional()?;
        Ok(Loop {
            label: label.or(Some(id)),
            counter,
            kind: LoopKind::Over {
                control: id,
                target,
                for_count,
            },
            conditional,
        })
    }

    /// `newDoWithLoop` (`InstructionParser.cpp:1582`):
    /// `DO WITH INDEX i ITEM v OVER expr`.
    ///
    /// The cursor is on the `INDEX` or `ITEM` keyword, the `WITH` having been
    /// stepped past. At least one of the two variables is required and `OVER`
    /// must follow both.
    fn do_with(
        &mut self,
        label: Option<SymbolId>,
        counter: Option<SymbolId>,
    ) -> Result<Loop, ParseError> {
        let mut index = None;
        let mut item = None;
        while let Some(token) = self.peek_real() {
            if token.kind.tag() != Tag::Symbol {
                break;
            }
            let slot = match self.sub_keyword(token) {
                Some(SUB_INDEX) => &mut index,
                Some(SUB_ITEM) => &mut item,
                _ => break,
            };
            if slot.is_some() {
                return Err(self.error(27, 902));
            }
            self.next_real();
            // `requiredVariable` then `addVariable`, so a non-symbol is
            // 20.929 and a symbol that is not a variable is `needVariable`'s
            // own number: measured, `do with index 1 over x` is 31.2 and
            // `do with index .a over x` is 31.3.
            let Some(name) = self.next_real() else {
                return Err(self.error(20, 929));
            };
            let TokenKind::Symbol { id, class } = name.kind else {
                return Err(self.error(20, 929));
            };
            need_variable(self.ctx, id, class, self.clause_byte)?;
            match self.sub_keyword(token) {
                Some(SUB_INDEX) => index = Some(id),
                _ => item = Some(id),
            }
        }
        if index.is_none() && item.is_none() {
            // `Error_Invalid_do_with_no_control`. Unreachable through
            // `createLoop`, which only comes here when the token after WITH is
            // INDEX or ITEM, and kept because `newDoWithLoop` checks it.
            return Err(self.error(27, 903));
        }
        // `Error_Invalid_do_with_no_over`. Measured: `do with index i x` is
        // rc 229, Error 27.904.
        let over = self
            .peek_real()
            .filter(|token| token.kind.tag() == Tag::Symbol);
        if over.and_then(|token| self.sub_keyword(token)) != Some(SUB_OVER) {
            return Err(self.error(27, 904));
        }
        self.next_real();
        let target = self.expr(Terminators::OVER, 911)?;
        let (for_count, conditional) = self.for_and_conditional()?;
        Ok(Loop {
            label,
            counter,
            kind: LoopKind::With {
                index,
                item,
                target,
                for_count,
            },
            conditional,
        })
    }

    /// The `FOR n` and `WHILE`/`UNTIL` tail that `DO OVER` and `DO WITH`
    /// share.
    fn for_and_conditional(
        &mut self,
    ) -> Result<(Option<Expr>, Option<LoopConditional>), ParseError> {
        let mut for_count = None;
        let mut conditional = None;
        while let Some(token) = self.peek_real() {
            match self.sub_keyword(token) {
                Some(SUB_FOR) => {
                    if for_count.is_some() {
                        return Err(self.error(27, 902));
                    }
                    self.next_real();
                    for_count = Some(self.expr(Terminators::CONTROL, 907)?);
                }
                Some(SUB_WHILE | SUB_UNTIL) => {
                    conditional = self.loop_conditional(UNREACHABLE_SWITCH)?;
                    break;
                }
                _ => return Err(self.error(UNREACHABLE_SWITCH.0, UNREACHABLE_SWITCH.1)),
            }
        }
        Ok((for_count, conditional))
    }

    /// `parseLoopConditional` (`InstructionParser.cpp:4600`): an optional
    /// trailing `WHILE` or `UNTIL`, and nothing after it.
    ///
    /// `unexpected` is the error for a token that is neither, which the C++
    /// passes per caller: 27.901 from `parseForeverLoop` and `Error_None`
    /// everywhere else, where the terminator set makes it unreachable.
    fn loop_conditional(
        &mut self,
        unexpected: (u16, u16),
    ) -> Result<Option<LoopConditional>, ParseError> {
        let Some(token) = self.next_real() else {
            return Ok(None);
        };
        let until = match self.sub_keyword(token) {
            Some(SUB_WHILE) => false,
            Some(SUB_UNTIL) => true,
            _ => return Err(self.error(unexpected.0, unexpected.1)),
        };
        // `Error_Invalid_expression_while` is 35.908 and
        // `Error_Invalid_expression_until` is 35.909.
        let condition = self.logical(Terminators::COND, if until { 909 } else { 908 })?;
        // `Error_Invalid_do_whileuntil`. Measured:
        // `do i = 1 to 3 while 1 until 2` is rc 229, Error 27.1.
        self.required_end(27, 1)?;
        Ok(Some(LoopConditional { until, condition }))
    }

    /// `LEAVE`'s and `ITERATE`'s optional block name.
    ///
    /// The two sub-numbers differ only in which keyword is named:
    /// `Error_Symbol_expected_leave` is 20.907 and `Error_Invalid_data_leave`
    /// is 21.907, against 20.908 and 21.908 for `ITERATE`.
    fn block_name(&mut self, sub: u16) -> Result<Option<SymbolId>, ParseError> {
        let Some(token) = self.next_real() else {
            return Ok(None);
        };
        let TokenKind::Symbol { id, .. } = token.kind else {
            return Err(self.error(20, sub));
        };
        self.required_end(21, sub)?;
        Ok(Some(id))
    }

    /// `selectNew` (`InstructionParser.cpp:3811`): an optional `LABEL name`,
    /// then an optional `CASE expr`, which is a different instruction class in
    /// the C++ and a different `WHEN` grammar under it.
    fn select(&mut self) -> Result<InstructionKind, ParseError> {
        let mut label = None;
        let mut case = None;
        if let Some(token) = self.peek_real() {
            // Anything that is not a symbol cannot be either keyword.
            if token.kind.tag() != Tag::Symbol {
                return Err(self.error(25, 923));
            }
            let mut token = token;
            if self.sub_keyword(token) == Some(SUB_LABEL) {
                self.next_real();
                let Some(name) = self.next_real() else {
                    return Err(self.error(20, 918));
                };
                let TokenKind::Symbol { id, .. } = name.kind else {
                    return Err(self.error(20, 918));
                };
                label = Some(id);
                match self.peek_real() {
                    Some(next) => token = next,
                    None => return Ok(InstructionKind::Select { label, case }),
                }
            }
            if token.kind.tag() == Tag::Symbol {
                if self.sub_keyword(token) != Some(SUB_CASE) {
                    return Err(self.error(25, 923));
                }
                self.next_real();
                case = Some(self.expr(Terminators::EOC, 933)?);
            }
            // Nothing else may follow either option.
            self.required_end(25, 923)?;
        }
        Ok(InstructionKind::Select { label, case })
    }

    /// `RexxToken::subKeyword` (`KeywordConstants.cpp:495`).
    fn sub_keyword(&self, token: &Token) -> Option<usize> {
        match &token.kind {
            TokenKind::Symbol { id, .. } => self.ctx.keywords.sub_keywords.index_of(*id),
            _ => None,
        }
    }

    /// `ifNew` (`InstructionParser.cpp:2678`) and the `SELECT` half of
    /// `whenNew` (`:2708`), which share `RexxInstructionIf`.
    ///
    /// The condition is a logical list, so commas are an AND rather than an
    /// array: measured, `if then nop`, `if , 1 = 1 then nop` and
    /// `if 1 = 1, then nop` are all 35.929.
    fn if_instruction(
        mut self,
        cursor: &mut ClauseCursor,
        which: PendingThen,
    ) -> Result<Instruction, ParseError> {
        // 929 is `Error_Invalid_expression_logical_list`, which `parseLogical`
        // raises before `requiredLogicalExpression`'s own 35.902 or 35.903 can
        // be reached. A `WHEN` inside `SELECT CASE` needs 35.934 here instead,
        // and that is the one argument the control stack will change.
        let condition = self.logical(Terminators::IF, 929)?;
        let end_at = self.condition_end();
        let kind = match which {
            PendingThen::If => InstructionKind::If { condition },
            PendingThen::When => InstructionKind::When { condition },
        };
        let instruction = self.finish_split(cursor, kind, end_at);
        cursor.expect_then(which);
        Ok(instruction)
    }
}

#[cfg(test)]
mod tests;
