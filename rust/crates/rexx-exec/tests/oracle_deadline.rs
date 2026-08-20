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

//! A program that never finishes reddens the oracle harness at its
//! deadline, rather than hanging it.
//!
//! # What this proves and what it does not
//!
//! `do forever; end` is run through `support::oracle::Oracle::run` -- the
//! same entry point every other differential test in this crate uses -- and
//! the assertion is that the call **returns** with
//! [`Termination::TimedOut`], inside a bounded amount of wall time, rather
//! than blocking forever the way `Command::output` (what `Oracle::run`
//! called directly, with no deadline) would have.
//!
//! **The before state**, for comparison, measured by hand rather than in
//! this suite -- there is no surviving code path that still calls
//! `Command::output` without a deadline for this file to run as a
//! counter-test: the same program, wrapped the way
//! `support::oracle::locate`'s doc and `rust/CLAUDE.md`'s "Wrap every oracle
//! run" both give (`( ulimit -v 1048576; LD_LIBRARY_PATH=.../lib timeout -s
//! KILL 10 .../bin/rexx do-forever.rex )`), ran the full ten seconds and was
//! killed by the external `timeout`: rc 137, both channels empty.
//!
//! This does not prove the crate side of a gate-table row can be bounded:
//! `Invocation::with_engine` runs **in-process**, and an in-process run
//! cannot be killed by anything short of the whole test process dying with
//! it. That half stays a human check: every probe program a later task
//! commits is run by hand through `rexx-run` under `timeout -s KILL 10`
//! first, and the task says it did. Nothing here changes that.
//!
//! # The gate
//!
//! Gated on `REXX_CORPUS_GATE`, the switch every oracle-invoking harness in
//! this crate uses, for the reason they all give: an offline checkout is not
//! asked to produce an oracle. Here it also keeps the ~10-second wait for the
//! deadline to fire out of a plain `cargo test`, where nothing else in this
//! crate deliberately runs that long.

mod support;

use std::fs;
use std::path::PathBuf;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use support::oracle::{ORACLE_DEADLINE, Termination, did_not_finish};

const GATE_ENV: &str = "REXX_CORPUS_GATE";

fn gate_mode() -> bool {
    match std::env::var(GATE_ENV) {
        Ok(value) => !value.is_empty() && value != "0",
        Err(_) => false,
    }
}

/// A directory of this run's own -- `support::oracle::Oracle::run`'s doc has
/// the measured consequence of running a synthesised program from a
/// directory of stale `.rex` files.
fn fresh_run_dir() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("the clock is after the epoch")
        .as_nanos();
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("oracle-deadline-{}-{nanos}", std::process::id()));
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("cannot create {}: {e}", dir.display()));
    dir
}

/// `do forever; end` reddens at [`ORACLE_DEADLINE`] instead of hanging the
/// suite. The one program in this file, and the whole point of it: a
/// controlled loop with no terminating condition, which is exactly the shape
/// `rust/CLAUDE.md`'s probe rules warn a *rounded-away step* can produce by
/// accident and this constructs on purpose.
#[test]
fn a_program_that_never_finishes_reddens_at_the_deadline_instead_of_hanging() {
    if !gate_mode() {
        eprintln!(
            "*** SKIPPED -- the deadline verification needs the oracle and \
             runs only with {GATE_ENV} set, since it also spends \
             ORACLE_DEADLINE's own ~10 seconds proving the kill fires. ***"
        );
        return;
    }

    let dir = fresh_run_dir();
    let file = dir.join("hang.rex");
    fs::write(&file, b"do forever\nend\n")
        .unwrap_or_else(|e| panic!("cannot write {}: {e}", file.display()));
    let abs = fs::canonicalize(&file)
        .unwrap_or_else(|e| panic!("cannot resolve {}: {e}", file.display()));

    let oracle = support::oracle::locate();
    let started = Instant::now();
    let outcome = oracle.run(&abs);
    let elapsed = started.elapsed();

    assert_eq!(
        outcome.termination,
        Termination::TimedOut,
        "a program that never exits must be classified as the harness's own \
         kill, not a signal death or (impossibly) a normal exit; got {:?}",
        outcome.termination
    );
    assert!(
        did_not_finish(&outcome),
        "did_not_finish must agree with the classification above -- the \
         same fact read through the helper a caller actually reaches for"
    );
    assert!(
        outcome.stdout.is_empty() && outcome.stderr.is_empty(),
        "`do forever; end` writes nothing to either channel before being \
         killed; a non-empty channel here would mean the harness ran a \
         different program than the one written above: stdout={:?} \
         stderr={:?}",
        outcome.stdout,
        outcome.stderr
    );
    assert!(
        elapsed >= ORACLE_DEADLINE,
        "the run returned after {elapsed:?}, before its own {ORACLE_DEADLINE:?} \
         deadline -- the kill fired early, or nothing here actually waited"
    );
    // A generous ceiling, not a tight one: the point is "returned at all"
    // rather than "returned exactly on schedule". A run that took ten times
    // its own deadline to come back after being killed would still prove the
    // classification above, but it would no longer be true that this test
    // "reddens at the deadline instead of hanging" -- it would merely have
    // stopped hanging eventually.
    assert!(
        elapsed < ORACLE_DEADLINE * 3,
        "the run took {elapsed:?} to return after being killed at \
         {ORACLE_DEADLINE:?} -- reaping or the reader threads are stuck \
         rather than the deadline having actually bounded the wait"
    );

    assert_eq!(
        oracle.invocations(),
        1,
        "the run above must have actually started the oracle"
    );

    let _ = fs::remove_dir_all(&dir);
}
