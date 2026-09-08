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

//! `corpus/refusal-sites.tsv` holds every `Loud` and `Raised` constructor in
//! `src/`, and the ones a message send itself can produce carry a verdict for
//! an instance receiver. This file re-derives the enumeration from the source
//! and holds the committed one equal to it.
//!
//! # Why the enumeration is derived rather than listed
//!
//! The verdicts answer a question a receiver is an input to -- a class object
//! renders as its id and an instance as an article and a class id, and the
//! frame above the error differs the same way. A constructor added after the
//! walk would be a site nobody asked that question of, and prose saying "all
//! of them" cannot see one arrive. Re-deriving is what makes the new site red
//! here instead.
//!
//! # What this cannot see
//!
//! It does not re-run a probe. A verdict of `agrees` is a claim about a
//! measurement taken once, and nothing here would notice the crate's answer
//! moving away from the oracle's afterwards; `tests/corpus.rs` is what sees
//! that, for the rows that reach it. What this sees is a **site** appearing,
//! disappearing or moving files, which is the axis the walk's completeness
//! rests on.
//!
//! # The scanner
//!
//! `#[cfg(test)] mod` blocks are excluded, so a constructor only a test builds
//! is not a site; line comments are stripped, so a constructor named only in
//! prose is not one either. Both matter: `Loud::parse` appears twice in
//! comments and does not exist, and counting it is what made an earlier figure
//! for this surface one too many.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// One committed row.
#[derive(Debug, PartialEq, Eq)]
struct Row {
    kind: String,
    name: String,
    surface: String,
    definition: String,
    verdict: String,
    reached: String,
    answer: String,
    witness: String,
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("the crate sits two levels below `rust/`")
        .to_path_buf()
}

fn table_path() -> PathBuf {
    repo_root().join("corpus/refusal-sites.tsv")
}

fn committed() -> Vec<Row> {
    let text = std::fs::read_to_string(table_path()).expect("the table is checked in");
    text.lines()
        .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
        .map(|line| {
            let mut fields = line.split('\t');
            let mut next = || {
                fields
                    .next()
                    .unwrap_or_else(|| panic!("eight tab-separated fields in {line:?}"))
                    .to_string()
            };
            Row {
                kind: next(),
                name: next(),
                surface: next(),
                definition: next(),
                verdict: next(),
                reached: next(),
                answer: next(),
                witness: next(),
            }
        })
        .collect()
}

/// Every `.rs` file under `src/`, with `#[cfg(test)] mod` blocks dropped and
/// line comments stripped, as `(line number, code)` pairs.
fn code_lines(path: &Path) -> Vec<(usize, String)> {
    let text = std::fs::read_to_string(path).expect("a source file this crate compiles");
    let lines: Vec<&str> = text.lines().collect();
    let mut out = Vec::new();
    let mut at = 0;
    while at < lines.len() {
        if lines[at].starts_with("#[cfg(test)]") {
            let mut opener = at + 1;
            while opener < lines.len() && lines[opener].starts_with("#[") {
                opener += 1;
            }
            if opener < lines.len()
                && lines[opener].starts_with("mod ")
                && lines[opener].trim_end().ends_with('{')
            {
                // rustfmt puts a module's closing brace in column one, so the
                // first bare `}` ends it.
                let mut close = opener + 1;
                while close < lines.len() && lines[close] != "}" {
                    close += 1;
                }
                at = close + 1;
                continue;
            }
        }
        let code = match lines[at].find("//") {
            Some(cut) => lines[at][..cut].to_string(),
            None => lines[at].to_string(),
        };
        out.push((at + 1, code));
        at += 1;
    }
    out
}

fn sources() -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut pending = vec![Path::new(env!("CARGO_MANIFEST_DIR")).join("src")];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("a directory this crate compiles") {
            let path = entry.expect("a readable directory entry").path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|kind| kind == "rs") {
                found.push(path);
            }
        }
    }
    found.sort();
    found
}

fn is_name_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

