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

//! The structural checks gate table C runs: its row sets checked against
//! each other and against the probe directories, a row's probe run on both
//! sides, and the oracle's answer checked for the shape its row asks for.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use crate::Concept;
use crate::gate_tables::{
    Descriptors, Structural, compare_raw, excerpt, is_loud, refused_construct, run_gate_probe,
    stdout_lines,
};
use crate::support;
use crate::support::oracle::{CppOutcome, did_not_finish, wrapped_exit_code};

use super::probes::{
    DOCUMENTED_EDGE_MARKER, ENTRY_MARKER, class_probe_class_questions, class_probe_entry_questions,
};
use super::rows::{ClassRow, Construction, Edge, Section};

/// How many lines a class wiring row's probe prints on the oracle.
pub(crate) fn class_probe_shape(name: &str, entry: &str) -> OracleShape {
    let entry_questions = class_probe_entry_questions(name).len();
    match entry {
        "class" => OracleShape::Exactly(entry_questions + class_probe_class_questions(name).len()),
        _ => OracleShape::Exactly(entry_questions),
    }
}

/// The `entry` values [`class_probe_shape`] and [`check_edge_endpoints`] know
/// how to read.
const ENTRY_KINDS: &[&str] = &["class", "instance"];

/// Every concept row's committed line count is one a probe can be evidence
/// for.
pub(crate) fn check_concept_line_counts(
    concepts: &[(&Section, &'static Concept)],
    structural: &mut Vec<Structural>,
) {
    for (section, concept) in concepts {
        if concept.oracle_lines == 0 {
            structural.push(Structural {
                subject: format!("CONCEPTS arm {}", section.id),
                detail: "its `oracle_lines` is zero, and a bound of zero lines is met by a \
                         program that produced nothing at all. A concept probe exercises a \
                         mechanism and prints what it observed; one that prints nothing is \
                         not evidence, and the row would read `agree` on both interpreters \
                         failing alike"
                    .to_string(),
            });
        }
    }
}

/// Every class row's `entry` column is a value this file knows how to read.
pub(crate) fn check_entry_kinds(classes: &[ClassRow], structural: &mut Vec<Structural>) {
    for row in classes {
        if !ENTRY_KINDS.contains(&row.entry.as_str()) {
            structural.push(Structural {
                subject: format!("class-set.txt row {}", row.name),
                detail: format!(
                    "its `entry` column reads {:?}, which is not one of {ENTRY_KINDS:?}. \
                     Both the bound on its probe's output and its place in the hierarchy \
                     rows' referential check are decided by that column, so an \
                     unrecognised value exempts the row from each rather than reddening",
                    row.entry
                ),
            });
        }
    }
}

/// Records a structural failure when the oracle does not resolve the row's
/// name to an `.environment` entry at all.
pub(crate) fn check_entry_present(
    probe: &str,
    name: &str,
    oracle_stdout: &[u8],
    structural: &mut Vec<Structural>,
) -> bool {
    let text = String::from_utf8_lossy(oracle_stdout);
    let answer = text
        .lines()
        .find_map(|line| line.strip_prefix(ENTRY_MARKER));
    let unresolved = format!(".{}", name.to_ascii_uppercase());
    if let Some(answer) = answer
        && answer != unresolved
    {
        return true;
    }
    structural.push(Structural {
        subject: probe.to_string(),
        detail: format!(
            "the oracle does not resolve .{name} to an `.environment` entry: its \
             `{ENTRY_MARKER}` answer is {answer:?}, which is how an unresolved \
             environment symbol renders. The row claims this name is an entry; where the \
             shipped build has no such name, both interpreters raise alike on every \
             question below and the row would read `agree` over a class that does not \
             exist"
        ),
    });
    false
}

/// What the oracle's `stdout` must look like for a row's probe to have asked
/// the question the row is about.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum OracleShape {
    /// Exactly this many lines, because every `say` the program holds is
    /// reached.
    Exactly(usize),
    /// Every line or none at all, and nothing between: the shape of a program
    /// that constructs something first and asks its questions of the result,
    /// where the construction either answers or raises.
    AllOrNothing(usize),
}

impl OracleShape {
    /// Whether `lines` is a count this shape admits.
    fn admits(self, lines: usize) -> bool {
        match self {
            OracleShape::Exactly(want) => lines == want,
            OracleShape::AllOrNothing(want) => lines == want || lines == 0,
        }
    }

    /// What this shape demands, for the failure message.
    fn describe(self) -> String {
        match self {
            OracleShape::Exactly(want) => format!("exactly {want}"),
            OracleShape::AllOrNothing(want) => format!("{want} or none"),
        }
    }
}

/// Records a structural failure when the oracle's output is not a shape the
/// probe could have produced while asking its row's question.
pub(crate) fn check_oracle_shape(
    probe: &str,
    subject: &str,
    shape: OracleShape,
    oracle_stdout: &[u8],
    structural: &mut Vec<Structural>,
) -> bool {
    let lines = stdout_lines(oracle_stdout).len();
    if shape.admits(lines) {
        return true;
    }
    structural.push(Structural {
        subject: probe.to_string(),
        detail: format!(
            "the oracle answered {lines} line(s) where this row's probe asks for {}. The \
             row for {subject} therefore has no answer to compare -- and two sides that \
             both fail to answer agree on all three descriptors, so without this the row \
             would read `agree` for a question nobody asked",
            shape.describe()
        ),
    });
    false
}

/// Records a structural failure when the oracle does not confirm the edge the
/// row claims.
pub(crate) fn check_documented_edge(
    probe: &str,
    subject: &str,
    oracle_stdout: &[u8],
    structural: &mut Vec<Structural>,
) {
    let text = String::from_utf8_lossy(oracle_stdout);
    let answer = text
        .lines()
        .find_map(|line| line.strip_prefix(DOCUMENTED_EDGE_MARKER));
    if answer == Some("1") {
        return;
    }
    structural.push(Structural {
        subject: probe.to_string(),
        detail: format!(
            "the oracle does not confirm the documented edge {subject}: its \
             `{DOCUMENTED_EDGE_MARKER}` answer is {answer:?}, not \"1\". The row claims the \
             parent is present in the child's ~superClasses; where the shipped interpreter \
             says otherwise, both sides can answer `0` alike and the row would read `agree` \
             while asserting nothing about either the book or the build"
        ),
    });
}

/// A probe's two runs -- this crate on both engines, and the oracle -- or the
/// structural failure that stopped them.
pub(crate) struct Ran {
    pub(crate) crate_exit: i32,
    pub(crate) crate_stdout: Vec<u8>,
    pub(crate) crate_stderr: Vec<u8>,
    pub(crate) crate_loud: bool,
    pub(crate) crate_refused: Option<String>,
    pub(crate) oracle_exit: i32,
    pub(crate) oracle_stdout: Vec<u8>,
    pub(crate) oracle_stderr: Vec<u8>,
    pub(crate) differs: Descriptors,
}

/// Runs one probe on both crate engines and on the oracle, or records why it
/// could not be run.
pub(crate) fn run_probe(
    oracle: &support::oracle::Oracle,
    corpus: &Path,
    probe: &str,
    subject: &str,
    on_disk: &BTreeSet<String>,
    structural: &mut Vec<Structural>,
) -> Option<Ran> {
    let abs = match fs::canonicalize(corpus.join(probe)) {
        Ok(abs) => abs,
        Err(error) => {
            if on_disk.contains(probe) {
                structural.push(Structural {
                    subject: probe.to_string(),
                    detail: format!(
                        "the probe is listed in the directory but cannot be resolved: \
                         {error}. The row for {subject} therefore has no program to run, \
                         which is structural -- it is never a row the table drops"
                    ),
                });
            }
            return None;
        }
    };

    // A committed construction expression may locate a sibling file through
    // its own running path (StreamSupplier's fixture does), which only
    // answers correctly when the probe runs at its own committed location.
    // Staging it into a copy elsewhere -- `method_bodies.rs` already does
    // this for a different table -- would break that silently, at whichever
    // row happens to depend on it, rather than here.
    let corpus_root = fs::canonicalize(corpus)
        .unwrap_or_else(|e| panic!("cannot canonicalize {}: {e}", corpus.display()));
    if !abs.starts_with(&corpus_root) {
        structural.push(Structural {
            subject: probe.to_string(),
            detail: format!(
                "runs from {}, outside {}, so a construction expression resolving a sibling \
                 path from its own running location would resolve the wrong one",
                abs.display(),
                corpus_root.display(),
            ),
        });
        return None;
    }

    let crate_side = run_gate_probe(&abs);
    let cpp: CppOutcome = oracle.run(&abs);
    if did_not_finish(&cpp) {
        structural.push(Structural {
            subject: probe.to_string(),
            detail: format!(
                "the oracle did not finish: {:?}. Whatever it left on its descriptors is \
                 where it was interrupted, not an answer to compare",
                cpp.termination
            ),
        });
        return None;
    }

    let differs = compare_raw(&crate_side, &cpp);
    Some(Ran {
        crate_exit: wrapped_exit_code(crate_side.exit_code),
        crate_loud: is_loud(&crate_side),
        crate_refused: refused_construct(&crate_side.stderr),
        crate_stdout: crate_side.stdout,
        crate_stderr: crate_side.stderr,
        oracle_exit: cpp.expect_exit_code(),
        oracle_stdout: cpp.stdout,
        oracle_stderr: cpp.stderr,
        differs,
    })
}

/// Compares a probe family's derived path set against the directory's own
/// listing, in both directions, and returns the listing.
pub(crate) fn probe_set(
    corpus: &Path,
    subdir: &str,
    expected: &BTreeSet<String>,
    structural: &mut Vec<Structural>,
) -> BTreeSet<String> {
    let dir = corpus.join(subdir);
    let mut on_disk: BTreeSet<String> = BTreeSet::new();
    for entry in fs::read_dir(&dir).unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()))
    {
        let name = entry
            .unwrap_or_else(|e| panic!("cannot read an entry of {}: {e}", dir.display()))
            .file_name();
        match name.clone().into_string() {
            Ok(name) if name.ends_with(".rex") => {
                on_disk.insert(format!("{subdir}/{name}"));
            }
            Ok(_) => {}
            // A name no row can ever derive, because every derived path is
            // built from ASCII. Dropping it would make the file invisible to
            // the orphan half of the check below -- present in the directory,
            // named by nothing, and never reported.
            Err(_) => structural.push(Structural {
                subject: format!("{subdir}/{}", name.to_string_lossy()),
                detail: "a file in the probe directory whose name is not valid UTF-8. No \
                         row derives such a path, so nothing runs it and the orphan check \
                         cannot name it"
                    .to_string(),
            }),
        }
    }
    for missing in expected.difference(&on_disk) {
        structural.push(Structural {
            subject: missing.clone(),
            detail: "a row has no probe program. A row without one is structural and is \
                     never skipped: the table reddens on the missing program rather than \
                     shrinking to the rows that still have one"
                .to_string(),
        });
    }
    for orphan in on_disk.difference(expected) {
        structural.push(Structural {
            subject: orphan.clone(),
            detail: "a probe program no row names, so nothing runs it".to_string(),
        });
    }
    on_disk
}

