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

//! Repeated independent invocations of one cheap oracle comparison, and the
//! distribution of the ratio across them.
//!
//! # What this is for
//!
//! `phase-4d-gate.md`'s escalation rule: measure cheaply, and spend more
//! measurement only on an axis whose ratio lands near 1.0. This program is the
//! cheap measurement and, run repeatedly, the escalation. It answers "how far
//! apart do independent invocations land", which is the question an axis near
//! 1.0 has to answer before its verdict means anything.
//!
//! **It is not the gate's verdict and it does not establish a global band.**
//! That band was withdrawn -- see `phase-4d-gate.md`'s second amendment -- and
//! nothing here should be quoted as one. An axis far from 1.0 is decided
//! without this program.
//!
//! **It is not the optimisation loop's accept rule either.** That rule is a
//! paired comparison against this crate's own previous binary on one machine
//! state, and the oracle does not enter it.
//!
//! # Why this is not the suite with more pairs
//!
//! `rexx-bench-suite` reports a sign-test interval over the pairs inside one
//! invocation. That interval says how far the median would move if that exact
//! run were repeated, and `docs/superpowers/plans/perf-baseline.md` records it
//! being wrong about reality: `compound` produced **disjoint** intervals across
//! two quiet runs of a byte-identical binary. So adding pairs inside one
//! invocation tightens an interval around a quantity that is already known to
//! understate the movement, which is worse than not tightening it.
//!
//! One invocation of this program is therefore **one pass**: one independent
//! observation, emitted as rows. A driver loop invokes it many times, so the
//! process boundary between passes is real rather than a loop iteration, and
//! `--summarise` reads the accumulated rows back and reduces them.
//!
//! # The two modes
//!
//! * **collect** (default) -- run one pass and write tab-separated rows to
//!   standard output, one row per sampled run, progress on standard error.
//! * **`--summarise FILE...`** -- read rows back and print the per-axis
//!   between-pass distribution, plus the within-pass spread beside it so the
//!   two are never confused for each other.
//!
//! Reduction lives here rather than in a throwaway script because a figure
//! that decides an axis near 1.0 has to be recheckable, and a derivation in an
//! uncommitted script is not.
//!
//! # The control pair
//!
//! An `--axes` entry holding a `/` is a path rather than a name in
//! `bench-programs/`, which is how `bench-control/` reaches this program. The
//! escalation rule requires a pair differing by a known small amount to run
//! alongside an axis claiming parity, so that a tight interval is shown to be
//! sensitivity rather than blindness.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use rexx_bench::arms::Arm;
use rexx_bench::child::{Counted, Side, Wrapper, parse_counters, run};

/// Number of fields in a row. Named so [`row`] can take an array of exactly
/// this length and the compiler, rather than a reader, keeps the header and
/// the rows in step.
const FIELDS: usize = 10;

/// Columns of the tab-separated stream, in order.
const COLUMNS: [&str; FIELDS] = [
    "pass",
    "config",
    "axis",
    "side",
    "rep",
    "wall_s",
    "cycles",
    "instructions",
    "load1",
    "stdout",
];

/// One tab-separated row. Fixed-length array rather than a slice, so a field
/// added to [`COLUMNS`] and not to the emission site does not compile.
fn row(fields: [String; FIELDS]) -> String {
    fields.join("\t")
}

/// One pass's samples for one axis: the oracle's, then this crate's.
type PassSamples = (Vec<f64>, Vec<f64>);

/// Configuration, axis and pass -- the key a single observation carries.
type PassKey = (String, String, String);

/// Configuration and axis -- the key a distribution over passes carries.
type AxisKey = (String, String);

