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
use crate::version::{
    BIT_WIDTH, BUILD_DATE, LANGUAGE_LEVEL, MAJOR_VERSION, MODIFICATION, RELEASE, VERSION_NUMBER,
};

/// The measured oracle answers for one template, as `(template, pieces)`.
/// Every row was taken from the oracle on 2026-08-05 (this task's report
/// carries the transcripts); the driver below replays only the *movement*,
/// so a row here is a claim about [`Cursor`] alone.
fn pieces(source: &str, template: &[Trigger]) -> Vec<String> {
    let string = source.as_bytes();
    let mut cursor = Cursor::new(string.len());
    let mut out = Vec::new();
    for step in template {
        match step {
            Trigger::End => cursor.move_to_end(),
            Trigger::Plus(n) => cursor.forward(*n),
            Trigger::Minus(n) => cursor.backward(*n),
            Trigger::Absolute(n) => cursor.absolute(*n),
            Trigger::PlusLength(n) => cursor.forward_length(*n),
            Trigger::MinusLength(n) => cursor.backward_length(*n),
            Trigger::Search(needle) => cursor.search(string, needle.as_bytes()),
            Trigger::Caseless(needle) => cursor.caseless_search(string, needle.as_bytes()),
            Trigger::Targets(count) => {
                for index in 0..*count {
                    let piece = if index + 1 == *count {
                        cursor.remainder()
                    } else {
                        cursor.next_word(string)
                    };
                    out.push(String::from_utf8_lossy(&string[piece]).into_owned());
                }
            }
        }
    }
    out
}

