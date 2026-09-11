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

//! `ir_recorded_cases`' recorded expectations, checked against the running oracle
//! rather than against this crate.
//! ```text
//! program not-oracle-bytes=do-with-refused
//! ```

mod support;

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use rexx_exec::{Invocation, Outcome, run_program};
use support::oracle::{Oracle, did_not_finish, locate};

/// The same directory `ir_recorded.rs` walks. Two tests reading one directory is
/// the point: one says this crate matches the recording, the other says the
/// recording matches the interpreter, and neither claim implies the other.
const CASE_DIR: &str = "tests/ir_recorded_cases";

/// The path a case's program is reported as running from. `ir_recorded.rs` records
/// the same literal, because a traceback's middle line names the program file
/// and a temporary path would put this run's process id into an expectation.
const INLINE_PATH: &str = "/nonexistent/ir-dual-case.rex";

/// Env var that flips this file from a skip into the check. `corpus.rs`'s own
/// switch, reused rather than given a spelling of its own.
const GATE_ENV: &str = "REXX_CORPUS_GATE";

/// The directive-line argument marking a stanza whose recorded bytes are
/// deliberately not the oracle's.
const NOT_ORACLE_BYTES: &str = "not-oracle-bytes";

fn gate_mode() -> bool {
    match std::env::var(GATE_ENV) {
        Ok(value) => !value.is_empty() && value != "0",
        Err(_) => false,
    }
}

/// A directory of this run's own: an unresolved Rexx name searches the current
/// directory for an external routine, so a synthesised program run from a
/// directory of unrelated `.rex` files can execute one of them instead of
/// failing. `support::oracle::Oracle::run`'s doc has the measured consequence.
fn fresh_run_dir() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("the clock is after the epoch")
        .as_nanos();
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("ir-dual-oracle-{}-{nanos}", std::process::id()));
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("cannot create {}: {e}", dir.display()));
    dir
}

/// One case's program, rendered in the tagged form the case files record.
fn tagged(exit_code: i32, stdout: &[u8], stderr: &[u8]) -> String {
    let mut out = format!("rc> {exit_code}\n");
    for line in String::from_utf8_lossy(stdout).lines() {
        out.push_str(&format!("out> {line}\n"));
    }
    for line in String::from_utf8_lossy(stderr).lines() {
        out.push_str(&format!("err> {line}\n"));
    }
    out
}

/// The oracle's answer for one program, with the path it ran from rewritten to
/// [`INLINE_PATH`] so a traceback's middle line is comparable.
fn render_oracle(oracle: &Oracle, run_dir: &Path, label: &str, program: &str) -> String {
    let path = run_dir.join("case.rex");
    fs::write(&path, program.as_bytes())
        .unwrap_or_else(|e| panic!("cannot write {}: {e}", path.display()));
    let outcome = oracle.run(&path);
    // An unconditional `assert!` naming the stanza, before `expect_exit_code`
    // below can reach its own path-less panic.
    assert!(
        !did_not_finish(&outcome),
        "{label}: the oracle did not finish: {:?} -- a structural failure, not \
         a byte comparison",
        outcome.termination
    );
    let exit_code = outcome.expect_exit_code();
    let shown = path.to_string_lossy().into_owned();
    let rewrite = |bytes: Vec<u8>| {
        String::from_utf8_lossy(&bytes)
            .replace(&shown, INLINE_PATH)
            .into_bytes()
    };
    tagged(
        exit_code,
        &rewrite(outcome.stdout),
        &rewrite(outcome.stderr),
    )
}

/// This crate's own answer, which is what a `not-oracle-bytes` stanza records.
fn render_crate(program: &str) -> String {
    let Outcome {
        stdout,
        stderr,
        exit_code,
        ..
    } = run_program(INLINE_PATH, program.as_bytes().to_vec(), Invocation::none());
    tagged(exit_code, &stdout, &stderr)
}

/// Every recorded expectation is still what the oracle produces, or is marked
/// as deliberately something else.
#[test]
fn every_recorded_expectation_is_still_what_the_oracle_produces() {
    if !gate_mode() {
        eprintln!("skipped: set {GATE_ENV}=1 to check the recordings against the oracle");
        return;
    }
    assert!(
        std::env::var_os("REWRITE").is_none(),
        "REWRITE would replace oracle-measured expectations with whatever this crate currently \
         prints, which is the one thing they must not be"
    );

    let oracle = locate();
    let run_dir = fresh_run_dir();
    let mut checked = 0usize;
    // Keyed by the reason so the message can name what was claimed, and sorted
    // so a failure reads the same way twice.
    let mut marked: BTreeMap<String, usize> = BTreeMap::new();

    datadriven::walk(CASE_DIR, |file| {
        let filename = file.filename.clone();
        // Reset per file, unlike `checked` below: `datadriven` 0.9.0 walks
        // `fs::read_dir` unsorted, so a running total across every file is
        // neither the stanza's position in *this* file nor stable between
        // two machines -- adding a case file anywhere renumbers everything
        // after it. A within-file count is stable under both.
        let mut in_file = 0usize;
        file.run(|case| {
            assert_eq!(
                case.directive, "program",
                "unknown directive {:?}",
                case.directive
            );
            checked += 1;
            in_file += 1;
            let label = format!("{filename}#{in_file}");
            let reason = case.args.remove(NOT_ORACLE_BYTES).map(|values| {
                assert!(
                    !values.is_empty(),
                    "{NOT_ORACLE_BYTES} must name the mechanism, as                      `{NOT_ORACLE_BYTES}=do-with-refused`, so the marker says what it is waiting                      on rather than only that it is waiting"
                );
                values.join(" ")
            });
            let from_oracle = render_oracle(&oracle, &run_dir, &label, &case.input);
            match reason {
                None => from_oracle,
                Some(reason) => {
                    let ours = render_crate(&case.input);
                    assert_ne!(
                        ours, from_oracle,
                        "this stanza is marked {NOT_ORACLE_BYTES}={reason}, but the oracle now \
                         produces exactly what it records. Drop the marker: leaving it in place \
                         tells the next reader that a live divergence is a deliberate one"
                    );
                    *marked.entry(reason).or_default() += 1;
                    ours
                }
            }
        });
    });

    // A directory that produced no stanzas -- emptied, renamed, or with every
    // directive stopped being recognised -- otherwise passes having run the
    // oracle not at all.
    assert!(
        checked > 0,
        "{CASE_DIR} produced no cases, so this test asserts nothing"
    );
    assert_eq!(
        oracle.invocations(),
        checked,
        "every stanza must reach the oracle exactly once"
    );
    for (reason, count) in &marked {
        eprintln!("{count} stanza(s) recorded as not the oracle's bytes: {reason}");
    }
}
