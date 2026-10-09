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

//! **Every refusal that names no phase has a committed disposition.**
//!
//! `closed_phases.rs` holds that no refusal names a phase that has closed. A
//! refusal that names no phase at all escapes that check, so this file reads
//! every `Loud` constructor in `src/` and requires each one that can build a
//! message with no owner to have a row in `corpus/refusal-dispositions.tsv`,
//! saying why no phase owes it: GUARD (no program reaches it; the record names
//! the probes that tried), DEVIATION (matching the oracle is wrong; the record
//! cites an `oracle-crashes.txt` entry or an exclusions row), LIMIT (a design
//! limit; the record cites the deviation or the design document that licenses
//! it), or REHOME (a remaining phase owes it, and the constructor names that
//! phase).
//!
//! An owner is read from the constructor's own `owned_message` call: `None`, or
//! no `owned_message` at all, is no owner; `Some("Phase N")` is phase N; an
//! `Option` a caller passes in is no owner, since a caller can pass `None`; and
//! an owner computed by a function is no owner unless [`COMPUTED_OWNERS`] names
//! the function and the test that holds it to `Some`.
//! Every `message:` field of a constructor is read, and the constructor has
//! an owner only when each of them does. A `Loud` struct literal
//! outside a constructor must carry an owner, so an ownerless refusal always
//! has a constructor name to key a row on.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Owners computed by a function that never answers `None` where the refusal
/// is built, with the test that holds it so.
const COMPUTED_OWNERS: &[(&str, &str)] = &[
    (
        "rexx_inventory::builtins::owner_of",
        "`unresolved_call` is built for an excluded builtin, whose owner \
         owners.rs's every_excluded_builtin_names_a_phase_that_owes_it requires, \
         and for `BuiltinTarget::Gap`, which builtin/tests.rs asserts no name reaches",
    ),
    (
        "rexx_api::layout::refusal_owner",
        "an unfilled slot is a refusing stub, and rexx-api's \
         a_populated_table_refuses_exactly_the_members_it_names holds the refusing \
         stubs to the `REFUSING_MEMBERS` rows, each of which names its phase",
    ),
];

/// The phases a REHOME row may name: the ones still open.
const OPEN_PHASES: &[&str] = &["Phase 9", "Phase 10", "Phase 11"];

/// The four dispositions.
const KINDS: &[&str] = &["GUARD", "DEVIATION", "LIMIT", "REHOME"];

/// Whose a constructor's refusal is.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Owner {
    /// It can build a message with no owner.
    None,
    /// It always names this phase.
    Phase(String),
    /// It names a phase its caller passes as a `&'static str`, or one a
    /// [`COMPUTED_OWNERS`] function answers.
    Given,
}

/// One committed row.
#[derive(Debug)]
struct Row {
    constructor: String,
    kind: String,
    phase: String,
    record: String,
}

fn rust_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("the crate sits two levels below `rust/`")
        .to_path_buf()
}

fn table_path() -> PathBuf {
    rust_root().join("corpus/refusal-dispositions.tsv")
}

fn parse_table(text: &str) -> Vec<Row> {
    text.lines()
        .filter(|line| !line.starts_with('#') && !line.trim().is_empty())
        .map(|line| {
            let fields: Vec<&str> = line.split('\t').collect();
            assert_eq!(
                fields.len(),
                4,
                "four tab-separated fields (constructor, kind, phase, record) in {line:?}"
            );
            Row {
                constructor: fields[0].to_string(),
                kind: fields[1].to_string(),
                phase: fields[2].to_string(),
                record: fields[3].to_string(),
            }
        })
        .collect()
}

