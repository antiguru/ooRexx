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

use std::cell::Cell;

use rexx_core::FrameId;
use rexx_parse::{Operator, PrefixOp, SymbolId};

use crate::error::Raised;
use crate::eval::SymbolRead;
use crate::run::{
    HeaderRole, QueueKeyword, Resolved, ReturnKeyword, raised_if_not_logical,
    raised_when_not_logical,
};
use crate::trace::ChunkTrace;

mod compile;
mod drive;
pub(crate) use compile::compile;

// `render`, the golden-test serialiser, is a test-only rendering of the op
// stream: nothing the interpreter does at run time reads a chunk back as text.
// Gated here rather than carrying a permanent `#[allow(dead_code)]`, so a
// production caller for it would not compile rather than passing unnoticed.
#[cfg(test)]
mod golden;

#[cfg(test)]
mod golden_tests;

// The corpus-wide statement of the same promotion set `golden_tests` pins per
// construct: an invariant derived from each body's own instructions rather
// than a committed stream, so it holds over every corpus program without
// anything to regenerate. Its module doc has what it cannot see.
#[cfg(test)]
mod corpus_shape_tests;

/// **The stream's width, asserted so that a change to it is a compile error.**
/// Every op in every chunk is as wide as the widest variant, so the width is a
/// fact about the whole array rather than about the variant that sets it, and a
/// variant that outgrows this number grows every op there is. That is what this
/// line catches, at the moment it happens, rather than leaving it to a
/// measurement somebody has to take again. The widest payload has tail padding
/// for the discriminant to sit in, which is why sixteen is an equality and not
/// a bound.
///
/// **What sixteen costs was measured, and on these axes it was nothing.** The
/// record's entry 24 sat a twelve-byte head build against a control carrying a
/// dead variant at that same width and against a sixteen-byte build, nine
/// rounds per axis, because widening an enum by adding a variant moves two
/// things at once and a two-arm reading cannot tell them apart. Every axis's
/// width column came out negative, and the cost that reading had charged to the
/// width belongs to *having an extra variant* instead. **The entry says in its
/// own voice that it did not build the change that landed here**: both its wide
/// arms widen the enum by adding a dead variant, where widening a variant
/// already present adds none -- and it calls the width column the right
/// prediction for that case. So sixteen rests on the sitting plus that
/// prediction, and not on an argument from cache lines.
const _: () = assert!(size_of::<Op>() == 16);

