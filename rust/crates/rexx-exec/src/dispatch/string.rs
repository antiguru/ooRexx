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
    backward_search_arguments, forward_search, forward_search_arguments, wordpos_arguments,
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
        "CASELESSLASTPOS",
        Arity::Fixed(3),
        native_string_caselesslastpos,
    ),
    (
        "String",
        "CASELESSPOS",
        Arity::Fixed(3),
        native_string_caselesspos,
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
    ("String", "LASTPOS", Arity::Fixed(3), native_string_lastpos),
    ("String", "POS", Arity::Fixed(3), native_string_pos),
    ("String", "WORDPOS", Arity::Fixed(2), native_string_wordpos),
];