/// One sampled run.
#[derive(Clone, Debug)]
struct Row {
    pass: String,
    config: String,
    axis: String,
    side: String,
    wall: f64,
    cycles: Option<u64>,
}

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().collect();
    if let Some(at) = arguments.iter().position(|arg| arg == "--summarise") {
        // Up to the next flag, not every non-flag argument on the line: with
        // a filter instead of this, `--summarise rows.tsv --metric cycles`
        // read `cycles` as a third file to open.
        let files: Vec<PathBuf> = arguments[at + 1..]
            .iter()
            .take_while(|arg| !arg.starts_with("--"))
            .map(PathBuf::from)
            .collect();
        let reduction = Reduction {
            metric: flag(&arguments, "--metric").unwrap_or_else(|| "wall".to_string()),
            ratio: flag(&arguments, "--ratio").unwrap_or_else(|| "pooled".to_string()),
            statistic: flag(&arguments, "--statistic").unwrap_or_else(|| "median".to_string()),
        };
        return summarise(&files, &reduction);
    }
    collect(&arguments)
}

fn flag(arguments: &[String], name: &str) -> Option<String> {
    let at = arguments.iter().position(|arg| arg == name)?;
    arguments.get(at + 1).cloned()
}

fn flag_usize(arguments: &[String], name: &str, default: usize) -> usize {
    flag(arguments, name)
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}

/// The one-minute load average, recorded with every run because contention on
/// this machine has produced an 82.77% spread once already and a row with no
/// load figure cannot be told apart afterwards from a row taken on a quiet
/// machine.
fn load_average() -> String {
    fs::read_to_string("/proc/loadavg")
        .ok()
        .and_then(|text| text.split_whitespace().next().map(str::to_string))
        .unwrap_or_else(|| "?".to_string())
}

fn collect(arguments: &[String]) -> ExitCode {
    // **No `--engine` flag, because there is one engine.** It used to pick an
    // arm and reach the rows through the side's own label -- `rust-ir` or
    // `rust-tw` in the `side` column. Keeping it would have accepted
    // `tree-walker`, set a `REXX_ENGINE` nothing reads, and written a
    // `rust-tw` row the compiled stream produced. Rows written before the
    // choice existed read plain `rust` and are tree-walker rows; `summarise`
    // branches on `oracle`, so both shapes still reduce.
    let arm = Arm::Ir;
    let pass = flag(arguments, "--pass").unwrap_or_else(|| "1".to_string());
    let config = flag(arguments, "--config").unwrap_or_else(|| "default".to_string());
    let axes: Vec<String> = flag(arguments, "--axes")
        .unwrap_or_else(|| "alloc4c,arith,compound,strings,varlookup".to_string())
        .split(',')
        .map(str::to_string)
        .collect();
    let pairs = flag_usize(arguments, "--pairs", 3);
    let warmup = flag_usize(arguments, "--warmup", 1);
    let wrapper = Wrapper {
        pin: flag(arguments, "--pin"),
        // `Counted::Total` and not `Counted::User`: this program's rows are
        // the between-run distribution the escalation rule reads, and the
        // committed rows were taken over every privilege level.
        counters: if arguments.iter().any(|arg| arg == "--counters") {
            Counted::Total
        } else {
            Counted::Nothing
        },
    };
    let header = arguments.iter().any(|arg| arg == "--header");

    let oracle = Side::oracle();
    let rust_binary = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/release/rexx-run");
    let rust_binary = rust_binary.canonicalize().unwrap_or(rust_binary);
    if !rust_binary.is_file() || !oracle.binary.is_file() {
        eprintln!(
            "rexx-bench-band: {} or {} is missing; build both interpreters first",
            rust_binary.display(),
            oracle.binary.display()
        );
        return ExitCode::FAILURE;
    }
    let rust = Side::rust(rust_binary, arm);

    // Every child runs here rather than in the repository or in the
    // scratchpad root: the oracle resolves an unresolved call name against the
    // current directory, and a directory holding stale `.rex` files has
    // already made a probe here execute a file nobody meant to run.
    let workdir = std::env::temp_dir().join(format!("rexx-bench-band-{}", std::process::id()));
    if let Err(error) = fs::create_dir_all(&workdir) {
        eprintln!(
            "rexx-bench-band: cannot create {}: {error}",
            workdir.display()
        );
        return ExitCode::FAILURE;
    }

    if header {
        println!("#{}", COLUMNS.join("\t"));
    }
    for entry in &axes {
        let (axis, program) = resolve(entry);
        eprintln!("pass {pass} [{config}]: {axis}");
        for index in 0..(warmup + pairs) {
            let sampled = index >= warmup;
            // The oracle first inside every pair, so the two sides always sit
            // in the same order relative to whatever the machine is doing.
            for side in [&oracle, &rust] {
                let load = load_average();
                let completed = match run(side, &program, &workdir, &wrapper) {
                    Ok(completed) => completed,
                    Err(error) => {
                        eprintln!("rexx-bench-band: {error}");
                        return ExitCode::FAILURE;
                    }
                };
                if !completed.succeeded() {
                    eprintln!(
                        "rexx-bench-band: {} exited {:?} on {}: {}",
                        side.label,
                        completed.exit_code,
                        program.display(),
                        String::from_utf8_lossy(&completed.stderr).trim()
                    );
                    return ExitCode::FAILURE;
                }
                if !sampled {
                    continue;
                }
                let counters = wrapper
                    .counters
                    .events()
                    .and_then(|events| parse_counters(&completed.stderr, events));
                if wrapper.counters.events().is_some() && counters.is_none() {
                    eprintln!(
                        "rexx-bench-band: counters were requested and `perf stat` produced none \
                         on {} {}: {}",
                        side.label,
                        axis,
                        String::from_utf8_lossy(&completed.stderr).trim()
                    );
                    return ExitCode::FAILURE;
                }
                println!(
                    "{}",
                    row([
                        pass.clone(),
                        config.clone(),
                        axis.clone(),
                        side.label.to_string(),
                        (index - warmup).to_string(),
                        format!("{:.6}", completed.wall.as_secs_f64()),
                        counters.map_or_else(|| "-".to_string(), |c| c.cycles.to_string()),
                        counters.map_or_else(|| "-".to_string(), |c| c.instructions.to_string()),
                        load,
                        printable(&completed.stdout),
                    ])
                );
            }
        }
    }
    let _ = fs::remove_dir(&workdir);
    ExitCode::SUCCESS
}

