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

//! **D45, site one: dispatch passes through exactly one chokepoint.**
//!
//! # What this test counts, and what it cannot see
//!
//! It counts, over every `.rs` file under `crates/rexx-exec/src/`:
//!
//! * occurrences of the token `seam::clear(`, the call to the chokepoint:
//!   exactly one;
//! * occurrences of `Cleared(())`, which is both the token's own tuple-struct
//!   declaration and the only expression that builds one: exactly two, of
//!   which exactly one is the declaration -- so exactly one construction.
//!
//! Counting text is worth something here **only
//! because of a type-system property that stands behind it**:
//! `dispatch::seam::Cleared` is a struct with a private field declared in a
//! module whose whole body is those few lines, and
//! `dispatch::NativeMethod`'s signature takes one. So a second path that
//! invokes a primitive method cannot be written without either calling
//! `seam::clear` (counted) or adding a second `Cleared(())` (counted) --
//! the compiler refuses every other spelling. That is what makes "exactly
//! one" a property rather than an inspection.
//!
//! **What would pass this while a second dispatch path existed**, stated
//! plainly rather than argued away:
//!
//! * **A path that invokes something other than a primitive method.** The
//!   token guards `NativeMethod` calls. Phase 5a Task 7 enters Rexx method
//!   bodies, and if it enters them without going through
//!   `Interp::invoke`, this test stays green with two dispatch paths in the
//!   tree. The remedy is for that task to route its invocation through the
//!   same `Interp::invoke`, and this comment is where the requirement is
//!   written down; nothing here can enforce it today, because the second
//!   kind of method does not exist yet.
//! * **A path in another crate.** The scan reads `rexx-exec/src` only.
//!   Nothing outside this crate can call `Interp::invoke` (it is
//!   `pub(crate)`) or build a `Cleared`, so a second path elsewhere would
//!   have to reimplement the object model rather than reuse it -- which R9
//!   already forbids for a different reason, and which this test does not
//!   check.
//! * **A second call added inside a comment or a string.** That is counted
//!   too, so it fails rather than passes -- the safe direction, and the
//!   reason this file's own text says `seam` and `clear` separately below
//!   rather than spelling the call.
//!
//! The counts are over `src/` and not over this file, so nothing here can
//! satisfy its own assertion.

use std::fs;
use std::path::{Path, PathBuf};

/// Every `.rs` file under `crates/rexx-exec/src/`, recursively.
fn source_files() -> Vec<PathBuf> {
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
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut out = Vec::new();
    walk(&src, &mut out);
    out.sort();
    assert!(
        !out.is_empty(),
        "the scan found no source files at all, so its counts would be a \
         vacuous zero rather than a measurement"
    );
    out
}

/// Every occurrence of `needle` across the crate's own sources, as
/// `(path, line number)` pairs so a failure names where the extra one is.
fn occurrences(needle: &str) -> Vec<String> {
    let mut found = Vec::new();
    for path in source_files() {
        let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        for (index, line) in text.lines().enumerate() {
            for _ in line.matches(needle) {
                found.push(format!("{}:{}", path.display(), index + 1));
            }
        }
    }
    found
}

/// The whole of D45's site-one claim: one call to the seam, and one way to
/// build the token it hands out.
#[test]
fn dispatch_passes_through_exactly_one_chokepoint() {
    // Built rather than written literally, so this file's own text does not
    // contribute to the counts it takes.
    let call = format!("{}::{}(", "seam", "clear");
    let construct = format!("{}(())", "Cleared");

    let calls = occurrences(&call);
    assert_eq!(
        calls.len(),
        1,
        "dispatch must pass through exactly one chokepoint; {call} is called at {calls:?}"
    );

    // The tuple struct's own declaration reads the same as a construction of
    // it, so the two are counted apart: one declaration, and one thing that
    // is not the declaration.
    let declaration = format!("struct {construct}");
    let declarations = occurrences(&declaration);
    assert_eq!(
        declarations.len(),
        1,
        "the seam's token must be declared exactly once; {declaration} appears \
         at {declarations:?}"
    );
    let mentions = occurrences(&construct);
    assert_eq!(
        mentions.len(),
        2,
        "the seam's token must have exactly one construction site beside its \
         one declaration, or counting calls to the seam stops bounding the \
         paths that reach a native method; {construct} appears at {mentions:?}"
    );
}

/// The negative control for the scan itself: a token this crate does not
/// contain is found nowhere, and one it plainly does contain is found.
///
/// Without this, a `source_files` that silently read the wrong directory, or
/// an `occurrences` that never matched anything, would make the assertions
/// above pass by finding nothing at all -- and "exactly one" would then be
/// the one thing it could not report.
#[test]
fn the_scan_can_tell_a_present_token_from_an_absent_one() {
    assert!(
        occurrences("Interp::invoke").len() >= 2,
        "the scan found fewer than two mentions of a name the dispatch module \
         defines and documents, so it is not reading the crate's sources"
    );
    assert!(
        occurrences("no_such_token_exists_in_this_crate").is_empty(),
        "the scan matched a token that is not in the crate, so its counts mean nothing"
    );
}
