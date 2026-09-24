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

use crate::{Invocation, run_program};

/// Runs `source` and hands back `(exit code, stdout, stderr)`.
fn run_source(source: &[u8]) -> (i32, String, String) {
    let outcome = run_program("/t.rex", source.to_vec(), Invocation::none());
    (
        outcome.exit_code,
        String::from_utf8_lossy(&outcome.stdout).into_owned(),
        String::from_utf8_lossy(&outcome.stderr).into_owned(),
    )
}

/// Every operator the oracle sends to its left operand as a message
/// refuses loudly, naming the operator and the operand's shape.
#[test]
fn an_operator_sent_to_an_object_is_loud() {
    // (source, the operator the message must name, the shape it must name)
    let cases: &[(&[u8], &str, &str)] = &[
        // **No `.array` row here**, and that is the property rather than
        // an omission: a class object is an operator *receiver*
        // (`Interp::operator_message_receiver`), so every operator
        // reaches it as a message and none of them reports a gap.
        // `a_class_objects_operators_are_sent_as_messages` is where they
        // are asserted, and the rows below are the control -- a change
        // that widened past class handles would move one of them.
        // oracle 0 -- two distinct tables rendering the same text
        (
            b"say (.methods == .routines)\n::method m\n  return 1\n::routine r\n  return 2\n",
            "==",
            "one of the interpreter's own objects",
        ),
        // An array, which `~superClasses` puts in a program's hands.
        // oracle 97.1 at rc 159, `Object "an Array" does not understand
        // message "+"` -- `.Array` defines no arithmetic or logical
        // method, so the send finds nothing.
        (b"say (.Object~superClasses + 1)\n", "+", "an array"),
        (b"say (.Object~superClasses & 1)\n", "&", "an array"),
        // oracle 0 on both, measured -- `Object`'s own identity
        // comparison, which this crate does not model. **A build
        // converting through the array's string value answers `1`**: the
        // array is `.Object`'s own superclass list, which holds nothing,
        // so the items joined are the empty string. Measured, three
        // descriptors, `say (.Object~superClasses = '')` and the `==`
        // form: `0` on the oracle at rc 0.
        (b"say (.Object~superClasses = '')\n", "=", "an array"),
        (b"say (.Object~superClasses == '')\n", "==", "an array"),
        // An array whose joined items *are* a number, which the rows
        // above cannot reach: `.Object~superClasses` holds nothing, so
        // its string value is empty and parses as no number whatever
        // this crate does with it. Measured, oracle: `a = (1,)` then
        // `say a + 1` and `say (a & 1)` are 97.1 at rc 159, and
        // `say (a = 1)` is `0` at rc 0 -- `Object`'s identity test, not
        // a comparison of `1` against `1`.
        (b"a = (1,)\nsay (a + 1)\n", "+", "an array"),
        (b"a = (1,)\nsay (a = 1)\n", "=", "an array"),
        (b"a = (1,)\nsay (a & 1)\n", "&", "an array"),
        (b"a = (1,)\nsay \\a\n", "\\", "an array"),
        // A package object, which `~package` puts in a program's hands
        // and which reaches the same arm `.environment` does.
        // oracle 97.1 at rc 159
        (
            b"say (.Array~package + 1)\n",
            "+",
            "one of the interpreter's own objects",
        ),
        // oracle 1 -- one package object, measured:
        // `(.Array~package == .String~package)`
        (
            b"say (.Array~package == .String~package)\n",
            "==",
            "one of the interpreter's own objects",
        ),
    ];
    for (source, op, kind) in cases {
        let (code, stdout, stderr) = run_source(source);
        let expected = format!(
            "rexx-exec: the operator `{op}` applied to {kind} is not implemented (Phase 5)\n"
        );
        assert_eq!(
            (code, stdout.as_str(), stderr.as_str()),
            (120, "", expected.as_str()),
            "{:?}",
            String::from_utf8_lossy(source)
        );
    }
}