/// One step in a compiled stream.
///
/// **Every op that runs a clause carries its own instruction index**, and
/// that is what lets it sit inside a region a jump lands in: the driver's
/// program counter is an *op* index, so there is no instruction counter
/// walking alongside it to read the index off. `Op::Generic` is what an
/// unpromoted instruction compiles to; a promoted construct compiles to a
/// `Clause` region followed by whatever jumps its control flow needs.
pub(crate) enum Op {
    /// Delegates the instruction at `index` back to the tree-walker's own
    /// clause unit, which runs the whole clause.
    Generic { index: u32 },
    /// Echoes the `>K>` line of one `DO`/`LOOP` header value, from register
    /// `src`, under the tag [`HeaderRole`] gives it.
    ///
    /// **A separate op from whatever produced the value, and that is the whole
    /// reason this construct waited for the trace ops.** A loop header
    /// interleaves evaluation and emission -- it evaluates `TO`, echoes it,
    /// evaluates `BY`, echoes it, in the order the keywords were written -- so
    /// an op that only evaluated could not reproduce the ordering, and an op
    /// that did both would be the whole evaluate-and-trace unit rather than the
    /// general expression op the later tasks need. With the emission its own
    /// op, **the order in `Controlled::order` is the order of ops**.
    ///
    /// **Only valid inside a [`Op::Clause`] region**, whose clause is the
    /// `DO`/`LOOP`'s own: the indent it echoes at is that clause's.
    ///
    /// Emitted unconditionally rather than under [`ChunkTrace`]'s decision the
    /// way [`Op::TraceClause`] is, because the gate this line answers to is
    /// `trace_mode().results` rather than the clause echo's, and a second
    /// compiled emission decision would need a staleness rule of its own. What
    /// the op form buys here is the ordering, not the elision.
    TraceKeyword { role: HeaderRole, src: u16 },
    /// Validates the `DO`/`LOOP` header value in register `src` for the role
    /// it plays and files it for [`Op::LoopRun`].
    ///
    /// **Its own op, in front of the next value's evaluation**, because that
    /// ordering is observable: `do i = 1 to 'a' by zf()` raises 41.1 on `TO`
    /// and never calls `zf`, so a stream that gathered every value first and
    /// validated afterwards would call it.
    ///
    /// `Interp::accept_header_value` is the one implementation of what each
    /// role requires, entered from here and from the tree-walker's own
    /// `eval_loop_header`.
    ///
    /// **Only valid inside a [`Op::Clause`] region**, and the region must be
    /// the one [`Op::LoopRun`] closes: the values it files have nowhere else to
    /// go.
    LoopHeaderValue { role: HeaderRole, src: u16 },
    /// Runs the `DO`/`LOOP` at `index` from the header values the ops before it
    /// filed, with **its body's clauses stepped from this chunk**.
    ///
    /// The construct itself is resolved by the same
    /// `Interp::run_loop_with_header` the tree-walker reaches -- every
    /// iteration, `WHILE`/`UNTIL`, the `LEAVE`/`ITERATE` label search and every
    /// trace echo are one implementation, entered from both engines, rather
    /// than two. What this op changes is the one line inside it that was
    /// engine-specific: the body's clauses are stepped from the compiled stream
    /// instead of straight into the tree-walker's clause unit, so an
    /// instruction inside a loop is reachable from the stream at all.
    ///
    /// **The last op of a [`Op::Clause`] region, and inside it rather than
    /// after it.** The whole loop runs inside the `DO` clause exactly as it does
    /// on the tree-walker, so the clause's temps frame stays open across every
    /// pass -- which is what roots the per-pass temporaries a `WHILE` or `UNTIL`
    /// test pushes -- and the clause's boundary is where the tree-walker has it.
    /// A header value that outlives the header is rooted by its **register**
    /// rather than by that frame: `compile` allocates the header's registers in
    /// the enclosing scope and releases them past the loop's `END`, so a
    /// `DO OVER`'s target stays addressed for as long as `LoopState` holds it.
    /// The body's own clauses are not region ops: they are reached through
    /// `run_bounded`, which re-enters the driver for the body's range, so no op
    /// inside the region opens a clause.
    ///
    /// **A `DO`/`LOOP` this crate refuses reaches this op too**, with an empty
    /// header in front of it, because the refusal is
    /// `run_loop_with_header`'s -- `run.rs`'s `loop_header_plan` decides it for
    /// both engines, before anything is evaluated, and a compiler that refused
    /// on its own would be a second copy of that decision.
    LoopRun { index: u32 },
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
    /// **No `Generic` op may sit inside `(here, end)`**, which `compile`
    /// asserts: it runs a whole clause through `step_in_temps_frame`, which
    /// echoes the clause itself, and the echo is not idempotent. An op that
    /// runs clauses belonging to something other than this region's own
    /// instruction is a different thing and is allowed -- [`Op::LoopRun`]
    /// reaches the body's clauses through `run_bounded` and [`Op::Call`]
    /// reaches a callee's through `run_activation`, each re-entering a driver
    /// rather than stepping a clause here.
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
    /// Runs the call at `path` inside expression `slot` of the instruction at
    /// `index`, into register `dst`, through the resolution site `site` keeps.
    ///
    /// **The one thing this does that [`Op::EvalExpr`] does not is skip
    /// `resolve_call`.** Everything else is the same code on the same node:
    /// the argument loop with its `>A>` lines, the depth guard, the activation
    /// bookkeeping and the three `Ended` arms are `invoke_call`'s and
    /// `eval_call_resolved`'s, entered from here exactly as `eval.rs` enters
    /// them. That is deliberate rather than minimal -- a call's observable
    /// surface is large (`SIGL`, the activation level, 44.1 for a routine that
    /// returns nothing) and none of it is worth a second implementation for
    /// the sake of one lookup.
    ///
    /// **`slot` and `path` together are the address**, and `path` is what
    /// makes it finer than an expression slot: it is the route down from slot
    /// `slot`'s own root, [`NodePath::ROOT`] naming that root itself, and
    /// `Interp::chunk_node_at` is the descent that resolves it. A node the
    /// descent cannot reach, or one that is not a call when it arrives, is
    /// `Loud::call_op_off_its_node` rather than a panic.
    ///
    /// **An address reaching a call is necessary and not sufficient.**
    /// `native_shape` decides for a whole expression slot at once, so a call
    /// takes one of these when every node of its slot compiles natively *and*
    /// the address reaches the call; a single sibling term with no op of its
    /// own leaves the whole slot on [`Op::EvalExpr`] and this call with
    /// nothing. `golden_tests`'s `a_call_promotes_at_the_root_and_below_it`
    /// states where an op is taken,
    /// `a_call_nested_past_the_paths_width_leaves_the_slot_general` states the
    /// address running out, and
    /// `the_value_shapes_outside_the_native_set_stay_general` states the
    /// sibling taking the call down with it.
    ///
    /// **It owes the `>F>` line itself**, through [`Op::TraceFunction`] behind
    /// it, for the reason every native op owes its own echo: `eval`'s
    /// post-order hook is what emits it and this op does not go through
    /// `eval`.
    ///
    /// **`slot` and `site` are `u16` where [`Op::EvalExpr`]'s slot is `u32`**,
    /// and that is the width budget deciding rather than a preference:
    /// widening either to `u32` makes this variant 20 bytes and the assertion
    /// above [`Op`] fails. Both bounds are guarded rather than assumed -- a
    /// chunk that would exceed either is refused with `ChunkTooLarge`, the
    /// same answer every other index in this module gives.
    CallExpr {
        index: u32,
        slot: u16,
        path: NodePath,
        site: u16,
        dst: u16,
    },
    /// The `>F>` line [`Op::CallExpr`] owes, for the value now in `src`.
    ///
    /// **It carries its call op's address rather than only its register**,
    /// because the line is traced against the node: `trace_intermediate` reads
    /// the expression to decide the tag it prints under, so an echo addressing
    /// some other node would put the right value on the wrong line.
    /// `compile::assert_call_echoes_follow_their_op` is what checks the
    /// position, the register and the whole address rather than assuming them.
    ///
    /// **Only valid inside a [`Op::Clause`] region**, and immediately behind
    /// the call whose register it reads: `eval.rs` emits this post-order, with
    /// the value in hand, so a call inside an operand prints its line before
    /// the operator holding it prints one.
    TraceFunction {
        index: u32,
        slot: u16,
        path: NodePath,
        src: u16,
    },
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
    /// **The whole job in one op**, which is what a `WHEN CASE` needs and what
    /// a plain `WHEN` falls back to: it evaluates, traces and (for a plain
    /// `WHEN`) validates, through `Interp::scan_when`. A plain `WHEN` whose
    /// condition `native_shape` accepts compiles to that condition's own ops
    /// and an [`Op::Condition`] instead, and no op of this kind is emitted for
    /// it. A `WHEN CASE` never takes that route: its values are compared
    /// against the enclosing `SELECT CASE`'s text rather than validated as a
    /// logical value, which is not what [`Op::Condition`] does.
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
    /// Loads constant `konst` of [`Chunk::consts`] into register `dst`.
    ///
    /// **The phase's first native expression op**, and the whole of what makes
    /// it native is that `eval.rs` is not entered: the literal's bytes come
    /// from the chunk's own constant table and `Interp::literal` turns them
    /// into the value, which is the same function `eval_node`'s own `Literal`
    /// arm calls.
    ///
    /// **`Interp::literal` and not `Interp::text`**, which is where the tagged
    /// small integer a literal like `'1'` starts as comes from. That is a
    /// representation choice rather than a behaviour -- `Interp::literal`'s own
    /// doc comment says a `SmallInt` reaches every consumer in the crate
    /// already -- so what the wrong call here would cost is an allocation per
    /// pass, not an answer.
    ///
    /// **This op has no performance evidence in either direction, and that is
    /// measured rather than an omission.** `push_native`'s literal arm is what
    /// emits it, so a literal in any slot `compile` offers that function
    /// compiles to one -- and no registered benchmark axis executes one
    /// **inside a measured loop**: counting
    /// executions on each axis at `n` and at `2n` gives a figure independent of
    /// `n` every time -- one execution on `bench-programs/emptyloop.rex` (its
    /// closing `say 'done'`) and one on `bench-programs/strings.rex` (the
    /// subject string it assigns before its loop), zero on `varlookup`, `arith`,
    /// `compound`, `alloc4c` and `startup`. So **no measurement of those axes
    /// says anything about this op**, and a per-clause cost measured on them
    /// belongs to the clause machinery around it -- [`Op::Clause`],
    /// [`Op::TraceClause`], [`Op::EvalExpr`], [`Op::Store`] -- rather than
    /// here.
    ///
    /// **It emits nothing, and [`Op::TraceLiteral`] is why that is safe.**
    /// `eval.rs` emits a literal's `>L>` line as a side effect of evaluating
    /// it, so an op that only loads a value silently drops that line -- and
    /// every line after it still matches, which is what let the mechanics
    /// spike ship the defect. The emission is a separate op for exactly the
    /// reason [`Op::TraceKeyword`] is one: an op that both computed and
    /// emitted could not be ordered against its neighbours, and the ordering
    /// is what a stream buys.
    ///
    /// **Only valid inside a [`Op::Clause`] region**, whose clause owns the
    /// value indent the line after this one traces at.
    ///
    /// **`ExprKind::Constant` does not compile to this**, and the reason is
    /// where its bytes live rather than what it is: a constant symbol's value
    /// is its own upcased spelling, which is in the symbol table, and
    /// `compile` deliberately takes nothing but the body, the plan and the
    /// trace setting -- `Interp::chunk_for`'s own doc comment turns that into
    /// the cache key's completeness. [`Op::LoadConstant`] is what it compiles
    /// to instead, naming the symbol and reading its spelling where the table
    /// is in hand.
    Const { dst: u16, konst: u32 },
    /// Loads the value of the constant symbol `symbol` into register `dst`:
    /// its own upcased spelling, which is observable rather than incidental --
    /// `say 1e5` prints `1E5`.
    ///
    /// **[`Op::Const`] with the bytes somewhere else, and that is the whole of
    /// the difference.** Both build their value through `Interp::literal`,
    /// which is what `eval_node`'s own `Literal` and `Constant` arms each call;
    /// what separates them is that a quoted literal's bytes are in the node, so
    /// the chunk can intern them, and a constant symbol's are in the symbol
    /// table, which `compile` does not have. So the symbol travels in the op
    /// and the spelling is read where `code` is in hand -- the same shape
    /// [`Op::Load`] uses for the same reason.
    ///
    /// **Its echo is [`Op::TraceLiteral`], not an op of its own**, because the
    /// line is the same one: `trace_intermediate` sends `Literal` and
    /// `Constant` alike to `echo_literal`, both `>L>` with no tag.
    ///
    /// **This is what an unquoted number in an expression is**, which is why
    /// promoting arithmetic needed it: `zx + 1` holds no `ExprKind::Literal` at
    /// all, and with the `1` left unpromotable the whole expression falls to
    /// [`Op::EvalExpr`] and no arithmetic op is emitted for it.
    ///
    /// **Only valid inside a [`Op::Clause`] region**, whose clause owns the
    /// value indent the line after this one traces at.
    LoadConstant { symbol: SymbolId, dst: u16 },
    /// Echoes the `>L>` line of the literal or constant symbol in register
    /// `src`.
    ///
    /// **A separate op from the [`Op::Const`] or [`Op::LoadConstant`] that
    /// loaded it**, which is that op's own doc comment. One echo for both,
    /// because `trace_intermediate` sends both node kinds to the same
    /// `echo_literal`. Emitted unconditionally rather than under
    /// [`ChunkTrace`]'s decision the way [`Op::TraceClause`] is, for the
    /// reason [`Op::TraceKeyword`] gives: the gate this line answers to is
    /// `trace_mode().intermediates`, which [`ChunkTrace`] does not carry, and
    /// widening it would put a second emission decision under a staleness rule
    /// of its own. What the op form buys here is that the line exists at all,
    /// not its elision.
    ///
    /// **Only valid inside a [`Op::Clause`] region**, and immediately behind
    /// the `Const` whose register it reads: `eval.rs` emits this line
    /// post-order, with the value in hand, so a line emitted anywhere else
    /// would print in the wrong place relative to the value lines around it.
    /// `compile::assert_literal_echoes_follow_their_load` is what makes that an
    /// assertion rather than a sentence, and it checks the register as well as
    /// the position: an echo behind the wrong load lands in the right place with
    /// the wrong value in it.
    ///
    /// **It runs exactly where [`Op::Const`] does, so it has no performance
    /// evidence either** -- see that op's own note for the measurement.
    TraceLiteral { src: u16 },
    /// Loads the value of the bare symbol `symbol` into register `dst`, by the
    /// read [`SymbolRead`] names.
    ///
    /// **Native in the sense [`Op::Const`] is: `eval.rs` is not entered.**
    /// `Interp::read_symbol` is the whole of what `eval_node`'s own
    /// `Variable`/`Stem`/`Compound` arms do, entered from here and from there,
    /// so the derived name an unset read answers, the `Body::Stem` a bare stem
    /// miss allocates, the tail key a compound resolves and the `NOVALUE`
    /// condition two of the three raise are one implementation rather than a
    /// second one beside it. What is left out is `eval`'s own wrapper: the
    /// depth bookkeeping D19 needs, which a bare symbol cannot recurse
    /// through, and the post-order trace hook, which is the op below.
    ///
    /// **Emitted wherever the descent reaches a bare symbol -- as a slot's
    /// whole expression, and inside one.** `zv + 1` compiles to this op and
    /// the operator applied to it, because an operator's operands are compiled
    /// to ops of their own. A symbol the descent never walks to gets none -- a
    /// call's arguments go back through `eval.rs` -- and neither does one
    /// inside a slot that falls to [`Op::EvalExpr`] entire, which is the whole
    /// slot's choice rather than this node's.
    ///
    /// `at` is the slot the plan already resolved this symbol to, which is the
    /// one thing this op knows that the tree-walker's own read has to work out
    /// -- see [`PlanSlot`] for what carries it and why a compound never has
    /// one.
    ///
    /// **Only valid inside a [`Op::Clause`] region**, whose clause owns the
    /// value indent the line after this one traces at, and the failure site a
    /// `NOVALUE` raised here is reported against.
    Load {
        symbol: SymbolId,
        read: SymbolRead,
        at: PlanSlot,
        dst: u16,
    },
    /// Echoes the `>V>` line of the read in register `src`, and the `>C>` line
    /// a compound read owes in front of it.
    ///
    /// **A separate op from the [`Op::Load`] that read it**, for the reason
    /// [`Op::TraceLiteral`] is separate from [`Op::Const`]: the load emits
    /// nothing, `eval.rs` emits these lines as a side effect of *evaluating*
    /// the expression, and a promoted clause with no such op drops them while
    /// every line after them still matches. Emitted unconditionally rather
    /// than under [`ChunkTrace`]'s decision, because the gate these lines
    /// answer to is `trace_mode().intermediates`, which [`ChunkTrace`] does
    /// not carry.
    ///
    /// **Only valid inside a [`Op::Clause`] region**, and immediately behind
    /// the `Load` whose register it reads and whose symbol and read kind it
    /// repeats: `eval.rs` emits these post-order, with the value in hand.
    /// `compile::assert_read_echoes_follow_their_load` is what makes that an
    /// assertion rather than a sentence, and it checks all three -- an echo
    /// behind the wrong load lands in the right place with the wrong value or
    /// the wrong tag in it.
    TraceRead {
        symbol: SymbolId,
        read: SymbolRead,
        src: u16,
    },
    /// Computes `lhs op rhs` into register `dst`, for the operators
    /// `eval::is_arithmetic` names.
    ///
    /// **Native in the sense [`Op::Const`] and [`Op::Load`] are: `eval.rs` is
    /// not entered, and neither is it for the operands.** An expression
    /// compiles to this op only when *both* its operands compile to native ops
    /// too, so `za + zb * 4` runs with no `eval` recursion in it, while a slot
    /// holding one node with no op of its own -- a `.NIL`, or a call the
    /// address does not reach -- is one [`Op::EvalExpr`] entire, because
    /// `EvalExpr` names an expression *slot* of an instruction, which a
    /// subexpression is not.
    ///
    /// The arithmetic itself is `Interp::arith_small_int` and
    /// `Interp::arith_general`, entered from here and from
    /// `Interp::eval_arithmetic`, so the operand conversion, the arithmetic
    /// operators' own `rexx-num` calls, the 41.1 a nonnumeric operand raises
    /// and the 26.8 a `**` exponent raises are one implementation rather than
    /// a second one beside it. **What this op adds is the order they are tried
    /// in, which [`Chunk::hints`] makes a per-site decision** -- see
    /// [`PatchSlot`] for what a hint may and may not do.
    ///
    /// **`lhs` may be `dst`, and usually is**: the left operand is evaluated
    /// into the destination register and the right into a scratch one above
    /// it, so a left-nested chain reuses two registers however deep it runs
    /// rather than one per operator. Both sources are read before the
    /// destination is written, which is what makes that safe.
    ///
    /// **It emits nothing, and [`Op::TraceOperator`] is why that is safe** --
    /// [`Op::Const`]'s own doc comment has the mechanism, and `>O>` is the
    /// line this op's evaluation used to emit as a side effect.
    ///
    /// **Only valid inside a [`Op::Clause`] region**, whose clause owns the
    /// value indent the line after this one traces at, and the failure site a
    /// 41.1 raised here is reported against.
    Arith {
        op: Operator,
        /// This site's own slot in [`Chunk::hints`] -- **not** a position in
        /// the op stream. The table is dense over the ops that can specialise,
        /// keyed off the op the way [`Chunk::consts`] is, so a stream carries
        /// no entry for the ops that never quicken and the driver's own loop
        /// needs no counter walking beside it to find one.
        hint: u32,
        lhs: u16,
        rhs: u16,
        dst: u16,
    },
    /// Computes `lhs op rhs` into register `dst`, for the concatenation,
    /// comparison and logical operators -- every operator `eval::
    /// is_native_binary` names and `eval::is_arithmetic` does not.
    ///
    /// **Native in the sense [`Op::Const`] and [`Op::Load`] are: `eval.rs` is
    /// not entered, and neither is it for the operands**, on [`Op::Arith`]'s
    /// own terms and under the same whole-expression decision.
    ///
    /// The operator itself is `Interp::apply_binary`, entered from here and
    /// from `eval_node`'s own binary arm, so the join `Blank` puts one space
    /// in, the comparison settings a strict operator never reads, the 34.901 a
    /// non-logical operand raises and the substitution it carries are one
    /// implementation rather than a second one beside it.
    ///
    /// **What this op does not have is [`Op::Arith`]'s hint, and that is the
    /// whole of the difference between the two.** Arithmetic has two paths to
    /// choose between and these operators have one, so there is nothing here
    /// for a per-site decision to decide. [`Chunk::hints`] is dense over the
    /// ops that *can* specialise: a slot reserved for this op and never read
    /// would shift every later arithmetic site's own index by one, which is a
    /// wrong answer rather than a wasted word.
    ///
    /// **`lhs` may be `dst`, and usually is**, for [`Op::Arith`]'s reason and
    /// with the same guarantee behind it: both sources are read before the
    /// destination is written.
    ///
    /// **It emits nothing, and [`Op::TraceOperator`] is why that is safe** --
    /// [`Op::Const`]'s own doc comment has the mechanism, and `>O>` is the
    /// line this op's evaluation used to emit as a side effect.
    ///
    /// **Only valid inside a [`Op::Clause`] region**, whose clause owns the
    /// value indent the line after this one traces at, and the failure site a
    /// 34.901 raised here is reported against.
    Binary {
        op: Operator,
        lhs: u16,
        rhs: u16,
        dst: u16,
    },
    /// Echoes the `>O>` line of the operator result in register `src`.
    ///
    /// **A separate op from the [`Op::Arith`] or [`Op::Binary`] that computed
    /// it**, for the
    /// reason [`Op::TraceLiteral`] is separate from [`Op::Const`]: the
    /// computation emits nothing, `eval.rs` emits this line as a side effect of
    /// *evaluating* a binary node, and a promoted clause with no such op drops
    /// it while every line after it still matches. Emitted unconditionally
    /// rather than under [`ChunkTrace`]'s decision, because the gate this line
    /// answers to is `trace_mode().intermediates`, which [`ChunkTrace`] does
    /// not carry.
    ///
    /// `op` is repeated here rather than read off the operation behind it,
    /// because the tag is the operator's own spelling and an echo carrying a
    /// different one lands in the right place with the wrong tag in it.
    /// `compile::assert_operator_echoes_follow_their_op` is what checks the
    /// position, the register and the operator rather than assuming them.
    ///
    /// **Only valid inside a [`Op::Clause`] region**, and immediately behind
    /// the operation whose register it reads: `eval.rs` emits this post-order,
    /// with the value in hand, so an inner operator's line precedes the outer
    /// one's exactly as the ops do.
    TraceOperator { op: Operator, src: u16 },
    /// Computes `op src` into register `dst`, for the prefix operators `+`,
    /// `-` and `\`.
    ///
    /// **Native in the sense [`Op::Const`] and [`Op::Load`] are: `eval.rs` is
    /// not entered, and neither is it for the operand**, on [`Op::Arith`]'s
    /// own terms and under the same whole-expression decision.
    ///
    /// The operator itself is `Interp::apply_prefix`, entered from here and
    /// from `Interp::eval_prefix`, so the rounding `-za` takes from the
    /// activation's `DIGITS`, the 41.1 a nonnumeric operand raises and the
    /// 34.901 `\` raises on a non-logical one are one implementation rather
    /// than a second one beside it.
    ///
    /// **`src` may be `dst`**, for [`Op::Arith`]'s reason and with the same
    /// guarantee behind it: the source is read before the destination is
    /// written.
    ///
    /// **It emits nothing, and [`Op::TracePrefix`] is why that is safe** --
    /// [`Op::Const`]'s own doc comment has the mechanism, and `>P>` is the
    /// line `eval.rs` emits as a side effect of evaluating a prefix node.
    ///
    /// **Only valid inside a [`Op::Clause`] region**, whose clause owns the
    /// value indent the line after this one traces at, and the failure site a
    /// 41.1 raised here is reported against.
    Prefix { op: PrefixOp, src: u16, dst: u16 },
    /// Echoes the `>P>` line of the prefix-operator result in register `src`.
    ///
    /// **A separate op from the [`Op::Prefix`] that computed it**, for the
    /// reason [`Op::TraceOperator`] is separate from [`Op::Binary`]: the
    /// computation emits nothing, `eval.rs` emits this line as a side effect of
    /// *evaluating* a prefix node, and a promoted clause with no such op drops
    /// it while every line after it still matches.
    ///
    /// **A different op from [`Op::TraceOperator`], because `>P>` is a
    /// different line from `>O>`** -- `Interp::trace_prefix_op` and
    /// `Interp::trace_operator` are the two emissions, and that function's own
    /// doc has the C++ pair behind them. An echo that reused the binary op
    /// would put the right value on the wrong line, which is also why this
    /// carries a `PrefixOp` where that one carries an `Operator`.
    ///
    /// `op` is repeated here rather than read off the operation behind it, for
    /// [`Op::TraceOperator`]'s reason, and
    /// `compile::assert_prefix_echoes_follow_their_op` is what checks the
    /// position, the register and the operator rather than assuming them.
    ///
    /// **Only valid inside a [`Op::Clause`] region**, and immediately behind
    /// the operation whose register it reads: `eval.rs` emits this post-order,
    /// with the value in hand, so an inner operator's line precedes the outer
    /// one's exactly as the ops do.
    TracePrefix { op: PrefixOp, src: u16 },
    /// Writes register `src` through the target of the `Assignment` at
    /// `index`, and traces the write.
    ///
    /// `Interp::assign_evaluated` is the whole of what `step`'s own
    /// `Assignment` arm does once its value is computed, entered from here and
    /// from there -- so the `>>>` result line, the `>C>` resolved-name line a
    /// compound target announces, the `>=>` write line, and the stem and
    /// compound-tail dispatch itself (`Interp::assign_expr_target`) are one
    /// implementation rather than a second one beside it.
    ///
    /// `at` is the slot the plan already resolved a **simple**-variable target
    /// to, [`PlanSlot::UNRESOLVED`] for every other target shape, and it is
    /// handed to `assign_expr_target` as an argument rather than acted on here
    /// -- see that function for why the alternative is the defect the
    /// dual-engine gate exists to catch.
    ///
    /// **Only valid inside a [`Op::Clause`] region**, whose clause is this
    /// assignment's own: the indent every line above traces at and the
    /// boundary the write precedes both belong to it.
    Store { index: u32, at: PlanSlot, src: u16 },
    /// Prints the `SAY` at `index` from register `src`, or a blank line when
    /// it has no expression at all.
    ///
    /// `Interp::say_evaluated` is the whole of what `step`'s own `Say` arm
    /// does once its expression is computed, entered from here and from there.
    /// The bare form is a blank line **and** a `>>>` line for the null string,
    /// not a skipped clause, which is why `src` is an `Option` rather than a
    /// register holding an empty value: the two are the same output and only
    /// one of them is what the instruction says.
    ///
    /// `index` is read only by a debug assertion that the op still describes
    /// the instruction it was emitted for -- the line and the indent this
    /// clause prints at come from `Interp::clause_state`, which the enclosing
    /// [`Op::Clause`] region's own clause unit set.
    ///
    /// **Only valid inside a [`Op::Clause`] region**, for that reason.
    Say { index: u32, src: Option<u16> },
    /// Ends the activation at `index` with the value in register `src`, or
    /// with no value at all for the bare form, and answers the `Flow` its
    /// keyword calls for.
    ///
    /// `Interp::returned_value` is the whole of what `step`'s own `Return` and
    /// `Exit` arms do once their expression is computed, entered from here and
    /// from there -- so the rooting, the `>>>` line and the choice of `Flow`
    /// are one implementation rather than a second one beside it.
    ///
    /// **`src` is an `Option` because the two forms are different
    /// instructions, not the same one with an empty value**, and the
    /// difference is measured on the oracle: after `call zempty` where
    /// `zempty:` is `return ''`, the caller's `say 'x:' result` prints `x: `,
    /// and after a bare `return` it prints `x: RESULT`, which is what an unset
    /// Rexx variable renders as. Nor does either form trace a `>>>` line for a
    /// value it does not have. A register holding the null string would answer
    /// the first for both.
    ///
    /// **The last op of a [`Op::Clause`] region, and inside it**: the driver
    /// ends the region with `RegionEnd::Flowed`, which is what [`Op::Call`]'s
    /// arm does with the `Flow` a call answers. Whatever follows this op in
    /// the stream is unreachable through it.
    ///
    /// `index` is read only by a debug assertion that the op still describes
    /// the instruction it was emitted for, for [`Op::Say`]'s reason.
    Return {
        index: u32,
        src: Option<u16>,
        keyword: ReturnKeyword,
    },
    /// Queues the line in register `src` at the end its keyword names, or the
    /// null string for the bare form, for the `PUSH`/`QUEUE` at `index`.
    ///
    /// `Interp::queue_evaluated` is the whole of what `step`'s own arm does
    /// once its expression is computed, entered from here and from there, so
    /// the string rendering, the `>>>` line and the write are one
    /// implementation.
    ///
    /// **`src` is an `Option` for the reason [`Op::Say`]'s is**, and the
    /// answer is the same one: the bare form queues a null string **and**
    /// traces it, rather than being a skipped clause, so the two are the same
    /// stored line and only one of them is what the instruction says.
    ///
    /// **This op does not end its region**, unlike [`Op::Return`]: a `PUSH`
    /// and a `QUEUE` answer `Flow::Next`, so the region ends where a
    /// [`Op::Say`]'s does.
    ///
    /// `index` is read only by a debug assertion, for [`Op::Say`]'s reason.
    Queue {
        index: u32,
        src: Option<u16>,
        keyword: QueueKeyword,
    },
    /// Runs the `CALL name` at `index`, from the resolution this site has kept
    /// or a fresh one, and settles `RESULT` from what came back.
    ///
    /// `Interp::resolve_call` and `Interp::invoke_named_call` are the two
    /// halves, entered from here and from `step`'s own `Call` arm -- so the
    /// four-step resolution order, the argument loop with its `>A>` lines, the
    /// `MAX_ACTIVATION_DEPTH` guard, the five pieces of level state saved
    /// around the nested `run_activation` and the `RESULT` settle with its
    /// `>>>` are one implementation rather than a second one beside it.
    /// `eval_call` enters the same two for `ExprKind::Call`, which is what
    /// stops `CALL length 'abc'` and `say length('abc')` answering
    /// differently.
    ///
    /// **This op emits nothing of its own, and unlike every other computing op
    /// in this stream it owes no echo op -- which is a conclusion rather than
    /// an omission.** The lines a call produces are `>A>` per argument, the
    /// argument expressions' own `>L>`/`>V>`/`>O>`, the callee's clause echoes,
    /// and the `>>>` of the `RESULT` settle. Every one of them is emitted
    /// inside the two halves above, which this op enters rather than replaces,
    /// so there is no line here for an echo op to restore. That is the
    /// difference between this promotion and [`Op::Const`]'s: a literal's value
    /// was *taken away* from `eval.rs`, and a call's arguments were not.
    ///
    /// **The instrument that follows from it.** `tests/ir_dual.rs` compares the
    /// two engines against each other, so a line both arms emit from one shared
    /// function is a line it structurally cannot police -- measured, by making
    /// the `>A>` emission a no-op: the sweep stays green and
    /// `tests/ir_dual_cases/calls` reddens on four rows. The oracle-pinned rows
    /// are what hold these lines, and the dual sweep is what holds the clause
    /// echo this op's region newly decides at compile time.
    ///
    /// **Only `CALL name` and `CALL "name"`.** `CALL ON`/`OFF` resolves no name
    /// at all, and `CALL (expr)` learns its name at run time, so neither
    /// reaches this op -- `compile` leaves both as [`Op::Generic`].
    ///
    /// **The last op of a [`Op::Clause`] region, and inside it** rather than
    /// after it: the whole call runs inside the `CALL` clause exactly as it
    /// does on the tree-walker, so the clause boundary that follows it is where
    /// a `CALL ON` handler the callee queued is delivered. The callee's own
    /// clauses are not ops of this region -- they are a nested activation's,
    /// reached through `run_activation`, which chooses its own engine.
    ///
    /// `site` is this call site's own slot in [`Chunk::calls`], **not** a
    /// position in the op stream, for the reason [`Op::Arith`]'s `hint` is not.
    Call { index: u32, site: u16 },
    /// Runs the message-send clause at `index`: the receiver, the scope
    /// override, the arguments and their `>A>` lines, the send, the `>M>`
    /// line, and `RESULT`. All of it through `Interp::exec_message`, the
    /// same function the tree-walker's own `step` arm calls.
    ///
    /// **No `site`, and that is D28 rather than an omission.** Message
    /// resolution is dynamic with no per-call-site cache, so this op has
    /// nothing to keep between executions -- which is the one field that
    /// separates it from [`Op::Call`], whose `site` is a classic-call cache.
    ///
    /// **What the promotion buys is the clause region, not a cache.** An
    /// instruction left as [`Op::Generic`] cannot sit inside one (see
    /// [`Op::Clause`]'s own rule), so the clause echo, the `SIGL` line, the
    /// temps frame, the condition-delivery boundary and the failing clause's
    /// own site would all come from `Interp::step_in_temps_frame` instead of
    /// this stream. With the op, `tests/ir_dual.rs` compares two genuinely
    /// different routes to the same clause for every corpus program that
    /// sends a message as a whole clause.
    ///
    /// **The last op of a [`Op::Clause`] region, and inside it**, for the
    /// reason [`Op::Call`] is: the send runs inside the clause exactly as it
    /// does on the tree-walker, so the clause boundary that follows it is
    /// where a handler queued during the send is delivered.
    ///
    /// **Every form the clause has**: `q~append(1)`, `q~~append(1)`, and the
    /// message-assignment `q[1] = 2`, which is one `InstructionKind::Message`
    /// with a value and is decided inside `exec_message` rather than by an op
    /// of its own.
    Message { index: u32 },
    /// Continues at op `target`.
    Jump { target: u32 },
    /// Continues at op `target` unless register `reg` holds the logical value
    /// `1`, and at the next op if it does.
    ///
    /// **Only valid inside a `Clause` region**, for the reason
    /// [`Op::EvalExpr`] is: the register it reads is one that region wrote,
    /// and the region's own end is where that register is released.
    JumpUnless { reg: u16, target: u32 },
    /// Validates the condition value in `reg`, emits the `>>>` line it owes,
    /// and leaves the logical value [`Op::JumpUnless`] reads back in that same
    /// register.
    ///
    /// **This is the tail of what a condition used to be one
    /// [`Op::EvalExpr`] or one [`Op::WhenTest`] for.** Those ops evaluate the
    /// expression *and* validate it, through `Interp::eval_if_condition` and
    /// `Interp::scan_when`; here the expression is native ops and this is the
    /// rest. Both of those ops stay, whole, for the conditions `native_shape`
    /// declines -- one op doing both halves, with no op of this kind behind
    /// it.
    ///
    /// **One register, read and written**, because the value it validates is
    /// the value it replaces: `Interp::condition_value` answers a `bool`, and
    /// the Rexx logical value of that answer is what a jump tests. A second
    /// register would hold the unvalidated value that nothing reads again.
    ///
    /// **A native condition is never a comma list**, because
    /// `native_shape` has no `ExprKind::Logical` arm and takes its decision
    /// for the whole expression at once. That is what licenses the driver
    /// passing `checked: false`: nothing upstream of this op has validated
    /// the value, so [`ConditionKeyword::raiser`] is the right raiser and 34.6
    /// cannot be owed here.
    ///
    /// **Only valid inside a [`Op::Clause`] region**, whose clause is the
    /// `IF` or the `WHEN`: the indent its `>>>` prints at and the clause a
    /// failure is blamed on are that region's.
    Condition {
        index: u32,
        reg: u16,
        keyword: ConditionKeyword,
    },
}

