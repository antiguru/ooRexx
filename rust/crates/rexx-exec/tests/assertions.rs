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

//! Task 15's consuming half: runs every `AssertionRow` `rexx-extract`'s
//! `extract_assertions` lifts out of `ootest/ooRexx/base/expressions/` (Task
//! 15a) through `rexx_exec`'s public entry point and checks it the way the
//! oracle's own test framework does.

mod watchdog;

use rayon::prelude::*;
use rexx_exec::{NOT_IMPLEMENTED_EXIT, Outcome};
use rexx_extract::{
    AssertionRow, BlockedMethod, Form, RaiseExpectation, extract_assertions, find_test_groups,
};
use std::collections::BTreeMap;
use std::env;
use std::fmt::Write as _;
use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// The path every row's synthetic program is reported under. There is no
/// real file backing a row -- its program is assembled from the row's own
/// fields -- so this is a label, not a location, in the same spirit as
/// `spike.rs`'s `SPIKE_PATH`.
const ROW_PATH: &str = "/nonexistent/assertion-row.rex";

/// Env var that flips this test from a progress report into the phase gate.
/// Named separately from `corpus.rs`'s `REXX_CORPUS_GATE` because the two
/// gate independent things -- the L0 corpus and this L1 assertion table --
/// and a caller should be able to run one without the other.
const GATE_ENV: &str = "REXX_ASSERTIONS_GATE";

fn gate_mode() -> bool {
    match env::var(GATE_ENV) {
        Ok(value) => !value.is_empty() && value != "0",
        Err(_) => false,
    }
}

/// `ootest/ooRexx/base/expressions/`, hardcoded relative to this crate.
fn suite_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../ootest/ooRexx/base/expressions")
}

/// Every `AssertionRow` and `BlockedMethod` in the suite, group by group in
/// sorted file order (`find_test_groups` already sorts).
fn collect_all() -> (Vec<AssertionRow>, Vec<BlockedMethod>) {
    let dir = suite_root();
    let mut groups = find_test_groups(&dir);
    groups.sort();
    assert!(
        !groups.is_empty(),
        "no .testGroup files under {} -- suite_root points at the wrong \
         directory, or the checkout is missing base/expressions entirely",
        dir.display()
    );

    let mut rows = Vec::new();
    let mut blocked = Vec::new();
    for path in &groups {
        let bytes =
            fs::read(path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
        let source = String::from_utf8_lossy(&bytes);
        let group_name = path.file_stem().and_then(|s| s.to_str()).unwrap_or("group");
        let extraction = extract_assertions(group_name, &source);
        rows.extend(extraction.rows);
        blocked.extend(extraction.blocked);
    }
    (rows, blocked)
}

/// The keyword `NUMERIC FORM` takes for a row's [`Form`].
fn form_keyword(form: Form) -> &'static str {
    match form {
        Form::Scientific => "SCIENTIFIC",
        Form::Engineering => "ENGINEERING",
    }
}

/// Turns one row into a standalone program: the `DIGITS`/`FORM` in force,
/// the method's assignment prelude verbatim, then `expr`.
fn program_for(row: &AssertionRow) -> Vec<u8> {
    let mut text = String::new();
    writeln!(text, "numeric digits {}", row.digits).unwrap();
    writeln!(text, "numeric form {}", form_keyword(row.form)).unwrap();
    for line in &row.prelude {
        writeln!(text, "{line}").unwrap();
    }
    writeln!(text, "say {}", row.expr).unwrap();
    if row.expect_raise.is_none() {
        writeln!(text, "say {}", row.expected).unwrap();
    }
    text.into_bytes()
}

