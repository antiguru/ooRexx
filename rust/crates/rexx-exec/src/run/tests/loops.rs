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

/// A controlled loop's `BY` is negative exactly when comparing it against
/// zero says it is, at every `DIGITS` and `FUZZ`.
#[test]
fn a_negative_by_is_what_comparing_it_against_zero_says() {
    let mut checked = 0usize;
    let mut negatives = 0usize;
    for spelling in [
        "1",
        "-1",
        "0",
        "-0",
        "0.0",
        "1.1",
        "-1.1",
        "0.0001",
        "-0.0001",
        "1e9",
        "-1e9",
        "1e-9",
        "-1e-9",
        "999999999",
        "-999999999",
    ] {
        let by = Number::parse(spelling).expect("a literal");
        for digits in [1u64, 2, 9, 20] {
            for fuzz in [0u64, 1, 9, 20] {
                let against_zero = numeric_less(&by, &Number::zero(), digits, fuzz)
                    .expect("comparing against zero cannot overflow");
                assert_eq!(
                    by.signum() < 0,
                    against_zero,
                    "{spelling} at digits {digits} fuzz {fuzz}"
                );
                checked += 1;
                if against_zero {
                    negatives += 1;
                }
            }
        }
    }
    assert!(checked > 100, "only {checked} case(s)");
    assert!(
        negatives > 0,
        "no case was negative, so only the false answer was compared"
    );
}
// ---- DO/LOOP: every LoopKind ----

#[test]
fn a_simple_do_block_runs_its_body_exactly_once_with_no_control() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"do\nsay 'once'\nend"),
        b"once\n".to_vec()
    );
}

/// `LOOP` alone is `DO FOREVER`; `DO` alone is a block, not a loop
/// (`ast.rs`'s own doc comment on `LoopKind::Simple`/`Forever`) --
/// proven here by the fact that only the `LOOP` form is stopped by a
/// bare `LEAVE`.
#[test]
fn loop_alone_is_forever_and_do_alone_is_a_block_not_a_loop() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"n = 0\nloop\nn = n + 1\nif n = 3 then leave\nend\nsay n"
        ),
        b"3\n".to_vec()
    );
}

#[test]
fn do_forever_runs_until_a_bare_leave() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"n = 0\ndo forever\nn = n + 1\nif n = 3 then leave\nend\nsay n"
        ),
        b"3\n".to_vec()
    );
}

#[test]
fn do_count_repeats_exactly_the_evaluated_number_of_times() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"n = 0\ndo 3\nn = n + 1\nend\nsay n"),
        b"3\n".to_vec()
    );
    // The repeat count is an expression, evaluated once.
    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"n = 0\ndo 1 + 2\nn = n + 1\nend\nsay n"),
        b"3\n".to_vec()
    );
    // Zero repetitions is legal and runs the body no times at all.
    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"n = 0\ndo 0\nn = n + 1\nend\nsay n"),
        b"0\n".to_vec()
    );
}

/// F1, found by review: a bare count `DO n` traced no `>K>` line at
/// all, where the oracle traces it tagged `FOR` -- the same tag an
/// explicit `DO ... FOR n` gets, measured (this task's own report).
/// A bare count reaches that line through `HeaderRole::Count`'s `FOR`
/// answer in `HeaderRole::keyword`, which is what `echo_header_value`
/// hands `trace_keyword`. It is asserted here because the report's own
/// verification claimed `>K>` was checked while never actually running a
/// bare-count program through it.
#[test]
fn a_bare_repeat_count_traces_as_for_the_same_as_an_explicit_one() {
    let mut interp = Interp::new();
    say_output_traced(&mut interp, b"do 2\nnop\nend");
    // `>K>` fires exactly once, on the first pass, matching every
    // other single-evaluation control-setup keyword (`TO`/`BY`/
    // `OVER`) -- the third `do 2` re-echo is the exit-check pass
    // `run_repeating`'s own per-pass re-echo already covers, verified
    // by running this exact assertion once and reading the bytes
    // back rather than hand-composing them (`this file's own report
    // has the trap: a hand-guessed expectation here was short by
    // exactly this one pass on the first attempt).
    assert_eq!(
        interp.trace,
        b"     1 *-* do 2\n       >K>   \"FOR\" => \"2\"\n     2 *-*   nop\n     \
          3 *-* end\n     1 *-* do 2\n     2 *-*   nop\n     3 *-* end\n     1 *-* do 2\n"
            .to_vec()
    );
}

