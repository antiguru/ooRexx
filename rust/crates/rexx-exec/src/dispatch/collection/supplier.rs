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

//! `Supplier`'s primitive methods, and the constructor every collection's
//! `supplier` method builds one with.

use super::{
    Arity, ArrayArgument, Cleared, Decoded, Failure, Interp, Loud, NativeMethod, ObjRef, Raised,
    array_argument, array_slots, new_instance, pool_variable, request_array,
};

/// The `Supplier` state's three names, bound in the receiver's own pool under
/// the `Supplier` class as scope.
const SUPPLIER_ITEMS: &[u8] = b"ITEMS";

const SUPPLIER_INDEXES: &[u8] = b"INDEXES";

const SUPPLIER_POSITION: &[u8] = b"POSITION";

// ---- Supplier ----

/// A `Supplier` over `items` and `indexes`, positioned at the first pair.
pub(in crate::dispatch) fn new_supplier(
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

/// `Supplier`'s primitive methods, chained into `ObjectModel::build` beside
/// [`super::super::NATIVE_METHODS`].
pub(in crate::dispatch) const NATIVE_METHODS: &[(&str, &str, Arity, NativeMethod)] = &[
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
