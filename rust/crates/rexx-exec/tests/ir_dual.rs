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

//! The dual-engine comparison: every program of this crate's existing
//! populations, run twice -- once on the tree-walker and once on the
//! register-based instruction stream -- with stdout, stderr and exit status
//! required to be byte-identical.
//!
//! # What this proves, and what it does not
//!
//! **It does not yet prove that a promoted construct is right, because
//! nothing is promoted.** Every instruction compiles to `Op::Generic`, which
//! delegates its clause back to the tree-walker's own clause unit. So an
//! all-`Generic` program agrees with itself by construction, and this file's
//! comparison is not evidence about expression evaluation, arithmetic, or
//! any construct's semantics.
//!
//! What it does prove is everything *around* that delegation, which is the
//! whole of what the driver adds and is not shared with `run_activation`:
//!
//! * the outer clause loop terminates where the tree-walker's does, on the
//!   same instruction, for every program in the population;
//! * the `pc` stays an instruction index across `Flow::Goto` and
//!   `Flow::Signal`, so `SIGNAL`, `IF`, `SELECT`, `DO` and `LEAVE` resume at
//!   the same clause under both engines;
//! * the trap offer sits where `run_activation` puts it, so a condition
//!   trapped by `SIGNAL ON`/`CALL ON` is offered once per activation, not
//!   once per nested construct;
//! * `grant_procedure_permission` is granted and spent identically, so
//!   `PROCEDURE` and `USE LOCAL` see the same first-instruction answer;
//! * the register region is opened and closed without disturbing the
//!   temporaries stack any clause depends on;
//! * every body in the population compiles -- `chunks_refused`, which counts
//!   refusals rather than distinct bodies, is asserted zero on both arms, so
//!   a compiler that started refusing ordinary bodies could not pass by
//!   quietly running everything on the tree-walker.
//!
//! Which *engine* actually ran is not observable from a program's output at
//! this point in the phase, and this file makes no attempt to infer it from
//! one. That question is answered where it can be answered honestly, by
//! counting `run_chunk` entries: `src/ir/drive/tests.rs`.
//!
//! # There is no REPORT mode here
//!
//! `corpus.rs`, `assertions.rs`, `bif_assertions.rs` and
//! `keyword_assertions.rs` each default to an always-green progress report
//! and gate only under their own environment variable
//! (`REXX_CORPUS_GATE`, `REXX_ASSERTIONS_GATE`, `REXX_BIF_GATE`,
//! `REXX_KEYWORD_GATE`). That is right for them: each measures how far this
//! crate has got against an external expectation, and how far that is
//! changes with every task.
//!
//! It would be wrong here. A divergence between two engines running the same
//! program is a defect at any point in the phase, with nothing to forgive
//! and no denominator to report -- so **this file asserts unconditionally
//! and reads no environment variable at all.** Setting the four gate
//! variables changes nothing about what it checks, which is strictly
//! stronger than honouring them: there is no mode in which it exits 0 having
//! found a divergence.
//!
//! What those four variables still matter for is the *other* half of the
//! argument. This file says the two engines agree; it says nothing about
//! whether either is right. That comes from the four harnesses above, run in
//! STRICT, on the default engine. The two halves compose and neither
//! substitutes for the other.
//!
//! # The populations, and why an arm cannot silently skip one
//!
//! Both arms run from **one** list. [`populations`] builds every case once
//! and [`compare`] runs that same case twice, so "the two arms saw the same
//! programs" is a property of the code rather than something asserted about
//! two separately built lists.
//!
//! What is asserted is that the list is complete, and every such assertion
//! compares against something outside this file:
//! [`the_dual_harness_reads_every_phase_subset_file`] pins the corpus half
//! against the corpus directory itself (the shape `corpus.rs`'s own
//! `the_differential_reads_every_phase_subset_file` uses),
//! [`the_sweep_runs_every_ootest_suite_a_sibling_harness_runs`] pins the
//! other half against the suite roots the sibling harnesses in `tests/` name,
//! and [`every_population_the_tree_calls_for_is_present_and_non_empty`]
//! requires each to have found programs. A population that silently
//! extracted nothing, or was deleted along with the line declaring it, would
//! otherwise pass with a shrunken denominator and no output saying so.

