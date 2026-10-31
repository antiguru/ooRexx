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
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use super::super::group_runner::{
    GATE_ENV, Run, SwitchMode, VERBOSITY, agree, failing, first_difference, fresh_copy, gate_mode,
    group_file, mark_starts, masked, outcome, reaching_rxapi, read_lossy, refusal,
    rows_in_parallel, run_crate_within, run_oracle_within, skip, started,
};
use super::super::support::oracle;
use super::{WALL_CLOCK, inverted};

/// Where the table is written when set.
const TABLE_ENV: &str = "REXX_WHOLE_GROUPS_TABLE";

/// A directory each run's descriptors are written to when set.
const DUMP_ENV: &str = "REXX_WHOLE_GROUPS_DUMP";

/// Where the tests each run here started are written when set.
const STARTED_ENV: &str = "REXX_WHOLE_GROUPS_STARTED";

/// A comma-separated list of group files (`base/class/Message.testGroup`)
/// to run alone when set, for re-deriving their rows.
const ONLY_ENV: &str = "REXX_WHOLE_GROUPS_ONLY";

/// Oracle runs per group, and per group whose oracle outcome varies or
/// differs from ours (ruling P83).
const ORACLE_RUNS: usize = 5;
const ORACLE_RUNS_UNSETTLED: usize = 30;

/// How many runs of one group run at once.
const RUNS_AT_ONCE: usize = 5;

/// The bound on one oracle run: `base/bif/TIME` whole takes about
/// 94 s.
const ORACLE_DEADLINE: Duration = Duration::from_secs(300);

/// The bound on one run here: the oracle's slowest run times
/// `CRATE_FACTOR`, and at least `CRATE_FLOOR`.
const CRATE_FACTOR: u32 = 4;
const CRATE_FLOOR: Duration = Duration::from_secs(60);

/// Which tests of the group a run holds: all, those of the derived
/// list, or all but the tests whose run here refuses (ruling P85).
#[derive(Clone, Copy, PartialEq, Eq)]
enum Part {
    Whole,
    Derived,
    Rest,
}

impl Part {
    fn label(self) -> &'static str {
        match self {
            Part::Whole => "whole",
            Part::Derived => "derived",
            Part::Rest => "rest",
        }
    }
}

/// The tests every part leaves out, on both sides, each with its reason:
/// each allocates here until memory runs out, which takes the shared
/// test process with it.
const REST_LEFT_OUT: &[(&str, &str, &str)] = &[
    (
        "base/class/Class.testGroup",
        "TEST_SUBCLASSES_GC",
        "loops until a dropped class is collected; classes are never collected (D59)",
    ),
    (
        "base/class/Object.testGroup",
        "TEST_UNINIT",
        "loops until a weak reference is cleared, which here it is not before memory runs out",
    ),
    (
        "base/class/Object.testGroup",
        "TEST_UNINIT_CLASS",
        "loops until a weak reference is cleared, which here it is not before memory runs out",
    ),
];

/// The runs here, by group, part and mode, that need not be an outcome
/// the oracle produced, each with its [`key`] and reason.
const DIFFERING: &[(&str, &str, &str, &str, &str)] = &[
    (
        "base/bif/STREAM.testGroup",
        "whole",
        "normal",
        "error, assertions 195, rc 2, last started TEST_WRONG_TOO_MANY_ARGS_S, failing [TEST_RELATIVE_FILE_EXISTS TEST_RELATIVE_FILE_EXISTS2 TEST_QUERYFILE_EXISTS_OPENED_01 TEST_QUERYFILE_EXISTS_OPENED_03 TEST_SEEK_CLOSEDFILE_2785896 TEST_SEEK_CLOSEDFILE_2787994 TEST_OPEN_WRITE_ONLY_3274050_C TEST_OPEN_WRITE_ONLY_3274050_D]",
        "tests outside the derived list that fail alone too (whole-groups/alone.txt), and TEST_RELATIVE_FILE_EXISTS and _EXISTS2, which fail on the oracle too",
    ),
    (
        "base/bif/STREAM.testGroup",
        "whole",
        "every",
        "error, assertions 195, rc 2, last started TEST_WRONG_TOO_MANY_ARGS_S, failing [TEST_RELATIVE_FILE_EXISTS TEST_RELATIVE_FILE_EXISTS2 TEST_QUERYFILE_EXISTS_OPENED_01 TEST_QUERYFILE_EXISTS_OPENED_03 TEST_SEEK_CLOSEDFILE_2785896 TEST_SEEK_CLOSEDFILE_2787994 TEST_OPEN_WRITE_ONLY_3274050_C TEST_OPEN_WRITE_ONLY_3274050_D]",
        "tests outside the derived list that fail alone too (whole-groups/alone.txt), and TEST_RELATIVE_FILE_EXISTS and _EXISTS2, which fail on the oracle too",
    ),
    (
        "base/class/Class.testGroup",
        "whole",
        "normal",
        "failure, assertions 298, rc 1, last started TEST_UNINHERIT_TWO_ARGS, failing [TEST_ACTIVATE TEST_METHODS]",
        "tests outside the derived list that fail alone too (whole-groups/alone.txt)",
    ),
    (
        "base/class/Class.testGroup",
        "whole",
        "every",
        "failure, assertions 298, rc 1, last started TEST_UNINHERIT_TWO_ARGS, failing [TEST_ACTIVATE TEST_METHODS]",
        "tests outside the derived list that fail alone too (whole-groups/alone.txt)",
    ),
    (
        "base/class/Method.testGroup",
        "whole",
        "normal",
        "error, assertions 92, rc 2, last started TEST_SOURCE, failing [TEST_NEW_ARRAY_FROM_FILE TEST_NEW_FILE_COMPILED]",
        "tests outside the derived list that fail alone too: Method NEW's array and compiled-file shapes",
    ),
    (
        "base/class/Method.testGroup",
        "whole",
        "every",
        "error, assertions 92, rc 2, last started TEST_SOURCE, failing [TEST_NEW_ARRAY_FROM_FILE TEST_NEW_FILE_COMPILED]",
        "tests outside the derived list that fail alone too: Method NEW's array and compiled-file shapes",
    ),
    (
        "base/class/MutexSemaphore.testGroup",
        "whole",
        "every",
        "rexx-exec: the run exceeded its deadline, rc none, last started TEST_RELEASE_ONE_ARG, failing []",
        "TEST_EXCLUSION's own race: the oracle hangs in it when that schedule is forced (SysSleep 0.01 after step = 6, P46)",
    ),
    (
        "base/class/MutexSemaphore.testGroup",
        "derived",
        "every",
        "rexx-exec: the run exceeded its deadline, rc none, last started TEST_RELEASE_ONE_ARG, failing []",
        "TEST_EXCLUSION's own race: the oracle hangs in it when that schedule is forced (SysSleep 0.01 after step = 6, P46)",
    ),
    (
        "base/class/Object.testGroup",
        "whole",
        "normal",
        "failure, assertions 272, rc 1, last started TEST_UNSETMETHOD_NO_ARG, failing [TEST_INSTANCEMETHOD TEST_SETMETHOD]",
        "tests outside the derived list that fail alone too (whole-groups/alone.txt)",
    ),
    (
        "base/class/Object.testGroup",
        "whole",
        "every",
        "failure, assertions 272, rc 1, last started TEST_UNSETMETHOD_NO_ARG, failing [TEST_INSTANCEMETHOD TEST_SETMETHOD]",
        "tests outside the derived list that fail alone too (whole-groups/alone.txt)",
    ),
    (
        "base/class/RexxContext.testGroup",
        "whole",
        "normal",
        "failure, assertions 358, rc 1, last started TEST_SOURCELINE_LAST_LINE, failing [TESTRS01]",
        "tests outside the derived list that fail alone too (whole-groups/alone.txt)",
    ),
    (
        "base/class/RexxContext.testGroup",
        "whole",
        "every",
        "failure, assertions 358, rc 1, last started TEST_SOURCELINE_LAST_LINE, failing [TESTRS01]",
        "tests outside the derived list that fail alone too (whole-groups/alone.txt)",
    ),
    (
        "base/directives/ATTRIBUTE.testGroup",
        "whole",
        "normal",
        "error, assertions 303, rc 2, last started TEST_SET_RETURN, failing [TESTMISPLACEDCLASSMETHOD]",
        "TESTMISPLACEDCLASSMETHOD: `::attribute 'foo' class` with a body is 99.937 here, 99.905 on the oracle",
    ),
    (
        "base/directives/ATTRIBUTE.testGroup",
        "whole",
        "every",
        "error, assertions 303, rc 2, last started TEST_SET_RETURN, failing [TESTMISPLACEDCLASSMETHOD]",
        "TESTMISPLACEDCLASSMETHOD: `::attribute 'foo' class` with a body is 99.937 here, 99.905 on the oracle",
    ),
    (
        "base/directives/CONSTANT.testGroup",
        "whole",
        "normal",
        "error, assertions 173, rc 2, last started TEST_SUBCLASS_DEFINITION, failing [TEST_EXPRESSION_NOVALUE_ERROR]",
        "tests outside the derived list that fail alone too (whole-groups/alone.txt)",
    ),
    (
        "base/directives/CONSTANT.testGroup",
        "whole",
        "every",
        "error, assertions 173, rc 2, last started TEST_SUBCLASS_DEFINITION, failing [TEST_EXPRESSION_NOVALUE_ERROR]",
        "tests outside the derived list that fail alone too (whole-groups/alone.txt)",
    ),
    (
        "base/keyword/RAISE.testGroup",
        "whole",
        "normal",
        "failure, assertions 120, rc 1, last started TEST_RAISE_USER, failing [TEST_RAISE_INSERT_CRLF TEST_RAISE_PROPAGATE TEST_RAISE_SYNTAX_EXIT_02 TEST_RAISE_SYNTAX_RETURN_02]",
        "TEST_RAISE_INSERT_CRLF as alone, and tests outside the derived list that fail alone too (whole-groups/alone.txt)",
    ),
    (
        "base/keyword/RAISE.testGroup",
        "whole",
        "every",
        "failure, assertions 120, rc 1, last started TEST_RAISE_USER, failing [TEST_RAISE_INSERT_CRLF TEST_RAISE_PROPAGATE TEST_RAISE_SYNTAX_EXIT_02 TEST_RAISE_SYNTAX_RETURN_02]",
        "TEST_RAISE_INSERT_CRLF as alone, and tests outside the derived list that fail alone too (whole-groups/alone.txt)",
    ),
    (
        "base/keyword/RAISE.testGroup",
        "derived",
        "normal",
        "failure, assertions 0, rc 1, last started TEST_RAISE_INSERT_CRLF, failing [TEST_RAISE_INSERT_CRLF]",
        "TEST_RAISE_INSERT_CRLF, as alone: message text conversion, outside Phase 6",
    ),
    (
        "base/keyword/RAISE.testGroup",
        "derived",
        "every",
        "failure, assertions 0, rc 1, last started TEST_RAISE_INSERT_CRLF, failing [TEST_RAISE_INSERT_CRLF]",
        "TEST_RAISE_INSERT_CRLF, as alone: message text conversion, outside Phase 6",
    ),
    (
        "base/keyword/REPLY.testGroup",
        "whole",
        "every",
        "pass, assertions 19, rc 0, last started TEST_REPLY__CODE_RETURN, failing []",
        "every continuation runs before the program ends; the oracle races them with its end (P41, phase-4-exclusions.txt row 22)",
    ),
    (
        "base/keyword/REPLY.testGroup",
        "derived",
        "every",
        "pass, assertions 18, rc 0, last started TEST_REPLY__CODE_RETURN, failing []",
        "every continuation runs before the program ends; the oracle races them with its end (P41, phase-4-exclusions.txt row 22)",
    ),
    (
        "base/keyword/TRACE.testGroup",
        "whole",
        "normal",
        "error, assertions 129, rc 2, last started TEST_TRACE_VALUE_NUMERIC_INVALID, failing [TEST_TRACE_DROP TEST_TRACE_EXIT TEST_TRACE_EXPOSE TEST_TRACE_IGNORED TEST_TRACE_LABEL_WITH_FORWARD TEST_TRACE_OTHER_ENTRYPOINT TEST_TRACE_PROCEDURE]",
        "tests outside the derived list that fail alone too (whole-groups/alone.txt); TEST_TRACE_LABEL_WITH_FORWARD's FORWARD/REPLY trace lines, measured alone 2026-10-07",
    ),
    (
        "base/keyword/TRACE.testGroup",
        "whole",
        "every",
        "error, assertions 129, rc 2, last started TEST_TRACE_VALUE_NUMERIC_INVALID, failing [TEST_TRACE_DROP TEST_TRACE_EXIT TEST_TRACE_EXPOSE TEST_TRACE_IGNORED TEST_TRACE_LABEL_WITH_FORWARD TEST_TRACE_OTHER_ENTRYPOINT TEST_TRACE_PROCEDURE]",
        "tests outside the derived list that fail alone too (whole-groups/alone.txt); TEST_TRACE_LABEL_WITH_FORWARD's FORWARD/REPLY trace lines, measured alone 2026-10-07",
    ),
    (
        "base/keyword/TRACE_TraceObject.testGroup",
        "whole",
        "normal",
        "error, assertions 18, rc 2, last started TEST_VARIABLE, failing [TEST_CALLER_STACK_FRAME_REPLY_START TEST_OBJECT_AND_SCOPE TEST_VARIABLE TEST_CALLER_STACK_FRAME TEST_TRACEOBJECT_COLLECTOR]",
        "tests that fail alone too, measured 2026-10-07: TraceObject VARIABLE, CALLERSTACKFRAME and collected trace lines; TEST_OBJECT_AND_SCOPE's collector receives no trace lines (0 against 19)",
    ),
    (
        "base/keyword/TRACE_TraceObject.testGroup",
        "whole",
        "every",
        "error, assertions 18, rc 2, last started TEST_VARIABLE, failing [TEST_CALLER_STACK_FRAME_REPLY_START TEST_OBJECT_AND_SCOPE TEST_VARIABLE TEST_CALLER_STACK_FRAME TEST_TRACEOBJECT_COLLECTOR]",
        "tests that fail alone too, measured 2026-10-07: TraceObject VARIABLE, CALLERSTACKFRAME and collected trace lines; TEST_OBJECT_AND_SCOPE's collector receives no trace lines (0 against 19)",
    ),
    (
        "base/keyword/TRACE_TraceObject.testGroup",
        "derived",
        "normal",
        "error, assertions 2, rc 2, last started TEST_TRACEOBJECT_COLLECTOR, failing [TEST_CALLER_STACK_FRAME_REPLY_START TEST_TRACEOBJECT_COLLECTOR]",
        "tests that fail alone too, measured 2026-10-07: TraceObject CALLERSTACKFRAME THREAD entries and collected trace lines",
    ),
    (
        "base/keyword/TRACE_TraceObject.testGroup",
        "derived",
        "every",
        "error, assertions 2, rc 2, last started TEST_TRACEOBJECT_COLLECTOR, failing [TEST_CALLER_STACK_FRAME_REPLY_START TEST_TRACEOBJECT_COLLECTOR]",
        "tests that fail alone too, measured 2026-10-07: TraceObject CALLERSTACKFRAME THREAD entries and collected trace lines",
    ),
];

