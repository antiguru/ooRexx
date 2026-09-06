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

// ---- the contents protocol ----

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
    let empty = array_slots(interp, receiver)?.iter().flatten().count() == 0;
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
    let length = array_slots(interp, receiver)?.len();
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
    let Some(position) = array_position(interp, receiver, args, IndexUse::Get)? else {
        return Ok(Some(crate::eval::logical(false)));
    };
    let held = matches!(
        array_slots(interp, receiver)?.get(position - 1),
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
    let Some(position) = array_position(interp, receiver, args, IndexUse::Get)? else {
        return Ok(Some(ObjRef::NIL));
    };
    let held = match array_slots(interp, receiver)?.get(position - 1) {
        Some(Some(item)) => *item,
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
    let slots = array_slots_owned(interp, receiver)?;
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
