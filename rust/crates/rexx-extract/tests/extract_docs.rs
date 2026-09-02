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

//! The both-directions check over `corpus/docs/`, and the D56 revision stamp.
//!
//! **Both directions, and that is the point.** A row that stops being derived
//! is as red as one that appears, which is the property
//! `corpus/keyword-exempt.txt` already has and the reason these row sets are
//! committed rather than derived at gate time.
//!
//! **What this cannot see**, said here rather than left implied: it fires when
//! one side moves and not when both move together in one commit -- a
//! regeneration. That is a diff for a human, deliberately, and it is why the
//! committed row sets are reviewed as artifacts rather than waved through as
//! generated output.
//!
//! **`oodocs/` is git-ignored and none of CI's five platforms has it**, so
//! every check here fires on one developer machine per gate rather than
//! continuously. It **fails** when `oodocs/` is absent unless the run is
//! explicitly marked docs-less with `REXX_DOCS_LESS=1` (D56), because a check
//! that silently skips is a failure mode this project has shipped.

use rexx_extract::docs::{self, classes, hierarchy};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// The environment variable that marks a run as knowingly docs-less.
const DOCS_LESS: &str = "REXX_DOCS_LESS";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn corpus_docs() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus/docs")
}

/// The `oodocs/` checkout, or `None` when it is absent **and** the run says so.
///
/// Absent without the marker is a failure, not a skip.
fn oodocs() -> Option<PathBuf> {
    let root = repo_root().join("oodocs");
    if root.join(docs::REXXREF).is_dir() {
        return Some(root);
    }
    assert!(
        std::env::var_os(DOCS_LESS).is_some(),
        "{} is absent. It is a git-ignored SVN working copy, not checked-in test data:\n  \
         svn checkout https://svn.code.sf.net/p/oorexx/code-0/docs/trunk/rexxref oodocs/rexxref\n\
         Set {DOCS_LESS}=1 to run the rest of the suite without it -- deliberately, and having \
         read that the row sets under corpus/docs/ are then unchecked",
        root.display()
    );
    None
}

fn derived() -> Option<Vec<(&'static str, docs::RowSet)>> {
    let root = oodocs()?;
    let revision = docs::svn_revision(&root.join("rexxref")).expect("svn info oodocs/rexxref");
    Some(
        docs::derive(&root, &repo_root().join("interpreter"), &revision)
            .unwrap_or_else(|e| panic!("{e}")),
    )
}

fn committed(name: &str) -> String {
    let path = corpus_docs().join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}

fn rows_of(text: &str) -> Vec<&str> {
    text.lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
        .collect()
}

/// The check the whole arrangement exists for.
#[test]
fn every_row_set_is_exactly_what_its_extractor_derives_today() {
    let Some(files) = derived() else { return };
    for (name, set) in &files {
        let rendered = set.render();
        let committed = committed(name);
        if rendered == committed {
            continue;
        }
        let derived_rows: BTreeSet<&str> = rows_of(&rendered).into_iter().collect();
        let committed_rows: BTreeSet<&str> = rows_of(&committed).into_iter().collect();
        let appeared: Vec<&&str> = derived_rows.difference(&committed_rows).collect();
        let vanished: Vec<&&str> = committed_rows.difference(&derived_rows).collect();
        panic!(
            "{name} is not what the extractor derives.\n\
             rows derived and not committed ({}): {appeared:#?}\n\
             rows committed and no longer derived ({}): {vanished:#?}\n\
             If both lists are empty the header moved, which is policed the same way.\n\
             Regenerate with:\n  cargo run -p rexx-extract --bin rexx-extract-docs -- \
             --oodocs ../oodocs --interpreter ../interpreter --out corpus/docs\n\
             and read the diff -- a row that stops being derived is as red as one that appears.",
            appeared.len(),
            vanished.len()
        );
    }
}

/// D56's stamp. `oodocs/` is git-ignored, so nothing in the checkout records
/// what it held; the stamp and `svn info` are the only pair that can say
/// whether a committed row set is stale.
#[test]
fn every_row_set_is_stamped_with_the_revision_the_checkout_is_at() {
    let Some(root) = oodocs() else { return };
    let revision = docs::svn_revision(&root.join("rexxref")).expect("svn info oodocs/rexxref");
    let expected = format!("# {}", docs::stamp_line(&revision));
    for name in docs::FILES {
        let text = committed(name);
        let stamps: Vec<&str> = text.lines().filter(|l| l.contains("derived-at:")).collect();
        assert_eq!(
            stamps,
            [expected.as_str()],
            "{name}'s revision stamp is not the checkout's. `svn info oodocs/rexxref` reads \
             {revision}; re-derive and re-commit rather than editing the stamp"
        );
    }
}