/// Every non-test `.rs` file under `src/`, with `#[cfg(test)] mod` blocks
/// dropped and line comments stripped, as `(line number, code)` pairs.
fn sources() -> Vec<(String, Vec<(usize, String)>)> {
    let mut found = Vec::new();
    let mut pending = vec![Path::new(env!("CARGO_MANIFEST_DIR")).join("src")];
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(&directory).expect("a directory this crate compiles") {
            let path = entry.expect("a readable directory entry").path();
            let shown = path.display().to_string();
            if path.is_dir() {
                if !shown.ends_with("/tests") {
                    pending.push(path);
                }
            } else if shown.ends_with(".rs")
                && !shown.ends_with("/tests.rs")
                && !shown.ends_with("_tests.rs")
            {
                let text = fs::read_to_string(&path).expect("a source file this crate compiles");
                let relative = path
                    .strip_prefix(rust_root())
                    .expect("a path under rust/")
                    .display()
                    .to_string();
                found.push((relative, code_lines(&text)));
            }
        }
    }
    found.sort();
    found
}

fn code_lines(text: &str) -> Vec<(usize, String)> {
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
                let mut close = opener + 1;
                while close < lines.len() && lines[close] != "}" {
                    close += 1;
                }
                at = close + 1;
                continue;
            }
        }
        let code = match lines[at].find("//") {
            Some(cut) => &lines[at][..cut],
            None => lines[at],
        };
        out.push((at + 1, code.to_string()));
        at += 1;
    }
    out
}

/// A `Loud` constructor: its name, where it is, its parameter list and its
/// body.
struct Constructor {
    name: String,
    site: String,
    params: String,
    body: String,
}

/// Whether the line at `index` of `lines` sits inside an `impl Loud` block,
/// for each line.
fn in_impl_loud(lines: &[(usize, String)]) -> Vec<bool> {
    let mut inside = false;
    lines
        .iter()
        .map(|(_, code)| {
            if code.starts_with("impl Loud") {
                inside = true;
            } else if code == "}" {
                inside = false;
            }
            inside
        })
        .collect()
}

/// Every `fn NAME(...) -> Loud`, and `-> Self` inside `impl Loud`, in
/// `files`, with the body up to the closing brace at the `fn` line's indent.
fn constructors(files: &[(String, Vec<(usize, String)>)]) -> Vec<Constructor> {
    let mut found = Vec::new();
    for (path, lines) in files {
        let impl_loud = in_impl_loud(lines);
        for (index, (number, code)) in lines.iter().enumerate() {
            let Some(at) = code.find("fn ") else {
                continue;
            };
            if at > 0 && code.as_bytes()[at - 1].is_ascii_alphanumeric() {
                continue;
            }
            // The signature may wrap; join lines until the body opens.
            let mut signature = code[at + 3..].to_string();
            let mut end = index;
            while !signature.contains('{') && !signature.contains(';') && end + 1 < lines.len() {
                end += 1;
                signature.push(' ');
                signature.push_str(lines[end].1.trim());
            }
            let Some(open) = signature.find('(') else {
                continue;
            };
            let name = signature[..open].trim().to_string();
            if name.is_empty() || !name.bytes().all(|b| b == b'_' || b.is_ascii_alphanumeric()) {
                continue;
            }
            let Some(close) = signature.rfind(')') else {
                continue;
            };
            let returns = signature[close + 1..].trim_start();
            let Some(returns) = returns.strip_prefix("->") else {
                continue;
            };
            let returns = returns.trim_start();
            let names = |kind: &str| {
                returns.starts_with(kind)
                    && !returns
                        .as_bytes()
                        .get(kind.len())
                        .is_some_and(|b| *b == b'_' || b.is_ascii_alphanumeric())
            };
            if !(names("Loud") || (impl_loud[index] && names("Self"))) {
                continue;
            }
            let indent = code.len() - code.trim_start().len();
            let closing = format!("{}}}", " ".repeat(indent));
            let mut stop = end + 1;
            while stop < lines.len() && lines[stop].1.trim_end() != closing {
                stop += 1;
            }
            let body = lines[index..stop.min(lines.len())]
                .iter()
                .map(|(_, code)| code.as_str())
                .collect::<Vec<_>>()
                .join("\n");
            found.push(Constructor {
                name,
                site: format!("{path}:{number}"),
                params: signature[open + 1..close].to_string(),
                body,
            });
        }
    }
    found
}

