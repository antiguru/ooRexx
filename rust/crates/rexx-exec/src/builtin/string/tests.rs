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

use super::super::dispatch;
use crate::error::Failure;
use crate::{Interp, error::Raised};
use rexx_core::{Decoded, ObjRef};

/// Runs `name` over `arguments`, each `None` standing for an omitted
/// interior position, and answers the result's own bytes.
fn call(name: &[u8], arguments: &[Option<&[u8]>]) -> Result<Vec<u8>, Failure> {
    let mut interp = Interp::new();
    let args: Vec<_> = arguments
        .iter()
        .map(|argument| argument.map(|bytes| interp.text(bytes)))
        .collect();
    let result = dispatch(&mut interp, name, &args).expect("a builtin name")?;
    Ok(interp.to_text(result).into_owned())
}

/// [`call`], answering the handle as well as the bytes, for the one
/// property that is invisible in the bytes.
fn call_handle(name: &[u8], arguments: &[&[u8]]) -> (ObjRef, Vec<u8>) {
    let mut interp = Interp::new();
    let args: Vec<_> = arguments
        .iter()
        .map(|bytes| Some(interp.text(bytes)))
        .collect();
    let result = dispatch(&mut interp, name, &args)
        .expect("a builtin name")
        .expect("this call succeeds");
    let bytes = interp.to_text(result).into_owned();
    (result, bytes)
}

/// `call`, for the cases whose answer is the bytes and nothing else.
fn answer(name: &[u8], arguments: &[&[u8]]) -> Vec<u8> {
    let arguments: Vec<_> = arguments.iter().map(|bytes| Some(*bytes)).collect();
    call(name, &arguments).expect("this call succeeds")
}

/// The `(major, sub)` and substitutions of the condition `name` raises.
fn raised(name: &[u8], arguments: &[Option<&[u8]>]) -> (u16, u16, Vec<Vec<u8>>) {
    let failure = call(name, arguments).expect_err("this call raises");
    let Failure::Raised(raised) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    (raised.number, raised.sub, raised.additional)
}

/// The oracle transcripts each of these lines came from are in the
/// function's own doc comment; this is the same table run through
/// `dispatch`.
#[test]
fn the_padding_builtins_answer_the_oracles_own_bytes() {
    assert_eq!(answer(b"CENTER", &[b"ab", b"6", b"-"]), b"--ab--");
    assert_eq!(answer(b"CENTRE", &[b"ab", b"6", b"-"]), b"--ab--");
    assert_eq!(answer(b"CENTER", &[b"ab", b"6"]), b"  ab  ");
    assert_eq!(answer(b"CENTER", &[b"abcdef", b"3"]), b"bcd");
    assert_eq!(answer(b"CENTER", &[b"abcdef", b"2"]), b"cd");
    // The odd pad and the odd truncation both go right, which an
    // even-width case cannot tell from their going left.
    assert_eq!(answer(b"CENTER", &[b"ab", b"5", b"-"]), b"-ab--");
    assert_eq!(answer(b"CENTRE", &[b"ab", b"5", b"-"]), b"-ab--");
    assert_eq!(answer(b"CENTER", &[b"abc", b"6", b"-"]), b"-abc--");
    assert_eq!(answer(b"CENTER", &[b"abcdef", b"5"]), b"abcde");
    assert_eq!(answer(b"CENTER", &[b"abcde", b"2"]), b"bc");
    assert_eq!(answer(b"CENTER", &[b"The blue sky", b"7"]), b"e blue ");
    assert_eq!(answer(b"CENTER", &[b"abc", b"0"]), b"");
    assert_eq!(answer(b"CENTER", &[b"abc", b"3"]), b"abc");
    assert_eq!(answer(b"CENTER", &[b"", b"4", b"*"]), b"****");

    assert_eq!(answer(b"LEFT", &[b"ab", b"5", b"."]), b"ab...");
    assert_eq!(answer(b"LEFT", &[b"ab", b"5"]), b"ab   ");
    assert_eq!(answer(b"LEFT", &[b"abcdef", b"3"]), b"abc");
    assert_eq!(answer(b"LEFT", &[b"abc", b"0"]), b"");

    assert_eq!(answer(b"RIGHT", &[b"ab", b"5", b"."]), b"...ab");
    assert_eq!(answer(b"RIGHT", &[b"abcdef", b"3"]), b"def");
    assert_eq!(answer(b"RIGHT", &[b"abc", b"0"]), b"");
}

#[test]
fn the_extraction_builtins_answer_the_oracles_own_bytes() {
    assert_eq!(answer(b"SUBSTR", &[b"abcdef", b"2", b"3"]), b"bcd");
    assert_eq!(answer(b"SUBSTR", &[b"abcdef", b"2"]), b"bcdef");
    assert_eq!(answer(b"SUBSTR", &[b"abcdef", b"2", b"8"]), b"bcdef   ");
    assert_eq!(
        answer(b"SUBSTR", &[b"abcdef", b"2", b"8", b"."]),
        b"bcdef..."
    );
    assert_eq!(answer(b"SUBSTR", &[b"abcdef", b"7"]), b"");
    assert_eq!(answer(b"SUBSTR", &[b"abcdef", b"7", b"3", b"."]), b"...");
    assert_eq!(answer(b"SUBSTR", &[b"abcdef", b"2", b"0"]), b"");
    assert_eq!(
        call(b"SUBSTR", &[Some(b"abcdef"), Some(b"3"), None, Some(b".")])
            .expect("an omitted length is legal past the minimum"),
        b"cdef"
    );

    assert_eq!(answer(b"DELSTR", &[b"abcdef", b"3", b"2"]), b"abef");
    assert_eq!(answer(b"DELSTR", &[b"abcdef", b"3"]), b"ab");
    assert_eq!(answer(b"DELSTR", &[b"abcdef", b"1", b"6"]), b"");
    assert_eq!(answer(b"DELSTR", &[b"abcdef", b"9", b"2"]), b"abcdef");
    assert_eq!(answer(b"DELSTR", &[b"abcdef"]), b"");
    assert_eq!(answer(b"DELSTR", &[b"abcdef", b"3", b"0"]), b"abcdef");
    assert_eq!(
        call(b"DELSTR", &[Some(b"abcdef"), None, Some(b"2")]).expect("n defaults to 1"),
        b"cdef"
    );

    assert_eq!(answer(b"REVERSE", &[b"abcdef"]), b"fedcba");
    assert_eq!(answer(b"REVERSE", &[b""]), b"");
    assert_eq!(answer(b"REVERSE", &[b"a"]), b"a");
}