/// Which keyword's condition an [`Op::Condition`] is validating.
///
/// **A tag on one op rather than two ops**, because the two arms would
/// otherwise be the same arm twice: an `IF`'s condition and a plain `WHEN`'s
/// reach `Interp::condition_value` with the same
/// `ConditionTrace::Result(indent)` and the same `checked: false`, and differ
/// only in which raiser answers a value that is not exactly `0`/`1`.
#[derive(Clone, Copy)]
pub(crate) enum ConditionKeyword {
    If,
    When,
}

impl ConditionKeyword {
    /// The raiser for a condition value that is not exactly `0` or `1`.
    ///
    /// Measured on the oracle: `if 'x' then nop` is 34.1 and
    /// `select; when 'x' then nop; end` is 34.2, and the two catalogue texts
    /// name their own keyword ("following IF keyword", "following WHEN
    /// keyword"). That difference is the whole of what this tag decides.
    pub(crate) fn raiser(self) -> fn(&[u8]) -> Raised {
        match self {
            ConditionKeyword::If => raised_if_not_logical,
            ConditionKeyword::When => raised_when_not_logical,
        }
    }
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

/// The route from an expression slot's root down to the node one op names.
///
/// **One bit per step, most significant first, behind a sentinel `1`.** A step
/// descends into one of at most two children -- a binary operator's left or
/// right, a prefix operator's only operand, which are the arms
/// `Interp::chunk_node_at` walks -- so a step is a bit, and a `u32` holds
/// thirty-one of them before the sentinel falls off the top.
/// [`NodePath::ROOT`] is the slot's own expression, which is what a call that
/// *is* the whole slot carries.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct NodePath(u32);

impl NodePath {
    /// The slot's own expression: the sentinel alone, with no steps below it.
    pub(crate) const ROOT: NodePath = NodePath(1);

