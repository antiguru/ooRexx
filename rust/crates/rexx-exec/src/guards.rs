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

//! The guard locks: per object and method scope, the owning activity, its
//! nesting count and the activities waiting, served FIFO
//! (`VariableDictionary::reserve`/`release`/`transfer`,
//! `execution/VariableDictionary.cpp:522-617`).

use std::collections::VecDeque;
use std::collections::hash_map::Entry;

use rexx_core::ObjRef;
use rustc_hash::FxHashMap;

use crate::Interp;
use crate::activation::{Activation, ActivationId};
use crate::error::{Failure, Raised};
use crate::scheduler::{ActivityId, MessageId, ParkReason, Scheduler};

/// One object's variable pool for one scope: what a guard lock covers.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct GuardKey {
    pub(crate) object: ObjRef,
    pub(crate) scope: ObjRef,
}

/// What a parked activity waits on, which the deadlock check follows
/// (`Activity::waitingObject`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Waiting {
    Guard(GuardKey),
    Message(MessageId),
}

struct Lock {
    owner: ActivityId,
    count: u32,
    waiters: VecDeque<ActivityId>,
}

/// The outcome of [`GuardTable::try_reserve`].
pub(crate) enum Reserve {
    /// Taken, or nested one level deeper by its owner.
    Held,
    /// Owned by this other activity; nothing was queued.
    Contended(ActivityId),
}

/// Every lock held or waited for, and the variable pools numbered for a
/// `TraceObject`'s `ATTRIBUTEPOOL`.
#[derive(Default)]
pub(crate) struct GuardTable {
    /// Whether a second activity has ever existed. Until then no lock can be
    /// contended, the table holds nothing, and a method activation's own flag
    /// is the whole of its lock.
    live: bool,
    locks: FxHashMap<GuardKey, Lock>,
    waiting: FxHashMap<ActivityId, Waiting>,
    pools: FxHashMap<GuardKey, u32>,
    last_pool: u32,
    /// Each watched variable's watchers, by owning object, in the order they
    /// began watching. An entry stays once made (`RexxVariable::dependents`).
    watches: FxHashMap<ObjRef, Vec<Watch>>,
}

/// One variable a `GUARD WHEN` has watched, in one scope's pool.
struct Watch {
    scope: ObjRef,
    name: Box<[u8]>,
    watchers: Vec<ActivityId>,
}

impl GuardTable {
    /// Takes `key` for `me`, or nests it where `me` owns it.
    pub(crate) fn try_reserve(&mut self, key: GuardKey, me: ActivityId) -> Reserve {
        match self.locks.entry(key) {
            Entry::Vacant(vacant) => {
                vacant.insert(Lock {
                    owner: me,
                    count: 1,
                    waiters: VecDeque::new(),
                });
                Reserve::Held
            }
            Entry::Occupied(mut held) if held.get().owner == me => {
                held.get_mut().count += 1;
                Reserve::Held
            }
            Entry::Occupied(held) => Reserve::Contended(held.get().owner),
        }
    }

    /// Whether a second activity has ever existed.
    #[inline]
    pub(crate) fn is_live(&self) -> bool {
        self.live
    }

    /// Records one more level of `key` held by `owner`, as a lock held before
    /// the table went live.
    fn hold(&mut self, key: GuardKey, owner: ActivityId) {
        self.locks
            .entry(key)
            .or_insert(Lock {
                owner,
                count: 0,
                waiters: VecDeque::new(),
            })
            .count += 1;
    }

    /// Queues `me` behind `key`'s owner and records the wait.
    pub(crate) fn enqueue(&mut self, key: GuardKey, me: ActivityId) {
        if let Some(lock) = self.locks.get_mut(&key) {
            lock.waiters.push_back(me);
        }
        self.waiting.insert(me, Waiting::Guard(key));
    }

    /// Releases one level of `key`. At the last, the first waiter becomes
    /// the owner at count 1, and is answered for waking.
    pub(crate) fn release(&mut self, key: GuardKey) -> Option<ActivityId> {
        let Entry::Occupied(mut held) = self.locks.entry(key) else {
            return None;
        };
        let lock = held.get_mut();
        lock.count -= 1;
        if lock.count > 0 {
            return None;
        }
        match lock.waiters.pop_front() {
            Some(next) => {
                lock.owner = next;
                lock.count = 1;
                Some(next)
            }
            None => {
                held.remove();
                None
            }
        }
    }

