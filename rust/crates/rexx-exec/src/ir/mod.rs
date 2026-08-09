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
//! instruction. This task promotes none of them, so every op is
//! `Op::Generic`, which delegates the instruction at its index back to the
//! tree-walker's own clause unit (`Interp::step`, `clause.rs`). A later task
//! promotes a construct by teaching `compile` to emit something other than
//! `Generic` for it, never by changing what `Generic` means.
//!
//! `Interp::chunk_for` (`plan.rs`, beside `plan_for`, under the same
//! `BodyKey`) is the cache: a body compiles once, on first entry, and the
//! result is kept for every later one. Nothing in this crate calls it yet --
//! Task 3's driver is its first production caller -- so this module and
//! `plan.rs`'s new cache fields are exercised only by the tests in
//! `golden_tests.rs` until then, which is why several items below carry an
//! `#[allow(dead_code)]` rather than a caller.

mod compile;
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
    #[allow(dead_code, reason = "Task 3's driver executes this; nothing does yet")]
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
#[allow(
    dead_code,
    reason = "Task 3's engine selection is this type's first production reader"
)]
pub(crate) struct ChunkTooLarge {
    /// What overflowed. Always `"op stream"` today: op indices are `u32`
    /// and this task allocates no registers, so nothing else can overflow
    /// before Task 4's register allocator exists to overflow `u16::MAX`.
    pub(crate) what: &'static str,
}

/// One body's compiled instruction stream, cached on `Interp` under the same
/// `BodyKey` its `Plan` is (`Interp::chunk_for`, in `plan.rs`).
pub(crate) struct Chunk {
    /// One entry per instruction, `Op::Generic` for every one of them in
    /// this task -- D21's "every instruction compiles, nothing refuses" is
    /// a claim about instructions, not about promotion.
    #[allow(
        dead_code,
        reason = "Task 3's driver executes a chunk's ops; nothing does yet"
    )]
    ops: Vec<Op>,
    /// Instruction index -> op index into `ops`. One entry per instruction,
    /// in order, plus one final entry at `ops.len()`, pushed *after* the
    /// loop that fills the rest: `run_bounded`'s absorption guard is
    /// inclusive, so a construct's resume point can be one past its last
    /// instruction, and a map that stopped at `len - 1` would panic there
    /// instead of failing loudly at compile.
    #[allow(
        dead_code,
        reason = "Task 3's driver reads this to resume mid-body; nothing does yet"
    )]
    op_of: Vec<u32>,
    /// The register allocator's high-water mark (the plan's Decisions
    /// section: "the chunk records its high-water mark"). Always `0` here:
    /// nothing allocates a register until Task 4's allocator exists. Task
    /// 3's driver reserves this many registers before running a chunk.
    #[allow(
        dead_code,
        reason = "Task 3's driver reserves this many registers; nothing does yet"
    )]
    registers: u16,
}