    /// One step further down, into the right child when `right` and the left
    /// or only child otherwise, or `None` when this width has no room for it.
    ///
    /// **`None` rather than a silently dropped sentinel.** Shifting a path
    /// whose sentinel already sits at the top bit leaves a shorter path, and a
    /// shorter path resolves to some *other* node. A refusal costs an address
    /// nobody can give out; a dropped sentinel costs a wrong one.
    pub(crate) fn child(self, right: bool) -> Option<NodePath> {
        (self.0 >> (u32::BITS - 1) == 0).then(|| NodePath(self.0 << 1 | u32::from(right)))
    }

    /// The steps below the sentinel, **outermost first**: the order a descent
    /// from the slot's root walks them in.
    pub(crate) fn steps(self) -> impl Iterator<Item = bool> {
        // The sentinel is the highest set bit, so its position is how many
        // steps sit below it -- zero for `ROOT`, whose value is the sentinel
        // alone.
        let depth = u32::BITS - 1 - self.0.leading_zeros();
        (0..depth).rev().map(move |bit| self.0 >> bit & 1 == 1)
    }
}

/// The frame slot a symbol resolves to, worked out when this chunk was
/// compiled, or the absence of one.
///
/// **The same resolution for a read and for a write**, which is what makes it
/// one type rather than two: [`Op::Load`] and [`Op::Store`] each carry one, and
/// what the writing side does with it is pass it into the one
/// `Interp::assign_expr_target` both engines enter, never a store path of its
/// own.
///
/// **A `u32` with one reserved value rather than an `Option<u32>`, and the op
/// array does not force that.** An `Option<u32>` is eight bytes where this is
/// four, and it is a field [`Op::Load`] and [`Op::Store`] carry -- but
/// measured 2026-08-12, the assertion above [`Op`] passes with this type
/// holding one. So the reserved value buys no width at all here, and anything
/// resting on it has to rest on something else.
///
/// **A compound never has one; a stem read carries one and a stem write takes
/// its own from elsewhere.** A compound's read goes through the *stem's* slot
/// and a tail key resolved at the read site, so the symbol's own slot is not
/// what it reads, and [`PlanSlot::UNRESOLVED`] is the honest answer. A bare
/// stem *read* does go to the symbol's own slot, and [`Op::Load`] carries it.
/// A bare stem *write* goes to that same slot -- but `Interp::
/// assign_expr_target`, which both engines enter, reads it off the plan's own
/// `CompoundName` entry, so an op carrying it would be a second source for one
/// number and would serve one engine where the entry serves both. Either way
/// the run-time path resolves what it was not given.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct PlanSlot(u32);

impl PlanSlot {
    /// No slot resolved: the read or write works its own out, which is what
    /// every one of them did before there was a compiler to do it earlier.
    pub(crate) const UNRESOLVED: PlanSlot = PlanSlot(u32::MAX);

