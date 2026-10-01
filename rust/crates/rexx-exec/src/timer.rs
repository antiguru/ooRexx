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
//! global state (spec 2026-09-29 section 4, R3). Each interpreter registers
//! its request word; the timer sets `SLICE` in every armed one each slice,
//! and wakes an idle one when the deadline it idles until is due.

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Condvar, Mutex, MutexGuard, Once, PoisonError};
use std::time::{Duration, Instant};

/// The running activity's slice is over.
pub(crate) const SLICE: u32 = 1;

/// `ActivityManager::timeSliceLength` (`concurrency/ActivityManager.hpp:359`).
const SLICE_LENGTH: Duration = Duration::from_millis(24);

/// An interpreter's request bits, which other threads set.
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

/// One live interpreter.
struct Live {
    id: u64,
    requests: Arc<Requests>,
    /// When its current slice began, while it is armed.
    slice_began: Option<Instant>,
    /// The deadline it idles until, while it is idle.
    idle_until: Option<Instant>,
    /// Set by the timer when `idle_until` is due.
    woken: bool,
    /// What it idles on, with the registry's lock.
    wake: Arc<Condvar>,
}

struct Registry {
    live: Mutex<(u64, Vec<Live>)>,
    changed: Condvar,
}

static REGISTRY: Registry = Registry {
    live: Mutex::new((0, Vec::new())),
    changed: Condvar::new(),
};

static TIMER: Once = Once::new();

fn live() -> MutexGuard<'static, (u64, Vec<Live>)> {
    REGISTRY.live.lock().unwrap_or_else(PoisonError::into_inner)
}

/// An interpreter's entry in the registry, removed when it drops.
pub(crate) struct Registration {
    id: u64,
    requests: Arc<Requests>,
    armed: bool,
}

impl Registration {
    pub(crate) fn new() -> Registration {
        let requests = Arc::new(Requests(AtomicU32::new(0)));
        let mut live = live();
        live.0 += 1;
        let id = live.0;
        live.1.push(Live {
            id,
            requests: Arc::clone(&requests),
            slice_began: None,
            idle_until: None,
            woken: false,
            wake: Arc::new(Condvar::new()),
        });
        Registration {
            id,
            requests,
            armed: false,
        }
    }

    pub(crate) fn requests(&self) -> &Requests {
        &self.requests
    }

    /// Has the timer set `SLICE` each slice from now on.
    pub(crate) fn arm(&mut self) {
        if self.armed {
            return;
        }
        self.armed = true;
        start_timer();
        let mut live = live();
        if let Some(entry) = live.1.iter_mut().find(|entry| entry.id == self.id) {
            entry.slice_began = Some(Instant::now());
        }
        REGISTRY.changed.notify_one();
    }

    /// Whether the registry has this interpreter armed.
    #[cfg(test)]
    pub(crate) fn armed_in_registry(&self) -> bool {
        live()
            .1
            .iter()
            .any(|entry| entry.id == self.id && entry.slice_began.is_some())
    }

    pub(crate) fn disarm(&mut self) {
        if !self.armed {
            return;
        }
        self.armed = false;
        let mut live = live();
        if let Some(entry) = live.1.iter_mut().find(|entry| entry.id == self.id) {
            entry.slice_began = None;
        }
    }

    /// Blocks this thread until the timer finds `at` due.
    pub(crate) fn idle_until(&self, at: Instant) {
        start_timer();
        let mut live = live();
        let Some(entry) = live.1.iter_mut().find(|entry| entry.id == self.id) else {
            return;
        };
        entry.idle_until = Some(at);
        entry.woken = false;
        let wake = Arc::clone(&entry.wake);
        REGISTRY.changed.notify_one();
        loop {
            live = wake.wait(live).unwrap_or_else(PoisonError::into_inner);
            let entry = live.1.iter_mut().find(|entry| entry.id == self.id);
            if let Some(entry) = entry
                && entry.woken
            {
                entry.woken = false;
                return;
            }
        }
    }
}

fn start_timer() {
    TIMER.call_once(|| {
        std::thread::Builder::new()
            .name("rexx-timer".to_string())
            .spawn(run_timer)
            .expect("spawning the timer thread");
    });
}

impl Drop for Registration {
    fn drop(&mut self) {
        live().1.retain(|entry| entry.id != self.id);
    }
}

/// The timer thread: sets `SLICE` in each armed interpreter whose slice has
/// run out, wakes each idle one whose deadline is due, and sleeps until the
/// next of either, or with no deadline where there is none.
fn run_timer() {
    let mut live = live();
    loop {
        let now = Instant::now();
        let mut next: Option<Instant> = None;
        for entry in &mut live.1 {
            if let Some(at) = entry.idle_until {
                if at <= now {
                    entry.idle_until = None;
                    entry.woken = true;
                    entry.wake.notify_one();
                } else {
                    next = Some(next.map_or(at, |next| next.min(at)));
                }
            }
            let Some(began) = entry.slice_began else {
                continue;
            };
            let mut due = began + SLICE_LENGTH;
            if due <= now {
                entry.requests.set(SLICE);
                entry.slice_began = Some(now);
                due = now + SLICE_LENGTH;
            }
            next = Some(next.map_or(due, |next| next.min(due)));
        }
        live = match next {
            None => REGISTRY
                .changed
                .wait(live)
                .unwrap_or_else(PoisonError::into_inner),
            Some(at) => {
                REGISTRY
                    .changed
                    .wait_timeout(live, at.saturating_duration_since(now))
                    .unwrap_or_else(PoisonError::into_inner)
                    .0
            }
        };
    }
}
