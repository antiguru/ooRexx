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

//! Block structure: the control stack, and assembling one code body.
//!
//! Ported from `LanguageParser::translateBlock` (`LanguageParser.cpp:1176`),
//! with the errors it reaches through `blockError` (`:4180`),
//! `RexxInstructionSelect::matchEnd` (`SelectInstruction.cpp:181`) and
//! `RexxBaseBlockInstruction::matchEnd`/`matchLabel`
//! (`BaseDoInstruction.cpp:139`/`:172`).
//!
//! # Why this owns the clause loop rather than post-processing a chain
//!
//! `translateBlock` is not a pass over a finished instruction list, and it
//! cannot be reorganised into one. Three `nextInstruction` constructors read
//! block state while they parse:
//!
//! * `whenNew` (`InstructionParser.cpp:2708`) asks `topBlockInstruction()`
//!   whether a `SELECT` is open, which decides both whether the `WHEN` is legal
//!   at all (error 9.1) and which grammar its clause follows.
//! * `guardNew` (`:2646`) needs the exposed-variable table to know whether the
//!   `GUARD` expression named an exposed variable (error 99.913).
//! * `exposeNew` and `useLocalNew` (`:2315`, `:2349`) read `lastInstruction` to
//!   check that nothing precedes them (errors 99.907 and 99.910).
//!
//! So the block state has to exist while clauses are being parsed, and this
//! module drives `parse_instruction` rather than running after it.
//!
//! # What the control stack holds
//!
//! A stack of instruction indices, not of nodes, because the instructions live
//! in a `Vec` (`pushDo`/`popDo`/`topDo`/`topDoType`/`topBlockInstruction`,
//! `LanguageParser.hpp:306`-`312`). Two frames stand for something with no
//! instruction of its own: the bottom frame, which is the C++'s dummy first
//! instruction, and the frame that marks a finished `THEN` or `WHEN` branch,
//! which is the C++'s `RexxInstructionEndIf`. See `Instruction` for why that
//! marker is not a node here.

use std::collections::{BTreeMap, BTreeSet};

use crate::ast::{
    Call, CodeBody, EndStyle, EndTarget, Expr, ExprKind, Instruction, InstructionKind, LoopKind,
    ParseSource, Redirection, Signal, Trace, Use, VariableRef,
};
use crate::clause::ClauseCursor;
use crate::instruction::{missing_then_sub, parse_instruction};
use crate::token::{ParseCtx, ParseError, SymbolId, SymbolTable, Tag};

/// What a control-stack frame stands for: the `InstructionKeyword` values that
/// `pushDo` can put on the stack, and nothing else.
///
/// A bare `IF` or `WHEN` is never pushed -- the `THEN` attached to it is -- so
/// there is no variant for either, and `isControl()` therefore coincides with
/// `isBlock()` for everything that can be here.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum Control {
    /// `KEYWORD_FIRST`: the dummy instruction at the bottom of the stack, which
    /// is what tells the assembler it has closed everything it opened.
    First,
    /// `KEYWORD_SIMPLE_BLOCK`: `DO` with no control expression, the one block
    /// form that is not a loop.
    Do,
    /// Every `KEYWORD_LOOP_*` form, and `LOOP` itself.
    Loop,
    Select,
    SelectCase,
    Otherwise,
    /// `KEYWORD_IFTHEN`: the `THEN` of an `IF`.
    IfThen,
    /// `KEYWORD_WHENTHEN`: the `THEN` of a `WHEN`.
    WhenThen,
    Else,
    /// `KEYWORD_ENDTHEN`: an `IF`'s `THEN` branch has been closed and an `ELSE`
    /// may still follow.
    EndThen,
    /// `KEYWORD_ENDWHEN`: a `WHEN`'s branch has been closed.
    EndWhen,
}

impl Control {
    /// `isBlock()` (`RexxInstruction.hpp:137`, `OtherwiseInstruction.hpp:55`):
    /// whether an `END` can close this.
    ///
    /// Also answers `isControl()` for anything on the stack. The C++ splits the
    /// two because `isControl()` is additionally true for a bare `IF`
    /// (`IfInstruction.hpp:64`), and an `IF` is never pushed.
    fn is_block(self) -> bool {
        matches!(
            self,
            Control::Do
                | Control::Loop
                | Control::Select
                | Control::SelectCase
                | Control::Otherwise
        )
    }

    /// Whether this frame is a `SELECT` of either spelling, which is what the
    /// membership check and `addWhen` test.
    fn is_select(self) -> bool {
        matches!(self, Control::Select | Control::SelectCase)
    }
}

/// One control-stack frame.
#[derive(Copy, Clone, Debug)]
struct Frame {
    kind: Control,
    /// The instruction this frame stands for. `None` for `First` and for the
    /// two branch-end markers, neither of which is an instruction here.
    index: Option<usize>,
    /// For `IfThen` and `WhenThen`, the `IF` or `WHEN` the `THEN` belongs to,
    /// whose `false_target` is written when the frame is popped. `None`
    /// otherwise.
    parent: Option<usize>,
}

impl Frame {
    /// The instruction a frame that has one stands for.
    fn index(&self) -> usize {
        self.index
            .expect("this frame kind always stands for an instruction")
    }
}

/// Which enclosing block a `WHEN` found, which is all `whenNew` needs.
///
/// `topBlockInstruction()` (`LanguageParser.cpp:1772`) drills past the `THEN`
/// and branch-end frames to the innermost real block, so a `WHEN` after an
/// earlier `WHEN` in the same `SELECT` still finds that `SELECT`. Measured both
/// ways: `select` / `when 1 = 1 then nop` / `when 2 = 2 then nop` / `end` is
/// rc 0, while a `WHEN` inside a `DO` inside a `SELECT` is 9.1, because the `DO`
/// is a block and stops the search.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) enum EnclosingSelect {
    /// A plain `SELECT`, so the clause is a logical condition.
    Plain,
    /// A `SELECT CASE`, so the clause is a list of values to compare.
    Case,
    /// No block at all, or one that is not a `SELECT`: error 9.1.
    None,
}

