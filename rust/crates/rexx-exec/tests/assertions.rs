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
//! `task-15b-report.md`): 35 rows, all in `Literals.testGroup`, none
//! anywhere else -- `base/expressions` otherwise constructs only the
//! arithmetic, comparison, concatenation and logical forms 4a already
//! evaluates, plus plain literals and variables.
//!
//! **Attribution is "the sub-phase that actually unblocks this row", not
//! "whichever construct its program happens to hit first today", and those
//! two questions have different answers for 2 of the 35.** Both come from
//! `Literals.testGroup`'s `test_string_range`, whose program's *first*
//! `NOT_IMPLEMENTED_EXIT` is `a function call` (`xrange()`, in its own
//! prelude) -- but the very next prelude line is `all~changeStr(.String~cr,
//! "")`, a message send, so even a 4b that implements `Call` would only
//! move this row's first blocker one line later, not make it pass. **All 35
//! rows are unblocked only by Phase 5** (`Message`, per the design spec's
//! own split table: "4a has no general message dispatch"). [`EXEMPT`] is
//! where this fact is committed -- each entry's `unblocked_by` is `"Phase
//! 5"`, including the two whose first-observed blocker is a 4b construct --
//! and `RowOutcome::RuntimeBlocked`'s own `construct` field is kept
//! alongside it, unrenamed, as the separate and genuinely first-hit fact it
//! is: useful for a reader who wants to know what actually happened when
//! this ran, not a stand-in for what would need to change to fix it.
//!
//! # The exempt set, and why STRICT can use it without becoming an escape hatch
//!
//! Criterion 2, taken literally within 4a alone, cannot pass STRICT: it
//! contains rows only Phase 5 can ever satisfy, and 4a's own gate criteria
//! never named Phase 5 as something 4a delivers. [`EXEMPT`] is a **committed,
//! explicit list of the 35 rows** this is true for today (identified by
//! `group`, `method`, source-order `occurrence` within that method --
//! needed because two `test_string_range` rows share byte-identical
//! `expr`/`expected` text -- and the `expr`/`expected` text itself, checked
//! together so a corpus edit that changes a row's text without changing its
//! position cannot silently keep matching the wrong entry). This is
//! criterion 5's own device, reused: its owner arm requires the *set* of
//! out-of-scope variants to be asserted, precisely so a variant that turns
//! out hard cannot be quietly relabelled instead of getting a witness
//! (`docs/superpowers/plans/phase-4-exclusions.txt`'s SET assertion is the
//! same idea again, one level up).
//! [`the_exempt_set_matches_the_current_blocked_rows`] asserts the set
//! unconditionally, in every mode; STRICT
//! (inside [`assertions_differential`]) additionally fails if a row **not**
//! on the list is not passing, or if a row **on** the list *is* passing --
//! the second case means the exemption is stale and the fix is to edit
//! [`EXEMPT`], which shows up in a diff, not to let the harness quietly
//! decide for itself that the row no longer needs forgiving.
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
/// number, and does not independently recount `self~assertSame` occurrences
/// either -- both pins already live on the extractor's own side
/// (`rexx-extract/tests/extract_assertions.rs`'s
/// `base_expressions_yields_the_measured_row_and_blocked_counts` and
/// `every_assert_same_in_base_expressions_is_a_row_or_an_accounted_for_drop`),
/// and duplicating either here would just be a second place that can drift
/// out of sync with the first rather than a real cross-check. What this
/// function's own caller checks is only that the count is nonzero, which is
/// the narrower "not a silently empty extraction" property this crate can
/// state without repeating the extractor's own arithmetic.
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
///
/// All 35 are `"Phase 5"`: `test_hexadecimal`/`test_binary` both open with
/// `tab = .String~tab` (or the corresponding prelude line), a message
/// send, so *every* row in either method blocks there regardless of its
/// own `expr`/`expected` text; `test_string_range` opens with `all =
/// xrange()` (a function call, first-blocked as 4b's) but its very next
/// prelude line is a message send, so implementing 4b's `Call` would not
/// make either of its two rows pass either.
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
        occurrence: 1,
        expr: "\"AB\"",
        expected: "\"41 42\"x",
        unblocked_by: "Phase 5",
    },
    ExemptRow {
        group: "Literals",
        method: "test_hexadecimal",
        occurrence: 2,
        expr: "\"AB\"",
        expected: "\"41  42\"x",
        unblocked_by: "Phase 5",
    },
    ExemptRow {
        group: "Literals",
        method: "test_hexadecimal",
        occurrence: 3,
        expr: "\"AB\"",
        expected: "\"41   42\"x",
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
        method: "test_hexadecimal",
        occurrence: 9,
        expr: "'04'x",
        expected: "\"4\"x",
        unblocked_by: "Phase 5",
    },
    ExemptRow {
        group: "Literals",
        method: "test_hexadecimal",
        occurrence: 10,
        expr: "\"A\"",
        expected: "\"41\"x",
        unblocked_by: "Phase 5",
    },
    ExemptRow {
        group: "Literals",
        method: "test_hexadecimal",
        occurrence: 11,
        expr: "'00'x || \"A\"",
        expected: "\"041\"x",
        unblocked_by: "Phase 5",
    },
    ExemptRow {
        group: "Literals",
        method: "test_hexadecimal",
        occurrence: 12,
        expr: "\"AB\"",
        expected: "\"4142\"x",
        unblocked_by: "Phase 5",
    },
    ExemptRow {
        group: "Literals",
        method: "test_hexadecimal",
        occurrence: 13,
        expr: "'04'x || \"AB\"",
        expected: "\"441 42\"x",
        unblocked_by: "Phase 5",
    },
    ExemptRow {
        group: "Literals",
        method: "test_hexadecimal",
        occurrence: 14,
        expr: "\"ABC\"",
        expected: "\"414243\"x",
        unblocked_by: "Phase 5",
    },
    ExemptRow {
        group: "Literals",
        method: "test_hexadecimal",
        occurrence: 15,
        expr: ".String~xdigit~x2c",
        expected: "'0123456789ABCDEFabcdef'x",
        unblocked_by: "Phase 5",
    },
    ExemptRow {
        group: "Literals",
        method: "test_binary",
        occurrence: 1,
        expr: "\"A\"",
        expected: "\"0100 0001\"b",
        unblocked_by: "Phase 5",
    },
    ExemptRow {
        group: "Literals",
        method: "test_binary",
        occurrence: 2,
        expr: "\"A\"",
        expected: "\"0100  0001\"b",
        unblocked_by: "Phase 5",
    },
    ExemptRow {
        group: "Literals",
        method: "test_binary",
        occurrence: 3,
        expr: "\"A\"",
        expected: "\"0100   0001\"b",
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
    ExemptRow {
        group: "Literals",
        method: "test_binary",
        occurrence: 9,
        expr: "\"AB\"",
        expected: "\"0100 0001 0100 0010\"b",
        unblocked_by: "Phase 5",
    },
    ExemptRow {
        group: "Literals",
        method: "test_binary",
        occurrence: 10,
        expr: "\"AB\"",
        expected: "\"0100 0001  01000010\"b",
        unblocked_by: "Phase 5",
    },
    ExemptRow {
        group: "Literals",
        method: "test_binary",
        occurrence: 11,
        expr: "\"AB\"",
        expected: "\"0100 00010100  0010\"b",
        unblocked_by: "Phase 5",
    },
    ExemptRow {
        group: "Literals",
        method: "test_binary",
        occurrence: 12,
        expr: "\"AB\"",
        expected: "\"0100   000101000010\"b",
        unblocked_by: "Phase 5",
    },
    ExemptRow {
        group: "Literals",
        method: "test_binary",
        occurrence: 13,
        expr: "\"AB\"",
        expected: "\"01000001 0100 0010\"b",
        unblocked_by: "Phase 5",
    },
    ExemptRow {
        group: "Literals",
        method: "test_binary",
        occurrence: 14,
        expr: "\"AB\"",
        expected: "\"01000001  01000010\"b",
        unblocked_by: "Phase 5",
    },
    ExemptRow {
        group: "Literals",
        method: "test_binary",
        occurrence: 15,
        expr: "\"AB\"",
        expected: "\"010000010100  0010\"b",
        unblocked_by: "Phase 5",
    },
    ExemptRow {
        group: "Literals",
        method: "test_binary",
        occurrence: 16,
        expr: "0",
        expected: "\"00110000\"b",
        unblocked_by: "Phase 5",
    },
    ExemptRow {
        group: "Literals",
        method: "test_binary",
        occurrence: 17,
        expr: "0",
        expected: "\"0110000\"b",
        unblocked_by: "Phase 5",
    },
    ExemptRow {
        group: "Literals",
        method: "test_binary",
        occurrence: 18,
        expr: "0",
        expected: "\"110000\"b",
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
///
/// `gate_failures` is shown unconditionally, in REPORT mode too, not only
/// when `gate` is set: each entry is precisely a reason STRICT would fail
/// if it ran right now, and a reader in REPORT mode should not have to
/// re-run with `{GATE_ENV}=1` just to find out whether any exist.
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
///
/// STRICT fails on exactly two shapes, both against the committed
/// [`EXEMPT`] list rather than anything recomputed here: a row **not** on
/// the list that is not passing (an unattributed regression), or a row
/// **on** the list that *is* passing (a stale exemption -- the fix is to
/// remove that row from `EXEMPT`, not for the gate to quietly stop
/// forgiving it on its own).
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
    for row in &rows {
        let occurrence = occurrence_of(&mut occurrence_counts, row);
        let exempt = exempt_entry(row, occurrence);
        let outcome = evaluate_row(row);

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
///
/// Verified to actually catch something rather than merely compile: with
/// `program_for`'s prelude-writing loop commented out (so
/// `test_hexadecimal`/`test_binary`'s `tab = .String~tab` line,
/// `test_string_range`'s `all = xrange()` chain, and (load-bearing for a
/// much larger set) `CONCATENATION`'s own `a`..`g` prelude all stop
/// running), this test failed immediately with a length mismatch, `336`
/// not-passing rows against `EXEMPT`'s `35`. Both directions this device
/// exists to catch actually fired at once: 22 of the 35 committed rows --
/// every one of `test_hexadecimal`/`test_binary`'s pure-literal
/// comparisons with no `self~` in their own `expr`/`expected` text, e.g.
/// `"AB"` vs `"41 42"x` -- started passing outright (a stale exemption, in
/// `assertions_differential`'s own words for it: `"... now PASSES but is
/// still listed in EXEMPT"`), and separately 323 previously-passing rows
/// outside the committed list, almost all of them `CONCATENATION`'s
/// (unset `a`..`g` rendering as their own names, exactly the silent/loud
/// split Task 15's brief and `task-15a-report.md` already measured),
/// started failing with no exemption to explain them. Reverted before
/// committing; see `task-15b-report.md` for the full transcript and the
/// exact counts, taken from `assertions_differential`'s own "EXEMPT-set
/// violations" section while the mutation was live.
#[test]
fn the_exempt_set_matches_the_current_blocked_rows() {
    let (rows, _) = collect_all();
    let mut occurrence_counts: std::collections::HashMap<(String, String), usize> =
        std::collections::HashMap::new();
    let mut still_blocked: Vec<(String, String, usize, String, String)> = Vec::new();
    for row in &rows {
        let occurrence = occurrence_of(&mut occurrence_counts, row);
        if !matches!(evaluate_row(row), RowOutcome::Pass) {
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