/// A template step, for [`pieces`]. `Targets(n)` is the assignment loop
/// belonging to the trigger before it, which is where a template's
/// targets attach (this module's own doc).
enum Trigger {
    End,
    Plus(usize),
    Minus(usize),
    Absolute(usize),
    PlusLength(usize),
    MinusLength(usize),
    Search(&'static str),
    Caseless(&'static str),
    Targets(usize),
}
use Trigger::*;

const D: &str = "abcdefghij";

/// `-n` and `<n` are unrelated operations. The pair this crate was warned
/// would be conflated, and the reason the two match positions exist.
#[test]
fn minus_and_minus_length_differ_on_the_same_movement() {
    assert_eq!(
        pieces(
            D,
            &[
                Absolute(5),
                Targets(1),
                Minus(2),
                Targets(1),
                End,
                Targets(1)
            ]
        ),
        ["abcd", "efghij", "cdefghij"]
    );
    assert_eq!(
        pieces(
            D,
            &[
                Absolute(5),
                Targets(1),
                MinusLength(2),
                Targets(1),
                End,
                Targets(1)
            ]
        ),
        ["abcd", "cd", "cdefghij"]
    );
}

/// `=n` and `+n`/`-n` share one rule: forward of the current position
/// gives `[current, new)`, and anything else -- including *equal* -- gives
/// `[current, END]`.
#[test]
fn absolute_and_relative_share_the_backward_rule() {
    assert_eq!(
        pieces(D, &[Absolute(5), Targets(1), End, Targets(1)]),
        ["abcd", "efghij"]
    );
    // 1 is not greater than 1, so the second target is the remainder and
    // not the null string.
    assert_eq!(
        pieces(D, &[Absolute(1), Targets(1), End, Targets(1)]),
        ["abcdefghij", "abcdefghij"]
    );
    assert_eq!(
        pieces(D, &[Absolute(11), Targets(1), End, Targets(1)]),
        ["abcdefghij", ""]
    );
    assert_eq!(
        pieces(
            D,
            &[
                Absolute(5),
                Targets(1),
                Absolute(5),
                Targets(1),
                End,
                Targets(1)
            ]
        ),
        ["abcd", "efghij", "efghij"]
    );
    assert_eq!(
        pieces(
            D,
            &[
                Absolute(5),
                Targets(1),
                Minus(99),
                Targets(1),
                End,
                Targets(1)
            ]
        ),
        ["abcd", "efghij", "abcdefghij"]
    );
    assert_eq!(
        pieces(D, &[Plus(0), Targets(1), End, Targets(1)]),
        ["abcdefghij", "abcdefghij"]
    );
}

/// `>n`/`<n` do **not** share it: they are exact slices, clamped at the
/// ends, and an offset of zero gives the null string where `+0` gives the
/// whole remainder.
#[test]
fn length_triggers_are_exact_slices() {
    assert_eq!(
        pieces(D, &[PlusLength(0), Targets(1), End, Targets(1)]),
        ["", "abcdefghij"]
    );
    assert_eq!(
        pieces(D, &[MinusLength(0), Targets(1), End, Targets(1)]),
        ["", "abcdefghij"]
    );
    assert_eq!(
        pieces(
            D,
            &[
                PlusLength(3),
                Targets(1),
                MinusLength(2),
                Targets(1),
                End,
                Targets(1)
            ]
        ),
        ["abc", "bc", "bcdefghij"]
    );
    assert_eq!(
        pieces(D, &[PlusLength(20), Targets(1), End, Targets(1)]),
        ["abcdefghij", ""]
    );
    assert_eq!(
        pieces(D, &[MinusLength(20), Targets(1), End, Targets(1)]),
        ["", "abcdefghij"]
    );
}

/// An absent pattern matches at END, the empty pattern behaves as absent,
/// and a pattern at position 1 gives the null string. Searches are
/// non-overlapping and the next one starts after the previous match.
#[test]
fn string_patterns_match_at_end_when_absent() {
    assert_eq!(
        pieces(D, &[Search("z"), Targets(1), End, Targets(1)]),
        ["abcdefghij", ""]
    );
    assert_eq!(
        pieces(D, &[Search(""), Targets(1), End, Targets(1)]),
        ["abcdefghij", ""]
    );
    assert_eq!(
        pieces(D, &[Search("a"), Targets(1), End, Targets(1)]),
        ["", "bcdefghij"]
    );
    assert_eq!(
        pieces(
            "aXbXc",
            &[
                Search("X"),
                Targets(1),
                Search("X"),
                Targets(1),
                End,
                Targets(1)
            ]
        ),
        ["a", "b", "c"]
    );
    assert_eq!(
        pieces(
            "aXXb",
            &[
                Search("XX"),
                Targets(1),
                Search("X"),
                Targets(1),
                End,
                Targets(1)
            ]
        ),
        ["a", "b", ""]
    );
}

/// After a string pattern the next *target* starts past the match, but a
/// following *relative* trigger measures from the match's START -- the
/// subtle one, and the reason `pattern_start` and `pattern_end` are two
/// fields.
#[test]
fn a_relative_trigger_after_a_pattern_measures_from_the_match_start() {
    let plain = pieces(D, &[Search("c"), Targets(1), End, Targets(1)]);
    assert_eq!(plain, ["ab", "defghij"]);
    // `+1` moves to exactly where the match ended, so the End trigger's
    // own section is the same one it would have been with no trigger at
    // all.
    assert_eq!(
        pieces(D, &[Search("c"), Targets(1), Plus(1), End, Targets(1)]),
        plain
    );
    assert_eq!(
        pieces(D, &[Search("c"), Targets(1), Minus(1), End, Targets(1)]),
        ["ab", "bcdefghij"]
    );
    assert_eq!(
        pieces(
            D,
            &[Search("c"), Targets(1), MinusLength(1), End, Targets(1)]
        ),
        ["ab", "bcdefghij"]
    );
}

/// `CASELESS` folds ASCII letters and nothing else. The three
/// non-matching rows are byte pairs exactly `0x20` apart that are not
/// letters, plus a high byte in each direction.
#[test]
fn caseless_folds_ascii_letters_only() {
    assert_eq!(
        pieces("aZb", &[Caseless("z"), Targets(1), End, Targets(1)]),
        ["a", "b"]
    );
    assert_eq!(
        pieces(
            "a\u{7b}b",
            &[Caseless("\u{5b}"), Targets(1), End, Targets(1)]
        ),
        ["a{b", ""]
    );
    assert_eq!(
        pieces(
            "a\u{5f}b",
            &[Caseless("\u{3f}"), Targets(1), End, Targets(1)]
        ),
        ["a_b", ""]
    );
    // A byte alphabet, so the two high bytes are compared as bytes and
    // not as text: 0xc9 and 0xe9 are 0x20 apart and match only
    // themselves.
    let string = [b'a', 0xc9, b'b'];
    let mut high = Cursor::new(string.len());
    high.caseless_search(&string, &[0xe9]);
    let piece = high.remainder();
    assert_eq!(string[piece], [b'a', 0xc9, b'b']);
    let mut same = Cursor::new(string.len());
    same.caseless_search(&string, &[0xc9]);
    let piece = same.remainder();
    assert_eq!(string[piece], [b'a']);
}

/// Word carving: only the final target keeps its leading blanks, extra
/// targets get the null string, and a tab is whitespace.
#[test]
fn word_carving_keeps_leading_blanks_only_on_the_last_target() {
    assert_eq!(pieces("a  b  c", &[End, Targets(3)]), ["a", "b", " c"]);
    assert_eq!(pieces("a  b  c", &[End, Targets(2)]), ["a", " b  c"]);
    assert_eq!(pieces("  a b  ", &[End, Targets(2)]), ["a", "b  "]);
    assert_eq!(pieces("a b", &[End, Targets(4)]), ["a", "b", "", ""]);
    assert_eq!(pieces("a\tb", &[End, Targets(2)]), ["a", "b"]);
}

/// Every `TriggerKind` reaches the [`Cursor`] operation that belongs to
/// it, and every source reaches its own string -- run through
/// `run_program`, so the `step` arm and [`Interp::apply_trigger`]'s own
/// dispatch are inside what is being tested.
#[test]
fn every_trigger_kind_and_source_reaches_its_own_operation() {
    let d = "d = 'abcdefghij'\n";
    let show3 = "say '['||p||']['||q||']['||r||']'\n";
    let show2 = "say '['||p||']['||q||']'\n";
    let rows: &[(&str, &str)] = &[
        // `+n` and `>n` agree here; `-n` and `<n` do not agree with
        // either or with each other.
        ("parse value d with p 5 q +2 r\n", "[abcd][ef][ghij]\n"),
        (
            "parse value d with p 5 q -2 r\n",
            "[abcd][efghij][cdefghij]\n",
        ),
        ("parse value d with p 5 q <2 r\n", "[abcd][cd][cdefghij]\n"),
        ("parse value d with p 5 q >2 r\n", "[abcd][ef][ghij]\n"),
    ];
    for (template, expected) in rows {
        let source = format!("{d}{template}{show3}");
        let outcome = crate::run_program(
            "/tmp/parse-dispatch.rex",
            source.into_bytes(),
            crate::Invocation::none(),
        );
        assert_eq!(
            String::from_utf8_lossy(&outcome.stdout),
            *expected,
            "{template}"
        );
    }

    let pairs: &[(&str, &str)] = &[
        // The offset that separates `+n` from `>n`: the no-movement rule
        // applies to one and not the other.
        ("parse value d with p +0 q\n", "[abcdefghij][abcdefghij]\n"),
        ("parse value d with p >0 q\n", "[][abcdefghij]\n"),
        // A bare numeric symbol is the same absolute column `=n` is.
        ("parse value d with p 2 q\n", "[a][bcdefghij]\n"),
        ("parse value d with p =2 q\n", "[a][bcdefghij]\n"),
        // `String` against `Mixed`: the same needle, and only the
        // caseless one matches.
        ("parse value 'aXb' with p 'x' q\n", "[aXb][]\n"),
        ("parse caseless value 'aXb' with p 'x' q\n", "[a][b]\n"),
        // `End`, which has no operand at all.
        ("parse value 'w1 w2 w3' with p q\n", "[w1][w2 w3]\n"),
        // The sources: each has to reach its own string rather than
        // another source's.
        ("vv = 'v1 v2'; parse var vv p q\n", "[v1][v2]\n"),
        ("parse value 'e1 e2' with p q\n", "[e1][e2]\n"),
    ];
    for (template, expected) in pairs {
        let source = format!("{d}{template}{show2}");
        let outcome = crate::run_program(
            "/tmp/parse-dispatch.rex",
            source.into_bytes(),
            crate::Invocation::none(),
        );
        assert_eq!(
            String::from_utf8_lossy(&outcome.stdout),
            *expected,
            "{template}"
        );
    }
}

/// A `PARSE ARG` template past the first reads an argument that every
/// earlier template's allocations have had a chance to collect.
#[test]
fn a_late_parse_arg_template_survives_collection() {
    // **The arguments are built by concatenation rather than written as
    // literals**, and that is what gives this test its teeth: a literal is
    // interned where the collector cannot take it, so a template reading
    // one would answer correctly however badly it was rooted.
    let source = concat!(
        "call sub 'first' 'argument here', 'second' 'argument here'\n",
        "exit\n",
        "sub:\n",
        "parse arg a1 a2 a3, b1 b2 b3\n",
        "say '['a1']['a2']['a3']['b1']['b2']['b3']'\n",
        "return\n",
    );
    let expected = "[first][argument][here][second][argument][here]\n";
    let stressed = crate::run_program_collect_every_alloc(
        "/tmp/parse-arg-collect.rex",
        source.as_bytes().to_vec(),
        crate::Invocation::none(),
    );
    assert_eq!(String::from_utf8_lossy(&stressed.stdout), expected);
    assert_eq!(stressed.exit_code, 0);
    assert!(
        stressed.collections > 0,
        "the stress mode collected nothing, so this proves nothing"
    );
    let ordinary = crate::run_program(
        "/tmp/parse-arg-collect.rex",
        source.as_bytes().to_vec(),
        crate::Invocation::none(),
    );
    assert_eq!(String::from_utf8_lossy(&ordinary.stdout), expected);
}

/// A template walk whose own allocations collect, over a parse string the
/// walk reads where it lives rather than out of a copy.
///
/// **Every piece below is longer than `INLINE_TEXT`**, so each assignment
/// allocates and, under the stress mode, collects -- which is what puts a
/// collection between one target and the next. **Every parse string is
/// built by concatenation** rather than written as a literal, for the
/// reason [`a_late_parse_arg_template_survives_collection`] gives: a
/// literal is interned where the collector cannot take it.
///
/// **The last template writes over the compound it is reading**, and the
/// object was built inside `fill` so no live register still holds it:
/// from the second target on, the walk's own temporary is what keeps it.
/// Measured with every root on the path deleted -- `Interp::source_text`'s,
/// the `PARSE VAR` arm's and `required_string_value`'s -- the stress run
/// sweeps the source between two of this template's targets and
/// `SourceText::bytes` panics here. Any one of them left in place keeps it
/// alive, so this asserts that the source is rooted and not that a
/// particular line roots it.
#[test]
fn a_template_walk_survives_a_collection_between_its_targets() {
    let source = concat!(
        "vr = 'alphabetic' 'bookkeeper' 'cannonball' 'dreadnought'\n",
        "parse var vr w1 w2 w3 w4\n",
        "say '['w1']['w2']['w3']['w4']'\n",
        "pat = '=' || '='\n",
        "st = 'leftmostpiece' || pat || 'rightmostpiece'\n",
        "parse var st a1 (pat) a2\n",
        "say '['a1']['a2']'\n",
        "parse value 'valuepiece' 'secondpiece' 'thirdpiece' with v1 . v3\n",
        "say '['v1']['v3']'\n",
        "parse value 'columnarpiece' || 'tailingpiece' with c1 14 c2\n",
        "say '['c1']['c2']'\n",
        "call fill\n",
        "parse var sm.1 s1 sm.1 s3 s4\n",
        "say '['s1']['sm.1']['s3']['s4']'\n",
        "exit\n",
        "fill:\n",
        "sm.1 = 'firstliteral' 'secondliteral' 'thirdliteral' 'fourthliteral'\n",
        "return\n",
    );
    let expected = concat!(
        "[alphabetic][bookkeeper][cannonball][dreadnought]\n",
        "[leftmostpiece][rightmostpiece]\n",
        "[valuepiece][thirdpiece]\n",
        "[columnarpiece][tailingpiece]\n",
        "[firstliteral][secondliteral][thirdliteral][fourthliteral]\n",
    );
    let stressed = crate::run_program_collect_every_alloc(
        "/tmp/parse-in-place-collect.rex",
        source.as_bytes().to_vec(),
        crate::Invocation::none(),
    );
    assert_eq!(String::from_utf8_lossy(&stressed.stdout), expected);
    assert_eq!(stressed.exit_code, 0);
    assert!(
        stressed.collections > 0,
        "the stress mode collected nothing, so this proves nothing"
    );
    let ordinary = crate::run_program(
        "/tmp/parse-in-place-collect.rex",
        source.as_bytes().to_vec(),
        crate::Invocation::none(),
    );
    assert_eq!(String::from_utf8_lossy(&ordinary.stdout), expected);
    assert_eq!(ordinary.exit_code, 0);
}

/// `PARSE SOURCE` and `PARSE VERSION` reach their own strings, which no
/// other source's answer can be mistaken for.
#[test]
fn source_and_version_carry_their_own_strings() {
    let outcome = crate::run_program(
        "/tmp/parse-source.rex",
        b"parse source s\nsay s\nparse version v\nsay v\n".to_vec(),
        crate::Invocation::none(),
    );
    let mut expected = b"LINUX COMMAND /tmp/parse-source.rex\n".to_vec();
    expected.extend_from_slice(VERSION);
    expected.push(b'\n');
    assert_eq!(outcome.stdout, expected);
}

/// The `RexxInfo` fields reassemble into [`VERSION`] exactly as
/// `runtime/Version.cpp:73` assembles it.
#[test]
fn the_version_fields_reassemble_into_the_constant() {
    let mut rebuilt = b"REXX-ooRexx_".to_vec();
    rebuilt.extend_from_slice(MAJOR_VERSION);
    rebuilt.push(b'.');
    rebuilt.extend_from_slice(RELEASE);
    rebuilt.push(b'.');
    rebuilt.extend_from_slice(MODIFICATION);
    rebuilt.extend_from_slice(b"(MT)_");
    rebuilt.extend_from_slice(BIT_WIDTH);
    rebuilt.extend_from_slice(b"-bit ");
    rebuilt.extend_from_slice(LANGUAGE_LEVEL);
    rebuilt.push(b' ');
    rebuilt.extend_from_slice(BUILD_DATE);
    assert_eq!(rebuilt, VERSION);
    let mut number = MAJOR_VERSION.to_vec();
    number.push(b'.');
    number.extend_from_slice(RELEASE);
    number.push(b'.');
    number.extend_from_slice(MODIFICATION);
    assert_eq!(number, VERSION_NUMBER);
    assert_eq!(
        BIT_WIDTH,
        (size_of::<*const ()>() * 8).to_string().as_bytes(),
        "`__REXX64__` in Version.cpp:73 and `sizeof(void *) * 8` in \
         RexxInfo::getArchitecture are the same fact, so a VERSION whose \
         width disagrees with this build would make `RexxInfo~architecture` \
         and `RexxInfo~name` contradict each other"
    );
}

/// A fractional, non-numeric or negative positional operand is 26.4, and
/// the conversion is bounded by the **active** `NUMERIC DIGITS` rather
/// than by a fixed width -- measured, `numeric digits 2` makes `+(100)`
/// fail where the default 9 digits accept it.
#[test]
fn a_positional_operand_must_be_a_whole_number_within_the_active_digits() {
    let refused: &[&str] = &[
        "parse value 'abc' with p 2.5 q\n",
        "parse value 'abc' with p +('x') q\n",
        "parse value 'abc' with p +(-1) q\n",
        "numeric digits 2\nparse value 'abc' with p +(100) q\n",
    ];
    for source in refused {
        let outcome = crate::run_program(
            "/tmp/parse-26-4.rex",
            source.as_bytes().to_vec(),
            crate::Invocation::none(),
        );
        assert_eq!(outcome.exit_code, 230, "{source}");
        assert!(
            String::from_utf8_lossy(&outcome.stderr).contains(
                "Error 26.4:  Positional pattern of PARSE template must be a whole number"
            ),
            "{source}: {}",
            String::from_utf8_lossy(&outcome.stderr)
        );
    }
    let accepted = crate::run_program(
        "/tmp/parse-26-4.rex",
        b"parse value 'abcdefghij' with p +(100) q\nsay '['||p||']'\n".to_vec(),
        crate::Invocation::none(),
    );
    assert_eq!(accepted.exit_code, 0);
    assert_eq!(
        String::from_utf8_lossy(&accepted.stdout),
        "[abcdefghij]\n",
        "an operand past the string's own end is a clamp, not an error"
    );
}
