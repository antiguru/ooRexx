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

//! The ooTest suite: every group `testOORexx.rex` finds under `ootest/ooRexx`,
//! each run whole through its single-group form, once on the oracle and on this
//! crate until a run ends normally (spec 2026-10-10 section 2). The suite run is
//! gate-only, under `REXX_SUITE_GATE`; `REXX_SUITE_GROUPS` restricts it to the
//! comma-separated `dir/group` names it lists.
//!
//! Each side runs from its own copy: [`group_runner::fresh_copy_from`], the
//! group file's namesakes elsewhere in the copy deleted, the tests
//! [`group_runner::reaching_rxapi_in`] finds renamed out, and each test's start
//! marked in [`marker`]'s file. A run of this crate that a refusal, a crash, a
//! timeout or the memory cap ends has its last started test renamed out, and
//! runs again.

#[path = "support/group_runner.rs"]
mod group_runner;
mod support;
mod watchdog;

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use group_runner::{
    Ended, ProcessRun, VERBOSITY, crate_environment_with_path, failing, fresh_copy_from,
    groups_root, interpreter_bin, mark_starts_in_file, reaching_rxapi_in, read_bytes, refusal,
    renamed, run_process, set_environment, skip, started_in_file, worktree,
};
use support::oracle::{ORACLE_MEMORY_LIMIT_KIB, oracle_root};

/// Runs the suite when set to anything but empty or `0`.
pub const GATE_ENV: &str = "REXX_SUITE_GATE";

/// The `dir/group` names, comma-separated, the gate runs instead of every
/// group.
pub const GROUPS_ENV: &str = "REXX_SUITE_GROUPS";

/// Where the table is written; the oracle-difference report goes beside it.
pub const TABLE_ENV: &str = "REXX_SUITE_TABLE";

/// The crate's least bound in seconds, where set.
pub const FLOOR_ENV: &str = "REXX_SUITE_CRATE_FLOOR";

/// The bound on one oracle run.
pub const ORACLE_DEADLINE: Duration = Duration::from_secs(300);

/// A crate run's bound is the oracle's time times this, and at least the
/// floor.
pub const CRATE_FACTOR: u32 = 4;
pub const CRATE_FLOOR: Duration = Duration::from_secs(60);

/// The most runs of one group on this crate.
pub const MAX_RUNS: usize = 60;

/// How many groups run at once.
pub const GROUPS_AT_ONCE: usize = 8;

/// The crate side's `PATH` after its own `bin`.
pub const SYSTEM_PATH: &[&str] = &["/usr/local/bin", "/usr/bin", "/bin"];

/// The crate's memory cap under `memcap`.
pub const CRATE_MEMORY: &str = "2G";

/// The crate's address-space limit in KiB where `memcap` is not found.
pub const FALLBACK_LIMIT_KIB: u64 = 4_194_304;

/// The file the ootest checkout's svn revision is recorded in, relative to
/// `rust/`.
pub const REVISION_FILE: &str = "corpus/suite-revision.txt";

/// What bounded a side's memory.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CapKind {
    /// `memcap` at [`CRATE_MEMORY`].
    Memcap,
    /// `ulimit -v` at [`FALLBACK_LIMIT_KIB`], `memcap` not being found.
    UlimitFallback,
    /// The oracle's `ulimit -v` at [`ORACLE_MEMORY_LIMIT_KIB`].
    OracleUlimit,
}

impl CapKind {
    pub fn label(self) -> &'static str {
        match self {
            CapKind::Memcap => "memcap 2G",
            CapKind::UlimitFallback => "ulimit -v 4194304 (no memcap)",
            CapKind::OracleUlimit => "ulimit -v 1048576",
        }
    }
}

/// One group on one side. A test is its upper-cased method name.
pub struct GroupRun {
    /// `dir/group`, as `base/bif/C2X`.
    pub group: String,
    pub expected: BTreeSet<String>,
    /// Each test any run started.
    pub started: BTreeSet<String>,
    /// The tests started in the run that ended normally, by its `-V 2`
    /// detail.
    pub passed: BTreeSet<String>,
    pub failed: BTreeSet<String>,
    /// Each test a run ended in, with the refusal or how the run ended.
    pub blocked: BTreeMap<String, String>,
    /// How the run ended, where it ended before any test started.
    pub blocked_at_load: Option<String>,
    pub iterations: usize,
    /// Whether the last run ended with the framework's summary.
    pub ended_normally: bool,
    pub cap: CapKind,
    /// The framework's own `Tests ran:` figure of the run that ended normally.
    pub tests_ran: Option<String>,
    /// The runs' time, summed.
    pub took: Duration,
}

