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

//! The interleaved two-interpreter benchmark suite: this crate against the
//! built C++ oracle, one axis at a time, and the baseline report it emits.
//!
//! Report on standard output, progress on standard error. The report is
//! markdown so it can go into `docs/superpowers/plans/perf-baseline.md`
//! without being retyped; a number retyped is a number that can be retyped
//! wrong, and this project has already put a figure into a gate document
//! against the wrong commit that way.
//!
//! # Why the two sides alternate
//!
//! For each axis the suite runs the oracle, then this crate, then the oracle,
//! then this crate, for [`PAIRS`] pairs. Not one side's whole set and then
//! the other's. On this 32-core part the clock drifts across minutes by more
//! than several of the effects being measured, and a block-per-side layout
//! turns that drift into a bias on whichever side ran during the drift.
//! Alternating makes it common-mode: both sides see the same drift, in the
//! same order, and a ratio taken pairwise is insensitive to it.
//!
//! # What the numbers are
//!
//! Absolute throughput on both sides, not only their ratio. A ratio hides its
//! denominator, and one open question this phase has to settle is whether the
//! wide per-axis spread of ratios is a property of the oracle's variation or
//! of this crate's -- which only the two sides' absolute spreads can answer.
//! Iterations per second comes from each program's own `n = ...` loop bound,
//! read out of the program text, so it is exact rather than an estimate of
//! clauses per iteration.
//!
//! The fixed per-process offset gets its own line rather than being spread
//! across every axis, measured with a program whose body is a single `say`.
//! It is not symmetric between the two sides: this crate reserves 512 MiB of
//! address space (`rexx_exec::INTERPRETER_STACK_BYTES`) before running
//! anything and the oracle reserves nothing comparable.
//!
//! # Linux only
//!
//! The child wrapper is `/bin/sh` and the fingerprints come from `stat` and
//! `sha256sum`. `std::process::Command` has no rlimit hook and the workspace
//! forbids the `unsafe` `pre_exec` that would give it one, so the address-
//! space cap has to be a shell builtin -- the same wrapper
//! `rexx-exec/tests/support/oracle.rs` uses for the same reason.
//!
//! The wrapper itself lives in `rexx_bench::child`, shared with
//! `rexx-bench-band`, so the two harnesses cannot launch their children
//! differently. `--pin <cpulist>` confines every child to those CPUs; without
//! it the wrapper is the one every committed figure was taken through.

use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::time::Duration;

use rexx_bench::child::{ADDRESS_SPACE_LIMIT_KIB, Counted, ORACLE_ROOT, Side, Wrapper, run};
use rexx_bench::timing::{MedianInterval, median_interval_indices};

/// Paired measurements per axis.
///
/// Nine, and odd so the median is an element rather than a mean of two. Nine
/// is the smallest odd count whose distribution-free 95% median interval is
/// tighter than the whole sample: at nine the interval is the second to the
/// eighth order statistic, at seven and below it is the whole range and would
/// overlap anything. It is also what the suite can afford -- this crate takes
/// about 28 s on the slowest axis, so nine pairs is minutes rather than
/// tens of minutes, and a baseline nobody re-runs is a baseline that goes
/// stale.
const PAIRS: usize = 9;

/// Pairs run and discarded before sampling starts, per axis. One is enough
/// because both interpreters are cold only in the page cache, and both files
/// are read by the pair that precedes the first sampled one.
const WARMUP_PAIRS: usize = 1;

/// Pairs for the per-process offset line. Larger than [`PAIRS`] because the
/// quantity is milliseconds rather than seconds, so its relative noise is far
/// higher and its absolute cost is nil: fifty-one pairs of a `say 1` is well
/// under a second in total.
const OFFSET_PAIRS: usize = 51;

/// Warm-up pairs for the offset line.
const OFFSET_WARMUP_PAIRS: usize = 5;

/// The coverage every interval in this report targets.
///
/// The gate's definition of "slower" (the plan's Global Constraints,
/// "Performance gate") is the point estimate falling outside the C++
/// baseline's confidence interval on the slow side, so the report must carry
/// an interval and not only a median. 95% because that is the level the
/// existing criterion rows in `perf-baseline.md` are stated at, and a later
/// measurement has to be comparable to this one.
const TARGET_COVERAGE: f64 = 0.95;

