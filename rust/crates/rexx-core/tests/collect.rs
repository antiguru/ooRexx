use rexx_core::{BehaviourId, Body, Heap, RootSet};
use std::collections::HashMap;

#[test]
fn unreachable_objects_are_swept() {
    let mut heap = Heap::new();
    let roots = RootSet::new();
    heap.alloc(Body::Text { bytes: b"garbage".to_vec(), num: None });
    let stats = heap.collect(&roots);
    assert_eq!(stats.swept, 1);
    assert_eq!(heap.live_count(), 0);
}

#[test]
fn objects_reachable_from_a_root_survive() {
    let mut heap = Heap::new();
    let mut roots = RootSet::new();
    let kept = heap.alloc(Body::Text { bytes: b"kept".to_vec(), num: None });
    roots.add_global(".KEPT", kept);
    heap.alloc(Body::Text { bytes: b"dropped".to_vec(), num: None });
    let stats = heap.collect(&roots);
    assert_eq!(stats.swept, 1);
    assert_eq!(stats.live, 1);
    assert!(heap.get(kept).is_some());
}

#[test]
fn transitively_reachable_objects_survive() {
    let mut heap = Heap::new();
    let mut roots = RootSet::new();
    let leaf = heap.alloc(Body::Text { bytes: b"leaf".to_vec(), num: None });
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
    let Some(obj) = heap.get_mut(a) else {
        panic!("a exists")
    };
    obj.body = Body::Array(vec![b]);
    let stats = heap.collect(&roots);
    assert_eq!(stats.swept, 2, "a cycle with no root must not survive");
}

#[test]
fn swept_slots_are_reused_by_the_next_allocation() {
    let mut heap = Heap::new();
    let roots = RootSet::new();
    heap.alloc(Body::Text { bytes: b"x".to_vec(), num: None });
    heap.collect(&roots);
    let reused = heap.alloc(Body::Text { bytes: b"y".to_vec(), num: None });
    assert_eq!(
        heap.slot_capacity(),
        1,
        "the freed slot was reused, not appended"
    );
    assert!(heap.get(reused).is_some());
}

#[test]
fn a_handle_to_a_swept_object_does_not_alias_the_slots_next_occupant() {
    let mut heap = Heap::new();
    let roots = RootSet::new();
    let stale = heap.alloc(Body::Text { bytes: b"x".to_vec(), num: None });
    heap.collect(&roots);
    let reused = heap.alloc(Body::Text { bytes: b"y".to_vec(), num: None });
    assert_ne!(stale, reused, "reuse must bump the generation");
    assert!(
        heap.get(stale).is_none(),
        "the stale handle reads as a miss"
    );
    assert!(
        matches!(heap.get(reused).map(|o| &o.body), Some(Body::Text { bytes, .. }) if bytes == b"y")
    );
}

#[test]
fn a_stems_tails_and_default_are_traced() {
    let mut heap = Heap::new();
    let mut roots = RootSet::new();
    let tail = heap.alloc(Body::Text { bytes: b"kept".to_vec(), num: None });
    let default = heap.alloc(Body::Text { bytes: b"dflt".to_vec(), num: None });
    let mut tails = HashMap::new();
    tails.insert(b"1".to_vec(), Some(tail));
    // A tombstone: present, and reaching nothing.
    tails.insert(b"2".to_vec(), None);
    let stem = heap.alloc_with(BehaviourId::STEM, Body::Stem {
        name: b"A.".to_vec().into_boxed_slice(),
        default: Some(default),
        tails,
    });
    roots.add_global("a.", stem);
    heap.collect(&roots);
    assert!(heap.get(tail).is_some(), "a live tail was swept");
    assert!(heap.get(default).is_some(), "the stem default was swept");
}

#[test]
fn slot_frames_keep_locals_alive_and_release_them_on_pop() {
    let mut heap = Heap::new();
    let mut roots = RootSet::new();
    let frame = roots.push_slots(2);
    let v = heap.alloc(Body::Text { bytes: b"local".to_vec(), num: None });
    roots.set_slot(frame, 0, v);
    heap.collect(&roots);
    assert!(heap.get(v).is_some(), "a live local was swept");
    roots.pop_slots(frame);
    let stats = heap.collect(&roots);
    assert_eq!(stats.swept, 1, "the local outlived its frame");
}

#[test]
fn a_slot_frame_grows_for_a_name_the_plan_never_saw() {
    let mut roots = RootSet::new();
    let frame = roots.push_slots(1);
    let index = roots.grow_slots(frame);
    assert_eq!(index, 1);
}
