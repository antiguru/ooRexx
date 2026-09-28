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
//! copy of the framework and the group's directory at the same path. A test
//! whose stdout, stderr or exit status differs is a member of the failing
//! set, and that set must be exactly [`RECORDED`]. Then each group runs whole
//! in one process on both sides, its recorded tests renamed out of the group
//! in the copy, and the three descriptors must agree: some tests depend on an
//! earlier one (FUNCTION's io tests on `TEST`), which a lone run cannot see.
//! Gate-only.
//!
//! Two options keep the framework itself off paths other phases own, on both
//! sides alike: `-U` starts no ticker (a `REPLY` thread waiting on `GUARD ON
//! WHEN`, Phase 6), and `-V 0` prints the brief summary, where the full one
//! calls `RXFUNCQUERY` (Phase 10), which the oracle answers through rxapi.

mod support;
mod watchdog;

use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use rexx_exec::Invocation;
use support::oracle::{self, did_not_finish};

const GATE_ENV: &str = "REXX_CORPUS_GATE";

fn gate_mode() -> bool {
    match env::var(GATE_ENV) {
        Ok(value) => !value.is_empty() && value != "0",
        Err(_) => false,
    }
}

/// The groups `api_group_partition.rs` derives as Phase 8's.
const GROUPS: &[&str] = &["CONVERSION", "FUNCTION", "METHOD"];

/// The tests that differ, as `GROUP.TEST`. Each is recorded, with its cause
/// and the phase that owns it, in `docs/superpowers/plans/phase-4-exclusions.txt`.
const RECORDED: &[&str] = &[
    "FUNCTION.TEST_INPUT_OUTPUT_STREAM",
    "FUNCTION.TEST_REXXQUEUE",
    "METHOD.TESTCLASS01",
    "METHOD.TESTDROPOBJECTVARIABLE01",
    "METHOD.TESTGETOBJECTVARIABLE01",
    "METHOD.TESTGETOBJECTVARIABLE02",
    "METHOD.TESTGETOBJECTVARIABLE03",
    "METHOD.TESTGETPACKAGECLASSES01",
    "METHOD.TESTGETPACKAGEPUBLICCLASSES01",
    "METHOD.TESTGETPACKAGEPUBLICROUTINES01",
    "METHOD.TESTGETPACKAGEROUTINES01",
    "METHOD.TESTNEWMETHOD02",
    "METHOD.TESTNEWROUTINE02",
    "METHOD.TESTSETOBJECTVARIABLE01",
];

fn worktree() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn copy_tree(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap_or_else(|e| panic!("cannot create {}: {e}", to.display()));
    for entry in
        fs::read_dir(from).unwrap_or_else(|e| panic!("cannot read {}: {e}", from.display()))
    {
        let entry = entry.expect("a directory entry");
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), &target)
                .unwrap_or_else(|e| panic!("cannot copy {}: {e}", entry.path().display()));
        }
    }
}

/// Empties `run` and lays the framework and the `API/oo` directory into it.
/// `rxregexp.cls`, which `ooTest.frm` requires, sits beside the driver.
fn fresh_copy(run: &Path) {
    if run.exists() {
        fs::remove_dir_all(run).unwrap_or_else(|e| panic!("cannot empty {}: {e}", run.display()));
    }
    let ootest = worktree().join("ootest");
    copy_tree(&ootest.join("framework"), &run.join("framework"));
    copy_tree(&ootest.join("ooRexx/API/oo"), &run.join("ooRexx/API/oo"));
    for file in ["testOORexx.rex", "worker.rex", "ooTest.frm"] {
        fs::copy(ootest.join(file), run.join(file))
            .unwrap_or_else(|e| panic!("cannot copy {file}: {e}"));
    }
    fs::copy(
        worktree().join("extensions/rxregexp/rxregexp.cls"),
        run.join("rxregexp.cls"),
    )
    .expect("cannot copy rxregexp.cls");
}

/// One run's three descriptors.
struct Run {
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    status: Option<i32>,
}

fn run_oracle(oracle: &oracle::Oracle, run: &Path, args: &[&str]) -> Run {
    let outcome = oracle.run_with(&run.join("testOORexx.rex"), args, None);
    let status = (!did_not_finish(&outcome)).then(|| outcome.expect_exit_code());
    Run {
        stdout: outcome.stdout,
        stderr: outcome.stderr,
        status,
    }
}

fn run_crate(run: &Path, args: &[&str]) -> Run {
    let driver = run.join("testOORexx.rex");
    let text = fs::read(&driver).expect("the copied driver");
    let lib = oracle::oracle_root().join("lib");
    let mut environment: Vec<(Vec<u8>, Vec<u8>)> = env::vars()
        .filter(|(name, _)| name != "LD_LIBRARY_PATH")
        .map(|(name, value)| (name.into_bytes(), value.into_bytes()))
        .collect();
    environment.push((
        b"LD_LIBRARY_PATH".to_vec(),
        lib.to_string_lossy().into_owned().into_bytes(),
    ));
    let invocation = Invocation::with_argument(args.join(" ").into_bytes())
        .with_directory(run.to_path_buf())
        .with_environment(environment);
    let outcome = watchdog::run_bounded(&driver.to_string_lossy(), text, invocation);
    let status = (!watchdog::did_not_finish(&outcome)).then_some(outcome.exit_code);
    Run {
        stdout: outcome.stdout,
        stderr: outcome.stderr,
        status,
    }
}

