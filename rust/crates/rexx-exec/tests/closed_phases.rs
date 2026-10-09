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
//! first is enumerable. A literal is read whole, so an owner inside a longer
//! refusal text (`"... is not implemented (Phase 8)"`) is found too. The
//! owner tables in `tests/` are scanned the same way, and the exclusions
//! file's OWNER rows are held to the same rule.

use std::fs;
use std::path::{Path, PathBuf};

/// The phases whose work is finished and which this assertion covers.
const CLOSED: &[&str] = &["Phase 5", "Phase 6", "Phase 7", "Phase 8"];

/// This crate's test files holding owner tables: the instruction and
/// expression tags, the assertion harnesses' exempt rows and the phase
/// vocabularies their attributions are checked against.
const OWNER_TABLES: &[&str] = &[
    "owners.rs",
    "assertions.rs",
    "bif_assertions.rs",
    "keyword_assertions.rs",
];

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
    match text.rfind("/src/").or_else(|| text.rfind("/tests/")) {
        Some(at) => text[at + 1..].to_string(),
        None => text,
    }
}

/// Every string literal in `text`, as its starting line and its contents,
/// comments skipped: a comment naming a phase is prose about the code, and
/// the rules elsewhere govern that.
fn literals(text: &str) -> Vec<(usize, String)> {
    let chars: Vec<char> = text.chars().collect();
    let mut found = Vec::new();
    let (mut at, mut line) = (0, 1);
    while at < chars.len() {
        match chars[at] {
            '\n' => line += 1,
            '/' if chars.get(at + 1) == Some(&'/') => {
                while at < chars.len() && chars[at] != '\n' {
                    at += 1;
                }
                continue;
            }
            '/' if chars.get(at + 1) == Some(&'*') => {
                at += 2;
                while at < chars.len() && !(chars[at] == '*' && chars.get(at + 1) == Some(&'/')) {
                    line += usize::from(chars[at] == '\n');
                    at += 1;
                }
                at += 2;
                continue;
            }
            '\'' => {
                // A character literal, which may be a quote; otherwise a lifetime.
                if chars.get(at + 1) == Some(&'\\') {
                    at += 3;
                    while at < chars.len() && chars[at] != '\'' {
                        at += 1;
                    }
                } else if chars.get(at + 2) == Some(&'\'') {
                    at += 2;
                }
            }
            'r' if matches!(chars.get(at + 1), Some('"' | '#')) && starts_word(&chars, at) => {
                let hashes = chars[at + 1..].iter().take_while(|c| **c == '#').count();
                if chars.get(at + 1 + hashes) == Some(&'"') {
                    let start = line;
                    let mut body = String::new();
                    at += hashes + 2;
                    while at < chars.len()
                        && !(chars[at] == '"'
                            && chars[at + 1..]
                                .iter()
                                .take(hashes)
                                .filter(|c| **c == '#')
                                .count()
                                == hashes)
                    {
                        line += usize::from(chars[at] == '\n');
                        body.push(chars[at]);
                        at += 1;
                    }
                    found.push((start, body));
                    at += hashes + 1;
                    continue;
                }
            }
            '"' => {
                let start = line;
                let mut body = String::new();
                at += 1;
                while at < chars.len() && chars[at] != '"' {
                    if chars[at] == '\\' {
                        if chars.get(at + 1) == Some(&'\n') {
                            // A continued literal drops the line break and the
                            // next line's indentation.
                            at += 2;
                            line += 1;
                            while at < chars.len() && chars[at].is_whitespace() {
                                line += usize::from(chars[at] == '\n');
                                at += 1;
                            }
                            continue;
                        }
                        body.push(chars[at]);
                        at += 1;
                    }
                    if at < chars.len() {
                        line += usize::from(chars[at] == '\n');
                        body.push(chars[at]);
                        at += 1;
                    }
                }
                found.push((start, body));
            }
            _ => {}
        }
        at += 1;
    }
    found
}

/// Whether the `r` at `at` opens a raw literal rather than ending a name;
/// `br"..."` is one too.
fn starts_word(chars: &[char], at: usize) -> bool {
    let ident = |c: char| c.is_alphanumeric() || c == '_';
    match at.checked_sub(1).map(|before| chars[before]) {
        None => true,
        Some('b') => at < 2 || !ident(chars[at - 2]),
        Some(before) => !ident(before),
    }
}

/// Whether `text` names `phase` as a whole word: `Phase 1` is not in
/// `Phase 10`.
fn names(text: &str, phase: &str) -> bool {
    text.match_indices(phase).any(|(at, _)| {
        !text[at + phase.len()..]
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_digit())
    })
}

/// Every string literal naming a closed phase, alone (`"Phase 8"`) or
/// inside a longer text (`"... not implemented (Phase 8)"`), which is what a
/// refusal is built from.
fn mentions(crate_dir: &str) -> Vec<String> {
    mentions_in(source_files(crate_dir))
}

