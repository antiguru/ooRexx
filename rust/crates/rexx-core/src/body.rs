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

use crate::ObjRef;

/// Identifies the behaviour (class + method dictionary) an object responds to.
///
/// Behaviours themselves live in a side table, not in the heap, because they
/// are created during bootstrap and never collected.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct BehaviourId(pub u16);

impl BehaviourId {
    pub const STRING: BehaviourId = BehaviourId(0);
    pub const ARRAY: BehaviourId = BehaviourId(1);
    pub const OBJECT: BehaviourId = BehaviourId(2);
}

/// The payload of a heap object.
///
/// Every variant that can reach another object must be handled in
/// `Body::trace`. Adding a variant without extending `trace` is the one way
/// to reintroduce the C++ implementation's defect class, so `trace` matches
/// exhaustively and must never gain a `_ =>` arm.
#[derive(Clone, Debug)]
pub enum Body {
    String(String),
    Array(Vec<ObjRef>),
    /// A user-defined object: its instance variables.
    Instance(Vec<(String, ObjRef)>),
    /// A reference that does not keep its target alive. Traces to nothing --
    /// that is the whole point -- and the collector rewrites the target to
    /// `ObjRef::NIL` once it dies.
    WeakRef(ObjRef),
}

impl Body {
    /// Appends every object this one can reach.
    ///
    /// This single exhaustive match replaces the 148 hand-written `live()`
    /// implementations in the C++ tree. It has no wildcard arm on purpose:
    /// adding a `Body` variant must be a compile error here, not a runtime
    /// use-after-free.
    pub fn trace(&self, out: &mut Vec<ObjRef>) {
        match self {
            Body::String(_) => {}
            Body::Array(items) => out.extend_from_slice(items),
            Body::Instance(vars) => out.extend(vars.iter().map(|(_, v)| *v)),
            // Deliberately reaches nothing: a weak reference must not keep
            // its target alive.
            Body::WeakRef(_) => {}
        }
    }
}

#[derive(Clone, Debug)]
pub struct Object {
    pub behaviour: BehaviourId,
    pub body: Body,
    /// Set when the object defines an `UNINIT` method. Such an object is
    /// resurrected by the collector and reported through
    /// `CollectStats::pending_uninit` rather than swept, and is cleared once
    /// the caller reports the finalizer has run.
    pub has_uninit: bool,
}
