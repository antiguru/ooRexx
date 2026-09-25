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

//! The collection classes' primitive methods.

use super::{
    Arity, ArrayArgument, BehaviourId, Body, Cleared, Decoded, Failure, IndexUse, Interp, Loud,
    NativeMethod, ObjRef, Raised, array_argument, array_dimensions, array_position, array_slots,
    array_slots_owned, new_instance, request_array,
};
use super::{INIT, array_size_argument, optional_length_argument, unsigned_index};
use super::{native_array_at_for, positive_index};
use rexx_parse::Operator;

// `List`'s primitive methods, chained into `ObjectModel::build` ahead of this
// module's own.
pub(super) mod list;
use super::array::surface::native_array_delete;
use list::{is_list, list_insert_at, list_state};

// `Queue`'s primitive methods, chained into `ObjectModel::build` ahead of this
// module's own.
pub(super) mod queue;

// `Supplier`'s primitive methods, chained into `ObjectModel::build` after this
// module's own.
pub(super) mod supplier;
pub(super) use supplier::new_supplier;

/// [`array_slots`] over the receiver's store rather than the receiver.
pub(super) fn slots_of(
    interp: &mut Interp,
    receiver: ObjRef,
) -> Result<Vec<Option<ObjRef>>, Failure> {
    let store = store_of(interp, receiver)?;
    array_slots_owned(interp, store)
}

/// [`array_dimensions`] over the receiver's store rather than the receiver.
pub(super) fn dimensions_of(
    interp: &mut Interp,
    receiver: ObjRef,
) -> Result<Option<Vec<usize>>, Failure> {
    let store = store_of(interp, receiver)?;
    array_dimensions(interp, store)
}

/// [`array_position`] over the receiver's store rather than the receiver.
pub(super) fn position_in(
    interp: &mut Interp,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
    index_use: IndexUse,
) -> Result<Option<usize>, Failure> {
    let store = store_of(interp, receiver)?;
    array_position(interp, store, args, index_use)
}

/// The Array-shaped store's pool name, in the receiver's own pool under the
/// `Array` class as scope -- `Supplier`'s arrangement and for the same
/// reason: `ScopePools` is already walked by the collector.
pub(super) const QUEUE_ITEMS: &[u8] = b"ITEMS";

/// The allocated extent a `Queue` was built with, which its index bound is
/// measured against and which is not its item count.
const QUEUE_CAPACITY: &[u8] = b"CAPACITY";

/// `ArrayClass::DefaultArraySize` (`classes/ArrayClass.hpp:327`).
const DEFAULT_ARRAY_SIZE: usize = 16;

/// The scope the Array-shaped store is bound under.
pub(super) fn store_scope(interp: &mut Interp) -> ObjRef {
    interp
        .classes()
        .lookup("Array")
        .expect("Array is a native class")
}

// ---- the contents protocol ----

/// The array that actually holds `receiver`'s slots.
pub(super) fn store_of(interp: &mut Interp, receiver: ObjRef) -> Result<ObjRef, Failure> {
    if interp.array_slots(receiver).is_some() {
        return Ok(receiver);
    }
    let scope = store_scope(interp);
    if let Some(store) = pool_variable(interp, receiver, scope, QUEUE_ITEMS) {
        return Ok(store);
    }
    // **A subclass `INIT` that never forwards still has a store.** Upstream
    // the contents belong to the object `newRexx` allocates, so they are
    // there before any `INIT` runs and a subclass that does not chain up
    // cannot lose them. Measured: a `Queue` subclass whose `INIT` only sets
    // an exposed variable answers `items 1` after `queue('a')`, and the same
    // of a `CircularQueue` subclass -- both of which this crate refused with
    // `a value that is not an array` when the store was `INIT`'s to make.
    if !holds_array_store(interp, receiver) {
        return Err(Loud::receiver_class("a value that is not an array").into());
    }
    let store = interp.alloc_with(BehaviourId::ARRAY, Body::array(Vec::new()));
    interp.roots.push_temp(store);
    interp.set_pool_variable(receiver, scope, QUEUE_ITEMS, store);
    Ok(store)
}

