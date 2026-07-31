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
//!
//! Every expected transcript here was captured from `build/bin/rexx` under
//! `( ulimit -v 1048576; … )` before the test was written, and the program
//! text is quoted verbatim beside it so the two can be re-run against each
//! other. These are not L0 corpus programs: they go through the library entry
//! point rather than through `rexx-run` and the diff harness, which is Task
//! 14's.

use rexx_exec::{NOT_IMPLEMENTED_EXIT, run_program, run_program_interpret_spike};

/// The path these tests report programs under.
///
/// They build programs from bytes rather than from files, so there is no real
/// path here to canonicalise. It reaches output in exactly one place, the
/// middle line of a raised condition's report, and
/// `a_raised_condition_reports_the_failing_clause` asserts it there verbatim.
///
/// What a made-up path deliberately does **not** check is the canonicalisation
/// itself: the oracle prints the absolute, dot-normalised path, and that
/// conversion lives in `rexx-run`, on the far side of this entry point. It is
/// covered by running real files through both interpreters, not from here, and
/// a test that asserted canonical output against a value it invented would be
/// checking its own arithmetic.
const SPIKE_PATH: &str = "/nonexistent/spike-program.rex";

/// The whole of Step 1: an interpreter small enough to prove the shape and
/// nothing else.
///
/// Oracle, `say 'hello'`: stdout `hello\n`, rc 0.
#[test]
fn say_hello_prints_hello() {
    let outcome = run_program(SPIKE_PATH, b"say 'hello'\n".to_vec());
    assert_eq!(outcome.stdout, b"hello\n");
    assert_eq!(outcome.stderr, b"");
    assert_eq!(outcome.exit_code, 0);
}

/// The variable pool, without a fragment anywhere near it: a value assigned to
/// a slot, read back out of it, and concatenated.
///
/// Oracle:
///
/// ```text
/// greeting = 'hello'
/// say greeting || ', world'
/// ```
///
/// gives `hello, world\n`, rc 0.
#[test]
fn a_variable_round_trips_through_its_slot() {
    let outcome = run_program(
        SPIKE_PATH,
        b"greeting = 'hello'\nsay greeting || ', world'\n".to_vec(),
    );
    assert_eq!(outcome.stdout, b"hello, world\n");
    assert_eq!(outcome.exit_code, 0);
}

/// An uninitialised read yields the derived name, upcased (D16).
///
/// Oracle, `say nosuchvariable`: stdout `NOSUCHVARIABLE\n`, rc 0.
#[test]
fn an_unset_variable_reads_as_its_own_name() {
    let outcome = run_program(SPIKE_PATH, b"say nosuchvariable\n".to_vec());
    assert_eq!(outcome.stdout, b"NOSUCHVARIABLE\n");
    assert_eq!(outcome.exit_code, 0);
}

/// Step 2, and the reason the spike exists in the shape it does.
///
/// Three separate things are being asserted by one transcript, and they are
/// listed here because a single `assert_eq!` hides which one broke:
///
/// 1. A fragment created mid-instruction **reads** a name the enclosing body
///    bound (`zzz`).
/// 2. A fragment **introduces** a name that appears in no instruction of the
///    enclosing body (`zork`), and a *later, separate* fragment reads it back.
///    This is the case that forces the enclosing activation to own a mutable
///    name-to-slot map, because the enclosing plan is an `Rc` and cannot be
///    extended.
/// 3. The enclosing body carries on afterwards with its own slots intact, and
///    a third fragment sees the updated value.
///
/// Oracle, verbatim:
///
/// ```text
/// zzz = 'from the enclosing frame'
/// interpret "say zzz"
/// interpret "zork = 42"
/// interpret "say zork"
/// zzz = zzz || '!'
/// interpret "say zzz"
/// ```
///
/// ```text
/// from the enclosing frame
/// 42
/// from the enclosing frame!
/// ```
///
/// rc 0.
#[test]
fn a_fragment_shares_the_enclosing_frames_variable_pool() {
    let program = b"zzz = 'from the enclosing frame'\n\
                    interpret \"say zzz\"\n\
                    interpret \"zork = 42\"\n\
                    interpret \"say zork\"\n\
                    zzz = zzz || '!'\n\
                    interpret \"say zzz\"\n";
    let outcome = run_program_interpret_spike(SPIKE_PATH, program.to_vec());
    assert_eq!(outcome.exit_code, 0, "stderr: {:?}", outcome.stderr);
    assert_eq!(
        outcome.stdout,
        b"from the enclosing frame\n42\nfrom the enclosing frame!\n"
    );
}

