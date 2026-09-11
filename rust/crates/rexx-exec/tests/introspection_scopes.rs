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

//! `corpus/introspection-scopes.tsv` (Phase 5i Task 0): which scope defines
//! each documented introspection method, whether that scope's body is C++ or
//! Rexx, and for a C++ body the `Setup.cpp` token and arity operand.

mod support;

use support::arity::{self, Receiver};
use support::setup_cpp::setup_tables;

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

const TABLE_FILE: &str = "corpus/introspection-scopes.tsv";
const REFRESH_ENV: &str = "REXX_INTROSPECTION_SCOPES_REFRESH";
const RECEIVERS: &str = "introspection-receivers.tsv";
const NO_EVIDENCE: &str = "--";

/// The classes this table covers.
const SCOPE_CLASSES: &[&str] = &[
    "Buffer",
    "Class",
    "Method",
    "Object",
    "Package",
    "Pointer",
    "RexxContext",
    "RexxInfo",
    "Routine",
    "StackFrame",
    "WeakReference",
];

/// The classes whose INSTANCE arm is excluded by name rather than by silence,
/// the way `collection_scopes.rs` excludes `RexxQueue`.
const NO_INSTANCE_ARM: &[&str] = &["Buffer", "Pointer"];

/// The documented rows whose name is a rendering rather than a message.
const DISPLAY_NAMES: &[(&str, &str)] = &[("Object", "(abuttal)"), ("Object", "(blank)")];

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

/// The documented (class, method, arm) rows of the classes in scope, less the
/// display names.
fn documented() -> Vec<(String, String, String)> {
    let mut rows: Vec<(String, String, String)> =
        arity::read_table(&arity::corpus_root().join("docs/class-methods.txt"), 7)
            .into_iter()
            .filter(|row| SCOPE_CLASSES.contains(&row[0].as_str()))
            .filter(|row| !DISPLAY_NAMES.contains(&(row[0].as_str(), row[1].as_str())))
            .filter(|row| !(row[2] == "instance" && NO_INSTANCE_ARM.contains(&row[0].as_str())))
            .map(|row| (row[0].clone(), row[1].clone(), row[2].clone()))
            .collect();
    rows.sort();
    rows
}

fn receivers() -> HashMap<(String, String), Receiver> {
    arity::read_receivers(RECEIVERS, true)
}

// ---- the oracle's answer for the scope column ----

/// One Rexx program per (class, arm), printing
/// `class TAB method TAB arm TAB scope` per row.
fn scope_probe(receiver: &Receiver, rows: &[(String, String, String)]) -> String {
    let mut body = String::new();
    for statement in receiver.setup.split('|') {
        body.push_str(statement.trim());
        body.push('\n');
    }
    for (class, method, arm) in rows {
        body.push_str(&format!(
            "call one r, {}, {}, {}\n",
            quoted(class),
            quoted(method),
            quoted(arm)
        ));
    }
    let mut text = String::new();
    if receiver.wrapper == "call" {
        text.push_str("call probe 'a1', 'a2'\nexit\nprobe:\n");
        text.push_str(&body);
        text.push_str("return\n");
    } else {
        text.push_str(&body);
        text.push_str("exit\n");
    }
    text.push_str(
        "one: procedure\n\
         \x20 use arg recv, cls, nm, arm\n\
         \x20 m = recv~instanceMethod(translate(nm))\n\
         \x20 if m == .nil then s = \"NOMETHOD\"\n\
         \x20 else s = m~scope~id\n\
         \x20 say cls || \"09\"x || nm || \"09\"x || arm || \"09\"x || s\n\
         \x20 return\n",
    );
    if receiver.directives != NO_EVIDENCE {
        for line in receiver.directives.split('|') {
            text.push_str(line.trim());
            text.push('\n');
        }
    }
    text
}

/// A Rexx string literal for `value`, which may hold `[`, `]`, `=` or a quote.
fn quoted(value: &str) -> String {
    format!("\"{}\"", value.replace('"', "\"\""))
}