/// Whether `receiver` is an instance of a class this file gives an
/// Array-shaped store to, which is what [`store_of`] may build one for. Any
/// other receiver keeps the refusal.
fn holds_array_store(interp: &mut Interp, receiver: ObjRef) -> bool {
    let Some(class) = interp.class_of_value(receiver) else {
        return false;
    };
    for name in ["Array", "Queue", "CircularQueue"] {
        let Some(base) = interp.classes().lookup(name) else {
            continue;
        };
        if interp.classes().is_a(class, base) {
            return true;
        }
    }
    false
}

/// Every (index, item) pair an ordered receiver holds, in the store's own
/// order, holes skipped.
pub(super) fn ordered_pairs(
    interp: &mut Interp,
    receiver: ObjRef,
) -> Result<Vec<(ObjRef, ObjRef)>, Failure> {
    let receiver = store_of(interp, receiver)?;
    let slots = array_slots_owned(interp, receiver)?;
    let dimensions = array_dimensions(interp, receiver)?;
    let mut pairs = Vec::new();
    for (offset, slot) in slots.iter().enumerate() {
        let Some(item) = *slot else { continue };
        let index = subscript_object(interp, offset, dimensions.as_deref());
        // **Rooted as it is built.** A multi-dimensional index is a freshly
        // allocated `Array` and the next iteration allocates again, so an
        // unrooted one is collected and the send that reads it reports the
        // object is no longer live. Measured: `collect_stress` caught exactly
        // that on `array_enumeration.rex`'s multi-dimensional half, and
        // nothing in the ordinary run saw it.
        interp.roots.push_temp(index);
        // **The item is rooted too, and for a different reason than the
        // index.** The index is a fresh allocation; the item is only
        // reachable through the collection, and the very next thing a caller
        // does with these pairs is send `==` or `compare`, which runs Rexx
        // that may empty the collection.
        interp.roots.push_temp(item);
        pairs.push((index, item));
    }
    Ok(pairs)
}

/// `ArrayClass`'s `lastItem`: the 1-based index of the outermost occupied
/// slot, or 0 for a collection holding nothing.
pub(super) fn last_item(interp: &mut Interp, receiver: ObjRef) -> Result<usize, Failure> {
    Ok(occupied(interp, receiver)?
        .last()
        .map_or(0, |last| last + 1))
}

/// The index object for the flat 0-based `offset` of an array shaped
/// `dimensions`: the 1-based position for a single dimension, and an `Array`
/// of coordinates for more than one.
pub(super) fn subscript_object(
    interp: &mut Interp,
    offset: usize,
    dimensions: Option<&[usize]>,
) -> ObjRef {
    let Some(shape) = dimensions.filter(|shape| shape.len() > 1) else {
        return interp.counted(offset + 1);
    };
    let mut coordinates = vec![None; shape.len()];
    let mut rest = offset;
    for axis in (0..shape.len()).rev() {
        let extent = shape[axis].max(1);
        coordinates[axis] = Some(interp.counted(rest % extent + 1));
        rest /= extent;
    }
    interp.alloc_with(BehaviourId::ARRAY, Body::array(coordinates))
}

/// An `Array` object over `items`.
pub(super) fn array_of(interp: &mut Interp, items: Vec<ObjRef>) -> ObjRef {
    array_of_slots(interp, items.into_iter().map(Some).collect())
}

/// An `Array` object over `slots`, holes and all -- what a `List` carrying an
/// entry that holds nothing answers for `allItems`.
fn array_of_slots(interp: &mut Interp, slots: Vec<Option<ObjRef>>) -> ObjRef {
    let array = interp.alloc_with(BehaviourId::ARRAY, Body::array(slots));
    interp.roots.push_temp(array);
    array
}

