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
    backward_search_arguments, case_shift_arguments, changestr_arguments, copies_argument,
    delete_arguments, delword_arguments, ends_with, forward_search, forward_search_arguments,
    insert_arguments, match_region_arguments, match_region_over, overlay_arguments, pad_arguments,
    replace_at_bytes, replace_at_plan, space_arguments, starts_with, substr_arguments,
    translate_arguments, translate_in_table, verify_arguments, wordpos_arguments,
};
use crate::error::Raised;
use crate::eval::logical_value;
use rexx_parse::Operator;

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

/// `RexxString::insert` (`classes/StringClassSub.cpp:170`).
///
/// **Answers a new string and leaves the receiver alone**, where the
/// `MutableBuffer` row writes itself and answers the receiver. Measured,
/// oracle: `'abcdef'~insert('XY', 2)` is `abXYcdef` with `'abcdef'` unchanged,
/// and the same send to a buffer returns the buffer, now `abXYcdef`.
fn native_string_insert(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let (new, begin, length, pad) = insert_arguments(interp, args)?;
    let mut out = interp.take_result_buffer();
    {
        let bytes = interp.to_text(receiver);
        crate::builtin::string::insert_bytes(&mut out, &bytes, &new, begin, length, pad)?;
    }
    interp.give_result_buffer(new);
    Ok(Some(interp.text_built(out)))
}

/// `RexxString::overlay` (`classes/StringClassSub.cpp:292`).
fn native_string_overlay(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let (new, begin, length, pad) = overlay_arguments(interp, args)?;
    let mut out = interp.take_result_buffer();
    {
        let bytes = interp.to_text(receiver);
        crate::builtin::string::overlay_bytes(&mut out, &bytes, &new, begin, length, pad)?;
    }
    interp.give_result_buffer(new);
    Ok(Some(interp.text_built(out)))
}

/// `RexxString::replaceAt` (`classes/StringClassSub.cpp:376`).
///
/// **This is the one row of the shared set whose argument layer is not
/// shared**, so it reads its own. `RexxString::replaceAt` opens
/// `stringArgument(newStrObj, ARG_ONE)` (`:380`) where
/// `MutableBuffer::replaceAt` opens `stringArgument(str, "new")`
/// (`classes/MutableBufferClass.cpp:572`) and takes its position and pad by
/// name too. Measured, oracle: a bare send is 93.903 here and 88.901 there, a
/// zero position 93.924 against 88.912, a two-character pad 93.922 against
/// 88.910. Only `.nil` in the first position agrees, at 88.909.
fn native_string_replaceat(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let new = super::string_method_argument(interp, args, 0)?;
    let begin = super::required_position_argument(interp, args, 1)? - 1;
    let length = super::optional_length_argument(interp, args, 2)?;
    let pad = super::pad_method_argument(interp, args, 3)?.unwrap_or(b' ');
    let mut out = interp.take_result_buffer();
    {
        let bytes = interp.to_text(receiver);
        let (replaced, _) = replace_at_plan(bytes.len(), begin, length, new.len());
        replace_at_bytes(&mut out, &bytes, &new, begin, replaced, pad)?;
    }
    interp.give_result_buffer(new);
    Ok(Some(interp.text_built(out)))
}

/// `RexxString::delstr` (`classes/StringClassSub.cpp:117`).
fn native_string_delstr(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let (begin, range) = delete_arguments(interp, args)?;
    let mut out = interp.take_result_buffer();
    {
        let bytes = interp.to_text(receiver);
        out.extend_from_slice(&bytes);
    }
    crate::builtin::string::delete_range(&mut out, begin, range);
    Ok(Some(interp.text_built(out)))
}

/// `RexxString::delWord` (`classes/StringClassWord.cpp:58`).
fn native_string_delword(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let (position, count) = delword_arguments(interp, args)?;
    let mut out = interp.take_result_buffer();
    {
        let bytes = interp.to_text(receiver);
        out.extend_from_slice(&bytes);
    }
    crate::builtin::word::delword_bytes(&mut out, position, count);
    Ok(Some(interp.text_built(out)))
}

/// `RexxString::changeStr` (`classes/StringClassMisc.cpp:451`).
fn native_string_changestr(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_changestr(
        interp,
        receiver,
        args,
        crate::builtin::string::changestr_bytes,
    )
}

