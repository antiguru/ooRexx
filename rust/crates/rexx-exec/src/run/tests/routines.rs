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

/// The body selector's `Some(index)` half, which no execution path can
/// construct yet (`Activation::body`'s own doc has why). Exercised
/// directly so the arm is not merely written: a parsed `::routine` has a
/// body, and the selector resolves to *that* body rather than to `main`.
#[test]
fn the_body_selector_resolves_a_routine_directive_and_rejects_a_bad_index() {
    let program = parse_program(b"call foo\nexit\n::routine foo\nsay 'in foo'\n".to_vec()).unwrap();
    let main = body_of(&program, None).expect("the main body always resolves");
    assert_eq!(main.instructions.len(), program.main.instructions.len());

    let routine = body_of(&program, Some(0)).expect("the routine directive has a body");
    assert_eq!(
        routine.instructions.len(),
        1,
        "the routine's own body is its one `say`, not the main body's two clauses"
    );
    assert!(
        !std::ptr::eq(routine, main),
        "a routine selector must not resolve to the main body"
    );

    assert!(
        body_of(&program, Some(1)).is_none(),
        "an out-of-range selector resolves to nothing rather than panicking"
    );
}

// ---- builtin dispatch ----

/// A builtin reached through every call form the resolution serves, and
/// the one form that must **not** reach it.
/// ```text
/// say length('abc')      3            rc 0
/// say Length('abcd')     4            rc 0     (a symbol target upcases)
/// say "LENGTH"('abc')    3            rc 0     (a literal target matches verbatim)
/// say "length"('abc')    Error 43.1   rc 213   (and so does not match)
/// call length 'abc'      RESULT 3     rc 0
/// call "LENGTH" 'abc'    RESULT 3     rc 0
/// ```
#[test]
fn length_dispatches_from_every_call_form_that_reaches_the_builtin_table() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"say length('abc')\nsay Length('abcd')"),
        b"3\n4\n".to_vec(),
        "the expression form, at both source spellings of the symbol"
    );

    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"say \"LENGTH\"('abc')"),
        b"3\n".to_vec(),
        "a literal target matches the table verbatim"
    );

    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"call length 'abc'\nsay result"),
        b"3\n".to_vec(),
        "`CALL` settles RESULT from the builtin's own value"
    );

    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"call \"LENGTH\" 'abc'\nsay result"),
        b"3\n".to_vec(),
        "and so does `CALL` with a literal target"
    );

    let mut interp = Interp::new();
    let failure = run_source(&mut interp, b"say \"length\"('abc')").unwrap_err();
    assert!(
        matches!(&failure, Failure::Raised(raised) if raised.number == 43 && raised.sub == 1),
        "a lowercase literal target matches no builtin, so it is the \
         oracle's own 43.1; got {failure:?}"
    );
}

/// **The name a stream builtin looks up is rooted while the stream object is
/// built from it.** `STREAM`'s state and description queries keep nothing in
/// the table, so each builds a fresh object from the name; under a collection
/// at every allocation both answer as under none. The stdout is the
/// oracle's.
#[test]
fn a_stream_builtins_name_survives_a_collection_at_every_allocation() {
    let text = b"say stream('a_stream_name_long_enough_for_the_heap.txt', 's')\n\
        say stream('a_stream_name_long_enough_for_the_heap.txt', 'd')\n"
        .to_vec();
    let name = "/tmp/stream_name_rooted.rex";
    let plain = crate::run_program(name, text.clone(), crate::Invocation::none());
    assert_eq!(
        plain.exit_code,
        0,
        "{}",
        String::from_utf8_lossy(&plain.stderr)
    );
    assert_eq!(
        String::from_utf8_lossy(&plain.stdout),
        "UNKNOWN\nUNKNOWN:\n"
    );
    let swept = crate::run_program_collect_every_alloc(name, text, crate::Invocation::none());
    assert_eq!(swept.exit_code, plain.exit_code);
    assert_eq!(swept.stdout, plain.stdout);
    assert_eq!(swept.stderr, plain.stderr);
    assert!(swept.collections > 0, "the stress mode did not collect");
}

// ---- `::ROUTINE` dispatch ----