/// A run's refusal or summary, its status, the last test it started
/// and the tests failing in it.
fn key(run: &Run) -> String {
    let status = run
        .status
        .map_or_else(|| "none".to_string(), |status| status.to_string());
    let last = started(run).pop().unwrap_or_else(|| "-".to_string());
    format!(
        "{}, rc {status}, last started {last}, failing [{}]",
        refusal(run).unwrap_or_else(|| outcome(&run.stdout)),
        failing(run).join(" ")
    )
}

/// `run` with its summary's assertion count masked.
fn count_masked(run: &Run) -> Run {
    let text = String::from_utf8_lossy(&run.stdout).into_owned();
    let stdout: Vec<&str> = text
        .split('\n')
        .map(|line| {
            if line.starts_with("Assertions:") {
                "Assertions:"
            } else {
                line
            }
        })
        .collect();
    Run {
        stdout: stdout.join("\n").into_bytes(),
        stderr: run.stderr.clone(),
        status: run.status,
    }
}

/// Whether the oracle's distinct outcomes `seen` differ from each other
/// only in their assertion count (ruling P86).
fn counted_only(seen: &[&Run]) -> bool {
    seen.len() > 1
        && seen
            .iter()
            .all(|run| agree(&count_masked(run), &count_masked(seen[0])))
}

/// `run` with the copy's path, and the copy's directory name in a
/// path the framework abbreviates, replaced by `RUN`, so runs from
/// different copies of one row compare.
fn relative(run: Run, at: &Path) -> Run {
    let name = at.file_name().expect("a copy name").to_string_lossy();
    let patterns = [
        at.to_string_lossy().into_owned().into_bytes(),
        format!("/run/{name}/").into_bytes(),
    ];
    let replace = |bytes: Vec<u8>| {
        patterns.iter().fold(bytes, |bytes, pattern| {
            let with: &[u8] = if pattern.starts_with(b"/run/") {
                b"/run/RUN/"
            } else {
                b"RUN"
            };
            let mut out = Vec::with_capacity(bytes.len());
            let mut rest = bytes.as_slice();
            while !rest.is_empty() {
                if rest.starts_with(pattern) {
                    out.extend_from_slice(with);
                    rest = &rest[pattern.len()..];
                } else {
                    out.push(rest[0]);
                    rest = &rest[1..];
                }
            }
            out
        })
    };
    Run {
        stdout: replace(run.stdout),
        stderr: replace(run.stderr),
        status: run.status,
    }
}

/// The copy `name` below `run` with the tests of `left_out` renamed
/// out and each test's start marked, and the group file's path in it.
fn copy(
    run: &Path,
    name: &str,
    dir: &str,
    group: &str,
    left_out: &BTreeSet<String>,
) -> (PathBuf, String) {
    let at = run.join(name);
    fresh_copy(&at, dir);
    mark_starts(&at);
    let file = group_file(&at, dir, group);
    skip(Path::new(&file), group, left_out);
    (at, file)
}

/// Runs `body` over `items` with at most [`RUNS_AT_ONCE`] at once,
/// answering the results in order.
fn at_once<I: Sync, T: Send>(items: &[I], body: impl Fn(&I) -> T + Sync) -> Vec<T> {
    let mut results = Vec::new();
    for batch in items.chunks(RUNS_AT_ONCE) {
        std::thread::scope(|scope| {
            let handles: Vec<_> = batch
                .iter()
                .map(|item| scope.spawn(|| body(item)))
                .collect();
            results.extend(handles.into_iter().map(|handle| {
                handle
                    .join()
                    .unwrap_or_else(|panic| std::panic::resume_unwind(panic))
            }));
        });
    }
    results
}

/// The distinct outcomes of `runs`, each with its count, most
/// frequent first.
fn distribution(runs: &[(Run, Duration)]) -> Vec<(&Run, usize)> {
    let mut distinct: Vec<(&Run, usize)> = Vec::new();
    for (run, _) in runs {
        match distinct.iter_mut().find(|(seen, _)| agree(seen, run)) {
            Some((_, count)) => *count += 1,
            None => distinct.push((run, 1)),
        }
    }
    distinct.sort_by_key(|(_, count)| std::cmp::Reverse(*count));
    distinct
}

fn difference(theirs: &Run, ours: &Run) -> String {
    first_difference(&masked(&theirs.stdout), &masked(&ours.stdout)).replace(['\n', '\t'], " ")
}

/// One row's result: the table's cells, the tests each of our runs
/// started, whether each mode agrees with the oracle, and whether
/// each ended in an inverted wait or did not finish.
struct Row {
    part: Part,
    cells: String,
    started: String,
    keys: [String; 2],
    agrees: [bool; 2],
    stuck: [bool; 2],
}

fn dump(file: &str, part: Part, side: &str, run: &Run) {
    let Some(dir) = std::env::var_os(DUMP_ENV) else {
        return;
    };
    let stem = format!("{}-{}-{side}", file.replace('/', "__"), part.label());
    let dir = Path::new(&dir);
    std::fs::create_dir_all(dir).expect("the dump directory");
    std::fs::write(dir.join(format!("{stem}.out")), &run.stdout).expect("a dump");
    std::fs::write(dir.join(format!("{stem}.err")), &run.stderr).expect("a dump");
}

/// The group file's directory below `ootest/ooRexx` and its name.
fn split(file: &str) -> (&str, &str) {
    file.trim_end_matches(".testGroup")
        .rsplit_once('/')
        .expect("a group below a directory")
}

/// The rows of `file`'s `part`: the whole part also answers its rest
/// row when its run here refuses.
fn rows_of(
    oracle: &oracle::Oracle,
    run: &Path,
    file: &str,
    part: Part,
    derived: &BTreeSet<String>,
) -> Vec<Row> {
    let (dir, group) = split(file);
    let mut left_out = left_out_of(file, part, derived);
    let (row, ours) = checked(oracle, run, file, part, &left_out, &[]);
    let mut rows = vec![row];
    if part == Part::Whole {
        let refused = |run: &Run| {
            refusal(run).filter(|line| !line.ends_with("the run exceeded its deadline"))
        };
        let mut refusing = Vec::new();
        let mut last = ours;
        let mut before = false;
        while let Some(line) = refused(&last) {
            let Some(test) = started(&last).pop() else {
                rows[0]
                    .started
                    .push_str(&format!("\trest not run: {line} before any test started"));
                before = true;
                break;
            };
            assert!(!refusing.contains(&test), "{file}: {test} refused twice");
            left_out.insert(format!("{group}.{test}"));
            refusing.push(test);
            let (at, path) = copy(run, "r00", dir, group, &left_out);
            let args = ["-f", path.as_str(), "-U", "-V", VERBOSITY];
            last = relative(
                run_crate_within(&at, &args, SwitchMode::None, ORACLE_DEADLINE),
                &at,
            );
        }
        if !refusing.is_empty() && !before {
            rows.push(checked(oracle, run, file, Part::Rest, &left_out, &refusing).0);
        }
    }
    rows
}

/// The `GROUP.TEST`s every run of `file`'s `part` renames out: those
/// reaching rxapi, [`REST_LEFT_OUT`]'s, and for the derived part each
/// test `derived` (the derived list's tests of the group) does not
/// name.
fn left_out_of(file: &str, part: Part, derived: &BTreeSet<String>) -> BTreeSet<String> {
    let (dir, group) = split(file);
    let mut left_out = reaching_rxapi(dir, &[group]);
    for (_, test, _) in REST_LEFT_OUT.iter().filter(|(listed, ..)| *listed == file) {
        left_out.insert(format!("{group}.{test}"));
    }
    if part == Part::Derived {
        let path = super::super::worktree()
            .join("ootest/ooRexx")
            .join(dir)
            .join(format!("{group}.testGroup"));
        left_out.extend(
            super::super::units(&read_lossy(&path))
                .into_iter()
                .filter(|unit| unit.test)
                .map(|unit| unit.name.to_ascii_uppercase())
                .filter(|test| !derived.contains(test))
                .map(|test| format!("{group}.{test}")),
        );
    }
    left_out
}

