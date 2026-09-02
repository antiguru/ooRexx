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

//! Phase 4c's L1 obligation: runs every row `rexx-extract`'s `bif` module
//! lifts out of `ootest/ooRexx/base/bif/` through `rexx_exec`'s public entry
//! point, and checks each the way the ooTest framework's own `assertSame`
//! does.
//!
//! # Two populations, not one
//!
//! A **value row** is one `self~assertSame(A, B)` in a body with no
//! `expectSyntax`. It becomes a four-part program -- the `NUMERIC` settings in
//! force, the body's assignment prelude, `say A`, `say B` -- and passes when
//! the two output lines are byte-identical. Byte-identical rather than asking
//! `rexx_exec` to evaluate `==` itself, for `assertions.rs`'s reason: the
//! rendering of a number is what this is checking, and making the executor's
//! own comparison operator the judge would put the code under test in the
//! judge's seat.
//!
//! A **raise row** stands for a body that opens `self~expectSyntax(major.sub)`.
//! There the framework's trap is a frame up (`OOREXXUNIT.CLS:1563`) and the
//! raise abandons the body, so `assertSame` is never entered and its second
//! argument is never compared to anything. Such a row's program evaluates the
//! call's arguments in order and passes when the program raises exactly
//! `major.sub` -- major and sub together, so a row expecting `40.5` cannot be
//! satisfied by `40.9`. See `rexx_extract::bif`'s own doc for the mechanism and
//! for why emitting these as value rows would be confidently backwards.
//!
//! # A measurement, not a threshold
//!
//! This harness's headline -- how many rows pass -- is **reported and not
//! gated**. `base/bif` is the whole builtin surface, including the fifteen
//! names D4 excludes and everything Phase 5 and Phase 7 own, so a number over
//! it is a progress signal rather than a statement about 4c's scope.
//!
//! What *does* have teeth is [`the_exempt_set_matches_the_current_failures`],
//! and it runs under a plain `cargo test` rather than behind [`GATE_ENV`].
//! That is deliberate: a measurement nobody gates on, policed by an assertion
//! nobody runs, is coverage that does not exist. The set assertion is checked
//! in both directions -- a listed row that starts passing is as red as an
//! unlisted row that starts failing -- so an improvement shows up in a diff
//! instead of being quietly absorbed.
//!
//! # The attribution column is derived
//!
//! For a row that fails **loudly**, `rust/corpus/bif-exempt.txt`'s second
//! column is the owner string `rexx-exec`'s own message carries
//! (`instruction_owner` / `expr_owner`), re-read on every run, so the file
//! cannot drift from those tables. The same two limits apply as to
//! `keyword-exempt.txt`: a derived owner says what a row hits *first*, not
//! what would make it pass, and the non-loud categories
//! ([`RowOutcome::attribution`]'s constants) are compared against a file
//! holding the same constant, where what still has teeth is *membership*.
//!
//! # REPORT vs STRICT
//!
//! [`GATE_ENV`] switches an always-green progress report into a run that also
//! fails on an unaccounted row, matching `corpus.rs`'s `REXX_CORPUS_GATE`,
//! `assertions.rs`'s `REXX_ASSERTIONS_GATE` and `keyword_assertions.rs`'s
//! `REXX_KEYWORD_GATE`. `emit_uncaptured` pipes the report through a child
//! process whose stderr is inherited, because a `println!` inside a `#[test]`
//! reaches libtest's thread-local capture sink and not the terminal; see
//! `corpus.rs`'s module doc for the measurement behind that.

mod watchdog;

use rayon::prelude::*;
use rexx_exec::{NOT_IMPLEMENTED_EXIT, Outcome};
use rexx_extract::bif::{DropReason, RaiseRow, extract_bif};
use rexx_extract::keyword::count_assert_same;
use rexx_extract::{AssertionRow, Form, RaiseExpectation, find_test_groups};
use std::collections::BTreeMap;
use std::env;
use std::fmt::Write as _;
use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// The path every row is reported under. There is no real file behind one --
/// the program is assembled from one `.testGroup` assertion -- so this is a
/// label, in the same spirit as `assertions.rs`'s `ROW_PATH`.
///
/// It carries a `.` in its basename on purpose:
/// [`parse_condition_number`]'s first-line rejection turns on the reported
/// path not parsing as a bare integer.
const ROW_PATH: &str = "/nonexistent/bif-row.rex";