/// The top-level arguments of the call whose `(` is at `open` in `text`.
fn arguments(text: &str, open: usize) -> Vec<String> {
    let bytes = text.as_bytes();
    let mut depth = 0usize;
    let mut quoted = false;
    let mut current = String::new();
    let mut out = Vec::new();
    let mut at = open;
    while at < bytes.len() {
        let byte = bytes[at];
        if quoted {
            current.push(byte as char);
            if byte == b'\\' {
                if let Some(next) = bytes.get(at + 1) {
                    current.push(*next as char);
                }
                at += 2;
                continue;
            }
            if byte == b'"' {
                quoted = false;
            }
            at += 1;
            continue;
        }
        match byte {
            b'"' => {
                quoted = true;
                current.push('"');
            }
            b'(' | b'[' | b'{' => {
                depth += 1;
                if depth > 1 {
                    current.push(byte as char);
                }
            }
            b')' | b']' | b'}' => {
                depth -= 1;
                if depth == 0 {
                    out.push(current.trim().to_string());
                    return out;
                }
                current.push(byte as char);
            }
            b',' if depth == 1 => {
                out.push(current.trim().to_string());
                current.clear();
            }
            _ => current.push(byte as char),
        }
        at += 1;
    }
    out
}

/// The expression starting at `from` in `text`, up to the comma or closing
/// bracket that ends it at its own depth.
fn expression_at(text: &str, from: usize) -> &str {
    let bytes = text.as_bytes();
    let mut depth = 0usize;
    let mut quoted = false;
    let mut at = from;
    while at < bytes.len() {
        let byte = bytes[at];
        if quoted {
            match byte {
                b'\\' => at += 1,
                b'"' => quoted = false,
                _ => {}
            }
        } else {
            match byte {
                b'"' => quoted = true,
                b'(' | b'[' | b'{' => depth += 1,
                b')' | b']' | b'}' if depth == 0 => break,
                b')' | b']' | b'}' => depth -= 1,
                b',' if depth == 0 => break,
                _ => {}
            }
        }
        at += 1;
    }
    text[from..at.min(text.len())].trim()
}

/// The index just past the bracket matching the one at `open` in `text`, or
/// the end of `text`.
fn closing(text: &str, open: usize) -> usize {
    let bytes = text.as_bytes();
    let mut depth = 0usize;
    let mut quoted = false;
    let mut at = open;
    while at < bytes.len() {
        let byte = bytes[at];
        if quoted {
            match byte {
                b'\\' => at += 1,
                b'"' => quoted = false,
                _ => {}
            }
        } else {
            match byte {
                b'"' => quoted = true,
                b'(' | b'[' | b'{' => depth += 1,
                b')' | b']' | b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        return at + 1;
                    }
                }
                _ => {}
            }
        }
        at += 1;
    }
    text.len()
}

/// The owner every refusal built in `body` carries, given the parameters
/// `params` its owner may come from: each `message:` field is read, and the
/// body has an owner only when every one of them does.
fn owner_in(body: &str, params: &str) -> Owner {
    let mut owners = Vec::new();
    for (at, _) in body.match_indices("message:") {
        if body[..at]
            .bytes()
            .next_back()
            .is_some_and(|b| b == b'_' || b.is_ascii_alphanumeric())
        {
            continue;
        }
        owners.push(message_owner(
            expression_at(body, at + "message:".len()),
            body,
            params,
        ));
    }
    if owners.is_empty() || owners.contains(&Owner::None) {
        return Owner::None;
    }
    if owners.iter().all(|owner| *owner == owners[0]) {
        return owners.swap_remove(0);
    }
    Owner::Given
}

