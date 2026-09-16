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

/// `SIGNAL` out of a `DO`, landing on a label past it. Unlike `LEAVE`,
/// there is no search and no name to match -- every enclosing construct
/// is abandoned unconditionally, so the loop's own later iterations
/// never happen and neither does the clause right after `END`.
#[test]
fn signal_out_of_a_fragment_does_not_collide_with_the_fragments_own_index_space() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"say 'A'\n\
              interpret \"nop; signal here; say 'WRONG BRANCH RAN'\"\n\
              here: say 'landed correctly'\n",
        ),
        b"A\nlanded correctly\n".to_vec()
    );
}

/// I3 (Task 6 fix round 1): `SIGNAL` out of a `SELECT`, the one Step 1
/// shape the original landing measured for `DO` and `INTERPRET` but
/// never for `SELECT` -- `leave_select` (`run.rs`) is one of the
/// forwarding sites `Flow::Signal`'s own design argument depends on, and
/// it was the only one with no witness. Measured (source with a leading
/// `trace r` clause, lines decremented by one as every other traced test
/// in this file already does): `SELECT` is abandoned exactly like `DO`
/// is, unconditionally, and `SIGL` (C1, this same fix round) reads back
/// the `WHEN`'s own line, all three sub-clauses (`WHEN`, `THEN`,
/// `SIGNAL`) sharing that one source line.
#[test]
fn signal_out_of_a_select_unwinds_it_and_lands_on_its_label() {
    let mut interp = Interp::new();
    run_source_traced(
        &mut interp,
        b"say 'before'\n\
          select\n\
          \x20 when 1 = 1 then signal there\n\
          \x20 otherwise\n\
          \x20   say 'not reached'\n\
          end\n\
          say 'after select, not reached'\n\
          exit\n\
          there:\n\
          say 'reached there sigl:' sigl\n",
    )
    .expect("the program runs");
    assert_eq!(
        interp.trace,
        b"     1 *-* say 'before'\n\
          \x20      >>>   \"before\"\n\
          \x20    2 *-* select\n\
          \x20    3 *-*   when 1 = 1 \n\
          \x20      >>>     \"1\"\n\
          \x20    3 *-*     then\n\
          \x20    3 *-*       signal there\n\
          \x20    9 *-* there:\n\
          \x20   10 *-* say 'reached there sigl:' sigl\n\
          \x20      >>>   \"reached there sigl: 3\"\n"
            .to_vec()
    );
    assert_eq!(interp.out, b"before\nreached there sigl: 3\n".to_vec());
}

/// C1 (Task 6 fix round 1): `SIGL`. The oracle's own `RexxActivation::
/// signalTo`/`internalCall` (`execution/RexxActivation.cpp`, read
/// directly) both call `new_integer(lineNum)` at the point of transfer
/// -- an integer object, which is why `set_sigl` uses `self.text`
/// rather than `self.number`: measured, a `SIGL` value of `22` still
/// renders `22` under `NUMERIC DIGITS 1`, where the identical magnitude
/// as an arithmetic result would round to `2E+1`.
#[test]
fn sigl_is_set_at_every_control_transfer() {
    // Uninitialised until the first transfer, like any other variable;
    // `SIGNAL` sets it to its own line.
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"say 'sigl before:' sigl\nsignal there\nsay 'no'\nthere:\nsay 'sigl after:' sigl\n",
        ),
        b"sigl before: SIGL\nsigl after: 2\n".to_vec()
    );

    // `CALL` sets it too, visible inside the callee (D9r's shared pool)
    // and left set after the callee returns -- an ordinary variable,
    // never restored at the activation boundary.
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"say 'before:' sigl\n\
              call sub\n\
              say 'after call:' sigl\n\
              exit\n\
              \n\
              sub:\n\
              say 'in sub sigl:' sigl\n\
              return\n",
        ),
        b"before: SIGL\nin sub sigl: 2\nafter call: 2\n".to_vec()
    );

    // From inside an `INTERPRET` fragment, `SIGL` reads the *enclosing*
    // `INTERPRET` clause's own line, not any line internal to the
    // fragment -- matching the oracle's own `signalTo`, which delegates
    // a `SIGNAL` fired inside an interpret-created activation to its
    // parent, whose own currently-executing instruction is the
    // `INTERPRET` itself (this crate reproduces the observable answer
    // through `current_clause_line`/`clause_line_override` instead,
    // without adopting that nested-activation architecture).
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"say 'before:' sigl\n\
              interpret \"signal there\"\n\
              say 'no'\n\
              there:\n\
              say 'sigl after signal-in-fragment:' sigl\n",
        ),
        b"before: SIGL\nsigl after signal-in-fragment: 2\n".to_vec()
    );

    // The expression-call form (`f(1)`, Task 4) sets it exactly like
    // `CALL` does.
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"say 'before:' sigl\n\
              say f(1)\n\
              say 'after:' sigl\n\
              exit\n\
              f: return 'called, sigl=' || sigl\n",
        ),
        b"before: SIGL\ncalled, sigl=2\nafter: 2\n".to_vec()
    );

    // `PROCEDURE` isolates it exactly like any other variable: the
    // callee's own `SIGL` starts uninitialised in its own frame, and
    // the caller's `SIGL` (already set by the `CALL` itself, before the
    // callee ever ran) is unaffected by whatever the isolated callee
    // does with its own copy.
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"call sub\n\
              say 'main sigl after sub returns:' sigl\n\
              exit\n\
              \n\
              sub:\n\
              procedure\n\
              say 'sub sigl (isolated):' sigl\n\
              return\n",
        ),
        b"sub sigl (isolated): SIGL\nmain sigl after sub returns: 1\n".to_vec()
    );
}