/// `RexxString::caselessChangeStr` (`classes/StringClassMisc.cpp:516`).
fn native_string_caselesschangestr(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_changestr(
        interp,
        receiver,
        args,
        crate::builtin::string::caseless_changestr_bytes,
    )
}

/// Every occurrence of the needle replaced, into a new string.
type ChangeStr = fn(&mut Vec<u8>, &[u8], &[u8], &[u8], usize) -> Result<(), super::Raised>;

/// [`changestr_arguments`]' replacement over the receiver's own text.
fn string_changestr(
    interp: &mut Interp,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
    change: ChangeStr,
) -> Result<Option<ObjRef>, Failure> {
    let (needle, replacement, limit) = changestr_arguments(interp, args)?;
    let mut out = interp.take_result_buffer();
    {
        let bytes = interp.to_text(receiver);
        change(&mut out, &bytes, &needle, &replacement, limit)?;
    }
    interp.give_result_buffer(needle);
    interp.give_result_buffer(replacement);
    Ok(Some(interp.text_built(out)))
}

/// `RexxString::translate` (`classes/StringClassMisc.cpp:681`).
///
/// **With no tables, pad or start it is an upper-case shift**, and the range
/// arguments move to positions 3 and 4 -- measured, oracle rc 0:
/// `'abcABCabc'~translate` is `ABCABCABC`.
fn native_string_translate(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if args.iter().take(3).all(Option::is_none) {
        return string_case_shift(interp, receiver, args, 3, u8::to_ascii_uppercase);
    }
    let (out_table, in_table, pad, start, range) = translate_arguments(interp, args)?;
    let mut out = interp.take_result_buffer();
    {
        let bytes = interp.to_text(receiver);
        out.extend_from_slice(&bytes);
    }
    crate::builtin::string::translate_bytes(
        &mut out,
        &out_table,
        translate_in_table(&in_table),
        pad,
        start,
        range,
    );
    Ok(Some(interp.text_built(out)))
}

/// `RexxString::lowerRexx` (`classes/StringClass.cpp:1732`).
fn native_string_lower(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_case_shift(interp, receiver, args, 0, u8::to_ascii_lowercase)
}

/// A case shift over the receiver's own text, answering a new string.
fn string_case_shift(
    interp: &mut Interp,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
    first: usize,
    shift: fn(&u8) -> u8,
) -> Result<Option<ObjRef>, Failure> {
    let (start, range) = case_shift_arguments(interp, args, first)?;
    let mut out = interp.take_result_buffer();
    {
        let bytes = interp.to_text(receiver);
        out.extend_from_slice(&bytes);
    }
    crate::builtin::string::case_shift_bytes(&mut out, start, range, shift);
    Ok(Some(interp.text_built(out)))
}

/// `RexxString::space` (`classes/StringClassWord.cpp:121`).
fn native_string_space(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let (gap, pad) = space_arguments(interp, args)?;
    let mut out = interp.take_result_buffer();
    {
        let bytes = interp.to_text(receiver);
        crate::builtin::string::space_bytes(&mut out, &bytes, gap, pad)?;
    }
    Ok(Some(interp.text_built(out)))
}

/// `String~append`, which is **`RexxString::concatRexx` under a second name**
/// -- `AddMethod("Append", RexxString::concatRexx, 1)`
/// (`memory/Setup.cpp:578`), so it concatenates and takes exactly one
/// argument. It routes to [`Interp::apply_binary`] for that reason rather
/// than reading its argument as a string: concatenation *renders* what it is
/// given, so `.nil` becomes `The NIL object` where a string argument would be
/// refused at 88.909.
///
/// **It shares nothing with `MutableBuffer~append`**, which is
/// `MutableBuffer::appendRexx` at `A_COUNT` (`memory/Setup.cpp:1422`): that one
/// is variadic and writes the buffer. Measured, oracle:
/// `.MutableBuffer~new('abc')~append('X', 'Y')` is `abcXY` where
/// `'abc'~append('X', 'Y')` is 93.902, and `'abc'~append(.nil)` answers
/// `abcThe NIL object` because concatenation renders its argument.
fn native_string_append(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let Some(tail) = args.first().copied().flatten() else {
        return Err(super::Raised::missing_method_argument(1).into());
    };
    Ok(Some(interp.apply_binary(
        rexx_parse::Operator::Concatenate,
        receiver,
        tail,
    )?))
}

