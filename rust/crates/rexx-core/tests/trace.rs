use rexx_core::{Body, ObjRef};

#[test]
fn a_string_reaches_nothing() {
    let mut out = Vec::new();
    Body::String("x".into()).trace(&mut out);
    assert!(out.is_empty());
}

#[test]
fn an_array_reaches_every_element_including_duplicates() {
    let a = ObjRef::heap(3, 0);
    let mut out = Vec::new();
    Body::Array(vec![a, a, ObjRef::NIL]).trace(&mut out);
    assert_eq!(out, vec![a, a, ObjRef::NIL]);
}

#[test]
fn an_instance_reaches_its_variable_values_but_not_their_names() {
    let v = ObjRef::heap(9, 0);
    let mut out = Vec::new();
    Body::Instance(vec![("NAME".into(), v)]).trace(&mut out);
    assert_eq!(out, vec![v]);
}
