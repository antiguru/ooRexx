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

//! The driver: what runs a compiled [`Chunk`] for the activation on top of
//! the stack.
//!
//! **The program counter is an op index.** That is the whole of what makes a
//! promoted construct possible: an `IF` compiles to a run of ops sitting
//! inside one instruction's position, and a counter that walks instructions
//! cannot enter one. The instruction space has not gone away -- `Flow::Goto`
//! and `Flow::Signal` both carry instruction indices, and `Chunk::op_of` is
//! the one table where the two spaces meet.
//!
//! It sits beside `run_activation`'s own loop rather than replacing it --
//! `Interp::engine` chooses between the two -- and it discharges exactly the
//! same per-clause obligations, through the same functions, because those
//! were extracted from that loop rather than copied out of it:
//! `grant_procedure_permission`, the clause unit, `offer_to_trap`,
//! `apply_flow` and `absorb`.
//!
//! **The clause unit is reached by its two halves here and by its closure form
//! there, and they are two entry shapes into one implementation** --
//! `Interp::enter_stepped_clause` and `Interp::leave_stepped_clause`, which
//! `Interp::in_stepped_clause_with` is itself defined in terms of. That is what
//! lets a promoted clause's ops run in this file's own loop, with no callee
//! between the clause's two ends, without the driver owning a second copy of
//! what a clause boundary owes.

use rexx_core::{Decoded, FrameId, ObjRef};
use rexx_parse::{Call, ExprKind, Instruction, InstructionKind, ProgramSource, SymbolId};

use super::{BodyEngine, Chunk, ConditionKeyword, Op};
use crate::clause::{ClauseOutcome, ClauseValue};
use crate::eval::{SymbolRead, call_target_name};
use crate::run::{
    Absorbed, ConditionTrace, Echo, Ended, Flow, LoopHeaderValues, QueueKeyword, ReturnKeyword,
    SelectEscape, SelectResume, absorb, otherwise_range, otherwise_resume, select_escape,
    select_parts, when_resume, when_targets,
};
use crate::{Code, Failure, Interp, Loud};

/// What a body that runs off its own end answers.
///
/// `Exited` and not `Returned`, for the reason `Ended::Exited`'s own doc
/// gives and `run_activation`'s loop ends with: a callee that runs off the
/// end of the file ends the *program*, and the caller's next clause never
/// runs.
const END_OF_BODY: Ended = Ended::Exited(None);

/// One construct the driver has open: a `SELECT` branch that is running, and
/// everything an escaping `Flow` needs in order to leave it.
///
/// **This is the tree-walker's own Rust call frame, flattened.** There, a
/// matched `WHEN`'s branch is `run_bounded(code, when + 1, body_end, ...)?`
/// followed by `leave_select`, and the call stack is what makes an escaping
/// `Flow` meet that `leave_select` before it meets the enclosing range. A flat
/// op stream has no call between the two, so the driver keeps the frames
/// itself and an escaping `Flow` walks them.
///
/// **One stack on `Interp`, sliced per [`Interp::run_ops`] call.** Each entry
/// records the length it found and treats everything below as another level's,
/// so a `DO` body entering `run_ops` again through `run_bounded_from_chunk`
/// still gets a region of its own: a `LEAVE` there meets the loop first and
/// this frame afterwards, in that order, exactly as the nested calls give.
/// [`Interp::unwind_frames`] is what a `Failure` costs -- it unwinds out of
/// `run_ops` with frames still open, and the entry boundary truncates back to
/// the length it recorded rather than each raise remembering to.
pub(crate) struct SelectFrame {
    /// The `SELECT` instruction this branch belongs to, which is the position
    /// `pop_search_frame` resets a forwarded `LEAVE`'s indent to.
    select: usize,
    /// `SELECT LABEL name`'s own label.
    label: Option<SymbolId>,
    /// That `SELECT`'s own `OTHERWISE` marker, for [`select_escape`].
    otherwise: Option<usize>,
    /// The branch's own instruction range -- the range the tree-walker bounds
    /// the identical `run_bounded` call to, and the one an escaping `Flow` is
    /// absorbed against here.
    start: usize,
    end: usize,
    /// Where `leave_select` resumes this branch, which is two answers rather
    /// than one ([`SelectResume`]).
    resume: SelectResume,
    /// The `SELECT`'s own `END`, as the node records it, for
    /// [`Interp::leave_otherwise`] to build the same answer from.
    select_end: Option<usize>,
    /// One past the branch's last op. **Reaching it is the branch running off
    /// its own end**, which is the arrival the tree-walker gets as
    /// `run_bounded` answering `Flow::Next` -- and it is an op position rather
    /// than an instruction one because that is the space the counter walks.
    op_end: u32,
    /// Which branch this is, which decides how it is left.
    branch: Branch,
}

/// **SPIKE.** One construct this level has open: a `SELECT`'s branch, or a
/// flattened `DO`/`LOOP`.
///
/// **The two are one stack because a `Flow` leaving either is absorbed the
/// same way** -- `settle` reads a range off whichever frame is innermost and
/// asks `absorb`, and only what it does with an escape differs. A loop that
/// held its state in a local of the driver instead would put that state in
/// `run_ops`' own frame, which entry 66 measured at 66 instructions per pass
/// before the flat path is ever taken.
/// **The two fields the driver reads per op are in the frame itself rather
/// than behind the kind**, so `pop_if`'s check and `settle`'s range stay the
/// field loads they were before a second kind of frame existed. Measured: with
/// them behind a `match`, a body clause costs 4 instructions more even when no
/// loop is ever flattened, and a program has far more clauses than passes.
pub(crate) struct Frame {
    /// One past this frame's last op, which reaching means the frame's own
    /// range ran out.
    ///
    /// **`u32::MAX` for a loop**, because a loop's body running out is an
    /// arrival at [`super::Op::LoopNext`] rather than a position the driver
    /// tests for: the op is at the `END`, so the driver decodes it on arrival
    /// and this check must never fire for a loop.
    op_end: u32,
    /// The instruction range an escaping `Flow` is absorbed against.
    start: usize,
    end: usize,
    kind: FrameKind,
}

pub(crate) enum FrameKind {
    Select(SelectFrame),
    /// Boxed so that the stack's element stays near a `SelectFrame`'s width:
    /// a loop's state holds the header's `Number`s and is several times that.
    /// The state is on `Interp::flat_loops`, innermost last, which is the same
    /// order this stack is in -- so the loop a frame belongs to is that stack's
    /// top when the frame is the innermost one, and no index is needed.
    Loop,
}

impl Frame {
    fn select(frame: SelectFrame) -> Self {
        Frame {
            op_end: frame.op_end,
            start: frame.start,
            end: frame.end,
            kind: FrameKind::Select(frame),
        }
    }

    fn loop_pass(body_start: usize, end_index: usize) -> Self {
        Frame {
            op_end: u32::MAX,
            start: body_start,
            end: end_index,
            kind: FrameKind::Loop,
        }
    }
}

/// Which of a `SELECT`'s two kinds of branch a [`SelectFrame`] is open over.
///
/// The two leave differently, and the difference is `run_otherwise`'s own
/// split from `step`'s `Select` arm: `OTHERWISE`'s dispatch restores
/// `Interp::indent_offset` once it is over, and a matched `WHEN`'s does not.
enum Branch {
    /// A matched listed `WHEN`'s branch.
    When,
    /// The `OTHERWISE` branch.
    Otherwise,
}

/// Where a `Flow` leaves the counter, once every open frame and the range
/// itself have had their say.
enum Settled {
    /// Continue at this op.
    At(u32),
    /// Nothing absorbed it: it is this whole `run_ops` call's answer.
    Escaped(Flow),
}

/// What a promoted clause's own ops answered: where they left the program
/// counter, or the `Flow` a construct they resolved produced.
///
/// A type of its own so it can carry [`ClauseValue`]: `Interp::in_clause`
/// chooses what to root across a delivered `CALL ON` handler from its work's
/// return type. A counter position roots nothing, because a promoted clause's
/// values live in the chunk's register region, which the clause's own temps
/// frame is not the root for and does not unwind. A `Flow` does -- `Flow::Exit`
/// carries a value whose only root can be that frame -- and it answers through
/// `Flow`'s own implementation rather than a second copy of the rule.
enum RegionEnd {
    /// Continue at this op.
    At(u32),
    /// The construct this clause resolves inside itself answered this `Flow`,
    /// which the enclosing range settles.
    Flowed(Flow),
}

impl ClauseValue for RegionEnd {
    fn rooted(&self) -> Option<ObjRef> {
        match self {
            RegionEnd::At(_) => None,
            RegionEnd::Flowed(flow) => flow.rooted(),
        }
    }
}

/// The name [`Loud::op_not_driven`] reports for an op the driver's own loop
/// has no arm for.
///
/// **Out of line and `#[cold]`, so the loop's dispatch does not carry an arm
/// per undriven op.** Every op that belongs to a clause region reaches the
/// loop's wildcard, and naming it there would put a second table beside the
/// one the driven ops use -- measured, and the arms are what it costs rather
/// than the frame: dropping them took `emptyloop.rex` -2.105% and
/// `varlookup.rex` -1.087% while the frame grew.
#[cold]
fn undriven_op_name(op: &Op) -> &'static str {
    match op {
        Op::Generic { .. } => "Generic",
        Op::TraceKeyword { .. } => "TraceKeyword",
        Op::LoopHeaderValue { .. } => "LoopHeaderValue",
        Op::LoopRun { .. } => "LoopRun",
        Op::LoopNext { .. } => "LoopNext",
        Op::Clause { .. } => "Clause",
        Op::TraceClause { .. } => "TraceClause",
        Op::EvalExpr { .. } => "EvalExpr",
        Op::CallExpr { .. } => "CallExpr",
        Op::PushArg { .. } => "PushArg",
        Op::TraceArgument { .. } => "TraceArgument",
        Op::CallArgs { .. } => "CallArgs",
        Op::TraceFunction { .. } => "TraceFunction",
        Op::SelectCaseText { .. } => "SelectCaseText",
        Op::WhenTest { .. } => "WhenTest",
        Op::EndBranch => "EndBranch",
        Op::EndWhen => "EndWhen",
        Op::EnterWhen { .. } => "EnterWhen",
        Op::EnterOtherwise { .. } => "EnterOtherwise",
        Op::Const { .. } => "Const",
        Op::LoadConstant { .. } => "LoadConstant",
        Op::TraceLiteral { .. } => "TraceLiteral",
        Op::Load { .. } => "Load",
        Op::TraceRead { .. } => "TraceRead",
        Op::Arith { .. } => "Arith",
        Op::Binary { .. } => "Binary",
        Op::TraceOperator { .. } => "TraceOperator",
        Op::Prefix { .. } => "Prefix",
        Op::TracePrefix { .. } => "TracePrefix",
        Op::Store { .. } => "Store",
        Op::Say { .. } => "Say",
        Op::Signal { .. } => "Signal",
        Op::Parse { .. } => "Parse",
        Op::Return { .. } => "Return",
        Op::Queue { .. } => "Queue",
        Op::Call { .. } => "Call",
        Op::CallNamed { .. } => "CallNamed",
        Op::Message { .. } => "Message",
        Op::Expose { .. } => "Expose",
        Op::Escape { .. } => "Escape",
        Op::Jump { .. } => "Jump",
        Op::JumpUnless { .. } => "JumpUnless",
        Op::Condition { .. } => "Condition",
    }
}

/// What [`Interp::leave_ended_select_branch`] found.
enum BranchEnd {
    /// No frame ended here.
    None,
    /// A frame ended and the driver resumes at this op.
    At(u32),
    /// A frame ended and its `Flow` escaped the range being driven.
    Escaped(Flow),
}

impl Interp {
    /// One [`crate::ir::Op::PushArg`]: the register's value, or an omitted
    /// position, onto the argument stack.
    #[inline(never)]
    fn push_call_arg(&mut self, registers: FrameId, src: u16) {
        let value = if src == Op::ARG_OMITTED {
            None
        } else {
            Some(self.roots.temp_at(registers, src as usize))
        };
        self.value_buffer.push(value);
    }

    /// One [`crate::ir::Op::TraceArgument`]'s `>A>` line, its gate already
    /// answered by the caller.
    ///
    /// The indent is the calling clause's own and is read fresh, for
    /// `invoke_call`'s own measured reason: an earlier argument's callee can
    /// have moved it.
    #[inline(never)]
    fn trace_call_arg(&mut self, registers: FrameId, src: u16) {
        let indent = self.clause_state.current_value_indent;
        if src == Op::ARG_OMITTED {
            self.trace_argument(indent, b"");
            return;
        }
        let value = self.roots.temp_at(registers, src as usize);
        if let Some(rendered) = self.intermediate_text(value) {
            self.trace_argument(indent, &rendered);
        }
    }