/// What running one row's program decided.
enum RowOutcome {
    /// The two `SAY` lines were byte-identical (a value-comparison row), or
    /// the expected condition was raised with the exact major and sub (a
    /// raise-expectation row).
    Pass,
    /// Both lines rendered; they differ. A real divergence between
    /// `rexx_exec` and what the oracle's own test suite asserts.
    Mismatch { actual: Vec<u8>, expected: Vec<u8> },
    /// A raise-expectation row where the program did not raise the expected
    /// condition: either it did not raise at all (`actual: None`), or it
    /// raised a *different* major.sub (`actual: Some(_)`) -- checked as a
    /// pair, not major alone, so a row expecting `26.11` cannot be
    /// satisfied by `26.2`.
    RaiseMismatch {
        expect: RaiseExpectation,
        actual: Option<(u32, u32)>,
    },
    /// The program hit `NOT_IMPLEMENTED_EXIT`: some form in `expr` or
    /// `expected` is outside 4a's scope. `construct` is whatever
    /// `Loud`'s message named, taken verbatim off stderr.
    RuntimeBlocked { construct: String },
    /// None of the above: for a value-comparison row, a real Rexx condition
    /// escaped one that the oracle's own suite asserts passes, or the
    /// program did not print exactly two lines; for a raise-expectation
    /// row, it raised *something* but this harness could not parse a
    /// `major.sub` back out of its stderr, or the parsed major disagreed
    /// with the exit code's own `256 - major`. Reported with full detail
    /// rather than folded into `Mismatch`/`RaiseMismatch`, since neither of
    /// those is really what happened.
    Anomaly { detail: String },
}

/// Pulls `X` out of a `rexx-exec: X is not implemented` line. Identical in
/// shape to `corpus.rs`'s `owner_from_stderr`; both harnesses classify the
/// same loud-failure marker, so the pattern is duplicated deliberately
/// rather than pulled into a shared helper neither crate is a natural home
/// for (`rexx-exec/tests/` cannot depend on itself, and `rexx-extract`
/// has never run a program).
fn construct_from_stderr(stderr: &[u8]) -> Option<String> {
    let text = String::from_utf8_lossy(stderr);
    const MARKER: &str = "rexx-exec: ";
    const SUFFIX: &str = " is not implemented";
    let after_marker = &text[text.find(MARKER)? + MARKER.len()..];
    let end = after_marker.find(SUFFIX)?;
    Some(after_marker[..end].to_string())
}

/// One row this harness cannot make pass through `rexx_exec`'s public entry
/// point today, named explicitly. See the module doc's "The exempt set" for
/// why this exists and how STRICT is allowed to use it without becoming an
/// unpoliced escape hatch.
struct ExemptRow {
    group: &'static str,
    method: &'static str,
    /// 1-based position of this `self~assertSame` within its own
    /// `group`+`method`, in source order (see [`occurrence_of`]). Needed
    /// because `test_string_range`'s two rows share byte-identical `expr`/
    /// `expected` text -- only their prelude differs, and `AssertionRow`
    /// does not carry the prelude into this identity check -- so `(group,
    /// method, expr, expected)` alone is not a unique key for them.
    occurrence: usize,
    expr: &'static str,
    expected: &'static str,
    /// The sub-phase whose delivery would actually make this row pass.
    /// **Not** the same question as "which construct does this row's
    /// program happen to hit first today" (`RowOutcome::RuntimeBlocked`'s
    /// own `construct` field, reported separately) -- see the module doc's
    /// "Rows this harness cannot run yet" for the two `test_string_range`
    /// rows where the two answers differ.
    unblocked_by: &'static str,
}

