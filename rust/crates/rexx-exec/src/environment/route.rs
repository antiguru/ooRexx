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

//! Output routing: where `SAY` sends and where a finished trace line is
//! delivered, through `.local`'s `OUTPUT` and `TRACEOUTPUT` entries.

use super::{Body, EnvScope, Failure, Interp, ObjRef, env_seam, hash};

/// Where a `TraceObject` went.
enum Delivered {
    /// To the `.TRACEOUTPUT` destination, by `LINEOUT`.
    Sent,
    /// Nowhere, under `.TraceObject~option` `P`.
    KeptBack,
    /// To standard error, as the line.
    ToStandardError,
    /// To standard error, as the object's string.
    ToStandardErrorAs(Vec<u8>),
}

impl Interp {
    /// What `.local` holds for one of the route names, or `None` when it holds
    /// nothing.
    ///
    /// **Not [`Interp::dot_variable`]**, which is what a `.NAME` in a program
    /// resolves through: on a miss that answers the name *as text*, and `SAY`
    /// sent to the string `".OUTPUT"` would be a wrong answer. The routes need
    /// the plain question -- is there an entry -- which is what
    /// `directory_lookup` answers, and it is reused rather than reopened so the
    /// seam still has one read chokepoint. An owed entry answers `None`.
    ///
    /// `.local` only: measured, `RexxActivation::resolveStream` and
    /// `Activity::sayOutput` both read `getLocalEnvironment` and never
    /// `.environment`.
    pub(crate) fn local_route(&mut self, name: &[u8]) -> Result<Option<ObjRef>, Failure> {
        match self.directory_lookup(&[EnvScope::Local], name, &env_seam::Access::Direct)? {
            hash::DirectoryEntry::Found(found) => Ok(Some(found)),
            hash::DirectoryEntry::Owed(_) | hash::DirectoryEntry::Absent => Ok(None),
        }
    }

    /// Records that a route's far end may have moved, so the next `SAY`
    /// decides afresh.
    pub(crate) fn bump_route_generation(&mut self) {
        self.route_generation = self.route_generation.wrapping_add(1);
    }

    /// Where `SAY` sends, or `None` to write straight to the buffer.
    ///
    /// **The decision is cached, not either half of it**: deciding afresh is a
    /// probe of `.local` and a walk of the monitors' destination queues, 764
    /// instructions a line between them, and caching one half would leave most
    /// of that standing.
    pub(crate) fn output_route(&mut self) -> Result<Option<ObjRef>, Failure> {
        if let Some((generation, cached)) = self.output_route
            && generation == self.route_generation
        {
            // A writer this crate forgot to count would otherwise be a wrong
            // answer no corpus program could see, because none of them
            // redirect `.OUTPUT`.
            #[cfg(debug_assertions)]
            {
                let fresh = unshared!(self, self.output_route_uncached())?;
                assert_eq!(
                    cached, fresh,
                    "the cached SAY route is stale, so some writer does not bump \
                     Interp::route_generation"
                );
            }
            return Ok(cached);
        }
        let fresh = self.output_route_uncached()?;
        self.output_route = Some((self.route_generation, fresh));
        Ok(fresh)
    }

    /// [`Interp::output_route`] with no cache in front of it.
    fn output_route_uncached(&mut self) -> Result<Option<ObjRef>, Failure> {
        match self.local_route(b"OUTPUT")? {
            Some(route)
                if route != ObjRef::NIL && !self.route_ends_at(route, self.bootstrap_stdout) =>
            {
                Ok(Some(route))
            }
            _ => Ok(None),
        }
    }