/// One code body being assembled: the chain, the control stack, and the
/// per-body tables that `nextInstruction` reads.
pub(crate) struct Block<'a> {
    /// Held so that a variable slot's `SymbolId` can be resolved to the name
    /// `referenced` is keyed by.
    symbols: &'a SymbolTable,
    /// The chain, in `addClause` order, which is why index order is the chain.
    instructions: Vec<Instruction>,
    labels: BTreeMap<Box<[u8]>, usize>,
    control: Vec<Frame>,
    /// The `IF` or `WHEN` most recently added, and whether it was a `WHEN`.
    ///
    /// The C++ finishes the `IF` and builds its `THEN` inside one iteration,
    /// because `translateBlock` consumes the `THEN` token itself. Here the
    /// `THEN` arrives as its own clause from `parse_instruction`, so the pairing
    /// is carried across one iteration. Nothing can come between the two: a
    /// clause after an `IF` that is not its `THEN` is error 18.1 or 18.2, which
    /// `ClauseCursor` already raises.
    pending_then: Option<(usize, bool)>,
    /// `exposedVariables` (`LanguageParser.hpp:504`): the names an `EXPOSE`
    /// listed. `None` until the first `EXPOSE`, which is not the same as empty
    /// -- see `is_exposed`.
    exposed: Option<Vec<Box<[u8]>>>,
    /// `localVariables` (`:505`): the names `USE LOCAL` listed, seeded with the
    /// five the C++ seeds (`autoExpose`, `LanguageParser.cpp:2232`). `None`
    /// until a `USE LOCAL`.
    local: Option<Vec<Box<[u8]>>>,
    /// Every variable name referenced by an instruction already in the chain,
    /// which is the part of `variables` (`LanguageParser.hpp:502`) that is
    /// observable from here.
    ///
    /// It exists for one reason: `addCompound` (`LanguageParser.cpp:2124`)
    /// returns early on a cache hit, BEFORE reaching the `addStem` and
    /// `addSimpleVariable` calls that would capture a guard variable, where
    /// `addSimpleVariable` and `addStem` themselves capture unconditionally and
    /// say so in a comment. So a compound reference feeds a `GUARD ... WHEN`
    /// only the first time that exact spelling appears in the body. See
    /// `compound_is_cached`.
    referenced: BTreeSet<Box<[u8]>>,
}

impl<'a> Block<'a> {
    fn new(symbols: &'a SymbolTable) -> Self {
        Block {
            symbols,
            instructions: Vec::new(),
            labels: BTreeMap::new(),
            // `pushDo(instruction)` on the dummy first instruction, which is
            // the bottom of the stack and also what "everything is closed"
            // means (`LanguageParser.cpp:1188`).
            control: vec![Frame {
                kind: Control::First,
                index: None,
                parent: None,
            }],
            pending_then: None,
            exposed: None,
            local: None,
            referenced: BTreeSet::new(),
        }
    }

    // ---- what `nextInstruction` reads ----

    /// `topBlockInstruction()`, reduced to the answer `whenNew` acts on.
    pub(crate) fn enclosing_select(&self) -> EnclosingSelect {
        match self
            .control
            .iter()
            .rev()
            .find(|frame| frame.kind.is_block())
            .map(|frame| frame.kind)
        {
            Some(Control::Select) => EnclosingSelect::Plain,
            Some(Control::SelectCase) => EnclosingSelect::Case,
            _ => EnclosingSelect::None,
        }
    }

    /// `lastInstruction->isType(KEYWORD_FIRST)`: whether nothing has been added
    /// to this body yet, which is what `EXPOSE` and `USE LOCAL` require.
    ///
    /// A label counts as something, because `addClause` adds one: measured,
    /// `::method m` / `lab:` / `expose a` is 99.907.
    pub(crate) fn at_body_start(&self) -> bool {
        self.instructions.is_empty()
    }

    /// `isExposed` (`LanguageParser.cpp:1991`).
    ///
    /// The three cases are ordered, and the order is what makes `USE LOCAL`
    /// mean the opposite of `EXPOSE`: with an `EXPOSE` present only the listed
    /// names are exposed, with a `USE LOCAL` present every name EXCEPT the
    /// listed ones is, and with neither nothing is. Measured all three:
    /// `expose a` / `guard on when b` is 99.913, `use local b` /
    /// `guard on when a` is rc 0, and `guard on when a` with neither is 99.913.
    pub(crate) fn is_exposed(&self, name: &[u8]) -> bool {
        if let Some(exposed) = &self.exposed {
            return exposed.iter().any(|n| n.as_ref() == name);
        }
        if let Some(local) = &self.local {
            return !local.iter().any(|n| n.as_ref() == name);
        }
        false
    }

    /// Whether a compound variable of this exact spelling has already been
    /// referenced by an instruction in this body, which makes `addCompound`
    /// return it from the cache and capture nothing.
    ///
    /// Measured, and the direction is the surprising one: an EARLIER reference
    /// makes a LATER guard illegal. `::method m` / `expose a.` /
    /// `guard on when a.1` is rc 0, and inserting `say a.1` between the two is
    /// 99.913. Seventeen shapes measured, including that two guards on one
    /// compound reject the second, that the reverse order is accepted, that a
    /// simple variable and a stem are unaffected because their own `addVariable`
    /// paths capture unconditionally, and that the cache is per body.
    ///
    /// Almost certainly an upstream defect rather than a design: the comment on
    /// the capture call in `addSimpleVariable` (`LanguageParser.cpp:2069`) says
    /// "we need to always perform the capturing test", and `addCompound`'s early
    /// return defeats exactly that. The oracle defines behaviour, so it is
    /// reproduced.
    pub(crate) fn compound_is_cached(&self, name: &[u8]) -> bool {
        self.referenced.contains(name)
    }

    /// `expose` (`LanguageParser.cpp:2218`), called for each symbol an `EXPOSE`
    /// lists.
    ///
    /// Not called for the `EXPOSE (list)` indirect form, which
    /// `processVariableList` handles on a different path
    /// (`InstructionParser.cpp:4505`): measured, `::method m` / `expose (a)` /
    /// `guard on when a` is 99.913, so an indirect name is not exposed as far as
    /// this check is concerned.
    pub(crate) fn expose(&mut self, name: Box<[u8]>) {
        self.exposed.get_or_insert_with(Vec::new).push(name);
    }

