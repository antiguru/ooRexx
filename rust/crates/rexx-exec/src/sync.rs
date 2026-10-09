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

/// The timer thread's wake source: a socket pair and, on Linux, a timerfd.
/// The timer thread polls the socket's read end and the timerfd, armed for
/// its next deadline; on other unix targets the deadline is the poll's
/// timeout. Registration changes and the signal handlers each write one
/// byte to the socket's nonblocking write end. The socket is never closed,
/// so a byte written while no timer thread reads stays until one does.
#[cfg(not(all(loom, test)))]
pub(crate) struct Wake {
    reader: std::os::unix::net::UnixStream,
    writer: std::os::unix::net::UnixStream,
    #[cfg(any(target_os = "linux", target_os = "android"))]
    timer: std::os::fd::OwnedFd,
}

#[cfg(not(all(loom, test)))]
impl Wake {
    pub(crate) fn new() -> Wake {
        let (reader, writer) =
            std::os::unix::net::UnixStream::pair().expect("the timer's wake socket");
        for end in [&reader, &writer] {
            end.set_nonblocking(true)
                .expect("a nonblocking wake socket");
        }
        Wake {
            reader,
            writer,
            #[cfg(any(target_os = "linux", target_os = "android"))]
            timer: rustix::time::timerfd_create(
                rustix::time::TimerfdClockId::Monotonic,
                rustix::time::TimerfdFlags::NONBLOCK | rustix::time::TimerfdFlags::CLOEXEC,
            )
            .expect("the timer's deadline timer"),
        }
    }

    /// Wakes the timer. A full socket already holds a wake.
    pub(crate) fn notify(&self) {
        use std::io::Write;
        let _ = (&self.writer).write(&[0]);
    }

    /// Blocks until a wake, `timeout`, or an interrupted poll, taking every
    /// wake written. The caller recomputes its deadline on every return.
    pub(crate) fn wait(&self, timeout: Option<Duration>) {
        use rustix::event::{PollFd, PollFlags, poll};
        use std::io::Read;
        if timeout == Some(Duration::ZERO) {
            return;
        }
        #[cfg(any(target_os = "linux", target_os = "android"))]
        {
            use rustix::time::{Itimerspec, TimerfdTimerFlags, Timespec, timerfd_settime};
            let zero = Timespec {
                tv_sec: 0,
                tv_nsec: 0,
            };
            let deadline = Itimerspec {
                it_interval: zero,
                it_value: timeout
                    .and_then(|timeout| Timespec::try_from(timeout).ok())
                    .unwrap_or(zero),
            };
            let _ = timerfd_settime(&self.timer, TimerfdTimerFlags::empty(), &deadline);
            let mut polled = [
                PollFd::new(&self.reader, PollFlags::IN),
                PollFd::new(&self.timer, PollFlags::IN),
            ];
            let _ = poll(&mut polled, None);
            let _ = rustix::io::read(&self.timer, &mut [0u8; 8]);
        }
        #[cfg(not(any(target_os = "linux", target_os = "android")))]
        {
            // `poll(2)`'s timeout is a C int of milliseconds, so a wait is
            // capped at a day and the caller recomputes after it.
            let timeout = timeout
                .map(|timeout| timeout.min(Duration::from_secs(86_400)))
                .and_then(|timeout| rustix::time::Timespec::try_from(timeout).ok());
            let _ = poll(
                &mut [PollFd::new(&self.reader, PollFlags::IN)],
                timeout.as_ref(),
            );
        }
        let _ = (&self.reader).read(&mut [0u8; 64]);
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

/// The model's wake source: a wake flag under a lock, set by a notify and
/// taken by a wait, and the signal flag as the handler and the timer use it,
/// a lock-free atomic. Timeouts are not modelled, so a wait waits for a
/// notify.
#[cfg(all(loom, test))]
pub(crate) struct Wake {
    woken: Mutex<bool>,
    arrived: Condvar,
    signalled: loom::sync::atomic::AtomicBool,
}

#[cfg(all(loom, test))]
impl Wake {
    pub(crate) fn new() -> Wake {
        Wake {
            woken: Mutex::new(false),
            arrived: Condvar::new(),
            signalled: loom::sync::atomic::AtomicBool::new(false),
        }
    }

    pub(crate) fn notify(&self) {
        *lock(&self.woken) = true;
        self.arrived.notify_one();
    }

    pub(crate) fn wait(&self, _timeout: Option<Duration>) {
        let mut woken = lock(&self.woken);
        while !*woken {
            woken = wait(&self.arrived, woken);
        }
        *woken = false;
    }

    /// What a signal handler does: the pending flag, then the wake.
    pub(crate) fn signal(&self) {
        self.signalled.store(true, Ordering::Release);
        self.notify();
    }

    /// The shipped take, `signal::take_pending`'s.
    pub(crate) fn take_signal(&self) -> bool {
        self.signalled
            .compare_exchange(true, false, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
    }
}

/// The timer thread's clock, which slices and wakes on real time.
#[cfg(not(all(loom, test)))]
#[expect(
    clippy::disallowed_methods,
    reason = "the timer thread, which the simulation mode does not arm"
)]
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

#[cfg(all(test, not(loom)))]
#[expect(clippy::disallowed_methods, reason = "these tests time real waits")]
mod tests {
    use super::Wake;
    use std::time::{Duration, Instant};

    /// A timed wait with nothing written ends at its timeout: the best of
    /// three 300 ms waits is under 1 ms late. Measured at niceness 5, a
    /// socket read timeout was 15 to 24 ms late and a poll timeout about
    /// 1.5 ms.
    #[test]
    fn a_timed_wait_ends_close_to_its_timeout() {
        let wake = Wake::new();
        let timeout = Duration::from_millis(300);
        let late = (0..3)
            .map(|_| {
                let began = Instant::now();
                wake.wait(Some(timeout));
                let waited = began.elapsed();
                assert!(waited >= timeout, "a wait ended early, after {waited:?}");
                waited - timeout
            })
            .min()
            .expect("three waits");
        assert!(late < Duration::from_millis(1), "{late:?} late");
    }

    /// A wake written before the wait ends it at once, and the wait takes
    /// it, so the next wait runs to its timeout.
    #[test]
    fn a_written_wake_ends_one_wait() {
        let wake = Wake::new();
        wake.notify();
        let began = Instant::now();
        wake.wait(Some(Duration::from_secs(5)));
        assert!(began.elapsed() < Duration::from_secs(1));
        let began = Instant::now();
        wake.wait(Some(Duration::from_millis(50)));
        assert!(began.elapsed() >= Duration::from_millis(50));
    }
}
