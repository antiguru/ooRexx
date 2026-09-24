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

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use rexx_parse::DirectiveKind;

use super::*;

/// The interpreter's own bootstrap Rexx, at the paths the C++ tree keeps
/// it at.
fn bootstrap_files() -> Vec<PathBuf> {
    let root = PathBuf::from("/home/moritz/dev/repos/ooRexx/interpreter");
    vec![
        root.join("RexxClasses/CoreClasses.orx"),
        root.join("RexxClasses/StreamClasses.orx"),
        root.join("platform/unix/PlatformObjects.orx"),
    ]
}

/// Each bootstrap file's directives, parsed by this crate's own parser
/// rather than scanned.
fn bootstrap_directives() -> Vec<(PathBuf, rexx_parse::Program)> {
    bootstrap_files()
        .into_iter()
        .map(|path| {
            let text = std::fs::read(&path).unwrap_or_else(|e| {
                panic!(
                    "cannot read {}: {e}. This crate is checked against that C++ \
                     tree; without it there is nothing to resolve against",
                    path.display()
                )
            });
            let program = rexx_parse::parse_program(text)
                .unwrap_or_else(|e| panic!("{} does not parse: {e:?}", path.display()));
            (path, program)
        })
        .collect()
}

/// Every `EXTERNAL` the bootstrap files declare is the form this phase
/// binds, and every one of them resolves.
#[test]
fn every_bootstrap_external_binds_to_an_entry_point_this_registry_holds() {
    let mut reached = 0usize;
    for (path, program) in bootstrap_directives() {
        for directive in &program.directives {
            match &directive.kind {
                DirectiveKind::Method(method) => {
                    let Some(external) = method_external(method) else {
                        continue;
                    };
                    reached += 1;
                    check_binds(&path, "::METHOD", &external);
                }
                DirectiveKind::Attribute(attribute) => {
                    let Some(external) = attribute_external(attribute) else {
                        continue;
                    };
                    reached += 1;
                    check_binds(&path, "::ATTRIBUTE", &external);
                }
                // `::ROUTINE` carries an `EXTERNAL` too and this phase
                // refuses every spelling of it, so a bootstrap file
                // declaring one would be refused at install whatever this
                // registry holds. Asserted rather than written down,
                // because "the bootstrap declares no `::ROUTINE
                // EXTERNAL`" is a premise this registry's scope rests on.
                DirectiveKind::Routine(routine) => assert!(
                    routine.external.is_none(),
                    "{} declares a ::ROUTINE EXTERNAL, which this phase refuses",
                    path.display()
                ),
                _ => {}
            }
        }
    }
    assert!(
        reached > 0,
        "no bootstrap file declared a single ::METHOD EXTERNAL, so this test \
         compared nothing"
    );
}

/// One bootstrap directive's `EXTERNAL`: it names `LIBRARY REXX` and every
/// entry point it resolves against is one the registry holds. Panics with
/// what a reader needs to place the failure otherwise.
fn check_binds(path: &Path, keyword: &str, external: &MethodExternal) {
    let missing = match external {
        MethodExternal::LibraryRexx(entry) => entry.as_ref().err(),
        MethodExternal::Attribute(binds) => binds.iter().find_map(|bind| bind.entry.as_ref().err()),
        MethodExternal::OtherLibrary { library, .. } => panic!(
            "{} declares a {keyword} whose EXTERNAL names the library {:?}, which \
             is not the REXX package the bootstrap files are meant to bind against",
            path.display(),
            String::from_utf8_lossy(library)
        ),
    };
    if let Some(missing) = missing {
        panic!(
            "{} declares a {keyword} binding the entry point {:?}, which the REXX \
             package does not export -- so the whole file is refused at install",
            path.display(),
            String::from_utf8_lossy(missing)
        );
    }
}

/// The registry and the bootstrap files name the same entry points.
#[test]
fn the_bootstrap_files_and_the_registry_name_the_same_entry_points() {
    let mut declared: BTreeSet<String> = BTreeSet::new();
    for (_, program) in bootstrap_directives() {
        for directive in &program.directives {
            if let DirectiveKind::Method(method) = &directive.kind
                && let Some(MethodExternal::LibraryRexx(Ok(entry))) = method_external(method)
            {
                declared.insert(entry.name.to_ascii_lowercase());
            }
        }
    }
    let held: BTreeSet<String> = LIBRARY_REXX_METHODS
        .iter()
        .map(|entry| entry.name.to_ascii_lowercase())
        .collect();
    assert_eq!(
        declared, held,
        "the entry points the bootstrap files declare and the entry points \
         this registry holds are not the same set, compared caselessly"
    );
}