/// Compares one derived probe program against its committed file, in both
/// directions.
pub(crate) fn check_probe_text(
    corpus: &Path,
    probe: &str,
    derived: &str,
    on_disk: &BTreeSet<String>,
    structural: &mut Vec<Structural>,
) {
    let path = corpus.join(probe);
    let committed = match fs::read_to_string(&path) {
        Ok(committed) => committed,
        Err(error) => {
            // **Returning quietly here is only right for one of the ways this
            // read fails, and the others leave the file in place.**
            // `read_to_string` fails on invalid UTF-8 and on a permission
            // error as well as on a missing file, and in both of those the
            // program still exists, still runs, and still produces a verdict
            // -- so a silent return turns off the only thing standing between
            // a row's verdict and a program about a different subject.
            if on_disk.contains(probe) {
                structural.push(Structural {
                    subject: probe.to_string(),
                    detail: format!(
                        "the probe is listed in the directory but its bytes cannot be \
                         read as text: {error}. The derivation check is what keeps a \
                         probe from asking about a subject other than its row's, and it \
                         cannot run on a file it cannot read -- so this is structural \
                         rather than a check that quietly does not happen"
                    ),
                });
            }
            return;
        }
    };
    if committed != derived {
        // The **first differing line**, not the head of either file. Every
        // program in these families opens with a comment naming its own row,
        // so a bounded excerpt from the top is the same text on both sides
        // and shows a reader nothing at all.
        let (at, left, right) = first_difference(derived, &committed);
        structural.push(Structural {
            subject: probe.to_string(),
            detail: format!(
                "the committed probe is not what its row derives. The derivation in \
                 gate_table_c.rs is the definition of this program, so a difference means \
                 the file asks something other than what its row says it asks -- which a \
                 comparison against the oracle cannot see, because a probe asking about \
                 the wrong subject agrees for the wrong reason.\n    line {at} derived:   \
                 {}\n    line {at} committed: {}",
                excerpt(left.as_bytes()),
                excerpt(right.as_bytes()),
            ),
        });
    }
}

