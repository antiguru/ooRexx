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

//! Condition traps and their delivery, `RAISE`, and `SIGNAL`.

use super::{
    ActiveCondition, CallEntry, CallType, Code, ConditionTrap, Cow, Ended, Failure, Flow,
    HandlerExit, Interp, Loud, Novalue, Number, ObjRef, PendingTrap, Raise, Raised, Rc, Search,
    Trap, TrappedCondition, body_of, raise_syntax_condition,
};

impl Interp {
    /// `SIGNAL ON`/`OFF` and `CALL ON`/`OFF`, which are one instruction with
    /// one flag between them.
    pub(crate) fn exec_condition_trap(
        &mut self,
        trap: &ConditionTrap,
        call: bool,
    ) -> Result<Flow, Failure> {
        match &trap.label {
            Some(label) => {
                // The required-string protocol's other arming route: a
                // NOSTRING trap is what turns an object with no string value
                // from a rendering into a raise. `ANY` counts, measured --
                // `signal on any` over `say .environment` runs the handler
                // with `CONDITION('C')` `NOSTRING`. See
                // `Interp::reqstr_armed` for why this only ever sets.
                if matches!(&trap.condition[..], b"NOSTRING" | b"ANY") {
                    self.reqstr_armed = true;
                }
                let entry = Trap {
                    call,
                    label: std::rc::Rc::from(label.as_ref()),
                    delayed: false,
                };
                self.activation_mut()
                    .traps
                    .insert(trap.condition.clone(), entry);
                // `RexxActivation::trapOn`'s second half: arming a trap turns
                // off whatever `::OPTIONS ... SYNTAX` asked for the same
                // condition, so the trap takes it rather than a SYNTAX error
                // preempting the trap. `novalue_trapped` is false because the
                // guard is `trapOff`'s alone.
                self.activation_mut()
                    .condition_syntax
                    .disable_for(&trap.condition, !call, false);
            }
            None => {
                self.activation_mut().traps.remove(&trap.condition);
                // `RexxActivation::trapOff`'s second half, with the one guard
                // that arm carries and `trapOn`'s does not: a `NOVALUE` or
                // `ANY` trap still in the table keeps `NOVALUE`'s escalation
                // where it was.
                let traps = &self.activation().traps;
                let novalue_trapped = traps.get(b"NOVALUE".as_slice()).is_some()
                    || traps.get(b"ANY".as_slice()).is_some();
                self.activation_mut().condition_syntax.disable_for(
                    &trap.condition,
                    !call,
                    novalue_trapped,
                );
            }
        }
        Ok(Flow::Next)
    }

    /// The trap the running activation has enabled for `condition`, if any.
    pub(crate) fn trap_for(&self, condition: &[u8]) -> Option<Trap> {
        let traps = &self.trap_frame()?.traps;
        traps
            .get(condition)
            .or_else(|| traps.get(b"ANY".as_slice()))
            // A trap held while its own `CALL ON` handler runs does not fire
            // -- see `Trap::delayed`, which is what a handler that calls
            // something raising the same condition depends on.
            .filter(|trap| !trap.delayed)
            .cloned()
    }

