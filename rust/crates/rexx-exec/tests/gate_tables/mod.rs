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

use rexx_exec::{Invocation, Outcome, run_program};

use crate::support::oracle::{
    CppOutcome, StderrComparison, descriptor_diff_with, wrapped_exit_code,
};

pub mod orx;

/// Which of the three observable channels a row's two sides disagree on.
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
/// Runs one probe in process and hands back its outcome.
pub fn run_gate_probe(abs: &Path) -> Outcome {
    let text = fs::read(abs).unwrap_or_else(|e| panic!("cannot read {}: {e}", abs.display()));
    let path = abs
        .to_str()
        .unwrap_or_else(|| panic!("probe path {} is not valid UTF-8", abs.display()));

    let outcome = run_program(path, text, Invocation::none());

    // A refusal raises, so this count is zero in any row that produced a
    // measurement; reading it is the cheapest statement that the row measured
    // what it says it measured.
    assert!(
        outcome.chunks_refused == 0,
        "a body of {} was refused by the compiler, so this row did not measure \
         what it reports.",
        abs.display(),
    );

    outcome
}

/// Compares a crate outcome against an oracle outcome on all three channels,
/// with `stderr` compared **raw**.
pub fn compare_raw(crate_side: &Outcome, oracle: &CppOutcome) -> Descriptors {
    let diff = descriptor_diff_with(crate_side, oracle, StderrComparison::Raw);
    Descriptors {
        status: diff.exit_code,
        stdout: diff.stdout,
        stderr: diff.stderr,
    }
}

/// Whether this crate declined to answer, rather than answering.
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
pub struct Structural {
    pub subject: String,
    pub detail: String,
}

/// Fails the test if anything structural happened, whatever the mode.
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
/// `7` is here while the phase is still landing tasks, which the sentence
/// above does not otherwise describe: its corpus subset is committed and its
/// programs agree, so a verdict of its own moving is a regression rather than
/// progress, and every task that implements one refreshes the table.
pub const CLOSED_PHASES: &[&str] = &[
    "5a", "5b", "5c", "5d", "5e", "5f", "5g", "5h", "5i", "5j", "7",
];

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
pub fn verdict_is_gated(phase: &str) -> bool {
    corpus_gate() && (closing_phase().as_deref() == Some(phase) || CLOSED_PHASES.contains(&phase))
}

/// The name a row is reported and gated under when the oracle did not answer
/// its question at all, so no comparison of the two sides means anything.
pub const UNANSWERED: &str = "unanswered";

/// The label a row is reported and tallied under, whether or not it has a
/// verdict.
pub fn verdict_label(verdict: Option<Verdict>) -> &'static str {
    match verdict {
        Some(verdict) => verdict.label(),
        None => UNANSWERED,
    }
}

/// Splits a program's `stdout` into its lines.
pub fn stdout_lines(bytes: &[u8]) -> Vec<&[u8]> {
    if bytes.is_empty() {
        return Vec::new();
    }
    let mut lines: Vec<&[u8]> = bytes.split(|&b| b == b'\n').collect();
    if bytes.last() == Some(&b'\n') {
        lines.pop();
    }
    lines
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
            let closing = closing_phase();
            let mut gated: Vec<&str> = CLOSED_PHASES.to_vec();
            if let Some(phase) = closing.as_deref()
                && !gated.contains(&phase)
            {
                gated.push(phase);
            }
            report.line(&format!(
                "mode: STRICT (the gate) -- {CORPUS_GATE_ENV} is set, verdicts gated for {}",
                gated.join(", ")
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
