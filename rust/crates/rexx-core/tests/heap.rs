use rexx_core::{Body, Bytes, Decoded, Heap, ObjRef};

#[test]
fn allocation_returns_a_heap_handle_that_reads_back() {
    let mut heap = Heap::new();
    let s = heap.alloc(Body::Text {
        bytes: Bytes::from_slice(b"hello"),
        num: None,
    });
    assert!(matches!(s.decode(), Decoded::Heap { .. }));
    assert!(
        matches!(heap.get(s).map(|o| &o.body), Some(Body::Text { bytes, .. }) if bytes.as_slice() == b"hello")
    );
}

#[test]
fn a_handle_from_a_stale_generation_does_not_read_the_slots_new_occupant() {
    let mut heap = Heap::new();
    let stale = heap.alloc(Body::Text {
        bytes: Bytes::from_slice(b"gone"),
        num: None,
    });
    let Decoded::Heap { slot, generation } = stale.decode() else {
        panic!("heap handle")
    };
    let forged = ObjRef::heap(slot, generation + 1);
    assert!(
        heap.get(forged).is_none(),
        "a generation mismatch is a miss, not an alias"
    );
}

#[test]
fn small_integer_handles_are_not_in_the_heap() {
    let heap = Heap::new();
    assert!(heap.get(ObjRef::small_int(7).unwrap()).is_none());
    assert_eq!(heap.live_count(), 0);
}

#[test]
fn arrays_hold_handles_to_other_objects() {
    let mut heap = Heap::new();
    let a = heap.alloc(Body::Text {
        bytes: Bytes::from_slice(b"a"),
        num: None,
    });
    let arr = heap.alloc(Body::array(vec![
        Some(a),
        Some(ObjRef::small_int(1).unwrap()),
        None,
        Some(ObjRef::NIL),
    ]));
    let Some(Body::Array { slots: items, .. }) = heap.get(arr).map(|o| &o.body) else {
        panic!("expected an array")
    };
    assert_eq!(items.len(), 4);
    assert_eq!(items[0], Some(a));
    // An empty slot and a slot holding `.nil` are different values, which is
    // the distinction `~items` reads and `~at` does not expose.
    assert_eq!(items[2], None);
    assert_eq!(items[3], Some(ObjRef::NIL));
}

/// **A class identity and an arena handle can never be the same handle.**
///
/// Both counters start at their own base and the arena's is asserted never to
/// reach the class range (`Heap::alloc_with_uncollected`), so the two spaces
/// are disjoint by construction. This is what says so.
///
/// **Before `ObjRef::class` existed, `ClassRegistry::reserve_id` minted
/// `ObjRef::heap(n, 0)` from a counter starting at zero**, so the first class
/// and the first allocated slot were equal: the `allocated.contains` assertion
/// below fails on the very first pair, and `class_id()` answers `None` for
/// every class handle, failing the second.
#[test]
fn a_class_identity_is_never_an_arena_handle() {
    let mut heap = Heap::new();
    let allocated: Vec<ObjRef> = (0..64)
        .map(|n| {
            heap.alloc(Body::Text {
                bytes: Bytes::from_slice(&n.to_string().into_bytes()),
                num: None,
            })
        })
        .collect();
    let classes: Vec<ObjRef> = (0..64)
        .map(|n| ObjRef::class(n).expect("inside the reserved range"))
        .collect();

    for class in &classes {
        assert!(
            !allocated.contains(class),
            "a class identity equals an arena handle: {class:?}"
        );
        assert!(
            class.class_id().is_some(),
            "a class handle reads its index back"
        );
        assert!(
            heap.get(*class).is_none(),
            "a class identity must resolve to no object at all"
        );
    }
    for value in &allocated {
        assert!(
            value.class_id().is_none(),
            "an arena handle reads back as a class identity: {value:?}"
        );
    }
    // The two ends of the reserved range, which the loop above does not
    // reach: the highest id there is, and the first id there is not.
    assert!(ObjRef::class(rexx_core::CLASS_SLOT_BASE - 1).is_some());
    assert!(ObjRef::class(rexx_core::CLASS_SLOT_BASE).is_none());
}
