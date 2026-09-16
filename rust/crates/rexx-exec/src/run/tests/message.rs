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

/// [`Interp::chunk_node_at`]'s steps land on the node the path names, and
/// answer `None` for a step that has nowhere to go.
#[test]
fn a_paths_steps_land_on_the_node_it_names() {
    let program = parse_program(b"zz = -za + zb * f(1)".to_vec()).expect("test program parses");
    let instruction = &program.main.instructions[0];
    let node = |path| Interp::chunk_node_at(instruction, 0, path);
    let variable_at = |path| match node(path).map(|node| &node.kind) {
        Some(ExprKind::Variable(id)) => program.symbols.name(*id).to_string(),
        found => panic!("expected a variable at this address, found {found:?}"),
    };

    let root = node(NodePath::ROOT).expect("slot 0 is the assignment's value");
    assert!(matches!(root.kind, ExprKind::Binary { .. }));

    let left = NodePath::ROOT.child(false).expect("one step fits");
    let right = NodePath::ROOT.child(true).expect("one step fits");
    assert!(matches!(
        node(left).expect("the left operand is there").kind,
        ExprKind::Prefix { .. }
    ));
    assert!(matches!(
        node(right).expect("the right operand is there").kind,
        ExprKind::Binary { .. }
    ));

    assert_eq!(variable_at(left.child(false).expect("two steps fit")), "ZA");
    assert_eq!(
        variable_at(right.child(false).expect("two steps fit")),
        "ZB"
    );
    assert!(matches!(
        node(right.child(true).expect("two steps fit"))
            .expect("the call is the multiplication's right operand")
            .kind,
        ExprKind::Call { .. }
    ));

    // A prefix has one child and it is the `false` step, so the `true` one
    // has nowhere to go; a call has no children at all; and slot 1 is not
    // an address this names.
    assert!(node(left.child(true).expect("two steps fit")).is_none());
    let past_the_call = right
        .child(true)
        .and_then(|path| path.child(false))
        .expect("three steps fit");
    assert!(node(past_the_call).is_none());
    assert!(Interp::chunk_node_at(instruction, 1, NodePath::ROOT).is_none());
}

/// **The two message-send indents, pinned here because nothing else in the
/// tree can pin them**, for the reason
/// [`task_9s_two_new_indents_are_the_oracles_own_and_normalisation_cannot_see_them`]
/// gives: `tests/trace_oracle.rs` and `tests/corpus.rs` both compare through
/// DEVIATION 0's `normalize_stderr`, which collapses exactly the space run
/// these two lines differ in. A unit test's own `assert_eq!` is outside
/// either comparison function.
#[test]
fn a_message_sends_two_indents_are_the_oracles_own_and_normalisation_cannot_see_them() {
    let mut interp = Interp::new();
    run_source(
        &mut interp,
        b"trace i\ndo ii = 1 to 1\n  say 'abc'~length\nend\n",
    )
    .expect("the program runs");
    assert_eq!(
        String::from_utf8(interp.trace.clone()).expect("trace is UTF-8"),
        concat!(
            "     2 *-* do ii = 1 to 1\n",
            "       >L>   \"1\"\n",
            "       >L>   \"1\"\n",
            "       >K>   \"TO\" => \"1\"\n",
            "       >=>   II <= \"1\"\n",
            "     3 *-*   say 'abc'~length\n",
            "       >L>     \"abc\"\n",
            "       >M>     \"LENGTH\" => \"3\"\n",
            "       >>>     \"3\"\n",
            "     4 *-* end\n",
            "     2 *-* do ii = 1 to 1\n",
            "       >V>     II => \"1\"\n",
            "       >>>     \"1\"\n",
            "       >>>     \"2\"\n",
            "       >=>     II <= \"2\"\n",
        )
    );

    let outcome = crate::run_program(
        "/abs/nested.rex",
        b"do ii = 1 to 1\n  do jj = 1 to 1\n    say 'abc'~length(1)\n  end\nend\n".to_vec(),
        crate::Invocation::none(),
    );
    assert_eq!(
        String::from_utf8(outcome.stderr).expect("the report is UTF-8"),
        concat!(
            "       *-* Compiled method \"LENGTH\" with scope \"String\".\n",
            "     3 *-*     say 'abc'~length(1)\n",
            "Error 93 running /abs/nested.rex line 3:  Incorrect call to method.\n",
            "Error 93.902:  Too many arguments in invocation of method; 0 expected.\n",
        )
    );
}