/// [`mentions`] over `paths`.
fn mentions_in(paths: Vec<PathBuf>) -> Vec<String> {
    let mut found = Vec::new();
    for path in paths {
        let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        for (line, literal) in literals(&text) {
            for phase in CLOSED {
                if names(&literal, phase) {
                    found.push(format!("{}:{line}: {phase}", shown(&path)));
                }
            }
        }
    }
    found
}

/// The whole claim, over every crate a refusal can come from.
#[test]
fn no_refusal_names_a_closed_phase() {
    for crate_dir in [
        "rexx-exec",
        "rexx-parse",
        "rexx-core",
        "rexx-inventory",
        "rexx-api",
    ] {
        let named = mentions(crate_dir);
        assert!(
            named.is_empty(),
            "these sites name a phase that has closed, so the refusal they build \
             tells a reader to wait for work that is already done:\n{named:#?}"
        );
    }
}

/// The same claim over the owner tables in this crate's `tests/`, which name
/// the phase a row or a tag waits for.
#[test]
fn no_owner_table_names_a_closed_phase() {
    let tests = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let paths: Vec<PathBuf> = OWNER_TABLES.iter().map(|name| tests.join(name)).collect();
    for path in &paths {
        assert!(path.is_file(), "{} is not a file", path.display());
    }
    let named = mentions_in(paths);
    assert!(
        named.is_empty(),
        "these owner-table entries name a phase that has closed:\n{named:#?}"
    );
}

/// The words a record uses when an owner's work has been done or moved.
const RESOLVED: &[&str] = &["DELIVERED", "CLOSED", "RE-HOMED", "FIXED", "RESOLVED"];

/// Every OWNER sentence of `text` naming a closed phase with no resolution
/// after it, in the rest of its paragraph or in the next one, before the
/// next OWNER: an exclusions record keeps its owner line as history and says
/// beside or below it what became of the work. `OWNER`, or `Owner:` in any
/// case, is an owner and the phase matches in any case; `NO OWNER` is not an
/// owner. A sentence ends at a full stop followed by a space, and a
/// resolution word resolves only where it opens a sentence, a paragraph or
/// the clause after a colon, so a word about something else in the same
/// paragraph (`EXTERNAL was DELIVERED`) resolves nothing.
fn open_owners(text: &str) -> Vec<String> {
    let mut paragraphs: Vec<String> = Vec::new();
    let mut current = Vec::new();
    for line in text.lines().chain([""]) {
        if line.trim().is_empty() {
            if !current.is_empty() {
                paragraphs.push(current.join(" "));
                current.clear();
            }
        } else {
            current.extend(line.split_whitespace());
        }
    }
    let mut found = Vec::new();
    for (position, paragraph) in paragraphs.iter().enumerate() {
        let next = paragraphs.get(position + 1).map_or("", String::as_str);
        let window = format!("{paragraph} {next}");
        let upper = window.to_ascii_uppercase();
        let owners: Vec<usize> = upper
            .match_indices("OWNER")
            .map(|(at, _)| at)
            .filter(|&at| {
                word_at(&upper, at, "OWNER")
                    && (window[at..].starts_with("OWNER") || upper[at + 5..].starts_with(':'))
            })
            .collect();
        let opens = paragraph.len() + 1;
        for (index, &at) in owners.iter().enumerate() {
            if at >= paragraph.len() || upper[..at].ends_with("NO ") {
                continue;
            }
            let end = window[at..]
                .match_indices('.')
                .map(|(stop, _)| at + stop)
                .find(|&stop| {
                    window[stop + 1..]
                        .chars()
                        .next()
                        .is_none_or(char::is_whitespace)
                })
                .unwrap_or(window.len());
            let sentence = &window[at..end];
            let stop = owners
                .get(index + 1)
                .copied()
                .unwrap_or(window.len())
                .max(end);
            let resolved = RESOLVED.iter().any(|marker| {
                window[at..stop]
                    .match_indices(marker)
                    .any(|(offset, _)| resolves(&window, at + offset, marker, opens))
            });
            if resolved {
                continue;
            }
            for phase in CLOSED {
                if names(&upper[at..end], &phase.to_ascii_uppercase()) {
                    found.push(sentence.to_string());
                }
            }
        }
    }
    found
}

/// Whether `word` stands alone at `at` in `text`.
fn word_at(text: &str, at: usize, word: &str) -> bool {
    let joined = |c: char| c.is_ascii_alphanumeric() || c == '-' || c == '_';
    text[at..].starts_with(word)
        && !text[..at].chars().next_back().is_some_and(joined)
        && !text[at + word.len()..].chars().next().is_some_and(joined)
}

/// Whether the resolution `word` at `at` resolves: standing alone, not
/// quoted in backticks, not after `not` or `not yet`, and opening a sentence,
/// the paragraph starting at `opens`, or the clause after a colon.
fn resolves(text: &str, at: usize, word: &str, opens: usize) -> bool {
    let quoted = |c: Option<char>| c == Some('`');
    let mut before = text[..at]
        .split_whitespace()
        .rev()
        .map(str::to_ascii_lowercase);
    let negated = matches!(
        (before.next().as_deref(), before.next().as_deref()),
        (Some("not"), _) | (Some("yet"), Some("not"))
    );
    let leads = at == opens
        || text[..at]
            .trim_end()
            .chars()
            .next_back()
            .is_none_or(|c| c == '.' || c == ':');
    word_at(text, at, word)
        && !quoted(text[..at].chars().next_back())
        && !quoted(text[at + word.len()..].chars().next())
        && !negated
        && leads
}

