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

//! Trace-line **indent** against the oracle, compared raw.
//!
//! **This file exists because every other differential harness in the tree is
//! blind to an indent.** `tests/corpus.rs` and `tests/trace_oracle.rs` both
//! compare stderr through `support::normalize_stderr` -- DEVIATION 0, which
//! collapses the run of spaces between a trace line's marker and its content
//! down to one. A transcript emitted at the wrong indent therefore compares
//! *equal* to the same transcript emitted at the right one, so a test routed
//! through either harness passes whether the indent is right or wrong. That is
//! the "test that cannot fail" shape, and
//! [`deviation_0_collapses_every_wrong_answer_this_file_holds`] measures the
//! blindness rather than describing it.
//!
//! So the comparison here is **byte for byte on raw stderr**, plus stdout and
//! the exit code, and it runs under **both engines**. DEVIATION 0 itself is
//! untouched: it is a recorded, accepted deviation with its own justification,
//! and whether it should survive is a question for whoever reviews it, not
//! something a harness reaching around it should decide by editing it.
//!
//! ## What the cases pin
//!
//! Both quantities are the oracle's own `settings.traceIndent`, a counter this
//! crate has no equivalent of: it derives a clause's indent from that clause's
//! lexical nesting plus an activation base. The two places the derivation
//! parts company with the counter are what these cases hold:
//!
//! * **A `SIGNAL` resets the counter to zero** (`RexxActivation::signalTo`),
//!   including a `SIGNAL` that a `SIGNAL ON` trap performs, and including one
//!   inside a `CALL`ed label -- where this crate used to keep the callee's own
//!   base and echo everything after the transfer two columns too deep.
//! * **A `DO`/`LOOP` header's own clause ends one level deeper than it
//!   echoed** when the body is about to run (`newBlockInstruction`'s
//!   `traceIndent++` runs after the clause is traced and after its control
//!   expressions are evaluated), and at its own level when the loop is over
//!   (`terminate` takes it back off). A `CALL ON` handler delivered at that
//!   boundary is based on whichever it is, so the two directions are one rule
//!   and a case each way is what says so.
//!
//! ## Regeneration
//!
//! Each `<name>.expected` was captured from the oracle exactly as
//! `tests/trace_oracle.rs`'s own module doc describes, with one substitution:
//! the absolute program path the oracle prints in a raised condition's middle
//! line is replaced by [`CASE_PATH`], which is the path this harness hands
//! `run_program`. `ir_dual.rs`'s `/nonexistent/ir-dual-case.rex` is the same
//! device for the same reason -- running the program under the oracle prints
//! the oracle's own path there instead.
//!
//! Each `<name>.wrong` is a transcript this crate really produced from a build
//! that got this case's indent wrong. **It is not an expectation and nothing
//! is compared to it directly**: its whole job is to give
//! [`deviation_0_collapses_every_wrong_answer_this_file_holds`] a concrete
//! wrong answer per case, so that the claim "only a raw comparison can hold
//! this" is measured on each case rather than asserted once for the file.

mod support;

use rexx_exec::{Engine, Invocation, run_program};
use std::path::{Path, PathBuf};

/// The path this harness hands `run_program`, and the path every
/// `.expected` file's raised-condition line carries.
///
/// Fixed rather than the case file's own location, because the expectation is
/// committed and a real path is the one byte in it that would differ between
/// two checkouts.
const CASE_PATH: &str = "/nonexistent/trace-indent-case.rex";

/// One case's fixed oracle answer: exit code, stdout, stderr.
struct Expected {
    rc: i32,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

fn case_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/trace_indent")
}