    /// Whether `route` still ends at `terminal`, following each monitor's
    /// destination queue head (`CoreClasses.orx`'s `Monitor`, whose `UNKNOWN`
    /// forwards to `destination~peek`).
    ///
    /// A route that ends at the stream the bundle minted writes the same bytes
    /// whether it is delivered or written straight to the buffer, and the
    /// direct write is the one that runs no Rexx. That matters twice over.
    /// Delivering re-enters the interpreter mid-clause, which perturbs the
    /// argument stack, `PROCEDURE` permission and a `SELECT`'s own search --
    /// measured, all three. And it is not cheap: measured, delivering every
    /// `SAY` costs 16,360 instructions a line, which is `sayloop` at 9.63x.
    pub(crate) fn route_ends_at(&mut self, route: ObjRef, terminal: Option<ObjRef>) -> bool {
        let Some(terminal) = terminal else {
            return false;
        };
        let Some(monitor_class) = self.rexx_package_class(b"MONITOR") else {
            return false;
        };
        let Some(array_class) = self.classes().lookup("Array") else {
            return false;
        };
        let mut at = route;
        // The chain the mint builds is `TRACEOUTPUT` over `ERROR` over
        // `STDERR`, and a program may leave it shorter or longer; the bound
        // is what keeps a cycle from spinning here.
        for _ in 0..8 {
            if at == terminal {
                return true;
            }
            let Some(queue) = self.pool_entry(at, monitor_class, b"DESTINATION") else {
                return false;
            };
            let store = match self.array_slots(queue) {
                Some(_) => queue,
                None => match self.pool_entry(queue, array_class, b"ITEMS") {
                    Some(store) => store,
                    None => return false,
                },
            };
            let Some(Some(head)) = self.array_slots(store).and_then(<[_]>::first).copied() else {
                return false;
            };
            at = head;
        }
        false
    }

    /// What `name` holds in `owner`'s pool for `scope`, or `None`.
    fn pool_entry(&self, owner: ObjRef, scope: ObjRef, name: &[u8]) -> Option<ObjRef> {
        match self.heap.get(owner).map(|object| &object.body) {
            Some(Body::Instance { pools, .. }) => pools.get(scope, name),
            _ => None,
        }
    }

    /// One finished trace line, delivered to `.TRACEOUTPUT` as a `TraceObject`
    /// -- `Activity::traceOutput` (`concurrency/Activity.cpp:3171`).
    ///
    /// `start` is where the line begins in the buffer: a `.STDERR` `CHAROUT`
    /// leaves an unterminated fragment there, so the line cannot be found by
    /// scanning back for a newline. A route that cannot take the line leaves
    /// it on stdout, which is `corpus/oracle-crashes.txt` entry 11's licensed
    /// answer for a route the oracle dies on.
    ///
    /// Code that shows no source traces nothing through here
    /// ([`Interp::untraceable_code`]).
    // Inlined into `loop_advance`, it cost sayloop +0.31% against +0.14%.
    #[inline(never)]
    pub(crate) fn route_trace_line(&mut self, start: usize) {
        if self.untraceable_code() {
            self.trace.truncate(start);
            return;
        }
        self.route_trace_line_as(start, false);
    }

    /// [`Interp::route_trace_line`] for a `>I>` (`entry`) or `<I<` line,
    /// whose `TraceObject` names the caller's frame for `>I>`. Standard error
    /// is handed over after it. Not gated on the source: the oracle's
    /// `traceEntryOrExit` (`RexxActivation.cpp:3678`) writes these for a
    /// restored program too (measured).
    pub(crate) fn route_trace_line_as(&mut self, start: usize, entry: bool) {
        self.deliver_trace_line_as(start, entry);
        self.hand_over_stderr();
    }