/// Runs `source` through the real entry point, which is the only path
/// that installs directives -- this module's own `activate` pushes an
/// activation directly and never sees one.
pub(super) fn routine_program(source: &[u8]) -> crate::Outcome {
    crate::run_program(
        "/tmp/routine.rex",
        source.to_vec(),
        crate::Invocation::none(),
    )
}

/// The whole resolution order in one program, and the order is what each
/// line is for rather than the dispatch: every name below resolves to
/// **something** on this crate, so a wrong order is a wrong answer and
/// not a failure.
#[test]
fn the_resolution_order_is_label_then_builtin_then_routine() {
    let outcome = routine_program(
        b"call max 1, 9\n\
          say result\n\
          call zorkolo\n\
          say result\n\
          call 'ZORKOLO'\n\
          say result\n\
          call 'MAX' 1, 9\n\
          say result\n\
          exit\n\
          zorkolo:\n\
          return 'LABEL'\n\
          ::routine max\n\
          return 'ROUTINE-MAX'\n\
          ::routine zorkolo\n\
          return 'ROUTINE'\n",
    );
    assert_eq!(
        outcome.exit_code,
        0,
        "stderr: {}",
        String::from_utf8_lossy(&outcome.stderr)
    );
    assert_eq!(outcome.stdout, b"9\nLABEL\nROUTINE\n9\n".to_vec());
}

/// The routine lookup upcases both sides, where the builtin step in front
/// of it is case-sensitive -- and the second half is what makes the first
/// observable at all, since a `::routine 'max'` is only reachable because
/// `call 'max'` misses the builtin table.
#[test]
fn a_routine_lookup_upcases_both_sides_where_the_builtin_lookup_does_not() {
    let outcome = routine_program(
        b"call 'ZORK'\n\
          say result\n\
          call zork\n\
          say result\n\
          call 'max' 1, 9\n\
          say result\n\
          ::routine 'zork'\n\
          return 'HIT'\n\
          ::routine 'max'\n\
          return 'ROUTINE-lower-max'\n",
    );
    assert_eq!(
        outcome.exit_code,
        0,
        "stderr: {}",
        String::from_utf8_lossy(&outcome.stderr)
    );
    assert_eq!(
        outcome.stdout,
        b"HIT\nHIT\nROUTINE-lower-max\n".to_vec(),
        "an upcased-both-sides lookup finds all three"
    );
}

/// A `::ROUTINE` gets a pool of its own, and a `CALL`ed label does not --
/// the pair, because the isolating half alone passes just as well against
/// an implementation that isolates everything.
#[test]
fn a_routine_has_its_own_pool_and_a_called_label_shares_the_callers() {
    let outcome = routine_program(
        b"vv = 'CALLER'\n\
          call rtn\n\
          say vv\n\
          call lbl\n\
          say vv\n\
          exit\n\
          lbl:\n\
          say vv\n\
          vv = 'LABEL-WROTE'\n\
          return\n\
          ::routine rtn\n\
          say vv\n\
          vv = 'ROUTINE-WROTE'\n\
          return\n",
    );
    assert_eq!(
        outcome.exit_code,
        0,
        "stderr: {}",
        String::from_utf8_lossy(&outcome.stderr)
    );
    assert_eq!(
        outcome.stdout,
        b"VV\nCALLER\nCALLER\nLABEL-WROTE\n".to_vec()
    );
}

/// None of the five things [`Inherited`] carries crosses into a
/// `::ROUTINE`, and the neighbouring `CALL`ed label shows each of them
/// crossing -- so this pins the difference rather than the defaults.
#[test]
fn a_routine_inherits_none_of_the_five_a_called_label_inherits() {
    let outcome = routine_program(
        b"numeric digits 7\n\
          numeric form engineering\n\
          address system\n\
          call rtn\n\
          call lbl\n\
          exit\n\
          lbl:\n\
          say digits() form()\n\
          say address()\n\
          say trace()\n\
          return\n\
          ::routine rtn\n\
          say digits() form()\n\
          say address()\n\
          say trace()\n\
          return\n",
    );
    assert_eq!(
        outcome.exit_code,
        0,
        "stderr: {}",
        String::from_utf8_lossy(&outcome.stderr)
    );
    assert_eq!(
        outcome.stdout,
        b"9 SCIENTIFIC\nsh\nN\n7 ENGINEERING\nSYSTEM\nN\n".to_vec(),
        "the routine reports defaults and the label reports the caller's"
    );
}

