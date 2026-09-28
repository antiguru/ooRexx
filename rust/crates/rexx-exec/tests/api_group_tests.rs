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
//! Both sides run at `-V 2`, which prints the assertion count and each
//! failure's and error's detail, so a test that fails differently, or runs
//! fewer assertions, differs. `-U` starts no ticker (a
//! `REPLY` thread waiting on `GUARD ON WHEN`, Phase 6). The copy's
//! `ooTest.frm` has `printSummary`'s `rxfuncquery` probes removed: on the
//! oracle they reach rxapi (`interpreter/package/PackageManager.cpp:618-635`),
//! and neither name they ask for is registered on either side, so the summary
//! prints the same lines without them.
//!
//! A test whose source uses the data queue or the function and macro-space
//! registries ([`reaching_rxapi`]) runs on neither side, the listing run
//! included: on the oracle those reach rxapi's daemon and the state it keeps
//! between processes. The oracle still asks rxapi's macro space on each
//! `::REQUIRES` and external call (`PackageManager.cpp:747`,
//! `RexxActivation.cpp:2996`), which no ooTest run can avoid.

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

/// See the module doc.
const VERBOSITY: &str = "2";

/// The summary lines whose value is a duration.
const TIMINGS: &[&str] = &[
    "File search:",
    "Suite construction:",
    "Test execution:",
    "Total time:",
];

/// The detail headers whose rest of line is a timestamp.
const STAMPED: &[&str] = &["[failure] ", "[error] "];

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

/// The `GROUP.TEST` of each test of [`GROUPS`] whose source uses the data
/// queue or the function or macro-space registries: an instruction or a call
/// in its own method body, or through a `::METHOD` or `::ROUTINE` of the group
/// file, or of a file it requires or loads from its own directory, that does.
fn reaching_rxapi() -> BTreeSet<String> {
    let dir = worktree().join("ootest/ooRexx/API/oo");
    let mut found = BTreeSet::new();
    for group in GROUPS {
        let group_file = dir.join(format!("{group}.testGroup"));
        let text = read_lossy(&group_file);
        let mut bodies = directive_bodies(&text);
        for loaded in loaded_files(&text) {
            let path = dir.join(&loaded);
            if path.is_file() {
                bodies.extend(directive_bodies(&read_lossy(&path)));
            }
        }
        let mut reaching: BTreeSet<String> = bodies
            .iter()
            .filter(|(_, body)| body.lines().any(uses_rxapi))
            .map(|(name, _)| name.clone())
            .collect();
        loop {
            let more: Vec<String> = bodies
                .iter()
                .filter(|(name, body)| {
                    !reaching.contains(name) && words(body).any(|word| reaching.contains(word))
                })
                .map(|(name, _)| name.clone())
                .collect();
            if more.is_empty() {
                break;
            }
            reaching.extend(more);
        }
        found.extend(
            reaching
                .into_iter()
                .filter(|name| name.starts_with("test"))
                .map(|name| format!("{group}.{}", name.to_ascii_uppercase())),
        );
    }
    found
}

fn read_lossy(path: &Path) -> String {
    let bytes = fs::read(path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    String::from_utf8_lossy(&bytes).into_owned()
}

/// Each `::METHOD` and `::ROUTINE` of `text`, lower-cased, with its body.
fn directive_bodies(text: &str) -> Vec<(String, String)> {
    let mut bodies: Vec<(String, String)> = Vec::new();
    let mut current: Option<(String, String)> = None;
    for line in text.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("::") {
            bodies.extend(current.take());
            let mut parts = trimmed[2..].split_whitespace();
            let kind = parts.next().unwrap_or("").to_ascii_lowercase();
            if kind == "method" || kind == "routine" {
                let name = parts
                    .next()
                    .unwrap_or("")
                    .trim_matches(['\'', '"'])
                    .to_ascii_lowercase();
                current = Some((name, String::new()));
            }
        } else if let Some((_, body)) = current.as_mut() {
            body.push_str(&line.to_ascii_lowercase());
            body.push('\n');
        }
    }
    bodies.extend(current);
    bodies
}

/// The files `text` names in a `::REQUIRES` or a `loadPackage` call.
fn loaded_files(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(|line| {
            let lower = line.to_ascii_lowercase();
            let at = lower
                .find("::requires")
                .or_else(|| lower.find("loadpackage("))?;
            let rest = &line[at..];
            let open = rest.find(['\'', '"'])?;
            let quote = rest[open..].chars().next()?;
            let name = rest[open + 1..].split(quote).next()?;
            Some(name.to_string())
        })
        .collect()
}

