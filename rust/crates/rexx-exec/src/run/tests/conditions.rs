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

use super::*;

// ---- condition traps, RAISE and NOVALUE ----

/// The base case, and the one every other test here is a variation of.
#[test]
fn a_signal_on_syntax_trap_runs_its_handler_and_sets_sigl_to_the_raising_clause() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"signal on syntax\nzwitness = 'BEFORE'\nsay 1/0\nexit\nsyntax:\nzwitness = 'TRAPPED'\nsay zwitness sigl\n",
        ),
        b"TRAPPED 3\n".to_vec()
    );
}

/// The adjacent success for the test above: the identical program with
/// the `SIGNAL ON` clause removed is the ordinary fatal 42.3, so the
/// handler's output in that test is caused by the trap and not by
/// anything else in the program.
#[test]
fn the_same_program_without_the_trap_is_the_ordinary_fatal_condition() {
    let mut interp = Interp::new();
    let failure = run_source(
        &mut interp,
        b"zwitness = 'BEFORE'\nsay 1/0\nexit\nsyntax:\nzwitness = 'TRAPPED'\nsay zwitness sigl\n",
    )
    .unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (42, 3));
    assert!(
        interp.out.is_empty(),
        "the handler must not have run: {:?}",
        String::from_utf8_lossy(&interp.out)
    );
}

/// `SIGNAL OFF` really removes the trap, and the pair is what pins it to
/// `SIGNAL OFF` rather than to anything about where the raise sits.
#[test]
fn signal_off_removes_a_trap_that_signal_on_had_enabled() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"signal on syntax\nsignal on novalue\nsignal off syntax\nsay zunset\nexit\nsyntax:\nsay 'SYNTAX-HANDLER'\nexit\nnovalue:\nsay 'NOVALUE-HANDLER'\n",
        ),
        b"NOVALUE-HANDLER\n".to_vec(),
        "NOVALUE is still enabled and SYNTAX is not, so the read traps \
         and nothing reaches the SYNTAX handler"
    );

    let mut interp = Interp::new();
    let failure = run_source(
        &mut interp,
        b"signal on syntax\nsignal off syntax\nsay 1/0\nexit\nsyntax:\nsay 'SYNTAX-HANDLER'\n",
    )
    .unwrap_err();
    assert!(
        matches!(&failure, Failure::Raised(raised) if raised.number == 42),
        "after SIGNAL OFF the same raise is fatal, got {failure:?}"
    );
}

/// The trap that fired is gone from the table by the time the handler
/// runs.
#[test]
fn the_trap_that_fired_is_removed_from_the_table() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"signal on syntax\nsignal on novalue\nsay 1/0\nexit\nsyntax:\nsay 'TRAPPED'\n",
        ),
        b"TRAPPED\n".to_vec()
    );
    let traps = &interp.activation().traps;
    assert!(
        !traps.contains_key(b"SYNTAX".as_slice()),
        "the SYNTAX trap fired and must be gone"
    );
    assert!(
        traps.contains_key(b"NOVALUE".as_slice()),
        "the NOVALUE trap did not fire and must be untouched -- without \
         this half the assertion above is satisfied by clearing the whole \
         table, or by never filling it"
    );
}

/// `SIGNAL ON` inside the handler re-arms the condition, under a new
/// label: the first raise reaches `first` and the second reaches
/// `second`.
#[test]
fn a_trap_can_be_re_armed_inside_its_own_handler() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"signal on syntax name first\nsay 1/0\nexit\nfirst:\nsay 'FIRST' sigl\nsignal on syntax name second\nsay 2/0\nexit\nsecond:\nsay 'SECOND' sigl\n",
        ),
        b"FIRST 2\nSECOND 7\n".to_vec()
    );
}

/// Without the re-arm the second raise is fatal -- the adjacent case
/// that pins the test above to "the trap was disabled" rather than to
/// "the second raise happened to reach a different label".
#[test]
fn a_second_raise_inside_a_handler_is_fatal_without_a_re_arm() {
    let mut interp = Interp::new();
    let failure = run_source(
        &mut interp,
        b"signal on syntax name first\nsay 1/0\nexit\nfirst:\nsay 'FIRST'\nsay 2/0\n",
    )
    .unwrap_err();
    assert!(matches!(&failure, Failure::Raised(raised) if raised.number == 42));
    assert_eq!(interp.out, b"FIRST\n".to_vec());
}

/// **Inherited item I11.** `Interp::failure_site` is first-wins, so a
/// second raise after a trapped first one reported the *first* site
/// until `offer_to_trap` began clearing it. The report must name line 6,
/// the second raise's own clause, and a version without the clearing
/// names line 2.
#[test]
fn a_second_raise_after_a_trapped_one_reports_its_own_site() {
    let mut interp = Interp::new();
    run_source(
        &mut interp,
        b"signal on syntax\nsay 1/0\nexit\nsyntax:\nsay 'HANDLER'\nsay 2/0\n",
    )
    .unwrap_err();
    let mut sites = std::mem::take(&mut interp.failure_sites);
    sites.extend(interp.failure_site.take());
    let lines: Vec<usize> = sites
        .iter()
        .map(|site| site.line().expect("a clause site"))
        .collect();
    assert_eq!(
        lines,
        vec![6],
        "exactly one echo, naming the second raise's own clause -- the \
         first raise's site (line 2) must not have survived being trapped"
    );
    assert_eq!(sites[0].text(), b"say 2/0");
}