/// A caller's condition trap does not arm inside a `::ROUTINE`, and the
/// probe separates "not inherited" from "the caller caught it after the
/// routine unwound" -- which every two-level program answers the same way
/// unless the routine has a label of the trap's own name.
#[test]
fn a_routine_does_not_inherit_the_callers_condition_traps() {
    let outcome = routine_program(
        b"signal on syntax name mytrap\n\
          call rtn\n\
          say 'unreached'\n\
          exit\n\
          mytrap:\n\
          say 'CALLER TRAP'\n\
          exit 0\n\
          ::routine rtn\n\
          say 'in routine'\n\
          n1 = 1/0\n\
          return\n\
          mytrap:\n\
          say 'ROUTINE TRAP'\n\
          return\n",
    );
    assert_eq!(
        outcome.exit_code,
        0,
        "stderr: {}",
        String::from_utf8_lossy(&outcome.stderr)
    );
    assert_eq!(outcome.stdout, b"in routine\nCALLER TRAP\n".to_vec());
}

/// A `::ROUTINE` call sets no `SIGL` on either side.
#[test]
fn a_routine_call_sets_no_sigl_where_a_called_label_does() {
    let outcome = routine_program(
        b"signal there\n\
          there:\n\
          call rtn\n\
          say sigl\n\
          call lbl\n\
          say sigl\n\
          exit\n\
          lbl:\n\
          return\n\
          ::routine rtn\n\
          say sigl\n\
          return\n",
    );
    assert_eq!(
        outcome.exit_code,
        0,
        "stderr: {}",
        String::from_utf8_lossy(&outcome.stderr)
    );
    assert_eq!(
        outcome.stdout,
        b"SIGL\n1\n5\n".to_vec(),
        "the routine has none, the caller keeps line 1, the label sets 5"
    );
}

/// Two `::ROUTINE` directives of the same name refuse the program before
/// its first clause, with the oracle's own translation error.
#[test]
fn a_duplicate_routine_directive_is_99_903_before_the_first_clause() {
    let outcome = routine_program(
        b"say 'main ran'\n\
          ::routine zork\n\
          return 'A'\n\
          ::routine zork\n\
          return 'B'\n",
    );
    assert_eq!(outcome.exit_code, 157, "256 - 99");
    assert_eq!(outcome.stdout, b"", "stdout is empty: main never ran");
    let stderr = String::from_utf8_lossy(&outcome.stderr);
    assert!(
        stderr.contains("Error 99.903:  Duplicate ::ROUTINE directive instruction.")
            && stderr.contains("*-* ::routine zork"),
        "expected the oracle's own report with the directive echoed, got: {stderr}"
    );
}

/// Two `::RESOURCE` directives of the same name refuse the program before its
/// first clause, and two of different names do not.
#[test]
fn a_duplicate_resource_name_is_refused_and_a_distinct_one_is_not() {
    let refused = routine_program(
        b"say 'main ran'\n\
          ::resource \"d\"\n\
          one\n\
          ::END\n\
          ::resource D\n\
          two\n\
          ::END\n",
    );
    assert_eq!(refused.exit_code, 157, "256 - 99");
    assert_eq!(refused.stdout, b"", "stdout is empty: main never ran");
    let stderr = String::from_utf8_lossy(&refused.stderr);
    assert!(
        stderr.contains("Error 99.942:  Duplicate ::RESOURCE directive instruction.")
            && stderr.contains("*-* ::resource D"),
        "expected the oracle's own report with the second directive echoed, got: {stderr}"
    );

    // The adjacent success, which is what says the refusal is about the name
    // and not about a second `::RESOURCE` at all.
    let accepted = routine_program(
        b"say 'main ran'\n\
          ::resource d\n\
          one\n\
          ::END\n\
          ::resource e\n\
          two\n\
          ::END\n",
    );
    assert_eq!(
        accepted.exit_code,
        0,
        "stderr: {}",
        String::from_utf8_lossy(&accepted.stderr)
    );
    assert_eq!(accepted.stdout, b"main ran\n".to_vec());
}

