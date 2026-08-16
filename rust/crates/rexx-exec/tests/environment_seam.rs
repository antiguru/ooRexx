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
//!
//! **D45, site two: `.local`/`.environment` lookup passes through exactly one
//! chokepoint.**
//!
//! Site one is dispatch's, in `tests/dispatch_seam.rs`. This file is built to
//! that file's standard, including its honesty about what a lexical scan can
//! and cannot bound, and its needles are spelled differently on purpose so
//! that neither file's counts move when the other's module changes.
//!
//! # The half the compiler enforces, which is more than site one's
//!
//! `environment::env_seam` holds two structs whose fields are private to it.
//! `Directories` carries the two directory handles and `Admitted` is the
//! clearance; `Admitted` is neither `Copy` nor `Clone` and has no constructor
//! outside the module, and `env_seam::directory` -- the only expression that
//! yields a directory handle -- takes one **by value**.
//!
//! So the reachability claim here is not "a lookup ought to pass the seam" but
//! **"nothing outside the module can name a directory at all"**: reading
//! `held.environment` anywhere else is `error[E0616]`, and calling `directory`
//! without a clearance has nothing to pass. One trip through `admit` yields
//! exactly one handle, because the token is moved.
//!
//! That is the whole of the compiler's contribution. It bounds *whether* the
//! seam can be gone around, not *how many* producers of the token the module
//! holds.
//!
//! # The half this test enforces, and what would pass it anyway
//!
//! Everything below is a lexical scan of `crates/rexx-exec/src`. It counts
//! three things:
//!
//! * one call to `env_seam::admit(`, so there is one chokepoint and not two;
//! * one call to `env_seam::directory(`, so a clearance is spent in one place;
//! * the items inside `mod env_seam`, so a second producer of the token is a
//!   second item and fails, whatever it is called -- which needle counting
//!   alone would miss, since `fn admit_also(..) -> Admitted { let ok = ();
//!   Admitted(ok) }` matches neither spelling.
//!
//! **What would pass every count here while a second lookup path exists**, and
//! this list is the honest answer rather than an argument that there is none:
//!
//! * **A reader that goes to the object rather than to the directory.** The
//!   token guards the two *handles*; once a handle is in hand the entries are
//!   read off `rexx_core::NativeObject`, whose accessors are public to the
//!   whole workspace. A function that took `.environment`'s handle as a
//!   parameter -- from `dot_variable`, say -- and read entries out of it would
//!   be a second lookup path that spends no clearance of its own. Nothing here
//!   sees that, and nothing structural prevents it; what bounds it today is
//!   that the handle has exactly one source.
//! * **An `admit` that hands out more than one clearance per call**, returning
//!   a pair or a `Vec<Admitted>`. Every count below is satisfied while two
//!   lookups are fed from one trip through the seam. This is site one's own
//!   last can't-see item, unchanged: not being `Copy` does not stop the
//!   producer building as many as it likes.
//! * **A macro-produced item inside the module, or a second `mod env_seam`
//!   elsewhere.** Neither exists; both would pass. The brace-matching read
//!   below finds the first `mod env_seam {` in `environment.rs` and no other
//!   file is read for it at all.
//! * **A `.NAME` resolved somewhere other than `Interp::dot_variable`.** Both
//!   callers -- `eval.rs`'s `ExprKind::DotVariable` arm and
//!   `builtin::datatype::literal_value` -- go through that one function today,
//!   and nothing here checks that a third one would. `Interp::dot_variable`
//!   is `pub(crate)`, so a third caller inside this crate is legal; a fourth
//!   outside it is not.
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

/// D45's site-two claim: one chokepoint, and one place a clearance is spent.
#[test]
fn directory_lookup_passes_through_exactly_one_chokepoint() {
    // Built rather than written literally, so this file's own text does not
    // contribute to the counts it takes.
    let module = format!("{}_{}", "env", "seam");
    let admit = format!("{module}::{}(", "admit");
    let directory = format!("{module}::{}(", "directory");

    let admits = occurrences(&admit);
    assert_eq!(
        admits.len(),
        1,
        "a directory lookup must pass through exactly one chokepoint; {admit} \
         is called at {admits:?}"
    );

    let reads = occurrences(&directory);
    assert_eq!(
        reads.len(),
        1,
        "a clearance must be spent in exactly one place, or counting calls to \
         the seam stops bounding the reads that reach a directory; {directory} \
         is called at {reads:?}"
    );
}

/// The negative control for the scan itself: a token this crate does not
/// contain is found nowhere, and one it plainly does contain is found.
///
/// Without this, a `source_files` that silently read the wrong directory, or
/// an `occurrences` that never matched anything, would make the assertions
/// above pass by finding nothing at all -- and "exactly one" would then be the
/// one thing it could not report.
#[test]
fn the_scan_can_tell_a_present_token_from_an_absent_one() {
    assert!(
        occurrences("dot_variable(").len() >= 2,
        "the scan found fewer than two mentions of the function every `.NAME` \
         resolves through, so it is not reading the crate's sources"
    );
    assert!(
        occurrences("no_such_token_exists_in_this_crate").is_empty(),
        "the scan matched a token that is not in the crate, so its counts mean nothing"
    );
}

/// The seam module's own body, by brace matching from its `mod` line.
///
/// Read rather than counted from the outside, because the private fields put
/// every possible producer of the token, and every possible reader of a
/// directory handle, inside these lines and nowhere else.
fn seam_module_body() -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/environment.rs");
    let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let header = format!("mod {}_{} {{", "env", "seam");
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

/// **The producer bound: the seam module holds the two structs and the three
/// functions it is designed around, and no other item at all.**
///
/// The private fields mean every producer of the clearance and every reader of
/// a directory handle must be written here, so counting the items here counts
/// them -- where counting two token spellings does not, and the module doc has
/// the evasion that motivates this.
///
/// Had a second producer been added -- `fn admit_also(..) -> Admitted { let ok
/// = (); Admitted(ok) }`, the exact shape the needle tallies miss -- the `fn`
/// count below is 4 and this fails.
#[test]
fn the_seam_module_holds_two_structs_and_three_functions() {
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
        ("struct ", 2usize),
        ("fn ", 3),
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
             its token and the readers of its handles are no longer bounded by reading \
             it:\n{code}",
            keyword.trim()
        );
    }
    // Neither derive can be present: a `Copy` or `Clone` clearance would let
    // one trip through the seam serve two lookups, which is the same defect
    // the item count is here to bound.
    assert!(
        !code.contains("derive"),
        "an item in the seam module derives something; `Copy` or `Clone` on the clearance \
         would let one of them serve more than one lookup:\n{code}"
    );
}