/// `SIGNAL ON NOVALUE` fires for a simple variable and for a compound,
/// and **not** for a bare stem. The third row is the one that cannot be
/// guessed: measured, `say zstem.` under the same trap prints the
/// derived name and the program carries on, where `say zstem.1` traps.
#[test]
fn novalue_fires_for_a_simple_variable_and_a_compound_but_not_a_bare_stem() {
    for (source, expected) in [
        (
            &b"signal on novalue\nsay zprobe\nexit\nnovalue:\nsay 'TRAPPED' sigl\n"[..],
            &b"TRAPPED 2\n"[..],
        ),
        (
            &b"signal on novalue\nsay zprobe.1\nexit\nnovalue:\nsay 'TRAPPED' sigl\n"[..],
            &b"TRAPPED 2\n"[..],
        ),
        (
            &b"signal on novalue\nsay zprobe.\nsay 'RESUMED'\nexit\nnovalue:\nsay 'TRAPPED'\n"[..],
            &b"ZPROBE.\nRESUMED\n"[..],
        ),
    ] {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(&mut interp, source),
            expected.to_vec(),
            "{}",
            String::from_utf8_lossy(source)
        );
    }
}

/// An untrapped `NOVALUE` costs nothing and changes nothing -- the read
/// still yields the derived name. The neighbouring case that pins
/// `novalue_check`'s gate to "there is a trap" rather than to anything
/// about the variable.
#[test]
fn an_untrapped_novalue_still_reads_the_derived_name() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"say zprobe\nsay zprobe.1\n"),
        b"ZPROBE\nZPROBE.1\n".to_vec()
    );
}

/// A trap label that does not exist is `16.1`, reported against the
/// **raising** clause rather than against the `SIGNAL ON` clause or the
/// missing label -- so the site the raise had already recorded survives,
/// which is why `offer_to_trap` resolves the label before it clears
/// anything.
#[test]
fn a_trap_label_that_does_not_exist_is_16_1_at_the_raising_clause() {
    let mut interp = Interp::new();
    let failure = run_source(
        &mut interp,
        b"signal on syntax name nosuchlabel\nsay 'a'\nsay 1/0\n",
    )
    .unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (16, 1));
    assert_eq!(raised.additional, vec![b"NOSUCHLABEL".to_vec()]);
    let site = interp.failure_site.expect("a site was resolved");
    assert_eq!((site.line(), site.text()), (Some(3), &b"say 1/0"[..]));
}

/// A trap is inherited by a callee and fires **there**, in the callee's
/// own activation -- which is observable only because `PROCEDURE`
/// isolates the pool: the handler reads the callee's `ZOWNER`, not the
/// caller's. `SIGL` is the callee's raising line for the same reason.
#[test]
fn a_trap_is_inherited_by_a_callee_and_fires_in_the_callees_own_activation() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"signal on syntax\nzowner = 'CALLER-POOL'\ncall sub\nexit\nsub: procedure\nzowner = 'CALLEE-POOL'\nsay 1/0\nreturn\nsyntax:\nsay zowner sigl\n",
        ),
        b"CALLEE-POOL 7\n".to_vec()
    );
}

/// The other half: turning the trap off *in the callee* leaves the
/// caller's own enabled, so the condition propagates outward and is
/// trapped there instead -- with `SIGL` now the caller's `call` clause.
/// Together the two tests say the table is inherited **by copy**.
#[test]
fn a_callees_signal_off_leaves_the_callers_trap_enabled() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"signal on syntax\nzowner = 'CALLER-POOL'\ncall sub\nexit\nsub: procedure\nsignal off syntax\nsay 1/0\nreturn\nsyntax:\nsay zowner sigl\n",
        ),
        b"CALLER-POOL 3\n".to_vec()
    );
}

/// **The mid-clause resumption shape, and the one route that could still
/// have created two activations within one clause.** `zz = one(1)
/// two(2)`: `one` raises, the inherited trap transfers to a handler that
/// `RETURN`s, so `one(1)` yields the handler's value and evaluation
/// resumes *inside the enclosing clause*, which then calls `two`.
#[test]
fn a_trap_that_resumes_mid_clause_leaves_the_enclosing_clauses_state_intact() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"signal on syntax\nzz = one(1) two(2)\nsay 'zz=' zz\nexit\none:\nsay 1/0\nreturn 'ONEVAL'\ntwo:\nreturn 'SIGLIS' sigl\nsyntax:\nreturn 'FROMHANDLER'\n",
        ),
        b"zz= FROMHANDLER SIGLIS 2\n".to_vec()
    );
}

/// **A `CALL ON` handler runs at the clause boundary, not at the raise.**
/// `one` raises a trapped `USER` condition and returns its `RAISE ...
/// RETURN` value; the enclosing clause finishes -- `two(2)` and the
/// assignment included -- and only then does the handler run and
/// overwrite `zz`.
#[test]
fn a_call_trap_waits_for_the_raising_clause_to_finish() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"call on user marker name uh\nzz = one(1) two(2)\nsay 'zz=' zz\nexit\none:\nraise user marker return 'ONEVAL'\ntwo:\nreturn 'TWOVAL'\nuh:\nzz = 'HANDLER-RAN-AT' sigl\nreturn\n",
        ),
        b"zz= HANDLER-RAN-AT 2\n".to_vec(),
        "the assignment stored ONEVAL TWOVAL first, then the handler \
         overwrote it -- a handler that fired at the raise would leave \
         `zz` as the routine's own value"
    );
}