use std::fs;
use std::path::{Path, PathBuf};

use rexx_exec::{Engine, Invocation, Outcome, run_program};
use rexx_extract::bif::extract_bif;
use rexx_extract::keyword::extract_keyword;
use rexx_extract::{AssertionRow, Form, extract_assertions, find_test_groups};

/// The brief's own first test: a two-clause program, both engines, byte for
/// byte.
///
/// Kept beside the population sweep rather than folded into it because it is
/// the one case a reader can check by eye, and because it is the smallest
/// program that fails if engine selection or the driver's outer loop is
/// broken outright.
#[test]
fn both_engines_agree_on_an_all_generic_program() {
    let text = b"n1 = 2\nsay n1 + 3\n".to_vec();
    let tw = run(text.clone(), Engine::TreeWalker);
    let ir = run(text, Engine::Ir);
    assert_eq!(tw.stdout, ir.stdout);
    assert_eq!(tw.stderr, ir.stderr);
    assert_eq!(tw.exit_code, ir.exit_code);
    assert_eq!(tw.stdout, b"5\n", "the tree-walker's own answer moved");
}

fn run(text: Vec<u8>, engine: Engine) -> Outcome {
    run_program(INLINE_PATH, text, Invocation::none().with_engine(engine))
}

/// The path a program with no file behind it is reported under -- the rows
/// and bodies extracted from `ootest/`, and the inline program above. A
/// label, not a location, in the same spirit as `assertions.rs`'s `ROW_PATH`.
const INLINE_PATH: &str = "/nonexistent/ir-dual-case.rex";

/// One program, run once per engine.
struct Case {
    /// What a divergence report names this case by.
    name: String,
    /// The path the program is reported under, which reaches output through
    /// a raised condition's middle line and so has to be identical on both
    /// arms.
    path: String,
    text: Vec<u8>,
}

/// A named group of cases, drawn from one source.
struct Population {
    name: &'static str,
    cases: Vec<Case>,
}

/// The `ootest/ooRexx/base` suites this sweep draws programs from, sorted.
///
/// A literal, checked against the tree by
/// [`the_sweep_runs_every_ootest_suite_a_sibling_harness_runs`]: the sibling
/// harnesses in `tests/` name the suites they run, on disk, outside this
/// file, so a suite dropped from here is red rather than a quietly smaller
/// sweep. That is [`SUBSET_FILES`]'s arrangement exactly, one literal against
/// one external enumeration, and it is the arrangement a second in-repo list
/// does not have -- deleting a population and its name from a list beside it
/// is one edit, not two.
const OOTEST_SUITES: &[&str] = &["bif", "expressions", "keyword"];

/// The name of the population that is not an `ootest` suite.
const CORPUS_POPULATION: &str = "corpus";

/// The subset files the corpus population reads, in union order.
///
/// The same list `corpus.rs` keeps, for the same reason and pinned the same
/// way -- see [`the_dual_harness_reads_every_phase_subset_file`]. Duplicated
/// rather than shared because these are separate integration-test binaries
/// and neither can `mod` the other.
///
/// A literal here and a directory listing on the other side of the
/// assertion, never two literals: that asymmetry is the whole of what the pin
/// is worth.
const SUBSET_FILES: &[&str] = &["phase-4a.txt", "phase-4b.txt", "phase-4c.txt"];

fn corpus_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus")
}

fn ootest_dir(suite: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../ootest/ooRexx/base")
        .join(suite)
}

/// The phase subset files that exist in the corpus directory, sorted.
///
/// Read from the directory rather than listed a second time, so the
/// assertion cannot be satisfied by a copy of [`SUBSET_FILES`] edited in the
/// same change.
fn phase_subset_files_on_disk() -> Vec<String> {
    let dir = corpus_dir();
    let entries =
        fs::read_dir(&dir).unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()));
    let mut names: Vec<String> = entries
        .flatten()
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| name.starts_with("phase-") && name.ends_with(".txt"))
        .collect();
    names.sort();
    names
}

