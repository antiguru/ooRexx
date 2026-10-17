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
#[path = "support/suite_rows.rs"]
mod suite_rows;
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
    renamed, run_process, set_environment, skip, source_test_names_in, started_in_file, worktree,
};
use suite_rows::{Context, Owner, Row, check_rows, group_path, read_rows, table_path};
use support::oracle::{ORACLE_MEMORY_LIMIT_KIB, oracle_root};

/// Runs the suite when set to anything but empty or `0`.
pub const GATE_ENV: &str = "REXX_SUITE_GATE";

/// The `dir/group` names, comma-separated, the gate runs instead of every
/// group.
pub const GROUPS_ENV: &str = "REXX_SUITE_GROUPS";

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

/// Files outside a group's directory that its tests open by a path relative
/// to the group's, by `dir/group`, each relative to `ootest/ooRexx`: the
/// copy holds them where the checkout does.
pub const NEIGHBOURS: &[(&str, &[&str])] = &[
    (
        "base/bif/STREAM",
        &["base/class/Array.testGroup", "API/oo/CONVERSION.testGroup"],
    ),
    ("base/class/Stream", &["base/bif/STREAM.testGroup"]),
];

/// The groups that run one at a time, after the others and beside none: each
/// binds a fixed port, so two at once fail with `EADDRINUSE` on the oracle
/// (`rust/corpus/suite-oracle-runs/samples-*.txt`), and each passes alone.
pub const ALONE: &[&str] = &[
    "samples/scclient",
    "samples/scserver",
    "samples/sfclient",
    "samples/sfserver",
];

/// `groups` split into those that run [`GROUPS_AT_ONCE`] at a time and those
/// [`ALONE`] names, each in `groups`' order.
pub fn schedule(groups: Vec<(String, String)>) -> (Vec<(String, String)>, Vec<(String, String)>) {
    groups
        .into_iter()
        .partition(|(dir, group)| !ALONE.contains(&group_path(dir, group).as_str()))
}

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
    /// How the first run ended, where it ended before any test started.
    pub blocked_at_load: Option<String>,
    /// How a later run ended, where it started no test and ended early.
    pub ended_before_a_test: Option<String>,
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
            ended_before_a_test: None,
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

/// The tests of `group` of `dir`, as [`group_runner::source_test_names`]
/// enumerates them: the denominator before the row files' rows.
pub fn expected_set(dir: &str, group: &str) -> BTreeSet<String> {
    expected_set_in(&groups_root(), dir, group)
}