/// A program carrying a `::ROUTINE` it never calls runs exactly as one
/// with no directive does -- the adjacent success for the refusal above,
/// and the boundary Step 4's rule turns on: presence is not use.
#[test]
fn an_uncalled_routine_directive_changes_nothing() {
    let outcome = routine_program(
        b"say 'main ran'\n\
          ::routine zork\n\
          return 'A'\n",
    );
    assert_eq!(
        outcome.exit_code,
        0,
        "stderr: {}",
        String::from_utf8_lossy(&outcome.stderr)
    );
    assert_eq!(outcome.stdout, b"main ran\n".to_vec());
}

/// A value too large to copy twice runs to completion on every path
/// whose second copy exists only to be traced.
#[test]
fn nothing_is_rendered_for_a_trace_line_that_will_not_print() {
    let mut interp = Interp::new();
    let program = parse_program(b"n1 = 1".to_vec()).expect("test program parses");
    activate(&mut interp, program);
    let value = interp.text(b"whatever");

    assert_eq!(interp.trace_mode().letter, b'N', "the default setting");
    assert!(interp.intermediate_text(value).is_none());
    assert!(interp.result_text(value).is_none());

    // The neighbouring settings, because "always None" would pass the
    // three lines above. `R` traces results and not intermediates, which
    // is the one mode in which the two functions disagree.
    interp.set_trace_mode(crate::trace::mode_from_setting(b"r").expect("R is a valid setting"));
    assert!(interp.intermediate_text(value).is_none());
    assert_eq!(interp.result_text(value).as_deref(), Some(&b"whatever"[..]));

    interp.set_trace_mode(crate::trace::mode_from_setting(b"i").expect("I is a valid setting"));
    assert_eq!(
        interp.intermediate_text(value).as_deref(),
        Some(&b"whatever"[..])
    );
    assert_eq!(interp.result_text(value).as_deref(), Some(&b"whatever"[..]));
}

/// `EXIT` inside a `::ROUTINE` ends the routine and settles `RESULT`,
/// where `EXIT` inside a `CALL`ed label ends the program.
#[test]
fn exit_inside_a_routine_ends_the_routine_where_a_labels_exit_ends_the_program() {
    let outcome = routine_program(
        b"call rtn_value\n\
          say result\n\
          call rtn_bare\n\
          say result\n\
          say rtn_expr()\n\
          call rtn_interpret\n\
          call rtn_falls_off\n\
          say result\n\
          say 'main ran on'\n\
          exit 0\n\
          ::routine rtn_value\n\
          exit 5\n\
          ::routine rtn_bare\n\
          exit\n\
          ::routine rtn_expr\n\
          n1 = inner()\n\
          say 'unreached'\n\
          return 1\n\
          inner:\n\
          exit 9\n\
          ::routine rtn_interpret\n\
          interpret \"exit 4\"\n\
          say 'unreached'\n\
          return\n\
          ::routine rtn_falls_off\n\
          n0 = 0\n",
    );
    assert_eq!(
        outcome.exit_code,
        0,
        "stderr: {}",
        String::from_utf8_lossy(&outcome.stderr)
    );
    assert_eq!(
        outcome.stdout,
        b"5\nRESULT\n9\nRESULT\nmain ran on\n".to_vec(),
        "a bare EXIT and falling off the end both leave RESULT unset"
    );

    let program_exit = routine_program(
        b"call sub\n\
          say 'never'\n\
          exit\n\
          sub:\n\
          exit 5\n",
    );
    assert_eq!(program_exit.exit_code, 5, "a label's EXIT ends the program");
    assert_eq!(program_exit.stdout, b"");
}

/// A `::ROUTINE`'s own clauses echo at indent **0**, however deeply the
/// call site is nested -- the same fact as `TRACE` not crossing into a
/// routine, seen from the other side.
#[test]
fn a_routines_own_clauses_echo_at_indent_zero_however_deep_the_call_site_is() {
    const PATH: &str = "/tmp/rtn-indent.rex";
    let outcome = crate::run_program(
        PATH,
        b"do 1\n\
          do 1\n\
          call rtn\n\
          end\n\
          end\n\
          ::routine rtn\n\
          trace r\n\
          n1 = 1\n\
          return\n"
            .to_vec(),
        crate::Invocation::none(),
    );
    assert_eq!(
        String::from_utf8_lossy(&outcome.stderr),
        format!(
            "       >I> Routine \"RTN\" in package \"{PATH}\".\n\
             \x20    8 *-* n1 = 1\n\
             \x20      >>>   \"1\"\n\
             \x20    9 *-* return\n\
             \x20      <I< Routine \"RTN\" in package \"{PATH}\".\n"
        )
    );
}

