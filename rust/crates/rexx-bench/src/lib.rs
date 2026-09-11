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

//! Shared plumbing for the Task 0.7 benchmark harness (`benches/interpreter.rs`),
//! the cold-start timer (`src/bin/rexx-time.rs`), the interleaved
//! two-interpreter suite (`src/bin/rexx-bench-suite.rs`) and the between-run
//! band tool (`src/bin/rexx-bench-band.rs`): resolving `REXX_BENCH_BINARY`
//! into a runnable `Interpreter`, locating `bench-programs/`, the subprocess
//! timing core in [`timing`], the one capped, directory-pinned way of
//! launching either interpreter in [`child`], and the in-phase arm-against-arm
//! measurement in [`arms`].

pub mod arms;
pub mod child;
pub mod timing;

use rexx_oracle::Interpreter;
use std::path::{Path, PathBuf};

/// Names the interpreter binary under test. Read by both the criterion
/// harness and, indirectly, by whatever invokes `rexx-time` on it.
pub const BINARY_VAR: &str = "REXX_BENCH_BINARY";

/// The programs the criterion harness (`benches/interpreter.rs`) benchmarks,
/// in report order. `startup` is listed first because it is the one program
/// this suite is not sizing for 0.5-2s -- see `bench-programs/startup.rex`.
pub static PROGRAMS: &[&str] = &[
    "startup",
    "dispatch",
    "dispatchclass",
    "varlookup",
    "compound",
    "strings",
    "arith",
    "alloc",
    "alloc4c",
    "emptyloop",
    // Added 2026-09-10 with the coverage they close: `parse` and `textnum`
    // because `rexxcps` performs 5,580,002 text-to-number conversions and
    // 2.24M `PARSE` ops that no program here exercised, and `decloop`/
    // `decrender` because decimal loop control is a different axis from
    // integer control -- 1.22x against 1.63x -- and carried the whole of
    // `rexxcps`' excess over its other primitives.
    "parse",
    "textnum",
    "decloop",
    "decrender",
];

/// Programs in `bench-programs/` that the criterion harness deliberately does
/// not benchmark.
pub static NOT_BENCHMARKED: &[&str] = &["heapshape"];

/// Resolves `REXX_BENCH_BINARY` into an `Interpreter`, deriving its library
/// search path the same way `rexx-diff` does (`rexx-oracle/src/bin/rexx-diff.rs`):
/// the binary's own directory, plus a sibling `lib/`, which is where
/// `build/bin/rexx` finds `build/lib/*.so`.
pub fn interpreter_under_test() -> Interpreter {
    let raw = std::env::var(BINARY_VAR)
        .unwrap_or_else(|_| panic!("set {BINARY_VAR} to the interpreter binary to benchmark"));
    let binary = std::fs::canonicalize(&raw)
        .unwrap_or_else(|e| panic!("{BINARY_VAR}={raw} does not resolve: {e}"));
    let library_paths = binary
        .parent()
        .map(|dir| vec![dir.to_path_buf(), dir.join("../lib")])
        .unwrap_or_default();
    Interpreter {
        binary,
        library_paths,
    }
}

/// Directory holding the benchmark programs, resolved relative to this
/// crate's own manifest so it does not depend on the caller's cwd.
pub fn programs_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../bench-programs")
}

/// Full path to one benchmark program, e.g. `program_path("compound")`.
pub fn program_path(name: &str) -> PathBuf {
    programs_dir().join(format!("{name}.rex"))
}

/// Directory holding the control programs, resolved the same way.
pub fn controls_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../bench-control")
}

/// REXXCPS 2.2 with its loop counts fixed, resolved the same way.
pub fn rexxcps_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../bench-rexxcps/rexxcps.rex")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every program in `bench-programs/` is either benchmarked or exempted.
    #[test]
    fn the_benchmark_list_accounts_for_every_program() {
        let mut accounted: Vec<String> = PROGRAMS
            .iter()
            .chain(NOT_BENCHMARKED)
            .map(|name| (*name).to_string())
            .collect();
        accounted.sort();
        let before = accounted.len();
        accounted.dedup();
        assert_eq!(
            before,
            accounted.len(),
            "a program is named twice across PROGRAMS and NOT_BENCHMARKED"
        );

        let dir = programs_dir();
        let entries = std::fs::read_dir(&dir)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()));
        let mut on_disk: Vec<String> = entries
            .flatten()
            .filter_map(|entry| entry.file_name().into_string().ok())
            .filter_map(|name| name.strip_suffix(".rex").map(str::to_string))
            .collect();
        on_disk.sort();

        assert_eq!(
            accounted, on_disk,
            "PROGRAMS plus NOT_BENCHMARKED does not match bench-programs/. A program on \
             disk and in neither list is a dimension the criterion harness has stopped \
             covering, with nothing to notice; a name in a list and not on disk cannot run"
        );
    }

    /// A control differs from the axis it controls by a known amount, and by
    /// nothing else.
    #[test]
    fn the_control_differs_from_its_axis_only_in_the_loop_bound() {
        let axis = program_path("alloc4c");
        let control = controls_dir().join("alloc4c-101.rex");
        let read = |path: &Path| {
            std::fs::read_to_string(path).unwrap_or_else(|e| panic!("cannot read {path:?}: {e}"))
        };
        let bound = |text: &str| -> u64 {
            let mut found = None;
            for line in text.lines() {
                if let Some(rest) = line.trim().strip_prefix("n = ")
                    && let Ok(count) = rest.trim().parse::<u64>()
                {
                    assert!(found.is_none(), "more than one loop bound");
                    found = Some(count);
                }
            }
            found.expect("a loop bound")
        };
        let axis_text = read(&axis);
        let control_text = read(&control);

        let strip = |text: &str| -> String {
            text.lines()
                .filter(|line| !line.trim().starts_with("n = "))
                .collect::<Vec<_>>()
                .join("\n")
        };
        assert_eq!(
            strip(&axis_text),
            strip(&control_text),
            "the control and its axis differ somewhere other than the loop bound, so the \
             difference between them is no longer the known amount it is used as"
        );

        let axis_bound = bound(&axis_text);
        let control_bound = bound(&control_text);
        assert_eq!(
            control_bound * 100,
            axis_bound * 101,
            "the control is {control_bound} iterations against the axis's {axis_bound}, which \
             is not the 1% the escalation rule runs it for"
        );
    }

    /// Whether a benchmark program measures and reports its own timing.
    fn reports_its_own_timing(program: &str) -> bool {
        let path = program_path(program);
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
        text.to_ascii_lowercase().contains("time(")
    }

    /// Each exemption is true of the program it names, and no benchmarked
    /// program qualifies for one.
    #[test]
    fn the_exemptions_are_true_of_the_programs_they_name() {
        assert!(
            !NOT_BENCHMARKED.is_empty(),
            "no exemption is claimed, so this test asserts nothing"
        );
        for name in NOT_BENCHMARKED {
            assert!(
                reports_its_own_timing(name),
                "{name} is exempt from the criterion harness, but it does not call TIME() and \
                 so reports no timing of its own. That is the only reason NOT_BENCHMARKED \
                 recognises. Benchmark it, or give the exemption a reason this test can check"
            );
        }
        for name in PROGRAMS {
            assert!(
                !reports_its_own_timing(name),
                "{name} is benchmarked by the criterion harness and also reports a timing of \
                 its own, so the harness is measuring the sum of parts the program separates. \
                 It belongs in NOT_BENCHMARKED"
            );
        }
    }
}
