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

//! Refusal text that names a representation instead of a language rule
//! (spec `docs/superpowers/specs/2026-10-14-phase-9b1-uniform-object-model.md`,
//! section 6). Every string literal in `crates/*/src` containing one of
//! [`PHRASES`] must be on [`ALLOWED`], and every row of [`ALLOWED`] must still
//! match a literal. Test code (`tests.rs` and `tests/` under `src`) and
//! comments are not refusal text.
//!
//! `ALLOWED` was filled from this command, run from `rust/`, keeping the lines
//! whose match is inside a string literal:
//!
//! ```text
//! grep -rnE --include=*.rs \
//!     'did not build|does not keep|is no longer live|is value-based|this crate did not|this crate does not|this crate built' \
//!     crates/*/src | grep -vE '/tests(\.rs:|/)'
//! ```

use std::path::{Path, PathBuf};

/// The phrases that mark a refusal as one about this crate's representation,
/// plus the value-based rule's own refusal, which is allowed by Deviation 36.
const PHRASES: &[&str] = &[
    "did not build",
    "does not keep",
    "is no longer live",
    "is value-based",
    "this crate did not",
    "this crate does not",
    "this crate built",
];

/// `(file:substring, owner)`: a literal in `file` (relative to `crates/`)
/// containing `substring` is allowed. The owner is a permanent Deviation, the
/// phase that owes the construct, or the task of
/// `docs/superpowers/plans/2026-10-14-phase-9b1.md` that removes it.
const ALLOWED: &[(&str, &str)] = &[
    (
        "rexx-exec/src/lib.rs:a metaclass NEW answering a class Class~new did not build",
        "Deviation 32",
    ),
    (
        "rexx-exec/src/lib.rs:OBJECTNAME= on a String is refused because a String is value-based",
        "Deviation 36",
    ),
    (
        "rexx-exec/src/lib.rs:a directory whose entries this crate does not fill",
        "Phase 10",
    ),
    (
        "rexx-exec/src/environment/identities.rs:a .ROUTINES entry that is not a routine this crate built",
        "Deviation 28",
    ),
    // No leak in `.superpowers/sdd/2026-10-14-phase-9b1/scout-leaks.md` names
    // these guards on a freed handle or an unexpected body, so the sweep owns them.
    (
        "rexx-exec/src/dispatch/context.rs:a stack frame this crate did not build",
        "Task 8",
    ),
    (
        "rexx-exec/src/dispatch/object_protocol.rs:a stack frame this crate did not build",
        "Task 8",
    ),
    (
        "rexx-exec/src/dispatch/construct.rs:an object this crate did not build",
        "Task 8",
    ),
    (
        "rexx-exec/src/dispatch/construct.rs:a value whose object is no longer live",
        "Task 8",
    ),
    (
        "rexx-exec/src/dispatch/object_protocol.rs:a value whose object is no longer live",
        "Task 8",
    ),
    (
        "rexx-exec/src/dispatch.rs:a value whose object is no longer live",
        "Task 8",
    ),
    (
        "rexx-exec/src/environment.rs:a value whose object is no longer live",
        "Task 8",
    ),
    // An internal guard: a package asked for after it was collected is a
    // rooting defect, refused rather than rebuilt.
    (
        "rexx-exec/src/environment/identities.rs:a package whose object is no longer live",
        "Task 8",
    ),
];

/// The owners a row may name.
const OWNERS: &[&str] = &[
    "Deviation 28",
    "Deviation 32",
    "Deviation 36",
    "Task 8",
    "Phase 10",
];

fn crates_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the crate sits in `crates/`")
        .to_path_buf()
}

/// Every non-test `.rs` file under `dir`.
fn sources(dir: &Path, out: &mut Vec<PathBuf>) {
    let mut entries: Vec<PathBuf> = std::fs::read_dir(dir)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()))
        .map(|entry| entry.expect("a directory entry").path())
        .collect();
    entries.sort();
    for path in entries {
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
        if path.is_dir() {
            if name != "tests" {
                sources(&path, out);
            }
        } else if name.ends_with(".rs") && name != "tests.rs" {
            out.push(path);
        }
    }
}