/// The label and the program file one `--axes` entry names.
///
/// A bare stem is a program in `bench-programs/`; anything holding a `/` is a
/// path taken as given, labelled by its file stem. The second form exists for
/// the negative control the band needs: a program that differs from an axis by
/// a known amount does not belong in `bench-programs/`, where the suite's own
/// axis list would then have to declare it a dimension of the baseline.
fn resolve(entry: &str) -> (String, PathBuf) {
    if entry.contains('/') {
        let path = PathBuf::from(entry);
        let label = path
            .file_stem()
            .map_or_else(|| entry.to_string(), |stem| stem.to_string_lossy().into());
        return (label, path);
    }
    (entry.to_string(), rexx_bench::program_path(entry))
}

/// One line of a child's standard output, safe to put in a tab-separated
/// field. Every loop axis prints a single number, so this is the whole output
/// rather than a sample of it, and it is what lets the reduction check that
/// every run did the same work.
fn printable(stdout: &[u8]) -> String {
    String::from_utf8_lossy(stdout)
        .trim()
        .replace(['\t', '\n', '\r'], " ")
}

/// How one pass's samples become one ratio, and what quantity they are.
///
/// These choices are free: they re-reduce rows already collected, so each can
/// be compared against the others on the same machine state rather than on a
/// fresh run of its own. That matters, because a between-run comparison of two
/// reductions taken from two different runs would carry the very variance the
/// reductions are being judged on.
struct Reduction {
    /// `wall` or `cycles`.
    metric: String,
    /// `pooled` reduces each side and divides; `paired` divides inside each
    /// pair and reduces the ratios. They differ when the machine drifts during
    /// a pass, which is what the alternating layout exists to make
    /// common-mode.
    ratio: String,
    /// `median` or `min`. The minimum is the least-perturbed run of the set:
    /// interference can only add time, so the smallest sample is the closest
    /// to the work itself.
    statistic: String,
}

