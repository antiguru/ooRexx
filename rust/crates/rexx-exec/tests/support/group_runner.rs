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
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SwitchMode {
    /// The scheduler as it ships.
    None,
    /// A switch at every clause boundary, as `rexx_exec::SwitchMode` names it.
    EveryOpportunity,
    /// The simulation mode, its `REXX_SWITCH_MODE` value (`sim:SEED,...`): the
    /// run is a `rexx` process of its own, see [`run_crate_process`].
    Sim(String),
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
pub fn groups_root() -> PathBuf {
    worktree().join("ootest/ooRexx")
}

/// The bytes of `path`. A copy that is written back is read with this: a
/// group file or framework file may hold bytes that are not UTF-8 (a literal
/// `0xAA` in `C2X.testGroup`), and the copy must keep them.
pub fn read_bytes(path: &Path) -> Vec<u8> {
    fs::read(path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}

/// `path` as text, invalid UTF-8 replaced: for reading only, never for a
/// file that is written back.
pub fn read_lossy(path: &Path) -> String {
    String::from_utf8_lossy(&read_bytes(path)).into_owned()
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
    reaching_rxapi_in(&groups_root(), dir, groups)
}

/// [`reaching_rxapi`] over the groups below `root` instead of `ootest/ooRexx`.
pub fn reaching_rxapi_in(root: &Path, dir: &str, groups: &[&str]) -> BTreeSet<String> {
    let dir = root.join(dir);
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
    fresh_copy_from(run, &groups_root(), dir);
}

/// [`fresh_copy`] with `dir` taken from below `root` instead of
/// `ootest/ooRexx`; the framework is still `ootest`'s.
pub fn fresh_copy_from(run: &Path, root: &Path, dir: &str) {
    if run.exists() {
        fs::remove_dir_all(run).unwrap_or_else(|e| panic!("cannot empty {}: {e}", run.display()));
    }
    let ootest = worktree().join("ootest");
    copy_tree(&ootest.join("framework"), &run.join("framework"));
    copy_tree(&root.join(dir), &run.join("ooRexx").join(dir));
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

/// Whether `haystack` holds `needle`.
fn contains(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}

/// Removes each `rxfuncquery` test from the copied `ooTest.frm`, with the
/// comment above it and the assignment it guards. The file is read and
/// written as bytes, so a byte that is not UTF-8 survives.
pub fn without_rxfuncquery(frm: &Path) {
    let text = read_bytes(frm);
    let lines: Vec<&[u8]> = text.split(|&byte| byte == b'\n').collect();
    let hits: Vec<usize> = (0..lines.len())
        .filter(|&at| contains(lines[at], b"rxfuncquery("))
        .collect();
    assert!(!hits.is_empty(), "ooTest.frm has no rxfuncquery to remove");
    let mut dropped = BTreeSet::new();
    for at in hits {
        assert!(
            at > 0
                && lines[at - 1].trim_ascii_start().starts_with(b"--")
                && lines
                    .get(at + 1)
                    .is_some_and(|next| next.trim_ascii_start().starts_with(b"versions[")),
            "ooTest.frm line {} is not a commented rxfuncquery test guarding one assignment",
            at + 1
        );
        dropped.extend([at - 1, at, at + 1]);
    }
    let kept: Vec<&[u8]> = (0..lines.len())
        .filter(|at| !dropped.contains(at))
        .map(|at| lines[at])
        .collect();
    fs::write(frm, kept.join(&b'\n')).expect("cannot rewrite the copied ooTest.frm");
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
    let path = env::join_paths(
        std::iter::once(oracle::oracle_root().join("bin"))
            .chain(env::var_os("PATH").iter().flat_map(env::split_paths)),
    )
    .expect("a PATH");
    let outcome = oracle.run_within_env(
        &run.join("testOORexx.rex"),
        args,
        None,
        deadline,
        &[("PATH", path)],
    );
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
    if let SwitchMode::Sim(config) = &mode {
        return run_crate_sim(run, args, config, deadline).run;
    }
    let driver = run.join("testOORexx.rex");
    let environment = crate_environment(run);
    let text = fs::read(&driver).expect("the copied driver");
    let mut environment = environment;
    if mode == SwitchMode::EveryOpportunity {
        environment.push((b"REXX_SWITCH_MODE".to_vec(), b"every".to_vec()));
    }
    let invocation = Invocation::with_argument(args.join(" ").into_bytes());
    let invocation = match mode {
        SwitchMode::None | SwitchMode::Sim(_) => invocation,
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

/// The environment a run of the group copy `run` gets here: the process's
/// own without `LD_LIBRARY_PATH`, `PATH`, `REXX_SWITCH_MODE` and the
/// `REXX_SIM_` variables (a sim child gets its own from the command that
/// starts it), then the
/// oracle's library directory and [`interpreter_bin`] ahead of `PATH`.
fn crate_environment(run: &Path) -> Vec<(Vec<u8>, Vec<u8>)> {
    crate_environment_with_path(
        run,
        env::var_os("PATH")
            .iter()
            .flat_map(env::split_paths)
            .collect(),
    )
}

/// [`crate_environment`] with `rest` after [`interpreter_bin`] on `PATH`
/// instead of the process's own `PATH`.
pub fn crate_environment_with_path(run: &Path, rest: Vec<PathBuf>) -> Vec<(Vec<u8>, Vec<u8>)> {
    let lib = oracle::oracle_root().join("lib");
    let bin = interpreter_bin(run);
    let path = env::join_paths(std::iter::once(bin).chain(rest)).expect("a PATH");
    let mut environment: Vec<(Vec<u8>, Vec<u8>)> = env::vars()
        .filter(|(name, _)| {
            !matches!(
                name.as_str(),
                "LD_LIBRARY_PATH" | "PATH" | "REXX_SWITCH_MODE"
            ) && !name.starts_with("REXX_SIM_")
        })
        .map(|(name, value)| (name.into_bytes(), value.into_bytes()))
        .collect();
    environment.push((
        b"LD_LIBRARY_PATH".to_vec(),
        lib.to_string_lossy().into_owned().into_bytes(),
    ));
    environment.push((
        b"PATH".to_vec(),
        path.to_string_lossy().into_owned().into_bytes(),
    ));
    environment
}

/// [`run_crate_within`] in the simulation mode `config`, with how the
/// process ended and how long it took.
pub fn run_crate_sim(
    run: &Path,
    args: &[&str],
    config: &str,
    deadline: std::time::Duration,
) -> ProcessRun {
    let mut words = vec![run.join("testOORexx.rex").to_string_lossy().into_owned()];
    words.extend(args.iter().map(|word| (*word).to_string()));
    run_crate_process(
        &words,
        run,
        Some(crate_environment(run)),
        config,
        None,
        deadline,
    )
}

/// How a [`run_crate_process`] ended.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ended {
    /// It exited with this status.
    Exited(i32),
    /// It died of this signal, not one sent here.
    Signaled(i32),
    /// Its process group was killed at the deadline.
    Killed,
}

/// One [`run_crate_process`]: its descriptors (status `None` unless it
/// exited), how it ended and how long it took.
pub struct ProcessRun {
    pub run: Run,
    pub ended: Ended,
    pub took: std::time::Duration,
}

/// Runs `rexx` on `words` (the program, then its arguments) from `cwd`
/// with `REXX_SWITCH_MODE` set to `mode`, in a process group of its own that
/// is killed at `deadline`; a command a simulation run starts runs in a group
/// of its own, which that kill does not reach. `environment`
/// replaces the process's own where given, else is inherited; `stdin` is
/// written to its standard input, which is otherwise empty.
pub fn run_crate_process(
    words: &[String],
    cwd: &Path,
    environment: Option<Vec<(Vec<u8>, Vec<u8>)>>,
    mode: &str,
    stdin: Option<&[u8]>,
    deadline: std::time::Duration,
) -> ProcessRun {
    let mut command = std::process::Command::new(env!("CARGO_BIN_EXE_rexx"));
    if let Some(environment) = environment {
        set_environment(&mut command, environment);
    }
    command
        .args(words)
        .current_dir(cwd)
        .env("REXX_SWITCH_MODE", mode);
    run_process(command, stdin, deadline)
}

/// Replaces `command`'s environment with `environment`.
pub fn set_environment(command: &mut std::process::Command, environment: Vec<(Vec<u8>, Vec<u8>)>) {
    use std::os::unix::ffi::OsStringExt as _;
    command
        .env_clear()
        .envs(environment.into_iter().map(|(name, value)| {
            (
                std::ffi::OsString::from_vec(name),
                std::ffi::OsString::from_vec(value),
            )
        }));
}

/// Runs `command` in a process group of its own that is killed at
/// `deadline`, and again once it has ended, so a process it left running
/// does not hold its pipes open; a command it starts in a group of its own is
/// not reached by that kill. `stdin` is written to its standard input, which
/// is otherwise empty.
#[expect(
    clippy::disallowed_methods,
    reason = "this harness bounds a real process"
)]
pub fn run_process(
    mut command: std::process::Command,
    stdin: Option<&[u8]>,
    deadline: std::time::Duration,
) -> ProcessRun {
    use std::io::{Read as _, Write as _};
    use std::os::unix::process::{CommandExt as _, ExitStatusExt as _};
    use std::process::Stdio;

    let words = format!(
        "{:?} {:?}",
        command.get_program(),
        command.get_args().collect::<Vec<_>>()
    );
    command
        .process_group(0)
        .stdin(if stdin.is_some() {
            Stdio::piped()
        } else {
            Stdio::null()
        })
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let start = std::time::Instant::now();
    let mut child = command
        .spawn()
        .unwrap_or_else(|e| panic!("cannot start {words}: {e}"));
    let group = rustix::process::Pid::from_child(&child);
    let kill_group = || {
        // The group may be gone already; that is the outcome wanted.
        let _ = rustix::process::kill_process_group(group, rustix::process::Signal::KILL);
    };
    if let Some(bytes) = stdin {
        let mut pipe = child.stdin.take().expect("stdin was piped");
        let bytes = bytes.to_vec();
        // A program that stops reading early closes the pipe; what it read is
        // the outcome. A thread of its own, so a program that never reads
        // cannot hold the deadline off.
        std::thread::spawn(move || {
            let _ = pipe.write_all(&bytes);
        });
    }
    let reader = |mut pipe: Box<dyn std::io::Read + Send>| {
        let (sender, receiver) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let mut bytes = Vec::new();
            let _ = pipe.read_to_end(&mut bytes);
            let _ = sender.send(bytes);
        });
        receiver
    };
    let stdout = reader(Box::new(child.stdout.take().expect("stdout was piped")));
    let stderr = reader(Box::new(child.stderr.take().expect("stderr was piped")));
    let ended = loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                break match (status.code(), status.signal()) {
                    (Some(code), _) => Ended::Exited(code),
                    (None, Some(signal)) => Ended::Signaled(signal),
                    (None, None) => Ended::Signaled(0),
                };
            }
            Ok(None) if start.elapsed() >= deadline => {
                kill_group();
                let _ = child.wait();
                break Ended::Killed;
            }
            Ok(None) => std::thread::sleep(std::time::Duration::from_millis(5)),
            Err(e) => panic!("cannot wait for {words}: {e}"),
        }
    };
    let took = start.elapsed();
    // A child the program started and left running holds the pipes open.
    kill_group();
    let grace = std::time::Duration::from_secs(10);
    let stdout = stdout.recv_timeout(grace).unwrap_or_default();
    let stderr = stderr.recv_timeout(grace).unwrap_or_default();
    ProcessRun {
        run: Run {
            stdout,
            stderr,
            status: match ended {
                Ended::Exited(code) => Some(code),
                Ended::Signaled(_) | Ended::Killed => None,
            },
        },
        ended,
        took,
    }
}