    /// The slot `at`, or [`PlanSlot::UNRESOLVED`] when it does not fit this
    /// width.
    ///
    /// **Not a [`ChunkTooLarge`], which would refuse the whole body for
    /// something that is only an optimisation.** A slot index past `u32` is a
    /// frame with four billion names in it; the read still has a correct
    /// answer, and it is the one the tree-walker computes.
    pub(crate) fn of(at: usize) -> PlanSlot {
        match u32::try_from(at) {
            Ok(at) if at != PlanSlot::UNRESOLVED.0 => PlanSlot(at),
            _ => PlanSlot::UNRESOLVED,
        }
    }

    /// The slot, or `None` when this op carries none.
    pub(crate) fn resolved(self) -> Option<usize> {
        (self != PlanSlot::UNRESOLVED).then_some(self.0 as usize)
    }
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

/// Whether a compiled arithmetic site keeps a hint about which path to try
/// first.
///
/// **The whole of the patch table's removal, in one place**, so that measuring
/// what the table is worth is a single edit rather than a change threaded
/// through the driver's arms. With this `false`, [`Hints`] allocates nothing,
/// no slot is loaded or stored, and every [`Op::Arith`] tries the
/// small-integer path and falls through to the general one -- which is exactly
/// what the op does with no table at all. A win that survives the switch was
/// the static promotion of the operands and the operator, not the quickening.
const QUICKENING: bool = true;

/// The state a site starts in and stays in while the small-integer path keeps
/// answering: try that path first.
const TRY_SMALL_INT: u32 = 0;

/// The state a site moves to the first time the small-integer path answers
/// `None`: go straight to the general one.
const GENERAL: u32 = 1;

/// One arithmetic site's hint.
///
/// **A hint never removes a precondition check** (D22). What it decides is
/// which of `Interp::arith_small_int` and `Interp::arith_general` is *tried
/// first*, and both are correct for every operand: the small-integer path
/// answers `None` for every case where the two could disagree, so a site that
/// skips it computes the same value more slowly, and a site that tries it
/// re-decodes both operands and re-checks them against `DIGITS` every time.
/// Nothing here can make an answer wrong; it can only make one slower.
///
/// **The state only ever moves one way**, from [`TRY_SMALL_INT`] to
/// [`GENERAL`], so the store happens at most once per site and a demoted site
/// pays no store at all. The cost of that is a site whose operands leave the
/// exact-integer range once and return: it keeps the general path afterwards.
/// Re-arming needs a policy -- a counter, an interval -- and there is no
/// measurement here to choose one from, so the simple monotone rule is what
/// this builds.
///
/// **`Cell<u32>`, and the reason is that the alternative's bet cannot be
/// collected.** What bites today is interior mutability at all: a chunk is
/// reached as `&Chunk` through an `Rc`, so a site cannot record anything
/// without it. An `AtomicU32` would add `Sync` on top of that, against the day
/// ooRexx shares a routine body across activities -- but a chunk reaches its
/// caller as `Rc<Chunk>`, and an `Rc` is neither `Send` nor `Sync` whatever it
/// holds, so that day needs a different owner before it needs a different slot;
/// and [`CallSite`] one construct over holds a `Cell` whose payload is too wide
/// for a lock-free atomic, so a `Chunk` is not `Sync` either way -- both of
/// which rustc confirms, and [`CallSite`]'s own doc has that answer in full. The atomic
/// therefore bought a property nothing could observe, at a price that was
/// measured rather than assumed: **8 instructions per pass on `arith`, 2 on
/// `compound`, 1 on `varlookup` and 0 on `emptyloop`**, two builds of one
/// sitting, `instructions:u`. The three methods below are the only place the
/// choice appears, so pricing the other one again is the same three lines.
struct PatchSlot(Cell<u32>);

impl PatchSlot {
    fn new() -> PatchSlot {
        PatchSlot(Cell::new(TRY_SMALL_INT))
    }

