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
    assert_eq!(run.passed, names(&["TEST_A"]), "{run:#?}");
    assert!(run.ended_normally, "{run:#?}");
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
    assert!(
        !super::process_ownership::grandchild_sleeps(&run),
        "the command outlived the run"
    );
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


/// A bounded diagnostic of the original group, with metadata traces added
/// only to a private source copy. Passing this does not close the queued flake.
#[test]
#[ignore = "bounded SysFileTree diagnosis, run explicitly with retained records"]
fn sysfiletree_group_retains_assertion_and_metadata_context() {
    let source = super::groups_root().join("base/rexxutil/SysFileTree.testGroup");
    let private = scratch_run("sysfiletree-diagnostic-source");
    fs::create_dir_all(&private).unwrap();
    let mut text = fs::read_to_string(source).unwrap();
    text = text.replace("  use strict arg line, typeT, typeS, file, message", "  use strict arg line, typeT, typeS, file, message\n  .error~say('SFT assert' message 'actual='line 'expected='file~absolutePath 'mtime='file~lastModified 'size='file~length 'cwd='directory() 'TZ='value('TZ', , 'ENVIRONMENT'))");
    text = text.replace("-- will run exactly once after all tests have finished", "  .error~say('SFT setup complete cwd='directory())\n  do f over allFiles\n    .error~say('SFT metadata' f~absolutePath f~lastModified f~length)\n  end\n\n-- will run exactly once after all tests have finished");
    text = text.replace("::method remove class\n  expose allFiles root current", "::method remove class\n  expose allFiles root current\n  .error~say('SFT teardown cwd='directory())");
    fs::write(private.join("SysFileTree.testGroup"), text).unwrap();
    let oracle = run_group_on_oracle_from(&private, "", "SysFileTree", &BTreeSet::new());
    let settings = CrateSettings { root: private.clone(), ..settings() };
    let ours = run_group_on_crate_within(&settings, "", "SysFileTree", &BTreeSet::new(), Duration::from_secs(60));
    eprintln!("SysFileTree oracle={oracle:#?} crate={ours:#?}");
    assert!(oracle.failed.is_empty(), "{oracle:#?}");
    assert!(ours.failed.is_empty(), "{ours:#?}");
    assert!(oracle.passed.contains("TEST_TREE_DIRECTORY"), "{oracle:#?}");
    assert!(ours.passed.contains("TEST_TREE_DIRECTORY"), "{ours:#?}");
    fs::remove_dir_all(private.parent().unwrap()).unwrap();
}
