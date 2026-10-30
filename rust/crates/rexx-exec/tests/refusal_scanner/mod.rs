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

use super::{BTreeMap, Path, PathBuf, repo_root};

/// Every `.rs` file under `src/`, with `#[cfg(test)] mod` blocks dropped and
/// line comments stripped, as `(line number, code)` pairs.
pub(super) fn code_lines(path: &Path) -> Vec<(usize, String)> {
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

pub(super) fn sources() -> Vec<PathBuf> {
    sources_in(&Path::new(env!("CARGO_MANIFEST_DIR")).join("src"))
}

fn sources_in(root: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut pending = vec![root.to_path_buf()];
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
    let mut test_files = Vec::new();
    let mut test_directories = Vec::new();
    for path in &found {
        let text = std::fs::read_to_string(path).expect("a source file this crate compiles");
        let mut test_attribute = false;
        for line in text.lines() {
            let code = line.split("//").next().unwrap().trim();
            if code == "#[cfg(test)]" {
                test_attribute = true;
                continue;
            }
            if code.is_empty() || code.starts_with("#[") {
                continue;
            }
            if test_attribute
                && let Some(name) = code.strip_prefix("mod ").and_then(|s| s.strip_suffix(';'))
            {
                let parent = path.parent().unwrap();
                let directory = if matches!(path.file_name().and_then(|s| s.to_str()), Some("lib.rs" | "mod.rs")) {
                    parent.to_path_buf()
                } else {
                    parent.join(path.file_stem().unwrap())
                };
                test_files.push(directory.join(format!("{name}.rs")));
                test_directories.push(directory.join(name));
            }
            test_attribute = false;
        }
    }
    found.retain(|path| !test_files.contains(path) && !test_directories.iter().any(|directory| path.starts_with(directory)));
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
pub(super) fn definitions(
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
pub(super) fn bodies(
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

/// Whether the definition belongs to an impl block in rustfmt-formatted source.
fn defined_in_impl(files: &BTreeMap<PathBuf, Vec<(usize, String)>>, site: &str) -> bool {
    let (path, line) = site.rsplit_once(':').expect("a definition site");
    let line = line.parse::<usize>().expect("a source line");
    let lines = &files[&repo_root().join(path)];
    let mut owners = Vec::new();
    for (number, code) in lines {
        if *number == line {
            return !owners.is_empty();
        }
        let trimmed = code.trim_start();
        let indent = code.len() - trimmed.len();
        if trimmed.starts_with("impl ") || trimmed.starts_with("impl<") {
            owners.push(indent);
        } else if trimmed == "}" && owners.last() == Some(&indent) {
            owners.pop();
        }
    }
    panic!("the definition's line is in the file");
}

/// Which surfaces a constructor's own construction sites are on, and the text
/// of those sites.
pub(super) fn surface(
    files: &BTreeMap<PathBuf, Vec<(usize, String)>>,
    definitions: &BTreeMap<String, (String, String)>,
) -> (BTreeMap<String, String>, BTreeMap<String, String>) {
    // A constructor defined outside an impl is a free function, so its
    // call sites are spelled without a type in front.
    let free: Vec<&String> = definitions
        .iter()
        .filter(|(_, (_, at))| !defined_in_impl(files, at))
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


#[cfg(test)]
mod tests;