/// The one-based number of the first line at which two texts differ, and that
/// line from each side, **terminator included**. A line present on one side
/// only is reported as the empty string on the other.
fn first_difference(left: &str, right: &str) -> (usize, String, String) {
    let mut lefts = left.split_inclusive('\n');
    let mut rights = right.split_inclusive('\n');
    let mut at = 0usize;
    loop {
        at += 1;
        match (lefts.next(), rights.next()) {
            (None, None) => return (at, String::new(), String::new()),
            (a, b) if a != b => {
                return (
                    at,
                    a.unwrap_or_default().to_string(),
                    b.unwrap_or_default().to_string(),
                );
            }
            _ => {}
        }
    }
}

/// The row-set text a derived probe copies into a Rexx block comment, checked
/// for the sequences that would end the comment early or split the line.
pub(crate) fn check_interpolated_text(classes: &[ClassRow], structural: &mut Vec<Structural>) {
    for row in classes {
        if row.reason.contains("*/") {
            structural.push(Structural {
                subject: format!("class-set.txt row {}", row.name),
                detail: format!(
                    "its `reason` contains `*/`, which ends the Rexx block comment the \
                     derived probe writes it into: {:?}",
                    row.reason
                ),
            });
        }
        let Construction::Constructs { program, directive } = &row.construction else {
            continue;
        };
        for (field, text) in [
            ("construction", Some(program)),
            ("directives", directive.as_ref()),
        ] {
            let Some(text) = text else { continue };
            if text.contains("*/") || text.contains('\n') || text.trim() != text.as_str() {
                structural.push(Structural {
                    subject: format!("class-set.txt row {}", row.name),
                    detail: format!(
                        "its `{field}` is not a single trimmed line free of `*/`, and the \
                         derived probe writes it into both a Rexx block comment and its own \
                         line: {text:?}"
                    ),
                });
            }
        }
    }
}