#[test]
fn the_splicing_builtins_answer_the_oracles_own_bytes() {
    assert_eq!(answer(b"INSERT", &[b"-", b"abc", b"1"]), b"a-bc");
    assert_eq!(answer(b"INSERT", &[b"-", b"abc"]), b"-abc");
    assert_eq!(answer(b"INSERT", &[b"-", b"abc", b"0"]), b"-abc");
    assert_eq!(answer(b"INSERT", &[b"-", b"abc", b"5"]), b"abc  -");
    assert_eq!(answer(b"INSERT", &[b"XY", b"abc", b"1", b"4"]), b"aXY  bc");
    assert_eq!(
        answer(b"INSERT", &[b"XY", b"abc", b"1", b"4", b"."]),
        b"aXY..bc"
    );
    assert_eq!(answer(b"INSERT", &[b"XY", b"abc", b"1", b"1"]), b"aXbc");
    assert_eq!(answer(b"INSERT", &[b"XY", b"abc", b"1", b"0"]), b"abc");
    assert_eq!(answer(b"INSERT", &[b"", b"abc", b"2"]), b"abc");
    assert_eq!(
        answer(b"INSERT", &[b"123", b"abc", b"5", b"6", b"+"]),
        b"abc++123+++"
    );
    assert_eq!(answer(b"INSERT", &[b"", b"", b"3", b"0"]), b"   ");
    assert_eq!(answer(b"INSERT", &[b"", b"", b"3", b"1"]), b"    ");

    assert_eq!(answer(b"OVERLAY", &[b"XY", b"abcdef", b"3"]), b"abXYef");
    assert_eq!(answer(b"OVERLAY", &[b"XY", b"abcdef"]), b"XYcdef");
    assert_eq!(
        answer(b"OVERLAY", &[b"XY", b"abcdef", b"3", b"4"]),
        b"abXY  "
    );
    assert_eq!(
        answer(b"OVERLAY", &[b"XY", b"abcdef", b"3", b"4", b"."]),
        b"abXY.."
    );
    assert_eq!(
        answer(b"OVERLAY", &[b"XY", b"abcdef", b"3", b"1"]),
        b"abXdef"
    );
    assert_eq!(answer(b"OVERLAY", &[b"XY", b"abcdef", b"8"]), b"abcdef XY");
    assert_eq!(answer(b"OVERLAY", &[b"XY", b"abc", b"3", b"0"]), b"abc");
    assert_eq!(answer(b"OVERLAY", &[b"", b"abcdef", b"3"]), b"abcdef");
    assert_eq!(answer(b"OVERLAY", &[b"", b"abc", b"6", b"1"]), b"abc   ");
    assert_eq!(answer(b"OVERLAY", &[b"", b"abc", b"3", b"1"]), b"ab ");
    assert_eq!(answer(b"OVERLAY", &[b"XY", b"abc", b"4", b"0"]), b"abc");
    assert_eq!(answer(b"OVERLAY", &[b"XY", b"abc", b"5", b"0"]), b"abc ");
    assert_eq!(answer(b"OVERLAY", &[b"qq", b"abcd", b"4"]), b"abcqq");
}

