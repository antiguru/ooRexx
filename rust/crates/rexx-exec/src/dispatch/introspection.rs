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

//! `Class`'s readers over its own graph and `Object`'s three over a
//! receiver's behaviour -- `classes/ClassClass.cpp` and
//! `classes/ObjectClass.cpp`, bound by `memory/Setup.cpp:462`-`:477` and
//! `:538`-`:541`.

use super::{
    Arity, Behaviour, Cleared, Failure, Interp, NativeMethod, ObjRef, Raised, class_receiver,
    collection::{array_of, new_supplier},
};
use crate::eval::logical;
use rexx_classes::MethodId;

/// `Class`'s and `Object`'s introspection readers. Chained into
/// `ObjectModel::build` beside [`super::NATIVE_METHODS`].
pub(super) static NATIVE_METHODS: &[(&str, &str, Arity, NativeMethod)] = &[
    ("Class", "ISABSTRACT", Arity::Fixed(0), is_abstract),
    ("Class", "ISMETACLASS", Arity::Fixed(0), is_metaclass),
    ("Class", "METHODS", Arity::Fixed(1), methods),
    (
        "Class",
        "QUERYMIXINCLASS",
        Arity::Fixed(0),
        query_mixin_class,
    ),
    ("Class", "SUBCLASSES", Arity::Fixed(0), subclasses),
    ("Object", "INSTANCEMETHOD", Arity::Fixed(1), instance_method),
    (
        "Object",
        "INSTANCEMETHODS",
        Arity::Fixed(1),
        instance_methods,
    ),
    ("Object", "ISINSTANCEOF", Arity::Fixed(1), is_instance_of),
];

/// `Class~isAbstract`: `RexxClass::isAbstractRexx`
/// (`classes/ClassClass.cpp:314`).
fn is_abstract(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    Ok(Some(logical(interp.classes().is_abstract(class))))
}

/// `Class~isMetaClass`: `RexxClass::isMetaClassRexx`
/// (`classes/ClassClass.cpp:303`).
fn is_metaclass(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    Ok(Some(logical(interp.classes().is_metaclass(class))))
}

/// `Class~queryMixinClass`: `RexxClass::queryMixinClass`
/// (`classes/ClassClass.cpp:292`).
fn query_mixin_class(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    Ok(Some(logical(interp.classes().is_mixin(class))))
}

/// `Class~subclasses`: `RexxClass::getSubClasses`
/// (`classes/ClassClass.cpp:470`), the reverse edge `subclass` and `inherit`
/// maintain.
fn subclasses(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    let items = interp.classes().subclasses(class).to_vec();
    Ok(Some(array_of(interp, items)))
}

/// `Class~methods(scope)`: `RexxClass::methods`
/// (`classes/ClassClass.cpp:1018`).
fn methods(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    // `.nil` selects the receiving class, which is the whole of what this
    // adds over `Object~instanceMethods` (`:1022`-`:1025`).
    let scope = match args.first().copied().flatten() {
        Some(ObjRef::NIL) => Some(class),
        other => other,
    };
    let handle = interp.classes().instance_behaviour_handle(class);
    let entries = behaviour_entries(interp, Selection::Behaviour(handle), scope);
    supplier_of(interp, class, entries)
}

/// `Object~isInstanceOf(class)`: `RexxObject::isInstanceOfRexx`
/// (`classes/ObjectClass.cpp:258`), which is the receiver's class compared
/// against `class` through `isCompatibleWith`.
fn is_instance_of(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    // **The one row here that validates its argument**, where `~methods` and
    // `~instanceMethods` answer an empty supplier for a non-class. Measured,
    // oracle rc 168: `.Object~new~isInstanceOf(1)` is `88.914 Argument class
    // must be an instance of the Class class.` and the argumentless form is
    // `88.901`, which is `class_argument`'s pair.
    let argument = super::class_argument(interp, args)?;
    let Some(class) = interp.class_of_value(receiver) else {
        return Ok(Some(logical(false)));
    };
    Ok(Some(logical(interp.classes().is_a(class, argument))))
}

/// `Object~instanceMethod(name)`: `RexxObject::instanceMethodRexx`
/// (`classes/ObjectClass.cpp:370`).
fn instance_method(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    // 93.903 and not `Class~method`'s 88.901 -- measured, oracle:
    // `.Object~new~instanceMethod` is `93.903 Missing argument in method;
    // argument 1 is required.` at rc 163 where `.Array~method` is `88.901
    // Missing argument; argument method name is required.` at rc 168.
    let Some(Some(argument)) = args.first().copied() else {
        return Err(Raised::missing_method_argument(1).into());
    };
    let argument = super::required_string_named_argument(interp, argument, "method name")?;
    let name = interp.to_text(argument).to_ascii_uppercase();
    let selection = receiver_selection(interp, receiver)?;
    let owner = interp.class_of_value(receiver).unwrap_or(ObjRef::NIL);
    let Some((scope, method)) = lookup(interp, selection, &String::from_utf8_lossy(&name)) else {
        return Ok(Some(ObjRef::NIL));
    };
    Ok(Some(method_object_for(interp, owner, &name, scope, method)))
}

