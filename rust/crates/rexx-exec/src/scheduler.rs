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
//! wait, inside the Rust frames that pin the outgoing activity. A pinned
//! waiter that becomes ready while a loop nested above its own runs waits
//! for that loop to return, so a buried sleeper can wake after a later
//! deadline: a recorded divergence from the oracle, counted as a late wake.

use std::cmp::Reverse;
use std::collections::{BinaryHeap, VecDeque};
use std::time::Instant;

use rexx_core::{ActivityRoots, ObjRef};
use rustc_hash::FxHashMap;

use crate::activity::Activity;
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
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
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
    /// `SysSleep`, until `deadline`.
    Sleep { deadline: Instant },
}

impl ParkReason {
    /// What a wait for this reason waits for, as a refusal names it.
    fn waits_for(self) -> &'static str {
        match self {
            ParkReason::Guard => "a guard",
            ParkReason::MessageResult(_) | ParkReason::MessageWait(_) => "a message's completion",
            ParkReason::Sleep { .. } => "a sleep's end",
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
    /// Sleeping activities, earliest deadline first, then in the order they
    /// parked.
    sleepers: BinaryHeap<Reverse<(Instant, u64, ActivityId)>>,
    /// The order the next sleeper parks in.
    next_sleeper: u64,
    /// The activities whose loops of [`Interp::run_others`] are on the Rust
    /// stack, outermost first, each with whether it is buried: whether it
    /// entered its loop from Rust frames that stay live below it (ruling P30).
    owners: Vec<(ActivityId, bool)>,
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
    /// Failures kept by [`Interp::keep_late_failure`].
    late_failures: Vec<Failure>,
}

/// Main's handle, which `Activities::new` gives the activity that runs the
/// program.
const MAIN: ActivityId = ActivityId(0);

/// The Rust stack kept free below the deepest point
/// [`Interp::stack_exhausted`] admits, for the work between two of its
/// checks and the raise of 11.1.
const STACK_MARGIN: usize = 32 * 1024 * 1024;

/// `ActivityManager::MAX_THREAD_POOL_SIZE` (`ActivityManager.hpp:358`): an
/// ended activity is pooled while the pool holds no more than this.
const MAX_POOLED: usize = 5;

impl Activities {
    pub(crate) fn new() -> Activities {
        Activities {
            running: MAIN,
            idle: vec![None],
            free: Vec::new(),
            ready: VecDeque::new(),
            sleepers: BinaryHeap::new(),
            next_sleeper: 0,
            owners: Vec::new(),
            set_aside: Vec::new(),
            message_ids: FxHashMap::default(),
            next_message: 0,
            waiters: FxHashMap::default(),
            retired: Vec::new(),
            pooled: VecDeque::new(),
            last_number: 0,
            late_failures: Vec::new(),
        }
    }