/// `EXIT` inside a fragment ends the *program*, not the fragment, so control
/// leaves the nested loop and the enclosing one together and both `Rc` locals
/// drop in order.
///
/// Oracle, verbatim:
///
/// ```text
/// say 'before'
/// interpret "say 'inside'"
/// interpret "exit"
/// say 'after'
/// ```
///
/// gives `before\ninside\n`, rc 0. `after` is not printed.
#[test]
fn an_exit_inside_a_fragment_ends_the_program() {
    let program = b"say 'before'\n\
                    interpret \"say 'inside'\"\n\
                    interpret \"exit\"\n\
                    say 'after'\n";
    let outcome = run_program_interpret_spike(SPIKE_PATH, program.to_vec());
    assert_eq!(outcome.exit_code, 0, "stderr: {:?}", outcome.stderr);
    assert_eq!(outcome.stdout, b"before\ninside\n");
}

/// The other half of Step 2: 4a builds the machinery, 4b builds the keyword.
///
/// The same program the previous two tests run through the spike entry point
/// must still fail loudly through the real one, or 4a would be shipping an
/// `INTERPRET` that 4b has not written and the split table says it does not
/// have.
#[test]
fn the_interpret_keyword_still_fails_loudly() {
    let program = b"interpret \"say 'reached'\"\n";
    let outcome = run_program(SPIKE_PATH, program.to_vec());
    assert_eq!(outcome.exit_code, NOT_IMPLEMENTED_EXIT);
    assert_eq!(outcome.stdout, b"", "the fragment must not have run");
    let stderr = String::from_utf8(outcome.stderr).expect("the loud message is ASCII");
    assert!(stderr.contains("INTERPRET"), "stderr was {stderr:?}");
}

/// The loud-failure code is outside the band a Rexx error can produce, so a
/// differential run can never mistake an implementation gap for a condition.
///
/// **The probe construct changed under Task 11.** This used to run `do i =
/// 1 to 3 / end`, which was true of every task before Task 11 and stopped
/// being true the moment `DO`/`LOOP` were implemented -- that program now
/// runs to completion instead of failing loudly, and the test would have
/// started asserting `0 == NOT_IMPLEMENTED_EXIT` and failed for a reason
/// that has nothing to do with what it is meant to check. `CALL` is 4b's,
/// still not implemented through `run_program`, and is the new probe.
#[test]
fn the_loud_failure_code_cannot_be_confused_with_a_rexx_error() {
    assert!(
        !(157..=253).contains(&NOT_IMPLEMENTED_EXIT),
        "256 - major lives in 157..=253 for majors 3 to 99"
    );
    let outcome = run_program(SPIKE_PATH, b"call \"sub\"\n".to_vec());
    assert_eq!(outcome.exit_code, NOT_IMPLEMENTED_EXIT);
    let stderr = String::from_utf8(outcome.stderr).expect("the loud message is ASCII");
    assert!(stderr.contains("CALL"), "stderr was {stderr:?}");
}

