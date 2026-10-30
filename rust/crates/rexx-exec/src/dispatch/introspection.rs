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
    let selection = Selection::Behaviour(handle);
    let entries = behaviour_entries(interp, selection, scope);
    supplier_of(interp, selection, class, entries)
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
    // The object's own dictionary answers first: the object `setMethod`
    // installed, or `.nil` for a name it hides.
    if let Some(entry) = interp.own_method_entry(receiver, &name) {
        return Ok(Some(entry.map_or(ObjRef::NIL, |entry| entry.executable)));
    }
    let selection = receiver_selection(interp, receiver)?;
    let owner = interp.class_of_value(receiver).unwrap_or(ObjRef::NIL);
    let Some((scope, method)) = lookup(interp, selection, &String::from_utf8_lossy(&name)) else {
        return Ok(Some(ObjRef::NIL));
    };
    Ok(Some(method_object_for(
        interp, selection, owner, &name, scope, method,
    )?))
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
    let mut entries = object_level_entries(interp, receiver, scope);
    entries.extend(behaviour_entries(interp, selection, scope));
    supplier_of(interp, selection, owner, entries)
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
        .map_err(|kind| Failure::from(super::Loud::unknown_receiver(kind)))?;
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

/// One method a supplier lists: its name, its scope and its method, and the
/// `Method` object it already has, where it has one.
struct Listed {
    name: Vec<u8>,
    scope: ObjRef,
    method: MethodId,
    object: Option<ObjRef>,
}

/// Every method entry of `selection`, shadowed ones included, filtered to
/// `scope` when one is given: `MethodDictionary::getMethods`
/// (`behaviour/MethodDictionary.cpp:507`).
fn behaviour_entries(
    interp: &mut Interp,
    selection: Selection,
    scope: Option<ObjRef>,
) -> Vec<Listed> {
    let entries = match selection {
        Selection::Behaviour(handle) => interp.classes().defined_entries_at(handle),
        Selection::ClassSide(class) => interp.classes().class_defined_entries(class),
    };
    entries
        .into_iter()
        .filter(|(_, found, _)| scope.is_none_or(|wanted| wanted == *found))
        .map(|(name, scope, method)| Listed {
            name: name.into_bytes(),
            scope,
            method,
            object: None,
        })
        .collect()
}

/// The methods `setMethod` gave the receiver itself, filtered to `scope` when
/// one is given, each with the object `setMethod` installed.
fn object_level_entries(interp: &Interp, receiver: ObjRef, scope: Option<ObjRef>) -> Vec<Listed> {
    // A class object's own methods sit on the object holding its variable
    // pools, where `Interp::own_method_entry` reads them.
    let holder = match interp.heap.get(receiver) {
        Some(rexx_core::Object {
            body: rexx_core::Body::Class { .. },
            ..
        }) => interp.class_variables.get(&receiver).copied(),
        _ => Some(receiver),
    };
    let Some(rexx_core::Object {
        body: rexx_core::Body::Instance { own: Some(own), .. },
        ..
    }) = holder.and_then(|holder| interp.heap.get(holder))
    else {
        return Vec::new();
    };
    own.defined_names()
        .into_iter()
        .filter_map(|name| {
            let entry = own.get(&name)??;
            scope
                .is_none_or(|wanted| wanted == entry.scope)
                .then(|| Listed {
                    name: name.into_vec(),
                    scope: entry.scope,
                    method: entry.method,
                    object: Some(entry.executable),
                })
        })
        .collect()
}

/// A `Supplier` whose items are `Method` objects and whose indexes are their
/// names, which is what the oracle's own supplier carries -- measured, its
/// `~index` is the name and its `~item~class~id` is `Method`.
fn supplier_of(
    interp: &mut Interp,
    selection: Selection,
    owner: ObjRef,
    entries: Vec<Listed>,
) -> Result<Option<ObjRef>, Failure> {
    let mut items = Vec::with_capacity(entries.len());
    let mut indexes = Vec::with_capacity(entries.len());
    for Listed {
        name,
        scope,
        method,
        object,
    } in entries
    {
        // Rooted as each is built rather than at the end: every allocation
        // below can collect, and a handle held only by these `Vec`s is one
        // the collector cannot see.
        let object = match object {
            Some(object) => object,
            None => method_object_for(interp, selection, owner, &name, scope, method)?,
        };
        interp.roots.activity_mut().push_temp(object);
        items.push(object);
        let index = interp.text_built(name);
        interp.roots.activity_mut().push_temp(index);
        indexes.push(index);
    }
    let items = array_of(interp, items);
    interp.roots.activity_mut().push_temp(items);
    let indexes = array_of(interp, indexes);
    interp.roots.activity_mut().push_temp(indexes);
    new_supplier(interp, items, indexes).map(Some)
}

/// The `Method` object for one resolved entry, built the way `Class~method`
/// builds its own. An entry is keyed by the class that defines it, so two
/// classes' methods of one name are two objects and a subclass reaches its
/// superclass's.
fn method_object_for(
    interp: &mut Interp,
    selection: Selection,
    owner: ObjRef,
    name: &[u8],
    scope: ObjRef,
    method: MethodId,
) -> Result<ObjRef, Failure> {
    // A `.nil`-scoped entry is one `setMethod` or `Class~enhanced` installed
    // from an object of its own, which is what the oracle's behaviour holds.
    if scope == ObjRef::NIL
        && let Some(object) = interp.executable_objects.get(&method).copied()
    {
        return Ok(object);
    }
    let record = crate::ExecutableRecord {
        source: interp.installed_executable_source(method),
        installed: Some(method),
        routine: None,
        owns_installed: false,
    };
    match selection {
        Selection::ClassSide(_) => interp.method_object(scope, true, name, scope, record),
        Selection::Behaviour(_) if interp.heap.is_class(scope) => {
            interp.method_object(scope, false, name, scope, record)
        }
        Selection::Behaviour(_) => interp.method_object(owner, false, name, scope, record),
    }
}

#[cfg(test)]
mod tests;