#[test]
fn the_searching_builtins_answer_the_oracles_own_bytes() {
    assert_eq!(answer(b"POS", &[b"an", b"banana"]), b"2");
    assert_eq!(answer(b"POS", &[b"an", b"banana", b"3"]), b"4");
    assert_eq!(answer(b"POS", &[b"an", b"banana", b"5"]), b"0");
    assert_eq!(answer(b"POS", &[b"an", b"banana", b"1", b"3"]), b"2");
    assert_eq!(answer(b"POS", &[b"an", b"banana", b"1", b"2"]), b"0");
    assert_eq!(answer(b"POS", &[b"", b"banana"]), b"0");
    assert_eq!(answer(b"POS", &[b"a", b""]), b"0");
    assert_eq!(answer(b"POS", &[b"a", b"banana", b"9"]), b"0");
    assert_eq!(answer(b"POS", &[b"a", b"BANANA"]), b"0");
    assert_eq!(answer(b"POS", &[b"e", b"abcdeeeeef", b"2", b"4"]), b"5");
    assert_eq!(answer(b"POS", &[b"eee", b"abcdeeeeef", b"5", b"2"]), b"0");

    // The one position `StringUtil::pos` searches past the window, which
    // `find_forward`'s own doc explains and transcribes the oracle for.
    // **Each of these is paired with the case one step away that answers
    // differently**, because the rule is not positional and a row without
    // its neighbour pins nothing: the first pair differs only in a decoy
    // `a` inside the window, the second in whether the position past the
    // window is reached at all, and the third in the needle's length.
    let zoo: &[u8] = b"banana bandana abracadabra";
    assert_eq!(answer(b"POS", &[b"an", zoo, b"6", b"4"]), b"9");
    assert_eq!(answer(b"POS", &[b"an", zoo, b"6", b"3"]), b"0");
    assert_eq!(answer(b"POS", &[b"an", zoo, b"9", b"1"]), b"0");
    assert_eq!(answer(b"POS", &[b"an", zoo, b"9", b"2"]), b"9");
    assert_eq!(answer(b"POS", &[b"an", b"axan", b"1", b"3"]), b"3");
    assert_eq!(answer(b"POS", &[b"an", b"zxan", b"1", b"3"]), b"0");
    assert_eq!(answer(b"POS", &[b"an", b"axaxan", b"1", b"4"]), b"0");
    assert_eq!(answer(b"POS", &[b"an", b"axaxan", b"1", b"5"]), b"5");
    assert_eq!(answer(b"POS", &[b"abc", b"axxabc", b"1", b"5"]), b"4");
    assert_eq!(answer(b"POS", &[b"abc", b"zxxabc", b"1", b"5"]), b"0");
    // A one-byte needle takes the C++'s early return and never reaches its
    // rescan, so the position past the window is unreachable for it.
    assert_eq!(answer(b"POS", &[b"a", b"zza", b"1", b"2"]), b"0");
    assert_eq!(answer(b"POS", &[b"a", b"zza", b"1", b"3"]), b"3");

    // DEVIATION 3 (`docs/superpowers/plans/phase-4-exclusions.txt`),
    // licensed by Moritz 2026-08-14: the overrun above can land one byte
    // past the haystack, where the oracle reads the `RexxString` NUL
    // terminator and matches a needle ending in `'00'x` -- measured, the
    // oracle's own answer here is 2, not this crate's 0. Pinned so a
    // future change to the overrun cannot silently start inventing that
    // byte; `find_forward`'s own doc has the full account and the reason
    // exact agreement is not available (`changestr` on the same needle
    // dies on the oracle at rc 139).
    assert_eq!(answer(b"POS", &[b"a\0", b"aa"]), b"0");

    assert_eq!(answer(b"LASTPOS", &[b"an", b"banana"]), b"4");
    assert_eq!(answer(b"LASTPOS", &[b"an", b"banana", b"4"]), b"2");
    assert_eq!(answer(b"LASTPOS", &[b"an", b"banana", b"3"]), b"2");
    assert_eq!(answer(b"LASTPOS", &[b"a", b"banana", b"3"]), b"2");
    assert_eq!(answer(b"LASTPOS", &[b"an", b"banana", b"6", b"2"]), b"0");
    assert_eq!(answer(b"LASTPOS", &[b"an", b"banana", b"6", b"3"]), b"4");
    assert_eq!(answer(b"LASTPOS", &[b"", b"banana"]), b"0");
    assert_eq!(answer(b"LASTPOS", &[b"a", b"banana", b"99"]), b"6");
    assert_eq!(answer(b"LASTPOS", &[b"abc", b"xxabc", b"5"]), b"3");
    assert_eq!(answer(b"LASTPOS", &[b"abc", b"xxabc", b"4"]), b"0");
    assert_eq!(answer(b"LASTPOS", &[b"a", b"aaaaabcdef", b"9", b"5"]), b"5");
    assert_eq!(
        answer(b"LASTPOS", &[b"a", b"aaaaabcdef", b"10", b"5"]),
        b"0"
    );
    // The range default is the whole haystack and not what is left of it
    // from the start; a needle before the start position is the only way
    // to see the difference, and the explicit range is the other half.
    assert_eq!(answer(b"LASTPOS", &[b"b", b"banana", b"5"]), b"1");
    assert_eq!(answer(b"LASTPOS", &[b"b", b"banana", b"5", b"2"]), b"0");
    // The pair that rules out `find_forward`'s rescan here: a decoy
    // sharing the needle's first byte, at the same position a POS window
    // this size would have let it reach one range further back.
    // `find_backward`'s own doc has the oracle source citation.
    assert_eq!(
        answer(b"LASTPOS", &[b"345", b"Y3Y345YYYYYY", b"8", b"4"]),
        b"0"
    );
    assert_eq!(
        answer(b"LASTPOS", &[b"345", b"Y3Y345YYYYYY", b"8", b"5"]),
        b"4"
    );

    assert_eq!(answer(b"COUNTSTR", &[b"a", b"banana"]), b"3");
    assert_eq!(answer(b"COUNTSTR", &[b"an", b"banana"]), b"2");
    assert_eq!(answer(b"COUNTSTR", &[b"aa", b"aaaa"]), b"2");
    assert_eq!(answer(b"COUNTSTR", &[b"", b"abc"]), b"0");
    assert_eq!(answer(b"COUNTSTR", &[b"abc", b""]), b"0");

    assert_eq!(answer(b"COMPARE", &[b"abcde", b"abcde"]), b"0");
    assert_eq!(answer(b"COMPARE", &[b"abcde", b"abXde"]), b"3");
    assert_eq!(answer(b"COMPARE", &[b"abc", b"abc "]), b"0");
    assert_eq!(answer(b"COMPARE", &[b"abc", b"abc."]), b"4");
    assert_eq!(answer(b"COMPARE", &[b"abc", b"abc.", b"."]), b"0");
    assert_eq!(answer(b"COMPARE", &[b"", b"a"]), b"1");
    assert_eq!(answer(b"COMPARE", &[b"abc", b"ab"]), b"3");
    assert_eq!(answer(b"COMPARE", &[b"ab", b"abc"]), b"3");
    assert_eq!(answer(b"COMPARE", &[b"abc", b"ab", b"c"]), b"0");

    assert_eq!(answer(b"ABBREV", &[b"Print", b"Pri"]), b"1");
    assert_eq!(answer(b"ABBREV", &[b"Print", b"Pro"]), b"0");
    assert_eq!(answer(b"ABBREV", &[b"Print", b""]), b"1");
    assert_eq!(answer(b"ABBREV", &[b"Print", b"", b"1"]), b"0");
    assert_eq!(answer(b"ABBREV", &[b"Print", b"Pri", b"4"]), b"0");
    assert_eq!(answer(b"ABBREV", &[b"Print", b"Pri", b"3"]), b"1");
    assert_eq!(answer(b"ABBREV", &[b"", b""]), b"1");
    assert_eq!(answer(b"ABBREV", &[b"", b"x"]), b"0");
    assert_eq!(answer(b"ABBREV", &[b"Print", b"PRI"]), b"0");
    assert_eq!(answer(b"ABBREV", &[b"Print", b"Printer"]), b"0");
}

