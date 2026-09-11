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

//! What the borrow-shape spike has to demonstrate, one test per claim.

use rexx_exec::{NOT_IMPLEMENTED_EXIT, run_program};

/// The path these tests report programs under.
const SPIKE_PATH: &str = "/nonexistent/spike-program.rex";

/// The whole of Step 1: an interpreter small enough to prove the shape and
/// nothing else.
#[test]
fn say_hello_prints_hello() {
    let outcome = run_program(
        SPIKE_PATH,
        b"say 'hello'\n".to_vec(),
        rexx_exec::Invocation::none(),
    );
    assert_eq!(outcome.stdout, b"hello\n");
    assert_eq!(outcome.stderr, b"");
    assert_eq!(outcome.exit_code, 0);
}

/// The variable pool, without a fragment anywhere near it: a value assigned to
/// a slot, read back out of it, and concatenated.
/// ```text
/// greeting = 'hello'
/// say greeting || ', world'
/// ```
#[test]
fn a_variable_round_trips_through_its_slot() {
    let outcome = run_program(
        SPIKE_PATH,
        b"greeting = 'hello'\nsay greeting || ', world'\n".to_vec(),
        rexx_exec::Invocation::none(),
    );
    assert_eq!(outcome.stdout, b"hello, world\n");
    assert_eq!(outcome.exit_code, 0);
}

/// An uninitialised read yields the derived name, upcased (D16).
#[test]
fn an_unset_variable_reads_as_its_own_name() {
    let outcome = run_program(
        SPIKE_PATH,
        b"say nosuchvariable\n".to_vec(),
        rexx_exec::Invocation::none(),
    );
    assert_eq!(outcome.stdout, b"NOSUCHVARIABLE\n");
    assert_eq!(outcome.exit_code, 0);
}

/// The loud-failure code is outside the band a Rexx error can produce, so a
/// differential run can never mistake an implementation gap for a condition.
#[test]
fn the_loud_failure_code_cannot_be_confused_with_a_rexx_error() {
    assert!(
        !(157..=253).contains(&NOT_IMPLEMENTED_EXIT),
        "256 - major lives in 157..=253 for majors 3 to 99"
    );
    let outcome = run_program(
        SPIKE_PATH,
        b"options 'x'\n".to_vec(),
        rexx_exec::Invocation::none(),
    );
    assert_eq!(outcome.exit_code, NOT_IMPLEMENTED_EXIT);
    assert_eq!(
        String::from_utf8(outcome.stderr).expect("the loud message is ASCII"),
        "rexx-exec: OPTIONS is not implemented (Phase 5)\n"
    );
}

/// A loud failure names the **form** and never formats the node, so the
/// message stays a constant size however big the expression behind it is.
#[test]
fn a_loud_failure_message_does_not_grow_with_the_expression() {
    const BOUND: usize = 300;

    let small = run_program(
        SPIKE_PATH,
        b"options 'x'\n".to_vec(),
        rexx_exec::Invocation::none(),
    );
    assert_eq!(small.exit_code, NOT_IMPLEMENTED_EXIT);

    // The same refusal over a three-thousand-term concatenation: the
    // instruction is what fails, and its operand is what a message formatting
    // the node would print.
    let mut deep = b"options ".to_vec();
    for _ in 0..3_000 {
        deep.extend_from_slice(b"'a' || ");
    }
    deep.extend_from_slice(b"'z'\n");
    let deep = run_program(SPIKE_PATH, deep, rexx_exec::Invocation::none());
    assert_eq!(deep.exit_code, NOT_IMPLEMENTED_EXIT);

    assert_eq!(
        small.stderr, deep.stderr,
        "the same unimplemented form should give the same message whatever is under it"
    );
    assert!(
        deep.stderr.len() < BOUND,
        "a loud message for a 3000-term expression was {} bytes, over the {BOUND}-byte bound",
        deep.stderr.len()
    );
    let stderr = String::from_utf8(deep.stderr).expect("the loud message is ASCII");
    assert!(
        stderr.contains("OPTIONS"),
        "the message should name the form it could not evaluate, and was {stderr:?}"
    );
}