/// A class object answers the comparison operators by **identity** and
/// refuses every other one the way the oracle does, at both spellings.
#[test]
fn a_class_objects_operators_are_sent_as_messages() {
    // Identity and not a comparison of renderings, which is what the
    // `'The Array class'` rows pin: measured, oracle rc 0.
    for (source, expected) in [
        (&b"say (.array = .array)\n"[..], "1"),
        (b"say (.array == .array)\n", "1"),
        (b"say (.array = 'The Array class')\n", "0"),
        (b"say (.array == 'The Array class')\n", "0"),
        (b"say (.array \\= .array)\n", "0"),
        (b"say (.array \\== .array)\n", "0"),
        (b"say (.array <> .string)\n", "1"),
        (b"say (.array >< .string)\n", "1"),
        // The message spelling of the same six.
        (b"say .Array~'='(.Array)\n", "1"),
        (b"say .Array~'=='(.Array)\n", "1"),
        (b"say .Array~'\\='(.Array)\n", "0"),
        (b"say .Array~'\\=='(.Array)\n", "0"),
        (b"say .Array~'<>'(.String)\n", "1"),
        (b"say .Array~'><'(.String)\n", "1"),
        // Concatenation never asked the gap and still does not, which is
        // the control for the receiver change: measured, oracle rc 0.
        (b"say (.array || 'x')\n", "The Array classx"),
        (b"say (.array 'x')\n", "The Array class x"),
    ] {
        let (code, stdout, stderr) = run_source(source);
        assert_eq!(
            (code, stdout.as_str(), stderr.as_str()),
            (0, format!("{expected}\n").as_str(), ""),
            "{:?}",
            String::from_utf8_lossy(source)
        );
    }

    // Every other operator, at rc 159 rather than this crate's own
    // refusal. Measured, oracle: `Object "The Array class" does not
    // understand message`.
    for (source, message) in [
        (&b"say (.array > .array)\n"[..], ">"),
        (b"say (.array < .string)\n", "<"),
        (b"say (.array + 1)\n", "+"),
        (b"say (.array ** 1)\n", "**"),
        (b"say (.array & 1)\n", "&"),
        (b"say -.array\n", "-"),
        (b"say \\.array\n", "\\"),
        (b"say .Array~'+'(1)\n", "+"),
    ] {
        let (code, stdout, stderr) = run_source(source);
        assert_eq!(
            (code, stdout.as_str()),
            (159, ""),
            "{:?}",
            String::from_utf8_lossy(source)
        );
        assert!(
            stderr.contains(&format!(
                "Error 97.1:  Object \"The Array class\" does not understand message \
                 \"{message}\"."
            )),
            "{:?} reported {stderr:?}",
            String::from_utf8_lossy(source)
        );
    }
}

/// **R12 at the one numeric surface that is not an operator**: a
/// controlled `DO` header's `initial`, `TO` and `BY` values.
#[test]
fn an_object_in_a_do_headers_numeric_position_is_loud() {
    let cases: &[(&[u8], &str, &str)] = &[
        (b"do i = .array to 5\nend\n", "initial", "a class object"),
        (b"do i = 1 to .array\nend\n", "TO", "a class object"),
        (b"do i = 1 to 5 by .array\nend\n", "BY", "a class object"),
        (
            b"do i = .environment to 5\nend\n",
            "initial",
            "an instance of a user class",
        ),
        (
            b"do i = .Object~superClasses to 5\nend\n",
            "initial",
            "an array",
        ),
    ];
    for (source, role, kind) in cases {
        let (code, stdout, stderr) = run_source(source);
        let expected = format!(
            "rexx-exec: {kind} as a DO header's {role} value is not implemented (Phase 5)\n"
        );
        assert_eq!(
            (code, stdout.as_str(), stderr.as_str()),
            (120, "", expected.as_str()),
            "{:?}",
            String::from_utf8_lossy(source)
        );
    }
}