    fn get(&self) -> u32 {
        self.0.get()
    }

    fn set(&self, state: u32) {
        self.0.set(state);
    }
}

/// One chunk's patch table: a hint per [`Op::Arith`], indexed by that op's own
/// `hint` field.
///
/// **Dense over the ops that can specialise rather than parallel to the op
/// stream**, which is what makes D22's "an op that never quickens pays
/// nothing" true of the driver's loop and not just of its arms. A table
/// indexed by op position would need the position, and a region's ops are
/// walked as a slice -- so finding it would mean a counter incremented for
/// every op in every region, paid by the ops the table has no entry for.
struct Hints {
    /// One slot per arithmetic op, or **empty** when [`QUICKENING`] is off.
    slots: Vec<PatchSlot>,
    /// How many arithmetic ops have taken a slot.
    ///
    /// Kept separately from `slots.len()` so that the indices the ops carry are
    /// the same whether or not the table exists -- a golden op stream then
    /// reads identically under either setting, and the switch measures the
    /// table rather than also moving what compiled.
    next: u32,
}

impl Hints {
    fn new() -> Hints {
        Hints {
            slots: Vec::new(),
            next: 0,
        }
    }

    /// Reserves the slot for one arithmetic op, answering the index it
    /// carries.
    ///
    /// A [`ChunkTooLarge`] past `u32`, for the reason every other index in this
    /// stream has one: the op's field is that wide.
    fn reserve(&mut self) -> Result<u32, ChunkTooLarge> {
        let at = self.next;
        self.next = at.checked_add(1).ok_or(ChunkTooLarge {
            what: "arithmetic sites past u32",
        })?;
        if QUICKENING {
            self.slots.push(PatchSlot::new());
        }
        Ok(at)
    }

