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

use super::*;

pub(super) fn take(
    interp: &mut Interp,
    receiver: ObjRef,
    index: ObjRef,
) -> Result<Option<ObjRef>, Failure> {
    take_in(interp, receiver, CONTENTS, index)
}

/// [`take`] from either half of a `Directory`.
pub(super) fn take_in(
    interp: &mut Interp,
    receiver: ObjRef,
    half: Half,
    index: ObjRef,
) -> Result<Option<ObjRef>, Failure> {
    interp.bump_route_generation();
    let Some(store) = read_store(interp, receiver, half)? else {
        return Ok(None);
    };
    let (store, found) = probe_in(interp, receiver, half, store, index)?;
    let Some(slot) = found.found else {
        return Ok(None);
    };
    let previous = found.last.filter(|last| *last != slot);
    remove_at(interp, receiver, half, &store, slot, previous)
}

/// [`take`] for an entry the caller has already found, named by its slot
/// rather than by its index -- what `removeItem` needs, since two entries
/// under one index differ only by their item.
pub(super) fn take_at(
    interp: &mut Interp,
    receiver: ObjRef,
    store: &Store,
    slot: usize,
    previous: Option<usize>,
) -> Result<Option<ObjRef>, Failure> {
    interp.bump_route_generation();
    remove_at(interp, receiver, CONTENTS, store, slot, previous)
}

/// Unlinks the entry at `slot`, whose chain predecessor is `previous`, and
/// answers the item it held.
pub(super) fn remove_at(
    interp: &mut Interp,
    receiver: ObjRef,
    half: Half,
    store: &Store,
    slot: usize,
    previous: Option<usize>,
) -> Result<Option<ObjRef>, Failure> {
    let item = slot_at(interp, store.items, slot)?;
    let next = link_at(interp, store, slot)?;
    if slot < store.buckets {
        // A bucket slot cannot be freed, so the chain is closed by copying
        // the next entry into it -- `HashContents::closeChain`.
        if next < store.total {
            let moved_index = slot_at(interp, store.indexes, next)?;
            let moved_item = slot_at(interp, store.items, next)?;
            let moved_next = link_at(interp, store, next)?;
            write_slot(interp, store.indexes, slot, moved_index);
            write_slot(interp, store.items, slot, moved_item);
            write_link(interp, store, slot, moved_next);
            free_slot(interp, receiver, half, store, next);
        } else {
            write_slot(interp, store.indexes, slot, None);
            write_slot(interp, store.items, slot, None);
            write_link(interp, store, slot, store.no_more());
        }
        return Ok(item);
    }
    if let Some(previous) = previous {
        write_link(interp, store, previous, next);
    }
    free_slot(interp, receiver, half, store, slot);
    Ok(item)
}

fn free_slot(interp: &mut Interp, receiver: ObjRef, half: Half, store: &Store, slot: usize) {
    write_slot(interp, store.indexes, slot, None);
    write_slot(interp, store.items, slot, None);
    let free = store.free;
    write_link(interp, store, slot, free);
    set_free(interp, receiver, half, slot);
}

/// Whether a removal refuses an owed entry before taking it: `Answered` for
/// one that hands the item to the program unread, `Discarded` for one that
/// drops it or whose caller has already read it through [`item_at`].
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Removal {
    Answered,
    Discarded,
}

/// `DirectoryClass::remove`: answers what `get` would -- which may run a
/// method, or the unknown method -- and then drops the name from both halves.
///
/// An [`Removal::Answered`] removal refuses an owed entry; a discarded one
/// takes it without reading it.
pub(super) fn take_merged(
    interp: &mut Interp,
    receiver: ObjRef,
    index: ObjRef,
    removal: Removal,
) -> Result<Option<ObjRef>, Failure> {
    if method_store(interp, receiver)?.is_none() && unknown_method(interp, receiver).is_none() {
        interp.bump_route_generation();
        let (store, found) = probe(interp, receiver, index)?;
        let Some(slot) = found.found else {
            return Ok(None);
        };
        if removal == Removal::Answered {
            item_at(interp, &store, slot)?;
        }
        let previous = found.last.filter(|last| *last != slot);
        return remove_at(interp, receiver, CONTENTS, &store, slot, previous);
    }
    // `get` runs a method only where the contents miss, so a contents entry
    // that is discarded needs no read.
    let old = if removal == Removal::Discarded && probe(interp, receiver, index)?.1.found.is_some()
    {
        None
    } else {
        merged_get(interp, receiver, index)?
    };
    take(interp, receiver, index)?;
    take_in(interp, receiver, METHODS, index)?;
    Ok(old)
}

pub(super) fn native_hash_remove_item(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    note_routines_write(interp, receiver);
    if !owns(interp, receiver) {
        return Err(not_this_task(interp, receiver, b"REMOVEITEM"));
    }
    let wanted = super::collection::item_argument(args)?;
    let keys = keys_of(interp, receiver);
    if method_store(interp, receiver)?.is_none() && unknown_method(interp, receiver).is_none() {
        let store = store_of(interp, receiver)?;
        for (slot, previous) in entries_in(interp, &store, None)? {
            let Some(item) = item_at(interp, &store, slot)? else {
                continue;
            };
            if same_held_item(interp, keys, wanted, item)? {
                take_at(interp, receiver, &store, slot, previous)?;
                return Ok(Some(item));
            }
        }
        return Ok(Some(ObjRef::NIL));
    }
    for (index, item) in pairs(interp, receiver)? {
        if same_held_item(interp, keys, wanted, item)? {
            take_merged(interp, receiver, index, Removal::Discarded)?;
            return Ok(Some(item));
        }
    }
    Ok(Some(ObjRef::NIL))
}
