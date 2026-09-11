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

//! The directive halves of `CoreClasses.orx` and `StreamClasses.orx`,
//! installed and then questioned, against the oracle.

mod support;

use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use support::oracle::{descriptor_diffs, did_not_finish};

/// Env var that flips this file from a skip into the check --
/// `tests/corpus.rs`'s own switch, for the reason
/// `tests/parse_version_oracle.rs` reuses it.
const GATE_ENV: &str = "REXX_CORPUS_GATE";

fn gate_mode() -> bool {
    match std::env::var(GATE_ENV) {
        Ok(value) => !value.is_empty() && value != "0",
        Err(_) => false,
    }
}

/// The questions, and each is here because a wrong answer to it means
/// something specific.
const QUESTIONS: &str = "\
say .TraceObject~option
say .String~hasMethod(\"DEFINECLASSMETHOD\")
say .Supplier~hasMethod(\"INHERITINSTANCEMETHODS\")
say .Class~hasMethod(\"DEFINE\")
say .Supplier~superClasses
say .Alarm~id
say .Comparable~id
say .Stream~id
say .File~id
say .TraceObject~id
exit 0
";

/// A directory of this run's own -- `tests/parse_version_oracle.rs`'s
/// `fresh_run_dir` has why.
fn fresh_run_dir() -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("the clock is after the epoch")
        .as_nanos();
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join(format!("bootstrap-install-{}-{nanos}", std::process::id()));
    fs::create_dir_all(&dir).unwrap_or_else(|e| panic!("cannot create {}: {e}", dir.display()));
    dir
}

/// One embedded file's directive section: everything from its first line
/// beginning `::` onwards.
fn directives_of(program: &rexx_lib::Program) -> &str {
    let text = std::str::from_utf8(program.source)
        .unwrap_or_else(|e| panic!("{} is not UTF-8: {e}", program.name));
    let start = text
        .lines()
        .position(|line| line.starts_with("::"))
        .unwrap_or_else(|| panic!("{} has no directive", program.name));
    let offset = text
        .split_inclusive('\n')
        .take(start)
        .map(str::len)
        .sum::<usize>();
    &text[offset..]
}

/// The questions, then both files' directives.
fn composed_program() -> String {
    let mut text = String::from(QUESTIONS);
    for name in ["CoreClasses.orx", "StreamClasses.orx"] {
        let program = rexx_lib::lookup(name).unwrap_or_else(|| panic!("{name} is embedded"));
        text.push('\n');
        text.push_str(directives_of(program));
    }
    text
}

/// Both files' directives install, and the questions answer what the oracle
/// answers on all three descriptors.
#[test]
fn the_two_library_files_install_and_answer_what_the_oracle_answers() {
    if !gate_mode() {
        eprintln!(
            "*** SKIPPED -- the library files' install is not compared against \
             the oracle unless {GATE_ENV} is set. ***"
        );
        return;
    }

    let dir = fresh_run_dir();
    let file = dir.join("bootstrap_install.orx");
    fs::write(&file, composed_program())
        .unwrap_or_else(|e| panic!("cannot write {}: {e}", file.display()));
    let abs = fs::canonicalize(&file)
        .unwrap_or_else(|e| panic!("cannot resolve {}: {e}", file.display()));
    let path = abs
        .to_str()
        .unwrap_or_else(|| panic!("probe path {} is not valid UTF-8", abs.display()));

    let oracle = support::oracle::locate();
    let cpp = oracle.run(&abs);
    assert!(
        !did_not_finish(&cpp),
        "the composed program's oracle run did not finish: {:?}",
        cpp.termination
    );

    let text = fs::read(&abs).unwrap_or_else(|e| panic!("cannot read {}: {e}", abs.display()));
    let rust = rexx_exec::run_program(path, text, rexx_exec::Invocation::none());
    let diffs = descriptor_diffs(&rust, &cpp);
    assert!(
        diffs.is_empty(),
        "the composed library install disagrees with the oracle on [{}], \
           rust stdout:   {:?}\n  oracle stdout: {:?}\n  \
         rust stderr:   {:?}\n  oracle stderr: {:?}\n  exit: rust {} oracle {}",
        diffs.join(", "),
        String::from_utf8_lossy(&rust.stdout),
        String::from_utf8_lossy(&cpp.stdout),
        String::from_utf8_lossy(&rust.stderr),
        String::from_utf8_lossy(&cpp.stderr),
        rust.exit_code,
        cpp.expect_exit_code()
    );

    assert_eq!(
        oracle.invocations(),
        1,
        "the comparison above must have actually started the oracle"
    );

    // Only on success: a failing run's file is the evidence.
    let _ = fs::remove_dir_all(&dir);
}

/// The composed program really is both files' directives and neither file's
/// prologue.
#[test]
fn the_composed_program_carries_both_files_directives_and_neither_prologue() {
    let text = composed_program();
    assert!(
        !text.contains("use arg rexxPackage"),
        "a prologue survived into the composed program"
    );
    assert!(
        !text.contains("call 'StreamClasses.orx'"),
        "CoreClasses' own CALL survived into the composed program"
    );
    for name in ["CoreClasses.orx", "StreamClasses.orx"] {
        let program = rexx_lib::lookup(name).unwrap_or_else(|| panic!("{name} is embedded"));
        let section = directives_of(program);
        assert!(
            section.starts_with("::"),
            "{name}'s directive section does not start at a directive"
        );
        assert!(
            text.contains(section),
            "{name}'s directive section is not in the composed program"
        );
    }
    // The one class each file declares that the other does not, so a
    // composition that dropped either file is red rather than green.
    assert!(text.contains("::class \"TraceObject\" subclass StringTable public"));
    assert!(text.contains("::CLASS \"File\" public inherit Comparable Orderable"));
}