/// Env var that flips this test from a progress report into a run that fails
/// on an unaccounted row. Named separately from the other three gates because
/// the four measure independent things.
const GATE_ENV: &str = "REXX_BIF_GATE";

/// The ooTest revision this harness's committed exempt set was measured at.
/// `ootest/` is git-ignored and is an SVN working copy, not checked-in test
/// data, so it can move with nothing in this repository changing; read it back
/// with `svn info ootest`.
const OOTEST_REVISION: &str = "r13178";

fn gate_mode() -> bool {
    match env::var(GATE_ENV) {
        Ok(value) => !value.is_empty() && value != "0",
        Err(_) => false,
    }
}

fn suite_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../ootest/ooRexx/base/bif")
}

fn exempt_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus/bif-exempt.txt")
}

/// One row, either kind, with the identity the exempt set keys on.
struct Row {
    /// `GROUP::METHOD#N`, `N` being the 1-based position of this row among the
    /// rows its own method yielded.
    ///
    /// The occurrence number is not decoration: a `base/bif` method routinely
    /// carries several `assertSame` calls, and `COPIES.testGroup` writes the
    /// same two operand texts in more than one of them, so `(group, method,
    /// expr, expected)` is not a key.
    key: String,
    kind: RowKind,
}

enum RowKind {
    Value(AssertionRow),
    Raise(RaiseRow),
}

