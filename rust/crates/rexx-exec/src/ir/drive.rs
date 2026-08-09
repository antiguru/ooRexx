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
use rexx_parse::ProgramSource;

use super::{BodyEngine, Chunk, Op};
use crate::clause::{ClauseOutcome, ClauseValue};
use crate::run::{Absorbed, Ended, Flow, absorb};
use crate::{Code, Failure, Interp, Loud};

/// What a body that runs off its own end answers.
///
/// `Exited` and not `Returned`, for the reason `Ended::Exited`'s own doc
/// gives and `run_activation`'s loop ends with: a callee that runs off the
/// end of the file ends the *program*, and the caller's next clause never
/// runs.
const END_OF_BODY: Ended = Ended::Exited(None);

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
        let mut pc = at;
        while pc < stop {
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
                // Both are only meaningful inside a `Clause` region, which
                // `run_clause_region` walks: reaching one here means a jump
                // landed in the middle of a region rather than on its
                // `Clause`. Loud rather than a panic, which is this crate's
                // standing rule for a state the type system admits and the
                // compiler does not produce.
                Op::EvalExpr { .. } => return Err(Loud::op_not_driven("EvalExpr").into()),
                Op::JumpUnless { .. } => return Err(Loud::op_not_driven("JumpUnless").into()),
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
            match absorb(flow, start, end) {
                Absorbed::Advance => pc = next,
                Absorbed::Resume(target) => {
                    let Some(at) = chunk.op_at(target) else {
                        return Err(Loud::chunk_map_too_short().into());
                    };
                    pc = at;
                }
                Absorbed::Escaped(other) => return Ok(other),
            }
        }
        Ok(Flow::Next)
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
                Op::Jump { target } => {
                    return Ok(ClauseNext(*target));
                }
                Op::Generic { .. } => return Err(Loud::op_not_driven("Generic").into()),
                Op::Loop { .. } => return Err(Loud::op_not_driven("Loop").into()),
                Op::Clause { .. } => return Err(Loud::op_not_driven("Clause").into()),
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