/// Every binary operator `String` answers as a message, and the
/// [`rexx_parse::Operator`] each one is.
///
/// **The name column is the operator's own spelling**, which
/// `operator_spellings_match_the_parser` asserts against
/// [`rexx_parse::Operator::spelling`] rather than restating here. The oracle
/// registers all of them at one argument -- `AddMethod("+", RexxString::plus,
/// 1)` and the rest of that block (`memory/Setup.cpp:649`-`:677`) -- and `\\`
/// and `?` are the two that are not in it, at zero and two.
#[cfg(test)]
const BINARY_OPERATORS: &[(&str, Operator)] = &[
    ("+", Operator::Plus),
    ("-", Operator::Subtract),
    ("*", Operator::Multiply),
    ("**", Operator::Power),
    ("/", Operator::Divide),
    ("%", Operator::IntDiv),
    ("//", Operator::Remainder),
    ("", Operator::Abuttal),
    ("||", Operator::Concatenate),
    (" ", Operator::Blank),
    ("=", Operator::Equal),
    ("\\=", Operator::BackslashEqual),
    ("<>", Operator::LessThanGreaterThan),
    ("><", Operator::GreaterThanLessThan),
    (">", Operator::GreaterThan),
    ("<", Operator::LessThan),
    (">=", Operator::GreaterThanEqual),
    ("\\<", Operator::BackslashLessThan),
    ("<=", Operator::LessThanEqual),
    ("\\>", Operator::BackslashGreaterThan),
    ("==", Operator::StrictEqual),
    ("\\==", Operator::StrictBackslashEqual),
    (">>", Operator::StrictGreaterThan),
    ("<<", Operator::StrictLessThan),
    (">>=", Operator::StrictGreaterThanEqual),
    ("\\<<", Operator::StrictBackslashLessThan),
    ("<<=", Operator::StrictLessThanEqual),
    ("\\>>", Operator::StrictBackslashGreaterThan),
    ("&", Operator::And),
    ("|", Operator::Or),
    ("&&", Operator::Xor),
];

/// `String~"+"`, [`Operator::Plus`] through the arithmetic pair.
fn native_string_op_plus(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_arithmetic_operator(interp, receiver, args, Operator::Plus)
}

/// `String~"-"`, [`Operator::Subtract`] through the arithmetic pair.
fn native_string_op_subtract(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_arithmetic_operator(interp, receiver, args, Operator::Subtract)
}

/// `String~"*"`, [`Operator::Multiply`] through the arithmetic pair.
fn native_string_op_multiply(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_arithmetic_operator(interp, receiver, args, Operator::Multiply)
}

/// `String~"**"`, [`Operator::Power`] through the arithmetic pair.
fn native_string_op_power(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_arithmetic_operator(interp, receiver, args, Operator::Power)
}

/// `String~"/"`, [`Operator::Divide`] through the arithmetic pair.
fn native_string_op_divide(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_arithmetic_operator(interp, receiver, args, Operator::Divide)
}

/// `String~"%"`, [`Operator::IntDiv`] through the arithmetic pair.
fn native_string_op_intdiv(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_arithmetic_operator(interp, receiver, args, Operator::IntDiv)
}

/// `String~"//"`, [`Operator::Remainder`] through the arithmetic pair.
fn native_string_op_remainder(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_arithmetic_operator(interp, receiver, args, Operator::Remainder)
}

/// `String~""`, [`Operator::Abuttal`] through [`Interp::apply_binary`].
fn native_string_op_abuttal(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_binary_operator(interp, receiver, args, Operator::Abuttal)
}

/// `String~"||"`, [`Operator::Concatenate`] through [`Interp::apply_binary`].
fn native_string_op_concatenate(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_binary_operator(interp, receiver, args, Operator::Concatenate)
}

/// `String~" "`, [`Operator::Blank`] through [`Interp::apply_binary`].
fn native_string_op_blank(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_binary_operator(interp, receiver, args, Operator::Blank)
}

/// `String~"="`, [`Operator::Equal`] through [`Interp::apply_binary`].
fn native_string_op_equal(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_binary_operator(interp, receiver, args, Operator::Equal)
}

