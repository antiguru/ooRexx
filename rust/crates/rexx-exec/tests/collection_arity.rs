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

//! `corpus/collection-arity.tsv` (Phase 5g Task 0b): what each documented
//! collection method does when it is sent an argument list it could accept.
//!
//! # Why this exists at all
//!
//! `corpus/method-bodies.txt` classifies a row by sending its name with NO
//! arguments, and its own header says what that costs: for a method that
//! needs arguments the two sides agree about an *arity error*. Measured, that
//! is most of these classes -- `Table~at` reads `answers` and
//! `.Table~new; t['k'] = 'v'` refuses at rc 120; `.Bag~new~put('x')` reads
//! `answers` and is rc 168 against the oracle's rc 0. **No task in this phase
//! may cite that column as a reason a row needs no work.** This table is what
//! it reads instead.
//!
//! # The rule that makes the table mean anything
//!
//! **A row's argument list is real only if the ORACLE completes the send.**
//! The probe prints `SENT` after the send; an oracle run that does not reach
//! it is a harness failure for that row and not a data point, and
//! [`the_oracle_completes_every_send`] makes that a test failure.
//!
//! Without the rule the instrument is defeated two ways that both read green:
//! fill in argument lists for a couple of rows and leave the rest empty, or
//! send two arguments to everything and let both sides agree on 93.902. The
//! rule found eight of my own wrong lists and one crash on the first run.
//!
//! **An earlier version of the rule was itself vacuous**: it asked only for
//! oracle exit 0, and the probe traps `SYNTAX` and exits 0, so a list that
//! *raised* satisfied it. `Stem~index` sent nothing passed that check while
//! segfaulting the interpreter. Requiring `SENT` is what closed it.
//!
//! # The verdicts
//!
//! * `agree` -- the oracle and both engines give identical three descriptors.
//! * `send-differs` -- the receiver was built on both sides and the send
//!   differs. This is the per-row signal.
//! * `setup-differs` -- one side could not build the receiver at all, so the
//!   row says nothing about its own method. **That is the phase's headline
//!   measurement rather than a harness fault**: a class with no store fails
//!   here, and `corpus/collection-receivers.tsv` deliberately uses the richest
//!   receiver the *oracle* can build rather than the richest both sides can.
//! * `engine-differs` -- the two engines disagree with each other, which is a
//!   defect of its own and never expected.
//! * `exempt` -- a row with a committed reason instead of a list.
//!
//! Refresh with
//!   `REXX_COLLECTION_ARITY_REFRESH=1 cargo test --release -p rexx-exec \
//!        --test collection_arity`

mod support;

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const TABLE_FILE: &str = "corpus/collection-arity.tsv";
const REFRESH_ENV: &str = "REXX_COLLECTION_ARITY_REFRESH";
const EXEMPT: &str = "EXEMPT:";

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct Row {
    class: String,
    method: String,
    verdict: String,
    evidence: String,
}

fn corpus_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../corpus")
}

