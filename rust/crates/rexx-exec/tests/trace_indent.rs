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

mod support;

use rexx_exec::{Invocation, run_program};
use std::path::{Path, PathBuf};

/// The path this harness hands `run_program`, and the path every
/// `.expected` file's raised-condition line carries.
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
fn describe(label: &str, actual: &[u8], expected: &[u8]) -> String {
    format!(
        "  {label}\n    actual:   {:?}\n    expected: {:?}",
        String::from_utf8_lossy(actual),
        String::from_utf8_lossy(expected)
    )
}

/// Runs one case under one engine and compares all three descriptors against
/// the oracle's own bytes, with **no normalisation on any of them**.
fn check_case(name: &str) -> Vec<String> {
    let expected = parse_expected(&read_case(name, "expected"), name);
    let source = read_case(name, "rex");
    let outcome = run_program(CASE_PATH, source, Invocation::none());

    let mut mismatches = Vec::new();
    if outcome.stdout != expected.stdout {
        mismatches.push(describe(
            &format!("{name}: stdout"),
            &outcome.stdout,
            &expected.stdout,
        ));
    }
    if outcome.stderr != expected.stderr {
        mismatches.push(describe(
            &format!("{name}: stderr, raw"),
            &outcome.stderr,
            &expected.stderr,
        ));
    }
    if outcome.exit_code != expected.rc {
        mismatches.push(format!(
            "  {name}: exit code\n    actual:   {}\n    expected: {}",
            outcome.exit_code, expected.rc
        ));
    }
    mismatches
}

/// Every case, each one run whatever the ones before it did,
/// with every mismatch named in a single failure.
fn check_every_case() {
    let names = case_names();
    let failed: Vec<(&String, Vec<String>)> = names
        .iter()
        .map(|name| (name, check_case(name)))
        .filter(|(_, mismatches)| !mismatches.is_empty())
        .collect();
    let descriptors: usize = failed.iter().map(|(_, m)| m.len()).sum();
    let detail: Vec<&str> = failed
        .iter()
        .flat_map(|(_, m)| m.iter().map(String::as_str))
        .collect();
    assert!(
        failed.is_empty(),
        "{} of {} cases disagree with the oracle, in {descriptors} \
         descriptors, compared raw -- this comparison is the whole point of this file \
         and does not go through DEVIATION 0:\n{}",
        failed.len(),
        names.len(),
        detail.join("\n")
    );
}

#[test]
fn every_case_matches_the_oracle() {
    check_every_case();
}

/// The blindness this file exists to reach around, measured per case.
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
