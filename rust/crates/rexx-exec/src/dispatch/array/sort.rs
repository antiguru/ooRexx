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

//! `Array`'s sort family: `sort`, `sortWith` and their stable twins, one
//! merge sort over the Array-shaped store.

use super::collection::{item_argument, slots_of, store_of};
use super::{
    Arity, Body, Cleared, Failure, Interp, Loud, NativeMethod, ObjRef, Raised, array_slots,
    whole_comparison,
};

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

/// `Array`'s sort methods, chained into `ObjectModel::build` ahead of the
/// rest of `Array`'s collection surface.
pub(in crate::dispatch) const NATIVE_METHODS: &[(&str, &str, Arity, NativeMethod)] = &[
    ("Array", "SORT", Arity::Fixed(0), native_array_sort),
    ("Array", "SORTWITH", Arity::Fixed(1), native_array_sort_with),
    ("Array", "STABLESORT", Arity::Fixed(0), native_array_sort),
    (
        "Array",
        "STABLESORTWITH",
        Arity::Fixed(1),
        native_array_sort_with,
    ),
];
