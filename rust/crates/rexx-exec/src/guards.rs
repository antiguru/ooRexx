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
    locks: FxHashMap<GuardKey, Lock>,
    waiting: FxHashMap<ActivityId, Waiting>,
    pools: FxHashMap<GuardKey, u32>,
    last_pool: u32,
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

    /// Drops the pool numbers of objects `live` no longer answers for.
    pub(crate) fn prune_pools(&mut self, live: impl Fn(ObjRef) -> bool) {
        self.pools.retain(|key, _| live(key.object));
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

/// Whether a method the directive `kind` declares is guarded: every
/// `::METHOD` and `::ATTRIBUTE` without `UNGUARDED`, and no `::CONSTANT`
/// (`parser/DirectiveParser.cpp:2525`).
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
    fn of(activation: &Activation) -> Option<GuardKey> {
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
    /// Reserves `key` for the method activation just pushed, or leaves the
    /// reserve to a clause boundary of the method: its first, where another
    /// activity owns the lock, and the one after its `>I>` where that is
    /// traced early (`RexxActivation::run`, `execution/RexxActivation.cpp:523-532`).
    pub(crate) fn reserve_for_method(&mut self, key: GuardKey) {
        let after_entry = self.activation().plan.traces_entry_early;
        if !after_entry {
            let me = self.running_activity();
            if let Reserve::Held = self.activities.guards.try_reserve(key, me) {
                self.activation_mut().scope_reserved = true;
                return;
            }
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
        let me = self.running_activity();
        let owner = match self.activities.guards.try_reserve(wait.key, me) {
            Reserve::Held => {
                self.activation_mut().scope_reserved = true;
                return Ok(());
            }
            Reserve::Contended(owner) => owner,
        };
        if self.deadlocks(owner) {
            self.blame_first_clause();
            return Err(Raised::deadlock().into());
        }
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
        match self.pinned_wait(ParkReason::Guard(wait.key)) {
            None => {
                self.activation_mut().scope_reserved = true;
                Ok(())
            }
            Some(failure) => Err(failure),
        }
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
            running.scope_reserved = true;
        }
    }

    /// Releases one level of `key`, waking the waiter it passes to.
    pub(crate) fn release_guard(&mut self, key: GuardKey) {
        if let Some(next) = self.activities.guards.release(key) {
            self.unpark(next);
        }
    }

    /// The running method activation's end, unless a `REPLY` moves it on:
    /// its guard lock released, and a reserve still pending dropped
    /// (`RexxActivation::termination`'s `guardOff`).
    pub(crate) fn guard_off_at_end(&mut self) {
        let Some(ending) = self.activity.running.as_deref() else {
            return;
        };
        if ending
            .replied
            .as_ref()
            .is_some_and(|replied| replied.continuation.is_some())
        {
            return;
        }
        let id = ending.id;
        let key = if ending.scope_reserved {
            GuardKey::of(ending)
        } else {
            None
        };
        if !self.activity.guard_waits.is_empty() {
            self.activity
                .guard_waits
                .retain(|wait| wait.activation != id);
        }
        if let Some(key) = key {
            self.activation_mut().scope_reserved = false;
            self.release_guard(key);
        }
    }

    /// `GUARD ON` and `GUARD OFF`'s change of the running method's lock
    /// (`RexxActivation::guardOn`, `execution/RexxActivation.cpp:2193-2207`,
    /// and `guardOff`). A contended `GUARD ON` waits pinned, under the
    /// instruction's own pinned frame, and fails with 98.905 where the waits
    /// it would join lead back to this activity.
    pub(crate) fn set_guard_state(&mut self, on: bool) -> Result<(), Failure> {
        let activation = self.activation();
        let Some(key) = GuardKey::of(activation) else {
            return Ok(());
        };
        if !on {
            if activation.scope_reserved {
                self.activation_mut().scope_reserved = false;
                self.release_guard(key);
            }
            return Ok(());
        }
        if activation.scope_reserved {
            return Ok(());
        }
        let me = self.running_activity();
        if let Reserve::Contended(owner) = self.activities.guards.try_reserve(key, me) {
            if self.deadlocks(owner) {
                return Err(Raised::deadlock().into());
            }
            self.activities.guards.enqueue(key, me);
            if let Some(failure) = self.pinned_wait(ParkReason::Guard(key)) {
                return Err(failure);
            }
        }
        self.activation_mut().scope_reserved = true;
        Ok(())
    }

    /// The lock a replying `activation` holds, as its continuation will:
    /// moved at nesting count 1, else released one level and reserved again
    /// by the continuation (`VariableDictionary::transfer`,
    /// `execution/VariableDictionary.cpp:600-617`).
    pub(crate) fn transfer_on_reply(&mut self, activation: &mut Activation) -> Transfer {
        if !activation.scope_reserved {
            return Transfer::None;
        }
        let Some(key) = GuardKey::of(activation) else {
            return Transfer::None;
        };
        if self.activities.guards.count(key) == 1 {
            return Transfer::Moved(key);
        }
        self.release_guard(key);
        activation.scope_reserved = false;
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
        match self.activities.guards.try_reserve(wait.key, me) {
            Reserve::Held => {
                if let Some(running) = self.activity.running.as_deref_mut() {
                    running.scope_reserved = true;
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
