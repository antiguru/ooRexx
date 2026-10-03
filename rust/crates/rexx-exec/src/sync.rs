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

//! The synchronisation the baton, the inbox and the timer use across threads:
//! `std`'s, or `loom`'s where `tests/loom.rs` compiles these same sources
//! under `--cfg loom` (spec 2026-09-29 R4). `loom` stays a dev-dependency,
//! so only a test build ever selects it, and the clock is a model clock
//! there.

#[cfg(all(loom, test))]
pub(crate) use loom::sync::atomic::{AtomicU32, Ordering};
#[cfg(all(loom, test))]
pub(crate) use loom::sync::{Arc, Condvar, Mutex, MutexGuard};
#[cfg(all(loom, test))]
pub(crate) use loom::thread;
#[cfg(not(all(loom, test)))]
pub(crate) use std::sync::atomic::{AtomicU32, Ordering};
#[cfg(not(all(loom, test)))]
pub(crate) use std::sync::{Arc, Condvar, Mutex, MutexGuard};
#[cfg(not(all(loom, test)))]
pub(crate) use std::thread;

use std::sync::PoisonError;
use std::time::{Duration, Instant};

/// Locks `mutex`. A panic that poisoned it left its state whole: every
/// critical section here writes its fields before anything can panic.
pub(crate) fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

/// Waits on `condvar`, giving up `guard` meanwhile.
pub(crate) fn wait<'a, T>(condvar: &Condvar, guard: MutexGuard<'a, T>) -> MutexGuard<'a, T> {
    condvar.wait(guard).unwrap_or_else(PoisonError::into_inner)
}

/// [`wait`], for at most `timeout`. `loom` models no timeouts, so a model
/// waits for a notification.
pub(crate) fn wait_timeout<'a, T>(
    condvar: &Condvar,
    guard: MutexGuard<'a, T>,
    timeout: Duration,
) -> MutexGuard<'a, T> {
    condvar
        .wait_timeout(guard, timeout)
        .unwrap_or_else(PoisonError::into_inner)
        .0
}

#[cfg(not(all(loom, test)))]
pub(crate) fn now() -> Instant {
    Instant::now()
}

/// The model clock: the instant the model execution began, plus what
/// [`advance`] added.
#[cfg(all(loom, test))]
pub(crate) fn now() -> Instant {
    let (began, elapsed) = &*CLOCK;
    *began + Duration::from_nanos(elapsed.load(std::sync::atomic::Ordering::SeqCst))
}

/// Moves the model clock on by `by`.
#[cfg(all(loom, test))]
pub(crate) fn advance(by: Duration) {
    let nanos = u64::try_from(by.as_nanos()).expect("a model advance fits in u64 nanoseconds");
    CLOCK
        .1
        .fetch_add(nanos, std::sync::atomic::Ordering::SeqCst);
}

#[cfg(all(loom, test))]
loom::lazy_static! {
    static ref CLOCK: (Instant, std::sync::atomic::AtomicU64) =
        (Instant::now(), std::sync::atomic::AtomicU64::new(0));
}
