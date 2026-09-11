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
    array_slots_owned, new_instance, request_array, whole_comparison,
};
use rexx_parse::Operator;

/// The `Supplier` state's three names, bound in the receiver's own pool under
/// the `Supplier` class as scope.
const SUPPLIER_ITEMS: &[u8] = b"ITEMS";
const SUPPLIER_INDEXES: &[u8] = b"INDEXES";
const SUPPLIER_POSITION: &[u8] = b"POSITION";

/// [`array_slots`] over the receiver's store rather than the receiver.
fn slots_of(interp: &mut Interp, receiver: ObjRef) -> Result<Vec<Option<ObjRef>>, Failure> {
    let store = store_of(interp, receiver)?;
    array_slots_owned(interp, store)
}

/// [`array_dimensions`] over the receiver's store rather than the receiver.
fn dimensions_of(interp: &mut Interp, receiver: ObjRef) -> Result<Option<Vec<usize>>, Failure> {
    let store = store_of(interp, receiver)?;
    array_dimensions(interp, store)
}

/// [`array_position`] over the receiver's store rather than the receiver.
fn position_in(
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
const QUEUE_ITEMS: &[u8] = b"ITEMS";

/// The allocated extent a `Queue` was built with, which its index bound is
/// measured against and which is not its item count.
const QUEUE_CAPACITY: &[u8] = b"CAPACITY";

/// `ArrayClass::DefaultArraySize` (`classes/ArrayClass.hpp:327`).
const DEFAULT_ARRAY_SIZE: usize = 16;

/// The scope the Array-shaped store is bound under.
fn store_scope(interp: &mut Interp) -> ObjRef {
    interp
        .classes()
        .lookup("Array")
        .expect("Array is a native class")
}

// ---- the contents protocol ----

/// The array that actually holds `receiver`'s slots.
fn store_of(interp: &mut Interp, receiver: ObjRef) -> Result<ObjRef, Failure> {
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
fn ordered_pairs(interp: &mut Interp, receiver: ObjRef) -> Result<Vec<(ObjRef, ObjRef)>, Failure> {
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
fn last_item(interp: &mut Interp, receiver: ObjRef) -> Result<usize, Failure> {
    Ok(occupied(interp, receiver)?
        .last()
        .map_or(0, |last| last + 1))
}

/// The index object for the flat 0-based `offset` of an array shaped
/// `dimensions`: the 1-based position for a single dimension, and an `Array`
/// of coordinates for more than one.
fn subscript_object(interp: &mut Interp, offset: usize, dimensions: Option<&[usize]>) -> ObjRef {
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

// ---- Array's shared collection surface ----

/// `Array~allItems`: every item, holes skipped, in index order --
/// `ArrayClass::allItems`.
fn native_array_all_items(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let items = ordered_pairs(interp, receiver)?
        .into_iter()
        .map(|(_, item)| item)
        .collect();
    Ok(Some(array_of(interp, items)))
}

/// `Array~allIndexes`: the index of every item, in the same order --
/// `ArrayClass::allIndexes`.
fn native_array_all_indexes(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let indexes = ordered_pairs(interp, receiver)?
        .into_iter()
        .map(|(index, _)| index)
        .collect();
    Ok(Some(array_of(interp, indexes)))
}

/// `Array~makeArray`: `RexxObject::makeArrayRexx`, which is
/// `return makeArray();` and therefore a **virtual**.
fn native_array_make_array(
    interp: &mut Interp,
    cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    native_array_all_items(interp, cleared, receiver, args)
}

/// `Array~isEmpty`: whether the array holds no item -- `ArrayClass::isEmptyRexx`.
fn native_array_is_empty(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let empty = slots_of(interp, receiver)?.iter().flatten().count() == 0;
    Ok(Some(crate::eval::logical(empty)))
}

/// `Array~empty`: drops every item and keeps the array's shape --
/// `ArrayClass::empty`.
fn native_array_empty(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let length = slots_of(interp, receiver)?.len();
    for offset in 0..length {
        clear_array_slot(interp, receiver, offset)?;
    }
    Ok(Some(receiver))
}

/// `Array~hasItem(item)`: whether any slot holds it -- `ArrayClass::hasItemRexx`.
fn native_array_has_item(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let wanted = item_argument(args)?;
    for (_, item) in ordered_pairs(interp, receiver)? {
        if same_item(interp, wanted, item)? {
            return Ok(Some(crate::eval::logical(true)));
        }
    }
    Ok(Some(crate::eval::logical(false)))
}

/// `Array~index(item)`: the index of the first slot holding it, or `.nil` --
/// `ArrayClass::indexRexx`.
fn native_array_index(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let wanted = item_argument(args)?;
    for (index, item) in ordered_pairs(interp, receiver)? {
        if same_item(interp, wanted, item)? {
            return Ok(Some(index));
        }
    }
    Ok(Some(ObjRef::NIL))
}

/// `Array~hasIndex(index...)`: whether that slot holds an item --
/// `ArrayClass::hasIndexRexx`, `A_COUNT` because a multi-dimensional
/// subscript arrives as several arguments.
fn native_array_has_index(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let Some(position) = position_in(interp, receiver, args, IndexUse::Get)? else {
        return Ok(Some(crate::eval::logical(false)));
    };
    let held = matches!(
        slots_of(interp, receiver)?.get(position - 1).copied(),
        Some(Some(_))
    );
    Ok(Some(crate::eval::logical(held)))
}

/// `Array~remove(index...)`: takes the item out of that slot and answers it,
/// or `.nil` -- `ArrayClass::removeRexx`.
fn native_array_remove(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let Some(position) = position_in(interp, receiver, args, IndexUse::Get)? else {
        return Ok(Some(ObjRef::NIL));
    };
    let held = match slots_of(interp, receiver)?.get(position - 1).copied() {
        Some(Some(item)) => item,
        Some(None) | None => return Ok(Some(ObjRef::NIL)),
    };
    clear_array_slot(interp, receiver, position - 1)?;
    Ok(Some(held))
}

/// `Array~removeItem(item)`: clears the first slot holding it and answers it,
/// or `.nil` -- `ArrayClass::removeItemRexx`.
fn native_array_remove_item(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let wanted = item_argument(args)?;
    let slots = slots_of(interp, receiver)?;
    for (offset, slot) in slots.iter().enumerate() {
        let Some(item) = *slot else { continue };
        if same_item(interp, wanted, item)? {
            clear_array_slot(interp, receiver, offset)?;
            return Ok(Some(item));
        }
    }
    Ok(Some(ObjRef::NIL))
}

/// `Array~supplier`: a `Supplier` over this array's items and indexes --
/// `ArrayClass::supplier`.
fn native_array_supplier(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let pairs = ordered_pairs(interp, receiver)?;
    let items: Vec<ObjRef> = pairs.iter().map(|(_, item)| *item).collect();
    let indexes: Vec<ObjRef> = pairs.into_iter().map(|(index, _)| index).collect();
    let items = array_of(interp, items);
    interp.roots.push_temp(items);
    let indexes = array_of(interp, indexes);
    interp.roots.push_temp(indexes);
    new_supplier(interp, items, indexes).map(Some)
}

/// Empties one slot of an array receiver.
fn clear_array_slot(interp: &mut Interp, receiver: ObjRef, offset: usize) -> Result<(), Failure> {
    let receiver = store_of(interp, receiver)?;
    match interp.heap.get_mut(receiver).map(|object| &mut object.body) {
        Some(Body::Array { slots, .. }) => {
            if let Some(slot) = slots.get_mut(offset) {
                *slot = None;
            }
            Ok(())
        }
        _ => Err(Loud::receiver_class("a value that is not an array").into()),
    }
}

/// What `name` holds in `owner`'s pool for `scope`, or `None`.
fn pool_variable(interp: &Interp, owner: ObjRef, scope: ObjRef, name: &[u8]) -> Option<ObjRef> {
    match interp.heap.get(owner).map(|object| &object.body) {
        Some(Body::Instance { pools, .. }) => pools.get(scope, name),
        _ => None,
    }
}

// ---- Supplier ----

/// A `Supplier` over `items` and `indexes`, positioned at the first pair.
pub(super) fn new_supplier(
    interp: &mut Interp,
    items: ObjRef,
    indexes: ObjRef,
) -> Result<ObjRef, Failure> {
    let class = interp
        .classes()
        .lookup("Supplier")
        .expect("Supplier is a native class");
    let object = new_instance(interp, class)?;
    interp.roots.push_temp(object);
    store_supplier(interp, object, items, indexes, 0);
    Ok(object)
}

fn store_supplier(
    interp: &mut Interp,
    receiver: ObjRef,
    items: ObjRef,
    indexes: ObjRef,
    position: usize,
) {
    let scope = interp
        .classes()
        .lookup("Supplier")
        .expect("Supplier is a native class");
    let position = interp.counted(position);
    interp.set_pool_variable(receiver, scope, SUPPLIER_ITEMS, items);
    interp.set_pool_variable(receiver, scope, SUPPLIER_INDEXES, indexes);
    interp.set_pool_variable(receiver, scope, SUPPLIER_POSITION, position);
}

/// The supplier's two arrays and its 0-based position, or the refusal a
/// receiver that is not one gets.
fn supplier_state(
    interp: &mut Interp,
    receiver: ObjRef,
) -> Result<(ObjRef, ObjRef, usize), Failure> {
    let scope = interp
        .classes()
        .lookup("Supplier")
        .expect("Supplier is a native class");
    let (Some(items), Some(indexes), Some(position)) = (
        pool_variable(interp, receiver, scope, SUPPLIER_ITEMS),
        pool_variable(interp, receiver, scope, SUPPLIER_INDEXES),
        pool_variable(interp, receiver, scope, SUPPLIER_POSITION),
    ) else {
        return Err(Loud::receiver_class("a value that is not a supplier").into());
    };
    let Decoded::SmallInt(position) = position.decode() else {
        unreachable!("the position this crate stored is a small integer")
    };
    Ok((items, indexes, position as usize))
}

/// How many pairs the supplier still has, measured from the current position
/// against its ITEMS array.
fn supplier_remaining(interp: &mut Interp, receiver: ObjRef) -> Result<usize, Failure> {
    let (items, _, position) = supplier_state(interp, receiver)?;
    let items = array_slots(interp, items)?.len();
    Ok(items.saturating_sub(position))
}

/// `Supplier~available`: whether a pair is still there --
/// `SupplierClass::available`.
fn native_supplier_available(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let remaining = supplier_remaining(interp, receiver)?;
    Ok(Some(crate::eval::logical(remaining > 0)))
}

/// `Supplier~item` and `Supplier~index`: the current pair's halves.
fn supplier_half(
    interp: &mut Interp,
    receiver: ObjRef,
    take_items: bool,
) -> Result<Option<ObjRef>, Failure> {
    if supplier_remaining(interp, receiver)? == 0 {
        return Err(Raised::no_more_supplier_items().into());
    }
    let (items, indexes, position) = supplier_state(interp, receiver)?;
    let side = if take_items { items } else { indexes };
    Ok(Some(match array_slots(interp, side)?.get(position) {
        Some(Some(value)) => *value,
        Some(None) | None => ObjRef::NIL,
    }))
}

fn native_supplier_item(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    supplier_half(interp, receiver, true)
}

fn native_supplier_index(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    supplier_half(interp, receiver, false)
}

/// `Supplier~next`: steps to the next pair, raising 93.937 past the end.
fn native_supplier_next(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    if supplier_remaining(interp, receiver)? == 0 {
        return Err(Raised::no_more_supplier_items().into());
    }
    let (items, indexes, position) = supplier_state(interp, receiver)?;
    store_supplier(interp, receiver, items, indexes, position + 1);
    Ok(None)
}

/// `Supplier~init(items, indexes)`: `SupplierClass::initRexx`.
fn native_supplier_init(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let mut sides = Vec::with_capacity(2);
    for position in [1, 2] {
        let Some(value) = args.get(position - 1).copied().flatten() else {
            return Err(Raised::missing_method_argument(position).into());
        };
        // `arrayArgument` is `requestArray`
        // (`runtime/MethodArguments.hpp:683`), so a `List` or a `Queue` is
        // converted and the CONVERTED array is what the supplier keeps --
        // measured, `.Supplier~new(.List~of('a'), .List~of('h'))` answers
        // `1 a h`. An argument that already is an array is kept as it
        // stands, object identity and all.
        let value = request_array(interp, value)?;
        // Validated for the argument error it raises, after the conversion
        // that upstream makes first.
        array_argument(interp, value, ArrayArgument::Positional)?;
        sides.push(value);
    }
    store_supplier(interp, receiver, sides[0], sides[1], 0);
    Ok(None)
}

// ---- Array's own surface ----

/// `ArrayClass::checkMultiDimensional` (`classes/ArrayClass.cpp:426`): the
/// four methods that only work on a single-dimensional array.
fn single_dimension_only(
    interp: &mut Interp,
    receiver: ObjRef,
    method: &str,
) -> Result<(), Failure> {
    let receiver = store_of(interp, receiver)?;
    if dimensions_of(interp, receiver)?.is_some_and(|shape| shape.len() > 1) {
        return Err(Raised::single_dimension_only(method).into());
    }
    Ok(())
}

/// The 0-based offsets of `receiver`'s occupied slots.
fn occupied(interp: &mut Interp, receiver: ObjRef) -> Result<Vec<usize>, Failure> {
    let receiver = store_of(interp, receiver)?;
    Ok(array_slots(interp, receiver)?
        .iter()
        .enumerate()
        .filter_map(|(offset, slot)| slot.map(|_| offset))
        .collect())
}

/// `Array~first` and `Array~last`: the INDEX of the outermost occupied slot,
/// or `.nil` -- `ArrayClass::firstRexx`, `ArrayClass::lastRexx`.
fn array_end(
    interp: &mut Interp,
    receiver: ObjRef,
    take_last: bool,
    want_item: bool,
) -> Result<Option<ObjRef>, Failure> {
    let receiver = store_of(interp, receiver)?;
    let offsets = occupied(interp, receiver)?;
    let found = if take_last {
        offsets.last().copied()
    } else {
        offsets.first().copied()
    };
    let Some(offset) = found else {
        return Ok(Some(ObjRef::NIL));
    };
    if want_item {
        return Ok(Some(match array_slots(interp, receiver)?[offset] {
            Some(item) => item,
            None => ObjRef::NIL,
        }));
    }
    let dimensions = dimensions_of(interp, receiver)?;
    Ok(Some(subscript_object(
        interp,
        offset,
        dimensions.as_deref(),
    )))
}

fn native_array_first(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    array_end(interp, receiver, false, false)
}

fn native_array_last(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    array_end(interp, receiver, true, false)
}

fn native_array_first_item(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    array_end(interp, receiver, false, true)
}

fn native_array_last_item(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    array_end(interp, receiver, true, true)
}

/// `Array~next(index...)` and `Array~previous(index...)`: the nearest occupied
/// index on that side, or `.nil` -- `ArrayClass::nextRexx`,
/// `ArrayClass::previousRexx`.
fn array_step(
    interp: &mut Interp,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
    forward: bool,
) -> Result<Option<ObjRef>, Failure> {
    let receiver = store_of(interp, receiver)?;
    // **An index past the end still steps.** Measured:
    // `.Array~of('a','b')~previous(5)` answers `2`, where bailing out on the
    // out-of-range subscript answers `.nil`.
    let position = match position_in(interp, receiver, args, IndexUse::Get)? {
        Some(position) => position,
        None => match args {
            [Some(only)] => super::positive_index(interp, *only, 1)?,
            _ => return Ok(Some(ObjRef::NIL)),
        },
    };
    let offsets = occupied(interp, receiver)?;
    let found = if forward {
        offsets.into_iter().find(|offset| *offset + 1 > position)
    } else {
        offsets
            .into_iter()
            .rev()
            .find(|offset| *offset + 1 < position)
    };
    let Some(offset) = found else {
        return Ok(Some(ObjRef::NIL));
    };
    let dimensions = dimensions_of(interp, receiver)?;
    Ok(Some(subscript_object(
        interp,
        offset,
        dimensions.as_deref(),
    )))
}

fn native_array_next(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    array_step(interp, receiver, args, true)
}

fn native_array_previous(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    array_step(interp, receiver, args, false)
}

/// `Array~append(item)`: puts `item` past the last slot and answers its index
/// -- `ArrayClass::appendRexx`.
fn native_array_append(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let item = item_argument(args)?;
    single_dimension_only(interp, receiver, "APPEND")?;
    let at = append_slot(interp, receiver, item)?;
    Ok(Some(interp.counted(at)))
}

/// `append`'s body without the message send: writes `item` past the last
/// item, growing to fit, and answers the 1-based index it landed on.
fn append_slot(interp: &mut Interp, receiver: ObjRef, item: ObjRef) -> Result<usize, Failure> {
    let at = last_item(interp, receiver)?;
    let length = slots_of(interp, receiver)?.len();
    if at >= length {
        array_grow(interp, receiver, at + 1)?;
    }
    write_slot(interp, receiver, at, Some(item))?;
    Ok(at + 1)
}

/// A `Queue`'s two-tier index bound, which is two different errors.
fn queue_bound(interp: &mut Interp, receiver: ObjRef, position: usize) -> Result<(), Failure> {
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

/// `Array~insert(item [, index])`: puts `item` **after** `index` and shifts
/// the rest along, answering the index it landed on --
/// `ArrayClass::insertRexx`.
fn native_array_insert(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    single_dimension_only(interp, receiver, "INSERT")?;
    // **The item is optional**, unlike `append`'s: `insertRexx` has no
    // `requiredArgument` for it. Measured, `.Array~of('x','y')~insert`
    // answers `3` and leaves size 3 with items 2 -- an empty slot.
    let item = args.first().copied().flatten();
    let at = match args.get(1).copied() {
        // `.nil` is the front, which is the one spelling that is not an
        // index at all (`classes/ArrayClass.cpp:755`).
        Some(Some(index)) if index == ObjRef::NIL => 0,
        Some(Some(index)) => {
            let position = super::positive_index(interp, index, 2)?;
            // `QueueClass::checkInsertIndex` (`classes/QueueClass.cpp:113`):
            // "the position must be location of an existing item within the
            // bounds of the queue, unlike an array which can insert at empty
            // slots or beyond the existing bounds".
            queue_bound(interp, receiver, position)?;
            position
        }
        // Omitted: after the last OCCUPIED slot, not the end of the array.
        Some(None) | None => last_item(interp, receiver)?,
    };
    let length = slots_of(interp, receiver)?.len();
    if at > length {
        array_grow(interp, receiver, at)?;
    }
    splice_absorbing_slack(interp, receiver, at, item)?;
    Ok(Some(interp.counted(at + 1)))
}

/// `Array~delete(index...)`: takes the slot out, closes the gap and answers
/// the item -- `ArrayClass::deleteRexx`.
fn native_array_delete(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    // `requiredArgument(index, ARG_ONE)` comes FIRST and is 93.903, where
    // the index-validating family is 93.901 -- `classes/ArrayClass.cpp:822`.
    item_argument(args)?;
    single_dimension_only(interp, receiver, "DELETE")?;
    let Some(position) = position_in(interp, receiver, args, IndexUse::Get)? else {
        return Ok(Some(ObjRef::NIL));
    };
    let held = match slots_of(interp, receiver)?.get(position - 1).copied() {
        Some(slot) => slot,
        None => return Ok(Some(ObjRef::NIL)),
    };
    array_splice(interp, receiver, position - 1, None)?;
    Ok(Some(held.unwrap_or(ObjRef::NIL)))
}

/// `Array~fill(item)`: puts `item` in every slot and answers the receiver --
/// `ArrayClass::fillRexx`.
fn native_array_fill(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let item = item_argument(args)?;
    // Through the store like everything else -- reaching `receiver` directly
    // made this the one row that worked on an `Array` and refused on a
    // subclass of one.
    let store = store_of(interp, receiver)?;
    match interp.heap.get_mut(store).map(|object| &mut object.body) {
        Some(Body::Array { slots, .. }) => {
            for slot in slots.iter_mut() {
                *slot = Some(item);
            }
            Ok(Some(receiver))
        }
        _ => Err(Loud::receiver_class("a value that is not an array").into()),
    }
}

/// `Array~section(start [, count])`: a new array over that run --
/// `ArrayClass::sectionRexx`.
fn native_array_section(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    // The dimension check comes FIRST here and the required argument second,
    // which is the opposite order from `delete` -- `:1502` against `:822`.
    single_dimension_only(interp, receiver, "SECTION")?;
    let start = item_argument(args)?;
    let start = super::positive_index(interp, start, 1)?;
    let slots = slots_of(interp, receiver)?;
    let available = slots.len().saturating_sub(start.saturating_sub(1));
    let count = match args.get(1).copied().flatten() {
        Some(count) => super::array_size_argument(interp, Some(count), 2)?.min(available),
        None => available,
    };
    let taken: Vec<Option<ObjRef>> = slots
        .into_iter()
        .skip(start.saturating_sub(1))
        .take(count)
        .collect();
    Ok(Some(same_class_array(interp, receiver, taken)?))
}

/// `Array~dimensions`: an `Array` of the extents --
/// `ArrayClass::getDimensionsRexx`.
fn native_array_dimensions(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    // **A single-dimensional array answers its SIZE, not the extent it was
    // fixed at.** `getDimensionsRexx` is `new_array(new_integer(size()))` for
    // one (`classes/ArrayClass.cpp:1084`), and only a multi-dimensional array
    // reports the stored shape. Measured: `.Array~new(0)~append('q')` then
    // `~dimensions` is `1` -- and `.Array~of()` the same -- where reading the
    // fixed list answered the stale `0`.
    let extents = match dimensions_of(interp, receiver)? {
        Some(shape) if shape.len() > 1 => shape,
        _ => vec![slots_of(interp, receiver)?.len()],
    };
    let items = extents
        .into_iter()
        .map(|extent| interp.counted(extent))
        .collect();
    Ok(Some(array_of(interp, items)))
}

// ---- the sort family ----

/// Every item of a receiver the sort family will accept, in index order.
fn dense_items(interp: &mut Interp, receiver: ObjRef) -> Result<Vec<ObjRef>, Failure> {
    let receiver = store_of(interp, receiver)?;
    let slots = array_slots(interp, receiver)?;
    let mut items = Vec::with_capacity(slots.len());
    for (offset, slot) in slots.iter().enumerate() {
        match slot {
            Some(item) => items.push(*item),
            None => return Err(Raised::missing_array_element(offset + 1).into()),
        }
    }
    Ok(items)
}

/// How the sort family orders two items.
enum Order {
    /// The default: send `compareTo` to the first item.
    CompareTo,
    /// `sortWith`: send `compare(first, second)` to the comparator
    /// (`:2897`).
    With(ObjRef),
}

fn order_of(
    interp: &mut Interp,
    order: &Order,
    left: ObjRef,
    right: ObjRef,
) -> Result<i64, Failure> {
    let caller = interp.caller();
    let answer = match order {
        Order::CompareTo => {
            interp.send_message(left, b"COMPARETO", None, &[Some(right)], caller)?
        }
        Order::With(comparator) => interp.send_message(
            *comparator,
            b"COMPARE",
            None,
            &[Some(left), Some(right)],
            caller,
        )?,
    };
    let name: &[u8] = match order {
        Order::CompareTo => b"COMPARETO",
        Order::With(_) => b"COMPARE",
    };
    let Some(answer) = answer else {
        return Err(Raised::no_result(name).into());
    };
    // **The answer is converted, not read for a sign.** Both comparators put
    // it through `numberValue` and raise for a result that is not a whole
    // number -- `WithSortComparator` 26.903 (`classes/ArrayClass.cpp:2909`)
    // and `compareTo` 26.902 (`classes/ObjectClass.cpp:245`). Measured: a
    // comparator answering `'-1.0'`, `'1.0'` and `'0.0'` sorts `b,a,c` into
    // `a,b,c`, where parsing the text left it untouched; one answering
    // `'abc'` raises, where this crate ran to completion.
    let Some(value) = whole_comparison(interp, answer) else {
        let found = interp.string_value_text(answer);
        return Err(match order {
            Order::CompareTo => Raised::compare_to_result_not_whole(&found).into(),
            Order::With(_) => Raised::compare_result_not_whole(&found).into(),
        });
    };
    Ok(value)
}

/// Upstream's stable merge sort, comparison for comparison.
fn merge_sort(
    interp: &mut Interp,
    order: &Order,
    items: Vec<ObjRef>,
) -> Result<Vec<ObjRef>, Failure> {
    let count = items.len();
    if count <= 1 {
        return Ok(items);
    }
    let mut one = Vec::with_capacity(count + 1);
    one.push(items[0]);
    one.extend(items);
    let mut working = one.clone();
    sort_range(interp, order, &mut one, &mut working, 1, count)?;
    one.remove(0);
    Ok(one)
}

/// `ArrayClass::mergeSort` over the inclusive one-based range `left..=right`.
fn sort_range(
    interp: &mut Interp,
    order: &Order,
    items: &mut [ObjRef],
    working: &mut [ObjRef],
    left: usize,
    right: usize,
) -> Result<(), Failure> {
    // `len <= 10` is upstream's threshold, and it is why nine elements sort
    // by insertion with no merge at all. Bound as `size_t len = right - left
    // + 1` is, rather than folded into the test, so the two read alike.
    let len = right - left + 1;
    if len <= 10 {
        for i in left + 1..=right {
            let current = items[i];
            let mut prev = items[i - 1];
            if order_of(interp, order, current, prev)? < 0 {
                let mut j = i;
                loop {
                    items[j] = prev;
                    j -= 1;
                    if j > left {
                        prev = items[j - 1];
                        if order_of(interp, order, current, prev)? < 0 {
                            continue;
                        }
                    }
                    break;
                }
                items[j] = current;
            }
        }
        return Ok(());
    }
    let mid = (right + left) / 2;
    sort_range(interp, order, items, working, left, mid)?;
    sort_range(interp, order, items, working, mid + 1, right)?;
    merge(interp, order, items, working, left, mid + 1, right)
}

/// `ArrayClass::merge` (`classes/ArrayClass.cpp:2669`): the two partitions
/// `left..mid` and `mid..=right`, merged through `working`.
fn merge(
    interp: &mut Interp,
    order: &Order,
    items: &mut [ObjRef],
    working: &mut [ObjRef],
    left: usize,
    mid: usize,
    right: usize,
) -> Result<(), Failure> {
    let left_end = mid - 1;
    // Already in order: one comparison and no merge.
    if order_of(interp, order, items[left_end], items[mid])? <= 0 {
        return Ok(());
    }
    let mut left_cursor = left;
    let mut right_cursor = mid;
    let mut position = left;
    loop {
        let from_value = items[left_cursor];
        let right_value = items[right_cursor];
        if order_of(interp, order, from_value, right_value)? <= 0 {
            let insertion = find(
                interp,
                order,
                items,
                right_value,
                -1,
                left_cursor + 1,
                left_end,
            )?;
            let count = insertion - left_cursor + 1;
            copy_run(items, left_cursor, working, position, count);
            position += count;
            working[position] = right_value;
            position += 1;
            right_cursor += 1;
            left_cursor = insertion + 1;
        } else {
            let insertion = find(interp, order, items, from_value, 0, right_cursor + 1, right)?;
            let count = insertion - right_cursor + 1;
            copy_run(items, right_cursor, working, position, count);
            position += count;
            working[position] = from_value;
            position += 1;
            left_cursor += 1;
            right_cursor = insertion + 1;
        }
        if !(right >= right_cursor && mid > left_cursor) {
            break;
        }
    }
    if left_cursor < mid {
        copy_run(items, left_cursor, working, position, mid - left_cursor);
    } else {
        // `right - rightCursor + 1` upstream, where both are `size_t` and the
        // loop can leave `rightCursor` one past `right` -- which wraps to
        // zero there and would panic here.
        copy_run(
            items,
            right_cursor,
            working,
            position,
            (right + 1).saturating_sub(right_cursor),
        );
    }
    copy_run(working, left, items, left, right - left + 1);
    Ok(())
}

/// `ArrayClass::find` (`classes/ArrayClass.cpp:2773`): where `value` belongs
/// in the sorted run `left..=right`, by exponential search then bisection.
fn find(
    interp: &mut Interp,
    order: &Order,
    items: &[ObjRef],
    value: ObjRef,
    limit: i64,
    left: usize,
    right: usize,
) -> Result<usize, Failure> {
    let (mut left, mut right) = (left, right);
    let mut check = left;
    let mut delta = 1;
    while check <= right {
        if order_of(interp, order, value, items[check])? > limit {
            left = check + 1;
        } else {
            right = check - 1;
            break;
        }
        check += delta;
        delta *= 2;
    }
    while left <= right {
        check = left.midpoint(right);
        if order_of(interp, order, value, items[check])? > limit {
            left = check + 1;
        } else {
            right = check - 1;
        }
    }
    Ok(left - 1)
}

/// `ArrayClass::arraycopy` (`classes/ArrayClass.cpp:2745`).
fn copy_run(source: &[ObjRef], start: usize, target: &mut [ObjRef], index: usize, count: usize) {
    target[index..index + count].copy_from_slice(&source[start..start + count]);
}

/// Writes `items` back over the receiver's slots, which is what makes the
/// sort family sort **in place**: measured, `p = .Array~of('b','a')` then
/// `q = p~sort` leaves `p` reading `a,b` and `q` is that same array.
fn write_back(interp: &mut Interp, receiver: ObjRef, items: Vec<ObjRef>) -> Result<(), Failure> {
    let receiver = store_of(interp, receiver)?;
    match interp.heap.get_mut(receiver).map(|object| &mut object.body) {
        Some(Body::Array { slots, .. }) => {
            for (slot, item) in slots.iter_mut().zip(items) {
                *slot = Some(item);
            }
            Ok(())
        }
        _ => Err(Loud::receiver_class("a value that is not an array").into()),
    }
}

fn sort_by(
    interp: &mut Interp,
    receiver: ObjRef,
    order: &Order,
) -> Result<Option<ObjRef>, Failure> {
    let before = slots_of(interp, receiver)?.iter().flatten().count();
    let items = dense_items(interp, receiver)?;
    // The whole run is held across every `COMPARE`/`COMPARETO`, each of which
    // runs Rexx that may empty the receiver -- so the items are rooted for
    // the duration rather than living only in the `Vec`.
    for item in &items {
        interp.roots.push_temp(*item);
    }
    let sorted = merge_sort(interp, order, items)?;
    // **Upstream sorts in place, so a comparator that changes the receiver
    // under the sort loses the sort's writes.** This crate sorts a copy, so
    // it has to notice: measured, `sortWith` over a comparator that empties
    // the array leaves it holding nothing on the oracle, where writing the
    // copy back would restore the pre-sort contents. Approximated by the item
    // count, which is what a callback that empties or refills moves; a
    // callback that swaps one item for another is not distinguished and is
    // not measured.
    let after = slots_of(interp, receiver)?.iter().flatten().count();
    if after == before {
        write_back(interp, receiver, sorted)?;
    }
    Ok(Some(receiver))
}

/// `Array~sort` and `Array~stableSort`, both `ArrayClass::stableSortRexx`.
fn native_array_sort(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    sort_by(interp, receiver, &Order::CompareTo)
}

/// `Array~sortWith(comparator)` and `Array~stableSortWith(comparator)`, both
/// `ArrayClass::stableSortWithRexx`.
fn native_array_sort_with(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let comparator = item_argument(args)?;
    sort_by(interp, receiver, &Order::With(comparator))
}

/// Inserts `item` at 0-based `at`, or deletes the slot there when it is
/// `None`, shifting everything after it.
fn array_splice(
    interp: &mut Interp,
    receiver: ObjRef,
    at: usize,
    item: Option<ObjRef>,
) -> Result<(), Failure> {
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

/// A new array of the receiver's own class over `slots`.
fn same_class_array(
    interp: &mut Interp,
    receiver: ObjRef,
    slots: Vec<Option<ObjRef>>,
) -> Result<ObjRef, Failure> {
    let store = interp.alloc_with(BehaviourId::ARRAY, Body::array(slots));
    interp.roots.push_temp(store);
    if interp.array_slots(receiver).is_some() {
        return Ok(store);
    }
    let class = interp
        .class_of_value(receiver)
        .ok_or_else(|| Failure::from(Loud::receiver_class("a value that is not an array")))?;
    let object = new_instance(interp, class)?;
    interp.roots.push_temp(object);
    let scope = store_scope(interp);
    interp.set_pool_variable(object, scope, QUEUE_ITEMS, store);
    Ok(object)
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
fn array_splice_slot(
    interp: &mut Interp,
    receiver: ObjRef,
    at: usize,
    item: Option<ObjRef>,
) -> Result<(), Failure> {
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

/// [`array_splice_slot`], except that a trailing empty slot absorbs the shift
/// instead of the array growing.
fn splice_absorbing_slack(
    interp: &mut Interp,
    receiver: ObjRef,
    at: usize,
    item: Option<ObjRef>,
) -> Result<(), Failure> {
    // **Slack the array ALREADY had**, not slack the insert just created.
    // Measured: `.Array~new(4)~insert('j')` leaves size 4, where
    // `.Array~of('x','y')~insert` -- whose last slot is occupied -- leaves
    // size 3.
    let slack = slots_of(interp, receiver)?.last() == Some(&None);
    array_splice_slot(interp, receiver, at, item)?;
    if !slack {
        return Ok(());
    }
    let store = store_of(interp, receiver)?;
    match interp.heap.get_mut(store).map(|object| &mut object.body) {
        Some(Body::Array { slots, .. }) => {
            if slots.last() == Some(&None) {
                slots.pop();
            }
            Ok(())
        }
        _ => Err(Loud::receiver_class("a value that is not an array").into()),
    }
}

/// Grows `receiver` to `length` empty slots, for an `insert` past the end.
fn array_grow(interp: &mut Interp, receiver: ObjRef, length: usize) -> Result<(), Failure> {
    let receiver = store_of(interp, receiver)?;
    match interp.heap.get_mut(receiver).map(|object| &mut object.body) {
        Some(Body::Array { slots, .. }) => {
            slots.resize(length, None);
            Ok(())
        }
        _ => Err(Loud::receiver_class("a value that is not an array").into()),
    }
}

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

// ---- List ----

/// A `List` index argument, whose position in the error is **two** whatever
/// its position in the argument list is.
fn list_index_argument(args: &[Option<ObjRef>], at: usize) -> Result<ObjRef, Failure> {
    args.get(at)
        .copied()
        .flatten()
        .ok_or_else(|| Raised::missing_method_argument(2).into())
}

/// A `List`'s three pool entries, under the `List` class as scope: the items
/// in order, the handle of each, and the stack of handles removal has freed.
const LIST_ITEMS: &[u8] = b"ITEMS";
const LIST_HANDLES: &[u8] = b"HANDLES";
const LIST_FREE: &[u8] = b"FREE";

/// The `List` class, which is the scope its three entries are bound under.
fn list_scope(interp: &mut Interp) -> ObjRef {
    interp
        .classes()
        .lookup("List")
        .expect("List is a native class")
}

/// A list's items, its handles and its free stack.
fn list_state(interp: &mut Interp, receiver: ObjRef) -> Result<(ObjRef, ObjRef, ObjRef), Failure> {
    let scope = list_scope(interp);
    if let (Some(items), Some(handles), Some(free)) = (
        pool_variable(interp, receiver, scope, LIST_ITEMS),
        pool_variable(interp, receiver, scope, LIST_HANDLES),
        pool_variable(interp, receiver, scope, LIST_FREE),
    ) {
        return Ok((items, handles, free));
    }
    // Built on demand for [`store_of`]'s reason: upstream the contents belong
    // to the allocated object, so a subclass `INIT` that does not forward
    // still has them. Measured, a `List` subclass whose `INIT` only sets an
    // exposed variable answers `items 1` after `append('a')`. The three are
    // only ever written together, so a partial set is not a state this can
    // meet.
    if !is_list(interp, receiver) {
        return Err(Loud::receiver_class("a value that is not a list").into());
    }
    let mut built = Vec::with_capacity(3);
    for name in [LIST_ITEMS, LIST_HANDLES, LIST_FREE] {
        let store = interp.alloc_with(BehaviourId::ARRAY, Body::array(Vec::new()));
        interp.roots.push_temp(store);
        interp.set_pool_variable(receiver, scope, name, store);
        built.push(store);
    }
    Ok((built[0], built[1], built[2]))
}

/// Whether `receiver` is a `List` or something deriving from one --
/// [`is_queue`]'s question for the other family.
fn is_list(interp: &mut Interp, receiver: ObjRef) -> bool {
    let Some(class) = interp.class_of_value(receiver) else {
        return false;
    };
    let Some(list) = interp.classes().lookup("List") else {
        return false;
    };
    interp.classes().is_a(class, list)
}

/// Where in the list `handle` sits, or `None` for an index the list does not
/// hold.
fn list_position(
    interp: &mut Interp,
    receiver: ObjRef,
    handle: ObjRef,
) -> Result<Option<usize>, Failure> {
    let Some(wanted) = super::unsigned_index(interp, handle) else {
        let written = interp.to_text(handle);
        return Err(Raised::incorrect_list_index(&written).into());
    };
    let (items, handles, _) = list_state(interp, receiver)?;
    let held = array_slots_owned(interp, handles)?;
    let stored = array_slots_owned(interp, items)?;
    for (offset, slot) in held.iter().enumerate() {
        if let Some(slot) = *slot
            && super::unsigned_index(interp, slot) == Some(wanted)
        {
            // `isIndexValid` is `isInUse`
            // (`classes/support/ListContents.hpp:235`), so `validateIndex`
            // answers NoLink for an entry holding nothing and every lookup
            // reads it as absent. Measured over `l~put(, 1)`: `hasIndex(1)`
            // is `0`, `at(1)` and `next(1)` are `.nil`, and `section(1,1)`
            // raises -- while the chain still walks through the entry, which
            // is [`list_pairs`]'s business and not this function's.
            return Ok(stored.get(offset).copied().flatten().map(|_| offset));
        }
    }
    Ok(None)
}

/// The handle a new item takes: the last one removal freed, or a fresh one.
fn list_next_handle(interp: &mut Interp, receiver: ObjRef) -> Result<ObjRef, Failure> {
    let (_, handles, free) = list_state(interp, receiver)?;
    let stack = array_slots_owned(interp, free)?;
    if let Some(Some(reused)) = stack.last().copied() {
        let last = stack.len() - 1;
        array_splice(interp, free, last, None)?;
        return Ok(reused);
    }
    // Fresh handles count from zero and never repeat while the list holds
    // them, so the next one is the highest ever issued plus one. The highest
    // is tracked as the count of handles ever issued, which is the live ones
    // plus the freed ones.
    let live = array_slots_owned(interp, handles)?.len();
    Ok(interp.counted(live))
}

/// Puts `item` at `at` with a fresh handle, and answers the handle.
fn list_insert_at(
    interp: &mut Interp,
    receiver: ObjRef,
    at: usize,
    item: Option<ObjRef>,
) -> Result<ObjRef, Failure> {
    let handle = list_next_handle(interp, receiver)?;
    let (items, handles, _) = list_state(interp, receiver)?;
    array_splice_slot(interp, items, at, item)?;
    array_splice_slot(interp, handles, at, Some(handle))?;
    Ok(handle)
}

/// `List~init([size])`: the optional capacity, and the three entries.
fn native_list_init(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    super::optional_length_argument(interp, args, 0)?;
    // **`INIT` sent a second time keeps the contents.** `ListClass::initRexx`
    // takes the optional size and nothing else; the entries belong to the
    // allocated object. Measured, `l~init` on a two-item list still answers
    // `items 2`, where rebuilding the entries here answered `0`.
    list_state(interp, receiver)?;
    Ok(None)
}

/// `List~append(item)`: adds at the end and answers the new handle.
fn native_list_append(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let item = item_argument(args)?;
    let (items, _, _) = list_state(interp, receiver)?;
    let at = array_slots(interp, items)?.len();
    Ok(Some(list_insert_at(interp, receiver, at, Some(item))?))
}

/// `List~insert(item [, index])`: after `index`, or at the end when the index
/// is omitted, or at the front when it is `.nil`.
fn native_list_insert(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    // The value is optional -- `insertRexx` has no `requiredArgument` for it
    // (`classes/ListClass.cpp:454`). Measured: `.List~new~insert` answers `0`
    // at rc 0.
    let item = args.first().copied().flatten();
    let (items, _, _) = list_state(interp, receiver)?;
    let at = match args.get(1).copied() {
        Some(Some(index)) if index == ObjRef::NIL => 0,
        Some(Some(index)) => match list_position(interp, receiver, index)? {
            Some(offset) => offset + 1,
            None => {
                let written = interp.to_text(index);
                return Err(Raised::incorrect_list_index(&written).into());
            }
        },
        Some(None) | None => array_slots(interp, items)?.len(),
    };
    Ok(Some(list_insert_at(interp, receiver, at, item)?))
}

/// `List~at(index)` and `List~[index]`: the item at that handle, or `.nil`.
fn native_list_at(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    // **Argument TWO, not one.** `ListClass::getRexx` reaches
    // `validateIndex(argIndex, ARG_TWO)` (`classes/ListClass.cpp:381`), so a
    // bare `~at` reports `argument 2 is required` even though the method
    // takes one argument. Measured on the oracle.
    let handle = list_index_argument(args, 0)?;
    let Some(at) = list_position(interp, receiver, handle)? else {
        return Ok(Some(ObjRef::NIL));
    };
    let (items, _, _) = list_state(interp, receiver)?;
    Ok(Some(match array_slots(interp, items)?.get(at) {
        Some(Some(item)) => *item,
        Some(None) | None => ObjRef::NIL,
    }))
}

/// `List~put(item, index)` and `List~[index] = item`: replaces at that
/// handle, and 93.918 for a handle the list does not hold -- the same error
/// `Queue~put` raises, which is why the constructor is shared.
fn native_list_put(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    // The INDEX is what is required, and it is argument two:
    // `requiredIndex(argIndex, ARG_TWO)` (`classes/ListClass.cpp:349`). The
    // value is not checked at all -- measured, `l~put(, handle)` succeeds and
    // leaves the entry holding nothing.
    let handle = list_index_argument(args, 1)?;
    let item = args.first().copied().flatten();
    let Some(at) = list_position(interp, receiver, handle)? else {
        let written = interp.to_text(handle);
        return Err(Raised::incorrect_list_index(&written).into());
    };
    let (items, _, _) = list_state(interp, receiver)?;
    match interp.heap.get_mut(items).map(|object| &mut object.body) {
        Some(Body::Array { slots, .. }) => {
            slots[at] = item;
            Ok(None)
        }
        _ => Err(Loud::receiver_class("a value that is not a list").into()),
    }
}

/// `List~remove(index)` and `List~delete(index)`, one body upstream: takes
/// the item out, frees the handle and answers the item, or `.nil` for a
/// handle the list does not hold.
fn native_list_remove(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let handle = item_argument(args)?;
    let Some(at) = list_position(interp, receiver, handle)? else {
        return Ok(Some(ObjRef::NIL));
    };
    list_take(interp, receiver, at)
}

/// Removes the entry at `at`, frees its handle and answers the item.
fn list_take(interp: &mut Interp, receiver: ObjRef, at: usize) -> Result<Option<ObjRef>, Failure> {
    let (items, handles, free) = list_state(interp, receiver)?;
    let item = match array_slots(interp, items)?.get(at) {
        Some(Some(item)) => *item,
        Some(None) | None => ObjRef::NIL,
    };
    let freed = match array_slots(interp, handles)?.get(at) {
        Some(Some(handle)) => Some(*handle),
        Some(None) | None => None,
    };
    array_splice(interp, items, at, None)?;
    array_splice(interp, handles, at, None)?;
    if let Some(freed) = freed {
        let top = array_slots(interp, free)?.len();
        array_splice_slot(interp, free, top, Some(freed))?;
    }
    Ok(Some(item))
}

/// `List~removeItem(item)`: removes the first entry holding it.
fn native_list_remove_item(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let wanted = item_argument(args)?;
    let (items, _, _) = list_state(interp, receiver)?;
    let held = array_slots_owned(interp, items)?;
    for (offset, slot) in held.iter().enumerate() {
        if let Some(slot) = *slot
            && same_item(interp, wanted, slot)?
        {
            return list_take(interp, receiver, offset);
        }
    }
    Ok(Some(ObjRef::NIL))
}

/// The list's entries in chain order: every handle, each with the item it
/// holds or `None` for an entry holding nothing.
fn list_pairs(
    interp: &mut Interp,
    receiver: ObjRef,
) -> Result<Vec<(ObjRef, Option<ObjRef>)>, Failure> {
    let (items, handles, _) = list_state(interp, receiver)?;
    let items = array_slots_owned(interp, items)?;
    let handles = array_slots_owned(interp, handles)?;
    let pairs: Vec<(ObjRef, Option<ObjRef>)> = handles
        .into_iter()
        .zip(items)
        .filter_map(|(handle, item)| Some((handle?, item)))
        .collect();
    // Rooted for [`ordered_pairs`]'s reason: a caller sends `==` per pair and
    // the callback may empty the list.
    for (handle, item) in &pairs {
        interp.roots.push_temp(*handle);
        if let Some(item) = *item {
            interp.roots.push_temp(item);
        }
    }
    Ok(pairs)
}

fn native_list_all_items(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let items = list_pairs(interp, receiver)?
        .into_iter()
        .map(|(_, item)| item)
        .collect();
    Ok(Some(array_of_slots(interp, items)))
}

fn native_list_all_indexes(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let handles = list_pairs(interp, receiver)?
        .into_iter()
        .map(|(handle, _)| handle)
        .collect();
    Ok(Some(array_of(interp, handles)))
}

fn native_list_items(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let count = list_pairs(interp, receiver)?.len();
    Ok(Some(interp.counted(count)))
}

fn native_list_is_empty(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let empty = list_pairs(interp, receiver)?.is_empty();
    Ok(Some(crate::eval::logical(empty)))
}

/// `List~empty`: drops every entry, after which the handles start again from
/// zero.
fn native_list_empty(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let (items, handles, free) = list_state(interp, receiver)?;
    for store in [items, handles, free] {
        match interp.heap.get_mut(store).map(|object| &mut object.body) {
            Some(Body::Array { slots, .. }) => slots.clear(),
            _ => return Err(Loud::receiver_class("a value that is not a list").into()),
        }
    }
    // Answers the receiver, as `Array~empty` does -- measured, `say l~empty`
    // prints `a List` at rc 0.
    Ok(Some(receiver))
}

/// `List~first`, `~last`, `~firstItem`, `~lastItem`: the handle or the item at
/// either end, or `.nil` for an empty list.
fn list_end(
    interp: &mut Interp,
    receiver: ObjRef,
    take_last: bool,
    want_item: bool,
) -> Result<Option<ObjRef>, Failure> {
    let pairs = list_pairs(interp, receiver)?;
    let found = if take_last {
        pairs.last()
    } else {
        pairs.first()
    };
    let Some((handle, item)) = found.copied() else {
        return Ok(Some(ObjRef::NIL));
    };
    Ok(Some(match want_item {
        // `firstItem` over an entry holding nothing is `.nil`, not the first
        // entry that holds something -- measured, and likewise `lastItem`.
        true => item.unwrap_or(ObjRef::NIL),
        false => handle,
    }))
}

fn native_list_first(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    list_end(interp, receiver, false, false)
}

fn native_list_last(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    list_end(interp, receiver, true, false)
}

fn native_list_first_item(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    list_end(interp, receiver, false, true)
}

fn native_list_last_item(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    list_end(interp, receiver, true, true)
}

/// `List~next(index)` and `List~previous(index)`: the neighbouring handle, or
/// `.nil` past either end and for a handle the list does not hold.
fn list_step(
    interp: &mut Interp,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
    forward: bool,
) -> Result<Option<ObjRef>, Failure> {
    let handle = item_argument(args)?;
    let Some(at) = list_position(interp, receiver, handle)? else {
        return Ok(Some(ObjRef::NIL));
    };
    let pairs = list_pairs(interp, receiver)?;
    let neighbour = if forward {
        at.checked_add(1).filter(|next| *next < pairs.len())
    } else {
        at.checked_sub(1)
    };
    Ok(Some(match neighbour {
        Some(offset) => pairs[offset].0,
        None => ObjRef::NIL,
    }))
}

fn native_list_next(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    list_step(interp, receiver, args, true)
}

fn native_list_previous(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    list_step(interp, receiver, args, false)
}

/// `List~hasIndex(index)`.
fn native_list_has_index(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let handle = item_argument(args)?;
    let held = list_position(interp, receiver, handle)?.is_some();
    Ok(Some(crate::eval::logical(held)))
}

/// `List~hasItem(item)`.
fn native_list_has_item(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let wanted = item_argument(args)?;
    for (_, item) in list_pairs(interp, receiver)? {
        if let Some(item) = item
            && same_item(interp, wanted, item)?
        {
            return Ok(Some(crate::eval::logical(true)));
        }
    }
    Ok(Some(crate::eval::logical(false)))
}

/// `List~index(item)`: the handle of the first entry holding it, or `.nil`.
fn native_list_index(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let wanted = item_argument(args)?;
    for (handle, item) in list_pairs(interp, receiver)? {
        if let Some(item) = item
            && same_item(interp, wanted, item)?
        {
            return Ok(Some(handle));
        }
    }
    Ok(Some(ObjRef::NIL))
}

/// `List~section(index, count)`: a new `List` over that run, which is a
/// `List` and not an `Array`.
fn native_list_section(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let handle = item_argument(args)?;
    let Some(at) = list_position(interp, receiver, handle)? else {
        let written = interp.to_text(handle);
        return Err(Raised::incorrect_list_index(&written).into());
    };
    let pairs = list_pairs(interp, receiver)?;
    let available = pairs.len() - at;
    let count = match args.get(1).copied().flatten() {
        Some(count) => super::array_size_argument(interp, Some(count), 2)?.min(available),
        None => available,
    };
    // **Always a `List`, and never the receiver's class.**
    // `ListClass::section` is an unconditional `new ListClass`
    // (`classes/ListClass.cpp:429`), where `Array` and `Queue` both build
    // from the receiver's. Measured, with one subclass of each:
    // `.MyList~of('a','b','c')~section(0,2)~class~id` is `List`, while the
    // `Queue` subclass answers `MYQ` and the `Array` subclass `MYARR`.
    let class = list_scope(interp);
    let section = new_instance(interp, class)?;
    interp.roots.push_temp(section);
    let caller = interp.caller();
    interp.send_message(section, super::INIT, None, &[], caller)?;
    for (_, item) in pairs.into_iter().skip(at).take(count) {
        let at = list_pairs(interp, section)?.len();
        list_insert_at(interp, section, at, item)?;
    }
    Ok(Some(section))
}

/// `List~supplier`: a `Supplier` over the items and their handles.
fn native_list_supplier(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let pairs = list_pairs(interp, receiver)?;
    let items: Vec<Option<ObjRef>> = pairs.iter().map(|(_, item)| *item).collect();
    let handles: Vec<ObjRef> = pairs.into_iter().map(|(handle, _)| handle).collect();
    let items = array_of_slots(interp, items);
    let handles = array_of(interp, handles);
    new_supplier(interp, items, handles).map(Some)
}

/// `List~makeArray`: `ArrayClass::makeArray`'s side of the virtual -- items,
/// not handles.
fn native_list_make_array(
    interp: &mut Interp,
    cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    native_list_all_items(interp, cleared, receiver, args)
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

/// The collection classes' primitive methods, chained into
/// `ObjectModel::build`.
pub(super) const NATIVE_METHODS: &[(&str, &str, Arity, NativeMethod)] = &[
    ("List", "INIT", Arity::Fixed(1), native_list_init),
    ("List", "APPEND", Arity::Fixed(1), native_list_append),
    ("List", "INSERT", Arity::Fixed(2), native_list_insert),
    ("List", "AT", Arity::Fixed(1), native_list_at),
    ("List", "[]", Arity::Fixed(1), native_list_at),
    ("List", "PUT", Arity::Fixed(2), native_list_put),
    ("List", "[]=", Arity::Fixed(2), native_list_put),
    ("List", "REMOVE", Arity::Fixed(1), native_list_remove),
    ("List", "DELETE", Arity::Fixed(1), native_list_remove),
    (
        "List",
        "REMOVEITEM",
        Arity::Fixed(1),
        native_list_remove_item,
    ),
    ("List", "ALLITEMS", Arity::Fixed(0), native_list_all_items),
    (
        "List",
        "ALLINDEXES",
        Arity::Fixed(0),
        native_list_all_indexes,
    ),
    ("List", "ITEMS", Arity::Fixed(0), native_list_items),
    ("List", "ISEMPTY", Arity::Fixed(0), native_list_is_empty),
    ("List", "EMPTY", Arity::Fixed(0), native_list_empty),
    ("List", "FIRST", Arity::Fixed(0), native_list_first),
    ("List", "LAST", Arity::Fixed(0), native_list_last),
    ("List", "FIRSTITEM", Arity::Fixed(0), native_list_first_item),
    ("List", "LASTITEM", Arity::Fixed(0), native_list_last_item),
    ("List", "NEXT", Arity::Fixed(1), native_list_next),
    ("List", "PREVIOUS", Arity::Fixed(1), native_list_previous),
    ("List", "HASINDEX", Arity::Fixed(1), native_list_has_index),
    ("List", "HASITEM", Arity::Fixed(1), native_list_has_item),
    ("List", "INDEX", Arity::Fixed(1), native_list_index),
    ("List", "SECTION", Arity::Fixed(2), native_list_section),
    ("List", "SUPPLIER", Arity::Fixed(0), native_list_supplier),
    ("List", "MAKEARRAY", Arity::Fixed(0), native_list_make_array),
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
    ("Array", "SORT", Arity::Fixed(0), native_array_sort),
    ("Array", "SORTWITH", Arity::Fixed(1), native_array_sort_with),
    ("Array", "STABLESORT", Arity::Fixed(0), native_array_sort),
    (
        "Array",
        "STABLESORTWITH",
        Arity::Fixed(1),
        native_array_sort_with,
    ),
    ("Array", "APPEND", Arity::Fixed(1), native_array_append),
    ("Array", "DELETE", Arity::Fixed(1), native_array_delete),
    (
        "Array",
        "DIMENSIONS",
        Arity::Fixed(0),
        native_array_dimensions,
    ),
    ("Array", "FILL", Arity::Fixed(1), native_array_fill),
    ("Array", "FIRST", Arity::Fixed(0), native_array_first),
    (
        "Array",
        "FIRSTITEM",
        Arity::Fixed(0),
        native_array_first_item,
    ),
    ("Array", "INSERT", Arity::Fixed(2), native_array_insert),
    ("Array", "LAST", Arity::Fixed(0), native_array_last),
    ("Array", "LASTITEM", Arity::Fixed(0), native_array_last_item),
    ("Array", "NEXT", Arity::Counted, native_array_next),
    ("Array", "PREVIOUS", Arity::Counted, native_array_previous),
    ("Array", "SECTION", Arity::Fixed(2), native_array_section),
    (
        "Array",
        "ALLINDEXES",
        Arity::Fixed(0),
        native_array_all_indexes,
    ),
    ("Array", "ALLITEMS", Arity::Fixed(0), native_array_all_items),
    ("Array", "EMPTY", Arity::Fixed(0), native_array_empty),
    ("Array", "HASINDEX", Arity::Counted, native_array_has_index),
    ("Array", "HASITEM", Arity::Fixed(1), native_array_has_item),
    ("Array", "INDEX", Arity::Fixed(1), native_array_index),
    ("Array", "ISEMPTY", Arity::Fixed(0), native_array_is_empty),
    (
        "Array",
        "MAKEARRAY",
        Arity::Fixed(0),
        native_array_make_array,
    ),
    ("Array", "REMOVE", Arity::Counted, native_array_remove),
    (
        "Array",
        "REMOVEITEM",
        Arity::Fixed(1),
        native_array_remove_item,
    ),
    ("Array", "SUPPLIER", Arity::Fixed(0), native_array_supplier),
    (
        "Supplier",
        "AVAILABLE",
        Arity::Fixed(0),
        native_supplier_available,
    ),
    ("Supplier", "INIT", Arity::Fixed(2), native_supplier_init),
    ("Supplier", "INDEX", Arity::Fixed(0), native_supplier_index),
    ("Supplier", "ITEM", Arity::Fixed(0), native_supplier_item),
    ("Supplier", "NEXT", Arity::Fixed(0), native_supplier_next),
];