/// Every corpus program named by a phase subset file, each once.
fn corpus_cases() -> Vec<Case> {
    let dir = corpus_dir();
    let mut seen = std::collections::HashSet::new();
    let mut cases = Vec::new();
    for name in SUBSET_FILES {
        let list_path = dir.join(name);
        let text = fs::read_to_string(&list_path)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", list_path.display()));
        for line in text.lines().map(str::trim) {
            if line.is_empty() || line.starts_with('#') || !seen.insert(line.to_string()) {
                continue;
            }
            let path = dir.join(line);
            // Canonicalised for the same reason `corpus.rs`'s runner passes
            // an absolute path: a raised condition's report names the
            // program, so a relative path would reach stderr. Both arms get
            // the identical string either way, so this is about the
            // comparison being run on realistic bytes rather than about the
            // two agreeing.
            let path = fs::canonicalize(&path)
                .unwrap_or_else(|e| panic!("cannot canonicalise {}: {e}", path.display()));
            let text =
                fs::read(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
            cases.push(Case {
                name: format!("corpus {line}"),
                path: path.to_string_lossy().into_owned(),
                text,
            });
        }
    }
    cases
}

/// The `NUMERIC FORM` keyword for a row's [`Form`].
fn form_keyword(form: Form) -> &'static str {
    match form {
        Form::Scientific => "SCIENTIFIC",
        Form::Engineering => "ENGINEERING",
    }
}

/// A program that establishes one row's state and then `SAY`s each clause.
///
/// The same shape `assertions.rs` and `bif_assertions.rs` build, and for the
/// same reason: `NUMERIC DIGITS`/`FORM` first, always, because a row
/// evaluated at the wrong precision exercises different code in both arms
/// rather than the code the row is about.
fn row_program(digits: u32, form: Form, prelude: &[String], clauses: &[&str]) -> Vec<u8> {
    let mut text = String::new();
    text.push_str(&format!("numeric digits {digits}\n"));
    text.push_str(&format!("numeric form {}\n", form_keyword(form)));
    for line in prelude {
        text.push_str(line);
        text.push('\n');
    }
    for clause in clauses {
        text.push_str("say ");
        text.push_str(clause);
        text.push('\n');
    }
    text.into_bytes()
}

