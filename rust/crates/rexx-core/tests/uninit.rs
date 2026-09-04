use rexx_core::{BehaviourHandle, Body, Bytes, Heap, ObjRef, RootSet, ScopePools};

#[test]
fn an_object_with_uninit_is_reported_rather_than_swept_immediately() {
    let mut heap = Heap::new();
    let roots = RootSet::new();
    let obj = heap.alloc(Body::Instance {
        class: ObjRef::class(0).expect("a class identity"),
        behaviour: BehaviourHandle::new(0),
        name: None,
        pools: ScopePools::new(),
        own: None,
        native: None,
    });
    assert!(heap.set_uninit(obj), "the handle names a live object");
    let stats = heap.collect(&roots);
    assert_eq!(stats.pending_uninit, vec![obj]);
    assert!(
        heap.get(obj).is_some(),
        "it must survive until UNINIT has run"
    );
    assert!(heap.get(obj).unwrap().has_uninit(), "and still be flagged");

    // The resurrection is one-shot: once the finalizer has been reported, the
    // object is ordinary again and the next collection takes it. Without this
    // the object would be resurrected on every collection for the rest of the
    // run, which is a leak the first half of the test cannot see.
    heap.clear_uninit_all(&[obj]);
    let stats = heap.collect(&roots);
    assert_eq!(stats.pending_uninit, vec![]);
    assert!(heap.get(obj).is_none(), "and it is swept this time");
}

#[test]
fn a_weak_reference_does_not_keep_its_target_alive() {
    let mut heap = Heap::new();
    let mut roots = RootSet::new();
    let target = heap.alloc(Body::Text {
        bytes: Bytes::from_slice(b"target"),
        num: None,
    });
    let weak = heap.alloc(Body::WeakRef(target));
    roots.add_global(".WEAK", weak);
    heap.collect(&roots);
    assert!(
        heap.get(target).is_none(),
        "the target was only weakly held"
    );
}

#[test]
fn a_cleared_weak_reference_reads_as_nil() {
    let mut heap = Heap::new();
    let mut roots = RootSet::new();
    let target = heap.alloc(Body::Text {
        bytes: Bytes::from_slice(b"target"),
        num: None,
    });
    let weak = heap.alloc(Body::WeakRef(target));
    roots.add_global(".WEAK", weak);
    heap.collect(&roots);
    assert!(matches!(heap.get(weak).map(|o| &o.body), Some(Body::WeakRef(r)) if *r == ObjRef::NIL));
}

#[test]
fn a_weak_reference_to_an_uninit_pending_object_is_still_cleared() {
    // The oracle clears weak references BEFORE the uninit list is marked, so
    // resurrection for UNINIT must not retroactively rescue a weak reference.
    // See RexxMemory.cpp:422-426.
    let mut heap = Heap::new();
    let mut roots = RootSet::new();
    let target = heap.alloc(Body::Instance {
        class: ObjRef::class(0).expect("a class identity"),
        behaviour: BehaviourHandle::new(0),
        name: None,
        pools: ScopePools::new(),
        own: None,
        native: None,
    });
    assert!(heap.set_uninit(target), "the handle names a live object");
    let weak = heap.alloc(Body::WeakRef(target));
    roots.add_global(".WEAK", weak);
    let stats = heap.collect(&roots);
    assert_eq!(
        stats.pending_uninit,
        vec![target],
        "it is still queued for UNINIT"
    );
    assert!(
        heap.get(target).is_some(),
        "and still alive until UNINIT has run"
    );
    assert!(
        matches!(heap.get(weak).map(|o| &o.body), Some(Body::WeakRef(r)) if *r == ObjRef::NIL),
        "but the weak reference was cleared before resurrection, as in the oracle"
    );
}

/// Flagging an object that was flagged and cleared before the collection
/// reports it once, not once per flagging.
///
/// **This is the registry's one hazard**, and it is the only place it is
/// pinned: the collector reads a list `set_uninit` appends to rather than
/// walking the arena, so an object that is cleared and flagged again holds
/// two entries unless clearing takes its own away. Both are live and both
/// flagged, so both would be reported, and `UNINIT` would run twice for one
/// object. Deleting the registry filter in `Heap::clear_uninit_all`
/// reddens this and nothing else in the workspace.
#[test]
fn an_object_flagged_twice_across_a_clear_is_reported_once() {
    let mut heap = Heap::new();
    let roots = RootSet::new();
    let obj = heap.alloc(Body::Instance {
        class: ObjRef::class(0).expect("a class identity"),
        behaviour: BehaviourHandle::new(0),
        name: None,
        pools: ScopePools::new(),
        own: None,
        native: None,
    });
    assert!(heap.set_uninit(obj), "the handle names a live object");
    heap.clear_uninit_all(&[obj]);
    assert!(heap.set_uninit(obj), "and can be flagged again");
    let stats = heap.collect(&roots);
    assert_eq!(stats.pending_uninit, vec![obj]);
}

/// A flagged object that stays unreachable across several collections is
/// resurrected every time and reported once.
///
/// **Both halves are load-bearing and they pull in opposite directions.** The
/// flag is the object's only root, so a collection that stopped resurrecting
/// it would sweep an object whose finalizer has not run; and a caller
/// recording every report has to scan what it already holds unless the
/// reports are unique, which is what made a program with many pending
/// finalizers quadratic.
#[test]
fn a_still_unreachable_flagged_object_is_reported_once_and_resurrected_every_time() {
    let mut heap = Heap::new();
    let roots = RootSet::new();
    let obj = heap.alloc(Body::Instance {
        class: ObjRef::class(0).expect("a class identity"),
        behaviour: BehaviourHandle::new(0),
        name: None,
        pools: ScopePools::new(),
        own: None,
        native: None,
    });
    assert!(heap.set_uninit(obj), "the handle names a live object");
    assert_eq!(heap.collect(&roots).pending_uninit, vec![obj]);
    assert_eq!(
        heap.collect(&roots).pending_uninit,
        vec![],
        "reported once, not once per collection"
    );
    assert!(heap.get(obj).is_some(), "and still alive to be finalized");
}

/// `take_uninit_flagged` answers every flagged object and leaves none behind.
///
/// The termination sweep's entry point: after it, the objects are ordinary
/// and the next collection takes them, which is what makes them finalizable
/// exactly once.
#[test]
fn taking_the_flagged_objects_clears_every_flag() {
    let mut heap = Heap::new();
    let mut roots = RootSet::new();
    let mut objects = Vec::new();
    for index in 0..3 {
        let obj = heap.alloc(Body::Instance {
            class: ObjRef::class(0).expect("a class identity"),
            behaviour: BehaviourHandle::new(0),
            name: None,
            pools: ScopePools::new(),
            own: None,
            native: None,
        });
        roots.add_global(&format!(".KEPT{index}"), obj);
        assert!(heap.set_uninit(obj));
        objects.push(obj);
    }
    assert_eq!(
        heap.collect(&roots).pending_uninit,
        vec![],
        "a rooted object is never unreachable, so no collection reports it"
    );
    assert_eq!(heap.take_uninit_flagged(), objects);
    assert_eq!(heap.take_uninit_flagged(), vec![]);
    for obj in objects {
        assert!(!heap.get(obj).expect("still rooted").has_uninit());
    }
}
