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

//! Sending to a `LIBRARY REXX` entry point this phase does not implement is
//! loud and names the phase that owes it a body (D37).

mod gate_tables;
mod support;

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use gate_tables::run_gate_probe;
use rexx_exec::{NOT_IMPLEMENTED_EXIT, NativeEntryPoint};
use support::oracle::wrapped_exit_code;

/// Where the per-family programs live, relative to the corpus root.
const PROBE_SUBDIR: &str = "gate-tables/native-entries";

/// The line every program prints before its send.
const BEFORE_THE_SEND: &[u8] = b"main\n";

/// The entry points this phase implements rather than deferring, which the
/// plan names one by one: `StreamClasses.orx`'s `::CONSTANT separator` and
/// `::CONSTANT pathSeparator` send them while the package is still
/// installing, so a bootstrap that cannot run them cannot install that
/// file.
const IMPLEMENTED: &[&str] = &[
    "file_separator",
    "file_path_separator",
    // Phase 7's stream skeleton: everything a stream answers without opening
    // anything. The I/O entry points beside them are still deferred, which is
    // what the family's own probe binds.
    "qualify",
    "query_exists",
    "query_handle",
    "query_size",
    "query_streamtype",
    "query_time",
    "std_set",
    "stream_close",
    "stream_description",
    "stream_flush",
    "stream_init",
    "stream_open",
    "stream_state",
    "stream_uninit",
];

fn corpus_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus")
}

/// The entry point a program names, read out of its own text.
fn entry_point_named_by(text: &str, probe: &Path) -> String {
    let mut found: Vec<String> = Vec::new();
    for line in text
        .lines()
        .filter(|line| line.trim_start().starts_with("::"))
    {
        for rest in line.split("LIBRARY REXX ").skip(1) {
            let name: String = rest
                .chars()
                .take_while(|c| !c.is_whitespace() && *c != '\'' && *c != '"')
                .collect();
            found.push(name);
        }
    }
    assert_eq!(
        found.len(),
        1,
        "{} names {found:?} after `LIBRARY REXX ` on its directive clauses. A \
         program here binds exactly one entry point, because the row it is \
         checked against is that entry point's",
        probe.display()
    );
    found.pop().expect("the assertion above admits one")
}

/// The registry row for `entry`, or a failure naming it.
fn row_for(entry: &str) -> NativeEntryPoint {
    rexx_exec::native_entry_points()
        .into_iter()
        .find(|row| row.entry.eq_ignore_ascii_case(entry))
        .unwrap_or_else(|| {
            panic!(
                "the registry exports no entry point named {entry:?}, so a \
                 `::METHOD ... EXTERNAL 'LIBRARY REXX {entry}'` would be refused at \
                 install and the program below would never reach its send"
            )
        })
}

/// Every family the registry names has a program, and every program has a
/// family.
#[test]
fn every_family_has_a_program_and_every_program_has_a_family() {
    let families: BTreeSet<String> = rexx_exec::native_entry_points()
        .iter()
        .map(|row| row.family.to_string())
        .collect();
    assert!(
        !families.is_empty(),
        "the registry named no families at all, which is a defect in the \
         registry and not an empty pass"
    );
    let dir = corpus_dir().join(PROBE_SUBDIR);
    let on_disk: BTreeSet<String> = fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()))
        .flatten()
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter_map(|name| name.strip_suffix(".rex").map(str::to_string))
        .collect();
    assert_eq!(
        families,
        on_disk,
        "the families the registry names and the programs in {} are not the \
         same set",
        dir.display()
    );
}

/// One program per family: the bind succeeds, the program runs to its send,
/// and the send is loud and names the entry point and its owning phase.
#[test]
fn invoking_an_unimplemented_entry_point_is_loud_and_names_its_phase() {
    let dir = corpus_dir().join(PROBE_SUBDIR);
    let families: BTreeSet<String> = rexx_exec::native_entry_points()
        .iter()
        .map(|row| row.family.to_string())
        .collect();
    for family in &families {
        let probe = dir.join(format!("{family}.rex"));
        let text = fs::read_to_string(&probe)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", probe.display()));
        let entry = entry_point_named_by(&text, &probe);
        let row = row_for(&entry);
        assert_eq!(
            row.family,
            family.as_str(),
            "{} binds {entry:?}, which the registry files under the {:?} family",
            probe.display(),
            row.family
        );
        assert!(
            !row.implemented,
            "{} binds {entry:?}, which this phase implements -- so its send \
             answers and this program pins nothing. Point it at an entry point \
             of the same family that is still deferred",
            probe.display()
        );

        let abs = fs::canonicalize(&probe)
            .unwrap_or_else(|e| panic!("cannot resolve {}: {e}", probe.display()));
        let outcome = run_gate_probe(&abs);
        let expected = format!(
            "rexx-exec: the LIBRARY REXX entry point \"{}\" is not implemented ({})\n",
            row.entry, row.owner
        );
        assert_eq!(
            wrapped_exit_code(outcome.exit_code),
            NOT_IMPLEMENTED_EXIT,
            "{}: exit status, stderr {}",
            probe.display(),
            String::from_utf8_lossy(&outcome.stderr)
        );
        assert_eq!(
            outcome.stdout,
            BEFORE_THE_SEND,
            "{}: the line before the send. Empty means the directive was \
             refused at install instead, which is the regression this half \
             exists to catch",
            probe.display()
        );
        assert_eq!(
            String::from_utf8_lossy(&outcome.stderr),
            expected,
            "{}: the refusal",
            probe.display()
        );
    }
}

/// The entry points this phase implements are exactly the ones the plan
/// pulls forward from Phase 7.
#[test]
fn the_implemented_entry_points_are_the_ones_the_plan_names() {
    let implemented: BTreeSet<&str> = rexx_exec::native_entry_points()
        .iter()
        .filter(|row| row.implemented)
        .map(|row| row.entry)
        .collect();
    assert_eq!(
        implemented,
        IMPLEMENTED.iter().copied().collect::<BTreeSet<&str>>(),
        "the set of `LIBRARY REXX` entry points this phase runs drifted from \
         the scope addition the plan states"
    );
}

/// None of these programs is a corpus row.
#[test]
fn no_family_program_is_a_corpus_row() {
    let corpus = corpus_dir();
    let subset_files = fs::read_dir(&corpus)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", corpus.display()))
        .flatten()
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| name.starts_with("phase-") && name.ends_with(".txt"));
    for name in subset_files {
        let text = fs::read_to_string(corpus.join(&name))
            .unwrap_or_else(|e| panic!("cannot read {name}: {e}"));
        for line in text.lines() {
            let line = line.trim();
            assert!(
                line.starts_with('#') || !line.starts_with(PROBE_SUBDIR),
                "{name} lists {line}, and the oracle answers that program \
                 where this crate declines"
            );
        }
    }
}