/// What one mode of `row` amounts to against the oracle and
/// [`DIFFERING`]: `Ok` when allowed, else why not.
fn verdict(file: &str, row: &Row, at: usize) -> Result<(), String> {
    let mode = ["normal", "every"][at];
    let listed = DIFFERING.iter().find(|(group, part, listed_mode, ..)| {
        *group == file && *part == row.part.label() && *listed_mode == mode
    });
    match listed {
        Some(_) if row.agrees[at] => Ok(()),
        Some((.., key, _)) if row.keys[at] != *key => {
            Err(format!("listed as {key:?}, is {:?}", row.keys[at]))
        }
        Some(_) => Ok(()),
        None if row.stuck[at] => Err("an inverted wait or a hang".to_string()),
        None if !row.agrees[at] => Err(format!("not an oracle outcome: {:?}", row.keys[at])),
        None => Ok(()),
    }
}

/// [`one_row`], run again once when a mode is not allowed and the
/// group holds a row whose outcome depends on the wall clock (ruling
/// P48).
fn checked(
    oracle: &oracle::Oracle,
    run: &Path,
    file: &str,
    part: Part,
    left_out: &BTreeSet<String>,
    refusing: &[String],
) -> (Row, Run) {
    let first = one_row(oracle, run, file, part, left_out, refusing);
    let timed = WALL_CLOCK
        .iter()
        .any(|row| row.starts_with(&format!("{file} ")));
    if !timed || (0..2).all(|at| verdict(file, &first.0, at).is_ok()) {
        return first;
    }
    eprintln!("P48 rerun: {file} {}", part.label());
    let (mut row, normal) = one_row(oracle, run, file, part, left_out, refusing);
    row.cells.push_str(" (P48 rerun)");
    (row, normal)
}

/// One row: `file` with the tests of `left_out` renamed out, on the
/// oracle and here in both modes; also answers the normal run here.
fn one_row(
    oracle: &oracle::Oracle,
    run: &Path,
    file: &str,
    part: Part,
    left_out: &BTreeSet<String>,
    refusing: &[String],
) -> (Row, Run) {
    let (dir, group) = split(file);
    let oracle_run = |k: &usize| {
        let (at, path) = copy(run, &format!("o{k:02}"), dir, group, left_out);
        let args = ["-f", path.as_str(), "-U", "-V", VERBOSITY];
        let start = Instant::now();
        let theirs = run_oracle_within(oracle, &at, &args, ORACLE_DEADLINE);
        (relative(theirs, &at), start.elapsed())
    };
    let first: Vec<usize> = (0..ORACLE_RUNS).collect();
    let mut theirs = at_once(&first, oracle_run);
    let slowest = theirs.iter().map(|(_, took)| *took).max().expect("a run");
    let deadline = (slowest * CRATE_FACTOR).max(CRATE_FLOOR);
    let modes = [
        ("n00", SwitchMode::None),
        ("e00", SwitchMode::EveryOpportunity),
    ];
    let mut ours = at_once(&modes, |(name, mode)| {
        let (at, path) = copy(run, name, dir, group, left_out);
        let args = ["-f", path.as_str(), "-U", "-V", VERBOSITY];
        relative(run_crate_within(&at, &args, mode.clone(), deadline), &at)
    });
    let settled = {
        let seen = distribution(&theirs);
        seen.len() == 1 && ours.iter().all(|run| agree(seen[0].0, run))
    };
    if !settled {
        let more: Vec<usize> = (ORACLE_RUNS..ORACLE_RUNS_UNSETTLED).collect();
        theirs.extend(at_once(&more, oracle_run));
    }
    let seen = distribution(&theirs);
    dump(file, part, "oracle", seen[0].0);
    dump(file, part, "normal", &ours[0]);
    dump(file, part, "every", &ours[1]);
    let mut cells = format!("{file}\t{}\t{}\t", part.label(), theirs.len());
    for (at, (run, count)) in seen.iter().enumerate() {
        if at > 0 {
            cells.push_str("; ");
        }
        cells.push_str(&format!("{}: {count} {}", at + 1, key(run)));
    }
    let mut started_cells = format!("{file}\t{}\tskipped [{}]", part.label(), refusing.join(" "));
    // Ruling P86: where the oracle's runs differ from each other only
    // in their assertion count, the count is not compared.
    let counted_only = counted_only(&seen.iter().map(|(run, _)| *run).collect::<Vec<_>>());
    let mut keys = [String::new(), String::new()];
    let mut agrees = [false; 2];
    let mut stuck = [false; 2];
    for (at, run) in ours.iter().enumerate() {
        stuck[at] = inverted(run) || run.status.is_none();
        keys[at] = key(run);
        let cell = match seen.iter().position(|(theirs, _)| agree(theirs, run)) {
            Some(number) => {
                agrees[at] = true;
                format!("agrees with {}", number + 1)
            }
            None if counted_only && agree(&count_masked(seen[0].0), &count_masked(run)) => {
                agrees[at] = true;
                format!("agrees but for the assertion count (P86): {}", keys[at])
            }
            None => format!("differs: {}; {}", keys[at], difference(seen[0].0, run)),
        };
        cells.push_str(&format!("\t{cell}"));
        let names = started(run);
        started_cells.push_str(&format!("\t{} [{}]", names.len(), names.join(" ")));
    }
    let same = if agree(&ours[0], &ours[1]) {
        "same"
    } else {
        "differ"
    };
    cells.push_str(&format!("\t{same}"));
    let normal = ours.swap_remove(0);
    let row = Row {
        part,
        cells,
        started: started_cells,
        keys,
        agrees,
        stuck,
    };
    (row, normal)
}

fn summary_run(assertions: u32, failures: u32, stderr: &str, status: i32) -> Run {
    Run {
                stdout: format!(
                    "Tests ran:          1\nAssertions:         {assertions}\nFailures:           {failures}\nErrors:             0\n"
                )
                .into_bytes(),
                stderr: stderr.as_bytes().to_vec(),
                status: Some(status),
            }
}

/// Each test a [`DIFFERING`] key names as failing or as its last
/// started test, as `(dir/group, TEST)`.
fn differing_tests() -> BTreeSet<(String, String)> {
    let mut tests = BTreeSet::new();
    for (group, _, _, key, _) in DIFFERING {
        let group = group
            .strip_suffix(".testGroup")
            .expect("a group file")
            .to_string();
        if let Some((_, rest)) = key.split_once("last started ") {
            let last = rest.split([',', ' ']).next().expect("a test");
            tests.insert((group.clone(), last.to_string()));
        }
        if let Some((_, rest)) = key.split_once("failing [") {
            let list = rest.split(']').next().expect("a closed list");
            for test in list.split_whitespace() {
                tests.insert((group.clone(), test.to_string()));
            }
        }
    }
    tests
}

#[test]
fn the_differing_tests_are_read_from_every_key() {
    let tests = differing_tests();
    for (group, test) in [
        ("base/bif/STREAM", "TEST_RELATIVE_FILE_EXISTS2"),
        ("base/bif/STREAM", "TEST_WRONG_TOO_MANY_ARGS_S"),
        ("base/keyword/TRACE", "TEST_TRACE_DROP"),
        ("base/class/RexxContext", "TESTRS01"),
    ] {
        assert!(
            tests.contains(&(group.to_string(), test.to_string())),
            "{group} {test}"
        );
    }
}

/// Spec 2026-10-10 section 2: every test a [`DIFFERING`] key names as
/// failing or last started passes in the per-test record of the suite
/// gate's last run of every group, at the recorded ootest revision, or
/// has a row in one of the three row files. Gate-only, under the suite
/// gate's variable, since it reads that record; a record of a
/// `REXX_SUITE_GROUPS` run or of another revision fails it.
#[test]
fn every_differing_test_is_fixed_or_has_a_row() {
    use super::super::suite_rows::{
        REVISION, TESTS_FILE, by_test, full_run_record, read_rows, table_path,
    };
    if std::env::var("REXX_SUITE_GATE").map_or(true, |value| value.is_empty() || value == "0") {
        return;
    }
    let record = table_path().with_file_name(TESTS_FILE);
    let text = read_lossy(&record);
    let revision = read_lossy(&super::super::group_runner::worktree().join(REVISION));
    if let Err(problem) = full_run_record(&text, revision.trim()) {
        panic!("{}: {problem}", record.display());
    }
    let passed: BTreeSet<(String, String)> = text
        .lines()
        .filter(|line| !line.starts_with('#'))
        .filter_map(|line| {
            let fields: Vec<&str> = line.split('\t').collect();
            (fields.len() == 5 && fields[4] == "passed")
                .then(|| (fields[0].to_string(), fields[1].to_string()))
        })
        .collect();
    let (rows, malformed) = read_rows();
    assert!(malformed.is_empty(), "{malformed:#?}");
    let rowed = by_test(&rows);
    let open: Vec<String> = differing_tests()
        .into_iter()
        .filter(|test| !passed.contains(test) && !rowed.contains_key(test))
        .map(|(group, test)| format!("{group} {test}"))
        .collect();
    assert!(
        open.is_empty(),
        "neither passing in {} nor rowed: {open:#?}",
        record.display()
    );
}

#[test]
fn a_run_agrees_but_for_the_count_only_when_the_rest_agrees() {
    let theirs = summary_run(12, 0, "", 0);
    assert!(agree(
        &count_masked(&theirs),
        &count_masked(&summary_run(10, 0, "", 0))
    ));
    assert!(!agree(
        &count_masked(&theirs),
        &count_masked(&summary_run(10, 0, "noise\n", 0))
    ));
    assert!(!agree(
        &count_masked(&theirs),
        &count_masked(&summary_run(10, 1, "", 1))
    ));
}

#[test]
fn one_oracle_outcome_is_not_counted_only() {
    assert!(!counted_only(&[&summary_run(12, 0, "", 0)]));
}

#[test]
fn outcomes_differing_only_in_the_count_are_counted_only() {
    assert!(counted_only(&[
        &summary_run(12, 0, "", 0),
        &summary_run(17, 0, "", 0)
    ]));
}

#[test]
fn outcomes_differing_in_stderr_alone_are_not_counted_only() {
    assert!(!counted_only(&[
        &summary_run(12, 0, "", 0),
        &summary_run(12, 0, "noise\n", 0)
    ]));
}

#[test]
fn outcomes_differing_in_count_and_status_are_not_counted_only() {
    assert!(!counted_only(&[
        &summary_run(12, 0, "", 0),
        &summary_run(17, 1, "", 1)
    ]));
}

