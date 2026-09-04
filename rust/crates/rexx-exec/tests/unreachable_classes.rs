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

//! D73's guard: a class `corpus/docs/class-set.txt` marks `unreachable` still
//! has no route to an instance, on both engines and on the oracle.
//!
//! `unreachable` is grounded in a sentence of the reference saying instances
//! come only from native code, and gate table C reads it as the reason a
//! whole instance arm has no answer. Nothing else asserts on that status, so
//! a class that quietly started constructing would leave those rows reported
//! as impossible while an instance existed.
//!
//! The set comes from the status column rather than from a list of names, so
//! a class that becomes `unreachable` later is guarded by existing. Both
//! sides are asserted and both sides' three descriptors are reported: the
//! oracle's refusal is what makes the status a claim about the language, and
//! this crate's is what makes it a claim about this build.

mod gate_tables;
mod support;
mod watchdog;

use std::fs;
use std::path::{Path, PathBuf};

use gate_tables::{Report, excerpt};
use rexx_exec::{Engine, Invocation, Outcome};
use support::oracle::{CppOutcome, did_not_finish, wrapped_exit_code};

/// The `class-set.txt` status this guard is about.
const UNREACHABLE_STATUS: &str = "unreachable";

/// The `class-set.txt` placeholder for a class with no construction program.
const NO_PROGRAM: &str = "-";

fn corpus_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus")
}

/// The classes `class-set.txt` marks `unreachable`, each with the documented
/// construction route gate table C's instance arm asks.
fn unreachable_classes() -> Vec<(String, String)> {
    let path = corpus_dir().join("docs/class-set.txt");
    let text =
        fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    let mut rows = Vec::new();
    for line in text.lines() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let row: Vec<&str> = line.split('\t').collect();
        assert_eq!(row.len(), 9, "{}: {line:?}", path.display());
        if row[4] != UNREACHABLE_STATUS {
            continue;
        }
        let route = match row[6] {
            NO_PROGRAM => format!(".{}~new", row[0]),
            program => program.to_string(),
        };
        rows.push((row[0].to_string(), route));
    }
    rows
}

/// Whether an outcome is a run that reached the `say` below the construction.
///
/// The construction answering is the only thing this guard forbids, so the
/// discriminator is the probe's own output rather than an exit code: a raise
/// and a refusal differ from each other and neither prints a line.
fn constructed(stdout: &[u8]) -> bool {
    !stdout.is_empty()
}

#[test]
fn a_class_marked_unreachable_still_has_no_route_to_an_instance() {
    let oracle = support::oracle::locate();
    let classes = unreachable_classes();
    assert!(
        !classes.is_empty(),
        "no row of class-set.txt carries the `{UNREACHABLE_STATUS}` status, so this guard has \
         no subject. Derived from the column rather than from a list of names, an empty set is \
         the one way it could pass while asserting nothing"
    );

    let staging = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("unreachable-classes-{}", std::process::id()));
    let mut report = Report::new(
        "rexx-exec D73 -- every class-set.txt row marked `unreachable` still refuses its \
         documented construction route",
    );
    let mut answered: Vec<String> = Vec::new();

    for (name, route) in &classes {
        let dir = staging.join(name);
        fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("cannot create {}: {e}", dir.display()));
        let file = dir.join("probe.rex");
        let text = format!(
            "/* D73: {name} is `{UNREACHABLE_STATUS}` in corpus/docs/class-set.txt, so its\n\
             \x20  documented construction route must not answer an instance. Derived by\n\
             \x20  crates/rexx-exec/tests/unreachable_classes.rs on every run. */\n\
             o = {route}\n\
             say o\n"
        );
        fs::write(&file, &text).unwrap_or_else(|e| panic!("cannot write {}: {e}", file.display()));
        let abs = fs::canonicalize(&file)
            .unwrap_or_else(|e| panic!("cannot canonicalize {}: {e}", file.display()));
        let path = abs.to_str().expect("the staging path is valid UTF-8");

        let run = |engine| -> Outcome {
            watchdog::run_bounded(
                path,
                text.clone().into_bytes(),
                Invocation::none().with_engine(engine),
            )
        };
        let tree_walker = run(Engine::TreeWalker);
        let ir = run(Engine::Ir);
        let cpp: CppOutcome = oracle.run(&abs);
        assert!(
            !did_not_finish(&cpp),
            "the oracle did not finish {route}: {:?}",
            cpp.termination
        );

        report.line("");
        report.line(&format!("  {name}: {route}"));
        for (side, exit, stdout, stderr) in [
            (
                "tree-walker",
                wrapped_exit_code(tree_walker.exit_code),
                &tree_walker.stdout,
                &tree_walker.stderr,
            ),
            (
                "ir",
                wrapped_exit_code(ir.exit_code),
                &ir.stdout,
                &ir.stderr,
            ),
            ("oracle", cpp.expect_exit_code(), &cpp.stdout, &cpp.stderr),
        ] {
            report.line(&format!(
                "      {side:<12} rc={exit:<4} out={} err={}",
                excerpt(stdout),
                excerpt(stderr),
            ));
            if constructed(stdout) {
                answered.push(format!("{name} ({route}) on the {side}"));
            }
        }
    }

    gate_tables::emit_uncaptured(&report.finish());
    assert!(
        answered.is_empty(),
        "a class-set.txt row marked `{UNREACHABLE_STATUS}` has a route to an instance after \
         all: {answered:?}. Gate table C reads that status as the reason a whole instance arm \
         can never agree, so a construction that answers makes those rows a claim about \
         nothing",
    );

    let _ = fs::remove_dir_all(&staging);
}
