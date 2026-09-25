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

//! `compile`: the one pass that turns a body into a [`Chunk`] (the plan's
//! Decisions section: "compilation is whole-body and lazy: one body at a
//! time, on first entry, cached").

use std::collections::HashMap;

use rexx_parse::{
    Call, CodeBody, Expr, ExprKind, Instruction, InstructionKind, LoopKind, ParseSource, SymbolId,
};

use super::{
    Calls, Chunk, ChunkTooLarge, ClausePosition, ConditionKeyword, Hints, NodePath, Op, PlanSlot,
};
use crate::eval::{SymbolRead, is_arithmetic, is_native_binary};
use crate::plan::Plan;
use crate::run::{
    HeaderPlan, QueueKeyword, ReturnKeyword, if_targets, loop_header_plan, loop_header_slot,
    otherwise_range, when_targets,
};
use crate::trace::ChunkTrace;

mod invariants;
use invariants::{
    assert_call_echoes_follow_their_op, assert_exec_regions_hold_nothing_else,
    assert_keyword_echoes_precede_their_value, assert_literal_echoes_follow_their_load,
    assert_operator_echoes_follow_their_op, assert_prefix_echoes_follow_their_op,
    assert_read_echoes_follow_their_load, assert_region_ops_name_their_clause,
    assert_trace_ops_open_a_clause_region,
};

/// The compile-time register stack (the plan's Decisions section: "register
/// allocation is a compile-time stack, and the chunk records its high-water
/// mark").
struct Registers {
    /// The next index `alloc` hands out.
    next: u16,
    /// The largest `next` ever reached, which is what [`Chunk::registers`]
    /// records -- the size of the region the driver has to reserve for a run
    /// of this chunk to address every register it names.
    high_water: u16,
}

/// A saved register top, handed back to [`Registers::release`].
#[derive(Clone, Copy)]
struct Mark(u16);

impl Registers {
    fn new() -> Registers {
        Registers {
            next: 0,
            high_water: 0,
        }
    }

    fn mark(&self) -> Mark {
        Mark(self.next)
    }

    /// The next register, or the refusal a body needing more than `u16::MAX`
    /// live at once earns (the plan's Decisions section: "register indices
    /// are `u16`").
    fn alloc(&mut self) -> Result<u16, ChunkTooLarge> {
        let index = self.next;
        self.next = self
            .next
            .checked_add(1)
            .ok_or(ChunkTooLarge { what: "registers" })?;
        self.high_water = self.high_water.max(self.next);
        Ok(index)
    }

    /// Puts the top back to `mark`, so every register allocated since is
    /// handed out again.
    fn release(&mut self, mark: Mark) {
        self.next = mark.0;
    }

    fn high_water(&self) -> u16 {
        self.high_water
    }
}

/// The constant table [`Op::Const`] indexes, built as the pass goes and
/// **interned**: a literal written twice gets one entry.
struct Constants<'a> {
    /// One entry per distinct literal, in the order they were first seen,
    /// which is what [`Chunk::consts`] becomes.
    values: Vec<Box<[u8]>>,
    /// Which entry a literal's bytes already have.
    index: HashMap<&'a [u8], u32>,
}

impl<'a> Constants<'a> {
    fn new() -> Constants<'a> {
        Constants {
            values: Vec::new(),
            index: HashMap::new(),
        }
    }

    /// The entry `bytes` has, adding one if this is the first occurrence.
    fn intern(&mut self, bytes: &'a [u8]) -> Result<u32, ChunkTooLarge> {
        if let Some(&at) = self.index.get(bytes) {
            return Ok(at);
        }
        let at =
            u32::try_from(self.values.len()).map_err(|_| ChunkTooLarge { what: "constants" })?;
        self.values.push(Box::from(bytes));
        self.index.insert(bytes, at);
        Ok(at)
    }
}

/// A jump whose target is an instruction index the forward pass has not
/// reached yet, and the op that has to be rewritten once it has.
struct Patch {
    /// The op to rewrite.
    op: u32,
    /// The instruction it continues at.
    target: usize,
    /// Which of the two ops an instruction can be entered at this jump wants.
    kind: PatchKind,
}

/// An op that has to be emitted in front of an instruction's own first op,
/// so that arriving at that instruction from elsewhere runs it and falling
/// into the instruction from the op before it does not.
enum Before {
    /// The end of an `IF`'s true branch: its clause boundary
    /// ([`Op::EndBranch`]), and the jump past the `ELSE`
    /// when there is one to skip.
    ThenEnd { resume: Option<usize> },
    /// A `SELECT`'s [`Op::EnterOtherwise`], in front of the `OTHERWISE` marker
    /// whose branch it opens a frame over. The `SELECT` is at this index.
    EnterOtherwise(usize),
    /// A `SELECT` branch runs out in front of this instruction
    /// ([`Op::EndWhen`]): a listed `WHEN`'s own `false_target`, or the end of
    /// the `OTHERWISE` body.
    SelectBranchEnd,
}

/// What a listed `WHEN` needs from the `SELECT` that collected it, recorded
/// when that `SELECT` compiles and read when the `WHEN` itself does.
struct WhenInfo {
    /// The `SELECT` this `WHEN` belongs to.
    select: u32,
    /// The register that `SELECT`'s `CASE` value is in, and `None` for a plain
    /// `SELECT`.
    case: Option<u16>,
    /// Where the scan goes when this `WHEN` does not hold: the next listed
    /// `WHEN`, the `OTHERWISE`, or the `END` whose 7.3 is what "no `WHEN`
    /// matched and there is no `OTHERWISE`" means.
    false_target: usize,
    /// Which of that instruction's two entries the scan wants. It is
    /// [`PatchKind::Resume`] for an `OTHERWISE`, whose `op_of` is the
    /// [`Op::EnterOtherwise`] that opens the frame, and [`PatchKind::Enter`]
    /// for the other two, which have nothing in front of them.
    false_kind: PatchKind,
}

/// Which op a jump to an instruction means, for the one instruction where
/// the two differ: the `ELSE` a branch-end jump sits in front of.
#[derive(Clone, Copy)]
enum PatchKind {
    /// The instruction's own first op: run the instruction.
    Enter,
    /// Where control resumes when it arrives at this instruction from
    /// anywhere else, which is `Chunk::op_of`'s own answer: the branch-end
    /// jump when there is one in front, so that a true branch finishing --
    /// by falling off its end or by a nested construct's `Flow::Goto` landing
    /// exactly on the boundary -- skips the `ELSE` rather than running it.
    Resume,
}

