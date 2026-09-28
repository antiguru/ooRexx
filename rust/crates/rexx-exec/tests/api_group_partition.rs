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

//! Which phase owns each native-API ooTest group, derived from the libraries
//! the group binds and what the oracle's builds of them need and import.
//! ELF metadata is read with `readelf`; no library is ever loaded.

mod support;

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The partition this test asserts, per group file stem. Phase 8 is the
/// groups whose libraries need no interpreter library; any other group is
/// the latest phase one of its imports belongs to.
const PARTITION: &[(&str, u8)] = &[
    ("METHOD", 8),
    ("CONVERSION", 8),
    ("FUNCTION", 8),
    ("RexxStart", 10),
    ("ProcessRexxStart", 10),
    ("INVOCATION", 10),
    ("ProcessInvocation", 10),
    ("CLASSIC", 10),
];

/// SONAME stems of the oracle's own interpreter. The walk stops at them:
/// what they import from each other is not the group's.
const INTERPRETER_LIBRARIES: &[&str] = &["librexx.so", "librexxapi.so"];

/// Imports that create or start an interpreter: the embedding API, Phase 9's.
const EMBEDDING: &[&str] = &["RexxCreateInterpreter", "RexxStart"];

/// The function, subcom, exit, queue and macro-space registries: RXAPI,
/// Phase 10's.
const REGISTRIES: &[&str] = &[
    "RexxRegisterFunctionExe",
    "RexxRegisterFunctionDll",
    "RexxDeregisterFunction",
    "RexxQueryFunction",
    "RexxRegisterSubcomExe",
    "RexxRegisterSubcomDll",
    "RexxDeregisterSubcom",
    "RexxQuerySubcom",
    "RexxRegisterExitExe",
    "RexxRegisterExitDll",
    "RexxDeregisterExit",
    "RexxQueryExit",
    "RexxCreateQueue",
    "RexxOpenQueue",
    "RexxDeleteQueue",
    "RexxQueryQueue",
    "RexxQueueExists",
    "RexxAddQueue",
    "RexxPullFromQueue",
    "RexxClearQueue",
    "RexxAddMacro",
    "RexxDropMacro",
    "RexxClearMacroSpace",
    "RexxLoadMacroSpace",
    "RexxSaveMacroSpace",
    "RexxQueryMacro",
    "RexxReorderMacro",
];

/// Imports that name no phase of their own: memory and the variable pool,
/// used from inside a callback whichever phase hosts it.
const NEUTRAL: &[&str] = &["RexxAllocateMemory", "RexxFreeMemory", "RexxVariablePool"];

const IMPORT_PHASES: &[(&[&str], u8)] = &[(EMBEDDING, 9), (REGISTRIES, 10)];

/// Where a `NEEDED` entry is looked for after the library's own `RUNPATH`.
const SYSTEM_LIBRARY_DIRS: &[&str] = &[
    "/lib/x86_64-linux-gnu",
    "/usr/lib/x86_64-linux-gnu",
    "/lib64",
];

fn api_groups_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../ootest/ooRexx/API")
}

fn oracle_lib_dir() -> PathBuf {
    support::oracle::oracle_root().join("lib")
}

fn group_files(dir: &Path, found: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(dir).unwrap_or_else(|e| panic!("reading {}: {e}", dir.display()));
    for entry in entries {
        let path = entry.expect("a directory entry").path();
        if path.is_dir() {
            group_files(&path, found);
        } else if path.extension().is_some_and(|e| e == "testGroup") {
            found.push(path);
        }
    }
}

fn read_source(path: &Path) -> String {
    let bytes = fs::read(path).unwrap_or_else(|e| panic!("reading {}: {e}", path.display()));
    String::from_utf8_lossy(&bytes).into_owned()
}

/// Every quoted literal in `text`, in order.
fn literals(text: &str) -> Vec<&str> {
    let mut found = Vec::new();
    let mut rest = text;
    while let Some(open) = rest.find(['\'', '"']) {
        let quote = rest.as_bytes()[open] as char;
        let body = &rest[open + 1..];
        let Some(close) = body.find(quote) else { break };
        found.push(&body[..close]);
        rest = &body[close + 1..];
    }
    found
}

/// Offsets just past each case-blind occurrence of `word` in `text`.
fn after_each<'a>(text: &'a str, word: &str) -> impl Iterator<Item = &'a str> {
    let lower = text.to_ascii_lowercase();
    let word = word.to_ascii_lowercase();
    lower
        .match_indices(&word)
        .map(|(at, _)| at + word.len())
        .collect::<Vec<_>>()
        .into_iter()
        .map(move |end| &text[end..])
}

