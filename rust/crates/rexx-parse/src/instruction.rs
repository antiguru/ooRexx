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
    Address, AddressIo, Call, ConditionTrap, ControlExpr, Controlled, Expr, Forward, Guard,
    Instruction, InstructionKind, Loop, LoopConditional, LoopKind, NumericSetting, OutputOption,
    Parse, ParseSource, ParseTrigger, Raise, RaiseResult, Redirection, Signal, Trace, TriggerKind,
    Use, UseTarget, VariableRef,
};
use crate::clause::{Clause, ClauseCursor, PendingThen, split_clauses};
use crate::convert::{check_trace_setting, whole_number};
use crate::expr::{
    Terminators, need_variable, parse_arg_list, parse_constant_expression, parse_expr,
    parse_expression, parse_logical, parse_message_term, parse_paren_expression,
    parse_variable_or_message_term, symbol_kind,
};
use crate::source::SourceKind;
use crate::token::{
    Operator, ParseCtx, ParseError, SymbolClass, SymbolId, Tag, Token, TokenCursor, TokenKind,
};

// Positions in the `INSTRUCTIONS` table, which `KeywordSet::index_of`
// returns. An entry's position is its meaning, so these are indices and not
// spellings, and `tests::keyword_indices_still_name_their_own_spellings` pins
// every one against the table.
const KW_ADDRESS: usize = 0;
const KW_ARG: usize = 1;
const KW_CALL: usize = 2;
const KW_DO: usize = 3;
const KW_DROP: usize = 4;
const KW_ELSE: usize = 5;
const KW_END: usize = 6;
const KW_EXIT: usize = 7;
const KW_EXPOSE: usize = 8;
const KW_FORWARD: usize = 9;
const KW_GUARD: usize = 10;
const KW_IF: usize = 11;
const KW_INTERPRET: usize = 12;
const KW_ITERATE: usize = 13;
const KW_LEAVE: usize = 14;
const KW_LOOP: usize = 15;
const KW_NOP: usize = 16;
const KW_NUMERIC: usize = 17;
const KW_OPTIONS: usize = 18;
const KW_OTHERWISE: usize = 19;
const KW_PARSE: usize = 20;
const KW_PROCEDURE: usize = 21;
const KW_PULL: usize = 22;
const KW_PUSH: usize = 23;
const KW_QUEUE: usize = 24;
const KW_RAISE: usize = 25;
const KW_REPLY: usize = 26;
const KW_RETURN: usize = 27;
const KW_SAY: usize = 28;
const KW_SELECT: usize = 29;
const KW_SIGNAL: usize = 30;
const KW_THEN: usize = 31;
const KW_TRACE: usize = 32;
const KW_USE: usize = 33;
const KW_WHEN: usize = 34;

// Positions in the `CONDITIONS` table. `CALL ON` accepts a strict subset of
// what `SIGNAL ON` does, so both lists are spelled out at their use sites.
const COND_ANY: usize = 0;
const COND_ERROR: usize = 1;
const COND_FAILURE: usize = 2;
const COND_HALT: usize = 3;
const COND_LOSTDIGITS: usize = 4;
const COND_NOMETHOD: usize = 5;
const COND_NOSTRING: usize = 6;
const COND_NOTREADY: usize = 7;
const COND_NOVALUE: usize = 8;
const COND_PROPAGATE: usize = 9;
const COND_SYNTAX: usize = 10;
const COND_USER: usize = 11;

// Positions in the `PARSE_OPTIONS` table. `VALUE`, `ARG` and `PULL` appear in
// `SUB_KEYWORDS` as well, at different indices and meaning different things,
// which is why the two tables are separate and never conflated.
const POPT_ARG: usize = 0;
const POPT_CASELESS: usize = 1;
const POPT_LINEIN: usize = 2;
const POPT_LOWER: usize = 3;
const POPT_PULL: usize = 4;
const POPT_SOURCE: usize = 5;
const POPT_UPPER: usize = 6;
const POPT_VALUE: usize = 7;
const POPT_VAR: usize = 8;
const POPT_VERSION: usize = 9;