#[test]
fn the_rewriting_builtins_answer_the_oracles_own_bytes() {
    assert_eq!(answer(b"CHANGESTR", &[b"a", b"banana", b"X"]), b"bXnXnX");
    assert_eq!(
        answer(b"CHANGESTR", &[b"a", b"banana", b"X", b"2"]),
        b"bXnXna"
    );
    assert_eq!(
        answer(b"CHANGESTR", &[b"a", b"banana", b"X", b"0"]),
        b"banana"
    );
    assert_eq!(answer(b"CHANGESTR", &[b"a", b"banana", b""]), b"bnn");
    assert_eq!(answer(b"CHANGESTR", &[b"", b"banana", b"X"]), b"banana");
    assert_eq!(
        answer(b"CHANGESTR", &[b"an", b"banana", b"ANA"]),
        b"bANAANAa"
    );
    assert_eq!(answer(b"CHANGESTR", &[b"z", b"banana", b"X"]), b"banana");
    assert_eq!(answer(b"CHANGESTR", &[b"a", b"", b"X"]), b"");

    assert_eq!(answer(b"COPIES", &[b"ab", b"3"]), b"ababab");
    assert_eq!(answer(b"COPIES", &[b"ab", b"0"]), b"");
    assert_eq!(answer(b"COPIES", &[b"", b"5"]), b"");
    assert_eq!(answer(b"COPIES", &[b"ab", b"1"]), b"ab");
    assert_eq!(answer(b"COPIES", &[b"ab", b" -0 "]), b"");

    assert_eq!(answer(b"SPACE", &[b"a   b  c"]), b"a b c");
    assert_eq!(answer(b"SPACE", &[b"a   b  c", b"2"]), b"a  b  c");
    assert_eq!(answer(b"SPACE", &[b"a   b  c", b"0"]), b"abc");
    assert_eq!(answer(b"SPACE", &[b"a   b  c", b"2", b"-"]), b"a--b--c");
    assert_eq!(answer(b"SPACE", &[b"   "]), b"");
    assert_eq!(answer(b"SPACE", &[b""]), b"");
    assert_eq!(answer(b"SPACE", &[b"  a  "]), b"a");
    assert_eq!(answer(b"SPACE", &[b"a\tb"]), b"a b");
    assert_eq!(answer(b"SPACE", &[b"a\nb"]), b"a\nb");
}

#[test]
fn the_option_taking_builtins_answer_the_oracles_own_bytes() {
    assert_eq!(answer(b"STRIP", &[b"  ab  "]), b"ab");
    assert_eq!(answer(b"STRIP", &[b"  ab  ", b"L"]), b"ab  ");
    assert_eq!(answer(b"STRIP", &[b"  ab  ", b"T"]), b"  ab");
    assert_eq!(answer(b"STRIP", &[b"  ab  ", b"B"]), b"ab");
    assert_eq!(answer(b"STRIP", &[b"  ab  ", b"l"]), b"ab  ");
    assert_eq!(answer(b"STRIP", &[b"  ab  ", b"Leading"]), b"ab  ");
    assert_eq!(answer(b"STRIP", &[b"xyabyx", b"B", b"xy"]), b"ab");
    assert_eq!(answer(b"STRIP", &[b"   "]), b"");
    assert_eq!(answer(b"STRIP", &[b"a\tb\t"]), b"a\tb");
    assert_eq!(answer(b"STRIP", &[b"abc", b"B", b""]), b"abc");
    assert_eq!(
        answer(b"STRIP", &[b"+-+-a-+b-+-+", b"L", b"-+"]),
        b"a-+b-+-+"
    );
    assert_eq!(
        call(b"STRIP", &[Some(b"xxabxx"), None, Some(b"x")]).expect("the option defaults to B"),
        b"ab"
    );

    assert_eq!(answer(b"VERIFY", &[b"abcde", b"abc"]), b"4");
    assert_eq!(answer(b"VERIFY", &[b"abcde", b"abcde"]), b"0");
    assert_eq!(answer(b"VERIFY", &[b"abcde", b"abc", b"M"]), b"1");
    assert_eq!(answer(b"VERIFY", &[b"abcde", b"xyz", b"M"]), b"0");
    assert_eq!(answer(b"VERIFY", &[b"abcde", b"abc", b"m"]), b"1");
    assert_eq!(answer(b"VERIFY", &[b"abcde", b"abc", b"Nope"]), b"4");
    assert_eq!(answer(b"VERIFY", &[b"abcde", b""]), b"1");
    assert_eq!(answer(b"VERIFY", &[b"abcde", b"", b"M"]), b"0");
    assert_eq!(answer(b"VERIFY", &[b"", b"abc"]), b"0");
    assert_eq!(answer(b"VERIFY", &[b"abc", b"", b"N", b"2", b"0"]), b"2");
    assert_eq!(
        answer(b"VERIFY", &[b"abcde", b"ab", b"N", b"1", b"2"]),
        b"0"
    );
    assert_eq!(answer(b"VERIFY", &[b"abcde", b"abc", b"N", b"9"]), b"0");
    assert_eq!(
        answer(b"VERIFY", &[b"ABCDEF", b"ABC", b"N", b"2", b"3"]),
        b"4"
    );
    assert_eq!(
        answer(b"VERIFY", &[b"ABCDEF", b"ADEF", b"M", b"2", b"3"]),
        b"4"
    );
}

