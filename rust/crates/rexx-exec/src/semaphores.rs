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

//! The state of `MutexSemaphore` and `EventSemaphore` instances and of the
//! unnamed RexxUtil semaphores, and the activities waiting on each. A post or
//! release readies waiters, and each re-tests its condition when it resumes,
//! as the oracle's wait loops do.

use std::time::{Duration, Instant};

use rexx_core::ObjRef;
use rustc_hash::FxHashMap;

use crate::Interp;
use crate::scheduler::ActivityId;

/// What a semaphore wait waits on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) enum SemaphoreKey {
    /// A `MutexSemaphore` or `EventSemaphore` instance.
    Object(ObjRef),
    /// An unnamed `Sys*Sem` handle.
    Handle(u64),
}

/// How a semaphore wait re-tests its condition when it resumes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WaitKind {
    /// `EventSemaphore~wait`: a timed one re-tests the post, as
    /// `SysSemaphore::wait(t)` loops (`common/platform/unix/SysSemaphore.cpp:298`);
    /// an untimed one does not (`:250`).
    Event,
    /// `MutexSemaphore~acquire`: takes the lock where it is free.
    Mutex,
    /// An untimed `Sys*Sem` wait (`sem_wait`): takes a unit where there is one.
    Counting,
    /// A timed `Sys*Sem` wait, with this many polls left: it tries the
    /// semaphore at each poll and a post never wakes it.
    Poll(u32),
}

/// A semaphore wait about to park.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SemaphoreWait {
    pub(crate) key: SemaphoreKey,
    pub(crate) kind: WaitKind,
    /// When a timed wait ends, or a polling one next polls.
    pub(crate) deadline: Option<Instant>,
}

/// How often a timed `Sys*Sem` wait tries its semaphore
/// (`SEM_WAIT_PERIOD`, `platform/unix/SysRexxUtil.cpp:800`).
pub(crate) const SEM_POLL: Duration = Duration::from_millis(100);

/// One semaphore's state: an event's post, a counting semaphore's value, or a
/// mutex's lock (`classes/MutexSemaphore.cpp`).
#[derive(Default)]
struct Semaphore {
    posted: bool,
    count: u64,
    /// The activity holding the lock. `None` with `depth` above zero is a
    /// lock an ended activity kept, which nothing releases.
    owner: Option<ActivityId>,
    /// The recursive pthread mutex's lock count.
    depth: u32,
    /// `MutexSemaphoreClass::nestCount`.
    nest: u32,
    /// Whether `MutexSemaphoreClass::close` has run.
    closed: bool,
    waiters: Vec<Waiter>,
}

/// One activity parked on a semaphore.
#[derive(Clone, Copy)]
pub(crate) struct Waiter {
    pub(crate) activity: ActivityId,
    /// A timed wait's sleeper order.
    pub(crate) order: Option<u64>,
}

/// Every semaphore with state, per interpreter.
#[derive(Default)]
pub(crate) struct Semaphores {
    table: FxHashMap<SemaphoreKey, Semaphore>,
    /// The queue each parked waiter is on.
    parked: FxHashMap<ActivityId, SemaphoreKey>,
    /// The mutexes each activity holds.
    held: FxHashMap<ActivityId, Vec<SemaphoreKey>>,
    /// The answer each re-test that ended a wait gave, until its resume reads
    /// it.
    answers: FxHashMap<ActivityId, bool>,
    handles: u64,
}

/// The first unnamed `Sys*Sem` handle. The oracle's are `malloc` addresses
/// (`platform/unix/SysRexxUtil.cpp:676`), wider than a whole number at the
/// default digits; these are too.
const FIRST_HANDLE: u64 = 1 << 40;

impl Semaphores {
    /// Files `waiter` on `key`'s queue; a closed handle has none.
    pub(crate) fn enqueue(&mut self, key: SemaphoreKey, waiter: Waiter) {
        let semaphore = match key {
            SemaphoreKey::Object(_) => self.table.entry(key).or_default(),
            SemaphoreKey::Handle(_) => match self.table.get_mut(&key) {
                Some(semaphore) => semaphore,
                None => return,
            },
        };
        semaphore.waiters.push(waiter);
        self.parked.insert(waiter.activity, key);
    }