/// A directory beside `run` holding `rexx`, this crate's own, so a
/// test running `rexx` as a command runs the side under test; the oracle's
/// side gets its own `bin` directory instead.
///
/// `rexx` is a hard link (a copy where one cannot be made), not a symlink,
/// because `.RexxInfo~executable` resolves links: the test framework finds
/// the samples at `../samples` from the executable's own directory, which
/// is the oracle's install layout, and a `samples` link beside this
/// directory names the oracle's.
pub fn interpreter_bin(run: &Path) -> PathBuf {
    let bin = run.parent().expect("a parent").join("bin-rexx");
    fs::create_dir_all(&bin).expect("the interpreter bin directory");
    let rexx = bin.join("rexx");
    let ours = env!("CARGO_BIN_EXE_rexx");
    let current = || {
        fs::metadata(&rexx)
            .ok()
            .zip(fs::metadata(ours).ok())
            .is_some_and(|(theirs, ours)| {
                use std::os::unix::fs::MetadataExt as _;
                (theirs.dev(), theirs.ino()) == (ours.dev(), ours.ino())
            })
    };
    if !current() {
        // A copy or an older link left by an earlier build is replaced, and
        // the copy below never writes onto an existing file, which could be
        // a link to `ours` itself.
        match fs::remove_file(&rexx) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => {
                panic!("cannot replace {}: {e}", rexx.display())
            }
            _ => {}
        }
        // A run sharing this directory may link it between the removal and
        // the link, which leaves the file this one would have made.
        if let Err(link) = fs::hard_link(ours, &rexx)
            && !current()
        {
            assert!(
                !rexx.exists(),
                "{} appeared while it was replaced",
                rexx.display()
            );
            fs::copy(ours, &rexx).unwrap_or_else(|copy| {
                panic!("cannot link ({link}) or copy ({copy}) {}", rexx.display())
            });
        }
    }
    let samples = bin.parent().expect("a parent").join("samples");
    match std::os::unix::fs::symlink(oracle::oracle_root().join("samples"), &samples) {
        Err(e) if e.kind() != std::io::ErrorKind::AlreadyExists => {
            panic!("cannot link {}: {e}", samples.display())
        }
        _ => {}
    }
    bin
}

