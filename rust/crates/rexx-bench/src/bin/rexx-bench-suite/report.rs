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

//! Renders the paired samples `measure_interleaved` collects into the
//! markdown report `main` prints.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::time::Duration;

use rexx_bench::child::{ADDRESS_SPACE_LIMIT_KIB, Counters, Side, Wrapper, run};
use rexx_bench::timing::{MedianInterval, median_interval_indices};

use super::{AXES, AxisRow, Paired, Role, capture};

/// The coverage every interval in this report targets.
const TARGET_COVERAGE: f64 = 0.95;

/// Point estimate and interval for one side of one axis, in seconds.
#[derive(Clone, Copy, Debug)]
struct Stats {
    median: f64,
    min: f64,
    max: f64,
    low: f64,
    high: f64,
    coverage: f64,
}

impl Stats {
    fn of(values: &mut [f64]) -> Stats {
        values.sort_by(f64::total_cmp);
        let interval = interval_for(values.len());
        Stats {
            median: values[values.len() / 2],
            min: values[0],
            max: values[values.len() - 1],
            low: values[interval.low],
            high: values[interval.high],
            coverage: interval.coverage,
        }
    }

    /// Relative spread, as a fraction of the median.
    fn spread(&self) -> f64 {
        (self.max - self.min) / self.median
    }
}

/// The interval indices for a sample of this size, falling back to the whole
/// range when the sample is too small for [`TARGET_COVERAGE`].
pub(super) fn interval_for(n: usize) -> MedianInterval {
    median_interval_indices(n, TARGET_COVERAGE).unwrap_or(MedianInterval {
        low: 0,
        high: n - 1,
        coverage: if n == 0 {
            0.0
        } else {
            1.0 - 2.0 * 0.5f64.powi(n as i32)
        },
    })
}

/// The per-side coverage one axis's intervals carry, taken from its sample
/// size rather than restated, so it cannot drift from [`crate::PAIRS`].
fn interval_coverage(row: &AxisRow) -> f64 {
    interval_for(row.paired.oracle.len()).coverage
}

/// The Bonferroni lower bound on the ratio interval's coverage, from one
/// side's coverage.
pub(super) fn joint_coverage_bound(per_side: f64) -> f64 {
    (1.0 - 2.0 * (1.0 - per_side)).max(0.0)
}

/// The paragraph that tells a reader what the ratio interval is worth.
pub(super) fn ratio_interval_caveat(per_side: f64) -> String {
    let joint = joint_coverage_bound(per_side);
    if joint > 0.0 {
        format!(
            "**The ratio interval is indicative, and the verdict is not taken from it.** It \
             divides one side's interval by the other's, so its joint coverage is at least \
             {:.1}% by Bonferroni -- one minus the two sides' miss probabilities added -- not \
             the {:.1}% either side carries alone. The verdict applies Global Constraints' rule \
             directly: this crate's point estimate against the oracle's interval, slow side.",
            joint * 100.0,
            per_side * 100.0
        )
    } else {
        format!(
            "**The ratio interval is indicative, and the verdict is not taken from it.** Its \
             joint coverage is a Bonferroni bound -- one minus the two sides' miss \
             probabilities added -- and at this sample size that bound is vacuous, the interval \
             on each side carrying only {:.1}%. The verdict applies Global Constraints' rule \
             directly: this crate's point estimate against the oracle's interval, slow side.",
            per_side * 100.0
        )
    }
}

fn seconds(samples: &[Duration]) -> Vec<f64> {
    samples.iter().map(Duration::as_secs_f64).collect()
}

fn fingerprint(path: &Path) -> String {
    let stat = capture(
        "stat",
        &[
            "-L",
            "-c",
            "size=%s bytes, mtime=%y",
            &path.display().to_string(),
        ],
    );
    let digest = capture("sha256sum", &["-b", &path.display().to_string()]);
    let sha = digest.split_whitespace().next().unwrap_or("?").to_string();
    format!("{stat}, sha256={sha}")
}

