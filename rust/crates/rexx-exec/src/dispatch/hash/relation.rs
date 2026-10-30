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

//! `Relation`'s and `Bag`'s own primitive methods, over the store their parent
//! module keeps.

use super::{
    Arity, Cleared, Failure, Interp, Keys, NativeMethod, ObjRef, keys_of, native_hash_supplier,
    not_this_task, owns, pairs, same_index, slot_at, store_of, take, walk,
};

/// Every matching slot in chain order, paired with the store it belongs to.
fn slots_for(
    interp: &mut Interp,
    receiver: ObjRef,
    index: ObjRef,
) -> Result<(super::Store, Vec<usize>), Failure> {
    let keys = keys_of(interp, receiver);
    let hash = super::hash_of(interp, keys, index)?;
    'search: loop {
        let store = store_of(interp, receiver)?;
        // Keep the saved array identities live across replacement callbacks.
        for array in [store.indexes, store.items, store.next] {
            interp.roots.activity_mut().push_temp(array);
        }
        let bucket = (hash % store.buckets as u64) as usize;
        let entries = super::entries_in(interp, &store, Some(bucket))?;
        let mut indexes = Vec::with_capacity(entries.len());
        for &(slot, _) in &entries {
            let held = slot_at(interp, store.indexes, slot)?;
            if let Some(held) = held {
                interp.roots.activity_mut().push_temp(held);
            }
            indexes.push(held);
        }
        let mut found = Vec::new();
        for &(slot, _) in &entries {
            let writes = interp.store_writes;
            let generation = interp.store_generation;
            let matches = same_index(interp, &store, keys, slot, index)?;
            // Equality can replace arrays or change an earlier collision
            // entry. Validate the whole saved chain before retaining slots.
            if writes != interp.store_writes || generation != interp.store_generation {
                let current = store_of(interp, receiver)?;
                if current != store || super::entries_in(interp, &current, Some(bucket))? != entries
                {
                    continue 'search;
                }
                for (&(saved, _), &held) in entries.iter().zip(&indexes) {
                    if slot_at(interp, current.indexes, saved)? != held {
                        continue 'search;
                    }
                }
            }
            if matches {
                found.push(slot);
            }
        }
        return Ok((store, found));
    }
}

/// `RelationClass::allAt(index)`: every item under that index, newest first.
fn native_relation_all_at(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if !owns(interp, receiver) {
        return Err(not_this_task(interp, receiver, b"ALLAT"));
    }
    // **93.903 and not 88.901.** `RelationClass`'s own methods take their
    // index positionally where `HashCollection`'s take it as a named
    // argument: measured on a `.Relation~new`, `allAt()` and `removeAll()`
    // report `Missing argument in method; argument 1 is required.` where
    // `at()` and `remove()` report `Missing argument; argument index is
    // required.`
    let index = super::collection::item_argument(args)?;
    let (store, slots) = slots_for(interp, receiver, index)?;
    let mut items = Vec::new();
    for slot in slots {
        items.push(slot_at(interp, store.items, slot)?.unwrap_or(ObjRef::NIL));
    }
    Ok(Some(super::collection::array_of(interp, items)))
}

/// `RelationClass::allIndexRexx(item)`: every index whose item matches.
fn native_relation_all_index(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if !owns(interp, receiver) {
        return Err(not_this_task(interp, receiver, b"ALLINDEX"));
    }
    let wanted = super::collection::item_argument(args)?;
    let mut indexes = Vec::new();
    for (index, item) in pairs(interp, receiver)? {
        if super::collection::same_item(interp, wanted, item)? {
            indexes.push(index);
        }
    }
    Ok(Some(super::collection::array_of(interp, indexes)))
}

