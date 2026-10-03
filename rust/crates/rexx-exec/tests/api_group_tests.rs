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

//! The native-API ooTest groups this phase owns, run through `testOORexx.rex`'s
//! single-group form on both sides, one test per run, each run from a fresh
//! copy of the framework and the group's directory at the same path, stdout
//! compared with its timings and timestamps masked. A test whose outcome,
//! assertion count, stderr or exit status differs must be in [`RECORDED`],
//! and each of [`RECORDED`]'s run tests must not pass on this crate; one that
//! differs only in a failure's or an error's detail must be in
//! [`DETAIL_DIFFERS`]. Then each group runs whole in one process on both
//! sides, the tests [`left_out`] names renamed out of the group in the copy,
//! and must pass on this crate and agree on all three descriptors: some tests
//! depend on an earlier one (FUNCTION's io tests on `TEST`, which fail alone
//! on both sides), so the whole run is where every test [`RECORDED`] does
//! not name must pass. Gate-only.
//!
//! The invocation, the copy and the tests that reach rxapi are
//! `support/group_runner.rs`'s ([`group_runner::reaching_rxapi`] derives the
//! ones [`RECORDED`] must run on neither side).

#[path = "support/group_runner.rs"]
mod group_runner;
mod support;
mod watchdog;

use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use group_runner::{
    GATE_ENV, SwitchMode, VERBOSITY, directive_bodies, excerpt, first_difference, fresh_copy,
    gate_mode, group_file, masked, outcome, passes, run_crate, run_oracle, skip, test_names,
    uses_rxapi, worktree,
};
use support::oracle;

/// The directory of the groups below `ootest/ooRexx`.
const DIR: &str = "API/oo";

/// The groups `api_group_partition.rs` derives as Phase 8's.
const GROUPS: &[&str] = &["CONVERSION", "FUNCTION", "METHOD"];

/// The tests this crate does not pass, as `GROUP.TEST`, each recorded with its
/// cause and the phase that owns it in
/// `docs/superpowers/plans/phase-4-exclusions.txt`; `false` for one that runs
/// on neither side, which must be exactly the ones [`reaching_rxapi`] derives.
const RECORDED: &[(&str, bool)] = &[
    ("FUNCTION.TEST_REXXQUEUE", false),
    (
        "METHOD.TEST_REXXC_WITH_NEWROUTINE_LOADPACKAGEFROMDATA",
        true,
    ),
];

/// The tests whose outcome, assertion count, stderr and exit status agree while
/// a failure's or an error's detail differs, as `GROUP.TEST`, each recorded as
/// [`RECORDED`]'s are; `true` where the detail differs in the whole-group run
/// too, which then leaves the test out as it does a recorded one.
const DETAIL_DIFFERS: &[(&str, bool)] = &[];

/// The tests a whole-group run leaves out.
fn left_out() -> BTreeSet<&'static str> {
    RECORDED
        .iter()
        .map(|(name, _)| *name)
        .chain(
            DETAIL_DIFFERS
                .iter()
                .filter(|(_, whole)| *whole)
                .map(|(name, _)| *name),
        )
        .collect()
}

/// The run-on-neither-side entries of [`RECORDED`].
fn not_run() -> BTreeSet<String> {
    RECORDED
        .iter()
        .filter(|(_, run)| !run)
        .map(|(name, _)| name.to_string())
        .collect()
}

/// Fails unless the tests the source says reach rxapi are exactly the ones
/// [`RECORDED`] runs on neither side.
fn assert_rxapi_tests_are_not_run() {
    assert_eq!(
        group_runner::reaching_rxapi(DIR, GROUPS),
        not_run(),
        "the tests reaching rxapi, derived from the group sources, against RECORDED's not-run entries"
    );
}

#[test]
fn the_detector_tells_a_queue_use_from_a_name_that_only_looks_like_one() {
    for line in [
        "  q = .rexxqueue~new('x')",
        "  call rxqueue 'create', 'q'",
        "  name = rxqueue('get')",
        "  if a then call rxfuncadd 'f', 'lib', 'f'",
        "  call sysaddrexxmacro 'm', 'm.rex'",
    ] {
        assert!(uses_rxapi(line), "{line:?} reaches rxapi");
    }
    for line in [
        "  queue = .circularqueue~of(123)",
        "  queue~queue('line1')",
        "  self~assertsame(1, queue~queued)",
        "  -- some rexxqueue tests",
        "  push 'a'",
        "  n = queued()",
        "  parse pull line",
    ] {
        assert!(!uses_rxapi(line), "{line:?} does not reach rxapi");
    }
    let bodies = directive_bodies(
        "::method test_a\n  call helper\n::method test_b\n  say 1\n::routine helper\n  push 1\n",
    );
    assert_eq!(
        bodies
            .iter()
            .map(|(name, _)| name.as_str())
            .collect::<Vec<_>>(),
        ["test_a", "test_b", "helper"]
    );
}