    /// `RexxActivation::processTraceInfo`: every line becomes a `TraceObject`,
    /// which `Activity::traceOutput` sends to `.TRACEOUTPUT`, writes to
    /// standard error where that is the error stream, or keeps back under
    /// `.TraceObject~option` `P`. Without the class the line goes to
    /// standard error as it is.
    fn deliver_trace_line_as(&mut self, start: usize, entry: bool) {
        let variable = self.traced_variable.take();
        if self.activity.routing_trace {
            return;
        }
        if self.trace_failure_queued() {
            self.trace.truncate(start);
            return;
        }
        let Some(class) = self.rexx_package_class(b"TRACEOBJECT") else {
            return;
        };
        // Only the line end the formatter appended: measured, a traced clause
        // ending in `'0a'x` is delivered with it.
        let mut line = self.trace.split_off(start);
        let ended = line.last() == Some(&b'\n');
        if ended {
            line.pop();
        }
        self.activity.routing_trace = true;
        self.deliveries.push(crate::activity::Delivery::default());
        // **The nested-send protocol, not a bare send**, and around the whole
        // delivery because `NEW`, each `PUT` and the `LINEOUT` all run Rexx
        // clauses. A clause boundary clears the shared argument stack, so
        // delivering on the caller's own stack wipes the arguments an
        // enclosing call has already pushed: measured, `zz = length('abcd')`
        // under `trace i` refuses with `call_op_off_its_node`, where a call
        // with no arguments is untouched.
        // The permission a traced clause holds for `PROCEDURE` is the
        // activity's, and the clauses the delivery runs spend it: measured,
        // `p: procedure` under `trace i` with a Rexx destination was 17.1.
        let permitted = (
            self.activity.procedure_permitted,
            self.activity.region_procedure_permitted,
        );
        let (values, mark) = self.take_value_buffer();
        let delivered = pinned!(
            self,
            crate::pinning::PinKind::TraceWrapper,
            self.deliver_trace_line(class, &line, entry, variable)
        );
        self.give_value_buffer(values, mark);
        (
            self.activity.procedure_permitted,
            self.activity.region_procedure_permitted,
        ) = permitted;
        self.activity.routing_trace = false;
        self.deliveries.pop();
        match delivered {
            Ok(Delivered::Sent | Delivered::KeptBack) => {}
            Ok(Delivered::ToStandardError) => {
                self.trace.extend_from_slice(&line);
                if ended {
                    self.trace.push(b'\n');
                }
            }
            Ok(Delivered::ToStandardErrorAs(shown)) => {
                self.trace.extend_from_slice(&shown);
                if ended {
                    self.trace.push(b'\n');
                }
            }
            // A failure before the line reached a destination of the
            // program's own, such as making the `TraceObject` with the stack
            // exhausted, leaves the line to the error stream.
            Err(_) if self.routes_to_standard_error() => {
                self.trace.extend_from_slice(&line);
                if ended {
                    self.trace.push(b'\n');
                }
            }
            Err(failure) => self.queue_trace_failure(failure, &line),
        }
    }

    /// Whether `.TRACEOUTPUT` writes to standard error, or is not there.
    fn routes_to_standard_error(&mut self) -> bool {
        match self.local_route(b"TRACEOUTPUT") {
            Ok(Some(route)) if route != ObjRef::NIL => {
                self.route_ends_at(route, self.bootstrap_stderr)
            }
            _ => true,
        }
    }

    /// Whether a trace line's failed delivery is waiting to be raised in the
    /// running activation, which traces nothing more until it is.
    pub(crate) fn trace_failure_queued(&self) -> bool {
        let Some(running) = self.running_activation() else {
            return false;
        };
        let here = running.id;
        self.activity
            .pending_traps
            .iter()
            .any(|pending| pending.trace_failure.is_some() && pending.activation == here)
    }

    /// The failure [`Interp::queue_trace_failure`] queued for the running
    /// activation, taken off the queue.
    #[cold]
    #[inline(never)]
    pub(crate) fn take_trace_failure(&mut self) -> Option<Failure> {
        let here = self.running_activation()?.id;
        let at =
            self.activity.pending_traps.iter().position(|pending| {
                pending.trace_failure.is_some() && pending.activation == here
            })?;
        self.activity
            .pending_traps
            .remove(at)
            .and_then(|pending| pending.trace_failure)
    }