#[allow(clippy::too_many_arguments)]
pub(super) fn write_provenance(
    report: &mut String,
    oracle_binary: &Path,
    oracle_objects: &[PathBuf],
    rust_binary: &Path,
    pairs: usize,
    warmup: usize,
    offset_pairs: usize,
    offset_warmup: usize,
    wrapper: &Wrapper,
) {
    let _ = writeln!(report, "### Provenance\n");
    let _ = writeln!(report, "| | |");
    let _ = writeln!(report, "|---|---|");
    let _ = writeln!(report, "| measured | {} |", capture("date", &["-Is"]));
    let _ = writeln!(
        report,
        "| repo commit | `{}` |",
        capture(
            "git",
            &["-C", env!("CARGO_MANIFEST_DIR"), "rev-parse", "HEAD"]
        )
    );
    let _ = writeln!(
        report,
        "| oracle `bin/rexx` | `{}` -- {} |",
        oracle_binary.display(),
        fingerprint(oracle_binary)
    );
    // The launcher is 60 KB of `main`; the interpreter is in the shared
    // objects it loads. A rebuild of a library alone leaves the launcher's
    // fingerprint unchanged, so fingerprinting only `bin/rexx` would not
    // detect the oracle moving under this baseline.
    for object in oracle_objects {
        let _ = writeln!(
            report,
            "| oracle `lib/{}` | {} |",
            object
                .file_name()
                .map_or_else(|| "?".into(), |name| name.to_string_lossy()),
            fingerprint(object)
        );
    }
    let _ = writeln!(
        report,
        "| this crate `rexx-run` | `{}` -- {} |",
        rust_binary.display(),
        fingerprint(rust_binary)
    );
    // The engine still belongs here for the same reason the sha256 does: it
    // decides what the ratios below are ratios of, and a baseline taken on
    // another arm must not be comparable with nothing to notice. It is no
    // longer a choice -- there is one engine and no `REXX_ENGINE` to set --
    // so the row states that rather than reporting a variable nothing reads.
    // Every figure recorded before this row existed is a tree-walker figure,
    // because that is what `rexx-run` defaulted to at the time.
    let _ = writeln!(
        report,
        "| this crate's engine | the compiled stream, the only one; \
         `REXX_ENGINE` is not read |"
    );
    let _ = writeln!(
        report,
        "| address-space cap | `ulimit -v {ADDRESS_SPACE_LIMIT_KIB}` KiB, **both sides, every axis** |"
    );
    let _ = writeln!(
        report,
        "| pairs per axis | {pairs} sampled, {warmup} warm-up pair(s) discarded, oracle and this crate alternating |"
    );
    let _ = writeln!(
        report,
        "| pairs for the offset line | {offset_pairs} sampled, {offset_warmup} warm-up |"
    );
    let _ = writeln!(
        report,
        "| statistic | median; interval is the distribution-free sign-test interval for the median at a {:.0}% target |",
        TARGET_COVERAGE * 100.0
    );
    let _ = writeln!(
        report,
        "| working directory of every child | a fresh empty temporary directory |"
    );
    // Emitted only when it is on, so a run with no arguments prints the block
    // `perf-baseline.md` carries. An absent row therefore means an unpinned
    // run, which is what every figure in that document was taken with.
    if let Some(cpus) = &wrapper.pin {
        let _ = writeln!(
            report,
            "| CPU affinity of every child | `taskset -c {cpus}`, **both sides, every axis** |"
        );
    }
    let _ = writeln!(report);
}

