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

// ---- assignment ----

#[test]
fn assignment_to_a_variable_a_stem_and_a_compound() {
    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"x = 5\nsay x"),
        b"5\n".to_vec(),
        "a simple variable"
    );

    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"a. = 'wd'\nsay a.1"),
        b"wd\n".to_vec(),
        "a bare stem assignment, read through an unset tail"
    );

    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"a.1 = 'one'\nsay a.1\nsay a.2"),
        b"one\nA.2\n".to_vec(),
        "a compound assignment mutates one tail and leaves the rest deriving its name"
    );
}

// ---- SAY ----

#[test]
fn say_of_each_value_kind_and_of_an_omitted_expression() {
    let mut interp = Interp::new();
    assert_eq!(say_output(&mut interp, b"say 'abc'"), b"abc\n".to_vec());
    assert_eq!(say_output(&mut interp, b"say 1 + 2"), b"3\n".to_vec());
    assert_eq!(
        say_output(&mut interp, b"say .nil"),
        b"The NIL object\n".to_vec()
    );
    // No expression at all: a blank line, not nothing.
    assert_eq!(say_output(&mut interp, b"say"), b"\n".to_vec());
}

// ---- DROP ----

#[test]
fn drop_of_a_variable_returns_it_to_unset() {
    // a = 5; drop a; say a -> A (the derived name, not left over from
    // before -- and never `.nil`, which is a value and not an absence).
    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"a = 5\ndrop a\nsay a"),
        b"A\n".to_vec()
    );

    // The `.nil`-versus-dropped distinction `RootSet::clear_frame_slot` exists
    // for: `x = .nil` renders "The NIL object"; a dropped variable
    // derives its own name instead, never that string.
    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"y = .nil\ndrop y\nsay y"),
        b"Y\n".to_vec()
    );
}

#[test]
fn drop_of_a_tail_tombstones_it_without_taking_the_default() {
    // u. = 'd'; u.1 = 'one'; drop u.1; say u.1 -> U.1 (tombstoned, not
    // falling back to the default); say u.2 -> d (an untouched tail
    // still does).
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"u. = 'd'\nu.1 = 'one'\ndrop u.1\nsay u.1\nsay u.2"
        ),
        b"U.1\nd\n".to_vec()
    );
}

#[test]
fn drop_of_a_whole_stem_leaves_it_looking_untouched() {
    // x. = 'd'; x.1 = 'one'; drop x.; say x.1; say x. -> X.1, X. (exactly
    // what a never-touched stem would give).
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"x. = 'd'\nx.1 = 'one'\ndrop x.\nsay x.1\nsay x."
        ),
        b"X.1\nX.\n".to_vec()
    );
}

#[test]
fn drop_of_the_indirect_form() {
    // A simple variable named by another's (upcased) value.
    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"v = 'x'\nx = 1\ndrop (v)\nsay x"),
        b"X\n".to_vec(),
        "the wrapper's value is upcased before it names a variable"
    );

    // A whole stem, named indirectly.
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"v = 'A.'\na. = 'wd'\na.1 = 'one'\ndrop (v)\nsay a.1\nsay a."
        ),
        b"A.1\nA.\n".to_vec()
    );

    // One tail, named indirectly, joined-dots key taken verbatim rather
    // than re-resolved as source -- the discriminating transcript from
    // `drop_variable`'s own doc comment.
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"v = 'A.1.2'\na.1.2 = 'x'\ndrop (v)\nsay a.1.2"
        ),
        b"A.1.2\n".to_vec()
    );
}

