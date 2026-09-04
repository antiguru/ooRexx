/*----------------------------------------------------------------------------*/
/*                                                                            */
/* Copyright (c) 2026 Rexx Language Association. All rights reserved.          */
/*                                                                            */
/* This program and the accompanying materials are made available under       */
/* the terms of the Common Public License v1.0 which accompanies this         */
/* distribution. A copy is also available at the following address:           */
/* https://www.oorexx.org/license.html                                        */
/*                                                                            */
/*----------------------------------------------------------------------------*/

//! [`ScopePools`], the storage `EXPOSE` binds a method's names to (D40).

use rexx_core::{BehaviourHandle, Body, ObjRef, ScopePools};

fn scopes() -> (ObjRef, ObjRef) {
    (
        ObjRef::class(0).expect("a class identity"),
        ObjRef::class(1).expect("a class identity"),
    )
}

#[test]
fn a_name_the_pool_never_held_reads_as_unset() {
    let (sup, _) = scopes();
    assert_eq!(ScopePools::new().get(sup, b"V"), None);
}

/// The pool's whole reason for being keyed on a scope: one object, one name,
/// two values at once. Measured on the oracle -- a class method on `sup` and
/// one on `sub subclass sup`, each `expose v`, each writing its own value, and
/// each reading it back through the receiving class object `.sub` -- prints
/// `S B`. A pool keyed on the name alone prints `B B`.
#[test]
fn two_scopes_hold_one_name_at_two_values() {
    let (sup, sub) = scopes();
    let s = ObjRef::heap(1, 0);
    let b = ObjRef::heap(2, 0);
    let mut pools = ScopePools::new();
    pools.set(sup, b"V", s);
    pools.set(sub, b"V", b);
    assert_eq!(pools.get(sup, b"V"), Some(s));
    assert_eq!(pools.get(sub, b"V"), Some(b));
}

#[test]
fn a_second_write_to_one_name_replaces_rather_than_shadows() {
    let (sup, _) = scopes();
    let first = ObjRef::heap(1, 0);
    let second = ObjRef::heap(2, 0);
    let mut pools = ScopePools::new();
    pools.set(sup, b"V", first);
    pools.set(sup, b"V", second);
    assert_eq!(pools.get(sup, b"V"), Some(second));
    // A shadowing push would leave the first value reachable and would make
    // the pool grow without bound under a loop that assigns one variable.
    let mut out = Vec::new();
    let class = ObjRef::class(2).expect("a class identity");
    Body::Instance {
        class,
        behaviour: BehaviourHandle::new(0),
        name: None,
        pools,
        own: None,
        native: None,
    }
    .trace(&mut out);
    assert_eq!(out, vec![class, sup, second]);
}

/// `DROP` on an exposed name, and the neighbouring success that pins it to the
/// name rather than to the pool: the other scope's binding of the same name is
/// untouched.
#[test]
fn clearing_one_scopes_binding_leaves_the_others() {
    let (sup, sub) = scopes();
    let s = ObjRef::heap(1, 0);
    let b = ObjRef::heap(2, 0);
    let mut pools = ScopePools::new();
    pools.set(sup, b"V", s);
    pools.set(sub, b"V", b);
    pools.clear(sup, b"V");
    assert_eq!(pools.get(sup, b"V"), None);
    assert_eq!(pools.get(sub, b"V"), Some(b));
}

#[test]
fn clearing_a_name_the_pool_never_held_is_a_no_op() {
    let (sup, sub) = scopes();
    let s = ObjRef::heap(1, 0);
    let mut pools = ScopePools::new();
    pools.set(sup, b"V", s);
    pools.clear(sup, b"NEVERBOUND");
    pools.clear(sub, b"V");
    assert_eq!(pools.get(sup, b"V"), Some(s));
}
