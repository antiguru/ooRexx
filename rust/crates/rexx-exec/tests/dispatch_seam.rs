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
//! # The half the compiler enforces
//!
//! `dispatch::seam::Cleared` is a struct with a private field, and
//! `dispatch::NativeMethod`'s signature takes one by value. It is neither
//! `Copy` nor `Clone` and has no other constructor. So **a primitive method
//! cannot be called at all without a value produced inside `mod seam`** --
//! measured, not argued: writing `native_length(interp, Cleared(()), ..)`
//! anywhere else in the crate is `error[E0423]: cannot initialize a tuple
//! struct which contains private fields`.
//!
//! That is the whole of the compiler's contribution, and it bounds *whether*
//! the seam is reachable around, not *how many* producers there are.
//!
//! # The half this test enforces, and how it is evadable
//!
//! Everything below is a lexical scan, and the previous version of this
//! comment claimed more: it said "the compiler refuses every other
//! spelling", which is false. A second producer written **inside** `mod
//! seam` -- the one place the private field permits -- evades a needle
//! search entirely:
//!
//! ```ignore
//! pub(super) fn clear_for(..) -> Cleared { let ok = (); Cleared(ok) }
//! ```
//!
//! `Cleared(ok)` is not the text `Cleared(())`, and `seam::clear_for(` does
//! not contain `seam::clear(`. So the producer count is bounded by reading
//! the module rather than by counting two tokens, which is what
//! [`the_seam_module_holds_one_struct_and_one_function`] does: the module
//! body is extracted by brace matching and its item keywords are counted, so
//! **any** second producer -- a second `fn`, an `impl` block, a `const` of
//! that type -- is a second item and fails.
//!
//! What that still cannot see, stated rather than argued away:
//!
//! * **An item introduced by a macro expansion inside the module**, or a
//!   second `mod seam` in another file. Neither exists; both would pass.
//! * **A path that invokes something other than a primitive method.** The
//!   token guards `NativeMethod` calls. Phase 5a Task 7 enters Rexx method
//!   bodies, and if it enters them without going through `Interp::invoke`,
//!   this test stays green with two dispatch paths in the tree. Nothing here
//!   can enforce it today, because the second kind of method does not exist
//!   yet; this comment is where the requirement is written down.
//! * **A path in another crate.** The scan reads `rexx-exec/src` only.
//!   Nothing outside this crate can call `Interp::invoke` (it is
//!   `pub(crate)`) or build a `Cleared`, so a second path elsewhere would
//!   have to reimplement the object model rather than reuse it -- which R9
//!   already forbids for a different reason, and which this test does not
//!   check.
//! * **A mention inside a comment or a string** counts toward the needle
//!   tallies, so a stray one fails rather than passes -- the safe direction,
//!   and the reason this file builds its needles with `format!` rather than
//!   spelling them.
//!
//! The scan reads `src/` and never this file, so nothing here can satisfy its
//! own assertion.

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

/// The seam module's own body, by brace matching from its `mod` line.
///
/// Read rather than counted from the outside, because the private field puts
/// every possible producer of the token inside these lines and nowhere else.
fn seam_module_body() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/dispatch.rs");
    let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let header = format!("mod {} {{", "seam");
    let start = text
        .find(&header)
        .unwrap_or_else(|| panic!("{} declares no {header}", path.display()))
        + header.len();
    let mut depth = 1usize;
    for (offset, byte) in text[start..].bytes().enumerate() {
        match byte {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return text[start..start + offset].to_string();
                }
            }
            _ => {}
        }
    }
    panic!(
        "{}: the seam module's braces do not balance",
        path.display()
    );
}

/// **The producer bound: the seam module holds one struct and one function,
/// and no other item at all.**
///
/// The private field means every producer of the token must be written here;
/// counting the items here therefore counts the producers, where counting two
/// token spellings does not -- the module doc has the evasion that motivated
/// this.
///
/// Had a second producer been added -- `fn clear_for(..) -> Cleared { let ok
/// = (); Cleared(ok) }`, the exact shape the needle tallies miss -- the `fn`
/// count below is 2 and this fails.
#[test]
fn the_seam_module_holds_one_struct_and_one_function() {
    let body = seam_module_body();
    // Comments and doc comments are stripped first, so a keyword inside the
    // module's own prose is not counted as an item.
    let code: String = body
        .lines()
        .map(str::trim_start)
        .filter(|line| !line.starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    for (keyword, expected) in [
        ("struct ", 1usize),
        ("fn ", 1),
        ("impl ", 0),
        ("const ", 0),
        ("static ", 0),
        ("mod ", 0),
        ("macro_rules!", 0),
        ("trait ", 0),
        ("union ", 0),
        ("enum ", 0),
    ] {
        assert_eq!(
            code.matches(keyword).count(),
            expected,
            "the seam module holds an unexpected number of `{}` items, so the producers of \
             its token are no longer bounded by reading it:\n{code}",
            keyword.trim()
        );
    }
    // Neither derive can be present: a `Copy` or `Clone` token would let one
    // clearance reach two invocations, which is the same defect the item
    // count is here to bound.
    assert!(
        !code.contains("derive"),
        "the seam token derives something; `Copy` or `Clone` on it would let one clearance \
         serve more than one invocation:\n{code}"
    );
}
