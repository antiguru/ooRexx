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

//! `Class`'s own methods: its identity and graph readers, `define`, `subclass`,
//! `new`, `inherit`, the metaclass factory, and a package's `addClass`.

use super::{
    BehaviourId, Body, ClassKind, Cleared, Failure, INIT, InheritRefusal, Interp, Loud, MethodSlot,
    ObjRef, Object, ObjectMethod, ObjectMethodWrite, ObjectMethods, Package, Primitive, Raised,
    hash, required_string_named_argument,
};

/// `Class~baseClass`: `RexxClass::getBaseClass`, bound as a method of
/// `.Class` by `Setup.cpp:455`.
pub(super) fn native_base_class(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    Ok(Some(interp.classes().base_class(class)))
}

/// The class object a receiver whose messages resolve against a class's
/// **class** behaviour is, or the refusal for one that is not a class object.
pub(super) fn class_receiver(interp: &Interp, receiver: ObjRef) -> Result<ObjRef, Failure> {
    match interp.receiver_kind(receiver) {
        Ok(Primitive::Class(class)) => Ok(class),
        Ok(_) => Err(Loud::receiver_class("a value that is not a class object").into()),
        Err(kind) => Err(Loud::receiver_class(kind).into()),
    }
}

/// The argument a method that requires a class object was given --
/// `classArgument(other, TheClassClass, "class")`
/// (`runtime/MethodArguments.hpp:727`), which refuses an omitted argument
/// with 88.901 and a value that is not a class object with 88.914.
pub(super) fn class_argument(interp: &Interp, args: &[Option<ObjRef>]) -> Result<ObjRef, Failure> {
    let Some(Some(argument)) = args.first().copied() else {
        return Err(Raised::missing_named_argument("class").into());
    };
    match interp.heap.is_class(argument) {
        true => Ok(argument),
        false => Err(Raised::argument_not_a_class("class").into()),
    }
}

/// `Class~id`: the name the class was declared with, case unmodified --
/// `RexxClass::getId` (`classes/ClassClass.cpp:385`).
pub(super) fn native_id(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    let id = interp.class_id_text(class).as_bytes().to_vec();
    Ok(Some(interp.text_built(id)))
}

/// `Class~defaultName`: `The <id> class` -- `RexxClass::defaultNameRexx`
/// (`classes/ClassClass.cpp:614`).
pub(super) fn native_class_default_name(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    let name = interp.classes().default_name(class).as_bytes().to_vec();
    Ok(Some(interp.text_built(name)))
}

/// `Class~metaClass`: `RexxClass::getMetaClass` (`classes/ClassClass.cpp:419`).
pub(super) fn native_metaclass(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    Ok(Some(interp.classes().metaclass(class)))
}

/// `Class~superClass`: the class's own direct superclass, or `.nil` for
/// `.Object` -- `RexxClass::getSuperClass` (`classes/ClassClass.cpp:441`),
/// whose body is `superClasses->getFirstItem()`, so it reads the **first** entry
/// of the superclass list and not its last.
pub(super) fn native_superclass(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    Ok(Some(
        interp.classes().superclass(class).unwrap_or(ObjRef::NIL),
    ))
}

/// `Class~superClasses`: a **fresh** array of the class's own direct
/// superclasses -- `RexxClass::getSuperClasses`
/// (`classes/ClassClass.cpp:458`), whose body is `superClasses->copy()`, so it
/// hands out a copy rather than the class's own list.
pub(super) fn native_superclasses(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    let items: Vec<Option<ObjRef>> = interp
        .classes()
        .superclasses(class)
        .iter()
        .map(|class| Some(*class))
        .collect();
    // Every item is a class identity, which lives outside the arena
    // (`rexx_core::CLASS_SLOT_BASE`), so the allocation below cannot collect
    // one of them out from under this array.
    Ok(Some(
        interp.alloc_with(BehaviourId::ARRAY, Body::array(items)),
    ))
}

/// `Class~isSubclassOf(class)`: whether the receiver **is** `class` or derives
/// from it -- `RexxClass::isSubclassOf` (`classes/ClassClass.cpp:1692`),
/// which asks `isCompatibleWith` on the receiver itself.
pub(super) fn native_is_subclass_of(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let other = class_argument(interp, args)?;
    let class = class_receiver(interp, receiver)?;
    let answer = interp.classes().is_a(class, other);
    Ok(Some(interp.counted(usize::from(answer))))
}

/// `Class~annotation(name)`, and the same method at `Method`, `Routine` and
/// `Package`: the annotation `name` holds, or `.nil`.
pub(super) fn native_annotation(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let Some(Some(argument)) = args.first().copied() else {
        return Err(Raised::missing_named_argument("name").into());
    };
    let argument = required_string_named_argument(interp, argument, "name")?;
    let name = interp.to_text(argument).to_ascii_uppercase();
    let table = interp.annotations_of(receiver)?;
    Ok(Some(
        interp.native_entry(table, &name).unwrap_or(ObjRef::NIL),
    ))
}