/// Step 3's numbers, and the reason this test exists rather than a note in the
/// report: **Task 11 sets D19's evaluation-depth limit from what this prints**,
/// so it has to be re-runnable rather than a figure someone recorded once.
#[test]
fn records_the_stack_cost_of_one_eval_frame() {
    const TERMS: usize = 100_000;

    let mut program = b"say 'a'".to_vec();
    for term in 1..TERMS {
        // The empty string either way: only the expression's *shape* differs,
        // and one call is what puts the whole chain through `eval`.
        if term == 5 {
            program.extend_from_slice(b"||substr('x',1,0)");
        } else {
            program.extend_from_slice(b"||''");
        }
    }
    program.push(b'\n');

    let outcome = run_program(SPIKE_PATH, program, rexx_exec::Invocation::none());
    assert_eq!(outcome.exit_code, 0, "stderr: {:?}", outcome.stderr);
    assert_eq!(
        outcome.stdout, b"a\n",
        "the oracle prints a for this program"
    );

    assert_eq!(
        outcome.stack.max_depth, TERMS,
        "a left-deep chain of {TERMS} terms should recurse once per term"
    );
    let bytes_per_frame = outcome
        .stack
        .bytes_per_frame()
        .expect("the run recursed, so there is a per-frame cost");

    // Printed rather than only asserted: `cargo test -p rexx-exec -- --nocapture`
    // is how Task 11 reads the two numbers it needs.
    println!(
        "interpreter stack: {} bytes, eval depth reached: {}, span: {} bytes, \
         per frame: {bytes_per_frame:.1} bytes",
        rexx_exec::INTERPRETER_STACK_BYTES,
        outcome.stack.max_depth,
        outcome.stack.bytes,
    );

    // A loose sanity bound rather than a tight one, because the exact frame
    // size is a property of the compiler and the profile and will move. What
    // must not move is that the number is real: a zero would mean the probe
    // measured nothing.
    assert!(
        (16.0..=4096.0).contains(&bytes_per_frame),
        "measured {bytes_per_frame} bytes per eval frame, which is outside the range \
         INTERPRETER_STACK_BYTES was sized against"
    );

    // The two-sided bound D19 states, checked against the size actually
    // configured rather than against the comment describing it. The limit Task
    // 11 picks has to fit between 100,000 and this.
    let survivable = rexx_exec::INTERPRETER_STACK_BYTES as f64 / bytes_per_frame;
    assert!(
        survivable > 100_000.0,
        "the stack survives only {survivable:.0} eval frames, and D19 needs the depth \
         limit to be at least 100,000"
    );
}

/// The wiring between the instruction loop and `Raised::report`: which clause
/// the echo names, which line it resolves to, and the `256 - major` exit code.
/// ```text
/// a
/// b
///      3 *-* say 2 & 1
/// Error 34 running /abs/pin.rex line 3:  Logical value not 0 or 1.
/// Error 34.901:  Logical value must be exactly "0" or "1"; found "2".
/// ```
#[test]
fn a_raised_condition_reports_the_failing_clause() {
    let outcome = run_program(
        SPIKE_PATH,
        b"say 'a'\nsay 'b'\nsay 2 & 1\n".to_vec(),
        rexx_exec::Invocation::none(),
    );

    assert_eq!(outcome.stdout, b"a\nb\n");
    assert_eq!(outcome.exit_code, 256 - 34);
    assert_eq!(
        String::from_utf8(outcome.stderr).expect("the report is ASCII"),
        format!(
            "     3 *-* say 2 & 1\n\
             Error 34 running {SPIKE_PATH} line 3:  Logical value not 0 or 1.\n\
             Error 34.901:  Logical value must be exactly \"0\" or \"1\"; found \"2\".\n"
        )
    );
}

/// I6/D10: unbounded `CALL` recursion is a reportable 11.1 condition, not a
/// native abort.
#[test]
fn unbounded_call_recursion_raises_11_1_rather_than_overflowing() {
    let outcome = run_program(
        SPIKE_PATH,
        b"n = 0\ncall sub\nexit\nsub:\nn = n + 1\ncall sub\nreturn\n".to_vec(),
        rexx_exec::Invocation::none(),
    );

    assert_eq!(outcome.exit_code, 256 - 11);
    let stderr = String::from_utf8(outcome.stderr).expect("the report is ASCII");
    // The report's own two lines, verbatim from the oracle. Everything above
    // them is the echo stack, one `call sub` per activation the recursion
    // reached -- the same shape the oracle prints, at a different depth: it
    // echoes 27,318 lines where this echoes `MAX_ACTIVATION_DEPTH` of them.
    assert!(
        stderr.ends_with(&format!(
            "     2 *-* call sub\n\
             Error 11 running {SPIKE_PATH} line 6:  Control stack full.\n\
             Error 11.1:  Insufficient control stack space; cannot continue execution.\n"
        )),
        "expected the 11.1 report, got the last 200 bytes {:?}",
        &stderr[stderr.len().saturating_sub(200)..]
    );
    assert_eq!(
        stderr.lines().filter(|l| l.ends_with("call sub")).count(),
        10_000,
        "one echo per activation, and every one of them popped again on the \
         way out rather than the stack unwinding through a native abort"
    );
}