/// `RAISE`'s delivery table, the part that a two-level program cannot
/// tell apart. Each row is a three-level chain with the trap enabled in
/// exactly one place; the expected value names which handler ran.
#[test]
fn raise_delivery_depends_on_the_tail_and_on_the_condition() {
    // `RAISE SYNTAX ... RETURN` searches from the raising activation, so
    // the trap `lev2` inherited from `lev1` is the one that fires.
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"call lev1\nexit\nlev1:\nsignal on syntax name mid\ncall lev2\nreturn\nlev2:\nraise syntax 40.4 return\nmid:\nsay 'MID' sigl\nexit\n",
        ),
        b"MID 8\n".to_vec()
    );

    // The same raise with no tail skips every level but the outermost,
    // so `mid` never runs and the condition is fatal.
    let mut interp = Interp::new();
    let failure = run_source(
        &mut interp,
        b"call lev1\nexit\nlev1:\nsignal on syntax name mid\ncall lev2\nreturn\nlev2:\nraise syntax 40.4\nmid:\nsay 'MID' sigl\nexit\n",
    )
    .unwrap_err();
    assert!(matches!(&failure, Failure::Raised(raised) if raised.number == 40));
    assert!(interp.out.is_empty(), "`mid` must not have run");

    // ... and enabling it at the top as well is what catches it, with
    // `SIGL` the main body's own clause.
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"signal on syntax name outer\ncall lev1\nexit\nlev1:\nsignal on syntax name mid\ncall lev2\nreturn\nlev2:\nraise syntax 40.4\nmid:\nsay 'MID' sigl\nexit\nouter:\nsay 'OUTER' sigl\nexit\n",
        ),
        b"OUTER 2\n".to_vec()
    );

    // A non-`SYNTAX` condition with a `RETURN` tail searches from the
    // *caller*, so the raising routine's own inherited trap is skipped:
    // `SIGL` is the caller's clause, not the `raise` line.
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"signal on user marker\ncall sub\nexit\nsub:\nraise user marker return\nmarker:\nsay 'MARKER' sigl\nexit\n",
        ),
        b"MARKER 2\n".to_vec()
    );
}

/// An untrapped condition's default action: `HALT` reports, everything
/// else is silent. The pair is what makes `Raised::reportable`'s split
/// mean something rather than being a spelling.
#[test]
fn an_untrapped_raise_reports_for_halt_and_is_silent_for_the_rest() {
    let mut interp = Interp::new();
    let failure = run_source(&mut interp, b"say 'a'\nraise halt\nsay 'b'\n").unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (4, 1));

    for source in [
        &b"say 'a'\nraise user marker\nsay 'b'\n"[..],
        &b"say 'a'\nraise error 5\nsay 'b'\n"[..],
    ] {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(&mut interp, source),
            b"a\n".to_vec(),
            "{}: silent, and `b` is not reached because the tail-less \
             RAISE ends the program",
            String::from_utf8_lossy(source)
        );
    }
}

/// `RC` is the major for a trapped `SYNTAX` however it arose, the raise's
/// own argument for `ERROR`, and untouched for `NOVALUE` -- three rows
/// that no two of which share a rule.
#[test]
fn rc_is_set_from_the_condition_when_a_trap_fires() {
    for (source, expected) in [
        (
            &b"signal on syntax\nsay 1/0\nexit\nsyntax:\nsay rc\n"[..],
            &b"42\n"[..],
        ),
        (
            &b"signal on syntax\nraise syntax 40.4\nexit\nsyntax:\nsay rc\n"[..],
            &b"40\n"[..],
        ),
        (
            &b"signal on error\ncall sub\nexit\nsub:\nraise error 5 return\nerror:\nsay rc\n"[..],
            &b"5\n"[..],
        ),
        (
            &b"signal on novalue\nsay zprobe\nexit\nnovalue:\nsay rc\n"[..],
            &b"RC\n"[..],
        ),
    ] {
        let mut interp = Interp::new();
        assert_eq!(
            say_output(&mut interp, source),
            expected.to_vec(),
            "{}",
            String::from_utf8_lossy(source)
        );
    }
}

/// **Inherited item I16, re-verified against a real trap rather than
/// argued.** I16 concluded that `SIGNAL ON SYNTAX` cannot accumulate a
/// temps leak, resting entirely on `Op::Clause`'s region being the single
/// chokepoint that heals the `?`-skipped `pop_frame` sites in
/// `eval.rs`. The conclusion is measured here rather than inherited: two
/// hundred trap-and-resume cycles and
/// four hundred must leave the same number of live temps, and a leak of
/// even one root per cycle would make the second number two hundred
/// larger.
#[test]
fn a_trap_that_resumes_does_not_accumulate_temps_frames() {
    fn live_temps_after(cycles: usize) -> usize {
        let source = format!(
            "signal on novalue name h\nzcount = 0\ntop:\nzcount = zcount + 1\nsay ((zprobe))\nh:\nsignal on novalue name h\nif zcount < {cycles} then signal top\nsay 'done' zcount\n"
        );
        let mut interp = Interp::new();
        let out = say_output(&mut interp, source.as_bytes());
        assert_eq!(
            out,
            format!("done {cycles}\n").into_bytes(),
            "the loop must actually have trapped {cycles} times"
        );
        interp.roots.temps_len()
    }
    assert_eq!(
        live_temps_after(200),
        live_temps_after(400),
        "a temps root retained per trapped-and-resumed cycle would make \
         the four-hundred-cycle run exactly two hundred larger"
    );
}

