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

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use super::group_runner::{
    GATE_ENV, Outcome, Run, SwitchMode, TestResult, VERBOSITY, excerpt, first_difference,
    fresh_copy, gate_mode, group_file, masked, on_a_quiet_machine, reaching_rxapi,
    rows_in_parallel, run_crate, run_tests, source_test_names, test_names,
};
use super::support::oracle;

/// Where the Message table is written when set, for the SDD record.
const TABLE_ENV: &str = "REXX_GROUP_TABLE";

/// Where the Object table is written when set.
const OBJECT_TABLE_ENV: &str = "REXX_OBJECT_TABLE";

/// Where the Message start tests' table under `EveryOpportunity` is
/// written when set.
const SWITCHED_TABLE_ENV: &str = "REXX_SWITCHED_TABLE";

/// The Message start tests that do not pass, each with the method its
/// refusal names.
const MESSAGE_START_REFUSED: &[(&str, &str)] = &[("TEST_STARTWITH_NOT_ARRAY", "MAKEARRAY")];

/// The Message start tests whose outcome differs from the oracle's.
const MESSAGE_START_DIFFERING: &[&str] = &[];

/// The same under `EveryOpportunity`.
const MESSAGE_START_DIFFERING_SWITCHED: &[&str] = &[];

fn scratch(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("{name}-{}", std::process::id()))
        .join("run")
}

fn listed(oracle: &oracle::Oracle, run: &Path, dir: &str, group: &str) -> Vec<String> {
    let not_run: BTreeSet<String> = reaching_rxapi(dir, &[group]);
    test_names(oracle, run, dir, group, &not_run)
}

#[test]
fn the_runner_passes_a_passing_group_and_reports_an_altered_test_as_differing() {
    if !gate_mode() {
        eprintln!("group_runs: skipped without {GATE_ENV}");
        return;
    }
    let oracle = oracle::locate();
    let run = scratch("group-runner-self-test");
    let (dir, group) = ("API/oo", "CONVERSION");
    let tests = listed(&oracle, &run, dir, group);
    assert_eq!(
        source_test_names(dir, group),
        tests,
        "the source's test list against the oracle's listing"
    );
    let results = run_tests(&oracle, &run, dir, group, &tests, SwitchMode::None, None);
    let not_passing: Vec<String> = results
        .iter()
        .filter(|row| !matches!(row.outcome, Outcome::Pass))
        .map(|row| format!("{} {}", row.test, row.outcome.label()))
        .collect();
    assert!(not_passing.is_empty(), "not passing: {not_passing:?}");

    let altered = |run: &Path| {
        let file = group_file(run, dir, group);
        let text = fs::read_to_string(&file).expect("the copied group file");
        let from = "self~assertSame(1, .CONVERSIONTester~TestObjectToValue(1.0,";
        assert!(text.contains(from), "the altered assertion is in the group");
        fs::write(&file, text.replacen(from, &from.replace("(1,", "(2,"), 1))
            .expect("cannot rewrite the copied group file");
    };
    let name = tests
        .iter()
        .find(|name| name.eq_ignore_ascii_case("testint01"))
        .expect("testint01 is listed")
        .clone();
    let results = run_tests(
        &oracle,
        &run,
        dir,
        group,
        std::slice::from_ref(&name),
        SwitchMode::None,
        Some(&altered),
    );
    let Outcome::Differ {
        oracle: theirs,
        ours,
    } = &results[0].outcome
    else {
        panic!("the altered test is {}", results[0].outcome.label());
    };
    eprintln!(
        "altered {name}: {}",
        first_difference(&masked(&theirs.stdout), &masked(&ours.stdout))
    );
    fs::remove_dir_all(run.parent().expect("a parent")).expect("cannot remove the run");
}

