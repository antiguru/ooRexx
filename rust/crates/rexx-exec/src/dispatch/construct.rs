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

//! The native `~new` methods no class's own module holds, the capacity
//! `~init`, and `Pointer`'s and `WeakReference`'s methods.

use super::{
    Arity, Body, Cleared, Failure, INIT, Interp, Loud, NativeMethod, ObjRef, Package, Raised,
    class_receiver, compile_method_source, compile_routine_source, decode_message_name, hash,
    native, native_new, new_instance, optional_length_argument, required_string_argument,
    required_string_named_argument,
};

/// `Pointer`'s and `WeakReference`'s instance methods. Chained into
/// `ObjectModel::build` beside [`super::NATIVE_METHODS`].
pub(super) static NATIVE_METHODS: &[(&str, &str, Arity, NativeMethod)] = &[
    // `memory/Setup.cpp:1645`-`:1649`. `=` and `==` name one function and
    // `\\=` and `\\==` name the other, each a row rather than a second
    // implementation for the reason `Object`'s pairs give.
    ("Pointer", "=", Arity::Fixed(1), native_pointer_equal),
    ("Pointer", "==", Arity::Fixed(1), native_pointer_equal),
    ("Pointer", "\\=", Arity::Fixed(1), native_pointer_not_equal),
    ("Pointer", "\\==", Arity::Fixed(1), native_pointer_not_equal),
    ("Pointer", "ISNULL", Arity::Fixed(0), native_pointer_is_null),
    // `AddMethod("Value", WeakReference::value, 0)`, `memory/Setup.cpp:1686`.
    (
        "WeakReference",
        "VALUE",
        Arity::Fixed(0),
        native_weak_reference_value,
    ),
];

/// `StringTable~new` and `Directory~new`: an empty hash collection --
/// `StringTable::newRexx` and `DirectoryClass::newRexx`
/// (`memory/Setup.cpp:875`, `:928`), each of which allocates and then sends
/// `INIT` with the whole argument list.
pub(super) fn native_hash_collection_new(
    interp: &mut Interp,
    cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    // **A `Directory` a program makes is a collection, not the environment.**
    // `NativeObject`'s map holds its keys already uppercased, by its callers,
    // and a `.Directory~new` does not: measured, `d['lower'] = 1` leaves
    // `allIndexes` reading `lower` and `d['LOWER']` answering `.nil`. So a
    // collection gets the object-keyed store with a string-key protocol on
    // top.
    hash::native_hash_new(interp, cleared, receiver, args)
}

/// `Directory~new`: the hash body above for `.Directory` itself, and
/// [`native_new`]'s plain instance for a subclass.
pub(super) fn native_directory_new(
    interp: &mut Interp,
    cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    if class == interp.object_model().directory {
        native_hash_collection_new(interp, cleared, receiver, args)
    } else {
        native_new(interp, cleared, receiver, args)
    }
}

/// A collection's `~init(size)`: the initial-size argument, validated and
/// then dropped -- `HashCollection::initRexx`
/// (`classes/support/HashCollection.cpp:63`), and the `QueueClass::initRexx`
/// and `ListClass::initRexx` beside it (`classes/QueueClass.cpp:261`,
/// `classes/ListClass.cpp:102`), which differ only in the default capacity.
pub(super) fn native_capacity_init(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    optional_length_argument(interp, args, 0)?;
    Ok(None)
}

/// The refusal a constructor that has checked its arguments and cannot build
/// the primitive body answers.
fn unbuilt_new(interp: &mut Interp, class: ObjRef) -> Failure {
    unbuilt_class_method(interp, class, b"NEW")
}

/// [`unbuilt_new`] for a class method under some other name.
fn unbuilt_class_method(interp: &mut Interp, class: ObjRef, name: &[u8]) -> Failure {
    let id = interp.classes().id_string(class).to_string();
    Loud::native_method(name, &id).into()
}

/// `Class~new(id, ...)`: the class id is required and this crate builds no
/// class from it -- `RexxClass::newRexx` (`classes/ClassClass.cpp:1776`).
pub(super) fn native_class_new(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    if args.is_empty() {
        return Err(Raised::not_enough_method_arguments(1).into());
    }
    match args[0] {
        Some(id) => required_string_named_argument(interp, id, "class id")?,
        None => return Err(Raised::missing_named_argument("class id").into()),
    };
    Err(unbuilt_new(interp, class))
}