/// Renames each test of `group` that `left_out` names in the copy's group
/// file, so the framework no longer counts it a test; each must be found.
/// The file is matched and rewritten as bytes: only the renamed methods'
/// lines change, whatever bytes the rest holds.
pub fn skip<S: AsRef<str> + Ord>(group_file: &Path, group: &str, left_out: &BTreeSet<S>) {
    let (text, skipped) = renamed(&read_bytes(group_file), group, left_out);
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
    fs::write(group_file, text).expect("cannot rewrite the copied group file");
}

/// The group file `text` with each test of `group` that `left_out` names
/// renamed as [`skip`] renames it, and the `GROUP.TEST` names it renamed.
pub fn renamed<S: AsRef<str> + Ord>(
    text: &[u8],
    group: &str,
    left_out: &BTreeSet<S>,
) -> (Vec<u8>, BTreeSet<String>) {
    let mut skipped = BTreeSet::new();
    let lines: Vec<Vec<u8>> = text
        .split(|&byte| byte == b'\n')
        .map(|line| {
            let trimmed = line.trim_ascii_start();
            let Some(rest) = trimmed
                .get(..8)
                .filter(|head| head.eq_ignore_ascii_case(b"::method"))
                .map(|_| &trimmed[8..])
            else {
                return line.to_vec();
            };
            let rest = rest.trim_ascii_start();
            let quoted = rest.starts_with(b"'") || rest.starts_with(b"\"");
            let name_start = usize::from(quoted);
            let name: Vec<u8> = rest[name_start..]
                .iter()
                .copied()
                .take_while(|&byte| {
                    if quoted {
                        !matches!(byte, b'\'' | b'"')
                    } else {
                        byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'!' | b'?' | b'.')
                    }
                })
                .collect();
            let member = format!(
                "{group}.{}",
                String::from_utf8_lossy(&name).to_ascii_uppercase()
            );
            if !left_out.iter().any(|name| name.as_ref() == member) {
                return line.to_vec();
            }
            skipped.insert(member);
            let at = line.len() - rest.len() + name_start;
            [&line[..at], b"SKIPPED_".as_slice(), &line[at..]].concat()
        })
        .collect();
    (lines.join(&b'\n'), skipped)
}