/// The owner of one `message:` field's value `field`.
fn message_owner(field: &str, body: &str, params: &str) -> Owner {
    let field = field.strip_prefix("crate::").unwrap_or(field);
    if !field.starts_with("owned_message(") {
        return Owner::None;
    }
    let args = arguments(field, "owned_message".len());
    let Some(mut owner) = args.get(1).cloned() else {
        return Owner::None;
    };
    owner = owner.split_whitespace().collect::<Vec<_>>().join(" ");
    // A local binding: read through it once.
    if owner
        .bytes()
        .all(|b| b == b'_' || b.is_ascii_alphanumeric())
    {
        let binding = format!("let {owner} = ");
        if let Some(found) = body.find(&binding) {
            let rest = &body[found + binding.len()..];
            let end = rest.find(';').unwrap_or(rest.len());
            owner = rest[..end].split_whitespace().collect::<Vec<_>>().join(" ");
        }
    }
    let param = |name: &str, kind: &str| {
        let squeezed: String = params.split_whitespace().collect();
        let wanted = format!("{name}:{kind}");
        squeezed.match_indices(&wanted).any(|(at, _)| {
            !squeezed[..at]
                .bytes()
                .next_back()
                .is_some_and(|b| b == b'_' || b.is_ascii_alphanumeric())
        })
    };
    if let Some(inner) = owner
        .strip_prefix("Some(")
        .and_then(|s| s.strip_suffix(')'))
    {
        if let Some(literal) = inner.strip_prefix('"').and_then(|s| s.strip_suffix('"')) {
            return Owner::Phase(literal.to_string());
        }
        if param(inner, "&'staticstr") {
            return Owner::Given;
        }
    }
    if COMPUTED_OWNERS
        .iter()
        .any(|(function, _)| owner.starts_with(&format!("{function}(")))
    {
        return Owner::Given;
    }
    // `None`, an `Option` parameter, a function this file does not vouch
    // for, or anything else: a caller or a branch can make it `None`.
    Owner::None
}

/// Every `Loud { ... }` struct literal outside a constructor, and `Self {
/// ... }` inside `impl Loud`, as its site and the owner it carries. The
/// literal is read to its matching brace. The parameters an owner may come
/// from are read from the nearest enclosing signature or closure header
/// above the literal.
fn literals(
    files: &[(String, Vec<(usize, String)>)],
    constructors: &[Constructor],
) -> Vec<(String, Owner)> {
    let mut found = Vec::new();
    for (path, lines) in files {
        let impl_loud = in_impl_loud(lines);
        for (index, (number, code)) in lines.iter().enumerate() {
            if code.contains("struct Loud")
                || code.contains("impl Loud")
                || code.contains("-> Loud")
                || code.contains("-> Self")
            {
                continue;
            }
            let opener = ["Loud {", "Self {"]
                .iter()
                .filter(|opener| **opener == "Loud {" || impl_loud[index])
                .filter_map(|opener| code.find(opener))
                .find(|&at| at == 0 || !code.as_bytes()[at - 1].is_ascii_alphanumeric());
            let Some(at) = opener else {
                continue;
            };
            let site = format!("{path}:{number}");
            let inside = constructors.iter().any(|ctor| {
                let Some((file, line)) = ctor.site.rsplit_once(':') else {
                    return false;
                };
                let line: usize = line.parse().expect("a line number");
                let length = ctor.body.lines().count();
                file == path && (line..line + length).contains(number)
            });
            if inside {
                continue;
            }
            let rest = lines[index..]
                .iter()
                .map(|(_, code)| code.as_str())
                .collect::<Vec<_>>()
                .join("\n");
            let open = at + "Loud ".len();
            let literal = &rest[open..closing(&rest, open)];
            let header = lines[index.saturating_sub(15)..index]
                .iter()
                .map(|(_, code)| code.as_str())
                .collect::<Vec<_>>()
                .join(",");
            found.push((site, owner_in(literal, &header)));
        }
    }
    found
}

/// Every constructor that can build a message with no owner, and every one
/// that names a phase, by name.
fn owners(constructors: &[Constructor]) -> BTreeMap<String, (Owner, String)> {
    constructors
        .iter()
        .map(|ctor| {
            (
                ctor.name.clone(),
                (owner_in(&ctor.body, &ctor.params), ctor.site.clone()),
            )
        })
        .collect()
}

/// Whether `record` cites an `oracle-crashes.txt` entry that exists, and the
/// citations that do not.
fn crash_citations(record: &str, crashes: &str) -> (bool, Vec<String>) {
    citations(record, "oracle-crashes.txt entry ", |number| {
        crashes.contains(&format!("\n## {number}. "))
    })
}

