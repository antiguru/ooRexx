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

//! The ooRexx class/method-dictionary model.
//!
//! `MethodDict` is a flat name-to-method map plus scope ordering, built at
//! class-definition time -- the oracle's `MethodDictionary`
//! (`interpreter/behaviour/MethodDictionary.cpp`). `ClassGraph` is the
//! oracle's `RexxClass` cascade: `define`, `inherit` and
//! `inherit_instance_methods` (`interpreter/classes/ClassClass.cpp:558-1172`),
//! replacing `rexx-core::BehaviourTable`'s lookup-time superclass walk for
//! user-defined classes (D29, `docs/superpowers/specs/2026-08-15-phase-5-object-model.md`).
//!
//! `rexx-core::BehaviourTable` is untouched by this crate and still serves
//! the primitive fast path (`BehaviourId`); nothing here is wired to it or
//! to `rexx-core::Object`. Message dispatch (`resolve`/`invoke`) and every
//! other piece of interpreter wiring are later tasks -- this crate is the
//! data structure and its own unit tests, nothing else.

mod class_graph;
mod method_dict;

pub use class_graph::{BehaviourHandle, ClassGraph, ClassKind};
pub use method_dict::{MethodDict, MethodId};
