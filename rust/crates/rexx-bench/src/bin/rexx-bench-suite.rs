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

use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::time::Duration;

use rexx_bench::arms::Arm;
use rexx_bench::child::{Counted, Counters, ORACLE_ROOT, Side, Wrapper, parse_counters, run};

// Renders the paired samples below into the markdown report `main` prints.
// `#[path]` is needed here: a `[[bin]]` crate root's implicit submodule
// directory is the one it sits in, not one named after its own stem.
#[path = "rexx-bench-suite/report.rs"]
mod report;

/// Paired measurements per axis.
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

/// REXXCPS 2.2 with its loop counts fixed, in this repository.
fn rexxcps_program() -> std::path::PathBuf {
    rexx_bench::rexxcps_path()
}

/// What the suite does with one benchmark program.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Role {
    /// Timed on both sides. Iterations per second comes from the program's
    /// own loop bound.
    Loop,
    /// Timed on both sides and reported as the fixed per-process offset
    /// rather than as an axis. Its two sides **are** comparable: each pays
    /// the startup cost of the same Rexx-written library, the oracle by
    /// restoring its saved image and this crate by parsing and installing
    /// the `.orx` sources that image was built from.
    Offset,
    /// This crate cannot run it. Reported with the exit status and message it
    /// actually produced, never omitted -- an axis that quietly leaves the
    /// table is the failure this list exists to prevent.
    Blocked,
    /// Run on both sides interleaved like a [`Role::Loop`] axis and reported
    /// from the figures the program prints, because the process wall time is
    /// the sum of parts the program itself separates.
    SelfTimed,
}

/// One benchmark axis: the stem of a file in `rust/bench-programs/`.
struct Axis {
    name: &'static str,
    role: Role,
}