/// Runs every test of `group` on both sides and answers the outcomes,
/// with the table written to the file `table_env` names where set.
fn outcome_table(
    name: &str,
    dir: &str,
    group: &str,
    table_env: &str,
    mode: SwitchMode,
    chosen: impl Fn(&str) -> bool,
) -> Vec<TestResult> {
    let oracle = oracle::locate();
    let tests: Vec<String> = source_test_names(dir, group)
        .into_iter()
        .filter(|test| chosen(test))
        .collect();
    let results = rows_in_parallel(
        name,
        &tests,
        |test| WALL_CLOCK.contains(&format!("{dir}/{group}.testGroup {test}").as_str()),
        |run, test| {
            let one = std::slice::from_ref(test);
            let mut results = run_tests(&oracle, run, dir, group, one, mode.clone(), None);
            results.pop().expect("one result")
        },
    );
    let mut table = String::new();
    for row in &results {
        let detail = match &row.outcome {
            Outcome::Refused { message, .. } => message.clone(),
            Outcome::Differ { oracle, ours } => format!(
                "{}; ours stderr {:?}",
                first_difference(&masked(&oracle.stdout), &masked(&ours.stdout)),
                excerpt(&ours.stderr)
            ),
            _ => String::new(),
        };
        table.push_str(&format!(
            "{}\t{}\t{detail}\n",
            row.test,
            row.outcome.label()
        ));
    }
    eprintln!("{table}");
    if let Some(path) = std::env::var_os(table_env) {
        fs::write(path, &table).expect("cannot write the table");
    }
    assert_eq!(results.len(), tests.len());
    results
}

/// The start tests of `results` that neither pass nor are refused as
/// `refused` says nor differ where `differing` names them.
fn not_passing(
    results: &[TestResult],
    refused: &[(&str, &str)],
    differing: &[&str],
) -> Vec<String> {
    results
        .iter()
        .filter(|row| row.test.to_ascii_uppercase().contains("START"))
        .filter(|row| match &row.outcome {
            Outcome::Pass => false,
            Outcome::Refused { message, .. } => !refused.iter().any(|(test, method)| {
                row.test.eq_ignore_ascii_case(test)
                    && message.contains(&format!("method \"{method}\""))
            }),
            Outcome::Differ { .. } => !differing
                .iter()
                .any(|test| row.test.eq_ignore_ascii_case(test)),
            _ => true,
        })
        .map(|row| format!("{} {}", row.test, row.outcome.label()))
        .collect()
}

#[test]
fn the_outcome_table_of_the_message_group() {
    if !gate_mode() {
        eprintln!("group_runs: skipped without {GATE_ENV}");
        return;
    }
    let results = outcome_table(
        "message-table",
        "base/class",
        "Message",
        TABLE_ENV,
        SwitchMode::None,
        |_| true,
    );
    let failing = not_passing(&results, MESSAGE_START_REFUSED, MESSAGE_START_DIFFERING);
    assert!(failing.is_empty(), "not passing: {failing:?}");
    for test in ["TEST_SEND", "TEST_START", "TEST_REPLY", "TEST_NOTIFY"] {
        let row = results
            .iter()
            .find(|row| row.test.eq_ignore_ascii_case(test))
            .unwrap_or_else(|| panic!("{test} is listed"));
        assert!(
            matches!(row.outcome, Outcome::Pass),
            "{test} is {}",
            row.outcome.label()
        );
    }
}

/// The GUARD group's tests that pass: every test of the group.
const GUARD_PASSING: &[&str] = &[
    "TEST_INVALID_OPTION_ONOFF",
    "TEST_INVALID_OPTION_ONOFF_WHEN",
    "TEST_INVALID_OPTION_WHEN",
    "TEST_NO_OPTION",
    "TEST_OFF",
    "TEST_ON",
    "TEST_ON_DEFAULT",
    "TEST_ON_OFF",
    "TEST_ON_OFF_CONSECUTIVE",
    "TEST_UNGUARDED",
    "TEST_WAIT_MULTIPLE",
    "TEST_WAIT_SIMPLE",
    "TEST_WAIT_SIMPLE_TRIGGER",
    "TEST_WHEN_EXPRESSION_COMPOUND",
    "TEST_WHEN_EXPRESSION_CONSTANT",
    "TEST_WHEN_EXPRESSION_REFERENCE",
    "TEST_WHEN_EXPRESSION_SELF",
    "TEST_WHEN_MULTIPLE_NO_WAIT",
    "TEST_WHEN_NOT_BOOLEAN",
    "TEST_WHEN_NOVALUE",
    "TEST_WHEN_SINGLE_NO_WAIT",
    "TEST_WHEN_SINGLE_UNINITIALIZED_NO_WAIT",
    "TEST_WHEN_USE_LOCAL",
    "TEST_WHEN_USE_LOCAL_NO_WAIT",
];

