use rexx_core::{Body, Bytes, Decoded, Heap, ObjRef, RootSet};

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
#[test]
fn a_class_is_an_ordinary_arena_object_and_a_freed_one_misses() {
    let mut heap = Heap::new();
    let values: Vec<ObjRef> = (0..8)
        .map(|n| {
            heap.alloc(Body::Text {
                bytes: Bytes::from_slice(&n.to_string().into_bytes()),
                num: None,
            })
        })
        .collect();
    let class = heap.alloc(Body::Class { owned: Vec::new() });

    assert!(
        !values.contains(&class),
        "a class handle is distinct from every value handle"
    );
    assert!(
        matches!(
            heap.get(class).map(|object| &object.body),
            Some(Body::Class { .. })
        ),
        "a class resolves to an object, and its body is what says it is a class"
    );
    for value in &values {
        assert!(
            !matches!(
                heap.get(*value).map(|object| &object.body),
                Some(Body::Class { .. })
            ),
            "a value must not read back as a class: {value:?}"
        );
    }

    // Nothing roots it, so the collection takes it and the slot's generation
    // moves on.
    let roots = RootSet::new();
    heap.collect(&roots);
    assert!(
        heap.get(class).is_none(),
        "a collected class handle misses rather than answering the slot's next tenant"
    );
}