/// `>I>`/`<I<` fire for exactly the four `TRACE` letters whose setting
/// traces labels, and for no other, when the routine's own `TRACE` is its
/// first instruction.
#[test]
fn the_invocation_prefixes_fire_for_exactly_the_four_label_tracing_letters() {
    const PATH: &str = "/tmp/rtn-letters.rex";
    let announced = format!(
        "       >I> Routine \"RTN\" in package \"{PATH}\".\n\
         \x20      <I< Routine \"RTN\" in package \"{PATH}\".\n"
    );
    for letter in *b"ailr" {
        let source = format!(
            "call rtn\n::routine rtn\ntrace {}\nreturn\n",
            letter as char
        );
        let outcome = crate::run_program(PATH, source.into_bytes(), crate::Invocation::none());
        let stderr = String::from_utf8_lossy(&outcome.stderr).into_owned();
        assert!(
            stderr.starts_with(&announced[..announced.find('\n').unwrap() + 1]),
            "trace {}: expected the `>I>` line first, got {stderr:?}",
            letter as char
        );
        assert!(
            stderr.ends_with(&announced[announced.find('\n').unwrap() + 1..]),
            "trace {}: expected the `<I<` line last, got {stderr:?}",
            letter as char
        );
    }
    for letter in *b"ncefo" {
        let source = format!(
            "call rtn\n::routine rtn\ntrace {}\nreturn\n",
            letter as char
        );
        let outcome = crate::run_program(PATH, source.into_bytes(), crate::Invocation::none());
        assert_eq!(
            outcome.stderr, b"",
            "trace {}: this setting does not trace labels, so neither line \
             may be announced",
            letter as char
        );
    }
}

/// A `TRACE` that is not a **top-level** clause of the routine announces
/// nothing, however the construct around it nests -- and the two
/// `INTERPRET` rows that break that rule in both directions.
#[test]
fn the_invocation_prefixes_are_not_announced_from_inside_a_nested_construct() {
    const PATH: &str = "/tmp/rtn-nested.rex";
    let both = format!(
        "       >I> Routine \"RTN\" in package \"{PATH}\".\n\
         \x20      <I< Routine \"RTN\" in package \"{PATH}\".\n"
    );
    let cases: &[(&str, &str, &str)] = &[
        ("an IF then-branch", "if 1=1 then trace l", ""),
        ("an IF else-branch", "if 1=0 then nop; else trace l", ""),
        ("a DO body", "do 1; trace l; end", ""),
        ("a controlled DO body", "do i = 1 to 1; trace l; end", ""),
        ("a WHEN body", "select; when 1=1 then trace l; end", ""),
        // The same nesting under a different letter: the setting takes
        // effect (the `return` echoes) and the pair is still not
        // announced, so this row separates "announced nothing" from
        // "the TRACE did nothing".
        (
            "an IF then-branch, TRACE R",
            "if 1=1 then trace r",
            "     5 *-* return\n",
        ),
        // A fragment counts its own clauses from zero, so a `TRACE`
        // that is the fragment's own first clause DOES announce...
        ("an INTERPRET", "interpret \"trace l\"", "BOTH"),
        // ...and everything that breaks one of the fragment rule's three
        // conditions does not.
        (
            "an INTERPRET, second fragment clause",
            "interpret \"nop; trace l\"",
            "",
        ),
        (
            "an INTERPRET that is not the routine's first clause",
            "n0 = 0\ninterpret \"trace l\"",
            "",
        ),
        (
            "an INTERPRET inside an IF",
            "if 1=1 then interpret \"trace l\"",
            "",
        ),
        (
            "an INTERPRET inside an INTERPRET",
            "interpret \"interpret 'trace l'\"",
            "",
        ),
        // The neighbouring announced case, so the eleven silences above
        // are pinned to the nesting and not to something else about
        // these programs.
        ("a flat first clause", "trace l", "BOTH"),
    ];
    for (what, body, want) in cases {
        let source = format!("call rtn\nsay 'after'\n::routine rtn\n{body}\nreturn\n");
        let outcome = crate::run_program(PATH, source.into_bytes(), crate::Invocation::none());
        let want = if *want == "BOTH" { both.as_str() } else { want };
        assert_eq!(
            String::from_utf8_lossy(&outcome.stderr),
            want,
            "{what}: stderr"
        );
        assert_eq!(outcome.stdout, b"after\n".to_vec(), "{what}: stdout");
    }
}