impl GroupRun {
    fn new(group: String, cap: CapKind) -> GroupRun {
        GroupRun {
            group,
            expected: BTreeSet::new(),
            started: BTreeSet::new(),
            passed: BTreeSet::new(),
            failed: BTreeSet::new(),
            blocked: BTreeMap::new(),
            blocked_at_load: None,
            iterations: 0,
            ended_normally: false,
            cap,
            tests_ran: None,
            took: Duration::ZERO,
        }
    }

    /// What this side found of `test`.
    fn verdict(&self, test: &str) -> String {
        if let Some(reason) = &self.blocked_at_load {
            format!("blocked at load: {reason}")
        } else if let Some(reason) = self.blocked.get(test) {
            format!("blocked: {reason}")
        } else if self.passed.contains(test) {
            "passed".to_string()
        } else if self.failed.contains(test) {
            "failed".to_string()
        } else if self.started.contains(test) {
            "started".to_string()
        } else {
            "not started".to_string()
        }
    }
}

/// Where the groups come from and how the crate side is bounded.
pub struct CrateSettings {
    /// The directory the groups' `dir`s are below.
    pub root: PathBuf,
    /// The name `memcap` is looked up by on this process's `PATH`.
    pub memcap: String,
    pub floor: Duration,
    pub max_runs: usize,
}

impl CrateSettings {
    pub fn from_env() -> CrateSettings {
        CrateSettings {
            root: groups_root(),
            memcap: "memcap".to_string(),
            floor: floor_from(std::env::var(FLOOR_ENV).ok().as_deref()),
            max_runs: MAX_RUNS,
        }
    }
}

/// The crate floor `value` names in seconds, else [`CRATE_FLOOR`].
pub fn floor_from(value: Option<&str>) -> Duration {
    value.map_or(CRATE_FLOOR, |value| {
        Duration::from_secs(
            value
                .parse()
                .unwrap_or_else(|_| panic!("{FLOOR_ENV} is not a count of seconds: {value:?}")),
        )
    })
}

/// `dir/group`, or `group` for a group at the top.
pub fn group_path(dir: &str, group: &str) -> String {
    if dir.is_empty() {
        group.to_string()
    } else {
        format!("{dir}/{group}")
    }
}

/// Each `*.testGroup` below `ootest/ooRexx`, as `(dir, group)`, in the order
/// of their paths, as `ooTestFinder::findFiles` lists them (`ooTest.frm`).
pub fn discover_groups() -> Vec<(String, String)> {
    discover_in(&groups_root())
}

fn discover_in(root: &Path) -> Vec<(String, String)> {
    fn walk(root: &Path, at: &Path, found: &mut Vec<(String, String)>) {
        for entry in
            fs::read_dir(at).unwrap_or_else(|e| panic!("cannot read {}: {e}", at.display()))
        {
            let path = entry.expect("a directory entry").path();
            if path.is_dir() {
                walk(root, &path, found);
            } else if let Some(group) = path
                .file_name()
                .and_then(|name| name.to_str())
                .and_then(|name| name.strip_suffix(".testGroup"))
            {
                let dir = path
                    .parent()
                    .and_then(|parent| parent.strip_prefix(root).ok())
                    .expect("a group below the root");
                found.push((dir.to_string_lossy().into_owned(), group.to_string()));
            }
        }
    }
    let mut found = Vec::new();
    walk(root, root, &mut found);
    found.sort_by_key(|(dir, group)| format!("{}.testGroup", group_path(dir, group)));
    found
}

/// `value` as a `PATH` string.
pub fn path_of(value: Option<std::ffi::OsString>) -> String {
    value.map_or_else(String::new, |value| value.to_string_lossy().into_owned())
}

/// Where `name` resolves on `path`, as `command -v` resolves it: the first
/// directory holding an executable file of that name, or `name` itself where
/// it holds a `/`.
pub fn resolve(name: &str, path: &str) -> Option<PathBuf> {
    use std::os::unix::fs::PermissionsExt as _;
    let executable = |candidate: &Path| {
        fs::metadata(candidate)
            .is_ok_and(|meta| meta.is_file() && meta.permissions().mode() & 0o111 != 0)
    };
    if name.contains('/') {
        return Some(PathBuf::from(name)).filter(|candidate| executable(candidate));
    }
    path.split(':')
        .filter(|dir| !dir.is_empty())
        .map(|dir| Path::new(dir).join(name))
        .find(|candidate| executable(candidate))
}

