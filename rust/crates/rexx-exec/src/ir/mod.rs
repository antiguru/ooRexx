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
//! instruction. Every op it emits is `Op::Generic`, which delegates the
//! instruction at its index back to the tree-walker's own clause unit
//! (`Interp::step_in_temps_frame`, and `clause.rs` for the boundary it
//! carries). A construct is promoted by teaching `compile` to emit something
//! other than `Generic` for it, never by changing what `Generic` means.
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
/// `Generic` is what every instruction in this task compiles to: it carries
/// no payload because there is nothing to add to what the tree-walker's own
/// clause unit already does with the instruction at this index.
///
/// `Clause` and `EvalExpr` are declared here and constructed by none of this
/// task's own code -- Task 4 is the first to emit either (the plan's
/// "Ambiguities the controller resolved for this task"). Declaring both now
/// is what lets Task 4 extend this enum rather than reshape it.
pub(crate) enum Op {
    /// Delegates the instruction at this index back to the tree-walker.
    Generic,
    /// A promoted clause. `end` is the op index one past this clause's last
    /// op -- the mark the register allocator releases to when the clause
    /// finishes (the plan's Decisions section: "a promoted clause takes a
    /// mark when its `Clause` op is emitted and releases to it at `end`").
    #[expect(dead_code, reason = "Task 4 is this variant's first constructor")]
    Clause { end: u32 },
    /// A native expression evaluation, still dispatched through `eval.rs`
    /// rather than reimplemented here (the plan's Decisions section: trace
    /// ops land before the first *native* expression op, so `EvalExpr`
    /// stays trace-identical to the `eval.rs` call it wraps). `index` and
    /// `slot` address the expression within its instruction and `dst` is
    /// the destination register; their exact meaning is Task 4's to define.
    #[expect(dead_code, reason = "Task 4 is this variant's first constructor")]
    EvalExpr { index: u32, slot: u32, dst: u16 },
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
    /// `Err`, and a refusal is not a failure there: it counts the body in
    /// `Interp::chunks_refused` and runs it on the tree-walker, with nothing
    /// to print. The field is what makes a refusal diagnosable when a
    /// second reason for one exists; the counter is what makes it
    /// impossible to miss.
    #[allow(dead_code, reason = "no production caller renders a refusal's reason")]
    pub(crate) what: &'static str,
}

/// One body's compiled instruction stream, cached on `Interp` under the same
/// `BodyKey` its `Plan` is (`Interp::chunk_for`, in `plan.rs`).
pub(crate) struct Chunk {
    /// One entry per instruction, `Op::Generic` for every one of them --
    /// D21's "every instruction compiles, nothing refuses" is a claim about
    /// instructions, not about promotion.
    ops: Vec<Op>,
    /// Instruction index -> op index into `ops`. One entry per instruction,
    /// in order, plus one final entry at `ops.len()`, pushed *after* the
    /// loop that fills the rest: `run_bounded`'s absorption guard is
    /// inclusive, so a construct's resume point can be one past its last
    /// instruction, and a map that stopped at `len - 1` would panic there
    /// instead of failing loudly at compile.
    op_of: Vec<u32>,
    /// The register allocator's high-water mark (the plan's Decisions
    /// section: "the chunk records its high-water mark"). Always `0` here:
    /// nothing allocates a register until Task 4's allocator exists.
    /// `Interp::run_chunk` reserves this many registers before running a
    /// chunk and truncates them away on the way out.
    registers: u16,
}
