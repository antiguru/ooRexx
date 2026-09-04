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

//! `corpus/lang/package_namespace.rex` against the oracle, on both engines.
//!
//! **A binary of its own rather than a line in a phase subset file**, for the
//! reason `tests/package_requires.rs` gives: a committed `corpus/phase-5d.txt`
//! would oblige `CLOSED_PHASES` to name 5d before the phase closes. The
//! program lives in `corpus/lang/` like any other, `corpus/unfiled.txt`
//! records that the differential does not run it, and this does. Whoever
//! closes 5d moves the line into the subset file and deletes this file.
//!
//! **The two files it requires are `.cls`**, per `corpus/README.md`'s section
//! on a program needing a second file beside it, and are named without an
//! extension in the directive so the search's own `.cls` step resolves them.
//!
//! Three descriptors compared separately and raw. The program writes no
//! trace, so there is no normalisation to apply and none is applied.

mod support;

use std::fs;
use std::path::{Path, PathBuf};

use rexx_exec::{Engine, Invocation, Outcome, run_program};
use support::oracle::locate;

/// The program, which is the witness rather than anything in this file.
const PROGRAM: &str = "corpus/lang/package_namespace.rex";

fn program_path() -> PathBuf {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(PROGRAM);
    fs::canonicalize(&path).unwrap_or_else(|e| panic!("cannot resolve {}: {e}", path.display()))
}

fn run_crate(path: &Path, engine: Engine) -> Outcome {
    let text = fs::read(path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    let path_str = path
        .to_str()
        .unwrap_or_else(|| panic!("{} is not valid UTF-8", path.display()));
    run_program(path_str, text, Invocation::none().with_engine(engine))
}

/// The program answers the oracle byte for byte on stdout, stderr and exit
/// status, on both engines.
///
/// **At BASE `d5221c5e3` it exits 120 on its first directive**,
/// `::REQUIRES NAMESPACE is not implemented (Phase 5)`, with stdout empty --
/// so every line below was unreachable, the two required files' prologues
/// included.
///
/// The lines that would move first if a lookup narrowed or widened: the
/// qualified class and routine, the ones reached through the namespace
/// package's own `::REQUIRES`, the four error numbers a miss on either half
/// carries, and the three package-local rows, whose order against `.local` and
/// against the REXX package's classes is the documented search order's steps 4
/// to 6.
#[test]
fn the_package_namespace_program_answers_the_oracle() {
    let path = program_path();
    let oracle = locate();
    let cpp = oracle.run(&path);
    let ir = run_crate(&path, Engine::Ir);
    let tree_walker = run_crate(&path, Engine::TreeWalker);

    for (engine, outcome) in [(Engine::Ir, &ir), (Engine::TreeWalker, &tree_walker)] {
        assert_eq!(
            String::from_utf8_lossy(&outcome.stdout),
            String::from_utf8_lossy(&cpp.stdout),
            "{PROGRAM}: stdout differs from the oracle on {engine:?}"
        );
        assert_eq!(
            String::from_utf8_lossy(&outcome.stderr),
            String::from_utf8_lossy(&cpp.stderr),
            "{PROGRAM}: stderr differs from the oracle on {engine:?}"
        );
        assert_eq!(
            outcome.exit_code,
            cpp.expect_exit_code(),
            "{PROGRAM}: exit status differs from the oracle on {engine:?}"
        );
    }

    // Asserted on the runs rather than inferred from the two comparisons
    // above passing: a program the oracle answered with nothing at all would
    // satisfy both while the engines had drifted apart.
    assert!(
        !cpp.stdout.is_empty(),
        "{PROGRAM}: the oracle answered nothing, so the comparisons above \
         compare two absences"
    );
    assert_eq!(
        (&ir.stdout, &ir.stderr, ir.exit_code),
        (
            &tree_walker.stdout,
            &tree_walker.stderr,
            tree_walker.exit_code
        ),
        "{PROGRAM}: the two crate engines disagree"
    );

    // **Each refusal is named rather than left to the byte comparison.** All
    // four are `raised NN.NNN` lines, so a crate raising the wrong one of them
    // would differ from the oracle by three bytes on one line -- true of the
    // comparison above, and this says which property failed. `98.988` appears
    // three times and is counted rather than searched for, since a crate
    // answering it for every miss would satisfy a substring test.
    let stdout = String::from_utf8_lossy(&ir.stdout);
    for (number, times) in [("98.987", 1), ("98.988", 3), ("43.902", 2)] {
        assert_eq!(
            stdout
                .lines()
                .filter(|line| line.ends_with(&format!("raised {number}")))
                .count(),
            times,
            "{PROGRAM}: {number} was not raised exactly {times} time(s):\n{stdout}"
        );
    }

    // **The prologue of a file required under two qualifiers runs once**,
    // which is the cache answering the second `::REQUIRES` rather than the
    // registration loading a second copy. Asserted on the count for the reason
    // `tests/package_requires.rs` asserts its own: a crate running it twice
    // differs on every line below it too.
    for prologue in ["nslib prologue", "nsdep prologue"] {
        assert_eq!(
            stdout.lines().filter(|line| *line == prologue).count(),
            1,
            "{PROGRAM}: {prologue:?} did not run exactly once:\n{stdout}"
        );
    }
}