/// The header positions that read the value's **text** rather than
/// converting it, which both implementations answer alike.
#[test]
fn a_header_position_that_reads_text_keeps_the_oracles_own_diagnostic() {
    for (source, major) in [
        (&b"do i = 1 to 5 for .array\nend\n"[..], "26.3"),
        (b"do .array\nend\n", "26.2"),
        (b"numeric digits .array\n", "26.5"),
    ] {
        let (code, _stdout, stderr) = run_source(source);
        assert_eq!(code, 230, "{:?}", String::from_utf8_lossy(source));
        assert!(
            stderr.contains(&format!("Error {major}:")),
            "{:?} reported {stderr:?}",
            String::from_utf8_lossy(source)
        );
    }
}

/// **`DO OVER` hands its target to `requestArray`**, which is neither
/// `stringValue()` nor an operator, so it falls outside every boundary the
/// two tests above draw.
#[test]
fn an_object_as_a_do_over_target_is_loud() {
    let cases: &[(&[u8], &str)] = &[
        // Measured, oracle rc 0 and `done`: a package's own local
        // directory, which starts empty.
        (
            b"do e over .context~package~local\nsay e\nend\nsay 'done'\n",
            "one of the interpreter's own objects",
        ),
        (
            b"signal on syntax\nsay 1 + 'a'\nsyntax:\ndo e over condition('O')\nend\n",
            "one of the interpreter's own objects",
        ),
    ];
    for (source, kind) in cases {
        let (code, stdout, stderr) = run_source(source);
        let expected = format!(
            "rexx-exec: {kind} as a DO header's OVER target is not implemented (Phase 5)\n"
        );
        assert_eq!(
            (code, stdout.as_str(), stderr.as_str()),
            (120, "", expected.as_str()),
            "{:?}",
            String::from_utf8_lossy(source)
        );
    }
    // **A class object is no longer one of them, and that is the point of
    // the pair.** It reaches `requestArray`, answers no `MAKEARRAY`, and
    // raises the oracle's own `Error_Execution_noarray` naming itself --
    // measured, oracle rc 158, `Unable to convert object "The Array
    // class" to a single-dimensional array value.` So what is left
    // refusing above is a `Directory` built on `NativeObject`'s map, which
    // has no store for `MAKEARRAY` to read.
    let (code, stdout, stderr) = run_source(b"do e over .array\nsay e\nend\n");
    assert_eq!((code, stdout.as_str()), (158, ""));
    assert!(
        stderr.contains(
            "Error 98.913:  Unable to convert object \"The Array class\" to a \
             single-dimensional array value."
        ),
        "a class object must raise the oracle's own noarray, got {stderr:?}"
    );

    // The adjacent successes: `.environment` has a store and iterates
    // through `MAKEARRAY` (measured, oracle `69` at rc 0), and a
    // `StringTable` built on the map iterates by its own walk, whose count
    // is order-independent for the reason `Interp::hash_collection_indexes`
    // gives.
    assert_eq!(
        run_source(b"n = 0\ndo e over .environment\n  n = n + 1\nend\nsay n\n"),
        (0, "69\n".to_string(), String::new())
    );
    assert_eq!(
        run_source(b"n = 0\ndo e over .methods\n  n = n + 1\nend\nsay n\n::method a\n::method b\n"),
        (0, "2\n".to_string(), String::new())
    );
}

/// **A stem redirects to its default, and every check has to follow.**
#[test]
fn an_object_reached_through_a_stem_default_answers_as_the_object() {
    // Measured, oracle rc 0 and rc 159: the redirect reaches the class
    // itself, so the identity test and the 97.1 are the oracle's.
    for (source, code, stdout, message) in [
        (
            &b"a. = .array\nsay (a. == 'The Array class')\n"[..],
            0,
            "0\n",
            None,
        ),
        (b"a. = .array\nsay a. + 1\n", 159, "", Some("+")),
        (b"a. = .array\nsay a.zz + 1\n", 159, "", Some("+")),
    ] {
        let (actual, out, stderr) = run_source(source);
        assert_eq!(
            (actual, out.as_str()),
            (code, stdout),
            "{:?} reported {stderr:?}",
            String::from_utf8_lossy(source)
        );
        if let Some(message) = message {
            assert!(
                stderr.contains(&format!(
                    "Error 97.1:  Object \"The Array class\" does not understand message \
                     \"{message}\"."
                )),
                "{:?} reported {stderr:?}",
                String::from_utf8_lossy(source)
            );
        }
    }

    // The control, and a gap this crate still has: a `DO` header converts
    // through `Interp::header_number`, which asks
    // `Interp::operator_operand_gap` and never the send, so this refuses
    // where the oracle answers 97.1 at rc 159.
    let (code, stdout, stderr) = run_source(b"a. = .array\ndo i = 1 to a.\nend\n");
    assert_eq!((code, stdout.as_str()), (120, ""), "reported {stderr:?}");
    assert!(
        stderr.contains("a class object"),
        "must name the shape it refused, got {stderr:?}"
    );
}

