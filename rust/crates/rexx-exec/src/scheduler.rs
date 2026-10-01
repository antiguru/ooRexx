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

//! The scheduler seam (spec 2026-09-29 section 2.1) and the activity table:
//! the running activity sits inline on `Interp`, every other one boxed here,
//! and a switch swaps the two where a driver has exited or, for a pinned
//! wait, inside the Rust frames that pin the outgoing activity.

use std::collections::VecDeque;

use rexx_core::{ActivityRoots, ObjRef};
use rustc_hash::FxHashMap;

use crate::activity::Activity;
use crate::dispatch::{Caller, Then};
use crate::error::Failure;
use crate::run::{Ended, Flow, Started};
use crate::{Interp, Loud};

/// What an instruction run by [`crate::ir::Op::Exec`] answers the driver.
pub(crate) enum ExecOutcome {
    /// It ran, and this is its `Flow`.
    Done(Flow),
    /// The instruction would park the activity, which the driver refuses
    /// loudly.
    #[expect(
        dead_code,
        reason = "constructed only by the test-only arm, and no arm reads the reason"
    )]
    Park(ParkReason),
    /// Its continuation goes to a new activity, and the current one goes on
    /// with this `Flow`.
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "constructed only by the test-only arm")
    )]
    Split(Flow),
}

/// An activity's handle: its offset in the interpreter's activity table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ActivityId(u32);

/// A message object's identity among the activities waiting on it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct MessageId(u32);

/// Why an activity parks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ParkReason {
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "constructed only by the test-only arm")
    )]
    Guard,
    /// `~result`, until the message's send completes.
    MessageResult(MessageId),
    /// `~wait`, until the message's send completes.
    MessageWait(MessageId),
}

impl ParkReason {
    /// What a wait for this reason waits for, as a refusal names it.
    fn waits_for(self) -> &'static str {
        match self {
            ParkReason::Guard => "a guard",
            ParkReason::MessageResult(_) | ParkReason::MessageWait(_) => "a message's completion",
        }
    }
}

/// A started activity's first step: the send `~start` asked for.
pub(crate) struct StartedSend {
    pub(crate) receiver: ObjRef,
    pub(crate) name: Vec<u8>,
    pub(crate) scope: Option<ObjRef>,
    pub(crate) args: Vec<Option<ObjRef>>,
    pub(crate) caller: Caller,
}

impl StartedSend {
    pub(crate) fn object_roots(&self, out: &mut Vec<ObjRef>) {
        out.push(self.receiver);
        out.extend(self.scope);
        out.extend(self.args.iter().flatten().copied());
        out.extend(self.caller.receiver());
    }
}

/// An activity that is not running.
struct Idle {
    activity: Activity,
    roots: ActivityRoots,
}

/// Every activity but the running one, and what the scheduler knows of them.
pub(crate) struct Activities {
    running: ActivityId,
    /// Indexed by [`ActivityId`]; `None` for the running activity and for a
    /// free handle.
    idle: Vec<Option<Box<Idle>>>,
    free: Vec<u32>,
    /// Woken and spawned activities, in arrival order.
    ready: VecDeque<ActivityId>,
    /// The activities whose loops of [`Interp::run_others`] are on the Rust
    /// stack, outermost first.
    owners: Vec<ActivityId>,
    /// Ready owners a loop could not run, each with the index in `owners` of
    /// the loop that set it aside; it goes back to the ready queue when that
    /// loop returns.
    set_aside: Vec<(ActivityId, usize)>,
    message_ids: FxHashMap<ObjRef, MessageId>,
    next_message: u32,
    /// The parked activities waiting on each message, in the order they
    /// parked.
    waiters: FxHashMap<MessageId, Vec<ActivityId>>,
    /// The thread contexts of ended activities, which an extension may have
    /// kept and which last until interpreter termination.
    retired: Vec<rexx_api::ffi::ThreadContext>,
}

