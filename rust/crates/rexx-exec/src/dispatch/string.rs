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

//! `String`'s primitive methods.
//!
//! A child of [`super`] rather than a sibling, for the reason its `mod native`
//! comment gives: the rows point at `NativeMethod`s, whose parameter list
//! names types only that module can.
//!
//! # A second table, chained rather than merged
//!
//! [`NATIVE_METHODS`] is registered beside [`super::NATIVE_METHODS`] in
//! `ObjectModel::build`, which reads the two as one sequence. Chaining rather
//! than merging is what keeps this file's rows and bodies together: a
//! receiver whose methods are all one shape does not need its rows
//! interleaved alphabetically with every other class's.
//!
//! **A row's method name is looked up in the class's own dictionary and the
//! build panics if it is not there**, so a name that does not appear in
//! `String`'s behaviour cannot be added here quietly.
//!
//! # The receiver's bytes, and what that rules out
//!
//! `Interp::to_text` is the whole of a `String` receiver's state, so a body
//! here is that call, an argument layer, and a byte core. There is no
//! `Body::Instance` to reach and nothing to keep between sends.
//!
//! **The argument layer is not the like-named builtin's.** The two agree on
//! the answer and on nothing else -- measured on `left`, a missing argument
//! is 93.903 at rc 163 through the method and 40.3 at rc 216 through the
//! builtin, a non-numeric one is 93.923 against 40.12, and the argument
//! numbering differs by one because the receiver is the builtin's first
//! argument. Share the byte core; never the builtin's argument handling.

use super::{
    Arity, Cleared, Failure, Interp, NativeMethod, ObjRef, backward_search,
    backward_search_arguments, ends_with, forward_search, forward_search_arguments,
    match_region_arguments, match_region_over, starts_with, substr_arguments, verify_arguments,
    wordpos_arguments,
};

/// `RexxString::posRexx` (`classes/StringClassMisc.cpp:581`).
fn native_string_pos(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let found = string_pos(interp, receiver, args, crate::builtin::string::find_forward)?;
    Ok(Some(interp.counted(found)))
}

/// `RexxString::caselessPosRexx` (`classes/StringClassMisc.cpp:612`).
fn native_string_caselesspos(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let found = string_pos(
        interp,
        receiver,
        args,
        crate::builtin::string::caseless_find_forward,
    )?;
    Ok(Some(interp.counted(found)))
}

/// `RexxString::containsRexx` (`classes/StringClassMisc.cpp:597`): `POS`'s
/// search reported as a boolean.
fn native_string_contains(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let found = string_pos(interp, receiver, args, crate::builtin::string::find_forward)?;
    Ok(Some(crate::eval::logical(found > 0)))
}

/// `RexxString::caselessContains` (`classes/StringClassMisc.cpp:632`).
fn native_string_caselesscontains(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let found = string_pos(
        interp,
        receiver,
        args,
        crate::builtin::string::caseless_find_forward,
    )?;
    Ok(Some(crate::eval::logical(found > 0)))
}

/// [`forward_search`] over the receiver's own text.
///
/// **The result buffer is taken before the bytes and not after**, because
/// `Interp::to_text` borrows the interpreter mutably where the buffer's
/// accessor does not; resolving the receiver into an owned `Vec` instead would
/// put an allocation on every send.
fn string_pos(
    interp: &mut Interp,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
    scan: fn(&[u8], &[u8], usize, usize) -> usize,
) -> Result<usize, Failure> {
    let (needle, start, range) = forward_search_arguments(interp, args)?;
    let found = {
        let bytes = interp.to_text(receiver);
        forward_search(&bytes, &needle, start, range, scan)
    };
    interp.give_result_buffer(needle);
    Ok(found)
}

/// `RexxString::lastPosRexx` (`classes/StringClassMisc.cpp:352`).
fn native_string_lastpos(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let found = string_lastpos(
        interp,
        receiver,
        args,
        crate::builtin::string::find_backward,
    )?;
    Ok(Some(interp.counted(found)))
}

