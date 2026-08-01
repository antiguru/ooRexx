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

//! Criterion 3 (D17): `TRACE` output matches the oracle byte for byte, over a
//! committed table mapping each reachable prefix to a witness program.
//!
//! **This is the opposite strategy from `tests/corpus.rs`, deliberately.**
//! `corpus.rs` shells out to the oracle *live*, every run, because it exists
//! to track progress across tasks and a committed expectation would need
//! regenerating on every task that changes behaviour. A trace witness is a
//! fixed artefact instead, because its whole value is that it cannot drift
//! silently, and capturing it needs a running oracle that a machine checking
//! out this tree may not have. Neither is wrong; they answer different
//! questions, and `rexx-parse/tests/sourceline_oracle.rs` is this design's
//! own precedent -- unrelated to `TRACE` but the same "committed expectation,
//! regenerate by a driver script" shape.
//!
//! **The prefix table**, all 19 from `RexxActivation.hpp:90`-`110`, and which
//! witness (if any) below reaches it. Measured reachable from pure-4a code:
//! `*-*`, `>>>`, `>=>`, `>L>`, `>V>`, `>O>`, `>K>`, `>C>`, `>P>` -- the design
//! spec's own list, all nine covered here. `>E>` is *not* on that list but
//! is, in fact, reachable (`dotvariable_beyond_the_list.rex`, this task's own
//! report has the correction and the reasoning). The other nine
//! (`+++`/`>.>`/`>F>`/`>M>`/`>A>`/`>I>`/`>N>`/`>R>`/`<I<`) are 4b's or later
//! (command errors/failures, `PARSE`, message sends, function calls,
//! namespaces, aliasing, method/routine invocation) and have no witness here.
//!
//! | prefix | witness |
//! |---|---|
//! | `*-*` | every witness below |
//! | `>>>` | every witness below |
//! | `>=>` | `compound_read_write.rex` (and `trace_output.rex`'s simple form) |
//! | `>L>` | `trace_output.rex` |
//! | `>V>` | `trace_output.rex`, `compound_read_write.rex` |
//! | `>O>` | `trace_output.rex` |
//! | `>K>` | `keyword_while.rex` (`WHILE`) |
//! | `>C>` | `compound_read_write.rex` |
//! | `>P>` | `prefix_operators.rex` |
//! | `>E>` (bonus, not required) | `dotvariable_beyond_the_list.rex` |
//!
//! **This table used to be prose only, and a branch review (H3,
//! `branch-review-harness.md`) showed exactly what that cost**: replacing
//! `keyword_while.rex` with a straight-line program emitting no `>K>` at
//! all, regenerating its `.expected` from the live oracle with this file's
//! own documented recipe, still passed all five tests -- the byte-for-byte
//! check compares this crate's output to the committed file and nothing
//! else, so a witness that stopped witnessing its own prefix went
//! unnoticed. [`WITNESS_PREFIXES`] and
//! [`every_witness_still_emits_every_prefix_it_is_named_for`] turn the
//! table above into an assertion: each witness's committed `.expected`
//! stderr must contain every prefix this table claims for it, checked as a
//! byte substring, and the union across all five must be exactly the ten
//! prefixes claimed. A witness can still be swapped for a better one, but
//! not for one that silently covers less.
//!
//! **A known, disclosed gap, not a witness that avoids it.** A `Controlled`
//! (`TO`-style) `DO`/`LOOP`'s own re-tested pass traces two more `>>>` lines
//! (the control variable's pre- and post-increment value, `DoBlock::
//! checkControl`, `DoBlock.cpp:182`) that this crate does not yet produce --
//! `run.rs`'s own `loop_advance`, `LoopState::Controlled` arm, has the full
//! account of why. `keyword_while.rex` is chosen for `>K>` specifically
//! because it is a *complete* answer (a real repeating construct, re-echoing
//! its own clause every pass, `>K>` re-firing every pass, all measured and
//! matched) -- not because it dodges the gap, which lives in a different
//! `LoopKind` entirely. This task's report names the gap and the transcripts
//! that found it; it does not appear in this table because no witness here
//! reaches it, and adding one that does would fail this file's own test
//! rather than the phase's gate, which is not this task's call to make.
//!
//! **Regeneration.** Every `.expected` file was captured with:
//!
//! ```bash
//! ( ulimit -v 1048576; \
//!   LD_LIBRARY_PATH=/path/to/ooRexx/build/lib \
//!   /path/to/ooRexx/build/bin/rexx PROGRAM.rex ) \
//!   1>/tmp/out 2>/tmp/err; rc=$?
//! { echo "RC $rc"; echo "===STDOUT==="; cat /tmp/out; \
//!   echo "===STDERR==="; cat /tmp/err; } > PROGRAM.expected
//! ```
//!
//! `trace_output.rex` itself lives in `rust/corpus/lang/` (Task 14a's own
//! file, already a Phase 4a subset member) rather than being duplicated here
//! -- this test reads it from there by relative path, and only its own
//! `.expected` lives in this directory.