/// Every family has at least one entry point this phase defers, so a
/// program pinning that family's refusal can be written at all.
#[test]
fn every_deferred_entry_point_names_an_open_phase() {
    // The phases that still owe entry points, and no more: a fourth
    // spelling is either a typo or work nobody has been assigned, and a
    // phase that has closed cannot owe anything -- a refusal naming one
    // is a lie a program can read.
    const OPEN: &[&str] = &["Phase 6", "Phase 8", "Phase 10"];
    let mut deferred = 0usize;
    for entry in LIBRARY_REXX_METHODS {
        let ExternalBody::Deferred { owner } = entry.body else {
            continue;
        };
        deferred += 1;
        assert!(
            OPEN.contains(&owner),
            "{} defers to {owner:?}, which is not one of {OPEN:?}",
            entry.name
        );
    }
    assert!(
        deferred > 0,
        "the registry defers nothing at all, so this test is passing over an empty set"
    );
}

/// The lookup is caseless and the spelling the table carries is not what
/// makes a name resolve.
#[test]
fn an_entry_point_resolves_whatever_case_it_is_asked_for() {
    for spelling in [
        b"file_temporary_path".to_vec(),
        b"FILE_TEMPORARY_PATH".to_vec(),
        b"File_Temporary_Path".to_vec(),
    ] {
        let found = entry_point(&spelling)
            .unwrap_or_else(|| panic!("{} did not resolve", String::from_utf8_lossy(&spelling)));
        assert_eq!(found.name, "file_temporary_path");
    }
    assert!(entry_point(b"file_temporary_path ").is_none());
    assert!(entry_point(b"no_such_entry_point_xyz").is_none());
}

/// The `EXTERNAL` of the last directive of `source`, decided by whichever
/// half of the boundary owns that directive's keyword.
fn last_external(source: &str) -> MethodExternal {
    let program = rexx_parse::parse_program(source.as_bytes().to_vec())
        .unwrap_or_else(|e| panic!("{source:?} does not parse: {e:?}"));
    let last = program
        .directives
        .last()
        .unwrap_or_else(|| panic!("{source:?} declares no directive"));
    let external = match &last.kind {
        DirectiveKind::Method(method) => method_external(method),
        DirectiveKind::Attribute(attribute) => attribute_external(attribute),
        other => panic!("{source:?} ends with a ::{} directive", other.keyword()),
    };
    external.unwrap_or_else(|| panic!("{source:?} carries no EXTERNAL"))
}

/// Each accessor of `external`, as the dictionary key it fills paired with
/// the entry point it resolved or the procedure it did not.
fn accessors(external: &MethodExternal) -> Vec<(String, Result<&'static str, String>)> {
    let MethodExternal::Attribute(binds) = external else {
        panic!("not an attribute-shaped EXTERNAL");
    };
    binds
        .iter()
        .map(|bind| {
            let entry = match &bind.entry {
                Ok(entry) => Ok(entry.name),
                Err(procedure) => Err(String::from_utf8_lossy(procedure).into_owned()),
            };
            (String::from_utf8_lossy(&bind.key).into_owned(), entry)
        })
        .collect()
}

