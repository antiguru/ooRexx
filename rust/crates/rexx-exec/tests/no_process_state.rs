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

//! Nothing in the interpreter changes the process's environment or its current
//! directory. Both are shared: the test harnesses run interpreters on threads
//! in one process, so a write reaching the process reaches every other run, and
//! `std::env::set_var` is `unsafe` in this edition besides. The interpreter
//! keeps its own (`Interp::env`, `Interp::cwd`) and every consumer reads those.

use std::fs;
use std::path::{Path, PathBuf};

/// The calls no source file here may make. Matched as a **call through the
/// module path** rather than as a bare name: `set_var` is a substring of this
/// interpreter's own `set_variable`, which sets a Rexx variable and has
/// nothing to do with the process -- the first version of this test flagged
/// twenty of those and nothing real.
const FORBIDDEN_CALLS: &[&str] = &["env::set_var(", "env::remove_var(", "env::set_current_dir("];

/// Importing one of them un-qualifies the call and would evade
/// [`FORBIDDEN_CALLS`], so the import is forbidden too.
const FORBIDDEN_IMPORTS: &[&str] = &[
    "use std::env::set_var",
    "use std::env::remove_var",
    "use std::env::set_current_dir",
];

fn crate_src() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// Every `.rs` file under `dir`, recursively.
fn sources(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for entry in fs::read_dir(dir).unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()))
    {
        let path = entry.expect("a readable directory entry").path();
        if path.is_dir() {
            out.extend(sources(&path));
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
    out
}

#[test]
fn the_interpreter_never_writes_the_process_environment_or_directory() {
    let files = sources(&crate_src());
    assert!(!files.is_empty(), "no sources were scanned at all");

    let mut hits = Vec::new();
    for file in &files {
        let text = fs::read_to_string(file).unwrap_or_else(|e| panic!("cannot read {file:?}: {e}"));
        for (number, line) in text.lines().enumerate() {
            // A comment cannot call anything, and the fields' own docs name
            // these functions to say why they are not used.
            if line.trim_start().starts_with("//") {
                continue;
            }
            let named = FORBIDDEN_CALLS
                .iter()
                .chain(FORBIDDEN_IMPORTS)
                .any(|forbidden| line.contains(forbidden));
            if named {
                hits.push(format!(
                    "{}:{}: {}",
                    file.display(),
                    number + 1,
                    line.trim()
                ));
            }
        }
    }
    assert!(
        hits.is_empty(),
        "these lines write process-global state, which every interpreter on \
         every thread shares -- use `Interp::env` and `Interp::cwd`:\n{}",
        hits.join("\n")
    );
}
