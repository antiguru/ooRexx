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

//! Phase 5j's witnesses for when a class dies and what runs when it does.

use std::path::PathBuf;

fn corpus_program(name: &str) -> Vec<u8> {
    let path: PathBuf = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../corpus/lang")
        .join(name);
    std::fs::read(&path).unwrap_or_else(|error| panic!("reading {}: {error}", path.display()))
}

fn agrees(program: &str, expected: &str) {
    let text = corpus_program(program);
    let outcome = rexx_exec::run_program(program, text.clone(), rexx_exec::Invocation::none());
    assert_eq!(
        String::from_utf8_lossy(&outcome.stdout),
        expected,
        "{program}"
    );
    assert_eq!(String::from_utf8_lossy(&outcome.stderr), "", "{program}");
    assert_eq!(outcome.exit_code, 0, "{program}");
}

/// A class nothing refers to any more is collected, and leaves its
/// superclass's `~subclasses`.
#[test]
fn a_class_nothing_refers_to_is_collected() {
    agrees("class_collected.rex", "created: 1\nafter drop+gc: 0\n");
}

/// A live instance keeps its class, and only the instance does.
#[test]
fn an_instance_keeps_its_class_alive_and_nothing_else_does() {
    agrees(
        "class_pinned_by_instance.rex",
        "instance alive, delta: 1\ninstance class: TEMPC\ninstance gone, delta: 0\n",
    );
}

/// A class-side `UNINIT` runs at the collection that reaches the class, not at
/// termination.
#[test]
fn a_class_side_uninit_runs_at_the_collection_that_reaches_the_class() {
    agrees("class_uninit_gc.rex", "before\nclass uninit\nafter\n");
}