#[test]
fn each_group_of_the_derived_list_in_one_run_in_both_modes() {
    if !gate_mode() {
        eprintln!("group_runs: skipped without {GATE_ENV}");
        return;
    }
    let (list, _) = super::super::derive(&super::super::worktree().join("ootest/ooRexx"));
    let derived: BTreeSet<String> = list
        .iter()
        .map(|row| {
            let group = row.group.trim_end_matches(".testGroup");
            let group = group.rsplit_once('/').map_or(group, |(_, name)| name);
            format!("{group}.{}", row.test)
        })
        .collect();
    let mut files: Vec<String> = list.iter().map(|row| row.group.clone()).collect();
    files.dedup();
    if let Ok(only) = std::env::var(ONLY_ENV) {
        let named: Vec<&str> = only.split(',').collect();
        for group in &named {
            assert!(
                files.iter().any(|file| file == group),
                "{ONLY_ENV}: {group:?} is no group file of the derived list"
            );
        }
        files.retain(|file| named.contains(&file.as_str()));
    }
    let parts: Vec<(&String, Part)> = files
        .iter()
        .flat_map(|file| [(file, Part::Whole), (file, Part::Derived)])
        .collect();
    let oracle = oracle::locate();
    let results = rows_in_parallel(
        "whole-groups",
        &parts,
        |(file, _)| {
            WALL_CLOCK
                .iter()
                .any(|row| row.starts_with(&format!("{file} ")))
        },
        |run, (file, part)| {
            let (_, group) = split(file);
            let mine: BTreeSet<String> = derived
                .iter()
                .filter_map(|row| row.strip_prefix(&format!("{group}.")))
                .map(str::to_string)
                .collect();
            rows_of(&oracle, run, file, *part, &mine)
        },
    );
    let mut table = String::from(
        "group\tpart\toracle runs\toracle outcomes\tnormal\tevery\tnormal against every\n",
    );
    let mut started = String::from("group\tpart\tskipped\tnormal started\tevery started\n");
    let mut failing = Vec::new();
    let mut listed_agreeing = Vec::new();
    for ((file, _), rows) in parts.iter().zip(&results) {
        for row in rows {
            table.push_str(&row.cells);
            table.push('\n');
            started.push_str(&row.started);
            started.push('\n');
            for (at, mode) in ["normal", "every"].into_iter().enumerate() {
                let name = format!("{file} {} {mode}", row.part.label());
                let listed = DIFFERING.iter().any(|(group, part, listed_mode, ..)| {
                    group == file && *part == row.part.label() && *listed_mode == mode
                });
                if listed && row.agrees[at] {
                    listed_agreeing.push(name.clone());
                }
                if let Err(why) = verdict(file, row, at) {
                    failing.push(format!("{name}: {why}"));
                }
            }
        }
    }
    table.push_str(&format!(
        "\nlisted as differing, agreeing: {listed_agreeing:?}\n"
    ));
    eprintln!("{table}\n{started}");
    if let Some(path) = std::env::var_os(TABLE_ENV) {
        std::fs::write(path, &table).expect("cannot write the table");
    }
    if let Some(path) = std::env::var_os(STARTED_ENV) {
        std::fs::write(path, &started).expect("cannot write the started table");
    }
    assert!(failing.is_empty(), "{failing:#?}");
}

/// The seeded gate (spec 2026-10-07 section 4, "Gate"): each part of
/// `corpus/sim-gate.tsv` in the simulation mode under the seeds its row
/// derives, judged by the crate's own invariants and, but under `pct`
/// and in the rows injecting failures, by the program's own checks
/// (ruling R6). An outcome no committed oracle outcome of its part has
/// goes to the report, never red.
mod sim_gate {
    use std::collections::{BTreeMap, BTreeSet};
    use std::fmt::Write as _;
    use std::path::{Path, PathBuf};
    use std::sync::Mutex;
    use std::time::Duration;

    use super::super::super::group_runner::{
        Ended, GATE_ENV, ProcessRun, Run, VERBOSITY, gate_mode, masked, outcome, run_crate_process,
        run_crate_sim, run_oracle_within,
    };
    use super::super::super::support::{oracle, sidecar};
    use super::{
        DIFFERING, ORACLE_DEADLINE, ORACLE_RUNS, ORACLE_RUNS_UNSETTLED, Part, WALL_CLOCK, at_once,
        copy, failing, key, left_out_of, refusal, relative, split, started,
    };

    /// The gate's parts, relative to `rust/`.
    const TABLE: &str = "corpus/sim-gate.tsv";

    /// The reds ruled not to be defects, relative to `rust/`.
    const EXEMPT: &str = "corpus/sim-exempt.tsv";

    /// The committed oracle outcome sets, one file per part, relative
    /// to `rust/`.
    const ORACLE_DIR: &str = "corpus/sim-oracle";

    /// The seed count of every row, in place of the table's.
    const SEEDS_ENV: &str = "REXX_SIM_SEEDS";

    /// The policy of every seed, in place of the mix; `pre:D` and
    /// `pct:D` take the row's k.
    const POLICY_ENV: &str = "REXX_SIM_POLICY";

    /// One run, `GROUP:PART:SEED:POLICY`, in place of the gate's.
    const ONLY_ENV: &str = "REXX_SIM_ONLY";

    /// Set to `1`, the oracle sets are run and grown.
    const REFRESH_ENV: &str = "REXX_SIM_ORACLE_REFRESH";

    /// Where the run's report is written; else below the target's
    /// temporary directory.
    const REPORT_ENV: &str = "REXX_SIM_REPORT";

    /// How many runs go at once; [`JOBS_DEFAULT`] where unset.
    const JOBS_ENV: &str = "REXX_SIM_JOBS";
    const JOBS_DEFAULT: usize = 8;

    /// Where the calibration writes its rows; the calibration runs
    /// only where it is set.
    const CALIBRATION_ENV: &str = "REXX_SIM_CALIBRATION";

    /// The policy of seed i is entry i mod 7.
    const MIX: [&str; 7] = [
        "pre:1",
        "pre:2",
        "pre:3",
        "pct:3",
        "uniform:0.01",
        "uniform:0.2",
        "uniform:1",
    ];

    /// A run's deadline: its part's calibrated time in this build
    /// times [`DEADLINE_FACTOR`], and at least [`DEADLINE_FLOOR`].
    const DEADLINE_FACTOR: u32 = 10;
    const DEADLINE_FLOOR: Duration = Duration::from_secs(60);

    /// The calibration's bound on one run.
    const CALIBRATION_DEADLINE: Duration = Duration::from_secs(600);

    /// How a gate program's assertion line starts.
    const ASSERTION: &str = "FAIL";

    /// The directory of the gate's own programs, relative to `rust/`.
    const PROGRAMS: &str = "crates/rexx-exec/tests/sim_gate/";

    /// The refusals naming a broken invariant of the scheduler.
    const INVARIANT: &str = "rexx-exec: the scheduler found ";

    /// The refusals of a determinism breach: a post from another
    /// thread, and a replay taking a decision its trace does not hold.
    const DETERMINISM: &[&str] = &[
        "from another thread in the simulation mode is not implemented",
        "rexx-exec: the replay of ",
    ];

    /// The design-limit refusals, red unless the part's oracle set
    /// holds a run that did not end.
    const DESIGN_LIMIT: &[&str] = &[
        "that only an activity pinned below it can end",
        "a REPLY its method body runs on a nested Rust frame",
        "a wait that nothing left to run can end",
        "a wait in the simulation mode that only a signal can end",
        "a command in the simulation mode that waits longer than its bound",
        "a native call in the simulation mode that runs longer than its bound",
    ];

