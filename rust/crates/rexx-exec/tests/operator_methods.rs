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

//! `corpus/lang/operator_methods.rex` against the oracle, on both engines.
//!
//! **A binary of its own rather than a line in a phase subset file.** A
//! committed `corpus/phase-5d.txt` would oblige `CLOSED_PHASES` to name 5d
//! (`gate_table_c.rs`'s `every_closed_phase_this_table_owns_rows_for_is_gated`),
//! which cannot happen until the phase closes -- so the program lives in
//! `corpus/lang/` like any other, `corpus/unfiled.txt` records that the
//! differential does not run it, and this does. Whoever closes 5d moves the
//! line into the subset file and deletes this file.
//!
//! Three descriptors compared separately and raw. The program writes no
//! trace, so there is no normalisation to apply and none is applied: this is
//! the stricter claim.

mod support;

use std::fs;
use std::path::PathBuf;

use rexx_exec::{Engine, Invocation, Outcome, run_program};
use support::oracle::locate;

/// The program, which is the witness rather than anything in this file.
const PROGRAM: &str = "corpus/lang/operator_methods.rex";

fn program_path() -> PathBuf {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(PROGRAM);
    fs::canonicalize(&path).unwrap_or_else(|e| panic!("cannot resolve {}: {e}", path.display()))
}

fn run_crate(path: &std::path::Path, engine: Engine) -> Outcome {
    let text = fs::read(path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    let path_str = path
        .to_str()
        .unwrap_or_else(|| panic!("{} is not valid UTF-8", path.display()));
    run_program(path_str, text, Invocation::none().with_engine(engine))
}

/// The program answers the oracle byte for byte on stdout, stderr and exit
/// status, on both engines.
///
/// **At BASE `71153941a` it exits 120 on its fourth line**, `the operator `+`
/// applied to an instance of a user class is not implemented (Phase 5)`, so
/// every row below it was unreachable. Two of those rows are the ones that
/// matter most: `p || 'x'` answered the instance's default name at rc 0 where
/// the oracle sends `||` to it, and a class defining `::METHOD string` or
/// `makeString` was ignored -- a silent wrong answer, not a refusal.
#[test]
fn the_operator_methods_program_answers_the_oracle() {
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
    // above passing: a program the oracle answers with nothing at all would
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
}