    /// `autoExpose` (`LanguageParser.cpp:2232`): a `USE LOCAL` inverts the
    /// exposure rule, and seeds the local list with the five special names.
    ///
    /// Measured, the seeding is observable: `use local a` / `guard on when self`
    /// is 99.913, because `SELF` is local and so not exposed.
    pub(crate) fn auto_expose(&mut self) {
        self.local = Some(
            [b"SUPER".as_slice(), b"SELF", b"RC", b"RESULT", b"SIGL"]
                .into_iter()
                .map(Box::from)
                .collect(),
        );
    }

    /// `localVariable` (`LanguageParser.cpp:2250`), called for each symbol a
    /// `USE LOCAL` lists.
    pub(crate) fn local_variable(&mut self, name: Box<[u8]>) {
        self.local.get_or_insert_with(Vec::new).push(name);
    }

    // ---- the stack ----

    fn top(&self) -> Frame {
        *self
            .control
            .last()
            .expect("the First frame is never popped before the body ends")
    }

    fn push(&mut self, kind: Control, index: Option<usize>, parent: Option<usize>) {
        self.control.push(Frame {
            kind,
            index,
            parent,
        });
    }

    fn pop(&mut self) -> Frame {
        self.control
            .pop()
            .expect("the First frame is never popped before the body ends")
    }

    // ---- the chain ----

    /// `addClause` (`LanguageParser.cpp:2544`): append, and nothing else.
    ///
    /// Also the one place a label enters the label table, so that the table
    /// holds indices into this body's chain. First occurrence wins, matching
    /// `addLabel` (`:2559`), whose own comment says a duplicate label is legal
    /// and only the first can be a target.
    fn add_clause(&mut self, instruction: Instruction) -> usize {
        let index = self.instructions.len();
        if let InstructionKind::Label { name } = &instruction.kind {
            self.labels.entry(name.clone()).or_insert(index);
        }
        // Registered here rather than during the parse, so that an instruction
        // never sees its own references. That is what a `GUARD` needs: the
        // C++'s cache is consulted as each reference is built, and within one
        // clause the only reference that could precede the guard expression is
        // in the guard expression itself, where the first occurrence captures
        // and a later duplicate is a no-op on a set. Measured:
        // `guard on when a.1 & a.1` is rc 0.
        let symbols = self.symbols;
        for_each_variable_name(&instruction, symbols, &mut |name| {
            self.referenced.insert(Box::from(name.as_bytes()));
        });
        self.instructions.push(instruction);
        index
    }

    /// Whether the instruction just added was a label, which is what forbids an
    /// `ELSE` from following it (`LanguageParser.cpp:1430`).
    fn last_is_label(&self) -> bool {
        matches!(
            self.instructions.last().map(|i| &i.kind),
            Some(InstructionKind::Label { .. })
        )
    }

    /// Where control resumes after everything added so far, which is the C++'s
    /// `->nextInstruction` of whatever currently ends the chain.
    ///
    /// The value can be one past the end while assembly is still running, and
    /// `resolve_targets` turns that into `None`.
    fn next_index(&self) -> usize {
        self.instructions.len()
    }

    fn set_false_target(&mut self, index: usize, target: usize) {
        match &mut self.instructions[index].kind {
            InstructionKind::If { false_target, .. }
            | InstructionKind::When { false_target, .. }
            | InstructionKind::WhenCase { false_target, .. } => *false_target = Some(target),
            other => panic!("a THEN frame's parent is an IF or a WHEN, not {other:?}"),
        }
    }

    fn set_then_exit(&mut self, index: usize, target: usize) {
        match &mut self.instructions[index].kind {
            InstructionKind::Else { then_exit } => *then_exit = Some(target),
            other => panic!("an ELSE frame's instruction is an ELSE, not {other:?}"),
        }
    }

    /// `flushControl` (`LanguageParser.cpp:1919`): close out whatever branch the
    /// arrival of `instruction` completes, adding it in the right place.
    ///
    /// Returns where `instruction` landed. The synthetic branch-end markers the
    /// C++ adds here have no node, so where each would sit is recorded as the
    /// jump target of the instruction that jumps: `next_index()` at the moment
    /// the marker would be appended is exactly that marker's own
    /// `->nextInstruction`.
    fn flush_control(&mut self, instruction: Option<Instruction>) -> Option<usize> {
        let mut instruction = instruction;
        let mut added = None;
        loop {
            match self.top().kind {
                // A pending ELSE. Its branch is now complete, so the THEN branch
                // of the same IF learns where to resume.
                Control::Else => {
                    let frame = self.pop();
                    if let Some(inst) = instruction.take() {
                        added = Some(self.add_clause(inst));
                    }
                    let target = self.next_index();
                    self.set_then_exit(frame.index(), target);
                    // The C++ goes around again rather than breaking, so a stack
                    // of pending ELSEs unwinds in one call.
                }
                // A pending THEN. Its branch is now complete, so the IF or WHEN
                // learns where to go when its condition is false.
                Control::IfThen | Control::WhenThen => {
                    let frame = self.pop();
                    if let Some(inst) = instruction.take() {
                        added = Some(self.add_clause(inst));
                    }
                    let parent = frame
                        .parent
                        .expect("a THEN frame carries the IF or WHEN it belongs to");
                    let target = self.next_index();
                    self.set_false_target(parent, target);
                    let end = match frame.kind {
                        Control::IfThen => Control::EndThen,
                        _ => Control::EndWhen,
                    };
                    self.push(end, None, None);
                    break;
                }
                // Anything else: the instruction just joins the stream.
                _ => {
                    if let Some(inst) = instruction.take() {
                        added = Some(self.add_clause(inst));
                    }
                    break;
                }
            }
        }
        added
    }

    // ---- errors ----

    /// The byte an error about the state of the block is reported against.
    ///
    /// `blockError` sets `clauseLocation` from `lastInstruction`
    /// (`LanguageParser.cpp:4182`), so this is the last instruction ADDED and
    /// not the last clause read. Measured with blank lines to separate the two:
    /// `do` / `nop` / `nop` / `nop` with no `END` reports the third `nop`'s line
    /// and carries the `DO`'s line only as a substitution.
    fn last_byte(&self) -> usize {
        self.instructions
            .last()
            .expect("an unclosed block was itself added, so the chain is not empty")
            .clause_span
            .start
    }

