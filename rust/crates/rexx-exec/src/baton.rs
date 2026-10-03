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
//! interpreter holds it; it is released only at a driver exit.

use crate::sync::{Condvar, Mutex, lock, thread, wait};

pub(crate) struct Baton {
    /// The thread holding it, or `None` while it is free.
    holder: Mutex<Option<thread::ThreadId>>,
    /// Notified at each release.
    released: Condvar,
}

impl Baton {
    /// A free baton.
    pub(crate) fn new() -> Baton {
        Baton {
            holder: Mutex::new(None),
            released: Condvar::new(),
        }
    }

    /// Blocks until the baton is free, then holds it on this thread.
    ///
    /// # Panics
    ///
    /// If this thread holds it already.
    pub(crate) fn acquire(&self) {
        let me = thread::current().id();
        let mut holder = lock(&self.holder);
        assert_ne!(*holder, Some(me), "a thread acquired a baton it holds");
        while holder.is_some() {
            holder = wait(&self.released, holder);
        }
        *holder = Some(me);
    }

    /// Frees the baton for the next thread waiting in [`Baton::acquire`].
    ///
    /// # Panics
    ///
    /// If this thread does not hold it.
    #[cfg(test)]
    pub(crate) fn release(&self) {
        let mut holder = lock(&self.holder);
        assert_eq!(
            *holder,
            Some(thread::current().id()),
            "a thread released a baton it does not hold"
        );
        *holder = None;
        drop(holder);
        self.released.notify_one();
    }

    /// Whether this thread holds the baton.
    pub(crate) fn held_here(&self) -> bool {
        *lock(&self.holder) == Some(thread::current().id())
    }
}
