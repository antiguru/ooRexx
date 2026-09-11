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

use std::time::{Duration, Instant};

use crate::run::Flow;
use crate::{Code, Ended, Failure, Interp, ObjRef};

/// A wall-clock bound on a whole run, honoured at the clause boundary and
/// unset by default.
pub(crate) struct Deadline {
    /// When the run must stop.
    at: Instant,
    /// Whether the clock has already been found past [`Deadline::at`].
    expired: bool,
}

impl Deadline {
    /// Clauses between two reads of the clock, and so the granularity the
    /// bound is honoured to.
    pub(crate) const CLAUSES_PER_CHECK: u32 = 1024;

    /// Clauses between two visits to [`Interp::countdown_reached`] for a run
    /// with no deadline, where the visit does nothing but reload this.
    pub(crate) const NO_DEADLINE_SPACING: u32 = u32::MAX;

    /// A bound `limit` from now.
    pub(crate) fn starting_now(limit: Duration) -> Deadline {
        Deadline {
            at: Instant::now() + limit,
            expired: false,
        }
    }
}

/// Proof that the clause about to be opened has had the deadline counted
/// against it.
pub(crate) struct DeadlineCounted(());

/// Every piece of state `step_in_temps_frame` sets fresh, unconditionally, on
/// **every** instruction it steps -- and so every field a caller pushing a
/// nested activation (`Interp::invoke_call`, `run.rs`) must save before the
/// callee runs and restore after it returns, because the callee's own
/// `step_in_temps_frame` calls overwrite these exactly as the caller's own
/// next clause would.
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
    pub(crate) current_value_indent: usize,
    /// Whether the clause now running had instruction tracing in force **when
    /// it began**, which is the only thing an assignment's `>>>` echo is
    /// allowed to consult.
    pub(crate) instructions_traced_at_entry: bool,
    /// The line the clause currently being stepped starts at -- **`SIGL`'s**
    /// own value, one control transfer away from being read, and the exact
    /// analogue of `current_value_indent` just above: `Interp::invoke_call`
    /// (`CALL`, and `ExprKind::Call`'s expression form through `eval_call`,
    /// `eval.rs`) and `SIGNAL`'s own two `step` arms all need "which line is
    /// this transfer's own", and `eval_call` reaches `invoke_call`
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
    /// The index, in the running activation's own body, of the clause being
    /// stepped -- the one piece `StackFrame~traceLine` needs that the two
    /// fields above do not carry, since a clause's *text* is its
    /// instruction's `clause_span` and nothing else records which
    /// instruction is running (`Activation::pc` stands on the enclosing
    /// construct, per [`crate::activation::ClauseSnapshot`]).
    pub(crate) current_clause_index: usize,
}

impl ClauseState {
    /// The current clause's line -- `SIGL`'s own value.
    pub(crate) fn line(&self) -> usize {
        self.current_clause_line
    }

    /// The clause index [`ClauseState::current_clause_index`] holds.
    pub(crate) fn clause_index(&self) -> usize {
        self.current_clause_index
    }

    /// The zero state `Interp::new` starts from.
    pub(crate) fn new() -> ClauseState {
        ClauseState {
            current_value_indent: 0,
            current_clause_line: 0,
            current_clause_index: 0,
            // `false` is the state an interpreter with nothing running is in,
            // matching `Interp::trace_cache`'s own `TraceMode::OFF`.
            instructions_traced_at_entry: false,
        }
    }
}

/// A `ClauseState` taken out of an `Interp` so it can be put back -- and
/// nothing else.
pub(crate) struct SavedClauseState(ClauseState);

impl SavedClauseState {
    /// The saved intermediate-value indent, which `Interp::invoke_call`
    /// reads to compute the callee's own base indent (that clause's printed
    /// indent plus two, D2r).
    pub(crate) fn value_indent(&self) -> usize {
        self.0.current_value_indent
    }
}

/// What a clause resolved to, for the one thing a boundary needs from it:
/// a value that has to stay rooted while a `CALL ON` handler runs.
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

/// A clause boundary that is open: [`Interp::enter_clause`] makes one and
/// [`Interp::leave_clause`] spends it.
#[must_use]
pub(crate) struct ClauseEntry(());

/// How [`Interp::in_clause`] finished.
pub(crate) enum ClauseOutcome<T> {
    /// The clause ran, successfully or not, and here is what it produced.
    Ran(Result<T, Failure>),
    /// A `CALL ON` handler ran at this clause's boundary and ended the whole
    /// program.
    Ended(HandlerExit),
}

