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

//! The register-based instruction stream (Phase 4e): `Op`, `Chunk`, and the
//! shapes later tasks extend rather than reshape.
//!
//! `compile` (`compile.rs`) walks a body once and emits one `Op` per
//! instruction. `Op::Generic` delegates the instruction at its index back to
//! the tree-walker's own clause unit (`Interp::step_in_temps_frame`, and
//! `clause.rs` for the boundary it carries). A construct is promoted by
//! teaching `compile` to emit something other than `Generic` for it, never by
//! changing what `Generic` means.
//!
//! `Interp::chunk_for` (`plan.rs`, beside `plan_for`, under the same
//! `BodyKey`) is the cache: a body compiles once, on first entry, and the
//! result is kept for every later one. `drive.rs` is what runs the result,
//! and `run_activation` is what chooses between it and the tree-walker
//! (`Interp::engine`, from the `Invocation`).

use rexx_core::FrameId;

mod compile;
mod drive;
pub(crate) use compile::compile;

// `render`, the golden-test serialiser, has no caller outside `golden_tests.rs`
// and the plan's ten tasks never give it one -- gated here rather than carrying
// a permanent `#[allow(dead_code)]` for a caller that is never coming.
#[cfg(test)]
mod golden;

#[cfg(test)]
mod golden_tests;

/// One step in a compiled stream.
///
/// **Every op that runs a clause carries its own instruction index**, and
/// that is what lets it sit inside a region a jump lands in: the driver's
/// program counter is an *op* index, so there is no instruction counter
/// walking alongside it to read the index off. `Op::Generic` is what an
/// unpromoted instruction compiles to and `Op::Loop` the same for a `DO`/
/// `LOOP` whose body the driver steps; a promoted construct compiles to a
/// `Clause` region followed by whatever jumps its control flow needs.
pub(crate) enum Op {
    /// Delegates the instruction at `index` back to the tree-walker's own
    /// clause unit, which runs the whole clause.
    Generic { index: u32 },
    /// A `DO`/`LOOP` at `index` whose **member clauses run from this chunk**.
    ///
    /// The construct itself is resolved by the same `Interp::run_loop` the
    /// tree-walker enters -- header validation, every iteration,
    /// `WHILE`/`UNTIL`, the `LEAVE`/`ITERATE` label search and every trace
    /// echo are one implementation, entered from both engines, rather than
    /// two. What this op changes is the one line inside it that was
    /// engine-specific: the body's clauses are stepped from the compiled
    /// stream instead of straight into the tree-walker's clause unit, so an
    /// instruction inside a loop is reachable from the stream at all.
    Loop { index: u32 },
    /// Opens the promoted clause of the instruction at `index`. `end` is the
    /// op index one past this clause's last op -- the mark the register
    /// allocator releases to when the clause finishes (the plan's Decisions
    /// section: "a promoted clause takes a mark when its `Clause` op is
    /// emitted and releases to it at `end`").
    ///
    /// **No `Generic` or `Loop` op may sit inside `(here, end)`**, which
    /// `compile` asserts: both run a whole clause through
    /// `step_in_temps_frame`, which echoes the clause itself, and the echo is
    /// not idempotent.
    Clause { index: u32, end: u32 },
    /// Evaluates expression `slot` of the instruction at `index` into
    /// register `dst`, still dispatched through `eval.rs` rather than
    /// reimplemented here (the plan's Decisions section: trace ops land
    /// before the first *native* expression op, so this stays
    /// trace-identical to the `eval.rs` call it wraps).
    ///
    /// **Only valid inside a `Clause` region**, whose clause is the one the
    /// evaluation's trace lines and failure attribution belong to.
    ///
    /// `slot` numbers the instruction's own expressions. An `If` has one,
    /// slot `0`, and its value is a Rexx logical value -- `eval_condition`
    /// validates it as exactly `0` or `1` before this op stores it, which is
    /// why [`Op::JumpUnless`] can test it without repeating the validation.
    EvalExpr { index: u32, slot: u32, dst: u16 },
    /// Continues at op `target`.
    Jump { target: u32 },
    /// Continues at op `target` unless register `reg` holds the logical value
    /// `1`, and at the next op if it does.
    ///
    /// **Only valid inside a `Clause` region**, for the reason
    /// [`Op::EvalExpr`] is: the register it reads is one that region wrote,
    /// and the region's own end is where that register is released.
    JumpUnless { reg: u16, target: u32 },
}