    /// Moves `key`, held at count 1, to `to`.
    pub(crate) fn transfer(&mut self, key: GuardKey, to: ActivityId) {
        if let Some(lock) = self.locks.get_mut(&key) {
            lock.owner = to;
        }
    }

    pub(crate) fn owner(&self, key: GuardKey) -> Option<ActivityId> {
        self.locks.get(&key).map(|lock| lock.owner)
    }

    /// `key`'s nesting count, 0 where nobody holds it.
    pub(crate) fn count(&self, key: GuardKey) -> u32 {
        self.locks.get(&key).map_or(0, |lock| lock.count)
    }

    pub(crate) fn set_waiting(&mut self, activity: ActivityId, waiting: Waiting) {
        self.waiting.insert(activity, waiting);
    }

    pub(crate) fn waiting(&self, activity: ActivityId) -> Option<Waiting> {
        self.waiting.get(&activity).copied()
    }

    /// The first way the locks and the waits recorded disagree, where
    /// `alive` answers which handles name an activity.
    pub(crate) fn inconsistency(&self, alive: impl Fn(ActivityId) -> bool) -> Option<&'static str> {
        for (key, lock) in &self.locks {
            if !alive(lock.owner) {
                return Some("a guard held by no activity");
            }
            for (at, waiter) in lock.waiters.iter().enumerate() {
                if *waiter == lock.owner || lock.waiters.iter().take(at).any(|w| w == waiter) {
                    return Some("a guard waiter queued twice or behind itself");
                }
                if self.waiting.get(waiter) != Some(&Waiting::Guard(*key)) {
                    return Some("a guard waiter with no wait recorded for its guard");
                }
            }
        }
        let queued = |activity: &ActivityId, key: &GuardKey| {
            self.locks
                .get(key)
                .is_some_and(|lock| lock.waiters.contains(activity))
        };
        for (activity, waiting) in &self.waiting {
            if let Waiting::Guard(key) = waiting
                && !queued(activity, key)
            {
                return Some("a guard wait missing from its guard's queue");
            }
        }
        None
    }

    /// The first activity queued for a guard, if any.
    #[cfg(test)]
    pub(crate) fn first_queued(&self) -> Option<ActivityId> {
        self.locks
            .values()
            .find_map(|lock| lock.waiters.front().copied())
    }

    /// Ends `activity`'s wait: its record, and its place in a lock's queue.
    pub(crate) fn withdraw(&mut self, activity: ActivityId) {
        if let Some(Waiting::Guard(key)) = self.waiting.remove(&activity)
            && let Some(lock) = self.locks.get_mut(&key)
        {
            lock.waiters.retain(|waiter| *waiter != activity);
        }
    }

    /// Forgets `activity`'s wait record, its queue place already given up.
    pub(crate) fn woken(&mut self, activity: ActivityId) {
        self.waiting.remove(&activity);
    }

    /// `key`'s pool number, assigned from 1 when first asked
    /// (`VariableDictionary::getIdntfr`).
    pub(crate) fn pool_number(&mut self, key: GuardKey) -> u32 {
        let last = &mut self.last_pool;
        *self.pools.entry(key).or_insert_with(|| {
            *last += 1;
            *last
        })
    }

    /// Drops the pool numbers and watches of objects `live` no longer
    /// answers for.
    pub(crate) fn prune_pools(&mut self, live: impl Fn(ObjRef) -> bool) {
        self.pools.retain(|key, _| live(key.object));
        self.watches.retain(|owner, _| live(*owner));
    }

    /// Adds `watcher` to the watchers of `name` in `scope`'s pool on
    /// `owner`, once (`RexxVariable::inform`).
    pub(crate) fn watch(&mut self, owner: ObjRef, scope: ObjRef, name: &[u8], watcher: ActivityId) {
        let watches = self.watches.entry(owner).or_default();
        let at =
            match (watches.iter()).position(|watch| watch.scope == scope && *watch.name == *name) {
                Some(at) => at,
                None => {
                    watches.push(Watch {
                        scope,
                        name: name.into(),
                        watchers: Vec::new(),
                    });
                    watches.len() - 1
                }
            };
        let watchers = &mut watches[at].watchers;
        if !watchers.contains(&watcher) {
            watchers.push(watcher);
        }
    }

    /// Removes `watcher` from the watchers of `name` in `scope`'s pool on
    /// `owner` (`RexxVariable::uninform`).
    pub(crate) fn unwatch(
        &mut self,
        owner: ObjRef,
        scope: ObjRef,
        name: &[u8],
        watcher: ActivityId,
    ) {
        if let Some(watch) = (self.watches.get_mut(&owner).into_iter().flatten())
            .find(|watch| watch.scope == scope && *watch.name == *name)
        {
            watch.watchers.retain(|seen| *seen != watcher);
        }
    }

    /// The watchers of `name` in `scope`'s pool on `owner`.
    pub(crate) fn watchers(&self, owner: ObjRef, scope: ObjRef, name: &[u8]) -> &[ActivityId] {
        (self.watches.get(&owner).into_iter().flatten())
            .find(|watch| watch.scope == scope && *watch.name == *name)
            .map_or(&[], |watch| watch.watchers.as_slice())
    }
}