    /// One [`crate::ir::Op::CallArgs`]: resolve the callee, then run it over
    /// the `argc` arguments its own ops left on the argument stack.
    ///
    /// **Deliberately not inlined into the driver.** See the op's arm for the
    /// measurement; what it costs to inline is paid by every op in the stream
    /// and not only by calls.
    #[inline(never)]
    #[allow(
        clippy::too_many_arguments,
        reason = "every parameter is one field of the op or the stream it stands in, and folding them into a struct would put the wide fields back in the driver's frame"
    )]
    fn run_call_args(
        &mut self,
        code: &Code<'_>,
        chunk: &Chunk,
        clause: &Instruction,
        slot: u16,
        path: super::NodePath,
        site: u16,
        argc: u16,
    ) -> Result<ObjRef, Failure> {
        let Some(mark) = self.value_buffer.len().checked_sub(argc as usize) else {
            return Err(Loud::call_op_off_its_node().into());
        };
        // **The spelling is recovered only when the site has nothing**, which
        // is the whole reason this op is cheaper than `Op::CallExpr`: a
        // resolved builtin dispatches through the row the site holds, and no
        // part of that path reads it.
        let mut spelling: &[u8] = b"";
        let resolved = match chunk.resolved_call(site) {
            Some(resolved) => {
                #[cfg(test)]
                count_call_site_hit();
                resolved
            }
            None => {
                let Some(ExprKind::Call { target, .. }) =
                    Interp::chunk_node_at(clause, slot, path).map(|node| &node.kind)
                else {
                    return Err(Loud::call_op_off_its_node().into());
                };
                let (name, search_labels) = call_target_name(code, target);
                spelling = name;
                let resolved = self.resolve_call(name, search_labels)?;
                chunk.remember_call(site, resolved);
                resolved
            }
        };
        let probe = 0u8;
        self.enter_eval_node(&raw const probe)?;
        let value = self.call_over_pushed_args(resolved, spelling, mark);
        self.depth -= 1;
        value
    }

    /// One [`crate::ir::Op::CallNamed`]: resolve the callee off the site or
    /// the instruction's own name, then run it over the `argc` arguments its
    /// own ops left on the argument stack.
    ///
    /// **Deliberately not inlined into the driver**, for the reason
    /// [`Interp::run_call_args`] gives: what it costs to inline is paid by
    /// every op in the stream and not only by calls.
    #[inline(never)]
    fn run_call_named(
        &mut self,
        clause: &Instruction,
        chunk: &Chunk,
        site: u16,
        argc: u16,
    ) -> Result<Flow, Failure> {
        let InstructionKind::Call(call) = &clause.kind else {
            return Err(Loud::call_op_off_its_node().into());
        };
        let Call::Named { name, literal, .. } = &**call else {
            return Err(Loud::call_op_off_its_node().into());
        };
        let Some(mark) = self.value_buffer.len().checked_sub(argc as usize) else {
            return Err(Loud::call_op_off_its_node().into());
        };
        // The site's own kept answer and the resolution when it has none --
        // `Op::Call`'s arm has the whole argument, and this is the same site
        // table read the same way. The name comes off the instruction here
        // rather than off a node, which is why this op needs no address where
        // `Op::CallArgs` does.
        let resolved = match chunk.resolved_call(site) {
            Some(resolved) => {
                #[cfg(test)]
                count_call_site_hit();
                resolved
            }
            None => {
                let resolved = self.resolve_call(name, !*literal)?;
                chunk.remember_call(site, resolved);
                resolved
            }
        };
        self.invoke_named_call_over_pushed_args(resolved, name, mark)
    }

    /// Closes the innermost `SELECT` branch if it runs out at `pc`.
    ///
    /// **The test [`super::Op::EndWhen`] exists to make, and the one the range
    /// boundary makes for itself.** It used to run in front of every op, which
    /// is where the answer was wanted for a branch that falls out of its own
    /// end; the op sits at exactly that position instead, and a jump to the
    /// instruction it belongs to lands on it because `Chunk::op_of` resolves
    /// to a resume point in front of the ops emitted before that instruction.
    ///
    /// `Flow::Next` rather than a carried flow, because falling out of a
    /// branch is what this closes; an escaping flow reaches `settle` instead
    /// and unwinds the frames itself.
    #[expect(
        clippy::too_many_arguments,
        reason = "the range `settle` absorbs against, and every argument is one the driver \
                  already holds"
    )]
    fn leave_ended_select_branch(
        &mut self,
        code: &Code<'_>,
        chunk: &Chunk,
        base: usize,
        pc: u32,
        start: usize,
        end: usize,
        source: Option<&ProgramSource>,
    ) -> Result<BranchEnd, Failure> {
        if self.frames.len() <= base {
            return Ok(BranchEnd::None);
        }
        let Some(frame) = self.frames.pop_if(|frame| pc >= frame.op_end) else {
            return Ok(BranchEnd::None);
        };
        let FrameKind::Select(frame) = frame.kind else {
            return Err(Loud::op_not_driven("a loop frame ran off its own end").into());
        };
        let flow = self.leave_branch(code, &frame, Flow::Next)?;
        Ok(
            match self.settle(code, chunk, base, flow, pc, start, end, source)? {
                Settled::At(target) => BranchEnd::At(target),
                Settled::Escaped(other) => BranchEnd::Escaped(other),
            },
        )
    }
}

impl Interp {
    /// Runs `chunk` for the activation on top of the stack.
    ///
    /// The register region is reserved once, here, from `chunk.registers`,
    /// and truncated away on every path out. It sits on the temporaries
    /// stack (`RootSet::reserve_temps`' own doc has why neither a slot frame
    /// of its own nor the activation's own frame works), below every
    /// watermark a clause takes, so a clause's own frame pops back to above
    /// it rather than through it.
    pub(crate) fn run_chunk(
        &mut self,
        code: &Code<'_>,
        chunk: &Chunk,
        source: Option<&ProgramSource>,
    ) -> Result<Ended, Failure> {
        #[cfg(test)]
        count_run_chunk_entry();

        let registers = self.roots.reserve_temps(chunk.registers as usize);
        // Truncated on both paths rather than only on `Ok`: the loud and
        // raised paths leave the activation for `Interp::run` and
        // `resolve_and_run_call` to tear down, and a region left behind
        // would keep its registers rooted for the rest of the run.
        let ended = self.run_chunk_clauses(code, chunk, registers, source);
        self.roots.pop_frame(registers);
        ended
    }

    /// The body of `run_chunk` past the register region, split out so the
    /// truncation above covers every way this returns.
    ///
    /// **Two levels rather than one flat loop, and the reason is the trap
    /// offer.** [`Interp::run_ops`] runs ops until one escapes its range, and
    /// is the same function `run_bounded`'s chunk arm enters for a construct's
    /// body -- so it must not offer anything to a trap, exactly as
    /// `run_bounded` does not. This level is the activation's own, which is
    /// where the offer belongs: one per activation, made by the activation
    /// that is unwinding. A nested `run_ops` (a `DO` body) shares this
    /// activation's traps and must not get a second offer.
    fn run_chunk_clauses(
        &mut self,
        code: &Code<'_>,
        chunk: &Chunk,
        registers: FrameId,
        source: Option<&ProgramSource>,
    ) -> Result<Ended, Failure> {
        let len = code.body.instructions.len();
        loop {
            // The activation's own `pc` is where a body is entered at -- `0`
            // for a program, a label's index for a `CALL`, a handler's for a
            // trap -- and `apply_flow` is what moves it afterwards.
            //
            // **The op counter below is a local, and what licenses that is a
            // property of the `pc` rather than a claim about who reads it.**
            // The tree-walker already leaves the `pc` sitting on an `IF` for
            // the whole of that `IF`'s branch and on a `DO` for the whole of
            // its loop, so a `pc` that does not name the clause currently
            // running is the behaviour every reader of it already has to
            // tolerate. This driver leaves it in exactly the same states: on
            // whichever clause `apply_flow` last routed to.
            let entry = self.activation().pc;
            if entry >= len {
                return Ok(END_OF_BODY);
            }
            let Some(at) = chunk.op_at(entry) else {
                return Err(Loud::chunk_map_too_short().into());
            };
            // `[0, len]` is the whole body, so every `Goto` a clause of it
            // produces is absorbed here and only the flows that end or
            // redirect the activation come back.
            let flow = match self.run_ops::<true>(code, chunk, registers, at, 0, len, source) {
                Ok(flow) => flow,
                // `offer_to_trap` answers `Flow::Signal` for a trap that
                // fired and `Flow::Exit` for a handler that ended the
                // program, never `Flow::Next`, so a trapped condition
                // redirects this loop rather than ending it.
                Err(failure) => self.offer_to_trap(code, failure)?,
            };
            // `Flow::Next` is `run_ops` reaching the end of its range, and
            // the range here is the whole body -- so it is the end of the
            // body itself rather than one clause finishing, and there is no
            // `pc` to advance. Everything else is a flow the activation
            // itself owns, which `apply_flow` applies exactly as
            // `run_activation`'s loop does; a `Signal` writes the `pc` that
            // the top of this loop then reads back.
            if matches!(flow, Flow::Next) {
                return Ok(END_OF_BODY);
            }
            if let Some(ended) = self.apply_flow(code, flow)? {
                return Ok(ended);
            }
        }
    }

    /// `run_bounded`'s chunk arm: the instructions of `[start, end)`, run from
    /// `chunk`'s ops.
    ///
    /// The one mapping from the instruction space a range is expressed in
    /// into the op space the loop walks, and the one guard on it. `compile`
    /// writes one entry per instruction plus a final one, so the lookup is in
    /// range for any index that indexes an instruction.
    ///
    /// `#[inline]` because this is entered once per pass of every promoted
    /// `DO` body and does one map lookup before handing over, where the
    /// tree-walker's own arm is inlined into its caller outright.
    #[inline]
    pub(crate) fn run_bounded_from_chunk(
        &mut self,
        code: &Code<'_>,
        chunk: &Chunk,
        registers: FrameId,
        start: usize,
        end: usize,
        source: Option<&ProgramSource>,
    ) -> Result<Flow, Failure> {
        let Some(at) = chunk.op_at(start) else {
            return Err(Loud::chunk_map_too_short().into());
        };
        self.run_ops::<false>(code, chunk, registers, at, start, end, source)
    }

