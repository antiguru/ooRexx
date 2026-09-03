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

//! `corpus/lang/directive_options*.rex` against the oracle, on both engines.
//!
//! **A binary of its own rather than lines in a phase subset file**, for the
//! reason `variable_reference.rs` states: `gate_table_c.rs`'s
//! `every_closed_phase_this_table_owns_rows_for_is_gated` makes a phase that
//! has a subset file owe every one of its table C rows an exit status, which
//! 5c cannot pay while its rows are still landing. Whoever closes 5c moves
//! these lines into `corpus/phase-5c.txt` and deletes this file.
//!
//! Three descriptors compared separately and raw, `directive_options_trace`
//! included: its `>I>`/`<I<` lines name the program's own path, which is the
//! same absolute path on both sides, so no normalisation is applied and none
//! is needed.

mod support;

use std::fs;
use std::path::{Path, PathBuf};

use rexx_exec::{Engine, Invocation, Outcome, run_program};
use support::oracle::locate;

/// The prefix every program this binary runs shares.
const PREFIX: &str = "directive_options";

/// The directory the programs live in, relative to the corpus root.
const SUBDIR: &str = "lang";

fn corpus_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus")
}

/// Every program this binary runs, read from the directory rather than
/// listed here.
///
/// **Derived so a program added later cannot be silently unrun**, which is
/// the same reason `corpus.rs`'s `phase_subset_files_on_disk` reads its
/// directory: a committed list edited in the same change as the file it
/// forgets would satisfy an assertion against itself.
fn programs() -> Vec<PathBuf> {
    let dir = corpus_dir().join(SUBDIR);
    let entries =
        fs::read_dir(&dir).unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()));
    let mut found: Vec<PathBuf> = entries
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.starts_with(PREFIX) && name.ends_with(".rex"))
        })
        .collect();
    found.sort();
    found
}

fn run_crate(path: &Path, engine: Engine) -> Outcome {
    let text = fs::read(path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
    let path_str = path
        .to_str()
        .unwrap_or_else(|| panic!("{} is not valid UTF-8", path.display()));
    run_program(path_str, text, Invocation::none().with_engine(engine))
}

/// Every `::OPTIONS` program answers the oracle byte for byte on stdout,
/// stderr and exit status, on both engines.
///
/// **Unconditional, not gated.** A `::OPTIONS` this crate parsed and then
/// ignored would leave `digits()` at 9 and an unset read answering its own
/// name -- rc 0 with empty stderr on both counts, so nothing else here would
/// go red for it.
#[test]
fn every_directive_options_program_answers_the_oracle() {
    let oracle = locate();
    let found = programs();
    assert!(
        !found.is_empty(),
        "no corpus/{SUBDIR}/{PREFIX}*.rex programs on disk, so this binary asserted nothing"
    );
    for path in found {
        let path = fs::canonicalize(&path)
            .unwrap_or_else(|e| panic!("cannot resolve {}: {e}", path.display()));
        let name = path.display().to_string();
        let cpp = oracle.run(&path);
        for engine in [Engine::Ir, Engine::TreeWalker] {
            let outcome = run_crate(&path, engine);
            assert_eq!(
                String::from_utf8_lossy(&outcome.stdout),
                String::from_utf8_lossy(&cpp.stdout),
                "{name}: stdout differs from the oracle on {engine:?}"
            );
            assert_eq!(
                String::from_utf8_lossy(&outcome.stderr),
                String::from_utf8_lossy(&cpp.stderr),
                "{name}: stderr differs from the oracle on {engine:?}"
            );
            assert_eq!(
                outcome.exit_code,
                cpp.expect_exit_code(),
                "{name}: exit status differs from the oracle on {engine:?}"
            );
        }
    }
}

/// **`LOSTDIGITS` is the one escalation with no raise site behind it, and
/// this pins what that costs.** This crate never detects that an arithmetic
/// operand carries more digits than the precision in force, so
/// `::OPTIONS LOSTDIGITS SYNTAX` is stored, never read, and the program below
/// answers rc 0 where the oracle is 98.972 -- a silent divergence, and the
/// only program shape this task moved from a loud refusal to a wrong answer.
///
/// The second row is the same gap reached without `::OPTIONS` at all and
/// predates this task: `SIGNAL ON LOSTDIGITS` parses, arms nothing, and the
/// handler never runs. Whoever implements the condition owns both rows and
/// deletes this test; until then a green run here is the record that the gap
/// is still open, and a red one says it closed.
#[test]
fn lostdigits_is_accepted_by_options_and_never_raised() {
    let oracle = locate();
    let cases: &[(&str, &str)] = &[
        (
            "options-escalation",
            "numeric digits 3\nsay 1.23456789 + 0\n::options lostdigits syntax\n",
        ),
        (
            "signal-on-lostdigits",
            "signal on lostdigits\nnumeric digits 3\nsay 1.23456789 + 0\nexit\n\
             lostdigits:\nsay 'trapped'\n",
        ),
    ];
    for (name, source) in cases {
        let path = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("lostdigits-{name}.rex"));
        fs::write(&path, source).unwrap_or_else(|e| panic!("cannot write {}: {e}", path.display()));
        let cpp = oracle.run(&path);
        for engine in [Engine::Ir, Engine::TreeWalker] {
            let outcome = run_crate(&path, engine);
            assert_eq!(
                outcome.exit_code, 0,
                "{name}: this crate raised on {engine:?}, so the gap this pins has closed \
                 and the row belongs deleted rather than kept"
            );
            assert_eq!(
                String::from_utf8_lossy(&outcome.stdout),
                "1.23\n",
                "{name}: this crate's answer on {engine:?}"
            );
            assert!(
                outcome.stderr.is_empty(),
                "{name}: this crate wrote on stderr on {engine:?}, so the divergence is no \
                 longer the silent one recorded here"
            );
        }
        // The oracle's own half, stated as "not what we answer" rather than
        // as bytes: the escalation row is 98.972 at rc 158 with no stdout and
        // the trap row is rc 0 printing `trapped`, so the two share no
        // expected transcript and what they do share is disagreeing with us.
        assert_ne!(
            String::from_utf8_lossy(&cpp.stdout),
            "1.23\n",
            "{name}: the oracle now answers what this crate does, so the gap closed on \
             its side and this row's record is stale"
        );
    }
}