    /// Appends every `ObjRef` an activity that is not running holds.
    pub(crate) fn object_roots(&self, out: &mut Vec<ObjRef>) {
        for failure in &self.late_failures {
            if let Failure::Exited(value) = failure {
                out.extend(*value);
            }
        }
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
        let running = self.activities.running;
        let table = &mut self.activities;
        match reason {
            ParkReason::Guard => {}
            ParkReason::MessageResult(id) | ParkReason::MessageWait(id) => {
                table.waiters.entry(id).or_default().push(running);
            }
            ParkReason::Sleep { deadline } => {
                table.next_sleeper += 1;
                table
                    .sleepers
                    .push(Reverse((deadline, table.next_sleeper, running)));
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
        if self.activities.ready.is_empty() {
            self.timer.disarm();
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
        let failure = match self.run_others(false) {
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
        pinned_park!(self, reason);
        self.park(reason);
        let waited = self.run_others(true);
        pinned_unpark!(self);
        let failure = match waited {
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

    /// Withdraws the running activity from every message's waiters and from
    /// the sleepers.
    fn cancel_wait(&mut self) {
        let running = self.activities.running;
        for waiters in self.activities.waiters.values_mut() {
            waiters.retain(|waiter| *waiter != running);
        }
        self.activities
            .sleepers
            .retain(|Reverse((_, _, sleeper))| *sleeper != running);
    }

    /// Measures this interpreter's Rust stack from `base`, an address near
    /// the base of a thread stack of `bytes`.
    pub(crate) fn measure_stack(&mut self, base: usize, bytes: usize) {
        self.stack_base = base;
        self.stack_room = bytes.saturating_sub(STACK_MARGIN);
    }

    /// Whether `here`, an address on this thread's stack, lies deeper than
    /// the stack may reach: past it a call, a nested scheduler loop or an
    /// expression level raises 11.1 rather than risk the thread's end.
    #[inline(always)]
    pub(crate) fn stack_exhausted<T>(&self, here: *const T) -> bool {
        self.stack_base.abs_diff(here as usize) > self.stack_room
    }

    /// Moves every sleeper whose deadline is due to the ready queue, in
    /// deadline order.
    fn wake_due_sleepers(&mut self) {
        let now = Instant::now();
        let table = &mut self.activities;
        while let Some(Reverse((deadline, _, sleeper))) = table.sleepers.peek().copied() {
            if deadline > now {
                break;
            }
            table.sleepers.pop();
            table.ready.push_back(sleeper);
        }
    }

    /// [`Interp::run_started_activities`] until it has nothing left to run,
    /// with each failure it answered on the way, in order.
    pub(crate) fn run_started_to_end(&mut self) -> Vec<(Failure, Vec<crate::FailureSite>)> {
        let mut failures: Vec<_> = std::mem::take(&mut self.activities.late_failures)
            .into_iter()
            .map(|failure| (failure, Vec::new()))
            .collect();
        while let Err(failure) = self.run_started_activities() {
            failures.push((failure, Vec::new()));
        }
        failures
    }

    /// A failure another activity answered main's root loop with after
    /// main's own end, kept for the program's end to report.
    fn keep_late_failure(&mut self, failure: Failure) {
        self.activities.late_failures.push(failure);
    }

    /// Runs every started activity to its end, for the end of the program.
    /// Activities left waiting are abandoned, with the refusal.
    pub(crate) fn run_started_activities(&mut self) -> Result<(), Failure> {
        self.run_others(false)?;
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
            table.sleepers.clear();
            return Err(Loud::unsatisfiable_wait().into());
        }
        Ok(())
    }

    /// Runs ready activities with the running one set aside, until it is
    /// ready again, it has ended as main ends in another loop, or nothing can
    /// run. It is running again on return. The running activity is `buried`
    /// where it waits from Rust frames below this loop; a buried activity is
    /// not run by a loop above its own: it is set aside and goes back to the
    /// ready queue for its own loop when this one returns. An activity whose
    /// state is all in its record runs in any loop (ruling P30).
    fn run_others(&mut self, buried: bool) -> Result<Waited, Failure> {
        self.run_round(buried).map(|(waited, _)| waited)
    }

    /// [`Interp::run_others`], answering besides whether the loop set aside
    /// a buried activity that was ready.
    fn run_round(&mut self, buried: bool) -> Result<(Waited, bool), Failure> {
        let me = self.activities.running;
        let level = self.activities.owners.len();
        let probe = 0u8;
        if self.stack_exhausted(&raw const probe) {
            self.activities.ready.retain(|ready| *ready != me);
            return Err(Raised::insufficient_stack().into());
        }
        self.activities.owners.push((me, buried));
        let waited = self.run_others_from(me, level);
        let table = &mut self.activities;
        let mine = table
            .set_aside
            .iter()
            .position(|(_, by)| *by >= level)
            .unwrap_or(table.set_aside.len());
        let set_aside = mine < table.set_aside.len();
        for (activity, _) in table.set_aside.drain(mine..).rev() {
            table.ready.push_front(activity);
        }
        table.owners.pop();
        waited.map(|waited| (waited, set_aside))
    }

    /// [`Interp::run_others`]' loop, the one at `level` of the stack.
    fn run_others_from(&mut self, me: ActivityId, level: usize) -> Result<Waited, Failure> {
        let Some(next) = self.next_runnable(me, level)? else {
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
            if self.root_ended(me) {
                self.switch_to(me, ended);
                return Ok(Waited::Woken);
            }
            let woken = match self.next_runnable(me, level) {
                Ok(woken) => woken,
                Err(failure) => {
                    self.activities.ready.retain(|ready| *ready != me);
                    self.switch_to(me, ended);
                    return Err(failure);
                }
            };
            match woken {
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

    /// Whether `activity`, not running, is main whose root driver ended in a
    /// loop other than its own.
    fn root_ended(&self, activity: ActivityId) -> bool {
        self.activities
            .idle
            .get(activity.0 as usize)
            .and_then(Option::as_ref)
            .is_some_and(|idle| idle.activity.root_end.is_some())
    }

    /// The next ready activity the loop at `level` can run, or `me`; a buried
    /// activity is set aside. With none ready, this thread idles until the
    /// earliest sleeper is due; the run's deadline passing first fails it.
    fn next_runnable(
        &mut self,
        me: ActivityId,
        level: usize,
    ) -> Result<Option<ActivityId>, Failure> {
        loop {
            if !self.activities.sleepers.is_empty() {
                self.wake_due_sleepers();
            }
            let Some(ready) = self.activities.ready.pop_front() else {
                let Some(Reverse((due, _, _))) = self.activities.sleepers.peek().copied() else {
                    return Ok(None);
                };
                self.idle_until(due)?;
                continue;
            };
            let buried = self
                .activities
                .owners
                .iter()
                .any(|&(owner, buried)| owner == ready && buried);
            if ready == me || !buried {
                return Ok(Some(ready));
            }
            #[cfg(feature = "pinning")]
            if let Some(Some(idle)) = self.activities.idle.get(ready.0 as usize) {
                late_wake!(self, idle.activity);
            }
            self.activities.set_aside.push((ready, level));
        }
    }

    /// The running activity, a started one, run until it parks or ends: its
    /// first step, or the rest of the work its wake left. Its outcome is
    /// recorded on its message; a failure that is not a condition is
    /// answered.
    fn run_started(&mut self) -> Result<Stopped, Failure> {
        if self.activities.running == MAIN {
            self.root_step(None);
            return Ok(Stopped::Parked);
        }
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
    /// runs the other activities until it is ready again. Its continuation
    /// is kept in its record, so a nested loop may run it too, and its end
    /// there is recorded for this loop to answer (ruling P30).
    pub(crate) fn run_activity_root(&mut self) -> Result<Ended, Failure> {
        let level = self.prepare_level()?;
        let driven = self.drive_from(DriveStart::Level(level), true);
        self.root_driven(driven);
        loop {
            if let Some(end) = self.activity.root_end.take() {
                return end;
            }
            let failure = self.wait_until_woken();
            match failure {
                // Main ended in a nested round, and this failure came after:
                // the program's end reports it, as it does once main has
                // ended here.
                Some(failure) if self.activity.root_end.is_some() => {
                    self.keep_late_failure(failure);
                }
                failure if self.activity.root_end.is_none() => self.root_step(failure),
                _ => {}
            }
        }
    }

    /// Main's root driver run from its recorded continuation, with `failure`
    /// as the answer of the wait it ends.
    fn root_step(&mut self, failure: Option<Failure>) {
        let driven = match (self.activity.sliced.take(), failure) {
            (Some(_), Some(failure)) => Err(failure),
            (Some((floor, at)), None) => self.drive_from(DriveStart::Sliced { floor, at }, true),
            (None, failure) => match self.activity.drive_floor.take() {
                Some(floor) => {
                    let sent = self.resume_parked(failure);
                    self.drive_from(DriveStart::Woken { floor, sent }, true)
                }
                None => Err(Loud::scheduler_inconsistency("a main with no continuation").into()),
            },
        };
        self.root_driven(driven);
    }

    /// Records where main's root driver stopped: its park, its slice, which
    /// puts it at the back of the ready queue, or its end.
    fn root_driven(&mut self, driven: Result<Driven, Failure>) {
        match driven {
            Ok(Driven::Parked(floor)) => self.activity.drive_floor = Some(floor),
            Ok(Driven::Sliced { floor, at }) => {
                self.activity.sliced = Some((floor, at));
                self.yield_at_slice();
            }
            Ok(Driven::Ended(ended)) => self.activity.root_end = Some(Ok(ended)),
            Err(failure) => self.activity.root_end = Some(Err(failure)),
        }
    }

    /// Switches activities as `mode` says, and never on the timer's word.
    pub(crate) fn set_switch_mode(&mut self, mode: SwitchMode) {
        self.switch = Some(Switch { mode, clauses: 0 });
        self.timer.disarm();
        self.clause_countdown = 1;
    }

    /// The requests a visit of the clause countdown serves, before the clause
    /// begins: the sleepers now due, the switch mode's count, the timer's
    /// arming, and `SLICE`. A slice ends where the clause `yields` and
    /// nothing pins the activity; anywhere else it is deferred to the next
    /// clause that does (spec 2026-09-29 P6-4), and a second slice that finds
    /// the activity still pinned takes a pinned yield (ruling P29).
    pub(crate) fn serve_requests(&mut self, yields: bool) -> Result<(), Failure> {
        if self.stress_collect {
            self.collect_now();
        }
        if !self.activities.sleepers.is_empty() {
            self.wake_due_sleepers();
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
        let fresh = self.timer.requests().pending(SLICE);
        if !fresh && !self.slice_deferred {
            return Ok(());
        }
        if fresh {
            self.timer.requests().clear(SLICE);
        }
        if self.activities.ready.is_empty() {
            self.slice_deferred = false;
            return Ok(());
        }
        if yields && self.activity.pin_depth == 0 {
            self.slice_deferred = false;
            return Err(Failure::Slice);
        }
        if fresh && self.slice_deferred && self.activity.pin_depth > 0 {
            self.slice_deferred = false;
            return self.pinned_yield();
        }
        if fresh && !self.slice_deferred {
            self.slice_deferred = true;
            deferred_slice!(self);
        }
        self.clause_countdown = 1;
        Ok(())
    }

    /// A pinned activity's yield (ruling P29): one round of the other ready
    /// activities on its own stack, as a pinned wait runs them, each until it
    /// parks, ends or its own slice ends; those whose frames lie below are
    /// set aside, and a round that set one aside is counted as an inverted
    /// yield. A pinned busy-wait on a buried activity therefore spins where
    /// the oracle completes, a recorded divergence (ruling P31).
    fn pinned_yield(&mut self) -> Result<(), Failure> {
        pinned_yield!(self);
        self.yield_at_slice();
        let (_, set_aside) = self.run_round(true)?;
        if set_aside {
            inverted_yield!(self);
        }
        Ok(())
    }

    /// `RexxActivation::processClauseBoundary`'s halt
    /// (`execution/RexxActivation.cpp:4085`-`:4093`), at the end of the
    /// clause the request found running, in that clause's activation: a
    /// `CALL ON` trap queues it, a `SIGNAL ON` trap takes it, and untrapped it
    /// is 4.1.
    pub(crate) fn raise_requested_halt(
        &mut self,
        description: Option<Vec<u8>>,
    ) -> Result<(), Failure> {
        let raised = Raised {
            description: description.clone(),
            ..Raised::condition(std::borrow::Cow::Borrowed("HALT"))
        };
        match self.trap_for(b"HALT") {
            Some(trap) if trap.call => {
                let object = self.build_condition_object(&raised, Some(true))?;
                self.activity.pending_traps.push_back(PendingTrap {
                    condition: b"HALT".as_slice().into(),
                    rc: None,
                    description,
                    object: Some(object),
                    activation: self.activation().id,
                    queued_during_delivery: true,
                    request: false,
                    fragment_depth: self.activity.fragment_depth,
                });
                Ok(())
            }
            Some(_) => Err(raised.into()),
            None => Err(Raised::halt().into()),
        }
    }

    /// `Message~halt` (`MessageClass::halt`, `classes/MessageClass.cpp:806`):
    /// false for a message never started and for an activity already asked;
    /// true where the request is queued for the started activity's running
    /// activation (`RexxActivation::halt`), and where that activity has no
    /// Rexx frame to make it to (`Activity::halt`,
    /// `concurrency/Activity.cpp:2155`), which drops it.
    pub(crate) fn halt_message(&mut self, message: ObjRef, description: Option<Vec<u8>>) -> bool {
        if !self.started_messages.contains(&message) {
            return false;
        }
        let runs = |activity: &Activity| matches!(activity.root_then, Some(Then::Started(started)) if started == message);
        let target = if runs(&self.activity) {
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
        let Some(activation) = target.running.as_ref().map(|running| running.id) else {
            return true;
        };
        if target.pending_traps.iter().any(|pending| pending.request) {
            return false;
        }
        target.pending_traps.push_back(PendingTrap {
            condition: b"HALT".as_slice().into(),
            rc: None,
            description,
            object: None,
            activation,
            queued_during_delivery: true,
            request: true,
            fragment_depth: target.fragment_depth,
        });
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
