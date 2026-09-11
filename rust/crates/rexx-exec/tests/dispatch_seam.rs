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
//! ```ignore
//! pub(super) fn clear_for(..) -> Cleared { let ok = (); Cleared(ok) }
//! ```

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

/// Every occurrence of `needle` on a line that is not a whole-line comment,
/// as `(path, line number)` pairs.
fn code_occurrences(needle: &str) -> Vec<String> {
    let mut found = Vec::new();
    for path in source_files() {
        let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        for (index, line) in text.lines().enumerate() {
            if line.trim_start().starts_with("//") {
                continue;
            }
            for _ in line.matches(needle) {
                found.push(format!("{}:{}", path.display(), index + 1));
            }
        }
    }
    found
}

/// Every collection in this crate goes through `Interp::collect_now`, which
/// is where the root set is completed before `Heap::collect` is handed it.
#[test]
fn heap_collect_is_called_from_collect_now_alone() {
    // Built rather than written literally, so this file's own text does not
    // contribute to the count it takes.
    let call = format!("{}.{}(", "heap", "collect");

    // Whitespace-collapsed, so a wrapped call is still one match. Comment
    // lines go first, so that a comment cannot satisfy the assertion.
    let mut sites = Vec::new();
    for path in source_files() {
        let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        let code: String = text
            .lines()
            .filter(|line| !line.trim_start().starts_with("//"))
            .collect::<Vec<_>>()
            .join(" ");
        let collapsed: String = code.split_whitespace().collect::<Vec<_>>().join("");
        for _ in collapsed.matches(&call) {
            sites.push(path.display().to_string());
        }
    }
    assert_eq!(
        sites.len(),
        1,
        "every collection must go through `Interp::collect_now`, which is \
         what completes the root set; {call} appears in {sites:?}"
    );
    assert!(
        sites[0].contains("lib.rs"),
        "the one collection site is outside `lib.rs`, in {:?} -- if \
         `collect_now` moved there too this assertion needs updating, and if \
         it did not, the sweep is being bypassed",
        sites[0]
    );

    // Inside `collect_now` and not merely somewhere in `lib.rs`, which is
    // what makes this stronger than the file check above: the body is read
    // from the signature to the first closing brace at the same indentation.
    let lib = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("lib.rs"),
    )
    .expect("lib.rs is readable");
    let signature = format!("    fn {}_{}(&mut self) {{", "collect", "now");
    let start = lib
        .find(&signature)
        .unwrap_or_else(|| panic!("no function matching {signature:?}"));
    let body = &lib[start..];
    let end = body
        .find("\n    }\n")
        .expect("the function has a closing brace at its own indentation");
    assert!(
        body[..end].contains(&call),
        "the one `Heap::collect` call in this crate is in `lib.rs` but not \
         inside `collect_now`, so it does not get the swept root set"
    );
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

/// **The `PROTECTED` question is asked inside the seam and nowhere else.**
#[test]
fn the_protected_question_is_asked_only_inside_the_seam() {
    let predicate = format!("{}_is_{}(", "method", "protected");
    let sites = code_occurrences(&predicate);
    assert_eq!(
        sites.len(),
        2,
        "the protected predicate must have exactly one caller beside its own \
         definition; {predicate} appears at {sites:?}"
    );
    let elsewhere: Vec<&String> = sites
        .iter()
        .filter(|site| !site.contains("dispatch.rs"))
        .collect();
    assert!(
        elsewhere.is_empty(),
        "the protected predicate is named outside `dispatch.rs`, at {elsewhere:?}, \
         so the seam is no longer the only place the manager is asked"
    );
}

/// The files that may name the seam's token, listed rather than matched by
/// prefix so that adding one is an edit to this line.
const CLEARANCE_CONSUMERS: &[&str] = &[
    "src/dispatch.rs",
    "src/dispatch/string.rs",
    "src/dispatch/collection.rs",
    "src/dispatch/hash.rs",
    "src/dispatch/rexx_info.rs",
    "src/dispatch/executable.rs",
    "src/dispatch/introspection.rs",
    "src/dispatch/context.rs",
    "src/dispatch/package.rs",
];

/// **Every consumer of the seam's token is written in one of
/// [`CLEARANCE_CONSUMERS`]**, which is what lets the item read below bound the
/// producers *and* those files bound the consumers.
#[test]
fn the_seam_token_is_named_only_by_the_dispatch_module() {
    let token = format!("{}{}", "Clea", "red");
    let sites = code_occurrences(&token);
    let elsewhere: Vec<&String> = sites
        .iter()
        .filter(|site| !CLEARANCE_CONSUMERS.iter().any(|file| site.contains(file)))
        .collect();
    assert!(
        elsewhere.is_empty(),
        "the seam's token is named in code outside {CLEARANCE_CONSUMERS:?}, at \
         {elsewhere:?}, so the functions that can consume a clearance are no \
         longer bounded by reading those files"
    );
    assert!(
        sites.len() >= 2,
        "the scan found fewer than two code mentions of the token inside \
         dispatch.rs, so the filter above is passing by finding nothing"
    );
}

/// The negative control for the scan itself: a token this crate does not
/// contain is found nowhere, and one it plainly does contain is found.
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