    /// Raises `NOTREADY` for the stream named `name` -- the name **as the
    /// program wrote it**, which is what `CONDITION('D')` answers.
    ///
    /// Three outcomes, all measured 2026-09-12. A `SIGNAL ON` trap takes it
    /// as a failure, so the raising clause is abandoned. A `CALL ON` trap
    /// queues it against the running activation, and `leave_clause` runs the
    /// handler once the clause finishes -- measured, the clause prints its
    /// own empty value first and the next clause runs after the handler.
    /// Untrapped it is silent unless `::OPTIONS NOTREADY SYNTAX` is in force,
    /// which makes it 98.974 at rc 158.
    ///
    /// **The caller answers normally after `Ok`**: a raise never changes what
    /// the failing call returns -- the residual, `1`, or the null string.
    pub(crate) fn raise_notready(&mut self, name: &[u8]) -> Result<(), Failure> {
        // One `Raised` for both branches, so the queued form can build its
        // condition object here rather than at delivery.
        let raised = Raised {
            description: Some(name.to_vec()),
            ..Raised::condition(Cow::Borrowed("NOTREADY"))
        };
        match self.trap_for(b"NOTREADY") {
            Some(trap) if trap.call => {
                let object = self.build_condition_object(&raised, true)?;
                self.pending_traps.push_back(PendingTrap {
                    condition: b"NOTREADY".as_slice().into(),
                    rc: None,
                    description: Some(name.to_vec()),
                    object: Some(object),
                    // The **running** activation, not its caller: a native
                    // method raises inside the clause that sent to it, and
                    // that clause is still running. The routine-return path
                    // queues against the caller because its own activation is
                    // being popped; nothing is popped here.
                    activation: self.activation().id,
                    queued_during_delivery: false,
                    fragment_depth: self.fragment_depth,
                });
                Ok(())
            }
            Some(_) => Err(Raised {
                description: Some(name.to_vec()),
                ..Raised::condition(Cow::Borrowed("NOTREADY"))
            }
            .into()),
            None => {
                if self.condition_raises_syntax(b"NOTREADY") {
                    return Err(Raised::notready_syntax(name).into());
                }
                Ok(())
            }
        }
    }

    /// Turns an uninitialised variable read into a `NOVALUE` condition --
    /// but only when this activation has a `NOVALUE` trap that could take
    /// it.
    #[inline(always)]
    pub(crate) fn novalue_check(&self, novalue: Novalue, read: ObjRef) -> Result<(), Failure> {
        if novalue == Novalue::Set {
            return Ok(());
        }
        self.novalue_raised(read)
    }

    /// [`Interp::novalue_check`]'s uninitialised half: whether the setting in
    /// force turns this read into a raised `NOVALUE`, into
    /// `::OPTIONS NOVALUE SYNTAX`'s 98.986, or into neither -- the derived
    /// name `read_at` already produced.
    #[cold]
    #[inline(never)]
    fn novalue_raised(&self, read: ObjRef) -> Result<(), Failure> {
        if self.trap_for(b"NOVALUE").is_none_or(|trap| trap.call) {
            if self.condition_raises_syntax(b"NOVALUE") {
                return Err(Raised::unassigned_variable(&self.derived_name_text(read)).into());
            }
            return Ok(());
        }
        // The variable's own derived name, which `CONDITION('D')` and the
        // condition object's `DESCRIPTION` both report -- measured, the
        // oracle answers `ZZUNDEF` for `say zzundef` under `SIGNAL ON
        // NOVALUE`. `derived_name_text` is the same value the
        // `::OPTIONS NOVALUE SYNTAX` branch above already builds; leaving it
        // off here is what made both answer empty.
        let mut raised = Raised::condition(Cow::Borrowed("NOVALUE"));
        raised.description = Some(self.derived_name_text(read));
        Err(raised.into())
    }

    /// The bytes of the derived name an uninitialised read answered.
    fn derived_name_text(&self, read: ObjRef) -> Vec<u8> {
        match read.decode() {
            rexx_core::Decoded::Text(inline) => inline.to_vec(),
            rexx_core::Decoded::Heap { .. } => match self.heap.get(read).map(|object| &object.body)
            {
                Some(rexx_core::Body::Text { bytes, .. }) => bytes.to_vec(),
                other => {
                    debug_assert!(
                        false,
                        "an uninitialised read answered something other than a string: {other:?}"
                    );
                    Vec::new()
                }
            },
            other => {
                debug_assert!(
                    false,
                    "an uninitialised read answered something other than a string: {other:?}"
                );
                Vec::new()
            }
        }
    }