    /// Whether site `at` is still worth trying the small-integer path for.
    ///
    /// `true` for a slot this table does not have, which is what makes
    /// [`QUICKENING`] off behave as no table at all rather than as a table
    /// that answers no.
    fn tries_small_int(&self, at: u32) -> bool {
        !QUICKENING
            || self
                .slots
                .get(at as usize)
                .is_none_or(|slot| slot.get() == TRY_SMALL_INT)
    }

    /// Records that site `at` has had the small-integer path answer `None`.
    fn saw_general(&self, at: u32) {
        if QUICKENING && let Some(slot) = self.slots.get(at as usize) {
            slot.set(GENERAL);
        }
    }
}

/// Whether a compiled call site keeps the resolution it made.
///
/// **The whole of the resolution table's removal, in one place**, for the
/// reason [`QUICKENING`] is one: measuring what the table is worth is then a
/// single edit rather than a change threaded through the driver's arm. With
/// this `false`, [`Calls`] allocates nothing, no slot is read or written, and
/// every [`Op::Call`] resolves afresh -- which is exactly what the tree-walker
/// does for the same call.
const CALL_SITE_CACHE: bool = true;

/// One call site's kept resolution, or none yet.
///
/// **What makes keeping it correct is that nothing can invalidate it**, and
/// each of the four resolution steps has its own reason. A `Resolved::Label`
/// indexes the running activation's body, and a chunk is compiled per body and
/// entered only for an activation of that body (`run_activation` looks its
/// chunk up under the running activation's own `body_key`). The builtin table
/// is static. **`Interp::routines` never rebinds a name it has already
/// bound**: a second `::ROUTINE` directive for one is refused outright with
/// 99.903 (`install_directives`, `lib.rs`, whose own comment has the oracle
/// measurement), so a name that resolves to a routine keeps that routine for
/// the rest of the run however many programs are loaded. And the fourth step
/// raises rather than resolving, so a failure is never recorded here at all --
/// a site that raised 43.1 asks again next time.
///
/// **The property the table needs is that one, and not "the map is written
/// once"**, which is a stronger thing that happens to be true today and would
/// stop being the reason if a `::REQUIRES` or an external-file call ever
/// installed a routine mid-run. Append-only is what the refusal enforces, and
/// append-only is enough.
///
/// **A `Cell`, and a `Resolved` is what forces it rather than a preference.**
/// A `Resolved` is wider than any lock-free atomic here can carry, so an atomic
/// slot would cost either an encoding or a lock, where [`PatchSlot`]'s state is
/// a `u32` that either could hold.
///
/// **What stops a [`Chunk`] being `Sync` is this field and [`PatchSlot`], and
/// an earlier version of this comment claimed it was this one alone.** Re-taken
/// 2026-08-12 the way the claim says to take it -- `const _: () = { const fn
/// assert_sync<T: Sync>() {} assert_sync::<Chunk>(); }` -- and rustc answers
/// with two errors, `Cell<Option<Resolved>>` through `Calls` and `Cell<u32>`
/// through `Hints`, and nothing else. Two fields, both of them a per-site
/// cache, and no third.
///
/// **Neither should become an atomic before `Rc<Chunk>` becomes an `Arc`**, and
/// that ordering is [`PatchSlot`]'s own argument rather than a new one: an `Rc`
/// is neither `Send` nor `Sync` whatever it holds, so a `Sync` slot inside one
/// buys a property nothing can observe -- and that comment carries the price of
/// buying it early, **8 instructions per pass on `arith`**, measured. What has
/// changed since it was written is only this field's arithmetic: `Resolved`'s
/// builtin arm now carries a `u16` row rather than nothing, so two of the three
/// arms would fit a `u64` and only `Resolved::Routine`, which is two `usize`s,
/// still would not. An encoding that declines that arm -- leaving such a site to
/// resolve afresh, exactly as a site that raised already does -- is what an
/// atomic version would be, and it is work for the day the owner changes.
struct CallSite(Cell<Option<Resolved>>);

impl CallSite {
    fn new() -> CallSite {
        CallSite(Cell::new(None))
    }