    /// Withdraws `activity` from the queue it is on, and drops an answer no
    /// resume read; answers whether it was on one.
    pub(crate) fn withdraw(&mut self, activity: ActivityId) -> bool {
        self.answers.remove(&activity);
        let Some(key) = self.parked.remove(&activity) else {
            return false;
        };
        if let Some(semaphore) = self.table.get_mut(&key) {
            semaphore
                .waiters
                .retain(|waiter| waiter.activity != activity);
        }
        true
    }

    /// Appends each mutex semaphore an activity holds, as the oracle's
    /// `heldMutexes` marks it.
    pub(crate) fn object_roots(&self, out: &mut Vec<ObjRef>) {
        out.extend(self.held.values().flatten().filter_map(|key| match key {
            SemaphoreKey::Object(object) => Some(*object),
            SemaphoreKey::Handle(_) => None,
        }));
    }

    /// Records that `owner` now holds the mutex `key`.
    fn hold(&mut self, key: SemaphoreKey, owner: ActivityId) {
        self.held.entry(owner).or_default().push(key);
    }

    /// Records that `owner` no longer holds the mutex `key`.
    fn unhold(&mut self, key: SemaphoreKey, owner: ActivityId) {
        if let Some(keys) = self.held.get_mut(&owner) {
            keys.retain(|held| *held != key);
            if keys.is_empty() {
                self.held.remove(&owner);
            }
        }
    }

    /// Drops the state of every instance `live` says the collector freed.
    pub(crate) fn prune(&mut self, live: impl Fn(ObjRef) -> bool) {
        self.table.retain(|key, _| match key {
            SemaphoreKey::Object(object) => live(*object),
            SemaphoreKey::Handle(_) => true,
        });
    }

    /// Whether `activity` is queued on a semaphore.
    pub(crate) fn queues(&self, activity: ActivityId) -> bool {
        self.parked.contains_key(&activity)
    }

    fn take_waiters(&mut self, key: SemaphoreKey) -> Vec<Waiter> {
        self.table
            .get_mut(&key)
            .map(|semaphore| std::mem::take(&mut semaphore.waiters))
            .unwrap_or_default()
    }

    fn put_waiters(&mut self, key: SemaphoreKey, mut waiters: Vec<Waiter>) {
        if let Some(semaphore) = self.table.get_mut(&key) {
            waiters.append(&mut semaphore.waiters);
            semaphore.waiters = waiters;
        }
    }
}

/// What a request for a mutex semaphore's lock answers before any wait.
pub(crate) enum Request {
    Acquired,
    Refused,
    Wait,
}

impl Interp {
    /// Readies the waiters on `key` still parked, every one where `all`,
    /// else the first.
    fn wake_waiters(&mut self, key: SemaphoreKey, all: bool) {
        let mark = self.activities.ready.len();
        let mut waiters = self.activities.semaphores.take_waiters(key);
        let mut woken = false;
        waiters.retain(|waiter| {
            if woken && !all || !self.wake_semaphore_waiter(*waiter) {
                return true;
            }
            self.activities.semaphores.parked.remove(&waiter.activity);
            woken = true;
            false
        });
        self.activities.semaphores.put_waiters(key, waiters);
        if self.sim.is_some() {
            self.sim_order_event(mark);
        }
    }