/// `Message~new(target, message, ...)`: a message object whose send has not
/// been made -- `MessageClass::newRexx` (`classes/MessageClass.cpp:828`).
pub(super) fn native_message_new(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    if args.len() < 2 {
        return Err(Raised::not_enough_method_arguments(2).into());
    }
    if args[0].is_none() {
        return Err(Raised::missing_named_argument("message target").into());
    }
    decode_message_name(interp, args[1])?;
    if args.len() > 2 || class != interp.object_model().message {
        return Err(unbuilt_new(interp, class));
    }
    let object = interp.native_instance(class);
    let caller = interp.caller();
    interp.send_message(object, INIT, None, &[], caller)?;
    Ok(Some(object))
}

/// `Method~new(name, source, ...)` and `Routine~new(name, source, ...)`: both
/// arguments are required -- `BaseExecutable::processNewExecutableArgs`
/// (`execution/BaseExecutable.cpp:225`), which `MethodClass::newRexx` and
/// `RoutineClass::newRexx` share.
pub(super) fn native_executable_new(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    let name = executable_name_argument(interp, args)?;
    let Some(source) = args.get(1).copied().flatten() else {
        return Err(Raised::missing_named_argument("source").into());
    };
    let routine = class == interp.routine_class();
    if args.len() > 2 || !(routine || class == interp.method_class()) {
        return Err(unbuilt_new(interp, class));
    }
    let name = interp.to_text(name).to_vec();
    let object = if routine {
        compile_routine_source(interp, &name, source, "source")?
    } else {
        compile_method_source(interp, &name, source, "source")?
    };
    Ok(Some(object))
}

/// `Method~newFile(name [, context])` and `Routine~newFile(name [, context])`:
/// the executable a file's own text becomes -- `MethodClass::newFileRexx`
/// (`classes/MethodClass.cpp:521`) and `RoutineClass::newFileRexx`
/// (`classes/RoutineClass.cpp:341`).
pub(super) fn native_new_file(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    let name = executable_name_argument(interp, args)?;
    let routine = class == interp.routine_class();
    if !routine && class != interp.method_class() {
        return Err(unbuilt_new(interp, class));
    }
    let parent = match args.get(1).copied().flatten() {
        Some(context) => new_file_context(interp, context)?,
        None => None,
    };
    let name = interp.to_text(name).to_vec();
    interp.new_file_executable(&name, routine, parent).map(Some)
}

/// `newFile`'s context argument, resolved to the package the loaded file
/// resolves names against. `None` is the caller's own package, which is what
/// an absent argument and `"PROGRAMSCOPE"` both mean -- measured, the two
/// answer the caller's `::ROUTINE` alike at rc 0.
fn new_file_context(interp: &mut Interp, context: ObjRef) -> Result<Option<Package>, Failure> {
    let class = interp.class_of_value(context);
    let resolved = if class == Some(interp.package_class()) {
        interp.which_package(context)
    } else if class == Some(interp.routine_class()) || class == Some(interp.method_class()) {
        interp.executable_context_package(context)
    } else if class == Some(interp.string_class())
        && interp
            .to_text(context)
            .eq_ignore_ascii_case(b"PROGRAMSCOPE")
    {
        return Ok(None);
    } else {
        None
    };
    match resolved {
        Some(package) => Ok(Some(package)),
        // The found value is its string *conversion*, not its string value:
        // measured, `.array~of(1)` reports `found "1"` where `.NIL` reports
        // `found "The NIL object"`.
        None => {
            let found = interp.to_text(context).to_vec();
            Err(Raised::argument_not_in_list(
                b"NEWFILE",
                2,
                "\"PROGRAMSCOPE\", Method, Routine, or Package object",
                &found,
            )
            .into())
        }
    }
}

