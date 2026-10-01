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

use crate::activity::{Activity, HaltRequest};
use crate::dispatch::{Caller, Then};
use crate::error::{Failure, Raised};
use crate::ir::{DriveStart, Driven};
use crate::run::{Ended, Flow, Started};
use crate::timer::SLICE;
use crate::{Interp, Loud, PendingTrap, SwitchMode};

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
    /// The numbers of ended activities a new one takes first, oldest first:
    /// the oracle's pool of idle threads (`availableActivities`,
    /// `concurrency/ActivityManager.cpp:556`), whose threads keep their
    /// numbers, including none.
    pooled: VecDeque<Option<u32>>,
    /// The last number assigned (`Activity::getIdntfr`'s counter).
    last_number: u32,
}

/// `ActivityManager::MAX_THREAD_POOL_SIZE` (`ActivityManager.hpp:358`): an
/// ended activity is pooled while the pool holds no more than this.
const MAX_POOLED: usize = 5;

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
            pooled: VecDeque::new(),
            last_number: 0,
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
    /// Nothing it can run is ready; `inverted` where an activity is ready
    /// that only a loop below it can run.
    Blocked { inverted: bool },
}

/// How a run of the running activity stopped.
enum Stopped {
    /// Parked, or at the back of the ready queue with its slice over.
    Parked,
    Ended,
}

/// The deterministic switch mode in force, and the clauses counted under it.
pub(crate) struct Switch {
    mode: SwitchMode,
    clauses: u64,
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
    /// Puts the running activity, whose slice is over, at the back of the
    /// ready queue.
    fn yield_at_slice(&mut self);
}

