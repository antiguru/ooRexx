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

//! Phase 4b's L1 obligation: runs every method body `rexx-extract`'s
//! `keyword` module lifts out of `ootest/ooRexx/base/keyword/` through
//! `rexx_exec`'s public entry point, and checks each `self~assertSame` the
//! way the ooTest framework's own `assertSame` does.
//!
//! # A body, not a row
//!
//! `tests/assertions.rs` runs one two-line program per assertion, because in
//! `base/expressions` an assertion's meaning is fixed by a short assignment
//! prelude. `base/keyword` tests **statements**: an assertion's meaning is
//! the loop, `IF` or `SIGNAL` that ran before it, so the unit here is the
//! whole method body, rewritten once by the extractor and run once. See
//! `rexx_extract::keyword`'s own doc for the rewrite and the measurement
//! behind it.
//!
//! # How a body reports
//!
//! Each `self~assertSame(A, B)` became `say '@@ASSERTSAME n' ((A) == (B))`
//! ([`rexx_extract::keyword::ASSERTION_MARKER`]), so a body's own stdout
//! carries one marker line per assertion **execution** -- several for an
//! assertion inside a loop. A body passes when it exits 0, emits at least
//! one marker, and every marker reads `1`.
//!
//! **"At least one" is load-bearing and is not a formality.** An assertion
//! inside a loop that never runs emits nothing, and a harness that only
//! checked "no marker says 0" would call that a pass having verified
//! nothing -- the exact vacuous shape this project keeps finding in its own
//! instruments. [`RunOutcome::NoAssertionExecuted`] is a distinct,
//! non-passing outcome, and
//! [`a_body_whose_assertions_never_run_is_not_a_pass`] is the constructed
//! witness for it, since no body in the corpus does this today.
//!
//! # The committed exempt set, and why its attribution cannot rot
//!
//! `rust/corpus/keyword-exempt.txt` names every body that does not pass, with
//! what stands between it and passing. For a body that fails **loudly** that
//! column is **derived, not asserted by hand**: it is the owner string
//! `rexx-exec`'s own message carries (`instruction_owner` / `expr_owner`),
//! re-read on every run and compared against the file, so it cannot disagree
//! with the interpreter's own tables.
//!
//! **Two limits on that, both understated by an earlier version of this
//! paragraph.**
//!
//! First, a derived owner says **what blocks the body first**, not what would
//! make it pass. Those differ whenever a second blocker stands behind the
//! first, and here they are known to differ for at least four bodies (see
//! "Bodies that are not their method" below). So a `4c` row means "4c is what
//! it hits today", and the group's pass rate is a **lower bound** on what 4c
//! would leave, not a measure of it.
//!
//! Second, the `defect:` rows are **not** derived: [`RunOutcome::attribution`]
//! maps every [`RunOutcome::AssertionFailed`] to one constant string, so for
//! those rows the set test compares a constant against a file holding the
//! same constant. What still has teeth there is *membership*, not the label:
//! a body that starts failing its assertions and is not already listed goes
//! red as an unaccounted failure, so a new one cannot be quietly absorbed
//! under the existing tag. Anyone adding a second defect class has to split
//! the constant by hand, and nothing here will remind them.
//!
//! # Bodies that are not their method
//!
//! Extraction lifts a body out of its `.testGroup` and runs it alone, which
//! is not always faithful. Four are known not to be, found by running every
//! extracted program under the C++ oracle:
//!
//! * `CALL::test_expression`, `CALL::test_literal`, `CALL::test_on_name` --
//!   the oracle itself fails these at `Error 43, Routine not found` (rc 213),
//!   because the body calls `::routine`s defined elsewhere in the file that a
//!   standalone program does not carry.
//! * `NUMERIC::test_42` -- exits **3** under the oracle, because the body
//!   falls through into its own `dig: Return digits()` and a program's
//!   `RETURN` value becomes its exit status.
//!
//! All four are listed `4c` today only because `rexx-exec` blocks on a
//! builtin first, so nothing here is currently wrong -- but when 4c lands
//! they will not simply start passing, and their labels will need revisiting
//! rather than deleting. The exempt-set test is what forces that: their
//! measured attribution will stop matching the file and go red.
//!
//! [`the_exempt_set_matches_the_current_failures`] asserts the set in both
//! directions, in every mode: a listed body that starts passing is as red as
//! an unlisted body that starts failing. That is `tests/assertions.rs`'s own
//! device, and the reason for it is unchanged -- an improvement should show
//! up in a diff, not be quietly absorbed by a harness that decides for
//! itself what to forgive.
//!
//! # REPORT vs STRICT
//!
//! [`GATE_ENV`] switches an always-green progress report into the phase
//! gate, matching `corpus.rs`'s `REXX_CORPUS_GATE` and `assertions.rs`'s
//! `REXX_ASSERTIONS_GATE`. `emit_uncaptured` pipes the report through a
//! child process whose stderr is inherited, because a `println!` inside a
//! `#[test]` reaches libtest's thread-local capture sink and not the
//! terminal; see `corpus.rs`'s module doc for the fuller argument and the
//! measurement behind it, which is not re-derived here.