/// Every program in `rust/bench-programs/`, in sorted order.
const AXES: &[Axis] = &[
    Axis {
        name: "alloc",
        role: Role::Loop,
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
        name: "assign",
        role: Role::Loop,
    },
    Axis {
        name: "compound",
        role: Role::Loop,
    },
    Axis {
        name: "decloop",
        role: Role::Loop,
    },
    Axis {
        name: "decrender",
        role: Role::Loop,
    },
    Axis {
        name: "dispatch",
        role: Role::Loop,
    },
    Axis {
        name: "dispatchclass",
        role: Role::Loop,
    },
    Axis {
        name: "emptyloop",
        role: Role::Loop,
    },
    Axis {
        name: "heapshape",
        role: Role::SelfTimed,
    },
    Axis {
        name: "nop",
        role: Role::Loop,
    },
    Axis {
        name: "parse",
        role: Role::Loop,
    },
    Axis {
        name: "sayloop",
        role: Role::Loop,
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
        name: "textnum",
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
    // Which engine "this crate" means, named rather than inherited, and
    // printed in the provenance block beside every other identity a ratio
    // here depends on. The default is the engine `rexx-run` ships, so a run
    // with no arguments measures what this crate now is; `phase-4e-anchor.md`
    // and `perf-baseline.md` were taken before this choice existed and are
    // tree-walker figures, so reproducing either needs
    // `--engine tree-walker`. An unrecognised value is refused rather than
    // defaulted, exactly as `rexx-run` refuses one.
    // **No `--engine`.** It used to choose an arm; accepting `tree-walker`
    // now would set a `REXX_ENGINE` nothing reads and label the rows for an
    // engine that did not run them.
    let arm = Arm::Ir;
    let (pairs, warmup, offset_pairs, offset_warmup) = if self_check {
        (1, 0, 3, 0)
    } else {
        (PAIRS, WARMUP_PAIRS, OFFSET_PAIRS, OFFSET_WARMUP_PAIRS)
    };
    // Off by default, so a run with no arguments is the run the committed
    // baseline was taken with. `rexx-bench-band` measures what pinning is
    // worth; this flag is how the suite gets the same treatment once that
    // measurement says it is worth having.
    // Every run is counted as well as timed. The three quantities then
    // describe the *same* executions rather than three sets of them, which is
    // what lets a row's cycles and its wall time be read against each other.
    let wrapper = Wrapper {
        pin: flag_value(&arguments, "--pin"),
        counters: Counted::User,
    };

    // Settled here rather than at each reading: a suite that stopped counting
    // would otherwise print its counter columns empty, and a report missing a
    // column reads like an axis nobody measured rather than like a harness
    // that was reconfigured.
    assert!(
        wrapper.counters.events().is_some(),
        "the suite reports cycles and instructions beside every wall time, so it must run counted"
    );

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
    let rust = Side::rust(rust_binary.clone(), arm);

    let mut report = String::new();
    let mut failures: Vec<String> = Vec::new();

    if self_check {
        report.push_str(
            "**SELF-CHECK ONLY -- one pair per axis. These numbers are not a baseline.**\n\n",
        );
    }

    report::write_provenance(
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
    report::write_offset(&mut report, offset_axis.name, &offset);

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
    report::write_axes(&mut report, &rows, &offset);
    report::write_counters(&mut report, &rows);

    eprintln!("measuring rexxcps");
    let cps = measure_interleaved(
        &oracle,
        &rust,
        &rexxcps_program(),
        &workdir,
        pairs,
        warmup,
        &wrapper,
    );
    match &cps {
        Ok(paired) => report::write_rexxcps(&mut report, paired),
        Err(error) => failures.push(format!("rexxcps: {error}")),
    }

    let mut self_timed = AXES
        .iter()
        .filter(|axis| axis.role == Role::SelfTimed)
        .peekable();
    if self_timed.peek().is_some() {
        let _ = writeln!(report, "### Axes that report their own figures\n");
    }
    for axis in self_timed {
        let path = rexx_bench::program_path(axis.name);
        eprintln!("measuring {} (its own figures)", axis.name);
        match measure_interleaved(&oracle, &rust, &path, &workdir, pairs, warmup, &wrapper) {
            Ok(paired) => report::write_self_timed(&mut report, axis.name, pairs, &paired),
            Err(error) => failures.push(format!("{}: {error}", axis.name)),
        }
    }

    for name in report::write_blocked(&mut report, &rust, &workdir, &wrapper) {
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
    /// `cycles:u` and `instructions:u` for every sampled run, in the same
    /// order as the wall times beside them and from the same executions.
    oracle_counters: Vec<Counters>,
    rust_counters: Vec<Counters>,
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
        oracle_counters: Vec::with_capacity(pairs),
        rust_counters: Vec::with_capacity(pairs),
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
                // Refused whole rather than recorded as absent. `perf` reports
                // a multiplexed-out event *scaled up* to what it would have
                // been, in the same column and format as an exact count, and
                // `parse_counters` declines that; a row silently missing its
                // counters would read as an axis this suite measured.
                // A wrapper that counts must produce a reading, and it is
                // refused whole rather than recorded as absent: `perf` reports
                // a multiplexed-out event *scaled up* to what it would have
                // been, in the same column and format as an exact count, and
                // `parse_counters` declines that. Whether the suite counts at
                // all is settled once in `main`; this stays conditional so the
                // alternation test can drive it with a shell probe on a
                // machine without `perf`.
                let counters = match wrapper.counters.events() {
                    None => None,
                    Some(events) => Some(parse_counters(&completed.stderr, events).ok_or_else(|| {
                        format!(
                            "no complete {}/{} reading from {} on {}. Is `perf` installed, and is `kernel.perf_event_paranoid` low enough to count a child?",
                            events[0],
                            events[1],
                            side.label,
                            program.display()
                        )
                    })?),
                };
                let (times, outputs, counts) = if is_oracle {
                    (
                        &mut result.oracle,
                        &mut result.oracle_stdout,
                        &mut result.oracle_counters,
                    )
                } else {
                    (
                        &mut result.rust,
                        &mut result.rust_stdout,
                        &mut result.rust_counters,
                    )
                };
                times.push(completed.wall);
                outputs.push(completed.stdout);
                counts.extend(counters);
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
fn rust_binary_candidates() -> [PathBuf; 2] {
    let target = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target");
    [
        target.join("release/rexx-run"),
        target.join("debug/rexx-run"),
    ]
}

/// The ooRexx shared objects `binary` actually loads, sorted.
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

#[cfg(test)]
mod tests {
    use super::report::{
        interval_for, joint_coverage_bound, parse_cps, quoted_line, ratio_interval_caveat,
    };
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

    /// Every axis declared `Role::Blocked` really does fail on this crate, and
    /// every axis declared `Role::SelfTimed` really does run.
    #[test]
    fn every_declared_runnability_still_holds() {
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

        let checked: Vec<&Axis> = AXES
            .iter()
            .filter(|a| matches!(a.role, Role::Blocked | Role::SelfTimed))
            .collect();
        assert!(
            !checked.is_empty(),
            "no axis is declared Role::Blocked or Role::SelfTimed, so this test asserts nothing"
        );
        for axis in checked {
            let path = rexx_bench::program_path(axis.name);
            // Through the same capped, directory-pinned wrapper the suite
            // uses, so a program that outgrows the cap fails here rather than
            // taking the machine's memory.
            let completed =
                run(&side, &path, &dir, &Wrapper::default()).expect("the runner launches");
            match axis.role {
                Role::Blocked => assert!(
                    !completed.succeeded(),
                    "{} is declared Role::Blocked but exited 0. The suite would report it as \
                     unrunnable and time it with nothing. Give it Role::Loop and a loop bound, \
                     or decide deliberately that it stays out",
                    axis.name
                ),
                _ => assert!(
                    completed.succeeded(),
                    "{} is declared Role::SelfTimed but exited {:?}: {}. The suite would report \
                     its figures as an axis it measured. Give it Role::Blocked",
                    axis.name,
                    completed.exit_code,
                    String::from_utf8_lossy(&completed.stderr).trim()
                ),
            }
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
