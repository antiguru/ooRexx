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

//! The differential corpus runner: every program named in [`SUBSET_FILES`] --
//! every phase subset file `rust/corpus/` has, read as a union -- run under
//! both interpreters, compared byte for byte on stdout and exit code, and on
//! stderr up to DEVIATION 0's own narrow indent normalisation (see the
//! "DEVIATION 0" section below).
//!
//! **Every phase's subset file is read, not only the current phase's.** A
//! construct a later phase implements cannot have its witness in
//! `phase-4a.txt`, whose own header excludes those constructs by definition,
//! so each phase adds a file and this runner unions them. 4b's Task 1 is the
//! first task to make that true here; before it, this call site was the one
//! `read_subset` caller of four still pinned to a single file, which would
//! have left a 4b witness enumerated by `coverage.rs` and never actually run
//! against the oracle by anything.
//!
//! **That is exactly what had happened again to `phase-4c.txt`.** It was
//! added, `coverage.rs` read it, and this runner did not -- so four programs
//! written as differential witnesses were only ever being *parsed*, and a
//! criterion-1 witness that nothing runs is a witness that cannot fail.
//! Adding the file here found no divergence in any of them, which is the
//! outcome that makes it a correction rather than a change of behaviour.
//!
//! **This is a repeatable progress instrument, not a once-at-the-end gate.**
//! It replaces the hand-run shell loop 4a used after every task (3 of 26
//! programs matching before that phase's Task 9, 9 of 26 after -- a dated
//! record of 4a, not a live figure), and every task that lands should be able
//! to see its own effect by re-running it. See
//! [`REPORT vs STRICT`](#report-vs-strict) below for how the same run serves
//! both that daily use and a phase gate.
//!
//! The forward reference this paragraph used to carry -- "each of the two
//! tasks still to land" -- was 4a's plan state and outlived it (review finding
//! M2). Phase-relative counts of remaining tasks do not belong in a file that
//! outlives the phase; the subset size does not either, which is why every
//! figure this runner reports is computed from `subset.len()` at run time.
//!
//! # The oracle, the memory limit, and the three-descriptor comparison
//!
//! All three live in `tests/support/oracle.rs`, whose own module doc carries
//! them in full: why the oracle path is hardcoded, why a missing binary is a
//! loud failure rather than a skip, how the `ulimit -v` wrapper is built and
//! how it was verified, and what [`support::oracle::descriptor_diffs`]
//! compares. This file wrote them first and was their only user;
//! `tests/builtin_status.rs` needs the identical behaviour, and one copy is
//! what keeps the two harnesses' results comparable.
//!
//! # Owner grouping
//!
//! Every mismatch today is a *clean loud failure*: nothing produces a wrong
//! answer, an unimplemented construct exits [`rexx_exec::NOT_IMPLEMENTED_EXIT`]
//! with `rexx-exec: X is not implemented` on stderr, naming the construct.
//! This is exactly the string the hand-run shell loop grepped for to
//! partition failures by which task owns them, and doing the same thing here
//! is what turns "17 mismatches" into an actionable list rather than a wall of
//! diffs. A mismatch whose stderr does not have that shape is a genuine
//! divergence rather than a gap, and is reported as `UNCLASSIFIED` with a
//! bounded excerpt of all three channels, since that is the case a reader
//! cannot otherwise diagnose from the summary alone.
//!
//! # REPORT vs STRICT
//!
//! [`GATE_ENV`] chosen as the switch, because that is what this phase's own
//! ledger calls the distinction ("the strict switch, so it runs in report
//! mode with a flag the gate flips"): unset or empty or `"0"` is REPORT mode,
//! anything else is STRICT.
//!
//! REPORT (the default) always exits 0, however many programs disagree with
//! the oracle, and prints the count, the full mismatch list and the
//! owner breakdown. **The summary line itself carries the caveat that this is
//! not the gate**, top and bottom, rather than leaving it to a doc comment
//! nobody reads at the moment they see green: a `cargo test` line that means
//! "17 of 26 disagree" is exactly the failure this project keeps finding in
//! its own harnesses, and the worst place to introduce it is the instrument
//! that measures the others.
//!
//! **The report does not rely on `--nocapture`.** `println!`/`eprintln!`
//! inside a `#[test]` write through a *thread-local* sink libtest swaps in for
//! the duration of the test, not through the process's real file descriptor
//! 2 -- so a passing test's own prints are invisible under a plain `cargo
//! test`, `--nocapture` or not, is a flag the reader has to already know to
//! reach for, and documenting that flag as the answer is the same
//! reader-has-to-already-know-to-ask shape as the silent pass this report
//! exists to prevent. `emit_uncaptured` instead pipes the finished report into
//! a `cat >&2` child process whose *stderr is inherited*: a child's inherited
//! descriptor is dup'd from this process's own real fd 2 at spawn time, which
//! the thread-local swap never touches, so the write reaches the terminal (or
//! whatever `cargo test`'s own stderr is connected to) regardless of capture
//! state. Verified, not assumed: a throwaway two-test crate
//! (`a_captured_println_is_invisible` / `a_subprocess_write_bypasses_capture`)
//! run under plain `cargo test`, no flags, showed the `println!` line nowhere
//! in the terminal and the subprocess-written line printed inline between the
//! two `test ... ok` lines.
//!
//! `demonstrate_the_report_reaches_a_plain_cargo_test` below is that same
//! proof, kept in the tree rather than left as a one-off experiment. It cannot
//! observe its *own* real fd 2 from inside itself with no `unsafe` (the only
//! way to intercept a process's own inherited descriptor is a `dup2`-shaped
//! syscall), so it instead re-executes **this same test binary**
//! (`std::env::current_exe`) as a child, asking libtest for exactly one
//! `#[ignore]`d probe test and *not* passing `--nocapture` -- the identical
//! invocation `cargo test` itself uses on every test binary in the workspace.
//! Piping that child's stdout and stderr back (`Command::output`) is legitimate
//! here in a way it would not be for the real report: this process is the
//! child's *parent*, so reading its pipes is not "capturing your own tests'
//! output" but observing a separate process from outside, exactly what a
//! human at a terminal does. Finding the probe's marker in that captured
//! output demonstrates the mechanism survives an ordinary, flagless test run;
//! not finding it would mean this file's whole premise is wrong.
//!
//! STRICT (`REXX_CORPUS_GATE=1 cargo test ...`) runs the identical comparison
//! and fails the test if any program mismatches. The report is written the
//! same way either way, so a gate failure and a report-mode run are equally
//! visible; the assertion failure is what turns the mismatch into a non-zero
//! exit, not what makes it legible.
//!
//! # DEVIATION 0: leading indentation on stderr is normalised
//!
//! The stderr comparison inside `support::oracle::descriptor_diffs` runs
//! both sides through `support::normalize_stderr` first
//! (`tests/support/mod.rs` has the full scope statement and its own
//! negative-control tests). Exit status,
//! stdout, and every other byte of stderr -- the clause text, the line
//! numbers, a value line's own content, and the presence, absence and
//! order of every line -- stay byte-exact; only the run of spaces between
//! a trace line's own 3-byte marker and its content is collapsed. See
//! `docs/superpowers/plans/phase-4-exclusions.txt`'s DEVIATION 0 for why:
//! in short, that run is driven by a mutable counter the oracle itself
//! restores inconsistently on two different loop-exit paths
//! (`BaseDoInstruction.cpp:161` vs `:377`), so matching it byte-for-byte
//! proves nothing the clause-sequence comparison does not already prove.
//!
//! **What still fails if indentation breaks in some other way.** DEVIATION
//! 0 requires a small set of pinned witnesses, at nesting depth <= 3 with
//! no completed loop, that are compared *without* normalisation. Those
//! already existed before this comparison was written, as `rexx-exec/src/
//! run/tests.rs` unit tests asserting an exact `FailureSite`/trace indent --
//! normalisation cannot reach a unit test, since it lives only in this
//! file's and `trace_oracle.rs`'s own comparison functions, so pinning
//! them here is a matter of naming them rather than adding anything new:
//! `one_two_and_three_enclosing_dos_indent_by_two_four_and_six`,
//! `the_corrected_28x_indent_rule_matches_all_fourteen_probed_shapes`, and
//! `an_absorbed_whencases_escaping_false_branch_reports_end_at_its_own_
//! residual_indent`. **Not** `the_indent_after_a_loop_has_already_exited_
//! is_not_left_over_from_it`: that one runs at top level, where the
//! oracle's counter is already clamped at 0 and the correct and incorrect
//! models agree, so it is not a witness for the gap this deviation carves
//! out even though its own name suggests it is.
//!
//! **None of the three pinned witnesses demonstrates the counter defect
//! itself** (review round 1, I2) -- that requires a completed loop pass
//! followed by a failing control test, and all three raise on a *first*
//! iteration, before any pass completes and before the loop the failure
//! sits inside ever ends. They are correct pinned witnesses for lexical
//! nesting depth, which is what DEVIATION 0's own "must still fail"
//! requirement asks for; nothing pinned here exercises the counter gap
//! itself, and no claim to the contrary should be read into their names.
//!
//! # Opting a program out of DEVIATION 0
//!
//! [`RAW_STDERR_COMPARISON`] names corpus programs compared byte-for-byte on
//! `stderr`, through `support::oracle::StderrComparison::Raw`, rather than
//! through DEVIATION 0's normalisation. A program belongs here when the run
//! of spaces normalisation collapses is the thing the program exists to
//! witness, and its own entry says which indent that is.

