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

//! The machinery a Phase 5 gate table is built out of: the verdict function,
//! the two-engine crate run, the structural channel, the gate modes and the
//! report.
//!
//! A gate table is a committed row set, one probe program per row, and a
//! verdict per row that comes from **running** that program under both
//! interpreters. Nothing here holds a copy of what the oracle answers: the
//! oracle is re-read on the run that uses it, so a row cannot pass by
//! agreeing with a recording of itself.
//!
//! # The verdict function
//!
//! [`verdict`] maps the three-descriptor comparison -- exit status, `stdout`,
//! `stderr` -- onto five cells: [`Verdict::Agree`], [`Verdict::DivergeStatus`],
//! [`Verdict::DivergeStdout`], [`Verdict::DivergeStderr`] and
//! [`Verdict::DivergeBoth`]. Its input is three booleans, so its domain is a
//! cube of eight points, and it is written as eight literal `match` arms over
//! that cube rather than as a chain of `if`s. **That is what makes "mutually
//! exclusive and jointly exhaustive" a compiler property rather than a claim
//! in a comment**: a missing point is a non-exhaustive `match` and a duplicated
//! one is an unreachable pattern, and both are errors here. There is no
//! fallthrough arm and therefore no precedence question --
//! [`the_verdict_function_partitions_the_descriptor_cube`] walks all eight
//! points from the outside and checks the five preimages partition them.
//!
//! # `loud` is not a verdict
//!
//! [`is_loud`] is a predicate on the **crate's own output alone**: this crate
//! declining to answer, at [`rexx_exec::NOT_IMPLEMENTED_EXIT`] with a
//! `rexx-exec: ` line on `stderr`. It says nothing about the oracle, so it is
//! a column of its own that can co-fire with any of the five cells. The
//! distinction it draws is between a gap and a wrong answer: a row that is
//! not `agree` and not loud is this crate answering, confidently, something
//! other than what the oracle answers.
//!
//! # Both engines run, and a disagreement is structural
//!
//! [`run_on_both_engines`] runs the probe twice in process, through
//! `Invocation::with_engine`, and asserts the two agree on all three
//! descriptors **before** any verdict exists. The assertion is unconditional
//! and names the program: the two engines disagreeing is not a fact about the
//! oracle, so routing it through a verdict channel that a gate mode can relax
//! would let it be absorbed silently.
//!
//! `REXX_ENGINE` is deliberately not used. It is read once per process by the
//! `rexx-run` binary, which cannot give two arms inside one `cargo test`
//! process; `Invocation` is `run_program`'s own parameter and is what makes a
//! per-run choice expressible.
//!
//! # Structural versus verdict
//!
//! A [`Structural`] failure -- a missing probe, an oracle run that did not
//! finish, the two engines disagreeing, a derived set differing from its
//! committed file in either direction -- is red in **every** mode.
//! [`assert_no_structural_failures`] is called unconditionally and before any
//! gated assertion. A row without a probe is one of these, never a skip.
//!
//! A verdict failure is red only under [`CORPUS_GATE_ENV`], and then only for
//! a row whose owning phase is closing ([`PHASE_GATE_ENV`]) or already closed
//! ([`CLOSED_PHASES`]). Outside that, a table is a progress report that always
//! exits zero and prints the same text it prints under the gate.
//!
//! # The report reaches a plain `cargo test`
//!
//! [`emit_uncaptured`] writes through a `sh -c 'cat >&2'` child whose `stderr`
//! is inherited, for the reason `corpus.rs`'s module doc gives in full:
//! `println!` inside a `#[test]` goes to a thread-local sink libtest swaps in,
//! so a passing test's own prints are invisible under a plain `cargo test`
//! whatever flags are passed, while a child's inherited descriptor is dup'd
//! from this process's real fd 2 at spawn time.

// This module is written to be linked by more than one gate-table test
// binary, and each links only the part of it that binary uses. A helper used
// by one table and not another is "dead" from the other's point of view,
// which is a fact about Cargo's test-target model rather than about this
// code. `tests/support/oracle.rs` carries the same attribute for the same
// reason.
#![allow(dead_code)]

use std::env;
use std::fmt::Write as _;
use std::fs;
use std::io::Write as _;
use std::path::Path;
use std::process::{Command, Stdio};

use rexx_exec::{Engine, Invocation, Outcome, run_program};

