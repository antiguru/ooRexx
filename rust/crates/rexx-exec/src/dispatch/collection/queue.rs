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

//! `Queue`'s primitive methods, over the Array-shaped store its parent module
//! keeps.

use super::{
    Arity, Body, Cleared, DEFAULT_ARRAY_SIZE, Failure, IndexUse, Interp, Loud, NativeMethod,
    ObjRef, QUEUE_CAPACITY, Raised, append_slot, array_position, array_slots, array_splice,
    array_splice_slot, item_argument, native_array_delete, pool_variable, queue_bound, slots_of,
    store_of, store_scope,
};

// ---- Queue ----

/// `Queue~init([size])`: validates the optional capacity the way every
/// collection's does, and gives the instance the array that holds its items.
fn native_queue_init(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let asked = super::optional_length_argument(interp, args, 0)?.unwrap_or(0);
    // **Keeps the contents and the extent a second `INIT` finds.** Measured,
    // `q~init` on a two-item queue still answers `items 2`, and on a
    // `.Queue~new(50)` holding one item `put('m', 50)` is still 93.966 --
    // inside the extent, past the last item -- where a reset extent would
    // have made it 93.918.
    store_of(interp, receiver)?;
    let scope = store_scope(interp);
    if pool_variable(interp, receiver, scope, QUEUE_CAPACITY).is_none() {
        // Measured: `.Queue~new(5)` bounds at 16 and `.Queue~new(50)` at 50,
        // so the requested extent is floored at the default rather than
        // replacing it.
        let capacity = interp.counted(asked.max(DEFAULT_ARRAY_SIZE));
        interp.set_pool_variable(receiver, scope, QUEUE_CAPACITY, capacity);
    }
    Ok(None)
}

/// `Queue~queue(item)`: adds at the **end** -- `QueueClass::queueRexx`.
fn native_queue_queue(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let item = item_argument(args)?;
    // Past the last ITEM, as `append` is: a queue emptied after holding two
    // starts again at index 1, and `peek` reads that slot.
    append_slot(interp, receiver, item)?;
    Ok(None)
}

/// `Queue~push(item)`: adds at the **front** -- `QueueClass::pushRexx`.
fn native_queue_push(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let item = item_argument(args)?;
    let store = store_of(interp, receiver)?;
    array_splice_slot(interp, store, 0, Some(item))?;
    Ok(None)
}

/// `Queue~peek`: the front item without removing it, or `.nil` for an empty
/// queue -- `QueueClass::peek`.
fn native_queue_peek(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let store = store_of(interp, receiver)?;
    Ok(Some(match array_slots(interp, store)?.first() {
        Some(Some(item)) => *item,
        Some(None) | None => ObjRef::NIL,
    }))
}

/// `Queue~pull`: takes the front item off and answers it, or `.nil` --
/// `QueueClass::pullRexx`.
fn native_queue_pull(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let store = store_of(interp, receiver)?;
    let front = match array_slots(interp, store)?.first() {
        Some(Some(item)) => *item,
        Some(None) | None => return Ok(Some(ObjRef::NIL)),
    };
    array_splice(interp, store, 0, None)?;
    Ok(Some(front))
}

/// `Queue~delete(index)` and `Queue~remove(index)`, which are **one body**
/// upstream (`QueueClass::deleteRexx`) where `Array` has two.
fn native_queue_delete(
    interp: &mut Interp,
    cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    native_array_delete(interp, cleared, receiver, args)
}

/// `Queue~put(item, index)` and `Queue~[index] = item`:
/// `QueueClass::putRexx`, which replaces rather than growing.
fn native_queue_put(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let item = item_argument(args)?;
    let Some(index) = args.get(1).copied().flatten() else {
        return Err(Raised::missing_method_argument(2).into());
    };
    let store = store_of(interp, receiver)?;
    let refuse = |interp: &mut Interp| {
        let written = interp.to_text(index);
        Failure::from(Raised::incorrect_list_index(&written))
    };
    // `putRexx` calls `checkInsertIndex` (`classes/QueueClass.cpp:205`)
    // before anything else can refuse, so an index past the last item is
    // 93.966 and never the 93.918 an unheld index would answer.
    if let Some(index) = args.get(1).copied().flatten() {
        let position = super::positive_index(interp, index, 2)?;
        queue_bound(interp, receiver, position)?;
    }
    let Some(position) = array_position(interp, store, &args[1..], IndexUse::Get)? else {
        return Err(refuse(interp));
    };
    match interp.heap.get_mut(store).map(|object| &mut object.body) {
        Some(Body::Array { slots, .. }) => {
            slots[position - 1] = Some(item);
            Ok(None)
        }
        _ => Err(Loud::receiver_class("a value that is not an array").into()),
    }
}

/// `Queue~at(index)` and `Queue~[index]`: `ArrayClass::getRexx` over the
/// store.
fn native_queue_at(
    interp: &mut Interp,
    cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let store = store_of(interp, receiver)?;
    super::native_array_at_for(interp, cleared, store, args)
}

/// `Queue~items`: how many slots of the store hold an object.
fn native_queue_items(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let items = slots_of(interp, receiver)?.iter().flatten().count();
    Ok(Some(interp.counted(items)))
}

/// `Queue~size`: how many slots the store has.
/// **`Setup.cpp:789` maps it to `ArrayClass::itemsRexx`**, so a `Queue`'s
/// size is its item count and not its slot count -- measured, a queue emptied
/// after holding two answers `size 0`.
fn native_queue_size(
    interp: &mut Interp,
    cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    native_queue_items(interp, cleared, receiver, args)
}

/// `Queue`'s primitive methods, chained into `ObjectModel::build` beside
/// [`super::NATIVE_METHODS`].
pub(in crate::dispatch) const NATIVE_METHODS: &[(&str, &str, Arity, NativeMethod)] = &[
    ("Queue", "AT", Arity::Counted, native_queue_at),
    ("Queue", "[]", Arity::Counted, native_queue_at),
    ("Queue", "ITEMS", Arity::Fixed(0), native_queue_items),
    ("Queue", "SIZE", Arity::Fixed(0), native_queue_size),
    ("Queue", "INIT", Arity::Fixed(1), native_queue_init),
    ("Queue", "QUEUE", Arity::Fixed(1), native_queue_queue),
    ("Queue", "PUSH", Arity::Fixed(1), native_queue_push),
    ("Queue", "PEEK", Arity::Fixed(0), native_queue_peek),
    ("Queue", "PULL", Arity::Fixed(0), native_queue_pull),
    ("Queue", "DELETE", Arity::Fixed(1), native_queue_delete),
    ("Queue", "REMOVE", Arity::Fixed(1), native_queue_delete),
    ("Queue", "PUT", Arity::Fixed(2), native_queue_put),
    ("Queue", "[]=", Arity::Fixed(2), native_queue_put),
];