/// F4, found by review: a comma-list condition traced no `>>>` for its
/// own elements at all, only the list's overall result, under
/// `TRACE R` (not merely `TRACE I`) -- measured, the oracle shows
/// *three* `>>>` lines for `if 1, 1 then`, one per element plus one
/// for the list, and `eval_logical_list`'s own fix (`eval.rs`) is what
/// this test defends. The mutation it kills: removing that function's
/// `self.trace_result(indent, &text)` call drops the middle two lines,
/// leaving only the third (`eval_condition`'s own, unaffected) --
/// caught by this test and by neither of the pre-existing comma-list
/// tests (`if_condition_that_is_a_comma_list_raises_34_6_not_34_1`,
/// the short-circuit test), since neither one traces anything.
#[test]
fn a_comma_list_conditions_own_elements_each_trace_their_result_under_trace_r() {
    let mut interp = Interp::new();
    say_output_traced(&mut interp, b"if 1, 1 then nop");
    assert_eq!(
        interp.trace,
        b"     1 *-* if 1, 1 \n       >>>   \"1\"\n       >>>   \"1\"\n       >>>   \"1\"\n     \
          1 *-*   then\n     1 *-*     nop\n"
            .to_vec()
    );
}

/// Found while re-verifying F4 rather than assumed clean: fixing the
/// comma-list `>>>` gap exposed that `DO UNTIL`'s own re-echo was
/// wired to the wrong site. The mutation this kills is two-sided --
/// removing `is_until_loop`'s own gate on the top-of-loop echo makes
/// a multi-pass `UNTIL` loop echo its clause **twice** per pass (once
/// there, once at `UNTIL`'s own site); removing `UNTIL`'s own
/// unconditional echo instead makes it echo **zero** times on the
/// first pass (the top-of-loop site is `first_pass`-gated and never
/// fires before the loop has gone around once). Only the fix in
/// between gets exactly one echo per completed pass, matching the
/// oracle (`t13_until_multi.rex`, this task's report).
#[test]
fn do_until_re_echoes_its_clause_exactly_once_per_pass_not_twice_or_zero() {
    let mut interp = Interp::new();
    say_output_traced(&mut interp, b"n = 0\ndo until n = 2\nn = n + 1\nend\nsay n");
    assert_eq!(
        interp.trace,
        b"     1 *-* n = 0\n       >>>   \"0\"\n     2 *-* do until n = 2\n     \
          3 *-*   n = n + 1\n       >>>     \"1\"\n     4 *-* end\n     2 *-* do until n = 2\n       \
          >K>     \"UNTIL\" => \"0\"\n     3 *-*   n = n + 1\n       >>>     \"2\"\n     4 *-* end\n     \
          2 *-* do until n = 2\n       >K>     \"UNTIL\" => \"1\"\n     5 *-* say n\n       >>>   \"2\"\n"
            .to_vec()
    );
}

#[test]
fn do_with_takes_the_loud_path() {
    let mut interp = Interp::new();
    let failure = run_source(&mut interp, b"do with index i over 'x'\nsay i\nend").unwrap_err();
    let Failure::Loud(loud) = failure else {
        panic!("expected Loud, got {failure:?}");
    };
    // Review finding I1: `instruction.kind` here is `InstructionKind::Do`,
    // which this crate implements -- `lib.rs`'s `owned_message` must not
    // attribute this to a phase (there is none to blame; `DO WITH` is
    // Phase 5's *reason*, but the message names the construct, not the
    // reason). Mutation-kill for deleting the `None` carve-out in
    // `owned_message`: this assertion is what turns that deletion into a
    // failure here.
    assert_eq!(
        loud.message, "DO is not implemented",
        "a construct 4a does implement must not be attributed to a phase; \
         see lib.rs's owned_message"
    );
}