/// Whether the collection holds `wanted`, comparing it against `element`.
pub(super) fn same_item(
    interp: &mut Interp,
    wanted: ObjRef,
    element: ObjRef,
) -> Result<bool, Failure> {
    let (left, right) = (wanted, element);
    let answer = interp.apply_binary(Operator::StrictEqual, left, right)?;
    let text = interp.to_text(answer);
    Ok(crate::eval::logical_value(&text).unwrap_or(false))
}

/// The one item argument a collection method takes, which is required.
pub(super) fn item_argument(args: &[Option<ObjRef>]) -> Result<ObjRef, Failure> {
    args.first()
        .copied()
        .flatten()
        .ok_or_else(|| Raised::missing_method_argument(1).into())
}

/// What `name` holds in `owner`'s pool for `scope`, or `None`.
fn pool_variable(interp: &Interp, owner: ObjRef, scope: ObjRef, name: &[u8]) -> Option<ObjRef> {
    match interp.heap.get(owner).map(|object| &object.body) {
        Some(Body::Instance { pools, .. }) => pools.get(scope, name),
        _ => None,
    }
}

/// The 0-based offsets of `receiver`'s occupied slots.
pub(super) fn occupied(interp: &mut Interp, receiver: ObjRef) -> Result<Vec<usize>, Failure> {
    let receiver = store_of(interp, receiver)?;
    Ok(array_slots(interp, receiver)?
        .iter()
        .enumerate()
        .filter_map(|(offset, slot)| slot.map(|_| offset))
        .collect())
}

/// `append`'s body without the message send: writes `item` past the last
/// item, growing to fit, and answers the 1-based index it landed on.
pub(super) fn append_slot(
    interp: &mut Interp,
    receiver: ObjRef,
    item: ObjRef,
) -> Result<usize, Failure> {
    let at = last_item(interp, receiver)?;
    let length = slots_of(interp, receiver)?.len();
    if at >= length {
        array_grow(interp, receiver, at + 1)?;
    }
    write_slot(interp, receiver, at, Some(item))?;
    Ok(at + 1)
}

/// A `Queue`'s two-tier index bound, which is two different errors.
pub(super) fn queue_bound(
    interp: &mut Interp,
    receiver: ObjRef,
    position: usize,
) -> Result<(), Failure> {
    if !is_queue(interp, receiver) {
        return Ok(());
    }
    // The extent is the larger of what the queue was built with and what it
    // now holds -- measured, a default queue grown to 20 items bounds at 20.
    let scope = store_scope(interp);
    let declared = match pool_variable(interp, receiver, scope, QUEUE_CAPACITY) {
        Some(value) => match value.decode() {
            Decoded::SmallInt(value) => value as usize,
            _ => DEFAULT_ARRAY_SIZE,
        },
        None => DEFAULT_ARRAY_SIZE,
    };
    let extent = declared.max(slots_of(interp, receiver)?.iter().flatten().count());
    if position > extent {
        let written = position.to_string().into_bytes();
        return Err(Raised::incorrect_list_index(&written).into());
    }
    if position > last_item(interp, receiver)? {
        return Err(Raised::incorrect_queue_index(position).into());
    }
    Ok(())
}

/// Whether `receiver` is a `Queue` or something deriving from one.
fn is_queue(interp: &mut Interp, receiver: ObjRef) -> bool {
    let Some(class) = interp.class_of_value(receiver) else {
        return false;
    };
    let Some(queue) = interp.classes().lookup("Queue") else {
        return false;
    };
    interp.classes().is_a(class, queue)
}