    /// Runs `chunk`'s ops from op `at` until one of them produces a `Flow`
    /// that `[start, end]` does not absorb, and answers that `Flow`.
    ///
    /// `start` and `end` are **instruction** indices, because that is what a
    /// `Flow::Goto` carries and what a construct computes its body's bounds
    /// in; `at` and the counter are **op** indices. `Chunk::op_at` is where
    /// the two spaces meet, and `absorb` -- shared with the tree-walker's own
    /// bounded loop -- is where the rule that reads them lives.
    ///
    /// Reaching the op one past `end`'s own first op, whether by falling
    /// through or by an absorbed `Goto`, is the only way this answers
    /// `Flow::Next`; every other exit is the escaping `Flow` unchanged.
    ///
    /// `GRANTING` is whether this level is the activation's own, and so owes
    /// each clause it opens the first-instruction permission. **The
    /// tree-walker's own split, kept exactly**: `run_activation`'s loop calls
    /// `grant_procedure_permission` and `run_bounded` does not, so a clause
    /// inside a construct's body never spends the permission and a `PROCEDURE`
    /// there is 17.1. Granting at every level instead would be a `mem::take`
    /// per body clause that answers `false` every time. It is a `const`
    /// parameter rather than a value because each caller knows its own answer
    /// at the call site, which turns a per-clause branch into no code at all.
    #[expect(
        clippy::too_many_arguments,
        reason = "two callers, and every argument is a value each already holds"
    )]
    // **Bundling the four range-invariant arguments into a struct was tried and
    // rejected**, so that nobody spends the session on it again. It does what
    // it promises to `instructions:u` -- 6 fewer per range entry, 1 fewer per
    // `Op::Generic` clause -- and on `cycles:u` it moves both arms of
    // `bench-programs/emptyloop.rex` far more than that in opposite directions:
    // the tree-walker arm, which never enters this function, 8.61 to 7.96
    // billion cycles, and the compiled arm 8.63 to 9.12, taking that axis'
    // cycle ratio from 1.00 to 1.15 while its instruction count falls.
    // Reproduced across two sittings and two builds, with branch misses at
    // 70,000 out of 6.3 billion branches either way, so it is not prediction.
    // No mechanism below "the layout changed" was found, and a change with that
    // profile is not worth 7 instructions.
    fn run_ops<const GRANTING: bool>(
        &mut self,
        code: &Code<'_>,
        chunk: &Chunk,
        registers: FrameId,
        at: u32,
        start: usize,
        end: usize,
        source: Option<&ProgramSource>,
    ) -> Result<Flow, Failure> {
        // The frames below this length belong to the levels that entered
        // before this one, and every check here stops at it. **This is the
        // whole of what makes one stack safe to share**: `settle` walks down
        // to it and no further, so an escaping `Flow` meets exactly the
        // constructs this level opened, in the order it opened them.
        let base = self.frames.len();
        #[cfg(test)]
        record_frame_floor(base);
        let flow =
            self.run_ops_from::<GRANTING>(code, chunk, registers, at, start, end, source, base);
        // A `Flow` that left through `settle` has already popped every frame
        // this level opened, so this fires only where a `Failure` unwound past
        // them -- the point the `Vec` local to this call used to be dropped at.
        if self.frames.len() > base {
            self.unwind_frames(base);
        }
        flow
    }

    /// [`Interp::run_ops`]' body, with the frame stack's floor already taken.
    #[expect(
        clippy::too_many_arguments,
        reason = "one caller, and every argument is a value it already holds"
    )]
    fn run_ops_from<const GRANTING: bool>(
        &mut self,
        code: &Code<'_>,
        chunk: &Chunk,
        registers: FrameId,
        at: u32,
        start: usize,
        end: usize,
        source: Option<&ProgramSource>,
        base: usize,
    ) -> Result<Flow, Failure> {
        let Some(stop) = chunk.op_at(end) else {
            return Err(Loud::chunk_map_too_short().into());
        };
        let depth = self.activation_depth();
        // **SPIKE.** Whether the permission is still worth asking about. It is
        // granted to this activation's first instruction and has to be cleared
        // by the one after it; from there on `grant_procedure_permission` writes
        // the same `false` over and over, once per clause, and this stops it.
        let mut granting = GRANTING;
        let mut pc = at;
        'ops: loop {
            // **A branch that runs out exactly at this range's own end.**
            // `stop` is `Chunk::op_of`'s entry for `end`, which is where the
            // branch's own `Op::EndWhen` sits -- so the op is one past what
            // this range drives and the boundary owes the close itself.
            if pc >= stop {
                match self.leave_ended_select_branch(code, chunk, base, pc, start, end, source)? {
                    BranchEnd::At(target) => {
                        pc = target;
                        continue;
                    }
                    BranchEnd::Escaped(other) => return Ok(other),
                    BranchEnd::None => return Ok(Flow::Next),
                }
            }
            // **The tripwire for the op that replaced the check that used to
            // stand here.** A frame is overdue only at the position its own
            // `Op::EndWhen` occupies; anywhere else means a branch outlived
            // the op that closes it, which is a `SELECT` running on into what
            // follows it and is silent in any program whose branches happen to
            // agree.
            #[cfg(debug_assertions)]
            if !matches!(chunk.op_at_index(pc), Some(Op::EndWhen)) {
                debug_assert!(
                    !(self.frames.len() > base
                        && self.frames.last().is_some_and(|frame| pc >= frame.op_end)),
                    "the innermost SELECT frame ended before op {pc}, which is not its own EndWhen"
                );
            }
            let Some(op) = chunk.op_at_index(pc) else {
                return Err(Loud::chunk_map_too_short().into());
            };
            let (flow, next) = match op {
                // **`Generic` delegates to `step_in_temps_frame`, not to
                // `step`.** `step` is not the clause unit: its wrapper carries
                // the per-clause clock invalidation, the `>I>` trace-entry
                // decay, `current_value_indent`, the `SIGL` clause line, the
                // clause boundary through `in_clause`, the clause echo, the GC
                // temps frame with its watermark tripwire, and failure-site
                // resolution. That call is also why **no `Clause` op precedes
                // a `Generic` one**: it echoes the clause itself, the echo is
                // not idempotent, and `compile` asserts the arrangement.
                Op::Generic { index } => {
                    #[cfg(test)]
                    count_clause_op_entry();
                    let index = *index as usize;
                    let Some(instruction) = code.body.instructions.get(index) else {
                        return Err(Loud::chunk_map_too_short().into());
                    };
                    if granting {
                        self.grant_procedure_permission(instruction);
                        granting = self.procedure_permitted
                            || matches!(instruction.kind, InstructionKind::Label { .. });
                    }
                    let flow = self.step_in_temps_frame(code, index, instruction, source)?;
                    (flow, pc + 1)
                }
                Op::Clause { index, end } => {
                    #[cfg(test)]
                    count_clause_op_entry();
                    let index = *index as usize;
                    let end = *end;
                    // **The instruction this whole region names**, fetched once
                    // and read by every index-bearing op inside the region
                    // rather than each resolving its own `index` against the
                    // body, which is a bounds-checked lookup of the same
                    // instruction per op that would do it.
                    // `compile::assert_region_ops_name_their_clause` is what
                    // makes the two the same instruction by checking rather
                    // than by assuming, and [`debug_assert_names_the_clause`]
                    // is the same check per op in debug.
                    let Some(clause) = code.body.instructions.get(index) else {
                        return Err(Loud::chunk_map_too_short().into());
                    };
                    if granting {
                        self.grant_procedure_permission(clause);
                        // **A label leaves the permission alone**, so a label
                        // reading `false` here is not the permission being
                        // spent: `sub: procedure expose zg` grants at the
                        // `PROCEDURE`, one clause after the label.
                        granting = self.procedure_permitted
                            || matches!(clause.kind, InstructionKind::Label { .. });
                    }
                    // **Whether the setting in force is still the one this
                    // chunk's trace ops were emitted for**, and the whole of
                    // what makes a compiled-in emission decision safe.
                    //
                    // The widened cache key answers this on the way in:
                    // `chunk_for` is asked for the chunk of the setting in
                    // force, so a body entered under a second setting gets a
                    // second chunk rather than the first one. What the key
                    // cannot answer is a `TRACE` run *while this chunk is
                    // running*, which changes the setting with nothing
                    // consulting the cache -- and the reply is not a
                    // re-compile but a fall back to the run-time gate for the
                    // clauses that follow, which is what the tree-walker does
                    // for the same clause anyway.
                    //
                    // **Read here, per promoted clause, rather than kept in a
                    // local the loop initialises -- and that is a measurement
                    // rather than the obvious shape.** A local has to be
                    // initialised, and `bench-programs/emptyloop.rex`, whose
                    // body holds no promoted clause at all, enters `run_ops`
                    // once per pass: it has nothing for the answer to decide
                    // and would pay for it anyway. Measured with `perf stat -e
                    // instructions:u` against 40.3009 billion user
                    // instructions before this task:
                    //
                    //   * initialised at every `run_ops` entry -- 40.7259
                    //     billion, 17 per pass;
                    //   * read here -- 40.4009 billion, 4 per pass, and those
                    //     four are the extra op variant in this match rather
                    //     than the read, since this program reaches no
                    //     `Clause` op at all.
                    //
                    // A clause that does have an echo to decide pays one
                    // comparison, which is the one `in_stepped_clause` used to
                    // make for it.
                    //
                    // It is also the shape with no premise about *who* can
                    // change the setting mid-body. A local refreshed after the
                    // ops that can run a `TRACE` needs that list to be right,
                    // and the list is an internal enumeration; reading the
                    // setting where the answer is used needs nothing.
                    let stale = chunk.trace().clause_echoes() != self.chunk_trace().clause_echoes();
                    // **`stale` moves the clause echo from the stream back to
                    // the run-time gate, in both directions at once.** The
                    // region's own [`Op::TraceClause`] is skipped and
                    // [`Echo::Gated`] is passed instead, so a chunk compiled to
                    // echo under a setting that no longer does prints nothing,
                    // and one compiled silent under a setting that now echoes
                    // prints the same line the tree-walker would. The two have
                    // to be one decision: doing only the first would leave a
                    // `TRACE R` inside a body invisible to every promoted
                    // clause after it, and only the second would leave `TRACE
                    // N` unable to switch one off.
                    let echo = if stale { Echo::Gated } else { Echo::Compiled };
                    // **The clause unit, entered by its two halves rather than
                    // by its closure form**, which is what puts the region's
                    // ops in this function's own frame instead of a callee's.
                    // There is one implementation of the clause boundary --
                    // `Interp::enter_stepped_clause` and
                    // `Interp::leave_stepped_clause`, which
                    // `Interp::in_stepped_clause_with` is itself defined in
                    // terms of -- so a promoted clause and an unpromoted one
                    // discharge the same list from the same code.
                    //
                    // What the shape is worth is an IR-minus-tree-walker gap
                    // per pass, and it moves with every promotion landed after
                    // it, so it is recorded per commit in
                    // `bench-baselines/phase-4e-arms.tsv` rather than restated
                    // here -- a row keyed by a hash cannot go stale where a
                    // number written into this line can.
                    let entry = self.enter_stepped_clause(
                        echo,
                        code,
                        index,
                        clause,
                        source,
                        chunk.position_at(index),
                    );
                    // Taken on entry exactly as `step` takes it, because a
                    // promoted clause is a clause and the permission is spent
                    // by whichever clause the activation granted it to.
                    //
                    // **Load-bearing, and so is the `grant_procedure_permission`
                    // call in front of this region -- each with a witness of its
                    // own.** Both were once unobservable, on the premise that an
                    // op which grants follows every region and grants again
                    // before any `PROCEDURE` is reached. Neither half of that
                    // holds, and each fails for a different reason:
                    //
                    // * a construct whose body clauses are stepped by a nested,
                    //   *non-granting* driver entry has no later grant at all,
                    //   so without this take a `PROCEDURE` as a loop body's
                    //   first instruction is permitted. Measured: dropping it
                    //   makes `tests/ir_dual_cases/loop-header-boundaries`'
                    //   "procedure as a loop body's first instruction" row
                    //   diverge between the engines, and nothing else in the
                    //   workspace notices;
                    // * a promoted clause that *is* the activation's first
                    //   instruction has to consume `first_instruction_pending`
                    //   itself, or the next clause's grant consumes it instead
                    //   and a `PROCEDURE` behind a promoted clause is permitted.
                    //   Measured: dropping the grant makes
                    //   `tests/ir_dual_cases/assignment-and-say`'s "procedure
                    //   after an assignment in a called label" row diverge, and
                    //   again nothing else notices.
                    let _first_instruction = std::mem::take(&mut self.procedure_permitted);
                    // The ops of this promoted clause, `[pc + 1, end)`, and
                    // where they leave the counter.
                    //
                    // **A labelled block rather than a callee**, and the
                    // failure path is what makes that possible: the clause's
                    // result reaches `leave_stepped_clause` as a value, so an
                    // op that fails leaves the region carrying it rather than
                    // taking a `?` past the boundary that owes it a site.
                    //
                    // Only the ops that are part of a clause's own work appear
                    // here. An op that runs a whole clause of its own does not,
                    // and cannot: `compile` asserts no `Generic` sits inside a
                    // region, because it echoes the clause and the echo is not
                    // idempotent.
                    //
                    // The register mark the plan's Decisions section describes
                    // is a compile-time quantity -- the allocator releases to it
                    // when this region's ops were emitted -- so there is nothing
                    // to release here: the registers this region wrote are
                    // simply not addressed again.
                    let ran: Result<RegionEnd, Failure> = 'cold: {
                        // **The region answers an op index and nothing else.**
                        // Where a clause leaves the counter is the whole of what
                        // an ordinary one has to say, and carrying that in a
                        // `Result<RegionEnd, Failure>` costs a 24-byte value
                        // built and moved per clause. The two answers that do
                        // need one -- a `Flow` the enclosing range settles, and a
                        // failure -- leave through `'cold` instead, so the
                        // discriminant rides the program counter on the path
                        // every clause takes and the value exists only on the
                        // paths that have something to put in it.
                        let next: u32 = 'region: {
                            // A `DO`/`LOOP` header's values, accumulated across this
                            // region's own ops because they are not `ObjRef`s and so
                            // have no register to live in: a bound is a `Number` and
                            // a budget is a count.
                            //
                            // **`None` until an op needs one**, so a region that is
                            // not a loop header -- an `IF`'s, a `WHEN`'s, a
                            // `SELECT`'s -- pays one discriminant store rather than
                            // the struct's own initialisation.
                            let mut header: Option<LoopHeaderValues> = None;
                            let Some(ops) = chunk.ops_in(pc + 1, end) else {
                                break 'cold Err(Loud::chunk_map_too_short().into());
                            };
                            for region_op in ops {
                                match region_op {
                                    // **No gate**: this op exists only in a chunk
                                    // compiled under a setting that echoes, which is
                                    // the decision. `stale` is the one thing that
                                    // can withdraw it, and then the clause unit has
                                    // already asked the current setting instead.
                                    Op::TraceClause { index } => {
                                        debug_assert_names_the_clause(
                                            code,
                                            *index,
                                            clause,
                                            "TraceClause",
                                        );
                                        if !stale {
                                            #[cfg(test)]
                                            count_trace_op_echo();
                                            // The indent this clause's own entry
                                            // computed, read back rather than
                                            // recomputed: `enter_stepped_clause`
                                            // sets this field to `printed_indent`
                                            // for the clause it is opening and
                                            // nothing between there and here writes
                                            // it, so the two engines cannot come to
                                            // print an echo at two different indents
                                            // for one clause.
                                            let indent = self.clause_state.current_value_indent;
                                            self.echo_compiled_clause(source, clause, indent);
                                        }
                                    }
                                    // **The one thing this does that `EvalExpr`
                                    // does not is skip `resolve_call`.** The
                                    // argument loop, the `>A>` lines, the
                                    // activation bookkeeping and the three `Ended`
                                    // arms are the same functions `eval.rs` calls
                                    // on the same node; only the resolution comes
                                    // from the site instead of being made again.
                                    //
                                    // `enter_eval_node` is called because the
                                    // *arguments* go back through `eval`, and a
                                    // call that skipped it would start them one
                                    // level shallower than the tree-walker does --
                                    // see that function's own doc.
                                    Op::CallExpr {
                                        index,
                                        slot,
                                        path,
                                        site,
                                        dst,
                                    } => {
                                        debug_assert!(
                                            chunk.holds_register(*dst),
                                            "op writes register {dst} outside the region the chunk \
                                         reserved"
                                        );
                                        debug_assert_names_the_clause(
                                            code, *index, clause, "CallExpr",
                                        );
                                        // The address resolves to a node, and the
                                        // pair this op needs is that node's own.
                                        // The match is here rather than inside the
                                        // descent, which answers with an
                                        // expression so that the echo op below can
                                        // address a node of any kind.
                                        let Some(ExprKind::Call { target, args }) =
                                            Interp::chunk_node_at(clause, *slot, *path)
                                                .map(|node| &node.kind)
                                        else {
                                            break 'cold Err(Loud::call_op_off_its_node().into());
                                        };
                                        let (name, search_labels) = call_target_name(code, target);
                                        // A raise is deliberately not recorded, and
                                        // a hit needs no guard: `CallSite`'s own
                                        // doc has both reasons, and they are the
                                        // same ones `Op::Call` reads them for.
                                        let resolved = match chunk.resolved_call(*site) {
                                            Some(resolved) => {
                                                #[cfg(test)]
                                                count_call_site_hit();
                                                resolved
                                            }
                                            None => match self.resolve_call(name, search_labels) {
                                                Ok(resolved) => {
                                                    chunk.remember_call(*site, resolved);
                                                    resolved
                                                }
                                                Err(failure) => break 'cold Err(failure),
                                            },
                                        };
                                        let probe = 0u8;
                                        if let Err(failure) = self.enter_eval_node(&raw const probe)
                                        {
                                            break 'cold Err(failure);
                                        }
                                        let value =
                                            self.eval_call_resolved(code, resolved, name, args);
                                        self.depth -= 1;
                                        let value = match value {
                                            Ok(value) => value,
                                            Err(failure) => break 'cold Err(failure),
                                        };
                                        self.roots.set_temp(registers, *dst as usize, value);
                                    }
                                    // **Every one of these bodies is behind
                                    // a call.** What they do is small, but
                                    // what they *contain* is not -- a `Vec`
                                    // push carries its own growth path and an
                                    // argument echo builds a rendering -- and
                                    // inlining either into the driver's frame
                                    // is paid by every op in the stream.
                                    // Measured with the bodies written out
                                    // here, `varlookup` -- which never
                                    // executes one -- retired 5.97% more
                                    // instructions, `compound` 4.14%.
                                    Op::PushArg { src } => {
                                        self.push_call_arg(registers, *src);
                                    }
                                    Op::TraceArgument { src } => {
                                        if !self.tracing_intermediates() {
                                            continue;
                                        }
                                        self.trace_call_arg(registers, *src);
                                    }
                                    Op::CallArgs {
                                        slot,
                                        path,
                                        site,
                                        argc,
                                        dst,
                                    } => {
                                        debug_assert!(
                                            chunk.holds_register(*dst),
                                            "op writes register {dst} outside the region the chunk \
                                         reserved"
                                        );
                                        // **The body is behind a call and not
                                        // written here**, which is layout
                                        // rather than style: a call's
                                        // resolution, its argument run and its
                                        // outcome are wide, and inlining them
                                        // into the driver's own frame raises
                                        // the register pressure every *other*
                                        // op then pays. Measured: with this
                                        // body inline, `varlookup` -- which
                                        // calls nothing at all -- retired 4.9%
                                        // more instructions.
                                        match self.run_call_args(
                                            code, chunk, clause, *slot, *path, *site, *argc,
                                        ) {
                                            Ok(value) => {
                                                self.roots.set_temp(
                                                    registers,
                                                    *dst as usize,
                                                    value,
                                                );
                                            }
                                            Err(failure) => break 'cold Err(failure),
                                        }
                                    }
                                    // The `>F>` line the op above owes, emitted
                                    // behind it because `eval`'s own hook is
                                    // post-order and this is the same line.
                                    Op::TraceFunction {
                                        index,
                                        slot,
                                        path,
                                        src,
                                    } => {
                                        debug_assert_names_the_clause(
                                            code,
                                            *index,
                                            clause,
                                            "TraceFunction",
                                        );
                                        // **The gate in front of the descent, not
                                        // only inside `trace_intermediate`.** That
                                        // function returns immediately under the
                                        // same condition, so this changes no
                                        // output; what it changes is that an
                                        // untraced run walks no path to reach a
                                        // node it is not going to print.
                                        if !self.tracing_intermediates() {
                                            continue;
                                        }
                                        let Some(expr) =
                                            Interp::chunk_node_at(clause, *slot, *path)
                                        else {
                                            break 'cold Err(Loud::call_op_off_its_node().into());
                                        };
                                        let value = self.roots.temp_at(registers, *src as usize);
                                        self.trace_intermediate(code, expr, value);
                                    }
                                    Op::EvalExpr { index, slot, dst } => {
                                        debug_assert!(
                                            chunk.holds_register(*dst),
                                            "op writes register {dst} outside the region the chunk \
                                         reserved"
                                        );
                                        debug_assert_names_the_clause(
                                            code, *index, clause, "EvalExpr",
                                        );
                                        let value = match self.eval_chunk_expr(code, clause, *slot)
                                        {
                                            Ok(value) => value,
                                            Err(failure) => break 'cold Err(failure),
                                        };
                                        self.roots.set_temp(registers, *dst as usize, value);
                                    }
                                    // **The phase's first native expression op**:
                                    // the literal's value, built from the chunk's
                                    // own interned bytes through the same
                                    // `Interp::literal` that `eval_node`'s `Literal`
                                    // arm calls, with `eval.rs` not entered at all.
                                    // It emits nothing -- `Op::TraceLiteral` below
                                    // is the line that loading a literal owes.
                                    Op::Const { dst, konst } => {
                                        debug_assert!(
                                            chunk.holds_register(*dst),
                                            "op writes register {dst} outside the region the chunk \
                                         reserved"
                                        );
                                        // **Built once for the whole run.** The
                                        // value is interned into the chunk on the
                                        // first execution and read from it after
                                        // that, which is what stops this op
                                        // allocating a fresh object for a literal
                                        // longer than the handle carries inline --
                                        // 560,000 times on the pinned `rexxcps`
                                        // before this. `Chunk::interned`'s own doc
                                        // has why the shared value is safe and why
                                        // `NIL` is the empty state.
                                        let mut value = chunk.interned_konst(*konst);
                                        if value == ObjRef::NIL {
                                            let Some(bytes) = chunk.konst(*konst) else {
                                                break 'cold Err(
                                                    Loud::constant_out_of_range().into()
                                                );
                                            };
                                            #[cfg(test)]
                                            count_const_build();
                                            value = self.interned_literal(bytes);
                                            debug_assert_ne!(
                                                value,
                                                ObjRef::NIL,
                                                "a constant interned to the handle that means \
                                             'not built yet', so it would be rebuilt on every \
                                             execution and the cache would be dead code"
                                            );
                                            chunk.remember_konst(*konst, value);
                                        }
                                        self.roots.set_temp(registers, *dst as usize, value);
                                    }
                                    // A constant symbol's own value: its upcased
                                    // spelling, built through the same
                                    // `Interp::literal` `Op::Const` above uses and
                                    // read out of the symbol table `code` carries,
                                    // which is what `eval_node`'s own `Constant`
                                    // arm does. It emits nothing -- the
                                    // `Op::TraceLiteral` below is the line it owes
                                    // too, because both print `>L>`.
                                    Op::LoadConstant { symbol, dst } => {
                                        debug_assert!(
                                            chunk.holds_register(*dst),
                                            "op writes register {dst} outside the region the chunk \
                                         reserved"
                                        );
                                        // Interned exactly as `Op::Const`'s value
                                        // is, and keyed by the symbol's own id --
                                        // so the symbol table is read once for the
                                        // whole run rather than on every execution.
                                        let mut value = chunk.interned_symbol(*symbol);
                                        if value == ObjRef::NIL {
                                            #[cfg(test)]
                                            count_load_constant_build();
                                            value = self.interned_literal(
                                                code.symbols.name(*symbol).as_bytes(),
                                            );
                                            debug_assert_ne!(
                                                value,
                                                ObjRef::NIL,
                                                "a constant symbol interned to the handle that means \
                                             'not built yet', so it would be rebuilt on every \
                                             execution and the cache would be dead code"
                                            );
                                            chunk.remember_symbol(*symbol, value);
                                        }
                                        self.roots.set_temp(registers, *dst as usize, value);
                                    }
                                    // The `>L>` line of one literal. **Its own op**,
                                    // because the load emits nothing and `eval.rs`
                                    // emits this as a side effect of evaluating --
                                    // so a promoted clause with no such op drops the
                                    // line while every line after it still matches.
                                    Op::TraceLiteral { src } => {
                                        debug_assert!(
                                            chunk.holds_register(*src),
                                            "op reads register {src} outside the region the chunk \
                                         reserved"
                                        );
                                        // **The gate in front of the register read,
                                        // not only inside `echo_literal`.** That
                                        // function returns immediately under the
                                        // same condition, so this changes no output;
                                        // what it changes is that an untraced run
                                        // does not reach for a value it is not going
                                        // to print, and `temp_at` is a bounds-check
                                        // and a load per echo op in a stream that
                                        // carries one behind every literal, read
                                        // and operator. Measured over this arm and
                                        // the other value-echo arms carrying the
                                        // same gate, with the read moved behind it:
                                        // `bench-programs/varlookup.rex` 23.751 to
                                        // 21.737 billion user instructions and the
                                        // pinned `rexxcps` 11.427 to 11.060
                                        // billion.
                                        if !self.tracing_intermediates() {
                                            continue;
                                        }
                                        let value = self.roots.temp_at(registers, *src as usize);
                                        self.echo_literal(value);
                                    }
                                    // **The second native expression op**: one bare
                                    // symbol's own value, through the same
                                    // `Interp::read_symbol` that `eval_node`'s
                                    // `Variable`/`Stem`/`Compound` arms enter, with
                                    // `eval.rs` itself not entered at all. It emits
                                    // nothing -- `Op::TraceRead` below is what
                                    // reading a symbol owes.
                                    Op::Load {
                                        symbol,
                                        read,
                                        at,
                                        dst,
                                    } => {
                                        debug_assert!(
                                            chunk.holds_register(*dst),
                                            "op writes register {dst} outside the region the chunk \
                                         reserved"
                                        );
                                        let at = at.resolved();
                                        // **The tripwire for a chunk run against a
                                        // plan that is not the one it was compiled
                                        // from.** `at` was read out of that plan's
                                        // `by_symbol`, which is the map `code.slots`
                                        // is a view of, so a mismatch here is two
                                        // different plans for one body -- and it
                                        // would read somebody else's slot rather
                                        // than fail, which is a wrong value found by
                                        // chasing it.
                                        debug_assert!(
                                            at.is_none() || code.slot_for(*symbol) == at,
                                            "a compiled read names a slot this body's plan does not \
                                         give its symbol"
                                        );
                                        // **The simple read is resolved here rather
                                        // than through `read_symbol`, and that is a
                                        // measurement.** A simple read is a slot
                                        // index; reaching it through the shared
                                        // function costs an out-of-line call, a
                                        // match on `SymbolRead`, a tuple return and
                                        // a `Result` return, per read. Measured with
                                        // the marginal method -- a body run at N and
                                        // 2N iterations, differenced -- `z = a` costs
                                        // 479 user instructions per execution through
                                        // `read_symbol` and 431 resolved here, while
                                        // `z = 1` is unmoved at 367, which is the
                                        // control saying the change reached the read
                                        // and nothing else.
                                        //
                                        // The arm is `read_symbol`'s own
                                        // `SymbolRead::Simple` arm, and the other
                                        // kinds still go there: a stem allocates on a
                                        // miss and a compound resolves a tail key,
                                        // neither of which belongs in a driver.
                                        let value = match read {
                                            SymbolRead::Simple => {
                                                let (value, novalue) =
                                                    self.read_at(code, *symbol, at);
                                                if let Err(failure) = self.novalue_check(novalue) {
                                                    break 'cold Err(failure);
                                                }
                                                value
                                            }
                                            SymbolRead::Stem | SymbolRead::Compound => {
                                                match self.read_symbol(code, *read, *symbol, at) {
                                                    Ok(value) => value,
                                                    Err(failure) => break 'cold Err(failure),
                                                }
                                            }
                                        };
                                        self.roots.set_temp(registers, *dst as usize, value);
                                    }
                                    // The `>V>` line one read owes, and the `>C>`
                                    // line in front of it when the read is a
                                    // compound. **Its own op**, because the load
                                    // emits nothing and `eval.rs` emits these as a
                                    // side effect of evaluating -- so a promoted
                                    // clause with no such op drops them while every
                                    // line after them still matches.
                                    Op::TraceRead { symbol, src, .. } => {
                                        debug_assert!(
                                            chunk.holds_register(*src),
                                            "op reads register {src} outside the region the chunk \
                                         reserved"
                                        );
                                        // The gate in front of the register read,
                                        // for `Op::TraceLiteral`'s own reason.
                                        if !self.tracing_intermediates() {
                                            continue;
                                        }
                                        let value = self.roots.temp_at(registers, *src as usize);
                                        self.echo_symbol_read(code, *symbol, value);
                                    }
                                    // **A native expression op**: one arithmetic
                                    // operator applied to two registers, through
                                    // the same `Interp::arith_small_int` and
                                    // `Interp::arith_general` that
                                    // `eval_arithmetic` enters, with `eval.rs`
                                    // itself not entered at all -- for the operands
                                    // either, which is what the ops in front of
                                    // this one are. It emits nothing --
                                    // `Op::TraceOperator` below is what applying an
                                    // operator owes.
                                    Op::Arith {
                                        op,
                                        hint,
                                        lhs,
                                        rhs,
                                        dst,
                                    } => {
                                        debug_assert!(
                                            chunk.holds_register(*lhs)
                                                && chunk.holds_register(*rhs),
                                            "op reads registers {lhs}/{rhs} outside the region the \
                                         chunk reserved"
                                        );
                                        debug_assert!(
                                            chunk.holds_register(*dst),
                                            "op writes register {dst} outside the region the chunk \
                                         reserved"
                                        );
                                        // **Both read before either is written**,
                                        // which is what makes `lhs == dst` -- the
                                        // shape a chain compiles to -- safe.
                                        let left = self.roots.temp_at(registers, *lhs as usize);
                                        let right = self.roots.temp_at(registers, *rhs as usize);
                                        // The operands are rooted by the registers
                                        // they came from, which is what
                                        // `arith_general` requires of a caller and
                                        // is why no frame is pushed here where
                                        // `eval_arithmetic` pushes one: it has to
                                        // root values held in Rust locals across
                                        // the evaluation of the operand after them,
                                        // and this op's operands were rooted before
                                        // it ran.
                                        //
                                        // **The hint decides only which path is
                                        // tried first, and removes no check.** The
                                        // small-integer path re-decodes both
                                        // operands and re-checks them against
                                        // `DIGITS` on every execution -- Rexx
                                        // rounds the operands before it operates,
                                        // so an operand too wide for the precision
                                        // makes the exact answer the wrong one --
                                        // and answers `None` for every case where
                                        // the two paths could disagree. Both are
                                        // therefore correct for every operand, and
                                        // what the hint can change is speed alone.
                                        let mut quick = None;
                                        if chunk.tries_small_int(*hint) {
                                            quick = self.arith_small_int(*op, left, right);
                                            // The one state change a site makes,
                                            // and it makes it at most once: a site
                                            // that has fallen through skips the
                                            // attempt from here on, so the store
                                            // never repeats and the line the table
                                            // sits on is not dirtied again.
                                            if quick.is_none() {
                                                chunk.saw_general(*hint);
                                            }
                                        } else {
                                            #[cfg(test)]
                                            count_arith_hint_skip();
                                        }
                                        let value = match quick {
                                            Some(value) => value,
                                            None => match self.arith_general(*op, left, right) {
                                                Ok(value) => value,
                                                Err(failure) => break 'cold Err(failure),
                                            },
                                        };
                                        self.roots.set_temp(registers, *dst as usize, value);
                                    }
                                    // **Every other binary operator**: one
                                    // concatenation, comparison or logical
                                    // operator applied to two registers, through
                                    // the same `Interp::apply_binary` that
                                    // `eval_node`'s own binary arm enters, with
                                    // `eval.rs` itself not entered at all -- for
                                    // the operands either, which is what the ops in
                                    // front of this one are. It emits nothing --
                                    // `Op::TraceOperator` below is what applying an
                                    // operator owes.
                                    Op::Binary { op, lhs, rhs, dst } => {
                                        debug_assert!(
                                            chunk.holds_register(*lhs)
                                                && chunk.holds_register(*rhs),
                                            "op reads registers {lhs}/{rhs} outside the region the \
                                         chunk reserved"
                                        );
                                        debug_assert!(
                                            chunk.holds_register(*dst),
                                            "op writes register {dst} outside the region the chunk \
                                         reserved"
                                        );
                                        // **Both read before either is written**,
                                        // which is what makes `lhs == dst` -- the
                                        // shape a chain compiles to -- safe.
                                        let left = self.roots.temp_at(registers, *lhs as usize);
                                        let right = self.roots.temp_at(registers, *rhs as usize);
                                        // The operands are rooted by the registers
                                        // they came from, which is what
                                        // `apply_binary` requires of a caller and
                                        // is why no frame is pushed here where
                                        // `eval_node`'s own arm pushes one: it has
                                        // to root values held in Rust locals across
                                        // the evaluation of the operand after them,
                                        // and this op's operands were rooted before
                                        // it ran.
                                        let value = match self.apply_binary(*op, left, right) {
                                            Ok(value) => value,
                                            Err(failure) => break 'cold Err(failure),
                                        };
                                        self.roots.set_temp(registers, *dst as usize, value);
                                    }
                                    // The `>O>` line one operator owes. **Its own
                                    // op**, because the operation emits nothing and
                                    // `eval.rs` emits this as a side effect of
                                    // evaluating -- so a promoted clause with no
                                    // such op drops the line while every line after
                                    // it still matches.
                                    Op::TraceOperator { op, src } => {
                                        debug_assert!(
                                            chunk.holds_register(*src),
                                            "op reads register {src} outside the region the chunk \
                                         reserved"
                                        );
                                        // The gate in front of the register read,
                                        // for `Op::TraceLiteral`'s own reason.
                                        if !self.tracing_intermediates() {
                                            continue;
                                        }
                                        let value = self.roots.temp_at(registers, *src as usize);
                                        self.echo_operator(*op, value);
                                    }
                                    // **A prefix operator**: `+`, `-` or `\`
                                    // applied to one register, through the same
                                    // `Interp::apply_prefix` that
                                    // `Interp::eval_prefix` enters, with `eval.rs`
                                    // itself not entered at all -- for the operand
                                    // either, which is what the ops in front of
                                    // this one are. It emits nothing --
                                    // `Op::TracePrefix` below is what applying an
                                    // operator owes.
                                    Op::Prefix { op, src, dst } => {
                                        debug_assert!(
                                            chunk.holds_register(*src),
                                            "op reads register {src} outside the region the chunk \
                                         reserved"
                                        );
                                        debug_assert!(
                                            chunk.holds_register(*dst),
                                            "op writes register {dst} outside the region the chunk \
                                         reserved"
                                        );
                                        // **Read before the destination is
                                        // written**, which is what makes `src ==
                                        // dst` -- the shape a prefix compiles to --
                                        // safe.
                                        let value = self.roots.temp_at(registers, *src as usize);
                                        // The operand is rooted by the register it
                                        // came from, which is what `apply_prefix`
                                        // requires of a caller and is why no frame
                                        // is pushed here where `eval_prefix` pushes
                                        // one: it has to root a value held in a
                                        // Rust local across the operator's own
                                        // allocation, and this op's operand was
                                        // rooted before it ran.
                                        let value = match self.apply_prefix(*op, value) {
                                            Ok(value) => value,
                                            Err(failure) => break 'cold Err(failure),
                                        };
                                        self.roots.set_temp(registers, *dst as usize, value);
                                    }
                                    // The `>P>` line one prefix operator owes.
                                    // **Its own op**, for the reason
                                    // `Op::TraceOperator` above is, and a different
                                    // op from it because `>P>` is a different line
                                    // from `>O>`.
                                    Op::TracePrefix { op, src } => {
                                        debug_assert!(
                                            chunk.holds_register(*src),
                                            "op reads register {src} outside the region the chunk \
                                         reserved"
                                        );
                                        // The gate in front of the register read,
                                        // for `Op::TraceLiteral`'s own reason.
                                        if !self.tracing_intermediates() {
                                            continue;
                                        }
                                        let value = self.roots.temp_at(registers, *src as usize);
                                        self.echo_prefix_op(*op, value);
                                    }
                                    // The write, through `Interp::assign_evaluated`
                                    // -- the whole of what `step`'s own
                                    // `Assignment` arm does past the evaluation, so
                                    // the `>>>`/`>C>`/`>=>` lines and the stem and
                                    // compound dispatch are that arm's rather than a
                                    // second copy.
                                    Op::Store { index, at, src } => {
                                        debug_assert!(
                                            chunk.holds_register(*src),
                                            "op reads register {src} outside the region the chunk \
                                         reserved"
                                        );
                                        debug_assert_names_the_clause(
                                            code, *index, clause, "Store",
                                        );
                                        let InstructionKind::Assignment { target, .. } =
                                            &clause.kind
                                        else {
                                            break 'cold Err(Loud::store_op_off_its_node().into());
                                        };
                                        let at = at.resolved();
                                        // `Op::Load`'s own tripwire, on the writing
                                        // side: `at` came out of the plan this chunk
                                        // was compiled from, `code.slots` is a view
                                        // of that same map, and a mismatch is two
                                        // different plans for one body -- which would
                                        // write into somebody else's slot rather than
                                        // fail.
                                        debug_assert!(
                                            at.is_none()
                                                || matches!(
                                                    &target.kind,
                                                    ExprKind::Variable(id)
                                                        if code.slot_for(*id) == at
                                                ),
                                            "a compiled write names a slot this body's plan does not \
                                         give its target"
                                        );
                                        let value = self.roots.temp_at(registers, *src as usize);
                                        // **A simple target with a resolved slot
                                        // is a slot write**, taken here rather
                                        // than through `assign_evaluated`, which
                                        // roots the value, materialises the
                                        // `Option` its `>>>` line would want, reads
                                        // the clause indent, and then matches the
                                        // target's shape to reach the same store.
                                        // The cost is that path, not rendering:
                                        // `result_text` and `trace_result` both
                                        // gate on `trace_mode().results` themselves,
                                        // so an untraced run renders nothing.
                                        //
                                        // The gate is `results` because it is the
                                        // weaker of the two lines the long path
                                        // emits: `>>>` is gated on `results` and
                                        // `>=>` on `intermediates`, and `results`
                                        // is true wherever `intermediates` is
                                        // (`Interp::assign_evaluated` makes the
                                        // same argument for the same reason), so a
                                        // run that would print neither is exactly
                                        // `!results`.
                                        //
                                        // No `push_temp` here, unlike the long
                                        // path: the value is read out of a
                                        // register and written into a slot, both
                                        // of which are roots, and nothing between
                                        // them allocates. `the_l0_subset_passes_
                                        // again_under_collect_on_every_allocation`
                                        // is what would find that wrong.
                                        //
                                        // Measured with the marginal method, a
                                        // body run at N and 2N iterations and
                                        // differenced: `z = 1` costs 368 user
                                        // instructions per execution through
                                        // `assign_evaluated` and 275 here, `z = a`
                                        // 431 and 338.
                                        if let Some(slot) = at
                                            && matches!(target.kind, ExprKind::Variable(_))
                                            && !self.trace_mode().results
                                        {
                                            let frame = self.activation().frame;
                                            self.set_variable(frame, slot, value);
                                        } else if let Err(failure) =
                                            self.assign_evaluated(code, target, value, at)
                                        {
                                            break 'cold Err(failure);
                                        }
                                    }
                                    // The print, through `Interp::say_evaluated`,
                                    // for the same reason `Op::Store` goes through
                                    // `assign_evaluated`.
                                    // **`SIGNAL`, all three forms**, each doing
                                    // what `step`'s own arm does and nothing else:
                                    // the two transferring forms answer
                                    // `Flow::Signal` and end the region, the trap
                                    // form edits the activation's table and falls
                                    // through with whatever `exec_condition_trap`
                                    // answers.
                                    Op::Signal { index, src } => {
                                        debug_assert_names_the_clause(
                                            code, *index, clause, "Signal",
                                        );
                                        let InstructionKind::Signal(signal) = &clause.kind else {
                                            break 'cold Err(Loud::instruction(&clause.kind).into());
                                        };
                                        let flow = match (&**signal, src) {
                                            (rexx_parse::Signal::Label(name), None) => {
                                                match self.signal_to_label(name) {
                                                    Ok(flow) => flow,
                                                    Err(failure) => break 'cold Err(failure),
                                                }
                                            }
                                            (rexx_parse::Signal::Value(_), Some(register)) => {
                                                debug_assert!(
                                                    chunk.holds_register(*register),
                                                    "op reads register {register} outside the region                                                  the chunk reserved"
                                                );
                                                let value = self
                                                    .roots
                                                    .temp_at(registers, *register as usize);
                                                match self.signal_to_value(value) {
                                                    Ok(flow) => flow,
                                                    Err(failure) => break 'cold Err(failure),
                                                }
                                            }
                                            (rexx_parse::Signal::Trap(trap), None) => {
                                                match self.exec_condition_trap(trap, false) {
                                                    Ok(flow) => flow,
                                                    Err(failure) => break 'cold Err(failure),
                                                }
                                            }
                                            // A form whose operand does not match
                                            // the op's own: loud rather than a
                                            // guess about which of the two is right.
                                            _ => {
                                                break 'cold Err(
                                                    Loud::signal_op_off_its_node().into()
                                                );
                                            }
                                        };
                                        break 'cold Ok(RegionEnd::Flowed(flow));
                                    }
                                    // `PARSE`/`ARG`/`PULL`. The whole instruction is
                                    // `exec_parse`, which both engines enter; what
                                    // this op adds is the source expression already
                                    // evaluated, for the one source that has one.
                                    Op::Parse { index, src } => {
                                        debug_assert_names_the_clause(
                                            code, *index, clause, "Parse",
                                        );
                                        let (InstructionKind::Parse(parse)
                                        | InstructionKind::Arg(parse)
                                        | InstructionKind::Pull(parse)) = &clause.kind
                                        else {
                                            break 'cold Err(Loud::instruction(&clause.kind).into());
                                        };
                                        let evaluated = src.map(|register| {
                                        debug_assert!(
                                            chunk.holds_register(register),
                                            "op reads register {register} outside the region the \
                                             chunk reserved"
                                        );
                                        self.roots.temp_at(registers, register as usize)
                                    });
                                        if let Err(failure) =
                                            self.exec_parse(code, parse, evaluated)
                                        {
                                            break 'cold Err(failure);
                                        }
                                        break 'cold Ok(RegionEnd::Flowed(Flow::Next));
                                    }
                                    Op::Say { index, src } => {
                                        debug_assert_names_the_clause(code, *index, clause, "Say");
                                        debug_assert!(
                                            matches!(
                                                &clause.kind,
                                                InstructionKind::Say { expression }
                                                    if expression.is_some() == src.is_some()
                                            ),
                                            "a Say op names an instruction that is not a SAY of \
                                         matching arity"
                                        );
                                        let value = src.map(|register| {
                                        debug_assert!(
                                            chunk.holds_register(register),
                                            "op reads register {register} outside the region the \
                                             chunk reserved"
                                        );
                                        self.roots.temp_at(registers, register as usize)
                                    });
                                        if let Err(failure) = self.say_evaluated(value) {
                                            break 'cold Err(failure);
                                        }
                                    }
                                    // The end of the activation, through
                                    // `Interp::returned_value`, for the same
                                    // reason `Op::Say` goes through
                                    // `say_evaluated`: the `>>>` line, the root
                                    // and the choice between `Flow::Return` and
                                    // `Flow::Exit` are that function's rather than
                                    // a second copy.
                                    //
                                    // **The region ends here**, carrying the
                                    // `Flow` out to `settle` exactly as
                                    // `Op::Call`'s arm does with the `Flow` a call
                                    // answers. The value leaves this clause inside
                                    // that `Flow`, so what roots it across the
                                    // boundary is `RegionEnd`'s own `ClauseValue`,
                                    // which forwards to `ClauseValue for Flow` --
                                    // the same rule `step`'s arm reaches through
                                    // `in_clause`, rather than a second copy.
                                    Op::Return {
                                        index,
                                        src,
                                        keyword,
                                    } => {
                                        debug_assert_names_the_clause(
                                            code, *index, clause, "Return",
                                        );
                                        debug_assert!(
                                            matches!(
                                                &clause.kind,
                                                InstructionKind::Return { expression }
                                                    | InstructionKind::Exit { expression }
                                                    if expression.is_some() == src.is_some()
                                            ),
                                            "a Return op names an instruction that is not a RETURN or \
                                         an EXIT of matching arity"
                                        );
                                        debug_assert!(
                                            matches!(
                                                (&clause.kind, keyword),
                                                (
                                                    InstructionKind::Return { .. },
                                                    ReturnKeyword::Return
                                                ) | (
                                                    InstructionKind::Exit { .. },
                                                    ReturnKeyword::Exit
                                                )
                                            ),
                                            "a Return op's keyword names the other half of the pair \
                                         from the clause it ends"
                                        );
                                        let value = src.map(|register| {
                                        debug_assert!(
                                            chunk.holds_register(register),
                                            "op reads register {register} outside the region the \
                                             chunk reserved"
                                        );
                                        self.roots.temp_at(registers, register as usize)
                                    });
                                        let flow = self.returned_value(value, *keyword);
                                        break 'cold Ok(RegionEnd::Flowed(flow));
                                    }
                                    // The queue write and its `>>>`, through
                                    // `Interp::queue_evaluated`. This one does not
                                    // end the region: a `PUSH` and a `QUEUE`
                                    // answer `Flow::Next`, so the region ends
                                    // where a `SAY`'s does.
                                    Op::Queue {
                                        index,
                                        src,
                                        keyword,
                                    } => {
                                        debug_assert_names_the_clause(
                                            code, *index, clause, "Queue",
                                        );
                                        debug_assert!(
                                            matches!(
                                                &clause.kind,
                                                InstructionKind::Push { expression }
                                                    | InstructionKind::Queue { expression }
                                                    if expression.is_some() == src.is_some()
                                            ),
                                            "a Queue op names an instruction that is not a PUSH or a \
                                         QUEUE of matching arity"
                                        );
                                        debug_assert!(
                                            matches!(
                                                (&clause.kind, keyword),
                                                (InstructionKind::Push { .. }, QueueKeyword::Push)
                                                    | (
                                                        InstructionKind::Queue { .. },
                                                        QueueKeyword::Queue
                                                    )
                                            ),
                                            "a Queue op's keyword names the other half of the pair \
                                         from the clause it writes for"
                                        );
                                        let value = src.map(|register| {
                                        debug_assert!(
                                            chunk.holds_register(register),
                                            "op reads register {register} outside the region the \
                                             chunk reserved"
                                        );
                                        self.roots.temp_at(registers, register as usize)
                                    });
                                        if let Err(failure) = self.queue_evaluated(value, *keyword)
                                        {
                                            break 'cold Err(failure);
                                        }
                                    }
                                    // The call, through the same
                                    // `Interp::resolve_call` and
                                    // `Interp::invoke_named_call` that `step`'s own
                                    // `Call` arm reaches -- so the resolution
                                    // order, the argument loop with its `>A>`
                                    // lines, the depth guard, the level state saved
                                    // around the nested activation and the `RESULT`
                                    // settle are that arm's rather than a second
                                    // copy. This op emits nothing itself and owes
                                    // no echo op, because it took no line away from
                                    // `eval.rs`: `Op::Call`'s own doc comment has
                                    // the argument and the measurement behind it.
                                    Op::Call { index, site } => {
                                        debug_assert_names_the_clause(code, *index, clause, "Call");
                                        let InstructionKind::Call(call) = &clause.kind else {
                                            break 'cold Err(Loud::call_op_off_its_node().into());
                                        };
                                        let Call::Named {
                                            name,
                                            literal,
                                            args,
                                        } = &**call
                                        else {
                                            break 'cold Err(Loud::call_op_off_its_node().into());
                                        };
                                        // **The site's own kept answer, and the
                                        // resolution when it has none.** Nothing
                                        // invalidates one -- `CallSite`'s own doc
                                        // comment has the reason per resolution
                                        // step -- so a hit needs no guard and there
                                        // is none. A raise is deliberately not
                                        // recorded: `resolve_call` answers `Err`
                                        // for a name that matched nothing, and a
                                        // site that raised asks again.
                                        let resolved = match chunk.resolved_call(*site) {
                                            Some(resolved) => {
                                                #[cfg(test)]
                                                count_call_site_hit();
                                                resolved
                                            }
                                            // A failure is held until the
                                            // arguments have run, which on
                                            // this op they have not --
                                            // `Op::CallNamed` took every
                                            // clause whose arguments compile,
                                            // so what is left here evaluates
                                            // them inside `invoke_call`.
                                            // `Interp::resolved_after_
                                            // arguments` has the citation.
                                            None => {
                                                let resolution = self.resolve_call(name, !*literal);
                                                match self.resolved_after_arguments(
                                                    code, resolution, args,
                                                ) {
                                                    Ok(resolved) => {
                                                        chunk.remember_call(*site, resolved);
                                                        resolved
                                                    }
                                                    Err(failure) => break 'cold Err(failure),
                                                }
                                            }
                                        };
                                        let flow = match self
                                            .invoke_named_call(code, resolved, name, args)
                                        {
                                            Ok(flow) => flow,
                                            Err(failure) => break 'cold Err(failure),
                                        };
                                        break 'cold Ok(RegionEnd::Flowed(flow));
                                    }
                                    // The same clause with its arguments
                                    // already computed by ops of this region.
                                    // The body is behind a call for the reason
                                    // `Op::CallArgs`'s arm gives.
                                    Op::CallNamed { index, site, argc } => {
                                        debug_assert_names_the_clause(
                                            code,
                                            *index,
                                            clause,
                                            "CallNamed",
                                        );
                                        match self.run_call_named(clause, chunk, *site, *argc) {
                                            Ok(flow) => break 'cold Ok(RegionEnd::Flowed(flow)),
                                            Err(failure) => break 'cold Err(failure),
                                        }
                                    }
                                    // A message send that is a whole clause,
                                    // whichever form it was written in.
                                    // `exec_message` is the tree-walker's own arm,
                                    // entered here with the fields it reads: the
                                    // term, and the message-assignment form's
                                    // value.
                                    Op::Message { index } => {
                                        debug_assert_names_the_clause(
                                            code, *index, clause, "Message",
                                        );
                                        let InstructionKind::Message { term, value } = &clause.kind
                                        else {
                                            break 'cold Err(Loud::instruction(&clause.kind).into());
                                        };
                                        let flow =
                                            match self.exec_message(code, term, value.as_ref()) {
                                                Ok(flow) => flow,
                                                Err(failure) => break 'cold Err(failure),
                                            };
                                        break 'cold Ok(RegionEnd::Flowed(flow));
                                    }
                                    // `EXPOSE`. `exec_expose` is the tree-walker's
                                    // own arm, entered here with the one field it
                                    // reads.
                                    Op::Expose { index } => {
                                        debug_assert_names_the_clause(
                                            code, *index, clause, "Expose",
                                        );
                                        let InstructionKind::Expose { variables } = &clause.kind
                                        else {
                                            break 'cold Err(Loud::instruction(&clause.kind).into());
                                        };
                                        if let Err(failure) = self.exec_expose(code, variables) {
                                            break 'cold Err(failure);
                                        }
                                        break 'cold Ok(RegionEnd::Flowed(Flow::Next));
                                    }
                                    // `LEAVE`/`ITERATE`. `leave_origin` is the
                                    // tree-walker's own capture, entered here with
                                    // the clause this region has open; which `Flow`
                                    // carries it is the only thing the two keywords
                                    // differ in, and the clause is what says which.
                                    Op::Escape { index: at } => {
                                        debug_assert_names_the_clause(code, *at, clause, "Escape");
                                        let origin = Box::new(
                                            self.leave_origin(code, index, source, clause),
                                        );
                                        let flow = match clause.kind {
                                            InstructionKind::Leave { name } => {
                                                Flow::Leave(name, origin)
                                            }
                                            InstructionKind::Iterate { name } => {
                                                Flow::Iterate(name, origin)
                                            }
                                            _ => {
                                                break 'cold Err(
                                                    Loud::instruction(&clause.kind).into()
                                                );
                                            }
                                        };
                                        break 'cold Ok(RegionEnd::Flowed(flow));
                                    }
                                    // An `IF`'s or a plain `WHEN`'s condition
                                    // validation and its `>>>` line, through the
                                    // same `Interp::condition_value` the
                                    // tree-walker's own `eval_condition` reaches
                                    // -- so the trace, the temps frame, the
                                    // readback and the raiser are that function's
                                    // rather than a second copy. One arm for both
                                    // keywords, because the raiser is the only
                                    // thing that differs and the op carries it.
                                    // The register is read and written in place:
                                    // what a jump tests is the logical value of
                                    // the answer, and the unvalidated value has no
                                    // reader left.
                                    Op::Condition {
                                        index,
                                        reg,
                                        keyword,
                                    } => {
                                        debug_assert!(
                                            chunk.holds_register(*reg),
                                            "op reads register {reg} outside the region the chunk \
                                         reserved"
                                        );
                                        debug_assert_names_the_clause(
                                            code,
                                            *index,
                                            clause,
                                            "Condition",
                                        );
                                        debug_assert!(
                                            matches!(
                                                (&clause.kind, keyword),
                                                (InstructionKind::If { .. }, ConditionKeyword::If)
                                                    | (
                                                        InstructionKind::When { .. },
                                                        ConditionKeyword::When
                                                    )
                                            ),
                                            "a Condition op's keyword does not name the clause whose \
                                         condition it is validating"
                                        );
                                        let value = self.roots.temp_at(registers, *reg as usize);
                                        // Read live rather than compiled in, for
                                        // the reason `eval_if_condition` reads it
                                        // live: a nested activation moves it.
                                        let indent = self.clause_state.current_value_indent;
                                        // **A value that is already a logical
                                        // needs neither a frame nor the general
                                        // test.** `condition_value` opens a temps
                                        // frame and roots the value, which its own
                                        // doc explains is for `WHILE`/`UNTIL`: they
                                        // re-test once per pass inside the
                                        // enclosing `DO`'s single frame. An
                                        // `IF`/`WHEN` tests once, and its value is
                                        // already rooted in the register it came
                                        // from.
                                        //
                                        // Both forms a condition arrives in are
                                        // taken: the small int this op writes back,
                                        // and the inline `"1"`/`"0"` a comparison
                                        // answers with (`eval.rs` builds those with
                                        // `self.text`). Anything else -- a heap
                                        // string, a number, a value that is not a
                                        // logical at all -- falls through to the
                                        // general path, which is also the only path
                                        // that can raise, since nothing reaching
                                        // the quick arms can fail its own test.
                                        //
                                        // Gated on `results` because that is what
                                        // `condition_value` prints its `>>>` under.
                                        //
                                        // Measured with the marginal method, a body
                                        // run at N and 2N iterations and
                                        // differenced: `if a = b then nop` costs 756
                                        // user instructions per execution through
                                        // the general path and 707 here, `if 1 then
                                        // nop` 682 and 634.
                                        // **Two handles, because a logical reaches
                                        // here two ways.** A comparison answers with
                                        // the inline `"1"`/`"0"` `crate::eval::
                                        // logical` builds, which is a constant and
                                        // so compares as an integer; a source
                                        // literal `1` is inlined as a tagged small
                                        // int instead (`Interp::literal`), which the
                                        // decode below is for. Measured, taking only
                                        // the constants cost `if 1 then nop` 62 user
                                        // instructions per execution -- literals
                                        // fall through to the general path without
                                        // the second arm.
                                        let quick = if value == crate::eval::LOGICAL_TRUE {
                                            Some(true)
                                        } else if value == crate::eval::LOGICAL_FALSE {
                                            Some(false)
                                        } else {
                                            match value.decode() {
                                                rexx_core::Decoded::SmallInt(1) => Some(true),
                                                rexx_core::Decoded::SmallInt(0) => Some(false),
                                                _ => None,
                                            }
                                        };
                                        let holds = match quick {
                                            Some(holds) if !self.trace_mode().results => holds,
                                            _ => match self.condition_value(
                                                value,
                                                ConditionTrace::Result(indent),
                                                false,
                                                keyword.raiser(),
                                            ) {
                                                Ok(holds) => holds,
                                                Err(failure) => break 'cold Err(failure),
                                            },
                                        };
                                        // In range unconditionally: `SMALL_INT_MAX`
                                        // is far above one.
                                        let logical = ObjRef::small_int(i64::from(holds))
                                            .unwrap_or(ObjRef::NIL);
                                        self.roots.set_temp(registers, *reg as usize, logical);
                                    }
                                    Op::JumpUnless { reg, target } => {
                                        debug_assert!(
                                            chunk.holds_register(*reg),
                                            "op reads register {reg} outside the region the chunk \
                                         reserved"
                                        );
                                        match self.register_holds(registers, *reg) {
                                            Ok(true) => {}
                                            Ok(false) => break 'region *target,
                                            Err(failure) => break 'cold Err(failure),
                                        }
                                    }
                                    Op::WhenTest { index, case, dst } => {
                                        debug_assert!(
                                            chunk.holds_register(*dst),
                                            "op writes register {dst} outside the region the chunk \
                                         reserved"
                                        );
                                        let case_text = match case {
                                            Some(register) => {
                                                debug_assert!(
                                                    chunk.holds_register(*register),
                                                    "op reads register {register} outside the region \
                                                 the chunk reserved"
                                                );
                                                let value = self
                                                    .roots
                                                    .temp_at(registers, *register as usize);
                                                Some(self.to_text(value).to_vec())
                                            }
                                            None => None,
                                        };
                                        debug_assert_names_the_clause(
                                            code, *index, clause, "WhenTest",
                                        );
                                        let holds = match self.scan_when(
                                            code,
                                            clause,
                                            case_text.as_deref(),
                                        ) {
                                            Ok(holds) => holds,
                                            Err(failure) => break 'cold Err(failure),
                                        };
                                        // In range unconditionally: `SMALL_INT_MAX`
                                        // is far above one. Stored as the logical
                                        // value `Op::JumpUnless` reads back, exactly
                                        // as an `IF`'s condition is.
                                        let value = ObjRef::small_int(i64::from(holds))
                                            .unwrap_or(ObjRef::NIL);
                                        self.roots.set_temp(registers, *dst as usize, value);
                                    }
                                    // The `>K>` line of one header value. **The
                                    // emission is its own op**, which is what lets
                                    // the stream reproduce the order the oracle
                                    // evaluates and echoes a loop header in:
                                    // evaluate `TO`, echo it, evaluate `BY`, echo
                                    // it.
                                    Op::TraceKeyword { role, src } => {
                                        debug_assert!(
                                            chunk.holds_register(*src),
                                            "op reads register {src} outside the region the chunk \
                                         reserved"
                                        );
                                        let value = self.roots.temp_at(registers, *src as usize);
                                        self.echo_header_value(*role, value);
                                    }
                                    // One header value's own validation, in front of
                                    // the next value's evaluation because that order
                                    // is observable (`Op::LoopHeaderValue`'s own doc
                                    // comment).
                                    Op::LoopHeaderValue { role, src } => {
                                        debug_assert!(
                                            chunk.holds_register(*src),
                                            "op reads register {src} outside the region the chunk \
                                         reserved"
                                        );
                                        let value = self.roots.temp_at(registers, *src as usize);
                                        let values =
                                            header.get_or_insert_with(LoopHeaderValues::default);
                                        if let Err(failure) =
                                            self.accept_header_value(*role, value, values)
                                        {
                                            break 'cold Err(failure);
                                        }
                                    }
                                    // The construct itself, from the values the ops
                                    // above filed. `run_loop_with_header` is the
                                    // same function the tree-walker reaches, and
                                    // `BodyEngine::Chunk` is the one thing this call
                                    // says that the tree-walker's does not: the
                                    // body's clauses come from this chunk.
                                    Op::LoopRun { index } => {
                                        debug_assert_names_the_clause(
                                            code, *index, clause, "LoopRun",
                                        );
                                        let index = *index as usize;
                                        let (InstructionKind::Do(body)
                                        | InstructionKind::Loop(body)) = &clause.kind
                                        else {
                                            break 'cold Err(Loud::loop_op_off_its_node().into());
                                        };
                                        let values = header.take().unwrap_or_default();
                                        // **SPIKE.** Flattened when this is a
                                        // shape the spike drives: the frame goes
                                        // on the stack, the region ends, and the
                                        // counter falls into the body's first op,
                                        // which is the op after this region.
                                        let values = match self.flat_loop_start(
                                            code, index, clause, body, source, values, end,
                                        ) {
                                            Ok(crate::run::FlatStart::Flat {
                                                body_start,
                                                end_index,
                                            }) => {
                                                self.frames
                                                    .push(Frame::loop_pass(body_start, end_index));
                                                break 'region end;
                                            }
                                            Ok(crate::run::FlatStart::Ended(flow)) => {
                                                break 'cold Ok(RegionEnd::Flowed(flow));
                                            }
                                            Ok(crate::run::FlatStart::Fallback(values)) => values,
                                            Err(failure) => break 'cold Err(failure),
                                        };
                                        let flow = match self.run_loop_with_header(
                                            code,
                                            index,
                                            clause,
                                            body,
                                            source,
                                            BodyEngine::Chunk { chunk, registers },
                                            values,
                                        ) {
                                            Ok(flow) => flow,
                                            Err(failure) => break 'cold Err(failure),
                                        };
                                        break 'cold Ok(RegionEnd::Flowed(flow));
                                    }
                                    Op::Jump { target } => {
                                        break 'region *target;
                                    }
                                    Op::Generic { .. } => {
                                        break 'cold Err(Loud::op_not_driven("Generic").into());
                                    }
                                    Op::LoopNext { .. } => {
                                        break 'cold Err(Loud::op_not_driven("LoopNext").into());
                                    }
                                    Op::Clause { .. } => {
                                        break 'cold Err(Loud::op_not_driven("Clause").into());
                                    }
                                    Op::SelectCaseText { .. } => {
                                        break 'cold Err(
                                            Loud::op_not_driven("SelectCaseText").into()
                                        );
                                    }
                                    Op::EnterWhen { .. } => {
                                        break 'cold Err(Loud::op_not_driven("EnterWhen").into());
                                    }
                                    Op::EnterOtherwise { .. } => {
                                        break 'cold Err(
                                            Loud::op_not_driven("EnterOtherwise").into()
                                        );
                                    }
                                    Op::EndBranch => {
                                        break 'cold Err(Loud::op_not_driven("EndBranch").into());
                                    }
                                    Op::EndWhen => {
                                        break 'cold Err(Loud::op_not_driven("EndWhen").into());
                                    }
                                }
                            }
                            end
                        };
                        // **The hot exit, and the whole point of the split.**
                        // `leave_clause`'s own fast path is this same question,
                        // so asking it here reaches the same answer without
                        // building the value that answer would travel in.
                        // `finish_plain_clause` discharges what is left of the
                        // boundary.
                        if self.pending_traps.is_empty() {
                            self.finish_plain_clause(entry);
                            pc = next;
                            continue 'ops;
                        }
                        Ok(RegionEnd::At(next))
                    };
                    match self.leave_stepped_clause(entry, code, index, clause, source, ran)? {
                        ClauseOutcome::Ran(region) => match region? {
                            // A promoted clause produces no `Flow` of its own:
                            // where it leaves the counter *is* its answer, which
                            // is what a jump op is for. Only its boundary can
                            // end the activation, and that is the `Ended` below.
                            RegionEnd::At(next) => {
                                pc = next;
                                continue;
                            }
                            // Settled against this range from the op past the
                            // region, which is where an absorbed `Flow::Next`
                            // continues -- the same position `pc + 1` is for an
                            // op that runs one clause and no more.
                            RegionEnd::Flowed(flow) => (flow, end),
                        },
                        ClauseOutcome::Ended(exit) => (Flow::Exit(exit.value()), pc),
                    }
                }
                // **A jump past this range's end is loud rather than a
                // stop.** `absorb` cannot check it -- a jump target is an op
                // index and absorption is decided in instruction space -- so
                // without this the loop's own `pc < stop` reads an escaping
                // jump as "the range completed" and answers `Flow::Next`,
                // which is a construct silently finishing where it should have
                // propagated. Landing exactly on `stop` *is* completion, which
                // is what a branch-end jump at a range boundary does, so the
                // comparison is strict. A backward jump out of the range is
                // not checked and is not emitted: it would re-run ops inside
                // the range, which is a wrong answer rather than a silent one,
                // and checking it costs a second `op_at` on the hot path.
                Op::Jump { target } => {
                    if *target > stop {
                        return Err(Loud::jump_out_of_range().into());
                    }
                    pc = *target;
                    continue;
                }
                // Handing an absorbed `WHEN CASE` the text it compares against
                // (`Op::SelectCaseText`'s own doc has why it is here rather
                // than inside the header's clause region), and opening a frame
                // over a branch. None of the three runs a clause or produces a
                // `Flow`, so each continues straight to the next op.
                Op::SelectCaseText { index, case } => {
                    let value = case.map(|register| {
                        debug_assert!(
                            chunk.holds_register(register),
                            "op reads register {register} outside the region the chunk reserved"
                        );
                        self.roots.temp_at(registers, register as usize)
                    });
                    debug_assert!(
                        code.body.instructions.get(*index as usize).is_some(),
                        "a SelectCaseText op names an instruction outside its own body"
                    );
                    self.open_select_case(value);
                    pc += 1;
                    continue;
                }
                // The boundary the tree-walker's own wrapper around an
                // `IF`'s whole arm runs, which a flattened construct has no
                // wrapper to run. `Interp::end_promoted_branch`'s doc comment
                // has the program that says it is not a spare one.
                Op::EndBranch => (self.end_promoted_branch(code, Flow::Next)?, pc + 1),
                Op::EndWhen => {
                    match self
                        .leave_ended_select_branch(code, chunk, base, pc, start, end, source)?
                    {
                        BranchEnd::At(target) => {
                            pc = target;
                            continue;
                        }
                        BranchEnd::Escaped(other) => return Ok(other),
                        // The scan's own landing place when the branch this
                        // ends was never entered.
                        BranchEnd::None => (Flow::Next, pc + 1),
                    }
                }
                Op::EnterWhen { select, when } => {
                    let frame = self.when_frame(code, chunk, *select as usize, *when as usize)?;
                    self.frames.push(Frame::select(frame));
                    pc += 1;
                    continue;
                }
                Op::EnterOtherwise { select } => {
                    let frame = self.otherwise_frame(code, chunk, *select as usize)?;
                    self.frames.push(Frame::select(frame));
                    pc += 1;
                    continue;
                }
                // **SPIKE.** The bottom of a flattened pass, reached by the
                // body falling out of its last clause into the `END`'s own op.
                Op::LoopNext { index } => {
                    if self.frames.len() <= base
                        || !matches!(self.frames.last().map(|f| &f.kind), Some(FrameKind::Loop))
                    {
                        return Err(Loud::op_not_driven("LoopNext").into());
                    }
                    debug_assert_eq!(
                        self.flat_loops.last().map(|flat| flat.do_index),
                        Some(*index as usize),
                        "a LoopNext op ended a pass of a loop other than the one it names"
                    );
                    match self.flat_loop_step_top(code, source, Flow::Next)? {
                        crate::run::FlatStep::Body(op_body) => {
                            pc = op_body;
                            continue;
                        }
                        crate::run::FlatStep::Done(flow) => {
                            self.frames.pop();
                            (flow, pc)
                        }
                    }
                }
                // The ops below are only meaningful inside a `Clause` region,
                // which the `Op::Clause` arm above walks: reaching one here
                // means a jump landed in the middle of a region rather than on
                // its `Clause`. Loud rather than a panic, which is this crate's
                // standing rule for a state the type system admits and the
                // compiler does not produce.
                // Every op that belongs inside a clause region, which this
                // loop does not drive -- `undriven_op_name` has why they are
                // one arm rather than one each.
                _ => return Err(Loud::op_not_driven(undriven_op_name(op)).into()),
            };
            // **Per clause, not per escaping flow.** A clause that left the
            // activation stack changed makes this loop's `code` describe a
            // frame it is no longer running, and every op after it is stepped
            // against the wrong body -- so the check has to sit where a clause
            // finishes rather than where control leaves this function.
            debug_assert_eq!(
                self.activation_depth(),
                depth,
                "a clause left the activation stack changed, so this loop's `code` and its `pc` \
                 no longer describe the same frame"
            );
            match self.settle(code, chunk, base, flow, next, start, end, source)? {
                Settled::At(target) => pc = target,
                Settled::Escaped(other) => return Ok(other),
            }
        }
    }

    /// Drops every frame this level opened, and the loop state each open loop
    /// frame stands for.
    ///
    /// **The `Failure` path, and the reason the two stacks are unwound
    /// together.** A `FrameKind::Loop` frame and an entry on
    /// `Interp::flat_loops` are one construct held in two places -- the frame
    /// carries what an escaping `Flow` is absorbed against, the entry carries
    /// the header's own state -- so a raise that discards one without the
    /// other leaves the next `Op::LoopNext` reading a loop that is not its own.
    /// The boxes go back to `flat_spares`, which is where a loop that ended
    /// normally puts them.
    #[cold]
    #[inline(never)]
    fn unwind_frames(&mut self, base: usize) {
        while self.frames.len() > base {
            let Some(frame) = self.frames.pop() else {
                return;
            };
            if matches!(frame.kind, FrameKind::Loop)
                && let Some(flat) = self.flat_loops.pop()
            {
                self.flat_spares.push(flat);
            }
        }
    }

    /// Where `flow` leaves the counter, once every open frame and then the
    /// range itself have had their say.
    ///
    /// **The flattening of the tree-walker's own nesting.** There, a `Flow`
    /// leaving a matched `WHEN`'s branch is absorbed against that branch's
    /// range by `run_bounded`, then handed to `leave_select`,
    /// and whatever comes back is absorbed against the enclosing range by
    /// whichever loop called it -- one round per Rust call frame. Here the
    /// rounds are a loop over the frame stack, in the same order and through
    /// the same two functions.
    ///
    /// `next` is where an unabsorbed `Flow::Next` continues, which is the op
    /// after whichever one produced it.
    ///
    /// **`inline(always)`, and it is a measurement rather than a habit.** This
    /// is one call per clause of every promoted body, where the driver's loop
    /// otherwise decides a `Flow` inline. Left to the inliner's judgement it is
    /// emitted as a function, and `bench-programs/emptyloop.rex` -- a loop
    /// whose body does nothing, so the measurement is the per-clause cost and
    /// almost nothing else -- runs 3.38-3.40s on the compiled stream against
    /// 2.84-2.87s without the frame stack at all. With the annotation it is
    /// 2.85-2.87s, which is that same level. Interleaved between arms within
    /// one sitting, three sittings.
    #[expect(
        clippy::too_many_arguments,
        reason = "two callers inside one loop, and every argument is a value that loop holds"
    )]
    #[inline(always)]
    fn settle(
        &mut self,
        code: &Code<'_>,
        chunk: &Chunk,
        base: usize,
        mut flow: Flow,
        next: u32,
        start: usize,
        end: usize,
        source: Option<&ProgramSource>,
    ) -> Result<Settled, Failure> {
        // **SPIKE.** `Flow::Next` settles at `next` whatever is open: `absorb`
        // answers `Advance` for it against every range, and every arm below
        // turns `Advance` into `At(next)`. So the walk over the open frames is
        // skipped for the one flow that every clause of every body produces,
        // which is what makes a frame affordable to hold open across a loop's
        // whole body rather than only across a `WHEN`'s branch.
        if matches!(flow, Flow::Next) {
            return Ok(Settled::At(next));
        }
        loop {
            // Copied out rather than held, because the `Escaped` arm pops the
            // frame this came from. **`base` and not emptiness** is what ends
            // the walk: the frames below it are another level's, and this one's
            // own range has the next say.
            let Some((frame_start, frame_end)) = (self.frames.len() > base)
                .then(|| self.frames.last().map(|f| (f.start, f.end)))
                .flatten()
            else {
                return Ok(match absorb(flow, start, end) {
                    Absorbed::Advance => Settled::At(next),
                    Absorbed::Resume(target) => Settled::At(op_at(chunk, target)?),
                    Absorbed::Escaped(other) => Settled::Escaped(other),
                });
            };
            match absorb(flow, frame_start, frame_end) {
                Absorbed::Advance => return Ok(Settled::At(next)),
                // `target` can be the branch's own `end`, which is one past its
                // last instruction: `op_at` answers this frame's `op_end`
                // there, and the loop's own check closes the frame on the next
                // pass rather than a second rule doing it here.
                Absorbed::Resume(target) => return Ok(Settled::At(op_at(chunk, target)?)),
                Absorbed::Escaped(other) => {
                    match self
                        .frames
                        .pop()
                        .expect("the check above just observed one")
                        .kind
                    {
                        FrameKind::Select(frame) => {
                            flow = self.leave_branch(code, &frame, other)?;
                        }
                        // **SPIKE.** A `LEAVE`/`ITERATE` that reached this
                        // loop, decided by the same `do_body_outcome` the
                        // nested form hands the answering `Flow` to. An
                        // `ITERATE` this loop consumes puts the frame back and
                        // resumes at the body's first op -- the same place
                        // `Op::LoopNext` resumes a pass that fell through.
                        FrameKind::Loop => match self.flat_loop_step_top(code, source, other)? {
                            crate::run::FlatStep::Body(op_body) => {
                                self.frames.push(Frame {
                                    op_end: u32::MAX,
                                    start: frame_start,
                                    end: frame_end,
                                    kind: FrameKind::Loop,
                                });
                                return Ok(Settled::At(op_body));
                            }
                            crate::run::FlatStep::Done(escape) => flow = escape,
                        },
                    }
                }
            }
        }
    }

    /// What a `SELECT` does with a `Flow` that left one of its branches:
    /// `select_escape` and `leave_select`, exactly what `step`'s own `Select`
    /// arm and `run_otherwise` do with the same `Flow`, and then the boundary
    /// the tree-walker's own wrapper around the whole arm runs.
    ///
    /// **`select_escape` decides one thing here that it also decides there,
    /// and one thing it does not have to.** Where control goes is answered by
    /// the op layout either way -- [`Op::EnterOtherwise`] sits at the
    /// `OTHERWISE` marker's entry in `Chunk::op_of`, so every arrival there
    /// opens the branch's frame, the scan running out of `WHEN`s and an
    /// absorbed `WHEN CASE`'s escape alike. What the layout cannot answer is
    /// whether the *construct* has finished: a redirect into `OTHERWISE` is
    /// one `SELECT` still running, so it owes no end-of-branch boundary yet,
    /// where the tree-walker gets that for free by not having returned from
    /// its own `step` call. Answering it here is what keeps the two engines to
    /// one boundary per construct.
    ///
    /// [`Op::EnterOtherwise`]: super::Op::EnterOtherwise
    fn leave_branch(
        &mut self,
        code: &Code<'_>,
        frame: &SelectFrame,
        flow: Flow,
    ) -> Result<Flow, Failure> {
        let flow = match frame.branch {
            Branch::When => match select_escape(frame.otherwise, flow) {
                // Still inside this `SELECT`: `EnterOtherwise` at the target's
                // own entry opens the next branch's frame, and this `WHEN`'s
                // branch did not finish -- control was redirected out of it,
                // so the end-of-branch boundary below is not owed.
                SelectEscape::Otherwise(target) => return Ok(Flow::Goto(target)),
                SelectEscape::Forward(flow) => flow,
            },
            // `Interp::leave_otherwise` is the whole of leaving this branch,
            // shared with `run_otherwise`: the escape elevation is restored
            // and `leave_select` decides where control goes. **And no
            // end-of-branch boundary follows it**, because `OTHERWISE`'s
            // branch ends at the `END`, a real instruction with a boundary of
            // its own -- measured, a handler queued by the last clause of an
            // `OTHERWISE` and re-queued by its own handler is delivered at the
            // `END`'s line, not at the branch's.
            Branch::Otherwise => {
                return self.leave_otherwise(
                    code,
                    frame.select,
                    frame.label,
                    frame.end,
                    frame.select_end,
                    flow,
                );
            }
        };
        let flow = self.leave_select(code, frame.select, frame.label, frame.resume, flow)?;
        // A matched `WHEN`'s branch is one the oracle closes with a synthetic
        // instruction, so it owes that instruction's boundary.
        self.end_promoted_branch(code, flow)
    }

    /// The frame [`Op::EnterWhen`] opens: the matched branch of the listed
    /// `WHEN` at `when`, belonging to the `SELECT` at `select`.
    ///
    /// Every bound comes from the nodes themselves through the same
    /// [`when_targets`] and [`select_parts`] the tree-walker reads, so the two
    /// engines cannot come to disagree about where a branch ends or which
    /// label leaves it.
    fn when_frame(
        &self,
        code: &Code<'_>,
        chunk: &Chunk,
        select: usize,
        when: usize,
    ) -> Result<SelectFrame, Failure> {
        let (Some(select_instruction), Some(when_instruction)) = (
            code.body.instructions.get(select),
            code.body.instructions.get(when),
        ) else {
            return Err(Loud::chunk_map_too_short().into());
        };
        let Some(parts) = select_parts(&select_instruction.kind) else {
            return Err(Loud::select_op_off_its_node().into());
        };
        let targets = when_targets(&when_instruction.kind, code.body.instructions.len());
        Ok(SelectFrame {
            select,
            label: parts.label,
            otherwise: parts.otherwise,
            select_end: parts.end,
            start: when + 1,
            end: targets.body_end,
            resume: when_resume(&targets),
            op_end: op_at(chunk, targets.body_end)?,
            branch: Branch::When,
        })
    }

    /// The frame [`Op::EnterOtherwise`] opens: the `OTHERWISE` branch of the
    /// `SELECT` at `select`.
    ///
    /// The range **starts at the marker**, not past it, which is
    /// `run_otherwise`'s own range: the marker is an ordinary clause, stepped
    /// by the same unit as everything else in the branch.
    fn otherwise_frame(
        &self,
        code: &Code<'_>,
        chunk: &Chunk,
        select: usize,
    ) -> Result<SelectFrame, Failure> {
        let Some(select_instruction) = code.body.instructions.get(select) else {
            return Err(Loud::chunk_map_too_short().into());
        };
        let Some(parts) = select_parts(&select_instruction.kind) else {
            return Err(Loud::select_op_off_its_node().into());
        };
        let Some(otherwise) = parts.otherwise else {
            return Err(Loud::select_op_off_its_node().into());
        };
        let otherwise_end = otherwise_range(code.body.instructions.len(), parts.end);
        Ok(SelectFrame {
            select,
            label: parts.label,
            otherwise: parts.otherwise,
            select_end: parts.end,
            start: otherwise,
            end: otherwise_end,
            resume: otherwise_resume(code.body.instructions.len(), parts.end),
            op_end: op_at(chunk, otherwise_end)?,
            branch: Branch::Otherwise,
        })
    }

    /// Whether register `reg` holds the Rexx logical value `1`.
    ///
    /// The op that decided the branch has already written its answer here, as
    /// the small integer of the `bool` it computed, so this is a readback
    /// rather than a second check --
    /// re-deriving the answer from the value's text would be a second
    /// implementation of the rule that decides a branch. Anything else in the
    /// register means the two ops came apart, which is loud rather than a
    /// silently-taken branch.
    fn register_holds(&self, registers: FrameId, reg: u16) -> Result<bool, Failure> {
        let value = self.roots.temp_at(registers, reg as usize);
        // The two handles a logical arrives in, compared as integers: the
        // constant a comparison answers with, and the small int
        // `Op::Condition` writes back for everything else.
        if value == crate::eval::LOGICAL_TRUE {
            return Ok(true);
        }
        if value == crate::eval::LOGICAL_FALSE {
            return Ok(false);
        }
        match value.decode() {
            Decoded::SmallInt(1) => Ok(true),
            Decoded::SmallInt(0) => Ok(false),
            _ => Err(Loud::register_not_logical().into()),
        }
    }
}

