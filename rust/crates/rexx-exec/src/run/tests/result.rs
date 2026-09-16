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

// ---- EXIT ----

#[test]
fn exit_with_and_without_an_expression() {
    let mut interp = Interp::new();
    let value = run_source(&mut interp, b"exit").expect("bare exit runs");
    assert_eq!(value, None);

    let mut interp = Interp::new();
    let value = run_source(&mut interp, b"exit 42").expect("exit with a result runs");
    let value = value.expect("EXIT 42 carries a result");
    assert_eq!(&*interp.to_text(value), b"42");
}

#[test]
fn exit_code_for_converts_the_result_the_way_the_oracle_does() {
    // The whole transcript this task's report re-verifies against the
    // oracle: a bare EXIT and a huge, fractional or non-numeric result
    // all leave the code at 0; a literal in i32 range converts exactly,
    // and an arithmetic result (EXIT's own unary minus counts) is
    // already rounded to the active NUMERIC DIGITS by the time it gets
    // here, which is where the negative-vs-positive asymmetry the report
    // measured against the oracle comes from.
    let mut interp = Interp::new();
    assert_eq!(interp.exit_code_for(None), 0, "a bare EXIT");

    let value = run_source(&mut interp, b"exit 2147483647")
        .unwrap()
        .expect("a result");
    assert_eq!(
        interp.exit_code_for(Some(value)),
        2147483647,
        "INT32_MAX exactly"
    );

    let value = run_source(&mut interp, b"exit 2147483648")
        .unwrap()
        .expect("a result");
    assert_eq!(
        interp.exit_code_for(Some(value)),
        0,
        "one past INT32_MAX falls back to 0"
    );

    let value = run_source(&mut interp, b"exit 5.9")
        .unwrap()
        .expect("a result");
    assert_eq!(interp.exit_code_for(Some(value)), 0, "fractional");

    let value = run_source(&mut interp, b"exit 5.0")
        .unwrap()
        .expect("a result");
    assert_eq!(
        interp.exit_code_for(Some(value)),
        5,
        "a whole number spelled with a point"
    );

    let value = run_source(&mut interp, b"exit 'abc'")
        .unwrap()
        .expect("a result");
    assert_eq!(interp.exit_code_for(Some(value)), 0, "non-numeric");

    // The asymmetry: -2147483647 is 0 - 2147483647, rounded to the
    // *active* DIGITS (9 by default) the moment it is created, landing
    // one past INT32_MIN; raising DIGITS before the subtraction removes
    // the rounding and the failure with it.
    let value = run_source(&mut interp, b"exit -2147483647")
        .unwrap()
        .expect("a result");
    assert_eq!(
        interp.exit_code_for(Some(value)),
        0,
        "rounded to 9 digits at creation, one past INT32_MIN"
    );

    let value = run_source(&mut interp, b"numeric digits 20\nexit -2147483647")
        .unwrap()
        .expect("a result");
    assert_eq!(
        interp.exit_code_for(Some(value)),
        -2147483647,
        "no rounding at DIGITS 20, exact"
    );
}

// ---- CALL and RETURN (Task 3) ----

/// D9r's default, and the one property a witness without variables in it
/// cannot check: a callee with no `PROCEDURE` reads the caller's
/// variables **and its writes survive the return**. An implementation
/// that gave every callee a fresh pool passes `call sub` / `sub: say
/// 'callee'` and fails this.
#[test]
fn a_routine_without_procedure_shares_the_callers_pool() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"v = 'caller-v'\ncall sub\nsay 'caller sees:' v w\nexit\n\
              sub:\nsay 'callee sees v:' v\nw = 'callee-w'\nreturn\n",
        ),
        b"callee sees v: caller-v\ncaller sees: caller-v callee-w\n".to_vec()
    );
}

/// The body selector's own reason to exist: the callee runs *its* clauses,
/// from the label, and the caller resumes after the `CALL` rather than at
/// the top.
#[test]
fn a_called_label_runs_its_own_clauses_not_the_main_body() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"call sub\nsay 'main'\nexit\nsub: say 'callee'\nreturn\n"
        ),
        b"callee\nmain\n".to_vec()
    );
}