    /// Raises 98.972 where `::OPTIONS LOSTDIGITS SYNTAX` is in force and
    /// `operand` really does carry more digits than the precision in force.
    #[inline(always)]
    pub(crate) fn lostdigits_check(
        &mut self,
        operand: &Number,
        value: ObjRef,
    ) -> Result<(), Failure> {
        if !self.lostdigits_armed {
            return Ok(());
        }
        self.lostdigits_raise(operand, value)
    }

    /// The two-operand form, so an operator pays the gate once.
    #[inline(always)]
    pub(crate) fn lostdigits_check2(
        &mut self,
        left: &Number,
        left_value: ObjRef,
        right: &Number,
        right_value: ObjRef,
    ) -> Result<(), Failure> {
        if !self.lostdigits_armed {
            return Ok(());
        }
        self.lostdigits_raise(left, left_value)?;
        self.lostdigits_raise(right, right_value)
    }

    /// [`Interp::lostdigits_check`]'s armed half: the per-activation setting,
    /// then the operand's own digit count, then the operand's bytes.
    #[cold]
    #[inline(never)]
    fn lostdigits_raise(&mut self, operand: &Number, value: ObjRef) -> Result<(), Failure> {
        if !self.condition_raises_syntax(b"LOSTDIGITS") {
            return Ok(());
        }
        let digits = usize::try_from(self.activation().settings.digits()).unwrap_or(usize::MAX);
        if operand.digit_count() > digits {
            let text = self.string_value_text(value);
            return Err(Raised::lostdigits(&text).into());
        }
        Ok(())
    }

