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
    /// The instruction parks the activity for this reason, and runs again
    /// once it wakes.
    Park(ParkReason),
    /// A `REPLY` with this value: the rest of its body goes to a new
    /// activity at the next clause boundary.
    Split(Option<ObjRef>),
}

/// An activity's handle: its offset in the interpreter's activity table.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct ActivityId(u32);

impl ActivityId {
    /// The handle's offset in the activity table.
    pub(crate) fn index(self) -> usize {
        self.0 as usize
    }
}

#[cfg(test)]
impl ActivityId {
    pub(crate) fn test(index: u32) -> ActivityId {
        ActivityId(index)
    }
}

/// A timer the natives behind `.Alarm` and `.Ticker` wait on, numbered per
/// interpreter from 1; its `EVENTSEMHANDLE` pointer holds the number.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct TimerId(u32);

impl TimerId {
    pub(crate) fn from_address(address: usize) -> Option<TimerId> {
        u32::try_from(address).ok().map(TimerId)
    }

    pub(crate) fn address(self) -> usize {
        self.0 as usize
    }
}

/// A timer's state: the oracle's event semaphore (`SysSemaphore`), posted
/// until reset, and the activities waiting on it.
#[derive(Default)]
struct Timer {
    posted: bool,
    /// Whether a post woke a waiter whose object was cancelled.
    cancelled: bool,
    /// Whether the timer ends with a wait on it, as an alarm's does.
    ends_with_wait: bool,
    /// The wait about to park: its receiver, scope and the end of its whole
    /// days.
    pending: Option<(ObjRef, ObjRef, Instant)>,
    waiters: Vec<TimerWaiter>,
}

/// One activity waiting on a timer.
struct TimerWaiter {
    activity: ActivityId,
    /// Its sleeper's park order.
    order: u64,
    /// The waiting call's receiver and method scope, whose `CANCELED` a post
    /// reads.
    receiver: ObjRef,
    scope: ObjRef,
    days_end: Instant,
}

/// A message object's identity among the activities waiting on it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct MessageId(u32);

/// Why an activity parks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ParkReason {
    /// A guard lock, until a release makes this activity its owner.
    Guard(crate::guards::GuardKey),
    /// A `GUARD WHEN`, until a store or `DROP` of a variable it watches.
    GuardWhen,
    /// `~result`, until the message's send completes.
    MessageResult(MessageId),
    /// `~wait`, until the message's send completes.
    MessageWait(MessageId),
    /// `SysSleep`, until `deadline`.
    Sleep { deadline: Instant },
    /// A timer's wait, until `deadline` or a post to `cancel`.
    Timer { deadline: Instant, cancel: TimerId },
    /// A semaphore's wait, until a post or release hands it over or a timed
    /// wait's end.
    Semaphore(crate::semaphores::SemaphoreWait),
    /// A native call prepared on the baton, until its completion is drained.
    Native,
    /// A blocking operation run off the baton, until it has ended.
    Block,
}

impl ParkReason {
    /// What a wait for this reason waits for, as a refusal names it.
    fn waits_for(self) -> &'static str {
        match self {
            ParkReason::Guard(_) => "a guard",
            ParkReason::GuardWhen => "a guard's WHEN",
            ParkReason::MessageResult(_) | ParkReason::MessageWait(_) => "a message's completion",
            ParkReason::Sleep { .. } => "a sleep's end",
            ParkReason::Timer { .. } => "a timer's end",
            ParkReason::Semaphore(_) => "a semaphore",
            ParkReason::Native => "a native call's return",
            ParkReason::Block => "a command's end",
        }
    }
}

/// What other threads post to an interpreter's inbox.
pub(crate) enum Posted {
    Completed(Completed),
    Recall(Recall),
    /// `activity`'s blocking operation, posted under `token`, has ended.
    Unblocked {
        activity: ActivityId,
        token: u64,
        ended: crate::command::Waited,
    },
    /// What a command's child off the baton wrote, to standard error where
    /// `error`, written to the interpreter's own as it arrives.
    Output {
        error: bool,
        bytes: Vec<u8>,
    },
    /// A pool thread's job panicked; the baton's holder panics with it.
    Panicked(Box<dyn std::any::Any + Send>),
    /// A signal arrived: every activity is halted ([`Interp::halt_all`]).
    Halt,
    /// A chunk of standard input, empty at its end.
    Input(Vec<u8>),
}

// Another thread posts it, so it holds no `ObjRef`.
const _: () = {
    const fn require_send<T: Send>() {}
    require_send::<Posted>();
};

/// A native call's completion, posted to the inbox by the thread that ran
/// the call, for the activity that made it.
pub(crate) struct Completed {
    pub(crate) activity: ActivityId,
    /// The call's native frame, as [`Interp::native_token`] names it: a call
    /// abandoned while it ran completes no later call of its activity.
    pub(crate) frame: u64,
    pub(crate) completion: rexx_api::invoke::Completion,
}

/// A thread's request for the baton, for a callback or a native call's end.
pub(crate) struct Recall {
    pub(crate) by: Recaller,
    pub(crate) thread: std::thread::ThreadId,
    /// The address of a local near the base of that thread's stack, where
    /// Rexx code may run under the lend.
    pub(crate) base: Option<usize>,
}

/// Which activity runs while a recalled baton is lent.
pub(crate) enum Recaller {
    /// A pool thread running `activity`'s native call in the native frame
    /// `frame`, as [`Interp::native_token`] names it.
    Call { activity: ActivityId, frame: u64 },
    /// A thread running no native call of this interpreter's, calling back
    /// through the activity's thread context at this address.
    Context(usize),
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
pub(crate) struct Idle {
    pub(crate) activity: Activity,
    pub(crate) roots: ActivityRoots,
}

impl Idle {
    pub(crate) fn object_roots(&self, out: &mut Vec<ObjRef>) {
        self.activity.object_roots(out);
        out.extend(self.roots.iter());
    }
}

/// A new activity's first step.
pub(crate) enum First {
    /// The send `~start` asked for.
    Send(Box<StartedSend>),
    /// The rest of a replied method body, already on the activity's record.
    Reply,
}

/// Every activity but the running one, and what the scheduler knows of them.
pub(crate) struct Activities {
    running: ActivityId,
    /// Indexed by [`ActivityId`]; `None` for the running activity and for a
    /// free handle.
    idle: Vec<Option<Box<Idle>>>,
    free: Vec<u32>,
    /// Woken and spawned activities, in arrival order.
    pub(crate) ready: VecDeque<ActivityId>,
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
    timers: FxHashMap<TimerId, Timer>,
    next_timer: u32,
    message_ids: FxHashMap<ObjRef, MessageId>,
    next_message: u32,
    /// The activity each `Message` object was last sent from
    /// (`MessageClass`'s `startActivity`, set by `dispatch`,
    /// `classes/MessageClass.cpp:432`), which `Message~halt` asks. Pruned
    /// of swept messages at each collection.
    senders: FxHashMap<ObjRef, ActivityId>,
    /// The parked activities waiting on each message, in the order they
    /// parked.
    waiters: FxHashMap<MessageId, Vec<ActivityId>>,
    /// The thread contexts of ended activities that made a native call or
    /// ran a package hook, which an extension may have kept and which last
    /// until interpreter termination.
    retired: Vec<rexx_api::ffi::ThreadContext>,
    /// The thread table every activity's context links.
    thread_table: rexx_api::ffi::ThreadTable,
    /// The numbers of ended activities a new one takes first, oldest first:
    /// the oracle's pool of idle threads (`availableActivities`,
    /// `concurrency/ActivityManager.cpp:556`), whose threads keep their
    /// numbers, including none.
    pooled: VecDeque<Option<u32>>,
    /// The last number assigned (`Activity::getIdntfr`'s counter).
    last_number: u32,
    /// The next [`crate::NativeFrame`] number, wrapping: one counter for
    /// every activity, so a frame token names its call on one of them.
    next_native: u32,
    /// Failures kept by [`Interp::keep_late_failure`].
    late_failures: Vec<Failure>,
    /// Native calls and blocking operations that have left their driver and
    /// whose completions have not been drained.
    in_flight: usize,
    /// The native frames of calls abandoned while they ran, by frame token
    /// ([`Interp::native_token`]), until their completions are drained.
    abandoned: Vec<(u64, crate::NativeFrame)>,
    /// Those of them whose run is below the running loop, under a callback,
    /// so that no loop above them can wait for their completions.
    runs_below: usize,
    /// The last token a blocking operation was posted under.
    next_block: u64,
    /// The tokens of blocking operations a halt abandoned, no longer in
    /// flight, until their ends are drained.
    abandoned_blocks: Vec<u64>,
    /// The guard locks, and what each parked activity waits on.
    pub(crate) guards: crate::guards::GuardTable,
    pub(crate) semaphores: crate::semaphores::Semaphores,
    /// The switches from one activity to another.
    switches: u64,
    /// Whether main's program has ended, so main waits for the others as an
    /// activity that has finished.
    main_finished: bool,
}

/// The oracle's `msecInADay`: a timer's wait is whole days of this, then a
/// remainder.
pub(crate) const TIMER_DAY: std::time::Duration = std::time::Duration::from_secs(86_400);

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
    /// Drops the sender records of messages `live` no longer answers for.
    pub(crate) fn prune_senders(&mut self, live: impl Fn(ObjRef) -> bool) {
        self.senders.retain(|message, _| live(*message));
    }