/// Whether the crate side's `path` resolves `rexx` inside `bin` and `rexxc`
/// nowhere.
pub fn check_path(path: &str, bin: &Path) -> Result<(), String> {
    let rexx = resolve("rexx", path);
    if rexx.as_deref() != Some(bin.join("rexx").as_path()) {
        return Err(format!(
            "rexx resolves to {rexx:?} on {path}, not inside {}",
            bin.display()
        ));
    }
    match resolve("rexxc", path) {
        Some(rexxc) => Err(format!("rexxc resolves to {} on {path}", rexxc.display())),
        None => Ok(()),
    }
}

/// The environment a crate run of the copy `run` gets: `PATH` is
/// [`interpreter_bin`] and [`SYSTEM_PATH`].
pub fn crate_side_environment(run: &Path) -> Vec<(Vec<u8>, Vec<u8>)> {
    crate_environment_with_path(run, SYSTEM_PATH.iter().map(PathBuf::from).collect())
}

/// The ootest checkout's svn revision, asserted equal to [`REVISION_FILE`]'s.
pub fn suite_revision() -> String {
    let output = Command::new("svn")
        .args(["info", "--show-item", "revision"])
        .arg(worktree().join("ootest"))
        .output()
        .expect("cannot run svn");
    assert!(output.status.success(), "svn info: {output:?}");
    let revision = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let recorded = fs::read_to_string(worktree().join("rust").join(REVISION_FILE))
        .expect("the recorded suite revision");
    assert_eq!(
        revision,
        recorded.trim(),
        "the ootest checkout is not at {REVISION_FILE}'s revision"
    );
    revision
}

/// A directory of its own for one run of a group on one side.
fn work_dir(group: &str, side: &str) -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "suite-{}-{}-{side}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed),
        group.replace('/', "__")
    ))
}

/// Lays out the copy `run` of `group` with `left_out` renamed out, each
/// test's start marked in [`marker`]'s file, which is emptied, and answers its
/// group file.
fn prepare(
    root: &Path,
    run: &Path,
    dir: &str,
    group: &str,
    left_out: &BTreeSet<String>,
) -> PathBuf {
    fresh_copy_from(run, root, dir);
    let group_file = run
        .join("ooRexx")
        .join(dir)
        .join(format!("{group}.testGroup"));
    remove_namesakes(&run.join("ooRexx"), &group_file);
    skip(&group_file, group, left_out);
    let marker = marker(run);
    if marker.exists() {
        fs::remove_file(&marker)
            .unwrap_or_else(|e| panic!("cannot remove {}: {e}", marker.display()));
    }
    mark_starts_in_file(run, &marker);
    group_file
}

/// The file the copy `run` names each test in as it starts, beside the copy
/// rather than in it: a test reading its directory sees only the copy.
/// A file rather than stderr because this crate hands its stderr to the
/// descriptor only at a read, an idle or the end, so a run killed at its bound
/// would name no test.
pub fn marker(run: &Path) -> PathBuf {
    run.parent().expect("a parent").join("started.txt")
}

/// Deletes each file below `at` named as `group_file` is, other than it.
fn remove_namesakes(at: &Path, group_file: &Path) {
    for entry in fs::read_dir(at).unwrap_or_else(|e| panic!("cannot read {}: {e}", at.display())) {
        let path = entry.expect("a directory entry").path();
        if path.is_dir() {
            remove_namesakes(&path, group_file);
        } else if path.file_name() == group_file.file_name() && path != group_file {
            fs::remove_file(&path)
                .unwrap_or_else(|e| panic!("cannot remove {}: {e}", path.display()));
        }
    }
}

/// The driver's words for `group_file`.
fn driver_args(run: &Path, group_file: &Path) -> Vec<String> {
    vec![
        run.join("testOORexx.rex").to_string_lossy().into_owned(),
        "-f".to_string(),
        group_file.to_string_lossy().into_owned(),
        "-U".to_string(),
        "-V".to_string(),
        VERBOSITY.to_string(),
    ]
}