    /// Offers a failure escaping the running activation to that activation's
    /// trap table, and either transfers control or hands the failure back to
    /// keep unwinding.
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
        // A phantom does not trap. `RexxActivation::trap`
        // (`execution/RexxActivation.cpp:2450`) reads `isForwarded` before it
        // looks at any trap table, and this crate reaches the frame it drills
        // to by declining here and letting the failure leave this activation.
        // [`Interp::trap_for`] does the drilling for the callers that ask
        // whether a condition would be trapped at all, which is the same
        // question `RexxActivation::willTrap` answers.
        if self.running_activation().is_some_and(|a| a.forwarded) {
            return Err(failure);
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
        // `Op::Clause`'s region recorded a moment ago and which the clearing
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
        // Built here, on the raising clause, because everything in it is a
        // raise-time fact: `POSITION` is this clause's own line and
        // `STACKFRAMES` the stack as it stands now.
        let object = self.build_condition_object(&raised, false)?;
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
            object: Some(object),
        });
        // What a later `RAISE PROPAGATE` re-raises. See `exec_raise_
        // propagate` for what is and is not measured about it.
        self.active_condition = Some(ActiveCondition {
            raised: *raised,
            site,
            sites,
        });
        // **The failed clause's own boundary, and the one place it can
        // happen** (fix round 3). `Op::Clause`'s region ends a *completing*
        // clause; a clause that raised has not completed, and at that moment
        // nothing yet knows whether it will be trapped here or unwind the
        // activation. This is the point where that is decided in favour of
        // "trapped here, execution continues", so it is the point where a
        // `CALL ON` handler queued by the same clause is owed its run.
        if let Some(exit) = self.deliver_pending_traps(code)? {
            return Ok(Flow::Exit(exit.value()));
        }
        Ok(Flow::Signal(target))
    }

    /// Runs every handler this clause boundary owes, in the order the
    /// conditions were queued, and stops early only if one of them ends the
    /// program.
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
            // Carried from the queue rather than built here. By delivery the
            // raising clause has finished and its routine may have returned,
            // so `POSITION` and `STACKFRAMES` no longer exist to be read --
            // measured, `POSITION` is 7 for a command inside a routine, not
            // the caller's own `call` line.
            object: pending.object,
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
            Err(failure) => {
                self.active_condition = enclosing;
                Err(failure)
            }
            // `EXIT` inside the handler ends the program, exactly as it does
            // inside any other called routine. Nothing will read
            // `active_condition` again, so it is left as it is.
            Ok(exited @ Ended::Exited(_)) => Ok(HandlerExit::from_ended(exited)),
        }
    }

    /// The trap the running activation's **caller** has enabled, or `None`
    /// at top level.
    fn caller_trap_for(&self, condition: &[u8]) -> Option<Trap> {
        let traps = &self.caller_activation()?.traps;
        traps
            .get(condition)
            .or_else(|| traps.get(b"ANY".as_slice()))
            .cloned()
    }

    /// `RAISE`, in all of its forms.
    /// ```text
    /// RAISE SYNTAX n.m RETURN [e]   search from the raising activation outward
    /// RAISE SYNTAX n.m             \  the OUTERMOST activation's trap only;
    /// RAISE SYNTAX n.m EXIT [e]    /  every level in between skips its own
    /// RAISE other ... RETURN [e]      search from the raising activation's CALLER
    /// RAISE other ...              \  no trap at all -- the program ends, and
    /// RAISE other ... EXIT [e]     /  the condition's default action applies
    /// ```
    pub(super) fn exec_raise(&mut self, code: &Code<'_>, raise: &Raise) -> Result<Flow, Failure> {
        if raise.propagate {
            return self.exec_raise_propagate();
        }
        // Each option traces a `>K>` line as it is evaluated, in source
        // order, at this clause's own indent. Measured, all five spellings:
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
        let indent = self.clause_state.current_value_indent;
        let rc_text = match &raise.rc {
            Some(expr) => {
                let value = self.eval(code, expr)?;
                self.roots.push_temp(value);
                // **The line renders the object and the code is its string
                // value**, which `Interp::string_value_text`'s own doc has the
                // split for: `traceKeywordResult(conditionName, rc)`
                // (`instructions/RaiseInstruction.cpp:182`) is handed the
                // object and `rc->requestString()` (`:193`) is what becomes the
                // code. Measured, `trace i` over `raise syntax (1,2)`:
                // `>K>   "SYNTAX" => "an Array"`.
                let traced = self.string_value_text(value);
                let keyword = String::from_utf8_lossy(&raise.condition).into_owned();
                self.trace_keyword(indent, &keyword, &traced);
                Some(self.to_text(value).to_vec())
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
            // The object on the line and its string value in the condition,
            // the same split the `rc` keyword above takes. Measured,
            // `trace i` over `raise syntax 93.900 description (1,2)
            // additional 'x'`: `>K>   "DESCRIPTION" => "an Array"`.
            let traced = self.string_value_text(value);
            self.trace_keyword(indent, "DESCRIPTION", &traced);
            description = Some(self.to_text(value).to_vec());
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
            // **The array is split for every condition, but only `SYNTAX`
            // refuses a value that is not one.** The C++ calls `requestArray`
            // inside its `SYNTAX` branch alone, and that governs the
            // *substitution list* a catalogue message renders from -- which
            // only a `SYNTAX` condition has. The condition object's
            // `ADDITIONAL` carries the raise's own array whatever the
            // condition is: measured, `raise user mycond additional
            // (.array~of('x','y'))` gives a directory holding two items, and
            // pushing the rendered whole value gave one holding `x\ny`.
            // `Raised::message` is the only other reader and runs for a
            // catalogue entry alone, so nothing else sees the split.
            let converted = raise.condition.eq_ignore_ascii_case(b"SYNTAX");
            let slots = self.array_slots_of(value);
            if converted
                && slots.is_none()
                && let Some(kind) = self.operator_operand_gap(value)
            {
                return Err(Loud::object_position("a RAISE ADDITIONAL value", kind).into());
            }
            // The object on the line, the same split the two keywords above
            // take. Measured, `trace i` over `raise syntax 93.900 additional
            // (1,2)`: `>K>   "ADDITIONAL" => "an Array"`.
            let traced = self.string_value_text(value);
            self.trace_keyword(indent, "ADDITIONAL", &traced);
            // The object itself, for the condition object to carry. The
            // substitution list below is a different thing: it is what a
            // catalogue message renders from, and only a `SYNTAX` condition
            // has one.
            self.pending_additional = Some(value);
            match slots {
                Some(slots) => {
                    for slot in slots {
                        additional.push(match slot {
                            Some(item) => self.string_value_text(item),
                            None => Vec::new(),
                        });
                    }
                }
                None => additional.push(self.to_text(value).to_vec()),
            }
        }
        if let Some(items) = &raise.array {
            // **The elements first, then the `>K>` line** -- corrected at
            // Task 9, which owns `>A>` and measured the ordering while
            // adding it. An earlier version of this arm traced the `>K>`
            // first and said so in a comment that claimed "the elements
            // produce no lines of their own"; both halves are false.
            // Measured, `trace i` / `raise syntax 40.4 array('R',,'X')`:
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
                let rendered = self.string_value_text(value);
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
            raised.position = u32::try_from(self.clause_state.line()).unwrap_or(0);
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
                // Built before the raising activation is popped, which is
                // what `STACKFRAMES` and `POSITION` describe. The trap is
                // queued against the caller, but the object is the raise's.
                let queued = Raised {
                    rc: rc.clone(),
                    description: description.clone(),
                    // `RAISE ... ADDITIONAL`'s own list, which the condition
                    // object reports and which this arm used to drop.
                    additional: additional.clone(),
                    position: u32::try_from(self.clause_state.line()).unwrap_or(0),
                    ..Raised::condition(Cow::Owned(String::from_utf8_lossy(&name).into_owned()))
                };
                let object = self.build_condition_object(&queued, true)?;
                self.pending_traps.push_back(PendingTrap {
                    condition: name,
                    rc,
                    description: description.clone(),
                    object: Some(object),
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
                // Captured here, where the raising activation is still
                // current: it is popped by the `Flow::Return` this branch
                // does not take, and a `SIGNAL ON` handler sees the caller's
                // frame rather than this one.
                raised.position = u32::try_from(self.clause_state.line()).unwrap_or(0);
                // The raise's own `ADDITIONAL`, which the condition object
                // reports. Measured: `raise user mycond additional (an
                // array)` gives a directory carrying `ADDITIONAL`, and this
                // arm built the condition without it.
                raised.additional = additional;
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
    /// identical fix for `CALL` (`run.rs:2153-2154` in
    /// the tree this task started from) -- found there by running the
    /// composition rather than reading the code, and true of `SIGNAL` for
    /// the same reason: measured, `interpret "signal there"` reaches an
    /// enclosing `there:`, and `call sub` into `sub:` containing `signal
    /// caller_label` reaches a label back in the caller's own text, because
    /// at this phase every internal `CALL` target shares its caller's exact
    /// body (no `::routine` directive gives it one of its own yet) -- not
    /// because `SIGNAL` reaches across an activation boundary on its own.
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
    pub(crate) fn signal_to_value(&mut self, value: ObjRef) -> Result<Flow, Failure> {
        let text = self.to_text(value).to_vec();
        // `>K>` names the object and the search reads the conversion, which
        // is `RexxInstructionDynamicSignal::execute`'s own order:
        // `traceKeywordResult(VALUE, result)` and then
        // `result->requestString()` (`instructions/SignalInstruction.cpp:217`
        // -`:219`). Measured, oracle rc 240: `signal value .K` with a
        // class-side `makeString` returning `'NOSUCH'` reports `Label
        // "NOSUCH" not found.`, so 16.1 names the conversion where NUMERIC's
        // own errors name the object.
        self.trace_keyword(self.clause_state.current_value_indent, "VALUE", &text);
        let converted = self.required_string_value(value)?;
        if converted == value {
            return self.signal_to_label(&text);
        }
        let text = self.to_text(converted).to_vec();
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
}

/// A `RAISE`'s condition name as `Raised::condition` carries it.
fn condition_name(name: &[u8]) -> Cow<'static, str> {
    Cow::Owned(String::from_utf8_lossy(name).into_owned())
}