/// The five things other than the letter that decide whether the pair is
/// announced, each with the neighbouring case that is announced.
#[test]
fn the_invocation_prefixes_are_gated_on_more_than_the_trace_letter() {
    const PATH: &str = "/tmp/rtn-gate.rex";
    let entry = format!("       >I> Routine \"RTN\" in package \"{PATH}\".\n");
    let exit = format!("       <I< Routine \"RTN\" in package \"{PATH}\".\n");
    let both = format!("{entry}{exit}");

    let check = |what: &str, source: &str, want: &str| {
        let outcome =
            crate::run_program(PATH, source.as_bytes().to_vec(), crate::Invocation::none());
        assert_eq!(
            String::from_utf8_lossy(&outcome.stderr),
            want,
            "{what}: stderr"
        );
    };

    // The routine's `TRACE` must be its FIRST instruction, and a LABEL
    // spends that permission where `PROCEDURE`'s own permission survives
    // one.
    check(
        "trace l first",
        "call rtn\n::routine rtn\ntrace l\nreturn\n",
        &both,
    );
    check(
        "an assignment in front of it",
        "call rtn\n::routine rtn\nn0 = 0\ntrace l\nreturn\n",
        "",
    );
    check(
        "a label in front of it",
        "call rtn\n::routine rtn\nlbl:\ntrace l\nreturn\n",
        "",
    );

    // A comment is not an instruction, so it does not spend it.
    check(
        "a comment in front of it",
        "call rtn\n::routine rtn\n/* c */\ntrace l\nreturn\n",
        &both,
    );

    // The dynamic form reaches the same `setTrace` route.
    check(
        "trace value 'l'",
        "call rtn\n::routine rtn\ntrace value 'l'\nreturn\n",
        &both,
    );

    // `<I<` re-reads the setting, so a routine that turns tracing off
    // announces its entry and not its exit.
    check(
        "trace l then trace off",
        "call rtn\n::routine rtn\ntrace l\ntrace off\nreturn\n",
        &entry,
    );

    // The caller's setting never crosses, and a main body never
    // announces one however it is traced.
    check(
        "trace l in the caller",
        "trace l\ncall rtn\n::routine rtn\nn0 = 0\nreturn\n",
        "",
    );
    check("trace l in a main body alone", "trace l\nn0 = 0\n", "");

    // Once per activation, so two calls announce two pairs.
    check(
        "two calls",
        "call rtn\ncall rtn\n::routine rtn\ntrace l\nreturn\n",
        &format!("{both}{both}"),
    );
}

/// **A `::ROUTINE EXTERNAL` naming a library installs and runs at the call.**
/// The library and the procedure are resolved at install time, so a name that
/// is not there is the oracle's own condition. Measured, oracle: `say pi4()`
/// is `3.14159265` at rc 0.
#[test]
fn a_library_backed_routine_installs_and_runs_when_it_is_called() {
    // This worktree's own `build/lib`, not the oracle checkout's, reached
    // through this interpreter's environment rather than the process's, which
    // nothing here may write.
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../build/lib")
        .canonicalize()
        .expect("the worktree's build/lib is three directories above this crate");
    let run = |source: &[u8]| {
        let invocation = crate::Invocation::none().with_environment(vec![(
            b"LD_LIBRARY_PATH".to_vec(),
            directory.clone().into_os_string().into_encoded_bytes(),
        )]);
        crate::run_program("/tmp/routine.rex", source.to_vec(), invocation)
    };

    let outcome = run(b"say 'main ran'\n::routine pi4 external \"LIBRARY rxmath RxCalcPi\"\n");
    assert_eq!(
        outcome.exit_code,
        0,
        "the directive installs, stderr {}",
        String::from_utf8_lossy(&outcome.stderr)
    );
    assert_eq!(outcome.stdout, b"main ran\n");

    let outcome = run(b"say pi4()\n::routine pi4 external \"LIBRARY rxmath RxCalcPi\"\n");
    assert_eq!(
        outcome.exit_code,
        0,
        "exit code, stderr {}",
        String::from_utf8_lossy(&outcome.stderr)
    );
    assert_eq!(outcome.stdout, b"3.14159265\n");
}