/// The line `TestCase~execute` starts each test with, in the copy's
/// `OOREXXUNIT.CLS`, and what it becomes: the same line, which also
/// writes `started NAME` to stderr, so line numbers are unchanged.
const START_LINE: &[u8] = b"  aTestResult~startTest(self)       -- remember test started\n  self~setUp                        -- make sure setup is invoked before test\n";
const START_MARKED: &[u8] = b"  aTestResult~startTest(self); .error~say('started' fName) -- remember test started\n  self~setUp                        -- make sure setup is invoked before test\n";

/// Makes the copy at `at` write `started NAME` to stderr as each test
/// starts. The file is read and written as bytes.
pub fn mark_starts(at: &Path) {
    mark_starts_with(at, START_MARKED);
}

/// Makes the copy at `at` append `started NAME` to `marker` as each test
/// starts, closing it after each line, so the line is on disk before the
/// test runs. `marker`'s path holds no quote.
pub fn mark_starts_in_file(at: &Path, marker: &Path) {
    let marker = marker.to_str().expect("a UTF-8 marker path");
    assert!(
        !marker.contains(['\'', '"']),
        "a marker path without quotes"
    );
    let marked = format!(
        "  aTestResult~startTest(self); .stream~new('{marker}')~~lineout('started' fName)~close -- remember test started\n  self~setUp                        -- make sure setup is invoked before test\n"
    );
    mark_starts_with(at, marked.as_bytes());
}