impl Activities {
    pub(crate) fn new() -> Activities {
        Activities {
            running: ActivityId(0),
            idle: vec![None],
            free: Vec::new(),
            ready: VecDeque::new(),
            owners: Vec::new(),
            set_aside: Vec::new(),
            message_ids: FxHashMap::default(),
            next_message: 0,
            waiters: FxHashMap::default(),
            retired: Vec::new(),
        }
    }

    /// Appends every `ObjRef` an activity that is not running holds.
    pub(crate) fn object_roots(&self, out: &mut Vec<ObjRef>) {
        for idle in self.idle.iter().flatten() {
            idle.activity.object_roots(out);
            out.extend(idle.roots.iter());
        }
    }

    /// The identity `message`'s waiters park under.
    pub(crate) fn message_id(&mut self, message: ObjRef) -> MessageId {
        let next = &mut self.next_message;
        *self.message_ids.entry(message).or_insert_with(|| {
            *next += 1;
            MessageId(*next)
        })
    }

    #[cfg(test)]
    pub(crate) fn retired(&self) -> &[rexx_api::ffi::ThreadContext] {
        &self.retired
    }
}

/// How a loop of [`Interp::run_others`] ended.
enum Waited {
    /// The activity it set aside is ready.
    Woken,
    /// Nothing it can run is ready; `inverted` where a loop enclosing it set
    /// a ready activity aside.
    Blocked { inverted: bool },
}

/// How a run of the running activity stopped.
enum Stopped {
    Parked,
    Ended,
}

/// What the driver asks of whatever runs the interpreter's activities.
pub(crate) trait Scheduler {
    /// Files a new activity whose first step is `send`, as ready.
    fn spawn(&mut self, send: StartedSend, then: Then) -> ActivityId;
    /// Runs the running activity, a started one, until it parks, which
    /// answers `true`, or ends.
    fn run_until_park(&mut self) -> Result<bool, Failure>;
    /// Registers the running activity as waiting for `reason`.
    fn park(&mut self, reason: ParkReason);
    /// Moves a parked activity to the back of the ready queue.
    fn unpark(&mut self, activity: ActivityId);
}

impl Scheduler for Interp {
    fn spawn(&mut self, send: StartedSend, then: Then) -> ActivityId {
        let mut activity = Activity::new();
        activity.first_send = Some(Box::new(send));
        activity.root_then = Some(then);
        let mut roots = ActivityRoots::new();
        roots.set_frame_block(self.roots.activity().frames().size());
        let table = &mut self.activities;
        let id = match table.free.pop() {
            Some(index) => ActivityId(index),
            None => {
                table.idle.push(None);
                ActivityId((table.idle.len() - 1) as u32)
            }
        };
        table.idle[id.0 as usize] = Some(Box::new(Idle { activity, roots }));
        table.ready.push_back(id);
        id
    }

    fn run_until_park(&mut self) -> Result<bool, Failure> {
        self.run_started()
            .map(|stopped| matches!(stopped, Stopped::Parked))
    }

    fn park(&mut self, reason: ParkReason) {
        match reason {
            ParkReason::Guard => {}
            ParkReason::MessageResult(id) | ParkReason::MessageWait(id) => {
                let running = self.activities.running;
                self.activities.waiters.entry(id).or_default().push(running);
            }
        }
    }

    fn unpark(&mut self, activity: ActivityId) {
        self.activities.ready.push_back(activity);
    }
}

impl Interp {
    /// Wakes every activity waiting on `message`, whose send has completed.
    pub(crate) fn message_completed(&mut self, message: ObjRef) {
        let Some(id) = self.activities.message_ids.remove(&message) else {
            return;
        };
        for waiter in self.activities.waiters.remove(&id).unwrap_or_default() {
            self.unpark(waiter);
        }
    }