    /// `blockError` (`LanguageParser.cpp:4180`): an unclosed block at the end of
    /// the body, with one number per block kind.
    ///
    /// The C++ also has arms for a bare `IF`, `WHEN` and `WHEN_CASE`, which
    /// cannot be reached because none of the three is ever pushed.
    fn block_error(&self, kind: Control) -> ParseError {
        let sub = match kind {
            // `Error_Incomplete_do_do`. Measured: `do label a` / `nop` is 14.1
            // too, so a LABEL does not move it to the loop number.
            Control::Do => 1,
            // `Error_Incomplete_do_select`, for both SELECT spellings.
            Control::Select | Control::SelectCase => 2,
            // `Error_Incomplete_do_then`. Reached here rather than at the C++'s
            // own check, which fires from the failed `nextClause()` right after
            // the THEN (`LanguageParser.cpp:1370`). The reported line comes out
            // the same either way, because that check reports against the clause
            // holding the THEN and this reports against the THEN instruction,
            // whose span is that keyword: measured, `if 1 = 1` / blank / `then`
            // at end of file reports line 5 and substitutes line 3.
            //
            // That equality is not free. It holds because Task 3.4 splits a
            // `THEN` off into a clause of its own, so the THEN instruction's
            // span starts where the clause the C++ reports against starts. A
            // change that stopped splitting there would break it silently, which
            // is what `a_then_or_else_with_nothing_after_it_is_14_3_or_14_4`
            // pins with the blank-line spelling.
            Control::IfThen | Control::WhenThen => 3,
            // `Error_Incomplete_do_else`.
            Control::Else => 4,
            // `Error_Incomplete_do_loop`. Measured for `do while 1`, `do 3`,
            // `loop`, `loop forever` and `do i over x`.
            Control::Loop => 5,
            // `Error_Incomplete_do_otherwise`, whose sub-number is 901 and not
            // 6. Measured, not derived from the position in the table.
            Control::Otherwise => 901,
            Control::First | Control::EndThen | Control::EndWhen => {
                unreachable!("neither First nor a branch end is an unclosed block")
            }
        };
        ParseError::new(14, sub, self.last_byte())
    }

    /// The misplaced-label check (`LanguageParser.cpp:1224`-`1244`).
    ///
    /// Three numbers from one condition, and which one depends on what is open.
    /// `EndThen` is deliberately absent from every arm: a label there may be
    /// sitting in front of an `ELSE`, which the `ELSE` itself checks, so it is
    /// allowed here. Measured both ways: `if 1 = 1 then nop` / `lab:` / `nop` is
    /// rc 0, and the same with `else nop` last is 47.3.
    fn label_error(&self, byte: usize) -> Option<ParseError> {
        let sub = match self.top().kind {
            Control::IfThen | Control::Else => 3,
            Control::Select
            | Control::SelectCase
            | Control::WhenThen
            | Control::EndWhen
            | Control::Otherwise => 4,
            Control::Do | Control::Loop => 2,
            Control::First | Control::EndThen => return None,
        };
        Some(ParseError::new(47, sub, byte))
    }

    // ---- matching an END ----

    /// The `END`'s own optional block name.
    fn end_name(&self, end: usize) -> Option<SymbolId> {
        match &self.instructions[end].kind {
            InstructionKind::End { name, .. } => *name,
            other => panic!("end_name on {other:?}"),
        }
    }

    /// `matchLabel` (`BaseDoInstruction.cpp:172`) and the name half of
    /// `RexxInstructionSelect::matchEnd` (`SelectInstruction.cpp:181`).
    ///
    /// Four errors, and which one fires depends on both what the `END` failed to
    /// close and whether that block had a name at all. All four measured:
    /// `do label a` / `end b` is 10.2, `do` / `end 1` is 10.3,
    /// `select label a` / `end b` is 10.4, and `select` / `end 1` is 10.7.
    /// Reported against the `END`, which the C++ passes as `endLocation` rather
    /// than letting `clauseLocation` stand.
    fn match_label(
        block_label: Option<SymbolId>,
        end_name: Option<SymbolId>,
        is_select: bool,
        end_byte: usize,
    ) -> Result<(), ParseError> {
        // No name on the END always matches, whether the block has a label or
        // not. Measured: `do label a` / `end` is rc 0.
        let Some(end_name) = end_name else {
            return Ok(());
        };
        let sub = match (block_label, is_select) {
            (Some(label), _) if label == end_name => return Ok(()),
            // `Error_Unexpected_end_control` / `Error_Unexpected_end_select`.
            (Some(_), false) => 2,
            (Some(_), true) => 4,
            // `Error_Unexpected_end_nocontrol` /
            // `Error_Unexpected_end_select_nolabel`.
            (None, false) => 3,
            (None, true) => 7,
        };
        Err(ParseError::new(10, sub, end_byte))
    }

    /// `RexxInstructionSelect::matchEnd`: the name check, the `WHEN`-count
    /// check, the `WHEN` exits, and the `END` style.
    fn match_select_end(
        &mut self,
        select: usize,
        end: usize,
        end_byte: usize,
    ) -> Result<EndStyle, ParseError> {
        let (label, whens, has_otherwise) = match &mut self.instructions[select].kind {
            InstructionKind::Select {
                label,
                whens,
                otherwise,
                end: slot,
                ..
            } => {
                *slot = Some(end);
                // Cloned rather than taken. The C++ drops its `whenList` here,
                // because it has just moved the information into each WHEN, but
                // the list is not recoverable from the result: which WHENs
                // between a SELECT and its END belong to it is exactly what the
                // list records, and a nested one may not.
                (*label, whens.clone(), otherwise.is_some())
            }
            other => panic!("match_select_end on {other:?}"),
        };
        Self::match_label(label, self.end_name(end), true, end_byte)?;

        // `Error_When_expected_when`, reported against the SELECT's OWN location
        // and not the END's, which is the one place in this family where that is
        // true. Measured: `nop` / blank / `select` / blank / `end` reports line
        // 3, the SELECT, with the END on line 5.
        if whens.is_empty() {
            let byte = self.instructions[select].clause_span.start;
            return Err(ParseError::new(7, 1, byte));
        }

        // `fixWhen` (`SelectInstruction.cpp:222`): one true WHEN ends the whole
        // SELECT, so every WHEN resumes after the END.
        for when in whens {
            match &mut self.instructions[when].kind {
                InstructionKind::When { exit, .. } | InstructionKind::WhenCase { exit, .. } => {
                    *exit = Some(end + 1);
                }
                other => panic!("a SELECT's when list holds WHENs, not {other:?}"),
            }
        }

        Ok(match (has_otherwise, label.is_some()) {
            (false, _) => EndStyle::Select,
            (true, false) => EndStyle::Otherwise,
            (true, true) => EndStyle::LabeledOtherwise,
        })
    }