#[test]
fn the_case_shifting_builtins_answer_the_oracles_own_bytes() {
    assert_eq!(answer(b"LOWER", &[b"ABCdef"]), b"abcdef");
    assert_eq!(answer(b"LOWER", &[b"ABCDEF", b"3"]), b"ABcdef");
    assert_eq!(answer(b"LOWER", &[b"ABCDEF", b"3", b"2"]), b"ABcdEF");
    assert_eq!(answer(b"LOWER", &[b"ABCDEF", b"9"]), b"ABCDEF");
    assert_eq!(answer(b"LOWER", &[b"ABCDEF", b"3", b"0"]), b"ABCDEF");
    assert_eq!(answer(b"LOWER", &[b"ABCDEF", b"3", b"99"]), b"ABcdef");
    assert_eq!(answer(b"UPPER", &[b"abcDEF"]), b"ABCDEF");
    assert_eq!(answer(b"UPPER", &[b"abcdef", b"3", b"2"]), b"abCDef");
    // Only ASCII folds; measured, `upper('e9'x)` is its own argument.
    assert_eq!(answer(b"UPPER", &[&[0xe9]]), &[0xe9]);
    assert_eq!(answer(b"LOWER", &[&[0xc9]]), &[0xc9]);

    assert_eq!(answer(b"TRANSLATE", &[b"abcdef"]), b"ABCDEF");
    assert_eq!(
        answer(b"TRANSLATE", &[b"abcdef", b"123", b"abc"]),
        b"123def"
    );
    assert_eq!(answer(b"TRANSLATE", &[b"abcdef", b"123"]), b"      ");
    assert_eq!(
        answer(b"TRANSLATE", &[b"abcdef", b"12", b"abcd"]),
        b"12  ef"
    );
    assert_eq!(
        answer(b"TRANSLATE", &[b"abcdef", b"1234", b"ab"]),
        b"12cdef"
    );
    assert_eq!(
        answer(b"TRANSLATE", &[b"abcdef", b"12", b"abcd", b"."]),
        b"12..ef"
    );
    assert_eq!(answer(b"TRANSLATE", &[b"abcdef", b"", b""]), b"abcdef");
    assert_eq!(
        answer(b"TRANSLATE", &[b"abcABC", b"123", b"abc"]),
        b"123ABC"
    );
    assert_eq!(answer(b"TRANSLATE", &[b"aXbXc", b"12", b"XX"]), b"a1b1c");
    assert_eq!(answer(b"TRANSLATE", &[b"4123", b"abcd", b"1234"]), b"dabc");
    assert_eq!(
        answer(
            b"TRANSLATE",
            &[b"abcdef", b"123456", b"aaabbbcc", b".", b"2", b"3"]
        ),
        b"a4.def"
    );
    assert_eq!(
        call(
            b"TRANSLATE",
            &[Some(b"abcdef"), None, None, None, Some(b"2"), Some(b"3")]
        )
        .expect("with no tables and no pad this is UPPER"),
        b"aBCDef"
    );
    assert_eq!(
        call(b"TRANSLATE", &[Some(b"abcdef"), None, Some(b"abc")])
            .expect("an omitted tableout is the null string"),
        b"   def"
    );
    assert_eq!(
        call(b"TRANSLATE", &[Some(b"abcdef"), None, None, Some(b".")])
            .expect("a pad alone still translates"),
        b"......"
    );
}

/// The 40.x call-layer family, with the routine name and the *call's* own
/// argument position substituted.
#[test]
fn a_bad_argument_kind_names_the_routine_and_the_call_position() {
    assert_eq!(
        raised(b"LEFT", &[Some(b"ab"), Some(b"q")]),
        (40, 12, vec![b"LEFT".to_vec(), b"2".to_vec(), b"q".to_vec()])
    );
    assert_eq!(
        raised(
            b"SUBSTR",
            &[Some(b"abc"), Some(b"2"), Some(b"3"), Some(b"pq")]
        ),
        (
            40,
            23,
            vec![b"SUBSTR".to_vec(), b"4".to_vec(), b"pq".to_vec()]
        )
    );
    assert_eq!(
        raised(
            b"TRANSLATE",
            &[Some(b"abc"), None, None, Some(b"$"), Some(b"1"), Some(b"q")]
        ),
        (
            40,
            12,
            vec![b"TRANSLATE".to_vec(), b"6".to_vec(), b"q".to_vec()]
        )
    );
    assert_eq!(
        raised(
            b"INSERT",
            &[
                Some(b"-"),
                Some(b"abc"),
                Some(b"1"),
                Some(b"2"),
                Some(b"pq")
            ]
        ),
        (
            40,
            23,
            vec![b"INSERT".to_vec(), b"5".to_vec(), b"pq".to_vec()]
        )
    );
    // A pad is refused whether or not it could be used, and a value
    // needing more than ARGUMENT_DIGITS is not a whole number.
    assert_eq!(
        raised(b"LEFT", &[Some(b""), Some(b"0"), Some(b"xx")]),
        (
            40,
            23,
            vec![b"LEFT".to_vec(), b"3".to_vec(), b"xx".to_vec()]
        )
    );
    assert_eq!(
        raised(b"LEFT", &[Some(b"ab"), Some(b"1E18")]),
        (
            40,
            12,
            vec![b"LEFT".to_vec(), b"2".to_vec(), b"1E18".to_vec()]
        )
    );
    assert_eq!(
        raised(b"COPIES", &[Some(b"ab"), Some(b"")]),
        (40, 12, vec![b"COPIES".to_vec(), b"2".to_vec(), Vec::new()])
    );

    // `CENTRE` is `CENTER`'s implementation under its own name, and the
    // message is where the two are told apart.
    assert_eq!(
        raised(b"CENTRE", &[Some(b"ab"), Some(b"6"), Some(b"--")]),
        (
            40,
            23,
            vec![b"CENTRE".to_vec(), b"3".to_vec(), b"--".to_vec()]
        )
    );
    assert_eq!(
        raised(b"CENTER", &[Some(b"ab"), Some(b"6"), Some(b"--")]),
        (
            40,
            23,
            vec![b"CENTER".to_vec(), b"3".to_vec(), b"--".to_vec()]
        )
    );
}