#[test]
fn the_tests_reaching_rxapi_run_on_neither_side() {
    assert_rxapi_tests_are_not_run();
}

#[test]
fn every_test_of_the_phase_8_groups_passes_and_matches_the_oracle_but_the_recorded() {
    if !gate_mode() {
        eprintln!("api_group_tests: skipped without {GATE_ENV}");
        return;
    }
    assert_rxapi_tests_are_not_run();
    let exclusions =
        fs::read_to_string(worktree().join("docs/superpowers/plans/phase-4-exclusions.txt"))
            .expect("the exclusions file");
    // A record names each test on a line of its own.
    let record_lines: BTreeSet<&str> = exclusions.lines().map(str::trim).collect();
    for name in RECORDED
        .iter()
        .chain(DETAIL_DIFFERS.iter())
        .map(|(name, _)| name)
    {
        let test = name.split_once('.').expect("GROUP.TEST").1;
        assert!(
            record_lines.contains(test),
            "{name} has no record in phase-4-exclusions.txt"
        );
    }

    let oracle = oracle::locate();
    let run = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("api-group-tests-{}", std::process::id()))
        .join("run");
    let not_run = not_run();
    let mut passing = BTreeSet::new();
    let mut differing = BTreeSet::new();
    let mut detail_differing = BTreeSet::new();
    let mut report = String::new();
    for group in GROUPS {
        let group_file = group_file(&run, DIR, group);
        for name in test_names(&oracle, &run, DIR, group, &not_run) {
            let member = format!("{group}.{name}");
            assert!(
                !not_run.contains(&member),
                "{member} reaches rxapi and was listed to run"
            );
            let args = [
                "-f",
                group_file.as_str(),
                "-U",
                "-V",
                VERBOSITY,
                "-t",
                name.as_str(),
            ];
            fresh_copy(&run, DIR);
            let theirs = run_oracle(&oracle, &run, &args);
            fresh_copy(&run, DIR);
            let ours = run_crate(&run, &args, SwitchMode::None);
            assert!(
                theirs.status.is_some(),
                "the oracle did not finish {group}.{name}"
            );
            if passes(&ours) {
                passing.insert(member.clone());
            }
            let (their_out, our_out) = (masked(&theirs.stdout), masked(&ours.stdout));
            if their_out != our_out || theirs.stderr != ours.stderr || theirs.status != ours.status
            {
                report.push_str(&format!(
                    "{member}: oracle {:?} {} {:?}, ours {:?} {} {:?}; {}\n",
                    theirs.status,
                    outcome(&theirs.stdout),
                    excerpt(&theirs.stderr),
                    ours.status,
                    outcome(&ours.stdout),
                    excerpt(&ours.stderr),
                    first_difference(&their_out, &our_out),
                ));
                if outcome(&theirs.stdout) == outcome(&ours.stdout)
                    && theirs.stderr == ours.stderr
                    && theirs.status == ours.status
                {
                    detail_differing.insert(member);
                } else {
                    differing.insert(member);
                }
            }
        }
    }
    let mut whole = String::new();
    for group in GROUPS {
        let shown = group_file(&run, DIR, group);
        let group_file = std::path::Path::new(&shown);
        let args = ["-f", shown.as_str(), "-U", "-V", VERBOSITY, "-S"];
        fresh_copy(&run, DIR);
        skip(group_file, group, &left_out());
        let theirs = run_oracle(&oracle, &run, &args);
        fresh_copy(&run, DIR);
        skip(group_file, group, &left_out());
        let ours = run_crate(&run, &args, SwitchMode::None);
        assert!(theirs.status.is_some(), "the oracle did not finish {group}");
        let (their_out, our_out) = (masked(&theirs.stdout), masked(&ours.stdout));
        if their_out != our_out
            || theirs.stderr != ours.stderr
            || theirs.status != ours.status
            || !passes(&ours)
        {
            whole.push_str(&format!(
                "{group}: oracle {:?} {}, ours {:?} {}; {}\n",
                theirs.status,
                outcome(&theirs.stdout),
                ours.status,
                outcome(&ours.stdout),
                first_difference(&their_out, &our_out),
            ));
        }
    }
    fs::remove_dir_all(run.parent().expect("a parent")).expect("cannot remove the run directory");

    let recorded: BTreeSet<String> = RECORDED
        .iter()
        .filter(|(_, run)| *run)
        .map(|(name, _)| name.to_string())
        .collect();
    let newly_differing: Vec<&String> = differing.difference(&recorded).collect();
    let newly_passing: Vec<&String> = recorded.intersection(&passing).collect();
    let mut problems = Vec::new();
    if !newly_differing.is_empty() || !newly_passing.is_empty() {
        problems.push(format!(
            "newly differing: {newly_differing:?}\nrecorded but passing: {newly_passing:?}\n{report}"
        ));
    }
    let detail: BTreeSet<String> = DETAIL_DIFFERS
        .iter()
        .map(|(name, _)| name.to_string())
        .collect();
    let detail_differing: BTreeSet<String> =
        detail_differing.difference(&recorded).cloned().collect();
    if detail_differing != detail {
        let newly: Vec<&String> = detail_differing.difference(&detail).collect();
        let gone: Vec<&String> = detail.difference(&detail_differing).collect();
        problems.push(format!(
            "detail newly differing: {newly:?}\ndetail no longer differing: {gone:?}\n{report}"
        ));
    }
    if !whole.is_empty() {
        problems.push(format!(
            "a whole-group run without the left-out tests differs or does not pass:\n{whole}"
        ));
    }
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

