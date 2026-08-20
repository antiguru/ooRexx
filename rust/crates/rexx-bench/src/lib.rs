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
    "alloc4c",
    "emptyloop",
];

/// Programs in `bench-programs/` that the criterion harness deliberately does
/// not benchmark.
///
/// **Membership here is a claim about the program, and the claim is checked.**
/// A program belongs here when it measures and reports its own timing: it
/// prints a figure the harness cannot produce, and timing the whole process
/// instead would measure the sum of the parts it separates.
/// `the_exemptions_are_true_of_the_programs_they_name` asserts that in both
/// directions, so appending a name here cannot turn a red assertion green
/// unless the program really does time itself.
///
/// `heapshape` reports a build time and a forced-collection pause on standard
/// output, and only the second is comparable with anything; its own header
/// says not to use the sum. `d1-decision.md` runs it directly for that reason.
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
///
/// Separate from [`programs_dir`] on purpose: `rexx-bench-suite` asserts its
/// axis list against that directory in both directions, so a program placed
/// there becomes a dimension of the committed baseline. A control is not a
/// dimension of anything -- see `bench-control/README.md`.
pub fn controls_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../bench-control")
}

/// REXXCPS 2.2 with its loop counts fixed, resolved the same way.
///
/// Outside `bench-programs/` for the reason a control is: the axis list is
/// asserted against that directory in both directions, and this is one
/// program reported on its own terms rather than a dimension of the baseline.
/// It has no `n = <digits>` line for [`programs_dir`]'s consumers to read.
/// `bench-rexxcps/README.md` has the provenance.
pub fn rexxcps_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../bench-rexxcps/rexxcps.rex")
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

    /// A control differs from the axis it controls by a known amount, and by
    /// nothing else.
    ///
    /// Both halves are asserted because either can fail silently. A control
    /// that drifted from its axis anywhere but the loop bound would still look
    /// like a 1% control and would have stopped being one; a bound edited to a
    /// rounder number would change the size of the effect the instrument is
    /// asked to resolve, with nothing in the file to notice. The escalation
    /// rule reads a tight interval as sensitivity only because this pair is
    /// what it says it is.
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
    ///
    /// `TIME('R')`/`TIME('E')` is how a Rexx program reads its own elapsed
    /// clock, and case does not matter in Rexx, so the source is folded before
    /// the search.
    fn reports_its_own_timing(program: &str) -> bool {
        let path = program_path(program);
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
        text.to_ascii_lowercase().contains("time(")
    }

    /// Each exemption is true of the program it names, and no benchmarked
    /// program qualifies for one.
    ///
    /// Without this, [`NOT_BENCHMARKED`] is a place to put a name. Someone
    /// facing a red `the_benchmark_list_accounts_for_every_program` can
    /// satisfy it by appending, and the program is then exempt from the
    /// criterion harness with nothing anywhere claiming it should be -- which
    /// is how this project has twice had a corpus subset shrink in silence.
    /// Reproduced before this test was written: a dummy program dropped into
    /// `bench-programs/` turned that assertion red, and adding its name to
    /// `NOT_BENCHMARKED` turned it green again.
    ///
    /// **"Fails on this crate" would not have worked as the property.**
    /// `dispatch` and `alloc` exit 120 on this crate exactly as `heapshape`
    /// does, and both are benchmarked, so failing here does not distinguish
    /// the two lists at all. A check keyed on it would have passed for
    /// `heapshape` by coincidence, and would have forced the wrong decision at
    /// Phase 5, when `heapshape` starts running and its real reason for
    /// exemption still holds. The criterion harness runs whatever
    /// `REXX_BENCH_BINARY` names, which for the committed baseline is the C++
    /// oracle, where every program in the directory runs.
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
