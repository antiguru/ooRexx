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
//! # A verdict says the two sides agree; it does not say either answered
//!
//! Every probe here prints one line, so before any row gets a verdict the
//! oracle's `stdout` line count is checked against
//! [`expected_oracle_lines`] -- one, or none for the rows [`ORACLE_REFUSES`]
//! names. Two interpreters that fail identically agree on all three
//! descriptors, so without this a row whose probe the oracle never reached
//! reads `agree` and is counted as satisfied with nothing asked. A row that
//! fails the check keeps its place and is reported as `unanswered`, never as
//! `agree`.
//!
//! # What this table cannot see
//!
//! A keyword neither `dire.xml` nor `DirectiveParser.cpp` names is outside the
//! denominator, because the row set is their union and nothing else.
//!
//! **For a row [`ORACLE_REFUSES`] names, what is checked is that the oracle
//! refused, not what it refused.** Those rows print nothing on `stdout` --
//! measured, and it is not a probe that could be written differently: the
//! refusal is a translate-time or install-time failure, and both precede the
//! program's own first clause, so a line printed "before" the directive does
//! not exist. The bound is therefore that the oracle wrote a report on
//! `stderr` at all. A probe rewritten to fail for an unrelated reason still
//! writes one and is not caught here; nothing derives these programs, so the
//! instrument against that is the diff. The task that could close it is one
//! that gives table D's row set a column saying what each row's probe is
//! expected to produce -- the same standing gate table C's `entry` and
//! `status` columns have, and the same "a plan task's committed output"
//! ruling that put those there.
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
    is_loud, refused_construct, run_on_both_engines, stdout_lines, verdict, verdict_is_gated,
    verdict_label,
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
///   to distinguish the two forms, which is
///   `corpus/docs/directive-options.txt`'s shape and not this file's -- so the
///   task that draws the boundary decides it, and this note is where it meets
///   the consequence.
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

/// The rows whose probe the oracle refuses before it reaches its `say`.
///
/// **Why this table needs a list where gate table C derives one.** Every probe
/// here prints one line, which is what `corpus/gate-tables/README.md` says
/// they are for: "the oracle side shows the program ran rather than that it
/// produced nothing". Without a check behind that sentence a row reads `agree`
/// whenever the two interpreters fail *identically* -- same status, same
/// `stderr`, both `stdout` empty -- and is counted as a satisfied row of its
/// phase while neither side answered anything. Gate table C bounds each of its
/// families from the probe's own derived text; these probes are hand-written
/// and their row set carries no column that separates the ones the oracle
/// refuses, so the separation is committed here, beside [`owning_phase`] and
/// with the same standing: a human reads it out of a diff.
///
/// **Policed in both directions** by [`expected_oracle_lines`]'s caller: a row
/// named here whose oracle *did* answer is as red as a row not named here
/// whose oracle answered nothing. So the list cannot quietly grow to cover a
/// probe that stopped working.
///
/// Each arm's reason, all of them the same shape -- the probe's own subject is
/// what the oracle refuses:
///
/// * `::CLASS CLASS` and `::RESOURCE LIBRARY` are the row set's
///   `cross-reference` rows: the section documents the name and the
///   directive's own parser has no arm for it, so the directive is a syntax
///   error and no clause of the program runs.
/// * `::ATTRIBUTE`, `::METHOD` and `::ROUTINE`'s `EXTERNAL` name a shared
///   library, and `::REQUIRES`'s `LIBRARY` and `NAMESPACE` name a package;
///   neither is present on this build, so the failure is at install time,
///   before the program's own first clause.
const ORACLE_REFUSES: &[(&str, &str)] = &[
    ("::ATTRIBUTE", "EXTERNAL"),
    ("::CLASS", "CLASS"),
    ("::METHOD", "EXTERNAL"),
    ("::REQUIRES", "LIBRARY"),
    ("::REQUIRES", "NAMESPACE"),
    ("::RESOURCE", "LIBRARY"),
    ("::ROUTINE", "EXTERNAL"),
];

/// Whether [`ORACLE_REFUSES`] names this row.
fn oracle_refuses(row: &Row) -> bool {
    ORACLE_REFUSES
        .iter()
        .any(|&(directive, keyword)| directive == row.directive && keyword == row.keyword)
}

/// How many lines of `stdout` a row's probe prints on the oracle: one, unless
/// the row is one [`ORACLE_REFUSES`] names, and then none.
fn expected_oracle_lines(row: &Row) -> usize {
    usize::from(!oracle_refuses(row))
}