/// Compiles `body` into a [`Chunk`], once, whole.
pub(crate) fn compile(
    body: &CodeBody,
    plan: &Plan,
    trace: ChunkTrace,
) -> Result<Chunk, ChunkTooLarge> {
    #[cfg(test)]
    count_compile_call();

    // **Whether a clause carries a value echo at all**, decided here rather
    // than gated per execution: an op that is not in the stream cannot be
    // gated back on, so this decision is only safe where the setting the
    // clause runs under is settled. `trace_flow` settles it per clause, from
    // the setting the chunk is keyed under; a clause it cannot settle answers
    // [`super::trace_flow::Setting::Unknown`], which keeps the echoes and
    // keeps their run-time gate.
    let settings = super::trace_flow::analyse(body, plan, trace);
    #[cfg(debug_assertions)]
    assert_analysis_only_narrows(plan, trace, &settings);

    let len = body.instructions.len();
    let mut ops: Vec<Op> = Vec::with_capacity(len);
    let mut op_of = Vec::with_capacity(len + 1);
    let mut registers = Registers::new();
    let mut consts = Constants::new();
    let mut hints = Hints::new();
    let mut calls = Calls::new();
    let mut patches: Vec<Patch> = Vec::new();
    // Indexed by instruction: the op that goes in front of that instruction's
    // own, emitted at its `op_of` entry. A `Vec` keyed by the instruction the
    // op sits in front of, rather than a stack, because each entry belongs to
    // exactly one construct -- an `ELSE` to one `THEN`, an `OTHERWISE` to one
    // `SELECT` -- and the `debug_assert`s below are what say so rather than
    // assuming it.
    // A list per instruction, not one entry: nested `IF`s whose branches end
    // at the same instruction each owe a boundary of their own, which the
    // oracle runs as one synthetic instruction per branch. `len + 1` entries,
    // because a branch can end at the body's own end and still owe one.
    let mut before: Vec<Vec<Before>> = Vec::new();
    before.resize_with(len + 1, Vec::new);
    // Indexed by instruction: what a listed `WHEN` compiling at that index
    // needs from the `SELECT` that collected it.
    let mut when_info: Vec<Option<WhenInfo>> = Vec::new();
    when_info.resize_with(len, || None);
    // Indexed by instruction: a register top to put back on reaching it. A
    // `SELECT CASE`'s own value is what needs one -- it is allocated in the
    // enclosing scope so that every member clause's release leaves it alone,
    // and this is where that allocation ends.
    let mut release_at: Vec<Option<Mark>> = vec![None; len];

    // **SPIKE.** Indexed by instruction: for an `END` that closes a repeating
    // `DO`/`LOOP`, that loop's own instruction index. Filled in at the header,
    // which is where the kind and the `END`'s position are both known, and read
    // when the walk reaches the `END` itself.
    let mut loop_of_end: Vec<Option<u32>> = vec![None; len];

    // Indexed by instruction: the instruction's own first op, which is
    // `op_of`'s entry except where a `Before` op sits in front of it.
    // Compile-time only -- the driver never wants it, because `PatchKind`'s
    // own doc comment has why the ops that do are emitted here.
    let mut first_op_of = Vec::with_capacity(len);

    for (index, instruction) in body.instructions.iter().enumerate() {
        op_of.push(op_index(&ops)?);
        if let Some(mark) = release_at[index] {
            registers.release(mark);
        }
        emit_before(&mut ops, &mut patches, &mut before[index])?;
        first_op_of.push(op_index(&ops)?);
        // This clause's own answers, which is where a per-body `bool` used to
        // stand. The `>K>` op asks a different bit than the value echoes:
        // `TraceMode::results`' own doc names `>>>`/`>K>` as the pair it owns,
        // and the oracle prints `>K>` under `R` and `I` and under neither `A`
        // nor `N` (measured, `do i = 1 to 2`).
        let pool = super::trace_flow::for_emission(plan, index, settings[index]);
        let echoes_values = pool.echoes_values();
        let echoes_keyword = pool.echoes_keyword();
        match &instruction.kind {
            // `DO` and `LOOP` are the same construct under two spellings
            // (`step`'s own arm matches them together), so they compile the
            // same way: the header's own clause region, then the op that runs
            // the construct from what that region evaluated.
            InstructionKind::Do(body_node) | InstructionKind::Loop(body_node) => {
                // Allocated in the *enclosing* scope and released past the
                // whole loop, because these registers are what roots the
                // header's values while the loop runs: a `DO OVER`'s target
                // lives in `LoopState` for the construct's lifetime, and a
                // register released at a body clause's own boundary would be
                // handed out again and overwritten while it is still in use.
                let outer = registers.mark();
                // Named apart from `plan`, which this arm reads too: a slot
                // that compiles natively resolves its reads against it,
                // exactly as an assignment's value does.
                let header_plan = loop_header_plan(body_node);
                let roles = header_plan.as_ref().map_or(&[][..], HeaderPlan::roles);
                let mut header = Vec::with_capacity(roles.len());
                for &role in roles {
                    header.push((role, registers.alloc()?));
                }
                let header_top = registers.mark();
                let at = op_index(&ops)?;
                let echo = echoes(trace, instruction);
                // One group per header expression, and `LoopRun` inside the
                // region behind them: the whole loop runs inside the `DO` clause.
                // `close_region` below is
                // what fixes the region's end, from what was actually pushed.
                ops.push(Op::Clause {
                    index: instruction_index(index)?,
                    end: 0,
                });
                push_echo(&mut ops, echo, instruction_index(index)?);
                for (slot, &(role, dst)) in header.iter().enumerate() {
                    let slot = instruction_index(slot)?;
                    // **Each slot decides for itself**, so a header holding one
                    // expression outside the native set keeps native ops for
                    // its others. A slot `loop_header_slot` has no expression
                    // for takes the same answer as one `native_shape` declines,
                    // and they are one arm because they are one answer: the
                    // slot stays a whole `Op::EvalExpr`, which is what it was
                    // before any of it compiled. `run.rs`'s own
                    // `eval_chunk_expr` is where a slot with no expression then
                    // ends up, and it is loud there.
                    match loop_header_slot(body_node, slot) {
                        Some(expr) if native_shape(expr, Some(NodePath::ROOT)) => push_native(
                            &mut ops,
                            echoes_values,
                            &mut consts,
                            &mut registers,
                            &mut hints,
                            &mut calls,
                            plan,
                            expr,
                            instruction_index(index)?,
                            slot,
                            Some(NodePath::ROOT),
                            dst,
                        )?,
                        _ => ops.push(Op::EvalExpr {
                            index: instruction_index(index)?,
                            slot,
                            dst,
                        }),
                    }
                    if echoes_keyword && role.keyword().is_some() {
                        ops.push(Op::TraceKeyword { role, src: dst });
                    }
                    ops.push(Op::LoopHeaderValue { role, src: dst });
                }
                // **The header's own registers, exactly: neither more nor
                // fewer.** An equality rather than a bound, because the two
                // directions are different defects and only one of them is
                // harmful.
                debug_assert_eq!(
                    registers.mark().0,
                    header_top.0,
                    "the register top after the header is not the header's own: above it is a \
                     register nothing reuses until the END, below it is a header value handed \
                     to the body while the loop still reads it"
                );
                ops.push(Op::LoopRun {
                    index: instruction_index(index)?,
                });
                close_region(&mut ops, at)?;
                // **SPIKE.** A repeating loop's `END` carries the op that ends
                // a pass. `Simple` is left alone: it does not repeat, so it has
                // no pass to end, and its `END` stays the delegating
                // `Op::Exec` region that no range ever reaches.
                if let Some(end) = body_node.end
                    && end < len
                    && !matches!(body_node.kind, LoopKind::Simple)
                {
                    loop_of_end[end] = Some(instruction_index(index)?);
                }
                // Past the `END`, so nothing between here and there can reuse a
                // register the running loop still reads. A loop whose `END` is
                // the body's last instruction has nothing to hang the release
                // on and needs none: there is nothing after it to hand the
                // registers to.
                if let Some(end) = body_node.end
                    && end + 1 < len
                {
                    release_to(&mut release_at[end + 1], outer);
                }
            }
            // The `IF` clause is its condition and nothing else -- the branch
            // it chooses is not inside it, which is the boundary
            // `run.rs`'s own `If` arm measured (`SIGL` reports the `IF`'s line,
            // not the branch's). So the region is the condition's ops and the
            // jump that reads them, and the register the condition lands in is
            // released at its end.
            InstructionKind::If {
                condition,
                false_target,
                ..
            } => {
                let targets = if_targets(&body.instructions, *false_target);
                let mark = registers.mark();
                let dst = registers.alloc()?;
                let at = op_index(&ops)?;
                let echo = echoes(trace, instruction);
                ops.push(Op::Clause {
                    index: instruction_index(index)?,
                    end: 0,
                });
                push_echo(&mut ops, echo, instruction_index(index)?);
                let jump = if native_shape(condition, Some(NodePath::ROOT)) {
                    push_native(
                        &mut ops,
                        echoes_values,
                        &mut consts,
                        &mut registers,
                        &mut hints,
                        &mut calls,
                        plan,
                        condition,
                        instruction_index(index)?,
                        0,
                        Some(NodePath::ROOT),
                        dst,
                    )?;
                    let jump = op_index(&ops)?;
                    ops.push(Op::ConditionJump {
                        index: instruction_index(index)?,
                        reg: dst,
                        keyword: ConditionKeyword::If,
                        target: 0,
                    });
                    jump
                } else {
                    ops.push(Op::EvalExpr {
                        index: instruction_index(index)?,
                        slot: 0,
                        dst,
                    });
                    let jump = op_index(&ops)?;
                    ops.push(Op::JumpUnless {
                        reg: dst,
                        target: 0,
                    });
                    jump
                };
                close_region(&mut ops, at)?;
                patches.push(Patch {
                    op: jump,
                    target: targets.false_target,
                    kind: PatchKind::Enter,
                });
                registers.release(mark);
                // Only when the false path lands on an `ELSE`: without one,
                // the true branch's fallthrough is already the resume, and a
                // jump to where control was going anyway is an op the driver
                // would execute for nothing.
                // The boundary is owed either way; the jump only when the
                // false path lands on an `ELSE`, since without one the true
                // branch's fallthrough is already the resume and a jump to
                // where control was going anyway is an op the driver would run
                // for nothing.
                let resume = (targets.resume != targets.false_target).then_some(targets.resume);
                before[targets.false_target].push(Before::ThenEnd { resume });
            }
            // The `SELECT` clause is its `CASE` expression and nothing else,
            // which is the boundary `run.rs`'s own `Select` arm measured
            // (`SIGL` reports the `SELECT`'s line, not the first `WHEN`'s), and
            // a plain `SELECT` has an empty region rather than none, because
            // the clause and its boundary are owed either way.
            InstructionKind::Select {
                case,
                whens,
                otherwise,
                end,
                ..
            } => {
                let select_end = otherwise_range(len, *end);
                // Allocated in the *enclosing* scope, before any member
                // clause's mark is taken, because the last `WHEN`'s test comes
                // after every earlier `WHEN`'s branch has already run: a
                // register released at a clause boundary inside the construct
                // would be handed out again while this one is still live.
                let outer = registers.mark();
                let case_reg = match case {
                    Some(_) => Some(registers.alloc()?),
                    None => None,
                };
                let at = op_index(&ops)?;
                let echo = echoes(trace, instruction);
                ops.push(Op::Clause {
                    index: instruction_index(index)?,
                    end: 0,
                });
                push_echo(&mut ops, echo, instruction_index(index)?);
                if let Some(dst) = case_reg {
                    ops.push(Op::EvalExpr {
                        index: instruction_index(index)?,
                        slot: 0,
                        dst,
                    });
                }
                // Closed here, so that `SelectCaseText` below is past the
                // region's end -- setting the case text is the construct's
                // business rather than the header clause's, and its own doc
                // comment has why.
                close_region(&mut ops, at)?;
                ops.push(Op::SelectCaseText {
                    index: instruction_index(index)?,
                    case: case_reg,
                });
                // Past the whole construct, so nothing between here and the
                // `END` can reuse the register. A `select_end` of `len` has no
                // instruction to hang the release on and needs none: there is
                // nothing after it to hand the register to.
                if select_end < len {
                    release_to(&mut release_at[select_end], outer);
                }
                // Where the scan goes once no listed `WHEN` is left, which is
                // also where it starts when there is no `WHEN` at all.
                let no_match = match otherwise {
                    Some(otherwise_index) => {
                        debug_assert!(
                            !before[*otherwise_index]
                                .iter()
                                .any(|before| matches!(before, Before::EnterOtherwise(_))),
                            "two SELECTs want a frame opened in front of instruction \
                             {otherwise_index}, so an OTHERWISE belongs to more than one SELECT"
                        );
                        before[*otherwise_index].push(Before::EnterOtherwise(index));
                        // The `OTHERWISE` body's own end, which is the
                        // `SELECT`'s: `otherwise_frame` gives that frame this
                        // same range.
                        before[select_end].push(Before::SelectBranchEnd);
                        (*otherwise_index, PatchKind::Resume)
                    }
                    // Landing on the `END` is what makes 7.3 the `END`'s own
                    // clause rather than this `SELECT`'s.
                    None => (select_end, PatchKind::Enter),
                };
                for (position, &when_index) in whens.iter().enumerate() {
                    let (false_target, false_kind) = match whens.get(position + 1) {
                        Some(&next) => (next, PatchKind::Enter),
                        None => no_match,
                    };
                    when_info[when_index] = Some(WhenInfo {
                        select: instruction_index(index)?,
                        case: case_reg,
                        false_target,
                        false_kind,
                    });
                }
                // The first listed `WHEN` is the instruction after this one in
                // every program that parses, so the scan is reached by falling
                // through and a jump to it would be an op the driver runs for
                // nothing. Emitted only when it is not -- a `SELECT` with no
                // listed `WHEN` at all is the case that reaches this.
                if whens.first() != Some(&(index + 1)) {
                    let (target, kind) = match whens.first() {
                        Some(&first) => (first, PatchKind::Enter),
                        None => no_match,
                    };
                    let op = op_index(&ops)?;
                    ops.push(Op::Jump { target: 0 });
                    patches.push(Patch { op, target, kind });
                }
            }
            // A **listed** `WHEN`/`WHEN CASE`: one whose `SELECT` collected it,
            // which is what `when_info` holds an entry for. An *absorbed* one
            // -- itself another `WHEN`'s consequence, never collected -- has
            // none, and falls to the absorbed arm below, where `Op::Exec`
            // runs it through `Interp::exec_instruction`'s own arm for the
            // kind.
            InstructionKind::When { .. } | InstructionKind::WhenCase { .. }
                if when_info[index].is_some() =>
            {
                let info = when_info[index]
                    .take()
                    .expect("the guard above just observed one");
                // **Where this `WHEN`'s branch runs out, taken from the
                // instruction rather than from the `SELECT`'s scan chain.**
                // `Interp::when_frame` reads `when_targets` for the frame's
                // own `op_end`, and the two part company: a `WHEN` absorbed as
                // another's `THEN` is not in the list the scan walks, so the
                // next *listed* `WHEN` is not where this branch ends.
                before[when_targets(&instruction.kind, len).body_end].push(Before::SelectBranchEnd);
                let mark = registers.mark();
                let dst = registers.alloc()?;
                let at = op_index(&ops)?;
                let echo = echoes(trace, instruction);
                ops.push(Op::Clause {
                    index: instruction_index(index)?,
                    end: 0,
                });
                push_echo(&mut ops, echo, instruction_index(index)?);
                // **Only a plain `WHEN`'s condition promotes.** A `WhenCase`
                // holds a list of values compared against the enclosing
                // `SELECT CASE`'s own text through `Interp::test_case_when`,
                // which is not a condition at all: it traces two `>>>` lines
                // per value and raises nothing for a value that is not
                // `0`/`1`. So it stays on `Op::WhenTest`, which is the op that
                // does that whole job, and so does a `When` whose condition
                // `native_shape` declines.
                let jump = match &instruction.kind {
                    InstructionKind::When { condition, .. }
                        if native_shape(condition, Some(NodePath::ROOT)) =>
                    {
                        push_native(
                            &mut ops,
                            echoes_values,
                            &mut consts,
                            &mut registers,
                            &mut hints,
                            &mut calls,
                            plan,
                            condition,
                            instruction_index(index)?,
                            0,
                            Some(NodePath::ROOT),
                            dst,
                        )?;
                        let jump = op_index(&ops)?;
                        ops.push(Op::ConditionJump {
                            index: instruction_index(index)?,
                            reg: dst,
                            keyword: ConditionKeyword::When,
                            target: 0,
                        });
                        jump
                    }
                    _ => {
                        ops.push(Op::WhenTest {
                            index: instruction_index(index)?,
                            case: info.case,
                            dst,
                        });
                        let jump = op_index(&ops)?;
                        ops.push(Op::JumpUnless {
                            reg: dst,
                            target: 0,
                        });
                        jump
                    }
                };
                // Closed before `EnterWhen`, which opens the branch's frame and
                // is the branch's business rather than the clause's.
                close_region(&mut ops, at)?;
                patches.push(Patch {
                    op: jump,
                    target: info.false_target,
                    kind: info.false_kind,
                });
                registers.release(mark);
                ops.push(Op::EnterWhen {
                    select: info.select,
                    when: instruction_index(index)?,
                });
            }
            // An assignment's clause is its value expression and the write,
            // both inside the region: the write is what the clause *is*, and
            // the boundary that follows it is where a `CALL ON` handler queued
            // by the value expression runs -- measured, the handler sees the
            // assignment already done.
            InstructionKind::Assignment { target, value } => {
                let mark = registers.mark();
                let dst = registers.alloc()?;
                let at = op_index(&ops)?;
                let echo = echoes(trace, instruction);
                ops.push(Op::Clause {
                    index: instruction_index(index)?,
                    end: 0,
                });
                push_echo(&mut ops, echo, instruction_index(index)?);
                push_value(
                    &mut ops,
                    echoes_values,
                    &mut consts,
                    &mut registers,
                    &mut hints,
                    &mut calls,
                    plan,
                    value,
                    instruction_index(index)?,
                    0,
                    dst,
                )?;
                ops.push(Op::Store {
                    index: instruction_index(index)?,
                    at: write_slot(plan, target),
                    src: dst,
                });
                close_region(&mut ops, at)?;
                registers.release(mark);
            }
            // A `SAY` is the same shape with the print in place of the write,
            // and one register fewer when it has no expression: the bare form
            // prints a blank line and traces the null string, which is a
            // decision `Op::Say` carries rather than an empty register.
            InstructionKind::Say { expression } => {
                let mark = registers.mark();
                let at = op_index(&ops)?;
                let echo = echoes(trace, instruction);
                ops.push(Op::Clause {
                    index: instruction_index(index)?,
                    end: 0,
                });
                push_echo(&mut ops, echo, instruction_index(index)?);
                let src = match expression {
                    Some(expression) => {
                        let dst = registers.alloc()?;
                        push_value(
                            &mut ops,
                            echoes_values,
                            &mut consts,
                            &mut registers,
                            &mut hints,
                            &mut calls,
                            plan,
                            expression,
                            instruction_index(index)?,
                            0,
                            dst,
                        )?;
                        Some(dst)
                    }
                    None => None,
                };
                ops.push(Op::Say {
                    index: instruction_index(index)?,
                    src,
                });
                close_region(&mut ops, at)?;
                registers.release(mark);
            }
            // `SIGNAL`, all three forms. The `SAY` shape, with the expression
            // present only for `VALUE` -- and it goes through `push_value`
            // like any other, so a `SIGNAL VALUE` whose expression is a
            // literal or a bare symbol reaches the same native op an
            // assignment's value would.
            InstructionKind::Signal(signal) => {
                let mark = registers.mark();
                let at = op_index(&ops)?;
                let echo = echoes(trace, instruction);
                ops.push(Op::Clause {
                    index: instruction_index(index)?,
                    end: 0,
                });
                push_echo(&mut ops, echo, instruction_index(index)?);
                let src = match &**signal {
                    rexx_parse::Signal::Value(expression) => {
                        let dst = registers.alloc()?;
                        push_value(
                            &mut ops,
                            echoes_values,
                            &mut consts,
                            &mut registers,
                            &mut hints,
                            &mut calls,
                            plan,
                            expression,
                            instruction_index(index)?,
                            0,
                            dst,
                        )?;
                        Some(dst)
                    }
                    rexx_parse::Signal::Label(_) | rexx_parse::Signal::Trap(_) => None,
                };
                ops.push(Op::Signal {
                    index: instruction_index(index)?,
                    src,
                });
                close_region(&mut ops, at)?;
                registers.release(mark);
            }
            // `PARSE`, `ARG` and `PULL`, which are one instruction with three
            // spellings and reach one `exec_parse`. The `SAY` shape, with the
            // expression present only for `PARSE VALUE expr WITH` -- every
            // other source reads a variable, an argument list or a line, none
            // of which is an `Expr` a register could hold. `Op::Parse`'s own
            // doc has why the template's `(expr)` patterns stay behind.
            InstructionKind::Parse(parse)
            | InstructionKind::Arg(parse)
            | InstructionKind::Pull(parse) => {
                let mark = registers.mark();
                let at = op_index(&ops)?;
                let echo = echoes(trace, instruction);
                ops.push(Op::Clause {
                    index: instruction_index(index)?,
                    end: 0,
                });
                push_echo(&mut ops, echo, instruction_index(index)?);
                let src = match &parse.source {
                    ParseSource::Value(Some(expression)) => {
                        let dst = registers.alloc()?;
                        push_value(
                            &mut ops,
                            echoes_values,
                            &mut consts,
                            &mut registers,
                            &mut hints,
                            &mut calls,
                            plan,
                            expression,
                            instruction_index(index)?,
                            0,
                            dst,
                        )?;
                        Some(dst)
                    }
                    _ => None,
                };
                ops.push(Op::Parse {
                    index: instruction_index(index)?,
                    src,
                });
                close_region(&mut ops, at)?;
                registers.release(mark);
            }
            // `RETURN` and `EXIT` are the `SAY` shape with a `Flow` in place
            // of the print, and one arm rather than two because the keyword is
            // all that differs -- read off the kind here exactly as `step`'s
            // own `PUSH`/`QUEUE` arm reads its end of the queue.
            InstructionKind::Return { expression } | InstructionKind::Exit { expression } => {
                let keyword = if matches!(instruction.kind, InstructionKind::Return { .. }) {
                    ReturnKeyword::Return
                } else {
                    ReturnKeyword::Exit
                };
                let mark = registers.mark();
                let at = op_index(&ops)?;
                let echo = echoes(trace, instruction);
                ops.push(Op::Clause {
                    index: instruction_index(index)?,
                    end: 0,
                });
                push_echo(&mut ops, echo, instruction_index(index)?);
                let src = match expression {
                    Some(expression) => {
                        let dst = registers.alloc()?;
                        push_value(
                            &mut ops,
                            echoes_values,
                            &mut consts,
                            &mut registers,
                            &mut hints,
                            &mut calls,
                            plan,
                            expression,
                            instruction_index(index)?,
                            0,
                            dst,
                        )?;
                        Some(dst)
                    }
                    None => None,
                };
                ops.push(Op::Return {
                    index: instruction_index(index)?,
                    src,
                    keyword,
                });
                close_region(&mut ops, at)?;
                registers.release(mark);
            }
            // `PUSH` and `QUEUE` are the `SAY` shape with the queue in place
            // of the print, one arm for the reason the pair above is one.
            InstructionKind::Push { expression } | InstructionKind::Queue { expression } => {
                let keyword = if matches!(instruction.kind, InstructionKind::Push { .. }) {
                    QueueKeyword::Push
                } else {
                    QueueKeyword::Queue
                };
                let mark = registers.mark();
                let at = op_index(&ops)?;
                let echo = echoes(trace, instruction);
                ops.push(Op::Clause {
                    index: instruction_index(index)?,
                    end: 0,
                });
                push_echo(&mut ops, echo, instruction_index(index)?);
                let src = match expression {
                    Some(expression) => {
                        let dst = registers.alloc()?;
                        push_value(
                            &mut ops,
                            echoes_values,
                            &mut consts,
                            &mut registers,
                            &mut hints,
                            &mut calls,
                            plan,
                            expression,
                            instruction_index(index)?,
                            0,
                            dst,
                        )?;
                        Some(dst)
                    }
                    None => None,
                };
                ops.push(Op::Queue {
                    index: instruction_index(index)?,
                    src,
                    keyword,
                });
                close_region(&mut ops, at)?;
                registers.release(mark);
            }
            // A `CALL name`/`CALL "name"` clause, whose arguments compile to
            // ops of their own when every one of them is a value -- a `>name`
            // reference carries the caller's own slot rather than an `ObjRef`,
            // and an argument that is itself a call needs an address this
            // instruction has no expression slot for, so either one keeps the
            // whole clause on `Op::Call` and leaves the arguments, the `>A>`
            // lines and their intermediates to `Interp::invoke_call`.
            InstructionKind::Call(call) if matches!(&**call, Call::Named { .. }) => {
                let Call::Named { args, .. } = &**call else {
                    unreachable!("the guard above admits only `Call::Named`")
                };
                let mark = registers.mark();
                let at = op_index(&ops)?;
                let echo = echoes(trace, instruction);
                ops.push(Op::Clause {
                    index: instruction_index(index)?,
                    end: 0,
                });
                push_echo(&mut ops, echo, instruction_index(index)?);
                // **The arguments become ops of their own where they can**,
                // exactly as `ExprKind::Call`'s arm does it, and the guard is
                // the identical one: `native_shape` with no address declines a
                // nested call -- which would need a slot this instruction has
                // none of -- and declines the `>v` reference form, which
                // carries a variable's home where the stack carries a value.
                let native = u16::try_from(args.len()).is_ok()
                    && args
                        .iter()
                        .all(|arg| arg.as_ref().is_none_or(|expr| native_shape(expr, None)));
                if native {
                    let argc = u16::try_from(args.len()).map_err(|_| ChunkTooLarge {
                        what: "call arguments past u16",
                    })?;
                    for arg in args {
                        let src = match arg {
                            None => Op::ARG_OMITTED,
                            Some(expr) => {
                                let src = registers.alloc()?;
                                push_native(
                                    &mut ops,
                                    echoes_values,
                                    &mut consts,
                                    &mut registers,
                                    &mut hints,
                                    &mut calls,
                                    plan,
                                    expr,
                                    instruction_index(index)?,
                                    0,
                                    None,
                                    src,
                                )?;
                                src
                            }
                        };
                        ops.push(Op::PushArg { src });
                        // Behind the argument's own ops, so its `>L>`/`>V>`
                        // lines print first -- the order `invoke_call`'s own
                        // loop produces.
                        if echoes_values {
                            ops.push(Op::TraceArgument { src });
                        }
                    }
                    ops.push(Op::CallNamed {
                        index: instruction_index(index)?,
                        site: calls.reserve()?,
                        argc,
                    });
                } else {
                    ops.push(Op::Call {
                        index: instruction_index(index)?,
                        site: calls.reserve()?,
                    });
                }
                close_region(&mut ops, at)?;
                registers.release(mark);
            }
            // A message send as a whole clause: the plain form, the `~~`
            // form, and the message-assignment form. No register and no
            // expression slot -- `Interp::exec_message` evaluates the term
            // itself, exactly as arm does, so what this
            // promotion decides is the clause region around it and nothing
            // about the send.
            InstructionKind::Message { .. } => {
                let at = op_index(&ops)?;
                let echo = echoes(trace, instruction);
                ops.push(Op::Clause {
                    index: instruction_index(index)?,
                    end: 0,
                });
                push_echo(&mut ops, echo, instruction_index(index)?);
                ops.push(Op::Message {
                    index: instruction_index(index)?,
                });
                close_region(&mut ops, at)?;
            }
            // `EXPOSE`. No register and no expression slot: the names come
            // out of the instruction, and a `VariableRef::Indirect` reads its
            // selector through `Interp::read`, so what this promotion decides
            // is the clause
            // region around it and nothing about the binding.
            InstructionKind::Expose { .. } => {
                let at = op_index(&ops)?;
                let echo = echoes(trace, instruction);
                ops.push(Op::Clause {
                    index: instruction_index(index)?,
                    end: 0,
                });
                push_echo(&mut ops, echo, instruction_index(index)?);
                ops.push(Op::Expose {
                    index: instruction_index(index)?,
                });
                close_region(&mut ops, at)?;
            }
            // `LEAVE`/`ITERATE`. No register and no expression slot: the name
            // is in the instruction and the origin is captured at run time
            // from the clause the region has open.
            InstructionKind::Leave { .. } | InstructionKind::Iterate { .. } => {
                let at = op_index(&ops)?;
                let echo = echoes(trace, instruction);
                ops.push(Op::Clause {
                    index: instruction_index(index)?,
                    end: 0,
                });
                push_echo(&mut ops, echo, instruction_index(index)?);
                ops.push(Op::Escape {
                    index: instruction_index(index)?,
                });
                close_region(&mut ops, at)?;
            }
            // The marker instructions: `step` answers `Ok(Flow::Next)` for
            // each and computes nothing, so the clause region is the whole of
            // what they do and it holds no op but its own echo
            // ([`Op::Clause`]'s own doc comment).
            InstructionKind::Nop
            | InstructionKind::Then
            | InstructionKind::Else { .. }
            | InstructionKind::Otherwise
            | InstructionKind::Label { .. } => {
                let at = op_index(&ops)?;
                let echo = echoes(trace, instruction);
                ops.push(Op::Clause {
                    index: instruction_index(index)?,
                    end: 0,
                });
                push_echo(&mut ops, echo, instruction_index(index)?);
                close_region(&mut ops, at)?;
            }
            // **SPIKE.** The `END` of a repeating loop, which the flattened
            // form reaches by falling out of the body rather than by a range
            // check. An `END` closing anything else keeps the delegating
            // `Op::Exec` region below.
            InstructionKind::End { .. } if loop_of_end[index].is_some() => {
                ops.push(Op::LoopNext {
                    index: loop_of_end[index].expect("the guard just observed it"),
                });
            }
            // The kinds whose whole execution is one call into
            // `Interp::exec_instruction`. What the promotion decides is the
            // clause region around that call and nothing about the call: the
            // region owes the echo, the boundary, the temps frame, the
            // deadline count and the failing clause's site, and
            // [`Op::Exec`] owes the work.
            InstructionKind::Command { .. }
            | InstructionKind::End { .. }
            | InstructionKind::Drop { .. }
            | InstructionKind::Call(..)
            | InstructionKind::Procedure { .. }
            | InstructionKind::Interpret { .. }
            | InstructionKind::Guard { .. }
            | InstructionKind::Reply { .. }
            | InstructionKind::Forward { .. }
            | InstructionKind::Raise { .. }
            | InstructionKind::Use { .. }
            | InstructionKind::Numeric { .. }
            | InstructionKind::Address { .. }
            | InstructionKind::Trace { .. }
            | InstructionKind::Options { .. } => {
                let at = op_index(&ops)?;
                let echo = echoes(trace, instruction);
                ops.push(Op::Clause {
                    index: instruction_index(index)?,
                    end: 0,
                });
                push_echo(&mut ops, echo, instruction_index(index)?);
                ops.push(Op::Exec {
                    index: instruction_index(index)?,
                });
                close_region(&mut ops, at)?;
            }
            // An **absorbed** `WHEN` or `WHEN CASE`: one that is itself
            // another `WHEN`'s consequence, so the enclosing `SELECT` never
            // collected it and it is nobody's listed branch. The listed forms
            // took their own arm above; this is what falls past it.
            InstructionKind::When { .. } | InstructionKind::WhenCase { .. } => {
                let at = op_index(&ops)?;
                let echo = echoes(trace, instruction);
                ops.push(Op::Clause {
                    index: instruction_index(index)?,
                    end: 0,
                });
                push_echo(&mut ops, echo, instruction_index(index)?);
                ops.push(Op::Exec {
                    index: instruction_index(index)?,
                });
                close_region(&mut ops, at)?;
            }
        }
    }
    // A branch that ends at the body's own end still owes its boundary, and
    // there is no instruction iteration left to emit it in.
    emit_before(&mut ops, &mut patches, &mut before[len])?;
    // One entry past the last instruction, pushed after the loop above:
    // `run_bounded`'s absorption guard is inclusive, so a construct's
    // resume point can be `end`, one past its last instruction, and a table
    // that stopped at `len - 1` would panic there instead of failing loudly
    // at compile.
    let end = op_index(&ops)?;
    op_of.push(end);
    first_op_of.push(end);

    for patch in patches {
        // In range for every target: both tables have an entry per
        // instruction plus the end one, and `if_targets` clamps `None` to
        // `len`.
        let target = match patch.kind {
            PatchKind::Enter => first_op_of[patch.target],
            PatchKind::Resume => op_of[patch.target],
        };
        match &mut ops[patch.op as usize] {
            Op::Jump { target: slot }
            | Op::JumpUnless { target: slot, .. }
            | Op::ConditionJump { target: slot, .. } => *slot = target,
            _ => unreachable!("a patch names the op it was recorded beside"),
        }
    }

    assert_trace_ops_open_a_clause_region(&ops);
    assert_literal_echoes_follow_their_load(&ops);
    assert_read_echoes_follow_their_load(&ops);
    assert_operator_echoes_follow_their_op(&ops);
    assert_prefix_echoes_follow_their_op(&ops);
    assert_call_echoes_follow_their_op(&ops);
    assert_keyword_echoes_precede_their_value(&ops);
    assert_region_ops_name_their_clause(&ops);
    assert_exec_regions_hold_nothing_else(&ops);

    let consts = consts.values;
    // Sized from the stream rather than from the program's symbol table, which
    // `compile` does not have: the widest constant symbol this body actually
    // names, and nothing for a body that names none. `Chunk::interned_symbols`
    // has why the gap in front of a high id is the right trade.
    let widest_constant_symbol = ops
        .iter()
        .filter_map(|op| match op {
            Op::LoadConstant { symbol, .. } => Some(symbol.index()),
            _ => None,
        })
        .max();
    let registers = registers.high_water();
    Ok(Chunk {
        trace,
        #[cfg(debug_assertions)]
        settings,
        ops: super::valid::ValidOps::new(ops, registers)?,
        op_of,
        registers,
        positions: clause_positions(body, plan),
        interned: vec![std::cell::Cell::new(rexx_core::ObjRef::NIL); consts.len()],
        interned_symbols: vec![
            std::cell::Cell::new(rexx_core::ObjRef::NIL);
            widest_constant_symbol.map_or(0, |widest| widest + 1)
        ],
        consts,
        hints,
        calls,
    })
}

