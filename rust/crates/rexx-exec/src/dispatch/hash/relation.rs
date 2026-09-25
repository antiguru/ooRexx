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
    not_this_task, owns, pairs, same_index, slot_at, store_of, take, take_at, walk,
};

/// Every slot whose index matches, in chain order.
fn slots_for(interp: &mut Interp, receiver: ObjRef, index: ObjRef) -> Result<Vec<usize>, Failure> {
    let store = store_of(interp, receiver)?;
    let keys = keys_of(interp, receiver);
    let mut found = Vec::new();
    for slot in walk(interp, receiver)? {
        if same_index(interp, &store, keys, slot, index)? {
            found.push(slot);
        }
    }
    Ok(found)
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
    let store = store_of(interp, receiver)?;
    let mut items = Vec::new();
    for slot in slots_for(interp, receiver, index)? {
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
        Some(index) => slots_for(interp, receiver, index)?.len(),
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
        interp.roots.push_temp(item);
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
    let store = store_of(interp, receiver)?;
    let mut items = Vec::new();
    let mut indexes = Vec::new();
    for slot in slots_for(interp, receiver, index)? {
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
    let under = args.get(1).copied().flatten();
    let store = store_of(interp, receiver)?;
    let keys = keys_of(interp, receiver);
    for slot in walk(interp, receiver)? {
        let Some(item) = slot_at(interp, store.items, slot)? else {
            continue;
        };
        if !super::collection::same_item(interp, wanted, item)? {
            continue;
        }
        match under {
            None => return Ok(Some(crate::eval::logical(true))),
            Some(index) if same_index(interp, &store, keys, slot, index)? => {
                return Ok(Some(crate::eval::logical(true)));
            }
            Some(_) => {}
        }
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
    let under = args.get(1).copied().flatten();
    let store = store_of(interp, receiver)?;
    let keys = keys_of(interp, receiver);
    for slot in walk(interp, receiver)? {
        let Some(item) = slot_at(interp, store.items, slot)? else {
            continue;
        };
        if !super::collection::same_item(interp, wanted, item)? {
            continue;
        }
        let held = slot_at(interp, store.indexes, slot)?;
        let matches = match under {
            None => true,
            Some(index) => same_index(interp, &store, keys, slot, index)?,
        };
        if matches && let Some(held) = held {
            take_at(interp, receiver, held, slot)?;
            return Ok(Some(item));
        }
    }
    Ok(Some(ObjRef::NIL))
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