/// `RelationClass::uniqueIndexes`: each index once, in first-seen order.
fn native_relation_unique_indexes(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if !owns(interp, receiver) {
        return Err(not_this_task(interp, receiver, b"UNIQUEINDEXES"));
    }
    let keys = keys_of(interp, receiver);
    let mut unique: Vec<ObjRef> = Vec::new();
    for (index, _) in pairs(interp, receiver)? {
        let mut seen = false;
        for held in &unique {
            let same = match keys {
                Keys::Identity => *held == index,
                Keys::Equality => super::collection::same_item(interp, index, *held)?,
                Keys::StringValue => {
                    let wanted = interp.to_text(index).into_owned();
                    let found = interp.to_text(*held).into_owned();
                    wanted == found
                }
            };
            if same {
                seen = true;
                break;
            }
        }
        if !seen {
            unique.push(index);
        }
    }
    Ok(Some(super::collection::array_of(interp, unique)))
}

/// `RelationClass::itemsRexx([index])`: the whole count, or the count under
/// one index. Measured, a relation holding `k -> v1`, `k -> v2` and `j -> w`
/// answers `items` 3, `items('k')` 2 and `items('z')` 0.
fn native_relation_items(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if !owns(interp, receiver) {
        return Err(not_this_task(interp, receiver, b"ITEMS"));
    }
    let count = match args.first().copied().flatten() {
        Some(index) => slots_for(interp, receiver, index)?.1.len(),
        None => walk(interp, receiver)?.len(),
    };
    Ok(Some(interp.counted(count)))
}

/// `RelationClass::removeAll(index)`: takes every entry under that index out
/// and answers them as an array, newest first -- an empty `Array` for an
/// index the relation does not hold.
fn native_relation_remove_all(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if !owns(interp, receiver) {
        return Err(not_this_task(interp, receiver, b"REMOVEALL"));
    }
    // **93.903 and not 88.901.** `RelationClass`'s own methods take their
    // index positionally where `HashCollection`'s take it as a named
    // argument: measured on a `.Relation~new`, `allAt()` and `removeAll()`
    // report `Missing argument in method; argument 1 is required.` where
    // `at()` and `remove()` report `Missing argument; argument index is
    // required.`
    let index = super::collection::item_argument(args)?;
    let mut removed = Vec::new();
    while let Some(item) = take(interp, receiver, index)? {
        interp.roots.activity_mut().push_temp(item);
        removed.push(item);
    }
    Ok(Some(super::collection::array_of(interp, removed)))
}

/// `RelationClass::supplierRexx([index])`: a supplier over the whole
/// relation, or over one index's items.
fn native_relation_supplier(
    interp: &mut Interp,
    cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if !owns(interp, receiver) {
        return Err(not_this_task(interp, receiver, b"SUPPLIER"));
    }
    let Some(index) = args.first().copied().flatten() else {
        return native_hash_supplier(interp, cleared, receiver, &[]);
    };
    let (store, slots) = slots_for(interp, receiver, index)?;
    let mut items = Vec::new();
    let mut indexes = Vec::new();
    for slot in slots {
        items.push(slot_at(interp, store.items, slot)?.unwrap_or(ObjRef::NIL));
        indexes.push(slot_at(interp, store.indexes, slot)?.unwrap_or(ObjRef::NIL));
    }
    let items = super::collection::array_of(interp, items);
    let indexes = super::collection::array_of(interp, indexes);
    super::collection::new_supplier(interp, items, indexes).map(Some)
}

/// `RelationClass::hasItemRexx(item [, index])`: whether the relation holds
/// that item, optionally under that index. Measured, `hasItem('v1','k')` is
/// 1 and `hasItem('v1','j')` is 0 for a relation holding `k -> v1`.
fn native_relation_has_item(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if !owns(interp, receiver) {
        return Err(not_this_task(interp, receiver, b"HASITEM"));
    }
    let wanted = super::collection::item_argument(args)?;
    let under = args
        .get(1)
        .copied()
        .flatten()
        .or_else(|| super::index_only(interp, receiver).then_some(wanted));
    let keys = keys_of(interp, receiver);
    let hash = under
        .map(|index| super::hash_of(interp, keys, index))
        .transpose()?;
    let store = store_of(interp, receiver)?;
    let bucket = hash.map(|hash| (hash % store.buckets as u64) as usize);
    for (slot, _) in super::entries_in(interp, &store, bucket)? {
        if let Some(index) = under
            && !same_index(interp, &store, keys, slot, index)?
        {
            continue;
        }
        let Some(item) = slot_at(interp, store.items, slot)? else {
            continue;
        };
        if !super::collection::same_item(interp, wanted, item)? {
            continue;
        }
        return Ok(Some(crate::eval::logical(true)));
    }
    Ok(Some(crate::eval::logical(false)))
}