/// **Fires on the loop's first pass, deliberately** -- a second pass
/// through a `Controlled` (`TO`-style) `DO`/`LOOP` retraces two further
/// `>>>` lines this crate does not reproduce (the documented "KNOWN
/// GAP" at `loop_advance`'s own `Controlled` arm, unrelated to `SIGNAL`),
/// and a witness that reached a second
/// pass would be asserting that gap's own wrong output rather than
/// `SIGNAL`'s. Measured (source with a leading `trace r` clause, every
/// line number then decremented by one to match `run_source_traced`'s
/// own externally-set mode -- `current_value_indent_is_restored_after_a_
/// nested_expression_call`'s own doc comment has the transformation):
/// the `SIGNAL` clause itself traces with no `>>>` line, unlike `CALL`'s
/// dynamic form or `RETURN` with a value, because it produces nothing.
#[test]
fn signal_unwinds_a_nested_do_and_lands_on_its_label() {
    let mut interp = Interp::new();
    run_source_traced(
        &mut interp,
        b"say 'before'\n\
          do i = 1 to 3\n\
          \x20 if i = 1 then signal there\n\
          \x20 say 'i=' i\n\
          end\n\
          say 'after loop, not reached'\n\
          exit\n\
          there:\n\
          say 'reached there'\n",
    )
    .expect("the program runs");
    assert_eq!(
        interp.trace,
        b"     1 *-* say 'before'\n\
          \x20      >>>   \"before\"\n\
          \x20    2 *-* do i = 1 to 3\n\
          \x20      >K>   \"TO\" => \"3\"\n\
          \x20    3 *-*   if i = 1 \n\
          \x20      >>>     \"1\"\n\
          \x20    3 *-*     then\n\
          \x20    3 *-*       signal there\n\
          \x20    8 *-* there:\n\
          \x20    9 *-* say 'reached there'\n\
          \x20      >>>   \"reached there\"\n"
            .to_vec()
    );
    assert_eq!(interp.out, b"before\nreached there\n".to_vec());
}

/// A `SIGNAL` target that matches no label in the running activation's
/// own body is Error 16.1, "Label not found" -- unlike `CALL`'s own
/// unresolved name, which still has a builtin/external fallback to defer
/// to (`Interp::resolve_call`'s own doc), `SIGNAL` has none, so this is
/// the oracle's real answer and not a loud gap.
#[test]
fn signal_to_an_undefined_label_raises_16_1() {
    let mut interp = Interp::new();
    let failure = run_source(
        &mut interp,
        b"say 'before'\nsignal nowhere\nsay 'not reached'\n",
    )
    .unwrap_err();
    assert!(
        matches!(
            &failure,
            Failure::Raised(raised)
                if raised.number == 16
                    && raised.sub == 1
                    && raised.additional == vec![b"NOWHERE".to_vec()]
        ),
        "expected 16.1 naming \"NOWHERE\", got {failure:?}"
    );
    assert_eq!(
        interp.out,
        b"before\n".to_vec(),
        "the clause after SIGNAL must not run"
    );
}