    /// Puts the running activity's state into `next`'s record and runs
    /// `next`; the outgoing activity stays filed under its handle, or is
    /// retired where it has `ended`.
    fn switch_to(&mut self, next: ActivityId, ended: bool) {
        let table = &mut self.activities;
        let Some(mut idle) = table.idle[next.0 as usize].take() else {
            unreachable!("a ready handle names an idle activity");
        };
        std::mem::swap(&mut self.activity, &mut idle.activity);
        std::mem::swap(self.roots.activity_mut(), &mut idle.roots);
        let outgoing = std::mem::replace(&mut table.running, next);
        if ended {
            table.retired.push(idle.activity.thread.clone());
            table.free.push(outgoing.0);
        } else {
            table.idle[outgoing.0 as usize] = Some(idle);
        }
    }

    /// Runs other activities until the running one, parked, is woken. The
    /// failure, where nothing left to run can wake it or another activity
    /// failed with something that is not a condition, is the one its wait
    /// answers.
    pub(crate) fn wait_until_woken(&mut self) -> Option<Failure> {
        let failure = match self.run_others() {
            Ok(Waited::Woken) => return None,
            Ok(Waited::Blocked { .. }) => Loud::unsatisfiable_wait().into(),
            Err(failure) => failure,
        };
        self.cancel_wait();
        Some(failure)
    }

    /// A wait for `reason` with Rust frames between the scheduler and the
    /// running driver (spec 2026-09-29 section 2.6): the other activities run
    /// on this stack until the running one is woken. Where nothing can run,
    /// the refusal is an inverted wait if a loop enclosing this one set a
    /// ready activity aside, and a wait nothing can end otherwise.
    pub(crate) fn pinned_wait(&mut self, reason: ParkReason) -> Option<Failure> {
        self.park(reason);
        let failure = match self.run_others() {
            Ok(Waited::Woken) => return None,
            Ok(Waited::Blocked { inverted: false }) => Loud::unsatisfiable_wait().into(),
            Ok(Waited::Blocked { inverted: true }) => {
                inverted_wait!(self, reason);
                Loud::inverted_wait(reason.waits_for()).into()
            }
            Err(failure) => failure,
        };
        self.cancel_wait();
        Some(failure)
    }

    /// Withdraws the running activity from every message's waiters.
    fn cancel_wait(&mut self) {
        let running = self.activities.running;
        for waiters in self.activities.waiters.values_mut() {
            waiters.retain(|waiter| *waiter != running);
        }
    }

    /// [`Interp::run_started_activities`] until it has nothing left to run,
    /// with each failure it answered on the way, in order.
    pub(crate) fn run_started_to_end(&mut self) -> Vec<(Failure, Vec<crate::FailureSite>)> {
        let mut failures = Vec::new();
        while let Err(failure) = self.run_started_activities() {
            failures.push((failure, Vec::new()));
        }
        failures
    }

    /// Runs every started activity to its end, for the end of the program.
    /// Activities left waiting are abandoned, with the refusal.
    pub(crate) fn run_started_activities(&mut self) -> Result<(), Failure> {
        self.run_others()?;
        self.cancel_wait();
        let table = &mut self.activities;
        let me = table.running.0 as usize;
        let mut blocked = false;
        for (index, idle) in table.idle.iter_mut().enumerate() {
            if index == me {
                continue;
            }
            if let Some(idle) = idle.take() {
                table.retired.push(idle.activity.thread.clone());
                table.free.push(index as u32);
                blocked = true;
            }
        }
        if blocked {
            table.waiters.clear();
            table.message_ids.clear();
            return Err(Loud::unsatisfiable_wait().into());
        }
        Ok(())
    }

    /// Runs ready activities with the running one set aside, until it is
    /// ready again or nothing can run. It is running again on return. An
    /// activity whose Rust frames lie below this loop, another loop's owner,
    /// is not run here: it is set aside and goes back to the ready queue for
    /// its own loop when this one returns.
    fn run_others(&mut self) -> Result<Waited, Failure> {
        let me = self.activities.running;
        let level = self.activities.owners.len();
        self.activities.owners.push(me);
        let waited = self.run_others_from(me, level);
        let table = &mut self.activities;
        let mine = table
            .set_aside
            .iter()
            .position(|(_, by)| *by >= level)
            .unwrap_or(table.set_aside.len());
        for (activity, _) in table.set_aside.drain(mine..).rev() {
            table.ready.push_front(activity);
        }
        table.owners.pop();
        waited
    }

