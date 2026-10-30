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

use super::*;

impl Interp {
    /// `RAISE`, in all of its forms.
    /// ```text
    /// RAISE SYNTAX n.m RETURN [e]   search from the raising activation outward
    /// RAISE SYNTAX n.m             \  the nearest activation not an internal
    /// RAISE SYNTAX n.m EXIT [e]    /  call, then outward; internal calls skip
    /// RAISE other ... RETURN [e]      search from the raising activation's CALLER
    /// RAISE other ...              \  the nearest activation not an internal
    /// RAISE other ... EXIT [e]     /  call, then its CALLER, as RETURN does
    /// ```
    pub(in crate::run) fn exec_raise(&mut self, code: &Code<'_>, raise: &Raise) -> Result<Flow, Failure> {
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
                if self.traced_mode().results {
                    let traced = self.traced_string_value_text(value)?;
                    let keyword = String::from_utf8_lossy(&raise.condition).into_owned();
                    self.trace_keyword(indent, &keyword, &traced);
                }
                // A SYNTAX code is `rc->requestString()` (`:193`).
                let code = match raise.condition.as_ref() == b"SYNTAX" {
                    true => self.required_string_value(value)?,
                    false => value,
                };
                Some(self.to_text(code).to_vec())
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
            if self.traced_mode().results {
                let traced = self.traced_string_value_text(value)?;
                self.trace_keyword(indent, "DESCRIPTION", &traced);
            }
            let value = self.required_string_value(value)?;
            self.roots.activity_mut().push_temp(value);
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
            if self.traced_mode().results {
                let traced = self.traced_string_value_text(value)?;
                self.trace_keyword(indent, "ADDITIONAL", &traced);
            }
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
                self.trace_argument(indent, &rendered);
                self.trace_argument(indent, &rendered);
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

}