/// `Class~annotations`, and the same method at `Method`, `Routine` and
/// `Package`: the receiver's own annotation table.
pub(super) fn native_annotations(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    Ok(Some(interp.annotations_of(receiver)?))
}

/// `Class~method(name)`: the method object `name` names **in this class's own
/// instance dictionary**, and 97.1 for anything else.
pub(super) fn native_method(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let Some(Some(argument)) = args.first().copied() else {
        return Err(Raised::missing_named_argument("method name").into());
    };
    let argument = required_string_named_argument(interp, argument, "method name")?;
    let name = interp.to_text(argument).to_ascii_uppercase();
    let class = class_receiver(interp, receiver)?;
    match interp
        .classes()
        .own_instance_slot(class, &String::from_utf8_lossy(&name))
    {
        None => {
            let target = interp.class_default_name(class).to_vec();
            Err(Raised::no_method(&target, &name).into())
        }
        Some(MethodSlot::Hidden) => Ok(Some(ObjRef::NIL)),
        Some(MethodSlot::Defined { scope, method }) => {
            let record = crate::ExecutableRecord {
                source: interp.installed_executable_source(method),
                installed: Some(method),
                routine: None,
            };
            Ok(Some(interp.method_object(class, &name, scope, record)))
        }
    }
}

/// `Method~scope`: the class the method object was defined at, `.nil` for a
/// method object no class has taken -- [`Interp::method_scope`] carries the
/// C++ and the measurements.
pub(super) fn native_scope(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    Ok(Some(interp.method_scope(receiver)?))
}

/// The `REXX_DEFINED` refusal every class mutator opens with --
/// `isRexxDefined()` and its `reportException(Error_Execution_rexx_defined_class)`
/// (`classes/ClassClass.cpp:823`, `:522`, `:955`, `:1290`, `:1382`, one per
/// mutator).
fn rexx_defined_lock(interp: &mut Interp, class: ObjRef) -> Result<(), Failure> {
    // **Open while the interpreter's own library runs**, which is the state
    // `Setup.cpp` builds the image in: `CoreClasses.orx:93` onwards is a run
    // of `~inherit` clauses against exactly the classes this flag guards,
    // and the C++ sets `REXX_DEFINED` on them at image-save time
    // (`RexxClass::liveGeneral`, `ClassClass.cpp:136`-`:142`) rather than
    // before. This crate flags each library class as `Interp::install_class`
    // creates it, so the bypass is not a convenience: it is what lets the
    // prologue's own `~inherit` clauses run at all. See
    // `Interp::library_bootstrap` for what else the same flag opens and for
    // why no program can be inside it.
    if !interp.library_bootstrap && interp.classes().is_rexx_defined(class) {
        return Err(Raised::rexx_defined_class().into());
    }
    Ok(())
}

/// The physical lines a method source is compiled from, or the refusal for
/// a value this crate cannot read as one -- `processExecutableSource`
/// (`execution/BaseExecutable.cpp:169`).
pub(super) fn method_source_lines(
    interp: &mut Interp,
    source: ObjRef,
    position: &'static str,
) -> Result<Vec<Vec<u8>>, Failure> {
    if let Ok(Primitive::Array) = interp.receiver_kind(source) {
        let slots = interp.array_slots_of(source).unwrap_or_default();
        let last_item = slots
            .iter()
            .rposition(Option::is_some)
            .map_or(0, |at| at + 1);
        let mut lines = Vec::with_capacity(last_item);
        for slot in &slots[..last_item] {
            let item = slot.filter(|item| is_source_line(interp, *item));
            let Some(item) = item else {
                return Err(Raised::method_source_not_all_strings(position).into());
            };
            lines.push(interp.to_text(item).to_vec());
        }
        return Ok(lines);
    }
    if is_source_line(interp, source) {
        return Ok(vec![interp.to_text(source).to_vec()]);
    }
    Err(Loud::method_from_source("a method source that is neither a string nor an array").into())
}

/// Whether one value is a source line -- the C++'s `isString(source)` for
/// the whole source and its `makeString()` for an array's items
/// (`execution/BaseExecutable.cpp:174`, `classes/StringClassUtil.cpp:428`).
fn is_source_line(interp: &Interp, value: ObjRef) -> bool {
    matches!(
        interp.receiver_kind(value),
        Ok(Primitive::String | Primitive::SmallInt)
    )
}