/// The committed exempt set: every row this harness measured as not
/// passing at the time this list was written, with the sub-phase that
/// would actually unblock it. Generated once from a real run and hand
/// -verified against the source (`Literals.testGroup`), not hand-guessed --
/// see `task-15b-report.md` for the method.
const EXEMPT: &[ExemptRow] = &[
    ExemptRow {
        group: "Literals",
        method: "test_string_range",
        occurrence: 1,
        expr: "all",
        expected: "self~runDynamicSource(\"return\" self~q(all~changeStr('\"', '\"\"')))",
        unblocked_by: "Phase 5",
    },
    ExemptRow {
        group: "Literals",
        method: "test_string_range",
        occurrence: 2,
        expr: "all",
        expected: "self~runDynamicSource(\"return\" self~q(all~changeStr('\"', '\"\"')))",
        unblocked_by: "Phase 5",
    },
    ExemptRow {
        group: "Literals",
        method: "test_hexadecimal",
        occurrence: 4,
        expr: "\"AB\"",
        expected: "self~runDynamicSource(\"return\" self~hex(\"41\" || tab || \"42\"))",
        unblocked_by: "Phase 5",
    },
    ExemptRow {
        group: "Literals",
        method: "test_hexadecimal",
        occurrence: 5,
        expr: "\"AB\"",
        expected: "self~runDynamicSource(\"return\" self~hex(\"41\" || tab || tab || \"42\"))",
        unblocked_by: "Phase 5",
    },
    ExemptRow {
        group: "Literals",
        method: "test_hexadecimal",
        occurrence: 6,
        expr: "\"AB\"",
        expected: "self~runDynamicSource(\"return\" self~hex(\"41\" || tab || tab || tab || \"42\"))",
        unblocked_by: "Phase 5",
    },
    ExemptRow {
        group: "Literals",
        method: "test_hexadecimal",
        occurrence: 7,
        expr: "\"AB\"",
        expected: "self~runDynamicSource(\"return\" self~hex(\"41 \" || tab || \"42\"))",
        unblocked_by: "Phase 5",
    },
    ExemptRow {
        group: "Literals",
        method: "test_hexadecimal",
        occurrence: 8,
        expr: "\"AB\"",
        expected: "self~runDynamicSource(\"return\" self~hex(\"41\" || tab || \" 42\"))",
        unblocked_by: "Phase 5",
    },
    ExemptRow {
        group: "Literals",
        method: "test_binary",
        occurrence: 4,
        expr: "\"A\"",
        expected: "self~runDynamicSource(\"return\" self~bin(\"0100\" || tab || \"0001\"))",
        unblocked_by: "Phase 5",
    },
    ExemptRow {
        group: "Literals",
        method: "test_binary",
        occurrence: 5,
        expr: "\"A\"",
        expected: "self~runDynamicSource(\"return\" self~bin(\"0100\" || tab || tab || \"0001\"))",
        unblocked_by: "Phase 5",
    },
    ExemptRow {
        group: "Literals",
        method: "test_binary",
        occurrence: 6,
        expr: "\"A\"",
        expected: "self~runDynamicSource(\"return\" self~bin(\"0100\" || tab || tab || tab || \"0001\"))",
        unblocked_by: "Phase 5",
    },
    ExemptRow {
        group: "Literals",
        method: "test_binary",
        occurrence: 7,
        expr: "\"A\"",
        expected: "self~runDynamicSource(\"return\" self~bin(\"0100 \" || tab || \"0001\"))",
        unblocked_by: "Phase 5",
    },
    ExemptRow {
        group: "Literals",
        method: "test_binary",
        occurrence: 8,
        expr: "\"A\"",
        expected: "self~runDynamicSource(\"return\" self~bin(\"0100\" || tab || \" 0001\"))",
        unblocked_by: "Phase 5",
    },
];

/// `row`'s 1-based position among every row seen so far (including `row`
/// itself) sharing its `group` and `method`, in the source order `rows`
/// already carries. `counts` is the caller's running tally, keyed by
/// `(group, method)`, threaded through one call per row in a single pass
/// rather than recomputed by re-scanning `rows` every time -- there are
/// 4,259 of them, and this runs once per row.
fn occurrence_of(
    counts: &mut std::collections::HashMap<(String, String), usize>,
    row: &AssertionRow,
) -> usize {
    let key = (row.group.clone(), row.method.clone());
    let count = counts.entry(key).or_insert(0);
    *count += 1;
    *count
}

/// Looks `row` (at its `occurrence` position within its own group+method)
/// up in [`EXEMPT`] by full identity -- `group`, `method`, `occurrence` and
/// both `expr` and `expected` text, all five, so a corpus edit that shifts
/// occurrence numbers or changes a row's text cannot silently keep
/// matching the wrong committed entry.
fn exempt_entry(row: &AssertionRow, occurrence: usize) -> Option<&'static ExemptRow> {
    EXEMPT.iter().find(|e| {
        e.group == row.group
            && e.method == row.method
            && e.occurrence == occurrence
            && e.expr == row.expr
            && e.expected == row.expected
    })
}

/// Runs one row's program and classifies what happened.
fn evaluate_row(row: &AssertionRow) -> RowOutcome {
    let outcome = watchdog::run_bounded(ROW_PATH, program_for(row), rexx_exec::Invocation::none());
    classify(row, outcome)
}

/// The classification step on its own, apart from running the program, so
/// the falsification tests can reuse it against a hand-built `Outcome`-
/// producing run without duplicating the exit-code logic. Dispatches on
/// `row.expect_raise` to one of the two shapes a row can test.
fn classify(row: &AssertionRow, outcome: Outcome) -> RowOutcome {
    match row.expect_raise {
        Some(expect) => classify_raise(expect, outcome),
        None => classify_value(outcome),
    }
}