#[test]
fn do_counter_takes_the_loud_path_regardless_of_which_other_kind_it_rides_on() {
    let mut interp = Interp::new();
    let failure = run_source(&mut interp, b"do counter c i = 1 to 3\nnop\nend").unwrap_err();
    let Failure::Loud(loud) = failure else {
        panic!("expected Loud, got {failure:?}");
    };
    // Same mutation-kill as `do_with_takes_the_loud_path`, above.
    assert_eq!(
        loud.message, "DO is not implemented",
        "a construct 4a does implement must not be attributed to a phase; \
         see lib.rs's owned_message"
    );
}

// ---- DO i = TO/BY/FOR (controlled), and DO OVER ----

#[test]
fn a_controlled_loop_runs_to_then_by_then_stops() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"do i = 1 to 3\nsay i\nend"),
        b"1\n2\n3\n".to_vec()
    );
    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"do i = 10 to 1 by -4\nsay i\nend"),
        b"10\n6\n2\n".to_vec(),
        "measured against the oracle, do_loop_forms.rex's own transcript"
    );
}

/// A non-whole control value is legal (Step 2's own table): `do i = 1.5
/// to 3` steps by fractional values, not an error.
#[test]
fn a_controlled_loops_own_values_need_only_be_numeric_not_whole() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"do i = 1.5 to 3\nsay i\nend"),
        b"1.5\n2.5\n".to_vec()
    );
}

/// `BY 0` loops forever -- behaviour to reproduce, not an error
/// (Step 2's own table) -- bounded here by a `LEAVE` so the test itself
/// terminates.
#[test]
fn a_controlled_loop_with_by_0_loops_forever() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"i = 0\ndo j = 1 by 0 to 3\ni = i + 1\nif i > 5 then leave\nend\nsay i"
        ),
        b"6\n".to_vec(),
        "measured against the oracle"
    );
}

#[test]
fn a_controlled_loops_own_for_caps_the_iteration_count_independent_of_to() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"n = 0\ndo i = 1 to 100 for 3\nn = n + 1\nend\nsay n"
        ),
        b"3\n".to_vec()
    );
}

/// The control variable is bound to its own header value **before** the
/// loop's own bound test, even for a loop that ends up running zero
/// iterations -- measured against the oracle.
#[test]
fn the_control_variable_is_bound_even_when_the_loop_never_runs_its_body() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"do i = 5 to 3\nsay 'never'\nend\nsay i"),
        b"5\n".to_vec()
    );
}

/// A compound variable as a `DO` control variable is bound on every
/// pass, through the same tail resolution `say cv.j` already uses, not
/// stored as a simple variable literally named `CV.J`. Measured against
/// the oracle: `1`/`2`/`3` inside the loop, then `4` twice -- the
/// loop's own final bound-test value, read back through the same tail
/// both by `cv.j` and by the literal `cv.7`.
#[test]
fn a_compound_control_variable_is_bound_on_every_pass() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"j = 7\ndo cv.j = 1 to 3\nsay cv.j\nend\nsay cv.j\nsay cv.7"
        ),
        b"1\n2\n3\n4\n4\n".to_vec()
    );
}

/// **Pairs with the test above**: an ordinary simple control variable
/// is unaffected by routing a stem or compound one through
/// `assign_expr_target` -- the fast, unmodified slot write is still the
/// only path a simple control ever takes.
#[test]
fn a_simple_control_variable_still_binds_through_the_fast_slot_path() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"do ii = 1 to 3\nend\nsay ii"),
        b"4\n".to_vec()
    );
}