/// `String~"\\="`, [`Operator::BackslashEqual`] through [`Interp::apply_binary`].
fn native_string_op_backslash_equal(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_binary_operator(interp, receiver, args, Operator::BackslashEqual)
}

/// `String~"<>"`, [`Operator::LessThanGreaterThan`] through [`Interp::apply_binary`].
fn native_string_op_less_greater(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_binary_operator(interp, receiver, args, Operator::LessThanGreaterThan)
}

/// `String~"><"`, [`Operator::GreaterThanLessThan`] through [`Interp::apply_binary`].
fn native_string_op_greater_less(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_binary_operator(interp, receiver, args, Operator::GreaterThanLessThan)
}

/// `String~">"`, [`Operator::GreaterThan`] through [`Interp::apply_binary`].
fn native_string_op_greater(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_binary_operator(interp, receiver, args, Operator::GreaterThan)
}

/// `String~"<"`, [`Operator::LessThan`] through [`Interp::apply_binary`].
fn native_string_op_less(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_binary_operator(interp, receiver, args, Operator::LessThan)
}

/// `String~">="`, [`Operator::GreaterThanEqual`] through [`Interp::apply_binary`].
fn native_string_op_greater_equal(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_binary_operator(interp, receiver, args, Operator::GreaterThanEqual)
}

/// `String~"\\<"`, [`Operator::BackslashLessThan`] through [`Interp::apply_binary`].
fn native_string_op_backslash_less(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_binary_operator(interp, receiver, args, Operator::BackslashLessThan)
}

/// `String~"<="`, [`Operator::LessThanEqual`] through [`Interp::apply_binary`].
fn native_string_op_less_equal(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_binary_operator(interp, receiver, args, Operator::LessThanEqual)
}

/// `String~"\\>"`, [`Operator::BackslashGreaterThan`] through [`Interp::apply_binary`].
fn native_string_op_backslash_greater(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_binary_operator(interp, receiver, args, Operator::BackslashGreaterThan)
}

/// `String~"=="`, [`Operator::StrictEqual`] through [`Interp::apply_binary`].
fn native_string_op_strict_equal(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_binary_operator(interp, receiver, args, Operator::StrictEqual)
}

/// `String~"\\=="`, [`Operator::StrictBackslashEqual`] through [`Interp::apply_binary`].
fn native_string_op_strict_backslash_equal(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_binary_operator(interp, receiver, args, Operator::StrictBackslashEqual)
}

/// `String~">>"`, [`Operator::StrictGreaterThan`] through [`Interp::apply_binary`].
fn native_string_op_strict_greater(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_binary_operator(interp, receiver, args, Operator::StrictGreaterThan)
}

/// `String~"<<"`, [`Operator::StrictLessThan`] through [`Interp::apply_binary`].
fn native_string_op_strict_less(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_binary_operator(interp, receiver, args, Operator::StrictLessThan)
}

/// `String~">>="`, [`Operator::StrictGreaterThanEqual`] through [`Interp::apply_binary`].
fn native_string_op_strict_greater_equal(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_binary_operator(interp, receiver, args, Operator::StrictGreaterThanEqual)
}

/// `String~"\\<<"`, [`Operator::StrictBackslashLessThan`] through [`Interp::apply_binary`].
fn native_string_op_strict_backslash_less(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_binary_operator(interp, receiver, args, Operator::StrictBackslashLessThan)
}

/// `String~"<<="`, [`Operator::StrictLessThanEqual`] through [`Interp::apply_binary`].
fn native_string_op_strict_less_equal(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_binary_operator(interp, receiver, args, Operator::StrictLessThanEqual)
}

/// `String~"\\>>"`, [`Operator::StrictBackslashGreaterThan`] through [`Interp::apply_binary`].
fn native_string_op_strict_backslash_greater(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_binary_operator(interp, receiver, args, Operator::StrictBackslashGreaterThan)
}

/// `String~"&"`, [`Operator::And`] through [`Interp::apply_binary`].
fn native_string_op_and(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_binary_operator(interp, receiver, args, Operator::And)
}

/// `String~"|"`, [`Operator::Or`] through [`Interp::apply_binary`].
fn native_string_op_or(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_binary_operator(interp, receiver, args, Operator::Or)
}