/// [`GUARD_PASSING`] with the shipped scheduler and with a switch at every
/// opportunity, and every other test refused.
#[test]
fn the_outcome_table_of_the_guard_group_in_both_modes() {
    if !gate_mode() {
        eprintln!("group_runs: skipped without {GATE_ENV}");
        return;
    }
    for (name, mode) in [
        ("guard-table", SwitchMode::None),
        ("guard-table-switched", SwitchMode::EveryOpportunity),
    ] {
        let results = outcome_table(
            name,
            "base/keyword",
            "GUARD",
            "REXX_GUARD_TABLE",
            mode,
            |_| true,
        );
        let passing: Vec<&str> = results
            .iter()
            .filter(|row| matches!(row.outcome, Outcome::Pass))
            .map(|row| row.test.as_str())
            .collect();
        assert_eq!(passing, GUARD_PASSING, "{name}");
        let other: Vec<String> = results
            .iter()
            .filter(|row| !matches!(row.outcome, Outcome::Pass | Outcome::Refused { .. }))
            .map(|row| format!("{} {}", row.test, row.outcome.label()))
            .collect();
        assert!(
            other.is_empty(),
            "{name}: neither passing nor refused: {other:?}"
        );
    }
}

/// Every test of the `Alarm` and `Ticker` groups passes with the shipped
/// scheduler and with a switch at every opportunity; a [`WALL_CLOCK`] row
/// that does not is run again once, the oracle too, and the table written
/// to `REXX_TIMER_TABLE` names the rerun.
#[test]
fn the_alarm_and_ticker_groups_pass_in_both_modes() {
    if !gate_mode() {
        eprintln!("group_runs: skipped without {GATE_ENV}");
        return;
    }
    let mut not_passing = Vec::new();
    let mut table = String::from("group\tmode\ttest\toutcome\n");
    for group in ["Alarm", "Ticker"] {
        for (name, mode) in [
            ("timer-table", SwitchMode::None),
            ("timer-table-switched", SwitchMode::EveryOpportunity),
        ] {
            let results = outcome_table(
                name,
                "base/class",
                group,
                "REXX_TIMER_TABLE",
                mode.clone(),
                |_| true,
            );
            let oracle = oracle::locate();
            for row in &results {
                let mut label = row.outcome.label().to_string();
                let key = format!("base/class/{group}.testGroup {}", row.test);
                if !matches!(row.outcome, Outcome::Pass) && WALL_CLOCK.contains(&key.as_str()) {
                    eprintln!("P48 rerun: {key} {name}: {label}");
                    let run = scratch(&format!("{name}-rerun"));
                    let one = std::slice::from_ref(&row.test);
                    let again = on_a_quiet_machine(|| {
                        run_tests(&oracle, &run, "base/class", group, one, mode.clone(), None)
                    });
                    fs::remove_dir_all(run.parent().expect("a parent"))
                        .expect("cannot remove the run");
                    label = format!("{label}, then {} (P48 rerun)", again[0].outcome.label());
                    eprintln!("P48 rerun: {key} {name}: {}", again[0].outcome.label());
                    if !matches!(again[0].outcome, Outcome::Pass) {
                        not_passing.push(format!("{group} {name} {} {label}", row.test));
                    }
                } else if !matches!(row.outcome, Outcome::Pass) {
                    not_passing.push(format!("{group} {name} {} {label}", row.test));
                }
                table.push_str(&format!("{group}\t{name}\t{}\t{label}\n", row.test));
            }
        }
    }
    eprintln!("{table}");
    // Written over the last group's own table, so the file holds both
    // groups in both modes and each rerun.
    if let Some(path) = std::env::var_os("REXX_TIMER_TABLE") {
        fs::write(path, &table).expect("cannot write the table");
    }
    assert!(not_passing.is_empty(), "not passing: {not_passing:?}");
}