/// The compound's tail re-resolves fresh on every pass, against
/// whichever *current* value the tail variable holds -- not the tail
/// the loop's header resolved once at setup. Measured against the
/// oracle: the body's own `i = i + 1` moves which tail of `a.` this
/// loop's own read-and-increment step resolves to on the very next
/// pass, so `TO 3` keeps comparing against a fresh, still-default `0`
/// tail instead of the one `a.i` incremented, and the loop runs to `i
/// = 8` (the `LEAVE` bound) rather than stopping at `i = 4`.
#[test]
fn a_compound_control_variables_tail_re_resolves_every_pass() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"a. = 0\ni = 1\ndo a.i = 1 to 3\nif i > 7 then leave\ni = i + 1\nend\nsay i"
        ),
        b"8\n".to_vec()
    );
}

/// **Crosses TO-bound-only termination with a pre-existing stem default
/// that masks a missing write** (review round 1, I3) -- the shape
/// `a_compound_control_variables_tail_re_resolves_every_pass` above does
/// not reach, because that test's own termination is an independent
/// `LEAVE` on `i`, not the loop's own `TO` bound. Here nothing but `TO 7`
/// ends the loop, and `a.`'s own pre-existing default (`a. = 0`) is
/// exactly what would paper over a write that lands nowhere: a `stem_
/// get` miss falls back to that default rather than to `NOVALUE`'s
/// derived-name text, so a broken write does not fail loudly here, it
/// makes the read-back see `0` on every pass and the loop never reach
/// its bound at all.
#[test]
fn a_compound_controls_to_bound_survives_a_masking_stem_default() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"i = 1\na. = 0\nc = 0\ndo a.i = 1 to 7 for 1000\nc = c + 1\nif i = 1 then i = 2\nelse i = 1\nend\nsay c"
        ),
        b"14\n".to_vec()
    );
}

/// A bare stem as a `DO` control variable (`do cv. = 13`) binds through
/// `stem_assign`, the same "replace and rebind" an ordinary `cv. = 13`
/// assignment uses. Paired with the compound tests above because a
/// stem's own spelling has no tail to resolve: it is the shape this fix
/// must leave working, not the one it corrects.
#[test]
fn a_stem_control_variable_binds_through_stem_assign() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"i = 0\ndo cv. = 13\nif i > 10 then leave\ni = i + 1\nend\nsay cv.\nsay i"
        ),
        b"24\n11\n".to_vec()
    );

    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"do cv. = 13\ncv.1 = 99\nleave\nend\nsay cv.1\nsay cv."
        ),
        b"99\n13\n".to_vec()
    );
}

/// **A stem-controlled `DO` inside an `INTERPRET`, which is where the loop
/// runs with no precomputed slot at all.**
#[test]
fn a_stem_control_inside_a_fragment_writes_the_enclosing_frames_slot() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"interpret \"do zq. = 1 to 3; nop; end\"\nsay zq.\n\
              interpret \"do zt. = 1 to 3; nop; end\"\n\
              interpret \"say zt.\"\ninterpret \"say zt.9\"\n"
        ),
        b"4\n4\n4\n".to_vec()
    );
}

/// `LEAVE`/`ITERATE` naming a compound control variable is unaffected
/// by this fix: the name match that selects which loop to unwind is
/// against the control's own spelling, decided independently of
/// whether that spelling's tail is ever resolved. Measured against the
/// oracle for both instructions.
#[test]
fn leave_and_iterate_by_name_still_reach_a_compound_controlled_loop() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"c = 0\nj = 1\ndo i.j = 0 to 6\nc = c + 1\nif c = 2 then leave i.j\nend i.j\nsay c\nsay i.1"
        ),
        b"2\n1\n".to_vec()
    );

    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"c = 0\nj = 1\ndo i.j = 0 to 6\nc = c + 1\nif c = 2 then iterate i.j\nend i.j\nsay c\nsay i.1"
        ),
        b"7\n7\n".to_vec()
    );
}