/// `String~"&&"`, [`Operator::Xor`] through [`Interp::apply_binary`].
fn native_string_op_xor(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    string_binary_operator(interp, receiver, args, Operator::Xor)
}

/// One arithmetic operator sent as a message.
///
/// **Arithmetic does not go through [`Interp::apply_binary`]**, whose own doc
/// says so: `**`'s exponent is not converted the way its base is, so it does
/// not share the operand handling the other families do. The expression form
/// and `crate::ir::Op::Arith` both enter
/// [`Interp::arith_small_int`] and [`Interp::arith_general`] in this order,
/// and so does this, which is what keeps the three from disagreeing.
fn string_arithmetic_operator(
    interp: &mut Interp,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
    op: Operator,
) -> Result<Option<ObjRef>, Failure> {
    if args.first().copied().flatten().is_none() {
        match op {
            // **`+` and `-` sent with no operand are the prefix forms, not an
            // error.** Measured, oracle: `'12'~'+'()` is `12` and `'12'~'-'()`
            // is `-12`, so the unary reading belongs to exactly the two
            // operators the language has a prefix form for.
            Operator::Plus => {
                return Ok(Some(
                    interp.apply_prefix(rexx_parse::PrefixOp::Plus, receiver)?,
                ));
            }
            Operator::Subtract => {
                return Ok(Some(
                    interp.apply_prefix(rexx_parse::PrefixOp::Minus, receiver)?,
                ));
            }
            // The other five convert the receiver *before* they complain about
            // the operand. Measured, oracle: `.String~new('abc')~'*'()` is 41.1
            // naming `"abc"`, while `'12'~'*'()` is 93.903 -- so a receiver
            // that is not a number is what the send reports, and the missing
            // argument only surfaces once the receiver converts.
            _ => {
                interp.arith_operand(receiver)?;
            }
        }
    }
    let other = super::operator_argument(args)?;
    let value = match interp.arith_small_int(op, receiver, other) {
        Some(value) => value,
        None => interp.arith_general(op, receiver, other)?,
    };
    Ok(Some(value))
}

/// One binary operator sent as a message.
///
/// **It routes to [`Interp::apply_binary`], the same dispatch the expression
/// form enters**, so the two cannot come to disagree about what an operator
/// answers. The send's own frame is already pushed by [`Interp::invoke`] --
/// measured, the crate prints `Compiled method "SUBSTR" with scope
/// "MutableBuffer".` for a raising native send byte-identically to the oracle
/// -- so the traceback line an operator *message* carries and an operator
/// *expression* does not comes from the send rather than from here.
fn string_binary_operator(
    interp: &mut Interp,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
    op: Operator,
) -> Result<Option<ObjRef>, Failure> {
    let other = super::operator_argument(args)?;
    Ok(Some(interp.apply_binary(op, receiver, other)?))
}

/// `String~"\\"`, the prefix `\` as a message.
///
/// **The one operator row that takes no argument** --
/// `AddMethod("\\", RexxString::notOp, 0)` (`memory/Setup.cpp:674`) -- so it
/// routes to [`Interp::apply_prefix`] rather than `apply_binary`. It is a text
/// check and never a numeric one: measured, `say \'abc'` is 34.901, not 41.1.
fn native_string_op_not(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    Ok(Some(
        interp.apply_prefix(rexx_parse::PrefixOp::Not, receiver)?,
    ))
}

/// `String~"?"`, `RexxString::choiceRexx` (`classes/StringClass.cpp:2017`).
///
/// **The one row here whose name is not a [`rexx_parse::Operator`]** -- the
/// oracle registers it beside the operators, `AddMethod("?",
/// RexxString::choiceRexx, 2)` (`memory/Setup.cpp:678`), but the language has
/// no `?` operator for an expression to spell, so this is the only dispatch
/// there is and it has nothing to agree with.
///
/// The oracle's body is three lines and so is this, in that order: both
/// arguments are required and named, and only then is the receiver read as a
/// logical value. **The order is the whole of the behaviour** -- measured,
/// `'abc'~"?"()` is 88.901 naming `true value`, not the 34.901 its receiver
/// would otherwise earn.
fn native_string_op_choice(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let on_true = args
        .first()
        .copied()
        .flatten()
        .ok_or_else(|| Raised::missing_named_argument("true value"))?;
    let on_false = args
        .get(1)
        .copied()
        .flatten()
        .ok_or_else(|| Raised::missing_named_argument("false value"))?;
    let text = interp.to_text(receiver).to_vec();
    let holds = logical_value(&text).ok_or_else(|| Raised::not_logical(&text))?;
    Ok(Some(if holds { on_true } else { on_false }))
}

