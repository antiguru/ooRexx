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
                let fresh = self.output_route_uncached()?;
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
    pub(crate) fn route_trace_line(&mut self, start: usize) {
        self.route_trace_line_as(start, false);
    }

    /// [`Interp::route_trace_line`] for a line that is `>I>` (`entry`), whose
    /// `TraceObject` names the caller's frame too.
    pub(crate) fn route_trace_line_as(&mut self, start: usize, entry: bool) {
        if self.activity.routing_trace {
            return;
        }
        let Ok(Some(route)) = self.local_route(b"TRACEOUTPUT") else {
            return;
        };
        if route == ObjRef::NIL || self.route_ends_at(route, self.bootstrap_stderr) {
            return;
        }
        let Some(class) = self.rexx_package_class(b"TRACEOBJECT") else {
            return;
        };
        let mut line = self.trace.split_off(start);
        while line.last() == Some(&b'\n') {
            line.pop();
        }
        self.activity.routing_trace = true;
        // **The nested-send protocol, not a bare send**, and around the whole
        // delivery because `NEW`, each `PUT` and the `LINEOUT` all run Rexx
        // clauses. A clause boundary clears the shared argument stack, so
        // delivering on the caller's own stack wipes the arguments an
        // enclosing call has already pushed: measured, `zz = length('abcd')`
        // under `trace i` refuses with `call_op_off_its_node`, where a call
        // with no arguments is untouched.
        let (values, mark) = self.take_value_buffer();
        let delivered = pinned!(
            self,
            crate::pinning::PinKind::TraceWrapper,
            self.deliver_trace_line(class, route, &line, entry)
        );
        self.give_value_buffer(values, mark);
        self.activity.routing_trace = false;
        if delivered.is_err() {
            self.out.extend_from_slice(&line);
            self.out.push(b'\n');
        }
    }

    /// Each line of an error report, delivered the way a trace line is:
    /// measured, the oracle sends one `LINEOUT` per line -- three for an
    /// uncaught `1/0` -- and not one for the report.
    pub(crate) fn write_trace_report(&mut self, report: &[u8]) {
        let body = report.strip_suffix(b"\n").unwrap_or(report);
        for line in body.split(|&byte| byte == b'\n') {
            let start = self.trace.len();
            self.trace.extend_from_slice(line);
            self.trace.push(b'\n');
            self.route_trace_line(start);
        }
    }

    /// The send itself: a `TraceObject` carrying `line` and where it was
    /// traced, then `LINEOUT` to `route` (`RexxActivation::createTraceObject`,
    /// `execution/RexxActivation.cpp:5160`). `ATTRIBUTEPOOL` and the guard
    /// entries read zero: they come from the guard table (plan Task 11).
    fn deliver_trace_line(
        &mut self,
        class: ObjRef,
        route: ObjRef,
        line: &[u8],
        entry: bool,
    ) -> Result<(), Failure> {
        let thread = self.activity_number();
        let invocation = self.invocation_of(0).unwrap_or(0);
        let levels = crate::dispatch::context::live_levels(self);
        let frame = match levels.first() {
            Some(&level) => Some(self.frame_record(level)?),
            None => None,
        };
        let caller = match (entry, levels.get(1)) {
            (false, _) => None,
            (true, Some(&level)) => Some(CallerFrame::Of(self.frame_record(level)?)),
            (true, None) => Some(CallerFrame::Spawner),
        };
        let receiver = match (levels.first(), self.frame_at(0)) {
            (Some(crate::dispatch::context::LiveLevel::Activation(0)), Some(activation))
                if activation.entry == crate::activation::Entry::Method =>
            {
                activation
                    .method_identity
                    .as_ref()
                    .map(|identity| identity.receiver)
            }
            _ => None,
        };

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
        if let Some(receiver) = receiver {
            let zero = self.counted(0);
            self.trace_object_put(object, b"ATTRIBUTEPOOL", zero)?;
            self.trace_object_put(object, b"ISGUARDED", crate::eval::logical(false))?;
            self.trace_object_put(object, b"SCOPELOCKCOUNT", zero)?;
            self.trace_object_put(object, b"HASSCOPELOCK", crate::eval::logical(false))?;
            self.trace_object_put(object, b"RECEIVER", receiver)?;
        }
        let caller_ref = self.caller();
        self.send_message(
            route,
            crate::dispatch::LINEOUT,
            None,
            &[Some(object)],
            caller_ref,
        )?;
        Ok(())
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
