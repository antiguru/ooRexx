use rexx_core::{BehaviourId, Body, Bytes, Heap, ObjRef, Object, RootSet};

#[test]
fn unreachable_objects_are_swept() {
    let mut heap = Heap::new();
    let roots = RootSet::new();
    heap.alloc(Body::Text {
        bytes: Bytes::from_slice(b"garbage"),
        num: None,
    });
    let stats = heap.collect(&roots);
    assert_eq!(stats.swept, 1);
    assert_eq!(heap.live_count(), 0);
}

#[test]
fn objects_reachable_from_a_root_survive() {
    let mut heap = Heap::new();
    let mut roots = RootSet::new();
    let kept = heap.alloc(Body::Text {
        bytes: Bytes::from_slice(b"kept"),
        num: None,
    });
    roots.add_global(".KEPT", kept);
    heap.alloc(Body::Text {
        bytes: Bytes::from_slice(b"dropped"),
        num: None,
    });
    let stats = heap.collect(&roots);
    assert_eq!(stats.swept, 1);
    assert_eq!(stats.live, 1);
    assert!(heap.get(kept).is_some());
}

#[test]
fn transitively_reachable_objects_survive() {
    let mut heap = Heap::new();
    let mut roots = RootSet::new();
    let leaf = heap.alloc(Body::Text {
        bytes: Bytes::from_slice(b"leaf"),
        num: None,
    });
    let holder = heap.alloc(Body::array(vec![Some(leaf)]));
    roots.add_global(".HOLDER", holder);
    heap.collect(&roots);
    assert!(heap.get(leaf).is_some());
}

#[test]
fn reference_cycles_are_collected() {
    let mut heap = Heap::new();
    let roots = RootSet::new();
    let a = heap.alloc(Body::array(vec![None]));
    let b = heap.alloc(Body::array(vec![Some(a)]));
    // Written in place: a replaced body would hold bytes the heap was never
    // told of.
    let Some(Object {
        body: Body::Array { slots, .. },
        ..
    }) = heap.get_mut(a)
    else {
        panic!("a exists")
    };
    slots[0] = Some(b);
    let stats = heap.collect(&roots);
    assert_eq!(stats.swept, 2, "a cycle with no root must not survive");
}

#[test]
fn swept_slots_are_reused_by_the_next_allocation() {
    let mut heap = Heap::new();
    let roots = RootSet::new();
    heap.alloc(Body::Text {
        bytes: Bytes::from_slice(b"x"),
        num: None,
    });
    heap.collect(&roots);
    let reused = heap.alloc(Body::Text {
        bytes: Bytes::from_slice(b"y"),
        num: None,
    });
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
    let stale = heap.alloc(Body::Text {
        bytes: Bytes::from_slice(b"x"),
        num: None,
    });
    heap.collect(&roots);
    let reused = heap.alloc(Body::Text {
        bytes: Bytes::from_slice(b"y"),
        num: None,
    });
    assert_ne!(stale, reused, "reuse must bump the generation");
    assert!(
        heap.get(stale).is_none(),
        "the stale handle reads as a miss"
    );
    assert!(
        matches!(heap.get(reused).map(|o| &o.body), Some(Body::Text { bytes, .. }) if bytes.as_slice() == b"y")
    );
}

#[test]
fn a_stems_tails_and_default_are_traced() {
    let mut heap = Heap::new();
    let mut roots = RootSet::new();
    let tail = heap.alloc(Body::Text {
        bytes: Bytes::from_slice(b"kept"),
        num: None,
    });
    let default = heap.alloc(Body::Text {
        bytes: Bytes::from_slice(b"dflt"),
        num: None,
    });
    let mut tails = rexx_core::NameMap::default();
    tails.insert(b"1".to_vec(), (0, Some(tail)));
    // A tombstone: present, and reaching nothing.
    tails.insert(b"2".to_vec(), (1, None));
    let stem = heap.alloc_with_uncollected(
        BehaviourId::STEM,
        Body::Stem {
            name: b"A.".to_vec().into_boxed_slice(),
            default: Some(default),
            tails,
            exposed: None,
        },
    );
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
    let v = heap.alloc(Body::Text {
        bytes: Bytes::from_slice(b"local"),
        num: None,
    });
    roots.set_frame_slot(frame, 0, v);
    heap.collect(&roots);
    assert!(heap.get(v).is_some(), "a live local was swept");
    roots.activity_mut().pop_slots(frame);
    let stats = heap.collect(&roots);
    assert_eq!(stats.swept, 1, "the local outlived its frame");
}

#[test]
fn a_slot_frame_grows_for_a_name_the_plan_never_saw() {
    let mut roots = RootSet::new();
    let frame = roots.push_slots(1);
    let index = roots.activity_mut().grow_slots_of(frame, 1);
    assert_eq!(index, 1);
}