/// `RAISE PROPAGATE` re-raises the condition whose handler is running,
/// past every enclosing trap, and the report names the **original**
/// raising clause rather than the `raise propagate` clause -- which is
/// why the echo stack travels on `ActiveCondition` rather than being
/// dropped when the trap cleared it.
#[test]
fn raise_propagate_re_raises_the_original_condition_and_its_site() {
    let mut interp = Interp::new();
    let failure = run_source(
        &mut interp,
        b"signal on syntax name outer\ncall sub\nexit\nsub:\nsignal on syntax name inner\nsay 1/0\nreturn\ninner:\nraise propagate\nouter:\nsay 'OUTER-MUST-NOT-RUN'\n",
    )
    .unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (42, 3));
    assert!(
        raised.delivery.positionless,
        "the major line drops its ` running <path> line <n>` span"
    );
    assert!(interp.out.is_empty(), "`outer` must not have run");
    let mut sites = std::mem::take(&mut interp.failure_sites);
    sites.extend(interp.failure_site.take());
    let lines: Vec<usize> = sites
        .iter()
        .map(|site| site.line().expect("a clause site"))
        .collect();
    assert_eq!(
        lines,
        vec![6, 2],
        "the original `say 1/0` clause and the `call sub` above it, not \
         the `raise propagate` clause on line 9"
    );
}

/// With no handler running, `RAISE PROPAGATE` is `98.918` -- the
/// adjacent case that pins the test above to "there was an active
/// condition" rather than to anything about `PROPAGATE` itself.
#[test]
fn raise_propagate_with_no_active_condition_is_98_918() {
    let mut interp = Interp::new();
    let failure = run_source(&mut interp, b"say 'a'\nraise propagate\n").unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (98, 918));
    assert!(!raised.delivery.positionless);
}

/// A `CALL ON` trap does not catch a condition that has no resumption
/// point, even through `ANY` -- the one spelling that makes the question
/// askable at all, since `CALL ON SYNTAX` is a parse error. And `SIGNAL
/// ON ANY` does catch it, which is the pair that keeps this a statement
/// about `CALL` rather than about `ANY` not working.
#[test]
fn a_call_trap_declines_a_condition_with_nowhere_to_resume_but_a_signal_trap_takes_it() {
    let mut interp = Interp::new();
    let failure = run_source(
        &mut interp,
        b"call on any name uh\nsay 1/0\nexit\nuh:\nsay 'UH-MUST-NOT-RUN'\nreturn\n",
    )
    .unwrap_err();
    assert!(matches!(&failure, Failure::Raised(raised) if raised.number == 42));
    assert!(interp.out.is_empty());

    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"signal on any\nsay 1/0\nexit\nany:\nsay 'ANY-TRAPPED' sigl\n",
        ),
        b"ANY-TRAPPED 2\n".to_vec()
    );
}

// ---- fix round 1: the pending-trap delivery boundary, and its identity ----

/// **Fix round 1's Critical, half (a).** The clause that finishes may
/// itself be the `RETURN`: `aa`'s whole body is `return bb()`, and `bb`
/// raises a trapped `USER` condition. The oracle runs the handler at that
/// clause's own boundary -- so `SIGL` is line 7, `aa`'s `return bb()` --
/// and then returns from `aa`.
#[test]
fn a_pending_trap_is_delivered_when_the_trapping_clause_is_a_return() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"call on user foo name uh\nzmark = 'NOMARK'\ncall aa\nsay 'end mark=' zmark\nexit\naa:\nreturn bb()\nbb:\nraise user foo return 'BBVAL'\nuh:\nzmark = 'HANDLER-AT' sigl\nreturn\n",
        ),
        b"end mark= HANDLER-AT 7\n".to_vec()
    );
}

/// **Fix round 1's Critical, half (b).** The same program with a second,
/// unrelated `call cc` after it. The handler still belongs to `aa`'s
/// `return bb()` clause (`SIGL` 8 here), and printed `HANDLER-AT 11` --
/// the `cc:` label's own line -- before the fix, because it ran inside
/// `cc`.
#[test]
fn a_pending_trap_is_not_delivered_into_a_later_activation_at_the_same_depth() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"call on user foo name uh\nzmark = 'NOMARK'\ncall aa\ncall cc\nsay 'end mark=' zmark\nexit\naa:\nreturn bb()\nbb:\nraise user foo return 'BBVAL'\ncc:\nsay 'in cc'\nreturn\nuh:\nzmark = 'HANDLER-AT' sigl\nreturn\n",
        ),
        b"in cc\nend mark= HANDLER-AT 8\n".to_vec()
    );
}