/// **An extension reading the interpreter instance through its thread context
/// reaches a table rather than a null pointer.** `orxmethod`'s
/// `TestInterpreterVersion` reads `context->InterpreterVersion()` and answers
/// it as a `size_t`; measured, oracle rc 0 printing `328448`. The test loads
/// this worktree's own `build/lib/liborxmethod.so`; the measurement used the
/// oracle checkout's.
#[test]
fn an_extension_reading_the_instance_reaches_its_table() {
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../build/lib")
        .canonicalize()
        .expect("the worktree's build/lib is three directories above this crate");
    let invocation = crate::Invocation::none().with_environment(vec![(
        b"LD_LIBRARY_PATH".to_vec(),
        directory.into_os_string().into_encoded_bytes(),
    )]);
    let outcome = crate::run_program(
        "/tmp/instance.rex",
        b"say 'before'\nsay .k~new~v\n::class k\n\
          ::method v external \"LIBRARY orxmethod TestInterpreterVersion\"\n"
            .to_vec(),
        invocation,
    );
    assert_eq!(
        outcome.exit_code,
        0,
        "stderr {}",
        String::from_utf8_lossy(&outcome.stderr)
    );
    assert_eq!(outcome.stdout, b"before\n328448\n");
}

/// **An extension's own raise, built from an array of substitutions, is the
/// call's condition, and the program's earlier output is kept.** Measured,
/// oracle: `RxCalcSin(30, 3, 'X')` is 88.916 at rc 168, its units check
/// raising through `RaiseException` over `ArrayOfThree`
/// (`extensions/rxmath/rxmath.cpp:177`). The test loads this worktree's own
/// `build/lib/librxmath.so`; the measurements used the oracle checkout's.
#[test]
fn an_extensions_raise_with_substitutions_is_the_calls_condition() {
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../build/lib")
        .canonicalize()
        .expect("the worktree's build/lib is three directories above this crate");
    let invocation = crate::Invocation::none().with_environment(vec![(
        b"LD_LIBRARY_PATH".to_vec(),
        directory.into_os_string().into_encoded_bytes(),
    )]);
    let outcome = crate::run_program(
        "/tmp/unwritten.rex",
        b"say 'before'\nsay RxCalcSin(30, 3, 'X')\nsay 'after'\n::requires 'rxmath' LIBRARY\n"
            .to_vec(),
        invocation,
    );
    assert_eq!(outcome.exit_code, 168);
    assert_eq!(outcome.stdout, b"before\n");
    assert_eq!(
        String::from_utf8_lossy(&outcome.stderr),
        "       *-* Compiled routine \"RXCALCSIN\".\n     2 *-* say RxCalcSin(30, 3, 'X')\n\
         Error 88 running /tmp/unwritten.rex line 2:  Invalid argument.\n\
         Error 88.916:  Argument 3 must be one of D, R, or G; found \"X\".\n"
    );
}

/// **A routine a `::REQUIRES ... LIBRARY` registered runs when it is
/// called**: measured, `::requires 'rxmath' LIBRARY` and `say RxCalcPi()` is
/// `3.14159265` at rc 0.
#[test]
fn a_routine_a_required_library_exports_runs_rather_than_answering_43_1() {
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../build/lib")
        .canonicalize()
        .expect("the worktree's build/lib is three directories above this crate");
    let run = |source: &[u8]| {
        let invocation = crate::Invocation::none().with_environment(vec![(
            b"LD_LIBRARY_PATH".to_vec(),
            directory.clone().into_os_string().into_encoded_bytes(),
        )]);
        crate::run_program("/tmp/routine.rex", source.to_vec(), invocation)
    };

    let outcome = run(b"say RxCalcPi()\n::requires 'rxmath' LIBRARY\n");
    assert_eq!(
        outcome.exit_code,
        0,
        "exit code, stderr {}",
        String::from_utf8_lossy(&outcome.stderr)
    );
    assert_eq!(outcome.stdout, b"3.14159265\n");

    // The adjacent pair: a name the library does not export is still the
    // oracle's own 43.1, so the call above is found in the library's own
    // table and not by every unresolved name in such a program.
    let outcome = run(b"say zorkolo()\n::requires 'rxmath' LIBRARY\n");
    assert_eq!(outcome.exit_code, 213);
    let stderr = String::from_utf8_lossy(&outcome.stderr).into_owned();
    assert!(
        stderr.contains("Error 43.1:") && stderr.contains("ZORKOLO"),
        "{stderr}"
    );
}

