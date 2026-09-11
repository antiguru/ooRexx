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

#![allow(dead_code)]

use std::sync::mpsc;
use std::time::Duration;

use rexx_exec::{DEADLINE_EXIT, Invocation, Outcome, StackSpan, run_program};

/// Layer 1's bound on one sweep row.
pub const ROW_DEADLINE: Duration = Duration::from_secs(60);

/// Layer 2's bound on the wait for one sweep row.
pub const ROW_ABANDON: Duration = Duration::from_secs(90);

/// [`run_program`], bounded both ways.
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
pub fn did_not_finish(outcome: &Outcome) -> bool {
    outcome.stderr.ends_with(rexx_exec::DEADLINE_REPORT)
        || outcome.stderr.starts_with(b"rexx-watchdog: ")
}
