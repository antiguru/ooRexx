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

//! The executable half of the DEVIATION rows that license a difference from
//! the oracle: each row's program run through the C++ interpreter and through
//! both of this crate's engines, with all three descriptors asserted on every
//! side.
//!
//! # The property, and why a fix must delete a row rather than update it
//!
//! A row is red if **either** side's answer moves, in either direction. That
//! includes the crate starting to agree with the oracle: a licence that
//! quietly stopped being needed is a decision to revisit, so the row is
//! deleted with the licence rather than edited to match. `ir_recorded.rs`'s
//! `KNOWN_DIVERGENCES` states the same property for its own rows and is the
//! shape this file is built from.
//!
//! # How to read a red row, because the licence is wider than this test
//!
//! The DEVIATION rows license **ordering**: a finalizer may run at any point
//! after its object is unreachable and no later than termination, which the
//! documentation quoted in each row makes a free choice. This test pins the
//! exact bytes anyway, so that nothing moves quietly. Both are wanted, and the
//! two answers a red row can carry are not the same finding.
//!
//! A change on **this crate's** side, inside the boundary each row's SCOPE
//! states, is a deliberate edit to that row by whoever re-timed the finalizer
//! -- not a defect this test caught. A change on the **oracle's** side, or any
//! exit-status or stderr change on either side, is a real finding.
//!
//! # What is different from that table, and it is the whole design
//!
//! `KNOWN_DIVERGENCES` compares **tree-walker against ir**, both in process.
//! These rows compare **the crate against the oracle**, which needs
//! `support::oracle` and a subprocess, and each row runs **both** crate
//! engines: it asserts that they agree with each other while both differ from
//! the oracle in exactly the recorded way. A row that checked one engine
//! would stay green on a build where the two engines had drifted apart, which
//! is a defect no licence covers.
//!
//! # All three descriptors, on both sides
//!
//! A row records one exit status and one stderr for *both* sides, because a
//! divergence licensed here is a silent one: the two interpreters agree on
//! those and their stdout differs only in where a line sits. Both agreements
//! are asserted rather than assumed, so a row cannot stay green through the
//! crate starting to raise. stderr is compared raw, not through DEVIATION 0's
//! normalisation, which is the stricter claim and available because these
//! programs write none.
//!
//! # A `const` table rather than a `datadriven` case file
//!
//! The standing rule for inline-case tables in this tree is a data file
//! (`docs/superpowers/plans/2026-08-09-phase-4e-ir.md`'s dependency
//! exception). It does not fit here, and the reason is mechanical rather than
//! stylistic: `datadriven::TestFile::run` branches on `env::var("REWRITE")`
//! and rewrites each file's expected block in place
//! (`datadriven-0.9.0/src/lib.rs:501`). A table whose whole claim is "this
//! must not be updated in place" cannot live in a medium with a supported
//! command for updating it in place.
//!
//! # Not gated on `REXX_CORPUS_GATE`
//!
//! Most oracle harnesses here gate on it so an offline checkout is not asked
//! to produce an interpreter. `parse_version_oracle.rs`'s own module doc
//! records that the protection is already unavailable in this crate --
//! `tests/builtin_status.rs` invokes the oracle on a plain `cargo test` with
//! no gate -- so gating buys nothing back. These rows are the only instrument
//! standing between a licensed divergence changing shape and nobody noticing,
//! and each costs one subprocess, so they run in every mode.

mod support;

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use rexx_exec::{Invocation, Outcome, run_program};
use support::oracle::{CppOutcome, did_not_finish, locate, wrapped_exit_code};