#[test]
fn the_outcome_table_of_the_object_group() {
    if !gate_mode() {
        eprintln!("group_runs: skipped without {GATE_ENV}");
        return;
    }
    // The start tests only: `TEST_UNINIT` and `TEST_UNINIT_CLASS`
    // allocate until the memory cap stops them, on the base as here, and
    // two at once outgrow the gate's.
    let results = outcome_table(
        "object-table",
        "base/class",
        "Object",
        OBJECT_TABLE_ENV,
        SwitchMode::None,
        |test| test.to_ascii_uppercase().contains("START"),
    );
    let failing = not_passing(&results, &[], &[]);
    assert!(failing.is_empty(), "not passing: {failing:?}");
}

/// The Message start tests again with a switch at every clause boundary
/// the started activities reach.
#[test]
fn the_message_start_tests_under_every_opportunity() {
    if !gate_mode() {
        eprintln!("group_runs: skipped without {GATE_ENV}");
        return;
    }
    let results = outcome_table(
        "message-table-switched",
        "base/class",
        "Message",
        SWITCHED_TABLE_ENV,
        SwitchMode::EveryOpportunity,
        |test| test.to_ascii_uppercase().contains("START"),
    );
    let failing = not_passing(
        &results,
        MESSAGE_START_REFUSED,
        MESSAGE_START_DIFFERING_SWITCHED,
    );
    assert!(failing.is_empty(), "not passing: {failing:?}");
}

/// The features of criterion 1's list that S2 covers; a row naming none
/// of them belongs to a later stage.
const S2_FEATURES: &[&str] = &[
    "REPLY",
    "~start",
    "Message~reply",
    "SysSleep",
    ".context~thread",
    "TraceObject field",
];

/// Where the both-modes table of the S2 rows is written when set.
const CRITERION_ONE_TABLE_ENV: &str = "REXX_CRITERION_ONE_TABLE";

/// The S2 rows whose runs differ between the two modes, in the assertion
/// count alone: each asserts after a `REPLY` in the continuation, which
/// races the end of the program. The shipped scheduler ends first;
/// `EveryOpportunity` runs the continuation first (ruling P41).
const MODE_DIFFERING: &[&str] = &[
    "base/keyword/REPLY.testGroup TEST_REPLY_TWICE_REPLYASSERT",
    "base/keyword/REPLY.testGroup TEST_REPLY_RETURN_CODE_REPLYASSERT",
    "base/keyword/REPLY.testGroup TEST_REPLY_RETURN_CODE_SAME_REPLYASSERT",
    "base/keyword/REPLY.testGroup TEST_REPLY_EXIT_CODE_REPLYASSERT",
    "base/keyword/REPLY.testGroup TEST_REPLY_STACK_REPLYASSERT",
    "base/keyword/REPLY.testGroup TEST_REPLY_SAME_REPLYASSERT",
];

/// The text of the crate's inverted-wait refusal (`lib.rs`, `inverted_wait`).
const INVERTED_WAIT: &str = "that only an activity pinned below it can end";

/// Whether `run` ended with the inverted-wait refusal.
fn inverted(run: &Run) -> bool {
    String::from_utf8_lossy(&run.stderr).contains(INVERTED_WAIT)
}

/// The rows whose failure output prints an elapsed time, which differs
/// between any two runs.
const PRINTS_ELAPSED: &[&str] = &[
    "base/bif/TIME.testGroup TEST_4",
    "base/bif/TIME.testGroup TEST_5",
    "base/bif/TIME.testGroup TEST_10",
    "base/bif/TIME.testGroup TEST_11",
];