use rexx_exec::{NOT_IMPLEMENTED_EXIT, Outcome, run_program};
use rexx_extract::find_test_groups;
use rexx_extract::keyword::{ASSERTION_MARKER, DropReason, KeywordBody, extract_keyword};
use std::collections::BTreeMap;
use std::env;
use std::fmt::Write as _;
use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// The path every rewritten body is reported under. There is no real file
/// behind one -- the program is assembled from a `.testGroup` method -- so
/// this is a label, in the same spirit as `assertions.rs`'s `ROW_PATH`.
const BODY_PATH: &str = "/nonexistent/keyword-body.rex";

/// Env var that flips this test from a progress report into the phase gate.
/// Named separately from the other two gates because the three measure
/// independent things and a caller should be able to run one without the
/// others.
const GATE_ENV: &str = "REXX_KEYWORD_GATE";

/// The ooTest revision this harness's committed exempt set was measured at.
/// `ootest/` is git-ignored and is an SVN working copy, not checked-in test
/// data, so it can move with nothing in this repository changing; read it
/// back with `svn info ootest`.
const OOTEST_REVISION: &str = "r13178";

fn gate_mode() -> bool {
    match env::var(GATE_ENV) {
        Ok(value) => !value.is_empty() && value != "0",
        Err(_) => false,
    }
}

fn suite_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../ootest/ooRexx/base/keyword")
}

fn exempt_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus/keyword-exempt.txt")
}

/// Every extracted body in the suite, in sorted file order.
///
/// The row and drop counts are **not** re-pinned here: they already live on
/// the extractor's own side (`rexx-extract/tests/extract_keyword.rs`), and a
/// second copy would be one more thing to drift rather than a cross-check.
/// What this asserts is only that the extraction is not silently empty.
fn collect_bodies() -> Vec<KeywordBody> {
    collect().0
}

