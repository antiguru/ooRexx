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
