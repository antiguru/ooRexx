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

//! The signal handlers (D-U3; spec 2026-09-29 section 4). A process entry
//! point that starts Rexx for a user installs them once (ruling P62): SIGINT
//! and SIGTERM where the action is the default or ignored, SIGHUP only where
//! it is the default, never over a handler, without `SA_RESTART`
//! (`platform/unix/SystemInterpreter.cpp:117-145`, ruling P61); SIGPIPE is
//! ignored (`:146-148`). A handler sets [`PENDING`] and writes one byte to
//! the timer's wake socket, nothing else; the timer thread serves it. These
//! statics are the signal half of the timer's wake source.

#![allow(unsafe_code)]

use std::os::fd::RawFd;
use std::sync::Once;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

/// The signals that halt every live interpreter, each with whether it is
/// installed over an ignored action too.
const HALTING: [(libc::c_int, bool); 3] = [
    (libc::SIGINT, true),
    (libc::SIGTERM, true),
    (libc::SIGHUP, false),
];

/// A signal arrived that the timer has not yet served.
static PENDING: AtomicBool = AtomicBool::new(false);

/// The nonblocking write end of the timer's wake socket, which lives as
/// long as the process.
static WAKE: AtomicI32 = AtomicI32::new(-1);

static INSTALL: Once = Once::new();

/// Installs the handlers, once per process, with `wake` as the socket they
/// write to.
pub(crate) fn install(wake: RawFd) {
    INSTALL.call_once(|| {
        WAKE.store(wake, Ordering::Release);
        for (signal, over_ignored) in HALTING {
            install_where_unset(signal, over_ignored);
        }
        // SAFETY: `signal` with `SIG_IGN` installs no code of ours.
        unsafe {
            libc::signal(libc::SIGPIPE, libc::SIG_IGN);
        }
    });
}

/// Installs [`handler`] for `signal` where its action is the default, or
/// ignored where `over_ignored`.
fn install_where_unset(signal: libc::c_int, over_ignored: bool) {
    // SAFETY: both `sigaction` structs are zeroed plain data, which is a
    // valid value for every field, and each pointer passed is to a live
    // local or null. The handler installed is async-signal-safe (see its
    // own note).
    unsafe {
        let mut previous: libc::sigaction = std::mem::zeroed();
        if libc::sigaction(signal, std::ptr::null(), &raw mut previous) != 0 {
            return;
        }
        let unset = previous.sa_sigaction == libc::SIG_DFL
            || over_ignored && previous.sa_sigaction == libc::SIG_IGN;
        if !unset {
            return;
        }
        let mut action: libc::sigaction = std::mem::zeroed();
        action.sa_sigaction = handler as extern "C" fn(libc::c_int) as libc::sighandler_t;
        libc::sigfillset(&raw mut action.sa_mask);
        action.sa_flags = 0;
        libc::sigaction(signal, &raw const action, std::ptr::null_mut());
    }
}

extern "C" fn handler(_: libc::c_int) {
    PENDING.store(true, Ordering::Release);
    let wake = WAKE.load(Ordering::Acquire);
    // SAFETY: async-signal-safe: an atomic load and store, `write(2)` and
    // `errno`'s own location, which the handler saves and restores for the
    // code it interrupted. No allocation, no lock, no interpreter state.
    // `wake` is a nonblocking socket that is never closed, so the write
    // neither blocks nor reaches a reused descriptor; a full socket already
    // holds a wake.
    unsafe {
        let errno = errno_location();
        let saved = *errno;
        libc::write(wake, [0u8].as_ptr().cast(), 1);
        *errno = saved;
    }
}

/// This thread's `errno`.
#[cfg(any(target_os = "linux", target_os = "android"))]
fn errno_location() -> *mut libc::c_int {
    // SAFETY: `__errno_location` has no precondition.
    unsafe { libc::__errno_location() }
}

/// This thread's `errno`.
#[cfg(any(target_vendor = "apple", target_os = "freebsd"))]
fn errno_location() -> *mut libc::c_int {
    // SAFETY: `__error` has no precondition.
    unsafe { libc::__error() }
}

/// This thread's `errno`.
#[cfg(any(target_os = "openbsd", target_os = "netbsd"))]
fn errno_location() -> *mut libc::c_int {
    // SAFETY: `__errno` has no precondition.
    unsafe { libc::__errno() }
}

/// Changes how this thread's mask treats the halting signals.
fn mask(how: libc::c_int) {
    // SAFETY: the set is initialised by `sigemptyset` before it is read, and
    // both pointers are to a live local or null.
    unsafe {
        let mut set: libc::sigset_t = std::mem::zeroed();
        libc::sigemptyset(&raw mut set);
        for (signal, _) in HALTING {
            libc::sigaddset(&raw mut set, signal);
        }
        libc::pthread_sigmask(how, &raw const set, std::ptr::null_mut());
    }
}

/// Blocks the halting signals in this thread, so the kernel delivers them
/// only to a thread that takes them (ruling P67).
pub(crate) fn block() {
    mask(libc::SIG_BLOCK);
}

/// Unblocks the halting signals in this thread where the handlers are
/// installed: for the interpreter's own thread, which runs Rexx and drains
/// what a blocking wait posts, so a handler it takes has run before it
/// files that post.
pub(crate) fn receive() {
    if INSTALL.is_completed() {
        mask(libc::SIG_UNBLOCK);
    }
}