/// Whether `record` cites a numbered deviation that exists in the
/// exclusions file's DEVIATIONS list, and the citations that do not.
fn deviation_citations(record: &str, exclusions: &str) -> (bool, Vec<String>) {
    citations(record, "Deviation ", |number| {
        exclusions.lines().any(|line| {
            line.trim_start()
                .strip_prefix(&format!("{number}. "))
                .is_some_and(|rest| rest.starts_with(|c: char| c.is_ascii_uppercase() || c == '`'))
        })
    })
}

fn citations(record: &str, marker: &str, exists: impl Fn(&str) -> bool) -> (bool, Vec<String>) {
    let mut any = false;
    let mut missing = Vec::new();
    for (at, _) in record.match_indices(marker) {
        let number: String = record[at + marker.len()..]
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric())
            .collect();
        if number.is_empty() || !number.starts_with(|c: char| c.is_ascii_digit()) {
            continue;
        }
        any = true;
        if !exists(&number) {
            missing.push(format!("{marker}{number}"));
        }
    }
    (any, missing)
}

/// The repository paths `record` names (`docs/...`, `rust/...`), each with
/// any `:line` suffix dropped, that do not exist.
fn missing_paths(record: &str, repository: &Path) -> (bool, Vec<String>) {
    let mut any = false;
    let mut missing = Vec::new();
    for word in record.split(|c: char| c.is_whitespace() || c == ',' || c == ';' || c == '`') {
        if !(word.starts_with("docs/") || word.starts_with("rust/")) {
            continue;
        }
        let path = word
            .split(':')
            .next()
            .unwrap_or(word)
            .trim_end_matches(['.', ')']);
        any = true;
        if !repository.join(path).exists() {
            missing.push(path.to_string());
        }
    }
    (any, missing)
}

/// Everything wrong with `rows` against the constructors `owners` derives
/// from the source and the ownerless `literals` outside a constructor.
fn problems(
    owners: &BTreeMap<String, (Owner, String)>,
    literals: &[(String, Owner)],
    rows: &[Row],
    documents: &Documents,
) -> Vec<String> {
    let mut found = Vec::new();
    for (site, owner) in literals {
        if *owner == Owner::None {
            found.push(format!(
                "{site}: a `Loud` struct literal with no owner; make it a constructor so it can \
                 carry a disposition row"
            ));
        }
    }
    let mut seen: BTreeMap<&str, usize> = BTreeMap::new();
    for row in rows {
        *seen.entry(row.constructor.as_str()).or_default() += 1;
        if !KINDS.contains(&row.kind.as_str()) {
            found.push(format!(
                "{}: kind {:?} is not one of {KINDS:?}",
                row.constructor, row.kind
            ));
            continue;
        }
        let Some((owner, site)) = owners.get(&row.constructor) else {
            found.push(format!(
                "{}: a row with no constructor in src/",
                row.constructor
            ));
            continue;
        };
        if row.record.trim().is_empty() || row.record == "-" {
            found.push(format!("{}: a row with no record", row.constructor));
        }
        if row.kind == "REHOME" {
            if !OPEN_PHASES.contains(&row.phase.as_str()) {
                found.push(format!(
                    "{}: REHOME to {:?}, which is not an open phase {OPEN_PHASES:?}",
                    row.constructor, row.phase
                ));
            }
            if *owner != Owner::Phase(row.phase.clone()) {
                found.push(format!(
                    "{}: REHOME to {}, but the constructor at {site} carries {owner:?}",
                    row.constructor, row.phase
                ));
            }
            let quoted: Vec<&str> = row.record.split('"').skip(1).step_by(2).collect();
            if quoted.is_empty() || quoted.iter().any(|q| !documents.roadmap.contains(q)) {
                found.push(format!(
                    "{}: a REHOME record quotes the roadmap row naming the work, and every quote \
                     is found in it",
                    row.constructor
                ));
            }
            continue;
        }
        if row.phase != "-" {
            found.push(format!(
                "{}: phase {:?} on a {} row; only REHOME names a phase",
                row.constructor, row.phase, row.kind
            ));
        }
        if *owner != Owner::None {
            found.push(format!(
                "{}: a {} row for a constructor at {site} that carries {owner:?}",
                row.constructor, row.kind
            ));
        }
        let (crash, bad_crash) = crash_citations(&row.record, &documents.crashes);
        let (deviation, bad_deviation) = deviation_citations(&row.record, &documents.exclusions);
        let (path, bad_path) = missing_paths(&row.record, &documents.repository);
        for bad in bad_crash.iter().chain(&bad_deviation).chain(&bad_path) {
            found.push(format!(
                "{}: cites {bad}, which does not exist",
                row.constructor
            ));
        }
        let quoted_row = row
            .record
            .split('"')
            .skip(1)
            .step_by(2)
            .any(|q| q.len() >= 12 && documents.exclusions.contains(q));
        match row.kind.as_str() {
            "DEVIATION" if !(crash || deviation || quoted_row) => found.push(format!(
                "{}: a DEVIATION cites an oracle-crashes.txt entry, a Deviation, or quotes an \
                 exclusions row",
                row.constructor
            )),
            "LIMIT" if !(deviation || path) => found.push(format!(
                "{}: a LIMIT cites the Deviation or the design document that licenses it",
                row.constructor
            )),
            _ => {}
        }
    }
    for (name, count) in seen {
        if count > 1 {
            found.push(format!("{name}: {count} rows"));
        }
    }
    for (name, (owner, site)) in owners {
        if *owner == Owner::None && !rows.iter().any(|row| row.constructor == *name) {
            found.push(format!("{name} ({site}): no owner and no disposition row"));
        }
    }
    found
}