    /// `Activity::traceOutput`'s failed `LINEOUT`: raised in the traced
    /// activation at its clause's end. With no activation running, the line
    /// is left on stdout, as `corpus/oracle-crashes.txt` entry 11 licenses.
    /// A refusal is never dropped: an activation that ends before a clause
    /// end raises it passes it to its caller ([`Interp::carry_trace_refusals`]).
    #[cold]
    #[inline(never)]
    fn queue_trace_failure(&mut self, failure: Failure, line: &[u8]) {
        let refusal = !matches!(failure, Failure::Raised(_));
        let Some(running) = self.running_activation() else {
            self.write_out_line(line);
            if refusal {
                self.trace_refusal.get_or_insert(failure);
            }
            return;
        };
        let activation = running.id;
        if refusal {
            // So that its end runs `Interp::end_cold`, which carries the
            // refusal on.
            self.activation_mut().cold_mut();
        }
        self.activity.pending_traps.push_back(crate::PendingTrap {
            condition: b"SYNTAX".as_slice().into(),
            rc: None,
            description: None,
            object: None,
            activation,
            queued_during_delivery: true,
            request: false,
            fragment_depth: self.activity.fragment_depth,
            trace_failure: Some(failure),
        });
    }

    /// Moves each refusal [`Interp::queue_trace_failure`] queued for the
    /// activation now ending onto its caller, which raises it at its own
    /// clause's end, or with no caller onto the activity, which ends with it
    /// ([`Interp::trace_refusal`]).
    #[cold]
    #[inline(never)]
    pub(crate) fn carry_trace_refusals(&mut self) {
        let Some(ending) = self.running_activation().map(|activation| activation.id) else {
            return;
        };
        let caller = self
            .activity
            .suspended
            .last()
            .map(|activation| activation.id);
        let depth = self.activity.fragment_depth;
        let mut index = 0;
        while index < self.activity.pending_traps.len() {
            let pending = &mut self.activity.pending_traps[index];
            let refused = pending.activation == ending
                && pending
                    .trace_failure
                    .as_ref()
                    .is_some_and(|failure| !matches!(failure, Failure::Raised(_)));
            if !refused {
                index += 1;
                continue;
            }
            if let Some(caller) = caller {
                pending.activation = caller;
                pending.fragment_depth = depth;
                index += 1;
                continue;
            }
            let failure = self
                .activity
                .pending_traps
                .remove(index)
                .and_then(|pending| pending.trace_failure);
            if let Some(failure) = failure {
                self.trace_refusal.get_or_insert(failure);
            }
        }
    }

    /// Each line of an error report, delivered the way a trace line is:
    /// measured, the oracle sends one `LINEOUT` per line -- three for an
    /// uncaught `1/0` -- and not one for the report.
    pub(crate) fn write_trace_report(&mut self, report: &[u8]) {
        let body = report.strip_suffix(b"\n").unwrap_or(report);
        for line in body.split(|&byte| byte == b'\n') {
            self.write_trace_report_line(line);
        }
    }

    /// [`Interp::write_trace_report`] over the report's own lines, each
    /// delivered whole, with any line end it holds.
    pub(crate) fn write_trace_report_lines(&mut self, lines: &[Vec<u8>]) {
        for line in lines {
            self.write_trace_report_line(line);
        }
    }

    fn write_trace_report_line(&mut self, line: &[u8]) {
        let start = self.trace.len();
        self.trace.extend_from_slice(line);
        self.trace.push(b'\n');
        self.route_trace_line(start);
    }