/// Nothing sits in `corpus/docs/` that no extractor writes and no check
/// polices.
#[test]
fn the_row_set_directory_holds_exactly_the_row_sets() {
    let mut present: Vec<String> = std::fs::read_dir(corpus_docs())
        .expect("corpus/docs")
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    present.sort();
    let mut expected: Vec<String> = docs::FILES.iter().map(|n| (*n).to_string()).collect();
    expected.sort();
    assert_eq!(present, expected);
}

/// Rule 1 of the hierarchy derivation, seen firing against the real book
/// rather than asserted to work.
///
/// The other two rules are witnessed by running the edge set against the
/// oracle, because their edges fail there when present. This one is not:
/// `.ArgUtil~superClasses` is exactly `The Object class`, so an extractor that
/// skips the comment blanking emits an extra edge the oracle confirms, and the
/// end-to-end run stays at zero failures over a wrong member set.
#[test]
fn skipping_the_comment_blanking_fires_the_argutil_assertion() {
    let Some(root) = oodocs() else { return };
    let path = root.join(docs::REXXREF).join("provide.xml");
    let provide = std::fs::read_to_string(&path).expect("provide.xml");

    let blanked = docs::xml::blank_comments(&provide);
    let clean = hierarchy::edges_from_members(
        &hierarchy::chi_members(&blanked),
        &[classes::INSTANCE_ENTRY_CLASS],
    );
    assert!(!clean.iter().any(|e| e.child == "ArgUtil"));

    let raw = std::panic::catch_unwind(|| {
        hierarchy::edges_from_members(
            &hierarchy::chi_members(&provide),
            &[classes::INSTANCE_ENTRY_CLASS],
        )
    })
    .expect_err("reading the chi members without blanking comments must fire the assertion");
    // A `panic!`/`assert!` message with no interpolation is a `&'static str`
    // payload and one with interpolation is a `String`; reading only the
    // second is how this test read "<not a string>" for the panic it was
    // looking at.
    let message = raw
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| raw.downcast_ref::<&str>().copied())
        .unwrap_or("<panic payload is neither a str nor a String>");
    assert!(
        message.contains("ArgUtil is in the derived edge set"),
        "the derivation panicked, but not on the ArgUtil assertion: {message}"
    );
}

/// A committed construction program is only evidence if something runs it,
/// and what runs it is the instance-arm probe its class's method rows share.
///
/// Measured: `Buffer`, `Singleton`, `Validate` and `ArgUtil` have no
/// instance-arm row, so a program committed for one of them would make the
/// class `covered` with nothing on either side ever constructing.
#[test]
fn every_committed_construction_program_has_an_instance_arm_to_run_it() {
    let methods: &'static str = Box::leak(committed("class-methods.txt").into_boxed_str());
    let instance_arms: BTreeSet<String> = rows_of(methods)
        .into_iter()
        .filter_map(|row| {
            let fields: Vec<&str> = row.split('\t').collect();
            (fields.get(2) == Some(&"instance")).then(|| fields[0].to_ascii_uppercase())
        })
        .collect();
    for (class, program) in classes::CONSTRUCTION_PROGRAMS {
        assert!(
            instance_arms.contains(*class),
            "{class} has the committed construction program {program} and no instance-arm \
             method row, so no probe ever runs it"
        );
    }
}

/// The two class row sets have to agree about every class, or a method row's
/// status says one thing and its class's row another.
#[test]
fn every_method_rows_class_and_status_come_from_the_class_set() {
    let class_set: BTreeMap<&str, &str> = {
        let text: &'static str = Box::leak(committed("class-set.txt").into_boxed_str());
        rows_of(text)
            .into_iter()
            .map(|row| {
                let mut f = row.split('\t');
                (f.next().unwrap_or_default(), f.nth(3).unwrap_or_default())
            })
            .collect()
    };
    let methods: &'static str = Box::leak(committed("class-methods.txt").into_boxed_str());
    for row in rows_of(methods) {
        let mut f = row.split('\t');
        let class = f.next().unwrap_or_default();
        let status = f.nth(2).unwrap_or_default();
        let expected = class_set.get(class).unwrap_or_else(|| {
            panic!("class-methods.txt has rows for {class}, which class-set.txt does not carry")
        });
        assert_eq!(
            &status, expected,
            "{class}'s status disagrees between the two row sets"
        );
    }
}