/// Asserts, in debug, that the op naming instruction `index` from inside a
/// clause region names that region's own clause.
///
/// **What licenses [`Interp::run_ops`]' own `Op::Clause` arm reading the
/// instruction off the region instead of looking each op's `index` up.** Every
/// index-bearing op `compile` emits inside a region is emitted from the arm of
/// the instruction whose region it is, so the two are the same instruction by
/// construction -- `compile::assert_region_ops_name_their_clause` is that
/// stated as a check on the emitted stream rather than as a sentence about the
/// emitting code, and this is the run-time half for a stream that reached the
/// driver some other way.
fn debug_assert_names_the_clause(code: &Code<'_>, index: u32, clause: &Instruction, op: &str) {
    debug_assert_eq!(
        code.body
            .instructions
            .get(index as usize)
            .map(std::ptr::from_ref),
        Some(std::ptr::from_ref(clause)),
        "a {op} op names an instruction that is not the clause of the region it sits in"
    );
}

/// The op instruction `target` resumes at, or the loud failure a chunk whose
/// map is shorter than its own body earns.
///
/// `compile` writes one entry per instruction plus a final one, so this is in
/// range for every index a construct computes -- including one past the last
/// instruction, which is what a branch's own `end` is.
fn op_at(chunk: &Chunk, target: usize) -> Result<u32, Failure> {
    chunk
        .op_at(target)
        .ok_or_else(|| Loud::chunk_map_too_short().into())
}

