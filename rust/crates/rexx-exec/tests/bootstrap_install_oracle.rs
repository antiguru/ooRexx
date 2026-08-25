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
//!
//! # Why a synthesised program and not a corpus row
//!
//! `corpus/` holds programs; these two files are read-only upstream sources
//! that this repository tracks under `interpreter/`, and neither is a program
//! a corpus row could name. `rexx-lib` embeds their bytes with a sha256 pin,
//! and this file composes the two directive sections into one program that
//! both interpreters can run. Nothing here holds a copy of what the files
//! say -- the program is built from the embedded bytes on every run, so an
//! upstream change reaches the comparison rather than being asserted against
//! a stale transcript.
//!
//! # What the composed program is
//!
//! Each file's own executable prologue is dropped and replaced by the
//! questions below, because **the prologue needs an argument neither
//! interpreter supplies to a program run from the command line**: measured,
//! `CoreClasses.orx` run directly is rc 159 on both, `Object "REXXPACKAGE"
//! does not understand message "ADDCLASS"`, since `use arg rexxPackage` with
//! nothing supplied leaves the symbol as its own name. What is left is every
//! `::` directive of both files, which is what installs.
//!
//! # What it can and cannot see
//!
//! It sees the install: every `::CLASS`, `::METHOD`, `::ATTRIBUTE`,
//! `::CONSTANT` and `::ANNOTATE` in both files, the `::METHOD ... EXTERNAL`
//! binds, the `::CONSTANT` expressions that run at install time, and the
//! `ACTIVATE` pass. `.TraceObject~option` is the question that separates a
//! run that made that last pass from one that did not, because
//! `TraceObject`'s own class-side `ACTIVATE` assigns it -- and the line
//! before the assignment is `self~activate:super`, a message scope override,
//! so this question also fails if that send stops working.
//!
//! **It cannot see the prologue**, which is the other half of the bootstrap:
//! `.queue~inherit(.OrderedCollection)`, `.supplier~inheritInstanceMethods`
//! and `.String~defineClassMethod` all run there. So the questions below are
//! deliberately restricted to state this program's own directives decide, or
//! that both sides agree on for their own reasons. Measured, and the reason
//! for the restriction: `.Queue~superClasses` is `The Object class` and `The
//! OrderedCollection class` on the oracle, whose own image ran that prologue
//! at start, and `The Object class` here, which has not.

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
///
/// A file's prologue runs before its first directive and cannot contain one,
/// so the first such line is where the executable half ends. What sits
/// between the prologue's `exit` and that line is comment banner text, which
/// this drops along with the prologue.
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
///
/// **What a degenerate implementation would do here**, since that is the test
/// this crate's own Method rules ask of an assertion: a build that refused
/// any directive in either file would exit 120 with a loud message on stderr,
/// which no oracle run produces; a build that installed everything and
/// skipped the `ACTIVATE` pass prints `OPTION` for the first question where
/// the oracle prints `N`; reversing that pass, swapping the two arms of
/// `Interp::receiver_has_scope`, and dropping `DefineClassMethod` from
/// `native_classes`' `REMOVED_BY_IMAGE_SAVE` each redden something. All four
/// were applied and run.
///
/// **None of the four is caught here and nowhere else**, and saying otherwise
/// would be the "can fail is not adds coverage" claim `rust/CLAUDE.md`
/// forbids: measured with this file moved out of `tests/`, the corpus
/// differential catches the first three and
/// `rexx-classes/tests/native_classes_wiring.rs` catches the fourth. What
/// this file does that nothing else in the tree does is run **the two
/// upstream files' own bytes** through both interpreters; every other
/// instrument for the same properties runs a program written here or compares
/// this crate against a second copy of itself.
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

    for engine in [rexx_exec::Engine::TreeWalker, rexx_exec::Engine::Ir] {
        let text = fs::read(&abs).unwrap_or_else(|e| panic!("cannot read {}: {e}", abs.display()));
        let rust = rexx_exec::run_program(
            path,
            text,
            rexx_exec::Invocation::none().with_engine(engine),
        );
        let diffs = descriptor_diffs(&rust, &cpp);
        assert!(
            diffs.is_empty(),
            "the composed library install disagrees with the oracle on [{}], \
             engine {engine:?}:\n  rust stdout:   {:?}\n  oracle stdout: {:?}\n  \
             rust stderr:   {:?}\n  oracle stderr: {:?}\n  exit: rust {} oracle {}",
            diffs.join(", "),
            String::from_utf8_lossy(&rust.stdout),
            String::from_utf8_lossy(&cpp.stdout),
            String::from_utf8_lossy(&rust.stderr),
            String::from_utf8_lossy(&cpp.stderr),
            rust.exit_code,
            cpp.expect_exit_code()
        );
    }

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
///
/// Without this, a `directives_of` that answered the empty string would leave
/// the differential above comparing two runs of the questions alone -- which
/// would still pass on the oracle, whose image already answers every one of
/// them, and would have installed nothing.
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