fn summarise(files: &[PathBuf], reduction: &Reduction) -> ExitCode {
    if files.is_empty() {
        eprintln!("rexx-bench-band: --summarise needs at least one file of rows");
        return ExitCode::FAILURE;
    }
    let mut rows: Vec<Row> = Vec::new();
    let mut outputs: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for file in files {
        let text = match fs::read_to_string(file) {
            Ok(text) => text,
            Err(error) => {
                eprintln!("rexx-bench-band: cannot read {}: {error}", file.display());
                return ExitCode::FAILURE;
            }
        };
        for line in text.lines() {
            if line.starts_with('#') || line.trim().is_empty() {
                continue;
            }
            let fields: Vec<&str> = line.split('\t').collect();
            if fields.len() < FIELDS {
                eprintln!("rexx-bench-band: short row in {}: {line}", file.display());
                return ExitCode::FAILURE;
            }
            let Ok(wall) = fields[5].parse::<f64>() else {
                eprintln!("rexx-bench-band: unreadable wall time: {line}");
                return ExitCode::FAILURE;
            };
            outputs
                .entry(fields[2].to_string())
                .or_default()
                .push(fields[9].to_string());
            rows.push(Row {
                pass: fields[0].to_string(),
                config: fields[1].to_string(),
                axis: fields[2].to_string(),
                side: fields[3].to_string(),
                wall,
                cycles: fields[6].parse::<u64>().ok(),
            });
        }
    }

    // A wall time is only about the workload if the workload ran. Checked
    // before any statistic is printed, because a run that died on its first
    // clause is extremely fast and no reduction below would notice.
    let mut disagreed = Vec::new();
    for (axis, printed) in &outputs {
        let first = &printed[0];
        if printed.iter().any(|line| line != first) {
            disagreed.push(axis.clone());
        }
    }
    if !disagreed.is_empty() {
        eprintln!(
            "rexx-bench-band: runs of {} did not all print the same bytes; \
             the timings below would not be over the same workload",
            disagreed.join(", ")
        );
        return ExitCode::FAILURE;
    }

    let use_cycles = reduction.metric == "cycles";
    if use_cycles && rows.iter().any(|row| row.cycles.is_none()) {
        eprintln!("rexx-bench-band: --metric cycles, but some rows carry no cycle count");
        return ExitCode::FAILURE;
    }
    let value = |row: &Row| -> f64 {
        if use_cycles {
            row.cycles.unwrap_or(0) as f64
        } else {
            row.wall
        }
    };

    // (config, axis, pass) -> the pass's own two medians. The pass is the unit
    // of observation; everything below is a statistic over passes.
    let mut passes: BTreeMap<PassKey, PassSamples> = BTreeMap::new();
    for row in &rows {
        let cell = passes
            .entry((row.config.clone(), row.axis.clone(), row.pass.clone()))
            .or_default();
        if row.side == "oracle" {
            cell.0.push(value(row));
        } else {
            cell.1.push(value(row));
        }
    }

    let use_min = reduction.statistic == "min";
    let paired = reduction.ratio == "paired";
    let mut per_axis: BTreeMap<AxisKey, Vec<f64>> = BTreeMap::new();
    let mut within: BTreeMap<AxisKey, Vec<f64>> = BTreeMap::new();
    for ((config, axis, _pass), (oracle, rust)) in passes {
        if oracle.is_empty() || rust.len() != oracle.len() {
            continue;
        }
        let key = (config, axis);
        per_axis
            .entry(key.clone())
            .or_default()
            .push(pass_ratio(&oracle, &rust, paired, use_min));
        within.entry(key).or_default().push(spread(&rust));
    }

    println!(
        "| config | axis | passes | median ratio | min | max | envelope | 95% band | relative sd | within-pass spread, this crate |"
    );
    println!("|---|---|---:|---:|---:|---:|---:|---:|---:|---:|");
    let mut worst: BTreeMap<String, f64> = BTreeMap::new();
    for ((config, axis), mut ratios) in per_axis {
        let n = ratios.len();
        let median_ratio = median(&mut ratios);
        let envelope = ratios
            .iter()
            .map(|ratio| (ratio / median_ratio - 1.0).abs())
            .fold(0.0f64, f64::max);
        let band = central_band(&ratios, median_ratio);
        let mut spreads = within.remove(&(config.clone(), axis.clone())).unwrap();
        let typical = median(&mut spreads);
        let entry = worst.entry(config.clone()).or_insert(0.0);
        *entry = entry.max(envelope);
        println!(
            "| {config} | `{axis}` | {n} | {median_ratio:.4}x | {:.4}x | {:.4}x | {:.2} % | {:.2} % | {:.2} % | {:.2} % |",
            ratios[0],
            ratios[n - 1],
            envelope * 100.0,
            band * 100.0,
            relative_sd(&ratios) * 100.0,
            typical * 100.0
        );
    }
    println!();
    // Named as the worst axis rather than as "the band". A single number over
    // every axis is exactly the global constant `phase-4d-gate.md`'s second
    // amendment withdrew, and the per-axis rows above are what the escalation
    // rule reads.
    for (config, envelope) in worst {
        println!(
            "* `{config}`: widest per-axis envelope is **{:.2} %** ({} / {} / {}). \
             This is one configuration's worst axis, not a threshold for any other axis.",
            envelope * 100.0,
            reduction.metric,
            reduction.ratio,
            reduction.statistic
        );
    }
    ExitCode::SUCCESS
}