    /// `matchEnd` for a `DO` or `LOOP` (`BaseDoInstruction.cpp:139`).
    fn match_do_end(
        &mut self,
        block: usize,
        end: usize,
        end_byte: usize,
    ) -> Result<EndStyle, ParseError> {
        let (label, simple) = match &self.instructions[block].kind {
            InstructionKind::Do(body) | InstructionKind::Loop(body) => {
                (body.label, body.kind == LoopKind::Simple)
            }
            other => panic!("match_do_end on {other:?}"),
        };
        Self::match_label(label, self.end_name(end), false, end_byte)?;
        match &mut self.instructions[block].kind {
            InstructionKind::Do(body) | InstructionKind::Loop(body) => body.end = Some(end),
            other => panic!("match_do_end on {other:?}"),
        }
        // `getEndStyle`: the block form distinguishes a LABEL where every loop
        // form answers `LOOP_BLOCK` regardless (`DoInstruction.hpp:82`, `:105`).
        Ok(match (simple, label) {
            (true, None) => EndStyle::Do,
            (true, Some(_)) => EndStyle::LabeledDo,
            (false, _) => EndStyle::Loop,
        })
    }

    /// The `END` arm of `translateBlock`'s switch (`LanguageParser.cpp:1500`).
    fn match_end(&mut self, end: usize, end_byte: usize) -> Result<(), ParseError> {
        let frame = self.pop();
        if !frame.kind.is_block() {
            // The C++ has two more specific numbers here, `Error_Unexpected_end_then`
            // for an END closing a THEN and `Error_Unexpected_end_else` for one
            // closing an ELSE, and neither is reachable. The argument is
            // structural rather than empirical: an END has `isControl() == false`
            // and its type is not `KEYWORD_ELSE`, so `flushControl` ALWAYS runs
            // before this switch, and `flushControl` cannot return with `ELSE`,
            // `IFTHEN` or `WHENTHEN` on top -- it pops an ELSE outright and
            // rewrites a THEN into a branch-end marker. The type this arm tests
            // for therefore cannot be present.
            //
            // 24 probes agree, across both shapes on one line and on separate
            // lines, nested in a DO, with a named END, and inside a method. Every
            // one answers 10.1. `Error_Unexpected_end_nodo`.
            return Err(ParseError::new(10, 1, end_byte));
        }
        // An END on an OTHERWISE really closes the SELECT behind it.
        let frame = match frame.kind {
            Control::Otherwise => self.pop(),
            _ => frame,
        };
        let block = frame.index();
        let style = if frame.kind.is_select() {
            self.match_select_end(block, end, end_byte)?
        } else {
            self.match_do_end(block, end, end_byte)?
        };
        match &mut self.instructions[end].kind {
            InstructionKind::End { closes, .. } => *closes = Some(EndTarget { block, style }),
            other => panic!("match_end on {other:?}"),
        }
        Ok(())
    }

    // ---- finishing ----

    /// Turns every jump target that points one past the end of the chain into
    /// `None`, which is what "control falls out of this body" means.
    ///
    /// A target is recorded from `next_index()` while assembly is still running,
    /// so a branch that turns out to be the last thing in the body records an
    /// index that never gets an instruction.
    fn resolve_targets(&mut self) {
        let len = self.instructions.len();
        for instruction in &mut self.instructions {
            let (first, second) = match &mut instruction.kind {
                InstructionKind::If { false_target, .. } => (Some(false_target), None),
                InstructionKind::Else { then_exit } => (Some(then_exit), None),
                InstructionKind::When {
                    false_target, exit, ..
                }
                | InstructionKind::WhenCase {
                    false_target, exit, ..
                } => (Some(false_target), Some(exit)),
                _ => (None, None),
            };
            for target in [first, second].into_iter().flatten() {
                if target.is_some_and(|t| t >= len) {
                    *target = None;
                }
            }
        }
    }

    fn finish(mut self) -> CodeBody {
        self.resolve_targets();
        // `assert!` rather than `debug_assert!`: this is the only thing standing
        // between a release consumer and a `None` that the field's contract says
        // cannot occur, and `resolve_targets` has just walked the same list, so
        // the cost is not worth trading the guarantee for.
        assert!(
            self.instructions.iter().all(|i| match &i.kind {
                InstructionKind::Do(body) | InstructionKind::Loop(body) => body.end.is_some(),
                InstructionKind::Select { end, .. } => end.is_some(),
                InstructionKind::End { closes, .. } => closes.is_some(),
                _ => true,
            }),
            "every block and every END is matched in a body that assembled"
        );
        CodeBody {
            instructions: self.instructions,
            labels: self.labels,
        }
    }
}