/// The files a record's citations are checked against.
struct Documents {
    crashes: String,
    exclusions: String,
    roadmap: String,
    repository: PathBuf,
}

fn documents() -> Documents {
    let repository = rust_root()
        .parent()
        .expect("rust/ sits in the repository")
        .to_path_buf();
    let read = |path: &str| {
        fs::read_to_string(repository.join(path)).unwrap_or_else(|e| panic!("{path}: {e}"))
    };
    Documents {
        crashes: read("rust/corpus/oracle-crashes.txt"),
        exclusions: read("docs/superpowers/plans/phase-4-exclusions.txt"),
        roadmap: read("docs/superpowers/plans/2026-07-27-rust-rewrite.md"),
        repository,
    }
}

/// The whole claim.
#[test]
fn every_ownerless_refusal_has_a_disposition() {
    let files = sources();
    let constructors = constructors(&files);
    let literals = literals(&files, &constructors);
    let owners = owners(&constructors);
    let text = fs::read_to_string(table_path()).unwrap_or_default();
    let found = problems(&owners, &literals, &parse_table(&text), &documents());
    assert!(
        found.is_empty(),
        "corpus/refusal-dispositions.tsv and src/ disagree:\n{}",
        found.join("\n")
    );
}

/// The scanner finds the constructors it has to, so an empty problem list is
/// a measurement and not a scan of nothing.
#[test]
fn the_scanner_reads_each_owner_form() {
    let constructors = constructors(&sources());
    let owners = owners(&constructors);
    assert_eq!(
        owners.get("native_method").map(|(owner, _)| owner),
        Some(&Owner::Phase("Phase 9".to_string()))
    );
    assert_eq!(
        owners.get("named_semaphore").map(|(owner, _)| owner),
        Some(&Owner::Phase("Phase 10".to_string()))
    );
    assert_eq!(
        owners.get("internal_routine").map(|(owner, _)| owner),
        Some(&Owner::Given)
    );
    assert_eq!(
        owners.get("deferred_send").map(|(owner, _)| owner),
        Some(&Owner::Given),
        "a constructor outside lib.rs is found"
    );
    assert_eq!(
        owners.get("unresolved_call").map(|(owner, _)| owner),
        Some(&Owner::Given),
        "an owner read through a local binding"
    );
    assert_eq!(
        owners.get("binary_operator").map(|(owner, _)| owner),
        Some(&Owner::None),
        "a message with no owned_message has no owner"
    );
    assert_eq!(
        owners.get("instruction").map(|(owner, _)| owner),
        Some(&Owner::None),
        "an owner function this file does not vouch for"
    );
}