/// A directive naming a library nothing loads is the oracle's own 98.903,
/// before the program's first clause and whatever the directive's keyword.
/// ```text
/// ::requires 'zorkolib' LIBRARY                          98.903 rc 158
/// ::method x external "LIBRARY zorkolib z"               98.903 rc 158
/// ::attribute a external "LIBRARY zorkolib z"            98.903 rc 158
/// ::method x attribute external "LIBRARY zorkolib z"     98.903 rc 158
/// ::routine x external "LIBRARY zorkolib z"              98.903 rc 158
/// ```
#[test]
fn a_directive_naming_a_library_that_is_not_there_is_98_903() {
    let cases: &[&[u8]] = &[
        b"say 'main ran'\n::requires zzznolib library\n",
        b"say 'main ran'\n::routine z external \"LIBRARY zzznolib nosuchfn\"\n",
        b"say 'main ran'\n::class foo\n::method m external \"LIBRARY zzznolib nosuchfn\"\n",
        b"say 'main ran'\n::class foo\n::attribute a external \"LIBRARY zzznolib nosuchfn\"\n",
        b"say 'main ran'\n::class foo\n::method m attribute external \"LIBRARY zzznolib f\"\n",
    ];
    for source in cases {
        let outcome = routine_program(source);
        let stderr = String::from_utf8_lossy(&outcome.stderr).into_owned();
        let shown = String::from_utf8_lossy(source).into_owned();
        assert_eq!(
            outcome.exit_code, 158,
            "{shown}: exit code, stderr {stderr}"
        );
        assert_eq!(
            outcome.stdout, b"",
            "{shown}: stdout must be empty -- main must not have run"
        );
        // The `::REQUIRES` row spells the name unquoted and the rest quote
        // it inside the `EXTERNAL` string, which is the case difference the
        // message carries: measured, oracle, `::requires zzznolib library` is
        // `Unable to load library "ZZZNOLIB".`
        let upper = shown.contains("::requires");
        let named = if upper { "ZZZNOLIB" } else { "zzznolib" };
        assert!(
            stderr.contains(&format!(
                "Error 98.903:  Unable to load library \"{named}\"."
            )),
            "{shown}: stderr {stderr}"
        );
    }
}

/// A builtin's result is a value whose rendering `NUMERIC DIGITS` cannot
/// reach, and D15 is still visible on it from the other side.
#[test]
fn a_builtins_result_renders_independently_of_the_digits_in_force() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"numeric digits 3\nnn = length('abcdefghijklmnop')\nnumeric digits 1\n\
              say nn\nsay nn + 0\nsay length('abcdefghij')"
        ),
        b"16\n2E+1\n10\n".to_vec()
    );
}

/// `say length()` produces the oracle's own 40.3 report, byte for byte.
/// ```text
///      1 *-* say length()
/// Error 40 running /.../p09.rex line 1:  Incorrect call to routine.
/// Error 40.3:  Not enough arguments in invocation of LENGTH; minimum expected is 1.
/// ```
#[test]
fn a_builtin_called_with_too_few_arguments_reports_the_oracles_40_3() {
    let path = "/tmp/length-arity.rex";
    let outcome = crate::run_program(path, b"say length()\n".to_vec(), crate::Invocation::none());
    assert_eq!(outcome.exit_code, 216, "256 - 40");
    assert_eq!(
        String::from_utf8_lossy(&outcome.stderr),
        format!(
            "     1 *-* say length()\n\
             Error 40 running {path} line 1:  Incorrect call to routine.\n\
             Error 40.3:  Not enough arguments in invocation of LENGTH; \
             minimum expected is 1.\n"
        )
    );
}