fn oracle_scopes(rows: &[(String, String, String)]) -> HashMap<(String, String, String), String> {
    let receivers = receivers();
    let dir = tempdir();
    let mut scopes = HashMap::new();
    let mut keys: Vec<(String, String)> = rows
        .iter()
        .map(|(class, _, arm)| (class.clone(), arm.clone()))
        .collect();
    keys.sort();
    keys.dedup();
    for key in keys {
        let receiver = receivers
            .get(&key)
            .unwrap_or_else(|| panic!("{RECEIVERS} has no setup for {} ({})", key.0, key.1));
        let mine: Vec<(String, String, String)> = rows
            .iter()
            .filter(|(class, _, arm)| (class.clone(), arm.clone()) == key)
            .cloned()
            .collect();
        let path = dir.join(format!("scopes_{}_{}.rex", key.0, key.1));
        fs::write(&path, scope_probe(receiver, &mine)).expect("probe is writable");
        let outcome = support::oracle::locate().run(&path);
        assert_eq!(
            outcome.expect_exit_code(),
            0,
            "the scope probe for {} ({}) did not run cleanly on the oracle. stderr:\n{}",
            key.0,
            key.1,
            String::from_utf8_lossy(&outcome.stderr)
        );
        let stdout = String::from_utf8(outcome.stdout).expect("the probe prints text");
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
    }
    scopes
}

/// A private directory for the probe, because a Rexx call to an unresolved
/// name searches the working directory for an external routine.
fn tempdir() -> PathBuf {
    let base = std::env::temp_dir().join(format!(
        "rexx-introspection-scopes-{}-{}",
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
    let rows = documented();
    let scopes = oracle_scopes(&rows);
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
    let path = arity::corpus_root().join("introspection-scopes.tsv");
    if !path.exists() {
        return Vec::new();
    }
    let mut rows: Vec<Scoped> = arity::read_table(&path, 7)
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
# Which scope defines each documented introspection method, and whether that
# scope's body is C++ or Rexx (Phase 5i Task 0). Derived by
# crates/rexx-exec/tests/introspection_scopes.rs, which re-derives it on every
# run; edit that, never this.
#
# class<TAB>method<TAB>arm<TAB>scope<TAB>kind<TAB>token<TAB>arity
#
# `scope` is the oracle's own answer to `receiver~instanceMethod(NAME)~scope~id`
# -- the behaviour after every InheritInstanceMethods copy, RemoveMethod and
# HideMethod, which is why no file scan can produce this column. The receivers
# are corpus/introspection-receivers.tsv, one probe program per (class, arm).
#
# `token` IS A CITATION, NOT A BODY IDENTITY, in both directions: two tokens
# can name one C++ function and one token can reach several behaviours. Count
# tokens for an upper bound on bodies; never for a count of behaviours.
#
# `arity` is Setup.cpp's third operand verbatim: a literal is a MAXIMUM,
# A_COUNT is Arity::Counted.
#
# Pointer's and Buffer's INSTANCE arm is deliberately absent: class-set.txt
# gives them no construction expression, because the reference says instances
# come only from native code (utilityclasses.xml:429, :6910). Their class arm
# is here -- `.Pointer` needs no construction.
#
# Object's (abuttal) and (blank) are deliberately absent: measured on the
# oracle, the book's displayed name reaches no method -- o~'(abuttal)'('x')
# raises 97.1 while o~''('x') and o~' '('x') both answer.
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
        fs::write(arity::corpus_root().join("introspection-scopes.tsv"), text)
            .expect("the table is writable");
        return;
    }
    let committed = committed();
    assert!(
        !committed.is_empty(),
        "{TABLE_FILE} is missing. Create it with `{REFRESH_ENV}=1 cargo test --release \
         -p rexx-exec --test introspection_scopes`"
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
    let committed = committed();
    if !refreshing() {
        assert!(
            !committed.is_empty(),
            "{TABLE_FILE} is missing, so this reads green over nothing"
        );
    }
    let unresolved: Vec<String> = committed
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
    let committed = committed();
    if !refreshing() {
        assert!(
            !committed.is_empty(),
            "{TABLE_FILE} is missing, so this reads green over nothing"
        );
    }
    for row in committed {
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

/// Every class in scope has a receiver for every arm its documented rows use.
#[test]
fn every_class_in_scope_has_a_receiver() {
    let receivers = receivers();
    let mut missing = Vec::new();
    for (class, _, arm) in documented() {
        if !receivers.contains_key(&(class.clone(), arm.clone())) {
            missing.push(format!("{class} ({arm})"));
        }
    }
    missing.sort();
    missing.dedup();
    assert!(missing.is_empty(), "{missing:#?}");
    for class in SCOPE_CLASSES {
        assert!(
            receivers.keys().any(|(name, _)| name == class),
            "{RECEIVERS} has no receiver for {class} at all"
        );
    }
}
