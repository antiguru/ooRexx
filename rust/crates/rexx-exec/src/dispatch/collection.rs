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

//! The collection classes' primitive methods.
//!
//! A child of [`super`] rather than a sibling, for the reason its `mod native`
//! comment gives: the rows point at `NativeMethod`s, whose parameter list
//! names types only that module can. [`NATIVE_METHODS`] is chained into
//! `ObjectModel::build` beside [`super::NATIVE_METHODS`] rather than merged,
//! so this file's rows and bodies stay together.
//!
//! **Empty on purpose at Phase 5g Task 0.** The registration path is landed
//! before any body so that the first task to add one is debugging its own
//! code and not the wiring; a chain that silently registers nothing looks
//! exactly like a chain that works.
//!
//! # What a row here may not be read as
//!
//! **The name a row binds is a citation, not a body identity.** Upstream,
//! `Setup.cpp` writes `Set`'s `HasItem` as `IdentityTable::hasIndexRexx` and
//! `IdentityTableClass.hpp` declares no such member -- it is
//! `HashCollection::hasIndexRexx` reached through C++ inheritance. So two
//! tokens there can name one function, and separately one token can reach
//! three behaviours, selected by the contents class the receiver allocated:
//! `HashCollection::putRexx` passes `IndexOnlyHashCollection`'s validation on
//! a `Set` or `Bag` and reaches `MultiValueContents::put` on a `Relation` or
//! `Bag`. A Rust body shared between two rows here is a claim that the
//! *behaviour* is shared, which the upstream token does not establish either
//! way.
//!
//! `corpus/collection-scopes.tsv` is where that join is recorded, and its own
//! test re-derives it.

use super::{Arity, NativeMethod};

/// The collection classes' primitive methods, chained into
/// `ObjectModel::build`.
///
/// Each row's method name is looked up in the class's own dictionary and the
/// build panics if it is not there, so a name a class's behaviour does not
/// answer cannot be added quietly.
pub(super) const NATIVE_METHODS: &[(&str, &str, Arity, NativeMethod)] = &[];