/// The contents of every string literal in `text`, with a `\` line
/// continuation and the indentation after it removed. Comments, character
/// literals and raw strings are recognised so a `"` inside one does not open a
/// literal.
fn string_literals(text: &str) -> Vec<String> {
    let bytes = text.as_bytes();
    let mut literals = Vec::new();
    let mut at = 0;
    while at < bytes.len() {
        match bytes[at] {
            b'/' if bytes.get(at + 1) == Some(&b'/') => {
                while at < bytes.len() && bytes[at] != b'\n' {
                    at += 1;
                }
            }
            b'/' if bytes.get(at + 1) == Some(&b'*') => {
                at += 2;
                while at + 1 < bytes.len() && !(bytes[at] == b'*' && bytes[at + 1] == b'/') {
                    at += 1;
                }
                at += 2;
            }
            b'\'' if bytes.get(at + 1) == Some(&b'\\') => {
                at += 2;
                while at < bytes.len() && bytes[at] != b'\'' {
                    at += 1;
                }
                at += 1;
            }
            b'\'' if bytes.get(at + 2) == Some(&b'\'') => at += 3,
            b'r' if matches!(bytes.get(at + 1), Some(b'"' | b'#'))
                && (at == 0
                    || !(bytes[at - 1].is_ascii_alphanumeric() || bytes[at - 1] == b'_')) =>
            {
                let hashes = bytes[at + 1..].iter().take_while(|&&b| b == b'#').count();
                let open = at + 1 + hashes;
                if bytes.get(open) != Some(&b'"') {
                    at += 1;
                    continue;
                }
                let close = format!("\"{}", "#".repeat(hashes));
                let end = text[open + 1..]
                    .find(&close)
                    .map_or(bytes.len(), |end| open + 1 + end);
                literals.push(text[open + 1..end].to_string());
                at = end + close.len();
            }
            b'"' => {
                let mut literal = Vec::new();
                at += 1;
                while at < bytes.len() && bytes[at] != b'"' {
                    if bytes[at] == b'\\' && bytes.get(at + 1) == Some(&b'\n') {
                        at += 2;
                        while at < bytes.len() && bytes[at].is_ascii_whitespace() {
                            at += 1;
                        }
                        continue;
                    }
                    let step = if bytes[at] == b'\\' { 2 } else { 1 };
                    literal.extend_from_slice(&bytes[at..(at + step).min(bytes.len())]);
                    at += step;
                }
                literals.push(String::from_utf8_lossy(&literal).into_owned());
                at += 1;
            }
            _ => at += 1,
        }
    }
    literals
}

/// `(file relative to crates/, literal)` for every literal containing a phrase.
fn matches() -> Vec<(String, String)> {
    let crates = crates_dir();
    let mut files = Vec::new();
    let mut crate_dirs: Vec<PathBuf> = std::fs::read_dir(&crates)
        .expect("crates/ is readable")
        .map(|entry| entry.expect("a directory entry").path().join("src"))
        .filter(|src| src.is_dir())
        .collect();
    crate_dirs.sort();
    for src in crate_dirs {
        sources(&src, &mut files);
    }
    let mut found = Vec::new();
    for file in files {
        let text = std::fs::read_to_string(&file).expect("a source file");
        let relative = file
            .strip_prefix(&crates)
            .expect("under crates/")
            .to_string_lossy()
            .into_owned();
        for literal in string_literals(&text) {
            if PHRASES.iter().any(|phrase| literal.contains(phrase)) {
                found.push((relative.clone(), literal));
            }
        }
    }
    found
}

fn allows(row: &str, file: &str, literal: &str) -> bool {
    let (row_file, substring) = row.split_once(':').expect("a `file:substring` row");
    row_file == file && literal.contains(substring)
}

#[test]
fn every_representation_refusal_is_on_the_allow_list() {
    let found = matches();
    let unlisted: Vec<String> = found
        .iter()
        .filter(|(file, literal)| !ALLOWED.iter().any(|(row, _)| allows(row, file, literal)))
        .map(|(file, literal)| format!("  {file}: {literal:?}"))
        .collect();
    assert!(
        unlisted.is_empty(),
        "refusal text naming a representation, not on ALLOWED:\n{}",
        unlisted.join("\n")
    );
    let stale: Vec<&str> = ALLOWED
        .iter()
        .filter(|(row, _)| {
            !found
                .iter()
                .any(|(file, literal)| allows(row, file, literal))
        })
        .map(|(row, _)| *row)
        .collect();
    assert!(
        stale.is_empty(),
        "ALLOWED rows that match no literal any more, to be deleted: {stale:#?}"
    );
}

#[test]
fn every_row_names_an_owner() {
    for (row, owner) in ALLOWED {
        assert!(OWNERS.contains(owner), "{row}: owner {owner:?}");
    }
}

#[test]
fn the_scanner_reads_literals_and_skips_comments() {
    let source = "// \"did not build\" in a comment\n\
                  /* \"does not keep\" */\n\
                  let c = '\"'; let d = '\\\\';\n\
                  let a = \"one did not build\";\n\
                  let b = r#\"two \"is no longer live\"#;\n\
                  let e = \"three is \\\n             value-based\";\n";
    assert_eq!(
        string_literals(source),
        [
            "one did not build",
            "two \"is no longer live",
            "three is value-based"
        ]
    );
}
