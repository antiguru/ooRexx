use rexx_core::{Body, Decoded, Heap, ObjRef};

#[test]
fn allocation_returns_a_heap_handle_that_reads_back() {
    let mut heap = Heap::new();
    let s = heap.alloc(Body::String("hello".into()));
    assert!(matches!(s.decode(), Decoded::Heap { .. }));
    assert!(matches!(heap.get(s).map(|o| &o.body), Some(Body::String(t)) if t == "hello"));
}

#[test]
fn a_handle_from_a_stale_generation_does_not_read_the_slots_new_occupant() {
    let mut heap = Heap::new();
    let stale = heap.alloc(Body::String("gone".into()));
    let Decoded::Heap { slot, generation } = stale.decode() else { panic!("heap handle") };
    let forged = ObjRef::heap(slot, generation + 1);
    assert!(heap.get(forged).is_none(), "a generation mismatch is a miss, not an alias");
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
    let a = heap.alloc(Body::String("a".into()));
    let arr = heap.alloc(Body::Array(vec![a, ObjRef::small_int(1).unwrap(), ObjRef::NIL]));
    let Some(Body::Array(items)) = heap.get(arr).map(|o| &o.body) else {
        panic!("expected an array")
    };
    assert_eq!(items.len(), 3);
    assert_eq!(items[0], a);
}