    fn profile() -> &'static str {
        if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        }
    }

    fn rust_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    /// FNV-1a 64 over `bytes`.
    fn fnv(bytes: &[u8]) -> u64 {
        bytes.iter().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(0x0100_0000_01b3)
        })
    }

    /// Output `i` (from 0) of the splitmix64 stream starting at `state`.
    fn splitmix(state: u64, i: u64) -> u64 {
        let mut z = state.wrapping_add((i + 1).wrapping_mul(0x9e37_79b9_7f4a_7c15));
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    /// Seed `i` of a part: `splitmix(hash(group, part), i)`, the hash
    /// FNV-1a over `GROUP:PART`.
    fn seed(group: &str, part: &str, i: u64) -> u64 {
        splitmix(fnv(format!("{group}:{part}").as_bytes()), i)
    }

    /// `mix` with the part's `k` where the policy takes one.
    fn with_k(mix: &str, k: u64) -> String {
        if (mix.starts_with("pre:") || mix.starts_with("pct:")) && !mix.contains(",k=") {
            format!("{mix},k={}", k.max(1))
        } else {
            mix.to_string()
        }
    }

    /// One row of [`TABLE`].
    #[derive(Clone, Debug)]
    struct Row {
        /// An ooTest group file below `ootest/ooRexx`, or a program
        /// relative to `rust/`.
        group: String,
        /// `whole`, `derived`, `rest`, `single` (the one test `left_out`
        /// names, alone) or `program`.
        part: String,
        /// The contended steps of the calibration run.
        k: u64,
        /// Seeds in a release build and in a debug build; 0 leaves the
        /// row out of that build's gate.
        release_seeds: u64,
        debug_seeds: u64,
        /// The calibration run's wall time in each build.
        release_ms: u64,
        debug_ms: u64,
        /// Knobs appended to every run's mode.
        knobs: Option<String>,
        /// The tests left out beside the part's own; a single row's one
        /// test.
        left_out: Vec<String>,
    }

    impl Row {
        fn program(&self) -> bool {
            self.part == "program"
        }

        /// Whether the row's knobs inject failures or halts
        /// (`fail=wait:K`, `halt@K`): its program's own checks are not
        /// judged and its outcomes are not compared with the oracle.
        fn injects(&self) -> bool {
            self.knobs
                .as_deref()
                .is_some_and(|knobs| knobs.contains("fail=") || knobs.contains("halt@"))
        }

        fn seeds(&self) -> u64 {
            if let Ok(count) = std::env::var(SEEDS_ENV) {
                return count
                    .parse()
                    .unwrap_or_else(|_| panic!("{SEEDS_ENV} is not a count: {count:?}"));
            }
            if cfg!(debug_assertions) {
                self.debug_seeds
            } else {
                self.release_seeds
            }
        }

        fn deadline(&self) -> Duration {
            let ms = if cfg!(debug_assertions) {
                self.debug_ms
            } else {
                self.release_ms
            };
            (Duration::from_millis(ms) * DEADLINE_FACTOR).max(DEADLINE_FLOOR)
        }

        /// The policy of seed `i`.
        fn policy(&self, i: u64) -> String {
            let mix = std::env::var(POLICY_ENV)
                .unwrap_or_else(|_| MIX[usize::try_from(i % 7).expect("below 7")].to_string());
            with_k(&mix, self.k)
        }

        /// Whether the row's virtual time starts at the wall clock.
        fn real_clock(&self) -> bool {
            self.knobs
                .as_deref()
                .is_some_and(|knobs| knobs.split(',').any(|knob| knob == "clock=real"))
        }

        fn mode(&self, seed: u64, policy: &str) -> String {
            let mut mode = format!("sim:{seed},{policy}");
            if let Some(knobs) = &self.knobs {
                mode.push(',');
                mode.push_str(knobs);
            }
            mode
        }

        fn oracle_file(&self) -> PathBuf {
            rust_root().join(ORACLE_DIR).join(format!(
                "{}.{}.tsv",
                self.group.replace('/', "__"),
                self.part
            ))
        }
    }

    fn read_table() -> Vec<Row> {
        let path = rust_root().join(TABLE);
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
        text.lines()
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(|line| {
                let cells: Vec<&str> = line.split('\t').collect();
                assert_eq!(cells.len(), 9, "{TABLE}: {line:?} is not nine cells");
                let number = |at: usize| -> u64 {
                    cells[at]
                        .parse()
                        .unwrap_or_else(|_| panic!("{TABLE}: {line:?} cell {at}"))
                };
                Row {
                    group: cells[0].to_string(),
                    part: cells[1].to_string(),
                    k: number(2),
                    release_seeds: number(3),
                    debug_seeds: number(4),
                    release_ms: number(5),
                    debug_ms: number(6),
                    knobs: (cells[7] != "-").then(|| cells[7].to_string()),
                    left_out: if cells[8] == "-" {
                        Vec::new()
                    } else {
                        cells[8].split(' ').map(str::to_string).collect()
                    },
                }
            })
            .collect()
    }

    /// One run of the gate.
    #[derive(Clone)]
    struct Unit {
        row: Row,
        seed: u64,
        policy: String,
    }

    impl Unit {
        fn mode(&self) -> String {
            self.row.mode(self.seed, &self.policy)
        }

        fn name(&self) -> String {
            format!(
                "{}:{}:{}:{}",
                self.row.group, self.row.part, self.seed, self.policy
            )
        }

        /// The command that runs this unit alone.
        fn replay(&self) -> String {
            let profile = if cfg!(debug_assertions) {
                ("", "debug")
            } else {
                (" --release", "release")
            };
            format!(
                "REXX_CORPUS_GATE=1 {ONLY_ENV}='{}' memcap 8G cargo test -j 4{} -p \
                         rexx-exec --test concurrency_tests the_seeded_gate -- --nocapture \
                         (profile={}, stack={}, REXX_SWITCH_MODE={}, scratch {})",
                self.name(),
                profile.0,
                profile.1,
                rexx_exec::INTERPRETER_STACK_BYTES,
                self.mode(),
                unit_dir("sim-gate", self).display()
            )
        }
    }

    /// The gate's units: each row's seeds, or where [`ONLY_ENV`] is
    /// set, `GROUP` or `GROUP:PART` its rows' seeds and
    /// `GROUP:PART:SEED:POLICY` that one run.
    fn units(rows: &[Row]) -> Vec<Unit> {
        let only = std::env::var(ONLY_ENV).ok();
        let cells: Vec<&str> = only
            .as_deref()
            .map_or_else(Vec::new, |only| only.splitn(4, ':').collect());
        let chosen = |row: &&Row| match cells[..] {
            [] => true,
            [group] => row.group == group,
            [group, part, ..] => row.group == group && row.part == part,
        };
        if let [_, _, seed, policy] = cells[..] {
            let row = rows
                .iter()
                .find(chosen)
                .unwrap_or_else(|| panic!("{ONLY_ENV}: no row {only:?}"));
            return vec![Unit {
                row: row.clone(),
                seed: seed
                    .parse()
                    .unwrap_or_else(|_| panic!("{ONLY_ENV}: {seed:?} is not a seed")),
                policy: policy.to_string(),
            }];
        }
        assert!(
            cells.len() < 3,
            "{ONLY_ENV} is GROUP, GROUP:PART or GROUP:PART:SEED:POLICY, not {only:?}"
        );
        rows.iter()
            .filter(chosen)
            .flat_map(|row| {
                (0..row.seeds()).map(move |i| Unit {
                    row: row.clone(),
                    seed: seed(&row.group, &row.part, i),
                    policy: row.policy(i),
                })
            })
            .collect()
    }

    /// The derived list's tests of each group file.
    fn derived_tests() -> BTreeMap<String, BTreeSet<String>> {
        let (list, _) =
            super::super::super::derive(&super::super::super::worktree().join("ootest/ooRexx"));
        let mut tests: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for row in list {
            tests.entry(row.group).or_default().insert(row.test);
        }
        tests
    }

    /// The `GROUP.TEST`s a run of `row` renames out.
    fn left_out(row: &Row, derived: &BTreeMap<String, BTreeSet<String>>) -> BTreeSet<String> {
        let (_, group) = split(&row.group);
        let part = match row.part.as_str() {
            "derived" => Part::Derived,
            "rest" => Part::Rest,
            _ => Part::Whole,
        };
        let mine = derived.get(&row.group).cloned().unwrap_or_default();
        let mut left_out = left_out_of(&row.group, part, &mine);
        if row.part != "single" {
            left_out.extend(row.left_out.iter().map(|test| format!("{group}.{test}")));
        }
        left_out
    }

    /// The driver's arguments for the group file `path`: a single
    /// row's run names its one test.
    fn arguments(row: &Row, path: &str) -> Vec<String> {
        let mut args: Vec<String> = ["-f", path, "-U", "-V", VERBOSITY]
            .map(str::to_string)
            .into();
        if row.part == "single" {
            args.push("-t".to_string());
            args.extend(row.left_out.iter().cloned());
        }
        args
    }

    /// What one run of a program part reads beside its text: its
    /// path, its run directory (laid out afresh) and its sidecar.
    fn program_run(
        row: &Row,
        scratch: &Path,
    ) -> (PathBuf, PathBuf, sidecar::Sidecar, Vec<(String, String)>) {
        let path = rust_root()
            .join(&row.group)
            .canonicalize()
            .unwrap_or_else(|e| panic!("{}: {e}", row.group));
        let side = row
            .group
            .strip_prefix("corpus/")
            .map(|rel| sidecar::sidecar_for(&rust_root().join("corpus"), rel))
            .unwrap_or_default();
        let dir = scratch.join("p");
        let overrides = sidecar::resolved_environment(&side, &dir);
        let cwd = sidecar::prepare_run_directory(&dir, &side);
        (path, cwd, side, overrides)
    }

    /// One run here of `row` under `mode`, its copy below `scratch`.
    fn run_here(
        row: &Row,
        derived: &BTreeMap<String, BTreeSet<String>>,
        scratch: &Path,
        mode: &str,
        deadline: Duration,
    ) -> ProcessRun {
        if row.program() {
            let (path, cwd, side, overrides) = program_run(row, scratch);
            let mut environment: Vec<(String, String)> = std::env::vars()
                .filter(|(name, _)| {
                    name != "REXX_SWITCH_MODE"
                        && !name.starts_with("REXX_SIM_")
                        && !overrides.iter().any(|(over, _)| over == name)
                })
                .collect();
            environment.extend(overrides);
            let mut environment: Vec<(Vec<u8>, Vec<u8>)> = environment
                .into_iter()
                .map(|(name, value)| (name.into_bytes(), value.into_bytes()))
                .collect();
            environment.sort();
            return run_crate_process(
                &[path.to_string_lossy().into_owned()],
                &cwd,
                Some(environment),
                mode,
                side.stdin.as_deref(),
                deadline,
            );
        }
        let (dir, group) = split(&row.group);
        let (at, path) = copy(scratch, "s00", dir, group, &left_out(row, derived));
        let args = arguments(row, &path);
        let args: Vec<&str> = args.iter().map(String::as_str).collect();
        let mut ran = run_crate_sim(&at, &args, mode, deadline);
        ran.run = relative(ran.run, &at);
        ran
    }

    /// One oracle run of `row`, its copy below `scratch`.
    fn run_oracle(
        oracle: &oracle::Oracle,
        row: &Row,
        derived: &BTreeMap<String, BTreeSet<String>>,
        scratch: &Path,
    ) -> Run {
        if row.program() {
            let (path, cwd, side, overrides) = program_run(row, scratch);
            let borrowed: Vec<(&str, &str)> = overrides
                .iter()
                .map(|(name, value)| (name.as_str(), value.as_str()))
                .collect();
            let theirs = oracle.run_in_with(&path, &cwd, &borrowed, side.stdin.as_deref());
            let status = (!oracle::did_not_finish(&theirs)).then(|| theirs.expect_exit_code());
            return Run {
                stdout: theirs.stdout,
                stderr: theirs.stderr,
                status,
            };
        }
        let (dir, group) = split(&row.group);
        let (at, path) = copy(scratch, "o00", dir, group, &left_out(row, derived));
        let args = arguments(row, &path);
        let args: Vec<&str> = args.iter().map(String::as_str).collect();
        relative(run_oracle_within(oracle, &at, &args, ORACLE_DEADLINE), &at)
    }

    /// `stderr` without the `rexx-sim: ` lines, a child's included.
    fn sim_masked(stderr: &[u8]) -> Vec<u8> {
        let text = String::from_utf8_lossy(stderr);
        text.split_inclusive('\n')
            .filter(|line| !line.starts_with("rexx-sim: "))
            .collect::<String>()
            .into_bytes()
    }

    /// The run's own `rexx-sim: ` line, its last line.
    fn sim_line(stderr: &[u8]) -> Option<String> {
        String::from_utf8_lossy(stderr)
            .lines()
            .last()
            .filter(|line| line.starts_with("rexx-sim: "))
            .map(str::to_string)
    }

    /// The value of `name=` in a `rexx-sim: ` line.
    fn sim_field<'a>(line: &'a str, name: &str) -> Option<&'a str> {
        line.split(' ')
            .find_map(|word| word.strip_prefix(name)?.strip_prefix('='))
    }

    /// `run` as compared: stdout masked, `rexx-sim: ` lines dropped.
    fn compared(run: &Run) -> Run {
        Run {
            stdout: masked(&run.stdout),
            stderr: sim_masked(&run.stderr),
            status: run.status,
        }
    }

    /// The digest of a compared run's three descriptors.
    fn digest(run: &Run) -> u64 {
        let mut bytes = run.stdout.clone();
        bytes.push(0xff);
        bytes.extend_from_slice(&run.stderr);
        bytes.push(0xff);
        bytes.extend_from_slice(
            run.status
                .map_or_else(|| "none".to_string(), |status| status.to_string())
                .as_bytes(),
        );
        fnv(&bytes)
    }

    /// A compared run's key: [`key`]'s for a group part, its status
    /// and line counts for a program.
    fn outcome_key(row: &Row, run: &Run) -> String {
        if !row.program() {
            return key(run);
        }
        let lines = |bytes: &[u8]| bytes.iter().filter(|&&b| b == b'\n').count();
        format!(
            "rc {}, stdout {} lines, stderr {} lines{}",
            run.status
                .map_or_else(|| "none".to_string(), |status| status.to_string()),
            lines(&run.stdout),
            lines(&run.stderr),
            refusal(run)
                .map(|line| format!(", {line}"))
                .unwrap_or_default()
        )
    }

    /// One outcome of a part's oracle set.
    #[derive(Clone, Debug)]
    struct Seen {
        count: usize,
        status: String,
        digest: u64,
        key: String,
    }

    fn read_oracle(row: &Row) -> Option<Vec<Seen>> {
        let text = std::fs::read_to_string(row.oracle_file()).ok()?;
        Some(
            text.lines()
                .filter(|line| !line.is_empty() && !line.starts_with('#'))
                .map(|line| {
                    let cells: Vec<&str> = line.splitn(4, '\t').collect();
                    let [count, status, digest, key] = cells[..] else {
                        panic!("{}: {line:?}", row.oracle_file().display());
                    };
                    Seen {
                        count: count.parse().expect("a count"),
                        status: status.to_string(),
                        digest: u64::from_str_radix(digest, 16).expect("a digest"),
                        key: key.to_string(),
                    }
                })
                .collect(),
        )
    }

    fn write_oracle(row: &Row, seen: &[Seen]) {
        let runs: usize = seen.iter().map(|seen| seen.count).sum();
        let mut text = format!(
            "# The oracle's outcomes of {} {}, {runs} runs, written under \
                     {REFRESH_ENV}=1.\n# count, status, digest of the compared \
                     descriptors, key.\n",
            row.group, row.part
        );
        for one in seen {
            let _ = writeln!(
                text,
                "{}\t{}\t{:016x}\t{}",
                one.count, one.status, one.digest, one.key
            );
        }
        let path = row.oracle_file();
        std::fs::create_dir_all(path.parent().expect("a parent")).expect("the oracle directory");
        std::fs::write(&path, text).expect("cannot write an oracle set");
    }

    /// The tests a key names as failing.
    fn failing_of(key: &str) -> BTreeSet<String> {
        key.rsplit_once("failing [")
            .and_then(|(_, rest)| rest.split_once(']'))
            .map(|(names, _)| {
                names
                    .split(' ')
                    .filter(|name| !name.is_empty())
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default()
    }

    /// What a red is.
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Kind {
        Panic,
        Crash,
        Hang,
        Invariant,
        Determinism,
        DesignLimit,
        Check,
        Abort,
        NoReport,
    }

    impl Kind {
        fn label(self) -> &'static str {
            match self {
                Kind::Panic => "panic",
                Kind::Crash => "crash",
                Kind::Hang => "hang",
                Kind::Invariant => "invariant",
                Kind::Determinism => "determinism",
                Kind::DesignLimit => "design-limit",
                Kind::Check => "check",
                Kind::Abort => "abort",
                Kind::NoReport => "no-report",
            }
        }
    }

    /// One red: its kind, the test it is in (`-` for a program) and
    /// the line that shows it.
    #[derive(Clone, Debug)]
    struct Red {
        kind: Kind,
        test: String,
        line: String,
    }

    /// The reds of one run (spec section 4, "Gate"; ruling R6).
    fn judge(row: &Row, policy: &str, ran: &ProcessRun, seen: Option<&[Seen]>) -> Vec<Red> {
        let run = &ran.run;
        let text = String::from_utf8_lossy(&run.stderr).into_owned();
        let test = if row.program() {
            "-".to_string()
        } else {
            started(run).pop().unwrap_or_else(|| "-".to_string())
        };
        let red = |kind, line: &str| {
            vec![Red {
                kind,
                test: test.clone(),
                line: line.to_string(),
            }]
        };
        match ran.ended {
            Ended::Killed => {
                return red(
                    Kind::Hang,
                    &format!("killed at the deadline, {} s", row.deadline().as_secs()),
                );
            }
            Ended::Signaled(signal) => {
                return red(Kind::Crash, &format!("ended by signal {signal}"));
            }
            Ended::Exited(_) => {}
        }
        if let Some(line) = text.lines().find(|line| line.contains("panicked at")) {
            return red(Kind::Panic, line);
        }
        if run.status == Some(101) {
            return red(Kind::Panic, "rc 101");
        }
        if let Some(line) = refusal(run) {
            if line.starts_with(INVARIANT) {
                return red(Kind::Invariant, &line);
            }
            if DETERMINISM.iter().any(|text| line.contains(text)) {
                return red(Kind::Determinism, &line);
            }
            let twin_hangs = seen.is_some_and(|seen| seen.iter().any(|one| one.status == "none"));
            if DESIGN_LIMIT.iter().any(|text| line.contains(text)) {
                if twin_hangs {
                    return Vec::new();
                }
                return red(Kind::DesignLimit, &line);
            }
        }
        if sim_line(&run.stderr).is_none() {
            return red(Kind::NoReport, "no rexx-sim line ends stderr");
        }
        let differing: Vec<&str> = differing_keys(row).collect();
        if !row.injects()
            && let Some(line) = early_end(row, policy, run, seen.unwrap_or_default(), &differing)
        {
            return red(Kind::Abort, &line);
        }
        if row.injects() || policy.starts_with("pct:") {
            return Vec::new();
        }
        if row.program() {
            return String::from_utf8_lossy(&run.stdout)
                .lines()
                .filter(|line| line.starts_with(ASSERTION))
                .map(|line| Red {
                    kind: Kind::Check,
                    test: test.clone(),
                    line: line.to_string(),
                })
                .collect();
        }
        let mut known: BTreeSet<String> = seen
            .unwrap_or_default()
            .iter()
            .flat_map(|one| failing_of(&one.key))
            .collect();
        known.extend(failing_unswitched(row));
        failing(run)
            .into_iter()
            .filter(|name| !known.contains(name))
            .map(|name| Red {
                kind: Kind::Check,
                line: format!("{name} fails; no oracle outcome of the part fails it"),
                test: name,
            })
            .collect()
    }

    /// The keys [`DIFFERING`] lists for `row`'s part in the shipped
    /// scheduler's mode.
    fn differing_keys(row: &Row) -> impl Iterator<Item = &'static str> {
        DIFFERING
            .iter()
            .filter(|(group, part, mode, ..)| {
                *group == row.group && *part == row.part && *mode == "normal"
            })
            .map(|(.., key, _)| *key)
    }

    /// Why a run ended where none of the part's oracle outcomes and
    /// [`DIFFERING`] keys `differing` does: a refusal none of them
    /// names, or, but under `pct`, no ooTest summary, or a status none
    /// of them has in a run naming no failing test.
    fn early_end(
        row: &Row,
        policy: &str,
        run: &Run,
        seen: &[Seen],
        differing: &[&str],
    ) -> Option<String> {
        let mut keys: Vec<&str> = differing.to_vec();
        keys.extend(seen.iter().map(|one| one.key.as_str()));
        let status = run
            .status
            .map_or_else(|| "none".to_string(), |status| status.to_string());
        if let Some(line) = refusal(run) {
            return (!keys.iter().any(|key| refusal_listed(key, &line, &status))).then(|| {
                format!(
                    "{line}, rc {status}: no oracle outcome or DIFFERING key of the part \
                             ends so"
                )
            });
        }
        if policy.starts_with("pct:") {
            return None;
        }
        if !row.program() && outcome(&run.stdout).starts_with("no summary") {
            return Some("no ooTest summary".to_string());
        }
        let listed = seen.iter().any(|one| one.status == status)
            || differing
                .iter()
                .any(|key| key.contains(&format!(", rc {status},")));
        (!listed && (row.program() || failing(run).is_empty())).then(|| {
            format!("rc {status}, which no oracle outcome or DIFFERING key of the part has")
        })
    }

    /// Whether `key` is an outcome ending in the refusal `line` at
    /// `status`: a group part's key starts with the refusal and then
    /// its status, a program's key starts with its status and ends
    /// with the refusal.
    fn refusal_listed(key: &str, line: &str, status: &str) -> bool {
        key.strip_prefix(line)
            .is_some_and(|rest| rest.starts_with(&format!(", rc {status},")))
            || (key.starts_with(&format!("rc {status},"))
                && key
                    .strip_suffix(line)
                    .is_some_and(|head| head.ends_with(", ")))
    }

    /// The reds of a run whose first run's WALL_CLOCK checks on the
    /// tests `first` were rerun, given the rerun's `reds`: outside
    /// `clock=real` a seed and mode fix the run, so a rerun that
    /// passes shows the first run was not fixed.
    fn after_rerun(row: &Row, first: String, mut reds: Vec<Red>) -> Vec<Red> {
        if reds.is_empty() && !row.real_clock() {
            reds.push(Red {
                kind: Kind::Determinism,
                line: format!("{first} failed, and the rerun of the same seed and mode passed"),
                test: first,
            });
        }
        reds
    }

    /// The tests [`DIFFERING`] lists as failing in `row`'s part in the
    /// shipped scheduler's mode, where its outcome is not the oracle's.
    fn failing_unswitched(row: &Row) -> BTreeSet<String> {
        differing_keys(row).flat_map(failing_of).collect()
    }

    /// One row of [`EXEMPT`]: group, part, test, kind and a text the
    /// red's line holds (its refusal where it has one), each `*` for
    /// any, then the reason and its evidence.
    struct Exempt {
        cells: [String; 5],
    }

    fn read_exempt() -> Vec<Exempt> {
        let path = rust_root().join(EXEMPT);
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
        text.lines()
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(|line| {
                let cells: Vec<&str> = line.split('\t').collect();
                assert_eq!(cells.len(), 7, "{EXEMPT}: {line:?} is not seven cells");
                assert!(
                    cells[5..].iter().all(|cell| !cell.trim().is_empty()),
                    "{EXEMPT}: {line:?} has no reason or no evidence"
                );
                Exempt {
                    cells: [0, 1, 2, 3, 4].map(|at| cells[at].to_string()),
                }
            })
            .collect()
    }

    fn exempted(exempt: &[Exempt], row: &Row, red: &Red) -> bool {
        let wanted = [
            row.group.as_str(),
            row.part.as_str(),
            red.test.as_str(),
            red.kind.label(),
        ];
        exempt.iter().any(|one| {
            one.cells[..4]
                .iter()
                .zip(wanted)
                .all(|(cell, wanted)| cell == "*" || cell == wanted)
                && (one.cells[4] == "*" || red.line.contains(one.cells[4].as_str()))
        })
    }

    /// Runs `body` over `items` on [`JOBS_ENV`] threads, answering the
    /// results in order.
    fn in_jobs<I: Sync, T: Send>(items: &[I], body: impl Fn(usize, &I) -> T + Sync) -> Vec<T> {
        let jobs = std::env::var(JOBS_ENV).map_or(JOBS_DEFAULT, |value| {
            value
                .parse()
                .unwrap_or_else(|_| panic!("{JOBS_ENV} is not a count: {value:?}"))
        });
        let next = Mutex::new(0_usize);
        let results: Mutex<Vec<Option<T>>> = Mutex::new((0..items.len()).map(|_| None).collect());
        std::thread::scope(|scope| {
            for _ in 0..jobs.max(1) {
                scope.spawn(|| {
                    loop {
                        let at = {
                            let mut next = next.lock().expect("the work index");
                            let at = *next;
                            *next += 1;
                            at
                        };
                        if at >= items.len() {
                            break;
                        }
                        let result = body(at, &items[at]);
                        results.lock().expect("the results")[at] = Some(result);
                    }
                });
            }
        });
        results
            .into_inner()
            .expect("the results")
            .into_iter()
            .map(|result| result.expect("every item ran"))
            .collect()
    }

    /// A fresh scratch directory for item `at` of the run `name`.
    fn scratch(name: &str, at: usize) -> PathBuf {
        PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("{name}-{}", std::process::id()))
            .join(at.to_string())
            .join("run")
    }

    /// The scratch directory of `unit` below the run `name` and this
    /// build's profile, named by the unit alone: a program may read its
    /// own path, so a replay from the same target directory runs where
    /// the gate ran.
    fn unit_dir(name: &str, unit: &Unit) -> PathBuf {
        PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("{name}-{}", profile()))
            .join(format!("{:016x}", fnv(unit.name().as_bytes())))
            .join("run")
    }

    /// Removes the item directory a [`scratch`] directory `dir` is in.
    fn remove(dir: &Path) {
        let item = dir.parent().expect("an item directory");
        if item.exists() {
            std::fs::remove_dir_all(item).expect("cannot remove a run");
        }
        // The shared run directory stays: removing it races another job's create_dir_all.
    }

    /// What one unit found.
    struct Found {
        key: String,
        sim: String,
        took: Duration,
        reds: Vec<Red>,
        /// `None` where not compared, else the oracle outcome it
        /// matches, by number.
        compared: Option<Option<usize>>,
        /// The rerun of a run whose reds were all checks in a test
        /// whose outcome depends on the wall clock (ruling P48), and
        /// what it found.
        rerun: Option<String>,
    }

    /// Whether every red of `reds` is a failed check in a test
    /// `WALL_CLOCK` lists for `row`'s group.
    fn wall_clock_checks(row: &Row, reds: &[Red]) -> bool {
        !reds.is_empty()
            && reds.iter().all(|red| {
                red.kind == Kind::Check
                    && WALL_CLOCK.contains(&format!("{} {}", row.group, red.test).as_str())
            })
    }

    fn run_unit(unit: &Unit, derived: &BTreeMap<String, BTreeSet<String>>, dir: &Path) -> Found {
        let mut ran = run_here(&unit.row, derived, dir, &unit.mode(), unit.row.deadline());
        remove(dir);
        let seen = read_oracle(&unit.row);
        let mut reds = judge(&unit.row, &unit.policy, &ran, seen.as_deref());
        let mut rerun = None;
        if wall_clock_checks(&unit.row, &reds) {
            let first: Vec<&str> = reds.iter().map(|red| red.test.as_str()).collect();
            let first = first.join(" ");
            ran = run_here(&unit.row, derived, dir, &unit.mode(), unit.row.deadline());
            remove(dir);
            reds = judge(&unit.row, &unit.policy, &ran, seen.as_deref());
            rerun = Some(format!(
                "{}: failed {first}, rerun {}",
                unit.name(),
                if reds.is_empty() {
                    "passed".to_string()
                } else {
                    format!("failed {}", outcome_key(&unit.row, &compared(&ran.run)))
                }
            ));
            reds = after_rerun(&unit.row, first, reds);
        }
        let mine = compared(&ran.run);
        let compared = (!unit.row.injects()).then(|| {
            let digest = digest(&mine);
            seen.as_deref()
                .unwrap_or_default()
                .iter()
                .position(|one| one.digest == digest)
        });
        Found {
            key: outcome_key(&unit.row, &mine),
            sim: sim_line(&ran.run.stderr).unwrap_or_else(|| "-".to_string()),
            took: ran.took,
            reds,
            compared,
            rerun,
        }
    }

    fn report_path() -> PathBuf {
        std::env::var_os(REPORT_ENV).map_or_else(
            || PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("sim-gate-report.txt"),
            PathBuf::from,
        )
    }

    #[test]
    fn the_seeded_gate() {
        if !gate_mode() {
            eprintln!("sim_gate: skipped without {GATE_ENV}");
            return;
        }
        let rows = read_table();
        let exempt = read_exempt();
        let derived = derived_tests();
        let units = units(&rows);
        assert!(!units.is_empty(), "the gate has no run in this build");
        let start = std::time::Instant::now();
        let found = in_jobs(&units, |_, unit| {
            run_unit(unit, &derived, &unit_dir("sim-gate", unit))
        });
        let wall = start.elapsed();
        let mut report = format!(
            "# sim gate: {} runs in {} s, profile {}, stack {}\n\
                     # group\tpart\tseed\tpolicy\tms\toracle\tkey\trexx-sim\n",
            units.len(),
            wall.as_secs(),
            if cfg!(debug_assertions) {
                "debug"
            } else {
                "release"
            },
            rexx_exec::INTERPRETER_STACK_BYTES
        );
        let mut differences = String::new();
        let reruns: Vec<&str> = found
            .iter()
            .filter_map(|one| one.rerun.as_deref())
            .collect();
        let mut reds = Vec::new();
        let mut exempted_reds = Vec::new();
        for (unit, found) in units.iter().zip(&found) {
            let oracle = match found.compared {
                None => "not compared".to_string(),
                Some(Some(at)) => format!("agrees with {}", at + 1),
                Some(None) => {
                    let theirs: Vec<String> = read_oracle(&unit.row)
                        .unwrap_or_default()
                        .iter()
                        .map(|one| format!("{} x {}", one.count, one.key))
                        .collect();
                    let _ = writeln!(
                        differences,
                        "{}: ours {}; oracle [{}]",
                        unit.name(),
                        found.key,
                        theirs.join("; ")
                    );
                    "differs".to_string()
                }
            };
            let _ = writeln!(
                report,
                "{}\t{}\t{}\t{}\t{}\t{oracle}\t{}\t{}",
                unit.row.group,
                unit.row.part,
                unit.seed,
                unit.policy,
                found.took.as_millis(),
                found.key,
                found.sim
            );
            for red in &found.reds {
                let line = format!(
                    "{} {} in {}: {}\n  replay: {}",
                    red.kind.label(),
                    unit.name(),
                    red.test,
                    red.line,
                    unit.replay()
                );
                if exempted(&exempt, &unit.row, red) {
                    exempted_reds.push(line);
                } else {
                    reds.push(line);
                }
            }
        }
        let _ = write!(
            report,
            "\n## Differences from the oracle sets\n{differences}\n## Wall-clock reruns \
                     (ruling P48)\n{}\n\n## Reds\n{}\n\n## Exempted reds\n{}\n",
            reruns.join("\n"),
            reds.join("\n"),
            exempted_reds.join("\n")
        );
        let path = report_path();
        std::fs::write(&path, &report).expect("cannot write the report");
        eprintln!(
            "sim gate: {} runs, {} s, {} reds, {} exempted, {} differences; report {}",
            units.len(),
            wall.as_secs(),
            reds.len(),
            exempted_reds.len(),
            differences.lines().count(),
            path.display()
        );
        assert!(reds.is_empty(), "{}", reds.join("\n"));
    }

    /// Step 4's self-test: each row with debug seeds, seed 0 under
    /// `pre:2`, run twice, each in a process of its own, gives one
    /// trace hash and one outcome.
    #[test]
    fn a_seeded_run_repeats_in_a_fresh_process() {
        if !gate_mode() {
            eprintln!("sim_gate: skipped without {GATE_ENV}");
            return;
        }
        let derived = derived_tests();
        let sample: Vec<Unit> = self_test_rows(read_table())
            .into_iter()
            .map(|row| Unit {
                seed: seed(&row.group, &row.part, 0),
                policy: with_k("pre:2", row.k),
                row,
            })
            .collect();
        assert!(!sample.is_empty(), "no row has debug seeds");
        let pairs = in_jobs(&sample, |_, unit| {
            [0, 1].map(|_| {
                let dir = unit_dir("sim-self", unit);
                let ran = run_here(&unit.row, &derived, &dir, &unit.mode(), unit.row.deadline());
                remove(&dir);
                ran
            })
        });
        let mut broken = Vec::new();
        for (unit, [first, second]) in sample.iter().zip(&pairs) {
            let hash = |ran: &ProcessRun| {
                sim_line(&ran.run.stderr)
                    .and_then(|line| sim_field(&line, "trace").map(str::to_string))
            };
            let same = hash(first).is_some()
                && hash(first) == hash(second)
                && compared(&first.run).stdout == compared(&second.run).stdout
                && compared(&first.run).stderr == compared(&second.run).stderr
                && first.run.status == second.run.status;
            eprintln!(
                "self-test {}: trace {:?} {:?}, {}",
                unit.name(),
                hash(first),
                hash(second),
                if same { "same" } else { "DIFFERENT" }
            );
            if !same {
                broken.push(format!(
                    "{}: {} / {}\n  replay: {}",
                    unit.name(),
                    outcome_key(&unit.row, &compared(&first.run)),
                    outcome_key(&unit.row, &compared(&second.run)),
                    unit.replay()
                ));
            }
        }
        assert!(broken.is_empty(), "{}", broken.join("\n"));
    }

    /// The rows the determinism self-test runs: those with debug seeds
    /// whose virtual time does not start at the wall clock.
    fn self_test_rows(rows: Vec<Row>) -> Vec<Row> {
        rows.into_iter()
            .filter(|row| row.debug_seeds > 0)
            // Ruling R6: a clock=real row reads the wall clock, so two processes need not agree.
            .filter(|row| !row.real_clock())
            .collect()
    }

    /// The self-test leaves out every `clock=real` row of the committed
    /// table, and the table has one it would otherwise pick.
    #[test]
    fn the_self_test_runs_only_virtual_clock_rows() {
        let rows = read_table();
        assert!(
            rows.iter()
                .any(|row| row.debug_seeds > 0 && row.real_clock()),
            "no clock=real row has debug seeds, so the filter is not exercised"
        );
        let picked = self_test_rows(rows);
        assert!(!picked.is_empty(), "the self-test picks no row");
        for row in &picked {
            assert!(
                !row.real_clock(),
                "{} {} reads the wall clock",
                row.group,
                row.part
            );
        }
    }

    /// Under [`REFRESH_ENV`], runs each row's oracle 5 times, 30 where
    /// its outcomes vary or none is the outcome of seed 0 here under
    /// `fifo`, and adds what it saw to the row's committed set.
    #[test]
    fn the_oracle_sets_grow_only_under_refresh() {
        if !gate_mode() || std::env::var(REFRESH_ENV).as_deref() != Ok("1") {
            eprintln!("sim_gate: oracle sets not refreshed without {REFRESH_ENV}=1");
            return;
        }
        let oracle = oracle::locate();
        let derived = derived_tests();
        let rows: Vec<Row> = read_table()
            .into_iter()
            .filter(|row| !row.injects())
            .filter(|row| {
                std::env::var(ONLY_ENV).map_or(true, |only| {
                    let cells: Vec<&str> = only.splitn(3, ':').collect();
                    cells[0] == row.group && cells.get(1).is_none_or(|part| *part == row.part)
                })
            })
            .collect();
        in_jobs(&rows, |at, row| {
            let dir = scratch("sim-oracle", at);
            let oracle_runs = |runs: std::ops::Range<usize>| {
                at_once(&runs.collect::<Vec<_>>(), |k| {
                    let run = run_oracle(&oracle, row, &derived, &dir.join(format!("o{k}")));
                    compared(&run)
                })
            };
            let mut runs = oracle_runs(0..ORACLE_RUNS);
            let mine = compared(
                &run_here(
                    row,
                    &derived,
                    &dir.join("here"),
                    &row.mode(seed(&row.group, &row.part, 0), "fifo"),
                    CALIBRATION_DEADLINE,
                )
                .run,
            );
            let digests: BTreeSet<u64> = runs.iter().map(digest).collect();
            if digests.len() > 1 || !digests.contains(&digest(&mine)) {
                runs.extend(oracle_runs(ORACLE_RUNS..ORACLE_RUNS_UNSETTLED));
            }
            remove(&dir);
            let mut seen = read_oracle(row).unwrap_or_default();
            for run in &runs {
                let digest = digest(run);
                match seen.iter_mut().find(|one| one.digest == digest) {
                    Some(one) => one.count += 1,
                    None => seen.push(Seen {
                        count: 1,
                        status: run
                            .status
                            .map_or_else(|| "none".to_string(), |status| status.to_string()),
                        digest,
                        key: outcome_key(row, run),
                    }),
                }
            }
            seen.sort_by_key(|one| std::cmp::Reverse(one.count));
            write_oracle(row, &seen);
            eprintln!(
                "oracle set {} {}: {} outcomes",
                row.group,
                row.part,
                seen.len()
            );
        });
    }

    mod calibration;

    #[test]
    fn a_seed_is_derived_by_rule_and_the_mix_repeats_every_seven() {
        assert_eq!(fnv(b""), 0xcbf2_9ce4_8422_2325);
        assert_eq!(splitmix(0, 0), 0xe220_a839_7b1d_cdaf);
        let a = seed("base/class/Alarm.testGroup", "whole", 0);
        assert_eq!(a, seed("base/class/Alarm.testGroup", "whole", 0));
        assert_ne!(a, seed("base/class/Alarm.testGroup", "whole", 1));
        assert_ne!(a, seed("base/class/Alarm.testGroup", "derived", 0));
        let row = Row {
            group: "g".to_string(),
            part: "whole".to_string(),
            k: 40,
            release_seeds: 7,
            debug_seeds: 0,
            release_ms: 0,
            debug_ms: 0,
            knobs: None,
            left_out: Vec::new(),
        };
        if std::env::var(POLICY_ENV).is_err() {
            let policies: Vec<String> = (0..8).map(|i| row.policy(i)).collect();
            assert_eq!(
                policies,
                [
                    "pre:1,k=40",
                    "pre:2,k=40",
                    "pre:3,k=40",
                    "pct:3,k=40",
                    "uniform:0.01",
                    "uniform:0.2",
                    "uniform:1",
                    "pre:1,k=40"
                ]
            );
        }
        assert_eq!(with_k("pre:2", 0), "pre:2,k=1");
    }

    #[test]
    fn a_run_is_compared_without_its_sim_lines() {
        let stderr = b"rexx-sim: seed=9 policy=fifo\nstarted X\nrexx-sim: seed=1 \
                    policy=fifo steps=3 contended=2 switches=1 trace=00000000000000ff \
                    profile=release stack=1\n";
        assert_eq!(sim_masked(stderr), b"started X\n");
        let line = sim_line(stderr).expect("the run's own line");
        assert_eq!(sim_field(&line, "contended"), Some("2"));
        assert_eq!(sim_field(&line, "trace"), Some("00000000000000ff"));
        assert_eq!(sim_line(b"rexx-sim: x\nrexx-exec: y\n"), None);
    }

    /// A finished run of a group part: `stdout`, the started test `T`
    /// and `refusal` on stderr, a `rexx-sim:` line, `status`.
    fn finished(stdout: &str, refusal: Option<&str>, status: i32) -> ProcessRun {
        let mut stderr = "started T\n".to_string();
        if let Some(line) = refusal {
            stderr.push_str(line);
            stderr.push('\n');
        }
        stderr.push_str("rexx-sim: seed=1 policy=fifo steps=1 contended=0 switches=0\n");
        ProcessRun {
            run: Run {
                stdout: stdout.as_bytes().to_vec(),
                stderr: stderr.into_bytes(),
                status: Some(status),
            },
            ended: Ended::Exited(status),
            took: Duration::ZERO,
        }
    }

    fn group_row(group: &str) -> Row {
        Row {
            group: group.to_string(),
            part: "whole".to_string(),
            k: 1,
            release_seeds: 1,
            debug_seeds: 1,
            release_ms: 0,
            debug_ms: 0,
            knobs: None,
            left_out: Vec::new(),
        }
    }

    const PASSED: &str = "Tests ran:          1\nAssertions:         3\n\
                                  Failures:           0\nErrors:             0\n";

    /// The early-end reds (review I1): an exit status no oracle
    /// outcome has, beside a pass summary, and a refusal that neither
    /// an oracle outcome nor the part's `DIFFERING` key names; each
    /// beside the run that differs from it only in what is listed.
    #[test]
    fn a_run_ending_where_no_listed_outcome_ends_is_red() {
        let row = group_row("base/x/Y.testGroup");
        let seen = [Seen {
            count: 5,
            status: "0".to_string(),
            digest: 0,
            key: "pass, assertions 3, rc 0, last started T, failing []".to_string(),
        }];
        let kinds = |ran: &ProcessRun, row: &Row, policy: &str| -> Vec<(Kind, String)> {
            judge(row, policy, ran, Some(&seen))
                .into_iter()
                .map(|red| (red.kind, red.line))
                .collect()
        };
        assert_eq!(kinds(&finished(PASSED, None, 0), &row, "pre:1,k=1"), []);
        assert_eq!(
            kinds(&finished(PASSED, None, 1), &row, "pre:1,k=1"),
            [(
                Kind::Abort,
                "rc 1, which no oracle outcome or DIFFERING key of the part has".to_string()
            )]
        );
        assert_eq!(kinds(&finished(PASSED, None, 1), &row, "pct:3,k=1"), []);
        // Object whole's DIFFERING key ends at rc 1.
        let object = group_row("base/class/Object.testGroup");
        assert_eq!(kinds(&finished(PASSED, None, 1), &object, "pre:1,k=1"), []);
        assert_eq!(
            kinds(&finished("", None, 0), &row, "pre:1,k=1"),
            [(Kind::Abort, "no ooTest summary".to_string())]
        );
        let listed = "rexx-exec: method \"OBJECTNAME=\" of class \"Object\" is not \
                              implemented (Phase 9)";
        let other = "rexx-exec: method \"COPY\" of class \"Object\" is not implemented \
                             (Phase 9)";
        let key = format!("{listed}, rc 120, last started T, failing []");
        let differing = [key.as_str()];
        let early = |line: &str, status: i32, policy: &str| {
            early_end(
                &row,
                policy,
                &finished("", Some(line), status).run,
                &seen,
                &differing,
            )
        };
        let unlisted = |line: &str, status: i32| {
            Some(format!(
                "{line}, rc {status}: no oracle outcome or DIFFERING key of the part ends so"
            ))
        };
        assert_eq!(early(listed, 120, "pre:1,k=1"), None);
        assert_eq!(early(other, 120, "pre:1,k=1"), unlisted(other, 120));
        assert_eq!(early(other, 120, "pct:3,k=1"), unlisted(other, 120));
        assert_eq!(
            kinds(&finished("", Some(listed), 120), &row, "pre:1,k=1"),
            [(Kind::Abort, unlisted(listed, 120).expect("a line"))]
        );
        // A strict prefix of the listed refusal, and the listed refusal
        // at another status, are not the listed outcome.
        let prefix = listed.trim_end_matches(" (Phase 9)");
        for (line, status) in [(prefix, 120), (listed, 1)] {
            assert_eq!(early(line, status, "pre:1,k=1"), unlisted(line, status));
        }
        // A program's status is judged against its oracle set's.
        let program = Row {
            part: "program".to_string(),
            ..group_row("crates/rexx-exec/tests/sim_gate/x.rex")
        };
        let seen_program = [Seen {
            count: 5,
            status: "0".to_string(),
            digest: 0,
            key: "rc 0, stdout 1 lines, stderr 0 lines".to_string(),
        }];
        let program_run = |status: i32| {
            let mut ran = finished("done\n", None, status);
            ran.run.stderr = b"rexx-sim: seed=1 policy=fifo\n".to_vec();
            judge(&program, "pre:1,k=1", &ran, Some(&seen_program))
                .into_iter()
                .map(|red| (red.kind, red.line))
                .collect::<Vec<_>>()
        };
        assert_eq!(program_run(0), []);
        assert_eq!(
            program_run(3),
            [(
                Kind::Abort,
                "rc 3, which no oracle outcome or DIFFERING key of the part has".to_string()
            )]
        );
    }

    /// A WALL_CLOCK rerun that passes is a determinism red outside
    /// `clock=real` and not under it; a rerun that fails keeps its
    /// own reds.
    #[test]
    fn a_passing_rerun_is_a_determinism_red_outside_the_real_clock() {
        let virtual_clock = group_row("base/bif/STREAM.testGroup");
        let real_clock = Row {
            knobs: Some("clock=real".to_string()),
            ..group_row("base/bif/STREAM.testGroup")
        };
        let reds = after_rerun(
            &virtual_clock,
            "TEST_QUERYDIR_EXISTS".to_string(),
            Vec::new(),
        );
        assert_eq!(
            reds.iter()
                .map(|red| (red.kind, red.test.as_str(), red.line.as_str()))
                .collect::<Vec<_>>(),
            [(
                Kind::Determinism,
                "TEST_QUERYDIR_EXISTS",
                "TEST_QUERYDIR_EXISTS failed, and the rerun of the same seed and mode passed"
            )]
        );
        assert!(
            after_rerun(&real_clock, "TEST_QUERYDIR_EXISTS".to_string(), Vec::new()).is_empty()
        );
        let failed = vec![Red {
            kind: Kind::Check,
            test: "TEST_QUERYDIR_EXISTS".to_string(),
            line: "fails".to_string(),
        }];
        let kept = after_rerun(&virtual_clock, "TEST_QUERYDIR_EXISTS".to_string(), failed);
        assert_eq!(kept.len(), 1);
        assert_eq!(kept[0].kind, Kind::Check);
    }

    #[test]
    fn a_key_names_its_failing_tests() {
        assert_eq!(
            failing_of("failure, assertions 3, rc 1, last started B, failing [A B]"),
            BTreeSet::from(["A".to_string(), "B".to_string()])
        );
        assert!(failing_of("pass, assertions 3, rc 0, last started B, failing []").is_empty());
    }

    /// Every row of the table is a part the gate names: each group
    /// file of the derived list whole and derived, each program of
    /// `corpus/phase-6.txt`, and the gate's own programs; a rest or
    /// single row names a group with a whole row. Every other row has an
    /// oracle set, and every exempt row names a table row.
    #[test]
    fn the_table_holds_the_parts_the_spec_names() {
        let rows = read_table();
        let have: BTreeSet<(String, String)> = rows
            .iter()
            .filter(|row| {
                !matches!(row.part.as_str(), "rest" | "single") && !row.group.starts_with(PROGRAMS)
            })
            .map(|row| (row.group.clone(), row.part.clone()))
            .collect();
        let mut want = BTreeSet::new();
        for group in derived_tests().keys() {
            want.insert((group.clone(), "whole".to_string()));
            want.insert((group.clone(), "derived".to_string()));
        }
        let list = rust_root().join("corpus/phase-6.txt");
        for line in std::fs::read_to_string(&list).expect("phase-6.txt").lines() {
            let line = line.trim();
            if !line.is_empty() && !line.starts_with('#') {
                want.insert((format!("corpus/{line}"), "program".to_string()));
            }
        }
        assert_eq!(have, want);
        for row in rows.iter().filter(|row| row.part == "single") {
            assert!(
                have.contains(&(row.group.clone(), "whole".to_string())),
                "{} single has no whole row",
                row.group
            );
            assert_eq!(row.left_out.len(), 1, "{} single names one test", row.group);
        }
        for row in rows.iter().filter(|row| row.part == "rest") {
            assert!(
                have.contains(&(row.group.clone(), "whole".to_string())),
                "{} rest has no whole row",
                row.group
            );
            assert!(
                !row.left_out.is_empty(),
                "{} rest leaves nothing out",
                row.group
            );
        }
        for row in rows.iter().filter(|row| !row.injects()) {
            assert!(
                read_oracle(row).is_some_and(|seen| !seen.is_empty()),
                "{} {} has no committed oracle set",
                row.group,
                row.part
            );
        }
        assert!(rows.iter().any(|row| row.release_seeds > 0));
        assert!(rows.iter().any(|row| row.debug_seeds > 0));
        for one in read_exempt() {
            assert!(
                rows.iter().any(|row| {
                    (one.cells[0] == "*" || one.cells[0] == row.group)
                        && (one.cells[1] == "*" || one.cells[1] == row.part)
                }),
                "{EXEMPT} row {:?} names no table row",
                one.cells
            );
        }
    }
}