/// `MethodClass::loadExternalMethod(name, descriptor)` and
/// `RoutineClass::loadExternalRoutine` (`memory/Setup.cpp:1091`, `:1130`).
pub(super) fn native_load_external(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    let name = executable_name_argument(interp, args)?;
    let routine = class == interp.routine_class();
    if !routine && class != interp.method_class() {
        return Err(unbuilt_new(interp, class));
    }
    let Some(descriptor) = args.get(1).copied().flatten() else {
        return Err(Raised::missing_named_argument("descriptor").into());
    };
    let descriptor = required_string_named_argument(interp, descriptor, "descriptor")?;
    let descriptor = interp.to_text(descriptor).to_vec();
    let name = interp.to_text(name).to_vec();
    let Some((library, entry)) = external_specification(&descriptor, &name) else {
        return Err(Raised::bad_external_specification(&descriptor).into());
    };
    // **The keyword is case-insensitive and the library name is not**, which
    // is measured rather than assumed: oracle rc 0,
    // `library REXX file_separator` and `LiBrArY REXX file_separator` each
    // answer a `Method`, while `LIBRARY rexx file_separator` and
    // `LIBRARY Rexx file_separator` answer `.nil`.
    let rexx = library == b"REXX";
    // The procedure the shared code is keyed by, as [`crate::LibraryCodeKey`]
    // documents, or `None` where nothing answers the name.
    let procedure = if rexx {
        // The `REXX` package's routine table is `rexx_routines[]` and not
        // this registry (`runtime/InternalPackage.cpp:230`).
        if routine {
            let Some(row) = crate::internal_routines::rexx_package_routine(&entry) else {
                return Ok(Some(ObjRef::NIL));
            };
            let object = interp.native_instance(class);
            interp.record_rexx_routine_object(object, row);
            return Ok(Some(object));
        }
        native::entry_point(&entry).map(|_| entry.clone())
    } else {
        // **Neither a library that is not there nor a procedure it does not
        // export raises**: measured, oracle rc 0, `.Method~loadExternalMethod('x',
        // 'LIBRARY zorkolib RegExp_Parse')` and the same naming `rxregexp
        // NoSuchEntry` both answer `The NIL object`, and
        // `.Routine~loadExternalRoutine` answers it for the same two.
        match interp.resolve_library(&library) {
            crate::LibraryLoad::Loaded(loaded) => {
                if routine {
                    loaded.routine(&entry).map(|row| row.name.clone())
                } else {
                    loaded.method(&entry).map(|_| entry.clone())
                }
            }
            crate::LibraryLoad::Missing => None,
            // Measured, oracle rc 158: 98.982 on the first ask, as
            // `Package~loadLibrary` raises it.
            crate::LibraryLoad::Version => {
                return Err(Raised::library_version(&library).into());
            }
            crate::LibraryLoad::Raised(failure) => return Err(failure),
        }
    };
    let Some(procedure) = procedure else {
        return Ok(Some(ObjRef::NIL));
    };
    if rexx {
        let object = interp.native_instance(class);
        interp.record_native_executable(object);
        return Ok(Some(object));
    }
    let code = interp.library_code(crate::LibraryCodeKey {
        library,
        procedure,
        routine,
    });
    // A routine is the library's own object for the row, the one an
    // imported-routine table answers.
    if routine {
        return Ok(Some(interp.library_routine_object(code)));
    }
    let object = interp.native_instance(class);
    interp.record_loaded_executable(object, code);
    Ok(Some(object))
}

/// A `loadExternal*` descriptor split into its library and its entry point,
/// or `None` for one that is not an external name specification.
fn external_specification(descriptor: &[u8], name: &[u8]) -> Option<(Vec<u8>, Vec<u8>)> {
    let mut words = descriptor
        .split(|byte| byte.is_ascii_whitespace())
        .filter(|word| !word.is_empty());
    if !words.next()?.eq_ignore_ascii_case(b"LIBRARY") {
        return None;
    }
    let library = words.next()?.to_vec();
    let entry = words.next().map_or_else(|| name.to_vec(), <[u8]>::to_vec);
    if words.next().is_some() {
        return None;
    }
    Some((library, entry))
}

/// `Package~new(name, source, ...)`: the name is required, the source is not,
/// and this crate loads no package either way -- `PackageClass::newRexx`
/// (`classes/PackageClass.cpp:158`).
pub(super) fn native_package_new(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    executable_name_argument(interp, args)?;
    Err(unbuilt_new(interp, class))
}

/// The `name` argument `Method`, `Routine` and `Package` share --
/// `stringArgument(pgmname, "name")`.
fn executable_name_argument(
    interp: &mut Interp,
    args: &[Option<ObjRef>],
) -> Result<ObjRef, Failure> {
    match args.first().copied().flatten() {
        Some(name) => required_string_named_argument(interp, name, "name"),
        None => Err(Raised::missing_named_argument("name").into()),
    }
}

/// `String~new(value, ...)`: a string carrying `value`'s own bytes --
/// `RexxString::newRexx` (`classes/StringClass.cpp:2352`).
pub(super) fn native_string_new(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    let Some(Some(value)) = args.first().copied() else {
        return Err(Raised::missing_method_argument(1).into());
    };
    let text = required_string_argument(interp, value, 1)?;
    if class != interp.object_model().string {
        return Err(unbuilt_new(interp, class));
    }
    let bytes = interp.to_text(text).to_vec();
    let object = interp.text_built(bytes);
    interp.roots.push_temp(object);
    let caller = interp.caller();
    interp.send_message(object, INIT, None, &args[1..], caller)?;
    Ok(Some(object))
}