pub(super) fn write_offset(report: &mut String, name: &str, paired: &Paired) {
    let oracle = Stats::of(&mut seconds(&paired.oracle));
    let rust = Stats::of(&mut seconds(&paired.rust));
    let _ = writeln!(report, "### Fixed per-process offset (`{name}.rex`)\n");
    let _ = writeln!(
        report,
        "**Comparable, and the difference is a result about startup.** Both sides pay a fixed \
         cost for the same Rexx-written library before a program's first clause: the oracle \
         restores its saved image, this crate parses and installs the `.orx` sources that image \
         was built from. The two numbers below are each side's own fixed cost, reported so every \
         axis above can be read net of it.\n"
    );
    let _ = writeln!(
        report,
        "| side | median | min | max | {:.1}% interval | spread |",
        oracle.coverage * 100.0
    );
    let _ = writeln!(report, "|---|---:|---:|---:|---|---:|");
    for (label, stats) in [("oracle", oracle), ("this crate", rust)] {
        let _ = writeln!(
            report,
            "| {label} | {:.3} ms | {:.3} ms | {:.3} ms | {:.3} - {:.3} ms | {:.1} % |",
            stats.median * 1e3,
            stats.min * 1e3,
            stats.max * 1e3,
            stats.low * 1e3,
            stats.high * 1e3,
            stats.spread() * 100.0
        );
    }
    let _ = writeln!(report);
    let _ = writeln!(
        report,
        "Both offsets include one `/bin/sh` `exec` from the `ulimit` wrapper, on both sides \
         equally.\n"
    );
}

/// Median of one counter across a side's sampled runs.
fn counter_median(readings: &[Counters], pick: fn(&Counters) -> u64) -> f64 {
    let mut values: Vec<f64> = readings.iter().map(|r| pick(r) as f64).collect();
    Stats::of(&mut values).median
}

/// The hardware counters, per axis and per side.
pub(super) fn write_counters(report: &mut String, rows: &[AxisRow]) {
    if rows.is_empty() {
        return;
    }
    let _ = writeln!(report, "#### Cycles and instructions\n");
    let _ = writeln!(
        report,
        "Counted with `perf stat -e cycles:u,instructions:u` on the **same runs** the wall times \
         above come from, so a row's three quantities describe one set of executions rather than \
         three. A reading `perf` could not schedule for the whole run is refused rather than \
         recorded, because it would be reported scaled up to an estimate in the same format as an \
         exact count.\n"
    );
    let _ = writeln!(
        report,
        "**The instruction ratio is not the scoreboard and does not stand in for the wall ratio.** \
         Read the two ratio columns below against the wall ratios above: `cycles` tracks wall time \
         closely on every axis, and `instructions` tracks it on none, missing in *both* directions \
         -- an axis can retire far more instructions than the oracle and lose by less than that \
         suggests, or retire about as many and win comfortably. The two interpreters reach a \
         variable and hold a small integer differently enough that their instructions are not the \
         same unit of work, so a ratio between their counts is not a ratio of anything. **Quote \
         cycles or wall time when comparing the two sides.**\n"
    );
    let _ = writeln!(
        report,
        "| axis | side | cycles | instructions | IPC | cycles/iter | instr/iter |"
    );
    let _ = writeln!(report, "|---|---|---:|---:|---:|---:|---:|");
    for row in rows {
        for (label, readings) in [
            ("oracle", &row.paired.oracle_counters),
            ("this crate", &row.paired.rust_counters),
        ] {
            if readings.is_empty() {
                let _ = writeln!(
                    report,
                    "| `{}` | {label} | **no reading** | **no reading** | | | |",
                    row.name
                );
                continue;
            }
            let cycles = counter_median(readings, |r| r.cycles);
            let instructions = counter_median(readings, |r| r.instructions);
            let iterations = row.iterations as f64;
            let _ = writeln!(
                report,
                "| `{}` | {label} | {:.0} | {:.0} | {:.2} | {:.1} | {:.1} |",
                row.name,
                cycles,
                instructions,
                instructions / cycles,
                cycles / iterations,
                instructions / iterations,
            );
        }
    }
    let _ = writeln!(report);

    let _ = writeln!(report, "| axis | cycles ratio | instructions ratio |");
    let _ = writeln!(report, "|---|---:|---:|");
    for row in rows {
        if row.paired.oracle_counters.is_empty() || row.paired.rust_counters.is_empty() {
            continue;
        }
        let oracle_cycles = counter_median(&row.paired.oracle_counters, |r| r.cycles);
        let rust_cycles = counter_median(&row.paired.rust_counters, |r| r.cycles);
        let oracle_instructions = counter_median(&row.paired.oracle_counters, |r| r.instructions);
        let rust_instructions = counter_median(&row.paired.rust_counters, |r| r.instructions);
        let _ = writeln!(
            report,
            "| `{}` | {:.2}x | {:.2}x |",
            row.name,
            rust_cycles / oracle_cycles,
            rust_instructions / oracle_instructions,
        );
    }
    let _ = writeln!(
        report,
        "\nBoth are this crate's median over the oracle's, so below 1.00x is this crate ahead -- \
         the same direction as the wall ratio above.\n\n**Where `instructions:u` is the right \
         instrument is this crate against itself.** It is deterministic to several significant \
         figures where cycles move a few per cent between runs on this machine, which is why every \
         optimisation in this tree is justified with it and why `rexx-arms` reports it. That is a \
         claim about A/B-ing one binary against another, not about the column beside it.\n"
    );
}