/// The 93.9xx operation-layer family, which names neither the routine nor
/// the position and reports the *converted* value.
#[test]
fn a_bad_argument_range_reports_the_converted_value() {
    assert_eq!(
        raised(b"LEFT", &[Some(b"ab"), Some(b"-1.0")]),
        (93, 923, vec![b"-1".to_vec()])
    );
    assert_eq!(
        raised(b"SUBSTR", &[Some(b"abc"), Some(b"0.0")]),
        (93, 924, vec![b"0".to_vec()])
    );
    assert_eq!(
        raised(b"SUBSTR", &[Some(b"abc"), Some(b"2"), Some(b"-1")]),
        (93, 923, vec![b"-1".to_vec()])
    );
    assert_eq!(
        raised(b"COPIES", &[Some(b"ab"), Some(b"-1.0")]),
        (93, 906, vec![b"1".to_vec(), b"-1".to_vec()])
    );
    assert_eq!(
        raised(b"INSERT", &[Some(b"a"), Some(b"b"), Some(b"-1")]),
        (93, 906, vec![b"2".to_vec(), b"-1".to_vec()])
    );
    assert_eq!(
        raised(
            b"CHANGESTR",
            &[Some(b"a"), Some(b"b"), Some(b"c"), Some(b"-1")]
        ),
        (93, 906, vec![b"3".to_vec(), b"-1".to_vec()])
    );
    assert_eq!(
        raised(b"STRIP", &[Some(b"ab"), Some(b"Xyz")]),
        (93, 915, vec![b"BLT".to_vec(), b"Xyz".to_vec()])
    );
    assert_eq!(
        raised(b"VERIFY", &[Some(b"a"), Some(b"b"), Some(b"")]),
        (93, 915, vec![b"MN".to_vec(), Vec::new()])
    );
    // A range error inside a builtin whose start is out of range is
    // still raised, which is the ordering `delstr`'s own comment names.
    assert_eq!(
        raised(b"DELSTR", &[Some(b"abc"), Some(b"9"), Some(b"-1")]),
        (93, 923, vec![b"-1".to_vec()])
    );
}

/// Every conversion runs before every range check, at the three shapes
/// that can tell the two layers apart.
#[test]
fn the_call_layer_is_checked_before_the_operation_layer() {
    assert_eq!(
        raised(
            b"SUBSTR",
            &[Some(b"abc"), Some(b"0"), Some(b"5"), Some(b"xx")]
        ),
        (
            40,
            23,
            vec![b"SUBSTR".to_vec(), b"4".to_vec(), b"xx".to_vec()]
        )
    );
    assert_eq!(
        raised(
            b"SUBSTR",
            &[Some(b"abc"), Some(b"0"), Some(b"5"), Some(b"x")]
        ),
        (93, 924, vec![b"0".to_vec()])
    );

    assert_eq!(
        raised(
            b"TRANSLATE",
            &[Some(b"abc"), None, None, Some(b"$"), Some(b"0"), Some(b"q")]
        ),
        (
            40,
            12,
            vec![b"TRANSLATE".to_vec(), b"6".to_vec(), b"q".to_vec()]
        )
    );
    assert_eq!(
        raised(
            b"TRANSLATE",
            &[Some(b"abc"), None, None, Some(b"$"), Some(b"0"), Some(b"1")]
        ),
        (93, 924, vec![b"0".to_vec()])
    );

    assert_eq!(
        raised(b"VERIFY", &[Some(b"a"), Some(b"b"), Some(b"X"), Some(b"q")]),
        (
            40,
            12,
            vec![b"VERIFY".to_vec(), b"4".to_vec(), b"q".to_vec()]
        )
    );
    assert_eq!(
        raised(b"VERIFY", &[Some(b"a"), Some(b"b"), Some(b"X"), Some(b"1")]),
        (93, 915, vec![b"MN".to_vec(), b"X".to_vec()])
    );
}

/// A result too large to allocate is the oracle's own Error 5, not a
/// process abort.
#[test]
fn a_result_too_large_to_allocate_is_the_oracles_own_error_5() {
    let failure =
        call(b"LEFT", &[Some(b"ab"), Some(b"999999999999999999")]).expect_err("too large");
    let Failure::Raised(exhausted) = failure else {
        panic!("expected Raised, got {failure:?}");
    };
    assert_eq!((exhausted.number, exhausted.sub), (5, 0));
    assert_eq!(exhausted.exit_code(), 251);
    assert!(exhausted.additional.is_empty());

    assert_eq!(
        raised(b"LEFT", &[Some(b"ab"), Some(b"1234567890123456789")]),
        (
            40,
            12,
            vec!["LEFT".into(), "2".into(), "1234567890123456789".into()]
        )
    );

    for name in [b"COPIES".as_slice(), b"CENTER", b"SPACE", b"RIGHT"] {
        let subject: &[u8] = if name == b"SPACE" { b"a b" } else { b"ab" };
        let failure =
            call(name, &[Some(subject), Some(b"123456789012345678")]).expect_err("too large");
        let Failure::Raised(exhausted) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!(
            (exhausted.number, exhausted.sub),
            (5, 0),
            "{} did not report the allocation failure",
            String::from_utf8_lossy(name)
        );
    }
}