/// What a `REPLY` does with its activation's guard lock.
pub(crate) enum Transfer {
    None,
    /// Moves this lock to the continuation once it has a handle.
    Moved(GuardKey),
    /// Leaves the continuation this reserve to make.
    Again(GuardWait),
}

/// A method's wait to reserve its guard lock, served at its first clause
/// boundary.
#[derive(Clone, Copy)]
pub(crate) struct GuardWait {
    pub(crate) key: GuardKey,
    /// The waiting method's activation.
    pub(crate) activation: ActivationId,
    /// Whether the activity is queued behind the owner.
    pub(crate) queued: bool,
    /// Whether the reserve waits for the method's `>I>`, traced first.
    pub(crate) after_entry: bool,
}

/// A `GUARD` instruction that parked, kept on its activity until its op runs
/// it again.
pub(crate) struct GuardExec {
    pub(crate) activation: ActivationId,
    /// The instruction's index in its body.
    pub(crate) index: usize,
    pub(crate) wait: GuardExecWait,
    /// The variables its `WHEN` watches.
    pub(crate) watched: Vec<crate::activation::InstanceVar>,
}

/// What a parked [`GuardExec`] waits for.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum GuardExecWait {
    /// The lock, which a release grants it.
    Reserve,
    /// A change to a watched variable, after which it reserves the lock again
    /// where it held it (`RexxActivation::guardWait`,
    /// `execution/RexxActivation.cpp:3350-3372`).
    When { reacquire: bool },
}

/// Whether a method the directive `kind` declares is guarded: every
/// `::METHOD` and `::ATTRIBUTE` without `UNGUARDED`, and no `::CONSTANT`
/// (`parser/DirectiveParser.cpp:2525`).
#[inline]
pub(crate) fn directive_guarded(kind: &rexx_parse::DirectiveKind) -> bool {
    use rexx_parse::{DirectiveKind, GuardOption};
    match kind {
        DirectiveKind::Method(method) => method.guard != GuardOption::Unguarded,
        DirectiveKind::Attribute(attribute) => attribute.guard != GuardOption::Unguarded,
        _ => false,
    }
}

impl GuardKey {
    /// The lock a method activation of `activation` reserves.
    pub(crate) fn of(activation: &Activation) -> Option<GuardKey> {
        activation
            .method_identity
            .as_ref()
            .map(|identity| GuardKey {
                object: identity.receiver,
                scope: identity.scope,
            })
    }
}

impl Interp {
    /// Takes `key` for the running activity, or nests it: from the table once
    /// a second activity has existed, at once before.
    pub(crate) fn take_guard(&mut self, key: GuardKey) -> Reserve {
        if !self.activities.guards.live {
            return Reserve::Held;
        }
        let me = self.running_activity();
        self.activities.guards.try_reserve(key, me)
    }

    /// Puts into the table the locks the running activity's method
    /// activations hold, and `moving`'s, which a `REPLY` has popped, the
    /// first time a second activity is about to exist.
    pub(crate) fn guards_go_live(&mut self, moving: Option<&Activation>) {
        if self.activities.guards.live {
            return;
        }
        self.activities.guards.live = true;
        let me = self.running_activity();
        let activity = &self.activity;
        let held: Vec<GuardKey> = (activity.suspended.iter().map(|a| &**a))
            .chain(activity.running.as_deref())
            .chain(moving)
            .filter(|activation| activation.flags.reserved())
            .filter_map(GuardKey::of)
            .collect();
        for key in held {
            self.activities.guards.hold(key, me);
        }
    }