    /// The table of an interpreter whose activities' thread contexts link
    /// `thread_table`.
    pub(crate) fn new(thread_table: rexx_api::ffi::ThreadTable) -> Activities {
        Activities {
            running: MAIN,
            idle: vec![None],
            free: Vec::new(),
            ready: VecDeque::new(),
            sleepers: BinaryHeap::new(),
            next_sleeper: 0,
            owners: Vec::new(),
            set_aside: Vec::new(),
            timers: FxHashMap::default(),
            next_timer: 0,
            message_ids: FxHashMap::default(),
            next_message: 0,
            senders: FxHashMap::default(),
            waiters: FxHashMap::default(),
            retired: Vec::new(),
            thread_table,
            pooled: VecDeque::new(),
            last_number: 0,
            next_native: 0,
            late_failures: Vec::new(),
            in_flight: 0,
            abandoned: Vec::new(),
            runs_below: 0,
            next_block: 0,
            abandoned_blocks: Vec::new(),
            guards: crate::guards::GuardTable::default(),
            semaphores: crate::semaphores::Semaphores::default(),
            switches: 0,
            main_finished: false,
        }
    }

    /// The switches from one activity to another so far.
    pub(crate) fn switches(&self) -> u64 {
        self.switches
    }

    /// The handles the table has made, free ones included.
    pub(crate) fn handles(&self) -> usize {
        self.idle.len()
    }

    /// The running activity's offset in the table.
    pub(crate) fn running_index(&self) -> usize {
        self.running.index()
    }

    /// Puts `activity` at the back of the ready queue unless it is ready
    /// already: a second entry would end the next park it makes.
    fn make_ready(&mut self, activity: ActivityId) {
        if !self.ready.contains(&activity)
            && !self.set_aside.iter().any(|(set, _)| *set == activity)
        {
            self.ready.push_back(activity);
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
            idle.object_roots(out);
        }
        for (_, frame) in &self.abandoned {
            frame.object_roots(out);
        }
        for timer in self.timers.values() {
            out.extend(
                timer
                    .pending
                    .iter()
                    .flat_map(|(receiver, scope, _)| [*receiver, *scope]),
            );
            out.extend(
                timer
                    .waiters
                    .iter()
                    .flat_map(|waiter| [waiter.receiver, waiter.scope]),
            );
        }
        self.semaphores.object_roots(out);
    }

    /// The identity `message`'s waiters park under.
    pub(crate) fn message_id(&mut self, message: ObjRef) -> MessageId {
        let next = &mut self.next_message;
        *self.message_ids.entry(message).or_insert_with(|| {
            *next += 1;
            MessageId(*next)
        })
    }

    /// Whether a native call or a blocking operation is in flight.
    /// Holds the native frame of a call abandoned while it ran, named by the
    /// frame token `token`, until its completion is drained.
    pub(crate) fn keep_abandoned(&mut self, token: u64, frame: crate::NativeFrame) {
        self.abandoned.push((token, frame));
    }

    /// The native frame [`Activities::keep_abandoned`] holds for `token`.
    pub(crate) fn take_abandoned(&mut self, token: u64) -> Option<crate::NativeFrame> {
        let row = self.abandoned.iter().position(|(held, _)| *held == token)?;
        Some(self.abandoned.swap_remove(row).1)
    }

    pub(crate) fn in_flight(&self) -> bool {
        self.in_flight > 0
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
    /// Files `idle`, a record [`Interp::new_activity`] made, as ready.
    fn spawn(&mut self, idle: Box<Idle>) -> ActivityId;
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
    /// Stops every activity's driver for a collection (spec 2026-09-29
    /// section 5): with the one thread holding the baton running a driver,
    /// that is holding the baton.
    fn stop_the_world(&self);
    /// Runs the native call the running activity, parked for it, prepared,
    /// off the baton (spec 2026-09-29 P6-3): on a pool thread, which this
    /// thread lends the baton to while it enters the call, or, where the pool
    /// is off or no thread can be spawned, here on the baton. Its completion
    /// is posted.
    fn exit_for_native(&mut self);
    /// Runs `block`, a blocking operation of the running activity's that
    /// touches no island value, on a pool thread, which posts what it ended
    /// with under the token answered; `Err(block)` where the pool is off or
    /// no thread can be spawned.
    fn exit_for_block(
        &mut self,
        block: crate::command::Block,
    ) -> Result<u64, crate::command::Block>;
    /// Posts `completion`, of `activity`'s native call in the native frame
    /// `frame`, to `inbox` for the baton's holder to drain; it touches
    /// nothing else.
    fn post_completion(
        inbox: &crate::timer::Inbox<crate::timer::Posted>,
        activity: ActivityId,
        frame: u64,
        completion: rexx_api::invoke::Completion,
    ) where
        Self: Sized;
}

impl Scheduler for Interp {
    fn spawn(&mut self, idle: Box<Idle>) -> ActivityId {
        self.guards_go_live(None);
        // `Activity::setCallerStackFrameAsStringTable` numbers the spawner
        // (`concurrency/Activity.cpp:1206`).
        self.activity_number();
        let table = &mut self.activities;
        let id = match table.free.pop() {
            Some(index) => ActivityId(index),
            None => {
                table.idle.push(None);
                ActivityId((table.idle.len() - 1) as u32)
            }
        };
        table.idle[id.0 as usize] = Some(idle);
        table.ready.push_back(id);
        if self.sim.is_some() {
            self.sim_spawned(id.index());
        }
        id
    }

    fn run_until_park(&mut self) -> Result<bool, Failure> {
        let ended = match self.run_started() {
            Ok(Stopped::Parked) => {
                self.exit_if_awaited();
                return Ok(true);
            }
            Ok(Stopped::Ended) => self.run_ending_uninits(),
            Err(failure) => Err(failure),
        };
        self.release_ended_mutexes();
        ended.map(|()| false)
    }

    fn park(&mut self, reason: ParkReason) {
        let running = self.activities.running;
        let table = &mut self.activities;
        self.activity.woken_by_halt = false;
        self.activity.semaphore_wait = None;
        match reason {
            ParkReason::Guard(_) => {}
            ParkReason::GuardWhen => self.activity.when_parked = true,
            ParkReason::MessageResult(id) | ParkReason::MessageWait(id) => {
                table.waiters.entry(id).or_default().push(running);
                table
                    .guards
                    .set_waiting(running, crate::guards::Waiting::Message(id));
            }
            ParkReason::Sleep { deadline } => {
                table.next_sleeper += 1;
                table
                    .sleepers
                    .push(Reverse((deadline, table.next_sleeper, running)));
                self.activity.asleep = Some(table.next_sleeper);
            }
            ParkReason::Timer { deadline, cancel } => {
                table.next_sleeper += 1;
                let order = table.next_sleeper;
                table.sleepers.push(Reverse((deadline, order, running)));
                if let Some(timer) = table.timers.get_mut(&cancel)
                    && let Some((receiver, scope, days_end)) = timer.pending.take()
                {
                    timer.waiters.push(TimerWaiter {
                        activity: running,
                        order,
                        receiver,
                        scope,
                        days_end,
                    });
                }
            }
            ParkReason::Native | ParkReason::Block => {}
            ParkReason::Semaphore(wait) => {
                let order = wait.deadline.map(|deadline| {
                    table.next_sleeper += 1;
                    let order = table.next_sleeper;
                    table.sleepers.push(Reverse((deadline, order, running)));
                    order
                });
                self.activity.semaphore_wait = Some(order);
                if !matches!(wait.kind, crate::semaphores::WaitKind::Poll(_)) {
                    table.semaphores.enqueue(
                        wait.key,
                        crate::semaphores::Waiter {
                            activity: running,
                            order,
                        },
                    );
                }
            }
        }
    }

    fn unpark(&mut self, activity: ActivityId) {
        self.activities.guards.woken(activity);
        self.activities.make_ready(activity);
    }

    fn yield_at_slice(&mut self) {
        let running = self.activities.running;
        self.activities.ready.push_back(running);
    }

    fn stop_the_world(&self) {
        assert!(
            self.baton.held_here(),
            "a collection on a thread not holding the interpreter's baton"
        );
    }

    fn exit_for_native(&mut self) {
        if self.stress_collect {
            self.collect_now();
        }
        let park = self.activity.native_park.take();
        let Some(record) = self.activity.native_call.as_mut() else {
            self.activity.native_park = park;
            return;
        };
        let (mut call, frame, pending) = record.leave(park);
        #[cfg(test)]
        NATIVE_EXITS.with(|exits| exits.set(exits.get() + 1));
        self.activities.in_flight += 1;
        let thread = self.thread_context();
        let baton = crate::sync::Arc::clone(&self.baton);
        let inbox = self.timer.inbox();
        let activity = self.activities.running;
        if let Some(worker) = self.pool.reserve() {
            let to = worker.thread();
            let pooled = crate::dispatch::library::PooledCall::new(
                call,
                thread,
                frame,
                pending,
                activity,
                &baton,
                crate::sync::Arc::clone(&inbox),
            );
            let lent = crate::sync::Arc::clone(&baton);
            worker.run(Box::new(move || {
                pool::posting_panics(&inbox, &lent, || pooled.run());
            }));
            baton.lend(to, crate::island::Island::of(self));
            return;
        }
        pin_enter!(self, crate::pinning::PinKind::NativeApiCallback);
        self.activities.runs_below += 1;
        let watch = self.sim_watch_native();
        let pending = call.run(frame, pending, self, &baton, &thread, |completion| {
            Self::post_completion(&inbox, activity, frame, completion);
        });
        if let Some(watch) = watch {
            self.sim_end_watch(watch);
        }
        self.activities.runs_below -= 1;
        pin_leave!(self);
        if let Some(record) = self.activity.native_call.as_mut() {
            self.activity.native_park = record.back(call, pending);
        }
    }