/// Whether the oracle answered a refusing row's question.
///
/// **A bound of zero lines is satisfied by a program that produced nothing at
/// all**, which is the whole of finding 1 reintroduced through the other side
/// of the same `if`: a refusing row's two sides both print nothing on
/// `stdout`, so `compare_raw` agrees and the row reads `agree` with nothing
/// asked. Measured, replacing such a row's probe with a program reading `nop`
/// did exactly that and took the 5a open count down by one.
///
/// **A non-zero `stdout` bound does not exist for these rows, and that was
/// measured rather than assumed.** Every probe here already opens with
/// `say 'main'`; for these the oracle prints nothing, because the refusal is
/// a translate-time or install-time failure and both precede the program's
/// first clause. So the answer these rows do give is a refusal on `stderr`,
/// and that is what is required: measured, each of them writes a report there
/// (the shortest 244 bytes) while `nop` writes none.
fn refusal_answered(oracle_stderr: &[u8]) -> bool {
    !oracle_stderr.is_empty()
}

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
    /// `None` where the oracle did not answer the row's question at all, so
    /// no comparison of the two sides means anything. **The row stays in the
    /// table** rather than being dropped: one row fewer is a lower gated
    /// count, and a close criterion phrased over that count would then be
    /// satisfiable by removing evidence. It is reported as `unanswered`, is
    /// never `agree`, and its own structural failure is what reddens the run.
    verdict: Option<Verdict>,
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
        // to -- so a probe that is a **dangling symlink** is present in the
        // listing, passes the set comparison, and would leave the table here.
        // Measured: with this path silent, such a row simply vanishes -- one
        // row fewer, no structural failure, exit 0, and the gated count one
        // lower than it was. A close criterion phrased over the gated count is
        // then satisfiable by removing evidence.
        //
        // **A probe whose bytes cannot be read is not this arm's case**, which
        // is worth saying because it is the other thing "unreadable" suggests:
        // measured, a `chmod 000` probe canonicalises fine and fails in
        // `run_on_both_engines`'s own read, naming the path.
        let abs = match fs::canonicalize(corpus.join(&probe)) {
            Ok(abs) => abs,
            Err(error) => {
                // Guarded, because a row whose probe was simply deleted
                // reaches here too and the set check has already reported it.
                // Without the guard that row reports twice and this message
                // states the one fact separating the two cases -- that the
                // name is in the listing -- about a name that is not.
                if on_disk.contains(&probe) {
                    structural.push(Structural {
                        subject: probe.clone(),
                        detail: format!(
                            "the probe is listed in the directory but cannot be \
                             resolved: {error}. The row for {} {} {} therefore has no \
                             program to run, which is structural -- it is never a row \
                             the table drops",
                            row.directive, row.keyword, row.position
                        ),
                    });
                }
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

        // Before any verdict: did the oracle answer at all? Two sides that
        // fail identically agree on all three descriptors, so a row whose
        // probe the oracle never reached would read `agree` and be counted
        // as satisfied with nothing asked.
        let answered = stdout_lines(&cpp.stdout).len();
        let expected = expected_oracle_lines(&row);
        let refusal_missing = expected == 0 && !refusal_answered(&cpp.stderr);
        if refusal_missing {
            structural.push(Structural {
                subject: probe.clone(),
                detail: "the oracle refuses this row's probe, so its answer is the report \
                         it writes on `stderr` -- and it wrote none. A row whose two sides \
                         both produce nothing on either channel agrees on all three \
                         descriptors and would read `agree` for a question neither was \
                         asked"
                    .to_string(),
            });
        }
        if answered != expected {
            structural.push(Structural {
                subject: probe.clone(),
                detail: format!(
                    "the oracle answered {answered} line(s) on `stdout` where this row \
                     expects {expected}. Every probe here prints one line so that the \
                     oracle side shows the program ran; the rows the oracle refuses are \
                     the ones `ORACLE_REFUSES` names, and this row is {}named there. \
                     Either the probe stopped running or the list is wrong -- and a row \
                     whose two sides both answer nothing reads `agree` for a question \
                     neither was asked",
                    if expected == 0 { "" } else { "not " }
                ),
            });
        }

        let differs = compare_raw(&crate_side, &cpp);
        measured.push(Measured {
            probe,
            phase,
            verdict: (answered == expected && !refusal_missing).then(|| verdict(differs)),
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
            verdict_label(row.verdict),
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
        if row.verdict != Some(Verdict::Agree) {
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
        *by_verdict.entry(verdict_label(row.verdict)).or_insert(0) += 1;
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
        // `unanswered` is not `agree`, so it counts as open and, on a gated
        // phase, as gated: a row nothing could answer must never make the
        // gated count smaller.
        if row.verdict != Some(Verdict::Agree) {
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
        .filter(|row| row.verdict != Some(Verdict::Agree) && verdict_is_gated(row.phase))
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
