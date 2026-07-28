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

//! Shared plumbing for the Task 0.7 benchmark harness (`benches/interpreter.rs`)
//! and the cold-start timer (`src/bin/rexx-time.rs`): resolving
//! `REXX_BENCH_BINARY` into a runnable `Interpreter` and locating
//! `bench-programs/`.

use rexx_oracle::Interpreter;
use std::path::{Path, PathBuf};

/// Names the interpreter binary under test. Read by both the criterion
/// harness and, indirectly, by whatever invokes `rexx-time` on it.
pub const BINARY_VAR: &str = "REXX_BENCH_BINARY";

/// One `.rex` file per D9 dimension, in report order. `startup` is listed
/// first because it is the one program this suite is not sizing for
/// 0.5-2s -- see `bench-programs/startup.rex`.
pub static PROGRAMS: &[&str] = &[
    "startup",
    "dispatch",
    "varlookup",
    "compound",
    "strings",
    "arith",
    "alloc",
];

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