/// A loud failure names the **form** and never formats the node, so the
/// message stays a constant size however big the expression behind it is.
///
/// This is a regression test for a real defect and not a tidiness rule. The
/// first version of `Loud::expression` wrote `{kind:?}`, and `ExprKind`'s
/// derived `Debug` walks the whole tree: one clause of
/// `corpus/lang/deep_nested_expr.rex` produced **373,332 bytes** of stderr
/// beginning `Binary { op: Plus, left: Expr { kind: Binary { op: Plus, left:
/// ...`. Failing loudly is a gate criterion and every later task inherits this
/// path, and criterion 1 compares stderr byte for byte, so an unbounded
/// message is an unbounded diff.
///
/// The bound is deliberately loose. What must hold is that the message does
/// not grow with the tree, which is why the same assertion runs against two
/// expressions three orders of magnitude apart in size.
///
/// **What this test establishes on its own is narrower than the contract, and
/// the rest is a compile-time property rather than a tested one.** Both
/// programs here use `~`, so this pins the message for exactly one of the
/// fifteen `ExprKind` forms, and the unit test named below pins two more. What
/// extends it to the remaining twelve is `form_name`'s match: it is exhaustive
/// with no `_` arm, and every arm returns a `&'static str` or one
/// `Operator::spelling`, so a form whose message could grow with its input
/// cannot be added without failing to compile. Auditing coverage from this
/// test alone would over-read it.
///
/// The other two loud paths are bounded for their own reasons, checked in
/// review rather than here, so the size contract holds on all three:
/// `Loud::instruction` returns `keyword()`'s `&'static str` or one of four
/// literals, and `Loud::parse` renders a `ParseError`, which deliberately
/// carries no substitution values.
///
/// **A message send, and the choice of witness is the point.** This test needs
/// a form the executor does not evaluate, and it has now been broken twice by
/// its own witness being implemented underneath it: it began on `+`, moved to
/// `=` when Task 7 landed arithmetic, and went red again when Task 8 landed
/// comparison. Picking a third operator would only schedule a third break --
/// *every* operator, prefix and dyadic, is inside Phase 4a. `~` is Phase 5's
/// object model, so no task in this plan can take it away, and the message it
/// produces is `form_name`'s `&'static "a message send"`.
///
/// The cost of moving off an operator is that this test no longer reaches
/// either of the two arms that call `format!`, which are the only two where
/// the size contract could actually break. That coverage did not go away with
/// it: `lib.rs`'s `the_two_formatting_arms_do_not_grow_with_the_subtree` calls
/// `form_name` directly on a 200-deep `Binary` and `Prefix`, which needs no
/// unimplemented form and so cannot be broken by a later task either.
#[test]
fn a_loud_failure_message_does_not_grow_with_the_expression() {
    const BOUND: usize = 300;

    let small = run_program(SPIKE_PATH, b"say 1~a\n".to_vec());
    assert_eq!(small.exit_code, NOT_IMPLEMENTED_EXIT);

    let mut deep = b"say 1".to_vec();
    for _ in 0..3_000 {
        deep.extend_from_slice(b"~a");
    }
    deep.push(b'\n');
    let deep = run_program(SPIKE_PATH, deep);
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
        stderr.contains("a message send"),
        "the message should name the form it could not evaluate, and was {stderr:?}"
    );
}

/// The reported span comes from one call chain, so what else the program
/// evaluated cannot change it.
///
/// A regression test for a defect the review found by measuring rather than by
/// reading. `eval` records a depth-1 address and a deepest address; if the
/// first is rewritten by *every* top-level evaluation while the second moves
/// only on a new maximum, the two can end up describing different chains, and
/// the frames above `eval` then no longer cancel. A fragment's evaluation runs
/// under `run_fragment` under `step` under the enclosing `eval`, about 2 KB
/// deeper than a top-level one, so appending one `INTERPRET` to a program was
/// enough: measured before the fix, this pair reported **784.0** and
/// **782.158**, and the second is the dangerous direction, since a smaller
/// per-level cost implies more survivable levels than there are.
///
/// The assertion is equality of the two spans rather than a bound on either,
/// because the property is "the span does not depend on what else ran" and a
/// bound would pass for both the fixed and the broken version.
#[test]
fn the_stack_span_does_not_depend_on_what_else_the_program_evaluated() {
    let mut alone = b"say 'a'".to_vec();
    for _ in 1..1_000 {
        alone.extend_from_slice(b"||''");
    }
    alone.push(b'\n');

    let mut then_a_fragment = alone.clone();
    then_a_fragment.extend_from_slice(b"interpret \"say 'b'\"\n");

    let alone = run_program(SPIKE_PATH, alone);
    let then_a_fragment = run_program_interpret_spike(SPIKE_PATH, then_a_fragment);

    assert_eq!(alone.exit_code, 0, "stderr: {:?}", alone.stderr);
    assert_eq!(
        then_a_fragment.exit_code, 0,
        "stderr: {:?}",
        then_a_fragment.stderr
    );
    assert_eq!(
        alone.stack.max_depth, then_a_fragment.stack.max_depth,
        "the fragment's own evaluation is shallow, so it must not move the maximum"
    );
    assert_eq!(
        alone.stack.bytes, then_a_fragment.stack.bytes,
        "the span must come from the chain that reached the maximum, not from the last \
         top-level evaluation to start"
    );
}