fn words(text: &str) -> impl Iterator<Item = &str> {
    text.split(|c: char| !(c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '!' | '?')))
        .filter(|word| !word.is_empty())
}

/// Whether a lower-cased source line uses the data queue or a registry.
fn uses_rxapi(line: &str) -> bool {
    const CALLS: &[&str] = &[
        "rxqueue",
        "queued",
        "rxfuncadd",
        "rxfuncdrop",
        "rxfuncquery",
    ];
    if line.contains(".rexxqueue") || line.contains("rexxmacro") {
        return true;
    }
    if CALLS.iter().any(|call| line.contains(&format!("{call}("))) {
        return true;
    }
    let clauses = [" then ", " else ", " otherwise "]
        .iter()
        .fold(format!(" {line} ").replace(';', " ; "), |text, lead| {
            text.replace(lead, " ; ")
        });
    clauses.split(';').any(|clause| {
        let mut tokens = clause.split_whitespace();
        let first = tokens.next().unwrap_or("");
        let assignment = tokens
            .clone()
            .next()
            .is_some_and(|next| next.starts_with('='));
        match first {
            "push" | "queue" | "pull" => !assignment,
            "call" => tokens.next().is_some_and(|target| CALLS.contains(&target)),
            "parse" => tokens
                .find(|word| !matches!(*word, "upper" | "lower" | "caseless"))
                .is_some_and(|word| word == "pull"),
            _ => false,
        }
    })
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
        reaching_rxapi(),
        not_run(),
        "the tests reaching rxapi, derived from the group sources, against RECORDED's not-run entries"
    );
}