// Test-only instrumentation: how many chunks this thread has driven.
//
// The engine-selection tests need this to tell "the IR engine ran this body"
// apart from "the run produced the answer the tree-walker also produces".
// Both engines resolve every construct through the same functions, so they
// agree on every program by construction and no observable output tells them
// apart -- a selection test resting on output alone would pass with selection
// deleted.
//
// **Per thread, not per process, and the difference is what keeps a delta
// meaningful when the default engine is not the tree-walker.** A test reading
// a process-wide count measures every program any concurrently running test
// happens to drive, and no lock between the reading tests can exclude that,
// because the contamination comes from tests that never take it. A thread's
// own count is contaminated by nothing, since `libtest` gives each test a
// thread and the interpreter runs on whichever thread entered it. That is
// what obliges the tests to enter through `execute` rather than
// `run_program`, which spawns a thread of its own: the counter would then be
// incremented on a thread no test can read.
#[cfg(test)]
thread_local! {
    static RUN_CHUNK_ENTRIES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
fn count_run_chunk_entry() {
    RUN_CHUNK_ENTRIES.with(|entries| entries.set(entries.get() + 1));
}

#[cfg(test)]
pub(crate) fn run_chunk_entries() -> usize {
    RUN_CHUNK_ENTRIES.with(std::cell::Cell::get)
}

// Test-only instrumentation: how many clauses this thread has stepped from a
// compiled stream.
//
// `run_chunk_entries` counts *activations* driven, which cannot see the one
// thing promoting a construct changes: whether a clause **inside** that
// construct reaches the stream at all. Both engines produce identical bytes
// for every program by construction here, and an activation entered is one
// entry either way, so this is the only observable that separates a body
// driven from the chunk from a body driven straight into the tree-walker.
//
// Counted where a clause *begins*, which is every op that opens one: a
// `Generic` and a promoted `Clause` region. So a construct that moves from one
// of those shapes to another does not change the count, and a construct whose
// clauses stop reaching the stream does.
//
// Per thread for the reason `RUN_CHUNK_ENTRIES` is: see its own comment.
#[cfg(test)]
thread_local! {
    static CLAUSE_OP_ENTRIES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
fn count_clause_op_entry() {
    CLAUSE_OP_ENTRIES.with(|entries| entries.set(entries.get() + 1));
}

#[cfg(test)]
pub(crate) fn clause_op_entries() -> usize {
    CLAUSE_OP_ENTRIES.with(std::cell::Cell::get)
}

// Test-only instrumentation: how many clause echoes this thread has emitted
// from a chunk's own `Op::TraceClause`.
//
// **This is the only observable that says the compiled emission decision is
// reached in production at all**, and without it the whole mechanism has no
// witness. Two separate ways to make `Op::TraceClause` dead leave every
// output-comparing test in the workspace green, because both are covered by
// the run-time gate the driver falls back to and that gate prints the same
// bytes: asking `chunk_for` for a chunk under a setting that is not the one in
// force, and answering `stale` yes for every clause. Output cannot tell an
// echo emitted by an op from the identical echo emitted by the clause unit;
// this counts which one did it.
//
// Counted where the echo is *emitted*, not where the op is fetched, because
// the second of those two mutations leaves the op in the stream and skips its
// work -- a count of arrivals would stay green on it.
//
// Per thread for the reason `RUN_CHUNK_ENTRIES` is: see its own comment.
#[cfg(test)]
thread_local! {
    static TRACE_OP_ECHOES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
fn count_trace_op_echo() {
    TRACE_OP_ECHOES.with(|echoes| echoes.set(echoes.get() + 1));
}

#[cfg(test)]
pub(crate) fn trace_op_echoes() -> usize {
    TRACE_OP_ECHOES.with(std::cell::Cell::get)
}

// Test-only instrumentation: how many times this thread has skipped the
// small-integer path because a site's hint said it had already fallen through.
//
// **The only observable that says the patch table is read in production at
// all.** Both paths answer identically for every operand -- that is what makes
// a hint safe -- so no program's output can tell which one ran, and a table
// that was never consulted would leave every output-comparing test in the
// workspace green. Deleting the table is not even a behaviour change: every
// site then tries the small-integer path and falls through, which is what the
// op does with no table at all.
//
// Counted at the skip rather than at the load, because that is the branch the
// table exists to take: a count of loads would stay green on a table whose
// state never changed.
//
// Per thread for the reason `RUN_CHUNK_ENTRIES` is: see its own comment.
#[cfg(test)]
thread_local! {
    static ARITH_HINT_SKIPS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
fn count_arith_hint_skip() {
    ARITH_HINT_SKIPS.with(|skips| skips.set(skips.get() + 1));
}

#[cfg(test)]
pub(crate) fn arith_hint_skips() -> usize {
    ARITH_HINT_SKIPS.with(std::cell::Cell::get)
}

// Test-only instrumentation: how many times this thread has run a compiled call
// from the resolution its site had already kept.
//
// **The only observable that says the resolution table is read in production at
// all**, and it is `ARITH_HINT_SKIPS`' argument one op over: a kept resolution
// and a fresh one are the same answer -- that is what makes keeping it safe --
// so no program's output can tell which one ran, and a table nothing consulted
// would leave every output-comparing test in the workspace green.
//
// Counted at the hit rather than at the store, because a count of stores would
// stay green on a table that is written and never read.
//
// Per thread for the reason `RUN_CHUNK_ENTRIES` is: see its own comment.
#[cfg(test)]
thread_local! {
    static CALL_SITE_HITS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
fn count_call_site_hit() {
    CALL_SITE_HITS.with(|hits| hits.set(hits.get() + 1));
}

#[cfg(test)]
pub(crate) fn call_site_hits() -> usize {
    CALL_SITE_HITS.with(std::cell::Cell::get)
}

// Test-only instrumentation: how many times an [`super::Op::Const`] on this
// thread has had to build its value rather than read an interned one.
//
// **One counter per op rather than one for both**, because the two do not
// occur independently in a program: a loop header's own bounds are constant
// symbols, so a counted `Op::Const` test would be counting the header's
// `Op::LoadConstant`s as well and its number would be about the header.
//
// **Output cannot see this and no other instrument in this crate can either.**
// A constant built fresh on every execution and a constant built once produce
// the same bytes, the same trace and the same exit status, by construction --
// which is what makes the interning safe and also what leaves it unpinned. The
// count is the only observable, and the property it states is the one worth
// having: this is per *distinct* constant reached, not per execution.
//
// Counted where the value is built, so a cache that was written and never read
// would show every execution here.
//
// Per thread for the reason `RUN_CHUNK_ENTRIES` is: see its own comment.
#[cfg(test)]
thread_local! {
    static CONST_BUILDS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
fn count_const_build() {
    CONST_BUILDS.with(|builds| builds.set(builds.get() + 1));
}

#[cfg(test)]
pub(crate) fn const_builds() -> usize {
    CONST_BUILDS.with(std::cell::Cell::get)
}

// The same for [`super::Op::LoadConstant`]; see `CONST_BUILDS` for why the two
// are counted apart.
#[cfg(test)]
thread_local! {
    static LOAD_CONSTANT_BUILDS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
fn count_load_constant_build() {
    LOAD_CONSTANT_BUILDS.with(|builds| builds.set(builds.get() + 1));
}

#[cfg(test)]
pub(crate) fn load_constant_builds() -> usize {
    LOAD_CONSTANT_BUILDS.with(std::cell::Cell::get)
}

// Test-only instrumentation: the deepest frame stack any [`Interp::run_ops`]
// entry on this thread has found already open.
//
// **The observable a shared frame stack owes and a local `Vec` did not.** A
// level that leaves frames behind cannot corrupt the level above it -- `base`
// is what stops the walk, so the leftovers are inert -- and the interpreter
// goes on printing the right bytes while the stack grows for as long as the
// program runs. Output cannot see that, and neither can an exit code; what
// sees it is the floor the next entry finds. Measured on a program that traps
// out of two open loops 50,000 times: 46,108 kB of peak resident memory with
// `unwind_frames` deleted against 4,792 kB with it.
//
// Recorded at entry rather than at exit because that is where a leftover
// becomes another level's problem, and as a maximum because a leak shows up as
// a floor that climbs rather than as one bad entry.
//
// Per thread for the reason `RUN_CHUNK_ENTRIES` is: see its own comment.
#[cfg(test)]
thread_local! {
    static FRAME_FLOOR_HIGH_WATER: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
fn record_frame_floor(base: usize) {
    FRAME_FLOOR_HIGH_WATER.with(|floor| floor.set(floor.get().max(base)));
}

#[cfg(test)]
pub(crate) fn frame_floor_high_water() -> usize {
    FRAME_FLOOR_HIGH_WATER.with(std::cell::Cell::get)
}

#[cfg(test)]
mod tests;