/// `String~"CENTER"` and `String~"CENTRE"`, `RexxString::center`
/// (`classes/StringClassSub.cpp:59`).
///
/// **The two names are one body**, registered twice
/// (`memory/Setup.cpp:582`-`:583`), the way the oracle registers them.
fn native_string_center(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let (width, pad) = pad_arguments(interp, args)?;
    let bytes = interp.to_text(receiver).to_vec();
    match crate::builtin::string::center_bytes(interp, &bytes, width, pad)? {
        Some(out) => Ok(Some(interp.text_built(out))),
        None => Ok(Some(receiver)),
    }
}

/// `String~"LEFT"`, `RexxString::left` (`classes/StringClassSub.cpp:248`).
fn native_string_left(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let (size, pad) = pad_arguments(interp, args)?;
    let bytes = interp.to_text(receiver).to_vec();
    let out = crate::builtin::string::left_bytes(interp, &bytes, size, pad)?;
    Ok(Some(interp.text_built(out)))
}

/// `String~"RIGHT"`, `RexxString::right` (`classes/StringClassSub.cpp:485`).
fn native_string_right(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let (size, pad) = pad_arguments(interp, args)?;
    let bytes = interp.to_text(receiver).to_vec();
    match crate::builtin::string::right_bytes(interp, &bytes, size, pad)? {
        Some(out) => Ok(Some(interp.text_built(out))),
        None => Ok(Some(receiver)),
    }
}

/// `String~"COPIES"`, `RexxString::copies` (`classes/StringClassMisc.cpp:288`).
///
/// Its count is not a length: 93.906 rather than the 93.923 the three above
/// raise, which is why it does not share their argument layer.
fn native_string_copies(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let count = copies_argument(interp, args)?;
    let bytes = interp.to_text(receiver).to_vec();
    match crate::builtin::string::copies_bytes(interp, &bytes, count)? {
        Some(out) => Ok(Some(interp.text_built(out))),
        None => Ok(Some(receiver)),
    }
}