/// A compound control variable traces its own `>C>` before the setup's
/// `>=>` and again before the re-tested pass's `>V>`, exactly the order
/// `eval_node`'s own `Compound` arm uses for an ordinary read -- the
/// same order this fix's `bind_control` and its read-back both reuse
/// rather than reimplement.
#[test]
fn a_compound_control_variable_traces_its_own_c_line() {
    let mut interp = Interp::new();
    let program =
        parse_program(b"j = 1\ndo cv.j = 1 to 2\nnop\nend".to_vec()).expect("test program parses");
    let program = activate(&mut interp, program);
    interp.set_trace_mode(crate::trace::mode_from_setting(b"i").expect("I is a valid setting"));
    run_activated(&mut interp, &program).expect("test program runs");
    let trace = String::from_utf8_lossy(&interp.trace);
    assert!(
        trace.contains(">C>   CV.J => \"CV.1\"") && trace.contains(">C>     CV.J => \"CV.1\""),
        "expected a >C> line at both the setup indent and the re-tested-pass indent, \
         resolving CV.J to CV.1; trace was:\n{trace}"
    );
}

/// `DO name OVER expr` on a **non-stem** target: a string and a number
/// each iterate exactly once, yielding themselves -- Deviation 1 keeps
/// a stem target out of scope, and this test uses none.
#[test]
fn do_over_a_non_stem_target_iterates_once_yielding_itself() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"do x over 'hello'\nsay x\nend"),
        b"hello\n".to_vec()
    );
    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"do x over 42\nsay x\nend"),
        b"42\n".to_vec()
    );
}

/// `DO OVER ... FOR` on a non-stem target: `FOR 0` skips the one
/// iteration entirely; any `FOR` at least `1` still runs it exactly
/// once (there is only ever one item).
#[test]
fn do_over_for_0_skips_the_single_non_stem_iteration() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"do x over 'hello' for 0\nsay x\nend\nsay 'after'"
        ),
        b"after\n".to_vec()
    );
    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"do x over 'hello' for 5\nsay x\nend"),
        b"hello\n".to_vec()
    );
    // The skipped pass still binds, so the name outlives the loop holding the
    // item rather than whatever it held before.
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"x = 'pre'\ndo x over 'hello' for 0\nsay 'body'\nend\nsay x"
        ),
        b"hello\n".to_vec()
    );
}

/// **The oracle's own bytes for `DO OVER ... FOR`'s count**, captured
/// 2026-08-14 under both `TRACE I` and `TRACE R`, from a fresh oracle run
/// wrapped exactly as `rust/CLAUDE.md` specifies. The oracle prints
/// `>K>   "FOR" => "1"` for the count, and
/// `HeaderRole::OverFor::keyword()` is what decides whether this crate
/// emits it.
#[test]
fn a_do_over_for_echoes_the_for_keyword_the_oracle_prints() {
    let mut interp = Interp::new();
    run_source(
        &mut interp,
        b"trace i\nzs = 'abc'\ndo qq over zs for 1\n  leave\nend\n",
    )
    .expect("the program runs");
    assert_eq!(
        String::from_utf8(interp.trace.clone()).expect("trace is UTF-8"),
        concat!(
            "     2 *-* zs = 'abc'\n",
            "       >L>   \"abc\"\n",
            "       >>>   \"abc\"\n",
            "       >=>   ZS <= \"abc\"\n",
            "     3 *-* do qq over zs for 1\n",
            "       >V>   ZS => \"abc\"\n",
            "       >K>   \"OVER\" => \"abc\"\n",
            "       >L>   \"1\"\n",
            "       >K>   \"FOR\" => \"1\"\n",
            "       >=>     QQ <= \"abc\"\n",
            "     4 *-*   leave\n",
        )
    );

    let mut interp = Interp::new();
    run_source(
        &mut interp,
        b"trace r\nzs = 'abc'\ndo qq over zs for 1\n  leave\nend\n",
    )
    .expect("the program runs");
    assert_eq!(
        String::from_utf8(interp.trace.clone()).expect("trace is UTF-8"),
        concat!(
            "     2 *-* zs = 'abc'\n",
            "       >>>   \"abc\"\n",
            "     3 *-* do qq over zs for 1\n",
            "       >K>   \"OVER\" => \"abc\"\n",
            "       >K>   \"FOR\" => \"1\"\n",
            "     4 *-*   leave\n",
        )
    );
}