/// Step 3's numbers, and the reason this test exists rather than a note in the
/// report: **Task 11 sets D19's evaluation-depth limit from what this prints**,
/// so it has to be re-runnable rather than a figure someone recorded once.
///
/// The program is the concatenation analogue of D19's `1 + 1 + … + 1`: a
/// left-deep chain of 100,000 terms, which is exactly the depth the oracle
/// handles and exits 0 at. Measured on the oracle, `say 'a'` followed by
/// 99,999 repetitions of `||''`: stdout `a\n`, rc 0.
///
/// Three recursions ride on the same stack here and the test covers all three
/// at once, because it parses, plans, evaluates and then drops the tree:
/// `Plan::note`, `eval`, and `Drop` for the `Box<Expr>` chain.
///
/// **Task 11 itself changed what this test can still measure, and this is
/// the paragraph recording that rather than letting the test quietly stop
/// witnessing what its own name claims.** `eval.rs`'s `MAX_EVAL_DEPTH` is
/// now 100,000, enforced inside `eval` itself, so `run_program` -- the only
/// way anything outside this crate reaches a sized stack at all -- refuses
/// to recurse past it. This test's own chain is exactly 100,000 terms, at
/// or below the limit, so it still runs for real and still measures a real
/// `bytes_per_frame` at a real depth; nothing about *this* assertion became
/// vacuous. What is gone is the ability to go past 100,000 *through this
/// same public entry point* to independently re-confirm the `survivable`
/// extrapolation below against an actual guard-page abort, the way the
/// external `rexx-run` bisection this file's sibling doc comments describe
/// once did (`lib.rs`'s own `INTERPRETER_STACK_BYTES` doc comment records
/// the same consequence, and how to deliberately get past it again: raise
/// `MAX_EVAL_DEPTH`, or call `eval` directly bypassing `run_program`, for
/// the duration of a fresh bisection only, and put it back).
///
/// **The re-measured figure this test prints, recorded here too and not
/// only in the report file**: at Task 11's own implementation, this test
/// prints `per frame: 1840.0 bytes` (up from the ~1600 `lib.rs`'s own doc
/// comment on `INTERPRETER_STACK_BYTES` recorded after Task 7), giving
/// `survivable ≈ 291,777` -- comfortably over the `> 100_000.0` the
/// assertion below checks, still by about 2.9x. `lib.rs`'s own doc comment
/// on the constant carries the dated row; this comment exists so `grep`
/// for the figure finds it beside the test that produces it too, not only
/// in a report a later reader may never open.
#[test]
fn records_the_stack_cost_of_one_eval_frame() {
    const TERMS: usize = 100_000;

    let mut program = b"say 'a'".to_vec();
    for _ in 1..TERMS {
        program.extend_from_slice(b"||''");
    }
    program.push(b'\n');

    let outcome = run_program(SPIKE_PATH, program);
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
    //
    // Raised from 1024 to 4096 after Task 7: `eval_node` grew from four
    // match arms to fifteen (Stem, Compound, DotVariable, Prefix, seven
    // arithmetic operators, Abuttal/Blank), and the measured cost moved from
    // 784 to 1600 bytes even on this test's own unchanged `||`-only stress
    // program -- see `INTERPRETER_STACK_BYTES`'s doc comment for the
    // re-measurement and why 512 MiB stays as it is (still over three times
    // D19's 100,000-level minimum at the new cost). The real safety check is
    // the `survivable` assertion below, computed from whatever this run
    // measured rather than a constant; this bound exists only to catch the
    // probe measuring nothing, or a genuine blow-up, not to pin an exact
    // figure that every later task's new `ExprKind` arms will keep moving.
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
///
/// `error.rs`'s own tests pin the report's *format* against captured oracle
/// bytes. None of them can pin the three things that only exist once a program
/// actually runs, which is what this covers: that the clause echoed is the one
/// that failed rather than the first or the last, that its line survives
/// `run`'s teardown of the activation it came from, and that the exit code
/// stops being `NOT_IMPLEMENTED_EXIT`.
///
/// Oracle, with the same three lines in a file at an absolute path (rc 222):
///
/// ```text
/// a
/// b
///      3 *-* say 2 & 1
/// Error 34 running /abs/pin.rex line 3:  Logical value not 0 or 1.
/// Error 34.901:  Logical value must be exactly "0" or "1"; found "2".
/// ```
///
/// `say 'a'` and `say 'b'` are there to make the line number wrong in a
/// noticeable way if the site were taken from the first instruction or from
/// the activation's `pc` after teardown, both of which would still produce a
/// well-formed report.
#[test]
fn a_raised_condition_reports_the_failing_clause() {
    let outcome = run_program(SPIKE_PATH, b"say 'a'\nsay 'b'\nsay 2 & 1\n".to_vec());

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