/// The ops that leave expression `slot` of instruction `index` in register
/// `dst`.
#[expect(
    clippy::too_many_arguments,
    reason = "the emission sinks, the plan a read resolves its slot against, and the address an \
              EvalExpr or a call op has to name"
)]
fn push_value<'a>(
    ops: &mut Vec<Op>,
    echoes_values: bool,
    consts: &mut Constants<'a>,
    registers: &mut Registers,
    hints: &mut Hints,
    calls: &mut Calls,
    plan: &Plan,
    expr: &'a Expr,
    index: u32,
    slot: u32,
    dst: u16,
) -> Result<(), ChunkTooLarge> {
    if native_shape(expr, Some(NodePath::ROOT)) {
        push_native(
            ops,
            echoes_values,
            consts,
            registers,
            hints,
            calls,
            plan,
            expr,
            index,
            slot,
            Some(NodePath::ROOT),
            dst,
        )
    } else {
        ops.push(Op::EvalExpr { index, slot, dst });
        Ok(())
    }
}

/// Whether `expr` compiles to native ops entire, with `eval.rs` not entered
/// for any part of it.
fn native_shape(expr: &Expr, path: Option<NodePath>) -> bool {
    match &expr.kind {
        ExprKind::Literal(_)
        | ExprKind::Constant(_)
        | ExprKind::Variable(_)
        | ExprKind::Stem(_)
        | ExprKind::Compound(_) => true,
        // A call needs an address, and a node past the width has none. The
        // whole slot then falls to `Op::EvalExpr`, which is the answer it had
        // before there was an address at all.
        ExprKind::Call { .. } => path.is_some(),
        ExprKind::Binary { op, left, right } => {
            is_native_binary(*op)
                && native_shape(left, descend(path, false))
                && native_shape(right, descend(path, true))
        }
        // No operator condition beside the operand's, unlike the arm above:
        // `Interp::apply_prefix` matches `PrefixOp` exhaustively, so a variant
        // added to that enum is a compile error there rather than an operator
        // silently promoted here.
        ExprKind::Prefix { operand, .. } => native_shape(operand, descend(path, false)),
        _ => false,
    }
}