#[test]
fn the_detector_tells_a_queue_use_from_a_name_that_only_looks_like_one() {
    for line in [
        "  q = .rexxqueue~new('x')",
        "  push 'a'",
        "  if a then queue 'b'",
        "  x = 1; pull",
        "  parse upper pull line",
        "  n = queued()",
        "  call rxfuncadd 'f', 'lib', 'f'",
        "  call sysaddrexxmacro 'm', 'm.rex'",
    ] {
        assert!(uses_rxapi(line), "{line:?} reaches rxapi");
    }
    for line in [
        "  queue = .circularqueue~of(123)",
        "  queue~queue('line1')",
        "  self~assertsame(1, queue~queued)",
        "  -- some rexxqueue tests",
        "  parse arg pull",
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
    without_rxfuncquery(&run.join("ooTest.frm"));
    fs::copy(
        worktree().join("extensions/rxregexp/rxregexp.cls"),
        run.join("rxregexp.cls"),
    )
    .expect("cannot copy rxregexp.cls");
}

/// Removes each `rxfuncquery` test from the copied `ooTest.frm`, with the
/// comment above it and the assignment it guards.
fn without_rxfuncquery(frm: &Path) {
    let text = fs::read_to_string(frm).expect("the copied ooTest.frm");
    let lines: Vec<&str> = text.split('\n').collect();
    let hits: Vec<usize> = (0..lines.len())
        .filter(|&at| lines[at].contains("rxfuncquery("))
        .collect();
    assert!(!hits.is_empty(), "ooTest.frm has no rxfuncquery to remove");
    let mut dropped = BTreeSet::new();
    for at in hits {
        assert!(
            at > 0
                && lines[at - 1].trim_start().starts_with("--")
                && lines
                    .get(at + 1)
                    .is_some_and(|next| next.trim_start().starts_with("versions[")),
            "ooTest.frm line {} is not a commented rxfuncquery test guarding one assignment",
            at + 1
        );
        dropped.extend([at - 1, at, at + 1]);
    }
    let kept: Vec<&str> = (0..lines.len())
        .filter(|at| !dropped.contains(at))
        .map(|at| lines[at])
        .collect();
    fs::write(frm, kept.join("\n")).expect("cannot rewrite the copied ooTest.frm");
}

/// `stdout` with what differs between any two runs of one side masked: each
/// timing line's duration and each detail header's timestamp.
fn masked(stdout: &[u8]) -> Vec<u8> {
    let text = String::from_utf8_lossy(stdout);
    let lines: Vec<&str> = text
        .split('\n')
        .map(|line| {
            TIMINGS
                .iter()
                .chain(STAMPED)
                .find(|prefix| line.starts_with(**prefix))
                .map_or(line, |prefix| *prefix)
        })
        .collect();
    lines.join("\n").into_bytes()
}

/// The outcome class and assertion count a run's summary reports.
fn outcome(stdout: &[u8]) -> String {
    let text = String::from_utf8_lossy(stdout);
    let value = |label: &str| {
        text.lines()
            .find_map(|line| line.strip_prefix(label))
            .map(|rest| rest.trim().to_string())
    };
    let class = match (value("Errors:"), value("Failures:")) {
        (Some(errors), _) if errors != "0" => "error",
        (_, Some(failures)) if failures != "0" => "failure",
        (Some(_), Some(_)) => "pass",
        _ => "no summary",
    };
    let asserts = value("Assertions:").unwrap_or_else(|| "none".to_string());
    format!("{class}, assertions {asserts}")
}

/// The first line of the masked stdout the two sides disagree on.
fn first_difference(theirs: &[u8], ours: &[u8]) -> String {
    let theirs = String::from_utf8_lossy(theirs);
    let ours = String::from_utf8_lossy(ours);
    let mut left = theirs.lines();
    let mut right = ours.lines();
    loop {
        match (left.next(), right.next()) {
            (None, None) => return "stdout agrees".to_string(),
            (a, b) if a == b => {}
            (a, b) => return format!("oracle {a:?}, ours {b:?}"),
        }
    }
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

/// The group's tests in the order the oracle runs them, from `-S`'s lines,
/// which runs them all, so the tests [`not_run`] names are renamed out first.
fn test_names(oracle: &oracle::Oracle, run: &Path, group: &str) -> Vec<String> {
    let path = run.join(format!("ooRexx/API/oo/{group}.testGroup"));
    let group_file = path.to_string_lossy().into_owned();
    let group_file = group_file.as_str();
    fresh_copy(run);
    skip(&path, group, &not_run());
    let listed = run_oracle(
        oracle,
        run,
        &["-f", group_file, "-U", "-V", VERBOSITY, "-S"],
    );
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

/// Renames each test of `group` that `left_out` names in the copy's group
/// file, so the framework no longer counts it a test; each must be found.
fn skip<S: AsRef<str> + Ord>(group_file: &Path, group: &str, left_out: &BTreeSet<S>) {
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
            if !left_out.iter().any(|name| name.as_ref() == member) {
                return line.to_string();
            }
            skipped.insert(member);
            let at = line.len() - rest.len() + name_start;
            format!("{}SKIPPED_{}", &line[..at], &line[at..])
        })
        .collect();
    let wanted: BTreeSet<String> = left_out
        .iter()
        .map(AsRef::as_ref)
        .filter(|name| name.starts_with(&format!("{group}.")))
        .map(str::to_string)
        .collect();
    assert_eq!(
        skipped, wanted,
        "the left-out tests of {group} renamed in the copy"
    );
    fs::write(group_file, lines.join("\n")).expect("cannot rewrite the copied group file");
}

fn excerpt(bytes: &[u8]) -> String {
    let text = String::from_utf8_lossy(bytes);
    text.chars().take(300).collect()
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
        let group_file = run
            .join(format!("ooRexx/API/oo/{group}.testGroup"))
            .to_string_lossy()
            .into_owned();
        for name in test_names(&oracle, &run, group) {
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
            fresh_copy(&run);
            let theirs = run_oracle(&oracle, &run, &args);
            fresh_copy(&run);
            let ours = run_crate(&run, &args);
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
        let group_file = run.join(format!("ooRexx/API/oo/{group}.testGroup"));
        let shown = group_file.to_string_lossy().into_owned();
        let args = ["-f", shown.as_str(), "-U", "-V", VERBOSITY, "-S"];
        fresh_copy(&run);
        skip(&group_file, group, &left_out());
        let theirs = run_oracle(&oracle, &run, &args);
        fresh_copy(&run);
        skip(&group_file, group, &left_out());
        let ours = run_crate(&run, &args);
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

/// Whether a run's summary reports a pass and it exited 0.
fn passes(run: &Run) -> bool {
    run.status == Some(0) && outcome(&run.stdout).starts_with("pass,")
}
