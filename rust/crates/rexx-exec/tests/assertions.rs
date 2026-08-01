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
//!
//! # Why two `SAY`s and not `==`
//!
//! For a row with no raise expectation (below): `OOREXXUNIT.CLS`'s
//! `assertSame` is `if \ (expected == actual) then … fail`, and Rexx `==` is
//! exact-string identity, no padding, no coercion. This harness reproduces
//! that by turning each row into a two-line program -- `SAY <expr>` then
//! `SAY <expected>` -- and comparing the two output lines **byte for byte**
//! rather than asking `rexx_exec` to evaluate `==` itself.
//! That is deliberate and not merely equivalent: criterion 2 is a byte-for-
//! byte comparison to catch the created-digits/created-form rendering story
//! (D15), and asking the executor's own `==` operator to be the judge would
//! make the harness depend on the very code path it exists to check. Two
//! independent `SAY`s and a `Vec<u8>` comparison in the test binary itself
//! have no such circularity.
//!
//! # `NUMERIC DIGITS`/`FORM` per row, not once per file
//!
//! Every row carries the `DIGITS`/`FORM` in force at the point its assertion
//! ran (Task 15a scanned each `.testGroup` file sequentially to compute
//! this, from 1 to 100 and `SCIENTIFIC`/`ENGINEERING`). This harness emits
//! both as the first two lines of that row's own program, never a shared
//! default, and [`digits_and_form_are_carried_not_defaulted`] is a
//! regression test for the failure mode named in Task 15's brief: a row
//! evaluated at the wrong precision can render an answer that still happens
//! to match and passes while testing the wrong thing.
//!
//! # `CONCATENATION`'s prelude
//!
//! `AssertionRow::prelude` (Task 15a) carries the method's `a`..`g`
//! assignment lines verbatim; this harness's program builder ([`program_for`])
//! writes them out before the two `SAY`s, so `expr`/`expected` see the same
//! bindings the original method body would have given them. Nothing here
//! substitutes a made-up prelude or leaves it out: a row whose prelude Task
//! 15a could not represent is in `AssertionExtraction::blocked`, not among
//! the rows this file runs.
//!
//! # Rows that expect a raise, not a value
//!
//! `AssertionRow::expect_raise` (`rexx-extract`) is `Some(major.sub)` for a
//! row that follows a `self~expectSyntax` in its own method: the ooTest
//! framework defers a raise-check to whatever runs next, so that row is not
//! testing "expr equals expected" at all, it is testing "evaluating expr
//! raises major.sub" -- and `expected` is never even reached under the
//! oracle, since Rexx evaluates a message send's arguments left to right and
//! the raise happens while evaluating the first one. [`program_for`] builds
//! such a row's program with `expected` left out entirely, matching that
//! evaluation order rather than printing a second line nothing would ever
//! read; [`RowOutcome::RaiseMismatch`] is what a row like this produces
//! instead of [`RowOutcome::Mismatch`], and [`the_raise_falsification_proof`]
//! is the mismatch-shaped falsification proof for it, matching item 3's own
//! concern (major and sub are checked, not major alone -- `26.11` and `26.2`
//! must not be confused for each other).
//!
//! # Rows this harness cannot run yet
//!
//! A row is **runtime-blocked** if its program hits
//! [`rexx_exec::NOT_IMPLEMENTED_EXIT`] -- some `ExprKind` the row's `expr` or
//! `expected` text constructs is outside 4a's scope. Measured over the full
//! 4,259-row set (see [`assertions_differential`]'s own report and
//! `task-15b-report.md`): 35 rows, all in `Literals.testGroup` (2 `a function
//! call`, 33 `a message send`), none anywhere else -- `base/expressions`
//! otherwise constructs only the arithmetic, comparison, concatenation and
//! logical forms 4a already evaluates, plus plain literals and variables.
//! [`owning_subphase`] names the sub-phase for the two constructs actually
//! observed (`a function call` is 4b's, `a message send` is Phase 5's, both
//! per the design spec's own split table); anything else falls to an
//! honestly-unknown fallback rather than a guess, since this harness has
//! never measured it. The classification exists at all because "measured
//! today" is a fact about this corpus at this commit, not a property that
//! should be assumed going forward: a later task landing a new group of
//! `.testGroup` files, or 4a itself narrowing its scope, could change this
//! set, and a harness that could not even name what it hit would be the
//! silent-vacuous-harness shape this project keeps finding in its own
//! instruments.
//!
//! # REPORT vs STRICT, and why the report reaches the terminal
//!
//! Modelled directly on `tests/corpus.rs`, which solved both problems first:
//! [`GATE_ENV`] switches between an always-green progress report and the
//! phase gate, and [`emit_uncaptured`] pipes the report through a child
//! process whose stderr is inherited, because a `println!`/`eprintln!`
//! inside a `#[test]` writes to libtest's thread-local capture sink and
//! never reaches the terminal under a plain `cargo test`. See
//! `corpus.rs`'s own module doc for the fuller argument and the measurement
//! that motivated it; nothing about the mechanism differs here, so it is
//! not re-derived.

