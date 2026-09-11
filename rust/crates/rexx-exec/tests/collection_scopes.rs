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

//! `corpus/collection-scopes.tsv` (D89): which scope defines each documented
//! collection method, whether that scope's body is C++ or Rexx, and for a C++
//! body the `Setup.cpp` token and arity operand.

mod support;

use support::setup_cpp::setup_tables;

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

const TABLE_FILE: &str = "corpus/collection-scopes.tsv";
const REFRESH_ENV: &str = "REXX_COLLECTION_SCOPES_REFRESH";
const NO_EVIDENCE: &str = "--";

/// The classes this table covers.
const SCOPE_CLASSES: &[&str] = &[
    "Array",
    "Bag",
    "CircularQueue",
    "Directory",
    "IdentityTable",
    "List",
    "Properties",
    "Queue",
    "Relation",
    "Set",
    "Stem",
    "StringTable",
    "Supplier",
    "Table",
];

/// A row of the derived table.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
struct Scoped {
    class: String,
    method: String,
    arm: String,
    scope: String,
    kind: String,
    token: String,
    arity: String,
}

impl Scoped {
    fn line(&self) -> String {
        format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}",
            self.class, self.method, self.arm, self.scope, self.kind, self.token, self.arity
        )
    }
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

/// Each in-scope class's committed construction expression.
fn constructions(corpus: &Path) -> HashMap<String, String> {
    let mut found = HashMap::new();
    for row in read_table(&corpus.join("docs/class-set.txt"), 9) {
        if SCOPE_CLASSES.contains(&row[0].as_str()) && row[6] != NO_EVIDENCE {
            found.insert(row[0].clone(), row[6].clone());
        }
    }
    for class in SCOPE_CLASSES {
        assert!(
            found.contains_key(*class),
            "class-set.txt gives {class} no construction expression, so no instance-arm row of \
             it can be scoped. Every class in SCOPE_CLASSES needs one."
        );
    }
    found
}

/// The documented (class, method, arm) rows of the classes in scope.
fn documented(corpus: &Path) -> Vec<(String, String, String)> {
    let mut rows: Vec<(String, String, String)> =
        read_table(&corpus.join("docs/class-methods.txt"), 7)
            .into_iter()
            .filter(|row| SCOPE_CLASSES.contains(&row[0].as_str()))
            .map(|row| (row[0].clone(), row[1].clone(), row[2].clone()))
            .collect();
    rows.sort();
    rows
}

// ---- the oracle's answer for the scope column ----

/// One Rexx program that prints `class TAB method TAB arm TAB scope` per row.
fn scope_probe(rows: &[(String, String, String)], built: &HashMap<String, String>) -> String {
    let mut text = String::new();
    let mut classes: Vec<&String> = built.keys().collect();
    classes.sort();
    for (index, class) in classes.iter().enumerate() {
        text.push_str(&format!("r{index} = {}\n", built[*class]));
    }
    let slot: HashMap<&str, usize> = classes
        .iter()
        .enumerate()
        .map(|(index, class)| (class.as_str(), index))
        .collect();
    for (class, method, arm) in rows {
        let receiver = if arm == "class" {
            format!(".{class}")
        } else {
            format!("r{}", slot[class.as_str()])
        };
        text.push_str(&format!(
            "call one {receiver}, {}, {}, {}\n",
            quoted(class),
            quoted(method),
            quoted(arm)
        ));
    }
    text.push_str(
        "exit\n\
         one: procedure\n\
         \x20 use arg recv, cls, nm, arm\n\
         \x20 m = recv~instanceMethod(translate(nm))\n\
         \x20 if m == .nil then s = \"NOMETHOD\"\n\
         \x20 else s = m~scope~id\n\
         \x20 say cls || \"09\"x || nm || \"09\"x || arm || \"09\"x || s\n\
         \x20 return\n",
    );
    text
}

/// A Rexx string literal for `value`, which may hold `[`, `]`, `=` or a quote.
fn quoted(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

fn oracle_scopes(
    rows: &[(String, String, String)],
    built: &HashMap<String, String>,
) -> HashMap<(String, String, String), String> {
    let dir = tempdir();
    let path = dir.join("collection_scopes.rex");
    fs::write(&path, scope_probe(rows, built)).expect("probe is writable");
    let outcome = support::oracle::locate().run(&path);
    assert_eq!(
        outcome.expect_exit_code(),
        0,
        "the scope probe did not run cleanly on the oracle. stderr:\n{}",
        String::from_utf8_lossy(&outcome.stderr)
    );
    let stdout = String::from_utf8(outcome.stdout).expect("the probe prints text");
    let mut scopes = HashMap::new();
    for line in stdout.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        assert_eq!(parts.len(), 4, "the probe printed {line:?}");
        scopes.insert(
            (
                parts[0].to_string(),
                parts[1].to_string(),
                parts[2].to_string(),
            ),
            parts[3].to_string(),
        );
    }
    scopes
}

/// A private directory for the probe, because a Rexx call to an unresolved
/// name searches the working directory for an external routine.
fn tempdir() -> PathBuf {
    let base = std::env::temp_dir().join(format!(
        "rexx-collection-scopes-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("the clock is after the epoch")
            .as_nanos()
    ));
    fs::create_dir_all(&base).expect("a private probe directory");
    base
}

// ---- the join, the committed table, and the tests ----