/// **A third shape of the same defect, which the review did not reach and
/// a depth cannot close.** The pending condition's activation is unwound
/// by an error its *caller* traps, so it dies without ever finishing
/// another clause; the caller then calls something else, which lands at
/// the very depth the dead activation had.
#[test]
fn a_pending_trap_whose_activation_is_gone_is_never_delivered() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"signal on syntax name sh\ncall on user foo name uh\nzmark = 'NOMARK'\ncall aa\nsay 'end mark=' zmark\nexit\naa:\nsignal off syntax\nzq = bb() + 1/0\nreturn\nbb:\nraise user foo return 'BBVAL'\ncc:\nsay 'in cc'\nreturn\nuh:\nzmark = 'HANDLER-AT' sigl\nreturn\nsh:\nsay 'SYNTAX at' sigl\ncall cc\nsay 'after mark=' zmark\nexit\n",
        ),
        b"SYNTAX at 4\nin cc\nafter mark= NOMARK\n".to_vec()
    );
}

/// **Fix round 1's finding 2.** Once a `CALL ON` handler has returned,
/// its condition is no longer active, so a later `RAISE PROPAGATE` has
/// nothing to re-raise and is `98.918`.
#[test]
fn a_returned_call_handler_leaves_no_active_condition_to_propagate() {
    let mut interp = Interp::new();
    let failure = run_source(
        &mut interp,
        b"call on user foo name uh\ncall sub\nsay 'resumed'\nraise propagate\nsay 'not reached'\nexit\nsub:\nraise user foo return 'SVAL'\nuh:\nsay 'UH ran'\nreturn\n",
    )
    .unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (98, 918));
    assert_eq!(interp.out, b"UH ran\nresumed\n".to_vec());
}

/// The adjacent case that keeps the clearing where it belongs: a `SIGNAL
/// ON` handler that runs *on* -- `SIGNAL`s to another label and only then
/// propagates -- must still find its condition. Measured, and it is why
/// `offer_to_trap` has no equivalent of the line above.
#[test]
fn a_signal_handler_that_runs_on_can_still_propagate() {
    let mut interp = Interp::new();
    let failure = run_source(
        &mut interp,
        b"signal on syntax name th\nsay 1/0\nsay 'x'\nth:\nsay 'TH ran'\nsignal onward\nonward:\nsay 'onward'\nraise propagate\n",
    )
    .unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (42, 3));
    assert!(raised.delivery.positionless);
    assert_eq!(interp.out, b"TH ran\nonward\n".to_vec());
}

/// **4b's fix round 1, finding 3(a).** A `CALL ON` trap is held for its
/// handler's duration and released afterwards, unlike a `SIGNAL ON`
/// trap, which is removed and stays removed. `deliver_pending_traps`
/// documented this and nothing tested it: deleting the release left the
/// whole suite and the corpus gate green.
#[test]
fn a_call_trap_is_put_back_after_its_handler_returns() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"call on user foo name uh\ncall raiser\nsay 'mid'\ncall raiser\nsay 'end'\nexit\nraiser:\nraise user foo return 'RV'\nuh:\nsay 'UH' sigl\nreturn\n",
        ),
        b"UH 2\nmid\nUH 4\nend\n".to_vec()
    );
}

/// **Fix round 1's finding 3(b).** `RAISE PROPAGATE` of a condition with
/// no catalogue entry ends the program silently rather than reporting.
/// The guard that makes that true had no test, and deleting it does not
/// merely go unnoticed -- it emits `Error 0:  <no message 0.0 in the
/// catalogue>`, which is the exact failure mode `novalue_check`'s own doc
/// comment says the design keeps unreachable.
#[test]
fn raise_propagate_of_an_unreportable_condition_ends_the_program_silently() {
    let mut interp = Interp::new();
    let ended = run_source(
        &mut interp,
        b"call on user foo name uh\ncall sub\nsay 'after'\nexit\nsub:\nraise user foo return 'SV'\nuh:\nsay 'UH ran'\nraise propagate\n",
    );
    assert!(
        ended.is_ok(),
        "a USER condition has no report to give, so this ends the \
         program rather than raising: {ended:?}"
    );
    assert_eq!(
        interp.out,
        b"UH ran\n".to_vec(),
        "`say 'after'` must not run -- the propagate ends the program"
    );
    assert!(
        !interp.trace.windows(7).any(|w| w == b"Error 0"),
        "no `Error 0` placeholder may reach stderr: {:?}",
        String::from_utf8_lossy(&interp.trace)
    );
}