    /// The `TraceObject` carrying `line` and where it was traced
    /// (`RexxActivation::createTraceObject`,
    /// `execution/RexxActivation.cpp:5160`), then its delivery.
    fn deliver_trace_line(
        &mut self,
        class: ObjRef,
        line: &[u8],
        entry: bool,
        variable: Option<crate::trace::TracedVariable>,
    ) -> Result<Delivered, Failure> {
        let waiting = self.activity.waiting_traced.take();
        let thread = self.activity_number();
        let invocation = self.invocation_of(0).unwrap_or(0);
        let levels = crate::dispatch::context::live_levels(self);
        let frame = match levels.first() {
            Some(&level) => Some(self.frame_record(level)?),
            None => None,
        };
        let native = match (entry, levels.first()) {
            (true, Some(crate::dispatch::context::LiveLevel::Activation(0))) => {
                self.entering_native_frame()
            }
            _ => None,
        };
        let caller = match (entry, native, levels.get(1)) {
            (false, ..) => None,
            (true, Some(frame), _) => Some(CallerFrame::Of(frame)),
            (true, None, Some(&level)) => Some(CallerFrame::Of(self.frame_record(level)?)),
            (true, None, None) => Some(CallerFrame::Spawner),
        };
        let method = match (levels.first(), self.frame_at(0)) {
            (Some(crate::dispatch::context::LiveLevel::Activation(0)), Some(activation))
                if activation.entry == crate::activation::Entry::Method =>
            {
                activation.method_identity.as_ref().map(|identity| {
                    (
                        crate::guards::GuardKey {
                            object: identity.receiver,
                            scope: identity.scope,
                        },
                        activation.flags.guarded(),
                        activation.flags.reserved(),
                    )
                })
            }
            _ => None,
        };

        // **A call tail of the delivery's own**, so that a primitive method
        // the delivery sends parks its tail above every tail the traced code
        // parked: a `<I<` is delivered after its call tail is popped, at the
        // depth where the sender's primitive `NEW` waits.
        self.activity.call_tails.push(crate::run::CallTail::method(
            crate::CallContext::default(),
            self.save_clause_state(),
            0,
            0,
            None,
        ));
        let delivered = (|| -> Result<Delivered, Failure> {
            let caller_ref = self.caller();
            let Some(object) = self.send_message(class, b"NEW", None, &[], caller_ref)? else {
                return Err(Failure::Raised(Box::new(crate::error::Raised::syntax(
                    97,
                    1,
                    vec![b"TraceObject".to_vec()],
                ))));
            };
            self.roots.activity_mut().push_temp(object);
            let traced = self.text(line);
            self.trace_object_put(object, b"TRACELINE", traced)?;
            let interpreter = self.counted(1);
            self.trace_object_put(object, b"INTERPRETER", interpreter)?;
            let thread = self.counted(thread as usize);
            self.trace_object_put(object, b"THREAD", thread)?;
            let invocation = self.counted(invocation as usize);
            self.trace_object_put(object, b"INVOCATION", invocation)?;
            let stack_frame = match frame {
                Some(frame) => self.frame_table(Some(frame))?,
                None => ObjRef::NIL,
            };
            self.trace_object_put(object, b"STACKFRAME", stack_frame)?;
            if let Some(caller) = caller {
                let table = match caller {
                    CallerFrame::Of(frame) => self.frame_table(Some(frame))?,
                    CallerFrame::Spawner => self.spawner_table()?,
                };
                self.trace_object_put(object, b"CALLERSTACKFRAME", table)?;
            }
            if let Some((key, guarded, reserved)) = method {
                let pool = self.activities.guards.pool_number(key);
                let pool = self.counted(pool as usize);
                self.trace_object_put(object, b"ATTRIBUTEPOOL", pool)?;
                self.trace_object_put(object, b"ISGUARDED", crate::eval::logical(guarded))?;
                let count = self.guard_count(key);
                let count = self.counted(count as usize);
                self.trace_object_put(object, b"SCOPELOCKCOUNT", count)?;
                self.trace_object_put(object, b"HASSCOPELOCK", crate::eval::logical(reserved))?;
                self.trace_object_put(object, b"RECEIVER", key.object)?;
                if let Some(waiting) = waiting {
                    self.trace_object_put(object, b"ISWAITING", crate::eval::logical(waiting))?;
                }
            }
            if let Some(variable) = variable {
                self.trace_object_variable(object, variable)?;
            }
            let caller_ref = self.caller();
            let notify = self
                .send_message(class, b"NOTIFY", None, &[], caller_ref)?
                .unwrap_or(ObjRef::NIL);
            if notify != ObjRef::NIL {
                let caller_ref = self.caller();
                self.send_message(notify, b"APPEND", None, &[Some(object)], caller_ref)?;
            }
            // `Activity::traceOutput`: the option the object was made under.
            let index = self.text(b"OPTION");
            self.roots.activity_mut().push_temp(index);
            let caller_ref = self.caller();
            let option = self
                .send_message(object, b"[]", None, &[Some(index)], caller_ref)?
                .unwrap_or(ObjRef::NIL);
            let letter = if option == ObjRef::NIL {
                b'N'
            } else {
                self.to_text(option).first().copied().unwrap_or(b'N')
            };
            if letter == b'P' {
                return Ok(Delivered::KeptBack);
            }
            if self.routes_to_standard_error() {
                // The error stream writes the object's string, which is the line
                // itself under option `N` and `makeStringImpl`'s format otherwise.
                if letter == b'N' {
                    return Ok(Delivered::ToStandardError);
                }
                let caller_ref = self.caller();
                let shown = self
                    .send_message(object, b"MAKESTRING", None, &[], caller_ref)?
                    .unwrap_or(ObjRef::NIL);
                let shown = self.to_text(shown).to_vec();
                return Ok(Delivered::ToStandardErrorAs(shown));
            }
            let Some(route) = self.local_route(b"TRACEOUTPUT")? else {
                return Ok(Delivered::ToStandardError);
            };
            let floor =
                Some(self.activity.suspended.len() + usize::from(self.activity.running.is_some()));
            if let Some(delivery) = self.deliveries.last_mut() {
                delivery.floor = floor;
            }
            let caller_ref = self.caller();
            self.send_message(
                route,
                crate::dispatch::LINEOUT,
                None,
                &[Some(object)],
                caller_ref,
            )?;
            Ok(Delivered::Sent)
        })();
        self.activity.call_tails.pop();
        delivered
    }

