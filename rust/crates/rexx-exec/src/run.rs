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
//! `run_activation` is the **activation stack**'s own driver, and having more
//! than one activation to run is what shapes it rather than only adding arms:
//! it reads the activation's own body selector instead of hardcoding
//! `&program.main`, and it answers `Ended` rather than a bare
//! `Option<ObjRef>`, because a callee has two ways out that differ in what
//! the caller does next. `CALL`'s own work lives in `exec_call`, which is
//! `run_fragment`'s counterpart at the other kind of level boundary -- both
//! save and restore the same indent state and both `seal_site_level` on the
//! way out, and the one place they differ (this one *clears*
//! `clause_line_override` where the other *sets* it) is measured and stated
//! at both.

use crate::activation::{
    Activation, CallType, Entry, Inherited, InstanceVar, TraceEntry, Trap, TrappedCondition,
    body_of,
};
use crate::builtin;
use crate::clause::{ClauseEntry, ClauseOutcome, ClauseValue, HandlerExit};
use crate::error::{FailureSite, Raised, Search};
use crate::eval::logical_value;
use crate::ir::{BodyEngine, NodePath};
use crate::plan::BodyKey;
use crate::trace::{
    Announced, is_whole_number, mode_from_setting, raised_invalid_trace_letter,
    raised_numeric_trace_interactive_only,
};
use crate::value::{exact_small_int, within_digits};
use crate::{
    ActiveCondition, Argument, CallContext, Code, Engine, Failure, InstalledRoutine, Interp, Loud,
    Novalue, PendingTrap, VarHome,
};
use rexx_core::{BehaviourId, Body, Decoded, FrameId, ObjRef, ScopePools, SlotFrame};
use rexx_num::{ArithError, CompareOp, Number, SettingsError, compare_decoded};
use rexx_parse::{
    ConditionTrap, ControlExpr, DirectiveKind, EndStyle, Expr, ExprKind, Fragment, Instruction,
    InstructionKind, Loop, LoopConditional, LoopKind, NumericSetting, ProgramSource, Raise,
    SymbolId, Trace, Use, UseTarget, VariableRef, parse_interpret,
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
    /// (`run/tests.rs`) pins exactly this shape: a `DO` with an `ITERATE`
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
    ///
    /// **Boxed, and the box is what keeps `Flow` small.** [`LeaveOrigin`] is
    /// 48 bytes, of which 24 are an inline `Vec<u8>`, and it is the widest
    /// payload any variant here carries -- so holding it inline made every
    /// `Flow` in the interpreter 64 bytes wide, and every clause returns one.
    /// Measured on `emptyloop` with `perf stat -e instructions:u`: boxing it
    /// takes `size_of::<Flow>()` from 64 to 24 and removes 950 million
    /// instructions from a 40-billion-instruction run, 2.4% of the whole. The
    /// allocation it adds is paid once per `LEAVE`/`ITERATE` *executed*, not
    /// once per clause, and a loop executing one `LEAVE` per pass measured
    /// 4.1% cheaper boxed as well, so the trade is favourable on both sides.
    Leave(Option<SymbolId>, Box<LeaveOrigin>),
    /// `ITERATE`, bare or by name. See `Leave`'s own doc comment; the two
    /// variants are handled by nearly identical logic in `Do`/`Select`'s own
    /// arms, differing only in which of the oracle's measured asymmetries
    /// applies (`Select` never consumes a bare `Iterate` at all, and a named
    /// one that matches its own label but is not a loop is 28.5, not simply
    /// "not mine, keep looking").
    Iterate(Option<SymbolId>, Box<LeaveOrigin>),
    /// `SIGNAL label` and `SIGNAL VALUE`, once the target resolves to an
    /// instruction index.
    ///
    /// **A distinct variant from `Goto`, and that is not merely for
    /// clarity.** `Goto`'s own contract (`run_bounded`'s doc comment) is "in
    /// range for the body currently being stepped", which is exactly wrong
    /// for `SIGNAL`: the target always resolves against the running
    /// *activation's* own body (`resolve_signal_target`, mirroring
    /// `Interp::resolve_call`'s identical fix for `CALL`), and inside an
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
    /// fragments_own_index_space` (`run/tests.rs`) is a program
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
/// A callee has two ways out that differ in what the *caller* does next, and
/// the value alone cannot tell them apart -- `return` and `exit` with the same
/// value are the same `Option` and the opposite instruction. A bare
/// `Option<ObjRef>` is enough only while every activation is the program's own
/// and every way out of it stops the program.
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

/// Who emits a stepped clause's own `*-*` line
/// ([`Interp::in_stepped_clause_with`]).
///
/// **Two answers rather than a `bool`**, because the two are not two settings
/// of one switch: one asks the current `TRACE` setting and the other says the
/// question has already been answered somewhere the clause unit cannot see.
pub(crate) enum Echo {
    /// The clause unit asks [`Interp::tracing_clause`] and echoes if the
    /// answer is yes. Every tree-walker clause, and a promoted clause whose
    /// chunk was compiled under a setting that is no longer in force.
    Gated,
    /// The clause unit emits nothing: this clause's chunk already decided,
    /// and carries the decision as [`crate::ir::Op::TraceClause`] or as the
    /// absence of it.
    Compiled,
}

/// A stepped clause that is open: [`Interp::enter_stepped_clause`] makes one
/// and [`Interp::leave_stepped_clause`] spends it.
///
/// It carries what the clause's two halves have to hand each other and nothing
/// else -- the GC temps frame to truncate to, the watermark the debug tripwire
/// compares against, and the [`ClauseEntry`] the boundary itself needs. Every
/// other thing the second half does is computed from arguments the caller
/// already holds, which is what keeps this three fields rather than a copy of
/// the clause's own state.
///
/// **What the watermark checks, and why it is here rather than in
/// `pop_frame`.** `pop_frame` truncates to a watermark rather than popping one
/// frame, and its own doc comment forbids a balance assertion there: `eval.rs`
/// sites open a frame and then use `?`, so their own `pop_frame` goes
/// unreached on the error path and is healed by the clause unit's
/// unconditional, outer truncation -- an assertion inside `pop_frame` would
/// fire on the ordinary error path of a correct program. That healing is
/// exactly what makes `Err` uninteresting to check and `Ok` interesting: on
/// the `Ok` path every site did run its own `pop_frame`, so the stack must be
/// back at or above where this step found it. Below it means a step popped
/// temps it did not own -- someone else's roots, dropped early, which is the
/// direction that could turn into a use-after-free once a collector runs for
/// real.
///
/// **`#[must_use]`, and what that does and does not close.** Nothing outside
/// `clause.rs` can build the `ClauseEntry` inside this, so a leave with no
/// enter in front of it does not compile; an enter whose token is dropped
/// rather than spent warns. A token deliberately discarded is reached by
/// neither, which is the same standing exposure `clause.rs`'s module doc names
/// for the rest of that module's `pub(crate)` surface.
#[must_use]
pub(crate) struct SteppedClause {
    /// The clause boundary this entry opened.
    entry: ClauseEntry,
    /// The GC temps frame the clause's own work pushes into.
    frame: FrameId,
    /// `RootSet::temps_len` as the clause was opened.
    temps_at_entry: usize,
}

/// What a called name resolved to, decided in one place before any argument
/// is evaluated.
///
/// The fourth outcome -- none of the three below -- is not a variant: it
/// raises 43.1 at the point of decision, so nothing downstream can hold a
/// `Resolved` that has nothing to run. Three variants, three paths, and the
/// paths differ in more than which code runs: [`Interp::invoke_call`]'s own
/// doc comment has what the builtin path deliberately skips and what a
/// `::ROUTINE` starts from instead of inheriting.
///
/// **`Copy`, because a resolution is a value a call site may keep.**
/// [`Interp::resolve_call`] answers one and nothing downstream mutates it;
/// `crate::ir::Op::Call` records it against the op position it was resolved
/// at, which needs the answer to be a plain value rather than a borrow of the
/// table it came from.
#[derive(Clone, Copy)]
pub(crate) enum Resolved {
    /// A label in the *running activation's* body, at this instruction index.
    Label(usize),
    /// A builtin function name, and **which** builtin -- a row index, not a
    /// copy of the row, so nothing here can drift from the arity check and
    /// the code that live on that row together.
    ///
    /// Carrying it is what lets a call site keep its answer, which is the
    /// property [`crate::builtin::BuiltinTarget`] was introduced for and did
    /// not have while this variant was empty: resolution hashed the name to
    /// decide it was a builtin, and the dispatch behind it hashed the same
    /// name again to decide which one. A compiled call site resolves once and
    /// hashes never again.
    Builtin(crate::builtin::BuiltinTarget),
    /// A `::ROUTINE` this program installed. `InstalledRoutine::directive` is
    /// the same integer `Activation::body` and `BodyKey::directive` carry.
    Routine(InstalledRoutine),
}

/// Which of the two activation-pushing outcomes a resolved call took, kept
/// past the push so the decisions that follow it can read it.
///
/// [`Resolved`] cannot serve here: `Resolved::Builtin` returns before any
/// activation exists, and a type that still admits it would need a dead arm
/// at each of those reads. Each is measured and each differs between the
/// variants: whether `SIGL` is set, which constructor and pool the callee
/// gets, what `activation_indent` the callee starts at, and the receiver its
/// calling convention carries ([`entered_receiver`]).
#[derive(Copy, Clone)]
enum Entered {
    Label(usize),
    Routine(InstalledRoutine),
}

/// How the callee was reached: written in the program, or delivered to it.
///
/// **The distinction is the receiver's and nothing else's.** The oracle runs
/// both through one function, `RexxActivation::run`, and the callers differ
/// in the receiver they hand it: `RexxActivation::internalCall` passes its
/// own (`execution/RexxActivation.cpp:3313`) and
/// `RexxActivation::internalCallTrap` passes `OREF_NULL` (`:3343`), where
/// `run` assigns it at `:474`.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) enum CallEntry {
    /// A `CALL`, an internal function call, or either form reaching a
    /// `::ROUTINE` -- a call the program's own text asked for.
    Written,
    /// A `CALL ON` handler, run because a condition was delivered.
    Trap,
}

/// The receiver the callee's calling convention carries (D24), given the
/// caller's own.
///
/// Measured on the oracle, all three arms, with a `::method priv class
/// private` as the probe and `self~priv` as the send -- a private send is
/// allowed exactly when the sending activation's receiver is the object being
/// sent to (`RexxObject::checkPrivate`, `classes/ObjectClass.cpp:616`-`:620`):
///
/// * `CALL inner` inside a class method, `inner` sending `self~priv`: **rc 0**,
///   `private-reached`. So a label call inherits.
/// * the same send from that method's `CALL ON ERROR` handler: **rc 159**,
///   `97.2 Object "The K class" cannot accept private message "PRIV" from this
///   context.` So a trap does not.
/// * the same send from a `::ROUTINE` the method called with `self` as its
///   argument: **rc 159**, the same 97.2. So a routine does not.
///
/// The control for all three, `.K~priv` at the top level, is that same rc 159
/// and 97.2 -- which is what says the two refusals above are the receiver's
/// absence and not something about where the send was written.
///
/// **Latent in this crate today**, because no access scope is implemented
/// here; what reads it is `Interp::caller`, and `Interp::resolve`'s callers
/// are where the checks go.
fn entered_receiver(entered: Entered, entry: CallEntry, caller: Option<ObjRef>) -> Option<ObjRef> {
    match (entered, entry) {
        (Entered::Label(_), CallEntry::Written) => caller,
        (Entered::Label(_), CallEntry::Trap) | (Entered::Routine(_), _) => None,
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
/// budget it shares with `run_bounded` (and, once dispatch exists, dispatch)
/// is what a real fix has to bound. That is a documented minimum stack or a
/// shared depth budget, and it is not this task's.
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
pub(crate) const MAX_ACTIVATION_DEPTH: usize = 10_000;

/// The longest `ADDRESS` environment name accepted, beyond which the
/// instruction raises 29.1.
///
/// `MAX_ADDRESS_NAME_LENGTH` in `platform/unix/MiscSystem.cpp`. The C++ keeps
/// one per platform rather than one language constant, and **both are 250,
/// with identical `validateAddressName` bodies** -- `platform/windows/
/// MiscSystem.cpp` read directly, not assumed from the unix one. So the
/// per-platform spelling there is a structure, not a difference, and this
/// single constant is faithful to both.
///
/// Measured against the oracle on this host: 250 bytes is accepted and
/// reported back by `ADDRESS()` at length 250, 251 raises `Error 29.1` at
/// rc 227.
///
/// **Only the two setting forms check it.** A bare `ADDRESS` swaps two names
/// that were each validated when they were set, and `RexxActivation::
/// toggleAddress` accordingly validates nothing.
const MAX_ADDRESS_NAME_LENGTH: usize = 250;

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
    /// stepped, which **no caller produces**: `run_fragment` passes its own
    /// fragment source, so a `LEAVE`/`ITERATE` inside fragment text resolves a
    /// real site and becomes the report's innermost echo. See
    /// `Interp::clause_site`.
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
    /// instruction. `site` is the same clause's echo site, for the case
    /// where that re-test fails -- see [`HeaderClause::Iterate`].
    Iterated {
        line: usize,
        site: Option<(usize, Vec<u8>)>,
    },
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

impl HeaderOutcome {
    /// Whether the loop's block is still open once this header clause has
    /// finished -- what [`Interp::settle_block_indent`] takes.
    fn entered(&self) -> bool {
        matches!(self, HeaderOutcome::Continue)
    }
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
///
/// **`Copy`, with the `ITERATE`'s echo site held beside it rather than in it
/// -- a measurement, not a taste.** Every pass boundary assigns this, and an
/// owned `Vec<u8>` inside the enum makes that assignment a drop rather than a
/// store: read the old discriminant, test it against each variant that owns
/// nothing, branch, and only then write. Measured, the site held inside
/// instead: 17.0 more `instructions:u` per pass boundary on both
/// `bench-programs/emptyloop.rex` and `bench-programs/varlookup.rex`, which is
/// +2.681% and +1.268%. More than the drop's own instructions, because it also
/// denies `flat_loop_step_top` a register for the loop state's own address and
/// leaves it reloading that off the stack around every use.
#[derive(Clone, Copy)]
enum HeaderClause {
    /// The first pass: the `DO`/`LOOP` clause's own line.
    Do,
    /// The previous pass fell through to `END`.
    End,
    /// The previous pass ended in an `ITERATE`, whose own echo site is the
    /// [`IterateSite`] its holder carries beside this.
    Iterate {
        /// That `ITERATE` clause's own line -- `SIGL`'s quantity, which
        /// honours `clause_line_override` inside an `INTERPRET`.
        line: usize,
    },
}

/// The `(line, text)` echo site of the `ITERATE` a [`HeaderClause::Iterate`]
/// stands for, carried so that a re-test which *fails* can be blamed on it.
/// `LeaveOrigin` captured this the instant the `ITERATE` stepped; without
/// carrying it the pair is gone by the time the next header runs, and the
/// failure is misattributed to the `DO` clause (review round 1, F2).
/// **Not** `LeaveOrigin::indent`, which is that clause's own lexical one:
/// measured, an `ITERATE` nested two blocks deep inside the body echoes at
/// its own depth when it steps and at the *loop body's* depth on the failure
/// path.
///
/// Read only while the tag beside it is [`HeaderClause::Iterate`], so a
/// boundary that moves the tag off `Iterate` leaves this alone rather than
/// clearing it -- clearing is the drop the split exists to remove.
type IterateSite = Option<(usize, Vec<u8>)>;

/// **SPIKE, not for commit.** One repeating loop being driven from the op
/// driver's own frame: the state `run_repeating` holds in locals, held here
/// instead because the pass loop is the driver's rather than its own.
pub(crate) struct FlatLoop {
    /// The first op of the body, where every pass starts.
    pub(crate) op_body: u32,
    /// The body's own instruction range, which an escaping `Flow` is absorbed
    /// against exactly as `run_bounded`'s own bounds absorb it today.
    pub(crate) body_start: usize,
    pub(crate) end_index: usize,
    pub(crate) do_index: usize,
    resume: usize,
    label: Option<SymbolId>,
    do_indent: usize,
    loop_indent: usize,
    do_line: usize,
    end_line: usize,
    header_clause: HeaderClause,
    /// The echo site `header_clause` names when it is `Iterate`.
    iterate_site: IterateSite,
    /// `Some(true)` for `UNTIL`, `Some(false)` for `WHILE`, `None` for a loop
    /// with neither. **The condition's own node is not held here**, because
    /// this outlives the borrow of `code` a reference to it would need; the
    /// node is read back off the `DO` instruction at the one point per pass
    /// that tests it, and this says whether that read is owed at all.
    conditional: Option<bool>,
    state: LoopState,
}

impl FlatLoop {
    /// A `FlatLoop` naming nothing, which exists only to give
    /// [`Interp::flat_loop_start`] a box to write a real one into when the
    /// spare pool is empty. Every field is overwritten before anything reads
    /// one.
    const fn vacant() -> FlatLoop {
        FlatLoop {
            op_body: 0,
            body_start: 0,
            end_index: 0,
            do_index: 0,
            resume: 0,
            label: None,
            do_indent: 0,
            loop_indent: 0,
            do_line: 0,
            end_line: 0,
            header_clause: HeaderClause::Do,
            iterate_site: None,
            conditional: None,
            state: LoopState::Forever,
        }
    }

    /// Which clause a header or `UNTIL` test on this pass belongs to.
    fn header_line(&self) -> usize {
        match self.header_clause {
            HeaderClause::Do => self.do_line,
            HeaderClause::End => self.end_line,
            HeaderClause::Iterate { line } => line,
        }
    }
}

/// **SPIKE.** What a header clause's own answer means: `Some(flow)` is the
/// loop finishing, `None` is one more pass.
fn flat_header_outcome(
    header: ClauseOutcome<bool>,
    resume: usize,
) -> Result<Option<Flow>, Failure> {
    match header {
        ClauseOutcome::Ended(exit) => Ok(Some(Flow::Exit(exit.value()))),
        ClauseOutcome::Ran(Err(failure)) => Err(failure),
        ClauseOutcome::Ran(Ok(false)) => Ok(Some(Flow::Goto(resume))),
        ClauseOutcome::Ran(Ok(true)) => Ok(None),
    }
}

/// **SPIKE.** The `WHILE`/`UNTIL` of the `DO`/`LOOP` at `index`, read back off
/// the instruction because a [`FlatLoop`] outlives any borrow of `code`.
fn loop_conditional_of<'a>(code: &'a Code<'_>, index: usize) -> Option<&'a LoopConditional> {
    match &code.body.instructions.get(index)?.kind {
        InstructionKind::Do(body) | InstructionKind::Loop(body) => body.conditional.as_ref(),
        _ => None,
    }
}

/// **SPIKE.** What `Interp::flat_loop_start` decided.
pub(crate) enum FlatStart {
    /// Driven from the driver's frame. The state is on `Interp::flat_loops`;
    /// this is the body's own instruction range, which the driver's frame
    /// absorbs an escaping `Flow` against.
    Flat { body_start: usize, end_index: usize },
    /// The header said zero passes, so the construct is already over.
    Ended(Flow),
    /// Not a shape this spike drives: take the nested path, with the header
    /// values handed back so that the nested path can move them rather than
    /// this one copying them.
    Fallback(LoopHeaderValues),
}

/// **SPIKE.** What one pass boundary decided.
pub(crate) enum FlatStep {
    /// One more pass, from this op.
    Body(u32),
    /// The construct is over and this is its answer.
    Done(Flow),
}

/// What drives one repeating `DO`/`LOOP`'s own iteration, once its header
/// has already been evaluated and validated -- everything `LoopKind` can be
/// except `Simple` (a block, never repeats, and `run_loop_with_header`'s own
/// `Simple` arm never builds one of these at all) and `With` (the loud path).
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
    /// target takes the loud path in `run_loop_with_header` before one of
    /// these is ever built): iterates exactly once, binding `control` to
    /// `value` itself (measured, the brief's own framing: "a string and a
    /// number each iterate once, yielding themselves"). `remaining` is
    /// `FOR`'s own budget, already validated, independent of `done`.
    OverOnce {
        control: SymbolId,
        /// [`control_slot`], taken once when this loop was entered.
        at: Option<usize>,
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
        /// [`control_slot`], taken once when this loop was entered.
        ///
        /// **The one field here that is a cache rather than state**, and what
        /// makes it safe is that a name's slot in a frame never moves: `Plan`
        /// is immutable, an activation's `extra` bindings are only ever added
        /// to, and `RootSet::grow_slots` appends. A slot that later becomes an
        /// alias -- `PROCEDURE EXPOSE` -- keeps its index and is chased at
        /// every read and write, so exposure does not invalidate this either.
        at: Option<usize>,
        current: ControlValue,
        to: Option<Number>,
        by: Number,
        for_remaining: Option<u64>,
        /// `TO`/`BY` as plain integers, and the precision they hold for.
        cached_digits: u64,
        to_int: Option<i64>,
        by_int: Option<i64>,
        /// Whether the control variable is spelled simple, stem or compound.
        ///
        /// Taken when the loop is entered, because it is a property of the
        /// name alone and the name is fixed by the parse: `control` is a
        /// `SymbolId` this state owns, and the `Code` a pass is driven with
        /// is the one that compiled it. Both halves of a pass read it from
        /// here -- the re-read to pick how it reads, `bind_control` to pick
        /// how it writes.
        ///
        /// **The resolved *name* is not cached and must not be**, only the
        /// shape: a compound control resolves its tail afresh on every pass
        /// against whatever the tail variable holds now. Measured against
        /// the oracle, `a.=0; k=1; do a.k = 1 to 3; k=k+1; end` never
        /// advances, because each pass reads and writes a different tail.
        shape: NameShape,
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

/// A controlled loop's running control value.
///
/// `Small` is not a different value from `Wide`, only a cheaper way to hold
/// the same one: a counted loop spends its whole life on integers small
/// enough to add in a register, and holding those as a `Number` costs a heap
/// `Vec` per iteration for the increment and another for the bound test.
///
/// `Small` is produced **only** where the value was computed as an `i64` in
/// the first place, never by converting a `Number` that arrived some other
/// way. A `Number` carries its own spelling (`1.50` and `1.5` are different
/// objects), and while every spelling this could hold renders the same, the
/// conversion would be a second place where that has to stay true.
enum ControlValue {
    Small(i64),
    Wide(Number),
}

impl ControlValue {
    /// The value as a `Number`, borrowed when it already is one.
    fn number(&self) -> Cow<'_, Number> {
        match self {
            ControlValue::Small(value) => Cow::Owned(Number::from_i64(*value)),
            ControlValue::Wide(number) => Cow::Borrowed(number),
        }
    }

    /// The value as an integer the bound test may compare exactly, or `None`
    /// when the fuzzed comparison has to run instead. The caller supplies the
    /// other half of the interpreter's condition, that `NUMERIC FUZZ` is zero.
    ///
    /// **This asks which representation the value is in, not what it is
    /// worth**, because that is the question the interpreter asks:
    /// `RexxInteger::comp` (`interpreter/classes/IntegerClass.cpp`) takes its
    /// exact path only when both sides are already integer objects that fit
    /// `NUMERIC DIGITS` and `number_fuzz()` is zero, and anything else falls
    /// to `NumberString::comp`. A `Wide` value is one this crate is holding as
    /// a `Number` -- either a value with a fractional part, or an integer too
    /// wide for `DIGITS` -- and neither reaches the exact path.
    /// Deciding this by value would answer `Some` for a `Number` worth
    /// `100000002` that reached that worth by rounding `100000002.0`, which
    /// the interpreter never compares as two integers.
    fn small(&self, digits: u64) -> Option<i64> {
        match self {
            ControlValue::Small(value) => within_digits(*value, digits).then_some(*value),
            ControlValue::Wide(_) => None,
        }
    }
}

/// What one expression of a `DO`/`LOOP` header is for.
///
/// **The role decides three things at once**, and they are one fact rather
/// than three: which `>K>` tag the value is echoed under, how it is validated,
/// and which field of [`LoopHeaderValues`] it lands in. Keeping them on one
/// enum is what lets both engines evaluate a header through the same
/// [`Interp::accept_header_value`] while differing only in *what drives* the
/// sequence -- a Rust loop over [`HeaderPlan`], or the compiled stream's own
/// ops.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum HeaderRole {
    /// A controlled loop's own starting value. Echoed under no tag at all:
    /// measured, `trace i` over `do ii = 1 to 2` shows the initial value's
    /// `>L>` line and then `>K>   "TO"`, with no `>K>` of its own.
    Initial,
    To,
    By,
    /// A controlled loop's `FOR`.
    For,
    /// A bare `DO expr`'s repeat count, **echoed under the `FOR` tag**
    /// (measured: the oracle traces a bare repeat count under `FOR`, the
    /// same as an explicit `DO ... FOR n`), and validated against 26.2 where
    /// a `FOR` is 26.3.
    Count,
    /// `DO name OVER expr`'s target, echoed under the `OVER` tag.
    Over,
    /// `DO name OVER expr FOR expr`'s count, **echoed under the `FOR` tag**
    /// (measured against the oracle: `do qq over zs for 1` prints
    /// `>K>   "FOR" => "1"`, on both `trace i` and `trace r` -- the same tag
    /// a controlled loop's own `FOR` and a bare `DO`'s repeat count carry).
    OverFor,
}

impl HeaderRole {
    /// The `>K>` tag this value's own echo carries, or `None` for the one
    /// role the oracle echoes nothing for: `Initial`, a control variable's
    /// own starting value (its own doc has the measurement).
    pub(crate) fn keyword(self) -> Option<&'static str> {
        match self {
            HeaderRole::Initial => None,
            HeaderRole::To => Some("TO"),
            HeaderRole::By => Some("BY"),
            HeaderRole::For | HeaderRole::Count | HeaderRole::OverFor => Some("FOR"),
            HeaderRole::Over => Some("OVER"),
        }
    }

    /// How a loud failure names the position this value sits in.
    ///
    /// Not [`HeaderRole::keyword`]: that answers `None` for `Initial`, which
    /// is right for a `>K>` tag the oracle does not print and useless in a
    /// message that has to say which of a header's expressions was refused.
    /// The whole phrase rather than the keyword, because
    /// [`Loud::object_position`] serves sites that are not header roles at
    /// all.
    pub(crate) fn value_name(self) -> &'static str {
        match self {
            HeaderRole::Initial => "a DO header's initial value",
            HeaderRole::To => "a DO header's TO value",
            HeaderRole::By => "a DO header's BY value",
            HeaderRole::For | HeaderRole::OverFor => "a DO header's FOR value",
            HeaderRole::Count => "a DO header's repeat count",
            HeaderRole::Over => "a DO header's OVER target",
        }
    }
}

/// The header expressions of one `DO`/`LOOP`, in **the order they are
/// evaluated**, which is the order they were written in
/// (`Controlled::order`, recorded because an expression can have side
/// effects).
///
/// **One table, read by both engines.** `Interp::eval_loop_header` iterates it
/// and `ir::compile` emits one group of ops per entry from the same iteration,
/// so the evaluation order -- which is observable, and which interleaves
/// evaluation with `>K>` emission -- is one implementation rather than a Rust
/// loop and an op stream that have to be kept in step.
///
/// A fixed array rather than a `Vec`: a header is at most four expressions
/// (`DO i = a TO b BY c FOR d`), and this is built once per loop entry on the
/// tree-walker's own path.
pub(crate) struct HeaderPlan {
    roles: [HeaderRole; 4],
    len: usize,
}

impl HeaderPlan {
    fn new() -> HeaderPlan {
        HeaderPlan {
            roles: [HeaderRole::Initial; 4],
            len: 0,
        }
    }

    fn push(&mut self, role: HeaderRole) {
        debug_assert!(
            self.len < self.roles.len(),
            "a DO/LOOP header has more expressions than TO, BY, FOR and one control value"
        );
        self.roles[self.len] = role;
        self.len += 1;
    }

    /// The roles in evaluation order.
    pub(crate) fn roles(&self) -> &[HeaderRole] {
        &self.roles[..self.len]
    }
}

/// The header of `body`, or `None` for a `DO`/`LOOP` this crate refuses
/// **before evaluating anything**.
///
/// The three refusals are one answer here rather than three checks scattered
/// through the construct, and that placement is the semantics: `do counter c
/// with index i over x` fails loudly without evaluating `x`, and a stem `OVER`
/// target is detected from its own syntax rather than by evaluating it
/// (Deviation 1, `phase-4-exclusions.txt`: a stem target's tail order does not
/// reproduce the oracle's).
///
/// `COUNTER`'s own running-count bookkeeping is Phase-5-shaped extra state that
/// no other of `DO`/`LOOP`'s forms needs, and `DO WITH` sends `SUPPLIER` a
/// message, which nothing in this crate answers.
///
/// **A parenthesised stem is caught too**, measured
/// (`do_over_a_parenthesised_stem_target_is_also_caught`): a single
/// parenthesised sub-expression collapses to that sub-expression's own
/// `ExprKind` rather than being wrapped in `ExprKind::List`, so `(a.)` is
/// already `ExprKind::Stem` here. What escapes is a stem reached through
/// something that does not collapse this way -- a function call returning one
/// -- and no test may write one either way.
pub(crate) fn loop_header_plan(body: &Loop) -> Option<HeaderPlan> {
    if body.counter.is_some() {
        return None;
    }
    let mut plan = HeaderPlan::new();
    match &body.kind {
        // A block and a `FOREVER` loop each have no header expression at all.
        LoopKind::Simple | LoopKind::Forever => {}
        // `count_loop`'s own parser always calls `opt_expr`, which can answer
        // `None`; nothing in this crate's tests reaches `DO` with truly
        // nothing after it and no recognised keyword either, because
        // `create_loop`'s own `at_end()` check catches a bare `DO` first and
        // builds `LoopKind::Simple` instead.
        LoopKind::Count(None) => {}
        LoopKind::Count(Some(_)) => plan.push(HeaderRole::Count),
        LoopKind::Controlled(ctrl) => {
            plan.push(HeaderRole::Initial);
            for entry in &ctrl.order {
                plan.push(match entry {
                    ControlExpr::To => HeaderRole::To,
                    ControlExpr::By => HeaderRole::By,
                    ControlExpr::For => HeaderRole::For,
                });
            }
        }
        LoopKind::Over {
            target, for_count, ..
        } => {
            if matches!(target.kind, ExprKind::Stem(_)) {
                return None;
            }
            plan.push(HeaderRole::Over);
            if for_count.is_some() {
                plan.push(HeaderRole::OverFor);
            }
        }
        LoopKind::With { .. } => return None,
    }
    Some(plan)
}

/// The expression `role` names in `kind`, or `None` when that kind has no
/// expression for it.
///
/// A `None` a caller reaches is a role that did not come from
/// [`loop_header_plan`] for this same node, which is the one way the two can
/// come apart.
fn header_expr_for(kind: &LoopKind, role: HeaderRole) -> Option<&Expr> {
    match (kind, role) {
        (LoopKind::Count(expr), HeaderRole::Count) => expr.as_ref(),
        (LoopKind::Controlled(ctrl), HeaderRole::Initial) => Some(&ctrl.initial),
        (LoopKind::Controlled(ctrl), HeaderRole::To) => ctrl.to.as_ref(),
        (LoopKind::Controlled(ctrl), HeaderRole::By) => ctrl.by.as_ref(),
        (LoopKind::Controlled(ctrl), HeaderRole::For) => ctrl.for_count.as_ref(),
        (LoopKind::Over { target, .. }, HeaderRole::Over) => Some(target),
        (LoopKind::Over { for_count, .. }, HeaderRole::OverFor) => for_count.as_ref(),
        _ => None,
    }
}

/// The expression of `body`'s header at `slot` -- the compiled stream's own
/// addressing, where a slot is a position in [`HeaderPlan::roles`].
///
/// **The one resolution, read by the compiler and by both of the driver's
/// entries.** `ir::compile` asks it which expression a slot's ops are emitted
/// for, [`Interp::eval_chunk_expr`] asks it which expression an
/// `ir::Op::EvalExpr` at that slot evaluates, and [`Interp::chunk_node_at`]
/// asks it which expression an addressed op descends from -- so a slot means
/// the same thing to all three by construction rather than by three tables
/// agreeing.
pub(crate) fn loop_header_slot(body: &Loop, slot: u32) -> Option<&Expr> {
    let plan = loop_header_plan(body)?;
    let role = *plan.roles().get(slot as usize)?;
    header_expr_for(&body.kind, role)
}

/// One `DO`/`LOOP` header's evaluated and validated values.
///
/// **Filled one role at a time, in evaluation order**, because the order is
/// observable: `do i = 1 to 'a' by zf()` raises on `TO` before `BY` is
/// evaluated at all, so a shape that gathered every value first and validated
/// afterwards would call `zf` where the oracle does not.
#[derive(Default)]
pub(crate) struct LoopHeaderValues {
    /// A controlled loop's starting value, rounded at the digits in force,
    /// **in the representation the header's own value was in** -- an integer
    /// stays one, so the first pass's bound test can be the exact comparison
    /// the interpreter makes for two integer objects.
    initial: Option<ControlValue>,
    to: Option<Number>,
    by: Option<Number>,
    /// A `FOR`'s own budget, from either a controlled loop's `FOR` or a
    /// `DO OVER`'s.
    for_remaining: Option<u64>,
    /// A `DO OVER`'s target value.
    over: Option<ObjRef>,
    /// A bare `DO expr`'s repeat count.
    count: Option<u64>,
}

/// What `eval_condition` should do with the value it just computed, beyond
/// answering the caller's `bool` -- a caller-chosen variant rather than a
/// decision `eval_condition` makes on its own, because the same function
/// serves `IF`/`WHEN` (their own `>>>`, measured) and `WHILE`/`UNTIL`
/// (their own `>K>` instead, never a bare `>>>` alongside it, also
/// measured) and the two are genuinely different oracle behaviours, not
/// two spellings of one.
pub(crate) enum ConditionTrace<'a> {
    /// `IF`/`WHEN`'s own `>>>`.
    Result(usize),
    /// `WHILE`/`UNTIL`'s own `>K>`, tagged `"WHILE"`/`"UNTIL"`.
    Keyword(usize, &'a str),
}

/// Which keyword ended the activation, for [`Interp::returned_value`].
///
/// **A tag rather than two functions**, because the two arms would otherwise
/// be the same arm twice: `RETURN` and `EXIT` root, trace and carry their
/// value identically, and differ only in which [`Flow`] they answer.
#[derive(Clone, Copy)]
pub(crate) enum ReturnKeyword {
    Return,
    Exit,
}

/// Which end of the queue a line lands on, for [`Interp::queue_evaluated`].
///
/// **A tag rather than two functions**, for [`ReturnKeyword`]'s reason and
/// with a witness in `step`'s own history: `PUSH` and `QUEUE` were already one
/// arm choosing between `Queue::push` and `Queue::queue`, and this is that
/// choice named. See `queue.rs`'s module doc for the measured LIFO/FIFO order
/// the two spellings produce.
#[derive(Clone, Copy)]
pub(crate) enum QueueKeyword {
    Push,
    Queue,
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
            plan: Some(&plan),
        };
        // The key this body's plan was cached under, and the key its chunk is
        // cached under. **They have to be the same key**, and nothing but this
        // says so: `plan` is read off the activation while `key` is rebuilt
        // from the activation's program id and body selector, so a push that
        // paired a plan with the wrong id would put a body's chunk under
        // another body's name -- a wrong answer rather than a miss, exactly
        // what `BodyKey::directive`'s own doc says about the selector.
        //
        // Unpinned by anything else today because production loads one
        // program, so every `ProgramId` is 0 and `directive` alone
        // discriminates. A read, never an insert: `plan_for` would paper over
        // the mismatch by caching the plan a second time under the wrong key.
        let key = self.activation().body_key();
        debug_assert!(
            self.plans
                .get(&key)
                .is_some_and(|cached| Rc::ptr_eq(cached, &plan)),
            "the running activation's body key does not name the plan it is running with, so \
             its chunk would be cached under another body's name"
        );

        // **Engine selection, and it is here rather than at either caller.**
        // `run_activation` is the one function that runs an activation's
        // body, so entering an activation is what reaches the compiled
        // stream, rather than each place that enters one having to ask.
        // Putting the decision at the callers instead would need the `Code`
        // above rebuilt at each, and any caller that was missed would
        // tree-walk its whole body while a dual-engine gate still passed.
        //
        // `None` from `chunk_for` is a body that does not fit the stream's
        // index widths: it runs the loop below, and `Interp::chunks_refused`
        // has already counted the refusal so the fallback is not silent.
        // The setting in force *now* is what this body's chunk is looked up
        // by, not the one the program started under: the setting is an input
        // to compilation (D23), so entering a body under a second setting is
        // entering a second chunk.
        if matches!(self.engine, Engine::Ir)
            && let Some(chunk) = self.chunk_for(key, self.chunk_trace(), body, &plan)
        {
            return self.run_chunk(&code, &chunk, Some(&program.source));
        }

        let depth = self.activation_depth();

        while let Some(instruction) = code.body.instructions.get(self.activation().pc) {
            let index = self.activation().pc;
            self.grant_procedure_permission(instruction);
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
            // **Both lines are produced.** `run_fragment` passes
            // `Some(&fragment.source)` so the
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
            // The condition-trap boundary, and the reason it
            // is *here* rather than inside `step_in_temps_frame`: one offer
            // per activation, made by the activation that is unwinding. A
            // nested `run_bounded` (an `IF` branch, a `WHEN` body) shares
            // this activation's traps and must not get a second offer, and
            // a callee's own offer already happened in its own copy of this
            // loop before `invoke_call` re-threw. See
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
            if let Some(ended) = self.apply_flow(&code, flow)? {
                return Ok(ended);
            }
            debug_assert_eq!(
                self.activation_depth(),
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

    /// Transfers "is this the first instruction executed in this activation"
    /// to the step about to run, which `PROCEDURE` and `USE LOCAL` are the
    /// only readers of.
    ///
    /// A `Label` is transparent to it: measured, `call sub` into `sub:` /
    /// `lbl2:` / `procedure` runs, while the same with a `nop` in place of
    /// the second label is error 17.1. So a label neither grants the
    /// permission nor spends it.
    ///
    /// Cleared *before* the step and carried across it on
    /// `Interp::procedure_permitted`, which `step` takes on entry -- that is
    /// what stops an `INTERPRET` fragment or an `IF` branch inheriting it,
    /// measured through `sub: interpret "procedure"` being 17.1. See the
    /// field's own doc comment.
    ///
    /// Extracted from `run_activation`'s loop so a second engine discharges
    /// the same obligation rather than reimplementing it. It is per
    /// *clause*, not per instruction-node, so an engine that begins a
    /// clause without an `Instruction` in hand has to supply one.
    pub(crate) fn grant_procedure_permission(&mut self, instruction: &Instruction) {
        if !matches!(instruction.kind, InstructionKind::Label { .. }) {
            self.procedure_permitted =
                std::mem::take(&mut self.activation_mut().first_instruction_pending);
        }
    }

    /// Applies one clause's `Flow` to this activation.
    ///
    /// `Ok(None)` continues the body; `Ok(Some(_))` finishes the activation.
    ///
    /// Extracted from `run_activation`'s loop for the same reason as
    /// `grant_procedure_permission`. The `Leave`/`Iterate` arms
    /// are the reason this is worth extracting rather than copying: they are
    /// error semantics, not plumbing.
    ///
    /// **`run_fragment`'s own `Leave`/`Iterate` arms are not a call site for
    /// this, and the difference is the useful part.** They are the same
    /// event at a different boundary, so they share the four constructors
    /// and the `record_leave_failure` call -- but they resolve the name
    /// against the *fragment's* symbol table, which is the last point at
    /// which the id means anything, and they `seal_site_level` first.
    /// Absorbing them would mean parameterising both, which buys nothing
    /// here and would make one function answer to two boundaries.
    /// A second engine wanting this behaviour at a *third* boundary should
    /// re-read that before assuming one shared function covers it.
    pub(crate) fn apply_flow(
        &mut self,
        code: &Code<'_>,
        flow: Flow,
    ) -> Result<Option<Ended>, Failure> {
        match flow {
            Flow::Next => self.activation_mut().pc += 1,
            Flow::Goto(target) => self.activation_mut().pc = target,
            // `SIGNAL`, once its target has escaped every nested construct
            // and every `INTERPRET` fragment it fired from (`Flow::Signal`'s
            // own doc comment has why it cannot ride `Goto` to get here).
            // The only consumer, matching `Goto`'s own arm exactly: `target`
            // already resolved against this activation's own body
            // (`resolve_signal_target`), which is exactly the body `code` is
            // bound to.
            //
            // **The transfer resets this activation's trace indent to the
            // program's**, which is `RexxActivation::signalTo`'s own
            // `settings.traceIndent = 0` read directly, beside the
            // `blockNest = 0` that discards the block state this crate has
            // no counter for. Both addends of [`Interp::printed_indent`] go,
            // because the oracle's is one absolute counter rather than a
            // base and an elevation: a `SIGNAL` out of an escaped `WHEN`
            // would otherwise keep the elevation the escape put there.
            //
            // **Setting the base to `0` rather than restoring a saved one is
            // the whole of it, and a label's own lexical indent is why that
            // is enough.** The oracle refuses a label inside any block
            // instruction -- measured, 47.2 inside a `DO`/`LOOP`, 47.3
            // inside an `IF`, 47.4 inside a `SELECT` -- so a `SIGNAL`
            // target's `static_indent` is always `0` and the sum is `0` with
            // both addends cleared.
            //
            // Cleared on *this* activation, and `Interp::invoke_call`'s own
            // save/restore is what keeps it there: a `SIGNAL` inside a
            // `CALL`ed label echoes the rest of that label at the program's
            // indent (measured, oracle `6 *-* onward:` where this crate read
            // `6 *-*   onward:`), and the caller's own indent comes back
            // when the callee returns.
            Flow::Signal(target) => {
                self.activation_mut().pc = target;
                self.activation_indent = 0;
                self.indent_offset = 0;
            }
            // **The one place an activation's value stops being a clause's
            // temporary**, which is why the root that outlives the temps
            // stack is taken here rather than at each of the half-dozen
            // constructs that can produce one. `EXIT`, a top-level `RETURN`,
            // a `RAISE` with an `EXIT` tail and a handler's own exit all
            // arrive as one of these two variants; every one of them can end
            // up as the value `execute` hands `exit_code_for`, and by then
            // the frame that rooted it has been popped. See
            // [`Interp::root_exit_value`] for the measurement.
            Flow::Exit(value) => {
                if let Some(value) = value {
                    self.root_exit_value(value);
                }
                return Ok(Some(Ended::Exited(value)));
            }
            // The activation boundary `Flow::Return` was added to reach.
            // Every construct between the `RETURN` and here forwarded it
            // untouched; this is the one consumer.
            Flow::Return(value) => {
                if let Some(value) = value {
                    self.root_exit_value(value);
                }
                return Ok(Some(Ended::Returned(value)));
            }
            // Task 11: a `LEAVE`/`ITERATE` that reached the very top of the
            // program -- nothing anywhere, at any nesting depth, ever
            // matched it. This is the exhausted-search family, 28.1 (bare
            // `LEAVE`)/28.2 (bare `ITERATE`)/28.3 (named `LEAVE`)/28.4
            // (named `ITERATE`). `origin.indent` already holds this family's
            // own answer by the time it gets here -- every `Select`/`Do`
            // frame the search walked through on the way up has already
            // reset it to its own `static_indent` as it forwarded past
            // (`LeaveOrigin`'s own doc comment has the rule, corrected after
            // review: it is **not** always zero, only when every popped
            // frame along the way happened to sit at top level). 28.5 (a
            // named `ITERATE` that *did* match something, just not a loop)
            // is a different family, raised where the match was found, in
            // `Select`/`Do`'s own arms, and never reaches here.
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
        Ok(None)
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
    /// caller passes `Some`**, `run_fragment` included: a fragment resolves
    /// its clauses against its own source, with the
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
                let value = match expression {
                    Some(expression) => Some(self.eval(code, expression)?),
                    None => None,
                };
                self.say_evaluated(value);
                Ok(Flow::Next)
            }

            InstructionKind::Assignment { target, value } => {
                let value = self.eval(code, value)?;
                // `None`: this engine resolves nothing ahead of time, so the
                // write resolves its own slot the way it always has.
                self.assign_evaluated(code, target, value, None)?;
                Ok(Flow::Next)
            }

            // A simple variable back to unset (`Interp::clear_variable`, added
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

            // `EXIT`, bare or with a result: the spike had only the bare form
            // (`expression: None` matched literally, nothing else reaching
            // this arm at all). The value crosses out of the instruction loop
            // as `Flow::Exit`, unconverted -- `Interp::exit_code_for` (`lib.rs`)
            // is what turns it into a process exit code, and it runs once in
            // `execute` rather than here, because a `Flow::Exit` can also
            // come from inside a fragment (`run_fragment`'s own propagating
            // arm, below), and the conversion needs nothing this loop knows
            // that `execute` does not already have.
            //
            // The rooting, the `>>>` line and the `Flow` are
            // `Interp::returned_value`'s, shared with `RETURN`'s arm below and
            // with `crate::ir::Op::Return`.
            InstructionKind::Exit { expression } => {
                let value = match expression {
                    Some(expression) => Some(self.eval(code, expression)?),
                    None => None,
                };
                Ok(self.returned_value(value, ReturnKeyword::Exit))
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
            // run it against **this** activation, through `run_fragment`. The
            // arm is thin because `run_fragment` is where the work is.
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
                // is what the `run_fragment` call below produces. Handing
                // `run_fragment` a `Some(&fragment.source)` and nothing else
                // is not enough, for two independent reasons: the
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
                // That is a divergence with nothing to do with fragments,
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
                // **The fragment's own condition queue**, which the depth is
                // the key to rather than a second collection --
                // `Interp::fragment_depth` has why the oracle has one and what
                // was measured on either side of it. Incremented rather than
                // replaced the way the values saved above are, because a
                // nested fragment inherits each of those and needs a level of
                // its own here.
                self.fragment_depth += 1;
                // `saved_line` read before the replace above is also the
                // answer to "is a fragment already running", which is the one
                // extra thing `enter_fragment` needs and the only place it is
                // in hand. Nothing new is tracked for it.
                let saved_entry = self.enter_fragment(saved_line.is_some());
                let flow = self.run_fragment(text);
                // **A condition still queued at this depth dies with the
                // fragment**, on the failing path as well as this one, because
                // the activation whose queue it was in is what ends. Measured,
                // `interpret 'zq = raiser()'` with a handler that requeues: the
                // oracle runs the first handler and never runs the second, and
                // the enclosing clause's own boundary does not pick it up.
                // Dropping the entries also keeps a program that runs
                // `INTERPRET` in a loop from accumulating undeliverable ones.
                let depth = self.fragment_depth;
                self.pending_traps
                    .retain(|pending| pending.fragment_depth != depth);
                // **Nothing deeper than the fragment just left may survive
                // it**, which is the invariant that lets the delivery key be
                // an equality rather than a comparison: a deeper entry would
                // belong to a fragment that already ran this same discard on
                // its own way out, and a boundary out here would then have to
                // decide whether it inherits one. Asserted rather than
                // reasoned about, because the discard runs inside a
                // `deliver_pending_traps` that a handler can have re-entered.
                debug_assert!(
                    self.pending_traps
                        .iter()
                        .all(|pending| pending.fragment_depth < depth),
                    "a condition queued inside a fragment outlived that fragment's own exit"
                );
                self.fragment_depth -= 1;
                self.leave_fragment(saved_entry);
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
                let targets = if_targets(&code.body.instructions, *false_target);
                let false_target = targets.false_target;
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
                let holds =
                    match self.in_clause(code, line, |it| it.eval_if_condition(code, condition))? {
                        ClauseOutcome::Ended(exit) => return Ok(Flow::Exit(exit.value())),
                        ClauseOutcome::Ran(holds) => holds?,
                    };
                if holds {
                    let resume = targets.resume;
                    match self.run_bounded(
                        code,
                        index + 1,
                        false_target,
                        source,
                        BodyEngine::TreeWalker,
                    )? {
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
                // **The `SELECT` clause ends when its `CASE` expression has
                // been evaluated** -- same rule and same reason as `IF`'s,
                // just above: the oracle's `RexxInstructionSelect::execute`
                // returns here, and every `WHEN` after it is an instruction
                // of its own. Measured: `select case sub()` on line 3 with
                // `when 'SV' then say ...` on line 4 reports `SIGL` 3 on the
                // oracle and reported 4 here. Entered unconditionally, `CASE`
                // or no `CASE`, because the oracle's boundary is after the
                // instruction rather than after the expression. **A plain
                // `SELECT`'s clause queues nothing itself and its boundary
                // still has work**: a condition an earlier clause queued and
                // that clause's handler requeued is delivered here, measured
                // under `trace r` on both engines.
                let select_line = self.clause_state.line();
                let mut case_value: Option<ObjRef> = None;
                match self.in_clause(code, select_line, |it| {
                    let Some(case_expr) = case else {
                        return Ok(());
                    };
                    case_value = Some(it.select_case(code, case_expr)?);
                    Ok(())
                })? {
                    ClauseOutcome::Ended(exit) => return Ok(Flow::Exit(exit.value())),
                    ClauseOutcome::Ran(ran) => ran?,
                }
                // The hand-off an absorbed `WhenCase` needs and nothing else
                // threads to it -- `lib.rs`'s own doc comment on
                // `current_case_text` has the full argument, including the
                // disclosed nested-`SELECT CASE` limitation.
                let case_text = self.open_select_case(case_value);
                for &when_index in whens {
                    let when_instruction = &code.body.instructions[when_index];
                    // **Each listed `WHEN` is a clause of its own, and the
                    // clause unit is what says so** -- `in_stepped_clause`,
                    // the same entry point an unpromoted instruction reaches
                    // through `step_in_temps_frame`, rather than a bare
                    // `in_clause` with the rest of a clause's obligations
                    // written out beside it. In the oracle every `WHEN` is an
                    // instruction the activation's loop fetches separately, so
                    // a condition queued while testing one is delivered before
                    // the next `WHEN`, before `OTHERWISE`, and before a matched
                    // `WHEN`'s own body; all three were measured wrong before
                    // a boundary existed here at all.
                    //
                    // **What the clause unit discharges that the hand-rolled
                    // version did not, and both were measured.** The clause
                    // echo, the value indent and the *condition's* own failure
                    // site were written out at this call site; the failure site
                    // of the clause's **boundary** was not, so a `CALL ON`
                    // handler delivered here and failing was blamed on the
                    // enclosing `SELECT`. Measured against the oracle: `call on
                    // user zx name h` / `select` / `when raiser() = 'V' then
                    // ...` with a handler that divides by zero echoes `3 *-*
                    // when raiser() = 'V'`, and this arm echoed `2 *-* select`.
                    // The indent was wrong for the same reason and in the same
                    // program: the hand-rolled version passed the `WHEN`'s
                    // indent to `scan_when` as an argument and never wrote it
                    // to `current_value_indent`, so the handler's own
                    // activation was based two columns short of the oracle's.
                    let scanned =
                        self.in_stepped_clause(code, when_index, when_instruction, source, |it| {
                            it.scan_when(code, when_instruction, case_text.as_deref())
                        })?;
                    let holds = match scanned {
                        ClauseOutcome::Ended(exit) => return Ok(Flow::Exit(exit.value())),
                        ClauseOutcome::Ran(ran) => ran?,
                    };
                    if holds {
                        let targets = when_targets(&when_instruction.kind, len);
                        let body_end = targets.body_end;
                        let resume = when_resume(&targets);
                        let flow = self.run_bounded(
                            code,
                            when_index + 1,
                            body_end,
                            source,
                            BodyEngine::TreeWalker,
                        )?;
                        // F-EX1, and the classifier is [`select_escape`] so
                        // that both engines make this decision once. **Do not
                        // clear `indent_offset` on the redirect**: `OTHERWISE`'s
                        // own marker *and its whole body* need the offset still
                        // active through the dispatch (`lib.rs`'s own doc
                        // comment on `indent_offset` has the measured
                        // transcript), and what restores it to `0` is the end
                        // of that dispatch, not its start.
                        return match select_escape(*otherwise, flow) {
                            SelectEscape::Otherwise(target) => {
                                self.run_otherwise(code, index, *label, target, *end, source)
                            }
                            SelectEscape::Forward(flow) => {
                                self.leave_select(code, index, *label, resume, flow)
                            }
                        };
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
                // **Anything that reaches `step` is being stepped by the
                // tree-walker**, which is a property of this function's own
                // signature rather than of what its callers happen to pass:
                // `step` takes no [`BodyEngine`], so there is no engine here to
                // forward and none can be threaded in without changing it. A
                // promoted `DO`/`LOOP` does not come through here at all -- its
                // header is a compiled clause region and `ir::Op::LoopRun`
                // enters `run_loop_with_header` with the chunk's own engine.
                self.run_loop(
                    code,
                    index,
                    instruction,
                    body,
                    source,
                    BodyEngine::TreeWalker,
                )
            }

            // `LEAVE`/`ITERATE`, bare or by name -- Task 11. Resolves to
            // data, not a failure (`Flow::Leave`'s own doc comment): whether
            // this instruction's own name matches anything is answered by
            // whichever `Do`/`Select` (or `run_activation`'s own top level,
            // if none does) inspects the `Flow` this returns, never here.
            InstructionKind::Leave { name } => Ok(Flow::Leave(
                *name,
                Box::new(self.leave_origin(code, index, source, instruction)),
            )),
            InstructionKind::Iterate { name } => Ok(Flow::Iterate(
                *name,
                Box::new(self.leave_origin(code, index, source, instruction)),
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

            // `CALL name`, `CALL "name"`, `CALL (expr)` and `CALL ON`/`CALL
            // OFF`. The one arm of `rexx_parse::Call` that stays loud keeps
            // its own owner (`instruction_owner`, `lib.rs`): `Qualified`
            // (`CALL ns:name`) is Phase 5's.
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
            // The rooting, the `>>>` line and the `Flow` are
            // `Interp::returned_value`'s, whose own doc has the two-indent
            // measurement that says which of the two `>>>` lines a returned
            // value produces belongs here.
            InstructionKind::Return { expression } => {
                let value = match expression {
                    Some(expression) => Some(self.eval(code, expression)?),
                    None => None,
                };
                Ok(self.returned_value(value, ReturnKeyword::Return))
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
                rexx_parse::Signal::Label(name) => self.signal_to_label(name),
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
                    // Rooted here rather than inside `signal_to_value`, which
                    // the compiled stream reaches with the value already in a
                    // register: the temp is what roots it across the render
                    // there, and a second one would be a frame this clause
                    // does not own.
                    self.roots.push_temp(value);
                    self.signal_to_value(value)
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

            // `EXPOSE`: binds this method's names to the receiving object's
            // pool for the scope the method was declared in. See
            // `exec_expose`.
            InstructionKind::Expose { variables } => {
                self.exec_expose(code, variables)?;
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

            // `PUSH`/`QUEUE line` (I15). One arm, not two copies that can
            // drift (review round 1's M4): the two spellings differ only in
            // which end of the queue the value lands on, decided below by
            // which variant matched. The rendering, the `>>>` line and the
            // write are `Interp::queue_evaluated`'s, shared with
            // `crate::ir::Op::Queue`.
            InstructionKind::Push { expression } | InstructionKind::Queue { expression } => {
                let value = match expression {
                    Some(expression) => Some(self.eval(code, expression)?),
                    None => None,
                };
                let keyword = if matches!(instruction.kind, InstructionKind::Push { .. }) {
                    QueueKeyword::Push
                } else {
                    QueueKeyword::Queue
                };
                self.queue_evaluated(value, keyword);
                Ok(Flow::Next)
            }

            // `PARSE`, in every source spelling, plus the two short forms that
            // are the same instruction with `UPPER` already set: `ARG
            // template` is `PARSE UPPER ARG template` and `PULL template` is
            // `PARSE UPPER PULL template`. One arm, because `rexx-parse`
            // builds the identical `Parse` body for all three (its
            // `parse_instruction_body` takes the implied source and sets
            // `upper` from it), so a second arm here would be a second copy of
            // the dispatch and nothing else.
            //
            // Confirmed rather than assumed, with arguments `mIxEd CaSe`: `arg
            // n1 n2` gives `MIXED`/`CASE`, `parse arg n3 n4` gives
            // `mIxEd`/`CaSe`, and `parse upper arg n5 n6` gives `MIXED`/`CASE`.
            // `UPPER` is a keyword only *before* the source: `arg upper t4`
            // assigns `UPPER = 'MIXED'` and `t4 = 'CASE'`, taking `UPPER` as an
            // ordinary template target -- which needs no special handling here
            // because `rexx-parse` has already resolved it that way.
            //
            // The template engine, the trace shape and the movement rules are
            // `parse_template.rs`'s; this arm is the dispatch.
            InstructionKind::Parse(parse)
            | InstructionKind::Arg(parse)
            | InstructionKind::Pull(parse) => {
                self.exec_parse(code, parse, None)?;
                Ok(Flow::Next)
            }

            // `ADDRESS`, the three forms that only name an environment: the
            // constant `ADDRESS env`, the computed `ADDRESS VALUE expr` (and
            // its parenthesised spelling), and the bare toggle. See
            // `exec_address`.
            //
            // `ADDRESS env command` and any `WITH` redirection are the
            // command dispatch, and both fail loudly naming Phase 7 --
            // `instruction_owner` (`lib.rs`) draws the identical line, and
            // `owners.rs`'s own `Address::Command`/`Address::Environment`
            // rows are what hold the two matches equal.
            //
            // `io` without `command` is a real shape, not a defensive extra:
            // `address foo with output stem o.` sets the environment *and*
            // registers a redirection for later commands, which is the
            // `RexxInstructionAddressWith` half of the instruction.
            InstructionKind::Address(address) => {
                if address.command.is_some() || address.io.is_some() {
                    return Err(Loud::instruction(&instruction.kind).into());
                }
                self.exec_address(code, address)?;
                Ok(Flow::Next)
            }

            // A message send as a whole clause: `q~append(1)`, `q~~append(1)`
            // and the message-assignment form `q[1] = 2`. See
            // `exec_message`.
            InstructionKind::Message { term, value } => {
                self.exec_message(code, term, value.as_ref())
            }

            other => Err(Loud::instruction(other).into()),
        }
    }

    /// A message send that is a clause of its own
    /// (`RexxInstructionMessage::execute`, `MessageInstruction.cpp:151`).
    ///
    /// **What it does that the expression form does not is settle `RESULT`**,
    /// as a `CALL` does. Measured, `'abc'~length` followed by `say result`
    /// prints `3`, and `'abc'~~length` prints `abc` -- the cascade's
    /// replacement of the result by the target happens before `RESULT` is
    /// set, not after.
    ///
    /// **The oracle's other branch drops `RESULT` for a send that produced no
    /// value**, and only a Rexx body can produce one -- a primitive method's
    /// implementation returns an `ObjRef` by its own signature. Measured on
    /// `::method quiet class` ending in a bare `return`: after `result =
    /// 'unset'` and `.K~quiet`, `symbol('RESULT')` is `LIT`.
    ///
    /// **There is no `>>>` line**, measured: the clause's own value is not a
    /// result the instruction reports, so `>M>` is the last line a traced
    /// send emits.
    ///
    /// `value` is the message-assignment form's right-hand side. The oracle
    /// builds that form as an ordinary message instruction whose name has
    /// gained a `=` and whose argument list has gained that value in front
    /// (`MessageInstruction.cpp:76-88`), which is why it shares this arm
    /// rather than having one of its own -- and why the scope-override check
    /// applies to it too, measured: `x~a:super = 2` is 88.914.
    pub(crate) fn exec_message(
        &mut self,
        code: &Code<'_>,
        term: &Expr,
        value: Option<&Expr>,
    ) -> Result<Flow, Failure> {
        let ExprKind::Message {
            target,
            name,
            super_class,
            args,
            cascade,
        } = &term.kind
        else {
            // `rexx-parse` builds this variant only from a message term
            // (`instruction.rs`'s `message`), so nothing else can arrive;
            // loud rather than a panic, on the standing rule that a parser
            // guarantee the type system does not carry must not abort.
            return Err(Loud::expression(&term.kind).into());
        };
        // **`message_term` directly rather than through `Interp::eval`**,
        // even for the form that is an ordinary expression: `eval`'s own
        // `ExprKind::Message` arm turns a valueless send into 91.999, which
        // is the expression position's error and not this one's -- measured,
        // a whole-clause `.K~m` on a method ending in a bare `return` is
        // rc 0.
        //
        // The `enter_eval_node` pair is the one thing `eval` did for this
        // node that is kept rather than dropped: it is where
        // `MAX_EVAL_DEPTH` counts from, and this arm should count the term
        // at the depth an expression would. **No probe here separates the
        // two.** Measured, `(...)~length` as a whole clause and as an
        // assignment's right-hand side both run at 50,000 nested
        // parentheses and both refuse at 50,001, with this pair present and
        // with it deleted -- so it is here to leave the depth where it was,
        // not on the strength of an observable. The assignment form below
        // takes no level, which is likewise where it stood already.
        // `eval`'s other contribution, `trace_intermediate`, is an empty arm
        // for `ExprKind::Message` and nothing is lost by not calling it.
        let mut assigned_name;
        let result = match value {
            None => {
                let probe = 0u8;
                self.enter_eval_node(&raw const probe)?;
                let sent = self.message_term(
                    code,
                    &crate::dispatch::MessageTerm {
                        target,
                        name,
                        super_class: super_class.as_deref(),
                        args,
                        cascade: *cascade,
                        assigned: None,
                    },
                );
                self.depth -= 1;
                sent?
            }
            Some(value) => {
                assigned_name = name.to_vec();
                assigned_name.push(b'=');
                self.message_term(
                    code,
                    &crate::dispatch::MessageTerm {
                        target,
                        name: &assigned_name,
                        super_class: super_class.as_deref(),
                        args,
                        // The oracle builds this form as `KEYWORD_MESSAGE`
                        // whatever the term's own tilde count, so a `~~`
                        // written here is not a cascade.
                        cascade: false,
                        assigned: Some(value),
                    },
                )?
            }
        };
        let slot = self.reserved_result_slot();
        let frame = self.activation().frame;
        // **A send that produced no value drops `RESULT`** rather than
        // leaving the previous one in place -- the same rule a bare `return`
        // from a `CALL` follows. Measured: `result = 'unset'` then `.K~m`
        // then `symbol('RESULT')` is `LIT`.
        match result {
            Some(result) => {
                self.roots.push_temp(result);
                self.set_variable(frame, slot, result);
            }
            None => self.clear_variable(frame, slot),
        }
        Ok(Flow::Next)
    }

    /// `PROCEDURE`, with or without an `EXPOSE` list (D9r).
    ///
    /// Two things happen here, in this order, and the order is the design:
    /// every exposed name is resolved **while the caller's frame is still the
    /// top one**, and only then does the callee get a frame of its own. That
    /// is what lets a computed `expose (v)` naming a symbol no instruction
    /// mentions go through `Interp::slot_of` -- which may call
    /// `RootSet::grow_slots` -- without ever growing a non-top frame. The
    /// invariant `grow_slots` asserts is therefore untouched:
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
        // needed -- every other entry fails the second, and anything after
        // another instruction in the same activation fails the first.
        //
        // **A `::ROUTINE` is not an internal call**, measured on the oracle
        // both ways round: `call sub` and `qq = sub()` into a `::routine sub`
        // whose first instruction is `procedure` are 17.1 at rc 239, where
        // the identical pair into an internal label runs. `Entry`'s own doc
        // has the table, including the `::METHOD` row.
        let entered_by_internal_call = match self.activation().entry {
            Entry::InternalCall => true,
            Entry::TopLevel | Entry::Routine | Entry::Method => false,
        };
        if !(first_instruction && entered_by_internal_call) {
            return Err(Raised::procedure_out_of_place().into());
        }

        // The swap at the end of this function gives the callee a frame of
        // its own, and an activation that already owned one would be pushing
        // a second onto the same stack. That is the state
        // `RootSet::grow_slots` and `pop_slots` catch a step or two later,
        // by which time the instruction that caused it has returned, so the
        // invariant is asserted where it is established rather than where
        // the damage surfaces. `Activation::nested` builds the entry kind
        // admitted above, and it starts the callee sharing.
        assert!(
            !self.activation().owns_frame,
            "a PROCEDURE admitted in an activation that already owns its frame"
        );

        let names = self.expose_names(code, variables)?;

        // Resolved against the pool still in force, which is the caller's:
        // this activation has not swapped in a frame of its own yet.
        let outer = self.activation().frame;
        let mut bindings: Vec<(Box<[u8]>, usize, VarHome)> = Vec::with_capacity(names.len());
        for name in names {
            // Whole stems alias fine -- the stem object lives in one slot,
            // so aliasing that slot shares the object and every measured
            // stem transcript falls out of it. A single tail does not; see
            // `Loud::compound_expose`.
            if shape_of(&name) == NameShape::Compound {
                return Err(Loud::compound_expose("PROCEDURE EXPOSE", &name).into());
            }
            let slot = self.slot_of(&name);
            // A name the enclosing method exposed has no frame storage to
            // alias: its home is the object's pool, and the callee gets the
            // same home rather than a slot. Measured -- a class method
            // exposing `v` and calling `inner: procedure expose v`, which
            // assigns `v` -- the object variable is what changes.
            let target = match self.exposure(outer, slot) {
                Some(var) => VarHome::Instance(Box::new(var.clone())),
                None => VarHome::Slot(self.roots.slot_ref(outer, slot)),
            };
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
        if let Some(caller) = self.caller_activation_mut() {
            caller.extra = resolved;
        }

        // Sized from the caller's *current* frame length rather than from
        // `plan.len()`: an exposed name may sit at an index the caller grew
        // into, and that same index has to address something on this side of
        // the alias too.
        let len = self.roots.frame_len(outer);
        let inner = self.roots.push_slots(len);
        let mut exposed: Vec<(usize, InstanceVar)> = Vec::new();
        for (_, slot, target) in &bindings {
            match target {
                VarHome::Slot(target) => self.roots.alias_slot(inner, *slot, *target),
                VarHome::Instance(var) => exposed.push((*slot, (**var).clone())),
            }
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
        // **Replaced, not extended.** This activation inherited the caller's
        // exposures when it was pushed, and a `PROCEDURE` isolates the pool:
        // a name the caller exposed and this list does not name is an
        // ordinary local here. Measured -- a class method exposing `v` and
        // calling `inner: procedure` with no list, which assigns `v` -- the
        // object variable is unchanged.
        activation.exposed = exposed;
        Ok(())
    }

    /// `EXPOSE`: bind every name it lists to the receiving object's variable
    /// pool for the scope the running method was declared in.
    ///
    /// **The scope, not the receiver's class**, and that is the whole of what
    /// keys a pool. With `sub subclass sup`, a class method on each exposing
    /// `v`, and both sent to `.sub`, the two writes stand at once -- one
    /// object holding one name at two values.
    /// `corpus/lang/expose_two_scopes.rex` is that program, run against the
    /// oracle on both engines; `rexx-core`'s `scope_pools.rs` holds the same
    /// property against the storage directly, and
    /// `a_pool_entry_belongs_to_one_scope_and_not_to_another` holds it against
    /// this function.
    ///
    /// **The binding is per slot and lasts the activation**, so every later
    /// route to the name -- a plan-resolved read, an `INTERPRET` fragment, a
    /// `VALUE('V')` call, a `DROP` -- reaches the pool without knowing this
    /// ran. `Interp::variable` and its two siblings are where that redirect is
    /// applied.
    ///
    /// **Placement is the parser's rule and is not restated here.** An
    /// `EXPOSE` that is not a method body's first instruction is 99.907 at
    /// translation, measured, so what can reach this function is an `EXPOSE`
    /// first in a body -- and a body that is not a method's, which is 98.992.
    pub(crate) fn exec_expose(
        &mut self,
        code: &Code<'_>,
        variables: &[VariableRef],
    ) -> Result<(), Failure> {
        let Some(identity) = self.activation().method_identity.as_ref() else {
            return Err(Raised::expose_outside_method().into());
        };
        let scope = identity.scope;
        let receiver = identity.receiver;
        let owner = self.pool_owner(receiver)?;
        for variable in variables {
            match variable {
                VariableRef::Direct(id) => {
                    let name = code.symbols.name(*id).as_bytes().into();
                    self.bind_exposed(owner, scope, name)?;
                }
                // **The selector's own name is bound before its value is
                // read**, and the order is observable rather than tidy.
                // Measured: with a class-scope `LISTER` holding `'BETA'` and a
                // class-scope `BETA` holding `'beta-value'`, `expose (lister)`
                // in a third method reads `[BETA][beta-value]` -- so `LISTER`
                // was read out of the object's pool and `BETA` was exposed
                // from it. Reading the selector first, out of the frame, gets
                // `[BETA][BETA]`: the frame's `LISTER` is unset, so its
                // derived name `LISTER` is what spells the list.
                //
                // The selector is exposed itself as well as the words it
                // spells, which is the same plurality `PROCEDURE EXPOSE` has
                // and is what the reads above rest on.
                VariableRef::Indirect(id) => {
                    let name = code.symbols.name(*id).as_bytes().into();
                    self.bind_exposed(owner, scope, name)?;
                    let (value, _novalue) = self.read(code, *id);
                    let text = self.to_text(value).into_owned();
                    for word in split_indirect_words(&text) {
                        let word = validate_indirect_word(word)?;
                        self.bind_exposed(owner, scope, word.into())?;
                    }
                }
            }
        }
        Ok(())
    }

    /// Binds one name to `scope`'s pool on `owner`, for the rest of this
    /// activation.
    fn bind_exposed(
        &mut self,
        owner: ObjRef,
        scope: ObjRef,
        name: Box<[u8]>,
    ) -> Result<(), Failure> {
        // A whole stem is one value in one pool entry, so it binds like any
        // other name; a single tail is aliasing *inside* a stem object, which
        // this crate has no representation for. Measured on the oracle,
        // `expose a.1` in one class method assigning `a.1` and `a.2` and the
        // same in another reading them back: `[tail-one][A.2]`, so tail 1 is
        // shared and tail 2 is the method's own local. Exposing the whole stem
        // instead would be a silent wrong answer.
        if shape_of(&name) == NameShape::Compound {
            return Err(Loud::compound_expose("EXPOSE", &name).into());
        }
        let slot = self.slot_of(&name);
        let var = InstanceVar { owner, scope, name };
        let activation = self.activation_mut();
        // Replaced rather than appended: `expose v v` is legal and rc 0 on the
        // oracle, and two entries for one slot would leave every later read
        // deciding between them by list order.
        match activation.exposed.iter_mut().find(|(at, _)| *at == slot) {
            Some(bound) => bound.1 = var,
            None => activation.exposed.push((slot, var)),
        }
        Ok(())
    }

    /// The object whose [`rexx_core::ScopePools`] a send to `receiver` binds
    /// into.
    ///
    /// **A class object gets one made for it.** `RexxClass` is an ordinary
    /// object in the C++ and carries `objectVariables` like any other, which
    /// is why `::method m class` may `EXPOSE` at all; here a class identity
    /// names no arena slot, so the pools live in an arena object created on
    /// first use and rooted for as long as the class is -- which is for ever
    /// in this phase.
    ///
    /// **Any other receiver is refused rather than given one**, and the reason
    /// is the root rather than the storage: an instance's pools live in its
    /// own body and are reached by tracing it, so they are safe exactly while
    /// something roots the instance -- and the receiver of a running send is
    /// rooted here only by the `SELF` slot, which the body may assign over.
    /// The task that creates instances (`~new`) is the one that can settle
    /// that, and no send in this phase reaches a `::METHOD` body with a
    /// non-class receiver.
    fn pool_owner(&mut self, receiver: ObjRef) -> Result<ObjRef, Failure> {
        let Some(class) = receiver.class_id() else {
            return Err(Loud::expose_receiver().into());
        };
        if let Some(owner) = self.class_variables.get(&receiver) {
            return Ok(*owner);
        }
        let owner = self.alloc_with(BehaviourId::OBJECT, Body::Instance(ScopePools::new()));
        // Rooted before anything else can allocate, the rule `.environment`
        // and `.local` are created under. The key is per class and starts with
        // a period, so it can collide neither with another class's nor with a
        // Rexx variable name.
        self.roots
            .add_global(&format!(".class-variables {class}"), owner);
        self.class_variables.insert(receiver, owner);
        Ok(owner)
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
    /// `USE LOCAL` is never legal here -- no entry this crate can construct
    /// is a method invocation -- so implementing it means implementing which
    /// of its two refusals applies. Measured on the oracle, in a clean
    /// directory:
    ///
    /// ```text
    /// as the program's own first instruction    98.993, rc 158
    /// as a ::ROUTINE's own first instruction    98.993, rc 158
    /// anywhere else                             99.910, rc 157
    /// ```
    ///
    /// 98.993 is "may only be used from method invocations" and 99.910 is
    /// "must be the first instruction executed after a method invocation".
    ///
    /// **The 98.993 rows reach this function and the 99.910 arm does
    /// not.** `rexx-parse` already enforces the placement rule
    /// at parse time (`instruction.rs`'s own `use_local`, error 99.910, and
    /// 99.915 for a fragment), so every shape that would take the second arm
    /// fails before execution begins: the second instruction of a program,
    /// after a label on its own line, after a label on the same line, after a
    /// `PROCEDURE`, inside a `DO` block, inside an `IF`, and inside an
    /// `INTERPRET` in two positions were tried, and every one was
    /// intercepted. Those cases already answer the oracle's own number; what
    /// they do not answer byte for byte is the clause echo, which is the
    /// standing parse-error limitation `execute` documents (`lib.rs`).
    ///
    /// The arm is kept rather than collapsed, on the same reasoning
    /// `Loud::missing_body` states for its own unreached arm: a rule the
    /// parser happens to enforce first is not a guarantee this function can
    /// rely on, and answering 98.993 unconditionally would be a silent wrong
    /// answer the day that check moves. It carries no test, because a test
    /// for it would necessarily pass through the parse-time path instead and
    /// so could not fail if this arm were wrong.
    ///
    /// **The question the arm below asks is "is this a method invocation",
    /// and not "was this entered by a call".** The two answers differ on a
    /// `::ROUTINE`, which is entered by a call and is not a method
    /// invocation: measured, `use local` first in one, reached by `CALL` and
    /// as a function, is 98.993 both ways -- the top-level answer, not the
    /// other one.
    fn exec_use(
        &mut self,
        code: &Code<'_>,
        use_: &Use,
        first_instruction: bool,
    ) -> Result<(), Failure> {
        match use_ {
            Use::Local { .. } => {
                // 98.993 is "this is not a method invocation" and 99.910 is
                // "it is one, but this is not its first instruction", so
                // what decides between them is the entry kind rather than
                // whether there was a call. Measured on the oracle: `use
                // local` first in a `::ROUTINE` is 98.993 at rc 158, the
                // same answer the top-level shape gets.
                let method_invocation = match self.activation().entry {
                    Entry::TopLevel | Entry::InternalCall | Entry::Routine => false,
                    Entry::Method => true,
                };
                if first_instruction && method_invocation {
                    // The one shape the oracle **runs**: measured, `use
                    // local` as a `::METHOD`'s first instruction is rc 0.
                    // What it does is bind every name in its list as a
                    // local, which is `EXPOSE`'s own machinery seen from the
                    // other side, so it is loud until that lands rather than
                    // answering a condition the oracle does not raise.
                    Err(Loud::use_local_in_a_method().into())
                } else if first_instruction {
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
            match slot {
                VarHome::Slot(slot) => self.roots.alias_slot(frame, index, slot),
                // The same binding `EXPOSE` makes, on this activation's own
                // slot: the target names the caller's object variable rather
                // than any frame storage, so there is nothing to alias to.
                VarHome::Instance(var) => {
                    let activation = self.activation_mut();
                    match activation.exposed.iter_mut().find(|(at, _)| *at == index) {
                        Some(bound) => bound.1 = *var,
                        None => activation.exposed.push((index, *var)),
                    }
                }
            }
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
                // `results` and not `intermediates`, though the pair below
                // needs both: `results` is the weaker gate, true wherever
                // `intermediates` is, so this renders for either line and
                // drops neither.
                let rendered = self.result_text(value);
                if let Some(rendered) = &rendered {
                    self.trace_result(indent, rendered);
                }
                self.assign_by_name(&name, value);
                if let Some(rendered) = &rendered {
                    self.trace_assignment(indent, &name, rendered);
                }
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
        match self.variable(frame, index) {
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
    fn use_target_name(&self, code: &Code<'_>, target: &UseTarget) -> Result<Vec<u8>, Failure> {
        match &target.target.kind {
            ExprKind::Variable(id) | ExprKind::Stem(id) | ExprKind::Compound(id) => {
                Ok(code.symbols.name(*id).as_bytes().to_vec())
            }
            other => Err(Loud::expression(other).into()),
        }
    }

    /// Everything one `SAY` does once its expression has been evaluated:
    /// `>>>`, then the line itself.
    ///
    /// `None` is the bare `SAY`, which is a blank line **and** a traced null
    /// string rather than a skipped clause (`RexxInstructionExpression::
    /// evaluateStringExpression`'s own `else` arm: `traceResult(GlobalNames::
    /// NULLSTRING)`).
    ///
    /// **The one implementation both engines enter**: `step`'s own `Say` arm
    /// evaluates and calls this, and `crate::ir::Op::Say` does the same with a
    /// register's value, so the trace line and the output cannot come apart
    /// between them.
    ///
    /// **`inline`, and it is a measurement rather than a habit** --
    /// [`Interp::assign_evaluated`]'s own doc comment carries the numbers, since
    /// the two annotations were measured together.
    ///
    /// [`Interp::assign_evaluated`]: Interp::assign_evaluated
    #[inline]
    pub(crate) fn say_evaluated(&mut self, value: Option<ObjRef>) {
        let line = match value {
            Some(value) => {
                self.roots.push_temp(value);
                self.to_text(value).to_vec()
            }
            None => Vec::new(),
        };
        self.trace_result(self.clause_state.current_value_indent, &line);
        self.out.extend_from_slice(&line);
        self.out.push(b'\n');
    }

    /// Everything a `RETURN` or an `EXIT` does once its expression has been
    /// evaluated: its `>>>`, and the `Flow` that leaves the activation.
    ///
    /// **The one implementation both engines enter**: `step`'s own `Return`
    /// and `Exit` arms evaluate and call this, and `crate::ir::Op::Return`
    /// does the same with a register's value.
    ///
    /// `None` is the bare form, which traces **no line at all** -- unlike a
    /// bare `SAY`, which traces the null string. The oracle's
    /// `RexxInstructionExit::execute` (`ExitInstruction.cpp`) evaluates
    /// through `RexxInstructionExpression::evaluateExpression`
    /// (`RexxInstruction.cpp:223`-`235`, read directly), whose own
    /// `traceResult` runs only inside the `expression != OREF_NULL` arm; a
    /// bare `RETURN` leaves `RESULT` unset where `RETURN ''` sets it, which is
    /// the same distinction in the caller.
    ///
    /// The value's `>>>` fires **here**, at this clause's own indent, and a
    /// caller reading `RESULT` traces a *second* one at its own -- measured,
    /// `return 9` from a routine called at top level prints `>>>     "9"` then
    /// `>>>   "9"`, two lines for one value at two indents. `exec_call` owns
    /// the second; this owns the first. Measured for `EXIT` on a three-line
    /// program with no condition and no call in it -- `trace r` / `say 'a'` /
    /// `exit 0` traces `>>>   "0"` after the `exit 0` echo -- so the line
    /// belongs to the instruction and not to anything around it.
    ///
    /// **The rooting here is one clause's, which is shorter than a value that
    /// ends the program needs**, and that is true of either keyword: a
    /// top-level `RETURN` exits with its value exactly as an `EXIT` does
    /// (measured, `return 5` as a whole program is rc 5), and
    /// [`Interp::apply_flow`] takes the surviving root on its `Flow::Return`
    /// arm and its `Flow::Exit` arm alike. The temp pushed here is rooted like
    /// every other `eval` result and the clause's own frame is popped before
    /// the `Flow` has reached `run_activation`, so from there through the
    /// activation teardown to `execute`'s `exit_code_for` call nothing on the
    /// temps stack names it; `root_exit_value` (`lib.rs`) is what does, and
    /// its own doc has the measurement that says the window is real rather
    /// than theoretical. The compiled engine's own register is a second root
    /// for the same value while its region runs, so this push is redundant
    /// there and harmless.
    pub(crate) fn returned_value(&mut self, value: Option<ObjRef>, keyword: ReturnKeyword) -> Flow {
        if let Some(value) = value {
            self.roots.push_temp(value);
            if let Some(rendered) = self.result_text(value) {
                self.trace_result(self.clause_state.current_value_indent, &rendered);
            }
        }
        match keyword {
            ReturnKeyword::Return => Flow::Return(value),
            ReturnKeyword::Exit => Flow::Exit(value),
        }
    }

    /// Everything a `PUSH` or a `QUEUE` does once its expression has been
    /// evaluated: the `>>>` line, and the line itself onto one end of the
    /// queue.
    ///
    /// **The one implementation both engines enter**, `step`'s own arm and
    /// `crate::ir::Op::Queue` alike.
    ///
    /// **`SAY`'s tail with a different sink**, and that is the oracle's own
    /// shape rather than a convenience here: `RexxInstructionQueue::execute`
    /// shares `SAY`'s `RexxInstructionExpression::evaluateStringExpression`
    /// (`QueueInstruction.cpp:69`), so the value is rendered to string form
    /// and traced exactly as [`Interp::say_evaluated`] renders and traces it,
    /// and `None` queues a null string traced as one rather than being a
    /// skipped clause. Reading a line back is `Interp::pull_line`'s
    /// (`input.rs`), not this function's.
    pub(crate) fn queue_evaluated(&mut self, value: Option<ObjRef>, keyword: QueueKeyword) {
        let line = match value {
            Some(value) => {
                self.roots.push_temp(value);
                self.to_text(value).to_vec()
            }
            None => Vec::new(),
        };
        self.trace_result(self.clause_state.current_value_indent, &line);
        match keyword {
            QueueKeyword::Push => self.queue.push(line),
            QueueKeyword::Queue => self.queue.queue(line),
        }
    }

    /// Everything one assignment does once its value has been evaluated:
    /// `>>>`, then the write and the lines the write itself produces.
    ///
    /// **The one implementation both engines enter**, `step`'s own
    /// `Assignment` arm and `crate::ir::Op::Store` alike -- which is what
    /// keeps a stem target, a compound tail and the `>=>` line to one copy.
    /// The target dispatch itself is [`Interp::assign_expr_target`], shared
    /// with `PARSE`; its own doc comment carries which shapes `addVariable`
    /// can build, and why its fourth arm is loud and is reachable from the
    /// other caller but not from this one.
    ///
    /// **`inline` rather than `inline(always)`, and the difference was
    /// measured on both.** With `perf stat -e instructions:u`, base against
    /// head interleaved in one sitting, this annotation and
    /// [`Interp::say_evaluated`]'s together are worth 722,000,000 user
    /// instructions on the compiled stream's arm of
    /// `bench-programs/varlookup.rex` -- 78.5657 against 77.8437 billion, 19
    /// per body clause -- and read exactly zero on the tree-walker's arm and on
    /// every cell of `bench-programs/emptyloop.rex`. `inline(always)` was
    /// measured too and is worse overall: it recovers a further 76,000,000 here
    /// and costs `emptyloop` 550,000,000 on **both** arms, a program whose loop
    /// body enters neither function.
    ///
    /// [`Interp::say_evaluated`]: Interp::say_evaluated
    ///
    /// `at` is forwarded to [`Interp::assign_expr_target`] unchanged and is
    /// that function's parameter rather than this one's; `None` is what a
    /// caller with no earlier resolution passes.
    #[inline]
    pub(crate) fn assign_evaluated(
        &mut self,
        code: &Code<'_>,
        target: &Expr,
        value: ObjRef,
        at: Option<usize>,
    ) -> Result<(), Failure> {
        self.roots.push_temp(value);
        // `>>>` fires before the assignment itself
        // (`RexxInstructionAssignment::execute`: evaluate, trace, *then*
        // assign), which matters only in that the traced value can never be
        // affected by the write it precedes.
        // Reads `current_value_indent` rather than recomputing
        // `static_indent(index)` independently -- the clause unit already
        // computed exactly this value (`indent_offset` included, F-EX1's own
        // correction to F3) for this same instruction right before the clause
        // ran, and a second computation of the identical quantity is how the
        // two drift, which is exactly what happened here before this fix: this
        // site's own copy never learned about the offset when the field was
        // added.
        let indent = self.clause_state.current_value_indent;
        // One render for both lines, and `results` is the gate because it is
        // the weaker of the two: `>>>` is gated on `results` and `>=>` on
        // `intermediates`, and `results` is true wherever `intermediates` is.
        // Guarding on `intermediates` instead would drop the `>>>` line under
        // `TRACE R`.
        let rendered = self.result_text(value);
        if let Some(rendered) = &rendered {
            // **The entry gate, not just the setting in force.** An assignment
            // whose expression turned tracing on owes no `>>>`, because the
            // oracle chose the path without one before it evaluated. See
            // `ClauseState::instructions_traced_at_entry`, which carries the
            // measurement.
            if self.clause_state.instructions_traced_at_entry {
                self.trace_result(indent, rendered);
            }
        }
        self.assign_expr_target(code, target, value, rendered.as_deref(), indent, at)
    }

    /// Writes `value` through one assignment *target expression*, and traces
    /// the write.
    ///
    /// `rendered` is `value`'s own text, supplied rather than recomputed here:
    /// every caller already has it, and rendering a number twice is how a
    /// second rendering under a different `DIGITS` would get a chance to
    /// disagree with the first (D15).
    ///
    /// **The two callers do not agree about whether the `other` arm can be
    /// reached, and that was measured rather than reasoned from one of them.**
    /// An `Assignment`'s target is whatever `addVariable` builds, which is
    /// only the three shapes below (`ast.rs`'s own doc comment on
    /// `Assignment`). A `PARSE` target is `parseVariableOrMessageTerm`, so the
    /// grammar admits a message term there: measured, `parse value 'a b' with
    /// q~x r` parses, and the oracle answers `Error 97.1` (`Object "Q" does
    /// not understand message "X="`) where this crate reports a `Phase 5` gap.
    /// So the arm is live for one caller and unreachable for the other, and it
    /// is reported through `Loud::expression` rather than `unreachable!` for
    /// both: a guarantee the grammar makes is not one the type system
    /// enforces, and failing loudly beats trusting it.
    ///
    /// **Shared by `Assignment` and `PARSE`**, which is what makes a
    /// compound `PARSE` target behave exactly as `a.i = value` does --
    /// measured, `ii = 3; parse value 'one two' with aa.ii cc.` traces
    /// `>C> AA.II => "AA.3"` then `>=> AA.II <= "one"` and stores the value
    /// under the resolved tail.
    /// `rendered` is `None` when no `>=>` line would print it, which every
    /// arm below forwards unchanged to [`Interp::trace_assignment`].
    ///
    /// **An `Option` rather than an empty slice**, and the reason is the
    /// defect it closes: `rendered` is a full copy of the assigned value, so
    /// a caller that produced it unconditionally copied a value of any size
    /// on a path that discards it -- measured, `x = copies('a',400000000)`
    /// aborts the process at the project's own `ulimit -v 1048576` where the
    /// oracle answers. The type is what makes the caller's guard visible
    /// here: a `&[u8]` that is empty because tracing is off and a `&[u8]`
    /// that is empty because the value is the null string are the same value,
    /// and only one of them may be printed.
    ///
    /// **`at` is the slot a compiler already resolved a simple-variable target
    /// to, and it is a parameter of *this* function rather than a store of its
    /// own on purpose.** `crate::ir::Op::Store` carries one and the tree-walker
    /// passes `None`, so both engines still arrive here and a stem target, a
    /// compound tail and the `>=>` line stay one implementation -- a second
    /// store path executed in the driver would be the two-implementations
    /// defect the dual-engine gate exists to catch, however much faster it
    /// measured. **Only the first arm below reads it**, and the other two find
    /// their own slot on the entry `Code::compound` holds, which is the same
    /// answer on both engines where an op's is the compiled one's alone.
    ///
    /// **A simple variable and a bare stem write the same slot; a compound
    /// writes a different one; and the reason `at` serves only the first is
    /// different again for each.** A simple variable and a bare stem both
    /// write the slot their own symbol is bound to, and for a bare stem `at`
    /// would therefore be the *right* number -- it is not supplied because
    /// supplying it was tried at `bind_control` and measured to cost more than
    /// it saves ([`control_slot`]'s own doc has the figures), and because
    /// `Plan::bind` records that same slot on the entry, which reaches both
    /// engines where an op reaches one. A compound writes the *stem's* slot,
    /// a different name on a different slot that the entry carries separately
    /// and `Code::stem` hands over, so `at` would be the wrong number there.
    /// Both of those arms assert that no caller passed one.
    ///
    /// **`None` is always correct.** The slot is then resolved by the write
    /// itself exactly as it was before any caller could supply one, which is
    /// what `Interp::slot_of` does; a supplied slot is the same resolution
    /// made earlier, from the plan's own map, and never a different answer.
    pub(crate) fn assign_expr_target(
        &mut self,
        code: &Code<'_>,
        target: &Expr,
        value: ObjRef,
        rendered: Option<&[u8]>,
        indent: usize,
        at: Option<usize>,
    ) -> Result<(), Failure> {
        match &target.kind {
            ExprKind::Variable(id) => {
                let name = code.symbols.name(*id).as_bytes();
                let slot = match at {
                    Some(slot) => slot,
                    None => self.slot_of(name),
                };
                let frame = self.activation().frame;
                self.set_variable(frame, slot, value);
                if let Some(rendered) = rendered {
                    self.trace_assignment(indent, name, rendered);
                }
            }
            // `stem. = expr`: replace-and-rebind (D15a), through the
            // library `stem_assign` already builds -- this arm is the
            // dispatch, not new stem logic.
            //
            // `Code::compound` rather than `Code::stem`, though the two hold
            // the same slot for a stem-shaped name: only the slot is wanted
            // here, and `Code::stem`'s no-plan fallback splits the spelling to
            // produce a name this arm already has, allocating a tail vector to
            // throw away on every `INTERPRET`ed stem write.
            ExprKind::Stem(id) => {
                // The tripwire `crate::ir::drive`'s `Op::Store` arm carries
                // for the compiled side, here where **both** engines pass:
                // a slot handed to this arm was resolved against the symbol's
                // own id and is silently shadowed below, so a caller that
                // started supplying one would write through the entry's slot
                // and never learn that its own was ignored.
                debug_assert!(
                    at.is_none(),
                    "a stem write was handed a slot, and the slot it writes comes from the entry"
                );
                let name = code.symbols.name(*id).as_bytes();
                let at = code.compound(*id).and_then(|entry| entry.stem_at);
                self.stem_assign_at(name, at, value);
                if let Some(rendered) = rendered {
                    self.trace_assignment(indent, name, rendered);
                }
            }
            // `a.b = expr`: resolve the tail key the same way reading
            // `a.b` would (`eval_node`'s own `Compound` arm), then
            // mutate that one tail in place through `stem_set`.
            //
            // `>C>` before `>=>` (`RexxActivation.cpp:4791`-`4802`'s
            // own order for a *read*; measured that a *write* through
            // `ExpressionCompoundVariable::assign` announces the same
            // resolved name first too): the tag is the compound's own
            // **source spelling** (`a.i` stays `A.I` regardless of `i`'s
            // value), and the resolved name is `stem_name` (the read site's
            // own, matching `stem_set`'s own convention) concatenated with
            // `key`.
            ExprKind::Compound(id) => {
                // `read_symbol`'s own compound tripwire, on the writing
                // side. A compound-shaped name **can** reach
                // `Plan::by_symbol`: `Plan::bind` puts every name it binds
                // whole there, and `note_loop` and `note_parse` both call it
                // with spellings that may be compound-shaped. So a caller
                // reaching for a slot by this symbol's id can find one, and it
                // is not the slot this arm writes.
                debug_assert!(
                    at.is_none(),
                    "a compound write was handed a slot, and the slot it writes is the stem's"
                );
                // **Borrowed, not copied.** `Code::stem` and `SymbolTable::
                // name` both answer with the lifetime of the `Code`, which is
                // the program rather than this `Interp`, so neither needs an
                // owned copy to survive the `&mut self` calls below. The
                // `Variable` arm above has always passed its name borrowed;
                // this arm copied both, and measured with `heaptrack` on
                // `samples/rexxcps.rex` that cost two allocations per compound
                // write.
                let tag = code.symbols.name(*id).as_bytes();
                let (stem_name, stem_at) = code.stem(*id);
                let mut key = self.take_key_buffer();
                self.tail_key_into(code, *id, &mut key);
                self.stem_set_at(stem_name, stem_at, &key, value);
                // **The resolved name is built only when a line will print
                // it.** `trace_compound_name` returns at once unless
                // intermediates are on, so joining the stem to the tail key
                // ahead of that check allocated a name to discard. This is the
                // rule `rendered` below already follows -- `trace.rs`'s own
                // doc comment gives the reasoning for the value half, and the
                // name half is the same argument.
                if self.tracing_intermediates() {
                    let mut resolved = stem_name.to_vec();
                    resolved.extend_from_slice(&key);
                    self.trace_compound_name(indent, tag, &resolved);
                }
                if let Some(rendered) = rendered {
                    self.trace_assignment(indent, tag, rendered);
                }
                self.give_key_buffer(key);
            }
            other => return Err(Loud::expression(other).into()),
        }
        Ok(())
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
                self.set_variable(frame, slot, value);
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
    /// two `step` arms and `Interp::invoke_call` (`CALL`, and `ExprKind::
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
        let value = self.counted_text(line);
        // Not through `assign_by_name`: that reads the name's shape and then
        // hashes it, and `SIGL` is a simple name whose slot the plan already
        // holds. The fallback covers a plan with no name map at all.
        let slot = match self.activation().plan.sigl_slot {
            Some(slot) => slot,
            None => self.slot_of(b"SIGL"),
        };
        let frame = self.activation().frame;
        self.set_variable(frame, slot, value);
    }

    /// The slot `RESULT` lives in, from the plan when it has one.
    fn reserved_result_slot(&mut self) -> usize {
        match self.activation().plan.result_slot {
            Some(slot) => slot,
            None => self.slot_of(b"RESULT"),
        }
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
    pub(crate) fn exec_condition_trap(
        &mut self,
        trap: &ConditionTrap,
        call: bool,
    ) -> Result<Flow, Failure> {
        match &trap.label {
            Some(label) => {
                let entry = Trap {
                    call,
                    label: std::rc::Rc::from(label.as_ref()),
                    delayed: false,
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
    /// `set_sigl`), which a borrow of the running activation held across would
    /// make the `E0502` `run_activation`'s own doc comment writes out.
    pub(crate) fn trap_for(&self, condition: &[u8]) -> Option<Trap> {
        let traps = &self.activation().traps;
        traps
            .get(condition)
            .or_else(|| traps.get(b"ANY".as_slice()))
            // A trap held while its own `CALL ON` handler runs does not fire
            // -- see `Trap::delayed`, which is what a handler that calls
            // something raising the same condition depends on.
            .filter(|trap| !trap.delayed)
            .cloned()
    }

    /// Turns an uninitialised variable read into a `NOVALUE` condition --
    /// but only when this activation has a `NOVALUE` trap that could take
    /// it.
    ///
    /// **Inherited item I13, and `Novalue::Unset`'s first reader.** D16 put
    /// the flag on the read path from the start rather than leave a raise to
    /// be retrofitted into it, and this is the retrofit that did not have to
    /// happen.
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
    /// **The gate is `inline(always)` and the raise is `cold`, which is a
    /// measurement rather than a decoration.** An initialised read -- every
    /// read in a working program -- reaches only the first comparison, and
    /// with the whole function out of line it paid a call and a `Result`
    /// return to learn that. A plain `#[inline]` did not move it: the tail
    /// looks up a trap and builds a condition, which is enough to put the
    /// inliner off. Splitting says which half is hot instead of hinting.
    /// Measured with the marginal method -- a body run at N and 2N
    /// iterations, differenced -- `z = a` costs 449 user instructions per
    /// execution undivided and 431 split.
    #[inline(always)]
    pub(crate) fn novalue_check(&self, novalue: Novalue) -> Result<(), Failure> {
        if novalue == Novalue::Set {
            return Ok(());
        }
        self.novalue_raised()
    }

    /// [`Interp::novalue_check`]'s uninitialised half: whether the setting in
    /// force turns this read into a raised `NOVALUE` rather than the derived
    /// name `read_at` already produced.
    #[cold]
    #[inline(never)]
    fn novalue_raised(&self) -> Result<(), Failure> {
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
    /// does catch reaches it through `Interp::pending_traps` instead, without
    /// ever becoming a failure.
    ///
    /// Returns a `Flow` rather than a bare target so that
    /// `run_activation`'s existing `match` does the transfer: `Flow::Signal`
    /// is exactly "set this activation's `pc`", which is what a trap does.
    pub(crate) fn offer_to_trap(
        &mut self,
        code: &Code<'_>,
        failure: Failure,
    ) -> Result<Flow, Failure> {
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
                return Err(Failure::Raised(raised));
            }
            // The outermost activation is the only one allowed to look.
            Search::Top if self.activation_depth() > 1 => return Err(failure),
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
        // What `CONDITION()` reports for the rest of this activation.
        // Written here and not onto `active_condition` below, because the
        // two have different lifetimes: this one dies with the activation
        // (`TrappedCondition`), while `active_condition` is the interpreter's
        // one slot for `RAISE PROPAGATE`.
        self.activation_mut().condition = Some(TrappedCondition {
            name: raised.condition.as_bytes().into(),
            // Only a `SYNTAX` condition has a `CODE` item at all
            // (`Activity::createExceptionObject` is the one place it is put
            // in), so the name and not `reportable()` is the test: measured,
            // a trapped `HALT` reports `E` as the null string, and `HALT` is
            // the one non-`SYNTAX` condition this crate numbers.
            code_sub: (raised.condition == "SYNTAX").then_some(raised.sub),
            call: false,
            description: raised.description.clone(),
        });
        // What a later `RAISE PROPAGATE` re-raises. See `exec_raise_
        // propagate` for what is and is not measured about it.
        self.active_condition = Some(ActiveCondition {
            raised: *raised,
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
        if let Some(exit) = self.deliver_pending_traps(code)? {
            return Ok(Flow::Exit(exit.value()));
        }
        Ok(Flow::Signal(target))
    }

    /// Runs every handler this clause boundary owes, in the order the
    /// conditions were queued, and stops early only if one of them ends the
    /// program.
    ///
    /// **Bounded to what was already queued when the boundary began**, which
    /// is what defers a handler's own requeue to the next clause --
    /// `Interp::pending_traps` carries the transcripts for both halves.
    /// Entries belonging to another activation are stepped over rather than
    /// blocking the ones this activation owes; `PendingTrap::activation` has
    /// why that identity is the right key. Entries queued in a different
    /// `INTERPRET` fragment are stepped over the same way, for the reason
    /// `PendingTrap::fragment_depth` states.
    ///
    /// **The wait is the measured part.** `zres = one(1)`, where `one`
    /// raises a `CALL ON`-trapped condition and the handler assigns `zres`
    /// itself, prints the *handler's* value -- so the assignment had already
    /// stored the routine's result before the handler ran. `say 'a' one(1)
    /// two(2)` in the same shape prints the whole line, `two`'s value
    /// included, and only then the handler. Neither is what a trap that
    /// fired at the raise would print.
    ///
    /// The trap is **held for the handler's duration and released
    /// afterwards**, unlike a `SIGNAL ON` trap, which is removed and stays
    /// removed. Measured: a handler that itself calls a routine raising the
    /// same condition does not re-enter, and the program then carries on
    /// normally rather than running the handler a second time later.
    ///
    /// "Held" is `Trap::delayed` since 4c Task 10 and was a `remove` with a
    /// re-insert before it. The two agree on everything but
    /// `CONDITION('S')`, which is why the change was needed and why nothing
    /// else in this function's behaviour moved with it.
    pub(crate) fn deliver_pending_traps(
        &mut self,
        code: &Code<'_>,
    ) -> Result<Option<HandlerExit>, Failure> {
        // Snapshotted rather than re-read, so that anything a handler queues
        // lands beyond the prefix this boundary is answering for.
        let mut owed = self.pending_traps.len();
        while owed > 0 {
            // Only the activation whose trap table matched delivers, and only
            // once it is running again -- `PendingTrap::activation`'s own doc
            // comment has the three transcripts this identity check answers,
            // including the two a stack depth got wrong.
            let here = self.activation().id;
            // Both keys, and they answer different questions: the identity
            // says which activation's trap table matched, the depth says which
            // `INTERPRET` fragment's queue the condition is sitting in. A
            // fragment is an activation in the oracle and is not one here, so
            // the second is what the first cannot see.
            let depth = self.fragment_depth;
            let Some(at) =
                self.pending_traps.iter().take(owed).position(|pending| {
                    pending.activation == here && pending.fragment_depth == depth
                })
            else {
                return Ok(None);
            };
            let pending = self
                .pending_traps
                .remove(at)
                .expect("position answered an index inside the queue");
            owed -= 1;
            if let Some(exit) = self.deliver_one_pending_trap(code, pending)? {
                return Ok(Some(exit));
            }
        }
        Ok(None)
    }

    /// One queued condition's handler, run at the boundary that owes it.
    ///
    /// `Ok(None)` means the boundary may go on to the next entry: either the
    /// handler returned, or this condition turned out to have no `CALL ON`
    /// trap to run and is discarded.
    fn deliver_one_pending_trap(
        &mut self,
        code: &Code<'_>,
        pending: PendingTrap,
    ) -> Result<Option<HandlerExit>, Failure> {
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
        // **Delayed, not removed** (`Trap::delayed`). The two are the same
        // to every lookup that decides whether to trap, and different to
        // `CONDITION('S')` alone, which reports `DELAY` here and `OFF` for a
        // trap that is absent. Nothing else moves: an earlier version of
        // this comment claimed the flag also protects a handler's own `CALL
        // OFF`, and that is false -- the handler's table is a copy, so its
        // `CALL OFF` never reached this one to be undone.
        if let Some(trap) = self.activation_mut().traps.get_mut(&key) {
            trap.delayed = true;
        }
        // What `CONDITION()` answers inside the handler. Set on *this*
        // activation and restored afterwards, because the handler inherits
        // its copy at call time and the caller must be left as it was --
        // measured, `condition()` back in the caller after a `CALL ON`
        // handler returns is the null string.
        let enclosing_condition = self.activation_mut().condition.replace(TrappedCondition {
            name: key.clone(),
            // No condition reaching here has a `CODE`, and both halves of
            // that are measured. A `CALL ON` trap cannot name `SYNTAX`
            // directly -- `call on syntax` is a 25.1 translation error --
            // and `CALL ON ANY`, the one spelling that could smuggle it in,
            // does not catch a `SYNTAX` condition either: `call on any name
            // uh` with `say 1/0` is the ordinary fatal 42.3 at rc 214 on
            // both interpreters. `TrapHandler::canHandle` is the C++ side of
            // the same rule.
            code_sub: None,
            call: true,
            description: pending.description.clone(),
        });
        let queued_before = self.pending_traps.len();
        // `CallType::Subroutine` because a `CALL ON` handler is a `CALL`, and
        // that reaches `PARSE SOURCE` when the trap's name resolves to a
        // `::ROUTINE` rather than to a label. Measured: a trapped `USER`
        // condition whose handler is a `::routine` running `parse source`
        // answers `SUBROUTINE`, where the same handler written as a label
        // answers whatever the trapping activation answers.
        // **`CallEntry::Trap`, which is the one thing about a handler's own
        // activation that differs from an internal `CALL`'s**: the oracle's
        // `internalCallTrap` passes `OREF_NULL` where `internalCall` passes
        // the caller's receiver, so the handler's calling convention carries
        // none -- `entered_receiver` has the measurement for both.
        let ended = self.resolve_and_run_call(
            code,
            &trap.label,
            true,
            &[],
            CallType::Subroutine,
            CallEntry::Trap,
        );
        // A trap queued by the handler that just ran is not one the
        // interrupted clause owes, and `in_clause`'s tripwire has to be able
        // to tell the two apart -- see the field's own doc comment.
        for pending in self.pending_traps.iter_mut().skip(queued_before) {
            pending.queued_during_delivery = true;
        }
        self.activation_mut().condition = enclosing_condition;
        // `trapUndelay`. The `if let` mirrors the C++ testing the handler
        // for null before enabling it; nothing a Rexx program can do
        // removes the entry between here and the delay above, so the arm is
        // structural rather than a case anything reaches.
        if let Some(trap) = self.activation_mut().traps.get_mut(&key) {
            trap.delayed = false;
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
            // wholesale. So this line changes no output while that holds.
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
    ///
    /// **A second difference from `trap_for`, and it is deliberate: this one
    /// does not filter [`Trap::delayed`].** Matching a delayed handler and
    /// then declining to run it is what the C++ does
    /// (`RexxActivation::raiseCondition` queues without asking;
    /// `processTraps` skips), and `deliver_pending_traps`'s own `trap_for`
    /// is the decline. Measured rather than argued: a `CALL ON` handler that
    /// calls a routine raising the same condition runs once on both
    /// interpreters, byte for byte.
    fn caller_trap_for(&self, condition: &[u8]) -> Option<Trap> {
        let traps = &self.caller_activation()?.traps;
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
    /// **The table is a LABEL activation's, and the oracle does not apply it
    /// across a `::ROUTINE` one.** Measured: `raise user boom` with no tail,
    /// and with `EXIT`, inside a `::ROUTINE` reached by `CALL` runs the
    /// caller's enabled USER trap on the oracle, where the same raise from an
    /// internal label runs no trap on either side. This crate applies the
    /// table's rule to both, so those two cells diverge on stdout at rc 0 --
    /// recorded in `docs/superpowers/plans/phase-4-exclusions.txt` with the
    /// whole matrix and the label control, and owned by no task here.
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
        // Kept, not only traced: a trapping handler reads it back through
        // `CONDITION('D')` -- measured, `raise syntax 40.4 description 'zd'`
        // trapped gives `zd` where the same raise without the clause gives
        // the null string.
        let mut description: Option<Vec<u8>> = None;
        if let Some(expr) = &raise.description {
            let value = self.eval(code, expr)?;
            self.roots.push_temp(value);
            let rendered = self.to_text(value).to_vec();
            self.trace_keyword(indent, "DESCRIPTION", &rendered);
            description = Some(rendered);
        }
        // `ADDITIONAL expr` and `ARRAY (a, b)` produce the identical
        // substitution list -- measured, `additional ('MYROUTINE', 3)` and
        // `array ('MYROUTINE', 3)` give byte-identical reports -- so they
        // share one `Vec` here rather than being kept apart to no end. A
        // single non-array `ADDITIONAL` value is one substitution, also
        // measured: `additional 'JUSTONE'` fills `&1` and leaves `&2` as the
        // literal `&2`.
        let mut additional: Vec<Vec<u8>> = Vec::new();
        if let Some(expr) = &raise.additional {
            let value = self.eval(code, expr)?;
            self.roots.push_temp(value);
            // **A surface that is neither `stringValue()` nor an operator,
            // and only under one condition.** `RaiseInstruction::execute`
            // (`instructions/RaiseInstruction.cpp:270`-`290`) calls
            // `requestArray` on the additional information exactly once, and
            // only inside `if (errorCode->strCompare(SYNTAX))` -- so under
            // `USER` or any other condition the value is never
            // array-converted and this crate's rendering is the oracle's own
            // answer. Measured both ways: `raise syntax 40.1 additional
            // (.array)` is a 98 execution error at rc 158, `additional
            // (.environment)` substitutes `INPUTOUTPUTSTREAM` -- the first
            // entry of the array the directory converts to -- and `raise user
            // zork additional (.array)` under a trap is rc 0 on both sides.
            //
            // A propagate never reaches here: `exec_raise` returns to
            // `exec_raise_propagate` before any option is evaluated, so the
            // `ADDITIONAL` expression is not evaluated at all under one. The
            // divergence that leaves is recorded in `phase-4-exclusions.txt`
            // and predates this refusal.
            if raise.condition.eq_ignore_ascii_case(b"SYNTAX")
                && let Some(kind) = self.operator_operand_gap(value)
            {
                return Err(Loud::object_position("a RAISE ADDITIONAL value", kind).into());
            }
            let rendered = self.to_text(value).to_vec();
            self.trace_keyword(indent, "ADDITIONAL", &rendered);
            additional.push(rendered);
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
                    additional.push(Vec::new());
                    continue;
                };
                let value = self.eval(code, expr)?;
                self.roots.push_temp(value);
                // **No gap check here, and the absence is the decision.**
                // `RaiseInstruction::execute` builds a real `ArrayClass` from
                // these elements (`RaiseInstruction.cpp:217`-`239`) and the
                // `requestArray` below it therefore gets an array already and
                // returns it unchanged -- the elements are never
                // array-converted, only rendered by the substitution
                // machinery. Measured: `array (.array)`, `array
                // (.environment)` and `array (.array, 'b')` are all rc 216
                // and byte-identical here. A check on this arm refused all
                // three.
                let rendered = self.to_text(value).to_vec();
                self.trace_argument(indent, &rendered);
                self.trace_argument(indent, &rendered);
                additional.push(rendered);
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
                    if let Some(rendered) = self.result_text(value) {
                        self.trace_keyword(indent, "RESULT", &rendered);
                    }
                    Some(value)
                }
                None => None,
            },
            None => None,
        };
        let returns = raise.result.as_ref().is_some_and(|result| !result.exit);

        if raise.condition.as_ref() == b"SYNTAX" {
            let mut raised = raise_syntax_condition(rc_text.as_deref().unwrap_or(b""), additional);
            raised.description = description;
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
            // caller's current clause to finish -- `deliver_pending_traps`
            // has the two transcripts that pin the wait.
            Some(trap) if trap.call => {
                self.pending_traps.push_back(PendingTrap {
                    condition: name,
                    rc,
                    description: description.clone(),
                    // The caller's own identity -- this activation is about
                    // to be popped, and `caller_trap_for` above just read
                    // that same activation's table. See the field's own doc
                    // comment for the three transcripts behind it.
                    activation: self
                        .caller_activation()
                        .expect("a raise reaching here has a caller to queue against")
                        .id,
                    // Set by `deliver_pending_traps` if this turns out to have
                    // been queued while a handler was running, which is not
                    // knowable here: this is the raise, not the delivery.
                    queued_during_delivery: false,
                    // Which `INTERPRET` fragment's queue this joins. The
                    // raising activation is about to be popped and the depth
                    // is not its own -- a fragment does not push an activation
                    // here -- so it is read straight off `Interp`, where the
                    // `Interpret` arm maintains it.
                    fragment_depth: self.fragment_depth,
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
                raised.description = description;
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
    /// we answered silence at rc 0. `deliver_pending_traps` clears the field
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
    /// interpreted text being 47.1). Mirrors `Interp::resolve_call`'s
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
    /// `SIGNAL label`, past the point where the label's bytes are known:
    /// resolve, record `SIGL`, and answer the transfer.
    ///
    /// **One implementation, entered from `step` and from `Op::Signal`.**
    pub(crate) fn signal_to_label(&mut self, name: &[u8]) -> Result<Flow, Failure> {
        let target = self.resolve_signal_target(name)?;
        // Set only once the target actually resolves -- an unresolved
        // `SIGNAL` (16.1) ends the program regardless, matching the oracle's
        // own `signalTo`, which a caller only ever invokes with an
        // already-resolved target.
        self.set_sigl(self.clause_state.line());
        Ok(Flow::Signal(target))
    }

    /// `SIGNAL VALUE expr`, past the expression: its `>K>` echo, then the
    /// same search and transfer a written label takes.
    ///
    /// `value` must already be rooted by its caller -- a temp on the
    /// tree-walker, a register on the compiled stream -- because the render
    /// below can collect.
    pub(crate) fn signal_to_value(&mut self, value: ObjRef) -> Result<Flow, Failure> {
        let text = self.to_text(value).to_vec();
        self.trace_keyword(self.clause_state.current_value_indent, "VALUE", &text);
        self.signal_to_label(&text)
    }

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

    /// Resolves `name` to the thing a call of it runs, with no argument
    /// evaluated and nothing entered.
    ///
    /// **The resolution half of a call, and the seam every route goes
    /// through.** `exec_call` (`CALL`), `eval_call` (`ExprKind::Call`'s
    /// expression form, `eval.rs`) and `crate::ir::Op::Call` each ask this
    /// and then hand the answer to [`Interp::invoke_call`], so the four-step
    /// order below is decided in one place -- which is what stops `CALL
    /// length 'abc'` and `say length('abc')` answering differently.
    ///
    /// **It takes no `Code`, and that is the contract rather than an
    /// omission**: the search goes against the running *activation's* body,
    /// which the body a caller happens to be walking is not inside an
    /// `INTERPRET` fragment.
    ///
    /// `search_labels` is false for `CALL "name"` and for `ExprKind::Call`'s
    /// `CallTarget::Literal`, and its own call sites have the measurements.
    ///
    /// **Resolution order is internal label, then builtin, then `::ROUTINE`,
    /// then the external file this crate answers 43.1 in place of**, and the
    /// name is settled against all four *before* an argument is evaluated --
    /// [`Resolved`]'s three variants for the three that run something, and an
    /// immediate raise for the fourth. The order matters both ways round: a
    /// label wins over a builtin of the same name, and a builtin wins over
    /// anything behind it.
    ///
    /// The external file is Phase 7's, and 43.1 is the oracle's own answer for
    /// every program that has no such file beside it: measured in a clean
    /// directory, `call zorkolo` gives 43.1 rc 213 `Could not find routine
    /// "ZORKOLO".`
    pub(crate) fn resolve_call(
        &self,
        name: &[u8],
        search_labels: bool,
    ) -> Result<Resolved, Failure> {
        // **Resolved against the running *activation's* body, not against the
        // body a caller is walking, and the two differ inside an `INTERPRET`
        // fragment.** A fragment's `labels` is always empty -- a label in
        // interpreted text is error 47.1 -- so searching the walked body would
        // make every `CALL` inside a fragment unresolvable. Measured on the
        // oracle: `interpret "call sub"` runs the enclosing program's `sub:`.
        // Found by running the composition rather than by reading the code:
        // the first version of this searched the walked body and passed every
        // test that had no `INTERPRET` in it.
        let program = Rc::clone(&self.activation().program);
        let selector = self.activation().body;
        let Some(activation_body) = body_of(&program, selector) else {
            return Err(Loud::missing_body().into());
        };
        let label = if search_labels {
            activation_body.labels.get(name).copied()
        } else {
            None
        };
        // **The whole resolution happens here, upstream of the argument loop
        // in `invoke_call`**, and the shape is load-bearing rather than tidy.
        // The builtin step needs its arguments already evaluated, so it cannot
        // sit where the raising return sits; putting the lookup between the
        // label miss and that return would have placed it upstream of the
        // evaluation it consumes. Deciding all four outcomes first is what
        // lets one argument loop serve three of them.
        //
        // **The order is measured in both directions.** A label wins over a
        // builtin of the same name; a builtin wins over a `::ROUTINE` of the
        // same name (`call max 1, 9` with a `::routine max` present reports
        // 9, and the routine never runs), so a `::ROUTINE` search in front of
        // the builtin step would silently run the wrong routine. A quoted
        // target is a second order rather than the same one: `call
        // 'ZORKOLO'` skips the internal `zorkolo:` label -- `search_labels`
        // is already false for it -- and still reaches the `::routine`.
        //
        // The routine lookup upcases both sides (`Interp::routines`' own
        // doc), where the builtin step in front of it is case-sensitive:
        // measured, `call 'max' 1, 9` is 43.1 and `call 'MAX' 1, 9` is 9, and
        // that asymmetry is exactly what makes a `::routine 'max'` reachable
        // at all.
        let resolved = match label {
            Some(target) => Resolved::Label(target),
            // **`resolve` rather than `is_builtin`, and it answers the same
            // question.** Every row's name is in scope
            // (`every_implemented_row_names_an_in_scope_builtin`) and a name in
            // scope with no row comes back `Gap`, so `resolve(name).is_some()`
            // and `is_builtin(name)` agree on every name. It costs the same
            // one lookup and keeps which builtin it found.
            None if let Some(target) = builtin::resolve(name) => Resolved::Builtin(target),
            // A builtin Phase 4 excludes outright is still a builtin, so it
            // sits here rather than behind the routine lookup -- see
            // `builtin::is_excluded_builtin`'s own doc for why neither the
            // routine step nor 43.1 is an acceptable answer for one.
            None if builtin::is_excluded_builtin(name) => {
                return Err(Loud::unresolved_call(name).into());
            }
            None => match self.routines.get(&name.to_ascii_uppercase()[..]).copied() {
                Some(installed) => Resolved::Routine(installed),
                // **43.1, not this crate's loud gap**, and the difference is
                // one search: the oracle looks for an external Rexx file
                // named for the target before answering, and this crate does
                // not (Phase 7, `phase-4-exclusions.txt`). Measured in a
                // clean directory with nothing of that name beside the
                // program, the oracle's own answer is exactly this condition
                // -- `call zorkolo` gives 43.1 rc 213 `Could not find routine
                // "ZORKOLO".` -- so answering it here is right for every
                // program with no such file and wrong only for one that has
                // one, where the oracle runs the file at rc 0.
                None => return Err(Raised::routine_not_found(name).into()),
            },
        };
        Ok(resolved)
    }

    /// Takes the shared value buffer, empty and ready to build into.
    pub(crate) fn take_value_buffer(&mut self) -> Vec<Option<ObjRef>> {
        let mut buffer = std::mem::take(&mut self.value_buffer);
        buffer.clear();
        buffer
    }

    /// Hands the value buffer back for the next builtin call.
    pub(crate) fn give_value_buffer(&mut self, buffer: Vec<Option<ObjRef>>) {
        self.value_buffer = buffer;
    }

    /// One builtin call: its arguments evaluated in the caller, then the row
    /// run over them.
    ///
    /// **Evaluates into its own buffer of values**, before an `Argument` is
    /// ever built. A builtin wants the values and nothing else: the
    /// `Reference` half of an `Argument` exists for `USE ARG >`, which no
    /// builtin has, so building a `Vec<Option<Argument>>` here would only be
    /// something to copy into a `Vec<Option<ObjRef>>` before handing it over.
    /// That the shape of this path is worth caring about is a measurement:
    /// with `perf` on `bench-programs/strings.rex`, whose loop makes four
    /// builtin calls, the call path was 14.76% of samples -- more than any
    /// builtin it dispatches.
    ///
    /// The evaluation itself is shared with the label path
    /// (`eval_traced_argument`), so the `>p` reference form still traces its
    /// `>O>` line here exactly as it does for a label call, and `>A>` fires
    /// once per position with the omitted ones included.
    ///
    /// **A value and not an [`Ended`], which is the reason this is reachable
    /// from outside [`Interp::invoke_call`] at all.** A builtin runs no
    /// activation, so it can neither exit nor return nothing: `Ended`'s two
    /// variants are both the same answer for it, and `Result<Ended, Failure>`
    /// is wider than a register pair where `Result<ObjRef, Failure>` is not.
    /// `Interp::eval_call_resolved` wants the value and enters here directly;
    /// `invoke_call` wraps the same call for the callers that hold an `Ended`.
    ///
    /// See `builtin`'s own module doc for what this deliberately does *not*
    /// do that the label path does -- `SIGL`, the depth guard and the
    /// activation level, with a probe for each.
    pub(crate) fn invoke_builtin_call(
        &mut self,
        code: &Code<'_>,
        target: crate::builtin::BuiltinTarget,
        name: &[u8],
        args: &[Option<Expr>],
    ) -> Result<ObjRef, Failure> {
        // **Pushed onto the shared stack one at a time**, rather than into a
        // buffer lent for the whole loop: each argument's evaluation calls
        // back into `&mut self`, so nothing may hold the `Vec` across it, and
        // a run that is only ever appended to needs no borrow between pushes.
        // The compiled path's `Op::PushArg` writes the same stack.
        let mark = self.value_buffer.len();
        for arg in args {
            let value = match arg {
                None => {
                    self.trace_argument(self.clause_state.current_value_indent, b"");
                    None
                }
                Some(expr) if self.leaf_argument(expr) => {
                    Some(self.eval_leaf_argument(code, expr)?)
                }
                Some(expr) => Some(self.eval_traced_argument(code, expr)?.value()),
            };
            self.value_buffer.push(value);
        }
        self.run_over_pushed_args(mark, |interp, values| {
            builtin::run(interp, name, target, values)
        })
    }

    /// Evaluates the arguments of a call already resolved to `resolved` and
    /// runs it, in its own nested activation where it has one.
    ///
    /// **The invocation half, and the counterpart to
    /// [`Interp::resolve_call`].** Every route into a call reaches this:
    /// `exec_call` (`CALL`, which goes on to settle `RESULT` and translate the
    /// outcome into a `Flow`), `eval_call` (`ExprKind::Call`, `eval.rs`, which
    /// never touches `RESULT` and has no `Flow` to report through since `eval`
    /// returns a value rather than a step outcome), and `crate::ir::Op::Call`.
    ///
    /// **Shared rather than duplicated.** Every caller needs the identical
    /// argument evaluation and its `>A>` lines, the identical
    /// `MAX_ACTIVATION_DEPTH` guard and the identical five-piece level
    /// bookkeeping around the nested `run_activation` -- measured to matter
    /// for the expression form too (`trace r` under a flat `zz = f(1) + 1`
    /// echoes `f`'s own clauses at the calling clause's indent plus two, the
    /// same D2r rule `CALL` already carries) -- and a second hand-copied
    /// version of this is exactly the drift this crate's other shared tables
    /// (`owners.rs`, `phase-4-exclusions.txt`) exist to avoid one level up.
    ///
    /// **The builtin outcome runs no activation at all**, which is measured
    /// and is why it returns from the middle of this function rather than
    /// joining the label path below: `builtin`'s own module doc has the three
    /// observables -- `SIGL`, the `>A>` argument lines and the activation
    /// level -- with the probe for each. The arguments are evaluated for it by
    /// exactly the same loop the label path uses, which is what makes those
    /// `>A>` lines identical without anything here arranging it.
    ///
    /// `code` is the body the **argument expressions** are written in, which
    /// is the caller's own and is not what `resolve_call` searched.
    ///
    /// `call_type` is which invocation form got here, and it is a parameter
    /// because only the caller knows: a `::ROUTINE` body reached by `CALL`
    /// and the same body reached as a function are the same `Entry::Routine`
    /// and answer different `PARSE SOURCE` second words ([`CallType`]'s own
    /// doc has the measurement). The label path below ignores it and takes
    /// the enclosing activation's instead, which is measured too and is why
    /// the field travels in [`Inherited`].
    ///
    /// **That parameter costs one instruction per builtin call and every
    /// program pays it**, because `Resolved::Builtin` returns from the middle
    /// of this function before any activation exists: a builtin materialises
    /// the argument and nothing ever reads it. Measured over a five-round
    /// interleaved sitting, `instructions:u` per pass, against the sitting
    /// before it -- `strings` calls four builtins per pass and moved +4.0004
    /// (tw) / +4.0000 (ir); `alloc4c` calls one and moved +0.67 (tw) / +1.01
    /// (ir), where the tw figure is inside that arm's own round spread for
    /// the sitting and so does not resolve an effect this small -- it is the
    /// ir arm that carries `alloc4c`'s agreement with the model, and
    /// `strings` that carries the model; `emptyloop` and `varlookup` call
    /// none and moved by under a
    /// thousandth; `arith` and `compound` call none and moved by less than
    /// the unchanged pinned build's own drift on those axes. That is 0.030%
    /// of `strings` and 0.009% (tw) of `alloc4c`, an order of magnitude under
    /// this phase's 1% floor, which is why it was accepted.
    ///
    /// **The cheaper shape, named here so it is not rediscovered as a
    /// cost:** keep the value off the path a builtin takes. [`Entered`] is
    /// already the type that exists only past the builtin return, so a call
    /// type carried on it would leave the builtin arm with nothing to
    /// materialise. That one is not built here and has not been measured, so
    /// what is known about it is the cost and the direction, not the win.
    ///
    /// **A caller that has already resolved the name to a builtin wants
    /// [`Interp::invoke_builtin_call`] instead**, and that is the other half
    /// of the same shape: an entry point the call site chooses carries no
    /// call type to materialise, and it answers a value rather than an
    /// `Ended`, which is wider than a register pair and travels through
    /// memory. `Interp::eval_call_resolved` takes it. What still arrives
    /// here with a builtin is the `CALL` instruction's own route, which holds
    /// an `Ended` for its `Flow` regardless.
    pub(crate) fn invoke_call(
        &mut self,
        code: &Code<'_>,
        resolved: Resolved,
        name: &[u8],
        args: &[Option<Expr>],
        call_type: CallType,
        entry: CallEntry,
    ) -> Result<Ended, Failure> {
        // **Evaluated in the caller, before anything is pushed**, which is
        // where the argument expressions' own variables live. Observable
        // through failure, and the failure is real and measured: `call sub
        // 1/0` is Error 42.3
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
        // `current_value_indent` -- `invoke_call` restores it on
        // the way out, so re-reading it is what keeps a second argument's
        // own line at the caller's indent rather than at the first
        // argument's callee's. Measured (`trace i`): `call sub 1,,3` traces
        // `>A>   "1"`, `>A>   ""`, `>A>   "3"`, in that order, each right
        // after its own argument's `>L>`/`>V>` lines.
        // **The builtin path returns from here**, before an `Argument` is ever
        // built and before any activation exists -- [`Interp::
        // invoke_builtin_call`] is the whole of it, and its own doc says what
        // it does not do that the label path below does.
        if let Resolved::Builtin(target) = resolved {
            return Ok(Ended::Returned(Some(
                self.invoke_builtin_call(code, target, name, args)?,
            )));
        }

        // A fresh `Vec` and not a lent one: this path always hands the
        // arguments to the callee, which keeps them, so there is nothing to
        // give back and a pool would allocate on every call anyway.
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
                Some(expr) if self.leaf_argument(expr) => {
                    arguments.push(Some(Argument::Value(self.eval_leaf_argument(code, expr)?)));
                }
                Some(expr) => arguments.push(Some(self.eval_traced_argument(code, expr)?)),
            }
        }
        self.invoke_call_over(resolved, name, arguments, call_type, entry)
    }

    /// One compiled call over the arguments its own ops already evaluated.
    ///
    /// **The values stand on [`Interp::call_args`] rather than being passed
    /// as a slice**, because a builtin needs `&mut Interp` and the arguments
    /// at once: the stack is taken out for the duration and put back with
    /// this call's own run removed. A callee that pushes runs of its own
    /// starts from an empty stack and leaves it empty, so what comes back is
    /// what went out.
    ///
    /// `name` is only ever read on a failure -- an unresolved routine names
    /// itself, and a function returning nothing names itself -- so the caller
    /// recovers it from the op's address rather than on every execution.
    pub(crate) fn call_over_pushed_args(
        &mut self,
        resolved: Resolved,
        name: &[u8],
        mark: usize,
    ) -> Result<ObjRef, Failure> {
        self.run_over_pushed_args(mark, |interp, values| {
            interp.call_over_values(resolved, name, values)
        })
    }

    /// Runs `body` over the argument run standing above `mark`, with the
    /// stack lent out for the duration and this run removed on the way back.
    ///
    /// **The stack is taken out rather than borrowed**, because `body` needs
    /// `&mut Interp` and the values at once. What that leaves behind is an
    /// empty stack, which is exactly what a callee pushing runs of its own
    /// should start from.
    ///
    /// **Restored before the outcome is read**, so a raised condition leaves
    /// the stack as an ordinary return does.
    fn run_over_pushed_args(
        &mut self,
        mark: usize,
        body: impl FnOnce(&mut Interp, &[Option<ObjRef>]) -> Result<ObjRef, Failure>,
    ) -> Result<ObjRef, Failure> {
        let mut values = std::mem::take(&mut self.value_buffer);
        let outcome = body(self, &values[mark..]);
        values.truncate(mark);
        self.value_buffer = values;
        outcome
    }

    /// [`Interp::call_over_pushed_args`] with the run in hand.
    fn call_over_values(
        &mut self,
        resolved: Resolved,
        name: &[u8],
        values: &[Option<ObjRef>],
    ) -> Result<ObjRef, Failure> {
        // The builtin path, which runs no activation -- the same shortcut
        // `eval_call_resolved` takes and for the same measured reason.
        if let Resolved::Builtin(target) = resolved {
            return builtin::run(self, name, target, values);
        }
        let arguments = values
            .iter()
            .map(|value| value.map(Argument::Value))
            .collect();
        match self.invoke_call_over(
            resolved,
            name,
            arguments,
            CallType::Function,
            CallEntry::Written,
        )? {
            Ended::Exited(value) => Err(Failure::Exited(value)),
            Ended::Returned(Some(value)) => Ok(value),
            Ended::Returned(None) => Err(Raised::no_data_returned(name).into()),
        }
    }

    /// [`Interp::invoke_call`] past its argument evaluation: everything a
    /// callee needs once its arguments are values.
    ///
    /// **Split out for the compiled call path**, which evaluates arguments
    /// through ops of its own (`Op::PushArg`) and so arrives here holding
    /// values where `invoke_call` arrives holding expressions. Both reach one
    /// copy of the activation bookkeeping below, which is what keeps `SIGL`,
    /// the depth guard and the level accounting from drifting apart between
    /// the two.
    pub(crate) fn invoke_call_over(
        &mut self,
        resolved: Resolved,
        name: &[u8],
        arguments: Vec<Option<Argument>>,
        call_type: CallType,
        entry: CallEntry,
    ) -> Result<Ended, Failure> {
        // **The builtin outcome ends here**, before `SIGL`, before the depth
        // guard and before any activation is pushed -- each of those three is
        // the label path's, and the oracle answers that the builtin path has
        // none of them (`builtin`'s own module doc carries the probe for
        // each). Every argument is still rooted by the loop above, so the
        // allocation a builtin's result costs happens with the inputs
        // reachable, and the value handed back is rooted by whichever caller
        // receives it exactly as a callee's `RETURN` value already is.
        let entered = match resolved {
            // Answered above, before the loop that just ran.
            Resolved::Builtin(_) => unreachable!("the builtin path returns before this"),
            Resolved::Label(target) => Entered::Label(target),
            Resolved::Routine(installed) => Entered::Routine(installed),
        };

        // The caller's own program and body selector, which a label callee
        // inherits: the same pair `resolve_call` searched, read again here
        // rather than threaded out of it, because what they are wanted for is
        // building the callee rather than finding it.
        //
        // **Below the builtin return rather than above it**, which is where
        // the same three reads used to sit when resolution and invocation were
        // one function: a builtin runs no activation at all, so it has no
        // callee to build and the `Rc::clone` would be a refcount pair it
        // never uses. Every builtin call in an expression reaches this, which
        // is the shape `bench-programs/strings.rex` runs four of per pass.
        let program = Rc::clone(&self.activation().program);
        let program_id = self.activation().program_id;
        let selector = self.activation().body;

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
        //
        // **Not on the `::ROUTINE` path, and both halves of that are
        // measured.** `signal there` / `there:` / `call rtn` leaves the
        // caller's own `SIGL` at 1, the `SIGNAL`'s line, so a routine call
        // does not overwrite it; and `sigl` read inside the routine prints
        // the derived name `SIGL`, so nothing sets one in the routine's own
        // pool either. This one line writes the *caller's* pool for a label
        // (the two share it) and would write the *routine's* for a routine,
        // so both probes would go wrong if it ran on both paths.
        if matches!(entered, Entered::Label(_)) {
            self.set_sigl(self.clause_state.line());
        }

        // D19/I6: one Rust frame per activation, plus this counter, so an
        // unbounded recursion becomes a reportable condition instead of a
        // native abort. `Raised::insufficient_stack` already existed
        // (`error.rs`); measured, the oracle answers the same 11.1 at rc 245
        // for the same program, at its own depth of 27,314.
        if self.activation_depth() >= MAX_ACTIVATION_DEPTH {
            return Err(Raised::insufficient_stack().into());
        }

        let callee_id = self.next_activation_id();
        match entered {
            Entered::Label(target) => {
                // **D9r's default: a shared pool.** The callee reuses the
                // caller's `SlotFrame`, so it reads and writes the caller's
                // variables and its writes survive the return -- measured,
                // and `pop_slots` is deliberately not called on the way out
                // because the frame is not this activation's to free. Task
                // 5's `PROCEDURE` is what will ever push a frame of its own.
                //
                // `extra` is cloned in and moved back out for the same
                // reason: it is the *name* half of that one pool (`plan.rs`'s
                // own `slot_of`), and leaving the callee with an empty one
                // would strand a name bound at run time inside it. Measured
                // on the oracle -- a callee running `interpret "zork = 42"`
                // and a caller then saying `zork` prints 42, which needs the
                // binding as well as the slot to cross the return. Empty in
                // every program that has no `INTERPRET` and no `DROP (v)`,
                // which is why the clone is not a cost worth avoiding.
                let caller = self.activation();
                let plan = Rc::clone(&caller.plan);
                let frame = caller.frame;
                // The `call_type` parameter is deliberately not read here: an
                // internal label answers `PARSE SOURCE`'s second word from the
                // activation it was called out of, whichever form called it.
                // Measured, a `::routine` invoked as a function whose body
                // `CALL`s a label reads `FUNCTION` inside that label, not
                // `SUBROUTINE`.
                let caller_call_type = caller.call_type;
                let settings = caller.settings.clone();
                let trace_mode = caller.trace_mode;
                let extra = caller.extra.clone();
                // Cloned with `extra`, and for the same reason: a label
                // reached without `PROCEDURE` shares the caller's pool, and an
                // exposed name is part of that pool. Measured -- a class
                // method exposing `v`, calling a label that assigns `v`, and a
                // second class method reading `v` back -- the assignment
                // reaches the object variable. Without this the label writes
                // the frame slot the exposure left empty and the write is
                // lost.
                let exposed = caller.exposed.clone();
                // Cloned in and never written back, exactly like `settings`
                // and `trace_mode` beside it -- `Activation::traps`' own doc
                // comment has the three probes that measure the inheritance
                // and its one-way direction.
                let traps = caller.traps.clone();
                // Both halves of the pair, not just the current one:
                // measured, a callee's own bare `ADDRESS` swaps to the
                // *caller's* alternate. `Activation::address`' own doc
                // comment has the transcript.
                let address = caller.address.clone();
                // Same one-way rule again: an internal call sees the caller's
                // `CONDITION()` answers and a reset inside the callee dies
                // with it. `TrappedCondition`'s own doc comment has the
                // four-line transcript.
                let condition = caller.condition.clone();
                let mut callee = Activation::nested(
                    callee_id,
                    program,
                    program_id,
                    selector,
                    plan,
                    frame,
                    target,
                    Inherited {
                        call_type: caller_call_type,
                        settings,
                        trace_mode,
                        address,
                        traps,
                        condition,
                    },
                );
                callee.extra = extra;
                callee.exposed = exposed;
                self.push_activation(callee);
            }
            // **A pool of its own, and not one of the five inheritances**
            // -- `Activation::routine` is where that is stated and
            // `Activation::nested`'s own doc carries the six probes. The
            // plan is the routine body's own, cached under its own
            // `BodyKey`, and it is what sizes the frame: a routine's names
            // are not the caller's, so a frame sized from the caller's plan
            // would be the wrong length.
            Entered::Routine(installed) => {
                // Reached through `programs` rather than through the running
                // activation's own `Rc`, so the plan's cache key and the
                // activation's program are the same program by construction
                // (`InstalledRoutine`'s own doc).
                let routine_program = Rc::clone(&self.programs[installed.program.0]);
                let Some(body) = body_of(&routine_program, Some(installed.directive)) else {
                    return Err(Loud::missing_body().into());
                };
                let plan = self.plan_for(
                    BodyKey {
                        program: installed.program,
                        directive: Some(installed.directive),
                    },
                    body,
                    &routine_program.symbols,
                    &routine_program.source,
                );
                let frame = self.roots.push_slots(plan.len());
                self.push_activation(Activation::routine(
                    callee_id,
                    routine_program,
                    installed.program,
                    installed.directive,
                    plan,
                    frame,
                    call_type,
                ));
            }
        }

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
        //   **A `::ROUTINE` gets 0 instead**, which is the same fact as
        //   `TRACE` not crossing into one seen from the other side: measured,
        //   a routine called from inside two nested `DO` blocks and turning
        //   `trace r` on itself echoes its own clauses at indent 0, not at 6.
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
        //   `current_clause_line_is_restored_after_a_nested_expression_call`
        //   (`run/tests.rs`) is what fails if this one line is
        //   ever removed a second time; `current_value_indent_is_restored_
        //   after_a_nested_expression_call` is its own sibling for the
        //   other field.
        //   The pair is `save_clause_state`/`restore_clause_state` rather
        //   than two plain assignments (fix round 4): a `ClauseState` this
        //   function could copy freely was also one it could *replace*, which
        //   is a clause line set with no boundary attached -- the exact thing
        //   `clause.rs` exists to make unwritable.
        let saved_clause_state = self.save_clause_state();
        let callee_indent = match entered {
            Entered::Label(_) => saved_clause_state.value_indent() + 2,
            Entered::Routine(_) => 0,
        };
        let saved_base = std::mem::replace(&mut self.activation_indent, callee_indent);
        let saved_offset = std::mem::take(&mut self.indent_offset);
        let saved_line = std::mem::take(&mut self.clause_line_override);
        let inherited = entered_receiver(entered, entry, self.call_context.receiver);
        let saved_context = std::mem::replace(
            &mut self.call_context,
            CallContext {
                name: name.to_vec(),
                arguments,
                // Read out of the caller's own convention before this
                // replaces it, which is the only place it can be read from:
                // `entered_receiver` carries the rule and the measurement
                // for each of its arms.
                receiver: inherited,
            },
        );

        let ended = self.run_activation();

        // Before the pop, because both halves of `<I<`'s gate are the
        // callee's own -- its `trace_entry` state and its `TRACE` setting.
        // `RexxActivation::termination` is where the C++ puts it, which is
        // likewise inside the activation.
        self.trace_invocation_exit();

        // Popped on both paths, and unconditionally: `run_activation`'s own
        // loop asserts the activation stack is where it found it after every
        // step, so a `CALL` that returned with the callee still on it would
        // trip that assertion in the caller rather than quietly running the
        // wrong frame's `pc`.
        let mut callee = self.pop_activation().expect("the activation just pushed");
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
            // Taken rather than moved out, so the box stays whole and can be
            // parked: moving a field out of a `Box` moves the whole of it out
            // and frees the box, which is the allocation the pool exists to
            // keep. What is left behind is the empty map a fresh activation
            // starts with.
            self.activation_mut().extra = std::mem::take(&mut callee.extra);
        }
        self.recycle_activation(callee);
        self.activation_indent = saved_base;
        self.indent_offset = saved_offset;
        self.clause_line_override = saved_line;
        self.restore_clause_state(saved_clause_state);
        self.call_context = saved_context;

        // **`EXIT` inside a `::ROUTINE` ends the routine, not the program**,
        // where `EXIT` inside a `CALL`ed label ends the program. The C++'s
        // `implicitExit` sets `RETURNED` outright for an `isProgramLevelCall`
        // activation and only otherwise walks up through `exitFrom`
        // (`RexxActivation.cpp:1455`-`1469`), and a routine invocation is one
        // of those. Measured on the oracle, rc 0 every time, and the shapes
        // matter because the exit arrives here by three different routes:
        //
        // ```text
        // call rtn / say result       ::routine rtn ; exit 5      ->  5, and main runs on
        // n1 = rtn() / say n1         ::routine rtn ; exit 7      ->  7
        // call rtn / say result       ::routine rtn ; exit        ->  RESULT, unset
        // call rtn / say 'after'      a label INSIDE the routine exits 9 -> "after" runs
        // call rtn / say 'after'      interpret "exit 4" in the routine  -> "after" runs
        // ```
        //
        // The second and fourth arrive as `Failure::Exited` rather than as
        // `Ended::Exited`, because an `EXIT` reached through an expression
        // call has no `Flow` to travel on (`Failure::Exited`'s own doc,
        // `error.rs`). Both are the same event and both stop here. Falling
        // off the routine's own end is the same rule seen from the other
        // side: measured, it leaves `RESULT` unset and the caller runs on.
        let ended = match ended {
            Ok(Ended::Exited(value)) | Err(Failure::Exited(value))
                if matches!(entered, Entered::Routine(_)) =>
            {
                return Ok(Ended::Returned(value));
            }
            other => other,
        };

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

    /// [`Interp::resolve_call`] followed by [`Interp::invoke_call`], with
    /// nothing remembered in between.
    ///
    /// **The uncached composition, which is what every tree-walker route
    /// uses.** A call site that can name itself -- a compiled
    /// `crate::ir::Op::Call`, which has an op position to hang an answer on --
    /// keeps the resolution instead and calls the two halves itself; nothing
    /// here has such a name, so it resolves afresh every time.
    pub(crate) fn resolve_and_run_call(
        &mut self,
        code: &Code<'_>,
        name: &[u8],
        search_labels: bool,
        args: &[Option<Expr>],
        call_type: CallType,
        entry: CallEntry,
    ) -> Result<Ended, Failure> {
        let resolved = self.resolve_call(name, search_labels)?;
        self.invoke_call(code, resolved, name, args, call_type, entry)
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
    /// an expression through the ordinary path rather than by reading the
    /// slot directly, which is what keeps a stem reference rendering the way
    /// a bare stem read does. **Which** expression is the paragraph below --
    /// the reference node, not the inner variable; this sentence used to say
    /// "the inner expression" and contradicted it (review round 1, F6).
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
        let slot = match code.slot_for(id) {
            Some(slot) => slot,
            None => self.slot_of(code.symbols.name(id).as_bytes()),
        };
        let frame = self.activation().frame;
        // Resolved here, in the caller, where the name's own home is: an
        // alias the caller itself holds is still addressable, which is what
        // makes `>p` work when the caller's own `p` came from *its* caller,
        // and an `EXPOSE` binding is still on this activation, which is what
        // makes it work on an object variable. Chasing the slot for an
        // exposed name would hand the callee the empty slot the exposure left
        // behind.
        let target = match self.exposure(frame, slot) {
            Some(var) => VarHome::Instance(Box::new(var.clone())),
            None => VarHome::Slot(self.roots.slot_ref(frame, slot)),
        };
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

    /// One call argument, evaluated, rooted and traced -- the step both call
    /// paths share.
    ///
    /// The builtin path keeps only [`Argument::value`] and drops the rest at
    /// once; the label and routine path keeps the whole thing, because a
    /// `USE ARG >` target needs the slot a `Reference` carries. Sharing the
    /// step is what keeps `>A>` and the `>O>` line a `>p` argument traces
    /// identical on both.
    /// Whether `expr` is an argument this can evaluate without
    /// [`Interp::eval_argument`]'s wrapper -- see [`eval_leaf_argument`].
    ///
    /// [`eval_leaf_argument`]: Interp::eval_leaf_argument
    #[inline(always)]
    fn leaf_argument(&self, expr: &Expr) -> bool {
        !self.tracing_intermediates()
            && matches!(
                expr.kind,
                ExprKind::Literal(_) | ExprKind::Constant(_) | ExprKind::Variable(_)
            )
    }

    /// One argument's value, for a shape whose evaluation produces that
    /// value and nothing else.
    ///
    /// `eval` wraps every node in the depth bookkeeping and a post-order
    /// trace hook. The hook answers to `intermediates`, so with tracing off
    /// an argument that is a bare literal, constant symbol or variable read
    /// pays the wrapper for a line that is never emitted. Measured on
    /// `bench-programs/strings.rex`, evaluating each argument one extra time
    /// costs 25.05% of the program, and taking this route for its leaves
    /// gives back 8.88%.
    ///
    /// **The depth bookkeeping stays.** `StackSpan`'s own `max_depth` is
    /// observable and both engines' tests compare it, so this enters through
    /// [`Interp::enter_eval_node`] and leaves the same way `eval` does; only
    /// the hook is skipped, and only once its gate has already answered.
    ///
    /// **A reference argument is not a leaf.** `call sub >v` parses to its
    /// own expression kind, and the `Argument::Reference` that `USE ARG >`
    /// writes back through is built by `eval_argument`, which this bypasses
    /// -- so the three kinds admitted above are exactly the ones that carry
    /// no reference. Measured against the oracle, `call sub >vv` with a
    /// `use arg > a` that assigns still writes `vv` in the caller.
    ///
    /// The value is rooted here for the same reason
    /// [`Interp::eval_traced_argument`] roots its own: the callee's frame is
    /// not open yet, and everything between here and it can allocate.
    fn eval_leaf_argument(&mut self, code: &Code<'_>, expr: &Expr) -> Result<ObjRef, Failure> {
        let anchor = 0u8;
        self.enter_eval_node(&raw const anchor)?;
        let value = self.eval_node(code, expr);
        self.depth -= 1;
        let value = value?;
        self.roots.push_temp(value);
        Ok(value)
    }

    pub(crate) fn eval_traced_argument(
        &mut self,
        code: &Code<'_>,
        expr: &Expr,
    ) -> Result<Argument, Failure> {
        let argument = self.eval_argument(code, expr)?;
        self.roots.push_temp(argument.value());
        if let Some(rendered) = self.intermediate_text(argument.value()) {
            self.trace_argument(self.clause_state.current_value_indent, &rendered);
        }
        Ok(argument)
    }

    /// A call's resolution, held back until its arguments have run.
    ///
    /// **A name that matches nothing cannot be reported before the arguments
    /// have had their chance to raise.** The C++ resolves a call's target at
    /// parse time -- `externalTarget`, `targetInstruction` and `builtinIndex`
    /// are fields of the instruction -- and
    /// `RexxInstructionCall::execute` then evaluates the arguments before it
    /// dispatches on any of them (`instructions/CallInstruction.cpp:166`, the
    /// line commented "evaluate the arguments first"). A resolution that
    /// happens at run time here therefore owes the same order, which this
    /// gives it: on success nothing changes, and on failure the arguments run
    /// first and any condition they raise is reported instead.
    ///
    /// Measured against the oracle: `call nosuch 1/0` is 42.3 and not 43.1,
    /// and under `trace i`, `call nosuch 1, 2` traces `>L> "1"`, `>A> "1"`,
    /// `>L> "2"`, `>A> "2"` and only then reports 43.1. Both hold for the
    /// `nosuch(1, 2)` expression form too.
    ///
    /// The compiled paths need none of this: their arguments are ops that
    /// have already run by the time the call op resolves.
    /// **The body is `#[cold]` and the wrapper is not**, because the failure
    /// is the only case this exists for and every resolved call pays whatever
    /// stands in front of it. Measured with the loop written inline here:
    /// `bench-programs/strings.rex` retired 2,999,401 more instructions --
    /// one per iteration -- over a program that never fails to resolve
    /// anything.
    #[inline(always)]
    pub(crate) fn resolved_after_arguments(
        &mut self,
        code: &Code<'_>,
        resolution: Result<Resolved, Failure>,
        args: &[Option<Expr>],
    ) -> Result<Resolved, Failure> {
        match resolution {
            Ok(resolved) => Ok(resolved),
            Err(failure) => Err(self.arguments_before_failure(code, args, failure)),
        }
    }

    /// [`Interp::resolved_after_arguments`]' failing half: run the arguments
    /// for their trace lines and their own conditions, then report the
    /// resolution failure if none of them raised first.
    #[cold]
    #[inline(never)]
    fn arguments_before_failure(
        &mut self,
        code: &Code<'_>,
        args: &[Option<Expr>],
        failure: Failure,
    ) -> Failure {
        // The same three shapes `invoke_call`'s own loop has, for their trace
        // lines and their failures; the values themselves are dropped, since
        // there is no callee to hand them to.
        for arg in args {
            let raised = match arg {
                None => {
                    self.trace_argument(self.clause_state.current_value_indent, b"");
                    continue;
                }
                Some(expr) if self.leaf_argument(expr) => self.eval_leaf_argument(code, expr).err(),
                Some(expr) => self.eval_traced_argument(code, expr).err(),
            };
            if let Some(raised) = raised {
                return raised;
            }
        }
        failure
    }

    /// Runs one named `CALL`: `resolve_call`, then [`Interp::invoke_named_call`].
    ///
    /// See `resolve_call`'s own doc for the resolution order and
    /// `invoke_call`'s for the argument evaluation and indent bookkeeping this
    /// used to carry directly, and why both are shared with `eval_call`
    /// (`eval.rs`) rather than duplicated.
    fn exec_call(
        &mut self,
        code: &Code<'_>,
        name: &[u8],
        search_labels: bool,
        args: &[Option<Expr>],
    ) -> Result<Flow, Failure> {
        let resolution = self.resolve_call(name, search_labels);
        let resolved = self.resolved_after_arguments(code, resolution, args)?;
        self.invoke_named_call(code, resolved, name, args)
    }

    /// The `CALL` instruction past its resolution: [`Interp::invoke_call`],
    /// then settle `RESULT` and translate the outcome into this instruction's
    /// own `Flow`.
    ///
    /// **Split from `exec_call` so that a compiled call site can enter here**
    /// with a resolution it kept from an earlier execution
    /// (`crate::ir::Op::Call`). Everything a `CALL` does that
    /// `ExprKind::Call` does not is in this function and nowhere else, so the
    /// two routes cannot come to settle `RESULT` differently.
    pub(crate) fn invoke_named_call(
        &mut self,
        code: &Code<'_>,
        resolved: Resolved,
        name: &[u8],
        args: &[Option<Expr>],
    ) -> Result<Flow, Failure> {
        // Captured before `invoke_call` runs the callee, which overwrites
        // `current_value_indent` with its own clauses' -- this is the `CALL`
        // clause's own printed indent, needed below for the caller-side
        // `RESULT` trace.
        let base_indent = self.clause_state.current_value_indent;
        let ended = self.invoke_call(
            code,
            resolved,
            name,
            args,
            CallType::Subroutine,
            CallEntry::Written,
        )?;
        self.settle_call_result(ended, base_indent)
    }

    /// What a `CALL` does with the outcome its callee handed back: the
    /// caller's own `>>>` and `RESULT`.
    ///
    /// `base_indent` is the `CALL` clause's own printed indent, captured
    /// before the callee ran -- the callee overwrites `current_value_indent`
    /// with its own clauses'.
    ///
    /// **Shared by both call paths rather than copied**, because the rules
    /// below are the difference between a `CALL` and a function call and
    /// nothing about how the arguments were evaluated.
    fn settle_call_result(&mut self, ended: Ended, base_indent: usize) -> Result<Flow, Failure> {
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
        let slot = self.reserved_result_slot();
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
                // The caller's own `>>>`, at the `CALL` clause's indent --
                // `base_indent`, saved before the callee overwrote
                // `current_value_indent` with its own clauses'.
                if let Some(rendered) = self.result_text(value) {
                    self.trace_result(base_indent, &rendered);
                }
                self.set_variable(frame, slot, value);
            }
            None => self.clear_variable(frame, slot),
        }
        Ok(Flow::Next)
    }

    /// [`Interp::invoke_named_call`] over arguments the compiled stream has
    /// already evaluated onto the argument stack above `mark`.
    ///
    /// **The same split `call_over_pushed_args` makes for a call in an
    /// expression, made for the instruction**, and it is a different function
    /// rather than a `CallType` on that one because the two differ after the
    /// callee returns, not before: a subroutine settles `RESULT` and may hand
    /// back an `EXIT` that leaves the program, where a function must produce a
    /// value and raises 44.1 when it does not.
    pub(crate) fn invoke_named_call_over_pushed_args(
        &mut self,
        resolved: Resolved,
        name: &[u8],
        mark: usize,
    ) -> Result<Flow, Failure> {
        let base_indent = self.clause_state.current_value_indent;
        let ended = self.subroutine_over_pushed_args(resolved, name, mark)?;
        self.settle_call_result(ended, base_indent)
    }

    /// The callee half of [`Interp::invoke_named_call_over_pushed_args`]: the
    /// argument run is lent out, turned into `Argument`s, and removed again on
    /// the way back whichever way the call ended.
    fn subroutine_over_pushed_args(
        &mut self,
        resolved: Resolved,
        name: &[u8],
        mark: usize,
    ) -> Result<Ended, Failure> {
        let mut values = std::mem::take(&mut self.value_buffer);
        let outcome = match resolved {
            // No activation, no `Argument`s built -- the shortcut
            // `call_over_values` takes, and the reason a `CALL` to a builtin
            // reaches `Ended::Returned(Some(_))` with a value to settle.
            Resolved::Builtin(target) => builtin::run(self, name, target, &values[mark..])
                .map(|value| Ended::Returned(Some(value))),
            _ => {
                let arguments = values[mark..]
                    .iter()
                    .map(|value| value.map(Argument::Value))
                    .collect();
                self.invoke_call_over(
                    resolved,
                    name,
                    arguments,
                    CallType::Subroutine,
                    CallEntry::Written,
                )
            }
        };
        values.truncate(mark);
        self.value_buffer = values;
        outcome
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
    /// **A `DO`/`LOOP` clause is the one instruction whose frame outlives a
    /// single pass, and the two sites that pushed per pass now carry frames
    /// of their own.** `run_loop`/`run_repeating` resolve an entire
    /// multi-pass loop inside this one call -- the doc comment two paragraphs
    /// below explains why a `Goto`-shaped re-entry cannot be used instead --
    /// so anything pushed per pass and left to *this* frame would accumulate
    /// for the loop's whole run rather than one iteration's, at one `ObjRef`
    /// per pass plus whatever heap object each one pins. `loop_advance`'s
    /// `Controlled` arm and `eval_condition` are the two that push per pass,
    /// and each opens and pops a frame around its own pushes; the header
    /// values `eval_loop_header` pushes are deliberately *not* inside either,
    /// because a `DO OVER`'s target has to stay reachable for the loop's own
    /// lifetime and this frame is the one that gives it that.
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
    /// **First-wins is per *level*, and an `INTERPRET` fragment is a level.**
    /// `run_fragment` calls `seal_site_level` on its way out, so the clause
    /// this guard protects is the first one recorded *since the current level
    /// opened*, not the first one recorded in the whole run. Without that,
    /// the fragment's own clause would win the race outright and the
    /// enclosing `INTERPRET` would never be echoed at all -- which is the
    /// second of the two ways the obvious one-line fix was measured wrong.
    pub(crate) fn step_in_temps_frame(
        &mut self,
        code: &Code<'_>,
        index: usize,
        instruction: &Instruction,
        source: Option<&ProgramSource>,
    ) -> Result<Flow, Failure> {
        match self.in_stepped_clause(code, index, instruction, source, |it| {
            it.step(code, index, instruction, source)
        })? {
            ClauseOutcome::Ran(flow) => flow,
            ClauseOutcome::Ended(exit) => Ok(Flow::Exit(exit.value())),
        }
    }

    /// Everything one clause of `code` owes, around whatever `work` is: the
    /// clock invalidation, the `>I>` decay, the value indent, the clause line
    /// and boundary, the clause echo, the GC temps frame with its watermark
    /// tripwire, and the failing clause's own site -- **whether the failure is
    /// the clause's own or its boundary's**.
    ///
    /// **The one clause unit, and both engines enter it.**
    /// `step_in_temps_frame_with` passes `step`, so an unpromoted instruction
    /// gets exactly what it always did. A promoted clause passes the work its
    /// [`crate::ir`] ops do instead, which is what stops a flattened construct
    /// re-deriving any of the list above -- the defect a second implementation
    /// of this function would be.
    ///
    /// `work`'s return type is what `ClauseValue` is chosen from, which is the
    /// question "does this carry an `ObjRef` whose only root was this clause's
    /// temps frame?"; `clause.rs`'s own doc has why that has to be answered
    /// explicitly and what it still does not close.
    ///
    /// **Both failure sites are recorded here rather than by the caller**, and
    /// that is not tidiness. The `Err` this function answers is the
    /// *boundary's* -- a `CALL ON` handler delivered at the end of this clause
    /// that itself raised -- and the clause the oracle blames for it is this
    /// one, the one whose boundary ran the handler, not the enclosing
    /// instruction. Measured: a failing handler queued by an `IF`'s own
    /// condition echoes `2 *-* if raiser() = 'V'`, and with the record left to
    /// the caller a promoted `IF` echoed nothing there while an unpromoted one
    /// echoed correctly -- the two engines diverging on a program's stderr.
    /// A caller that has to remember is a caller that can forget, and one did.
    ///
    /// **`inline(always)`, and it is a measurement rather than a habit.** This
    /// wraps `Interp::in_clause`, so a clause step that reaches `step` passes
    /// through two generic-over-a-closure layers; left to the inliner's own
    /// judgement neither collapses, and the `work` closure is emitted as a
    /// function of its own that every clause calls. Measured on `emptyloop`
    /// with `perf stat -e instructions:u`, this annotation together with
    /// `in_clause`'s own removes 1.075 billion instructions from a 40-billion
    /// run, 2.7% of the whole, on the tree-walker and the compiled stream
    /// alike. `#[inline]` alone reads zero.
    #[inline(always)]
    pub(crate) fn in_stepped_clause<T: ClauseValue>(
        &mut self,
        code: &Code<'_>,
        index: usize,
        instruction: &Instruction,
        source: Option<&ProgramSource>,
        work: impl FnOnce(&mut Self) -> Result<T, Failure>,
    ) -> Result<ClauseOutcome<T>, Failure> {
        self.in_stepped_clause_with(Echo::Gated, code, index, instruction, source, work)
    }

    /// [`Interp::in_stepped_clause`], naming who emits this clause's `*-*`
    /// echo.
    ///
    /// A promoted clause can carry the echo as an op ([`Echo::Compiled`])
    /// instead, so the answer is an argument rather than a constant. Two
    /// shapes take it: this closure form, and [`Interp::enter_stepped_clause`]
    /// for a caller running the clause's own work in a loop of its own.
    #[inline(always)]
    pub(crate) fn in_stepped_clause_with<T: ClauseValue>(
        &mut self,
        echo: Echo,
        code: &Code<'_>,
        index: usize,
        instruction: &Instruction,
        source: Option<&ProgramSource>,
        work: impl FnOnce(&mut Self) -> Result<T, Failure>,
    ) -> Result<ClauseOutcome<T>, Failure> {
        // The tree-walker reaches a clause with no chunk in hand, so it reads
        // the plan as this always did.
        let entry = self.enter_stepped_clause(echo, code, index, instruction, source, None);
        let ran = work(self);
        self.leave_stepped_clause(entry, code, index, instruction, source, ran)
    }

    /// Opens a stepped clause of `code`: everything
    /// [`Interp::in_stepped_clause_with`] owes before the clause's own work
    /// runs.
    ///
    /// Half of the clause unit rather than a function in its own right. The
    /// closure form above is the unit's contract and the other entry shape
    /// into these same two halves; `clause.rs`'s module doc has why there are
    /// two shapes, and [`SteppedClause`] has what the token does and does not
    /// close.
    ///
    /// **`inline(always)` for the reason the closure form carries**, and the
    /// measurement there was taken on the whole unit rather than on either
    /// half.
    #[inline(always)]
    pub(crate) fn enter_stepped_clause(
        &mut self,
        echo: Echo,
        code: &Code<'_>,
        index: usize,
        instruction: &Instruction,
        source: Option<&ProgramSource>,
        position: Option<crate::ir::ClausePosition>,
    ) -> SteppedClause {
        // `DATE`/`TIME`'s per-clause clock cache (`activation.rs`'s own doc
        // on `Activation::clock_stale`) is invalidated **unconditionally,
        // once per call, on whichever activation is executing right now**,
        // mirroring `RexxActivation::run`'s `settings.timeStamp.valid =
        // false` right after `nextInst->execute()` returns
        // (`RexxActivation.cpp:647`). This is that same one place every
        // flat instruction position -- including one nested inside `IF`/
        // `SELECT`/`DO`/`INTERPRET`, per this function's own doc, and
        // including a callee's own body reached through `resolve_and_run_
        // call` -- passes through, so a clause's first clock read gets a
        // fresh one, any further read of the same clause sees the value
        // that read cached, and a callee's own instructions invalidate only
        // the callee's own cache rather than reaching back into the
        // caller's. The cached *value* itself (`Activation::cached_clock`)
        // is left untouched here -- `TIME('R')`'s own lazy reset needs it
        // still readable one call later, and clearing it here is what an
        // earlier version of this did instead.
        //
        // **Reached together with the `>I>` decay below through one borrow**,
        // because both write the activation that is executing right now and
        // each reach for it is a null check on the running slot.
        let activation = self.activation_mut();
        activation.clock_stale = true;
        // `>I>`'s own "am I still on the first instruction" decay
        // (`TraceEntry`, `activation.rs`), spent **here** and not in
        // `run_activation`'s loop. `RexxActivation.cpp:657`-`659` clears the
        // C++'s flag at the bottom of its instruction loop, and an `IF`'s
        // then-clause, a `DO` body clause and a fragment's clauses are each a
        // separate instruction in that loop -- where here they are nested
        // inside their enclosing clause's step. This function is the one
        // place a clause is stepped at any depth, so it is the only site that
        // counts them all. Measured before the move: `if 1=1 then trace l` as
        // a routine's first clause announced the pair here and nothing on the
        // oracle.
        //
        // At the *top*, before the clause runs, because `Pending -> Allowed`
        // is what a `TRACE` inside this very clause must see.
        activation.trace_entry = activation.trace_entry.stepped();
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
        // **The chunk's own table where the driver had one to hand over.**
        // Both answers are properties of where the clause is written, so the
        // compiler settled them and what is left here is `activation_indent`
        // and `indent_offset`, which only a running interpreter knows. The
        // `source` and override tests keep the shortcut exactly equivalent to
        // the reads it replaces: `clause_line_at` answers `None` with no
        // source, and honours `clause_line_override` ahead of any table.
        let shortcut = match position {
            Some(position) if source.is_some() && self.clause_line_override.is_none() => {
                Some(position)
            }
            _ => None,
        };
        let indent = match shortcut {
            Some(position) => {
                position.indent as usize + self.activation_indent + self.indent_offset
            }
            None => self.printed_indent(code, index),
        };
        debug_assert_eq!(
            indent,
            self.printed_indent(code, index),
            "the chunk's indent table disagrees with the plan's for instruction {index}"
        );
        self.clause_state.current_value_indent = indent;
        // Set here with the indent, and for the same reason that one is: this
        // is the one place every stepped instruction passes before its `step`
        // call runs, which is where the oracle reads it too. See the field.
        self.clause_state.instructions_traced_at_entry = self.trace_mode().all;
        // Set unconditionally, exactly like `current_value_indent` just
        // above and for the identical reason (that field's own doc comment):
        // `SIGL` (`lib.rs`'s doc on `current_clause_line`) has to stay
        // correct whether or not `TRACE` is on, and this is the one place
        // every stepped instruction, `SIGNAL`/`CALL` included, passes
        // through before its own `step` call runs.
        // `enter_clause` rather than a bare assignment: the clause line and
        // the clause boundary are one operation (`clause.rs`), and the
        // `ClauseEntry` this hands back is what `leave_stepped_clause` spends
        // on the matching half.
        let line = match shortcut {
            Some(position) => position.line as usize,
            None => self
                .clause_line_at(code, index, instruction, source)
                .unwrap_or_else(|| self.clause_state.line()),
        };
        debug_assert_eq!(
            line,
            self.clause_line_at(code, index, instruction, source)
                .unwrap_or_else(|| self.clause_state.line()),
            "the chunk's line table disagrees with the plan's for instruction {index}"
        );
        let entry = self.enter_clause(line);
        // **`Echo::Gated` asks whether the setting in force echoes this
        // clause; `Echo::Compiled` is a clause whose chunk already answered
        // that**, and emits the echo as an op of its own
        // (`crate::ir::Op::TraceClause`) rather than here. The whole point of
        // the second arm is that a chunk compiled under a setting that does
        // not echo pays nothing at all for the decision -- no gate, no
        // `clause_site`, no op.
        if matches!(echo, Echo::Gated) {
            self.echo_stepped_clause(source, instruction, indent);
        }
        // The debug tripwire I22 asks for, alongside `RootSet::temps_len`, its
        // one prerequisite. `SteppedClause` carries the watermark and the
        // frame across to the matching half, which is where the check reads
        // them; that type's own doc comment has what it checks and why it is
        // there rather than in `pop_frame`.
        //
        // Cheap enough to leave on in debug and absent in release: one
        // `Vec::len` before and after, and a comparison.
        let temps_at_entry = self.roots.temps_len();
        let frame = self.roots.push_frame();
        SteppedClause {
            entry,
            frame,
            temps_at_entry,
        }
    }

    /// Closes the stepped clause `entry` opened, around `ran` -- everything
    /// [`Interp::in_stepped_clause_with`] owes once the clause's own work has
    /// run.
    ///
    /// **`ran` is the clause's own result as a value**, which is what lets a
    /// caller whose work is a loop rather than a closure reach this at all;
    /// `Interp::leave_clause` has the whole of that reasoning, and the `Err`
    /// this answers is the boundary's rather than the clause's exactly as it
    /// is there.
    #[inline(always)]
    pub(crate) fn leave_stepped_clause<T: ClauseValue>(
        &mut self,
        entry: SteppedClause,
        code: &Code<'_>,
        index: usize,
        instruction: &Instruction,
        source: Option<&ProgramSource>,
        ran: Result<T, Failure>,
    ) -> Result<ClauseOutcome<T>, Failure> {
        debug_assert!(
            ran.is_err() || self.roots.temps_len() >= entry.temps_at_entry,
            "step popped below its own temps watermark ({} -> {}), so it \
             discarded roots it did not push",
            entry.temps_at_entry,
            self.roots.temps_len()
        );
        self.roots.pop_frame(entry.frame);
        if ran.is_err() {
            self.record_failure_site(code, index, source, instruction);
        }
        // **A `DO`/`LOOP`'s step is not a clause the oracle has, so it owes no
        // boundary** -- `Interp::leave_clause_without_boundary` has the
        // mechanism and the transcript. The boundaries the construct does owe
        // are opened elsewhere: a plain `DO`'s header and `END` clauses in
        // `run_loop_with_header`'s own `LoopKind::Simple` arm, and a
        // repeating loop's clauses in `run_repeating`.
        let outcome = match &instruction.kind {
            InstructionKind::Do(_) | InstructionKind::Loop(_) => {
                self.leave_clause_without_boundary(entry.entry, ran)
            }
            _ => self.leave_clause(entry.entry, code, ran),
        };
        // The clause's *own* failure came back as `Ran(Err(_))` and was
        // recorded just above; this `Err` is the boundary's, and it is the
        // same clause that owes the site. Recording it twice is harmless --
        // `record_failure_at`'s first-wins guard makes the second call a
        // no-op -- and recording it in neither place is what the measurement
        // in `in_stepped_clause_with`'s doc comment describes.
        if outcome.is_err() {
            self.record_failure_site(code, index, source, instruction);
        }
        outcome
    }

    /// The stepped-clause boundary for a clause that produced **neither a
    /// `Flow` nor a failure**, with nothing queued to deliver.
    ///
    /// Everything [`Interp::leave_stepped_clause`] does, minus the parts that
    /// exist for a value: there is no failure, so no site to record, and no
    /// outcome to build. What is left is releasing the temps frame and
    /// spending the entry.
    ///
    /// **Why it is a second function rather than a fast path inside the
    /// first.** `leave_stepped_clause` takes and answers
    /// `Result<ClauseOutcome<T>, Failure>`, and building that value is the
    /// cost this avoids -- a fast path *inside* it would still have to
    /// construct the answer. Measured on a loop whose body is `nop`, the one
    /// line calling `leave_clause` was 25.8% of the program's user
    /// instructions (`perf record -e instructions:u`, `perf report --sort
    /// srcline`), against a clause that does no work at all.
    ///
    /// The caller owes the `pending_traps` check, because it is the caller
    /// that knows whether this exit is available; `spend_clause_entry`
    /// re-asserts it in debug.
    #[inline(always)]
    pub(crate) fn finish_plain_clause(&mut self, entry: SteppedClause) {
        debug_assert!(
            self.roots.temps_len() >= entry.temps_at_entry,
            "step popped below its own temps watermark ({} -> {}), so it \
             discarded roots it did not push",
            entry.temps_at_entry,
            self.roots.temps_len()
        );
        self.roots.pop_frame(entry.frame);
        self.spend_clause_entry(entry.entry);
    }

    /// One stepped clause's `*-*` echo, **if the setting in force echoes a
    /// clause of that kind**, at `indent`.
    ///
    /// The gate is `tracing_clause` rather than `trace_mode().all` so that
    /// the decision lives in one place, and it is a gate at all because
    /// `clause_site` allocates the clause's text.
    ///
    /// **`is_label` is what makes `TRACE L` produce anything at all** (4b
    /// Task 9, review round 1, F8): the oracle's `RexxInstructionLabel::
    /// execute` traces through `traceLabel` and nothing else, and that gate is
    /// `tracingLabels()`, true under `L` as well as `A`/`R`/`I`. This is the
    /// only clause-echo site a `LABEL` ever reaches, so it is the only one
    /// that has to ask. Measured under `trace l`: a fallen-through label, a
    /// `CALL` target and a `SIGNAL` target all echo, in that one program's
    /// whole stderr, and every other clause is silent.
    ///
    /// **`inline(always)`, and it is a measurement rather than a habit.** This
    /// is one call per clause of every body on either engine, and on an
    /// untraced run the gate is the whole of what it does. Left to the
    /// inliner's own judgement it is emitted as a function and costs
    /// `bench-programs/emptyloop.rex` **525,000,000 user instructions** --
    /// 38.0009 against 38.5259 billion on the tree-walker, and 40.4009 against
    /// 40.9259 on the compiled stream (`perf stat -e instructions:u`).
    ///
    /// **Counted in instructions and not in seconds, and that is not a
    /// preference.** `emptyloop`'s wall clock on this machine moves about 2%
    /// between builds that execute an identical instruction count, which is
    /// larger than most of what is being decided here -- a wall-clock reading
    /// reported a regression on the tree-walker arm that its own instruction
    /// count denies.
    #[inline(always)]
    pub(crate) fn echo_stepped_clause(
        &mut self,
        source: Option<&ProgramSource>,
        instruction: &Instruction,
        indent: usize,
    ) {
        let is_label = matches!(instruction.kind, InstructionKind::Label { .. });
        if self.tracing_clause(is_label)
            && let Some((line, text)) = self.clause_site(source, instruction)
        {
            self.trace_stepped_clause(is_label, line, indent, &text);
        }
    }

    /// The same echo with **no gate at all**, for a caller that has already
    /// decided ([`crate::ir::Op::TraceClause`], whose presence in a chunk is
    /// that decision).
    ///
    /// `trace_stepped_clause`'s own `tracing_clause` gate is bypassed by
    /// passing `is_label` through unchanged and calling the formatter
    /// directly, so the bytes are the ones `echo_stepped_clause` would have
    /// produced and only the question of *whether* differs.
    pub(crate) fn echo_compiled_clause(
        &mut self,
        source: Option<&ProgramSource>,
        instruction: &Instruction,
        indent: usize,
    ) {
        if let Some((line, text)) = self.clause_site(source, instruction) {
            crate::trace::push_clause(&mut self.trace, line, indent, &text);
        }
    }

    /// Resolves `instruction`'s own clause (and its statically-derived
    /// indent, `static_indent`) into `self.failure_site`, first call wins,
    /// when `source` is `Some`.
    ///
    /// **The guard is not here and is not spelled `is_none()`.** It is an
    /// early `if self.failure_site.is_some() { return; }` at the top of
    /// [`Interp::record_failure_at`], which this function's own last line
    /// delegates to.
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
    pub(crate) fn record_failure_site(
        &mut self,
        code: &Code<'_>,
        index: usize,
        source: Option<&ProgramSource>,
        instruction: &Instruction,
    ) {
        let indent = self.printed_indent(code, index);
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
            self.failure_site = Some(FailureSite::Clause { line, text, indent });
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
    pub(crate) fn leave_origin(
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
            indent: self.printed_indent(code, index),
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
        self.record_failure_site_at(origin.site.clone(), origin.indent);
    }

    /// The same first-wins record from an already-resolved `(line, text)`
    /// pair rather than from an `&Instruction`, for the one caller that has
    /// no instruction left to resolve: a loop re-test blamed on the
    /// `ITERATE` that transferred control back to it, whose site was
    /// captured a pass earlier (`HeaderClause::Iterate`). `indent` is the
    /// caller's, because that blame prints at the loop body's indent rather
    /// than at the `ITERATE`'s own -- see that variant's doc comment.
    fn record_failure_site_at(&mut self, site: Option<(usize, Vec<u8>)>, indent: usize) {
        if self.failure_site.is_some() {
            return;
        }
        if let Some((line, text)) = site {
            self.failure_site = Some(FailureSite::Clause { line, text, indent });
        }
    }

    /// The clause boundary a promoted construct owes once the branch it chose
    /// has finished -- **the one `step_in_temps_frame` runs for the
    /// tree-walker and flattening removed.**
    ///
    /// `IF` and `SELECT` each end their own *header* clause before running
    /// anything else (`clause.rs`'s whole rule), and both engines do that the
    /// same way. What the tree-walker also has, and a flattened construct does
    /// not, is the wrapper around the whole arm: `step_in_temps_frame_with`
    /// opens a clause for the `IF`/`SELECT` instruction, resolves the branch
    /// inside it, and runs a boundary on the way out. A promoted construct is a
    /// run of ops with no wrapper, so that boundary has to be an op.
    ///
    /// **It is not a spare boundary, and the program that says so is this
    /// one:**
    ///
    /// ```text
    /// call on user zx name h        /* h raises zy */
    /// call on user zy name g        /* g says SIGL */
    /// select
    /// when 1 = 1 then zq = raiser()
    /// end
    /// say 'after'
    /// ```
    ///
    /// `h` runs at the body clause's own boundary and its `RAISE ... RETURN`
    /// **leaves a new trap queued behind it**, and a boundary drains only the
    /// entries that were queued when it began, so that new trap is not one that
    /// boundary owes. The last member clause's boundary is therefore not the
    /// last boundary with work to do, and without this one `g` runs after
    /// `say 'after'` instead of before it, at the wrong `SIGL`. Oracle
    /// and tree-walker print `G ran 4` then `after`; the compiled stream
    /// printed `after` then `G ran 6`, and in debug tripped `in_clause`'s own
    /// assertion. The `IF` spelling of the same program is the same defect.
    ///
    /// **The line is left alone**, which is what makes `SIGL` agree: the oracle
    /// closes a branch with a synthetic instruction Phase 3 elides (`ast.rs`'s
    /// "Why there is no node for the synthetic end of a branch"), and a clause
    /// with no source position of its own does not move `SIGL` off the last
    /// clause that had one. Measured across four spellings: the delivered
    /// handler reports the branch's last clause's line.
    ///
    /// `flow` is what the branch answered, passed through so it is rooted
    /// across a delivered handler exactly as `step_in_temps_frame_with`'s own
    /// `ClauseValue for Flow` roots it.
    pub(crate) fn end_promoted_branch(
        &mut self,
        code: &Code<'_>,
        flow: Flow,
    ) -> Result<Flow, Failure> {
        let line = self.clause_state.line();
        match self.in_clause(code, line, move |_| Ok(flow))? {
            ClauseOutcome::Ran(ran) => ran,
            ClauseOutcome::Ended(exit) => Ok(Flow::Exit(exit.value())),
        }
    }

    /// A `SELECT CASE`'s own `CASE` expression: the whole of what the
    /// `SELECT` header clause does, and the value every `WHEN CASE` of that
    /// `SELECT` is compared against.
    ///
    /// **One implementation, entered from both engines.** `step`'s own
    /// `Select` arm calls it inside the header's `in_clause` and keeps the
    /// value in a local for the scan; `ir::compile` emits it as the one op of
    /// the header's clause region and keeps the value in a register of the
    /// enclosing scope, which is what makes it outlive the member clauses that
    /// read it. What each engine does with the answer differs; deriving it
    /// does not.
    ///
    /// The value is pushed as a temp because the text is taken from it after
    /// the clause boundary has run, and a `CALL ON` handler delivered there
    /// allocates.
    pub(crate) fn select_case(
        &mut self,
        code: &Code<'_>,
        case_expr: &Expr,
    ) -> Result<ObjRef, Failure> {
        // The clause unit set this to the `SELECT`'s own printed indent on the
        // way in.
        let indent = self.clause_state.current_value_indent;
        let value = self.eval(code, case_expr)?;
        self.roots.push_temp(value);
        let text = self.to_text(value).to_vec();
        // `>K>` (`SelectInstruction.cpp:372`, `traceKeywordResult(CASE, ...)`),
        // at the `SELECT`'s own level -- measured, `>K>   "CASE" => "2"` sits
        // at the same indent as `select case ...` itself, not the `WHEN`-scan
        // level a `WhenCase`'s own comparison lines are indented to.
        self.trace_keyword(indent, "CASE", &text);
        // **The block opens after this**, so the header clause's own boundary
        // runs one level in -- `newBlockInstruction` is the next thing
        // `RexxInstructionSelectCase::execute` does once the scrutinee has
        // been evaluated and its `>K>` traced, and a `CALL ON` handler
        // delivered at that boundary is based on whatever the counter reads
        // then. Measured: the handler `h:` of a `select case raiser()` at top
        // level echoes at `12 *-*     h:` on the oracle, where the `SELECT`
        // clause itself echoes unindented.
        //
        // Here rather than at either engine's own call site because this
        // function is the shared one: the tree-walker calls it inside the
        // header's `in_clause` and the compiled stream reaches it from the
        // one `crate::ir::Op::EvalExpr` of the header's clause region, and
        // both boundaries are past this line.
        //
        // **A `SELECT` with no `CASE` never reaches here, so nothing settles
        // its boundary -- and that is a gap rather than an absence.** A
        // boundary delivers whatever is pending, not only what its own clause
        // queued: a `CALL ON` handler ending in `raise ... return` leaves a
        // second condition for the next boundary to take. Measured under
        // `trace r` on both engines, a plain `select` reached that way echoes
        // the second handler two columns short of the oracle, while `select
        // case 1` -- the same program with a scrutinee that queues nothing
        // either -- agrees, because this line runs for it. So what decides is
        // whether anything settled the boundary, never what the clause
        // queued. Recorded in `tests/ir_dual_cases/loop-header-boundaries`.
        self.settle_block_indent(true, indent);
        Ok(value)
    }

    /// Hands a `SELECT` its case text: the value its `WHEN CASE`s compare
    /// against, and `Interp::current_case_text` for the **absorbed** ones that
    /// have no other way to reach it (`lib.rs`'s own doc comment on the
    /// field).
    ///
    /// **Called after the header clause has ended, by both engines**, so a
    /// `CALL ON` handler delivered at that clause's boundary cannot be the
    /// last writer of the field. That placement is the whole content of this
    /// function, which is why it is one function rather than an assignment
    /// written out at each engine's own call site.
    pub(crate) fn open_select_case(&mut self, value: Option<ObjRef>) -> Option<Vec<u8>> {
        let text = value.map(|value| self.to_text(value).to_vec());
        self.current_case_text = text.clone();
        text
    }

    /// One listed `WHEN`/`WHEN CASE`'s own condition, and whether it holds.
    ///
    /// **The tree-walker's entry for both, and the compiled stream's for a
    /// `WHEN CASE` and for a `WHEN` whose condition `native_shape`
    /// declined**: a plain `WHEN` whose condition compiled does not come
    /// through here at all -- its ops leave the value in a register and
    /// `crate::ir::Op::Condition` enters [`Interp::condition_value`] with it,
    /// which is the half the two share, tagged so that 34.2 is still the
    /// raiser.
    ///
    /// **The work of one clause, and nothing a clause owes around it.** The
    /// caller opens the clause with [`Interp::in_stepped_clause`], so the
    /// `*-*` echo, the value indent this reads back, the `SIGL` line, the
    /// boundary and *both* failure sites -- the condition's own and the
    /// boundary's -- are that unit's, entered from both engines through it
    /// rather than written out beside each caller. The call site in `Select`'s
    /// own arm has what a hand-rolled version of that list measured as.
    ///
    /// The answer is a `bool` because that is what the clause produces: a
    /// Rexx logical value already consumed into one, with no `ObjRef` whose
    /// only root was this clause's temps frame (`ClauseValue for bool`).
    /// Where a matched `WHEN` sends control is [`when_targets`], read from
    /// the same node by whichever engine needs it.
    pub(crate) fn scan_when(
        &mut self,
        code: &Code<'_>,
        when_instruction: &Instruction,
        case_text: Option<&[u8]>,
    ) -> Result<bool, Failure> {
        // The clause unit set this to this `WHEN`'s own printed indent on the
        // way in, which is what its condition's `>>>` lines trace at.
        let indent = self.clause_state.current_value_indent;
        match &when_instruction.kind {
            InstructionKind::When { condition, .. } => self.eval_condition(
                code,
                condition,
                ConditionTrace::Result(indent),
                raised_when_not_logical,
            ),
            InstructionKind::WhenCase { values, .. } => match case_text {
                Some(case_text) => self.test_case_when(code, values, case_text, indent),
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
                    Ok(false)
                }
            },
            other => panic!("a SELECT's whens holds only When/WhenCase, not {other:?}"),
        }
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
    /// **The `OTHERWISE` marker is inside the range, not in front of it**, so
    /// it is stepped by the same clause unit every other instruction reaches
    /// and its `*-*` echo, its value indent and its boundary are that unit's
    /// rather than written out here. `step`'s own `Otherwise` arm is the
    /// no-op the marker's execution is. That is also what lets `ir::compile`
    /// emit the marker as an ordinary op: an engine that had to reproduce a
    /// hand-rolled echo would be reproducing it, not sharing it.
    fn run_otherwise(
        &mut self,
        code: &Code<'_>,
        index: usize,
        label: Option<SymbolId>,
        otherwise_index: usize,
        end: Option<usize>,
        source: Option<&ProgramSource>,
    ) -> Result<Flow, Failure> {
        let otherwise_end = otherwise_range(code.body.instructions.len(), end);
        let flow = self.run_bounded(
            code,
            otherwise_index,
            otherwise_end,
            source,
            BodyEngine::TreeWalker,
        )?;
        // Restores the offset to `0` now that `OTHERWISE`'s own whole
        // dispatch (marker and body alike) is finished reading it --
        // `?` above already returned early without reaching this line if
        // `run_bounded` raised, which this crate's own rule leaves
        // unrestored deliberately: a raise that is not trapped is fatal, so
        // nothing runs afterward to see a stale value, the same reasoning
        // `lib.rs`'s own doc comment gives for never restoring it after
        // `END`'s own 7.3 either.
        self.leave_otherwise(code, index, label, otherwise_end, end, flow)
    }

    /// Leaving a `SELECT`'s `OTHERWISE` branch: the escape elevation is
    /// restored now that the whole dispatch -- marker and body alike -- is
    /// finished reading it, and then `leave_select` decides where control
    /// goes.
    ///
    /// **One function because it is one rule, and both engines reach it.** A
    /// raise leaves the offset unrestored deliberately: `run_otherwise`'s own
    /// `?` returns before this is called, and a raise that is not trapped is
    /// fatal, so nothing runs afterward to see a stale value -- the same
    /// reasoning `lib.rs`'s doc comment gives for never restoring it after
    /// `END`'s own 7.3 either.
    pub(crate) fn leave_otherwise(
        &mut self,
        code: &Code<'_>,
        index: usize,
        label: Option<SymbolId>,
        otherwise_end: usize,
        end: Option<usize>,
        flow: Flow,
    ) -> Result<Flow, Failure> {
        debug_assert_eq!(
            otherwise_end,
            otherwise_range(code.body.instructions.len(), end),
            "an OTHERWISE branch was run over a range that is not its own"
        );
        self.indent_offset = 0;
        let resume = otherwise_resume(code.body.instructions.len(), end);
        self.leave_select(code, index, label, resume, flow)
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
    pub(crate) fn leave_select(
        &mut self,
        code: &Code<'_>,
        index: usize,
        label: Option<SymbolId>,
        resume: SelectResume,
        flow: Flow,
    ) -> Result<Flow, Failure> {
        match flow {
            Flow::Next => Ok(Flow::Goto(resume.done)),
            Flow::Leave(Some(name), _) if label == Some(name) => Ok(Flow::Goto(resume.left)),
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
    /// currently running rather than to a frame being unwound. The
    /// fourteen-point probe behind that rule leaves it exactly as it was,
    /// because `activation_indent` is `0` in every one of those fourteen
    /// shapes.
    fn pop_search_frame(
        &self,
        code: &Code<'_>,
        index: usize,
        mut origin: Box<LeaveOrigin>,
    ) -> Box<LeaveOrigin> {
        origin.indent = static_indent(&code.body.instructions, index) + self.activation_indent;
        // `site` and `clause_line` are left alone: this resets the *indent*
        // the search reports at, and the clause line stays the
        // `LEAVE`/`ITERATE`'s own however many frames it is forwarded past --
        // measured, `iterate lab` inside an inner loop attributes the outer
        // loop's re-test to the `ITERATE`'s line, not to anything about the
        // frames in between.
        //
        // Updated through the existing box rather than built as a fresh
        // `LeaveOrigin`, so forwarding a flow past a construct moves a
        // pointer instead of copying the site's own buffer.
        origin
    }

    /// `target`'s own **absolute printed indent**: its lexical
    /// `static_indent`, plus the activation base it is running under, plus
    /// any escape elevation currently in force.
    ///
    /// **The one place either offset is applied, and it exists because
    /// open-coding it was a defect.** The six sites that needed
    /// `+ self.indent_offset` each wrote it out, and one of them -- the
    /// `WHEN` scan in `Select`'s own arm -- did not.
    ///
    /// **The divergence does not need a fragment.** The missing
    /// addend is already wrong for a nested `SELECT` inside an escaped
    /// `OTHERWISE`, with no `INTERPRET` anywhere: measured under `trace r`,
    /// `select case 2` / `when 2 then` / `when 3 then nop` / `otherwise` /
    /// `select` / `when 1 = 1 then nop` / `end` / `end` printed the inner
    /// `WHEN` at 6 where the oracle prints 10. A plain `SELECT` inside an
    /// `INTERPRET` inside one `DO` also hits it, and that is not deep nesting.
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
    pub(crate) fn printed_indent(&self, code: &Code<'_>, target: usize) -> usize {
        // The table when this body has one, and the walk when it does not --
        // an `INTERPRET` fragment is the case with none, and its instruction
        // list is short enough that the walk is what it always was.
        let base = match code.plan {
            Some(plan) => plan.indent_of(&code.body.instructions, target),
            None => static_indent(&code.body.instructions, target),
        };
        base + self.activation_indent + self.indent_offset
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
    /// unchanged, exactly as received. That covers `Flow::Exit`, and is
    /// written to keep covering `LEAVE`/`ITERATE` too:
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
    ///
    /// `engine` decides only which driver each member clause is stepped
    /// through, and nothing about this loop's own absorption rule
    /// ([`BodyEngine`]'s own doc comment).
    fn run_bounded(
        &mut self,
        code: &Code<'_>,
        start: usize,
        end: usize,
        source: Option<&ProgramSource>,
        engine: BodyEngine<'_>,
    ) -> Result<Flow, Failure> {
        match engine {
            BodyEngine::TreeWalker => self.run_bounded_instructions(code, start, end, source),
            // The op-level counterpart. It walks the same range under the
            // same absorption rule ([`absorb`], which both loops decide
            // through), and the whole of the difference is the space its
            // program counter lives in: a promoted construct is a run of ops
            // *inside* one instruction's position, so a loop stepping one
            // instruction at a time cannot enter it.
            BodyEngine::Chunk { chunk, registers } => {
                self.run_bounded_from_chunk(code, chunk, registers, start, end, source)
            }
        }
    }

    /// [`Interp::run_bounded`]'s tree-walker arm: one instruction at a time,
    /// straight into the tree-walker's own clause unit.
    fn run_bounded_instructions(
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
            match absorb(flow, start, end) {
                Absorbed::Advance => pc += 1,
                Absorbed::Resume(target) => pc = target,
                Absorbed::Escaped(other) => return Ok(other),
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
    /// **`COUNTER`, `DO WITH` and a stem `OVER` target all take the loud
    /// path, decided before a single header expression is evaluated.** That
    /// is [`loop_header_plan`]'s answer and its doc comment has why each of
    /// the three is refused; deciding all three in one place is what makes
    /// `do counter c with index i over x` -- two of them at once -- fail
    /// loudly without evaluating `x` either.
    ///
    /// **One implementation, entered from both engines.** `engine` reaches
    /// exactly one thing: which driver steps each of the body's clauses
    /// ([`BodyEngine`]). Nothing below branches on it -- not the header, not
    /// `WHILE`/`UNTIL`, not the label search, not a single trace echo -- which
    /// is what makes promoting `DO`/`LOOP` an extraction rather than a second
    /// loop to keep in step with this one.
    ///
    /// **The compiled stream enters at the second half rather than here.** A
    /// promoted `DO`/`LOOP` evaluates its own header as ops -- one group per
    /// entry of the same [`HeaderPlan`] this function iterates, each ending in
    /// the `ir::Op::LoopHeaderValue` that validates and files that value --
    /// and then reaches [`Interp::run_loop_with_header`] with what they
    /// produced, so the two engines share the *validation* of each value and
    /// the whole of the construct below it, and differ only in what drives the
    /// header's sequence.
    #[allow(
        clippy::too_many_arguments,
        reason = "the same argument `run_repeating`'s own allow makes: every parameter is state one DO/LOOP needs"
    )]
    fn run_loop(
        &mut self,
        code: &Code<'_>,
        index: usize,
        instruction: &Instruction,
        body: &Loop,
        source: Option<&ProgramSource>,
        engine: BodyEngine<'_>,
    ) -> Result<Flow, Failure> {
        // A refused `DO`/`LOOP` has no header to evaluate, and the refusal
        // itself belongs to `run_loop_with_header` rather than here, so that
        // both engines reach it through one function. The compiled stream
        // emits an empty header region and that same op for a refused loop
        // for exactly this reason: a compiler that refused on its own would
        // be a second copy of the decision, and the copy that panicked
        // instead was invisible to the whole workspace.
        let values = match loop_header_plan(body) {
            Some(plan) => self.eval_loop_header(code, body, &plan)?,
            None => LoopHeaderValues::default(),
        };
        self.run_loop_with_header(code, index, instruction, body, source, engine, values)
    }

    /// Evaluates every expression of `body`'s header, in `plan`'s order,
    /// echoing each value under its own `>K>` tag as it goes.
    ///
    /// **The interleaving is the semantics, not an implementation detail.**
    /// The oracle evaluates `TO`, echoes it, validates it, and only then
    /// evaluates `BY`: measured, `do i = 1 to 'a' by 2` echoes `>K>  "TO" =>
    /// "a"` and raises 41.1 with no `>K>  "BY"` line at all. So the echo and
    /// the validation both sit inside this loop rather than after it.
    ///
    /// `push_temp` roots each value for the whole of the `DO` clause, which is
    /// what a `DO OVER`'s target needs: `LoopState::OverOnce` keeps it for the
    /// loop's own lifetime, and the loop runs inside this clause.
    fn eval_loop_header(
        &mut self,
        code: &Code<'_>,
        body: &Loop,
        plan: &HeaderPlan,
    ) -> Result<LoopHeaderValues, Failure> {
        let mut values = LoopHeaderValues::default();
        for &role in plan.roles() {
            let expr = header_expr_for(&body.kind, role)
                .expect("the plan names only roles this node has an expression for");
            let value = self.eval(code, expr)?;
            self.roots.push_temp(value);
            self.echo_header_value(role, value);
            self.accept_header_value(role, value, &mut values)?;
        }
        Ok(values)
    }

    /// One header value's own `>K>` line, at the `DO`/`LOOP` clause's own
    /// indent, for the roles the oracle echoes one for.
    ///
    /// **`current_value_indent` rather than a recomputed `static_indent`**, and
    /// **not** `loop_indent` (`+2`, `WHILE`/`UNTIL`'s own level): measured,
    /// `>K>   "TO" => "2"` sits at the same indent as `do i = 1 to 2` itself,
    /// because these header expressions are evaluated once at loop entry,
    /// before the body's own frame exists at all
    /// (`control_setup_expressions_are_unindented_unlike_the_loop_body_they_precede`
    /// makes the identical point about a *raise* at this same point).
    ///
    /// Read here rather than captured before the header's first evaluation, so
    /// that both engines read the same field at the same point rather than
    /// agreeing by an argument about what an evaluation can leave behind. The
    /// two are the same answer: `Interp::invoke_call` restores
    /// `current_value_indent` on the way out, which
    /// `current_value_indent_is_restored_after_a_call` pins.
    pub(crate) fn echo_header_value(&mut self, role: HeaderRole, value: ObjRef) {
        let Some(keyword) = role.keyword() else {
            return;
        };
        // `trace_keyword` carries its own `results` gate, so this is not a second
        // decision about whether to *print*: it decides whether to render the
        // value into a `Vec` at all, which an untraced run has no use for. The
        // same shape and the same reason as `bind_control`'s own check, one level
        // down from where that one sits.
        if !self.trace_mode().results {
            return;
        }
        let text = self.to_text(value).to_vec();
        self.trace_keyword(self.clause_state.current_value_indent, keyword, &text);
    }

    /// Validates one header value against whatever its role requires and files
    /// it in `values`.
    ///
    /// `initial`/`to`/`by` need only be *numeric* (41.1 if not, via
    /// `arith_operand` -- the same check ordinary arithmetic already makes),
    /// never *whole*: measured, `do i = 1.5 to 3` is legal and steps by
    /// fractional values. A count is the exception, checked against
    /// `whole_nonneg` -- 26.2 for a bare `DO`'s own repeat count and 26.3 for a
    /// `FOR`, which is the only way the two differ.
    ///
    /// **The digits in force are read where they are used**, which is the
    /// oracle's "current `NUMERIC DIGITS` at loop entry" rule
    /// (`round_via_unary_plus`'s own doc comment has the citation). Every read
    /// inside one header gives the same answer: `NUMERIC DIGITS` changes only
    /// by executing an instruction, this activation executes none between its
    /// own header's expressions, and a called routine's own setting does not
    /// survive its return.
    pub(crate) fn accept_header_value(
        &mut self,
        role: HeaderRole,
        value: ObjRef,
        values: &mut LoopHeaderValues,
    ) -> Result<(), Failure> {
        match role {
            // **The rounded value and the representation it arrived in.** The
            // rounding is the oracle's, and for an integer inside `DIGITS` it
            // is a no-op, so the two arms carry the same worth and differ only
            // in what [`ControlValue::small`] may then answer.
            HeaderRole::Initial => {
                let digits = self.activation().settings.digits();
                // **The `Number` is built only by the arm that keeps it.**
                // `ControlValue::Small` carries the `i64` the tag already
                // holds, so asking `header_number` first and then discarding
                // its answer builds a digit vector for every ordinary counted
                // loop and throws it away. What that call would have checked
                // -- an object in an operand position, a non-numeric value --
                // a tagged integer cannot fail.
                values.initial = Some(match value.decode() {
                    Decoded::SmallInt(small) if within_digits(small, digits) => {
                        ControlValue::Small(small)
                    }
                    _ => ControlValue::Wide(self.header_number(role, value)?),
                });
            }
            HeaderRole::To => values.to = Some(self.header_number(role, value)?),
            HeaderRole::By => values.by = Some(self.header_number(role, value)?),
            // **The rendering is built inside the failing arm**, because a
            // count is nearly always whole and the copy only ever reaches the
            // message: hoisting it renders and frees a string per loop header
            // to serve a branch that is not taken.
            HeaderRole::For | HeaderRole::OverFor => {
                values.for_remaining = Some(match self.whole_nonneg(value) {
                    Some(count) => count,
                    None => return Err(raised_for_count_not_whole(&self.to_text(value)).into()),
                });
            }
            HeaderRole::Count => {
                values.count = Some(match self.whole_nonneg(value) {
                    Some(count) => count,
                    None => {
                        return Err(raised_repetition_count_not_whole(&self.to_text(value)).into());
                    }
                });
            }
            // **`DO OVER` is not `stringValue()` and not an operator**, which
            // is why R12's other sites do not cover it: the oracle hands the
            // target to `requestArray`. Measured, `do e over .array` is
            // 98.913 at rc 158 and `do e over .environment` iterates the
            // directory's own entries, where `LoopState::OverOnce` binds the
            // target once and yields the object's rendering. A string still
            // iterates once yielding itself, which is that state's own rule
            // and stays true.
            HeaderRole::Over => {
                if let Some(kind) = self.operator_operand_gap(value) {
                    return Err(Loud::object_position(role.value_name(), kind).into());
                }
                values.over = Some(value);
            }
        }
        Ok(())
    }

    /// One controlled-loop header value as the `Number` the loop runs on:
    /// numeric (41.1 if not) and rounded at the digits in force.
    ///
    /// **R12 reaches here, and this is the one numeric surface it reaches that
    /// is not an operator.** `round_via_unary_plus` is a real unary `+` on the
    /// oracle too, so `do i = 1 to .array` sends `+` to the object and answers
    /// 97.1 -- measured, and measured in all three positions, which is why the
    /// check is here rather than on the left-hand one. `FOR` and a bare `DO`'s
    /// repeat count do **not** reach it: they go through `whole_nonneg`, which
    /// reads the value's text, so both implementations answer 26.3 and 26.2
    /// from that text alike. `NUMERIC DIGITS` does not reach it either, by a
    /// different route: `exec_numeric` renders the value with `to_text` and
    /// hands the bytes to `set_digits_str`, so it never asks for a number at
    /// all.
    ///
    /// A test in front rather than a rewrite of the failing path, unlike every
    /// other R12 site: a header value is evaluated once per loop entry, not
    /// once per iteration, so there is no hot path here to keep clear of.
    fn header_number(&mut self, role: HeaderRole, value: ObjRef) -> Result<Number, Failure> {
        let entry_digits = self.activation().settings.digits();
        // **A tagged integer no wider than `DIGITS` is its own rounding**, so
        // the unary `+` below has nothing to do to it and the general path
        // only spends: `arith_operand` builds a `Number`, the addition copies
        // it into a zero-left fast path, rounds it, and copies it out again.
        // Neither of the two checks the general path makes can fail here --
        // the tag holds no object for `operator_operand_gap` to find, and an
        // integer is numeric -- so this answers directly.
        //
        // **The header is a hot path, which it does not look like.** Measured
        // against the -O3 oracle: `do n = 1 to N ; do j = 1 to 1 ; end ; end`
        // costs 1366 cycles an outer iteration here against its 596, and the
        // inner header runs once per iteration of the loop above it -- an
        // entry, not an iteration, is still per-iteration work when the loop
        // is nested.
        if let Decoded::SmallInt(small) = value.decode()
            && within_digits(small, entry_digits)
        {
            let shortcut = Number::from_i64(small);
            // The tripwire, not the test: a disagreement is invisible in the
            // answer for every program that stays inside the tag, which is
            // why it is asserted on every header of every program the debug
            // gate runs rather than probed once.
            debug_assert_eq!(
                Ok(&shortcut),
                round_via_unary_plus(&Number::from_i64(small), entry_digits).as_ref(),
                "a tagged integer within DIGITS {entry_digits} is not its own unary +"
            );
            return Ok(shortcut);
        }
        if let Some(kind) = self.operator_operand_gap(value) {
            return Err(Loud::object_position(role.value_name(), kind).into());
        }
        let result = self.header_number_body(value, entry_digits);
        // Blamed on any failure past the object-position check above, not
        // only `arith_operand`'s own conversion -- the same reason
        // `Interp::arith_general` (`eval.rs`) blames its own whole
        // computation rather than only `Interp::arith_left_operand`'s.
        // Measured: `numeric digits 1; s. = '9.9E999999999'; do i = s. to
        // 5` is 42.901 with the frame, past a conversion that already
        // succeeded -- `round_via_unary_plus`'s own range check is what
        // raises, still inside the same forwarded unary `+`.
        if result.is_err() {
            self.blame_stem_forwarded_operator(b"+", value);
        }
        result
    }

    /// [`Interp::header_number`]'s own computation for a position that
    /// reaches it (`Initial`, `To` and `By` -- see that function's own
    /// doc), wrapped by it so every failing step is blamed once.
    fn header_number_body(&mut self, value: ObjRef, entry_digits: u64) -> Result<Number, Failure> {
        let operand = self.arith_operand(value)?;
        Ok(round_via_unary_plus(&operand, entry_digits).map_err(Raised::from)?)
    }

    /// A `DO`/`LOOP` past its header: the construct itself, driven from the
    /// values whichever engine evaluated that header produced.
    ///
    /// **This is the whole of the loop that is not its header**, and it is one
    /// function so that a promoted `DO`/`LOOP` reaches every iteration, every
    /// trace echo and the `LEAVE`/`ITERATE` search through the same code the
    /// tree-walker does.
    #[allow(
        clippy::too_many_arguments,
        reason = "the same argument `run_repeating`'s own allow makes: every parameter is state one DO/LOOP needs"
    )]
    pub(crate) fn run_loop_with_header(
        &mut self,
        code: &Code<'_>,
        index: usize,
        instruction: &Instruction,
        body: &Loop,
        source: Option<&ProgramSource>,
        engine: BodyEngine<'_>,
        values: LoopHeaderValues,
    ) -> Result<Flow, Failure> {
        // **The refusal, for both engines, and it has to be here rather than
        // in each engine's own entry.** `loop_header_plan` answers `None` for
        // exactly the three forms this crate does not run, and every arm below
        // relies on that answer: `LoopKind::With` has no `LoopState`, and a
        // `COUNTER` or a stem `OVER` reaches an `expect` on a value nothing
        // evaluated. Measured, before this check existed: all three panicked on
        // the compiled stream while failing loudly on the tree-walker, and no
        // test in the workspace was red.
        if loop_header_plan(body).is_none() {
            return Err(Loud::instruction(&instruction.kind).into());
        }
        let body_start = index + 1;
        let end_index = body
            .end
            .expect("an unclosed DO/LOOP is error 14.1/14.5, so a body that parsed has this set");
        let resume = end_index + 1;
        let label = body.label;

        let state = match &body.kind {
            // A block, not a loop: exactly one pass, and `WHILE`/`UNTIL`
            // can never be present (`create_loop`'s own parser only reaches
            // `LoopKind::Simple` through the bare, at-end-of-clause `DO`
            // arm, before any conditional is even looked for). Still
            // leavable by an explicit `DO LABEL`, never by a bare `LEAVE`
            // (`do_body_outcome`'s own `is_loop: false`) -- measured, a
            // labelled simple block is leavable but an unlabelled one is
            // 28.1 on a bare `LEAVE` reaching it.
            LoopKind::Simple => {
                // Captured before the body runs, for the same reason
                // `run_repeating` captures it: this is the `DO`'s own indent,
                // and `current_value_indent` holds whatever the last body
                // clause left once `run_bounded` has returned.
                let do_indent = self.clause_state.current_value_indent;
                // **The block's own clause, opened and ended before any body
                // instruction runs.** The oracle's
                // `RexxInstructionSimpleDo::execute` traces the instruction,
                // opens the block and returns -- it never runs the body -- so
                // a condition queued before the `DO` is delivered here, at the
                // `DO`'s own line and with the block open. Measured, a handler
                // that requeues ahead of `do` / `say 'body'` / `end`: the
                // oracle runs the requeued handler with `SIGL` naming the `DO`
                // and prints its output ahead of `body`.
                //
                // Entered whether or not anything is queued, exactly as
                // `InstructionKind::Select`'s own clause is: what decides is
                // whether the boundary exists, never what the clause queued.
                let do_line = self
                    .clause_line_at(code, index, instruction, source)
                    .unwrap_or_else(|| self.clause_state.line());
                match self.in_clause(code, do_line, |it| {
                    // A block that opened, so the boundary's own handler runs
                    // one level in -- the `select case raiser()` row of
                    // `settle_block_indent`'s table, and the same call.
                    it.settle_block_indent(true, do_indent);
                    Ok(())
                })? {
                    ClauseOutcome::Ended(exit) => return Ok(Flow::Exit(exit.value())),
                    ClauseOutcome::Ran(ran) => ran?,
                }
                let flow = self.run_bounded(code, body_start, end_index, source, engine)?;
                return match self.do_body_outcome(code, index, label, false, resume, flow)? {
                    DoOutcome::Escaped(escape) => Ok(escape),
                    // Falls through to `END`, which `run_bounded`'s own
                    // `[body_start, end_index)` range never visits (the
                    // `Goto(resume)` below jumps straight past it) --
                    // unlike a repeating loop, a `Simple` block never runs
                    // this arm again, so one explicit echo here is the
                    // whole story, not a per-pass one the way
                    // `run_repeating`'s own is. Measured, Task 11's
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
                    DoOutcome::FellThrough | DoOutcome::Iterated { .. } => {
                        // **`END` is a clause of its own, and its boundary is
                        // where a condition queued by the block's last body
                        // clause is delivered.** The oracle executes `END` as
                        // an instruction, so a handler requeued inside the
                        // block runs there rather than after the whole
                        // construct. Measured, `do` / `end` with a handler
                        // that requeues twice: the oracle reports the second
                        // handler's `SIGL` as `END`'s line, ahead of the
                        // `SAY` that follows the block; without this clause
                        // it ran after that `SAY` had already printed, on
                        // both engines.
                        //
                        // Only on the pass that reaches `END`: an escape
                        // returns above, and the oracle's `END` is jumped
                        // straight over by a `LEAVE`.
                        let end_instruction = &code.body.instructions[end_index];
                        let end_line = self
                            .clause_line_at(code, end_index, end_instruction, source)
                            .unwrap_or_else(|| self.clause_state.line());
                        // A fresh computation, not `current_value_indent` --
                        // `run_bounded`, just above, has already stepped this
                        // block's own body, so that field now holds whatever
                        // the *last* body instruction left it at, not this
                        // `DO`'s own. `printed_indent` for the same reason
                        // every other site on this page uses it: consistency
                        // if this `Simple` block's own `END` is ever itself
                        // the direct landing point of an escape (untested,
                        // but cheap to keep uniform rather than silently
                        // exempt).
                        let end_indent = self.printed_indent(code, index);
                        match self.in_clause(code, end_line, |it| {
                            if it.trace_mode().all
                                && let Some((line, text)) = it.clause_site(source, end_instruction)
                            {
                                it.trace_clause(line, end_indent, &text);
                            }
                            // The block is closed by the time this clause's
                            // boundary runs, so its handler sits back out at
                            // the `DO`'s own level.
                            it.settle_block_indent(false, do_indent);
                            Ok(())
                        })? {
                            ClauseOutcome::Ended(exit) => return Ok(Flow::Exit(exit.value())),
                            ClauseOutcome::Ran(ran) => ran?,
                        }
                        Ok(Flow::Goto(resume))
                    }
                };
            }
            LoopKind::Forever => LoopState::Forever,
            // A bare `DO` with no expression at all runs a single pass,
            // matching `Simple`'s own behaviour -- see `loop_header_plan`'s
            // own note on why nothing in this crate's tests reaches it.
            LoopKind::Count(_) => LoopState::Count {
                remaining: values.count.unwrap_or(1),
            },
            LoopKind::Controlled(ctrl) => LoopState::Controlled {
                control: ctrl.control,
                at: control_slot(code, ctrl.control),
                // The header's own value, in the representation the header
                // produced it in. The first re-test replaces it, computing its
                // own representation the same way.
                current: values
                    .initial
                    .expect("a controlled loop's plan always names its initial value"),
                to: values.to,
                by: match values.by {
                    Some(by) => by,
                    // No `round_via_unary_plus` needed on the default: a bare
                    // literal `1` is already whole at any width, so rounding
                    // it at the header's digits could only ever answer `1`
                    // again.
                    None => Number::one(),
                },
                for_remaining: values.for_remaining,
                cached_digits: u64::MAX,
                to_int: None,
                by_int: None,
                shape: shape_of(code.symbols.name(ctrl.control).as_bytes()),
                stepped: false,
            },
            LoopKind::Over { control, .. } => LoopState::OverOnce {
                control: *control,
                at: control_slot(code, *control),
                value: values
                    .over
                    .expect("a DO OVER's plan always names its target"),
                done: false,
                remaining: values.for_remaining,
            },
            LoopKind::With { .. } => unreachable!("DO WITH takes the loud path above"),
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
            state,
            engine,
        )
    }

    /// Leaves `current_value_indent` at the indent a **block instruction's**
    /// own clause **boundary** runs at: one level in when that instruction's
    /// block is still open once the clause has finished, the clause's own
    /// indent when it is not.
    ///
    /// **The clause echoes at one indent and ends at another, and only the
    /// boundary can see the difference.** In the oracle a block instruction
    /// traces its clause, evaluates whatever expressions that clause carries,
    /// and only then calls `newBlockInstruction`, whose
    /// `settings.traceIndent++` is the block's own level --
    /// `RexxInstructionBaseLoop::execute` evaluates its control expressions
    /// in `setup` first, and `RexxInstructionSelectCase::execute` evaluates
    /// its `CASE` scrutinee and traces the `>K>` for it first. So the counter
    /// at the end of that one instruction reads one deeper than its own echo.
    /// A loop's iteration test that fails then calls `terminate`, which pops
    /// the block and takes the level back off again; a `SELECT`'s clause has
    /// no such path, so `open` is what each caller knows and this cannot
    /// work out for itself.
    ///
    /// What observes it is a `CALL ON` handler delivered at this boundary,
    /// since `internalCallTrap` bases the handler's own activation on
    /// whatever the counter reads then. Measured under `trace r` with a
    /// handler `h:`, both directions of each header shape, because a fix
    /// that moved the entered case alone would have broken the rest. The
    /// oracle's own first handler line, with the line number each program
    /// happened to put `h:` on:
    ///
    /// ```text
    /// do zi = 1 to raiser()      raiser returns 1 -> 10 *-*     h:
    /// do zi = 1 to raiser()      raiser returns 0 -> 10 *-*   h:
    /// do while raiser() < 1      test false       -> 10 *-*   h:
    /// do while raiser() < 1      test true        -> 12 *-*     h:
    /// do until raiser() > 0      test true        -> 10 *-*   h:
    /// do until raiser() > 0      test false       -> 13 *-*     h:
    /// select case raiser()       always open      -> 12 *-*     h:
    /// ```
    ///
    /// **A level is two columns, and it is derived here rather than passed
    /// in**: measured, a `WHEN`'s own condition sits two columns in from its
    /// `SELECT` and a loop body two in from its `DO`. Taking the deeper
    /// indent as a second argument would put two same-typed indents side by
    /// side at a call site, and handing them over the wrong way round is
    /// exactly the two-column answer this function exists to stop.
    ///
    /// **Nothing but the boundary reads what this writes.** Whatever runs
    /// next -- a loop body's first clause, the clause a loop resumes at, a
    /// `SELECT`'s own first `WHEN` -- sets the field again through
    /// `step_in_temps_frame` before anything traces, so this is not a value
    /// that survives the clause it belongs to.
    fn settle_block_indent(&mut self, open: bool, clause_indent: usize) {
        self.clause_state.current_value_indent = if open {
            clause_indent + 2
        } else {
            clause_indent
        };
    }

    /// The shared driver for every repeating `LoopKind` (everything but
    /// `Simple`, which never repeats and runs through
    /// `run_loop_with_header`'s own arm directly):
    /// advance-test-run-test-advance, in the order the oracle is measured to
    /// use it in (`report`'s own transcripts) --
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
        engine: BodyEngine<'_>,
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
        // before `run_loop_with_header` ever called into this function.
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
        let mut iterate_site: IterateSite = None;
        let end_line = self
            .clause_line_at(code, end_index, &code.body.instructions[end_index], source)
            .unwrap_or(0);
        // Hoisted out of the loop body (review round 1, F2): the header's own
        // failure path needs it one statement *before* the body that used to
        // bind it. Derived from `code`, so it borrows nothing this function
        // mutates.
        let end_instruction = &code.body.instructions[end_index];

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
                .clause_line_at(code, do_index, do_instruction, source)
                .unwrap_or_else(|| self.clause_state.line());
            let header_line = match header_clause {
                HeaderClause::Do => do_line,
                HeaderClause::End => end_line,
                HeaderClause::Iterate { line } => line,
            };
            let header = self.in_clause(code, header_line, |it| {
                // **A header that fails is blamed on the clause that
                // transferred control back here, at the loop body's indent**
                // -- not on the `DO` clause, which is where the enclosing
                // `step_in_temps_frame` would put it (review round 1, F2).
                // Reachable since the control variable is genuinely re-read:
                // a body that leaves it non-numeric fails the `BY` addition
                // on the next re-test. Measured, three shapes, all `trace r`
                // with `ii = 'abc'` in a `do ii = 1 to 3` body:
                //
                // * falling through to `END` -> `     4 *-*   end`, then
                //   `Error 41 ... line 4`;
                // * an `ITERATE` in the body -> `     5 *-*   iterate`, line 5;
                // * that same `ITERATE` two blocks deeper -> still its own
                //   line, and still at the *body's* indent rather than its
                //   own lexical one, which is why `loop_indent` is passed
                //   here rather than `LeaveOrigin::indent` being reused.
                //
                // The first pass is deliberately left alone: it is reached
                // from the `DO` clause itself, which is exactly what the
                // enclosing `step_in_temps_frame` already blames.
                let outcome = 'header: {
                    let advanced = match it.loop_advance(code, &mut state, do_indent, loop_indent) {
                        Ok(advanced) => advanced,
                        Err(failure) => {
                            match header_clause {
                                HeaderClause::Do => {}
                                HeaderClause::End => {
                                    it.record_failure_at(source, end_instruction, loop_indent);
                                }
                                HeaderClause::Iterate { .. } => {
                                    it.record_failure_site_at(iterate_site.clone(), loop_indent);
                                }
                            }
                            return Err(failure);
                        }
                    };
                    if !advanced {
                        break 'header HeaderOutcome::Stop;
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
                            Ok(false) => break 'header HeaderOutcome::Stop,
                            Err(failure) => {
                                it.record_failure_at(source, do_instruction, loop_indent);
                                return Err(failure);
                            }
                        }
                    }
                    HeaderOutcome::Continue
                };
                it.settle_block_indent(outcome.entered(), do_indent);
                Ok(outcome)
            })?;
            match header {
                ClauseOutcome::Ended(exit) => return Ok(Flow::Exit(exit.value())),
                ClauseOutcome::Ran(Err(failure)) => return Err(failure),
                ClauseOutcome::Ran(Ok(HeaderOutcome::Stop)) => return Ok(Flow::Goto(resume)),
                ClauseOutcome::Ran(Ok(HeaderOutcome::Continue)) => {}
            }

            let flow = self.run_bounded(code, body_start, end_index, source, engine)?;
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
                DoOutcome::Iterated { line, site } => {
                    header_clause = HeaderClause::Iterate { line };
                    iterate_site = site;
                }
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
                    HeaderClause::Iterate { line } => line,
                };
                // **This clause's boundary is unobservable on every probe
                // tried, and it is here because it cannot be separated from
                // the line.**
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
                    let held = it.eval_condition(
                        code,
                        &cond.condition,
                        ConditionTrace::Keyword(loop_indent, "UNTIL"),
                        raised_until_not_logical,
                    )?;
                    // An `UNTIL` that held ends the loop, so this clause's
                    // own boundary sits outside the block -- the mirror of
                    // the top-of-loop test's, and the same call.
                    it.settle_block_indent(!held, do_indent);
                    Ok(held)
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
    /// into what `run_repeating`/`run_loop_with_header`'s own `Simple` arm
    /// does next.
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
    /// `is_loop` is `false` only for `LoopKind::Simple`
    /// (`run_loop_with_header`'s own `Simple` arm passes it); every
    /// `LoopState` variant `run_repeating` drives is a real, repetitive loop
    /// and passes `true`.
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
                Ok(DoOutcome::Iterated {
                    line: origin.clause_line,
                    site: origin.site,
                })
            }
            other => Ok(DoOutcome::Escaped(other)),
        }
    }

    /// **SPIKE, not for commit.** Sets a repeating loop up to be driven from
    /// the op driver's own frame instead of a nested `run_ops` entry, or
    /// declines and leaves the caller to take the nested path.
    #[allow(clippy::too_many_arguments, reason = "spike")]
    pub(crate) fn flat_loop_start(
        &mut self,
        code: &Code<'_>,
        index: usize,
        instruction: &Instruction,
        body: &Loop,
        source: Option<&ProgramSource>,
        values: LoopHeaderValues,
        op_body: u32,
    ) -> Result<FlatStart, Failure> {
        // SPIKE: the switch is a run-time one so that both arms are the same
        // binary -- the per-op checks this spike adds to the driver's loop are
        // compiled in either way, so an arm with the flat path never taken
        // prices those checks on their own.
        static FLAT: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        // **`loop_header_plan` is the whole refusal**, exactly as it is for
        // `run_loop_with_header`: it answers `None` for a `COUNTER`, a stem
        // `OVER` and `DO WITH`, which this crate does not run on either
        // engine, and the nested path is where that becomes the loud error.
        if *FLAT.get_or_init(|| std::env::var_os("REXX_NO_FLAT").is_some())
            || loop_header_plan(body).is_none()
        {
            return Ok(FlatStart::Fallback(values));
        }
        let state = match &body.kind {
            LoopKind::Forever => LoopState::Forever,
            LoopKind::Count(_) => LoopState::Count {
                remaining: values.count.unwrap_or(1),
            },
            LoopKind::Controlled(ctrl) => LoopState::Controlled {
                control: ctrl.control,
                at: control_slot(code, ctrl.control),
                current: values
                    .initial
                    .expect("a controlled loop's plan always names its initial value"),
                to: values.to,
                by: match values.by {
                    Some(by) => by,
                    None => Number::one(),
                },
                for_remaining: values.for_remaining,
                cached_digits: u64::MAX,
                to_int: None,
                by_int: None,
                shape: shape_of(code.symbols.name(ctrl.control).as_bytes()),
                stepped: false,
            },
            LoopKind::Over { control, .. } => LoopState::OverOnce {
                control: *control,
                at: control_slot(code, *control),
                value: values
                    .over
                    .expect("a DO OVER's plan always names its target"),
                done: false,
                remaining: values.for_remaining,
            },
            // A block, not a loop: one pass, its own trace shape, and
            // `run_loop_with_header`'s own arm resolves the whole of it
            // without ever reaching a pass boundary. `DO WITH` is refused
            // above and cannot arrive here.
            LoopKind::Simple | LoopKind::With { .. } => {
                return Ok(FlatStart::Fallback(values));
            }
        };
        let end_index = body
            .end
            .expect("an unclosed DO/LOOP is error 14.1/14.5, so a body that parsed has this set");
        let do_indent = self.clause_state.current_value_indent;
        let end_line = self
            .clause_line_at(code, end_index, &code.body.instructions[end_index], source)
            .unwrap_or(0);
        let do_line = self
            .clause_line_at(code, index, instruction, source)
            .unwrap_or_else(|| self.clause_state.line());
        // **Built where it will live, not on the stack and then moved
        // there.** A `FlatLoop` is 328 bytes, 200 of them the `LoopState` a
        // controlled loop's three `Number`s live in, and the version that
        // built one here and assigned it into the box afterwards wrote those
        // bytes twice per loop entry -- which for a nested loop is twice per
        // iteration of the loop above it. Measured on `do n = 1 to N ; do j =
        // 1 to 1 ; end ; end`: -5.871% retired instructions, with
        // `__memmove_avx_unaligned_erms` falling from 8.41% of the program's
        // cycles to 2.52%.
        //
        // **The struct literal is what does it, and writing the fields one at
        // a time through the box instead is worse** -- measured, the same
        // program at -5.271% and `rexxcps` at -0.365% against -0.424%. The
        // literal has one destination and the compiler builds it there; a run
        // of field stores does not coalesce back into that.
        //
        // The spare goes back to the pool on the paths that never drive it, so
        // a loop its own header ends costs no allocation either.
        let mut boxed = match self.flat_spares.pop() {
            Some(spare) => spare,
            None => Box::new(FlatLoop::vacant()),
        };
        *boxed = FlatLoop {
            op_body,
            body_start: index + 1,
            end_index,
            do_index: index,
            resume: end_index + 1,
            label: body.label,
            do_indent,
            loop_indent: do_indent + 2,
            end_line,
            do_line,
            header_clause: HeaderClause::Do,
            iterate_site: None,
            conditional: body.conditional.as_ref().map(|cond| cond.until),
            state,
        };
        let header = match self.flat_loop_header(code, source, &mut boxed) {
            Ok(header) => header,
            Err(failure) => {
                self.flat_spares.push(boxed);
                return Err(failure);
            }
        };
        match header {
            Some(flow) => {
                // The header ended the loop, so nothing will drive it and
                // this box is spare again rather than leaked back to the
                // allocator.
                self.flat_spares.push(boxed);
                Ok(FlatStart::Ended(flow))
            }
            None => {
                let range = (boxed.body_start, boxed.end_index);
                self.flat_loops.push(boxed);
                Ok(FlatStart::Flat {
                    body_start: range.0,
                    end_index: range.1,
                })
            }
        }
    }

    /// **SPIKE.** One pass boundary of the innermost flat loop: the state is
    /// taken off `Interp::flat_loops` so that this can hold a `&mut Interp`
    /// beside it, and put back when another pass follows.
    pub(crate) fn flat_loop_step_top(
        &mut self,
        code: &Code<'_>,
        source: Option<&ProgramSource>,
        arrival: Flow,
    ) -> Result<FlatStep, Failure> {
        let Some(mut top) = self.flat_loops.pop() else {
            return Err(Loud::op_not_driven("a pass boundary with no loop open").into());
        };
        match self.flat_loop_step(code, source, &mut top, arrival) {
            Ok(FlatStep::Body(op_body)) => {
                self.flat_loops.push(top);
                Ok(FlatStep::Body(op_body))
            }
            Ok(FlatStep::Done(flow)) => {
                self.flat_spares.push(top);
                Ok(FlatStep::Done(flow))
            }
            Err(failure) => Err(failure),
        }
    }

    /// **SPIKE.** One pass boundary: what the body just answered, then the
    /// next pass's header test.
    fn flat_loop_step(
        &mut self,
        code: &Code<'_>,
        source: Option<&ProgramSource>,
        flat: &mut FlatLoop,
        arrival: Flow,
    ) -> Result<FlatStep, Failure> {
        // **Read once per pass boundary and used for both echoes.** The two
        // events are one boundary and nothing between them can run a `TRACE`,
        // so the second read could only ever answer what the first did.
        let echoing = self.trace_mode().all;
        // **A pass that fell out of its body needs nothing decided.**
        // `do_body_outcome`'s own `Flow::Next` arm answers `FellThrough` and
        // reads none of its other arguments, and that is the arrival of every
        // pass that did not end in a `LEAVE`, an `ITERATE` or an escape -- so
        // asking is a call per pass for an answer the discriminant already
        // gives.
        let outcome = if matches!(arrival, Flow::Next) {
            DoOutcome::FellThrough
        } else {
            self.do_body_outcome(code, flat.do_index, flat.label, true, flat.resume, arrival)?
        };
        match outcome {
            DoOutcome::Escaped(escape) => return Ok(FlatStep::Done(escape)),
            DoOutcome::Iterated { line, site } => {
                flat.header_clause = HeaderClause::Iterate { line };
                flat.iterate_site = site;
            }
            DoOutcome::FellThrough => {
                flat.header_clause = HeaderClause::End;
                // `END` echoes for a pass that fell through to it and for no
                // other, exactly as `run_repeating`'s own arm does.
                if echoing
                    && let Some((line, text)) =
                        self.clause_site(source, &code.body.instructions[flat.end_index])
                {
                    self.trace_clause(line, flat.do_indent, &text);
                }
            }
        }
        // **`UNTIL`'s own test, and its own re-echo of the `DO`/`LOOP`
        // clause.** `run_repeating`'s own arm has the measurement: the oracle
        // re-enters the loop instruction to make this decision as much as to
        // test `WHILE` or advance a control variable, so the echo here is
        // unconditional rather than sharing the top-of-loop one below -- which
        // is why that one is not emitted at all for an `UNTIL` loop.
        if flat.conditional == Some(true) {
            if echoing
                && let Some((line, text)) =
                    self.clause_site(source, &code.body.instructions[flat.do_index])
            {
                self.trace_clause(line, flat.do_indent, &text);
            }
            if let Some(flow) = self.flat_loop_until(code, source, flat)? {
                return Ok(FlatStep::Done(flow));
            }
        } else if echoing
            // **The re-echo of the `DO`/`LOOP` clause itself, once per pass
            // after the first**, and it is asked here rather than at the
            // loop's entry because a `TRACE` in the body changes the answer:
            // measured, an `ir_dual` case that switches tracing on inside the
            // body loses this line and `END`'s when the decision is made once
            // on the way in.
            && let Some((line, text)) =
                self.clause_site(source, &code.body.instructions[flat.do_index])
        {
            self.trace_clause(line, flat.do_indent, &text);
        }
        match self.flat_loop_header(code, source, flat)? {
            Some(flow) => Ok(FlatStep::Done(flow)),
            None => Ok(FlatStep::Body(flat.op_body)),
        }
    }

    /// **SPIKE.** An `UNTIL` loop's own bottom-of-pass test: `Some(flow)` is
    /// the loop finishing, `None` is carrying on to the header.
    ///
    /// The test belongs to whichever clause transferred control back to the
    /// loop, which is the same clause the *next* header test belongs to --
    /// `HeaderClause`'s own doc comment has the oracle transcript that tells
    /// the two candidates apart.
    fn flat_loop_until(
        &mut self,
        code: &Code<'_>,
        source: Option<&ProgramSource>,
        flat: &FlatLoop,
    ) -> Result<Option<Flow>, Failure> {
        let Some(cond) = loop_conditional_of(code, flat.do_index) else {
            return Err(Loud::instruction(&code.body.instructions[flat.do_index].kind).into());
        };
        let until_line = flat.header_line();
        let (do_indent, loop_indent, resume) = (flat.do_indent, flat.loop_indent, flat.resume);
        // The re-echoed `END` clause just above leaves `current_value_indent`
        // untouched -- `trace_clause` does not set it -- so without this the
        // `UNTIL`'s intermediates would read the `DO`'s own indent.
        self.clause_state.current_value_indent = loop_indent;
        let tested = self.in_clause(code, until_line, |it| {
            let held = it.eval_condition(
                code,
                &cond.condition,
                ConditionTrace::Keyword(loop_indent, "UNTIL"),
                raised_until_not_logical,
            )?;
            // An `UNTIL` that held ends the loop, so this clause's own
            // boundary sits outside the block.
            it.settle_block_indent(!held, do_indent);
            Ok(held)
        })?;
        match tested {
            ClauseOutcome::Ended(exit) => Ok(Some(Flow::Exit(exit.value()))),
            ClauseOutcome::Ran(Ok(true)) => Ok(Some(Flow::Goto(resume))),
            ClauseOutcome::Ran(Ok(false)) => Ok(None),
            ClauseOutcome::Ran(Err(failure)) => {
                let end = &code.body.instructions[flat.end_index];
                self.record_failure_at(source, end, loop_indent);
                Err(failure)
            }
        }
    }

    /// **SPIKE.** Who a failing header re-test is blamed on: the clause that
    /// transferred control back to the loop, at the body's indent.
    ///
    /// **Its own function, and `#[cold]` rather than inline.** It sits inside
    /// the closure the clause unit runs, and a closure that carries it inline
    /// costs the *succeeding* path: measured on `bench-programs/emptyloop.rex`,
    /// whose header never fails, about 70 instructions per pass.
    #[cold]
    #[inline(never)]
    fn blame_header_failure(
        &mut self,
        code: &Code<'_>,
        source: Option<&ProgramSource>,
        blame: HeaderClause,
        site: &IterateSite,
        end_index: usize,
        loop_indent: usize,
    ) {
        match blame {
            HeaderClause::Do => {}
            HeaderClause::End => {
                self.record_failure_at(source, &code.body.instructions[end_index], loop_indent);
            }
            HeaderClause::Iterate { .. } => {
                self.record_failure_site_at(site.clone(), loop_indent);
            }
        }
    }

    /// **SPIKE.** A `WHILE` that failed, blamed on the `DO`/`LOOP` clause at
    /// the body's indent. `#[cold]` for the reason above.
    #[cold]
    #[inline(never)]
    fn blame_while_failure(
        &mut self,
        code: &Code<'_>,
        source: Option<&ProgramSource>,
        do_index: usize,
        loop_indent: usize,
    ) {
        self.record_failure_at(source, &code.body.instructions[do_index], loop_indent);
    }

    /// **SPIKE.** The header re-test, in its own clause: `Some(flow)` is the
    /// loop finishing, `None` is one more pass.
    ///
    /// **A `WHILE` takes `flat_loop_header_while` instead, and the split is a
    /// measurement rather than a shape.** The two differ by one test, so one
    /// function with a branch is the obvious form -- and the branch is inside
    /// the closure the clause unit runs, where carrying it costs the loops
    /// that have no `WHILE` about 70 instructions per pass on
    /// `bench-programs/emptyloop.rex`. `flat_loop_step` asks which of the two
    /// this loop wants once per pass, outside that closure.
    ///
    /// **`inline(always)`, and it is a measurement rather than a habit.** This
    /// is one call per pass of every flattened loop, and the inliner's own
    /// judgement changed the moment `WHILE` was added beside it: measured on
    /// `bench-programs/emptyloop.rex`, left alone it is emitted as a function
    /// and the flat path goes from 3.08% ahead of the nested one to 3.37%
    /// behind, with `flat_loop_header` appearing in the profile at 8.44% where
    /// it had not appeared at all.
    #[inline(always)]
    fn flat_loop_header(
        &mut self,
        code: &Code<'_>,
        source: Option<&ProgramSource>,
        flat: &mut FlatLoop,
    ) -> Result<Option<Flow>, Failure> {
        if flat.conditional == Some(false) {
            return self.flat_loop_header_while(code, source, flat);
        }
        let header_line = flat.header_line();
        let do_indent = flat.do_indent;
        let loop_indent = flat.loop_indent;
        let resume = flat.resume;
        let end_index = flat.end_index;
        let blame = flat.header_clause;
        let site = &flat.iterate_site;
        let state = &mut flat.state;
        let header = self.in_clause(code, header_line, |it| {
            let advanced = match it.loop_advance(code, state, do_indent, loop_indent) {
                Ok(advanced) => advanced,
                Err(failure) => {
                    it.blame_header_failure(code, source, blame, site, end_index, loop_indent);
                    return Err(failure);
                }
            };
            it.settle_block_indent(advanced, do_indent);
            Ok(advanced)
        })?;
        flat_header_outcome(header, resume)
    }

    /// **SPIKE.** [`Interp::flat_loop_header`] for a loop that carries a
    /// `WHILE`: the same advance, then the condition, both inside the one
    /// clause the oracle re-enters to make this decision.
    ///
    /// `inline(never)` so that the branch above it stays a call this loop's
    /// shape decides once, rather than code the loops without a `WHILE` carry.
    #[inline(never)]
    fn flat_loop_header_while(
        &mut self,
        code: &Code<'_>,
        source: Option<&ProgramSource>,
        flat: &mut FlatLoop,
    ) -> Result<Option<Flow>, Failure> {
        let Some(cond) = loop_conditional_of(code, flat.do_index) else {
            return Err(Loud::instruction(&code.body.instructions[flat.do_index].kind).into());
        };
        let header_line = flat.header_line();
        let do_indent = flat.do_indent;
        let loop_indent = flat.loop_indent;
        let resume = flat.resume;
        let (do_index, end_index) = (flat.do_index, flat.end_index);
        let blame = flat.header_clause;
        let site = &flat.iterate_site;
        let state = &mut flat.state;
        let header = self.in_clause(code, header_line, |it| {
            let advanced = match it.loop_advance(code, state, do_indent, loop_indent) {
                Ok(advanced) => advanced,
                Err(failure) => {
                    it.blame_header_failure(code, source, blame, site, end_index, loop_indent);
                    return Err(failure);
                }
            };
            if !advanced {
                it.settle_block_indent(false, do_indent);
                return Ok(false);
            }
            // Overrides what stepping the `DO`/`LOOP` instruction set:
            // `WHILE`'s condition is evaluated here, inside that same step,
            // never through a `step_in_temps_frame` of its own.
            it.clause_state.current_value_indent = loop_indent;
            let held = match it.eval_condition(
                code,
                &cond.condition,
                ConditionTrace::Keyword(loop_indent, "WHILE"),
                raised_while_not_logical,
            ) {
                Ok(held) => held,
                Err(failure) => {
                    it.blame_while_failure(code, source, do_index, loop_indent);
                    return Err(failure);
                }
            };
            it.settle_block_indent(held, do_indent);
            Ok(held)
        })?;
        flat_header_outcome(header, resume)
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
                at,
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
                self.bind_control(
                    code,
                    *control,
                    loop_indent,
                    *value,
                    *at,
                    shape_of(code.symbols.name(*control).as_bytes()),
                )?;
                Ok(true)
            }
            LoopState::Controlled {
                control,
                at,
                current,
                to,
                by,
                for_remaining,
                cached_digits,
                to_int,
                by_int,
                shape,
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
                // The `!first` is exactly `stepped` here. `trace i` /
                // `do ii = 1 to 2`'s own second pass:
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
                // **The oracle binary on this machine emits neither `>>>`
                // line, and that is its age rather than a disagreement.**
                // Its `DoBlock::checkControl` has no `traceResult` call at
                // all; the two above were added between the version it was
                // built from and the `interpreter/` this crate reimplements.
                // Measured 2026-08-20 against both oracle builds. So the
                // transcript here is read off this repository's C++, and a
                // probe of this arm against the oracle will differ by
                // exactly these two lines.
                //
                // **The pair is emitted before either termination test**, on
                // the failing pass as well -- measured, `do ii = 1 to 3`
                // traces `>>>     "3"` then `>>>     "4"` on the pass that
                // ends the loop, so these are not "the values of an
                // iteration that ran".
                //
                // **`control->evaluate` is a genuine READ of the variable,
                // not a look at the loop's own saved value, and the two part
                // company the moment a body writes to the control variable**
                // (review round 1, F2). Measured: `do ii = 1 to 3 ; ii = 10 ;
                // end ; say ii` prints `11` on the oracle -- it reads `10`
                // back, adds `1`, and `11 > 3` ends the loop after one pass.
                // Reusing `current` here instead ran three passes and printed
                // `4`, and the `>V>` line above then positively stated a
                // value the oracle contradicts. So the value below is read
                // out of the variable pool, and `current` is only ever the
                // *header's* value now -- on the first pass, and never
                // again.
                //
                // Two adjacent shapes fall out of that read rather than
                // needing their own handling, and both are measured: a body
                // that `DROP`s the control variable reads the derived name
                // (`>>>   "II"`, then 41.1 `Nonnumeric value ("II")`), and a
                // body that assigns a non-numeric reads it and fails the same
                // way (41.1, `("abc")`). The reader's own miss answer and
                // `arith_operand`'s own raiser produce both without a special
                // case here. **Which reader, and why it is not the obvious
                // one, is the paragraph below** -- this one said
                // `read_by_name` until round 2 changed it and left the
                // sentence behind, which is the same defect twice on one
                // arm.
                //
                // **Bound before the decision, not after** -- measured
                // against the oracle, `do i = 5 to 3 / say never / end /
                // say i` prints `5`: the control variable takes its own
                // header value even for a loop that goes on to run zero
                // iterations, for *either* reason a candidate iteration can
                // fail below (an exhausted `FOR` budget or the `TO` bound).
                let digits = self.activation().settings.digits();
                // **The ordinary counted pass, answered without reaching any
                // of the code below.** Every branch of the arm from here on
                // exists for a shape this test excludes: a control variable
                // that is not a plain name, a value or a bound too wide for
                // the tag, a `FOR` budget, a `TRACE` that has to render the
                // value, or the first pass, whose value the header already
                // computed. What is left is read the slot, add, write the
                // slot, compare -- and, because both handles are tagged
                // integers rather than heap objects, no temporaries frame
                // for the collector to walk.
                //
                // The gate is deliberately a conjunction of the cheapest
                // available tests and never re-derives anything: `cached_
                // digits == digits` is what makes `to_int`/`by_int` usable,
                // exactly as the invalidation below defines them.
                if *stepped
                    && *shape == NameShape::Simple
                    && *cached_digits == digits
                    && for_remaining.is_none()
                    && let Some(slot) = *at
                    && let Some(step) = *by_int
                    && let Some(bound) = *to_int
                    && let ControlValue::Small(_) = current
                {
                    let mode = self.trace_mode();
                    // `FUZZ` sits here rather than in the conjunction above so
                    // a pass that fails an earlier test never reads it: the
                    // arm below is the only one that answers the bound test
                    // without consulting it, and the general path reads it once
                    // on its own account.
                    if !mode.results
                        && !mode.intermediates
                        && self.activation().settings.fuzz() == 0
                    {
                        let frame = self.activation().frame;
                        // The read is `Interp::variable` rather than
                        // `read_at`: the shape is `Simple` and the slot is
                        // the loop's own, so the two agree, and an
                        // uninitialised slot answers `None` here instead of
                        // building a derived name -- which is one of the
                        // shapes this path declines, since it goes on to
                        // fail 41.1 below.
                        if let Some(previous) = self.variable(frame, slot)
                            && let Decoded::SmallInt(value) = previous.decode()
                            && within_digits(value, digits)
                            && let Some(sum) = value.checked_add(step)
                            && within_digits(sum, digits)
                            && let Some(handle) = exact_small_int(sum, digits)
                        {
                            debug_assert_eq!(
                                *shape,
                                shape_of(code.symbols.name(*control).as_bytes()),
                                "the loop's cached control-variable shape is not the one its name gives"
                            );
                            debug_assert_eq!(
                                *to_int,
                                to.as_ref().and_then(|bound| bound.plain_integer(digits)),
                                "the loop's cached TO disagrees with DIGITS {digits}"
                            );
                            debug_assert_eq!(
                                *by_int,
                                by.plain_integer(digits),
                                "the loop's cached BY disagrees with DIGITS {digits}"
                            );
                            *current = ControlValue::Small(sum);
                            self.set_variable(frame, slot, handle);
                            return Ok(if step < 0 { sum >= bound } else { sum <= bound });
                        }
                    }
                }
                let fuzz = self.activation().settings.fuzz();
                let form = self.activation().settings.form();
                if *cached_digits != digits {
                    *cached_digits = digits;
                    *to_int = to.as_ref().and_then(|bound| bound.plain_integer(digits));
                    *by_int = by.plain_integer(digits);
                }
                // The cache is only ever as good as its invalidation, so the
                // debug gate re-derives both on every pass and compares. This
                // is the tripwire, not the test: a stale entry is invisible in
                // the answer for every program tried against the oracle, which
                // is exactly why it needs an assertion rather than a probe.
                debug_assert_eq!(
                    *to_int,
                    to.as_ref().and_then(|bound| bound.plain_integer(digits)),
                    "the loop's cached TO disagrees with DIGITS {digits}"
                );
                debug_assert_eq!(
                    *by_int,
                    by.plain_integer(digits),
                    "the loop's cached BY disagrees with DIGITS {digits}"
                );
                let (to_int, by_int) = (*to_int, *by_int);
                // The same tripwire for the control variable's shape, which
                // needs no invalidation at all: it is a property of a name
                // the parse fixed, so nothing can move it. The assertion is
                // what says so on every pass of every program the debug gate
                // runs, rather than a comment claiming it.
                debug_assert_eq!(
                    *shape,
                    shape_of(code.symbols.name(*control).as_bytes()),
                    "the loop's cached control-variable shape is not the one its name gives"
                );
                let shape = *shape;
                let re_tested = std::mem::replace(stepped, true);
                // **One pass's own temps frame, released before the next pass
                // opens one.** The enclosing `step_in_temps_frame` belongs to
                // the whole `DO` instruction, so without this every root
                // pushed below survives until the *loop* ends rather than
                // until the *pass* does -- one `ObjRef` per iteration, and
                // each one pins whatever heap object it names. Popped after
                // `bind_control` has written the new value into the control
                // variable's own storage, which is what roots it from there
                // on; the `?` paths below leave it to the outer truncation,
                // exactly as `pop_frame`'s own doc describes.
                let pass = self.roots.push_frame();
                if re_tested {
                    // **`read`, not `read_by_name`: this is an evaluation and
                    // it can raise `NOVALUE`** (review round 1 re-review,
                    // NEW-1 -- a defect this arm shipped with, not a
                    // pre-existing one). The oracle's own `control->evaluate`
                    // is a full expression evaluation, so a body that
                    // `DROP`s the control variable makes the next re-test
                    // raise `NOVALUE` rather than read a derived name.
                    // Measured, `signal on novalue name nv` around
                    // `do ii = 1 to 3 ; drop ii ; end`: the oracle runs the
                    // handler and exits 0, where `read_by_name` here gave a
                    // spurious 41.1 at rc 215. `read_by_name` reports
                    // nothing to its caller and cannot express that.
                    //
                    // **And `novalue_check` runs before any tracing**, which
                    // is measured rather than tidy: under `trace i` the
                    // oracle's failing re-test emits the `DO` re-echo and
                    // then nothing at all -- no `>V>`, no `>>>` -- because
                    // the raise happens inside the evaluation, before
                    // `traceResult` is reached.
                    //
                    // With no `NOVALUE` trap armed, `novalue_check` is a
                    // no-op and the derived name flows on to fail 41.1 on
                    // `("II")`, which is the untrapped shape and still
                    // matches.
                    // **The re-test's own read goes through the same three
                    // shapes `bind_control`'s write does**: a compound's
                    // tail is resolved fresh here too, against whichever
                    // *current* value the tail variable holds on *this*
                    // pass, not the tail the loop's header resolved once at
                    // setup.
                    // Measured against the oracle: `a.=0; i=1; Do a.i=1 To
                    // 3; If i>7 Then Leave; i=i+1; End; say i` answers `8`,
                    // not `4` -- the body's own `i=i+1` moves which tail of
                    // `a.` this read (and the write after it) resolves to on
                    // the very next pass, so the loop's own `TO 3` bound
                    // keeps comparing against a fresh, still-default `0`
                    // tail instead of the one `a.i` incremented.
                    let (previous, novalue, resolved) = match shape {
                        NameShape::Simple => {
                            // `read_at` with the slot `control_slot` took when
                            // the loop was entered, which is the same
                            // resolution this read made for itself on every
                            // pass before -- `None` still makes it, so the two
                            // shapes below and a control this resolution does
                            // not reach are unaffected.
                            let (value, novalue) = self.read_at(code, *control, *at);
                            (value, novalue, None)
                        }
                        // A bare stem never raises `NOVALUE` on read (`eval_
                        // node`'s own `ExprKind::Stem` arm has the citation),
                        // so there is no fallible read to thread through.
                        // The slot comes off the entry rather than from the
                        // loop's kept `at`, which is the same source the write
                        // half of this pass uses and the reason `control_slot`
                        // declines a stem.
                        NameShape::Stem => {
                            let at = code.compound(*control).and_then(|entry| entry.stem_at);
                            let name = code.symbols.name(*control).as_bytes();
                            (self.read_stem_at(name, at), Novalue::Set, None)
                        }
                        NameShape::Compound => {
                            let (stem_name, stem_at) = code.stem(*control);
                            let key = self.tail_key(code, *control);
                            let (value, novalue) = self.stem_get_at(stem_name, stem_at, &key);
                            let mut resolved = stem_name.to_vec();
                            resolved.extend_from_slice(&key);
                            (value, novalue, Some(resolved))
                        }
                    };
                    self.novalue_check(novalue)?;
                    self.roots.push_temp(previous);
                    // `>C>` before `>V>`, both self-gated on `intermediates`
                    // like every other value-bearing prefix -- `stem_get`'s
                    // own read announces the fully-resolved name it used
                    // before either of the value lines shows what is stored
                    // there, the same order `eval_node`'s `Compound` arm and
                    // its own tracing counterpart use for an ordinary read.
                    if let Some(resolved) = &resolved {
                        let name = code.symbols.name(*control).as_bytes();
                        self.trace_compound_name(loop_indent, name, resolved);
                    }
                    // `result_text` for the pair, not `intermediate_text`:
                    // `>V>` is `intermediates` and `>>>` is `results`, and
                    // `results` is the weaker of the two, so it renders for
                    // either and drops neither.
                    if let Some(rendered) = self.result_text(previous) {
                        let name = code.symbols.name(*control).as_bytes();
                        self.trace_variable(loop_indent, name, &rendered);
                        self.trace_result(loop_indent, &rendered);
                    }
                    // The increment, on integers when it can be. `previous`
                    // comes back out of the variable pool as a tagged small
                    // integer for every ordinary counted loop, and `BY` is
                    // whole; the guard is the same one `eval`'s own
                    // arithmetic fast path uses, and for the same reason --
                    // an operand too wide for `DIGITS` is rounded before the
                    // addition, so the exact `i64` sum would be the wrong
                    // answer.
                    //
                    // **The value in force has to be an integer too, and its
                    // tag does not answer that.** `ObjRef::small_int` is
                    // admitted on what a value *renders* as (D15), so a
                    // `Number` that reached a whole rendering by rounding --
                    // `100000002.0` at `DIGITS 9` -- comes back tagged. The
                    // interpreter's `NumberString` stays one through its own
                    // `+`, so a loop that started fractional keeps comparing
                    // its bound fuzzily forever; carrying `current`'s
                    // representation across the step is what reproduces that.
                    // Measured at `DIGITS 9 FUZZ 8`,
                    // `do zi = 100000002.0 to 100000001` does not terminate.
                    let stepped = match (&*current, previous.decode()) {
                        (ControlValue::Small(_), Decoded::SmallInt(value))
                            if within_digits(value, digits) =>
                        {
                            by_int
                                .and_then(|step| value.checked_add(step))
                                .filter(|sum| within_digits(*sum, digits))
                        }
                        _ => None,
                    };
                    // **Written inside each arm rather than assigned from the
                    // `match`'s own value**, which is layout rather than
                    // style. A single assignment site has to write a whole
                    // `ControlValue`, and that is as wide as the `Number` its
                    // other arm carries, so the integer arm pays that width
                    // to deliver a tag and an `i64`; per arm, each writes
                    // only the bytes it has. Measured, 10 instructions a pass
                    // -- 190,000,000 across `bench-programs/varlookup.rex`.
                    match stepped {
                        Some(sum) => *current = ControlValue::Small(sum),
                        None => {
                            *current =
                                ControlValue::Wide(self.controlled_step_wide(previous, by, digits)?)
                        }
                    }
                }
                // The first pass takes the value the header already computed,
                // unincremented and with no line of its own beyond the `>=>`
                // below -- `checkControl`'s own `else` arm reads it with
                // `getValue`, whose comment says why: the initial assignment
                // was already traced during setup, and tracing here too
                // "prevents getting an extra add looking item traced".
                let value = if let ControlValue::Small(small) = current
                    && let Some(handle) = exact_small_int(*small, digits)
                {
                    handle
                } else {
                    self.controlled_value_wide(current, digits, form)
                };
                // Rooted before anything else can allocate: the render below
                // builds a `Vec`, and `bind_control`'s compound arm resolves a
                // tail key, which allocates in the arena. Nothing collected
                // between this allocation and the write before the trigger
                // existed, so this push closes a window that was inert rather
                // than absent.
                self.roots.push_temp(value);
                let bind_indent = if re_tested { loop_indent } else { do_indent };
                if re_tested && let Some(rendered) = self.result_text(value) {
                    self.trace_result(loop_indent, &rendered);
                }
                self.bind_control(code, *control, bind_indent, value, *at, shape)?;
                // The control variable's own storage now roots `value`, and
                // `previous` is dead, so the pass's frame goes here. The three
                // `return Ok(false)` paths below end the loop, whose enclosing
                // frame truncates past this one anyway.
                self.roots.pop_frame(pass);

                if let Some(r) = for_remaining
                    && *r == 0
                {
                    return Ok(false);
                }
                if let Some(to) = to {
                    // The bound test, on integers when it can be.
                    // `numeric_less` reaches it through a subtraction, which
                    // allocates twice per pass for what an `i64` comparison
                    // answers outright.
                    //
                    // **A non-zero `FUZZ` disqualifies the exact path**, which
                    // is why `fuzz == 0` guards the integer arm. The
                    // interpreter reaches the bound test as an ordinary Rexx
                    // comparison -- `DoBlock::checkControl` calls
                    // `result->callOperatorMethod(compare, to)` -- and
                    // `RexxInteger::comp`
                    // (`interpreter/classes/IntegerClass.cpp`) subtracts
                    // directly only when both sides are integer objects that
                    // fit `NUMERIC DIGITS` *and* `number_fuzz() == 0`.
                    // Anything else falls to the fuzzed `NumberString::comp`.
                    // Measured at `DIGITS 9 FUZZ 8`, `do zi = 100000002 to
                    // 100000001` keeps running, and so does the same loop with
                    // the bound spelled `100000002.0`; at `FUZZ 0` both run no
                    // passes at all.
                    //
                    // **Matched on the three `Option`s together rather than
                    // threaded through a `?` chain**, which is codegen rather
                    // than style: written as an immediately-invoked closure
                    // returning `Option<(i64, i64, i64)>`, LLVM left the
                    // closure out of line and returned the tuple through
                    // memory. Measured with callgrind on
                    // `do i = 1 to 300000`, that line cost 53 instructions a
                    // pass, 16 of them in the closure's own body.
                    let within = match (current.small(digits), to_int, by_int) {
                        (Some(current), Some(to), Some(by)) if fuzz == 0 => {
                            if by < 0 {
                                current >= to
                            } else {
                                current <= to
                            }
                        }
                        _ => Self::controlled_within_wide(current, to, by, digits, fuzz)?,
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

    /// One controlled pass's step where the control variable is not an
    /// integer the tag holds, or the sum leaves what `DIGITS` admits.
    ///
    /// **Out of line so that its `Number` locals do not size
    /// [`Interp::loop_advance`]'s own frame**, which every pass pays for
    /// whether it comes here or not; the same reason `derived_name` is not
    /// an arm of `read_at`. Measured: with the three wide paths written
    /// inline, `loop_advance` allocated a 696-byte frame on every pass; with
    /// them out of line it allocates 248.
    #[inline(never)]
    fn controlled_step_wide(
        &mut self,
        previous: ObjRef,
        by: &Number,
        digits: u64,
    ) -> Result<Number, Failure> {
        // The control variable is the **left** operand of the oracle's own
        // implicit `+`, so an object assigned to it inside the body is 97.1
        // there -- measured, `do i = 1 to 3; i = .array; end` prints one
        // iteration and then raises.
        if let Some(kind) = self.operator_operand_gap(previous) {
            return Err(Loud::object_position("a controlled DO's control variable", kind).into());
        }
        let read = self.arith_operand(previous)?;
        read.add(by, digits)
            .map_err(Raised::from)
            .map_err(Failure::from)
    }

    /// The handle a controlled pass binds when [`exact_small_int`] declines
    /// the control value. Out of line for the reason
    /// [`Interp::controlled_step_wide`] gives.
    #[inline(never)]
    fn controlled_value_wide(
        &mut self,
        current: &ControlValue,
        digits: u64,
        form: rexx_num::Form,
    ) -> ObjRef {
        let number = current.number().into_owned();
        self.number(number, crate::eval::saturate_digits(digits), form)
    }

    /// A controlled loop's `TO` test where either side is wider than an
    /// `i64` comparison answers. Out of line for the reason
    /// [`Interp::controlled_step_wide`] gives.
    ///
    /// `signum`, not `numeric_less` against a zero built for the occasion:
    /// `numeric_order`'s first act is to compare the two operands' signs and
    /// answer from them alone whenever they differ, and one of them being
    /// zero is exactly that case, so neither `digits` nor `fuzz` can reach
    /// the answer. `a_negative_by_is_what_comparing_it_against_zero_says`
    /// holds the two against each other rather than this paragraph doing it.
    #[inline(never)]
    fn controlled_within_wide(
        current: &ControlValue,
        to: &Number,
        by: &Number,
        digits: u64,
        fuzz: u64,
    ) -> Result<bool, Failure> {
        let current = current.number();
        let within = if by.signum() < 0 {
            !numeric_less(&current, to, digits, fuzz).map_err(Raised::from)?
        } else {
            !numeric_less(to, &current, digits, fuzz).map_err(Raised::from)?
        };
        Ok(within)
    }

    /// Writes `value` into `control`'s own variable, through whichever of
    /// the three shapes (`shape_of`) its own spelling is.
    ///
    /// **Traces its own `>=>` (and, for a compound, the `>C>` ahead of it),
    /// at `indent`** (Task 9). Every write to a control variable is an
    /// assignment to the oracle and traces like one: `control->assign
    /// (context, result)` in both `DoBlock::checkOver` (`DoBlock.cpp:165`)
    /// and `DoBlock::checkControl` (`:197`), and again in a controlled
    /// loop's own setup. `indent` is the caller's, not this function's to
    /// derive, because the same write is traced at two different indents
    /// depending on which of those three events it is -- `loop_advance`'s
    /// own arms have the measured rule.
    ///
    /// The simple-variable case stays a direct slot write, with its own
    /// `intermediates` gate kept below it (see that arm's own comment for
    /// the measurement behind the gate). A stem or compound control instead
    /// goes through `assign_expr_target`'s own `Stem`/`Compound` arms --
    /// the same tail resolution, against `j`'s *current* value, that `say
    /// cv.j` already uses -- built here from a synthetic `Expr` around
    /// `control`'s own `SymbolId` rather than duplicated, since that is the
    /// oracle-verified logic an ordinary `cv.j = expr` assignment already
    /// runs. Measured, `j=7; do cv.j = 1 to 3; say cv.j; end`: the oracle
    /// prints `1`/`2`/`3` and this crate, before this fix, printed `CV.7`
    /// three times, because `Controlled::control` is a bare `SymbolId` and
    /// the old code ran every shape through the simple-variable slot write
    /// unconditionally, so a compound's tail was never resolved at all.
    ///
    /// `at` is [`control_slot`]'s answer for this loop, taken once when it was
    /// entered: the same resolution the `Simple` arm below makes for itself
    /// when it is `None`, and read by that arm alone. The other two arms hand
    /// [`Interp::assign_expr_target`] a literal `None` and let it find the slot
    /// on the entry `Code::compound` holds -- for the stem because forwarding
    /// this one costs every controlled loop 2 instructions a pass
    /// ([`control_slot`]'s own doc has the measurement), and for the compound
    /// because the slot it writes is the stem's rather than this symbol's.
    fn bind_control(
        &mut self,
        code: &Code<'_>,
        control: SymbolId,
        indent: usize,
        value: ObjRef,
        at: Option<usize>,
        shape: NameShape,
    ) -> Result<(), Failure> {
        debug_assert_eq!(
            shape,
            shape_of(code.symbols.name(control).as_bytes()),
            "a control variable's write was told a shape its name does not give"
        );
        match shape {
            NameShape::Simple => {
                // The tripwire `crate::ir::Op::Load` and `Op::Store` each carry,
                // on the one write that keeps its slot across passes rather
                // than reading it out of an op: a kept index that is not the
                // one this body's plan gives the name would write into another
                // variable's slot rather than fail.
                debug_assert!(
                    at.is_none() || control_slot(code, control) == at,
                    "a loop's kept control slot is not the one this body's plan gives its name"
                );
                let slot = match at {
                    Some(slot) => slot,
                    // The name is looked up here rather than above the
                    // `match`, so that a loop whose slot was resolved when it
                    // was entered -- every counted loop -- never asks for its
                    // own name again on a pass.
                    None => self.slot_of(code.symbols.name(control).as_bytes()),
                };
                let frame = self.activation().frame;
                self.set_variable(frame, slot, value);
                // `trace_assignment` carries its own `intermediates` gate, so
                // the check below is not a second decision about whether to
                // *print*: it decides whether to *build* the two `Vec`s, and
                // it exists because this function runs once per loop pass.
                //
                // **Kept on a measurement, not on a pattern.** A
                // 2,000,000-pass `do ii = 1 to 2000000` under `TRACE OFF`,
                // release build, three runs each: 3.13/3.13/3.15 s with this
                // check and 3.22/3.22/3.23 s without it -- about 40 ns per
                // pass, which is the two allocations. Small, real, and a
                // fixed number that stays true however the rest of the file
                // changes.
                //
                // Two earlier versions of this note argued from sibling
                // sites instead and were false both times (review round 1
                // F9, re-reviews NEW-5 and NEW-F1): first that other tracing
                // sites share the shape, then that this is the only one --
                // the second sentence quoted a search command whose own
                // search term it contained, so committing the evidence
                // changed the answer. Neither claim was load-bearing. Do not
                // restore either; if the question ever matters, the compiler
                // and the profiler answer it, and this comment should not
                // try to.
                if self.tracing_intermediates() {
                    let name = code.symbols.name(control).as_bytes().to_vec();
                    let rendered = self.to_text(value).to_vec();
                    self.trace_assignment(indent, &name, &rendered);
                }
                Ok(())
            }
            NameShape::Stem => {
                let target = Expr {
                    kind: ExprKind::Stem(control),
                    span: 0..0,
                };
                let rendered = self.intermediate_text(value);
                // `None`, and a literal one rather than the loop's kept
                // slot: `assign_expr_target`'s `Stem` arm finds this stem's
                // slot on its own entry, and a value here in place of the
                // constant costs every controlled loop 2 instructions a pass
                // (`control_slot`'s doc has the measurement).
                self.assign_expr_target(code, &target, value, rendered.as_deref(), indent, None)
            }
            NameShape::Compound => {
                let target = Expr {
                    kind: ExprKind::Compound(control),
                    span: 0..0,
                };
                let rendered = self.intermediate_text(value);
                // `None` because a compound writes one tail through a key
                // resolved on this pass, so the symbol's own slot is not what
                // is written and there is none to carry.
                self.assign_expr_target(code, &target, value, rendered.as_deref(), indent, None)
            }
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
    pub(crate) fn whole_nonneg(&mut self, value: ObjRef) -> Option<u64> {
        let digits = usize::try_from(self.activation().settings.digits()).ok()?;
        // **A tagged integer already is the answer**, when it is small enough
        // that `whole_value` would round nothing -- which is what `whole_i64`
        // decides, and `builtin::whole_number` takes the same shortcut against
        // its own fixed width. Going the long way builds a `Number` out of the
        // tag, digit by digit, and takes the `i64` straight back out of it. A
        // `None` here means only that the rounding rule has to run, so the
        // general path below still does.
        if let Decoded::SmallInt(small) = value.decode()
            && let Some(whole) = rexx_num::whole_i64(small, digits)
        {
            return u64::try_from(whole).ok();
        }
        let number = self.to_number(value).ok()?;
        let whole = number.whole_value(digits)?;
        u64::try_from(whole).ok()
    }

    /// An `IF`'s own condition, evaluated as the whole of the `IF` clause's
    /// work.
    ///
    /// **The tree-walker's entry, and the compiled stream's only for a
    /// condition `native_shape` declined**: `step`'s `If` arm calls it inside
    /// its own `in_clause`, and `Op::EvalExpr` calls it for the compiled
    /// form's `Clause` region. A condition that compiled does not come through
    /// here at all -- its ops leave the value in a register and
    /// `crate::ir::Op::Condition` enters [`Interp::condition_value`] with it,
    /// which is the half the two share. The indent it traces at is
    /// read from `current_value_indent` rather than recomputed, for the same
    /// reason `Assignment`'s own arm reads it: `in_stepped_clause` has already
    /// set it to this clause's own printed indent, and recomputing
    /// `static_indent(index)` would drop both the activation base and any
    /// escape elevation in force.
    fn eval_if_condition(&mut self, code: &Code<'_>, condition: &Expr) -> Result<bool, Failure> {
        let indent = self.clause_state.current_value_indent;
        self.eval_condition(
            code,
            condition,
            ConditionTrace::Result(indent),
            raised_if_not_logical,
        )
    }

    /// The expression an op's address names: expression `slot` of
    /// `instruction`, then `path`'s steps down from that slot's root.
    ///
    /// **The address is a slot and a route, not a slot alone**, which is what
    /// lets an op name a node *inside* an expression rather than only the
    /// whole of one. [`NodePath::ROOT`] is the whole of it, so a call that is
    /// the slot's own expression resolves with no descent at all.
    ///
    /// **The slot arms are the slots `compile` enters `push_native` for**, and
    /// that is the whole rule. An addressed op exists only where the whole
    /// slot compiled natively, so a slot `compile` never offers to
    /// `push_native` -- a `SELECT CASE`'s expression -- holds no node any op
    /// names and must never reach here.
    ///
    /// **These are not a subset of [`Interp::eval_chunk_expr`]'s arms, and
    /// containment is the wrong invariant to hold them to**, because the two
    /// functions answer different questions: this one resolves a call inside a
    /// slot that compiled, and that one evaluates a slot that declined. A slot
    /// is in both exactly when it is offered to `push_native` *and* its
    /// declining fallback is [`crate::ir::Op::EvalExpr`], and slots part
    /// company in both directions: a plain `WHEN`'s condition is here and not
    /// there, because a declining one stays on [`crate::ir::Op::WhenTest`]
    /// doing the whole job, and a `SELECT CASE`'s expression is there and not
    /// here, because it is never offered to `push_native` at all.
    ///
    /// `None` for a slot this does not name and for a step that lands on a
    /// node with no such child. Both are `Loud::call_op_off_its_node` at the
    /// caller, which is this crate's standing answer for a stream state that
    /// cannot arise rather than a panic.
    pub(crate) fn chunk_node_at(
        instruction: &Instruction,
        slot: u16,
        path: NodePath,
    ) -> Option<&Expr> {
        let mut node = match (&instruction.kind, slot) {
            (InstructionKind::Assignment { value, .. }, 0) => value,
            // The one expression of an instruction that computes a value and
            // does something with it, slot `0`. A bare form holds no
            // expression at all, so it names no node and matches no arm here.
            (
                InstructionKind::Say {
                    expression: Some(expression),
                }
                | InstructionKind::Return {
                    expression: Some(expression),
                }
                | InstructionKind::Exit {
                    expression: Some(expression),
                }
                | InstructionKind::Push {
                    expression: Some(expression),
                }
                | InstructionKind::Queue {
                    expression: Some(expression),
                },
                0,
            ) => expression,
            // The two instructions whose slot `0` exists only in one of their
            // forms. Both are reachable: the expression is compiled through
            // `push_value` like any other, so a call inside it emits an
            // `Op::CallExpr` that addresses the node from here.
            (InstructionKind::Signal(signal), 0) => match &**signal {
                rexx_parse::Signal::Value(expression) => expression,
                rexx_parse::Signal::Label(_) | rexx_parse::Signal::Trap(_) => return None,
            },
            (
                InstructionKind::Parse(parse)
                | InstructionKind::Arg(parse)
                | InstructionKind::Pull(parse),
                0,
            ) => match &parse.source {
                rexx_parse::ParseSource::Value(Some(expression)) => expression,
                _ => return None,
            },
            (InstructionKind::If { condition, .. }, 0) => condition,
            // A **plain** `WHEN`'s condition. A `WhenCase`'s values are not a
            // condition and compile to no native op at all, so no address ever
            // names one and this arm does not answer for them.
            (InstructionKind::When { condition, .. }, 0) => condition,
            // A `DO`/`LOOP` header's `slot`th expression, resolved through the
            // same `loop_header_slot` the compiler emitted the slot's ops from
            // and the same one `eval_chunk_expr` evaluates a declining slot
            // with. Every slot of a header is addressable, not just one, which
            // is why this arm binds `slot` rather than matching a number.
            (InstructionKind::Do(body) | InstructionKind::Loop(body), slot) => {
                loop_header_slot(body, u32::from(slot))?
            }
            _ => return None,
        };
        for right in path.steps() {
            node = match (&node.kind, right) {
                (ExprKind::Binary { left, .. }, false) => left,
                (ExprKind::Binary { right, .. }, true) => right,
                (ExprKind::Prefix { operand, .. }, false) => operand,
                _ => return None,
            };
        }
        Some(node)
    }

    /// Expression `slot` of `instruction`, evaluated as the compiled stream's
    /// [`crate::ir::Op::EvalExpr`] asks.
    ///
    /// `instruction` is the clause of the region the op sits in, handed over by
    /// the driver rather than looked up from the op's own index: the two are the
    /// same instruction by construction and `compile` asserts it.
    ///
    /// The answer is the expression's own Rexx value. An `If` has one
    /// expression, slot `0`, and its value is a logical one: `eval_condition`
    /// has already validated it as exactly `0` or `1`, so it is stored as the
    /// small integer of that name and `Op::JumpUnless` reads it back without
    /// repeating the validation. A `SELECT CASE` has one too, slot `0`, and
    /// its value is stored as it came: nothing branches on it, and every
    /// `WHEN CASE` of that `SELECT` compares its own values against this one's
    /// text.
    ///
    /// **Loud rather than a panic for every shape that is not one this
    /// stream emits**, which is this crate's standing rule for a state the
    /// type system admits and the compiler does not produce: a promotion that
    /// emits an `EvalExpr` for a third instruction adds the arm here that
    /// gives it meaning.
    pub(crate) fn eval_chunk_expr(
        &mut self,
        code: &Code<'_>,
        instruction: &Instruction,
        slot: u32,
    ) -> Result<ObjRef, Failure> {
        match (&instruction.kind, slot) {
            (InstructionKind::If { condition, .. }, 0) => {
                let holds = self.eval_if_condition(code, condition)?;
                // In range unconditionally: `SMALL_INT_MAX` is far above one.
                Ok(ObjRef::small_int(i64::from(holds)).unwrap_or(ObjRef::NIL))
            }
            (
                InstructionKind::Select {
                    case: Some(case_expr),
                    ..
                },
                0,
            ) => self.select_case(code, case_expr),
            // A `DO`/`LOOP` header's `slot`th expression, in the order
            // `loop_header_plan` puts them in -- the order they were written,
            // which is the order they are evaluated. The expression is
            // evaluated and nothing else: its `>K>` echo and its validation are
            // ops of their own, because both have to happen before the next
            // expression is evaluated at all.
            (InstructionKind::Do(body) | InstructionKind::Loop(body), slot) => {
                let Some(expr) = loop_header_slot(body, slot) else {
                    return Err(Loud::instruction(&instruction.kind).into());
                };
                self.eval(code, expr)
            }
            // An `Assignment`'s value, and the one expression of a `SAY`, a
            // `RETURN`, an `EXIT`, a `PUSH` or a `QUEUE`, slot `0`: whatever
            // the expression came to, unvalidated and untagged. Each reaches
            // this arm only for an expression `compile` did not emit a native
            // op for -- a literal is `crate::ir::Op::Const` and a bare symbol
            // is `crate::ir::Op::Load` instead -- and each is trace-identical
            // to the tree-walker's own arm here because this is the same
            // `eval` call it makes.
            // `SIGNAL VALUE`'s own expression, for the shapes `compile` emits
            // no native op for. The `>K>` echo and the search are
            // `Op::Signal`'s, not this arm's, exactly as a loop header's
            // validation is its own op.
            (InstructionKind::Signal(signal), 0) => {
                let rexx_parse::Signal::Value(expr) = &**signal else {
                    return Err(Loud::instruction(&instruction.kind).into());
                };
                self.eval(code, expr)
            }
            // `PARSE VALUE expr WITH`'s own expression, on the same terms. The
            // `>K>` echo and the whole template walk are `Op::Parse`'s; only
            // the source is a register, and only this source has one --
            // `compile` emits no register for any other, so reaching here with
            // one is an op off its node.
            (
                InstructionKind::Parse(parse)
                | InstructionKind::Arg(parse)
                | InstructionKind::Pull(parse),
                0,
            ) => {
                let rexx_parse::ParseSource::Value(Some(expr)) = &parse.source else {
                    return Err(Loud::instruction(&instruction.kind).into());
                };
                self.eval(code, expr)
            }
            (InstructionKind::Assignment { value, .. }, 0) => self.eval(code, value),
            (
                InstructionKind::Say {
                    expression: Some(expression),
                }
                | InstructionKind::Return {
                    expression: Some(expression),
                }
                | InstructionKind::Exit {
                    expression: Some(expression),
                }
                | InstructionKind::Push {
                    expression: Some(expression),
                }
                | InstructionKind::Queue {
                    expression: Some(expression),
                },
                0,
            ) => self.eval(code, expression),
            (kind, _) => Err(Loud::instruction(kind).into()),
        }
    }

    /// Evaluates `condition` and answers whether it holds, for `IF`/`WHEN`.
    ///
    /// **A comma list checks itself, and it does so before this function has
    /// the value.** `ExprKind::Logical` is evaluated through `eval`'s own
    /// dispatch to `eval_logical_list`, which checks that each element it
    /// evaluates is exactly `0`/`1`, raises 34.6 on the first that is not,
    /// stops at the first that is `0`, and otherwise answers `b"0"` or `b"1"`
    /// and nothing else. So the `checked` this hands
    /// [`Interp::condition_value`] spares a check that could not have failed,
    /// rather than one that would have raised the wrong number. A single,
    /// non-list expression never passes through `eval_logical_list` at all
    /// (there is no list to iterate), so nothing has checked it yet, and
    /// `raise` is the keyword-specific raiser for that case.
    ///
    /// **What the two numbers distinguish is where the raise happened, and
    /// that is measured**: `if 'x', 1 then` is 34.6 on the oracle, from inside
    /// the list and whichever element failed, while `if 'x' then` is 34.1,
    /// from the keyword.
    fn eval_condition(
        &mut self,
        code: &Code<'_>,
        condition: &Expr,
        trace: ConditionTrace<'_>,
        raise: fn(&[u8]) -> Raised,
    ) -> Result<bool, Failure> {
        let value = self.eval(code, condition)?;
        let checked = matches!(condition.kind, ExprKind::Logical(_));
        self.condition_value(value, trace, checked, raise)
    }

    /// Everything an evaluated condition value still owes: its `>>>` or `>K>`
    /// line, and the answer to whether it holds.
    ///
    /// **Split from the evaluation so that the tail is one implementation**,
    /// entered by any caller that has a condition value in hand however it
    /// came by it.
    ///
    /// `checked` is whether whatever produced `value` has already validated it
    /// as exactly `0`/`1`, in which case the answer is read back rather than
    /// checked again. `raise` is the keyword-specific raiser for the unchecked
    /// case, chosen by whichever caller had the condition in hand.
    ///
    /// **The flag decides an answer for the compiled caller and not for the
    /// tree-walker.** `crate::ir::Op::Condition` hands over a value it read out
    /// of a register with nothing having validated it, so it passes `false`;
    /// measured, a driver that passed `true` there answers `if 'x' then` false
    /// instead of raising 34.1. [`Interp::eval_condition`] passes `true` only
    /// for a comma list, and `eval_logical_list` answers `b"0"` or `b"1"` and
    /// nothing else, so the check it skips is one that could not have failed.
    /// Measured: forcing that `true` to `false` leaves the whole workspace
    /// green, which is what says the tree-walker's half of this flag is a
    /// spared check rather than a different answer.
    pub(crate) fn condition_value(
        &mut self,
        value: ObjRef,
        trace: ConditionTrace<'_>,
        checked: bool,
        raise: fn(&[u8]) -> Raised,
    ) -> Result<bool, Failure> {
        // **The test's own temps frame, and it is a per-pass frame for the two
        // callers that are loop headers.** `WHILE` and `UNTIL` re-evaluate
        // their condition once per pass inside the enclosing `DO`
        // instruction's single frame, so the push below accumulates one
        // `ObjRef` per pass -- and one pinned heap object per pass whenever
        // the condition's result is one -- for the whole of the loop's run.
        // The value is never handed back (this answers a `bool`), so a frame
        // closed here releases it at the right time for every caller,
        // `IF`/`WHEN` included. The `?` path below leaves the pop to the outer
        // truncation, as `pop_frame`'s own doc describes.
        let frame = self.roots.push_frame();
        self.roots.push_temp(value);
        // **The owned copy is taken only when a line will print it.** Both
        // formatters below return at once unless `results` is on, and the copy
        // exists only because `to_text` borrows `self` while they need it
        // mutably -- so off that path the borrow is enough and the answer is
        // decided from it directly. Measured with `heaptrack` on
        // `samples/rexxcps.rex`: this was the largest single allocation site
        // in the interpreter, one copy per condition evaluated.
        let decided = if self.trace_mode().results {
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
            Self::condition_holds(checked, &text, raise)
        } else {
            // **Off the tracing path the bytes need not be moved anywhere.**
            // `to_text` would copy them into the interpreter's scratch slot
            // and hand back a borrow of it; for a value that carries its own
            // bytes that copy buys nothing. A condition's value is one byte --
            // `0` or `1` -- far more often than it is anything else, and one
            // byte always rides in the handle.
            match value.decode() {
                Decoded::Text(inline) => Self::condition_holds(checked, &inline, raise),
                // `to_text` renders these two as `"0"` and `"1"`, which both
                // arms of `condition_holds` then read as this same answer.
                // Every other integer falls to the arm below, so the failure
                // it raises still names the value as the oracle spells it.
                Decoded::SmallInt(number) if number == 0 || number == 1 => Ok(number == 1),
                _ => {
                    let text = self.to_text(value);
                    Self::condition_holds(checked, &text, raise)
                }
            }
        };
        // After the rendering above is out of scope, so the borrow it may hold
        // on `self` has ended. The decision itself touches neither `self` nor
        // the roots, so making it before this pop rather than after changes no
        // answer and no lifetime.
        self.roots.pop_frame(frame);
        decided
    }

    /// Whether a condition's rendered text holds, and the failure when it is
    /// neither `0` nor `1`.
    ///
    /// A free function rather than a method because both arms of
    /// [`Interp::condition_value`] call it while a rendering borrows `self` --
    /// which is the whole point of the arm that does not copy.
    fn condition_holds(
        checked: bool,
        text: &[u8],
        raise: fn(&[u8]) -> Raised,
    ) -> Result<bool, Failure> {
        if checked {
            Ok(text == b"1")
        } else {
            logical_value(text).ok_or_else(|| raise(text).into())
        }
    }

    /// Whether any of a `WHEN CASE`'s `values` compares `==` (byte-for-byte,
    /// no padding, no numeric awareness) equal to the `SELECT CASE`'s own
    /// `case_text`, matching on the first that does (an OR of `==`, the
    /// opposite of a plain `WHEN`'s comma list, which is an AND checked for
    /// `0`/`1` -- `ast.rs`'s own doc comment on `WhenCase`).
    ///
    /// **Reasoned rather than routed through `apply_binary`'s own
    /// `Operator::StrictEqual`, and this is why.** The design's own
    /// "Expression evaluation" section states the strict family's rule in
    /// full: "there is no padding and the shorter string is less" -- for an
    /// *ordering* comparison. Equality has no "less" to fall back on, so
    /// under that same rule two strict operands are equal if and only if
    /// they are the same length and every byte matches, which is exactly
    /// `==` on the two `Vec<u8>`s below and needs no numeric awareness or
    /// `rexx-num` call to compute. Measured, matching D15's own example:
    /// `select case '007'` does not match `when 7`, because `"007"` and
    /// `"7"` are not byte-identical. Calling `apply_binary` would answer the
    /// same and is reachable from here, but only one side of this comparison
    /// is a value: it would mean allocating one to hold `case_text` so that
    /// `compare_values` could render it straight back to the bytes already
    /// in hand.
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
    /// # The outward direction, measured
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
    /// Both are produced. The `LEAVE`'s own clause is the innermost entry:
    /// `leave_origin` resolves a real site because this function passes
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
        // it. It is an `Rc` because an `INTERPRET` inside a fragment makes this
        // function reentrant and each level anchors its own.
        let slots = self.fragment_plan(&fragment);
        let code = Code {
            body: &fragment.body,
            symbols: &fragment.symbols,
            slots: &slots,
            // A fragment carries no plan of its own -- `fragment_plan` keeps
            // only the id-to-enclosing-slot translation out of the one it
            // builds, for the reason `Code::plan` gives -- so its clause
            // indents and its compound splits are both computed the way they
            // were before either table existed.
            plan: None,
        };

        // `exit` inside `INTERPRET` ends the program, not the fragment, so
        // it has to propagate rather than stop here -- `run_bounded`'s own
        // catch-all does exactly that for anything it does not own, `Exit`
        // included, with nothing fragment-specific to add.
        //
        // **`Some(&fragment.source)`.** The fragment
        // resolves its own clauses: its spans are the only thing that
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
            // A fragment compiles to no chunk, and its `Code` is a different
            // body from the one an enclosing chunk's `op_of` indexes.
            BodyEngine::TreeWalker,
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
    /// level** -- `run_fragment` and `Interp::invoke_call`. The rule is the
    /// same for both: seal before the failure
    /// leaves the callee, never after. Sealing a level that recorded nothing
    /// is a no-op, which is what gives a fragment that failed to parse one
    /// echo instead of two.
    ///
    /// **Nothing here clears either field, and that is deliberate**: this
    /// function is the *unwinding* half, and a level that seals still has a
    /// report to give. Clearing is the *trapping* half, which is
    /// `offer_to_trap`'s (inherited item I11) -- it empties both
    /// the slot and this stack, because a trapped condition prints no report
    /// at all and its sites must not survive to be printed against a later,
    /// untrapped one. The two-raise transcript in
    /// `a_second_raise_after_a_trapped_one_reports_its_own_site` is what
    /// observes that.
    pub(crate) fn seal_site_level(&mut self) {
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
                    let (stem_name, stem_at) = code.stem(*id);
                    let key = self.tail_key(code, *id);
                    self.stem_drop_tail_at(stem_name, stem_at, &key);
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
                self.clear_variable(frame, slot);
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
    /// `SCIENTIFIC`. This crate has no `::OPTIONS` to move the package default
    /// away from `Scientific`, which is why `FormDefault` and `FormScientific` do
    /// the identical thing below; a later phase's `::OPTIONS FORM` is what
    /// would make the two differ, and should split this arm rather than
    /// assume they stay equal.
    /// `TRACE`'s four forms (D17). `Trace::Default` (bare `TRACE`) and the
    /// `Trace::Setting` letters that are recognised but have nothing visible
    /// to show in this crate's scope (`C`/`E`/`F`/`N`/`O`) are all silent
    /// (measured: `trace` alone and `trace value 'N'` produce no output).
    /// They are still six distinct settings rather than one, because
    /// `TRACE()` reports which was asked for -- `TraceMode::letter` has that
    /// measurement, and bare `TRACE` is `NORMAL` rather than `OFF`.
    ///
    /// **`L` is not in that list** and was until Task 9's review round 1:
    /// it lands on `TraceMode::LABELS` and echoes every executed `LABEL`
    /// clause. Corrected at the re-review (NEW-2), which found this arm
    /// still naming the old answer one commit after the behaviour changed.
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
                // `setTraceNormal`, which is silent here and is *not*
                // `TRACE OFF` -- measured, `trace r` then bare `trace` then
                // `trace()` gives `N`, where `trace off` gives `O`.
                self.set_trace_mode(crate::trace::TraceMode::NORMAL);
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
        // `RexxActivation::setTrace` calls `traceEntry()` right after
        // installing the new flags (`RexxActivation.cpp:1024`), which is the
        // route every 4c-reachable `>I>` takes. `Trace::Skip` never gets here
        // -- it returned above -- and the C++ agrees: its own arm sets no
        // flags and calls nothing.
        self.trace_invocation_entry();
        Ok(())
    }

    /// `>I>`, if this activation is a `::ROUTINE` still on its first
    /// instruction and the setting just installed traces labels.
    ///
    /// **The gate is two predicates that no single field expresses**,
    /// `tracingLabels() && isMethodOrRoutine()` (`RexxActivation.cpp:3655`),
    /// plus the once-only pair on the activation. Each half is measured on
    /// its own:
    ///
    /// * `tracingLabels()` -- the routine's own `trace` instruction fires
    ///   these lines for exactly **A, I, L and R**, verified by running the
    ///   same routine under all nine accepted letters; `n`, `c`, `e`, `f` and
    ///   `o` produce zero stderr. `TraceMode::labels` is that predicate.
    /// * `isMethodOrRoutine()` -- a main body never announces one. Measured,
    ///   `trace l` as a program's own first clause emits nothing, and
    ///   `::options trace labels` in a file whose only code is a main body
    ///   likewise.
    /// * `traceEntryAllowed` -- the `trace` must be the routine's **first**
    ///   instruction. Measured, a routine whose first clause is `n0 = 0` and
    ///   whose second is `trace r` echoes its clauses and announces nothing.
    /// * `traceEntryDone` -- once per activation, and the second of two calls
    ///   into the same routine announces its own pair, not a third line
    ///   (measured, `call rtn` twice gives `>I>`/`<I<` twice).
    ///
    /// **The caller's setting is not one of the halves and cannot be.**
    /// Measured, `trace l` in a caller targeting a routine emits nothing at
    /// all -- a routine inherits no `TraceMode`, so the only setting this
    /// ever reads is one the routine itself installed.
    ///
    /// **A dynamic `TRACE VALUE` reaches this too**, and that is the C++'s
    /// own second route rather than an accident: `earlyTraceEntry`'s code
    /// analysis (`:3630`-`:3641`) demands a *non-dynamic* `TRACE`, but
    /// `setTrace` calls `traceEntry` unconditionally, and by then the flags
    /// are installed and `traceEntryAllowed` is still true. Measured,
    /// `trace value 'l'` as a routine's first clause announces both lines.
    /// This crate implements the `setTrace` route only; the code-analysis
    /// route exists in the C++ to announce the entry *before* a guarded
    /// method takes its object lock, and with no methods and no locks here
    /// nothing can run between the two points, so no probe separates them.
    ///
    /// **`::OPTIONS TRACE LABELS` is a third route and this crate does not
    /// implement it.** Measured, a routine with no `trace` instruction of its
    /// own, in a file carrying `::options trace labels`, announces both lines
    /// with identical bytes. `::OPTIONS` is a declared Phase 5 gap
    /// (`directive_gap`, `lib.rs`), so such a program is refused here rather
    /// than running without the lines.
    fn trace_invocation_entry(&mut self) {
        if !self.activation().trace_entry.may_announce() {
            return;
        }
        let Some(subject) = self.invocation_subject() else {
            return;
        };
        if !self.trace_mode().labels {
            return;
        }
        self.activation_mut().trace_entry = TraceEntry::Done;
        let package = self.program_path.clone().into_bytes();
        self.trace_invocation(">I>", &subject, &package);
    }

    /// `<I<`, on every way a routine activation can end.
    ///
    /// Called with the callee still on the activation stack, because both
    /// halves of the gate are read off it. Measured on all four endings, and
    /// all four announce it: `return`, `exit`, falling off the routine's own
    /// end, and an untrapped condition -- where the line lands **before** the
    /// error report's own clause echoes, which is the order this crate
    /// produces anyway since the report is written at the very end.
    ///
    /// `tracingLabels()` is re-read here rather than assumed from
    /// the `Done` state: measured, a routine whose body is `trace l` then
    /// `trace off` announces `>I>` and no `<I<`.
    pub(crate) fn trace_invocation_exit(&mut self) {
        if self.activation().trace_entry != TraceEntry::Done {
            return;
        }
        let Some(subject) = self.invocation_subject() else {
            return;
        };
        if !self.trace_mode().labels {
            return;
        }
        let package = self.program_path.clone().into_bytes();
        self.trace_invocation("<I<", &subject, &package);
    }

    /// Gives a fragment its own `>I>` count, and answers with the enclosing
    /// state for [`Interp::leave_fragment`] to put back.
    ///
    /// **A fragment is a separate activation in the C++ and is not one here**,
    /// which is the whole reason this is two functions rather than the plain
    /// decay every other nested construct gets. `RexxActivation.cpp:3644`-
    /// `:3652` gates an interpret activation's own announcement on
    /// `tracingLabels() && parent->isMethodOrRoutine() &&
    /// parent->traceEntryAllowed && !parent->traceEntryDone`, so the fragment
    /// counts its own clauses from zero, but only when the enclosing
    /// `INTERPRET` was itself the routine's first clause.
    ///
    /// `nested` is the `isMethodOrRoutine()` half: an interpret activation is
    /// neither, so a fragment inside a fragment can never announce however
    /// its own clauses fall. Measured, and it is the row that separates this
    /// from a plain reset -- `interpret "interpret 'trace l'"` as a routine's
    /// only clause writes zero bytes on the oracle, where
    /// `interpret "trace l"` writes both lines.
    fn enter_fragment(&mut self, nested: bool) -> TraceEntry {
        let enclosing = self.activation().trace_entry;
        let entry = if enclosing.may_announce() && !nested {
            TraceEntry::Pending
        } else {
            TraceEntry::Spent
        };
        self.activation_mut().trace_entry = entry;
        enclosing
    }

    /// Puts the enclosing state back after a fragment, **except** that a
    /// fragment which announced leaves the activation `Done`.
    ///
    /// That exception is `RexxActivation.cpp:3664`, `if (isInterpret())
    /// parent->traceEntryDone = true`, and it is what the `<I<` owes its
    /// existence to: measured, `interpret "trace l"` as a routine's only
    /// clause emits `>I>` **and** `<I<`, so the announcement has to survive
    /// the fragment it happened in.
    fn leave_fragment(&mut self, enclosing: TraceEntry) {
        if self.activation().trace_entry != TraceEntry::Done {
            self.activation_mut().trace_entry = enclosing;
        }
    }

    /// What the running activation announces itself as, or `None` when it
    /// announces nothing at all -- which is the `isMethodOrRoutine()` half of
    /// the gate, expressed as the lookup that would supply the substitutions.
    ///
    /// A `::ROUTINE` names itself by the **directive's** own spelling,
    /// verbatim. Measured: `::routine 'zork'` announces `"zork"` and
    /// `::routine MiXeD` announces `"MIXED"`, the second because the scanner
    /// upcases a bare symbol before the directive parser sees it.
    ///
    /// A `::METHOD` names itself by the **message** name and its defining
    /// scope instead, both taken from `Activation::method_identity` --
    /// see [`MethodIdentity`] for why neither is read off the directive.
    fn invocation_subject(&self) -> Option<Announced> {
        if let Some(identity) = &self.activation().method_identity {
            return Some(Announced::Method {
                name: identity.name.to_vec(),
                scope: self.class_id_text(identity.scope).as_bytes().to_vec(),
            });
        }
        let index = self.activation().body?;
        match &self.activation().program.directives.get(index)?.kind {
            DirectiveKind::Routine(routine) => Some(Announced::Routine {
                name: routine.name.to_vec(),
            }),
            _ => None,
        }
    }

    /// `ADDRESS`'s three environment-naming forms. The caller has already
    /// turned the command and `WITH` forms away.
    ///
    /// # The two forms upcase differently, and it is the cheap thing to get
    /// wrong
    ///
    /// `rexx-parse` has already resolved this: `environment` is a symbol's
    /// *upcased* spelling or a literal's verbatim bytes (`ast::Address`'s own
    /// doc), so nothing here folds case. Measured on the oracle, four
    /// spellings of the same intent:
    ///
    /// ```text
    /// address envC                    ->  ENVC
    /// address 'LiTeRaL'               ->  LiTeRaL
    /// nm = 'mIxEd'; address value nm  ->  mIxEd
    /// nm = 'mIxEd'; address (nm)      ->  mIxEd
    /// ```
    ///
    /// So the computed forms never upcase, and the constant one does only
    /// because a symbol token is already upcased when it is read.
    ///
    /// # Order of operations on the computed form
    ///
    /// Evaluate, trace the value, *then* validate the length -- the same
    /// order `RexxInstructionAddress::execute` has (`traceResult` precedes
    /// `SystemInterpreter::validateAddressName`), and measured: under `trace
    /// r` a 251-byte name traces its own `>>>` line and only then raises
    /// 29.1. The value line is `>>>` under `TRACE I` as well as under
    /// `TRACE R`, measured -- unlike a `PARSE` target's, it is not a choice
    /// of prefix between the two modes.
    fn exec_address(
        &mut self,
        code: &Code<'_>,
        address: &rexx_parse::Address,
    ) -> Result<(), Failure> {
        // `environment` before `dynamic`, mirroring the C++'s own `if
        // (environment != OREF_NULL)` ahead of its `ADDRESS VALUE` arm. The
        // parser never fills both, so the order decides nothing today.
        match (&address.environment, &address.dynamic) {
            (None, None) => {
                self.activation_mut().address.toggle();
                return Ok(());
            }
            (Some(environment), _) => {
                if environment.len() > MAX_ADDRESS_NAME_LENGTH {
                    return Err(Raised::environment_name_too_long(
                        MAX_ADDRESS_NAME_LENGTH,
                        environment,
                    )
                    .into());
                }
                self.activation_mut().address.set_bytes(environment);
            }
            (None, Some(expression)) => {
                let value = self.eval(code, expression)?;
                self.roots.push_temp(value);
                // Built in the lent buffer: the rendering exists only to be
                // traced and then copied into the `Rc`, so an owned `Vec` of
                // its own would be an allocation and a free per `ADDRESS
                // VALUE` clause.
                let mut text = self.take_result_buffer();
                text.extend_from_slice(&self.to_text(value));
                self.trace_result(self.clause_state.current_value_indent, &text);
                if text.len() > MAX_ADDRESS_NAME_LENGTH {
                    let raised = Raised::environment_name_too_long(MAX_ADDRESS_NAME_LENGTH, &text);
                    self.give_result_buffer(text);
                    return Err(raised.into());
                }
                self.activation_mut().address.set_bytes(&text);
                self.give_result_buffer(text);
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
                match self.numeric_operand(code, expression, "DIGITS")? {
                    Some(text) => {
                        let parsed = String::from_utf8_lossy(&text);
                        let outcome = self.activation_mut().settings.set_digits_str(&parsed);
                        self.give_result_buffer(text);
                        outcome.map_err(raised_from_settings)?;
                    }
                    // The reset stores the default and makes the operand
                    // form's FUZZ check against it, which is the arm the
                    // interpreter runs: it can fail, and the value it names
                    // is the default rather than the setting in force.
                    None => self
                        .activation_mut()
                        .settings
                        .reset_digits()
                        .map_err(raised_from_settings)?,
                }
            }
            NumericSetting::Fuzz => match self.numeric_operand(code, expression, "FUZZ")? {
                Some(text) => {
                    let parsed = String::from_utf8_lossy(&text);
                    let outcome = self.activation_mut().settings.set_fuzz_str(&parsed);
                    self.give_result_buffer(text);
                    outcome.map_err(raised_from_settings)?;
                }
                None => self.activation_mut().settings.reset_fuzz(),
            },
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
    /// **The lent result buffer, and the caller hands it back.** The operand
    /// used to be copied into an owned `Vec` and then copied again into an
    /// owned `String` for `set_digits_str`, which takes `&str`. The buffer is
    /// taken *after* the expression is evaluated, so a builtin call inside
    /// that expression gets the buffer for its own result first.
    /// `from_utf8_lossy` borrows in the caller for every operand that is
    /// valid UTF-8, which is every operand that can parse as a count.
    fn numeric_operand(
        &mut self,
        code: &Code<'_>,
        expression: &Option<Expr>,
        keyword: &str,
    ) -> Result<Option<Vec<u8>>, Failure> {
        let Some(expression) = expression else {
            return Ok(None);
        };
        let value = self.eval(code, expression)?;
        self.roots.push_temp(value);
        let mut text = self.take_result_buffer();
        text.extend_from_slice(&self.to_text(value));
        self.trace_keyword(self.clause_state.current_value_indent, keyword, &text);
        Ok(Some(text))
    }

    /// `instruction`'s own clause text and the 1-based line to print it against,
    /// or `None` when `source` is `None`.
    ///
    /// **`Interp::clause_line_override` is why this is a method and not a free
    /// function.** The line and the text do not always come
    /// from the same place: inside an `INTERPRET` fragment the text is the
    /// fragment's (its spans index the fragment's own source, and nothing else
    /// can resolve them) while the line is the enclosing `INTERPRET` clause's,
    /// measured. Threading that override through the four call sites --
    /// `step_in_temps_frame`, `record_failure_at`, `leave_origin`,
    /// `run_otherwise` -- would give each of them a parameter about a construct
    /// none of them otherwise knows exists, so it reads the field instead,
    /// exactly as `current_value_indent` and `indent_offset` already do.
    ///
    /// `source: None` has no caller: `run_fragment` passes
    /// `Some(&fragment.source)`, which is what gives the report an echo
    /// per level. The parameter is still an `Option` because collapsing it is a
    /// mechanical change across every signature that threads it, which is a
    /// restructuring rather than this task's -- **but nothing below may assume a
    /// site is unresolvable any more**, and the comments that used to say so
    /// have been corrected rather than left standing.
    pub(crate) fn clause_site(
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
    pub(crate) fn clause_line(
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

    /// [`Interp::clause_line`] for a caller that knows the instruction's
    /// **index**, which is what lets the answer come from `Plan::lines`
    /// rather than from a search.
    ///
    /// The same rule as `clause_line` and the same `clause_line_override`
    /// honoured the same way, so a fragment's clauses keep reporting the
    /// enclosing `INTERPRET` clause's line; `Plan::line_at` is only reached
    /// once the override has declined, and a body with no plan takes the
    /// search exactly as it did before the table existed -- the shape
    /// `printed_indent` already has for `Plan::indents`.
    ///
    /// **`index` and `instruction` must name the same clause**, since the
    /// first indexes the table and the second supplies the span the fallback
    /// and the tripwire search on. Every caller derives one from the other,
    /// and the `debug_assert!` says so rather than trusting it: the two
    /// coming apart is a wrong line number, which is silent in every program
    /// that neither raises nor traces.
    pub(crate) fn clause_line_at(
        &self,
        code: &Code<'_>,
        index: usize,
        instruction: &Instruction,
        source: Option<&ProgramSource>,
    ) -> Option<usize> {
        let source = source?;
        if let Some(line) = self.clause_line_override {
            return Some(line);
        }
        debug_assert!(
            code.body
                .instructions
                .get(index)
                .is_some_and(|at| std::ptr::eq(at, instruction)),
            "clause_line_at was given index {index} and an instruction that is not the one at \
             that index, so the table would be read for a different clause"
        );
        Some(match code.plan {
            Some(plan) => plan.line_at(instruction, source, index),
            None => source.line_of(instruction.clause_span.start),
        })
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
/// not close together.** Closing the value lines -- the `BY` increment in
/// `loop_advance` -- does not close this indent, which is not this function's
/// to fix. **What matters here is that the
/// paragraph above reads as settled and is not**, so a later reader does not
/// build on it: this function computes the *lexical* indent, and closing the
/// gap means modelling the oracle's counter rather than making this function
/// impure.
///
/// `the_indent_after_a_loop_has_already_exited_is_not_left_over_from_it`
/// (`run/tests.rs`) does not catch it, and the reason is worth
/// keeping: it runs at top level, where the oracle's counter is already at 0
/// and cannot go lower. That is the same "at indent 0 the base is 0" blind
/// spot that hid two of four mutations in one round.
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
/// is_not_left_over_from_it` (`run/tests.rs`) is the test that would
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
/// Fills `out[i]` with the static clause indent of every position in
/// `[start, end)`, in one walk of the range.
///
/// The same traversal [`indent_in_range`] performs to answer for a single
/// target, doing every position at once. That function answers one index by
/// walking from the start of the body, so asking it for all of them costs
/// the body's length squared; this costs the body's length.
///
/// **Every arm is the same arm, transcribed.** Each `return k` there becomes
/// one assignment of `base + k` here, and each `return k +
/// indent_in_range(a, b, target)` becomes a recursive fill of `[a, b)` at
/// `base + k`. Where that function decides between an equality case and a
/// range case, this one fills the range first and then writes the equality
/// case over it -- the equality checks come first there, so they must win
/// here. `whens` is walked in reverse for the same reason: that function
/// answers from the *first* matching entry, and a later fill would otherwise
/// overwrite an earlier one.
///
/// A transcription is a second statement of a dozen separately measured
/// oracle behaviours, and it can be wrong where the original is right.
/// `every_corpus_program_fills_the_indents_static_indent_computes` is what
/// holds them together, over every program in the corpus rather than over
/// examples chosen here.
fn fill_indents(
    instructions: &[Instruction],
    start: usize,
    end: usize,
    base: usize,
    out: &mut [usize],
) {
    let len = instructions.len();
    let mut pc = start;
    while pc < end {
        out[pc] = base;
        match &instructions[pc].kind {
            InstructionKind::If { false_target, .. } => {
                let false_target = false_target.unwrap_or(len);
                let then_start = pc + 1;
                fill_indents(
                    instructions,
                    then_start,
                    false_target.min(len),
                    base + 4,
                    out,
                );
                if then_start < len {
                    out[then_start] = base + 2;
                }
                match instructions.get(false_target).map(|i| &i.kind) {
                    Some(InstructionKind::Else { then_exit }) => {
                        let else_end = then_exit.unwrap_or(len);
                        fill_indents(
                            instructions,
                            false_target + 1,
                            else_end.min(len),
                            base + 4,
                            out,
                        );
                        out[false_target] = base + 2;
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
                fill_indents(instructions, body_start, end_index.min(len), base + 2, out);
                // `END`'s own position: `indent_in_range` skips past it
                // (`pc = end_index + 1`) and so answers for it from the
                // enclosing level, which is what aligns an `END` with its
                // `DO`.
                if end_index < len {
                    out[end_index] = base;
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
                // The arm's own fallback, written first so the shapes below
                // overwrite it: a position inside a `SELECT` that matches
                // none of them keeps the enclosing level rather than
                // asserting anything about how it got there.
                for slot in out.iter_mut().take(select_end.min(len)).skip(pc + 1) {
                    *slot = base;
                }
                if let Some(otherwise_index) = otherwise {
                    fill_indents(
                        instructions,
                        otherwise_index + 1,
                        select_end.min(len),
                        base + 4,
                        out,
                    );
                    out[*otherwise_index] = base + 2;
                }
                for &when_index in whens.iter().rev() {
                    let (body_start, body_end) = match &instructions[when_index].kind {
                        InstructionKind::When { false_target, .. }
                        | InstructionKind::WhenCase { false_target, .. } => {
                            (when_index + 1, false_target.unwrap_or(len))
                        }
                        _ => continue,
                    };
                    fill_indents(instructions, body_start, body_end.min(len), base + 6, out);
                    if body_start < len {
                        out[body_start] = base + 4;
                    }
                    out[when_index] = base + 2;
                }
                pc = select_end;
                continue;
            }
            _ => {}
        }
        pc += 1;
    }
}

/// Every position's static clause indent, in one walk.
pub(crate) fn all_indents(instructions: &[Instruction]) -> Box<[usize]> {
    let mut out = vec![0usize; instructions.len()];
    fill_indents(instructions, 0, instructions.len(), 0, &mut out);
    out.into_boxed_slice()
}

pub(crate) fn static_indent(instructions: &[Instruction], target: usize) -> usize {
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
pub(crate) enum NameShape {
    Simple,
    Stem,
    Compound,
}

/// The frame slot a loop's control variable writes and re-reads, resolved
/// **once, when the loop is entered**, or `None` for a control this cannot
/// answer for.
///
/// **Simple spellings only, and a stem control and a compound control are
/// declined for different reasons.**
/// A compound control resolves a tail key afresh on every pass --
/// measured, `a.=0; i=1; Do a.i=1 To 3; If i>7 Then Leave; i=i+1; End; say i`
/// answers `8`, because the body moves which tail the control is -- so there
/// is no slot to resolve ahead of the pass that uses it.
/// A stem control **does** have one, and it is taken: `Plan::bind` puts its
/// spelling and its id on a single slot, and records that slot on the entry
/// `Code::compound` hands back, which is where `bind_control`'s stem arm and
/// the re-test's stem read each take it from. `None` here says the loop does
/// not need to carry it, not that there is nothing to carry.
///
/// **Carrying it here was measured, and it costs more than it saves.**
/// Forwarding this answer from `bind_control`'s stem arm into
/// `Interp::assign_expr_target` replaces a compile-time `None` at that call
/// site with a value, and **2 instructions then appear on every pass of every
/// controlled loop**, simple controls included, which never enter that arm at
/// all: `perf stat -e instructions:u`, `do i = 1 to 25000000; nop; end` at
/// +50,000,000 and `do i = 1 to 19000000` with two simple assignments at
/// +38,000,000, where the same binary against itself spans 1,348 and 1,576.
/// **Attributed by partial revert and not by reading the assembly**: undoing
/// that one line and nothing else returns the first to a figure
/// indistinguishable from base -- 27,150,813,214 against a base of
/// 27,150,813,500, inside that axis's own span -- and recomputing the slot
/// inside the arm instead costs +100,000,000. Why the generated code changes
/// was not established. Taking the slot from the entry
/// leaves both axes where they were and keeps the stem loop's own saving.
///
/// **Declining the compound is unobservable, and it is written down as such
/// rather than defended as a guard.** `bind_control` and the re-test both
/// select their arms by the same `shape_of`, and neither compound arm reads
/// this answer, so a slot resolved for a compound control would be computed
/// and discarded. Measured: answering for a compound control as well leaves
/// the whole workspace suite green, this crate's own loop tests included. What
/// it buys is that the value this function returns means what its name says at
/// every call site, not that anything downstream is stopped.
///
/// **It reads the plan's own map and nothing else, which is what makes it
/// free of side effects.** `Interp::slot_of` would *grow* the frame for a
/// name nobody has bound, and doing that at loop entry rather than at the
/// first write would bind a name earlier than the interpreter does. `None`
/// leaves both the write and the re-read resolving their own slot exactly
/// as they did before this existed.
///
/// The answers agree because they come from one map: `Plan::bind` gives a
/// symbol the slot its *name* already has, so `by_symbol[id]` and
/// `slot_of(name)` cannot disagree for a name the plan holds.
fn control_slot(code: &Code<'_>, control: SymbolId) -> Option<usize> {
    match shape_of(code.symbols.name(control).as_bytes()) {
        NameShape::Simple => code.slot_for(control),
        NameShape::Stem | NameShape::Compound => None,
    }
}

pub(crate) fn shape_of(name: &[u8]) -> NameShape {
    let dots = name.iter().filter(|&&b| b == b'.').count();
    if dots == 0 {
        NameShape::Simple
    } else if dots == 1 && name.last() == Some(&b'.') {
        NameShape::Stem
    } else {
        NameShape::Compound
    }
}

/// Splits an indirect wrapper's value into its subsidiary list's words --
/// `DROP (v)`, `EXPOSE (v)` and `PROCEDURE EXPOSE (v)` all spell the same
/// list and reach the same split.
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

/// Validates one word of an indirect subsidiary list and answers its upcased
/// name, or the condition the oracle raises for it.
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

/// What [`Interp::run_bounded`]'s absorption rule says about one clause's
/// `Flow`, in a range bounded by `[start, end]`.
///
/// **One rule, two loops.** The instruction-level loop and the op-level one
/// both decide through [`absorb`] and differ only in what they do with the
/// answer: an `Advance` moves an instruction counter by one or an op counter
/// to the op after the clause, and a `Resume` maps its instruction index
/// through the chunk's own table in the op case. Writing the range test twice
/// is how the two would come to disagree about which `Goto` is an escape.
pub(crate) enum Absorbed {
    /// Nothing to redirect: continue after the clause that produced it.
    Advance,
    /// An in-range `Flow::Goto`: continue at this instruction.
    Resume(usize),
    /// Not this range's: hand it back to the caller unchanged.
    Escaped(Flow),
}

/// [`Absorbed`] for `flow` in the range `[start, end]`.
///
/// `end` is inclusive, and deliberately: a nested construct's own resume point
/// landing exactly on this range's boundary is normal completion, not an
/// escape. Everything that is not a `Next` or an in-range `Goto` escapes,
/// including a `Flow` variant this function does not name -- which is the
/// same case as an out-of-range `Goto` on purpose, so a new variant
/// propagates outward rather than being silently mishandled by a wrong arm.
pub(crate) fn absorb(flow: Flow, start: usize, end: usize) -> Absorbed {
    match flow {
        Flow::Next => Absorbed::Advance,
        Flow::Goto(target) if target >= start && target <= end => Absorbed::Resume(target),
        other => Absorbed::Escaped(other),
    }
}

/// Where an `IF` sends control on each of its two paths.
///
/// **One computation, read by both engines.** `step`'s own `If` arm runs
/// `[index + 1, false_target)` and answers `Goto(resume)`; `ir::compile`
/// emits a `JumpUnless` to `false_target` and, when the two differ, a `Jump`
/// to `resume` at the end of the true branch. Having each work the pair out
/// for itself is how the two would come to disagree about which instruction an
/// `ELSE` starts at.
pub(crate) struct IfTargets {
    /// Where control goes when the condition is false: the `ELSE` when there
    /// is one, otherwise the instruction after the `THEN` branch.
    pub(crate) false_target: usize,
    /// Where control resumes once the *true* branch has finished, which is
    /// past the `ELSE` branch when there is one and identical to
    /// `false_target` when there is not.
    pub(crate) resume: usize,
}

/// [`IfTargets`] for an `If` whose parsed `false_target` is `raw`, against the
/// body it belongs to.
pub(crate) fn if_targets(instructions: &[Instruction], raw: Option<usize>) -> IfTargets {
    // `None` is the end of this body (`InstructionKind::If`'s own doc), which
    // is one past the last instruction and exactly what an empty range there
    // needs.
    let false_target = raw.unwrap_or(instructions.len());
    IfTargets {
        false_target,
        resume: skip_else(instructions, false_target),
    }
}

/// Where a listed `WHEN` sends control once its own condition holds.
///
/// **One computation, read by both engines**, exactly as [`IfTargets`] is.
/// `step`'s own `Select` arm runs `[when + 1, body_end)` and hands whatever
/// comes back to `leave_select`, which answers `Goto(resume)` for a branch
/// that finished; `ir::compile` lays the same range out as the ops between
/// this `WHEN`'s clause region and the next one's, and the driver opens a
/// frame over exactly it. Having each work the pair out for itself is how the
/// two would come to disagree about where a matched branch ends.
pub(crate) struct WhenTargets {
    /// One past the last instruction of this `WHEN`'s own branch: the next
    /// listed `WHEN`, the `OTHERWISE`, or the enclosing `SELECT`'s `END`.
    pub(crate) body_end: usize,
    /// Where control resumes once that branch has finished, which is past the
    /// whole `SELECT`, because one true `WHEN` ends it.
    pub(crate) resume: usize,
}

/// [`SelectResume`] for a matched listed `WHEN`: one answer twice, because one
/// true `WHEN` ends the whole `SELECT` whether its branch finished or was left
/// by name.
pub(crate) fn when_resume(targets: &WhenTargets) -> SelectResume {
    SelectResume {
        done: targets.resume,
        left: targets.resume,
    }
}

/// [`SelectResume`] for the `OTHERWISE` branch: falling off its end runs the
/// `END`, and a `LEAVE` naming this `SELECT` resumes past it.
pub(crate) fn otherwise_resume(len: usize, end: Option<usize>) -> SelectResume {
    SelectResume {
        done: otherwise_range(len, end),
        left: select_exit(len, end),
    }
}

/// [`WhenTargets`] for a listed `When`/`WhenCase` node, against the body it
/// belongs to.
///
/// `len` is that body's own instruction count, which is what a `None` target
/// means (`InstructionKind::When`'s own doc). The panic is the parser's
/// invariant that a `SELECT`'s `whens` collects nothing else, the same one
/// `Interp::scan_when` states.
pub(crate) fn when_targets(kind: &InstructionKind, len: usize) -> WhenTargets {
    match kind {
        InstructionKind::When {
            false_target, exit, ..
        }
        | InstructionKind::WhenCase {
            false_target, exit, ..
        } => WhenTargets {
            body_end: false_target.unwrap_or(len),
            resume: exit.unwrap_or(len),
        },
        other => panic!("a SELECT's whens holds only When/WhenCase, not {other:?}"),
    }
}

/// What a `SELECT` node tells whoever is running one of its branches.
///
/// `step`'s own `Select` arm has these in scope from the `match` that
/// destructured the node; the driver, which arrives at a branch through an op
/// carrying an instruction index and nothing else, reads them back from the
/// same node through [`select_parts`].
pub(crate) struct SelectParts {
    /// `SELECT LABEL name`'s own label, which a `LEAVE`/`ITERATE` may name.
    pub(crate) label: Option<SymbolId>,
    /// This `SELECT`'s own `OTHERWISE` marker, if it has one.
    pub(crate) otherwise: Option<usize>,
    /// The `END` that closes it.
    pub(crate) end: Option<usize>,
}

/// [`SelectParts`] for a `Select` node, and `None` for anything else.
pub(crate) fn select_parts(kind: &InstructionKind) -> Option<SelectParts> {
    match kind {
        InstructionKind::Select {
            label,
            otherwise,
            end,
            ..
        } => Some(SelectParts {
            label: *label,
            otherwise: *otherwise,
            end: *end,
        }),
        _ => None,
    }
}

/// One past the last instruction of a `SELECT`'s `OTHERWISE` branch, which is
/// also where control resumes once that branch has finished: its own `END`,
/// where the `EndStyle::Otherwise` arm does nothing.
///
/// `len` is the body's instruction count, which is what a `None` `end` means.
/// One computation for the same reason [`when_targets`] is one.
pub(crate) fn otherwise_range(len: usize, end: Option<usize>) -> usize {
    end.unwrap_or(len)
}

/// Where a `LEAVE` naming a `SELECT` resumes: past the `END` that closes it.
///
/// The same answer a listed `WHEN` carries in its own `exit` (`ast.rs`: "the
/// instruction after the enclosing `SELECT`'s `END`"), computed for the
/// `OTHERWISE` branch, which has no node of its own to carry it.
pub(crate) fn select_exit(len: usize, end: Option<usize>) -> usize {
    match end {
        Some(end) => (end + 1).min(len),
        None => len,
    }
}

/// Where a `SELECT` sends control when one of its branches is over, which is
/// **two answers and not one**.
///
/// They coincide for a matched `WHEN` and differ for `OTHERWISE`. The pairing
/// is built by [`when_resume`] and [`otherwise_resume`] rather than at each
/// engine's own call site, so that the rule deciding *which* of the two a
/// branch gets is one thing in one place: measured under `trace r`, a `LEAVE`
/// naming a `SELECT` from inside its `OTHERWISE` echoes no `end` clause where
/// the same branch falling through echoes one, and a single answer cannot be
/// right for both.
#[derive(Clone, Copy)]
pub(crate) struct SelectResume {
    /// A branch that ran off its own end. `OTHERWISE`'s falls through onto the
    /// `END`, which executes and does nothing; a matched `WHEN`'s resumes past
    /// the `END`, because one true `WHEN` ends the whole `SELECT`.
    pub(crate) done: usize,
    /// A `LEAVE` naming this `SELECT`, which resumes past the `END` from
    /// **either** branch. Measured under `trace r`: `select label s` /
    /// `when 1 = 0 then nop` / `otherwise leave s` / `end` echoes no `end`
    /// clause at all, where the same `OTHERWISE` falling through echoes one.
    pub(crate) left: usize,
}

/// What a `SELECT` does with a `Flow` that escaped the branch it was running.
///
/// **The tree-walker's decision, and the compiled stream expresses the same
/// one as layout rather than as a second copy of this.** `run_bounded` owns
/// one range, so a `Flow::Goto` onto the `OTHERWISE` marker escapes it and
/// leaves the construct entirely unless something recognises the target --
/// which is what this is for. In the stream `Chunk::op_of` *is* the resume
/// table and the op that opens `OTHERWISE`'s frame sits at the marker's entry
/// in it, so every arrival there already opens that frame
/// (`Interp::leave_branch`'s own doc comment has what was measured).
pub(crate) enum SelectEscape {
    /// The flow lands exactly on this `SELECT`'s own `OTHERWISE` marker, so
    /// that branch runs **with this `SELECT`'s search frame still standing**.
    ///
    /// An absorbed `WhenCase`'s false-branch escape is the one thing that
    /// produces it: a bare `Flow::Goto` past the matched branch's own bounds,
    /// which forwarded unrecognised would run `OTHERWISE`'s body under
    /// whichever outer construct received the `Goto`, with no `SELECT` frame
    /// for a `LEAVE`/`ITERATE` inside it to find. Measured, `select label s
    /// case 2` / `when 2 then` / `when 3 then nop` / `otherwise say 'O'` /
    /// `leave s` / `end`: oracle `O`, `after`, rc 0, where forwarding it
    /// outward is `Error 28.3`, rc 228.
    Otherwise(usize),
    /// Anything else, `leave_select`'s to resolve.
    Forward(Flow),
}

/// [`SelectEscape`] for `flow` against a `SELECT` whose `OTHERWISE` is at
/// `otherwise`.
pub(crate) fn select_escape(otherwise: Option<usize>, flow: Flow) -> SelectEscape {
    match flow {
        Flow::Goto(target) if otherwise == Some(target) => SelectEscape::Otherwise(target),
        other => SelectEscape::Forward(other),
    }
}

/// Where the *true* branch resumes, given where the false one goes.
///
/// `target` is the `If`'s own `false_target`: an `ELSE`'s index when there is
/// one, in which case the true branch resumes past that `ELSE`'s own branch
/// (`Else::then_exit`), and otherwise already the resume itself.
fn skip_else(instructions: &[Instruction], target: usize) -> usize {
    match instructions.get(target).map(|i| &i.kind) {
        Some(InstructionKind::Else { then_exit }) => then_exit.unwrap_or(instructions.len()),
        _ => target,
    }
}

/// 20.928: a subsidiary-list word is not a legal symbol at all (contains a
/// byte outside `is_symbol_byte`'s set, which is also what a parenthesised
/// entry like `"(w)"` fails on).
fn raised_symbol_expected(found: &[u8]) -> Raised {
    Raised::syntax(20, 928, vec![found.to_vec()])
}

/// 31.2: a subsidiary-list word starts with a digit.
fn raised_digit_led(found: &[u8]) -> Raised {
    Raised::syntax(31, 2, vec![found.to_vec()])
}

/// 31.3: a subsidiary-list word starts with a period.
fn raised_dot_led(found: &[u8]) -> Raised {
    Raised::syntax(31, 3, vec![found.to_vec()])
}

/// 34.1: a single (non-list) `IF` condition is not exactly `0` or `1`.
/// `Error_Logical_value_if`, catalogue text "Value of expression following
/// IF keyword must be exactly \"0\" or \"1\"; found \"...\"", one
/// substitution, the operand's own rendered text.
pub(crate) fn raised_if_not_logical(found: &[u8]) -> Raised {
    Raised::syntax(34, 1, vec![found.to_vec()])
}

/// 34.2: a single (non-list) `WHEN` condition is not exactly `0` or `1`.
/// `Error_Logical_value_when`, the same shape as `raised_if_not_logical`
/// with `WHEN`'s own sub-number -- a plain `WHEN`'s comma list is the
/// opposite case (`WhenCase`'s doc comment) and never reaches this raiser:
/// [`Interp::eval_condition`] hands a list over already `checked`, and
/// `crate::ir::compile`'s `native_shape` declines `ExprKind::Logical`, so a
/// list never becomes a `crate::ir::Op::Condition` either.
pub(crate) fn raised_when_not_logical(found: &[u8]) -> Raised {
    Raised::syntax(34, 2, vec![found.to_vec()])
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
/// `Raised::syntax`.
/// Turns `RAISE SYNTAX`'s own argument into the condition it names, or into
/// the condition the oracle raises when it names nothing.
///
/// **Three outcomes, all measured.** Two of them arise only because `RAISE`
/// lets a program name an arbitrary error number.
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
fn raise_syntax_condition(text: &[u8], additional: Vec<Vec<u8>>) -> Raised {
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
    Raised::syntax(98, 941, vec![found.into_bytes()])
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
    let additional = crate::error::into_substitutions(error.additional());
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
    Raised::syntax(34, 3, vec![found.to_vec()])
}

/// 34.4: `UNTIL`'s own version of `raised_while_not_logical`.
fn raised_until_not_logical(found: &[u8]) -> Raised {
    Raised::syntax(34, 4, vec![found.to_vec()])
}

/// 26.2: a bare `DO`'s own repetition-count expression is not zero or a
/// positive whole number. `Error_Invalid_expression_do`, measured: `do
/// 'a'`/`do -1`/`do 2.5` all give this, `found` the operand's own
/// unmodified text (`"a"`/`"-1"`/`"2.5"`).
fn raised_repetition_count_not_whole(found: &[u8]) -> Raised {
    Raised::syntax(26, 2, vec![found.to_vec()])
}

/// 26.3: a `DO`/`LOOP`'s `FOR` expression is not zero or a positive whole
/// number. Measured: `do i = 1 to 3 for 'x'`/`for -1`/`for 1.5`.
fn raised_for_count_not_whole(found: &[u8]) -> Raised {
    Raised::syntax(26, 3, vec![found.to_vec()])
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
    Raised::syntax(28, 3, vec![found.to_vec()])
}

/// 28.4: `ITERATE`'s own version of `raised_leave_no_match`.
fn raised_iterate_no_match(found: &[u8]) -> Raised {
    Raised::syntax(28, 4, vec![found.to_vec()])
}

/// 28.5: a named `ITERATE name` matched a block on the enclosing chain by
/// label, but that block is not a repetitive loop (a labelled `DO`/plain
/// block, or a `SELECT LABEL` -- `ITERATE` never accepts either, unlike
/// `LEAVE`). Measured: `do label x / say 1 / iterate x / end` gives this,
/// not 28.4, because `x` *did* match something.
fn raised_iterate_wrong_kind(found: &[u8]) -> Raised {
    Raised::syntax(28, 5, vec![found.to_vec()])
}

/// Whether `a < b`, numerically, through `rexx-num`'s own `compare_decoded`
/// rather than a hand-rolled sign comparison -- this crate's standing rule
/// against a second copy of a comparison `rexx-num` already owns
/// (`compare_values`' own doc comment states it for the expression
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
mod tests;