/// The whole-number conversion runs at `ARGUMENT_DIGITS`, not at the
/// caller's `NUMERIC DIGITS`, and the two disagree in both directions.
#[test]
fn the_whole_number_conversion_uses_the_argument_precision() {
    assert_eq!(
        raised(b"LEFT", &[Some(b"ab"), Some(b"1.0000001")]),
        (
            40,
            12,
            vec![b"LEFT".to_vec(), b"2".to_vec(), b"1.0000001".to_vec()]
        )
    );
    assert_eq!(answer(b"LEFT", &[b"ab", b"1.0000000000000000000004"]), b"a");
    // The generous spellings the oracle accepts, each measured.
    assert_eq!(answer(b"LEFT", &[b"ab", b" 5 "]), b"ab   ");
    assert_eq!(answer(b"LEFT", &[b"ab", b"+5"]), b"ab   ");
    assert_eq!(answer(b"LEFT", &[b"ab", b"1e1"]), b"ab        ");
    assert_eq!(answer(b"SUBSTR", &[b"abcdef", b"2e0"]), b"bcdef");
}

/// A count or offset is created as text, so `NUMERIC DIGITS` cannot
/// reach it.
#[test]
fn a_counting_builtin_answers_text_that_no_digits_setting_reshapes() {
    let mut interp = Interp::new();
    for (name, arguments) in [
        (b"POS".as_slice(), [b"a".as_slice(), b"bbbbbbbbba"]),
        (b"LASTPOS", [b"a", b"bbbbbbbbba"]),
        (b"COMPARE", [b"bbbbbbbbba", b"bbbbbbbbbz"]),
        (b"COUNTSTR", [b"a", b"aaaaaaaaaa"]),
        (b"VERIFY", [b"bbbbbbbbba", b"b"]),
        (b"LENGTH", [b"bbbbbbbbba", b"bbbbbbbbba"]),
    ] {
        let take = if name == b"LENGTH" { 1 } else { 2 };
        let args: Vec<_> = arguments[..take]
            .iter()
            .map(|bytes| Some(interp.text(bytes)))
            .collect();
        let result = dispatch(&mut interp, name, &args)
            .expect("a builtin name")
            .expect("this call succeeds");
        assert_eq!(
            interp.to_text(result).into_owned(),
            b"10",
            "{} did not answer plain text",
            String::from_utf8_lossy(name)
        );
    }
}

/// The condition every 93.9xx here carries is rc 163, not the 40.x
/// family's 216, which is the observable that made the two families
/// worth separating.
#[test]
fn the_two_argument_families_exit_differently() {
    let failure =
        call(b"SUBSTR", &[Some(b"abc"), Some(b"2"), Some(b"-1")]).expect_err("a negative length");
    let Failure::Raised(range) = failure else {
        panic!("expected Raised");
    };
    assert_eq!(range.exit_code(), 163);

    let failure = call(b"SUBSTR", &[Some(b"abc"), Some(b"x")]).expect_err("a non-numeric position");
    let Failure::Raised(kind) = failure else {
        panic!("expected Raised");
    };
    assert_eq!(kind.exit_code(), 216);
}

/// A substitution carries the argument's own bytes, and the report
/// applies the oracle's display rule to them.
#[test]
fn a_substitution_carries_bytes_and_the_report_makes_them_displayable() {
    assert_eq!(
        raised(b"COPIES", &[Some(b"ab"), Some(&[0xff])]),
        (40, 12, vec![b"COPIES".to_vec(), b"2".to_vec(), vec![0xff]])
    );
    assert_eq!(
        raised(b"LEFT", &[Some(b"ab"), Some(b"5"), Some(&[0x00, 0x12])]),
        (
            40,
            23,
            vec![b"LEFT".to_vec(), b"3".to_vec(), vec![0x00, 0x12]]
        )
    );

    // The rule is applied by `Raised::report`, so the check is on the
    // rendered line rather than on the stored substitution.
    let site = crate::error::ClauseSite {
        sites: &[],
        path: "/p.rex",
    };
    let high = Raised::argument_not_whole(b"COPIES", 2, &[0xff]);
    assert!(
        high.report(&site)
            .windows(4)
            .any(|w| w == [b'"', 0xff, b'"', b'.']),
        "a byte at or above 0x80 must reach the report unchanged"
    );
    let control = Raised::argument_not_a_pad(b"LEFT", 3, &[0x00, 0x12]);
    assert!(
        control.report(&site).windows(5).any(|w| w == *b"\"??\"."),
        "a control byte must reach the report as a question mark"
    );
    // Tab, carriage return and line feed are the three that stay, and
    // without them this rule would be "every byte below 0x20".
    let kept = Raised::argument_not_a_pad(b"LEFT", 3, b"\t\r\n");
    assert!(
        kept.report(&site).windows(5).any(|w| w == *b"\"\t\r\n\""),
        "tab, carriage return and line feed are not sanitised"
    );
}