pub(super) fn write_axes(report: &mut String, rows: &[AxisRow], offset: &Paired) {
    let _ = writeln!(report, "### Axes\n");
    // Every axis having failed is already reported loudly elsewhere; what
    // this guard buys is that the sections below never have to describe a
    // statistic over nothing.
    let Some(first) = rows.first() else {
        let _ = writeln!(
            report,
            "**No axis produced samples.** There is nothing to state a throughput, a ratio or \
             an interval over; see the failures listed at the end.\n"
        );
        return;
    };
    let oracle_offset = Stats::of(&mut seconds(&offset.oracle)).median;
    let rust_offset = Stats::of(&mut seconds(&offset.rust)).median;

    let _ = writeln!(
        report,
        "`iters/s` is the program's own loop bound divided by the median wall time. \
         `iters/s net` divides by the median wall time less that side's per-process offset above.\n"
    );
    let _ = writeln!(
        report,
        "| axis | iterations | side | median | min | max | interval | spread | iters/s | iters/s net |"
    );
    let _ = writeln!(report, "|---|---:|---|---:|---:|---:|---|---:|---:|---:|");
    for row in rows {
        let oracle = Stats::of(&mut seconds(&row.paired.oracle));
        let rust = Stats::of(&mut seconds(&row.paired.rust));
        for (label, stats, side_offset) in [
            ("oracle", oracle, oracle_offset),
            ("this crate", rust, rust_offset),
        ] {
            let _ = writeln!(
                report,
                "| `{}` | {} | {label} | {:.4} s | {:.4} s | {:.4} s | {:.4} - {:.4} s | {:.2} % | {:.0} | {:.0} |",
                row.name,
                row.iterations,
                stats.median,
                stats.min,
                stats.max,
                stats.low,
                stats.high,
                stats.spread() * 100.0,
                row.iterations as f64 / stats.median,
                row.iterations as f64 / (stats.median - side_offset),
            );
        }
    }
    let _ = writeln!(report);

    let _ = writeln!(report, "#### Ratios and the gate call\n");
    let _ = writeln!(
        report,
        "The throughput ratio (oracle iters/s over this crate's) and the wall ratio (this crate's \
         median over the oracle's) are the same number, because both sides run the same iteration \
         count.\n"
    );
    // The joint coverage is stated because the obvious reading of the column
    // is wrong. Dividing one side's interval by the other's is a Bonferroni
    // combination, so the guarantee is one minus the sum of the two miss
    // probabilities, not either side's own coverage. The verdict does not use
    // this column -- it is the point estimate against the oracle interval,
    // exactly as the gate is stated -- so a later reader quoting the ratio
    // interval as a 95% interval is the only thing this narrows, and that is
    // the reader worth protecting.
    let _ = writeln!(
        report,
        "{}\n",
        ratio_interval_caveat(interval_coverage(first))
    );
    let _ = writeln!(
        report,
        "| axis | oracle median | this crate median | ratio | ratio interval | verdict |"
    );
    let _ = writeln!(report, "|---|---:|---:|---:|---|---|");
    for row in rows {
        let oracle = Stats::of(&mut seconds(&row.paired.oracle));
        let rust = Stats::of(&mut seconds(&row.paired.rust));
        let verdict = if rust.median > oracle.high {
            "SLOWER"
        } else if rust.median < oracle.low {
            "faster"
        } else {
            "within the oracle interval"
        };
        let _ = writeln!(
            report,
            "| `{}` | {:.4} s | {:.4} s | {:.2}x | {:.2}x - {:.2}x | {verdict} |",
            row.name,
            oracle.median,
            rust.median,
            rust.median / oracle.median,
            rust.low / oracle.high,
            rust.high / oracle.low,
        );
    }
    let _ = writeln!(report);

    let _ = writeln!(report, "#### Same work on both sides\n");
    let _ = writeln!(
        report,
        "A wall time is only about the workload if the workload ran. Every sampled run on each \
         side printed the same bytes, and the two sides printed the same bytes as each other.\n"
    );
    let _ = writeln!(
        report,
        "| axis | stable within a side | identical across sides | stdout |"
    );
    let _ = writeln!(report, "|---|---|---|---|");
    for row in rows {
        let oracle = Paired::stable_stdout(&row.paired.oracle_stdout);
        let rust = Paired::stable_stdout(&row.paired.rust_stdout);
        let stable = oracle.is_some() && rust.is_some();
        let identical = oracle.is_some() && oracle == rust;
        let _ = writeln!(
            report,
            "| `{}` | {} | {} | `{}` |",
            row.name,
            if stable { "yes" } else { "**NO**" },
            if identical { "yes" } else { "**NO**" },
            String::from_utf8_lossy(oracle.unwrap_or(b"")).trim()
        );
    }
    let _ = writeln!(report);
}