/// The rows whose trace output, on stderr, comes from a `REPLY`
/// continuation and its sender, so the lines interleave differently by
/// mode. Allowed while both runs end in the same `rexx-exec: ` refusal, or
/// end alike with the same stderr lines in another order.
const TRACE_INTERLEAVES: &[&str] =
    &["base/keyword/TRACE_TraceObject.testGroup TEST_TRACEOBJECT_COLLECTOR"];

/// The rows whose outcome on this crate depends on a wall-clock boundary or
/// a sleep's duration, each with the lines that make it so: a failing check
/// on one is run again once, and only a second failure counts (ruling P48).
const WALL_CLOCK: &[&str] = &[
    // STREAM.testGroup:866-876
    "base/bif/STREAM.testGroup TEST_QUERYDIR_EXISTS",
    // DateTime.testGroup:593-596
    "base/class/DateTime.testGroup TEST_ELAPSED1",
    // SysSleep.testGroup:83-84, :107-113
    "base/rexxutil/SysSleep.testGroup TEST_SLEEP_DURATION",
    // SysSleep.testGroup:91-99
    "base/rexxutil/SysSleep.testGroup TEST_SLEEP_CONCURRENT",
    // EventSemaphore.testGroup:151-155, :162
    "base/class/EventSemaphore.testGroup TEST_WAIT_CONCURRENT",
    // MutexSemaphore.testGroup:124, :128
    "base/class/MutexSemaphore.testGroup TEST_EXCLUSION",
    // Message.testGroup:651-652, :667-668, :685-686, :958, :977
    "base/class/Message.testGroup TEST_HALT_START",
    // CALL.testGroup:424-425
    "base/keyword/CALL.testGroup TEST_4",
    // TIME.testGroup:1840, :1875 and the same midnight assertion in each
    "base/bif/TIME.testGroup TEST_2",
    "base/bif/TIME.testGroup TEST_3",
    "base/bif/TIME.testGroup TEST_4",
    "base/bif/TIME.testGroup TEST_5",
    "base/bif/TIME.testGroup TEST_8",
    "base/bif/TIME.testGroup TEST_9",
    "base/bif/TIME.testGroup TEST_10",
    "base/bif/TIME.testGroup TEST_11",
    // Ticker.testGroup:52-55, :76-79, :100-103, :123-126
    "base/class/Ticker.testGroup TEST_TICKER_TWO_ARGS_STRING_CANCEL",
    "base/class/Ticker.testGroup TEST_TICKER_TWO_ARGS_TIMESPAN_CANCEL",
    "base/class/Ticker.testGroup TEST_TICKER_THREE_ARGS_STRING_CANCEL",
    "base/class/Ticker.testGroup TEST_TICKER_THREE_ARGS_TIMESPAN_CANCEL",
    // Ticker.testGroup:62-65, :86-89, :110-113, :133-136, :164-170
    "base/class/Ticker.testGroup TEST_TICKER_TWO_ARGS_STRING_TRIGGER",
    "base/class/Ticker.testGroup TEST_TICKER_TWO_ARGS_TIMESPAN_TRIGGER",
    "base/class/Ticker.testGroup TEST_TICKER_THREE_ARGS_STRING_TRIGGER",
    "base/class/Ticker.testGroup TEST_TICKER_THREE_ARGS_TIMESPAN_TRIGGER",
    "base/class/Ticker.testGroup TEST_TICKER_THREE_ARGS_STRING_TRIGGER_MESSAGE",
    // Ticker.testGroup:147-150
    "base/class/Ticker.testGroup TEST_TICKER_THREE_ARGS_TIMESPAN_TRIGGER_MULTIPLE",
    // Alarm.testGroup:57-64, :74, :119-127, :137-146, :157-167, :201-209
    "base/class/Alarm.testGroup TEST_BASE_ALARM",
];