/// `RexxString::caselessLastPosRexx` (`classes/StringClassMisc.cpp:366`).
fn native_string_caselesslastpos(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let found = string_lastpos(
        interp,
        receiver,
        args,
        crate::builtin::string::caseless_find_backward,
    )?;
    Ok(Some(interp.counted(found)))
}

/// [`backward_search`] over the receiver's own text.
fn string_lastpos(
    interp: &mut Interp,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
    scan: fn(&[u8], &[u8], usize, usize) -> usize,
) -> Result<usize, Failure> {
    let (needle, start, range) = backward_search_arguments(interp, args)?;
    let found = {
        let bytes = interp.to_text(receiver);
        backward_search(&bytes, &needle, start, range, scan)
    };
    interp.give_result_buffer(needle);
    Ok(found)
}

/// `RexxString::countStrRexx` (`classes/StringClassMisc.cpp:419`).
fn native_string_countstr(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let count = string_countstr(
        interp,
        receiver,
        args,
        crate::builtin::string::count_occurrences,
    )?;
    Ok(Some(interp.counted(count)))
}

/// `RexxString::caselessCountStrRexx` (`classes/StringClassMisc.cpp:434`).
fn native_string_caselesscountstr(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let count = string_countstr(
        interp,
        receiver,
        args,
        crate::builtin::string::caseless_count_occurrences,
    )?;
    Ok(Some(interp.counted(count)))
}

/// The non-overlapping count of `needle` in the receiver's own text.
///
/// The limit is unbounded: `countStrRexx` passes `Numerics::MAX_WHOLENUMBER`
/// (`classes/StringClassMisc.cpp:423`), which no count over a string can reach.
fn string_countstr(
    interp: &mut Interp,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
    count: fn(&[u8], &[u8], usize) -> usize,
) -> Result<usize, Failure> {
    let needle = super::string_method_argument(interp, args, 0)?;
    let found = {
        let bytes = interp.to_text(receiver);
        count(&bytes, &needle, usize::MAX)
    };
    interp.give_result_buffer(needle);
    Ok(found)
}

/// `RexxString::wordPos` (`classes/StringClassWord.cpp:256`).
fn native_string_wordpos(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let found = string_wordpos(interp, receiver, args, crate::builtin::word::wordpos_bytes)?;
    Ok(Some(interp.counted(found)))
}

/// `RexxString::caselessWordPos` (`classes/StringClassWord.cpp:284`).
fn native_string_caselesswordpos(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let found = string_wordpos(
        interp,
        receiver,
        args,
        crate::builtin::word::caseless_wordpos_bytes,
    )?;
    Ok(Some(interp.counted(found)))
}

/// `RexxString::containsWord` (`classes/StringClassWord.cpp:270`): `WORDPOS`'s
/// search reported as a boolean.
fn native_string_containsword(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let found = string_wordpos(interp, receiver, args, crate::builtin::word::wordpos_bytes)?;
    Ok(Some(crate::eval::logical(found > 0)))
}

/// `RexxString::caselessContainsWord` (`classes/StringClassWord.cpp:298`).
fn native_string_caselesscontainsword(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let found = string_wordpos(
        interp,
        receiver,
        args,
        crate::builtin::word::caseless_wordpos_bytes,
    )?;
    Ok(Some(crate::eval::logical(found > 0)))
}

/// [`wordpos_arguments`]' search over the receiver's own text.
fn string_wordpos(
    interp: &mut Interp,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
    scan: fn(&[u8], &[u8], usize) -> usize,
) -> Result<usize, Failure> {
    let (phrase, start) = wordpos_arguments(interp, args)?;
    let found = {
        let bytes = interp.to_text(receiver);
        scan(&phrase, &bytes, start)
    };
    interp.give_result_buffer(phrase);
    Ok(found)
}

/// `RexxString::startsWithRexx` (`classes/StringClassMisc.cpp:874`).
fn native_string_startswith(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_at_an_end(interp, receiver, args, starts_with, <[u8]>::eq)
}

/// `RexxString::caselessStartsWithRexx` (`classes/StringClassMisc.cpp:888`).
fn native_string_caselessstartswith(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_at_an_end(
        interp,
        receiver,
        args,
        starts_with,
        crate::builtin::string::caseless_eq,
    )
}

