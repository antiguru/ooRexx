use rexx_core::{Body, Heap, RootSet};

#[test]
fn unreachable_objects_are_swept() {
    let mut heap = Heap::new();
    let roots = RootSet::new();
    heap.alloc(Body::String("garbage".into()));
    let stats = heap.collect(&roots);
    assert_eq!(stats.swept, 1);
    assert_eq!(heap.live_count(), 0);
}

#[test]
fn objects_reachable_from_a_root_survive() {
    let mut heap = Heap::new();
    let mut roots = RootSet::new();
    let kept = heap.alloc(Body::String("kept".into()));
    roots.add_global(".KEPT", kept);
    heap.alloc(Body::String("dropped".into()));
    let stats = heap.collect(&roots);
    assert_eq!(stats.swept, 1);
    assert_eq!(stats.live, 1);
    assert!(heap.get(kept).is_some());
}

#[test]
fn transitively_reachable_objects_survive() {
    let mut heap = Heap::new();
    let mut roots = RootSet::new();
    let leaf = heap.alloc(Body::String("leaf".into()));
    let holder = heap.alloc(Body::Array(vec![leaf]));
    roots.add_global(".HOLDER", holder);
    heap.collect(&roots);
    assert!(heap.get(leaf).is_some());
}

#[test]
fn reference_cycles_are_collected() {
    let mut heap = Heap::new();
    let roots = RootSet::new();
    let a = heap.alloc(Body::Array(vec![]));
    let b = heap.alloc(Body::Array(vec![a]));
    let Some(obj) = heap.get_mut(a) else { panic!("a exists") };
    obj.body = Body::Array(vec![b]);
    let stats = heap.collect(&roots);
    assert_eq!(stats.swept, 2, "a cycle with no root must not survive");
}

#[test]
fn swept_slots_are_reused_by_the_next_allocation() {
    let mut heap = Heap::new();
    let roots = RootSet::new();
    heap.alloc(Body::String("x".into()));
    heap.collect(&roots);
    let reused = heap.alloc(Body::String("y".into()));
    assert_eq!(heap.slot_capacity(), 1, "the freed slot was reused, not appended");
    assert!(heap.get(reused).is_some());
}

#[test]
fn a_handle_to_a_swept_object_does_not_alias_the_slots_next_occupant() {
    let mut heap = Heap::new();
    let roots = RootSet::new();
    let stale = heap.alloc(Body::String("x".into()));
    heap.collect(&roots);
    let reused = heap.alloc(Body::String("y".into()));
    assert_ne!(stale, reused, "reuse must bump the generation");
    assert!(heap.get(stale).is_none(), "the stale handle reads as a miss");
    assert!(matches!(heap.get(reused).map(|o| &o.body), Some(Body::String(t)) if t == "y"));
}