/// A controlled loop's own increment adds to the control variable, and
/// that variable is the oracle's **left** operand of the implicit `+`.
#[test]
fn an_object_assigned_to_a_control_variable_is_loud_at_the_increment() {
    let (code, stdout, stderr) = run_source(b"do i = 1 to 3\nsay 'iter' i\ni = .array\nend\n");
    assert_eq!(
        (code, stdout.as_str(), stderr.as_str()),
        (
            120,
            "iter 1\n",
            "rexx-exec: a class object as a controlled DO's control variable is not \
             implemented (Phase 5)\n"
        )
    );
}

/// `RAISE ... ADDITIONAL` hands its value to `requestArray` -- **under a
/// `SYNTAX` condition and nowhere else**.
#[test]
fn an_object_as_a_raise_syntax_substitution_is_loud() {
    for source in [
        &b"raise syntax 40.1 additional (.array)\n"[..],
        b"raise syntax 40.1 additional (.environment)\n",
    ] {
        let (code, stdout, stderr) = run_source(source);
        assert_eq!(
            (code, stdout.as_str()),
            (120, ""),
            "{:?} reported {stderr:?}",
            String::from_utf8_lossy(source)
        );
        assert!(
            stderr.contains("a RAISE ADDITIONAL value"),
            "{:?} must name its position, got {stderr:?}",
            String::from_utf8_lossy(source)
        );
    }
}

/// **The `RAISE` over-refusal control, and the reason it is its own
/// test.**
#[test]
fn a_raise_the_oracle_does_not_array_convert_still_answers() {
    let cases: &[(&[u8], i32, &str)] = &[
        // The ARRAY form: the oracle has an array already, so the object
        // is rendered into the substitution like any other value.
        (
            b"raise syntax 40.1 array (.array)\n",
            216,
            "External routine \"The Array class\" failed.",
        ),
        (
            b"raise syntax 40.1 array (.environment)\n",
            216,
            "External routine \"The Environment Directory\" failed.",
        ),
        (
            b"raise syntax 40.1 array (.array, 'b')\n",
            216,
            "External routine \"The Array class\" failed.",
        ),
        // Not a SYNTAX condition, so `requestArray` is never reached.
        (
            b"signal on user zork name got\nraise user zork additional (.array)\ngot:\nsay 'trapped'\n",
            0,
            "",
        ),
        // DESCRIPTION is a different keyword and never array-converted.
        (
            b"raise syntax 40.1 description (.array)\n",
            216,
            "External routine \"&1\" failed.",
        ),
    ];
    for (source, expected_code, expected_in_stderr) in cases {
        let (code, _stdout, stderr) = run_source(source);
        assert_eq!(
            code,
            *expected_code,
            "{:?} reported {stderr:?}",
            String::from_utf8_lossy(source)
        );
        assert!(
            stderr.contains(expected_in_stderr),
            "{:?} must still answer the oracle's own bytes, got {stderr:?}",
            String::from_utf8_lossy(source)
        );
    }
}