/// Every row in the suite, in sorted file order, plus the per-reason
/// accounting for the `assertSame` calls that did not become one.
///
/// The drop counts are reported here and **pinned** on the extractor's own
/// side (`rexx-extract/tests/extract_bif.rs`); a second copy of the literals
/// would be one more thing to drift rather than a cross-check.
fn collect() -> (Vec<Row>, BTreeMap<DropReason, (usize, usize)>, usize) {
    let dir = suite_root();
    let groups = find_test_groups(&dir);
    assert!(
        !groups.is_empty(),
        "no .testGroup files under {} -- suite_root points at the wrong directory, or the \
         ootest checkout is missing base/bif entirely",
        dir.display()
    );

    let mut rows = Vec::new();
    let mut dropped: BTreeMap<DropReason, (usize, usize)> = BTreeMap::new();
    let mut calls = 0usize;
    for path in &groups {
        let bytes =
            fs::read(path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
        let source = String::from_utf8_lossy(&bytes);
        let group = path.file_stem().and_then(|s| s.to_str()).unwrap_or("group");
        let extraction = extract_bif(group, &source);
        calls += count_assert_same(&source);
        for blocked in &extraction.blocked {
            let entry = dropped.entry(blocked.reason).or_default();
            entry.0 += 1;
            entry.1 += blocked.dropped;
        }
        let mut seen: BTreeMap<String, usize> = BTreeMap::new();
        for row in extraction.rows {
            let key = next_key(&mut seen, &row.group, &row.method);
            rows.push(Row {
                key,
                kind: RowKind::Value(row),
            });
        }
        for row in extraction.raises {
            let key = next_key(&mut seen, &row.group, &row.method);
            rows.push(Row {
                key,
                kind: RowKind::Raise(row),
            });
        }
    }
    assert!(
        !rows.is_empty(),
        "extract_bif produced no rows at all -- that is an extraction defect, not an empty pass"
    );
    (rows, dropped, calls)
}

fn next_key(seen: &mut BTreeMap<String, usize>, group: &str, method: &str) -> String {
    let stem = format!("{group}::{method}");
    let n = seen.entry(stem.clone()).or_insert(0);
    *n += 1;
    format!("{stem}#{n}")
}

/// A program that establishes one row's state and then `SAY`s each of
/// `clauses` in order.
///
/// `NUMERIC DIGITS`/`FORM` first, always, and never a shared default: a row
/// evaluated at the wrong precision can render an answer that still happens to
/// match, and would then pass while testing the wrong thing.
fn program(digits: u32, form: Form, prelude: &[String], clauses: &[&str]) -> Vec<u8> {
    let mut text = String::new();
    writeln!(text, "numeric digits {digits}").unwrap();
    writeln!(
        text,
        "numeric form {}",
        match form {
            Form::Scientific => "scientific",
            Form::Engineering => "engineering",
        }
    )
    .unwrap();
    for line in prelude {
        writeln!(text, "{line}").unwrap();
    }
    for clause in clauses {
        writeln!(text, "say {clause}").unwrap();
    }
    text.into_bytes()
}

/// What running one row decided.
#[derive(Debug)]
enum RowOutcome {
    /// A value row whose two `SAY` lines were byte-identical, or a raise row
    /// whose program raised exactly the expected major and sub.
    Pass,
    /// A value row whose two lines both rendered and differ.
    Mismatch,
    /// A raise row whose program did not raise the expected condition: either
    /// it did not raise at all, or it raised a *different* `major.sub`.
    /// Checked as a pair, so a row expecting `40.5` cannot be satisfied by
    /// `40.9`.
    RaiseMismatch,
    /// Hit [`NOT_IMPLEMENTED_EXIT`]. `construct` is what the program ran into
    /// first; `owner` is the phase `rexx-exec`'s own tables name for it, which
    /// is the separate question of what would actually unblock it.
    Blocked {
        construct: String,
        owner: Option<String>,
    },
    /// A real condition escaped a value row the ooTest suite asserts passes,
    /// or a raise row's stderr could not be read back as a `major.sub`.
    Anomaly { detail: String },
}

impl RowOutcome {
    /// The phase that would make this row pass, in the vocabulary the
    /// committed exempt file uses. `None` for a passing row.
    fn attribution(&self) -> Option<String> {
        match self {
            RowOutcome::Pass => None,
            RowOutcome::Blocked { owner, construct } => Some(
                owner
                    .clone()
                    .unwrap_or_else(|| format!("UNATTRIBUTED:{construct}")),
            ),
            RowOutcome::Mismatch => Some("MISMATCH".to_string()),
            RowOutcome::RaiseMismatch => Some("RAISE-MISMATCH".to_string()),
            RowOutcome::Anomaly { .. } => Some("ANOMALY".to_string()),
        }
    }
}

/// Splits a `rexx-exec: X is not implemented (OWNER)` line into `X` and
/// `OWNER`. The owner is optional: `Loud::compound_expose` deliberately
/// carries none, because nothing has been scheduled to build what it needs.
fn parse_loud(stderr: &[u8]) -> (String, Option<String>) {
    let text = String::from_utf8_lossy(stderr);
    const MARKER: &str = "rexx-exec: ";
    const SUFFIX: &str = " is not implemented";
    let Some(after) = text.find(MARKER).map(|at| &text[at + MARKER.len()..]) else {
        return ("<unnamed>".to_string(), None);
    };
    let Some(end) = after.find(SUFFIX) else {
        return ("<unnamed>".to_string(), None);
    };
    let construct = after[..end].to_string();
    let owner = after[end + SUFFIX.len()..]
        .trim_start()
        .strip_prefix('(')
        .and_then(|rest| rest.split(')').next())
        .map(str::to_string);
    (construct, owner)
}

fn evaluate(row: &RowKind) -> RowOutcome {
    match row {
        // **Two runs, not one program printing two lines.** A single program
        // whose output is split on `\n` cannot compare operands that are
        // themselves binary: `BITAND.testGroup`'s expected values include
        // `'0A'x`, so its rows print four lines from two `SAY`s and every one
        // of them reads as a malformed run. Running each operand alone and
        // comparing the two stdouts byte for byte has no such blind spot, and
        // it keeps `assertions.rs`'s reason for not asking `rexx_exec` to
        // evaluate `==` itself: the rendering is what is under test, so the
        // executor's own comparison operator must not be the judge.
        RowKind::Value(r) => {
            let left = watchdog::run_bounded(
                ROW_PATH,
                program(r.digits, r.form, &r.prelude, &[&r.expr]),
                rexx_exec::Invocation::none(),
            );
            let right = watchdog::run_bounded(
                ROW_PATH,
                program(r.digits, r.form, &r.prelude, &[&r.expected]),
                rexx_exec::Invocation::none(),
            );
            classify_value(left, right)
        }
        RowKind::Raise(r) => {
            let operands: Vec<&str> = r.operands.iter().map(String::as_str).collect();
            classify_raise(
                r.expect,
                watchdog::run_bounded(
                    ROW_PATH,
                    program(r.digits, r.form, &r.prelude, &operands),
                    rexx_exec::Invocation::none(),
                ),
            )
        }
    }
}

/// The classification step for a value row, apart from running anything, so
/// the constructed witnesses below can reuse it.
fn classify_value(left: Outcome, right: Outcome) -> RowOutcome {
    for side in [&left, &right] {
        if side.exit_code == NOT_IMPLEMENTED_EXIT {
            let (construct, owner) = parse_loud(&side.stderr);
            return RowOutcome::Blocked { construct, owner };
        }
    }
    for side in [&left, &right] {
        if side.exit_code != 0 {
            return RowOutcome::Anomaly {
                detail: format!(
                    "exit {} (a real condition escaped a row the oracle's own suite asserts \
                     passes); stderr={}",
                    side.exit_code,
                    excerpt(&side.stderr)
                ),
            };
        }
    }
    if left.stdout == right.stdout {
        RowOutcome::Pass
    } else {
        RowOutcome::Mismatch
    }
}

fn classify_raise(expect: RaiseExpectation, outcome: Outcome) -> RowOutcome {
    if outcome.exit_code == NOT_IMPLEMENTED_EXIT {
        let (construct, owner) = parse_loud(&outcome.stderr);
        return RowOutcome::Blocked { construct, owner };
    }
    if outcome.exit_code == 0 {
        return RowOutcome::RaiseMismatch;
    }
    let Some((major, sub)) = parse_condition_number(&outcome.stderr) else {
        return RowOutcome::Anomaly {
            detail: format!(
                "exit {} but no parseable \"Error major.sub:\" line in stderr: {}",
                outcome.exit_code,
                excerpt(&outcome.stderr)
            ),
        };
    };
    if major == expect.major && sub == expect.sub {
        RowOutcome::Pass
    } else {
        RowOutcome::RaiseMismatch
    }
}

/// Finds `major`/`sub` in the oracle-format report's second line, `Error
/// <major>.<sub>:  <message>.`.
///
/// The report's *first* line, `Error <major> running <path> line <n>:  ...`,
/// also starts with `Error ` but never parses this way: its segment before the
/// first `:` is `<major> running <path> line <n>`, whose half before the first
/// `.` is non-numeric text. Duplicated from `assertions.rs` rather than shared,
/// because two integration tests in the same `tests/` directory are separate
/// binaries and neither can `mod` the other.
fn parse_condition_number(stderr: &[u8]) -> Option<(u32, u32)> {
    let text = String::from_utf8_lossy(stderr);
    for line in text.lines() {
        // `continue`, never `?`: the report's *first* line is the traced clause
        // (`   4 *-* say ABS('a')`), which does not start `Error ` at all, so a
        // short-circuit here abandons the scan before reaching the line that
        // carries the number and reports every raise as unreadable.
        let Some(rest) = line.strip_prefix("Error ") else {
            continue;
        };
        let Some(colon) = rest.find(':') else {
            continue;
        };
        let Some((major, sub)) = rest[..colon].split_once('.') else {
            continue;
        };
        if let (Ok(major), Ok(sub)) = (major.trim().parse::<u32>(), sub.trim().parse::<u32>()) {
            return Some((major, sub));
        }
    }
    None
}

fn excerpt(bytes: &[u8]) -> String {
    const BOUND: usize = 200;
    let text = String::from_utf8_lossy(bytes);
    if text.chars().count() > BOUND {
        format!("{}...", text.chars().take(BOUND).collect::<String>())
    } else {
        text.into_owned()
    }
}

/// The committed exempt set, `GROUP::METHOD#N -> unblocked_by`.
fn committed_exempt() -> BTreeMap<String, String> {
    let path = exempt_path();
    let text =
        fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    let mut out = BTreeMap::new();
    for line in text.lines() {
        let line = line.trim_end();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let (key, attribution) = line.split_once('\t').unwrap_or_else(|| {
            panic!(
                "{}: line is not `GROUP::METHOD#N<TAB>unblocked_by`: {line:?}",
                path.display()
            )
        });
        if let Some(previous) = out.insert(key.to_string(), attribution.to_string()) {
            panic!(
                "{}: {key} is listed twice ({previous:?} and {attribution:?})",
                path.display()
            );
        }
    }
    out
}

/// Polices the committed exempt set itself, in every mode, independent of
/// [`GATE_ENV`]: the current failure set must equal the committed one exactly,
/// attribution included.
///
/// Both directions matter and neither is the "real" one. A listed row that
/// starts passing means the exemption is stale, and the fix is to edit the
/// file -- which shows up in a diff -- not for the harness to stop forgiving it
/// on its own. An unlisted row that starts failing is a regression with nothing
/// accounting for it. And a row whose attribution changes means its blocker
/// moved between phases, which should not be able to happen silently.
///
/// **Not behind [`GATE_ENV`]**, unlike the report below. The headline this
/// harness produces is explicitly a measurement rather than a threshold, so if
/// the set assertion were behind the env var too, nothing anyone runs would
/// police `base/bif` at all.
#[test]
fn the_exempt_set_matches_the_current_failures() {
    let (rows, _, _) = collect();
    let committed = committed_exempt();

    let outcomes: Vec<RowOutcome> = rows.par_iter().map(|row| evaluate(&row.kind)).collect();
    let mut measured: BTreeMap<String, String> = BTreeMap::new();
    for (row, outcome) in rows.iter().zip(outcomes) {
        if let Some(attribution) = outcome.attribution() {
            measured.insert(row.key.clone(), attribution);
        }
    }

    let mut problems = Vec::new();
    for (key, attribution) in &measured {
        match committed.get(key) {
            None => problems.push(format!(
                "{key} is failing ({attribution}) and is not on the committed exempt list"
            )),
            Some(listed) if listed != attribution => problems.push(format!(
                "{key} is listed as {listed:?} but now measures {attribution:?} -- its blocker \
                 moved"
            )),
            Some(_) => {}
        }
    }
    for key in committed.keys() {
        if !measured.contains_key(key) {
            problems.push(format!(
                "{key} now PASSES but is still on the committed exempt list -- remove it"
            ));
        }
    }

    // Bounded, because the set can legitimately be thousands of rows wide and a
    // failure message that long is unreadable; the count above it is the figure
    // that matters and is never truncated.
    const SHOWN: usize = 40;
    let shown: Vec<&String> = problems.iter().take(SHOWN).collect();
    assert!(
        problems.is_empty(),
        "the committed exempt set ({}) no longer describes reality ({} failing). Measured at \
         ooTest {OOTEST_REVISION}; check `svn info ootest` first if the corpus may have moved.\n\
         {} problem(s), first {}:\n{}",
        committed.len(),
        measured.len(),
        problems.len(),
        shown.len(),
        shown
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// The runner and the report. Always green in REPORT mode; under [`GATE_ENV`]
/// it additionally fails on a row with no matching exempt entry, which is the
/// same condition [`the_exempt_set_matches_the_current_failures`] asserts in
/// one direction.
#[test]
fn bif_assertions_differential() {
    let (rows, dropped, calls) = collect();
    let committed = committed_exempt();

    let mut value_total = 0usize;
    let mut value_pass = 0usize;
    let mut raise_total = 0usize;
    let mut raise_pass = 0usize;
    let mut by_attribution: BTreeMap<String, usize> = BTreeMap::new();
    let mut by_construct: BTreeMap<String, usize> = BTreeMap::new();
    let mut per_group: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    let mut unaccounted = Vec::new();
    let mut anomalies = Vec::new();

    let outcomes: Vec<RowOutcome> = rows.par_iter().map(|row| evaluate(&row.kind)).collect();
    for (row, outcome) in rows.iter().zip(outcomes) {
        let (group, raise) = match &row.kind {
            RowKind::Value(r) => (r.group.clone(), false),
            RowKind::Raise(r) => (r.group.clone(), true),
        };
        let entry = per_group.entry(group).or_insert((0, 0));
        entry.0 += 1;
        if raise {
            raise_total += 1;
        } else {
            value_total += 1;
        }

        if let RowOutcome::Blocked { construct, .. } = &outcome {
            *by_construct.entry(construct.clone()).or_insert(0) += 1;
        }
        if let RowOutcome::Anomaly { detail } = &outcome {
            anomalies.push(format!("  {}: {detail}", row.key));
        }
        match outcome.attribution() {
            None => {
                entry.1 += 1;
                if raise {
                    raise_pass += 1;
                } else {
                    value_pass += 1;
                }
            }
            Some(attribution) => {
                *by_attribution.entry(attribution.clone()).or_insert(0) += 1;
                if committed.get(&row.key) != Some(&attribution) {
                    unaccounted.push(format!("  {}: {attribution}", row.key));
                }
            }
        }
    }

    let gate = gate_mode();
    emit_uncaptured(&build_report(
        calls,
        (value_pass, value_total),
        (raise_pass, raise_total),
        &dropped,
        &by_attribution,
        &by_construct,
        &per_group,
        &anomalies,
        &unaccounted,
        gate,
    ));

    assert!(
        !gate || unaccounted.is_empty(),
        "STRICT ({GATE_ENV}) mode: {} row(s) are failing with no matching entry in the committed \
         exempt list; see the report above.",
        unaccounted.len()
    );
}

#[expect(
    clippy::too_many_arguments,
    reason = "a report formatter with one parameter per measured column; bundling them into a \
              struct used once would move the same list three lines up"
)]
fn build_report(
    calls: usize,
    value: (usize, usize),
    raise: (usize, usize),
    dropped: &BTreeMap<DropReason, (usize, usize)>,
    by_attribution: &BTreeMap<String, usize>,
    by_construct: &BTreeMap<String, usize>,
    per_group: &BTreeMap<String, (usize, usize)>,
    anomalies: &[String],
    unaccounted: &[String],
    gate: bool,
) -> String {
    let banner = "=".repeat(78);
    let mut report = String::new();
    let w = &mut report;
    writeln!(w, "{banner}").unwrap();
    writeln!(
        w,
        "rexx-exec bif row report -- ootest/ooRexx/base/bif @ {OOTEST_REVISION}"
    )
    .unwrap();
    if gate {
        writeln!(w, "mode: STRICT -- {GATE_ENV} is set").unwrap();
    } else {
        writeln!(
            w,
            "*** REPORT MODE. Set {GATE_ENV}=1 to fail on an unaccounted row. ***"
        )
        .unwrap();
    }
    writeln!(
        w,
        "THIS HEADLINE IS A MEASUREMENT, NOT A THRESHOLD -- base/bif covers the whole builtin \
         surface, including the names D4 excludes and everything Phase 5 and Phase 7 own."
    )
    .unwrap();
    writeln!(
        w,
        "{} of {} value rows passing, {} of {} raise rows passing, out of {calls} assertSame calls",
        value.0, value.1, raise.0, raise.1
    )
    .unwrap();

    writeln!(w, "per group (rows passing/total):").unwrap();
    for (group, (total, passing)) in per_group {
        writeln!(w, "  {group:<14} {passing:>5}/{total}").unwrap();
    }

    writeln!(
        w,
        "outside the extracted population, by reason (bodies, assertSame calls):"
    )
    .unwrap();
    for reason in DropReason::ALL {
        let (bodies, count) = dropped.get(reason).copied().unwrap_or((0, 0));
        writeln!(w, "  {:<45} {bodies:>5} {count:>7}", reason.label()).unwrap();
    }

    writeln!(w, "not passing, by what would unblock it:").unwrap();
    for (attribution, count) in by_attribution {
        writeln!(w, "  {attribution:<45} {count}").unwrap();
    }
    writeln!(w, "first construct hit, for a row that failed loudly:").unwrap();
    for (construct, count) in by_construct {
        writeln!(w, "  {construct:<45} {count}").unwrap();
    }
    if !anomalies.is_empty() {
        writeln!(
            w,
            "anomalies ({}) -- neither a gap nor a comparison this harness can read:",
            anomalies.len()
        )
        .unwrap();
        for line in anomalies.iter().take(20) {
            writeln!(w, "{line}").unwrap();
        }
    }
    if !unaccounted.is_empty() {
        writeln!(
            w,
            "NOT on the committed exempt list ({}) -- what STRICT would fail on:",
            unaccounted.len()
        )
        .unwrap();
        for line in unaccounted.iter().take(40) {
            writeln!(w, "{line}").unwrap();
        }
    }
    writeln!(w, "{banner}").unwrap();
    report
}

/// Writes `text` to the real, process-level stderr. Identical mechanism to
/// `corpus.rs::emit_uncaptured` -- see that file's own doc for why `println!`
/// cannot do this from inside a `#[test]`.
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

/// The falsification proof: perturbing one passing value row's expression must
/// make exactly that row fail.
///
/// Prepends `'ZZZ-FALSIFICATION-MARKER' ||` inside a fresh pair of parentheses
/// around the whole expression rather than appending to its text. Concatenation
/// binds tighter than comparison in Rexx, so appending to an operand that
/// itself contains a top-level comparison would regroup the expression instead
/// of changing its value -- `assertions.rs`'s own falsification proof hit
/// exactly that.
#[test]
fn the_falsification_proof() {
    let (rows, _, _) = collect();
    let row = rows
        .iter()
        .find_map(|row| match &row.kind {
            RowKind::Value(r) if matches!(evaluate(&row.kind), RowOutcome::Pass) => Some(r),
            _ => None,
        })
        .expect("at least one value row passes; if none does, this harness measures nothing");

    let mut perturbed = row.clone();
    perturbed.expr = format!("('ZZZ-FALSIFICATION-MARKER' || ({}))", row.expr);
    assert!(
        matches!(evaluate(&RowKind::Value(perturbed)), RowOutcome::Mismatch),
        "perturbing a row's expression did not make it fail -- the harness is not sensitive to \
         what the rows compare, which is exactly the vacuous-table shape this exists to avoid. \
         Row: {} == {}",
        row.expr,
        row.expected
    );

    // The other direction: the unperturbed row still passes, so the failure
    // above is the perturbation and not something leaking between runs.
    assert!(matches!(
        evaluate(&RowKind::Value(row.clone())),
        RowOutcome::Pass
    ));
}

/// A raise row is satisfied by the exact `major.sub`, not by the major alone.
///
/// Constructed rather than found, because the corpus contains no body that
/// raises the right major with the wrong sub -- and a harness that waited for
/// one would be testing the corpus rather than itself. Both codes below are
/// real `base/bif` ones under the same major: `40.12` is a builtin's argument
/// of the wrong type and `40.5` is a required argument omitted, which is
/// exactly the pair `C2D.testGroup`'s own `expectSyntax` bodies distinguish.
#[test]
fn a_raise_row_needs_the_sub_number_too() {
    let row = RaiseRow {
        group: "SYNTHETIC".to_string(),
        method: "raises_40_12".to_string(),
        prelude: Vec::new(),
        operands: vec!["substr('abc', 'x')".to_string()],
        digits: 9,
        form: Form::Scientific,
        expect: RaiseExpectation { major: 40, sub: 12 },
    };
    assert!(
        matches!(evaluate(&RowKind::Raise(row.clone())), RowOutcome::Pass),
        "the control does not raise 40.12, so the negative below would prove nothing: {:?}",
        evaluate(&RowKind::Raise(row.clone()))
    );

    let wrong_sub = RaiseRow {
        expect: RaiseExpectation { major: 40, sub: 5 },
        ..row.clone()
    };
    assert!(
        matches!(
            evaluate(&RowKind::Raise(wrong_sub)),
            RowOutcome::RaiseMismatch
        ),
        "a raise row expecting 40.5 was satisfied by a program raising 40.12 -- the sub number is \
         not being checked"
    );

    let no_raise = RaiseRow {
        operands: vec!["'quiet'".to_string()],
        ..row
    };
    assert!(
        matches!(
            evaluate(&RowKind::Raise(no_raise)),
            RowOutcome::RaiseMismatch
        ),
        "a raise row whose program raised nothing at all was not distinguished from one that \
         raised what it expected"
    );
}

/// The exempt file's `unblocked_by` column is in the vocabulary
/// `phase-4-exclusions.txt` fixes for owner strings, or is one of the four
/// categories [`RowOutcome::attribution`] emits for a row that neither passes
/// nor names a construct. Nothing else, so a typo cannot quietly become a new
/// category that the set-equality test then happily matches against itself.
///
/// Each derived category also carries its exact row count, including the ones
/// standing at zero.
#[test]
fn every_exempt_attribution_is_a_known_phase_or_a_declared_outcome() {
    const PHASES: &[&str] = &["4b", "4c", "Phase 5", "Phase 7"];

    // The count is the whole point of this table, not decoration on it.
    //
    // A phase string names who will do the work and `UNATTRIBUTED:` names the
    // construct that has to be built; these three name neither, so a row
    // carrying one is a divergence nothing owns. Accepting the spelling
    // without pinning the count means a future value divergence can be
    // repaired by adding a row: the set-equality test goes green again, the
    // headline drops by one, and nothing asserts on the headline.
    //
    // `MISMATCH` is the sharp one -- both renderings produced output and they
    // differ, which is the single thing this differential exists to detect --
    // and the only rows of that shape this extraction has ever produced were
    // twelve that three drop reasons then removed. A category pinned at zero
    // fails the first time the corpus grows one, which is exactly when a
    // reader needs to know; `extract_bif.rs`'s `DropReason` table pins its own
    // zeroes for the same reason.
    const DERIVED: &[(&str, usize)] = &[("MISMATCH", 0), ("RAISE-MISMATCH", 0), ("ANOMALY", 3)];

    let names: Vec<&str> = DERIVED.iter().map(|(name, _)| *name).collect();
    let mut counts = vec![0usize; DERIVED.len()];
    for (key, attribution) in committed_exempt() {
        assert!(
            PHASES.contains(&attribution.as_str())
                || names.contains(&attribution.as_str())
                || attribution.starts_with("UNATTRIBUTED:"),
            "{key} is attributed to {attribution:?}, which is none of {PHASES:?} nor {names:?}"
        );
        if let Some(i) = names.iter().position(|name| *name == attribution) {
            counts[i] += 1;
        }
    }
    for (count, (name, expected)) in counts.iter().zip(DERIVED) {
        assert_eq!(
            count,
            expected,
            "{count} rows of {} carry the derived attribution {name}, against the {expected} \
             pinned here. A row with this attribution names no owner and no construct, so it is \
             a divergence no phase is holding: rule on it -- give it a phase, or record it as a \
             KNOWN GAP in phase-4-exclusions.txt -- and move this number deliberately rather \
             than letting the set-equality test absorb it",
            exempt_path().display()
        );
    }
}
