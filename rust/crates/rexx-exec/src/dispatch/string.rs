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
    backward_search_arguments, forward_search, forward_search_arguments,
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
        "CONTAINS",
        Arity::Fixed(3),
        native_string_contains,
    ),
    ("String", "LASTPOS", Arity::Fixed(3), native_string_lastpos),
    ("String", "POS", Arity::Fixed(3), native_string_pos),
];
