use rexx_core::{Body, Heap, ObjRef, RootSet};

#[test]
fn an_object_with_uninit_is_reported_rather_than_swept_immediately() {
    let mut heap = Heap::new();
    let roots = RootSet::new();
    let obj = heap.alloc(Body::Instance(vec![]));
    heap.get_mut(obj).unwrap().has_uninit = true;
    let stats = heap.collect(&roots);
    assert_eq!(stats.pending_uninit, vec![obj]);
    assert!(
        heap.get(obj).is_some(),
        "it must survive until UNINIT has run"
    );
}

#[test]
fn a_weak_reference_does_not_keep_its_target_alive() {
    let mut heap = Heap::new();
    let mut roots = RootSet::new();
    let target = heap.alloc(Body::Text {
        bytes: b"target".to_vec(),
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
        bytes: b"target".to_vec(),
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
    let target = heap.alloc(Body::Instance(vec![]));
    heap.get_mut(target).unwrap().has_uninit = true;
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