    /// The re-test a woken wait makes when it resumes: `Some` wait to park
    /// again, else `None` with the wait's answer kept for its resume.
    pub(crate) fn retest_semaphore(&mut self, wait: SemaphoreWait) -> Option<SemaphoreWait> {
        let me = self.running_activity();
        self.activities.semaphores.withdraw(me);
        // A halt withdrew the wait: an untimed `Sys*Sem` wait answers `0`, as
        // `sem_wait` interrupted does (`platform/unix/SysRexxUtil.cpp:836-850`),
        // and every other wait answers that it took nothing.
        if std::mem::take(&mut self.activity.woken_by_halt) {
            let answer = wait.kind == WaitKind::Counting;
            self.activities.semaphores.answers.insert(me, answer);
            return None;
        }
        let now = self.now();
        let expired = wait.deadline.is_some_and(|deadline| now >= deadline);
        let answer = match (wait.kind, wait.key) {
            (WaitKind::Event, SemaphoreKey::Object(object)) => {
                if wait.deadline.is_none() || self.event_posted(object) {
                    true
                } else if expired {
                    false
                } else {
                    return Some(wait);
                }
            }
            (WaitKind::Mutex, SemaphoreKey::Object(object)) => {
                match self.request_mutex(object, false) {
                    Request::Acquired => true,
                    Request::Refused => false,
                    Request::Wait if expired => false,
                    Request::Wait => return Some(wait),
                }
            }
            (WaitKind::Counting, SemaphoreKey::Handle(handle)) => {
                if self.take_counting(handle) != Some(true) {
                    return Some(wait);
                }
                true
            }
            (WaitKind::Poll(0), _) => false,
            (WaitKind::Poll(left), SemaphoreKey::Handle(handle)) => {
                if self.take_counting(handle) != Some(true) {
                    return Some(SemaphoreWait {
                        kind: WaitKind::Poll(left - 1),
                        deadline: wait.deadline.map(|deadline| deadline + SEM_POLL),
                        ..wait
                    });
                }
                true
            }
            _ => unreachable!("a wait's kind and key are made together"),
        };
        self.activities.semaphores.answers.insert(me, answer);
        None
    }

    /// The answer the running activity's last re-test ended its wait with.
    pub(crate) fn take_semaphore_answer(&mut self) -> bool {
        let me = self.running_activity();
        self.activities
            .semaphores
            .answers
            .remove(&me)
            .unwrap_or(false)
    }

    /// Whether the event semaphore `object` is posted.
    pub(crate) fn event_posted(&self, object: ObjRef) -> bool {
        let key = SemaphoreKey::Object(object);
        self.activities
            .semaphores
            .table
            .get(&key)
            .is_some_and(|semaphore| semaphore.posted)
    }

    /// Posts the event semaphore `object` and readies every waiter
    /// (`SysSemaphore::post`, a broadcast).
    pub(crate) fn post_event(&mut self, object: ObjRef) {
        let key = SemaphoreKey::Object(object);
        self.activities
            .semaphores
            .table
            .entry(key)
            .or_default()
            .posted = true;
        self.wake_waiters(key, true);
    }

    /// Clears the event semaphore `object`'s post.
    pub(crate) fn reset_event(&mut self, object: ObjRef) {
        if let Some(semaphore) = self
            .activities
            .semaphores
            .table
            .get_mut(&SemaphoreKey::Object(object))
        {
            semaphore.posted = false;
        }
    }

    /// `MutexSemaphoreClass::request` before any wait: the running activity
    /// takes the lock where it is free or its own; `immediate` is a timeout
    /// of zero, which never waits, and only a trylock reaches a closed one.
    pub(crate) fn request_mutex(&mut self, object: ObjRef, immediate: bool) -> Request {
        let me = self.running_activity();
        let semaphore = self
            .activities
            .semaphores
            .table
            .entry(SemaphoreKey::Object(object))
            .or_default();
        if semaphore.closed && !immediate {
            return Request::Refused;
        }
        if semaphore.owner == Some(me) || semaphore.depth == 0 {
            let taken = semaphore.depth == 0;
            semaphore.owner = Some(me);
            semaphore.depth += 1;
            semaphore.nest += 1;
            if taken {
                self.activities
                    .semaphores
                    .hold(SemaphoreKey::Object(object), me);
            }
            return Request::Acquired;
        }
        if immediate {
            Request::Refused
        } else {
            Request::Wait
        }
    }