mod support;

use std::collections::BTreeMap;
use std::env;
use std::fmt::Write as _;
use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use rexx_exec::Outcome;
use support::oracle::{
    Oracle, StderrComparison, descriptor_diffs_with, did_not_finish, wrapped_exit_code,
};

/// Env var that flips this test from a progress report into the phase gate.
/// See the module doc's "REPORT vs STRICT" section.
const GATE_ENV: &str = "REXX_CORPUS_GATE";

fn gate_mode() -> bool {
    match env::var(GATE_ENV) {
        Ok(value) => !value.is_empty() && value != "0",
        Err(_) => false,
    }
}

/// The union of every non-comment, non-blank line across `list_paths`, in
/// first-seen order, each entry appearing once even if two files name the
/// same corpus program. Neither the list nor its count is assumed anywhere
/// else in this file; both come from the files themselves, so the subset can
/// grow or shrink with no change here.
///
/// **Task 0's Step 4.** Was a single-file reader (`&Path`); widened to `&[&Path]`
/// so a later task's own subset file can run *alongside* `phase-4a.txt`
/// rather than replacing it -- see `coverage.rs`'s own copy of this function
/// for the fuller argument. The caller below passes every phase subset file
/// there is: `phase-4a.txt` and `phase-4b.txt` since 4b's Task 1, and
/// `phase-4c.txt` since 4c's Task 9, which found it had been left out.
fn read_subset(list_paths: &[&Path]) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut union = Vec::new();
    for list_path in list_paths {
        let text = fs::read_to_string(list_path)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", list_path.display()));
        for line in text.lines().map(str::trim) {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if seen.insert(line.to_string()) {
                union.push(line.to_string());
            }
        }
    }
    union
}

