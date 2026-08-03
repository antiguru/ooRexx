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
//! Three rounds of this task each fixed the sites they knew about and each
//! asserted an exhaustiveness that was false a round later:
//!
//! * round 1 fixed `run_activation`'s `Flow` arms and said there was "no path
//!   at all" left -- `run_bounded` was a path;
//! * round 2 extracted one shared function and said it had "exactly two
//!   callers", offering a grep -- `run_loop` was a third;
//! * and each time the *same* omission produced two symptoms at once, because
//!   a site that fails to say "a new clause is starting" also fails to run
//!   what a clause boundary owes. `do while zn < sub()` reported `SIGL` from
//!   a clause three lines away *and* delivered its `CALL ON` handler at the
//!   wrong moment: one missing call, two wrong answers.
//!
//! A grep is evidence about today's tree. What is in this module instead is
//! the same move that finally killed the per-clause-restore defect: bind the
//! two facts into one operation so that doing one without the other is not
//! expressible. [`Interp::enter_clause`] is the only way to set the line, and
//! it hands back a [`ClauseToken`] that only [`Interp::end_clause`] consumes.
//!
//! **What that does and does not guarantee, measured rather than asserted** --
//! because overstating exactly this is the error the file is a response to.
//! Deleting the `end_clause` call from `step_in_temps_frame` and building
//! gives, verbatim:
//!
//! ```text
//! error: unused variable: `token`
//!     --> crates/rexx-exec/src/run.rs:3452:13
//!      |
//! 3452 |         let token = self.enter_clause(line);
//!      |             ^^^^^ help: if this is intentional, prefix it with an underscore: `_token`
//! error: could not compile `rexx-exec` (lib) due to 1 previous error
//! ```
//!
//! at exit 101 under the workspace's `cargo clippy … -D warnings` gate. So a
//! clause that is entered and never ended **does not build**.
//!
//! It does **not** stop `let _token = …`, and rustc's own message advertises
//! that escape. Nothing short of a scoped-closure API would close it, and
//! that is written down here rather than papered over. What this removes is
//! the *silent* case, which is the one that cost three rounds: forgetting the
//! boundary now costs a build failure, and getting the line wrong costs a
//! wrong `SIGL`, which tests assert.

use crate::run::Flow;
use crate::{Code, Ended, Failure, Interp};

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
/// own owner tables were already burned by three times. One `Copy` struct
/// and one assignment at the save/restore site (`let saved = self.
/// clause_state; ...; self.clause_state = saved;`) is what makes a third
/// omission structurally impossible rather than merely against the rules:
/// a field added *here* is restored by that existing assignment with no
/// second edit anywhere, where a field added directly to `Interp` needs
/// someone to have read this comment first.
#[derive(Copy, Clone)]
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

/// Proof that a clause was entered and that its boundary is still owed.
///
/// `#[must_use]` so that dropping one is a compile error under the
/// workspace's `-D warnings`: a site that begins a clause and never ends it
/// does not build. See this module's doc comment for the one hole this leaves
/// (`let _ = token`) and why it is stated rather than closed.
#[must_use = "a clause that was entered must be ended through Interp::end_clause, \
              which is what delivers a pending CALL ON handler for it"]
pub(crate) struct ClauseToken {
    /// The line in force before this clause, restored by nothing -- carried
    /// only so the token is not a zero-sized value that is easy to conjure
    /// from thin air without `enter_clause`.
    _previous: usize,
}

impl Interp {
    /// A Rexx clause begins here: records its line and hands back the
    /// obligation to end it.
    ///
    /// Every site that runs a clause calls this, and the set is wider than
    /// "instructions stepped": a `DO`/`LOOP` header is a clause, and so is
    /// the `END` at which a `WHILE`/`UNTIL` is re-tested -- measured, `do
    /// while zn < sub()` reports `SIGL` 4 on the first test and 7 (the
    /// `END`'s own line) on the second.
    pub(crate) fn enter_clause(&mut self, line: usize) -> ClauseToken {
        let previous = self.clause_state.current_clause_line;
        self.clause_state.current_clause_line = line;
        ClauseToken {
            _previous: previous,
        }
    }

    /// The clause is over: run a `CALL ON` handler that was waiting on it.
    ///
    /// Returns `Some` only when the handler itself ended the program.
    pub(crate) fn end_clause(
        &mut self,
        token: ClauseToken,
        code: &Code<'_>,
        end: ClauseEnd<'_>,
    ) -> Result<Option<Ended>, Failure> {
        let ClauseToken { .. } = token;
        let ClauseEnd::Completed(flow) = end else {
            // A clause that is unwinding never reached a boundary, so it
            // delivers nothing. The token is still consumed here, which is
            // what stops "the error path" being the next place this rule
            // quietly did not apply.
            return Ok(None);
        };
        if self.pending_trap.is_none() {
            return Ok(None);
        }
        if let Some(Flow::Return(Some(value)) | Flow::Exit(Some(value))) = flow {
            self.roots.push_temp(*value);
        }
        self.deliver_pending_trap(code)
    }
}

/// How a clause finished, for [`Interp::end_clause`].
///
/// **Two cases and not an `Option<&Flow>`**, which is what this was first
/// written as and which conflated them: a loop header completes and has no
/// `Flow`, while a failing clause has no `Flow` *and* no boundary. Under the
/// `Option` both passed `None` and the error path started delivering
/// handlers it must not -- caught immediately by
/// `a_pending_trap_whose_activation_is_gone_is_never_delivered`, which is the
/// test that exists for exactly that shape.
pub(crate) enum ClauseEnd<'a> {
    /// The clause ran to its end. `Some` carries what it resolved to, for
    /// the callers that have a `Flow`; a loop header or a `WHILE` re-test
    /// passes `None` because such a clause produces no value at all.
    ///
    /// `flow` may carry an `ObjRef` whose one-clause temps frame is already
    /// popped, and the handler is a nested activation that allocates, so the
    /// value is rooted across it. Measured with a negative control: without
    /// the `push_temp` a `Flow::Return` and a `Flow::Exit` program each panic
    /// on `a live value` under collect-on-every-allocation.
    Completed(Option<&'a Flow>),
    /// The clause is unwinding. No boundary, nothing delivered.
    Failed,
}
