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

//! `Array`'s collection surface: the methods it shares with the other
//! collections (`allItems`, `hasItem`, `supplier`, ...) and its own (`first`,
//! `insert`, `section`, ...), over the Array-shaped store collection.rs keeps.

use super::collection::{
    QUEUE_ITEMS, append_slot, array_grow, array_of, array_splice, array_splice_slot, dimensions_of,
    item_argument, last_item, new_supplier, occupied, ordered_pairs, position_in, queue_bound,
    same_item, slots_of, store_of, store_scope, subscript_object,
};
use super::{
    Arity, BehaviourId, Body, Cleared, Failure, IndexUse, Interp, Loud, NativeMethod, ObjRef,
    Raised, array_slots, new_instance,
};

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
pub(in crate::dispatch) fn native_array_delete(
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

/// `Array`'s collection-surface methods, chained into `ObjectModel::build`
/// after its sort family.
pub(in crate::dispatch) const NATIVE_METHODS: &[(&str, &str, Arity, NativeMethod)] = &[
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
];