/// Deviation 1 (`phase-4-exclusions.txt`): `DO OVER` on a stem does not
/// reproduce the oracle's traversal order, and takes the loud path
/// instead -- detected from `target`'s own syntax (a bare `NAME.`
/// parses as `ExprKind::Stem`), never by evaluating it.
#[test]
fn do_over_a_stem_target_takes_the_loud_path() {
    let mut interp = Interp::new();
    let failure = run_source(&mut interp, b"a.1 = 'x'\ndo v over a.\nsay v\nend").unwrap_err();
    let Failure::Loud(loud) = failure else {
        panic!("expected Loud, got {failure:?}");
    };
    // Same mutation-kill as `do_with_takes_the_loud_path` (above, in this
    // module): `DO`/`LOOP` is implemented regardless of which deviation
    // routed this particular clause to the loud path.
    assert_eq!(
        loud.message, "DO is not implemented",
        "a construct 4a does implement must not be attributed to a phase; \
         see lib.rs's owned_message"
    );
}

/// A stem target wrapped in parens is **also** caught -- corrected after
/// review, which found this task's own comment on the `Over` arm
/// claimed the opposite (`over (a.)` "is not detected"). It is detected:
/// a single parenthesised sub-expression collapses to that
/// sub-expression's own `ExprKind` rather than wrapping it in
/// `ExprKind::List`, so `(a.)` is already `ExprKind::Stem` by the time
/// `loop_header_plan`'s own `matches!` check sees it, with nothing extra
/// needed. The safe direction either way (loud, never a silent
/// divergence), but the comment was wrong about which one it is.
#[test]
fn do_over_a_parenthesised_stem_target_is_also_caught() {
    let mut interp = Interp::new();
    let failure = run_source(&mut interp, b"a.1 = 'x'\ndo v over (a.)\nsay v\nend").unwrap_err();
    let Failure::Loud(loud) = failure else {
        panic!("expected Loud, got {failure:?}");
    };
    // Same mutation-kill as `do_with_takes_the_loud_path` (above, in this
    // module).
    assert_eq!(
        loud.message, "DO is not implemented",
        "a construct 4a does implement must not be attributed to a phase; \
         see lib.rs's owned_message"
    );
}

/// F1 (branch review, Important): `initial`/`to`/`by` are rounded under
/// the *entry* `NUMERIC DIGITS`, once, not stored as their exact parse
/// -- `round_via_unary_plus`'s own doc comment has the oracle citation
/// and both transcripts this mirrors exactly. Masked while `DIGITS`
/// stays constant (every later use re-rounds to the same width, so the
/// exact and the rounded value render identically); this widens
/// `DIGITS` *inside* the loop body, after entry, so only a value
/// rounded at entry -- not the exact parse -- can still show `1.23` on
/// every pass. Mutation killed: removing either `round_via_unary_plus`
/// call in `setup_controlled` (the one on `current`, or the one in
/// `ControlExpr::By`'s own arm) makes the affected probe's second and
/// third lines read the exact, unrounded value instead (`2.23456`/
/// `3.23456` for `current`, or accumulating by the exact `1.2345`
/// instead of the rounded `1.23` for `by`) -- verified by reverting
/// each in turn.
#[test]
fn a_controlled_loops_header_values_round_at_entry_not_the_exact_parse() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"numeric digits 3\ndo i = 1.23456 to 3\nsay i\nnumeric digits 9\nend"
        ),
        b"1.23\n2.23\n".to_vec(),
        "measured against the oracle: current is rounded once at entry \
         (1.23456 -> 1.23), and widening digits inside the loop must \
         not un-round it -- the exact parse would give 2.23456"
    );

    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"numeric digits 3\ndo i = 1 to 9 by 1.2345\nsay i\nnumeric digits 9\nend"
        ),
        b"1\n2.23\n3.46\n4.69\n5.92\n7.15\n8.38\n".to_vec(),
        "measured against the oracle: by is rounded to 1.23 once at \
         entry and accumulated at that width, not the exact 1.2345 -- \
         the loop stops after 8.38 because the next value, 9.61, is \
         past the bound"
    );
}

