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
//!
//! `registry.rs` adds `ClassRegistry`: class objects (an [`rexx_core::ObjRef`]
//! identity plus an id string, over `ClassGraph`'s own behaviours/superclass/
//! subclass lists) and the name-to-class registry. `native_classes.rs`
//! bootstraps the primitive class set `interpreter/memory/Setup.cpp` builds
//! before `CoreClasses.orx` ever runs, from `build.rs`'s derived per-class
//! method tables, and records the deferral table for every derived class
//! name this crate does not build.

mod class_graph;
mod method_dict;
mod native_classes;
mod registry;

pub use class_graph::{BehaviourHandle, ClassGraph, ClassKind};
pub use method_dict::{MethodDict, MethodId};
pub use native_classes::{Deferral, deferred_classes, native_classes, setup_class_names};
pub use registry::ClassRegistry;