/// `MethodClass::newMethodObject`'s compiling arm
/// (`classes/MethodClass.cpp:462`-`:485`): the `Method` object a source text
/// becomes, carrying no scope, for a caller that is about to install it.
pub(super) fn compile_method_source(
    interp: &mut Interp,
    name: &[u8],
    source: ObjRef,
    position: &'static str,
) -> Result<ObjRef, Failure> {
    let lines = method_source_lines(interp, source, position)?;
    let borrowed: Vec<&[u8]> = lines.iter().map(Vec::as_slice).collect();
    let parsed = rexx_parse::parse_lines(&borrowed).map_err(|error| {
        Failure::from(Loud::method_from_source(&format!(
            "reporting a method source that does not parse ({}, {error})",
            String::from_utf8_lossy(name)
        )))
    })?;
    if !parsed.directives.is_empty() {
        return Err(Loud::method_from_source("a method source that carries a directive").into());
    }
    let method_class = interp.method_class();
    let object = interp.native_instance(method_class);
    // Every `Method` object this crate builds carries an annotation table, so
    // that `~annotations` and `~annotation` answer from the object rather
    // than from a way back to a directive. A compiled method has no directive
    // and its table starts empty, which is the oracle's own answer: measured,
    // rc 0, `.k~define("m", 'return 1')` then `.k~method("M")~annotation('x')`
    // is `The NIL object`.
    let site = crate::environment::Annotated::Compiled(interp.compiled_methods);
    interp.compiled_methods += 1;
    interp.attach_annotations(object, site);
    interp.record_compiled_body(object, name, parsed);
    Ok(object)
}

/// `RoutineClass::newRexx`'s compiling arm (`classes/RoutineClass.cpp:379`):
/// [`compile_method_source`] answering a `Routine` instead.
pub(super) fn compile_routine_source(
    interp: &mut Interp,
    name: &[u8],
    source: ObjRef,
    position: &'static str,
) -> Result<ObjRef, Failure> {
    let lines = method_source_lines(interp, source, position)?;
    let borrowed: Vec<&[u8]> = lines.iter().map(Vec::as_slice).collect();
    let parsed = rexx_parse::parse_lines(&borrowed).map_err(|error| {
        Failure::from(Loud::method_from_source(&format!(
            "reporting a routine source that does not parse ({}, {error})",
            String::from_utf8_lossy(name)
        )))
    })?;
    // The oracle installs them and this crate does not: measured, oracle rc
    // 0, `.Routine~new('R', <four lines with a ::ROUTINE among them>)~call`
    // resolves the declared routine and answers. Refused loudly here rather
    // than dropped, and it is the refusal [`compile_method_source`] already
    // makes over the same shape.
    if !parsed.directives.is_empty() {
        return Err(Loud::method_from_source("a routine source that carries a directive").into());
    }
    let routine_class = interp.routine_class();
    let object = interp.native_instance(routine_class);
    let site = crate::environment::Annotated::Compiled(interp.compiled_methods);
    interp.compiled_methods += 1;
    interp.attach_annotations(object, site);
    interp.record_compiled_routine(object, name, parsed);
    Ok(object)
}

/// The `method name` argument `~define`, `~delete` and `~method` share:
/// required, string-valued, and upcased before it reaches a dictionary --
/// `stringArgument(method_name, "method name")->upper()`
/// (`classes/ClassClass.cpp:831`-`:832`, `:961`, `:987`).
pub(super) fn method_name_argument(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
) -> Result<Vec<u8>, Failure> {
    Ok(method_name_pair(interp, args)?.1)
}

/// [`method_name_argument`] with the spelling the caller wrote kept beside
/// the dictionary key.
fn method_name_pair(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
) -> Result<(Vec<u8>, Vec<u8>), Failure> {
    let Some(Some(argument)) = args.first().copied() else {
        return Err(Raised::missing_named_argument("method name").into());
    };
    let argument = required_string_named_argument(interp, argument, "method name")?;
    let written = interp.to_text(argument).to_vec();
    let key = written.to_ascii_uppercase();
    Ok((written, key))
}

/// `Class~define(name, method)`: install one instance method on the receiver
/// -- `RexxClass::defineMethod` (`classes/ClassClass.cpp:819`).
pub(super) fn native_define(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    rexx_defined_lock(interp, class)?;
    let (written, name) = method_name_pair(interp, args)?;
    match args.get(1).copied().flatten() {
        None => {
            interp
                .classes()
                .hide_instance_method(class, &String::from_utf8_lossy(&name));
            interp.drop_method_object(class, &name);
        }
        Some(source) if source == ObjRef::NIL => {
            interp
                .classes()
                .delete_instance_method(class, &String::from_utf8_lossy(&name));
            interp.drop_method_object(class, &name);
        }
        Some(source) => {
            let source = if interp.receiver_kind(source) == Ok(Primitive::Method) {
                source
            } else {
                compile_method_source(interp, &written, source, "method")?
            };
            interp
                .define_method_object(class, &name, source)
                .ok_or_else(|| {
                    Failure::from(Loud::receiver_class(
                        "a method object this crate did not build",
                    ))
                })?;
        }
    }
    Ok(None)
}

