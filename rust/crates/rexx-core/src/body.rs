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
use rexx_num::{Form, Number};
use std::collections::HashMap;

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
    pub const STEM: BehaviourId = BehaviourId(3);
}

/// A byte string that failed `Number::parse`, or whose bytes are not even
/// UTF-8 -- both collapse into this one marker (D15), and that is not a
/// simplification paid for later: nothing observable distinguishes the two
/// causes. A Rexx program that uses a non-numeric value in arithmetic gets
/// error 41.1, "Nonnumeric value ("val") used in arithmetic operation",
/// which substitutes the *value* and never says why it failed to parse. The
/// distinction is also about to get thinner still: once `rexx-num` gains a
/// byte-slice parse entry point (a later task), there is no separate
/// `from_utf8` step left to fail on its own -- a parse over bytes either
/// yields a number or does not.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct NotNumeric;

/// The payload of a heap object.
///
/// Every variant that can reach another object must be handled in
/// `Body::trace`. Adding a variant without extending `trace` is the one way
/// to reintroduce the C++ implementation's defect class, so `trace` matches
/// exhaustively and must never gain a `_ =>` arm.
#[derive(Clone, Debug)]
pub enum Body {
    /// A value whose identity is its bytes (D15). `num` is a tri-state cache
    /// of the one parse those bytes ever get: `None` is "not yet asked", and
    /// must keep meaning exactly that -- treating it as "definitely not a
    /// number" after some other value has been filled in answers wrongly for
    /// a value nobody has asked about yet. The two `Result` arms tell "is a
    /// number" from "is not" apart so a non-numeric string is not re-parsed
    /// on every comparison.
    ///
    /// The cache holds the exact parse and is never rounded to fit a later
    /// `DIGITS`. Rounding belongs to the operation reading it, which is what
    /// keeps the cache safe across a `NUMERIC` change. Measured: `x =
    /// '1.234567890123456789'` gives `1.2346` under `DIGITS 5` and the full
    /// nineteen-digit value under `DIGITS 20`, both read from the one stored
    /// parse. An implementation that "helpfully" rounds at fill time creates
    /// exactly the staleness this tri-state exists to avoid.
    Text {
        bytes: Vec<u8>,
        num: Option<Result<Box<Number>, NotNumeric>>,
    },
    /// A value whose identity is its number (D15). `created_digits` and
    /// `created_form` are the `NUMERIC DIGITS`/`NUMERIC FORM` in force when
    /// this value was produced, and `text` -- formatted under exactly those,
    /// never the settings in force when it is later read -- is fixed for the
    /// object's whole lifetime once filled in.
    Num {
        value: Number,
        created_digits: u32,
        created_form: Form,
        text: Option<Vec<u8>>,
    },
    /// A stem: its own name (rendered when the stem itself is read as a
    /// value, e.g. an alias to `q.` still prints `Q.`), an optional default,
    /// and its tails keyed by the tail's value verbatim (D15a). A tail entry
    /// present and mapped to `None` is a tombstone -- an explicitly dropped
    /// tail, which must not fall back to the default -- and is distinct from
    /// the key being absent, which does.
    Stem {
        name: Box<[u8]>,
        default: Option<ObjRef>,
        tails: HashMap<Vec<u8>, Option<ObjRef>>,
    },
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
            // Neither reaches an `ObjRef`: a `Number` and a byte string are
            // both plain data, never a handle into the heap.
            Body::Text { .. } => {}
            Body::Num { .. } => {}
            Body::Stem { default, tails, .. } => {
                out.extend(default.iter().copied());
                // A tombstone (`None`) reaches nothing, same as a weak
                // reference clearing to `.nil` -- it is present but dead.
                out.extend(tails.values().filter_map(|t| *t));
            }
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