pub(super) fn write_rexxcps(report: &mut String, paired: &Paired) {
    let oracle_wall = Stats::of(&mut seconds(&paired.oracle));
    let rust_wall = Stats::of(&mut seconds(&paired.rust));
    let _ = writeln!(report, "### `bench-rexxcps/rexxcps.rex`\n");
    let _ = writeln!(
        report,
        "REXXCPS 2.2 with its loop counts fixed, so **both sides do identical work** and the wall \
         times below are directly comparable. The clauses-per-second figure each side prints is \
         per clause and is comparable for the same reason. Each side's `Averaged:` line is quoted \
         so that identity is visible rather than assumed: the two must read the same, and a report \
         where they do not is one where something rewrote the counts. The stock \
         `samples/rexxcps.rex` scales its own count until a trial takes about a second, which is \
         what this file exists to remove -- see `bench-rexxcps/README.md`.\n"
    );
    let _ = writeln!(
        report,
        "| side | wall median | wall interval | cycles | instructions | IPC | `Averaged:` |"
    );
    let _ = writeln!(report, "|---|---:|---|---:|---:|---:|---|");
    for (label, stats, runs, readings) in [
        (
            "oracle",
            oracle_wall,
            &paired.oracle_stdout,
            &paired.oracle_counters,
        ),
        (
            "this crate",
            rust_wall,
            &paired.rust_stdout,
            &paired.rust_counters,
        ),
    ] {
        let cycles = counter_median(readings, |r| r.cycles);
        let instructions = counter_median(readings, |r| r.instructions);
        let _ = writeln!(
            report,
            "| {label} | {:.4} s | {:.4} - {:.4} s | {:.0} | {:.0} | {:.2} | {} |",
            stats.median,
            stats.low,
            stats.high,
            cycles,
            instructions,
            instructions / cycles,
            quoted_line(runs.last().map_or(&[][..], Vec::as_slice), "Averaged:")
        );
    }
    let _ = writeln!(report);

    // The comparable figure. Every sampled run prints one, so this is a
    // sample of the same size as the wall times beside it rather than a
    // single reading dressed up as an estimate.
    let mut oracle_cps: Vec<f64> = paired
        .oracle_stdout
        .iter()
        .filter_map(|run| parse_cps(run))
        .map(|cps| cps as f64)
        .collect();
    let mut rust_cps: Vec<f64> = paired
        .rust_stdout
        .iter()
        .filter_map(|run| parse_cps(run))
        .map(|cps| cps as f64)
        .collect();
    if oracle_cps.len() != paired.oracle.len() || rust_cps.len() != paired.rust.len() {
        let _ = writeln!(
            report,
            "**Some runs printed no `Performance:` line** -- {} of {} on the oracle, {} of {} on \
             this crate. The rows below cover only the runs that did.\n",
            oracle_cps.len(),
            paired.oracle.len(),
            rust_cps.len(),
            paired.rust.len()
        );
    }
    if oracle_cps.is_empty() || rust_cps.is_empty() {
        let _ = writeln!(report, "**No clauses-per-second figure could be read.**\n");
        return;
    }
    let oracle = Stats::of(&mut oracle_cps);
    let rust = Stats::of(&mut rust_cps);
    let _ = writeln!(
        report,
        "| side | median cps | min | max | {:.1}% interval | spread |",
        oracle.coverage * 100.0
    );
    let _ = writeln!(report, "|---|---:|---:|---:|---|---:|");
    for (label, stats) in [("oracle", oracle), ("this crate", rust)] {
        let _ = writeln!(
            report,
            "| {label} | {:.0} | {:.0} | {:.0} | {:.0} - {:.0} | {:.2} % |",
            stats.median,
            stats.min,
            stats.max,
            stats.low,
            stats.high,
            stats.spread() * 100.0
        );
    }
    let _ = writeln!(
        report,
        "\n**Internal cps ratio: {:.2}x** (oracle median over this crate's median), interval \
         {:.2}x - {:.2}x.\n",
        oracle.median / rust.median,
        oracle.low / rust.high,
        oracle.high / rust.low
    );
}

