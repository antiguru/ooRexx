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

//! A live handle resolves; one that outlived its activation misses.

use rexx_api::handles::Table;
use rexx_core::{
    Body, Decoded, GENERATION_MAX, Heap, ObjRef, RootSet, SMALL_INT_MAX, SMALL_INT_MIN,
};

/// An object with `slots` empty slots, so two allocations are told apart by
/// what they hold and not only by the handle that names them.
fn array(slots: usize) -> Body {
    Body::Array {
        slots: vec![None; slots],
        dimensions: None,
    }
}

fn heap_parts(object: ObjRef) -> (u32, u32) {
    match object.decode() {
        Decoded::Heap { slot, generation } => (slot, generation),
        other => panic!("expected a heap reference, got {other:?}"),
    }
}

fn array_len(heap: &Heap, object: ObjRef) -> Option<usize> {
    match &heap.get(object)?.body {
        Body::Array { slots, .. } => Some(slots.len()),
        other => panic!("expected an array, got {other:?}"),
    }
}

/// The positive control. Without it, the staleness test below passes for a
/// table that never resolves anything.
#[test]
fn a_live_handle_resolves_to_the_object_it_was_made_from() {
    let mut heap = Heap::new();
    let roots = RootSet::new();
    let object = heap.alloc(array(3));

    let mut table = Table::new();
    let handle = table.register(object);

    assert!(
        !handle.is_null(),
        "a registered object's handle is NULLOBJECT"
    );
    assert_eq!(table.resolve(handle), Some(object));
    assert_eq!(array_len(&heap, object), Some(3));

    // The registration is what keeps it alive across a collection, and it
    // still resolves to the same object afterwards.
    let mut anchor = RootSet::new();
    for root in table.roots() {
        anchor.push_temp(root);
    }
    let stats = heap.collect(&anchor);
    assert_eq!(stats.swept, 0);
    assert_eq!(table.resolve(handle), Some(object));
    assert_eq!(array_len(&heap, object), Some(3));

    // ... and with nothing rooting it, the same collection sweeps it, which
    // is what makes the line above a claim about the roots rather than about
    // an object nothing could have collected.
    let stats = heap.collect(&roots);
    assert_eq!(stats.swept, 1);
}

/// The defining property: a handle that outlived its activation is a lookup
/// miss even after the slot it named has been handed to a different object
/// and that object registered.
#[test]
fn a_handle_that_outlived_its_activation_misses_the_slots_next_occupant() {
    let mut heap = Heap::new();
    let roots = RootSet::new();

    // The native call runs and is handed an object.
    let first = heap.alloc(array(3));
    let mut table = Table::new();
    let stale = table.register(first);
    assert_eq!(table.resolve(stale), Some(first));

    // Its activation ends.
    table.clear();

    // The collection the handle outlives.
    let stats = heap.collect(&roots);
    assert_eq!(stats.swept, 1);
    assert!(heap.get(first).is_none());

    // The next allocation takes the freed slot back off the free list. Both
    // halves are asserted: a test that could not tell "the slot was reused by
    // a different object" from "the slot was never reused" would not be
    // measuring anything.
    let second = heap.alloc(array(4));
    assert_eq!(heap_parts(second).0, heap_parts(first).0);
    assert_ne!(heap_parts(second).1, heap_parts(first).1);
    assert_eq!(array_len(&heap, second), Some(4));

    // A later native call is handed the slot's new occupant.
    let mut later = Table::new();
    let live = later.register(second);
    assert_eq!(later.resolve(live), Some(second));

    // The stale handle carries the old generation, so it names nothing --
    // rather than the object that took the slot.
    assert_eq!(later.resolve(stale), None);
    assert_ne!(stale, live);
}

#[test]
fn remove_drops_one_registration_and_clear_drops_them_all() {
    let mut heap = Heap::new();
    let first = heap.alloc(array(1));
    let second = heap.alloc(array(2));

    let mut table = Table::new();
    let first_handle = table.register(first);
    let second_handle = table.register(second);

    table.remove(first_handle);
    assert_eq!(table.resolve(first_handle), None);
    assert_eq!(table.resolve(second_handle), Some(second));
    // As `removeLocalReference` ignores an object it does not hold.
    table.remove(first_handle);

    table.clear();
    assert_eq!(table.resolve(second_handle), None);
    assert_eq!(table.roots().count(), 0);
}

#[test]
fn registering_the_same_object_twice_answers_the_same_handle() {
    let mut heap = Heap::new();
    let object = heap.alloc(array(1));

    let mut table = Table::new();
    let first = table.register(object);
    let second = table.register(object);

    assert_eq!(first, second);
    assert_eq!(table.roots().count(), 1);
}

/// `NULLOBJECT` is the null pointer, so no object may encode to it -- and the
/// `ObjRef` whose bits are zero is the first one a fresh heap allocates.
#[test]
fn the_extremes_of_the_handle_space_are_not_null() {
    let mut table = Table::new();
    for object in [
        ObjRef::heap(0, 0),
        ObjRef::heap(u32::MAX, GENERATION_MAX),
        ObjRef::NIL,
        ObjRef::small_int(SMALL_INT_MAX).expect("in range by construction"),
        ObjRef::small_int(SMALL_INT_MIN).expect("in range by construction"),
        ObjRef::inline_text(&[0xFF; 7]).expect("seven bytes fit inline"),
    ] {
        let handle = table.register(object);
        assert!(!handle.is_null(), "{object:?} encodes to NULLOBJECT");
        assert_eq!(table.resolve(handle), Some(object));
    }
}
