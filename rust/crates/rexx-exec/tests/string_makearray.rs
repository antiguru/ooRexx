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

//! `corpus/lang/string_makearray.rex` against the oracle, on both engines.
//!
//! **A binary of its own rather than a line in a phase subset file**, for the
//! reason `variable_reference.rs` gives: `corpus/phase-5c.txt` would make
//! every one of table C's 5c rows an exit status, which 5c cannot pay while
//! its rows are still landing. Whoever writes that file moves this program's
//! line into it and deletes this binary.
//!
//! Three descriptors compared separately and raw. The program writes no
//! trace, so no normalisation is applied.

mod support;

use std::fs;
use std::path::PathBuf;

use rexx_exec::{Engine, Invocation, Outcome, run_program};
use support::oracle::locate;

/// The program, which is the witness rather than anything in this file.
const PROGRAM: &str = "corpus/lang/string_makearray.rex";

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
/// **Unconditional, not gated.** A constructed stem that rendered as `a Stem`
/// instead of as its own default value did so at rc 0 with empty stderr,
/// and table C's own probe is a `hasMethod` readback that such a stem still
/// satisfies -- so nothing in any harness went red for it. This is the
/// harness that does.
#[test]
fn the_string_makearray_program_answers_the_oracle() {
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
