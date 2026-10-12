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

//! The local-reference table: the `ObjRef` behind an extension's handle.

use rustc_hash::FxHashMap;

use rexx_core::ObjRef;

use crate::layout::RexxObjectPtr;

// A handle carries an `ObjRef`'s bits in the address of a pointer nothing
// dereferences, so the whole width has to fit.
const _: () = assert!(usize::BITS == u64::BITS);

/// The objects one activation has handed a native call, held so the collector
/// can see them (D5).
///
/// A handle is derived from the whole of the `ObjRef`, generation included, so
/// two handles are equal exactly when they name the same object and a handle
/// minted before a slot was recycled cannot name the slot's next occupant.
#[derive(Default)]
pub struct Table {
    entries: FxHashMap<usize, ObjRef>,
    /// Handles [`Table::release`] dropped, which root nothing and resolve
    /// while their object lives.
    released: FxHashMap<usize, ObjRef>,
    /// The size of `released` at which [`Table::release`] next drops the
    /// handles whose object is gone.
    released_limit: usize,
}

/// The least `released_limit`.
const RELEASED_FLOOR: usize = 4096;

impl Table {
    pub fn new() -> Table {
        Table::default()
    }

    /// The handle for `object`, held until [`Table::release`] or
    /// [`Table::clear`]. Registering an object this table already holds
    /// answers the same handle and changes nothing.
    pub fn register(&mut self, object: ObjRef) -> RexxObjectPtr {
        let address = address_of(object);
        self.entries.insert(address, object);
        self.released.remove(&address);
        std::ptr::without_provenance_mut(address)
    }

    /// The object `handle` names, held or released, or `None` if this table
    /// has neither.
    pub fn resolve(&self, handle: RexxObjectPtr) -> Option<ObjRef> {
        self.resolve_live(handle, |_| true)
    }

    /// [`Table::resolve`], a released handle answered only where `live`
    /// says its object is.
    pub fn resolve_live(
        &self,
        handle: RexxObjectPtr,
        live: impl Fn(ObjRef) -> bool,
    ) -> Option<ObjRef> {
        let address = handle.addr();
        self.entries.get(&address).copied().or_else(|| {
            self.released
                .get(&address)
                .copied()
                .filter(|object| live(*object))
        })
    }

    /// Whether this table holds `object`.
    pub fn holds(&self, object: ObjRef) -> bool {
        self.entries.contains_key(&address_of(object))
    }

    /// Drops `handle`'s registration, so it no longer roots its object
    /// (`NativeActivation::removeLocalReference`,
    /// `interpreter/execution/NativeActivation.cpp:1219`). A handle this table
    /// does not hold is ignored, as the oracle ignores one. Each time the
    /// released handles pass their limit, those whose object `live` says is
    /// gone are forgotten.
    pub fn release(&mut self, handle: RexxObjectPtr, live: impl Fn(ObjRef) -> bool) {
        let address = handle.addr();
        let Some(object) = self.entries.remove(&address) else {
            return;
        };
        self.released.insert(address, object);
        if self.released.len() >= self.released_limit.max(RELEASED_FLOOR) {
            self.released.retain(|_, object| live(*object));
            self.released_limit = self.released.len() * 2;
        }
    }

    /// Drops every registration and every released handle, which is what
    /// the end of an activation does.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.released.clear();
        self.released_limit = 0;
    }

    /// Every object this table holds, for the collector.
    pub fn roots(&self) -> impl Iterator<Item = ObjRef> + '_ {
        self.entries.values().copied()
    }
}

/// `object`'s handle, as an address.
///
/// # Panics
/// If every bit of `object` is set, which no constructible `ObjRef` has: the
/// tag reserves the low two bits, and the only tag that sets both leaves the
/// three above the length clear.
fn address_of(object: ObjRef) -> usize {
    // `NULLOBJECT` is the null pointer (`api/rexx.h:155`), so the `ObjRef`
    // whose bits are zero -- heap slot zero at generation zero, the first
    // object a fresh heap allocates -- must not encode to it.
    let biased = object.bits().wrapping_add(1);
    assert!(biased != 0, "an ObjRef with every bit set has no handle");
    biased as usize
}
