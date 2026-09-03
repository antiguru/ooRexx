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

/// **`LOSTDIGITS` is the one escalation this crate cannot honour, so it
/// refuses the operand instead of answering it.**
///
/// The condition itself is unimplemented -- `SIGNAL ON LOSTDIGITS` arms
/// nothing -- so `::OPTIONS LOSTDIGITS SYNTAX` asks for a diagnosis nothing
/// here can produce. `Interp::lostdigits_check` answers that by refusing at
/// rc 120 wherever the oracle would have raised 98.972, which is loud where
/// answering the operation would be a wrong number.
///
/// **The rows that must NOT refuse are the point of the table**, not padding:
/// a gate that fired on the directive alone rather than on a real digit loss
/// would refuse `1 + 1`, and every `expect: Runs` row below is a program the
/// oracle answers cleanly.
///
/// Whoever implements the LOSTDIGITS condition deletes this test: every
/// `Refuses` row becomes an oracle-agreeing 98.972, and `signal-on-trap`
/// becomes the trap firing.
#[test]
fn lostdigits_is_refused_where_the_oracle_raises_and_nowhere_else() {
    #[derive(PartialEq, Debug)]
    enum Expect {
        /// rc 120 here; the oracle raises 98.972.
        Refuses,
        /// rc 0 on both sides with the same stdout -- the over-fire control.
        Agrees,
        /// Neither: a divergence this task did not close, named in the row.
        Diverges,
    }
    use Expect::{Agrees, Diverges, Refuses};
    let armed = "::options lostdigits syntax\n";
    let cases: &[(&str, &str, Expect)] = &[
        // Every operator whose operand the oracle checks.
        ("plus", "numeric digits 3\nsay 1.23456789 + 0\n", Refuses),
        (
            "plus-right",
            "numeric digits 3\nsay 0 + 1.23456789\n",
            Refuses,
        ),
        ("minus", "numeric digits 3\nsay 1.23456789 - 0\n", Refuses),
        ("times", "numeric digits 3\nsay 1.23456789 * 1\n", Refuses),
        ("divide", "numeric digits 3\nsay 1.23456789 / 1\n", Refuses),
        (
            "remainder",
            "numeric digits 3\nsay 1.23456789 // 1\n",
            Refuses,
        ),
        ("intdiv", "numeric digits 3\nsay 1.23456789 % 1\n", Refuses),
        (
            "power-base",
            "numeric digits 3\nsay 1.23456789 ** 1\n",
            Refuses,
        ),
        (
            "prefix-plus",
            "numeric digits 3\nsay +1.23456789\n",
            Refuses,
        ),
        (
            "prefix-minus",
            "numeric digits 3\nsay -1.23456789\n",
            Refuses,
        ),
        (
            "compare-eq",
            "numeric digits 3\nsay 1.23456789 = 1\n",
            Refuses,
        ),
        (
            "compare-gt",
            "numeric digits 3\nsay 123456789 > 1\n",
            Refuses,
        ),
        (
            "do-initial",
            "numeric digits 3\ndo k = 1.23456789 to 2\nleave\nend\n",
            Refuses,
        ),
        (
            "do-to",
            "numeric digits 3\ndo k = 1 to 123456789\nleave\nend\n",
            Refuses,
        ),
        (
            "do-by",
            "numeric digits 3\ndo k = 1 to 2 by 123456789\nleave\nend\n",
            Refuses,
        ),
        (
            "through-a-variable",
            "numeric digits 3\nz = 123456789\nsay z + 0\n",
            Refuses,
        ),
        (
            "through-an-argument",
            "numeric digits 3\ncall s 123456789\nexit\ns: use arg a\nsay a + 0\nreturn\n",
            Refuses,
        ),
        (
            "all-syntax-spelling",
            "numeric digits 3\nsay 1.23456789 + 0\n",
            Refuses,
        ),
        // The over-fire controls: the directive is armed and nothing is lost.
        ("nothing-lost", "numeric digits 3\nsay 1 + 1\n", Agrees),
        (
            "exactly-at-digits",
            "numeric digits 3\nsay 1.20 + 0\n",
            Agrees,
        ),
        ("default-precision", "say 1.23456789 + 0\n", Agrees),
        (
            "no-arithmetic",
            "numeric digits 3\nz = 1.23456789\nsay z\n",
            Agrees,
        ),
        // Positions the oracle does not check, measured one at a time.
        (
            "strict-compare",
            "numeric digits 3\nsay 1.23456789 == 1\n",
            Agrees,
        ),
        (
            "concatenation",
            "numeric digits 3\nsay 1.23456789 || 'a'\n",
            Agrees,
        ),
        (
            "string-compare",
            "numeric digits 3\nsay 1.23456789 = 'abc'\n",
            Agrees,
        ),
        ("abs", "numeric digits 3\nsay abs(1.23456789)\n", Agrees),
        ("trunc", "numeric digits 3\nsay trunc(1.23456789)\n", Agrees),
        (
            "format",
            "numeric digits 3\nsay format(1.23456789)\n",
            Agrees,
        ),
        ("max", "numeric digits 3\nsay max(1.23456789, 1)\n", Agrees),
        (
            "power-exponent",
            "numeric digits 3\nsay 2 ** 1234\n",
            Agrees,
        ),
        (
            "whole-number-argument",
            "numeric digits 3\nsay word('a b', 123456789)\n",
            Agrees,
        ),
        // `CONDITION` is the default spelling and turns the escalation off,
        // so it neither refuses nor diverges.
        (
            "condition-spelling",
            "numeric digits 3\nsay 1.23456789 + 0\n",
            Agrees,
        ),
        // The two this task did not close, each loud on both sides or silent
        // on both -- named so a change to either is visible here.
        //
        // `signal-on-trap`: the trap disables the escalation, so the check
        // declines and this crate answers the rounded number where the oracle
        // runs the handler. That is the LOSTDIGITS **condition** gap, which
        // predates `::OPTIONS` and is reachable with no directive at all.
        (
            "signal-on-trap",
            "numeric digits 3\nsignal on lostdigits\nsay 1.23456789 + 0\nexit\nlostdigits:\nsay 'trapped'\n",
            Diverges,
        ),
        // `do-repeat-count`: a bare `DO n` converts rather than computes, so
        // this crate answers 26.2 -- which is what BOTH sides answer without
        // the directive. The oracle's LOSTDIGITS preempts its own 26.2 and
        // ours does not: loud on both sides, different loud.
        (
            "do-repeat-count",
            "numeric digits 3\ndo 123456789\nleave\nend\n",
            Diverges,
        ),
    ];
    let oracle = locate();
    for (name, body, expect) in cases {
        let directive = match *name {
            "all-syntax-spelling" => "::options all syntax\n",
            "condition-spelling" => "::options lostdigits condition\n",
            _ => armed,
        };
        let source = format!("{body}say 'end'\n{directive}");
        let path = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("lostdigits-{name}.rex"));
        fs::write(&path, &source)
            .unwrap_or_else(|e| panic!("cannot write {}: {e}", path.display()));
        let cpp = oracle.run(&path);
        for engine in [Engine::Ir, Engine::TreeWalker] {
            let outcome = run_crate(&path, engine);
            let stderr = String::from_utf8_lossy(&outcome.stderr).into_owned();
            match expect {
                Refuses => {
                    assert_eq!(
                        outcome.exit_code, 120,
                        "{name} on {engine:?}: expected the loud refusal, got stderr {stderr}"
                    );
                    assert!(
                        stderr.contains("LOSTDIGITS"),
                        "{name} on {engine:?}: refused for some other reason: {stderr}"
                    );
                    assert!(
                        String::from_utf8_lossy(&cpp.stderr).contains("98.972"),
                        "{name}: the oracle no longer raises LOSTDIGITS here, so this row \
                         is refusing over nothing"
                    );
                }
                Agrees => {
                    assert_eq!(
                        String::from_utf8_lossy(&outcome.stdout),
                        String::from_utf8_lossy(&cpp.stdout),
                        "{name} on {engine:?}: stdout differs from the oracle"
                    );
                    assert_eq!(
                        String::from_utf8_lossy(&outcome.stderr),
                        String::from_utf8_lossy(&cpp.stderr),
                        "{name} on {engine:?}: stderr differs from the oracle"
                    );
                    assert_eq!(
                        outcome.exit_code,
                        cpp.expect_exit_code(),
                        "{name} on {engine:?}: exit status differs from the oracle"
                    );
                }
                Diverges => {
                    assert_ne!(
                        outcome.exit_code, 120,
                        "{name} on {engine:?}: this row records a divergence the refusal does \
                         not cover, and the refusal now covers it"
                    );
                    assert_ne!(
                        (
                            String::from_utf8_lossy(&outcome.stdout).into_owned(),
                            outcome.exit_code
                        ),
                        (
                            String::from_utf8_lossy(&cpp.stdout).into_owned(),
                            cpp.expect_exit_code()
                        ),
                        "{name} on {engine:?}: the two sides now agree, so this row's \
                         divergence closed and the row belongs deleted"
                    );
                }
            }
        }
    }
}