/// The address of one child of the node addressed by `path`: the right child
/// when `right`, and the left or only one otherwise.
fn descend(path: Option<NodePath>, right: bool) -> Option<NodePath> {
    path?.child(right)
}

/// The ops that leave `expr` -- which [`native_shape`] has already accepted --
/// in register `dst`.
#[expect(
    clippy::too_many_arguments,
    reason = "the emission sinks, the plan a read resolves its slot against, and the address a \
              call op has to name"
)]
fn push_native<'a>(
    ops: &mut Vec<Op>,
    echoes_values: bool,
    consts: &mut Constants<'a>,
    registers: &mut Registers,
    hints: &mut Hints,
    calls: &mut Calls,
    plan: &Plan,
    expr: &'a Expr,
    index: u32,
    slot: u32,
    path: Option<NodePath>,
    dst: u16,
) -> Result<(), ChunkTooLarge> {
    match &expr.kind {
        ExprKind::Literal(bytes) => {
            ops.push(Op::Const {
                dst,
                konst: consts.intern(bytes)?,
            });
            // Behind the load rather than in front of it, because `eval.rs`
            // emits this line post-order, with the value in hand.
            if echoes_values {
                ops.push(Op::TraceLiteral { src: dst });
            }
        }
        // A constant symbol's value is its own upcased spelling, which lives
        // in the symbol table rather than in the node -- so the symbol travels
        // in the op and the spelling is read at run time, where the table is
        // in hand. The echo behind it is the literal's, because the line is the
        // same `>L>`.
        ExprKind::Constant(id) => {
            ops.push(Op::LoadConstant { symbol: *id, dst });
            if echoes_values {
                ops.push(Op::TraceLiteral { src: dst });
            }
        }
        // The three bare-symbol reads. **`ExprKind::DotVariable` and
        // `ExprKind::VariableReference` are not among them and are not
        // reads**: they trace `>E>` and `>O>` rather than `>V>`.
        ExprKind::Variable(id) => push_read(ops, echoes_values, plan, SymbolRead::Simple, *id, dst),
        ExprKind::Stem(id) => push_read(ops, echoes_values, plan, SymbolRead::Stem, *id, dst),
        ExprKind::Compound(id) => {
            push_read(ops, echoes_values, plan, SymbolRead::Compound, *id, dst)
        }
        ExprKind::Binary { op, left, right } => {
            // **The left operand lands in `dst` itself and only the right one
            // takes a register of its own**, which is what keeps a chain's
            // register cost at two however long it runs: `za + zb + zc + zd`
            // is left-nested, so each operator writes its result back into the
            // register the next one reads as its left. Both sources are read
            // before the destination is written, which is the whole of what
            // makes the aliasing safe.
            push_native(
                ops,
                echoes_values,
                consts,
                registers,
                hints,
                calls,
                plan,
                left,
                index,
                slot,
                descend(path, false),
                dst,
            )?;
            let mark = registers.mark();
            let rhs = registers.alloc()?;
            push_native(
                ops,
                echoes_values,
                consts,
                registers,
                hints,
                calls,
                plan,
                right,
                index,
                slot,
                descend(path, true),
                rhs,
            )?;
            // **Only arithmetic takes a hint slot**, because only arithmetic
            // has a second path for one to choose. `Chunk::hints` is dense over
            // the ops that can specialise, so a slot reserved for an operator
            // that never reads one would shift every later arithmetic site's
            // index by one and hand it somebody else's decision.
            ops.push(if is_arithmetic(*op) {
                Op::Arith {
                    op: *op,
                    hint: hints.reserve()?,
                    lhs: dst,
                    rhs,
                    dst,
                }
            } else {
                Op::Binary {
                    op: *op,
                    lhs: dst,
                    rhs,
                    dst,
                }
            });
            // Behind the operation rather than in front of it, because
            // `eval.rs` emits this line post-order, with the value in hand --
            // so an inner operator's line precedes the outer one's.
            if echoes_values {
                ops.push(Op::TraceOperator { op: *op, src: dst });
            }
            // Held until the operation has run: the right operand's register
            // is live right up to it, and a release any earlier would hand it
            // out to the next operand of an enclosing operator.
            registers.release(mark);
        }
        ExprKind::Prefix { op, operand } => {
            // **The operand lands in `dst` itself and no register is taken**,
            // which is the one-operand case of the discipline the binary arm
            // above states: the operand is read before the destination is
            // written, so a chain of prefixes costs the registers its innermost
            // term does and no more.
            push_native(
                ops,
                echoes_values,
                consts,
                registers,
                hints,
                calls,
                plan,
                operand,
                index,
                slot,
                descend(path, false),
                dst,
            )?;
            ops.push(Op::Prefix {
                op: *op,
                src: dst,
                dst,
            });
            // Behind the operation rather than in front of it, because
            // `eval.rs` emits this line post-order, with the value in hand --
            // so an inner operator's line precedes the outer one's.
            if echoes_values {
                ops.push(Op::TracePrefix { op: *op, src: dst });
            }
        }
        // **A call takes its own op wherever it sits**, which is
        // [`Op::CallExpr`], and the only thing that op changes about running
        // the call is that the resolution comes from a site instead of being
        // made afresh. What it needs and no other arm here does is an address:
        // the value is not computed from operand registers, so the driver goes
        // back to the node to find the target and the arguments.
        ExprKind::Call { args, .. } => {
            let slot16 = u16::try_from(slot).map_err(|_| ChunkTooLarge {
                what: "expression slots past u16",
            })?;
            let path = path.expect("native_shape accepts a call only where an address reaches it");
            // **The arguments become ops of their own where they can.** Then
            // the call op takes values off the driver's argument stack
            // instead of walking back to the node for expressions and
            // evaluating each through `eval`. What that saves is the descent,
            // the name lookup and one `eval` entry per argument; measured on
            // a ladder of `zq = length(s)` clauses, a builtin call cost 617
            // instructions against the -O3 interpreter's 293.
            if args
                .iter()
                .all(|arg| arg.as_ref().is_none_or(|expr| native_shape(expr, None)))
            {
                let argc = u16::try_from(args.len()).map_err(|_| ChunkTooLarge {
                    what: "call arguments past u16",
                })?;
                let mark = registers.mark();
                for arg in args {
                    match arg {
                        None => {
                            ops.push(Op::PushArg {
                                src: Op::ARG_OMITTED,
                            });
                            if echoes_values {
                                ops.push(Op::TraceArgument {
                                    src: Op::ARG_OMITTED,
                                });
                            }
                        }
                        Some(expr) => {
                            let src = registers.alloc()?;
                            push_native(
                                ops,
                                echoes_values,
                                consts,
                                registers,
                                hints,
                                calls,
                                plan,
                                expr,
                                index,
                                slot,
                                None,
                                src,
                            )?;
                            ops.push(Op::PushArg { src });
                            // Behind the argument's own ops, so its `>L>`/
                            // `>V>` lines print first -- the order
                            // `invoke_call`'s own loop produces.
                            if echoes_values {
                                ops.push(Op::TraceArgument { src });
                            }
                        }
                    }
                }
                ops.push(Op::CallArgs {
                    slot: slot16,
                    path,
                    site: calls.reserve()?,
                    argc,
                    dst,
                });
                registers.release(mark);
                ops.push(Op::TraceFunction {
                    index,
                    slot: slot16,
                    path,
                    src: dst,
                });
                return Ok(());
            }
            let slot = slot16;
            ops.push(Op::CallExpr {
                index,
                slot,
                path,
                site: calls.reserve()?,
                dst,
            });
            // Behind the call rather than in front of it, because `eval.rs`
            // emits this line post-order, with the value in hand.
            ops.push(Op::TraceFunction {
                index,
                slot,
                path,
                src: dst,
            });
        }
        _ => unreachable!("this descends only into an expression native_shape accepted"),
    }
    Ok(())
}

