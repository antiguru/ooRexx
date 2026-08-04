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

//! The instruction loop: `Flow`, `step`, and the two functions that run it.
//!
//! `run_activation`, `step`, `step_in_temps_frame` and `run_fragment` moved
//! here from Task 3's spike (`lib.rs`), `Flow` alongside them. This is where
//! the design's "The borrow shape" is actually written down: clone the `Rc`
//! into a local on entry, and derive every `&CodeBody` and `&Expr` from that
//! local rather than from `self`. `run_activation`'s own doc comment carries
//! the argument in full, including the version that does not compile, and
//! none of it changed in the move -- `lib.rs` keeps `Code` itself, `Interp`,
//! the entry point and the loud-failure catalogue, which is what makes the
//! move safe: every field and every sibling method this file's functions
//! reach into is already visible here, exactly as it was in the crate root.
//!
//! Task 9 is the first task to extend `step` rather than only prove its
//! shape. `SAY` and `Assignment`'s `Variable` target, `EXIT` with no result,
//! and `INTERPRET` under the spike flag are the spike's own witnesses and
//! move unchanged. New here: `Assignment`'s `Stem`/`Compound` targets, all of
//! `DROP`, all six spellings of `NUMERIC`, `EXIT` with a result, and `LABEL`/
//! `NOP` as the no-ops they are.
//!
//! Task 10 is the first to make `Flow::Goto` live, for `IF`/`SELECT`/
//! `SELECT CASE`. **`If` and `Select` each resolve their whole construct
//! inside their own `step` arm**, running the winning branch through a
//! bounded local loop (`run_bounded`, shaped like `run_fragment`'s) rather
//! than leaving the outer `run_activation` loop to fall through the flat
//! instruction list on its own. That is not a style choice: Phase 3 elides
//! the C++'s synthetic end-of-branch markers, so `false_target` on an `If`
//! lands exactly on its `Else` (never past it), and a matched branch's own
//! completion, via plain `pc += 1`, lands on that exact same index -- the
//! true and false arrivals are indistinguishable from `(instruction, pc)`
//! alone, and only one of the two is supposed to enter the `Else`'s body.
//! `SELECT`/`WHEN` has the same defect with no marker to disambiguate with
//! at all. `run_bounded`'s doc comment carries the resolution. `Then`,
//! `Else` and `Otherwise` step as pure no-ops (like `Label`), only ever read
//! as data by `If`/`Select` or walked over inside a bounded loop. `When` and
//! `WhenCase` no longer are (Task 13's absorbed-`WHEN` fix and its own F3):
//! an ordinary, listed `When`/`WhenCase` is still never independently
//! stepped for its own decision -- `Select`'s arm evaluates and dispatches
//! it directly -- but one *absorbed* into another `When`/`WhenCase`'s own
//! `THEN` (never collected into the enclosing `SELECT`'s own `whens` list,
//! `ast.rs`'s own doc comment) is reached only by ordinary stepping, and its
//! arm evaluates its condition and, for `WhenCase`, branches on the result.
//!
//! 4b's Task 3 is the first to make the **activation stack** real, and it is
//! `run_activation` that changes shape rather than only gaining arms: it
//! reads the activation's own body selector instead of hardcoding
//! `&program.main`, and it answers `Ended` rather than a bare
//! `Option<ObjRef>`, because a callee has two ways out that differ in what
//! the caller does next. `CALL`'s own work lives in `exec_call`, which is
//! `run_fragment`'s counterpart at the other kind of level boundary -- both
//! save and restore the same indent state and both `seal_site_level` on the
//! way out, and the one place they differ (this one *clears*
//! `clause_line_override` where the other *sets* it) is measured and stated
//! at both.

use crate::activation::{Activation, Inherited, Trap, body_of};
use crate::clause::{ClauseOutcome, ClauseValue, HandlerExit};
use crate::error::{FailureSite, Raised, Search};
use crate::eval::logical_value;
use crate::trace::{
    is_whole_number, mode_from_setting, raised_invalid_trace_letter,
    raised_numeric_trace_interactive_only,
};
use crate::{
    ActiveCondition, Argument, CallContext, Code, Failure, Interp, Loud, Novalue, PendingTrap,
};
use rexx_core::{ObjRef, SlotFrame, SlotRef};
use rexx_num::{ArithError, CompareOp, Number, SettingsError, compare_decoded};
use rexx_parse::{
    ConditionTrap, ControlExpr, Controlled, EndStyle, Expr, ExprKind, Fragment, Instruction,
    InstructionKind, Loop, LoopConditional, LoopKind, NumericSetting, ProgramSource, Raise,
    SymbolId, Trace, Use, UseTarget, VariableRef, compound_parts, parse_interpret,
};
use std::borrow::Cow;
use std::collections::HashMap;
use std::rc::Rc;

/// Where control goes after one instruction (the design's "Control flow").
pub(crate) enum Flow {
    Next,
    /// Live since Task 10: `If`/`Select` each resolve to one `Goto` that
    /// skips straight to their construct's true resume point, and
    /// `run_bounded`'s own internal loop applies one whenever a nested
    /// construct's target lands inside its range.
    ///
    /// **This includes inside a fragment.** A label inside `INTERPRET` text
    /// is still error 47.1 (Task 1), so a fragment's `labels` is still
    /// always empty and nothing there can jump to a *label* -- but Task 10
    /// makes `IF`/`SELECT` able to appear (and jump) anywhere a `Code` body
    /// can be stepped, a fragment's own included, with no label involved at
    /// all. `run_fragment` no longer has an `unreachable!` on this variant.
    /// It now runs through `run_bounded` for exactly this reason, and
    /// `run_fragment`'s own doc comment carries the argument for why every
    /// jump such a construct computes stays inside the fragment's own range.
    Goto(usize),
    Exit(Option<ObjRef>),
    /// `RETURN`, with the expression's value or `None` for the bare form.
    /// Live since Task 3.
    ///
    /// **Why none of the four variants above expresses this.** `Goto` moves
    /// within one body; `Exit` ends the whole program and every activation
    /// with it; `Leave`/`Iterate` are consumed by a `DO`/`SELECT` frame
    /// inside the current activation. `RETURN` unwinds to the **activation**
    /// boundary and no further -- past every enclosing `DO`, `SELECT` and
    /// `IF` in the callee, and past no part of the caller. Measured: a
    /// `return` inside a `do forever` inside a called routine resumes the
    /// caller's next clause rather than looping, and `LEAVE`'s own search
    /// (which does stop at those frames) is exactly the behaviour it must
    /// not have.
    ///
    /// So it travels as data through the same channel `Exit` does -- every
    /// `run_bounded` catch-all and every `Do`/`Select` forwarding arm already
    /// passes anything they do not own straight out -- and `run_activation`
    /// is the only thing that ever consumes it.
    ///
    /// **In the *main* body, with no caller, it ends the program with its
    /// value, exactly like `EXIT`.** Measured: `say 'a'` / `return 5` /
    /// `say 'b'` prints `a` and exits 5, and a bare `return` there exits 0.
    /// `run_activation` reports it as `Ended::Returned` regardless and
    /// `Interp::run` is what treats the two alike at the top -- kept apart
    /// down here because a callee genuinely has to tell them apart.
    Return(Option<ObjRef>),
    /// `LEAVE`, bare (`None`) or by name. Live since Task 11.
    ///
    /// **Why this is a `Flow` variant and not an immediate `Err`:** `LEAVE`
    /// finding its own target is not a failure, so it has to travel as data
    /// through exactly the same channel `Goto`/`Exit` already do --
    /// `run_bounded`'s own catch-all (`other => return Ok(other)`, its doc
    /// comment names this variant by name as the reason it exists) forwards
    /// it outward through any nested `IF` untouched, and `Do`/`Select`'s own
    /// arms are the only two that ever inspect it, matching the oracle's own
    /// rule that only a `SELECT`/`DO`/`LOOP` block ever participates in the
    /// search (`RexxActivation::leaveLoop`, read directly and cited in the
    /// report -- `IF`/`THEN`/`ELSE` never push a frame at all, so they are
    /// transparent by construction, not by a case this crate has to add).
    ///
    /// **The invariant this crate's whole `Do`/`Select` design holds so this
    /// variant is safe to use at all**: unlike `Goto`, a `Do`'s own
    /// repetition is never expressed as a `Goto` back to its own body's top.
    /// The trap that would create is exactly `run_bounded`'s own doc comment
    /// warns about for a future `LEAVE`/`ITERATE` variant -- a `Goto` whose
    /// target lands inside an *enclosing* `IF`/`SELECT`'s own range (a `DO`
    /// nested in an `IF`'s `THEN`, iterating) is absorbed by that enclosing
    /// `run_bounded` directly, never seen by the `DO`'s own arm again, which
    /// would re-enter it as a first entry with its own state reset. `Do`'s
    /// own arm therefore never returns to its caller until the *entire*
    /// loop -- every iteration -- is over, one way or another: it drives its
    /// own `run_bounded(body)` calls in an internal `loop {}` and only ever
    /// returns a `Flow` once there is truly nothing left for it to decide.
    /// `leave_and_iterate_survive_a_do_nested_in_an_ifs_then_iterating_repeatedly`
    /// (this file's own tests) pins exactly this shape: a `DO` with an `ITERATE`
    /// in its body, nested inside an `IF`'s `THEN`, run enough times that a
    /// version which instead returned a re-entry `Goto` to the loop's own
    /// top would either loop forever (the `IF`'s own `run_bounded` silently
    /// re-entering the `DO` as a fresh first pass on every `Goto`) or lose
    /// the loop's own running total, depending on exactly how such a bug
    /// were shaped -- either way, not the correct, small, printed result the
    /// test asserts.
    ///
    /// The payload eagerly resolves the `LEAVE`/`ITERATE` instruction's own
    /// clause and static indent (`LeaveOrigin`) at the moment it steps,
    /// before any propagation: **28.1-28.4 (the "found nothing at all"
    /// family) and 28.5 (the "found a name match, but it names something
    /// that is not a loop" family) report at two different, both
    /// oracle-measured, indentations that no longer-lived state can recover
    /// once the search has moved on** -- see the report's own transcripts.
    Leave(Option<SymbolId>, LeaveOrigin),
    /// `ITERATE`, bare or by name. See `Leave`'s own doc comment; the two
    /// variants are handled by nearly identical logic in `Do`/`Select`'s own
    /// arms, differing only in which of the oracle's measured asymmetries
    /// applies (`Select` never consumes a bare `Iterate` at all, and a named
    /// one that matches its own label but is not a loop is 28.5, not simply
    /// "not mine, keep looking").
    Iterate(Option<SymbolId>, LeaveOrigin),
    /// `SIGNAL label` and `SIGNAL VALUE`, once the target resolves to an
    /// instruction index. Live since 4b's Task 6.
    ///
    /// **A distinct variant from `Goto`, and that is not merely for
    /// clarity.** `Goto`'s own contract (`run_bounded`'s doc comment) is "in
    /// range for the body currently being stepped", which is exactly wrong
    /// for `SIGNAL`: the target always resolves against the running
    /// *activation's* own body (`resolve_signal_target`, mirroring
    /// `resolve_and_run_call`'s identical fix for `CALL`), and inside an
    /// `INTERPRET` fragment that body is a completely different `Code` from
    /// the one `run_fragment` is stepping.
    ///
    /// **The risk is real, and reusing `Goto` does not always fail -- which
    /// is exactly why this needed a program built to collide, not a doc
    /// comment alone (I1/I2, review round 1).** `run_fragment`'s own
    /// `run_bounded(&code, 0, fragment.len())` absorbs any escaping
    /// `Flow::Goto(target)` with `target <= fragment.len()` (`run_bounded`'s
    /// own guard, `start == 0` here) as if it were its own, silently
    /// resuming the fragment's own unrelated instruction at that position
    /// instead of escaping -- and whether a given program collides depends
    /// only on whether the target's index happens to be no greater than the
    /// fragment's own instruction count, which has nothing to do with where
    /// the label actually is. This file's own witness, `interpret "signal
    /// there"`, does **not** collide even under a `Goto`-reuse build,
    /// because `there:` sits well past that one-instruction fragment's own
    /// length -- so that measurement alone was never evidence for this
    /// decision. `signal_out_of_a_fragment_does_not_collide_with_the_
    /// fragments_own_index_space` (this file's own tests) is a program
    /// built to collide instead (a label at index 2, a three-instruction
    /// fragment) and does fail under the reuse, printing a wrong branch's
    /// own output silently rather than crashing or hanging.
    ///
    /// **Nesting inside `DO`/`LOOP`, `IF` or `SELECT` needs no equivalent
    /// care**, a fact worth recording because it looks like it should:
    /// `rexx-parse` already rejects a label written inside any of the three
    /// (47.2, 47.3, 47.4, measured), so a `SIGNAL` target can never sit
    /// strictly inside a range `run_bounded` is currently absorbing a
    /// `Goto` into. Reusing `Goto` there would very likely have worked by
    /// construction; it is the fragment boundary alone that cannot
    /// tolerate it.
    ///
    /// Forwarded exactly like `Exit`/`Return` by every `run_bounded`/
    /// `do_body_outcome`/`leave_select`/`run_fragment` catch-all, and by
    /// `If`'s own true-branch arm (`step`'s `InstructionKind::If` handling)
    /// -- nothing in any of those needed its own arm for it -- and consumed
    /// only by `run_activation`'s own top-level dispatch, the same way
    /// `Goto` is.
    Signal(usize),
}

/// How one activation finished, which is not the same question as what value
/// it produced.
///
/// `run_activation` had a bare `Option<ObjRef>` through 4a, when every
/// activation was the program's only one and every way out of it stopped the
/// program. A callee has two ways out that differ in what the *caller* does
/// next, and the value alone cannot tell them apart -- `return` and `exit`
/// with the same value are the same `Option` and the opposite instruction.
///
/// Measured, and the distinction is not cosmetic: `call sub` / `say 'after'`
/// with `sub:` ending in `exit` never prints `after`, and with `sub:` ending
/// in `return` it does.
pub(crate) enum Ended {
    /// `RETURN`: the caller resumes at its next clause, with this value in
    /// `RESULT`.
    Returned(Option<ObjRef>),
    /// `EXIT`, **or the body running out of instructions.** The whole program
    /// stops. Falling off the end belongs here rather than with `Returned`
    /// and that is measured, not assumed: a callee whose label is the last
    /// thing in the file ends the program -- `trace r` / `call sub` / `say
    /// 'after'` / `exit` / `sub:` / `hh = 1` echoes the callee's clauses, then
    /// stops at rc 0 with `after` neither printed nor echoed.
    Exited(Option<ObjRef>),
}

impl Ended {
    /// The value, whichever way the activation finished -- what the *top*
    /// level wants, where the distinction carries no information.
    pub(crate) fn value(self) -> Option<ObjRef> {
        match self {
            Ended::Returned(value) | Ended::Exited(value) => value,
        }
    }
}

/// How many activations may be live at once before `CALL` raises 11.1
/// ("Insufficient control stack space", `Raised::insufficient_stack`).
///
/// **The oracle's own number is 27,314** (measured: unbounded `call sub`
/// recursion under `signal on syntax`, counting the depth reached), and this
/// limit is deliberately *not* it. What decides ours is where our own native
/// stack gives out, because one activation costs one `run_activation` Rust
/// frame (D19's choice, I6) and a native overflow is the silent death this
/// counter exists to convert into a reportable condition.
///
/// **Measured on this crate's own 512 MiB entry thread** (`lib.rs`'s
/// `on_interpreter_thread`), by bisecting the depth at which `rexx-run`
/// aborts. `cargo test`'s own debug profile is the binding one, and the
/// second column is what the value below is chosen against:
///
/// ```text
/// enclosing DO blocks per activation | deepest surviving (debug) | (release)
///                                  0 |                    22,534 |   133,150
///                                  1 |                    14,062 |    94,518
///                                  5 |                     5,616 |         -
///                                 25 |                     1,403 |         -
/// ```
///
/// The rows are ~23.8 KB for a bare activation and ~14.4 KB for each further
/// `run_bounded` level inside it, in debug -- `run_bounded` costs a Rust
/// frame per *lexical* nesting level, and the "0" row already includes one,
/// since the recursion is guarded by an `IF`.
///
/// **So no fixed counter over activations can be a guarantee, and that is
/// the honest reading of I34 rather than a caveat on it.** A body with about
/// four or more block levels around its own recursive `CALL` still aborts
/// natively before this fires: at 25 levels the abort is at 1,403, two
/// orders below. The counter converts the realistic shapes -- flat and
/// lightly nested recursion, which is what a recursive routine is -- and the
/// budget it shares with `run_bounded` (and, from Phase 5, dispatch) is what
/// a real fix has to bound. That is a documented minimum stack or a shared
/// depth budget, and it is not this task's.
///
/// **Nothing in this tree is on the small-stack cliff, but a `cargo test`
/// thread is.** Every public entry point spawns the sized thread; a test
/// reaching the crate internals directly does not, and on the default 2 MiB
/// this crate's debug build survives fewer than 90 activations (measured: 80
/// survives, 90 aborts). `tests/spike.rs`'s own recursion test says so at
/// its definition -- it was written as a unit test first and aborted the
/// binary.
///
/// The value is deliberately not the oracle's 27,314: that is above our own
/// debug abort in every row above, so matching it would mean shipping a
/// counter that never fires.
const MAX_ACTIVATION_DEPTH: usize = 10_000;

/// Where a `LEAVE`/`ITERATE` instruction itself sits, captured the instant
/// it steps rather than reconstructed later -- see `Flow::Leave`'s own doc
/// comment for why eagerly.
///
/// **`indent`'s own rule, corrected after review.** `indent` starts as
/// `static_indent` applied to the `LEAVE`/`ITERATE`'s own index -- its full
/// lexical depth, computed once when it steps -- and from there is the
/// search's own running **residual**, updated (not merely read) as the
/// `Flow` this is attached to propagates outward: every `SELECT` (always)
/// and every `DO`/`LOOP` that either `is_loop` or carries an explicit
/// `LABEL` (i.e. every one **except** an unlabelled `Simple` block) "owns a
/// search frame," and when such a construct examines this `Flow` and does
/// **not** consume it, it resets `indent` to *its own* `static_indent`
/// (`pop_search_frame`) before forwarding -- mirroring the oracle's own
/// `popBlockInstruction`, which restores `traceIndent` to the value saved
/// when the frame it is popping was pushed. A construct that *does*
/// consume the `Flow` (a match, successful or 28.5) does **not** reset
/// anything itself; whatever `indent` already holds at that point is the
/// answer. An unlabelled `Simple` block owns no frame and is fully
/// transparent, exactly like `IF` (which never even sees this `Flow` at
/// all, since it is not a block instruction and forwards everything
/// through ordinary fallthrough).
///
/// This first shipped as two hardcoded shapes -- 28.1-28.4 always zero,
/// 28.5 always the origin's own unmodified full lexical depth -- which
/// happened to match every probe behind it because none of them mixed an
/// `IF`/unlabelled-`Simple` intervenor with a `SELECT`/`DO`/`LOOP` one. A
/// reviewer's fourteen-point probe (nine of theirs, plus this task's own
/// five re-measured and added afterward) falsified seven of the fourteen
/// under that rule and fits all fourteen under this one; the report has
/// every transcript.
pub(crate) struct LeaveOrigin {
    /// `None` only when `source` was `None` at the moment this instruction
    /// stepped, which **no caller now produces**: `run_fragment` was the one
    /// that did, and since 4b's Task 2 it passes its own fragment source, so
    /// a `LEAVE`/`ITERATE` inside fragment text resolves a real site and
    /// becomes the report's innermost echo. See `Interp::clause_site`.
    site: Option<(usize, Vec<u8>)>,
    indent: usize,
    /// This `LEAVE`/`ITERATE` clause's own line, captured the same way and at
    /// the same moment as `site` and `indent`, and for the same reason.
    ///
    /// **Not `site`'s own line**, which is the fragment-relative one inside
    /// an `INTERPRET`: this is `SIGL`'s quantity, so it honours
    /// `clause_line_override` -- measured, a loop with an `ITERATE` inside
    /// `interpret` text reports the enclosing `INTERPRET` clause's line for
    /// the re-test, on the oracle and here.
    ///
    /// Read by `run_repeating`, through `do_body_outcome`, to answer "which
    /// clause does this pass's loop re-test belong to": the oracle re-enters
    /// the loop from whichever instruction transferred control back to it
    /// (`RexxInstructionEnd::execute` and `RexxActivation::iterate` both call
    /// `reExecute` themselves), so an `ITERATE` that cut a pass short owns
    /// the re-test that follows it.
    clause_line: usize,
}

/// What one pass of a `DO`/`LOOP` body just did, from `do_body_outcome`.
enum DoOutcome {
    /// The body ran off its end into the `END` clause.
    FellThrough,
    /// An `ITERATE` naming this construct cut the pass short. The line is
    /// that `ITERATE` clause's own -- what the loop's next re-test is
    /// attributed to, because the oracle re-enters the loop from inside
    /// `RexxActivation::iterate`, with the `ITERATE` still the current
    /// instruction.
    Iterated(usize),
    /// Stop: this `Flow` is the whole construct's final answer.
    Escaped(Flow),
}

/// What a repeating `DO`/`LOOP`'s own header clause decided: run the body
/// once more, or stop.
///
/// One type for the two ways to stop -- the control budget ran out, or a
/// `WHILE` tested false -- because `run_repeating` does the identical thing
/// for both, and a header clause that *failed* is the closure's own `Err`
/// rather than a third variant here.
enum HeaderOutcome {
    Continue,
    Stop,
}

impl ClauseValue for HeaderOutcome {
    /// Nothing to root: a loop header produces a decision, never a value
    /// whose only root was this clause's own temps frame.
    fn rooted(&self) -> Option<ObjRef> {
        None
    }
}

/// Which clause a repeating `DO`/`LOOP`'s own header evaluation -- the
/// control advance, a `WHILE` test, an `UNTIL` test -- belongs to on this
/// pass.
///
/// **The rule is the oracle's own architecture rather than a fitted table.**
/// The header is re-evaluated by `RexxInstructionBaseLoop::reExecute`, which
/// nothing calls on its own: it is called *by* the instruction that transfers
/// control back to the loop, and there are exactly three of those -- the
/// `DO`/`LOOP` clause itself on entry (`RexxInstructionBaseLoop::execute`),
/// the `END` clause when the body falls through
/// (`RexxInstructionEnd::execute`'s `LOOP_BLOCK` arm), and an `ITERATE`
/// (`RexxActivation::iterate`). Whichever of the three it was is still the
/// current instruction while the header runs, so it is that clause's line
/// `SIGL` reports and that clause's boundary a queued `CALL ON` handler is
/// delivered at.
///
/// Measured on all three, with no trap anywhere so it is a plain `SIGL`
/// question -- `do i = 1 to 3 while zs() < 3` with an `ITERATE` as the body's
/// last clause reports `2, 4, 4` (the `DO` line, then the `ITERATE`'s twice),
/// where the same loop without the `ITERATE` reports the `DO` line then
/// `END`'s.
enum HeaderClause {
    /// The first pass: the `DO`/`LOOP` clause's own line.
    Do,
    /// The previous pass fell through to `END`.
    End,
    /// The previous pass ended in an `ITERATE`, whose own line this is.
    Iterate(usize),
}

/// What drives one repeating `DO`/`LOOP`'s own iteration, once its header
/// has already been evaluated and validated -- everything `LoopKind` can be
/// except `Simple` (a block, never repeats, and `run_loop`'s own `Simple`
/// arm never builds one of these at all) and `With` (the loud path).
///
/// `Count`, `OverOnce` and `Controlled` all decrement whatever budget the
/// oracle is measured to decrement once per candidate iteration, including
/// one an `ITERATE` cuts short (measured: a `FOR 3` loop with an `ITERATE`
/// on its first pass still stops after exactly three iterations, not four).
enum LoopState {
    Forever,
    /// `DO expr`: a fixed repeat count, decremented to zero.
    Count {
        remaining: u64,
    },
    /// `DO name OVER expr`, a **non-stem** target only (Deviation 1: a stem
    /// target takes the loud path in `run_loop` before one of these is ever
    /// built): iterates exactly once, binding `control` to `value` itself
    /// (measured, the brief's own framing: "a string and a number each
    /// iterate once, yielding themselves"). `remaining` is `FOR`'s own
    /// budget, already validated, independent of `done`.
    OverOnce {
        control: SymbolId,
        value: ObjRef,
        done: bool,
        remaining: Option<u64>,
    },
    /// `DO i = initial TO to BY by FOR for_count`. `to`/`for_remaining` are
    /// `None` when that keyword was not written at all (an absent `TO`
    /// loops until `LEAVE` or `FOR` stops it, exactly like `FOREVER` with a
    /// control variable riding along); `by` is never absent here --
    /// `setup_controlled` already defaulted it to `1`.
    Controlled {
        control: SymbolId,
        current: Number,
        to: Option<Number>,
        by: Number,
        for_remaining: Option<u64>,
        /// Whether at least one candidate iteration has already been
        /// decided, which is exactly the oracle's own `!first` argument to
        /// `DoBlock::checkControl` (`ControlledDoInstruction.cpp:162`): it
        /// selects between "read the value the header computed" and
        /// "increment it, tracing on both sides of the addition". A `bool`
        /// on the state rather than a flag threaded through `run_repeating`
        /// because `loop_advance` is the only reader and the only writer,
        /// and because it has to survive an `ITERATE`, which re-enters that
        /// function without passing through the top of the driver's loop.
        stepped: bool,
    },
}

/// What `eval_condition` should do with the value it just computed, beyond
/// answering the caller's `bool` -- a caller-chosen variant rather than a
/// decision `eval_condition` makes on its own, because the same function
/// serves `IF`/`WHEN` (their own `>>>`, measured) and `WHILE`/`UNTIL`
/// (their own `>K>` instead, never a bare `>>>` alongside it, also
/// measured) and the two are genuinely different oracle behaviours, not
/// two spellings of one.
enum ConditionTrace<'a> {
    /// `IF`/`WHEN`'s own `>>>`.
    Result(usize),
    /// `WHILE`/`UNTIL`'s own `>K>`, tagged `"WHILE"`/`"UNTIL"`.
    Keyword(usize, &'a str),
}

impl Interp {
    // ---- the instruction loop, which is what this spike is for ----

    /// Runs the current activation's body to completion.
    ///
    /// **This function is the architectural claim.** The discipline is: clone
    /// the `Rc` into a local on entry, and derive every `&CodeBody` and
    /// `&Expr` from that local. It compiles for exactly one reason, and the
    /// reason is that `code` borrows `program` and `plan`, which are locals,
    /// rather than borrowing `self` -- so `self.step(…)`, which takes
    /// `&mut self`, has nothing to collide with.
    ///
    /// The version that does **not** compile, kept because the next phase to
    /// touch this will want to know which shape is wrong. Reaching the body
    /// through the activation and then stepping:
    ///
    /// ```text
    /// fn run_activation_wrong(&mut self) -> Result<Option<ObjRef>, Loud> {
    ///     let body = &self.activations.last().expect("a live activation").program.main;
    ///     while let Some(instruction) = body.instructions.get(self.activation().pc) {
    ///         self.step_wrong(body, instruction)?;
    ///     }
    ///     Ok(None)
    /// }
    /// ```
    ///
    /// The block below was captured by hand: the wrong version was written
    /// into this file, built, and deleted again, and this is what rustc 1.96.1
    /// printed. **Nothing re-checks it.** If the borrow checker's wording or
    /// its choice of underline changes, this text goes stale and no test
    /// fails, which is exactly why the doctests further down exist. Read it as
    /// a record of what was seen once, not as an assertion about what rustc
    /// does now:
    ///
    /// ```text
    /// error[E0502]: cannot borrow `*self` as mutable because it is also borrowed as immutable
    ///    --> crates/rexx-exec/src/lib.rs:851:13
    ///     |
    /// 849 |         let body = &self.activations.last().expect("a live activation").program.main;
    ///     |                     ---------------- immutable borrow occurs here
    /// 850 |         while let Some(instruction) = body.instructions.get(self.activation().pc) {
    ///     |                                       ----------------- immutable borrow later used here
    /// 851 |             self.step_wrong(body, instruction)?;
    ///     |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ mutable borrow occurs here
    /// ```
    ///
    /// Worth reading the second underline rather than the third: the borrow
    /// that survives is not the argument, it is the **loop condition**.
    /// Compiled here too, rather than reasoned about: replacing the body of
    /// the loop with a `self.step_wrong_noargs()` that takes no arguments at
    /// all gives the identical `E0502`, with `while body.instructions.get(…)`
    /// underlined as the later use. So handing the body to `step` some other
    /// way is not the fix. `Rc::clone` is the fix, because what has to change
    /// is where `body` is rooted, not who it is passed to.
    ///
    /// The `Rc::clone` is what removes it, and it is not a workaround for the
    /// borrow checker being conservative: the checker is right. An activation
    /// can be replaced under a running loop, and a `&CodeBody` reached through
    /// the activation would then point into a program the activation no longer
    /// holds. The `Rc` in the local is what makes that impossible rather than
    /// merely unlikely.
    ///
    /// # The pair of doctests below, and what they are worth
    ///
    /// **What keeps the function honest is the compiler.** This function would
    /// not build if the discipline broke, which is the whole reason the
    /// discipline is worth having and is a stronger guarantee than any test.
    /// **What the pair below keeps honest is the documentation**, which is the
    /// part with no compiler behind it: everything above is prose, and nothing
    /// in the tree would notice if it stopped describing anything real.
    ///
    /// So the two snippets are a miniature of the same borrow, once in the
    /// shape that compiles and once in the shape that does not. `cargo test`
    /// runs both. A precondition that nothing states and that the pair
    /// silently depends on: **rustdoc does collect doctests on private
    /// items.** Confirmed rather than assumed, by putting a deliberately
    /// failing snippet on a private method and watching it run. If it did not,
    /// both of these would not exist rather than fail, and the pair would be
    /// decoration.
    ///
    /// **Read them for what they prove and not more.** `compile_fail` proves
    /// only "this does not compile", not "this fails with `E0502`". The
    /// `compile_fail,E0502` spelling looks like it pins the code and does not:
    /// measured on rustc 1.96.1, a doctest annotated `compile_fail,E0502`
    /// whose body is `let x: u32 = "not a u32";` passes, and that is `E0308`.
    /// A `compile_fail` snippet with a typo in it therefore passes for the
    /// wrong reason, which is the standard trap with this attribute.
    ///
    /// What narrows it is the **first** snippet, which must compile. The two
    /// differ only in how `body` is obtained, two lines against one, and share
    /// everything else, so any breakage in the shared part fails the passing
    /// twin instead of silently satisfying the failing one. Checked by
    /// mutation rather than assumed: rewriting the passing snippet's
    /// `Rc::clone` line into the failing snippet's shape makes it fail, and it
    /// fails with `E0502`. One test says "the fix works", the other says "the
    /// shape it fixes is still broken", and neither is worth much alone.
    ///
    /// Two residuals, the second larger than the first and worth stating
    /// because the pair is easy to over-read.
    ///
    /// A typo confined to the failing snippet's own `let body = …` line passes
    /// for the wrong reason and nothing here closes that.
    ///
    /// And **the miniature can drift away from this function.** It models
    /// `Rc<CodeBody>` over a `Vec<u32>` with a two-argument `step`, where the
    /// real thing has `Rc<Program>`, a three-field `Code<'a>` and a different
    /// `step`. Nothing ties the two together. A future rewrite of
    /// `run_activation` into a shape the miniature does not model leaves both
    /// doctests green while the prose above them describes a function that no
    /// longer exists. That is a real limit and not a reason to drop the pair:
    /// the compiler is still what stops the *function* going wrong, and the
    /// pair is still what stops this *comment* claiming a borrow error that
    /// the language no longer produces.
    ///
    /// Compiles, because `body` borrows the local `program`:
    ///
    /// ```
    /// use std::rc::Rc;
    /// struct CodeBody { instructions: Vec<u32> }
    /// struct Frame { program: Rc<CodeBody>, pc: usize }
    /// struct Interp { activations: Vec<Frame> }
    /// impl Interp {
    ///     fn step(&mut self, _instruction: &u32) {}
    ///     fn run(&mut self) {
    ///         let program = Rc::clone(&self.activations.last().unwrap().program);
    ///         let body = &program.instructions;
    ///         while let Some(instruction) = body.get(self.activations.last().unwrap().pc) {
    ///             self.step(instruction);
    ///             self.activations.last_mut().unwrap().pc += 1;
    ///         }
    ///     }
    /// }
    /// ```
    ///
    /// Does not compile, because `body` borrows `self`. One line different:
    ///
    /// ```compile_fail
    /// use std::rc::Rc;
    /// struct CodeBody { instructions: Vec<u32> }
    /// struct Frame { program: Rc<CodeBody>, pc: usize }
    /// struct Interp { activations: Vec<Frame> }
    /// impl Interp {
    ///     fn step(&mut self, _instruction: &u32) {}
    ///     fn run(&mut self) {
    ///         let body = &self.activations.last().unwrap().program.instructions;
    ///         while let Some(instruction) = body.get(self.activations.last().unwrap().pc) {
    ///             self.step(instruction);
    ///             self.activations.last_mut().unwrap().pc += 1;
    ///         }
    ///     }
    /// }
    /// ```
    pub(crate) fn run_activation(&mut self) -> Result<Ended, Failure> {
        // `code` is bound to the activation on top of the stack at entry,
        // while every `pc` read and write below goes to whatever is on top
        // *now*. Those are the same frame only because `step` leaves the
        // activation stack as it found it -- true for a fragment, which runs
        // inside the creating activation rather than pushing its own, and
        // true for a `CALL` only because the `Call` arm pops the callee
        // before it returns.
        //
        // **That is now a real risk and the compiler will not mention it.**
        // A `CALL` pushes an activation inside `step`, and if it ever
        // returned with the callee still on the stack, this loop would carry
        // on reading the callee's `pc` while executing the caller's body: a
        // wrong answer, not a borrow error, because both are plain field
        // accesses on `self`. The assertion below is what turns that into a
        // failure at the first instruction instead of a debugging session,
        // and it is why the `Call` arm's pop is unconditional across both the
        // `Ok` and the `Err` path.
        //
        // The body comes from the activation's own selector rather than being
        // hardcoded to `program.main` (Task 3): `body_of` is the one place
        // that mapping lives, shared with `BodyKey::directive`'s own.
        let program = Rc::clone(&self.activation().program);
        let plan = Rc::clone(&self.activation().plan);
        let selector = self.activation().body;
        // A selector that resolves to nothing is an internal inconsistency
        // and not a program error: it can only be built by a resolution step
        // that already looked the body up. Loud rather than a panic, matching
        // this crate's standing rule -- an abort is precisely the outcome
        // that rule exists to exclude.
        let Some(body) = body_of(&program, selector) else {
            return Err(Loud::missing_body().into());
        };
        let code = Code {
            body,
            symbols: &program.symbols,
            slots: &plan.by_symbol,
        };
        let depth = self.activations.len();

        while let Some(instruction) = code.body.instructions.get(self.activation().pc) {
            let index = self.activation().pc;
            // "Is this the first instruction executed in this activation" --
            // consumed here, one instruction at a time, and read by
            // `PROCEDURE` and `USE LOCAL` alone. A `Label` is transparent to
            // it: measured, `call sub` into `sub:` / `lbl2:` / `procedure`
            // runs, while the same with a `nop` in place of the second label
            // is error 17.1. So a label neither grants the permission nor
            // spends it.
            //
            // Cleared *before* the step and carried across it on
            // `Interp::procedure_permitted`, which `step` takes on entry --
            // that is what stops an `INTERPRET` fragment or an `IF` branch
            // inheriting it, measured through `sub: interpret "procedure"`
            // being 17.1. See the field's own doc comment.
            if !matches!(instruction.kind, InstructionKind::Label { .. }) {
                self.procedure_permitted =
                    std::mem::take(&mut self.activation_mut().first_instruction_pending);
            }
            // The failing clause's site, if any escapes, is resolved inside
            // `step_in_temps_frame` itself (Task 10's own doc comment there):
            // this call may nest arbitrarily deep through `If`/`Select`'s own
            // `run_bounded`, and only the *innermost* one has the failing
            // instruction in hand. `run` pops this activation on the way out,
            // so a site resolved any higher up than that would have nothing
            // left to resolve against.
            //
            // A condition raised inside an `INTERPRET` fragment arrives here
            // too, and this level records the enclosing `INTERPRET` clause --
            // but it is no longer the *only* thing recorded. The oracle
            // prints one echo per level, innermost first, each carrying the
            // enclosing clause's line number (measured, `interpret "say 2 &
            // 1"` on line 2):
            //
            // ```text
            //      2 *-* say 2 & 1
            //      2 *-* interpret "say 2 & 1"
            // ```
            //
            // **Both lines are produced now, and 4b's Task 2 is what closed
            // it** -- through 4a and 4b's Task 1 this reproduced only the
            // second. `run_fragment` passes `Some(&fragment.source)` so the
            // fragment's own spans resolve its clause *text*, the `Interpret`
            // arm puts the enclosing clause's line and indent in force for
            // the duration (`Interp::clause_line_override`,
            // `Interp::indent_offset`), and `seal_site_level` closes the
            // fragment's level so the first-wins slot is free for this one.
            // Three separate mechanisms, because the naive version -- pass
            // the source down and nothing else -- was built twice and
            // measured wrong twice, once on the line number and once on which
            // clause won the race.
            //
            // The same gap ran through `TRACE`, where nothing is raised at
            // all (review finding I1), and closed with it: measured, `trace
            // r` / `zz = 'nop'` / `interpret zz` now prints the oracle's
            // three lines rather than its first two, and `run.rs`'s own
            // `interpret_traces_the_text_it_is_about_to_run` asserts the
            // whole transcript instead of stopping one line short.
            // The condition-trap boundary (4b's Task 7), and the reason it
            // is *here* rather than inside `step_in_temps_frame`: one offer
            // per activation, made by the activation that is unwinding. A
            // nested `run_bounded` (an `IF` branch, a `WHEN` body) shares
            // this activation's traps and must not get a second offer, and
            // a callee's own offer already happened in its own copy of this
            // loop before `resolve_and_run_call` re-threw. See
            // `offer_to_trap` for the search rules and for why only a
            // `SIGNAL ON` trap can take a failure at all.
            let flow =
                match self.step_in_temps_frame(&code, index, instruction, Some(&program.source)) {
                    Ok(flow) => flow,
                    Err(failure) => self.offer_to_trap(&code, failure)?,
                };
            // **No clause boundary here any more** (fix round 3). It moved
            // inside `step_in_temps_frame`, which is the one place a clause
            // is stepped -- so this loop, `run_bounded`, and every future
            // caller get it without being enumerated. See `clause.rs`.
            match flow {
                Flow::Next => self.activation_mut().pc += 1,
                Flow::Goto(target) => self.activation_mut().pc = target,
                // `SIGNAL`, once its target has escaped every nested
                // construct and every `INTERPRET` fragment it fired from
                // (`Flow::Signal`'s own doc comment has why it cannot ride
                // `Goto` to get here). The only consumer, matching `Goto`'s
                // own arm exactly: `target` already resolved against this
                // activation's own body (`resolve_signal_target`), which is
                // exactly the body `code` above is bound to.
                Flow::Signal(target) => self.activation_mut().pc = target,
                Flow::Exit(value) => return Ok(Ended::Exited(value)),
                // The activation boundary `Flow::Return` was added to reach.
                // Every construct between the `RETURN` and here forwarded it
                // untouched; this is the one consumer.
                Flow::Return(value) => return Ok(Ended::Returned(value)),
                // Task 11: a `LEAVE`/`ITERATE` that reached the very top of
                // the program -- nothing anywhere, at any nesting depth,
                // ever matched it. This is the exhausted-search family,
                // 28.1 (bare `LEAVE`)/28.2 (bare `ITERATE`)/28.3 (named
                // `LEAVE`)/28.4 (named `ITERATE`). `origin.indent` already
                // holds this family's own answer by the time it gets here
                // -- every `Select`/`Do` frame the search walked through on
                // the way up has already reset it to its own `static_indent`
                // as it forwarded past (`LeaveOrigin`'s own doc comment has
                // the rule, corrected after review: it is **not** always
                // zero, only when every popped frame along the way happened
                // to sit at top level). 28.5 (a named `ITERATE` that *did*
                // match something, just not a loop) is a different family,
                // raised where the match was found, in `Select`/`Do`'s own
                // arms, and never reaches here.
                Flow::Leave(name, origin) => {
                    self.record_leave_failure(&origin);
                    let raised = match name {
                        None => raised_leave_no_loop(),
                        Some(n) => raised_leave_no_match(code.symbols.name(n).as_bytes()),
                    };
                    return Err(raised.into());
                }
                Flow::Iterate(name, origin) => {
                    self.record_leave_failure(&origin);
                    let raised = match name {
                        None => raised_iterate_no_loop(),
                        Some(n) => raised_iterate_no_match(code.symbols.name(n).as_bytes()),
                    };
                    return Err(raised.into());
                }
            }
            debug_assert_eq!(
                self.activations.len(),
                depth,
                "step left the activation stack changed, so this loop's `code` and its `pc` \
                 no longer describe the same frame"
            );
        }
        // Out of instructions. `Exited` and not `Returned`: measured, a
        // callee that runs off the end of the file ends the *program* and the
        // caller's next clause never runs. See `Ended::Exited`'s own doc.
        Ok(Ended::Exited(None))
    }

    /// Runs one instruction.
    ///
    /// `code` is the caller's, so everything reached through it outlives every
    /// `&mut self` call in here. The `Assignment` arm is the clearest case:
    /// `name` is a `&[u8]` pulled out of `code.symbols` and it stays valid
    /// across `self.eval(…)`, which the same slice read out of `self` would
    /// not.
    ///
    /// **Every `eval` result is rooted before anything else runs.** `eval`
    /// hands back an unrooted handle by design, so its caller owns the moment
    /// it becomes a root, and here that is a `push_temp` on the line after
    /// each call. The instruction loops open a temps frame around this call
    /// and close it after, so what is pushed here lives exactly one clause.
    /// Not doing it would happen to work in an ordinary run, because
    /// `Heap::alloc_with_uncollected` never collects on its own, and would
    /// become a use-after-free the day something does, found by chasing a
    /// wrong value rather than by a compiler message. **Task 16's
    /// collect-on-every-allocation mode is that something, opt in**, and this
    /// discipline is what it verified: with the mode on, deleting one
    /// `push_temp` in `eval_arithmetic` panics seven of the subset's
    /// programs.
    ///
    /// `index` is `instruction`'s own position in `code.body.instructions`,
    /// added for Task 10: `If` and `Select` need their own position to
    /// compute a branch's start (`index + 1`, past the `Then`/`When` node
    /// itself), and nothing else in `self.activation().pc` can stand in for
    /// it here, because a nested call (from inside `run_bounded`) is
    /// stepping an instruction the activation's own `pc` is not pointing at.
    ///
    /// `source`, also added for Task 10, is threaded through to `If`/
    /// `Select`'s own `run_bounded` calls purely so `step_in_temps_frame`
    /// can resolve *its own* clause when an error escapes from inside one --
    /// without it, an error raised several `run_bounded` levels deep would
    /// only ever be attributed to the outermost `If`/`SELECT` that
    /// `run_activation` itself was stepping, which is wrong for exactly the
    /// same reason `run_activation`'s own doc comment gives for popping this
    /// activation before resolving a site: the last place the failing
    /// instruction is in hand has to be the one that resolves it. **Every
    /// caller passes `Some` since 4b's Task 2**, `run_fragment` included: a
    /// fragment resolves its clauses against its own source now, with the
    /// enclosing clause's line and indent supplied separately
    /// (`Interp::clause_site`'s own doc comment has why the two come apart).
    /// The parameter stays an `Option` only because collapsing it is a
    /// mechanical change across every signature that threads it.
    fn step(
        &mut self,
        code: &Code<'_>,
        index: usize,
        instruction: &Instruction,
        source: Option<&ProgramSource>,
    ) -> Result<Flow, Failure> {
        // Taken on entry, unconditionally, so that this call consumes it and
        // every nested `step` below it -- a fragment's, an `IF` branch's --
        // sees `false`. Only `run_activation` ever sets it. `Procedure` and
        // `Use` are the two arms that read it.
        let first_instruction = std::mem::take(&mut self.procedure_permitted);
        match &instruction.kind {
            InstructionKind::Say { expression } => {
                let line = match expression {
                    Some(expression) => {
                        let value = self.eval(code, expression)?;
                        self.roots.push_temp(value);
                        self.to_text(value).to_vec()
                    }
                    // `say` with no expression is a blank line. Still
                    // traced (`RexxInstructionExpression::
                    // evaluateStringExpression`'s own `else` arm:
                    // `traceResult(GlobalNames::NULLSTRING)`), as an empty
                    // string, not skipped.
                    None => Vec::new(),
                };
                self.trace_result(self.clause_state.current_value_indent, &line);
                self.out.extend_from_slice(&line);
                self.out.push(b'\n');
                Ok(Flow::Next)
            }

            InstructionKind::Assignment { target, value } => {
                // `addVariable` builds only `Variable`, `Stem` or `Compound`
                // targets (`ast.rs`'s own doc comment on `Assignment`), so the
                // `other` arm below is unreachable through any program that
                // parsed. Kept anyway, and exercised through `Loud::expression`
                // rather than `unreachable!`, for the same reason `Loud`'s own
                // exhaustive matches in `lib.rs` are: a guarantee the grammar
                // makes is not one the type system enforces, and this crate's
                // own rule is to fail loudly rather than trust it blindly.
                let value = self.eval(code, value)?;
                self.roots.push_temp(value);
                // `>>>` fires before the assignment itself
                // (`RexxInstructionAssignment::execute`: evaluate, trace,
                // *then* assign), which matters only in that the traced
                // value can never be affected by the write it precedes.
                // Reads `current_value_indent` rather than recomputing
                // `static_indent(index)` independently -- `step_in_temps_
                // frame` already computed exactly this value (`indent_
                // offset` included, F-EX1's own correction to F3) for this
                // same instruction right before calling `step`, and a
                // second computation of the identical quantity is how the
                // two drift, which is exactly what happened here before
                // this fix: this site's own copy never learned about the
                // offset when the field was added.
                let indent = self.clause_state.current_value_indent;
                let rendered = self.to_text(value).to_vec();
                self.trace_result(indent, &rendered);
                match &target.kind {
                    ExprKind::Variable(id) => {
                        let name = code.symbols.name(*id).as_bytes().to_vec();
                        let slot = self.slot_of(&name);
                        let frame = self.activation().frame;
                        self.roots.set_slot(frame, slot, value);
                        self.trace_assignment(indent, &name, &rendered);
                    }
                    // `stem. = expr`: replace-and-rebind (D15a), through the
                    // library `stem_assign` already builds -- this arm is the
                    // dispatch Task 9 owns, not new stem logic.
                    ExprKind::Stem(id) => {
                        let name = code.symbols.name(*id).as_bytes().to_vec();
                        self.stem_assign(&name, value);
                        self.trace_assignment(indent, &name, &rendered);
                    }
                    // `a.b = expr`: resolve the tail key the same way reading
                    // `a.b` would (`eval_node`'s own `Compound` arm), then
                    // mutate that one tail in place through `stem_set`.
                    //
                    // `>C>` before `>=>` (`RexxActivation.cpp:4791`-`4802`'s
                    // own order for a *read*; measured, this task's report,
                    // that a *write* through `ExpressionCompoundVariable::
                    // assign` announces the same resolved name first too):
                    // the tag is the compound's own **source spelling**
                    // (`a.i` stays `A.I` regardless of `i`'s value), and the
                    // resolved name is `stem_name` (the read site's own,
                    // matching `stem_set`'s own convention) concatenated
                    // with `key`.
                    ExprKind::Compound(id) => {
                        let tag = code.symbols.name(*id).as_bytes().to_vec();
                        let (stem_name, _tails) = compound_parts(code.symbols.name(*id));
                        let stem_name = stem_name.as_bytes().to_vec();
                        let key = self.tail_key(code, *id);
                        self.stem_set(&stem_name, &key, value);
                        let mut resolved = stem_name;
                        resolved.extend_from_slice(&key);
                        self.trace_compound_name(indent, &tag, &resolved);
                        self.trace_assignment(indent, &tag, &rendered);
                    }
                    other => return Err(Loud::expression(other).into()),
                }
                Ok(Flow::Next)
            }

            // A simple variable back to unset (`RootSet::clear_slot`, added
            // expressly for this and never `ObjRef::NIL`, which is a value
            // and not an absence -- `x = .nil; say x` and `y = .nil; drop y;
            // say y` render differently, measured in `drop_variable`'s own
            // doc comment), a whole stem, one tail, or the `(v)` indirect
            // form. See `drop_variable`.
            InstructionKind::Drop { variables } => {
                for variable in variables {
                    self.drop_variable(code, variable)?;
                }
                Ok(Flow::Next)
            }

            // `NUMERIC DIGITS`/`FUZZ`/`FORM`, every spelling `NumericSetting`
            // has. See `exec_numeric`.
            InstructionKind::Numeric {
                setting,
                expression,
            } => {
                self.exec_numeric(code, setting, expression)?;
                Ok(Flow::Next)
            }

            // `TRACE` (D17): sets the running activation's own trace mode, or
            // raises 24.901 for
            // the interactive-only skip-count forms. See `exec_trace`.
            InstructionKind::Trace(setting) => {
                self.exec_trace(code, setting)?;
                Ok(Flow::Next)
            }

            // `EXIT` with a result: the spike had only the bare form
            // (`expression: None` matched literally, nothing else reaching
            // this arm at all). The value crosses out of the instruction loop
            // as `Flow::Exit`, unconverted -- `Interp::exit_code_for` (`lib.rs`)
            // is what turns it into a process exit code, and it runs once in
            // `execute` rather than here, because a `Flow::Exit` can also
            // come from inside a fragment (`run_fragment`'s own propagating
            // arm, below), and the conversion needs nothing this loop knows
            // that `execute` does not already have.
            InstructionKind::Exit { expression } => {
                let value = match expression {
                    Some(expression) => {
                        let value = self.eval(code, expression)?;
                        // Rooted for exactly one clause, like every other
                        // `eval` result -- and that is shorter than this
                        // value actually needs. `step_in_temps_frame` pops
                        // this temp unconditionally right after `step`
                        // returns, before `Flow::Exit` ever reaches
                        // `run_activation`, so from that pop through `run`'s
                        // activation teardown and into `execute`'s
                        // `exit_code_for` call, this `ObjRef` is an
                        // **under-rooted** value -- longer and later than
                        // any other window in this crate. Benign only
                        // because nothing on that path allocates or
                        // collects (`Heap::alloc_with_uncollected` never
                        // collects on its own, and `to_number`/`to_text` read
                        // an existing object rather than making one, which is
                        // why Task 16's stress mode never fires inside this
                        // window either); once a collector exists,
                        // this needs a root that survives past the
                        // temps-frame pop -- a global, or a dedicated field
                        // on `Interp` -- rather than the one-clause
                        // `push_temp` every other instruction result gets.
                        self.roots.push_temp(value);
                        // `>>>`, at this `EXIT`'s own clause indent (Task 9).
                        // Measured on a three-line program with no condition
                        // and no call in it -- `trace r` / `say 'a'` /
                        // `exit 0` traces `>>>   "0"` after the `exit 0`
                        // echo -- so this belongs to the instruction, not to
                        // anything around it, and the bare form (`expression:
                        // None`) traces no line at all because there is no
                        // value: `RexxInstructionExit::execute`
                        // (`ExitInstruction.cpp`) evaluates through
                        // `RexxInstructionExpression::evaluateExpression`
                        // (`RexxInstruction.cpp:223`-`235`, read directly),
                        // whose own `traceResult` runs only inside the
                        // `expression != OREF_NULL` arm.
                        let rendered = self.to_text(value).to_vec();
                        self.trace_result(self.clause_state.current_value_indent, &rendered);
                        Some(value)
                    }
                    None => None,
                };
                Ok(Flow::Exit(value))
            }

            // A label is a traced no-op: the C++'s own `execute` on a label
            // instruction only traces it (Task 13's own construct -- a
            // `Label` clause is echoed here via `step_in_temps_frame`, same
            // as any other instruction) and does nothing else besides.
            // `SIGNAL`/`CALL` reach a label by jumping to the instruction
            // after it; nothing ever executes the label node for its own
            // effect.
            InstructionKind::Label { .. } => Ok(Flow::Next),

            InstructionKind::Nop => Ok(Flow::Next),

            // `INTERPRET expr`: evaluate to a string, parse it as a fragment,
            // run it against **this** activation. 4a built the machinery
            // (`run_fragment`) and 4b's Task 1 is the keyword; the arm used to
            // be gated on an `interpret_spike` flag that only a test entry
            // point set, and deleting that flag is the whole of the keyword's
            // implementation, because `run_fragment` already did the work.
            //
            // The `Flow` `run_fragment` answers is forwarded unchanged.
            // `Flow::Exit` crossing this arm is what makes `interpret "exit"`
            // end the program rather than the fragment (measured, and pinned
            // by `an_exit_inside_a_fragment_ends_the_program` in `lib.rs`);
            // `Flow::Leave`/`Iterate` can no longer reach here at all, since
            // the search does not cross the boundary -- `run_fragment`'s own
            // doc comment has the oracle transcripts.
            InstructionKind::Interpret { expression } => {
                let value = self.eval(code, expression)?;
                self.roots.push_temp(value);
                let text = self.to_text(value).to_vec();
                // `>>>` on the interpreted text itself, before the fragment
                // runs -- the same `trace_result` every other value-producing
                // arm calls (`Say`, `Assignment`), at the same
                // `current_value_indent`. Review finding I1(a): the arm
                // shipped without this and was the only value-producing arm in
                // the crate that traced nothing. Measured (`trace r`, `zz =
                // 'nop'`, `interpret zz`), the oracle prints
                //
                // ```text
                //      3 *-* interpret zz
                //        >>>   "nop"
                //      3 *-* nop
                // ```
                //
                // and re-measured one `DO` deeper, where the `>>>` picks up
                // that construct's own two spaces exactly as `Say`'s does.
                //
                // **Before `run_fragment`, not after, and that is not
                // cosmetic**: the fragment's own stepping overwrites
                // `current_value_indent` on every instruction it runs, so
                // reading the field afterwards would report the *fragment's*
                // last indent for the enclosing clause's own value.
                //
                // The third line above -- the fragment's own clause echo --
                // is what the `run_fragment` call below now produces, and it
                // took the whole of 4b's Task 2 rather than the one-line
                // change it looks like. Handing `run_fragment` a
                // `Some(&fragment.source)` and nothing else was built twice
                // and measured wrong twice, for two independent reasons: the
                // fragment's spans carry the *fragment's* line numbering
                // where the oracle prints the enclosing `INTERPRET` clause's
                // (measured, a raise inside fragment text reports line 3 and
                // the naive fix reports line 1), and the fragment's clause
                // would win `record_failure_at`'s first-wins race, taking the
                // report off the enclosing clause instead of adding to it.
                // The two lines below are the fix for the first and
                // `run_fragment`'s own `seal_site_level` is the fix for the
                // second.
                self.trace_result(self.clause_state.current_value_indent, &text);
                // **The fragment's level, with delta 0.** Measured: a
                // fragment's clauses print at the enclosing `INTERPRET`
                // clause's own absolute indent plus whatever nests them
                // *inside* the fragment -- `interpret "do jj = 1 to 1; say 2
                // & 1; end"` at top level echoes the inner clause at 2 and
                // the `INTERPRET` at 0, and the identical fragment two `DO`s
                // deep echoes them at 6 and 4. So the base is the enclosing
                // clause's printed indent exactly, with no bump of its own --
                // unlike a called routine's, which the same measurements put
                // two spaces further in (`call sub1` at printed indent 4 into
                // a flat routine echoes the callee's clause at 6), and which
                // is Task 3's to add.
                //
                // **The rule above is complete except after a repetitive
                // `DO`/`LOOP` that completed a body pass and then ended on a
                // failing control test** -- count exhausted, `WHILE` false or
                // `UNTIL` true alike. `static_indent` is a pure function of
                // lexical nesting and the oracle's own counter is not: such a
                // loop leaves later clauses two spaces lower. A zero-trip loop
                // and one left by `LEAVE` do not do it, so the property is "a
                // pass completed", not "a re-test failed".
                //
                // **The cause is a C++ defect, and stating the cause is the
                // only version of this that has not needed correcting.**
                // `traceIndent` is a counter; a loop ending normally restores
                // the value `DoBlock` saved, while a loop whose control test
                // fails takes a different exit path that bare-decrements it
                // (`BaseDoInstruction.cpp:161` against `:377`). So the stray
                // decrement survives exactly until some enclosing construct
                // restores from its own saved block, and is discarded there.
                //
                // Four earlier revisions of this comment each stated a rule
                // about *constructs* instead, and each drifted: the
                // qualification predicate, the scope, accumulation, and the
                // discarding class. `phase-4-exclusions.txt`'s row has the
                // C++ citations and the measured tables. **Do not write a
                // fifth construct-shaped rule here** -- if a shape is not in
                // a table, work out which exit path it takes.
                //
                // That is a 4a divergence with nothing to do with fragments,
                // but it reaches this base from both sides --
                // `interpret "do jj = 1 to 1; nop; end; say 1/0"` one `DO`
                // deep reports 2 against the oracle's 0, and a completed loop
                // *inside* a fragment lowers the **enclosing** program's later
                // clauses, so the base is computed from an indent that has
                // already drifted. See `phase-4-exclusions.txt`'s KNOWN GAP
                // row on the re-tested pass's own *indent*, which is where
                // this symptom lives -- the same pass's missing value lines
                // were a different mechanism and closed separately at Task 9.
                //
                // `activation_indent` is the mechanism (`lib.rs`'s own doc
                // comment on the field), **set rather than added**, and
                // `indent_offset` is zeroed alongside it: the enclosing
                // clause's `current_value_indent` already contains whatever
                // escape elevation was in force, so leaving that field alone
                // would count it twice -- measured on an `INTERPRET` in an
                // escaped `OTHERWISE`'s own body one `DO` deep, where the
                // oracle prints the fragment's clause at 12 and the
                // double-counting version prints 16. A fragment is a fresh
                // level, so it starts with a fresh escape elevation.
                //
                // All three are saved and restored rather than cleared, so a
                // fragment inside a fragment cannot strand the outer one's
                // values -- and restoring the line override is what makes the
                // *inner* fragment inherit the outer's line rather than
                // resolving one of its own, measured on `interpret 'interpret
                // "say 2 & 1"'` where all three echoes carry the outermost
                // line.
                let base_indent = self.clause_state.current_value_indent;
                let base_line = self.clause_site(source, instruction).map(|(line, _)| line);
                let saved_base = std::mem::replace(&mut self.activation_indent, base_indent);
                let saved_offset = std::mem::take(&mut self.indent_offset);
                let saved_line = std::mem::replace(&mut self.clause_line_override, base_line);
                let flow = self.run_fragment(text);
                self.activation_indent = saved_base;
                self.indent_offset = saved_offset;
                self.clause_line_override = saved_line;
                flow
            }

            // `IF`/`THEN`/`ELSE`. This arm resolves the whole construct
            // itself rather than leaving the outer loop to fall through the
            // flat list -- see `run_bounded`'s doc comment for why that is
            // not optional. `false_target`'s own doc comment: "the ELSE if
            // there is one, otherwise the instruction after the THEN
            // branch" -- confirmed by tracing `block.rs` by hand, it is the
            // `Else` instruction's own index when there is one, landing
            // *on* it rather than past it.
            InstructionKind::If {
                condition,
                false_target,
            } => {
                let len = code.body.instructions.len();
                let false_target = false_target.unwrap_or(len);
                // Reads `current_value_indent` rather than recomputing
                // `static_indent(index)` -- same reasoning as `Assignment`'s
                // own arm, above.
                let indent = self.clause_state.current_value_indent;
                // **The `IF` clause ends when its condition has been
                // evaluated**, not when the branch it chose has finished
                // running (fix round 4, re-review finding NEW-2). In the
                // oracle `RexxInstructionIf::execute` evaluates the condition
                // and returns; `THEN` and everything under it are separate
                // instructions its own loop fetches, each with a clause
                // boundary of its own. Here the true branch runs inside this
                // same `step`, so without this the first clause of that
                // branch collected the boundary the `IF` owed. Measured: `if
                // sub() = 'SV'` on line 3 with `then say ...` on line 4
                // reports `SIGL` 3 on the oracle and reported 4 here. The
                // one-line spelling agrees either way, which is why five
                // rounds of probes never separated them.
                let line = self.clause_state.line();
                let holds = match self.in_clause(code, line, |it| {
                    it.eval_condition(
                        code,
                        condition,
                        ConditionTrace::Result(indent),
                        raised_if_not_logical,
                    )
                })? {
                    ClauseOutcome::Ended(exit) => return Ok(Flow::Exit(exit.value())),
                    ClauseOutcome::Ran(holds) => holds?,
                };
                if holds {
                    let resume = self.skip_else(code, false_target);
                    match self.run_bounded(code, index + 1, false_target, source)? {
                        Flow::Next => Ok(Flow::Goto(resume)),
                        other => Ok(other),
                    }
                } else {
                    // Nothing to skip on the false path: whether
                    // `false_target` names an `Else` (whose own body then
                    // runs, and only traces on the way in, per its own doc
                    // comment) or is simply where control resumes with no
                    // `ELSE` at all, the outer loop's ordinary fallthrough
                    // already lands in the right place with no ambiguity --
                    // that is only true of the false path. See
                    // `run_bounded`'s doc comment for why the true path
                    // cannot rely on the same thing.
                    Ok(Flow::Goto(false_target))
                }
            }

            // A pure marker: only ever reached inside `If`'s own bounded
            // sub-loop (the true branch, right after the `IF`) or via
            // ordinary fallthrough on the false path. Never independently
            // dispatched for a decision of its own.
            InstructionKind::Then => Ok(Flow::Next),

            // Also a pure marker (`ast.rs`'s own doc comment: "executing an
            // ELSE only traces"). Reached only by ordinary fallthrough on
            // the false path -- the true path's `Goto` in the `If` arm above
            // skips straight past it to `then_exit`, so this is never asked
            // to decide anything.
            InstructionKind::Else { .. } => Ok(Flow::Next),

            // `SELECT`/`SELECT CASE`. Evaluates `case` at most once (if this
            // is a `SELECT CASE`), then tests each of its own *listed*
            // `whens` in source order by reading the `When`/`WhenCase` node
            // directly as data (`condition`/`values`, `false_target`,
            // `exit`) rather than dispatching through `step_in_temps_frame`
            // -- a *listed* `When`/`WhenCase` node (one collected into this
            // `whens` list, `ast.rs`'s own doc comment) must never be
            // independently stepped for a decision of its own, only ever
            // run past inside a bounded sub-loop. An *absorbed* one (never
            // collected here at all, because it is itself another `When`/
            // `WhenCase`'s own `THEN`) is the exception, and is
            // independently stepped -- see the `When`/`WhenCase` arm,
            // below, for both halves.
            InstructionKind::Select {
                label,
                case,
                whens,
                otherwise,
                end,
            } => {
                let len = code.body.instructions.len();
                // Reads `current_value_indent` rather than recomputing
                // `static_indent(index)` -- same reasoning as `Assignment`'s
                // own arm.
                let select_indent = self.clause_state.current_value_indent;
                // **The `SELECT` clause ends when its `CASE` expression has
                // been evaluated** -- same rule and same reason as `IF`'s,
                // just above: the oracle's `RexxInstructionSelect::execute`
                // returns here, and every `WHEN` after it is an instruction
                // of its own. Measured: `select case sub()` on line 3 with
                // `when 'SV' then say ...` on line 4 reports `SIGL` 3 on the
                // oracle and reported 4 here. Entered unconditionally, `CASE`
                // or no `CASE`, because the oracle's boundary is after the
                // instruction rather than after the expression -- a plain
                // `SELECT` simply has nothing that could have queued.
                let select_line = self.clause_state.line();
                let mut case_text: Option<Vec<u8>> = None;
                match self.in_clause(code, select_line, |it| {
                    let Some(case_expr) = case else {
                        return Ok(());
                    };
                    let value = it.eval(code, case_expr)?;
                    it.roots.push_temp(value);
                    let text = it.to_text(value).to_vec();
                    // `>K>` (`SelectInstruction.cpp:372`,
                    // `traceKeywordResult(CASE, ...)`), at the
                    // `SELECT`'s own level -- measured, this task's
                    // report, `>K>   "CASE" => "2"` sits at the same
                    // indent as `select case ...` itself, not the
                    // `WHEN`-scan level `WhenCase`'s own comparison
                    // lines (below) are indented to.
                    it.trace_keyword(select_indent, "CASE", &text);
                    case_text = Some(text);
                    Ok(())
                })? {
                    ClauseOutcome::Ended(exit) => return Ok(Flow::Exit(exit.value())),
                    ClauseOutcome::Ran(ran) => ran?,
                }
                // F3: the one hand-off an absorbed `WhenCase` needs and
                // nothing else threads to it -- `lib.rs`'s own doc comment
                // on `current_case_text` has the full argument, including
                // the disclosed nested-`SELECT CASE` limitation.
                self.current_case_text = case_text.clone();
                for &when_index in whens {
                    let when_instruction = &code.body.instructions[when_index];
                    let when_indent = self.printed_indent(&code.body.instructions, when_index);
                    // Overrides the enclosing `SELECT`'s own
                    // `current_value_indent` (`step_in_temps_frame` set it
                    // to `select_indent` before this arm even started) for
                    // the same reason `scan_when`'s own explicit clause echo
                    // and `record_failure_site` calls exist: this condition
                    // is evaluated outside any `step_in_temps_frame` call of
                    // its own, so nothing else sets any of the three.
                    // **Each listed `WHEN` is a clause of its own, with its
                    // own boundary** (fix round 4, re-review finding NEW-2).
                    // In the oracle every `WHEN` is an instruction the
                    // activation's loop fetches separately, so a condition
                    // queued while testing one is delivered before the next
                    // `WHEN`, before `OTHERWISE`, and before a matched
                    // `WHEN`'s own body. All three measured, all three wrong
                    // before this: a false `when sub() = 'NO'` on line 4
                    // followed by a winning `when` on line 5 reported `SIGL`
                    // 5; the same falling to `OTHERWISE` did not deliver
                    // until after `OTHERWISE`'s body had already run and read
                    // the handler's variable unset; and a *true* `when sub()
                    // = 'SV'` on line 4 with `then` on line 5 reported 5.
                    let when_line = self
                        .clause_line(source, when_instruction)
                        .unwrap_or_else(|| self.clause_state.line());
                    let mut outcome: Option<(usize, usize)> = None;
                    let scanned = self.in_clause(code, when_line, |it| {
                        it.scan_when(
                            code,
                            source,
                            when_index,
                            when_instruction,
                            when_indent,
                            case_text.as_deref(),
                            len,
                            &mut outcome,
                        )
                    })?;
                    match scanned {
                        ClauseOutcome::Ended(exit) => return Ok(Flow::Exit(exit.value())),
                        ClauseOutcome::Ran(ran) => ran?,
                    }
                    if let Some((body_end, resume)) = outcome {
                        let flow = self.run_bounded(code, when_index + 1, body_end, source)?;
                        // **F-EX1, found by the whole-branch review, not by
                        // this task's own probes.** An absorbed `WhenCase`'s
                        // own false-branch escape (its own arm, below) can
                        // land exactly on this `SELECT`'s own `OTHERWISE`
                        // marker via a bare `Flow::Goto`, which `leave_
                        // select`'s own `other => Ok(other)` arm would
                        // otherwise forward unrecognised -- all the way out
                        // of this `SELECT`'s own `step` entirely, so
                        // `OTHERWISE`'s own body would then run under
                        // whichever *outer* construct happens to receive
                        // that `Goto`, with no `SELECT` frame on the search
                        // a `LEAVE`/`ITERATE` inside it needs to find. That
                        // is precisely the pre-Task-11 shape Task 11 built
                        // `run_otherwise` (below, this fix's own extraction
                        // of what was inline here) to fix in the first
                        // place -- measured, `select label s case 2 / when
                        // 2 then / when 3 then nop / otherwise say 'O' /
                        // leave s / end`: oracle `O`, `after`, rc 0; before
                        // this fix, `O`, then `Error 28.3`, rc 228, because
                        // `leave s` searched outward from *outside* this
                        // `SELECT` and never found it. Redirecting through
                        // `run_otherwise` here, exactly as the ordinary "no
                        // `WHEN` matched" path already does below, is what
                        // restores the frame.
                        if let Flow::Goto(target) = flow
                            && *otherwise == Some(target)
                        {
                            // **Do not clear `indent_offset` here.** F-EX1's
                            // own re-review found the first version of this
                            // comment wrong: `OTHERWISE`'s own marker *and
                            // its whole body* need the offset still active
                            // through `run_otherwise`'s own dispatch (`lib.
                            // rs`'s own doc comment on `indent_offset` has
                            // the measured transcript) -- `run_otherwise`
                            // itself is what restores it to `0`, once that
                            // whole dispatch is over, not here before it
                            // has even started.
                            return self.run_otherwise(code, index, *label, target, *end, source);
                        }
                        return self.leave_select(code, index, *label, resume, flow);
                    }
                }
                match otherwise {
                    // Task 11: **used to be** a plain `Goto` onto the
                    // `OTHERWISE` marker, left for the outer loop's own
                    // ordinary fallthrough to run -- correct for clause
                    // attribution (nothing here ever needed a bounded
                    // sub-loop to get *that* right, and the fallthrough
                    // still lands exactly on `END` with nothing to skip,
                    // same as before), but wrong for `LEAVE`/`ITERATE`: this
                    // `SELECT` never gets a chance to recognise its own
                    // label from inside a branch it does not itself call
                    // `run_bounded` over. So `OTHERWISE`'s own body is now
                    // `[otherwise_index + 1, end)`, run the same way a
                    // matched `WHEN`'s is -- attribution is unaffected
                    // (`step_in_temps_frame`'s own resolution does not care
                    // which loop dispatched it), and a `LEAVE`/`ITERATE`
                    // naming this `SELECT`'s own label from inside
                    // `OTHERWISE` is now caught here too.
                    Some(otherwise_index) => {
                        self.run_otherwise(code, index, *label, *otherwise_index, *end, source)
                    }
                    // Landing exactly on `END` is deliberate: that is what
                    // makes 7.3's clause echo the `END`'s and not the
                    // `SELECT`'s (`End`'s own arm, below, is where it
                    // raises). No body ran, so there is nothing for a
                    // `LEAVE`/`ITERATE` to have escaped from here.
                    None => Ok(Flow::Goto(end.unwrap_or(len))),
                }
            }

            // **Fixed after review: this used to be a bare `Ok(Flow::Next)`,
            // and that was a silently wrong answer, not a formatting gap.**
            // A `When`/`WhenCase` is only ever reached here through the
            // absorbed-`WHEN` shape -- a `WHEN` whose own `THEN` consequence
            // is itself a `WHEN`/`WHEN CASE` clause, which the enclosing
            // `SELECT`'s own `whens` never collects (`ast.rs`'s own doc
            // comment on `whens`, `LanguageParser.cpp:1319`) -- since a
            // *listed* `When`/`WhenCase` is always fully handled by
            // `Select`'s own explicit arm, above, without ever calling
            // `step` on itself (its own body range never contains another
            // listed sibling's index).
            //
            // The old comment's own measurement was real but its conclusion
            // was wrong: `select / when 1=1 then / when 2=2 then n=42 /
            // otherwise / n=99 / end / say n` prints `0`, and that alone is
            // consistent with *never evaluating* condition B just as much
            // as with *evaluating it and discarding the answer*. The
            // measurement that tells the two apart is a raising absorbed
            // condition: `select / when 1=1 then / when 1/0 then nop / end`
            // is rc **214**, `Error 42.3`, on the oracle -- so B's condition
            // *is* evaluated for real, and simply never gets to take its
            // own branch (confirmed the other direction too: `when 2=2
            // then say 'x'` with no `otherwise` prints nothing but `after`,
            // not `x` -- true or false, the absorbed consequence never
            // runs). `Select`'s own arm already reasons this exactly right
            // for a *listed* `WHEN`'s condition (raise first, decide
            // second); this arm was the one place that reasoning did not
            // reach, because nothing before this task's own review probed
            // a raising absorbed condition -- every prior probe used a
            // side-effect-free true one, which cannot distinguish the two
            // models.
            //
            // `current_value_indent` is already correct here without any
            // extra work: this instruction *is* being stepped through the
            // ordinary `step_in_temps_frame` path (unlike a listed `WHEN`,
            // which `Select`'s own arm overrides it for explicitly), so the
            // clause echo and this trace both land at this absorbed
            // clause's own static indent.
            InstructionKind::When { condition, .. } => {
                self.eval_condition(
                    code,
                    condition,
                    ConditionTrace::Result(self.clause_state.current_value_indent),
                    raised_when_not_logical,
                )?;
                Ok(Flow::Next)
            }
            // `SELECT CASE`'s own absorbed form.
            //
            // **F3, fixed by review: unlike plain `WHEN`'s absorbed form,
            // this one *does* branch, on the false side.** Measured:
            // `select case 2 / when 2 then / when 3 then nop / otherwise
            // say 'O' / end / say 'after'` prints `O` then `after` on the
            // oracle; this crate, before this fix, printed only `after`.
            // Read the parsed field values directly rather than guessed
            // (`f3dbg`, a throwaway debug binary against `rexx_parse::
            // parse_program`) to find the mechanism: the *outer*, listed
            // `WhenCase`'s own `false_target` stops *before* the absorbed
            // `WhenCase`'s own body (it bounds `Select`'s own `run_bounded`
            // call to `[when_index+1, false_target)`, which ends exactly at
            // the absorbed node's own index), so the absorbed body is
            // structurally unreachable through that call *regardless* of
            // this arm's own answer when it does **not** need to branch.
            // The one case that does need to branch is a **false** match on
            // the absorbed condition: the absorbed node's own `false_target`
            // points past its own (unrun) body to whatever comes next in
            // the *enclosing* body (`OTHERWISE`, here) -- outside the range
            // the *outer* `WhenCase`'s own `run_bounded` call is bounded to,
            // so `Flow::Goto(false_target)` escapes it unchanged exactly the
            // way a `LEAVE`/`ITERATE` naming an enclosing construct already
            // does (`run_bounded`'s own doc comment).
            //
            // **The one further change: whatever this `Goto` lands on
            // reports every indent `self.indent_offset` spaces higher than
            // its own ordinary `static_indent`, for as long as that stays
            // non-zero.** Found by review, one perimeter deeper than the
            // fix above (`select case 2 / when 2 then / when 3 then nop /
            // end / say 'after'`, no `OTHERWISE`: `END`'s own 7.3 clause
            // reports at indent 4, not `END`'s own ordinary `0`) --
            // **and corrected once more by a second review** that found an
            // absolute-replacement version of this field right for `END`
            // only by coincidence (`0 + 4` and `4` are the same number) and
            // wrong for F-EX1's own `OTHERWISE` redirect just below, whose
            // ordinary marker level is `2`, not `0`. `lib.rs`'s own doc
            // comment on `indent_offset` has the full argument and the
            // measured `TRACE R` transcript pinning all three numbers
            // (the absorbed condition's own `6`, `OTHERWISE`'s own marker
            // at `6`, its own body at `8`) to one additive offset, `6 - 2`,
            // not three different rules -- and why it is carried through a
            // field rather than by growing `Flow::Goto` a payload every
            // ordinary resume-`Goto` would then have to carry too.
            //
            // **A true match still never runs its own consequence**,
            // confirmed by a dedicated probe (`t13_f3_true.rex`, this
            // task's report) rather than assumed from the false case's own
            // fix: "matches on both sides" is the coordinator's own
            // phrase for this, meaning this crate's pre-F3 behaviour (fall
            // through, `Flow::Next`) already agreed with the oracle for a
            // true absorbed match, and F3 is entirely the false path's own
            // fix.
            //
            // **The plain-`WHEN` sibling of this false path is
            // deliberately left alone.** Its own false-absorbed shape is
            // one line away from `select / when 1=0 then / when 2=2 then
            // nop / end`, SF #2018's segfault -- the oracle cannot answer
            // what a false absorbed plain `WHEN` should do because it
            // crashes before answering anything, so there is no oracle
            // byte to fix `InstructionKind::When`'s own arm against, and
            // probing to find out is explicitly out of scope (this task's
            // own coordinator, and `phase-4-exclusions.txt`'s standing rule
            // against reproducing that crash).
            //
            // `current_case_text` is `lib.rs`'s own new field, the one
            // hand-off a listed `WhenCase` already has (`case_text`,
            // passed directly) that an absorbed one otherwise has no way
            // to reach; `None` only if this is somehow absorbed inside a
            // plain `SELECT` with no `CASE` expression at all, which the
            // parser should never produce for a `WhenCase` node (only a
            // `SELECT CASE` ever builds one) -- kept as a fallback that
            // evaluates for side effects and never branches, rather than
            // an `unreachable!`, on this crate's own rule against turning
            // an unproven parser invariant into a crash.
            InstructionKind::WhenCase {
                values,
                false_target,
                ..
            } => match self.current_case_text.clone() {
                Some(case_text) => {
                    let indent = self.clause_state.current_value_indent;
                    if self.test_case_when(code, values, &case_text, indent)? {
                        Ok(Flow::Next)
                    } else {
                        // **Corrected after a second re-verification found
                        // the first version of this line wrong under
                        // nesting.** `current_value_indent.saturating_sub
                        // (2)` (this line's own first attempt) gave the
                        // right answer at the top level by coincidence
                        // (`6 - 2 = 4`) and the *wrong* one nested one `DO`
                        // deeper (`8 - 2 = 6`, where the oracle still wants
                        // `4`) -- measured directly (`t13_f3_nested.rex`,
                        // then a `TRACE R` transcript one level deeper
                        // again, `i_trace_nested_escape.rex`/`j_trace_
                        // nested_otherwise.rex`, this task's report has
                        // all three). The offset is the **constant** `4`,
                        // not a function of how deep the absorbed
                        // condition itself sits: it is exactly two
                        // `indent()` bumps -- the enclosing, listed
                        // `WHEN`/`WHEN CASE`'s own marker, then its own
                        // body entry -- past wherever an *ordinary* `SELECT`
                        // -level construct (`END`, `OTHERWISE`) would sit,
                        // and that gap is the same two bumps regardless of
                        // how many other constructs enclose the whole
                        // `SELECT`. Confirmed at both nesting depths for
                        // all three landing shapes (`END`, `OTHERWISE`'s
                        // own marker, `OTHERWISE`'s own body) before
                        // trusting it a second time.
                        self.indent_offset = 4;
                        Ok(Flow::Goto(
                            false_target.unwrap_or(code.body.instructions.len()),
                        ))
                    }
                }
                None => {
                    for value in values {
                        let v = self.eval(code, value)?;
                        self.roots.push_temp(v);
                    }
                    Ok(Flow::Next)
                }
            },
            InstructionKind::Otherwise => Ok(Flow::Next),

            // `DO`/`LOOP`, every kind but `DO WITH` (the loud path,
            // `run_loop`'s own doc comment) -- Task 11. Resolves the whole
            // construct itself, every iteration, exactly the discipline
            // `If`/`Select` already established: see `Flow::Leave`'s own
            // doc comment for why `Do`'s own arm never returns until the
            // entire loop is over, one way or another.
            InstructionKind::Do(body) | InstructionKind::Loop(body) => {
                self.run_loop(code, index, instruction, body, source)
            }

            // `LEAVE`/`ITERATE`, bare or by name -- Task 11. Resolves to
            // data, not a failure (`Flow::Leave`'s own doc comment): whether
            // this instruction's own name matches anything is answered by
            // whichever `Do`/`Select` (or `run_activation`'s own top level,
            // if none does) inspects the `Flow` this returns, never here.
            InstructionKind::Leave { name } => Ok(Flow::Leave(
                *name,
                self.leave_origin(code, index, source, instruction),
            )),
            InstructionKind::Iterate { name } => Ok(Flow::Iterate(
                *name,
                self.leave_origin(code, index, source, instruction),
            )),

            // `END`. `Select`'s two non-7.3 closings (`OTHERWISE` present)
            // are reached only by that `OTHERWISE`'s own ordinary body
            // fallthrough and do nothing. `EndStyle::Select`'s own doc
            // comment: "Reaching this END at run time is error 7.3, because
            // every WHEN was false" -- the ordinary way to land here, but
            // **not the only one since F3**: an absorbed `WhenCase`'s own
            // false-branch escape (`InstructionKind::WhenCase`'s own arm,
            // above) can also `Goto` straight onto this exact instruction,
            // carrying its own residual indent along in `pending_escape_
            // indent` for this arm's own 7.3 to be reported at rather than
            // this position's ordinary `static_indent`. An earlier version
            // of this comment said `Select`'s own arm "sends every other
            // path around this instruction entirely," which was true of
            // every path *it* controls directly and false of this one,
            // which escapes through it rather than being dispatched by it.
            //
            // **`EndStyle::Do`/`LabeledDo`/`Loop` used to fail loudly here,
            // and no longer do.** Task 11's `Do`/`Loop` arm now resolves its
            // own construct exactly the way `If`/`Select` already do,
            // returning a `Goto` past this exact instruction on every exit
            // path -- normal completion, a consumed `LEAVE`, or `UNTIL`
            // finally holding. So this is reached only inside a bounded
            // sub-loop, as inert filler, precisely like `Then`/`Else`/
            // `When`/`Otherwise` above; nothing independently dispatches it
            // for a decision of its own, and a plain no-op is what those
            // four already do in that position.
            InstructionKind::End { closes, .. } => {
                let closes = closes
                    .as_ref()
                    .expect("an End's closes is only None while its body is still being assembled");
                match closes.style {
                    EndStyle::Select => Err(raised_select_no_when().into()),
                    EndStyle::Otherwise
                    | EndStyle::LabeledOtherwise
                    | EndStyle::Do
                    | EndStyle::LabeledDo
                    | EndStyle::Loop => Ok(Flow::Next),
                }
            }

            // `CALL name`, `CALL "name"` and `CALL (expr)`. The other two
            // arms of `rexx_parse::Call` stay loud and keep their own owners
            // (`instruction_owner`, `lib.rs`): `Trap` (`CALL ON`/`CALL OFF`)
            // is Task 7's, `Qualified` (`CALL ns:name`) is Phase 5's.
            InstructionKind::Call(call) => match &**call {
                // `name` arrives already upcased for the symbol form and
                // verbatim for the quoted one (`rexx-parse`'s own `Call`
                // doc). `literal` inverts into "may this search the label
                // table": measured, `call "SUB"` with `sub:` present is
                // Error 43.1 and not a call, so the quoted form bypasses the
                // search entirely rather than merely matching case-sensitively.
                rexx_parse::Call::Named {
                    name,
                    literal,
                    args,
                } => self.exec_call(code, name, !*literal, args),
                // `CALL (expr)`: the target is evaluated in the caller, its
                // value is traced, and the **verbatim** text is what the
                // label search sees. Both halves are measured and they pull
                // in opposite directions from the quoted form: `nm = 'SUB';
                // call (nm)` runs `sub:`, so this form *does* search labels,
                // while `nm = 'sub'; call (nm)` is Error 43.1 `Could not find
                // routine "sub"`, so the value is not upcased on the way in.
                rexx_parse::Call::Dynamic { target, args } => {
                    let value = self.eval(code, target)?;
                    self.roots.push_temp(value);
                    let name = self.to_text(value).to_vec();
                    // Its own `>>>`, at the `CALL` clause's own indent, which
                    // `Call::Named` has no equivalent of -- measured, `call
                    // sub 1+1, 'q'` under `trace r` traces no value line at
                    // all while `call (nm)` traces one for the target.
                    self.trace_result(self.clause_state.current_value_indent, &name);
                    self.exec_call(code, &name, true, args)
                }
                // `CALL ON cond NAME label` / `CALL OFF cond`. Shares every
                // line of its implementation with `SIGNAL ON`/`OFF` except
                // the one `bool` that decides how the handler runs -- see
                // `exec_condition_trap`, and `Trap`'s own doc comment
                // (`activation.rs`) for the two measured behaviours that
                // `bool` selects between.
                rexx_parse::Call::Trap(trap) => self.exec_condition_trap(trap, true),
                rexx_parse::Call::Qualified { .. } => {
                    Err(Loud::instruction(&instruction.kind).into())
                }
            },

            // `RETURN`, bare or with a value. Unwinds to the activation
            // boundary; `Flow::Return`'s own doc comment has why none of the
            // other variants expresses that, and why the main body's own
            // `RETURN` ends the program.
            //
            // The value's `>>>` fires **here**, at the `RETURN`'s own clause
            // indent, and the caller traces a *second* one at its own --
            // measured, `return 9` from a routine called at top level prints
            // `>>>     "9"` then `>>>   "9"`, two lines for one value at two
            // indents. `exec_call` owns the second; this owns the first.
            InstructionKind::Return { expression } => {
                let value = match expression {
                    Some(expression) => {
                        let value = self.eval(code, expression)?;
                        self.roots.push_temp(value);
                        let rendered = self.to_text(value).to_vec();
                        self.trace_result(self.clause_state.current_value_indent, &rendered);
                        Some(value)
                    }
                    None => None,
                };
                Ok(Flow::Return(value))
            }

            // `SIGNAL label` and `SIGNAL VALUE`. `Signal::Trap` (`SIGNAL
            // ON`/`SIGNAL OFF`) stays loud, Task 7's own owner
            // (`instruction_owner`, `lib.rs`).
            InstructionKind::Signal(signal) => match &**signal {
                // `name` is already upcased for a bare symbol and verbatim
                // for a quoted one (`rexx-parse`'s own `Signal` doc), and
                // **both forms search the label table** -- unlike `CALL
                // "name"`, which never does, because `SIGNAL` has no
                // builtin/external fallback for a literal spelling to
                // deliberately bypass into. Measured: `signal "sub"` with
                // `sub:` present still raises 16.1 (case-sensitive against
                // the label's own upcased spelling, so the lowercase quoted
                // form does not match), while `signal Sub` (bare, mixed
                // case) and `signal "SUB"` both run it.
                rexx_parse::Signal::Label(name) => {
                    let target = self.resolve_signal_target(name)?;
                    // Set only once the target actually resolves -- an
                    // unresolved `SIGNAL` (16.1) ends the program regardless,
                    // matching the oracle's own `signalTo`, which a caller
                    // only ever invokes with an already-resolved target.
                    self.set_sigl(self.clause_state.line());
                    Ok(Flow::Signal(target))
                }
                // `SIGNAL VALUE expr`. Its own `>K>` -- `"VALUE" => text`, at
                // this clause's own indent with no `+2` the way `WHILE`/
                // `UNTIL` carry (measured one `DO` deep: `signal value
                // target` traces `>K>     "VALUE" => "THERE"` at the same
                // indent as its own clause echo, unlike those two, which are
                // evaluated as part of the *enclosing* `DO`/`LOOP`'s own
                // step). The rendered text is then searched exactly like
                // `Label`'s own bytes, with **no shape check on the value at
                // all** -- measured, a number, an empty string and an
                // ordinary non-label string all raise 16.1 naming that exact
                // text, none of them a different error.
                rexx_parse::Signal::Value(expr) => {
                    let value = self.eval(code, expr)?;
                    self.roots.push_temp(value);
                    let text = self.to_text(value).to_vec();
                    self.trace_keyword(self.clause_state.current_value_indent, "VALUE", &text);
                    let target = self.resolve_signal_target(&text)?;
                    self.set_sigl(self.clause_state.line());
                    Ok(Flow::Signal(target))
                }
                // `SIGNAL ON cond NAME label` / `SIGNAL OFF cond`. Unlike the
                // two arms above it transfers no control of its own: it edits
                // this activation's trap table and falls through to the next
                // clause, and the transfer happens later, if the condition is
                // ever raised.
                rexx_parse::Signal::Trap(trap) => self.exec_condition_trap(trap, false),
            },

            // `PROCEDURE`, bare or with an `EXPOSE` list (D9r). Isolates the
            // callee's variable pool and aliases the exposed names back into
            // the pool they came from. See `exec_procedure`.
            InstructionKind::Procedure { variables } => {
                self.exec_procedure(code, variables, first_instruction)?;
                Ok(Flow::Next)
            }

            // `USE ARG`/`USE STRICT ARG`/`USE LOCAL`. See `exec_use`.
            InstructionKind::Use(use_) => {
                self.exec_use(code, use_, first_instruction)?;
                Ok(Flow::Next)
            }

            // `RAISE`, in all of its forms. See `exec_raise`, whose doc
            // comment carries the delivery table -- which is the whole of
            // this instruction and is not derivable from the grammar.
            InstructionKind::Raise(raise) => self.exec_raise(code, raise),

            // `PUSH`/`QUEUE line` (4b's Task 8, I15). Both evaluate to string
            // form and trace the result exactly like `SAY` above -- the
            // oracle's own `RexxInstructionQueue::execute` shares `SAY`'s
            // `RexxInstructionExpression::evaluateStringExpression`
            // (`QueueInstruction.cpp:69`), differing only in which end of
            // the queue the value lands on, decided below by which variant
            // matched (review round 1's M4: one arm, not two copies that can
            // drift). See `queue.rs`'s own module doc for the measured LIFO
            // (`PUSH`)/FIFO (`QUEUE`) order and why nothing here reads a
            // line back: `PULL`, `PARSE PULL` and `QUEUED()` are all 4c's.
            InstructionKind::Push { expression } | InstructionKind::Queue { expression } => {
                let line = match expression {
                    Some(expression) => {
                        let value = self.eval(code, expression)?;
                        self.roots.push_temp(value);
                        self.to_text(value).to_vec()
                    }
                    // No expression queues a null string, traced as one --
                    // the same `else` arm `SAY`'s own blank line takes.
                    None => Vec::new(),
                };
                self.trace_result(self.clause_state.current_value_indent, &line);
                if matches!(instruction.kind, InstructionKind::Push { .. }) {
                    self.queue.push(line);
                } else {
                    self.queue.queue(line);
                }
                Ok(Flow::Next)
            }

            other => Err(Loud::instruction(other).into()),
        }
    }

    /// `PROCEDURE`, with or without an `EXPOSE` list (D9r).
    ///
    /// Two things happen here, in this order, and the order is the design:
    /// every exposed name is resolved **while the caller's frame is still the
    /// top one**, and only then does the callee get a frame of its own. That
    /// is what lets a computed `expose (v)` naming a symbol no instruction
    /// mentions go through `Interp::slot_of` -- which may call
    /// `RootSet::grow_slots` -- without ever growing a non-top frame. The 4a
    /// invariant `grow_slots` asserts is therefore untouched by this task:
    /// it was not overlooked, it is what this ordering preserves.
    ///
    /// **The frame is allocated here and not at the `CALL`**, and that is
    /// measured rather than a matter of taste. Whether a `PROCEDURE` is legal
    /// is a property of how control arrived, not of the body's text: `call
    /// sub` into `sub: procedure` runs, and falling through into the very
    /// same `sub:` label raises 17.1. A precomputed per-body "does this start
    /// with `PROCEDURE`" flag cannot distinguish the two, so there is nothing
    /// for `CALL` to act on -- `Activation::first_instruction_pending` has
    /// the full four-shape table.
    ///
    /// **Exposure is transitive, and the transitivity is in `slot_ref`.**
    /// Measured: `a` exposes `n` to `b`, `b` exposes the same `n` to `c`, `c`
    /// writes it, and `a` sees the write. Resolving `c`'s target through the
    /// frame in force -- which is `b`'s, already carrying `b`'s own alias --
    /// chases to `a` in one step. Binding to `b`'s storage instead would give
    /// a silently wrong value two levels up.
    ///
    /// **One `PROCEDURE` can expose names that live in different frames**, so
    /// the redirect is per slot and not one target frame for the whole
    /// callee. Measured: `c: procedure expose n m` above, where `n` chases to
    /// `a` and `m` stops at `b` because `m` was `b`'s own local -- `b` sees
    /// both of `c`'s writes and `a` sees only `n`'s. An earlier statement of
    /// this design called the redirect "a bitset over slot indices plus one
    /// target `SlotFrame`"; that shape cannot represent this pair, and this
    /// program is what shows it.
    fn exec_procedure(
        &mut self,
        code: &Code<'_>,
        variables: &[VariableRef],
        first_instruction: bool,
    ) -> Result<(), Failure> {
        // 17.1 covers every shape but one: the first instruction executed
        // after an internal `CALL` or function invocation. Both halves are
        // needed -- top level fails the second, and anything after another
        // instruction fails the first.
        if !(first_instruction && self.activation().entered_by_call) {
            return Err(Raised::procedure_out_of_place().into());
        }

        let names = self.expose_names(code, variables)?;

        // Resolved against the pool still in force, which is the caller's:
        // this activation has not swapped in a frame of its own yet.
        let outer = self.activation().frame;
        let mut bindings: Vec<(Box<[u8]>, usize, SlotRef)> = Vec::with_capacity(names.len());
        for name in names {
            // Whole stems alias fine -- the stem object lives in one slot,
            // so aliasing that slot shares the object and every measured
            // stem transcript falls out of it. A single tail does not; see
            // `Loud::compound_expose`.
            if shape_of(&name) == NameShape::Compound {
                return Err(Loud::compound_expose(&name).into());
            }
            let slot = self.slot_of(&name);
            let target = self.roots.slot_ref(outer, slot);
            bindings.push((name, slot, target));
        }

        // Any name that needed a fresh slot just grew the caller's frame and
        // was recorded in *this* activation's `extra` -- which is a clone of
        // the caller's, taken at the call. The caller has to learn about it,
        // because after the isolation below this map is replaced and the
        // return path deliberately does not write it back.
        //
        // Measured, and it does not fall out of anything else: a caller with
        // `nm = 'ZQXW'`, a callee `procedure expose (nm)` doing `interpret
        // "zqxw = 'set-in-callee'"`, and the caller then reading `zqxw`
        // through its own `interpret` prints `set-in-callee`. `ZQXW` appears
        // in no instruction of either, so the plan has no slot for it and
        // both sides reach it only through a run-time binding.
        let resolved = self.activation().extra.clone();
        if let Some(caller) = self
            .activations
            .len()
            .checked_sub(2)
            .and_then(|index| self.activations.get_mut(index))
        {
            caller.extra = resolved;
        }

        // Sized from the caller's *current* frame length rather than from
        // `plan.len()`: an exposed name may sit at an index the caller grew
        // into, and that same index has to address something on this side of
        // the alias too.
        let len = self.roots.frame_len(outer);
        let inner = self.roots.push_slots(len);
        for (_, slot, target) in &bindings {
            self.roots.alias_slot(inner, *slot, *target);
        }

        // The callee's own run-time bindings start empty -- that is the
        // isolation -- except for exposed names the plan never saw, which
        // must keep resolving to the index the alias was installed at.
        let plan = Rc::clone(&self.activation().plan);
        let mut extra = HashMap::new();
        for (name, slot, _) in bindings {
            if plan.slot_of(&name).is_none() {
                extra.insert(name, slot);
            }
        }

        let activation = self.activation_mut();
        activation.frame = inner;
        activation.owns_frame = true;
        activation.extra = extra;
        Ok(())
    }

    /// Every name one `PROCEDURE EXPOSE` list names, in source order.
    ///
    /// **The indirect form is plural and also exposes its own selector.**
    /// Measured twice: with `list = 'ALPHA BETA'`, `procedure expose (list)`
    /// exposes `ALPHA` and `BETA` and nothing else; and with `v = 'zzz'`,
    /// `procedure expose (v)` exposes `v` *itself* as well as `ZZZ` -- the
    /// callee reads `v` as `zzz` (the caller's value) and a write to either
    /// name in the callee is visible in the caller. So the selector's own
    /// name goes on the list beside the words its value spells.
    ///
    /// The value is split and validated exactly the way `DROP (v)`'s own arm
    /// does it, through the same two functions, and for the same measured
    /// reason: a word is upcased only after it validates, one word at a time,
    /// never as a whole. Validation runs over the entire list before any of
    /// it is used, so a bad word later in the list cannot leave half a
    /// `PROCEDURE` performed.
    fn expose_names(
        &mut self,
        code: &Code<'_>,
        variables: &[VariableRef],
    ) -> Result<Vec<Box<[u8]>>, Failure> {
        let mut names: Vec<Box<[u8]>> = Vec::new();
        for variable in variables {
            match variable {
                VariableRef::Direct(id) => names.push(code.symbols.name(*id).as_bytes().into()),
                VariableRef::Indirect(id) => {
                    // The selector itself, then the names its value spells.
                    names.push(code.symbols.name(*id).as_bytes().into());
                    let (value, _novalue) = self.read(code, *id);
                    let text = self.to_text(value).into_owned();
                    for word in split_indirect_words(&text) {
                        names.push(validate_indirect_word(word)?.into());
                    }
                }
            }
        }
        Ok(names)
    }

    /// `USE ARG`, `USE STRICT ARG` and `USE LOCAL`.
    ///
    /// `USE LOCAL` is never legal here -- this crate has no method
    /// invocations at all -- so implementing it means implementing which of
    /// its two refusals applies. Measured on the oracle: as a program's own
    /// first instruction it is 98.993 ("may only be used from method
    /// invocations"); as a program's second instruction, as a called
    /// routine's first instruction, and after a `PROCEDURE`, it is 99.910
    /// ("must be the first instruction executed after a method invocation").
    ///
    /// **Only the 98.993 shape reaches this function, and the 99.910 arm is
    /// unreached today.** `rexx-parse` already enforces the placement rule
    /// at parse time (`instruction.rs`'s own `use_local`, error 99.910, and
    /// 99.915 for a fragment), so every shape that would take the second arm
    /// fails before execution begins. Eight were tried and all were
    /// intercepted: second instruction of a program, after a label on its own
    /// line, after a label on the same line, after a `PROCEDURE`, inside a
    /// `DO` block, inside an `IF`, and inside an `INTERPRET` in two
    /// positions. Those cases already answer the oracle's own number; what
    /// they do not answer byte for byte is the clause echo, which is the
    /// standing parse-error limitation `execute` documents (`lib.rs`) and is
    /// unchanged by this task -- the baseline binary emits the identical
    /// bytes for them.
    ///
    /// The arm is kept rather than collapsed, on the same reasoning
    /// `Loud::missing_body` states for its own unreached arm: a rule the
    /// parser happens to enforce first is not a guarantee this function can
    /// rely on, and answering 98.993 unconditionally would be a silent wrong
    /// answer the day that check moves. It carries no test, because a test
    /// for it would necessarily pass through the parse-time path instead and
    /// so could not fail if this arm were wrong.
    ///
    /// The shape that would separate "is a method invocation" from "was not
    /// entered by a call" cannot be written in this phase either: no method
    /// invocation exists to write it with.
    fn exec_use(
        &mut self,
        code: &Code<'_>,
        use_: &Use,
        first_instruction: bool,
    ) -> Result<(), Failure> {
        match use_ {
            Use::Local { .. } => {
                if first_instruction && !self.activation().entered_by_call {
                    Err(Raised::use_local_outside_method().into())
                } else {
                    Err(Raised::use_local_not_first().into())
                }
            }
            Use::Arg {
                strict,
                allow_optionals,
                targets,
            } => self.exec_use_arg(code, *strict, *allow_optionals, targets),
        }
    }

    /// `USE ARG`/`USE STRICT ARG`: bind the call's arguments to this
    /// instruction's targets, positionally.
    ///
    /// Every rule below is measured, in a clean directory:
    ///
    /// * Extra arguments are ignored without `STRICT`. `call sub 1,2,3` into
    ///   `use arg p` binds `p = 1`.
    /// * An **absent** target is *dropped*, not left alone. `r = 'preset'`
    ///   before a no-`PROCEDURE` `call sub 1` into `use arg p, r` makes both
    ///   the callee and the caller read `r` as `R`. A probe using a target
    ///   whose prior value equalled its own derived name could not have seen
    ///   this.
    /// * An omitted position (`call sub 1,,3`) holds its place: `use arg p,
    ///   q, r` gives `[1] [Q] [3]`.
    /// * A default fills an absent *or* omitted position: `call sub 1,,3`
    ///   into `use arg p, q = 'dflt', r` gives `[1] [dflt] [3]`.
    /// * `STRICT` adds two arity checks, and a default satisfies the
    ///   minimum: `use strict arg p, q` with one argument is 40.3, while
    ///   `use strict arg p, q = 'dflt'` with one argument runs.
    /// * A trailing `...` suppresses the maximum check only. `use strict arg
    ///   p, q, ...` takes four arguments; `use strict arg p` takes one.
    fn exec_use_arg(
        &mut self,
        code: &Code<'_>,
        strict: bool,
        allow_optionals: bool,
        targets: &[Option<UseTarget>],
    ) -> Result<(), Failure> {
        if strict {
            let supplied = self.call_context.arguments.len();
            // The minimum is the position of the last target that must be
            // supplied -- one with no default of its own. A later target
            // carrying a default does not raise it, which is what makes `use
            // strict arg p, q = 'dflt'` legal with one argument.
            let minimum = targets
                .iter()
                .rposition(|target| {
                    target
                        .as_ref()
                        .is_none_or(|target| target.default.is_none())
                })
                .map_or(0, |index| index + 1);
            if supplied < minimum {
                let name = self.call_context.name.clone();
                return Err(Raised::not_enough_arguments(&name, minimum).into());
            }
            if !allow_optionals && supplied > targets.len() {
                let name = self.call_context.name.clone();
                return Err(Raised::too_many_arguments(&name, targets.len()).into());
            }
        }

        for (index, target) in targets.iter().enumerate() {
            let Some(target) = target else { continue };
            // `get` past the end and a `None` inside the list are the same
            // thing to a target: nothing was supplied for this position.
            let argument = self.call_context.arguments.get(index).cloned().flatten();
            self.bind_use_target(code, index, target, argument)?;
        }
        Ok(())
    }

    /// Binds one `USE ARG` target to one argument, or to its default, or to
    /// nothing.
    ///
    /// The `alias` case is the whole reason `Argument` is not a bare
    /// `ObjRef`: `>name` needs the *caller's* slot, and only an argument
    /// written `>something` at the call carries one. It has **three**
    /// separate measured refusals -- a supplied argument that is not a
    /// reference is 88.928, an omitted position is 88.931, and a target that
    /// is not currently unset is 98.995 ([`target_is_uninitialised`]).
    ///
    /// [`target_is_uninitialised`]: Interp::target_is_uninitialised
    fn bind_use_target(
        &mut self,
        code: &Code<'_>,
        index: usize,
        target: &UseTarget,
        argument: Option<Argument>,
    ) -> Result<(), Failure> {
        let position = index + 1;
        if target.alias {
            let Some(argument) = argument else {
                return Err(Raised::variable_reference_omitted(position).into());
            };
            let Argument::Reference {
                target: slot,
                name: reference,
                ..
            } = argument
            else {
                let found = self.to_text(argument.value()).to_vec();
                return Err(Raised::not_a_variable_reference(position, &found).into());
            };
            let name = self.use_target_name(code, target)?;
            // **The kinds must match, and the check is before the
            // uninitialised one.** Measured: a target that is both
            // kind-mismatched and already assigned reports the kind error,
            // not 98.995. Compound is not a third kind to handle -- `>p.1`
            // and `>q.1` are both rejected by `rexx-parse` (20.930/20.931),
            // so each side is a simple variable or a stem and nothing else.
            let target_is_stem = shape_of(&name) == NameShape::Stem;
            let reference_is_stem = shape_of(&reference) == NameShape::Stem;
            if target_is_stem != reference_is_stem {
                // Both substitute the *caller's* name, unlike 98.995 just
                // below, which names the target. Measured with a variable
                // whose value differs from its name, so the two cannot be
                // confused: `p = 'value-not-name'` passed as `>p` reports
                // `found "P"`.
                return Err(if target_is_stem {
                    Raised::not_a_stem_variable_reference(position, &reference).into()
                } else {
                    Raised::not_a_simple_variable_reference(position, &reference).into()
                });
            }
            let index = self.slot_of(&name);
            let frame = self.activation().frame;
            // The target must be **currently unset**. `RootSet::slot`
            // resolves through any alias already in force, which is what the
            // repeat case needs: after one `use arg >q`, `Q` reads the
            // caller's variable, so it "has a value" and the second attempt
            // is refused.
            if !self.target_is_uninitialised(&name, frame, index) {
                return Err(Raised::variable_reference_not_uninitialised(&name).into());
            }
            self.roots.alias_slot(frame, index, slot);
            // `>R>`, the alias's own line and the **only** trace line this
            // branch emits: no `>>>` and no `>=>`, because nothing was
            // evaluated and nothing was assigned (`UseInstruction.cpp:164`-
            // `167`, `aliasVariable` then `traceVariableAlias`, and
            // `handleArgument` `return`s before its own `traceResult` for
            // this case). Caller's name first, target's second -- see
            // `trace_alias`.
            self.trace_alias(self.clause_state.current_value_indent, &reference, &name);
            return Ok(());
        }

        // Present: bind the value. Absent: the default if there is one, and
        // otherwise drop the target -- measured, an absent target does not
        // keep whatever it held before.
        let value = match argument {
            Some(argument) => Some(argument.value()),
            None => match &target.default {
                Some(default) => {
                    let value = self.eval(code, default)?;
                    self.roots.push_temp(value);
                    Some(value)
                }
                None => None,
            },
        };
        let name = self.use_target_name(code, target)?;
        match value {
            Some(value) => {
                // `>>>` then `>=>`, in that order and both at this `USE`
                // clause's own indent -- `handleArgument`'s own
                // `traceResult(argument)` immediately before
                // `retriever->assign(context, argument)`, whose own
                // `traceAssignment` is the second line
                // (`UseInstruction.cpp:74`-`77`, and the default-value arm
                // ten lines below it does the identical pair). Measured
                // under `trace r`: `use arg a, b` on a two-argument call
                // traces `>>>     "1"` and `>>>     "2"` and no `>=>`, which
                // is the gating -- `>>>` is `results`, `>=>` is
                // `intermediates`, so the pair is not one line's worth of
                // conditional.
                //
                // **A dropped target traces neither**, which is the adjacent
                // measured case rather than an omission here: `call sub 1,,3`
                // into `use arg p, q, r` traces `>>>`/`>=>` for `P` and `R`
                // and nothing at all for `Q` (`variable->drop(context)` has
                // no trace call of its own).
                let indent = self.clause_state.current_value_indent;
                let rendered = self.to_text(value).to_vec();
                self.trace_result(indent, &rendered);
                self.assign_by_name(&name, value);
                self.trace_assignment(indent, &name, &rendered);
            }
            None => self.drop_by_name(&name),
        }
        Ok(())
    }

    /// Whether a `USE ARG >name` target is in the uninitialised state the
    /// oracle requires of it.
    ///
    /// **The trigger is only "does this variable have a value". It is not
    /// about exposure and not about locality**, despite error 98.995's own
    /// wording ("it must be an uninitialized local variable"). The pair that
    /// separates those hypotheses is measured, and without it the check would
    /// very plausibly have been written as an exposure test and been wrong:
    ///
    /// * `procedure expose q` where the exposed `q` **holds a value** -> rc
    ///   158, 98.995.
    /// * `procedure expose q` where the exposed `q` is **unset** -> rc 0, the
    ///   alias is installed, the caller prints `q: Q`.
    ///
    /// Exposure is identical in both; only the value differs. `DROP` restores
    /// the uninitialised state, so `q = 'local'; drop q; use arg >q` succeeds.
    /// Repeating `use arg >q` onto one target fits the same rule rather than
    /// being a case of its own: the first alias makes `Q` read the caller's
    /// variable, so it has a value by the second.
    ///
    /// **The stem exemption is this crate's own shape showing through, and it
    /// is measured on both sides.** `read_stem` vivifies a fresh, empty
    /// `Body::Stem` into the slot on a bare stem read (it must -- a stem's
    /// object identity is observable through `b. = a.`), and `stem_drop`
    /// leaves exactly the same thing. Neither is an initialised variable to
    /// the language, and the oracle agrees: `say q.` then `use arg >q.`
    /// succeeds, and so does `q.1 = 'x'; drop q.; use arg >q.`, while
    /// `q.1 = 'local'` and `q. = 'dflt'` both raise. `is_uninitialised_stem`
    /// has the full nine-row table, including the three rows that make the
    /// test "no default and no tail that still has a value" rather than the
    /// tempting "no default and no tails".
    ///
    /// **Keyed on the target's own name shape, not on the value's**, which is
    /// the distinction a "value is an empty stem" test would get wrong.
    /// Measured: `zz = q.` puts a fresh, empty stem object into a *simple*
    /// variable, and `use arg >zz` then raises 98.995. `ZZ` is an initialised
    /// simple variable that happens to hold a stem; `Q.` is a stem variable
    /// nobody has written.
    fn target_is_uninitialised(&self, name: &[u8], frame: SlotFrame, index: usize) -> bool {
        match self.roots.slot(frame, index) {
            None => true,
            Some(value) => shape_of(name) == NameShape::Stem && self.is_uninitialised_stem(value),
        }
    }

    /// One `USE ARG` target's variable name.
    ///
    /// A target is `parseVariableOrMessageTerm`, so the grammar admits a
    /// message term here as well as a variable (`UseTarget::target`'s own
    /// doc). Only the variable spellings are implemented, and a message term
    /// fails loudly through the same `Loud::expression` path every other
    /// unimplemented expression form uses rather than being approximated.
    fn use_target_name(&mut self, code: &Code<'_>, target: &UseTarget) -> Result<Vec<u8>, Failure> {
        match &target.target.kind {
            ExprKind::Variable(id) | ExprKind::Stem(id) | ExprKind::Compound(id) => {
                Ok(code.symbols.name(*id).as_bytes().to_vec())
            }
            other => Err(Loud::expression(other).into()),
        }
    }

    /// Assigns `value` to the variable, whole stem, or one verbatim-keyed
    /// tail that `name`'s own spelling names.
    ///
    /// `drop_by_name`'s counterpart, dispatched by the same `shape_of` and
    /// through the same stem entry points, so that `USE ARG` binding a stem
    /// target does what an ordinary `stem. = value` assignment does --
    /// measured, `call sub2 'val'` into `use arg st.` makes `st.` render as
    /// `val`.
    fn assign_by_name(&mut self, name: &[u8], value: ObjRef) {
        match shape_of(name) {
            NameShape::Simple => {
                let slot = self.slot_of(name);
                let frame = self.activation().frame;
                self.roots.set_slot(frame, slot, value);
            }
            NameShape::Stem => self.stem_assign(name, value),
            NameShape::Compound => {
                let dot = name
                    .iter()
                    .position(|&b| b == b'.')
                    .expect("NameShape::Compound guarantees at least one period");
                let (stem_name, key) = name.split_at(dot + 1);
                self.stem_set(stem_name, key, value);
            }
        }
    }

    /// `SIGL`, set at the point of every control transfer -- `SIGNAL`'s own
    /// two `step` arms and `resolve_and_run_call` (`CALL`, and `ExprKind::
    /// Call` through `eval_call`, `eval.rs`) -- to `line`, always `self.
    /// current_clause_line` at the call site (`lib.rs`'s own doc comment on
    /// that field has why it is a field and not a parameter here).
    ///
    /// **A plain string, not a `Number`.** The oracle's own `RexxActivation::
    /// signalTo`/`internalCall` (read directly, `execution/RexxActivation.
    /// cpp`) both call `new_integer(lineNum)`, an integer object that always
    /// renders in full decimal, never in exponential form -- measured here
    /// too: `numeric digits 1` in force does not turn a two-digit `SIGL`
    /// value into `2E+1` the way the identical magnitude would if it reached
    /// the program as an arithmetic result. `self.text` gives that directly,
    /// with no `created_digits` to reason about at all, matching how this
    /// crate already renders an ordinary literal.
    ///
    /// Through `assign_by_name`, so `SIGL` gets exactly the pool-sharing
    /// behaviour every other variable does: shared with the caller's frame
    /// by default (measured, a callee with no `PROCEDURE` sees the value the
    /// `CALL`/`SIGNAL` that reached it just set), isolated and starting
    /// uninitialised once `PROCEDURE` allocates a frame of its own (measured,
    /// `SIGL` reads back as the derived name `SIGL` inside a `PROCEDURE`d
    /// callee that has not yet transferred control itself), and never
    /// restored on the way out (measured, an inner `CALL`'s own `SIGL`
    /// outlives that call's own return, all the way up to the main body,
    /// exactly like any other shared-pool variable).
    fn set_sigl(&mut self, line: usize) {
        let value = self.text(line.to_string().as_bytes());
        self.assign_by_name(b"SIGL", value);
    }

    /// `SIGNAL ON`/`OFF` and `CALL ON`/`OFF`, which are one instruction with
    /// one flag between them.
    ///
    /// **Neither form transfers control here**, which is the whole reason
    /// this arm is three lines: `ON` records a trap in the running
    /// activation's table and `OFF` removes one, and the transfer -- or the
    /// call -- happens later, in `run_activation`, if the condition is ever
    /// raised.
    ///
    /// `label` rather than `on` is what tells the two apart, following
    /// `ConditionTrap`'s own doc comment (`rexx-parse`): the parser has
    /// already defaulted an `ON` with no `NAME` clause to the condition's own
    /// name (`USER foo`'s default label is `FOO`, not `USER FOO`, measured),
    /// and leaves `None` for `OFF` alone. Reading `on` as well would be two
    /// sources for one fact.
    ///
    /// **`ON` over an already-enabled trap replaces it rather than being an
    /// error**, and that is load-bearing rather than incidental: a trap is
    /// removed when it fires, and re-arming inside the handler is how a
    /// program traps the same condition twice. Measured -- a `SIGNAL ON
    /// SYNTAX` handler that runs `signal on syntax name second` and then
    /// divides by zero reaches `second`, where the same handler without the
    /// re-arm gets the ordinary fatal report.
    fn exec_condition_trap(&mut self, trap: &ConditionTrap, call: bool) -> Result<Flow, Failure> {
        match &trap.label {
            Some(label) => {
                let entry = Trap {
                    call,
                    label: label.clone(),
                };
                self.activation_mut()
                    .traps
                    .insert(trap.condition.clone(), entry);
            }
            None => {
                self.activation_mut().traps.remove(&trap.condition);
            }
        }
        Ok(Flow::Next)
    }

    /// The trap the running activation has enabled for `condition`, if any.
    ///
    /// **`ANY` is a fallback key, consulted only when the condition's own
    /// name is not in the table.** Measured: `signal on any` traps a plain
    /// `say 1/0`, with `SIGL` set exactly as a `signal on syntax` would set
    /// it. The parser already accepts `ANY` for both `CALL ON` and `SIGNAL
    /// ON` (`condition_trap`'s own comment records that measurement), so
    /// without this lookup an `ANY` trap would be recorded and never fire.
    ///
    /// Returns a clone rather than a borrow: every caller goes on to call a
    /// `&mut self` method in the same breath (`remove` the trap, then
    /// `set_sigl`), which a borrow of `self.activations` held across would
    /// make the `E0502` `run_activation`'s own doc comment writes out.
    pub(crate) fn trap_for(&self, condition: &[u8]) -> Option<Trap> {
        let traps = &self.activation().traps;
        traps
            .get(condition)
            .or_else(|| traps.get(b"ANY".as_slice()))
            .cloned()
    }

    /// Turns an uninitialised variable read into a `NOVALUE` condition --
    /// but only when this activation has a `NOVALUE` trap that could take
    /// it.
    ///
    /// **Inherited item I13, and `Novalue::Unset`'s first reader.** D16 put
    /// the flag on the read path from the start rather than have 4b retrofit
    /// a raise into it, and this is the retrofit that did not have to happen.
    ///
    /// **Gated on the trap rather than raised unconditionally**, for two
    /// reasons that both matter. An untrapped `NOVALUE` has no effect
    /// whatever -- the read yields the derived name, measured, and that is
    /// what every program in the corpus already depends on -- so raising and
    /// then discarding would build a condition per uninitialised read on the
    /// hottest path there is. And a `Raised::condition` carries no catalogue
    /// entry, so one escaping untrapped would report `Error 0`; the gate is
    /// what makes that unreachable rather than merely unlikely.
    ///
    /// The gate is the same test `offer_to_trap` will apply a moment later
    /// -- same activation, same table, and a `call` trap excluded from both
    /// -- so a condition raised here always finds the trap that let it be
    /// raised. `CALL ON NOVALUE` is a parse error anyway; the `call` half is
    /// reachable only through `CALL ON ANY`, which is measured not to catch
    /// a condition that has no resumption point.
    pub(crate) fn novalue_check(&mut self, novalue: Novalue) -> Result<(), Failure> {
        if novalue == Novalue::Set {
            return Ok(());
        }
        if self.trap_for(b"NOVALUE").is_none_or(|trap| trap.call) {
            return Ok(());
        }
        Err(Raised::condition(Cow::Borrowed("NOVALUE")).into())
    }

    /// Offers a failure escaping the running activation to that activation's
    /// trap table, and either transfers control or hands the failure back to
    /// keep unwinding.
    ///
    /// Called from `run_activation`'s own loop, once per instruction, on the
    /// `Err` path alone -- so the *innermost* activation gets first refusal
    /// and each enclosing one gets its turn as the failure propagates,
    /// which is the outward walk [`Search::Here`] describes. `resolve_and_
    /// run_call` has already restored the caller's `clause_state` by the time
    /// the caller's own loop sees the failure, so `SIGL` below reads the
    /// trapping activation's own clause line rather than the callee's --
    /// measured, and the two really do differ: the same `say 1/0` reports
    /// `SIGL` 9 when the callee traps it and 3 when the callee's trap is off
    /// and the caller's fires instead.
    ///
    /// **Only a `SIGNAL ON` trap ever takes a failure here.** A `CALL ON`
    /// trap resumes execution, and there is nothing to resume into once a
    /// clause has failed -- measured rather than assumed, and measurable
    /// only because `CALL ON ANY` is legal where `CALL ON SYNTAX` is a parse
    /// error: `call on any name uh` with `say 1/0` is **not** trapped, it is
    /// the ordinary fatal 42.3 at rc 214. So a `call` trap declines here and
    /// the failure keeps unwinding. Every condition a `CALL ON` trap really
    /// does catch reaches it through `Interp::pending_trap` instead, without
    /// ever becoming a failure.
    ///
    /// Returns a `Flow` rather than a bare target so that
    /// `run_activation`'s existing `match` does the transfer: `Flow::Signal`
    /// is exactly "set this activation's `pc`", which is what a trap does.
    fn offer_to_trap(&mut self, code: &Code<'_>, failure: Failure) -> Result<Flow, Failure> {
        let Failure::Raised(raised) = &failure else {
            return Err(failure);
        };
        match raised.delivery.search {
            Search::Here => {}
            // One level up, and this is that level's turn to decline. The
            // rewrite is what makes the *next* loop out offer it: without
            // it, `Caller` would skip every activation rather than one.
            Search::Caller => {
                let Failure::Raised(mut raised) = failure else {
                    unreachable!("matched Failure::Raised immediately above")
                };
                raised.delivery.search = Search::Here;
                return Err(raised.into());
            }
            // The outermost activation is the only one allowed to look.
            Search::Top if self.activations.len() > 1 => return Err(failure),
            Search::Top => {}
            Search::Nobody => return Err(failure),
        }
        let Some(trap) = self.trap_for(raised.condition.as_bytes()) else {
            return Err(failure);
        };
        if trap.call {
            return Err(failure);
        }
        // Resolved **before** anything is cleared or removed, because a
        // label that does not exist is not a trap that fired: measured,
        // `signal on syntax name nosuchlabel` with `say 1/0` on line 3
        // reports `Error 16.1 Label "NOSUCHLABEL" not found` against line 3
        // -- the raising clause's own site, which is still the one
        // `step_in_temps_frame` recorded a moment ago and which the clearing
        // below would have thrown away.
        let target = self.resolve_signal_target(&trap.label)?;
        let Failure::Raised(raised) = failure else {
            unreachable!("matched Failure::Raised immediately above")
        };
        // Removed when it fires (`Activation::traps`' own doc comment has
        // the two probes). Removed by the condition's *own* name and by
        // `ANY`, since `trap_for` may have matched either and leaving the
        // one that matched enabled would re-trap.
        let traps = &mut self.activation_mut().traps;
        traps.remove(raised.condition.as_bytes());
        traps.remove(b"ANY".as_slice());
        // **Inherited item I11, and the reason it is this task's.** Both
        // halves of the echo stack are dropped: a trapped condition prints
        // no report at all, so the sites it accumulated must not survive to
        // be printed against a *later*, untrapped one. Measured -- `say 1/0`
        // trapped on line 3 and `say 2/0` untrapped on line 8 inside the
        // handler reports line 8, alone, and a version that kept the first
        // site reports line 3.
        //
        // Dropped from the *interpreter* but kept on the condition, because
        // `RAISE PROPAGATE` re-raises this condition with its original
        // clause echoed rather than the `raise propagate` clause --
        // `ActiveCondition`'s own doc comment (`lib.rs`) has that transcript.
        let site = self.failure_site.take();
        let sites = std::mem::take(&mut self.failure_sites);
        self.set_sigl(self.clause_state.line());
        if let Some(rc) = &raised.rc {
            let value = self.text(rc);
            self.assign_by_name(b"RC", value);
        }
        // What a later `RAISE PROPAGATE` re-raises. See `exec_raise_
        // propagate` for what is and is not measured about it.
        self.active_condition = Some(ActiveCondition {
            raised,
            site,
            sites,
        });
        // **The failed clause's own boundary, and the one place it can
        // happen** (fix round 3). `step_in_temps_frame` ends a *completing*
        // clause; a clause that raised has not completed, and at that moment
        // nothing yet knows whether it will be trapped here or unwind the
        // activation. This is the point where that is decided in favour of
        // "trapped here, execution continues", so it is the point where a
        // `CALL ON` handler queued by the same clause is owed its run.
        //
        // Measured, and the pair is what places it here rather than in
        // `step_in_temps_frame`'s `Err` arm: `zq = sub() + 1/0` with both a
        // `CALL ON USER` and a `SIGNAL ON SYNTAX` trap prints `UH ran` then
        // `SH ran` -- so the queued handler runs even though its clause
        // failed -- while the same clause in a routine whose own `SIGNAL OFF`
        // sends the failure out of the activation entirely delivers nothing
        // at all. One completes here; the other never does.
        if let Some(exit) = self.deliver_pending_trap(code)? {
            return Ok(Flow::Exit(exit.value()));
        }
        Ok(Flow::Signal(target))
    }

    /// Runs a `CALL ON` trap's handler, at the clause boundary the condition
    /// has been waiting for.
    ///
    /// **The wait is the measured part.** `zres = one(1)`, where `one`
    /// raises a `CALL ON`-trapped condition and the handler assigns `zres`
    /// itself, prints the *handler's* value -- so the assignment had already
    /// stored the routine's result before the handler ran. `say 'a' one(1)
    /// two(2)` in the same shape prints the whole line, `two`'s value
    /// included, and only then the handler. Neither is what a trap that
    /// fired at the raise would print.
    ///
    /// The trap is removed for the handler's duration and **put back
    /// afterwards**, unlike a `SIGNAL ON` trap, which stays removed.
    /// Measured: a handler that itself calls a routine raising the same
    /// condition does not re-enter, and the program then carries on
    /// normally rather than running the handler a second time later.
    pub(crate) fn deliver_pending_trap(
        &mut self,
        code: &Code<'_>,
    ) -> Result<Option<HandlerExit>, Failure> {
        // Only the activation whose trap table matched delivers, and only
        // once it is running again -- `PendingTrap::activation`'s own doc
        // comment has the three transcripts this identity check answers,
        // including the two a stack depth got wrong.
        if self.pending_trap.as_ref().map(|pending| pending.activation)
            != Some(self.activation().id)
        {
            return Ok(None);
        }
        let Some(pending) = self.pending_trap.take() else {
            return Ok(None);
        };
        let Some(trap) = self.trap_for(&pending.condition) else {
            return Ok(None);
        };
        if !trap.call {
            // A `SIGNAL ON` trap never gets here: `exec_raise` throws for
            // that half instead, so the transfer happens where the raise is
            // rather than one clause later. Declining rather than asserting
            // keeps a future raiser that forgets the distinction from
            // silently running a `SIGNAL` handler as a call.
            return Ok(None);
        }
        self.set_sigl(self.clause_state.line());
        if let Some(rc) = &pending.rc {
            let value = self.text(rc);
            self.assign_by_name(b"RC", value);
        }
        // A `CALL ON` handler is running a condition too, and `RAISE
        // PROPAGATE` inside one asks for it -- measured, `raise propagate`
        // in a `CALL ON USER FOO` handler ends the program silently at rc 0,
        // where the same clause with no handler running at all is 98.918.
        // Recording nothing here would give the second answer for the first
        // program. No sites travel with it: nothing failed, so nothing was
        // cleared.
        let mut raised = Raised::condition(condition_name(&pending.condition));
        raised.rc = pending.rc.clone();
        // **Saved and restored, not cleared** (fix round 2's NEW 1). Round 1
        // set this back to `None` when the handler returned, which is right
        // only when nothing was active before -- and one clause can queue a
        // `CALL ON` condition *and* raise a `SIGNAL ON`-trapped one, so a
        // `SIGNAL` handler can be running when a `CALL` handler is delivered
        // inside it. Measured: `zq = sub() + 1/0` under both traps, with the
        // `SIGNAL` handler ending in `raise propagate`, is the original 42.3
        // at rc 214 on the oracle; clearing to `None` gave 98.918 at rc 158,
        // and never clearing at all gave silence at rc 0. Restoring gives the
        // oracle's answer in all three measured shapes, the "nothing was
        // active, restore `None`" one included.
        let enclosing = self.active_condition.take();
        self.active_condition = Some(ActiveCondition {
            raised,
            site: None,
            sites: Vec::new(),
        });
        let key: Box<[u8]> = pending.condition.clone();
        let removed = self.activation_mut().traps.remove(&key);
        let ended = self.resolve_and_run_call(code, &trap.label, true, &[]);
        if let Some(trap) = removed {
            self.activation_mut().traps.insert(key, trap);
        }
        match ended {
            // The handler returned; execution resumes at the clause after
            // the one that finished.
            //
            // **And the enclosing condition comes back here** (fix round 1,
            // corrected by round 2). A `RAISE PROPAGATE` after this point
            // must see whatever was active *before* this handler ran, which
            // is `None` in the common case -- measured, `call sub` (trapped,
            // handler returns) followed by `raise propagate` is `98.918` at
            // rc 158, where leaving this handler's own condition in place
            // gave silence at rc 0 -- and is a real condition when a `SIGNAL`
            // handler is running around it. See the `take` above.
            //
            // **Only on this arm, which is the measured half.** A `SIGNAL ON`
            // handler that runs on -- `SIGNAL`s to another label and then
            // propagates -- must still find the original condition, also
            // measured (both interpreters re-raise the original 42.3 at rc
            // 214). So the restore belongs to the point a *call* handler
            // returns, not to handlers in general, and `offer_to_trap`
            // deliberately has no equivalent.
            Ok(Ended::Returned(_)) => {
                self.active_condition = enclosing;
                Ok(None)
            }
            // The handler failed rather than returned. **Reachable but
            // unobservable, kept deliberately** (fix round 3). The
            // re-review's panic probe found four programs that take this
            // arm and no test that does, and established why nothing can see
            // it: every path that goes on to read `active_condition` passes
            // through `offer_to_trap` first, which overwrites the field
            // wholesale. So this line changes no output today.
            //
            // Kept rather than deleted because the alternative is not
            // "nothing" but "a wrong value that happens not to be read":
            // leaving this handler's own condition in place is exactly the
            // state the arm above exists to prevent, and it would become
            // observable the day a read reaches it without passing through
            // `offer_to_trap`. One line to be right by construction is
            // cheaper than a comment explaining why being wrong is safe.
            Err(failure) => {
                self.active_condition = enclosing;
                Err(failure)
            }
            // `EXIT` inside the handler ends the program, exactly as it does
            // inside any other called routine. Nothing will read
            // `active_condition` again, so it is left as it is.
            //
            // **This match is the whole announcement of "a delivered handler
            // only ever ends the program by `EXIT`"** (fix round 4). It used
            // to be an `unreachable!` in `run_bounded` (round 2), then six
            // copies of `Ok(Flow::Exit(ended.value()))` at the call sites
            // (round 3) -- and `Ended::value()` collapses `Returned` and
            // `Exited`, so those six would have turned a `RETURN` into an
            // `EXIT` in silence. `HandlerExit` can only be built here, from
            // this arm, so the arm above is the only thing that decides it.
            Ok(exited @ Ended::Exited(_)) => Ok(HandlerExit::from_ended(exited)),
        }
    }

    /// The trap the running activation's **caller** has enabled, or `None`
    /// at top level.
    ///
    /// `exec_raise`'s own lookup for the non-`SYNTAX` conditions, whose
    /// search starts one level out ([`Search::Caller`]). Separate from
    /// `trap_for` rather than parameterised by depth because these are the
    /// only two depths anything asks about, and a depth parameter would read
    /// as though arbitrary ones were meaningful.
    fn caller_trap_for(&self, condition: &[u8]) -> Option<Trap> {
        let caller = self.activations.len().checked_sub(2)?;
        let traps = &self.activations[caller].traps;
        traps
            .get(condition)
            .or_else(|| traps.get(b"ANY".as_slice()))
            .cloned()
    }

    /// `RAISE`, in all of its forms.
    ///
    /// # The delivery table, which is the whole instruction
    ///
    /// Nothing about `RAISE`'s grammar says that its tail decides *who* may
    /// trap it, and that is what it does. Measured, against a three-level
    /// call chain -- a two-level program gives identical bytes for the first
    /// and third rows, which is why the first version of this table was
    /// wrong:
    ///
    /// ```text
    /// RAISE SYNTAX n.m RETURN [e]   search from the raising activation outward
    /// RAISE SYNTAX n.m             \  the OUTERMOST activation's trap only;
    /// RAISE SYNTAX n.m EXIT [e]    /  every level in between skips its own
    /// RAISE other ... RETURN [e]      search from the raising activation's CALLER
    /// RAISE other ...              \  no trap at all -- the program ends, and
    /// RAISE other ... EXIT [e]     /  the condition's default action applies
    /// ```
    ///
    /// The three transcripts that force each row apart, each run twice, once
    /// with the trap enabled in the middle routine and once with it enabled
    /// in the main body as well:
    ///
    /// * `raise syntax 40.4` in `lev2`, `signal on syntax name mid` in
    ///   `lev1`: **not trapped**, rc 216, `mid` never runs. Add `signal on
    ///   syntax name outer` to the main body and `outer` runs, with `SIGL`
    ///   set to the main body's `call lev1` clause -- so it skipped `lev1`
    ///   and landed at the top.
    /// * `say 1/0` in the same place: `mid` runs, with `SIGL` set to
    ///   `lev2`'s own line. The ordinary search is not the `RAISE` one.
    /// * `raise user foo return 'RETVAL'` in `fun` with `signal on user foo`
    ///   in the main body: trapped, `SIGL` the main body's clause. The
    ///   identical program with `raise syntax 40.4 return` reports `SIGL` as
    ///   `fun`'s own `raise` line instead.
    ///
    /// # What the untrapped default action is, per condition
    ///
    /// Measured at top level with no trap enabled: `raise halt` is the fatal
    /// `Error 4.1` at rc 252; `raise error 5`, `raise user foo` and friends
    /// print nothing and exit 0. So `HALT` reports and the rest are silent,
    /// and [`Raised::reportable`] is where that split lives.
    ///
    /// **A `SIGNAL ON HALT` in the same activation does not change that**,
    /// which is the measurement that stops the last two rows above being
    /// `Search::Top`: `signal on halt` immediately above `raise halt` still
    /// gives the fatal report, where `signal on syntax` above `raise syntax
    /// 40.4` traps. Hence [`Search::Nobody`] for one and [`Search::Top`] for
    /// the other.
    ///
    /// # Evaluation order
    ///
    /// `rc`, then `DESCRIPTION`, then `ADDITIONAL`/`ARRAY`, then the
    /// `RETURN`/`EXIT` value -- source order, and every one of them is
    /// evaluated even when its value is then discarded, because an
    /// expression that raises has to raise.
    ///
    /// `DESCRIPTION`'s value is evaluated and dropped: it is observable only
    /// through `condition('D')`, a builtin this crate does not have, and the
    /// untrapped report is measured to be byte-identical with and without it.
    fn exec_raise(&mut self, code: &Code<'_>, raise: &Raise) -> Result<Flow, Failure> {
        if raise.propagate {
            return self.exec_raise_propagate();
        }
        // Each option traces a `>K>` line as it is evaluated, in source
        // order, at this clause's own indent. Measured, all five spellings:
        //
        // ```text
        // raise syntax 40.4 description 'zdesc' additional 'zadd'
        //   >K>   "SYNTAX" => "40.4"
        //   >K>   "DESCRIPTION" => "zdesc"
        //   >K>   "ADDITIONAL" => "zadd"
        // raise syntax 40.4 array ('ZORKROUTINE', 7)
        //   >K>   "SYNTAX" => "40.4"
        //   >K>   "ARRAY" => "an Array"
        // raise user marker description 'zdesc' return 'zret'
        //   >K>   "DESCRIPTION" => "zdesc"
        //   >K>   "RESULT" => "zret"
        // ```
        //
        // Those five are `trace r` transcripts, where `>A>` is invisible
        // (`intermediates` only). `ARRAY`'s own element lines, and the fact
        // that its `>K>` comes *after* them rather than before, are under
        // `trace i` -- see the `raise.array` arm below.
        //
        // The **condition's own name** is the first keyword, and only for
        // the three conditions that take a value after it -- `raise user
        // marker` traces no line for the condition at all, which is why this
        // is keyed on the value's presence rather than written out
        // unconditionally.
        let indent = self.clause_state.current_value_indent;
        let rc_text = match &raise.rc {
            Some(expr) => {
                let value = self.eval(code, expr)?;
                self.roots.push_temp(value);
                let rendered = self.to_text(value).to_vec();
                let keyword = String::from_utf8_lossy(&raise.condition).into_owned();
                self.trace_keyword(indent, &keyword, &rendered);
                Some(rendered)
            }
            None => None,
        };
        if let Some(expr) = &raise.description {
            let value = self.eval(code, expr)?;
            self.roots.push_temp(value);
            let rendered = self.to_text(value).to_vec();
            self.trace_keyword(indent, "DESCRIPTION", &rendered);
        }
        // `ADDITIONAL expr` and `ARRAY (a, b)` produce the identical
        // substitution list -- measured, `additional ('MYROUTINE', 3)` and
        // `array ('MYROUTINE', 3)` give byte-identical reports -- so they
        // share one `Vec` here rather than being kept apart to no end. A
        // single non-array `ADDITIONAL` value is one substitution, also
        // measured: `additional 'JUSTONE'` fills `&1` and leaves `&2` as the
        // literal `&2`.
        let mut additional: Vec<String> = Vec::new();
        if let Some(expr) = &raise.additional {
            let value = self.eval(code, expr)?;
            self.roots.push_temp(value);
            let rendered = self.to_text(value).to_vec();
            self.trace_keyword(indent, "ADDITIONAL", &rendered);
            additional.push(String::from_utf8_lossy(&rendered).into_owned());
        }
        if let Some(items) = &raise.array {
            // **The elements first, then the `>K>` line** -- corrected at
            // Task 9, which owns `>A>` and measured the ordering while
            // adding it. An earlier version of this arm traced the `>K>`
            // first and said so in a comment that claimed "the elements
            // produce no lines of their own"; both halves are false.
            // Measured, `trace i` / `raise syntax 40.4 array('R',,'X')`:
            //
            // ```text
            //   >L>   "R"
            //   >A>   "R"
            //   >A>   "R"
            //   >A>   ""
            //   >L>   "X"
            //   >A>   "X"
            //   >A>   "X"
            //   >K>   "ARRAY" => "an Array"
            // ```
            //
            // **`>A>` twice per supplied element, once for an omitted one**,
            // which is the oracle's own shape rather than a transcription
            // slip here: `RaiseInstruction.cpp:229`-`237` calls
            // `traceArgument(arg)` on both sides of the `put` into the
            // array, and the omitted arm calls it once with the null string.
            // Reproduced as measured rather than "cleaned up" to one line,
            // because criterion 2 is byte-for-byte agreement, not agreement
            // with what the C++ ought to have done.
            for item in items {
                let Some(expr) = item else {
                    // An omitted position (`array (1,,3)`) **holds its
                    // place** in the substitution list rather than closing
                    // up, and substitutes as empty. Measured -- this used to
                    // `continue`, on a stated-as-unmeasured guess, and the
                    // guess was wrong: `raise syntax 40.4 array('R',,'X')`
                    // reports "maximum expected is ." on the oracle (`&2` is
                    // the hole) where closing up reported "maximum expected
                    // is X." here.
                    self.trace_argument(indent, b"");
                    additional.push(String::new());
                    continue;
                };
                let value = self.eval(code, expr)?;
                self.roots.push_temp(value);
                let rendered = self.to_text(value).to_vec();
                self.trace_argument(indent, &rendered);
                self.trace_argument(indent, &rendered);
                additional.push(String::from_utf8_lossy(&rendered).into_owned());
            }
            // **`an Array`, verbatim and regardless of the elements** --
            // it is the Array class's own default string form, which is
            // what the oracle traces here (measured for `array
            // ('ZORKROUTINE', 7)`). This crate has no array object to render,
            // and building one purely to print a constant would be the
            // longer way to the same two words.
            self.trace_keyword(indent, "ARRAY", b"an Array");
        }
        // The `RETURN`/`EXIT` value, and its own `>K>` line -- measured for
        // both tails: `raise user foo return 'ONEVAL'` under `trace r`
        // traces `>K>     "RESULT" => "ONEVAL"`, and the same with `exit`
        // traces the identical line. `RETURN`'s own instruction arm traces
        // `>>>` instead, so this is not that path with a different indent.
        let result = match &raise.result {
            Some(result) => match &result.value {
                Some(expr) => {
                    let value = self.eval(code, expr)?;
                    self.roots.push_temp(value);
                    let rendered = self.to_text(value).to_vec();
                    self.trace_keyword(indent, "RESULT", &rendered);
                    Some(value)
                }
                None => None,
            },
            None => None,
        };
        let returns = raise.result.as_ref().is_some_and(|result| !result.exit);

        if raise.condition.as_ref() == b"SYNTAX" {
            let mut raised = raise_syntax_condition(rc_text.as_deref().unwrap_or(b""), additional);
            // **The delivery rule follows the tail even when the argument was
            // rejected**, which is measured rather than convenient: `raise
            // syntax 40.10` inside a routine, with the trap in the main body
            // and none in between, reports 98.941 fatally exactly as a
            // well-formed tail-less `RAISE SYNTAX` reports its own number
            // there. The substituted condition is still a `SYNTAX` condition
            // and travels like one.
            raised.delivery.search = if returns { Search::Here } else { Search::Top };
            return Err(raised.into());
        }

        // Every other condition. `HALT` is the one whose untrapped default
        // action reports; the rest are silent, and both halves are below.
        let halt = raise.condition.as_ref() == b"HALT";
        if raise.result.is_none() || !returns {
            // No tail, or `EXIT`: the program ends here and no trap is
            // consulted at any level. `Flow::Exit` carries `EXIT`'s own
            // value, which is `None` for the tail-less form.
            if halt {
                let mut raised = Raised::halt();
                raised.delivery.search = Search::Nobody;
                return Err(raised.into());
            }
            return Ok(Flow::Exit(result));
        }

        // `RETURN`: this routine returns `result`, and the condition is
        // offered to the caller.
        let name: Box<[u8]> = raise.condition.clone();
        // `RC` for `ERROR`/`FAILURE` is the raise's own argument, measured at
        // `rc= 5` for `raise error 5` trapped one level up. `SYNTAX`'s own
        // `RC` is the major and is filled in by `Raised::syntax` above.
        let rc = match raise.condition.as_ref() {
            b"ERROR" | b"FAILURE" => rc_text,
            _ => None,
        };
        match self.caller_trap_for(&name) {
            // A `CALL ON` trap resumes, so the condition waits for the
            // caller's current clause to finish -- `deliver_pending_trap`
            // has the two transcripts that pin the wait.
            Some(trap) if trap.call => {
                self.pending_trap = Some(PendingTrap {
                    condition: name,
                    rc,
                    // The caller's own identity -- this activation is about
                    // to be popped, and `caller_trap_for` above just read
                    // that same activation's table. See the field's own doc
                    // comment for the three transcripts behind it.
                    activation: self.activations[self.activations.len() - 2].id,
                });
                Ok(Flow::Return(result))
            }
            // A `SIGNAL ON` trap transfers, so the caller's clause is
            // abandoned rather than finished: measured, `say fun(1)` with a
            // trapped `raise user foo return 'RETVAL'` inside `fun` prints
            // nothing at all before the handler. That needs a real failure
            // unwinding this activation, not a value returned from it.
            Some(_) => {
                let mut raised = Raised::condition(condition_name(&name));
                raised.rc = rc;
                raised.delivery.search = Search::Caller;
                Err(raised.into())
            }
            // Nothing traps it. `HALT` reports; everything else is ignored
            // outright and the routine simply returns its value -- measured,
            // `raise user foo return 'RETVAL-88'` with no trap anywhere
            // prints `RETVAL-88` and the caller carries on.
            None if halt => {
                let mut raised = Raised::halt();
                raised.delivery.search = Search::Nobody;
                Err(raised.into())
            }
            None => Ok(Flow::Return(result)),
        }
    }

    /// `RAISE PROPAGATE`: re-raise the condition whose handler is running.
    ///
    /// **Measured, and it is not "raise it again in the caller".** From
    /// inside a `SIGNAL ON SYNTAX` handler, with another `SIGNAL ON SYNTAX`
    /// enabled one and two levels out, `raise propagate` is trapped by
    /// *neither* -- it is fatal, at the same rc the untrapped condition
    /// would have had, with the whole echo stack printed and the major line
    /// missing its ` running <path> line <n>` span
    /// ([`Delivery::positionless`]). So it goes to nobody.
    ///
    /// With no handler running at all it is `98.918`, "No active condition
    /// available for PROPAGATE", at rc 158 -- also measured, and the reason
    /// `active_condition` is an `Option` rather than something assumed
    /// present.
    ///
    /// A condition with no report to give ends the program silently instead,
    /// which is the `USER` half: measured, `raise propagate` inside a `CALL
    /// ON USER FOO` handler prints nothing more and exits 0.
    ///
    /// **What used to be a stated residual here is now measured, and it was
    /// a divergence** (fix round 1's finding 2). This comment said
    /// `active_condition` is "never cleared, so a `RAISE PROPAGATE` reached
    /// after a handler has finished re-raises that handler's condition where
    /// the oracle *may well* answer 98.918. Nothing measured pins that shape
    /// either way." One probe pinned it: the oracle does answer 98.918, and
    /// we answered silence at rc 0. `deliver_pending_trap` clears the field
    /// in its `Ended::Returned` arm now.
    ///
    /// The clearing is deliberately *not* symmetric. A `SIGNAL ON` handler
    /// that runs on -- `SIGNAL`s to another label and only then propagates --
    /// must still find its condition, also measured, so `offer_to_trap` has
    /// no equivalent line and a condition stays active for as long as its
    /// `SIGNAL` handler's activation does.
    fn exec_raise_propagate(&mut self) -> Result<Flow, Failure> {
        let Some(active) = &self.active_condition else {
            return Err(Raised::syntax(98, 918, Vec::new()).into());
        };
        if !active.raised.reportable() {
            return Ok(Flow::Exit(None));
        }
        let mut raised = active.raised.clone();
        raised.delivery.search = Search::Nobody;
        raised.delivery.positionless = true;
        // The original condition's echo stack, put back exactly as it stood
        // when the trap cleared it. `record_failure_at` is first-wins, so
        // restoring a full `failure_site` is also what stops this `raise
        // propagate` clause recording itself over the clause that actually
        // raised -- measured, the oracle echoes line 8 (`say 1/0`), not line
        // 12 (`raise propagate`).
        self.failure_site = active.site.clone();
        self.failure_sites = active.sites.clone();
        Err(raised.into())
    }

    /// Resolves a `SIGNAL`/`SIGNAL VALUE` target against the running
    /// *activation's* own body -- not `code.body`, which differs inside an
    /// `INTERPRET` fragment (whose own `labels` is always empty, a label in
    /// interpreted text being 47.1). Mirrors `resolve_and_run_call`'s
    /// identical fix for `CALL`, immediately below (`run.rs:2153-2154` in
    /// the tree this task started from) -- found there by running the
    /// composition rather than reading the code, and true of `SIGNAL` for
    /// the same reason: measured, `interpret "signal there"` reaches an
    /// enclosing `there:`, and `call sub` into `sub:` containing `signal
    /// caller_label` reaches a label back in the caller's own text, because
    /// at this phase every internal `CALL` target shares its caller's exact
    /// body (no `::routine` directive gives it one of its own yet) -- not
    /// because `SIGNAL` reaches across an activation boundary on its own.
    ///
    /// **No fallback, unlike `CALL`'s builtin/external search.** A `SIGNAL`
    /// target is only ever a label; the oracle's own answer when nothing
    /// matches is Error 16.1, and this crate can raise it directly rather
    /// than deferring to a later phase's table the way `resolve_and_run_
    /// call`'s own unresolved-name path has to.
    fn resolve_signal_target(&self, name: &[u8]) -> Result<usize, Failure> {
        let program = Rc::clone(&self.activation().program);
        let selector = self.activation().body;
        let Some(activation_body) = body_of(&program, selector) else {
            return Err(Loud::missing_body().into());
        };
        match activation_body.labels.get(name) {
            Some(target) => Ok(*target),
            None => Err(Raised::label_not_found(name).into()),
        }
    }

    /// Resolves `name`, evaluates the arguments and runs the resolved target
    /// in its own nested activation -- the whole middle of a call, shared
    /// between `exec_call` (`CALL`, which settles `RESULT` and translates
    /// the outcome into a `Flow`) and `eval_call` (`ExprKind::Call`'s
    /// expression form, `eval.rs`, Task 4, which never touches `RESULT` and
    /// has no `Flow` to report through since `eval` returns a value, not a
    /// step outcome).
    ///
    /// **Extracted rather than duplicated, by Task 4.** Both callers need
    /// the identical resolution order, the identical argument-evaluation
    /// discard, the identical `MAX_ACTIVATION_DEPTH` guard and the identical
    /// three-piece indent bookkeeping around the nested `run_activation` --
    /// measured to matter for the expression form too (`trace r` under a
    /// flat `zz = f(1) + 1` echoes `f`'s own clauses at the calling clause's
    /// indent plus two, the same D2r rule `CALL` already carries) -- and a
    /// second hand-copied version of this is exactly the drift this crate's
    /// other shared tables (`owners.rs`, `phase-4-exclusions.txt`) exist to
    /// avoid one level up. Task 3's own logic is unchanged by the split: the
    /// text moved, nothing about what it does did, and `exec_call`'s own
    /// extensive test suite is what confirms that rather than a claim about
    /// the diff.
    ///
    /// `search_labels` is false for `CALL "name"` and for `ExprKind::Call`'s
    /// `CallTarget::Literal`, and its own call sites have the measurements.
    ///
    /// **Resolution order is internal label, then builtin, then external**,
    /// and 4b builds only the front of it: a name that is not a label of the
    /// calling body fails loudly naming `4c`, which owns the builtin table.
    /// That is the right answer for `CALL "SUB"` even with `sub:` in the
    /// program -- the oracle's own Error 43.1 there is a statement that
    /// nothing outside the label table matched either, which is knowledge
    /// this phase does not have.
    ///
    /// **A same-file `::routine` is reachable for any non-builtin name, and
    /// is deferred rather than out of reach.** Measured: `call zorkolo` into
    /// `::routine zorkolo` dispatches on the oracle, where this falls through
    /// to the loud `4c` answer. What stops 4b dispatching is the step in
    /// front: a name that *collides* with a builtin must go to the builtin
    /// (measured, `::routine max` alongside `call max 1,2` still calls the
    /// builtin), and without 4c's table this arm would silently run the wrong
    /// routine instead of failing loudly -- which is the one outcome the
    /// failing-loudly rule exists to exclude. `Activation::body`'s own doc
    /// has what that costs, what whoever closes it inherits, and why the
    /// `max` probe alone could not tell "deferred" from "unreachable".
    pub(crate) fn resolve_and_run_call(
        &mut self,
        code: &Code<'_>,
        name: &[u8],
        search_labels: bool,
        args: &[Option<Expr>],
    ) -> Result<Ended, Failure> {
        // **Resolved against the running *activation's* body, not against
        // `code.body`, and the two differ inside an `INTERPRET` fragment.**
        // A fragment's `labels` is always empty -- a label in interpreted
        // text is error 47.1 -- so searching `code.body` would make every
        // `CALL` inside a fragment unresolvable. Measured on the oracle:
        // `interpret "call sub"` runs the enclosing program's `sub:`. Found
        // by running the composition rather than by reading the code: the
        // first version of this function searched `code.body` and passed
        // every test that had no `INTERPRET` in it.
        let program = Rc::clone(&self.activation().program);
        let selector = self.activation().body;
        let Some(activation_body) = body_of(&program, selector) else {
            return Err(Loud::missing_body().into());
        };
        let target = if search_labels {
            activation_body.labels.get(name).copied()
        } else {
            None
        };
        let Some(target) = target else {
            return Err(Loud::unresolved_call(name).into());
        };

        // **Evaluated in the caller, before anything is pushed**, which is
        // where the argument expressions' own variables live. Unobservable
        // in this task except through failure -- `USE ARG` (Task 4) and
        // `ARG()` (4c) are what read an argument, and both are still loud --
        // but the failure is real and measured: `call sub 1/0` is Error 42.3
        // reported against the `CALL` clause, at rc 214, and a version that
        // skipped evaluation would run the callee instead.
        //
        // An omitted position (`call sub 1,,3` parses as `[Some, None,
        // Some]`) stays a `None` here rather than being skipped or closed
        // up: measured, that call into three `USE ARG` targets gives `[1]
        // [Q] [3]`, so an omission holds its place and leaves its target
        // unset instead of shifting the ones after it. Task 3 evaluated and
        // discarded these; Task 5 keeps them, which is what that comment
        // said whoever landed `USE ARG` would do.
        //
        // **`>A>` fires here, once per position, omitted ones included**
        // (Task 9). The indent is the *calling* clause's own, read fresh on
        // each pass rather than captured once, because an argument
        // expression can itself contain a call whose callee overwrites
        // `current_value_indent` -- `resolve_and_run_call` restores it on
        // the way out, so re-reading it is what keeps a second argument's
        // own line at the caller's indent rather than at the first
        // argument's callee's. Measured (`trace i`): `call sub 1,,3` traces
        // `>A>   "1"`, `>A>   ""`, `>A>   "3"`, in that order, each right
        // after its own argument's `>L>`/`>V>` lines.
        let mut arguments: Vec<Option<Argument>> = Vec::with_capacity(args.len());
        for arg in args {
            match arg {
                None => {
                    // An omitted position traces an **empty** value line, not
                    // no line: `traceArgument(GlobalNames::NULLSTRING)`,
                    // `RexxInstruction.cpp:161`, and measured above.
                    self.trace_argument(self.clause_state.current_value_indent, b"");
                    arguments.push(None);
                }
                Some(expr) => {
                    let argument = self.eval_argument(code, expr)?;
                    self.roots.push_temp(argument.value());
                    let rendered = self.to_text(argument.value()).to_vec();
                    self.trace_argument(self.clause_state.current_value_indent, &rendered);
                    arguments.push(Some(argument));
                }
            }
        }

        // `SIGL`, set here rather than before the argument loop above: the
        // oracle's own `internalCall` (`RexxActivation.cpp`, read directly)
        // receives its arguments already evaluated by its caller, so they
        // are evaluated under whatever `SIGL` was already in force, and only
        // then does the transfer overwrite it. Measured: `signal there` /
        // `there: call sub sigl` into `sub: use arg a` reports the argument
        // as `1` (the `SIGNAL`'s own line, still in force during evaluation)
        // and `sub`'s own `SIGL` as the `CALL`'s line -- a version setting
        // `SIGL` before evaluating arguments would report the argument as
        // the `CALL`'s own line instead.
        self.set_sigl(self.clause_state.line());

        // D19/I6: one Rust frame per activation, plus this counter, so an
        // unbounded recursion becomes a reportable condition instead of a
        // native abort. `Raised::insufficient_stack` already existed
        // (`error.rs`); measured, the oracle answers the same 11.1 at rc 245
        // for the same program, at its own depth of 27,314.
        if self.activations.len() >= MAX_ACTIVATION_DEPTH {
            return Err(Raised::insufficient_stack().into());
        }

        // **D9r's default: a shared pool.** The callee reuses the caller's
        // `SlotFrame`, so it reads and writes the caller's variables and its
        // writes survive the return -- measured, and `pop_slots` is
        // deliberately not called on the way out because the frame is not
        // this activation's to free. Task 5's `PROCEDURE` is what will ever
        // push a frame of its own.
        //
        // `extra` is cloned in and moved back out for the same reason: it is
        // the *name* half of that one pool (`plan.rs`'s own `slot_of`), and
        // leaving the callee with an empty one would strand a name bound at
        // run time inside it. Measured on the oracle -- a callee running
        // `interpret "zork = 42"` and a caller then saying `zork` prints 42,
        // which needs the binding as well as the slot to cross the return.
        // Empty in every program that has no `INTERPRET` and no `DROP (v)`,
        // which is why the clone is not a cost worth avoiding.
        let caller = self.activation();
        let plan = Rc::clone(&caller.plan);
        let frame = caller.frame;
        let settings = caller.settings.clone();
        let trace_mode = caller.trace_mode;
        let extra = caller.extra.clone();
        // Cloned in and never written back, exactly like `settings` and
        // `trace_mode` beside it -- `Activation::traps`' own doc comment has
        // the three probes that measure the inheritance and its one-way
        // direction.
        let traps = caller.traps.clone();
        let callee_id = self.next_activation_id();
        let mut callee = Activation::nested(
            callee_id,
            program,
            selector,
            plan,
            frame,
            target,
            Inherited {
                settings,
                trace_mode,
                traps,
            },
        );
        callee.extra = extra;
        self.activations.push(callee);

        // Level state for the callee, five pieces, saved here and restored
        // on both paths below. `Interpret`'s own arm is the model for four
        // of them, and one differs from it deliberately -- the fifth,
        // `clause_state`, is not level state for the callee at all, and is
        // saved and restored for a different reason stated where it is:
        //
        // * `activation_indent` is **set** to the calling clause's printed
        //   indent plus two (D2r). Measured at three shapes rather than one,
        //   because "2 x depth" agrees with the truth at caller indent 0 and
        //   parts company immediately after: a flat `call` echoes the callee
        //   at 2, one `DO` deep at 4, two `DO`s deep at 6.
        // * `indent_offset` is zeroed alongside it, exactly as the fragment
        //   case is and for the same reason -- the calling clause's printed
        //   indent already contains any escape elevation, and leaving this
        //   would count it twice.
        // * `clause_line_override` is **cleared**, where `INTERPRET` sets it.
        //   Each activation's echo carries its *own* line, and the clearing
        //   is what makes that true inside a fragment: measured, `interpret
        //   "call sub"` on line 2 echoes the fragment's `call sub` at line 2
        //   and the callee's own clauses at lines 4, 5 and 6. Leaving the
        //   enclosing override in force would print all six as line 2.
        // * `call_context` is set to this call's own name and arguments, so
        //   a `USE ARG` inside the callee reads its own rather than an
        //   enclosing call's (added by Task 5, and saved here rather than
        //   anywhere else precisely because of the finding just below:
        //   `current_value_indent`, the fourth piece at the time, had gone
        //   unrestored, unobservable until two activations per clause
        //   became reachable).
        //
        // * `clause_state` (`current_value_indent`/`current_clause_line`,
        //   bundled -- that struct's own doc comment has the property that
        //   puts the two of them here rather than among the four above)
        //   is **saved whole and restored whole**, never set to anything
        //   new going in: `run_activation` -> `step_in_temps_frame`
        //   overwrites both fields on every clause the callee steps, the
        //   same way the caller's own next clause would regardless. Before
        //   `ExprKind::Call` at most one activation could be entered per
        //   clause, and that next clause's own `step_in_temps_frame`
        //   re-set both fields before anything read them -- so a version
        //   missing this restore passes every test with no more than one
        //   call per clause in it, and `say f(1) + g(2)` (two activations,
        //   one clause) is what makes the omission observable at all.
        //   `current_value_indent`'s own restore is review finding C1
        //   (Task 4 fix round 1): without it, `g`'s own base indent (and
        //   everything computed from it, including the enclosing clause's
        //   own `>>>`) reads `f`'s last clause instead of the caller's own.
        //   `current_clause_line`'s is Task 6 fix round 2, found the
        //   identical way after shipping without it: without this line,
        //   `g`'s own `SIGL` (`set_sigl` reading `current_clause_line`)
        //   reads `f`'s own last line instead of the calling clause's.
        //   `current_clause_line_is_restored_after_a_nested_call_or_signal`
        //   (this file's own tests) is what fails if this one line is
        //   ever removed a second time; `current_value_indent_is_restored_
        //   after_a_nested_expression_call` is its own sibling for the
        //   other field.
        //   The pair is `save_clause_state`/`restore_clause_state` rather
        //   than two plain assignments (fix round 4): a `ClauseState` this
        //   function could copy freely was also one it could *replace*, which
        //   is a clause line set with no boundary attached -- the exact thing
        //   `clause.rs` exists to make unwritable.
        let saved_clause_state = self.save_clause_state();
        let saved_base = std::mem::replace(
            &mut self.activation_indent,
            saved_clause_state.value_indent() + 2,
        );
        let saved_offset = std::mem::take(&mut self.indent_offset);
        let saved_line = std::mem::take(&mut self.clause_line_override);
        let saved_context = std::mem::replace(
            &mut self.call_context,
            CallContext {
                name: name.to_vec(),
                arguments,
            },
        );

        let ended = self.run_activation();

        // Popped on both paths, and unconditionally: `run_activation`'s own
        // loop asserts the activation stack is where it found it after every
        // step, so a `CALL` that returned with the callee still on it would
        // trip that assertion in the caller rather than quietly running the
        // wrong frame's `pc`.
        let callee = self.activations.pop().expect("the activation just pushed");
        // **The two halves of "was the pool shared" are one bool, and both
        // are needed.** A `PROCEDURE` callee pushed a frame of its own, so
        // that frame is popped here -- on the error path as well, which is
        // why this is not inside the `Ok` arm below. It also keeps its own
        // run-time name bindings, so they are *not* moved back: doing that
        // would overwrite the caller's `extra` with the callee's isolated
        // one. A shared-pool callee is the opposite on both counts, and its
        // `extra` write-back is what makes a name bound inside it survive
        // the return (measured, `interpret "zork = 42"` in a callee).
        if callee.owns_frame {
            self.roots.pop_slots(callee.frame);
        } else {
            self.activation_mut().extra = callee.extra;
        }
        self.activation_indent = saved_base;
        self.indent_offset = saved_offset;
        self.clause_line_override = saved_line;
        self.restore_clause_state(saved_clause_state);
        self.call_context = saved_context;

        match ended {
            Ok(ended) => Ok(ended),
            Err(failure) => {
                // Seal before the failure leaves the callee, never after --
                // `seal_site_level`'s own rule, and the same one
                // `run_fragment` follows. Without it the callee's clause
                // would win `record_failure_at`'s first-wins race outright
                // and the call would never be echoed. Measured, the oracle
                // prints one echo per level, innermost first: a `say 1/0` in
                // a routine called from a routine called from a `DO` gives
                // three lines, at indents 6, 4 and 2.
                self.seal_site_level();
                Err(failure)
            }
        }
    }

    /// Evaluates one call argument, keeping the caller's slot when the
    /// argument is a variable reference (`>name` or `<name`).
    ///
    /// **Every argument has a value and only some have a slot**, which is
    /// what `Argument`'s two variants say. A variable reference decays to
    /// the referenced variable's value everywhere except a `USE ARG >`
    /// target -- measured, `say >p` prints `p`'s value, and `call sub2 >p`
    /// into a plain `use arg q` binds that value and leaves the caller's `p`
    /// alone. So the value is computed here for both variants, by evaluating
    /// the inner expression through the ordinary path rather than by reading
    /// the slot directly, which is what keeps a stem reference rendering the
    /// way a bare stem read does.
    ///
    /// The inner node is always a `Variable` or a `Stem` (`rexx-parse`'s own
    /// doc on `ExprKind::VariableReference`; anything else is error 20.930 at
    /// parse time), so it names exactly one slot. The `other` arm is the same
    /// belt-and-braces shape the `Assignment` arm's own comment describes: a
    /// guarantee the grammar makes is not one the type system enforces, and
    /// this crate fails loudly rather than trusting it blindly.
    ///
    /// **The value is computed by evaluating the reference node itself, not
    /// its inner variable** (Task 9). Both spellings reach the identical
    /// value either way -- `eval_node`'s own `VariableReference` arm is
    /// `self.eval_node(code, inner)` -- but only the outer call reaches
    /// `trace_intermediate`'s own `VariableReference` arm, and the two
    /// differ by a measured line: the oracle traces `>O>   ">" => "PQ"`
    /// here, where evaluating the inner node through `eval` traced
    /// `>V>   PQ => "val"` instead.
    fn eval_argument(&mut self, code: &Code<'_>, expr: &Expr) -> Result<Argument, Failure> {
        let ExprKind::VariableReference(inner) = &expr.kind else {
            return Ok(Argument::Value(self.eval(code, expr)?));
        };
        let id = match &inner.kind {
            ExprKind::Variable(id) | ExprKind::Stem(id) => *id,
            other => return Err(Loud::expression(other).into()),
        };
        let slot = match code.slots.get(&id) {
            Some(slot) => *slot,
            None => self.slot_of(code.symbols.name(id).as_bytes()),
        };
        let frame = self.activation().frame;
        // Chased here, in the caller, where any alias the caller itself
        // holds is still addressable -- the same one-step chase exposure
        // uses, and what makes `>p` work when the caller's own `p` is
        // already exposed from *its* caller.
        let target = self.roots.slot_ref(frame, slot);
        let value = self.eval(code, expr)?;
        // The referenced variable's own spelling travels with the reference:
        // it is the reference's *kind* (`P` against `P.`) for the
        // 88.929/88.930 check, and it is what those two errors substitute.
        // Read from the caller's own symbol table, here, where it is the
        // right one.
        let name = code.symbols.name(id).as_bytes().into();
        Ok(Argument::Reference {
            target,
            value,
            name,
        })
    }

    /// Runs one named `CALL`: `resolve_and_run_call`, then settle `RESULT`
    /// and translate the outcome into this instruction's own `Flow`. See
    /// `resolve_and_run_call`'s own doc for the resolution order, the
    /// argument-evaluation and indent-bookkeeping detail this used to carry
    /// directly, and why it is shared with `eval_call` (`eval.rs`, Task 4)
    /// rather than duplicated.
    fn exec_call(
        &mut self,
        code: &Code<'_>,
        name: &[u8],
        search_labels: bool,
        args: &[Option<Expr>],
    ) -> Result<Flow, Failure> {
        // Captured before `resolve_and_run_call` runs the callee, which
        // overwrites `current_value_indent` with its own clauses' -- this is
        // the `CALL` clause's own printed indent, needed below for the
        // caller-side `RESULT` trace.
        let base_indent = self.clause_state.current_value_indent;
        let ended = self.resolve_and_run_call(code, name, search_labels, args)?;

        let value = match ended {
            // `EXIT` inside the callee ends the program rather than the
            // call, and so does running off the end of the body -- measured
            // both ways. Forwarded unchanged; `RESULT` is never touched on
            // this path.
            Ended::Exited(value) => return Ok(Flow::Exit(value)),
            Ended::Returned(value) => value,
        };

        // **`RESULT` is settled on return and not at the call.** Measured:
        // a caller setting `result = 'before'` and calling a no-`PROCEDURE`
        // routine has the callee print `inside result= before`, so nothing
        // is cleared on the way in. After `return 42` the caller reads `42`;
        // after a bare `return` it reads the derived name `RESULT`, which is
        // what an unset variable renders as.
        let slot = self.slot_of(b"RESULT");
        let frame = self.activation().frame;
        match value {
            Some(value) => {
                // Re-rooted in the caller: `step_in_temps_frame` popped the
                // callee's temps frame around every clause it ran, this one
                // included, so the `push_temp` the `RETURN` arm did is gone
                // by now. Same window `Flow::Exit`'s own arm documents,
                // closed here rather than left open, because unlike an exit
                // value this one goes on to be stored and read.
                self.roots.push_temp(value);
                let rendered = self.to_text(value).to_vec();
                // The caller's own `>>>`, at the `CALL` clause's indent --
                // `base_indent`, saved before the callee overwrote
                // `current_value_indent` with its own clauses'.
                self.trace_result(base_indent, &rendered);
                self.roots.set_slot(frame, slot, value);
            }
            None => self.roots.clear_slot(frame, slot),
        }
        Ok(Flow::Next)
    }

    /// Runs one instruction inside its own temps frame.
    ///
    /// The frame is opened and closed **here rather than inside `step`**,
    /// because `step` returns through a dozen `?` paths and a frame closed on
    /// only some of them is worse than none: it would leak on exactly the
    /// paths nobody tests. Closing it around the call covers every exit,
    /// including the loud failures.
    ///
    /// One clause is the right lifetime for a temporary. It is also what the
    /// C++ does, and it is why `step` can push freely without deciding when to
    /// let go.
    ///
    /// **No longer quite true of a `DO`/`LOOP` clause, since Task 11
    /// (F-EX4, branch review, Minor).** `run_loop`/`run_repeating` resolve
    /// an entire multi-pass loop inside this one call -- the doc comment
    /// two paragraphs below explains why a `Goto`-shaped re-entry cannot be
    /// used instead -- so everything pushed per pass (`eval_condition`'s own
    /// `push_temp` for every `WHILE`/`UNTIL` test, one `ObjRef` per
    /// iteration) accumulates for the loop's whole run rather than one
    /// iteration's. Not a correctness defect: nothing collects mid-run, and
    /// the temps are rooted throughout, so a stress collector sees nothing
    /// but live roots. It costs memory a future collector cannot reclaim
    /// early (a `do while` running 10^7 passes holds ~10^7 dead-but-rooted
    /// temps in one frame), and it means "one clause" describes every
    /// instruction here except this one.
    ///
    /// **Also resolves the failing clause's site, when one escapes and
    /// `source` is `Some`.** Moved here from `run_activation`'s own error
    /// path (Task 10), because `run_activation` only ever sees the outermost
    /// instruction it was stepping, and Task 10 nests `step_in_temps_frame`
    /// calls arbitrarily deep through `If`/`Select`'s own `run_bounded` --
    /// without this, an error raised inside a `WHEN`'s branch would be
    /// misattributed to the enclosing `SELECT`'s own clause (measured
    /// against the oracle before this existed: a `1/0` inside a matched
    /// `WHEN`'s `THEN` reported the `SELECT`'s line and text, not the
    /// failing clause's).
    ///
    /// **This alone is not enough for a `WHEN`/`WhenCase` whose own
    /// *condition* raises**, and a second, real defect this task's own
    /// review found: `Select`'s own arm evaluates a `When`/`WhenCase`'s
    /// condition directly, as data, and never opens a `step_in_temps_frame`
    /// for the `When`/`WhenCase` instruction itself (that instruction's own
    /// `step` arm is a pure no-op, per the `When`/`WhenCase` arm's own doc
    /// comment) -- so a raise there has no inner wrapper call to be the
    /// "innermost" one, and would still be attributed to the `SELECT`.
    /// Measured: `select` / `when 'x' then nop` / `end` reported the
    /// `SELECT`'s own line and clause, not the `WHEN`'s. `Select`'s own arm
    /// calls `record_failure_site` directly, past `code.body.instructions[
    /// when_index]`, on exactly that path -- see its own call sites there.
    ///
    /// An early `if self.failure_site.is_some() { return; }` at the top of
    /// [`Interp::record_failure_at`] is the guard that makes the *first*
    /// resolution win, which is always the most specific one available: the
    /// deepest `step_in_temps_frame` call, or `Select`'s own direct call for
    /// a `When`/`WhenCase` condition, always runs before any enclosing
    /// propagation reaches an outer wrapper.
    ///
    /// **This paragraph said "`self.failure_site.is_none()` is the guard, in
    /// both callers" and was wrong about the expression and about where it
    /// lives** -- corrected at 4b's Task 7, whose own brief flagged it,
    /// because a task told to clear that field would otherwise go looking in
    /// the callers and find nothing.
    ///
    /// **First-wins is per *level*, and an `INTERPRET` fragment is a level**
    /// (4b's Task 2). The guard is unchanged; what changed is that
    /// `run_fragment` calls `seal_site_level` on its way out, so the clause
    /// this guard protects is the first one recorded *since the current level
    /// opened*, not the first one recorded in the whole run. Without that,
    /// the fragment's own clause would win the race outright and the
    /// enclosing `INTERPRET` would never be echoed at all -- which is the
    /// second of the two ways the obvious one-line fix was measured wrong.
    fn step_in_temps_frame(
        &mut self,
        code: &Code<'_>,
        index: usize,
        instruction: &Instruction,
        source: Option<&ProgramSource>,
    ) -> Result<Flow, Failure> {
        // `TRACE`'s own `*-*` clause echo (D17), and the single insertion
        // point for it -- exactly the analogue of `eval`'s own split from
        // `eval_node`, since this is the one place `run_bounded`'s loop
        // (this function's only non-test caller) visits every flat
        // instruction position it steps, markers (`Then`/`Else`/
        // `Otherwise`/`When`/`WhenCase`/`Label`) included, matching the
        // oracle's own `RexxInstruction::traceInstruction`, which every one
        // of those calls too from its own `execute`.
        //
        // **Not the whole story for a `DO`/`LOOP`.** The oracle's own `DO`
        // instruction is re-executed once per iteration, so its own clause
        // (and `END`'s) echoes again on every pass -- this call site fires
        // exactly once, when the `DO`/`LOOP` instruction is first stepped,
        // because `run_loop`/`run_repeating` resolve every iteration inside
        // *this* one `step` call rather than returning between passes
        // (`Flow::Leave`'s own doc comment has the reason: a `Goto`-shaped
        // re-entry risks the absorption trap that design avoids). The
        // per-iteration re-echo is `run_repeating`'s own, separate call
        // into `trace_clause` -- see its doc comment.
        // `current_value_indent` (`lib.rs`'s own doc comment on the field)
        // is set here **unconditionally**, not only when `trace_mode.all`
        // -- `INTERMEDIATES` implies `all` (`TraceMode`'s own doc comment),
        // never the reverse, so anything that reads this field is already
        // gated by its own `intermediates` check; setting it plainly is
        // cheaper than a second `if` that would just repeat that gate.
        //
        // `printed_indent` rather than `static_indent` directly, so that
        // *which* offsets apply is one fact in one place -- see its own doc
        // comment for what it adds and why open-coding it was a defect.
        let indent = self.printed_indent(&code.body.instructions, index);
        self.clause_state.current_value_indent = indent;
        // Set unconditionally, exactly like `current_value_indent` just
        // above and for the identical reason (that field's own doc comment):
        // `SIGL` (`lib.rs`'s doc on `current_clause_line`) has to stay
        // correct whether or not `TRACE` is on, and this is the one place
        // every stepped instruction, `SIGNAL`/`CALL` included, passes
        // through before its own `step` call runs.
        // `in_clause` rather than a bare assignment: the clause line and the
        // clause boundary are one operation (`clause.rs`), and the clause's
        // whole body is the closure below.
        let line = self
            .clause_line(source, instruction)
            .unwrap_or_else(|| self.clause_state.line());
        let outcome = self.in_clause(code, line, |it| {
            if it.trace_mode().all
                && let Some((line, text)) = it.clause_site(source, instruction)
            {
                it.trace_clause(line, indent, &text);
            }
            // The debug tripwire I22 scheduled in 4a and left unbuilt, added
            // here by 4b's Task 1 along with `RootSet::temps_len`, its one
            // prerequisite.
            //
            // **What it checks, and why it is here rather than in
            // `pop_frame`.** `pop_frame` truncates to a watermark rather than
            // popping one frame, and its own doc comment forbids a balance
            // assertion there: six `eval.rs` sites open a frame and then use
            // `?`, so their own `pop_frame` goes unreached on the error path
            // and is healed by this function's unconditional, outer
            // truncation -- an assertion inside `pop_frame` would fire on the
            // ordinary error path of a correct program. That healing is
            // exactly what makes `Err` uninteresting to check and `Ok`
            // interesting: on the `Ok` path every site did run its own
            // `pop_frame`, so the stack must be back at or above where this
            // step found it. Below it means a step popped temps it did not
            // own -- someone else's roots, dropped early, which is the
            // direction that could turn into a use-after-free once a
            // collector runs for real.
            //
            // Cheap enough to leave on in debug and absent in release: one
            // `Vec::len` before and after, and a comparison.
            let temps_at_entry = it.roots.temps_len();
            let frame = it.roots.push_frame();
            let flow = it.step(code, index, instruction, source);
            debug_assert!(
                flow.is_err() || it.roots.temps_len() >= temps_at_entry,
                "step popped below its own temps watermark ({} -> {}), so it \
                 discarded roots it did not push",
                temps_at_entry,
                it.roots.temps_len()
            );
            it.roots.pop_frame(frame);
            if flow.is_err() {
                it.record_failure_site(code, index, source, instruction);
            }
            flow
        });
        match outcome {
            Ok(ClauseOutcome::Ran(flow)) => flow,
            Ok(ClauseOutcome::Ended(exit)) => Ok(Flow::Exit(exit.value())),
            // The handler run at this clause's boundary failed. The clause
            // the oracle blames is **this** one -- the one whose boundary
            // ran it -- not the enclosing instruction: measured, a failing
            // `CALL ON` handler queued by `call sub` inside a `DO` echoes
            // `3 *-* call sub`, where without this it echoed
            // `2 *-* do i = 1 to 1`, the `DO` clause's own site recorded one
            // level out. Fix round 3's NEW-B. The outer `Err` is the
            // handler's alone -- this clause's own failure comes back as
            // `Ran(Err(_))` above -- which is what keeps the two apart.
            Err(failure) => {
                self.record_failure_site(code, index, source, instruction);
                Err(failure)
            }
        }
    }

    /// Resolves `instruction`'s own clause (and its statically-derived
    /// indent, `static_indent`) into `self.failure_site`, first call wins,
    /// when `source` is `Some`.
    ///
    /// **The guard is not here and is not spelled `is_none()`** -- this line
    /// used to say it was both. It is an early `if self.failure_site.
    /// is_some() { return; }` at the top of [`Interp::record_failure_at`],
    /// which this function's own last line delegates to. Corrected at 4b's
    /// Task 7, the task that had to clear the field.
    ///
    /// The shared half of `step_in_temps_frame`'s own resolution (its doc
    /// comment has the full argument for why the *first* caller to run this
    /// is always the right one) -- factored out so `Select`'s own arm can
    /// call it directly for a `When`/`WhenCase` whose *condition* raises,
    /// which never goes through `step_in_temps_frame` at all since that
    /// instruction's own `step` arm never runs for a decision of its own.
    ///
    /// `index` is `instruction`'s own position in `code.body.instructions`,
    /// needed (beyond what `step_in_temps_frame` already required it for)
    /// so `static_indent` has something to walk the flat instruction list
    /// up to.
    ///
    /// Goes through `printed_indent`, same as `step_in_temps_frame`'s own
    /// indent computation, so both offsets apply here exactly as they do
    /// anywhere else.
    ///
    /// **An earlier version of this paragraph said the escape elevation "is
    /// always `0` here in practice". That was false about `indent_offset`
    /// alone, before any fragment base existed.** Measured with the addend
    /// dropped from this function and no `INTERPRET` in the program: a `WHEN`
    /// *condition* that raises, inside a nested `SELECT` inside an escaped
    /// `OTHERWISE`, reports at 6 where the oracle prints 10 -- which is
    /// exactly the `Select`-direct-call case the claim was about. So the
    /// conclusion was retracted correctly and the premise behind it was kept
    /// and is also wrong; both go.
    ///
    /// Once the same machinery carried an `INTERPRET` fragment's base it was
    /// wrong more often rather than newly wrong: the base is non-zero for the
    /// whole life of the fragment, `Select`'s direct calls included. The base
    /// has its own field now (`activation_indent`), the addend is emphatically
    /// **not** always zero, and nothing below may
    /// assume it is.
    fn record_failure_site(
        &mut self,
        code: &Code<'_>,
        index: usize,
        source: Option<&ProgramSource>,
        instruction: &Instruction,
    ) {
        let indent = self.printed_indent(&code.body.instructions, index);
        self.record_failure_at(source, instruction, indent);
    }

    /// Assigns `blame`'s own clause to `self.failure_site` at exactly
    /// `indent` spaces, first call wins, when `source` is `Some`.
    ///
    /// The common tail `record_failure_site` itself uses (computing
    /// `indent` from `blame`'s own position first) and that `Do`'s own
    /// `WHILE`/`UNTIL` checks call directly with a *different* indent --
    /// neither corresponds to a flat instruction position `static_indent`
    /// resolves correctly on its own (`static_indent`'s own doc comment has
    /// the full argument), so `Do`'s own arm computes `WHILE`'s/`UNTIL`'s
    /// indent itself and hands it straight to this function rather than
    /// asking `record_failure_site` to guess between two different, both
    /// correct, answers for the same instruction index.
    fn record_failure_at(
        &mut self,
        source: Option<&ProgramSource>,
        blame: &Instruction,
        indent: usize,
    ) {
        if self.failure_site.is_some() {
            return;
        }
        if let Some((line, text)) = self.clause_site(source, blame) {
            self.failure_site = Some(FailureSite { line, text, indent });
        }
    }

    /// Captures a `LEAVE`/`ITERATE` instruction's own clause site and static
    /// indent the instant it steps, before any propagation -- see
    /// `Flow::Leave`'s own doc comment for why eagerly, and `LeaveOrigin`'s
    /// own doc comment for why `indent` is computed here rather than read
    /// back later. `clause_site` is the free function `record_failure_site`
    /// itself resolves through, shared rather than duplicated: this needs
    /// the same (line, text) pair, just held onto instead of assigned to
    /// `self.failure_site` immediately, since a `LEAVE`/`ITERATE` might
    /// still be consumed by an enclosing `Do`/`Select` rather than ever
    /// becoming a failure at all.
    fn leave_origin(
        &self,
        code: &Code<'_>,
        index: usize,
        source: Option<&ProgramSource>,
        instruction: &Instruction,
    ) -> LeaveOrigin {
        LeaveOrigin {
            site: self.clause_site(source, instruction),
            // `+ self.indent_offset` (F-EX1's own correction to F3,
            // `lib.rs`'s own doc comment): found missing here on the
            // *second* re-verification of F-EX1's own fix, not the first --
            // a `LEAVE`/`ITERATE` inside an escaped `OTHERWISE`'s own body
            // captures its own origin indent here, not through `step_in_
            // temps_frame`'s own computation at all (`Flow::Leave`'s own
            // doc comment: eagerly, before any propagation), so it needs
            // the identical addition independently, not by inheritance.
            indent: self.printed_indent(&code.body.instructions, index),
            // Already this instruction's own line: `step_in_temps_frame`'s
            // `in_clause` set it before dispatching this `step`, through the
            // same `clause_line` call `SIGL` reads.
            clause_line: self.clause_state.line(),
        }
    }

    /// Assigns `origin`'s own captured site to `self.failure_site`, first
    /// call wins, at `origin`'s own captured indent.
    ///
    /// **Corrected after review.** This crate's first cut of the
    /// LEAVE/ITERATE indent family hardcoded the exhausted-search family
    /// (28.1-28.4) to zero and reported `origin.indent` unmodified for
    /// 28.5, on the theory that those were the only two shapes the
    /// oracle's own indent could take. A reviewer's fourteen-point probe
    /// falsified that in seven cases (re-measured independently against
    /// the oracle before changing anything -- see the report): the actual
    /// rule is that `origin.indent` is the search's own *residual*, updated
    /// every time a frame the search examines is popped rather than
    /// matched, and this function's only job now is to report whatever
    /// `origin.indent` already holds by the time either family reaches its
    /// own resolution point -- there is exactly one caller-facing function
    /// for both families, because the difference between them was never in
    /// how the site gets recorded, only in how far the search walked before
    /// giving up. See `Do`'s and `Select`'s own arms (`do_body_outcome`,
    /// `leave_select`) for where the residual is actually updated, and
    /// `LeaveOrigin`'s own doc comment for the rule in full.
    fn record_leave_failure(&mut self, origin: &LeaveOrigin) {
        if self.failure_site.is_some() {
            return;
        }
        if let Some((line, text)) = &origin.site {
            self.failure_site = Some(FailureSite {
                line: *line,
                text: text.clone(),
                indent: origin.indent,
            });
        }
    }

    /// Turns the `Flow` a `SELECT`'s own matched `WHEN` or `OTHERWISE` body
    /// produced into this `SELECT`'s own answer.
    ///
    /// `Flow::Next` becomes `Goto(resume)`, exactly the shape every branch
    /// gave before Task 11. A `LEAVE`/`ITERATE` naming this `SELECT`'s own
    /// `label` (`Some` only for `SELECT LABEL name` -- an ordinary clause
    /// label in front of a `SELECT` is a separate `Label` instruction and
    /// never reaches `label` at all, measured 28.3/28.4 exactly as for an
    /// unlabelled loop) is consumed here: a matching `LEAVE` resumes past
    /// the whole `SELECT`; a matching `ITERATE` is **28.5**, because
    /// `SELECT` is never a repetitive loop (`RexxInstructionSelect::isLoop`
    /// answers `false` unconditionally, read directly in the report) --
    /// measured, `ITERATE` never accepts a non-loop target even when the
    /// name matches. Everything else -- an unnamed `LEAVE`/`ITERATE` (a
    /// `SELECT` is never a bare target either, same reason), or one naming
    /// something else -- is **not matched, but not untouched either**: a
    /// `SELECT` always owns a search frame (unconditionally, labelled or
    /// not -- unlike `Do`'s own unlabelled-`Simple` exception), so
    /// forwarding it outward resets `origin.indent` to this `SELECT`'s own
    /// `static_indent` first (`LeaveOrigin`'s own doc comment has the full
    /// rule and the oracle transcripts that pin it). `Exit` and a `Goto`
    /// that escaped `run_bounded`'s own range pass through with nothing
    /// touched, same as always.
    /// One listed `WHEN`/`WHEN CASE`'s own condition, as its own clause body.
    ///
    /// Extracted from `Select`'s own scan loop (fix round 4) for one reason:
    /// a `WHEN` is a clause, so its condition has to run inside an
    /// `in_clause` closure, and a closure that borrows the loop's own locals
    /// is easier to read as a named function than inline. `matched` is an
    /// out-parameter rather than the return value because the closure's
    /// return type is what `ClauseValue` is chosen from, and `()` is the
    /// honest answer -- a `WHEN`'s decision is a pair of instruction indices,
    /// not an `ObjRef` this clause's temps frame was the only root for.
    ///
    /// Every fallible call below is matched explicitly, never through `?`, so
    /// a failure can be attributed to `when_instruction` -- the
    /// `When`/`WhenCase` whose condition is actually being evaluated --
    /// before it propagates. Nothing here goes through `step_in_temps_frame`
    /// at all: `When`/`WhenCase`'s own `step` arm is a no-op (see its own doc
    /// comment), so without this a raise here would still be attributed to
    /// the enclosing `SELECT` instruction, which is exactly the defect
    /// `record_failure_site`'s own doc comment describes. Measured: `select` /
    /// `when 'x' then nop` / `end` must report the `WHEN`'s own line and
    /// clause, not the `SELECT`'s.
    #[expect(
        clippy::too_many_arguments,
        reason = "\
        one caller, and every argument is a value that caller already holds: \
        bundling them into a struct would only move the same list one line up"
    )]
    fn scan_when(
        &mut self,
        code: &Code<'_>,
        source: Option<&ProgramSource>,
        when_index: usize,
        when_instruction: &Instruction,
        when_indent: usize,
        case_text: Option<&[u8]>,
        len: usize,
        matched: &mut Option<(usize, usize)>,
    ) -> Result<(), Failure> {
        // `When`/`WhenCase`'s own clause echo, explicit for the same reason
        // `record_failure_site`'s own calls below are: that instruction's
        // `step` arm is a pure no-op (never independently dispatched, only
        // ever read as data by the scan), so nothing else ever calls
        // `step_in_temps_frame` for it and its `*-*` line would otherwise
        // never appear at all -- measured, `select` / `when 1 = 1 then ...`
        // echoes the `WHEN`'s own clause on its own line before anything
        // about its condition.
        if self.trace_mode().all
            && let Some((line, text)) = self.clause_site(source, when_instruction)
        {
            self.trace_clause(line, when_indent, &text);
        }
        *matched = match &when_instruction.kind {
            InstructionKind::When {
                condition,
                false_target,
                exit,
            } => {
                let holds = match self.eval_condition(
                    code,
                    condition,
                    ConditionTrace::Result(when_indent),
                    raised_when_not_logical,
                ) {
                    Ok(holds) => holds,
                    Err(failure) => {
                        self.record_failure_site(code, when_index, source, when_instruction);
                        return Err(failure);
                    }
                };
                holds.then(|| (false_target.unwrap_or(len), exit.unwrap_or(len)))
            }
            InstructionKind::WhenCase {
                values,
                false_target,
                exit,
            } => match case_text {
                Some(case_text) => {
                    let matched = match self.test_case_when(code, values, case_text, when_indent) {
                        Ok(matched) => matched,
                        Err(failure) => {
                            self.record_failure_site(code, when_index, source, when_instruction);
                            return Err(failure);
                        }
                    };
                    matched.then(|| (false_target.unwrap_or(len), exit.unwrap_or(len)))
                }
                // A listed `WhenCase` with no `case` expression: a plain
                // `SELECT` with no `CASE` at all, which the parser should
                // never produce for a `WhenCase` node (only `SELECT CASE`
                // ever builds one, `ast.rs`'s own doc comment) -- F-EX3, the
                // same unproven parser invariant the absorbed `WhenCase` arm
                // already refuses to crash on, and formerly an `.expect()`
                // here that did. Evaluates `values` for side effects and
                // never matches, the identical fallback.
                None => {
                    for value in values {
                        let v = self.eval(code, value)?;
                        self.roots.push_temp(v);
                    }
                    None
                }
            },
            other => panic!("a SELECT's whens holds only When/WhenCase, not {other:?}"),
        };
        Ok(())
    }

    /// Runs a `SELECT`'s own `OTHERWISE`, `leave_select`-wrapped -- the one
    /// dispatch every path that reaches `OTHERWISE` must go through,
    /// whether it got there the ordinary way (no `WHEN` matched) or through
    /// F-EX1's own escape redirect (a matched `WHEN`'s own bounded body
    /// produced a bare `Flow::Goto` landing exactly on `otherwise_index`).
    /// Extracted from what was `Select`'s own inline `Some(otherwise_index)`
    /// arm, unchanged in behaviour, so the second call site cannot drift
    /// from the first one's.
    ///
    /// `otherwise_index`'s own clause echo is explicit for the same reason
    /// `WHEN`'s own is (`Select`'s own arm, above): its `step` arm is a
    /// no-op and its own index is never inside any `run_bounded` range
    /// (the body below starts *after* it), so nothing else ever visits it.
    /// Measured, this task's report: `otherwise` traces on its own line, at
    /// the `SELECT`'s own scan level, before its body.
    fn run_otherwise(
        &mut self,
        code: &Code<'_>,
        index: usize,
        label: Option<SymbolId>,
        otherwise_index: usize,
        end: Option<usize>,
        source: Option<&ProgramSource>,
    ) -> Result<Flow, Failure> {
        let otherwise_end = end.unwrap_or(code.body.instructions.len());
        let otherwise_instruction = &code.body.instructions[otherwise_index];
        // `static_indent`'s own fixed answer for `otherwise_index` (this
        // task's earlier fix, not `select_indent`): the marker sits at the
        // scan level (2 at top level), the same as a `WHEN`'s own
        // condition, not the `SELECT`'s own level. `+ self.indent_offset`
        // (F-EX1's own correction to F3, `lib.rs`'s own doc comment on the
        // field): `0` on the ordinary "no `WHEN` matched" path this
        // function already served before F-EX1, and the absorbed
        // `WhenCase`'s own escape's residual on the redirect path F-EX1
        // added -- one computation serves both callers correctly because
        // the field itself, not this function, is what carries the
        // difference between them.
        let otherwise_indent = self.printed_indent(&code.body.instructions, otherwise_index);
        self.clause_state.current_value_indent = otherwise_indent;
        if self.trace_mode().all
            && let Some((line, text)) = self.clause_site(source, otherwise_instruction)
        {
            self.trace_clause(line, otherwise_indent, &text);
        }
        let flow = self.run_bounded(code, otherwise_index + 1, otherwise_end, source)?;
        // Restores the offset to `0` now that `OTHERWISE`'s own whole
        // dispatch (marker and body alike) is finished reading it --
        // `?` above already returned early without reaching this line if
        // `run_bounded` raised, which this crate's own rule leaves
        // unrestored deliberately: a raise in 4a is always fatal (no
        // `SIGNAL ON`/condition trapping exists yet), so nothing runs
        // afterward to see a stale value, the same reasoning `lib.rs`'s
        // own doc comment gives for never restoring it after `END`'s own
        // 7.3 either.
        self.indent_offset = 0;
        self.leave_select(code, index, label, otherwise_end, flow)
    }

    fn leave_select(
        &mut self,
        code: &Code<'_>,
        index: usize,
        label: Option<SymbolId>,
        resume: usize,
        flow: Flow,
    ) -> Result<Flow, Failure> {
        match flow {
            Flow::Next => Ok(Flow::Goto(resume)),
            Flow::Leave(Some(name), _) if label == Some(name) => Ok(Flow::Goto(resume)),
            Flow::Iterate(Some(name), origin) if label == Some(name) => {
                self.record_leave_failure(&origin);
                Err(raised_iterate_wrong_kind(code.symbols.name(name).as_bytes()).into())
            }
            // Not consumed: this SELECT is being "popped" by the search,
            // so its own indent becomes the new residual before the flow
            // continues outward.
            Flow::Leave(name, origin) => Ok(Flow::Leave(
                name,
                self.pop_search_frame(code, index, origin),
            )),
            Flow::Iterate(name, origin) => Ok(Flow::Iterate(
                name,
                self.pop_search_frame(code, index, origin),
            )),
            other => Ok(other),
        }
    }

    /// Resets `origin.indent` to `index`'s own `static_indent`, for a
    /// `SELECT`/`DO`/`LOOP` that owns a search frame and is being forwarded
    /// past (not matched) -- the update `LeaveOrigin`'s own doc comment
    /// describes as "restoring the indent to the value saved when that
    /// frame was pushed." Shared by `leave_select` (always calls it, since
    /// a `SELECT` always owns a frame) and `do_body_outcome` (calls it only
    /// when the `Do`/`Loop` in question owns one, i.e. skips an unlabelled
    /// `Simple` block).
    ///
    /// **The one site that adds `activation_indent` without going through
    /// `printed_indent`, and the asymmetry is deliberate.** `origin.indent`
    /// is an absolute printed indent, so it needs the activation base:
    /// measured, `do z = 1 to 1` around `interpret "do jj = 1 to 1; leave
    /// zz; end"` reports the `LEAVE` at 2 on the oracle, and this function
    /// is what decides it -- the search walks out past the fragment's own
    /// `DO`, resetting the residual to that `DO`'s lexical position, which
    /// is 0 *within the fragment* and 2 in the program. Two further shapes
    /// (the same through a `SELECT`, and through two nested `DO`s) give 2 as
    /// well. But it must **not** pick up `indent_offset`: this function's
    /// whole contract is restoring the value saved when the frame was
    /// pushed, and an escape elevation belongs to the dispatch that is
    /// currently running rather than to a frame being unwound. Task 11's own
    /// fourteen-point probe fixed that behaviour and this fix leaves it
    /// exactly as it was, because `activation_indent` is `0` in every one of
    /// those fourteen shapes.
    fn pop_search_frame(&self, code: &Code<'_>, index: usize, origin: LeaveOrigin) -> LeaveOrigin {
        LeaveOrigin {
            site: origin.site,
            indent: static_indent(&code.body.instructions, index) + self.activation_indent,
            // Untouched: this resets the *indent* the search reports at, and
            // the clause line stays the `LEAVE`/`ITERATE`'s own however many
            // frames it is forwarded past -- measured, `iterate lab` inside
            // an inner loop attributes the outer loop's re-test to the
            // `ITERATE`'s line, not to anything about the frames in between.
            clause_line: origin.clause_line,
        }
    }

    /// `target`'s own **absolute printed indent**: its lexical
    /// `static_indent`, plus the activation base it is running under, plus
    /// any escape elevation currently in force.
    ///
    /// **The one place either offset is applied, and it exists because
    /// open-coding it was a defect.** Through 4a the six sites that needed
    /// `+ self.indent_offset` each wrote it out, and one of them -- the
    /// `WHEN` scan in `Select`'s own arm -- did not.
    ///
    /// **That was a live 4a divergence, not one 4b created.** The missing
    /// addend was already wrong for a nested `SELECT` inside an escaped
    /// `OTHERWISE`, with no `INTERPRET` anywhere: measured under `trace r`,
    /// `select case 2` / `when 2 then` / `when 3 then nop` / `otherwise` /
    /// `select` / `when 1 = 1 then nop` / `end` / `end` printed the inner
    /// `WHEN` at 6 where the oracle prints 10. What 4b's Task 2 changed was
    /// only how easy it is to reach -- a plain `SELECT` inside an `INTERPRET`
    /// inside one `DO` also hits it, and that is not deep nesting.
    ///
    /// The distinction matters because the old doc bounded the consequence
    /// with "no corpus or spec example nests this deeply", and that false
    /// bound is why nobody looked. Replacing it with a narrower false bound
    /// -- "it only became live once a fragment base rode the field" -- would
    /// set the same trap for the next reader. A missing addend is not
    /// something a reader notices, so the fix is to leave nothing to notice.
    ///
    /// `static_indent` itself is untouched and stays a pure function of
    /// `(instructions, target)` -- see its own doc comment for why that
    /// matters. This adds the two pieces of running state on top of it, and
    /// is deliberately *not* where the 40-column clamp lives: that is on the
    /// `*-*` echo alone (`trace::MAX_CLAUSE_INDENT`), and clamping here would
    /// truncate every `>>>` value line too.
    fn printed_indent(&self, instructions: &[Instruction], target: usize) -> usize {
        static_indent(instructions, target) + self.activation_indent + self.indent_offset
    }

    /// Runs `code.body.instructions[start..end]` in place, one instruction at
    /// a time through `step_in_temps_frame`, and answers what happened.
    ///
    /// **Why this exists at all.** Phase 3 elides the C++'s synthetic
    /// end-of-branch markers (`ast.rs`'s own "Why there is no node for the
    /// synthetic end of a branch"), so a flat instruction list has nowhere to
    /// hang "the THEN branch just finished, skip the ELSE" or "one true WHEN
    /// ends the whole SELECT" other than on the branch instruction itself.
    /// Concretely, traced by hand against `block.rs`: `if c then A else B`
    /// gives `If.false_target == Else`'s own index, so the true path (fall
    /// through `A`, land on `Else` by `pc += 1`) and the false path (`Goto`
    /// straight to `Else`) arrive at the *identical* `(instruction, pc)`, and
    /// only one of the two arrivals is supposed to enter `B`. `SELECT`/`WHEN`
    /// has the same defect with no marker at all: a matched `WHEN`'s body,
    /// left to fall through on its own, lands on the next `WHEN` and would
    /// test it again (`select_when_bodies.rex` is written to catch exactly
    /// that). Per-instruction dispatch on `(instruction, pc)` alone cannot
    /// tell these two arrivals apart, and a per-activation block stack would
    /// resolve it trivially but does not exist yet (`Activation`'s own doc
    /// comment: Task 11's).
    ///
    /// **The fix confines itself to a range instead.** `If`/`Select` compute
    /// the winning branch's `[start, end)` directly from data already on the
    /// node (`If`'s `false_target`, `Select`'s `whens`/`false_target`/`exit`)
    /// and run exactly that range here, then return one `Flow::Goto` past
    /// the whole construct -- so the ambiguous fallthrough this function
    /// would otherwise produce never reaches the outer loop at all. Nested
    /// constructs are safe under this with no extra bookkeeping, because
    /// `block.rs` assembles an inner branch's own jump targets to close
    /// before the outer one's does, so they always fall inside (or exactly
    /// at the boundary of) whichever range encloses them.
    ///
    /// **What this owns, and what it does not.** A `Flow::Next` advances the
    /// local `pc` by one. A `Flow::Goto(target)` is only "mine" when `target`
    /// is inside `[start, end]` (`end` inclusive: a nested construct's own
    /// resume point landing exactly on my own boundary is normal completion,
    /// not an escape) -- anything else, this returns immediately and
    /// unchanged, exactly as received. That covers `Flow::Exit` today and is
    /// written to keep covering whatever Task 11 adds for `LEAVE`/`ITERATE`:
    /// `leave sel` on a labelled `SELECT` has to unwind out of a nested Rust
    /// call the same way `Flow::Exit` already does here, and a `Flow`
    /// variant this function does not recognise is deliberately the same
    /// case as one whose `Goto` target falls outside the range -- both fall
    /// to the catch-all below and propagate outward rather than being
    /// matched (and silently mishandled) by name.
    ///
    /// Reaching `end` exactly, whether by `pc += 1` or by an in-range `Goto`,
    /// is the only way this returns `Ok(Flow::Next)`. Every other exit
    /// returns the escaping `Flow` unchanged, and the caller (`If`/`Select`)
    /// must check which happened rather than assume the former.
    ///
    /// `source` is forwarded to `step_in_temps_frame` unchanged, purely so it
    /// can resolve the failing clause's own site rather than the caller's --
    /// see that function's own doc comment.
    fn run_bounded(
        &mut self,
        code: &Code<'_>,
        start: usize,
        end: usize,
        source: Option<&ProgramSource>,
    ) -> Result<Flow, Failure> {
        let mut pc = start;
        while pc < end {
            let instruction = &code.body.instructions[pc];
            let flow = self.step_in_temps_frame(code, pc, instruction, source)?;
            match flow {
                Flow::Next => pc += 1,
                Flow::Goto(target) if target >= start && target <= end => pc = target,
                other => return Ok(other),
            }
        }
        Ok(Flow::Next)
    }

    /// `DO`/`LOOP`, every kind. Resolves the whole construct -- header
    /// validation, every iteration, `LEAVE`/`ITERATE`, `WHILE`/`UNTIL` --
    /// inside this one call, exactly the discipline `If`/`Select` already
    /// hold: see `Flow::Leave`'s own doc comment for why a `Do` must never
    /// return to its caller mid-loop.
    ///
    /// **`COUNTER` and `DO WITH` both take the loud path, checked first and
    /// unconditionally.** The brief this task started from names both
    /// explicitly and asks for a decision, not a silent fallthrough:
    /// `COUNTER`'s own running-count bookkeeping is Phase-5-shaped extra
    /// state that no other of `DO`/`LOOP`'s 21 other forms needs, and
    /// `DO WITH` sends `SUPPLIER` a message, which nothing in 4a answers
    /// (no message dispatch at all yet). Checked ahead of any header
    /// evaluation, so `do counter c with index i over x` -- both keywords
    /// at once -- fails loudly without evaluating `x` either.
    fn run_loop(
        &mut self,
        code: &Code<'_>,
        index: usize,
        instruction: &Instruction,
        body: &Loop,
        source: Option<&ProgramSource>,
    ) -> Result<Flow, Failure> {
        if body.counter.is_some() || matches!(body.kind, LoopKind::With { .. }) {
            return Err(Loud::instruction(&instruction.kind).into());
        }

        let body_start = index + 1;
        let end_index = body
            .end
            .expect("an unclosed DO/LOOP is error 14.1/14.5, so a body that parsed has this set");
        let resume = end_index + 1;
        let label = body.label;

        match &body.kind {
            // A block, not a loop: exactly one pass, and `WHILE`/`UNTIL`
            // can never be present (`create_loop`'s own parser only reaches
            // `LoopKind::Simple` through the bare, at-end-of-clause `DO`
            // arm, before any conditional is even looked for). Still
            // leavable by an explicit `DO LABEL`, never by a bare `LEAVE`
            // (`do_body_outcome`'s own `is_loop: false`) -- measured, a
            // labelled simple block is leavable but an unlabelled one is
            // 28.1 on a bare `LEAVE` reaching it.
            LoopKind::Simple => {
                let flow = self.run_bounded(code, body_start, end_index, source)?;
                match self.do_body_outcome(code, index, label, false, resume, flow)? {
                    DoOutcome::Escaped(escape) => Ok(escape),
                    // Falls through to `END`, which `run_bounded`'s own
                    // `[body_start, end_index)` range never visits (the
                    // `Goto(resume)` below jumps straight past it) --
                    // unlike a repeating loop, a `Simple` block never runs
                    // this arm again, so one explicit echo here is the
                    // whole story, not a per-pass one the way
                    // `run_repeating`'s own is. Measured, this task's
                    // report: `if 1 = 1 then do / say 'x' / end` traces
                    // `end` on its own line even though the block never
                    // repeats.
                    //
                    // `Iterated` cannot arrive here and is folded in rather
                    // than made an `unreachable!`: `is_loop` is `false` for a
                    // `Simple` block, so a bare `ITERATE` is never matched
                    // (it escapes to look further out) and a *named* one that
                    // matches this block's own label is error 28.5 inside
                    // `do_body_outcome` before it can return. Folding it in
                    // means a future `LoopKind` that does reach it echoes
                    // `END` once, which is what a fall-through does, rather
                    // than aborting.
                    DoOutcome::FellThrough | DoOutcome::Iterated(_) => {
                        if self.trace_mode().all
                            && let Some((line, text)) =
                                self.clause_site(source, &code.body.instructions[end_index])
                        {
                            // A fresh computation, not `current_value_
                            // indent` -- `run_bounded`, just above, has
                            // already stepped this block's own body, so
                            // that field now holds whatever the *last*
                            // body instruction left it at, not this `DO`'s
                            // own. `+ self.indent_offset` for the same
                            // reason every other site on this page has it:
                            // consistency if this `Simple` block's own
                            // `END` is ever itself the direct landing
                            // point of an escape (untested, but cheap to
                            // keep uniform rather than silently exempt).
                            self.trace_clause(
                                line,
                                self.printed_indent(&code.body.instructions, index),
                                &text,
                            );
                        }
                        Ok(Flow::Goto(resume))
                    }
                }
            }
            LoopKind::Forever => self.run_repeating(
                code,
                index,
                instruction,
                body_start,
                end_index,
                resume,
                label,
                body.conditional.as_ref(),
                source,
                LoopState::Forever,
            ),
            LoopKind::Count(count_expr) => {
                let remaining = match count_expr {
                    Some(expr) => {
                        let value = self.eval(code, expr)?;
                        self.roots.push_temp(value);
                        let text = self.to_text(value).to_vec();
                        // `>K>   "FOR" => "2"`, once, at the `DO`'s own
                        // level -- measured, this task's own report (F1,
                        // found by review): the oracle traces a bare
                        // repeat count under the `FOR` tag, the same as
                        // an explicit `DO ... FOR n`'s own. Fires before
                        // the `whole_nonneg` check below, matching every
                        // other `>K>` site in this function (the value is
                        // traced as evaluated, not as validated).
                        // Reads `current_value_indent` rather than
                        // recomputing `static_indent(index)`: `self.eval`
                        // just above never touches that field (only
                        // `step_in_temps_frame` does, for an
                        // *instruction*, and evaluating `expr` steps none),
                        // so it still holds exactly this `DO`'s own value.
                        self.trace_keyword(self.clause_state.current_value_indent, "FOR", &text);
                        self.whole_nonneg(value)
                            .ok_or_else(|| raised_repetition_count_not_whole(&text))?
                    }
                    // Defensive, not measured: `count_loop`'s own parser
                    // (`instruction.rs`) always calls `opt_expr`, which can
                    // answer `None`, but nothing in this crate's own tests
                    // reaches `DO` with truly nothing after it and no
                    // recognised keyword either -- `create_loop`'s own
                    // `at_end()` check catches a bare `DO` first and builds
                    // `LoopKind::Simple` instead. A single pass, matching
                    // `Simple`'s own behaviour, is the least surprising
                    // answer if this is ever reached.
                    None => 1,
                };
                self.run_repeating(
                    code,
                    index,
                    instruction,
                    body_start,
                    end_index,
                    resume,
                    label,
                    body.conditional.as_ref(),
                    source,
                    LoopState::Count { remaining },
                )
            }
            LoopKind::Controlled(ctrl) => {
                // Reads `current_value_indent` rather than recomputing --
                // same reasoning as `Count`'s own `FOR`, just above.
                let indent = self.clause_state.current_value_indent;
                let state = self.setup_controlled(code, ctrl, indent)?;
                self.run_repeating(
                    code,
                    index,
                    instruction,
                    body_start,
                    end_index,
                    resume,
                    label,
                    body.conditional.as_ref(),
                    source,
                    state,
                )
            }
            LoopKind::Over {
                control,
                target,
                for_count,
            } => {
                // Deviation 1 (`phase-4-exclusions.txt`): a stem target's
                // own tail order does not reproduce the oracle's (a
                // balanced tree against our hash map), and no corpus
                // program may contain one. Detected from `target`'s own
                // *syntax*, never by evaluating it: `over a.` parses `a.`
                // through the ordinary expression grammar, which recognises
                // a bare trailing-dot token as `ExprKind::Stem` the same
                // way a plain stem read anywhere else does, so a target
                // that is a stem never needs evaluating to know it is out
                // of scope.
                //
                // **Corrected after review**: an earlier version of this
                // comment said a stem reached indirectly (`over (a.)`) is
                // "not detected here". Measured (`do_over_a_parenthesised_
                // stem_target_is_also_caught`), it *is*: a single
                // parenthesised sub-expression collapses to that
                // sub-expression's own `ExprKind` rather than being wrapped
                // in `ExprKind::List`, so `(a.)` is already
                // `ExprKind::Stem` by the time the `matches!` below runs,
                // with nothing extra needed to catch it. What genuinely
                // escapes this check is a stem reached through something
                // that does not collapse this way -- a function call
                // returning one, for instance -- and that gap is real, not
                // a mistaken claim: no test may write one either way, so
                // nothing observable depends on catching it, but the
                // previous wording overstated the gap to include a case
                // this check already closes.
                if matches!(target.kind, ExprKind::Stem(_)) {
                    return Err(Loud::instruction(&instruction.kind).into());
                }
                let value = self.eval(code, target)?;
                self.roots.push_temp(value);
                // `>K>   "OVER" => "abc"`, once, at the `DO`'s own level --
                // measured, this task's report: fires on the first pass
                // only, exactly like `TO`/`BY`/`FOR`, because `target` is
                // evaluated once at loop entry here too. Reads `current_
                // value_indent` rather than recomputing -- same reasoning
                // as `Count`'s own `FOR`.
                let over_indent = self.clause_state.current_value_indent;
                let over_text = self.to_text(value).to_vec();
                self.trace_keyword(over_indent, "OVER", &over_text);
                let remaining = match for_count {
                    Some(expr) => {
                        let count_value = self.eval(code, expr)?;
                        self.roots.push_temp(count_value);
                        let text = self.to_text(count_value).to_vec();
                        Some(
                            self.whole_nonneg(count_value)
                                .ok_or_else(|| raised_for_count_not_whole(&text))?,
                        )
                    }
                    None => None,
                };
                self.run_repeating(
                    code,
                    index,
                    instruction,
                    body_start,
                    end_index,
                    resume,
                    label,
                    body.conditional.as_ref(),
                    source,
                    LoopState::OverOnce {
                        control: *control,
                        value,
                        done: false,
                        remaining,
                    },
                )
            }
            LoopKind::With { .. } => unreachable!("DO WITH takes the loud path above"),
        }
    }

    /// The shared driver for every repeating `LoopKind` (everything but
    /// `Simple`, which never repeats and runs through `run_loop`'s own
    /// arm directly): advance-test-run-test-advance, in the order the
    /// oracle is measured to use it in (`report`'s own transcripts) --
    /// `WHILE` tested before the body, `UNTIL` after, and a `LEAVE`/
    /// `ITERATE` handled identically to falling off the bottom of the body
    /// normally, because that is what the oracle's own `ITERATE` does
    /// (measured: `do until n = 1 / n = n + 1 / if n = 1 then iterate / ...`
    /// terminates immediately rather than skipping straight to the next
    /// pass's top, because `ITERATE` jumps to the loop's own
    /// bottom-of-iteration bookkeeping, which for an `UNTIL` loop includes
    /// testing `UNTIL` right there -- see the report for the full
    /// transcript).
    ///
    /// `do_index`/`do_instruction` are the `DO`/`LOOP` instruction's own
    /// position and node, needed only for `WHILE`'s own attribution
    /// (`end_index` is `END`'s own, for `UNTIL`'s) -- neither corresponds
    /// to a flat position `static_indent` resolves on its own, so both
    /// indents are computed here rather than asked of it (`record_failure_at`'s
    /// own doc comment).
    #[allow(
        clippy::too_many_arguments,
        reason = "every parameter is load-bearing state one repeating DO/LOOP needs; splitting it into a struct is Task 13's to consider if it too needs this shape"
    )]
    fn run_repeating(
        &mut self,
        code: &Code<'_>,
        do_index: usize,
        do_instruction: &Instruction,
        body_start: usize,
        end_index: usize,
        resume: usize,
        label: Option<SymbolId>,
        conditional: Option<&LoopConditional>,
        source: Option<&ProgramSource>,
        mut state: LoopState,
    ) -> Result<Flow, Failure> {
        // The loop's own two spaces of indent, added once here rather than
        // per-check: `static_indent(&code.body.instructions, do_index)` is
        // what a control-setup failure at `do_index` itself already
        // reports (measured: `do i = 1 to 3 for 1/0` is unindented at top
        // level), and `WHILE`/`UNTIL` both report two spaces *more* than
        // that (measured: `do while 1/0` at top level is indented two).
        // Captured from `current_value_indent` once, here, rather than
        // recomputed: `step_in_temps_frame` already set it to exactly this
        // value (`indent_offset` included) for this same `DO`/`LOOP`
        // instruction, and every caller into this function reaches it
        // through nothing but `self.eval` calls in between (never another
        // instruction step), so it has not moved.
        let do_indent = self.clause_state.current_value_indent;
        let loop_indent = do_indent + 2;
        // `TRACE`'s own per-iteration re-echo (D17, this task's report,
        // "Step 6"): the oracle's `DO`/`LOOP` instruction is re-executed
        // once per pass (`DoBlock::checkControl`, read directly), so its
        // own clause -- and `END`'s -- echo again on every pass, unlike
        // every other construct in this crate, which resolves its whole
        // repetition inside one `step` call and so is stepped, and echoed,
        // exactly once (`step_in_temps_frame`'s own doc comment). `false`
        // on entry because the *first* pass's echo already happened there,
        // before `run_loop` ever called into this function.
        //
        // **`UNTIL` gets no echo here at all, only its own further down.**
        // Measured (re-verifying this task's F4 fix rather than assuming
        // the existing re-echo covered it): a multi-pass `DO UNTIL` shows
        // exactly *one* `DO`/`LOOP` re-echo per completed pass, sitting
        // between `END` and the `UNTIL` test itself, never a second one
        // here too -- `UNTIL`'s own re-entry *is* the loop's only decision
        // point for this shape (there is no separate "test `WHILE`, then
        // maybe run the body again" event to echo for, unlike every other
        // `LoopConditional`/`LoopState` shape), so echoing both here and
        // at `UNTIL`'s own site would double it. `is_until_loop` decides
        // which of the two echo sites is live for this call, never both.
        let is_until_loop = matches!(conditional, Some(cond) if cond.until);
        let mut first_pass = true;
        // Which clause the loop header's own evaluation belongs to on this
        // pass -- `HeaderClause`'s own doc comment has the oracle mechanism
        // and the three measured transcripts.
        let mut header_clause = HeaderClause::Do;
        let end_line = self
            .clause_line(source, &code.body.instructions[end_index])
            .unwrap_or(0);

        loop {
            if !first_pass
                && !is_until_loop
                && self.trace_mode().all
                && let Some((line, text)) = self.clause_site(source, do_instruction)
            {
                self.trace_clause(line, do_indent, &text);
            }
            first_pass = false;

            // **The `DO` clause, entered and ended like any other** (fix
            // round 3). The header's control expressions and a `WHILE` test
            // are Rexx clauses in their own right: measured, `do while zn <
            // sub()` reports `SIGL` 4 -- the `DO` clause's own line -- for
            // the first test, and delivers a `CALL ON` handler queued by
            // `sub()` right there rather than after the whole loop.
            //
            // On iterations after the first the re-test belongs to whichever
            // clause transferred control back here -- `END` on a
            // fall-through, the `ITERATE` itself on an `ITERATE`. See
            // `HeaderClause`.
            let do_line = self
                .clause_line(source, do_instruction)
                .unwrap_or_else(|| self.clause_state.line());
            let header_line = match header_clause {
                HeaderClause::Do => do_line,
                HeaderClause::End => end_line,
                HeaderClause::Iterate(line) => line,
            };
            let header = self.in_clause(code, header_line, |it| {
                if !it.loop_advance(code, &mut state, do_indent, loop_indent)? {
                    return Ok(HeaderOutcome::Stop);
                }
                if let Some(cond) = conditional
                    && !cond.until
                {
                    // Overrides `step_in_temps_frame`'s own setting of
                    // `current_value_indent` (to `do_indent`, from stepping
                    // the `DO`/`LOOP` instruction itself) -- `WHILE`'s own
                    // condition is evaluated here, inside that same `step`
                    // call, never through a `step_in_temps_frame` of its own.
                    it.clause_state.current_value_indent = loop_indent;
                    match it.eval_condition(
                        code,
                        &cond.condition,
                        ConditionTrace::Keyword(loop_indent, "WHILE"),
                        raised_while_not_logical,
                    ) {
                        Ok(true) => {}
                        Ok(false) => return Ok(HeaderOutcome::Stop),
                        Err(failure) => {
                            it.record_failure_at(source, do_instruction, loop_indent);
                            return Err(failure);
                        }
                    }
                }
                Ok(HeaderOutcome::Continue)
            })?;
            match header {
                ClauseOutcome::Ended(exit) => return Ok(Flow::Exit(exit.value())),
                ClauseOutcome::Ran(Err(failure)) => return Err(failure),
                ClauseOutcome::Ran(Ok(HeaderOutcome::Stop)) => return Ok(Flow::Goto(resume)),
                ClauseOutcome::Ran(Ok(HeaderOutcome::Continue)) => {}
            }

            let flow = self.run_bounded(code, body_start, end_index, source)?;
            let end_instruction = &code.body.instructions[end_index];
            match self.do_body_outcome(code, do_index, label, true, resume, flow)? {
                DoOutcome::Escaped(escape) => return Ok(escape),
                // **`END` is not reached at all when an `ITERATE` ended the
                // pass**, so it neither echoes nor owns the re-test. Measured
                // (fix round 4, found while measuring NEW-1's `SIGL`
                // divergence): under `trace r`, `do while zn < 2 / zn = zn +
                // 1 / iterate / end` echoes `iterate` and then the `do`
                // clause again, with no `end` line between them, where the
                // same loop without the `ITERATE` does echo `end`. The
                // oracle's reason is structural: `END`'s own `execute` is
                // what calls `reExecute` on a fall-through, and
                // `RexxActivation::iterate` is what calls it for an
                // `ITERATE` -- `END` is jumped straight over.
                DoOutcome::Iterated(line) => header_clause = HeaderClause::Iterate(line),
                // Reached only when the body fell off its end. **Not**
                // reached on a matched `LEAVE`, which returns above instead
                // -- measured, this task's report (`DO FOREVER` with a
                // `LEAVE` on the second pass): `END` never echoes for that
                // final pass, only for a pass that genuinely falls through
                // to it.
                DoOutcome::FellThrough => {
                    header_clause = HeaderClause::End;
                    if self.trace_mode().all
                        && let Some((line, text)) = self.clause_site(source, end_instruction)
                    {
                        self.trace_clause(line, do_indent, &text);
                    }
                }
            }

            if let Some(cond) = conditional
                && cond.until
            {
                // **F4's own sibling, found while re-verifying this task's
                // review fixes rather than assumed clean**: `UNTIL`'s own
                // check needs a *second*, unconditional re-echo of the
                // `DO`/`LOOP` clause here, not only the top-of-loop one
                // above. Measured: `do until n = 1 / n = n + 1 / end`
                // re-echoes `do until ...` a second time, after `END`,
                // before testing `UNTIL` at all -- even on the very first
                // test, which runs after the body's only pass and before
                // the top-of-loop re-echo (gated on `!first_pass`) would
                // ever fire again. The oracle's own `DO`/`LOOP` instruction
                // is re-entered to make *this* decision too, exactly like
                // it is to test `WHILE` or advance a `Controlled` loop
                // (`checkControl`, read directly, this task's report) --
                // `UNTIL`'s decision point is not the same event as the
                // top-of-loop one, so it needs its own echo unconditionally
                // rather than sharing `first_pass`'s gate.
                if self.trace_mode().all
                    && let Some((line, text)) = self.clause_site(source, do_instruction)
                {
                    self.trace_clause(line, do_indent, &text);
                }
                // Same override as `WHILE`'s own, above -- the re-echoed
                // `END` clause just before this point left
                // `current_value_indent` untouched (its own `trace_clause`
                // call does not set it), so without this `UNTIL`'s
                // intermediates would otherwise still read `do_indent`.
                self.clause_state.current_value_indent = loop_indent;
                // `UNTIL`'s test belongs to the same clause the *next*
                // top-of-loop re-test does, and for the same reason: in the
                // oracle they are one event, `reExecute` called by whichever
                // instruction transferred control back to the loop. Measured
                // with an `ITERATE` in the body, which is what tells the two
                // candidates apart -- `do until zs() >= 2` with `if zn = 1
                // then iterate` on line 4 reports `4` for the first test and
                // `6` (the `END` line) for the second.
                let until_line = match header_clause {
                    HeaderClause::Do => do_line,
                    HeaderClause::End => end_line,
                    HeaderClause::Iterate(line) => line,
                };
                // **This clause's boundary is currently unobservable, and it
                // is here because it cannot be separated from the line.**
                // Round 3 shipped the line and the boundary as two calls, and
                // re-review 3 measured that replacing the boundary half alone
                // changed nothing on any of its 38 probes: between this test
                // and the next top-of-loop test no user clause runs and
                // nothing re-sets the clause line, so whichever of the two
                // boundaries fires first delivers at the same line. With
                // `in_clause` there is no half to remove -- the mutation
                // "keep the line, drop the boundary" is not expressible, and
                // dropping both is what `a_while_retest_belongs_to_the_do_
                // clause_then_to_the_end_clause` and `a_loop_retest_after_
                // an_iterate_belongs_to_the_iterate_clause` fail on.
                let tested = self.in_clause(code, until_line, |it| {
                    it.eval_condition(
                        code,
                        &cond.condition,
                        ConditionTrace::Keyword(loop_indent, "UNTIL"),
                        raised_until_not_logical,
                    )
                })?;
                match tested {
                    ClauseOutcome::Ended(exit) => return Ok(Flow::Exit(exit.value())),
                    ClauseOutcome::Ran(Ok(true)) => return Ok(Flow::Goto(resume)),
                    ClauseOutcome::Ran(Ok(false)) => {}
                    ClauseOutcome::Ran(Err(failure)) => {
                        self.record_failure_at(source, end_instruction, loop_indent);
                        return Err(failure);
                    }
                }
            }
            // Nothing happens at the bottom of a pass any more. A
            // `Controlled` loop's `BY` increment used to, as `loop_step`;
            // Task 9 moved it into `loop_advance`, where the oracle does it,
            // because the two `>>>` lines the oracle traces straddle that
            // addition and the value on the near side of it is gone by the
            // time the next `loop_advance` runs. `loop_advance`'s own doc
            // comment has the citation and why nothing else moved with it.
        }
    }

    /// What one repeating `Do`/`Loop`'s own body just produced, translated
    /// into what `run_repeating`/`run_loop`'s own `Simple` arm does next.
    ///
    /// `Ok(DoOutcome::FellThrough)`/`Ok(DoOutcome::Iterated(_))`: proceed to
    /// whatever bottom-of-iteration test/advance comes next. The two used to
    /// be one answer, `Ok(None)`, on the argument that they are the identical
    /// next *step* -- true of the control flow and false of the clause
    /// attribution, which is fix round 4's NEW-1: the oracle re-enters a loop
    /// from whichever instruction transferred control back to it, so a pass
    /// that ended in `ITERATE` gives the following re-test the `ITERATE`'s
    /// own clause where a pass that fell through gives it `END`'s.
    /// `Ok(DoOutcome::Escaped(f))`: stop, and `f` is this construct's own
    /// final answer -- either `Goto(resume)` (a consumed `LEAVE`) or an
    /// unconsumed `Flow` to propagate outward unchanged (`Exit`, a `Goto`
    /// that escaped `run_bounded`'s own range, or a `LEAVE`/`ITERATE` naming
    /// something else). `Err`: a named `ITERATE` matched `label`, but
    /// `is_loop` is `false` -- 28.5, `ITERATE` never accepts a labelled
    /// block, only a loop (measured).
    ///
    /// `is_loop` is `false` only for `LoopKind::Simple` (`run_loop`'s own
    /// `Simple` arm passes it); every `LoopState` variant `run_repeating`
    /// drives is a real, repetitive loop and passes `true`.
    ///
    /// **Whether this construct "owns a search frame" (`LeaveOrigin`'s own
    /// doc comment has the rule and the oracle transcripts) is `is_loop ||
    /// label.is_some()`, not `is_loop` alone.** A labelled `Simple` block
    /// does not repeat, but it is still leavable by name and still resets
    /// the search's own residual indent when a `LEAVE`/`ITERATE` naming
    /// something else is forwarded past it -- only an *unlabelled* `Simple`
    /// block is fully transparent, touching nothing as a `LEAVE`/`ITERATE`
    /// passes through.
    ///
    /// `do_index` is this `Do`/`Loop` instruction's own position, needed
    /// only to compute that reset (`pop_search_frame`); it is *not* used
    /// for clause attribution here, since nothing in this function raises
    /// against this instruction's own clause.
    fn do_body_outcome(
        &mut self,
        code: &Code<'_>,
        do_index: usize,
        label: Option<SymbolId>,
        is_loop: bool,
        resume: usize,
        flow: Flow,
    ) -> Result<DoOutcome, Failure> {
        let owns_frame = is_loop || label.is_some();
        match flow {
            Flow::Next => Ok(DoOutcome::FellThrough),
            Flow::Leave(name, origin) => {
                let matched = match name {
                    None => is_loop,
                    Some(n) => label == Some(n),
                };
                if matched {
                    Ok(DoOutcome::Escaped(Flow::Goto(resume)))
                } else if owns_frame {
                    Ok(DoOutcome::Escaped(Flow::Leave(
                        name,
                        self.pop_search_frame(code, do_index, origin),
                    )))
                } else {
                    Ok(DoOutcome::Escaped(Flow::Leave(name, origin)))
                }
            }
            Flow::Iterate(name, origin) => {
                let matched = match name {
                    None => is_loop,
                    Some(n) => label == Some(n),
                };
                if !matched {
                    return Ok(DoOutcome::Escaped(Flow::Iterate(
                        name,
                        if owns_frame {
                            self.pop_search_frame(code, do_index, origin)
                        } else {
                            origin
                        },
                    )));
                }
                if !is_loop {
                    let name = name.expect(
                        "is_loop is false and matched is true only through the named branch above",
                    );
                    self.record_leave_failure(&origin);
                    return Err(
                        raised_iterate_wrong_kind(code.symbols.name(name).as_bytes()).into(),
                    );
                }
                Ok(DoOutcome::Iterated(origin.clause_line))
            }
            other => Ok(DoOutcome::Escaped(other)),
        }
    }

    /// Decides whether one more candidate iteration of `state` should run,
    /// consuming whatever budget (`FOR`, a bare count) applies and binding
    /// a control variable **before** the decision is answered, not after --
    /// measured, `do i = 5 to 3 / say never / end / say i` prints `5`: the
    /// control variable is bound to its own value even for a loop that ends
    /// up running zero iterations.
    ///
    /// **Also where a `Controlled` loop's `BY` increment happens**, moved
    /// here from a separate bottom-of-pass `loop_step` at Task 9, because
    /// the oracle traces the value on *both* sides of that addition and a
    /// split that had already added it could not name the pre-increment
    /// value at all (the KNOWN GAP this closes, `DoBlock::checkControl`,
    /// `DoBlock.cpp:182`-`205`, read directly). Nothing else moved with it:
    /// no instruction runs between the old site and this one -- the only
    /// events in between are `END`'s or an `ITERATE`'s own transfer -- so
    /// the settings the addition runs under are the same ones it ran under
    /// before.
    ///
    /// The two indents are the **same clause's**, and which one a line takes
    /// is measured rather than derived. `do_indent` is the `DO`/`LOOP`
    /// clause's own printed indent and `loop_indent` is two further in:
    ///
    /// * A `Controlled` loop's **first** control assignment is the oracle's
    ///   own loop *setup*, before the block is pushed, so it prints at
    ///   `do_indent` -- measured, `trace i` / `do ii = 1 to 2` shows
    ///   `>=>   II <= "1"` at the same indent as `>K>   "TO" => "2"`.
    /// * Everything on a re-tested pass prints at `loop_indent`: `>V>`,
    ///   `>>>`, `>>>`, `>=>`, all four two spaces further in than the
    ///   `DO`'s own echo, in the same column as the body's clauses.
    /// * A `DO OVER`'s assignment prints at `loop_indent` even though it
    ///   only ever happens once (`checkOver` runs with the block already
    ///   pushed, unlike a controlled loop's setup) -- measured, `do qq over
    ///   'ab'` shows `>=>     QQ <= "ab"` two in from its own `>K>`.
    fn loop_advance(
        &mut self,
        code: &Code<'_>,
        state: &mut LoopState,
        do_indent: usize,
        loop_indent: usize,
    ) -> Result<bool, Failure> {
        match state {
            LoopState::Forever => Ok(true),
            LoopState::Count { remaining } => {
                if *remaining == 0 {
                    return Ok(false);
                }
                *remaining -= 1;
                Ok(true)
            }
            LoopState::OverOnce {
                control,
                value,
                done,
                remaining,
            } => {
                if *done {
                    return Ok(false);
                }
                if let Some(r) = remaining {
                    if *r == 0 {
                        *done = true;
                        return Ok(false);
                    }
                    *r -= 1;
                }
                *done = true;
                self.bind_control(code, *control, loop_indent, *value);
                Ok(true)
            }
            LoopState::Controlled {
                control,
                current,
                to,
                by,
                for_remaining,
                stepped,
            } => {
                // **The re-tested pass's own four lines** (Task 9, closing
                // the KNOWN GAP this arm used to disclose).
                // `DoBlock::checkControl` (`DoBlock.cpp:182`-`205`, read
                // directly) is called as `checkControl(context, stack,
                // !first)` (`ControlledDoInstruction.cpp:162`), and its
                // `increment` arm does four traceable things in this order:
                // read the control variable (`control->evaluate`, which
                // traces `>V>`), `traceResult` that value, add `BY`,
                // `traceResult` the sum, then `control->assign` (`>=>`).
                // The `!first` is exactly `stepped` here. Measured, `trace
                // i` / `do ii = 1 to 2`'s own second pass:
                //
                // ```text
                //   >V>     II => "1"
                //   >>>     "1"
                //   >>>     "2"
                //   >=>     II <= "2"
                // ```
                //
                // and the same program under `trace r` shows only the two
                // `>>>` lines, which is the gating: `>V>`/`>=>` are
                // `intermediates`, both `>>>` are `results`.
                //
                // **The pair is emitted before either termination test**, on
                // the failing pass as well -- measured, `do ii = 1 to 3`
                // traces `>>>     "3"` then `>>>     "4"` on the pass that
                // ends the loop, so these are not "the values of an
                // iteration that ran".
                //
                // **Bound before the decision, not after** -- measured
                // against the oracle, `do i = 5 to 3 / say never / end /
                // say i` prints `5`: the control variable takes its own
                // header value even for a loop that goes on to run zero
                // iterations, for *either* reason a candidate iteration can
                // fail below (an exhausted `FOR` budget or the `TO` bound).
                let digits = self.activation().settings.digits();
                let fuzz = self.activation().settings.fuzz();
                let form = self.activation().settings.form();
                let re_tested = std::mem::replace(stepped, true);
                if re_tested {
                    let previous =
                        self.number(current.clone(), crate::eval::saturate_digits(digits), form);
                    self.roots.push_temp(previous);
                    let rendered = self.to_text(previous).to_vec();
                    let name = code.symbols.name(*control).as_bytes().to_vec();
                    self.trace_variable(loop_indent, &name, &rendered);
                    self.trace_result(loop_indent, &rendered);
                    *current = current.add(by, digits).map_err(Raised::from)?;
                }
                // The first pass takes the value the header already computed,
                // unincremented and with no line of its own beyond the `>=>`
                // below -- `checkControl`'s own `else` arm reads it with
                // `getValue`, whose comment says why: the initial assignment
                // was already traced during setup, and tracing here too
                // "prevents getting an extra add looking item traced".
                let value =
                    self.number(current.clone(), crate::eval::saturate_digits(digits), form);
                let bind_indent = if re_tested { loop_indent } else { do_indent };
                if re_tested {
                    let rendered = self.to_text(value).to_vec();
                    self.trace_result(loop_indent, &rendered);
                }
                self.bind_control(code, *control, bind_indent, value);

                if let Some(r) = for_remaining
                    && *r == 0
                {
                    return Ok(false);
                }
                if let Some(to) = to {
                    let by_negative =
                        numeric_less(by, &Number::zero(), digits, fuzz).map_err(Raised::from)?;
                    let within = if by_negative {
                        !numeric_less(current, to, digits, fuzz).map_err(Raised::from)?
                    } else {
                        !numeric_less(to, current, digits, fuzz).map_err(Raised::from)?
                    };
                    if !within {
                        return Ok(false);
                    }
                }
                if let Some(r) = for_remaining {
                    *r -= 1;
                }
                Ok(true)
            }
        }
    }

    /// Evaluates a `Controlled` loop's header: `initial` first, then
    /// whichever of `TO`/`BY`/`FOR` were written, in the order they were
    /// written (`ctrl.order`, recorded because the expressions can have
    /// side effects) -- never a fixed `TO`-then-`BY`-then-`FOR` order.
    ///
    /// `initial`/`to`/`by` need only be *numeric* (41.1 if not, via
    /// `arith_operand` -- the same check ordinary arithmetic already
    /// makes), never *whole*: measured, `do i = 1.5 to 3` is legal and
    /// steps by fractional values. `by` defaults to `1` when absent.
    /// `for_count` is the one exception, checked against `whole_nonneg`
    /// (26.3) exactly like a bare `DO`'s own repeat count is (26.2).
    /// `indent`: the `DO`/`LOOP` instruction's own `static_indent`, for
    /// `>K>`'s own `TO`/`BY`/`FOR` lines -- **not** `loop_indent`
    /// (`+2`, `WHILE`/`UNTIL`'s own level): measured, `>K>   "TO" => "2"`
    /// sits at the same indent as `do i = 1 to 2` itself, because these
    /// header expressions are evaluated once at loop entry, before the
    /// body's own frame exists at all (`control_setup_expressions_are_
    /// unindented_unlike_the_loop_body_they_precede`, this file's own test
    /// from Task 11, makes the identical point about a *raise* at this
    /// same point).
    fn setup_controlled(
        &mut self,
        code: &Code<'_>,
        ctrl: &Controlled,
        indent: usize,
    ) -> Result<LoopState, Failure> {
        // Read once, not once per header component: nothing between here
        // and the last `round_via_unary_plus` call below executes an
        // instruction (only expression evaluation happens in a controlled
        // loop's own header), and `NUMERIC DIGITS` only ever changes by
        // running one, so one read stands in correctly for "current digits
        // at loop entry," which is the oracle's own rule (`round_via_unary_
        // plus`'s own doc comment has the citation).
        let entry_digits = self.activation().settings.digits();
        let initial_value = self.eval(code, &ctrl.initial)?;
        self.roots.push_temp(initial_value);
        let current = self.arith_operand(initial_value)?;
        let current = round_via_unary_plus(&current, entry_digits).map_err(Raised::from)?;

        let mut to = None;
        let mut by = None;
        let mut for_remaining = None;
        for entry in &ctrl.order {
            match entry {
                ControlExpr::To => {
                    let expr = ctrl
                        .to
                        .as_ref()
                        .expect("ctrl.order names To only when ctrl.to is Some");
                    let value = self.eval(code, expr)?;
                    self.roots.push_temp(value);
                    let text = self.to_text(value).to_vec();
                    self.trace_keyword(indent, "TO", &text);
                    let bound = self.arith_operand(value)?;
                    to = Some(round_via_unary_plus(&bound, entry_digits).map_err(Raised::from)?);
                }
                ControlExpr::By => {
                    let expr = ctrl
                        .by
                        .as_ref()
                        .expect("ctrl.order names By only when ctrl.by is Some");
                    let value = self.eval(code, expr)?;
                    self.roots.push_temp(value);
                    let text = self.to_text(value).to_vec();
                    self.trace_keyword(indent, "BY", &text);
                    let step = self.arith_operand(value)?;
                    by = Some(round_via_unary_plus(&step, entry_digits).map_err(Raised::from)?);
                }
                ControlExpr::For => {
                    let expr = ctrl
                        .for_count
                        .as_ref()
                        .expect("ctrl.order names For only when ctrl.for_count is Some");
                    let value = self.eval(code, expr)?;
                    self.roots.push_temp(value);
                    let text = self.to_text(value).to_vec();
                    self.trace_keyword(indent, "FOR", &text);
                    for_remaining = Some(
                        self.whole_nonneg(value)
                            .ok_or_else(|| raised_for_count_not_whole(&text))?,
                    );
                }
            }
        }
        // No `round_via_unary_plus` needed on the default: a bare literal
        // `1` is already whole at any width, so rounding it at `entry_
        // digits` could only ever answer `1` again.
        let by = match by {
            Some(by) => by,
            None => Number::parse("1").expect("the literal 1 always parses"),
        };
        Ok(LoopState::Controlled {
            control: ctrl.control,
            current,
            to,
            by,
            for_remaining,
            stepped: false,
        })
    }

    /// Writes `value` into `control`'s own slot -- the same read-the-name,
    /// resolve-a-slot, write path `Assignment`'s `Variable` target already
    /// uses (`step`'s own `Assignment` arm), reused rather than duplicated.
    ///
    /// **Traces its own `>=>`, at `indent`** (Task 9). Every write to a
    /// control variable is an assignment to the oracle and traces like one:
    /// `control->assign(context, result)` in both `DoBlock::checkOver`
    /// (`DoBlock.cpp:165`) and `DoBlock::checkControl` (`:197`), and again
    /// in a controlled loop's own setup. `indent` is the caller's, not this
    /// function's to derive, because the same write is traced at two
    /// different indents depending on which of those three events it is --
    /// `loop_advance`'s own arms have the measured rule.
    ///
    /// **A compound control variable (`do aa.1 = 1 to 2`) is already stored
    /// wrongly here** -- `rexx_parse::Controlled::control` is a bare
    /// `SymbolId`, so `slot_of` makes a simple variable literally named
    /// `AA.1` instead of resolving the compound -- and the `>C>` line the
    /// oracle traces before each of these `>=>`s is missing for the same
    /// reason. Measured and recorded as a KNOWN GAP in
    /// `phase-4-exclusions.txt`; not introduced by the tracing added here,
    /// and not fixable inside this crate alone.
    fn bind_control(&mut self, code: &Code<'_>, control: SymbolId, indent: usize, value: ObjRef) {
        let name = code.symbols.name(control).as_bytes();
        let slot = self.slot_of(name);
        let frame = self.activation().frame;
        self.roots.set_slot(frame, slot, value);
        if self.tracing_intermediates() {
            let name = code.symbols.name(control).as_bytes().to_vec();
            let rendered = self.to_text(value).to_vec();
            self.trace_assignment(indent, &name, &rendered);
        }
    }

    /// Validates `value` as "zero or a positive whole number" -- the rule a
    /// bare `DO`'s own repeat count and a `FOR` expression share (26.2/26.3
    /// respectively; the caller supplies which raiser applies, since that
    /// is the only way the two differ), and answers it as a `u64`, or
    /// `None` if it fails either check.
    ///
    /// **Corrected after the branch review's F3 (Important, a silently
    /// wrong answer).** This used to convert under `rexx_num::
    /// ARGUMENT_DIGITS` (18, `Numerics::ARGUMENT_DIGITS`'s own width), on
    /// the reasoning "a loop bound is no more digits-limited than `EXIT`'s
    /// own result is" -- wrong by measurement, not merely stale: `EXIT`
    /// genuinely does convert under `ARGUMENT_DIGITS` (`lib.rs`'s
    /// `exit_code_for`, unaffected by this fix, `exit 12345` under `digits
    /// 3` matches the oracle at rc 57 on both sides), but a loop bound does
    /// not. The oracle's own `ForLoop::setup` (`DoBlockComponents.cpp`
    /// ~80-100, verified by containment) rounds under the *current*
    /// `NUMERIC DIGITS` (`requestNumber(count, number_digits())`) before
    /// asking whether the result is whole -- the same rule `TRACE`'s own
    /// skip count uses (`Number::whole_value`'s own doc comment states the
    /// contrast between the two rules directly), not `EXIT`'s. Measured:
    /// `numeric digits 3; do 12345; end` is error 26.2, rc 230 on the
    /// oracle; this crate ran clean, rc 0, before this fix. `do i = 1 to
    /// 99999 for 12345` under `digits 3` is 26.3 on the oracle, same gap.
    fn whole_nonneg(&mut self, value: ObjRef) -> Option<u64> {
        let number = self.to_number(value).ok()?;
        let digits = usize::try_from(self.activation().settings.digits()).ok()?;
        let whole = number.whole_value(digits)?;
        u64::try_from(whole).ok()
    }

    /// Evaluates `condition` and answers whether it holds, for `IF`/`WHEN`.
    ///
    /// **A comma list checks itself, but a single expression does not, and
    /// this is the one place that gap gets closed.** `ExprKind::Logical` (a
    /// comma list) is evaluated through `eval`'s own dispatch to
    /// `eval_logical_list` exactly like any other expression, which already
    /// validates every element is exactly `0`/`1` and raises 34.6 on the
    /// first that is not -- re-checking its result here would misreport
    /// that failure as 34.1/34.2. A single, non-list expression never
    /// passes through `eval_logical_list` at all (there is no list to
    /// iterate), so nothing has checked it yet. `raise` is the
    /// keyword-specific raiser for exactly that case (34.1 `IF`, 34.2
    /// `WHEN`) -- measured across both, `if 'x', 1 then` is 34.6 (a list,
    /// regardless of which element failed) while `if 'x' then` is 34.1 (not
    /// a list at all).
    fn eval_condition(
        &mut self,
        code: &Code<'_>,
        condition: &Expr,
        trace: ConditionTrace<'_>,
        raise: fn(&[u8]) -> Raised,
    ) -> Result<bool, Failure> {
        let value = self.eval(code, condition)?;
        self.roots.push_temp(value);
        let text = self.to_text(value).to_vec();
        match trace {
            // `IF`/plain `WHEN`'s own `>>>` (`IfInstruction.cpp:140`, and
            // `select` / `when 1 = 1 then` measured to show the identical
            // shape) -- measured, `WHILE`/`UNTIL` never get this (this
            // task's report: `trace r` over a `DO WHILE` shows only `>K>
            // "WHILE" => ...`, no bare `>>>` alongside it), which is why
            // this is a variant a caller picks rather than something
            // `eval_condition` decides on its own. `SELECT CASE`'s own
            // `WHEN`/`WhenCase` comparison never reaches this function at
            // all -- see `test_case_when`'s own trace calls instead.
            ConditionTrace::Result(indent) => self.trace_result(indent, &text),
            // `WHILE`/`UNTIL`'s own `>K>` (`DoBlockComponents.cpp`'s
            // `traceKeywordResult(WHILE, ...)`/`(UNTIL, ...)`) -- re-fires
            // every pass because the oracle re-evaluates the condition every
            // pass too, which `run_repeating`'s own call site already does
            // without any change from this task.
            ConditionTrace::Keyword(indent, keyword) => {
                self.trace_keyword(indent, keyword, &text);
            }
        }
        if matches!(condition.kind, ExprKind::Logical(_)) {
            // `eval_logical_list` already validated every element and
            // answers exactly `b"0"`/`b"1"` (its own doc comment), so this
            // is a plain readback rather than a second check.
            Ok(text == b"1")
        } else {
            logical_value(&text).ok_or_else(|| raise(&text).into())
        }
    }

    /// Whether any of a `WHEN CASE`'s `values` compares `==` (byte-for-byte,
    /// no padding, no numeric awareness) equal to the `SELECT CASE`'s own
    /// `case_text`, matching on the first that does (an OR of `==`, the
    /// opposite of a plain `WHEN`'s comma list, which is an AND checked for
    /// `0`/`1` -- `ast.rs`'s own doc comment on `WhenCase`).
    ///
    /// **Reasoned rather than routed through `eval_compare`'s own
    /// `Operator::StrictEqual`, and this is why.** The design's own
    /// "Expression evaluation" section states the strict family's rule in
    /// full: "there is no padding and the shorter string is less" -- for an
    /// *ordering* comparison. Equality has no "less" to fall back on, so
    /// under that same rule two strict operands are equal if and only if
    /// they are the same length and every byte matches, which is exactly
    /// `==` on the two `Vec<u8>`s below and needs no numeric awareness or
    /// `rexx-num` call to compute. Measured, matching D15's own example:
    /// `select case '007'` does not match `when 7`, because `"007"` and
    /// `"7"` are not byte-identical. Calling `eval_compare` would work too,
    /// but needs a second `eval.rs` visibility bump beyond the one already
    /// asked for and approved for `logical_value`. This way needs none.
    /// `indent` traces two `>>>` lines per value tested, up to and including
    /// whichever one matches (`WhenCaseInstruction.cpp:154`/`158`:
    /// `traceResult(compareValue)` then `traceResult(result)`, "result"
    /// being the comparison's own `0`/`1` outcome, not a second copy of the
    /// value) -- measured, `select case 1 + 1 / when 2 then ...`:
    /// `>>>   "2"` (the one `values` entry evaluated) then `>>>   "1"`
    /// (matched). Stops at the first match, mirroring the early `return
    /// Ok(true)` below -- an oracle transcript with more than one `values`
    /// entry and no match on the first would need its own probe to confirm
    /// every untested entry gets the same pair, which this task did not run.
    fn test_case_when(
        &mut self,
        code: &Code<'_>,
        values: &[Expr],
        case_text: &[u8],
        indent: usize,
    ) -> Result<bool, Failure> {
        for value in values {
            let value = self.eval(code, value)?;
            self.roots.push_temp(value);
            let text = self.to_text(value).to_vec();
            self.trace_result(indent, &text);
            let matched = text == case_text;
            self.trace_result(indent, if matched { b"1" } else { b"0" });
            if matched {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// If `target` names an `Else` instruction, its own `then_exit`
    /// (defaulting to the end of the body when `None`, "the end of this
    /// body" per `ast.rs`'s own doc comment). Otherwise `target` unchanged.
    ///
    /// Shared by both of `If`'s own arms: the true path calls this to learn
    /// where to resume once its bounded branch finishes, and the doc comment
    /// on the `If` arm is where the false path's own reasoning for *not*
    /// needing this lives.
    fn skip_else(&self, code: &Code<'_>, target: usize) -> usize {
        match code.body.instructions.get(target).map(|i| &i.kind) {
            Some(InstructionKind::Else { then_exit }) => {
                then_exit.unwrap_or(code.body.instructions.len())
            }
            _ => target,
        }
    }

    /// Parses `text` as an `INTERPRET` fragment and runs it **inside the
    /// current activation**.
    ///
    /// This is the case that stresses the lifetime, and the three things it
    /// proves are:
    ///
    /// * The fragment's `Rc` is a **local** that outlives the nested loop, the
    ///   same shape `run_activation` uses, one level down. The enclosing
    ///   loop's own local `Rc<Program>` is untouched and still anchors the
    ///   instruction that is mid-execution.
    /// * The nested loop's program counter is a **local `usize`**, not the
    ///   activation's, because the activation's is sitting on the `INTERPRET`
    ///   instruction and has to still be there afterwards. A `LEAVE`/
    ///   `ITERATE` naming an outer loop could not target anything inside a
    ///   fragment for the measured reason that a fragment can never have a
    ///   label at all: one inside `INTERPRET` text is error 47.1 (Task 1), so
    ///   `body.labels` is always empty. `IF`/`SELECT` need no label and can
    ///   still appear and jump *inside* a fragment, which is why this reuses
    ///   `run_bounded` (Task 10) rather than the hand-rolled loop this
    ///   function used to have: every jump such a construct computes stays
    ///   within `[0, code.body.instructions.len())` by construction
    ///   (`resolve_targets` clamps everything else to `None`, and `None`
    ///   defaults to the body's own length), so `run_bounded` always "owns"
    ///   it and never mistakes it for an escape.
    /// * No frame is pushed. The fragment's assignments land in the enclosing
    ///   frame's slots, which is what `fragment_plan` resolves them against,
    ///   and it is why `RootSet::grow_slots`'s top-frame assertion holds here.
    ///
    /// That bullet covers only the *inward* direction -- a label inside the
    /// fragment cannot be targeted, because there are none.
    ///
    /// # The outward direction, measured (4b Task 1)
    ///
    /// **A `LEAVE`/`ITERATE` search never crosses this boundary.** The
    /// fragment's own body is where the search ends: one that reaches the
    /// end of `run_bounded` below is the exhausted search, and raises
    /// 28.1/28.2/28.3/28.4 here, exactly as `run_activation` does when the
    /// same `Flow` reaches the top of the *program*. Measured on the oracle,
    /// every one of the four families and both block shapes -- the report
    /// has the full transcripts, and this is the summary:
    ///
    /// | fragment text | enclosing construct | oracle |
    /// |---|---|---|
    /// | `leave outer` | `do label outer while 1` | 28.3, rc 228 |
    /// | `leave idx` | `do idx = 1 to 3` | 28.3, rc 228 |
    /// | `leave` | `do kk = 1 to 3` | 28.1, rc 228 |
    /// | `iterate` | `do kk = 1 to 3` | 28.2, rc 228 |
    /// | `iterate outer` | `do label outer idx = 1 to 3` | 28.4, rc 228 |
    /// | `leave choose` | `select label choose` | 28.3, rc 228 |
    ///
    /// So an enclosing loop is invisible from inside the text, including to
    /// a **bare** `LEAVE` -- which is the case worth pointing at, because
    /// "the fragment runs inside the enclosing activation" would predict the
    /// opposite, and until this was measured that is what this function did
    /// (it forwarded a bare `Leave` outward, and the enclosing `DO` consumed
    /// it). A loop written *inside* the fragment is unaffected and still
    /// works normally: its own `run_loop` consumes the `Flow` before it ever
    /// reaches this point (measured: `interpret "do jj = 1 to 5; ...; if jj
    /// = 2 then leave; end"` prints two lines and exits 0).
    ///
    /// This also settles F-EX2, the branch review's finding that the
    /// `SymbolId` in a named `Flow::Leave`/`Iterate` is interned in the
    /// fragment's own fresh `SymbolTable` (`parse_interpret`'s doc) and is
    /// meaningless to every consumer above this function, all of which
    /// resolve against the *program's* table -- silently, since nothing
    /// about a mismatched id looks wrong at the type level. That fix was a
    /// loud refusal, placed here because here is where the id can still be
    /// named correctly. The refusal is gone and the placement is why: the
    /// name is resolved against `fragment.symbols` at the same point, and
    /// turned into the condition the oracle actually raises. Nothing past
    /// this function ever sees the id.
    ///
    /// **The report names both clauses, innermost first, each carrying the
    /// enclosing `INTERPRET`'s line number** -- measured, `do outer = 1 to 1`
    /// around `interpret "leave outer"` on line 2:
    ///
    /// ```text
    ///      2 *-*   leave outer
    ///      2 *-*   interpret "leave outer"
    /// ```
    ///
    /// Both are produced since 4b's Task 2, where 4a and 4b's Task 1 produced
    /// the second alone. The `LEAVE`'s own clause is the innermost entry:
    /// `leave_origin` resolves a real site now that this function passes
    /// `Some(&fragment.source)`, `record_leave_failure` records it, and the
    /// `seal_site_level` call beside it closes this level so the enclosing
    /// `INTERPRET` clause can still record its own. Both arms below need that
    /// call separately from the `run_bounded` error path above, because a
    /// `Flow::Leave` reaches here as an `Ok` and only becomes an `Err` here.
    fn run_fragment(&mut self, text: Vec<u8>) -> Result<Flow, Failure> {
        let fragment: Rc<Fragment> = match parse_interpret(text) {
            Ok(fragment) => Rc::new(fragment),
            // **Step 5b: the oracle's own condition, not a loud refusal.**
            // Measured, `interpret "do forever then"` on line 2 raises 27.901
            // at rc 229; this used to be `Loud::parse`, `rexx-exec: INTERPRET
            // text did not parse: ...` at rc 120. `error.rs`'s own `impl
            // From<&ParseError> for Raised` has the transcript and states
            // exactly what the conversion cannot carry.
            //
            // **No level is sealed here, and that is a real one-line
            // divergence rather than an oversight.** The oracle echoes the
            // failing *fragment* clause too, at indent 0 whatever the
            // enclosing indent (measured: two `DO`s deep, the fragment's
            // `do forever then` still prints at 0 while the `INTERPRET`
            // prints at 4, so it is not this task's activation base under
            // another name -- a parse-time echo simply carries no indent).
            // Reproducing it needs the failing clause's *text*, and
            // `ParseError` carries the clause's start byte with no end, so
            // there is no span to cut. Guessing one -- to end of source, or
            // to the next `;` -- is right for a single-clause fragment and
            // silently wrong for `interpret "do jj = 1 to 1; do forever
            // then; end"`, whose echo is `do forever then;` and not the rest
            // of the text. Closing it wants a clause span on `ParseError`,
            // which is a `rexx-parse` change; `execute`'s own parse arm
            // records the same gap for the top-level path.
            Err(error) => return Err(Raised::from(&error).into()),
        };

        // An owned `Fragment` would do here, since nothing but this loop reads
        // it. It is an `Rc` because that is the shape 4b needs, where an
        // `INTERPRET` inside a fragment makes this function reentrant and each
        // level anchors its own.
        let slots = self.fragment_plan(&fragment);
        let code = Code {
            body: &fragment.body,
            symbols: &fragment.symbols,
            slots: &slots,
        };

        // `exit` inside `INTERPRET` ends the program, not the fragment, so
        // it has to propagate rather than stop here -- `run_bounded`'s own
        // catch-all does exactly that for anything it does not own, `Exit`
        // included, with nothing fragment-specific to add.
        //
        // **`Some(&fragment.source)`, where 4a passed `None`.** The fragment
        // resolves its own clauses now: its spans are the only thing that
        // can, and the `Interpret` arm has already put the enclosing clause's
        // line and indent in place so the *text* comes from here while the
        // *line* and the indent base do not. `?` is deliberately not used --
        // an error has to seal this level before it propagates, or the
        // enclosing `INTERPRET` clause's own `step_in_temps_frame` will find
        // `failure_site` already full and record nothing.
        let flow = match self.run_bounded(
            &code,
            0,
            code.body.instructions.len(),
            Some(&fragment.source),
        ) {
            Ok(flow) => flow,
            Err(failure) => {
                self.seal_site_level();
                return Err(failure);
            }
        };

        // The exhausted search, at the fragment's own boundary rather than
        // the program's: measured, the oracle's `LEAVE`/`ITERATE` search
        // does not cross an `INTERPRET` (this function's own doc comment has
        // the six transcripts). Byte-identical in shape to
        // `run_activation`'s own four arms, deliberately -- same four
        // constructors, same `record_leave_failure` call -- because it is
        // the same event happening at a different boundary.
        //
        // `fragment.symbols` is what resolves the name, and this is the last
        // point at which it can: the id is interned in the fragment's own
        // fresh table (F-EX2, above), so nothing outside this function could
        // name it correctly even if it wanted to.
        match flow {
            Flow::Leave(name, origin) => {
                self.record_leave_failure(&origin);
                self.seal_site_level();
                let raised = match name {
                    None => raised_leave_no_loop(),
                    Some(id) => raised_leave_no_match(fragment.symbols.name(id).as_bytes()),
                };
                Err(raised.into())
            }
            Flow::Iterate(name, origin) => {
                self.record_leave_failure(&origin);
                self.seal_site_level();
                let raised = match name {
                    None => raised_iterate_no_loop(),
                    Some(id) => raised_iterate_no_match(fragment.symbols.name(id).as_bytes()),
                };
                Err(raised.into())
            }
            other => Ok(other),
        }
    }

    /// Closes off the level that is unwinding now, so the level above it can
    /// record its own clause.
    ///
    /// `Interp::failure_site` is first-wins *within* a level; this is what
    /// makes "a level" mean something. It moves whatever this level recorded
    /// onto `Interp::failure_sites` (innermost first, since the innermost
    /// level always seals first) and leaves the slot empty for the enclosing
    /// clause's own `step_in_temps_frame` to fill on the way out.
    ///
    /// **Called only on an error path, and only by a construct that opened a
    /// level.** Today that is `run_fragment` alone; Task 3's `CALL` is the
    /// next, and the rule for it is the same -- seal before the failure
    /// leaves the callee, never after. Sealing a level that recorded nothing
    /// is a no-op, which is what gives a fragment that failed to parse one
    /// echo instead of two.
    ///
    /// **Nothing here clears either field, and that is deliberate**: this
    /// function is the *unwinding* half, and a level that seals still has a
    /// report to give. Clearing is the *trapping* half, which is
    /// `offer_to_trap`'s (4b's Task 7, inherited item I11) -- it empties both
    /// the slot and this stack, because a trapped condition prints no report
    /// at all and its sites must not survive to be printed against a later,
    /// untrapped one. Through 4a a raise was always fatal, so a stale stack
    /// could not be observed; the two-raise transcript in
    /// `a_second_raise_after_a_trapped_one_reports_its_own_site` is what
    /// observes it now.
    fn seal_site_level(&mut self) {
        if let Some(site) = self.failure_site.take() {
            self.failure_sites.push(site);
        }
    }

    /// Drops one `DROP` target: a plain variable, a whole stem, one tail, or
    /// the `(v)` indirect form.
    ///
    /// **`Direct` resolves a compound's tail pieces as variables; `Indirect`
    /// is a subsidiary list, not a single name.** `Direct(id)`'s name came
    /// through the scanner, so a compound-shaped spelling still has real
    /// tail pieces to resolve -- `tail_key`, exactly what a `Compound`
    /// expression's own read or `Assignment`'s own write already does.
    ///
    /// `Indirect(id)`'s value is **blank- or tab-separated list of variable
    /// symbols, each validated and dropped on its own**, not one verbatim
    /// name -- a fix-round correction to this function's first version,
    /// which treated the whole value as a single name and let three classes
    /// of bad input through silently. Measured, the six rows that pin it
    /// down (all against the oracle):
    ///
    /// ```text
    /// a=1; b=2; v='a b'      ; drop (v); say a; say b   ->  A / B  (both dropped)
    /// x=1;      v=' x '      ; drop (v); say x           ->  X      (trimmed)
    /// v='9'                  ; drop (v)                  ->  Error 31.2
    /// v='.x'                 ; drop (v)                  ->  Error 31.3
    /// w=1;      v='(w)'      ; drop (v)                  ->  Error 20.928
    /// a=1;      v='a'        ; drop (v); say a           ->  A      (agrees with the old reading)
    /// ```
    ///
    /// The last row is why every pre-fix test passed: a single-word,
    /// already-valid, unpadded value is exactly where "split, validate,
    /// resolve each" and "resolve the whole value" coincide. The `(w)` row
    /// is what rules out a recursive reading -- a parenthesised entry is
    /// 20.928, "Symbol expected as an indirect variable name", the same
    /// error any other not-a-symbol word gets (`a-b` gives the identical
    /// 20.928, `found "a-b"`), not a second round of indirection.
    ///
    /// **Validation runs over the whole list before any drop happens.**
    /// Measured: `a=1; b=2; v='a 9 b'; drop (v)` raises 31.2 on `"9"` and
    /// leaves *both* `a` and `b` at `1` and `2` -- `a` is never dropped even
    /// though it sits before the bad word. So this collects every word's
    /// validated, upcased name first (`validate_indirect_word`, which can
    /// fail) and only then drops each one (`drop_by_name`, which cannot),
    /// rather than interleaving the two.
    ///
    /// **The value is upcased only after validation, one word at a time,
    /// never as a whole.** Measured: `v = 'x'; x = 1; drop (v); say x`
    /// prints `X` -- each word is upcased exactly as the scanner would have
    /// upcased it had it been written directly, because `DROP (v)` never
    /// goes through the scanner at all. A `Direct` name is already upcased,
    /// by `SymbolTable::intern`, long before this ever runs.
    fn drop_variable(&mut self, code: &Code<'_>, variable: &VariableRef) -> Result<(), Failure> {
        match variable {
            VariableRef::Direct(id) => {
                let name = code.symbols.name(*id);
                if shape_of(name.as_bytes()) == NameShape::Compound {
                    let (stem_name, _tails) = compound_parts(name);
                    let key = self.tail_key(code, *id);
                    self.stem_drop_tail(stem_name.as_bytes(), &key);
                } else {
                    self.drop_by_name(name.as_bytes());
                }
            }
            VariableRef::Indirect(id) => {
                let (value, _novalue) = self.read(code, *id);
                let text = self.to_text(value).into_owned();
                let mut names = Vec::new();
                for word in split_indirect_words(&text) {
                    names.push(validate_indirect_word(word)?);
                }
                for name in &names {
                    self.drop_by_name(name);
                }
            }
        }
        Ok(())
    }

    /// Drops the variable, whole stem, or one verbatim-keyed tail `name`'s
    /// own spelling names, dispatched by `shape_of`.
    ///
    /// Shared by `Direct`'s `Simple`/`Stem` cases (an already-upcased
    /// compile-time name) and by every word of an indirect subsidiary list
    /// (already validated and upcased by `validate_indirect_word`) -- both
    /// are "a plain string names a variable, resolve it with no further
    /// symbol lookup", the same operation `drop_variable`'s own doc comment
    /// says the two cases share. **Not** used for `Direct`'s `Compound` case:
    /// a source-level compound's tail pieces are still symbols to resolve
    /// (`tail_key`), which this function's uniform verbatim split at the
    /// first period does not do.
    fn drop_by_name(&mut self, name: &[u8]) {
        match shape_of(name) {
            NameShape::Simple => {
                let slot = self.slot_of(name);
                let frame = self.activation().frame;
                self.roots.clear_slot(frame, slot);
            }
            NameShape::Stem => self.stem_drop(name),
            NameShape::Compound => {
                let dot = name
                    .iter()
                    .position(|&b| b == b'.')
                    .expect("NameShape::Compound guarantees at least one period");
                let (stem_name, key) = name.split_at(dot + 1);
                self.stem_drop_tail(stem_name, key);
            }
        }
    }

    /// `NUMERIC DIGITS`/`FUZZ`/`FORM`, in every spelling the parser produces
    /// (`NumericSetting`, `rexx-parse`'s own `instruction.rs::numeric`).
    ///
    /// `DIGITS`/`FUZZ` with no expression reset to the package default --
    /// measured, `numeric digits 3; numeric digits; y = 1/3; say y` gives
    /// `0.333333333`, the DIGITS-9 rendering, and the reset is reported
    /// exactly as if `"9"` had been typed rather than as some sentinel
    /// meaning "no change": `numeric digits 20; numeric fuzz 15; numeric
    /// digits` raises 33.1 with `("9")` as the rejected candidate. `FORM`
    /// alone (`FormDefault`) resets the same way, to `SCIENTIFIC` -- measured,
    /// `numeric form engineering; numeric form; say form()` gives
    /// `SCIENTIFIC`. 4a has no `::OPTIONS` to move the package default away
    /// from `Scientific`, which is why `FormDefault` and `FormScientific` do
    /// the identical thing below; a later phase's `::OPTIONS FORM` is what
    /// would make the two differ, and should split this arm rather than
    /// assume they stay equal.
    /// `TRACE`'s four forms (D17). `Trace::Default` (bare `TRACE`) and a
    /// `Trace::Setting` letter that recognises but has nothing visible to
    /// show in this crate's scope (`C`/`L`/`E`/`F`/`N`/`O`) both land on
    /// `TraceMode::OFF` -- `mode_from_setting` draws no distinction between
    /// them because this crate cannot observe one (measured: `trace` alone
    /// and `trace value 'N'` are both silent, this task's own report has
    /// the transcript).
    ///
    /// `Trace::Setting`'s own bytes were already validated by `rexx-parse`'s
    /// `check_trace_setting` at parse time, so `.expect()` rather than
    /// propagating the `Err` arm -- a `Trace::Setting` this crate ever sees
    /// cannot carry an unrecognised letter. `Trace::Value`'s text has no
    /// such guarantee (it is computed at run time from an arbitrary Rexx
    /// expression), which is the one path that can reach `mode_from_
    /// setting`'s `Err` for real, and does through `raised_invalid_trace_
    /// letter`.
    fn exec_trace(&mut self, code: &Code<'_>, setting: &Trace) -> Result<(), Failure> {
        match setting {
            Trace::Default => {
                self.set_trace_mode(crate::trace::TraceMode::OFF);
            }
            Trace::Setting(bytes) => {
                self.set_trace_mode(
                    mode_from_setting(bytes)
                        .expect("rexx-parse's check_trace_setting already validated this byte"),
                );
            }
            // 24.901, unconditional -- measured, `trace 0` raises it
            // exactly like `trace 5` (this task's report), because this
            // runtime has no interactive debugging for a nonzero skip
            // count to be valid *from* either way.
            Trace::Skip(_) => {
                return Err(raised_numeric_trace_interactive_only().into());
            }
            // `TRACE VALUE expr`: computed at run time, then classified
            // exactly like a literal `TRACE` setting would have been --
            // measured, `trace value 5` raises 24.901 like `trace 5`, and
            // `trace value 'R'` behaves exactly like `trace r` (this
            // task's report has both transcripts). A whole number is a
            // skip count checked *before* trying it as a letter, matching
            // `rexx-parse`'s own `trace` parser's order (`instruction.rs`'s
            // `whole_number` attempt precedes its `check_trace_setting`
            // fallback).
            Trace::Value(expression) => {
                let value = self.eval(code, expression)?;
                self.roots.push_temp(value);
                let text = self.to_text(value).to_vec();
                if is_whole_number(&text) {
                    return Err(raised_numeric_trace_interactive_only().into());
                }
                self.set_trace_mode(mode_from_setting(&text).map_err(raised_invalid_trace_letter)?);
            }
        }
        Ok(())
    }

    fn exec_numeric(
        &mut self,
        code: &Code<'_>,
        setting: &NumericSetting,
        expression: &Option<Expr>,
    ) -> Result<(), Failure> {
        match setting {
            NumericSetting::Digits => {
                let default = rexx_num::DEFAULT_DIGITS.to_string();
                let text = self.numeric_operand(code, expression, "DIGITS", &default)?;
                self.activation_mut()
                    .settings
                    .set_digits_str(&text)
                    .map_err(raised_from_settings)?;
            }
            NumericSetting::Fuzz => {
                let text = self.numeric_operand(code, expression, "FUZZ", "0")?;
                self.activation_mut()
                    .settings
                    .set_fuzz_str(&text)
                    .map_err(raised_from_settings)?;
            }
            NumericSetting::FormDefault | NumericSetting::FormScientific => {
                self.activation_mut()
                    .settings
                    .set_form_str("SCIENTIFIC")
                    .expect("a hardcoded valid spelling always validates");
            }
            NumericSetting::FormEngineering => {
                self.activation_mut()
                    .settings
                    .set_form_str("ENGINEERING")
                    .expect("a hardcoded valid spelling always validates");
            }
            NumericSetting::FormValue => {
                // The parser only ever produces this with an expression: an
                // explicit `VALUE` with none is 35.917 at parse time
                // (`instruction.rs::numeric`), and the implicit spelling
                // (`NUMERIC FORM (expr)`) only takes this branch once a token
                // is already known to be there. Loud rather than a panic, on
                // this crate's own rule against aborting on a shape the
                // grammar rules out but the type does not.
                let Some(expression) = expression else {
                    return Err(Loud {
                        message: "NUMERIC FORM VALUE with no expression".to_string(),
                    }
                    .into());
                };
                // `set_form_str`'s own doc comment: the runtime `VALUE` path
                // does no uppercasing, no trimming and no abbreviation, unlike
                // the keyword spellings above -- measured, `numeric form
                // value 'engineering'` is 25.11, not accepted
                // case-insensitively.
                let value = self.eval(code, expression)?;
                self.roots.push_temp(value);
                let text = self.to_text(value).to_vec();
                // `>K>   "FORM" => "engineering"` (F2, branch review): fires
                // before `set_form_str`'s own validation, same as `DIGITS`/
                // `FUZZ` below and `setup_controlled`'s own `TO`/`BY`/`FOR`
                // -- measured, `numeric form value 'engineering'` under
                // `trace r` traces `>K>` and *then* raises 25.11, not the
                // reverse. Untranslated, matching the error report's own
                // `found "engineering"` substitution, which is also
                // unmodified case -- `set_form_str`'s own no-uppercasing
                // rule for this one path, unlike the two keyword spellings
                // above.
                self.trace_keyword(self.clause_state.current_value_indent, "FORM", &text);
                let text = String::from_utf8_lossy(&text).into_owned();
                self.activation_mut()
                    .settings
                    .set_form_str(&text)
                    .map_err(raised_from_settings)?;
            }
        }
        Ok(())
    }

    /// Evaluates `expression`, or answers `default` when there is none
    /// (`NUMERIC DIGITS`/`FUZZ` alone).
    ///
    /// A `String` rather than the value's own bytes: `set_digits_str`/
    /// `set_fuzz_str` take `&str`, and a Rexx value's bytes are not
    /// guaranteed UTF-8. `from_utf8_lossy` is this crate's own standing choice
    /// for exactly that gap (`Raised::nonnumeric`'s substitution text,
    /// `error.rs`), and a lossy byte cannot parse as a valid DIGITS/FUZZ
    /// value either way, so the conversion still ends in the right error
    /// family rather than silently accepting mangled input.
    ///
    /// **`>K>` traces only when `expression` is `Some` (F2, branch review,
    /// Important).** `RexxInstructionNumeric::execute` calls
    /// `traceKeywordResult` for `DIGITS`/`FUZZ`/`FORM` alike whenever an
    /// expression is present (`NumericInstruction.cpp:98`/`135`/`174`,
    /// verified by containment) -- measured, `trace r; numeric digits 9`
    /// emits `>K>   "DIGITS" => "9"` after the clause echo, and a bare
    /// `numeric digits`/`numeric fuzz` (no expression, "restore to the
    /// previous value") traces nothing at all, confirming the gate is on
    /// the expression's presence and not on the keyword. Before the
    /// validating `set_digits_str`/`set_fuzz_str` call, same reason
    /// `setup_controlled` already traces `TO`/`BY`/`FOR` before validating
    /// them: measured, `numeric digits 'x'` under `trace r` emits `>K>
    /// "DIGITS" => "x"` and *then* raises 26.5, not the reverse.
    fn numeric_operand(
        &mut self,
        code: &Code<'_>,
        expression: &Option<Expr>,
        keyword: &str,
        default: &str,
    ) -> Result<String, Failure> {
        match expression {
            Some(expression) => {
                let value = self.eval(code, expression)?;
                self.roots.push_temp(value);
                let text = self.to_text(value).to_vec();
                self.trace_keyword(self.clause_state.current_value_indent, keyword, &text);
                Ok(String::from_utf8_lossy(&text).into_owned())
            }
            None => Ok(default.to_string()),
        }
    }

    /// `instruction`'s own clause text and the 1-based line to print it against,
    /// or `None` when `source` is `None`.
    ///
    /// **`Interp::clause_line_override` is why this is a method and not the free
    /// function it was through 4a.** The line and the text do not always come
    /// from the same place: inside an `INTERPRET` fragment the text is the
    /// fragment's (its spans index the fragment's own source, and nothing else
    /// can resolve them) while the line is the enclosing `INTERPRET` clause's,
    /// measured. Threading that override through the four call sites --
    /// `step_in_temps_frame`, `record_failure_at`, `leave_origin`,
    /// `run_otherwise` -- would give each of them a parameter about a construct
    /// none of them otherwise knows exists, so it reads the field instead,
    /// exactly as `current_value_indent` and `indent_offset` already do.
    ///
    /// `source: None` no longer has a caller: `run_fragment` was the one, and it
    /// passes `Some(&fragment.source)` since 4b's Task 2 gave the report an echo
    /// per level. The parameter is still an `Option` because collapsing it is a
    /// mechanical change across every signature that threads it, which is a
    /// restructuring rather than this task's -- **but nothing below may assume a
    /// site is unresolvable any more**, and the comments that used to say so
    /// have been corrected rather than left standing.
    fn clause_site(
        &self,
        source: Option<&ProgramSource>,
        instruction: &Instruction,
    ) -> Option<(usize, Vec<u8>)> {
        let line = self.clause_line(source, instruction)?;
        let source = source?;
        Some((
            line,
            source
                .join_span(instruction.clause_span.clone())
                .map_or_else(
                    // Visible rather than silent, matching `Raised::message`'s
                    // own reasoning for a catalogue miss: the error path is the
                    // worst place to turn a reportable condition into a crash or
                    // a blank line.
                    || b"<clause span outside the retained source>".to_vec(),
                    |bytes| bytes.into_owned(),
                ),
        ))
    }

    /// `clause_site`'s own line half, alone -- extracted so `current_clause_
    /// line` (`lib.rs`'s own doc comment on the field) can be kept fresh on
    /// every step without paying for `clause_site`'s own `join_span` text
    /// extraction, which nothing needs when only `SIGL`'s value is being
    /// computed. Identical rule, same `clause_line_override` honoured the
    /// same way, so a `SIGNAL`/`CALL` fired from inside an `INTERPRET`
    /// fragment reads the enclosing clause's own line here exactly as
    /// `clause_site` already gives the trace/error paths.
    fn clause_line(
        &self,
        source: Option<&ProgramSource>,
        instruction: &Instruction,
    ) -> Option<usize> {
        let source = source?;
        Some(
            self.clause_line_override
                .unwrap_or_else(|| source.line_of(instruction.clause_span.start)),
        )
    }

    // `fragment_plan` and `slot_of` live in `plan.rs` (Task 6), beside `Plan`
    // itself; `stem_assign`/`stem_set`/`stem_drop`/`stem_drop_tail`/
    // `tail_key` live in `stem.rs` (Task 5), beside the rest of the D15a
    // library. `read` lives in `lib.rs`, beside `Interp`'s other value-model
    // entry points.
}

/// How many spaces of nesting depth `target`'s own clause sits at --
/// Task 11's whole indentation feature, and the design decision at its
/// centre: **computed fresh from the flat instruction list every time,
/// never carried on a running `Interp` counter.**
///
/// Task 10's own report concluded the depth is derivable from the AST
/// statically, with no runtime block stack, and this task's own oracle
/// probes confirm it (the report has the full transcripts): a clause's
/// indentation never depends on which iteration of an enclosing loop is
/// currently running, only on how many `DO`/`LOOP` bodies, matched `IF`
/// branches and `SELECT` scans lexically enclose it -- exactly the
/// information `If`'s `false_target`, `Select`'s `whens`/`otherwise`/`end`
/// and `Loop`'s `end` already carry, with nothing further to add.
///
/// # That last paragraph is measurably false
///
/// **The oracle's indent is not a pure function of lexical nesting.** Any
/// repetitive `DO`/`LOOP` that **completes at least one body pass** and then
/// ends because a **control test fails** decrements the oracle's own counter
/// one time too many, so later clauses print two spaces lower than their
/// lexical depth. Count exhausted, `WHILE` false and `UNTIL` true all qualify.
/// Measured, no `INTERPRET` and no `CALL` anywhere -- `do` / `do jj = 1 to 1`
/// / `nop` / `end` / `say 1/0` / `end` reports the `say` at **0** on the
/// oracle and at 2 here, and `n=0; do while n = 0; n = 1; end` in the same
/// position does the same.
///
/// A **zero-trip** loop (`do while 0 = 1`, `do jj = 1 to 0`, `do 0`), a loop
/// left by **`LEAVE`**, and a non-repetitive block (`IF`, `SELECT`, plain
/// `DO`) do not. **The distinguishing property is whether a body pass
/// completed, not whether a re-test failed** -- a zero-trip loop's first test
/// also fails, and an earlier revision of this comment drew exactly that
/// wrong conclusion from the zero-trip row.
///
/// **The cause is a C++ defect, and naming the cause is the only version of
/// this that has not needed correcting.** `settings.traceIndent` is a mutable
/// counter. A loop ending normally restores the value `DoBlock` saved at
/// construction (`BaseDoInstruction.cpp:161`); a loop whose control test fails
/// takes a different exit path that bare-decrements it (`:377`). So a stray
/// decrement survives until some enclosing construct restores from its own
/// saved block, and is discarded there.
///
/// **An earlier revision of this paragraph enumerated the discarding
/// constructs and was falsified by `do label q ... end`** -- a plain,
/// non-repetitive `DO` carrying a `LABEL` discards it too, because
/// `SimpleDoInstruction.cpp:78-89` creates the saved block only when a `LABEL`
/// is present. That was the fourth construct-shaped rule here to drift, after
/// the qualification predicate, the scope, and accumulation.
///
/// **Do not write a fifth.** If a shape is not in a measured table, work out
/// which exit path it takes. `phase-4-exclusions.txt`'s row carries the C++
/// citations and every table.
///
/// It happens on the same *occasion* as the control variable's own value
/// lines -- a re-tested pass -- but **not by the same mechanism, and they do
/// not close together.** That row once said they did; 4b Task 9 closed the
/// value lines alone, by moving the `BY` increment into `loop_advance`, and
/// nothing about this indent changed. It is 4a's, not this function's to fix
/// under any task that has run so far. **What matters here is that the
/// paragraph above reads as settled and is not**, so a later reader does not
/// build on it: this function computes the *lexical* indent, and closing the
/// gap means modelling the oracle's counter rather than making this function
/// impure.
///
/// `the_indent_after_a_loop_has_already_exited_is_not_left_over_from_it`
/// (this file's own tests) does not catch it, and the reason is worth
/// keeping: it runs at top level, where the oracle's counter is already at 0
/// and cannot go lower. That is the same "at indent 0 the base is 0" blind
/// spot that hid two of 4b Task 2's own four mutations.
///
/// **A mutable counter was the design first tried here, and it was dropped
/// once it became clear what it would cost to keep correct.** It would need
/// to be incremented and decremented in exact lockstep on *every* exit path
/// out of *every* `IF`/`SELECT`/`DO` arm, including every `?`-propagated
/// error path and the `Goto`-absorption case `Flow::Leave`'s own doc comment
/// describes -- precisely the shape of defect this crate's own skipped-
/// `pop_frame` discussion elsewhere warns is easy to introduce and hard to
/// notice, because the symptom is two spaces of wrong stderr that no
/// existing test asserts on. A pure function of `(instructions, target)`
/// cannot desync from anything, because there is no state to desync: asking
/// it twice for the same `target` on the same body always gives the same
/// answer, computed the same way, whether the failure happens on a loop's
/// first pass or its thousandth. `the_indent_after_a_loop_has_already_exited_
/// is_not_left_over_from_it` (this file's own tests) is the test that would
/// have caught a live counter's most likely failure mode -- a raise reached
/// *after* a loop's own body has already run and exited, at a shallower
/// lexical depth, where a counter not perfectly unwound on every path out of
/// the loop would over-indent and a purely static answer cannot.
///
/// **The one place this recomputes something rather than reading it back**
/// is `WHILE`/`UNTIL`: neither corresponds to a *distinct* flat instruction
/// position with the right semantics (`WHILE` shares the `DO`/`LOOP`
/// instruction's own clause, tested *inside* the loop's own frame; `UNTIL`
/// shares the `END`'s, likewise inside), so `Do`'s own arm adds the loop's
/// own two spaces on top of `static_indent(instructions, do_index)` directly
/// at its two call sites rather than asking this function to guess which of
/// two different, correct answers a `DO`/`LOOP` instruction's *own* index
/// means (measured: `do i = 1 to 3 for 1/0`'s control-setup failure is
/// unindented at that same index, while `do while 1/0` is indented two).
///
/// Recurses into whichever construct's own range contains `target`, adding
/// that construct's contribution before descending -- see the report for
/// the additive model (two per `DO`/`LOOP`, four per matched `IF` branch,
/// two for a `SELECT`'s own scan plus four more for a matched `WHEN`'s
/// `THEN` or two more for `OTHERWISE`) and the oracle transcripts that pin
/// each number, including the two the brief this task started from did not
/// state: a `WHEN`'s own condition sits at the `SELECT`'s own two, not zero,
/// and `OTHERWISE`'s own body is two more, not the `WHEN`-`THEN` shape's
/// four more.
fn static_indent(instructions: &[Instruction], target: usize) -> usize {
    indent_in_range(instructions, 0, instructions.len(), target)
}

/// `static_indent`'s own recursive worker, over one `[start, end)` range --
/// the same range shape `run_bounded` itself runs, so this function's
/// dispatch on `If`/`Select`/`Do`/`Loop` mirrors `step`'s own arms for them,
/// reading the identical fields, just never evaluating anything.
fn indent_in_range(instructions: &[Instruction], start: usize, end: usize, target: usize) -> usize {
    let len = instructions.len();
    let mut pc = start;
    while pc < end {
        if pc == target {
            // `target` is this range's own instruction at this position --
            // a plain clause, or a block-opener's own clause (a `DO`'s
            // control-setup expressions, a `SELECT`'s own `CASE` scrutinee),
            // with nothing further to add beyond whatever the caller already
            // contributed before recursing in here.
            return 0;
        }
        match &instructions[pc].kind {
            InstructionKind::If { false_target, .. } => {
                let false_target = false_target.unwrap_or(len);
                let then_start = pc + 1;
                // `then_start` is the `Then` marker's *own* index, not its
                // body's first instruction -- measured against the oracle
                // (`ThenInstruction.cpp`'s `execute`: `indent(); trace;
                // indent();`), a marker clause sits at exactly two spaces,
                // half of what its own body gets (four). Before this check
                // existed, `target == then_start` fell into the body branch
                // below and got the wrong answer (4, not 2) because that
                // branch's own recursive call happens to return 0 for the
                // very first position of its range -- silently, since
                // nothing before this task ever asked for a `Then`'s own
                // indent (a marker clause carries no expression, so it can
                // never be a `FailureSite`, only ever a `TRACE` echo).
                if target == then_start {
                    return 2;
                }
                if target > then_start && target < false_target {
                    return 4 + indent_in_range(instructions, then_start, false_target, target);
                }
                match instructions.get(false_target).map(|i| &i.kind) {
                    Some(InstructionKind::Else { then_exit }) => {
                        let else_end = then_exit.unwrap_or(len);
                        // Same shape as `Then`, and the same measurement
                        // (`ElseInstruction.cpp`'s `execute` is byte-for-byte
                        // the same two-`indent()`-calls dance). Before this
                        // check, `target == false_target` fell all the way
                        // through this whole arm (the body check below is
                        // strict `>`) to `pc = else_end; continue`, which
                        // advances `pc` *past* `target` in the enclosing
                        // walk -- the `Else` marker's own index was never
                        // revisited by anything, silently returning
                        // whatever the *enclosing* level happened to be
                        // (0 too shallow) rather than erroring.
                        if target == false_target {
                            return 2;
                        }
                        if target > false_target && target < else_end {
                            return 4 + indent_in_range(
                                instructions,
                                false_target + 1,
                                else_end,
                                target,
                            );
                        }
                        pc = else_end;
                    }
                    _ => pc = false_target,
                }
                continue;
            }
            InstructionKind::Do(body) | InstructionKind::Loop(body) => {
                let body_start = pc + 1;
                let end_index = body.end.expect(
                    "an unclosed DO/LOOP is error 14.1/14.5, so a body that parsed has this set",
                );
                if target > pc && target < end_index {
                    return 2 + indent_in_range(instructions, body_start, end_index, target);
                }
                pc = end_index + 1;
                continue;
            }
            InstructionKind::Select {
                whens,
                otherwise,
                end: select_end,
                ..
            } => {
                let select_end = select_end.unwrap_or(len);
                if target > pc && target < select_end {
                    // Inside this SELECT's own scan-through-dispatch range:
                    // two spaces on their own (measured: a WHEN's own
                    // condition, `target == when_index` below, sits at
                    // exactly this level), plus whichever branch's own
                    // extra applies.
                    for &when_index in whens {
                        if when_index == target {
                            return 2;
                        }
                        let (body_start, body_end) = match &instructions[when_index].kind {
                            InstructionKind::When { false_target, .. }
                            | InstructionKind::WhenCase { false_target, .. } => {
                                (when_index + 1, false_target.unwrap_or(len))
                            }
                            // This asserted unreachability too, on a claim
                            // that turned out to be no better-founded than
                            // the `OTHERWISE` one three lines of history
                            // below: "`whens` holds only `When`/`WhenCase`"
                            // is `rexx-parse`'s own invariant, not this
                            // function's, and this phase's invariants have
                            // not all held -- the absorbed-`WHEN` case
                            // (`when_absorbing_a_when_parses_and_runs_at_
                            // rc_0`, this module's own test) is exactly a
                            // `When` instruction executing while its
                            // enclosing `SELECT`'s own `whens` does not list
                            // it, which is the same shape of surprise. If a
                            // reader ever sees this, `whens` names an index
                            // whose own kind is not what built it -- a
                            // `rexx-parse` defect, not a formatting one, and
                            // nothing this function can correct. Skipping
                            // the entry (matching the outer fallback's own
                            // "nothing further to add" answer once the loop
                            // and the `OTHERWISE` check both come up empty)
                            // keeps the diagnostic path alive instead of
                            // trading a wrong indent for a dead process.
                            _ => continue,
                        };
                        // `body_start` is the WHEN's own `Then` marker,
                        // sharing `InstructionKind::Then` with `IF` (both go
                        // through `instruction.rs`'s `if_instruction`) --
                        // measured against the oracle exactly like `IF`'s
                        // own: the marker sits at half its body's indent
                        // (four, not six). Before this check, `target ==
                        // body_start` matched the `>=` below and returned
                        // six, the body's own value -- again invisible
                        // before `TRACE`, since a `Then` marker never raises.
                        if target == body_start {
                            return 4;
                        }
                        if target > body_start && target < body_end {
                            return 6 + indent_in_range(instructions, body_start, body_end, target);
                        }
                    }
                    if let Some(otherwise_index) = otherwise {
                        // `OTHERWISE` traces its own clause once (no double
                        // `indent()` -- `OtherwiseInstruction.cpp`'s
                        // `execute` is `trace; indent();`, not `indent();
                        // trace; indent();`) at the SELECT's own scan level,
                        // the same two spaces a `WHEN`'s condition gets, and
                        // only its body gets the further two. Before this
                        // check, `target == *otherwise_index` matched
                        // neither this arm's `>` check nor anything in the
                        // `whens` loop, and fell all the way to the
                        // `unreachable!` below -- **a live panic**, not
                        // merely a wrong number, confirmed by directly
                        // calling `static_indent` on `select\nwhen 1 = 0
                        // then nop\notherwise\nsay 'y'\nend`'s own
                        // `otherwise_index` before this fix existed.
                        if target == *otherwise_index {
                            return 2;
                        }
                        if target > *otherwise_index && target < select_end {
                            return 4 + indent_in_range(
                                instructions,
                                otherwise_index + 1,
                                select_end,
                                target,
                            );
                        }
                    }
                    // `target` is in range but matches none of the above.
                    // After the two equality cases just added, every
                    // reachable position inside a resolved SELECT is
                    // provably one of: a WHEN's own index, a WHEN's own
                    // `Then` marker, one WHEN's own body, `OTHERWISE`'s own
                    // marker, or `OTHERWISE`'s own body -- so a body that
                    // parsed should never reach here. It reached here once
                    // already, though (the `OTHERWISE`-marker case, before
                    // its equality check existed), and this exact arm is
                    // where that panic actually happened -- `unreachable!`
                    // asserted a claim about the code's own shape, and the
                    // claim was false for a case nothing had exercised yet.
                    // This crate's rule for the diagnostic path (`error.rs`'s
                    // message-catalogue miss renders a visible marker
                    // instead of aborting; `clause_site`'s own fallback, this
                    // file, cites the identical reasoning) is that a
                    // formatting gap must never become a crash, and
                    // `static_indent` feeds both the error
                    // report and `TRACE` now -- so this returns the
                    // enclosing level (0 relative, "nothing further to add")
                    // rather than asserting unreachability a second time.
                    // If a reader ever sees indentation that looks too
                    // shallow by exactly the amount a `SELECT` construct
                    // should have contributed, this is where to look: it
                    // means some future `SELECT`-shaped clause position is
                    // not one of the five cases enumerated above.
                    return 0;
                }
                pc = select_end;
                continue;
            }
            _ => {}
        }
        pc += 1;
    }
    0
}

/// Which of the three variable shapes `name`'s own spelling is, from an
/// already-interned (or already-upcased runtime) name alone.
///
/// Reproduces the scanner's own classification (`scanner.rs::scan_symbol`,
/// `SymbolClass::{Variable,Stem,Compound}`) as a pure function of the byte
/// string, which is all a `DROP` target has by the time it reaches here --
/// a direct target's name came from the scanner originally, but an indirect
/// one (`DROP (v)`) never did, so this cannot simply read a tag the AST
/// already carries. The rule is exactly the scanner's: no period is a simple
/// variable; exactly one period, and it is the last byte, is a stem;
/// anything else with a period -- two or more, or one not at the end -- is a
/// compound.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum NameShape {
    Simple,
    Stem,
    Compound,
}

fn shape_of(name: &[u8]) -> NameShape {
    let dots = name.iter().filter(|&&b| b == b'.').count();
    if dots == 0 {
        NameShape::Simple
    } else if dots == 1 && name.last() == Some(&b'.') {
        NameShape::Stem
    } else {
        NameShape::Compound
    }
}

/// Splits a `DROP (v)` wrapper's value into its subsidiary list's words.
///
/// A blank (`' '`) or a tab (`'\t'`) separates words, any run of either
/// counts as one separator, and an empty word never results -- measured,
/// `'a    b'` and a leading/trailing-blank `'  a  b  '` both give exactly
/// `["a", "b"]`, and a `'09'x` (tab) byte between two names splits them the
/// same way a space does. A newline or carriage return does **not**
/// separate: `'a' || '0a'x || 'b'` is one word, `"a\nb"`, which then fails
/// `validate_indirect_word`'s character check and raises 20.928 rather than
/// splitting. An all-blank or empty value yields no words at all, which is
/// why `drop (v)` on an empty or blanks-only `v` is a silent no-op on the
/// oracle rather than an error.
fn split_indirect_words(text: &[u8]) -> impl Iterator<Item = &[u8]> {
    text.split(|&b| b == b' ' || b == b'\t')
        .filter(|word| !word.is_empty())
}

/// One legal symbol character, by the scanner's own character table
/// (`scanner.rs`'s `is_symbol_char`: `! . ? _ 0-9 A-Z a-z`, ASCII only --
/// `SymbolTable::intern`'s own doc says a non-ASCII byte cannot be part of a
/// symbol at all).
fn is_symbol_byte(b: u8) -> bool {
    matches!(b, b'!' | b'.' | b'?' | b'_' | b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z')
}

/// Validates one word of an indirect `DROP`'s subsidiary list and answers
/// its upcased name, or the condition the oracle raises for it.
///
/// Three ways a word can fail, checked in the order the oracle's own error
/// numbers imply (a character-set check before either shape check, since a
/// word with an illegal character is never inspected for its *first*
/// character at all -- measured, `'a-b'` and `'(w)'` both give 20.928, not
/// 31.2/31.3, even though neither starts with a digit or a period):
///
/// * any byte outside the symbol character set (`is_symbol_byte`) -- 20.928,
///   "Symbol expected as an indirect variable name"; this is also what rules
///   out treating a parenthesised entry as a nested indirect reference,
///   since `(`/`)` are not symbol characters and so are rejected the same
///   way any other stray punctuation is, not by a dedicated recursion guard;
/// * a leading digit -- 31.2, "Variable symbol must not start with a
///   number", matching `SymbolClass::Constant`'s own first-byte rule;
/// * a leading period -- 31.3, "Variable symbol must not start with a
///   '.'" (measured on a bare `"."` too, not only a longer dot-led word).
///
/// Every substitution is the word's **own, unmodified** bytes -- measured,
/// `'.X'`/`'9abc'` report `found ".X"`/`found "9abc"`, not the upcased form
/// -- so upcasing happens only on the success path, after every check.
fn validate_indirect_word(word: &[u8]) -> Result<Vec<u8>, Failure> {
    if !word.iter().copied().all(is_symbol_byte) {
        return Err(raised_symbol_expected(word).into());
    }
    match word[0] {
        b'0'..=b'9' => return Err(raised_digit_led(word).into()),
        b'.' => return Err(raised_dot_led(word).into()),
        _ => {}
    }
    Ok(word.to_ascii_uppercase())
}

/// 20.928: a subsidiary-list word is not a legal symbol at all (contains a
/// byte outside `is_symbol_byte`'s set, which is also what a parenthesised
/// entry like `"(w)"` fails on).
fn raised_symbol_expected(found: &[u8]) -> Raised {
    Raised::syntax(20, 928, vec![String::from_utf8_lossy(found).into_owned()])
}

/// 31.2: a subsidiary-list word starts with a digit.
fn raised_digit_led(found: &[u8]) -> Raised {
    Raised::syntax(31, 2, vec![String::from_utf8_lossy(found).into_owned()])
}

/// 31.3: a subsidiary-list word starts with a period.
fn raised_dot_led(found: &[u8]) -> Raised {
    Raised::syntax(31, 3, vec![String::from_utf8_lossy(found).into_owned()])
}

/// 34.1: a single (non-list) `IF` condition is not exactly `0` or `1`.
/// `Error_Logical_value_if`, catalogue text "Value of expression following
/// IF keyword must be exactly \"0\" or \"1\"; found \"...\"", one
/// substitution, the operand's own rendered text.
fn raised_if_not_logical(found: &[u8]) -> Raised {
    Raised::syntax(34, 1, vec![String::from_utf8_lossy(found).into_owned()])
}

/// 34.2: a single (non-list) `WHEN` condition is not exactly `0` or `1`.
/// `Error_Logical_value_when`, the same shape as `raised_if_not_logical`
/// with `WHEN`'s own sub-number -- a plain `WHEN`'s comma list is the
/// opposite case (`WhenCase`'s doc comment) and never reaches this raiser,
/// since `eval_condition` only calls it when `condition.kind` is not
/// `ExprKind::Logical`.
fn raised_when_not_logical(found: &[u8]) -> Raised {
    Raised::syntax(34, 2, vec![String::from_utf8_lossy(found).into_owned()])
}

/// 7.3: a `SELECT` reached its `END` with every `WHEN` false and no
/// `OTHERWISE`. `Error_When_expected_nootherwise`, catalogue text "All WHEN
/// expressions of SELECT are false; OTHERWISE expected.", no substitutions
/// (measured against `interpreter/messages/rexxmsg.xml`'s own `<Text>` for
/// major 7 sub 003, which carries no `<Sub>` tag).
///
/// Raised from `End`'s own arm, not `Select`'s -- `EndStyle::Select`'s own
/// doc comment says reaching that `END` at run time *is* the error, and
/// `Select`'s arm sends every other outcome around this instruction
/// entirely (`Flow::Goto` past it on a match, or onto it exactly on no
/// match/no `OTHERWISE`), so the clause `run_activation`'s failure path
/// echoes is the `END`'s own, matching the oracle (measured, rc 249).
fn raised_select_no_when() -> Raised {
    Raised::syntax(7, 3, Vec::new())
}

/// Converts a `rexx-num` settings failure into a `Raised`.
///
/// `ArithError` has a `sub_code` accessor `rexx-num` made `pub` expressly for
/// `error.rs`'s own `From` impl (that impl's doc comment says so);
/// `SettingsError`'s equivalent is still private, and nothing asked for it to
/// change for this one caller. The `(major, sub)` pairs below are copied from
/// `settings.rs`'s own doc comments on each variant rather than read through
/// an accessor that does not exist yet. The pair then goes through
/// `Raised::syntax`, like every other raiser in this file since 4b's Task 7.
/// Turns `RAISE SYNTAX`'s own argument into the condition it names, or into
/// the condition the oracle raises when it names nothing.
///
/// **Three outcomes, all measured, and 4a never had to know about two of them
/// because `RAISE` is the first construct that lets a program name an
/// arbitrary error number.**
///
/// ```text
/// raise syntax 40.4       -> 40.4       the catalogue entry
/// raise syntax 40         -> 40.0       ditto, major line only (sub 0)
/// raise syntax 40.001     -> 40.1       ".001" is the integer 1
/// raise syntax '4E1'      -> 40.0       each half is a Rexx number, not an int literal
/// raise syntax '40.1E2'   -> 98.941     found "40100"
/// raise syntax 40.10      -> 98.941     found "40010"
/// raise syntax 1          -> 98.941     found "1.0"
/// raise syntax 3.5        -> 98.941     found "3005"
/// raise syntax 0 / 100 / 999 / 'abc' / 40.1000 / '40.'  -> 33.904
/// ```
///
/// **The major must be 1..=99 and the sub 0..=999**; anything else -- a
/// non-number, zero, `100`, `40.1000` -- is `33.904`, "Incorrect expression
/// result following SYNTAX keyword of RAISE instruction", rc 223. Both bounds
/// measured at their boundary: `99` is accepted and `100` is not, `40.999` is
/// accepted and `40.1000` is not.
///
/// **Each half is a Rexx number, not a Rust integer literal** (fix round 2's
/// NEW 4). `Number::parse` then `whole_value` is what the oracle's own
/// `RexxString::numberValue` does, and it is observable: `'4E1'` is major 40,
/// and `'40.1E2'` has sub 100. A decimal point with nothing after it is
/// rejected outright, where no decimal point at all means sub 0 -- measured,
/// `raise syntax '40.'` is 33.904 and `raise syntax 40` is the `(40, 0)`
/// entry.
///
/// **The sub is the digits after the point as a number in their own right**,
/// not as a fraction: `.4` is 4, `.001` is 1, `.10` is 10. Measured through
/// `raise syntax 40.001`, which renders `(40, 1)`.
///
/// **A well-formed pair the catalogue does not know is `98.941`**, rc 158,
/// and its own `&1` is the *composed* number `major * 1000 + sub` -- except
/// when the catalogue has no `(major, 0)` entry at all, where it is the
/// original `major.sub`. Measured: `40.10` gives `"40010"` and `3.5` gives
/// `"3005"`, while `1` gives `"1.0"` and `2.1` gives `"2.1"`.
///
/// That exception is the oracle's own structure rather than a curve fit, and
/// the re-review confirmed it in the C++: `createExceptionObject` raises
/// 98.941 with a dot-formatted substitution when the *primary* message is
/// missing, `buildMessage` raises it with the integer form when only the
/// *secondary* is. Two call sites, two forms. The branch below asks
/// `lookup(major, 0)`, which is exactly "is the primary message there".
///
/// **How many majors take the dot form is not two.** An earlier version of
/// this comment said majors 1 and 2 were the only ones in 1..=99 with no
/// `(major, 0)` entry; counted from the generated catalogue there are 45 (1,
/// 2, 12, 32, 50-87, 94, 95, 96). The code was never wrong -- it looks the
/// major up rather than hard-coding a pair -- but the claim was asserted
/// from two probes rather than counted, which is the error the round it
/// appeared in was supposed to be about.
fn raise_syntax_condition(text: &[u8], additional: Vec<String>) -> Raised {
    /// One half of the argument as a Rexx number: `numberValue`, then a
    /// whole-number check. `None` for anything that is not a whole number,
    /// which the caller turns into 33.904.
    fn whole(text: &str) -> Option<i64> {
        Number::parse(text)?.whole_value(rexx_num::DEFAULT_DIGITS as usize)
    }

    let text = String::from_utf8_lossy(text);
    let (major, sub) = match text.split_once('.') {
        // A decimal point with an empty tail is rejected rather than read as
        // zero: `'40.'` is 33.904 where `40` is the `(40, 0)` entry.
        Some((_, "")) => return Raised::syntax(33, 904, Vec::new()),
        Some((major, sub)) => (whole(major), whole(sub)),
        None => (whole(text.as_ref()), Some(0)),
    };
    let (Some(major), Some(sub)) = (major, sub) else {
        return Raised::syntax(33, 904, Vec::new());
    };
    if !(1..=99).contains(&major) || !(0..=999).contains(&sub) {
        return Raised::syntax(33, 904, Vec::new());
    }
    // Both bounds are checked above, so neither narrowing can lose anything.
    let (major, sub) = (major as u16, sub as u16);
    if rexx_inventory::errors::lookup(major, sub).is_some() {
        return Raised::syntax(major, sub, additional);
    }
    let found = if rexx_inventory::errors::lookup(major, 0).is_some() {
        (u32::from(major) * 1000 + u32::from(sub)).to_string()
    } else {
        format!("{major}.{sub}")
    };
    Raised::syntax(98, 941, vec![found])
}

/// A `RAISE`'s condition name as `Raised::condition` carries it.
///
/// Always owned: the name comes from the program's own text (`USER FOO` is
/// built by the parser from the symbol after `USER`), which is exactly the
/// case `Cow` is there for. Every condition this crate raises on its own
/// stays on the borrowed side.
fn condition_name(name: &[u8]) -> Cow<'static, str> {
    Cow::Owned(String::from_utf8_lossy(name).into_owned())
}

fn raised_from_settings(error: SettingsError) -> Raised {
    let additional = error.additional();
    let (number, sub): (u16, u16) = match &error {
        SettingsError::InvalidForm { .. } => (25, 11),
        SettingsError::DigitsNotWhole { .. } => (26, 5),
        SettingsError::FuzzNotWhole { .. } => (26, 6),
        SettingsError::FuzzNotBelowDigits { .. } => (33, 1),
    };
    Raised::syntax(number, sub, additional)
}

/// 34.3: a single (non-list) `WHILE` condition is not exactly `0` or `1`.
/// Same shape as `raised_if_not_logical`/`raised_when_not_logical`, with
/// `WHILE`'s own sub-number; a comma-list condition never reaches this
/// raiser (34.6 instead, `eval_logical_list`'s own answer).
fn raised_while_not_logical(found: &[u8]) -> Raised {
    Raised::syntax(34, 3, vec![String::from_utf8_lossy(found).into_owned()])
}

/// 34.4: `UNTIL`'s own version of `raised_while_not_logical`.
fn raised_until_not_logical(found: &[u8]) -> Raised {
    Raised::syntax(34, 4, vec![String::from_utf8_lossy(found).into_owned()])
}

/// 26.2: a bare `DO`'s own repetition-count expression is not zero or a
/// positive whole number. `Error_Invalid_expression_do`, measured: `do
/// 'a'`/`do -1`/`do 2.5` all give this, `found` the operand's own
/// unmodified text (`"a"`/`"-1"`/`"2.5"`).
fn raised_repetition_count_not_whole(found: &[u8]) -> Raised {
    Raised::syntax(26, 2, vec![String::from_utf8_lossy(found).into_owned()])
}

/// 26.3: a `DO`/`LOOP`'s `FOR` expression is not zero or a positive whole
/// number. Measured: `do i = 1 to 3 for 'x'`/`for -1`/`for 1.5`.
fn raised_for_count_not_whole(found: &[u8]) -> Raised {
    Raised::syntax(26, 3, vec![String::from_utf8_lossy(found).into_owned()])
}

/// 28.1: a bare `LEAVE` found no repetitive loop or labeled block
/// instruction anywhere on the enclosing chain. No substitution.
fn raised_leave_no_loop() -> Raised {
    Raised::syntax(28, 1, Vec::new())
}

/// 28.2: a bare `ITERATE` found no repetitive loop anywhere on the
/// enclosing chain. No substitution.
fn raised_iterate_no_loop() -> Raised {
    Raised::syntax(28, 2, Vec::new())
}

/// 28.3: a named `LEAVE name` found nothing on the enclosing chain whose own
/// label (`DO LABEL`, or a controlled/`OVER` loop's own control variable)
/// matches `name` -- **an ordinary clause label never matches**, measured:
/// `outer: do i = 1 to 3` then `leave outer` is this, not a hit. `found` is
/// the symbol's own (already-upcased) spelling.
fn raised_leave_no_match(found: &[u8]) -> Raised {
    Raised::syntax(28, 3, vec![String::from_utf8_lossy(found).into_owned()])
}

/// 28.4: `ITERATE`'s own version of `raised_leave_no_match`.
fn raised_iterate_no_match(found: &[u8]) -> Raised {
    Raised::syntax(28, 4, vec![String::from_utf8_lossy(found).into_owned()])
}

/// 28.5: a named `ITERATE name` matched a block on the enclosing chain by
/// label, but that block is not a repetitive loop (a labelled `DO`/plain
/// block, or a `SELECT LABEL` -- `ITERATE` never accepts either, unlike
/// `LEAVE`). Measured: `do label x / say 1 / iterate x / end` gives this,
/// not 28.4, because `x` *did* match something.
fn raised_iterate_wrong_kind(found: &[u8]) -> Raised {
    Raised::syntax(28, 5, vec![String::from_utf8_lossy(found).into_owned()])
}

/// Whether `a < b`, numerically, through `rexx-num`'s own `compare_decoded`
/// rather than a hand-rolled sign comparison -- this crate's standing rule
/// against a second copy of a comparison `rexx-num` already owns
/// (`eval_compare`'s own doc comment states it for the twelve expression
/// operators; a controlled loop's own bound test and its `BY`'s sign are
/// the same rule applied to two `Number`s this crate already holds, not a
/// different one).
///
/// **The empty byte slices are provably unused, not a placeholder standing
/// in for something forgotten.** `compare_decoded`'s own body only reads
/// its `bytes` arguments when at least one side's `Option<Number>` is
/// `None` (the string-fallback and strict families) -- every call here
/// passes `Some` on both sides and a non-strict `CompareOp`, so the branch
/// that would read `a`/`b` is never taken. Passing real text would cost an
/// unwanted `Number::format` round-trip (rendering, then reparsing, which
/// is not exactly what a fresh comparison of the already-held `Number`s
/// would give at the boundary of a value too wide for `digits` to hold
/// exactly) for bytes the function does not use.
fn numeric_less(a: &Number, b: &Number, digits: u64, fuzz: u64) -> Result<bool, ArithError> {
    compare_decoded(b"", Some(a), b"", Some(b), digits, fuzz, CompareOp::Less)
}

/// Rounds `number` under `digits`, mirroring the oracle's own unary `+` at
/// a controlled loop's entry (F1, branch review, Important): `setup_
/// controlled` used to store `initial`/`to`/`by` as their exact parse, but
/// `ControlledLoop::setup` (`DoBlockComponents.cpp:126-166`, verified by
/// containment) rounds all three with `callOperatorMethod(OPERATOR_PLUS,
/// ...)` before the loop ever starts. Masked while `NUMERIC DIGITS` stays
/// constant (every later use re-rounds to the same width anyway) and wrong
/// the moment digits changes inside the loop body -- measured: `numeric
/// digits 3; do i = 1.23456 to 3; say i; numeric digits 9; end` is `1.23 /
/// 2.23 / 3.23` on the oracle (the header values were rounded once, at
/// entry, under digits 3, and stay that width even after digits widens);
/// this crate gave `1.23 / 2.23456 / 3.23456` before this fix (the exact
/// parse survived into the wider-digits passes untouched).
///
/// `Number::zero().add(number, digits)` rather than `Number::round_to`:
/// unary `+` is `0 + number` under the active digits (`eval.rs`'s own
/// `PrefixOp::Plus` arm does exactly this, though for an `ObjRef` this
/// function has no need to produce -- `LoopState::Controlled`'s own fields
/// are `Number`, not `ObjRef`), and `round_to`'s own doc comment
/// distinguishes the two: rounding alone is not what "prefix +" means, and
/// the oracle's citation is explicitly the operator, not a bare rounding.
/// A free function taking `&Number`/`u64` rather than a method, matching
/// `numeric_less`, just above: it needs no `&self` either, and reads only
/// what `rexx_num` already exposes as `pub`.
fn round_via_unary_plus(number: &Number, digits: u64) -> Result<Number, ArithError> {
    Number::zero().add(number, digits)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Activation;
    use crate::plan::{BodyKey, ProgramId};
    use rexx_parse::{Program, parse_program};

    /// Pushes a fresh top-level activation for `program`, the same setup
    /// `Interp::run` does, so a test can drive `step` through a live
    /// activation without the full instruction loop. Copied rather than
    /// shared, matching every other test module in this crate (`eval.rs`,
    /// `plan.rs`, `stem.rs` each keep their own).
    fn activate(interp: &mut Interp, program: Program) -> Rc<Program> {
        let program = Rc::new(program);
        let id = ProgramId(interp.programs.len());
        interp.programs.push(Rc::clone(&program));
        let plan = interp.plan_for(
            BodyKey {
                program: id,
                directive: None,
            },
            &program.main,
            &program.symbols,
        );
        let frame = interp.roots.push_slots(plan.len());
        let id = interp.next_activation_id();
        interp
            .activations
            .push(Activation::new(id, Rc::clone(&program), plan, frame));
        program
    }

    /// Parses `source`, activates it, and runs its whole body -- through
    /// `Interp::run_activation` itself since 4b's Task 7, not through a
    /// miniature of it.
    ///
    /// **This comment described the deleted miniature and was left in place;
    /// fix round 1's finding 4 corrects it.** Both of its claims are now
    /// false rather than merely stale. It is not "a miniature
    /// `run_activation`, through `run_bounded`": it is `run_activation`.
    /// And `slots` is not "an empty map throughout" -- `run_activation`
    /// builds its `Code` with `slots: &plan.by_symbol`, which
    /// `Plan::assign` populates, so every test in this module now runs
    /// through the plan's fast path rather than around it.
    ///
    /// That coverage shift is deliberate and is an improvement: these tests
    /// exercise what production runs. `eval.rs`, `stem.rs` and `plan.rs`
    /// still pass `&HashMap::new()` in their own helpers, so the by-name
    /// fallback keeps its expression-level coverage; what this file gains is
    /// whole-program coverage of the resolved path.
    fn run_source(interp: &mut Interp, source: &[u8]) -> Result<Option<ObjRef>, Failure> {
        let program = parse_program(source.to_vec()).expect("test program parses");
        let program = activate(interp, program);
        run_activated(interp, &program)
    }

    /// `run_source`'s second half, split out so `run_source_traced` can put a
    /// `TRACE` setting on the activation between the push and the run.
    ///
    /// **`run_activation` itself since 4b's Task 7, not a miniature of it.**
    /// This used to be a hand-rolled `run_bounded` loop that reproduced
    /// `run_activation`'s own `Flow` dispatch arm by arm, and it drifted
    /// exactly the way a second copy does: Task 6 had to teach it about
    /// `Flow::Signal`, and Task 7's condition traps -- which live in
    /// `run_activation`'s loop, one offer per activation -- did not exist
    /// here at all, so eleven trap tests written against this helper failed
    /// against a harness that could not trap while every one of the same
    /// programs matched the oracle byte for byte through `run_program`. A
    /// test harness that cannot reach the code under test is the sharpest
    /// version of a test that cannot fail.
    ///
    /// `activate` above already pushes exactly the activation `Interp::run`
    /// pushes, so there was never anything for the copy to supply. The
    /// activation is deliberately **not** popped afterwards, matching what
    /// this helper did before: several tests read `interp` after the run.
    fn run_activated(interp: &mut Interp, _program: &Program) -> Result<Option<ObjRef>, Failure> {
        interp.run_activation().map(Ended::value)
    }

    fn say_output(interp: &mut Interp, source: &[u8]) -> Vec<u8> {
        run_source(interp, source).expect("test program runs");
        std::mem::take(&mut interp.out)
    }

    /// `run_source`, with `TRACE R` already in force for the activation it
    /// pushes.
    ///
    /// **A helper rather than an `interp.set_trace_mode(...)` line before the
    /// call, which is how every one of these tests used to read.** Task 3
    /// moved `trace_mode` from `Interp` onto `Activation`, so there is
    /// nothing to set it on until an activation exists, and the activation is
    /// what `run_source` pushes. `TRACE R` is baked in rather than passed
    /// because every caller wants exactly that; a second setting gets its own
    /// helper rather than a parameter nobody varies.
    fn run_source_traced(interp: &mut Interp, source: &[u8]) -> Result<Option<ObjRef>, Failure> {
        let program = parse_program(source.to_vec()).expect("test program parses");
        let program = activate(interp, program);
        interp.set_trace_mode(mode_from_setting(b"r").expect("R is a valid TRACE setting"));
        run_activated(interp, &program)
    }

    fn say_output_traced(interp: &mut Interp, source: &[u8]) -> Vec<u8> {
        run_source_traced(interp, source).expect("test program runs");
        std::mem::take(&mut interp.out)
    }

    // ---- assignment ----

    #[test]
    fn assignment_to_a_variable_a_stem_and_a_compound() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(&mut interp, b"x = 5\nsay x"),
            b"5\n".to_vec(),
            "a simple variable"
        );

        let mut interp = Interp::new();
        assert_eq!(
            say_output(&mut interp, b"a. = 'wd'\nsay a.1"),
            b"wd\n".to_vec(),
            "a bare stem assignment, read through an unset tail"
        );

        let mut interp = Interp::new();
        assert_eq!(
            say_output(&mut interp, b"a.1 = 'one'\nsay a.1\nsay a.2"),
            b"one\nA.2\n".to_vec(),
            "a compound assignment mutates one tail and leaves the rest deriving its name"
        );
    }

    // ---- SAY ----

    #[test]
    fn say_of_each_value_kind_and_of_an_omitted_expression() {
        let mut interp = Interp::new();
        assert_eq!(say_output(&mut interp, b"say 'abc'"), b"abc\n".to_vec());
        assert_eq!(say_output(&mut interp, b"say 1 + 2"), b"3\n".to_vec());
        assert_eq!(
            say_output(&mut interp, b"say .nil"),
            b"The NIL object\n".to_vec()
        );
        // No expression at all: a blank line, not nothing.
        assert_eq!(say_output(&mut interp, b"say"), b"\n".to_vec());
    }

    // ---- DROP ----

    #[test]
    fn drop_of_a_variable_returns_it_to_unset() {
        // a = 5; drop a; say a -> A (the derived name, not left over from
        // before -- and never `.nil`, which is a value and not an absence).
        let mut interp = Interp::new();
        assert_eq!(
            say_output(&mut interp, b"a = 5\ndrop a\nsay a"),
            b"A\n".to_vec()
        );

        // The `.nil`-versus-dropped distinction `RootSet::clear_slot` exists
        // for: `x = .nil` renders "The NIL object"; a dropped variable
        // derives its own name instead, never that string.
        let mut interp = Interp::new();
        assert_eq!(
            say_output(&mut interp, b"y = .nil\ndrop y\nsay y"),
            b"Y\n".to_vec()
        );
    }

    #[test]
    fn drop_of_a_tail_tombstones_it_without_taking_the_default() {
        // u. = 'd'; u.1 = 'one'; drop u.1; say u.1 -> U.1 (tombstoned, not
        // falling back to the default); say u.2 -> d (an untouched tail
        // still does).
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"u. = 'd'\nu.1 = 'one'\ndrop u.1\nsay u.1\nsay u.2"
            ),
            b"U.1\nd\n".to_vec()
        );
    }

    #[test]
    fn drop_of_a_whole_stem_leaves_it_looking_untouched() {
        // x. = 'd'; x.1 = 'one'; drop x.; say x.1; say x. -> X.1, X. (exactly
        // what a never-touched stem would give).
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"x. = 'd'\nx.1 = 'one'\ndrop x.\nsay x.1\nsay x."
            ),
            b"X.1\nX.\n".to_vec()
        );
    }

    #[test]
    fn drop_of_the_indirect_form() {
        // A simple variable named by another's (upcased) value.
        let mut interp = Interp::new();
        assert_eq!(
            say_output(&mut interp, b"v = 'x'\nx = 1\ndrop (v)\nsay x"),
            b"X\n".to_vec(),
            "the wrapper's value is upcased before it names a variable"
        );

        // A whole stem, named indirectly.
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"v = 'A.'\na. = 'wd'\na.1 = 'one'\ndrop (v)\nsay a.1\nsay a."
            ),
            b"A.1\nA.\n".to_vec()
        );

        // One tail, named indirectly, joined-dots key taken verbatim rather
        // than re-resolved as source -- the discriminating transcript from
        // `drop_variable`'s own doc comment.
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"v = 'A.1.2'\na.1.2 = 'x'\ndrop (v)\nsay a.1.2"
            ),
            b"A.1.2\n".to_vec()
        );
    }

    /// Fix-round: the indirect form's value is a **subsidiary list**, not a
    /// single verbatim name. Every case here uses *set* targets (the trap
    /// the review named: an unset target's cleared slot and its own derived
    /// name render identically, so a test built on unset targets cannot
    /// tell "resolved the whole value as one name" apart from "split,
    /// validated and dropped two names" -- only a set value discriminates).
    #[test]
    fn drop_of_the_indirect_form_is_a_subsidiary_list_of_words() {
        // Two names, blank-separated, both set and both dropped.
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"a = 1\nb = 2\nv = 'a b'\ndrop (v)\nsay a\nsay b"
            ),
            b"A\nB\n".to_vec(),
            "a blank-separated list drops every word, not one name literally spelled 'A B'"
        );

        // A run of blanks between words, and leading/trailing blanks, both
        // collapse -- still exactly two words.
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"a = 1\nb = 2\nv = '  a    b  '\ndrop (v)\nsay a\nsay b"
            ),
            b"A\nB\n".to_vec()
        );

        // A tab (`'09'x`) separates two words exactly like a blank does.
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"a = 1\nb = 2\nv = 'a'||'09'x||'b'\ndrop (v)\nsay a\nsay b"
            ),
            b"A\nB\n".to_vec(),
            "a tab byte separates words the same way a blank does"
        );

        // A mix of shapes in one list: a whole stem and a simple variable.
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"a. = 'd'\na.1 = 'one'\nx = 5\nv = 'a. x'\ndrop (v)\nsay a.1\nsay x"
            ),
            b"A.1\nX\n".to_vec()
        );
    }

    #[test]
    fn drop_of_the_indirect_form_validates_every_word() {
        // A digit-led word: 31.2.
        let mut interp = Interp::new();
        let failure = run_source(&mut interp, b"v = '9'\ndrop (v)").unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (31, 2));
        assert_eq!(raised.additional, vec!["9".to_string()]);

        // A dot-led word: 31.3.
        let mut interp = Interp::new();
        let failure = run_source(&mut interp, b"v = '.x'\ndrop (v)").unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (31, 3));
        assert_eq!(raised.additional, vec![".x".to_string()]);

        // A parenthesised word: 20.928, not a second round of indirection --
        // proves the list is not recursive.
        let mut interp = Interp::new();
        let failure = run_source(&mut interp, b"w = 1\nv = '(w)'\ndrop (v)").unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (20, 928));
        assert_eq!(raised.additional, vec!["(w)".to_string()]);

        // **A newline does not separate.** It is not whitespace for this
        // purpose: the whole of `a`, the newline and `b` form ONE word, which
        // then fails the character-set check, and the reported name carries
        // the raw newline. Measured byte for byte against the oracle,
        // substitution included.
        //
        // This assertion is why `split_indirect_words` tests space and tab
        // explicitly instead of calling `is_ascii_whitespace`. Without it a
        // mutant that used `is_ascii_whitespace` passed all 79 tests, because
        // no other case here carries a newline, and the distinction rested
        // entirely on an end-to-end diff nobody re-runs. Carriage return,
        // form feed and vertical tab behave as the newline does.
        let mut interp = Interp::new();
        let failure = run_source(&mut interp, b"v = 'a'||'0a'x||'b'\ndrop (v)").unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (20, 928));
        assert_eq!(raised.additional, vec!["a\nb".to_string()]);
    }

    #[test]
    fn drop_of_the_indirect_form_validates_before_dropping_any_of_it() {
        // a=1; b=2; v='a 9 b'; drop (v) -> Error 31.2, and NEITHER a nor b
        // is dropped, even though `a` sits before the bad word -- measured
        // against the oracle (SIGNAL ON SYNTAX recovery there shows both
        // untouched). The whole list validates before any drop runs.
        let mut interp = Interp::new();
        let failure = run_source(&mut interp, b"a = 1\nb = 2\nv = 'a 9 b'\ndrop (v)").unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (31, 2));
        assert_eq!(raised.additional, vec!["9".to_string()]);

        // The activation is still on the stack (`run_source` does not pop
        // on error), so `a`'s slot is still directly inspectable.
        let a_slot = interp.slot_of(b"A");
        let frame = interp.activation().frame;
        let a_value = interp
            .roots
            .slot(frame, a_slot)
            .expect("a must still be set");
        assert_eq!(
            &*interp.to_text(a_value),
            b"1",
            "a must not have been dropped before the list's third word failed validation"
        );
    }

    #[test]
    fn drop_of_the_indirect_form_on_an_empty_or_blanks_only_value_is_a_no_op() {
        // Measured: `v=''`/`v='   '` both run clean under the oracle -- zero
        // words, nothing to validate or drop.
        let mut interp = Interp::new();
        assert_eq!(
            say_output(&mut interp, b"v = ''\ndrop (v)\nsay 'after'"),
            b"after\n".to_vec()
        );

        let mut interp = Interp::new();
        assert_eq!(
            say_output(&mut interp, b"v = '   '\ndrop (v)\nsay 'after'"),
            b"after\n".to_vec()
        );
    }

    // ---- NUMERIC ----

    #[test]
    fn numeric_digits_changes_rounding_and_resets_to_9_with_no_expression() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(&mut interp, b"numeric digits 3\nsay 1/3"),
            b"0.333\n".to_vec()
        );

        let mut interp = Interp::new();
        assert_eq!(
            say_output(&mut interp, b"numeric digits 3\nnumeric digits\nsay 1/3"),
            b"0.333333333\n".to_vec(),
            "NUMERIC DIGITS alone resets to the package default, 9"
        );
    }

    #[test]
    fn numeric_digits_reset_reports_a_conflict_exactly_as_if_9_were_typed() {
        // numeric digits 20; numeric fuzz 15; numeric digits -> 33.1, ("9")
        // rejected against the still-15 fuzz -- measured against the oracle.
        let mut interp = Interp::new();
        let failure = run_source(
            &mut interp,
            b"numeric digits 20\nnumeric fuzz 15\nnumeric digits",
        )
        .unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (33, 1));
        assert_eq!(raised.additional, vec!["9".to_string(), "15".to_string()]);
    }

    #[test]
    fn numeric_fuzz_changes_comparison_and_resets_to_0_with_no_expression() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"numeric digits 5\nnumeric fuzz 3\nsay (1.001 = 1)"
            ),
            b"1\n".to_vec()
        );

        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"numeric digits 5\nnumeric fuzz 3\nnumeric fuzz\nsay (1.001 = 1)"
            ),
            b"0\n".to_vec(),
            "NUMERIC FUZZ alone resets to the package default, 0"
        );
    }

    #[test]
    fn numeric_form_scientific_engineering_and_default() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(&mut interp, b"numeric form engineering\nsay 1e10 + 0"),
            b"10E+9\n".to_vec()
        );

        let mut interp = Interp::new();
        assert_eq!(
            say_output(&mut interp, b"numeric form scientific\nsay 1e10 + 0"),
            b"1E+10\n".to_vec()
        );

        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"numeric form engineering\nnumeric form\nsay 1e10 + 0"
            ),
            b"1E+10\n".to_vec(),
            "NUMERIC FORM alone resets to the package default, SCIENTIFIC"
        );
    }

    #[test]
    fn numeric_form_value_spellings() {
        // The keyword VALUE and the implicit `(expr)` spelling both set the
        // same setting.
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"numeric form value 'ENGINEERING'\nsay 1e10 + 0"
            ),
            b"10E+9\n".to_vec()
        );

        let mut interp = Interp::new();
        assert_eq!(
            say_output(&mut interp, b"numeric form ('ENGINEERING')\nsay 1e10 + 0"),
            b"10E+9\n".to_vec()
        );
    }

    #[test]
    fn numeric_form_value_is_not_case_insensitive() {
        // set_form_str's own rule: the runtime VALUE path does no
        // uppercasing -- measured, `numeric form value 'engineering'` is
        // 25.11 under the oracle.
        let mut interp = Interp::new();
        let failure = run_source(&mut interp, b"numeric form value 'engineering'").unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (25, 11));
    }

    /// F2 (branch review, Important): `NUMERIC DIGITS`/`FUZZ`/`FORM VALUE`
    /// trace `>K>` when an expression is present, and a bare `NUMERIC
    /// DIGITS`/`FUZZ` (no expression) traces nothing -- both measured
    /// against the oracle (`numeric_operand`'s own doc comment has the two
    /// transcripts this mirrors). Mutation killed: removing either
    /// `trace_keyword` call (`numeric_operand`'s own, shared by `DIGITS`/
    /// `FUZZ`, or `FormValue`'s own inline one) drops that setting's own
    /// `>K>` line from `interp.trace` entirely, verified by reverting each
    /// in turn.
    #[test]
    fn numeric_digits_fuzz_and_form_value_trace_k_only_with_an_expression() {
        let mut interp = Interp::new();
        say_output_traced(
            &mut interp,
            b"numeric digits 9\nnumeric fuzz 2\nnumeric form value 'SCIENTIFIC'",
        );
        assert_eq!(
            interp.trace,
            b"     1 *-* numeric digits 9\n       >K>   \"DIGITS\" => \"9\"\n     \
              2 *-* numeric fuzz 2\n       >K>   \"FUZZ\" => \"2\"\n     \
              3 *-* numeric form value 'SCIENTIFIC'\n       >K>   \"FORM\" => \"SCIENTIFIC\"\n"
                .to_vec()
        );

        let mut interp = Interp::new();
        say_output_traced(
            &mut interp,
            b"numeric digits 3\nnumeric digits\nnumeric fuzz",
        );
        assert_eq!(
            interp.trace,
            b"     1 *-* numeric digits 3\n       >K>   \"DIGITS\" => \"3\"\n     \
              2 *-* numeric digits\n     3 *-* numeric fuzz\n"
                .to_vec(),
            "a bare NUMERIC DIGITS/FUZZ, with no expression, traces nothing"
        );
    }

    // ---- EXIT ----

    #[test]
    fn exit_with_and_without_an_expression() {
        let mut interp = Interp::new();
        let value = run_source(&mut interp, b"exit").expect("bare exit runs");
        assert_eq!(value, None);

        let mut interp = Interp::new();
        let value = run_source(&mut interp, b"exit 42").expect("exit with a result runs");
        let value = value.expect("EXIT 42 carries a result");
        assert_eq!(&*interp.to_text(value), b"42");
    }

    #[test]
    fn exit_code_for_converts_the_result_the_way_the_oracle_does() {
        // The whole transcript this task's report re-verifies against the
        // oracle: a bare EXIT and a huge, fractional or non-numeric result
        // all leave the code at 0; a literal in i32 range converts exactly,
        // and an arithmetic result (EXIT's own unary minus counts) is
        // already rounded to the active NUMERIC DIGITS by the time it gets
        // here, which is where the negative-vs-positive asymmetry the report
        // measured against the oracle comes from.
        let mut interp = Interp::new();
        assert_eq!(interp.exit_code_for(None), 0, "a bare EXIT");

        let value = run_source(&mut interp, b"exit 2147483647")
            .unwrap()
            .expect("a result");
        assert_eq!(
            interp.exit_code_for(Some(value)),
            2147483647,
            "INT32_MAX exactly"
        );

        let value = run_source(&mut interp, b"exit 2147483648")
            .unwrap()
            .expect("a result");
        assert_eq!(
            interp.exit_code_for(Some(value)),
            0,
            "one past INT32_MAX falls back to 0"
        );

        let value = run_source(&mut interp, b"exit 5.9")
            .unwrap()
            .expect("a result");
        assert_eq!(interp.exit_code_for(Some(value)), 0, "fractional");

        let value = run_source(&mut interp, b"exit 5.0")
            .unwrap()
            .expect("a result");
        assert_eq!(
            interp.exit_code_for(Some(value)),
            5,
            "a whole number spelled with a point"
        );

        let value = run_source(&mut interp, b"exit 'abc'")
            .unwrap()
            .expect("a result");
        assert_eq!(interp.exit_code_for(Some(value)), 0, "non-numeric");

        // The asymmetry: -2147483647 is 0 - 2147483647, rounded to the
        // *active* DIGITS (9 by default) the moment it is created, landing
        // one past INT32_MIN; raising DIGITS before the subtraction removes
        // the rounding and the failure with it.
        let value = run_source(&mut interp, b"exit -2147483647")
            .unwrap()
            .expect("a result");
        assert_eq!(
            interp.exit_code_for(Some(value)),
            0,
            "rounded to 9 digits at creation, one past INT32_MIN"
        );

        let value = run_source(&mut interp, b"numeric digits 20\nexit -2147483647")
            .unwrap()
            .expect("a result");
        assert_eq!(
            interp.exit_code_for(Some(value)),
            -2147483647,
            "no rounding at DIGITS 20, exact"
        );
    }

    // ---- LABEL and NOP ----

    #[test]
    fn a_label_is_a_traced_no_op() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(&mut interp, b"here: say 'hit'"),
            b"hit\n".to_vec(),
            "the label itself produces no output of its own"
        );
    }

    #[test]
    fn nop_is_a_no_op() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(&mut interp, b"nop\nsay 'after'"),
            b"after\n".to_vec()
        );
    }

    // ---- IF/THEN/ELSE ----

    /// The discriminating shape for a wrong `false_target`/`then_exit`: a
    /// true condition whose `ELSE` branch has a **different** side effect
    /// than the `THEN` branch. A version that lets the true path fall
    /// through into the `ELSE` marker without skipping it (the naive
    /// fallthrough this task's whole design exists to avoid -- see
    /// `run_bounded`'s doc comment) runs *both* assignments and prints
    /// `abXc`. A version that never runs the `ELSE` branch on the false path
    /// at all prints `aXc` for the companion case below. Only the correct
    /// wiring prints `abc` and `aXc` respectively.
    #[test]
    fn if_then_else_runs_exactly_one_branch() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"a = 'a'\nif 1 = 1 then a = a || 'b'\nelse a = a || 'X'\na = a || 'c'\nsay a"
            ),
            b"abc\n".to_vec(),
            "true: runs the THEN branch and skips the ELSE branch entirely"
        );

        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"a = 'a'\nif 1 = 0 then a = a || 'b'\nelse a = a || 'X'\na = a || 'c'\nsay a"
            ),
            b"aXc\n".to_vec(),
            "false: runs the ELSE branch and skips the THEN branch entirely"
        );
    }

    #[test]
    fn if_then_with_no_else_falls_through_on_false() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"a = 'a'\nif 1 = 0 then a = a || 'b'\na = a || 'c'\nsay a"
            ),
            b"ac\n".to_vec()
        );
    }

    /// The `ast.rs`/`block.rs`-derived discriminating case: an `IF`/`ELSE
    /// IF`/`ELSE` chain (no `DO`, which Task 11 owns) where each link's
    /// `false_target` is the next link's own condition and the whole
    /// chain's resume point sits after all of them. Run once per branch so
    /// a wrong `false_target` (skipping or re-testing a link) and a wrong
    /// `then_exit`/resume (landing inside a later link, matching
    /// `if_else_chain.rex`'s own framing) are both visible.
    #[test]
    fn if_else_if_chain_takes_exactly_one_link() {
        let source = |n: &str| -> Vec<u8> {
            format!(
                "n = {n}\na = ''\nif n = 1 then a = a || 'one'\nelse if n = 2 then a = a || 'two'\nelse if n = 3 then a = a || 'three'\nelse a = a || 'other'\na = a || '-after'\nsay a"
            )
            .into_bytes()
        };
        for (n, expected) in [
            ("1", "one-after\n"),
            ("2", "two-after\n"),
            ("3", "three-after\n"),
            ("4", "other-after\n"),
        ] {
            let mut interp = Interp::new();
            assert_eq!(
                say_output(&mut interp, &source(n)),
                expected.as_bytes().to_vec(),
                "n = {n}"
            );
        }
    }

    #[test]
    fn if_condition_that_is_not_0_or_1_raises_34_1() {
        let mut interp = Interp::new();
        let failure = run_source(&mut interp, b"if 'x' then nop").unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (34, 1));
        assert_eq!(raised.additional, vec!["x".to_string()]);
    }

    /// A comma list is 34.6 regardless of which element fails, never 34.1 --
    /// re-checking `eval_logical_list`'s own result would misreport it.
    /// Measured against the oracle (brief's own transcript).
    #[test]
    fn if_condition_that_is_a_comma_list_raises_34_6_not_34_1() {
        let mut interp = Interp::new();
        let failure = run_source(&mut interp, b"if 'x', 1 then nop").unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (34, 6));
        assert_eq!(raised.additional, vec!["x".to_string()]);
    }

    #[test]
    fn a_true_comma_list_condition_is_an_and_of_its_parts() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(&mut interp, b"if 1, 1 then say 'hit'\nelse say 'miss'"),
            b"hit\n".to_vec()
        );
        let mut interp = Interp::new();
        assert_eq!(
            say_output(&mut interp, b"if 1, 0 then say 'hit'\nelse say 'miss'"),
            b"miss\n".to_vec()
        );
    }

    // ---- SELECT / WHEN ----

    #[test]
    fn select_when_runs_exactly_the_first_matching_when() {
        let source = |n: &str| -> Vec<u8> {
            format!(
                "n = {n}\nr = ''\nselect\n  when n = 1 then r = r || 'one'\n  when n = 2 then r = r || 'two'\n  when n = 3 then r = r || 'three'\n  otherwise r = r || 'other'\nend\nr = r || '-after'\nsay r"
            )
            .into_bytes()
        };
        for (n, expected) in [
            ("1", "one-after\n"),
            ("2", "two-after\n"),
            ("3", "three-after\n"),
            ("4", "other-after\n"),
        ] {
            let mut interp = Interp::new();
            assert_eq!(
                say_output(&mut interp, &source(n)),
                expected.as_bytes().to_vec(),
                "n = {n}"
            );
        }
    }

    /// The `select_when_bodies.rex` discriminating shape, reproduced without
    /// `DO` (Task 11's): each `WHEN`'s consequence is a nested `SELECT`
    /// spanning several flat instructions of its own
    /// (`Select`/`When`/`Then`/two assignments/`End`), so a wrong exit that
    /// lands even one instruction into a *later* `WHEN`'s span, rather than
    /// cleanly past the whole outer `SELECT`, shows up as an extra or
    /// missing accumulator update. Run for every `n` so a wrong exit wired
    /// to (for instance) always resume after the *first* `WHEN` would still
    /// be caught on `n = 2` or `n = 3`.
    #[test]
    fn select_when_wrong_exit_would_land_in_a_later_whens_multi_instruction_body() {
        let source = |n: &str| -> Vec<u8> {
            format!(
                "n = {n}\nr = ''\nselect\n  when n = 1 then select\n    when 1 = 1 then r = r || 'w1a'\n    otherwise nop\n  end\n  when n = 2 then select\n    when 1 = 1 then r = r || 'w2a'\n    otherwise nop\n  end\n  when n = 3 then select\n    when 1 = 1 then r = r || 'w3a'\n    otherwise nop\n  end\n  otherwise r = r || 'w4a'\nend\nr = r || '-done'\nsay r"
            )
            .into_bytes()
        };
        for (n, expected) in [
            ("1", "w1a-done\n"),
            ("2", "w2a-done\n"),
            ("3", "w3a-done\n"),
            ("4", "w4a-done\n"),
        ] {
            let mut interp = Interp::new();
            assert_eq!(
                say_output(&mut interp, &source(n)),
                expected.as_bytes().to_vec(),
                "n = {n}: exactly one nested SELECT's body ran, and nothing else did"
            );
        }
    }

    /// `when 1 = 1 then` followed immediately by `when 2 = 2 then n = 42` is
    /// the second `WHEN`'s absorption into the first's (empty) consequence
    /// (`ast.rs`'s own doc comment on `Select::whens`): the second `WHEN` is
    /// never collected, and its own `exit` is permanently `None`. Measured
    /// against the oracle: prints `0`, rc 0 -- the absorbed `WHEN`'s own
    /// condition being true (`2 = 2`) must not run `n = 42`, and `OTHERWISE`
    /// must not run either, because one true `WHEN` (the outer one) already
    /// ended the whole `SELECT`.
    #[test]
    fn an_absorbed_when_runs_neither_its_own_consequence_nor_otherwise() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"n = 0\nselect\n  when 1 = 1 then\n    when 2 = 2 then n = 42\n  otherwise\n    n = 99\nend\nsay n"
            ),
            b"0\n".to_vec()
        );
    }

    /// The true-condition-variant is accepted at rc 0 (`ast.rs:776`, and the
    /// brief's own transcript) -- the *false*-condition variant is a
    /// separate, upstream oracle segfault (SF #2018) and is deliberately not
    /// probed here.
    #[test]
    fn when_absorbing_a_when_parses_and_runs_at_rc_0() {
        let mut interp = Interp::new();
        assert_eq!(
            run_source(
                &mut interp,
                b"select\n  when 1 = 1 then\n    when 2 = 2 then nop\nend"
            )
            .expect("accepted, rc 0"),
            None
        );
    }

    /// **Critical, found by review, not by this task's own probes.** The
    /// mutation this kills is exactly the one the old `Ok(Flow::Next)`
    /// arm *was*: treating an absorbed `WHEN`'s own condition as never
    /// evaluated at all, rather than evaluated and discarded. A version
    /// that reverts `InstructionKind::When`'s arm to a bare `Ok(Flow::
    /// Next)` makes this test hang around `run_source` returning `Ok`
    /// (prints `after`, rc 0) where the oracle raises 42.3 at rc 214 --
    /// `an_absorbed_when_runs_neither_its_own_consequence_nor_otherwise`,
    /// just above, cannot distinguish the two models (both give `n = 0`
    /// for a side-effect-free true condition), which is exactly why every
    /// probe before this review used one and missed this.
    #[test]
    fn an_absorbed_whens_raising_condition_escapes_even_though_its_own_consequence_never_runs() {
        let mut interp = Interp::new();
        let failure = run_source(
            &mut interp,
            b"select\n  when 1 = 1 then\n    when 1 / 0 then nop\n  otherwise nop\nend\nsay 'after'",
        )
        .unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (42, 3));
    }

    /// The companion half: a **true** absorbed condition with a printable
    /// side effect still never runs its own consequence (matching the
    /// existing `n = 0` test, restated with `SAY` so a wrong "the
    /// absorbed WHEN's branch is taken" model would be caught by output
    /// content rather than only by a variable's final value) -- measured,
    /// this task's own report: `ABSORBED-RAN` never prints, only `after`.
    #[test]
    fn an_absorbed_whens_true_condition_still_never_runs_its_own_consequence() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"select\n  when 1 = 1 then\n    when 2 = 2 then say 'ABSORBED-RAN'\nend\nsay 'after'"
            ),
            b"after\n".to_vec(),
            "a reverted fix would print ABSORBED-RAN first"
        );
    }

    /// F3, found by review: unlike a plain `WHEN`'s absorbed form, a
    /// `WHEN CASE`'s own absorbed form branches to its own `false_target`
    /// on a false match. The mutation this kills: reverting
    /// `InstructionKind::WhenCase`'s arm to the old evaluate-and-discard
    /// shape (matching `When`'s own, still correct for `When`) makes this
    /// program print only `after`, where the oracle -- and this fix --
    /// print `O` then `after`. Verified by mutation: reverting made this
    /// test fail with exactly that wrong output, while
    /// `an_absorbed_whens_true_condition_still_never_runs_its_own_
    /// consequence` (a `WHEN`, not a `WHEN CASE`) stayed green, confirming
    /// the fix is scoped to `WhenCase` alone.
    #[test]
    fn an_absorbed_whencases_false_condition_branches_to_its_own_false_target() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"select case 2\n  when 2 then\n    when 3 then nop\n  otherwise say 'O'\nend\nsay 'after'"
            ),
            b"O\nafter\n".to_vec()
        );
    }

    /// The companion true-match case, matching the coordinator's own
    /// phrase "matches on both sides": a true absorbed `WhenCase` still
    /// never runs its own consequence, exactly like a plain `WHEN`'s own
    /// true-absorbed case -- this is *not* new behaviour F3 introduced, it
    /// is what this crate already did before the fix, re-pinned here so a
    /// future change to the false-path fix cannot silently start running
    /// the true path's own consequence too.
    #[test]
    fn an_absorbed_whencases_true_condition_still_never_runs_its_own_consequence() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"select case 2\n  when 2 then\n    when 2 then say 'INNER'\n  otherwise say 'O'\nend\nsay 'after'"
            ),
            b"after\n".to_vec()
        );
    }

    #[test]
    fn select_with_no_when_true_and_no_otherwise_raises_7_3() {
        let mut interp = Interp::new();
        let failure = run_source(&mut interp, b"select\n  when 1 = 0 then nop\nend").unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (7, 3));
        assert_eq!(raised.additional, Vec::<String>::new());
    }

    /// F3's own perimeter, found by review: an absorbed `WhenCase`'s
    /// false-branch escape landing directly on `END` reports 7.3 at the
    /// escape's own residual indent (4, the absorbed condition's own
    /// depth of 6 minus 2), not `END`'s own lexical `static_indent` (0,
    /// top-level `SELECT`). The mutation this kills: removing the
    /// `self.pending_escape_indent = Some(...)` assignment in
    /// `InstructionKind::WhenCase`'s own false-branch arm (or reverting
    /// `record_failure_site` to ignore it) makes this test's own `indent`
    /// read back `0` instead of `4`, while `select_with_no_when_true_and_
    /// no_otherwise_raises_7_3`, just above -- the *ordinary*, non-
    /// absorbed 7.3 path, unaffected by this fix -- stays green either
    /// way, confirming the fix is scoped to the escape path alone.
    #[test]
    fn an_absorbed_whencases_escaping_false_branch_reports_end_at_its_own_residual_indent() {
        let mut interp = Interp::new();
        let failure = run_source(
            &mut interp,
            b"select case 2\n  when 2 then\n    when 3 then nop\nend\nsay 'after'",
        )
        .unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (7, 3));
        let FailureSite { indent, .. } = interp.failure_site.expect("a site was resolved");
        assert_eq!(indent, 4);
    }

    /// The same shape, one `DO` level deeper -- `indent_offset`'s own doc
    /// comment (`lib.rs`) has the full argument for why the answer stays
    /// `6`, not `8`: the offset past an *ordinary* `SELECT`-level
    /// construct's own depth is the constant `4`, not a function of how
    /// deep the absorbed condition itself sits, and both grow by the same
    /// amount together under nesting. The mutation this kills: reverting
    /// `indent_offset`'s own assignment to `self.clause_state.current_value_indent.
    /// saturating_sub(2)` (an earlier, wrong version of this same fix)
    /// makes this test read back `6` (`8 - 2`) instead of `4`, while the
    /// non-nested version just above still reads back the right answer
    /// either way (`6 - 2` and the constant `4` coincide at the top
    /// level) -- this is the one test that distinguishes them.
    #[test]
    fn an_absorbed_whencases_escape_to_end_reports_the_same_constant_offset_nested() {
        let mut interp = Interp::new();
        let failure = run_source(
            &mut interp,
            b"do i = 1 to 1\n  select case 2\n    when 2 then\n      when 3 then nop\n  end\nend\nsay 'after'",
        )
        .unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (7, 3));
        let FailureSite { indent, .. } = interp.failure_site.expect("a site was resolved");
        assert_eq!(indent, 6);
    }

    /// **F-EX1, Important, found by the whole-branch review, not by this
    /// task's own probes.** An absorbed `WhenCase`'s own false-branch
    /// escape landing on `OTHERWISE` used to leave `Select`'s own arm
    /// through a bare `Flow::Goto` that `leave_select` had no way to
    /// recognise, so `OTHERWISE`'s own body ran under whichever *outer*
    /// construct received that `Goto` -- with no `SELECT` frame on the
    /// search a `LEAVE` naming the enclosing `SELECT LABEL` needs to find.
    /// The mutation this kills: reverting the escape-redirect check in
    /// `Select`'s own arm (the `if let Flow::Goto(target) = flow && *
    /// otherwise == Some(target)` branch, calling `run_otherwise` instead
    /// of forwarding the bare `Goto`) makes `leave s` search *outward* from
    /// outside this `SELECT` and find nothing, raising 28.3 at rc 228
    /// instead of resuming past the `SELECT` cleanly -- reproduced by
    /// actually reverting it (this task's report has the transcript).
    #[test]
    fn an_absorbed_whencases_escape_to_otherwise_still_finds_the_enclosing_selects_own_label() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"select label s case 2\n  when 2 then\n    when 3 then nop\n  otherwise say 'O'\n  \
                  leave s\nend\nsay 'after'"
            ),
            b"O\nafter\n".to_vec()
        );
    }

    /// The companion half of F-EX1: a **named `ITERATE`** inside the same
    /// escaped `OTHERWISE`, naming the enclosing `SELECT LABEL`, is 28.5
    /// (matches the name, but a `SELECT` is never a repetitive block) --
    /// not 28.4 (no match at all), which is what it read as before this
    /// fix, because the search never reached `leave_select`'s own name
    /// check at all. Distinguishes "the frame is restored" from "the frame
    /// is restored, and the specific consumption rule inside it still
    /// applies", which the `LEAVE` test above alone cannot.
    #[test]
    fn an_absorbed_whencases_escape_to_otherwise_reports_a_named_iterate_as_28_5_not_28_4() {
        let mut interp = Interp::new();
        let failure = run_source(
            &mut interp,
            b"select label s case 2\n  when 2 then\n    when 3 then nop\n  otherwise say 'O'\n  \
              iterate s\nend\nsay 'after'",
        )
        .unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (28, 5));
    }

    #[test]
    fn select_with_no_when_true_and_an_otherwise_runs_it_without_error() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"select\n  when 1 = 0 then nop\n  otherwise say 'lo'\nend"
            ),
            b"lo\n".to_vec()
        );
    }

    #[test]
    fn when_condition_that_is_not_0_or_1_raises_34_2() {
        let mut interp = Interp::new();
        let failure = run_source(&mut interp, b"select\n  when 'x' then nop\nend").unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (34, 2));
        assert_eq!(raised.additional, vec!["x".to_string()]);
    }

    /// **The coordinator's own finding, fixed after the first round.** A
    /// `WHEN`'s own `step` arm is a pure no-op (`Select`'s own arm reads it
    /// as data instead), so a raise while evaluating its *condition* never
    /// went through a `step_in_temps_frame` call for the `WHEN` itself --
    /// the first version of this task attributed it to the enclosing
    /// `SELECT`, wrong clause *and* wrong line, measured against the
    /// oracle. `record_failure_site`'s own doc comment on `Select`'s call
    /// sites has the fix. Checks `failure_site` directly (line and clause
    /// text), which `raised.number`/`.sub` alone -- every other test in
    /// this file -- cannot: both were already unaffected by the bug, since
    /// the bug is entirely in *which clause* gets echoed, not in which
    /// condition failed.
    #[test]
    fn a_when_conditions_own_failure_is_attributed_to_the_when_not_the_select() {
        let mut interp = Interp::new();
        run_source(&mut interp, b"select\nwhen 'x' then nop\nend").unwrap_err();
        let FailureSite { line, text, .. } = interp
            .failure_site
            .expect("a raised condition always resolves a site when source is Some");
        assert_eq!(line, 2, "the WHEN's own line, not the SELECT's (line 1)");
        assert_eq!(
            text,
            b"when 'x' ".to_vec(),
            "the WHEN's own clause text, not \"select\""
        );
    }

    /// The line has to move with the failing `WHEN`, not merely differ from
    /// the `SELECT`'s -- a test whose expected line is the first `WHEN`'s
    /// cannot tell a correct resolution from one that defaults to
    /// "whichever `WHEN` this loop happens to be looking at first".
    #[test]
    fn the_second_of_two_whens_own_failure_moves_the_line_with_it() {
        let mut interp = Interp::new();
        run_source(
            &mut interp,
            b"select\nwhen 1 = 0 then nop\nwhen 'x' then nop\nend",
        )
        .unwrap_err();
        let FailureSite { line, text, .. } = interp.failure_site.expect("a site was resolved");
        assert_eq!(line, 3, "the second WHEN's own line, not the first's (2)");
        assert_eq!(text, b"when 'x' ".to_vec());
    }

    /// A `SELECT CASE` expression that itself raises is the `SELECT`'s own
    /// clause -- confirmed against the oracle rather than assumed, since
    /// `case` is evaluated directly inside `Select`'s own `step` call and
    /// the coordinator asked this be checked, not taken on faith.
    #[test]
    fn a_select_cases_own_expression_failure_is_attributed_to_the_select() {
        let mut interp = Interp::new();
        run_source(&mut interp, b"select case (1/0)\nwhen 1 then nop\nend").unwrap_err();
        let FailureSite { line, text, .. } = interp.failure_site.expect("a site was resolved");
        assert_eq!(line, 1);
        assert_eq!(text, b"select case (1/0)".to_vec());
    }

    /// A `WhenCase` value expression that raises is the `WHEN`'s own
    /// clause, the same rule as a plain `WHEN`'s condition -- both go
    /// through `Select`'s own explicit-match-and-record path, never
    /// through `step_in_temps_frame` for the `When`/`WhenCase` node itself.
    #[test]
    fn a_whencase_values_own_failure_is_attributed_to_the_when_not_the_select() {
        let mut interp = Interp::new();
        run_source(&mut interp, b"select case 1\nwhen (1/0) then nop\nend").unwrap_err();
        let FailureSite { line, text, .. } = interp.failure_site.expect("a site was resolved");
        assert_eq!(line, 2);
        assert_eq!(text, b"when (1/0) ".to_vec());
    }

    /// A raise inside an `OTHERWISE` branch was already correct before this
    /// round's fix (`OTHERWISE`'s own body runs through the outer loop's
    /// ordinary `step_in_temps_frame`, never through `Select`'s own
    /// explicit-match path), and stays that way -- checked because the
    /// coordinator asked for it explicitly, not assumed from the `WHEN` fix.
    #[test]
    fn a_raise_inside_an_otherwise_branch_is_attributed_to_its_own_clause() {
        let mut interp = Interp::new();
        run_source(
            &mut interp,
            b"select\nwhen 1 = 0 then nop\notherwise\n  say 1/0\nend",
        )
        .unwrap_err();
        let FailureSite { line, text, .. } = interp.failure_site.expect("a site was resolved");
        assert_eq!(line, 4);
        assert_eq!(text, b"say 1/0".to_vec());
    }

    /// A raise inside a matched `WHEN`'s **body** (not its condition) has to
    /// go through `run_bounded`'s own `step_in_temps_frame` calls with
    /// `source` actually threaded through, which is a different path from
    /// every other test in this section: those all check a condition/value
    /// expression `Select`'s own arm evaluates directly, and
    /// `a_raise_inside_an_otherwise_branch_is_attributed_to_its_own_clause`
    /// runs through the *outer* loop's `step_in_temps_frame`, never through a
    /// `run_bounded` nested inside `If`/`Select`'s own arm. That last
    /// distinction matters and an earlier wording of it was wrong: in the
    /// test harness `run_source` routes everything through its own top-level
    /// `run_bounded` with `source` supplied directly, so "never through
    /// `run_bounded` at all" is true of the production outer loop only.
    ///
    /// This is round 1's own defect class -- an error escaping a nested
    /// `run_bounded` call misattributed to the enclosing construct -- and no
    /// existing test exercises the path that would regress if `source`
    /// stopped being threaded into `run_bounded`. Confirmed by mutation, not
    /// assumed: passing `None` in place of `source` at the two call sites,
    /// which are `If`'s and `Select`'s and not two of `Select`'s own, made
    /// this test and the `IF` one below fail while leaving all 102
    /// pre-existing tests green. Restored immediately after.
    #[test]
    fn a_raise_inside_a_matched_whens_body_is_attributed_to_its_own_clause() {
        let mut interp = Interp::new();
        run_source(&mut interp, b"select\nwhen 1 = 1 then\n  say 1/0\nend").unwrap_err();
        let FailureSite { line, text, .. } = interp.failure_site.expect("a site was resolved");
        assert_eq!(
            line, 3,
            "the WHEN's own body clause, not the SELECT's (line 1)"
        );
        assert_eq!(text, b"say 1/0".to_vec());
    }

    /// The `IF` analogue of the `WHEN`-body test above: a raise inside the
    /// matched `THEN` branch's own body, which likewise only ever reaches
    /// `step_in_temps_frame` through `run_bounded`. Confirmed by the same
    /// mutation (`None` for `source` at `If`'s own `run_bounded` call site
    /// made this fail too, alongside the `WHEN`-body test, both restored
    /// after).
    #[test]
    fn a_raise_inside_an_ifs_then_body_is_attributed_to_its_own_clause() {
        let mut interp = Interp::new();
        run_source(&mut interp, b"if 1 = 1 then\n  say 1/0").unwrap_err();
        let FailureSite { line, text, .. } = interp.failure_site.expect("a site was resolved");
        assert_eq!(
            line, 2,
            "the THEN branch's own body clause, not the IF's (line 1)"
        );
        assert_eq!(text, b"say 1/0".to_vec());
    }

    // ---- SELECT CASE / WhenCase ----

    /// **The central `WhenCase` rule.** `select case 2` / `when 1, 2 then`
    /// matches (an OR of `==`), while a plain `select` / `when 1, 2` on the
    /// same non-logical value is 34.6 (an AND, each element checked for
    /// `0`/`1`) -- the two commas parse into the same-looking node and mean
    /// opposites (`ast.rs:801-815`, and the brief's own framing).
    #[test]
    fn whencase_comma_is_an_or_of_equals_the_opposite_of_a_plain_whens_and() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"select case 2\n  when 1, 2 then say 'hit'\n  otherwise say 'miss'\nend"
            ),
            b"hit\n".to_vec(),
            "SELECT CASE: an OR of == comparisons"
        );

        let mut interp = Interp::new();
        let failure = run_source(&mut interp, b"select\n  when 1, 2 then nop\nend").unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!(
            (raised.number, raised.sub),
            (34, 6),
            "plain SELECT: 2 is not a logical value, an AND-with-a-check, not an OR-of-=="
        );
        assert_eq!(raised.additional, vec!["2".to_string()]);
    }

    /// `==` is strict: byte-for-byte, no padding, no numeric awareness.
    /// Measured (D15's own example, restated in the brief): `'007'` does not
    /// match `when 7`, because the two are not byte-identical.
    #[test]
    fn select_case_compares_with_strict_equality_not_numeric_equality() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"select case '007'\n  when 7 then say 'hit'\n  otherwise say 'miss'\nend"
            ),
            b"miss\n".to_vec()
        );

        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"select case 7\n  when 7 then say 'hit'\n  otherwise say 'miss'\nend"
            ),
            b"hit\n".to_vec()
        );
    }

    #[test]
    fn select_case_evaluates_its_own_expression_and_runs_exactly_the_matching_when() {
        let source = |v: &str| -> Vec<u8> {
            format!("select case {v}\n  when 1 then say 'one'\n  when 2 then say 'two'\n  otherwise say 'other'\nend")
                .into_bytes()
        };
        for (v, expected) in [("1", "one\n"), ("2", "two\n"), ("3", "other\n")] {
            let mut interp = Interp::new();
            assert_eq!(
                say_output(&mut interp, &source(v)),
                expected.as_bytes().to_vec(),
                "v = {v}"
            );
        }
    }

    // ---- SELECT with a LABEL, and a nested SELECT/IF ----

    #[test]
    fn a_labelled_select_with_otherwise_runs_normally() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"select label s\n  when 1 = 0 then nop\n  otherwise say 'lo'\nend"
            ),
            b"lo\n".to_vec()
        );
    }

    #[test]
    fn a_select_nested_inside_an_ifs_then_branch_is_fully_resolved_before_resuming() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"a = 'a'\nif 1 = 1 then select\n  when 1 = 1 then a = a || 'b'\nend\na = a || 'c'\nsay a"
            ),
            b"abc\n".to_vec()
        );
    }

    // ---- DO/LOOP: every LoopKind ----

    #[test]
    fn a_simple_do_block_runs_its_body_exactly_once_with_no_control() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(&mut interp, b"do\nsay 'once'\nend"),
            b"once\n".to_vec()
        );
    }

    /// `LOOP` alone is `DO FOREVER`; `DO` alone is a block, not a loop
    /// (`ast.rs`'s own doc comment on `LoopKind::Simple`/`Forever`) --
    /// proven here by the fact that only the `LOOP` form is stopped by a
    /// bare `LEAVE`.
    #[test]
    fn loop_alone_is_forever_and_do_alone_is_a_block_not_a_loop() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"n = 0\nloop\nn = n + 1\nif n = 3 then leave\nend\nsay n"
            ),
            b"3\n".to_vec()
        );
    }

    #[test]
    fn do_forever_runs_until_a_bare_leave() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"n = 0\ndo forever\nn = n + 1\nif n = 3 then leave\nend\nsay n"
            ),
            b"3\n".to_vec()
        );
    }

    #[test]
    fn do_count_repeats_exactly_the_evaluated_number_of_times() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(&mut interp, b"n = 0\ndo 3\nn = n + 1\nend\nsay n"),
            b"3\n".to_vec()
        );
        // The repeat count is an expression, evaluated once.
        let mut interp = Interp::new();
        assert_eq!(
            say_output(&mut interp, b"n = 0\ndo 1 + 2\nn = n + 1\nend\nsay n"),
            b"3\n".to_vec()
        );
        // Zero repetitions is legal and runs the body no times at all.
        let mut interp = Interp::new();
        assert_eq!(
            say_output(&mut interp, b"n = 0\ndo 0\nn = n + 1\nend\nsay n"),
            b"0\n".to_vec()
        );
    }

    /// F1, found by review: a bare count `DO n` traced no `>K>` line at
    /// all, where the oracle traces it tagged `FOR` -- the same tag an
    /// explicit `DO ... FOR n` gets, measured (this task's own report).
    /// The mutation this kills is exactly the gap: deleting the new
    /// `trace_keyword` call in `run_loop`'s `LoopKind::Count` arm makes
    /// `interp.trace` empty instead of carrying the `>K>` line, with
    /// every other assertion in this file untouched -- this is the one
    /// test that would have caught the omission, since the report's own
    /// verification claimed `>K>` was checked while never actually
    /// running a bare-count program through it.
    #[test]
    fn a_bare_repeat_count_traces_as_for_the_same_as_an_explicit_one() {
        let mut interp = Interp::new();
        say_output_traced(&mut interp, b"do 2\nnop\nend");
        // `>K>` fires exactly once, on the first pass, matching every
        // other single-evaluation control-setup keyword (`TO`/`BY`/
        // `OVER`) -- the third `do 2` re-echo is the exit-check pass
        // `run_repeating`'s own per-pass re-echo already covers, verified
        // by running this exact assertion once and reading the bytes
        // back rather than hand-composing them (`this file's own report
        // has the trap: a hand-guessed expectation here was short by
        // exactly this one pass on the first attempt).
        assert_eq!(
            interp.trace,
            b"     1 *-* do 2\n       >K>   \"FOR\" => \"2\"\n     2 *-*   nop\n     \
              3 *-* end\n     1 *-* do 2\n     2 *-*   nop\n     3 *-* end\n     1 *-* do 2\n"
                .to_vec()
        );
    }

    /// F4, found by review: a comma-list condition traced no `>>>` for its
    /// own elements at all, only the list's overall result, under
    /// `TRACE R` (not merely `TRACE I`) -- measured, the oracle shows
    /// *three* `>>>` lines for `if 1, 1 then`, one per element plus one
    /// for the list, and `eval_logical_list`'s own fix (`eval.rs`) is what
    /// this test defends. The mutation it kills: removing that function's
    /// `self.trace_result(indent, &text)` call drops the middle two lines,
    /// leaving only the third (`eval_condition`'s own, unaffected) --
    /// caught by this test and by neither of the pre-existing comma-list
    /// tests (`if_condition_that_is_a_comma_list_raises_34_6_not_34_1`,
    /// the short-circuit test), since neither one traces anything.
    #[test]
    fn a_comma_list_conditions_own_elements_each_trace_their_result_under_trace_r() {
        let mut interp = Interp::new();
        say_output_traced(&mut interp, b"if 1, 1 then nop");
        assert_eq!(
            interp.trace,
            b"     1 *-* if 1, 1 \n       >>>   \"1\"\n       >>>   \"1\"\n       >>>   \"1\"\n     \
              1 *-*   then\n     1 *-*     nop\n"
                .to_vec()
        );
    }

    /// Found while re-verifying F4 rather than assumed clean: fixing the
    /// comma-list `>>>` gap exposed that `DO UNTIL`'s own re-echo was
    /// wired to the wrong site. The mutation this kills is two-sided --
    /// removing `is_until_loop`'s own gate on the top-of-loop echo makes
    /// a multi-pass `UNTIL` loop echo its clause **twice** per pass (once
    /// there, once at `UNTIL`'s own site); removing `UNTIL`'s own
    /// unconditional echo instead makes it echo **zero** times on the
    /// first pass (the top-of-loop site is `first_pass`-gated and never
    /// fires before the loop has gone around once). Only the fix in
    /// between gets exactly one echo per completed pass, matching the
    /// oracle (`t13_until_multi.rex`, this task's report).
    #[test]
    fn do_until_re_echoes_its_clause_exactly_once_per_pass_not_twice_or_zero() {
        let mut interp = Interp::new();
        say_output_traced(&mut interp, b"n = 0\ndo until n = 2\nn = n + 1\nend\nsay n");
        assert_eq!(
            interp.trace,
            b"     1 *-* n = 0\n       >>>   \"0\"\n     2 *-* do until n = 2\n     \
              3 *-*   n = n + 1\n       >>>     \"1\"\n     4 *-* end\n     2 *-* do until n = 2\n       \
              >K>     \"UNTIL\" => \"0\"\n     3 *-*   n = n + 1\n       >>>     \"2\"\n     4 *-* end\n     \
              2 *-* do until n = 2\n       >K>     \"UNTIL\" => \"1\"\n     5 *-* say n\n       >>>   \"2\"\n"
                .to_vec()
        );
    }

    #[test]
    fn do_with_takes_the_loud_path() {
        let mut interp = Interp::new();
        let failure = run_source(&mut interp, b"do with index i over 'x'\nsay i\nend").unwrap_err();
        let Failure::Loud(loud) = failure else {
            panic!("expected Loud, got {failure:?}");
        };
        // Review finding I1: `instruction.kind` here is `InstructionKind::Do`,
        // 4a's own -- `lib.rs`'s `owned_message` must not attribute this to a
        // phase (there is none to blame; `DO WITH` is Phase 5's *reason*, but
        // the message names the construct, not the reason). Mutation-kill for
        // deleting the `"4a"`/`None` carve-out in `owned_message`: this
        // assertion is what turns that deletion into a failure here, since no
        // corpus program and no other test inspected the message text before.
        assert_eq!(
            loud.message, "DO is not implemented",
            "a construct 4a does implement must not be attributed to a phase; \
             see lib.rs's owned_message"
        );
    }

    #[test]
    fn do_counter_takes_the_loud_path_regardless_of_which_other_kind_it_rides_on() {
        let mut interp = Interp::new();
        let failure = run_source(&mut interp, b"do counter c i = 1 to 3\nnop\nend").unwrap_err();
        let Failure::Loud(loud) = failure else {
            panic!("expected Loud, got {failure:?}");
        };
        // Same mutation-kill as `do_with_takes_the_loud_path`, above.
        assert_eq!(
            loud.message, "DO is not implemented",
            "a construct 4a does implement must not be attributed to a phase; \
             see lib.rs's owned_message"
        );
    }

    // ---- DO i = TO/BY/FOR (controlled), and DO OVER ----

    #[test]
    fn a_controlled_loop_runs_to_then_by_then_stops() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(&mut interp, b"do i = 1 to 3\nsay i\nend"),
            b"1\n2\n3\n".to_vec()
        );
        let mut interp = Interp::new();
        assert_eq!(
            say_output(&mut interp, b"do i = 10 to 1 by -4\nsay i\nend"),
            b"10\n6\n2\n".to_vec(),
            "measured against the oracle, do_loop_forms.rex's own transcript"
        );
    }

    /// A non-whole control value is legal (Step 2's own table): `do i = 1.5
    /// to 3` steps by fractional values, not an error.
    #[test]
    fn a_controlled_loops_own_values_need_only_be_numeric_not_whole() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(&mut interp, b"do i = 1.5 to 3\nsay i\nend"),
            b"1.5\n2.5\n".to_vec()
        );
    }

    /// `BY 0` loops forever -- behaviour to reproduce, not an error
    /// (Step 2's own table) -- bounded here by a `LEAVE` so the test itself
    /// terminates.
    #[test]
    fn a_controlled_loop_with_by_0_loops_forever() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"i = 0\ndo j = 1 by 0 to 3\ni = i + 1\nif i > 5 then leave\nend\nsay i"
            ),
            b"6\n".to_vec(),
            "measured against the oracle"
        );
    }

    #[test]
    fn a_controlled_loops_own_for_caps_the_iteration_count_independent_of_to() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"n = 0\ndo i = 1 to 100 for 3\nn = n + 1\nend\nsay n"
            ),
            b"3\n".to_vec()
        );
    }

    /// The control variable is bound to its own header value **before** the
    /// loop's own bound test, even for a loop that ends up running zero
    /// iterations -- measured against the oracle.
    #[test]
    fn the_control_variable_is_bound_even_when_the_loop_never_runs_its_body() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(&mut interp, b"do i = 5 to 3\nsay 'never'\nend\nsay i"),
            b"5\n".to_vec()
        );
    }

    /// `DO name OVER expr` on a **non-stem** target: a string and a number
    /// each iterate exactly once, yielding themselves -- Deviation 1 keeps
    /// a stem target out of scope, and this test uses none.
    #[test]
    fn do_over_a_non_stem_target_iterates_once_yielding_itself() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(&mut interp, b"do x over 'hello'\nsay x\nend"),
            b"hello\n".to_vec()
        );
        let mut interp = Interp::new();
        assert_eq!(
            say_output(&mut interp, b"do x over 42\nsay x\nend"),
            b"42\n".to_vec()
        );
    }

    /// `DO OVER ... FOR` on a non-stem target: `FOR 0` skips the one
    /// iteration entirely; any `FOR` at least `1` still runs it exactly
    /// once (there is only ever one item). Not independently measured
    /// against the oracle (no oracle transcript pins `OVER ... FOR` on a
    /// non-stem target specifically); implemented as the direct, minimal
    /// extension of `FOR`'s own general "caps the iteration count" rule,
    /// and named as a judgement call in the report.
    #[test]
    fn do_over_for_0_skips_the_single_non_stem_iteration() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"do x over 'hello' for 0\nsay x\nend\nsay 'after'"
            ),
            b"after\n".to_vec()
        );
        let mut interp = Interp::new();
        assert_eq!(
            say_output(&mut interp, b"do x over 'hello' for 5\nsay x\nend"),
            b"hello\n".to_vec()
        );
    }

    /// Deviation 1 (`phase-4-exclusions.txt`): `DO OVER` on a stem does not
    /// reproduce the oracle's traversal order, and takes the loud path
    /// instead -- detected from `target`'s own syntax (a bare `NAME.`
    /// parses as `ExprKind::Stem`), never by evaluating it.
    #[test]
    fn do_over_a_stem_target_takes_the_loud_path() {
        let mut interp = Interp::new();
        let failure = run_source(&mut interp, b"a.1 = 'x'\ndo v over a.\nsay v\nend").unwrap_err();
        let Failure::Loud(loud) = failure else {
            panic!("expected Loud, got {failure:?}");
        };
        // Same mutation-kill as `do_with_takes_the_loud_path` (above, in this
        // module): `DO`/`LOOP` is 4a's own regardless of which deviation
        // routed this particular clause to the loud path.
        assert_eq!(
            loud.message, "DO is not implemented",
            "a construct 4a does implement must not be attributed to a phase; \
             see lib.rs's owned_message"
        );
    }

    /// A stem target wrapped in parens is **also** caught -- corrected after
    /// review, which found this task's own comment on the `Over` arm
    /// claimed the opposite (`over (a.)` "is not detected"). It is detected:
    /// a single parenthesised sub-expression collapses to that
    /// sub-expression's own `ExprKind` rather than wrapping it in
    /// `ExprKind::List`, so `(a.)` is already `ExprKind::Stem` by the time
    /// `run_loop`'s own `matches!` check sees it, with nothing extra
    /// needed. The safe direction either way (loud, never a silent
    /// divergence), but the comment was wrong about which one it is.
    #[test]
    fn do_over_a_parenthesised_stem_target_is_also_caught() {
        let mut interp = Interp::new();
        let failure =
            run_source(&mut interp, b"a.1 = 'x'\ndo v over (a.)\nsay v\nend").unwrap_err();
        let Failure::Loud(loud) = failure else {
            panic!("expected Loud, got {failure:?}");
        };
        // Same mutation-kill as `do_with_takes_the_loud_path` (above, in this
        // module).
        assert_eq!(
            loud.message, "DO is not implemented",
            "a construct 4a does implement must not be attributed to a phase; \
             see lib.rs's owned_message"
        );
    }

    /// F1 (branch review, Important): `initial`/`to`/`by` are rounded under
    /// the *entry* `NUMERIC DIGITS`, once, not stored as their exact parse
    /// -- `round_via_unary_plus`'s own doc comment has the oracle citation
    /// and both transcripts this mirrors exactly. Masked while `DIGITS`
    /// stays constant (every later use re-rounds to the same width, so the
    /// exact and the rounded value render identically); this widens
    /// `DIGITS` *inside* the loop body, after entry, so only a value
    /// rounded at entry -- not the exact parse -- can still show `1.23` on
    /// every pass. Mutation killed: removing either `round_via_unary_plus`
    /// call in `setup_controlled` (the one on `current`, or the one in
    /// `ControlExpr::By`'s own arm) makes the affected probe's second and
    /// third lines read the exact, unrounded value instead (`2.23456`/
    /// `3.23456` for `current`, or accumulating by the exact `1.2345`
    /// instead of the rounded `1.23` for `by`) -- verified by reverting
    /// each in turn.
    #[test]
    fn a_controlled_loops_header_values_round_at_entry_not_the_exact_parse() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"numeric digits 3\ndo i = 1.23456 to 3\nsay i\nnumeric digits 9\nend"
            ),
            b"1.23\n2.23\n".to_vec(),
            "measured against the oracle: current is rounded once at entry \
             (1.23456 -> 1.23), and widening digits inside the loop must \
             not un-round it -- the exact parse would give 2.23456"
        );

        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"numeric digits 3\ndo i = 1 to 9 by 1.2345\nsay i\nnumeric digits 9\nend"
            ),
            b"1\n2.23\n3.46\n4.69\n5.92\n7.15\n8.38\n".to_vec(),
            "measured against the oracle: by is rounded to 1.23 once at \
             entry and accumulated at that width, not the exact 1.2345 -- \
             the loop stops after 8.38 because the next value, 9.61, is \
             past the bound"
        );
    }

    // ---- DO/LOOP header errors (Step 2's own table, re-measured) ----

    #[test]
    fn a_non_numeric_control_value_raises_41_1() {
        for (source, found) in [
            (&b"do i = 'a' to 3\nnop\nend"[..], "a"),
            (&b"do i = 1 to 'x'\nnop\nend"[..], "x"),
            (&b"do i = 1 by 'x'\nnop\nend"[..], "x"),
        ] {
            let mut interp = Interp::new();
            let failure = run_source(&mut interp, source).unwrap_err();
            let Failure::Raised(raised) = failure else {
                panic!("expected Raised, got {failure:?}");
            };
            assert_eq!((raised.number, raised.sub), (41, 1), "{source:?}");
            assert_eq!(raised.additional, vec![found.to_string()], "{source:?}");
        }
    }

    #[test]
    fn a_bad_for_expression_raises_26_3() {
        for (source, found) in [
            (&b"do i = 1 to 3 for 'x'\nnop\nend"[..], "x"),
            (&b"do i = 1 to 3 for -1\nnop\nend"[..], "-1"),
            (&b"do i = 1 to 3 for 1.5\nnop\nend"[..], "1.5"),
        ] {
            let mut interp = Interp::new();
            let failure = run_source(&mut interp, source).unwrap_err();
            let Failure::Raised(raised) = failure else {
                panic!("expected Raised, got {failure:?}");
            };
            assert_eq!((raised.number, raised.sub), (26, 3), "{source:?}");
            assert_eq!(raised.additional, vec![found.to_string()], "{source:?}");
        }
    }

    #[test]
    fn a_bad_repetition_count_raises_26_2() {
        for (source, found) in [
            (&b"do 'a'\nnop\nend"[..], "a"),
            (&b"do -1\nnop\nend"[..], "-1"),
            (&b"do 2.5\nnop\nend"[..], "2.5"),
        ] {
            let mut interp = Interp::new();
            let failure = run_source(&mut interp, source).unwrap_err();
            let Failure::Raised(raised) = failure else {
                panic!("expected Raised, got {failure:?}");
            };
            assert_eq!((raised.number, raised.sub), (26, 2), "{source:?}");
            assert_eq!(raised.additional, vec![found.to_string()], "{source:?}");
        }
    }

    /// F3 (branch review, Important): a bare `DO` count and a `FOR` count
    /// are validated under the *current* `NUMERIC DIGITS`, not a fixed
    /// width -- `whole_nonneg`'s own doc comment has the oracle citation
    /// and the two oracle-measured transcripts this mirrors
    /// (`numeric digits 3; do 12345; end` is 26.2 rc 230; the `FOR` shape
    /// is 26.3). Mutation killed: reverting `whole_nonneg` to convert
    /// under `rexx_num::ARGUMENT_DIGITS` (18) instead of the activation's
    /// own `settings.digits()` makes both of these run clean (`after`
    /// prints, no error) instead of raising, since `12345`/`54321` both
    /// fit comfortably under 18 digits and only fail to fit under 3.
    #[test]
    fn a_repetition_or_for_count_is_validated_under_the_current_digits_not_a_fixed_width() {
        let mut interp = Interp::new();
        let failure =
            run_source(&mut interp, b"numeric digits 3\ndo 12345\nend\nsay 'after'").unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (26, 2));

        let mut interp = Interp::new();
        let failure = run_source(
            &mut interp,
            b"numeric digits 3\ndo i = 1 to 99999 for 12345\nend\nsay 'after'",
        )
        .unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (26, 3));
    }

    // ---- WHILE/UNTIL ----

    #[test]
    fn do_while_tests_the_condition_before_the_body_and_do_until_after() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(&mut interp, b"i = 0\ndo while i < 2\ni = i + 1\nsay i\nend"),
            b"1\n2\n".to_vec()
        );
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"i = 5\ndo until i >= 7\ni = i + 1\nsay i\nend"
            ),
            b"6\n7\n".to_vec(),
            "measured against the oracle, do_loop_forms.rex's own transcript"
        );
    }

    #[test]
    fn a_while_condition_that_is_not_0_or_1_raises_34_3_not_34_1() {
        let mut interp = Interp::new();
        let failure = run_source(&mut interp, b"do while 'x'\nnop\nend").unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (34, 3));
        assert_eq!(raised.additional, vec!["x".to_string()]);
    }

    #[test]
    fn an_until_condition_that_is_not_0_or_1_raises_34_4() {
        let mut interp = Interp::new();
        let failure = run_source(&mut interp, b"do until 'x'\nnop\nend").unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (34, 4));
        assert_eq!(raised.additional, vec!["x".to_string()]);
    }

    /// A comma-list `WHILE`/`UNTIL` condition is 34.6, never 34.3/34.4 --
    /// `eval_condition`'s own rule, reused unchanged from `IF`/`WHEN`.
    #[test]
    fn a_comma_list_while_condition_raises_34_6_not_34_3() {
        let mut interp = Interp::new();
        let failure = run_source(&mut interp, b"do while 'x', 1\nnop\nend").unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (34, 6));
    }

    /// **The discriminating measurement for this whole section**: `UNTIL`
    /// is tested at the *bottom* of the loop, so its own failure is
    /// attributed to the `END`'s own clause, not the `DO`'s -- a loop that
    /// evaluated `UNTIL` eagerly, at the top like `WHILE`, would still
    /// produce the same raised number and the same exit code, and only the
    /// echoed clause reveals the difference. Measured against the oracle:
    /// `do until 'x' / end` echoes `end`, `do while 'x' / end` echoes the
    /// `do` line.
    #[test]
    fn until_is_attributed_to_the_end_clause_while_while_is_attributed_to_the_do_clause() {
        let mut interp = Interp::new();
        run_source(&mut interp, b"do until 'x'\nnop\nend").unwrap_err();
        let FailureSite { line, text, .. } = interp.failure_site.expect("a site was resolved");
        assert_eq!(line, 3, "the END's own line");
        assert_eq!(text, b"end".to_vec(), "the END's own clause, not the DO's");

        let mut interp = Interp::new();
        run_source(&mut interp, b"do while 'x'\nnop\nend").unwrap_err();
        let FailureSite { line, text, .. } = interp.failure_site.expect("a site was resolved");
        assert_eq!(line, 1, "the DO's own line");
        assert_eq!(text, b"do while 'x'".to_vec(), "the DO's own clause");
    }

    /// **`ITERATE` jumps to the loop's own bottom-of-iteration bookkeeping,
    /// not to the top of the next pass** -- measured against the oracle,
    /// this exact program terminates immediately with `n` still `1`, rather
    /// than looping forever. A design that instead skipped `UNTIL` for the
    /// interrupted iteration would hang on this program (`n` would keep
    /// advancing past `1` and `UNTIL n = 1` would never hold again), which
    /// is the mutation this test is built to catch -- not run, for the
    /// obvious reason a hang cannot be asserted against, but predicted and
    /// confirmed absent by construction: `do_body_outcome`'s own `Iterate`
    /// arm answers `Ok(None)`, the *same* value `Flow::Next` answers, so
    /// `run_repeating`'s own `UNTIL` check below it runs on both paths
    /// identically.
    #[test]
    fn iterate_reaches_untils_own_bottom_of_iteration_test_rather_than_skipping_it() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"n = 0\ndo until n = 1\nn = n + 1\nif n = 1 then iterate\nsay 'unreached'\nend\nsay 'done' n"
            ),
            b"done 1\n".to_vec()
        );
    }

    // ---- LEAVE/ITERATE: the block-stack rules ----

    #[test]
    fn bare_leave_stops_the_innermost_loop() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"do i = 1 to 5\nif i = 3 then leave\nsay i\nend"
            ),
            b"1\n2\n".to_vec()
        );
    }

    #[test]
    fn bare_iterate_skips_the_rest_of_the_current_pass() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"do i = 1 to 4\nif i = 2 then iterate\nsay i\nend"
            ),
            b"1\n3\n4\n".to_vec()
        );
    }

    /// A bare `LEAVE`/`ITERATE` skips **transparently past** an unlabelled
    /// `DO` block on its way to the nearest enclosing loop -- measured
    /// against the oracle: this program prints `1` then `after`, not
    /// `unreached`, because the innermost `DO` (a block, not a loop) never
    /// intercepts the bare `LEAVE` at all.
    #[test]
    fn bare_leave_passes_transparently_through_an_unlabelled_simple_block() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"do i = 1 to 3\ndo\nsay i\nleave\nsay 'unreached'\nend\nend\nsay 'after'"
            ),
            b"1\nafter\n".to_vec()
        );
    }

    /// **Bare `LEAVE` in a simple `DO` block is 28.1** (not a loop, and
    /// unlabelled, so nothing on the enclosing chain ever matches) -- but a
    /// **labelled** simple block is leavable by its own explicit name,
    /// which the next test pins.
    #[test]
    fn bare_leave_in_an_unlabelled_simple_block_with_nothing_enclosing_it_raises_28_1() {
        let mut interp = Interp::new();
        let failure = run_source(&mut interp, b"do\nleave\nend").unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (28, 1));
    }

    #[test]
    fn a_labelled_simple_block_is_leavable_by_its_own_name() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"do label blk\nsay 'a'\nleave blk\nsay 'b'\nend\nsay 'after'"
            ),
            b"a\nafter\n".to_vec()
        );
    }

    /// **An ordinary clause label does not name a loop or a block.**
    /// `outer:` here is a plain `Label` instruction, entirely separate from
    /// the `DO`'s own `label` field (which is `i`, the control variable,
    /// since no `LABEL` keyword was written) -- measured, `leave outer` is
    /// 28.3, not a hit, exactly as `leave_nested_outer.rex`'s own comment
    /// states.
    #[test]
    fn an_ordinary_clause_label_does_not_name_the_loop_it_precedes() {
        let mut interp = Interp::new();
        let failure =
            run_source(&mut interp, b"outer: do i = 1 to 3\nleave outer\nend").unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (28, 3));
        assert_eq!(raised.additional, vec!["OUTER".to_string()]);

        let mut interp = Interp::new();
        let failure =
            run_source(&mut interp, b"outer: do i = 1 to 3\niterate outer\nend").unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (28, 4));
        assert_eq!(raised.additional, vec!["OUTER".to_string()]);
    }

    /// **What does name a loop for `LEAVE`/`ITERATE`**: a controlled loop's
    /// own control variable, as an automatic label, with no `LABEL` keyword
    /// needed at all. `leave i`/`iterate i` from inside a *nested* loop
    /// reaches the outer one and unwinds the inner one on the way --
    /// `leave_nested_outer.rex`'s own transcript, reproduced here.
    #[test]
    fn a_controlled_loops_own_control_variable_is_an_automatic_label() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"do outer = 1 to 3\ndo inner = 1 to 3\nif inner = 2 then leave outer\nsay 'inner' outer inner\nend\nsay 'after inner' outer\nend\nsay 'after outer'"
            ),
            b"inner 1 1\nafter outer\n".to_vec()
        );
    }

    #[test]
    fn iterate_naming_the_outer_loops_control_variable_cuts_every_outer_pass_short() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"do outer = 1 to 3\ndo inner = 1 to 3\nif inner = 2 then iterate outer\nsay 'l' outer inner\nend\nsay 'outer-after' outer\nend"
            ),
            b"l 1 1\nl 2 1\nl 3 1\n".to_vec(),
            "outer-after never prints for any pass, since every one is cut short at inner = 2"
        );
    }

    /// **`leave sel` exits a `SELECT` only when it was written
    /// `SELECT LABEL sel`.** An ordinary clause label in front of a
    /// `SELECT` does not make it leavable (28.3, exactly like a loop).
    #[test]
    fn leave_by_name_exits_a_select_only_when_it_was_given_that_label() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"s: select label s\nwhen 1 = 1 then\ndo\nleave s\nend\notherwise\nnop\nend\nsay 'after'"
            ),
            b"after\n".to_vec()
        );

        let mut interp = Interp::new();
        let failure = run_source(
            &mut interp,
            b"select\nwhen 1 = 1 then\ndo\nleave sel\nend\notherwise\nnop\nend",
        )
        .unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (28, 3));
        assert_eq!(raised.additional, vec!["SEL".to_string()]);
    }

    /// A named `LEAVE` reaching a `SELECT LABEL` from inside its
    /// `OTHERWISE` branch -- the fix that routes `OTHERWISE`'s own body
    /// through `Select`'s own `run_bounded`/`leave_select`, not the plain
    /// `Goto` it used before Task 11 (`Select`'s own arm has the full
    /// argument). Kills a version that still lets `OTHERWISE` fall through
    /// to the outer loop directly: that version would propagate this
    /// `LEAVE` all the way to `run_activation`'s own top level and raise
    /// 28.3 instead of the clean exit this test asserts.
    #[test]
    fn leave_by_name_reaches_a_select_label_from_inside_its_otherwise_branch() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"select label s\nwhen 1 = 0 then nop\notherwise\nsay 'o'\nleave s\nsay 'unreached'\nend\nsay 'after'"
            ),
            b"o\nafter\n".to_vec()
        );
    }

    /// **`ITERATE` never accepts a labelled block or a labelled `SELECT`,
    /// only a loop -- 28.5, not silently skipped, when the name matches but
    /// the kind is wrong.**
    #[test]
    fn a_named_iterate_matching_a_labelled_simple_block_raises_28_5_not_28_4() {
        let mut interp = Interp::new();
        let failure = run_source(&mut interp, b"do label x\nsay 1\niterate x\nend").unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (28, 5));
        assert_eq!(raised.additional, vec!["X".to_string()]);
    }

    #[test]
    fn a_named_iterate_matching_a_select_label_raises_28_5() {
        let mut interp = Interp::new();
        let failure = run_source(
            &mut interp,
            b"select label s\nwhen 1 = 1 then\ndo\niterate s\nend\notherwise\nnop\nend",
        )
        .unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (28, 5));
        assert_eq!(raised.additional, vec!["S".to_string()]);
    }

    #[test]
    fn iterate_from_inside_a_select_nested_in_a_loop_skips_only_the_current_pass() {
        // iterate_from_select.rex's own transcript: an ITERATE inside a
        // SELECT's WHEN body must skip straight to the loop's own next
        // pass, past both the SELECT's own resume point and back up to the
        // enclosing DO, printing exactly one "skip"/"keep" pair per
        // iteration and never both for the same i.
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"do i = 1 to 4\nselect\nwhen i = 2 then do\nsay 'skip' i\niterate\nend\notherwise nop\nend\nsay 'keep' i\nend"
            ),
            b"keep 1\nskip 2\nkeep 3\nkeep 4\n".to_vec()
        );
    }

    /// Two real, unnamed loops nested two deep, neither matching `zz`: both
    /// own a search frame (`is_loop` true for a `Controlled` loop), so both
    /// get popped and both reset the residual to their own `static_indent`
    /// -- the outer one last, to `0` (top level), which is what survives to
    /// the exhausted-search report. This is **not** "28.1-28.4 always
    /// report zero" (that rule was wrong -- see `n1`/`n2`/`n3` below, and
    /// `LeaveOrigin`'s own doc comment for the corrected one): it is zero
    /// here specifically because the outermost popped frame happens to sit
    /// at top level, the same way it did in every one of this task's own
    /// original probes, which is exactly how the wrong rule looked right
    /// for as long as it did.
    #[test]
    fn leave_no_match_through_two_real_loops_resets_to_the_outer_ones_own_indent() {
        let mut interp = Interp::new();
        run_source(
            &mut interp,
            b"do i = 1 to 3\ndo j = 1 to 3\nleave zz\nend\nend",
        )
        .unwrap_err();
        let FailureSite { indent, .. } = interp.failure_site.expect("a site was resolved");
        assert_eq!(indent, 0);
    }

    /// The one intervening construct is an *unlabelled* `Simple` block,
    /// which owns no search frame and is fully transparent -- so nothing
    /// ever resets the residual, and it stays at the `ITERATE`'s own full
    /// lexical depth all the way to the match (the outer labelled block,
    /// which does not reset on a match either). Two real loops in
    /// `leave_no_match_through_two_real_loops_resets_to_the_outer_ones_own_indent`
    /// above land on the *same* final answer as this test for an entirely
    /// different reason -- neither test tells the two rules apart, which is
    /// this task's own original mistake and why the corrected rule below has
    /// its own dedicated tests.
    #[test]
    fn iterate_wrong_kind_through_a_transparent_unlabelled_block_reports_full_lexical_depth() {
        let mut interp = Interp::new();
        run_source(&mut interp, b"do label x\ndo\niterate x\nend\nend").unwrap_err();
        let FailureSite { indent, .. } = interp.failure_site.expect("a site was resolved");
        assert_eq!(indent, 4, "two DO frames deep, matching the oracle");
    }

    /// **The corrected 28.x indent rule, all fourteen points**: nine from
    /// the reviewer's own probe (`p1`/`p2`/`p5`/`p8`/`p9`/`p10`/`p11`/`p12`/
    /// `p13`, their own naming, kept so the report's cross-reference to
    /// this table still resolves) plus five re-measured independently by
    /// this task against the oracle before touching any code (`n1`-`n3`
    /// entirely new shapes, `p1`/`p11` re-run to confirm the two that
    /// already matched still do). Every row was captured with `cat -A`
    /// against `build/bin/rexx`, byte for byte, not inferred.
    ///
    /// **The rule the whole table obeys**: start at the `LEAVE`/`ITERATE`'s
    /// own full lexical depth; walk outward; every `SELECT` (always) or
    /// `DO`/`LOOP` (unless an unlabelled `Simple` block) that is examined
    /// and does *not* match resets the residual to *its own*
    /// `static_indent`; a match stops the walk without resetting anything
    /// itself; report whatever the residual is at that point. `p11`/`p1`
    /// are the two rows where the very first frame examined is the match,
    /// so nothing ever resets and the reported value is the origin's own
    /// unmodified full depth -- the case the original, wrong rule
    /// generalised from.
    #[test]
    fn the_corrected_28x_indent_rule_matches_all_fourteen_probed_shapes() {
        for (name, source, expect) in [
            ("p5", &b"if 1=1 then leave"[..], 4),
            ("p9", &b"if 1=1 then do\nleave\nend"[..], 6),
            ("p12", &b"do\nleave\nend"[..], 2),
            ("p8", &b"if 1=1 then do i=1 to 3\nleave zz\nend"[..], 4),
            (
                "p2",
                &b"do label x\nselect\nwhen 1=1 then iterate x\notherwise nop\nend\nend"[..],
                2,
            ),
            (
                "p10",
                &b"do label x\nselect\nwhen 1=1 then do\niterate x\nend\notherwise nop\nend\nend"[..],
                2,
            ),
            (
                "p13",
                &b"do label x\ndo label y\niterate x\nend\nend"[..],
                2,
            ),
            (
                "p1",
                &b"do label x\nif 1=1 then do\niterate x\nend\nend"[..],
                8,
            ),
            (
                "p11",
                &b"select label s\nwhen 1=1 then iterate s\notherwise nop\nend"[..],
                6,
            ),
            // Independently added by this task, not in the reviewer's own
            // table: a bare LEAVE reaching only an unlabelled SELECT
            // (SELECT owns a frame unconditionally, even unlabelled, so
            // the pop still happens and still resets to its own indent).
            (
                "n1",
                &b"select\nwhen 1=1 then leave\notherwise nop\nend"[..],
                0,
            ),
            // Three real loops nested three deep, none matching: each pop
            // resets in turn, the outermost (top level) wins.
            (
                "n2",
                &b"do i=1 to 3\ndo j=1 to 3\ndo k=1 to 3\nleave zz\nend\nend\nend"[..],
                0,
            ),
            // A named ITERATE crossing one unlabelled real loop and one
            // unlabelled SELECT, matching neither: both pop, the outer
            // loop's own indent (0, top level) is what survives.
            (
                "n3",
                &b"do i=1 to 3\nselect\nwhen 1=1 then iterate zz\notherwise nop\nend\nend"[..],
                0,
            ),
        ] {
            let mut interp = Interp::new();
            run_source(&mut interp, source).unwrap_err();
            let FailureSite { indent, .. } = interp.failure_site.expect("a site was resolved");
            assert_eq!(indent, expect, "{name}: {source:?}");
        }
    }

    /// **The `run_bounded` `Goto`-absorption trap, named in `Flow::Leave`'s
    /// own doc comment.** A `DO` with an `ITERATE` in its body, nested
    /// inside an `IF`'s `THEN`, run enough times that a version which
    /// implemented repetition as a `Goto` back to the loop's own top
    /// (rather than an internal `run_repeating` loop that never returns
    /// mid-construct) would have that `Goto`'s target absorbed by the
    /// enclosing `IF`'s own `run_bounded` -- which owns the whole `THEN`
    /// range the `DO` sits inside -- and re-enter the `DO`'s own arm as a
    /// fresh first pass, with its running total reset to `Simple`/`Count`'s
    /// own initial state every time. That bug's own observable signature
    /// is `n` staying stuck at whatever the first `ITERATE`d pass produced,
    /// forever, or (depending on exactly how the re-entry is shaped)
    /// hanging outright, rather than the small, exact total this test
    /// asserts.
    #[test]
    fn leave_and_iterate_survive_a_do_nested_in_an_ifs_then_iterating_repeatedly() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"n = 0\nif 1 = 1 then do i = 1 to 5\nif i // 2 = 0 then iterate\nn = n + i\nend\nsay n"
            ),
            b"9\n".to_vec(),
            "1 + 3 + 5, the odd i's only -- a Goto-based re-entry would not \
             accumulate this total across the DO's own five iterations"
        );
    }

    // ---- the LEAVE/ITERATE search stops at the run_fragment boundary ----

    /// The measured rule, all four families at once: a `LEAVE`/`ITERATE`
    /// inside `INTERPRET` text never sees the enclosing loop, so an
    /// enclosing `DO` that would have consumed it does not, and it is the
    /// exhausted search instead. `run_fragment`'s own doc comment has the
    /// oracle transcripts these numbers come from.
    ///
    /// **The bare rows are the ones that decide the design**, and until
    /// this was measured the code did the opposite: a bare `Flow::Leave`
    /// forwarded out of the fragment and the enclosing `DO` swallowed it,
    /// which is what "the fragment runs inside the enclosing activation"
    /// predicts and what the oracle does not do. Mutation-kill: restore
    /// `Ok(flow)` for the `None` arms in `run_fragment` and the two bare
    /// rows here run to completion with no error at all.
    #[test]
    fn a_fragments_leave_or_iterate_never_reaches_the_enclosing_loop() {
        for (source, number, sub, additional) in [
            (
                &b"do kk = 1 to 3\ninterpret \"leave\"\nend\n"[..],
                28u16,
                1u16,
                Vec::new(),
            ),
            (
                &b"do kk = 1 to 3\ninterpret \"iterate\"\nend\n"[..],
                28,
                2,
                Vec::new(),
            ),
            (
                &b"do label outer while 1\ninterpret \"leave outer\"\nend\n"[..],
                28,
                3,
                vec!["OUTER".to_string()],
            ),
            (
                &b"do label outer kk = 1 to 3\ninterpret \"iterate outer\"\nend\n"[..],
                28,
                4,
                vec!["OUTER".to_string()],
            ),
        ] {
            let mut interp = Interp::new();
            let failure = run_source(&mut interp, source).unwrap_err();
            let Failure::Raised(raised) = failure else {
                panic!("expected Raised for {source:?}, got {failure:?}");
            };
            assert_eq!((raised.number, raised.sub), (number, sub), "{source:?}");
            assert_eq!(raised.additional, additional, "{source:?}");
        }
    }

    /// The name in 28.3/28.4 is resolved against the **fragment's** symbol
    /// table, which is the half of F-EX2 that survives the rule above.
    ///
    /// `run_fragment` gives `"leave foo"` its own fresh `SymbolTable`, so
    /// `foo` interns at id 0 there regardless of what the enclosing
    /// program's own table looks like. This program's own table also has
    /// exactly one symbol -- `BAR`, also id 0, from the assignment on the
    /// first line -- chosen deliberately so the two tables collide on the
    /// same id with *different* names, which is what makes resolving
    /// against the wrong table give a wrong answer rather than a panic.
    /// Mutation-kill: resolve through the enclosing `code.symbols` instead
    /// and 28.3 names `"BAR"`, the enclosing program's own symbol 0, not the
    /// one `leave` actually named.
    #[test]
    fn a_fragments_named_leave_is_resolved_against_the_fragments_own_table() {
        let mut interp = Interp::new();
        let failure = run_source(&mut interp, b"bar = 1\ninterpret \"leave foo\"\n").unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (28, 3));
        assert_eq!(raised.additional, vec!["FOO".to_string()]);
    }

    /// Review finding I1(a): `INTERPRET` traces `>>>` on the text it is about
    /// to run, like every other value-producing arm, and it shipped without
    /// doing so.
    ///
    /// Oracle, verbatim, for the first program (rc 0, empty stdout):
    ///
    /// ```text
    ///      1 *-* trace r
    ///      2 *-* zz = 'nop'
    ///        >>>   "nop"
    ///      3 *-* interpret zz
    ///        >>>   "nop"
    ///      3 *-* nop
    /// ```
    ///
    /// **The last line is 4b Task 2's half, and it is asserted now.** Task 1
    /// landed the `>>>` and left this expectation one line short of the
    /// oracle on purpose, because `run_fragment` passed `source: None` and
    /// `step_in_temps_frame` had no clause site to echo. The whole transcript
    /// is compared byte for byte below, including that the fragment's clause
    /// echoes as line **3** -- the enclosing `INTERPRET`'s line, not the
    /// fragment's own line 1, which is what `Interp::clause_line_override`
    /// exists for and what a naive `Some(&fragment.source)` gets wrong.
    /// (`say_output` drives `trace_mode` directly instead of running a `trace
    /// r` clause, so the program below is the oracle's minus its first line
    /// and every line number is one lower.)
    ///
    /// The second program pins the **indent**, which is the part a wrong fix
    /// would still get wrong: one `DO` deeper, the oracle's `>>>` picks up
    /// that construct's own two spaces, and it does so because the arm reads
    /// `current_value_indent` rather than recomputing anything. Mutation
    /// killed both ways: dropping the `trace_result` call empties the `>>>`
    /// lines, and moving it after `run_fragment` reports the *fragment's* last
    /// indent instead of this clause's.
    ///
    /// It now pins the fragment echo's indent too, which is the **delta-0**
    /// measurement: the oracle prints `     3 *-*   nop` -- the enclosing
    /// clause's own two spaces and no more. An implementation that gave the
    /// fragment a level's worth of extra indent (the two spaces a *called
    /// routine* really does get, measured) prints four here and fails.
    #[test]
    fn interpret_traces_the_text_it_is_about_to_run() {
        let mut interp = Interp::new();
        say_output_traced(&mut interp, b"zz = 'nop'\ninterpret zz");
        assert_eq!(
            interp.trace,
            concat!(
                "     1 *-* zz = 'nop'\n",
                "       >>>   \"nop\"\n",
                "     2 *-* interpret zz\n",
                "       >>>   \"nop\"\n",
                "     2 *-* nop\n",
            )
            .as_bytes()
        );

        let mut interp = Interp::new();
        say_output_traced(&mut interp, b"do kk = 1 to 1\ninterpret \"nop\"\nend");
        for expected in [&b"       >>>     \"nop\"\n"[..], &b"     2 *-*   nop\n"[..]] {
            assert!(
                interp
                    .trace
                    .windows(expected.len())
                    .any(|window| window == expected),
                "expected a line carrying the enclosing DO's own two spaces \
                 ({:?}), got {:?}",
                String::from_utf8_lossy(expected),
                String::from_utf8_lossy(&interp.trace)
            );
        }
    }

    /// The other side of the boundary rule: a loop written *inside* the
    /// fragment consumes its own `LEAVE` normally, so the rule above is a
    /// statement about crossing the boundary and not a blanket refusal.
    /// Measured: this program prints two lines and exits 0.
    #[test]
    fn a_leave_inside_the_fragments_own_loop_is_consumed_there() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"interpret \"do jj = 1 to 5; say 'frag' jj; if jj = 2 then leave; end\"\nsay 'after'\n"
            ),
            b"frag 1\nfrag 2\nafter\n".to_vec()
        );
    }

    // ---- Task 11's own indentation quantity ----

    #[test]
    fn one_two_and_three_enclosing_dos_indent_by_two_four_and_six() {
        for (source, spaces) in [
            (&b"do i = 1 to 3\nsay 1/0\nend"[..], 2),
            (&b"do i = 1 to 3\ndo j = 1 to 3\nsay 1/0\nend\nend"[..], 4),
            (
                &b"do i = 1 to 3\ndo j = 1 to 3\ndo k = 1 to 3\nsay 1/0\nend\nend\nend"[..],
                6,
            ),
        ] {
            let mut interp = Interp::new();
            run_source(&mut interp, source).unwrap_err();
            let FailureSite { indent, .. } = interp.failure_site.expect("a site was resolved");
            assert_eq!(indent, spaces, "{source:?}");
        }
    }

    /// **The test the coordinator's own design review asked for by name**:
    /// a raise *after* a loop has already exited, at a shallower lexical
    /// depth than the loop's own body -- exactly the shape a live,
    /// imperfectly-unwound `Interp` counter would over-indent and a purely
    /// static function of `(instructions, index)` cannot, because there is
    /// no state left over from the loop's own three completed iterations
    /// for anything to have failed to unwind.
    #[test]
    fn the_indent_after_a_loop_has_already_exited_is_not_left_over_from_it() {
        let mut interp = Interp::new();
        run_source(&mut interp, b"do i = 1 to 3\nsay i\nend\nsay 1/0").unwrap_err();
        let FailureSite { indent, text, .. } = interp.failure_site.expect("a site was resolved");
        assert_eq!(
            indent, 0,
            "top level, after the loop, not the loop's own two"
        );
        assert_eq!(text, b"say 1/0".to_vec());
    }

    /// `DO`'s own control-setup expressions (`TO`/`BY`/`FOR`/the repeat
    /// count/`OVER`'s target) are evaluated **before** the loop's own body
    /// is entered, and are unindented even though the `DO` clause they
    /// belong to sits at the same lexical position the body's own two
    /// spaces would apply to -- the case the brief's own bullet 4 names
    /// (`do i = 1 to 'x'` gets none), re-measured across the sibling forms
    /// the brief did not enumerate.
    #[test]
    fn control_setup_expressions_are_unindented_unlike_the_loop_body_they_precede() {
        for source in [
            &b"do i = 1 to 'x'\nsay 1\nend"[..],
            &b"do 1/0\nsay 1\nend"[..],
            &b"do i = 1 to 3 for 1/0\nsay 1\nend"[..],
            &b"do i = 1 to 3 by 1/0\nsay 1\nend"[..],
        ] {
            let mut interp = Interp::new();
            run_source(&mut interp, source).unwrap_err();
            let FailureSite { indent, .. } = interp.failure_site.expect("a site was resolved");
            assert_eq!(indent, 0, "{source:?}");
        }
    }

    /// `WHILE`/`UNTIL` are tested **inside** the loop's own frame, unlike
    /// the header's control-setup expressions -- measured, `do while 1/0`
    /// is indented two at top level, not zero.
    #[test]
    fn while_and_until_are_indented_inside_the_loops_own_frame() {
        let mut interp = Interp::new();
        run_source(&mut interp, b"do while 1/0\nsay 1\nend").unwrap_err();
        let FailureSite { indent, .. } = interp.failure_site.expect("a site was resolved");
        assert_eq!(indent, 2);

        let mut interp = Interp::new();
        run_source(&mut interp, b"do until 1/0\nsay 1\nend").unwrap_err();
        let FailureSite { indent, .. } = interp.failure_site.expect("a site was resolved");
        assert_eq!(indent, 2);
    }

    /// A `SELECT`'s own scan (evaluating each `WHEN`'s condition) is
    /// indented two, present even before any `WHEN` matches -- the finding
    /// this task made that the brief did not state (measured: `select /
    /// when 1/0 then nop / end` indents the failing `WHEN`'s own condition
    /// by two, not zero).
    #[test]
    fn a_whens_own_condition_is_indented_at_the_selects_own_two_spaces() {
        let mut interp = Interp::new();
        run_source(&mut interp, b"select\nwhen 1/0 then nop\nend").unwrap_err();
        let FailureSite { indent, .. } = interp.failure_site.expect("a site was resolved");
        assert_eq!(indent, 2);
    }

    /// A matched `WHEN`'s own `THEN` body indents six (`SELECT`'s two, plus
    /// the `WHEN`-`THEN` shape's own four, the same as an `IF`'s matched
    /// branch); `OTHERWISE`'s own body indents only four, **not** six --
    /// the second finding this task made that the brief did not state,
    /// because `OTHERWISE` is not built from the same double-frame `IF`-
    /// shaped machinery a `WHEN`'s `THEN` is.
    #[test]
    fn a_matched_whens_then_body_indents_six_but_otherwises_body_indents_only_four() {
        let mut interp = Interp::new();
        run_source(&mut interp, b"select\nwhen 1 = 1 then say 1/0\nend").unwrap_err();
        let FailureSite { indent, .. } = interp.failure_site.expect("a site was resolved");
        assert_eq!(indent, 6);

        let mut interp = Interp::new();
        run_source(
            &mut interp,
            b"select\nwhen 1 = 0 then nop\notherwise\nsay 1/0\nend",
        )
        .unwrap_err();
        let FailureSite { indent, .. } = interp.failure_site.expect("a site was resolved");
        assert_eq!(
            indent, 4,
            "OTHERWISE's own body, one frame, not the WHEN-THEN shape's two"
        );
    }

    /// An `IF`'s matched branch is four spaces, for **both** `THEN` and a
    /// plain `ELSE` (not only the "else if" chain shape the brief's own
    /// example used) -- and an "else if" chain nests two such branches,
    /// giving eight.
    #[test]
    fn an_ifs_matched_then_or_else_branch_indents_four_and_an_else_if_chain_indents_eight() {
        let mut interp = Interp::new();
        run_source(&mut interp, b"if 1 = 1 then say 1/0").unwrap_err();
        let FailureSite { indent, .. } = interp.failure_site.expect("a site was resolved");
        assert_eq!(indent, 4, "THEN");

        let mut interp = Interp::new();
        run_source(&mut interp, b"if 1 = 0 then say 2\nelse say 1/0").unwrap_err();
        let FailureSite { indent, .. } = interp.failure_site.expect("a site was resolved");
        assert_eq!(indent, 4, "a plain ELSE, not part of an else-if chain");

        let mut interp = Interp::new();
        run_source(
            &mut interp,
            b"if 1 = 0 then say 2\nelse if 1 = 1 then say 1/0",
        )
        .unwrap_err();
        let FailureSite { indent, .. } = interp.failure_site.expect("a site was resolved");
        assert_eq!(
            indent, 8,
            "the outer ELSE's own four, plus the inner IF's THEN's own four"
        );
    }

    /// Task 13's own four `static_indent` fixes, found while building
    /// `TRACE`'s clause echo -- **not a second computation of the
    /// indentation quantity**, a bug in the existing one, for a case
    /// Task 10/11 structurally could not exercise: none of `THEN`/`ELSE`/
    /// `OTHERWISE`/a `WHEN`'s own `THEN` carries an expression, so none of
    /// them can ever raise a condition and become a `FailureSite` -- the
    /// only way anything before this task ever asked `static_indent` a
    /// question. `TRACE` echoes every stepped instruction, markers
    /// included, which is what finally asks.
    ///
    /// Each expected number is the oracle's, read with `cat -A` against
    /// `build/bin/rexx` under `trace r` (the report has the full
    /// transcripts): a marker clause sits at exactly half the indent its
    /// own body gets, all the way down through nesting -- confirmed by
    /// `ThenInstruction.cpp`/`ElseInstruction.cpp`'s own `execute`
    /// (`indent(); trace; indent();`, so the marker traces after the first
    /// bump and the body after the second) and by `OtherwiseInstruction.cpp`
    /// (`trace; indent();`, one bump, so `OTHERWISE`'s own clause sits at
    /// the `SELECT`'s scan level, the same as a `WHEN`'s own condition).
    ///
    /// This calls `static_indent` directly rather than through a raise,
    /// because a marker clause cannot raise -- there is no `FailureSite` to
    /// read one back from.
    #[test]
    fn a_then_else_when_then_or_otherwise_markers_own_clause_indents_half_its_bodys() {
        // `IF`'s own `THEN`: `then_start` used to fall into the body's `+4`
        // branch (the branch's own recursive call returns 0 for the very
        // first position of its range) and answer 4, not 2.
        let instructions = instructions_of(b"if 1 = 1 then say 'x'");
        let then_start = if_then_start(&instructions, 0);
        assert_eq!(static_indent(&instructions, then_start), 2, "IF's own THEN");

        // `IF`'s own `ELSE`: `target == false_target` used to fall through
        // this whole arm entirely (`pc = else_end; continue`, which passes
        // straight over the marker's own index in the enclosing walk) and
        // answer 0, the enclosing level, not 2.
        let instructions = instructions_of(b"if 1 = 0 then say 'x'\nelse say 'y'");
        let if_index = 0;
        let InstructionKind::If { false_target, .. } = &instructions[if_index].kind else {
            panic!("index 0 is the IF")
        };
        let else_index = false_target.expect("this IF has an ELSE");
        assert_eq!(static_indent(&instructions, else_index), 2, "IF's own ELSE");

        // A `WHEN`'s own `THEN`, sharing `InstructionKind::Then` with `IF`
        // (`instruction.rs`'s `if_instruction` builds both): `target ==
        // body_start` used to match the loop's own `>=` and answer 6, the
        // body's value, not 4.
        let instructions = instructions_of(b"select\nwhen 1 = 1 then say 'x'\nend");
        let when_index = 1;
        let then_start = if_then_start(&instructions, when_index);
        assert_eq!(
            static_indent(&instructions, then_start),
            4,
            "WHEN's own THEN"
        );

        // `OTHERWISE`'s own clause -- **this one used to abort the process**,
        // not merely answer a wrong number: `target == *otherwise_index`
        // matched neither the `whens` loop nor the body check below it, and
        // fell to `unreachable!("a resolved SELECT's own range holds only
        // its WHENs and OTHERWISE")`. Reproduced against the tree before
        // this fix (`cargo test` aborted this exact test with that message,
        // `run.rs`'s panic site named in the fix's own commit) rather than
        // inferred.
        let instructions = instructions_of(b"select\nwhen 1 = 0 then nop\notherwise\nsay 'y'\nend");
        let otherwise_index = instructions
            .iter()
            .find_map(|i| match &i.kind {
                InstructionKind::Select { otherwise, .. } => *otherwise,
                _ => None,
            })
            .expect("this SELECT has an OTHERWISE");
        assert_eq!(
            static_indent(&instructions, otherwise_index),
            2,
            "OTHERWISE's own marker clause"
        );
    }

    /// `if_instruction` (`rexx-parse`'s `instruction.rs`) gives both `IF`
    /// and a `WHEN`'s own `THEN` the identical shape: the `Then` marker
    /// sits immediately after the condition-bearing instruction at
    /// `condition_index`. A free function rather than inlined three times
    /// in the test above, since all three call sites need the exact same
    /// index arithmetic and nothing else about `Instruction` to find it.
    fn if_then_start(instructions: &[Instruction], condition_index: usize) -> usize {
        assert!(
            matches!(
                instructions[condition_index].kind,
                InstructionKind::If { .. }
                    | InstructionKind::When { .. }
                    | InstructionKind::WhenCase { .. }
            ),
            "condition_index must be an IF, a WHEN or a WHEN CASE"
        );
        condition_index + 1
    }

    /// Parses `source` and returns its top-level instruction list, owned --
    /// the marker-index tests above need to read `InstructionKind` fields
    /// directly (`false_target`, `otherwise`) rather than only run the
    /// program to a raise, since a marker clause cannot raise at all.
    fn instructions_of(source: &[u8]) -> Vec<Instruction> {
        parse_program(source.to_vec())
            .expect("test program parses")
            .main
            .instructions
    }

    /// Nesting a `SELECT` (with a matched `WHEN`) inside a `DO`: two plus
    /// two plus four -- confirming the additive model composes across
    /// different construct kinds, not only same-kind nesting.
    #[test]
    fn a_select_nested_inside_a_do_composes_the_two_constructs_own_contributions() {
        let mut interp = Interp::new();
        run_source(
            &mut interp,
            b"do i = 1 to 3\nselect\nwhen 1 = 1 then say 1/0\notherwise nop\nend\nend",
        )
        .unwrap_err();
        let FailureSite { indent, .. } = interp.failure_site.expect("a site was resolved");
        assert_eq!(indent, 8);
    }

    // ---- End arm ----

    #[test]
    fn a_do_loops_own_end_is_a_pure_marker_never_independently_dispatched() {
        // Reached at all only if something jumped straight onto it, which
        // nothing in this crate does -- covered indirectly by every DO/LOOP
        // test above completing without the loud failure this arm used to
        // give before Task 11. Direct coverage: a labelled LOOP closes
        // cleanly (EndStyle::LabeledDo is exercised on the same code path
        // EndStyle::Do/Loop are).
        let mut interp = Interp::new();
        assert_eq!(
            say_output(&mut interp, b"do label x\nsay 'ok'\nend"),
            b"ok\n".to_vec()
        );
    }

    // ---- CALL and RETURN (Task 3) ----

    /// D9r's default, and the one property a witness without variables in it
    /// cannot check: a callee with no `PROCEDURE` reads the caller's
    /// variables **and its writes survive the return**. An implementation
    /// that gave every callee a fresh pool passes `call sub` / `sub: say
    /// 'callee'` and fails this.
    #[test]
    fn a_routine_without_procedure_shares_the_callers_pool() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"v = 'caller-v'\ncall sub\nsay 'caller sees:' v w\nexit\n\
                  sub:\nsay 'callee sees v:' v\nw = 'callee-w'\nreturn\n",
            ),
            b"callee sees v: caller-v\ncaller sees: caller-v callee-w\n".to_vec()
        );
    }

    /// The body selector's own reason to exist, at the level `run_activation`
    /// hardcoded through 4a: the callee runs *its* clauses, from the label,
    /// and the caller resumes after the `CALL` rather than at the top.
    #[test]
    fn a_called_label_runs_its_own_clauses_not_the_main_body() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"call sub\nsay 'main'\nexit\nsub: say 'callee'\nreturn\n"
            ),
            b"callee\nmain\n".to_vec()
        );
    }

    /// A name bound at run time inside the callee is part of the same shared
    /// pool, which needs the *name* to cross the return as well as the slot
    /// -- `Activation::extra`, cloned in and moved back out. Measured on the
    /// oracle both ways round; this is the direction that fails if `extra`
    /// is left behind with the callee.
    #[test]
    fn a_name_bound_by_interpret_inside_a_callee_survives_the_return() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"call sub\ninterpret \"say zork\"\nexit\nsub:\ninterpret \"zork = 42\"\nreturn\n",
            ),
            b"42\n".to_vec()
        );

        // And inward, which is the half that already worked: a name the
        // caller bound at run time is readable in the callee.
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"interpret \"zork = 42\"\ncall sub\nexit\nsub:\ninterpret \"say zork\"\nreturn\n",
            ),
            b"42\n".to_vec()
        );
    }

    /// `RESULT` is settled on **return**, not at the call: the callee sees
    /// the caller's own pre-call value, and only the return overwrites it.
    /// A bare `return` drops it, so it reads back as its own derived name.
    #[test]
    fn result_is_settled_on_return_and_a_bare_return_drops_it() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"result = 'before'\ncall sub\nsay 'after result=' result\n\
                  call bare\nsay 'after bare result=' result\nexit\n\
                  sub:\nsay 'inside result=' result\nreturn 42\nbare:\nreturn\n",
            ),
            b"inside result= before\nafter result= 42\nafter bare result= RESULT\n".to_vec()
        );
    }

    /// The two ways out of a callee that are *not* a return, both measured:
    /// an explicit `EXIT`, and the body simply running out of instructions.
    /// Either ends the program, so the caller's next clause never runs --
    /// which is why `Ended` distinguishes them from `Returned` at all.
    #[test]
    fn exiting_or_falling_off_the_end_inside_a_callee_ends_the_program() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"call sub\nsay 'never'\nsub:\nsay 'in sub'\nexit\n"
            ),
            b"in sub\n".to_vec()
        );

        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"call sub\nsay 'never'\nexit\nsub:\nsay 'in sub'\n"
            ),
            b"in sub\n".to_vec()
        );
    }

    /// A `RETURN` in the main body, with no active call: it ends the program
    /// with its value, exactly like `EXIT`. Measured at rc 5.
    #[test]
    fn a_return_in_the_main_body_ends_the_program_with_its_value() {
        let mut interp = Interp::new();
        let value =
            run_source(&mut interp, b"say 'a'\nreturn 5\nsay 'b'\n").expect("the program runs");
        assert_eq!(interp.out, b"a\n".to_vec());
        assert_eq!(interp.exit_code_for(value), 5);

        let mut interp = Interp::new();
        let value =
            run_source(&mut interp, b"say 'a'\nreturn\nsay 'b'\n").expect("the program runs");
        assert_eq!(interp.out, b"a\n".to_vec());
        assert_eq!(interp.exit_code_for(value), 0);
    }

    /// `RETURN` unwinds to the **activation** boundary and past every block
    /// frame in between -- which is exactly what `LEAVE` does not do. Both
    /// enclosing constructs here would consume a `Flow::Leave`; neither may
    /// consume a `Flow::Return`.
    #[test]
    fn a_return_escapes_every_enclosing_block_in_the_callee() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"call sub\nsay 'back'\nexit\nsub:\ndo forever\nreturn\nend\n",
            ),
            b"back\n".to_vec()
        );

        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"call sub\nsay 'back'\nexit\nsub:\nselect\nwhen 1 = 1 then return\notherwise nop\nend\n",
            ),
            b"back\n".to_vec()
        );
    }

    /// `NUMERIC` is inherited at call time and never written back -- measured
    /// with `digits 7` outside and `digits 3` inside. The second `say` is the
    /// discriminating one: an implementation that shared one `Settings`
    /// would print `3` there.
    #[test]
    fn numeric_settings_are_inherited_by_a_callee_and_not_written_back() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"numeric digits 7\ncall sub\nsay 1/3\nexit\n\
                  sub:\nsay 1/3\nnumeric digits 3\nsay 1/3\nreturn\n",
            ),
            b"0.3333333\n0.333\n0.3333333\n".to_vec()
        );
    }

    /// I4: `TRACE` moved onto `Activation`, so a callee's own `trace off`
    /// dies with the callee. The caller's clauses echo again afterwards,
    /// which is what a single `Interp`-wide field could not do.
    #[test]
    fn a_callees_trace_setting_does_not_survive_its_return() {
        let mut interp = Interp::new();
        run_source_traced(
            &mut interp,
            b"call sub\nsay 'after'\nexit\nsub:\ntrace off\nsay 'quiet'\nreturn\n",
        )
        .expect("the program runs");
        assert_eq!(
            interp.trace,
            b"     1 *-* call sub\n\
              \x20    4 *-*   sub:\n\
              \x20    5 *-*   trace off\n\
              \x20    2 *-* say 'after'\n\
              \x20      >>>   \"after\"\n\
              \x20    3 *-* exit\n"
                .to_vec(),
            "the callee's `trace off` must silence only the callee"
        );
    }

    /// D2r's rule, at **three** caller indents rather than one: a callee's
    /// clauses echo at the calling clause's own printed indent plus two.
    /// `2 x depth` agrees with all of this at caller indent 0 and predicts 2
    /// where the truth is 4 and 6, which is why one shape is not enough.
    /// Every byte here was captured from the oracle.
    #[test]
    fn a_callees_clauses_echo_at_the_calling_clauses_indent_plus_two() {
        // Flat caller, two levels deep: 0 -> 2 -> 4.
        let mut interp = Interp::new();
        run_source_traced(
            &mut interp,
            b"aa = 1\ncall one\nexit\none:\nbb = 2\ncall two\nreturn\ntwo:\ncc = 3\nreturn\n",
        )
        .expect("the program runs");
        assert_eq!(
            interp.trace,
            b"     1 *-* aa = 1\n\
              \x20      >>>   \"1\"\n\
              \x20    2 *-* call one\n\
              \x20    4 *-*   one:\n\
              \x20    5 *-*   bb = 2\n\
              \x20      >>>     \"2\"\n\
              \x20    6 *-*   call two\n\
              \x20    8 *-*     two:\n\
              \x20    9 *-*     cc = 3\n\
              \x20      >>>       \"3\"\n\
              \x20   10 *-*     return\n\
              \x20    7 *-*   return\n\
              \x20    3 *-* exit\n"
                .to_vec()
        );

        // Caller two `DO` blocks deep: the callee echoes at 6, where
        // `2 x depth` predicts 2.
        let mut interp = Interp::new();
        run_source_traced(
            &mut interp,
            b"do\ndo\ncall sub\nend\nend\nexit\nsub:\ndd = 4\nreturn\n",
        )
        .expect("the program runs");
        assert_eq!(
            interp.trace,
            b"     1 *-* do\n\
              \x20    2 *-*   do\n\
              \x20    3 *-*     call sub\n\
              \x20    7 *-*       sub:\n\
              \x20    8 *-*       dd = 4\n\
              \x20      >>>         \"4\"\n\
              \x20    9 *-*       return\n\
              \x20    4 *-*   end\n\
              \x20    5 *-* end\n\
              \x20    6 *-* exit\n"
                .to_vec()
        );
    }

    /// **Review finding C1, Task 4 fix round 1.** `current_value_indent` is
    /// a fourth piece of level state `resolve_and_run_call` must restore on
    /// the way out, alongside `activation_indent`/`indent_offset`/
    /// `clause_line_override` (that function's own doc comment) -- and this
    /// is the shape that tells a version missing the restore apart from a
    /// correct one: **two** internal-function calls inside *one* clause
    /// (`ExprKind::Call`, Task 4). Before Task 4 at most one activation
    /// could be entered per clause, and the *next* clause's own
    /// `step_in_temps_frame` re-set the field before anything read it, so
    /// the gap was unobservable through `CALL` alone. Without the restore,
    /// `g`'s own base indent -- and everything computed from it: its
    /// clauses, its `RETURN`'s own value trace, and the enclosing `say`
    /// clause's own final `>>>` -- is derived from `f`'s last clause instead
    /// of the caller's own.
    ///
    /// Byte-exact against the oracle in a clean directory (source measured
    /// with a leading `trace r` clause enabling tracing, then every line
    /// number decremented by one to match `run_source_traced`'s own
    /// externally-set mode, which consumes no line of its own -- the same
    /// transformation this file's other `run_source_traced` expectations
    /// already rely on, checked here against `a_callees_clauses_echo_at_
    /// the_calling_clauses_indent_plus_two`'s own source with a real
    /// `trace r` prepended). This assertion compares raw `interp.trace`
    /// bytes and is **not** reachable by `corpus.rs`'s `normalize_stderr`
    /// (DEVIATION 0), which collapses exactly this class of indent
    /// difference -- see `phase-4b.txt`'s own entry for `lang/
    /// call_expression.rex` for why the corpus differential cannot be
    /// trusted to catch this at all.
    #[test]
    fn current_value_indent_is_restored_after_a_nested_expression_call() {
        let mut interp = Interp::new();
        run_source_traced(
            &mut interp,
            b"say f(1) + g(2)\nexit\nf: return 1\ng: return 2\n",
        )
        .expect("the program runs");
        assert_eq!(
            interp.trace,
            b"     1 *-* say f(1) + g(2)\n\
              \x20    3 *-*   f:\n\
              \x20    3 *-*   return 1\n\
              \x20      >>>     \"1\"\n\
              \x20    4 *-*   g:\n\
              \x20    4 *-*   return 2\n\
              \x20      >>>     \"2\"\n\
              \x20      >>>   \"3\"\n\
              \x20    2 *-* exit\n"
                .to_vec()
        );
    }

    /// `current_value_indent`'s own sibling field, found the identical way
    /// (Task 6 fix round 2): `current_clause_line` (bundled with it into
    /// `ClauseState`, whose own doc comment states the property that puts
    /// both fields in one save/restore rather than two) is a piece of level
    /// state `resolve_and_run_call` must restore on the way out, and shipped
    /// once already without that restore -- the second field of this exact
    /// shape to do so, after `current_value_indent` itself went unrestored
    /// until the test just above this one caught it at Task 4.
    ///
    /// The same shape catches it: two internal-function calls inside *one*
    /// clause. Without the restore, `g`'s own `SIGL` (`set_sigl`, reading
    /// `current_clause_line`) reads `f`'s own last line (`return 1`, line 5)
    /// instead of the calling clause's own (line 1) -- measured against the
    /// oracle in a clean directory, `rexx-run` on this exact source once
    /// read `sigl in g: 5` before this fix and `sigl in g: 1` after it, and
    /// the oracle has always answered `1`.
    #[test]
    fn current_clause_line_is_restored_after_a_nested_expression_call() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"say f(1) + g(2)\n\
                  exit\n\
                  f:\n\
                  say 'in f'\n\
                  return 1\n\
                  g:\n\
                  say 'sigl in g:' sigl\n\
                  return 2\n",
            ),
            b"in f\nsigl in g: 1\n3\n".to_vec()
        );
    }

    /// A returned value traces **twice**, at two different indents: once as
    /// the `RETURN`'s own value in the callee, once as the call's result in
    /// the caller. A bare `return` traces neither.
    #[test]
    fn a_returned_value_traces_in_the_callee_and_again_in_the_caller() {
        let mut interp = Interp::new();
        run_source_traced(&mut interp, b"call sub\nexit\nsub:\nreturn 42\n")
            .expect("the program runs");
        assert_eq!(
            interp.trace,
            b"     1 *-* call sub\n\
              \x20    3 *-*   sub:\n\
              \x20    4 *-*   return 42\n\
              \x20      >>>     \"42\"\n\
              \x20      >>>   \"42\"\n\
              \x20    2 *-* exit\n"
                .to_vec()
        );

        let mut interp = Interp::new();
        run_source_traced(&mut interp, b"call sub\nexit\nsub:\nreturn\n")
            .expect("the program runs");
        assert_eq!(
            interp.trace,
            b"     1 *-* call sub\n\
              \x20    3 *-*   sub:\n\
              \x20    4 *-*   return\n\
              \x20    2 *-* exit\n"
                .to_vec(),
            "a bare return produces no value line at either indent"
        );
    }

    /// The composition nobody had measured: a `CALL` **inside** an
    /// `INTERPRET` fragment. Each activation's echo carries its *own* line,
    /// so the enclosing fragment's line override has to be cleared for the
    /// callee -- leaving it in force prints the callee's three clauses as
    /// line 1 instead of 3, 4 and 5.
    #[test]
    fn a_call_inside_a_fragment_echoes_each_activations_own_line() {
        let mut interp = Interp::new();
        run_source_traced(
            &mut interp,
            b"interpret \"call sub\"\nexit\nsub:\nff = 7\nreturn\n",
        )
        .expect("the program runs");
        assert_eq!(
            interp.trace,
            b"     1 *-* interpret \"call sub\"\n\
              \x20      >>>   \"call sub\"\n\
              \x20    1 *-* call sub\n\
              \x20    3 *-*   sub:\n\
              \x20    4 *-*   ff = 7\n\
              \x20      >>>     \"7\"\n\
              \x20    5 *-*   return\n\
              \x20    2 *-* exit\n"
                .to_vec()
        );
    }

    /// The other direction: an `INTERPRET` **inside** a called routine. The
    /// fragment adds no indent of its own on top of the activation's, so all
    /// four of the callee's lines sit at 2.
    #[test]
    fn an_interpret_inside_a_callee_runs_at_the_callees_own_level() {
        let mut interp = Interp::new();
        run_source_traced(
            &mut interp,
            b"call sub\nexit\nsub:\ninterpret \"gg = 8\"\nreturn\n",
        )
        .expect("the program runs");
        assert_eq!(
            interp.trace,
            b"     1 *-* call sub\n\
              \x20    3 *-*   sub:\n\
              \x20    4 *-*   interpret \"gg = 8\"\n\
              \x20      >>>     \"gg = 8\"\n\
              \x20    4 *-*   gg = 8\n\
              \x20      >>>     \"8\"\n\
              \x20    5 *-*   return\n\
              \x20    2 *-* exit\n"
                .to_vec()
        );
    }

    /// I12's `CALL` half: each activation seals its own level, so the report
    /// echoes one clause per activation, innermost first, each at its own
    /// indent. Two calls deep from inside a `DO` gives 6, 4, 2 -- the same
    /// three-deep transcript the oracle prints.
    #[test]
    fn the_report_echoes_one_clause_per_activation_innermost_first() {
        let mut interp = Interp::new();
        run_source(
            &mut interp,
            b"do\ncall one\nend\nexit\none:\ncall two\nreturn\ntwo:\nsay 1/0\nreturn\n",
        )
        .unwrap_err();
        let sealed: Vec<(usize, Vec<u8>, usize)> = interp
            .failure_sites
            .iter()
            .chain(interp.failure_site.iter())
            .map(|s| (s.line, s.text.clone(), s.indent))
            .collect();
        assert_eq!(
            sealed,
            vec![
                (9, b"say 1/0".to_vec(), 6),
                (6, b"call two".to_vec(), 4),
                (2, b"call one".to_vec(), 2),
            ]
        );
    }

    /// Arguments are evaluated in the caller, before the callee starts. Not
    /// observable through `USE ARG`/`ARG()` in this phase -- both still fail
    /// loudly -- but a failing argument is: the condition is 42.3 and it is
    /// reported against the `CALL` clause, not against anything in `sub`.
    #[test]
    fn a_calls_arguments_are_evaluated_in_the_caller() {
        let mut interp = Interp::new();
        let failure = run_source(&mut interp, b"call sub 1/0\nexit\nsub:\nreturn\n").unwrap_err();
        assert!(
            matches!(&failure, Failure::Raised(raised) if raised.number == 42),
            "an argument that raises must surface as its own condition, not run the callee: {failure:?}"
        );
        let site = interp.failure_site.expect("a site was resolved");
        assert_eq!((site.line, site.text), (1, b"call sub 1/0".to_vec()));
    }

    /// The quoted form bypasses the label search entirely, so `call "SUB"`
    /// with `sub:` present is *not* a call. 4b's answer is the loud
    /// builtin/external fallback naming 4c; the oracle's own is Error 43.1,
    /// which is a claim about what is *not* a builtin either and so not
    /// this phase's to make.
    #[test]
    fn a_quoted_call_name_never_reaches_the_label_table() {
        let mut interp = Interp::new();
        let failure = run_source(
            &mut interp,
            b"call \"SUB\"\nexit\nsub:\nsay 'ran'\nreturn\n",
        )
        .unwrap_err();
        assert!(
            matches!(&failure, Failure::Loud(loud) if loud.message.ends_with("is not implemented (4c)")),
            "expected the 4c fallback, got {failure:?}"
        );
        assert!(interp.out.is_empty(), "the label must not have run");

        // The unquoted spelling of the same name does reach it, which is
        // what makes the assertion above about the quotes and not about
        // `sub:` being unreachable.
        let mut interp = Interp::new();
        assert_eq!(
            say_output(&mut interp, b"call SUB\nexit\nsub:\nsay 'ran'\nreturn\n"),
            b"ran\n".to_vec()
        );
    }

    /// `CALL (expr)` searches the label table, but with the value **verbatim**
    /// -- the two halves pull opposite ways from the quoted form and both are
    /// measured. Lower case finds nothing even though `sub:` is stored
    /// upcased.
    #[test]
    fn a_dynamic_call_target_searches_labels_with_the_value_verbatim() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"nm = 'SUB'\ncall (nm)\nexit\nsub:\nsay 'ran'\nreturn\n"
            ),
            b"ran\n".to_vec()
        );

        let mut interp = Interp::new();
        let failure = run_source(
            &mut interp,
            b"nm = 'sub'\ncall (nm)\nexit\nsub:\nsay 'ran'\nreturn\n",
        )
        .unwrap_err();
        assert!(
            matches!(&failure, Failure::Loud(loud) if loud.message.contains("sub")),
            "the unupcased value must not match the upcased label: {failure:?}"
        );
        assert!(interp.out.is_empty(), "the label must not have run");
    }

    /// `SIGNAL` out of a `DO`, landing on a label past it. Unlike `LEAVE`,
    /// there is no search and no name to match -- every enclosing construct
    /// is abandoned unconditionally, so the loop's own later iterations
    /// never happen and neither does the clause right after `END`.
    ///
    /// I1 (Task 6 fix round 1): a direct regression for `Flow::Signal`
    /// versus reusing `Flow::Goto`. Collapsing the two `Ok(Flow::Signal(
    /// target))` sites in the `Signal` step arm to `Ok(Flow::Goto(target))`
    /// left every test in this file, and the whole workspace, green --
    /// including every other `SIGNAL` test above and below this one, none
    /// of which happens to have a fragment whose own instruction count
    /// reaches the enclosing label's own index. This one does, on purpose:
    /// `here:` sits at enclosing body index 2 (`say 'A'` is 0, `interpret`
    /// is 1), and the fragment `"nop; signal here; say 'WRONG BRANCH RAN'"`
    /// has 3 instructions, so the escaping target (2) satisfies
    /// `run_bounded`'s own absorption guard (`target >= start(0) && target
    /// <= end(3)`) against the *fragment's* range. A `Goto`-collapsed build
    /// -- verified directly, reverted before committing -- absorbs the jump
    /// as its own, resumes stepping the fragment's own third instruction,
    /// and prints `WRONG BRANCH RAN` in the middle; the correct build
    /// escapes past the fragment entirely and never prints it.
    ///
    /// **No second, self-referential ("g2") variant is added here.** The
    /// review that found this also found a shape where the escaping target
    /// lands on the fragment's *own* `SIGNAL` instruction (a label at
    /// enclosing index 0, a one-instruction fragment `"signal top"`) --
    /// under `Goto` reuse that does not print a wrong answer, it spins
    /// forever: `run_bounded`'s `while pc < end` loop has no iteration
    /// budget, and landing back on the same deterministic instruction
    /// reproduces the identical `Goto` every pass. That is true of *any*
    /// collision where the absorbed target is at or before the `SIGNAL`'s
    /// own position inside the fragment, not only the minimal one -- moving
    /// the target forward past the `SIGNAL` (this test's own shape) is what
    /// makes the wrong run terminate at all. There is no bounded encoding of
    /// the backward/self-referential shape as a live-executed test, only a
    /// choice between not testing it and risking a hang the moment this
    /// regression guard itself regresses; this crate's own precedent
    /// (`MAX_ACTIVATION_DEPTH`, D19/I6) is to convert an unbounded case into
    /// a bounded, reportable one rather than accept an unbounded test, and
    /// nothing here does that for a bare `while` loop's own iteration count.
    /// Documented instead of encoded: this doc comment and `Flow::Signal`'s
    /// own are where the fact lives.
    #[test]
    fn signal_out_of_a_fragment_does_not_collide_with_the_fragments_own_index_space() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"say 'A'\n\
                  interpret \"nop; signal here; say 'WRONG BRANCH RAN'\"\n\
                  here: say 'landed correctly'\n",
            ),
            b"A\nlanded correctly\n".to_vec()
        );
    }

    /// I3 (Task 6 fix round 1): `SIGNAL` out of a `SELECT`, the one Step 1
    /// shape the original landing measured for `DO` and `INTERPRET` but
    /// never for `SELECT` -- `leave_select` (`run.rs`) is one of the
    /// forwarding sites `Flow::Signal`'s own design argument depends on, and
    /// it was the only one with no witness. Measured (source with a leading
    /// `trace r` clause, lines decremented by one as every other traced test
    /// in this file already does): `SELECT` is abandoned exactly like `DO`
    /// is, unconditionally, and `SIGL` (C1, this same fix round) reads back
    /// the `WHEN`'s own line, all three sub-clauses (`WHEN`, `THEN`,
    /// `SIGNAL`) sharing that one source line.
    #[test]
    fn signal_out_of_a_select_unwinds_it_and_lands_on_its_label() {
        let mut interp = Interp::new();
        run_source_traced(
            &mut interp,
            b"say 'before'\n\
              select\n\
              \x20 when 1 = 1 then signal there\n\
              \x20 otherwise\n\
              \x20   say 'not reached'\n\
              end\n\
              say 'after select, not reached'\n\
              exit\n\
              there:\n\
              say 'reached there sigl:' sigl\n",
        )
        .expect("the program runs");
        assert_eq!(
            interp.trace,
            b"     1 *-* say 'before'\n\
              \x20      >>>   \"before\"\n\
              \x20    2 *-* select\n\
              \x20    3 *-*   when 1 = 1 \n\
              \x20      >>>     \"1\"\n\
              \x20    3 *-*     then\n\
              \x20    3 *-*       signal there\n\
              \x20    9 *-* there:\n\
              \x20   10 *-* say 'reached there sigl:' sigl\n\
              \x20      >>>   \"reached there sigl: 3\"\n"
                .to_vec()
        );
        assert_eq!(interp.out, b"before\nreached there sigl: 3\n".to_vec());
    }

    /// C1 (Task 6 fix round 1): `SIGL`. The oracle's own `RexxActivation::
    /// signalTo`/`internalCall` (`execution/RexxActivation.cpp`, read
    /// directly) both call `new_integer(lineNum)` at the point of transfer
    /// -- an integer object, which is why `set_sigl` uses `self.text`
    /// rather than `self.number`: measured, a `SIGL` value of `22` still
    /// renders `22` under `NUMERIC DIGITS 1`, where the identical magnitude
    /// as an arithmetic result would round to `2E+1`.
    ///
    /// Five shapes, each measured against the oracle in a clean directory
    /// before being pinned here:
    #[test]
    fn sigl_is_set_at_every_control_transfer() {
        // Uninitialised until the first transfer, like any other variable;
        // `SIGNAL` sets it to its own line.
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"say 'sigl before:' sigl\nsignal there\nsay 'no'\nthere:\nsay 'sigl after:' sigl\n",
            ),
            b"sigl before: SIGL\nsigl after: 2\n".to_vec()
        );

        // `CALL` sets it too, visible inside the callee (D9r's shared pool)
        // and left set after the callee returns -- an ordinary variable,
        // never restored at the activation boundary.
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"say 'before:' sigl\n\
                  call sub\n\
                  say 'after call:' sigl\n\
                  exit\n\
                  \n\
                  sub:\n\
                  say 'in sub sigl:' sigl\n\
                  return\n",
            ),
            b"before: SIGL\nin sub sigl: 2\nafter call: 2\n".to_vec()
        );

        // From inside an `INTERPRET` fragment, `SIGL` reads the *enclosing*
        // `INTERPRET` clause's own line, not any line internal to the
        // fragment -- matching the oracle's own `signalTo`, which delegates
        // a `SIGNAL` fired inside an interpret-created activation to its
        // parent, whose own currently-executing instruction is the
        // `INTERPRET` itself (this crate reproduces the observable answer
        // through `current_clause_line`/`clause_line_override` instead,
        // without adopting that nested-activation architecture).
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"say 'before:' sigl\n\
                  interpret \"signal there\"\n\
                  say 'no'\n\
                  there:\n\
                  say 'sigl after signal-in-fragment:' sigl\n",
            ),
            b"before: SIGL\nsigl after signal-in-fragment: 2\n".to_vec()
        );

        // The expression-call form (`f(1)`, Task 4) sets it exactly like
        // `CALL` does.
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"say 'before:' sigl\n\
                  say f(1)\n\
                  say 'after:' sigl\n\
                  exit\n\
                  f: return 'called, sigl=' || sigl\n",
            ),
            b"before: SIGL\ncalled, sigl=2\nafter: 2\n".to_vec()
        );

        // `PROCEDURE` isolates it exactly like any other variable: the
        // callee's own `SIGL` starts uninitialised in its own frame, and
        // the caller's `SIGL` (already set by the `CALL` itself, before the
        // callee ever ran) is unaffected by whatever the isolated callee
        // does with its own copy.
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"call sub\n\
                  say 'main sigl after sub returns:' sigl\n\
                  exit\n\
                  \n\
                  sub:\n\
                  procedure\n\
                  say 'sub sigl (isolated):' sigl\n\
                  return\n",
            ),
            b"sub sigl (isolated): SIGL\nmain sigl after sub returns: 1\n".to_vec()
        );
    }

    /// **Fires on the loop's first pass, deliberately** -- a second pass
    /// through a `Controlled` (`TO`-style) `DO`/`LOOP` retraces two further
    /// `>>>` lines this crate does not yet reproduce (the documented "KNOWN
    /// GAP" at `loop_advance`'s own `Controlled` arm, unrelated to `SIGNAL`
    /// and out of this task's scope), and a witness that reached a second
    /// pass would be asserting that gap's own wrong output rather than
    /// `SIGNAL`'s. Measured (source with a leading `trace r` clause, every
    /// line number then decremented by one to match `run_source_traced`'s
    /// own externally-set mode -- `current_value_indent_is_restored_after_a_
    /// nested_expression_call`'s own doc comment has the transformation):
    /// the `SIGNAL` clause itself traces with no `>>>` line, unlike `CALL`'s
    /// dynamic form or `RETURN` with a value, because it produces nothing.
    #[test]
    fn signal_unwinds_a_nested_do_and_lands_on_its_label() {
        let mut interp = Interp::new();
        run_source_traced(
            &mut interp,
            b"say 'before'\n\
              do i = 1 to 3\n\
              \x20 if i = 1 then signal there\n\
              \x20 say 'i=' i\n\
              end\n\
              say 'after loop, not reached'\n\
              exit\n\
              there:\n\
              say 'reached there'\n",
        )
        .expect("the program runs");
        assert_eq!(
            interp.trace,
            b"     1 *-* say 'before'\n\
              \x20      >>>   \"before\"\n\
              \x20    2 *-* do i = 1 to 3\n\
              \x20      >K>   \"TO\" => \"3\"\n\
              \x20    3 *-*   if i = 1 \n\
              \x20      >>>     \"1\"\n\
              \x20    3 *-*     then\n\
              \x20    3 *-*       signal there\n\
              \x20    8 *-* there:\n\
              \x20    9 *-* say 'reached there'\n\
              \x20      >>>   \"reached there\"\n"
                .to_vec()
        );
        assert_eq!(interp.out, b"before\nreached there\n".to_vec());
    }

    /// A `SIGNAL` target that matches no label in the running activation's
    /// own body is Error 16.1, "Label not found" -- unlike `CALL`'s own
    /// unresolved name, which still has a builtin/external fallback to defer
    /// to (`resolve_and_run_call`'s own doc), `SIGNAL` has none, so this is
    /// the oracle's real answer and not a loud gap.
    #[test]
    fn signal_to_an_undefined_label_raises_16_1() {
        let mut interp = Interp::new();
        let failure = run_source(
            &mut interp,
            b"say 'before'\nsignal nowhere\nsay 'not reached'\n",
        )
        .unwrap_err();
        assert!(
            matches!(
                &failure,
                Failure::Raised(raised)
                    if raised.number == 16
                        && raised.sub == 1
                        && raised.additional == vec!["NOWHERE".to_string()]
            ),
            "expected 16.1 naming \"NOWHERE\", got {failure:?}"
        );
        assert_eq!(
            interp.out,
            b"before\n".to_vec(),
            "the clause after SIGNAL must not run"
        );
    }

    /// A quoted `SIGNAL` target searches the label table -- unlike a quoted
    /// `CALL` target, which never does at all (`a_quoted_call_name_never_
    /// reaches_the_label_table`, above) -- but case-sensitively against the
    /// label's own upcased spelling, so a lowercase quoted spelling still
    /// misses and raises 16.1 naming the verbatim quoted text, rather than
    /// taking `CALL`'s own loud 4c fallback. A bare symbol is upcased at
    /// parse time regardless of its own case, and an uppercase quoted
    /// spelling matches for the same reason a lowercase one does not.
    #[test]
    fn a_quoted_signal_label_searches_case_sensitively_unlike_a_quoted_call() {
        let mut interp = Interp::new();
        let failure = run_source(
            &mut interp,
            b"signal \"sub\"\nsay 'not reached'\nsub:\nsay 'reached'\n",
        )
        .unwrap_err();
        assert!(
            matches!(
                &failure,
                Failure::Raised(raised)
                    if raised.number == 16
                        && raised.sub == 1
                        && raised.additional == vec!["sub".to_string()]
            ),
            "expected 16.1 naming the verbatim quoted text \"sub\": {failure:?}"
        );

        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"signal Sub\nsay 'not reached'\nsub:\nsay 'reached'\n"
            ),
            b"reached\n".to_vec(),
            "a bare, mixed-case symbol is upcased at parse time and matches"
        );

        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"signal \"SUB\"\nsay 'not reached'\nsub:\nsay 'reached'\n"
            ),
            b"reached\n".to_vec(),
            "the uppercase quoted spelling matches -- it is the case, not the quoting"
        );
    }

    /// The composition nobody had measured: `SIGNAL` from inside a called
    /// routine, targeting a label written back in the *caller's* own text.
    ///
    /// **It reaches, and that is not `SIGNAL` crossing an activation
    /// boundary on its own.** At this phase every internal `CALL` target
    /// shares its caller's exact body and label table (`resolve_and_run_
    /// call`'s own D9r comment: no `::routine` directive gives a callee one
    /// of its own yet), so `resolve_signal_target`'s "search the running
    /// activation's own body" finds `caller_label:` from inside `sub` for
    /// the mundane reason that `sub`'s own body *is* the caller's. Measured
    /// against the oracle rather than assumed, per this phase's own method.
    ///
    /// **And it never returns to the original `CALL`'s own next clause.**
    /// `SIGNAL`, unlike `RETURN`, never pops the activation it fires in --
    /// so once the label's own code runs out of further instructions, the
    /// *callee's* activation falls off the end, which ends the whole
    /// program (`Ended::Exited`'s own doc comment), not merely the call.
    #[test]
    fn signal_from_a_called_routine_reaches_a_label_in_the_shared_body_and_never_returns() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"call sub\n\
                  say 'after call, not reached'\n\
                  exit\n\
                  \n\
                  sub:\n\
                  say 'in sub'\n\
                  signal caller_label\n\
                  say 'sub not reached'\n\
                  return\n\
                  \n\
                  caller_label:\n\
                  say 'caller label reached'\n",
            ),
            b"in sub\ncaller label reached\n".to_vec()
        );
    }

    /// `SIGNAL` out of an `INTERPRET` fragment reaches an enclosing label --
    /// unlike `LEAVE`/`ITERATE`, whose own search stops dead at the fragment
    /// boundary (`run_fragment`'s own doc comment has that transcript
    /// table), `SIGNAL` is forwarded like `Exit`/`Return` because it is a
    /// new `Flow` variant with no arm of its own at that boundary
    /// (`Flow::Signal`'s own doc comment). Measured (source with a leading
    /// `trace r` clause, lines decremented by one to match `run_source_
    /// traced`, as above): the fragment's own clause echoes at the enclosing
    /// `INTERPRET`'s line, then control resumes at the enclosing label's own
    /// line, exactly like `interpret "call sub"` already does for `CALL`.
    #[test]
    fn signal_escapes_an_interpret_fragment_to_reach_an_enclosing_label() {
        let mut interp = Interp::new();
        run_source_traced(
            &mut interp,
            b"say 'before'\n\
              interpret \"signal there\"\n\
              say 'not reached'\n\
              exit\n\
              there:\n\
              say 'reached there'\n",
        )
        .expect("the program runs");
        assert_eq!(
            interp.trace,
            b"     1 *-* say 'before'\n\
              \x20      >>>   \"before\"\n\
              \x20    2 *-* interpret \"signal there\"\n\
              \x20      >>>   \"signal there\"\n\
              \x20    2 *-* signal there\n\
              \x20    5 *-* there:\n\
              \x20    6 *-* say 'reached there'\n\
              \x20      >>>   \"reached there\"\n"
                .to_vec()
        );
        assert_eq!(interp.out, b"before\nreached there\n".to_vec());
    }

    /// The failure twin of the test above: a `SIGNAL` inside an `INTERPRET`
    /// fragment that matches no label reports **both** clauses, innermost
    /// first, each carrying the enclosing `INTERPRET`'s own line -- the same
    /// shape `run_fragment`'s own doc comment tables for `LEAVE`/`ITERATE`,
    /// now measured for `SIGNAL` too.
    #[test]
    fn signal_to_an_undefined_label_inside_a_fragment_reports_both_clauses() {
        let mut interp = Interp::new();
        let failure = run_source(
            &mut interp,
            b"say 'before'\ninterpret \"signal nowhere\"\nsay 'not reached'\n",
        )
        .unwrap_err();
        assert!(matches!(
            &failure,
            Failure::Raised(raised) if raised.number == 16 && raised.sub == 1
        ));
        let sealed: Vec<(usize, Vec<u8>)> = interp
            .failure_sites
            .iter()
            .chain(interp.failure_site.iter())
            .map(|s| (s.line, s.text.clone()))
            .collect();
        assert_eq!(
            sealed,
            vec![
                (2, b"signal nowhere".to_vec()),
                (2, b"interpret \"signal nowhere\"".to_vec()),
            ]
        );
        assert_eq!(interp.out, b"before\n".to_vec());
    }

    /// `SIGNAL VALUE`'s own `>K>` line -- `"VALUE" => text`, at the clause's
    /// own indent with no extra `+2` the way `WHILE`/`UNTIL` carry, since
    /// nothing here is evaluated as part of an *enclosing* instruction's own
    /// step the way a loop's condition is. Measured one `DO` deep (lines
    /// decremented by one as above): once traced, the rendered value is
    /// searched exactly like a bare `SIGNAL label`'s own bytes.
    #[test]
    fn signal_value_traces_its_own_keyword_line_and_then_searches_like_label() {
        let mut interp = Interp::new();
        run_source_traced(
            &mut interp,
            b"target = 'THERE'\n\
              do i = 1 to 1\n\
              \x20 signal value target\n\
              end\n\
              say 'not reached'\n\
              exit\n\
              there:\n\
              say 'reached, one do deep'\n",
        )
        .expect("the program runs");
        assert_eq!(
            interp.trace,
            b"     1 *-* target = 'THERE'\n\
              \x20      >>>   \"THERE\"\n\
              \x20    2 *-* do i = 1 to 1\n\
              \x20      >K>   \"TO\" => \"1\"\n\
              \x20    3 *-*   signal value target\n\
              \x20      >K>     \"VALUE\" => \"THERE\"\n\
              \x20    7 *-* there:\n\
              \x20    8 *-* say 'reached, one do deep'\n\
              \x20      >>>   \"reached, one do deep\"\n"
                .to_vec()
        );
        assert_eq!(interp.out, b"reached, one do deep\n".to_vec());
    }

    /// `SIGNAL VALUE`'s target gets **no shape check at all** before the
    /// label search: a number, an empty string and an ordinary non-label
    /// string all raise 16.1 naming that exact rendered text, none of them a
    /// different error -- and the search is case-sensitive exactly like a
    /// quoted `SIGNAL label`'s own (`a_quoted_signal_label_searches_case_
    /// sensitively_unlike_a_quoted_call`, above), so a lowercase value does
    /// not match a label stored upcased even under the identical spelling.
    #[test]
    fn signal_value_targets_that_match_no_label_all_raise_16_1_naming_the_rendered_text() {
        for (expr, expected) in [
            ("123", "123"),
            ("''", ""),
            ("'no such label'", "no such label"),
        ] {
            let mut interp = Interp::new();
            let source = format!("target = {expr}\nsignal value target\nsay 'not reached'\n");
            let failure = run_source(&mut interp, source.as_bytes()).unwrap_err();
            assert!(
                matches!(
                    &failure,
                    Failure::Raised(raised)
                        if raised.number == 16
                            && raised.sub == 1
                            && raised.additional == vec![expected.to_string()]
                ),
                "{expr}: expected 16.1 naming {expected:?}, got {failure:?}"
            );
        }

        let mut interp = Interp::new();
        let failure = run_source(
            &mut interp,
            b"target = 'there'\n\
              signal value target\n\
              say 'not reached'\n\
              exit\n\
              there:\n\
              say 'reached'\n",
        )
        .unwrap_err();
        assert!(
            matches!(
                &failure,
                Failure::Raised(raised)
                    if raised.number == 16
                        && raised.sub == 1
                        && raised.additional == vec!["there".to_string()]
            ),
            "a lowercase value must not match the upcased label THERE: {failure:?}"
        );
    }

    /// The body selector's `Some(index)` half, which no execution path can
    /// construct yet (`Activation::body`'s own doc has why). Exercised
    /// directly so the arm is not merely written: a parsed `::routine` has a
    /// body, and the selector resolves to *that* body rather than to `main`.
    #[test]
    fn the_body_selector_resolves_a_routine_directive_and_rejects_a_bad_index() {
        let program =
            parse_program(b"call foo\nexit\n::routine foo\nsay 'in foo'\n".to_vec()).unwrap();
        let main = body_of(&program, None).expect("the main body always resolves");
        assert_eq!(main.instructions.len(), program.main.instructions.len());

        let routine = body_of(&program, Some(0)).expect("the routine directive has a body");
        assert_eq!(
            routine.instructions.len(),
            1,
            "the routine's own body is its one `say`, not the main body's two clauses"
        );
        assert!(
            !std::ptr::eq(routine, main),
            "a routine selector must not resolve to the main body"
        );

        assert!(
            body_of(&program, Some(1)).is_none(),
            "an out-of-range selector resolves to nothing rather than panicking"
        );
    }

    // ---- PROCEDURE, PROCEDURE EXPOSE, USE and the variable reference
    // (4b Task 5) ----
    //
    // Every program below runs its `PROCEDURE` through a real `CALL`, and
    // not because a `CALL` reads better: `run_source` drives the body
    // through `run_bounded`, and only `run_activation` grants the
    // first-instruction permission a `PROCEDURE` needs. A `PROCEDURE`
    // reached any other way is error 17.1 -- which is the oracle's own
    // answer too, measured, and what
    // `a_procedure_that_is_not_a_calls_first_instruction_raises_17_1`
    // asserts.
    //
    // **No value in these programs equals its own variable's derived name.**
    // An unexposed unset read yields the name, so a witness whose exposed
    // variable holds, say, `W` in `w` cannot tell exposure from
    // non-exposure. Every literal here is a hyphenated word no derived name
    // can collide with.

    #[test]
    fn procedure_isolates_and_expose_aliases_the_caller_entry() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"v = 'caller-v'\ncall sub\nsay v w\nexit\n\
                  sub: procedure expose w\nv = 'callee-v'\nw = 'callee-w'\nreturn\n",
            ),
            b"caller-v callee-w\n".to_vec(),
            "V is the callee's own variable and must not have escaped; W is \
             exposed and must have"
        );
    }

    /// Exposure is transitive: `a` exposes `n` to `b`, `b` exposes the same
    /// `n` to `c`, and `c`'s write is visible in `a`.
    ///
    /// Measured on the oracle. Binding `c`'s `n` to `b`'s frame instead of
    /// chasing `b`'s own alias passes at one level and gives `from-a` here.
    #[test]
    fn exposure_is_transitive_through_an_intermediate_procedure() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"n = 'from-a'\ncall bee\nsay 'a sees:' n\nexit\n\
                  bee: procedure expose n\ncall cee\nsay 'bee sees after c:' n\nreturn\n\
                  cee: procedure expose n\nn = 'set-by-cee'\nreturn\n",
            ),
            b"bee sees after c: set-by-cee\na sees: set-by-cee\n".to_vec()
        );
    }

    /// **The program that refuted "a bitset plus one target `SlotFrame`".**
    ///
    /// One `PROCEDURE` exposes two names that live in two different frames:
    /// `n` chases through `bee`'s alias up to `a`, while `m` is `bee`'s own
    /// local and stops there. Measured on the oracle -- `bee` sees both of
    /// `cee`'s writes and `a` sees only `n`'s.
    ///
    /// Any design carrying a single target frame per callee gets exactly one
    /// of the two names right, whichever frame it picked, so this is the
    /// test that cannot pass by accident. It is also why `RootSet`'s
    /// redirect is a per-slot `Vec<Option<usize>>`.
    #[test]
    fn one_procedure_can_expose_names_living_in_two_different_frames() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"n = 'from-a'\nm = 'from-a-m'\ncall bee\nsay 'a sees:' n m\nexit\n\
                  bee: procedure expose n\nm = 'from-bee-m'\ncall cee\n\
                  say 'bee sees:' n m\nreturn\n\
                  cee: procedure expose n m\nn = 'set-by-cee'\nm = 'set-by-cee-m'\nreturn\n",
            ),
            b"bee sees: set-by-cee set-by-cee-m\na sees: set-by-cee from-a-m\n".to_vec(),
            "N must reach A and M must stop at BEE, from one PROCEDURE"
        );
    }

    /// `EXPOSE (v)` is plural and also exposes `v` itself. Both halves are
    /// measured; `DROP (v)` took the identical correction in 4a.
    #[test]
    fn the_indirect_expose_form_is_plural_and_exposes_its_own_selector() {
        // Plural, with GAMMA as the control: it is never named and must not
        // be exposed, so a version that exposed everything passes the first
        // two assertions and fails on it.
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"list = 'ALPHA BETA'\nalpha = 'a-in-caller'\nbeta = 'b-in-caller'\n\
                  gamma = 'g-in-caller'\ncall sub\nsay alpha beta gamma\nexit\n\
                  sub: procedure expose (list)\n\
                  alpha = 'a-set'\nbeta = 'b-set'\ngamma = 'g-set'\nreturn\n",
            ),
            b"a-set b-set g-in-caller\n".to_vec()
        );

        // The selector itself. The callee reads `v` as `zzz` (the caller's
        // value, so `v` is exposed) and writes both names, and the caller
        // sees both writes.
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"v = 'zzz'\nzzz = 'z-in-caller'\ncall sub\n\
                  say 'caller v:' v 'caller zzz:' zzz\nexit\n\
                  sub: procedure expose (v)\nsay 'callee v:' v 'callee zzz:' zzz\n\
                  v = 'v-set'\nzzz = 'zzz-set'\nreturn\n",
            ),
            b"callee v: zzz callee zzz: z-in-caller\ncaller v: v-set caller zzz: zzz-set\n"
                .to_vec()
        );
    }

    /// The five stem transcripts D9r records, all measured on the oracle.
    ///
    /// Their common point is that `EXPOSE` aliases the caller's **variable
    /// entry**, not the stem *object*, which is what the `drop` pair pins:
    /// `drop a.` in the callee rebinds the caller's entry to a fresh stem,
    /// while a second variable holding the old object still sees the old
    /// tail. That is why `stem_drop`'s `replace_stem(name, None)` shape is
    /// correct under exposure and must not become a slot clear.
    #[test]
    fn an_exposed_stem_aliases_the_callers_entry_not_the_object() {
        for (source, expected, why) in [
            (
                &b"a.1 = 'kept'\ncall sub\nsay a.1\nexit\n\
                   sub: procedure expose a.\na.1 = 'changed'\nreturn\n"[..],
                &b"changed\n"[..],
                "a tail written in the callee is visible through the caller's stem",
            ),
            (
                b"a.1 = 'kept'\ncall sub\nsay a.1\nexit\n\
                  sub: procedure expose a.\na. = 'wiped'\nreturn\n",
                b"wiped\n",
                "a whole-stem assignment in the callee replaces the caller's stem",
            ),
            (
                b"a.1 = 'kept'\ncall sub\nsay a.1\nexit\n\
                  sub: procedure expose a.\ndrop a.\nreturn\n",
                b"A.1\n",
                "DROP of an exposed stem leaves the caller's stem looking untouched",
            ),
            (
                b"a.1 = 'orig'\nkeep. = a.\ncall sub\nsay a.1 keep.1\nexit\n\
                  sub: procedure expose a.\ndrop a.\nreturn\n",
                b"A.1 orig\n",
                "the DROP rebinds the entry; KEEP. still holds the old object, which \
                 is what distinguishes rebinding from clearing",
            ),
            (
                b"a.1 = 'from-caller'\nother.1 = 'not-exposed'\ncall sub\nexit\n\
                  sub: procedure expose a.\nsay 'callee reads:' a.1 other.1\nreturn\n",
                b"callee reads: from-caller OTHER.1\n",
                "the callee reads the exposed stem's tail and derives the name of the \
                 unexposed one",
            ),
        ] {
            let mut interp = Interp::new();
            assert_eq!(say_output(&mut interp, source), expected.to_vec(), "{why}");
        }
    }

    /// A name the plan never saw, exposed through the indirect form, has to
    /// keep resolving to the same slot on both sides of the return.
    ///
    /// `ZQXW` appears in no instruction of either routine -- only inside
    /// string literals -- so the plan has no slot for it and both sides
    /// reach it only through a run-time binding. Measured on the oracle.
    /// This is the case `exec_procedure`'s write-back of `extra` exists for,
    /// and the one that would otherwise need a non-top `grow_slots`.
    #[test]
    fn a_computed_expose_of_a_name_no_instruction_mentions_survives_the_return() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"nm = 'ZQXW'\ncall sub\ninterpret \"say 'caller:' zqxw\"\nexit\n\
                  sub: procedure expose (nm)\ninterpret \"zqxw = 'set-in-callee'\"\nreturn\n",
            ),
            b"caller: set-in-callee\n".to_vec()
        );
    }

    /// 17.1 at every shape but the legal one, and labels are transparent.
    ///
    /// All five measured on the oracle. The `nop` case beside the two-label
    /// case is what shows the rule is "first instruction *executed*" with
    /// labels not counting, rather than "first instruction in the body".
    #[test]
    fn a_procedure_that_is_not_a_calls_first_instruction_raises_17_1() {
        for (source, why) in [
            (
                &b"say 'top'\nprocedure\n"[..],
                "at top level, with no call at all",
            ),
            (
                b"say 'main'\nsub:\nprocedure\n",
                "fallen into rather than called",
            ),
            (
                b"call sub\nexit\nsub:\nnop\nprocedure\nreturn\n",
                "after a NOP in a called routine",
            ),
            (
                b"call sub\nexit\nsub: interpret \"procedure\"\nreturn\n",
                "inside a fragment, which does not inherit its host's permission",
            ),
        ] {
            let mut interp = Interp::new();
            let failure = run_source(&mut interp, source).unwrap_err();
            let Failure::Raised(raised) = failure else {
                panic!("expected Raised for {why}, got {failure:?}");
            };
            assert_eq!((raised.number, raised.sub), (17, 1), "{why}");
        }

        // Two labels between the CALL and the PROCEDURE: legal, measured.
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"outer = 'caller'\ncall sub\nsay outer\nexit\n\
                  sub:\nlbl2:\nprocedure\nouter = 'callee'\nreturn\n",
            ),
            b"caller\n".to_vec(),
            "a label neither grants the permission nor spends it, and the PROCEDURE \
             still isolated"
        );
    }

    /// An isolated callee's frame is released on the way out, on the error
    /// path as well as the ordinary one.
    ///
    /// Asserted against the root set's own slot count rather than through
    /// output, because a leak is invisible in a program's bytes: the run
    /// would still be correct and would simply hold one frame per call
    /// forever, which `do 100000; call sub; end` turns into 100,000 rooted
    /// frames. Both paths are checked here because they share the one
    /// `pop_slots` call whose position in `resolve_and_run_call` is the whole
    /// point -- outside the `Ok` arm, not inside it.
    ///
    /// The property is that frames balance, so this counts **frames** and not
    /// slots. A slot count moves for reasons that are not leaks -- the first
    /// `CALL` in a program that never writes `RESULT` grows the top frame by
    /// one to hold it, measured while writing this test -- and it moves by a
    /// *different* amount on the error path, which never reaches that write.
    /// `RootSet::live_frames` has no such confounder.
    ///
    /// `run_source` leaves the top-level frame standing (only `Interp::run`
    /// pops that one), so one frame is the correct answer for a balanced run
    /// and each unreleased callee would add one more.
    #[test]
    fn an_isolated_callees_frame_is_released_on_both_paths() {
        let mut interp = Interp::new();
        say_output(
            &mut interp,
            b"zz = 1\ncall sub\ncall sub\ncall sub\nexit\nsub: procedure\nyy = 2\nreturn\n",
        );
        assert_eq!(
            interp.roots.live_frames(),
            1,
            "three isolated calls must leave only the top-level frame open"
        );

        let mut interp = Interp::new();
        let failure = run_source(
            &mut interp,
            b"zz = 1\ncall sub\nexit\nsub: procedure\nyy = 2\nsay 1/0\nreturn\n",
        )
        .unwrap_err();
        assert!(
            matches!(failure, Failure::Raised(_)),
            "the callee must have raised, or this proves nothing about the error path"
        );
        assert_eq!(
            interp.roots.live_frames(),
            1,
            "a raise inside an isolated callee must still release its frame"
        );

        // The shared-pool path, which is the `else` branch of the same
        // `owns_frame` test and is reached by neither block above: a callee
        // with no PROCEDURE pushes no frame of its own (D9r), so it must not
        // pop one either.
        //
        // **What this pins, corrected after review.** It used to claim that
        // without it "an implementation that never pushed a callee frame
        // would pass both assertions above" -- false, and shown false by
        // building exactly that mutant, under which all three blocks pass,
        // because all three compare against the same number. No frame count
        // can catch a frame that is never pushed; the isolation tests are
        // what catch it (`procedure_isolates_and_expose_aliases_the_caller_
        // entry` and four others fail on it).
        //
        // What this block does catch, verified by mutation rather than
        // asserted: popping unconditionally instead of only when
        // `owns_frame`, which tears down the *caller's* still-live frame.
        // That mutant fails here and passes
        // `procedure_isolates_and_expose_aliases_the_caller_entry`, so this
        // block is the one carrying it.
        let mut interp = Interp::new();
        say_output(
            &mut interp,
            b"zz = 1\ncall sub\nexit\nsub:\nyy = 2\nreturn\n",
        );
        assert_eq!(
            interp.roots.live_frames(),
            1,
            "a shared-pool callee must leave the caller's frame open"
        );
    }

    // ---- USE ARG ----

    #[test]
    fn use_arg_binds_positionally_and_ignores_extra_arguments() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"call sub2 1,2,3\nexit\nsub2: procedure\nuse arg p\nsay '['p']'\nreturn\n",
            ),
            b"[1]\n".to_vec()
        );

        // An omitted position holds its place rather than closing the list
        // up: an implementation that skipped it would bind R to 3.
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"call sub2 1,,3\nexit\n\
                  sub2: procedure\nuse arg p, q, r\nsay '['p']' '['q']' '['r']'\nreturn\n",
            ),
            b"[1] [Q] [3]\n".to_vec()
        );
    }

    /// A target with no argument and no default is **dropped**, not left
    /// alone.
    ///
    /// The callee has no `PROCEDURE` on purpose, so the caller's `PRESET` is
    /// the same variable and the drop is observable after the return.
    /// `preset-value` is deliberately not `PRESET`: a target whose prior
    /// value equalled its own derived name would render identically whether
    /// it was dropped or left, so such a probe could not fail.
    #[test]
    fn use_arg_drops_a_target_with_no_argument_and_no_default() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"preset = 'preset-value'\ncall sub2 1\nsay 'after:' preset\nexit\n\
                  sub2:\nuse arg p, preset\nsay 'inside:' preset\nreturn\n",
            ),
            b"inside: PRESET\nafter: PRESET\n".to_vec()
        );
    }

    #[test]
    fn use_arg_defaults_fill_an_absent_or_omitted_position() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"call sub2 1\nexit\n\
                  sub2: procedure\nuse arg p, q = 'dflt'\nsay '['p']['q']'\nreturn\n",
            ),
            b"[1][dflt]\n".to_vec(),
            "absent past the end of the list"
        );

        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"call sub2 1,,3\nexit\n\
                  sub2: procedure\nuse arg p, q = 'dflt', r\nsay '['p']['q']['r']'\nreturn\n",
            ),
            b"[1][dflt][3]\n".to_vec(),
            "omitted in the middle"
        );
    }

    /// `STRICT`'s two arity checks, and the two things that switch them off.
    /// Every number and boundary measured on the oracle.
    #[test]
    fn use_strict_arg_checks_arity_at_both_ends() {
        // Too many: 40.4, naming the routine and the maximum.
        let mut interp = Interp::new();
        let failure = run_source(
            &mut interp,
            b"call sub2 1,2,3\nexit\nsub2: procedure\nuse strict arg p\nreturn\n",
        )
        .unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (40, 4));
        assert_eq!(raised.additional, vec!["SUB2".to_string(), "1".to_string()]);

        // Too few: 40.3, naming the minimum.
        let mut interp = Interp::new();
        let failure = run_source(
            &mut interp,
            b"call sub2 1\nexit\nsub2: procedure\nuse strict arg p, q\nreturn\n",
        )
        .unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (40, 3));
        assert_eq!(raised.additional, vec!["SUB2".to_string(), "2".to_string()]);

        // A trailing `...` suppresses the maximum check only.
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"call sub2 1,2,3,4\nexit\n\
                  sub2: procedure\nuse strict arg p, q, ...\nsay '['p']['q']'\nreturn\n",
            ),
            b"[1][2]\n".to_vec()
        );

        // A default satisfies the minimum, so this must not raise 40.3.
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"call sub2 1\nexit\n\
                  sub2: procedure\nuse strict arg p, q = 'dflt'\nsay '['p']['q']'\nreturn\n",
            ),
            b"[1][dflt]\n".to_vec()
        );
    }

    /// `USE ARG >name` aliases the caller's variable; the same call into a
    /// plain target copies its value instead.
    ///
    /// The pair is what makes this test discriminating: an implementation
    /// that aliased unconditionally, or never, gets exactly one of the two
    /// right.
    #[test]
    fn use_arg_alias_binds_the_callers_variable_and_a_plain_target_does_not() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"p = 'orig'\ncall sub2 >p\nsay 'after:' p\nexit\n\
                  sub2: procedure\nuse arg >q\nsay 'callee:' q\nq = 'aliased'\nreturn\n",
            ),
            b"callee: orig\nafter: aliased\n".to_vec()
        );

        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"p = 'orig'\ncall sub2 >p\nsay 'after:' p\nexit\n\
                  sub2: procedure\nuse arg q\nq = 'aliased'\nreturn\n",
            ),
            b"after: orig\n".to_vec(),
            "a plain target copies the value, so the caller's P is untouched"
        );

        // A stem aliases the same way, measured.
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"st.1 = 'orig'\ncall sub2 >st.\nsay 'after:' st.1\nexit\n\
                  sub2: procedure\nuse arg >q.\nq.1 = 'aliased'\nreturn\n",
            ),
            b"after: aliased\n".to_vec()
        );

        // An aliased but unset variable reads as the *callee's* own derived
        // name, not the caller's -- measured, and it falls out of the alias
        // pointing at an unset slot.
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"call sub2 >unsetvar\nexit\n\
                  sub2: procedure\nuse arg >q\nsay 'callee:' q\nreturn\n",
            ),
            b"callee: Q\n".to_vec()
        );
    }

    /// `USE ARG >` has two distinct refusals, and they are different
    /// sub-numbers rather than one shared complaint. Both measured.
    #[test]
    fn use_arg_alias_refuses_a_plain_value_and_an_omitted_position() {
        // A supplied argument that is not a reference: 88.928, carrying the
        // 1-based position and the argument's own *value*.
        let mut interp = Interp::new();
        let failure = run_source(
            &mut interp,
            b"zebra = 'orig'\ncall sub2 zebra\nexit\nsub2: procedure\nuse arg >q\nreturn\n",
        )
        .unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (88, 928));
        assert_eq!(
            raised.additional,
            vec!["1".to_string(), "orig".to_string()],
            "the substitution is the argument's value, not the variable's spelling -- \
             a probe naming the variable `caller` could not tell those apart"
        );

        // An omitted position: 88.931, a different complaint.
        let mut interp = Interp::new();
        let failure = run_source(
            &mut interp,
            b"call sub2 1\nexit\nsub2: procedure\nuse arg p, >q\nreturn\n",
        )
        .unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (88, 931));
        assert_eq!(raised.additional, vec!["2".to_string()]);
    }

    /// `USE ARG >name` requires its target to be **currently unset**, and
    /// raises 98.995 otherwise.
    ///
    /// **These three are a set and the middle one carries the weight.** The
    /// message says "it must be an uninitialized *local* variable", which
    /// invites writing the check as an exposure or locality test. The pair
    /// that rules that out is the second and third cases below: the same
    /// `procedure expose q`, raising when the exposed `q` holds a value and
    /// succeeding when it does not. Exposure is identical in both; only the
    /// value differs. The raising case alone does not pin which rule is being
    /// applied, and the succeeding case alone cannot fail against a wrong
    /// fix -- neither is worth much without the other.
    #[test]
    fn use_arg_alias_requires_an_uninitialised_target() {
        // Assigned, then aliased: refused, naming the target.
        let mut interp = Interp::new();
        let failure = run_source(
            &mut interp,
            b"p = 'p-orig'\ncall sub >p\nexit\n\
              sub: procedure\nq = 1\nuse arg >q\nreturn\n",
        )
        .unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (98, 995));
        assert_eq!(raised.additional, vec!["Q".to_string()]);

        // Exposed AND holding a value: still refused. Exposure is not the
        // trigger, but this case on its own cannot show that.
        let mut interp = Interp::new();
        let failure = run_source(
            &mut interp,
            b"p = 'p-orig'\nq = 'q-in-caller'\ncall sub >p\nexit\n\
              sub: procedure expose q\nuse arg >q\nreturn\n",
        )
        .unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (98, 995));

        // Exposed and UNSET: succeeds. This is what makes the pair
        // discriminating -- an exposure check would wrongly refuse here.
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"p = 'p-orig'\ncall sub >p\nsay 'p:' p 'q:' q\nexit\n\
                  sub: procedure expose q\nuse arg >q\nq = 'via-alias'\nreturn\n",
            ),
            b"p: via-alias q: Q\n".to_vec(),
            "the alias must have been installed, and the caller's own exposed Q must \
             still read unset"
        );

        // DROP restores the uninitialised state.
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"p = 'p-orig'\nq = 'local'\ndrop q\ncall sub >p\nsay 'p:' p\nexit\n\
                  sub:\nuse arg >q\nq = 'via-alias'\nreturn\n",
            ),
            b"p: via-alias\n".to_vec()
        );

        // Repeating `use arg >q` onto one target is the same rule, not a case
        // of its own: the first alias makes Q read the caller's variable, so
        // it has a value by the second. `RootSet::slot` resolving through the
        // alias is what makes this fall out.
        let mut interp = Interp::new();
        let failure = run_source(
            &mut interp,
            b"p = 'p-orig'\nr = 'r-orig'\ncall sub >p, >r\nexit\n\
              sub:\nuse arg >q\nuse arg xx, >q\nreturn\n",
        )
        .unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (98, 995));
    }

    /// `USE ARG >name` requires the reference's **kind** to match the
    /// target's: a simple reference into a stem target is 88.929, and a stem
    /// reference into a simple target is 88.930.
    ///
    /// **Each refusal is paired with its adjacent success, in this test
    /// rather than elsewhere, and the pairing is the point.** A test that
    /// only checks `>p` into `>q.` raises cannot distinguish "the kinds must
    /// match" from "a stem target is always refused"; the passing `>p.` into
    /// `>q.` case is what rules the second out. The same holds mirrored for
    /// 88.930. All four cells measured against the oracle.
    #[test]
    fn use_arg_alias_requires_the_reference_kind_to_match_the_target() {
        // Simple reference -> STEM target: refused.
        //
        // `p` holds `value-not-name` so that the substitution discriminates:
        // 88.929 names the *caller's variable*, `P`, where 88.928 in the same
        // position names the argument's *value*. A probe whose variable held
        // its own name could not tell those apart -- the mistake this task
        // already made once, on 88.928.
        let mut interp = Interp::new();
        let failure = run_source(
            &mut interp,
            b"p = 'value-not-name'\ncall sub >p\nexit\nsub: procedure\nuse arg >q.\nreturn\n",
        )
        .unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (88, 929));
        assert_eq!(raised.additional, vec!["1".to_string(), "P".to_string()]);

        // ...and the adjacent success: a STEM reference into the same stem
        // target. Without this, "stem targets are always refused" passes.
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"p.1 = 'orig'\ncall sub >p.\nsay 'after:' p.1\nexit\n\
                  sub: procedure\nuse arg >q.\nq.1 = 'via-alias'\nreturn\n",
            ),
            b"after: via-alias\n".to_vec()
        );

        // Stem reference -> SIMPLE target: refused, naming `P.` with its
        // period.
        let mut interp = Interp::new();
        let failure = run_source(
            &mut interp,
            b"p.1 = 'value-not-name'\ncall sub >p.\nexit\n\
              sub: procedure\nuse arg >q\nreturn\n",
        )
        .unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (88, 930));
        assert_eq!(raised.additional, vec!["1".to_string(), "P.".to_string()]);

        // ...and its adjacent success: a simple reference into a simple
        // target.
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"p = 'orig'\ncall sub >p\nsay 'after:' p\nexit\n\
                  sub: procedure\nuse arg >q\nq = 'via-alias'\nreturn\n",
            ),
            b"after: via-alias\n".to_vec()
        );

        // An argument that is not a reference at all is still 88.928, even
        // against a stem target, and still substitutes the VALUE. So the kind
        // check sits behind the is-a-reference check rather than replacing
        // it.
        let mut interp = Interp::new();
        let failure = run_source(
            &mut interp,
            b"p = 'value-not-name'\ncall sub p\nexit\nsub: procedure\nuse arg >q.\nreturn\n",
        )
        .unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (88, 928));
        assert_eq!(
            raised.additional,
            vec!["1".to_string(), "value-not-name".to_string()]
        );

        // The position substitution is the argument's own, not always 1.
        let mut interp = Interp::new();
        let failure = run_source(
            &mut interp,
            b"p = 'val'\ncall sub 1, >p\nexit\nsub: procedure\nuse arg aa, >q.\nreturn\n",
        )
        .unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!(raised.additional, vec!["2".to_string(), "P".to_string()]);

        // STRICT does not change the kind rule.
        let mut interp = Interp::new();
        let failure = run_source(
            &mut interp,
            b"p = 'val'\ncall sub >p\nexit\nsub: procedure\nuse strict arg >q.\nreturn\n",
        )
        .unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (88, 929));
    }

    /// The kind check runs **before** the uninitialised check, so a target
    /// that fails both reports the kind error.
    ///
    /// Measured both ways round. Ordering is not cosmetic here: each of the
    /// two errors is a different number and rc, so getting it backwards is a
    /// wrong answer rather than a differently-worded right one. A single
    /// test would not pin it -- both directions are needed, because a check
    /// that always reported the kind error would pass one of them alone.
    #[test]
    fn use_arg_alias_reports_the_kind_mismatch_before_the_uninitialised_target() {
        // Stem target, already assigned, given a simple reference: 88.929,
        // not 98.995.
        let mut interp = Interp::new();
        let failure = run_source(
            &mut interp,
            b"p = 'val'\ncall sub >p\nexit\n\
              sub: procedure\nq.1 = 'already-set'\nuse arg >q.\nreturn\n",
        )
        .unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (88, 929));

        // Simple target, already assigned, given a stem reference: 88.930,
        // not 98.995.
        let mut interp = Interp::new();
        let failure = run_source(
            &mut interp,
            b"p.1 = 'val'\ncall sub >p.\nexit\n\
              sub: procedure\nq = 'already-set'\nuse arg >q\nreturn\n",
        )
        .unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (88, 930));
    }

    /// The stem half of the same rule, which is where this crate's own
    /// representation shows through and where the obvious one-line check gets
    /// it wrong.
    ///
    /// `read_stem` vivifies a fresh empty `Body::Stem` into the slot on a bare
    /// stem read, and `stem_drop` leaves exactly the same thing, so a slot
    /// being `Some` is *not* the same question as the variable being
    /// initialised. All five measured against the oracle; the first three
    /// must succeed and would all raise under a plain `slot(..).is_some()`
    /// test.
    #[test]
    fn use_arg_alias_treats_an_empty_stem_as_uninitialised() {
        for (source, expected, why) in [
            (
                &b"st.1 = 'orig'\ncall sub >st.\nsay 'after:' st.1\nexit\n\
                   sub: procedure\nuse arg >q.\nq.1 = 'via-alias'\nreturn\n"[..],
                &b"after: via-alias\n"[..],
                "a never-touched stem target",
            ),
            (
                b"st.1 = 'orig'\ncall sub >st.\nsay 'after:' st.1\nexit\n\
                  sub: procedure\nsay 'bare read:' q.\nuse arg >q.\nq.1 = 'via-alias'\nreturn\n",
                b"bare read: Q.\nafter: via-alias\n",
                "a bare stem READ vivifies an empty stem into the slot, and must not \
                 count as initialising it",
            ),
            (
                b"st.1 = 'orig'\ncall sub >st.\nsay 'after:' st.1\nexit\n\
                  sub: procedure\nq.1 = 'x'\ndrop q.\nuse arg >q.\nq.1 = 'via-alias'\nreturn\n",
                b"after: via-alias\n",
                "DROP of a written stem restores the uninitialised state",
            ),
            (
                b"st.1 = 'orig'\ncall sub >st.\nsay 'after:' st.1\nexit\n\
                  sub: procedure\nq.1 = 'x'\ndrop q.1\nuse arg >q.\nq.1 = 'via-alias'\nreturn\n",
                b"after: via-alias\n",
                "a TOMBSTONED tail is not content -- `tails.is_empty()` would refuse this, \
                 and did until it was measured",
            ),
            (
                b"st.1 = 'orig'\ncall sub >st.\nsay 'after:' st.1\nexit\n\
                  sub: procedure\nq.1 = 'x'\nq.2 = 'y'\ndrop q.1\ndrop q.2\n\
                  use arg >q.\nq.1 = 'via-alias'\nreturn\n",
                b"after: via-alias\n",
                "every tail tombstoned is still no content",
            ),
        ] {
            let mut interp = Interp::new();
            assert_eq!(say_output(&mut interp, source), expected.to_vec(), "{why}");
        }

        // A written tail initialises the stem: refused, naming `Q.` with its
        // period, which is the target's own spelling.
        let mut interp = Interp::new();
        let failure = run_source(
            &mut interp,
            b"st.1 = 'orig'\ncall sub >st.\nexit\n\
              sub: procedure\nq.1 = 'local'\nuse arg >q.\nreturn\n",
        )
        .unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (98, 995));
        assert_eq!(raised.additional, vec!["Q.".to_string()]);

        // An assigned default initialises it too, with no tails written.
        let mut interp = Interp::new();
        let failure = run_source(
            &mut interp,
            b"st.1 = 'orig'\ncall sub >st.\nexit\n\
              sub: procedure\nq. = 'dflt'\nuse arg >q.\nreturn\n",
        )
        .unwrap_err();
        assert!(matches!(failure, Failure::Raised(_)));

        // **The two that make the rule "no LIVE tail" rather than "no
        // tails".** One tombstone beside one surviving tail still has
        // content; a default survives a tombstoned tail. Without this pair
        // the three succeeding tombstone rows above are equally consistent
        // with "ignore tails entirely", which would wrongly accept both.
        for source in [
            &b"st.1 = 'orig'\ncall sub >st.\nexit\n\
               sub: procedure\nq.1 = 'x'\nq.2 = 'y'\ndrop q.1\nuse arg >q.\nreturn\n"[..],
            b"st.1 = 'orig'\ncall sub >st.\nexit\n\
              sub: procedure\nq. = 'dflt'\ndrop q.1\nuse arg >q.\nreturn\n",
        ] {
            let mut interp = Interp::new();
            let failure = run_source(&mut interp, source).unwrap_err();
            let Failure::Raised(raised) = failure else {
                panic!("expected Raised, got {failure:?}");
            };
            assert_eq!((raised.number, raised.sub), (98, 995));
        }

        // **The exemption is keyed on the target's NAME shape, not on the
        // value's.** A simple variable holding a fresh, empty stem object is
        // an initialised simple variable -- measured, `zz = q.` then `use arg
        // >zz` raises. A check written against "the value is an empty stem"
        // passes everything above and fails here.
        let mut interp = Interp::new();
        let failure = run_source(
            &mut interp,
            b"p = 'p-orig'\ncall sub >p\nexit\n\
              sub: procedure\nzz = q.\nuse arg >zz\nreturn\n",
        )
        .unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (98, 995));
        assert_eq!(raised.additional, vec!["ZZ".to_string()]);
    }

    /// A variable reference in an ordinary value position is worth the
    /// referenced variable's value. Measured: `say >p` prints `p`'s value.
    #[test]
    fn a_variable_reference_decays_to_the_referenced_value() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(&mut interp, b"p = 'orig'\nsay >p"),
            b"orig\n".to_vec()
        );
    }

    /// The caller's own arguments survive a nested call.
    ///
    /// **This is Task 4's Critical, in this task's own currency.** That
    /// finding was a piece of per-activation state a call failed to restore,
    /// invisible until two activations per clause were reachable.
    /// `Interp::call_context` is the fifth such piece; without the restore in
    /// `resolve_and_run_call`, the second `USE ARG` below reads the *inner*
    /// call's arguments and prints `inner-arg`.
    #[test]
    fn a_callers_arguments_survive_a_nested_call() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"call outer 'outer-arg'\nexit\n\
                  outer: procedure\nuse arg first\ncall inner 'inner-arg'\n\
                  use arg second\nsay '['first']['second']'\nreturn\n\
                  inner: procedure\nuse arg deep\nreturn\n",
            ),
            b"[outer-arg][outer-arg]\n".to_vec()
        );
    }

    /// `USE LOCAL` as a program's own first instruction is 98.993 -- the one
    /// shape that reaches `exec_use` at all, since `rexx-parse` rejects the
    /// rest at parse time. `exec_use`'s own doc comment lists the eight
    /// shapes that were tried and why the 99.910 arm carries no test.
    ///
    /// **Driven through `run_program` rather than `run_source`**, and that is
    /// not incidental: this module's helper runs a body through
    /// `run_bounded`, which never grants the first-instruction permission,
    /// so a `run_source` version of this test would take the *other* arm and
    /// assert 99.910 -- passing against an implementation that had the two
    /// swapped. Only the real entry point puts the program in the state the
    /// oracle's own 98.993 describes. Verified against the oracle in a clean
    /// directory: rc 158 and these two lines.
    #[test]
    fn use_local_as_a_programs_first_instruction_raises_98_993() {
        let outcome = crate::run_program("/tmp/use-local.rex", b"use local outer\n".to_vec());
        assert_eq!(outcome.exit_code, 158, "256 - 98");
        let stderr = String::from_utf8_lossy(&outcome.stderr);
        assert!(
            stderr.contains("Error 98.993:")
                && stderr.contains(
                    "The USE LOCAL instruction may only be used from method invocations."
                ),
            "expected the oracle's own 98.993 report, got: {stderr}"
        );
    }

    /// `PROCEDURE EXPOSE` of a single compound tail fails loudly rather than
    /// approximating.
    ///
    /// Measured on the oracle: exposing `a.1` shares that one tail and
    /// leaves `a.2` the callee's own, which is aliasing inside a stem object
    /// and not something a whole-slot alias can express. Exposing the stem
    /// instead would silently share `a.2` too.
    #[test]
    fn procedure_expose_of_a_single_compound_tail_fails_loudly() {
        let mut interp = Interp::new();
        let failure = run_source(
            &mut interp,
            b"a.1 = 'kept'\ncall sub\nexit\nsub: procedure expose a.1\nreturn\n",
        )
        .unwrap_err();
        let Failure::Loud(loud) = failure else {
            panic!("expected Loud, got {failure:?}");
        };
        assert!(
            loud.message.contains("A.1"),
            "the message must name the tail it refused: {}",
            loud.message
        );
    }

    // ---- 4b Task 7: condition traps, RAISE and NOVALUE ----
    //
    // Every trap test below asserts a value the *handler set*, never that
    // the program exited 0: a criterion of the second kind is satisfied by a
    // program that never raised at all. The values are chosen so that a
    // handler which did not run prints an unset variable's derived name --
    // `ZWITNESS`, not something that reads like data.

    /// The base case, and the one every other test here is a variation of.
    ///
    /// `sigl` is asserted alongside the handler's own value because the two
    /// fail independently: a trap that fires with the wrong `SIGL` looks
    /// exactly like a correct one to any test that only checks the handler
    /// ran. Measured, `SIGL` is the **raising** clause's line (3), not the
    /// `SIGNAL ON` clause's (1) and not the handler's (5).
    #[test]
    fn a_signal_on_syntax_trap_runs_its_handler_and_sets_sigl_to_the_raising_clause() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"signal on syntax\nzwitness = 'BEFORE'\nsay 1/0\nexit\nsyntax:\nzwitness = 'TRAPPED'\nsay zwitness sigl\n",
            ),
            b"TRAPPED 3\n".to_vec()
        );
    }

    /// The adjacent success for the test above: the identical program with
    /// the `SIGNAL ON` clause removed is the ordinary fatal 42.3, so the
    /// handler's output in that test is caused by the trap and not by
    /// anything else in the program.
    #[test]
    fn the_same_program_without_the_trap_is_the_ordinary_fatal_condition() {
        let mut interp = Interp::new();
        let failure = run_source(
            &mut interp,
            b"zwitness = 'BEFORE'\nsay 1/0\nexit\nsyntax:\nzwitness = 'TRAPPED'\nsay zwitness sigl\n",
        )
        .unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (42, 3));
        assert!(
            interp.out.is_empty(),
            "the handler must not have run: {:?}",
            String::from_utf8_lossy(&interp.out)
        );
    }

    /// `SIGNAL OFF` really removes the trap, and the pair is what pins it to
    /// `SIGNAL OFF` rather than to anything about where the raise sits.
    #[test]
    fn signal_off_removes_a_trap_that_signal_on_had_enabled() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"signal on syntax\nsignal on novalue\nsignal off syntax\nsay zunset\nexit\nsyntax:\nsay 'SYNTAX-HANDLER'\nexit\nnovalue:\nsay 'NOVALUE-HANDLER'\n",
            ),
            b"NOVALUE-HANDLER\n".to_vec(),
            "NOVALUE is still enabled and SYNTAX is not, so the read traps \
             and nothing reaches the SYNTAX handler"
        );

        let mut interp = Interp::new();
        let failure = run_source(
            &mut interp,
            b"signal on syntax\nsignal off syntax\nsay 1/0\nexit\nsyntax:\nsay 'SYNTAX-HANDLER'\n",
        )
        .unwrap_err();
        assert!(
            matches!(&failure, Failure::Raised(raised) if raised.number == 42),
            "after SIGNAL OFF the same raise is fatal, got {failure:?}"
        );
    }

    /// The trap that fired is gone from the table by the time the handler
    /// runs.
    ///
    /// **A direct assertion on the table rather than on behaviour, and the
    /// mutation record is why.** The behavioural pair below -- re-arm under
    /// a new label, and a second raise with no re-arm -- was written first,
    /// and deleting the removal left the first of the two *green*: its
    /// handler re-arms before raising again, and `insert` over a live entry
    /// looks exactly like `insert` over an absent one. The second test does
    /// go red, but by looping until the harness kills it, which is a poor
    /// signal to leave as the only one. This one fails in microseconds and
    /// names the property.
    #[test]
    fn the_trap_that_fired_is_removed_from_the_table() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"signal on syntax\nsignal on novalue\nsay 1/0\nexit\nsyntax:\nsay 'TRAPPED'\n",
            ),
            b"TRAPPED\n".to_vec()
        );
        let traps = &interp.activation().traps;
        assert!(
            !traps.contains_key(b"SYNTAX".as_slice()),
            "the SYNTAX trap fired and must be gone"
        );
        assert!(
            traps.contains_key(b"NOVALUE".as_slice()),
            "the NOVALUE trap did not fire and must be untouched -- without \
             this half the assertion above is satisfied by clearing the whole \
             table, or by never filling it"
        );
    }

    /// `SIGNAL ON` inside the handler re-arms the condition, under a new
    /// label: the first raise reaches `first` and the second reaches
    /// `second`.
    ///
    /// This one is about the re-arm and **not** about the removal -- see
    /// `the_trap_that_fired_is_removed_from_the_table` above, which is the
    /// test that actually pins that, and the note there for how this one was
    /// measured not to.
    #[test]
    fn a_trap_can_be_re_armed_inside_its_own_handler() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"signal on syntax name first\nsay 1/0\nexit\nfirst:\nsay 'FIRST' sigl\nsignal on syntax name second\nsay 2/0\nexit\nsecond:\nsay 'SECOND' sigl\n",
            ),
            b"FIRST 2\nSECOND 7\n".to_vec()
        );
    }

    /// Without the re-arm the second raise is fatal -- the adjacent case
    /// that pins the test above to "the trap was disabled" rather than to
    /// "the second raise happened to reach a different label".
    ///
    /// Against an implementation that leaves the fired trap armed this
    /// program does not fail, it *loops*: the handler raises again, is
    /// trapped again, and prints `FIRST` forever.
    /// `the_trap_that_fired_is_removed_from_the_table` is the bounded
    /// version of the same property.
    #[test]
    fn a_second_raise_inside_a_handler_is_fatal_without_a_re_arm() {
        let mut interp = Interp::new();
        let failure = run_source(
            &mut interp,
            b"signal on syntax name first\nsay 1/0\nexit\nfirst:\nsay 'FIRST'\nsay 2/0\n",
        )
        .unwrap_err();
        assert!(matches!(&failure, Failure::Raised(raised) if raised.number == 42));
        assert_eq!(interp.out, b"FIRST\n".to_vec());
    }

    /// **Inherited item I11.** `Interp::failure_site` is first-wins, so a
    /// second raise after a trapped first one reported the *first* site
    /// until `offer_to_trap` began clearing it. The report must name line 6,
    /// the second raise's own clause, and a version without the clearing
    /// names line 2.
    #[test]
    fn a_second_raise_after_a_trapped_one_reports_its_own_site() {
        let mut interp = Interp::new();
        run_source(
            &mut interp,
            b"signal on syntax\nsay 1/0\nexit\nsyntax:\nsay 'HANDLER'\nsay 2/0\n",
        )
        .unwrap_err();
        let mut sites = std::mem::take(&mut interp.failure_sites);
        sites.extend(interp.failure_site.take());
        let lines: Vec<usize> = sites.iter().map(|site| site.line).collect();
        assert_eq!(
            lines,
            vec![6],
            "exactly one echo, naming the second raise's own clause -- the \
             first raise's site (line 2) must not have survived being trapped"
        );
        assert_eq!(sites[0].text, b"say 2/0".to_vec());
    }

    /// `SIGNAL ON NOVALUE` fires for a simple variable and for a compound,
    /// and **not** for a bare stem. The third row is the one that cannot be
    /// guessed: measured, `say zstem.` under the same trap prints the
    /// derived name and the program carries on, where `say zstem.1` traps.
    #[test]
    fn novalue_fires_for_a_simple_variable_and_a_compound_but_not_a_bare_stem() {
        for (source, expected) in [
            (
                &b"signal on novalue\nsay zprobe\nexit\nnovalue:\nsay 'TRAPPED' sigl\n"[..],
                &b"TRAPPED 2\n"[..],
            ),
            (
                &b"signal on novalue\nsay zprobe.1\nexit\nnovalue:\nsay 'TRAPPED' sigl\n"[..],
                &b"TRAPPED 2\n"[..],
            ),
            (
                &b"signal on novalue\nsay zprobe.\nsay 'RESUMED'\nexit\nnovalue:\nsay 'TRAPPED'\n"
                    [..],
                &b"ZPROBE.\nRESUMED\n"[..],
            ),
        ] {
            let mut interp = Interp::new();
            assert_eq!(
                say_output(&mut interp, source),
                expected.to_vec(),
                "{}",
                String::from_utf8_lossy(source)
            );
        }
    }

    /// An untrapped `NOVALUE` costs nothing and changes nothing -- the read
    /// still yields the derived name. The neighbouring case that pins
    /// `novalue_check`'s gate to "there is a trap" rather than to anything
    /// about the variable.
    #[test]
    fn an_untrapped_novalue_still_reads_the_derived_name() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(&mut interp, b"say zprobe\nsay zprobe.1\n"),
            b"ZPROBE\nZPROBE.1\n".to_vec()
        );
    }

    /// A trap label that does not exist is `16.1`, reported against the
    /// **raising** clause rather than against the `SIGNAL ON` clause or the
    /// missing label -- so the site the raise had already recorded survives,
    /// which is why `offer_to_trap` resolves the label before it clears
    /// anything.
    #[test]
    fn a_trap_label_that_does_not_exist_is_16_1_at_the_raising_clause() {
        let mut interp = Interp::new();
        let failure = run_source(
            &mut interp,
            b"signal on syntax name nosuchlabel\nsay 'a'\nsay 1/0\n",
        )
        .unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (16, 1));
        assert_eq!(raised.additional, vec!["NOSUCHLABEL".to_string()]);
        let site = interp.failure_site.expect("a site was resolved");
        assert_eq!((site.line, site.text), (3, b"say 1/0".to_vec()));
    }

    /// A trap is inherited by a callee and fires **there**, in the callee's
    /// own activation -- which is observable only because `PROCEDURE`
    /// isolates the pool: the handler reads the callee's `ZOWNER`, not the
    /// caller's. `SIGL` is the callee's raising line for the same reason.
    #[test]
    fn a_trap_is_inherited_by_a_callee_and_fires_in_the_callees_own_activation() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"signal on syntax\nzowner = 'CALLER-POOL'\ncall sub\nexit\nsub: procedure\nzowner = 'CALLEE-POOL'\nsay 1/0\nreturn\nsyntax:\nsay zowner sigl\n",
            ),
            b"CALLEE-POOL 7\n".to_vec()
        );
    }

    /// The other half: turning the trap off *in the callee* leaves the
    /// caller's own enabled, so the condition propagates outward and is
    /// trapped there instead -- with `SIGL` now the caller's `call` clause.
    /// Together the two tests say the table is inherited **by copy**.
    #[test]
    fn a_callees_signal_off_leaves_the_callers_trap_enabled() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"signal on syntax\nzowner = 'CALLER-POOL'\ncall sub\nexit\nsub: procedure\nsignal off syntax\nsay 1/0\nreturn\nsyntax:\nsay zowner sigl\n",
            ),
            b"CALLER-POOL 3\n".to_vec()
        );
    }

    /// **The mid-clause resumption shape, and the one route that could still
    /// have created two activations within one clause.** `zz = one(1)
    /// two(2)`: `one` raises, the inherited trap transfers to a handler that
    /// `RETURN`s, so `one(1)` yields the handler's value and evaluation
    /// resumes *inside the enclosing clause*, which then calls `two`.
    ///
    /// `two` reports `SIGL`, which is the quantity Tasks 4 and 6 each shipped
    /// a defect on: it must be the enclosing clause's line (2), not `one`'s
    /// raise line (6), not the handler's (11). It is right because
    /// `clause_state` lives in `ClauseState` and `resolve_and_run_call`
    /// restores it whole -- the mechanism the brief asked this route to
    /// test, verified rather than assumed.
    #[test]
    fn a_trap_that_resumes_mid_clause_leaves_the_enclosing_clauses_state_intact() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"signal on syntax\nzz = one(1) two(2)\nsay 'zz=' zz\nexit\none:\nsay 1/0\nreturn 'ONEVAL'\ntwo:\nreturn 'SIGLIS' sigl\nsyntax:\nreturn 'FROMHANDLER'\n",
            ),
            b"zz= FROMHANDLER SIGLIS 2\n".to_vec()
        );
    }

    /// **A `CALL ON` handler runs at the clause boundary, not at the raise.**
    /// `one` raises a trapped `USER` condition and returns its `RAISE ...
    /// RETURN` value; the enclosing clause finishes -- `two(2)` and the
    /// assignment included -- and only then does the handler run and
    /// overwrite `zz`.
    ///
    /// The `SIGL` in the handler's own output is what caught the first
    /// implementation, which delivered at the next clause boundary reached by
    /// *any* activation and so ran the handler inside `two`, reporting `two`'s
    /// label line instead of the enclosing clause's.
    #[test]
    fn a_call_trap_waits_for_the_raising_clause_to_finish() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"call on user marker name uh\nzz = one(1) two(2)\nsay 'zz=' zz\nexit\none:\nraise user marker return 'ONEVAL'\ntwo:\nreturn 'TWOVAL'\nuh:\nzz = 'HANDLER-RAN-AT' sigl\nreturn\n",
            ),
            b"zz= HANDLER-RAN-AT 2\n".to_vec(),
            "the assignment stored ONEVAL TWOVAL first, then the handler \
             overwrote it -- a handler that fired at the raise would leave \
             `zz` as the routine's own value"
        );
    }

    /// `RAISE`'s delivery table, the part that a two-level program cannot
    /// tell apart. Each row is a three-level chain with the trap enabled in
    /// exactly one place; the expected value names which handler ran.
    #[test]
    fn raise_delivery_depends_on_the_tail_and_on_the_condition() {
        // `RAISE SYNTAX ... RETURN` searches from the raising activation, so
        // the trap `lev2` inherited from `lev1` is the one that fires.
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"call lev1\nexit\nlev1:\nsignal on syntax name mid\ncall lev2\nreturn\nlev2:\nraise syntax 40.4 return\nmid:\nsay 'MID' sigl\nexit\n",
            ),
            b"MID 8\n".to_vec()
        );

        // The same raise with no tail skips every level but the outermost,
        // so `mid` never runs and the condition is fatal.
        let mut interp = Interp::new();
        let failure = run_source(
            &mut interp,
            b"call lev1\nexit\nlev1:\nsignal on syntax name mid\ncall lev2\nreturn\nlev2:\nraise syntax 40.4\nmid:\nsay 'MID' sigl\nexit\n",
        )
        .unwrap_err();
        assert!(matches!(&failure, Failure::Raised(raised) if raised.number == 40));
        assert!(interp.out.is_empty(), "`mid` must not have run");

        // ... and enabling it at the top as well is what catches it, with
        // `SIGL` the main body's own clause.
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"signal on syntax name outer\ncall lev1\nexit\nlev1:\nsignal on syntax name mid\ncall lev2\nreturn\nlev2:\nraise syntax 40.4\nmid:\nsay 'MID' sigl\nexit\nouter:\nsay 'OUTER' sigl\nexit\n",
            ),
            b"OUTER 2\n".to_vec()
        );

        // A non-`SYNTAX` condition with a `RETURN` tail searches from the
        // *caller*, so the raising routine's own inherited trap is skipped:
        // `SIGL` is the caller's clause, not the `raise` line.
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"signal on user marker\ncall sub\nexit\nsub:\nraise user marker return\nmarker:\nsay 'MARKER' sigl\nexit\n",
            ),
            b"MARKER 2\n".to_vec()
        );
    }

    /// An untrapped condition's default action: `HALT` reports, everything
    /// else is silent. The pair is what makes `Raised::reportable`'s split
    /// mean something rather than being a spelling.
    #[test]
    fn an_untrapped_raise_reports_for_halt_and_is_silent_for_the_rest() {
        let mut interp = Interp::new();
        let failure = run_source(&mut interp, b"say 'a'\nraise halt\nsay 'b'\n").unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (4, 1));

        for source in [
            &b"say 'a'\nraise user marker\nsay 'b'\n"[..],
            &b"say 'a'\nraise error 5\nsay 'b'\n"[..],
        ] {
            let mut interp = Interp::new();
            assert_eq!(
                say_output(&mut interp, source),
                b"a\n".to_vec(),
                "{}: silent, and `b` is not reached because the tail-less \
                 RAISE ends the program",
                String::from_utf8_lossy(source)
            );
        }
    }

    /// `RC` is the major for a trapped `SYNTAX` however it arose, the raise's
    /// own argument for `ERROR`, and untouched for `NOVALUE` -- three rows
    /// that no two of which share a rule.
    #[test]
    fn rc_is_set_from_the_condition_when_a_trap_fires() {
        for (source, expected) in [
            (
                &b"signal on syntax\nsay 1/0\nexit\nsyntax:\nsay rc\n"[..],
                &b"42\n"[..],
            ),
            (
                &b"signal on syntax\nraise syntax 40.4\nexit\nsyntax:\nsay rc\n"[..],
                &b"40\n"[..],
            ),
            (
                &b"signal on error\ncall sub\nexit\nsub:\nraise error 5 return\nerror:\nsay rc\n"[..],
                &b"5\n"[..],
            ),
            (
                &b"signal on novalue\nsay zprobe\nexit\nnovalue:\nsay rc\n"[..],
                &b"RC\n"[..],
            ),
        ] {
            let mut interp = Interp::new();
            assert_eq!(
                say_output(&mut interp, source),
                expected.to_vec(),
                "{}",
                String::from_utf8_lossy(source)
            );
        }
    }

    /// **Inherited item I16, re-verified against a real trap rather than
    /// argued.** 4a concluded that `SIGNAL ON SYNTAX` cannot accumulate a
    /// temps leak, resting entirely on `step_in_temps_frame` being the single
    /// chokepoint that heals the six `?`-skipped `pop_frame` sites in
    /// `eval.rs`. This is the first task where a trap actually acts, so the
    /// conclusion is measured here: two hundred trap-and-resume cycles and
    /// four hundred must leave the same number of live temps, and a leak of
    /// even one root per cycle would make the second number two hundred
    /// larger.
    ///
    /// The raise fires from inside a parenthesised expression on purpose --
    /// that is what puts `eval`'s own frame-opening sites on the path, which
    /// is what the chokepoint claim is about; a raise from a bare clause
    /// would exercise nothing.
    #[test]
    fn a_trap_that_resumes_does_not_accumulate_temps_frames() {
        fn live_temps_after(cycles: usize) -> usize {
            let source = format!(
                "signal on novalue name h\nzcount = 0\ntop:\nzcount = zcount + 1\nsay ((zprobe))\nh:\nsignal on novalue name h\nif zcount < {cycles} then signal top\nsay 'done' zcount\n"
            );
            let mut interp = Interp::new();
            let out = say_output(&mut interp, source.as_bytes());
            assert_eq!(
                out,
                format!("done {cycles}\n").into_bytes(),
                "the loop must actually have trapped {cycles} times"
            );
            interp.roots.temps_len()
        }
        assert_eq!(
            live_temps_after(200),
            live_temps_after(400),
            "a temps root retained per trapped-and-resumed cycle would make \
             the four-hundred-cycle run exactly two hundred larger"
        );
    }

    /// `RAISE PROPAGATE` re-raises the condition whose handler is running,
    /// past every enclosing trap, and the report names the **original**
    /// raising clause rather than the `raise propagate` clause -- which is
    /// why the echo stack travels on `ActiveCondition` rather than being
    /// dropped when the trap cleared it.
    #[test]
    fn raise_propagate_re_raises_the_original_condition_and_its_site() {
        let mut interp = Interp::new();
        let failure = run_source(
            &mut interp,
            b"signal on syntax name outer\ncall sub\nexit\nsub:\nsignal on syntax name inner\nsay 1/0\nreturn\ninner:\nraise propagate\nouter:\nsay 'OUTER-MUST-NOT-RUN'\n",
        )
        .unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (42, 3));
        assert!(
            raised.delivery.positionless,
            "the major line drops its ` running <path> line <n>` span"
        );
        assert!(interp.out.is_empty(), "`outer` must not have run");
        let mut sites = std::mem::take(&mut interp.failure_sites);
        sites.extend(interp.failure_site.take());
        let lines: Vec<usize> = sites.iter().map(|site| site.line).collect();
        assert_eq!(
            lines,
            vec![6, 2],
            "the original `say 1/0` clause and the `call sub` above it, not \
             the `raise propagate` clause on line 9"
        );
    }

    /// With no handler running, `RAISE PROPAGATE` is `98.918` -- the
    /// adjacent case that pins the test above to "there was an active
    /// condition" rather than to anything about `PROPAGATE` itself.
    #[test]
    fn raise_propagate_with_no_active_condition_is_98_918() {
        let mut interp = Interp::new();
        let failure = run_source(&mut interp, b"say 'a'\nraise propagate\n").unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (98, 918));
        assert!(!raised.delivery.positionless);
    }

    /// A `CALL ON` trap does not catch a condition that has no resumption
    /// point, even through `ANY` -- the one spelling that makes the question
    /// askable at all, since `CALL ON SYNTAX` is a parse error. And `SIGNAL
    /// ON ANY` does catch it, which is the pair that keeps this a statement
    /// about `CALL` rather than about `ANY` not working.
    #[test]
    fn a_call_trap_declines_a_condition_with_nowhere_to_resume_but_a_signal_trap_takes_it() {
        let mut interp = Interp::new();
        let failure = run_source(
            &mut interp,
            b"call on any name uh\nsay 1/0\nexit\nuh:\nsay 'UH-MUST-NOT-RUN'\nreturn\n",
        )
        .unwrap_err();
        assert!(matches!(&failure, Failure::Raised(raised) if raised.number == 42));
        assert!(interp.out.is_empty());

        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"signal on any\nsay 1/0\nexit\nany:\nsay 'ANY-TRAPPED' sigl\n",
            ),
            b"ANY-TRAPPED 2\n".to_vec()
        );
    }

    // ---- fix round 1: the pending-trap delivery boundary, and its identity ----

    /// **Fix round 1's Critical, half (a).** The clause that finishes may
    /// itself be the `RETURN`: `aa`'s whole body is `return bb()`, and `bb`
    /// raises a trapped `USER` condition. The oracle runs the handler at that
    /// clause's own boundary -- so `SIGL` is line 7, `aa`'s `return bb()` --
    /// and then returns from `aa`.
    ///
    /// Against the delivery check at the bottom of `run_activation`'s loop,
    /// past a `match` whose `Flow::Return` arm returns, the handler never ran
    /// at all and `ZMARK` kept its pre-set `NOMARK`. Chosen so that failure
    /// prints `NOMARK` rather than the derived name `ZMARK`: a flag that is
    /// merely unset reads as plausible data.
    #[test]
    fn a_pending_trap_is_delivered_when_the_trapping_clause_is_a_return() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"call on user foo name uh\nzmark = 'NOMARK'\ncall aa\nsay 'end mark=' zmark\nexit\naa:\nreturn bb()\nbb:\nraise user foo return 'BBVAL'\nuh:\nzmark = 'HANDLER-AT' sigl\nreturn\n",
            ),
            b"end mark= HANDLER-AT 7\n".to_vec()
        );
    }

    /// **Fix round 1's Critical, half (b).** The same program with a second,
    /// unrelated `call cc` after it. The handler still belongs to `aa`'s
    /// `return bb()` clause (`SIGL` 8 here), and printed `HANDLER-AT 11` --
    /// the `cc:` label's own line -- before the fix, because it ran inside
    /// `cc`.
    ///
    /// **Which mechanism this actually pins, measured rather than assumed.**
    /// It dies when the delivery check is put back behind the `Return`/`Exit`
    /// arms, and *survives* when the activation identity is degraded to a
    /// stack depth -- because once the check runs at `aa`'s own `return
    /// bb()` boundary, the condition is gone before `cc` is ever called, and
    /// a depth is sufficient for that. So this test covers the placement, and
    /// `a_pending_trap_whose_activation_is_gone_is_never_delivered` is the
    /// one that covers the identity: the two mutations kill exactly one test
    /// each, which is what makes them two mechanisms rather than one.
    #[test]
    fn a_pending_trap_is_not_delivered_into_a_later_activation_at_the_same_depth() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"call on user foo name uh\nzmark = 'NOMARK'\ncall aa\ncall cc\nsay 'end mark=' zmark\nexit\naa:\nreturn bb()\nbb:\nraise user foo return 'BBVAL'\ncc:\nsay 'in cc'\nreturn\nuh:\nzmark = 'HANDLER-AT' sigl\nreturn\n",
            ),
            b"in cc\nend mark= HANDLER-AT 8\n".to_vec()
        );
    }

    /// **A third shape of the same defect, which the review did not reach and
    /// a depth cannot close.** The pending condition's activation is unwound
    /// by an error its *caller* traps, so it dies without ever finishing
    /// another clause; the caller then calls something else, which lands at
    /// the very depth the dead activation had.
    ///
    /// Measured: the oracle drops the condition outright -- `after mark=
    /// NOMARK` -- and against a depth-keyed `PendingTrap` we printed
    /// `after mark= HANDLER-AT 13`, having run the handler inside `cc`. The
    /// identity check is what makes a dead activation's pending condition
    /// undeliverable rather than merely unlikely to be delivered.
    ///
    /// **This is the test that pins the identity, and the only one.** It
    /// survives the mutation that reverts the delivery *placement* and dies
    /// under the one that degrades `ActivationId` to a stack depth; its
    /// sibling above does the reverse. The reason it can tell them apart is
    /// that here the activation never reaches another clause boundary at all
    /// -- the error unwinds it -- so no amount of moving the check helps, and
    /// only "that activation is gone" answers it.
    #[test]
    fn a_pending_trap_whose_activation_is_gone_is_never_delivered() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"signal on syntax name sh\ncall on user foo name uh\nzmark = 'NOMARK'\ncall aa\nsay 'end mark=' zmark\nexit\naa:\nsignal off syntax\nzq = bb() + 1/0\nreturn\nbb:\nraise user foo return 'BBVAL'\ncc:\nsay 'in cc'\nreturn\nuh:\nzmark = 'HANDLER-AT' sigl\nreturn\nsh:\nsay 'SYNTAX at' sigl\ncall cc\nsay 'after mark=' zmark\nexit\n",
            ),
            b"SYNTAX at 4\nin cc\nafter mark= NOMARK\n".to_vec()
        );
    }

    /// **Fix round 1's finding 2.** Once a `CALL ON` handler has returned,
    /// its condition is no longer active, so a later `RAISE PROPAGATE` has
    /// nothing to re-raise and is `98.918`.
    ///
    /// The report's Concern 1 called this unmeasured; one probe settled it.
    /// Against the unclearing version the program was silent at rc 0.
    #[test]
    fn a_returned_call_handler_leaves_no_active_condition_to_propagate() {
        let mut interp = Interp::new();
        let failure = run_source(
            &mut interp,
            b"call on user foo name uh\ncall sub\nsay 'resumed'\nraise propagate\nsay 'not reached'\nexit\nsub:\nraise user foo return 'SVAL'\nuh:\nsay 'UH ran'\nreturn\n",
        )
        .unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (98, 918));
        assert_eq!(interp.out, b"UH ran\nresumed\n".to_vec());
    }

    /// The adjacent case that keeps the clearing where it belongs: a `SIGNAL
    /// ON` handler that runs *on* -- `SIGNAL`s to another label and only then
    /// propagates -- must still find its condition. Measured, and it is why
    /// `offer_to_trap` has no equivalent of the line above.
    #[test]
    fn a_signal_handler_that_runs_on_can_still_propagate() {
        let mut interp = Interp::new();
        let failure = run_source(
            &mut interp,
            b"signal on syntax name th\nsay 1/0\nsay 'x'\nth:\nsay 'TH ran'\nsignal onward\nonward:\nsay 'onward'\nraise propagate\n",
        )
        .unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (42, 3));
        assert!(raised.delivery.positionless);
        assert_eq!(interp.out, b"TH ran\nonward\n".to_vec());
    }

    /// **Fix round 1's finding 3(a).** A `CALL ON` trap is removed for its
    /// handler's duration and **put back** afterwards, unlike a `SIGNAL ON`
    /// trap, which stays removed. `deliver_pending_trap` documented this and
    /// nothing tested it: deleting the re-insertion left the whole suite and
    /// the corpus gate green.
    ///
    /// Two raises, and the second one's handler run is the assertion -- an
    /// implementation that never puts the trap back prints `UH 2 / mid / end`
    /// and drops the second condition silently.
    #[test]
    fn a_call_trap_is_put_back_after_its_handler_returns() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"call on user foo name uh\ncall raiser\nsay 'mid'\ncall raiser\nsay 'end'\nexit\nraiser:\nraise user foo return 'RV'\nuh:\nsay 'UH' sigl\nreturn\n",
            ),
            b"UH 2\nmid\nUH 4\nend\n".to_vec()
        );
    }

    /// **Fix round 1's finding 3(b).** `RAISE PROPAGATE` of a condition with
    /// no catalogue entry ends the program silently rather than reporting.
    /// The guard that makes that true had no test, and deleting it does not
    /// merely go unnoticed -- it emits `Error 0:  <no message 0.0 in the
    /// catalogue>`, which is the exact failure mode `novalue_check`'s own doc
    /// comment says the design keeps unreachable.
    #[test]
    fn raise_propagate_of_an_unreportable_condition_ends_the_program_silently() {
        let mut interp = Interp::new();
        let ended = run_source(
            &mut interp,
            b"call on user foo name uh\ncall sub\nsay 'after'\nexit\nsub:\nraise user foo return 'SV'\nuh:\nsay 'UH ran'\nraise propagate\n",
        );
        assert!(
            ended.is_ok(),
            "a USER condition has no report to give, so this ends the \
             program rather than raising: {ended:?}"
        );
        assert_eq!(
            interp.out,
            b"UH ran\n".to_vec(),
            "`say 'after'` must not run -- the propagate ends the program"
        );
        assert!(
            !interp.trace.windows(7).any(|w| w == b"Error 0"),
            "no `Error 0` placeholder may reach stderr: {:?}",
            String::from_utf8_lossy(&interp.trace)
        );
    }

    /// **Fix round 1's finding 5.** `RAISE SYNTAX`'s argument is validated
    /// rather than used verbatim. Every row measured against the oracle; see
    /// `raise_syntax_condition` for the rule and the boundary probes.
    ///
    /// Before this, rows 5 and 6 rendered a `<no message N.M in the
    /// catalogue>` placeholder to the user at rc 216, row 8 rendered an
    /// unrelated catalogue entry at rc 25, and rows 7, 9 and 10 answered
    /// `Error 0` at **rc 0** -- a report on stderr beside a successful exit
    /// status, which is the global constraint's worst case rather than a
    /// cosmetic one.
    #[test]
    fn raise_syntax_validates_its_argument() {
        for (argument, expected, substitution) in [
            (&b"40.4"[..], (40u16, 4u16), None),
            (&b"40"[..], (40, 0), None),
            (&b"40.001"[..], (40, 1), None),
            (&b"99"[..], (99, 0), None),
            (&b"40.10"[..], (98, 941), Some("40010")),
            (&b"3.5"[..], (98, 941), Some("3005")),
            // A major with no `(major, 0)` catalogue entry renders the
            // original `major.sub` instead of the composed number. There are
            // 45 such majors in 1..=99, not the two an earlier version of
            // this comment claimed -- `50.1` below is one of the others, and
            // is here so the row set does not accidentally describe a
            // two-element special case.
            (&b"1"[..], (98, 941), Some("1.0")),
            (&b"2.1"[..], (98, 941), Some("2.1")),
            (&b"50.1"[..], (98, 941), Some("50.1")),
            (&b"87"[..], (98, 941), Some("87.0")),
            // Fix round 2's NEW 2: the sub is bounded at 1000, and the
            // boundary is what pins it -- `40.999` is a well-formed unknown
            // code and `40.1000` is not a code at all.
            (&b"40.999"[..], (98, 941), Some("40999")),
            (&b"40.1000"[..], (33, 904), None),
            (&b"40.99999"[..], (33, 904), None),
            // Fix round 2's NEW 4: each half is a Rexx number, and a bare
            // decimal point is not one.
            (&b"'4E1'"[..], (40, 0), None),
            (&b"'40.1E2'"[..], (98, 941), Some("40100")),
            (&b"'40.'"[..], (33, 904), None),
            (&b"'+40'"[..], (40, 0), None),
            (&b"0"[..], (33, 904), None),
            (&b"100"[..], (33, 904), None),
            (&b"999"[..], (33, 904), None),
            (&b"'abc'"[..], (33, 904), None),
        ] {
            let mut source = b"raise syntax ".to_vec();
            source.extend_from_slice(argument);
            source.push(b'\n');
            let mut interp = Interp::new();
            let failure = run_source(&mut interp, &source).unwrap_err();
            let Failure::Raised(raised) = failure else {
                panic!("{}: expected Raised", String::from_utf8_lossy(argument));
            };
            assert_eq!(
                (raised.number, raised.sub),
                expected,
                "{}",
                String::from_utf8_lossy(argument)
            );
            if let Some(substitution) = substitution {
                assert_eq!(
                    raised.additional,
                    vec![substitution.to_string()],
                    "{}",
                    String::from_utf8_lossy(argument)
                );
            }
            // The exit code is a consequence of the number, but the global
            // constraint is about the *band*: a raise must never exit 0, and
            // three of these did.
            assert_ne!(
                raised.exit_code(),
                0,
                "{}: a raise must not exit 0",
                String::from_utf8_lossy(argument)
            );
        }
    }

    // ---- fix round 2: the clause boundary inside nested bodies ----

    /// **Fix round 2's NEW 5.** A clause inside a `DO` body reaches its own
    /// boundary, not the `END`'s. Both iterations must see the handler's
    /// value, and `SIGL` must be the `call sub` clause's line (4), not the
    /// `say`'s.
    ///
    /// While the delivery check lived only in `run_activation`'s loop this
    /// printed `NOMARK` on both passes and `HANDLER-AT 5` once, after the
    /// loop -- wrong in timing and in `SIGL`. `run_bounded` is where a loop
    /// body's clauses are actually stepped.
    #[test]
    fn a_pending_trap_is_delivered_inside_a_do_body() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"call on user foo name uh\nzmark = 'NOMARK'\ndo i = 1 to 2\ncall sub\nsay 'after call' i 'mark=' zmark\nend\nsay 'end mark=' zmark\nexit\nsub:\nraise user foo return 'SV'\nuh:\nzmark = 'HANDLER-AT' sigl\nreturn\n",
            ),
            b"after call 1 mark= HANDLER-AT 4\nafter call 2 mark= HANDLER-AT 4\nend mark= HANDLER-AT 4\n".to_vec()
        );
    }

    /// The same property for the other two constructs `run_bounded` serves:
    /// a `WHEN`'s body and an `INTERPRET` fragment. Three callers, one rule
    /// -- and all three were wrong together, which is what made this a
    /// mechanism rather than a `DO` quirk.
    #[test]
    fn a_pending_trap_is_delivered_inside_a_when_body_and_a_fragment() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"call on user foo name uh\nzmark = 'NOMARK'\nselect\nwhen 1 = 1 then do\ncall sub\nsay 'in when mark=' zmark\nend\notherwise nop\nend\nsay 'end mark=' zmark\nexit\nsub:\nraise user foo return 'SV'\nuh:\nzmark = 'HANDLER-AT' sigl\nreturn\n",
            ),
            b"in when mark= HANDLER-AT 5\nend mark= HANDLER-AT 5\n".to_vec()
        );

        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"call on user foo name uh\nzmark = 'NOMARK'\ninterpret \"call sub; say 'inside mark=' zmark\"\nsay 'end mark=' zmark\nexit\nsub:\nraise user foo return 'SV'\nuh:\nzmark = 'HANDLER-AT' sigl\nreturn\n",
            ),
            b"inside mark= HANDLER-AT 3\nend mark= HANDLER-AT 3\n".to_vec()
        );
    }

    /// **Fix round 2's NEW 1.** A `CALL ON` handler can be delivered while a
    /// `SIGNAL ON` handler is already running -- one clause can queue the
    /// first and raise the second -- and when it returns, the condition it
    /// interrupted must come back.
    ///
    /// `zq = sub() + 1/0` does both: `sub` queues a trapped `USER`
    /// condition, then `1/0` raises a trapped `SYNTAX` one. The `SYNTAX`
    /// handler ends in `raise propagate`, which must re-raise **42.3**.
    /// Round 1 set `active_condition = None` when the call handler returned
    /// and got `98.918`; before round 1 nothing was cleared and it was
    /// silence at rc 0. Both interpreters agree on the delivery order (`UH`
    /// then `SH`), so the assertion isolates to the propagate.
    #[test]
    fn a_call_handler_restores_the_condition_it_interrupted() {
        let mut interp = Interp::new();
        let failure = run_source(
            &mut interp,
            b"signal on syntax name sh\ncall on user foo name uh\nzq = sub() + 1/0\nsay 'not reached'\nexit\nsub:\nraise user foo return 'SV'\nuh:\nsay 'UH ran' sigl\nreturn\nsh:\nsay 'SH ran' sigl\nraise propagate\n",
        )
        .unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!(
            (raised.number, raised.sub),
            (42, 3),
            "the SYNTAX condition the SIGNAL handler is running for, not the \
             USER one the CALL handler was delivered with, and not 98.918"
        );
        assert_eq!(interp.out, b"UH ran 3\nSH ran 3\n".to_vec());
    }

    // ---- fix round 3: a loop header is a clause too ----

    /// **Fix round 3's NEW-A.** A `DO` header is a Rexx clause: its control
    /// expressions run in it, and its boundary is its own, not the whole
    /// loop's. `do i = 1 to sub()` must report `SIGL` 3 -- the `DO` clause --
    /// and deliver the handler before the body's first pass.
    ///
    /// Before this the handler ran after the `END`, so this test pins the
    /// *boundary*. It does **not** pin the header's line: measured, a
    /// mutation that leaves `header_line` at whatever was already current
    /// keeps this test green, because the `DO` instruction's own
    /// `step_in_temps_frame` has already set line 3. The sibling below is
    /// what pins the line, since only a re-test can be on a different clause
    /// from the header. Two tests, two properties, stated because the
    /// mutation said so rather than assumed because they look related.
    #[test]
    fn a_loop_header_is_a_clause_with_its_own_boundary() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"call on user foo name uh\nzmark = 'NOMARK'\ndo i = 1 to sub()\nsay 'body' i 'mark=' zmark\nend\nsay 'end mark=' zmark\nexit\nsub:\nraise user foo return 1\nuh:\nzmark = 'HANDLER-AT' sigl\nreturn\n",
            ),
            b"body 1 mark= HANDLER-AT 3\nend mark= HANDLER-AT 3\n".to_vec()
        );
    }

    /// `WHILE` and `UNTIL` are re-tested once per pass, and **which clause
    /// that test belongs to changes**: it is the clause that transferred
    /// control back to the loop header. See `HeaderClause` for the oracle
    /// mechanism; this test carries the `DO`-then-`END` half, and
    /// `a_loop_retest_after_an_iterate_belongs_to_the_iterate_clause` carries
    /// the third member.
    ///
    /// **Round 3's version of this comment said "the `DO` clause on the first
    /// pass, the `END` clause on every one after", which was false**: it
    /// fitted the two members probed and broke two previously-matching
    /// programs, because an `ITERATE` never reaches `END` at all.
    #[test]
    fn a_while_retest_belongs_to_the_do_clause_then_to_the_end_clause() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"call on user foo name uh\nzmark = 'NOMARK'\nzn = 0\ndo while zn < sub()\nzn = zn + 1\nsay 'body' zn 'mark=' zmark\nend\nsay 'end mark=' zmark\nexit\nsub:\nraise user foo return 2\nuh:\nzmark = 'HANDLER-AT' sigl\nreturn\n",
            ),
            b"body 1 mark= HANDLER-AT 4\nbody 2 mark= HANDLER-AT 7\nend mark= HANDLER-AT 7\n".to_vec()
        );

        // `UNTIL` is tested only after a pass, so every one of its tests is
        // the `END` clause's -- the same rule with the first-pass case
        // absent, which is why it needs its own row rather than being
        // assumed to follow.
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"call on user foo name uh\nzmark = 'NOMARK'\nzn = 0\ndo until zn >= sub()\nzn = zn + 1\nsay 'body' zn 'mark=' zmark\nend\nsay 'end mark=' zmark\nexit\nsub:\nraise user foo return 2\nuh:\nzmark = 'HANDLER-AT' sigl\nreturn\n",
            ),
            b"body 1 mark= NOMARK\nbody 2 mark= HANDLER-AT 7\nend mark= HANDLER-AT 7\n".to_vec()
        );
    }

    // ---- fix round 4: NEW-1, the third member of the re-test family ----

    /// **Fix round 4's NEW-1, a regression round 3 introduced.** When an
    /// `ITERATE` ends a pass, the re-test that follows belongs to the
    /// `ITERATE`'s own clause -- the oracle re-enters the loop from inside
    /// `RexxActivation::iterate`, so `END` is never reached and never owns
    /// anything. Round 3 attributed it to `END` and turned two
    /// byte-for-byte-matching programs into divergences.
    ///
    /// Three rows, no trap in the first two, so those are a plain `SIGL`
    /// question rather than a delivery one:
    ///
    /// * the `ITERATE` row itself (oracle `2, 4, 4`; round 3 gave `2, 5, 5`);
    /// * **the adjacent success**, the same loop with the `ITERATE` removed,
    ///   which must stay on `END` (oracle `2, 4, 4` with the `END` at 4) --
    ///   that is what pins the rule to "who transferred control" rather than
    ///   to "not `END`";
    /// * an `UNTIL` loop, where the two attributions alternate within one
    ///   program (oracle `4, 6`), which no single-shape row can produce.
    #[test]
    fn a_loop_retest_after_an_iterate_belongs_to_the_iterate_clause() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"zn = 0\ndo i = 1 to 3 while zs() < 3\nzn = zn + 1\niterate\nend\nexit\nzs:\nsay sigl\nreturn zn\n",
            ),
            b"2\n4\n4\n".to_vec(),
            "the DO clause on the first test, then the ITERATE's own line twice"
        );

        // The adjacent success: drop the `ITERATE` and the body falls through
        // to `END`, which is line 4 here, so the numbers coincide only
        // because `END` moved up a line -- the rule being tested is which
        // clause, not which number.
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"zn = 0\ndo i = 1 to 3 while zs() < 3\nzn = zn + 1\nend\nexit\nzs:\nsay sigl\nreturn zn\n",
            ),
            b"2\n4\n4\n".to_vec(),
            "a fall-through pass still hands the re-test to the END clause"
        );

        // `UNTIL`, where one program shows both: the first test follows an
        // `ITERATE` on line 4, the second follows a fall-through to `END` on
        // line 6.
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"zn = 0\ndo until zs() >= 2\nzn = zn + 1\nif zn = 1 then iterate\nnop\nend\nexit\nzs:\nsay sigl\nreturn zn\n",
            ),
            b"4\n6\n".to_vec(),
            "the ITERATE clause, then the END clause, in one program"
        );
    }

    /// `END` is **not executed** when an `ITERATE` ends a pass, so it does not
    /// echo either -- the other half of NEW-1, found while measuring it.
    ///
    /// The oracle's reason is the same one: `RexxInstructionEnd::execute` is
    /// what calls `reExecute` on a fall-through, and `RexxActivation::iterate`
    /// is what calls it for an `ITERATE`; `END` is jumped straight over. The
    /// adjacent success is the same loop with the `ITERATE` removed, which
    /// must still echo `end` once per pass.
    #[test]
    fn end_does_not_echo_for_a_pass_an_iterate_ended() {
        let mut interp = Interp::new();
        run_source(
            &mut interp,
            b"trace r\nzn = 0\ndo while zn < 2\nzn = zn + 1\niterate\nend\n",
        )
        .unwrap();
        assert!(
            !String::from_utf8_lossy(&interp.trace).contains("*-* end"),
            "no END echo for an ITERATE-ended pass, got:\n{}",
            String::from_utf8_lossy(&interp.trace)
        );

        let mut interp = Interp::new();
        run_source(
            &mut interp,
            b"trace r\nzn = 0\ndo while zn < 2\nzn = zn + 1\nend\n",
        )
        .unwrap();
        assert_eq!(
            String::from_utf8_lossy(&interp.trace)
                .matches("*-* end")
                .count(),
            2,
            "the adjacent success: a fall-through pass does echo END, once per pass, got:\n{}",
            String::from_utf8_lossy(&interp.trace)
        );
    }

    // ---- fix round 4: NEW-2, the header clause of a nesting construct ----

    /// **Fix round 4's NEW-2.** An `IF`'s condition, a `SELECT CASE`'s
    /// expression and each listed `WHEN`'s condition are clauses in their own
    /// right, so a `CALL ON` handler queued by one runs at *that* clause's
    /// boundary -- before the branch, before the next `WHEN`, before
    /// `OTHERWISE`.
    ///
    /// Every row writes `then` on the line **after** the condition, which is
    /// what separates the right answer from the wrong one: with `then` on the
    /// same line, the clause that wrongly collected the boundary reports the
    /// same number, and five rounds of probes never told them apart. The
    /// one-line spellings are the adjacent successes in
    /// `a_single_line_then_reports_the_same_line_either_way`.
    #[test]
    fn a_construct_header_is_a_clause_with_its_own_boundary() {
        let trap = b"call on user foo name uh\nzmark = 'NOMARK'\n";
        let handler =
            b"exit\nsub:\nraise user foo return 'SV'\nuh:\nzmark = 'HANDLER-AT' sigl\nreturn\n";
        let rows: [(&[u8], &[u8]); 4] = [
            // The `IF` clause is line 3; `then` is line 4.
            (
                b"if sub() = 'SV'\n  then say 'yes mark=' zmark\n",
                b"yes mark= HANDLER-AT 3\n",
            ),
            // A false `WHEN` on line 4, a winning one on line 5.
            (
                b"select\nwhen sub() = 'NO' then say 'first mark=' zmark\nwhen 1 = 1 then say 'second mark=' zmark\nend\n",
                b"second mark= HANDLER-AT 4\n",
            ),
            // `SELECT CASE`'s own expression, line 3.
            (
                b"select case sub()\nwhen 'SV' then say 'hit mark=' zmark\notherwise say 'oth mark=' zmark\nend\n",
                b"hit mark= HANDLER-AT 3\n",
            ),
            // A false `WHEN` on line 4 falling through to `OTHERWISE`, whose
            // body must already see the handler's value: this row is wrong in
            // timing as well as line without the fix (`NOMARK`, then a later
            // delivery at line 6).
            (
                b"select\nwhen sub() = 'NO' then say 'hit mark=' zmark\notherwise\nsay 'oth mark=' zmark\nend\n",
                b"oth mark= HANDLER-AT 4\n",
            ),
        ];
        for (body, expected) in rows {
            let mut program = trap.to_vec();
            program.extend_from_slice(body);
            program.extend_from_slice(handler);
            let mut interp = Interp::new();
            assert_eq!(
                say_output(&mut interp, &program),
                expected.to_vec(),
                "header clause boundary for:\n{}",
                String::from_utf8_lossy(body)
            );
        }
    }

    /// The adjacent success for `a_construct_header_is_a_clause_with_its_own_
    /// boundary`: with `then` on the *same* line as the condition, the right
    /// answer and the wrong one coincide, and both must be the oracle's.
    ///
    /// Kept as its own test rather than folded in, because it is the row that
    /// shows why the passing cases were never evidence: **its output is the
    /// same with and without the fix.** Measured -- with the per-`WHEN`
    /// clause removed, this test does fail, but on `in_clause`'s tripwire
    /// ("a clause at line 4 began while a condition queued by this
    /// activation's clause at line 3 was still waiting"), never on a wrong
    /// value. That is the tripwire earning its place: the coincidence that
    /// hid the defect for four rounds is exactly the case a value assertion
    /// cannot see.
    #[test]
    fn a_single_line_then_reports_the_same_line_either_way() {
        let trap = b"call on user foo name uh\nzmark = 'NOMARK'\n";
        let handler =
            b"exit\nsub:\nraise user foo return 'SV'\nuh:\nzmark = 'HANDLER-AT' sigl\nreturn\n";
        let rows: [(&[u8], &[u8]); 3] = [
            (
                b"if sub() = 'SV' then say 'yes mark=' zmark\n",
                b"yes mark= HANDLER-AT 3\n",
            ),
            (
                b"select\nwhen sub() = 'SV' then say 'hit mark=' zmark\nend\n",
                b"hit mark= HANDLER-AT 4\n",
            ),
            // The false `IF`, which was already right before the fix and must
            // stay right: nothing is nested inside the branch it takes.
            (
                b"if sub() = 'NO'\n  then say 'yes mark=' zmark\n  else say 'no mark=' zmark\n",
                b"no mark= HANDLER-AT 3\n",
            ),
        ];
        for (body, expected) in rows {
            let mut program = trap.to_vec();
            program.extend_from_slice(body);
            program.extend_from_slice(handler);
            let mut interp = Interp::new();
            assert_eq!(
                say_output(&mut interp, &program),
                expected.to_vec(),
                "coinciding answers for:\n{}",
                String::from_utf8_lossy(body)
            );
        }
    }

    /// **An `INTERPRET` fragment is the one construct that must *not* end its
    /// header clause before running what it nests**, and this is the test
    /// that stops the NEW-2 fix being applied to it by analogy.
    ///
    /// The oracle runs fragment text in an activation of its own
    /// (`RexxActivation::interpret`) whose condition queue is separate and is
    /// merged back only on the way out, so a condition queued by the
    /// `INTERPRET` clause's own expression waits for that clause's boundary --
    /// measured, the fragment's `say` reads the handler's variable **unset**.
    #[test]
    fn an_interpret_clause_does_not_deliver_before_its_fragment_runs() {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(
                &mut interp,
                b"call on user foo name uh\nzmark = 'NOMARK'\ninterpret sub()\nsay 'end mark=' zmark\nexit\nsub:\nraise user foo return \"say 'frag mark=' zmark\"\nuh:\nzmark = 'HANDLER-AT' sigl\nreturn\n",
            ),
            b"frag mark= NOMARK\nend mark= HANDLER-AT 3\n".to_vec()
        );
    }

    /// **Fix round 3's NEW-B.** When the handler run at a clause's boundary
    /// itself fails, the clause the report blames is the one whose boundary
    /// ran it -- `call sub` -- not the enclosing `DO`.
    ///
    /// The neighbouring case is what pins it to the boundary rather than to
    /// anything about `DO`: the same program with no trap at all, failing
    /// directly inside `sub`, already blamed `call sub` correctly.
    #[test]
    fn a_handler_that_fails_at_a_clause_boundary_blames_that_clause() {
        let mut interp = Interp::new();
        run_source(
            &mut interp,
            b"call on user foo name uh\ndo i = 1 to 1\ncall sub\nsay 'after'\nend\nexit\nsub:\nraise user foo return 'SV'\nuh:\nsay 1/0\nreturn\n",
        )
        .unwrap_err();
        let mut sites = std::mem::take(&mut interp.failure_sites);
        sites.extend(interp.failure_site.take());
        assert!(
            sites.iter().any(|site| site.text == b"call sub".to_vec()),
            "expected the `call sub` clause among the echoes, got {:?}",
            sites
                .iter()
                .map(|site| String::from_utf8_lossy(&site.text).into_owned())
                .collect::<Vec<_>>()
        );
    }
}
