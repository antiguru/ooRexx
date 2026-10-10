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

//! An interpreter's pool of threads, which run native calls and blocking
//! operations while the baton's holder runs other activities (spec
//! 2026-09-29 2.7).

use std::sync::{Arc, Condvar, Mutex, PoisonError};
use std::thread::{JoinHandle, ThreadId};

/// A pool thread's stack: the interpreter thread's, since a callback runs
/// Rexx code there, and the translator's and the collector's depth limits
/// are measured against that size.
pub(crate) const POOL_STACK_BYTES: usize = crate::INTERPRETER_STACK_BYTES;

/// The most threads an interpreter's pool keeps. A job that finds every
/// thread busy gets a thread beyond the bound, which ends once the pool is
/// back within it, so a blocking call never waits for a busy thread.
pub(crate) const POOL_BOUND: usize = 64;

/// What a pool thread runs.
pub(crate) type Job = Box<dyn FnOnce() + Send>;

/// Runs `job`, posting a panic it ends in to `inbox` for the baton's holder
/// to panic with, before the baton lent to this thread goes back: a lend
/// is not given back as a panic unwinds ([`crate::island::Lent`]), and the
/// lender drains the inbox before it runs a clause.
pub(crate) fn posting_panics(
    inbox: &crate::timer::Inbox<super::Posted>,
    baton: &crate::island::InterpBaton,
    job: impl FnOnce(),
) {
    if let Err(payload) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(job)) {
        // Widens the window a lend given back while the panic unwinds would
        // open, for a test to see.
        #[cfg(test)]
        #[expect(
            clippy::disallowed_methods,
            reason = "a pool thread, which sim does not run"
        )]
        std::thread::sleep(std::time::Duration::from_millis(100));
        inbox.post(super::Posted::Panicked(payload));
        if baton.lent().is_some() {
            baton.give_back();
        }
    }
}

pub(crate) struct Pool {
    shared: Arc<Shared>,
    stack: usize,
}

struct Shared {
    state: Mutex<State>,
}

struct State {
    idle: Vec<Worker>,
    threads: Vec<(ThreadId, JoinHandle<()>)>,
    closed: bool,
    bound: usize,
    /// Whether no thread is spawned beyond the bound.
    #[cfg(test)]
    fixed: bool,
}

/// A pool thread, idle or reserved for one job.
pub(crate) struct Worker {
    thread: ThreadId,
    mailbox: Arc<Mailbox>,
}

struct Mailbox {
    mail: Mutex<Option<Mail>>,
    arrived: Condvar,
}

enum Mail {
    Run(Job),
    End,
}

fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

impl Pool {
    /// An empty pool that keeps at most `bound` threads, each with `stack`
    /// bytes of stack.
    pub(crate) fn new(stack: usize, bound: usize) -> Pool {
        Pool {
            shared: Arc::new(Shared {
                state: Mutex::new(State {
                    idle: Vec::new(),
                    threads: Vec::new(),
                    closed: false,
                    bound,
                    #[cfg(test)]
                    fixed: false,
                }),
            }),
            stack,
        }
    }

    /// The stack each thread gets.
    pub(crate) fn stack(&self) -> usize {
        self.stack
    }

    /// Bounds the pool at `bound` threads kept from now on; at 0 no thread,
    /// idle or new, is reserved.
    pub(crate) fn set_bound(&self, bound: usize) {
        lock(&self.shared.state).bound = bound;
    }

    /// Spawns no thread beyond the bound from now on, so a job that finds
    /// every thread busy gets none.
    #[cfg(test)]
    pub(crate) fn fix_at_bound(&self) {
        lock(&self.shared.state).fixed = true;
    }