/// The original, value-comparison classification: two `SAY` lines, compared
/// byte for byte.
fn classify_value(outcome: Outcome) -> RowOutcome {
    if outcome.exit_code == NOT_IMPLEMENTED_EXIT {
        let construct =
            construct_from_stderr(&outcome.stderr).unwrap_or_else(|| "<unnamed>".to_string());
        return RowOutcome::RuntimeBlocked { construct };
    }
    if outcome.exit_code != 0 {
        return RowOutcome::Anomaly {
            detail: format!(
                "exit {} (a real condition escaped a row the oracle's own suite asserts \
                 passes); stderr={:?}",
                outcome.exit_code,
                String::from_utf8_lossy(&outcome.stderr)
            ),
        };
    }
    // Exactly two `SAY` lines are expected: `expr`'s rendering, then
    // `expected`'s. `strip_suffix` rather than a trailing-empty-element
    // filter on `split`, so a program that printed a third line (which
    // nothing in `program_for` should ever cause, but a future change to
    // it might) is caught as an anomaly instead of silently comparing the
    // wrong two lines.
    let trimmed = outcome
        .stdout
        .strip_suffix(b"\n")
        .unwrap_or(&outcome.stdout);
    let parts: Vec<&[u8]> = trimmed.split(|&b| b == b'\n').collect();
    if parts.len() != 2 {
        return RowOutcome::Anomaly {
            detail: format!(
                "expected exactly 2 SAY lines, got {}: {:?}",
                parts.len(),
                String::from_utf8_lossy(&outcome.stdout)
            ),
        };
    }
    let (actual, expected) = (parts[0].to_vec(), parts[1].to_vec());
    if actual == expected {
        RowOutcome::Pass
    } else {
        RowOutcome::Mismatch { actual, expected }
    }
}

/// The raise-expectation classification: `expr` alone must raise exactly
/// `expect.major`.`expect.sub`.
fn classify_raise(expect: RaiseExpectation, outcome: Outcome) -> RowOutcome {
    if outcome.exit_code == NOT_IMPLEMENTED_EXIT {
        let construct =
            construct_from_stderr(&outcome.stderr).unwrap_or_else(|| "<unnamed>".to_string());
        return RowOutcome::RuntimeBlocked { construct };
    }
    if outcome.exit_code == 0 {
        return RowOutcome::RaiseMismatch {
            expect,
            actual: None,
        };
    }
    let Some((major, sub)) = parse_condition_number(&outcome.stderr) else {
        return RowOutcome::Anomaly {
            detail: format!(
                "exit {} but no parseable \"Error major.sub:\" line in stderr: {:?}",
                outcome.exit_code,
                String::from_utf8_lossy(&outcome.stderr)
            ),
        };
    };
    let major_from_exit = 256 - outcome.exit_code;
    if i64::from(major) != i64::from(major_from_exit) {
        return RowOutcome::Anomaly {
            detail: format!(
                "exit code {} implies major {major_from_exit} (256 - major), but stderr's own \
                 report line says {major}.{sub}: {:?}",
                outcome.exit_code,
                String::from_utf8_lossy(&outcome.stderr)
            ),
        };
    }
    if major == expect.major && sub == expect.sub {
        RowOutcome::Pass
    } else {
        RowOutcome::RaiseMismatch {
            expect,
            actual: Some((major, sub)),
        }
    }
}

/// Finds `major`/`sub` in the oracle-format report's second line, `Error
/// <major>.<sub>:  <message>.`.
fn parse_condition_number(stderr: &[u8]) -> Option<(u32, u32)> {
    let text = String::from_utf8_lossy(stderr);
    for line in text.lines() {
        let Some(rest) = line.strip_prefix("Error ") else {
            continue;
        };
        let Some(colon) = rest.find(':') else {
            continue;
        };
        let code = &rest[..colon];
        let Some((major, sub)) = code.split_once('.') else {
            continue;
        };
        if let (Ok(major), Ok(sub)) = (major.trim().parse::<u32>(), sub.trim().parse::<u32>()) {
            return Some((major, sub));
        }
    }
    None
}

