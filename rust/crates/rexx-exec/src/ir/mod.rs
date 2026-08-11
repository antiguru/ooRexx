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
use std::sync::atomic::{AtomicU32, Ordering};

use rexx_core::FrameId;
use rexx_parse::{Operator, SymbolId};

use crate::eval::SymbolRead;
use crate::run::{HeaderRole, Resolved};
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

/// **The stream's own width, asserted rather than described.** Every op in
/// every chunk pays for the widest variant, so a field added to one of them is
/// a cost to all of them -- which is the argument [`ReadSlot`] rests on, and an
/// argument about a width is worth nothing without the width. The widest
/// payload today has tail padding for the discriminant to sit in; a variant
/// that needs more than that grows the array, and this is where that shows up
/// as a compile error rather than as a measurement somebody has to take again.
const _: () = assert!(size_of::<Op>() == 12);

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
    /// **A separate op from the [`Op::EvalExpr`] that produced the value, and
    /// that is the whole reason this construct waited for the trace ops.** A
    /// loop header interleaves evaluation and emission -- it evaluates `TO`,
    /// echoes it, evaluates `BY`, echoes it, in the order the keywords were
    /// written -- so an op that only evaluated could not reproduce the
    /// ordering, and an op that did both would be the whole evaluate-and-trace
    /// unit rather than the general expression op the later tasks need. With
    /// the emission its own op, **the order in `Controlled::order` is the order
    /// of ops**.
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
    /// measured rather than an omission.** A literal in an assignment's value
    /// position or a `SAY`'s expression position is what emits it, and no
    /// registered benchmark axis has one **inside a measured loop**: counting
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
    /// **Only the whole expression, never a symbol inside one.** `x + 1`
    /// compiles to [`Op::EvalExpr`] entire; nothing here descends into an
    /// operator's operands.
    ///
    /// `at` is the slot the plan already resolved this symbol to, which is the
    /// one thing this op knows that the tree-walker's own read has to work out
    /// -- see [`ReadSlot`] for what carries it and why a compound never has
    /// one.
    ///
    /// **Only valid inside a [`Op::Clause`] region**, whose clause owns the
    /// value indent the line after this one traces at, and the failure site a
    /// `NOVALUE` raised here is reported against.
    Load {
        symbol: SymbolId,
        read: SymbolRead,
        at: ReadSlot,
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
    /// Computes `lhs op rhs` into register `dst`, for the seven operators
    /// `eval::is_arithmetic` names.
    ///
    /// **Native in the sense [`Op::Const`] and [`Op::Load`] are: `eval.rs` is
    /// not entered, and neither is it for the operands.** An expression
    /// compiles to this op only when *both* its operands compile to native ops
    /// too, so `za + zb * 4` is six ops and no `eval` recursion, while
    /// `length('ab') + 1` is one [`Op::EvalExpr`] entire -- a call has no
    /// register to arrive in, and `EvalExpr` names an expression *slot* of an
    /// instruction, which a subexpression is not.
    ///
    /// The arithmetic itself is `Interp::arith_small_int` and
    /// `Interp::arith_general`, entered from here and from
    /// `Interp::eval_arithmetic`, so the operand conversion, the seven
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
    /// Echoes the `>O>` line of the operator result in register `src`.
    ///
    /// **A separate op from the [`Op::Arith`] that computed it**, for the
    /// reason [`Op::TraceLiteral`] is separate from [`Op::Const`]: the
    /// computation emits nothing, `eval.rs` emits this line as a side effect of
    /// *evaluating* a binary node, and a promoted clause with no such op drops
    /// it while every line after it still matches. Emitted unconditionally
    /// rather than under [`ChunkTrace`]'s decision, because the gate this line
    /// answers to is `trace_mode().intermediates`, which [`ChunkTrace`] does
    /// not carry.
    ///
    /// `op` is repeated here rather than read off the `Arith` behind it,
    /// because the tag is the operator's own spelling and an echo carrying a
    /// different one lands in the right place with the wrong tag in it.
    /// `compile::assert_operator_echoes_follow_their_op` is what checks the
    /// position, the register and the operator rather than assuming them.
    ///
    /// **Only valid inside a [`Op::Clause`] region**, and immediately behind
    /// the `Arith` whose register it reads: `eval.rs` emits this post-order,
    /// with the value in hand, so an inner operator's line precedes the outer
    /// one's exactly as the ops do.
    TraceOperator { op: Operator, src: u16 },
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
    /// **Only valid inside a [`Op::Clause`] region**, whose clause is this
    /// assignment's own: the indent every line above traces at and the
    /// boundary the write precedes both belong to it.
    Store { index: u32, src: u16 },
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
    Call { index: u32, site: u32 },
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

/// The frame slot [`Op::Load`] reads, resolved when this chunk was compiled,
/// or the absence of one.
///
/// **A `u32` with one reserved value rather than an `Option<u32>`, and it is
/// the op array that decides it.** An `Option<u32>` is eight bytes where this
/// is four, which is the difference between an [`Op`] that stays the width
/// every other variant already fits in and one that grows -- paid by every op
/// in every chunk, for a field two of them carry. The assertion above [`Op`] is
/// what holds that width rather than this sentence.
///
/// **A compound never has one.** Its read goes through the *stem's* slot and a
/// tail key resolved at the read site, so the symbol's own slot is not what it
/// reads; [`ReadSlot::UNRESOLVED`] is the honest answer and the run-time path
/// is what resolves it, exactly as it does for the tree-walker.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct ReadSlot(u32);

impl ReadSlot {
    /// No slot resolved: the read works its own out, which is what every read
    /// did before there was a compiler to do it earlier.
    pub(crate) const UNRESOLVED: ReadSlot = ReadSlot(u32::MAX);

    /// The slot `at`, or [`ReadSlot::UNRESOLVED`] when it does not fit this
    /// width.
    ///
    /// **Not a [`ChunkTooLarge`], which would refuse the whole body for
    /// something that is only an optimisation.** A slot index past `u32` is a
    /// frame with four billion names in it; the read still has a correct
    /// answer, and it is the one the tree-walker computes.
    fn of(at: usize) -> ReadSlot {
        match u32::try_from(at) {
            Ok(at) if at != ReadSlot::UNRESOLVED.0 => ReadSlot(at),
            _ => ReadSlot::UNRESOLVED,
        }
    }

    /// The slot, or `None` when this op carries none.
    fn resolved(self) -> Option<usize> {
        (self != ReadSlot::UNRESOLVED).then_some(self.0 as usize)
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
/// **`AtomicU32` rather than `Cell<u32>`, for two reasons of different
/// strength.** The one that bites today is interior mutability at all: a chunk
/// is reached as `&Chunk` through an `Rc`, so a site cannot record anything
/// without it, and `Cell<u32>` would serve. The one that does not bite yet is
/// `Sync`: ooRexx shares routine bodies across activities, and a `Cell` could
/// not survive that -- but nothing in this crate crosses a thread today, since
/// the chunk cache is an `Rc` map per `Interp`. So the atomic is a
/// forward-compatibility bet rather than a present necessity, and what it
/// costs against a plain read is a number this type is shaped to make
/// measurable: the three methods below are the only place the choice appears.
struct PatchSlot(AtomicU32);

impl PatchSlot {
    fn new() -> PatchSlot {
        PatchSlot(AtomicU32::new(TRY_SMALL_INT))
    }

    /// **`Relaxed`, and the ordering is not a shortcut**: the value is a hint
    /// that orders nothing else, no other memory is published with it, and
    /// every state it can hold is correct to read at any time. A racing pair
    /// of activities can only both decide the same site is general.
    fn get(&self) -> u32 {
        self.0.load(Ordering::Relaxed)
    }

    fn set(&self, state: u32) {
        self.0.store(state, Ordering::Relaxed);
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
/// is static. `Interp::routines` is written only by `install_directives`,
/// which runs once, before the first clause of the program. And the fourth
/// step raises rather than resolving, so a failure is never recorded here at
/// all -- a site that raised 43.1 asks again next time.
///
/// **`Cell` rather than the `AtomicU32` [`PatchSlot`] uses, and the difference
/// is the payload rather than a change of mind.** That type's state is a `u32`,
/// so an atomic costs it nothing and buys the `Sync` a shared routine body
/// would one day need; a `Resolved` is wider than any lock-free atomic here can
/// carry, so the same bet would cost either an encoding or a lock. Nothing in
/// this crate crosses a thread today -- the chunk cache is an `Rc` map per
/// `Interp` -- and this is the type that would have to change first if that
/// stopped being true.
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

/// One chunk's resolution table: a slot per [`Op::Call`], indexed by that op's
/// own `site` field.
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
    next: u32,
}

impl Calls {
    fn new() -> Calls {
        Calls {
            slots: Vec::new(),
            next: 0,
        }
    }

    /// Reserves the slot for one call op, answering the index it carries.
    fn reserve(&mut self) -> Result<u32, ChunkTooLarge> {
        let at = self.next;
        self.next = at.checked_add(1).ok_or(ChunkTooLarge {
            what: "call sites past u32",
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
    fn resolved(&self, at: u32) -> Option<Resolved> {
        self.slots.get(at as usize).and_then(CallSite::get)
    }

    /// Records what site `at` resolved to.
    fn remember(&self, at: u32, resolved: Resolved) {
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
    /// **One of the two mutable things a running chunk owns**, and the op
    /// stream is not either of them: an op is emitted once and never
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
    fn resolved_call(&self, at: u32) -> Option<Resolved> {
        self.calls.resolved(at)
    }

    /// Records what call site `at` resolved to ([`Calls::remember`]).
    fn remember_call(&self, at: u32, resolved: Resolved) {
        self.calls.remember(at, resolved);
    }
}