/// A delivered `CALL ON` handler ended the program with `EXIT`.
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
    #[inline(always)]
    pub(crate) fn in_clause<T: ClauseValue>(
        &mut self,
        code: &Code<'_>,
        line: usize,
        body: impl FnOnce(&mut Self) -> Result<T, Failure>,
    ) -> Result<ClauseOutcome<T>, Failure> {
        let counted = self.count_clause_against_deadline()?;
        let entry = self.enter_clause(line, counted);
        let ran = body(self);
        self.leave_clause(entry, code, ran)
    }

    /// Opens the clause at `line`: everything [`Interp::in_clause`] does before
    /// the clause's own work runs.
    #[inline(always)]
    pub(crate) fn enter_clause(&mut self, line: usize, counted: DeadlineCounted) -> ClauseEntry {
        let DeadlineCounted(()) = counted;
        // **The argument stack is empty at every clause boundary**, and this
        // is what keeps that true when a clause is abandoned part-way through
        // pushing a call's run: a trapped `NOVALUE` on a second argument
        // leaves the first standing, and nothing later would take it. Every
        // ordinary call removes its own run, so this stores a length that is
        // already zero. Measured: 200,000 abandoned pushes grew the process
        // by 3.3 MB without it.
        self.value_buffer.clear();
        // **The fourth-site tripwire** (fix round 4). A condition queued by
        // this activation's clause at line L that is still waiting when a
        // clause at a *different* line begins means some construct resolved
        // an instruction inside its own `step` without ending its header
        // clause first -- the defect behind re-review finding NEW-2's four
        // sites and three rounds' worth before them, always silently. The
        // handler will now report `SIGL` for the wrong clause, which is the
        // half of the defect that has recurred every round.
        debug_assert!(
            self.clause_state.current_clause_line == line
                || !self.pending_traps.iter().any(|pending| {
                    !pending.queued_during_delivery && pending.activation == self.activation().id
                }),
            "a clause at line {} began while a condition queued by this activation's clause at \
             line {} was still waiting: some construct ran an instruction inside its own step \
             without ending its header clause first",
            line,
            self.clause_state.current_clause_line
        );
        self.clause_state.current_clause_line = line;
        ClauseEntry(())
    }

    /// Counts one clause against this run's deadline, and answers the proof
    /// [`Interp::enter_clause`] needs.
    #[inline(always)]
    pub(crate) fn count_clause_against_deadline(&mut self) -> Result<DeadlineCounted, Failure> {
        self.clause_countdown -= 1;
        if self.clause_countdown == 0 {
            self.countdown_reached()?;
        }
        Ok(DeadlineCounted(()))
    }

    /// Reads the clock and either reloads the countdown or ends the run.
    #[cold]
    #[inline(never)]
    fn countdown_reached(&mut self) -> Result<(), Failure> {
        let Some(deadline) = &mut self.deadline else {
            self.clause_countdown = Deadline::NO_DEADLINE_SPACING;
            return Ok(());
        };
        if deadline.expired || Instant::now() >= deadline.at {
            // Reloaded to 1, so the very next clause lands here again and
            // fails too. A run that has outlived its bound does not get to
            // resume because a caller swallowed one failure.
            deadline.expired = true;
            self.clause_countdown = 1;
            return Err(Failure::Deadline);
        }
        self.clause_countdown = Deadline::CLAUSES_PER_CHECK;
        Ok(())
    }

    /// Whether this run was cut short by its deadline.
    pub(crate) fn deadline_expired(&self) -> bool {
        self.deadline
            .as_ref()
            .is_some_and(|deadline| deadline.expired)
    }

    /// Closes the clause `entry` opened, around `ran` -- everything
    /// [`Interp::in_clause`] does once the clause's own work has run.
    #[inline(always)]
    pub(crate) fn leave_clause<T: ClauseValue>(
        &mut self,
        entry: ClauseEntry,
        code: &Code<'_>,
        ran: Result<T, Failure>,
    ) -> Result<ClauseOutcome<T>, Failure> {
        let ClauseEntry(()) = entry;
        let Ok(value) = &ran else {
            // A clause that is unwinding never reached a boundary, so it
            // delivers nothing.
            return Ok(ClauseOutcome::Ran(ran));
        };
        if self.pending_traps.is_empty() {
            return Ok(ClauseOutcome::Ran(ran));
        }
        if let Some(value) = value.rooted() {
            self.roots.push_temp(value);
        }
        match self.deliver_pending_traps(code)? {
            Some(exit) => Ok(ClauseOutcome::Ended(exit)),
            None => Ok(ClauseOutcome::Ran(ran)),
        }
    }

    /// Closes the clause `entry` opened **without running its boundary**.
    #[inline(always)]
    pub(crate) fn leave_clause_without_boundary<T: ClauseValue>(
        &self,
        entry: ClauseEntry,
        ran: Result<T, Failure>,
    ) -> Result<ClauseOutcome<T>, Failure> {
        let ClauseEntry(()) = entry;
        Ok(ClauseOutcome::Ran(ran))
    }

    /// Spends `entry` with **no boundary and no value at all**, for a clause
    /// that produced neither a `Flow` nor a failure at a moment when
    /// `pending_traps` is empty.
    #[inline(always)]
    pub(crate) fn spend_clause_entry(&self, entry: ClauseEntry) {
        debug_assert!(
            self.pending_traps.is_empty(),
            "a clause boundary was skipped while a condition was queued for it"
        );
        let ClauseEntry(()) = entry;
    }

    /// Takes a copy of the clause state for `Interp::invoke_call` to put
    /// back after the callee has run.
    pub(crate) fn save_clause_state(&self) -> SavedClauseState {
        SavedClauseState(ClauseState {
            current_value_indent: self.clause_state.current_value_indent,
            current_clause_line: self.clause_state.current_clause_line,
            current_clause_index: self.clause_state.current_clause_index,
            instructions_traced_at_entry: self.clause_state.instructions_traced_at_entry,
        })
    }

    /// Puts back what [`Interp::save_clause_state`] took.
    pub(crate) fn restore_clause_state(&mut self, saved: SavedClauseState) {
        self.clause_state = saved.0;
    }
}