/// The row whose run under `EveryOpportunity` hangs at the test's own
/// race (ruling P46): main takes the mutex between the worker's
/// `step = 6` and its last `acquire` and never releases it. The oracle
/// hangs in the same interleaving, 3 of 3 at rc 137, on
/// `docs/superpowers/records/2026-10-01-phase-6-s2-s5/exclusion-every-hang.rex`.
/// Allowed only while the unswitched run passes at rc 0 and the switched
/// one ends at the deadline.
const EVERY_FORCES_THE_RACE: &str = "base/class/MutexSemaphore.testGroup TEST_EXCLUSION";

/// `run`'s stdout without the lines `drop` selects.
fn without(run: &Run, drop: impl Fn(&str) -> bool) -> String {
    String::from_utf8_lossy(&masked(&run.stdout))
        .lines()
        .filter(|line| !drop(line))
        .collect::<Vec<_>>()
        .join("\n")
}

fn assertions(run: &Run) -> String {
    String::from_utf8_lossy(&run.stdout)
        .lines()
        .find_map(|line| line.strip_prefix("Assertions:"))
        .map_or_else(|| "none".to_string(), |count| count.trim().to_string())
}

/// How two runs of this crate compare, and whether the difference, if
/// any, is one `row` is listed for.
fn compare_modes(row: &str, normal: &Run, every: &Run) -> (String, bool) {
    if masked(&normal.stdout) == masked(&every.stdout)
        && normal.stderr == every.stderr
        && normal.status == every.status
    {
        return ("same".to_string(), true);
    }
    let quiet = normal.stderr == every.stderr && normal.status == every.status;
    if quiet && normal.status == Some(0) && MODE_DIFFERING.contains(&row) {
        let count = |line: &str| line.starts_with("Assertions:");
        if without(normal, count) == without(every, count) {
            return (
                format!(
                    "assertions {} then {}",
                    assertions(normal),
                    assertions(every)
                ),
                true,
            );
        }
    }
    if quiet && PRINTS_ELAPSED.contains(&row) {
        let timed = |line: &str| {
            let line = line.trim_start();
            line.starts_with("[failure]")
                || line.starts_with("Expected:")
                || line.starts_with("Actual:")
                || line.starts_with("Message:")
        };
        if without(normal, timed) == without(every, timed) {
            return ("same apart from elapsed values".to_string(), true);
        }
    }
    let refusal = |run: &Run| {
        String::from_utf8_lossy(&run.stderr)
            .lines()
            .find(|line| line.starts_with("rexx-exec: "))
            .map(str::to_string)
    };
    let sorted_lines = |run: &Run| {
        let mut lines: Vec<String> = String::from_utf8_lossy(&run.stderr)
            .lines()
            .map(str::to_string)
            .collect();
        lines.sort();
        lines
    };
    if normal.status == every.status
        && masked(&normal.stdout) == masked(&every.stdout)
        && TRACE_INTERLEAVES.contains(&row)
    {
        if refusal(normal).is_some() && refusal(normal) == refusal(every) {
            return ("trace lines on stderr differ".to_string(), true);
        }
        if sorted_lines(normal) == sorted_lines(every) {
            return ("trace lines on stderr in another order".to_string(), true);
        }
    }
    (
        format!(
            "differs: status {:?} then {:?}",
            normal.status, every.status
        ),
        false,
    )
}

/// A result as a key and a detail. Two results agree when their keys do:
/// the label, and the refusal or the status of a difference. The detail
/// is the first line where a difference's stdout departs from the
/// oracle's.
fn cell(outcome: &Outcome) -> (String, String) {
    match outcome {
        Outcome::Refused { message, .. } => (format!("refused: {message}"), String::new()),
        Outcome::Differ { oracle, ours } => {
            let detail = first_difference(&masked(&oracle.stdout), &masked(&ours.stdout))
                .replace(['\n', '\t'], " ");
            match ours.status {
                None => ("differ: did not finish".to_string(), detail),
                Some(status) => (format!("differ: rc {status}"), detail),
            }
        }
        other => (other.label().to_string(), String::new()),
    }
}