/// `Object~instanceMethods(scope)`: `RexxObject::instanceMethodsRexx`
/// (`classes/ObjectClass.cpp:387`).
fn instance_methods(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let scope = args.first().copied().flatten();
    let selection = receiver_selection(interp, receiver)?;
    let owner = interp.class_of_value(receiver).unwrap_or(ObjRef::NIL);
    let entries = if scope == Some(ObjRef::NIL) {
        object_level_entries(interp, receiver)
    } else {
        behaviour_entries(interp, selection, scope)
    };
    supplier_of(interp, owner, entries)
}

/// Which table a receiver's methods are read out of: an instance's own
/// behaviour, or a class object's class-side one.
#[derive(Copy, Clone)]
enum Selection {
    Behaviour(rexx_classes::BehaviourHandle),
    ClassSide(ObjRef),
}

/// The [`Selection`] for a receiver of any kind.
fn receiver_selection(interp: &mut Interp, receiver: ObjRef) -> Result<Selection, Failure> {
    let behaviour = interp
        .receiver_behaviour(receiver)
        .map_err(|kind| Failure::from(super::Loud::receiver_class(kind)))?;
    Ok(match behaviour {
        Behaviour::Instance { methods, .. } => Selection::Behaviour(methods),
        Behaviour::ClassSide(class) => Selection::ClassSide(class),
    })
}

/// One name's scope and method in `selection`.
fn lookup(interp: &mut Interp, selection: Selection, name: &str) -> Option<(ObjRef, MethodId)> {
    match selection {
        Selection::Behaviour(handle) => interp.classes().lookup_at(handle, name),
        Selection::ClassSide(class) => interp.classes().lookup_class_method(class, name),
    }
}

/// Every name `selection` resolves, filtered to `scope` when one is given.
fn behaviour_entries(
    interp: &mut Interp,
    selection: Selection,
    scope: Option<ObjRef>,
) -> Vec<(Vec<u8>, ObjRef, MethodId)> {
    let names = match selection {
        Selection::Behaviour(handle) => interp.classes().method_names_at(handle),
        Selection::ClassSide(class) => interp.classes().class_method_names(class),
    };
    let mut entries = Vec::new();
    for name in names {
        let Some((found, method)) = lookup(interp, selection, &name) else {
            continue;
        };
        if scope.is_none_or(|wanted| wanted == found) {
            entries.push((name.into_bytes(), found, method));
        }
    }
    entries
}

/// The receiver's own `setMethod` and `Class~enhanced` level, which a `.nil`
/// scope selects.
fn object_level_entries(interp: &Interp, receiver: ObjRef) -> Vec<(Vec<u8>, ObjRef, MethodId)> {
    let Some(rexx_core::Object {
        body: rexx_core::Body::Instance { own: Some(own), .. },
        ..
    }) = interp.heap.get(receiver)
    else {
        return Vec::new();
    };
    own.defined_names()
        .into_iter()
        .filter_map(|name| {
            let entry = own.get(&name)??;
            Some((name.into_vec(), entry.scope, entry.method))
        })
        .collect()
}

/// A `Supplier` whose items are `Method` objects and whose indexes are their
/// names, which is what the oracle's own supplier carries -- measured, its
/// `~index` is the name and its `~item~class~id` is `Method`.
fn supplier_of(
    interp: &mut Interp,
    owner: ObjRef,
    entries: Vec<(Vec<u8>, ObjRef, MethodId)>,
) -> Result<Option<ObjRef>, Failure> {
    let mut items = Vec::with_capacity(entries.len());
    let mut indexes = Vec::with_capacity(entries.len());
    for (name, scope, method) in entries {
        // Rooted as each is built rather than at the end: every allocation
        // below can collect, and a handle held only by these `Vec`s is one
        // the collector cannot see.
        let object = method_object_for(interp, owner, &name, scope, method);
        interp.roots.push_temp(object);
        items.push(object);
        let index = interp.text_built(name);
        interp.roots.push_temp(index);
        indexes.push(index);
    }
    let items = array_of(interp, items);
    interp.roots.push_temp(items);
    let indexes = array_of(interp, indexes);
    interp.roots.push_temp(indexes);
    new_supplier(interp, items, indexes).map(Some)
}

/// The `Method` object for one resolved entry, built the way `Class~method`
/// builds its own.
fn method_object_for(
    interp: &mut Interp,
    owner: ObjRef,
    name: &[u8],
    scope: ObjRef,
    method: MethodId,
) -> ObjRef {
    let record = crate::ExecutableRecord {
        source: interp.installed_executable_source(method),
        installed: Some(method),
        routine: None,
    };
    interp.method_object(owner, name, scope, record)
}
