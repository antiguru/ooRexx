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

//! **No refusal names a phase that has closed.**
//!
//! A loud refusal ends with the phase that owes the work, and a program can
//! read it. Naming a closed phase tells a reader to wait for something that
//! has already happened, so a phase's close-out has to move every refusal it
//! leaves behind to whoever actually owes it -- and the ones it does not move
//! are the ones nobody looked at.
//!
//! The scan is over the crate's own sources rather than over a table,
//! because an owner reaches a refusal three ways: as a row in a table, as a
//! literal argument to a `Loud` constructor, and as a match arm. Only the
//! first is enumerable.

use std::fs;
use std::path::{Path, PathBuf};

/// The phases whose work is finished and which this assertion covers.
///
/// **`Phase 5` is deliberately absent, and its absence is a debt rather than
/// a licence.** That phase closed leaving refusals that name it -- a send to
/// an unimplemented `Directory` method is one -- and re-homing them was never
/// Phase 7's task. Adding it here would redden the tree for work this phase
/// did not take on; the honest statement is the narrow one, and the phase
/// that pays that debt widens this list.
const CLOSED: &[&str] = &["Phase 7"];

/// Every `.rs` file under one crate's `src/`, recursively.
fn source_files(crate_dir: &str) -> Vec<PathBuf> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        let entries = fs::read_dir(dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display()));
        for entry in entries {
            let path = entry.expect("a readable directory entry").path();
            if path.is_dir() {
                walk(&path, out);
            } else if path.extension().is_some_and(|e| e == "rs") {
                out.push(path);
            }
        }
    }
    let src = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join(crate_dir)
        .join("src");
    let mut out = Vec::new();
    walk(&src, &mut out);
    out.sort();
    assert!(
        !out.is_empty(),
        "the scan found no sources under {}, so its result would be a vacuous \
         zero rather than a measurement",
        src.display()
    );
    out
}

/// The path as it reads in a failure: the crate-relative tail.
fn shown(path: &Path) -> String {
    let text = path.display().to_string();
    match text.rfind("/src/") {
        Some(at) => text[at + 1..].to_string(),
        None => text,
    }
}

/// Every mention of a closed phase inside a string literal, which is what a
/// refusal is built from. Comments are skipped: a comment naming a phase is
/// prose about the code, and the rules elsewhere govern that.
fn mentions(crate_dir: &str) -> Vec<String> {
    let mut found = Vec::new();
    for path in source_files(crate_dir) {
        let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        for (index, line) in text.lines().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with("//") {
                continue;
            }
            for phase in CLOSED {
                let quoted = format!("\"{phase}\"");
                if line.contains(&quoted) {
                    found.push(format!("{}:{}: {phase}", shown(&path), index + 1));
                }
            }
        }
    }
    found
}

/// The whole claim, over every crate a refusal can come from.
#[test]
fn no_refusal_names_a_closed_phase() {
    for crate_dir in ["rexx-exec", "rexx-parse", "rexx-core", "rexx-inventory"] {
        let named = mentions(crate_dir);
        assert!(
            named.is_empty(),
            "these sites name a phase that has closed, so the refusal they build \
             tells a reader to wait for work that is already done:\n{named:#?}"
        );
    }
}

/// The negative control for the scan: a phase that is still open is found by
/// the same walk, so an empty result above means the sites are gone and not
/// that the scan is looking in the wrong place.
#[test]
fn the_scan_finds_an_open_phase_it_is_not_asked_about() {
    let mut open = 0usize;
    for path in source_files("rexx-exec") {
        let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        for line in text.lines() {
            if line.trim_start().starts_with("//") {
                continue;
            }
            if line.contains("\"Phase 8\"") || line.contains("\"Phase 10\"") {
                open += 1;
            }
        }
    }
    assert!(
        open >= 2,
        "the scan found {open} refusals naming a phase that is still open, so it \
         is not reading the sources the other test clears"
    );
}
