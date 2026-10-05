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

//! The live-interpreter registry and the timer thread, the process's only
//! global state (spec 2026-09-29 section 4, R3), and each interpreter's
//! inbox. Each interpreter registers its inbox; the timer sets `SLICE` in
//! every armed one each slice, wakes an idle one through its inbox when the
//! deadline it idles until is due, and posts a halt to every one when a
//! signal arrives. The timer thread runs while any interpreter is
//! registered.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use crate::sync::{
    Arc, AtomicU32, Condvar, Mutex, MutexGuard, Ordering, Wake, lock, now, thread, wait,
};

/// The running activity's slice is over.
pub(crate) const SLICE: u32 = 1;

/// Something was posted to the inbox and not yet taken.
pub(crate) const INBOX: u32 = 2;

/// A signal's halt was posted to the inbox, beside `INBOX`.
pub(crate) const HALT: u32 = 4;

/// `ActivityManager::timeSliceLength` (`concurrency/ActivityManager.hpp:359`).
const SLICE_LENGTH: Duration = Duration::from_millis(24);

/// What other threads post to an interpreter's inbox.
#[cfg(not(all(loom, test)))]
pub(crate) type Posted = crate::scheduler::Posted;

/// A model's stand-in for a completion.
#[cfg(all(loom, test))]
pub(crate) type Posted = u32;

/// What the timer posts for a signal.
#[cfg(not(all(loom, test)))]
fn halt() -> Posted {
    crate::scheduler::Posted::Halt
}

/// A model's stand-in for a signal's halt.
#[cfg(all(loom, test))]
pub(crate) const HALT_POSTED: Posted = u32::MAX;

#[cfg(all(loom, test))]
fn halt() -> Posted {
    HALT_POSTED
}

/// An interpreter's request bits, which other threads set. No data is
/// published through a bit: what a post carries is under the inbox's lock.
pub(crate) struct Requests(AtomicU32);

impl Requests {
    pub(crate) fn set(&self, bits: u32) {
        self.0.fetch_or(bits, Ordering::Release);
    }

    pub(crate) fn pending(&self, bits: u32) -> bool {
        self.0.load(Ordering::Acquire) & bits != 0
    }

    pub(crate) fn clear(&self, bits: u32) {
        self.0.fetch_and(!bits, Ordering::AcqRel);
    }
}

/// An interpreter's inbox (spec 2026-09-29 section 2.1): its request word,
/// and a queue other threads post to and the baton holder takes from. The
/// holder idles here, and a post or the timer wakes it.
pub(crate) struct Inbox<T> {
    requests: Requests,
    queue: Mutex<Queue<T>>,
    /// Notified at each post and each timer wake.
    arrived: Condvar,
}

struct Queue<T> {
    posted: VecDeque<T>,
    /// The timer found the holder's idle deadline due.
    woken: bool,
}

impl<T> Inbox<T> {
    pub(crate) fn new() -> Inbox<T> {
        Inbox {
            requests: Requests(AtomicU32::new(0)),
            queue: Mutex::new(Queue {
                posted: VecDeque::new(),
                woken: false,
            }),
            arrived: Condvar::new(),
        }
    }

    pub(crate) fn requests(&self) -> &Requests {
        &self.requests
    }

    /// Queues `item` for the holder, and wakes it if it idles.
    pub(crate) fn post(&self, item: T) {
        let mut queue = lock(&self.queue);
        queue.posted.push_back(item);
        self.requests.set(INBOX);
        drop(queue);
        self.arrived.notify_one();
    }

    /// Puts `items`, taken by the holder and not yet served, back ahead of
    /// anything posted since.
    pub(crate) fn requeue(&self, items: VecDeque<T>) {
        if items.is_empty() {
            return;
        }
        let mut queue = lock(&self.queue);
        for item in items.into_iter().rev() {
            queue.posted.push_front(item);
        }
        self.requests.set(INBOX);
        drop(queue);
        self.arrived.notify_one();
    }

    /// What was posted since the holder last took, in order; without
    /// locking while `INBOX` is clear.
    pub(crate) fn drain(&self) -> VecDeque<T> {
        if !self.requests.pending(INBOX) {
            return VecDeque::new();
        }
        Self::take(&mut lock(&self.queue), &self.requests)
    }