/// Every extracted body, plus the per-reason accounting for the calls that
/// did **not** become one.
///
/// The report shows both halves because they answer different questions and
/// a reader given only the second would mistake the pass rate's denominator
/// for the group's assertion count. The counts themselves are pinned on the
/// extractor's side (`rexx-extract/tests/extract_keyword.rs`); here they are
/// reported, not asserted.
fn collect() -> (Vec<KeywordBody>, BTreeMap<DropReason, (usize, usize)>) {
    let dir = suite_root();
    let groups = find_test_groups(&dir);
    assert!(
        !groups.is_empty(),
        "no .testGroup files under {} -- suite_root points at the wrong directory, or the \
         ootest checkout is missing base/keyword entirely",
        dir.display()
    );

    let mut bodies = Vec::new();
    let mut dropped: BTreeMap<DropReason, (usize, usize)> = BTreeMap::new();
    for path in &groups {
        let bytes =
            fs::read(path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
        let source = String::from_utf8_lossy(&bytes);
        let group = path.file_stem().and_then(|s| s.to_str()).unwrap_or("group");
        let extraction = extract_keyword(group, &source);
        for blocked in &extraction.blocked {
            let entry = dropped.entry(blocked.reason).or_default();
            entry.0 += 1;
            entry.1 += blocked.dropped;
        }
        bodies.extend(extraction.bodies);
    }
    assert!(
        !bodies.is_empty(),
        "extract_keyword produced no bodies at all -- that is an extraction defect, not an \
         empty pass"
    );
    (bodies, dropped)
}

/// What running one body decided.
#[derive(Debug)]
enum RunOutcome {
    /// Exited 0, emitted at least one marker, and every marker read `1`.
    ///
    /// `verified` is how many **distinct** `self~assertSame` calls actually
    /// ran, counted by marker index, not how many the body contains and not
    /// how many marker lines it printed. Those three differ: an assertion
    /// inside a loop prints once per pass (730 lines across the passing
    /// bodies today, against 713 distinct assertions), and an assertion in a
    /// branch that is not taken prints nothing at all. Crediting the static
    /// count would report an unexecuted assertion as verified; crediting the
    /// line count would report one assertion as several.
    Pass { verified: usize },
    /// Emitted markers and at least one read `0`: an assertion the ooTest
    /// suite asserts holds does not hold here.
    AssertionFailed { failed: usize, total: usize },
    /// Hit [`NOT_IMPLEMENTED_EXIT`]. `construct` is what the body ran into
    /// first; `owner` is the phase `rexx-exec`'s own tables name for it,
    /// which is the separate question of what would actually unblock it.
    Blocked {
        construct: String,
        owner: Option<String>,
    },
    /// Exited non-zero for a reason other than a loud gap: a real condition
    /// escaped a body the ooTest suite asserts passes.
    Raised { detail: String },
    /// Exited 0 having emitted no marker at all. Not a pass: see the module
    /// doc.
    NoAssertionExecuted,
}

impl RunOutcome {
    /// The phase that would make this body pass, in the vocabulary the
    /// committed exempt file uses. `None` for a passing body.
    fn attribution(&self) -> Option<String> {
        match self {
            RunOutcome::Pass { .. } => None,
            RunOutcome::Blocked { owner, construct } => Some(
                owner
                    .clone()
                    .unwrap_or_else(|| format!("UNATTRIBUTED:{construct}")),
            ),
            RunOutcome::AssertionFailed { .. } => {
                Some("defect:compound-do-control-variable".to_string())
            }
            RunOutcome::Raised { .. } => Some("RAISED".to_string()),
            RunOutcome::NoAssertionExecuted => Some("NO-ASSERTION-EXECUTED".to_string()),
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

/// Runs one body and classifies what happened.
fn evaluate(body: &KeywordBody) -> RunOutcome {
    classify(run_program(BODY_PATH, body.program.clone().into_bytes()))
}

/// The classification step alone, so the constructed witnesses below can
/// reuse it without duplicating the exit-code logic.
fn classify(outcome: Outcome) -> RunOutcome {
    if outcome.exit_code == NOT_IMPLEMENTED_EXIT {
        let (construct, owner) = parse_loud(&outcome.stderr);
        return RunOutcome::Blocked { construct, owner };
    }
    if outcome.exit_code != 0 {
        return RunOutcome::Raised {
            detail: format!(
                "exit {}; stderr={}",
                outcome.exit_code,
                excerpt(&outcome.stderr)
            ),
        };
    }

    let stdout = String::from_utf8_lossy(&outcome.stdout);
    let markers: Vec<&str> = stdout
        .lines()
        .filter(|line| line.starts_with(ASSERTION_MARKER))
        .collect();
    if markers.is_empty() {
        return RunOutcome::NoAssertionExecuted;
    }
    // Each marker line is `@@ASSERTSAME <n> <0|1>`, the `<0|1>` being what
    // the body's own `==` produced. A line that does not end ` 1` is a
    // failed assertion whether it ends ` 0` or anything else, so this does
    // not have to trust the comparison to render only those two.
    let failed = markers.iter().filter(|line| !line.ends_with(" 1")).count();
    if failed == 0 {
        // Distinct assertion indices, so a loop's repeats collapse to the
        // one assertion they re-run. `@@ASSERTSAME <n> <0|1>` -- take `<n>`.
        let verified: std::collections::BTreeSet<&str> = markers
            .iter()
            .filter_map(|line| line.split_whitespace().nth(1))
            .collect();
        RunOutcome::Pass {
            verified: verified.len(),
        }
    } else {
        RunOutcome::AssertionFailed {
            failed,
            total: markers.len(),
        }
    }
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

/// The committed exempt set, `GROUP::METHOD -> unblocked_by`.
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
                "{}: line is not `GROUP::METHOD<TAB>unblocked_by`: {line:?}",
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

/// The current failures, derived by running everything: `GROUP::METHOD ->
/// unblocked_by`.
fn measured_failures() -> (BTreeMap<String, String>, usize, usize) {
    let bodies = collect_bodies();
    let total_bodies = bodies.len();
    let mut failures = BTreeMap::new();
    let mut passing_assertions = 0usize;
    for body in &bodies {
        let outcome = evaluate(body);
        match outcome.attribution() {
            None => {
                if let RunOutcome::Pass { verified } = outcome {
                    passing_assertions += verified;
                }
            }
            Some(attribution) => {
                failures.insert(format!("{}::{}", body.group, body.method), attribution);
            }
        }
    }
    (failures, total_bodies, passing_assertions)
}

/// Polices the committed exempt set itself, in every mode, independent of
/// [`GATE_ENV`]: the current failure set must equal the committed one
/// exactly, attribution included.
///
/// Both directions matter and neither is the "real" one. A listed body that
/// starts passing means the exemption is stale, and the fix is to edit the
/// file -- which shows up in a diff -- not for the harness to stop forgiving
/// it on its own. An unlisted body that starts failing is a regression with
/// nothing accounting for it. And a body whose attribution changes means its
/// blocker moved between phases, which is a fact about the plan and should
/// not be able to happen silently.
#[test]
fn the_exempt_set_matches_the_current_failures() {
    let (measured, _, _) = measured_failures();
    let committed = committed_exempt();

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

    assert!(
        problems.is_empty(),
        "the committed exempt set ({}) no longer describes reality ({} failing). Measured at \
         ooTest {OOTEST_REVISION}; check `svn info ootest` first if the corpus may have moved.\n{}",
        committed.len(),
        measured.len(),
        problems.join("\n")
    );
}

/// The runner. REPORT by default, STRICT under [`GATE_ENV`].
///
/// STRICT fails on exactly the same condition
/// [`the_exempt_set_matches_the_current_failures`] asserts, so the gate adds
/// no second notion of correctness; what it adds is the report, and a
/// non-zero exit for a caller that wants one.
#[test]
fn keyword_assertions_differential() {
    let (bodies, dropped) = collect();
    let committed = committed_exempt();

    let mut passing_bodies = 0usize;
    let mut passing_assertions = 0usize;
    let mut total_assertions = 0usize;
    let mut by_attribution: BTreeMap<String, usize> = BTreeMap::new();
    let mut by_construct: BTreeMap<String, usize> = BTreeMap::new();
    let mut per_group: BTreeMap<String, (usize, usize, usize, usize)> = BTreeMap::new();
    let mut unaccounted = Vec::new();
    let mut divergences = Vec::new();
    let mut partial = Vec::new();

    for body in &bodies {
        total_assertions += body.assertions;
        let group = per_group.entry(body.group.clone()).or_insert((0, 0, 0, 0));
        group.0 += 1;
        group.1 += body.assertions;

        let outcome = evaluate(body);
        let key = format!("{}::{}", body.group, body.method);
        if let RunOutcome::Blocked { construct, .. } = &outcome {
            *by_construct.entry(construct.clone()).or_insert(0) += 1;
        }
        match &outcome {
            RunOutcome::AssertionFailed { failed, total } => {
                divergences.push(format!("  {key}: {failed} of {total} assertions failed"));
            }
            // Zero bodies reach this today, so the detail would be dead
            // weight if it were only stored. It is reported for the same
            // reason `corpus.rs` reports an UNCLASSIFIED divergence in full:
            // a condition escaping a body the ooTest suite asserts passes is
            // the one case a reader cannot diagnose from the summary alone.
            RunOutcome::Raised { detail } => divergences.push(format!("  {key}: raised {detail}")),
            _ => {}
        }
        match outcome.attribution() {
            None => {
                let verified = match outcome {
                    RunOutcome::Pass { verified } => verified,
                    _ => 0,
                };
                if verified != body.assertions {
                    partial.push(format!(
                        "  {key}: {verified} of {} assertions actually ran",
                        body.assertions
                    ));
                }
                passing_bodies += 1;
                passing_assertions += verified;
                group.2 += 1;
                group.3 += verified;
            }
            Some(attribution) => {
                *by_attribution.entry(attribution.clone()).or_insert(0) += 1;
                if committed.get(&key) != Some(&attribution) {
                    unaccounted.push(format!("  {key}: {attribution}"));
                }
            }
        }
    }

    let gate = gate_mode();
    emit_uncaptured(&build_report(
        bodies.len(),
        passing_bodies,
        total_assertions,
        passing_assertions,
        &dropped,
        &by_attribution,
        &by_construct,
        &per_group,
        &divergences,
        &partial,
        &unaccounted,
        gate,
    ));

    assert!(
        !gate || unaccounted.is_empty(),
        "STRICT ({GATE_ENV}) mode: {} body/bodies are failing with no matching entry in the \
         committed exempt list; see the report above.",
        unaccounted.len()
    );
}

#[expect(
    clippy::too_many_arguments,
    reason = "a report formatter with one parameter per measured column; bundling them into a \
              struct used once would move the same list three lines up"
)]
fn build_report(
    total_bodies: usize,
    passing_bodies: usize,
    total_assertions: usize,
    passing_assertions: usize,
    dropped: &BTreeMap<DropReason, (usize, usize)>,
    by_attribution: &BTreeMap<String, usize>,
    by_construct: &BTreeMap<String, usize>,
    per_group: &BTreeMap<String, (usize, usize, usize, usize)>,
    divergences: &[String],
    partial: &[String],
    unaccounted: &[String],
    gate: bool,
) -> String {
    let banner = "=".repeat(78);
    let mut report = String::new();
    let w = &mut report;
    writeln!(w, "{banner}").unwrap();
    writeln!(
        w,
        "rexx-exec keyword body report -- ootest/ooRexx/base/keyword @ {OOTEST_REVISION}"
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
    let caveat = if gate {
        ""
    } else {
        " -- REPORT MODE, NOT THE GATE"
    };
    writeln!(
        w,
        "{passing_bodies} of {total_bodies} bodies passing, carrying {passing_assertions} of \
         {total_assertions} assertSame calls{caveat}"
    )
    .unwrap();

    writeln!(
        w,
        "per group (bodies passing/total, assertions passing/total):"
    )
    .unwrap();
    for (group, (bodies, assertions, ok_bodies, ok_assertions)) in per_group {
        writeln!(
            w,
            "  {group:<20} {ok_bodies:>4}/{bodies:<4}  {ok_assertions:>5}/{assertions}"
        )
        .unwrap();
    }

    writeln!(
        w,
        "outside the extracted population, by reason (methods, assertSame calls):"
    )
    .unwrap();
    for reason in DropReason::ALL {
        let (methods, calls) = dropped.get(reason).copied().unwrap_or((0, 0));
        writeln!(w, "  {:<40} {methods:>5} {calls:>7}", reason.label()).unwrap();
    }

    writeln!(w, "not passing, by what would unblock it:").unwrap();
    for (attribution, count) in by_attribution {
        writeln!(w, "  {attribution:<40} {count}").unwrap();
    }
    writeln!(w, "first construct hit, for a body that failed loudly:").unwrap();
    for (construct, count) in by_construct {
        writeln!(w, "  {construct:<40} {count}").unwrap();
    }

    if !divergences.is_empty() {
        writeln!(
            w,
            "assertion failures ({}) -- not gaps: the body ran and disagreed:",
            divergences.len()
        )
        .unwrap();
        for line in divergences {
            writeln!(w, "{line}").unwrap();
        }
    }
    // Zero entries today, and the line is printed only when there are any.
    // A passing body whose assertions did not all run is credited only what
    // ran, so this cannot inflate the headline -- it exists so the
    // difference is visible rather than merely handled.
    if !partial.is_empty() {
        writeln!(
            w,
            "passing bodies that did not run every assertion ({}) -- credited only what ran:",
            partial.len()
        )
        .unwrap();
        for line in partial {
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
        for line in unaccounted {
            writeln!(w, "{line}").unwrap();
        }
    }

    if !gate {
        writeln!(
            w,
            "*** REPORT MODE -- NOT THE GATE. {passing_bodies} of {total_bodies} means {} bodies \
             are not passing yet; it is a progress signal, not a claim that base/keyword is \
             covered. ***",
            total_bodies - passing_bodies
        )
        .unwrap();
    }
    writeln!(w, "{banner}").unwrap();
    report
}

/// Writes `text` to the real, process-level stderr. Identical mechanism to
/// `corpus.rs::emit_uncaptured` -- see that file's own doc for why
/// `println!` cannot do this from inside a `#[test]` and how it was
/// verified. Not shared, because both are integration tests in different
/// crates' `tests/` directories and cannot depend on each other.
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

/// The falsification proof: perturbing one passing body's assertion must
/// make exactly that body fail.
///
/// Perturbs the **left** operand of the first marker's comparison, by
/// prepending `'ZZZ-FALSIFICATION-MARKER' ||` immediately inside its opening
/// parenthesis. Prepending inside the existing parens rather than appending
/// to the raw operand text is what keeps this safe: a body's operands are
/// ordinary program text, concatenation binds tighter than comparison in
/// Rexx, and appending to text that itself contains a top-level comparison
/// would regroup the expression instead of changing its value
/// (`assertions.rs`'s own falsification proof hit exactly that and wraps in
/// parens for the same reason). The parens here are grouping only, so
/// nothing outside them is regrouped.
#[test]
fn the_falsification_proof() {
    let bodies = collect_bodies();
    let body = bodies
        .iter()
        .find(|b| matches!(evaluate(b), RunOutcome::Pass { .. }))
        .expect("at least one body passes; if none does, this harness is measuring nothing");

    let mut perturbed = body.clone();
    perturbed.program = body.program.replacen(
        &format!("'{ASSERTION_MARKER} 1' (("),
        &format!("'{ASSERTION_MARKER} 1' (('ZZZ-FALSIFICATION-MARKER' || "),
        1,
    );
    assert_ne!(
        perturbed.program, body.program,
        "the perturbation did not apply, so the assertion below would prove nothing"
    );
    assert!(
        matches!(
            evaluate(&perturbed),
            RunOutcome::AssertionFailed { failed: 1, .. }
        ),
        "perturbing an operand did not make the body fail -- the harness is not sensitive to \
         what the assertions compare, which is exactly the vacuous-table shape this exists to \
         avoid. Program:\n{}",
        perturbed.program
    );

    // The other direction: the unperturbed body still passes, so the failure
    // above is the perturbation and not something leaking between runs.
    assert!(matches!(evaluate(body), RunOutcome::Pass { .. }));
}

/// A body that emits no marker has verified nothing and must not pass.
///
/// Constructed rather than found: no body in the corpus does this today, and
/// a test that waited for one to appear would be a test of the corpus rather
/// than of this harness. The shape is real -- an assertion inside a loop
/// whose bounds exclude every iteration -- and it is precisely what a
/// "no marker said 0, therefore pass" rule would wave through.
#[test]
fn a_body_whose_assertions_never_run_is_not_a_pass() {
    let never = KeywordBody {
        group: "SYNTHETIC".to_string(),
        method: "never_runs".to_string(),
        program: format!("do i = 1 to 0\n  say '{ASSERTION_MARKER} 1' ((1) == (1))\nend\n"),
        assertions: 1,
    };
    assert!(
        matches!(evaluate(&never), RunOutcome::NoAssertionExecuted),
        "a body that ran to completion without executing its assertion was not distinguished \
         from one that ran it and passed"
    );

    // The adjacent success, so the rule above is pinned to "no assertion
    // ran" rather than to the loop: the same body with a bound that does
    // execute passes.
    let runs = KeywordBody {
        program: never.program.replace("to 0", "to 1"),
        ..never.clone()
    };
    assert!(matches!(evaluate(&runs), RunOutcome::Pass { .. }));
}

/// An assertion inside a loop is checked on **every** pass, not once.
///
/// That is the property the unconditional `SAY` rewrite buys over a
/// conditional one, and it is what `base/keyword` needs: `DO`'s own tests
/// put the assertion inside the loop they are testing. A harness that
/// checked only the last marker, or that counted markers as if they were
/// assertions, would differ here.
#[test]
fn an_assertion_inside_a_loop_is_checked_on_every_pass() {
    let body = KeywordBody {
        group: "SYNTHETIC".to_string(),
        method: "loops".to_string(),
        program: format!("do i = 1 to 5\n  say '{ASSERTION_MARKER} 1' ((i) == (3))\nend\n"),
        assertions: 1,
    };
    assert!(
        matches!(
            evaluate(&body),
            RunOutcome::AssertionFailed {
                failed: 4,
                total: 5
            }
        ),
        "expected 4 of 5 executions to fail (i equals 3 on exactly one pass), got {:?}",
        evaluate(&body)
    );
}

/// The exempt file's `unblocked_by` column is in the vocabulary
/// `phase-4-exclusions.txt` fixes for owner strings, or is an explicit
/// `defect:` tag. Nothing else, so a typo cannot quietly become a new
/// category that the set-equality test then happily matches against itself.
#[test]
fn every_exempt_attribution_is_a_known_phase_or_a_declared_defect() {
    const PHASES: &[&str] = &["4b", "4c", "Phase 5", "Phase 7"];
    for (key, attribution) in committed_exempt() {
        assert!(
            PHASES.contains(&attribution.as_str()) || attribution.starts_with("defect:"),
            "{key} is attributed to {attribution:?}, which is neither one of {PHASES:?} nor a \
             `defect:` tag"
        );
    }
}