/// One licensed divergence: the program, and what each side answers.
struct LicensedDivergence {
    /// The name the licensing DEVIATION row carries after its [`MARKER`],
    /// held equal to this set by
    /// [`the_prose_rows_and_this_table_name_the_same_divergences`]. Short
    /// enough to sit on one line there, because that file is hard-wrapped and
    /// a name long enough to wrap could not be found by a line scan.
    name: &'static str,
    program: &'static str,
    /// The exit status both sides give.
    exit_code: i32,
    /// The stderr both sides give, compared raw.
    stderr: &'static str,
    oracle_stdout: &'static str,
    /// What both crate engines give. One field because a row where they
    /// disagreed would be an engine defect rather than a licensed
    /// divergence, and [`the_licensed_divergences_still_diverge_exactly_as_recorded`]
    /// asserts they do not.
    crate_stdout: &'static str,
}

/// The DEVIATION rows whose difference from the oracle is licensed, each
/// with the transcript its row records.
const LICENSED_DIVERGENCES: &[LicensedDivergence] = &[LicensedDivergence {
    name: "driven-collection-reaches-a-new-object",
    // Eight padding clauses, one short of the nine allocations that reach
    // the oracle's threshold. The padding is the only thing that may sit
    // between `~new` and `drop`: measured, one `say 'built'` either before
    // or after it makes the oracle answer `start / built / uninit ran /
    // after-gc` and both engines agree, three runs each, so the divergence
    // this row records is gone.
    program: "say 'start'\no = .K~new\nz1 = 'pad1'\nz2 = 'pad2'\nz3 = 'pad3'\n\
                  z4 = 'pad4'\nz5 = 'pad5'\nz6 = 'pad6'\nz7 = 'pad7'\nz8 = 'pad8'\n\
                  drop o\ncall gc 'force'\nsay 'after-gc'\n\n\
                  ::class k\n::method uninit\n  say 'uninit ran'\n",
    exit_code: 0,
    stderr: "",
    oracle_stdout: "start\nafter-gc\nuninit ran\n",
    crate_stdout: "start\nuninit ran\nafter-gc\n",
}];

/// Where the prose half lives. `builtin_status.rs`'s own `exclusions_path`,
/// duplicated because the two are separate integration-test binaries and
/// neither can `mod` the other.
fn exclusions_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../docs/superpowers/plans/phase-4-exclusions.txt")
}

/// A directory of this run's own, because an unresolved Rexx name searches the
/// current directory for an external routine -- `support::oracle::Oracle::run`'s
/// own doc comment carries what running a synthesised program from a directory
/// of stale `.rex` files measured.
fn fresh_run_dir(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("the clock is after the epoch")
        .as_nanos();
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("licensed-{name}-{}-{nanos}", std::process::id()));
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("cannot create {}: {e}", dir.display()));
    dir
}

/// Writes one row's program into a directory of its own and answers the path
/// both interpreters are given, canonical so that a raised condition's report
/// would name it identically on either side.
fn materialise(case: &LicensedDivergence) -> PathBuf {
    let file = fresh_run_dir(case.name).join("case.rex");
    fs::write(&file, case.program)
        .unwrap_or_else(|e| panic!("cannot write {}: {e}", file.display()));
    fs::canonicalize(&file).unwrap_or_else(|e| panic!("cannot resolve {}: {e}", file.display()))
}