/// One pass's ratio of this crate to the oracle.
///
/// `paired` divides inside each pair before reducing, which cancels a drift
/// both sides saw; `pooled` reduces each side first, which is what the suite's
/// ratio column does. `oracle` and `rust` are in pair order, so index `i` on
/// one is the partner of index `i` on the other.
fn pass_ratio(oracle: &[f64], rust: &[f64], paired: bool, use_min: bool) -> f64 {
    if !paired {
        let mut oracle = oracle.to_vec();
        let mut rust = rust.to_vec();
        return reduce(&mut rust, use_min) / reduce(&mut oracle, use_min);
    }
    let mut ratios: Vec<f64> = rust
        .iter()
        .zip(oracle)
        .map(|(rust, oracle)| rust / oracle)
        .collect();
    reduce(&mut ratios, use_min)
}

/// The median or the minimum of `values`, sorting in place either way.
fn reduce(values: &mut [f64], use_min: bool) -> f64 {
    let middle = median(values);
    if use_min { values[0] } else { middle }
}

/// Sorts in place and returns the middle element, the upper of the two at even
/// counts -- the same convention `timing::Summary` uses, so a figure from this
/// program and one from the suite are the same statistic.
fn median(values: &mut [f64]) -> f64 {
    values.sort_by(f64::total_cmp);
    values[values.len() / 2]
}

/// Relative spread of one pass's samples, as a fraction of their median.
fn spread(values: &[f64]) -> f64 {
    let mut sorted = values.to_vec();
    let middle = median(&mut sorted);
    (sorted[sorted.len() - 1] - sorted[0]) / middle
}

/// Half-width of the central 95% of `ratios`, relative to `centre`.
///
/// Reported beside the envelope rather than instead of it, because the two
/// answer different questions and both are quoted. The envelope is an extreme:
/// it can only widen as passes are added, so comparing an envelope over 30
/// passes with one over 3 is not like for like. This figure is a pair of
/// interpolated quantiles and does not systematically widen with the sample,
/// which is what makes a before-and-after comparison at different pass counts
/// honest.
///
/// `ratios` is assumed sorted, which [`median`] leaves it.
fn central_band(ratios: &[f64], centre: f64) -> f64 {
    let low = quantile(ratios, 0.025);
    let high = quantile(ratios, 0.975);
    (low / centre - 1.0).abs().max((high / centre - 1.0).abs())
}