/// `RelationClass::removeItemRexx(item [, index])`: takes out the first entry
/// holding that item, optionally under that index, and answers it.
fn native_relation_remove_item(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if !owns(interp, receiver) {
        return Err(not_this_task(interp, receiver, b"REMOVEITEM"));
    }
    let wanted = super::collection::item_argument(args)?;
    let under = args
        .get(1)
        .copied()
        .flatten()
        .or_else(|| super::index_only(interp, receiver).then_some(wanted));
    let keys = keys_of(interp, receiver);
    let hash = under
        .map(|index| super::hash_of(interp, keys, index))
        .transpose()?;
    'search: loop {
        let store = store_of(interp, receiver)?;
        // Keep the saved array identities live across replacement callbacks.
        for array in [store.indexes, store.items, store.next] {
            interp.roots.activity_mut().push_temp(array);
        }
        let selected = hash.map(|hash| (hash % store.buckets as u64) as usize);
        let mut bucket = selected.unwrap_or(0);
        for (slot, previous) in super::entries_in(interp, &store, selected)? {
            if previous.is_none() {
                bucket = slot;
            }
            if let Some(index) = under {
                let matches = same_index(interp, &store, keys, slot, index)?;
                if store_of(interp, receiver)? != store {
                    continue 'search;
                }
                if !matches {
                    continue;
                }
            }
            let Some(item) = slot_at(interp, store.items, slot)? else {
                continue;
            };
            if super::collection::same_item(interp, wanted, item)? {
                if let Some(removed) = super::take_at(interp, receiver, &store, slot, bucket)? {
                    return Ok(Some(removed));
                }
                continue 'search;
            }
            if store_of(interp, receiver)? != store {
                continue 'search;
            }
        }
        return Ok(Some(ObjRef::NIL));
    }
}

/// `Relation`'s and `Bag`'s primitive methods, chained into
/// `ObjectModel::build` beside [`super::NATIVE_METHODS`].
pub(in crate::dispatch) const NATIVE_METHODS: &[(&str, &str, Arity, NativeMethod)] = &[
    // **`Relation` and `Bag` have identical native entry-point sets**, which
    // is why they are one task: `Setup.cpp` writes `Bag`'s `AllAt`,
    // `AllIndex`, `Items`, `RemoveAll`, `Supplier` and `UniqueIndexes` as
    // `RelationClass::` bodies, and only `HasItem` and `RemoveItem` are
    // `BagClass::` -- and those two answer the same thing here, because a
    // `Bag`'s index is its item.
    ("Relation", "ALLAT", Arity::Fixed(1), native_relation_all_at),
    (
        "Relation",
        "ALLINDEX",
        Arity::Fixed(1),
        native_relation_all_index,
    ),
    (
        "Relation",
        "UNIQUEINDEXES",
        Arity::Fixed(0),
        native_relation_unique_indexes,
    ),
    ("Relation", "ITEMS", Arity::Fixed(1), native_relation_items),
    (
        "Relation",
        "REMOVEALL",
        Arity::Fixed(1),
        native_relation_remove_all,
    ),
    (
        "Relation",
        "SUPPLIER",
        Arity::Fixed(1),
        native_relation_supplier,
    ),
    (
        "Relation",
        "HASITEM",
        Arity::Fixed(2),
        native_relation_has_item,
    ),
    (
        "Relation",
        "REMOVEITEM",
        Arity::Fixed(2),
        native_relation_remove_item,
    ),
    ("Bag", "HASITEM", Arity::Fixed(2), native_relation_has_item),
    (
        "Bag",
        "REMOVEITEM",
        Arity::Fixed(2),
        native_relation_remove_item,
    ),
];