    /// The `VARIABLE` entry of a `>V>` or `>=>` line's `TraceObject`: a
    /// `StringTable` of `NAME`, `VALUE` and `ASSIGNMENT`.
    fn trace_object_variable(
        &mut self,
        object: ObjRef,
        variable: crate::trace::TracedVariable,
    ) -> Result<(), Failure> {
        let table = crate::dispatch::hash::new_string_table(self)?;
        self.roots.activity_mut().push_temp(table);
        self.trace_object_put(object, b"VARIABLE", table)?;
        let name = match &variable.name {
            Some(name) => self.text(name),
            None => ObjRef::NIL,
        };
        self.roots.activity_mut().push_temp(name);
        crate::dispatch::hash::directory_put(self, table, b"NAME", name)?;
        let value = variable.value.unwrap_or(ObjRef::NIL);
        crate::dispatch::hash::directory_put(self, table, b"VALUE", value)?;
        crate::dispatch::hash::directory_put(
            self,
            table,
            b"ASSIGNMENT",
            crate::eval::logical(variable.assignment),
        )
    }

    /// `PUT` of `value` under `name` on the `TraceObject` being built. A
    /// `StringTable` keeps its entries in a bucket table, so the put goes
    /// through the message; `t[i] = v` sends `t~"[]="(v, i)`, so the value
    /// leads.
    fn trace_object_put(
        &mut self,
        object: ObjRef,
        name: &[u8],
        value: ObjRef,
    ) -> Result<(), Failure> {
        self.roots.activity_mut().push_temp(value);
        let index = self.text(name);
        self.roots.activity_mut().push_temp(index);
        let caller = self.caller();
        self.send_message(object, b"PUT", None, &[Some(value), Some(index)], caller)?;
        Ok(())
    }

