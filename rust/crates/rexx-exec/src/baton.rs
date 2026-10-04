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

//! The baton: the right to touch an interpreter's state, held by one thread
//! at a time (spec 2026-09-29 P6-3, 2.3). The thread that creates an
//! interpreter holds it. A holder may lend it, with a value `L`, to a named
//! thread and waits, touching nothing, until that thread gives it back; lends
//! nest, so the holders form a stack whose top alone runs.

use crate::sync::{Condvar, Mutex, lock, thread, wait};

pub(crate) struct Baton<L: Copy> {
    state: Mutex<State<L>>,
    /// Notified at each change of holder.
    changed: Condvar,
    /// How many times a callback took it.
    #[cfg(all(test, not(loom)))]
    takes: std::sync::atomic::AtomicU64,
}

struct State<L> {
    /// The thread holding it, or `None` while it is free.
    holder: Option<thread::ThreadId>,
    /// The lends in force, innermost last.
    lends: Vec<Lend<L>>,
}

struct Lend<L> {
    lender: thread::ThreadId,
    lendee: thread::ThreadId,
    lent: L,
}

impl<L: Copy> Baton<L> {
    /// A free baton.
    pub(crate) fn new() -> Baton<L> {
        Baton {
            state: Mutex::new(State {
                holder: None,
                lends: Vec::new(),
            }),
            changed: Condvar::new(),
            #[cfg(all(test, not(loom)))]
            takes: std::sync::atomic::AtomicU64::new(0),
        }
    }

    /// Counts a callback's take.
    #[cfg(all(test, not(loom)))]
    pub(crate) fn count_take(&self) {
        self.takes
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    /// How many takes [`Baton::count_take`] counted.
    #[cfg(all(test, not(loom)))]
    pub(crate) fn takes(&self) -> u64 {
        self.takes.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Blocks until the baton is free, then holds it on this thread.
    ///
    /// # Panics
    ///
    /// If this thread holds it already.
    pub(crate) fn acquire(&self) {
        assert!(
            self.take_unless_held(),
            "a thread acquired a baton it holds"
        );
    }

    /// Frees the baton for the next thread waiting in [`Baton::acquire`].
    ///
    /// # Panics
    ///
    /// If this thread does not hold it, or holds it by a lend.
    pub(crate) fn release(&self) {
        let mut state = lock(&self.state);
        let me = thread::current().id();
        assert_eq!(
            state.holder,
            Some(me),
            "a thread released a baton it does not hold"
        );
        assert!(
            state.lends.last().is_none_or(|lend| lend.lendee != me),
            "a thread released a baton lent to it"
        );
        state.holder = None;
        drop(state);
        self.changed.notify_all();
    }

    /// Whether this thread holds the baton.
    pub(crate) fn held_here(&self) -> bool {
        lock(&self.state).holder == Some(thread::current().id())
    }

    /// Takes the baton unless this thread holds it, as a callback into the
    /// interpreter does (spec 2026-09-29 2.4), answering whether it took it.
    pub(crate) fn take_unless_held(&self) -> bool {
        let me = thread::current().id();
        let mut state = lock(&self.state);
        if state.holder == Some(me) {
            return false;
        }
        while state.holder.is_some() {
            state = wait(&self.changed, state);
        }
        state.holder = Some(me);
        true
    }

    /// Lends the baton this thread holds to `to` with `lent`, and blocks until
    /// `to` gives it back.
    ///
    /// # Panics
    ///
    /// If this thread does not hold it.
    pub(crate) fn lend(&self, to: thread::ThreadId, lent: L) {
        let me = thread::current().id();
        let mut state = lock(&self.state);
        assert_eq!(
            state.holder,
            Some(me),
            "a thread lent a baton it does not hold"
        );
        let depth = state.lends.len();
        state.lends.push(Lend {
            lender: me,
            lendee: to,
            lent,
        });
        state.holder = Some(to);
        self.changed.notify_all();
        while state.lends.len() > depth || state.holder != Some(me) {
            state = wait(&self.changed, state);
        }
    }

    /// Blocks until the baton is lent to this thread, and answers what was
    /// lent with it.
    pub(crate) fn await_lend(&self) -> L {
        let me = thread::current().id();
        let mut state = lock(&self.state);
        loop {
            if state.holder == Some(me)
                && let Some(lend) = state.lends.last()
                && lend.lendee == me
            {
                return lend.lent;
            }
            state = wait(&self.changed, state);
        }
    }

    /// What was lent with the baton, where this thread holds it by the
    /// innermost lend.
    pub(crate) fn lent(&self) -> Option<L> {
        let me = thread::current().id();
        let state = lock(&self.state);
        match state.lends.last() {
            Some(lend) if state.holder == Some(me) && lend.lendee == me => Some(lend.lent),
            _ => None,
        }
    }

    /// Gives the baton lent to this thread back to its lender.
    ///
    /// # Panics
    ///
    /// If the innermost lend is not to this thread.
    pub(crate) fn give_back(&self) {
        let me = thread::current().id();
        let mut state = lock(&self.state);
        assert_eq!(
            state.holder,
            Some(me),
            "a thread gave back a baton it does not hold"
        );
        let lend = state
            .lends
            .pop()
            .expect("a thread gave back a baton nobody lent");
        assert_eq!(lend.lendee, me, "a thread gave back another thread's lend");
        state.holder = Some(lend.lender);
        drop(state);
        self.changed.notify_all();
    }
}