/// **The anti-over-refusal control, and it is not optional.** Making
/// *every* operator loud would satisfy the test above and refuse a pile
/// of programs the oracle runs at rc 0 with bytes this crate already
/// matches.
#[test]
fn an_object_the_operator_is_not_sent_to_still_answers() {
    let cases: &[(&[u8], &str)] = &[
        // the concatenation family, object on the left
        (b"say .array || 'x'\n", "The Array classx\n"),
        (b"say .array'x'\n", "The Array classx\n"),
        (b"say .array 'x'\n", "The Array class x\n"),
        (b"say .environment || 'x'\n", "The Environment Directoryx\n"),
        // an object on the right of an operator sent to a string
        (
            b"say ('a StringTable' == .methods)\n::method m\n  return 1\n",
            "1\n",
        ),
        (b"say (1 == .methods)\n::method m\n  return 1\n", "0\n"),
        // and the plain rendering both `.NAME` routes owe
        (b"say .LOCAL\n", "The Local Directory\n"),
        (b"say value('.LOCAL')\n", "The Local Directory\n"),
        (b"x = .array; say x\n", "The Array class\n"),
        // `DO OVER` on a string still iterates once yielding itself,
        // which is `Interp::over_snapshot`' own rule for a target that is
        // not an array.
        (b"do e over 'abc'\nsay e\nend\n", "abc\n"),
        // A stem whose default is an ordinary value is untouched by the
        // redirect the check now follows.
        (b"a. = .array\nsay a.\n", "The Array class\n"),
        (b"a. = .array\nsay a. || 'x'\n", "The Array classx\n"),
        (b"z. = 5\ndo i = 1 to z.\nsay i\nend\n", "1\n2\n3\n4\n5\n"),
    ];
    for (source, expected) in cases {
        let (code, stdout, stderr) = run_source(source);
        assert_eq!(
            (code, stdout.as_str(), stderr.as_str()),
            (0, *expected, ""),
            "{:?}",
            String::from_utf8_lossy(source)
        );
    }
}

/// A condition is not an operator, and the oracle's answer for one is
/// this crate's already.
#[test]
fn a_condition_on_an_object_keeps_the_oracles_own_diagnostic() {
    for (source, major) in [
        (&b"if .array then say 'y'\n"[..], "34.1"),
        (b"if .array, 1 then say 'y'\n", "34.6"),
        (b"do while .array\nend\n", "34.3"),
    ] {
        let (code, _stdout, stderr) = run_source(source);
        assert_eq!(code, 222, "{:?}", String::from_utf8_lossy(source));
        assert!(
            stderr.contains(&format!("Error {major}:")),
            "{:?} reported {stderr:?}",
            String::from_utf8_lossy(source)
        );
    }
}