/// `RexxString::endsWithRexx` (`classes/StringClassMisc.cpp:902`).
fn native_string_endswith(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_at_an_end(interp, receiver, args, ends_with, <[u8]>::eq)
}

/// `RexxString::caselessEndsWithRexx` (`classes/StringClassMisc.cpp:923`).
fn native_string_caselessendswith(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_at_an_end(
        interp,
        receiver,
        args,
        ends_with,
        crate::builtin::string::caseless_eq,
    )
}

/// [`starts_with`] or [`ends_with`] over the receiver's own text.
///
/// **The argument is named, and a missing one is 88.901 at rc 168** rather
/// than the 93.903 at rc 163 every other method in this file raises -- the
/// C++ takes it through `stringArgument(other, "match")`, so the name reaches
/// the message. Measured, oracle: `'abc'~startsWith` is
/// `Missing argument; argument match is required.`
fn string_at_an_end(
    interp: &mut Interp,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
    at: fn(&[u8], &[u8], fn(&[u8], &[u8]) -> bool) -> bool,
    matches: fn(&[u8], &[u8]) -> bool,
) -> Result<Option<ObjRef>, Failure> {
    let needle = super::named_string_argument(interp, args, 0, "match")?;
    let answer = {
        let bytes = interp.to_text(receiver);
        at(&bytes, &needle, matches)
    };
    interp.give_result_buffer(needle);
    Ok(Some(crate::eval::logical(answer)))
}

/// `RexxString::match` (`classes/StringClassMisc.cpp:792`).
fn native_string_match(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_match(interp, receiver, args, <[u8]>::eq)
}

/// `RexxString::caselessMatch` (`classes/StringClassMisc.cpp:837`).
fn native_string_caselessmatch(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_match(interp, receiver, args, crate::builtin::string::caseless_eq)
}

/// [`match_region_over`] over the receiver's own text.
///
/// **A start past the end answers `0` before the second argument is read**,
/// so a bad `other` there is not a refusal. Measured, oracle rc 0:
/// `'abcabc'~match(99, .nil)` is `0`.
fn string_match(
    interp: &mut Interp,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
    matches: fn(&[u8], &[u8]) -> bool,
) -> Result<Option<ObjRef>, Failure> {
    let start = super::required_position_argument(interp, args, 0)?;
    if start > interp.text_len(receiver) {
        return Ok(Some(crate::eval::logical(false)));
    }
    let other = super::string_method_argument(interp, args, 1)?;
    let region = match_region_arguments(interp, args, other.len())?;
    let answer = {
        let bytes = interp.to_text(receiver);
        match_region_over(&bytes, start, &other, region, matches)
    };
    interp.give_result_buffer(other);
    Ok(Some(crate::eval::logical(answer)))
}

/// `RexxString::matchChar` (`classes/StringClassMisc.cpp:1001`).
fn native_string_matchchar(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_matchchar(interp, receiver, args, u8::eq)
}

/// `RexxString::caselessMatchChar` (`classes/StringClassMisc.cpp:1038`).
fn native_string_caselessmatchchar(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_matchchar(interp, receiver, args, u8::eq_ignore_ascii_case)
}

/// Whether the byte at `position` is in the set, over the receiver's own text.
///
/// **A position past the end answers `0` before the set is read**, the same
/// order [`string_match`] keeps.
fn string_matchchar(
    interp: &mut Interp,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
    matches: fn(&u8, &u8) -> bool,
) -> Result<Option<ObjRef>, Failure> {
    let position = super::required_position_argument(interp, args, 0)?;
    if position > interp.text_len(receiver) {
        return Ok(Some(crate::eval::logical(false)));
    }
    let set = super::string_method_argument(interp, args, 1)?;
    let answer = {
        let bytes = interp.to_text(receiver);
        bytes
            .get(position - 1)
            .is_some_and(|byte| set.iter().any(|member| matches(member, byte)))
    };
    interp.give_result_buffer(set);
    Ok(Some(crate::eval::logical(answer)))
}