    /// `MutexSemaphoreClass::release`: `false` where the lock is not held at
    /// all or not by the running activity.
    pub(crate) fn release_mutex(&mut self, object: ObjRef) -> bool {
        let me = self.running_activity();
        let key = SemaphoreKey::Object(object);
        let Some(semaphore) = self.activities.semaphores.table.get_mut(&key) else {
            return false;
        };
        if semaphore.nest == 0 || semaphore.owner != Some(me) {
            return false;
        }
        semaphore.nest -= 1;
        semaphore.depth -= 1;
        if semaphore.depth == 0 {
            semaphore.owner = None;
            self.activities.semaphores.unhold(key, me);
            self.wake_waiters(key, false);
        }
        true
    }

    /// `MutexSemaphoreClass::close`: the nesting goes and only a trylock can
    /// take the lock again, while a lock it held stays held.
    pub(crate) fn close_mutex(&mut self, object: ObjRef) {
        let semaphore = self
            .activities
            .semaphores
            .table
            .entry(SemaphoreKey::Object(object))
            .or_default();
        semaphore.closed = true;
        semaphore.nest = 0;
    }

    /// `Activity::cleanupMutexes` for the running activity, which has ended:
    /// each lock it holds is released once per nesting level.
    pub(crate) fn release_ended_mutexes(&mut self) {
        let me = self.running_activity();
        let mut freed = Vec::new();
        let held = self.activities.semaphores.held.remove(&me);
        for key in held.into_iter().flatten() {
            let Some(semaphore) = self.activities.semaphores.table.get_mut(&key) else {
                continue;
            };
            semaphore.depth -= semaphore.nest.min(semaphore.depth);
            semaphore.nest = 0;
            semaphore.owner = None;
            if semaphore.depth == 0 {
                freed.push(key);
            }
        }
        for key in freed {
            self.wake_waiters(key, false);
        }
    }

    /// A new unnamed counting semaphore of value `count`, by its handle.
    pub(crate) fn create_counting(&mut self, count: u64) -> u64 {
        let semaphores = &mut self.activities.semaphores;
        let handle = FIRST_HANDLE + semaphores.handles * 16;
        semaphores.handles += 1;
        semaphores.table.insert(
            SemaphoreKey::Handle(handle),
            Semaphore {
                count,
                ..Semaphore::default()
            },
        );
        handle
    }

    /// The value of the counting semaphore `handle`, or `None` for a handle
    /// no create made or one closed since.
    pub(crate) fn counting_value(&self, handle: u64) -> Option<u64> {
        let key = SemaphoreKey::Handle(handle);
        self.activities
            .semaphores
            .table
            .get(&key)
            .map(|semaphore| semaphore.count)
    }

    /// `sem_post` on `handle`: the value grows and the first untimed waiter
    /// is readied. `false` for an unknown handle.
    pub(crate) fn post_counting(&mut self, handle: u64) -> bool {
        let key = SemaphoreKey::Handle(handle);
        let Some(semaphore) = self.activities.semaphores.table.get_mut(&key) else {
            return false;
        };
        semaphore.count += 1;
        self.wake_waiters(key, false);
        true
    }

    /// `sem_init(handle, 1, 0)`: the value back to zero.
    pub(crate) fn reset_counting(&mut self, handle: u64) {
        if let Some(semaphore) = self
            .activities
            .semaphores
            .table
            .get_mut(&SemaphoreKey::Handle(handle))
        {
            semaphore.count = 0;
        }
    }

    /// `sem_trywait` on `handle`: whether a unit was taken, or `None` for an
    /// unknown handle.
    pub(crate) fn take_counting(&mut self, handle: u64) -> Option<bool> {
        let key = SemaphoreKey::Handle(handle);
        let semaphore = self.activities.semaphores.table.get_mut(&key)?;
        let taken = semaphore.count > 0;
        if taken {
            semaphore.count -= 1;
        }
        Some(taken)
    }

    /// Ends `handle`: `false` where it is unknown. Its waiters stay parked.
    pub(crate) fn close_counting(&mut self, handle: u64) -> bool {
        self.activities
            .semaphores
            .table
            .remove(&SemaphoreKey::Handle(handle))
            .is_some()
    }
}

#[cfg(test)]
mod tests;