/// Each way a table can be wrong is found: an ownerless constructor with no
/// row, a row with no constructor, a REHOME whose constructor carries another
/// phase, a disposition row for a constructor that names a phase, an ownerless
/// struct literal (beside an owned one too), a citation that does not
/// resolve, a `-> Self` constructor, and one whose every branch is not owned.
#[test]
fn the_check_finds_each_kind_of_disagreement() {
    let source = "\
impl Loud {
    fn plain() -> Loud {
        Loud {
            message: \"x\".to_string(),
        }
    }
    fn none(what: &str) -> Loud {
        Loud {
            message: owned_message(what, None),
        }
    }
    fn nine() -> Loud {
        Loud {
            message: owned_message(\"y\", Some(\"Phase 9\")),
        }
    }
    fn given(owner: &'static str) -> Loud {
        Loud {
            message: owned_message(\"z\", Some(owner)),
        }
    }
    fn optional(owner: Option<&'static str>) -> Loud {
        Loud {
            message: owned_message(\"w\", owner),
        }
    }
    fn mutant_self(what: &str) -> Self {
        Self {
            message: owned_message(what, None),
        }
    }
    fn mixed(what: &str, early: bool) -> Loud {
        if early {
            return Loud {
                message: owned_message(what, Some(\"Phase 9\")),
            };
        }
        Loud {
            message: what.to_string(),
        }
    }
}
fn elsewhere() -> Result<(), Failure> {
    Err(Loud {
        message: owned_message(\"v\", None),
    }
    .into())
}
fn adjacent(r: u8) -> Failure {
    match r {
        0 => return Loud { message: String::new() }.into(),
        _ => Loud {
            message: owned_message(\"t\", Some(\"Phase 10\")),
        }
        .into(),
    }
}
";
    let files = vec![("src/lib.rs".to_string(), code_lines(source))];
    let constructors = constructors(&files);
    let owners = owners(&constructors);
    let literals = literals(&files, &constructors);
    assert_eq!(owners.len(), 7);
    assert_eq!(owners["optional"].0, Owner::None);
    assert_eq!(owners["given"].0, Owner::Given);
    assert_eq!(owners["mutant_self"].0, Owner::None);
    assert_eq!(owners["mixed"].0, Owner::None);
    let documents = Documents {
        crashes: "\n## 6. An array\n".to_string(),
        exclusions: " 20. A WAIT NOTHING LEFT TO RUN CAN END IS REFUSED\n".to_string(),
        roadmap: "the row's text".to_string(),
        repository: rust_root().parent().expect("the repository").to_path_buf(),
    };
    let table = "\
plain\tGUARD\t-\tprobe p1
none\tDEVIATION\t-\toracle-crashes.txt entry 6
optional\tLIMIT\t-\tDeviation 20
mutant_self\tGUARD\t-\tprobe p4
mixed\tGUARD\t-\tprobe p5
";
    assert!(
        problems(&owners, &[], &parse_table(table), &documents).is_empty(),
        "{:?}",
        problems(&owners, &[], &parse_table(table), &documents)
    );
    let wrong = "\
plain\tGUARD\t-\tprobe p1
none\tDEVIATION\t-\toracle-crashes.txt entry 7
gone\tGUARD\t-\tprobe p2
nine\tREHOME\tPhase 10\t\"the row's text\"
given\tGUARD\t-\tprobe p3
";
    let found = problems(&owners, &literals, &parse_table(wrong), &documents);
    let expect = [
        "src/lib.rs:44: a `Loud` struct literal",
        "src/lib.rs:51: a `Loud` struct literal",
        "mutant_self (src/lib.rs:27): no owner and no disposition row",
        "mixed (src/lib.rs:32): no owner and no disposition row",
        "none: cites oracle-crashes.txt entry 7",
        "gone: a row with no constructor",
        "nine: REHOME to Phase 10, but the constructor",
        "given: a GUARD row for a constructor",
        "optional (src/lib.rs:22): no owner and no disposition row",
    ];
    for needle in expect {
        assert!(
            found.iter().any(|problem| problem.starts_with(needle)),
            "{needle:?} not among {found:#?}"
        );
    }
    assert_eq!(found.len(), expect.len(), "{found:#?}");
}