/// The group's tests in the order the oracle runs them, from `-S`'s lines.
fn test_names(oracle: &oracle::Oracle, run: &Path, group_file: &str) -> Vec<String> {
    fresh_copy(run);
    let listed = run_oracle(oracle, run, &["-f", group_file, "-U", "-V", "0", "-S"]);
    assert!(
        listed.status.is_some(),
        "the oracle's listing of {group_file} did not finish"
    );
    let names: Vec<String> = String::from_utf8_lossy(&listed.stderr)
        .lines()
        .filter_map(|line| {
            line.split_once(" test case ")
                .map(|(_, name)| name.to_string())
        })
        .collect();
    assert!(
        !names.is_empty(),
        "the oracle listed no test in {group_file}"
    );
    names
}

/// Renames each recorded test of `group` in the copy's group file, so the
/// framework no longer counts it a test; each must be found.
fn skip_recorded(group_file: &Path, group: &str) {
    let text = fs::read(group_file).expect("the copied group file");
    let text = String::from_utf8_lossy(&text).into_owned();
    let mut skipped = BTreeSet::new();
    let lines: Vec<String> = text
        .split('\n')
        .map(|line| {
            let trimmed = line.trim_start();
            let Some(rest) = trimmed
                .get(..8)
                .filter(|head| head.eq_ignore_ascii_case("::method"))
                .map(|_| &trimmed[8..])
            else {
                return line.to_string();
            };
            let rest = rest.trim_start();
            let quoted = rest.starts_with(['\'', '"']);
            let name_start = usize::from(quoted);
            let name: String = rest[name_start..]
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect();
            let member = format!("{group}.{}", name.to_ascii_uppercase());
            if !RECORDED.contains(&member.as_str()) {
                return line.to_string();
            }
            skipped.insert(member);
            let at = line.len() - rest.len() + name_start;
            format!("{}SKIPPED_{}", &line[..at], &line[at..])
        })
        .collect();
    let wanted: BTreeSet<String> = RECORDED
        .iter()
        .filter(|name| name.starts_with(&format!("{group}.")))
        .map(|name| name.to_string())
        .collect();
    assert_eq!(
        skipped, wanted,
        "the recorded tests of {group} renamed in the copy"
    );
    fs::write(group_file, lines.join("\n")).expect("cannot rewrite the copied group file");
}

fn excerpt(bytes: &[u8]) -> String {
    let text = String::from_utf8_lossy(bytes);
    text.chars().take(300).collect()
}

#[test]
fn every_test_of_the_phase_8_groups_matches_the_oracle_but_the_recorded() {
    if !gate_mode() {
        eprintln!("api_group_tests: skipped without {GATE_ENV}");
        return;
    }
    let exclusions =
        fs::read_to_string(worktree().join("docs/superpowers/plans/phase-4-exclusions.txt"))
            .expect("the exclusions file");
    for name in RECORDED {
        let test = name.split_once('.').expect("GROUP.TEST").1;
        assert!(
            exclusions.contains(test),
            "{name} has no record in phase-4-exclusions.txt"
        );
    }

    let oracle = oracle::locate();
    let run = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("api-group-tests-{}", std::process::id()))
        .join("run");
    let mut differing = BTreeSet::new();
    let mut report = String::new();
    for group in GROUPS {
        let group_file = run
            .join(format!("ooRexx/API/oo/{group}.testGroup"))
            .to_string_lossy()
            .into_owned();
        for name in test_names(&oracle, &run, &group_file) {
            let args = [
                "-f",
                group_file.as_str(),
                "-U",
                "-V",
                "0",
                "-t",
                name.as_str(),
            ];
            fresh_copy(&run);
            let theirs = run_oracle(&oracle, &run, &args);
            fresh_copy(&run);
            let ours = run_crate(&run, &args);
            assert!(
                theirs.status.is_some(),
                "the oracle did not finish {group}.{name}"
            );
            if theirs.stdout != ours.stdout
                || theirs.stderr != ours.stderr
                || theirs.status != ours.status
            {
                let member = format!("{group}.{name}");
                report.push_str(&format!(
                    "{member}: oracle {:?} {:?}, ours {:?} {:?}\n",
                    theirs.status,
                    excerpt(&theirs.stderr),
                    ours.status,
                    excerpt(&ours.stderr),
                ));
                differing.insert(member);
            }
        }
    }
    let mut whole = String::new();
    for group in GROUPS {
        let group_file = run.join(format!("ooRexx/API/oo/{group}.testGroup"));
        let shown = group_file.to_string_lossy().into_owned();
        let args = ["-f", shown.as_str(), "-U", "-V", "0", "-S"];
        fresh_copy(&run);
        skip_recorded(&group_file, group);
        let theirs = run_oracle(&oracle, &run, &args);
        fresh_copy(&run);
        skip_recorded(&group_file, group);
        let ours = run_crate(&run, &args);
        assert!(theirs.status.is_some(), "the oracle did not finish {group}");
        if theirs.stdout != ours.stdout
            || theirs.stderr != ours.stderr
            || theirs.status != ours.status
        {
            whole.push_str(&format!(
                "{group}: oracle {:?} {:?}, ours {:?} {:?}\n",
                theirs.status,
                excerpt(&theirs.stdout),
                ours.status,
                excerpt(&ours.stdout),
            ));
        }
    }
    fs::remove_dir_all(run.parent().expect("a parent")).expect("cannot remove the run directory");

    let recorded: BTreeSet<String> = RECORDED.iter().map(|name| name.to_string()).collect();
    let newly_failing: Vec<&String> = differing.difference(&recorded).collect();
    let newly_passing: Vec<&String> = recorded.difference(&differing).collect();
    assert!(
        newly_failing.is_empty() && newly_passing.is_empty(),
        "newly failing: {newly_failing:?}\nnewly passing: {newly_passing:?}\n{report}"
    );
    assert!(
        whole.is_empty(),
        "a whole-group run without the recorded tests differs:\n{whole}"
    );
}
