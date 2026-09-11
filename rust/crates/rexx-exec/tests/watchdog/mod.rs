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

//! The outer bound on one in-process run, so that a program which does not
//! terminate reddens its own row instead of stalling a sweep.
//!
//! This file stays under a directory for `tests/support/mod.rs`'s reason: a
//! bare `tests/watchdog.rs` would be auto-discovered as a test binary of its
//! own as well as being pulled in by the files that declare `mod watchdog;`.
//!
//! # Two layers, and which one catches what
//!
//! [`run_bounded`] arms both, and they are not interchangeable.
//!
//! * **Layer 1** is [`Invocation::with_deadline`], honoured by the
//!   interpreter at the clause boundary. It catches a run that keeps
//!   executing clauses, which is the one crate-side non-termination this
//!   project has found. It returns a real [`Outcome`]: the program's own
//!   stdout up to the cut, [`DEADLINE_REPORT`] on stderr and
//!   [`DEADLINE_EXIT`] as the status, and the thread running it is finished
//!   and joined. `rexx_exec::clause::Deadline`'s own doc names what it cannot
//!   see -- a park, a spin inside one clause or one builtin, and the parse.
//! * **Layer 2** is this file: a bound on the wait, not on the run. It
//!   catches anything layer 1 cannot see, because it is not looking at the
//!   interpreter at all -- it gives up on the channel and synthesises
//!   [`abandoned`]'s outcome.
//!
//! **Neither layer survives a native stack overflow**, and one is reachable
//! from Rexx: measured, `zs = 'interpret zs'` then `interpret zs` aborts the
//! process at rc 134 with no deadline set. Layer 2 abandons a thread; it
//! cannot outlive the process the thread takes with it. A deadline does cut
//! that program first, so under `run_bounded` the row reddens -- but that is
//! layer 1 winning a race, not layer 2 covering the case.
//!
//! # What layer 2 costs, and it is not free
//!
//! **A Rust thread cannot be killed.** Giving up on the wait abandons the
//! thread, and it keeps running: at 100% of a core for a spin, competing for
//! a core with the rest of the sweep for as long as the test binary lives. It
//! also keeps the [`INTERPRETER_STACK_BYTES`] reservation its own interpreter
//! thread made. The process does still exit -- a detached thread does not
//! hold `main` open -- and `deadline.rs`'s
//! `layer_two_returns_for_a_run_layer_one_is_not_watching` is what witnesses
//! that: it abandons a spinning thread, and the binary it runs in has to
//! reach its own exit for the suite to report at all.
//!
//! **Layer 2 is defensive, and against a real crate-side case it is
//! untested.** No parked crate-side shape is known: the parked shape the
//! oracle has, `GUARD ... WHEN` with a false expression
//! (`corpus/oracle-crashes.txt` entry 7), is refused here at
//! `Loud::guard_when_false` before it can park. It is reached in
//! `deadline.rs` by giving layer 1 no deadline at all, which proves the
//! mechanism and not that the mechanism has a subject.
//!
//! [`INTERPRETER_STACK_BYTES`]: rexx_exec::INTERPRETER_STACK_BYTES

#![allow(dead_code)]

use std::sync::mpsc;
use std::time::Duration;

use rexx_exec::{DEADLINE_EXIT, Invocation, Outcome, StackSpan, run_program};

/// Layer 1's bound on one sweep row.
///
/// **Chosen from the slowest row anyone has measured, not from taste.**
/// Measured 2026-09-02 with this function itself timed and printing every row
/// over 50 ms, under `REXX_CORPUS_GATE=1 memcap 8G cargo test --test corpus
/// --test assertions --test bif_assertions --test ir_recorded -p rexx-exec
/// --no-fail-fast`: 51,084 rows above that floor, median 154 ms, p99 196 ms,
/// **slowest 294 ms** -- an assertion row, with the corpus's own programs
/// below it. Separately, debug `rexx-run` over the 331 programs the corpus
/// differential runs, wall clock including process startup, was 198 ms run
/// one at a time and 299 ms with 32 running at once.
///
/// This is two orders of magnitude above that, because the cost of a value
/// too large is paid only by a row that was going to hang anyway, and the
/// cost of one too small is a green program reported as a hang.
pub const ROW_DEADLINE: Duration = Duration::from_secs(60);