/// The identifier starting at `from`, if one does.
fn name_at(code: &str, from: usize) -> Option<&str> {
    let bytes = code.as_bytes();
    if !bytes
        .get(from)
        .is_some_and(|byte| *byte == b'_' || byte.is_ascii_lowercase())
    {
        return None;
    }
    let end = bytes[from..]
        .iter()
        .position(|byte| !is_name_byte(*byte))
        .map_or(code.len(), |at| from + at);
    Some(&code[from..end])
}

/// `Loud`/`Raised` constructors, keyed by name, as `(kind, definition site)`.
///
/// A signature may wrap, so the return type is read after the matching close
/// paren rather than off the `fn` line.
fn definitions(
    files: &BTreeMap<PathBuf, Vec<(usize, String)>>,
) -> BTreeMap<String, (String, String)> {
    let mut found = BTreeMap::new();
    for (path, lines) in files {
        let joined: String = lines
            .iter()
            .map(|(_, code)| code.as_str())
            .collect::<Vec<_>>()
            .join("\n");
        let numbers: Vec<usize> = lines.iter().map(|(number, _)| *number).collect();
        let bytes = joined.as_bytes();
        let mut at = 0;
        while let Some(hit) = joined[at..].find("fn ") {
            let start = at + hit;
            at = start + 3;
            if start > 0 && is_name_byte(bytes[start - 1]) {
                continue;
            }
            let Some(name) = name_at(&joined, at) else {
                continue;
            };
            let mut open = at + name.len();
            while bytes.get(open).is_some_and(|byte| *byte == b' ') {
                open += 1;
            }
            if bytes.get(open) != Some(&b'(') {
                continue;
            }
            let mut depth = 0usize;
            let mut scan = open;
            while scan < bytes.len() {
                match bytes[scan] {
                    b'(' => depth += 1,
                    b')' => {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    _ => {}
                }
                scan += 1;
            }
            let tail = joined[scan.saturating_add(1)..].trim_start();
            let Some(rest) = tail.strip_prefix("->") else {
                continue;
            };
            let rest = rest.trim_start();
            for kind in ["Loud", "Raised"] {
                if rest.starts_with(kind)
                    && !rest
                        .as_bytes()
                        .get(kind.len())
                        .is_some_and(|byte| is_name_byte(*byte))
                {
                    let line = numbers[joined[..start].matches('\n').count()];
                    let relative = path
                        .strip_prefix(repo_root())
                        .expect("a path under the repository");
                    found.insert(
                        name.to_string(),
                        (kind.to_string(), format!("{}:{line}", relative.display())),
                    );
                }
            }
        }
    }
    found
}

/// Each constructor's own body text, from its `fn` line to the closing brace
/// at that indent.
fn bodies(
    files: &BTreeMap<PathBuf, Vec<(usize, String)>>,
    definitions: &BTreeMap<String, (String, String)>,
) -> BTreeMap<String, String> {
    let mut found = BTreeMap::new();
    for (path, lines) in files {
        let relative = path
            .strip_prefix(repo_root())
            .expect("a path under the repository")
            .display()
            .to_string();
        for (name, (_, at)) in definitions {
            let Some(number) = at
                .strip_prefix(&relative)
                .and_then(|rest| rest.strip_prefix(':'))
                .and_then(|rest| rest.parse::<usize>().ok())
            else {
                continue;
            };
            let start = lines
                .iter()
                .position(|(line, _)| *line == number)
                .expect("the definition's own line is in the file");
            let indent = lines[start].1.len() - lines[start].1.trim_start().len();
            let close = format!("{}}}", " ".repeat(indent));
            let mut end = start + 1;
            while end < lines.len() && lines[end].1 != close {
                end += 1;
            }
            let text: String = lines[start..=end.min(lines.len() - 1)]
                .iter()
                .map(|(_, code)| code.as_str())
                .collect::<Vec<_>>()
                .join("\n");
            found.insert(name.clone(), text);
        }
    }
    found
}

/// Which surfaces a constructor's own construction sites are on, and the text
/// of those sites.
fn surface(
    files: &BTreeMap<PathBuf, Vec<(usize, String)>>,
    definitions: &BTreeMap<String, (String, String)>,
) -> (BTreeMap<String, String>, BTreeMap<String, String>) {
    // A constructor defined outside `error.rs` and `lib.rs` is a free
    // function, so its call sites are spelled without a type in front.
    let free: Vec<&String> = definitions
        .iter()
        .filter(|(_, (_, at))| !at.contains("error.rs:") && !at.contains("lib.rs:"))
        .map(|(name, _)| name)
        .collect();
    let mut tags: BTreeMap<String, Vec<&'static str>> = BTreeMap::new();
    let mut text: BTreeMap<String, String> = BTreeMap::new();
    for (path, lines) in files {
        let shown = path.display().to_string();
        let tag = if shown.ends_with("dispatch.rs") || shown.contains("/dispatch/") {
            "send"
        } else if shown.ends_with("ir.rs") || shown.contains("/ir/") {
            "ir"
        } else {
            "body"
        };
        for (number, code) in lines {
            let mut mark = |name: &str| {
                let at = format!(
                    "{}:{number}",
                    path.strip_prefix(repo_root())
                        .expect("a path under the repository")
                        .display()
                );
                if definitions.get(name).is_some_and(|(_, site)| *site == at) {
                    return;
                }
                let entry = tags.entry(name.to_string()).or_default();
                if !entry.contains(&tag) {
                    entry.push(tag);
                }
                let seen = text.entry(name.to_string()).or_default();
                seen.push('\n');
                seen.push_str(code);
            };
            for prefix in ["Loud::", "Raised::", "Self::", "native::"] {
                let mut at = 0;
                while let Some(hit) = code[at..].find(prefix) {
                    at += hit + prefix.len();
                    if let Some(name) = name_at(code, at)
                        && definitions.contains_key(name)
                    {
                        mark(name);
                    }
                }
            }
            for name in &free {
                let mut at = 0;
                while let Some(hit) = code[at..].find(name.as_str()) {
                    let start = at + hit;
                    at = start + name.len();
                    let before = code.as_bytes().get(start.wrapping_sub(1)).copied();
                    let after = code.as_bytes().get(at).copied();
                    let attached = start > 0
                        && before
                            .is_some_and(|byte| is_name_byte(byte) || byte == b'.' || byte == b':');
                    if !attached && !after.is_some_and(is_name_byte) {
                        mark(name);
                    }
                }
            }
        }
    }
    let surfaces = definitions
        .keys()
        .map(|name| {
            let mut found = tags.remove(name).unwrap_or_default();
            found.sort_unstable();
            (name.clone(), found.join("+"))
        })
        .collect();
    (surfaces, text)
}

/// What a constructor's `answer` column may say, derived from the constructor
/// itself: every `M.N` it builds with `syntax(M, N)`, and the text of its own
/// definition and of its construction sites.
struct Identifiers {
    numbers: Vec<String>,
    text: String,
}

impl Identifiers {
    fn admits(&self, answer: &str) -> bool {
        self.numbers.iter().any(|number| number == answer)
            || (answer.chars().count() >= 4 && self.text.contains(answer))
    }
}

fn identifiers(body: &str, sites: &str) -> Identifiers {
    let mut numbers = Vec::new();
    let mut at = 0;
    while let Some(hit) = body[at..].find("syntax(") {
        let rest = &body[at + hit + "syntax(".len()..];
        at += hit + "syntax(".len();
        let rest = rest.trim_start();
        let major: String = rest.chars().take_while(char::is_ascii_digit).collect();
        if major.is_empty() {
            continue;
        }
        let rest = rest[major.len()..].trim_start();
        let Some(rest) = rest.strip_prefix(',') else {
            continue;
        };
        let rest = rest.trim_start();
        let minor: String = rest.chars().take_while(char::is_ascii_digit).collect();
        if !minor.is_empty() {
            numbers.push(format!("{major}.{minor}"));
        }
    }
    Identifiers {
        numbers,
        text: format!("{body}\n{sites}"),
    }
}

/// The four derived columns, as the committed file spells them.
fn derived() -> Vec<(String, String, String, String)> {
    let files: BTreeMap<PathBuf, Vec<(usize, String)>> = sources()
        .into_iter()
        .map(|path| {
            let lines = code_lines(&path);
            (path, lines)
        })
        .collect();
    let definitions = definitions(&files);
    let (surfaces, _) = surface(&files, &definitions);
    definitions
        .iter()
        .map(|(name, (kind, at))| {
            (
                kind.clone(),
                name.clone(),
                surfaces[name].clone(),
                at.clone(),
            )
        })
        .collect()
}

/// Each constructor's admissible `answer` values.
fn admissible() -> BTreeMap<String, Identifiers> {
    let files: BTreeMap<PathBuf, Vec<(usize, String)>> = sources()
        .into_iter()
        .map(|path| {
            let lines = code_lines(&path);
            (path, lines)
        })
        .collect();
    let definitions = definitions(&files);
    let (_, sites) = surface(&files, &definitions);
    let bodies = bodies(&files, &definitions);
    definitions
        .keys()
        .map(|name| {
            let body = bodies.get(name).map_or("", String::as_str);
            let site = sites.get(name).map_or("", String::as_str);
            (name.clone(), identifiers(body, site))
        })
        .collect()
}

/// The committed table is what the source says, so a constructor added later
/// is red here rather than silently unwalked.
#[test]
fn the_table_holds_every_constructor_the_source_defines() {
    let derived = derived();
    let committed: Vec<(String, String, String, String)> = committed()
        .into_iter()
        .map(|row| (row.kind, row.name, row.surface, row.definition))
        .collect();
    // The rows themselves, not the whole vectors: the table is long enough
    // that an `assert_eq!` on it prints both copies and buries the one line
    // that moved.
    let only_derived: Vec<&(String, String, String, String)> = derived
        .iter()
        .filter(|row| !committed.contains(row))
        .collect();
    let only_committed: Vec<&(String, String, String, String)> = committed
        .iter()
        .filter(|row| !derived.contains(row))
        .collect();
    assert!(
        only_derived.is_empty() && only_committed.is_empty(),
        "corpus/refusal-sites.tsv disagrees with crates/rexx-exec/src; re-derive it rather \
         than editing the disagreeing row.\n  in the source, not the table: {only_derived:#?}\
         \n  in the table, not the source: {only_committed:#?}"
    );
}

/// Every constructor a message send can produce carries a verdict and the
/// witness it was taken on; everything else carries neither.
#[test]
fn every_send_surface_row_is_walked() {
    for row in committed() {
        if row.surface.split('+').any(|tag| tag == "send") {
            assert!(
                ["agrees", "diverges", "recorded", "not-run"].contains(&row.verdict.as_str()),
                "{}: {:?} is not one of the four verdicts",
                row.name,
                row.verdict
            );
            // A row that reached its site names the program that reached it. A
            // row that did not may name the program that went elsewhere, or
            // none at all -- `corpus/oracle-crashes.txt`'s shapes have no
            // witness because running one is what the entry forbids.
            assert!(
                row.reached != "yes" || (row.witness != "-" && !row.witness.trim().is_empty()),
                "{}: reached=yes with no witness naming the program that reached it",
                row.name
            );
        } else {
            assert_eq!(
                (
                    row.verdict.as_str(),
                    row.reached.as_str(),
                    row.answer.as_str(),
                    row.witness.as_str()
                ),
                ("off-send-surface", "-", "-", "-"),
                "{}: a row off the send surface carries no verdict",
                row.name
            );
        }
    }
}

/// **The coverage claim is over the sites a probe reached, not over the rows
/// that exist.** A `reached` of `yes` requires the row's `answer` to be an
/// identifier the constructor itself can produce, and a `no` requires it not to
/// be -- so a row cannot say it walked a site without naming that site's own
/// error number or message, and cannot say it did not while carrying one.
///
/// Without the second direction this would be the shape it exists to catch: an
/// assertion that every site has a row, green over rows whose probe measured an
/// access check that fires first.
#[test]
fn a_reached_row_carries_its_own_site_identifier_and_an_unreached_one_does_not() {
    let admissible = admissible();
    for row in committed() {
        if !row.surface.split('+').any(|tag| tag == "send") {
            continue;
        }
        let identifiers = admissible
            .get(&row.name)
            .unwrap_or_else(|| panic!("{}: no constructor to derive identifiers from", row.name));
        assert!(
            ["yes", "no"].contains(&row.reached.as_str()),
            "{}: reached is {:?}, which is neither yes nor no",
            row.name,
            row.reached
        );
        assert!(
            !row.answer.trim().is_empty() && row.answer != "-",
            "{}: a send-surface row says what its probe answered",
            row.name
        );
        if row.reached == "yes" {
            assert!(
                identifiers.admits(&row.answer),
                "{}: reached=yes, but {:?} is none of this constructor's own identifiers \
                 (its syntax(M, N) numbers are {:?}) and appears nowhere in its definition or \
                 its construction sites -- so the probe reached a different check",
                row.name,
                row.answer,
                identifiers.numbers
            );
        } else {
            assert!(
                !identifiers.admits(&row.answer),
                "{}: reached=no, but {:?} is one of this constructor's own identifiers, \
                 so the probe did reach it",
                row.name,
                row.answer
            );
        }
    }
}

/// The `answer` values more than one send-surface row carries, with the rows
/// that carry them.
///
/// `admits` accepts an answer that is one of the constructor's own
/// `syntax(M, N)` numbers, so two constructors raising the same number admit
/// each other's rows and
/// [`a_reached_row_carries_its_own_site_identifier_and_an_unreached_one_does_not`]
/// cannot tell a transposition inside such a pair from the truth. Measured, at
/// both entries below: swapping a pair's `answer` and `witness` together
/// leaves every test in this file green.
const SHARED_ANSWERS: &[(&str, &[&str])] = &[
    (
        "88.909",
        &[
            "argument_needs_a_string_value",
            "named_argument_needs_a_string_value",
        ],
    ),
    (
        "88.914",
        &[
            "argument_not_a_class",
            "argument_not_an_instance",
            "scope_override_not_a_class",
        ],
    ),
];

/// The rows sharing an `answer` are the recorded ones, so the table's own
/// header cannot go stale about what its check does not see.
///
/// The limit itself is not closed here: this says which pairs carry it.
#[test]
fn the_answers_more_than_one_row_shares_are_the_recorded_ones() {
    let mut by_answer: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for row in committed() {
        if row.surface.split('+').any(|tag| tag == "send") {
            by_answer.entry(row.answer).or_default().push(row.name);
        }
    }
    let shared: Vec<(String, Vec<String>)> = by_answer
        .into_iter()
        .filter(|(_, names)| names.len() > 1)
        .collect();
    let recorded: Vec<(String, Vec<String>)> = SHARED_ANSWERS
        .iter()
        .map(|(answer, names)| {
            (
                (*answer).to_string(),
                names.iter().map(|name| (*name).to_string()).collect(),
            )
        })
        .collect();
    assert_eq!(
        shared, recorded,
        "the send-surface rows sharing an `answer` have changed. The table's header names \
         the pairs this file's check cannot discriminate; correct both together"
    );
}

/// The scanner reads the source rather than a copy of the answer: the file it
/// is pointed at decides what it finds.
///
/// Without this, [`the_table_holds_every_constructor_the_source_defines`]
/// would be green over a scanner that found nothing at all and a table that
/// was empty -- and the table is not empty, so the pair is what says the
/// scanner ran.
#[test]
fn the_scanner_finds_constructors_and_only_constructors() {
    let derived = derived();
    assert!(
        derived
            .iter()
            .any(|(kind, name, _, _)| kind == "Loud" && name == "native_method"),
        "the scanner missed a constructor `impl Loud` defines"
    );
    assert!(
        derived
            .iter()
            .any(|(kind, name, _, _)| kind == "Raised" && name == "no_method"),
        "the scanner missed a constructor `error.rs` defines"
    );
    assert!(
        !derived.iter().any(|(_, name, _, _)| name == "parse"),
        "`Loud::parse` is named only in comments and must not be counted"
    );
    assert!(
        derived
            .iter()
            .any(|(_, name, _, _)| name == "raised_if_not_logical"),
        "the scanner missed a constructor that is not spelled `Raised::` at its definition"
    );
}