/// `RexxString::substr` (`classes/StringClassSub.cpp:598`).
fn native_string_substr(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let (start, length, pad) = substr_arguments(interp, args)?;
    let mut out = interp.take_result_buffer();
    {
        let bytes = interp.to_text(receiver);
        crate::builtin::string::substr_bytes(&mut out, &bytes, start, length, pad)?;
    }
    Ok(Some(interp.text_built(out)))
}

/// `RexxString::brackets` (`classes/StringClassSub.cpp:615`): `substr`'s
/// two-argument form, the length defaulting to one byte, capped at the end of
/// the receiver, and never padding.
fn native_string_brackets(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let start = super::required_position_argument(interp, args, 0)? - 1;
    let length = super::optional_length_argument(interp, args, 1)?.unwrap_or(1);
    let mut out = interp.take_result_buffer();
    {
        let bytes = interp.to_text(receiver);
        let capped = length.min(bytes.len().saturating_sub(start));
        crate::builtin::string::substr_bytes(&mut out, &bytes, start, Some(capped), b' ')?;
    }
    Ok(Some(interp.text_built(out)))
}

/// `RexxString::subchar` (`classes/StringClassSub.cpp:635`): the one byte at
/// `position`, or the empty string past the end.
fn native_string_subchar(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let position = super::required_position_argument(interp, args, 0)?;
    let mut out = interp.take_result_buffer();
    {
        let bytes = interp.to_text(receiver);
        out.extend(bytes.get(position - 1));
    }
    Ok(Some(interp.text_built(out)))
}

/// `RexxString::subWord` (`classes/StringClassWord.cpp:180`).
fn native_string_subword(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let position = super::required_position_argument(interp, args, 0)?;
    let count = super::optional_length_argument(interp, args, 1)?;
    let mut out = interp.take_result_buffer();
    {
        let bytes = interp.to_text(receiver);
        let found = crate::builtin::word::subword_range(&bytes, position, count);
        out.extend_from_slice(&bytes[found]);
    }
    Ok(Some(interp.text_built(out)))
}

/// `RexxString::subWords` (`classes/StringClassWord.cpp:200`).
///
/// **Answers an `Array`, not a string.** The C++ return type is `ArrayClass *`
/// where `subWord`'s is `RexxString *`, so a witness that only renders the
/// answer cannot tell one from the other.
fn native_string_subwords(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let position = super::optional_position_argument(interp, args, 0)?.unwrap_or(1);
    let count = super::optional_length_argument(interp, args, 1)?.unwrap_or(usize::MAX);
    let words: Vec<Vec<u8>> = {
        let bytes = interp.to_text(receiver);
        crate::builtin::word::word_slices(&bytes)
            .into_iter()
            .skip(position - 1)
            .take(count)
            .map(<[u8]>::to_vec)
            .collect()
    };
    Ok(Some(super::array_of_texts(interp, words)?))
}

/// `RexxString::word` (`classes/StringClassWord.cpp:214`).
fn native_string_word(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let position = super::required_position_argument(interp, args, 0)?;
    let mut out = interp.take_result_buffer();
    {
        let bytes = interp.to_text(receiver);
        let found = crate::builtin::word::word_range(&bytes, position).unwrap_or(0..0);
        out.extend_from_slice(&bytes[found]);
    }
    Ok(Some(interp.text_built(out)))
}

/// `RexxString::wordIndex` (`classes/StringClassWord.cpp:228`): the 1-based
/// byte at which the word starts, or 0.
fn native_string_wordindex(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let position = super::required_position_argument(interp, args, 0)?;
    let index = {
        let bytes = interp.to_text(receiver);
        crate::builtin::word::word_range(&bytes, position).map_or(0, |word| word.start + 1)
    };
    Ok(Some(interp.counted(index)))
}

/// `RexxString::wordLength` (`classes/StringClassWord.cpp:242`).
fn native_string_wordlength(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let position = super::required_position_argument(interp, args, 0)?;
    let length = {
        let bytes = interp.to_text(receiver);
        crate::builtin::word::word_range(&bytes, position).map_or(0, |word| word.len())
    };
    Ok(Some(interp.counted(length)))
}