/// The features of criterion 1's list that S3 covers.
const S3_FEATURES: &[&str] = &["GUARD", "semaphore class", "Sys*Sem", "Alarm", "Ticker"];

/// Where the both-modes table of the S3 rows is written when set.
const S3_TABLE_ENV: &str = "REXX_CRITERION_ONE_S3_TABLE";

/// Every S2 row of the derived list, run on both sides with the shipped
/// scheduler and under `EveryOpportunity`.
#[test]
fn the_s2_rows_of_the_derived_list_in_both_modes() {
    rows_in_both_modes("criterion-one", CRITERION_ONE_TABLE_ENV, |features| {
        features.iter().any(|f| S2_FEATURES.contains(f))
    });
}

/// Every row of the derived list naming an S3 feature and no S2 one, as
/// [`the_s2_rows_of_the_derived_list_in_both_modes`] runs the others.
#[test]
fn the_s3_rows_of_the_derived_list_in_both_modes() {
    rows_in_both_modes("criterion-one-s3", S3_TABLE_ENV, |features| {
        !features.iter().any(|f| S2_FEATURES.contains(f))
            && features.iter().any(|f| S3_FEATURES.contains(f))
    });
}

/// The rows of the derived list whose features `chosen` accepts, run in
/// the scratch directory `name`, with the table written to the file
/// `table_env` names where set.
fn rows_in_both_modes(
    name: &str,
    table_env: &str,
    chosen: impl Fn(&BTreeSet<&'static str>) -> bool,
) {
    if !gate_mode() {
        eprintln!("group_runs: skipped without {GATE_ENV}");
        return;
    }
    let (list, _) = super::derive(&super::worktree().join("ootest/ooRexx"));
    let mut groups: std::collections::BTreeMap<String, Vec<String>> = Default::default();
    for row in list.iter().filter(|row| chosen(&row.features)) {
        groups
            .entry(row.group.clone())
            .or_default()
            .push(row.test.clone());
    }
    let rows: Vec<(&String, &String)> = groups
        .iter()
        .flat_map(|(file, tests)| tests.iter().map(move |test| (file, test)))
        .collect();
    let oracle = oracle::locate();
    let results = rows_in_parallel(
        name,
        &rows,
        |(file, test)| WALL_CLOCK.contains(&format!("{file} {test}").as_str()),
        |run, (file, test)| row_in_both_modes(&oracle, run, file, test),
    );
    let mut table = String::from("group\ttest\tnormal against the oracle\tevery against normal\n");
    let mut differing = Vec::new();
    let mut stuck = Vec::new();
    for ((file, test), (normal, modes, hung)) in rows.iter().zip(results) {
        stuck.extend(hung);
        if !modes.1 {
            differing.push(format!("{file} {test}"));
        }
        let show = |(key, detail): &(String, String)| {
            if detail.is_empty() {
                key.clone()
            } else {
                format!("{key}: {detail}")
            }
        };
        table.push_str(&format!("{file}\t{test}\t{}\t{}\n", show(&normal), modes.0));
    }
    eprintln!("{table}");
    if let Some(path) = std::env::var_os(table_env) {
        fs::write(path, &table).expect("cannot write the table");
    }
    assert!(stuck.is_empty(), "an inverted wait or a hang: {stuck:?}");
    assert!(
        differing.is_empty(),
        "tests differing between the modes: {differing:?}"
    );
}

/// One row of [`rows_in_both_modes`] in the copy `run`: its cell against
/// the oracle, how the two modes compare and whether that is allowed, and
/// the modes that hung or ended in an inverted wait.
fn row_in_both_modes(
    oracle: &oracle::Oracle,
    run: &Path,
    file: &str,
    test: &str,
) -> ((String, String), (String, bool), Vec<String>) {
    let (dir, group) = file
        .trim_end_matches(".testGroup")
        .rsplit_once('/')
        .expect("a group below a directory");
    let one = &[test.to_string()];
    // The runner asserts that the oracle finishes within the deadline; a
    // test it outlasts is recorded as such.
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        run_tests(oracle, run, dir, group, one, SwitchMode::None, None)
    }));
    let normal = match result {
        Ok(results) => cell(&results[0].outcome),
        Err(payload) => {
            let deadline = payload
                .downcast_ref::<String>()
                .is_some_and(|m| m.starts_with("the oracle did not finish"));
            if !deadline {
                std::panic::resume_unwind(payload);
            }
            ("oracle did not finish".to_string(), String::new())
        }
    };
    if reaching_rxapi(dir, &[group]).contains(&format!("{group}.{test}")) {
        return (normal, ("not run".to_string(), true), Vec::new());
    }
    let row = format!("{file} {test}");
    let judge = || {
        let group_path = group_file(run, dir, group);
        let args = ["-f", group_path.as_str(), "-U", "-V", VERBOSITY, "-t", test];
        fresh_copy(run, dir);
        let shipped = run_crate(run, &args, SwitchMode::None);
        fresh_copy(run, dir);
        let every = run_crate(run, &args, SwitchMode::EveryOpportunity);
        let raced = row == EVERY_FORCES_THE_RACE
            && normal.0 == "pass"
            && shipped.status == Some(0)
            && !inverted(&shipped)
            && every.status.is_none()
            && !inverted(&every);
        let mut hung = Vec::new();
        if raced {
            let modes = (
                "every mode forces the test's own race; the oracle hangs in the \
                     same interleaving (P46)"
                    .to_string(),
                true,
            );
            return (modes, hung);
        }
        for (name, ours) in [("normal", &shipped), ("every", &every)] {
            if inverted(ours) || ours.status.is_none() {
                hung.push(format!("{row}: {name}"));
            }
        }
        (compare_modes(&row, &shipped, &every), hung)
    };
    let (mut judged, mut hung) = judge();
    if (!judged.1 || !hung.is_empty()) && WALL_CLOCK.contains(&row.as_str()) {
        eprintln!("P48 rerun: {row}: {} {hung:?}", judged.0);
        (judged, hung) = judge();
        judged.0.push_str(" (P48 rerun)");
    }
    (normal, judged, hung)
}