/// A name bound at run time inside the callee is part of the same shared
/// pool, which needs the *name* to cross the return as well as the slot
/// -- `Activation::extra`, cloned in and moved back out. Measured on the
/// oracle both ways round; this is the direction that fails if `extra`
/// is left behind with the callee.
#[test]
fn a_name_bound_by_interpret_inside_a_callee_survives_the_return() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"call sub\ninterpret \"say zork\"\nexit\nsub:\ninterpret \"zork = 42\"\nreturn\n",
        ),
        b"42\n".to_vec()
    );

    // And inward, which is the half that already worked: a name the
    // caller bound at run time is readable in the callee.
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"interpret \"zork = 42\"\ncall sub\nexit\nsub:\ninterpret \"say zork\"\nreturn\n",
        ),
        b"42\n".to_vec()
    );
}

/// `RESULT` is settled on **return**, not at the call: the callee sees
/// the caller's own pre-call value, and only the return overwrites it.
/// A bare `return` drops it, so it reads back as its own derived name.
#[test]
fn result_is_settled_on_return_and_a_bare_return_drops_it() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"result = 'before'\ncall sub\nsay 'after result=' result\n\
              call bare\nsay 'after bare result=' result\nexit\n\
              sub:\nsay 'inside result=' result\nreturn 42\nbare:\nreturn\n",
        ),
        b"inside result= before\nafter result= 42\nafter bare result= RESULT\n".to_vec()
    );
}

/// The two ways out of a callee that are *not* a return, both measured:
/// an explicit `EXIT`, and the body simply running out of instructions.
/// Either ends the program, so the caller's next clause never runs --
/// which is why `Ended` distinguishes them from `Returned` at all.
#[test]
fn exiting_or_falling_off_the_end_inside_a_callee_ends_the_program() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"call sub\nsay 'never'\nsub:\nsay 'in sub'\nexit\n"
        ),
        b"in sub\n".to_vec()
    );

    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"call sub\nsay 'never'\nexit\nsub:\nsay 'in sub'\n"
        ),
        b"in sub\n".to_vec()
    );
}

/// A `RETURN` in the main body, with no active call: it ends the program
/// with its value, exactly like `EXIT`. Measured at rc 5.
#[test]
fn a_return_in_the_main_body_ends_the_program_with_its_value() {
    let mut interp = Interp::new();
    let value = run_source(&mut interp, b"say 'a'\nreturn 5\nsay 'b'\n").expect("the program runs");
    assert_eq!(interp.out, b"a\n".to_vec());
    assert_eq!(interp.exit_code_for(value), 5);

    let mut interp = Interp::new();
    let value = run_source(&mut interp, b"say 'a'\nreturn\nsay 'b'\n").expect("the program runs");
    assert_eq!(interp.out, b"a\n".to_vec());
    assert_eq!(interp.exit_code_for(value), 0);
}

/// `RETURN` unwinds to the **activation** boundary and past every block
/// frame in between -- which is exactly what `LEAVE` does not do. Both
/// enclosing constructs here would consume a `Flow::Leave`; neither may
/// consume a `Flow::Return`.
#[test]
fn a_return_escapes_every_enclosing_block_in_the_callee() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"call sub\nsay 'back'\nexit\nsub:\ndo forever\nreturn\nend\n",
        ),
        b"back\n".to_vec()
    );

    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"call sub\nsay 'back'\nexit\nsub:\nselect\nwhen 1 = 1 then return\notherwise nop\nend\n",
        ),
        b"back\n".to_vec()
    );
}

/// `NUMERIC` is inherited at call time and never written back -- measured
/// with `digits 7` outside and `digits 3` inside. The second `say` is the
/// discriminating one: an implementation that shared one `Settings`
/// would print `3` there.
#[test]
fn numeric_settings_are_inherited_by_a_callee_and_not_written_back() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"numeric digits 7\ncall sub\nsay 1/3\nexit\n\
              sub:\nsay 1/3\nnumeric digits 3\nsay 1/3\nreturn\n",
        ),
        b"0.3333333\n0.333\n0.3333333\n".to_vec()
    );
}