    /// [`Interp::run_others`]' loop, the one at `level` of the stack.
    fn run_others_from(&mut self, me: ActivityId, level: usize) -> Result<Waited, Failure> {
        let Some(next) = self.next_runnable(me, level) else {
            return Ok(self.blocked(level));
        };
        if next == me {
            return Ok(Waited::Woken);
        }
        self.switch_to(next, false);
        loop {
            let parked = self.run_until_park();
            let ended = !matches!(parked, Ok(true));
            if let Err(failure) = parked {
                self.activities.ready.retain(|ready| *ready != me);
                self.switch_to(me, ended);
                return Err(failure);
            }
            match self.next_runnable(me, level) {
                Some(ready) if ready != me => self.switch_to(ready, ended),
                woken => {
                    self.switch_to(me, ended);
                    return Ok(match woken {
                        Some(_) => Waited::Woken,
                        None => self.blocked(level),
                    });
                }
            }
        }
    }

    /// The loop at `level` has nothing to run: inverted where a loop
    /// enclosing it set a ready activity aside.
    fn blocked(&self, level: usize) -> Waited {
        Waited::Blocked {
            inverted: self.activities.set_aside.iter().any(|(_, by)| *by < level),
        }
    }

    /// The next ready activity the loop at `level` can run, or `me`; another
    /// loop's owner is set aside.
    fn next_runnable(&mut self, me: ActivityId, level: usize) -> Option<ActivityId> {
        loop {
            let ready = self.activities.ready.pop_front()?;
            if ready == me || !self.activities.owners.contains(&ready) {
                return Some(ready);
            }
            self.activities.set_aside.push((ready, level));
        }
    }

    /// The running activity, a started one, run until it parks or ends: its
    /// first step, or the rest of the work its wake left. Its outcome is
    /// recorded on its message; a failure that is not a condition is
    /// answered.
    fn run_started(&mut self) -> Result<Stopped, Failure> {
        let sent = match self.activity.first_send.take() {
            Some(send) => {
                let frame = self.roots.activity_mut().push_frame();
                let mut roots = Vec::new();
                send.object_roots(&mut roots);
                for object in roots {
                    self.roots.activity_mut().push_temp(object);
                }
                let StartedSend {
                    receiver,
                    name,
                    scope,
                    args,
                    caller,
                } = *send;
                let started = self.begin_send(receiver, &name, scope, &args, caller);
                self.roots.activity_mut().pop_frame(frame);
                match started {
                    Ok(Started::Ran(value)) => Ok(value),
                    Err(failure) => Err(failure),
                    Ok(Started::Entered) => match self.activity.native_park.as_ref() {
                        Some(park) => {
                            let reason = park.reason();
                            self.park(reason);
                            return Ok(Stopped::Parked);
                        }
                        None => match self.prepare_level() {
                            Ok(level) => {
                                match self.drive_from(crate::ir::DriveStart::Level(level), true) {
                                    Ok(crate::ir::Driven::Parked(floor)) => {
                                        self.activity.drive_floor = Some(floor);
                                        return Ok(Stopped::Parked);
                                    }
                                    Ok(crate::ir::Driven::Ended(ended)) => {
                                        self.finish_send(Ok(ended))
                                    }
                                    Err(failure) => self.finish_send(Err(failure)),
                                }
                            }
                            Err(failure) => self.finish_send(Err(failure)),
                        },
                    },
                }
            }
            None => {
                let sent = self.resume_parked(None);
                match self.activity.drive_floor.take() {
                    None => sent,
                    Some(floor) => {
                        match self.drive_from(crate::ir::DriveStart::Woken { floor, sent }, true) {
                            Ok(crate::ir::Driven::Parked(floor)) => {
                                self.activity.drive_floor = Some(floor);
                                return Ok(Stopped::Parked);
                            }
                            Ok(crate::ir::Driven::Ended(ended)) => self.finish_send(Ok(ended)),
                            Err(failure) => self.finish_send(Err(failure)),
                        }
                    }
                }
            }
        };
        let sent = match sent {
            Err(Failure::Exited(value)) => Ok(value),
            other => other,
        };
        if let Err(Failure::Raised(raised)) = &sent {
            self.report_started_failure(raised);
        }
        let Some(then) = self.activity.root_then.take() else {
            unreachable!("a started activity records its outcome");
        };
        match self.apply_then(then, sent) {
            Ok(_) | Err(Failure::Raised(_)) => Ok(Stopped::Ended),
            Err(failure) => Err(failure),
        }
    }

