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

//! Timing one subprocess, summarising a set of such timings, and putting a
//! distribution-free confidence interval around the median of them.
//!
//! One copy, two callers. `src/bin/rexx-time.rs` is the cold-start timer
//! Phase 0's D2 gate uses; `src/bin/rexx-bench-suite.rs` is the interleaved
//! two-interpreter harness. Both need "launch a command, wall-clock it, do it
//! N times, reduce"; a second implementation of that is a second definition
//! of the quantity, and the two harnesses' numbers would then not be
//! comparable even when they agree.

use std::io;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// What a timed run does with the child's standard output and standard error.
///
/// `rexx-time` discards both, so no pipe read sits inside its timing window.
/// The suite collects both, because a run whose output is never looked at
/// cannot be checked for having done the work its label claims -- a program
/// that died on its first clause is very fast.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Capture {
    Discard,
    Collect,
}

/// What one timed invocation produced.
#[derive(Debug)]
pub struct Completed {
    /// Wall time from just before `spawn` to the child having been reaped.
    pub wall: Duration,
    /// `None` when the child was killed by a signal.
    pub exit_code: Option<i32>,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

impl Completed {
    pub fn succeeded(&self) -> bool {
        self.exit_code == Some(0)
    }
}

/// Runs `program` once with `args` and `env` overlaid on this process's
/// environment, and reports what it cost.
///
/// The child's standard input is always `/dev/null`. A timing harness that
/// let a child inherit its own descriptor would either block forever on a
/// program that reads a line, or consume whatever that descriptor happened to
/// hold, which is not the same on two machines. Nothing timed here reads
/// input, so this is a guard rather than a change of measured quantity.
pub fn time_once(
    program: &str,
    args: &[String],
    env: &[(String, String)],
    capture: Capture,
) -> io::Result<Completed> {
    let mut command = Command::new(program);
    command.args(args).stdin(Stdio::null());
    for (name, value) in env {
        command.env(name, value);
    }
    match capture {
        Capture::Discard => {
            command.stdout(Stdio::null()).stderr(Stdio::null());
            let start = Instant::now();
            let status = command.status()?;
            Ok(Completed {
                wall: start.elapsed(),
                exit_code: status.code(),
                stdout: Vec::new(),
                stderr: Vec::new(),
            })
        }
        Capture::Collect => {
            let start = Instant::now();
            let output = command.output()?;
            Ok(Completed {
                wall: start.elapsed(),
                exit_code: output.status.code(),
                stdout: output.stdout,
                stderr: output.stderr,
            })
        }
    }
}

/// Order statistics of a set of timings.
#[derive(Clone, Copy, Debug)]
pub struct Summary {
    pub min: Duration,
    pub median: Duration,
    pub mean: Duration,
    pub max: Duration,
}

impl Summary {
    /// Sorts `samples` in place and reduces them. `None` for an empty set,
    /// so a harness that collected nothing reports that rather than a zero.
    ///
    /// `median` is the upper of the two middle elements at even counts
    /// (`samples[len / 2]`) rather than their mean. Not because that is the
    /// better estimator -- it is not -- but because it is what `rexx-time`
    /// has reported since Phase 0, and the committed cold-start baseline is a
    /// number produced by this expression.
    pub fn of(samples: &mut [Duration]) -> Option<Summary> {
        if samples.is_empty() {
            return None;
        }
        samples.sort_unstable();
        Some(Summary {
            min: samples[0],
            median: samples[samples.len() / 2],
            mean: samples.iter().sum::<Duration>() / samples.len() as u32,
            max: samples[samples.len() - 1],
        })
    }
}

/// A confidence interval for a median, expressed as positions in the sorted
/// sample so it can be applied to any ordered measurement.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MedianInterval {
    /// Zero-based index of the lower endpoint in the sorted sample.
    pub low: usize,
    /// Zero-based index of the upper endpoint in the sorted sample.
    pub high: usize,
    /// Coverage this interval actually achieves, which is at least the target
    /// and generally above it: the attainable coverages are a discrete set.
    pub coverage: f64,
}

