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
//! the cold-start timer (`src/bin/rexx-time.rs`) and the interleaved
//! two-interpreter suite (`src/bin/rexx-bench-suite.rs`): resolving
//! `REXX_BENCH_BINARY` into a runnable `Interpreter`, locating
//! `bench-programs/`, and the subprocess timing core in [`timing`].

pub mod timing;

use rexx_oracle::Interpreter;
use std::path::{Path, PathBuf};

/// Names the interpreter binary under test. Read by both the criterion
/// harness and, indirectly, by whatever invokes `rexx-time` on it.
pub const BINARY_VAR: &str = "REXX_BENCH_BINARY";

/// The programs the criterion harness (`benches/interpreter.rs`) benchmarks,
/// in report order. `startup` is listed first because it is the one program
/// this suite is not sizing for 0.5-2s -- see `bench-programs/startup.rex`.
///
/// Not every program in `bench-programs/` is here; see [`NOT_BENCHMARKED`] for
/// the ones deliberately left out and why. The two together are asserted
/// against the directory by `the_benchmark_list_accounts_for_every_program`,
/// so a program added later is either benchmarked or exempted on purpose
/// rather than silently uncovered.
pub static PROGRAMS: &[&str] = &[
    "startup",
    "dispatch",
    "varlookup",
    "compound",
    "strings",
    "arith",
    "alloc",
];

/// Programs in `bench-programs/` that the criterion harness deliberately does
/// not benchmark.
///
/// `heapshape` reports two timings of its own on standard output -- a build
/// time and a forced-collection pause -- and only the second is comparable
/// with anything. Wrapping it in a harness that times the whole process would
/// measure the sum, which is the number its own header says not to use.
/// `d1-decision.md` runs it directly for that reason.
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Every program in `bench-programs/` is either benchmarked or exempted.
    ///
    /// The criterion harness iterates [`PROGRAMS`] and nothing else looks at
    /// the directory, so a program added there and not added here is a
    /// dimension the harness silently stops covering while every number it
    /// prints stays correct. Asserted rather than described, because the
    /// subject is two lists in this repository and either can change without
    /// a sentence about them being reread.
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
}