/// A frame below the top one grows, and every value in both frames stays
/// where its frame's offsets say and stays a root.
#[test]
fn growing_a_frame_below_the_top_keeps_every_frames_values() {
    let mut heap = Heap::new();
    let mut roots = RootSet::new();
    let outer = roots.push_slots(1);
    let inner = roots.push_slots(2);
    let text = |heap: &mut Heap, bytes: &[u8]| {
        heap.alloc(Body::Text {
            bytes: Bytes::from_slice(bytes),
            num: None,
        })
    };
    let (a, b, c, d) = (
        text(&mut heap, b"a"),
        text(&mut heap, b"b"),
        text(&mut heap, b"c"),
        text(&mut heap, b"d"),
    );
    roots.set_frame_slot_of(outer, 0, a);
    roots.set_frame_slot(inner, 0, b);
    roots.set_frame_slot(inner, 1, c);

    let grown = roots.activity_mut().grow_slots_of(outer, 1);
    assert_eq!(grown, 1);
    roots.set_frame_slot_of(outer, grown, d);

    assert_eq!(roots.frame_slot_of(outer, 0), Some(a));
    assert_eq!(roots.frame_slot_of(outer, grown), Some(d));
    assert_eq!(roots.frame_slot(inner, 0), Some(b));
    assert_eq!(roots.frame_slot(inner, 1), Some(c));
    assert_eq!(roots.activity().frame_len(outer), 2);
    assert_eq!(roots.activity().frame_len(inner), 2);
    let stats = heap.collect(&roots);
    assert_eq!(stats.swept, 0, "a value in a grown frame was swept");

    roots.activity_mut().pop_slots(inner);
    assert_eq!(roots.frame_slot(outer, grown), Some(d));
    let stats = heap.collect(&roots);
    assert_eq!(stats.swept, 2, "the popped frame's values outlived it");
}

/// Activation frames nest like any stack's call and return do, so popping
/// the outer of two open frames while the inner one is live panics where
/// the mistake is made instead of truncating the inner frame's locals.
#[test]
#[should_panic(expected = "pop_slots on a frame that is not the top one")]
fn popping_a_frame_that_is_not_the_top_one_panics() {
    let mut roots = RootSet::new();
    let outer = roots.push_slots(1);
    let _inner = roots.push_slots(1);
    roots.activity_mut().pop_slots(outer);
}

/// A cleared slot reads as unset, and that is **distinguishable from a slot
/// holding `.nil`** -- which is why `ObjRef::NIL` cannot encode "unset" and
/// this operation has to exist at all.
/// ```text
/// a = 5     ; drop a ; say a   ->  A                 (unset: the derived name)
/// x = .nil            ; say x  ->  The NIL object    (`.nil` is a value)
/// y = .nil  ; drop y  ; say y  ->  Y                 (unset again, not NIL)
/// ```
#[test]
fn a_cleared_slot_is_unset_and_differs_from_one_holding_nil() {
    let mut heap = Heap::new();
    let mut roots = RootSet::new();
    let frame = roots.push_slots(2);

    let five = heap.alloc(Body::Text {
        bytes: Bytes::from_slice(b"5"),
        num: None,
    });
    roots.set_frame_slot(frame, 0, five);
    roots.set_frame_slot(frame, 1, ObjRef::NIL);

    assert_eq!(roots.frame_slot(frame, 0), Some(five));
    assert_eq!(
        roots.frame_slot(frame, 1),
        Some(ObjRef::NIL),
        "a variable assigned .nil holds a value"
    );

    roots.clear_frame_slot(frame, 0);
    roots.clear_frame_slot(frame, 1);

    assert_eq!(
        roots.frame_slot(frame, 0),
        None,
        "drop a leaves the slot unset"
    );
    assert_eq!(
        roots.frame_slot(frame, 1),
        None,
        "drop y on a .nil-holding variable leaves it unset, not holding NIL"
    );
}

/// Clearing a slot stops it being a root, so the value it held becomes
/// collectable.
#[test]
fn a_cleared_slot_stops_being_a_root() {
    let mut heap = Heap::new();
    let mut roots = RootSet::new();
    let frame = roots.push_slots(1);

    let v = heap.alloc(Body::Text {
        bytes: Bytes::from_slice(b"dropped"),
        num: None,
    });
    roots.set_frame_slot(frame, 0, v);

    let stats = heap.collect(&roots);
    assert_eq!(stats.swept, 0, "a slot still holding the value is a root");
    assert!(heap.get(v).is_some());

    roots.clear_frame_slot(frame, 0);

    let stats = heap.collect(&roots);
    assert_eq!(stats.swept, 1, "a cleared slot must stop rooting its value");
    assert!(
        heap.get(v).is_none(),
        "the dropped value should be gone, not merely unreachable through slot()"
    );
}

/// Growth never hands out a cleared slot's index, and that is a requirement
/// rather than an accident of `grow_slots_of` appending.
#[test]
fn growth_does_not_recycle_a_cleared_slot() {
    let mut heap = Heap::new();
    let mut roots = RootSet::new();
    let frame = roots.push_slots(1);

    let v = heap.alloc(Body::Text {
        bytes: Bytes::from_slice(b"a"),
        num: None,
    });
    roots.set_frame_slot(frame, 0, v);
    roots.clear_frame_slot(frame, 0);

    let grown = roots.activity_mut().grow_slots_of(frame, 1);
    assert_eq!(grown, 1, "growth appends rather than reusing the cleared 0");

    // The cleared slot is still addressable and still its own name's, so a
    // later assignment to that name lands where the plan says it should.
    roots.set_frame_slot(frame, 0, v);
    assert_eq!(roots.frame_slot(frame, 0), Some(v));
    assert_eq!(
        roots.frame_slot(frame, grown),
        None,
        "growth yields an unset slot"
    );
}
