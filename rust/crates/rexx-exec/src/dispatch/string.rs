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

/// `String`'s rows, in the shape [`super::NATIVE_METHODS`] uses.
pub(super) static NATIVE_METHODS: &[(&str, &str, super::Arity, super::NativeMethod)] = &[];