/// `Class~defineMethods(methods)`: install a whole table of instance methods
/// in one mutation -- `RexxClass::defineMethodsRexx`
/// (`classes/ClassClass.cpp:518`).
pub(super) fn native_define_methods(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    rexx_defined_lock(interp, class)?;
    let Some(Some(table)) = args.first().copied() else {
        return Err(Raised::missing_named_argument("methods").into());
    };
    if !string_keyed_table(interp, table) {
        return Err(supplier_refusal(interp, table));
    }
    // **Asked before the walk**, because the walk cannot see the difference:
    // a collection whose entries this crate answers per name rather than
    // storing looks empty to `native_keys`, and a mutation over an empty walk
    // is a mutation that silently does nothing.
    if let Some(owner) = interp.unbuilt_collection_owner(table) {
        return Err(Loud::unreadable_collection(owner).into());
    }
    let mut names = interp.native_keys(table);
    names.sort();
    let mut entries: Vec<(Box<[u8]>, Option<ObjRef>)> = Vec::with_capacity(names.len());
    for name in names {
        let value = interp.native_entry(table, &name).unwrap_or(ObjRef::NIL);
        if value == ObjRef::NIL {
            entries.push((name, None));
            continue;
        }
        let value = if interp.receiver_kind(value) == Ok(Primitive::Method) {
            value
        } else {
            // The entry's own index is the method's name, as stored:
            // `createMethodDictionary` builds the object under
            // `supplier->index()->requestString()` and keys the dictionary
            // under its upcase (`classes/ClassClass.cpp:1255`-`:1258`).
            compile_method_source(interp, &name, value, "method source")?
        };
        entries.push((name, Some(value)));
    }
    interp.define_method_table(class, &entries).ok_or_else(|| {
        Failure::from(Loud::receiver_class(
            "a method object this crate did not build",
        ))
    })?;
    Ok(None)
}

/// Whether `value` is a table `~defineMethods`, `~enhanced` and `~setMethod`
/// can walk: one of this crate's own `Body::Native` directories, or a
/// `Directory` or `StringTable` a program made, which since Phase 5h Task 4
/// is a collection with a hash store.
fn string_keyed_table(interp: &mut Interp, value: ObjRef) -> bool {
    if matches!(
        interp.receiver_kind(value),
        Ok(Primitive::Directory | Primitive::StringTable(_))
    ) {
        return true;
    }
    hash::owns(interp, value) && hash::string_keyed(interp, value)
}

/// What a `SUPPLIER` send to `table` would have answered, as the failure
/// `~defineMethods` reports for a value it cannot walk.
fn supplier_refusal(interp: &mut Interp, table: ObjRef) -> Failure {
    match interp.lookup_for_refusal(table, b"SUPPLIER") {
        Some(scope) => Loud::native_method(b"SUPPLIER", &scope).into(),
        None => {
            let target = interp.string_value_text(table);
            Raised::no_method(&target, b"SUPPLIER").into()
        }
    }
}

/// `Class~subclass(id, metaclass, classMethods)`: build a class derived from
/// the receiver and answer it -- `RexxClass::subclassRexx`
/// (`classes/ClassClass.cpp:1543`), which forwards to the same
/// `RexxClass::subclass` a `::CLASS` directive calls, with `OREF_NULL` where
/// the directive passes its package (`:1546`).
pub(super) fn native_subclass(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    class_factory(interp, receiver, args, ClassKind::Regular)
}

/// `Class~mixinClass(id, metaclass, classMethods)`: the same factory, marking
/// what it builds a mixin -- `RexxClass::mixinClassRexx`
/// (`classes/ClassClass.cpp:1493`).
pub(super) fn native_mixin_class_factory(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    class_factory(interp, receiver, args, ClassKind::Mixin)
}

/// `Object~new`: what every class that declares no `NEW` of its own answers.
pub(super) fn native_new(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    let object = new_instance(interp, class)?;
    let caller = interp.caller();
    interp.send_message(object, INIT, None, args, caller)?;
    Ok(Some(object))
}