/// Every `.testGroup` under `ootest/ooRexx/base/<suite>`, read once.
fn suite_sources(suite: &str) -> Vec<(String, String)> {
    let dir = ootest_dir(suite);
    let mut groups = find_test_groups(&dir);
    groups.sort();
    assert!(
        !groups.is_empty(),
        "no .testGroup files under {} -- the ootest checkout is missing base/{suite}",
        dir.display()
    );
    groups
        .iter()
        .map(|path| {
            let bytes =
                fs::read(path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
            let group = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("group")
                .to_string();
            (group, String::from_utf8_lossy(&bytes).into_owned())
        })
        .collect()
}

fn case_of(row: &AssertionRow, index: usize) -> Case {
    // Both texts in one program, where `assertions.rs` says the expected
    // value only for a value row. The two arms are compared against each
    // other rather than against the row's own claim, so the extra clause
    // costs nothing and exercises a second expression per case.
    let clauses: Vec<&str> = if row.expect_raise.is_some() {
        vec![row.expr.as_str()]
    } else {
        vec![row.expr.as_str(), row.expected.as_str()]
    };
    Case {
        name: format!("{}::{}#{index}", row.group, row.method),
        path: INLINE_PATH.to_string(),
        text: row_program(row.digits, row.form, &row.prelude, &clauses),
    }
}

/// Every case, one population per entry of [`OOTEST_SUITES`] plus the
/// corpus.
///
/// The `match` has no catch-all: a suite named in that list with no arm here
/// cannot be turned into programs, and saying so loudly is the only honest
/// answer -- silently running three populations where four were declared is
/// the shrunken denominator this file exists to prevent.
fn populations() -> Vec<Population> {
    let mut out = vec![Population {
        name: CORPUS_POPULATION,
        cases: corpus_cases(),
    }];
    for suite in OOTEST_SUITES {
        let cases = match *suite {
            "expressions" => expression_cases(suite),
            "bif" => bif_cases(suite),
            "keyword" => keyword_cases(suite),
            other => panic!(
                "ootest suite base/{other} is declared in OOTEST_SUITES and nothing here turns \
                 its .testGroup files into programs"
            ),
        };
        out.push(Population { name: suite, cases });
    }
    out
}

fn expression_cases(suite: &str) -> Vec<Case> {
    let mut cases = Vec::new();
    for (group, source) in suite_sources(suite) {
        for (index, row) in extract_assertions(&group, &source).rows.iter().enumerate() {
            cases.push(case_of(row, index));
        }
    }
    cases
}

fn bif_cases(suite: &str) -> Vec<Case> {
    let mut cases = Vec::new();
    for (group, source) in suite_sources(suite) {
        let extraction = extract_bif(&group, &source);
        for (index, row) in extraction.rows.iter().enumerate() {
            cases.push(case_of(row, index));
        }
        for (index, row) in extraction.raises.iter().enumerate() {
            let operands: Vec<&str> = row.operands.iter().map(String::as_str).collect();
            cases.push(Case {
                name: format!("{}::{} raise #{index}", row.group, row.method),
                path: INLINE_PATH.to_string(),
                text: row_program(row.digits, row.form, &row.prelude, &operands),
            });
        }
    }
    cases
}

fn keyword_cases(suite: &str) -> Vec<Case> {
    let mut cases = Vec::new();
    for (group, source) in suite_sources(suite) {
        for body in extract_keyword(&group, &source).bodies {
            cases.push(Case {
                name: format!("{}::{}", body.group, body.method),
                path: INLINE_PATH.to_string(),
                text: body.program.clone().into_bytes(),
            });
        }
    }
    cases
}

/// Runs one case on both engines and describes the first difference, if any.
///
/// One `Case` and two runs, so the two arms cannot be given different
/// programs. Stdout, stderr and exit status are compared **unnormalised**:
/// there is no oracle here, so `corpus.rs`'s DEVIATION 0 does not apply and
/// a trace line's own indentation is required to match exactly.
fn compare(case: &Case) -> Option<String> {
    let tw = run_program(
        &case.path,
        case.text.clone(),
        Invocation::none().with_engine(Engine::TreeWalker),
    );
    let ir = run_program(
        &case.path,
        case.text.clone(),
        Invocation::none().with_engine(Engine::Ir),
    );

    if tw.exit_code != ir.exit_code {
        return Some(format!(
            "exit status: tree-walker {} vs ir {}",
            tw.exit_code, ir.exit_code
        ));
    }
    if tw.stdout != ir.stdout {
        return Some(format!(
            "stdout:\n  tree-walker {:?}\n  ir          {:?}",
            excerpt(&tw.stdout),
            excerpt(&ir.stdout)
        ));
    }
    if tw.stderr != ir.stderr {
        return Some(format!(
            "stderr:\n  tree-walker {:?}\n  ir          {:?}",
            excerpt(&tw.stderr),
            excerpt(&ir.stderr)
        ));
    }
    // Not a divergence between the arms, and that is exactly why it is
    // checked here rather than left to the comparison above: a body the
    // compiler refuses runs on the tree-walker under *both* arms, so the two
    // agree and the population passes while the engine under test never ran.
    if ir.chunks_refused != 0 {
        return Some(format!(
            "the ir arm refused a body {} times, running it on the tree-walker \
             instead",
            ir.chunks_refused
        ));
    }
    if tw.chunks_refused != 0 {
        return Some(format!(
            "the tree-walker arm counted {} refusals, and it compiles nothing \
             to refuse",
            tw.chunks_refused
        ));
    }
    None
}

/// Bounds a byte string to a readable excerpt, so a divergence stays
/// diagnosable without reprinting a program's whole output.
fn excerpt(bytes: &[u8]) -> String {
    const LIMIT: usize = 300;
    let text = String::from_utf8_lossy(bytes);
    if text.len() <= LIMIT {
        return text.into_owned();
    }
    format!("{}… ({} bytes)", &text[..LIMIT], bytes.len())
}

/// The dual harness reads **every** phase subset file the corpus has.
///
/// `corpus.rs`'s own `the_differential_reads_every_phase_subset_file` has the
/// argument: a file missing from the list is a phase whose programs are never
/// run, the sweep stays green over whatever is left, and nothing else here
/// can see it happen.
#[test]
fn the_dual_harness_reads_every_phase_subset_file() {
    assert_eq!(
        SUBSET_FILES,
        phase_subset_files_on_disk(),
        "the dual-engine sweep does not read every phase subset file in \
         rust/corpus/"
    );
}

/// Every `ootest` suite a sibling harness in `tests/` runs programs from,
/// read out of those files rather than listed here a second time.
///
/// The marker is the path the harnesses build their suite root from, which is
/// a literal in each of them. `ir_dual.rs` itself joins the suite name on
/// separately and so contributes nothing to this set, which is what stops the
/// answer being a copy of the question -- but the file is skipped by name as
/// well, so a later edit that spelled the path out here could not quietly
/// satisfy the pin either.
fn ootest_suites_sibling_harnesses_read() -> Vec<String> {
    const MARKER: &str = "ootest/ooRexx/base/";
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let entries =
        fs::read_dir(&dir).unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()));
    let mut suites = std::collections::BTreeSet::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_none_or(|e| e != "rs")
            || path.file_name().is_some_and(|n| n == "ir_dual.rs")
        {
            continue;
        }
        let text = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
        for (offset, _) in text.match_indices(MARKER) {
            let tail = &text[offset + MARKER.len()..];
            let end = tail
                .find(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                .unwrap_or(tail.len());
            if end > 0 {
                suites.insert(tail[..end].to_string());
            }
        }
    }
    suites.into_iter().collect()
}

