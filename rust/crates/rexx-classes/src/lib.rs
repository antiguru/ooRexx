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

mod class_graph;
mod method_dict;
mod native_classes;
mod registry;

pub use class_graph::{ClassGraph, ClassKind, InheritRefusal};
pub use method_dict::{MethodDict, MethodSlot};
pub use native_classes::{
    Deferral, deferred_classes, native_classes, native_classes_for_bootstrap, remove_setup_methods,
    setup_class_names, setup_method_names,
};
pub use registry::ClassRegistry;
pub use rexx_core::{BehaviourHandle, MethodId};
