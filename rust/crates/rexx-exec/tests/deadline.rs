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

//! The crate-side watchdog: a run that does not terminate ends as a bounded,
//! distinguishable outcome instead of running for ever.

mod watchdog;

use std::time::{Duration, Instant};

use rexx_exec::{DEADLINE_EXIT, DEADLINE_REPORT, Invocation, Outcome, run_program};

/// A path for a program that was never on disk, matching what the other
/// in-process harnesses pass.
const PATH: &str = "<deadline test>";

/// A `REPLY` whose remainder forwards back into the method it is in.
const SELF_FORWARD: &[u8] =
    b"o = .K~new\no~m\nsay 'returned'\n::class K\n::method m\n  reply\n  forward message('M')\n";

/// Long enough that a machine under load cannot reach it by being slow, short
/// enough to keep this file's own runtime down. The corpus's slowest program
/// is two orders of magnitude below `watchdog::ROW_DEADLINE`, which is itself
/// far above this.
const SHORT: Duration = Duration::from_millis(500);

fn under_deadline(text: &[u8], limit: Duration) -> Outcome {
    run_program(PATH, text.to_vec(), Invocation::none().with_deadline(limit))
}

/// The witness: the self-forward ends at its deadline instead of running for
/// ever, on both engines, and says so.
#[test]
fn the_self_forward_ends_at_its_deadline_on_both_engines() {
    let started = Instant::now();
    let outcome = under_deadline(SELF_FORWARD, SHORT);
    let elapsed = started.elapsed();

    assert_eq!(
        outcome.exit_code,
        DEADLINE_EXIT,
        "stderr={:?}",
        String::from_utf8_lossy(&outcome.stderr)
    );
    assert_eq!(outcome.stderr, DEADLINE_REPORT);
    assert_eq!(outcome.stdout, b"returned\n");
    assert!(
        elapsed >= SHORT,
        "returned after {elapsed:?}, inside its own {SHORT:?} bound -- \
         something other than the deadline ended this run"
    );
    // Generous rather than tight: what is under test is "bounded at all".
    // The check is per `Deadline::CLAUSES_PER_CHECK` clauses, so the
    // overshoot is however long that many clauses take.
    assert!(
        elapsed < SHORT * 20,
        "took {elapsed:?} to come back from a {SHORT:?} bound"
    );
}

/// The neighbour that says which half of the program is doing the work: the
/// same shape with a *valued* `reply` terminates on its own, well inside a
/// deadline it therefore never reaches.
#[test]
fn a_valued_reply_ends_the_same_program() {
    let text = b"o = .K~new\no~m\nsay 'returned'\n::class K\n::method m\n  reply 1\n  forward message('M')\n";
    let outcome = under_deadline(text, SHORT);
    assert_ne!(
        outcome.exit_code,
        DEADLINE_EXIT,
        "a valued reply must not reach the deadline; stderr={:?}",
        String::from_utf8_lossy(&outcome.stderr)
    );
    assert_eq!(
        outcome.stdout,
        b"returned\n",
        "stderr={:?}",
        String::from_utf8_lossy(&outcome.stderr)
    );
}

/// A deadline that is never reached changes nothing, byte for byte.
#[test]
fn a_deadline_a_run_finishes_inside_changes_nothing() {
    let programs: &[&[u8]] = &[
        b"say 'a'\n",
        b"do i = 1 to 100; say i; end\n",
        b"say 1/0\n",
        b"o = .K~new\nsay o~m\n::class K\n::method m\n  reply 7\n  say 'rest'\n",
        b"signal on syntax name h\nsay 1/0\nh: say 'trapped' sigl\n",
    ];
    for text in programs {
        let bounded = under_deadline(text, Duration::from_secs(600));
        let plain = run_program(PATH, text.to_vec(), Invocation::none());
        assert_eq!(
            (bounded.exit_code, &bounded.stdout, &bounded.stderr),
            (plain.exit_code, &plain.stdout, &plain.stderr),
            "a deadline nothing reached moved the answer for {:?}",
            String::from_utf8_lossy(text)
        );
    }
}

/// The deadline is not a Rexx condition: no trap takes it, and the handler
/// that would have caught a real one never runs.
#[test]
fn no_trap_catches_the_deadline() {
    let programs: &[&[u8]] = &[
        b"signal on any name h\ndo forever; nop; end\nh: say 'trapped'\n",
        b"call on any name h\ndo forever; nop; end\nh: say 'trapped'\nreturn\n",
        b"signal on syntax name h\ndo forever; nop; end\nh: say 'trapped'\n",
    ];
    for text in programs {
        let outcome = under_deadline(text, SHORT);
        assert_eq!(
            outcome.exit_code,
            DEADLINE_EXIT,
            "{:?}",
            String::from_utf8_lossy(text)
        );
        assert_eq!(outcome.stdout, b"", "a handler ran");
        assert_eq!(outcome.stderr, DEADLINE_REPORT);
    }
}