    /// An idle thread, or a new one, for one job: beyond the bound where
    /// every thread is busy. `None` where the bound is 0 or the spawn fails.
    pub(crate) fn reserve(&self) -> Option<Worker> {
        let mut state = lock(&self.shared.state);
        if state.bound == 0 {
            return None;
        }
        if let Some(worker) = state.idle.pop() {
            return Some(worker);
        }
        // Every thread busy: a new one beyond the bound, since a command or
        // native call run inline instead holds the baton while it waits, and
        // what it waits for may be another activity's to do.
        #[cfg(test)]
        if state.fixed && state.threads.len() >= state.bound {
            return None;
        }
        let mailbox = Arc::new(Mailbox {
            mail: Mutex::new(None),
            arrived: Condvar::new(),
        });
        let spawned = std::thread::Builder::new()
            .name("rexx-pool".to_string())
            .stack_size(self.stack)
            .spawn({
                let (shared, mailbox) = (Arc::clone(&self.shared), Arc::clone(&mailbox));
                move || work(&shared, &mailbox)
            })
            .ok()?;
        let thread = spawned.thread().id();
        #[cfg(test)]
        SPAWNED.with(|spawned| spawned.set(spawned.get() + 1));
        #[cfg(test)]
        if state.threads.len() >= state.bound {
            BEYOND.with(|beyond| beyond.set(beyond.get() + 1));
        }
        state.threads.push((thread, spawned));
        Some(Worker { thread, mailbox })
    }
}

impl Drop for Pool {
    /// Ends the idle threads and waits for them; a thread still running a
    /// job ends once it has run it.
    fn drop(&mut self) {
        let (idle, mut threads) = {
            let mut state = lock(&self.shared.state);
            state.closed = true;
            (
                std::mem::take(&mut state.idle),
                std::mem::take(&mut state.threads),
            )
        };
        for worker in idle {
            worker.send(Mail::End);
            if let Some(at) = threads.iter().position(|(id, _)| *id == worker.thread) {
                let (_, handle) = threads.swap_remove(at);
                let _ = handle.join();
            }
        }
    }
}

impl Worker {
    /// The thread's identity, which the baton is lent to.
    pub(crate) fn thread(&self) -> ThreadId {
        self.thread
    }

    /// Runs `job` on the thread.
    pub(crate) fn run(self, job: Job) {
        self.send(Mail::Run(job));
    }

    fn send(&self, mail: Mail) {
        *lock(&self.mailbox.mail) = Some(mail);
        self.mailbox.arrived.notify_one();
    }
}

#[cfg(test)]
thread_local! {
    /// How many pool threads this thread has spawned.
    static SPAWNED: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

/// [`SPAWNED`]'s count.
#[cfg(test)]
pub(crate) fn threads_spawned() -> u64 {
    SPAWNED.with(std::cell::Cell::get)
}

#[cfg(test)]
thread_local! {
    /// How many threads this thread has spawned beyond its pool's bound.
    static BEYOND: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

/// [`BEYOND`]'s count.
#[cfg(test)]
pub(crate) fn threads_beyond_bound() -> u64 {
    BEYOND.with(std::cell::Cell::get)
}

/// A pool thread: runs each job it is sent, then waits idle for the next,
/// or ends where the pool holds more threads than its bound.
fn work(shared: &Arc<Shared>, mailbox: &Arc<Mailbox>) {
    crate::signal::block();
    loop {
        let mail = {
            let mut mail = lock(&mailbox.mail);
            loop {
                if let Some(arrived) = mail.take() {
                    break arrived;
                }
                mail = mailbox
                    .arrived
                    .wait(mail)
                    .unwrap_or_else(PoisonError::into_inner);
            }
        };
        let Mail::Run(job) = mail else {
            return;
        };
        job();
        let mut state = lock(&shared.state);
        if state.closed {
            return;
        }
        if state.threads.len() > state.bound {
            let me = std::thread::current().id();
            if let Some(at) = state.threads.iter().position(|(id, _)| *id == me) {
                // Dropping the handle detaches this thread, which ends here.
                state.threads.swap_remove(at);
            }
            return;
        }
        state.idle.push(Worker {
            thread: std::thread::current().id(),
            mailbox: Arc::clone(mailbox),
        });
    }
}