/// Runs the executor in process, on `path`.
///
/// `path` is passed through as-is, already canonicalised by the caller: a
/// raised condition's report names the program by its absolute,
/// dot-normalised path, the oracle prints exactly that, and `rexx-run`'s own
/// `std::fs::canonicalize` is what makes the two agree. Passing anything else
/// here would make every raising program mismatch on stderr regardless of
/// whether the executor is right.
fn run_rust(path: &Path) -> Outcome {
    let text = fs::read(path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    let path_str = path
        .to_str()
        .unwrap_or_else(|| panic!("corpus path {} is not valid UTF-8", path.display()));
    rexx_exec::run_program(path_str, text, rexx_exec::Invocation::none())
}

/// One corpus program that disagreed with the oracle, or one whose oracle
/// run did not finish.
struct Mismatch {
    rel_path: String,
    /// The construct named in `rexx-exec: X is not implemented`, when the
    /// stderr has that shape. `None` for a genuine divergence and for a
    /// structural failure alike.
    owner: Option<String>,
    reason: String,
    /// `true` for an oracle run that did not finish, `false` for an ordinary
    /// byte divergence. Kept as its own field rather than inferred from
    /// `reason`'s text, because a structural failure is red in every mode
    /// and a divergence is red only under the gate -- `corpus_differential`
    /// asserts on this field unconditionally, before the gated assertion
    /// over the whole list, and a bit that says which kind a `Mismatch` is
    /// cannot be conflated with prose the way a substring match could be.
    structural: bool,
}

/// Pulls `X` out of a `rexx-exec: X is not implemented` line, the same
/// pattern the hand-run shell loop grepped for to partition failures by task.
fn owner_from_stderr(stderr: &[u8]) -> Option<String> {
    let text = String::from_utf8_lossy(stderr);
    const MARKER: &str = "rexx-exec: ";
    const SUFFIX: &str = " is not implemented";
    let after_marker = &text[text.find(MARKER)? + MARKER.len()..];
    let end = after_marker.find(SUFFIX)?;
    Some(after_marker[..end].to_string())
}

/// Bounds a byte string to a short, readable excerpt, so an UNCLASSIFIED
/// divergence's report stays diagnosable without reprinting a program's
/// entire output.
fn excerpt(bytes: &[u8]) -> String {
    const BOUND: usize = 200;
    let text = String::from_utf8_lossy(bytes);
    if text.len() > BOUND {
        format!("{}...", &text[..BOUND])
    } else {
        text.into_owned()
    }
}

/// Corpus programs compared byte-for-byte on `stderr` rather than through
/// DEVIATION 0's normalisation. See the module doc's "Opting a program out
/// of DEVIATION 0".
const RAW_STDERR_COMPARISON: &[&str] = &[
    // Phase 5a (2026-08-17 plan) Task 11: what a list expression and a
    // `DO OVER` trace. Every value line here is indented, and the indents are
    // what the program is for -- the `>A>` per written list position against
    // the `>>>` for the list itself, and the `>K>` a `DO OVER` header emits
    // against the `>=>` each pass emits two columns further in.
    "lang/array_trace.rex",
    // Fix round 1, finding B2: RAISE's own keyword value lines. Each renders an
    // object through `stringValue()` and each is indented, which is what the
    // three programs exist to pin; the two `ADDITIONAL`/`ARRAY` spellings also
    // differ from each other only in their traced element lines.
    "lang/raise_additional_array.rex",
    "lang/raise_array_spelling.rex",
    "lang/raise_keyword_object_traces.rex",
    // Fix round 2: a nested array as a substitution item. Its traced element
    // lines are indented and its report reads the item's own object name, both
    // of which the normaliser would blur.
    "lang/raise_array_nested.rex",
    "lang/raise_additional_nested.rex",
    // Phase 5a Task 7's two traced method activations. The indent a
    // `::METHOD` body's clauses echo at is the thing under test -- 0,
    // whatever the sending clause's own indent was -- and normalisation
    // erases exactly that difference, so these two are the first programs
    // that would pass the default comparison while being wrong.
    "lang/method_trace_invocation.rex",
    "lang/method_trace_nested.rex",
    // The `::ATTRIBUTE` accessor pair. Its `>I>`/`<I<` lines sit at the indent
    // the entries above are here for, and they carry the resolved message
    // name -- `"B"` against `"B="` -- which is the only place a program can
    // read that name itself rather than infer it from which body ran.
    "lang/method_attribute_set_body.rex",
    // Phase 5a (2026-08-17 plan) Task 6: the operator-forwarded native-method
    // frame. Raw mode is what asserts the frame line's own bytes -- its
    // leading whitespace and its absent line number included -- rather than
    // resting on whatever DEVIATION 0's normaliser happens to do to them.
    // Measured, od-verified: the gap after the frame line's own `*-*` marker
    // is exactly one space on both sides today, so normalisation is
    // currently a no-op on every entry below's stderr; raw mode is the
    // comparison that still asserts those bytes rather than one that would
    // pass by coincidence if the gap ever widened.
    "lang/operator_frame_stem_plus.rex",
    "lang/operator_frame_stem_power.rex",
    "lang/operator_frame_stem_prefix_minus.rex",
    // Fix round 1, finding 3: the same forwarded frame reached from a
    // controlled `DO` header's numeric position rather than from an
    // operator directly, one program per position. `Interp::header_number`
    // rounds each position through a real unary `+`, so a stem position
    // carries the identical frame -- and the same raw-mode reasoning above
    // applies to it unchanged.
    "lang/operator_frame_stem_do_initial.rex",
    "lang/operator_frame_stem_do_to.rex",
    "lang/operator_frame_stem_do_by.rex",
    // Fix round 2, finding 1: the frame belongs to the receiver of a
    // forwarded operator whatever step of evaluating it raises, not only its
    // own conversion. Each program below reaches the frame from a different
    // failing step past that conversion -- arithmetic overflow, the power
    // exponent's own range check, a non-logical operand on `&` and on prefix
    // `\`, and a DO header's own range check -- and the same raw-mode
    // reasoning above applies to each unchanged.
    "lang/operator_frame_stem_divide_by_zero.rex",
    "lang/operator_frame_stem_power_exponent_range.rex",
    "lang/operator_frame_stem_logical_and.rex",
    "lang/operator_frame_stem_prefix_not.rex",
    "lang/operator_frame_stem_do_exponent_range.rex",
    // Fix round 3, finding 1: the same forwarded frame reached through a
    // non-strict comparison's own numeric conversion, the operator family
    // whose need for a number -- not whether it is arithmetic -- is what
    // decides whether it reaches this frame at all.
    "lang/operator_frame_stem_compare_overflow.rex",
    // Phase 5a (2026-08-17 plan) Task 14: the frame the required-string
    // protocol's own `REQUEST` activation contributes when a `makeString`
    // reached through it raises. The pair is one frame from a language context
    // against two from a method argument, and the frame lines are indented and
    // carry no line number, which is what raw mode asserts.
    "lang/required_string_make_string_raises.rex",
    "lang/required_string_argument_make_string_raises.rex",
    // Phase 5a (2026-08-17 plan) Task 16: what a `REPLY` leaves on stderr.
    // Both programs are traceback-only, and both carry the two-space-and-more
    // gap of a `*-*` clause line at the indent an owed body reports at -- the
    // run of spaces normalisation collapses. `method_reply`'s 98.936 is
    // reported from the resumed half of the body and `method_reply_twice`'s
    // 98.935 from a clause the first half never reached, so the indent is the
    // only thing separating either report from one raised in the sender.
    "lang/method_reply.rex",
    "lang/method_reply_twice.rex",
];

/// Every entry in [`RAW_STDERR_COMPARISON`] is a line some phase subset file
/// actually names.
///
/// **`check_case` matches by exact string equality against `rel_path`**
/// (`RAW_STDERR_COMPARISON.contains(&rel_path)`), and nothing else checks
/// that an entry corresponds to a real subset program. A path with any
/// spelling difference from its subset line -- a typo, a missing `lang/`
/// prefix, a stray trailing character -- would silently fail that equality
/// check and fall through to the `else` branch, so `check_case` would give
/// it the *normalised* comparison it never asked for: a green run reporting
/// a byte-for-byte claim that was never actually checked byte for byte,
/// which is the exact failure this whole mechanism exists to prevent, one
/// level up. Without this test, only a human reading the diff would catch
/// that.
///
/// **`RAW_STDERR_COMPARISON` holds entries, so this test is load-bearing
/// rather than an iteration over zero rows.** Any entry misspelled relative
/// to its subset line fails this test with the exact path named, where
/// without it the same typo would compile, run, and report a passing
/// byte-for-byte comparison that had silently used the normalised path
/// instead.
#[test]
fn raw_stderr_comparison_only_names_programs_the_subset_actually_runs() {
    let corpus_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus");
    let paths: Vec<PathBuf> = SUBSET_FILES
        .iter()
        .map(|name| corpus_dir.join(name))
        .collect();
    let subset = read_subset(&paths.iter().map(PathBuf::as_path).collect::<Vec<_>>());
    for raw_path in RAW_STDERR_COMPARISON {
        assert!(
            subset.iter().any(|entry| entry == raw_path),
            "{raw_path} is listed in RAW_STDERR_COMPARISON but is not a line in \
             any phase subset file -- check_case would silently give it the \
             normalised comparison instead of the raw one it is supposed to opt \
             into"
        );
    }
}

/// Runs one corpus entry under both interpreters and compares all three
/// observable channels. `None` when they agree.
fn check_case(oracle: &Oracle, corpus_dir: &Path, rel_path: &str) -> Option<Mismatch> {
    let abs = fs::canonicalize(corpus_dir.join(rel_path))
        .unwrap_or_else(|e| panic!("cannot resolve corpus entry {rel_path}: {e}"));

    let rust = run_rust(&abs);
    let cpp = oracle.run(&abs);

    // Checked before `descriptor_diffs_with` calls `cpp.expect_exit_code()`
    // itself: that panic has no `rel_path` in it and fires from inside
    // `support/oracle.rs`, while this file's own report is built from the
    // `Mismatch`es this function returns and printed only after the whole
    // sweep finishes -- so the panic form is not merely less informative
    // here, it never reaches a reader at all. Naming the path is this
    // caller's job because it is the one thing `expect_exit_code` cannot
    // know.
    if did_not_finish(&cpp) {
        return Some(Mismatch {
            rel_path: rel_path.to_string(),
            owner: None,
            reason: format!(
                "the oracle did not finish: {:?} -- a structural failure, not a byte \
                 comparison",
                cpp.termination
            ),
            structural: true,
        });
    }

    let rust_exit = wrapped_exit_code(rust.exit_code);

    // DEVIATION 0 applies to the stderr comparison unless `rel_path` opted
    // out via `RAW_STDERR_COMPARISON` -- see this file's own module doc for
    // the scope and `tests/support/mod.rs` for the normalising function
    // itself.
    let stderr_mode = if RAW_STDERR_COMPARISON.contains(&rel_path) {
        StderrComparison::Raw
    } else {
        StderrComparison::Normalized
    };
    let diffs = descriptor_diffs_with(&rust, &cpp, stderr_mode);
    if diffs.is_empty() {
        return None;
    }

    let owner = owner_from_stderr(&rust.stderr);
    let reason = match &owner {
        Some(construct) => format!(
            "{} differ (loud failure: {construct} is not implemented; rust rc {rust_exit}, \
             oracle rc {})",
            diffs.join(", "),
            cpp.expect_exit_code()
        ),
        // No loud-failure marker: a real divergence rather than a known gap,
        // so give enough of all three channels to diagnose it from the
        // report alone.
        None => format!(
            "{} differ\n      rust:   stdout={:?} stderr={:?} exit={rust_exit}\n      \
             oracle: stdout={:?} stderr={:?} exit={}",
            diffs.join(", "),
            excerpt(&rust.stdout),
            excerpt(&rust.stderr),
            excerpt(&cpp.stdout),
            excerpt(&cpp.stderr),
            cpp.expect_exit_code()
        ),
    };

    Some(Mismatch {
        rel_path: rel_path.to_string(),
        owner,
        reason,
        structural: false,
    })
}

/// Builds the report text. A `String`, not a `println!` stream: the whole
/// point is that this text crosses to [`emit_uncaptured`] as one payload
/// rather than going through libtest's per-call capture at all.
fn build_report(matched: usize, total: usize, mismatches: &[Mismatch], gate: bool) -> String {
    let banner = "=".repeat(78);
    let mut report = String::new();
    let w = &mut report;
    writeln!(w, "{banner}").unwrap();
    writeln!(
        w,
        "rexx-exec differential corpus report -- rust/corpus/{}",
        SUBSET_FILES.join(" + ")
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
    writeln!(w, "{matched} of {total} matching{not_the_gate}").unwrap();

    if !mismatches.is_empty() {
        writeln!(w, "mismatches ({}):", mismatches.len()).unwrap();
        for mismatch in mismatches {
            let owner = mismatch.owner.as_deref().unwrap_or("UNCLASSIFIED");
            writeln!(
                w,
                "  [{owner:<10}] {}: {}",
                mismatch.rel_path, mismatch.reason
            )
            .unwrap();
        }

        let mut by_owner: BTreeMap<&str, usize> = BTreeMap::new();
        for mismatch in mismatches {
            let owner = mismatch.owner.as_deref().unwrap_or("UNCLASSIFIED");
            *by_owner.entry(owner).or_insert(0) += 1;
        }
        writeln!(w, "by owner:").unwrap();
        for (owner, count) in &by_owner {
            writeln!(w, "  {owner}: {count}").unwrap();
        }
    }

    if !gate {
        writeln!(
            w,
            "*** REPORT MODE -- NOT THE GATE. {matched} of {total} matching means {} \
             programs still disagree with the oracle; it is a progress signal for the \
             tasks still landing, not a claim that the phase is done. ***",
            total - matched
        )
        .unwrap();
    }
    writeln!(w, "{banner}").unwrap();
    report
}

/// Writes `text` to the real, process-level stderr, so it reaches the
/// terminal under a plain `cargo test` with no `--nocapture` -- see the
/// module doc's "REPORT vs STRICT" section for why `println!`/`eprintln!`
/// cannot do this from inside a `#[test]`.
///
/// `sh -c 'cat >&2'` rather than `Command::new("cat")` directly: `cat`'s own
/// stdout has to land on *this process's* real fd 2, and redirecting a
/// child's stdout to a specific existing descriptor is exactly what a shell's
/// `>&2` does; reaching for the same effect through `std::process::Stdio`
/// alone would need a raw-fd constructor, which is `unsafe`. The workspace
/// lint is `unsafe_code = "deny"`, so that is a grantable exception rather
/// than a closed door, and it is not worth granting for something a shell
/// builtin already does -- the bar is `rust/CLAUDE.md`'s and the granted set
/// is asserted by `rexx-core/tests/unsafe_sites.rs`. Setting the `Command`'s
/// own `stderr` to `Stdio::inherit()` is what makes that `>&2` resolve to the
/// *real* fd 2:
/// a child's inherited descriptor is dup'd from the parent's at spawn time,
/// upstream of libtest's thread-local capture.
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

/// The runner itself. See the module doc for REPORT vs STRICT and how the
/// oracle and the memory limit are handled.
///
/// Expected result at commit `e0e57825`: 9 of 26 matching, the
/// remaining 17 partitioned as `DO` 10, `TRACE` 4, `SELECT` 2, `IF` 1 --
/// reproduced with a standalone shell loop before this test was written.
/// Tasks implementing `IF` and `SELECT` may move this number out from under a
/// later run; that is expected, not a regression, and the fix is to re-run
/// and record which commit was measured, not to adjust this comment to match
/// a stale number.
///
/// Expected result at commit `a9420630`: **30 of 30 matching**, the subset
/// being `phase-4a.txt`'s 29 programs plus `phase-4b.txt`'s one. Added as a
/// second dated row rather than replacing the first, which is what the
/// paragraph above asks for -- 4b's Task 1 re-ran and had a number, and
/// recording it is the instruction, not merely leaving the older row intact
/// (review's ruling on the dated figure).
///
/// Expected result at commit `27606888`: **47 of 47 matching**, the subset
/// being those 42 plus `phase-4c.txt`'s five, which this call site had not
/// been reading. All five matched on the first run, so the widening moved no
/// number that was standing on anything.
///
/// Expected result at commit `1c519dfc`: **50 of 50 matching**, the subset
/// being `phase-4a.txt`'s 30, `phase-4b.txt`'s 12 and `phase-4c.txt`'s 8. No
/// call site moved between this row and the one above it; what moved is
/// `phase-4c.txt`, which three later 4c tasks added programs to.
///
/// **The commit named is the one the number is true *at*, not the one the
/// change was made against.** An earlier version of this row named
/// `2070cd9d`, this change's parent, where the harness still read two files
/// and reported 42 -- so the row was false as written, in the one way a dated
/// row exists to prevent.
/// The subset files this runner reads, in union order.
///
/// A named constant rather than a literal at the call site so that
/// [`the_differential_reads_every_phase_subset_file`] can assert it against
/// the corpus directory itself.
///
/// **Nothing else here can see a file dropped from this list.** The gate's own
/// assertion is over `mismatches`, and a subset that lost a whole phase has no
/// mismatches to report: measured, with `phase-4c.txt` removed, both plain
/// mode and `REXX_CORPUS_GATE=1` exit 0 and the only thing that moves is the
/// report's own "N of M matching" line, from 50 of 50 to 42 of 42. A number a
/// reader might eyeball is not a check, and criterion 1 of the 4c gate rests
/// on this figure.
const SUBSET_FILES: &[&str] = &[
    "phase-4a.txt",
    "phase-4b.txt",
    "phase-4c.txt",
    "phase-5a.txt",
    "phase-5b.txt",
];

/// The phase subset files that exist in the corpus directory, sorted.
///
/// Read from the directory rather than listed a second time, so the assertion
/// below cannot be satisfied by a copy of [`SUBSET_FILES`] edited in the same
/// change, and so a subset file added later and never wired in here is red
/// rather than silently unrun.
///
/// Duplicated from `collect_stress.rs` and `coverage.rs` rather than shared,
/// for the reason `read_subset` above is duplicated: these are three
/// integration-test binaries and none can `mod` another.
fn phase_subset_files_on_disk() -> Vec<String> {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus");
    let entries =
        fs::read_dir(&dir).unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()));
    let mut names: Vec<String> = entries
        .flatten()
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| name.starts_with("phase-") && name.ends_with(".txt"))
        .collect();
    names.sort();
    names
}