/// The complete passing population of `TRACE_TraceObject`.
/// The table printed beside it reports each test's compared outcome.
const TRACE_OBJECT_PASSING: &[&str] = &[
    "TEST_CALLER_STACK_FRAME",
    "TEST_CALLER_STACK_FRAME_REPLY_START",
    "TEST_OBJECT_AND_SCOPE",
    "TEST_RECEIVER",
    "TEST_SETMAKESTRING_WITH_METHOD_OBJECT",
    "TEST_TRACEOBJECT_COLLECTOR",
    "TEST_TRACEOBJECT_COLLECTOR_AND_NOTIFY_CLASS_ATTRIBUTES",
    "TEST_TRACEOBJECT_OPTION",
    "TEST_TRACEOBJECT_OPTION_INVALID",
    "TEST_VARIABLE",
];

#[test]
fn the_outcome_table_of_the_trace_object_group() {
    if !gate_mode() {
        eprintln!("group_runs: skipped without {GATE_ENV}");
        return;
    }
    let results = outcome_table(
        "trace-object-table",
        "base/keyword",
        "TRACE_TraceObject",
        "REXX_TRACE_OBJECT_TABLE",
        SwitchMode::None,
        |_| true,
    );
    assert_eq!(results.len(), TRACE_OBJECT_PASSING.len());
    let passing: Vec<&str> = results
        .iter()
        .filter(|row| matches!(row.outcome, Outcome::Pass))
        .map(|row| row.test.as_str())
        .collect();
    assert_eq!(passing, TRACE_OBJECT_PASSING);
}

/// Each group file of criterion 1's derived list in one run on the
/// oracle and on this crate in both modes, whole and with only its tests
/// of the derived list, the tests reaching rxapi renamed out of both.
/// Gate-only.
mod whole_groups;
