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

//! The in-phase measurement front end: both engine arms of one build, both
//! instruments, two problem sizes, one sitting.
//! ```text
//! rexx-arms --build base=/path/to/rexx-run --build head=target/release/rexx-run \
//!           --axis varlookup --axis emptyloop --rounds 5 \
//!           --task 7-M2 --commit ced6c209 --baseline bench-baselines/phase-4e-arms.tsv
//! ```

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use rexx_bench::arms::{
    Arm, Build, Figure, Instrument, MINIMUM_ROUNDS, Measured, Sitting, Size, Workload, measure,
};

/// Number of fields in a row. Named so [`row`] takes an array of exactly this
/// length and the compiler, rather than a reader, keeps the header and the
/// rows in step.
const FIELDS: usize = 12;

/// Columns of the tab-separated stream, in order.
fn columns() -> [String; FIELDS] {
    let [median, min, max, rounds] = Figure::column_names("value");
    [
        "task".to_string(),
        "commit".to_string(),
        "axis".to_string(),
        "build".to_string(),
        "scope".to_string(),
        "arm".to_string(),
        "size".to_string(),
        "instrument".to_string(),
        median,
        min,
        max,
        rounds,
    ]
}

/// One tab-separated row. Fixed-length array rather than a slice, so a field
/// added to [`columns`] and not to the emission site does not compile.
fn row(fields: [String; FIELDS]) -> String {
    fields.join("\t")
}

/// What a row is measuring. The `arm` and `size` columns mean different things
/// per scope, and this is the one place that says which.
struct Scope {
    /// The `scope` column.
    name: &'static str,
    /// The `arm` column: one arm's label, or the pair a ratio is over.
    arm: String,
    /// The `size` column, or `-` where the figure spans both sizes.
    size: &'static str,
}

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    match run(&arguments) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("rexx-arms: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run(arguments: &[String]) -> Result<(), String> {
    let builds: Vec<Build> = repeated(arguments, "--build")
        .iter()
        .map(|spec| Build::parse(spec))
        .collect::<Result<_, _>>()?;
    if builds.is_empty() {
        return Err("no --build LABEL=PATH given".to_string());
    }
    let axes = repeated(arguments, "--axis");
    if axes.is_empty() {
        return Err("no --axis given".to_string());
    }
    let rounds = single(arguments, "--rounds")
        .map(|value| value.parse::<usize>().map_err(|e| format!("--rounds: {e}")))
        .transpose()?
        .unwrap_or(MINIMUM_ROUNDS);
    let pin = single(arguments, "--pin");
    let task = single(arguments, "--task").unwrap_or_else(|| "-".to_string());
    let commit = single(arguments, "--commit").unwrap_or_else(|| "-".to_string());
    let baseline = single(arguments, "--baseline").map(PathBuf::from);
    let raw = single(arguments, "--raw").map(PathBuf::from);
    let workdir = single(arguments, "--workdir").map_or_else(
        || std::env::temp_dir().join(format!("rexx-arms-{}", std::process::id())),
        PathBuf::from,
    );

    if arguments.iter().any(|argument| argument == "--header") {
        println!("{}", row(columns()));
    }

    let mut appended: Vec<String> = Vec::new();
    for axis in &axes {
        let path = if axis.contains('/') {
            PathBuf::from(axis)
        } else {
            rexx_bench::program_path(axis)
        };
        let workload = Workload::read(&path)?;
        let measured = measure(&builds, &workload, rounds, &workdir, pin.clone(), |note| {
            eprintln!("rexx-arms: {note}");
        })?;
        for line in report(&measured, &task, &commit) {
            println!("{line}");
            appended.push(line);
        }
        if let Some(path) = &raw {
            write_raw(path, measured.sitting(), &task)?;
        }
        summarise(&measured);
    }

    if let Some(path) = baseline {
        let existed = path.exists();
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)
            .map_err(|error| format!("{}: {error}", path.display()))?;
        if !existed {
            writeln!(file, "{}", row(columns()))
                .map_err(|error| format!("{}: {error}", path.display()))?;
        }
        for line in &appended {
            writeln!(file, "{line}").map_err(|error| format!("{}: {error}", path.display()))?;
        }
        eprintln!(
            "rexx-arms: appended {} rows to {}",
            appended.len(),
            path.display()
        );
    }
    Ok(())
}