/// The differential reads **every** phase subset file the corpus has.
///
/// The pin on *which files* this runner reads, which is a different question
/// from what any of them contains -- `coverage.rs`'s three
/// `phase_*_subset_matches_the_committed_list` tests pin the contents.
#[test]
fn the_differential_reads_every_phase_subset_file() {
    assert_eq!(
        SUBSET_FILES,
        phase_subset_files_on_disk(),
        "the corpus differential does not read every phase subset file in \
         rust/corpus/. A file missing from SUBSET_FILES is a phase whose \
         programs are never run against the oracle, and the run stays green \
         over whatever is left -- the headline shrinks and nothing asserts on \
         it"
    );
}

#[test]
fn corpus_differential() {
    let oracle = support::oracle::locate();
    let corpus_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus");
    let paths: Vec<PathBuf> = SUBSET_FILES
        .iter()
        .map(|name| corpus_dir.join(name))
        .collect();
    let subset = read_subset(&paths.iter().map(PathBuf::as_path).collect::<Vec<_>>());
    // Covers the empty *union* -- every named file present and every one of
    // them naming nothing -- and nothing beyond it. A union that lost a whole
    // file is still non-empty while the others stand, which is why the pin
    // above exists rather than this guard being widened.
    assert!(
        !subset.is_empty(),
        "the phase subset files named no programs -- that is a corpus defect, \
         not an empty pass"
    );

    let mut mismatches = Vec::new();
    let mut matched = 0usize;
    for rel_path in &subset {
        match check_case(&oracle, &corpus_dir, rel_path) {
            None => matched += 1,
            Some(mismatch) => mismatches.push(mismatch),
        }
    }

    let total = subset.len();
    let gate = gate_mode();
    emit_uncaptured(&build_report(matched, total, &mismatches, gate));

    // Unconditional, and checked before the gated assertion below: a
    // structural failure is red in every mode, per the global constraints,
    // and `gate_mode()` exists to relax a *verdict* comparison, not to
    // decide whether a run that never produced bytes to compare gets
    // noticed. Named separately from `mismatches.is_empty()` so that a
    // report-mode run -- most runs -- cannot let a program that stopped
    // finishing pass as a fully-matching corpus just because the corpus
    // gate itself was not requested.
    let structural: Vec<&str> = mismatches
        .iter()
        .filter(|m| m.structural)
        .map(|m| m.rel_path.as_str())
        .collect();
    assert!(
        structural.is_empty(),
        "{} corpus program(s) did not finish, a structural failure in every \
         mode rather than a verdict `{GATE_ENV}` could relax: {structural:?}. \
         See the report above for which `Termination` each one carries.",
        structural.len()
    );

    assert!(
        !gate || mismatches.is_empty(),
        "STRICT ({GATE_ENV}) mode: {} of {total} corpus programs disagree with \
         the oracle; see the report above for which and why.",
        mismatches.len()
    );
}