    /// The refusal of a park that reached a caller other than its activity's
    /// root driver with no pinned frame counted, with the park dropped.
    #[cold]
    pub(crate) fn refuse_unrooted_park(&mut self) -> Failure {
        self.activity.native_park = None;
        Loud::scheduler_inconsistency("a wait outside every root driver and pinned frame").into()
    }

    /// The running activity's park, resumed now that it has woken.
    fn resume_parked(&mut self, failure: Option<Failure>) -> Result<Option<ObjRef>, Failure> {
        match self.activity.native_park.take() {
            Some(park) => self.resume_native_park(*park, failure),
            None => {
                Err(Loud::scheduler_inconsistency("a woken activity with no wait recorded").into())
            }
        }
    }

    /// The traceback an untrapped condition writes on the started activity it
    /// ended, at once (`MessageDispatcher.cpp:64-72`).
    fn report_started_failure(&mut self, raised: &crate::Raised) {
        let mut sites = std::mem::take(&mut self.activity.failure_sites);
        sites.extend(self.activity.failure_site.take());
        let path = self.program_path.clone();
        let site = crate::ClauseSite {
            path: &path,
            sites: &sites,
        };
        let report = raised.report(&site);
        self.write_trace_report(&report);
    }

    /// The running activity's main-program root, a park in which runs the
    /// other activities until it is woken.
    pub(crate) fn run_activity_root(&mut self) -> Result<Ended, Failure> {
        let level = self.prepare_level()?;
        let mut driven = self.drive_from(crate::ir::DriveStart::Level(level), true)?;
        loop {
            match driven {
                crate::ir::Driven::Ended(ended) => return Ok(ended),
                crate::ir::Driven::Parked(floor) => {
                    let failure = self.wait_until_woken();
                    let sent = self.resume_parked(failure);
                    driven = self.drive_from(crate::ir::DriveStart::Woken { floor, sent }, true)?;
                }
            }
        }
    }
}

/// An outcome [`crate::Interp::exec_instruction`] answers in place of running
/// its instruction.
#[cfg(test)]
#[derive(Clone, Copy, Debug)]
pub(crate) enum Scripted {
    /// `Park`, without running the instruction.
    Park,
    /// `Split`, with the instruction's own `Flow`.
    Split,
}

#[cfg(test)]
thread_local! {
    static SCRIPT: std::cell::RefCell<std::collections::VecDeque<Scripted>> =
        const { std::cell::RefCell::new(std::collections::VecDeque::new()) };
}

/// Queues `outcomes` for the next `Op::Exec` instructions this thread runs.
#[cfg(test)]
pub(crate) fn script(outcomes: &[Scripted]) {
    SCRIPT.with(|script| script.borrow_mut().extend(outcomes.iter().copied()));
}

/// The next queued outcome, outside the library bootstrap.
#[cfg(test)]
pub(crate) fn take_scripted() -> Option<Scripted> {
    if !crate::ir::drive::counting() {
        return None;
    }
    SCRIPT.with(|script| script.borrow_mut().pop_front())
}

#[cfg(test)]
mod tests;