/// Every figure this sitting supports, as rows.
fn report(measured: &Measured, task: &str, commit: &str) -> Vec<String> {
    let sitting = measured.sitting();
    let mut lines = Vec::new();
    let mut emit = |build: &str, scope: Scope, instrument: Instrument, figure: Option<Figure>| {
        let Some(figure) = figure else { return };
        let [median, min, max, rounds] = figure.columns();
        lines.push(row([
            task.to_string(),
            commit.to_string(),
            sitting.axis().to_string(),
            build.to_string(),
            scope.name.to_string(),
            scope.arm,
            scope.size.to_string(),
            instrument.label().to_string(),
            median,
            min,
            max,
            rounds,
        ]));
    };

    let sizes: Vec<Size> = match measured {
        Measured::Scaled(_) => vec![Size::Small, Size::Large],
        Measured::Fixed(_) => vec![Size::Small],
    };
    for (index, build) in sitting.builds().iter().enumerate() {
        for instrument in Instrument::BOTH {
            for size in &sizes {
                for arm in Arm::BOTH {
                    emit(
                        &build.label,
                        Scope {
                            name: "absolute",
                            arm: arm.label().to_string(),
                            size: size.label(),
                        },
                        instrument,
                        sitting.absolute(index, arm, *size, instrument),
                    );
                }
            }
            if let Measured::Scaled(scaled) = measured {
                for arm in Arm::BOTH {
                    emit(
                        &build.label,
                        Scope {
                            name: "per_pass",
                            arm: arm.label().to_string(),
                            size: "-",
                        },
                        instrument,
                        scaled.per_pass(index, arm, instrument),
                    );
                    emit(
                        &build.label,
                        Scope {
                            name: "fixed",
                            arm: arm.label().to_string(),
                            size: "-",
                        },
                        instrument,
                        scaled.fixed(index, arm, instrument),
                    );
                }
            }
        }
    }

    // Cross-build ratios, which are a different and weaker kind of comparison
    // -- see `Sitting::across_builds`. Emitted against the first build, which
    // is the one a caller names first and by convention the base.
    for (index, build) in sitting.builds().iter().enumerate().skip(1) {
        for instrument in Instrument::BOTH {
            for size in &sizes {
                for arm in Arm::BOTH {
                    emit(
                        &format!("{}>{}", sitting.builds()[0].label, build.label),
                        Scope {
                            name: "across_builds",
                            arm: arm.label().to_string(),
                            size: size.label(),
                        },
                        instrument,
                        sitting.across_builds(0, index, arm, *size, instrument),
                    );
                }
            }
        }
    }
    lines
}

/// Every individual reading, one line per run, appended to `path`.
fn write_raw(path: &Path, sitting: &Sitting, task: &str) -> Result<(), String> {
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|error| format!("{}: {error}", path.display()))?;
    for (round, build, arm, size, reading) in sitting.runs() {
        writeln!(
            file,
            "{task}\t{}\t{build}\t{}\t{}\t{round}\t{}\t{}",
            sitting.axis(),
            arm.label(),
            size.label(),
            reading.count(Instrument::Instructions),
            reading.count(Instrument::Cycles)
        )
        .map_err(|error| format!("{}: {error}", path.display()))?;
    }
    Ok(())
}

/// The human-readable table, on standard error beside the progress notes.
fn summarise(measured: &Measured) {
    let sitting = measured.sitting();
    // **Absolutes by build, where this printed an IR-over-tree-walker ratio.**
    // One engine leaves nothing to divide; what a caller compares now is one
    // build against another, which `across_builds` below already emits.
    eprintln!("rexx-arms: {} -- by build", sitting.axis());
    for (index, build) in sitting.builds().iter().enumerate() {
        if let Measured::Scaled(scaled) = measured {
            for instrument in Instrument::BOTH {
                eprintln!(
                    "  {:<10} {:<14} per pass {}",
                    build.label,
                    instrument.label(),
                    show(scaled.per_pass(index, Arm::Ir, instrument)),
                );
            }
        }
    }
}

fn show(figure: Option<Figure>) -> String {
    figure.map_or_else(|| "-".to_string(), |value| value.to_string())
}

/// Every `--flag VALUE` pair with this flag, in the order given.
fn repeated(arguments: &[String], flag: &str) -> Vec<String> {
    arguments
        .windows(2)
        .filter(|pair| pair[0] == flag)
        .map(|pair| pair[1].clone())
        .collect()
}

/// The last `--flag VALUE` with this flag.
fn single(arguments: &[String], flag: &str) -> Option<String> {
    repeated(arguments, flag).pop()
}