// ---- DO/LOOP header errors (Step 2's own table, re-measured) ----

#[test]
fn a_non_numeric_control_value_raises_41_1() {
    for (source, found) in [
        (&b"do i = 'a' to 3\nnop\nend"[..], "a"),
        (&b"do i = 1 to 'x'\nnop\nend"[..], "x"),
        (&b"do i = 1 by 'x'\nnop\nend"[..], "x"),
    ] {
        let mut interp = Interp::new();
        let failure = run_source(&mut interp, source).unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (41, 1), "{source:?}");
        assert_eq!(
            raised.additional,
            vec![found.as_bytes().to_vec()],
            "{source:?}"
        );
    }
}

#[test]
fn a_bad_for_expression_raises_26_3() {
    for (source, found) in [
        (&b"do i = 1 to 3 for 'x'\nnop\nend"[..], "x"),
        (&b"do i = 1 to 3 for -1\nnop\nend"[..], "-1"),
        (&b"do i = 1 to 3 for 1.5\nnop\nend"[..], "1.5"),
    ] {
        let mut interp = Interp::new();
        let failure = run_source(&mut interp, source).unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (26, 3), "{source:?}");
        assert_eq!(
            raised.additional,
            vec![found.as_bytes().to_vec()],
            "{source:?}"
        );
    }
}

#[test]
fn a_bad_repetition_count_raises_26_2() {
    for (source, found) in [
        (&b"do 'a'\nnop\nend"[..], "a"),
        (&b"do -1\nnop\nend"[..], "-1"),
        (&b"do 2.5\nnop\nend"[..], "2.5"),
    ] {
        let mut interp = Interp::new();
        let failure = run_source(&mut interp, source).unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (26, 2), "{source:?}");
        assert_eq!(
            raised.additional,
            vec![found.as_bytes().to_vec()],
            "{source:?}"
        );
    }
}

/// F3 (branch review, Important): a bare `DO` count and a `FOR` count
/// are validated under the *current* `NUMERIC DIGITS`, not a fixed
/// width -- `whole_nonneg`'s own doc comment has the oracle citation
/// and the two oracle-measured transcripts this mirrors
/// (`numeric digits 3; do 12345; end` is 26.2 rc 230; the `FOR` shape
/// is 26.3). Mutation killed: reverting `whole_nonneg` to convert
/// under `rexx_num::ARGUMENT_DIGITS` (18) instead of the activation's
/// own `settings.digits()` makes both of these run clean (`after`
/// prints, no error) instead of raising, since `12345`/`54321` both
/// fit comfortably under 18 digits and only fail to fit under 3.
#[test]
fn a_repetition_or_for_count_is_validated_under_the_current_digits_not_a_fixed_width() {
    let mut interp = Interp::new();
    let failure =
        run_source(&mut interp, b"numeric digits 3\ndo 12345\nend\nsay 'after'").unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (26, 2));

    let mut interp = Interp::new();
    let failure = run_source(
        &mut interp,
        b"numeric digits 3\ndo i = 1 to 99999 for 12345\nend\nsay 'after'",
    )
    .unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (26, 3));
}

// ---- WHILE/UNTIL ----

#[test]
fn do_while_tests_the_condition_before_the_body_and_do_until_after() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"i = 0\ndo while i < 2\ni = i + 1\nsay i\nend"),
        b"1\n2\n".to_vec()
    );
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"i = 5\ndo until i >= 7\ni = i + 1\nsay i\nend"
        ),
        b"6\n7\n".to_vec(),
        "measured against the oracle, do_loop_forms.rex's own transcript"
    );
}