/// Calls `f` with the name of every variable REFERENCE in `instruction`.
///
/// This is which slots reach `addVariable` in the C++, and it is not simply
/// "every symbol": a block name, a loop or `SELECT` label, a routine name, an
/// `ADDRESS` environment and a condition trap's label are all symbols that name
/// something other than a variable, and none of them touches the cache.
/// Measured, both directions, twenty shapes: `do label a.1` / `end a.1`,
/// `leave a.1`, `iterate a.1`, `select label a.1`, `signal a.1`, `call a.1`,
/// `address a.1` and `signal on syntax name a.1` all leave a later
/// `guard on when a.1` legal, while `drop a.1`, `expose a.1`,
/// `procedure expose a.1`, `parse var a.1 x`, `parse value 1 with a.1`,
/// `use arg a.1`, `do a.1 = 1 to 2`, `numeric digits a.1`, `interpret a.1`,
/// a bare `a.1` command and `a.1~string` all make it 99.913.
///
/// The order names arrive in does not matter, because the caller keeps a set and
/// consults it only for instructions already in the chain.
///
/// Every name is reported, not only compound ones. A simple variable and a stem
/// capture unconditionally whatever the cache holds, so their entries are never
/// read, and a compound spelling can only ever match another compound spelling.
/// Classifying a bare `SymbolId` here would mean re-deriving the scanner's rule
/// for `SymbolClass::Compound` in a second place.
///
/// The `match` is exhaustive on purpose: a new `InstructionKind` fails to
/// compile here rather than silently contributing nothing.
fn for_each_variable_name(
    instruction: &Instruction,
    symbols: &SymbolTable,
    f: &mut impl FnMut(&str),
) {
    match &instruction.kind {
        InstructionKind::Assignment { target, value } => {
            visit_expr(target, symbols, f);
            visit_expr(value, symbols, f);
        }
        InstructionKind::Message { term, value } => {
            visit_expr(term, symbols, f);
            visit_opt(value, symbols, f);
        }
        InstructionKind::Command { expression }
        | InstructionKind::Push { expression }
        | InstructionKind::Queue { expression }
        | InstructionKind::Say { expression }
        | InstructionKind::Return { expression }
        | InstructionKind::Exit { expression }
        | InstructionKind::Reply { expression } => visit_opt(expression, symbols, f),
        InstructionKind::Interpret { expression } | InstructionKind::Options { expression } => {
            visit_expr(expression, symbols, f);
        }
        InstructionKind::If { condition, .. } | InstructionKind::When { condition, .. } => {
            visit_expr(condition, symbols, f);
        }
        InstructionKind::WhenCase { values, .. } => {
            for value in values {
                visit_expr(value, symbols, f);
            }
        }
        InstructionKind::Do(body) | InstructionKind::Loop(body) => {
            // `label` is deliberately absent: a loop label names the block, not
            // a variable. `counter` and every control variable are variables.
            visit_slot(body.counter, symbols, f);
            match &body.kind {
                LoopKind::Simple | LoopKind::Forever => {}
                LoopKind::Count(count) => visit_opt(count, symbols, f),
                LoopKind::Controlled(controlled) => {
                    visit_slot(Some(controlled.control), symbols, f);
                    visit_expr(&controlled.initial, symbols, f);
                    visit_opt(&controlled.to, symbols, f);
                    visit_opt(&controlled.by, symbols, f);
                    visit_opt(&controlled.for_count, symbols, f);
                }
                LoopKind::Over {
                    control,
                    target,
                    for_count,
                } => {
                    visit_slot(Some(*control), symbols, f);
                    visit_expr(target, symbols, f);
                    visit_opt(for_count, symbols, f);
                }
                LoopKind::With {
                    index,
                    item,
                    target,
                    for_count,
                } => {
                    visit_slot(*index, symbols, f);
                    visit_slot(*item, symbols, f);
                    visit_expr(target, symbols, f);
                    visit_opt(for_count, symbols, f);
                }
            }
            if let Some(conditional) = &body.conditional {
                visit_expr(&conditional.condition, symbols, f);
            }
        }
        InstructionKind::Drop { variables }
        | InstructionKind::Expose { variables }
        | InstructionKind::Procedure { variables } => visit_refs(variables, symbols, f),
        InstructionKind::Parse(body) | InstructionKind::Arg(body) | InstructionKind::Pull(body) => {
            match &body.source {
                ParseSource::Var(id) => visit_slot(Some(*id), symbols, f),
                ParseSource::Value(value) => visit_opt(value, symbols, f),
                ParseSource::Arg
                | ParseSource::LineIn
                | ParseSource::Pull
                | ParseSource::Source
                | ParseSource::Version => {}
            }
            for trigger in body.template.iter().flatten() {
                visit_opt(&trigger.value, symbols, f);
                visit_list(&trigger.targets, symbols, f);
            }
        }
        InstructionKind::Call(call) => match call.as_ref() {
            // A routine name is not a variable, whichever spelling it took.
            Call::Named { args, .. } | Call::Qualified { args, .. } => visit_list(args, symbols, f),
            Call::Dynamic { target, args } => {
                visit_expr(target, symbols, f);
                visit_list(args, symbols, f);
            }
            Call::Trap(_) => {}
        },
        InstructionKind::Signal(signal) => match signal.as_ref() {
            // A label, and a trap's label, name instructions rather than
            // variables. `SIGNAL VALUE` evaluates an expression.
            Signal::Label(_) | Signal::Trap(_) => {}
            Signal::Value(value) => visit_expr(value, symbols, f),
        },
        InstructionKind::Guard(guard) => visit_opt(&guard.condition, symbols, f),
        InstructionKind::Forward(forward) => {
            visit_opt(&forward.to, symbols, f);
            visit_opt(&forward.message, symbols, f);
            visit_opt(&forward.class, symbols, f);
            visit_opt(&forward.arguments, symbols, f);
            if let Some(array) = &forward.array {
                visit_list(array, symbols, f);
            }
        }
        InstructionKind::Raise(raise) => {
            visit_opt(&raise.rc, symbols, f);
            visit_opt(&raise.description, symbols, f);
            visit_opt(&raise.additional, symbols, f);
            if let Some(array) = &raise.array {
                visit_list(array, symbols, f);
            }
            if let Some(result) = &raise.result {
                visit_opt(&result.value, symbols, f);
            }
        }
        InstructionKind::Use(use_) => match use_.as_ref() {
            Use::Arg { targets, .. } => {
                for target in targets.iter().flatten() {
                    visit_expr(&target.target, symbols, f);
                    visit_opt(&target.default, symbols, f);
                }
            }
            Use::Local { variables } => visit_refs(variables, symbols, f),
        },
        InstructionKind::Numeric { expression, .. } => visit_opt(expression, symbols, f),
        InstructionKind::Address(address) => {
            // `environment` is a name rather than a variable.
            visit_opt(&address.dynamic, symbols, f);
            visit_opt(&address.command, symbols, f);
            if let Some(io) = &address.io {
                for redirection in [&io.input, &io.output, &io.error] {
                    match redirection {
                        Redirection::Stem(id) => visit_slot(Some(*id), symbols, f),
                        Redirection::Stream(value) | Redirection::Using(value) => {
                            visit_expr(value, symbols, f)
                        }
                        Redirection::Default | Redirection::Normal => {}
                    }
                }
            }
        }
        InstructionKind::Trace(trace) => match trace {
            Trace::Value(value) => visit_expr(value, symbols, f),
            Trace::Default | Trace::Setting(_) | Trace::Skip(_) => {}
        },
        // No variable reference of any kind. A block name on an `END`, `LEAVE`,
        // `ITERATE` or `SELECT` names a block, and a label names itself.
        InstructionKind::Label { .. }
        | InstructionKind::Then
        | InstructionKind::Else { .. }
        | InstructionKind::Otherwise
        | InstructionKind::Leave { .. }
        | InstructionKind::Iterate { .. }
        | InstructionKind::End { .. }
        | InstructionKind::Nop => {}
        InstructionKind::Select { case, .. } => visit_opt(case, symbols, f),
    }
}

