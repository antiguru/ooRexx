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

use crate::trace::ChunkTrace;

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
    /// **This is the clause's unconditional half, and [`Op::TraceClause`] is
    /// the conditional one.** Everything this op runs is semantics rather than
    /// trace: the clause line `SIGL`, condition objects and syntax error
    /// messages all read, the clause boundary a queued `CALL ON` handler is
    /// delivered at, the GC temps frame, and the failing clause's own site.
    /// None of it may be elided with the echo, which is why the split is two
    /// ops rather than one op with a flag.
    ///
    /// **No `Generic` or `Loop` op may sit inside `(here, end)`**, which
    /// `compile` asserts: both run a whole clause through
    /// `step_in_temps_frame`, which echoes the clause itself, and the echo is
    /// not idempotent.
    Clause { index: u32, end: u32 },
    /// Echoes the `*-*` line of the clause of the instruction at `index`.
    ///
    /// **The conditional half of [`Op::Clause`], and its presence in the
    /// stream *is* the decision.** `compile` emits it exactly when
    /// `ChunkTrace::echoes` answers yes for that instruction, so a chunk
    /// compiled under a setting that echoes nothing has no such op and its
    /// clauses pay nothing at all for trace -- no gate, no `clause_site`
    /// allocation, no line to skip.
    ///
    /// **Only valid inside a [`Op::Clause`] region**, and it is the region's
    /// first op: the echo is the first thing the tree-walker's own clause unit
    /// does inside `in_clause`, before anything the clause computes.
    TraceClause { index: u32 },
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
    /// why [`Op::JumpUnless`] can test it without repeating the validation. A
    /// `SELECT CASE` has one too, slot `0`, and its value is whatever the
    /// expression came to, kept for every `WHEN CASE` of that `SELECT` to be
    /// compared against.
    EvalExpr { index: u32, slot: u32, dst: u16 },
    /// Hands the `SELECT` at `index` the text an **absorbed** `WHEN CASE`
    /// compares against, from register `case`, or clears it for a plain
    /// `SELECT` that has no `CASE` expression at all.
    ///
    /// `Interp::current_case_text` is the one hand-off an absorbed `WHEN CASE`
    /// has (`lib.rs`'s own doc comment on the field), and this op is where the
    /// tree-walker's own `Select` arm sets it: **after** the header clause,
    /// not inside it, so a `CALL ON` handler delivered at that clause's
    /// boundary cannot be the last writer.
    ///
    /// **Outside a [`Op::Clause`] region on purpose**, for exactly that
    /// reason. A listed `WHEN CASE` never reads it: it is handed the same
    /// register directly by [`Op::WhenTest`].
    SelectCaseText { index: u32, case: Option<u16> },
    /// Tests the listed `WHEN`/`WHEN CASE` at `index` and stores whether it
    /// holds in register `dst`, as the Rexx logical value [`Op::JumpUnless`]
    /// reads back.
    ///
    /// `case` is the register the enclosing `SELECT CASE` left its own value
    /// in, and `None` for a plain `SELECT`. It is a register of the
    /// **enclosing** scope rather than of this clause's, because the last
    /// `WHEN` of a `SELECT` is tested after every earlier `WHEN`'s branch has
    /// already run (the plan's Decisions section: "a construct whose state
    /// outlives its member clauses allocates in the enclosing scope").
    ///
    /// **Only valid inside a [`Op::Clause`] region**, whose clause is this
    /// `WHEN`'s own -- the echo, the value indent its comparison lines trace
    /// at, the boundary and both failure sites are that region's.
    WhenTest {
        index: u32,
        case: Option<u16>,
        dst: u16,
    },
    /// Closes the branch control has just run off the end of: the clause
    /// boundary a promoted construct owes where the tree-walker's own wrapper
    /// around the whole arm runs one.
    ///
    /// `Interp::end_promoted_branch`'s doc comment has the program that says
    /// this is not a spare boundary -- a `CALL ON` handler whose own `RAISE`
    /// leaves a second trap queued behind it, which nothing else is left to
    /// deliver.
    ///
    /// **Emitted at the branch's end, not at the construct's**, and reached
    /// only by falling out of the branch: it sits at the `op_of` entry of the
    /// instruction the branch ends before, where the `IF`'s own false path
    /// (`PatchKind::Enter`) does not land. A `SELECT` needs no op for it,
    /// because its branches are closed by their frames.
    EndBranch,
    /// Opens a frame over the branch of the listed `WHEN` at `when`, which
    /// belongs to the `SELECT` at `select`.
    ///
    /// The frame is what the tree-walker's own `run_bounded(...)?` followed by
    /// `leave_select` is when it is flattened: a `LEAVE`/`ITERATE` escaping
    /// this branch has to reach that `leave_select` rather than the enclosing
    /// range, and in a flat stream there is no Rust call frame between the two
    /// to make that happen.
    EnterWhen { select: u32, when: u32 },
    /// Opens a frame over the `OTHERWISE` branch of the `SELECT` at `select`,
    /// which is [`Op::EnterWhen`]'s counterpart for the branch no `WHEN`
    /// owns.
    ///
    /// A separate op rather than a flag, because the two branches leave
    /// differently: `OTHERWISE`'s restores `Interp::indent_offset` on the way
    /// out and a `WHEN`'s does not, which is `run_otherwise`'s own split from
    /// `Select`'s arm exactly.
    ///
    /// **It sits at the `OTHERWISE`'s own `op_of` entry, in front of the
    /// marker's op**, so that both ways of arriving there open the frame: the
    /// scan falling past its last `WHEN`, and an absorbed `WHEN CASE`'s
    /// escape landing on the marker ([`crate::run::SelectEscape::Otherwise`]).
    EnterOtherwise { select: u32 },
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

/// One body's compiled instruction stream, cached on `Interp` under its
/// `Plan`'s `BodyKey` **and the [`ChunkTrace`] it was compiled under**
/// (`Interp::chunk_for`, in `plan.rs`).
pub(crate) struct Chunk {
    /// The trace setting this stream's ops were emitted for, which is half of
    /// the key it is cached under and the thing the driver compares the
    /// setting in force against.
    ///
    /// **Kept on the chunk rather than only in the cache key**, because a
    /// `TRACE` executed while this chunk is running changes the setting
    /// without going anywhere near the cache -- `Interp::run_ops` reads this
    /// to notice, and a chunk that could not say what it was compiled for
    /// would leave that undetectable.
    trace: ChunkTrace,
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

    /// The setting this chunk's trace ops were emitted for.
    fn trace(&self) -> ChunkTrace {
        self.trace
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