#[test]
fn a_while_condition_that_is_not_0_or_1_raises_34_3_not_34_1() {
    let mut interp = Interp::new();
    let failure = run_source(&mut interp, b"do while 'x'\nnop\nend").unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (34, 3));
    assert_eq!(raised.additional, vec![b"x".to_vec()]);
}

#[test]
fn an_until_condition_that_is_not_0_or_1_raises_34_4() {
    let mut interp = Interp::new();
    let failure = run_source(&mut interp, b"do until 'x'\nnop\nend").unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (34, 4));
    assert_eq!(raised.additional, vec![b"x".to_vec()]);
}

/// A comma-list `WHILE`/`UNTIL` condition is 34.6, never 34.3/34.4 --
/// `eval_condition`'s own rule, reused unchanged from `IF`/`WHEN`.
#[test]
fn a_comma_list_while_condition_raises_34_6_not_34_3() {
    let mut interp = Interp::new();
    let failure = run_source(&mut interp, b"do while 'x', 1\nnop\nend").unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (34, 6));
}

/// **The discriminating measurement for this whole section**: `UNTIL`
/// is tested at the *bottom* of the loop, so its own failure is
/// attributed to the `END`'s own clause, not the `DO`'s -- a loop that
/// evaluated `UNTIL` eagerly, at the top like `WHILE`, would still
/// produce the same raised number and the same exit code, and only the
/// echoed clause reveals the difference. Measured against the oracle:
/// `do until 'x' / end` echoes `end`, `do while 'x' / end` echoes the
/// `do` line.
#[test]
fn until_is_attributed_to_the_end_clause_while_while_is_attributed_to_the_do_clause() {
    let mut interp = Interp::new();
    run_source(&mut interp, b"do until 'x'\nnop\nend").unwrap_err();
    let FailureSite::Clause { line, text, .. } = interp.failure_site.expect("a site was resolved")
    else {
        panic!("not a clause site")
    };
    assert_eq!(line, 3, "the END's own line");
    assert_eq!(text, b"end".to_vec(), "the END's own clause, not the DO's");

    let mut interp = Interp::new();
    run_source(&mut interp, b"do while 'x'\nnop\nend").unwrap_err();
    let FailureSite::Clause { line, text, .. } = interp.failure_site.expect("a site was resolved")
    else {
        panic!("not a clause site")
    };
    assert_eq!(line, 1, "the DO's own line");
    assert_eq!(text, b"do while 'x'".to_vec(), "the DO's own clause");
}

/// **`ITERATE` jumps to the loop's own bottom-of-iteration bookkeeping,
/// not to the top of the next pass** -- measured against the oracle,
/// this exact program terminates immediately with `n` still `1`, rather
/// than looping forever. A design that instead skipped `UNTIL` for the
/// interrupted iteration would hang on this program (`n` would keep
/// advancing past `1` and `UNTIL n = 1` would never hold again), which
/// is the mutation this test is built to catch -- not run, for the
/// obvious reason a hang cannot be asserted against, but predicted and
/// confirmed absent by construction: `do_body_outcome`'s own `Iterate`
/// arm answers `Ok(None)`, the *same* value `Flow::Next` answers, so
/// `run_repeating`'s own `UNTIL` check below it runs on both paths
/// identically.
#[test]
fn iterate_reaches_untils_own_bottom_of_iteration_test_rather_than_skipping_it() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"n = 0\ndo until n = 1\nn = n + 1\nif n = 1 then iterate\nsay 'unreached'\nend\nsay 'done' n"
        ),
        b"done 1\n".to_vec()
    );
}

// ---- End arm ----

#[test]
fn a_do_loops_own_end_is_a_pure_marker_never_independently_dispatched() {
    // Reached at all only if something jumped straight onto it, which
    // nothing in this crate does -- covered indirectly by every DO/LOOP
    // test above completing without the loud failure this arm used to
    // give before Task 11. Direct coverage: a labelled LOOP closes
    // cleanly (EndStyle::LabeledDo is exercised on the same code path
    // EndStyle::Do/Loop are).
    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"do label x\nsay 'ok'\nend"),
        b"ok\n".to_vec()
    );
}
