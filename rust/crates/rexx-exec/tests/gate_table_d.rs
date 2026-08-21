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

//! Gate table D: the directive and option surface.
//!
//! One row per line of `corpus/docs/directive-options.txt` -- the union of
//! `dire.xml` and `interpreter/parser/DirectiveParser.cpp`, derived and
//! committed with its extractor. One committed probe program per row under
//! `corpus/gate-tables/directives/`. One verdict per row, and the verdict
//! comes from **running** that program: this crate on both engines, in
//! process, against the C++ oracle in a subprocess, compared on all three
//! descriptors with `stderr` raw.
//!
//! # The table types no expected bytes
//!
//! There is no recorded oracle answer anywhere in this file or beside it. Each
//! row's oracle column is produced by launching the oracle on the run that
//! reports it, so a row cannot pass by agreeing with a recording of itself,
//! and the table cannot go stale against a rebuilt interpreter the way a baked
//! transcript can.
//!
//! # These probes are not corpus programs
//!
//! They live under `corpus/gate-tables/`, not in a phase subset file. Most of
//! them diverge, and `corpus/phase-5a.txt` means "agrees with the oracle"; a
//! probe moves there in the task that makes its row agree. The corpus
//! differential is unaffected by this subtree existing: every copy of
//! `phase_subset_files_on_disk` reads `corpus/` with a **non-recursive**
//! `read_dir` filtered to `phase-*.txt`, either property alone being enough,
//! and `corpus.rs`'s `the_differential_reads_every_phase_subset_file` is the
//! standing assertion over that -- so this is checked by a test that already
//! exists rather than by a sentence here.
//!
//! # What this table cannot see
//!
//! A keyword neither `dire.xml` nor `DirectiveParser.cpp` names is outside the
//! denominator, because the row set is their union and nothing else.
//!
//! And a row's verdict is about a directive **keyword**. A probe exercises the
//! keyword at its position; what the directive then does with it is only
//! visible here to the extent that the program's three descriptors show it. A
//! keyword this crate accepts and ignores, where the oracle also produces no
//! observable difference, reads `agree` -- the discriminator for that is a
//! reflection query, which is table C's half.
//!
//! # The owning-phase column
//!
//! [`owning_phase`] is a committed assignment, one arm per directive keyword,
//! and it is what decides whether a verdict mismatch is an exit status. It is
//! read by a human out of a diff; nothing checks that a row is filed under the
//! right phase, and a row filed under the wrong one escapes gating. Each arm
//! names the authority it comes from.

mod gate_tables;
mod support;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use gate_tables::orx::{self, CORE_CLASSES, STREAM_CLASSES};
use gate_tables::{
    Descriptors, Report, Structural, Verdict, assert_no_structural_failures, compare_raw, excerpt,
    is_loud, refused_construct, run_on_both_engines, verdict, verdict_is_gated,
};
use support::oracle::{did_not_finish, wrapped_exit_code};

/// The committed row set, derived from the two authorities by
/// `rexx-extract-docs` and re-derived in both directions by
/// `rexx-extract/tests/extract_docs.rs`.
fn row_file() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus/docs/directive-options.txt")
}

/// Where this table's probe programs live, relative to the corpus root.
const PROBE_SUBDIR: &str = "gate-tables/directives";

fn corpus_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus")
}

/// One row of the table: a directive, one keyword of it, and the position the
/// keyword occupies. The three together are the row's identity, and the two
/// `::OPTIONS` keywords that appear at more than one position are why the
/// position is part of it rather than a note beside it.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct Row {
    directive: String,
    keyword: String,
    position: String,
}

impl Row {
    /// The probe program this row runs, as a corpus-relative path.
    ///
    /// **Derived from the row, not looked up.** A table mapping rows to paths
    /// would be a second place for a row's identity to live and a place for a
    /// typo to hide; deriving it means a row and its probe cannot drift, and
    /// the only way a row can lose its probe is for the file to be missing --
    /// which is structural and named as such.
    fn probe_path(&self) -> String {
        let directive = self.directive.trim_start_matches(':').to_ascii_lowercase();
        let keyword = self.keyword.to_ascii_lowercase();
        let mut position = String::new();
        for c in self.position.chars() {
            if c.is_ascii_alphanumeric() {
                position.push(c.to_ascii_lowercase());
            } else if !position.ends_with('_') {
                position.push('_');
            }
        }
        let position = position.trim_matches('_');
        format!("{PROBE_SUBDIR}/{directive}__{keyword}__{position}.rex")
    }
}

