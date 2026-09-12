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
//! **D45, site two: `.local`/`.environment` lookup passes through exactly one
//! chokepoint.**

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

    // Two, not one: the read path (`directory_lookup`) and the write path
    // (`set_directory_entry`). D45's sentence says "lookup", and a write is
    // not one -- but a manager that can veto a read of `.local` and not a
    // write to it would be a seam with a hole, so the second call stays and
    // this count admits it. A third is still red.
    let admits = occurrences(&admit);
    assert_eq!(
        admits.len(),
        2,
        "a directory read and a directory write pass through one chokepoint \
         each, and nothing else does; {admit} is called at {admits:?}"
    );
    assert!(
        admits
            .iter()
            .all(|site| site.contains("src/environment.rs")),
        "every chokepoint is in the seam's own module; {admit} is called at {admits:?}"
    );

    // Two for the same reason `admit` is two: the clearance a read takes is
    // spent in the read path and the one a write takes in the write path.
    let reads = occurrences(&directory);
    assert_eq!(
        reads.len(),
        2,
        "a clearance is spent once per chokepoint and nowhere else, or counting \
         calls to the seam stops bounding the reads that reach a directory; \
         {directory} is called at {reads:?}"
    );
    assert!(
        reads.iter().all(|site| site.contains("src/environment.rs")),
        "every clearance is spent in the seam's own module; {directory} is \
         called at {reads:?}"
    );
}

/// The negative control for the scan itself: a token this crate does not
/// contain is found nowhere, and one it plainly does contain is found.
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

/// **The producer bound: the seam module holds exactly the structs and
/// functions it is designed around, and no other item at all.**
#[test]
fn the_seam_module_holds_only_the_items_it_is_designed_around() {
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
        ("fn ", 4),
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