    fn exit_for_block(
        &mut self,
        block: crate::command::Block,
    ) -> Result<u64, crate::command::Block> {
        let Some(worker) = self.pool.reserve() else {
            return Err(block);
        };
        self.activities.in_flight += 1;
        self.activities.next_block += 1;
        let token = self.activities.next_block;
        let inbox = self.timer.inbox();
        let baton = crate::sync::Arc::clone(&self.baton);
        let activity = self.activities.running;
        worker.run(Box::new(move || {
            pool::posting_panics(&inbox, &baton, || {
                let ended = crate::signal::unblocked(|| {
                    block.stream(&|error, bytes| {
                        inbox.post(Posted::Output { error, bytes });
                    })
                });
                inbox.post(Posted::Unblocked {
                    activity,
                    token,
                    ended,
                });
            });
        }));
        Ok(token)
    }

    fn post_completion(
        inbox: &crate::timer::Inbox<crate::timer::Posted>,
        activity: ActivityId,
        frame: u64,
        completion: rexx_api::invoke::Completion,
    ) {
        inbox.post(Posted::Completed(Completed {
            activity,
            frame,
            completion,
        }));
    }
}

impl Interp {
    /// A record for a new activity, with an empty arena of the running one's
    /// block size and the oldest pooled number.
    pub(crate) fn new_activity(&mut self) -> Box<Idle> {
        let mut activity = Activity::new();
        activity.number = self.activities.pooled.pop_front().flatten();
        #[cfg(feature = "sharing")]
        {
            activity.sharing_tag = self.heap.sharing_tag();
        }
        let mut roots = ActivityRoots::new();
        roots.set_frame_block(self.roots.activity().frames().size());
        Box::new(Idle { activity, roots })
    }

    /// The running activity's thread context, made at its first native call
    /// or package hook.
    pub(crate) fn thread_context(&mut self) -> rexx_api::ffi::ThreadContext {
        self.activity
            .thread
            .get_or_insert_with(|| {
                rexx_api::ffi::ThreadContext::linking(&self.activities.thread_table)
            })
            .clone()
    }

    /// The idle activity holding the activation whose `RexxContext` is
    /// `context`.
    pub(crate) fn idle_context_owner(&self, context: ObjRef) -> Option<ActivityId> {
        let holds = |idle: &Idle| {
            let activity = &idle.activity;
            activity
                .running
                .iter()
                .chain(&activity.suspended)
                .any(|activation| activation.context_object == Some(context))
        };
        let index = self
            .activities
            .idle
            .iter()
            .position(|idle| idle.as_deref().is_some_and(holds))?;
        Some(ActivityId(u32::try_from(index).expect("handles fit u32")))
    }

    /// Runs `read` with the idle activity `id` in the running one's place, as
    /// [`Interp::switch_to`] puts it there, and both back where they were
    /// after. `read` may allocate; it may not park, switch or run Rexx.
    pub(crate) fn with_idle_activity<R>(
        &mut self,
        id: ActivityId,
        read: impl FnOnce(&mut Interp) -> R,
    ) -> R {
        self.swap_idle(id);
        let frame = self.roots.activity_mut().push_frame();
        let before = self.idle_read_state();
        let answer = read(self);
        debug_assert!(
            self.idle_read_state() == before,
            "a read of an idle activity parked, switched or ran Rexx"
        );
        self.roots.activity_mut().pop_frame(frame);
        self.swap_idle(id);
        answer
    }

    /// What [`Interp::with_idle_activity`]'s `read` may not change: the
    /// running handle, the ready queue's length, the pin depth, the
    /// activation stack and the open driver constructs.
    fn idle_read_state(&self) -> (ActivityId, usize, u32, usize, Option<u64>, usize) {
        let activity = &self.activity;
        (
            self.activities.running,
            self.activities.ready.len(),
            activity.pin_depth,
            activity.suspended.len(),
            activity.running.as_ref().map(|running| running.id.0),
            activity.frames.len(),
        )
    }

    fn swap_idle(&mut self, id: ActivityId) {
        let Some(idle) = self.activities.idle[id.0 as usize].as_deref_mut() else {
            unreachable!("an idle handle names an idle activity");
        };
        std::mem::swap(&mut self.activity, &mut idle.activity);
        std::mem::swap(self.roots.activity_mut(), &mut idle.roots);
    }

    /// A number for a new native frame.
    pub(crate) fn next_native_id(&mut self) -> u32 {
        let id = self.activities.next_native;
        self.activities.next_native = id.wrapping_add(1);
        id
    }

    /// The idle activity whose native call the frame token `frame`
    /// ([`Interp::native_token`]) names, and that call's row.
    pub(crate) fn idle_native_owner(&self, frame: u64) -> Option<(ActivityId, usize)> {
        let row = usize::try_from(frame >> 32).ok()?;
        let id = frame & u64::from(u32::MAX);
        let index = self.activities.idle.iter().position(|idle| {
            idle.as_deref().is_some_and(|idle| {
                idle.activity
                    .native_handles
                    .get(row)
                    .is_some_and(|native| u64::from(native.id) == id)
            })
        })?;
        Some((
            ActivityId(u32::try_from(index).expect("handles fit u32")),
            row,
        ))
    }

    /// Whether the running activity has the native frame the frame token
    /// `frame` names.
    pub(crate) fn runs_native_frame(&self, frame: u64) -> bool {
        let id = frame & u64::from(u32::MAX);
        usize::try_from(frame >> 32)
            .ok()
            .and_then(|row| self.activity.native_handles.get(row))
            .is_some_and(|native| u64::from(native.id) == id)
    }

    /// `read` of the activity whose native call the frame token `frame`
    /// names: the running one as it is, or an idle one as
    /// [`Interp::with_idle_activity`] puts it in place.
    pub(crate) fn with_native_owner<R>(
        &mut self,
        frame: u64,
        read: impl FnOnce(&mut Interp) -> Option<R>,
    ) -> Option<R> {
        if self.runs_native_frame(frame) {
            return read(self);
        }
        let (owner, _) = self.idle_native_owner(frame)?;
        self.with_idle_activity(owner, read)
    }

    /// Whether an activity waits in the ready queue.
    pub(crate) fn any_ready(&self) -> bool {
        !self.activities.ready.is_empty()
    }

    /// How many activity slots exist besides main's: none before the first
    /// spawn.
    pub(crate) fn activities_spawned(&self) -> usize {
        self.activities.idle.len() - 1
    }

    /// Moves the oldest pooled number to the pool's tail, as a spawned
    /// activity that ends at once does.
    pub(crate) fn rotate_pooled(&mut self) {
        let pooled = &mut self.activities.pooled;
        let number = pooled.pop_front().flatten();
        if pooled.len() <= MAX_POOLED {
            pooled.push_back(number);
        }
    }

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

    /// The running activity's handle.
    pub(crate) fn running_activity(&self) -> ActivityId {
        self.activities.running
    }

    /// Whether the waits chained from `owner`'s lead back to the running
    /// activity (`Activity::checkDeadLock`, `concurrency/Activity.cpp:2000`):
    /// each waiting activity waits on a guard lock's owner or on the activity
    /// running a message's send.
    pub(crate) fn deadlocks(&self, owner: ActivityId) -> bool {
        let me = self.activities.running;
        let mut at = owner;
        for _ in 0..self.activities.idle.len() {
            let next = match self.activities.guards.waiting(at) {
                Some(crate::guards::Waiting::Guard(key)) => self.activities.guards.owner(key),
                Some(crate::guards::Waiting::Message(id)) => self
                    .activities
                    .message_ids
                    .iter()
                    .find(|(_, waited)| **waited == id)
                    .and_then(|(message, _)| self.message_runner(*message)),
                None => None,
            };
            match next {
                Some(next) if next == me => return true,
                Some(next) => at = next,
                None => return false,
            }
        }
        false
    }

    /// The activity running `message`'s send, started or sent, while it runs
    /// (`MessageClass::startActivity`).
    pub(crate) fn message_runner(&self, message: ObjRef) -> Option<ActivityId> {
        let runs = |activity: &Activity| {
            matches!(activity.root_then, Some(Then::Started(started)) if started == message)
                || activity
                    .native_tails
                    .iter()
                    .any(|tail| tail.records(message))
                || activity
                    .native_park
                    .as_ref()
                    .is_some_and(|park| park.records(message))
        };
        if runs(&self.activity) {
            return Some(self.activities.running);
        }
        let index = self
            .activities
            .idle
            .iter()
            .position(|idle| idle.as_deref().is_some_and(|idle| runs(&idle.activity)))?;
        Some(ActivityId(u32::try_from(index).expect("handles fit u32")))
    }