    /// `key`'s nesting count: the table's, or before it is live the running
    /// activity's activations holding it.
    pub(crate) fn guard_count(&self, key: GuardKey) -> u32 {
        if self.activities.guards.live {
            return self.activities.guards.count(key);
        }
        let activity = &self.activity;
        let held = (activity.suspended.iter().map(|a| &**a))
            .chain(activity.running.as_deref())
            .filter(|activation| {
                activation.flags.reserved() && GuardKey::of(activation) == Some(key)
            })
            .count();
        u32::try_from(held).unwrap_or(u32::MAX)
    }

    /// Reserves `key` for the method activation just pushed, or leaves the
    /// reserve to a clause boundary of the method: its first, where another
    /// activity owns the lock, and the one after its `>I>` where that is
    /// traced early (`RexxActivation::run`, `execution/RexxActivation.cpp:523-532`).
    #[cold]
    #[inline(never)]
    pub(crate) fn reserve_for_method(&mut self, key: GuardKey) {
        let after_entry = self.activation().plan.traces_entry_early;
        if !after_entry && let Reserve::Held = self.take_guard(key) {
            self.activation_mut().flags.set_reserved(true);
            return;
        }
        let activation = self.activation().id;
        self.activity.guard_waits.push(GuardWait {
            key,
            activation,
            queued: false,
            after_entry,
        });
        self.clause_countdown = 1;
    }

    /// The running method's pending reserve, at a countdown visit in its own
    /// activation: taken, or 98.905 where the waits it would join lead back
    /// to this activity, or a wait. The wait ends the slice at a clause that
    /// `yields` with nothing pinning the activity, and is a pinned wait
    /// anywhere else.
    #[cold]
    #[inline(never)]
    pub(crate) fn serve_guard_wait(&mut self, yields: bool) -> Result<(), Failure> {
        let Some(running) = self.running_activation() else {
            return Ok(());
        };
        let (id, entry) = (running.id, running.trace_entry);
        let Some(at) = (self.activity.guard_waits.iter())
            .position(|wait| wait.activation == id && !wait.queued)
        else {
            self.clause_countdown = 1;
            return Ok(());
        };
        let wait = self.activity.guard_waits[at];
        if wait.after_entry && entry == crate::activation::TraceEntry::Pending {
            self.clause_countdown = 1;
            return Ok(());
        }
        self.activity.guard_waits.remove(at);
        let owner = match self.take_guard(wait.key) {
            Reserve::Held => {
                self.activation_mut().flags.set_reserved(true);
                return Ok(());
            }
            Reserve::Contended(owner) => owner,
        };
        if self.deadlocks(owner) {
            self.blame_first_clause();
            return Err(Raised::deadlock().into());
        }
        let me = self.running_activity();
        self.activities.guards.enqueue(wait.key, me);
        if yields && self.activity.pin_depth == 0 {
            self.activity.guard_waits.push(GuardWait {
                queued: true,
                ..wait
            });
            // The visit ends here, before the switch mode's count reloads
            // the countdown, so whatever runs next visits its first clause.
            self.clause_countdown = 1;
            return Err(Failure::Slice);
        }
        let failure = self.pinned_wait(ParkReason::Guard(wait.key));
        if failure.is_none() || self.granted(wait.key) {
            self.activation_mut().flags.set_reserved(true);
        }
        failure.map_or(Ok(()), Err)
    }

    /// Blames the running activation's first clause for a failure raised
    /// before it runs, as the oracle's reserve fails with that clause
    /// current.
    fn blame_first_clause(&mut self) {
        let Some(activation) = self.running_activation() else {
            return;
        };
        let program = std::rc::Rc::clone(&activation.program);
        let plan = std::rc::Rc::clone(&activation.plan);
        let (selector, index) = (activation.body, activation.pc);
        let Some(body) = crate::activation::body_of(&program, selector) else {
            return;
        };
        let Some(instruction) = body.instructions.get(index) else {
            return;
        };
        let code = crate::Code {
            body,
            symbols: &program.symbols,
            slots: &plan.by_symbol,
            plan: Some(&plan),
        };
        self.record_failure_site(&code, index, Some(&program.source), instruction);
    }

    /// Whether the running activity is parked for a guard lock, which a
    /// release readies it from.
    pub(crate) fn parked_for_guard(&self) -> bool {
        self.activity.guard_waits.iter().any(|wait| wait.queued)
    }

