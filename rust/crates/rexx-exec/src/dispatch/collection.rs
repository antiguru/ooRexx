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
//!
//! A child of [`super`] rather than a sibling, for the reason its `mod native`
//! comment gives: the rows point at `NativeMethod`s, whose parameter list
//! names types only that module can. [`NATIVE_METHODS`] is chained into
//! `ObjectModel::build` beside [`super::NATIVE_METHODS`] rather than merged,
//! so this file's rows and bodies stay together.
//!
//! **Empty on purpose at Phase 5g Task 0.** The registration path is landed
//! before any body so that the first task to add one is debugging its own
//! code and not the wiring; a chain that silently registers nothing looks
//! exactly like a chain that works.
//!
//! # What a row here may not be read as
//!
//! **The name a row binds is a citation, not a body identity.** Upstream,
//! `Setup.cpp` writes `Set`'s `HasItem` as `IdentityTable::hasIndexRexx` and
//! `IdentityTableClass.hpp` declares no such member -- it is
//! `HashCollection::hasIndexRexx` reached through C++ inheritance. So two
//! tokens there can name one function, and separately one token can reach
//! three behaviours, selected by the contents class the receiver allocated:
//! `HashCollection::putRexx` passes `IndexOnlyHashCollection`'s validation on
//! a `Set` or `Bag` and reaches `MultiValueContents::put` on a `Relation` or
//! `Bag`. A Rust body shared between two rows here is a claim that the
//! *behaviour* is shared, which the upstream token does not establish either
//! way.
//!
//! `corpus/collection-scopes.tsv` is where that join is recorded, and its own
//! test re-derives it.

use super::{
    Arity, ArrayArgument, BehaviourId, Body, Cleared, Decoded, Failure, IndexUse, Interp, Loud,
    NativeMethod, ObjRef, Raised, array_argument, array_dimensions, array_position, array_slots,
    array_slots_owned, new_instance,
};
use rexx_parse::Operator;

/// The `Supplier` state's three names, bound in the receiver's own pool under
/// the `Supplier` class as scope.
///
/// A pool rather than a new `Body` variant: `ScopePools` is already walked by
/// the collector (`Body::trace`'s `Instance` arm), and a supplier holds two
/// arrays, so a payload the collector could not see would be exactly the
/// use-after-free spec D101 exists to prevent.
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
///
/// **One scope for every holder**, so that `Queue`, `CircularQueue` and any
/// user subclass of `Array` are all reached by the same [`store_of`].
const QUEUE_ITEMS: &[u8] = b"ITEMS";

/// The scope the Array-shaped store is bound under.
fn store_scope(interp: &mut Interp) -> ObjRef {
    interp
        .classes()
        .lookup("Array")
        .expect("Array is a native class")
}

// ---- the contents protocol ----

/// The array that actually holds `receiver`'s slots.
///
/// **A `Queue` is not a `Body::Array` and cannot be one.** A `Body::Array`
/// resolves to `Primitive::Array` wherever it is asked, so an object carrying
/// one answers `.Array` for `~class` -- there is nowhere in that body to say
/// which class it belongs to. Upstream has the opposite arrangement:
/// `Setup.cpp`'s `InheritInstanceMethods(Array)` copies `Array`'s whole
/// native behaviour into `Queue`, so one C++ body serves both receivers.
///
/// This function is what buys the same thing here. A `Queue` is an ordinary
/// instance whose pool holds an `Array`, and every body written against
/// `Array` reaches it by asking for the store rather than for the receiver.
/// Measured before it existed: `.Queue~new~allItems` refused with
/// `a message send to a value that is not an array` rather than with the
/// unimplemented message, which is the shared registration already in place
/// and only the store missing.
///
/// The alternative -- widening `Body::Array` with a class field -- costs
/// every array in the heap eight bytes to serve two classes, against
/// `body.rs`'s own size assertion. This costs a pool lookup on the collection
/// methods of one class.
fn store_of(interp: &mut Interp, receiver: ObjRef) -> Result<ObjRef, Failure> {
    if interp.array_slots(receiver).is_some() {
        return Ok(receiver);
    }
    let scope = store_scope(interp);
    pool_variable(interp, receiver, scope, QUEUE_ITEMS)
        .ok_or_else(|| Loud::receiver_class("a value that is not an array").into())
}