    /// Clears `INBOX` with the queue locked, so a post after it sets the bit
    /// again.
    fn take(queue: &mut Queue<T>, requests: &Requests) -> VecDeque<T> {
        requests.clear(INBOX);
        std::mem::take(&mut queue.posted)
    }

    /// Forgets a timer wake left from an earlier idle.
    fn expect_wake(&self) {
        lock(&self.queue).woken = false;
    }

    /// Called with the registry locked: never take the registry under `queue`.
    fn wake(&self) {
        lock(&self.queue).woken = true;
        self.arrived.notify_one();
    }

    /// Blocks the holder until the timer wakes it or something is posted,
    /// and answers what was posted.
    pub(crate) fn idle(&self) -> VecDeque<T> {
        let mut queue = lock(&self.queue);
        while !queue.woken && queue.posted.is_empty() {
            queue = wait(&self.arrived, queue);
        }
        queue.woken = false;
        Self::take(&mut queue, &self.requests)
    }
}

/// One live interpreter.
struct Live {
    id: u64,
    inbox: Arc<Inbox<Posted>>,
    /// When its current slice began, while it is armed.
    slice_began: Option<Instant>,
    /// The deadline it idles until, while it is idle.
    idle_until: Option<Instant>,
}

struct State {
    next_id: u64,
    live: Vec<Live>,
    timer_running: bool,
    /// Every timer thread started, for a model to join: a `loom` model ends
    /// only once every thread has, and the shipped timer thread is detached.
    #[cfg(all(loom, test))]
    timers: Vec<thread::JoinHandle<()>>,
}

impl State {
    fn entry(&mut self, id: u64) -> Option<&mut Live> {
        self.live.iter_mut().find(|entry| entry.id == id)
    }

    /// Starts the timer thread unless it runs.
    fn start_timer(&mut self) {
        if self.timer_running {
            return;
        }
        let timer = thread::Builder::new()
            .name("rexx-timer".to_string())
            .spawn(run_timer)
            .expect("spawning the timer thread");
        self.timer_running = true;
        #[cfg(all(loom, test))]
        self.timers.push(timer);
        #[cfg(not(all(loom, test)))]
        drop(timer);
    }

    /// One round of the timer at `now`: sets `SLICE` in each armed
    /// interpreter whose slice has run out, then wakes each idle one whose
    /// deadline is due, and answers the next instant either falls due.
    fn tick(&mut self, now: Instant) -> Option<Instant> {
        let mut next: Option<Instant> = None;
        for entry in &mut self.live {
            if let Some(began) = entry.slice_began {
                let mut due = began + SLICE_LENGTH;
                if due <= now {
                    entry.inbox.requests().set(SLICE);
                    entry.slice_began = Some(now);
                    due = now + SLICE_LENGTH;
                }
                next = Some(next.map_or(due, |next| next.min(due)));
            }
            if let Some(at) = entry.idle_until {
                if at <= now {
                    entry.idle_until = None;
                    entry.inbox.wake();
                } else {
                    next = Some(next.map_or(at, |next| next.min(at)));
                }
            }
        }
        next
    }
}

struct Registry {
    state: Mutex<State>,
    /// Woken when an entry is armed, idles or leaves, and by a signal.
    wake: Wake,
}

impl Registry {
    fn new() -> Registry {
        Registry {
            state: Mutex::new(State {
                next_id: 0,
                live: Vec::new(),
                timer_running: false,
                #[cfg(all(loom, test))]
                timers: Vec::new(),
            }),
            wake: Wake::new(),
        }
    }
}

#[cfg(not(all(loom, test)))]
fn registry() -> &'static Registry {
    static REGISTRY: std::sync::LazyLock<Registry> = std::sync::LazyLock::new(Registry::new);
    &REGISTRY
}

/// One registry per model execution.
#[cfg(all(loom, test))]
fn registry() -> &'static Registry {
    loom::lazy_static! {
        static ref REGISTRY: Registry = Registry::new();
    }
    &REGISTRY
}

fn live() -> MutexGuard<'static, State> {
    lock(&registry().state)
}

/// An interpreter's entry in the registry, removed when it drops.
pub(crate) struct Registration {
    id: u64,
    inbox: Arc<Inbox<Posted>>,
    armed: bool,
}