/// Reads the committed row set. Comment and blank lines are dropped, the same
/// way every other reader of a corpus list drops them.
fn read_rows() -> Vec<Row> {
    let path = row_file();
    let text =
        fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    let mut rows = Vec::new();
    for line in text.lines() {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let mut fields = line.split('\t');
        let directive = fields.next().expect("a row has a directive").to_string();
        let keyword = fields
            .next()
            .unwrap_or_else(|| panic!("row {line:?} has no keyword field"))
            .to_string();
        let position = fields
            .next()
            .unwrap_or_else(|| panic!("row {line:?} has no position field"))
            .to_string();
        rows.push(Row {
            directive,
            keyword,
            position,
        });
    }
    rows
}

/// Which phase owes this row an `agree`.
///
/// Every arm's authority, in the order the arms appear:
///
/// * `docs/superpowers/plans/2026-08-17-phase-5a.md`'s handover section hands
///   `::OPTIONS`, `::RESOURCE`, `::REQUIRES`'s `LIBRARY` and `NAMESPACE`, and
///   `::ROUTINE`'s option surface to **5c**, along with the readback of an
///   `::ANNOTATE ROUTINE`. Installing an `::ANNOTATE` target is 5a's, which is
///   what a row of this table measures, so the `ROUTINE` row is 5a and only
///   its readback is 5c's.
/// * The same plan puts `DELEGATE` in **5b**, because `dire.xml` defines it as
///   `expose` plus `forward to()` and `FORWARD` is 5b's.
/// * `::ROUTINE ... EXTERNAL` naming a real shared library is **Phase 7's** and
///   its refusal is untouched by this phase, which the plan states in the same
///   place it moves `::METHOD ... EXTERNAL 'LIBRARY REXX name'` into 5a.
///
///   **This one row spans a boundary the plan asks a later task to draw, and
///   the row cannot hold both sides of it.** A row's identity here is
///   (directive, keyword, position), so `::ROUTINE r EXTERNAL 'LIBRARY <lib>
///   <entry>'` and `::ROUTINE r EXTERNAL 'LIBRARY REXX <entry>'` are one row,
///   and its probe picks the first. If the `LIBRARY REXX` form moves into 5a
///   the way `::METHOD`'s did, that behaviour has **no row in this table**:
///   the row that exists is filed against a phase the 5a gate never reads, and
///   the moved form is outside the denominator. Closing that needs the row set
///   to distinguish the two forms, which is `corpus/docs/directive-options.txt`'s
///   shape and not this file's -- so the task that draws the boundary decides
///   it, and this note is where it meets the consequence.
/// * `::CLASS CLASS` and `::RESOURCE LIBRARY` are the row set's two
///   `cross-reference` rows: the section documents the name and the
///   directive's own parser has no arm for it, so both interpreters refuse
///   them. Both refuse with the same error number and sub-number and differ
///   only in how a syntax error is rendered, which
///   `docs/superpowers/plans/phase-4-exclusions.txt`'s 2026-08-20 note records
///   as a decided deferral that **nobody owns and that is not Phase 5 work**.
///   They are filed under that deferral so they are reported and never gated.
///
/// `None` is a row this assignment has no arm for, which is structural: the
/// row set gained a keyword and nobody decided who owes it.
fn owning_phase(row: &Row) -> Option<&'static str> {
    let directive = row.directive.as_str();
    let keyword = row.keyword.as_str();
    match (directive, keyword) {
        ("::CLASS", "CLASS") | ("::RESOURCE", "LIBRARY") => Some(PARSE_ERROR_RENDERING),
        ("::ROUTINE", "EXTERNAL") => Some("7"),
        ("::METHOD" | "::ATTRIBUTE", "DELEGATE") => Some("5b"),
        ("::OPTIONS" | "::RESOURCE" | "::REQUIRES" | "::ROUTINE", _) => Some("5c"),
        ("::ANNOTATE" | "::ATTRIBUTE" | "::CLASS" | "::METHOD", _) => Some("5a"),
        _ => None,
    }
}