    /// Posts `watcher`'s guard semaphore (`Activity::guardPost`): its next
    /// `GUARD WHEN` wait ends at once, and one it is parked in ends now.
    pub(crate) fn post_guard(&mut self, watcher: ActivityId) {
        if watcher == self.activities.running {
            self.activity.guard_posted = true;
            return;
        }
        let Some(Some(idle)) = self.activities.idle.get_mut(watcher.0 as usize) else {
            return;
        };
        idle.activity.guard_posted = true;
        if std::mem::take(&mut idle.activity.when_parked) {
            self.unpark(watcher);
        }
    }

    /// Wakes every activity waiting on `message`, whose send has completed.
    pub(crate) fn message_completed(&mut self, message: ObjRef) {
        let Some(id) = self.activities.message_ids.remove(&message) else {
            return;
        };
        let mark = self.activities.ready.len();
        for waiter in self.activities.waiters.remove(&id).unwrap_or_default() {
            self.unpark(waiter);
        }
        if self.sim.is_some() {
            self.sim_order_event(mark);
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
        #[cfg(feature = "sharing")]
        self.heap.share_as(self.activity.sharing_tag);
        let outgoing = std::mem::replace(&mut table.running, next);
        table.switches += 1;
        if ended {
            table.retired.extend(idle.activity.thread.take());
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
        if self.activities.ready.is_empty() && self.activities.in_flight == 0 {
            self.timer.disarm();
        }
        if self.stress_collect {
            self.collect_now();
        }
        if self.sim.is_some() {
            self.sim_check_switch();
        }
    }

    /// The scheduler's invariants (spec 2026-10-07 section 4), which the
    /// simulation mode checks at every switch: this thread holds the baton;
    /// each activity is running, ready, parked or finished, and one only; a
    /// ready activity holds no park reason and a parked one has a wake
    /// source; the guard queues agree with the waits recorded. One pass over
    /// the table's queues marks each handle, then each handle is checked.
    pub(crate) fn check_invariants(&self) -> Result<(), Loud> {
        let fail = |what| Err(Loud::scheduler_inconsistency(what));
        #[cfg(test)]
        let held = self.baton.held_here() && !BATON_ELSEWHERE.with(std::cell::Cell::get);
        #[cfg(not(test))]
        let held = self.baton.held_here();
        if !held {
            return fail("a switch on a thread not holding the baton");
        }
        let table = &self.activities;
        let running = table.running;
        let filed =
            |activity: ActivityId| matches!(table.idle.get(activity.index()), Some(Some(_)));
        if filed(running) {
            return fail("a running activity filed as idle");
        }
        if let Some(what) = table
            .guards
            .inconsistency(|activity| activity == running || filed(activity))
        {
            return fail(what);
        }
        let handles = table.idle.len();
        // A sleeper, a message's waiter or a guard's waiter; a `GUARD WHEN`
        // is the other park reason, kept on the activity's own record.
        let mut parked = vec![false; handles];
        let sleepers = table
            .sleepers
            .iter()
            .map(|Reverse((_, _, sleeper))| *sleeper);
        let waiters = table.waiters.values().flatten().copied();
        for activity in sleepers.chain(waiters).chain(table.guards.waiters()) {
            if let Some(mark) = parked.get_mut(activity.index()) {
                *mark = true;
            }
        }
        let when_parked = |activity: ActivityId| {
            matches!(
                table.idle.get(activity.index()),
                Some(Some(idle)) if idle.activity.when_parked
            )
        };
        let mut ready = vec![false; handles];
        let readied = (table.ready.iter().copied())
            .chain(table.set_aside.iter().map(|(activity, _)| *activity));
        for activity in readied {
            if activity == running {
                return fail("an activity both running and ready");
            }
            if ready.get(activity.index()) == Some(&true) {
                return fail("an activity ready twice");
            }
            if !filed(activity) {
                return fail("a ready handle naming no idle activity");
            }
            if parked[activity.index()] || when_parked(activity) {
                return fail("a ready activity holding a park reason");
            }
            ready[activity.index()] = true;
        }
        let mut free = vec![false; handles];
        for &handle in &table.free {
            if let Some(mark) = free.get_mut(handle as usize) {
                *mark = true;
            }
        }
        for (index, slot) in table.idle.iter().enumerate() {
            let activity = ActivityId(u32::try_from(index).expect("handles fit u32"));
            match slot {
                _ if activity == running => {
                    if free[index] {
                        return fail("a running activity's handle free");
                    }
                }
                None if !free[index] => return fail("a handle neither free nor filed"),
                None => {}
                Some(_) if free[index] => return fail("a free handle naming an activity"),
                Some(idle) => {
                    let record = &idle.activity;
                    let finished =
                        record.root_end.is_some() || activity == MAIN && table.main_finished;
                    // A deadline, a post (a message's completion, a
                    // semaphore's post or release, a native call's or a
                    // command's end) or a guard (its release, a store a
                    // `GUARD WHEN` watches); a halt ends each of these.
                    let wakes = parked[index]
                        || record.when_parked
                        || table.semaphores.queues(activity)
                        || record.native_call.is_some()
                        || record.blocked.is_some();
                    if !ready[index] && !finished && !wakes {
                        return fail("a parked activity with no wake source");
                    }
                }
            }
        }
        Ok(())
    }

    /// Runs other activities until the running one, parked, is woken. The
    /// failure, where nothing left to run can wake it or another activity
    /// failed with something that is not a condition, is the one its wait
    /// answers.
    pub(crate) fn wait_until_woken(&mut self) -> Option<Failure> {
        #[cfg(test)]
        if self.activity.native_call.is_some() && std::mem::take(&mut self.fail_native_wait) {
            self.cancel_wait();
            return Some(Raised::insufficient_stack().into());
        }
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
        if self.sim_fails_wait() {
            pinned_unpark!(self);
            self.cancel_wait();
            return Some(Raised::insufficient_stack().into());
        }
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

    /// Withdraws the running activity from every message's waiters, from the
    /// guard queues, from the sleepers, from the timers and from the
    /// semaphores.
    fn cancel_wait(&mut self) {
        let running = self.activities.running;
        #[cfg(test)]
        self.note_cancelled_wait();
        self.activity.when_parked = false;
        self.activities.guards.withdraw(running);
        self.activity.guard_waits.retain(|wait| !wait.queued);
        self.activity.guarded_send = None;
        for waiters in self.activities.waiters.values_mut() {
            waiters.retain(|waiter| *waiter != running);
        }
        self.activities.semaphores.withdraw(running);
        self.activities.timers.retain(|_, timer| {
            let before = timer.waiters.len();
            timer.waiters.retain(|waiter| waiter.activity != running);
            !(timer.ends_with_wait && timer.waiters.len() < before)
        });
        self.activities
            .sleepers
            .retain(|Reverse((_, _, sleeper))| *sleeper != running);
    }

    /// Counts in [`CANCELLED_WAITS`] what [`Interp::cancel_wait`] finds to
    /// withdraw the running activity from.
    #[cfg(test)]
    fn note_cancelled_wait(&self) {
        let running = self.activities.running;
        let found = CancelledWaits {
            when_parks: u64::from(self.activity.when_parked),
            guard_waits: u64::from(self.activities.guards.queues(running)),
            sleeps: u64::from(
                (self.activities.sleepers.iter())
                    .any(|Reverse((_, _, sleeper))| *sleeper == running),
            ),
        };
        CANCELLED_WAITS.with(|cancelled| {
            let counts = cancelled.get();
            cancelled.set(CancelledWaits {
                when_parks: counts.when_parks + found.when_parks,
                guard_waits: counts.guard_waits + found.guard_waits,
                sleeps: counts.sleeps + found.sleeps,
            });
        });
    }

    /// Measures this interpreter's Rust stack from `base`, an address near
    /// the base of a thread stack of `bytes`.
    ///
    /// # Panics
    ///
    /// If `bytes` does not exceed the margin kept free below the deepest
    /// point.
    pub(crate) fn measure_stack(&mut self, base: usize, bytes: usize) {
        assert!(
            bytes > STACK_MARGIN,
            "a thread stack of {bytes} bytes leaves nothing beyond the {STACK_MARGIN}-byte margin"
        );
        self.stack_base = base;
        self.stack_room = bytes - STACK_MARGIN;
    }

    /// Whether `here`, an address on this thread's stack, lies deeper than
    /// the stack may reach: past it a call, a nested scheduler loop or an
    /// expression level raises 11.1 rather than risk the thread's end.
    #[inline(always)]
    pub(crate) fn stack_exhausted<T>(&self, here: *const T) -> bool {
        self.stack_base.abs_diff(here as usize) > self.stack_room
    }

    /// A new timer, not posted, which ends with a wait on it where
    /// `ends_with_wait`.
    pub(crate) fn create_timer(&mut self, ends_with_wait: bool) -> TimerId {
        let table = &mut self.activities;
        table.next_timer += 1;
        let id = TimerId(table.next_timer);
        let timer = Timer {
            ends_with_wait,
            ..Timer::default()
        };
        table.timers.insert(id, timer);
        id
    }

    /// Ends `id`, whose waits are over.
    pub(crate) fn remove_timer(&mut self, id: TimerId) {
        self.activities.timers.remove(&id);
    }

    /// Whether `id` is posted, or `None` for a timer that has ended.
    pub(crate) fn timer_posted(&self, id: TimerId) -> Option<bool> {
        self.activities.timers.get(&id).map(|timer| timer.posted)
    }

    /// Clears `id`'s post.
    pub(crate) fn reset_timer(&mut self, id: TimerId) {
        if let Some(timer) = self.activities.timers.get_mut(&id) {
            timer.posted = false;
        }
    }

    /// Records the wait on `id` about to park: the call's receiver and method
    /// scope, and when its whole days end.
    pub(crate) fn begin_timer_wait(
        &mut self,
        id: TimerId,
        receiver: ObjRef,
        scope: ObjRef,
        days_end: Instant,
    ) {
        if let Some(timer) = self.activities.timers.get_mut(&id) {
            timer.pending = Some((receiver, scope, days_end));
        }
    }

    /// Ends the running activity's wait on `id`: whether the timer is
    /// posted, and whether a post woke a cancelled waiter.
    pub(crate) fn end_timer_wait(&mut self, id: TimerId) -> (bool, bool) {
        let running = self.activities.running;
        let Some(timer) = self.activities.timers.get_mut(&id) else {
            return (false, false);
        };
        timer.waiters.retain(|waiter| waiter.activity != running);
        (timer.posted, timer.cancelled)
    }

    /// The receiver and scope of each wait on `id`, in park order.
    pub(crate) fn timer_waiters(&self, id: TimerId) -> Vec<(ObjRef, ObjRef)> {
        self.activities
            .timers
            .get(&id)
            .map_or_else(Vec::new, |timer| {
                timer
                    .waiters
                    .iter()
                    .map(|waiter| (waiter.receiver, waiter.scope))
                    .collect()
            })
    }

    /// Posts `id`, `cancelled` saying for each of [`Interp::timer_waiters`]
    /// whether its object's `CANCELED` is `.true`, and wakes every waiter. A
    /// waiter in its whole days whose object is not cancelled resets the post
    /// and has its current day end instead, as the oracle's day loop does.
    ///
    /// # Panics
    ///
    /// If `cancelled` does not hold one flag per waiter.
    pub(crate) fn post_timer(&mut self, id: TimerId, cancelled: &[bool]) {
        let now = self.now();
        let mark = self.activities.ready.len();
        self.post_timer_at(id, cancelled, now);
        if self.sim.is_some() {
            self.sim_order_event(mark);
        }
    }

    /// [`Interp::post_timer`] at `now`, before the order of the waiters it
    /// readies is settled.
    fn post_timer_at(&mut self, id: TimerId, cancelled: &[bool], now: Instant) {
        let table = &mut self.activities;
        let Some(timer) = table.timers.get_mut(&id) else {
            return;
        };
        assert_eq!(
            timer.waiters.len(),
            cancelled.len(),
            "a timer posted with a cancel flag per waiter"
        );
        let mut moved: Vec<(u64, Instant, Instant)> = Vec::new();
        let mut woken: Vec<u64> = Vec::new();
        let mut waiting = Vec::new();
        for (waiter, &cancelled) in std::mem::take(&mut timer.waiters)
            .into_iter()
            .zip(cancelled)
        {
            let end = waiter.days_end;
            if !cancelled && now < end {
                let begun = (end - now).as_nanos().div_ceil(TIMER_DAY.as_nanos());
                let left = u32::try_from(begun - 1).unwrap_or(u32::MAX);
                let ended = now + TIMER_DAY * left;
                moved.push((waiter.order, end, ended));
                waiting.push(TimerWaiter {
                    days_end: ended,
                    ..waiter
                });
            } else {
                timer.cancelled |= cancelled;
                woken.push(waiter.order);
                table.ready.extend(
                    table
                        .sleepers
                        .iter()
                        .find(|Reverse((_, parked, _))| *parked == waiter.order)
                        .map(|Reverse((_, _, activity))| *activity),
                );
            }
        }
        timer.posted = moved.is_empty();
        timer.waiters = waiting;
        let mut sleepers = std::mem::take(&mut table.sleepers).into_vec();
        sleepers.retain(|Reverse((_, parked, _))| !woken.contains(parked));
        for Reverse((deadline, parked, _)) in &mut sleepers {
            if let Some((_, end, ended)) = moved.iter().find(|(order, ..)| order == parked) {
                *deadline = *ended + (*deadline - *end);
            }
        }
        table.sleepers = BinaryHeap::from(sleepers);
    }

    /// Readies `waiter`, parked on a semaphore, where it is still parked,
    /// answering whether it was: an untimed one, or a timed one still asleep.
    pub(crate) fn wake_semaphore_waiter(&mut self, waiter: crate::semaphores::Waiter) -> bool {
        if let Some(order) = waiter.order {
            let table = &mut self.activities;
            let mut sleepers = std::mem::take(&mut table.sleepers).into_vec();
            let before = sleepers.len();
            sleepers.retain(|Reverse((_, parked, _))| *parked != order);
            let asleep = sleepers.len() < before;
            table.sleepers = BinaryHeap::from(sleepers);
            if !asleep {
                return false;
            }
        }
        self.unpark(waiter.activity);
        true
    }

    /// The driver exit of the running activity, where its driver has just
    /// parked it for a native call.
    fn exit_if_awaited(&mut self) {
        if self
            .activity
            .native_call
            .as_ref()
            .is_some_and(|call| call.awaits_exit())
        {
            self.exit_for_native();
        }
    }

    /// Files the completions posted since the last drain.
    pub(crate) fn drain_completions(&mut self) {
        let posted = self.timer.drain();
        self.file_completions(posted);
    }

    /// Hands each completion of `posted` to the parked activity that made its
    /// call and readies that activity, and lends the baton to each thread
    /// that recalls it. What follows a recall goes back to the inbox before
    /// the lend, where a loop the lendee nests can drain it.
    pub(crate) fn file_completions(&mut self, mut posted: VecDeque<Posted>) {
        if self.sim.is_some() {
            self.sim_screen_posts(&posted);
        }
        while let Some(next) = posted.pop_front() {
            match next {
                Posted::Completed(Completed {
                    activity,
                    frame,
                    completion,
                }) => {
                    self.activities.in_flight -= 1;
                    if let Some(call) = self
                        .record_of(activity)
                        .and_then(|record| record.native_call.as_mut())
                        .filter(|call| call.frame() == frame)
                    {
                        call.complete(completion);
                        self.unpark(activity);
                    } else {
                        self.end_abandoned_call(frame);
                    }
                }
                Posted::Recall(recall) => {
                    self.timer.requeue(std::mem::take(&mut posted));
                    self.serve_recall(recall);
                    posted = self.timer.drain();
                    if self.sim.is_some() {
                        self.sim_screen_posts(&posted);
                    }
                }
                Posted::Output {
                    error: false,
                    bytes,
                } => self.write_out(&bytes),
                Posted::Output { error: true, bytes } => self.write_err(&bytes),
                Posted::Panicked(payload) => std::panic::resume_unwind(payload),
                Posted::Halt => {
                    self.timer.requests().clear(crate::timer::HALT);
                    self.halt_all();
                }
                Posted::Input(chunk) => self.input.receive(chunk),
                Posted::Unblocked {
                    activity,
                    token,
                    ended,
                } => {
                    self.serve_signal_now();
                    let abandoned = &mut self.activities.abandoned_blocks;
                    if let Some(at) = abandoned.iter().position(|held| *held == token) {
                        abandoned.swap_remove(at);
                        continue;
                    }
                    self.activities.in_flight -= 1;
                    if let Some(blocked) = self
                        .record_of(activity)
                        .and_then(|record| record.blocked.as_mut())
                    {
                        blocked.end(ended);
                        self.unpark(activity);
                    }
                }
            }
        }
    }

    /// `activity`'s record, running or idle, or `None` for a handle no
    /// activity has.
    fn record_of(&mut self, activity: ActivityId) -> Option<&mut Activity> {
        if activity == self.activities.running {
            return Some(&mut self.activity);
        }
        self.activities
            .idle
            .get_mut(activity.0 as usize)
            .and_then(Option::as_mut)
            .map(|idle| &mut idle.activity)
    }

    /// Lends the baton to the thread `recall` names, with the activity it
    /// calls back for as the running one, its frames pinned, and the stack
    /// measured from that thread's: the callback or the call's end runs above
    /// the activity that ran last, whose frames stay below. A thread running
    /// no native call of this interpreter's gets no stack to run Rexx code
    /// on.
    fn serve_recall(&mut self, recall: Recall) {
        let Recall { by, thread, base } = recall;
        let outgoing = self.activities.running;
        let activity = match by {
            Recaller::Call { activity, frame } => self
                .activities
                .idle
                .get(activity.0 as usize)
                .and_then(Option::as_ref)
                .and_then(|idle| idle.activity.native_call.as_ref())
                .is_some_and(|call| call.frame() == frame)
                .then_some(activity),
            Recaller::Context(address) => self.context_owner(address),
        }
        .filter(|activity| *activity != outgoing);
        if let Some(activity) = activity {
            self.swap_running(activity, outgoing);
            self.activities.owners.push((outgoing, true));
        }
        pin_enter!(self, crate::pinning::PinKind::NativeApiCallback);
        self.activities.runs_below += 1;
        let stack = (self.stack_base, self.stack_room);
        self.stack_base = base.unwrap_or(self.stack_base);
        self.stack_room = match base {
            Some(_) => self.pool.stack().saturating_sub(STACK_MARGIN),
            None => 0,
        };
        let baton = crate::sync::Arc::clone(&self.baton);
        baton.lend(thread, crate::island::Island::of(self));
        (self.stack_base, self.stack_room) = stack;
        self.activities.runs_below -= 1;
        pin_leave!(self);
        if let Some(activity) = activity {
            self.activities.owners.pop();
            self.swap_running(outgoing, activity);
        }
    }

    /// The idle activity whose thread context is at `address`.
    fn context_owner(&self, address: usize) -> Option<ActivityId> {
        let index = self.activities.idle.iter().position(|idle| {
            idle.as_ref()
                .and_then(|idle| idle.activity.thread.as_ref())
                .is_some_and(|thread| thread.address() == address)
        })?;
        Some(ActivityId(u32::try_from(index).expect("handles fit u32")))
    }

    /// Makes the idle `next` the running activity in place of `running`,
    /// whose record goes under its own handle, as [`Interp::switch_to`] does
    /// without a switch's other work.
    fn swap_running(&mut self, next: ActivityId, running: ActivityId) {
        let table = &mut self.activities;
        let Some(mut idle) = table.idle[next.0 as usize].take() else {
            unreachable!("a recalled handle names an idle activity");
        };
        std::mem::swap(&mut self.activity, &mut idle.activity);
        std::mem::swap(self.roots.activity_mut(), &mut idle.roots);
        #[cfg(feature = "sharing")]
        self.heap.share_as(self.activity.sharing_tag);
        table.idle[running.0 as usize] = Some(idle);
        table.running = next;
    }

    /// Whether an activity other than the running one is alive.
    pub(crate) fn others_live(&self) -> bool {
        self.activities.idle.len() - 1 > self.activities.free.len()
    }

    /// Moves every sleeper whose deadline is due to the ready queue, in
    /// deadline order.
    fn wake_due_sleepers(&mut self) {
        let now = self.now();
        let table = &mut self.activities;
        while let Some(Reverse((deadline, _, sleeper))) = table.sleepers.peek().copied() {
            if deadline > now {
                break;
            }
            table.sleepers.pop();
            table.make_ready(sleeper);
        }
    }

    /// [`Interp::run_started_activities`] until it has nothing left to run or
    /// the run's deadline has passed, with each failure it answered on the
    /// way, in order.
    pub(crate) fn run_started_to_end(&mut self) -> Vec<(Failure, Vec<crate::FailureSite>)> {
        let mut failures = self.take_late_failures();
        while let Err(failure) = self.run_started_activities() {
            let last = matches!(failure, Failure::Deadline) || self.sim_is_stuck();
            failures.push((failure, Vec::new()));
            if last {
                break;
            }
        }
        failures
    }

    /// The `UNINIT`s a collection readied, run on an activity whose dispatch
    /// has ended (`Activity::runThread`, `concurrency/Activity.cpp:249`) or
    /// whose activation returns. The first refusal they meet is answered; the
    /// later ones are kept for the program's end, in order.
    pub(crate) fn run_ending_uninits(&mut self) -> Result<(), Failure> {
        let mut refused = self.run_ready_uninits().into_iter();
        let Some(first) = refused.next() else {
            return Ok(());
        };
        for loud in refused {
            self.keep_late_failure(loud.into());
        }
        Err(first.into())
    }

    /// The failures kept for the program's end, in order.
    pub(crate) fn take_late_failures(&mut self) -> Vec<(Failure, Vec<crate::FailureSite>)> {
        std::mem::take(&mut self.activities.late_failures)
            .into_iter()
            .map(|failure| (failure, Vec::new()))
            .collect()
    }

    /// A failure another activity answered main's root loop with after
    /// main's own end, kept for the program's end to report.
    fn keep_late_failure(&mut self, failure: Failure) {
        self.activities.late_failures.push(failure);
    }

    /// Runs every started activity to its end, for the end of the program.
    /// One that nothing left can wake keeps the program waiting, as the
    /// oracle's termination does (`InterpreterInstance::terminate`,
    /// `runtime/InterpreterInstance.cpp:562`), until the run's deadline or
    /// a signal's halt wakes it.
    pub(crate) fn run_started_activities(&mut self) -> Result<(), Failure> {
        self.activities.main_finished = true;
        self.cancel_wait();
        loop {
            self.run_others(false)?;
            let table = &self.activities;
            let me = table.running.0 as usize;
            let others =
                (table.idle.iter().enumerate()).any(|(index, idle)| index != me && idle.is_some());
            if !others {
                return Ok(());
            }
            if table.ready.is_empty()
                && table.sleepers.is_empty()
                && table.in_flight == table.runs_below
            {
                self.idle_for_good()?;
            }
        }
    }

    /// Idles this thread with nothing left that can wake an activity but a
    /// signal's halt; the run's deadline ends it.
    fn idle_for_good(&mut self) -> Result<(), Failure> {
        if self.sim.is_some() {
            return Err(self.sim_endless_wait());
        }
        while self.activities.ready.is_empty() {
            self.idle_until(self.now() + TIMER_DAY)?;
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
        if self.timer.requests().pending(crate::timer::INBOX) {
            self.drain_completions();
        }
        loop {
            if !self.activities.sleepers.is_empty() {
                self.wake_due_sleepers();
            }
            let next = if self.sim.is_some() {
                self.sim_pick()
            } else {
                self.activities.ready.pop_front()
            };
            let Some(ready) = next else {
                let Some(Reverse((due, _, _))) = self.activities.sleepers.peek().copied() else {
                    if self.activities.in_flight == self.activities.runs_below {
                        return Ok(None);
                    }
                    self.idle_for_posts()?;
                    continue;
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
        let sent = match self.activity.first.take() {
            Some(First::Send(send)) => {
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
            first => 'resumed: {
                // The oracle's resumed activation announces itself again
                // where it had before the `REPLY` (`RexxActivation.cpp:561`).
                if matches!(first, Some(First::Reply)) {
                    if !self.reserve_for_continuation() {
                        self.activity.first = first;
                        return Ok(Stopped::Parked);
                    }
                    self.open_replied_level();
                    self.trace_invocation_entry();
                } else if !self.activity.guard_waits.is_empty() {
                    self.take_granted_guard();
                }
                let driven = match self.activity.sliced.take() {
                    Some((floor, at)) => self.drive_from(DriveStart::Sliced { floor, at }, true),
                    None => {
                        if self.parked_again() {
                            return Ok(Stopped::Parked);
                        }
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
        let started = match then {
            Then::Started(message) => Some(message),
            _ => None,
        };
        let recorded = self.apply_then(then, sent);
        let failed = match (untrapped, recorded) {
            (Some(raised), _) => raised,
            // A notification of the send's completion failed, which the
            // message is told of as the send's own failure.
            (None, Err(Failure::Raised(raised))) => {
                self.yield_after_notifier_failure()?;
                if let Some(message) = started {
                    self.activity.failed_sends.push(message);
                    self.message_outcomes.insert(message, Some(raised.clone()));
                }
                raised
            }
            (None, Err(failure)) => return Err(failure),
            (None, Ok(_)) => return Ok(Stopped::Ended),
        };
        self.end_failed_started(started, failed)?;
        Ok(Stopped::Ended)
    }

    /// Settles and reports `failed`, the failure a started activity ended
    /// with, as the oracle's `Activity::run` handler and then the thread's own
    /// do (`concurrency/Activity.cpp:3423`, `:233`): the message `started`
    /// is told of it, and where a notifier fails, that failure replaces it,
    /// the message is told of the replacement and both are reported, with a
    /// second notifier failure reported too.
    fn end_failed_started(
        &mut self,
        started: Option<ObjRef>,
        failed: Box<Raised>,
    ) -> Result<(), Failure> {
        self.settle_failed_sends(&failed)?;
        let Some(message) = started else {
            self.report_started_failure(&failed);
            return Ok(());
        };
        let replaced = match self.notify_failed_send(message, false) {
            Ok(()) => {
                self.report_started_failure(&failed);
                return Ok(());
            }
            Err(Failure::Raised(replaced)) => replaced,
            Err(failure) => return Err(failure),
        };
        self.yield_after_notifier_failure()?;
        self.activity.failed_sends.push(message);
        self.message_outcomes
            .insert(message, Some(replaced.clone()));
        self.settle_failed_sends(&replaced)?;
        self.report_started_failure(&replaced);
        match self.notify_failed_send(message, false) {
            Ok(()) => Ok(()),
            Err(Failure::Raised(again)) => {
                self.report_started_failure(&again);
                Ok(())
            }
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
                if !self.parked_for_guard() {
                    self.yield_at_slice();
                }
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

    /// The re-test a woken wait makes before it resumes: `Some` reason to
    /// park again for.
    pub(crate) fn retest_wait(&mut self, reason: ParkReason) -> Option<ParkReason> {
        match reason {
            ParkReason::Semaphore(wait) => self.retest_semaphore(wait).map(ParkReason::Semaphore),
            _ => None,
        }
    }

    /// Parks the running activity, woken from a park its root driver
    /// recorded, again where its re-test says so, answering whether it did.
    fn parked_again(&mut self) -> bool {
        let Some(reason) = self.activity.native_park.as_ref().map(|park| park.reason()) else {
            return false;
        };
        let Some(again) = self.retest_wait(reason) else {
            return false;
        };
        if let Some(park) = self.activity.native_park.as_mut() {
            park.set_reason(again);
        }
        self.park(again);
        true
    }

    /// The running activity's park, resumed now that it has woken.
    fn resume_parked(&mut self, failure: Option<Failure>) -> Result<Option<ObjRef>, Failure> {
        let park = self.activity.native_park.take().or_else(|| {
            self.activity
                .native_call
                .as_mut()
                .and_then(|call| call.take_park())
        });
        match park {
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
        self.exit_if_awaited();
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
                failure if self.activity.root_end.is_none() => {
                    self.root_step(failure);
                    self.exit_if_awaited();
                }
                _ => {}
            }
        }
    }

    /// Main's root driver run from its recorded continuation, with `failure`
    /// as the answer of the wait it ends.
    fn root_step(&mut self, failure: Option<Failure>) {
        let driven = match (self.activity.sliced.take(), failure) {
            (Some(_), Some(failure)) => Err(failure),
            (Some((floor, at)), None) => {
                if !self.activity.guard_waits.is_empty() {
                    self.take_granted_guard();
                }
                self.drive_from(DriveStart::Sliced { floor, at }, true)
            }
            (None, None) if self.parked_again() => return,
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
                if !self.parked_for_guard() {
                    self.yield_at_slice();
                }
            }
            Ok(Driven::Ended(ended)) => self.activity.root_end = Some(Ok(ended)),
            Err(failure) => self.activity.root_end = Some(Err(failure)),
        }
    }

    /// The clause boundaries the switch mode has counted.
    pub(crate) fn switch_clauses(&self) -> u64 {
        self.switch.as_ref().map_or(0, |switch| switch.clauses)
    }

    /// Whether the running activation has replied and its split has yet to
    /// run.
    pub(crate) fn split_owed(&self) -> bool {
        self.running_activation()
            .and_then(|activation| activation.replied.as_ref())
            .is_some_and(|replied| replied.continuation.is_none())
    }

    /// Switches activities as `mode` says, and never on the timer's word.
    pub(crate) fn set_switch_mode(&mut self, mode: SwitchMode) {
        if let SwitchMode::Sim(config) = &mode {
            self.start_sim(config.clone());
        }
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
        if !self.activity.guard_waits.is_empty() {
            self.serve_guard_wait(yields)?;
        }
        if self.activity.splits_owed > 0 {
            if yields && self.split_owed() {
                return Err(Failure::Slice);
            }
            // A boundary closing a construct's branch, or a clause of a trap
            // handler the `REPLY` clause's end ran: the split waits for the
            // replier's next clause.
            self.clause_countdown = 1;
        }
        if self.stress_collect {
            self.collect_now();
        }
        if self.timer.requests().pending(crate::timer::INBOX) {
            self.drain_completions();
        }
        if !self.activities.sleepers.is_empty() {
            self.wake_due_sleepers();
        }
        if let Some(switch) = &mut self.switch {
            self.clause_countdown = 1;
            switch.clauses += 1;
            let due = match &switch.mode {
                SwitchMode::EveryOpportunity => true,
                SwitchMode::AtClause(clause) => switch.clauses == *clause,
                SwitchMode::Sim(_) => {
                    let clauses = switch.clauses;
                    self.sim_clause(clauses)?
                }
            };
            if due {
                self.timer.requests().set(SLICE);
            }
        } else if self.activities.ready.is_empty() && self.activities.in_flight == 0 {
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

    /// One round of the other ready activities, where a started activity
    /// ending holds no frame of its driver's: on the oracle the activity a
    /// notifier failed in gives up the kernel lock while the failure
    /// unwinds, and an activity the completion woke runs then. Its end runs
    /// on below the round, so a loop above sets it aside. Each such yield
    /// nests, so one made with less than half the stack's room left is not
    /// made, keeping room for the code of the activities ending below it.
    fn yield_after_notifier_failure(&mut self) -> Result<(), Failure> {
        let probe = 0u8;
        if !self.any_ready()
            || self.stack_base.abs_diff(&raw const probe as usize) > self.stack_room / 2
        {
            return Ok(());
        }
        self.yield_at_slice();
        self.run_round(true).map(|_| ())
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
    /// is 4.1. A trap held while its handler runs drops it: the oracle queues
    /// it on the handler's activation, which keeps it delayed until that
    /// activation ends.
    pub(crate) fn raise_requested_halt(
        &mut self,
        description: Option<Vec<u8>>,
    ) -> Result<(), Failure> {
        let held = self.trap_frame().is_some_and(|frame| {
            let traps = &frame.traps;
            traps
                .get(b"HALT".as_slice())
                .or_else(|| traps.get(b"ANY".as_slice()))
                .is_some_and(|trap| trap.delayed)
        });
        if held {
            return Ok(());
        }
        let raised = Raised {
            description: description.clone(),
            ..Raised::condition(std::borrow::Cow::Borrowed("HALT"))
        };
        match self.trap_for(b"HALT") {
            Some(trap) if trap.call => {
                let object = self.build_condition_object(&raised, Some(true))?;
                // In a forwarding activation, such as the `.INPUT` monitor's
                // `UNKNOWN` a halted read ran in, the trap is the caller's,
                // whose clause is still running and takes it at its end:
                // measured, `parse pull` halted runs the handler before the
                // next clause.
                let running = self.activation().id;
                let owner = self.trap_frame().map_or(running, |frame| frame.id);
                self.activity.pending_traps.push_back(PendingTrap {
                    condition: b"HALT".as_slice().into(),
                    rc: None,
                    description,
                    object: Some(object),
                    activation: owner,
                    queued_during_delivery: owner == running,
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
    /// false for a message never started or sent and for an activity already
    /// asked; true where the request is queued for the running activation of
    /// the activity that started or sent it (`RexxActivation::halt`), and
    /// where that activity has no Rexx frame to make it to (`Activity::halt`,
    /// `concurrency/Activity.cpp:2155`), which drops it.
    pub(crate) fn halt_message(&mut self, message: ObjRef, description: Option<Vec<u8>>) -> bool {
        if !self.started_messages.contains(&message) {
            let Some(&sender) = self.activities.senders.get(&message) else {
                return false;
            };
            let target = if sender == self.activities.running {
                Some(&mut self.activity)
            } else {
                match self.activities.idle.get_mut(sender.index()) {
                    Some(Some(idle)) => Some(&mut idle.activity),
                    _ => None,
                }
            };
            return match target {
                Some(target) => request_halt(target, description) != Some(false),
                None => true,
            };
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
        match target {
            Some(target) => request_halt(target, description) != Some(false),
            None => true,
        }
    }

    /// Records the running activity as `message`'s sender, which
    /// [`Interp::halt_message`] asks.
    pub(crate) fn record_message_sender(&mut self, message: ObjRef) {
        self.activities
            .senders
            .insert(message, self.activities.running);
    }

    /// A signal's halt (`InterpreterInstance::haltAllActivities`,
    /// `runtime/InterpreterInstance.cpp:686`): every activity with a Rexx
    /// frame is asked to raise `HALT` with no description, and one parked in
    /// `SysSleep`, a `GUARD WHEN`, a semaphore wait or a command's wait is
    /// woken to take it (rulings P59, P60), in handle order.
    pub(crate) fn halt_all(&mut self) {
        let mark = self.activities.ready.len();
        self.halt_each();
        if self.sim.is_some() {
            self.sim_order_event(mark);
        }
    }

    /// [`Interp::halt_all`] before the order of the activities it readies is
    /// settled.
    fn halt_each(&mut self) {
        let running = self.activities.running;
        for index in 0..self.activities.idle.len() {
            let activity = ActivityId(u32::try_from(index).expect("handles fit u32"));
            let record = if activity == running {
                &mut self.activity
            } else {
                match self.activities.idle.get_mut(index) {
                    Some(Some(idle)) => &mut idle.activity,
                    _ => continue,
                }
            };
            if request_halt(record, None) == Some(true) {
                self.wake_for_halt(activity);
            }
        }
    }

    /// Ends `activity`'s park, if it is in one a halt ends.
    fn wake_for_halt(&mut self, activity: ActivityId) {
        let table = &mut self.activities;
        let record = if activity == table.running {
            &mut self.activity
        } else {
            match table.idle.get_mut(activity.0 as usize) {
                Some(Some(idle)) => &mut idle.activity,
                _ => return,
            }
        };
        if let Some(order) = record.asleep.take() {
            let before = table.sleepers.len();
            table
                .sleepers
                .retain(|Reverse((_, sleeper, _))| *sleeper != order);
            if table.sleepers.len() < before {
                record.woken_by_halt = true;
                table.make_ready(activity);
            }
        } else if std::mem::take(&mut record.when_parked) {
            record.woken_by_halt = true;
            self.unpark(activity);
        } else if let Some(token) = record
            .blocked
            .as_mut()
            .and_then(|blocked| blocked.abandon())
        {
            table.in_flight -= 1;
            table.abandoned_blocks.push(token);
            self.unpark(activity);
        } else if let Some(order) = record.semaphore_wait.take() {
            let before = table.sleepers.len();
            if let Some(order) = order {
                table
                    .sleepers
                    .retain(|Reverse((_, sleeper, _))| *sleeper != order);
            }
            if table.sleepers.len() < before || table.semaphores.withdraw(activity) {
                record.woken_by_halt = true;
                self.unpark(activity);
            }
        }
    }

    /// Serves a signal's halt posted while this thread ran outside the clause
    /// loop, in a blocking operation that has returned: the halt is queued
    /// for the end of the running clause. Everything else posted waits for
    /// the next drain.
    #[inline(always)]
    pub(crate) fn serve_posted_halt(&mut self) {
        if self.timer.requests().pending(crate::timer::HALT) {
            self.serve_halt_now();
        }
    }

    /// Serves a signal whose handler has run and whose halt the timer thread
    /// has not posted yet: a terminal's Ctrl-C ends a command's child and
    /// signals this process at once, and the halt belongs to the command's
    /// clause.
    pub(crate) fn serve_signal_now(&mut self) {
        crate::timer::serve_signal();
        self.serve_posted_halt();
    }

    #[cold]
    #[inline(never)]
    fn serve_halt_now(&mut self) {
        self.timer.requests().clear(crate::timer::HALT);
        let mut posted = self.timer.drain();
        let before = posted.len();
        posted.retain(|post| !matches!(post, Posted::Halt));
        let halted = posted.len() < before;
        self.timer.requeue(posted);
        if halted {
            self.halt_all();
        }
    }

    /// Takes the running activation's halt request and raises it, for a
    /// `GUARD WHEN` a halt woke: `true` where one was taken.
    pub(crate) fn take_requested_halt(&mut self) -> Result<bool, Failure> {
        let here = self.activation().id;
        let Some(at) = self
            .activity
            .pending_traps
            .iter()
            .position(|pending| pending.request && pending.activation == here)
        else {
            return Ok(false);
        };
        let pending = self
            .activity
            .pending_traps
            .remove(at)
            .expect("position answered an index inside the queue");
        self.raise_requested_halt(pending.description)?;
        Ok(true)
    }
}

/// Queues a halt request for `target`'s running activation
/// (`RexxActivation::halt`): `None` where it has none, `Some(false)` where
/// one is already queued, else `Some(true)`.
fn request_halt(target: &mut Activity, description: Option<Vec<u8>>) -> Option<bool> {
    let activation = target.running.as_ref()?.id;
    if target.pending_traps.iter().any(|pending| pending.request) {
        return Some(false);
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
    Some(true)
}

#[cfg(test)]
thread_local! {
    /// How many driver exits for a native call this thread has made.
    static NATIVE_EXITS: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
    /// How many abandoned calls' frames the last run on this thread still
    /// held at its end.
    static ABANDONED_HELD: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    /// What the cancelled waits on this thread withdrew their activities
    /// from.
    static CANCELLED_WAITS: std::cell::Cell<CancelledWaits> =
        const { std::cell::Cell::new(CancelledWaits { when_parks: 0, guard_waits: 0, sleeps: 0 }) };
}

/// How many cancelled waits found their activity in each wait
/// [`Interp::cancel_wait`] withdraws it from.
#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CancelledWaits {
    /// A `GUARD WHEN` park.
    pub(crate) when_parks: u64,
    /// A guard lock's queue.
    pub(crate) guard_waits: u64,
    /// The sleepers.
    pub(crate) sleeps: u64,
}

/// [`CANCELLED_WAITS`]'s counts.
#[cfg(test)]
pub(crate) fn cancelled_waits() -> CancelledWaits {
    CANCELLED_WAITS.with(std::cell::Cell::get)
}

/// [`ABANDONED_HELD`]'s count.
#[cfg(test)]
pub(crate) fn abandoned_held() -> usize {
    ABANDONED_HELD.with(std::cell::Cell::get)
}

/// Sets [`ABANDONED_HELD`] from `activities`.
#[cfg(test)]
pub(crate) fn note_abandoned_held(activities: &Activities) {
    ABANDONED_HELD.with(|held| held.set(activities.abandoned.len()));
}

/// [`NATIVE_EXITS`]'s count.
#[cfg(test)]
pub(crate) fn native_exits() -> u64 {
    NATIVE_EXITS.with(std::cell::Cell::get)
}

/// An outcome [`crate::Interp::exec_instruction`] answers in place of running
/// its instruction.
#[cfg(test)]
#[derive(Clone, Copy, Debug)]
pub(crate) enum Scripted {
    /// `Park` for a sleep already due, without running the instruction.
    Park,
}

/// A state the next switch in the simulation mode breaks before it checks
/// the invariants, where the run has what it needs.
#[cfg(test)]
#[derive(Clone, Copy, Debug)]
pub(crate) enum Corruption {
    /// The first ready activity queued again.
    ReadyTwice,
    /// The running activity queued.
    RunningReady,
    /// The first ready activity recorded as a message's waiter.
    ReadyParked,
    /// A sleeper taken off the sleepers, readied by nothing.
    NoWakeSource,
    /// A guard waiter's wait record dropped.
    GuardQueue,
    /// The baton read as held by another thread.
    Baton,
    /// A handle past the table's end queued as ready.
    BogusReady,
    /// A handle neither free nor naming an activity.
    UnfiledHandle,
    /// The first ready activity's handle also freed.
    FreeFiled,
    /// The running activity's handle freed.
    RunningFree,
    /// The first ready activity's record moved into the running slot.
    RunningFiled,
    /// A lock's owner set to a handle naming no activity.
    GuardOwnerDead,
    /// A lock's first waiter queued again.
    GuardWaiterTwice,
    /// A lock's first waiter taken off its queue, its wait record kept.
    GuardWaitUnqueued,
    /// The baton released for the check, and taken again after it.
    BatonReleased,
}

#[cfg(test)]
thread_local! {
    pub(crate) static CORRUPTION: std::cell::Cell<Option<Corruption>> =
        const { std::cell::Cell::new(None) };
    static BATON_ELSEWHERE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
    static BATON_RELEASED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

#[cfg(test)]
impl Interp {
    /// Applies the [`Corruption`] this thread asked for, once the run has a
    /// state it applies to.
    pub(crate) fn sim_corrupt(&mut self) {
        let Some(corruption) = CORRUPTION.with(std::cell::Cell::take) else {
            return;
        };
        let table = &mut self.activities;
        let running = table.running;
        let first = table.ready.front().copied();
        let applied = match (corruption, first) {
            (Corruption::ReadyTwice, Some(first)) => {
                table.ready.push_back(first);
                true
            }
            (Corruption::RunningReady, _) => {
                table.ready.push_back(running);
                true
            }
            (Corruption::ReadyParked, Some(first)) => {
                table
                    .waiters
                    .entry(MessageId(u32::MAX))
                    .or_default()
                    .push(first);
                true
            }
            (Corruption::NoWakeSource, _) => {
                let sleeper = table
                    .sleepers
                    .iter()
                    .map(|Reverse((_, _, sleeper))| *sleeper)
                    .find(|sleeper| *sleeper != running);
                if let Some(sleeper) = sleeper {
                    table
                        .sleepers
                        .retain(|Reverse((_, _, other))| *other != sleeper);
                }
                sleeper.is_some()
            }
            (Corruption::GuardQueue, _) => match table.guards.first_queued() {
                Some(waiter) => {
                    table.guards.woken(waiter);
                    true
                }
                None => false,
            },
            (Corruption::Baton, _) => {
                BATON_ELSEWHERE.with(|elsewhere| elsewhere.set(true));
                true
            }
            (Corruption::BogusReady, _) => {
                table.ready.push_back(ActivityId(9999));
                true
            }
            (Corruption::UnfiledHandle, _) => {
                table.idle.push(None);
                true
            }
            (Corruption::FreeFiled, Some(first)) => {
                table.free.push(first.0);
                true
            }
            (Corruption::RunningFree, _) => {
                table.free.push(running.0);
                true
            }
            (Corruption::RunningFiled, Some(first)) => {
                let moved = table.idle[first.index()].take();
                table.idle[running.index()] = moved;
                true
            }
            (Corruption::GuardOwnerDead, _) => table.guards.corrupt_owner(ActivityId(9999)),
            (Corruption::GuardWaiterTwice, _) => table.guards.corrupt_queue_twice(),
            (Corruption::GuardWaitUnqueued, _) => table.guards.corrupt_unqueue(),
            (Corruption::BatonReleased, _) => {
                self.baton.release();
                BATON_RELEASED.with(|released| released.set(true));
                true
            }
            (
                Corruption::ReadyTwice
                | Corruption::ReadyParked
                | Corruption::FreeFiled
                | Corruption::RunningFiled,
                None,
            ) => false,
        };
        if !applied {
            CORRUPTION.with(|asked| asked.set(Some(corruption)));
        }
    }

    /// Takes the baton back where [`Corruption::BatonReleased`] released it.
    pub(crate) fn sim_uncorrupt(&self) {
        if BATON_RELEASED.with(std::cell::Cell::take) {
            self.baton.acquire();
        }
    }
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

mod pool;
pub(crate) use pool::{POOL_BOUND, POOL_STACK_BYTES, Pool, posting_panics};
#[cfg(test)]
pub(crate) use pool::{threads_beyond_bound, threads_spawned};

#[cfg(test)]
#[expect(clippy::disallowed_methods, reason = "these tests time real runs")]
mod tests;