/// Inserts `item` at 0-based `at`, or deletes the slot there when it is
/// `None`, shifting everything after it.
pub(super) fn array_splice(
    interp: &mut Interp,
    receiver: ObjRef,
    at: usize,
    item: Option<ObjRef>,
) -> Result<(), Failure> {
    interp.bump_route_generation();
    let receiver = store_of(interp, receiver)?;
    match item {
        Some(item) => array_splice_slot(interp, receiver, at, Some(item)),
        None => match interp.heap.get_mut(receiver).map(|object| &mut object.body) {
            Some(Body::Array { slots, .. }) => {
                if at < slots.len() {
                    slots.remove(at);
                }
                Ok(())
            }
            _ => Err(Loud::receiver_class("a value that is not an array").into()),
        },
    }
}

/// An instance of `class` whose store is `store`, for a `~new` on a subclass
/// of `Array` and for a section of one.
pub(super) fn instance_over_store(
    interp: &mut Interp,
    class: ObjRef,
    store: ObjRef,
) -> Result<ObjRef, Failure> {
    interp.roots.push_temp(store);
    let object = new_instance(interp, class)?;
    interp.roots.push_temp(object);
    let scope = store_scope(interp);
    interp.set_pool_variable(object, scope, QUEUE_ITEMS, store);
    Ok(object)
}

/// Inserts a slot at 0-based `at`, which may be empty -- `insert` with its
/// item omitted opens a hole rather than refusing.
pub(super) fn array_splice_slot(
    interp: &mut Interp,
    receiver: ObjRef,
    at: usize,
    item: Option<ObjRef>,
) -> Result<(), Failure> {
    interp.bump_route_generation();
    let receiver = store_of(interp, receiver)?;
    match interp.heap.get_mut(receiver).map(|object| &mut object.body) {
        Some(Body::Array { slots, .. }) => {
            let at = at.min(slots.len());
            slots.insert(at, item);
            Ok(())
        }
        _ => Err(Loud::receiver_class("a value that is not an array").into()),
    }
}

/// Writes one slot of the store without shifting anything.
fn write_slot(
    interp: &mut Interp,
    receiver: ObjRef,
    at: usize,
    item: Option<ObjRef>,
) -> Result<(), Failure> {
    // A monitor keeps its destination in a `Queue`, so an array write is one
    // of the ways `SAY`'s route moves.
    interp.bump_route_generation();
    let store = store_of(interp, receiver)?;
    match interp.heap.get_mut(store).map(|object| &mut object.body) {
        Some(Body::Array { slots, .. }) => {
            if let Some(slot) = slots.get_mut(at) {
                *slot = item;
            }
            Ok(())
        }
        _ => Err(Loud::receiver_class("a value that is not an array").into()),
    }
}

/// Grows `receiver` to `length` empty slots, for an `insert` past the end.
pub(super) fn array_grow(
    interp: &mut Interp,
    receiver: ObjRef,
    length: usize,
) -> Result<(), Failure> {
    let receiver = store_of(interp, receiver)?;
    match interp.heap.get_mut(receiver).map(|object| &mut object.body) {
        Some(Body::Array { slots, .. }) => {
            slots.resize(length, None);
            Ok(())
        }
        _ => Err(Loud::receiver_class("a value that is not an array").into()),
    }
}
/// `Queue~of(item, ...)` and `List~of(item, ...)`: a new collection of the
/// receiver's own class holding the arguments.
pub(super) fn native_collection_of(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = super::class_receiver(interp, receiver)?;
    for (at, argument) in args.iter().enumerate() {
        if argument.is_none() {
            return Err(Raised::missing_method_argument(at + 1).into());
        }
    }
    let object = new_instance(interp, class)?;
    interp.roots.push_temp(object);
    let caller = interp.caller();
    interp.send_message(object, super::INIT, None, &[], caller)?;
    let list = is_list(interp, object);
    for argument in args.iter().flatten() {
        if list {
            let (items, _, _) = list_state(interp, object)?;
            let at = array_slots(interp, items)?.len();
            list_insert_at(interp, object, at, Some(*argument))?;
        } else {
            append_slot(interp, object, *argument)?;
        }
    }
    Ok(Some(object))
}