/// I4: `TRACE` moved onto `Activation`, so a callee's own `trace off`
/// dies with the callee. The caller's clauses echo again afterwards,
/// which is what a single `Interp`-wide field could not do.
#[test]
fn a_callees_trace_setting_does_not_survive_its_return() {
    let mut interp = Interp::new();
    run_source_traced(
        &mut interp,
        b"call sub\nsay 'after'\nexit\nsub:\ntrace off\nsay 'quiet'\nreturn\n",
    )
    .expect("the program runs");
    assert_eq!(
        interp.trace,
        b"     1 *-* call sub\n\
          \x20    4 *-*   sub:\n\
          \x20    5 *-*   trace off\n\
          \x20    2 *-* say 'after'\n\
          \x20      >>>   \"after\"\n\
          \x20    3 *-* exit\n"
            .to_vec(),
        "the callee's `trace off` must silence only the callee"
    );
}

/// **The two trace-line indents Task 9 added, pinned here because
/// nothing else in the tree can pin them** (review round 1, F1).
/// `tests/trace_oracle.rs`'s witnesses and `tests/corpus.rs` both
/// compare through DEVIATION 0's `normalize_stderr`, which collapses
/// exactly the space run these lines differ in -- measured, replacing
/// the `do_indent`/`loop_indent` split in `loop_advance` with
/// `loop_indent` alone, or emitting `>F>` two columns in, leaves both
/// harnesses green while diverging from the oracle byte for byte. A
/// unit test's own `assert_eq!` is outside either comparison function,
/// which is what makes this the instrument for the job -- the same
/// argument `phase-4-exclusions.txt`'s DEVIATION 0 already makes for
/// the pinned shallow-depth indent witnesses.
#[test]
fn task_9s_two_new_indents_are_the_oracles_own_and_normalisation_cannot_see_them() {
    // A `Controlled` loop: the setup assignment prints at the `DO`
    // clause's own indent (3 blanks after `>=>`) and every re-tested
    // pass's four lines print two further in (5 blanks). Collapsing the
    // two to one number is what this fails on.
    let mut interp = Interp::new();
    run_source(&mut interp, b"trace i\ndo ii = 1 to 2\n  nop\nend\n").expect("the program runs");
    assert_eq!(
        String::from_utf8(interp.trace.clone()).expect("trace is UTF-8"),
        concat!(
            "     2 *-* do ii = 1 to 2\n",
            "       >L>   \"1\"\n",
            "       >L>   \"2\"\n",
            "       >K>   \"TO\" => \"2\"\n",
            "       >=>   II <= \"1\"\n",
            "     3 *-*   nop\n",
            "     4 *-* end\n",
            "     2 *-* do ii = 1 to 2\n",
            "       >V>     II => \"1\"\n",
            "       >>>     \"1\"\n",
            "       >>>     \"2\"\n",
            "       >=>     II <= \"2\"\n",
            "     3 *-*   nop\n",
            "     4 *-* end\n",
            "     2 *-* do ii = 1 to 2\n",
            "       >V>     II => \"2\"\n",
            "       >>>     \"2\"\n",
            "       >>>     \"3\"\n",
            "       >=>     II <= \"3\"\n",
        )
    );

    // `>F>`: the caller's own indent, where the callee's own `>>>` for
    // the same value sits two further in on the line above it.
    let mut interp = Interp::new();
    run_source(
        &mut interp,
        b"trace i\nzz = twice(2)\nexit\ntwice:\nuse arg nn\nreturn nn + nn\n",
    )
    .expect("the program runs");
    assert_eq!(
        String::from_utf8(interp.trace.clone()).expect("trace is UTF-8"),
        concat!(
            "     2 *-* zz = twice(2)\n",
            "       >L>   \"2\"\n",
            "       >A>   \"2\"\n",
            "     4 *-*   twice:\n",
            "     5 *-*   use arg nn\n",
            "       >>>     \"2\"\n",
            "       >=>     NN <= \"2\"\n",
            "     6 *-*   return nn + nn\n",
            "       >V>     NN => \"2\"\n",
            "       >V>     NN => \"2\"\n",
            "       >O>     \"+\" => \"4\"\n",
            "       >>>     \"4\"\n",
            "       >F>   TWICE => \"4\"\n",
            "       >>>   \"4\"\n",
            "       >=>   ZZ <= \"4\"\n",
            "     3 *-* exit\n",
        )
    );
}