use rexx_exec::run_program;
use std::path::Path;

/// One witness's fixed oracle answer: exit code, stdout, stderr.
struct Expected {
    rc: i32,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
}

/// Parses an `.expected` file (`RC n` / `===STDOUT===` / bytes /
/// `===STDERR===` / bytes) -- a from-scratch format, not `sourceline_
/// oracle.rs`'s `count N` shape, because a witness needs three fields
/// (exit code, stdout, stderr) where a source-line expectation needs only
/// a count and lines. The two marker lines are chosen never to collide with
/// anything a trace prefix or Rexx `SAY` output could produce.
fn parse_expected(bytes: &[u8], path: &str) -> Expected {
    let text = std::str::from_utf8(bytes)
        .unwrap_or_else(|_| panic!("{path}: expectation file is not valid UTF-8"));
    let after_rc = text
        .strip_prefix("RC ")
        .unwrap_or_else(|| panic!("{path}: expectation file does not start with `RC `"));
    let (rc_text, rest) = after_rc
        .split_once('\n')
        .unwrap_or_else(|| panic!("{path}: no newline after the RC line"));
    let rc: i32 = rc_text
        .parse()
        .unwrap_or_else(|_| panic!("{path}: `{rc_text}` is not an exit code"));
    let rest = rest
        .strip_prefix("===STDOUT===\n")
        .unwrap_or_else(|| panic!("{path}: missing `===STDOUT===` marker"));
    let (stdout, rest) = rest
        .split_once("===STDERR===\n")
        .unwrap_or_else(|| panic!("{path}: missing `===STDERR===` marker"));
    Expected {
        rc,
        stdout: stdout.as_bytes().to_vec(),
        stderr: rest.as_bytes().to_vec(),
    }
}

/// Runs `program_path` through this crate's own public entry point and
/// asserts it against `<oracle_dir>/<name>.expected`, byte for byte on all
/// three of stdout, stderr and exit code -- never a substring or a loose
/// bound, matching criterion 2's own "byte for byte, never numerically"
/// rule for the same reason: a numeric or partial comparison here would
/// hide exactly the indentation and quoting divergences this task exists
/// to catch.
fn check_witness(name: &str, program_path: &Path) {
    let oracle_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/trace_oracle");
    let expected_path = oracle_dir.join(format!("{name}.expected"));
    let expected_bytes = std::fs::read(&expected_path).unwrap_or_else(|e| {
        panic!(
            "{}: no committed expectation for witness {name} ({e}); \
             regenerate per this file's own module comment",
            expected_path.display()
        )
    });
    let expected = parse_expected(&expected_bytes, name);

    let source = std::fs::read(program_path)
        .unwrap_or_else(|e| panic!("{}: unreadable ({e})", program_path.display()));
    let outcome = run_program(&program_path.to_string_lossy(), source);

    assert_eq!(outcome.stdout, expected.stdout, "{name}: stdout");
    assert_eq!(outcome.stderr, expected.stderr, "{name}: stderr");
    assert_eq!(outcome.exit_code, expected.rc, "{name}: exit code");
}

/// `>L>`/`>V>`/`>O>`/`>>>`/`>=>`/`*-*`: `TRACE I` over two assignments and an
/// `IF`/`THEN`/`SAY` -- the exact program that closed this task's own four
/// remaining corpus failures (`rust/corpus/phase-4a.txt`), read from
/// `rust/corpus/` rather than duplicated here.
#[test]
fn trace_output_covers_clause_result_assignment_literal_variable_and_operator() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus/lang/trace_output.rex");
    check_witness("trace_output", &path);
}

/// `>K>`: a `DO WHILE` loop, re-echoing its own clause and `END` every pass
/// and re-firing `>K> "WHILE"` every pass too -- a complete answer for a
/// real repeating construct, not a single-shot stand-in.
#[test]
fn keyword_while_covers_a_re_evaluated_keyword_across_every_pass() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/trace_oracle/keyword_while.rex");
    check_witness("keyword_while", &path);
}

