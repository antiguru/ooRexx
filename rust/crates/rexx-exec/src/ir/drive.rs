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
//! `grant_procedure_permission`, `in_stepped_clause`, `offer_to_trap`,
//! `apply_flow` and `absorb`.

use rexx_core::{Decoded, FrameId, ObjRef};
use rexx_parse::{ProgramSource, SymbolId};

use super::{BodyEngine, Chunk, Op};
use crate::clause::{ClauseOutcome, ClauseValue};
use crate::run::{
    Absorbed, Ended, Flow, SelectResume, absorb, otherwise_range, select_exit, select_parts,
    when_targets,
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
/// **Local to one [`Interp::run_ops`] call, not on `Interp`.** A `Failure`
/// unwinds out of that call with frames still open and the `Vec` is simply
/// dropped, exactly as the Rust frames it replaces are; a stack on `Interp`
/// would need every raise to remember to unwind it. A `DO` body inside a
/// branch enters `run_ops` again through `run_bounded_from_chunk` and gets a
/// stack of its own, which is right: a `LEAVE` there meets the loop first and
/// this frame afterwards, in that order, exactly as the nested calls give.
struct SelectFrame {
    /// The `SELECT` instruction this branch belongs to, which is the position
    /// `pop_search_frame` resets a forwarded `LEAVE`'s indent to.
    select: usize,
    /// `SELECT LABEL name`'s own label.
    label: Option<SymbolId>,
    /// The branch's own instruction range -- the range the tree-walker bounds
    /// the identical `run_bounded` call to, and the one an escaping `Flow` is
    /// absorbed against here.
    start: usize,
    end: usize,
    /// Where `leave_select` resumes this branch, which is two answers rather
    /// than one ([`SelectResume`]).
    resume: SelectResume,
    /// One past the branch's last op. **Reaching it is the branch running off
    /// its own end**, which is the arrival the tree-walker gets as
    /// `run_bounded` answering `Flow::Next` -- and it is an op position rather
    /// than an instruction one because that is the space the counter walks.
    op_end: u32,
    /// Which branch this is, which decides how it is left.
    branch: Branch,
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

/// Where a promoted clause left the program counter.
///
/// A newtype so it can carry [`ClauseValue`]: `Interp::in_stepped_clause`
/// chooses what to root across a delivered `CALL ON` handler from its work's
/// return type, and this answers `None` because a promoted clause's values
/// live in the chunk's register region, which the clause's own temps frame is
/// not the root for and does not unwind.
struct ClauseNext(u32);

impl ClauseValue for ClauseNext {
    fn rooted(&self) -> Option<ObjRef> {
        None
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
        reason = "two callers, and every argument is a value each already holds: bundling them \
                  into a struct would only move the same list one line up"
    )]
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
        let Some(stop) = chunk.op_at(end) else {
            return Err(Loud::chunk_map_too_short().into());
        };
        let depth = self.activations.len();
        // The constructs this level has open. Empty for every body until a
        // `SELECT` opens a branch, so the two comparisons per op below are a
        // `Vec::last` on an empty `Vec` for everything else.
        let mut frames: Vec<SelectFrame> = Vec::new();
        let mut pc = at;
        loop {
            // **A branch whose ops the counter has left has run off its own
            // end**, and that is the arrival the tree-walker gets as
            // `run_bounded` answering `Flow::Next`. Checked before the op is
            // fetched, because the op at `op_end` belongs to whatever follows
            // the branch and must not run until the branch has been left.
            if let Some(frame) = frames.pop_if(|frame| pc >= frame.op_end) {
                let flow = self.leave_branch(code, &frame, Flow::Next)?;
                match self.settle(code, chunk, &mut frames, flow, pc, start, end)? {
                    Settled::At(target) => pc = target,
                    Settled::Escaped(other) => return Ok(other),
                }
                continue;
            }
            if pc >= stop {
                return Ok(Flow::Next);
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
                    if GRANTING {
                        self.grant_procedure_permission(instruction);
                    }
                    let flow = self.step_in_temps_frame(code, index, instruction, source)?;
                    (flow, pc + 1)
                }
                // **The clause wrapper is the same one `Generic` takes**, and
                // the whole of the difference is the `BodyEngine` it carries:
                // the construct is resolved by `run_loop`, exactly as the
                // tree-walker resolves it, and the engine decides only how
                // each of its body's clauses is stepped. Writing a second loop
                // here instead is the defect the dual-engine sweep exists to
                // catch.
                Op::Loop { index } => {
                    #[cfg(test)]
                    count_clause_op_entry();
                    let index = *index as usize;
                    let Some(instruction) = code.body.instructions.get(index) else {
                        return Err(Loud::chunk_map_too_short().into());
                    };
                    if GRANTING {
                        self.grant_procedure_permission(instruction);
                    }
                    let flow = self.step_in_temps_frame_with(
                        code,
                        index,
                        instruction,
                        source,
                        BodyEngine::Chunk { chunk, registers },
                    )?;
                    (flow, pc + 1)
                }
                Op::Clause { index, end } => {
                    #[cfg(test)]
                    count_clause_op_entry();
                    let index = *index as usize;
                    let end = *end;
                    let Some(instruction) = code.body.instructions.get(index) else {
                        return Err(Loud::chunk_map_too_short().into());
                    };
                    if GRANTING {
                        self.grant_procedure_permission(instruction);
                    }
                    match self.run_clause_region(
                        code,
                        chunk,
                        registers,
                        pc + 1,
                        end,
                        index,
                        instruction,
                        source,
                    )? {
                        // A promoted clause produces no `Flow` of its own:
                        // where it leaves the counter *is* its answer, which
                        // is what a jump op is for. Only its boundary can end
                        // the activation, and that is the `Exit` below.
                        ClauseRegion::Continue(next) => {
                            pc = next;
                            continue;
                        }
                        ClauseRegion::Exit(value) => (Flow::Exit(value), pc),
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
                    let text = match case {
                        Some(register) => {
                            debug_assert!(
                                chunk.holds_register(*register),
                                "op reads register {register} outside the region the chunk \
                                 reserved"
                            );
                            let value = self.roots.temp_at(registers, *register as usize);
                            Some(self.to_text(value).to_vec())
                        }
                        None => None,
                    };
                    debug_assert!(
                        code.body.instructions.get(*index as usize).is_some(),
                        "a SelectCaseText op names an instruction outside its own body"
                    );
                    self.current_case_text = text;
                    pc += 1;
                    continue;
                }
                Op::EnterWhen { select, when } => {
                    frames.push(self.when_frame(code, chunk, *select as usize, *when as usize)?);
                    pc += 1;
                    continue;
                }
                Op::EnterOtherwise { select } => {
                    frames.push(self.otherwise_frame(code, chunk, *select as usize)?);
                    pc += 1;
                    continue;
                }
                // All three are only meaningful inside a `Clause` region, which
                // `run_clause_region` walks: reaching one here means a jump
                // landed in the middle of a region rather than on its
                // `Clause`. Loud rather than a panic, which is this crate's
                // standing rule for a state the type system admits and the
                // compiler does not produce.
                Op::EvalExpr { .. } => return Err(Loud::op_not_driven("EvalExpr").into()),
                Op::JumpUnless { .. } => return Err(Loud::op_not_driven("JumpUnless").into()),
                Op::WhenTest { .. } => return Err(Loud::op_not_driven("WhenTest").into()),
            };
            // **Per clause, not per escaping flow.** A clause that left the
            // activation stack changed makes this loop's `code` describe a
            // frame it is no longer running, and every op after it is stepped
            // against the wrong body -- so the check has to sit where a clause
            // finishes rather than where control leaves this function.
            debug_assert_eq!(
                self.activations.len(),
                depth,
                "a clause left the activation stack changed, so this loop's `code` and its `pc` \
                 no longer describe the same frame"
            );
            match self.settle(code, chunk, &mut frames, flow, next, start, end)? {
                Settled::At(target) => pc = target,
                Settled::Escaped(other) => return Ok(other),
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
    /// replaced an `absorb` match written out in the driver's own loop, which
    /// is one call per clause of every promoted body. Left to the inliner's
    /// judgement it is emitted as a function and `bench-programs/emptyloop.rex`
    /// -- a loop whose body does nothing, so the measurement is the per-clause
    /// cost and almost nothing else -- runs 2.85s before this promotion and
    /// 3.39s after, on the compiled stream, interleaved across three sittings.
    /// With the annotation it is 2.86s, which is the level the promotion found.
    #[expect(
        clippy::too_many_arguments,
        reason = "two callers inside one loop, and every argument is a value that loop holds"
    )]
    #[inline(always)]
    fn settle(
        &mut self,
        code: &Code<'_>,
        chunk: &Chunk,
        frames: &mut Vec<SelectFrame>,
        mut flow: Flow,
        next: u32,
        start: usize,
        end: usize,
    ) -> Result<Settled, Failure> {
        loop {
            // Copied out rather than held, because the `Escaped` arm pops the
            // frame this came from.
            let Some((frame_start, frame_end)) = frames.last().map(|f| (f.start, f.end)) else {
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
                    let frame = frames.pop().expect("the check above just observed one");
                    flow = self.leave_branch(code, &frame, other)?;
                }
            }
        }
    }

    /// What a `SELECT` does with a `Flow` that left one of its branches, which
    /// is `leave_select` -- exactly what `step`'s own `Select` arm and
    /// `run_otherwise` do with the same `Flow`.
    ///
    /// **`select_escape` has no counterpart here, and its absence is the op
    /// layout doing the same job.** The tree-walker needs that decision
    /// because its `run_bounded` owns one range and a `Flow::Goto` landing on
    /// the `OTHERWISE` marker escapes it, leaving the construct entirely
    /// unless something recognises the target. Here `Chunk::op_of` *is* the
    /// resume table and [`Op::EnterOtherwise`] sits at the `OTHERWISE`'s entry
    /// in it, so **every** arrival there opens the branch's frame -- the scan
    /// running out of `WHEN`s and an absorbed `WHEN CASE`'s escape alike.
    /// Measured: a driver arm making the decision explicitly as well could be
    /// deleted with the whole workspace, corpus included, still green, because
    /// the layout had already answered it; what does redden is moving
    /// `EnterOtherwise` off that entry.
    ///
    /// [`Op::EnterOtherwise`]: super::Op::EnterOtherwise
    fn leave_branch(
        &mut self,
        code: &Code<'_>,
        frame: &SelectFrame,
        flow: Flow,
    ) -> Result<Flow, Failure> {
        let flow = match frame.branch {
            Branch::When => flow,
            // `run_otherwise`'s own last two lines, in order: the offset is
            // restored once that whole dispatch -- marker and body alike -- is
            // finished reading it, and only then does `leave_select` run. A
            // raise leaves it unrestored here for the same reason it does
            // there, since nothing runs afterward to see a stale value.
            Branch::Otherwise => {
                self.indent_offset = 0;
                flow
            }
        };
        self.leave_select(code, frame.select, frame.label, frame.resume, flow)
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
            start: when + 1,
            end: targets.body_end,
            resume: SelectResume {
                done: targets.resume,
                left: targets.resume,
            },
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
            start: otherwise,
            end: otherwise_end,
            resume: SelectResume {
                done: otherwise_end,
                left: select_exit(code.body.instructions.len(), parts.end),
            },
            op_end: op_at(chunk, otherwise_end)?,
            branch: Branch::Otherwise,
        })
    }

    /// One promoted clause: the ops of `(at - 1, end)`, run inside the same
    /// clause wrapper an unpromoted instruction gets.
    ///
    /// `at` is the op after the `Clause` op itself. The register mark the
    /// plan's Decisions section describes is a compile-time quantity -- the
    /// allocator releases to it when this region's ops were emitted -- so
    /// there is nothing to release here: the registers this region wrote are
    /// simply not addressed again.
    #[expect(
        clippy::too_many_arguments,
        reason = "one caller, and every argument is a value that caller already holds"
    )]
    fn run_clause_region(
        &mut self,
        code: &Code<'_>,
        chunk: &Chunk,
        registers: FrameId,
        at: u32,
        end: u32,
        index: usize,
        instruction: &rexx_parse::Instruction,
        source: Option<&ProgramSource>,
    ) -> Result<ClauseRegion, Failure> {
        let outcome = self.in_stepped_clause(code, index, instruction, source, |it| {
            // Taken on entry exactly as `step` takes it, because a promoted
            // clause is a clause and the permission is spent by whichever
            // clause the activation granted it to.
            //
            // **Unobservable, and nothing here makes it observable.** Deleting
            // this line changes no test's answer, and so does deleting the
            // `grant_procedure_permission` call that precedes this clause. The
            // reason is a property of what `compile` happens to emit -- an op
            // that grants follows every `Clause` region it currently produces,
            // and grants again before any `PROCEDURE` can be reached -- and
            // **nothing enforces that property**: no assertion states it, and
            // a promotion that emits a region followed by something else would
            // make both lines load-bearing with nothing going red in between.
            // They stay because the obligation belongs to the clause unit; do
            // not read them as a guarantee that anything checks them.
            let _first_instruction = std::mem::take(&mut it.procedure_permitted);
            it.run_region_ops(code, chunk, registers, at, end)
        })?;
        match outcome {
            ClauseOutcome::Ran(next) => Ok(ClauseRegion::Continue(next?.0)),
            ClauseOutcome::Ended(exit) => Ok(ClauseRegion::Exit(exit.value())),
        }
    }

    /// The ops of one promoted clause, `[at, end)`, and where they leave the
    /// counter.
    ///
    /// Only the ops that are part of a clause's own work appear here. An op
    /// that runs a whole clause of its own does not, and cannot: `compile`
    /// asserts no `Generic` or `Loop` sits inside a region, because both echo
    /// the clause and the echo is not idempotent.
    fn run_region_ops(
        &mut self,
        code: &Code<'_>,
        chunk: &Chunk,
        registers: FrameId,
        at: u32,
        end: u32,
    ) -> Result<ClauseNext, Failure> {
        let mut pc = at;
        while pc < end {
            let Some(op) = chunk.op_at_index(pc) else {
                return Err(Loud::chunk_map_too_short().into());
            };
            match op {
                Op::EvalExpr { index, slot, dst } => {
                    debug_assert!(
                        chunk.holds_register(*dst),
                        "op writes register {dst} outside the region the chunk reserved"
                    );
                    let value = self.eval_chunk_expr(code, *index as usize, *slot)?;
                    self.roots.set_temp(registers, *dst as usize, value);
                    pc += 1;
                }
                Op::JumpUnless { reg, target } => {
                    debug_assert!(
                        chunk.holds_register(*reg),
                        "op reads register {reg} outside the region the chunk reserved"
                    );
                    if self.register_holds(registers, *reg)? {
                        pc += 1;
                    } else {
                        return Ok(ClauseNext(*target));
                    }
                }
                Op::WhenTest { index, case, dst } => {
                    debug_assert!(
                        chunk.holds_register(*dst),
                        "op writes register {dst} outside the region the chunk reserved"
                    );
                    let case_text = match case {
                        Some(register) => {
                            debug_assert!(
                                chunk.holds_register(*register),
                                "op reads register {register} outside the region the chunk \
                                 reserved"
                            );
                            let value = self.roots.temp_at(registers, *register as usize);
                            Some(self.to_text(value).to_vec())
                        }
                        None => None,
                    };
                    let Some(instruction) = code.body.instructions.get(*index as usize) else {
                        return Err(Loud::chunk_map_too_short().into());
                    };
                    let holds = self.scan_when(code, instruction, case_text.as_deref())?;
                    // In range unconditionally: `SMALL_INT_MAX` is far above
                    // one. Stored as the logical value `Op::JumpUnless` reads
                    // back, exactly as an `IF`'s condition is.
                    let value = ObjRef::small_int(i64::from(holds)).unwrap_or(ObjRef::NIL);
                    self.roots.set_temp(registers, *dst as usize, value);
                    pc += 1;
                }
                Op::Jump { target } => {
                    return Ok(ClauseNext(*target));
                }
                Op::Generic { .. } => return Err(Loud::op_not_driven("Generic").into()),
                Op::Loop { .. } => return Err(Loud::op_not_driven("Loop").into()),
                Op::Clause { .. } => return Err(Loud::op_not_driven("Clause").into()),
                Op::SelectCaseText { .. } => {
                    return Err(Loud::op_not_driven("SelectCaseText").into());
                }
                Op::EnterWhen { .. } => return Err(Loud::op_not_driven("EnterWhen").into()),
                Op::EnterOtherwise { .. } => {
                    return Err(Loud::op_not_driven("EnterOtherwise").into());
                }
            }
        }
        Ok(ClauseNext(end))
    }

    /// Whether register `reg` holds the Rexx logical value `1`.
    ///
    /// The only writer of a register a `JumpUnless` reads is an `EvalExpr`
    /// whose expression `eval_condition` has already validated as exactly
    /// `0` or `1`, so this is a readback rather than a second check --
    /// re-deriving the answer from the value's text would be a second
    /// implementation of the rule that decides a branch. Anything else in the
    /// register means the two ops came apart, which is loud rather than a
    /// silently-taken branch.
    fn register_holds(&self, registers: FrameId, reg: u16) -> Result<bool, Failure> {
        match self.roots.temp_at(registers, reg as usize).decode() {
            Decoded::SmallInt(1) => Ok(true),
            Decoded::SmallInt(0) => Ok(false),
            _ => Err(Loud::register_not_logical().into()),
        }
    }
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

/// How a promoted clause finished.
enum ClauseRegion {
    /// Continue at this op.
    Continue(u32),
    /// A `CALL ON` handler ran at this clause's boundary and ended the whole
    /// program.
    Exit(Option<ObjRef>),
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
// `Generic`, a `Loop`, and a promoted `Clause` region. So a construct that
// moves from one of those shapes to another does not change the count, and a
// construct whose clauses stop reaching the stream does.
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

#[cfg(test)]
mod tests;