/// The corpus program `rel` reads as a source string, so that a trace
/// expectation and the program it was captured from cannot drift apart:
/// editing the `.rex` file changes what this test runs, and the assertion
/// below then names the line it no longer matches.
macro_rules! corpus_source {
    ($rel:literal) => {
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../corpus/", $rel))
    };
}

/// Runs `source` at `path` and asserts stderr is exactly `expected`.
fn assert_stderr(path: &str, source: &str, expected: &str) {
    let outcome = crate::run_program(path, source.as_bytes().to_vec(), crate::Invocation::none());
    assert_eq!(
        String::from_utf8(outcome.stderr).expect("the trace is UTF-8"),
        expected,
        "the compiled stream"
    );
}

/// **A `::METHOD` activation's own trace indents, pinned here because
/// nothing else in the tree can pin them.**
#[test]
fn a_method_activations_trace_indents_are_the_oracles_own_and_normalisation_cannot_see_them() {
    assert_stderr(
        "/abs/method_trace_invocation.rex",
        corpus_source!("lang/method_trace_invocation.rex"),
        concat!(
            "     2 *-* say .K~m(3)\n",
            "       >E>   .K => \"The K class\"\n",
            "       >L>   \"3\"\n",
            "       >A>   \"3\"\n",
            "       >I> Method \"M\" with scope \"K\" in package \"/abs/method_trace_invocation.rex\".\n",
            "     9 *-* use arg n\n",
            "       >>>   \"3\"\n",
            "       >=>   N <= \"3\"\n",
            "    10 *-* return n + 1\n",
            "       >V>   N => \"3\"\n",
            "       >L>   \"1\"\n",
            "       >O>   \"+\" => \"4\"\n",
            "       >>>   \"4\"\n",
            "       <I< Method \"M\" with scope \"K\" in package \"/abs/method_trace_invocation.rex\".\n",
            "       >M>   \"M\" => \"4\"\n",
            "       >>>   \"4\"\n",
            "     3 *-* say 'after'\n",
            "       >L>   \"after\"\n",
            "       >>>   \"after\"\n",
        ),
    );

    assert_stderr(
        "/abs/method_trace_nested.rex",
        corpus_source!("lang/method_trace_nested.rex"),
        concat!(
            "     2 *-* do i = 1 to 1\n",
            "       >L>   \"1\"\n",
            "       >L>   \"1\"\n",
            "       >K>   \"TO\" => \"1\"\n",
            "       >=>   I <= \"1\"\n",
            "     3 *-*   say .K~outer\n",
            "       >E>     .K => \"The K class\"\n",
            "       >I> Method \"OUTER\" with scope \"K\" in package \"/abs/method_trace_nested.rex\".\n",
            "    10 *-* return .K~inner\n",
            "       >E>   .K => \"The K class\"\n",
            "       >I> Method \"INNER\" with scope \"K\" in package \"/abs/method_trace_nested.rex\".\n",
            "    14 *-* return 'deep'\n",
            "       >L>   \"deep\"\n",
            "       >>>   \"deep\"\n",
            "       <I< Method \"INNER\" with scope \"K\" in package \"/abs/method_trace_nested.rex\".\n",
            "       >M>   \"INNER\" => \"deep\"\n",
            "       >>>   \"deep\"\n",
            "       <I< Method \"OUTER\" with scope \"K\" in package \"/abs/method_trace_nested.rex\".\n",
            "       >M>     \"OUTER\" => \"deep\"\n",
            "       >>>     \"deep\"\n",
            "     4 *-* end\n",
            "     2 *-* do i = 1 to 1\n",
            "       >V>     I => \"1\"\n",
            "       >>>     \"1\"\n",
            "       >>>     \"2\"\n",
            "       >=>     I <= \"2\"\n",
        ),
    );
}