use crate::support::oracle::{
    CppOutcome, StderrComparison, descriptor_diffs_with, wrapped_exit_code,
};

pub mod orx;

/// Which of the three observable channels a row's two sides disagree on.
///
/// Three independent booleans, taken straight from the comparison and not
/// yet collapsed. [`verdict`] is the only thing that collapses them.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Descriptors {
    pub status: bool,
    pub stdout: bool,
    pub stderr: bool,
}

/// One row's verdict: exactly one of these applies to any
/// [`Descriptors`], and which one is decided by [`verdict`].
#[derive(Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub enum Verdict {
    /// All three channels agree byte for byte.
    Agree,
    /// The exit status differs and both byte channels agree.
    DivergeStatus,
    /// `stdout` differs and the exit status and `stderr` agree.
    DivergeStdout,
    /// `stderr` differs and the exit status and `stdout` agree.
    DivergeStderr,
    /// More than one channel differs.
    DivergeBoth,
}

impl Verdict {
    /// The name this verdict is reported and gated under.
    pub fn label(self) -> &'static str {
        match self {
            Verdict::Agree => "agree",
            Verdict::DivergeStatus => "diverge-status",
            Verdict::DivergeStdout => "diverge-stdout",
            Verdict::DivergeStderr => "diverge-stderr",
            Verdict::DivergeBoth => "diverge-both",
        }
    }

    /// Every verdict there is, for a test that needs to enumerate them
    /// without knowing how many there are.
    pub fn all() -> &'static [Verdict] {
        &[
            Verdict::Agree,
            Verdict::DivergeStatus,
            Verdict::DivergeStdout,
            Verdict::DivergeStderr,
            Verdict::DivergeBoth,
        ]
    }
}

/// The five cells, over the cube of three booleans.
///
/// **Eight literal arms, no wildcard.** See the module doc: writing the
/// domain out point by point is what hands exhaustiveness and non-overlap to
/// the compiler, and it is why no cell has to be tried before another.
pub fn verdict(differs: Descriptors) -> Verdict {
    match (differs.status, differs.stdout, differs.stderr) {
        (false, false, false) => Verdict::Agree,
        (true, false, false) => Verdict::DivergeStatus,
        (false, true, false) => Verdict::DivergeStdout,
        (false, false, true) => Verdict::DivergeStderr,
        (true, true, false) => Verdict::DivergeBoth,
        (true, false, true) => Verdict::DivergeBoth,
        (false, true, true) => Verdict::DivergeBoth,
        (true, true, true) => Verdict::DivergeBoth,
    }
}

/// Runs `abs` in process on both engines and returns the outcome they agree
/// on.
///
/// **The assertion is unconditional and names the program.** See the module
/// doc: the two engines disagreeing is a structural failure, and there is no
/// mode in which it is a verdict.
///
/// **In process, and therefore unbounded.** An oracle run is a subprocess and
/// carries a deadline; these two are calls, and nothing can kill them. What
/// stands in for the deadline is a rule outside the code: a probe is run by
/// hand through `rexx-run` under `timeout -s KILL 10` before it is committed.
///
/// `abs` is passed to the executor as-is, already canonicalised by the
/// caller, because a raised condition's report names the program by its
/// absolute dot-normalised path and the oracle prints exactly that.
pub fn run_on_both_engines(abs: &Path) -> Outcome {
    let text = fs::read(abs).unwrap_or_else(|e| panic!("cannot read {}: {e}", abs.display()));
    let path = abs
        .to_str()
        .unwrap_or_else(|| panic!("probe path {} is not valid UTF-8", abs.display()));

    let run = |engine| -> Outcome {
        run_program(path, text.clone(), Invocation::none().with_engine(engine))
    };
    let tree_walker = run(Engine::TreeWalker);
    let ir = run(Engine::Ir);

    let tw_exit = wrapped_exit_code(tree_walker.exit_code);
    let ir_exit = wrapped_exit_code(ir.exit_code);
    assert!(
        tw_exit == ir_exit && tree_walker.stdout == ir.stdout && tree_walker.stderr == ir.stderr,
        "the two engines disagree on {}, which is a structural failure and not a \
         verdict -- no comparison against the oracle is meaningful until they \
         agree.\n  tree-walker: exit={tw_exit} stdout={} stderr={}\n  ir:          \
         exit={ir_exit} stdout={} stderr={}",
        abs.display(),
        excerpt(&tree_walker.stdout),
        excerpt(&tree_walker.stderr),
        excerpt(&ir.stdout),
        excerpt(&ir.stderr),
    );

    tree_walker
}