/// The same claim over the exclusions file, whose rows name their owners.
#[test]
fn no_open_exclusions_row_names_a_closed_phase() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../docs/superpowers/plans/phase-4-exclusions.txt");
    let text = fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
    let named = open_owners(&text);
    assert!(
        named.is_empty(),
        "these rows give a closed phase as their owner and do not say what became of \
         the work:\n{named:#?}"
    );
}

/// The row check reads an owner, and a resolution after it, as a record
/// writes them.
#[test]
fn the_row_check_tells_an_open_owner_from_a_resolved_one() {
    let rows = "A. OWNER: Phase 8, the loader.\n\n  DELIVERED, later.\n\n\
                B. OWNER:\n Phase 8. C. NO OWNER: Phase 8 is closed.\n\n\
                D. OWNERSHIP of Phase 8. E. OWNER: Phase 80.\n\n\
                F. OWNER: Phase 8.\n\nG.\n\nFIXED.\n\n\
                H. OWNER: Phase 8.\n\nnot in `CLOSED_PHASES`.\n";
    assert_eq!(
        open_owners(rows),
        ["OWNER: Phase 8", "OWNER: Phase 8", "OWNER: Phase 8"]
    );
}

/// An owner in any case is an owner, and a resolution word that is quoted
/// or negated resolves nothing.
#[test]
fn the_row_check_reads_owners_in_any_case_and_no_negated_resolution() {
    let rows = "I. OWNER: Phase 8.\n\nsee the `CLOSED` list.\n\n\
                J. OWNER: Phase 8.\n\nNOT DELIVERED.\n\n\
                K. OWNER: Phase 8.\n\nnot yet DELIVERED.\n\n\
                L. Owner: Phase 8.\n\nM. OWNER: phase 8.\n\n\
                N. Owner: phase 8.\n\nDELIVERED.\n\nP. owner: Phase 8.\n";
    assert_eq!(
        open_owners(rows),
        [
            "OWNER: Phase 8",
            "OWNER: Phase 8",
            "OWNER: Phase 8",
            "Owner: Phase 8",
            "OWNER: phase 8",
            "owner: Phase 8"
        ]
    );
}

/// A resolution word resolves the owner only where it opens a sentence, a
/// paragraph or a clause after a colon: one about something else, later in
/// the same paragraph, leaves the owner open.
#[test]
fn the_row_check_reads_only_a_resolution_that_opens_a_clause() {
    let rows = "Q. OWNER: Phase 8 for the rest. EXTERNAL was DELIVERED by Phase 9.\n\n\
                R. OWNER: Phase 8 for the rest. The other half is FIXED.\n\n\
                S. OWNER: Phase 8 for the rest: DELIVERED by Phase 6.1 Task 5.\n\n\
                T. OWNER: Phase 8 for the rest. DELIVERED by Phase 9.\n\n\
                U. OWNER: Phase 8\n\nFIXED by a later task.\n";
    assert_eq!(
        open_owners(rows),
        ["OWNER: Phase 8 for the rest", "OWNER: Phase 8 for the rest"]
    );
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
            if line.contains("\"Phase 10\"") {
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

/// The literal scan reads a phase inside a longer refusal text, across a
/// continued line, and not in a comment, a character or a lifetime.
#[test]
fn the_literal_scan_finds_a_phase_inside_a_longer_text() {
    let source = r##"fn f<'a>(x: &'a str) -> char {
// "not (Phase 8)"
/* "(Phase 8)" */
let q = '"'; let e = '\''; let s = "a \"b\" (Phase 8)";
let t = "x is \
      not implemented (Phase 8)";
let r = r#"raw "q" Phase 8"#; let u = "Phase 80";
}
"##;
    let named: Vec<usize> = literals(source)
        .into_iter()
        .filter(|(_, literal)| names(literal, "Phase 8"))
        .map(|(line, _)| line)
        .collect();
    assert_eq!(named, [4, 5, 7]);
    let found = mentions("rexx-api")
        .into_iter()
        .filter(|site| site.starts_with("src/ffi.rs:"))
        .count();
    assert_eq!(found, 0, "rexx-api's ffi.rs names no closed phase");
    let open = source_files("rexx-api")
        .iter()
        .flat_map(|path| literals(&fs::read_to_string(path).expect("a source")))
        .filter(|(_, literal)| literal != "Phase 9" && names(literal, "Phase 9"))
        .count();
    assert!(
        open >= 1,
        "the scan found no longer text naming the open Phase 9 in rexx-api, where \
         the boundary's thread refusals name it"
    );
}
