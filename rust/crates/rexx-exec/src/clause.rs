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

//! One Rexx clause: which line it is, and what has to happen when it ends.
//!
//! **A module of its own so that `run.rs` cannot reach the line field**, which
//! is the whole point of the file (4b Task 7, fix round 3). A private field on
//! a crate-root struct is still visible to `crate::run`, because a child
//! module sees its ancestors' private items; a private field on a struct in
//! *this* module is not visible to `crate::run` at all. That difference is
//! what turns "remember to do both things" into "you cannot do one of them".
//!
//! # The defect this shape exists to end
//!
//! Four rounds of this task each fixed the sites they knew about and each
//! asserted an exhaustiveness that was false a round later:
//!
//! * round 1 fixed `run_activation`'s `Flow` arms and said there was "no path
//!   at all" left -- `run_bounded` was a path;
//! * round 2 extracted one shared function and said it had "exactly two
//!   callers", offering a grep -- `run_loop` was a third;
//! * round 3 made the boundary a token obligation and said "every site that
//!   runs a clause calls this" -- an `IF`'s condition, a `WHEN`'s condition
//!   and a `SELECT CASE`'s expression were four more sites, all silent;
//! * and each time the *same* omission produced two symptoms at once, because
//!   a site that fails to say "a new clause is starting" also fails to run
//!   what a clause boundary owes. `do while zn < sub()` reported `SIGL` from
//!   a clause three lines away *and* delivered its `CALL ON` handler at the
//!   wrong moment: one missing call, two wrong answers.
//!
//! # Where the set of sites comes from now, and it is not a list
//!
//! The oracle's own clause boundary is one place: `RexxActivation::run`'s
//! instruction loop calls `processClauseBoundary()` after each
//! `nextInst->execute()` returns (`RexxActivation.cpp:642-654`, read
//! directly). So **a clause boundary sits after every instruction of the
//! activation's own flat list**, and nothing else is one.
//!
//! This crate diverges from that in exactly one way, which is therefore the
//! whole rule for where [`Interp::in_clause`] belongs: `IF`, `SELECT`,
//! `DO`/`LOOP` and `INTERPRET` resolve *other* instructions inside their own
//! `step` (`run_bounded`'s doc comment has why that is not optional), where
//! the oracle's loop would have fetched each of them separately. Every such
//! construct therefore has to end its own **header** clause before it runs
//! anything else -- and `INTERPRET` is the one that must not, because there
//! the oracle runs the fragment in an activation of its own whose condition
//! queue is separate (`RexxActivation::interpret`, and measured: a condition
//! queued by the `INTERPRET` clause's own expression is *not* delivered
//! inside the fragment).
//!
//! `Interp::in_clause`'s own `debug_assert` is what makes a fifth such site
//! announce itself rather than being found by a reviewer's probe: a clause
//! that begins while an earlier clause of the same activation still owes a
//! delivery is exactly the defect, and it now aborts the test suite naming
//! itself.
//!
//! # What the shape guarantees, measured rather than asserted
//!
//! Overstating exactly this is the error the file is a response to, so what
//! follows was established by writing each mutation, building it and reading
//! the exit status -- see the report's fix-round-4 attack table.
//!
//! The clause's line and the clause's boundary are one operation with one
//! entry point, [`Interp::in_clause`], whose body is a closure. That closes
//! the whole family of "the two halves came apart" mutations round 3's token
//! left open, because there is no longer a value in scope to mishandle:
//! `let _token`, `drop(token)`, `std::mem::forget(token)`, an early `return`
//! between the two halves and a `?` between them are none of them
//! expressible. An early `return` *inside* the closure returns from the
//! closure, and the boundary still runs; an early `return` outside it is
//! after the boundary already ran.
//!
//! Two things remain expressible, and they are named here rather than left
//! for the next re-review to find:
//!
//! * **A site can decline to call `in_clause` at all.** Nothing in the type
//!   system requires an instruction to be a clause. That is what the
//!   `debug_assert` covers, and it covers it by behaviour rather than by
//!   type: it fires only when such a site is actually reached with a
//!   condition waiting.
//! * **What this module has to expose to its own legitimate callers is also
//!   what a caller could misuse, and that is not closed by a type the way
//!   the "two halves came apart" family above is.** `self.clause_state =
//!   ClauseState::new()` compiles, because `Interp::new` needs
//!   `ClauseState::new` to be `pub(crate)` and Rust visibility cannot grant
//!   that to `lib.rs` while withholding it from `lib.rs`'s other children --
//!   that reset is loud in its own right (`SIGL` 0 is not a line).
//!   `save_clause_state`/`restore_clause_state` are `pub(crate)` for the
//!   same reason, so `resolve_and_run_call` can put a callee's caller state
//!   back after the callee returns; nothing stops `run.rs` from restoring a
//!   *stale* [`SavedClauseState`] at a moment other than the one it was
//!   taken from, which sets a nonzero line with no boundary attached at
//!   that moment -- measured: builds, passes clippy, and passes all 296 lib
//!   tests, undetected. `deliver_pending_trap` is `pub(crate)` for the
//!   mirror reason (a failed clause's own boundary runs from
//!   `offer_to_trap`, not from `in_clause`), and nothing in the type system
//!   stops it running a boundary paired with no line set at all. **This is
//!   what is reachable through this module's own `pub(crate)` surface
//!   today, not a proof that nothing else is** -- the property behind all
//!   three is that a function this module must expose for one legitimate
//!   caller is a function every other `pub(crate)` caller can also reach.

