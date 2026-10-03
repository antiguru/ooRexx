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
//! release hands the semaphore to a waiter directly.

use std::time::Instant;

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

/// A semaphore wait about to park.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SemaphoreWait {
    pub(crate) key: SemaphoreKey,
    /// A timed wait's end, and the last instant a post or release still ends
    /// it early.
    pub(crate) timed: Option<(Instant, Instant)>,
}

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
    /// A timed wait's sleeper order and the last instant a post ends it.
    pub(crate) timed: Option<(u64, Instant)>,
}

/// Every semaphore with state, per interpreter.
#[derive(Default)]
pub(crate) struct Semaphores {
    table: FxHashMap<SemaphoreKey, Semaphore>,
    handles: u64,
}

/// The first unnamed `Sys*Sem` handle. The oracle's are `malloc` addresses
/// (`platform/unix/SysRexxUtil.cpp:676`), wider than a whole number at the
/// default digits; these are too.
const FIRST_HANDLE: u64 = 1 << 40;

impl Semaphores {
    /// Files `waiter` on `key`'s queue.
    pub(crate) fn enqueue(&mut self, key: SemaphoreKey, waiter: Waiter) {
        self.table.entry(key).or_default().waiters.push(waiter);
    }

    /// Withdraws `activity` from every queue.
    pub(crate) fn withdraw(&mut self, activity: ActivityId) {
        for semaphore in self.table.values_mut() {
            semaphore
                .waiters
                .retain(|waiter| waiter.activity != activity);
        }
    }

    /// Appends each mutex semaphore an activity holds, as the oracle's
    /// `heldMutexes` marks it.
    pub(crate) fn object_roots(&self, out: &mut Vec<ObjRef>) {
        out.extend(self.table.iter().filter_map(|(key, semaphore)| match key {
            SemaphoreKey::Object(object) if semaphore.owner.is_some() => Some(*object),
            _ => None,
        }));
    }

    /// Drops the state of every instance `live` says the collector freed.
    pub(crate) fn prune(&mut self, live: impl Fn(ObjRef) -> bool) {
        self.table.retain(|key, _| match key {
            SemaphoreKey::Object(object) => live(*object),
            SemaphoreKey::Handle(_) => true,
        });
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

    /// Whether the running activity's wait on `key` was ended by a post or
    /// release rather than its deadline; it leaves the queue either way.
    fn end_wait(&mut self, key: SemaphoreKey, me: ActivityId) -> bool {
        let Some(semaphore) = self.table.get_mut(&key) else {
            return false;
        };
        let before = semaphore.waiters.len();
        semaphore.waiters.retain(|waiter| waiter.activity != me);
        semaphore.waiters.len() == before
    }
}

/// What a request for a mutex semaphore's lock answers before any wait.
pub(crate) enum Request {
    Acquired,
    Refused,
    Wait,
}

impl Interp {
    /// Wakes the first waiter on `key` a post can still end, answering it.
    fn hand_off(&mut self, key: SemaphoreKey) -> Option<ActivityId> {
        let mut waiters = self.activities.semaphores.take_waiters(key);
        let taker = waiters
            .iter()
            .position(|waiter| self.wake_semaphore_waiter(*waiter))
            .map(|index| waiters.remove(index).activity);
        self.activities.semaphores.put_waiters(key, waiters);
        taker
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

    /// Posts the event semaphore `object` and wakes every waiter a post can
    /// still end (`SysSemaphore::post`, a broadcast).
    pub(crate) fn post_event(&mut self, object: ObjRef) {
        let key = SemaphoreKey::Object(object);
        self.activities
            .semaphores
            .table
            .entry(key)
            .or_default()
            .posted = true;
        let mut waiters = self.activities.semaphores.take_waiters(key);
        waiters.retain(|waiter| !self.wake_semaphore_waiter(*waiter));
        self.activities.semaphores.put_waiters(key, waiters);
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

    /// Ends the running activity's wait on `key`: whether a post or release
    /// ended it.
    pub(crate) fn end_semaphore_wait(&mut self, key: SemaphoreKey) -> bool {
        let me = self.running_activity();
        self.activities.semaphores.end_wait(key, me)
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
            semaphore.owner = Some(me);
            semaphore.depth += 1;
            semaphore.nest += 1;
            return Request::Acquired;
        }
        if immediate {
            Request::Refused
        } else {
            Request::Wait
        }
    }

    /// Ends the running activity's wait for `object`'s lock: whether a
    /// release handed it over.
    pub(crate) fn end_mutex_wait(&mut self, object: ObjRef) -> bool {
        let me = self.running_activity();
        let key = SemaphoreKey::Object(object);
        self.activities.semaphores.end_wait(key, me);
        self.activities
            .semaphores
            .table
            .get(&key)
            .is_some_and(|semaphore| semaphore.owner == Some(me))
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
            self.pass_mutex(key);
        }
        true
    }

    /// Gives the free lock of `key` to its first waiter a release can still
    /// end.
    fn pass_mutex(&mut self, key: SemaphoreKey) {
        let Some(taker) = self.hand_off(key) else {
            return;
        };
        if let Some(semaphore) = self.activities.semaphores.table.get_mut(&key) {
            semaphore.owner = Some(taker);
            semaphore.depth = 1;
            semaphore.nest += 1;
        }
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
        for (key, semaphore) in &mut self.activities.semaphores.table {
            if semaphore.owner != Some(me) {
                continue;
            }
            semaphore.depth -= semaphore.nest.min(semaphore.depth);
            semaphore.nest = 0;
            semaphore.owner = None;
            if semaphore.depth == 0 {
                freed.push(*key);
            }
        }
        for key in freed {
            self.pass_mutex(key);
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

    /// `sem_post` on `handle`: its first waiter a post can still end takes
    /// the unit, else the value grows. `false` for an unknown handle.
    pub(crate) fn post_counting(&mut self, handle: u64) -> bool {
        let key = SemaphoreKey::Handle(handle);
        if self.counting_value(handle).is_none() {
            return false;
        }
        if self.hand_off(key).is_none()
            && let Some(semaphore) = self.activities.semaphores.table.get_mut(&key)
        {
            semaphore.count += 1;
        }
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