impl Scheduler for Interp {
    fn spawn(&mut self, send: StartedSend, then: Then) -> ActivityId {
        // `Activity::setCallerStackFrameAsStringTable` numbers the spawner
        // (`concurrency/Activity.cpp:1206`).
        self.activity_number();
        let mut activity = Activity::new();
        activity.first_send = Some(Box::new(send));
        activity.root_then = Some(then);
        activity.number = self.activities.pooled.pop_front().flatten();
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

    fn yield_at_slice(&mut self) {
        let running = self.activities.running;
        self.activities.ready.push_back(running);
    }
}

impl Interp {
    /// The running activity's number, which `.context~thread` answers: main
    /// is 1 and an activity is numbered when first asked, as the oracle
    /// numbers its threads (`Activity::getIdntfr`,
    /// `concurrency/Activity.cpp:108`).
    pub(crate) fn activity_number(&mut self) -> u32 {
        if let Some(number) = self.activity.number {
            return number;
        }
        self.activities.last_number += 1;
        let number = self.activities.last_number;
        self.activity.number = Some(number);
        number
    }

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
            if table.pooled.len() <= MAX_POOLED {
                table.pooled.push_back(idle.activity.number);
            }
        } else {
            table.idle[outgoing.0 as usize] = Some(idle);
        }
        // A slice belongs to the activity it was set for.
        self.timer.requests().clear(SLICE);
        self.slice_deferred = false;
        if self.activity.halt.is_some() {
            self.clause_countdown = 1;
        }
        if self.stress_collect {
            self.collect_now();
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
    /// the refusal is an inverted wait if an activity is ready whose frames
    /// lie below a loop on the stack, and a wait nothing can end where no
    /// activity is ready at all.
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
            return Ok(self.blocked());
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
                // The activity whose slice ended, with nothing else this loop
                // can run, runs on.
                Some(ready) if ready == self.activities.running => {}
                Some(ready) if ready != me => self.switch_to(ready, ended),
                woken => {
                    self.switch_to(me, ended);
                    return Ok(match woken {
                        Some(_) => Waited::Woken,
                        None => self.blocked(),
                    });
                }
            }
        }
    }

    /// A loop has nothing to run: inverted where some activity is ready
    /// whose frames lie below a loop on the stack, which only that loop can
    /// run.
    fn blocked(&self) -> Waited {
        Waited::Blocked {
            inverted: !self.activities.set_aside.is_empty(),
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
                                let driven = self.drive_from(DriveStart::Level(level), true);
                                match self.started_driven(driven) {
                                    Some(sent) => sent,
                                    None => return Ok(Stopped::Parked),
                                }
                            }
                            Err(failure) => self.finish_send(Err(failure)),
                        },
                    },
                }
            }
            None => 'resumed: {
                let driven = match self.activity.sliced.take() {
                    Some((floor, at)) => self.drive_from(DriveStart::Sliced { floor, at }, true),
                    None => {
                        let sent = self.resume_parked(None);
                        let Some(floor) = self.activity.drive_floor.take() else {
                            break 'resumed sent;
                        };
                        self.drive_from(DriveStart::Woken { floor, sent }, true)
                    }
                };
                match self.started_driven(driven) {
                    Some(sent) => sent,
                    None => return Ok(Stopped::Parked),
                }
            }
        };
        let sent = match sent {
            Err(Failure::Exited(value)) => Ok(value),
            other => other,
        };
        let untrapped = match &sent {
            Err(Failure::Raised(raised)) => Some(raised.clone()),
            _ => None,
        };
        let Some(then) = self.activity.root_then.take() else {
            unreachable!("a started activity records its outcome");
        };
        let recorded = self.apply_then(then, sent);
        if let Some(raised) = untrapped {
            self.settle_failed_sends(&raised)?;
            self.report_started_failure(&raised);
        }
        match recorded {
            Ok(_) | Err(Failure::Raised(_)) => Ok(Stopped::Ended),
            Err(failure) => Err(failure),
        }
    }

    /// What a root driver's run of the running started activity answers:
    /// its send's outcome, or `None` where it parked or its slice ended.
    fn started_driven(
        &mut self,
        driven: Result<Driven, Failure>,
    ) -> Option<Result<Option<ObjRef>, Failure>> {
        match driven {
            Ok(Driven::Parked(floor)) => {
                self.activity.drive_floor = Some(floor);
                None
            }
            Ok(Driven::Sliced { floor, at }) => {
                self.activity.sliced = Some((floor, at));
                self.yield_at_slice();
                None
            }
            Ok(Driven::Ended(ended)) => Some(self.finish_send(Ok(ended))),
            Err(failure) => Some(self.finish_send(Err(failure))),
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
        // With no level that has a line, the report names no program or line
        // (`Activity::display` with no `POSITION`).
        let report = if sites.iter().any(|site| site.line().is_some()) {
            raised.report(&site)
        } else {
            let mut positionless = raised.clone();
            positionless.delivery.positionless = true;
            positionless.report(&site)
        };
        self.write_trace_report(&report);
    }

    /// The running activity's main-program root, a park or slice in which
    /// runs the other activities until it is ready again.
    pub(crate) fn run_activity_root(&mut self) -> Result<Ended, Failure> {
        let level = self.prepare_level()?;
        let mut driven = self.drive_from(DriveStart::Level(level), true)?;
        loop {
            driven = match driven {
                Driven::Ended(ended) => return Ok(ended),
                Driven::Parked(floor) => {
                    let failure = self.wait_until_woken();
                    let sent = self.resume_parked(failure);
                    self.drive_from(DriveStart::Woken { floor, sent }, true)?
                }
                Driven::Sliced { floor, at } => {
                    self.yield_at_slice();
                    if let Some(failure) = self.wait_until_woken() {
                        return Err(failure);
                    }
                    self.drive_from(DriveStart::Sliced { floor, at }, true)?
                }
            };
        }
    }

    /// Switches activities as `mode` says, and never on the timer's word.
    pub(crate) fn set_switch_mode(&mut self, mode: SwitchMode) {
        self.switch = Some(Switch { mode, clauses: 0 });
        self.timer.disarm();
        self.clause_countdown = 1;
    }

    /// The requests a visit of the clause countdown serves, before the clause
    /// begins: the switch mode's count, the timer's arming, a `HALT` asked of
    /// the running activity, and `SLICE`. A slice ends only where the clause
    /// `yields` and nothing pins the activity; anywhere else it is deferred
    /// to the next clause that does (spec 2026-09-29 P6-4).
    pub(crate) fn serve_requests(&mut self, yields: bool) -> Result<(), Failure> {
        if self.stress_collect {
            self.collect_now();
        }
        if let Some(switch) = &mut self.switch {
            self.clause_countdown = 1;
            switch.clauses += 1;
            let due = match switch.mode {
                SwitchMode::EveryOpportunity => true,
                SwitchMode::AtClause(clause) => switch.clauses == clause,
            };
            if due {
                self.timer.requests().set(SLICE);
            }
        } else if self.activities.ready.is_empty() {
            self.timer.disarm();
        } else {
            self.timer.arm();
        }
        if let Some(halt) = self.activity.halt.take() {
            return self.raise_requested_halt(halt);
        }
        if !self.timer.requests().pending(SLICE) {
            return Ok(());
        }
        if self.activities.ready.is_empty() {
            self.timer.requests().clear(SLICE);
            self.slice_deferred = false;
            return Ok(());
        }
        if !yields || self.activity.pin_depth > 0 {
            if !self.slice_deferred {
                self.slice_deferred = true;
                deferred_slice!(self);
            }
            self.clause_countdown = 1;
            return Ok(());
        }
        self.timer.requests().clear(SLICE);
        self.slice_deferred = false;
        Err(Failure::Slice)
    }

    /// `RexxActivation::processClauseBoundary`'s halt
    /// (`execution/RexxActivation.cpp:4085`-`:4093`), at the start of the
    /// clause after the one the request found running, whose line the
    /// condition still names: a `CALL ON` trap queues it, a `SIGNAL ON` trap
    /// takes it, and untrapped it is 4.1.
    fn raise_requested_halt(&mut self, halt: HaltRequest) -> Result<(), Failure> {
        let raised = Raised {
            description: halt.description.clone(),
            ..Raised::condition(std::borrow::Cow::Borrowed("HALT"))
        };
        match self.trap_for(b"HALT") {
            Some(trap) if trap.call => {
                let object = self.build_condition_object(&raised, Some(true))?;
                self.activity.pending_traps.push_back(PendingTrap {
                    condition: b"HALT".as_slice().into(),
                    rc: None,
                    description: halt.description,
                    object: Some(object),
                    activation: self.activation().id,
                    queued_during_delivery: true,
                    fragment_depth: self.activity.fragment_depth,
                });
                Ok(())
            }
            Some(_) => {
                self.record_entered_clause_site();
                Err(raised.into())
            }
            None => {
                self.record_entered_clause_site();
                Err(Raised::halt().into())
            }
        }
    }

    /// The clause the running activation entered last, as the site of a
    /// failure raised before the next one begins.
    fn record_entered_clause_site(&mut self) {
        let program = std::rc::Rc::clone(&self.activation().program);
        let plan = std::rc::Rc::clone(&self.activation().plan);
        let Some(body) = crate::activation::body_of(&program, self.activation().body) else {
            return;
        };
        let code = crate::Code {
            body,
            symbols: &program.symbols,
            slots: &plan.by_symbol,
            plan: Some(&plan),
        };
        let index = self.activity.clause_state.clause_index();
        if let Some(instruction) = body.instructions.get(index) {
            self.record_failure_site(&code, index, Some(&program.source), instruction);
        }
    }

    /// `Message~halt` (`MessageClass::halt`, `classes/MessageClass.cpp:806`):
    /// false for a message never started and for an activity already asked;
    /// true where the request is made, and where the started activity has no
    /// Rexx frame to make it to (`Activity::halt`,
    /// `concurrency/Activity.cpp:2155`), which drops it.
    pub(crate) fn halt_message(&mut self, message: ObjRef, description: Option<Vec<u8>>) -> bool {
        if !self.started_messages.contains(&message) {
            return false;
        }
        let runs = |activity: &Activity| matches!(activity.root_then, Some(Then::Started(started)) if started == message);
        let running = runs(&self.activity);
        let target = if running {
            Some(&mut self.activity)
        } else {
            self.activities
                .idle
                .iter_mut()
                .flatten()
                .map(|idle| &mut idle.activity)
                .find(|activity| runs(activity))
        };
        let Some(target) = target else {
            return true;
        };
        if target.running.is_none() {
            return true;
        }
        if target.halt.is_some() {
            return false;
        }
        target.halt = Some(HaltRequest { description });
        if running {
            self.clause_countdown = 1;
        }
        true
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
