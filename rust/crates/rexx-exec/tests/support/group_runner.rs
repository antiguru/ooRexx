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

//! Runs ooTest groups through `testOORexx.rex`'s single-group form on both
//! sides, one test per run, each from a fresh copy of the framework and the
//! group's directory at the same path, and compares the two. Declared with
//! `#[path]` by the harnesses that use it, because it needs the crate root's
//! `watchdog` module, which not every harness declares.
//!
//! Both sides run at `-V 2`, which prints the assertion count and each
//! failure's and error's detail, so a test that fails differently, or runs
//! fewer assertions, differs. `-U` starts no ticker. The copy's `ooTest.frm`
//! has `printSummary`'s `rxfuncquery` probes removed: on the oracle they reach
//! rxapi (`interpreter/package/PackageManager.cpp:618-635`), and neither name
//! they ask for is registered on either side, so the summary prints the same
//! lines without them.
//!
//! A test whose source uses a named queue or the function and macro-space
//! registries ([`reaching_rxapi`]) runs on neither side, the listing run
//! included: on the oracle those read or change the state rxapi's daemon
//! keeps between processes. The session queue lives for the process and is
//! not excluded. The oracle still asks rxapi's macro space, read-only, on each
//! `::REQUIRES` and external call (`interpreter/package/PackageManager.cpp:747`,
//! `interpreter/execution/RexxActivation.cpp:2996`), through
//! `rexxapi/client/MacroSpaceApi.cpp:217-230` and `LocalAPIManager.cpp:56-74`
//! and `:177-243`, which starts the daemon if it is not running; no ooTest run
//! on the oracle can avoid that.

// Pulled in by more than one test binary, each of which uses part of it.
#![allow(dead_code)]

use std::collections::BTreeSet;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use rexx_exec::Invocation;

use crate::support::oracle::{self, did_not_finish};
use crate::watchdog;

pub const GATE_ENV: &str = "REXX_CORPUS_GATE";

pub fn gate_mode() -> bool {
    match env::var(GATE_ENV) {
        Ok(value) => !value.is_empty() && value != "0",
        Err(_) => false,
    }
}

/// See the module doc.
pub const VERBOSITY: &str = "2";

/// The summary lines whose value is a duration.
const TIMINGS: &[&str] = &[
    "File search:",
    "Suite construction:",
    "Test execution:",
    "Total time:",
];

/// The detail headers whose rest of line is a timestamp.
const STAMPED: &[&str] = &["[failure] ", "[error] "];

/// How the crate's side schedules activities.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SwitchMode {
    /// The scheduler as it ships.
    None,
    /// A switch at every clause boundary, as `rexx_exec::SwitchMode` names it.
    EveryOpportunity,
}

/// One run's three descriptors.
pub struct Run {
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub status: Option<i32>,
}

/// What comparing the two sides on one test found.
pub enum Outcome {
    /// Identical on every descriptor, and the test passes.
    Pass,
    /// Identical on every descriptor, and the test does not pass.
    FailsOnBoth,
    /// The descriptors differ; both sides' output is kept.
    Differ { oracle: Run, ours: Run },
    /// This crate refused the program; the message is its refusal.
    Refused { message: String, oracle: Run },
    /// The test reaches rxapi and runs on neither side.
    NotRun,
}

impl Outcome {
    /// One word for a table row.
    pub fn label(&self) -> &'static str {
        match self {
            Outcome::Pass => "pass",
            Outcome::FailsOnBoth => "fails on both",
            Outcome::Differ { .. } => "differ",
            Outcome::Refused { .. } => "refused",
            Outcome::NotRun => "not run",
        }
    }
}

/// A test's name and what comparing the sides found.
pub struct TestResult {
    pub test: String,
    pub outcome: Outcome,
}

pub fn worktree() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

/// The directory of the ooTest sources the groups are under.
fn groups_root() -> PathBuf {
    worktree().join("ootest/ooRexx")
}