    /// The guard lock a release granted the running activity while it was
    /// parked for it, recorded on its waiting method, the running one.
    pub(crate) fn take_granted_guard(&mut self) {
        let Some(at) = self
            .activity
            .guard_waits
            .iter()
            .position(|wait| wait.queued)
        else {
            return;
        };
        let wait = self.activity.guard_waits.remove(at);
        debug_assert_eq!(
            self.activities.guards.owner(wait.key),
            Some(self.running_activity()),
            "a guard waiter resumed without the lock"
        );
        debug_assert_eq!(
            self.running_activation().map(|running| running.id),
            Some(wait.activation),
            "a guard waiter resumed in another activation"
        );
        if let Some(running) = self.activity.running.as_deref_mut() {
            running.flags.set_reserved(true);
        }
    }

    /// Whether a release has made the running activity `key`'s owner, which
    /// a wait that then failed still has to give back.
    pub(crate) fn granted(&self, key: GuardKey) -> bool {
        self.activities.guards.owner(key) == Some(self.running_activity())
    }

    /// Releases one level of `key`, waking the waiter it passes to.
    pub(crate) fn release_guard(&mut self, key: GuardKey) {
        if !self.activities.guards.live {
            return;
        }
        if let Some(next) = self.activities.guards.release(key) {
            self.unpark(next);
        }
    }

    /// The running method activation's end, unless a `REPLY` moves it on:
    /// its guard lock released, and a reserve still pending dropped
    /// (`RexxActivation::termination`'s `guardOff`). A `<I<` runs it first,
    /// as the oracle releases before tracing the exit.
    pub(crate) fn guard_off_at_end(&mut self) {
        let Some(mut ending) = self.activity.running.take() else {
            return;
        };
        self.guard_off_ended(&mut ending);
        self.activity.running = Some(ending);
    }

    /// [`Interp::guard_off_at_end`] for `ended`.
    fn guard_off_ended(&mut self, ended: &mut Activation) {
        if !self.activity.guard_waits.is_empty() {
            let id = ended.id;
            self.activity
                .guard_waits
                .retain(|wait| wait.activation != id);
        }
        if ended.flags.reserved() && !Self::moves_on(ended) {
            ended.flags.set_reserved(false);
            if self.activities.guards.is_live()
                && let Some(key) = GuardKey::of(ended)
            {
                self.release_guard(key);
            }
        }
    }

    /// Whether `ending` replied and its continuation takes it.
    fn moves_on(ending: &Activation) -> bool {
        ending
            .replied
            .as_ref()
            .is_some_and(|replied| replied.continuation.is_some())
    }

    /// `GUARD OFF`'s release of one level of the running method's lock,
    /// where it holds it (`RexxActivation::guardOff`).
    pub(crate) fn guard_off(&mut self, key: GuardKey) {
        if self.activation().flags.reserved() {
            self.activation_mut().flags.set_reserved(false);
            self.release_guard(key);
        }
    }

    /// `GUARD ON`'s reserve of the running method's lock
    /// (`RexxActivation::guardOn`, `execution/RexxActivation.cpp:2193-2207`):
    /// `false` where another activity owns it and this one is queued behind
    /// it, or 98.905 where the waits it would join lead back here.
    pub(crate) fn guard_on(&mut self, key: GuardKey) -> Result<bool, Failure> {
        if self.activation().flags.reserved() {
            return Ok(true);
        }
        if let Reserve::Contended(owner) = self.take_guard(key) {
            if self.deadlocks(owner) {
                return Err(Raised::deadlock().into());
            }
            let me = self.running_activity();
            self.activities.guards.enqueue(key, me);
            return Ok(false);
        }
        self.activation_mut().flags.set_reserved(true);
        Ok(true)
    }

    /// Registers the running activity as a watcher of each of `names` the
    /// running activation exposes (`RexxVariable::inform`), and answers them.
    pub(crate) fn watch_guard_variables(
        &mut self,
        names: &[Box<[u8]>],
    ) -> Vec<crate::activation::InstanceVar> {
        let exposed = &self.activation().exposed;
        let watched: Vec<crate::activation::InstanceVar> = names
            .iter()
            .filter_map(|name| {
                exposed
                    .iter()
                    .find(|(_, var)| var.name == *name)
                    .map(|(_, var)| var.clone())
            })
            .collect();
        for var in &watched {
            self.watch_variable(var.owner, var.scope, &var.name);
        }
        watched
    }