/// Bounds a byte string to a short, readable excerpt for the report.
fn excerpt(bytes: &[u8]) -> String {
    const BOUND: usize = 200;
    let text = String::from_utf8_lossy(bytes);
    if text.chars().count() > BOUND {
        format!("{}...", text.chars().take(BOUND).collect::<String>())
    } else {
        text.into_owned()
    }
}

/// One row that did not pass, with enough of its identity to find it again
/// in the `.testGroup` source.
struct Reported {
    group: String,
    method: String,
    expr: String,
    expected: String,
    detail: String,
}

/// `exempt` is this row's own [`EXEMPT`] entry, if it has one -- looked up
/// once by the caller (which already computed `occurrence`) rather than
/// re-derived here, so this function stays a pure formatter.
fn describe(
    row: &AssertionRow,
    outcome: &RowOutcome,
    exempt: Option<&ExemptRow>,
) -> Option<Reported> {
    let detail = match outcome {
        RowOutcome::Pass => return None,
        RowOutcome::Mismatch { actual, expected } => format!(
            "MISMATCH: expr rendered {:?}, expected rendered {:?}",
            excerpt(actual),
            excerpt(expected)
        ),
        RowOutcome::RaiseMismatch { expect, actual } => {
            let actual = match actual {
                Some((major, sub)) => format!("{major}.{sub}"),
                None => "did not raise at all (exit 0)".to_string(),
            };
            format!(
                "RAISE-MISMATCH: expected {}.{}, got {actual}",
                expect.major, expect.sub
            )
        }
        // `construct` is the first-hit fact ("what did this row's program
        // actually run into"); `exempt.unblocked_by` is the separate,
        // committed fact ("what would actually make this row pass") -- see
        // the module doc's "Rows this harness cannot run yet" for the two
        // rows where those differ. A `RuntimeBlocked` row with no exempt
        // entry is not on the committed list at all, and says so plainly
        // rather than guessing a phase for it.
        RowOutcome::RuntimeBlocked { construct } => match exempt {
            Some(e) => format!(
                "RUNTIME-BLOCKED: first hit {construct} (not implemented); unblocked only by {}",
                e.unblocked_by
            ),
            None => format!(
                "RUNTIME-BLOCKED: first hit {construct} (not implemented); NOT on the committed \
                 EXEMPT list -- this is a new blocked row, not a known one"
            ),
        },
        RowOutcome::Anomaly { detail } => format!("ANOMALY: {detail}"),
    };
    Some(Reported {
        group: row.group.clone(),
        method: row.method.clone(),
        expr: row.expr.clone(),
        expected: row.expected.clone(),
        detail,
    })
}

/// Builds the report text, in the same "always visible, caveated top and
/// bottom in REPORT mode" shape as `corpus.rs::build_report`.
fn build_report(
    total: usize,
    passed: usize,
    reported: &[Reported],
    extraction_blocked: &[BlockedMethod],
    gate_failures: &[String],
    gate: bool,
) -> String {
    let banner = "=".repeat(78);
    let mut report = String::new();
    let w = &mut report;
    writeln!(w, "{banner}").unwrap();
    writeln!(
        w,
        "rexx-exec assertion-table report -- ootest/ooRexx/base/expressions"
    )
    .unwrap();
    if gate {
        writeln!(w, "mode: STRICT (the gate) -- {GATE_ENV} is set").unwrap();
    } else {
        writeln!(
            w,
            "*** REPORT MODE -- NOT THE GATE. Set {GATE_ENV}=1 to run this as the gate. ***"
        )
        .unwrap();
    }
    let not_the_gate = if gate {
        ""
    } else {
        " -- REPORT MODE, NOT THE GATE"
    };
    writeln!(w, "{passed} of {total} rows passing{not_the_gate}").unwrap();

    writeln!(
        w,
        "extraction-blocked (Task 15a, unsupported prelude shape): {} methods, {} assertSame \
         calls dropped before ever becoming a row",
        extraction_blocked.len(),
        extraction_blocked.iter().map(|b| b.dropped).sum::<usize>()
    )
    .unwrap();
    for blocked in extraction_blocked {
        writeln!(
            w,
            "  extraction-blocked: {}::{} -- {} ({} dropped)",
            blocked.group, blocked.method, blocked.reason, blocked.dropped
        )
        .unwrap();
    }

    if !reported.is_empty() {
        writeln!(w, "not passing ({}):", reported.len()).unwrap();
        for r in reported {
            writeln!(
                w,
                "  [{}::{}] expr={:?} expected={:?}: {}",
                r.group, r.method, r.expr, r.expected, r.detail
            )
            .unwrap();
        }

        let mut by_kind: BTreeMap<&str, usize> = BTreeMap::new();
        for r in reported {
            let kind = r.detail.split(':').next().unwrap_or("UNKNOWN");
            *by_kind.entry(kind).or_insert(0) += 1;
        }
        writeln!(w, "by kind:").unwrap();
        for (kind, count) in &by_kind {
            writeln!(w, "  {kind}: {count}").unwrap();
        }
    }

    if !gate_failures.is_empty() {
        writeln!(
            w,
            "EXEMPT-set violations ({}) -- what STRICT would fail on right now:",
            gate_failures.len()
        )
        .unwrap();
        for failure in gate_failures {
            writeln!(w, "  {failure}").unwrap();
        }
    }

    if !gate {
        writeln!(
            w,
            "*** REPORT MODE -- NOT THE GATE. {passed} of {total} matching means {} rows are \
             not passing yet; it is a progress signal, not a claim that criterion 2 is met. ***",
            total - passed
        )
        .unwrap();
    }
    writeln!(w, "{banner}").unwrap();
    report
}

