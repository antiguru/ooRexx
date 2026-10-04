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

/// The most threads an interpreter's pool runs.
pub(crate) const POOL_BOUND: usize = 64;

/// What a pool thread runs.
pub(crate) type Job = Box<dyn FnOnce() + Send>;

/// Runs `job`, posting a panic it ends in to `inbox` for the baton's holder
/// to panic with, before the baton lent to this thread goes back: a lend
/// is not given back as a panic unwinds ([`crate::island::Lent`]).
pub(crate) fn posting_panics(
    inbox: &crate::timer::Inbox<super::Posted>,
    baton: &crate::island::InterpBaton,
    job: impl FnOnce(),
) {
    if let Err(payload) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(job)) {
        inbox.post(super::Posted::Panicked(payload));
        inbox.requests().set(crate::timer::PANICKED);
        if baton.lent().is_some() {
            baton.give_back();
        }
    }
}

pub(crate) struct Pool {
    shared: Arc<Shared>,
    stack: usize,
    bound: usize,
}

struct Shared {
    state: Mutex<State>,
}

struct State {
    idle: Vec<Worker>,
    threads: Vec<(ThreadId, JoinHandle<()>)>,
    closed: bool,
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
    /// An empty pool of at most `bound` threads, each with `stack` bytes of
    /// stack.
    pub(crate) fn new(stack: usize, bound: usize) -> Pool {
        Pool {
            shared: Arc::new(Shared {
                state: Mutex::new(State {
                    idle: Vec::new(),
                    threads: Vec::new(),
                    closed: false,
                }),
            }),
            stack,
            bound,
        }
    }

    /// The stack each thread gets.
    pub(crate) fn stack(&self) -> usize {
        self.stack
    }

    /// An idle thread, or a new one, for one job; `None` where the bound is
    /// reached or the spawn fails.
    pub(crate) fn reserve(&self) -> Option<Worker> {
        let mut state = lock(&self.shared.state);
        if let Some(worker) = state.idle.pop() {
            return Some(worker);
        }
        if state.threads.len() >= self.bound {
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

/// A pool thread: runs each job it is sent, then waits idle for the next.
fn work(shared: &Arc<Shared>, mailbox: &Arc<Mailbox>) {
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
        state.idle.push(Worker {
            thread: std::thread::current().id(),
            mailbox: Arc::clone(mailbox),
        });
    }
}