/// Every hierarchy edge names, at both ends, a class the class row set carries
/// as a class object.
pub(crate) fn check_edge_endpoints(
    classes: &[ClassRow],
    edges: &[Edge],
    structural: &mut Vec<Structural>,
) {
    let class_objects: BTreeSet<&str> = classes
        .iter()
        .filter(|row| row.entry == "class")
        .map(|row| row.name.as_str())
        .collect();
    for edge in edges {
        for (end, name) in [("child", &edge.child), ("parent", &edge.parent)] {
            if !class_objects.contains(name.as_str()) {
                structural.push(Structural {
                    subject: format!("{} <- {}", edge.child, edge.parent),
                    detail: format!(
                        "its {end}, {name}, is not a `class` row of class-set.txt, so no \
                         wiring row asks whether this build has it -- and an edge row \
                         whose endpoints neither interpreter can answer for reads `agree` \
                         on the two of them raising alike"
                    ),
                });
            }
        }
    }
}

/// The `ArgUtil` assertion, carried here as a wiring row.
pub(crate) fn argutil_assertion(
    classes: &[ClassRow],
    edges: &[Edge],
    structural: &mut Vec<Structural>,
) -> String {
    const NAME: &str = "ArgUtil";
    let in_class_set = classes.iter().any(|row| row.name == NAME);
    let edge: Vec<String> = edges
        .iter()
        .filter(|edge| edge.child == NAME || edge.parent == NAME)
        .map(|edge| {
            format!(
                "{} <- {} (provide.xml:{})",
                edge.child, edge.parent, edge.line
            )
        })
        .collect();
    if !in_class_set {
        structural.push(Structural {
            subject: "the ArgUtil assertion".to_string(),
            detail: "`ArgUtil` is not in class-set.txt. It is an `.environment` class the \
                     books document nowhere, carried by the row set with the XML comment at \
                     provide.xml:838 as its citation; without it the class denominator is \
                     the documented set rather than the reachable one"
                .to_string(),
        });
    }
    if !edge.is_empty() {
        structural.push(Structural {
            subject: "the ArgUtil assertion".to_string(),
            detail: format!(
                "hierarchy-edges.txt carries an edge naming `ArgUtil`: {}. `provide.xml` \
                 comments that member out, so an edge here means the derivation read the \
                 `<member>`s without blanking XML comments first. **No verdict row can see \
                 this**: the edge it emits is one the oracle confirms, so the table would \
                 be green over a wrong member set",
                edge.join(", ")
            ),
        });
    }
    match (in_class_set, edge.is_empty()) {
        (true, true) => "held: ArgUtil is a class row and emits no hierarchy edge".to_string(),
        _ => "FAILED -- see the structural failures above".to_string(),
    }
}