/// The sweep runs every `ootest` suite some other harness in this crate runs.
///
/// **What this rules out is a two-sided deletion**, which is the one that
/// happens: a population removed from [`populations`] *and* from the list
/// beside it leaves every other test in this file green over a sweep missing
/// thousands of programs. Pinning the list against a second list in the same
/// file does not rule that out, because both are one edit away. The sibling
/// harness sources are not.
#[test]
fn the_sweep_runs_every_ootest_suite_a_sibling_harness_runs() {
    let on_disk = ootest_suites_sibling_harnesses_read();
    assert!(
        !on_disk.is_empty(),
        "no sibling harness in tests/ names an ootest suite root, so this pin \
         found nothing to compare against and would accept any sweep at all"
    );
    assert_eq!(
        OOTEST_SUITES, on_disk,
        "the dual-engine sweep and this crate's other harnesses do not run the \
         same ootest suites. A suite only they run is one the two engines are \
         never compared on"
    );
}

/// Every population the tree calls for is built, and every one of them found
/// programs.
///
/// The expectation is derived, not restated: the corpus population is
/// required because `rust/corpus/` has phase subset files in it, and each
/// suite population because a sibling harness runs that suite. So deleting
/// either kind from [`populations`] is red without a second edit anywhere
/// being able to hide it.
///
/// Non-emptiness is separate from presence and catches the other shape: an
/// extractor pointed at the wrong directory, or a subset file naming nothing,
/// builds a population that exists and runs no programs.
#[test]
fn every_population_the_tree_calls_for_is_present_and_non_empty() {
    let mut required = vec![CORPUS_POPULATION.to_string()];
    assert!(
        !phase_subset_files_on_disk().is_empty(),
        "rust/corpus/ has no phase subset file, so nothing here requires the \
         corpus population to exist"
    );
    required.extend(ootest_suites_sibling_harnesses_read());
    required.sort();

    let populations = populations();
    let mut names: Vec<String> = populations.iter().map(|p| p.name.to_string()).collect();
    names.sort();
    assert_eq!(
        names, required,
        "a population the tree calls for is missing"
    );

    for population in &populations {
        assert!(
            !population.cases.is_empty(),
            "the {} population is empty, so the sweep asserts nothing about it",
            population.name
        );
    }
}

/// The sweep: every case of every population, both engines, byte for byte.
#[test]
fn both_engines_agree_across_every_population() {
    let populations = populations();
    let mut divergences = Vec::new();
    let mut compared = 0usize;
    for population in &populations {
        for case in &population.cases {
            compared += 1;
            if let Some(reason) = compare(case) {
                divergences.push(format!("[{}] {}: {reason}", population.name, case.name));
            }
        }
    }
    assert!(
        compared > 0,
        "the sweep compared nothing at all, which is a harness defect and not \
         an empty pass"
    );
    assert!(
        divergences.is_empty(),
        "{} of {compared} programs behave differently on the two engines:\n{}",
        divergences.len(),
        divergences.join("\n")
    );
}