/// A `0x00` first byte is an accepted option letter that is none of the
/// letters, and each caller answers with its own "neither" branch.
#[test]
fn a_null_option_byte_is_accepted_and_matches_no_letter() {
    assert_eq!(answer(b"STRIP", &[b"  ab  ", &[0x00]]), b"  ab  ");
    assert_eq!(answer(b"STRIP", &[b"  ab  ", &[0x00, b'L']]), b"  ab  ");
    assert_eq!(answer(b"STRIP", &[b"  ab  ", &[b'L', 0x00]]), b"ab  ");
    // `VERIFY`'s two branches test different letters, so the option that
    // is neither takes the second arm of each -- and the two arms give
    // *opposite* letters' answers. With a non-empty reference the test is
    // for `N`, so `0x00` answers as `M` does; with an empty one the test
    // is for `M`, so `0x00` answers as `N` does. A single flag for both
    // gets exactly one of these two lines wrong.
    assert_eq!(answer(b"VERIFY", &[b"abcde", b"abc", &[0x00]]), b"1");
    assert_eq!(answer(b"VERIFY", &[b"abcde", b"", &[0x00]]), b"1");
    assert_eq!(answer(b"VERIFY", &[b"abcde", b"", &[0x00], b"3"]), b"3");
    assert_eq!(answer(b"VERIFY", &[b"abcde", b"xyz", &[0x00]]), b"0");
    // The four letter cases either side of them, which is what pins each
    // arm to the letter it actually tests.
    assert_eq!(answer(b"VERIFY", &[b"abcde", b"abc", b"M"]), b"1");
    assert_eq!(answer(b"VERIFY", &[b"abcde", b"abc", b"N"]), b"4");
    assert_eq!(answer(b"VERIFY", &[b"abcde", b"", b"M"]), b"0");
    assert_eq!(answer(b"VERIFY", &[b"abcde", b"", b"N"]), b"1");

    assert_eq!(
        raised(b"STRIP", &[Some(b"ab"), Some(&[0x01])]),
        (93, 915, vec![b"BLT".to_vec(), vec![0x01]])
    );
    assert_eq!(
        raised(b"VERIFY", &[Some(b"a"), Some(b"b"), Some(&[0x01])]),
        (93, 915, vec![b"MN".to_vec(), vec![0x01]])
    );
}

/// A raiser this module owns renders through the catalogue, so a wrong
/// sub-number shows up as a message that is not the oracle's.
#[test]
fn the_new_raisers_render_the_oracles_own_message_text() {
    let cases = [
        (
            Raised::argument_not_whole(b"LEFT", 2, b"q"),
            "LEFT argument 2 must be a whole number; found \"q\".",
        ),
        (
            Raised::argument_not_a_pad(b"LEFT", 3, b"xx"),
            "LEFT argument 3 must be a single character; found \"xx\".",
        ),
        (
            Raised::invalid_length(b"-1"),
            "Invalid length argument specified; found \"-1\".",
        ),
        (
            Raised::invalid_position(b"0"),
            "Invalid position argument specified; found \"0\".",
        ),
        (
            Raised::argument_not_non_negative(1, b"-1"),
            "Method argument 1 must be zero or a positive whole number; found \"-1\".",
        ),
        (
            Raised::invalid_option("BLT", b"Xyz"),
            "Method option must be one of \"BLT\"; found \"Xyz\".",
        ),
        (Raised::system_resources(), "System resources exhausted."),
    ];
    for (raised, expected) in cases {
        let rendered = String::from_utf8(raised.report(&crate::error::ClauseSite {
            sites: &[],
            path: "/p.rex",
        }))
        .expect("the catalogue is UTF-8");
        assert!(
            rendered.contains(expected),
            "{rendered:?} does not carry {expected:?}"
        );
    }
}

/// Every builtin whose answer is a count, a length, an index or a
/// position hands it back in the tagged representation, through
/// `Interp::counted`.
#[test]
fn a_counted_answer_is_tagged_rather_than_a_heap_string() {
    /// A builtin's name, the arguments to call it with, and the integer
    /// its answer must decode to.
    type CountedCase = (&'static [u8], &'static [&'static [u8]], i64);

    let counted: &[CountedCase] = &[
        (b"LENGTH", &[b"hello"], 5),
        (b"POS", &[b"an", b"banana"], 2),
        (b"LASTPOS", &[b"an", b"banana"], 4),
        (b"COMPARE", &[b"abc", b"abd"], 3),
        (b"COUNTSTR", &[b"a", b"banana"], 3),
        (b"VERIFY", &[b"abcd", b"abc"], 4),
        (b"WORDS", &[b"a b c"], 3),
        (b"WORDINDEX", &[b"a b c", b"2"], 3),
        (b"WORDLENGTH", &[b"a bb c", b"2"], 2),
        (b"WORDPOS", &[b"b", b"a b c"], 2),
    ];
    for (name, arguments, expected) in counted {
        let (handle, bytes) = call_handle(name, arguments);
        assert!(
            matches!(handle.decode(), Decoded::SmallInt(value) if value == *expected),
            "{} answered {:?}, not SmallInt({expected})",
            String::from_utf8_lossy(name),
            handle.decode()
        );
        assert_eq!(bytes, expected.to_string().into_bytes());
    }

    // The adjacent success, and it is what pins the rule to *counted*
    // answers rather than to "anything that looks like a number": a
    // substring of digits keeps its own bytes, because those bytes are
    // the value and no integer stands behind them.
    // `SUBSTR('012345',1,3)` is `012`, which no `SmallInt` renders.
    let (handle, bytes) = call_handle(b"SUBSTR", &[b"012345", b"1", b"3"]);
    assert!(
        !matches!(handle.decode(), Decoded::SmallInt(_)),
        "a substring is not a counted answer, it is its own bytes"
    );
    assert_eq!(bytes, b"012");
}