/// Each line the framework's ticker starts, which its dots then follow.
const TICKER_LINES: &[&str] = &[
    "Searching for test containers",
    "Executing automated test suite",
];

/// Exit criterion 2 of the Phase 6 spec: the framework's ticker, a `REPLY`
/// thread and a `GUARD ON WHEN`, runs on this crate without `-U`. How many
/// dots it prints depends on wall time, so they are checked for presence
/// only; every other line equals the `-U` run's.
#[test]
fn the_framework_ticker_runs_without_dash_u() {
    if !gate_mode() {
        eprintln!("api_group_tests: skipped without {GATE_ENV}");
        return;
    }
    let run = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("framework-ticker-{}", std::process::id()))
        .join("run");
    let shown = group_file(&run, DIR, "CONVERSION");
    let mut runs = Vec::new();
    for ticker in [false, true] {
        let mut args = vec!["-f", shown.as_str(), "-V", VERBOSITY];
        if !ticker {
            args.push("-U");
        }
        fresh_copy(&run, DIR);
        runs.push(run_crate(&run, &args, SwitchMode::None));
    }
    fs::remove_dir_all(run.parent().expect("a parent")).expect("cannot remove the run directory");
    let (quiet, ticking) = (&runs[0], &runs[1]);
    assert_eq!(quiet.status, Some(0), "-U: {}", excerpt(&quiet.stderr));
    assert_eq!(
        ticking.status,
        Some(0),
        "ticker: {}",
        excerpt(&ticking.stderr)
    );
    assert_eq!(ticking.stderr, quiet.stderr);

    let ticking = String::from_utf8_lossy(&masked(&ticking.stdout)).into_owned();
    let mut ticked = Vec::new();
    let mut lines = Vec::new();
    for line in ticking.lines() {
        match TICKER_LINES.iter().find(|start| line.starts_with(**start)) {
            Some(start) => {
                let dots = &line[start.len()..];
                assert!(
                    !dots.is_empty() && dots.bytes().all(|b| b == b'.'),
                    "the ticker's line {line:?} is not its text and dots"
                );
                ticked.push(*start);
                lines.push(*start);
            }
            // Past 75 dots the ticker goes on on a line of its own.
            None if !line.is_empty() && line.bytes().all(|b| b == b'.') => {}
            None => lines.push(line),
        }
    }
    assert_eq!(
        ticked, TICKER_LINES,
        "each ticker line, with dots, in order"
    );
    let quiet = String::from_utf8_lossy(&masked(&quiet.stdout)).into_owned();
    assert_eq!(lines, quiet.lines().collect::<Vec<_>>());
}