/// A quoted `SIGNAL` target searches the label table -- unlike a quoted
/// `CALL` target, which never does at all (`a_quoted_call_name_never_
/// reaches_the_label_table`, above) -- but case-sensitively against the
/// label's own upcased spelling, so a lowercase quoted spelling still
/// misses and raises 16.1 naming the verbatim quoted text, rather than
/// taking `CALL`'s own loud unresolved-call fallback. A bare symbol is upcased at
/// parse time regardless of its own case, and an uppercase quoted
/// spelling matches for the same reason a lowercase one does not.
#[test]
fn a_quoted_signal_label_searches_case_sensitively_unlike_a_quoted_call() {
    let mut interp = Interp::new();
    let failure = run_source(
        &mut interp,
        b"signal \"sub\"\nsay 'not reached'\nsub:\nsay 'reached'\n",
    )
    .unwrap_err();
    assert!(
        matches!(
            &failure,
            Failure::Raised(raised)
                if raised.number == 16
                    && raised.sub == 1
                    && raised.additional == vec![b"sub".to_vec()]
        ),
        "expected 16.1 naming the verbatim quoted text \"sub\": {failure:?}"
    );

    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"signal Sub\nsay 'not reached'\nsub:\nsay 'reached'\n"
        ),
        b"reached\n".to_vec(),
        "a bare, mixed-case symbol is upcased at parse time and matches"
    );

    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"signal \"SUB\"\nsay 'not reached'\nsub:\nsay 'reached'\n"
        ),
        b"reached\n".to_vec(),
        "the uppercase quoted spelling matches -- it is the case, not the quoting"
    );
}

/// The composition nobody had measured: `SIGNAL` from inside a called
/// routine, targeting a label written back in the *caller's* own text.
#[test]
fn signal_from_a_called_routine_reaches_a_label_in_the_shared_body_and_never_returns() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"call sub\n\
              say 'after call, not reached'\n\
              exit\n\
              \n\
              sub:\n\
              say 'in sub'\n\
              signal caller_label\n\
              say 'sub not reached'\n\
              return\n\
              \n\
              caller_label:\n\
              say 'caller label reached'\n",
        ),
        b"in sub\ncaller label reached\n".to_vec()
    );
}

/// `SIGNAL` out of an `INTERPRET` fragment reaches an enclosing label --
/// unlike `LEAVE`/`ITERATE`, whose own search stops dead at the fragment
/// boundary (`run_fragment`'s own doc comment has that transcript
/// table), `SIGNAL` is forwarded like `Exit`/`Return` because it is a
/// new `Flow` variant with no arm of its own at that boundary
/// (`Flow::Signal`'s own doc comment). Measured (source with a leading
/// `trace r` clause, lines decremented by one to match `run_source_
/// traced`, as above): the fragment's own clause echoes at the enclosing
/// `INTERPRET`'s line, then control resumes at the enclosing label's own
/// line, exactly like `interpret "call sub"` already does for `CALL`.
#[test]
fn signal_escapes_an_interpret_fragment_to_reach_an_enclosing_label() {
    let mut interp = Interp::new();
    run_source_traced(
        &mut interp,
        b"say 'before'\n\
          interpret \"signal there\"\n\
          say 'not reached'\n\
          exit\n\
          there:\n\
          say 'reached there'\n",
    )
    .expect("the program runs");
    assert_eq!(
        interp.trace,
        b"     1 *-* say 'before'\n\
          \x20      >>>   \"before\"\n\
          \x20    2 *-* interpret \"signal there\"\n\
          \x20      >>>   \"signal there\"\n\
          \x20    2 *-* signal there\n\
          \x20    5 *-* there:\n\
          \x20    6 *-* say 'reached there'\n\
          \x20      >>>   \"reached there\"\n"
            .to_vec()
    );
    assert_eq!(interp.out, b"before\nreached there\n".to_vec());
}