/// The libraries `source` binds: the second word of each `EXTERNAL
/// "LIBRARY <name> ..."` and the second argument of each `rxfuncadd`.
fn bound_libraries(source: &str, path: &Path) -> BTreeSet<String> {
    let mut libraries = BTreeSet::new();
    for rest in after_each(source, "external") {
        let trimmed = rest.trim_start_matches([' ', '\t']);
        if trimmed.len() == rest.len() || !trimmed.starts_with(['\'', '"']) {
            continue;
        }
        let spec = literals(trimmed).into_iter().next().unwrap_or_default();
        let mut words = spec.split_whitespace();
        if words
            .next()
            .is_some_and(|w| w.eq_ignore_ascii_case("library"))
        {
            let name = words.next().unwrap_or_else(|| {
                panic!("{}: `EXTERNAL '{spec}'` names no library", path.display())
            });
            libraries.insert(name.to_string());
        }
    }
    for rest in after_each(source, "rxfuncadd") {
        let line = rest.lines().next().unwrap_or_default();
        let arguments = literals(line);
        let library = arguments.get(1).unwrap_or_else(|| {
            panic!(
                "{}: `rxfuncadd{line}` has no literal library argument",
                path.display()
            )
        });
        libraries.insert(library.to_string());
    }
    libraries
}

/// The files `loadPackage` names in `source`, beside `group`.
fn loaded_packages(source: &str, group: &Path) -> Vec<PathBuf> {
    after_each(source, "loadPackage(")
        .map(|rest| {
            let name = literals(rest.lines().next().unwrap_or_default())
                .into_iter()
                .next()
                .unwrap_or_else(|| panic!("{}: `loadPackage(` without a literal", group.display()));
            group.with_file_name(name)
        })
        .collect()
}

