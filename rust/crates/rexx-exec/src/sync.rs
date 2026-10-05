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
//! under `--cfg loom` (spec 2026-09-29 R4), with a model clock. Under
//! `--cfg loom` the library's own unit tests compile to nothing (`lib.rs`),
//! so only `tests/loom.rs` selects `loom`.

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

/// The timer thread's wake source: a socket pair whose read end the timer
/// waits on, with a read timeout for its next deadline, and whose
/// nonblocking write end registration changes and the signal handlers each
/// write one byte to. It is never closed, so a byte written while no timer
/// thread reads stays until one does.
#[cfg(not(all(loom, test)))]
pub(crate) struct Wake {
    reader: std::os::unix::net::UnixStream,
    writer: std::os::unix::net::UnixStream,
}

#[cfg(not(all(loom, test)))]
impl Wake {
    pub(crate) fn new() -> Wake {
        let (reader, writer) =
            std::os::unix::net::UnixStream::pair().expect("the timer's wake socket");
        writer
            .set_nonblocking(true)
            .expect("a nonblocking wake socket");
        Wake { reader, writer }
    }

    /// Wakes the timer. A full socket already holds a wake.
    pub(crate) fn notify(&self) {
        use std::io::Write;
        let _ = (&self.writer).write(&[0]);
    }

    /// Blocks until a wake or `timeout`, taking every wake written.
    pub(crate) fn wait(&self, timeout: Option<Duration>) {
        use std::io::Read;
        if timeout == Some(Duration::ZERO) {
            return;
        }
        let _ = self.reader.set_read_timeout(timeout);
        let mut bytes = [0u8; 64];
        let _ = (&self.reader).read(&mut bytes);
    }

    /// Installs the signal handlers, writing to this socket.
    pub(crate) fn install_signals(&self) {
        use std::os::fd::AsRawFd;
        crate::signal::install(self.writer.as_raw_fd());
    }

    /// Whether a signal arrived since the last call.
    pub(crate) fn take_signal(&self) -> bool {
        crate::signal::take_pending()
    }
}

/// The model's wake source: a wake flag and a signal flag under a lock, set
/// by a notify and a signal and taken by a wait and `take_signal`. Timeouts
/// are not modelled, so a wait waits for a notify.
#[cfg(all(loom, test))]
pub(crate) struct Wake {
    state: Mutex<WakeState>,
    arrived: Condvar,
}

#[cfg(all(loom, test))]
#[derive(Default)]
struct WakeState {
    woken: bool,
    signalled: bool,
}

#[cfg(all(loom, test))]
impl Wake {
    pub(crate) fn new() -> Wake {
        Wake {
            state: Mutex::new(WakeState::default()),
            arrived: Condvar::new(),
        }
    }

    pub(crate) fn notify(&self) {
        lock(&self.state).woken = true;
        self.arrived.notify_one();
    }

    pub(crate) fn wait(&self, _timeout: Option<Duration>) {
        let mut state = lock(&self.state);
        while !state.woken {
            state = wait(&self.arrived, state);
        }
        state.woken = false;
    }

    pub(crate) fn install_signals(&self) {}

    /// What a signal handler does: the pending flag, then the wake.
    pub(crate) fn signal(&self) {
        lock(&self.state).signalled = true;
        self.notify();
    }

    pub(crate) fn take_signal(&self) -> bool {
        std::mem::take(&mut lock(&self.state).signalled)
    }
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