/// D2r's rule, at **three** caller indents rather than one: a callee's
/// clauses echo at the calling clause's own printed indent plus two.
/// `2 x depth` agrees with all of this at caller indent 0 and predicts 2
/// where the truth is 4 and 6, which is why one shape is not enough.
/// Every byte here was captured from the oracle.
#[test]
fn a_callees_clauses_echo_at_the_calling_clauses_indent_plus_two() {
    // Flat caller, two levels deep: 0 -> 2 -> 4.
    let mut interp = Interp::new();
    run_source_traced(
        &mut interp,
        b"aa = 1\ncall one\nexit\none:\nbb = 2\ncall two\nreturn\ntwo:\ncc = 3\nreturn\n",
    )
    .expect("the program runs");
    assert_eq!(
        interp.trace,
        b"     1 *-* aa = 1\n\
          \x20      >>>   \"1\"\n\
          \x20    2 *-* call one\n\
          \x20    4 *-*   one:\n\
          \x20    5 *-*   bb = 2\n\
          \x20      >>>     \"2\"\n\
          \x20    6 *-*   call two\n\
          \x20    8 *-*     two:\n\
          \x20    9 *-*     cc = 3\n\
          \x20      >>>       \"3\"\n\
          \x20   10 *-*     return\n\
          \x20    7 *-*   return\n\
          \x20    3 *-* exit\n"
            .to_vec()
    );

    // Caller two `DO` blocks deep: the callee echoes at 6, where
    // `2 x depth` predicts 2.
    let mut interp = Interp::new();
    run_source_traced(
        &mut interp,
        b"do\ndo\ncall sub\nend\nend\nexit\nsub:\ndd = 4\nreturn\n",
    )
    .expect("the program runs");
    assert_eq!(
        interp.trace,
        b"     1 *-* do\n\
          \x20    2 *-*   do\n\
          \x20    3 *-*     call sub\n\
          \x20    7 *-*       sub:\n\
          \x20    8 *-*       dd = 4\n\
          \x20      >>>         \"4\"\n\
          \x20    9 *-*       return\n\
          \x20    4 *-*   end\n\
          \x20    5 *-* end\n\
          \x20    6 *-* exit\n"
            .to_vec()
    );
}

/// **Review finding C1, Task 4 fix round 1.** `current_value_indent` is
/// a fourth piece of level state `Interp::invoke_call` must restore on
/// the way out, alongside `activation_indent`/`indent_offset`/
/// `clause_line_override` (that function's own doc comment) -- and this
/// is the shape that tells a version missing the restore apart from a
/// correct one: **two** internal-function calls inside *one* clause
/// (`ExprKind::Call`, Task 4). Before Task 4 at most one activation
/// could be entered per clause, and the *next* clause's own
/// `Op::Clause`'s region re-set the field before anything read it, so
/// the gap was unobservable through `CALL` alone. Without the restore,
/// `g`'s own base indent -- and everything computed from it: its
/// clauses, its `RETURN`'s own value trace, and the enclosing `say`
/// clause's own final `>>>` -- is derived from `f`'s last clause instead
/// of the caller's own.
#[test]
fn current_value_indent_is_restored_after_a_nested_expression_call() {
    let mut interp = Interp::new();
    run_source_traced(
        &mut interp,
        b"say f(1) + g(2)\nexit\nf: return 1\ng: return 2\n",
    )
    .expect("the program runs");
    assert_eq!(
        interp.trace,
        b"     1 *-* say f(1) + g(2)\n\
          \x20    3 *-*   f:\n\
          \x20    3 *-*   return 1\n\
          \x20      >>>     \"1\"\n\
          \x20    4 *-*   g:\n\
          \x20    4 *-*   return 2\n\
          \x20      >>>     \"2\"\n\
          \x20      >>>   \"3\"\n\
          \x20    2 *-* exit\n"
            .to_vec()
    );
}