/// **Fix round 1's finding 5.** `RAISE SYNTAX`'s argument is validated
/// rather than used verbatim. Every row measured against the oracle; see
/// `raise_syntax_condition` for the rule and the boundary probes.
#[test]
fn raise_syntax_validates_its_argument() {
    for (argument, expected, substitution) in [
        (&b"40.4"[..], (40u16, 4u16), None),
        (&b"40"[..], (40, 0), None),
        (&b"40.001"[..], (40, 1), None),
        (&b"99"[..], (99, 0), None),
        (&b"40.10"[..], (98, 941), Some("40010")),
        (&b"3.5"[..], (98, 941), Some("3005")),
        // A major with no `(major, 0)` catalogue entry renders the
        // original `major.sub` instead of the composed number. There are
        // 45 such majors in 1..=99, not the two an earlier version of
        // this comment claimed -- `50.1` below is one of the others, and
        // is here so the row set does not accidentally describe a
        // two-element special case.
        (&b"1"[..], (98, 941), Some("1.0")),
        (&b"2.1"[..], (98, 941), Some("2.1")),
        (&b"50.1"[..], (98, 941), Some("50.1")),
        (&b"87"[..], (98, 941), Some("87.0")),
        // Fix round 2's NEW 2: the sub is bounded at 1000, and the
        // boundary is what pins it -- `40.999` is a well-formed unknown
        // code and `40.1000` is not a code at all.
        (&b"40.999"[..], (98, 941), Some("40999")),
        (&b"40.1000"[..], (33, 904), None),
        (&b"40.99999"[..], (33, 904), None),
        // Fix round 2's NEW 4: each half is a Rexx number, and a bare
        // decimal point is not one.
        (&b"'4E1'"[..], (40, 0), None),
        (&b"'40.1E2'"[..], (98, 941), Some("40100")),
        (&b"'40.'"[..], (33, 904), None),
        (&b"'+40'"[..], (40, 0), None),
        (&b"0"[..], (33, 904), None),
        (&b"100"[..], (33, 904), None),
        (&b"999"[..], (33, 904), None),
        (&b"'abc'"[..], (33, 904), None),
    ] {
        let mut source = b"raise syntax ".to_vec();
        source.extend_from_slice(argument);
        source.push(b'\n');
        let mut interp = Interp::new();
        let failure = run_source(&mut interp, &source).unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("{}: expected Raised", String::from_utf8_lossy(argument));
        };
        assert_eq!(
            (raised.number, raised.sub),
            expected,
            "{}",
            String::from_utf8_lossy(argument)
        );
        if let Some(substitution) = substitution {
            assert_eq!(
                raised.additional,
                vec![substitution.as_bytes().to_vec()],
                "{}",
                String::from_utf8_lossy(argument)
            );
        }
        // The exit code is a consequence of the number, but the global
        // constraint is about the *band*: a raise must never exit 0, and
        // three of these did.
        assert_ne!(
            raised.exit_code(),
            0,
            "{}: a raise must not exit 0",
            String::from_utf8_lossy(argument)
        );
    }
}

// ---- fix round 2: the clause boundary inside nested bodies ----

/// **Fix round 2's NEW 5.** A clause inside a `DO` body reaches its own
/// boundary, not the `END`'s. Both iterations must see the handler's
/// value, and `SIGL` must be the `call sub` clause's line (4), not the
/// `say`'s.
#[test]
fn a_pending_trap_is_delivered_inside_a_do_body() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"call on user foo name uh\nzmark = 'NOMARK'\ndo i = 1 to 2\ncall sub\nsay 'after call' i 'mark=' zmark\nend\nsay 'end mark=' zmark\nexit\nsub:\nraise user foo return 'SV'\nuh:\nzmark = 'HANDLER-AT' sigl\nreturn\n",
        ),
        b"after call 1 mark= HANDLER-AT 4\nafter call 2 mark= HANDLER-AT 4\nend mark= HANDLER-AT 4\n".to_vec()
    );
}

/// The same property for the other two constructs `run_bounded` serves:
/// a `WHEN`'s body and an `INTERPRET` fragment. Three callers, one rule
/// -- and all three were wrong together, which is what made this a
/// mechanism rather than a `DO` quirk.
#[test]
fn a_pending_trap_is_delivered_inside_a_when_body_and_a_fragment() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"call on user foo name uh\nzmark = 'NOMARK'\nselect\nwhen 1 = 1 then do\ncall sub\nsay 'in when mark=' zmark\nend\notherwise nop\nend\nsay 'end mark=' zmark\nexit\nsub:\nraise user foo return 'SV'\nuh:\nzmark = 'HANDLER-AT' sigl\nreturn\n",
        ),
        b"in when mark= HANDLER-AT 5\nend mark= HANDLER-AT 5\n".to_vec()
    );

    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"call on user foo name uh\nzmark = 'NOMARK'\ninterpret \"call sub; say 'inside mark=' zmark\"\nsay 'end mark=' zmark\nexit\nsub:\nraise user foo return 'SV'\nuh:\nzmark = 'HANDLER-AT' sigl\nreturn\n",
        ),
        b"inside mark= HANDLER-AT 3\nend mark= HANDLER-AT 3\n".to_vec()
    );
}

/// **Fix round 2's NEW 1.** A `CALL ON` handler can be delivered while a
/// `SIGNAL ON` handler is already running -- one clause can queue the
/// first and raise the second -- and when it returns, the condition it
/// interrupted must come back.
#[test]
fn a_call_handler_restores_the_condition_it_interrupted() {
    let mut interp = Interp::new();
    let failure = run_source(
        &mut interp,
        b"signal on syntax name sh\ncall on user foo name uh\nzq = sub() + 1/0\nsay 'not reached'\nexit\nsub:\nraise user foo return 'SV'\nuh:\nsay 'UH ran' sigl\nreturn\nsh:\nsay 'SH ran' sigl\nraise propagate\n",
    )
    .unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!(
        (raised.number, raised.sub),
        (42, 3),
        "the SYNTAX condition the SIGNAL handler is running for, not the \
         USER one the CALL handler was delivered with, and not 98.918"
    );
    assert_eq!(interp.out, b"UH ran 3\nSH ran 3\n".to_vec());
}

// ---- fix round 3: a loop header is a clause too ----