fn mark_starts_with(at: &Path, marked: &[u8]) {
    let file = at.join("framework/OOREXXUNIT.CLS");
    let text = read_bytes(&file);
    let hits: Vec<usize> = text
        .windows(START_LINE.len())
        .enumerate()
        .filter(|(_, window)| *window == START_LINE)
        .map(|(at, _)| at)
        .collect();
    assert_eq!(hits.len(), 1, "one test start in {}", file.display());
    let at = hits[0];
    let marked = [&text[..at], marked, &text[at + START_LINE.len()..]].concat();
    fs::write(&file, marked).expect("cannot rewrite the copied OOREXXUNIT.CLS");
}

/// The tests `run` started, in order.
pub fn started(run: &Run) -> Vec<String> {
    started_lines(&run.stderr)
}

/// The tests [`mark_starts_in_file`]'s `marker` names, in order; none where
/// it does not exist.
pub fn started_in_file(marker: &Path) -> Vec<String> {
    started_lines(&fs::read(marker).unwrap_or_default())
}

fn started_lines(bytes: &[u8]) -> Vec<String> {
    String::from_utf8_lossy(bytes)
        .lines()
        .filter_map(|line| line.strip_prefix("started "))
        .map(str::to_ascii_uppercase)
        .collect()
}

/// The tests a run's `-V 2` detail names as failing or in error, in
/// order.
pub fn failing(run: &Run) -> Vec<String> {
    let text = String::from_utf8_lossy(&run.stdout);
    let mut names = Vec::new();
    let mut detail = false;
    for line in text.lines() {
        if line.starts_with("[failure] ") || line.starts_with("[error] ") {
            detail = true;
        } else if let Some(name) = line.trim_start().strip_prefix("Test:").filter(|_| detail) {
            names.push(name.trim().to_string());
            detail = false;
        }
    }
    names
}