/// The slot one assignment *target* resolves to, or [`PlanSlot::UNRESOLVED`]
/// for a target that does not write a slot by name.
fn write_slot(plan: &Plan, target: &Expr) -> PlanSlot {
    match &target.kind {
        ExprKind::Variable(id) => plan
            .slot_for_symbol(*id)
            .map_or(PlanSlot::UNRESOLVED, PlanSlot::of),
        _ => PlanSlot::UNRESOLVED,
    }
}

/// The two ops one bare-symbol read is: the load, and the `>V>`/`>C>` line
/// that reading it owes.
fn push_read(
    ops: &mut Vec<Op>,
    echoes_values: bool,
    plan: &Plan,
    read: SymbolRead,
    symbol: SymbolId,
    dst: u16,
) {
    let at = match read {
        SymbolRead::Simple | SymbolRead::Stem => plan
            .slot_for_symbol(symbol)
            .map_or(PlanSlot::UNRESOLVED, PlanSlot::of),
        SymbolRead::Compound => PlanSlot::UNRESOLVED,
    };
    ops.push(Op::Load {
        symbol,
        read,
        at,
        dst,
    });
    // Behind the load rather than in front of it, because `eval.rs` emits
    // these lines post-order, with the value in hand.
    if echoes_values {
        ops.push(Op::TraceRead {
            symbol,
            read,
            src: dst,
        });
    }
}