/// `>C>`: a compound variable's own resolved name, announced before `>V>`
/// on a read and before `>=>` on a write, both under `TRACE I`.
#[test]
fn compound_read_write_covers_the_resolved_compound_name() {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/trace_oracle/compound_read_write.rex");
    check_witness("compound_read_write", &path);
}

/// `>P>`: the two prefix operators 4a implements, `+` and `\`.
#[test]
fn prefix_operators_covers_plus_and_backslash() {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/trace_oracle/prefix_operators.rex");
    check_witness("prefix_operators", &path);
}

/// `>E>`, a bonus beyond the design spec's own nine-prefix "measured
/// reachable from pure-4a code" list -- a correction this task found
/// (`.nil` is one of `ExprKind::DotVariable`'s own three 4a-admissible
/// names, D15, so it is reachable), not required by criterion 3's own
/// table but kept because it is real and cheap to pin.
#[test]
fn dotvariable_beyond_the_list_covers_the_spec_correction() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/trace_oracle/dotvariable_beyond_the_list.rex");
    check_witness("dotvariable_beyond_the_list", &path);
}

/// The module doc's own table, as data: which prefixes each witness is
/// claimed to cover. See the module doc's own note on why this exists --
/// found missing by a branch review (H3), which swapped `keyword_while.rex`
/// for a program with no `>K>` at all and watched every test stay green.
const WITNESS_PREFIXES: &[(&str, &[&str])] = &[
    ("trace_output", &["*-*", ">>>", ">=>", ">L>", ">V>", ">O>"]),
    ("keyword_while", &["*-*", ">>>", ">K>"]),
    (
        "compound_read_write",
        &["*-*", ">>>", ">=>", ">L>", ">V>", ">C>"],
    ),
    ("prefix_operators", &["*-*", ">>>", ">L>", ">P>"]),
    ("dotvariable_beyond_the_list", &["*-*", ">>>", ">E>"]),
];

/// The nine prefixes the design spec's own "measured reachable from pure-4a
/// code" list names, plus `>E>`, the one correction this task's report
/// records -- ten total. `WITNESS_PREFIXES`'s union must equal this set
/// exactly: not a subset (a prefix could otherwise be claimed by the module
/// doc's own table and never checked at all) and not a superset (a typo'd
/// prefix that no witness could ever really emit would go unnoticed
/// otherwise).
const CLAIMED_PREFIXES: &[&str] = &[
    "*-*", ">>>", ">=>", ">L>", ">V>", ">O>", ">K>", ">C>", ">P>", ">E>",
];

fn contains_bytes(haystack: &[u8], needle: &[u8]) -> bool {
    haystack.windows(needle.len().max(1)).any(|w| w == needle)
}

/// Turns the module doc's prefix-to-witness table from prose into a check.
/// For every witness, its committed `.expected` file's stderr must contain
/// every prefix `WITNESS_PREFIXES` claims for it, as a byte substring --
/// the same shape `check_witness` itself uses for the full comparison, at
/// a coarser grain. A witness that stops witnessing its own prefix (the
/// exact H3 attack: `keyword_while.rex` replaced with a straight-line
/// program, `.expected` regenerated from the live oracle, both still
/// "correct" in the sense that they agree with each other) now fails here
/// instead of passing silently.
#[test]
fn every_witness_still_emits_every_prefix_it_is_named_for() {
    let oracle_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/trace_oracle");
    let mut failures = String::new();
    let mut covered: Vec<&str> = Vec::new();

    for (name, prefixes) in WITNESS_PREFIXES {
        let expected_path = oracle_dir.join(format!("{name}.expected"));
        let bytes = std::fs::read(&expected_path)
            .unwrap_or_else(|e| panic!("{}: unreadable ({e})", expected_path.display()));
        let expected = parse_expected(&bytes, name);
        for prefix in *prefixes {
            if contains_bytes(&expected.stderr, prefix.as_bytes()) {
                covered.push(prefix);
            } else {
                use std::fmt::Write as _;
                writeln!(
                    failures,
                    "{name}: claimed to cover {prefix:?} but its committed \
                     `.expected` stderr does not contain it"
                )
                .unwrap();
            }
        }
    }
    assert!(
        failures.is_empty(),
        "a witness stopped witnessing a prefix this file's own table \
         claims for it:\n{failures}"
    );

    covered.sort_unstable();
    covered.dedup();
    let mut expected_union = CLAIMED_PREFIXES.to_vec();
    expected_union.sort_unstable();
    assert_eq!(
        covered, expected_union,
        "WITNESS_PREFIXES' union no longer matches CLAIMED_PREFIXES -- both \
         must be updated together, the module doc's own table with them"
    );
}