impl Registration {
    /// Registers a new interpreter and starts the timer thread unless it
    /// runs. A signal that arrived while none was registered halts nothing.
    pub(crate) fn new() -> Registration {
        let inbox = Arc::new(Inbox::new());
        let mut live = live();
        if live.live.is_empty() {
            registry().wake.take_signal();
        }
        live.start_timer();
        live.next_id += 1;
        let id = live.next_id;
        live.live.push(Live {
            id,
            inbox: Arc::clone(&inbox),
            slice_began: None,
            idle_until: None,
        });
        Registration {
            id,
            inbox,
            armed: false,
        }
    }

    pub(crate) fn requests(&self) -> &Requests {
        self.inbox.requests()
    }

    /// The inbox, for a thread that posts to it.
    pub(crate) fn inbox(&self) -> Arc<Inbox<Posted>> {
        Arc::clone(&self.inbox)
    }

    /// What was posted since the holder last took, in order.
    pub(crate) fn drain(&self) -> VecDeque<Posted> {
        self.inbox.drain()
    }

    /// [`Inbox::requeue`].
    pub(crate) fn requeue(&self, items: VecDeque<Posted>) {
        self.inbox.requeue(items);
    }

    /// Blocks this thread until something is posted, with no deadline for
    /// the timer to find due, and answers what was posted.
    pub(crate) fn idle(&self) -> VecDeque<Posted> {
        self.inbox.expect_wake();
        self.inbox.idle()
    }

    /// Has the timer set `SLICE` each slice from now on.
    pub(crate) fn arm(&mut self) {
        if self.armed {
            return;
        }
        self.armed = true;
        if let Some(entry) = live().entry(self.id) {
            entry.slice_began = Some(now());
        }
        registry().wake.notify();
    }

    /// Whether the registry has this interpreter armed.
    #[cfg(test)]
    pub(crate) fn armed_in_registry(&self) -> bool {
        live()
            .entry(self.id)
            .is_some_and(|entry| entry.slice_began.is_some())
    }

    pub(crate) fn disarm(&mut self) {
        if !self.armed {
            return;
        }
        self.armed = false;
        if let Some(entry) = live().entry(self.id) {
            entry.slice_began = None;
        }
    }

    /// Blocks this thread until the timer finds `at` due or something is
    /// posted to the inbox, and answers what was posted.
    pub(crate) fn idle_until(&self, at: Instant) -> VecDeque<Posted> {
        self.inbox.expect_wake();
        if let Some(entry) = live().entry(self.id) {
            entry.idle_until = Some(at);
        }
        registry().wake.notify();
        let posted = self.inbox.idle();
        if let Some(entry) = live().entry(self.id) {
            entry.idle_until = None;
        }
        posted
    }
}

/// Installs the signal handlers, writing to the timer's wake source.
#[cfg(not(all(loom, test)))]
pub(crate) fn install_signals() {
    registry().wake.install_signals();
}

/// What a signal handler does to the wake source.
#[cfg(all(loom, test))]
pub(crate) fn signal() {
    registry().wake.signal();
}

/// Waits for every timer thread started to end.
#[cfg(all(loom, test))]
pub(crate) fn join_timer() {
    let timers = std::mem::take(&mut live().timers);
    for timer in timers {
        timer.join().expect("the timer thread");
    }
}

impl Drop for Registration {
    fn drop(&mut self) {
        live().live.retain(|entry| entry.id != self.id);
        registry().wake.notify();
    }
}

/// Posts a halt to every live interpreter if a signal arrived since the
/// last take.
fn post_signal_halts(wake: &Wake, live: &State) {
    if wake.take_signal() {
        for entry in &live.live {
            entry.inbox.post(halt());
            entry.inbox.requests().set(HALT);
        }
    }
}

/// What the timer thread does for a signal, done now on this thread: a
/// signal that arrived with a blocking wait's end is served before the
/// wait's activity runs on.
#[cfg(not(all(loom, test)))]
pub(crate) fn serve_signal() {
    let registry = registry();
    post_signal_halts(&registry.wake, &lock(&registry.state));
}

/// The timer thread: posts a halt to every interpreter if a signal arrived,
/// ticks, then sleeps until the next deadline, or with no deadline where
/// there is none, until a wake; it ends when none is registered.
fn run_timer() {
    let registry = registry();
    loop {
        let timeout = {
            let mut live = lock(&registry.state);
            if live.live.is_empty() {
                live.timer_running = false;
                return;
            }
            post_signal_halts(&registry.wake, &live);
            let now = now();
            live.tick(now).map(|at| at.saturating_duration_since(now))
        };
        registry.wake.wait(timeout);
    }
}