/// [`expected_set`] over the groups below `root`.
pub fn expected_set_in(root: &Path, dir: &str, group: &str) -> BTreeSet<String> {
    source_test_names_in(root, dir, group).into_iter().collect()
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

/// Whether the crate side's `path` resolves `rexx` and `rexxc` inside `bin`,
/// and `bin` holds no ELF file but copies of this crate's `rexx` and `rexxc`.
pub fn check_path(path: &str, bin: &Path) -> Result<(), String> {
    use std::os::unix::fs::MetadataExt as _;
    let identity = |path: &Path| fs::metadata(path).ok().map(|meta| (meta.dev(), meta.ino()));
    let ours = [
        Path::new(env!("CARGO_BIN_EXE_rexx")),
        Path::new(env!("CARGO_BIN_EXE_rexxc")),
    ];
    for entry in fs::read_dir(bin).map_err(|e| format!("cannot read {}: {e}", bin.display()))? {
        let file = entry.map_err(|e| e.to_string())?.path();
        if identity(&file).is_some()
            && ours
                .iter()
                .any(|binary| identity(&file) == identity(binary))
        {
            continue;
        }
        let bytes = fs::read(&file).unwrap_or_default();
        if bytes.starts_with(b"\x7fELF")
            && !ours
                .iter()
                .any(|binary| bytes == fs::read(binary).unwrap_or_default())
        {
            return Err(format!(
                "{} is a binary other than this crate's",
                file.display()
            ));
        }
    }
    for name in ["rexx", "rexxc"] {
        let found = resolve(name, path);
        if found.as_deref() != Some(bin.join(name).as_path()) {
            return Err(format!(
                "{name} resolves to {found:?} on {path}, not inside {}",
                bin.display()
            ));
        }
    }
    Ok(())
}

/// Copies the oracle `bin`'s Rexx class files (`*.cls`) into the crate
/// side's `bin`, so a program searching `PATH` for one finds it on both sides.
fn lay_class_files(bin: &Path) {
    let from = oracle_root().join("bin");
    for entry in
        fs::read_dir(&from).unwrap_or_else(|e| panic!("cannot read {}: {e}", from.display()))
    {
        let file = entry.expect("a directory entry").path();
        if file.extension().is_some_and(|extension| extension == "cls") {
            let to = bin.join(file.file_name().expect("a file name"));
            fs::copy(&file, &to).unwrap_or_else(|e| panic!("cannot copy {}: {e}", file.display()));
        }
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
    let path = group_path(dir, group);
    for (_, files) in NEIGHBOURS.iter().filter(|(listed, _)| *listed == path) {
        for file in *files {
            let to = run.join("ooRexx").join(file);
            fs::create_dir_all(to.parent().expect("a directory"))
                .unwrap_or_else(|e| panic!("cannot create {}: {e}", to.display()));
            fs::copy(root.join(file), &to).unwrap_or_else(|e| panic!("cannot copy {file}: {e}"));
        }
    }
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

/// How a run ended early, or `None` where it exited below 128, other than
/// `timeout`'s 124, with the framework's summary. Stderr's text only labels a
/// run that ended early: a command the run starts writes to the same stderr.
fn end_reason(process: &ProcessRun, deadline: Duration) -> Option<String> {
    let summary = tests_ran(&process.run.stdout).is_some();
    let early = match process.ended {
        Ended::Exited(code) => code == 124 || code >= 128 || !summary,
        Ended::Signaled(_) | Ended::Killed => true,
    };
    if !early {
        return None;
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
    Some(match process.ended {
        Ended::Killed => format!("timeout: killed after {:?}", process.took),
        Ended::Exited(124 | 137) => format!("timeout after {} s", deadline.as_secs()),
        Ended::Signaled(signal) => format!("crash: signal {signal}"),
        Ended::Exited(code) => {
            if let Some(line) = refusal(&process.run) {
                line
            } else if let Some(line) = line_with("panicked at") {
                format!("crash: {line}")
            } else if code >= 128 {
                format!("crash: rc {code}")
            } else {
                let last = stderr.lines().last().unwrap_or("");
                format!("no summary: rc {code}, stderr ends {last:?}")
            }
        }
    })
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
            None if result.iterations == 1 => result.blocked_at_load = Some(reason.clone()),
            None => result.ended_before_a_test = Some(reason.clone()),
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
    lay_class_files(&bin);
    let path = environment
        .iter()
        .find(|(name, _)| name == b"PATH")
        .map(|(_, value)| String::from_utf8_lossy(value).into_owned())
        .unwrap_or_default();
    if let Err(problem) = check_path(&path, &bin) {
        panic!("{problem}");
    }
    // `--foreground` keeps `timeout` in the run's process group, so the kill
    // `run_process` sends the group once the run ends reaches what a test left
    // running.
    let mut words = vec![
        "timeout".to_string(),
        "--foreground".to_string(),
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
        run.ended_before_a_test
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
         group\tside\texpected\tstarted\tpassed\tfailed\tblocked\tblocked at load\tended before a test\titerations\tended normally\tcap\ttests ran\n"
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

/// The per-test record's groups line for this run.
fn groups_line() -> String {
    std::env::var(GROUPS_ENV).map_or_else(
        |_| suite_rows::ALL_GROUPS_LINE.to_string(),
        |names| format!("# groups {names}"),
    )
}

fn suite_gate() -> bool {
    std::env::var(GATE_ENV).is_ok_and(|value| !value.is_empty() && value != "0")
}

/// The rows of `group` (`dir/group`).
fn rows_of<'a>(rows: &'a [Row], group: &str) -> Vec<&'a Row> {
    rows.iter().filter(|row| row.group == group).collect()
}

/// The `GROUP.TEST` names of `rows` the harness renames out on both sides.
fn renamed_out_by_rows(group: &str, rows: &[&Row]) -> BTreeSet<String> {
    rows.iter()
        .filter(|row| row.renamed_out())
        .map(|row| format!("{group}.{}", row.test))
        .collect()
}

/// D1 on this crate's run `ours` of a group whose rows are `rows`, its
/// `expected` already the enumeration less the rows, and the oracle's run
/// `oracle` against the rows: each reason the group is not green.
pub fn judge(oracle: &GroupRun, ours: &GroupRun, rows: &[&Row]) -> Vec<String> {
    let rowed: BTreeSet<&String> = rows.iter().map(|row| &row.test).collect();
    let mut problems = Vec::new();
    let list = |tests: Vec<&String>| {
        tests
            .iter()
            .map(|test| test.as_str())
            .collect::<Vec<_>>()
            .join(" ")
    };
    if !ours.ended_normally {
        let how = ours
            .blocked_at_load
            .as_ref()
            .map(|reason| format!("blocked at load: {reason}"))
            .or_else(|| {
                ours.ended_before_a_test
                    .as_ref()
                    .map(|reason| format!("a run started no test: {reason}"))
            })
            .unwrap_or_else(|| format!("after {} runs", ours.iterations));
        problems.push(format!("did not end normally, {how}"));
    }
    let not_started: Vec<&String> = ours.expected.difference(&ours.started).collect();
    if !not_started.is_empty() {
        problems.push(format!("not started: {}", list(not_started)));
    }
    let not_passed: Vec<&String> = ours
        .expected
        .intersection(&ours.started)
        .filter(|test| !ours.passed.contains(*test))
        .collect();
    if !not_passed.is_empty() {
        problems.push(format!("not passed: {}", list(not_passed)));
    }
    let outside: Vec<&String> = ours
        .started
        .iter()
        .filter(|test| !ours.expected.contains(*test) && !rowed.contains(test))
        .collect();
    if !outside.is_empty() {
        problems.push(format!(
            "started outside the expected set and the rows: {}",
            list(outside)
        ));
    }
    for row in rows {
        if let Some(recorded) = row.refusal() {
            match ours.blocked.get(&row.test) {
                Some(captured) if captured == recorded => {}
                Some(captured) => problems.push(format!(
                    "{}'s refusal is not the one captured: {captured}",
                    row.at()
                )),
                None => problems.push(format!("{}'s refusal not captured", row.at())),
            }
        }
        if row.owner == Owner::NotRun && oracle.started.contains(&row.test) {
            problems.push(format!("{}'s test started on the oracle", row.at()));
        }
    }
    problems
}

/// The denominator check on the oracle's run: the tests it started that are
/// neither expected nor rowed, and the expected tests it did not start.
pub fn oracle_denominator(oracle: &GroupRun, rows: &[&Row]) -> String {
    let rowed: BTreeSet<&String> = rows.iter().map(|row| &row.test).collect();
    let extra: Vec<&str> = oracle
        .started
        .iter()
        .filter(|test| !oracle.expected.contains(*test) && !rowed.contains(test))
        .map(String::as_str)
        .collect();
    let missing: Vec<&str> = oracle
        .expected
        .difference(&oracle.started)
        .map(String::as_str)
        .collect();
    if extra.is_empty() && missing.is_empty() {
        "equal".to_string()
    } else {
        format!(
            "started, not expected: [{}]; expected, not started: [{}]",
            extra.join(" "),
            missing.join(" ")
        )
    }
}

/// Every test either side started or blocked, or the group expects or rows,
/// with its row's owner and each side's verdict.
pub fn per_test(
    revision: &str,
    groups_line: &str,
    results: &[(GroupRun, GroupRun)],
    rows: &[Row],
) -> String {
    let mut report =
        format!("# ootest revision {revision}\n{groups_line}\ngroup\ttest\trow\toracle\tcrate\n");
    for (oracle, ours) in results {
        let group_rows = rows_of(rows, &oracle.group);
        let tests: BTreeSet<&String> = [oracle, ours]
            .iter()
            .flat_map(|run| {
                run.started
                    .iter()
                    .chain(run.blocked.keys())
                    .chain(run.expected.iter())
            })
            .chain(group_rows.iter().map(|row| &row.test))
            .collect();
        for test in tests {
            let owner = group_rows
                .iter()
                .find(|row| &row.test == test)
                .map_or_else(|| "-".to_string(), |row| format!("{:?}", row.owner));
            let _ = writeln!(
                report,
                "{}\t{test}\t{owner}\t{}\t{}",
                oracle.group,
                oracle.verdict(test),
                ours.verdict(test)
            );
        }
    }
    report
}

/// D1 over the groups run, and spec section 2's denominator check: every
/// group `testOORexx.rex` finds, or those [`GROUPS_ENV`] names, on both
/// sides, each judged against the row files. Writes the table, the
/// oracle-difference report, the per-test record and `verdicts.tsv` beside
/// [`table_path`] before asserting every group green.
#[test]
fn l3_core_suite() {
    if !suite_gate() {
        return;
    }
    let revision = suite_revision();
    let (rows, malformed) = read_rows();
    assert!(malformed.is_empty(), "the row files: {malformed:#?}");
    let groups = selected(discover_groups());
    let settings = CrateSettings::from_env();
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(GROUPS_AT_ONCE)
        .thread_name(|at| format!("suite-group-{at}"))
        .build()
        .expect("cannot build the group pool");
    let judged = |(dir, group): &(String, String)| {
        let path = group_path(dir, group);
        let group_rows = rows_of(&rows, &path);
        let (mut oracle, mut ours) = run_group(
            &settings,
            dir,
            group,
            &renamed_out_by_rows(group, &group_rows),
        );
        let mut expected = expected_set(dir, group);
        for row in &group_rows {
            expected.remove(&row.test);
        }
        oracle.expected = expected.clone();
        ours.expected = expected;
        (oracle, ours)
    };
    let (pooled, alone) = schedule(groups);
    let mut results: Vec<(GroupRun, GroupRun)> = pool.install(|| {
        use rayon::prelude::*;
        pooled.par_iter().map(judged).collect()
    });
    results.extend(alone.iter().map(judged));
    let path = table_path();
    fs::create_dir_all(path.parent().expect("a directory")).expect("the table's directory");
    fs::write(&path, table(&revision, &results)).expect("cannot write the table");
    let beside = |name: &str| path.with_file_name(name);
    fs::write(
        beside("oracle-differences.tsv"),
        oracle_differences(&revision, &results),
    )
    .expect("cannot write the oracle-difference report");
    fs::write(
        beside(suite_rows::TESTS_FILE),
        per_test(&revision, &groups_line(), &results, &rows),
    )
    .expect("cannot write the per-test record");
    let mut verdicts =
        format!("# ootest revision {revision}\ngroup\tverdict\tcrate\toracle denominator\n");
    let mut red = Vec::new();
    for (oracle, ours) in &results {
        let group_rows = rows_of(&rows, &oracle.group);
        let problems = judge(oracle, ours, &group_rows);
        let verdict = if problems.is_empty() { "green" } else { "red" };
        if !problems.is_empty() {
            red.push(oracle.group.clone());
        }
        let _ = writeln!(
            verdicts,
            "{}\t{verdict}\t{}\t{}",
            oracle.group,
            problems.join(" | "),
            oracle_denominator(oracle, &group_rows)
        );
    }
    let verdict_path = beside("verdicts.tsv");
    fs::write(&verdict_path, verdicts).expect("cannot write the verdicts");
    eprintln!(
        "suite: {} groups, {} red, table {}, verdicts {}",
        results.len(),
        red.len(),
        path.display(),
        verdict_path.display()
    );
    assert!(
        red.is_empty(),
        "{} of {} groups are not green; each reason is in {}",
        red.len(),
        results.len(),
        verdict_path.display()
    );
}

/// The row files' owner checks over every group (spec section 2), and each
/// row [`suite_rows::derived_rows`] derives from the group files present.
#[test]
fn the_row_files_hold_only_checked_rows() {
    let (rows, mut problems) = read_rows();
    let context = Context::new(&groups_root(), &discover_groups());
    problems.extend(check_rows(&rows, &context));
    let present: BTreeSet<String> = rows
        .iter()
        .map(|row| {
            let owner = match row.owner {
                Owner::Phase10(reason) => format!("Phase 10 {}", reason.name()),
                Owner::Deviation(number) => format!("Deviation {number}"),
                other => format!("{other:?}"),
            };
            format!("{}\t{}\t{owner}\t{}", row.group, row.test, row.reason)
        })
        .collect();
    let missing: Vec<String> = suite_rows::derived_rows(&context)
        .into_iter()
        .filter(|line| !present.contains(line))
        .collect();
    if !missing.is_empty() {
        problems.push(format!("derived rows missing:\n{}", missing.join("\n")));
    }
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

/// The 9b gate-group file (spec 2026-10-10 section 4), and the derivation it
/// is committed from.
mod gate_groups_9b {
    use std::collections::BTreeSet;
    use std::fs;
    use std::path::Path;

    use super::suite_rows::{Owner, Row};
    use super::{expected_set_in, group_path, groups_root, read_rows, worktree};

    /// The file, relative to the worktree.
    const FILE: &str = "rust/corpus/suite-9b-groups.txt";

    /// The RexxUtil routines spec section 4 lists, plus `SysFileTree` and
    /// `SysFileXXX`.
    const ROUTINES: &[&str] = &[
        "SysCls",
        "SysCreatePipe",
        "SysDropFuncs",
        "SysDumpVariables",
        "SysFileCopy",
        "SysFileMove",
        "SysFileSearch",
        "SysFork",
        "SysFormatMessage",
        "SysGetErrorText",
        "SysGetFileDateTime",
        "SysGetKey",
        "SysGetMessage",
        "SysGetMessageX",
        "SysIsFileDirectory",
        "SysIsFileLink",
        "SysLoadFuncs",
        "SysOpenEventSem",
        "SysOpenMutexSem",
        "SysQueryProcess",
        "SysSearchPath",
        "SysSetFileDateTime",
        "SysSetPriority",
        "SysStemCopy",
        "SysStemDelete",
        "SysStemInsert",
        "SysStemSort",
        "SysTempFileName",
        "SysUtilVersion",
        "SysWait",
        "SysFileTree",
        "SysFileXXX",
    ];

    /// The command line the file's header records.
    fn command() -> String {
        format!("grep -rlawiE '{}' ootest/ooRexx", ROUTINES.join("|"))
    }

    /// Whether `text` holds `name` as a whole word, ignoring ASCII case: the
    /// match `grep -w -i` makes, where a word is letters, digits and `_`.
    fn has_word(text: &[u8], name: &str) -> bool {
        let text = text.to_ascii_lowercase();
        let name = name.to_ascii_lowercase().into_bytes();
        let word = |byte: u8| byte.is_ascii_alphanumeric() || byte == b'_';
        text.windows(name.len()).enumerate().any(|(at, window)| {
            window == name.as_slice()
                && (at == 0 || !word(text[at - 1]))
                && text.get(at + name.len()).is_none_or(|&byte| !word(byte))
        })
    }

    /// Every regular file below `at`, dotfiles and binary files included,
    /// symbolic links not followed, as `grep -r` walks.
    fn files_below(at: &Path, found: &mut Vec<std::path::PathBuf>) {
        for entry in
            fs::read_dir(at).unwrap_or_else(|e| panic!("cannot read {}: {e}", at.display()))
        {
            let entry = entry.expect("a directory entry");
            let kind = entry.file_type().expect("a file type");
            if kind.is_dir() {
                files_below(&entry.path(), found);
            } else if kind.is_file() {
                found.push(entry.path());
            }
        }
    }

    /// The `dir/group` of each `.testGroup` file that names a routine.
    fn grep_hits(root: &Path) -> BTreeSet<(String, String)> {
        let mut files = Vec::new();
        files_below(root, &mut files);
        let mut hits = BTreeSet::new();
        for path in files {
            let Some(group) = path
                .file_name()
                .and_then(|name| name.to_str())
                .and_then(|name| name.strip_suffix(".testGroup"))
            else {
                continue;
            };
            let bytes = fs::read(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
            if ROUTINES.iter().any(|name| has_word(&bytes, name)) {
                let dir = path
                    .parent()
                    .and_then(|parent| parent.strip_prefix(root).ok())
                    .expect("a group below the root");
                hits.insert((dir.to_string_lossy().into_owned(), group.to_string()));
            }
        }
        hits
    }

    /// Whether every test of the group has a Phase 10 row.
    fn phase_10_group(root: &Path, dir: &str, group: &str, rows: &[Row]) -> bool {
        let path = group_path(dir, group);
        let tests = expected_set_in(root, dir, group);
        let owned: BTreeSet<&str> = rows
            .iter()
            .filter(|row| row.group == path && matches!(row.owner, Owner::Phase10(_)))
            .map(|row| row.test.as_str())
            .collect();
        !tests.is_empty() && tests.iter().all(|test| owned.contains(test.as_str()))
    }

    /// The groups the derivation gives, as `dir/group` in path order.
    fn derived(root: &Path, rows: &[Row]) -> Vec<String> {
        let mut groups: Vec<(String, String)> = grep_hits(root)
            .into_iter()
            .filter(|(dir, group)| !phase_10_group(root, dir, group, rows))
            .collect();
        groups.sort_by_key(|(dir, group)| format!("{}.testGroup", group_path(dir, group)));
        groups
            .iter()
            .map(|(dir, group)| group_path(dir, group))
            .collect()
    }

    /// The names a file lists and the command its header records.
    fn parse(text: &str) -> (Vec<String>, bool) {
        let names = text
            .lines()
            .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
            .map(str::to_string)
            .collect();
        let recorded = format!("#   {}", command());
        (names, text.lines().any(|line| line == recorded))
    }

    #[test]
    fn the_9b_group_file_is_what_its_derivation_gives() {
        let text = fs::read_to_string(worktree().join(FILE))
            .unwrap_or_else(|e| panic!("cannot read {FILE}: {e}"));
        let (rows, problems) = read_rows();
        assert!(problems.is_empty(), "{}", problems.join("\n"));
        let (listed, recorded) = parse(&text);
        assert!(recorded, "{FILE}'s header does not record `{}`", command());
        assert_eq!(listed, derived(&groups_root(), &rows));
    }

    #[test]
    fn a_word_match_is_grep_dash_w_dash_i() {
        assert!(has_word(b"call SYSSTEMSORT stem.", "SysStemSort"));
        assert!(has_word(b"SysStemSort", "SysStemSort"));
        assert!(!has_word(b"MySysStemSort", "SysStemSort"));
        assert!(!has_word(b"SysStemSorter", "SysStemSort"));
        assert!(!has_word(b"SysStemSort_x", "SysStemSort"));
        assert!(has_word(b"xSysStemSortx SysStemSort(", "SysStemSort"));
        assert!(!has_word(b"SysGetMessageX", "SysGetMessage"));
    }
}

/// D1's judgement and the renaming rows ask for, on runs built by hand.
mod judging {
    use std::collections::BTreeSet;

    use super::suite_rows::{Owner, Row};
    use super::{CapKind, GroupRun, judge, oracle_denominator, renamed_out_by_rows};

    fn set(names: &[&str]) -> BTreeSet<String> {
        names.iter().map(|name| (*name).to_string()).collect()
    }

    /// A run that ended normally, expecting `expected`, starting
    /// `started` and passing `passed`.
    fn run(expected: &[&str], started: &[&str], passed: &[&str]) -> GroupRun {
        let mut run = GroupRun::new("base/x/G".to_string(), CapKind::Memcap);
        run.expected = set(expected);
        run.started = set(started);
        run.passed = set(passed);
        run.ended_normally = true;
        run.iterations = 1;
        run
    }

    fn row(test: &str, owner: Owner, reason: &str) -> Row {
        Row {
            file: "f".to_string(),
            line: 1,
            group: "base/x/G".to_string(),
            test: test.to_string(),
            owner,
            reason: reason.to_string(),
        }
    }

    #[test]
    fn a_group_is_green_only_when_every_expected_test_ran_and_passed_and_it_ended() {
        let oracle = run(&["TEST_A", "TEST_B"], &["TEST_A", "TEST_B", "TEST_R"], &[]);
        let flaky = row("TEST_R", Owner::Flaky, "common.txt: G.testGroup/TEST_R");
        let green = run(
            &["TEST_A", "TEST_B"],
            &["TEST_A", "TEST_B", "TEST_R"],
            &["TEST_A", "TEST_B"],
        );
        assert_eq!(judge(&oracle, &green, &[&flaky]), Vec::<String>::new());

        let unstarted = run(&["TEST_A", "TEST_B"], &["TEST_A"], &["TEST_A"]);
        assert_eq!(judge(&oracle, &unstarted, &[]), ["not started: TEST_B"]);

        let failing = run(&["TEST_A", "TEST_B"], &["TEST_A", "TEST_B"], &["TEST_A"]);
        assert_eq!(judge(&oracle, &failing, &[]), ["not passed: TEST_B"]);

        let unrowed = run(&["TEST_A"], &["TEST_A", "TEST_R"], &["TEST_A", "TEST_R"]);
        assert_eq!(
            judge(&oracle, &unrowed, &[]),
            ["started outside the expected set and the rows: TEST_R"]
        );

        let mut cut = run(&["TEST_A"], &["TEST_A"], &["TEST_A"]);
        cut.ended_normally = false;
        assert_eq!(judge(&oracle, &cut, &[]).len(), 1);
    }

    #[test]
    fn a_row_whose_refusal_or_claim_the_run_contradicts_reddens_its_group() {
        let refused = row(
            "TEST_R",
            Owner::Phase10(super::suite_rows::Deferral::Rxapi),
            "refusal: rexx-exec: x (Phase 10 Rxapi)",
        );
        let oracle = run(&["TEST_A"], &["TEST_A"], &["TEST_A"]);
        let mut ours = run(&["TEST_A"], &["TEST_A", "TEST_R"], &["TEST_A"]);
        ours.blocked.insert(
            "TEST_R".to_string(),
            "rexx-exec: x (Phase 10 Rxapi)".to_string(),
        );
        assert_eq!(judge(&oracle, &ours, &[&refused]), Vec::<String>::new());
        ours.blocked
            .insert("TEST_R".to_string(), "rexx-exec: y (Phase 9)".to_string());
        assert_eq!(judge(&oracle, &ours, &[&refused]).len(), 1);
        let failed = run(&["TEST_A"], &["TEST_A", "TEST_R"], &["TEST_A"]);
        assert_eq!(
            judge(&oracle, &failed, &[&refused]).len(),
            1,
            "a refusal row whose test ran without refusing"
        );

        let not_run = row("TEST_N", Owner::NotRun, "restrictOS line 1");
        let ours = run(&["TEST_A"], &["TEST_A"], &["TEST_A"]);
        assert_eq!(judge(&oracle, &ours, &[&not_run]), Vec::<String>::new());
        let started = run(&["TEST_A"], &["TEST_A", "TEST_N"], &["TEST_A"]);
        assert_eq!(judge(&started, &ours, &[&not_run]).len(), 1);
    }

    #[test]
    fn the_oracle_denominator_reports_both_directions() {
        let rowed = row("TEST_R", Owner::Flaky, "common.txt: G.testGroup/TEST_R");
        let equal = run(&["TEST_A"], &["TEST_A", "TEST_R"], &[]);
        assert_eq!(oracle_denominator(&equal, &[&rowed]), "equal");
        let both = run(&["TEST_A"], &["TEST_X"], &[]);
        assert_eq!(
            oracle_denominator(&both, &[]),
            "started, not expected: [TEST_X]; expected, not started: [TEST_A]"
        );
    }

    #[test]
    fn only_an_environment_row_whose_test_hangs_is_renamed_out() {
        let hangs = row(
            "TEST_H",
            Owner::Environment,
            "hangs; record: r; ruled: Moritz 2000-01-01",
        );
        let fails = row(
            "TEST_F",
            Owner::Environment,
            "fails; record: r; ruled: Moritz 2000-01-01",
        );
        let flaky = row("TEST_L", Owner::Flaky, "hangs");
        assert_eq!(
            renamed_out_by_rows("G", &[&hangs, &fails, &flaky]),
            set(&["G.TEST_H"])
        );
    }
}

#[test]
fn the_port_binding_groups_run_alone_and_the_rest_together() {
    let groups = discover_groups();
    let (pooled, alone) = schedule(groups.clone());
    let paths = |list: &[(String, String)]| -> Vec<String> {
        list.iter()
            .map(|(dir, group)| group_path(dir, group))
            .collect()
    };
    assert_eq!(paths(&alone), ALONE);
    assert_eq!(pooled.len() + alone.len(), groups.len());
    assert!(
        paths(&pooled)
            .iter()
            .all(|path| !ALONE.contains(&path.as_str()))
    );
}

#[test]
fn only_a_record_of_every_group_at_the_recorded_revision_is_a_full_run() {
    use suite_rows::{ALL_GROUPS_LINE, full_run_record};
    let header = |revision: &str, groups: &str| {
        format!("# ootest revision {revision}\n{groups}\ngroup\ttest\trow\toracle\tcrate\n")
    };
    assert_eq!(
        full_run_record(&header("13178", ALL_GROUPS_LINE), "13178"),
        Ok(())
    );
    assert!(full_run_record(&header("13178", "# groups base/bif/C2X"), "13178").is_err());
    assert!(full_run_record(&header("13177", ALL_GROUPS_LINE), "13178").is_err());
    assert!(full_run_record("# ootest revision 13178\ngroup\ttest\n", "13178").is_err());
}

/// Each owner's check, on rows each of which passes or fails it.
mod row_checks {
    use std::collections::BTreeSet;

    use super::group_runner::{groups_root, worktree};
    use super::suite_rows::{
        Context, EXCLUSIONS, LICENCES, NOT_RUN, ROW_10_GROUPS, check_row, check_rows, parse_rows,
    };

    fn context(groups: &[&str]) -> Context {
        let groups: Vec<(String, String)> = groups
            .iter()
            .map(|path| match path.rsplit_once('/') {
                Some((dir, group)) => (dir.to_string(), group.to_string()),
                None => (String::new(), (*path).to_string()),
            })
            .collect();
        Context::new(&groups_root(), &groups)
    }

    /// What is wrong with the one row `line` in `file`.
    fn problems(file: &str, line: &str, context: &Context) -> Vec<String> {
        let (revision, rows, mut problems) =
            parse_rows(file, &format!("# ootest revision 13178\n{line}\n"));
        assert_eq!(revision.as_deref(), Some("13178"));
        for row in &rows {
            if let Err(problem) = check_row(row, context) {
                problems.push(problem);
            }
        }
        problems
    }

    const GROUPS: &[&str] = &[
        "base/bif/RXQUEUE",
        "base/bif/CHAROUT",
        "base/class/RexxQueue",
        "extensions/json/json",
        "API/oo/RexxStart",
        "utilities/rxapi",
        "base/rexxutil/platform/unix/tilde",
        "regressions/bug2061_newline_misalignment",
        "doc/rexxref/chapter7/Section4",
        "samples/sfserver",
    ];

    const PASSING: &[(&str, &str)] = &[
        (
            EXCLUSIONS,
            "base/bif/RXQUEUE\tTEST_RXQUEUE_NOARG\tPhase 10 Rxapi\treaching_rxapi",
        ),
        (
            EXCLUSIONS,
            "extensions/json/json\tTEST_INVALID_EMPTY\tPhase 10 Extension\tgroup",
        ),
        (
            EXCLUSIONS,
            "base/class/RexxQueue\tTEST_CREATE\tPhase 10 Rxapi\trefusal: rexx-exec: a queue is not implemented (Phase 10 Rxapi)",
        ),
        (
            LICENCES,
            "base/bif/CHAROUT\tTEST_STDOUT_START\tFlaky\tcommon.txt: CHAROUT.testGroup/TEST_STDOUT_START",
        ),
        (
            LICENCES,
            "base/bif/CHAROUT\tTEST_STDOUT_START\tDeviation 4\tidentity",
        ),
        (
            LICENCES,
            "samples/sfserver\tTEST_04_SFSERVER.REX\tEnvironment\trecord: rust/corpus/suite-oracle-runs/samples-sfserver.txt; ruled: Moritz 2000-01-01",
        ),
        (
            NOT_RUN,
            "regressions/bug2061_newline_misalignment\tTEST_STDOUT_NEWLINE\tNotRun\trestrictOS line 41",
        ),
        (
            NOT_RUN,
            "doc/rexxref/chapter7/Section4\tTEST_BEEP_EXAMPLE_7_4_6\tNotRun\ttest type DOC_EXAMPLE_NOISE_TEST",
        ),
    ];

    const FAILING: &[(&str, &str)] = &[
        (
            EXCLUSIONS,
            "base/bif/RXQUEUE\tTEST_RXQUEUE_NOARG\tPhase 10 Rxapi",
        ),
        (
            EXCLUSIONS,
            "base/bif/RXQUEUE\tTEST_RXQUEUE_NOARG\tPhase 10 Queue\treaching_rxapi",
        ),
        (
            LICENCES,
            "base/bif/RXQUEUE\tTEST_RXQUEUE_NOARG\tPhase 10 Rxapi\treaching_rxapi",
        ),
        (
            EXCLUSIONS,
            "base/bif/RXQUEUE\tTEST_RXQUEUE_NOARG\tPhase 10 Extension\treaching_rxapi",
        ),
        (
            EXCLUSIONS,
            "base/bif/CHAROUT\tTEST_STDOUT_START\tPhase 10 Rxapi\treaching_rxapi",
        ),
        (
            EXCLUSIONS,
            "base/bif/CHAROUT\tTEST_NO_SUCH_TEST\tPhase 10 Rxapi\trefusal: rexx-exec: x (Phase 10 Rxapi)",
        ),
        (
            EXCLUSIONS,
            "base/bif/NO_SUCH_GROUP\tTEST_A\tPhase 10 Rxapi\trefusal: rexx-exec: x (Phase 10 Rxapi)",
        ),
        (
            EXCLUSIONS,
            "base/bif/CHAROUT\tTEST_STDOUT_START\tPhase 10 Extension\tgroup",
        ),
        (
            EXCLUSIONS,
            "API/oo/RexxStart\tTEST_A\tPhase 10 Extension\tgroup",
        ),
        (
            EXCLUSIONS,
            "utilities/rxapi\tTEST_A\tPhase 10 Extension\tgroup",
        ),
        (
            EXCLUSIONS,
            "base/class/RexxQueue\tTEST_CREATE\tPhase 10 Rxapi\trefusal: rexx-exec: Object~copy is not implemented (Phase 9)",
        ),
        (
            EXCLUSIONS,
            "base/class/RexxQueue\tTEST_CREATE\tPhase 10 Rxapi\trefusal: rexx-exec: a queue is not implemented (Phase 10)",
        ),
        (
            EXCLUSIONS,
            "base/class/RexxQueue\tTEST_CREATE\tPhase 10 Rxapi\trefusal: rexx-exec: a queue is not implemented (Phase 10 Extension)",
        ),
        (
            EXCLUSIONS,
            "base/class/RexxQueue\tTEST_CREATE\tPhase 10 Rxapi\trefusal: rexx-exec: a queue is not implemented (Phase 10 Rxapi2)",
        ),
        (
            EXCLUSIONS,
            "base/class/RexxQueue\tTEST_CREATE\tPhase 10 Rxapi\tit needs the daemon",
        ),
        (
            LICENCES,
            "base/bif/CHAROUT\tTEST_STDOUT_START\tDeviation 5\twithdrawn",
        ),
        (
            LICENCES,
            "base/bif/CHAROUT\tTEST_STDOUT_START\tDeviation 999\tnone",
        ),
        (
            LICENCES,
            "base/bif/CHAROUT\tTEST_STDOUT_START\tDeviation x1\tnone",
        ),
        (
            LICENCES,
            "base/bif/CHAROUT\tTEST_STDOUT_START\tEnvironment\truled: Moritz 2026-10-11",
        ),
        (
            LICENCES,
            "base/bif/CHAROUT\tTEST_STDOUT_START\tEnvironment\trecord: rust/corpus/suite-oracle-runs/none.txt; ruled: Moritz 2026-10-11",
        ),
        (
            LICENCES,
            "base/bif/CHAROUT\tTEST_STDOUT_START\tEnvironment\trecord: rust/corpus/suite-revision.txt; ruled: Moritz 2026-10-11",
        ),
        (
            LICENCES,
            "samples/sfserver\tTEST_04_SFSERVER.REX\tEnvironment\trecord: rust/corpus/suite-oracle-runs/samples-scclient.txt; ruled: Moritz 2000-01-01",
        ),
        (
            LICENCES,
            "samples/sfserver\tTEST_04_SFSERVER.REX\tEnvironment\trecord: rust/corpus/suite-oracle-runs/samples-sfserver.txt; ruled: Moritz",
        ),
        (
            LICENCES,
            "base/bif/CHAROUT\tTEST_STDOUT_START\tFlaky\tcommon.txt: LINEOUT.testGroup/TEST_LINE_TRANSIENT",
        ),
        (
            LICENCES,
            "base/bif/CHAROUT\tTEST_STDOUT_START\tFlaky\tit is slow",
        ),
        (
            LICENCES,
            "base/bif/CHAROUT\tTEST_SIMPLE\tFlaky\tcommon.txt: CHAROUT.testGroup/TEST_SIMPLE",
        ),
        (
            LICENCES,
            "base/bif/CHAROUT\tTEST_STDOUT_START\tNotRun\trestrictOS line 41",
        ),
        (
            NOT_RUN,
            "regressions/bug2061_newline_misalignment\tTEST_STDOUT_NEWLINE\tNotRun\trestrictOS line 40",
        ),
        (
            NOT_RUN,
            "base/rexxutil/platform/unix/tilde\tTEST_TILDE\tNotRun\trestrictOS line 41",
        ),
        (
            NOT_RUN,
            "doc/rexxref/chapter7/Section4\tTEST_BEEP_EXAMPLE_7_4_6\tNotRun\ttest type UNIT_TEST",
        ),
        (
            NOT_RUN,
            "base/bif/CHAROUT\tTEST_STDOUT_START\tNotRun\ttest type DOC_EXAMPLE_NOISE_TEST",
        ),
        (
            NOT_RUN,
            "base/bif/CHAROUT\tTEST_STDOUT_START\tNotRun\tit does not run",
        ),
        (
            NOT_RUN,
            "base/bif/CHAROUT\ttest_stdout_start\tNotRun\trestrictOS line 41",
        ),
    ];

    #[test]
    fn each_owner_check_passes_its_rows_and_fails_the_others() {
        let context = context(GROUPS);
        for (file, line) in PASSING {
            assert_eq!(
                problems(file, line, &context),
                Vec::<String>::new(),
                "{file}: {line}"
            );
        }
        for (file, line) in FAILING {
            assert!(
                !problems(file, line, &context).is_empty(),
                "passed: {file}: {line}"
            );
        }
    }

    #[test]
    fn the_rxapi_rows_are_reaching_rxapis_output() {
        let context = context(&["base/bif/RXQUEUE", "base/bif/CHAROUT"]);
        let renamed: Vec<String> = context
            .rxapi
            .iter()
            .map(|(group, test)| format!("{group}\t{test}\tPhase 10 Rxapi\treaching_rxapi"))
            .collect();
        assert!(renamed.len() > 1, "RXQUEUE reaches rxapi");
        let file = |lines: &[String]| {
            let (_, rows, malformed) = parse_rows(
                EXCLUSIONS,
                &format!("# ootest revision 13178\n{}\n", lines.join("\n")),
            );
            assert_eq!(malformed, Vec::<String>::new());
            check_rows(&rows, &context)
        };
        assert_eq!(file(&renamed), Vec::<String>::new());
        assert_eq!(file(&renamed[1..]).len(), 1, "a missing row");
        let doubled = [renamed.clone(), renamed[..1].to_vec()].concat();
        assert_eq!(file(&doubled).len(), 1, "a doubled row");
    }

    #[test]
    fn a_file_without_its_revision_is_malformed() {
        let (_, _, problems) = parse_rows(
            EXCLUSIONS,
            "base/bif/RXQUEUE\tTEST_RXQUEUE_NOARG\tPhase 10 Rxapi\treaching_rxapi\n",
        );
        assert_eq!(problems.len(), 1, "{problems:?}");
    }

    #[test]
    fn the_row_10_groups_are_the_native_api_partitions() {
        let source = std::fs::read_to_string(
            worktree().join("rust/crates/rexx-exec/tests/api_group_partition.rs"),
        )
        .expect("api_group_partition.rs");
        let stems: BTreeSet<&str> = source
            .lines()
            .filter_map(|line| line.trim().strip_prefix("(\""))
            .filter_map(|rest| rest.strip_suffix("\", 10),"))
            .collect();
        let ours: BTreeSet<&str> = ROW_10_GROUPS
            .iter()
            .map(|group| group.rsplit('/').next().expect("a stem"))
            .collect();
        assert_eq!(stems, ours);
    }
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
        discover_groups, end_reason, expected_set_in, floor_from, marker, path_of, resolve,
        run_group_on_crate_within, run_group_on_oracle_from,
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
        "rexx-exec: routine \"SYSQUERYREXXMACRO\" is not implemented (Phase 10 Rxapi)";

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
    fn the_crate_side_finds_its_own_rexx_and_rexxc() {
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
            fs::read(bin.join("rexx")).expect("the crate's rexx"),
            fs::read(env!("CARGO_BIN_EXE_rexx")).expect("rexx")
        );
        assert_eq!(
            fs::canonicalize(bin.join("rexx")).expect("the crate's rexx"),
            bin.canonicalize().expect("the bin").join("rexx"),
            "a symlink resolves to the target directory, where no samples are"
        );
        assert!(
            bin.join("../samples/factor.rex").is_file(),
            "the samples beside the bin"
        );
        assert_eq!(resolve("rexxc", &path), Some(bin.join("rexxc")));
        assert_eq!(
            fs::read(bin.join("rexxc")).expect("the crate's rexxc"),
            fs::read(env!("CARGO_BIN_EXE_rexxc")).expect("rexxc")
        );
        assert!(check_path("/usr/bin:/bin", &bin).is_err());
        fs::remove_file(bin.join("rexxc")).expect("the crate's rexxc");
        assert!(check_path(&path, &bin).is_err(), "no rexxc in the bin");
        fs::remove_dir_all(run.parent().expect("a parent")).expect("cannot remove the run");
    }

    /// A fresh `run` directory below the target's temporary directory.
    fn scratch_run(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("suite-{name}-{}", std::process::id()))
            .join("run")
    }

    fn memcap() -> Option<PathBuf> {
        resolve("memcap", &path_of(std::env::var_os("PATH")))
    }

    #[test]
    fn a_child_refusing_on_the_inherited_stderr_leaves_the_run_ended_normally() {
        let run = scratch_run("child-refuses");
        let (process, tests) = crate_run_once(
            &root(),
            &run,
            "",
            "ChildRefuses",
            &BTreeSet::new(),
            memcap().as_deref(),
            Duration::from_secs(60),
        );
        assert_eq!(tests, ["TEST_A"]);
        assert!(
            String::from_utf8_lossy(&process.run.stderr).contains(MACRO_REFUSAL),
            "the child's refusal is on the run's stderr: {:?}",
            String::from_utf8_lossy(&process.run.stderr)
        );
        assert_eq!(end_reason(&process, Duration::from_secs(60)), None);
        fs::remove_dir_all(run.parent().expect("a parent")).expect("cannot remove the run");

        let run = crate_run(&settings(), "ChildRefuses");
        assert_eq!(run.iterations, 1);
        assert!(run.blocked.is_empty(), "{:?}", run.blocked);
        assert_eq!(run.passed, names(&["TEST_A"]));
        assert!(run.ended_normally);
    }

    /// Whether a process runs `sleep 41.5`, which `Grandchild`'s test starts.
    fn grandchild_sleeps() -> bool {
        fs::read_dir("/proc")
            .expect("/proc")
            .filter_map(Result::ok)
            .any(|entry| {
                fs::read(entry.path().join("cmdline"))
                    .is_ok_and(|cmdline| cmdline == b"sleep\x0041.5\x00")
            })
    }

    #[test]
    fn a_command_a_test_leaves_running_is_killed_with_the_run() {
        let run = scratch_run("grandchild");
        let (process, tests) = crate_run_once(
            &root(),
            &run,
            "",
            "Grandchild",
            &BTreeSet::new(),
            memcap().as_deref(),
            Duration::from_secs(60),
        );
        assert_eq!(tests, ["TEST_A"]);
        assert_eq!(end_reason(&process, Duration::from_secs(60)), None);
        assert!(!grandchild_sleeps(), "the command outlived the run");
        fs::remove_dir_all(run.parent().expect("a parent")).expect("cannot remove the run");
    }

    #[test]
    fn a_command_the_crate_side_runs_finds_its_rexx_and_rexxc() {
        let run = scratch_run("child-path");
        let (_, tests) = crate_run_once(
            &root(),
            &run,
            "",
            "ChildPath",
            &BTreeSet::new(),
            memcap().as_deref(),
            Duration::from_secs(60),
        );
        assert_eq!(tests, ["TEST_A"]);
        let beside = run.parent().expect("a parent");
        assert_eq!(
            fs::read_to_string(beside.join("rexx-path.txt")).expect("rexx's path"),
            format!("{}\n", interpreter_bin(&run).join("rexx").display())
        );
        assert_eq!(
            fs::read_to_string(beside.join("rexxc-path.txt")).expect("rexxc's path"),
            format!("{}\n", interpreter_bin(&run).join("rexxc").display())
        );
        fs::remove_dir_all(beside).expect("cannot remove the run");
    }

    #[test]
    fn a_run_that_starts_no_test_after_an_earlier_one_did_is_not_blocked_at_load() {
        let run = crate_run(&settings(), "RefusedAfterFirst");
        assert_eq!(run.iterations, 2);
        assert_eq!(
            run.blocked,
            BTreeMap::from([("TEST_A".to_string(), MACRO_REFUSAL.to_string())])
        );
        assert_eq!(run.blocked_at_load, None);
        assert_eq!(run.ended_before_a_test.as_deref(), Some(MACRO_REFUSAL));
        assert!(!run.ended_normally);
    }

    #[test]
    fn the_expected_set_follows_test_classes_through_the_file_as_the_framework_does() {
        let expected = expected_set_in(&root(), "", "Inherits");
        assert_eq!(
            expected,
            names(&[
                "TEST_BASE",
                "TEST_CHILD",
                "TEST_GRAND",
                "TEST_MIX",
                "TEST_MIXED"
            ])
        );
        let oracle = run_group_on_oracle_from(&root(), "", "Inherits", &BTreeSet::new());
        assert!(oracle.ended_normally);
        assert_eq!(oracle.started, expected);
        assert_eq!(oracle.passed, expected);
    }

    #[test]
    fn the_expected_set_is_what_the_framework_starts_whatever_the_file_comments_or_quotes() {
        let expected = expected_set_in(&root(), "", "Enumerated");
        assert_eq!(
            expected,
            names(&[
                "TEST_AFTER_COMMENT",
                "TEST_ATTRIBUTE",
                "TEST_CONSTANT",
                "TEST_QUOTED NAME 1.5",
                "TEST_SINGLE"
            ])
        );
        let oracle = run_group_on_oracle_from(&root(), "", "Enumerated", &BTreeSet::new());
        assert!(oracle.ended_normally);
        assert_eq!(oracle.started, expected);
    }

    #[test]
    fn the_relative_path_tests_pass_on_the_oracle_with_their_neighbours_copied() {
        for (dir, group, tests) in [
            (
                "base/bif",
                "STREAM",
                ["TEST_RELATIVE_FILE_EXISTS", "TEST_RELATIVE_FILE_EXISTS2"],
            ),
            (
                "base/class",
                "Stream",
                ["TEST_RELATIVE_EXISTS2", "TEST_RELATIVE_EXISTS3"],
            ),
        ] {
            let run = super::run_group_on_oracle(dir, group, &BTreeSet::new());
            assert!(run.ended_normally, "{group}");
            for test in tests {
                assert!(run.passed.contains(test), "{group} {test}");
            }
        }
    }

    /// The `*.cls` file names in `dir`.
    fn class_files(dir: &std::path::Path) -> BTreeSet<String> {
        fs::read_dir(dir)
            .expect("a directory")
            .filter_map(Result::ok)
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .filter(|name| name.ends_with(".cls"))
            .collect()
    }

    #[test]
    fn the_crate_side_bin_holds_the_oracle_bins_class_files_and_no_other_binary() {
        let run = scratch_run("class-files");
        crate_run_once(
            &root(),
            &run,
            "",
            "ChildPath",
            &BTreeSet::new(),
            memcap().as_deref(),
            Duration::from_secs(60),
        );
        let bin = interpreter_bin(&run);
        let oracle_bin = super::oracle_root().join("bin");
        assert!(class_files(&oracle_bin).contains("mime.cls"));
        assert_eq!(class_files(&bin), class_files(&oracle_bin));
        let path = format!("{}:/usr/bin:/bin", bin.display());
        assert_eq!(check_path(&path, &bin), Ok(()));
        fs::copy(oracle_bin.join("rxqueue"), bin.join("rxqueue")).expect("an oracle binary");
        assert!(check_path(&path, &bin).is_err(), "an oracle ELF in the bin");
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