    /// Registers the running activity as a watcher of `name` in `scope`'s
    /// pool on `owner`, and marks `owner` as watched for the store barrier.
    pub(crate) fn watch_variable(&mut self, owner: ObjRef, scope: ObjRef, name: &[u8]) {
        let me = self.running_activity();
        if let Some(object) = self.heap.get_mut(owner) {
            object.mark_watched();
        }
        self.activities.guards.watch(owner, scope, name, me);
    }

    /// Withdraws the running activity from the watchers of `name` in
    /// `scope`'s pool on `owner`.
    pub(crate) fn unwatch_variable(&mut self, owner: ObjRef, scope: ObjRef, name: &[u8]) {
        let me = self.running_activity();
        self.activities.guards.unwatch(owner, scope, name, me);
    }

    /// The running activation's parked `GUARD` at instruction `index`, taken
    /// to run on.
    pub(crate) fn take_guard_exec(&mut self, index: usize) -> Option<GuardExec> {
        let id = self.running_activation()?.id;
        let parked = self.activity.guard_exec.take()?;
        if parked.activation == id && parked.index == index {
            return Some(*parked);
        }
        debug_assert!(false, "a parked GUARD outlived its instruction");
        self.end_guard_exec(*parked);
        None
    }

    /// The end of a `GUARD`'s run: its watches withdrawn
    /// (`RexxVariable::uninform`).
    pub(crate) fn end_guard_exec(&mut self, exec: GuardExec) {
        for var in &exec.watched {
            self.unwatch_variable(var.owner, var.scope, &var.name);
        }
    }

    /// The parked `GUARD` a failed wait ends: its watches withdrawn, and a
    /// lock a release granted it kept, for the method's end to release.
    #[cold]
    pub(crate) fn abandon_guard_exec(&mut self) {
        let Some(parked) = self.activity.guard_exec.take() else {
            return;
        };
        if parked.wait == GuardExecWait::Reserve
            && let Some(key) = self.running_activation().and_then(GuardKey::of)
            && self.granted(key)
        {
            self.activation_mut().flags.set_reserved(true);
        }
        self.end_guard_exec(*parked);
    }

    /// `NativeActivation::guardOn`/`guardOff` for the innermost native
    /// call, a method's: its lock reserved where `on`, waiting pinned for it
    /// where another activity owns it, else released one level where it
    /// holds it (`execution/NativeActivation.cpp:2395-2429`).
    pub(crate) fn native_guard(&mut self, on: bool) -> Result<(), Failure> {
        let frame = self.native_frame();
        if !frame.method || frame.reserved == on {
            return Ok(());
        }
        let key = GuardKey {
            object: frame.receiver,
            scope: frame.scope,
        };
        if !on {
            self.native_frame_mut().reserved = false;
            self.release_guard(key);
            return Ok(());
        }
        if let Reserve::Contended(owner) = self.take_guard(key) {
            if self.deadlocks(owner) {
                return Err(Raised::deadlock().into());
            }
            let me = self.running_activity();
            self.activities.guards.enqueue(key, me);
            if let Some(failure) = self.pinned_wait(ParkReason::Guard(key)) {
                if self.granted(key) {
                    self.native_frame_mut().reserved = true;
                }
                return Err(failure);
            }
        }
        self.native_frame_mut().reserved = true;
        Ok(())
    }

    /// `NativeActivation::guardWait` (`execution/NativeActivation.cpp:2435`):
    /// the innermost native call's lock released, a pinned wait for a store
    /// to a variable it watches, and the lock reserved again where it held
    /// it.
    pub(crate) fn native_guard_wait(&mut self) -> Result<(), Failure> {
        let reacquire = self.native_frame().reserved;
        self.native_guard(false)?;
        self.activity.guard_posted = false;
        if let Some(failure) = self.pinned_wait(ParkReason::GuardWhen) {
            return Err(failure);
        }
        self.activity.guard_posted = false;
        if reacquire {
            self.native_guard(true)?;
        }
        Ok(())
    }