/// **An untrapped condition inside a `::METHOD` body echoes two clauses,
/// innermost first, and neither carries an indent from the other.**
#[test]
fn a_condition_inside_a_method_body_echoes_the_body_and_then_the_send() {
    assert_stderr(
        "/abs/method_body_raises.rex",
        corpus_source!("lang/method_body_raises.rex"),
        concat!(
            "     9 *-* say 1/0\n",
            "     4 *-* say .K~boom\n",
            "Error 42 running /abs/method_body_raises.rex line 9:  \
             Arithmetic overflow/underflow.\n",
            "Error 42.3:  Arithmetic overflow; divisor must not be zero.\n",
        ),
    );
}

/// **The receiver a callee's calling convention carries** (D24), for each
/// route into a callee that this crate has.
#[test]
fn a_trap_handler_and_an_internal_call_do_not_carry_the_same_receiver() {
    let caller = ObjRef::small_int(7).expect("7 fits the tag");
    let label = Entered::Label(0);
    let routine = Entered::Routine(crate::InstalledRoutine {
        program: ProgramId(0),
        directive: 0,
    });

    assert_eq!(
        entered_receiver(label, CallEntry::Written, Some(caller)),
        Some(caller),
        "an internal CALL did not inherit its caller's receiver"
    );
    assert_eq!(
        entered_receiver(label, CallEntry::Trap, Some(caller)),
        None,
        "a CALL ON handler was given the receiver internalCallTrap withholds"
    );
    assert_eq!(
        entered_receiver(routine, CallEntry::Written, Some(caller)),
        None,
        "a ::ROUTINE inherited a receiver"
    );
    assert_eq!(
        entered_receiver(routine, CallEntry::Trap, Some(caller)),
        None
    );

    // A caller with no receiver of its own has none to pass on, whichever
    // route is taken: the top-level control's own state.
    for entered in [label, routine] {
        for entry in [CallEntry::Written, CallEntry::Trap] {
            assert_eq!(entered_receiver(entered, entry, None), None);
        }
    }
}

/// **What a `REPLY` leaves on stderr, in every `cargo test` rather than only
/// under the corpus gate.**
#[test]
fn a_value_returned_after_a_reply_reports_the_oracles_own_98_936() {
    assert_stderr(
        "/abs/method_reply.rex",
        corpus_source!("lang/method_reply.rex"),
        concat!(
            "    15 *-* return 'returned'\n",
            "Error 98 running /abs/method_reply.rex line 15:  Execution error.\n",
            "Error 98.936:  RETURN cannot return a value after a REPLY.\n",
        ),
    );

    // The same raise from a program whose main body chose its own exit status,
    // which is where a raise that settled the status would show.
    assert_stderr(
        "/abs/method_reply_exit_status.rex",
        corpus_source!("lang/method_reply_exit_status.rex"),
        concat!(
            "    13 *-* return 'boom'\n",
            "Error 98 running /abs/method_reply_exit_status.rex line 13:  Execution error.\n",
            "Error 98.936:  RETURN cannot return a value after a REPLY.\n",
        ),
    );
    let outcome = crate::run_program(
        "/abs/method_reply_exit_status.rex",
        corpus_source!("lang/method_reply_exit_status.rex")
            .as_bytes()
            .to_vec(),
        crate::Invocation::none(),
    );
    assert_eq!(outcome.exit_code, 7, "the compiled stream");

    // The adjacent success: a bare `RETURN` after a `REPLY` is legal, and the
    // owed body's own `SAY` still reaches stdout.
    let outcome = crate::run_program(
        "/abs/bare-return-after-reply.rex",
        b"say .K~m\n::class K\n::method m class\n  reply 'replied'\n  say 'tail'\n  return\n"
            .to_vec(),
        crate::Invocation::none(),
    );
    assert_eq!(outcome.stderr, b"", "the compiled stream");
    assert_eq!(outcome.stdout, b"replied\ntail\n", "the compiled stream");
    assert_eq!(outcome.exit_code, 0, "the compiled stream");
}

/// **A second `REPLY` reports the oracle's own 98.935**, on the terms the test
/// above states for 98.936: a `cargo test` instrument for a refusal whose
/// whole difference is on stderr.
#[test]
fn a_second_reply_reports_the_oracles_own_98_935() {
    assert_stderr(
        "/abs/method_reply_twice.rex",
        corpus_source!("lang/method_reply_twice.rex"),
        concat!(
            "    14 *-* reply 'two'\n",
            "Error 98 running /abs/method_reply_twice.rex line 14:  Execution error.\n",
            "Error 98.935:  REPLY can be issued only once per method invocation.\n",
        ),
    );
}

