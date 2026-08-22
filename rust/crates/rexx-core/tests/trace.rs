use rexx_core::{Body, Bytes, ObjRef, ScopePools};

#[test]
fn a_string_reaches_nothing() {
    let mut out = Vec::new();
    Body::Text {
        bytes: Bytes::from_slice(b"x"),
        num: None,
    }
    .trace(&mut out);
    assert!(out.is_empty());
}

#[test]
fn an_array_reaches_every_element_including_duplicates() {
    let a = ObjRef::heap(3, 0);
    let mut out = Vec::new();
    Body::Array(vec![Some(a), Some(a), Some(ObjRef::NIL)]).trace(&mut out);
    assert_eq!(out, vec![a, a, ObjRef::NIL]);
}

/// An empty slot reaches nothing, and the slots either side of it are still
/// reached -- the `None` arm must not stop the walk.
#[test]
fn an_array_reaches_past_an_empty_slot_and_not_through_it() {
    let a = ObjRef::heap(3, 0);
    let b = ObjRef::heap(4, 0);
    let mut out = Vec::new();
    Body::Array(vec![None, Some(a), None, Some(b), None]).trace(&mut out);
    assert_eq!(out, vec![a, b]);
}

#[test]
fn an_instance_reaches_every_scope_and_every_value_but_no_name() {
    let sup = ObjRef::class(0).expect("a class identity");
    let sub = ObjRef::class(1).expect("a class identity");
    let held_by_sup = ObjRef::heap(9, 0);
    let held_by_sub = ObjRef::heap(10, 0);
    let mut pools = ScopePools::new();
    // One name, two scopes: the collector must reach both values, and reaching
    // only one is what a pool keyed on the name alone would do.
    pools.set(sup, b"V", held_by_sup);
    pools.set(sub, b"V", held_by_sub);
    let mut out = Vec::new();
    Body::Instance(pools).trace(&mut out);
    assert_eq!(out, vec![sup, held_by_sup, sub, held_by_sub]);
}

#[test]
fn an_instance_stops_reaching_a_dropped_variable() {
    let scope = ObjRef::class(0).expect("a class identity");
    let value = ObjRef::heap(9, 0);
    let mut pools = ScopePools::new();
    pools.set(scope, b"V", value);
    pools.clear(scope, b"V");
    let mut out = Vec::new();
    Body::Instance(pools).trace(&mut out);
    // The scope's pool survives the drop and is still traced; the value it no
    // longer holds is not, which is what makes `DROP` release an object rather
    // than merely hide it.
    assert_eq!(out, vec![scope]);
}