/// Whether a promoted clause of `instruction` echoes under `trace`.
fn echoes(trace: ChunkTrace, instruction: &Instruction) -> bool {
    trace.echoes(
        matches!(instruction.kind, InstructionKind::Label { .. }),
        matches!(instruction.kind, InstructionKind::Command { .. }),
    )
}

/// Pushes the clause echo op, if `echo`, as the **first** op of the region
/// that follows -- the position clause unit echoes at,
/// before anything the clause computes.
fn push_echo(ops: &mut Vec<Op>, echo: bool, index: u32) {
    if echo {
        ops.push(Op::TraceClause { index });
    }
}

/// Emits the ops that go in front of one instruction, innermost first.
fn emit_before(
    ops: &mut Vec<Op>,
    patches: &mut Vec<Patch>,
    before: &mut [Before],
) -> Result<(), ChunkTooLarge> {
    before.reverse();
    // **A `SELECT` branch's close goes in front of everything else here**,
    // which is where the driver used to ask the question: it tested for an
    // ended frame before fetching the op at all, so a `WHEN` whose body is an
    // `IF` left the branch -- and jumped to the `SELECT`'s own end -- without
    // ever reaching that `IF`'s [`Op::EndBranch`]. Emitting the close second
    // runs that boundary first, which is a clause boundary that did not run
    // before. The sort is stable, so nested branches keep the innermost-first
    // order the reverse above gives them.
    before.sort_by_key(|entry| match entry {
        Before::SelectBranchEnd => 0,
        Before::ThenEnd { .. } | Before::EnterOtherwise(_) => 1,
    });
    debug_assert!(
        before
            .iter()
            .rev()
            .skip(1)
            .all(|entry| !matches!(entry, Before::ThenEnd { resume: Some(_) })),
        "an op in front of an instruction carries a jump with more ops behind it, which \
         nothing would reach"
    );
    for entry in before.iter() {
        match entry {
            Before::ThenEnd { resume } => {
                ops.push(Op::EndBranch);
                if let Some(resume) = resume {
                    let op = op_index(ops)?;
                    ops.push(Op::Jump { target: 0 });
                    patches.push(Patch {
                        op,
                        target: *resume,
                        kind: PatchKind::Resume,
                    });
                }
            }
            Before::SelectBranchEnd => ops.push(Op::EndWhen),
            Before::EnterOtherwise(select) => ops.push(Op::EnterOtherwise {
                select: instruction_index(*select)?,
            }),
        }
    }
    Ok(())
}

