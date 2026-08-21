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

//! The `oodocs/` extractors, and the row sets they emit into
//! `rust/corpus/docs/`.
//!
//! Each row set is the denominator of one half of a Phase 5 gate table, so a
//! row that stops being derived matters exactly as much as one that appears.
//! The files are committed beside these extractors and re-derived by
//! `tests/extract_docs.rs`, which compares in **both** directions and asserts
//! each file's revision stamp against the checkout it was derived from.
//!
//! Every extractor here is text-level; [`xml`]'s own doc gives the reason,
//! which is a property of the books rather than a convenience.

pub mod classes;
pub mod directives;
pub mod hierarchy;
pub mod provide;
pub mod xml;

/// The books these extractors read, relative to `oodocs/`.
pub const REXXREF: &str = "rexxref/en-US";

/// The four class books, in the order the class-set row block follows.
pub const CLASS_BOOKS: &[&str] = &[
    "fundclasses.xml",
    "collclasses.xml",
    "utilityclasses.xml",
    "streamclasses.xml",
];

/// A committed row set: the header comment its file carries, and its rows.
///
/// The header is part of the artifact rather than something the writer adds,
/// because the both-directions check compares whole files: a header edited in
/// the data file and not in the extractor is as red as a missing row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowSet {
    pub header: Vec<String>,
    pub rows: Vec<String>,
}

impl RowSet {
    /// The file's exact bytes: header lines, a blank line, then the rows.
    ///
    /// A header line is hard-wrapped, because a paragraph built by `format!`
    /// otherwise lands as one very long line in a file whose whole point is
    /// being read by a person. Lines indented in the source keep their
    /// indentation and are not wrapped, so a derived list stays a list.
    pub fn render(&self) -> String {
        let mut out = String::new();
        for line in &self.header {
            for wrapped in wrap(line) {
                out.push('#');
                if !wrapped.is_empty() {
                    out.push(' ');
                    out.push_str(&wrapped);
                }
                out.push('\n');
            }
        }
        out.push('\n');
        for row in &self.rows {
            out.push_str(row);
            out.push('\n');
        }
        out
    }
}

/// Header text at a width a terminal and a diff both handle.
const HEADER_WIDTH: usize = 74;

