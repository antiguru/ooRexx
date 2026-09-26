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

use std::collections::HashMap;

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
    entries: HashMap<usize, ObjRef>,
}

impl Table {
    pub fn new() -> Table {
        Table::default()
    }

    /// The handle for `object`, valid until [`Table::remove`] or
    /// [`Table::clear`]. Registering an object this table already holds
    /// answers the same handle and changes nothing.
    pub fn register(&mut self, object: ObjRef) -> RexxObjectPtr {
        let address = address_of(object);
        self.entries.insert(address, object);
        std::ptr::without_provenance_mut(address)
    }

    /// The object `handle` names, or `None` if this table does not hold it.
    pub fn resolve(&self, handle: RexxObjectPtr) -> Option<ObjRef> {
        self.entries.get(&handle.addr()).copied()
    }

    /// Whether this table holds `object`.
    pub fn holds(&self, object: ObjRef) -> bool {
        self.entries.contains_key(&address_of(object))
    }

    /// Drops `handle`'s registration. A handle this table never held is
    /// ignored, as `NativeActivation::removeLocalReference` ignores one.
    pub fn remove(&mut self, handle: RexxObjectPtr) {
        self.entries.remove(&handle.addr());
    }

    /// Drops every registration, which is what the end of an activation does.
    pub fn clear(&mut self) {
        self.entries.clear();
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