/// `String`'s rows, in the shape [`super::NATIVE_METHODS`] uses.
pub(super) static NATIVE_METHODS: &[(&str, &str, Arity, NativeMethod)] = &[
    ("String", "", Arity::Fixed(1), native_string_op_abuttal),
    ("String", " ", Arity::Fixed(1), native_string_op_blank),
    ("String", "%", Arity::Fixed(1), native_string_op_intdiv),
    ("String", "&", Arity::Fixed(1), native_string_op_and),
    ("String", "&&", Arity::Fixed(1), native_string_op_xor),
    ("String", "*", Arity::Fixed(1), native_string_op_multiply),
    ("String", "**", Arity::Fixed(1), native_string_op_power),
    ("String", "+", Arity::Fixed(1), native_string_op_plus),
    ("String", "-", Arity::Fixed(1), native_string_op_subtract),
    ("String", "/", Arity::Fixed(1), native_string_op_divide),
    ("String", "//", Arity::Fixed(1), native_string_op_remainder),
    ("String", "<", Arity::Fixed(1), native_string_op_less),
    (
        "String",
        "<<",
        Arity::Fixed(1),
        native_string_op_strict_less,
    ),
    (
        "String",
        "<<=",
        Arity::Fixed(1),
        native_string_op_strict_less_equal,
    ),
    ("String", "<=", Arity::Fixed(1), native_string_op_less_equal),
    (
        "String",
        "<>",
        Arity::Fixed(1),
        native_string_op_less_greater,
    ),
    ("String", "=", Arity::Fixed(1), native_string_op_equal),
    (
        "String",
        "==",
        Arity::Fixed(1),
        native_string_op_strict_equal,
    ),
    ("String", ">", Arity::Fixed(1), native_string_op_greater),
    (
        "String",
        "><",
        Arity::Fixed(1),
        native_string_op_greater_less,
    ),
    (
        "String",
        ">=",
        Arity::Fixed(1),
        native_string_op_greater_equal,
    ),
    (
        "String",
        ">>",
        Arity::Fixed(1),
        native_string_op_strict_greater,
    ),
    (
        "String",
        ">>=",
        Arity::Fixed(1),
        native_string_op_strict_greater_equal,
    ),
    ("String", "?", Arity::Fixed(2), native_string_op_choice),
    ("String", "APPEND", Arity::Fixed(1), native_string_append),
    ("String", "CENTER", Arity::Fixed(2), native_string_center),
    ("String", "CENTRE", Arity::Fixed(2), native_string_center),
    ("String", "COPIES", Arity::Fixed(1), native_string_copies),
    ("String", "LEFT", Arity::Fixed(2), native_string_left),
    ("String", "RIGHT", Arity::Fixed(2), native_string_right),
    (
        "String",
        "CASELESSCHANGESTR",
        Arity::Fixed(3),
        native_string_caselesschangestr,
    ),
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
        "CHANGESTR",
        Arity::Fixed(3),
        native_string_changestr,
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
    ("String", "DELSTR", Arity::Fixed(2), native_string_delstr),
    ("String", "DELWORD", Arity::Fixed(2), native_string_delword),
    (
        "String",
        "ENDSWITH",
        Arity::Fixed(1),
        native_string_endswith,
    ),
    ("String", "INSERT", Arity::Fixed(4), native_string_insert),
    ("String", "LASTPOS", Arity::Fixed(3), native_string_lastpos),
    ("String", "LOWER", Arity::Fixed(2), native_string_lower),
    ("String", "MATCH", Arity::Fixed(4), native_string_match),
    (
        "String",
        "MATCHCHAR",
        Arity::Fixed(2),
        native_string_matchchar,
    ),
    ("String", "OVERLAY", Arity::Fixed(4), native_string_overlay),
    ("String", "POS", Arity::Fixed(3), native_string_pos),
    (
        "String",
        "REPLACEAT",
        Arity::Fixed(4),
        native_string_replaceat,
    ),
    ("String", "SPACE", Arity::Fixed(2), native_string_space),
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
    (
        "String",
        "TRANSLATE",
        Arity::Fixed(5),
        native_string_translate,
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
    ("String", "\\", Arity::Fixed(0), native_string_op_not),
    (
        "String",
        "\\<",
        Arity::Fixed(1),
        native_string_op_backslash_less,
    ),
    (
        "String",
        "\\<<",
        Arity::Fixed(1),
        native_string_op_strict_backslash_less,
    ),
    (
        "String",
        "\\=",
        Arity::Fixed(1),
        native_string_op_backslash_equal,
    ),
    (
        "String",
        "\\==",
        Arity::Fixed(1),
        native_string_op_strict_backslash_equal,
    ),
    (
        "String",
        "\\>",
        Arity::Fixed(1),
        native_string_op_backslash_greater,
    ),
    (
        "String",
        "\\>>",
        Arity::Fixed(1),
        native_string_op_strict_backslash_greater,
    ),
    ("String", "|", Arity::Fixed(1), native_string_op_or),
    (
        "String",
        "||",
        Arity::Fixed(1),
        native_string_op_concatenate,
    ),
];

#[cfg(test)]
mod tests {
    use super::{BINARY_OPERATORS, NATIVE_METHODS};

    /// **The operator rows' names are the parser's own spellings**, so the
    /// table cannot name an operator the language does not have, and a
    /// spelling that changed would fail here rather than in a differential.
    ///
    /// It does not check that a row is wired to the *right* operator -- a `+`
    /// bound to `Operator::Subtract` would pass this and fail
    /// `corpus/lang/string_operators.rex` on its first line.
    #[test]
    fn operator_spellings_match_the_parser() {
        for (name, op) in BINARY_OPERATORS {
            assert_eq!(
                op.spelling(),
                *name,
                "the table spells {op:?} as {name:?} where the parser spells it {:?}",
                op.spelling()
            );
        }
    }

    /// Every operator [`BINARY_OPERATORS`] names has a row, so the two cannot
    /// come apart by an operator being listed and never registered.
    #[test]
    fn every_listed_operator_has_a_row() {
        for (name, _) in BINARY_OPERATORS {
            assert!(
                NATIVE_METHODS.iter().any(|(_, method, ..)| method == name),
                "{name:?} is listed as an operator and has no row"
            );
        }
    }
}
