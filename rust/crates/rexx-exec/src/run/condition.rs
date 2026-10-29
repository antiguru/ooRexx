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
    ActiveCondition, CallEntry, CallType, Code, ConditionTrap, Cow, Ended, Entry, Failure, Flow,
    HandlerExit, Interp, Loud, Novalue, Number, ObjRef, PendingTrap, Raise, Raised, Rc, ReplyState,
    Search, Trap, TrappedCondition, body_of, raise_syntax_condition,
};
use crate::error::FailureSite;

impl Interp {
    /// `SIGNAL ON`/`OFF` and `CALL ON`/`OFF`, which are one instruction with
    /// one flag between them.
    pub(crate) fn exec_condition_trap(
        &mut self,
        trap: &ConditionTrap,
        call: bool,
    ) -> Result<Flow, Failure> {
        let delayed = self
            .activation()
            .traps
            .get(&trap.condition)
            .is_some_and(|entry| entry.delayed);
        match &trap.label {
            Some(label) => {
                // An arming route of the required-string protocol: a
                // NOSTRING trap is what turns an object with no string value
                // from a rendering into a raise. `ANY` counts, measured --
                // `signal on any` over `say .environment` runs the handler
                // with `CONDITION('C')` `NOSTRING`. See
                // `Interp::reqstr_armed` for why this only ever sets.
                if matches!(&trap.condition[..], b"NOSTRING" | b"ANY") {
                    self.reqstr_armed = true;
                }
                // The same for LOSTDIGITS, which only a `SIGNAL ON` trap
                // takes: measured, `CALL ON ANY` over `1 / 9999999999` does
                // not run its handler.
                if !call && matches!(&trap.condition[..], b"LOSTDIGITS" | b"ANY") {
                    self.lostdigits_armed = true;
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
        if delayed {
            return self.release_delayed(&trap.condition, trap.label.is_some(), call);
        }
        Ok(Flow::Next)
    }

    /// What rexxref (`condtra.xml`, CALL ON) does with the conditions held
    /// for the delayed trap `key` once a trap instruction for it has been
    /// processed: after `CALL ON` each handler is called, after `SIGNAL ON`
    /// the first is signalled with the object it was raised with, and after
    /// `CALL OFF` or `SIGNAL OFF` each takes its default action, which only
    /// `HALT`'s reports.
    #[cold]
    #[inline(never)]
    fn release_delayed(&mut self, key: &[u8], on: bool, call: bool) -> Result<Flow, Failure> {
        let Some(delayed) = self.delayed_trap_mut(key) else {
            return Ok(Flow::Next);
        };
        let held = std::mem::take(&mut delayed.held);
        match (on, call) {
            (true, true) => {
                let here = self.activation().id;
                for mut pending in held {
                    pending.activation = here;
                    pending.fragment_depth = self.activity.fragment_depth;
                    pending.queued_during_delivery = false;
                    self.activity.pending_traps.push_back(pending);
                }
                Ok(Flow::Next)
            }
            (true, false) => {
                let Some(pending) = held.into_iter().next() else {
                    return Ok(Flow::Next);
                };
                let mut raised = Raised::condition(condition_name(&pending.condition));
                raised.rc = pending.rc;
                raised.description = pending.description;
                match pending.object {
                    Some(object) => {
                        self.activity.reraised_object = Some(object);
                        raised.delivery.held = true;
                    }
                    None => {
                        raised.position =
                            u32::try_from(self.activity.clause_state.line()).unwrap_or(0);
                    }
                }
                Err(raised.into())
            }
            (false, _) if held.iter().any(|pending| *pending.condition == *b"HALT") => {
                Err(Raised::halt().into())
            }
            (false, _) => Ok(Flow::Next),
        }
    }

    /// Moves into `held` each condition queued for the running activation's
    /// trap `key` since `queued_before`, while that trap was delayed: a `RAISE
    /// ... RETURN` in the handler offers its condition to this activation. A
    /// condition [`ignored_while_delayed`] is dropped instead.
    #[cold]
    #[inline(never)]
    fn take_queued_for_delayed(
        &mut self,
        queued_before: usize,
        key: &[u8],
        held: &mut Vec<PendingTrap>,
    ) {
        let here = self.activation().id;
        let depth = self.activity.fragment_depth;
        let mut row = queued_before;
        while row < self.activity.pending_traps.len() {
            let pending = &self.activity.pending_traps[row];
            let matches = !pending.request
                && pending.activation == here
                && pending.fragment_depth == depth
                && (*pending.condition == *key
                    || (key == b"ANY"
                        && self.activation().traps.get(&pending.condition).is_none()));
            if matches && let Some(pending) = self.activity.pending_traps.remove(row) {
                if !ignored_while_delayed(&pending.condition) {
                    held.push(pending);
                }
            } else {
                row += 1;
            }
        }
    }

    /// Holds `pending`, whose trap is delayed, for the innermost delayed trap
    /// it matches; a condition [`ignored_while_delayed`] is dropped instead.
    #[cold]
    #[inline(never)]
    fn hold_delayed(&mut self, pending: PendingTrap) {
        if ignored_while_delayed(&pending.condition) {
            return;
        }
        let key: &[u8] = match self
            .trap_frame()
            .is_some_and(|frame| frame.traps.get(&pending.condition).is_some())
        {
            true => &pending.condition,
            false => b"ANY",
        };
        let key: Box<[u8]> = key.into();
        if let Some(delayed) = self.delayed_trap_mut(&key) {
            delayed.held.push(pending);
        }
    }

    /// The innermost activation's trap `key` delayed while its handler runs.
    fn delayed_trap_mut(&mut self, key: &[u8]) -> Option<&mut crate::DelayedTrap> {
        let activity = &mut self.activity;
        activity
            .running
            .as_deref_mut()
            .into_iter()
            .chain(activity.suspended.iter_mut().rev().map(Box::as_mut))
            .filter_map(|activation| activation.cold.as_deref_mut()?.delayed.as_mut())
            .find(|delayed| *delayed.key == *key)
    }

    /// The trap the running activation has enabled for `condition`, if any.
    pub(crate) fn trap_for(&self, condition: &[u8]) -> Option<Trap> {
        let frame = self.trap_frame()?;
        if ignored_in_debug_pause(frame, condition) {
            return None;
        }
        let traps = &frame.traps;
        // A delayed trap is answered too: its condition queues as for any
        // `CALL ON` trap, and its delivery holds it (`Interp::hold_delayed`).
        traps
            .get(condition)
            .or_else(|| traps.get(b"ANY".as_slice()))
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
    /// The condition object carries `stream` as `ADDITIONAL` and `result`, the
    /// value the caller answers, as `RESULT` (`StreamInfo::notreadyError`).
    pub(crate) fn raise_notready(
        &mut self,
        name: &[u8],
        stream: ObjRef,
        result: ObjRef,
    ) -> Result<(), Failure> {
        // A native `LINEIN` the input reader sent runs in the reader's own
        // activation.
        let running = self.running_activation().map(|activation| activation.id);
        if self.input_dispatcher_takes(running) {
            return Ok(());
        }
        // One `Raised` for both branches, so the queued form can build its
        // condition object here rather than at delivery.
        let raised = Raised {
            description: Some(name.to_vec()),
            ..Raised::condition(Cow::Borrowed("NOTREADY"))
        };
        let trapping = self
            .trap_frame()
            .map_or(self.activation().id, |activation| activation.id);
        match self.trap_for(b"NOTREADY") {
            // Raised again while the trap is delayed, by a handler running or
            // by a raise this clause queued.
            Some(trap)
                if trap.call && (trap.delayed || self.notready_trap_delayed_at(trapping)) =>
            {
                Ok(())
            }
            Some(trap) if trap.call => {
                self.activity.pending_additional = Some(stream);
                self.activity.pending_result = Some(result);
                let object = self.build_condition_object(&raised, Some(true))?;
                self.activity.native_frames_owed.push(object);
                self.activity.pending_traps.push_back(PendingTrap {
                    condition: b"NOTREADY".as_slice().into(),
                    rc: None,
                    description: Some(name.to_vec()),
                    object: Some(object),
                    // The activation whose table matched, not its caller: a
                    // native method raises inside the clause that sent to it,
                    // and that clause is still running. Under a non-continuing
                    // `FORWARD` that is the frame beneath the forwarding
                    // phantoms. The routine-return path queues against the
                    // caller because its own activation is being popped;
                    // nothing is popped here.
                    activation: trapping,
                    queued_during_delivery: false,
                    request: false,
                    fragment_depth: self.activity.fragment_depth,
                    trace_failure: None,
                });
                Ok(())
            }
            Some(_) => {
                self.activity.pending_additional = Some(stream);
                self.activity.pending_result = Some(result);
                Err(Raised {
                    description: Some(name.to_vec()),
                    ..Raised::condition(Cow::Borrowed("NOTREADY"))
                }
                .into())
            }
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
    /// it -- or into the failure of the `.local~novalue` hook the read sent.
    #[inline(always)]
    pub(crate) fn novalue_check(&self, novalue: Novalue, read: ObjRef) -> Result<(), Failure> {
        if novalue == Novalue::Set {
            return Ok(());
        }
        self.novalue_raised(read)
    }

    /// [`Interp::novalue_check`]'s uninitialised half: what the read's
    /// `.local~novalue` hook came to where the read sent it, and otherwise
    /// whether the setting in force turns this read into a raised `NOVALUE`,
    /// into `::OPTIONS NOVALUE SYNTAX`'s 98.986, or into neither -- the
    /// derived name `read_at` already produced.
    #[cold]
    #[inline(never)]
    fn novalue_raised(&self, read: ObjRef) -> Result<(), Failure> {
        if let Some(hooked) = self.novalue_hooked.take() {
            return hooked;
        }
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

    /// The value of an uninitialised read whose derived name is `derived`,
    /// in `RexxActivation::handleNovalueEvent`'s order
    /// (`RexxActivation.cpp:2645-2674`): where `::OPTIONS NOVALUE SYNTAX` is
    /// about to raise 98.986, `derived`; otherwise the `.local~novalue`
    /// hook's answer, or `derived` where it answers none. The read is still
    /// [`Novalue::Unset`]: where the hook answered or failed, that outcome
    /// waits in `Interp::novalue_hooked` for the `novalue_check` after the
    /// read.
    #[cold]
    #[inline(never)]
    pub(crate) fn novalue_answer(&mut self, derived: ObjRef) -> ObjRef {
        let syntax = self.trap_for(b"NOVALUE").is_none_or(|trap| trap.call)
            && self.condition_raises_syntax(b"NOVALUE");
        if syntax {
            return derived;
        }
        match self.novalue_hook(derived) {
            Ok(Some(value)) => {
                self.novalue_hooked.set(Some(Ok(())));
                value
            }
            Ok(None) => derived,
            Err(failure) => {
                self.novalue_hooked.set(Some(Err(failure)));
                derived
            }
        }
    }

    /// The object `.local` holds under `NOVALUE`, sent `NOVALUE` with the
    /// uninitialised variable's derived name: an answer other than `.nil` is
    /// the read's value, and the variable stays unset
    /// (`RexxActivation::novalueHandler`, `RexxActivation.cpp:4763-4773`).
    fn novalue_hook(&mut self, read: ObjRef) -> Result<Option<ObjRef>, Failure> {
        let Some(handler) = self.local_route(b"NOVALUE")? else {
            return Ok(None);
        };
        self.roots.activity_mut().push_temp(read);
        let caller = self.caller();
        let answer = pinned!(
            self,
            crate::pinning::PinKind::NovalueHook,
            self.send_message(handler, b"NOVALUE", None, &[Some(read)], caller)
        )?;
        Ok(answer.filter(|value| *value != ObjRef::NIL))
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

    /// Raises LOSTDIGITS where `operand` carries more digits than the
    /// precision in force: to a `SIGNAL ON` trap, else as 98.972 where
    /// `::OPTIONS LOSTDIGITS SYNTAX` is in force
    /// (`NumberString::checkLostDigits`, `Activity::raiseCondition`).
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

    /// The comparison form. A numeric comparison is a subtraction carried out
    /// at DIGITS minus FUZZ (rexxref, "Numeric Comparisons"), so under FUZZ
    /// each operand is checked again at that precision, whatever the signs.
    #[inline(always)]
    pub(crate) fn lostdigits_check_compare(
        &mut self,
        left: &Number,
        left_value: ObjRef,
        right: &Number,
        right_value: ObjRef,
        fuzz: u64,
    ) -> Result<(), Failure> {
        if !self.lostdigits_armed {
            return Ok(());
        }
        self.lostdigits_raise(left, left_value)?;
        self.lostdigits_raise(right, right_value)?;
        if fuzz == 0 {
            return Ok(());
        }
        self.lostdigits_fuzz_raise(
            left,
            |interp| interp.unsent_string_value_text(left_value),
            right,
            |interp| interp.unsent_string_value_text(right_value),
            fuzz,
        )
    }

    /// The FUZZ half of [`Interp::lostdigits_check_compare`], for a caller
    /// whose operands carry no object: a `MAX` or `MIN` candidate and the
    /// answer so far, or a controlled loop's variable and its `TO` bound.
    pub(crate) fn lostdigits_check_fuzz(
        &mut self,
        left: &Number,
        left_text: impl FnOnce(&mut Self) -> Vec<u8>,
        right: &Number,
        right_text: impl FnOnce(&mut Self) -> Vec<u8>,
        fuzz: u64,
    ) -> Result<(), Failure> {
        if !self.lostdigits_armed || fuzz == 0 {
            return Ok(());
        }
        self.lostdigits_fuzz_raise(left, left_text, right, right_text, fuzz)
    }

    #[cold]
    #[inline(never)]
    fn lostdigits_fuzz_raise(
        &mut self,
        left: &Number,
        left_text: impl FnOnce(&mut Self) -> Vec<u8>,
        right: &Number,
        right_text: impl FnOnce(&mut Self) -> Vec<u8>,
        fuzz: u64,
    ) -> Result<(), Failure> {
        let digits = self.activation().settings.digits().saturating_sub(fuzz);
        self.lostdigits_raise_at(left, digits, left_text)?;
        self.lostdigits_raise_at(right, digits, right_text)
    }

    /// [`Interp::lostdigits_check`] with the condition's description from
    /// `description`, which runs only for a raise.
    pub(crate) fn lostdigits_check_with(
        &mut self,
        operand: &Number,
        description: impl FnOnce(&mut Self) -> Vec<u8>,
    ) -> Result<(), Failure> {
        if !self.lostdigits_armed {
            return Ok(());
        }
        self.lostdigits_raise_with(operand, description)
    }

    /// [`Interp::lostdigits_check`]'s armed half. The description is the
    /// operand as written: measured, `' 1111111111 ' + 1` names
    /// `' 1111111111 '` with its blanks.
    #[cold]
    #[inline(never)]
    fn lostdigits_raise(&mut self, operand: &Number, value: ObjRef) -> Result<(), Failure> {
        self.lostdigits_raise_with(operand, |interp| interp.unsent_string_value_text(value))
    }

    /// The operand's own digit count, then the trap, then the per-activation
    /// setting; `description` renders the operand only once one of them takes
    /// it.
    fn lostdigits_raise_with(
        &mut self,
        operand: &Number,
        description: impl FnOnce(&mut Self) -> Vec<u8>,
    ) -> Result<(), Failure> {
        let digits = self.activation().settings.digits();
        self.lostdigits_raise_at(operand, digits, description)
    }

    /// [`Interp::lostdigits_raise_with`] against `digits` rather than the
    /// setting in force.
    fn lostdigits_raise_at(
        &mut self,
        operand: &Number,
        digits: u64,
        description: impl FnOnce(&mut Self) -> Vec<u8>,
    ) -> Result<(), Failure> {
        let digits = usize::try_from(digits).unwrap_or(usize::MAX);
        if operand.digit_count() <= digits {
            return Ok(());
        }
        if self.trap_for(b"LOSTDIGITS").is_some_and(|trap| !trap.call) {
            let text = description(self);
            return Err(Raised {
                description: Some(text),
                ..Raised::condition(Cow::Borrowed("LOSTDIGITS"))
            }
            .into());
        }
        if self.condition_raises_syntax(b"LOSTDIGITS") {
            let text = description(self);
            return Err(Raised::lostdigits(&text).into());
        }
        Ok(())
    }

    /// Offers a failure escaping the running activation to that activation's
    /// trap table, and either transfers control or hands the failure back to
    /// keep unwinding.
    pub(crate) fn offer_to_trap(&mut self, failure: Failure) -> Result<Flow, Failure> {
        let Failure::Raised(raised) = &failure else {
            return Err(failure);
        };
        if raised.condition == "SYNTAX"
            && let Some(replaced) = self.notify_slot_failure(raised)
        {
            return self.offer_to_trap(replaced);
        }
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
            // `RexxActivation::raiseExit` (`execution/RexxActivation.cpp:1741`):
            // an internal call hands the raise to its parent, and the first
            // activation that is not one raises it, from there outward.
            Search::Top if self.activation().entry == Entry::InternalCall => return Err(failure),
            Search::Top => {
                let Failure::Raised(mut raised) = failure else {
                    unreachable!("matched Failure::Raised immediately above")
                };
                raised.delivery.search = Search::Here;
                return self.offer_to_trap(Failure::Raised(raised));
            }
            Search::AboveTop if self.activation().entry == Entry::InternalCall => {
                return Err(failure);
            }
            Search::AboveTop => {
                let Failure::Raised(mut raised) = failure else {
                    unreachable!("matched Failure::Raised immediately above")
                };
                raised.delivery.search = Search::Here;
                return Err(Failure::Raised(raised));
            }
            Search::Nobody => return Err(failure),
        }
        // A phantom does not trap. `RexxActivation::trap`
        // (`execution/RexxActivation.cpp:2450`) reads `isForwarded` before it
        // looks at any trap table, and this crate reaches the frame it drills
        // to by declining here and letting the failure leave this activation.
        // [`Interp::trap_for`] does the drilling for the callers that ask
        // whether a condition would be trapped at all, which is the same
        // question `RexxActivation::willTrap` answers.
        if self
            .running_activation()
            .is_some_and(|a| a.flags.forwarded())
        {
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
        // `RexxInstructionSignalOn::trap` turns the trap off before it raises
        // 16.1, so the 16.1 is offered to this same table: measured, `signal
        // on novalue` with no `NOVALUE:` label is trapped by `signal on
        // syntax` at the raising clause (upstream bug 1877).
        let target = match self.resolve_signal_target(&trap.label) {
            Ok(target) => target,
            Err(missing) => {
                // The 16.1 is raised here, at the raising clause of the Rexx
                // level holding the trap: nothing of the condition that found
                // the trap, nor of the levels it left, belongs to it.
                let site = self.activity.failure_site.take();
                self.clear_failure_levels();
                self.activity.failure_site = site;
                self.activity.pending_additional = None;
                self.activity.pending_result = None;
                self.activity.pending_rc = None;
                let traps = &mut self.activation_mut().traps;
                match traps.get(raised.condition.as_bytes()) {
                    Some(_) => traps.remove(raised.condition.as_bytes()),
                    None => traps.remove(b"ANY".as_slice()),
                }
                return self.offer_to_trap(missing);
            }
        };
        let Failure::Raised(mut raised) = failure else {
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
        let site = self.activity.failure_site.take();
        let sites = std::mem::take(&mut self.activity.failure_sites);
        let frames = std::mem::take(&mut self.activity.failure_frames);
        for &frame in &frames {
            self.roots.activity_mut().push_temp(frame);
        }
        self.activity.failure_frame = None;
        let origin = self.activity.failure_origin.take();
        self.hold_origin_package(origin);
        let propagated = std::mem::take(&mut self.activity.failure_propagated);
        let reraised = std::mem::take(&mut self.activity.failure_reraised);
        self.activity.propagate_absorb = 0;
        self.set_sigl(self.activity.clause_state.line());
        if let Some(rc) = &raised.rc {
            let value = self.text(rc);
            self.assign_by_name(b"RC", value);
        }
        // What `CONDITION()` reports for the rest of this activation, kept
        // apart from `active_condition` below, which carries the echo sites
        // `RAISE PROPAGATE` puts back.
        // Built here, on the raising clause, because everything in it is a
        // raise-time fact: `POSITION` is the innermost level's line and
        // `STACKFRAMES` the levels left above the stack as it stands now.
        let unwound = crate::condition::Unwound {
            sites: &sites,
            frames,
            origin,
            propagated,
            reraised,
        };
        let object = match self.activity.reraised_object.take() {
            Some(object) if raised.delivery.held => self.trap_held_condition_object(object)?,
            // `RexxActivation::raise` re-raises any other condition with the
            // object it was built with, where `Activity::reraiseException`
            // moves a SYNTAX condition's position to this level.
            Some(object) if raised.condition != "SYNTAX" => {
                self.trap_propagated_condition_object(object)?
            }
            Some(object) => self.trap_reraised_condition_object(object, unwound.origin)?,
            None => self.build_trapped_condition_object(&raised, &unwound)?,
        };
        raised.delivery.held = false;
        self.attach_condition(object);
        self.activation_mut()
            .replace_condition(Some(TrappedCondition {
                name: raised.condition.as_bytes().into(),
                // Only a `SYNTAX` condition has a `CODE` item at all
                // (`Activity::createExceptionObject` is the one place it is put
                // in), so the name and not the number is the test: measured,
                // a trapped `HALT` reports `E` as the null string.
                code_sub: (raised.condition == "SYNTAX").then_some(raised.sub),
                call: false,
                description: raised.description.clone(),
                object: Some(object),
            }));
        // What a later `RAISE PROPAGATE` re-raises. See `exec_raise_
        // propagate` for what is and is not measured about it.
        let replied = self.activation().reply != ReplyState::None;
        self.activation_mut().cold_mut().active_condition = Some(Rc::new(ActiveCondition {
            raised: *raised,
            site,
            sites,
            replied,
        }));
        // **The failed clause's own boundary, and the one place it can
        // happen** (fix round 3). `Op::Clause`'s region ends a *completing*
        // clause; a clause that raised has not completed, and at that moment
        // nothing yet knows whether it will be trapped here or unwind the
        // activation. This is the point where that is decided in favour of
        // "trapped here, execution continues", so it is the point where a
        // `CALL ON` handler queued by the same clause is owed its run. A halt
        // request waits for the label's clause: measured, the oracle reports
        // an untrapped one there, and `SIGL` names it for a trapped one.
        if let Some(exit) = self.deliver_pending(false)? {
            return Ok(Flow::Exit(exit.value()));
        }
        Ok(Flow::Signal(target))
    }

    /// Runs every handler this clause boundary owes, in the order the
    /// conditions were queued, and stops early only if one of them ends the
    /// program.
    pub(crate) fn deliver_pending_traps(&mut self) -> Result<Option<HandlerExit>, Failure> {
        self.deliver_pending(true)
    }

    /// [`Interp::deliver_pending_traps`], leaving a halt request queued where
    /// `requests` is false.
    fn deliver_pending(&mut self, requests: bool) -> Result<Option<HandlerExit>, Failure> {
        // Snapshotted rather than re-read, so that anything a handler queues
        // lands beyond the prefix this boundary is answering for.
        let mut owed = self.activity.pending_traps.len();
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
            let depth = self.activity.fragment_depth;
            let Some(at) = self
                .activity
                .pending_traps
                .iter()
                .take(owed)
                .position(|pending| {
                    pending.activation == here
                        && pending.fragment_depth == depth
                        && (requests || !pending.request)
                })
            else {
                return Ok(None);
            };
            let pending = self
                .activity
                .pending_traps
                .remove(at)
                .expect("position answered an index inside the queue");
            owed -= 1;
            if let Some(failure) = pending.trace_failure {
                return Err(failure);
            }
            if pending.request {
                self.raise_requested_halt(pending.description)?;
                continue;
            }
            if let Some(exit) = self.deliver_one_pending_trap(pending)? {
                return Ok(Some(exit));
            }
        }
        Ok(None)
    }

    /// One queued condition's handler, run at the boundary that owes it, and
    /// run again for each condition held while it ran.
    fn deliver_one_pending_trap(
        &mut self,
        pending: PendingTrap,
    ) -> Result<Option<HandlerExit>, Failure> {
        let (exit, held) = self.run_pending_handler(pending)?;
        if exit.is_some() || held.is_empty() {
            return Ok(exit);
        }
        self.redeliver_held(held)
    }

    /// rexxref (`condtra.xml`, CALL ON): on the handler's `RETURN` the trap
    /// is no longer delayed and the handler is called again at once for
    /// each condition `held` for it, in the order raised.
    #[cold]
    #[inline(never)]
    fn redeliver_held(&mut self, held: Vec<PendingTrap>) -> Result<Option<HandlerExit>, Failure> {
        let frame = self.roots.activity_mut().push_frame();
        let mut queue = std::collections::VecDeque::from(held);
        let mut delivered = Ok(None);
        while let Some(pending) = queue.pop_front() {
            // The conditions waiting are reachable from this queue alone.
            self.roots.activity_mut().pop_frame(frame);
            for object in std::iter::once(&pending)
                .chain(&queue)
                .filter_map(|pending| pending.object)
            {
                self.roots.activity_mut().push_temp(object);
            }
            match self.run_pending_handler(pending) {
                Ok((None, held)) => {
                    for pending in held.into_iter().rev() {
                        queue.push_front(pending);
                    }
                }
                ended => {
                    delivered = ended.map(|(exit, _)| exit);
                    break;
                }
            }
        }
        self.roots.activity_mut().pop_frame(frame);
        delivered
    }

    /// [`Interp::deliver_one_pending_trap`]'s one handler run, answering with
    /// the conditions held for its trap while it ran.
    fn run_pending_handler(
        &mut self,
        pending: PendingTrap,
    ) -> Result<(Option<HandlerExit>, Vec<PendingTrap>), Failure> {
        let Some(trap) = self.trap_for(&pending.condition) else {
            return Ok((None, Vec::new()));
        };
        if trap.delayed {
            self.hold_delayed(pending);
            return Ok((None, Vec::new()));
        }
        if !trap.call {
            // A `SIGNAL ON` trap never gets here: `exec_raise` throws for
            // that half instead, so the transfer happens where the raise is
            // rather than one clause later. Declining rather than asserting
            // keeps a future raiser that forgets the distinction from
            // silently running a `SIGNAL` handler as a call.
            return Ok((None, Vec::new()));
        }
        self.set_sigl(self.activity.clause_state.line());
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
        // Set here for the handler's activation to inherit
        // (`RexxActivation.cpp:3340`), and put back once it returns.
        let enclosing = self
            .activation_mut()
            .cold_mut()
            .active_condition
            .replace(Rc::new(ActiveCondition {
                raised,
                site: None,
                sites: Vec::new(),
                replied: false,
            }));
        let key: Box<[u8]> = pending.condition.clone();
        // The entry the trap was found under, `ANY` where the condition has
        // none: the trap instruction's own condition is what the oracle
        // delays (`CallInstruction.cpp:603`, `:632`).
        let delayed: Box<[u8]> = match self
            .trap_frame()
            .is_some_and(|frame| frame.traps.get(&key).is_some())
        {
            true => key.clone(),
            false => b"ANY".as_slice().into(),
        };
        // **Delayed, not removed** (`Trap::delayed`). The two are the same
        // to every lookup that decides whether to trap, and different to
        // `CONDITION('S')` alone, which reports `DELAY` here and `OFF` for a
        // trap that is absent. Nothing else moves: an earlier version of
        // this comment claimed the flag also protects a handler's own `CALL
        // OFF`, and that is false -- the handler's table is a copy, so its
        // `CALL OFF` never reached this one to be undone.
        let recorded = match self.activation_mut().traps.get_mut(&delayed) {
            Some(trap) => {
                trap.delayed = true;
                true
            }
            None => false,
        };
        if recorded {
            self.activation_mut().cold_mut().delayed = Some(crate::DelayedTrap {
                key: delayed.clone(),
                held: Vec::new(),
            });
        }
        // What `CONDITION()` answers inside the handler. Set on *this*
        // activation and restored afterwards, because the handler inherits
        // its copy at call time and the caller must be left as it was --
        // measured, `condition()` back in the caller after a `CALL ON`
        // handler returns is the null string.
        let enclosing_condition = self
            .activation_mut()
            .replace_condition(Some(TrappedCondition {
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
            }));
        let queued_before = self.activity.pending_traps.len();
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
        // The condition object is the handler's one argument
        // (`RexxInstructionCallOn::trap`), whether it is a label, a builtin or
        // an external routine.
        let arguments = pending.object.map(Some).into_iter().collect();
        let ended = pinned!(
            self,
            crate::pinning::PinKind::TrapHandler,
            self.resolve_and_run_call_over(
                &trap.label,
                true,
                arguments,
                CallType::Subroutine,
                CallEntry::Trap,
            )
        );
        // A trap queued by the handler that just ran is not one the
        // interrupted clause owes, and `in_clause`'s tripwire has to be able
        // to tell the two apart -- see the field's own doc comment.
        for pending in self.activity.pending_traps.iter_mut().skip(queued_before) {
            pending.queued_during_delivery = true;
        }
        self.activation_mut().replace_condition(enclosing_condition);
        // `trapUndelay`. The `if let` mirrors the C++ testing the handler
        // for null before enabling it; nothing a Rexx program can do
        // removes the entry between here and the delay above, so the arm is
        // structural rather than a case anything reaches.
        if let Some(trap) = self.activation_mut().traps.get_mut(&delayed) {
            trap.delayed = false;
        }
        let held = match recorded {
            true => self
                .activation_mut()
                .cold
                .as_deref_mut()
                .and_then(|cold| cold.delayed.take())
                .map(|delayed| delayed.held)
                .unwrap_or_default(),
            false => Vec::new(),
        };
        match ended {
            // The handler returned; execution resumes at the clause after
            // the one that finished.
            Ok(Ended::Returned(_)) => {
                self.activation_mut().cold_mut().active_condition = enclosing;
                let mut held = held;
                if recorded {
                    self.take_queued_for_delayed(queued_before, &delayed, &mut held);
                }
                Ok((None, held))
            }
            // The handler failed rather than returned.
            Err(failure) => {
                self.activation_mut().cold_mut().active_condition = enclosing;
                Err(failure)
            }
            // `EXIT` inside the handler ends the program, exactly as it does
            // inside any other called routine. Nothing will read
            // `active_condition` again, so it is left as it is.
            Ok(exited @ Ended::Exited(_)) => Ok((HandlerExit::from_ended(exited), Vec::new())),
        }
    }

    /// Whether a `CALL ON` delivery of `condition` is already queued for
    /// `activation`, which puts its trap in the delayed state.
    pub(crate) fn call_trap_delayed_at(
        &self,
        activation: crate::activation::ActivationId,
        condition: &[u8],
    ) -> bool {
        self.activity.pending_traps.iter().any(|pending| {
            !pending.request
                && pending.activation == activation
                && pending.condition.eq_ignore_ascii_case(condition)
        })
    }

    /// Whether a condition already queued at `activation` put the `CALL ON`
    /// trap taking `NOTREADY` there into the delayed state: a `NOTREADY`, or,
    /// where `CALL ON ANY` takes it, any condition that trap took. An internal
    /// routine inherits its caller's DELAY state (rexxref condtra.xml, Notes:
    /// the state of each trap is saved on entry to a subroutine), so one
    /// queued at a caller its internal calls reach counts too, while the
    /// routine's trap is still the one it inherited: a trap it set itself is
    /// not delayed by its caller's.
    fn notready_trap_delayed_at(&self, activation: crate::activation::ActivationId) -> bool {
        if self.activity.pending_traps.is_empty() {
            return false;
        }
        let frames: Vec<&crate::activation::Activation> = self
            .frames()
            .skip_while(|frame| frame.id != activation)
            .collect();
        let mut chain = Vec::new();
        for (at, frame) in frames.iter().enumerate() {
            chain.push(frame.id);
            let inherited = frame.entry == crate::activation::Entry::InternalCall
                && frames
                    .get(at + 1)
                    .is_some_and(|caller| same_notready_trap(frame, caller));
            if !inherited {
                break;
            }
        }
        let traps = self.trap_frame().map(|frame| &frame.traps);
        let by_any = traps.is_some_and(|traps| traps.get(b"NOTREADY".as_slice()).is_none());
        self.activity.pending_traps.iter().any(|pending| {
            !pending.request
                && chain.contains(&pending.activation)
                && (pending.condition.eq_ignore_ascii_case(b"NOTREADY")
                    || (by_any
                        && traps.is_some_and(|traps| traps.get(&pending.condition).is_none())))
        })
    }

    /// The trap the activation at `depth` has enabled, or `None` past the
    /// outermost level.
    fn trap_at_depth(&self, depth: usize, condition: &[u8]) -> Option<Trap> {
        let frame = self.frame_at(depth)?;
        if ignored_in_debug_pause(frame, condition) {
            return None;
        }
        let traps = &frame.traps;
        traps
            .get(condition)
            .or_else(|| {
                traps
                    .get(b"ANY".as_slice())
                    .filter(|any| !any.call || call_on_any_takes(condition))
            })
            .cloned()
    }

    /// `RAISE`, in all of its forms.
    /// ```text
    /// RAISE SYNTAX n.m RETURN [e]   search from the raising activation outward
    /// RAISE SYNTAX n.m             \  the nearest activation not an internal
    /// RAISE SYNTAX n.m EXIT [e]    /  call, then outward; internal calls skip
    /// RAISE other ... RETURN [e]      search from the raising activation's CALLER
    /// RAISE other ...              \  the nearest activation not an internal
    /// RAISE other ... EXIT [e]     /  call, then its CALLER, as RETURN does
    /// ```
    pub(super) fn exec_raise(&mut self, code: &Code<'_>, raise: &Raise) -> Result<Flow, Failure> {
        if raise.propagate {
            return self.exec_raise_propagate(code, raise);
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
        let indent = self.activity.clause_state.current_value_indent;
        let rc_text = match &raise.rc {
            Some(expr) => {
                let value = self.eval(code, expr)?;
                self.roots.activity_mut().push_temp(value);
                // **The line renders the object and the code is its string
                // value**, which `Interp::string_value_text`'s own doc has the
                // split for: `traceKeywordResult(conditionName, rc)`
                // (`instructions/RaiseInstruction.cpp:182`) is handed the
                // object and `rc->requestString()` (`:193`) is what becomes the
                // code. Measured, `trace i` over `raise syntax (1,2)`:
                // `>K>   "SYNTAX" => "an Array"`.
                let traced = self.string_representation(value)?;
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
            self.roots.activity_mut().push_temp(value);
            // The object on the line and its string value in the condition,
            // the same split the `rc` keyword above takes. Measured,
            // `trace i` over `raise syntax 93.900 description (1,2)
            // additional 'x'`: `>K>   "DESCRIPTION" => "an Array"`.
            let traced = self.string_representation(value)?;
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
            let mut value = self.eval(code, expr)?;
            self.roots.activity_mut().push_temp(value);
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
            let mut slots = self.array_slots_of(value);
            // The object on the line, the same split the two keywords above
            // take. Measured, `trace i` over `raise syntax 93.900 additional
            // (1,2)`: `>K>   "ADDITIONAL" => "an Array"`.
            let traced = self.string_representation(value)?;
            self.trace_keyword(indent, "ADDITIONAL", &traced);
            if converted && slots.is_none() {
                // `requestArray`, a string's lines included, and 98.939 for
                // `TheNilObject` or a multi-dimensional answer.
                let Some(array) = self.single_dimension_request(value)? else {
                    return Err(Raised::syntax_additional().into());
                };
                value = array;
                slots = self.array_slots_of(array);
            }
            // The object itself, for the condition object to carry. The
            // substitution list below is a different thing: it is what a
            // catalogue message renders from, and only a `SYNTAX` condition
            // has one.
            self.activity.pending_additional = Some(value);
            match slots {
                Some(slots) => {
                    for slot in slots {
                        additional.push(match slot {
                            Some(item) => self.string_value_text(item)?,
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
            let mut held = Vec::with_capacity(items.len());
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
                    held.push(None);
                    continue;
                };
                let value = self.eval(code, expr)?;
                self.roots.activity_mut().push_temp(value);
                held.push(Some(value));
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
                let rendered = self.string_value_text(value)?;
                if self.traced_mode().intermediates {
                    let traced = self.string_representation(value)?;
                    self.trace_argument(indent, &traced);
                    self.trace_argument(indent, &traced);
                }
                additional.push(rendered);
            }
            // The condition object's `ADDITIONAL` is the array of the items
            // themselves, as `ADDITIONAL`'s own value is.
            self.activity.pending_additional = Some(self.security_arguments_array(&held));
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
        let result = self.raise_result(code, raise)?;
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
            raised.position = u32::try_from(self.activity.clause_state.line()).unwrap_or(0);
            raised.delivery.search = if returns { Search::Here } else { Search::Top };
            // The condition object's `RESULT` (`Activity::raiseException`).
            self.activity.pending_result = result;
            return Err(raised.into());
        }

        // Every other condition. `HALT` is the one whose untrapped default
        // action reports; the rest are silent, and both halves are below.
        let halt = raise.condition.as_ref() == b"HALT";
        // No tail, or `EXIT`: `RexxActivation::raiseExit` (`execution/
        // RexxActivation.cpp:1741`) leaves every internal call, and the first
        // activation that is not one raises it as `RETURN` does, from there.
        let exits = !returns;
        let sender = self.raise_sender(returns);
        let finish = |result| match exits {
            true => Flow::Exit(result),
            false => Flow::Return(result),
        };

        // This routine returns `result`, and the condition is offered to the
        // caller -- the input reader's dispatcher, where the caller is the one
        // that sent the reader's `LINEIN`.
        let caller = sender.activation;
        if self.input_dispatcher_takes(caller) {
            return Ok(finish(result));
        }
        let name: Box<[u8]> = raise.condition.clone();
        // `RC` for `ERROR`/`FAILURE` is the raise's own argument, measured at
        // `rc= 5` for `raise error 5` trapped one level up. `SYNTAX`'s own
        // `RC` is the major and is filled in by `Raised::syntax` above.
        let rc = match raise.condition.as_ref() {
            b"ERROR" | b"FAILURE" => rc_text,
            _ => None,
        };
        let trap = caller.and_then(|_| self.trap_at_depth(sender.depth, &name));
        match trap {
            Some(trap) if trap.call && sender.reply_issued && result.is_some() => {
                Err(Raised::return_after_reply().into())
            }
            // Raised again while the trap is delayed by its first raise.
            Some(trap)
                if trap.call
                    && ignored_while_delayed(&raise.condition)
                    && caller.is_some_and(|caller| self.call_trap_delayed_at(caller, &name)) =>
            {
                Ok(finish(result))
            }
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
                    position: u32::try_from(self.activity.clause_state.line()).unwrap_or(0),
                    ..Raised::condition(Cow::Owned(String::from_utf8_lossy(&name).into_owned()))
                };
                // rexxref (`condtra.xml`, RESULT): the `RETURN` or `EXIT`
                // value, which the oracle drops (`RexxActivation::raise` passes
                // its `result` member, `RexxActivation.cpp:1827`).
                self.activity.pending_result = result;
                let object = self.build_condition_object(&queued, Some(true))?;
                self.activity.pending_traps.push_back(PendingTrap {
                    condition: name,
                    rc,
                    description: description.clone(),
                    object: Some(object),
                    // The caller's own identity -- this activation is about
                    // to be popped, and `trap_at_depth` above just read
                    // that same activation's table. See the field's own doc
                    // comment for the three transcripts behind it.
                    activation: caller
                        .expect("a raise reaching here has a caller to queue against"),
                    // Set by `deliver_pending_traps` if this turns out to have
                    // been queued while a handler was running, which is not
                    // knowable here: this is the raise, not the delivery.
                    queued_during_delivery: false,
                    request: false,
                    // Which `INTERPRET` fragment's queue this joins. The
                    // raising activation is about to be popped and the depth
                    // is not its own -- a fragment does not push an activation
                    // here -- so it is read straight off `Interp`, where the
                    // `Interpret` arm maintains it. A tail-less raise leaves
                    // the fragments of every level it unwinds.
                    fragment_depth: self.sender_fragment_depth(&sender),
                    trace_failure: None,
                });
                Ok(finish(result))
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
                raised.position = u32::try_from(self.activity.clause_state.line()).unwrap_or(0);
                // The raise's own `ADDITIONAL`, which the condition object
                // reports. Measured: `raise user mycond additional (an
                // array)` gives a directory carrying `ADDITIONAL`, and this
                // arm built the condition without it.
                raised.additional = additional;
                raised.delivery.search = sender.search();
                self.activity.pending_result = result;
                Err(raised.into())
            }
            // Nothing traps it. `HALT` and `NOMETHOD` report; everything else
            // is ignored outright and the routine simply returns its value -- measured,
            // `raise user foo return 'RETVAL-88'` with no trap anywhere
            // prints `RETVAL-88` and the caller carries on.
            // `RexxActivation::raise` reports an untrapped `HALT` or `NOMETHOD`
            // as an ordinary error of the level that raises it.
            None if halt => {
                let mut raised = Raised::halt();
                raised.delivery.search = if exits { Search::Top } else { Search::Here };
                Err(raised.into())
            }
            None if name.as_ref() == b"NOMETHOD" => {
                let mut raised = match (self.activity.pending_additional.take(), description) {
                    (Some(receiver), description) => {
                        let receiver = self.string_value_text(receiver)?;
                        Raised::no_method(&receiver, &description.unwrap_or_default())
                    }
                    (None, _) => Raised::syntax(97, 5, Vec::new()),
                };
                raised.delivery.search = if exits { Search::Top } else { Search::Here };
                Err(raised.into())
            }
            // `returnFrom`'s check (`RexxActivation::raise`).
            None if sender.reply_issued && result.is_some() => {
                Err(Raised::return_after_reply().into())
            }
            None => Ok(finish(result)),
        }
    }

    /// The value of a `RAISE`'s `RETURN` or `EXIT` expression, traced.
    fn raise_result(&mut self, code: &Code<'_>, raise: &Raise) -> Result<Option<ObjRef>, Failure> {
        let Some(expr) = raise
            .result
            .as_ref()
            .and_then(|result| result.value.as_ref())
        else {
            return Ok(None);
        };
        let value = self.eval(code, expr)?;
        self.roots.activity_mut().push_temp(value);
        if let Some(rendered) = self.result_text(value)? {
            let indent = self.activity.clause_state.current_value_indent;
            self.trace_keyword(indent, "RESULT", &rendered);
        }
        Ok(Some(value))
    }

    /// `RAISE PROPAGATE`: re-raise the condition whose handler is running. A
    /// `RETURN` or `EXIT` value replaces the condition object's `RESULT`
    /// (`RexxActivation::raise`).
    fn exec_raise_propagate(&mut self, code: &Code<'_>, raise: &Raise) -> Result<Flow, Failure> {
        let result = self.raise_result(code, raise)?;
        let Some(active) = self
            .activation()
            .cold
            .as_deref()
            .and_then(|cold| cold.active_condition.clone())
        else {
            return Err(Raised::syntax(98, 918, Vec::new()).into());
        };
        if let Some(result) = result
            && let Some(object) = self.activation().condition().and_then(|c| c.object)
        {
            self.condition_put(object, b"RESULT", result)?;
        }
        // By name: a trapped NOMETHOD carries its 97.1 for the untrapped
        // report, and is still re-raised as a condition.
        let returns = raise.result.as_ref().is_some_and(|result| !result.exit);
        if active.raised.condition != "SYNTAX" {
            return self.propagate_unreportable(&active.raised, returns);
        }
        let mut raised = active.raised.clone();
        raised.delivery.search = Search::Nobody;
        raised.delivery.positionless = true;
        // The original condition's echo stack, put back exactly as it stood
        // when the trap cleared it: the report is the object's `TRACEBACK`.
        // `record_failure_at` is first-wins, so restoring a full
        // `failure_site` is also what stops this `raise propagate` clause
        // recording itself over the clause that actually raised -- measured,
        // the oracle echoes line 8 (`say 1/0`), not line 12 (`raise
        // propagate`) -- and the levels below the trapping one seal nothing.
        let (site, sites) = (active.site.clone(), active.sites.clone());
        self.clear_failure_levels();
        self.activity.failure_site = site;
        self.activity.failure_sites = sites;
        let trapper = self.trapping_depth(&active);
        self.activity.propagate_absorb = trapper + self.fragments_within(trapper);
        // A `REPLY` since the trap left the levels above the trapping one on
        // the sender's activity, and the object's traceback still has them.
        if !active.replied
            && self
                .frame_at(trapper)
                .is_some_and(|frame| frame.reply != ReplyState::None)
            && let Some(object) = self.activation().condition().and_then(|c| c.object)
        {
            self.propagate_after_reply(object)?;
        }
        // `RexxActivation::raise` pops the level and `Activity::
        // reraiseException`s the same object (`execution/RexxActivation.cpp:
        // 1840`). Measured: without a `RETURN` tail, from an internal call it
        // is trapped neither by that call's parent nor by the first
        // activation that is not one, but by the caller of that activation.
        let sender = self.raise_sender(returns);
        if self.frame_at(sender.depth).is_some() {
            raised.delivery.search = sender.search();
            raised.delivery.positionless = false;
            raised.position = 0;
            self.reraise_failure_levels();
            // Re-raised at or below the trapping level, whose clauses the
            // object's traceback does not show, the report names the line
            // of the clause it is re-raised in (`Activity::reraiseException`).
            if sender.depth <= trapper {
                let line = self.clause_of(sender.depth).line;
                let marker = FailureSite::Reraised { line, name: None };
                self.activity.failure_sites.push(marker);
            }
            self.activity.reraised_object = self.activation().condition().and_then(|c| c.object);
            self.activity.reraise_leaving = sender.depth;
        }
        Err(raised.into())
    }

    /// The levels of a propagate whose trapping activation has replied since
    /// its trap: the object's whole `TRACEBACK`, and no level left on this
    /// activity adds one.
    #[cold]
    #[inline(never)]
    fn propagate_after_reply(&mut self, object: ObjRef) -> Result<(), Failure> {
        let lines = self.traceback_lines(object)?;
        let kept = self.activity.failure_sites.len() + 1;
        let above = lines
            .into_iter()
            .skip(kept)
            .map(|text| FailureSite::Rendered {
                text,
                package: None,
            });
        let site = self.activity.failure_site.take();
        self.activity.failure_sites.extend(site);
        self.activity.failure_sites.extend(above);
        self.activity.failure_site = Some(FailureSite::Reraised {
            line: 0,
            name: None,
        });
        self.activity.propagate_absorb = 0;
        Ok(())
    }

    /// The depth of the level whose trap took `active`: the outermost of the
    /// internal calls from the running one outward that share its condition.
    fn trapping_depth(&self, active: &Rc<ActiveCondition>) -> usize {
        let shares = |depth| {
            self.frame_at(depth)
                .and_then(|frame| frame.cold.as_deref())
                .and_then(|cold| cold.active_condition.as_ref())
                .is_some_and(|held| Rc::ptr_eq(held, active))
        };
        let mut trapper = 0;
        while shares(trapper + 1) {
            trapper += 1;
        }
        trapper
    }

    /// `RAISE PROPAGATE` of a condition other than `SYNTAX`. Like a tail-less
    /// `RAISE` it leaves this activation and every internal call beneath it,
    /// and offers the same condition object to the caller of the first
    /// activation that is not one -- none after a `REPLY`, whose sender has
    /// its answer. Untrapped, `HALT` and `NOMETHOD` are reported as errors.
    /// Unless a `SIGNAL ON` trap takes it, the activation returns the
    /// object's `RESULT`, which after a `REPLY` is 98.936
    /// (`RexxActivation::raise`, `execution/RexxActivation.cpp:1801`-`:1805`
    /// and `:1866`-`:1895`).
    fn propagate_unreportable(&mut self, raised: &Raised, returns: bool) -> Result<Flow, Failure> {
        let object = self.activation().condition().and_then(|c| c.object);
        let result = object.and_then(|object| self.condition_entry(object, b"RESULT"));
        if let Some(result) = result {
            self.roots.activity_mut().push_temp(result);
        }
        let sender = self.raise_sender(returns);
        let finish = |result| match returns {
            true => Flow::Return(result),
            false => Flow::Exit(result),
        };
        let caller = sender.activation;
        if caller.is_some() && self.input_dispatcher_takes(caller) {
            return Ok(finish(result));
        }
        let name = raised.condition.as_bytes();
        let trap =
            caller.and_then(|caller| Some((caller, self.trap_at_depth(sender.depth, name)?)));
        match trap {
            Some((_, trap)) if trap.call && sender.reply_issued && result.is_some() => {
                Err(Raised::return_after_reply().into())
            }
            Some((caller, trap)) if trap.call => {
                if let Some(object) = object {
                    self.propagate_to_call_trap(object)?;
                }
                self.activity.pending_traps.push_back(PendingTrap {
                    condition: name.into(),
                    rc: raised.rc.clone(),
                    description: raised.description.clone(),
                    object,
                    activation: caller,
                    queued_during_delivery: false,
                    request: false,
                    fragment_depth: self.sender_fragment_depth(&sender),
                    trace_failure: None,
                });
                Ok(finish(result))
            }
            Some(_) => {
                let mut raised = raised.clone();
                raised.delivery.search = sender.search();
                self.activity.reraised_object = object;
                Err(raised.into())
            }
            None if name == b"HALT" => {
                let mut raised = Raised::halt();
                raised.delivery.search = if returns { Search::Here } else { Search::Top };
                Err(raised.into())
            }
            None if name == b"NOMETHOD" => {
                let receiver =
                    object.and_then(|object| self.condition_entry_held(object, b"ADDITIONAL"));
                let mut raised = match (receiver, raised.description.clone()) {
                    (Some(receiver), Some(description)) => {
                        let receiver = self.string_value_text(receiver)?;
                        Raised::no_method(&receiver, &description)
                    }
                    _ => Raised::syntax(97, 5, Vec::new()),
                };
                raised.delivery.search = if returns { Search::Here } else { Search::Top };
                Err(raised.into())
            }
            None if sender.reply_issued && result.is_some() => {
                Err(Raised::return_after_reply().into())
            }
            None => Ok(finish(result)),
        }
    }

    /// The activation a `RAISE` of a condition other than SYNTAX, or any
    /// `RAISE PROPAGATE`, offers its condition to (`RexxActivation::raise`
    /// and `raiseExit`). With a `RETURN` tail that is the raising level's
    /// sender: an internal call's caller, or, from an `INTERPRET` fragment,
    /// the activation running it -- measured, `interpret "raise user u
    /// return 1"` in a routine runs that routine's own `CALL ON USER U`
    /// handler. Otherwise it is the caller of the first activation that is
    /// not an internal call. After a `REPLY` the sender has its answer and
    /// nothing is offered it.
    fn raise_sender(&self, returns: bool) -> Sender {
        let in_fragment = returns && self.fragments_within(0) > 0;
        let top = if returns {
            0
        } else {
            self.first_non_internal_depth()
        };
        let depth = if in_fragment { 0 } else { top + 1 };
        let reply_issued = self
            .frame_at(top)
            .is_some_and(|activation| activation.reply != ReplyState::None);
        let replied = !in_fragment && reply_issued;
        let activation = self
            .frame_at(depth)
            .map(|activation| activation.id)
            .filter(|_| !replied);
        Sender {
            top,
            depth,
            in_fragment,
            returns,
            reply_issued,
            activation,
        }
    }

    /// Which `INTERPRET` fragment's queue a `CALL ON` condition offered to
    /// `sender` joins: the levels the raise leaves take their fragments
    /// with them.
    fn sender_fragment_depth(&self, sender: &Sender) -> usize {
        let depth = self.activity.fragment_depth;
        match (sender.in_fragment, sender.returns) {
            (true, _) => depth - self.fragments_within(0),
            (false, true) => depth,
            (false, false) => depth - self.fragments_within(sender.top),
        }
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
        self.set_sigl(self.activity.clause_state.line());
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
        self.trace_keyword(
            self.activity.clause_state.current_value_indent,
            "VALUE",
            &text,
        );
        let converted = self.required_string_or_nil(value)?;
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

/// Where [`Interp::raise_sender`] offers a raised condition.
struct Sender {
    /// The level the raise leaves from: the running one, or the first that
    /// is not an internal call.
    top: usize,
    /// The level whose traps are asked.
    depth: usize,
    /// Whether that level is the running one, which an `INTERPRET` fragment
    /// with a `RETURN` tail offers it to.
    in_fragment: bool,
    returns: bool,
    /// Whether `top` has replied, which an `INTERPRET` fragment's sender
    /// inherits: a value it returns is then 98.936 (`returnFrom`) unless a
    /// `SIGNAL ON` trap takes the condition first.
    reply_issued: bool,
    /// The identity of the level at `depth`, unless that level is beyond a
    /// `top` that has replied.
    activation: Option<crate::activation::ActivationId>,
}

impl Sender {
    /// How a `SIGNAL ON` trap at the sender is reached.
    fn search(&self) -> Search {
        match (self.in_fragment, self.top) {
            (true, _) => Search::Here,
            (false, 0) => Search::Caller,
            (false, _) => Search::AboveTop,
        }
    }
}

/// Whether `condition`, raised while its `CALL ON` trap is delayed, is
/// ignored rather than held (rexxref `condtra.xml`, CALL ON and NOTREADY;
/// `iostrms.xml`, the DELAY state).
fn ignored_while_delayed(condition: &[u8]) -> bool {
    matches!(condition, b"ERROR" | b"FAILURE" | b"NOTREADY")
}

/// A `RAISE`'s condition name as `Raised::condition` carries it.
fn condition_name(name: &[u8]) -> Cow<'static, str> {
    Cow::Owned(String::from_utf8_lossy(name).into_owned())
}

/// Whether a `CALL ON ANY` trap takes `condition` (`TrapHandler::canHandle`).
fn call_on_any_takes(condition: &[u8]) -> bool {
    !matches!(
        condition,
        b"SYNTAX" | b"NOVALUE" | b"LOSTDIGITS" | b"NOMETHOD" | b"NOSTRING"
    )
}

/// `RexxActivation::trap` while a line typed at a debug pause runs: every
/// condition but `SYNTAX` goes untrapped (`RexxActivation.cpp:2475`, `:2600`).
fn ignored_in_debug_pause(frame: &crate::activation::Activation, condition: &[u8]) -> bool {
    frame.flags.debug_pause() && condition != b"SYNTAX"
}

/// Whether `frame` takes `NOTREADY` by the very trap `caller` does, the one
/// it inherited on entry: each `CALL ON` or `SIGNAL ON` makes a new label.
fn same_notready_trap(
    frame: &crate::activation::Activation,
    caller: &crate::activation::Activation,
) -> bool {
    fn in_force(activation: &crate::activation::Activation) -> Option<&std::rc::Rc<[u8]>> {
        let traps = &activation.traps;
        traps
            .get(b"NOTREADY".as_slice())
            .or_else(|| traps.get(b"ANY".as_slice()))
            .map(|trap| &trap.label)
    }
    match (in_force(frame), in_force(caller)) {
        (Some(own), Some(inherited)) => std::rc::Rc::ptr_eq(own, inherited),
        _ => false,
    }
}