/// `current_value_indent`'s own sibling field, found the identical way
/// (Task 6 fix round 2): `current_clause_line` (bundled with it into
/// `ClauseState`, whose own doc comment states the property that puts
/// both fields in one save/restore rather than two) is a piece of level
/// state `Interp::invoke_call` must restore on the way out, and shipped
/// once already without that restore -- the second field of this exact
/// shape to do so, after `current_value_indent` itself went unrestored
/// until the test just above this one caught it at Task 4.
#[test]
fn current_clause_line_is_restored_after_a_nested_expression_call() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"say f(1) + g(2)\n\
              exit\n\
              f:\n\
              say 'in f'\n\
              return 1\n\
              g:\n\
              say 'sigl in g:' sigl\n\
              return 2\n",
        ),
        b"in f\nsigl in g: 1\n3\n".to_vec()
    );
}

/// A returned value traces **twice**, at two different indents: once as
/// the `RETURN`'s own value in the callee, once as the call's result in
/// the caller. A bare `return` traces neither.
#[test]
fn a_returned_value_traces_in_the_callee_and_again_in_the_caller() {
    let mut interp = Interp::new();
    run_source_traced(&mut interp, b"call sub\nexit\nsub:\nreturn 42\n").expect("the program runs");
    assert_eq!(
        interp.trace,
        b"     1 *-* call sub\n\
          \x20    3 *-*   sub:\n\
          \x20    4 *-*   return 42\n\
          \x20      >>>     \"42\"\n\
          \x20      >>>   \"42\"\n\
          \x20    2 *-* exit\n"
            .to_vec()
    );

    let mut interp = Interp::new();
    run_source_traced(&mut interp, b"call sub\nexit\nsub:\nreturn\n").expect("the program runs");
    assert_eq!(
        interp.trace,
        b"     1 *-* call sub\n\
          \x20    3 *-*   sub:\n\
          \x20    4 *-*   return\n\
          \x20    2 *-* exit\n"
            .to_vec(),
        "a bare return produces no value line at either indent"
    );
}

/// The composition nobody had measured: a `CALL` **inside** an
/// `INTERPRET` fragment. Each activation's echo carries its *own* line,
/// so the enclosing fragment's line override has to be cleared for the
/// callee -- leaving it in force prints the callee's three clauses as
/// line 1 instead of 3, 4 and 5.
#[test]
fn a_call_inside_a_fragment_echoes_each_activations_own_line() {
    let mut interp = Interp::new();
    run_source_traced(
        &mut interp,
        b"interpret \"call sub\"\nexit\nsub:\nff = 7\nreturn\n",
    )
    .expect("the program runs");
    assert_eq!(
        interp.trace,
        b"     1 *-* interpret \"call sub\"\n\
          \x20      >>>   \"call sub\"\n\
          \x20    1 *-* call sub\n\
          \x20    3 *-*   sub:\n\
          \x20    4 *-*   ff = 7\n\
          \x20      >>>     \"7\"\n\
          \x20    5 *-*   return\n\
          \x20    2 *-* exit\n"
            .to_vec()
    );
}

/// The other direction: an `INTERPRET` **inside** a called routine. The
/// fragment adds no indent of its own on top of the activation's, so all
/// four of the callee's lines sit at 2.
#[test]
fn an_interpret_inside_a_callee_runs_at_the_callees_own_level() {
    let mut interp = Interp::new();
    run_source_traced(
        &mut interp,
        b"call sub\nexit\nsub:\ninterpret \"gg = 8\"\nreturn\n",
    )
    .expect("the program runs");
    assert_eq!(
        interp.trace,
        b"     1 *-* call sub\n\
          \x20    3 *-*   sub:\n\
          \x20    4 *-*   interpret \"gg = 8\"\n\
          \x20      >>>     \"gg = 8\"\n\
          \x20    4 *-*   gg = 8\n\
          \x20      >>>     \"8\"\n\
          \x20    5 *-*   return\n\
          \x20    2 *-* exit\n"
            .to_vec()
    );
}