    /// The lock a replying `activation` holds, as its continuation will:
    /// moved at nesting count 1, else released one level and reserved again
    /// by the continuation (`VariableDictionary::transfer`,
    /// `execution/VariableDictionary.cpp:600-617`).
    pub(crate) fn transfer_on_reply(&mut self, activation: &mut Activation) -> Transfer {
        if !activation.flags.reserved() {
            return Transfer::None;
        }
        let Some(key) = GuardKey::of(activation) else {
            return Transfer::None;
        };
        if self.guard_count(key) == 1 {
            return Transfer::Moved(key);
        }
        self.release_guard(key);
        activation.flags.set_reserved(false);
        Transfer::Again(GuardWait {
            key,
            activation: activation.id,
            queued: false,
            after_entry: false,
        })
    }

    /// A `REPLY` continuation's reserve of the lock its replier could not
    /// move (`RexxActivation::run`, `execution/RexxActivation.cpp:566-571`),
    /// before it resumes: whether it holds it now, else it is queued.
    pub(crate) fn reserve_for_continuation(&mut self) -> bool {
        if self.parked_for_guard() {
            self.take_granted_guard();
            return true;
        }
        let Some(wait) = self.activity.guard_waits.pop() else {
            return true;
        };
        let me = self.running_activity();
        match self.take_guard(wait.key) {
            Reserve::Held => {
                if let Some(running) = self.activity.running.as_deref_mut() {
                    running.flags.set_reserved(true);
                }
                true
            }
            Reserve::Contended(_) => {
                self.activities.guards.enqueue(wait.key, me);
                self.activity.guard_waits.push(GuardWait {
                    queued: true,
                    ..wait
                });
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(n: u32) -> GuardKey {
        GuardKey {
            object: ObjRef::heap(n, 0),
            scope: ObjRef::NIL,
        }
    }

    #[test]
    fn a_release_hands_the_lock_to_the_first_waiter_at_count_one() {
        let (a, b, c) = (
            ActivityId::test(1),
            ActivityId::test(2),
            ActivityId::test(3),
        );
        let mut table = GuardTable::default();
        assert!(matches!(table.try_reserve(key(1), a), Reserve::Held));
        assert!(matches!(table.try_reserve(key(1), a), Reserve::Held));
        assert!(matches!(table.try_reserve(key(1), b), Reserve::Contended(owner) if owner == a));
        table.enqueue(key(1), b);
        table.enqueue(key(1), c);
        assert_eq!(table.release(key(1)), None);
        assert_eq!(table.count(key(1)), 1);
        assert_eq!(table.release(key(1)), Some(b));
        assert_eq!((table.owner(key(1)), table.count(key(1))), (Some(b), 1));
        table.withdraw(c);
        assert_eq!(table.release(key(1)), None);
        assert_eq!(table.owner(key(1)), None);
    }

    /// Two started activities' sends of a guarded method run one after the
    /// other, and of an unguarded one interleave, under a switch at every
    /// clause.
    #[test]
    fn only_an_unguarded_method_interleaves_under_every_switch() {
        let source = "o = .k~new\n\
             a = o~start('G', 'A')\nb = o~start('G', 'B')\na~wait\nb~wait\n\
             c = o~start('U', 'C')\nd = o~start('U', 'D')\nc~wait\nd~wait\n\
             ::class k\n::method g\n  use arg n\n  do i = 1 to 3\n    say n i\n  end\n\
             ::method u unguarded\n  use arg n\n  do i = 1 to 3\n    say n i\n  end\n";
        let outcome = crate::run_program(
            "/tmp/guards.rex",
            source.as_bytes().to_vec(),
            crate::Invocation::none()
                .with_deadline(std::time::Duration::from_secs(60))
                .with_switch_mode(crate::SwitchMode::EveryOpportunity),
        );
        let out = String::from_utf8_lossy(&outcome.stdout).into_owned();
        assert_eq!(
            outcome.exit_code,
            0,
            "{}",
            String::from_utf8_lossy(&outcome.stderr)
        );
        let lines: Vec<&str> = out.lines().collect();
        assert_eq!(
            lines[..6],
            ["A 1", "A 2", "A 3", "B 1", "B 2", "B 3"],
            "{out}"
        );
        let at = |line: &str| {
            lines
                .iter()
                .position(|seen| *seen == line)
                .unwrap_or_else(|| panic!("no {line} in {out}"))
        };
        assert!(at("D 1") < at("C 3"), "{out}");
    }

    #[test]
    fn pools_are_numbered_once_each_in_the_order_asked() {
        let mut table = GuardTable::default();
        assert_eq!(table.pool_number(key(7)), 1);
        assert_eq!(table.pool_number(key(3)), 2);
        assert_eq!(table.pool_number(key(7)), 1);
    }
}