/// The oracle's own clauses-per-second benchmark, in the read-only C++ tree.
/// Not copied into this repository: it is the oracle's file, and a copy is a
/// thing that can drift from it.
const REXXCPS: &str = "/home/moritz/dev/repos/ooRexx/samples/rexxcps.rex";

/// What the suite does with one benchmark program.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Role {
    /// Timed on both sides. Iterations per second comes from the program's
    /// own loop bound.
    Loop,
    /// Timed on both sides and reported as the fixed per-process offset
    /// rather than as an axis. Its two sides are **not** comparable: this
    /// crate has no `CoreClasses.orx` bootstrap yet, so it starts fast by not
    /// doing the work the oracle does at startup.
    Offset,
    /// This crate cannot run it. Reported with the exit status and message it
    /// actually produced, never omitted -- an axis that quietly leaves the
    /// table is the failure this list exists to prevent.
    Blocked,
}

/// One benchmark axis: the stem of a file in `rust/bench-programs/`.
struct Axis {
    name: &'static str,
    role: Role,
}

/// Every program in `rust/bench-programs/`, in sorted order.
///
/// A literal here and asserted against the directory by
/// [`verify_axis_list`], which runs before any measurement and again as a
/// `#[test]`. Without the assertion an axis can be added to the directory and
/// never measured, or removed from it and silently vanish from the report,
/// and the report stays green over whatever is left -- the same defect
/// `rexx-exec/tests/corpus.rs` guards its phase subset files against, twice
/// having actually happened there.
///
/// Sorted, because the directory listing this is compared against is sorted.
const AXES: &[Axis] = &[
    Axis {
        name: "alloc",
        role: Role::Blocked,
    },
    Axis {
        name: "alloc4c",
        role: Role::Loop,
    },
    Axis {
        name: "arith",
        role: Role::Loop,
    },
    Axis {
        name: "compound",
        role: Role::Loop,
    },
    Axis {
        name: "dispatch",
        role: Role::Blocked,
    },
    Axis {
        name: "emptyloop",
        role: Role::Loop,
    },
    Axis {
        name: "heapshape",
        role: Role::Blocked,
    },
    Axis {
        name: "startup",
        role: Role::Offset,
    },
    Axis {
        name: "strings",
        role: Role::Loop,
    },
    Axis {
        name: "varlookup",
        role: Role::Loop,
    },
];

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().collect();
    let self_check = arguments.iter().any(|arg| arg == "--self-check");
    let (pairs, warmup, offset_pairs, offset_warmup) = if self_check {
        (1, 0, 3, 0)
    } else {
        (PAIRS, WARMUP_PAIRS, OFFSET_PAIRS, OFFSET_WARMUP_PAIRS)
    };
    // Off by default, so a run with no arguments is the run the committed
    // baseline was taken with. `rexx-bench-band` measures what pinning is
    // worth; this flag is how the suite gets the same treatment once that
    // measurement says it is worth having.
    let wrapper = Wrapper {
        pin: flag_value(&arguments, "--pin"),
        counters: Counted::Nothing,
    };

    verify_axis_list();

    let oracle_binary = PathBuf::from(ORACLE_ROOT).join("bin/rexx");
    let [rust_binary, _debug] = rust_binary_candidates();
    for required in [&oracle_binary, &rust_binary] {
        if !required.is_file() {
            eprintln!(
                "rexx-bench-suite: {} is missing. Build both interpreters first; \
                 a suite that measured only one side would still print a table.",
                required.display()
            );
            return ExitCode::FAILURE;
        }
    }
    // Manifest-relative until here, so it does not depend on the caller's
    // cwd; normalised now so the report names a path a reader can paste
    // rather than one with `../..` in the middle of it.
    let rust_binary = rust_binary.canonicalize().unwrap_or(rust_binary);

    // The entire output of this phase is ratios against the oracle, so an
    // oracle that could not be identified is not a measurement to report with
    // a caveat -- it is a run that must not produce a table at all.
    let objects = oracle_objects(&oracle_binary);
    if objects.is_empty() {
        eprintln!(
            "rexx-bench-suite: `ldd {}` resolved no shared object under {ORACLE_ROOT}. \
             The launcher is a thin `main` and the interpreter is in those objects, so \
             without them nothing here identifies the build every ratio is taken against.",
            oracle_binary.display()
        );
        return ExitCode::FAILURE;
    }

    // Every child runs with its cwd here rather than in the repository or in
    // the scratchpad. The oracle resolves an unresolved call name against the
    // current directory, and a directory holding stale `.rex` files has
    // already made a probe here execute a file nobody meant to run.
    let workdir = std::env::temp_dir().join(format!("rexx-bench-suite-{}", std::process::id()));
    if let Err(error) = fs::create_dir_all(&workdir) {
        eprintln!(
            "rexx-bench-suite: cannot create {}: {error}",
            workdir.display()
        );
        return ExitCode::FAILURE;
    }

    let oracle = Side::oracle();
    let rust = Side::rust(rust_binary.clone());

    let mut report = String::new();
    let mut failures: Vec<String> = Vec::new();

    if self_check {
        report.push_str(
            "**SELF-CHECK ONLY -- one pair per axis. These numbers are not a baseline.**\n\n",
        );
    }

    write_provenance(
        &mut report,
        &oracle_binary,
        &objects,
        &rust_binary,
        pairs,
        warmup,
        offset_pairs,
        offset_warmup,
        &wrapper,
    );

    // The offset first, because every axis below is read against it.
    let offset_axis = AXES
        .iter()
        .find(|axis| axis.role == Role::Offset)
        .expect("the axis list has an offset program");
    let offset_path = rexx_bench::program_path(offset_axis.name);
    eprintln!("measuring offset ({})", offset_axis.name);
    let offset = match measure_interleaved(
        &oracle,
        &rust,
        &offset_path,
        &workdir,
        offset_pairs,
        offset_warmup,
        &wrapper,
    ) {
        Ok(paired) => paired,
        Err(error) => {
            eprintln!("rexx-bench-suite: offset axis failed: {error}");
            return ExitCode::FAILURE;
        }
    };
    write_offset(&mut report, offset_axis.name, &offset);

    let mut rows: Vec<AxisRow> = Vec::new();
    for axis in AXES.iter().filter(|axis| axis.role == Role::Loop) {
        let path = rexx_bench::program_path(axis.name);
        let iterations = match loop_count(&path) {
            Ok(count) => count,
            Err(error) => {
                eprintln!("rexx-bench-suite: {}: {error}", path.display());
                return ExitCode::FAILURE;
            }
        };
        eprintln!("measuring {} ({iterations} iterations)", axis.name);
        match measure_interleaved(&oracle, &rust, &path, &workdir, pairs, warmup, &wrapper) {
            Ok(paired) => rows.push(AxisRow {
                name: axis.name.to_string(),
                iterations,
                paired,
            }),
            Err(error) => failures.push(format!("{}: {error}", axis.name)),
        }
    }
    write_axes(&mut report, &rows, &offset);

    eprintln!("measuring rexxcps");
    let cps = measure_interleaved(
        &oracle,
        &rust,
        Path::new(REXXCPS),
        &workdir,
        pairs,
        warmup,
        &wrapper,
    );
    match &cps {
        Ok(paired) => write_rexxcps(&mut report, paired),
        Err(error) => failures.push(format!("rexxcps: {error}")),
    }

    for name in write_blocked(&mut report, &rust, &workdir, &wrapper) {
        failures.push(format!(
            "{name} is declared Role::Blocked and no longer fails; it is reported as \
             unrunnable and measured by nothing"
        ));
    }

    if !failures.is_empty() {
        report.push_str(
            "\n### This run is not a baseline\n\nEach line below is either an axis that did \
             not complete or an axis whose declared role is no longer true.\n\n",
        );
        for failure in &failures {
            let _ = writeln!(report, "* {failure}");
        }
    }

    print!("{report}");
    // Only if it is empty, which it is: nothing writes into it.
    let _ = fs::remove_dir(&workdir);

    if failures.is_empty() {
        ExitCode::SUCCESS
    } else {
        eprintln!(
            "rexx-bench-suite: {} problem(s); the report above is not a baseline",
            failures.len()
        );
        ExitCode::FAILURE
    }
}