/// Every (index, item) pair an ordered receiver holds, in the store's own
/// order, holes skipped.
///
/// **The index is an object and not a number**, because for a
/// multi-dimensional array it is an `Array` of the coordinates: measured on
/// the oracle, `.Array~new(2,3)` with `[1,1]` and `[2,3]` set answers
/// `allIndexes` of two `Array`s reading `1,1` and `2,3`. Nothing about a
/// one-dimensional array shows that, which is why the protocol carries an
/// `ObjRef` here rather than a `usize`.
///
/// **An explicit `.nil` is an item and a hole is not.** Measured: an array
/// with `[1]`, `[3]` and `[5] = .nil` answers `size 5`, `items 3`,
/// `allIndexes 1,3,5`.
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
        pairs.push((index, item));
    }
    Ok(pairs)
}

/// The index object for the flat 0-based `offset` of an array shaped
/// `dimensions`: the 1-based position for a single dimension, and an `Array`
/// of coordinates for more than one.
///
/// Row-major, which is what the oracle's own layout is: measured, `[2,3]` of
/// a `2x3` array is flat position 6.
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
fn array_of(interp: &mut Interp, items: Vec<ObjRef>) -> ObjRef {
    let slots = items.into_iter().map(Some).collect();
    let array = interp.alloc_with(BehaviourId::ARRAY, Body::array(slots));
    interp.roots.push_temp(array);
    array
}

/// Whether `left` and `right` are the same item to a collection.
///
/// **A send and not a comparison of handles.** `ArrayClass::hasItemRexx`
/// reaches `equalValue`, which for anything but a primitive is the `==`
/// message, and a user class may override it. Measured on the oracle: an
/// array holding one `.K~new` answers `hasItem(.K~new)` as `1` when `K`
/// defines `::METHOD "==" return 1`.
///
/// Measured for the primitives too, and none of the obvious readings is
/// right: `'1'` matches `1`, `' 2'` does **not** match `2`, and `1` does not
/// match `1.0` -- so it is neither byte equality of the source spelling nor
/// numeric equality, but `==` on string values.
fn same_item(interp: &mut Interp, left: ObjRef, right: ObjRef) -> Result<bool, Failure> {
    let answer = interp.apply_binary(Operator::StrictEqual, left, right)?;
    let text = interp.to_text(answer);
    Ok(crate::eval::logical_value(&text).unwrap_or(false))
}