/// Every case in `tests/trace_indent`, discovered from the directory rather
/// than listed here.
///
/// **A list in this file would be the thing that rots**: a case file added
/// with no line here is a case that never runs, and nothing would say so.
/// Reading the directory means the set of cases is the set of files, and
/// [`every_case_ships_all_three_files`] is what turns a half-added case into
/// a failure instead of a silent omission.
fn case_names() -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(case_dir())
        .expect("the case directory is present")
        .map(|entry| entry.expect("a readable directory entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "rex"))
        .map(|path| {
            path.file_stem()
                .expect("a .rex file has a stem")
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    names.sort();
    assert!(!names.is_empty(), "no cases found in {:?}", case_dir());
    names
}

/// Parses a `.expected` file, in `tests/trace_oracle`'s own format (`RC n` /
/// `===STDOUT===` / bytes / `===STDERR===` / bytes) so that a capture taken
/// for one harness can be read by the other.
fn parse_expected(bytes: &[u8], name: &str) -> Expected {
    let text = std::str::from_utf8(bytes)
        .unwrap_or_else(|_| panic!("{name}: expectation file is not valid UTF-8"));
    let after_rc = text
        .strip_prefix("RC ")
        .unwrap_or_else(|| panic!("{name}: expectation file does not start with `RC `"));
    let (rc_text, rest) = after_rc
        .split_once('\n')
        .unwrap_or_else(|| panic!("{name}: no newline after the RC line"));
    let rc: i32 = rc_text
        .parse()
        .unwrap_or_else(|_| panic!("{name}: `{rc_text}` is not an exit code"));
    let rest = rest
        .strip_prefix("===STDOUT===\n")
        .unwrap_or_else(|| panic!("{name}: missing `===STDOUT===` marker"));
    let (stdout, stderr) = rest
        .split_once("===STDERR===\n")
        .unwrap_or_else(|| panic!("{name}: missing `===STDERR===` marker"));
    Expected {
        rc,
        stdout: stdout.as_bytes().to_vec(),
        stderr: stderr.as_bytes().to_vec(),
    }
}

fn read_case(name: &str, extension: &str) -> Vec<u8> {
    let path = case_dir().join(format!("{name}.{extension}"));
    std::fs::read(&path).unwrap_or_else(|e| panic!("{}: unreadable ({e})", path.display()))
}

/// Renders a mismatch between two byte strings for a failure message.
///
/// The comparison that decides is always on `&[u8]`; this is the *report* of
/// one that already failed, so a lossy rendering costs nothing and is what a
/// reader can actually read.
fn describe(label: &str, actual: &[u8], expected: &[u8]) -> String {
    format!(
        "  {label}\n    actual:   {:?}\n    expected: {:?}",
        String::from_utf8_lossy(actual),
        String::from_utf8_lossy(expected)
    )
}

/// Runs one case under one engine and compares all three descriptors against
/// the oracle's own bytes, with **no normalisation on any of them**.
///
/// **Answers rather than asserts, and that is what makes a run readable.** A
/// `#[test]` that asserts inside a loop over the cases stops at the first
/// failure, so a change that breaks one case is indistinguishable in the
/// output from one that breaks every case after it in sorted order -- and the
/// cases this file holds fall into two families that are meant to fail
/// separately. Measured: under a mutation of the loop-header rule, the first
/// case in sorted order is a loop case, so an asserting loop never ran the
/// `signal_*` cases at all and "only the loop cases went red" was not
/// something the run said. The callers collect these and assert once.
///
/// **stdout and stderr compare as `&[u8]`.** This file's premise is byte for
/// byte, and `String::from_utf8_lossy` maps every invalid sequence to one
/// replacement character, so two different non-UTF-8 stderrs would compare
/// equal through it. Today's transcripts are ASCII and it would not bite yet;
/// the premise is what has to stay true.
fn check_case(name: &str, engine: Engine) -> Vec<String> {
    let expected = parse_expected(&read_case(name, "expected"), name);
    let source = read_case(name, "rex");
    let outcome = run_program(CASE_PATH, source, Invocation::none().with_engine(engine));

    let mut mismatches = Vec::new();
    if outcome.stdout != expected.stdout {
        mismatches.push(describe(
            &format!("{name} ({engine:?}): stdout"),
            &outcome.stdout,
            &expected.stdout,
        ));
    }
    if outcome.stderr != expected.stderr {
        mismatches.push(describe(
            &format!("{name} ({engine:?}): stderr, raw"),
            &outcome.stderr,
            &expected.stderr,
        ));
    }
    if outcome.exit_code != expected.rc {
        mismatches.push(format!(
            "  {name} ({engine:?}): exit code\n    actual:   {}\n    expected: {}",
            outcome.exit_code, expected.rc
        ));
    }
    mismatches
}

/// Every case under one engine, each one run whatever the ones before it did,
/// with every mismatch named in a single failure.
fn check_every_case(engine: Engine) {
    let names = case_names();
    let mismatches: Vec<String> = names
        .iter()
        .flat_map(|name| check_case(name, engine))
        .collect();
    assert!(
        mismatches.is_empty(),
        "{} of {} cases disagree with the oracle under {engine:?}, compared raw -- \
         this comparison is the whole point of this file and does not go through \
         DEVIATION 0:\n{}",
        mismatches.len(),
        names.len(),
        mismatches.join("\n")
    );
}

#[test]
fn every_case_matches_the_oracle_on_the_tree_walker() {
    check_every_case(Engine::TreeWalker);
}

#[test]
fn every_case_matches_the_oracle_on_the_compiled_engine() {
    check_every_case(Engine::Ir);
}

/// The blindness this file exists to reach around, measured per case.
///
/// For each case, a wrong answer a real build produced and the oracle's own
/// bytes **differ raw** and are **equal under DEVIATION 0**. The first half
/// says the comparison above has something to catch; the second says neither
/// `corpus.rs` nor `trace_oracle.rs` could catch it, which is why the
/// comparison above is raw.
/// Collected per case and asserted once, for the reason [`check_case`] gives:
/// a loop that asserts reports only the first case that fails, and the answer
/// worth having here is which cases do.
#[test]
fn deviation_0_collapses_every_wrong_answer_this_file_holds() {
    let mut findings = Vec::new();
    for name in case_names() {
        let expected = parse_expected(&read_case(&name, "expected"), &name);
        let wrong = read_case(&name, "wrong");
        if wrong == expected.stderr {
            findings.push(format!(
                "  {name}: the wrong answer is the right one, so this case pins nothing"
            ));
        }
        if support::normalize_stderr(&wrong) != support::normalize_stderr(&expected.stderr) {
            findings.push(describe(
                &format!(
                    "{name}: DEVIATION 0 does not collapse this difference, so the shared \
                     harnesses can see it and this file is not what should be pinning it"
                ),
                &support::normalize_stderr(&wrong),
                &support::normalize_stderr(&expected.stderr),
            ));
        }
    }
    assert!(findings.is_empty(), "{}", findings.join("\n"));
}

/// A case is three files, and a missing one is a case that half-runs.
#[test]
fn every_case_ships_all_three_files() {
    let missing: Vec<String> = case_names()
        .iter()
        .flat_map(|name| {
            ["rex", "expected", "wrong"]
                .iter()
                .map(move |extension| case_dir().join(format!("{name}.{extension}")))
        })
        .filter(|path| !path.is_file())
        .map(|path| format!("  {}: missing", path.display()))
        .collect();
    assert!(missing.is_empty(), "{}", missing.join("\n"));
}