/// **Fix round 3's NEW-A.** A `DO` header is a Rexx clause: its control
/// expressions run in it, and its boundary is its own, not the whole
/// loop's. `do i = 1 to sub()` must report `SIGL` 3 -- the `DO` clause --
/// and deliver the handler before the body's first pass.
#[test]
fn a_loop_header_is_a_clause_with_its_own_boundary() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"call on user foo name uh\nzmark = 'NOMARK'\ndo i = 1 to sub()\nsay 'body' i 'mark=' zmark\nend\nsay 'end mark=' zmark\nexit\nsub:\nraise user foo return 1\nuh:\nzmark = 'HANDLER-AT' sigl\nreturn\n",
        ),
        b"body 1 mark= HANDLER-AT 3\nend mark= HANDLER-AT 3\n".to_vec()
    );
}

/// `WHILE` and `UNTIL` are re-tested once per pass, and **which clause
/// that test belongs to changes**: it is the clause that transferred
/// control back to the loop header. See `HeaderClause` for the oracle
/// mechanism; this test carries the `DO`-then-`END` half, and
/// `a_loop_retest_after_an_iterate_belongs_to_the_iterate_clause` carries
/// the third member.
#[test]
fn a_while_retest_belongs_to_the_do_clause_then_to_the_end_clause() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"call on user foo name uh\nzmark = 'NOMARK'\nzn = 0\ndo while zn < sub()\nzn = zn + 1\nsay 'body' zn 'mark=' zmark\nend\nsay 'end mark=' zmark\nexit\nsub:\nraise user foo return 2\nuh:\nzmark = 'HANDLER-AT' sigl\nreturn\n",
        ),
        b"body 1 mark= HANDLER-AT 4\nbody 2 mark= HANDLER-AT 7\nend mark= HANDLER-AT 7\n".to_vec()
    );

    // `UNTIL` is tested only after a pass, so every one of its tests is
    // the `END` clause's -- the same rule with the first-pass case
    // absent, which is why it needs its own row rather than being
    // assumed to follow.
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"call on user foo name uh\nzmark = 'NOMARK'\nzn = 0\ndo until zn >= sub()\nzn = zn + 1\nsay 'body' zn 'mark=' zmark\nend\nsay 'end mark=' zmark\nexit\nsub:\nraise user foo return 2\nuh:\nzmark = 'HANDLER-AT' sigl\nreturn\n",
        ),
        b"body 1 mark= NOMARK\nbody 2 mark= HANDLER-AT 7\nend mark= HANDLER-AT 7\n".to_vec()
    );
}

// ---- fix round 4: NEW-1, the third member of the re-test family ----

/// **Fix round 4's NEW-1, a regression round 3 introduced.** When an
/// `ITERATE` ends a pass, the re-test that follows belongs to the
/// `ITERATE`'s own clause -- the oracle re-enters the loop from inside
/// `RexxActivation::iterate`, so `END` is never reached and never owns
/// anything. Round 3 attributed it to `END` and turned two
/// byte-for-byte-matching programs into divergences.
#[test]
fn a_loop_retest_after_an_iterate_belongs_to_the_iterate_clause() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"zn = 0\ndo i = 1 to 3 while zs() < 3\nzn = zn + 1\niterate\nend\nexit\nzs:\nsay sigl\nreturn zn\n",
        ),
        b"2\n4\n4\n".to_vec(),
        "the DO clause on the first test, then the ITERATE's own line twice"
    );

    // The adjacent success: drop the `ITERATE` and the body falls through
    // to `END`, which is line 4 here, so the numbers coincide only
    // because `END` moved up a line -- the rule being tested is which
    // clause, not which number.
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"zn = 0\ndo i = 1 to 3 while zs() < 3\nzn = zn + 1\nend\nexit\nzs:\nsay sigl\nreturn zn\n",
        ),
        b"2\n4\n4\n".to_vec(),
        "a fall-through pass still hands the re-test to the END clause"
    );

    // `UNTIL`, where one program shows both: the first test follows an
    // `ITERATE` on line 4, the second follows a fall-through to `END` on
    // line 6.
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"zn = 0\ndo until zs() >= 2\nzn = zn + 1\nif zn = 1 then iterate\nnop\nend\nexit\nzs:\nsay sigl\nreturn zn\n",
        ),
        b"4\n6\n".to_vec(),
        "the ITERATE clause, then the END clause, in one program"
    );
}

/// `END` is **not executed** when an `ITERATE` ends a pass, so it does not
/// echo either -- the other half of NEW-1, found while measuring it.
#[test]
fn end_does_not_echo_for_a_pass_an_iterate_ended() {
    let mut interp = Interp::new();
    run_source(
        &mut interp,
        b"trace r\nzn = 0\ndo while zn < 2\nzn = zn + 1\niterate\nend\n",
    )
    .unwrap();
    assert!(
        !String::from_utf8_lossy(&interp.trace).contains("*-* end"),
        "no END echo for an ITERATE-ended pass, got:\n{}",
        String::from_utf8_lossy(&interp.trace)
    );

    let mut interp = Interp::new();
    run_source(
        &mut interp,
        b"trace r\nzn = 0\ndo while zn < 2\nzn = zn + 1\nend\n",
    )
    .unwrap();
    assert_eq!(
        String::from_utf8_lossy(&interp.trace)
            .matches("*-* end")
            .count(),
        2,
        "the adjacent success: a fall-through pass does echo END, once per pass, got:\n{}",
        String::from_utf8_lossy(&interp.trace)
    );
}

// ---- fix round 4: NEW-2, the header clause of a nesting construct ----