/// **What `GUARD` answers, and the Phase 6 refusals beside it, in every
/// `cargo test`.**
#[test]
fn the_guard_instructions_answers_and_the_phase_6_refusals() {
    struct Row {
        name: &'static str,
        source: &'static str,
        stdout: &'static [u8],
        exit_code: i32,
        stderr_contains: &'static str,
    }
    let rows = [
        Row {
            name: "every spelling in a class method, each a no-op",
            source: corpus_source!("lang/method_guard_instruction.rex"),
            stdout: b"guarded\n",
            exit_code: 0,
            stderr_contains: "",
        },
        Row {
            name: "GUARD outside a method invocation, 99.911",
            source: "say 'before'\nguard on\n",
            stdout: b"before\n",
            exit_code: 157,
            stderr_contains: "GUARD can only be issued in an object method invocation.",
        },
        Row {
            name: "a WHEN whose value is neither 0 nor 1, GUARD's own 34.902",
            source: "say .K~m\n::class K\n::method m class\n  expose v\n  v = 'x'\n  \
                     guard on when v\n  return 'no'\n",
            stdout: b"",
            exit_code: 222,
            stderr_contains: "Value of expression following GUARD keyword must be exactly",
        },
        Row {
            name: "a WHEN that does not hold, where the oracle blocks for ever",
            source: "say .K~m\n::class K\n::method m class\n  expose v\n  v = 0\n  \
                     guard on when v = 1\n  return 'no'\n",
            stdout: b"",
            exit_code: crate::NOT_IMPLEMENTED_EXIT,
            stderr_contains: "wait for another activity",
        },
        Row {
            name: "a REPLY under a construct, whose state an index cannot restore",
            source: "say .K~m\n::class K\n::method m class\n  do 1\n    reply 'in-do'\n  \
                     end\n  return 'no'\n",
            stdout: b"",
            exit_code: crate::NOT_IMPLEMENTED_EXIT,
            stderr_contains: "a REPLY inside a DO, SELECT or IF",
        },
    ];
    for row in rows {
        let outcome = crate::run_program(
            "/abs/guard-surface.rex",
            row.source.as_bytes().to_vec(),
            crate::Invocation::none(),
        );
        let stderr = String::from_utf8_lossy(&outcome.stderr).into_owned();
        assert_eq!(outcome.stdout, row.stdout, "{}", row.name);
        assert_eq!(outcome.exit_code, row.exit_code, "{}", row.name);
        if row.stderr_contains.is_empty() {
            assert_eq!(stderr, "", "{}", row.name);
        } else {
            assert!(
                stderr.contains(row.stderr_contains),
                "{}: stderr was {stderr:?}",
                row.name
            );
        }
    }
}

/// `Message~hasError` answering `1`, which no differential row can carry.
#[test]
fn a_started_method_that_raises_has_an_error_and_reraises_at_result() {
    const PATH: &str = "/tmp/message-has-error.rex";
    let source = b"o = .K~new\n\
                   m = o~start('M')\n\
                   say 'haserror' m~hasError 'completed' m~completed\n\
                   say 'result' m~result\n\
                   say 'never'\n\
                   ::class K\n\
                   ::method M\n\
                   \x20 return 1/0\n";
    let outcome = crate::run_program(PATH, source.to_vec(), crate::Invocation::none());
    assert_eq!(outcome.exit_code, 214, "256 - 42");
    assert_eq!(
        String::from_utf8_lossy(&outcome.stdout),
        "haserror 1 completed 1\n"
    );
    assert_eq!(
        String::from_utf8_lossy(&outcome.stderr),
        format!(
            "\x20    8 *-* return 1/0\n\
             \x20      *-* Compiled method \"RESULT\" with scope \"Message\".\n\
             \x20    4 *-* say 'result' m~result\n\
             Error 42 running {PATH} line 8:  Arithmetic overflow/underflow.\n\
             Error 42.3:  Arithmetic overflow; divisor must not be zero.\n"
        )
    );
}