/// The renamed-out set a side starts from: the tests reaching rxapi and the
/// caller's.
fn left_out(
    root: &Path,
    dir: &str,
    group: &str,
    renamed_out: &BTreeSet<String>,
) -> BTreeSet<String> {
    let mut left_out = reaching_rxapi_in(root, dir, &[group]);
    left_out.extend(
        renamed_out
            .iter()
            .filter(|name| name.starts_with(&format!("{group}.")))
            .cloned(),
    );
    left_out
}

/// The framework's `Tests ran:` figure in `stdout`.
fn tests_ran(stdout: &[u8]) -> Option<String> {
    String::from_utf8_lossy(stdout)
        .lines()
        .find_map(|line| line.strip_prefix("Tests ran:"))
        .map(|rest| rest.trim().to_string())
}

/// How a run ended early, or `None` where it ended with the framework's
/// summary.
fn end_reason(process: &ProcessRun, deadline: Duration) -> Option<String> {
    if let Some(line) = refusal(&process.run) {
        return Some(line);
    }
    let stderr = String::from_utf8_lossy(&process.run.stderr);
    let line_with = |needle: &str| {
        stderr
            .lines()
            .find(|line| line.contains(needle))
            .map(str::to_string)
    };
    if let Some(line) =
        line_with("memcap: OOM-killed").or_else(|| line_with("memory allocation of"))
    {
        return Some(format!("out of memory: {line}"));
    }
    if let Some(line) = line_with("panicked at") {
        return Some(format!("crash: {line}"));
    }
    match process.ended {
        Ended::Killed => return Some(format!("timeout: killed after {:?}", process.took)),
        Ended::Exited(124 | 137) => {
            return Some(format!("timeout after {} s", deadline.as_secs()));
        }
        Ended::Exited(code) if code >= 128 => return Some(format!("crash: rc {code}")),
        Ended::Signaled(signal) => return Some(format!("crash: signal {signal}")),
        Ended::Exited(_) => {}
    }
    if tests_ran(&process.run.stdout).is_none() {
        let last = stderr.lines().last().unwrap_or("");
        return Some(format!(
            "no summary: rc {:?}, stderr ends {last:?}",
            process.run.status
        ));
    }
    None
}

/// Records one run's tests in `result`, and answers how it ended early.
fn record(
    result: &mut GroupRun,
    process: &ProcessRun,
    tests: Vec<String>,
    deadline: Duration,
) -> Option<String> {
    result.iterations += 1;
    result.took += process.took;
    result.started.extend(tests.iter().cloned());
    let reason = end_reason(process, deadline);
    match &reason {
        None => {
            result.ended_normally = true;
            result.tests_ran = tests_ran(&process.run.stdout);
            let failing: BTreeSet<String> = failing(&process.run)
                .iter()
                .map(|name| name.to_ascii_uppercase())
                .collect();
            for test in tests {
                if failing.contains(&test) {
                    result.failed.insert(test);
                } else {
                    result.passed.insert(test);
                }
            }
        }
        Some(reason) => match tests.last() {
            None => result.blocked_at_load = Some(reason.clone()),
            Some(last) => {
                result.blocked.insert(last.clone(), reason.clone());
            }
        },
    }
    reason
}

/// `group` of `dir` on the oracle, once, with the tests reaching rxapi and
/// `renamed_out` (`GROUP.TEST` names) renamed out.
pub fn run_group_on_oracle(dir: &str, group: &str, renamed_out: &BTreeSet<String>) -> GroupRun {
    run_group_on_oracle_from(&groups_root(), dir, group, renamed_out)
}

/// [`run_group_on_oracle`] with the groups below `root`.
pub fn run_group_on_oracle_from(
    root: &Path,
    dir: &str,
    group: &str,
    renamed_out: &BTreeSet<String>,
) -> GroupRun {
    let binary = oracle_root().join("bin/rexx");
    assert!(binary.is_file(), "no oracle at {}", binary.display());
    let work = work_dir(&group_path(dir, group), "oracle");
    let run = work.join("run");
    let group_file = prepare(
        root,
        &run,
        dir,
        group,
        &left_out(root, dir, group, renamed_out),
    );
    let path = std::env::join_paths(
        std::iter::once(oracle_root().join("bin")).chain(
            std::env::var_os("PATH")
                .iter()
                .flat_map(std::env::split_paths),
        ),
    )
    .expect("a PATH");
    let mut command = Command::new("sh");
    command
        .arg("-c")
        .arg(format!(
            "ulimit -v {ORACLE_MEMORY_LIMIT_KIB} && exec \"$0\" \"$@\""
        ))
        .arg(&binary)
        .args(driver_args(&run, &group_file))
        .current_dir(&run)
        .env("LD_LIBRARY_PATH", oracle_root().join("lib"))
        .env("PATH", path);
    let process = run_process(command, None, ORACLE_DEADLINE);
    let mut result = GroupRun::new(group_path(dir, group), CapKind::OracleUlimit);
    record(
        &mut result,
        &process,
        started_in_file(&marker(&run)),
        ORACLE_DEADLINE,
    );
    remove_work(&work);
    result
}