/// Layer 2's bound on the wait for one sweep row.
///
/// Above [`ROW_DEADLINE`] so that layer 1 answers first whenever it can see
/// the run at all: layer 1 leaves no thread behind and this does.
pub const ROW_ABANDON: Duration = Duration::from_secs(90);

/// [`run_program`], bounded both ways.
///
/// The `Outcome` is either the run's own -- including the one layer 1 makes
/// for a run it cut -- or [`abandoned`]'s, and the two are told apart by
/// their stderr. Both are non-zero statuses, so a caller that classifies on
/// the exit code alone reddens the row either way.
///
/// **A panic is not a timeout and must never be reported as one.** A panic
/// inside `run_program` is resumed on the thread below, which drops the
/// sender; that arrives here as a disconnect rather than as a timeout, and it
/// is re-panicked. Swallowing it would turn a bug in this crate into a row
/// that merely looks slow.
pub fn run_bounded(path: &str, text: Vec<u8>, invocation: Invocation) -> Outcome {
    run_bounded_with(
        path,
        text,
        invocation.with_deadline(ROW_DEADLINE),
        ROW_ABANDON,
    )
}

/// [`run_bounded`] over durations of the caller's choosing, and with layer 1
/// left to whatever `invocation` already says.
///
/// The shape a test reaches for. Both arms are otherwise unreachable in
/// bounded time: layer 1's needs a program that outlives its deadline, and
/// layer 2's needs one layer 1 is not watching.
pub fn run_bounded_with(
    path: &str,
    text: Vec<u8>,
    invocation: Invocation,
    abandon: Duration,
) -> Outcome {
    let (sender, receiver) = mpsc::channel();
    let owned = path.to_string();
    std::thread::Builder::new()
        .name("rexx-watchdog".to_string())
        .spawn(move || {
            let outcome = run_program(&owned, text, invocation);
            // The receiver is gone for a run this side already gave up on,
            // and that send is the one that is allowed to fail.
            let _ = sender.send(outcome);
        })
        .expect("spawning the watchdog thread");
    match receiver.recv_timeout(abandon) {
        Ok(outcome) => outcome,
        Err(mpsc::RecvTimeoutError::Timeout) => abandoned(path, abandon),
        Err(mpsc::RecvTimeoutError::Disconnected) => panic!(
            "the interpreter thread running {path} panicked; its own message is above this one"
        ),
    }
}

/// The outcome of a run this harness gave up waiting for.
///
/// `stdout` is empty because there is nothing to read: the bytes the run
/// produced are in a buffer owned by the thread still holding them. That is a
/// property of the abandonment and not a claim that the program printed
/// nothing.
pub fn abandoned(path: &str, after: Duration) -> Outcome {
    Outcome {
        exit_code: DEADLINE_EXIT,
        stdout: Vec::new(),
        stderr: format!("rexx-watchdog: {path} was abandoned after {after:?}\n").into_bytes(),
        stack: StackSpan::default(),
        collections: 0,
        chunks_refused: 0,
    }
}

/// Whether `outcome` is a run that did not finish -- either layer's.
///
/// Reads the stderr rather than the status, because the status is a number a
/// program can name for itself ([`DEADLINE_EXIT`]'s own doc) and these two
/// messages are not.
///
/// **`ends_with` for layer 1 and `starts_with` for layer 2**, because that is
/// where each message lands: `execute` appends its report *after* whatever
/// the program already wrote to stderr, so a traced program that outlives its
/// deadline has trace lines in front of it, and layer 2's outcome is
/// synthesised here with nothing else in it.
pub fn did_not_finish(outcome: &Outcome) -> bool {
    outcome.stderr.ends_with(rexx_exec::DEADLINE_REPORT)
        || outcome.stderr.starts_with(b"rexx-watchdog: ")
}