use crate::run::Flow;
use crate::{Code, Ended, Failure, Interp, ObjRef};

/// Every piece of state `step_in_temps_frame` sets fresh, unconditionally, on
/// **every** instruction it steps -- and so every field a caller pushing a
/// nested activation (`resolve_and_run_call`, `run.rs`) must save before the
/// callee runs and restore after it returns, because the callee's own
/// `step_in_temps_frame` calls overwrite these exactly as the caller's own
/// next clause would.
///
/// **The property that decides membership**, so the next field can be
/// checked against it rather than added by analogy: set per clause by
/// `step_in_temps_frame`, *and* read somewhere that can run after a nested
/// activation has already run and returned within the same clause. `say
/// f(1) + g(2)` is what makes the second half observable at all -- at most
/// one activation could be entered per clause before Task 4 (`ExprKind::
/// Call`), and the *next* clause's own `step_in_temps_frame` re-set these
/// fields before anything read them, so a version missing the restore
/// passed every test with no more than one call per clause in it.
///
/// A field failing either half does not belong here. `resolve_and_run_
/// call`'s own five (`activation_indent`/`indent_offset`/
/// `clause_line_override`/`call_context`, plus this whole struct) are not
/// all one shape: those four are level state *for the callee*, each set
/// once per call to a value the callee computes (`activation_indent` to
/// the calling clause's indent plus two, `call_context` to that call's own
/// name and arguments, ...), never refreshed per clause the way this
/// struct's own fields are -- each already has its own reason, stated at
/// that save/restore block rather than here.
///
/// **Bundled into one field, `Interp::clause_state`, rather than left as
/// separate fields each needing its own save/restore line at a nested-
/// activation boundary.** `current_value_indent` is Task 4's own C1;
/// `current_clause_line` is Task 6's, and it shipped *without* the restore
/// its own sibling field already carried -- the second time in a row the
/// newer field of this exact shape went in without it, which is the same
/// "a hand-maintained list eventually drops an entry" shape this project's
/// own owner tables were already burned by three times. One struct and one
/// [`Interp::save_clause_state`]/[`Interp::restore_clause_state`] pair at
/// the save/restore site is what makes a third omission structurally
/// impossible rather than merely against the rules: a field added *here* is
/// restored by that existing pair with no second edit anywhere, where a
/// field added directly to `Interp` needs someone to have read this comment
/// first.
///
/// **Deliberately not `Copy` or `Clone`** (fix round 4). It was both, and
/// that is what made `self.clause_state = <some other ClauseState>` -- a
/// line set with no clause boundary attached -- expressible from `run.rs`
/// despite the private field, which round 3's module doc denied. The one
/// legitimate whole-struct write is the restore, and it now goes through a
/// type that can only carry a value this module produced.
pub(crate) struct ClauseState {
    /// The indent (Task 11's `static_indent` quantity, spaces already
    /// doubled) an intermediate value line traces at right now -- the one
    /// piece of state `eval`'s own single insertion point needs that
    /// `eval`'s signature does not otherwise carry, mirroring the oracle's
    /// own `settings.traceIndent` (a persistent field every `traceValue`/
    /// `traceVariable`/... call reads, never threaded as a parameter
    /// through `evaluate`). Set once per traced clause, at whichever call
    /// site already computed that clause's own indent for the `*-*` echo
    /// or a `>K>` line -- `run.rs`'s own doc comments name each site.
    ///
    /// **Why a field and not an `eval` parameter.** Threading an indent
    /// through `eval`/`eval_node`'s entire recursive call graph would touch
    /// every arm in `eval.rs`, exactly the "eighteen arms" retrofit the
    /// design's own withdrawn note wrongly predicted for the value events
    /// themselves -- reading the oracle's own field-not-parameter design
    /// avoids inventing that threading here instead.
    ///
    /// Public to the crate where its sibling is private, and that asymmetry
    /// is the point: an indent that is momentarily wrong prints a wrong
    /// number of spaces, where a line that is momentarily wrong means a
    /// clause boundary did not happen.
    pub(crate) current_value_indent: usize,
    /// The line the clause currently being stepped starts at -- **`SIGL`'s**
    /// own value, one control transfer away from being read, and the exact
    /// analogue of `current_value_indent` just above: `resolve_and_run_call`
    /// (`CALL`, and `ExprKind::Call`'s expression form through `eval_call`,
    /// `eval.rs`) and `SIGNAL`'s own two `step` arms all need "which line is
    /// this transfer's own", and `eval_call` reaches `resolve_and_run_call`
    /// from arbitrarily deep inside an expression tree with no `source`/
    /// `instruction` of its own to compute it from -- threading either
    /// through `eval`/`eval_node`'s entire recursive call graph is exactly
    /// the "every arm in `eval.rs`" retrofit `current_value_indent`'s own
    /// doc comment already declined for the identical reason. Set
    /// unconditionally by `step_in_temps_frame`, via `clause_line`, which
    /// honours `clause_line_override` the same way `clause_site` does -- so
    /// a `SIGNAL`/`CALL` fired from inside an `INTERPRET` fragment reads the
    /// *enclosing* `INTERPRET` clause's own line, matching the oracle's own
    /// `RexxActivation::signalTo`, read directly: an interpret-created
    /// activation delegates a `SIGNAL` to its parent rather than setting
    /// `SIGL` itself, so what ends up in `SIGL` is the parent's own
    /// currently-executing instruction -- the `INTERPRET` clause -- and this
    /// field reproduces that observable answer without this crate adopting
    /// the C++ architecture that produces it (`run_fragment` still runs
    /// inside the creating activation, not a nested one of its own).
    current_clause_line: usize,
}

