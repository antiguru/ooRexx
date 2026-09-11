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

//! The Rexx object model: handles, the arena heap, tracing, and behaviours.

mod behaviour;
mod body;
// **The one module in this workspace granted an `unsafe` exception** (Moritz,
// 2026-08-20). The workspace lint is `deny` rather than `forbid` so that this
// line can exist at all -- the root `Cargo.toml` has why, and why this
// attribute is now the record of the grant rather than a lint level somewhere.
// `bytes.rs`'s own `Bytes` doc states the invariant, names the only writer and
// the only reader, and gives the measurement that bought it;
// `tests/unsafe_sites.rs` is what stops a second site appearing quietly.
#[allow(unsafe_code)]
mod bytes;
mod handle;
mod heap;
mod roots;

pub use behaviour::BehaviourTable;
pub use body::{
    BehaviourHandle, BehaviourId, Body, BufferState, MethodId, NativeObject, NotNumeric, Object,
    ObjectMethod, ObjectMethods, ScopePools, VarRef, VarRefHome,
};
pub use bytes::{Bytes, INLINE_BYTES};
pub use handle::{
    Decoded, GENERATION_MAX, INLINE_TEXT, InlineText, ObjRef, SMALL_INT_MAX, SMALL_INT_MIN,
};
pub use heap::{CollectStats, Heap};
pub use roots::{FrameAliases, FrameId, Parked, RootSet, SlotFrame, SlotRef};

/// The hasher behind [`NameMap`].
pub type NameHasher = rustc_hash::FxBuildHasher;

/// A map keyed by a name out of the program text.
pub type NameMap<K, V> = std::collections::HashMap<K, V, NameHasher>;