/// **Fix round 4's NEW-2.** An `IF`'s condition, a `SELECT CASE`'s
/// expression and each listed `WHEN`'s condition are clauses in their own
/// right, so a `CALL ON` handler queued by one runs at *that* clause's
/// boundary -- before the branch, before the next `WHEN`, before
/// `OTHERWISE`.
#[test]
fn a_construct_header_is_a_clause_with_its_own_boundary() {
    let trap = b"call on user foo name uh\nzmark = 'NOMARK'\n";
    let handler =
        b"exit\nsub:\nraise user foo return 'SV'\nuh:\nzmark = 'HANDLER-AT' sigl\nreturn\n";
    let rows: [(&[u8], &[u8]); 4] = [
        // The `IF` clause is line 3; `then` is line 4.
        (
            b"if sub() = 'SV'\n  then say 'yes mark=' zmark\n",
            b"yes mark= HANDLER-AT 3\n",
        ),
        // A false `WHEN` on line 4, a winning one on line 5.
        (
            b"select\nwhen sub() = 'NO' then say 'first mark=' zmark\nwhen 1 = 1 then say 'second mark=' zmark\nend\n",
            b"second mark= HANDLER-AT 4\n",
        ),
        // `SELECT CASE`'s own expression, line 3.
        (
            b"select case sub()\nwhen 'SV' then say 'hit mark=' zmark\notherwise say 'oth mark=' zmark\nend\n",
            b"hit mark= HANDLER-AT 3\n",
        ),
        // A false `WHEN` on line 4 falling through to `OTHERWISE`, whose
        // body must already see the handler's value: this row is wrong in
        // timing as well as line without the fix (`NOMARK`, then a later
        // delivery at line 6).
        (
            b"select\nwhen sub() = 'NO' then say 'hit mark=' zmark\notherwise\nsay 'oth mark=' zmark\nend\n",
            b"oth mark= HANDLER-AT 4\n",
        ),
    ];
    for (body, expected) in rows {
        let mut program = trap.to_vec();
        program.extend_from_slice(body);
        program.extend_from_slice(handler);
        let mut interp = Interp::new();
        assert_eq!(
            say_output(&mut interp, &program),
            expected.to_vec(),
            "header clause boundary for:\n{}",
            String::from_utf8_lossy(body)
        );
    }
}

/// The adjacent success for `a_construct_header_is_a_clause_with_its_own_
/// boundary`: with `then` on the *same* line as the condition, the right
/// answer and the wrong one coincide, and both must be the oracle's.
#[test]
fn a_single_line_then_reports_the_same_line_either_way() {
    let trap = b"call on user foo name uh\nzmark = 'NOMARK'\n";
    let handler =
        b"exit\nsub:\nraise user foo return 'SV'\nuh:\nzmark = 'HANDLER-AT' sigl\nreturn\n";
    let rows: [(&[u8], &[u8]); 3] = [
        (
            b"if sub() = 'SV' then say 'yes mark=' zmark\n",
            b"yes mark= HANDLER-AT 3\n",
        ),
        (
            b"select\nwhen sub() = 'SV' then say 'hit mark=' zmark\nend\n",
            b"hit mark= HANDLER-AT 4\n",
        ),
        // The false `IF`, which was already right before the fix and must
        // stay right: nothing is nested inside the branch it takes.
        (
            b"if sub() = 'NO'\n  then say 'yes mark=' zmark\n  else say 'no mark=' zmark\n",
            b"no mark= HANDLER-AT 3\n",
        ),
    ];
    for (body, expected) in rows {
        let mut program = trap.to_vec();
        program.extend_from_slice(body);
        program.extend_from_slice(handler);
        let mut interp = Interp::new();
        assert_eq!(
            say_output(&mut interp, &program),
            expected.to_vec(),
            "coinciding answers for:\n{}",
            String::from_utf8_lossy(body)
        );
    }
}

/// **An `INTERPRET` fragment is the one construct that must *not* end its
/// header clause before running what it nests**, and this is the test
/// that stops the NEW-2 fix being applied to it by analogy.
#[test]
fn an_interpret_clause_does_not_deliver_before_its_fragment_runs() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"call on user foo name uh\nzmark = 'NOMARK'\ninterpret sub()\nsay 'end mark=' zmark\nexit\nsub:\nraise user foo return \"say 'frag mark=' zmark\"\nuh:\nzmark = 'HANDLER-AT' sigl\nreturn\n",
        ),
        b"frag mark= NOMARK\nend mark= HANDLER-AT 3\n".to_vec()
    );
}

/// **Fix round 3's NEW-B.** When the handler run at a clause's boundary
/// itself fails, the clause the report blames is the one whose boundary
/// ran it -- `call sub` -- not the enclosing `DO`.
#[test]
fn a_handler_that_fails_at_a_clause_boundary_blames_that_clause() {
    let mut interp = Interp::new();
    run_source(
        &mut interp,
        b"call on user foo name uh\ndo i = 1 to 1\ncall sub\nsay 'after'\nend\nexit\nsub:\nraise user foo return 'SV'\nuh:\nsay 1/0\nreturn\n",
    )
    .unwrap_err();
    let mut sites = std::mem::take(&mut interp.failure_sites);
    sites.extend(interp.failure_site.take());
    assert!(
        sites.iter().any(|site| site.text() == b"call sub"),
        "expected the `call sub` clause among the echoes, got {:?}",
        sites
            .iter()
            .map(|site| String::from_utf8_lossy(site.text()).into_owned())
            .collect::<Vec<_>>()
    );
}