/// The one item argument a collection method takes, which is required.
///
/// 93.903 and **not** 93.901: measured, `(1,2)~hasItem()`, `~index()` and
/// `~removeItem()` all report `Missing argument in method; argument 1 is
/// required.` while `~remove()`, `~hasIndex()` and `~at()` report `Not enough
/// arguments for method; 1 expected.` The two families raise different errors
/// and no zero-argument probe can tell them apart, because both are loud
/// until the bodies exist.
fn item_argument(args: &[Option<ObjRef>]) -> Result<ObjRef, Failure> {
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
///
/// `ArrayClass::makeArray` answers `allItems()`. **This body is not shared
/// with the mapped classes**, whose `HashCollection::makeArray` answers
/// `allIndexes()` instead -- measured, a `Directory` holding `k1`/`k2`
/// answers `makeArray` as `k1,k2` where an `Array` answers its items. One
/// token upstream, one body per store here.
fn native_array_make_array(
    interp: &mut Interp,
    cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    native_array_all_items(interp, cleared, receiver, args)
}

/// `Array~isEmpty`: whether the array holds no item -- `ArrayClass::isEmptyRexx`.
///
/// **Not `size == 0`.** Measured, `.Array~of('x','y')~empty` leaves `size 2`,
/// `items 0` and `isEmpty 1`.
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
///
/// Measured: `size` and `dimensions` both survive, so this clears the slots
/// rather than replacing the body.
///
/// **It answers the receiver**, which `ArrayClass::empty`'s own
/// `return this;` (`classes/ArrayClass.cpp:676`) is. A body answering nothing
/// passes any witness that calls `~empty` as a statement and reddens the
/// moment one reads its value -- measured, `say a~empty` is a blank line at
/// rc 0 on the oracle and `91.999 Message "EMPTY" did not return a result.`
/// at rc 165 without this.
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
        if same_item(interp, item, wanted)? {
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
        if same_item(interp, item, wanted)? {
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
///
/// **The array does not shrink.** Measured, removing index 1 of a size-5
/// array leaves `size 5` and `items` one lower.
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
        if same_item(interp, item, wanted)? {
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
fn new_supplier(interp: &mut Interp, items: ObjRef, indexes: ObjRef) -> Result<ObjRef, Failure> {
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

/// How many pairs the supplier still has, which is the shorter of its two
/// arrays measured from the current position.
///
/// **The two arrays need not be the same length.** Measured, a supplier over
/// one item and two indexes constructs and answers `available 1`.
fn supplier_remaining(interp: &mut Interp, receiver: ObjRef) -> Result<usize, Failure> {
    let (items, indexes, position) = supplier_state(interp, receiver)?;
    let items = array_slots(interp, items)?.len();
    let indexes = array_slots(interp, indexes)?.len();
    Ok(items.min(indexes).saturating_sub(position))
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
///
/// Past the end both raise **93.937**, and so does `~next`. Measured on the
/// oracle at rc 163: `No more supplier items available.`
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
///
/// **The items come first.** Measured, `.Supplier~new(.Array~of('i1'),
/// .Array~of('x1'))` answers `~item` as `i1` and `~index` as `x1`.
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
        // Validated for the argument error it raises, and then the array
        // OBJECT is what is kept: the supplier holds the arrays it was given.
        array_argument(interp, value, ArrayArgument::Positional)?;
        sides.push(value);
    }
    store_supplier(interp, receiver, sides[0], sides[1], 0);
    Ok(None)
}

// ---- Array's own surface ----

/// `ArrayClass::checkMultiDimensional` (`classes/ArrayClass.cpp:426`): the
/// four methods that only work on a single-dimensional array.
///
/// Its callers upstream are `APPEND`, `INSERT`, `DELETE` and `SECTION`, and
/// nothing else -- `fill`, `first`, `next` and the rest are happy with any
/// shape. Measured at rc 163: `.Array~new(2,3)~delete(1)` reports 93.954.
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
///
/// **Not the item.** `firstItem`/`lastItem` are those, and on a sparse array
/// the two cannot coincide: measured, an array holding `[1]`, `[3]`, `[5]`
/// answers `first 1 last 5` against `firstItem p lastItem t`.
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
///
/// **The starting index need not hold anything.** Measured on an array
/// holding `[1]`, `[3]`, `[5]`: `next(2)` is `3` and `previous(4)` is `3`.
/// Past either end is `.nil` rather than an error.
fn array_step(
    interp: &mut Interp,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
    forward: bool,
) -> Result<Option<ObjRef>, Failure> {
    let receiver = store_of(interp, receiver)?;
    let Some(position) = position_in(interp, receiver, args, IndexUse::Get)? else {
        return Ok(Some(ObjRef::NIL));
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
    let at = slots_of(interp, receiver)?.len();
    array_splice(interp, receiver, at, Some(item))?;
    Ok(Some(interp.counted(at + 1)))
}

/// `Array~insert(item [, index])`: puts `item` **after** `index` and shifts
/// the rest along, answering the index it landed on --
/// `ArrayClass::insertRexx`.
///
/// Measured: `.Array~of('x','y','z')~insert('q', 1)` answers `2` and leaves
/// `x,q,y,z`; with no index it appends and answers the new last index; index
/// `0` is `93.907`, not the front; and an index past the end extends, so
/// `insert('c', 9)` on a size-2 array answers `10` and leaves size 10.
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
        Some(Some(index)) => super::positive_index(interp, index, 2)?,
        // Omitted: after the last OCCUPIED slot, not the end of the array.
        Some(None) | None => occupied(interp, receiver)?
            .last()
            .map_or(0, |last| last + 1),
    };
    let length = slots_of(interp, receiver)?.len();
    if at > length {
        array_grow(interp, receiver, at)?;
    }
    array_splice_slot(interp, receiver, at, item)?;
    Ok(Some(interp.counted(at + 1)))
}

/// `Array~delete(index...)`: takes the slot out, closes the gap and answers
/// the item -- `ArrayClass::deleteRexx`.
///
/// **Not `remove`.** Measured on `x,q,y,z,w`: `delete(2)` answers `q` and
/// leaves `x,y,z,w` at size 4, while `remove(2)` answers the item and leaves
/// a hole with the size unchanged. Same argument, same answer, different
/// array.
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
    match interp.heap.get_mut(receiver).map(|object| &mut object.body) {
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
///
/// Measured on `1,2,3,4,5`: `section(2,3)` is `2,3,4`; `section(4,10)` clamps
/// to `4,5` rather than raising; `section(2,0)` is empty; and one argument
/// runs to the end.
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
///
/// **A single-dimensional array answers a one-element array holding its
/// size**, which is why this is not `~dimension`'s plural spelling: measured,
/// `.Array~of(1,2,3,4,5)~dimensions` prints `5` and `~dimension` prints `1`.
fn native_array_dimensions(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let extents = match dimensions_of(interp, receiver)? {
        Some(shape) => shape,
        None => vec![slots_of(interp, receiver)?.len()],
    };
    let items = extents
        .into_iter()
        .map(|extent| interp.counted(extent))
        .collect();
    Ok(Some(array_of(interp, items)))
}

// ---- the sort family ----

/// Every item of a receiver the sort family will accept, in index order.
///
/// **A hole is a refusal and not a skip.** Measured at rc 158, an array
/// holding `[1]`, `[3]`, `[5]` answers `~sort` with `98.975 Missing array
/// element at position 2.`, where `allItems` happily skips the same holes.
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
    ///
    /// `ArrayClass::BaseSortComparator::compare` is `first->compareTo(second)`
    /// (`classes/ArrayClass.cpp:2891`), a C++ virtual, which is why the
    /// default order is **not** numeric: measured,
    /// `.Array~of(10,9,2,100,1)~sort` answers `1,10,100,2,9`.
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
    // The sign is all the sort reads, which is what the C++ does with the
    // `wholenumber_t` its two comparators answer.
    let text = interp.to_text(answer);
    let value: i64 = std::str::from_utf8(&text)
        .ok()
        .and_then(|text| text.trim().parse().ok())
        .unwrap_or(0);
    Ok(value)
}

/// A stable merge sort over `items`, which is what the interpreter's own four
/// names all reach: `Setup.cpp` maps `Sort` and `StableSort` onto
/// `ArrayClass::stableSortRexx` and the two `With` spellings onto
/// `stableSortWithRexx`, so **there is one algorithm here and not two**.
///
/// Written out rather than handed to `slice::sort_by` because the comparison
/// runs Rexx and can raise, and a `Result` cannot travel through a `bool`
/// comparator.
fn merge_sort(
    interp: &mut Interp,
    order: &Order,
    items: Vec<ObjRef>,
) -> Result<Vec<ObjRef>, Failure> {
    if items.len() <= 1 {
        return Ok(items);
    }
    let mut items = items;
    let right = items.split_off(items.len() / 2);
    let mut left = merge_sort(interp, order, items)?.into_iter().peekable();
    let mut right = merge_sort(interp, order, right)?.into_iter().peekable();
    let mut merged = Vec::new();
    loop {
        match (left.peek().copied(), right.peek().copied()) {
            (Some(a), Some(b)) => {
                // `<= 0` keeps the left run first for equal keys, which is
                // what makes this stable: measured,
                // `.Array~of('b1','a1','b2','a2')~stableSortWith` over a
                // first-character comparator answers `a1,a2,b1,b2`.
                if order_of(interp, order, a, b)? <= 0 {
                    merged.push(a);
                    left.next();
                } else {
                    merged.push(b);
                    right.next();
                }
            }
            (Some(a), None) => {
                merged.push(a);
                left.next();
            }
            (None, Some(b)) => {
                merged.push(b);
                right.next();
            }
            (None, None) => break,
        }
    }
    Ok(merged)
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
    let items = dense_items(interp, receiver)?;
    let sorted = merge_sort(interp, order, items)?;
    write_back(interp, receiver, sorted)?;
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
///
/// `ArrayClass::sectionRexx` ends `allocateArrayOfClass`
/// (`classes/ArrayClass.cpp:1524`), so a `Queue`'s section is a **Queue**.
/// Measured: `q~section(1,1)~makeString` is 97.1 on the oracle, because a
/// `Queue` does not answer `makeString` -- an `Array` result would have
/// answered it, which is how this was found.
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
///
/// **This is the whole of spec D90's mechanism.** The instance is an ordinary
/// `Body::Instance`, so it has an object variable pool and the subclass's own
/// `expose` works; the store is a pool entry, so the collection methods
/// inherited from `Array` work through [`store_of`]. Both halves on one
/// object, which is what `CircularQueue~init`'s `expose size` beside its
/// `queue` needs.
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
///
/// Upstream a `Queue` *is* an array -- `QueueClass` derives from `ArrayClass`
/// and `Setup.cpp` copies the whole behaviour across. Here the store is an
/// `Array` object in the receiver's own pool, for the reason [`store_of`]
/// gives.
fn native_queue_init(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    super::optional_length_argument(interp, args, 0)?;
    let store = interp.alloc_with(BehaviourId::ARRAY, Body::array(Vec::new()));
    interp.roots.push_temp(store);
    let scope = store_scope(interp);
    interp.set_pool_variable(receiver, scope, QUEUE_ITEMS, store);
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
    let store = store_of(interp, receiver)?;
    let at = array_slots(interp, store)?.len();
    array_splice_slot(interp, store, at, Some(item))?;
    Ok(None)
}

/// `Queue~push(item)`: adds at the **front** -- `QueueClass::pushRexx`.
///
/// Measured: after `queue('a')`, `queue('b')`, `push('z')` the queue reads
/// `z,a,b`, so the two are opposite ends and not spellings of one another.
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
///
/// That is the difference worth stating: `Array~remove` leaves a hole and
/// keeps the size, while `Queue~remove` closes the gap. Measured on a queue
/// reading `a,b,c`, `remove(2)` answers `b` and leaves `a,c` at size 2.
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
    let Some(position) = array_position(interp, store, &args[1..], IndexUse::Get)? else {
        return Err(refuse(interp));
    };
    let held = array_slots(interp, store)?.len();
    if position > held {
        return Err(refuse(interp));
    }
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
///
/// A row of its own rather than the shared registration, because the bodies
/// `Setup.cpp` lets `Queue` inherit live in `dispatch.rs` and take the
/// receiver's own slots. The ones in this file reach the store through
/// [`store_of`] already; these four are the ones that do not.
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
fn native_queue_size(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let size = slots_of(interp, receiver)?.len();
    Ok(Some(interp.counted(size)))
}

// ---- List ----

/// A `List` index argument, whose position in the error is **two** whatever
/// its position in the argument list is.
///
/// `ListClass` passes `ARG_TWO` to `validateIndex`, `requiredIndex` and
/// `validateInsertionIndex` alike, so `~at`, `~put` and `~insert` all report
/// `argument 2 is required` for a missing index.
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
    let (Some(items), Some(handles), Some(free)) = (
        pool_variable(interp, receiver, scope, LIST_ITEMS),
        pool_variable(interp, receiver, scope, LIST_HANDLES),
        pool_variable(interp, receiver, scope, LIST_FREE),
    ) else {
        return Err(Loud::receiver_class("a value that is not a list").into());
    };
    Ok((items, handles, free))
}

/// Where in the list `handle` sits, or `None` for a handle the list does not
/// hold.
///
/// **A `List` index is a handle and not a position.** Measured: appending
/// `a`, `b`, `c` answers `0 1 2`; inserting after the first answers a fresh
/// `3` and leaves every old handle reaching the item it always did; removing
/// the second leaves the first and third still valid.
fn list_position(
    interp: &mut Interp,
    receiver: ObjRef,
    handle: ObjRef,
) -> Result<Option<usize>, Failure> {
    let (_, handles, _) = list_state(interp, receiver)?;
    let held = array_slots_owned(interp, handles)?;
    for (offset, slot) in held.iter().enumerate() {
        if let Some(slot) = *slot
            && same_item(interp, slot, handle)?
        {
            return Ok(Some(offset));
        }
    }
    Ok(None)
}

/// The handle a new item takes: the last one removal freed, or a fresh one.
///
/// **Last freed first, measured.** Removing handles 1 and then 3 from a
/// five-item list makes the next three appends answer `3`, `1` and a fresh
/// `5` -- so the free list is a stack, not a queue, and not "the lowest free
/// handle" either.
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
    let scope = list_scope(interp);
    for name in [LIST_ITEMS, LIST_HANDLES, LIST_FREE] {
        let store = interp.alloc_with(BehaviourId::ARRAY, Body::array(Vec::new()));
        interp.roots.push_temp(store);
        interp.set_pool_variable(receiver, scope, name, store);
    }
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
            && same_item(interp, slot, wanted)?
        {
            return list_take(interp, receiver, offset);
        }
    }
    Ok(Some(ObjRef::NIL))
}

/// The list's items and handles, as owned copies.
fn list_pairs(interp: &mut Interp, receiver: ObjRef) -> Result<Vec<(ObjRef, ObjRef)>, Failure> {
    let (items, handles, _) = list_state(interp, receiver)?;
    let items = array_slots_owned(interp, items)?;
    let handles = array_slots_owned(interp, handles)?;
    Ok(handles
        .into_iter()
        .zip(items)
        .filter_map(|(handle, item)| Some((handle?, item?)))
        .collect())
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
    Ok(Some(array_of(interp, items)))
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

/// `List~empty`: drops every entry. The freed handles go back on the stack,
/// which is what keeps `append` after `empty` answering what the oracle does.
fn native_list_empty(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    while !list_pairs(interp, receiver)?.is_empty() {
        list_take(interp, receiver, 0)?;
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
    Ok(Some(if want_item { item } else { handle }))
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
        if same_item(interp, item, wanted)? {
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
        if same_item(interp, item, wanted)? {
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
    let class = interp
        .class_of_value(receiver)
        .ok_or_else(|| Failure::from(Loud::receiver_class("a value that is not a list")))?;
    let section = new_instance(interp, class)?;
    interp.roots.push_temp(section);
    let caller = interp.caller();
    interp.send_message(section, super::INIT, None, &[], caller)?;
    for (_, item) in pairs.into_iter().skip(at).take(count) {
        let at = list_pairs(interp, section)?.len();
        list_insert_at(interp, section, at, Some(item))?;
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
    let items: Vec<ObjRef> = pairs.iter().map(|(_, item)| *item).collect();
    let handles: Vec<ObjRef> = pairs.into_iter().map(|(handle, _)| handle).collect();
    let items = array_of(interp, items);
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
///
/// **An omitted argument is refused, and the position named is its own.**
/// Measured at rc 163: `.List~of('a',,'c')` reports `Missing argument in
/// method; argument 2 is required.`
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
    for argument in args.iter().flatten() {
        let caller = interp.caller();
        interp.send_message(object, b"APPEND", None, &[Some(*argument)], caller)?;
    }
    Ok(Some(object))
}

/// The collection classes' primitive methods, chained into
/// `ObjectModel::build`.
///
/// Each row's method name is looked up in the class's own dictionary and the
/// build panics if it is not there, so a name a class's behaviour does not
/// answer cannot be added quietly.
///
/// **Arities come from `corpus/collection-scopes.tsv`**, which carries
/// `Setup.cpp`'s own third operand: a literal is a maximum and `A_COUNT` is
/// [`Arity::Counted`].
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