/// The paired samples for one axis, in the order they were taken.
struct Paired {
    oracle: Vec<Duration>,
    rust: Vec<Duration>,
    /// Standard output of every sampled run on each side, kept so the report
    /// can say whether the two sides produced the same bytes. A run that died
    /// on its first clause is extremely fast, and wall time alone cannot tell
    /// that apart from a fast interpreter. `rexxcps` also needs them: its
    /// result is a number it prints rather than a time this harness takes.
    oracle_stdout: Vec<Vec<u8>>,
    rust_stdout: Vec<Vec<u8>>,
}

impl Paired {
    /// The bytes one side printed, if every sampled run on that side printed
    /// the same thing. `None` when the runs disagreed with each other, which
    /// would make any single run's output unrepresentative.
    fn stable_stdout(runs: &[Vec<u8>]) -> Option<&[u8]> {
        let first = runs.first()?;
        runs.iter()
            .all(|run| run == first)
            .then_some(first.as_slice())
    }
}

struct AxisRow {
    name: String,
    iterations: u64,
    paired: Paired,
}

/// Runs both sides alternately and returns their samples.
///
/// A non-zero exit or a signal on either side aborts the axis rather than
/// contributing a sample: a failed run's wall time measures the failure.
fn measure_interleaved(
    oracle: &Side,
    rust: &Side,
    program: &Path,
    workdir: &Path,
    pairs: usize,
    warmup: usize,
    wrapper: &Wrapper,
) -> Result<Paired, String> {
    let mut result = Paired {
        oracle: Vec::with_capacity(pairs),
        rust: Vec::with_capacity(pairs),
        oracle_stdout: Vec::with_capacity(pairs),
        rust_stdout: Vec::with_capacity(pairs),
    };
    for index in 0..(warmup + pairs) {
        let sampled = index >= warmup;
        // The oracle first inside every pair, so the two sides always sit in
        // the same order relative to whatever the machine is doing.
        for is_oracle in [true, false] {
            let side = if is_oracle { oracle } else { rust };
            let completed = run(side, program, workdir, wrapper)?;
            if !completed.succeeded() {
                return Err(format!(
                    "{} exited {:?} on {}: {}",
                    side.label,
                    completed.exit_code,
                    program.display(),
                    String::from_utf8_lossy(&completed.stderr).trim()
                ));
            }
            if sampled {
                let (times, outputs) = if is_oracle {
                    (&mut result.oracle, &mut result.oracle_stdout)
                } else {
                    (&mut result.rust, &mut result.rust_stdout)
                };
                times.push(completed.wall);
                outputs.push(completed.stdout);
            }
        }
    }
    Ok(result)
}

