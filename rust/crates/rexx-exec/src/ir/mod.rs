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

/// One instruction in a compiled stream.
///
/// `Generic` is what an unpromoted instruction compiles to: it carries no
/// payload because there is nothing to add to what the tree-walker's own
/// clause unit already does with the instruction at this index. `Loop` is the
/// same shape for a construct whose body the driver steps.
///
/// `Clause` and `EvalExpr` are declared and unconstructed. They are the
/// shapes a promotion that *flattens* a construct into a run of ops needs --
/// a clause spanning more than one op, and an expression evaluated into a
/// register -- and no promotion so far flattens one. Declaring them here is
/// what lets that promotion extend this enum rather than reshape it.
pub(crate) enum Op {
    /// Delegates the instruction at this index back to the tree-walker.
    Generic,
    /// A `DO`/`LOOP` whose **member clauses run from this chunk**.
    ///
    /// The construct itself is resolved by the same `Interp::run_loop` the
    /// tree-walker enters -- header validation, every iteration,
    /// `WHILE`/`UNTIL`, the `LEAVE`/`ITERATE` label search and every trace
    /// echo are one implementation, entered from both engines, rather than
    /// two. What this op changes is the one line inside it that was
    /// engine-specific: the body's clauses are stepped through
    /// `Interp::step_from_chunk` instead of straight into the tree-walker's
    /// clause unit, so an instruction inside a loop is reachable from the
    /// compiled stream at all.
    ///
    /// Carries no payload for the same reason [`Op::Generic`] does not: the
    /// driver reaches it through `Chunk::op_of`, so the instruction index is
    /// already in hand.
    Loop,
    /// A promoted clause. `end` is the op index one past this clause's last
    /// op -- the mark the register allocator releases to when the clause
    /// finishes (the plan's Decisions section: "a promoted clause takes a
    /// mark when its `Clause` op is emitted and releases to it at `end`").
    #[expect(
        dead_code,
        reason = "declared for a promotion that flattens a construct into ops; none does yet"
    )]
    Clause { end: u32 },
    /// A native expression evaluation, still dispatched through `eval.rs`
    /// rather than reimplemented here (the plan's Decisions section: trace
    /// ops land before the first *native* expression op, so `EvalExpr`
    /// stays trace-identical to the `eval.rs` call it wraps). `index` and
    /// `slot` address the expression within its instruction and `dst` is
    /// the destination register; their exact meaning belongs to whichever
    /// promotion first emits one.
    #[expect(
        dead_code,
        reason = "declared for a promotion that flattens a construct into ops; none does yet"
    )]
    EvalExpr { index: u32, slot: u32, dst: u16 },
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
    /// The clauses are stepped from `Chunk`, whose `op_of` indexes exactly the
    /// body `Code::body` names.
    Chunk(&'a Chunk),
}

/// The one way [`compile`] can fail: a body that does not fit the index
/// widths this stream commits to, not a language construct it refuses (the
/// plan's Decisions section: "the compiler has one error, and it is a
/// machine width rather than a language construct"). Every instruction
/// still compiles; this reports that the *stream* built from them does not
/// fit some index's width.
#[derive(Debug)]
pub(crate) struct ChunkTooLarge {
    /// What overflowed. Always `"op stream"` today: op indices are `u32`
    /// and nothing allocates a register yet, so nothing else can overflow
    /// before a register allocator exists to overflow `u16::MAX`.
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
    /// Instruction index -> op index into `ops`. One entry per instruction,
    /// in order, plus one final entry at `ops.len()`, pushed *after* the
    /// loop that fills the rest: `run_bounded`'s absorption guard is
    /// inclusive, so a construct's resume point can be one past its last
    /// instruction, and a map that stopped at `len - 1` would panic there
    /// instead of failing loudly at compile.
    op_of: Vec<u32>,
    /// The register allocator's high-water mark (the plan's Decisions
    /// section: "the chunk records its high-water mark"). Always `0` while
    /// nothing addresses a register: a `DO`/`LOOP` holds its control value in
    /// `run_loop`'s own `LoopState`, not here.
    /// `Interp::run_chunk` reserves this many registers before running a
    /// chunk and truncates them away on the way out.
    registers: u16,
}