// Positions in the `SUB_KEYWORDS` table, pinned the same way by
// `tests::keyword_indices_still_name_their_own_spellings`.
const SUB_ADDITIONAL: usize = 0;
const SUB_APPEND: usize = 1;
const SUB_ARG: usize = 2;
const SUB_ARGUMENTS: usize = 3;
const SUB_ARRAY: usize = 4;
const SUB_BY: usize = 5;
const SUB_CASE: usize = 6;
const SUB_CLASS: usize = 7;
const SUB_CONTINUE: usize = 8;
const SUB_COUNTER: usize = 9;
const SUB_DESCRIPTION: usize = 10;
const SUB_DIGITS: usize = 11;
const SUB_ENGINEERING: usize = 12;
const SUB_ERROR: usize = 13;
const SUB_EXIT: usize = 14;
const SUB_EXPOSE: usize = 15;
const SUB_FOR: usize = 17;
const SUB_FOREVER: usize = 18;
const SUB_FORM: usize = 19;
const SUB_FUZZ: usize = 20;
const SUB_INDEX: usize = 21;
const SUB_INPUT: usize = 23;
const SUB_ITEM: usize = 24;
const SUB_LABEL: usize = 25;
const SUB_LOCAL: usize = 26;
const SUB_MESSAGE: usize = 27;
const SUB_NAME: usize = 28;
const SUB_NORMAL: usize = 30;
const SUB_OFF: usize = 31;
const SUB_ON: usize = 32;
const SUB_OUTPUT: usize = 33;
const SUB_OVER: usize = 34;
const SUB_REPLACE: usize = 35;
const SUB_RETURN: usize = 36;
const SUB_SCIENTIFIC: usize = 37;
const SUB_STEM: usize = 38;
const SUB_STREAM: usize = 39;
const SUB_STRICT: usize = 40;
const SUB_TO: usize = 42;
const SUB_UNTIL: usize = 44;
const SUB_USING: usize = 45;
const SUB_VALUE: usize = 46;
const SUB_WHEN: usize = 47;
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
/// of the following. Every one but the last two is a `translateBlock` error in
/// the C++ rather than a `nextInstruction` one:
///
/// * 7.1 and 7.2, a `SELECT` with no `WHEN` at all and an instruction other
///   than `WHEN`/`OTHERWISE`/`END` inside one. Measured: `select` / `end` is
///   7.1, `select case 1` / `otherwise nop` / `end` is 7.1, and
///   `select` / `nop` / `end` is 7.2.
/// * 8.2, an `ELSE` with no `THEN` above it.
/// * 9.1 and 9.2, a `WHEN` or `OTHERWISE` outside a `SELECT`.
/// * 10.1, 10.2, 10.3 and 10.7, an `END` with no block, one closing a `THEN`
///   or an `ELSE`, and one naming a block that is not the open one. The last
///   two differ by what the `END` failed to close: measured, `do` / `end 1` is
///   10.3 and `select` / `end 1` is 10.7.
/// * 14.x, an unclosed `DO`, `SELECT`, `THEN` or `ELSE` at the end of a body.
/// * The misplaced-label errors, which depend on the open block.
/// * 99.907 and 99.910, `EXPOSE` and `USE LOCAL` not being the first
///   instruction, which read `lastInstruction`.
/// * The chain indices themselves: which instruction an `IF` skips to, which
///   block an `END` closes, which `SELECT` a `WHEN` belongs to.
/// * 35.934 in place of 35.929 for a `WHEN` inside `SELECT CASE`, whose parse
///   needs the enclosing block to pick `parseCaseWhenList`. TWO things change,
///   not one: that sub-number, and the `When` node's shape, because
///   `parseCaseWhenList` builds a list of case values where `parseLogical`
///   builds an AND. `tests::a_when_inside_select_case_still_gets_the_interim_35_929`
///   pins the interim state so the change is deliberate. Raised in `whenNew`,
///   not in `translateBlock`.
/// * 99.913, a `GUARD ON WHEN` expression that references no variable exposed
///   at that point. Raised in `guardNew`, not in `translateBlock`, but it needs
///   a per-body set of exposed variables that this task does not keep. The
///   measurements are on `guard`.
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
    // (`LanguageParser.cpp:1341`). This is the one shape with no offending
    // clause to report against, so the IF's own byte is the reported one:
    // measured, `nop` / `nop` / `if 1 = 1` reports line 3.
    if let Some((which, byte)) = cursor.take_expected_then() {
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
    if let Some((which, byte)) = cursor.take_expected_then() {
        // A label clause is excluded, which is the one place this diverges
        // from the C++. There the label is still one clause with whatever
        // follows the colon, so a label spelled THEN becomes the THEN and the
        // leftover `:` then fails with 35.1. Task 3.4 has already split the
        // colon off here, leaving nothing that can fail, so the missing-THEN
        // error fires instead. Both reject `if 1 = 1` followed by
        // `then: nop`, measured 35.1 for both the IF and the WHEN spelling,
        // and only the number differs.
        //
        // `tests::a_label_after_an_if_is_rejected_by_the_label_guard` pins
        // both spellings. Without that test the guard could be deleted and
        // nothing would fail, and then the program would start being
        // ACCEPTED, with the label silently discarded.
        if parser.clause.label.is_some() || parser.first_keyword() != Some(KW_THEN) {
            // Reported against THIS clause, the offending one, and not against
            // the IF. The error carries both positions and only a source with
            // blank lines between them can tell which is which: measured,
            // `nop` / `if 1 = 1` / blank / blank / `nop` reports
            // `line 5: THEN expected` with `IF instruction on line 2` as a
            // substitution. This phase reproduces the reported line and not
            // the substitutions, so `byte` -- the IF's -- is deliberately
            // unused here. It is used where there is no offending clause, in
            // `parse_instructions`.
            let _ = byte;
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
        self.cursor.peek_real(self.ctx.tokens)
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
        let i = self.cursor.advance_real(self.ctx.tokens)?;
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
    /// The C++ would `resetPosition` backwards. Nothing here ever does, so
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
        self.trial_from(self.cursor.position())
    }

    /// A second cursor over the same clause, positioned at token `at`.
    ///
    /// Used to re-present a token already consumed, which is the C++'s
    /// `previousToken()` followed by a parse that may fail. The range is still
    /// the whole clause, so an error reports against the clause's first byte.
    fn trial_from(&self, at: usize) -> TokenCursor {
        let mut trial = TokenCursor::new(self.clause.tokens.clone());
        while trial.position() < at {
            trial.advance();
        }
        trial
    }

    fn at_end(&self) -> bool {
        self.peek_real_index().is_none()
    }

    /// `requiredEndOfClause`: nothing may follow.
    fn required_end(&self, code: u16, sub: u16) -> Result<(), ParseError> {
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
        let target = Expr::new(symbol_kind(id, class), target_span);
        let value = match op {
            None => value,
            Some(op) => Expr::binary(op, target.clone(), value),
        };
        Ok(Some(InstructionKind::Assignment { target, value }))
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
            // `sayNew`, `pushNew` and `queueNew` are the same two lines: an
            // optional expression to the end of the clause. PUSH and QUEUE
            // share `RexxInstructionQueue` and differ only in the instruction
            // type, which is why they are separate variants here.
            KW_SAY => {
                self.next_real();
                let expression = self.opt_expr(Terminators::EOC)?;
                Ok(self.finish(cursor, InstructionKind::Say { expression }))
            }
            KW_PUSH => {
                self.next_real();
                let expression = self.opt_expr(Terminators::EOC)?;
                Ok(self.finish(cursor, InstructionKind::Push { expression }))
            }
            KW_QUEUE => {
                self.next_real();
                let expression = self.opt_expr(Terminators::EOC)?;
                Ok(self.finish(cursor, InstructionKind::Queue { expression }))
            }
            KW_DROP => {
                self.next_real();
                let variables = self.variable_list(901)?;
                Ok(self.finish(cursor, InstructionKind::Drop { variables }))
            }
            KW_EXPOSE => {
                self.next_real();
                // `exposeNew` rejects this inside INTERPRET. Measured at run
                // time, because rexxc never parses the string:
                // `interpret "expose a"` is rc 157, Error 99.908.
                if self.ctx.source.kind() == SourceKind::Interpret {
                    return Err(self.error(99, 908));
                }
                let variables = self.variable_list(902)?;
                Ok(self.finish(cursor, InstructionKind::Expose { variables }))
            }
            // `parseNew(SUBKEY_NONE)`, `parseNew(SUBKEY_ARG)` and
            // `parseNew(SUBKEY_PULL)`. The short forms have no options and no
            // source keyword, and both imply UPPER.
            KW_PARSE => {
                self.next_real();
                let body = self.parse_instruction_body(None)?;
                Ok(self.finish(cursor, InstructionKind::Parse(Box::new(body))))
            }
            KW_ARG => {
                self.next_real();
                let body = self.parse_instruction_body(Some(ParseSource::Arg))?;
                Ok(self.finish(cursor, InstructionKind::Arg(Box::new(body))))
            }
            KW_PULL => {
                self.next_real();
                let body = self.parse_instruction_body(Some(ParseSource::Pull))?;
                Ok(self.finish(cursor, InstructionKind::Pull(Box::new(body))))
            }
            // The four instructions that are an optional expression and
            // nothing else, plus the two that require one.
            KW_RETURN => {
                self.next_real();
                let expression = self.opt_expr(Terminators::EOC)?;
                Ok(self.finish(cursor, InstructionKind::Return { expression }))
            }
            KW_EXIT => {
                self.next_real();
                let expression = self.opt_expr(Terminators::EOC)?;
                Ok(self.finish(cursor, InstructionKind::Exit { expression }))
            }
            KW_REPLY => {
                self.next_real();
                // Measured at run time: `interpret "reply 1"` is Error 99.924.
                if self.ctx.source.kind() == SourceKind::Interpret {
                    return Err(self.error(99, 924));
                }
                let expression = self.opt_expr(Terminators::EOC)?;
                Ok(self.finish(cursor, InstructionKind::Reply { expression }))
            }
            KW_INTERPRET => {
                self.next_real();
                // `Error_Invalid_expression_interpret`, measured as 35.912.
                let expression = self.expr(Terminators::EOC, 912)?;
                Ok(self.finish(cursor, InstructionKind::Interpret { expression }))
            }
            KW_OPTIONS => {
                self.next_real();
                // `Error_Invalid_expression_options`, measured as 35.913.
                let expression = self.expr(Terminators::EOC, 913)?;
                Ok(self.finish(cursor, InstructionKind::Options { expression }))
            }
            KW_PROCEDURE => {
                self.next_real();
                // `procedureNew`: only `EXPOSE` may follow, and the same error
                // covers a non-symbol because both are 25.17. Measured:
                // `procedure foo` is rc 231, Error 25.17.
                let variables = match self.peek_real() {
                    None => Vec::new(),
                    Some(token) => {
                        if self.sub_keyword(token) != Some(SUB_EXPOSE) {
                            return Err(self.error(25, 17));
                        }
                        self.next_real();
                        self.variable_list(902)?
                    }
                };
                Ok(self.finish(cursor, InstructionKind::Procedure { variables }))
            }
            KW_GUARD => {
                self.next_real();
                let guard = self.guard()?;
                Ok(self.finish(cursor, InstructionKind::Guard(Box::new(guard))))
            }
            KW_FORWARD => {
                self.next_real();
                let forward = self.forward()?;
                Ok(self.finish(cursor, InstructionKind::Forward(Box::new(forward))))
            }
            KW_RAISE => {
                self.next_real();
                let raise = self.raise()?;
                Ok(self.finish(cursor, InstructionKind::Raise(Box::new(raise))))
            }
            KW_USE => {
                self.next_real();
                let use_ = self.use_instruction()?;
                Ok(self.finish(cursor, InstructionKind::Use(Box::new(use_))))
            }
            KW_ADDRESS => {
                self.next_real();
                let address = self.address()?;
                Ok(self.finish(cursor, InstructionKind::Address(Box::new(address))))
            }
            KW_NUMERIC => {
                self.next_real();
                let (setting, expression) = self.numeric()?;
                Ok(self.finish(
                    cursor,
                    InstructionKind::Numeric {
                        setting,
                        expression,
                    },
                ))
            }
            KW_TRACE => {
                self.next_real();
                let trace = self.trace()?;
                Ok(self.finish(cursor, InstructionKind::Trace(trace)))
            }
            KW_CALL => {
                self.next_real();
                let call = self.call()?;
                Ok(self.finish(cursor, InstructionKind::Call(Box::new(call))))
            }
            KW_SIGNAL => {
                self.next_real();
                let signal = self.signal()?;
                Ok(self.finish(cursor, InstructionKind::Signal(Box::new(signal))))
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
            // Unreachable: `index` came from a 35-entry table and all 35 have
            // an arm above, which `tests::keyword_indices_still_name_their_own_spellings`
            // pins. `nextInstruction`'s own default is the same
            // `Error_Interpretation_switch`.
            _ => Err(self.error(UNREACHABLE_SWITCH.0, UNREACHABLE_SWITCH.1)),
        }
    }

    /// `endNew` (`InstructionParser.cpp:2246`): an optional block name.
    ///
    /// The gate is `isSymbol()`, which is class-agnostic, so a number, a stem
    /// and a compound are all legal block names as far as the parser is
    /// concerned. Do not add a class check here. Measured, and all four are
    /// block-MATCHING errors rather than parse errors, with the number chosen
    /// by what the END failed to close: `do` / `end 1`, `end loop` and
    /// `end a.` are Error 10.3, and the same three under a `select` are
    /// Error 10.7. Only a token that is not a symbol at all is rejected here,
    /// `end "x"` with 20.909, and only extra tokens after the name, `end a b`
    /// with 21.909.
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

    /// `addressNew` (`InstructionParser.cpp:563`).
    ///
    /// `ADDRESS` with nothing after it toggles between the current environment
    /// and the previous one, which is why every field is optional.
    fn address(&mut self) -> Result<Address, ParseError> {
        let mut address = Address::default();
        let Some(index) = self.peek_real_index() else {
            return Ok(address);
        };
        let token = &self.ctx.tokens[index];
        if !matches!(token.kind.tag(), Tag::Symbol | Tag::Literal) {
            // An implicit `ADDRESS VALUE`, with the token left in place.
            // Measured: `address (e)` is rc 0.
            address.dynamic = self.opt_expr(Terminators::PARSE_WITH)?;
        } else if self.sub_keyword(token) == Some(SUB_VALUE) {
            self.seek(index + 1);
            // `Error_Invalid_expression_address`, measured as 35.914 for
            // `address value`.
            address.dynamic = Some(self.expr(Terminators::PARSE_WITH, 914)?);
        } else {
            self.seek(index + 1);
            address.environment = Some(self.value_of(token));
            if !self.at_end() {
                // The command expression stops at WITH, so what follows is
                // either that keyword or the end of the clause. This can come
                // back empty, which is `ADDRESS env WITH ...`: a configuration
                // with no command.
                address.command = self.opt_expr(Terminators::PARSE_WITH)?;
            }
        }
        let with = self
            .peek_real()
            .filter(|token| self.sub_keyword(token) == Some(SUB_WITH));
        if with.is_some() {
            self.next_real();
            address.io = Some(Box::new(self.address_with()?));
        }
        Ok(address)
    }

    /// `parseAddressWith` (`InstructionParser.cpp:670`): the `INPUT`, `OUTPUT`
    /// and `ERROR` redirections, each at most once and in any order.
    fn address_with(&mut self) -> Result<AddressIo, ParseError> {
        let mut io = AddressIo::default();
        // `Error_Symbol_expected_address_with`, measured as 20.933 for
        // `address system with`.
        if self.at_end() {
            return Err(self.error(20, 933));
        }
        let mut seen = [false; 3];
        while let Some(token) = self.next_real() {
            if token.kind.tag() != Tag::Symbol {
                return Err(self.error(20, 933));
            }
            // The three duplicate errors are 25.930, 25.931 and 25.932, one per
            // stream. Measured: `with input normal input normal` is 25.930.
            let stream = match self.sub_keyword(token) {
                Some(SUB_INPUT) => 0,
                Some(SUB_OUTPUT) => 1,
                Some(SUB_ERROR) => 2,
                // `Error_Invalid_subkeyword_address_with_option`, measured as
                // 25.934 for `address system with foo` and for `with 1`.
                _ => return Err(self.error(25, 934)),
            };
            if seen[stream] {
                return Err(self.error(25, 930 + u16::try_from(stream).expect("0, 1 or 2")));
            }
            seen[stream] = true;
            // `NORMAL` resets the stream and takes no target.
            let normal = self
                .peek_real()
                .filter(|token| self.sub_keyword(token) == Some(SUB_NORMAL));
            if normal.is_some() {
                self.next_real();
                match stream {
                    0 => io.input = Redirection::Normal,
                    1 => io.output = Redirection::Normal,
                    _ => io.error = Redirection::Normal,
                }
                continue;
            }
            // Only an output stream takes APPEND or REPLACE, and both are
            // optional.
            if stream != 0 {
                let option = self.output_option();
                if stream == 1 {
                    io.output_option = option;
                } else {
                    io.error_option = option;
                }
            }
            let target = self.redirect_target()?;
            match stream {
                0 => io.input = target,
                1 => io.output = target,
                _ => io.error = target,
            }
        }
        Ok(io)
    }

    /// `parseRedirectOutputOptions` (`InstructionParser.cpp:812`): `APPEND` or
    /// `REPLACE`, or neither, in which case nothing is consumed.
    fn output_option(&mut self) -> OutputOption {
        let Some(token) = self.peek_real() else {
            return OutputOption::Default;
        };
        let option = match self.sub_keyword(token) {
            Some(SUB_REPLACE) => OutputOption::Replace,
            Some(SUB_APPEND) => OutputOption::Append,
            // Probably one of the target keywords, so leave it in place.
            _ => return OutputOption::Default,
        };
        self.next_real();
        option
    }

    /// `parseRedirectOptions` (`InstructionParser.cpp:834`): where one stream
    /// goes.
    fn redirect_target(&mut self) -> Result<Redirection, ParseError> {
        // `Error_Invalid_subkeyword_address_with_io_option`, measured as 25.933
        // for `address system with input` and for `with input foo`.
        let Some(token) = self.next_real() else {
            return Err(self.error(25, 933));
        };
        if token.kind.tag() != Tag::Symbol {
            return Err(self.error(25, 933));
        }
        match self.sub_keyword(token) {
            Some(SUB_STEM) => {
                // `Error_Symbol_expected_after_stem_keyword`, measured as
                // 20.932 for `with input stem a`, because `a` is a variable and
                // not a stem.
                let Some(name) = self.next_real() else {
                    return Err(self.error(20, 932));
                };
                let TokenKind::Symbol {
                    id,
                    class: SymbolClass::Stem,
                } = name.kind
                else {
                    return Err(self.error(20, 932));
                };
                Ok(Redirection::Stem(id))
            }
            // Both take the constant-expression form, so a bare variable is
            // 35.1: measured, `with error using x` is rc 221 while
            // `with error using (x)` would be accepted.
            Some(index @ (SUB_STREAM | SUB_USING)) => {
                match parse_constant_expression(self.ctx, &mut self.cursor)? {
                    // `Error_Invalid_expression_missing_general`, measured as
                    // 35.935 for `with input stream`.
                    None => Err(self.error(35, 935)),
                    Some(value) => Ok(if index == SUB_STREAM {
                        Redirection::Stream(value)
                    } else {
                        Redirection::Using(value)
                    }),
                }
            }
            _ => Err(self.error(25, 933)),
        }
    }

    /// `numericNew` (`InstructionParser.cpp:2959`).
    fn numeric(&mut self) -> Result<(NumericSetting, Option<Expr>), ParseError> {
        // `Error_Symbol_expected_numeric`, measured as 20.905 for a bare
        // `numeric` and for `numeric "x"`.
        let Some(token) = self.next_real() else {
            return Err(self.error(20, 905));
        };
        if token.kind.tag() != Tag::Symbol {
            return Err(self.error(20, 905));
        }
        match self.sub_keyword(token) {
            Some(SUB_DIGITS) => Ok((NumericSetting::Digits, self.opt_expr(Terminators::EOC)?)),
            Some(SUB_FUZZ) => Ok((NumericSetting::Fuzz, self.opt_expr(Terminators::EOC)?)),
            Some(SUB_FORM) => {
                let Some(index) = self.peek_real_index() else {
                    // `NUMERIC FORM` alone resets to the package default.
                    return Ok((NumericSetting::FormDefault, None));
                };
                let token = &self.ctx.tokens[index];
                if token.kind.tag() != Tag::Symbol {
                    // An implicit `NUMERIC FORM VALUE`, with the token left in
                    // place. Measured: `numeric form (e)` is rc 0.
                    return Ok((NumericSetting::FormValue, self.opt_expr(Terminators::EOC)?));
                }
                self.seek(index + 1);
                match self.sub_keyword(token) {
                    // `Error_Invalid_data_form`, measured as 21.911 for
                    // `numeric form scientific x`.
                    Some(SUB_SCIENTIFIC) => {
                        self.required_end(21, 911)?;
                        Ok((NumericSetting::FormScientific, None))
                    }
                    Some(SUB_ENGINEERING) => {
                        self.required_end(21, 911)?;
                        Ok((NumericSetting::FormEngineering, None))
                    }
                    // `Error_Invalid_expression_form`, 35.917.
                    Some(SUB_VALUE) => Ok((
                        NumericSetting::FormValue,
                        Some(self.expr(Terminators::EOC, 917)?),
                    )),
                    // `Error_Invalid_subkeyword_form`, measured as 25.11 for
                    // `numeric form foo`.
                    _ => Err(self.error(25, 11)),
                }
            }
            // `Error_Invalid_subkeyword_numeric`, measured as 25.15 for
            // `numeric foo`.
            _ => Err(self.error(25, 15)),
        }
    }

    /// `traceNew` (`InstructionParser.cpp:4124`), in its four forms.
    ///
    /// The order of the tests is what decides the shape: a symbol or a literal
    /// is a whole number if it can be, an option string otherwise, and only a
    /// token that is neither -- nor a signed number -- becomes an expression.
    fn trace(&mut self) -> Result<Trace, ParseError> {
        let Some(index) = self.peek_real_index() else {
            return Ok(Trace::Default);
        };
        let token = &self.ctx.tokens[index];
        match &token.kind {
            TokenKind::Symbol { .. } | TokenKind::Literal { .. } => {
                // `TRACE VALUE expr` is the one symbol that is not a setting.
                if self.sub_keyword(token) == Some(SUB_VALUE) {
                    self.seek(index + 1);
                    // `Error_Invalid_expression_trace`, measured as 35.916.
                    return Ok(Trace::Value(self.expr(Terminators::EOC, 916)?));
                }
                self.seek(index + 1);
                let value = self.value_of(token);
                // `Error_Invalid_data_trace`, measured as 21.906 for
                // `trace 5 x` and for `trace r x`.
                self.required_end(21, 906)?;
                match whole_number(&value, TRACE_DIGITS) {
                    Some(skip) => Ok(Trace::Skip(skip)),
                    None => {
                        check_trace_setting(&value).map_err(|_| self.error(24, 1))?;
                        Ok(Trace::Setting(value))
                    }
                }
            }
            // `TRACE -n` and `TRACE +n`, the skip forms with a sign.
            TokenKind::Operator(op @ (Operator::Subtract | Operator::Plus)) => {
                let negate = *op == Operator::Subtract;
                self.seek(index + 1);
                // Measured: `trace -a` is rc 230, Error 26.7, so the number
                // test happens after the end-of-clause test.
                let Some(number) = self.next_real() else {
                    return Err(self.error(35, 1));
                };
                if !matches!(number.kind.tag(), Tag::Symbol | Tag::Literal) {
                    return Err(self.error(35, 1));
                }
                let value = self.value_of(number);
                self.required_end(21, 906)?;
                // `Error_Invalid_whole_number_trace`, 26.7.
                let Some(skip) = whole_number(&value, TRACE_DIGITS) else {
                    return Err(self.error(26, 7));
                };
                Ok(Trace::Skip(if negate { -skip } else { skip }))
            }
            // An implicit `TRACE VALUE`, with the token left in place.
            // Measured: `trace (e)` is rc 0.
            _ => match self.opt_expr(Terminators::EOC)? {
                Some(expression) => Ok(Trace::Value(expression)),
                None => Ok(Trace::Default),
            },
        }
    }

    /// `guardNew` (`InstructionParser.cpp:2578`).
    ///
    /// The check behind `Error_Translation_guard_expose`, 99.913, is NOT made
    /// here. The rule is that the `WHEN` expression must reference at least one
    /// variable EXPOSED AT THAT POINT, and nothing weaker: measured, all three,
    /// `guard on when 1` is 99.913 in the main program with no method and no
    /// `EXPOSE` anywhere, `expose a` then `guard on when b` is 99.913 as well,
    /// and only `expose a` then `guard on when a` is rc 0.
    ///
    /// So it is not a `translateBlock` check and not a method-only one --
    /// `guardNew` raises it itself, from the variable set `setGuard`/`getGuard`
    /// captured while the expression was parsed. It is deferred anyway, because
    /// that set is per code body and this task holds no per-body state.
    fn guard(&mut self) -> Result<Guard, ParseError> {
        // Measured at run time: `interpret "guard on"` is Error 99.912.
        if self.ctx.source.kind() == SourceKind::Interpret {
            return Err(self.error(99, 912));
        }
        // `Error_Invalid_subkeyword_guard`, measured as 25.913 for a bare
        // `guard` and for `guard foo`.
        let Some(token) = self.next_real() else {
            return Err(self.error(25, 913));
        };
        let on = match self.sub_keyword(token) {
            Some(SUB_ON) => true,
            Some(SUB_OFF) => false,
            _ => return Err(self.error(25, 913)),
        };
        let condition = match self.next_real() {
            None => None,
            Some(token) => {
                // `Error_Invalid_subkeyword_guard_on`, measured as 25.912 for
                // `guard on foo` and for `guard on 1`.
                if self.sub_keyword(token) != Some(SUB_WHEN) {
                    return Err(self.error(25, 912));
                }
                // `Error_Invalid_expression_guard` is 35.921, and like every
                // other logical list the empty case is 35.929 first.
                Some(self.logical(Terminators::EOC, 929)?)
            }
        };
        Ok(Guard { on, condition })
    }

    /// `forwardNew` (`InstructionParser.cpp:2427`): six options in any order,
    /// each at most once.
    fn forward(&mut self) -> Result<Forward, ParseError> {
        // Measured at run time: `interpret "forward to 1"` is Error 99.923.
        if self.ctx.source.kind() == SourceKind::Interpret {
            return Err(self.error(99, 923));
        }
        let mut forward = Forward::default();
        while let Some(token) = self.next_real() {
            // `Error_Invalid_subkeyword_forward_option`, measured as 25.916.
            if token.kind.tag() != Tag::Symbol {
                return Err(self.error(25, 916));
            }
            match self.sub_keyword(token) {
                // `Error_Invalid_subkeyword_to` is 25.917 and
                // `Error_Invalid_expression_forward_to` is 35.925. Measured:
                // `forward to 1 to 2` is 25.917 and `forward to` is 35.925.
                Some(SUB_TO) => self.forward_option(&mut forward.to, 917, 925)?,
                Some(SUB_CLASS) => self.forward_option(&mut forward.class, 921, 928)?,
                Some(SUB_MESSAGE) => self.forward_option(&mut forward.message, 922, 927)?,
                // ARGUMENTS and ARRAY exclude each other and share one error,
                // `Error_Invalid_subkeyword_arguments`, 25.918.
                Some(SUB_ARGUMENTS) => {
                    if forward.arguments.is_some() || forward.array.is_some() {
                        return Err(self.error(25, 918));
                    }
                    let Some(value) = parse_constant_expression(self.ctx, &mut self.cursor)? else {
                        return Err(self.error(35, 926));
                    };
                    forward.arguments = Some(value);
                }
                Some(SUB_ARRAY) => {
                    if forward.arguments.is_some() || forward.array.is_some() {
                        return Err(self.error(25, 918));
                    }
                    // `Error_Invalid_expression_raise_list`, shared with
                    // RAISE ARRAY. Measured: `forward array 1` is 35.924.
                    let open = self
                        .next_real()
                        .filter(|token| token.kind.tag() == Tag::LeftParen);
                    if open.is_none() {
                        return Err(self.error(35, 924));
                    }
                    forward.array = Some(self.arg_list(Some(Tag::RightParen))?);
                }
                // `Error_Invalid_subkeyword_continue`, measured as 25.919.
                Some(SUB_CONTINUE) => {
                    if forward.continue_ {
                        return Err(self.error(25, 919));
                    }
                    forward.continue_ = true;
                }
                _ => return Err(self.error(25, 916)),
            }
        }
        Ok(forward)
    }

    /// One `FORWARD` option that takes a constant expression.
    ///
    /// `duplicate` is the sub-number of error 25 for a repeat and `missing`
    /// that of error 35 for an absent expression.
    fn forward_option(
        &mut self,
        slot: &mut Option<Expr>,
        duplicate: u16,
        missing: u16,
    ) -> Result<(), ParseError> {
        if slot.is_some() {
            return Err(self.error(25, duplicate));
        }
        match parse_constant_expression(self.ctx, &mut self.cursor)? {
            Some(value) => {
                *slot = Some(value);
                Ok(())
            }
            None => Err(self.error(35, missing)),
        }
    }

    /// `raiseNew` (`InstructionParser.cpp:3512`): a condition name, then five
    /// options.
    fn raise(&mut self) -> Result<Raise, ParseError> {
        // `Error_Symbol_expected_raise`, measured as 20.914 for a bare
        // `raise`.
        let Some(token) = self.next_real() else {
            return Err(self.error(20, 914));
        };
        let TokenKind::Symbol { id, .. } = token.kind else {
            return Err(self.error(20, 914));
        };
        let which = self.ctx.keywords.conditions.index_of(id);
        let mut raise = Raise {
            condition: self.value_of(token),
            propagate: false,
            rc: None,
            description: None,
            additional: None,
            array: None,
            result: None,
        };
        match which {
            // These three take a value after the condition name. SYNTAX also
            // needs run-time work, which the flag records there and the node
            // shape records here.
            Some(COND_FAILURE | COND_ERROR | COND_SYNTAX) => {
                let Some(value) = parse_constant_expression(self.ctx, &mut self.cursor)? else {
                    // Measured: `raise syntax` with no value is 35.1.
                    return Err(self.error(35, 1));
                };
                raise.rc = Some(value);
            }
            Some(COND_USER) => {
                // `Error_Symbol_expected_user`, measured as 20.915.
                let Some(name) = self.next_real() else {
                    return Err(self.error(20, 915));
                };
                let TokenKind::Symbol { .. } = name.kind else {
                    return Err(self.error(20, 915));
                };
                let mut composed = b"USER ".to_vec();
                composed.extend_from_slice(&self.value_of(name));
                raise.condition = composed.into_boxed_slice();
            }
            Some(COND_PROPAGATE) => raise.propagate = true,
            Some(
                COND_HALT | COND_NOMETHOD | COND_NOSTRING | COND_NOTREADY | COND_NOVALUE
                | COND_LOSTDIGITS,
            ) => {}
            // `Error_Invalid_subkeyword_raise`. ANY reaches here, because
            // nothing can be raised for every condition at once: measured,
            // `raise any` is rc 231, Error 25.906, as is `raise foo`.
            _ => return Err(self.error(25, 906)),
        }

        while let Some(token) = self.next_real() {
            // `Error_Invalid_subkeyword_raiseoption`, measured as 25.907.
            if token.kind.tag() != Tag::Symbol {
                return Err(self.error(25, 907));
            }
            match self.sub_keyword(token) {
                // `Error_Invalid_subkeyword_description` is 25.908 and
                // `Error_Invalid_expression_raise_description` is 35.922.
                Some(SUB_DESCRIPTION) => {
                    self.forward_option(&mut raise.description, 908, 922)?;
                }
                // ADDITIONAL and ARRAY exclude each other and share 25.909.
                Some(SUB_ADDITIONAL) => {
                    if raise.additional.is_some() || raise.array.is_some() {
                        return Err(self.error(25, 909));
                    }
                    let Some(value) = parse_constant_expression(self.ctx, &mut self.cursor)? else {
                        return Err(self.error(35, 923));
                    };
                    raise.additional = Some(value);
                }
                Some(SUB_ARRAY) => {
                    if raise.additional.is_some() || raise.array.is_some() {
                        return Err(self.error(25, 909));
                    }
                    let open = self
                        .next_real()
                        .filter(|token| token.kind.tag() == Tag::LeftParen);
                    if open.is_none() {
                        return Err(self.error(35, 924));
                    }
                    raise.array = Some(self.arg_list(Some(Tag::RightParen))?);
                }
                // RETURN and EXIT exclude each other, share
                // `Error_Invalid_subkeyword_result` (25.911), and both take an
                // OPTIONAL value: measured, `raise error 1 return` is rc 0.
                Some(index @ (SUB_RETURN | SUB_EXIT)) => {
                    if raise.result.is_some() {
                        return Err(self.error(25, 911));
                    }
                    let value = parse_constant_expression(self.ctx, &mut self.cursor)?;
                    raise.result = Some(RaiseResult {
                        exit: index == SUB_EXIT,
                        value,
                    });
                }
                _ => return Err(self.error(25, 907)),
            }
        }
        Ok(raise)
    }

    /// `useNew` (`InstructionParser.cpp:4267`) and `useLocalNew` (`:2349`).
    fn use_instruction(&mut self) -> Result<Use, ParseError> {
        let first = self.peek_real();
        if first.and_then(|token| self.sub_keyword(token)) == Some(SUB_LOCAL) {
            self.next_real();
            return self.use_local();
        }
        let strict = first.and_then(|token| self.sub_keyword(token)) == Some(SUB_STRICT);
        if strict {
            self.next_real();
        }
        // `Error_Invalid_subkeyword_use` is 25.905 and `_use_strict` is
        // 25.929. Measured: `use foo` is 25.905 and `use strict foo` is 25.929.
        let arg = self
            .next_real()
            .filter(|token| self.sub_keyword(token) == Some(SUB_ARG));
        if arg.is_none() {
            return Err(self.error(25, if strict { 929 } else { 905 }));
        }
        let mut targets: Vec<Option<UseTarget>> = Vec::new();
        let mut allow_optionals = false;
        while let Some(index) = self.peek_real_index() {
            let token = &self.ctx.tokens[index];
            // A bare comma is an omitted position, which keeps the argument
            // numbering right.
            if token.kind.tag() == Tag::Comma {
                self.seek(index + 1);
                targets.push(None);
                continue;
            }
            // `...` ends the list and stops argument-count checking.
            // `Error_Translation_use_arg_ellipsis`, measured as 99.930 for
            // `use arg ..., a`.
            if token.kind.tag() == Tag::Symbol && &*self.value_of(token) == b"..." {
                self.seek(index + 1);
                allow_optionals = true;
                if !self.at_end() {
                    return Err(self.error(99, 930));
                }
                break;
            }
            // `>a` and `<a` alias the caller's variable instead of copying it.
            if matches!(
                token.kind,
                TokenKind::Operator(Operator::GreaterThan | Operator::LessThan)
            ) {
                self.seek(index + 1);
                // `Error_Symbol_expected_after_use_arg_reference`, measured as
                // 20.931 for `use arg >a.b`, because a compound cannot be
                // aliased.
                let Some(name) = self.next_real() else {
                    return Err(self.error(20, 931));
                };
                let TokenKind::Symbol { id, class } = name.kind else {
                    return Err(self.error(20, 931));
                };
                if !matches!(class, SymbolClass::Variable | SymbolClass::Stem) {
                    return Err(self.error(20, 931));
                }
                targets.push(Some(UseTarget {
                    target: Expr::new(symbol_kind(id, class), name.span.clone()),
                    default: None,
                    alias: true,
                }));
                match self.next_real() {
                    None => break,
                    Some(next) if next.kind.tag() == Tag::Comma => continue,
                    // `Error_Translation_use_arg_reference_no_default`,
                    // measured as 99.950 for `use arg >a = 1`.
                    Some(next) if matches!(next.kind, TokenKind::Operator(Operator::Equal)) => {
                        return Err(self.error(99, 950));
                    }
                    // `Error_Variable_reference_use`, measured as 46.902.
                    Some(_) => return Err(self.error(46, 902)),
                }
            }
            // A variable or a message term. Measured: `use arg q~x` is rc 0.
            let mut trial = self.trial_from(index);
            let target = match parse_variable_or_message_term(self.ctx, &mut trial)? {
                Some(target) => {
                    self.cursor = trial;
                    target
                }
                // `Error_Variable_expected_USE`, 89.1.
                None => return Err(self.error(89, 1)),
            };
            let mut default = None;
            match self.next_real() {
                None => {
                    targets.push(Some(UseTarget {
                        target,
                        default,
                        alias: false,
                    }));
                    break;
                }
                Some(next) if next.kind.tag() == Tag::Comma => {}
                Some(next) if matches!(next.kind, TokenKind::Operator(Operator::Equal)) => {
                    // `Error_Invalid_expression_use_arg_default`, measured as
                    // 35.930 for `use arg a = `.
                    let Some(value) = parse_constant_expression(self.ctx, &mut self.cursor)? else {
                        return Err(self.error(35, 930));
                    };
                    default = Some(value);
                    match self.next_real() {
                        None => {
                            targets.push(Some(UseTarget {
                                target,
                                default,
                                alias: false,
                            }));
                            break;
                        }
                        Some(next) if next.kind.tag() == Tag::Comma => {}
                        Some(_) => return Err(self.error(35, 930)),
                    }
                }
                // `use arg a b` is 46.902, not a two-variable list.
                Some(_) => return Err(self.error(46, 902)),
            }
            targets.push(Some(UseTarget {
                target,
                default,
                alias: false,
            }));
        }
        Ok(Use::Arg {
            strict,
            allow_optionals,
            targets,
        })
    }

    /// `useLocalNew` (`InstructionParser.cpp:2349`).
    ///
    /// Close to `processVariableList` but not the same: there is no `(name)`
    /// form, a compound gets its own error, and the list may be empty.
    fn use_local(&mut self) -> Result<Use, ParseError> {
        // Measured at run time: `interpret "use local a"` is Error 99.915.
        if self.ctx.source.kind() == SourceKind::Interpret {
            return Err(self.error(99, 915));
        }
        let mut variables = Vec::new();
        while let Some(token) = self.next_real() {
            // `Error_Symbol_expected_use_local`, 20.927.
            let TokenKind::Symbol { id, class } = token.kind else {
                return Err(self.error(20, 927));
            };
            need_variable_class(class, self.clause_byte)?;
            // `Error_Translation_use_local_compound`, measured as 99.948 for
            // `use local a.b`: only a simple variable or a stem can be local.
            if class == SymbolClass::Compound {
                return Err(self.error(99, 948));
            }
            variables.push(VariableRef::Direct(id));
        }
        Ok(Use::Local { variables })
    }

    /// `callNew` (`InstructionParser.cpp:1147`) and the three constructors
    /// above it, which are four distinct instruction objects in the C++.
    fn call(&mut self) -> Result<Call, ParseError> {
        // `Error_Symbol_or_string_call`, measured as 19.2 for a bare `call`.
        let Some(token) = self.next_real() else {
            return Err(self.error(19, 2));
        };
        match &token.kind {
            TokenKind::Symbol { id, .. } => {
                // `CALL ns:name`. The colon is looked for with `nextToken`, so
                // no blank may sit before it, and none can: a blank is only a
                // token when a symbol, a literal, `(` or `[` follows it.
                if self.peek_token(0).map(|token| token.kind.tag()) == Some(Tag::Colon) {
                    self.cursor.advance();
                    // `Error_Symbol_expected_qualified_call`, measured as
                    // 20.922 for `call ns:`.
                    let Some(name) = self.next_real() else {
                        return Err(self.error(20, 922));
                    };
                    let TokenKind::Symbol { id: name, .. } = name.kind else {
                        return Err(self.error(20, 922));
                    };
                    let args = self.arg_list(None)?;
                    return Ok(Call::Qualified {
                        namespace: *id,
                        name,
                        args,
                    });
                }
                match self.ctx.keywords.sub_keywords.index_of(*id) {
                    Some(index @ (SUB_ON | SUB_OFF)) => {
                        Ok(Call::Trap(self.condition_trap(index == SUB_ON, true)?))
                    }
                    _ => {
                        let name = self.value_of(token);
                        let args = self.arg_list(None)?;
                        Ok(Call::Named {
                            name,
                            literal: false,
                            args,
                        })
                    }
                }
            }
            // `CALL "name"` never resolves to an internal label, which is what
            // `noInternal` records.
            TokenKind::Literal { value } => {
                let name = value.clone();
                let args = self.arg_list(None)?;
                Ok(Call::Named {
                    name,
                    literal: true,
                    args,
                })
            }
            // `CALL (expr) args`, whose target is only known at run time.
            TokenKind::LeftParen => {
                let Some(target) = parse_paren_expression(self.ctx, &mut self.cursor)? else {
                    // `Error_Invalid_expression_call`.
                    return Err(self.error(35, 932));
                };
                let args = self.arg_list(None)?;
                Ok(Call::Dynamic { target, args })
            }
            _ => Err(self.error(19, 2)),
        }
    }

    /// `signalNew` (`InstructionParser.cpp:4035`) and the two constructors
    /// above it.
    fn signal(&mut self) -> Result<Signal, ParseError> {
        // `Error_Symbol_or_string_signal`, measured as 19.4 for a bare
        // `signal`.
        let Some(index) = self.peek_real_index() else {
            return Err(self.error(19, 4));
        };
        let token = &self.ctx.tokens[index];
        match &token.kind {
            TokenKind::Symbol { id, .. } => {
                self.seek(index + 1);
                match self.ctx.keywords.sub_keywords.index_of(*id) {
                    Some(index @ (SUB_ON | SUB_OFF)) => {
                        Ok(Signal::Trap(self.condition_trap(index == SUB_ON, false)?))
                    }
                    // `SIGNAL VALUE expr`.
                    Some(SUB_VALUE) => self.dynamic_signal(),
                    _ => {
                        let name = self.value_of(token);
                        // `Error_Invalid_data_signal`. Measured: `signal lab x`
                        // and `signal 1+1` are both rc 235, Error 21.905,
                        // because a number is a symbol and so a label name.
                        self.required_end(21, 905)?;
                        Ok(Signal::Label(name))
                    }
                }
            }
            TokenKind::Literal { value } => {
                let name = value.clone();
                self.seek(index + 1);
                self.required_end(21, 905)?;
                Ok(Signal::Label(name))
            }
            // Anything else is an implicit `SIGNAL VALUE`, with the token left
            // in place for the expression. Measured: `signal (e)` is rc 0.
            _ => self.dynamic_signal(),
        }
    }

    /// `dynamicSignalNew` (`InstructionParser.cpp:3899`).
    fn dynamic_signal(&mut self) -> Result<Signal, ParseError> {
        // `Error_Invalid_expression_signal`.
        let target = self.expr(Terminators::EOC, 915)?;
        Ok(Signal::Value(target))
    }

    /// `callOnNew` (`InstructionParser.cpp:961`) and `signalOnNew` (`:3925`),
    /// which differ only in which conditions they accept and in four error
    /// numbers.
    ///
    /// `is_call` selects those: `CALL ON` accepts a strict subset of the
    /// conditions, because a call trap cannot resume from the conditions that
    /// have no continuation point.
    fn condition_trap(&mut self, on: bool, is_call: bool) -> Result<ConditionTrap, ParseError> {
        // `Error_Symbol_expected_on` is 20.911 and `Error_Symbol_expected_off`
        // is 20.912. Measured: `call on` is 20.911 and `call off` is 20.912.
        let missing = if on { 911 } else { 912 };
        let Some(token) = self.next_real() else {
            return Err(self.error(20, missing));
        };
        let TokenKind::Symbol { id, .. } = token.kind else {
            return Err(self.error(20, missing));
        };
        let condition = self.ctx.keywords.conditions.index_of(id);
        // `Error_Invalid_subkeyword_callon` is 25.1 and `_calloff` 25.2;
        // `_signalon` is 25.3 and `_signaloff` 25.4.
        let bad = match (is_call, on) {
            (true, true) => 1,
            (true, false) => 2,
            (false, true) => 3,
            (false, false) => 4,
        };
        let rejected = match condition {
            None => true,
            Some(COND_PROPAGATE) => true,
            // ANY is accepted by both, which is easy to assume otherwise:
            // measured, `call on any` and `signal on any` are both rc 0.
            Some(COND_ANY) => false,
            // Measured: `call on syntax` and `call on novalue` are 25.1, where
            // `signal on syntax` is rc 0.
            Some(COND_SYNTAX | COND_NOVALUE | COND_LOSTDIGITS | COND_NOMETHOD | COND_NOSTRING) => {
                is_call
            }
            _ => false,
        };
        if rejected {
            return Err(self.error(25, bad));
        }
        let mut label;
        let condition_name;
        if condition == Some(COND_USER) {
            // `Error_Symbol_expected_user`, measured as 20.915 for
            // `call on user`.
            let Some(name) = self.next_real() else {
                return Err(self.error(20, 915));
            };
            let TokenKind::Symbol { .. } = name.kind else {
                return Err(self.error(20, 915));
            };
            let name = self.value_of(name);
            // The condition's own name is `USER name`, built the way
            // `concatToCstring("USER ")` builds it.
            let mut composed = b"USER ".to_vec();
            composed.extend_from_slice(&name);
            condition_name = composed.into_boxed_slice();
            label = Some(name);
        } else {
            let name = self.value_of(token);
            condition_name = name.clone();
            label = Some(name);
        }

        if on {
            if let Some(token) = self.next_real() {
                // `Error_Invalid_subkeyword_callonname` is 25.914 and
                // `_signalonname` is 25.915.
                let name_sub = if is_call { 914 } else { 915 };
                if self.sub_keyword(token) != Some(SUB_NAME) {
                    return Err(self.error(25, name_sub));
                }
                // `Error_Symbol_or_string_name`, measured as 19.3 for
                // `call on error name`.
                let Some(target) = self.next_real() else {
                    return Err(self.error(19, 3));
                };
                if !matches!(target.kind.tag(), Tag::Symbol | Tag::Literal) {
                    return Err(self.error(19, 3));
                }
                label = Some(self.value_of(target));
                // `Error_Invalid_data_name`, measured as 21.903.
                self.required_end(21, 903)?;
            }
        } else {
            // The OFF form has no label at all, which is how the C++ tells the
            // two apart at run time.
            label = None;
            // `Error_Invalid_data_condition`, measured as 21.904 for
            // `call off error x`.
            self.required_end(21, 904)?;
        }

        Ok(ConditionTrap {
            on,
            condition: condition_name,
            label,
        })
    }

    /// `parseArgList` with no bracket to match, which is the form every
    /// instruction uses.
    fn arg_list(&mut self, closer: Option<Tag>) -> Result<Vec<Option<Expr>>, ParseError> {
        parse_arg_list(self.ctx, &mut self.cursor, closer)
    }

    /// `parseNew` (`InstructionParser.cpp:3102`), shared by `PARSE`, `ARG` and
    /// `PULL`.
    ///
    /// `short_form` is the source the `ARG` and `PULL` spellings imply. When it
    /// is present there are no options and no source keyword to parse, and
    /// UPPER is implied.
    fn parse_instruction_body(
        &mut self,
        short_form: Option<ParseSource>,
    ) -> Result<Parse, ParseError> {
        let mut upper = short_form.is_some();
        let mut lower = false;
        let mut caseless = false;
        let source = match short_form {
            Some(source) => source,
            None => {
                // The option modifiers come first and each only once. A
                // repeat falls through to the source-keyword switch, where it
                // is an unknown source: measured, `parse upper upper arg a` is
                // rc 231, Error 25.12.
                let option = loop {
                    let Some(token) = self.next_real() else {
                        return Err(self.error(20, 903));
                    };
                    if token.kind.tag() != Tag::Symbol {
                        return Err(self.error(20, 903));
                    }
                    let option = self.parse_option(token);
                    match option {
                        Some(POPT_UPPER) if !upper && !lower => upper = true,
                        Some(POPT_LOWER) if !upper && !lower => lower = true,
                        Some(POPT_CASELESS) if !caseless => caseless = true,
                        _ => break option,
                    }
                };
                match option {
                    Some(POPT_ARG) => ParseSource::Arg,
                    Some(POPT_LINEIN) => ParseSource::LineIn,
                    Some(POPT_PULL) => ParseSource::Pull,
                    Some(POPT_SOURCE) => ParseSource::Source,
                    Some(POPT_VERSION) => ParseSource::Version,
                    Some(POPT_VAR) => {
                        // Measured: `parse var` is 20.904 and `parse var 1 a`
                        // is 31.2, so the symbol test and the variable test are
                        // separate.
                        let Some(name) = self.next_real() else {
                            return Err(self.error(20, 904));
                        };
                        let TokenKind::Symbol { id, class } = name.kind else {
                            return Err(self.error(20, 904));
                        };
                        need_variable(self.ctx, id, class, self.clause_byte)?;
                        ParseSource::Var(id)
                    }
                    Some(POPT_VALUE) => {
                        // The expression is optional and defaults to the null
                        // string: measured, `parse value with a` is rc 0.
                        let value = self.opt_expr(Terminators::PARSE_WITH)?;
                        // `Error_Invalid_template_with`. Measured:
                        // `parse value "x" a` is rc 218, Error 38.3.
                        let with = self
                            .next_real()
                            .filter(|token| self.sub_keyword(token) == Some(SUB_WITH));
                        if with.is_none() {
                            return Err(self.error(38, 3));
                        }
                        ParseSource::Value(value)
                    }
                    // `Error_Invalid_subkeyword_parse`.
                    _ => return Err(self.error(25, 12)),
                }
            }
        };
        let template = self.parse_template(caseless)?;
        Ok(Parse {
            source,
            upper,
            lower,
            caseless,
            template,
        })
    }

    /// `RexxToken::parseOption` (`KeywordConstants.cpp:551`), which is a table
    /// of its own and not the sub-keyword table.
    fn parse_option(&self, token: &Token) -> Option<usize> {
        match &token.kind {
            TokenKind::Symbol { id, .. } => self.ctx.keywords.parse_options.index_of(*id),
            _ => None,
        }
    }

    /// The template grammar (`InstructionParser.cpp:3239`-`3418`), shared by
    /// all three spellings.
    ///
    /// One entry per trigger, with `None` marking the comma that switches to
    /// the next parse string. The trailing `End` trigger that assigns whatever
    /// is left is only emitted when there are variables waiting for it, which
    /// is what the C++ does with its `variableCount > 0` test.
    fn parse_template(&mut self, caseless: bool) -> Result<Vec<Option<ParseTrigger>>, ParseError> {
        let string_kind = if caseless {
            TriggerKind::Mixed
        } else {
            TriggerKind::String
        };
        let mut template: Vec<Option<ParseTrigger>> = Vec::new();
        let mut targets: Vec<Option<Expr>> = Vec::new();
        loop {
            let Some(index) = self.peek_real_index() else {
                if !targets.is_empty() {
                    template.push(Some(ParseTrigger {
                        kind: TriggerKind::End,
                        value: None,
                        targets,
                    }));
                }
                break;
            };
            let token = &self.ctx.tokens[index];
            self.seek(index + 1);
            match &token.kind {
                TokenKind::Comma => {
                    if !targets.is_empty() {
                        template.push(Some(ParseTrigger {
                            kind: TriggerKind::End,
                            value: None,
                            targets: std::mem::take(&mut targets),
                        }));
                    }
                    template.push(None);
                }
                TokenKind::Operator(op) => {
                    let kind = match op {
                        Operator::Plus => TriggerKind::Plus,
                        Operator::Subtract => TriggerKind::Minus,
                        Operator::Equal => TriggerKind::Absolute,
                        Operator::LessThan => TriggerKind::MinusLength,
                        Operator::GreaterThan => TriggerKind::PlusLength,
                        // `Error_Invalid_template_trigger`. Measured:
                        // `parse arg *3` is rc 218, Error 38.1.
                        _ => return Err(self.error(38, 1)),
                    };
                    let value = self.trigger_position()?;
                    template.push(Some(ParseTrigger {
                        kind,
                        value: Some(value),
                        targets: std::mem::take(&mut targets),
                    }));
                }
                TokenKind::LeftParen => {
                    let Some(value) = parse_paren_expression(self.ctx, &mut self.cursor)? else {
                        // `Error_Invalid_expression_parse`. Measured:
                        // `parse arg +()` is rc 221, Error 35.931.
                        return Err(self.error(35, 931));
                    };
                    template.push(Some(ParseTrigger {
                        kind: string_kind,
                        value: Some(value),
                        targets: std::mem::take(&mut targets),
                    }));
                }
                TokenKind::Literal { value } => {
                    let literal = Expr::new(
                        crate::ast::ExprKind::Literal(value.clone()),
                        token.span.clone(),
                    );
                    template.push(Some(ParseTrigger {
                        kind: string_kind,
                        value: Some(literal),
                        targets: std::mem::take(&mut targets),
                    }));
                }
                TokenKind::Symbol { id, class } => {
                    let (id, class) = (*id, *class);
                    match class {
                        // A bare number is an absolute column.
                        SymbolClass::Constant => {
                            template.push(Some(ParseTrigger {
                                kind: TriggerKind::Absolute,
                                value: Some(Expr::new(symbol_kind(id, class), token.span.clone())),
                                targets: std::mem::take(&mut targets),
                            }));
                        }
                        // A lone period consumes a field and assigns nothing.
                        SymbolClass::Dummy => targets.push(None),
                        _ => {
                            // Step back onto the symbol and parse a target,
                            // which may be a message term: measured,
                            // `parse arg q~x` is rc 0.
                            let mut trial = self.trial_from(index);
                            match parse_variable_or_message_term(self.ctx, &mut trial)? {
                                Some(target) => {
                                    self.cursor = trial;
                                    targets.push(Some(target));
                                }
                                // `Error_Variable_expected_PARSE`, 89.2.
                                None => return Err(self.error(89, 2)),
                            }
                        }
                    }
                }
                _ => return Err(self.error(38, 1)),
            }
        }
        Ok(template)
    }

    /// The column a `+`, `-`, `=`, `<` or `>` trigger moves to.
    ///
    /// A numeric symbol or a parenthesised expression, and nothing else. A
    /// VARIABLE is rejected: measured, `parse arg +x a` is rc 218, Error 38.2,
    /// while `parse arg +(x) a` is rc 0.
    fn trigger_position(&mut self) -> Result<Expr, ParseError> {
        let Some(token) = self.next_real() else {
            // `Error_Invalid_template_missing`. Measured: `parse arg +` is
            // rc 218, Error 38.901.
            return Err(self.error(38, 901));
        };
        match &token.kind {
            TokenKind::LeftParen => match parse_paren_expression(self.ctx, &mut self.cursor)? {
                Some(value) => Ok(value),
                None => Err(self.error(35, 931)),
            },
            TokenKind::Symbol { id, class } => {
                // `Error_Invalid_template_position`, for a variable, a stem or
                // a compound.
                if matches!(
                    class,
                    SymbolClass::Variable | SymbolClass::Stem | SymbolClass::Compound
                ) {
                    return Err(self.error(38, 2));
                }
                Ok(Expr::new(symbol_kind(*id, *class), token.span.clone()))
            }
            _ => Err(self.error(38, 2)),
        }
    }

    /// `processVariableList` (`InstructionParser.cpp:4469`), shared by `DROP`,
    /// `EXPOSE` and `PROCEDURE EXPOSE`.
    ///
    /// `missing` is the sub-number of error 20 for a token that cannot be a
    /// variable and for an empty list, which names the instruction:
    /// `Error_Symbol_expected_drop` is 20.901 and
    /// `Error_Symbol_expected_expose` is 20.902.
    fn variable_list(&mut self, missing: u16) -> Result<Vec<VariableRef>, ParseError> {
        let mut out = Vec::new();
        while let Some(token) = self.next_real() {
            match &token.kind {
                TokenKind::Symbol { id, class } => {
                    // The CLASS test, not the spelling test. Measured, and the
                    // two disagree: `drop .5` is 31.2 from here while
                    // `drop (.5)` is 31.3 from `addVariable`'s `needVariable`.
                    need_variable_class(*class, self.clause_byte)?;
                    out.push(VariableRef::Direct(*id));
                }
                // `(name)`, whose value names the variable. This path goes
                // through `addVariable`, so its error numbers come from the
                // spelling and not from the class.
                TokenKind::LeftParen => {
                    let Some(inner) = self.next_real() else {
                        return Err(self.error(20, 906));
                    };
                    let TokenKind::Symbol { id, class } = inner.kind else {
                        return Err(self.error(20, 906));
                    };
                    need_variable(self.ctx, id, class, self.clause_byte)?;
                    out.push(VariableRef::Indirect(id));
                    match self.next_real() {
                        // `Error_Variable_reference_missing`, measured as
                        // 46.901 for `drop (v`.
                        None => return Err(self.error(46, 901)),
                        Some(close) if close.kind.tag() == Tag::RightParen => {}
                        // `Error_Variable_reference_extra`, 46.1 for
                        // `drop (v x`.
                        Some(_) => return Err(self.error(46, 1)),
                    }
                }
                _ => return Err(self.error(20, missing)),
            }
        }
        if out.is_empty() {
            return Err(self.error(20, missing));
        }
        Ok(out)
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

    /// `RexxToken::value`: the upcased spelling of a symbol, or a literal's
    /// decoded bytes.
    ///
    /// Panics on any other token, which no caller passes: every one has
    /// already tested `isSymbol` or `isSymbolOrLiteral`.
    fn value_of(&self, token: &Token) -> Box<[u8]> {
        match &token.kind {
            TokenKind::Symbol { id, .. } => Box::from(self.ctx.symbols.name(*id).as_bytes()),
            TokenKind::Literal { value } => value.clone(),
            other => panic!("value_of on {other:?}"),
        }
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
        let byte = self.clause_byte;
        let instruction = self.finish_split(cursor, kind, end_at);
        cursor.expect_then(which, byte);
        Ok(instruction)
    }
}

/// Raises the error a non-variable symbol gets in a `DROP`, `EXPOSE`,
/// `PROCEDURE EXPOSE` or `USE LOCAL` list.
///
/// `processVariableList` (`InstructionParser.cpp:4487`-`4496`) tests the
/// symbol's CLASS where `needVariable` tests its spelling, and the two
/// disagree on a constant that starts with a period. Measured, both
/// directions: `drop .5` is 31.2 from here, `drop (.5)` is 31.3 from
/// `addVariable`, and `do .5 = 1 to 2` is 31.3 as well. Reproduced as two
/// functions rather than merged into one.
fn need_variable_class(class: SymbolClass, byte: usize) -> Result<(), ParseError> {
    match class {
        SymbolClass::Variable | SymbolClass::Stem | SymbolClass::Compound => Ok(()),
        // `Error_Invalid_variable_number`.
        SymbolClass::Constant => Err(ParseError::new(31, 2, byte)),
        // `Error_Invalid_variable_period`.
        SymbolClass::Dummy | SymbolClass::DotSymbol => Err(ParseError::new(31, 3, byte)),
    }
}

/// The number of digits a `TRACE` skip count is converted under.
///
/// `traceNew` calls `requestNumber(debug_skip, number_digits())`, and
/// `number_digits()` is the parse-time `NUMERIC DIGITS`, which is the default
/// unless a `::OPTIONS DIGITS` directive changed it. That directive belongs to
/// the directive parser, so the default is what applies here.
const TRACE_DIGITS: usize = 9;

#[cfg(test)]
mod tests;