/// Records that `mark` is dead by the time the instruction this slot belongs to
/// is reached, keeping **the lowest** mark any construct wants released there.
fn release_to(slot: &mut Option<Mark>, mark: Mark) {
    let lowest = match *slot {
        Some(existing) if existing.0 <= mark.0 => existing,
        _ => mark,
    };
    *slot = Some(lowest);
}

/// Rewrites the [`Op::Clause`] at `at` so that its `end` is the position one
/// past everything pushed since -- the region's own end.
fn close_region(ops: &mut [Op], at: u32) -> Result<(), ChunkTooLarge> {
    let end = op_index(ops)?;
    match &mut ops[at as usize] {
        Op::Clause { end: slot, .. } => *slot = end,
        _ => unreachable!("close_region is given the index of the Clause op it closes"),
    }
    Ok(())
}

/// Every clause's [`ClausePosition`], or an empty table where the plan cannot
/// supply one for each.
fn clause_positions(body: &CodeBody, plan: &Plan) -> Box<[ClausePosition]> {
    let len = body.instructions.len();
    if plan.lines.len() != len || plan.indents.len() != len {
        return Box::default();
    }
    let mut positions = Vec::with_capacity(len);
    for (line, indent) in plan.lines.iter().zip(plan.indents.iter()) {
        let (Ok(line), Ok(indent)) = (u32::try_from(*line), u32::try_from(*indent)) else {
            return Box::default();
        };
        positions.push(ClausePosition { line, indent });
    }
    positions.into_boxed_slice()
}

