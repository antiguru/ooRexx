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
//!
//! # The subject
//!
//! [`SELF_FORWARD`] is the one crate-side non-termination this project has
//! found. `corpus/oracle-crashes.txt`'s own entry for the self-forward family
//! carries the mechanism: the replied remainder is queued, and
//! `Interp::run_deferred_replies` drains a queue each drained body refills, so
//! no activation depth grows and `MAX_ACTIVATION_DEPTH` never fires. Measured
//! 2026-09-02 at `6656d5a4f` on the crate alone, both engines, launched
//! directly with `/proc/<pid>/stat` sampled over four seconds: 400 and 401
//! user ticks -- 100% of a core -- with `VmRSS` flat at 17,844 kB and 17,828
//! kB and the `rexx-run` thread parked in `futex_do_wait` joining. **A spin,
//! not a park.**
//!
//! **A valued `reply` does not reproduce it**, and this file's own
//! `a_valued_reply_ends_the_same_program` is that stated as a case rather than
//! as a warning: the point of it is that a reader who shortens the program
//! measures a different thing.
//!
//! **This file's programs are run against this crate only, never the
//! oracle.** The self-forward family is in `corpus/oracle-crashes.txt`
//! because it takes the oracle's C++ stack out at rc 139.
//!
//! # What is under test here and what is not
//!
//! Layer 1 -- the clause-boundary deadline -- and layer 2 -- `watchdog`'s
//! bound on the wait -- have different reach, and that module's own doc says
//! which catches what. Nothing here proves layer 2 against a run layer 1
//! genuinely cannot see: no such crate-side program is known, so layer 2 is
//! reached below by *withholding* a deadline rather than by a program that
//! outruns one.

mod watchdog;

use std::time::{Duration, Instant};

use rexx_exec::{DEADLINE_EXIT, DEADLINE_REPORT, Invocation, Outcome, run_program};

/// A path for a program that was never on disk, matching what the other
/// in-process harnesses pass.
const PATH: &str = "<deadline test>";

/// A `REPLY` whose remainder forwards back into the method it is in.
///
/// The bare `reply` is load-bearing -- see this module's doc.
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
///
/// The three descriptors are asserted separately, and `stdout` is the one
/// that says where the run got to: `say 'returned'` **does** run. `o~m`
/// answers at the `REPLY`, the main body finishes normally and prints, and
/// only then does `execute` reach the drain that never empties. So this
/// program's non-termination is entirely after its own last clause, which is
/// why a status alone would not have located it.
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
///
/// Measured by the controller before this file existed: 0.03 s. Kept as a
/// case because a reader shortening [`SELF_FORWARD`] to something easier to
/// read is exactly how the wrong reading was reached once already.
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
///
/// The pair is the point: the same programs run with and without a deadline
/// give identical descriptors, so the check cannot be perturbing a run that
/// finishes inside it. Without this, an implementation that reported every
/// run as abandoned would still pass the witness above.
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
///
/// Both trap forms, because they reach a failure by different routes --
/// `SIGNAL ON` through `Interp::offer_to_trap`, which declines everything
/// that is not a raised condition, and `CALL ON` through the pending-trap
/// queue, which a failure never enters at all. `ANY` rather than a named
/// condition so that nothing about which condition it is can be the reason it
/// does not fire.
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
///
/// A `Raised`'s own report echoes the failing clause and then names an
/// `Error <major>`; this echoes nothing and names no number at all. The
/// contrast is asserted against a program that really does raise, so the test
/// is comparing two live renderings rather than one rendering against a
/// description of the other.
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
///
/// `execute` appends its report **after** whatever the program wrote, so the
/// deadline line is at the end and not the start. The control is in the test:
/// the trace lines have to be there, or the two reads would agree for the
/// wrong reason. `trace o` bounds what the loop writes -- `trace i` left on
/// over `do forever` fills memory as fast as the machine can print.
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
///
/// `Interp::run_one_uninit` throws away a raised condition and an `EXIT`
/// because the oracle's own dispatcher does, so a finalizer that outlives the
/// deadline is the one place a run could have been abandoned and still
/// reported its main body's status. The control is the same program with a
/// finalizer that returns: it exits 0 and prints, so what this measures is
/// the loop and not the `UNINIT` machinery being broken.
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
///
/// **The table is the exhaustiveness claim, made checkable.** A deadline that
/// is honoured somewhere other than at every clause -- at a loop's own
/// advance, at a `SIGNAL` transfer, at the reply drain -- rests on "every
/// cycle passes through one of these sites", which is an assertion about this
/// interpreter's control flow and not something a type can carry. This runs
/// the shapes instead. It is worth having under the clause-boundary check
/// too, where it is cheap and says what the bound is for.
///
/// A row that hangs does not fail this test, it hangs it. That is what
/// `watchdog::run_bounded_with`'s outer bound is doing here: the row comes
/// back either way, and the assertion is on which.
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
///
/// **This is not a run layer 1 could not see** -- it is the self-forward with
/// no deadline set, which is the only way to reach this arm in bounded time,
/// since no crate-side program is known that layer 1 cannot see. Layer 2 is
/// therefore proved as a mechanism and remains untested against a real
/// subject of its own; `watchdog`'s module doc says so in the same terms.
///
/// The thread this abandons keeps running for the rest of this binary's life,
/// at 100% of a core.
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
///
/// The whole point of the two layers, and the shape a real gate has: the
/// population is run through `rayon`'s `collect`, which does not finish until
/// every element does. The two neighbours are what says the sweep completed
/// rather than merely returning -- their answers are asserted, so a `collect`
/// that gave up would fail here rather than pass with one row missing.
///
/// `run_bounded_with` at [`SHORT`] rather than `run_bounded` at
/// `watchdog::ROW_DEADLINE`, so that this costs half a second rather than a
/// minute; the durations are the only difference between them.
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