/// Not a corpus case. Exists only to be re-executed, alone, by
/// [`demonstrate_the_report_reaches_a_plain_cargo_test`], which is the reason
/// it is `#[ignore]`d: a normal test run must not run it as a case of its
/// own, only as the child process the demonstration spawns.
#[test]
#[ignore = "run only by demonstrate_the_report_reaches_a_plain_cargo_test, as a child process"]
fn probe_emit_uncaptured_marker() {
    emit_uncaptured("EMIT-UNCAPTURED-PROBE-MARKER\n");
}

/// Proves `emit_uncaptured` reaches the terminal under a plain `cargo test`,
/// rather than asserting it should. See the module doc's "REPORT vs STRICT"
/// section for why this cannot observe its own process's fd 2 and instead
/// re-executes this test binary as a child running only
/// [`probe_emit_uncaptured_marker`], with no `--nocapture` -- the same
/// invocation `cargo test` (workspace or single-crate) uses on every test
/// binary. `--include-ignored` is required for exactly one reason: the
/// probe's own `#[ignore]`, which exists so *this* process's normal test run
/// does not also execute it directly.
#[test]
fn demonstrate_the_report_reaches_a_plain_cargo_test() {
    let exe = env::current_exe().expect("a running test binary knows its own path");
    let output = Command::new(exe)
        .args([
            "--exact",
            "probe_emit_uncaptured_marker",
            "--include-ignored",
        ])
        .output()
        .expect("re-executing this test binary against itself");

    let mut combined = output.stdout;
    combined.extend_from_slice(&output.stderr);
    let text = String::from_utf8_lossy(&combined);
    assert!(
        text.contains("EMIT-UNCAPTURED-PROBE-MARKER"),
        "the probe's marker did not reach the child test binary's own captured \
         output under a plain (no --nocapture) run, so emit_uncaptured is not \
         bypassing libtest's capture. Full child output:\n{text}"
    );
}