fn wrap(line: &str) -> Vec<String> {
    if line.len() <= HEADER_WIDTH || line.starts_with(' ') {
        return vec![line.to_string()];
    }
    let mut out = Vec::new();
    let mut current = String::new();
    for word in line.split(' ') {
        if !current.is_empty() && current.len() + 1 + word.len() > HEADER_WIDTH {
            out.push(std::mem::take(&mut current));
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(word);
    }
    if !current.is_empty() {
        out.push(current);
    }
    out
}

/// The stamp line every row set carries, naming the upstream revision the
/// rows were derived at (D56).
///
/// Read back by `tests/extract_docs.rs` against `svn info`, which is the only
/// thing that can say whether a committed row set is stale: `oodocs/` is
/// git-ignored, so nothing in the checkout records what it held.
pub fn stamp_line(revision: &str) -> String {
    format!("derived-at: oodocs/{REXXREF} {revision}")
}

/// The `Revision:` line of `svn info <path>`.
///
/// `oodocs/` itself is **not** a working copy -- `svn info` on it is
/// `E155007` -- so this is asked of `oodocs/rexxref`, which is one.
pub fn svn_revision(path: &std::path::Path) -> Result<String, String> {
    let out = std::process::Command::new("svn")
        .arg("info")
        .arg(path)
        .output()
        .map_err(|e| format!("running svn info: {e}"))?;
    if !out.status.success() {
        return Err(format!(
            "svn info {} exited {}: {}",
            path.display(),
            out.status,
            String::from_utf8_lossy(&out.stderr).trim()
        ));
    }
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .find_map(|line| line.strip_prefix("Revision: "))
        .map(|n| format!("r{}", n.trim()))
        .ok_or_else(|| format!("svn info {} printed no Revision line", path.display()))
}

/// The name of every row set, in the order this module derives them.
pub const FILES: &[&str] = &[
    "provide-sections.txt",
    "hierarchy-edges.txt",
    "class-set.txt",
    "class-methods.txt",
    "directive-options.txt",
];

/// The comment beside `provide.xml`'s hierarchy list that is `ArgUtil`'s only
/// citation, since the books document the class nowhere else.
const ARGUTIL_COMMENT: &str = "<!-- commented ArgUtil (deprecated, won't document) -->";

/// Derives every row set. `oodocs` is the checkout root (the directory holding
/// `rexxref/`), `interpreter` is this repository's `interpreter/`.
pub fn derive(
    oodocs: &std::path::Path,
    interpreter: &std::path::Path,
    revision: &str,
) -> Result<Vec<(&'static str, RowSet)>, String> {
    let book_dir = oodocs.join(REXXREF);
    let read = |name: &str| -> Result<String, String> {
        let path = book_dir.join(name);
        std::fs::read_to_string(&path).map_err(|e| format!("cannot read {}: {e}", path.display()))
    };
    let stamp = stamp_line(revision);

    let provide_xml = read("provide.xml")?;
    let concepts = provide::concept_sections(&provide_xml);

    let blanked = xml::blank_comments(&provide_xml);
    let edges = hierarchy::edges_from_members(
        &hierarchy::chi_members(&blanked),
        &[classes::INSTANCE_ENTRY_CLASS],
    );

    let argutil_line = argutil_citation_line(&provide_xml)?;
    let argutil_citation = format!("provide.xml:{argutil_line}");

    let books: Vec<classes::Book> = CLASS_BOOKS
        .iter()
        .map(|name| Ok(classes::Book::new(name, &read(name)?)))
        .collect::<Result<_, String>>()?;
    let class_rows = classes::class_rows(&books, &argutil_citation);

    let (includes, unincluded) = classmethod_files(&book_dir, &books)?;
    let method_rows = classes::method_rows(&books, &includes, &class_rows);
    let unreferenced = classes::unreferenced_sections(&books, &method_rows);

    let dire_xml = read("dire.xml")?;
    let parser_path = interpreter.join("parser/DirectiveParser.cpp");
    let parser_source = std::fs::read_to_string(&parser_path)
        .map_err(|e| format!("cannot read {}: {e}", parser_path.display()))?;
    let option_rows = directives::option_rows(
        &directives::documented(&dire_xml),
        &directives::parser_arms(&parser_source),
    );

    Ok(vec![
        (
            "provide-sections.txt",
            RowSet {
                header: provide::header(&stamp),
                rows: provide::rows(&concepts),
            },
        ),
        (
            "hierarchy-edges.txt",
            RowSet {
                header: hierarchy::header(&stamp, &[classes::INSTANCE_ENTRY_CLASS]),
                rows: hierarchy::rows(&edges),
            },
        ),
        (
            "class-set.txt",
            RowSet {
                header: classes::class_set_header(&stamp, &argutil_citation),
                rows: classes::class_set_rows(&class_rows),
            },
        ),
        (
            "class-methods.txt",
            RowSet {
                header: method_header(&stamp, &unincluded, &unreferenced),
                rows: classes::method_set_rows(&method_rows),
            },
        ),
        (
            "directive-options.txt",
            RowSet {
                header: directives::header(&stamp),
                rows: directives::rows(&option_rows),
            },
        ),
    ])
}

fn argutil_citation_line(provide_xml: &str) -> Result<usize, String> {
    let hits: Vec<usize> = provide_xml
        .lines()
        .enumerate()
        .filter(|(_, line)| line.contains(ARGUTIL_COMMENT))
        .map(|(ix, _)| ix + 1)
        .collect();
    match hits.as_slice() {
        [only] => Ok(*only),
        [] => Err(format!("provide.xml no longer carries {ARGUTIL_COMMENT}")),
        many => Err(format!(
            "provide.xml carries {ARGUTIL_COMMENT} at {many:?}; the citation is ambiguous"
        )),
    }
}

/// The `*classmethods.xml` files the class tables include, and the ones in the
/// same directory that nothing includes.
///
/// The second list is derived rather than remembered, so it is policed by the
/// both-directions check: `objectclassmethods.xml` is included by nothing
/// today, and if that changes the header moves and the check reddens. An
/// extractor that globbed the directory instead of following the includes
/// would credit `Object` with a method its own class table comments out.
fn classmethod_files(
    book_dir: &std::path::Path,
    books: &[classes::Book],
) -> Result<(std::collections::BTreeMap<String, String>, Vec<String>), String> {
    let mut wanted: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for book in books {
        for (href, _) in xml::xincludes(&book.text) {
            if href.ends_with("classmethods.xml") {
                wanted.insert(href);
            }
        }
    }
    let mut includes = std::collections::BTreeMap::new();
    for name in &wanted {
        let path = book_dir.join(name);
        let text = std::fs::read_to_string(&path)
            .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
        includes.insert(name.clone(), xml::blank_comments(&text));
    }
    let mut unincluded = Vec::new();
    let entries = std::fs::read_dir(book_dir)
        .map_err(|e| format!("cannot read {}: {e}", book_dir.display()))?;
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.ends_with("classmethods.xml") && !wanted.contains(&name) {
            unincluded.push(name);
        }
    }
    unincluded.sort();
    Ok((includes, unincluded))
}

fn method_header(
    stamp: &str,
    unincluded: &[String],
    unreferenced: &[(String, String, usize)],
) -> Vec<String> {
    let mut header = classes::class_methods_header(stamp);
    let mut extra = vec![
        String::new(),
        "Two derived lists, here rather than in prose because the".into(),
        "both-directions check polices a header exactly as it polices a row.".into(),
        String::new(),
        "(1) *classmethods.xml files in rexxref/en-US that no class table".into(),
        "includes. An extractor that globbed the directory instead of following".into(),
        "the xi:includes would key their members to no class, or to the wrong".into(),
        "one:".into(),
    ];
    if unincluded.is_empty() {
        extra.push("    (none)".into());
    } else {
        extra.extend(unincluded.iter().map(|name| format!("    {name}")));
    }
    extra.extend([
        String::new(),
        "(2) mth* sections in the four books that no class table names, and so".into(),
        "produce no row. These are the named exceptions the method-row".into(),
        "denominator allows -- every other mth* section is a row:".into(),
    ]);
    if unreferenced.is_empty() {
        extra.push("    (none)".into());
    } else {
        extra.extend(
            unreferenced
                .iter()
                .map(|(id, file, line)| format!("    {id}  {file}:{line}")),
        );
    }
    extra.push(String::new());
    let stamp_at = header.len() - 1;
    header.splice(stamp_at..stamp_at, extra);
    header
}