/// `for_each_variable_in_expr` for an optional expression.
fn visit_opt(expr: &Option<Expr>, symbols: &SymbolTable, f: &mut impl FnMut(&str)) {
    if let Some(expr) = expr {
        visit_expr(expr, symbols, f);
    }
}

/// `for_each_variable_in_expr` over an argument list, whose omitted positions
/// hold no node.
fn visit_list(args: &[Option<Expr>], symbols: &SymbolTable, f: &mut impl FnMut(&str)) {
    for arg in args.iter().flatten() {
        visit_expr(arg, symbols, f);
    }
}

/// A bare variable slot, which is a name and not an expression.
fn visit_slot(id: Option<SymbolId>, symbols: &SymbolTable, f: &mut impl FnMut(&str)) {
    if let Some(id) = id {
        f(symbols.name(id));
    }
}

/// A `DROP`, `EXPOSE`, `PROCEDURE EXPOSE` or `USE LOCAL` list. Both spellings
/// reach `addVariable`, the indirect one through the symbol inside the
/// parentheses.
fn visit_refs(variables: &[VariableRef], symbols: &SymbolTable, f: &mut impl FnMut(&str)) {
    for variable in variables {
        let (VariableRef::Direct(id) | VariableRef::Indirect(id)) = variable;
        f(symbols.name(*id));
    }
}

/// Calls `f` with the name of every variable reference in one expression.
///
/// A constant, a `.name` environment symbol and a literal are not variables and
/// reach neither `addSimpleVariable` nor `addStem`, so none is reported.
fn visit_expr(expr: &Expr, symbols: &SymbolTable, f: &mut impl FnMut(&str)) {
    match &expr.kind {
        ExprKind::Variable(id) | ExprKind::Stem(id) | ExprKind::Compound(id) => {
            f(symbols.name(*id));
        }
        _ => {}
    }
    expr.kind
        .for_each_child(&mut |child| visit_expr(child, symbols, f));
}

/// Which stack frame a block instruction opens.
///
/// An `OTHERWISE` is a block too, but it is pushed by its own arm of the switch
/// rather than from here, so it is not listed.
fn opens(kind: &InstructionKind) -> Option<Control> {
    match kind {
        InstructionKind::Do(body) | InstructionKind::Loop(body) => Some(match body.kind {
            LoopKind::Simple => Control::Do,
            _ => Control::Loop,
        }),
        InstructionKind::Select { case: None, .. } => Some(Control::Select),
        InstructionKind::Select { case: Some(_), .. } => Some(Control::SelectCase),
        _ => None,
    }
}

/// `isControl()`: whether the instruction joins the chain immediately instead of
/// going through `flushControl`.
///
/// True for every `DO`/`LOOP` and `SELECT`, which derive from
/// `RexxBlockInstruction` (`RexxInstruction.hpp:139`), and additionally for a
/// bare `IF` (`IfInstruction.hpp:64`). NOT for a `WHEN`, whose own comment there
/// says a `WHEN` is part of its `SELECT` rather than a control type of its own,
/// and NOT for an `OTHERWISE`, which overrides only `isBlock`
/// (`OtherwiseInstruction.hpp:55`) and derives from `RexxInstruction` rather than
/// from `RexxBlockInstruction`, so it inherits `isControl() == false`.
///
/// An `OTHERWISE` therefore goes through `flushControl`, and that is invisible:
/// the branch-end frames have already been popped by the time it is reached, so
/// the `SELECT` is on top and `flushControl` only appends. It is spelled the
/// C++'s way anyway, because a comment claiming otherwise would be wrong.
fn is_control(kind: &InstructionKind) -> bool {
    opens(kind).is_some() || matches!(kind, InstructionKind::If { .. })
}

