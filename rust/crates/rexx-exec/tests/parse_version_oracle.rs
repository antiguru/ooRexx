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
//!
//! # Why this is a harness of its own and not a unit test
//!
//! `parse_template.rs`'s `VERSION` is the only value in this crate that is
//! **the oracle's own build identity**: interpreter name and version, language
//! level, and the interpreter's *build date*. Nothing in this crate can derive
//! the third field, so the constant is a recorded measurement -- and a
//! recorded measurement of a binary that gets rebuilt needs something that
//! notices when the binary moves.
//!
//! An assertion inside the crate cannot notice. The only thing such an
//! assertion can compare the constant against is a second copy of the same
//! bytes, which goes red when someone edits one and forgets the other and
//! stays green through every rebuild there has ever been -- the "test that
//! cannot fail" shape `rust/CLAUDE.md`'s Method section records as having
//! shipped repeatedly here, guarding a staleness that would in fact be
//! guarded by nothing.
//!
//! The check below runs `parse version v ; say v` through **both**
//! interpreters and compares all three channels, so it carries no copy of the
//! string at all. `VERSION` is the one place in the tree the bytes appear.
//!
//! # Why a corpus program cannot do this instead
//!
//! `tests/corpus.rs` already runs programs through both interpreters and would
//! be the natural home. It cannot be: the corpus's one standing rule
//! (`corpus/README.md`, "The one rule: determinism") is that a program's
//! output must be the same on every machine, and a build date is neither. So
//! the differential lives here, where its one program is synthesised rather
//! than committed.
//!
//! # The gate, and what it does and does not protect
//!
//! Gated on `REXX_CORPUS_GATE`, the same switch `tests/corpus.rs` uses, so an
//! offline checkout is not asked to produce an oracle for it. **That
//! protection is partial and saying otherwise here would be false**:
//! `tests/builtin_status.rs` invokes the oracle unconditionally on a plain
//! `cargo test`, with no gate at all -- `support::oracle::locate` asserts the
//! binary exists rather than skipping -- so a machine without the oracle
//! already cannot run this crate's default test suite. The gate is followed
//! here because it
//! is the convention for a check whose whole subject is the oracle, not
//! because it restores a property the workspace has.
//!
//! Report mode prints what it skipped rather than passing silently, matching
//! `corpus.rs`'s own REPORT/STRICT split.

mod support;

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use support::oracle::descriptor_diffs;

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
///
/// **This is the only guard on `VERSION`'s build date.** It goes red two ways
/// and both matter: someone edits the constant, or the oracle is rebuilt and
/// its build date moves. The second is the one no self-comparison can see, and
/// the commit that introduced the constant (`f322477f`'s own standing
/// consequence: "any recorded figure is a claim about the binary present when
/// it was taken") is why it needs seeing.
///
/// The remedy when it fires is a re-measurement, not an edit to make it pass:
/// read the oracle's answer out of the failure message and put *that* in
/// `VERSION`.
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
        cpp.exit_code
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