/// Compares a crate outcome against an oracle outcome on all three channels,
/// with `stderr` compared **raw**.
///
/// **Raw, never normalised, and that is not a preference.** The default
/// [`StderrComparison::Normalized`] is DEVIATION 0 and collapses the run of
/// spaces after a trace line's own marker. Here `stderr` equality is an
/// *input* to [`verdict`], so normalising would silently turn a
/// [`Verdict::DivergeStderr`] row into [`Verdict::Agree`] -- a green row
/// making a byte-for-byte claim that was never checked byte for byte.
///
/// The caller checks the oracle run finished before calling this: the exit
/// status of a killed or crashed process is not an answer to compare.
pub fn compare_raw(crate_side: &Outcome, oracle: &CppOutcome) -> Descriptors {
    let diffs = descriptor_diffs_with(crate_side, oracle, StderrComparison::Raw);
    Descriptors {
        status: diffs.contains(&"exit code"),
        stdout: diffs.contains(&"stdout"),
        stderr: diffs.contains(&"stderr"),
    }
}

/// Whether this crate declined to answer, rather than answering.
///
/// A predicate on the crate's output alone -- see the module doc for why that
/// makes it a column and not a verdict. The exit code and the marker are both
/// required: a program is free to `EXIT` with whatever
/// [`rexx_exec::NOT_IMPLEMENTED_EXIT`] happens to be, and a `rexx-exec: ` line
/// can only come from this crate's own refusal path.
pub fn is_loud(outcome: &Outcome) -> bool {
    wrapped_exit_code(outcome.exit_code) == rexx_exec::NOT_IMPLEMENTED_EXIT
        && String::from_utf8_lossy(&outcome.stderr)
            .lines()
            .any(|line| line.starts_with("rexx-exec: "))
}

/// The construct named in a `rexx-exec: X is not implemented` line, for
/// grouping a report by which gap a row is waiting on. `None` when the crate
/// answered, and for a refusal whose message does not have that shape.
pub fn refused_construct(stderr: &[u8]) -> Option<String> {
    let text = String::from_utf8_lossy(stderr);
    const MARKER: &str = "rexx-exec: ";
    const SUFFIX: &str = " is not implemented";
    let after_marker = &text[text.find(MARKER)? + MARKER.len()..];
    let end = after_marker.find(SUFFIX)?;
    Some(after_marker[..end].to_string())
}

/// One failure that is red in every mode.
///
/// `subject` is what failed -- a row, a probe path, a file -- and `detail`
/// says what about it. Kept as a struct rather than a formatted string so a
/// caller cannot conflate a structural failure with a verdict by matching on
/// prose.
pub struct Structural {
    pub subject: String,
    pub detail: String,
}

/// Fails the test if anything structural happened, whatever the mode.
///
/// Called before any gated assertion, so a report-mode run -- most runs --
/// cannot let a table that lost a probe, or whose engines disagree, pass as
/// a table whose rows merely have not landed yet.
pub fn assert_no_structural_failures(failures: &[Structural]) {
    if failures.is_empty() {
        return;
    }
    let mut listing = String::new();
    for failure in failures {
        writeln!(&mut listing, "  {}: {}", failure.subject, failure.detail).unwrap();
    }
    panic!(
        "structural failures, which are red in every mode and are not verdicts \
         `{CORPUS_GATE_ENV}` could relax:\n{listing}"
    );
}

/// Env var that turns a verdict mismatch into a non-zero exit, the same
/// switch `corpus.rs` reads.
pub const CORPUS_GATE_ENV: &str = "REXX_CORPUS_GATE";

/// Env var naming the phase being closed. Unset means no verdict gating at
/// all; set, it names the one phase whose rows a verdict mismatch reddens,
/// alongside every phase in [`CLOSED_PHASES`].
pub const PHASE_GATE_ENV: &str = "REXX_PHASE_GATE";

/// The phases whose rows stay gated for the rest of the project, without
/// anyone having to set [`PHASE_GATE_ENV`].
///
/// A phase is added here in the commit that closes it, and from then on a
/// regression in one of its rows is red under [`CORPUS_GATE_ENV`] alone.
pub const CLOSED_PHASES: &[&str] = &[];