/// `group` of `dir` on this crate under [`CRATE_FLOOR`] or [`FLOOR_ENV`]'s
/// bound; [`run_group`] scales the bound by the oracle's time.
pub fn run_group_on_crate(dir: &str, group: &str, renamed_out: &BTreeSet<String>) -> GroupRun {
    let settings = CrateSettings::from_env();
    run_group_on_crate_within(&settings, dir, group, renamed_out, settings.floor)
}

/// The crate's bound for a group the oracle ran as `oracle`.
pub fn crate_deadline(settings: &CrateSettings, oracle: &GroupRun) -> Duration {
    settings.floor.max(oracle.took * CRATE_FACTOR)
}

/// `group` of `dir` (below `settings.root`) on this crate, each run under
/// `deadline`, the tests reaching rxapi and `renamed_out` renamed out. A run
/// that ends early has its last started test renamed out and runs again, up
/// to `settings.max_runs` runs; one that ends before any test starts, or in a
/// test the group file does not define, is the last.
pub fn run_group_on_crate_within(
    settings: &CrateSettings,
    dir: &str,
    group: &str,
    renamed_out: &BTreeSet<String>,
    deadline: Duration,
) -> GroupRun {
    let memcap = resolve(&settings.memcap, &path_of(std::env::var_os("PATH")));
    let cap = if memcap.is_some() {
        CapKind::Memcap
    } else {
        CapKind::UlimitFallback
    };
    let source = read_bytes(&settings.root.join(dir).join(format!("{group}.testGroup")));
    let mut left_out = left_out(&settings.root, dir, group, renamed_out);
    let mut result = GroupRun::new(group_path(dir, group), cap);
    let work = work_dir(&result.group, "crate");
    let run = work.join("run");
    while result.iterations < settings.max_runs {
        let (process, tests) = crate_run_once(
            &settings.root,
            &run,
            dir,
            group,
            &left_out,
            memcap.as_deref(),
            deadline,
        );
        let last = tests.last().cloned();
        let Some(_) = record(&mut result, &process, tests, deadline) else {
            break;
        };
        let Some(last) = last else {
            break;
        };
        let member = format!("{group}.{last}");
        if left_out.contains(&member)
            || !renamed(&source, group, &BTreeSet::from([member.clone()]))
                .1
                .contains(&member)
        {
            break;
        }
        left_out.insert(member);
    }
    remove_work(&work);
    result
}

/// One run of the copy `run` of `group` on this crate under `deadline`, with
/// `left_out` renamed out, under `memcap` where given and the
/// [`FALLBACK_LIMIT_KIB`] address-space limit otherwise, and the tests it
/// started.
pub fn crate_run_once(
    root: &Path,
    run: &Path,
    dir: &str,
    group: &str,
    left_out: &BTreeSet<String>,
    memcap: Option<&Path>,
    deadline: Duration,
) -> (ProcessRun, Vec<String>) {
    let group_file = prepare(root, run, dir, group, left_out);
    let environment = crate_side_environment(run);
    let bin = interpreter_bin(run);
    let path = environment
        .iter()
        .find(|(name, _)| name == b"PATH")
        .map(|(_, value)| String::from_utf8_lossy(value).into_owned())
        .unwrap_or_default();
    if let Err(problem) = check_path(&path, &bin) {
        panic!("{problem}");
    }
    let mut words = vec![
        "timeout".to_string(),
        "-s".to_string(),
        "KILL".to_string(),
        deadline.as_secs().max(1).to_string(),
        bin.join("rexx").to_string_lossy().into_owned(),
    ];
    words.extend(driver_args(run, &group_file));
    let mut command = match memcap {
        Some(memcap) => {
            let mut command = Command::new(memcap);
            command.arg(CRATE_MEMORY);
            command
        }
        None => {
            let mut command = Command::new("sh");
            command.arg("-c").arg(format!(
                "ulimit -v {FALLBACK_LIMIT_KIB} && exec \"$0\" \"$@\""
            ));
            command
        }
    };
    command.args(&words).current_dir(run);
    set_environment(&mut command, environment);
    let process = run_process(command, None, deadline + Duration::from_secs(30));
    let tests = started_in_file(&marker(run));
    (process, tests)
}