/// The owner of the two `cross-reference` rows: not a phase, and deliberately
/// not spelled like one, so it can never match [`gate_tables::PHASE_GATE_ENV`]
/// or sit in [`gate_tables::CLOSED_PHASES`].
const PARSE_ERROR_RENDERING: &str = "deferred-parse-error-rendering";

/// The five cells partition the cube of three booleans.
///
/// The compiler already refuses a `verdict` that is non-exhaustive or has an
/// unreachable arm, so this adds the half a `match` cannot state: that the
/// five cells' preimages are pairwise disjoint and cover all eight points,
/// checked from outside the function by walking the cube.
///
/// **What it would do if the claim were false.** Two cells sharing a point is
/// unrepresentable -- `verdict` returns one value -- so the failure this can
/// actually catch is a cell with an empty preimage, which is a cell that
/// cannot fire and would be decoration. That is what the count below pins.
#[test]
fn the_verdict_function_partitions_the_descriptor_cube() {
    let mut seen: BTreeMap<Verdict, Vec<Descriptors>> = BTreeMap::new();
    let mut points = 0usize;
    for status in [false, true] {
        for stdout in [false, true] {
            for stderr in [false, true] {
                let differs = Descriptors {
                    status,
                    stdout,
                    stderr,
                };
                seen.entry(verdict(differs)).or_default().push(differs);
                points += 1;
            }
        }
    }
    assert_eq!(points, 8, "the cube of three booleans has eight points");
    assert_eq!(
        seen.values().map(Vec::len).sum::<usize>(),
        points,
        "every point of the cube landed in exactly one cell"
    );
    for cell in Verdict::all() {
        assert!(
            seen.contains_key(cell),
            "the `{}` cell has an empty preimage over the descriptor cube, so no \
             comparison can ever produce it: {seen:?}",
            cell.label()
        );
    }
    assert_eq!(
        seen.len(),
        Verdict::all().len(),
        "every cell of the verdict function is reachable and no other value is"
    );
}

/// One row's finished measurement, for the report.
struct Measured {
    row: Row,
    probe: String,
    phase: &'static str,
    verdict: Verdict,
    loud: bool,
    refused: Option<String>,
    oracle_exit: i32,
    oracle_stdout: Vec<u8>,
    oracle_stderr: Vec<u8>,
    crate_exit: i32,
    crate_stdout: Vec<u8>,
    crate_stderr: Vec<u8>,
    core_uses: Vec<orx::Hit>,
    stream_uses: Vec<orx::Hit>,
}

/// Renders the `.orx` usage column: how many clauses in each file use the
/// row's shape, and where the first one is.
fn usage_column(measured: &Measured) -> String {
    let cite = |hits: &[orx::Hit]| match hits.first() {
        None => "-".to_string(),
        Some(first) if hits.len() == 1 => first.cite(),
        Some(first) => format!("{} +{}", first.cite(), hits.len() - 1),
    };
    format!(
        "core={} stream={}",
        cite(&measured.core_uses),
        cite(&measured.stream_uses)
    )
}