/// `RexxString::words` (`classes/StringClassWord.cpp:309`).
fn native_string_words(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let count = {
        let bytes = interp.to_text(receiver);
        crate::builtin::word::word_count(&bytes)
    };
    Ok(Some(interp.counted(count)))
}

/// `RexxString::verify` (`classes/StringClassMisc.cpp:771`).
fn native_string_verify(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let (reference, option, start, range) = verify_arguments(interp, args)?;
    let answer = {
        let bytes = interp.to_text(receiver);
        crate::builtin::string::verify_bytes(&bytes, &reference, option, start, range)
    };
    interp.give_result_buffer(reference);
    Ok(Some(interp.counted(answer)))
}

/// `String`'s rows, in the shape [`super::NATIVE_METHODS`] uses.
pub(super) static NATIVE_METHODS: &[(&str, &str, Arity, NativeMethod)] = &[
    (
        "String",
        "CASELESSCONTAINS",
        Arity::Fixed(3),
        native_string_caselesscontains,
    ),
    (
        "String",
        "CASELESSCONTAINSWORD",
        Arity::Fixed(2),
        native_string_caselesscontainsword,
    ),
    (
        "String",
        "CASELESSCOUNTSTR",
        Arity::Fixed(1),
        native_string_caselesscountstr,
    ),
    (
        "String",
        "CASELESSENDSWITH",
        Arity::Fixed(1),
        native_string_caselessendswith,
    ),
    (
        "String",
        "CASELESSLASTPOS",
        Arity::Fixed(3),
        native_string_caselesslastpos,
    ),
    (
        "String",
        "CASELESSMATCH",
        Arity::Fixed(4),
        native_string_caselessmatch,
    ),
    (
        "String",
        "CASELESSMATCHCHAR",
        Arity::Fixed(2),
        native_string_caselessmatchchar,
    ),
    (
        "String",
        "CASELESSPOS",
        Arity::Fixed(3),
        native_string_caselesspos,
    ),
    (
        "String",
        "CASELESSSTARTSWITH",
        Arity::Fixed(1),
        native_string_caselessstartswith,
    ),
    (
        "String",
        "CASELESSWORDPOS",
        Arity::Fixed(2),
        native_string_caselesswordpos,
    ),
    (
        "String",
        "CONTAINS",
        Arity::Fixed(3),
        native_string_contains,
    ),
    (
        "String",
        "CONTAINSWORD",
        Arity::Fixed(2),
        native_string_containsword,
    ),
    (
        "String",
        "COUNTSTR",
        Arity::Fixed(1),
        native_string_countstr,
    ),
    (
        "String",
        "ENDSWITH",
        Arity::Fixed(1),
        native_string_endswith,
    ),
    ("String", "LASTPOS", Arity::Fixed(3), native_string_lastpos),
    ("String", "MATCH", Arity::Fixed(4), native_string_match),
    (
        "String",
        "MATCHCHAR",
        Arity::Fixed(2),
        native_string_matchchar,
    ),
    ("String", "POS", Arity::Fixed(3), native_string_pos),
    (
        "String",
        "STARTSWITH",
        Arity::Fixed(1),
        native_string_startswith,
    ),
    ("String", "SUBCHAR", Arity::Fixed(1), native_string_subchar),
    ("String", "SUBSTR", Arity::Fixed(3), native_string_substr),
    ("String", "SUBWORD", Arity::Fixed(2), native_string_subword),
    (
        "String",
        "SUBWORDS",
        Arity::Fixed(2),
        native_string_subwords,
    ),
    ("String", "VERIFY", Arity::Fixed(4), native_string_verify),
    ("String", "WORD", Arity::Fixed(1), native_string_word),
    (
        "String",
        "WORDINDEX",
        Arity::Fixed(1),
        native_string_wordindex,
    ),
    (
        "String",
        "WORDLENGTH",
        Arity::Fixed(1),
        native_string_wordlength,
    ),
    ("String", "WORDPOS", Arity::Fixed(2), native_string_wordpos),
    ("String", "WORDS", Arity::Fixed(0), native_string_words),
    ("String", "[]", Arity::Fixed(2), native_string_brackets),
];