/// I12's `CALL` half: each activation seals its own level, so the report
/// echoes one clause per activation, innermost first, each at its own
/// indent. Two calls deep from inside a `DO` gives 6, 4, 2 -- the same
/// three-deep transcript the oracle prints.
#[test]
fn the_report_echoes_one_clause_per_activation_innermost_first() {
    let mut interp = Interp::new();
    run_source(
        &mut interp,
        b"do\ncall one\nend\nexit\none:\ncall two\nreturn\ntwo:\nsay 1/0\nreturn\n",
    )
    .unwrap_err();
    let sealed: Vec<(usize, Vec<u8>, usize)> = interp
        .failure_sites
        .iter()
        .chain(interp.failure_site.iter())
        .map(|s| {
            (
                s.line().expect("a clause site"),
                s.text().to_vec(),
                s.indent().expect("a clause site"),
            )
        })
        .collect();
    assert_eq!(
        sealed,
        vec![
            (9, b"say 1/0".to_vec(), 6),
            (6, b"call two".to_vec(), 4),
            (2, b"call one".to_vec(), 2),
        ]
    );
}

/// Arguments are evaluated in the caller, before the callee starts. Not
/// observable through `USE ARG`/`ARG()` in this phase -- both still fail
/// loudly -- but a failing argument is: the condition is 42.3 and it is
/// reported against the `CALL` clause, not against anything in `sub`.
#[test]
fn a_calls_arguments_are_evaluated_in_the_caller() {
    let mut interp = Interp::new();
    let failure = run_source(&mut interp, b"call sub 1/0\nexit\nsub:\nreturn\n").unwrap_err();
    assert!(
        matches!(&failure, Failure::Raised(raised) if raised.number == 42),
        "an argument that raises must surface as its own condition, not run the callee: {failure:?}"
    );
    let site = interp.failure_site.expect("a site was resolved");
    assert_eq!((site.line(), site.text()), (Some(1), &b"call sub 1/0"[..]));
}

/// The quoted form bypasses the label search entirely, so `call "SUB"`
/// with `sub:` present is *not* a call. This crate's answer is the loud
/// builtin/external fallback naming `4c`; the oracle's own is Error 43.1,
/// which is a claim about what is *not* a builtin either and so not
/// this crate's to make.
#[test]
fn a_quoted_call_name_never_reaches_the_label_table() {
    let mut interp = Interp::new();
    let failure = run_source(
        &mut interp,
        b"call \"SUB\"\nexit\nsub:\nsay 'ran'\nreturn\n",
    )
    .unwrap_err();
    assert!(
        matches!(&failure, Failure::Raised(raised) if raised.number == 43 && raised.sub == 1),
        "expected the oracle's own 43.1, got {failure:?}"
    );
    assert!(interp.out.is_empty(), "the label must not have run");

    // The unquoted spelling of the same name does reach it, which is
    // what makes the assertion above about the quotes and not about
    // `sub:` being unreachable.
    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"call SUB\nexit\nsub:\nsay 'ran'\nreturn\n"),
        b"ran\n".to_vec()
    );
}

/// `CALL (expr)` searches the label table, but with the value **verbatim**
/// -- the two halves pull opposite ways from the quoted form and both are
/// measured. Lower case finds nothing even though `sub:` is stored
/// upcased.
#[test]
fn a_dynamic_call_target_searches_labels_with_the_value_verbatim() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"nm = 'SUB'\ncall (nm)\nexit\nsub:\nsay 'ran'\nreturn\n"
        ),
        b"ran\n".to_vec()
    );

    let mut interp = Interp::new();
    let failure = run_source(
        &mut interp,
        b"nm = 'sub'\ncall (nm)\nexit\nsub:\nsay 'ran'\nreturn\n",
    )
    .unwrap_err();
    assert!(
        matches!(&failure, Failure::Raised(raised) if raised.number == 43 && raised.sub == 1),
        "the unupcased value must not match the upcased label: {failure:?}"
    );
    assert!(interp.out.is_empty(), "the label must not have run");
}