fn read_table(path: &Path, fields: usize) -> Vec<Vec<String>> {
    let text =
        fs::read_to_string(path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    let mut rows = Vec::new();
    for line in text.lines() {
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        let row: Vec<String> = line.split('\t').map(str::to_string).collect();
        assert_eq!(
            row.len(),
            fields,
            "{}: row {line:?} has the wrong number of fields",
            path.display()
        );
        rows.push(row);
    }
    assert!(
        !rows.is_empty(),
        "{} named no rows -- that is a defect in the row file, not an empty pass",
        path.display()
    );
    rows
}

fn receivers() -> HashMap<String, String> {
    read_table(&corpus_root().join("collection-receivers.tsv"), 2)
        .into_iter()
        .map(|row| (row[0].clone(), row[1].clone()))
        .collect()
}

/// Every (class, method, argument list) the probe sends.
fn arguments() -> Vec<(String, String, String)> {
    read_table(&corpus_root().join("collection-arguments.tsv"), 3)
        .into_iter()
        .map(|row| (row[0].clone(), row[1].clone(), row[2].clone()))
        .collect()
}

/// The probe for one row: build the receiver, announce it, send, announce
/// that.
///
/// The message name is quoted because `[]` and `[]=` are not symbols; the
/// send is a bare statement rather than an assignment because a method
/// returning nothing would otherwise raise 91.1 and mask the send.
fn program(setup: &str, method: &str, arguments: &str) -> String {
    let mut text = String::new();
    for statement in setup.split('|') {
        text.push_str(statement.trim());
        text.push('\n');
    }
    let send = if arguments == "--" {
        format!("r~'{method}'")
    } else {
        format!("r~'{method}'({arguments})")
    };
    text.push_str("say 'SETUP-OK'\n");
    text.push_str("signal on syntax name oops\n");
    text.push_str(&send);
    text.push_str("\nsay 'SENT'\nexit\noops:\nsay 'SYNTAX' condition('O')~code\nexit 0\n");
    text
}

/// Exit status, stdout and stderr, read as three separate descriptors.
type Three = (i32, String, String);

fn run(command: &mut Command) -> Three {
    let out = command.output().expect("the probe runs");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

fn probe_dir() -> PathBuf {
    let base = std::env::temp_dir().join(format!(
        "rexx-collection-arity-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("the clock is after the epoch")
            .as_nanos()
    ));
    fs::create_dir_all(&base).expect("a private probe directory");
    base
}

/// The three sides for every row, plus the oracle's stdout so the harness
/// rule can be checked separately from the verdict.
fn measured() -> Vec<(Row, String)> {
    let receivers = receivers();
    let dir = probe_dir();
    let path = dir.join("probe.rex");
    let oracle = support::oracle::oracle_root();
    let binary = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/release/rexx-run");
    assert!(
        binary.is_file(),
        "this test compares the release binary; build it first ({})",
        binary.display()
    );
    let mut rows = Vec::new();
    for (class, method, list) in arguments() {
        if let Some(reason) = list.strip_prefix(EXEMPT) {
            rows.push((
                Row {
                    class,
                    method,
                    verdict: "exempt".into(),
                    evidence: reason.to_string(),
                },
                String::new(),
            ));
            continue;
        }
        let setup = receivers
            .get(&class)
            .unwrap_or_else(|| panic!("collection-receivers.tsv has no setup for {class}"));
        fs::write(&path, program(setup, &method, &list)).expect("the probe is writable");
        let cpp = run(Command::new("bash")
            .arg("-c")
            .arg(format!(
                "ulimit -v 1048576; exec {} {}",
                oracle.join("bin/rexx").display(),
                path.display()
            ))
            .current_dir(&dir)
            .env("LD_LIBRARY_PATH", oracle.join("lib")));
        let mut engines = Vec::new();
        for engine in ["ir", "tree-walker"] {
            engines.push(run(Command::new(&binary)
                .arg(&path)
                .current_dir(&dir)
                .env("REXX_ENGINE", engine)));
        }
        let (verdict, evidence) = classify(&cpp, &engines[0], &engines[1]);
        rows.push((
            Row {
                class,
                method,
                verdict,
                evidence,
            },
            cpp.1,
        ));
    }
    rows.sort();
    rows
}

fn classify(cpp: &Three, ir: &Three, tree: &Three) -> (String, String) {
    if ir != tree {
        return (
            "engine-differs".into(),
            format!("ir rc{} against tree-walker rc{}", ir.0, tree.0),
        );
    }
    if ir == cpp {
        return ("agree".into(), format!("rc{}", cpp.0));
    }
    if !ir.1.contains("SETUP-OK") {
        return (
            "setup-differs".into(),
            format!("crate rc{}: {}", ir.0, first_line(&ir.2)),
        );
    }
    (
        "send-differs".into(),
        format!(
            "oracle rc{} {}; crate rc{} {}",
            cpp.0,
            last_line(&cpp.1),
            ir.0,
            if ir.2.trim().is_empty() {
                last_line(&ir.1)
            } else {
                first_line(&ir.2)
            }
        ),
    )
}

fn first_line(text: &str) -> String {
    text.trim()
        .lines()
        .next()
        .unwrap_or("")
        .chars()
        .take(90)
        .collect()
}

fn last_line(text: &str) -> String {
    text.trim()
        .lines()
        .next_back()
        .unwrap_or("")
        .chars()
        .take(60)
        .collect()
}

const HEADER: &str = "\
# What each documented collection method does when sent an argument list it
# could accept (Phase 5g Task 0b). Derived by
# crates/rexx-exec/tests/collection_arity.rs; edit that, never this.
#
# class<TAB>method<TAB>verdict<TAB>evidence
#
# THIS TABLE REPLACES corpus/method-bodies.txt's verdict column for these
# classes. That column sends no arguments, so for a method that needs them it
# records agreement about an arity error. No task in this phase may cite it as
# a reason a row needs no work.
#
#   agree          the oracle and both engines give identical descriptors
#   send-differs   the receiver was built on both sides and the send differs
#   setup-differs  one side could not build the receiver, so the row says
#                  nothing about its own method -- the class has no store
#   engine-differs the two engines disagree with each other
#   exempt         a committed reason instead of an argument list
#
# The argument lists are corpus/collection-arguments.tsv and the receivers are
# corpus/collection-receivers.tsv. A list is only real if the ORACLE completes
# the send; the test makes an oracle run that does not a failure for that row.
";

fn committed() -> Vec<Row> {
    let path = corpus_root().join("collection-arity.tsv");
    if !path.exists() {
        return Vec::new();
    }
    let mut rows: Vec<Row> = read_table(&path, 4)
        .into_iter()
        .map(|row| Row {
            class: row[0].clone(),
            method: row[1].clone(),
            verdict: row[2].clone(),
            evidence: row[3].clone(),
        })
        .collect();
    rows.sort();
    rows
}

fn refreshing() -> bool {
    matches!(std::env::var(REFRESH_ENV), Ok(value) if !value.is_empty() && value != "0")
}

/// The committed table is what the three sides say today.
#[test]
fn the_table_matches_the_three_sides() {
    let measured = measured();
    if refreshing() {
        let mut text = String::from(HEADER);
        for (row, _) in &measured {
            text.push_str(&format!(
                "{}\t{}\t{}\t{}\n",
                row.class, row.method, row.verdict, row.evidence
            ));
        }
        fs::write(corpus_root().join("collection-arity.tsv"), text).expect("the table is writable");
        return;
    }
    let committed = committed();
    assert!(
        !committed.is_empty(),
        "{TABLE_FILE} is missing. Create it with `{REFRESH_ENV}=1 cargo test --release \
         -p rexx-exec --test collection_arity`"
    );
    let measured: Vec<Row> = measured.into_iter().map(|(row, _)| row).collect();
    let moved: Vec<(&Row, &Row)> = measured
        .iter()
        .zip(committed.iter())
        .filter(|(a, b)| a != b)
        .collect();
    assert!(
        measured.len() == committed.len() && moved.is_empty(),
        "{TABLE_FILE} disagrees with the interpreters. A row that moved because a task \
         implemented something is a refresh; a row that moved otherwise is the finding.\n\
         {moved:#?}"
    );
}

/// **The harness rule.** A row whose send the oracle does not complete is a
/// failure of this instrument for that row, never a data point.
#[test]
fn the_oracle_completes_every_send() {
    let unsent: Vec<String> = measured()
        .iter()
        .filter(|(row, stdout)| row.verdict != "exempt" && !stdout.contains("SENT"))
        .map(|(row, stdout)| {
            format!(
                "{}~{}: the oracle stopped at {:?}",
                row.class,
                row.method,
                stdout.trim().lines().next_back().unwrap_or("nothing")
            )
        })
        .collect();
    assert!(
        unsent.is_empty(),
        "these argument lists are not real -- the oracle does not complete the send, so the \
         row's verdict is about the list and not about the method. Fix the list in \
         corpus/collection-arguments.tsv, or exempt it with a reason.\n{unsent:#?}"
    );
}

/// Every documented instance row has a list, and a native row whose upstream
/// arity is not zero is sent something.
///
/// This is what defeats "fill in the two control rows and leave the rest
/// empty": an empty list on a method that takes arguments is caught here
/// rather than read as agreement.
#[test]
fn every_row_is_sent_something_its_arity_needs() {
    let scopes: HashMap<(String, String), (String, String)> =
        read_table(&corpus_root().join("collection-scopes.tsv"), 7)
            .into_iter()
            .filter(|row| row[2] == "instance")
            .map(|row| {
                (
                    (row[0].clone(), row[1].clone()),
                    (row[4].clone(), row[6].clone()),
                )
            })
            .collect();
    let lists: HashMap<(String, String), String> = arguments()
        .into_iter()
        .map(|(class, method, list)| ((class, method), list))
        .collect();
    let mut wrong = Vec::new();
    for (key, (kind, arity)) in &scopes {
        let Some(list) = lists.get(key) else {
            wrong.push(format!("{}~{} has no argument list at all", key.0, key.1));
            continue;
        };
        if kind == "native" && arity != "0" && list == "--" {
            wrong.push(format!(
                "{}~{} is native at arity {arity} and is sent nothing",
                key.0, key.1
            ));
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
}

/// An exemption carries its reason, so declining a row is a sentence and not
/// a blank.
#[test]
fn every_exemption_says_why() {
    for (class, method, list) in arguments() {
        if let Some(reason) = list.strip_prefix(EXEMPT) {
            assert!(
                reason.len() > 20,
                "{class}~{method} is exempt with no reason worth reading: {reason:?}"
            );
        }
    }
}

/// The two engines never disagree with each other.
#[test]
fn the_engines_agree_with_one_another() {
    let split: Vec<Row> = committed()
        .into_iter()
        .filter(|row| row.verdict == "engine-differs")
        .collect();
    assert!(
        split.is_empty(),
        "the compiled and tree-walking engines answer differently: {split:#?}"
    );
}