/// Every `label= value` figure one side's sampled runs printed, keyed by
/// label and in run order.
fn self_timed_figures(runs: &[Vec<u8>]) -> BTreeMap<String, Vec<f64>> {
    let mut figures: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    for run in runs {
        for line in String::from_utf8_lossy(run).lines() {
            let Some((label, value)) = line.split_once("= ") else {
                continue;
            };
            let Ok(value) = value.trim().parse::<f64>() else {
                continue;
            };
            figures
                .entry(label.trim().to_string())
                .or_default()
                .push(value);
        }
    }
    figures
}

/// The median of `samples`, which the caller has already checked is not empty.
fn median_of(samples: &[f64]) -> f64 {
    let mut sorted = samples.to_vec();
    sorted.sort_by(f64::total_cmp);
    sorted[sorted.len() / 2]
}

/// Reports a [`Role::SelfTimed`] axis from the figures the program printed on
/// each side rather than from the wall time this harness took.
pub(super) fn write_self_timed(report: &mut String, name: &str, pairs: usize, paired: &Paired) {
    let oracle = self_timed_figures(&paired.oracle_stdout);
    let rust = self_timed_figures(&paired.rust_stdout);
    let _ = writeln!(
        report,
        "`{name}`, {pairs} interleaved pair(s). Every number below is the median of what the \
         program itself printed; this harness timed nothing here, because the process wall \
         time is the sum of parts the program separates.\n"
    );
    let _ = writeln!(
        report,
        "| figure | oracle | this crate | this crate / oracle | runs (oracle, crate) |"
    );
    let _ = writeln!(report, "|---|---:|---:|---:|---:|");
    for (label, oracle_samples) in &oracle {
        let Some(rust_samples) = rust.get(label) else {
            let _ = writeln!(report, "| `{label}` | -- | *absent* | -- | -- |");
            continue;
        };
        if oracle_samples.is_empty() || rust_samples.is_empty() {
            continue;
        }
        let (left, right) = (median_of(oracle_samples), median_of(rust_samples));
        let _ = writeln!(
            report,
            "| `{label}` | {left:.6} | {right:.6} | {:.3}x | {}, {} |",
            right / left,
            oracle_samples.len(),
            rust_samples.len()
        );
    }
    for label in rust.keys().filter(|label| !oracle.contains_key(*label)) {
        let _ = writeln!(report, "| `{label}` | *absent* | -- | -- | -- |");
    }
    let _ = writeln!(report);
}