use rexx_exec::{NOT_IMPLEMENTED_EXIT, Outcome, run_program};
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
///
/// Not an env var: unlike `corpus.rs`'s oracle, this is checked-in test data
/// in the same repository, not an external build a machine might be missing
/// -- there is nothing here for a configurable path to usefully point at
/// instead.
fn suite_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../ootest/ooRexx/base/expressions")
}

/// Every `AssertionRow` and `BlockedMethod` in the suite, group by group in
/// sorted file order (`find_test_groups` already sorts).
///
/// This does **not** re-pin the row/blocked counts against a hardcoded
/// number: `rexx-extract`'s own tests (`base_expressions_yields_the_measured_
/// row_and_blocked_counts`) already do that for the extractor's output, and
/// duplicating the pin here would just be two places that can go stale
/// against each other. What this function's caller does check is that the
/// count is nonzero and matches what the extractor's own invariant assertion
/// (rows + dropped == assertSame calls, enforced inside `extract_assertions`'s
/// caller in `rexx-extract-assertions`) implies -- recomputed independently
/// below in `assertions_differential` from `count_assert_same`, so a future
/// regression in either crate's counting is still caught from this side too.
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
///
/// A value-comparison row (`expect_raise: None`) gets a second `SAY
/// <expected>` after it, exactly as captured. A raise-expectation row
/// (`expect_raise: Some(_)`) does not: `expected` is never evaluated under
/// the oracle either, since Rexx evaluates a message send's arguments left
/// to right and the raise happens while evaluating `expr`, the first one --
/// printing a second line for it here would test something the row never
/// actually claims.
///
/// `expected` is wrapped in nothing here -- when it is written at all, it
/// is written out exactly as `rexx-extract` captured it, a bare `SAY
/// <expected>`. [`the_falsification_proof`] is the one place a row's
/// `expected` text is ever wrapped, and it wraps a *copy*, never this
/// function's own row.
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

/// Which sub-phase owns a runtime-blocked construct, for the two this
/// harness has actually measured -- see the module doc's "Rows this
/// harness cannot run yet". `None` for anything else: an honest "not yet
/// attributed" rather than a guess, since nothing here has measured a third
/// construct.
///
/// * `"a function call"` is `ExprKind::Call` -- the design spec's split
///   table assigns `Call` to 4b ("routine resolution, which is handover
///   1").
/// * `"a message send"` is `ExprKind::Message` -- the same table assigns
///   `Message` to Phase 5 ("4a has no general message dispatch", `value.rs`'s
///   own words), alongside `Guard`/`Reply`/`Forward`/every directive.
fn owning_subphase(construct: &str) -> Option<&'static str> {
    match construct {
        "a function call" => Some("4b"),
        "a message send" => Some("Phase 5"),
        _ => None,
    }
}