/// `RexxClass::completeNewObject` (`classes/ClassClass.cpp:1882`) up to but
/// not including the `INIT` send: the abstract check, the behaviour, the
/// rooting and the `UNINIT` registration, in that order.
pub(super) fn new_instance(interp: &mut Interp, class: ObjRef) -> Result<ObjRef, Failure> {
    if interp.classes().is_abstract(class) {
        let id = interp.classes().id_string(class).as_bytes().to_vec();
        return Err(Raised::abstract_class(&id).into());
    }
    // The object's behaviour is set from the class here, once. What the class
    // holds later is a different behaviour or the same one rebuilt, and D58
    // turns on which.
    let behaviour = interp.classes().instance_behaviour_handle(class);
    let object = interp.alloc_with(
        rexx_core::BehaviourId::OBJECT,
        Body::Instance {
            class,
            behaviour,
            name: None,
            pools: rexx_core::ScopePools::new(),
            own: None,
            native: None,
        },
    );
    // `ProtectedObject p(newObj)` (`ObjectClass.cpp:2637`): the `INIT` send
    // allocates, and nothing else holds the object until it returns.
    interp.roots.push_temp(object);
    // See `Interp::reqstr_armed` for why an instance arms the protocol
    // outright rather than by the name a directive installed.
    interp.reqstr_armed = true;
    if interp.classes().has_uninit(class) {
        interp.heap.set_uninit(object);
    }
    Ok(object)
}

/// `RexxClass::subclass` (`classes/ClassClass.cpp:1562`), in its own order.
fn class_factory(
    interp: &mut Interp,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
    kind: ClassKind,
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    let metaclass = factory_metaclass(interp, class, args)?;
    let name = class_id_argument(interp, args)?;
    let id = interp.mint_class();
    interp.classes().define_unregistered_class(
        id,
        &String::from_utf8_lossy(&name),
        Some(class),
        kind,
        metaclass,
    );
    interp.record_packageless_class(id);
    if let Some(enhancing) = args.get(2).copied().flatten() {
        enhance_class_methods(interp, id, enhancing)?;
    }
    // `RexxClass::subclass`'s own tail, in its order (`:1615`-`:1637`), which
    // is the same sequence `Interp::install_class_at` makes for a directive.
    interp.classes().check_uninit(id);
    interp.flag_class_uninit(id);
    let caller = interp.caller();
    interp.send_message(id, INIT, None, &[], caller)?;
    interp.classes().refresh_parent_has_uninit(id);
    Ok(Some(id))
}

/// The metaclass a class factory builds from: the second argument, or the
/// receiver's own where the send omits it (`classes/ClassClass.cpp:1566`-
/// `:1569`).
pub(super) fn factory_metaclass(
    interp: &mut Interp,
    class: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    let metaclass = match args.get(1).copied().flatten() {
        None => interp.classes().metaclass(class),
        Some(named) => named,
    };
    let is_class = matches!(interp.receiver_kind(metaclass), Ok(Primitive::Class(_)));
    if !is_class || !interp.classes().is_metaclass(metaclass) {
        let shown = interp.string_value_text(metaclass);
        return Err(Raised::bad_metaclass(&shown).into());
    }
    // **The class object itself is built by a `NEW` send to the metaclass**
    // (`:1579`), so a metaclass carrying its own `NEW` decides what gets
    // built and what its arguments mean. This crate implements no `NEW` at
    // all; what it models is the one `.Class` declares, by constructing the
    // class in `class_factory` instead of sending. A resolution landing
    // anywhere else is a class this crate cannot build, and it refuses rather
    // than build the one `.Class` would have -- measured, oracle rc 0:
    // `::CLASS MyMeta SUBCLASS Class` with `::METHOD new CLASS` runs that
    // body for `.object~subclass("k", .MyMeta)`.
    let modelled = interp.object_model().metaclass;
    let resolved = interp
        .classes()
        .lookup_class_method(metaclass, "NEW")
        .map(|(scope, _)| scope);
    if resolved != Some(modelled) {
        let scope = interp.classes().id_string(metaclass).to_string();
        return Err(Loud::native_method(b"NEW", &scope).into());
    }
    Ok(metaclass)
}