impl ClauseState {
    /// The current clause's line -- `SIGL`'s own value.
    ///
    /// A getter because the field is private to this module: reading it is
    /// harmless, and it is *setting* it that has to drag the boundary along.
    pub(crate) fn line(&self) -> usize {
        self.current_clause_line
    }

    /// The zero state `Interp::new` starts from.
    pub(crate) fn new() -> ClauseState {
        ClauseState {
            current_value_indent: 0,
            current_clause_line: 0,
        }
    }
}

/// A `ClauseState` taken out of an `Interp` so it can be put back -- and
/// nothing else.
///
/// The whole point is what it does *not* offer: no constructor, no field
/// access to the line, and `ClauseState` itself is not `Copy`, so the only
/// way `run.rs` can write the clause line as part of a whole-struct
/// assignment is by restoring a value some [`Interp::in_clause`] set.
/// `current_value_indent` is readable because `resolve_and_run_call` computes
/// the callee's own base indent from it.
pub(crate) struct SavedClauseState(ClauseState);

impl SavedClauseState {
    /// The saved intermediate-value indent, which `resolve_and_run_call`
    /// reads to compute the callee's own base indent (that clause's printed
    /// indent plus two, D2r).
    pub(crate) fn value_indent(&self) -> usize {
        self.0.current_value_indent
    }
}