/// The first `rexx-exec: ` line of a run's stderr: this crate's refusal.
pub fn refusal(run: &Run) -> Option<String> {
    String::from_utf8_lossy(&run.stderr)
        .lines()
        .find(|line| line.starts_with("rexx-exec: "))
        .map(str::to_string)
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

/// The group's tests, sorted, distinct and upper case: the instance methods
/// named `test...` of its test classes ([`test_classes`]), which the
/// framework's `getTestMethods` runs (`framework/OOREXXUNIT.CLS:1761`): for a
/// group whose whole-group listing run outlasts the oracle's deadline. A
/// `::CLASS` directive split across lines is not followed.
pub fn source_test_names(dir: &str, group: &str) -> Vec<String> {
    source_test_names_in(&groups_root(), dir, group)
}

/// [`source_test_names`] over the groups below `root`.
pub fn source_test_names_in(root: &Path, dir: &str, group: &str) -> Vec<String> {
    let classes = source_classes(&read_lossy(
        &root.join(dir).join(format!("{group}.testGroup")),
    ));
    let tests = test_classes(&classes);
    let names: BTreeSet<String> = classes
        .iter()
        .filter(|class| tests.contains(&class.name))
        .flat_map(|class| class.methods.iter())
        .filter(|name| name.starts_with("TEST"))
        .cloned()
        .collect();
    names.into_iter().collect()
}

/// One `::CLASS` of a group file.
pub struct SourceClass {
    /// Lower case, unquoted.
    pub name: String,
    /// The classes its directive names after `SUBCLASS`, `MIXINCLASS` or
    /// `INHERIT`, lower case and unquoted.
    pub parents: Vec<String>,
    /// Whether its directive names `ooTestCase`.
    pub direct: bool,
    /// Its instance methods' names, upper case and unquoted, from its
    /// `::METHOD`, `::CONSTANT` and `::ATTRIBUTE` directives.
    pub methods: Vec<String>,
    /// Its methods' lines, lower case, comments removed.
    pub body: String,
}

/// `line` without its comments, `depth` being how many block comments are
/// open where it starts and, after, where it ends. Block comments nest and a
/// `--` ends the line, outside quotes.
fn without_comments(line: &str, depth: &mut usize) -> String {
    let chars: Vec<char> = line.chars().collect();
    let mut code = String::new();
    let mut quote: Option<char> = None;
    let mut at = 0;
    while at < chars.len() {
        let (this, next) = (chars[at], chars.get(at + 1).copied());
        if *depth > 0 {
            match (this, next) {
                ('/', Some('*')) => {
                    *depth += 1;
                    at += 2;
                }
                ('*', Some('/')) => {
                    *depth -= 1;
                    code.push(' ');
                    at += 2;
                }
                _ => at += 1,
            }
            continue;
        }
        match (quote, this, next) {
            (Some(open), _, _) => {
                if this == open {
                    quote = None;
                }
                code.push(this);
            }
            (None, '\'' | '"', _) => {
                quote = Some(this);
                code.push(this);
            }
            (None, '/', Some('*')) => {
                *depth = 1;
                at += 1;
            }
            (None, '-', Some('-')) => break,
            (None, _, _) => code.push(this),
        }
        at += 1;
    }
    code
}

/// The name a directive's `rest` starts with, upper case and unquoted, and
/// the words after it, lower case.
fn directive_name(rest: &str) -> (String, Vec<String>) {
    let rest = rest.trim_start();
    let (name, after) = match rest.chars().next() {
        Some(quote @ ('\'' | '"')) => {
            let close = rest[1..].find(quote).map_or(rest.len(), |at| at + 1);
            (&rest[1..close], rest.get(close + 1..).unwrap_or(""))
        }
        _ => rest.split_once(char::is_whitespace).unwrap_or((rest, "")),
    };
    (
        name.to_ascii_uppercase(),
        after
            .split_whitespace()
            .map(str::to_ascii_lowercase)
            .collect(),
    )
}

/// The `::CLASS`es of `text`, comments and resource bodies skipped.
pub fn source_classes(text: &str) -> Vec<SourceClass> {
    const KEYWORDS: &[&str] = &[
        "subclass",
        "mixinclass",
        "inherit",
        "public",
        "private",
        "metaclass",
        "abstract",
    ];
    let unquoted = |word: &str| word.trim_matches(['\'', '"']).to_ascii_lowercase();
    let mut classes: Vec<SourceClass> = Vec::new();
    let mut resource_end: Option<String> = None;
    let mut depth = 0;
    for raw in text.lines() {
        if let Some(marker) = &resource_end {
            if raw.starts_with(marker.as_str()) {
                resource_end = None;
            }
            continue;
        }
        let code = without_comments(raw, &mut depth);
        let line = code.trim_start();
        let lower = line.to_ascii_lowercase();
        if lower.starts_with("::resource") {
            resource_end = Some(resource_marker(line));
        } else if let Some(rest) = lower.strip_prefix("::class") {
            let words: Vec<&str> = rest.split_whitespace().collect();
            let mut parents = Vec::new();
            let mut after: Option<&str> = None;
            // `INHERIT` takes every word after it (the oracle's
            // `parser/DirectiveParser.cpp:461-469`).
            for word in words.iter().skip(1) {
                if after == Some("inherit") {
                    parents.push(unquoted(word));
                } else if KEYWORDS.contains(word) {
                    after = Some(word);
                } else if let Some(keyword) = after {
                    if matches!(keyword, "subclass" | "mixinclass") {
                        parents.push(unquoted(word));
                    }
                    after = None;
                }
            }
            classes.push(SourceClass {
                name: words
                    .first()
                    .map_or_else(String::new, |word| unquoted(word)),
                parents,
                direct: lower.contains("ootestcase"),
                methods: Vec::new(),
                body: String::new(),
            });
        } else if let Some(class) = classes.last_mut() {
            let directive = |keyword: &str| {
                lower
                    .starts_with(keyword)
                    .then(|| directive_name(&line[keyword.len()..]))
            };
            if let Some((name, after)) = directive("::method") {
                if !after.iter().any(|word| word == "class") {
                    class.methods.push(name);
                }
            } else if let Some((name, _)) = directive("::constant") {
                class.methods.push(name);
            } else if let Some((name, after)) = directive("::attribute") {
                if !after.iter().any(|word| word == "class") {
                    let get = !after.iter().any(|word| word == "set");
                    let set = !after.iter().any(|word| word == "get");
                    if get {
                        class.methods.push(name.clone());
                    }
                    if set {
                        class.methods.push(format!("{name}="));
                    }
                }
            } else if !lower.starts_with("::") {
                class.body.push_str(&lower);
                class.body.push('\n');
            }
        }
    }
    classes
}

/// The names of `classes` that are test classes: those naming `ooTestCase`,
/// and those naming a test class of `classes` as their superclass or a
/// class they inherit.
pub fn test_classes(classes: &[SourceClass]) -> BTreeSet<String> {
    let mut tests: BTreeSet<String> = classes
        .iter()
        .filter(|class| class.direct)
        .map(|class| class.name.clone())
        .collect();
    loop {
        let more: Vec<String> = classes
            .iter()
            .filter(|class| {
                !tests.contains(&class.name)
                    && class.parents.iter().any(|parent| tests.contains(parent))
            })
            .map(|class| class.name.clone())
            .collect();
        if more.is_empty() {
            return tests;
        }
        tests.extend(more);
    }
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
                let ours = run_crate(run, &args, mode.clone());
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