/// The narrowest distribution-free interval for the median of `n` samples
/// whose coverage still reaches `target_coverage`, or `None` when no interval
/// over `n` samples reaches it.
///
/// Distribution-free rather than a normal approximation or a bootstrap, and
/// this is the statistic every later measurement in this phase must match.
/// Run times here are not normal -- they are bounded below by the work and
/// have a long right tail from scheduling -- so a mean-and-standard-error
/// interval would be describing a distribution the samples do not have. A
/// bootstrap would need an RNG and would make the reported interval depend on
/// a seed. The sign-test interval needs neither: with `B` the number of
/// samples below the true median, `B ~ Binomial(n, 1/2)` whatever the
/// distribution's shape, so `P(x_(k+1) <= median <= x_(n-k))` is exactly
/// `1 - 2 * P(B <= k)` and is computed here rather than looked up.
///
/// Coverage falls as `k` rises, so the largest `k` that still clears the
/// target is the narrowest interval that does.
pub fn median_interval_indices(n: usize, target_coverage: f64) -> Option<MedianInterval> {
    // `0.5^n` underflows f64 below about n = 1075, and a sample set that
    // large is not something this harness produces. Refusing is better than
    // silently reporting an interval computed from a zero.
    if n == 0 || n > 1000 {
        return None;
    }
    let mut term = 0.5f64.powi(n as i32);
    let mut cumulative = term;
    let mut best: Option<MedianInterval> = None;
    let mut k = 0usize;
    while k < n / 2 {
        let coverage = 1.0 - 2.0 * cumulative;
        if coverage < target_coverage {
            break;
        }
        best = Some(MedianInterval {
            low: k,
            high: n - 1 - k,
            coverage,
        });
        // C(n, k + 1) = C(n, k) * (n - k) / (k + 1), so the next term of the
        // binomial CDF costs one multiply and no factorial.
        term *= (n - k) as f64 / (k + 1) as f64;
        cumulative += term;
        k += 1;
    }
    best
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The nine-pair sample the suite takes, at the 95% the gate is stated in.
    ///
    /// Pins the endpoints, not only that some interval came back: an
    /// implementation that always returned the full range `0..n-1` would
    /// satisfy "an interval exists" and would report the whole sample as the
    /// confidence interval, which overlaps everything and can never decide a
    /// gate.
    #[test]
    fn nine_samples_at_95_percent_give_the_second_and_eighth() {
        let interval = median_interval_indices(9, 0.95).expect("9 samples admit a 95% interval");
        assert_eq!(interval.low, 1);
        assert_eq!(interval.high, 7);
        // 1 - 2 * (C(9,0) + C(9,1)) / 2^9 = 1 - 20/512.
        assert!((interval.coverage - (1.0 - 20.0 / 512.0)).abs() < 1e-12);
    }

    /// Below six samples nothing reaches 95%, because the widest interval
    /// available -- the whole sample -- covers only `1 - 2 / 2^n`.
    #[test]
    fn five_samples_cannot_reach_95_percent() {
        assert_eq!(median_interval_indices(5, 0.95), None);
        let six = median_interval_indices(6, 0.95).expect("6 samples just clear 95%");
        assert_eq!((six.low, six.high), (0, 5));
    }

    /// More samples buy a tighter interval at the same coverage. This is the
    /// property that makes the offset line's larger sample worth taking.
    #[test]
    fn more_samples_narrow_the_interval() {
        let nine = median_interval_indices(9, 0.95).expect("9 samples");
        let fifty_one = median_interval_indices(51, 0.95).expect("51 samples");
        let nine_width = (nine.high - nine.low) as f64 / 9.0;
        let fifty_one_width = (fifty_one.high - fifty_one.low) as f64 / 51.0;
        assert!(
            fifty_one_width < nine_width,
            "51 samples gave a relatively wider interval than 9: {fifty_one:?} vs {nine:?}"
        );
        assert!(fifty_one.coverage >= 0.95);
    }

    #[test]
    fn a_single_sample_has_no_interval() {
        assert_eq!(median_interval_indices(1, 0.95), None);
        assert_eq!(median_interval_indices(0, 0.95), None);
    }

    #[test]
    fn summary_of_nothing_is_nothing() {
        assert!(Summary::of(&mut []).is_none());
    }

    #[test]
    fn summary_sorts_and_reduces() {
        let mut samples = [
            Duration::from_millis(30),
            Duration::from_millis(10),
            Duration::from_millis(20),
        ];
        let summary = Summary::of(&mut samples).expect("three samples");
        assert_eq!(summary.min, Duration::from_millis(10));
        assert_eq!(summary.median, Duration::from_millis(20));
        assert_eq!(summary.max, Duration::from_millis(30));
        assert_eq!(summary.mean, Duration::from_millis(20));
    }

    #[test]
    fn a_command_that_fails_reports_its_code_rather_than_erroring() {
        let completed = time_once(
            "/bin/sh",
            &["-c".into(), "exit 7".into()],
            &[],
            Capture::Collect,
        )
        .expect("/bin/sh runs");
        assert_eq!(completed.exit_code, Some(7));
        assert!(!completed.succeeded());
    }

    #[test]
    fn collect_returns_the_child_output_on_the_right_descriptor() {
        let completed = time_once(
            "/bin/sh",
            &["-c".into(), "echo out; echo err >&2".into()],
            &[],
            Capture::Collect,
        )
        .expect("/bin/sh runs");
        assert_eq!(completed.stdout, b"out\n");
        assert_eq!(completed.stderr, b"err\n");
    }

    #[test]
    fn discard_keeps_the_output_out_of_the_result() {
        let completed = time_once(
            "/bin/sh",
            &["-c".into(), "echo out; echo err >&2".into()],
            &[],
            Capture::Discard,
        )
        .expect("/bin/sh runs");
        assert!(completed.stdout.is_empty());
        assert!(completed.stderr.is_empty());
        assert!(completed.succeeded());
    }

    #[test]
    fn env_reaches_the_child() {
        let completed = time_once(
            "/bin/sh",
            &["-c".into(), "printf %s \"$REXX_BENCH_TIMING_PROBE\"".into()],
            &[("REXX_BENCH_TIMING_PROBE".into(), "seen".into())],
            Capture::Collect,
        )
        .expect("/bin/sh runs");
        assert_eq!(completed.stdout, b"seen");
    }
}