/// Runs one row's program in process.
fn run_crate(path: &Path) -> Outcome {
    let text = fs::read(path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    let path_str = path
        .to_str()
        .unwrap_or_else(|| panic!("case path {} is not valid UTF-8", path.display()));
    run_program(path_str, text, Invocation::none())
}

/// Every descriptor of one crate-side run, against what the row records.
fn assert_crate_side(case: &LicensedDivergence, outcome: &Outcome) {
    assert_eq!(
        String::from_utf8_lossy(&outcome.stdout),
        case.crate_stdout,
        "[{}] this crate's own stdout moved",
        case.name
    );
    assert_eq!(
        String::from_utf8_lossy(&outcome.stderr),
        case.stderr,
        "[{}] this crate's stderr moved; a licensed divergence \
         that starts raising is no longer silent and is no longer this row",
        case.name
    );
    assert_eq!(
        wrapped_exit_code(outcome.exit_code),
        case.exit_code,
        "[{}] this crate's exit status moved",
        case.name
    );
}

/// Every descriptor of the oracle-side run, against what the row records.
fn assert_oracle_side(case: &LicensedDivergence, cpp: &CppOutcome) {
    assert!(
        !did_not_finish(cpp),
        "[{}] the oracle did not finish: {:?} -- a structural failure, not a \
         byte comparison, and never a licensed divergence",
        case.name,
        cpp.termination
    );
    assert_eq!(
        String::from_utf8_lossy(&cpp.stdout),
        case.oracle_stdout,
        "[{}] the oracle's own answer moved, so what this row licenses a \
         difference from is no longer what it says",
        case.name
    );
    assert_eq!(
        String::from_utf8_lossy(&cpp.stderr),
        case.stderr,
        "[{}] the oracle's stderr moved",
        case.name
    );
    assert_eq!(
        cpp.expect_exit_code(),
        case.exit_code,
        "[{}] the oracle's exit status moved",
        case.name
    );
}

/// Every [`LICENSED_DIVERGENCES`] row still says exactly what it claims: the
/// two crate engines agree with each other, and both differ from the oracle
/// in the recorded way and in no other.
///
/// Red if either side's answer moves, in either direction -- including the
/// crate starting to agree, which is what should delete the row and its
/// DEVIATION together rather than update them.
#[test]
fn the_licensed_divergences_still_diverge_exactly_as_recorded() {
    assert!(
        !LICENSED_DIVERGENCES.is_empty(),
        "an empty table asserts nothing; delete the test with the last row"
    );
    let oracle = locate();
    for case in LICENSED_DIVERGENCES {
        let path = materialise(case);
        let crate_side = run_crate(&path);
        let cpp = oracle.run(&path);

        assert_crate_side(case, &crate_side);
        assert_oracle_side(case, &cpp);

        assert_ne!(
            case.oracle_stdout, case.crate_stdout,
            "[{}] the two answers recorded here are the same, so this row \
             records no divergence at all",
            case.name
        );
    }
    assert_eq!(
        oracle.invocations(),
        LICENSED_DIVERGENCES.len(),
        "every row must be measured by running the oracle on it"
    );
}

/// Marks a DEVIATION row as licensing the [`LICENSED_DIVERGENCES`] row whose
/// `name` follows it to the end of the line.
const MARKER: &str = "LICENSED DIVERGENCE WITNESS: ";

/// The names the exclusions file claims a witness for, one per [`MARKER`].
fn names_the_exclusions_file_claims(exclusions: &str) -> BTreeSet<&str> {
    exclusions
        .lines()
        .filter_map(|line| line.split_once(MARKER))
        .map(|(_, name)| name.trim())
        .collect()
}

/// The prose rows and the executable table name exactly the same divergences.
///
/// A **set equality**, in both directions, because the two failures it has to
/// catch are not symmetrical in the obvious way. Deleting a DEVIATION row
/// takes its marker with it, which a one-directional check would see. Deleting
/// a row from [`LICENSED_DIVERGENCES`] leaves the prose claiming a witness
/// that no longer runs -- the state `phase-4-exclusions.txt` was in before
/// this file existed, and the one its DEVIATION 4 transcripts are still in --
/// and only the other direction sees that. A typo in either copy of a name
/// fails both directions at once.
#[test]
fn the_prose_rows_and_this_table_name_the_same_divergences() {
    let exclusions = fs::read_to_string(exclusions_path())
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", exclusions_path().display()));
    let claimed = names_the_exclusions_file_claims(&exclusions);
    let witnessed: BTreeSet<&str> = LICENSED_DIVERGENCES.iter().map(|case| case.name).collect();
    assert_eq!(
        claimed,
        witnessed,
        "the licences in {} and the rows in LICENSED_DIVERGENCES have drifted \
         apart. A name only on the left is a DEVIATION claiming a witness that \
         does not run; a name only on the right is a witness for a divergence \
         nothing licenses.",
        exclusions_path().display()
    );
}