fn remove_work(work: &Path) {
    if work.exists() {
        fs::remove_dir_all(work)
            .unwrap_or_else(|e| panic!("cannot remove {}: {e}", work.display()));
    }
}

/// `group` of `dir` on the oracle, then on this crate under
/// [`crate_deadline`].
pub fn run_group(
    settings: &CrateSettings,
    dir: &str,
    group: &str,
    renamed_out: &BTreeSet<String>,
) -> (GroupRun, GroupRun) {
    let oracle = run_group_on_oracle_from(&settings.root, dir, group, renamed_out);
    let deadline = crate_deadline(settings, &oracle);
    let ours = run_group_on_crate_within(settings, dir, group, renamed_out, deadline);
    (oracle, ours)
}

/// One table row.
fn table_row(side: &str, run: &GroupRun) -> String {
    let blocked: Vec<String> = run
        .blocked
        .iter()
        .map(|(test, reason)| format!("{test}={reason}"))
        .collect();
    [
        run.group.clone(),
        side.to_string(),
        run.expected.len().to_string(),
        run.started.len().to_string(),
        run.passed.len().to_string(),
        run.failed.iter().cloned().collect::<Vec<_>>().join(" "),
        blocked.join(" | "),
        run.blocked_at_load
            .clone()
            .unwrap_or_else(|| "-".to_string()),
        run.iterations.to_string(),
        if run.ended_normally { "yes" } else { "no" }.to_string(),
        run.cap.label().to_string(),
        run.tests_ran.clone().unwrap_or_else(|| "-".to_string()),
    ]
    .join("\t")
}

/// The table of `results`, headed by the suite's `revision`.
pub fn table(revision: &str, results: &[(GroupRun, GroupRun)]) -> String {
    let mut table = format!(
        "# ootest revision {revision}\n\
         group\tside\texpected\tstarted\tpassed\tfailed\tblocked\tblocked at load\titerations\tended normally\tcap\ttests ran\n"
    );
    for (oracle, ours) in results {
        for (side, run) in [("oracle", oracle), ("crate", ours)] {
            table.push_str(&table_row(side, run));
            table.push('\n');
        }
    }
    table
}

/// Each test whose verdict differs between the sides, with both.
pub fn oracle_differences(revision: &str, results: &[(GroupRun, GroupRun)]) -> String {
    let mut report = format!("# ootest revision {revision}\ngroup\ttest\toracle\tcrate\n");
    for (oracle, ours) in results {
        let tests: BTreeSet<&String> = [oracle, ours]
            .iter()
            .flat_map(|run| run.started.iter().chain(run.blocked.keys()))
            .collect();
        for test in tests {
            let (theirs, mine) = (oracle.verdict(test), ours.verdict(test));
            if theirs != mine {
                let _ = writeln!(report, "{}\t{test}\t{theirs}\t{mine}", oracle.group);
            }
        }
    }
    report
}

/// Where the table goes.
fn table_path() -> PathBuf {
    std::env::var_os(TABLE_ENV).map_or_else(
        || {
            PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
                .parent()
                .expect("the target directory")
                .join("suite/table.tsv")
        },
        PathBuf::from,
    )
}

/// The groups [`GROUPS_ENV`] names, each of which must be discovered, else
/// every group.
fn selected(groups: Vec<(String, String)>) -> Vec<(String, String)> {
    let Ok(names) = std::env::var(GROUPS_ENV) else {
        return groups;
    };
    let names: BTreeSet<&str> = names
        .split(',')
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .collect();
    let chosen: Vec<(String, String)> = groups
        .into_iter()
        .filter(|(dir, group)| names.contains(group_path(dir, group).as_str()))
        .collect();
    let found: BTreeSet<String> = chosen
        .iter()
        .map(|(dir, group)| group_path(dir, group))
        .collect();
    let missing: Vec<&&str> = names
        .iter()
        .filter(|name| !found.contains(**name))
        .collect();
    assert!(
        missing.is_empty(),
        "{GROUPS_ENV} names no group {missing:?}"
    );
    chosen
}

fn suite_gate() -> bool {
    std::env::var(GATE_ENV).is_ok_and(|value| !value.is_empty() && value != "0")
}