/// What a clause resolved to, for the one thing a boundary needs from it:
/// a value that has to stay rooted while a `CALL ON` handler runs.
///
/// **No default implementation on purpose.** A type that reaches
/// [`Interp::in_clause`] has to answer "does this carry an `ObjRef` whose
/// only root was the clause's own temps frame?" explicitly, because the one
/// type that answers *yes* is `Flow` and getting that wrong is a
/// use-after-free rather than a wrong number. Round 3 expressed this as
/// `ClauseEnd::Completed(Option<&Flow>)`, where a site holding a `Flow`
/// could pass `None` and silently drop the rooting -- measured, that
/// mutation panicked `a live value` under collect-on-every-allocation while
/// clippy and all 970 tests stayed green. Here the value comes from the
/// closure's own return type, which narrows the *shape* a site can hand
/// back -- but does not decide the question by itself. A site can still
/// compute the `Flow` inside the closure, write it to a captured
/// `&mut Option<Flow>`, and return `Ok(())`, landing on `ClauseValue for
/// ()` and dropping the rooting the same way `Completed(None)` did:
/// measured, build 0, clippy 0, all 296 lib tests green, and it is
/// `a_clause_value_survives_the_handler_its_boundary_runs`
/// (`tests/collect_stress.rs`) that catches it, panicking `a live value`.
/// The type narrows what a site can return; that test is what actually
/// pins the rooting.
pub(crate) trait ClauseValue {
    /// The value to root across a delivered handler, if any.
    fn rooted(&self) -> Option<ObjRef>;
}

impl ClauseValue for Flow {
    /// `flow` may carry an `ObjRef` whose one-clause temps frame is already
    /// popped, and the handler is a nested activation that allocates, so the
    /// value is rooted across it. Measured with a negative control: without
    /// the `push_temp` in `in_clause` a `Flow::Return` and a `Flow::Exit`
    /// program each panic on `a live value` under collect-on-every-
    /// allocation.
    fn rooted(&self) -> Option<ObjRef> {
        match self {
            Flow::Return(Some(value)) | Flow::Exit(Some(value)) => Some(*value),
            _ => None,
        }
    }
}

impl ClauseValue for () {
    /// A loop header, a `WHILE`/`UNTIL` re-test and a `SELECT CASE`
    /// expression all produce no `Flow` at all, so there is nothing whose
    /// only root was this clause's frame.
    fn rooted(&self) -> Option<ObjRef> {
        None
    }
}

impl ClauseValue for bool {
    /// An `IF`'s or a `WHEN`'s condition: the answer is a Rexx logical
    /// value, already consumed into a `bool` by `eval_condition`, so no
    /// `ObjRef` escapes this clause.
    fn rooted(&self) -> Option<ObjRef> {
        None
    }
}

/// How [`Interp::in_clause`] finished.
///
/// The outer `Result`'s `Err` is the *handler's* own failure, never the
/// clause's: a clause that failed comes back as `Ran(Err(_))`, which is what
/// lets `step_in_temps_frame` tell "my instruction raised" from "the `CALL
/// ON` handler my boundary ran raised" and blame the right clause for each.
pub(crate) enum ClauseOutcome<T> {
    /// The clause ran, successfully or not, and here is what it produced.
    Ran(Result<T, Failure>),
    /// A `CALL ON` handler ran at this clause's boundary and ended the whole
    /// program.
    Ended(HandlerExit),
}

/// A delivered `CALL ON` handler ended the program with `EXIT`.
///
/// **A type rather than an `Ended`** (fix round 4). `deliver_pending_trap`
/// only ever reports `Ended::Exited`: a handler that *returns* resumes the
/// interrupted clause and reports `Ok(None)` instead. Round 2 said so with
/// an `unreachable!("clause_boundary reports only Ended::Exited")`; round 3
/// replaced that with six copies of `Ok(Flow::Exit(ended.value()))`, and
/// `Ended::value()` collapses `Returned` and `Exited`, so a future
/// `deliver_pending_trap` that reported `Returned` would silently turn a
/// `RETURN` into an `EXIT`. Now it cannot be built at all except from an
/// `Ended::Exited`, at the single point that match already lives.
pub(crate) struct HandlerExit(Option<ObjRef>);

impl HandlerExit {
    /// The value the handler exited with.
    pub(crate) fn value(self) -> Option<ObjRef> {
        self.0
    }

    /// The only constructor, and it is the invariant rather than a wrapper
    /// around it: an `Ended::Returned` answers `None`, because a handler that
    /// *returned* resumes the interrupted clause and has not ended anything.
    /// A caller that routes the wrong variant here therefore gets "carry on",
    /// which is the correct behaviour, rather than a `RETURN` silently
    /// rendered as an `EXIT` -- the shape re-review 3's NEW-6 named.
    pub(crate) fn from_ended(ended: Ended) -> Option<HandlerExit> {
        match ended {
            Ended::Exited(value) => Some(HandlerExit(value)),
            Ended::Returned(_) => None,
        }
    }
}

