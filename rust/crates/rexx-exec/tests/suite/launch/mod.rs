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

//! Bounded group launches and their owned work-directory lifetime.

use super::*;

/// The private temporary directory for the logical SysFileTree group only.
pub(super) fn temporary_directory(run: &Path, group: &str) -> Option<PathBuf> {
    (group == "base/rexxutil/SysFileTree").then(|| run.join("sysfiletree-tmp"))
}

/// Seeds the selected child temporary directory in a freshly prepared run.
fn prepare_temporary_directory(run: &Path, group: &str) -> Option<PathBuf> {
    let directory = temporary_directory(run, group)?;
    fs::create_dir(&directory).expect("create SysFileTree child temporary directory");
    fs::write(directory.join("stable-file.txt"), b"stable\n")
        .expect("seed SysFileTree temporary file");
    fs::create_dir(directory.join("stable-directory"))
        .expect("seed SysFileTree temporary directory");
    Some(directory)
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
    let temporary = prepare_temporary_directory(&run, &group_path(dir, group));
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
    if let Some(temporary) = temporary {
        command.env("TMPDIR", temporary);
    }
    let process = run_process(command, None, ORACLE_DEADLINE);
    let mut result = GroupRun::new(group_path(dir, group), CapKind::OracleUlimit);
    record(
        &mut result,
        &process,
        started_in_file(&marker(&run)),
        ORACLE_DEADLINE,
    );
    diagnostics::capture(&run, "oracle", &process, &result);
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
        let reason = record(&mut result, &process, tests, deadline);
        diagnostics::capture(&run, "crate", &process, &result);
        let Some(_) = reason else {
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
    let temporary = prepare_temporary_directory(run, &group_path(dir, group));
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
        deadline.as_secs_f64().to_string(),
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
    if let Some(temporary) = temporary {
        command.env("TMPDIR", temporary);
    }
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