#[test]
fn every_group_on_both_sides() {
    if !suite_gate() {
        return;
    }
    let revision = suite_revision();
    let groups = selected(discover_groups());
    let settings = CrateSettings::from_env();
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(GROUPS_AT_ONCE)
        .thread_name(|at| format!("suite-group-{at}"))
        .build()
        .expect("cannot build the group pool");
    let results: Vec<(GroupRun, GroupRun)> = pool.install(|| {
        use rayon::prelude::*;
        groups
            .par_iter()
            .map(|(dir, group)| run_group(&settings, dir, group, &BTreeSet::new()))
            .collect()
    });
    let path = table_path();
    fs::create_dir_all(path.parent().expect("a directory")).expect("the table's directory");
    fs::write(&path, table(&revision, &results)).expect("cannot write the table");
    let differences = path.with_file_name("oracle-differences.tsv");
    fs::write(&differences, oracle_differences(&revision, &results))
        .expect("cannot write the oracle-difference report");
    eprintln!(
        "suite: {} groups, table {}, differences {}",
        results.len(),
        path.display(),
        differences.display()
    );
}

/// The harness's own behaviour, on the groups under `tests/suite/fixtures/`
/// with the real framework copied around them.
mod fixtures {
    use std::collections::{BTreeMap, BTreeSet};
    use std::fs;
    use std::path::PathBuf;
    use std::time::Duration;

    use super::group_runner::{interpreter_bin, started_in_file};
    use super::{
        CapKind, CrateSettings, GroupRun, check_path, crate_run_once, crate_side_environment,
        discover_groups, floor_from, marker, path_of, resolve, run_group_on_crate_within,
    };

