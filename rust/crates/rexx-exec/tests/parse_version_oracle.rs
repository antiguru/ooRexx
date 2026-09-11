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

//! `PARSE VERSION`'s string, checked against the running oracle rather than
//! against a copy of itself.

mod support;

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use support::oracle::{descriptor_diffs, did_not_finish};

/// Env var that flips this file from a skip into the check. `corpus.rs`'s own
/// switch, deliberately reused rather than given a fourth spelling.
const GATE_ENV: &str = "REXX_CORPUS_GATE";

fn gate_mode() -> bool {
    match std::env::var(GATE_ENV) {
        Ok(value) => !value.is_empty() && value != "0",
        Err(_) => false,
    }
}

/// A directory of this run's own, because an unresolved Rexx name searches the
/// current directory for an external routine -- `support::oracle::Oracle::run`'s
/// own doc comment has the measured consequence of running a synthesised
/// program from a directory of stale `.rex` files.
fn fresh_run_dir() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("the clock is after the epoch")
        .as_nanos();
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("parse-version-{}-{nanos}", std::process::id()));
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("cannot create {}: {e}", dir.display()));
    dir
}

/// `PARSE VERSION` answers what the oracle answers, or this crate's recorded
/// constant has gone stale against the built interpreter.
#[test]
fn parse_version_still_answers_what_the_oracle_answers() {
    if !gate_mode() {
        eprintln!(
            "*** SKIPPED -- PARSE VERSION's string is not compared against the \
             oracle unless {GATE_ENV} is set. Nothing else in the tree guards \
             it, so a green run here is not evidence the constant is current. ***"
        );
        return;
    }

    let dir = fresh_run_dir();
    let file = dir.join("parse_version.rex");
    fs::write(&file, b"parse version v\nsay v\n")
        .unwrap_or_else(|e| panic!("cannot write {}: {e}", file.display()));
    let abs = fs::canonicalize(&file)
        .unwrap_or_else(|e| panic!("cannot resolve {}: {e}", file.display()));
    let path = abs
        .to_str()
        .unwrap_or_else(|| panic!("probe path {} is not valid UTF-8", abs.display()));

    let oracle = support::oracle::locate();
    let text = fs::read(&abs).unwrap_or_else(|e| panic!("cannot read {}: {e}", abs.display()));
    let rust = rexx_exec::run_program(path, text, rexx_exec::Invocation::none());
    let cpp = oracle.run(&abs);

    // Named even though this file runs exactly one program: `descriptor_diffs`
    // would otherwise reach `expect_exit_code`'s path-less panic, and an
    // unconditional `assert!` naming the program is the shape this crate's
    // other oracle-invoking harnesses use for the same event.
    assert!(
        !did_not_finish(&cpp),
        "PARSE VERSION's oracle run did not finish: {:?} -- a structural \
         failure, not a byte comparison",
        cpp.termination
    );

    let diffs = descriptor_diffs(&rust, &cpp);
    assert!(
        diffs.is_empty(),
        "PARSE VERSION disagrees with the oracle on [{}]. This crate's \
         parse_template.rs VERSION constant is a recorded measurement of the \
         oracle's own build identity -- name, language level, build date -- and \
         a rebuilt oracle moves the date. Re-measure rather than editing this \
         test:\n  rust:   {:?}\n  oracle: {:?}\n  exit: rust {} oracle {}",
        diffs.join(", "),
        String::from_utf8_lossy(&rust.stdout),
        String::from_utf8_lossy(&cpp.stdout),
        rust.exit_code,
        cpp.expect_exit_code()
    );
    assert_eq!(
        oracle.invocations(),
        1,
        "the comparison above must have actually started the oracle; a check \
         that compares this crate's answer against nothing is the shape this \
         file exists to replace"
    );

    // Only on success: a failing run's file is the evidence.
    let _ = fs::remove_dir_all(&dir);
}