/// The `class id` argument, and the traceback frame the oracle's own `NEW`
/// activation contributes when it refuses.
fn class_id_argument(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<Vec<u8>, Failure> {
    let outcome = required_class_id(interp, args);
    if outcome.is_err() {
        interp.blame_native_method(b"NEW", "Class");
    }
    outcome
}

/// [`class_id_argument`] without the frame, so that every way of failing
/// takes it.
fn required_class_id(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<Vec<u8>, Failure> {
    let Some(Some(argument)) = args.first().copied() else {
        return Err(Raised::missing_named_argument("class id").into());
    };
    let argument = required_string_named_argument(interp, argument, "class id")?;
    Ok(interp.to_text(argument).to_vec())
}

/// The third argument: class-side methods the new class is built with --
/// `createMethodDictionary(enhancing_methods, new_class)` merged into
/// `classMethodDictionary` (`classes/ClassClass.cpp:1602`-`:1608`).
fn enhance_class_methods(
    interp: &mut Interp,
    class: ObjRef,
    enhancing: ObjRef,
) -> Result<(), Failure> {
    if !string_keyed_table(interp, enhancing) {
        return Err(supplier_refusal(interp, enhancing));
    }
    if let Some(owner) = interp.unbuilt_collection_owner(enhancing) {
        return Err(Loud::unreadable_collection(owner).into());
    }
    let mut names = interp.native_keys(enhancing);
    names.sort();
    // One frame around the whole walk, for the reason
    // [`Interp::define_method_table`] has one: each entry allocates the copy
    // `newScope` makes, and the temporary rooting that copy carries until
    // `hold_method_object` roots it as a global has to be released somewhere.
    let frame = interp.roots.push_frame();
    let installed = install_enhancing_class_methods(interp, class, enhancing, &names);
    interp.roots.pop_frame(frame);
    installed
}

/// [`enhance_class_methods`]'s walk, split out so the root frame it runs
/// inside is released on the failure path too.
fn install_enhancing_class_methods(
    interp: &mut Interp,
    class: ObjRef,
    enhancing: ObjRef,
    names: &[Box<[u8]>],
) -> Result<(), Failure> {
    for name in names {
        let value = interp.native_entry(enhancing, name).unwrap_or(ObjRef::NIL);
        if interp.receiver_kind(value) != Ok(Primitive::Method) {
            // **Read as source before declining it**, so that a source the
            // oracle refuses is refused here the same way rather than
            // reaching the loud arm below: measured, oracle rc 163, a
            // literal array `('return 1', , 'nop')` in this table is
            // `93.952 Method argument method source is an array and does
            // not contain all string values.`
            method_source_lines(interp, value, "method source")?;
            return Err(Loud::method_from_source("a class method built from source text").into());
        }
        interp
            .define_class_method_object(class, name, value)
            .ok_or_else(|| {
                Failure::from(Loud::method_from_source(
                    "a class method whose body this crate does not hold",
                ))
            })?;
    }
    Ok(())
}

/// `Class~delete(name)`: take one instance method back off the receiver --
/// `RexxClass::deleteMethod` (`classes/ClassClass.cpp:952`).
pub(super) fn native_delete(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    rexx_defined_lock(interp, class)?;
    let name = method_name_argument(interp, args)?;
    interp
        .classes()
        .delete_instance_method(class, &String::from_utf8_lossy(&name));
    interp.drop_method_object(class, &name);
    Ok(None)
}

/// `Class~inherit(mixin, position)`: add a mixin to the receiver's
/// superclass list -- `RexxClass::inherit` (`classes/ClassClass.cpp:1287`),
/// the same function the `INHERIT` keyword of a `::CLASS` directive reaches
/// by sending this message ([`Interp::inherit_mixin`]).
pub(super) fn native_class_inherit(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    rexx_defined_lock(interp, class)?;
    let mixin = mixin_class_argument(interp, args)?;
    // **The position is not type-checked, and that is the oracle's shape.**
    // `RexxClass::inherit` uses it in one place, `superClasses->indexOf(position)`
    // (`classes/ClassClass.cpp:1346`), and a value that is not in the list gives
    // index 0 and `Error_Execution_uninherit` at `:1350` whatever kind of
    // object it was. Measured at rc 158, `::class K` inheriting `.M1`:
    // `.K~inherit(.M2, "abc")` is `98.945`, `Class "The K class" has not
    // inherited class "abc".` -- the second argument's error and not the
    // first's.
    let position = args.get(1).copied().flatten();
    match interp.classes().inherit_at(class, mixin, position) {
        Ok(()) => Ok(None),
        Err(refusal) => Err(inherit_refusal(interp, class, mixin, refusal)),
    }
}

/// `Class~defineClassMethod(name, method)`: install `method` as a class-side
/// method of the receiver -- `RexxClass::defineClassMethod`
/// (`classes/ClassClass.cpp:883`), which writes the class behaviour and
/// `classMethodDictionary` from one `newScope` copy.
pub(super) fn native_define_class_method(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    let name = method_name_argument(interp, args)?;
    let Some(Some(source)) = args.get(1).copied() else {
        return Err(Loud::setup_method("defineClassMethod with no method object").into());
    };
    if interp.receiver_kind(source) != Ok(Primitive::Method) {
        return Err(
            Loud::setup_method("defineClassMethod with a value that is not a method").into(),
        );
    }
    interp
        .define_class_method_object(class, &name, source)
        .ok_or_else(|| {
            Failure::from(Loud::setup_method(
                "defineClassMethod with a method object whose body this crate did not record",
            ))
        })?;
    Ok(None)
}

/// `Class~inheritInstanceMethods(source)`: copy `source`'s own instance
/// methods into the receiver's dictionary at the receiver's scope, with no
/// superclass edge added -- `RexxClass::inheritInstanceMethods`
/// (`classes/ClassClass.cpp:558`), the "phony inherit" `CoreClasses.orx:77`
/// names in its own comment.
pub(super) fn native_inherit_instance_methods(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    let Some(Some(argument)) = args.first().copied() else {
        return Err(Loud::setup_method("inheritInstanceMethods with no source class").into());
    };
    let Some(source) = interp.heap.is_class(argument).then_some(argument) else {
        return Err(
            Loud::setup_method("inheritInstanceMethods with a value that is not a class").into(),
        );
    };
    let source = class_receiver(interp, source)?;
    interp.classes().inherit_instance_methods(class, source);
    interp.classes().check_uninit(class);
    interp.flag_class_uninit(class);
    Ok(None)
}

/// `Class~uninherit(mixin)`: take a mixin back out of the receiver's
/// superclass list -- `RexxClass::uninherit` (`classes/ClassClass.cpp:1379`).
pub(super) fn native_uninherit(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    rexx_defined_lock(interp, class)?;
    let mixin = mixin_class_argument(interp, args)?;
    match interp.classes().uninherit(class, mixin) {
        Ok(()) => Ok(None),
        Err(refusal) => Err(inherit_refusal(interp, class, mixin, refusal)),
    }
}

/// The `mixin class` argument `~inherit` and `~uninherit` share: required,
/// and a `MIXINCLASS` class object or 98.942 naming the value
/// (`classes/ClassClass.cpp:1298`-`:1301`, `:1391`-`:1394`).
fn mixin_class_argument(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<ObjRef, Failure> {
    let Some(Some(argument)) = args.first().copied() else {
        return Err(Raised::missing_named_argument("mixin class").into());
    };
    class_receiver(interp, argument).map_err(|_| {
        let shown = interp.string_value_text(argument);
        Failure::from(Raised::inherit_needs_a_mixinclass(&shown))
    })
}

/// One [`InheritRefusal`] as the condition the oracle reports in its place,
/// with the substitutions rendered the way the message renders an object:
/// `stringValue()`, which is `~objectName` and so follows a rename.
fn inherit_refusal(
    interp: &mut Interp,
    class: ObjRef,
    mixin: ObjRef,
    refusal: InheritRefusal,
) -> Failure {
    let class_name = interp.string_value_text(class);
    let mixin_name = interp.string_value_text(mixin);
    match refusal {
        InheritRefusal::NotAMixin => Raised::inherit_needs_a_mixinclass(&mixin_name).into(),
        InheritRefusal::Recursive => Raised::recursive_inherit(&class_name, &mixin_name).into(),
        InheritRefusal::BaseClass(base) => {
            let base_name = interp.string_value_text(base);
            Raised::inherit_base_class(&class_name, &mixin_name, &base_name).into()
        }
        InheritRefusal::NotInherited(other) => {
            let other_name = interp.string_value_text(other);
            Raised::not_inherited(&class_name, &other_name).into()
        }
    }
}

/// `Package~addClass(name, class)` and `Package~addPublicClass(name, class)`
/// -- `PackageClass::addClassRexx` (`classes/PackageClass.cpp:1926`) and
/// `addPublicClassRexx` (`:1944`), which differ only in the flag they hand
/// `addInstalledClass`.
pub(super) fn native_package_add_class(
    interp: &mut Interp,
    cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    add_installed_class(interp, cleared, receiver, args, false)
}

/// `Package~addPublicClass` -- see [`native_package_add_class`].
pub(super) fn native_package_add_public_class(
    interp: &mut Interp,
    cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    add_installed_class(interp, cleared, receiver, args, true)
}

/// The body both `~addClass` rows share, since a second implementation is
/// where the two could come to disagree.
fn add_installed_class(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
    public: bool,
) -> Result<Option<ObjRef>, Failure> {
    let Some(Some(name)) = args.first().copied() else {
        return Err(Raised::missing_named_argument("name").into());
    };
    let name = required_string_named_argument(interp, name, "name")?;
    let name = interp.to_text(name).to_vec();
    let Some(Some(class)) = args.get(1).copied() else {
        return Err(Raised::missing_named_argument("class").into());
    };
    if !interp.heap.is_class(class) {
        return Err(Raised::argument_not_a_class("class").into());
    }
    match interp.which_package(receiver) {
        Some(Package::Program(program)) => {
            interp.add_installed_class(program, &name, class, public);
            Ok(Some(receiver))
        }
        Some(Package::Rexx) => Err(Raised::rexx_package_addition().into()),
        None => Err(Loud::receiver_class("a package object this crate did not build").into()),
    }
}

/// `Class~package`: the package the class was defined in --
/// `RexxClass::getPackage` (`classes/ClassClass.cpp:1732`).
pub(super) fn native_package(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    Ok(Some(interp.package_object_for(class)))
}

/// `Class~copy`: the refusal `memory/Setup.cpp:483` puts over [`super::native_copy`]
/// -- `RexxClass::copyRexx` (`classes/ClassClass.cpp:166`), whose whole body
/// is the raise.
pub(super) fn native_class_copy(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let target = interp.string_value_text(receiver);
    Err(Raised::copy_not_supported(&target).into())
}

/// `Class~enhanced(methods, ...)`: an instance of the receiver carrying
/// methods of its own -- `RexxClass::enhanced`
/// (`classes/ClassClass.cpp:1440`).
pub(super) fn native_enhanced(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    // The refusals are different errors and the C++ checks them in this
    // order (`:1443`-`:1451`). Measured: `.K~enhanced` and `.K~enhanced()`
    // are both `93.901 Not enough arguments for method; 1 expected.` at rc
    // 163, the second because a trailing omission is dropped from the count,
    // while `.K~enhanced(, 'x')` is `88.901 Missing argument; argument
    // methods is required.` at rc 168.
    if args.is_empty() {
        return Err(Raised::not_enough_method_arguments(1).into());
    }
    let Some(table) = args[0] else {
        return Err(Raised::missing_named_argument("methods").into());
    };
    if !string_keyed_table(interp, table) {
        return Err(supplier_refusal(interp, table));
    }
    if let Some(owner) = interp.unbuilt_collection_owner(table) {
        return Err(Loud::unreadable_collection(owner).into());
    }
    let mut names = interp.native_keys(table);
    names.sort();
    let object = new_instance(interp, class)?;
    let frame = interp.roots.push_frame();
    let installed = install_enhancing_object_methods(interp, object, table, &names);
    interp.roots.pop_frame(frame);
    installed?;
    // `enhanced_object->behaviour->setEnhanced()` (`:1478`), which
    // `RexxObject::defaultName` reads. **Unconditional, and not folded into
    // the walk above**: an empty table installs no method and still renders
    // `enhanced <id>` -- measured, oracle rc 0, `k~enhanced(.StringTable~new)`
    // answers `enhanced K` for `~string` and `~defaultName` alike.
    mark_enhanced_instance(interp, object);
    // `dummy_subclass->sendMessage(GlobalNames::NEW, args + 1, ...)`
    // (`:1470`), whose `INIT` send is `completeNewObject`'s -- the enhancing
    // `INIT` by then, since it is already in the behaviour.
    let caller = interp.caller();
    interp.send_message(object, INIT, None, &args[1..], caller)?;
    Ok(Some(object))
}

/// Marks `object` as one [`native_enhanced`] built, creating its
/// [`ObjectMethods`] when an empty enhancing table left it without one.
fn mark_enhanced_instance(interp: &mut Interp, object: ObjRef) {
    if let Some(Object {
        body: Body::Instance { own, .. },
        ..
    }) = interp.heap.get_mut(object)
    {
        own.get_or_insert_with(|| Box::new(ObjectMethods::new()))
            .mark_enhanced_instance();
    }
}

/// Whether `Class~enhanced` built `value`, which
/// [`crate::environment::default_object_name`] is not given because the
/// answer takes no article.
pub(super) fn is_enhanced_instance(interp: &Interp, value: ObjRef) -> bool {
    matches!(
        interp.heap.get(value).map(|object| &object.body),
        Some(Body::Instance { own: Some(own), .. }) if own.is_enhanced_instance()
    )
}

/// [`native_enhanced`]'s walk, split out so the root frame it runs inside is
/// released on the failure path too --
/// [`install_enhancing_class_methods`]'s shape.
fn install_enhancing_object_methods(
    interp: &mut Interp,
    object: ObjRef,
    table: ObjRef,
    names: &[Box<[u8]>],
) -> Result<(), Failure> {
    for name in names {
        let value = interp.native_entry(table, name).unwrap_or(ObjRef::NIL);
        let source = if interp.receiver_kind(value) == Ok(Primitive::Method) {
            value
        } else {
            compile_method_source(interp, name, value, "method source")?
        };
        let Some(body) = interp.table_method_bodies.get(&source).copied() else {
            return Err(Loud::method_from_source(
                "an enhancing method whose body this crate does not hold",
            )
            .into());
        };
        let method = interp.classes().mint_method_id();
        interp.method_bodies.insert(method, body);
        interp.write_object_method(
            object,
            name,
            ObjectMethodWrite::Enhance(ObjectMethod {
                method,
                scope: ObjRef::NIL,
            }),
        )?;
    }
    Ok(())
}
