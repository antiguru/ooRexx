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

//! The loader, run against the oracle's own compiled extension.

use rexx_api::load::{
    self, CURRENT_INTERPRETER_VERSION, Failure, RexxMethodEntry, RexxPackageEntry, RexxRoutineEntry,
};
use std::ffi::c_int;
use std::path::{Path, PathBuf};

/// The repository root, which is three directories above this crate.
fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the repository root is three directories above this crate")
}

/// The oracle's own build of the `rxregexp` extension, which D5's amendment
/// makes the instrument: it is loaded, never rebuilt.
fn rxregexp() -> PathBuf {
    repository_root().join("build/lib/librxregexp.so")
}

fn open_rxregexp() -> load::Library {
    load::open_path(&rxregexp(), "rxregexp")
        .expect("the extension asks for 4.0.0, which is below this interpreter")
        .expect("the extension publishes RexxGetPackage")
}

/// The style every row of `rxregexp_methods` carries, `METHOD_TYPED_STYLE`
/// (`api/oorexxapi.h:221`).
const METHOD_TYPED_STYLE: c_int = 1;

#[test]
fn the_oracles_extension_loads_and_names_itself() {
    let library = open_rxregexp();
    assert_eq!(library.name(), Some(b"rxregexp".as_slice()));
    assert_eq!(library.version(), Some(b"4.0".as_slice()));
}

#[test]
fn the_method_table_is_what_the_extension_declares() {
    let library = open_rxregexp();
    let table: Vec<(c_int, &[u8])> = library
        .methods()
        .iter()
        .map(|row| (row.style, row.name.as_slice()))
        .collect();
    assert_eq!(
        table,
        vec![
            (METHOD_TYPED_STYLE, b"RegExp_Init".as_slice()),
            (METHOD_TYPED_STYLE, b"RegExp_Uninit".as_slice()),
            (METHOD_TYPED_STYLE, b"RegExp_Parse".as_slice()),
            (METHOD_TYPED_STYLE, b"RegExp_Pos".as_slice()),
            (METHOD_TYPED_STYLE, b"RegExp_Match".as_slice()),
        ],
        "the walk answered a different table than `extensions/rxregexp/\
         rxregexp.cpp:207-215` declares"
    );
    for row in library.methods() {
        assert!(
            row.has_entry_point(),
            "{:?} carries no entry point",
            String::from_utf8_lossy(&row.name)
        );
    }
}

#[test]
fn the_extension_declares_no_routines() {
    // `rxregexp.cpp:226` passes NULL for the routine table, and a null table
    // is read as no rows rather than as a failure.
    assert_eq!(open_rxregexp().routines(), &[]);
}

#[test]
fn a_method_is_found_without_regard_to_case() {
    let library = open_rxregexp();
    let found = library
        .method(b"regexp_parse")
        .expect("the table carries RegExp_Parse");
    assert_eq!(found.name, b"RegExp_Parse".to_vec());
    assert_eq!(found.style, METHOD_TYPED_STYLE);
    assert!(found.has_entry_point());
    assert_eq!(library.method(b"RegExp_Zork"), None);
}

#[test]
fn a_library_that_is_nowhere_is_not_an_error() {
    let answer = load::open("zorkolib").expect("a library that is not there is not an error");
    assert!(answer.is_none());
}

#[test]
fn a_library_without_the_exporter_is_not_an_error() {
    // Without the first assertion the second cannot tell "loaded, no
    // `RexxGetPackage`" from "did not load": both answer `Ok(None)`.
    let libc = Path::new("libc.so.6");
    assert!(load::loads(libc), "libc.so.6 did not load");
    let answer = load::open_path(libc, "c").expect("a library with no exporter is not an error");
    assert!(answer.is_none());
}

/// A package entry with no tables, no strings and no hooks, for the fields a
/// test does set to be the only thing under assertion.
fn synthetic_entry(required_version: c_int) -> RexxPackageEntry {
    RexxPackageEntry {
        size: 0,
        api_version: 0,
        required_version,
        package_name: std::ptr::null(),
        package_version: std::ptr::null(),
        loader: None,
        unloader: None,
        routines: std::ptr::null_mut::<RexxRoutineEntry>(),
        methods: std::ptr::null_mut::<RexxMethodEntry>(),
    }
}

#[test]
fn a_package_asking_for_a_newer_interpreter_is_refused() {
    let entry = synthetic_entry(CURRENT_INTERPRETER_VERSION + 1);
    assert_eq!(
        load::check_version(&entry, "zorkolib"),
        Err(Failure::LibraryVersion("zorkolib".to_owned()))
    );
    assert_eq!(
        Failure::LibraryVersion("zorkolib".to_owned()).code(),
        (98, 982)
    );
}

#[test]
fn a_package_asking_for_this_interpreter_or_less_is_accepted() {
    // Zero is "any version" (`LibraryPackage.cpp:232`), and `rxregexp`'s own
    // `REXX_INTERPRETER_4_0_0` is the below-current case.
    for required in [0, 0x0004_0000, CURRENT_INTERPRETER_VERSION] {
        assert_eq!(
            load::check_version(&synthetic_entry(required), "zorkolib"),
            Ok(()),
            "requiredVersion {required:#010x} was refused"
        );
    }
}

#[test]
fn the_interpreter_version_is_the_frozen_headers() {
    let header = std::fs::read_to_string(repository_root().join("api/oorexxapi.h"))
        .expect("the frozen header is readable");
    let named = header
        .lines()
        .find_map(|line| line.strip_prefix("#define REXX_CURRENT_INTERPRETER_VERSION "))
        .expect("the header defines REXX_CURRENT_INTERPRETER_VERSION")
        .trim();
    let literal = header
        .lines()
        .find_map(|line| line.strip_prefix(&format!("#define {named} ")))
        .unwrap_or_else(|| panic!("the header defines {named}"))
        .trim();
    let value = c_int::from_str_radix(
        literal
            .strip_prefix("0x")
            .unwrap_or_else(|| panic!("{named} is a hexadecimal literal")),
        16,
    )
    .expect("a hexadecimal literal");
    assert_eq!(
        CURRENT_INTERPRETER_VERSION, value,
        "the constant no longer matches {named} in api/oorexxapi.h"
    );
}
