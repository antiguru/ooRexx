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

//! Phase 3 gate: every `.rex` file under `samples/` parses to an AST.
//!
//! The oracle side is already settled: every one of these files passes
//! `build/bin/rexxc` (syntax check only, no execution), measured for the gate
//! assessment in `docs/superpowers/plans/phase-3-gate.md`. So the expected
//! answer for every file is "parses", there is no per-file expectation to
//! curate, and any `Err` here is a real divergence from the oracle.
//!
//! Files are read as bytes, never as strings, because several samples are not
//! valid UTF-8 (`samples/windows/rexutils/drives.rex` is ISO-8859, measured
//! with `file`), and a Rexx literal may hold any byte at all.

use std::path::{Path, PathBuf};

use rexx_parse::parse_program;

/// Collects every `*.rex` file under `dir`, recursively, in a stable order.
///
/// Recursion is the point: `samples/*.rex` matches only the 36 top-level
/// files, and the other files live under `samples/api/`, `samples/windows/`
/// and deeper. Sorted so a failure list reads the same way every run.
fn rex_files_under(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for entry in std::fs::read_dir(&d)
            .unwrap_or_else(|e| panic!("cannot read directory {}: {e}", d.display()))
        {
            let path = entry.expect("readable directory entry").path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().and_then(|e| e.to_str()) == Some("rex") {
                files.push(path);
            }
        }
    }
    files.sort();
    files
}

#[test]
fn every_sample_rex_file_parses() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../samples");
    let files = rex_files_under(&dir);

    let mut lines = 0usize;
    let mut failures = Vec::new();
    for path in &files {
        let text = std::fs::read(path).expect("readable sample file");
        match parse_program(text) {
            Ok(p) => lines += p.source.line_count(),
            Err(e) => failures.push(format!("{}: {e:?}", path.display())),
        }
    }

    assert!(
        failures.is_empty(),
        "{} of {} sample files failed to parse:\n{}",
        failures.len(),
        files.len(),
        failures.join("\n")
    );

    // A walk that silently found nothing would pass vacuously, so a floor is
    // asserted on both counts rather than an exact number that rots when a
    // sample is added or removed. Measured at 301 files and 67,519 physical
    // lines when this was written; the line total here is `line_count()` after
    // Ctrl-Z truncation, so it can sit slightly below a `wc -l` figure.
    assert!(
        files.len() >= 250,
        "expected at least 250 sample files, found {}",
        files.len()
    );
    assert!(
        lines >= 60_000,
        "expected at least 60,000 sample lines, found {lines}"
    );
    println!(
        "parsed {} sample files, {} physical lines",
        files.len(),
        lines
    );
}