#[test]
fn directive_option_gate_table() {
    let oracle = support::oracle::locate();
    let corpus = corpus_dir();
    let rows = read_rows();
    let mut structural = Vec::new();

    assert!(
        !rows.is_empty(),
        "the committed row set named no rows -- that is a defect in the row \
         file, not an empty pass"
    );

    // Both directions between the row set and the probe corpus, before
    // anything runs. A row with no probe is structural, and so is a probe no
    // row names: an orphan program is one nothing ever runs, which is the
    // shape a deleted row leaves behind.
    let expected: BTreeSet<String> = rows.iter().map(|row| row.probe_path()).collect();
    let probe_dir = corpus.join(PROBE_SUBDIR);
    let on_disk: BTreeSet<String> = fs::read_dir(&probe_dir)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", probe_dir.display()))
        .flatten()
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| name.ends_with(".rex"))
        .map(|name| format!("{PROBE_SUBDIR}/{name}"))
        .collect();
    for missing in expected.difference(&on_disk) {
        structural.push(Structural {
            subject: missing.clone(),
            detail: "a row has no probe program. A row without one is structural and \
                     is never skipped: the table reddens on the missing program rather \
                     than shrinking to the rows that still have one"
                .to_string(),
        });
    }
    for orphan in on_disk.difference(&expected) {
        structural.push(Structural {
            subject: orphan.clone(),
            detail: "a probe program no row names, so nothing runs it".to_string(),
        });
    }

    let core = orx::scan(CORE_CLASSES);
    let stream = orx::scan(STREAM_CLASSES);

    let mut measured = Vec::new();
    for row in rows {
        let probe = row.probe_path();
        let Some(phase) = owning_phase(&row) else {
            structural.push(Structural {
                subject: format!("{} {} {}", row.directive, row.keyword, row.position),
                detail: "no arm of `owning_phase` covers this row, so nothing owes it \
                         an `agree` and no gate could ever read it"
                    .to_string(),
            });
            continue;
        };
        // A channel of its own rather than a fallthrough to the set check
        // above, because that check answers a different question. It reads
        // `read_dir`, which lists an entry by name whatever the name resolves
        // to -- so a probe that is a dangling symlink, or one whose bytes
        // cannot be reached, is *present* in the listing, passes the set
        // comparison, and would leave the table here. Measured: with that
        // path silent, such a row simply vanishes -- the table reports one row
        // fewer, no structural failure, exit 0, and the gated count one lower
        // than it was. A close criterion phrased over the gated count is then
        // satisfiable by removing evidence.
        let abs = match fs::canonicalize(corpus.join(&probe)) {
            Ok(abs) => abs,
            Err(error) => {
                structural.push(Structural {
                    subject: probe.clone(),
                    detail: format!(
                        "the probe is listed in the directory but cannot be resolved: \
                         {error}. The row for {} {} {} therefore has no program to run, \
                         which is structural -- it is never a row the table drops",
                        row.directive, row.keyword, row.position
                    ),
                });
                continue;
            }
        };

        let crate_side = run_on_both_engines(&abs);
        let cpp = oracle.run(&abs);
        if did_not_finish(&cpp) {
            structural.push(Structural {
                subject: probe.clone(),
                detail: format!(
                    "the oracle did not finish: {:?}. Whatever it left on its \
                     descriptors is where it was interrupted, not an answer to \
                     compare",
                    cpp.termination
                ),
            });
            continue;
        }

        let differs = compare_raw(&crate_side, &cpp);
        measured.push(Measured {
            probe,
            phase,
            verdict: verdict(differs),
            loud: is_loud(&crate_side),
            refused: refused_construct(&crate_side.stderr),
            oracle_exit: cpp.expect_exit_code(),
            oracle_stdout: cpp.stdout,
            oracle_stderr: cpp.stderr,
            crate_exit: wrapped_exit_code(crate_side.exit_code),
            crate_stdout: crate_side.stdout,
            crate_stderr: crate_side.stderr,
            core_uses: orx::usage(&core, &row.directive, &row.keyword, &row.position),
            stream_uses: orx::usage(&stream, &row.directive, &row.keyword, &row.position),
            row,
        });
    }

    let shapes = [
        (
            "a quoted class name, MIXINCLASS, and INHERIT naming more than one class",
            [
                orx::quoted_mixin_inheriting_several(&core),
                orx::quoted_mixin_inheriting_several(&stream),
            ]
            .concat(),
        ),
        (
            "a quoted SUBCLASS target",
            [
                orx::quoted_subclass_target(&core),
                orx::quoted_subclass_target(&stream),
            ]
            .concat(),
        ),
        (
            "an install-time ::CONSTANT sending a private class method of its own class",
            [
                orx::constant_sending_an_own_private_class_method(&core),
                orx::constant_sending_an_own_private_class_method(&stream),
            ]
            .concat(),
        ),
        (
            "a directive carrying its body on the same physical line",
            [
                orx::body_on_the_same_physical_line(&core),
                orx::body_on_the_same_physical_line(&stream),
            ]
            .concat(),
        ),
    ];
    for (shape, hits) in &shapes {
        if hits.is_empty() {
            structural.push(Structural {
                subject: format!("orx shape: {shape}"),
                detail: "the scan reached no occurrence of this shape in either \
                         bootstrap file. Either the scanner stopped seeing what those \
                         files contain, or the C++ tree moved out from under it -- \
                         both make every derived usage column below unreliable"
                    .to_string(),
            });
        }
    }

    let mut report = Report::new(
        "rexx-exec gate table D -- the directive and option surface \
         (corpus/docs/directive-options.txt)",
    );
    report.line(&format!(
        "{} rows, one committed probe each under corpus/{PROBE_SUBDIR}/. \
         stderr is compared raw on every row.",
        measured.len()
    ));
    report.line("");
    report.line("the four bootstrap shapes the usage column's scan reaches:");
    for (shape, hits) in &shapes {
        const SHOWN: usize = 6;
        let cites: Vec<String> = hits.iter().take(SHOWN).map(|hit| hit.cite()).collect();
        let rest = match hits.len().checked_sub(SHOWN) {
            Some(0) | None => String::new(),
            Some(more) => format!(", and {more} more"),
        };
        report.line(&format!("  {shape}"));
        report.line(&format!("    {}{rest}", cites.join(", ")));
    }
    report.line("");
    report.line(
        "rows (verdict / loud / owning phase / row / .orx usage / probe, whose \
         directory is named above):",
    );
    for row in &measured {
        report.line(&format!(
            "  {:<14} loud={:<3} {:<29} {:<12} {:<12} {:<16} {:<44} {}",
            row.verdict.label(),
            if row.loud { "yes" } else { "no" },
            row.phase,
            row.row.directive,
            row.row.keyword,
            row.row.position,
            usage_column(row),
            row.probe
                .strip_prefix(PROBE_SUBDIR)
                .and_then(|rest| rest.strip_prefix('/'))
                .unwrap_or(&row.probe),
        ));
        if row.verdict != Verdict::Agree {
            report.line(&format!(
                "      oracle rc={:<4} out={} err={}",
                row.oracle_exit,
                excerpt(&row.oracle_stdout),
                excerpt(&row.oracle_stderr),
            ));
            report.line(&format!(
                "      crate  rc={:<4} out={} err={}{}",
                row.crate_exit,
                excerpt(&row.crate_stdout),
                excerpt(&row.crate_stderr),
                match &row.refused {
                    Some(construct) => format!("   [{construct}]"),
                    None => String::new(),
                },
            ));
        }
    }

    let mut by_verdict: BTreeMap<&str, usize> = BTreeMap::new();
    for row in &measured {
        *by_verdict.entry(row.verdict.label()).or_insert(0) += 1;
    }
    report.line("");
    report.line("verdicts, over every row of this table:");
    for (label, count) in &by_verdict {
        report.line(&format!("  {label}: {count}"));
    }
    report.line(&format!(
        "  loud (this crate declined rather than answered): {}",
        measured.iter().filter(|row| row.loud).count()
    ));

    let mut by_phase: BTreeMap<&str, (usize, usize)> = BTreeMap::new();
    for row in &measured {
        let entry = by_phase.entry(row.phase).or_insert((0, 0));
        entry.0 += 1;
        if row.verdict != Verdict::Agree {
            entry.1 += 1;
        }
    }
    report.line("");
    report.line("by owning phase -- rows, and rows not yet `agree`:");
    for (phase, (total, open)) in &by_phase {
        report.line(&format!("  {phase}: {total} rows, {open} not yet `agree`"));
    }

    let mut by_construct: BTreeMap<&str, usize> = BTreeMap::new();
    for row in &measured {
        if let Some(construct) = &row.refused {
            *by_construct.entry(construct.as_str()).or_insert(0) += 1;
        }
    }
    if !by_construct.is_empty() {
        report.line("");
        report.line("what the loud rows are waiting on:");
        for (construct, count) in &by_construct {
            report.line(&format!("  {construct}: {count}"));
        }
    }

    let gated: Vec<&Measured> = measured
        .iter()
        .filter(|row| row.verdict != Verdict::Agree && verdict_is_gated(row.phase))
        .collect();
    report.line("");
    report.line(&format!(
        "gated by this run: {} row(s) whose owning phase is closing or closed and \
         whose verdict is not `agree`",
        gated.len()
    ));

    gate_tables::emit_uncaptured(&report.finish());

    // Unconditional and first: a structural failure is red in every mode, and
    // the gate mode exists to relax a *verdict* comparison rather than to
    // decide whether a run that produced nothing to compare gets noticed.
    assert_no_structural_failures(&structural);

    let names: Vec<&str> = gated.iter().map(|row| row.probe.as_str()).collect();
    assert!(
        gated.is_empty(),
        "{} row(s) of gate table D owned by a closing or closed phase do not \
         `agree` with the oracle: {names:?}. See the report above for each row's \
         three descriptors on both sides.",
        gated.len()
    );
}