/// The index the next op will be pushed at, refused rather than wrapped.
fn op_index(ops: &[Op]) -> Result<u32, ChunkTooLarge> {
    u32::try_from(ops.len()).map_err(|_| ChunkTooLarge { what: "op stream" })
}

/// An instruction index as an op payload, refused rather than wrapped.
fn instruction_index(index: usize) -> Result<u32, ChunkTooLarge> {
    u32::try_from(index).map_err(|_| ChunkTooLarge { what: "op stream" })
}

/// **What wiring the analysis into emission may and may not do**, asserted
/// while it is not yet wired in, so that the commit that wires it in cannot
/// be the one that discovers either.
///
/// * A body nothing in which can change the setting has one answer at every
///   clause, and it is the setting this chunk is keyed under -- so no such
///   body loses the compile-time decision it already has.
/// * No clause gains an echo the body-wide bool does not already carry, in
///   either of the two questions those bools answer.
#[cfg(debug_assertions)]
fn assert_analysis_only_narrows(
    plan: &Plan,
    trace: ChunkTrace,
    settings: &[super::trace_flow::Setting],
) {
    // The two body-wide decisions this analysis replaced, restated here
    // because this is now the only thing that reads them.
    let never_retraces = plan
        .trace_events()
        .iter()
        .all(|event| *event == crate::trace::TraceEvent::Keeps);
    let echoes_values = trace.intermediates() || !never_retraces;
    let echoes_keyword = trace.results() || !never_retraces;
    debug_assert!(
        !never_retraces
            || settings
                .iter()
                .all(|setting| *setting == super::trace_flow::Setting::Known(trace)),
        "nothing in this body can change the TRACE setting, so every clause runs under the one \
         the chunk is keyed to: {settings:?}"
    );
    debug_assert!(
        settings
            .iter()
            .all(|setting| (!setting.echoes_values() || echoes_values)
                && (!setting.echoes_keyword() || echoes_keyword)),
        "a clause's own answer asks for an echo this chunk does not emit today: {settings:?}"
    );
}

// Test-only instrumentation: how many times `compile` has actually run. The
// chunk-cache test (`golden_tests.rs`) needs this to tell "the chunk cache
// compiled the body once" apart from "the second lookup happened not to
// fail" -- an implementation that recompiles on every call and returns an
// equal-looking `Chunk` would still pass a check that only inspects the
// result, and a `thread_local` counter is what lets the test inspect the
// call count instead.
#[cfg(test)]
thread_local! {
    static COMPILE_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
fn count_compile_call() {
    COMPILE_CALLS.with(|calls| calls.set(calls.get() + 1));
}

#[cfg(test)]
pub(crate) fn compile_calls() -> usize {
    COMPILE_CALLS.with(|calls| calls.get())
}

#[cfg(test)]
mod tests;