/// A stem whose default value answers no operator at all -- `.nil` --
/// never reaches a method to run, so its own forwarded failure carries
/// no traceback frame, unlike `b.` (no default) and a `String`-defaulted
/// stem, both of which do.
#[test]
fn a_nil_defaulted_stem_carries_no_operator_frame() {
    let (code, stdout, stderr) = run_source(b"s. = .nil\nsay s. + 1\n");
    assert_eq!(code, 159, "{stderr:?}");
    assert!(
        stderr.contains(r#"Error 97.1:  Object "The NIL object" does not understand message "+"."#),
        "{stderr:?}"
    );
    assert_eq!(stdout, "");
    assert!(
        !stderr.contains("Compiled method"),
        "a `.nil`-defaulted stem's own arithmetic failure must carry no \
         operator-forwarded frame -- no method ever ran to raise from -- \
         but got {stderr:?}"
    );
}

/// An operator on an instance is the message send the oracle makes, and
/// a name that spells a number never converts the receiver.
#[test]
fn an_operator_on_an_instance_is_the_send_the_oracle_makes() {
    let prologue = "o = .K~new\no~objectName = '1'\n";
    let epilogue = "::CLASS K\n";
    for (expression, spelling) in [
        ("o + 1", "+"),
        ("o - 1", "-"),
        ("o * 2", "*"),
        ("o / 2", "/"),
        ("o // 2", "//"),
        ("o % 2", "%"),
        ("o ** 2", "**"),
        ("(o > 0)", ">"),
        ("(o < 2)", "<"),
        ("(o >= 0)", ">="),
        ("(o <= 2)", "<="),
        ("(o >> '0')", ">>"),
        ("(o << '2')", "<<"),
        ("(o \\> 0)", "\\>"),
        ("(o \\< 2)", "\\<"),
        ("(o & 1)", "&"),
        ("(o | 0)", "|"),
        ("(o && 1)", "&&"),
        ("(\\o)", "\\"),
        ("(-o)", "-"),
        ("(+o)", "+"),
    ] {
        let source = format!("{prologue}say {expression}\n{epilogue}");
        let (code, stdout, stderr) = run_source(source.as_bytes());
        assert_eq!(code, 159, "{expression}: {stderr:?}");
        assert_eq!(stdout, "", "{expression}");
        assert!(
            stderr.contains(&format!(
                "Object \"1\" does not understand message \"{spelling}\"."
            )),
            "{expression} reported {stderr:?}"
        );
    }
    for (expression, expected) in [
        ("(o = 1)", "0\n"),
        ("(o == '1')", "0\n"),
        ("(o \\= 1)", "1\n"),
        ("(o \\== '1')", "1\n"),
        ("(o <> 1)", "1\n"),
        ("(o >< 1)", "1\n"),
        ("(o || 'q')", "1q\n"),
        ("(o 'q')", "1 q\n"),
        ("(1 = o)", "1\n"),
        ("('q' || o)", "q1\n"),
    ] {
        let source = format!("{prologue}say {expression}\n{epilogue}");
        let (code, stdout, stderr) = run_source(source.as_bytes());
        assert_eq!(code, 0, "{expression}: {stderr:?}");
        assert_eq!(stdout, expected, "{expression}");
    }
    let source = format!("{prologue}if o then say 'yes'\nelse say 'no'\n{epilogue}");
    let (code, stdout, stderr) = run_source(source.as_bytes());
    assert_eq!(code, 0, "{stderr:?}");
    assert_eq!(stdout, "yes\n");
}

/// A value [`Interp::operator_operand_gap`] names parses as no number,
/// which is what makes [`Interp::compare_values`]'s skip past that gap
/// sound rather than merely cheap -- and the same for
/// [`Interp::operator_message_receiver`], which
/// [`Interp::arith_left_operand`] asks only where [`Interp::to_number`]
/// has already refused. An operand that produced a `Number` and was also
/// a send target would have its operator applied to the wrong side.
#[test]
fn a_value_the_operator_gap_names_parses_as_no_number() {
    let mut interp = crate::Interp::new();
    let one = interp.text(b"1");
    let array = interp.alloc_with(
        rexx_core::BehaviourId::ARRAY,
        rexx_core::Body::array(vec![Some(one)]),
    );
    let class = interp
        .classes()
        .lookup("Object")
        .expect("the Object class is registered");
    let behaviour = interp.classes().instance_behaviour_handle(class);
    let named = interp.alloc_with(
        rexx_core::BehaviourId::OBJECT,
        rexx_core::Body::Instance {
            class,
            behaviour,
            name: Some(b"123".to_vec().into_boxed_slice()),
            pools: rexx_core::ScopePools::new(),
            own: None,
            native: None,
        },
    );
    // A stem is in the gap's set only through its default, so it needs
    // one that is itself in the set.
    let aliased = interp.alloc_with(
        rexx_core::BehaviourId::STEM,
        rexx_core::Body::Stem {
            name: b"A.".to_vec().into(),
            default: Some(array),
            tails: rexx_core::NameMap::default(),
        },
    );
    for value in [array, named, aliased] {
        let rendering = interp.to_text(value).into_owned();
        assert!(
            rexx_num::Number::parse_bytes(&rendering).is_some(),
            "{value:?} renders as {rendering:?}, which is not a number"
        );
        assert!(interp.operator_operand_gap(value).is_some(), "{value:?}");
        assert_eq!(
            interp.to_number(value),
            Err(rexx_core::NotNumeric),
            "{value:?}"
        );
    }
    // The send targets among them, which is the half `arith_left_operand`
    // rides. `array` is in the gap's set and is not one, so this is not
    // the same assertion in different words.
    assert!(interp.operator_message_receiver(named).is_some());
    assert!(interp.operator_message_receiver(array).is_none());
}