/// Every documented edge names classes the class set carries, and the classes
/// carrying no edge are the ones the spec records as the disagreement between
/// the hierarchy list and the `cls*` sections.
#[test]
fn the_edge_set_and_the_class_set_name_the_same_classes() {
    let edges: &'static str = Box::leak(committed("hierarchy-edges.txt").into_boxed_str());
    let classes_text: &'static str = Box::leak(committed("class-set.txt").into_boxed_str());
    let named: BTreeSet<&str> = rows_of(classes_text)
        .into_iter()
        .filter_map(|r| r.split('\t').next())
        .collect();
    let mut children: BTreeSet<&str> = BTreeSet::new();
    for row in rows_of(edges) {
        let mut f = row.split('\t');
        let child = f.next().unwrap_or_default();
        let parent = f.next().unwrap_or_default();
        assert!(
            named.contains(child),
            "the edge set names {child}, which is not in the class set"
        );
        assert!(
            named.contains(parent),
            "the edge set names {parent}, which is not in the class set"
        );
        children.insert(child);
    }
    let no_edge: Vec<&str> = named.difference(&children).copied().collect();
    assert_eq!(
        no_edge,
        [
            // Documented nowhere, so not in the hierarchy list either; its
            // member is commented out beside it.
            "ArgUtil",
            // Documented in a cls* section and absent from the hierarchy list.
            // The spec records this trio as a disagreement between two parts
            // of the same book, not between the book and the image.
            "EventSemaphore",
            "MutexSemaphore",
            // The root: rule 2.
            "Object",
            // Excluded from the class set's class rows: rule 3.
            "RexxInfo",
            "Singleton",
        ],
        "the classes with no documented edge are not the ones the spec records"
    );
}

/// The two rows table C exists for. Neither is a directive keyword, so table D
/// cannot see either, and the superseded gate had only table D's shape.
#[test]
fn the_concept_row_set_carries_the_two_sections_the_gate_was_written_for() {
    let text: &'static str = Box::leak(committed("provide-sections.txt").into_boxed_str());
    let ids: BTreeSet<&str> = rows_of(text)
        .into_iter()
        .filter_map(|r| r.split('\t').next())
        .collect();
    for id in ["unkno", "reqstr", "xmeths", "chi"] {
        assert!(ids.contains(id), "provide-sections.txt has no {id} row");
    }
}

/// Every method row's `origin` lands on a line that names the row's `section`.
///
/// The header promises `origin` is where the name came from, and for most rows
/// that is the `<section id="mth...">` line while for a row whose class-table
/// member overrides the displayed name it is the `<member><xref linkend="mth...">`
/// line. Both spellings carry the id, so one assertion covers both -- and it is
/// the assertion that catches an offset error, which is how a member's line
/// inside a section slice can come out counted from the section instead of from
/// the file.
#[test]
fn every_method_rows_origin_line_names_its_section() {
    let Some(root) = oodocs() else { return };
    let book_dir = root.join(docs::REXXREF);
    let mut files: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let text: &'static str = Box::leak(committed("class-methods.txt").into_boxed_str());
    for row in rows_of(text) {
        let mut f = row.split('\t');
        let section = f.nth(4).unwrap_or_default();
        let origin = f.next().unwrap_or_default();
        let (file, line) = origin
            .rsplit_once(':')
            .unwrap_or_else(|| panic!("{origin} is not a file:line"));
        let line: usize = line
            .parse()
            .unwrap_or_else(|e| panic!("{origin} has no line number: {e}"));
        let lines = files.entry(file.to_string()).or_insert_with(|| {
            let path = book_dir.join(file);
            std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
                .lines()
                .map(str::to_string)
                .collect()
        });
        let found = lines
            .get(line - 1)
            .unwrap_or_else(|| panic!("{file} has no line {line}"));
        assert!(
            found.contains(section),
            "the row citing {origin} names {section}, and that line reads {found:?}"
        );
    }
}