/// Which procedure each accessor of an attribute-shaped `EXTERNAL`
/// resolves against, per style and per third word.
#[test]
fn an_attribute_external_resolves_the_procedure_the_oracle_names() {
    let cases: &[(&str, &[(&str, Result<&str, &str>)])] = &[
        // The BOTH style prepends whatever the third word is, and the
        // getter's key is the attribute's own name.
        (
            "::class k\n::attribute at external 'LIBRARY REXX zzz_no_entry'\n",
            &[
                ("AT", Err("GETzzz_no_entry")),
                ("AT=", Err("SETzzz_no_entry")),
            ],
        ),
        // No third word, so the procedure is the upcased name and the
        // prefix goes in front of that.
        (
            "::class k\n::attribute at external 'LIBRARY REXX'\n",
            &[("AT", Err("GETAT")), ("AT=", Err("SETAT"))],
        ),
        // A GET or SET style takes an explicit third word unchanged.
        (
            "::class k\n::attribute at get external 'LIBRARY REXX zzz_no_entry'\n",
            &[("AT", Err("zzz_no_entry"))],
        ),
        (
            "::class k\n::attribute at set external 'LIBRARY REXX zzz_no_entry'\n",
            &[("AT=", Err("zzz_no_entry"))],
        ),
        // and prefixes only the default one, which a third word spelled
        // like the upcased name also is.
        (
            "::class k\n::attribute at get external 'LIBRARY REXX'\n",
            &[("AT", Err("GETAT"))],
        ),
        (
            "::class k\n::attribute at get external 'LIBRARY REXX AT'\n",
            &[("AT", Err("GETAT"))],
        ),
        (
            "::class k\n::attribute at set external 'LIBRARY REXX AT'\n",
            &[("AT=", Err("SETAT"))],
        ),
        // The comparison is on the spelling, so a third word differing
        // only in case is not the default and resolves unchanged.
        (
            "::class k\n::attribute file_separator get external \
             'LIBRARY REXX file_separator'\n",
            &[("FILE_SEPARATOR", Ok("file_separator"))],
        ),
        (
            "::class k\n::attribute file_separator get external \
             'LIBRARY REXX FILE_SEPARATOR'\n",
            &[("FILE_SEPARATOR", Err("GETFILE_SEPARATOR"))],
        ),
        // An entry point the package exports, reached through each style.
        (
            "::class k\n::attribute at get external 'LIBRARY REXX file_separator'\n",
            &[("AT", Ok("file_separator"))],
        ),
        (
            "::class k\n::attribute at set external 'LIBRARY REXX FILE_SEPARATOR'\n",
            &[("AT=", Ok("file_separator"))],
        ),
        // `::METHOD ... ATTRIBUTE` has no style to choose and prepends
        // like the BOTH one.
        (
            "::class k\n::method m attribute external 'LIBRARY REXX file_separator'\n",
            &[
                ("M", Err("GETfile_separator")),
                ("M=", Err("SETfile_separator")),
            ],
        ),
        (
            "::class k\n::method m attribute external 'LIBRARY REXX'\n",
            &[("M", Err("GETM")), ("M=", Err("SETM"))],
        ),
    ];
    for (source, expected) in cases {
        let expected: Vec<(String, Result<&str, String>)> = expected
            .iter()
            .map(|(key, entry)| ((*key).to_owned(), entry.map_err(str::to_owned)))
            .collect();
        assert_eq!(accessors(&last_external(source)), expected, "{source:?}");
    }
}

/// The library is tested before the accessors are resolved, so a
/// directive naming another library carries that library's own name and
/// never a procedure this registry could not find. The accessor spelling
/// is still the attribute shape's: measured, oracle, `::attribute at
/// external "LIBRARY rxregexp NoSuchGet"` is `90.998 Unable to find
/// external method "GETNoSuchGet".`
#[test]
fn a_library_other_than_rexx_names_itself_and_its_own_accessors() {
    let cases: &[(&str, &[&str])] = &[
        (
            "::class k\n::attribute at external 'LIBRARY zzznolib zzz'\n",
            &["GETzzz", "SETzzz"],
        ),
        (
            "::class k\n::attribute at get external 'LIBRARY zzznolib zzz'\n",
            &["zzz"],
        ),
        (
            "::class k\n::attribute at external 'LIBRARY zzznolib'\n",
            &["GETAT", "SETAT"],
        ),
        (
            "::class k\n::method m attribute external 'LIBRARY zzznolib zzz'\n",
            &["GETzzz", "SETzzz"],
        ),
        (
            "::class k\n::method m external 'LIBRARY zzznolib zzz'\n",
            &["zzz"],
        ),
    ];
    for (source, procedures) in cases {
        let MethodExternal::OtherLibrary { library, binds } = last_external(source) else {
            panic!("{source:?} did not name a library other than REXX");
        };
        assert_eq!(library, b"zzznolib".to_vec(), "{source:?}");
        let named: Vec<String> = binds
            .iter()
            .map(|bind| String::from_utf8_lossy(&bind.procedure).into_owned())
            .collect();
        assert_eq!(named, *procedures, "{source:?}");
    }
}