/// Runs `wait` with the halting signals unblocked in this thread where the
/// handlers are installed: a signal then interrupts the wait, and its
/// handler has run before the wait returns.
pub(crate) fn unblocked<R>(wait: impl FnOnce() -> R) -> R {
    struct Reblock;
    impl Drop for Reblock {
        fn drop(&mut self) {
            block();
        }
    }
    if !INSTALL.is_completed() {
        return wait();
    }
    mask(libc::SIG_UNBLOCK);
    let _reblock = Reblock;
    wait()
}

/// Whether a signal arrived since the last call.
pub(crate) fn take_pending() -> bool {
    PENDING
        .compare_exchange(true, false, Ordering::AcqRel, Ordering::Acquire)
        .is_ok()
}

#[cfg(test)]
mod tests {
    /// Running a program installs no handler: only a process entry point
    /// does (ruling P62), so this test binary keeps its dispositions.
    #[test]
    fn running_a_program_installs_no_handler() {
        let outcome = crate::run_program(
            "/no/such/dir/t.rex",
            b"say 'ran'".to_vec(),
            crate::Invocation::none(),
        );
        assert_eq!(outcome.stdout, b"ran\n");
        assert!(!super::INSTALL.is_completed());
    }

    /// Set in the process a test runs its case in alone.
    const ALONE: &str = "REXX_SIGNAL_CASE_ALONE";

    /// Whether this is the process the test `name` runs its case in: a halt
    /// reaches every live interpreter in the process. Elsewhere it runs that
    /// process and asserts it passed.
    fn alone(name: &str) -> bool {
        if std::env::var_os(ALONE).is_some() {
            return true;
        }
        let output = std::process::Command::new(std::env::current_exe().expect("this binary"))
            .args(["--exact", name, "--test-threads=1"])
            .env(ALONE, "1")
            .output()
            .expect("the case's process");
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(output.status.success(), "{stdout}");
        assert!(stdout.contains("1 passed"), "{stdout}");
        false
    }

    /// A signal whose handler ran as a command's child ended halts that
    /// command's clause, before the timer thread posts it. The flag is set
    /// with no wake written, so the timer thread does not take it.
    #[test]
    fn a_signal_pending_at_a_commands_end_halts_it() {
        if !alone("signal::tests::a_signal_pending_at_a_commands_end_halts_it") {
            return;
        }
        // Off the baton, and redirected, which waits on it.
        for command in [
            "address system 'sleep 0.5'",
            "address system 'sleep 0.5' with output stem o.",
        ] {
            let raiser = std::thread::spawn(|| {
                std::thread::sleep(std::time::Duration::from_millis(200));
                super::PENDING.store(true, super::Ordering::Release);
            });
            let outcome = crate::run_program(
                "/no/such/dir/t.rex",
                format!("say 'a'\n{command}\nsay 'b' rc\n").into_bytes(),
                crate::Invocation::none(),
            );
            raiser.join().expect("the raiser");
            assert_eq!(String::from_utf8_lossy(&outcome.stdout), "a\n", "{command}");
            assert_eq!(
                String::from_utf8_lossy(&outcome.stderr),
                format!(
                    "     2 *-* {command}\n\
                     Error 4 running /no/such/dir/t.rex line 2:  Program interrupted.\n\
                     Error 4.1:  Program interrupted with HALT condition.\n"
                )
            );
        }
    }

    /// A halt readies a deadline-woken activity that a nested loop has set
    /// aside only once: main's timed wait in an `INTERPRET` is due while
    /// another activity's pinned wait runs a busy one, and a second entry
    /// would end main's later `SysSleep` at once. The halt queued in the
    /// `INTERPRET` runs no handler, as on the oracle.
    #[test]
    fn a_halt_readies_a_set_aside_activity_once() {
        if !alone("signal::tests::a_halt_readies_a_set_aside_activity_once") {
            return;
        }
        let raiser = std::thread::spawn(|| {
            std::thread::sleep(std::time::Duration::from_secs(1));
            super::PENDING.store(true, super::Ordering::Release);
            crate::timer::serve_signal();
        });
        let outcome = crate::run_program(
            "/no/such/dir/t.rex",
            b"s = .EventSemaphore~new\nu = .EventSemaphore~new\ncall on halt name mh\n\
              b = .b~new; b~start('go', u)\nc = .c~new; c~start('go', u)\n\
              interpret 'r = s~wait(0.3)'\ncall time 'R'\ncall SysSleep 1\n\
              say 'slept' (time('E') >= 1)\nexit\nmh: say 'mh'; return\n\
              ::class b\n::method go\n  use arg u\n  call on halt name bh\n\
              interpret 'u~wait'\n  return\nbh: return\n\
              ::class c\n::method go\n  use arg u\n  call on halt name ch\n  call time 'R'\n\
              do forever; if time('E') >= 2 then leave; end\n  u~post\n  return\nch: return\n"
                .to_vec(),
            crate::Invocation::none(),
        );
        raiser.join().expect("the raiser");
        assert_eq!(String::from_utf8_lossy(&outcome.stdout), "slept 1\n");
        assert_eq!(String::from_utf8_lossy(&outcome.stderr), "");
    }
}