/// The report names no error number and no condition, so nothing reading
/// stderr can classify it as a raised condition.
#[test]
fn the_report_is_not_a_condition() {
    let deadline = under_deadline(b"do forever; nop; end\n", SHORT);
    let raised = run_program(PATH, b"say 1/0\n".to_vec(), Invocation::none());

    let has_error_line = |bytes: &[u8]| {
        bytes
            .split(|byte| *byte == b'\n')
            .any(|line| line.starts_with(b"Error "))
    };
    assert!(has_error_line(&raised.stderr), "the control moved");
    assert!(!has_error_line(&deadline.stderr));
    assert!(!deadline.stderr.contains(&b'.'));
    assert_ne!(deadline.exit_code, raised.exit_code);
    // Outside the band a condition's `256 - major` occupies, which is what
    // stops a program expecting a condition from passing on a hang.
    assert!(!(157..=253).contains(&DEADLINE_EXIT));
}

/// A run that had already written to stderr is still read as one that did not
/// finish.
#[test]
fn a_run_that_traced_before_its_deadline_still_reads_as_unfinished() {
    let outcome = under_deadline(b"trace i\nnop\ntrace o\ndo forever; nop; end\n", SHORT);
    assert!(
        outcome.stderr.starts_with(b"     "),
        "the control moved: nothing was traced before the deadline, so this \
         says nothing about which end the report lands at; stderr={:?}",
        String::from_utf8_lossy(&outcome.stderr)
    );
    assert!(outcome.stderr.ends_with(DEADLINE_REPORT));
    assert!(watchdog::did_not_finish(&outcome));
    assert_eq!(outcome.exit_code, DEADLINE_EXIT);
}

/// A deadline fires even where the failure is on a path that discards
/// failures on purpose.
#[test]
fn a_finalizer_that_outlives_the_deadline_is_still_reported() {
    let looping = b"o = .K~new\ndrop o\ncall gc 'Force'\nsay 'after'\n\
                    ::class K\n::method uninit\n  do forever; nop; end\n";
    let control = b"o = .K~new\ndrop o\ncall gc 'Force'\nsay 'after'\n\
                    ::class K\n::method uninit\n  return\n";
    let outcome = under_deadline(looping, SHORT);
    assert_eq!(
        outcome.exit_code,
        DEADLINE_EXIT,
        "a deadline reached inside UNINIT was discarded; stdout={:?} stderr={:?}",
        String::from_utf8_lossy(&outcome.stdout),
        String::from_utf8_lossy(&outcome.stderr)
    );
    let control = under_deadline(control, Duration::from_secs(600));
    assert_eq!(
        control.exit_code,
        0,
        "the control must reach its own finalizer cleanly; stderr={:?}",
        String::from_utf8_lossy(&control.stderr)
    );
}