/// Writes `text` to the real, process-level stderr. Identical mechanism to
/// `corpus.rs::emit_uncaptured` -- see that file's own doc comment for why
/// `println!`/`eprintln!` cannot do this from inside a `#[test]` and how
/// this was verified. Not shared as a library function because both are
/// integration tests in different crates' `tests/` directories, which
/// cannot depend on each other.
fn emit_uncaptured(text: &str) {
    let mut child = Command::new("sh")
        .arg("-c")
        .arg("cat >&2")
        .stdin(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("spawning `sh -c 'cat >&2'` to bypass libtest's output capture");
    child
        .stdin
        .take()
        .expect("stdin was requested as piped")
        .write_all(text.as_bytes())
        .expect("writing the report to the uncaptured-output child's stdin");
    let status = child
        .wait()
        .expect("waiting for the uncaptured-output child");
    assert!(
        status.success(),
        "the uncaptured-output child (`sh -c 'cat >&2'`) exited abnormally: {status}"
    );
}

/// The runner. See the module doc for REPORT vs STRICT and "The exempt
/// set", and `task-15b-report.md` for the measured counts.
#[test]
fn assertions_differential() {
    let (rows, blocked) = collect_all();
    assert!(
        !rows.is_empty(),
        "extract_assertions produced no rows at all -- that is an extraction \
         defect, not an empty pass"
    );

    let total = rows.len();
    let mut passed = 0usize;
    let mut reported = Vec::new();
    let mut gate_failures: Vec<String> = Vec::new();
    let mut occurrence_counts: std::collections::HashMap<(String, String), usize> =
        std::collections::HashMap::new();
    let outcomes: Vec<RowOutcome> = rows.par_iter().map(evaluate_row).collect();
    for (row, outcome) in rows.iter().zip(outcomes) {
        let occurrence = occurrence_of(&mut occurrence_counts, row);
        let exempt = exempt_entry(row, occurrence);

        if matches!(outcome, RowOutcome::Pass) {
            passed += 1;
            if let Some(e) = exempt {
                gate_failures.push(format!(
                    "{}::{} occurrence {} now PASSES but is still listed in EXEMPT as \
                     unblocked_by {:?} -- remove it from EXEMPT",
                    e.group, e.method, e.occurrence, e.unblocked_by
                ));
            }
            continue;
        }

        if exempt.is_none() {
            gate_failures.push(format!(
                "{}::{} occurrence {occurrence} is not passing and is not on the committed \
                 EXEMPT list",
                row.group, row.method
            ));
        }
        if let Some(r) = describe(row, &outcome, exempt) {
            reported.push(r);
        }
    }

    let gate = gate_mode();
    emit_uncaptured(&build_report(
        total,
        passed,
        &reported,
        &blocked,
        &gate_failures,
        gate,
    ));

    assert!(
        !gate || gate_failures.is_empty(),
        "STRICT ({GATE_ENV}) mode: {} EXEMPT-set violation(s); see the report above for which \
         and why.",
        gate_failures.len()
    );
}

/// Polices [`EXEMPT`] itself, in every mode, independent of `{GATE_ENV}`:
/// the current not-passing set must equal the committed list **exactly**,
/// same identity check `exempt_entry` uses (`group`, `method`,
/// `occurrence`, `expr`, `expected`). This is the "assert that set" half
/// of criterion 5's own device -- `assertions_differential`'s STRICT mode
/// is the *use* of the committed list (deciding what to forgive); this is
/// the check that the list still describes reality, and it runs whether or
/// not anyone ever sets `{GATE_ENV}`.
#[test]
fn the_exempt_set_matches_the_current_blocked_rows() {
    let (rows, _) = collect_all();
    let mut occurrence_counts: std::collections::HashMap<(String, String), usize> =
        std::collections::HashMap::new();
    let outcomes: Vec<RowOutcome> = rows.par_iter().map(evaluate_row).collect();
    let mut still_blocked: Vec<(String, String, usize, String, String)> = Vec::new();
    for (row, outcome) in rows.iter().zip(outcomes) {
        let occurrence = occurrence_of(&mut occurrence_counts, row);
        if !matches!(outcome, RowOutcome::Pass) {
            still_blocked.push((
                row.group.clone(),
                row.method.clone(),
                occurrence,
                row.expr.clone(),
                row.expected.clone(),
            ));
        }
    }

    assert_eq!(
        still_blocked.len(),
        EXEMPT.len(),
        "the not-passing row count no longer matches EXEMPT's length -- a row was added to or \
         removed from reality without EXEMPT being updated to match"
    );
    for (group, method, occurrence, expr, expected) in &still_blocked {
        let found = EXEMPT.iter().any(|e| {
            e.group == *group
                && e.method == *method
                && e.occurrence == *occurrence
                && e.expr == *expr
                && e.expected == *expected
        });
        assert!(
            found,
            "{group}::{method} occurrence {occurrence} (expr={expr:?}, expected={expected:?}) \
             is not passing but has no EXEMPT entry -- add one, naming the sub-phase that \
             actually unblocks it"
        );
    }
}

/// The falsification proof Task 15's brief requires: perturbing one row's
/// `expected` value must make exactly that row's comparison fail, and must
/// not touch anything else.
#[test]
fn the_falsification_proof() {
    let (rows, _) = collect_all();
    let row = rows.first().expect(
        "collect_all's own non-empty assertion covers this too, restated for a \
                 reader of just this test",
    );

    let honest = evaluate_row(row);
    assert!(
        matches!(honest, RowOutcome::Pass),
        "the row picked for falsification does not even pass unperturbed, so a failure below \
         would prove nothing about the harness's sensitivity"
    );

    let mut perturbed = row.clone();
    perturbed.expected = format!("({}) || 'ZZZ-FALSIFICATION-MARKER'", row.expected);
    let falsified = evaluate_row(&perturbed);
    assert!(
        matches!(falsified, RowOutcome::Mismatch { .. }),
        "perturbing this row's expected value did not make it fail -- the comparison is not \
         sensitive to the expected text, which is exactly the vacuous-table shape criterion 2 \
         already had once"
    );

    // The other direction: an *unperturbed* second row (there are thousands;
    // any other suffices) must still be unaffected by having built and run
    // the perturbed clone above -- `Interp` is fresh per `run_program` call,
    // so there is no shared mutable state a stray perturbation could leak
    // through, but this is the cheap, direct check of that rather than an
    // appeal to the architecture.
    if let Some(other) = rows.get(1) {
        assert!(
            matches!(evaluate_row(other), RowOutcome::Pass),
            "a second, unperturbed row stopped passing after the perturbed clone ran -- \
             something is leaking state across rows"
        );
    }
}

/// Task 15's brief, item 3: a row evaluated under the wrong `NUMERIC
/// DIGITS`/`FORM` can render an answer that still happens to match and pass
/// while testing the wrong precision. This is the row `task-15a-report.md`
/// names as the pre-flight finding that added `FORM` to the row schema:
/// `ADDITION.testGroup`'s `test_198`, `Numeric Form ENGINEERING` + `Numeric
/// Digits 5`, `self~assertSame(9999999999999 + 9999999999999, 20.000E+12)`
/// -- `20.000E+12` is only the right answer in engineering notation at 5
/// digits.
#[test]
fn digits_and_form_are_carried_not_defaulted() {
    let (rows, _) = collect_all();
    let row = rows
        .iter()
        .find(|r| r.group == "ADDITION" && r.method == "test_198")
        .expect(
            "ADDITION.testGroup's test_198 is a fixed row this test names by hand; if it is \
             gone, the source file changed underneath this test and it needs a new witness, \
             not a relaxed assertion",
        );
    assert_eq!(row.digits, 5, "test_198's own Numeric Digits 5");
    assert_eq!(
        row.form,
        Form::Engineering,
        "test_198's own Numeric Form ENGINEERING"
    );

    let honest = evaluate_row(row);
    assert!(
        matches!(honest, RowOutcome::Pass),
        "test_198 at its real digits/form did not pass: {}",
        describe(row, &honest, None)
            .map(|r| r.detail)
            .unwrap_or_default()
    );

    let mut defaulted = row.clone();
    defaulted.digits = 9;
    defaulted.form = Form::Scientific;
    let wrong = evaluate_row(&defaulted);
    assert!(
        matches!(wrong, RowOutcome::Mismatch { .. }),
        "test_198 evaluated at the *default* digits/form (9, SCIENTIFIC) instead of its own \
         (5, ENGINEERING) still passed -- carrying these fields per row would not have been \
         load-bearing for this witness, which defeats the point of picking it"
    );
}

/// The falsification proof for the other row shape: a raise-expectation row
/// must fail if it does not raise at all, and -- the sharper case -- it
/// must fail if it raises the *wrong* condition, even one with the same
/// major. `DIVISION.testGroup`'s `test_262` (`self~expectSyntax(26.11)`,
/// `self~assertSame("-5678932" % "-37", 1)`) is the same witness
/// `task-15b-report.md` used to find the `expectSyntax` gap in the first
/// place, found by hand rather than by scanning `rows` for the first
/// `expect_raise: Some(_)` entry, so this test does not depend on which
/// row that happens to be.
#[test]
fn the_raise_falsification_proof() {
    let (rows, _) = collect_all();
    let row = rows
        .iter()
        .find(|r| r.group == "DIVISION" && r.method == "test_262")
        .expect(
            "DIVISION.testGroup's test_262 is a fixed row this test names by hand; if it is \
             gone, the source file changed underneath this test and it needs a new witness, \
             not a relaxed assertion",
        );
    let expect = row
        .expect_raise
        .expect("test_262 follows a self~expectSyntax(26.11) in its own method");
    assert_eq!(expect, RaiseExpectation { major: 26, sub: 11 });

    let honest = evaluate_row(row);
    assert!(
        matches!(honest, RowOutcome::Pass),
        "test_262 did not pass at its real expectation: {}",
        describe(row, &honest, None)
            .map(|r| r.detail)
            .unwrap_or_default()
    );

    // Did-not-raise: the sharpest possible perturbation, an expectation the
    // row's own `expr` cannot ever satisfy since it always raises 26.11.
    let mut impossible = row.clone();
    impossible.expect_raise = Some(RaiseExpectation { major: 1, sub: 1 });
    assert!(
        matches!(
            evaluate_row(&impossible),
            RowOutcome::RaiseMismatch {
                actual: Some((26, 11)),
                ..
            }
        ),
        "perturbing the expected condition to one the row's own expr cannot raise did not fail"
    );

    // The sharper case item 3 asks for by name: the *same major*, a
    // different sub. `26.2` is a real, distinct catalogue entry (a `DO`
    // repetitor error) from `26.11` ("Result of % operation did not result
    // in a whole number") -- a harness that only checked the major would
    // wave this one through.
    let mut wrong_sub = row.clone();
    wrong_sub.expect_raise = Some(RaiseExpectation { major: 26, sub: 2 });
    assert!(
        matches!(
            evaluate_row(&wrong_sub),
            RowOutcome::RaiseMismatch {
                actual: Some((26, 11)),
                ..
            }
        ),
        "perturbing only the sub-number (26.11 -> 26.2) did not fail -- the comparison is not \
         sensitive to sub, which is exactly the confusable-error-numbers hazard item 3 names"
    );
}