/// The `p` quantile of a sorted sample, linearly interpolated between order
/// statistics.
///
/// Interpolated rather than "drop the outermost k": at thirty passes an
/// integer count of dropped samples rounds to zero, so a nominal 95% figure
/// would silently equal the envelope and a reader comparing the two columns
/// would see agreement that means nothing.
fn quantile(sorted: &[f64], p: f64) -> f64 {
    let n = sorted.len();
    if n == 1 {
        return sorted[0];
    }
    let position = p * (n - 1) as f64;
    let below = position.floor() as usize;
    let above = position.ceil() as usize;
    sorted[below] + (position - below as f64) * (sorted[above] - sorted[below])
}

/// Standard deviation of `ratios` as a fraction of their mean.
///
/// Quoted so a reader can derive a bound at a coverage this program does not
/// print. It is not the band: run times are bounded below by the work and have
/// a long right tail, so a normal-theory interval would describe a
/// distribution these samples do not have.
fn relative_sd(ratios: &[f64]) -> f64 {
    let n = ratios.len();
    if n < 2 {
        return 0.0;
    }
    let mean = ratios.iter().sum::<f64>() / n as f64;
    let variance = ratios
        .iter()
        .map(|ratio| (ratio - mean) * (ratio - mean))
        .sum::<f64>()
        / (n - 1) as f64;
    variance.sqrt() / mean
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A bare stem resolves into the benchmark directory and a path resolves
    /// to itself, and neither is labelled with the other's name.
    #[test]
    fn an_axis_entry_is_a_stem_or_a_path() {
        let (label, path) = resolve("alloc4c");
        assert_eq!(label, "alloc4c");
        assert_eq!(path, rexx_bench::program_path("alloc4c"));

        let (label, path) = resolve("/tmp/control/alloc4c-101.rex");
        assert_eq!(label, "alloc4c-101");
        assert_eq!(path, PathBuf::from("/tmp/control/alloc4c-101.rex"));
    }

    #[test]
    fn the_median_is_an_element() {
        assert_eq!(median(&mut [3.0, 1.0, 2.0]), 2.0);
        assert_eq!(median(&mut [4.0, 1.0, 2.0, 3.0]), 3.0);
    }

    /// The envelope is the largest relative deviation from the median, in
    /// either direction. Pinned on a set whose extreme is below the median,
    /// because taking the maximum of the signed deviation rather than of its
    /// magnitude would report zero here and would do so silently.
    #[test]
    fn the_envelope_covers_deviations_below_the_median() {
        let mut ratios = [0.90, 1.00, 1.02];
        let centre = median(&mut ratios);
        let envelope = ratios
            .iter()
            .map(|ratio| (ratio / centre - 1.0).abs())
            .fold(0.0f64, f64::max);
        assert!((envelope - 0.10).abs() < 1e-12, "{envelope}");
    }

    /// The central band drops the outermost samples once there are enough of
    /// them to drop, and is the full envelope until then. A band that silently
    /// discarded a point at n = 3 would make a three-pass characterisation
    /// look tighter than it is.
    #[test]
    fn the_central_band_pulls_in_from_the_extremes() {
        // Three passes: the quantiles interpolate almost all the way to the
        // ends, so the band is close to the envelope and does not exceed it.
        let small: Vec<f64> = vec![0.9, 1.0, 1.1];
        assert!(central_band(&small, 1.0) <= 0.1);
        assert!(central_band(&small, 1.0) > 0.09);

        // Thirty passes with one outlier: the envelope is the outlier and the
        // band is not, which is the whole reason both columns are printed.
        let mut thirty: Vec<f64> = vec![1.0; 29];
        thirty.push(2.0);
        let centre = median(&mut thirty);
        let envelope = thirty
            .iter()
            .map(|ratio| (ratio / centre - 1.0).abs())
            .fold(0.0f64, f64::max);
        assert!((envelope - 1.0).abs() < 1e-12);
        assert!(
            central_band(&thirty, centre) < 0.3,
            "{:?}",
            central_band(&thirty, centre)
        );
    }

    /// Quantiles interpolate rather than rounding to an order statistic, and
    /// the endpoints are the endpoints.
    #[test]
    fn quantiles_interpolate() {
        let sorted = [0.0, 1.0, 2.0, 3.0, 4.0];
        assert!((quantile(&sorted, 0.0) - 0.0).abs() < 1e-12);
        assert!((quantile(&sorted, 1.0) - 4.0).abs() < 1e-12);
        assert!((quantile(&sorted, 0.5) - 2.0).abs() < 1e-12);
        assert!((quantile(&sorted, 0.125) - 0.5).abs() < 1e-12);
        assert!((quantile(&[7.0], 0.3) - 7.0).abs() < 1e-12);
    }

    /// The relative standard deviation is zero for a constant sample and is
    /// scale-free, which is what lets it be quoted as a percentage.
    #[test]
    fn the_relative_standard_deviation_is_scale_free() {
        assert!(relative_sd(&[2.0, 2.0, 2.0]).abs() < 1e-12);
        let small = relative_sd(&[1.0, 2.0, 3.0]);
        let large = relative_sd(&[10.0, 20.0, 30.0]);
        assert!((small - large).abs() < 1e-12);
        assert_eq!(relative_sd(&[1.0]), 0.0);
    }

    /// The two ratio modes are different reductions of the same samples, and
    /// they disagree when one side is perturbed on a pair the other is not.
    /// Pinned because the whole reason to offer both is that they can differ:
    /// an implementation where `paired` fell through to `pooled` would satisfy
    /// every other assertion here.
    #[test]
    fn pooled_and_paired_disagree_when_one_pair_is_perturbed() {
        let oracle = [1.0, 1.0, 2.0];
        let rust = [3.0, 1.0, 2.0];
        assert_eq!(pass_ratio(&oracle, &rust, false, false), 2.0);
        assert_eq!(pass_ratio(&oracle, &rust, true, false), 1.0);
    }

    /// The minimum reduction really takes the smallest sample, on both sides,
    /// after the sort the median leaves behind.
    #[test]
    fn the_minimum_reduction_takes_the_least_perturbed_run() {
        assert_eq!(reduce(&mut [3.0, 1.0, 2.0], true), 1.0);
        assert_eq!(reduce(&mut [3.0, 1.0, 2.0], false), 2.0);
        let oracle = [1.0, 1.2, 1.4];
        let rust = [4.0, 2.0, 3.0];
        assert_eq!(pass_ratio(&oracle, &rust, false, true), 2.0);
    }

    #[test]
    fn the_spread_is_relative_to_the_median() {
        assert!((spread(&[1.0, 1.0, 1.1]) - 0.1).abs() < 1e-12);
    }

    /// A row splits into exactly the fields the header names, and the two
    /// columns the reduction reads by position are where it reads them.
    ///
    /// The header and the emission site are separate expressions and a reader
    /// of a collected file has nothing else to align them by. The array length
    /// is what keeps them the same width; these assertions are what keep
    /// `wall_s` and `cycles` at the offsets `summarise` indexes.
    #[test]
    fn a_row_splits_into_the_columns_the_header_names() {
        let emitted = row(std::array::from_fn(|index| COLUMNS[index].to_string()));
        let fields: Vec<&str> = emitted.split('\t').collect();
        assert_eq!(fields, COLUMNS);
        assert_eq!(fields[5], "wall_s");
        assert_eq!(fields[6], "cycles");
        assert_eq!(fields[9], "stdout");
    }
}
