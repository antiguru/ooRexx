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

//! Re-derives the internal-package routine names from the C++ tree and
//! compares them with the crate's own table, in both directions. The names are
//! an external enumeration -- three files in a tree this project does not
//! write -- so the table is checked against them rather than described in
//! prose.

use std::collections::BTreeSet;
use std::fs;

const NATIVE_FUNCTIONS: &str = "../../../interpreter/runtime/NativeFunctions.h";
const REXXUTIL_COMMON: &str = "../../../interpreter/runtime/RexxUtilCommon.cpp";
const REXXUTIL_UNIX: &str = "../../../interpreter/platform/unix/SysRexxUtilFunctions.h";

/// Every `MACRO(Name, entry)` first argument in `text`, for the macro named.
fn first_arguments(text: &str, macro_name: &str) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for line in text.lines() {
        // A `#define` of the macro names it too, and is not a row.
        if line.trim_start().starts_with('#') {
            continue;
        }
        let Some(open) = line.find(&format!("{macro_name}(")) else {
            continue;
        };
        let rest = &line[open + macro_name.len() + 1..];
        let Some((name, _)) = rest.split_once(',') else {
            continue;
        };
        let name = name.trim();
        if !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            out.insert(name.to_string());
        }
    }
    out
}

fn read(path: &str) -> String {
    fs::read_to_string(path).unwrap_or_else(|e| panic!("cannot read {path}: {e}"))
}

/// The `REXXUTIL` package's rows: the common table plus the platform half it
/// `#include`s, which is a second file and not a second table.
fn oracle_rexxutil() -> BTreeSet<String> {
    let common = read(REXXUTIL_COMMON);
    let table = common
        .split_once("RexxRoutineEntry rexxutil_routines[]")
        .unwrap_or_else(|| panic!("{REXXUTIL_COMMON} no longer declares rexxutil_routines[]"))
        .1;
    let table = table
        .split_once("\n};")
        .unwrap_or_else(|| panic!("{REXXUTIL_COMMON}'s rexxutil_routines[] is unterminated"))
        .0;
    let mut names = first_arguments(table, "REXX_TYPED_ROUTINE");
    names.extend(first_arguments(&read(REXXUTIL_UNIX), "INTERNAL_ROUTINE"));
    names
}

/// The `REXX` package's rows.
fn oracle_rexx() -> BTreeSet<String> {
    first_arguments(&read(NATIVE_FUNCTIONS), "INTERNAL_ROUTINE")
}

/// The crate's rows for one package.
fn ours(package: &str) -> BTreeSet<String> {
    rexx_exec::internal_routine_rows()
        .into_iter()
        .filter(|(_, p, _)| *p == package)
        .map(|(name, _, _)| name.to_string())
        .collect()
}

#[test]
fn the_table_names_exactly_what_the_two_internal_packages_register() {
    // Per package, not as one union: the two are separate tables in the C++,
    // and a name moving between them is a fact about where a body has to come
    // from.
    for (package, oracle) in [("REXX", oracle_rexx()), ("REXXUTIL", oracle_rexxutil())] {
        let ours = ours(package);
        let missing: Vec<_> = oracle.difference(&ours).cloned().collect();
        let extra: Vec<_> = ours.difference(&oracle).cloned().collect();
        assert!(
            missing.is_empty() && extra.is_empty(),
            "the {package} rows disagree with the C++ tree.\n  \
             registered but not in the table: {missing:?}\n  \
             in the table but not registered: {extra:?}"
        );
        assert!(!oracle.is_empty(), "{package} derived no names at all");
    }
}

#[test]
fn every_row_names_a_phase_that_owes_it() {
    // The three the table uses, and no more: a fourth would be a phase nobody
    // has assigned this work to.
    const PHASES: &[&str] = &["Phase 6", "Phase 7", "Phase 10"];
    for (name, _package, owner) in rexx_exec::internal_routine_rows() {
        assert!(
            PHASES.contains(&owner),
            "{name} names owner {owner:?}, which is not one of {PHASES:?}"
        );
    }
}