    /// The `StackFrame` of `level` and its executable (`.nil` where this
    /// crate has no object for it), rooted as temporaries.
    pub(crate) fn frame_record(
        &mut self,
        level: crate::dispatch::context::LiveLevel,
    ) -> Result<(ObjRef, ObjRef), Failure> {
        let frame = crate::dispatch::context::build_live_frame(self, level)?;
        self.roots.activity_mut().push_temp(frame);
        let executable = match level {
            crate::dispatch::context::LiveLevel::Activation(depth) => {
                crate::dispatch::context::executable_at(self, depth).unwrap_or(ObjRef::NIL)
            }
            _ => ObjRef::NIL,
        };
        self.roots.activity_mut().push_temp(executable);
        Ok((frame, executable))
    }

    /// `RexxActivation::getStackFrameAsStringTable`
    /// (`execution/RexxActivation.cpp:5289`): a `StringTable` of `frame`'s
    /// entries, empty for none.
    fn frame_table(&mut self, frame: Option<(ObjRef, ObjRef)>) -> Result<ObjRef, Failure> {
        let table = crate::dispatch::hash::new_string_table(self)?;
        self.roots.activity_mut().push_temp(table);
        let Some((frame, executable)) = frame else {
            return Ok(table);
        };
        for name in [
            b"ARGUMENTS".as_slice(),
            b"INVOCATION",
            b"LINE",
            b"NAME",
            b"TRACELINE",
            b"TYPE",
        ] {
            self.frame_entry_into(table, frame, name)?;
        }
        crate::dispatch::hash::directory_put(self, table, b"EXECUTABLE", executable)?;
        self.frame_entry_into(table, frame, b"TARGET")?;
        Ok(table)
    }

    /// What `frame` answers to `name`, put in `table` under `name`.
    fn frame_entry_into(
        &mut self,
        table: ObjRef,
        frame: ObjRef,
        name: &[u8],
    ) -> Result<(), Failure> {
        let caller = self.caller();
        let value = self
            .send_message(frame, name, None, &[], caller)?
            .unwrap_or(ObjRef::NIL);
        self.roots.activity_mut().push_temp(value);
        crate::dispatch::hash::directory_put(self, table, name, value)
    }

    /// The table of the frame that started this activity, with the
    /// spawner's number as its `THREAD` (`Activity::setCallerStackFrameAsStringTable`,
    /// `concurrency/Activity.cpp:1199`), built once; `.nil` for main.
    fn spawner_table(&mut self) -> Result<ObjRef, Failure> {
        match self.activity.spawner {
            None => Ok(ObjRef::NIL),
            Some(crate::activity::Spawner::Table(table)) => Ok(table),
            Some(crate::activity::Spawner::Frame { thread, frame }) => {
                let table = self.frame_table(frame)?;
                let thread = self.counted(thread as usize);
                self.roots.activity_mut().push_temp(thread);
                crate::dispatch::hash::directory_put(self, table, b"THREAD", thread)?;
                self.activity.spawner = Some(crate::activity::Spawner::Table(table));
                Ok(table)
            }
        }
    }

    /// What a spawned activity records of the running one for its
    /// `CALLERSTACKFRAME`: its number and the frame of the innermost level.
    pub(crate) fn spawner_of_running(&mut self) -> Result<crate::activity::Spawner, Failure> {
        let thread = self.activity_number();
        let frame = match crate::dispatch::context::live_levels(self).first() {
            Some(&level) => Some(self.frame_record(level)?),
            None => None,
        };
        Ok(crate::activity::Spawner::Frame { thread, frame })
    }
}

/// Where a `>I>` line's `CALLERSTACKFRAME` comes from.
enum CallerFrame {
    /// The level below the traced one.
    Of((ObjRef, ObjRef)),
    /// None below it: the frame that started this activity.
    Spawner,
}