fn readelf(arguments: &[&str], path: &Path) -> String {
    let output = Command::new("readelf")
        .args(arguments)
        .arg(path)
        .output()
        .unwrap_or_else(|e| panic!("readelf could not be run: {e}"));
    assert!(
        output.status.success(),
        "readelf {arguments:?} {} failed: {}",
        path.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("readelf prints UTF-8")
}

/// The bracketed values of every dynamic entry tagged `tag`.
fn dynamic_entries(dynamic: &str, tag: &str) -> Vec<String> {
    dynamic
        .lines()
        .filter(|line| line.contains(tag))
        .filter_map(|line| {
            let open = line.find('[')?;
            let close = line.rfind(']')?;
            Some(line[open + 1..close].to_string())
        })
        .collect()
}

fn undefined_rexx_symbols(path: &Path) -> BTreeSet<String> {
    readelf(&["--dyn-syms", "-W"], path)
        .lines()
        .filter_map(|line| {
            let columns: Vec<&str> = line.split_whitespace().collect();
            (columns.len() >= 8 && columns[6] == "UND").then(|| columns[7])
        })
        .filter(|name| name.starts_with("Rexx"))
        .map(|name| name.split('@').next().unwrap_or(name).to_string())
        .collect()
}

fn resolve(needed: &str, runpath: &[String]) -> PathBuf {
    runpath
        .iter()
        .filter(|dir| !dir.is_empty())
        .map(PathBuf::from)
        .chain(std::iter::once(oracle_lib_dir()))
        .chain(SYSTEM_LIBRARY_DIRS.iter().map(PathBuf::from))
        .map(|dir| dir.join(needed))
        .find(|candidate| candidate.exists())
        .unwrap_or_else(|| panic!("NEEDED {needed} is in no searched directory"))
}

fn is_interpreter(file_name: &str) -> bool {
    INTERPRETER_LIBRARIES
        .iter()
        .any(|stem| file_name == *stem || file_name.starts_with(&format!("{stem}.")))
}

/// What a group's libraries reach: the interpreter libraries in their
/// `NEEDED` closure and the `Rexx*` symbols the rest of it imports.
#[derive(Default, Debug)]
struct Reach {
    interpreter: BTreeSet<String>,
    imports: BTreeSet<String>,
}

fn reach(libraries: &BTreeSet<String>, lib_dir: &Path) -> Reach {
    let mut reach = Reach::default();
    let mut pending: Vec<PathBuf> = libraries
        .iter()
        .map(|name| {
            let path = lib_dir.join(format!("lib{name}.so"));
            assert!(
                path.exists(),
                "library {name} has no build at {}",
                path.display()
            );
            path
        })
        .collect();
    let mut seen = BTreeSet::new();
    while let Some(path) = pending.pop() {
        if !seen.insert(path.clone()) {
            continue;
        }
        let dynamic = readelf(&["-d", "-W"], &path);
        let mut runpath = Vec::new();
        for entry in dynamic_entries(&dynamic, "(RUNPATH)")
            .iter()
            .chain(&dynamic_entries(&dynamic, "(RPATH)"))
        {
            runpath.extend(entry.split(':').map(str::to_string));
        }
        reach.imports.extend(undefined_rexx_symbols(&path));
        for needed in dynamic_entries(&dynamic, "(NEEDED)") {
            if is_interpreter(&needed) {
                reach.interpreter.insert(needed);
            } else {
                pending.push(resolve(&needed, &runpath));
            }
        }
    }
    reach
}

fn phase_of(group: &str, reach: &Reach) -> u8 {
    if reach.interpreter.is_empty() {
        assert!(
            reach.imports.is_empty(),
            "{group} needs no interpreter library but imports {:?}",
            reach.imports
        );
        return 8;
    }
    let mut phase = None;
    for import in &reach.imports {
        let class = IMPORT_PHASES
            .iter()
            .find(|(names, _)| names.contains(&import.as_str()));
        assert!(
            class.is_some() || NEUTRAL.contains(&import.as_str()),
            "{group} imports {import}, which this test does not classify"
        );
        phase = phase.max(class.map(|(_, phase)| *phase));
    }
    phase.unwrap_or_else(|| {
        panic!(
            "{group} needs {:?} but imports nothing that names a phase: {:?}",
            reach.interpreter, reach.imports
        )
    })
}

/// The derived partition of every `.testGroup` under `groups_dir`, with the
/// libraries each group binds.
fn derive(groups_dir: &Path) -> BTreeMap<String, (u8, BTreeSet<String>)> {
    let mut groups = Vec::new();
    group_files(groups_dir, &mut groups);
    assert!(
        !groups.is_empty(),
        "no .testGroup under {}",
        groups_dir.display()
    );
    groups
        .into_iter()
        .map(|group| {
            let name = group
                .file_stem()
                .expect("a group file name")
                .to_string_lossy()
                .into_owned();
            let source = read_source(&group);
            let mut libraries = bound_libraries(&source, &group);
            for package in loaded_packages(&source, &group) {
                libraries.extend(bound_libraries(&read_source(&package), &package));
            }
            assert!(!libraries.is_empty(), "{name} binds no library");
            let reach = reach(&libraries, &oracle_lib_dir());
            (name.clone(), (phase_of(&name, &reach), libraries))
        })
        .collect()
}

fn expected() -> BTreeMap<String, u8> {
    PARTITION
        .iter()
        .map(|(name, phase)| (name.to_string(), *phase))
        .collect()
}

fn phases(derived: &BTreeMap<String, (u8, BTreeSet<String>)>) -> BTreeMap<String, u8> {
    derived
        .iter()
        .map(|(name, (phase, _))| (name.clone(), *phase))
        .collect()
}

#[test]
fn each_api_group_belongs_to_the_phase_its_libraries_import_from() {
    let derived = derive(&api_groups_dir());
    assert_eq!(phases(&derived), expected(), "bindings: {derived:?}");
}

#[test]
fn a_group_whose_library_changes_changes_phase() {
    let scratch = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("api_group_partition-{}", std::process::id()));
    if scratch.exists() {
        fs::remove_dir_all(&scratch).expect("clearing the scratch copy");
    }
    copy_tree(&api_groups_dir(), &scratch);
    let package = scratch.join("oo/METHODPackage.cls");
    let source = read_source(&package);
    assert!(source.contains("LIBRARY orxmethod"));
    fs::write(
        &package,
        source.replace("LIBRARY orxmethod", "LIBRARY orxinvocation"),
    )
    .expect("rewriting the copy");

    let derived = phases(&derive(&scratch));
    fs::remove_dir_all(&scratch).expect("removing the scratch copy");
    let mut moved = expected();
    moved.insert("METHOD".to_string(), 10);
    assert_eq!(derived, moved);
}

fn copy_tree(from: &Path, to: &Path) {
    fs::create_dir_all(to).expect("creating the scratch copy");
    for entry in fs::read_dir(from).expect("reading the groups") {
        let entry = entry.expect("a directory entry");
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), &target).expect("copying a group file");
        }
    }
}