/// Assembles one code body, from the clause the cursor is sitting on up to the
/// first `::` directive clause or the end of the source.
///
/// This is `translateBlock` (`LanguageParser.cpp:1176`) with its own clause
/// loop: `parse_instruction` is called from inside it, so that the constructors
/// which read block state see the state as it stood when their clause was
/// reached.
pub(crate) fn translate_block(
    ctx: &ParseCtx,
    cursor: &mut ClauseCursor,
) -> Result<CodeBody, ParseError> {
    let mut block = Block::new(ctx.symbols);

    loop {
        // Consume label clauses, which are not instructions and take no part in
        // the control stack beyond being checked against it.
        let mut pending = None;
        while let Some(clause) = cursor.peek() {
            // A directive terminates the body, and the cursor is left sitting on
            // it for the caller.
            if ctx.tokens[clause.tokens.start].kind.tag() == Tag::DColon {
                break;
            }
            let instruction = parse_instruction(ctx, cursor, &mut block)?;
            if !matches!(instruction.kind, InstructionKind::Label { .. }) {
                pending = Some(instruction);
                break;
            }
            if let Some(error) = block.label_error(instruction.clause_span.start) {
                return Err(error);
            }
            block.add_clause(instruction);
        }

        let Some(instruction) = pending else {
            // End of the body with an IF or WHEN whose THEN never arrived.
            //
            // A directive can end a body as well as end of file can, and the two
            // report differently. `nextClause()` succeeds on the `::` clause, so
            // `clauseLocation` moves to it and that is the reported line, with
            // the IF's line only substituted. At end of file nothing moves and
            // the two coincide. Measured both: `nop` / blank / `if 1 = 1` /
            // blank / `::routine r` reports line 5 and substitutes line 3, while
            // `nop` / blank / `nop` / blank / `if 1 = 1` reports line 5 for both.
            if let Some((which, byte)) = cursor.take_expected_then() {
                let byte = cursor.peek().map_or(byte, |clause| clause.span.start);
                return Err(ParseError::new(18, missing_then_sub(which), byte));
            }
            // Close out any finished branch, then everything opened must be
            // closed.
            while matches!(block.top().kind, Control::EndThen | Control::EndWhen) {
                block.pop();
                block.flush_control(None);
            }
            let top = block.top().kind;
            if top != Control::First {
                return Err(block.block_error(top));
            }
            block.pop();
            return Ok(block.finish());
        };

        // A THEN is not dispatched at all in the C++: `translateBlock` builds it
        // itself inside the IF arm and goes straight back to the top of the
        // loop. Reproducing that placement matters, because a THEN reaching the
        // SELECT-membership check below would be rejected as a non-WHEN inside a
        // SELECT. Measured rc 0: `select` / `when 1 = 1` / `then nop` / `end`.
        if matches!(instruction.kind, InstructionKind::Then) {
            let (parent, is_when) = block
                .pending_then
                .take()
                .expect("a THEN instruction only exists where one was expected");
            let index = block.add_clause(instruction);
            let kind = if is_when {
                Control::WhenThen
            } else {
                Control::IfThen
            };
            block.push(kind, Some(index), Some(parent));
            continue;
        }

        let byte = instruction.clause_span.start;
        let opened = opens(&instruction.kind);
        let is_else = matches!(instruction.kind, InstructionKind::Else { .. });
        let is_when = matches!(
            instruction.kind,
            InstructionKind::When { .. } | InstructionKind::WhenCase { .. }
        );
        let is_end = matches!(instruction.kind, InstructionKind::End { .. });
        let is_otherwise = matches!(instruction.kind, InstructionKind::Otherwise);
        let is_if = matches!(instruction.kind, InstructionKind::If { .. });
        let control = is_control(&instruction.kind);
        // Held rather than moved, because an ELSE is added further down: it takes
        // no part in step 1 below, so the checks between here and there see the
        // stack as it stood when the ELSE arrived.
        let mut held = Some(instruction);

        // Anything but an ELSE may have finished branches to close first. More
        // than one can be pending, which is why this loops.
        if !is_else {
            while matches!(block.top().kind, Control::EndThen | Control::EndWhen) {
                block.pop();
                block.flush_control(None);
            }
        }

        // A control instruction joins the chain at once. Anything else but an
        // ELSE goes through `flushControl`, which may close a branch first, and
        // an ELSE is added by its own arm below.
        let index = if is_else {
            None
        } else if control {
            Some(block.add_clause(held.take().expect("held until consumed")))
        } else {
            block.flush_control(held.take())
        };

        // Only WHEN, OTHERWISE and END may appear directly inside a SELECT. This
        // runs for an ELSE too, and gets there first: measured, `select` /
        // `else nop` / `end` is 7.2, while `select` / `when 1 = 1 then nop` /
        // `else nop` / `end` is 8.2, because the WHEN's branch end is on top by
        // then. `Error_When_expected_whenotherwise`, reported against the
        // offending clause with the SELECT's line as a substitution.
        if block.top().kind.is_select() && !(is_when || is_otherwise || is_end) {
            return Err(ParseError::new(7, 2, byte));
        }

        if is_if || is_when {
            let index = index.expect("an IF or a WHEN was just added");
            block.pending_then = Some((index, is_when));
        }

        if is_when {
            // The WHEN joins its SELECT's list only when the SELECT is the
            // immediate top, which is narrower than "a SELECT is open": see
            // `InstructionKind::Select::whens`.
            let frame = block.top();
            if frame.kind.is_select() {
                let when = index.expect("the WHEN was just added");
                match &mut block.instructions[frame.index()].kind {
                    InstructionKind::Select { whens, .. } => whens.push(when),
                    other => panic!("a SELECT frame stands for a SELECT, not {other:?}"),
                }
            }
            continue;
        }

        if is_else {
            // `Error_Unexpected_then_else`: only a finished IF branch can take
            // an ELSE. A finished WHEN branch cannot, which is measured rather
            // than assumed: `select` / `when 1 = 1 then nop` / `else nop` is
            // 8.2.
            if block.top().kind != Control::EndThen {
                return Err(ParseError::new(8, 2, byte));
            }
            // No label may sit immediately in front of an ELSE. The C++ has no
            // label name to hand here and substitutes the string "ELSE", which
            // is why the measured message reads `found "ELSE"`.
            if block.last_is_label() {
                return Err(ParseError::new(47, 3, byte));
            }
            let index = block.add_clause(held.take().expect("an ELSE is added here"));
            block.pop();
            block.push(Control::Else, Some(index), None);
            continue;
        }

        if is_otherwise {
            // `Error_Unexpected_when_otherwise`.
            let frame = block.top();
            if !frame.kind.is_select() {
                return Err(ParseError::new(9, 2, byte));
            }
            let otherwise = index.expect("the OTHERWISE was just added");
            match &mut block.instructions[frame.index()].kind {
                InstructionKind::Select {
                    otherwise: slot, ..
                } => *slot = Some(otherwise),
                other => panic!("a SELECT frame stands for a SELECT, not {other:?}"),
            }
            block.push(Control::Otherwise, Some(otherwise), None);
            continue;
        }

        if is_end {
            let end = index.expect("the END was just added");
            block.match_end(end, byte)?;
            // The block that just closed may itself have been an IF's or a
            // WHEN's instruction, so any pending branch is flushed now.
            block.flush_control(None);
            continue;
        }

        if let Some(kind) = opened {
            let index = index.expect("a block instruction was just added");
            block.push(kind, Some(index), None);
        }
    }
}

#[cfg(test)]
mod tests;