impl Interp {
    /// Runs one Rexx clause: `line` is the clause's own line, `body` is the
    /// whole of the clause.
    ///
    /// Setting the line and running the boundary are one operation because
    /// they are one fact -- see this module's doc comment for the three
    /// rounds that established that the hard way, and for where the set of
    /// call sites comes from.
    ///
    /// The boundary is where a `CALL ON` condition queued *during* this
    /// clause gets its handler run: the wait is measured, `zres = one(1)`
    /// with `one` raising a trapped condition and the handler assigning
    /// `zres` prints the handler's value, so the assignment had already
    /// completed. A clause that failed reached no boundary and delivers
    /// nothing -- for a failure that is trapped *here* rather than unwinding
    /// the activation, `offer_to_trap` is the one place that knows, and it
    /// delivers there.
    pub(crate) fn in_clause<T: ClauseValue>(
        &mut self,
        code: &Code<'_>,
        line: usize,
        body: impl FnOnce(&mut Self) -> Result<T, Failure>,
    ) -> Result<ClauseOutcome<T>, Failure> {
        // **The fourth-site tripwire** (fix round 4). A condition queued by
        // this activation's clause at line L that is still waiting when a
        // clause at a *different* line begins means some construct resolved
        // an instruction inside its own `step` without ending its header
        // clause first -- the defect behind re-review finding NEW-2's four
        // sites and three rounds' worth before them, always silently. The
        // handler will now report `SIGL` for the wrong clause, which is the
        // half of the defect that has recurred every round.
        //
        // **What the line comparison exempts, and why it is not a hedge.**
        // Two clauses of this crate can carry one line legitimately, and in
        // both the oracle has a single clause there:
        //
        // * A `DO`/`LOOP`'s control setup (`run_loop`) and its first header
        //   test (`run_repeating`) are two clauses here and one instruction
        //   -- `RexxInstructionControlledDo::execute` -- there. `do i = 1 to
        //   sub()` queues in the first and delivers in the second, both at
        //   the `DO` line, which is the line the oracle reports.
        // * Every clause of an `INTERPRET` fragment carries the enclosing
        //   `INTERPRET` clause's line (`clause_line_override`), which is this
        //   crate's stand-in for the oracle running fragment text in an
        //   activation of its own. Measured: `interpret sub()`, where `sub`
        //   raises a `CALL ON`-trapped condition, runs the fragment's own
        //   `say` with the handler's variable still unset, on the oracle and
        //   here -- the condition waits for the enclosing clause.
        //
        // So this catches the wrong-`SIGL` half and says so; a delivery that
        // is late in *time* but lands on the same line is invisible to it,
        // and to `SIGL`.
        debug_assert!(
            self.clause_state.current_clause_line == line
                || self.pending_trap.as_ref().map(|pending| pending.activation)
                    != Some(self.activation().id),
            "a clause at line {} began while a condition queued by this activation's clause at \
             line {} was still waiting: some construct ran an instruction inside its own step \
             without ending its header clause first",
            line,
            self.clause_state.current_clause_line
        );
        self.clause_state.current_clause_line = line;
        let ran = body(self);
        let Ok(value) = &ran else {
            // A clause that is unwinding never reached a boundary, so it
            // delivers nothing.
            return Ok(ClauseOutcome::Ran(ran));
        };
        if self.pending_trap.is_none() {
            return Ok(ClauseOutcome::Ran(ran));
        }
        if let Some(value) = value.rooted() {
            self.roots.push_temp(value);
        }
        match self.deliver_pending_trap(code)? {
            Some(exit) => Ok(ClauseOutcome::Ended(exit)),
            None => Ok(ClauseOutcome::Ran(ran)),
        }
    }

    /// Takes a copy of the clause state for `resolve_and_run_call` to put
    /// back after the callee has run.
    ///
    /// Here rather than in `run.rs` because the fields are private to this
    /// module, which is what stops the copy being modified in between.
    pub(crate) fn save_clause_state(&self) -> SavedClauseState {
        SavedClauseState(ClauseState {
            current_value_indent: self.clause_state.current_value_indent,
            current_clause_line: self.clause_state.current_clause_line,
        })
    }

    /// Puts back what [`Interp::save_clause_state`] took.
    pub(crate) fn restore_clause_state(&mut self, saved: SavedClauseState) {
        self.clause_state = saved.0;
    }
}