/// Runs one row's program and classifies what happened.
fn evaluate_row(row: &AssertionRow) -> RowOutcome {
    let outcome = run_program(ROW_PATH, program_for(row));
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
///
/// `Raised` (the payload that would carry `major`/`sub` directly) is
/// `pub(crate)` inside `rexx-exec` and this is an integration test outside
/// the crate, so the two pieces come from the only channel a public caller
/// has: `Outcome::exit_code` (`256 - major`, `error.rs`'s own rule) and the
/// oracle-format report on `Outcome::stderr`, whose second line is always
/// `Error <major>.<sub>:  <message>.` (`parse_condition_number`). Both are
/// read and cross-checked against each other rather than trusting either
/// alone -- see that function's own doc for why the *first* report line
/// (`Error <major> running <path> line <n>:  ...`) never parses as a
/// `major.sub` pair by accident even though it also starts with `Error `.
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
///
/// Scans every line for one starting `Error `, splits the rest at the first
/// `:`, and tries to parse *that* as `<major>.<sub>` (both plain integers).
/// The report's *first* line, `Error <major> running <path> line <n>:  ...`,
/// also starts with `Error ` but never parses this way by construction: its
/// segment before the first `:` is `<major> running <path> line <n>`, and
/// `<path>` is `ROW_PATH`, which itself contains a `.` (`assertion-row.rex`)
/// -- so `split_once('.')`'s first half there is non-numeric text, not a
/// bare major, and the whole parse fails on that line and falls through to
/// the second. Verified directly rather than assumed: every stderr this
/// harness has produced so far has exactly this two-line shape, and the
/// function returns the *first* line it can parse rather than picking one
/// by position, so it does not depend on that continuing to hold.
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

fn describe(row: &AssertionRow, outcome: &RowOutcome) -> Option<Reported> {
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
        RowOutcome::RuntimeBlocked { construct } => match owning_subphase(construct) {
            Some(phase) => format!("RUNTIME-BLOCKED: {construct} is not implemented ({phase})"),
            None => format!(
                "RUNTIME-BLOCKED: {construct} is not implemented (sub-phase not attributed)"
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

/// The runner. See the module doc for REPORT vs STRICT, and
/// `task-15b-report.md` for the measured counts.
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
    for row in &rows {
        let outcome = evaluate_row(row);
        match describe(row, &outcome) {
            None => passed += 1,
            Some(r) => reported.push(r),
        }
    }

    let gate = gate_mode();
    emit_uncaptured(&build_report(total, passed, &reported, &blocked, gate));

    assert!(
        !gate || reported.is_empty(),
        "STRICT ({GATE_ENV}) mode: {} of {total} assertion-table rows are not passing; see \
         the report above for which and why.",
        reported.len()
    );
}

/// The falsification proof Task 15's brief requires: perturbing one row's
/// `expected` value must make exactly that row's comparison fail, and must
/// not touch anything else.
///
/// Wraps the original `expected` text in parens before appending the
/// concatenation, `({expected}) || 'ZZZ-FALSIFICATION-MARKER'`, rather than
/// appending to the raw text directly: concatenation binds *tighter* than
/// comparison in Rexx precedence, so appending straight onto text that
/// happens to contain a top-level comparison (several `CONCATENATION` rows
/// do) would silently regroup the expression instead of just perturbing its
/// value. The parens are grouping only -- confirmed in `rexx-parse`
/// (`expr.rs`'s `subterm`, `LeftParen` arm: "the parenthesised expression is
/// returned unchanged, so there is no node for the parentheses") -- so this
/// changes nothing about how the original text evaluates and only adds the
/// marker on top.
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
///
/// Proves the harness is actually *sensitive* to the row's own
/// `digits`/`form` fields, not merely that Task 15a computed them
/// correctly (its own tests already cover that): running this row's real
/// `expr`/`expected` at the *default* settings (`DIGITS 9`, `FORM
/// SCIENTIFIC`) must render a different pair of lines and must not pass,
/// which is the direct, measured version of "silently tests the wrong
/// precision and still passes" not holding here.
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
        describe(row, &honest).map(|r| r.detail).unwrap_or_default()
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
        describe(row, &honest).map(|r| r.detail).unwrap_or_default()
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