    fn get(&self) -> Option<Resolved> {
        self.0.get()
    }

    fn set(&self, resolved: Resolved) {
        self.0.set(Some(resolved));
    }
}

/// One chunk's resolution table: a slot per call op -- [`Op::Call`] and
/// [`Op::CallExpr`] alike -- indexed by that op's own `site` field.
///
/// **Dense over the ops that resolve rather than parallel to the op stream**,
/// which is [`Hints`]' own argument one construct over.
struct Calls {
    /// One slot per call op, or **empty** when [`CALL_SITE_CACHE`] is off.
    slots: Vec<CallSite>,
    /// How many call ops have taken a slot, kept separately from `slots.len()`
    /// so that the indices the ops carry are the same whether or not the table
    /// exists -- [`Hints::next`]'s own reason, so that a golden op stream reads
    /// identically under either setting.
    next: u16,
}

impl Calls {
    fn new() -> Calls {
        Calls {
            slots: Vec::new(),
            next: 0,
        }
    }

    /// Reserves the slot for one call op, answering the index it carries.
    fn reserve(&mut self) -> Result<u16, ChunkTooLarge> {
        let at = self.next;
        self.next = at.checked_add(1).ok_or(ChunkTooLarge {
            what: "call sites past u16",
        })?;
        if CALL_SITE_CACHE {
            self.slots.push(CallSite::new());
        }
        Ok(at)
    }

    /// What site `at` resolved to last time, or `None` for a site that has not
    /// resolved yet or has no slot at all -- which is what makes
    /// [`CALL_SITE_CACHE`] off behave as no table rather than as a table that
    /// answers wrongly.
    fn resolved(&self, at: u16) -> Option<Resolved> {
        self.slots.get(at as usize).and_then(CallSite::get)
    }

    /// Records what site `at` resolved to.
    fn remember(&self, at: u16, resolved: Resolved) {
        if let Some(slot) = self.slots.get(at as usize) {
            slot.set(resolved);
        }
    }
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
    /// The literal values [`Op::Const`] loads, **one entry per distinct
    /// literal** rather than one per occurrence.
    ///
    /// **The interning is measured rather than tidy.** An entry is a heap
    /// allocation, so a table keyed by occurrence costs one allocation per
    /// literal written -- which on a straight-line body is one per clause,
    /// paid once at compile time. The mechanics spike measured that artifact
    /// swamping the difference it existed to measure, and interning is what
    /// removes it.
    ///
    /// **Interning is invisible to a running program**, which is the property
    /// that makes it safe: `Op::Const` reads the bytes and builds a fresh
    /// value from them through `Interp::literal` every time it runs, so two
    /// occurrences of one literal share a table entry and share no value.
    /// `assignment-and-say`'s "one literal written twice traces twice" row is
    /// that stated as output.
    consts: Vec<Box<[u8]>>,
    /// The quickening hints [`Op::Arith`] reads, one per such op.
    ///
    /// **Mutable where the op stream is not**: an op is emitted once and never
    /// rewritten, so two activities running one body see the same instructions
    /// and differ only in what their sites have learned.
    hints: Hints,
    /// The resolutions [`Op::Call`] reads and writes, one per such op.
    calls: Calls,
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

    /// The ops of `[at, end)`, or `None` when that is not a range of this
    /// stream.
    ///
    /// **What a clause region is walked as, rather than one [`Chunk::op_at_index`]
    /// per op**, and the reason is that a region's counter only ever advances:
    /// every op inside one either falls through to the next or ends the region,
    /// so the range is settled once on the way in and the bounds check per op
    /// goes with it. Worth 2 instructions per promoted clause on
    /// `bench-programs/varlookup.rex`, whose regions hold three ops.
    fn ops_in(&self, at: u32, end: u32) -> Option<&[Op]> {
        self.ops.get(at as usize..end as usize)
    }

    /// The setting this chunk's trace ops were emitted for.
    fn trace(&self) -> ChunkTrace {
        self.trace
    }

    /// The bytes of constant `at`, or `None` when the index is outside the
    /// table this chunk was compiled with.
    fn konst(&self, at: u32) -> Option<&[u8]> {
        self.consts.get(at as usize).map(|bytes| &bytes[..])
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

    /// Whether arithmetic site `at` is still worth trying the small-integer
    /// path for ([`Hints::tries_small_int`]).
    fn tries_small_int(&self, at: u32) -> bool {
        self.hints.tries_small_int(at)
    }

    /// Records that arithmetic site `at` has fallen through to the general
    /// path ([`Hints::saw_general`]).
    fn saw_general(&self, at: u32) {
        self.hints.saw_general(at);
    }

    /// What call site `at` resolved to last time ([`Calls::resolved`]).
    fn resolved_call(&self, at: u16) -> Option<Resolved> {
        self.calls.resolved(at)
    }

    /// Records what call site `at` resolved to ([`Calls::remember`]).
    fn remember_call(&self, at: u16, resolved: Resolved) {
        self.calls.remember(at, resolved);
    }
}

#[cfg(test)]
mod tests {
    use super::NodePath;

    /// A path round-trips the steps it was built from, and refuses the step
    /// past its width rather than silently dropping the sentinel.
    #[test]
    fn a_node_path_carries_thirty_one_steps_and_refuses_the_thirty_second() {
        let mut path = NodePath::ROOT;
        assert_eq!(path.steps().count(), 0);
        for step in 0..31 {
            path = path.child(step % 2 == 0).expect("thirty-one steps fit");
        }
        assert_eq!(path.steps().count(), 31);
        assert!(path.child(false).is_none());
    }

    /// The steps come back outermost first, which is the order the descent
    /// walks them in -- a path read innermost first would land on the wrong
    /// node in every asymmetric expression.
    ///
    /// **Asymmetric on purpose.** A path of one step, or of two equal ones,
    /// reads the same in either direction and would be satisfied by the
    /// reversed iterator this pins against.
    #[test]
    fn a_node_paths_steps_come_back_outermost_first() {
        let path = NodePath::ROOT
            .child(true)
            .and_then(|path| path.child(false))
            .expect("two steps fit");
        assert_eq!(path.steps().collect::<Vec<_>>(), [true, false]);
    }
}
