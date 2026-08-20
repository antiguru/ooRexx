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
//! than blocking forever the way an undeadlined `Command::output` does. The
//! control this is measured against -- the same program with no deadline
//! anywhere in the path -- is measured by hand, not run here, and is in the
//! task's own report rather than in this file.
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
//! Gated on `REXX_CORPUS_GATE`, but not for the reason that gates most of
//! this crate's oracle-invoking harnesses: `corpus.rs`, `builtin_status.rs`
//! and `state_builtin_oracle.rs` put every program they own through
//! `wait_with_deadline` on a plain `cargo test` already, gating only their
//! own assertion, so the deadline **mechanism** -- a broken poll loop, a
//! misclassified normal exit -- would already redden ungated. What only
//! this file checks, and only under the gate, is that the **kill** itself
//! fires: proving that costs the full `ORACLE_DEADLINE`, which is worth
//! keeping out of a plain `cargo test`, where nothing else in this crate
//! deliberately runs that long.

mod support;

use std::fs;
use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

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

/// A program whose only "problem" is a background process holding its own
/// end of the pipe open still returns promptly, rather than blocking for
/// that process's own lifetime the way an unbounded `read_to_end` would.
///
/// `address system 'sleep 30 &'` forks a background `sleep` that inherits
/// the same stdout descriptor and then keeps running after `rexx` itself
/// exits at `say 'done'`; nothing kills that descendant, since `Child`
/// tracks only the direct process, so without a bound of its own on the
/// read this call blocks for the descendant's own lifetime -- thirty
/// seconds here, and unboundedly for a probe that backgrounds something
/// longer-lived. `Termination::TimedOut` rather than `Exited(0)` is the
/// right answer too: the transcript could not be read in full, so this run
/// is a structural failure to whatever calls it, not a byte comparison
/// against a `done\n` that in fact reached the pipe but was never
/// retrieved.
#[test]
fn a_background_process_holding_the_pipe_open_does_not_hang_the_run() {
    if !gate_mode() {
        eprintln!(
            "*** SKIPPED -- needs the oracle and runs only with {GATE_ENV} \
             set, since it spends most of ORACLE_DEADLINE proving the read \
             gives up rather than blocking for the backgrounded process's \
             own lifetime. ***"
        );
        return;
    }

    let dir = fresh_run_dir();
    let file = dir.join("orphan.rex");
    fs::write(&file, b"address system 'sleep 30 &'\nsay 'done'\n")
        .unwrap_or_else(|e| panic!("cannot write {}: {e}", file.display()));
    let abs = fs::canonicalize(&file)
        .unwrap_or_else(|e| panic!("cannot resolve {}: {e}", file.display()));

    let oracle = support::oracle::locate();
    let started = Instant::now();
    let outcome = oracle.run(&abs);
    let elapsed = started.elapsed();

    // The bound that would fail without the fix this test proves: the
    // backgrounded `sleep` runs thirty seconds, so a read joined
    // unconditionally on its pipe returns no sooner than that. Comfortably
    // under it, and comfortably over the read's own bounded budget, so this
    // is not a race against either number.
    assert!(
        elapsed < Duration::from_secs(20),
        "the run took {elapsed:?} to return -- a program that only leaves a \
         background process holding its own pipe open must not block for \
         that process's own lifetime (30s here)"
    );
    assert_eq!(
        outcome.termination,
        Termination::TimedOut,
        "the transcript could not be read in full within the deadline, so \
         this must read as a non-finish rather than as the direct \
         process's own successful exit; got {:?}",
        outcome.termination
    );
    assert!(
        did_not_finish(&outcome),
        "did_not_finish must agree with the classification above"
    );

    assert_eq!(
        oracle.invocations(),
        1,
        "the run above must have actually started the oracle"
    );

    // The backgrounded `sleep` holds a pipe descriptor, not anything under
    // `dir` itself, so removing the directory here is unrelated to the
    // process this test deliberately leaves running.
    let _ = fs::remove_dir_all(&dir);
}