/// The failure twin of the test above: a `SIGNAL` inside an `INTERPRET`
/// fragment that matches no label reports **both** clauses, innermost
/// first, each carrying the enclosing `INTERPRET`'s own line -- the same
/// shape `run_fragment`'s own doc comment tables for `LEAVE`/`ITERATE`,
/// now measured for `SIGNAL` too.
#[test]
fn signal_to_an_undefined_label_inside_a_fragment_reports_both_clauses() {
    let mut interp = Interp::new();
    let failure = run_source(
        &mut interp,
        b"say 'before'\ninterpret \"signal nowhere\"\nsay 'not reached'\n",
    )
    .unwrap_err();
    assert!(matches!(
        &failure,
        Failure::Raised(raised) if raised.number == 16 && raised.sub == 1
    ));
    let sealed: Vec<(usize, Vec<u8>)> = interp
        .failure_sites
        .iter()
        .chain(interp.failure_site.iter())
        .map(|s| (s.line().expect("a clause site"), s.text().to_vec()))
        .collect();
    assert_eq!(
        sealed,
        vec![
            (2, b"signal nowhere".to_vec()),
            (2, b"interpret \"signal nowhere\"".to_vec()),
        ]
    );
    assert_eq!(interp.out, b"before\n".to_vec());
}

/// `SIGNAL VALUE`'s own `>K>` line -- `"VALUE" => text`, at the clause's
/// own indent with no extra `+2` the way `WHILE`/`UNTIL` carry, since
/// nothing here is evaluated as part of an *enclosing* instruction's own
/// step the way a loop's condition is. Measured one `DO` deep (lines
/// decremented by one as above): once traced, the rendered value is
/// searched exactly like a bare `SIGNAL label`'s own bytes.
#[test]
fn signal_value_traces_its_own_keyword_line_and_then_searches_like_label() {
    let mut interp = Interp::new();
    run_source_traced(
        &mut interp,
        b"target = 'THERE'\n\
          do i = 1 to 1\n\
          \x20 signal value target\n\
          end\n\
          say 'not reached'\n\
          exit\n\
          there:\n\
          say 'reached, one do deep'\n",
    )
    .expect("the program runs");
    assert_eq!(
        interp.trace,
        b"     1 *-* target = 'THERE'\n\
          \x20      >>>   \"THERE\"\n\
          \x20    2 *-* do i = 1 to 1\n\
          \x20      >K>   \"TO\" => \"1\"\n\
          \x20    3 *-*   signal value target\n\
          \x20      >K>     \"VALUE\" => \"THERE\"\n\
          \x20    7 *-* there:\n\
          \x20    8 *-* say 'reached, one do deep'\n\
          \x20      >>>   \"reached, one do deep\"\n"
            .to_vec()
    );
    assert_eq!(interp.out, b"reached, one do deep\n".to_vec());
}

/// `SIGNAL VALUE`'s target gets **no shape check at all** before the
/// label search: a number, an empty string and an ordinary non-label
/// string all raise 16.1 naming that exact rendered text, none of them a
/// different error -- and the search is case-sensitive exactly like a
/// quoted `SIGNAL label`'s own (`a_quoted_signal_label_searches_case_
/// sensitively_unlike_a_quoted_call`, above), so a lowercase value does
/// not match a label stored upcased even under the identical spelling.
#[test]
fn signal_value_targets_that_match_no_label_all_raise_16_1_naming_the_rendered_text() {
    for (expr, expected) in [
        ("123", "123"),
        ("''", ""),
        ("'no such label'", "no such label"),
    ] {
        let mut interp = Interp::new();
        let source = format!("target = {expr}\nsignal value target\nsay 'not reached'\n");
        let failure = run_source(&mut interp, source.as_bytes()).unwrap_err();
        assert!(
            matches!(
                &failure,
                Failure::Raised(raised)
                    if raised.number == 16
                        && raised.sub == 1
                        && raised.additional == vec![expected.as_bytes().to_vec()]
            ),
            "{expr}: expected 16.1 naming {expected:?}, got {failure:?}"
        );
    }

    let mut interp = Interp::new();
    let failure = run_source(
        &mut interp,
        b"target = 'there'\n\
          signal value target\n\
          say 'not reached'\n\
          exit\n\
          there:\n\
          say 'reached'\n",
    )
    .unwrap_err();
    assert!(
        matches!(
            &failure,
            Failure::Raised(raised)
                if raised.number == 16
                    && raised.sub == 1
                    && raised.additional == vec![b"there".to_vec()]
        ),
        "a lowercase value must not match the upcased label THERE: {failure:?}"
    );
}
