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

//! The signal handlers (D-U3; spec 2026-09-29 section 4). SIGINT, SIGTERM
//! and SIGHUP each get one at the first interpreter start, where that signal
//! has no handler and is not ignored, without `SA_RESTART`
//! (`platform/unix/SystemInterpreter.cpp:117-145`); SIGPIPE is ignored
//! (`:146-148`). A handler sets [`PENDING`] and writes one byte to the
//! timer's wake socket, nothing else; the timer thread serves it. These
//! statics are the signal half of the timer's wake source.

#![allow(unsafe_code)]

use std::os::fd::RawFd;
use std::sync::Once;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};

/// The signals that halt every live interpreter.
const HALTING: [libc::c_int; 3] = [libc::SIGINT, libc::SIGTERM, libc::SIGHUP];

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
        for signal in HALTING {
            install_where_unset(signal);
        }
        // SAFETY: `signal` with `SIG_IGN` installs no code of ours.
        unsafe {
            libc::signal(libc::SIGPIPE, libc::SIG_IGN);
        }
    });
}

/// Installs [`handler`] for `signal` where its action is the default.
fn install_where_unset(signal: libc::c_int) {
    // SAFETY: both `sigaction` structs are zeroed plain data, which is a
    // valid value for every field, and each pointer passed is to a live
    // local or null. The handler installed is async-signal-safe (see its
    // own note).
    unsafe {
        let mut previous: libc::sigaction = std::mem::zeroed();
        if libc::sigaction(signal, std::ptr::null(), &raw mut previous) != 0
            || previous.sa_sigaction != libc::SIG_DFL
        {
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
    PENDING.store(true, Ordering::SeqCst);
    let wake = WAKE.load(Ordering::Acquire);
    // SAFETY: async-signal-safe: an atomic load and store, `write(2)` and
    // `errno`'s own location, which the handler saves and restores for the
    // code it interrupted. No allocation, no lock, no interpreter state.
    // `wake` is a nonblocking socket that is never closed, so the write
    // neither blocks nor reaches a reused descriptor; a full socket already
    // holds a wake.
    unsafe {
        let errno = libc::__errno_location();
        let saved = *errno;
        libc::write(wake, [0u8].as_ptr().cast(), 1);
        *errno = saved;
    }
}

/// Whether a signal arrived since the last call.
pub(crate) fn take_pending() -> bool {
    PENDING.swap(false, Ordering::SeqCst)
}
