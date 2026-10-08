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

/// An array and an object the interpreter builds natively are operator
/// receivers: the operator is sent, `Object`'s six answer by identity and
/// every other operator is the send's own 97.1. Each row measured, oracle
/// three descriptors identical.
#[test]
fn an_operator_sent_to_an_array_or_a_native_object_is_a_message() {
    // (source, exit code, stdout, the 97.1 line stderr must carry)
    let cases: &[(&[u8], i32, &str, &str)] = &[
        // Two distinct tables rendering the same text.
        (
            b"say (.methods == .routines)\n::method m\n  return 1\n::routine r\n  return 2\n",
            0,
            "0\n",
            "",
        ),
        (
            b"say (.Object~superClasses + 1)\n",
            159,
            "",
            "Object \"an Array\" does not understand message \"+\".",
        ),
        (
            b"say (.Object~superClasses & 1)\n",
            159,
            "",
            "Object \"an Array\" does not understand message \"&\".",
        ),
        // **A build converting through the array's string value answers
        // `1`**: `.Object`'s superclass list holds nothing, so its items
        // join to the empty string.
        (b"say (.Object~superClasses = '')\n", 0, "0\n", ""),
        (b"say (.Object~superClasses == '')\n", 0, "0\n", ""),
        // An array whose joined items *are* a number: identity, not a
        // comparison of `1` against `1`.
        (
            b"a = (1,)\nsay (a + 1)\n",
            159,
            "",
            "Object \"an Array\" does not understand message \"+\".",
        ),
        (b"a = (1,)\nsay (a = 1)\n", 0, "0\n", ""),
        (
            b"a = (1,)\nsay (a & 1)\n",
            159,
            "",
            "Object \"an Array\" does not understand message \"&\".",
        ),
        (
            b"a = (1,)\nsay \\a\n",
            159,
            "",
            "Object \"an Array\" does not understand message \"\\\".",
        ),
        (
            b"say (.Array~package + 1)\n",
            159,
            "",
            "Object \"The REXX Package\" does not understand message \"+\".",
        ),
        // One package object.
        (b"say (.Array~package == .String~package)\n", 0, "1\n", ""),
    ];
    for (source, code, stdout, raised) in cases {
        let (got_code, got_stdout, got_stderr) = run_source(source);
        let shown = String::from_utf8_lossy(source);
        assert_eq!(
            (got_code, got_stdout.as_str()),
            (*code, *stdout),
            "{shown:?}"
        );
        assert!(got_stderr.contains(raised), "{shown:?}: {got_stderr:?}");
        assert_eq!(got_stderr.is_empty(), raised.is_empty(), "{shown:?}");
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
/// controlled `DO` header's `initial`, `TO` and `BY` values are sent `+`
/// (`ControlledLoop::setup`), so an object without one is 97.1. Measured,
/// oracle rc 159 for each.
#[test]
fn an_object_in_a_do_headers_numeric_position_is_sent_plus() {
    for (source, object) in [
        (&b"do i = .array to 5\nend\n"[..], "The Array class"),
        (b"do i = 1 to .array\nend\n", "The Array class"),
        (b"do i = 1 to 5 by .array\nend\n", "The Array class"),
        (b"do i = .Object~superClasses to 5\nend\n", "an Array"),
    ] {
        let (code, stdout, stderr) = run_source(source);
        assert_eq!(
            (code, stdout.as_str()),
            (159, ""),
            "{:?} reported {stderr:?}",
            String::from_utf8_lossy(source)
        );
        assert!(
            stderr.contains(&format!(
                "Error 97.1:  Object \"{object}\" does not understand message \"+\"."
            )),
            "{:?} reported {stderr:?}",
            String::from_utf8_lossy(source)
        );
    }
    // `.environment`'s `+` answers `.nil` through `UNKNOWN`, and the loop
    // goes on by message: the first test sends `>` to it, 97.1 at rc 159.
    let (code, stdout, stderr) = run_source(b"do i = .environment to 5\nend\n");
    assert_eq!((code, stdout.as_str()), (159, ""), "reported {stderr:?}");
    assert!(
        stderr
            .contains("Error 97.1:  Object \"The NIL object\" does not understand message \">\"."),
        "reported {stderr:?}"
    );
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
fn an_object_as_a_do_over_target_is_array_requested() {
    // Measured, oracle rc 0 and `done`: a package's own local directory,
    // which starts empty.
    assert_eq!(
        run_source(b"do e over .context~package~local\nsay e\nend\nsay 'done'\n"),
        (0, "done\n".to_string(), String::new())
    );
    // A condition object is a store-backed `Directory` and converts --
    // measured, oracle rc 0 with nothing written.
    assert_eq!(
        run_source(b"signal on syntax\nsay 1 + 'a'\nsyntax:\ndo e over condition('O')\nend\n"),
        (0, String::new(), String::new())
    );
    // A class object reaches `requestArray`, answers no `MAKEARRAY`, and
    // raises the oracle's own `Error_Execution_noarray` naming itself --
    // measured, oracle rc 158, `Unable to convert object "The Array
    // class" to a single-dimensional array value.`
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

    // A `DO` header sends its `+` through the same redirect: measured,
    // oracle rc 159.
    let (code, stdout, stderr) = run_source(b"a. = .array\ndo i = 1 to a.\nend\n");
    assert_eq!((code, stdout.as_str()), (159, ""), "reported {stderr:?}");
    assert!(
        stderr
            .contains("Error 97.1:  Object \"The Array class\" does not understand message \"+\"."),
        "reported {stderr:?}"
    );
}

/// A controlled loop's own increment adds to the control variable, and
/// that variable is the oracle's **left** operand of the implicit `+`:
/// measured, oracle rc 159 after one pass.
#[test]
fn an_object_assigned_to_a_control_variable_is_sent_plus_at_the_increment() {
    let (code, stdout, stderr) = run_source(b"do i = 1 to 3\nsay 'iter' i\ni = .array\nend\n");
    assert_eq!(
        (code, stdout.as_str()),
        (159, "iter 1\n"),
        "reported {stderr:?}"
    );
    assert!(
        stderr
            .contains("Error 97.1:  Object \"The Array class\" does not understand message \"+\"."),
        "reported {stderr:?}"
    );
}

/// `RAISE ... ADDITIONAL` hands its value to `requestArray` -- **under a
/// `SYNTAX` condition and nowhere else**. Measured, oracle: a class object
/// has no `MAKEARRAY` and is 98.939 at rc 158, and `.environment` converts
/// to an array whose first item fills `&1` at rc 216.
#[test]
fn an_object_as_a_raise_syntax_substitution_is_array_requested() {
    for (source, expected_code, expected_in_stderr) in [
        (
            &b"raise syntax 40.1 additional (.array)\n"[..],
            158,
            "Error 98.939:  Additional information for SYNTAX errors must be a \
             single-dimensional array of values.",
        ),
        (
            b"raise syntax 40.1 additional (.environment)\n",
            216,
            "External routine \"INPUTOUTPUTSTREAM\" failed.",
        ),
    ] {
        let (code, stdout, stderr) = run_source(source);
        assert_eq!(
            (code, stdout.as_str()),
            (expected_code, ""),
            "{:?} reported {stderr:?}",
            String::from_utf8_lossy(source)
        );
        assert!(
            stderr.contains(expected_in_stderr),
            "{:?} reported {stderr:?}",
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
            exposed: None,
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
    // rides: the array and the instance are both sent to.
    assert!(interp.operator_message_receiver(named).is_some());
    assert!(interp.operator_message_receiver(array).is_some());
}
