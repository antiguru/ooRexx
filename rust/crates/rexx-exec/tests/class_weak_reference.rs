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

//! Phase 5j Task 1: a `WeakReference` to a live class answers the class.

use std::path::PathBuf;

fn corpus_program(name: &str) -> Vec<u8> {
    let path: PathBuf = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../corpus/lang")
        .join(name);
    std::fs::read(&path).unwrap_or_else(|error| panic!("reading {}: {error}", path.display()))
}

/// A `WeakReference` to a class reads the class while the class is live, for
/// a class made at run time and for one a `::CLASS` directive installed.
#[test]
fn a_weak_reference_to_a_live_class_answers_the_class() {
    let program = corpus_program("weakref_class.rex");
    let expected = concat!(
        "live class: TEMPC\n",
        "live object: an Object\n",
        "declared: DECL\n",
    );
    let outcome = rexx_exec::run_program(
        "corpus/lang/weakref_class.rex",
        program.clone(),
        rexx_exec::Invocation::none(),
    );
    assert_eq!(String::from_utf8_lossy(&outcome.stdout), expected);
    assert_eq!(String::from_utf8_lossy(&outcome.stderr), "");
    assert_eq!(outcome.exit_code, 0);
}