/// The value following `name` on the command line, if `name` is present.
fn flag_value(arguments: &[String], name: &str) -> Option<String> {
    let at = arguments.iter().position(|arg| arg == name)?;
    arguments.get(at + 1).cloned()
}

/// The `.rex` stems present in `rust/bench-programs/`, sorted.
///
/// Read from the directory rather than listed a second time, so the
/// assertion below cannot be satisfied by a copy of [`AXES`] edited in the
/// same change.
fn axis_names_on_disk() -> Vec<String> {
    let dir = rexx_bench::programs_dir();
    let entries =
        fs::read_dir(&dir).unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()));
    let mut names: Vec<String> = entries
        .flatten()
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter_map(|name| name.strip_suffix(".rex").map(str::to_string))
        .collect();
    names.sort();
    names
}

/// The suite measures every program the benchmark directory holds.
fn verify_axis_list() {
    let declared: Vec<String> = AXES.iter().map(|axis| axis.name.to_string()).collect();
    assert_eq!(
        declared,
        axis_names_on_disk(),
        "the axis list and rust/bench-programs/ disagree. A program missing \
         from AXES is a dimension this suite never measures, and the report \
         stays green over whatever is left; a name in AXES with no program \
         is an axis that cannot run"
    );
}

/// The iteration count a benchmark program's own loop bound gives.
///
/// Read out of the program text rather than restated here, so the reported
/// throughput cannot disagree with the work that was done. Every loop axis
/// writes its bound as a single `n = <digits>` line.
fn loop_count(program: &Path) -> Result<u64, String> {
    let text = fs::read_to_string(program).map_err(|error| format!("cannot read: {error}"))?;
    let mut found: Option<u64> = None;
    for line in text.lines() {
        let Some(rest) = line.trim().strip_prefix("n = ") else {
            continue;
        };
        let Ok(count) = rest.trim().parse::<u64>() else {
            continue;
        };
        if found.is_some() {
            return Err("more than one `n = <digits>` line; the loop bound is ambiguous".into());
        }
        found = Some(count);
    }
    found.ok_or_else(|| "no `n = <digits>` loop bound found".into())
}

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
///
/// The fallback is reported through `coverage`, which then reads well below
/// the target -- a self-check run at one pair has no interval, and saying so
/// is different from quietly widening the target.
fn interval_for(n: usize) -> MedianInterval {
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
/// size rather than restated, so it cannot drift from [`PAIRS`].
///
/// Takes the row rather than the slice: a slice admits the empty case, and the
/// caller has to have decided what an axis table with no axes says before it
/// can ask this.
fn interval_coverage(row: &AxisRow) -> f64 {
    interval_for(row.paired.oracle.len()).coverage
}

/// The Bonferroni lower bound on the ratio interval's coverage, from one
/// side's coverage.
///
/// Each endpoint of the ratio can miss on either side, so the joint guarantee
/// is one minus the two miss probabilities added. Clamped at zero because
/// below 50% per side the raw arithmetic goes negative, and "at least -100%"
/// is not a statement about anything -- `--self-check` takes one pair per axis
/// and lands exactly there.
fn joint_coverage_bound(per_side: f64) -> f64 {
    (1.0 - 2.0 * (1.0 - per_side)).max(0.0)
}

/// The paragraph that tells a reader what the ratio interval is worth.
///
/// Two sentences rather than one with a spliced fragment, because the two
/// cases make different claims: one reports a bound, the other reports that
/// there is no useful bound to report. The first is worded exactly as the
/// committed baseline carries it -- see
/// [`the_caveat_matches_the_committed_baseline`], which is what keeps that
/// document's "this block is the program's output byte for byte" true without
/// re-running a twelve-minute measurement to find out.
fn ratio_interval_caveat(per_side: f64) -> String {
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

fn capture(program: &str, args: &[&str]) -> String {
    Command::new(program)
        .args(args)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .unwrap_or_else(|| format!("<{program} failed>"))
}

/// Where `rexx-run` is built, release first.
///
/// `main` takes the release build and nothing else, because the profile is
/// part of what is being measured. The role check in the tests takes whichever
/// exists, because "does this program fail on this crate" does not depend on
/// the optimisation level and a `cargo test --workspace` builds only the debug
/// one.
fn rust_binary_candidates() -> [PathBuf; 2] {
    let target = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target");
    [
        target.join("release/rexx-run"),
        target.join("debug/rexx-run"),
    ]
}

/// The ooRexx shared objects `binary` actually loads, sorted.
///
/// Derived from `ldd` rather than listed, so the fingerprint set cannot fall
/// behind the build. Anything outside [`ORACLE_ROOT`] is the system's, not
/// the oracle's, and is left out: `libc` moving is not this baseline's
/// subject.
fn oracle_objects(binary: &Path) -> Vec<PathBuf> {
    let listing = capture("ldd", &[&binary.display().to_string()]);
    let mut objects: Vec<PathBuf> = listing
        .lines()
        .filter_map(|line| line.split_once(" => "))
        .filter_map(|(_, resolved)| resolved.split(" (").next())
        .map(|path| PathBuf::from(path.trim()))
        .filter(|path| path.starts_with(ORACLE_ROOT))
        .collect();
    objects.sort();
    objects.dedup();
    objects
}

#[allow(clippy::too_many_arguments)]
fn write_provenance(
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
    //
    // Resolved from `ldd` rather than named here, so an object the oracle
    // build gains later is fingerprinted without anyone remembering to add
    // it. The two objects loaded today carry different dates, which is the
    // observation that a hardcoded list is a list that can be short.
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

fn write_offset(report: &mut String, name: &str, paired: &Paired) {
    let oracle = Stats::of(&mut seconds(&paired.oracle));
    let rust = Stats::of(&mut seconds(&paired.rust));
    let _ = writeln!(report, "### Fixed per-process offset (`{name}.rex`)\n");
    let _ = writeln!(
        report,
        "**Not comparable, and not a pass.** This crate has no `CoreClasses.orx` bootstrap yet \
         (Phase 5), so it starts fast by not doing the work the oracle does at startup. The two \
         numbers below are each side's own fixed cost, reported so every axis above can be read \
         net of it -- not as a result about which interpreter starts faster.\n"
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

fn write_axes(report: &mut String, rows: &[AxisRow], offset: &Paired) {
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

fn write_rexxcps(report: &mut String, paired: &Paired) {
    let oracle_wall = Stats::of(&mut seconds(&paired.oracle));
    let rust_wall = Stats::of(&mut seconds(&paired.rust));
    let _ = writeln!(report, "### `samples/rexxcps.rex`\n");
    let _ = writeln!(
        report,
        "The oracle's own clauses-per-second benchmark, run from the read-only C++ tree. It \
         self-calibrates: a trial that comes in at or under a second is run again at twice the \
         count, so the two sides do **different amounts of work** and their wall times are not \
         directly comparable. The clauses-per-second figure each side prints is per clause and is \
         the comparable one. Each side's `Averaged:` line is quoted so the asymmetry is visible \
         rather than inferred.\n"
    );
    let _ = writeln!(
        report,
        "| side | wall median | wall interval | `Averaged:` |"
    );
    let _ = writeln!(report, "|---|---:|---|---|");
    for (label, stats, runs) in [
        ("oracle", oracle_wall, &paired.oracle_stdout),
        ("this crate", rust_wall, &paired.rust_stdout),
    ] {
        let _ = writeln!(
            report,
            "| {label} | {:.4} s | {:.4} - {:.4} s | {} |",
            stats.median,
            stats.low,
            stats.high,
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

fn quoted_line(stdout: &[u8], marker: &str) -> String {
    String::from_utf8_lossy(stdout)
        .lines()
        .find(|line| line.contains(marker))
        .map(|line| format!("`{}`", line.trim()))
        .unwrap_or_else(|| "*absent*".to_string())
}

fn parse_cps(stdout: &[u8]) -> Option<u64> {
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
///
/// **The role is checked, not trusted.** [`verify_axis_list`] pins which
/// programs exist; it says nothing about whether a program still fails. When
/// Phase 5 lands message sends these three start exiting 0, and with the role
/// unchecked they would keep appearing under this heading with status 0 and an
/// empty message, timed by nothing -- three dimensions dropping out of the
/// measurement while the run stayed green, which is the same defect the axis
/// pin exists to prevent arriving through a different door.
fn write_blocked(
    report: &mut String,
    rust: &Side,
    workdir: &Path,
    wrapper: &Wrapper,
) -> Vec<String> {
    let mut no_longer_blocked = Vec::new();
    let _ = writeln!(report, "### Axes this crate cannot run\n");
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The axis list names exactly the programs the benchmark directory holds.
    #[test]
    fn the_axis_list_covers_every_bench_program() {
        verify_axis_list();
    }

    /// Every loop axis has a readable, unambiguous loop bound, and the two
    /// axes that are not loop axes are not asked for one.
    #[test]
    fn every_loop_axis_has_a_loop_bound() {
        for axis in AXES.iter().filter(|axis| axis.role == Role::Loop) {
            let path = rexx_bench::program_path(axis.name);
            let count =
                loop_count(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
            assert!(count > 0, "{} has a zero loop bound", path.display());
        }
    }

    /// Every axis declared `Role::Blocked` really does fail on this crate.
    ///
    /// The names pin is not enough on its own. It catches an axis leaving the
    /// list; it cannot catch an axis staying in the list under a role that has
    /// stopped being true. When Phase 5 lands message sends these three exit
    /// 0, and without this they would go on being printed as unrunnable with
    /// status 0 and an empty message while nothing timed them -- the pin's own
    /// failure mode, reached by a different route. Red here forces the
    /// decision instead.
    #[test]
    fn every_blocked_axis_still_fails_on_this_crate() {
        let binary = rust_binary_candidates()
            .into_iter()
            .find(|path| path.is_file())
            .unwrap_or_else(|| {
                panic!(
                    "neither target/release/rexx-run nor target/debug/rexx-run exists. This \
                     test runs the blocked axes through this crate, and skipping instead \
                     would let the roles below go unchecked while the run stayed green. \
                     `cargo build -p rexx-exec --bin rexx-run` first."
                )
            });
        let side = Side {
            label: "rust",
            binary,
            env: Vec::new(),
        };
        let dir = std::env::temp_dir().join(format!(
            "rexx-bench-suite-blocked-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        fs::create_dir_all(&dir).expect("temporary directory");

        let blocked: Vec<&Axis> = AXES.iter().filter(|a| a.role == Role::Blocked).collect();
        assert!(
            !blocked.is_empty(),
            "no axis is declared Role::Blocked, so this test asserts nothing"
        );
        for axis in blocked {
            let path = rexx_bench::program_path(axis.name);
            // Through the same capped, directory-pinned wrapper the suite
            // uses. `alloc.rex` allocates without bound if it ever starts
            // running, and an uncapped in-process run of it would take the
            // machine's memory rather than the test.
            let completed =
                run(&side, &path, &dir, &Wrapper::default()).expect("the runner launches");
            assert!(
                !completed.succeeded(),
                "{} is declared Role::Blocked but exited 0. The suite would report it as \
                 unrunnable and time it with nothing. Give it Role::Loop and a loop bound, \
                 or decide deliberately that it stays out",
                axis.name
            );
        }
        fs::remove_dir(&dir).ok();
    }

    /// `startup.rex` has no loop, so asking it for one is an error rather
    /// than a silently fabricated count. Pins that `loop_count` reports
    /// absence instead of defaulting.
    #[test]
    fn a_program_without_a_loop_has_no_bound() {
        let path = rexx_bench::program_path("startup");
        assert!(loop_count(&path).is_err());
    }

    /// The Bonferroni bound is stated, and it is clamped rather than allowed
    /// to go negative. A `--self-check` run has one pair per axis, whose
    /// per-side coverage is zero, and the unclamped arithmetic printed
    /// "at least -100.0%" into a report that exits 0.
    #[test]
    fn the_joint_coverage_bound_is_clamped() {
        let nine = interval_for(9).coverage;
        assert!((joint_coverage_bound(nine) - (1.0 - 2.0 * (1.0 - nine))).abs() < 1e-12);
        assert!(joint_coverage_bound(nine) > 0.92 && joint_coverage_bound(nine) < 0.93);
        assert_eq!(joint_coverage_bound(interval_for(1).coverage), 0.0);
        assert_eq!(joint_coverage_bound(0.25), 0.0);

        // The bug's exact shape, not "contains no minus sign" -- the prose
        // carries `--` for its dashes and an assertion on that is red for a
        // reason that has nothing to do with coverage.
        let vacuous = ratio_interval_caveat(interval_for(1).coverage);
        assert!(vacuous.contains("that bound is vacuous"), "{vacuous}");
        assert!(vacuous.contains("only 0.0%"), "{vacuous}");
        assert!(
            !vacuous.contains("-100"),
            "a negative coverage reached the report: {vacuous}"
        );
    }

    /// The caveat this suite emits at [`PAIRS`] is the one the committed
    /// baseline carries.
    ///
    /// `perf-baseline.md` claims its whole block is this program's output byte
    /// for byte, and 4d-2 diffs a fresh run against that block to find oracle
    /// drift. Every other line of the block is a measured value that moves on
    /// every run, so a diff there is self-explanatory; this paragraph is the
    /// one piece of *prose* the harness emits, and prose that drifts would
    /// show up in that diff looking exactly like a finding. Pinned here so it
    /// cannot drift unnoticed, and so a wording change costs a re-paste rather
    /// than a re-measurement nobody budgeted for.
    #[test]
    fn the_caveat_matches_the_committed_baseline() {
        let baseline = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../docs/superpowers/plans/perf-baseline.md");
        let text = fs::read_to_string(&baseline)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", baseline.display()));
        let emitted = ratio_interval_caveat(interval_for(PAIRS).coverage);
        assert!(
            text.contains(&emitted),
            "the committed baseline does not contain the caveat this suite emits at {PAIRS} \
             pairs. Either the wording changed and the block needs re-pasting, or the block is \
             no longer this program's output and the document says it is.\n\nemitted:\n{emitted}"
        );
    }

    #[test]
    fn the_performance_line_parses() {
        let stdout = b"        Averaged: 200 x 100 iterations of 1000 clauses (over 1.2s)\n\n     Performance: 17137975 REXX clauses per second\n";
        assert_eq!(parse_cps(stdout), Some(17_137_975));
        assert_eq!(parse_cps(b"no such line\n"), None);
    }

    #[test]
    fn a_quoted_line_that_is_absent_says_so() {
        assert_eq!(quoted_line(b"nothing here\n", "Averaged:"), "*absent*");
        assert_eq!(
            quoted_line(b"  Averaged: 1 x 1\n", "Averaged:"),
            "`Averaged: 1 x 1`"
        );
    }

    /// The two sides really do alternate, and every child really does run in
    /// the directory it was given.
    ///
    /// The whole methodological claim of this harness is the interleaving,
    /// and nothing in the report it prints could distinguish an alternating
    /// run from a side-at-a-time one -- the tables would look the same, and
    /// the drift the alternation exists to cancel would be silently baked
    /// into the ratios instead. So the order is observed rather than
    /// asserted about: two `Side`s differing only in an environment variable
    /// run a script that appends that variable to a file in the working
    /// directory, and the file spells out the order the children ran in.
    /// `ooooRRRR` is what a side-at-a-time implementation would leave here.
    #[test]
    fn the_two_sides_alternate_and_run_in_the_working_directory() {
        let dir = std::env::temp_dir().join(format!(
            "rexx-bench-suite-order-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        fs::create_dir_all(&dir).expect("temporary directory");
        let script = dir.join("probe.sh");
        // Relative, so it lands in whatever directory the child was put in.
        fs::write(&script, "printf %s \"$REXX_BENCH_SIDE\" >> order\n").expect("probe script");

        let side = |mark: &str| Side {
            label: "probe",
            binary: PathBuf::from("/bin/sh"),
            env: vec![("REXX_BENCH_SIDE".to_string(), mark.to_string())],
        };
        measure_interleaved(
            &side("o"),
            &side("R"),
            &script,
            &dir,
            3,
            1,
            &Wrapper::default(),
        )
        .expect("the probe script runs");

        let order = fs::read_to_string(dir.join("order")).expect("the children wrote in `dir`");
        assert_eq!(
            order, "oRoRoRoR",
            "one warm-up pair plus three sampled pairs"
        );

        fs::remove_file(dir.join("order")).ok();
        fs::remove_file(&script).ok();
        fs::remove_dir(&dir).ok();
    }

    /// A side that fails aborts its axis instead of contributing the wall
    /// time of the failure. Without this, an interpreter that died on its
    /// first clause would report as extremely fast.
    #[test]
    fn a_failing_side_aborts_the_axis() {
        let dir = std::env::temp_dir().join(format!(
            "rexx-bench-suite-fail-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        fs::create_dir_all(&dir).expect("temporary directory");
        let script = dir.join("fails.sh");
        fs::write(&script, "echo broken >&2\nexit 3\n").expect("probe script");

        let side = |label: &'static str| Side {
            label,
            binary: PathBuf::from("/bin/sh"),
            env: Vec::new(),
        };
        let outcome = measure_interleaved(
            &side("oracle"),
            &side("rust"),
            &script,
            &dir,
            3,
            0,
            &Wrapper::default(),
        );
        let Err(error) = outcome else {
            panic!("a side exiting 3 is not a measurement");
        };
        assert!(error.contains("oracle exited Some(3)"), "{error}");
        assert!(error.contains("broken"), "{error}");

        fs::remove_file(&script).ok();
        fs::remove_dir(&dir).ok();
    }
}