/// Every shape of unbounded run this crate can be made to take, bounded.
#[test]
fn every_unbounded_shape_this_crate_can_take_is_bounded() {
    let shapes: &[(&str, &[u8])] = &[
        ("do forever", b"do forever\n  nop\nend\n"),
        ("do forever, empty body", b"do forever\nend\n"),
        ("do while", b"do while 1 = 1\n  nop\nend\n"),
        ("do until", b"do until 1 = 0\n  nop\nend\n"),
        ("do by, no to", b"do zi = 1 by 1\n  nop\nend\n"),
        (
            "nested do forever",
            b"do forever\n  do forever\n    nop\n  end\nend\n",
        ),
        (
            "do forever with IF",
            b"do forever\n  if 1 = 1 then nop\nend\n",
        ),
        (
            "do forever with SELECT",
            b"do forever\n  select\n    when 1 = 1 then nop\n    otherwise nop\n  end\nend\n",
        ),
        (
            "do forever calling a label",
            b"do forever\n  call sub\nend\nsub: return\n",
        ),
        (
            "do forever with INTERPRET",
            b"do forever\n  interpret 'nop'\nend\n",
        ),
        ("INTERPRET of a loop", b"interpret 'do forever; nop; end'\n"),
        ("SIGNAL to its own label", b"zl:\nsignal zl\n"),
        ("SIGNAL loop through a label", b"zl:\nnop\nsignal zl\n"),
        (
            "a loop inside a method",
            b"say .K~new~m\n::class K\n::method m\n  do forever\n    nop\n  end\n",
        ),
        (
            "a loop inside a routine",
            b"call zr\n::routine zr\n  do forever\n    nop\n  end\n",
        ),
        (
            "a loop inside a replied body",
            b"o = .K~new\no~m\n::class K\n::method m\n  reply 1\n  do forever\n    nop\n  end\n",
        ),
        ("the self-forward reply drain", SELF_FORWARD),
        // **The row that says where the check has to be.** This recursion
        // iterates no loop, transfers by no `SIGNAL` and resumes no reply, so
        // a deadline honoured at those cycles instead of at the clause
        // boundary never sees it. Measured 2026-09-02, both engines: with the
        // check at the clause boundary it is `DEADLINE_EXIT` in under 0.5 s;
        // with the check moved to `loop_advance`, `Flow::Signal` and the
        // reply drain it overflows the interpreter thread's stack and the
        // process dies with SIGABRT. **So a regression here aborts this test
        // binary rather than failing it** -- that is what the shape does with
        // no bound, and `on_interpreter_thread`'s own doc says a stack
        // overflow is the one failure it cannot report.
        (
            "INTERPRET recursion",
            b"zs = 'interpret zs'\ninterpret zs\n",
        ),
    ];
    for (name, text) in shapes {
        let started = Instant::now();
        let outcome = watchdog::run_bounded_with(
            PATH,
            text.to_vec(),
            Invocation::none().with_deadline(SHORT),
            Duration::from_secs(30),
        );
        let elapsed = started.elapsed();
        assert_eq!(
            outcome.exit_code,
            DEADLINE_EXIT,
            "{name}: stdout={:?} stderr={:?}",
            String::from_utf8_lossy(&outcome.stdout),
            String::from_utf8_lossy(&outcome.stderr)
        );
        assert!(
            outcome.stderr.ends_with(DEADLINE_REPORT),
            "{name}: the outer bound answered, so the interpreter \
             does not see this shape: {:?}",
            String::from_utf8_lossy(&outcome.stderr)
        );
        assert!(elapsed < SHORT * 20, "{name}: took {elapsed:?}");
    }
}

/// Layer 2: a run layer 1 is not watching still comes back, and the answer
/// says which layer produced it.
#[test]
fn layer_two_returns_for_a_run_layer_one_is_not_watching() {
    let started = Instant::now();
    let outcome = watchdog::run_bounded_with(
        PATH,
        SELF_FORWARD.to_vec(),
        Invocation::none(),
        Duration::from_secs(2),
    );
    let elapsed = started.elapsed();

    assert!(watchdog::did_not_finish(&outcome));
    assert!(
        outcome.stderr.starts_with(b"rexx-watchdog: "),
        "layer 1 answered a run it was not given a deadline for: {:?}",
        String::from_utf8_lossy(&outcome.stderr)
    );
    assert_eq!(outcome.exit_code, DEADLINE_EXIT);
    assert!(elapsed >= Duration::from_secs(2) && elapsed < Duration::from_secs(30));
}

/// A sweep with the self-forward in it reddens that one row and completes.
#[test]
fn a_sweep_containing_the_self_forward_reddens_one_row_and_completes() {
    use rayon::prelude::*;

    let population: Vec<(&str, &[u8])> = vec![
        ("before", b"say 'before'\n"),
        ("hang", SELF_FORWARD),
        ("after", b"say 'after'\n"),
    ];
    let started = Instant::now();
    let rows: Vec<(&str, Outcome)> = population
        .par_iter()
        .map(|(name, text)| {
            (
                *name,
                watchdog::run_bounded_with(
                    PATH,
                    text.to_vec(),
                    Invocation::none().with_deadline(SHORT),
                    Duration::from_secs(30),
                ),
            )
        })
        .collect();
    let elapsed = started.elapsed();

    assert_eq!(rows.len(), 3);
    for (name, outcome) in &rows {
        match *name {
            "hang" => {
                assert_eq!(outcome.exit_code, DEADLINE_EXIT);
                assert_eq!(outcome.stderr, DEADLINE_REPORT);
            }
            other => {
                assert_eq!(outcome.exit_code, 0, "{other}");
                assert_eq!(outcome.stdout, format!("{other}\n").into_bytes(), "{other}");
            }
        }
    }
    assert!(
        elapsed < Duration::from_secs(30),
        "the sweep took {elapsed:?}: it did not complete at the deadline"
    );
}
