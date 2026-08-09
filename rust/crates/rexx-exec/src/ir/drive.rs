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
//! It sits beside `run_activation`'s own loop rather than replacing it --
//! `Interp::engine` chooses between the two -- and it discharges exactly the
//! same per-clause obligations, through the same functions, because those
//! were extracted from that loop rather than copied out of it:
//! `grant_procedure_permission`, `step_in_temps_frame`, `offer_to_trap` and
//! `apply_flow`.

use rexx_parse::{Instruction, ProgramSource};

use super::{BodyEngine, Chunk, Op};
use crate::run::{Ended, Flow};
use crate::{Code, Failure, Interp, Loud};

impl Interp {
    /// Runs `chunk` for the activation on top of the stack.
    ///
    /// The outer loop iterates clauses; the inner one runs that clause's own
    /// ops. The `pc` stays an **instruction** index throughout, mapped
    /// through `chunk.op_of` at the top of each clause, which is what lets
    /// [`Interp::apply_flow`] be reused unchanged -- `Flow::Goto` and
    /// `Flow::Signal` both carry instruction indices, and `Signal`'s resolve
    /// against the activation's own body rather than against the body being
    /// stepped, which inside an `INTERPRET` fragment are different bodies.
    ///
    /// **Two levels rather than one flat loop, and the reason is
    /// `in_clause`.** That is a scoped closure (`clause.rs`): it sets the
    /// clause line, runs the whole clause, and then, only on the success
    /// path, delivers a queued `CALL ON` handler, which can end the program.
    /// The shape is built for a clause that spans a run of ops, because a
    /// flat stream has no scope to hang that on -- an `Op::Generic` needs no
    /// such scope, since the call it makes is a whole clause.
    ///
    /// The register region is reserved once, here, from `chunk.registers`,
    /// and truncated away on every path out. It sits on the temporaries
    /// stack (`RootSet::reserve_temps`' own doc has why neither a slot frame
    /// of its own nor the activation's own frame works), below every
    /// watermark `step_in_temps_frame` takes, so a clause's own frame pops
    /// back to above it rather than through it.
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
        let ended = self.run_chunk_clauses(code, chunk, source);
        self.roots.pop_frame(registers);
        ended
    }

    /// The outer level: one iteration per clause.
    ///
    /// The body of `run_chunk` past the register region, split out so the
    /// truncation above covers every way this returns.
    fn run_chunk_clauses(
        &mut self,
        code: &Code<'_>,
        chunk: &Chunk,
        source: Option<&ProgramSource>,
    ) -> Result<Ended, Failure> {
        let depth = self.activations.len();

        while let Some(instruction) = code.body.instructions.get(self.activation().pc) {
            let index = self.activation().pc;
            self.grant_procedure_permission(instruction);
            let flow = match self.step_from_chunk(code, chunk, index, instruction, source) {
                Ok(flow) => flow,
                // **The trap offer stays here**, at the same position it
                // holds in `run_activation`'s own loop, because the position
                // is the semantics: one offer per activation, made by the
                // activation that is unwinding. A nested `run_bounded` (an
                // `IF` branch, a `WHEN` body) shares this activation's traps
                // and must not get a second offer, which is what moving this
                // inside `apply_flow` would give it.
                Err(failure) => self.offer_to_trap(code, failure)?,
            };
            if let Some(ended) = self.apply_flow(code, flow)? {
                return Ok(ended);
            }
            debug_assert_eq!(
                self.activations.len(),
                depth,
                "a clause left the activation stack changed, so this loop's `code` and its `pc` \
                 no longer describe the same frame"
            );
        }
        // Out of instructions. `Exited` and not `Returned`, for the reason
        // `Ended::Exited`'s own doc gives and `run_activation`'s loop ends
        // with: a callee that runs off the end of the file ends the
        // *program*, and the caller's next clause never runs.
        Ok(Ended::Exited(None))
    }

    /// Steps the clause at instruction index `index` from `chunk`.
    ///
    /// **The one mapping from the instruction space a `pc` lives in into the
    /// op space the inner level walks**, and the one guard on it. Both the
    /// outer loop above and `Interp::run_bounded`'s [`BodyEngine::Chunk`] arm
    /// come through here, so a clause reached from inside a `DO`/`LOOP` body
    /// is mapped exactly as one reached from the top of the body is, and a
    /// map too short for the body it belongs to fails the same way from
    /// either.
    ///
    /// `compile` writes one entry per instruction plus a final one, so the
    /// lookup is in range for any index that indexes an instruction.
    pub(crate) fn step_from_chunk(
        &mut self,
        code: &Code<'_>,
        chunk: &Chunk,
        index: usize,
        instruction: &Instruction,
        source: Option<&ProgramSource>,
    ) -> Result<Flow, Failure> {
        let Some(&start) = chunk.op_of.get(index) else {
            return Err(Loud::chunk_map_too_short().into());
        };
        self.run_clause_ops(code, chunk, start, index, instruction, source)
    }

    /// The inner level: the ops of the clause starting at op `start`, whose
    /// instruction is `instruction` at instruction index `index`.
    ///
    /// Answers the clause's own `Flow`, which the outer loop applies.
    ///
    /// `start` is an **op** index, where the outer loop's `index` is an
    /// instruction index, and `chunk.op_of` is the one place the two spaces
    /// meet. Taking an op index rather than reusing the instruction one is
    /// what lets a clause be built from more than one op; an `Op::Generic` is
    /// a whole clause on its own, since the call it makes runs one, so it
    /// answers the clause's `Flow` directly.
    ///
    /// `index` travels beside `instruction` rather than being derived from
    /// it: `If` and `Select` compute a branch's start from their own
    /// position, and a nested `run_bounded` steps an instruction the
    /// activation's `pc` is not pointing at.
    fn run_clause_ops(
        &mut self,
        code: &Code<'_>,
        chunk: &Chunk,
        start: u32,
        index: usize,
        instruction: &Instruction,
        source: Option<&ProgramSource>,
    ) -> Result<Flow, Failure> {
        #[cfg(test)]
        count_clause_op_entry();

        let Some(op) = chunk.ops.get(start as usize) else {
            return Err(Loud::chunk_map_too_short().into());
        };
        match op {
            // **`Generic` delegates to `step_in_temps_frame`, not to
            // `step`.** `step` is not the clause unit: its wrapper carries
            // the per-clause clock invalidation, the `>I>` trace-entry
            // decay, `current_value_indent`, the `SIGL` clause line, the
            // clause boundary through `in_clause`, the clause echo, the GC
            // temps frame with its watermark tripwire, and failure-site
            // resolution. That call is also why **no `Clause` op precedes a
            // `Generic` one**: it echoes the clause itself, and the echo is
            // not idempotent.
            Op::Generic => self.step_in_temps_frame(code, index, instruction, source),
            // **The clause wrapper is the same one `Generic` takes**, and the
            // whole of the difference is the `BodyEngine` it carries: the
            // construct is resolved by `run_loop`, exactly as the tree-walker
            // resolves it, and the engine decides only how each of its body's
            // clauses is stepped. Writing a second loop here instead is the
            // defect the dual-engine sweep exists to catch.
            Op::Loop => self.step_in_temps_frame_with(
                code,
                index,
                instruction,
                source,
                BodyEngine::Chunk(chunk),
            ),
            // Neither variant has a constructor: `compile` emits `Generic`
            // and `Loop`, and `golden.rs`'s renderer is the only other thing
            // that names either. Loud rather than a panic, which is this
            // crate's standing rule for a state the type system admits and
            // the code does not produce.
            Op::Clause { .. } => Err(Loud::op_not_driven("Clause").into()),
            Op::EvalExpr { .. } => Err(Loud::op_not_driven("EvalExpr").into()),
        }
    }
}

// Test-only instrumentation: how many chunks this thread has driven.
//
// The engine-selection tests need this to tell "the IR engine ran this body"
// apart from "the run produced the answer the tree-walker also produces".
// Every op is `Op::Generic`, so the two engines agree on every program by
// construction and no observable output tells them apart -- a selection test
// resting on output alone would pass with selection deleted.
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
// thing promoting `DO`/`LOOP` changes: whether a clause **inside** a loop body
// reaches the stream at all. Both engines produce identical bytes for every
// program by construction here, and an activation entered is one entry either
// way, so this is the only observable that separates a body driven from the
// chunk from a body driven straight into the tree-walker.
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
