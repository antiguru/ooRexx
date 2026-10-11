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

//! Reading a crate's sources as string literals, for the scans that hold
//! what a refusal may name.

use std::fs;
use std::path::{Path, PathBuf};

/// Every `.rs` file under one crate's `src/`, recursively.
pub fn source_files(crate_dir: &str) -> Vec<PathBuf> {
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
pub fn shown(path: &Path) -> String {
    let text = path.display().to_string();
    match text.rfind("/src/").or_else(|| text.rfind("/tests/")) {
        Some(at) => text[at + 1..].to_string(),
        None => text,
    }
}

/// Every string literal in `text`, as its starting line and its contents,
/// comments skipped: a comment naming a phase is prose about the code, and
/// the rules elsewhere govern that.
pub fn literals(text: &str) -> Vec<(usize, String)> {
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
pub fn names(text: &str, phase: &str) -> bool {
    text.match_indices(phase).any(|(at, _)| {
        !text[at + phase.len()..]
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_digit())
    })
}