pub fn read_lossy(path: &Path) -> String {
    let bytes = fs::read(path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    String::from_utf8_lossy(&bytes).into_owned()
}

/// Each `::METHOD` and `::ROUTINE` of `text`, lower-cased, with its body.
pub fn directive_bodies(text: &str) -> Vec<(String, String)> {
    let mut bodies: Vec<(String, String)> = Vec::new();
    let mut current: Option<(String, String)> = None;
    for line in text.lines() {
        let trimmed = line.trim_start();
        if let Some(directive) = trimmed.strip_prefix("::") {
            bodies.extend(current.take());
            let mut parts = directive.split_whitespace();
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

/// Whether a lower-cased source line uses a named queue or a registry, the
/// daemon's persistent state. The session queue lives for the process.
pub fn uses_rxapi(line: &str) -> bool {
    const CALLS: &[&str] = &["rxqueue", "rxfuncadd", "rxfuncdrop", "rxfuncquery"];
    if line.contains(".rexxqueue") || line.contains("rexxmacro") {
        return true;
    }
    if CALLS.iter().any(|call| line.contains(&format!("{call}("))) {
        return true;
    }
    line.split(';').any(|clause| {
        let mut tokens = clause.split_whitespace();
        tokens.any(|word| word == "call")
            && tokens.next().is_some_and(|target| CALLS.contains(&target))
    })
}

/// The `GROUP.TEST` of each test of `groups`, files in `dir` (below
/// `ootest/ooRexx`), whose source uses a named queue or the function or
/// macro-space registries: an instruction or a call in its own method body,
/// or through a `::METHOD` or `::ROUTINE` of the group file, or of a file it
/// requires or loads from its own directory, that does.
pub fn reaching_rxapi(dir: &str, groups: &[&str]) -> BTreeSet<String> {
    let dir = groups_root().join(dir);
    let mut found = BTreeSet::new();
    for group in groups {
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

pub fn copy_tree(from: &Path, to: &Path) {
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

/// Empties `run` and lays the framework and `dir` (below `ootest/ooRexx`)
/// into it. `rxregexp.cls`, which `ooTest.frm` requires, sits beside the
/// driver.
pub fn fresh_copy(run: &Path, dir: &str) {
    if run.exists() {
        fs::remove_dir_all(run).unwrap_or_else(|e| panic!("cannot empty {}: {e}", run.display()));
    }
    let ootest = worktree().join("ootest");
    copy_tree(&ootest.join("framework"), &run.join("framework"));
    copy_tree(
        &ootest.join("ooRexx").join(dir),
        &run.join("ooRexx").join(dir),
    );
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
pub fn masked(stdout: &[u8]) -> Vec<u8> {
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
pub fn outcome(stdout: &[u8]) -> String {
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
pub fn first_difference(theirs: &[u8], ours: &[u8]) -> String {
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

pub fn excerpt(bytes: &[u8]) -> String {
    let text = String::from_utf8_lossy(bytes);
    text.chars().take(300).collect()
}

/// Whether a run's summary reports a pass and it exited 0.
pub fn passes(run: &Run) -> bool {
    run.status == Some(0) && outcome(&run.stdout).starts_with("pass,")
}

/// Whether the two runs agree on every descriptor, stdout masked.
pub fn agree(theirs: &Run, ours: &Run) -> bool {
    masked(&theirs.stdout) == masked(&ours.stdout)
        && theirs.stderr == ours.stderr
        && theirs.status == ours.status
}

/// How long one oracle run of a test may take: `Alarm` TEST_BASE_ALARM waits
/// out its alarms, about two seconds each.
pub const ORACLE_TEST_DEADLINE: std::time::Duration = std::time::Duration::from_secs(30);

pub fn run_oracle(oracle: &oracle::Oracle, run: &Path, args: &[&str]) -> Run {
    run_oracle_within(oracle, run, args, ORACLE_TEST_DEADLINE)
}

/// [`run_oracle`] under `deadline`.
pub fn run_oracle_within(
    oracle: &oracle::Oracle,
    run: &Path,
    args: &[&str],
    deadline: std::time::Duration,
) -> Run {
    let outcome = oracle.run_within(&run.join("testOORexx.rex"), args, None, deadline);
    let status = (!did_not_finish(&outcome)).then(|| outcome.expect_exit_code());
    Run {
        stdout: outcome.stdout,
        stderr: outcome.stderr,
        status,
    }
}

pub fn run_crate(run: &Path, args: &[&str], mode: SwitchMode) -> Run {
    run_crate_within(run, args, mode, watchdog::ROW_DEADLINE)
}

/// [`run_crate`] under `deadline`.
pub fn run_crate_within(
    run: &Path,
    args: &[&str],
    mode: SwitchMode,
    deadline: std::time::Duration,
) -> Run {
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
    let invocation = Invocation::with_argument(args.join(" ").into_bytes());
    let invocation = match mode {
        SwitchMode::None => invocation,
        SwitchMode::EveryOpportunity => {
            invocation.with_switch_mode(rexx_exec::SwitchMode::EveryOpportunity)
        }
    }
    .with_directory(run.to_path_buf())
    .with_environment(environment)
    .with_deadline(deadline);
    let abandon = deadline + (watchdog::ROW_ABANDON - watchdog::ROW_DEADLINE);
    let outcome = watchdog::run_bounded_with(&driver.to_string_lossy(), text, invocation, abandon);
    let status = (!watchdog::did_not_finish(&outcome)).then_some(outcome.exit_code);
    Run {
        stdout: outcome.stdout,
        stderr: outcome.stderr,
        status,
    }
}

/// Renames each test of `group` that `left_out` names in the copy's group
/// file, so the framework no longer counts it a test; each must be found.
pub fn skip<S: AsRef<str> + Ord>(group_file: &Path, group: &str, left_out: &BTreeSet<S>) {
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
                .take_while(|c| {
                    if quoted {
                        !matches!(c, '\'' | '"')
                    } else {
                        c.is_ascii_alphanumeric() || *c == '_'
                    }
                })
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

/// The path of `group`'s file in the copy `run`.
pub fn group_file(run: &Path, dir: &str, group: &str) -> String {
    run.join(format!("ooRexx/{dir}/{group}.testGroup"))
        .to_string_lossy()
        .into_owned()
}

/// The group's tests in the order the oracle runs them, from `-S`'s lines,
/// which runs them all, so the tests of `not_run` are renamed out first.
pub fn test_names(
    oracle: &oracle::Oracle,
    run: &Path,
    dir: &str,
    group: &str,
    not_run: &BTreeSet<String>,
) -> Vec<String> {
    let group_file = group_file(run, dir, group);
    fresh_copy(run, dir);
    skip(Path::new(&group_file), group, not_run);
    let listed = run_oracle(
        oracle,
        run,
        &["-f", &group_file, "-U", "-V", VERBOSITY, "-S"],
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

/// The group's tests, sorted and upper case, from the `::METHOD`s named
/// `test...` of its `ooTestCase` subclasses: for a group whose whole-group
/// listing run outlasts the oracle's deadline. A test class inheriting from
/// another test class, or a `::CLASS` directive split across lines, is not
/// listed; resource bodies are skipped.
pub fn source_test_names(dir: &str, group: &str) -> Vec<String> {
    let text = read_lossy(&groups_root().join(dir).join(format!("{group}.testGroup")));
    let mut in_test_class = false;
    let mut resource_end: Option<String> = None;
    let mut names = Vec::new();
    for line in text.lines() {
        if let Some(marker) = &resource_end {
            if line.starts_with(marker.as_str()) {
                resource_end = None;
            }
            continue;
        }
        let lower = line.trim_start().to_ascii_lowercase();
        if lower.starts_with("::resource") {
            resource_end = Some(resource_marker(line.trim_start()));
        } else if lower.starts_with("::class") {
            in_test_class = lower.contains("ootestcase");
        } else if let Some(rest) = lower.strip_prefix("::method") {
            let name = rest
                .split_whitespace()
                .next()
                .unwrap_or("")
                .trim_matches(['\'', '"']);
            if in_test_class && name.starts_with("test") {
                names.push(name.to_ascii_uppercase());
            }
        }
    }
    names.sort();
    names
}

/// The line prefix that ends the resource `directive` opens: its `END`
/// marker, a quoted string as written or a symbol in upper case, else
/// `::END` (`LanguageParser::checkMarker`, `parser/LanguageParser.cpp:959`).
/// A hex or binary string marker is not decoded.
fn resource_marker(directive: &str) -> String {
    let mut tokens = Vec::new();
    let mut rest = directive.trim_start_matches(':');
    while let Some(start) = rest.find(|c: char| !c.is_whitespace()) {
        rest = &rest[start..];
        let quote = rest.chars().next().filter(|c| matches!(c, '\'' | '"'));
        let end = match quote {
            Some(quote) => rest[1..].find(quote).map_or(rest.len(), |at| at + 2),
            None => rest.find(char::is_whitespace).unwrap_or(rest.len()),
        };
        tokens.push(&rest[..end]);
        rest = &rest[end..];
    }
    match tokens.as_slice() {
        [_, _, keyword, marker, ..] if keyword.eq_ignore_ascii_case("end") => {
            match marker.strip_prefix(['\'', '"']) {
                Some(quoted) => quoted[..quoted.len() - 1].to_string(),
                None => marker.to_ascii_uppercase(),
            }
        }
        _ => "::END".to_string(),
    }
}

/// Runs each of `tests` of `group` (files in `dir`, below `ootest/ooRexx`),
/// one test per run, on the oracle and on this crate under `mode`, and
/// compares the sides. `alter` rewrites this crate's copy after it is laid
/// out, for the runner's own negative control. A test of the group that
/// reaches rxapi is not run.
pub fn run_tests(
    oracle: &oracle::Oracle,
    run: &Path,
    dir: &str,
    group: &str,
    tests: &[String],
    mode: SwitchMode,
    alter: Option<&dyn Fn(&Path)>,
) -> Vec<TestResult> {
    let not_run = reaching_rxapi(dir, &[group]);
    let group_file = group_file(run, dir, group);
    tests
        .iter()
        .map(|name| {
            let outcome = if not_run.contains(&format!("{group}.{name}")) {
                Outcome::NotRun
            } else {
                let args = [
                    "-f",
                    group_file.as_str(),
                    "-U",
                    "-V",
                    VERBOSITY,
                    "-t",
                    name.as_str(),
                ];
                fresh_copy(run, dir);
                let theirs = run_oracle(oracle, run, &args);
                fresh_copy(run, dir);
                if let Some(alter) = alter {
                    alter(run);
                }
                let ours = run_crate(run, &args, mode);
                assert!(
                    theirs.status.is_some(),
                    "the oracle did not finish {group}.{name}"
                );
                assert!(
                    !String::from_utf8_lossy(&theirs.stdout)
                        .lines()
                        .any(|line| line.starts_with("Tests ran:")
                            && line.trim_end().ends_with(" 0")),
                    "{group}.{name} is not a test of the group: the oracle ran none"
                );
                compare(theirs, ours)
            };
            TestResult {
                test: name.clone(),
                outcome,
            }
        })
        .collect()
}

/// How many rows [`rows_in_parallel`] runs at once, where set; the default
/// is [`ROWS_DEFAULT`].
pub const ROWS_ENV: &str = "REXX_GROUP_ROWS";

pub const ROWS_DEFAULT: usize = 6;

/// The pool every [`rows_in_parallel`] of the process shares, so the bound
/// holds across tests.
fn row_pool() -> &'static rayon::ThreadPool {
    static POOL: std::sync::OnceLock<rayon::ThreadPool> = std::sync::OnceLock::new();
    POOL.get_or_init(|| {
        let rows = env::var(ROWS_ENV).map_or(ROWS_DEFAULT, |value| {
            value
                .parse()
                .unwrap_or_else(|_| panic!("{ROWS_ENV} is not a count: {value:?}"))
        });
        rayon::ThreadPoolBuilder::new()
            .num_threads(rows)
            .thread_name(|at| format!("group-row-{at}"))
            .build()
            .expect("cannot build the row pool")
    })
}

/// How many pooled and quiet rows are running, and how many pooled rows wait.
struct Rows {
    pooled: usize,
    quiet: usize,
    pooled_waiting: usize,
}

static ROWS: std::sync::Mutex<Rows> = std::sync::Mutex::new(Rows {
    pooled: 0,
    quiet: 0,
    pooled_waiting: 0,
});

static TURN: std::sync::Condvar = std::sync::Condvar::new();

/// A running row's place in [`ROWS`], given back on drop.
struct RowTurn {
    quiet: bool,
}

impl RowTurn {
    /// Waits until no row of the other kind runs; a quiet row also waits
    /// while a pooled row is waiting.
    fn take(quiet: bool) -> RowTurn {
        let wait = |rows| {
            TURN.wait(rows)
                .unwrap_or_else(std::sync::PoisonError::into_inner)
        };
        let mut rows = ROWS
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if quiet {
            while rows.pooled > 0 || rows.pooled_waiting > 0 {
                rows = wait(rows);
            }
            rows.quiet += 1;
        } else {
            rows.pooled_waiting += 1;
            while rows.quiet > 0 {
                rows = wait(rows);
            }
            rows.pooled_waiting -= 1;
            rows.pooled += 1;
        }
        RowTurn { quiet }
    }
}

impl Drop for RowTurn {
    fn drop(&mut self) {
        let mut rows = ROWS
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if self.quiet {
            rows.quiet -= 1;
        } else {
            rows.pooled -= 1;
        }
        TURN.notify_all();
    }
}

/// Runs `body` while no pooled row of any [`rows_in_parallel`] runs.
pub fn on_a_quiet_machine<T>(body: impl FnOnce() -> T) -> T {
    let _turn = RowTurn::take(true);
    body()
}

/// Runs `row` over each of `rows` in its own scratch copy,
/// `<name>-<pid>-<index>/run` below the target's temporary directory, removed
/// after the row. The rows `quiet` selects run after the others, each on its
/// own thread and [`on_a_quiet_machine`]; the rest run in the shared pool.
/// Answers the results in `rows`' order.
pub fn rows_in_parallel<R: Sync, T: Send>(
    name: &str,
    rows: &[R],
    quiet: impl Fn(&R) -> bool,
    row: impl Fn(&Path, &R) -> T + Sync,
) -> Vec<T> {
    use rayon::prelude::*;

    let one = |at: usize| {
        let run = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("{name}-{}-{at}", std::process::id()))
            .join("run");
        let result = row(&run, &rows[at]);
        let dir = run.parent().expect("a parent");
        if dir.exists() {
            fs::remove_dir_all(dir).expect("cannot remove the run");
        }
        result
    };
    let (alone, pooled): (Vec<usize>, Vec<usize>) =
        (0..rows.len()).partition(|&at| quiet(&rows[at]));
    let pooled: Vec<(usize, T)> = row_pool().install(|| {
        pooled
            .into_par_iter()
            .map(|at| {
                let _turn = RowTurn::take(false);
                (at, one(at))
            })
            .collect()
    });
    let mut results: Vec<Option<T>> = (0..rows.len()).map(|_| None).collect();
    for (at, result) in pooled {
        results[at] = Some(result);
    }
    let one = &one;
    let alone: Vec<(usize, T)> = std::thread::scope(|scope| {
        let rows: Vec<_> = alone
            .into_iter()
            .map(|at| scope.spawn(move || (at, on_a_quiet_machine(|| one(at)))))
            .collect();
        rows.into_iter()
            .map(|row| {
                row.join()
                    .unwrap_or_else(|panic| std::panic::resume_unwind(panic))
            })
            .collect()
    });
    for (at, result) in alone {
        results[at] = Some(result);
    }
    results
        .into_iter()
        .map(|result| result.expect("every row ran"))
        .collect()
}

fn compare(theirs: Run, ours: Run) -> Outcome {
    let stderr = String::from_utf8_lossy(&ours.stderr).into_owned();
    if let Some(line) = stderr.lines().find(|line| line.starts_with("rexx-exec: ")) {
        return Outcome::Refused {
            message: line["rexx-exec: ".len()..].to_string(),
            oracle: theirs,
        };
    }
    if !agree(&theirs, &ours) {
        Outcome::Differ {
            oracle: theirs,
            ours,
        }
    } else if passes(&ours) {
        Outcome::Pass
    } else {
        Outcome::FailsOnBoth
    }
}