pub(super) fn quoted_line(stdout: &[u8], marker: &str) -> String {
    String::from_utf8_lossy(stdout)
        .lines()
        .find(|line| line.contains(marker))
        .map(|line| format!("`{}`", line.trim()))
        .unwrap_or_else(|| "*absent*".to_string())
}

pub(super) fn parse_cps(stdout: &[u8]) -> Option<u64> {
    let text = String::from_utf8_lossy(stdout).into_owned();
    let line = text.lines().find(|line| line.contains("Performance:"))?;
    line.split_whitespace()
        .skip_while(|word| !word.contains("Performance:"))
        .nth(1)?
        .parse()
        .ok()
}

/// Runs the axes declared [`Role::Blocked`], reports what each produced, and
/// names any that no longer deserve the role.
pub(super) fn write_blocked(
    report: &mut String,
    rust: &Side,
    workdir: &Path,
    wrapper: &Wrapper,
) -> Vec<String> {
    let mut no_longer_blocked = Vec::new();
    let _ = writeln!(report, "### Axes this crate cannot run\n");
    let blocked = AXES.iter().filter(|axis| axis.role == Role::Blocked);
    if blocked.clone().next().is_none() {
        let _ = writeln!(report, "None: every axis in the list runs on both sides.\n");
        return no_longer_blocked;
    }
    let _ = writeln!(
        report,
        "Measured here rather than left out of the table, with the status and message each one \
         actually produced. These belong to later tasks in this phase; what belongs to this one \
         is that they are visible.\n"
    );
    let _ = writeln!(report, "| axis | exit status | message |");
    let _ = writeln!(report, "|---|---:|---|");
    for axis in AXES.iter().filter(|axis| axis.role == Role::Blocked) {
        let path = rexx_bench::program_path(axis.name);
        match run(rust, &path, workdir, wrapper) {
            Ok(completed) => {
                if completed.succeeded() {
                    no_longer_blocked.push(axis.name.to_string());
                }
                let _ = writeln!(
                    report,
                    "| `{}` | {} | `{}` |",
                    axis.name,
                    completed
                        .exit_code
                        .map_or_else(|| "signal".to_string(), |code| code.to_string()),
                    String::from_utf8_lossy(&completed.stderr).trim()
                );
            }
            Err(error) => {
                let _ = writeln!(
                    report,
                    "| `{}` | -- | could not launch: {error} |",
                    axis.name
                );
            }
        }
    }
    let _ = writeln!(report);
    if !no_longer_blocked.is_empty() {
        let _ = writeln!(
            report,
            "**{} no longer fails on this crate and is still declared `Role::Blocked`.** \
             It is being reported as unrunnable and timed by nothing. Give it `Role::Loop` \
             and a loop bound, or decide deliberately that it stays out.\n",
            no_longer_blocked.join(", ")
        );
    }
    no_longer_blocked
}