/// `Stem~new(name, ...)`: a stem object whose name is the optional argument
/// -- `StemClass::newRexx` (`classes/StemClass.cpp:92`).
pub(super) fn native_stem_new(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    let name = match args.first().copied().flatten() {
        Some(value) => {
            let text = required_string_argument(interp, value, 1)?;
            interp.to_text(text).into_owned()
        }
        None => Vec::new(),
    };
    if class != interp.object_model().stem {
        return Err(unbuilt_new(interp, class));
    }
    let object = interp.alloc_with(
        rexx_core::BehaviourId::STEM,
        Body::Stem {
            name: name.into(),
            default: None,
            tails: rexx_core::NameMap::default(),
        },
    );
    interp.roots.push_temp(object);
    let caller = interp.caller();
    let rest = args.get(1..).unwrap_or_default();
    interp.send_message(object, INIT, None, rest, caller)?;
    Ok(Some(object))
}

/// `Pointer~new` and `Buffer~new`: the raise that is the whole body of
/// `PointerClass::newRexx` (`classes/PointerClass.cpp:139`-`:143`) and
/// `BufferClass::newRexx` (`classes/BufferClass.cpp:86`-`:90`).
pub(super) fn native_unsupported_new(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    let id = interp.class_id_text(class).as_bytes().to_vec();
    Err(Raised::unsupported_new_method(&id).into())
}

/// The address a `.Pointer` holds, or `None` for anything else.
pub(crate) fn pointer_address(interp: &Interp, value: ObjRef) -> Option<*mut std::ffi::c_void> {
    match &interp.heap.get(value)?.body {
        Body::Instance {
            native: Some(state),
            ..
        } => state.pointer(),
        _ => None,
    }
}

/// `PointerClass::equal` and `PointerClass::notEqual`
/// (`classes/PointerClass.cpp:71`, `:91`): anything that is not a `.Pointer`
/// compares unequal, and two `.Pointer`s compare on their addresses.
fn pointers_are_equal(
    interp: &Interp,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<bool, Failure> {
    let other = args
        .first()
        .copied()
        .flatten()
        .ok_or_else(|| Failure::from(Raised::missing_method_argument(1)))?;
    let Some(other) = pointer_address(interp, other) else {
        return Ok(false);
    };
    Ok(pointer_address(interp, receiver) == Some(other))
}

fn native_pointer_equal(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let equal = pointers_are_equal(interp, receiver, args)?;
    Ok(Some(interp.counted(usize::from(equal))))
}

fn native_pointer_not_equal(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let equal = pointers_are_equal(interp, receiver, args)?;
    Ok(Some(interp.counted(usize::from(!equal))))
}

/// `PointerClass::isNull` (`classes/PointerClass.cpp:163`).
fn native_pointer_is_null(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let null = pointer_address(interp, receiver).is_none_or(|address| address.is_null());
    Ok(Some(interp.counted(usize::from(null))))
}

/// The entry `WeakReference`'s scope pool binds the referent cell to, in the
/// position [`super::object_protocol::COLLECTION_STORES`]' entries are in.
pub(super) const WEAK_REFERENT: &[u8] = b"REFERENT";

/// The cell `WEAK_REFERENT` holds: a `Body::WeakRef` allocated for `referent`.
fn weak_referent_cell(interp: &mut Interp, referent: ObjRef) -> ObjRef {
    let cell = interp.alloc_with(rexx_core::BehaviourId::OBJECT, Body::WeakRef(referent));
    interp.roots.push_temp(cell);
    cell
}

/// `WeakReference~new(value, ...)`: a reference that does not keep `value`
/// alive -- `WeakReference::newRexx` (`classes/WeakReferenceClass.cpp:231`).
pub(super) fn native_weak_reference_new(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let class = class_receiver(interp, receiver)?;
    let Some(referent) = args.first().copied().flatten() else {
        return Err(Raised::missing_method_argument(1).into());
    };
    let scope = interp.object_model().weak_reference;
    let object = new_instance(interp, class)?;
    let cell = weak_referent_cell(interp, referent);
    interp.set_pool_variable(object, scope, WEAK_REFERENT, cell);
    let caller = interp.caller();
    let rest: Vec<Option<ObjRef>> = args.iter().skip(1).copied().collect();
    interp.send_message(object, INIT, None, &rest, caller)?;
    Ok(Some(object))
}

/// `WeakReference~value`: the referent, or `.nil` once the collector has
/// cleared it -- `WeakReference::value` (`classes/WeakReferenceClass.cpp:217`),
/// whose whole body is `resultOrNil(referentObject)`.
fn native_weak_reference_value(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let scope = interp.object_model().weak_reference;
    let cell = interp
        .pools_of(receiver)
        .and_then(|pools| pools.get(scope, WEAK_REFERENT));
    let referent = match cell.and_then(|cell| interp.heap.get(cell).map(|object| &object.body)) {
        Some(Body::WeakRef(referent)) => *referent,
        _ => ObjRef::NIL,
    };
    Ok(Some(referent))
}