/// Which driver steps the member clauses of a construct that resolves the
/// whole of itself inside one clause step -- a `DO`/`LOOP`'s body today.
///
/// The construct's own semantics do not branch on this: `Interp::run_bounded`
/// is the one loop that reads it, and the only thing it decides is whether a
/// member clause is handed to the tree-walker's clause unit or to the
/// compiled stream's. Everything else about the construct -- ordering, trace,
/// clause attribution, the `LEAVE`/`ITERATE` search -- is above this and is
/// shared.
///
/// **A fragment is never `Chunk`.** `INTERPRET` text compiles to no chunk (the
/// plan's Decisions section: "a fragment does not compile to a chunk in this
/// phase"), and its `Code` is a different body from the one a chunk's
/// `op_of` indexes, so `run_fragment` passes `TreeWalker` unconditionally.
#[derive(Clone, Copy)]
pub(crate) enum BodyEngine<'a> {
    TreeWalker,
    /// The clauses are stepped from `chunk`, whose `op_of` indexes exactly the
    /// body `Code::body` names, with `registers` naming the region
    /// `Interp::run_chunk` reserved for it.
    ///
    /// The register region travels with the chunk rather than living on
    /// `Interp`, because the two are one fact: a register index means nothing
    /// without the region it is an index into, and a chunk compiled and run
    /// inside a running chunk (a callee's body) has a region of its own
    /// stacked above the caller's.
    Chunk {
        chunk: &'a Chunk,
        registers: FrameId,
    },
}

/// The one way [`compile`] can fail: a body that does not fit the index
/// widths this stream commits to, not a language construct it refuses (the
/// plan's Decisions section: "the compiler has one error, and it is a
/// machine width rather than a language construct"). Every instruction
/// still compiles; this reports that the *stream* built from them does not
/// fit some index's width.
#[derive(Debug)]
pub(crate) struct ChunkTooLarge {
    /// What overflowed: the `u32` op stream, or the `u16` register file.
    ///
    /// **No production reader, and the allow says so rather than a fake one
    /// being invented for it.** `chunk_for` is the only caller that sees an
    /// `Err`, and a refusal is not a failure there: it bumps
    /// `Interp::chunks_refused` and runs the body on the tree-walker, with
    /// nothing to print. The field is what makes a refusal diagnosable when a
    /// second reason for one exists; the counter is what makes it
    /// impossible to miss.
    #[allow(dead_code, reason = "no production caller renders a refusal's reason")]
    pub(crate) what: &'static str,
}

/// One body's compiled instruction stream, cached on `Interp` under the same
/// `BodyKey` its `Plan` is (`Interp::chunk_for`, in `plan.rs`).
pub(crate) struct Chunk {
    /// One entry per instruction, in instruction order -- D21's "every
    /// instruction compiles, nothing refuses" is a claim about instructions,
    /// not about promotion, so an instruction no task has promoted still gets
    /// an op.
    ops: Vec<Op>,
    /// Instruction index -> **the op control resumes at** when it arrives at
    /// that instruction. One entry per instruction, in order, plus one final
    /// entry at `ops.len()`: `run_bounded`'s absorption guard is inclusive,
    /// so a construct's resume point can be one past its last instruction,
    /// and a map that stopped at `len - 1` would panic there instead of
    /// failing loudly at compile.
    ///
    /// **A resume, not the instruction's own first op, and the two differ at
    /// exactly one place.** An `IF`'s true branch ends with a jump past the
    /// `ELSE`, and that jump sits at the `ELSE`'s entry here -- so a `THEN`
    /// branch that finishes, whether by falling off its end or by a nested
    /// construct answering `Flow::Goto` at the boundary, skips the `ELSE`
    /// rather than running it. The one arrival that must run the `ELSE` is
    /// the `IF`'s own false path, and that is a `JumpUnless` the compiler
    /// resolved against the instruction's first op instead.
    op_of: Vec<u32>,
    /// The register allocator's high-water mark (the plan's Decisions
    /// section: "the chunk records its high-water mark").
    /// `Interp::run_chunk` reserves this many registers before running a
    /// chunk and truncates them away on the way out.
    registers: u16,
}

impl Chunk {
    /// The op instruction `index` starts at, or `None` when the map is
    /// shorter than the body it belongs to.
    ///
    /// `end` is in range too: `compile` writes one entry past the last
    /// instruction, because a construct's resume point can be one past its
    /// own last one.
    fn op_at(&self, index: usize) -> Option<u32> {
        self.op_of.get(index).copied()
    }

    /// The op at op index `at`.
    fn op_at_index(&self, at: u32) -> Option<&Op> {
        self.ops.get(at as usize)
    }

    /// Whether `reg` is inside the region `run_chunk` reserves for this
    /// chunk.
    ///
    /// **The tripwire for a high-water mark that undercounts.** The region
    /// sits on the temporaries stack, so a register index past its end
    /// addresses whatever a clause happened to push there rather than failing
    /// -- it reads and writes a live temporary of somebody else's, which is a
    /// wrong value found by chasing it. `run_ops` asserts this in debug at
    /// both the write and the read.
    fn holds_register(&self, reg: u16) -> bool {
        reg < self.registers
    }
}