    fn root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/suite/fixtures")
    }

    fn settings() -> CrateSettings {
        CrateSettings {
            root: root(),
            memcap: "memcap".to_string(),
            floor: Duration::from_secs(60),
            max_runs: 60,
        }
    }

    fn crate_run(settings: &CrateSettings, group: &str) -> GroupRun {
        run_group_on_crate_within(settings, "", group, &BTreeSet::new(), settings.floor)
    }

    fn names(names: &[&str]) -> BTreeSet<String> {
        names.iter().map(|name| (*name).to_string()).collect()
    }

    const MACRO_REFUSAL: &str =
        "rexx-exec: routine \"SYSQUERYREXXMACRO\" is not implemented (Phase 10)";

    #[test]
    fn a_refusal_ends_the_run_and_the_next_run_goes_past_it() {
        let run = crate_run(&settings(), "RefusedCall");
        assert_eq!(run.group, "RefusedCall");
        assert_eq!(run.iterations, 2);
        assert_eq!(
            run.blocked,
            BTreeMap::from([("TEST_B".to_string(), MACRO_REFUSAL.to_string())])
        );
        assert_eq!(
            run.started,
            names(&["TEST_A", "TEST_B", "TEST_C", "TEST_D"])
        );
        assert_eq!(run.passed, names(&["TEST_A", "TEST_C"]));
        assert_eq!(run.failed, names(&["TEST_D"]));
        assert_eq!(run.blocked_at_load, None);
        assert!(run.ended_normally);
        assert!(run.expected.is_empty());
    }

    #[test]
    fn a_refusal_at_load_is_blocked_at_load_after_one_run() {
        let run = crate_run(&settings(), "RefusedProlog");
        assert_eq!(run.iterations, 1);
        assert_eq!(run.blocked_at_load.as_deref(), Some(MACRO_REFUSAL));
        assert!(run.started.is_empty());
        assert!(run.blocked.is_empty());
        assert!(run.passed.is_empty() && run.failed.is_empty());
        assert!(!run.ended_normally);
    }

    #[test]
    fn a_test_that_hangs_is_renamed_out_and_the_next_run_proceeds() {
        let settings = CrateSettings {
            floor: Duration::from_secs(5),
            ..settings()
        };
        let run = crate_run(&settings, "Hangs");
        assert_eq!(run.iterations, 2);
        assert_eq!(run.blocked.keys().collect::<Vec<_>>(), ["TEST_B"]);
        assert!(
            run.blocked["TEST_B"].starts_with("timeout"),
            "{:?}",
            run.blocked
        );
        assert_eq!(run.passed, names(&["TEST_A", "TEST_C"]));
        assert!(run.ended_normally);
    }

    #[test]
    fn the_marker_file_names_the_test_a_kill_at_the_bound_ended_in() {
        let run = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("suite-kill-{}", std::process::id()))
            .join("run");
        let memcap = resolve("memcap", &path_of(std::env::var_os("PATH")));
        let (process, tests) = crate_run_once(
            &root(),
            &run,
            "",
            "Hangs",
            &BTreeSet::new(),
            memcap.as_deref(),
            Duration::from_secs(5),
        );
        assert_eq!(process.run.status, Some(137));
        assert_eq!(tests, ["TEST_A", "TEST_B"]);
        assert_eq!(started_in_file(&marker(&run)), tests);
        fs::remove_dir_all(run.parent().expect("a parent")).expect("cannot remove the run");
    }

    #[test]
    fn the_run_cap_stops_a_group_that_hangs_on_every_test() {
        let settings = CrateSettings {
            floor: Duration::from_secs(5),
            max_runs: 2,
            ..settings()
        };
        let run = crate_run(&settings, "HangsEverywhere");
        assert_eq!(run.iterations, 2);
        assert_eq!(run.started, names(&["TEST_A", "TEST_B"]));
        assert_eq!(
            run.blocked.keys().cloned().collect::<BTreeSet<_>>(),
            names(&["TEST_A", "TEST_B"])
        );
        assert!(run.passed.is_empty());
        assert!(!run.ended_normally);
    }

    #[test]
    fn without_memcap_the_crate_run_is_under_the_ulimit_fallback() {
        let fallback = CrateSettings {
            memcap: "memcap-is-not-on-this-path".to_string(),
            ..settings()
        };
        let run = crate_run(&fallback, "AddressSpace");
        assert_eq!(run.cap, CapKind::UlimitFallback);
        assert_eq!(run.passed, names(&["TEST_LIMIT"]));
        if resolve("memcap", &path_of(std::env::var_os("PATH"))).is_some() {
            let run = crate_run(&settings(), "AddressSpace");
            assert_eq!(run.cap, CapKind::Memcap);
            assert_eq!(run.failed, names(&["TEST_LIMIT"]));
        }
    }

    #[test]
    fn the_crate_side_finds_its_own_rexx_and_no_rexxc() {
        let run = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("suite-path-{}", std::process::id()))
            .join("run");
        fs::create_dir_all(&run).expect("the run directory");
        let bin = interpreter_bin(&run);
        let path = crate_side_environment(&run)
            .into_iter()
            .find(|(name, _)| name == b"PATH")
            .map(|(_, value)| String::from_utf8(value).expect("a UTF-8 PATH"))
            .expect("a PATH");
        assert_eq!(
            path,
            format!("{}:/usr/local/bin:/usr/bin:/bin", bin.display())
        );
        assert_eq!(check_path(&path, &bin), Ok(()));
        assert_eq!(resolve("rexx", &path), Some(bin.join("rexx")));
        assert_eq!(
            fs::canonicalize(bin.join("rexx")).expect("the link"),
            fs::canonicalize(env!("CARGO_BIN_EXE_rexx-run")).expect("rexx-run")
        );
        assert_eq!(resolve("rexxc", &path), None);
        assert!(check_path("/usr/bin:/bin", &bin).is_err());
        fs::copy(env!("CARGO_BIN_EXE_rexx-run"), bin.join("rexxc")).expect("a rexxc");
        assert!(check_path(&path, &bin).is_err());
        fs::remove_dir_all(run.parent().expect("a parent")).expect("cannot remove the run");
    }

    #[test]
    fn the_crate_floor_is_read_from_its_variable() {
        assert_eq!(floor_from(None), Duration::from_secs(60));
        assert_eq!(floor_from(Some("5")), Duration::from_secs(5));
    }

    #[test]
    fn discovery_finds_groups_at_every_depth_in_path_order() {
        let groups = discover_groups();
        for wanted in [
            ("", "SimpleTests"),
            ("base/bif", "C2X"),
            ("samples", "samples"),
        ] {
            assert!(
                groups.contains(&(wanted.0.to_string(), wanted.1.to_string())),
                "{wanted:?}"
            );
        }
        let paths: Vec<String> = groups
            .iter()
            .map(|(dir, group)| format!("{}.testGroup", super::group_path(dir, group)))
            .collect();
        let mut sorted = paths.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(paths, sorted);
    }

    #[test]
    fn the_suite_revision_is_the_recorded_one() {
        super::suite_revision();
    }
}