/// Fix-round: the indirect form's value is a **subsidiary list**, not a
/// single verbatim name. Every case here uses *set* targets (the trap
/// the review named: an unset target's cleared slot and its own derived
/// name render identically, so a test built on unset targets cannot
/// tell "resolved the whole value as one name" apart from "split,
/// validated and dropped two names" -- only a set value discriminates).
#[test]
fn drop_of_the_indirect_form_is_a_subsidiary_list_of_words() {
    // Two names, blank-separated, both set and both dropped.
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"a = 1\nb = 2\nv = 'a b'\ndrop (v)\nsay a\nsay b"
        ),
        b"A\nB\n".to_vec(),
        "a blank-separated list drops every word, not one name literally spelled 'A B'"
    );

    // A run of blanks between words, and leading/trailing blanks, both
    // collapse -- still exactly two words.
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"a = 1\nb = 2\nv = '  a    b  '\ndrop (v)\nsay a\nsay b"
        ),
        b"A\nB\n".to_vec()
    );

    // A tab (`'09'x`) separates two words exactly like a blank does.
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"a = 1\nb = 2\nv = 'a'||'09'x||'b'\ndrop (v)\nsay a\nsay b"
        ),
        b"A\nB\n".to_vec(),
        "a tab byte separates words the same way a blank does"
    );

    // A mix of shapes in one list: a whole stem and a simple variable.
    let mut interp = Interp::new();
    assert_eq!(
        say_output(
            &mut interp,
            b"a. = 'd'\na.1 = 'one'\nx = 5\nv = 'a. x'\ndrop (v)\nsay a.1\nsay x"
        ),
        b"A.1\nX\n".to_vec()
    );
}

#[test]
fn drop_of_the_indirect_form_validates_every_word() {
    // A digit-led word: 31.2.
    let mut interp = Interp::new();
    let failure = run_source(&mut interp, b"v = '9'\ndrop (v)").unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (31, 2));
    assert_eq!(raised.additional, vec![b"9".to_vec()]);

    // A dot-led word: 31.3.
    let mut interp = Interp::new();
    let failure = run_source(&mut interp, b"v = '.x'\ndrop (v)").unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (31, 3));
    assert_eq!(raised.additional, vec![b".x".to_vec()]);

    // A parenthesised word: 20.928, not a second round of indirection --
    // proves the list is not recursive.
    let mut interp = Interp::new();
    let failure = run_source(&mut interp, b"w = 1\nv = '(w)'\ndrop (v)").unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (20, 928));
    assert_eq!(raised.additional, vec![b"(w)".to_vec()]);

    // **A newline does not separate.** It is not whitespace for this
    // purpose: the whole of `a`, the newline and `b` form ONE word, which
    // then fails the character-set check, and the reported name carries
    // the raw newline. Measured byte for byte against the oracle,
    // substitution included.
    let mut interp = Interp::new();
    let failure = run_source(&mut interp, b"v = 'a'||'0a'x||'b'\ndrop (v)").unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (20, 928));
    assert_eq!(raised.additional, vec![b"a\nb".to_vec()]);
}

#[test]
fn drop_of_the_indirect_form_validates_before_dropping_any_of_it() {
    // a=1; b=2; v='a 9 b'; drop (v) -> Error 31.2, and NEITHER a nor b
    // is dropped, even though `a` sits before the bad word -- measured
    // against the oracle (SIGNAL ON SYNTAX recovery there shows both
    // untouched). The whole list validates before any drop runs.
    let mut interp = Interp::new();
    let failure = run_source(&mut interp, b"a = 1\nb = 2\nv = 'a 9 b'\ndrop (v)").unwrap_err();
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((raised.number, raised.sub), (31, 2));
    assert_eq!(raised.additional, vec![b"9".to_vec()]);

    // The activation is still on the stack (`run_source` does not pop
    // on error), so `a`'s slot is still directly inspectable.
    let a_slot = interp.slot_of(b"A");
    let frame = interp.activation().frame;
    let a_value = interp
        .roots
        .frame_slot(frame, a_slot)
        .expect("a must still be set");
    assert_eq!(
        &*interp.to_text(a_value),
        b"1",
        "a must not have been dropped before the list's third word failed validation"
    );
}

#[test]
fn drop_of_the_indirect_form_on_an_empty_or_blanks_only_value_is_a_no_op() {
    // Measured: `v=''`/`v='   '` both run clean under the oracle -- zero
    // words, nothing to validate or drop.
    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"v = ''\ndrop (v)\nsay 'after'"),
        b"after\n".to_vec()
    );

    let mut interp = Interp::new();
    assert_eq!(
        say_output(&mut interp, b"v = '   '\ndrop (v)\nsay 'after'"),
        b"after\n".to_vec()
    );
}