fn derived() -> Vec<Scoped> {
    let corpus = corpus_root();
    let rows = documented(&corpus);
    let scopes = oracle_scopes(&rows, &constructions(&corpus));
    let (instance, class) = setup_tables();
    let mut out = Vec::new();
    for (class_name, method, arm) in rows {
        let key = (class_name.clone(), method.clone(), arm.clone());
        let scope = scopes
            .get(&key)
            .unwrap_or_else(|| panic!("the probe printed no line for {key:?}"))
            .clone();
        let tables = if arm == "class" { &class } else { &instance };
        let native = tables
            .get(&scope)
            .and_then(|table| table.get(&method.to_uppercase()));
        let (kind, token, arity) = match native {
            Some((token, arity)) => ("native", token.clone(), arity.clone()),
            None => ("rexx", NO_EVIDENCE.to_string(), NO_EVIDENCE.to_string()),
        };
        out.push(Scoped {
            class: class_name,
            method,
            arm,
            scope,
            kind: kind.to_string(),
            token,
            arity,
        });
    }
    out.sort();
    out
}

fn committed() -> Vec<Scoped> {
    let path = corpus_root().join("collection-scopes.tsv");
    if !path.exists() {
        return Vec::new();
    }
    let mut rows: Vec<Scoped> = read_table(&path, 7)
        .into_iter()
        .map(|row| Scoped {
            class: row[0].clone(),
            method: row[1].clone(),
            arm: row[2].clone(),
            scope: row[3].clone(),
            kind: row[4].clone(),
            token: row[5].clone(),
            arity: row[6].clone(),
        })
        .collect();
    rows.sort();
    rows
}

const HEADER: &str = "\
# Which scope defines each documented collection method, and whether that
# scope's body is C++ or Rexx (D89). Derived by
# crates/rexx-exec/tests/collection_scopes.rs, which re-derives it on every
# run; edit that, never this.
#
# class<TAB>method<TAB>arm<TAB>scope<TAB>kind<TAB>token<TAB>arity
#
# `scope` is the oracle's own answer to `receiver~instanceMethod(NAME)~scope~id`
# -- the behaviour after every InheritInstanceMethods copy, RemoveMethod and
# HideMethod, which is why no file scan can produce this column.
#
# `token` IS A CITATION, NOT A BODY IDENTITY, in both directions. Two tokens
# can name one C++ function: `Set`'s HasItem is written
# IdentityTable::hasIndexRexx and IdentityTableClass.hpp declares no such
# member. And one token reaches three behaviours selected by the contents
# class the receiver allocated: HashCollection::putRexx passes
# IndexOnlyHashCollection's validation on a Set or Bag and reaches
# MultiValueContents::put on a Relation or Bag. Count tokens for an upper
# bound on bodies; never for a count of behaviours.
#
# `arity` is Setup.cpp's third operand verbatim: a literal is a MAXIMUM,
# A_COUNT is Arity::Counted.
#
# RexxQueue is deliberately absent -- spec D93, the external data queue.
";

fn refreshing() -> bool {
    matches!(std::env::var(REFRESH_ENV), Ok(value) if !value.is_empty() && value != "0")
}

/// The committed table is what the oracle and `Setup.cpp` say together.
#[test]
fn the_table_matches_the_interpreter() {
    let derived = derived();
    if refreshing() {
        let mut text = String::from(HEADER);
        for row in &derived {
            text.push_str(&row.line());
            text.push('\n');
        }
        fs::write(corpus_root().join("collection-scopes.tsv"), text)
            .expect("the table is writable");
        return;
    }
    let committed = committed();
    assert!(
        !committed.is_empty(),
        "{TABLE_FILE} is missing. Create it with `{REFRESH_ENV}=1 cargo test --release \
         -p rexx-exec --test collection_scopes`"
    );
    let only_derived: Vec<&Scoped> = derived.iter().filter(|r| !committed.contains(r)).collect();
    let only_committed: Vec<&Scoped> = committed.iter().filter(|r| !derived.contains(r)).collect();
    assert!(
        only_derived.is_empty() && only_committed.is_empty(),
        "{TABLE_FILE} disagrees with the interpreter; re-derive it rather than editing the \
         disagreeing row.\n  in the interpreter, not the table: {only_derived:#?}\
         \n  in the table, not the interpreter: {only_committed:#?}"
    );
}

/// A row whose scope resolves nowhere is a failure, not a blank -- that is
/// what catches an upstream rename.
#[test]
fn every_documented_row_resolves_to_a_scope() {
    let unresolved: Vec<String> = committed()
        .iter()
        .filter(|row| row.scope == "NOMETHOD")
        .map(|row| format!("{}~{} ({})", row.class, row.method, row.arm))
        .collect();
    assert!(
        unresolved.is_empty(),
        "these documented rows reach no method on the oracle, so nothing can own them: \
         {unresolved:#?}"
    );
}

/// Every native row cites a `Setup.cpp` arity operand, and every Rexx row
/// cites neither token nor arity.
#[test]
fn the_kind_column_agrees_with_the_two_it_governs() {
    for row in committed() {
        match row.kind.as_str() {
            "native" => assert!(
                row.token != NO_EVIDENCE && row.arity != NO_EVIDENCE,
                "{}~{} is native and cites no token or arity",
                row.class,
                row.method
            ),
            "rexx" => assert!(
                row.token == NO_EVIDENCE && row.arity == NO_EVIDENCE,
                "{}~{} is a Rexx body and cites a C++ token",
                row.class,
                row.method
            ),
            other => panic!("{}~{} has kind {other:?}", row.class, row.method),
        }
    }
}
