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
mod bytes;
mod handle;
mod heap;
mod roots;

pub use behaviour::{BehaviourTable, MethodId};
pub use body::{BehaviourId, Body, NativeObject, NotNumeric, Object, ScopePools};
pub use bytes::{Bytes, INLINE_BYTES};
pub use handle::{
    CLASS_SLOT_BASE, Decoded, GENERATION_MAX, INLINE_TEXT, InlineText, ObjRef, SMALL_INT_MAX,
    SMALL_INT_MIN, is_class_slot,
};
pub use heap::{CollectStats, Heap};
pub use roots::{FrameId, RootSet, SlotFrame, SlotRef};

/// The hasher behind [`NameMap`].
///
/// **`RandomState`'s resistance to chosen collisions buys nothing here and is
/// not free.** It is `SipHash-1-3` with a per-process random key, which the
/// standard library picks because a `HashMap` may hold keys an attacker
/// supplies over a network. These two tables hold names out of a Rexx
/// program, which the interpreter is executing anyway -- a program able to
/// choose its own variable names can burn a machine's time far more directly
/// than by colliding them.
///
/// The keys are also short. A variable name or a compound tail is a handful
/// of bytes, which is the length at which SipHash's setup and finalisation
/// dominate and never amortise.
///
/// **`FxHash` because it was measured against the alternatives, not because
/// it is the usual answer.** Instructions retired on the pinned-loop-count
/// `rexxcps`, against the `RandomState` this replaced: `seahash` 16176756724
/// (+0.22%, *worse* -- it is built for throughput over long buffers and these
/// keys are a handful of bytes), `foldhash` 15781449142 and this 15784235642,
/// both -2.2%, against a baseline of 16141358350. `foldhash`'s remaining
/// 0.018% over this does not pay for its thirteen `unsafe` blocks where
/// `rustc-hash` has none, which is the tie-breaker this repository's own rule
/// on `unsafe` asks for.
pub type NameHasher = rustc_hash::FxBuildHasher;

/// A map keyed by a name out of the program text.
///
/// **Fixed-seed, so iteration order is now stable from run to run** where it
/// used to vary. Nothing observable depended on the old variation -- the one
/// place that walks [`Body::Stem`]'s tails is the collector, which wants a
/// set, and `Plan`'s slot numbering deliberately recovers walk order from the
/// numbering rather than from the map for exactly this reason -- so this is a
/// property gained rather than one traded away.
pub type NameMap<K, V> = std::collections::HashMap<K, V, NameHasher>;