/// Whether [`CORPUS_GATE_ENV`] is asking for the gate rather than the report.
pub fn corpus_gate() -> bool {
    match env::var(CORPUS_GATE_ENV) {
        Ok(value) => !value.is_empty() && value != "0",
        Err(_) => false,
    }
}

/// The phase [`PHASE_GATE_ENV`] names, if any.
pub fn closing_phase() -> Option<String> {
    match env::var(PHASE_GATE_ENV) {
        Ok(value) if !value.is_empty() => Some(value),
        _ => None,
    }
}

/// Whether a verdict mismatch on a row owned by `phase` is red.
///
/// Both halves are required: the corpus gate says "a mismatch is an exit
/// status" and the phase says "and this row is one whose mismatch counts".
/// A row owned by a phase that is neither closing nor closed is reported and
/// never gated, whatever the corpus gate says.
pub fn verdict_is_gated(phase: &str) -> bool {
    corpus_gate() && (closing_phase().as_deref() == Some(phase) || CLOSED_PHASES.contains(&phase))
}

/// Bounds a byte string to a short, readable, escaped excerpt, so a row's
/// three channels stay diagnosable from the report without reprinting a
/// program's entire output.
pub fn excerpt(bytes: &[u8]) -> String {
    const BOUND: usize = 96;
    let text = String::from_utf8_lossy(bytes);
    let shown: String = text.chars().take(BOUND).collect();
    if shown.len() < text.len() {
        format!("{shown:?}...")
    } else {
        format!("{shown:?}")
    }
}

/// A gate table's report, built as one payload rather than printed line by
/// line, because the whole point is that it crosses to [`emit_uncaptured`]
/// in one piece rather than through libtest's per-call capture.
pub struct Report {
    text: String,
    gate: bool,
}

impl Report {
    /// Opens a report for `title`, with the banner and the mode line
    /// `corpus.rs` uses: the "not the gate" caveat goes on the report itself,
    /// top and bottom, rather than in a doc comment nobody reads at the
    /// moment they see green.
    pub fn new(title: &str) -> Report {
        let gate = corpus_gate();
        let mut report = Report {
            text: String::new(),
            gate,
        };
        report.line(&"=".repeat(78));
        report.line(title);
        if gate {
            let closing = match closing_phase() {
                Some(phase) => format!("closing phase {phase}"),
                None => "no closing phase named, so no verdict is gated".to_string(),
            };
            report.line(&format!(
                "mode: STRICT (the gate) -- {CORPUS_GATE_ENV} is set, {closing}"
            ));
        } else {
            report.line(&format!(
                "*** REPORT MODE -- NOT THE GATE. Set {CORPUS_GATE_ENV}=1 to run this as the gate. ***"
            ));
        }
        report
    }

    /// Whether this report was opened under the gate.
    pub fn gate(&self) -> bool {
        self.gate
    }

    pub fn line(&mut self, text: &str) {
        writeln!(&mut self.text, "{text}").unwrap();
    }

    /// Closes the report with the trailing banner, repeating the caveat in
    /// report mode.
    pub fn finish(mut self) -> String {
        if !self.gate {
            self.line(&format!(
                "*** REPORT MODE -- NOT THE GATE. A non-`agree` row above is a row still \
                 waiting on the task that owns it, not a failing test. Set \
                 {CORPUS_GATE_ENV}=1 with {PHASE_GATE_ENV} to make a phase's rows an exit \
                 status. ***"
            ));
        }
        self.line(&"=".repeat(78));
        self.text
    }
}

/// Writes `text` to the real, process-level stderr, so it reaches the
/// terminal under a plain `cargo test` with no `--nocapture`.
///
/// `sh -c 'cat >&2'` rather than `Command::new("cat")` directly: `cat`'s own
/// stdout has to land on *this process's* real fd 2, which is exactly what a
/// shell's `>&2` does, and reaching the same effect through `Stdio` alone
/// would need a raw-fd constructor this workspace's `unsafe_code = "forbid"`
/// rules out. `Stdio::inherit()` on the child's own stderr is what makes that
/// `>&2` resolve to the real descriptor, upstream of libtest's thread-local
/// capture.
pub fn emit_uncaptured(text: &str) {
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
