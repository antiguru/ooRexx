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

//! `List`'s primitive methods: the three pool entries a list is, and the
//! methods over them.

use super::{
    Arity, BehaviourId, Body, Cleared, Failure, Interp, Loud, NativeMethod, ObjRef, Raised,
    array_of, array_of_slots, array_slots, array_slots_owned, array_splice, array_splice_slot,
    item_argument, new_instance, new_supplier, pool_variable, same_item,
};

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
pub(super) fn list_state(
    interp: &mut Interp,
    receiver: ObjRef,
) -> Result<(ObjRef, ObjRef, ObjRef), Failure> {
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
pub(super) fn is_list(interp: &mut Interp, receiver: ObjRef) -> bool {
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
pub(super) fn list_insert_at(
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

/// `List`'s primitive methods, chained into `ObjectModel::build` beside
/// [`super::NATIVE_METHODS`].
pub(in crate::dispatch) const NATIVE_METHODS: &[(&str, &str, Arity, NativeMethod)] = &[
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
];
