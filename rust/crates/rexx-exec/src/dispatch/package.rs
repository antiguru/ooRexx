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

//! `Package`'s readers and its four writes -- `classes/PackageClass.cpp`,
//! bound by `memory/Setup.cpp`'s `Package` block.

use std::collections::HashMap;

use rexx_core::{Body, ObjRef};

use super::{Arity, Cleared, Failure, Interp, Loud, NativeMethod, required_string_named_argument};
use crate::environment::PackageTable;
use crate::error::Raised;
use crate::options::{OptionQuery, PackageOptions};
use crate::plan::Package;
use crate::trace::TraceMode;
use crate::{InstalledRoutine, ProgramId};

/// `Package`'s instance methods. Chained into `ObjectModel::build` beside
/// [`super::NATIVE_METHODS`].
pub(super) static NATIVE_METHODS: &[(&str, &str, Arity, NativeMethod)] = &[
    ("Package", "ADDPACKAGE", Arity::Fixed(2), add_package),
    (
        "Package",
        "ADDPUBLICROUTINE",
        Arity::Fixed(2),
        add_public_routine,
    ),
    ("Package", "ADDROUTINE", Arity::Fixed(2), add_routine),
    ("Package", "CLASSES", Arity::Fixed(0), classes),
    (
        "Package",
        "DEFINEDMETHODS",
        Arity::Fixed(0),
        defined_methods,
    ),
    ("Package", "DIGITS", Arity::Fixed(0), digits),
    ("Package", "FINDCLASS", Arity::Fixed(1), find_class),
    ("Package", "FINDNAMESPACE", Arity::Fixed(1), find_namespace),
    ("Package", "FINDPROGRAM", Arity::Fixed(1), find_program),
    (
        "Package",
        "FINDPUBLICCLASS",
        Arity::Fixed(1),
        find_public_class,
    ),
    (
        "Package",
        "FINDPUBLICROUTINE",
        Arity::Fixed(1),
        find_public_routine,
    ),
    ("Package", "FINDROUTINE", Arity::Fixed(1), find_routine),
    ("Package", "FORM", Arity::Fixed(0), form),
    ("Package", "FUZZ", Arity::Fixed(0), fuzz),
    (
        "Package",
        "IMPORTEDCLASSES",
        Arity::Fixed(0),
        imported_classes,
    ),
    (
        "Package",
        "IMPORTEDPACKAGES",
        Arity::Fixed(0),
        imported_packages,
    ),
    (
        "Package",
        "IMPORTEDROUTINES",
        Arity::Fixed(0),
        imported_routines,
    ),
    ("Package", "LOADLIBRARY", Arity::Fixed(1), load_library),
    ("Package", "LOADPACKAGE", Arity::Fixed(2), load_package),
    ("Package", "NAMESPACES", Arity::Fixed(0), namespaces),
    ("Package", "OPTIONS", Arity::Fixed(2), options),
    ("Package", "PROLOG", Arity::Fixed(0), prolog),
    ("Package", "PUBLICCLASSES", Arity::Fixed(0), public_classes),
    (
        "Package",
        "PUBLICROUTINES",
        Arity::Fixed(0),
        public_routines,
    ),
    ("Package", "RESOURCE", Arity::Fixed(1), resource),
    ("Package", "RESOURCES", Arity::Fixed(0), resources),
    ("Package", "ROUTINES", Arity::Fixed(0), routines),
    (
        "Package",
        "SETSECURITYMANAGER",
        Arity::Fixed(1),
        set_security_manager,
    ),
    ("Package", "SOURCE", Arity::Fixed(0), source),
    ("Package", "SOURCELINE", Arity::Fixed(1), source_line),
    ("Package", "SOURCESIZE", Arity::Fixed(0), source_size),
    ("Package", "TRACE", Arity::Fixed(0), trace),
];

/// `Package`'s class methods.
pub(super) static NATIVE_CLASS_METHODS: &[(&str, &str, Arity, NativeMethod)] = &[
    (
        "Package",
        "DEFAULTOPTIONS",
        Arity::Fixed(2),
        default_options,
    ),
    // `AddClassMethod("New", PackageClass::newRexx, A_COUNT)`.
    ("Package", "NEW", Arity::Counted, package_new),
];

/// Which package `receiver` stands for: `Some(program)` for a file's package,
/// `None` for the interpreter's own.
fn package_of(interp: &Interp, receiver: ObjRef) -> Result<Option<ProgramId>, Failure> {
    match interp.which_package(receiver) {
        Some(Package::Program(program)) => Ok(Some(program)),
        Some(Package::Rexx) => Ok(None),
        None => Err(Loud::receiver_class("a package object this crate did not build").into()),
    }
}

/// `package_of` for the writes, which refuse the REXX package rather than
/// answering for it -- `checkRexxPackage` (`classes/PackageClass.cpp:470`).
fn writable_package_of(interp: &Interp, receiver: ObjRef) -> Result<ProgramId, Failure> {
    match package_of(interp, receiver)? {
        Some(program) => Ok(program),
        None => Err(Raised::rexx_package_addition().into()),
    }
}

/// The `::OPTIONS` settings a package carries, and the trace setting its
/// `~trace` reports.
fn settings_of(interp: &Interp, program: Option<ProgramId>) -> (PackageOptions, Option<TraceMode>) {
    match program {
        None => (PackageOptions::default(), None),
        Some(program) => {
            let options = interp.package_options.get(&program).cloned();
            let trace = options.as_ref().and_then(|options| options.trace);
            (
                options.unwrap_or_default(),
                Some(trace.unwrap_or(TraceMode::NORMAL)),
            )
        }
    }
}

/// `PackageClass::digitsRexx`: the package's own `DIGITS`.
fn digits(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let (options, _) = settings_of(interp, package_of(interp, receiver)?);
    let text = options.numeric.digits().to_string();
    Ok(Some(interp.text(text.as_bytes())))
}

/// `PackageClass::fuzzRexx`.
fn fuzz(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let (options, _) = settings_of(interp, package_of(interp, receiver)?);
    let text = options.numeric.fuzz().to_string();
    Ok(Some(interp.text(text.as_bytes())))
}

/// `PackageClass::formRexx`, whose two answers are `GlobalNames::SCIENTIFIC`
/// and `GlobalNames::ENGINEERING`.
fn form(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let (options, _) = settings_of(interp, package_of(interp, receiver)?);
    let text = match options.numeric.form() {
        rexx_num::Form::Scientific => b"SCIENTIFIC".as_slice(),
        rexx_num::Form::Engineering => b"ENGINEERING".as_slice(),
    };
    Ok(Some(interp.text(text)))
}

/// `PackageClass::traceRexx`: `TraceSetting::toString`, which is the one
/// letter the setting is known by and the null string for a setting that was
/// never made.
fn trace(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let (_, trace) = settings_of(interp, package_of(interp, receiver)?);
    let text = trace.map(|mode| vec![mode.letter]).unwrap_or_default();
    Ok(Some(interp.text_built(text)))
}

/// `PackageClass::options(optionName, newValue)`: the whole `::OPTIONS`
/// string, or one option's value, or a write.
fn options(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let (options, trace) = settings_of(interp, package_of(interp, receiver)?);
    let value = args.get(1).copied().flatten();
    let Some(name) = args.first().copied().flatten() else {
        if value.is_some() {
            return Err(Raised::missing_method_argument(1).into());
        }
        return Ok(Some(interp.text_built(options.to_options_string(trace))));
    };
    let name = required_string_named_argument(interp, name, "optionName")?;
    let name = interp.to_text(name).to_vec();
    let query = options.option_query(&name, trace);
    let Some(value) = value else {
        return match query {
            OptionQuery::Value(value) | OptionQuery::ReadOnly1(value) => {
                Ok(Some(interp.text_built(value)))
            }
            OptionQuery::MissingSecond => Err(Raised::not_enough_method_arguments(2).into()),
            OptionQuery::NeedsValue => Err(Raised::missing_method_argument(2).into()),
            OptionQuery::Unknown => Err(unknown_option(&name, false)),
        };
    };
    let value = required_string_named_argument(interp, value, "newValue")?;
    let value = interp.to_text(value).to_vec();
    if matches!(query, OptionQuery::Unknown) {
        return Err(unknown_option(&name, true));
    }
    // The empty check stands ahead of the per-option switch, so it fires even
    // for an option that cannot be set at all: measured at rc 163,
    // `p~options('X', '')` is 93.900 and `p~options('X', 'Y')` is 93.902.
    if value.is_empty() {
        return Err(Raised::method_user_defined("argument 2 must not be empty").into());
    }
    // `I`, `R` and `X` are invariants: the setting switch answers 93.902 for
    // each rather than writing anything.
    if matches!(query, OptionQuery::ReadOnly1(_)) {
        return Err(Raised::too_many_method_arguments(1).into());
    }
    Err(Loud::package_option_write().into())
}

/// `Error_Incorrect_method_list` for an option name `~options` does not
/// accept, in the two spellings the C++ has: the list a no-value send names
/// and the list a setting send names are not the same set.
fn unknown_option(name: &[u8], setting: bool) -> Failure {
    let list = if setting {
        "\"A[ll], D[igits], E[rror], FA[ilure], FO[rm], FU[zz], I[nitialOptions], \
         L[ostdigits], NU[meric], NOS[tring], NOT[ready], NOV[alue], P[rolog], \
         R[esetOptions], S[etOptions] or T[race]\""
    } else {
        "\"D[igits], E[rror], FA[ilure], FO[rm], FU[zz], I[nitialOptions], L[ostdigits], \
         NU[meric], NOS[tring], NOT[ready], NOV[alue], P[rolog], R[esetOptions], \
         S[etOptions], T[race] or X[explicitlyDefined]\""
    };
    Raised::method_argument_not_in_list(1, list, name).into()
}

/// `PackageClass::clzOptions`: the class-level override settings, which are
/// what a package that names an option itself does *not* get.
fn default_options(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let Some(name) = args.first().copied().flatten() else {
        return Err(Raised::missing_named_argument("optionName").into());
    };
    let name = required_string_named_argument(interp, name, "optionName")?;
    let name = interp.to_text(name).to_vec();
    let value = args.get(1).copied().flatten();
    let which = name.first().map(u8::to_ascii_uppercase);
    if !matches!(which, Some(b'D' | b'C')) {
        return Err(Raised::method_argument_not_in_list(
            1,
            "\"D[efineDefaultOptions] or C[ountOverrides]\"",
            &name,
        )
        .into());
    }
    if let Some(value) = value {
        let value = required_string_named_argument(interp, value, "newValue")?;
        let text = interp.to_text(value).to_vec();
        if text.is_empty() {
            return Err(Raised::method_user_defined("argument 2 must not be empty").into());
        }
        // `C[ountOverrides]` converts its value with `numberArgument` before
        // it stores it, and the conversion failure is the oracle's own:
        // measured at rc 163, `.Package~defaultOptions('C', 'x')` is 93.905.
        if which == Some(b'C') && !super::is_whole_method_argument(interp, value) {
            return Err(Raised::method_argument_not_whole(2, &text).into());
        }
        return Err(Loud::package_option_write().into());
    }
    if which == Some(b'C') {
        return Ok(Some(interp.counted(0)));
    }
    let text = PackageOptions::default().to_options_string(Some(TraceMode::NORMAL));
    Ok(Some(interp.text_built(text)))
}

/// `PackageClass::getSourceSizeRexx`: the package's own line count, and zero
/// for the REXX package, which holds no source.
fn source_size(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let lines = source_lines(interp, package_of(interp, receiver)?).len();
    Ok(Some(interp.counted(lines)))
}

/// `PackageClass::getSourceRexx`: the package's source as an `Array` of
/// strings, in program order.
fn source(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let lines = source_lines(interp, package_of(interp, receiver)?);
    let mut items = Vec::with_capacity(lines.len());
    let frame = interp.roots.push_frame();
    for line in &lines {
        let text = interp.text(line);
        interp.roots.push_temp(text);
        items.push(text);
    }
    let array = interp.object_array(items);
    interp.roots.pop_frame(frame);
    interp.roots.push_temp(array);
    Ok(Some(array))
}

/// `PackageClass::getSourceLineRexx`: one line, by position.
fn source_line(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let program = package_of(interp, receiver)?;
    let position = super::required_position_argument(interp, args, 0)?;
    let lines = source_lines(interp, program);
    let line = lines.get(position - 1).cloned().unwrap_or_default();
    Ok(Some(interp.text_built(line)))
}

/// The lines `~source`, `~sourceLine` and `~sourceSize` read.
fn source_lines(interp: &Interp, program: Option<ProgramId>) -> Vec<Vec<u8>> {
    let Some(program) = program.and_then(|program| interp.programs.get(program.0)) else {
        return Vec::new();
    };
    (1..=program.source.line_count())
        .filter_map(|number| program.source.line(number).map(<[u8]>::to_vec))
        .collect()
}

/// `PackageClass::getMainRexx`: the `Routine` a package's leading code
/// section became, and `.nil` for a package that has none.
fn prolog(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let Some(program) = package_of(interp, receiver)? else {
        return Ok(Some(ObjRef::NIL));
    };
    Ok(Some(interp.program_routine_object(program)))
}

/// `PackageClass::setSecurityManagerRexx`.
fn set_security_manager(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    writable_package_of(interp, receiver)?;
    if args.first().copied().flatten().is_some() {
        return Err(Loud::security_manager().into());
    }
    Ok(Some(interp.counted(1)))
}

/// `PackageClass::loadLibraryRexx`: `PackageManager::loadLibrary`, a
/// `dlopen` of a native library.
fn load_library(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    // The name is validated before the receiver is: `loadLibraryRexx` calls
    // `stringArgument` and then `checkRexxPackage`. Measured at rc 168,
    // `.Class~package~loadLibrary` is 88.901 and not 98.984.
    let Some(name) = args.first().copied().flatten() else {
        return Err(Raised::missing_named_argument("name").into());
    };
    required_string_named_argument(interp, name, "name")?;
    writable_package_of(interp, receiver)?;
    Err(Loud::external_entry_point("a native library load").into())
}

/// `PackageClass::getClassesRexx`: the classes this package's own `::CLASS`
/// directives and `~addClass` installed.
fn classes(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let entries = match package_of(interp, receiver)? {
        Some(program) => cloned(interp.package_classes.get(&program)),
        None => interp.rexx_package_class_table(false),
    };
    Ok(Some(interp.string_table_of(entries)))
}

/// `PackageClass::getPublicClassesRexx`.
fn public_classes(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let entries = match package_of(interp, receiver)? {
        Some(program) => cloned(interp.package_public_classes.get(&program)),
        None => interp.rexx_package_class_table(true),
    };
    Ok(Some(interp.string_table_of(entries)))
}

/// `PackageClass::getImportedClassesRexx`: `mergedPublicClasses`, which is
/// what this package's `::REQUIRES` and `~addPackage` brought in and **not**
/// its own public classes. Measured, oracle rc 0: a file declaring
/// `::class K public` and requiring one declaring `::class LIBC public`
/// answers `LIBC` alone.
fn imported_classes(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let entries = match package_of(interp, receiver)? {
        Some(program) => cloned(interp.merged_public_classes.get(&program)),
        None => Vec::new(),
    };
    Ok(Some(interp.string_table_of(entries)))
}

/// `PackageClass::getRoutinesRexx`: every routine this package's own
/// `::ROUTINE` directives and `~addRoutine` installed, public or not.
fn routines(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let program = package_of(interp, receiver)?;
    let entries = routine_entries(interp, program, |interp| &interp.routines);
    Ok(Some(interp.string_table_of(entries)))
}

/// `PackageClass::getPublicRoutinesRexx`.
fn public_routines(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let program = package_of(interp, receiver)?;
    let entries = routine_entries(interp, program, |interp| &interp.package_public_routines);
    Ok(Some(interp.string_table_of(entries)))
}

/// `PackageClass::getImportedRoutinesRexx`: `mergedPublicRoutines`.
fn imported_routines(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let program = package_of(interp, receiver)?;
    let entries = routine_entries(interp, program, |interp| &interp.merged_public_routines);
    Ok(Some(interp.string_table_of(entries)))
}

/// The (name, `Routine` object) pairs one of the three routine tables holds.
fn routine_entries(
    interp: &mut Interp,
    program: Option<ProgramId>,
    which: fn(&Interp) -> &HashMap<ProgramId, HashMap<Box<[u8]>, InstalledRoutine>>,
) -> Vec<(Box<[u8]>, ObjRef)> {
    let Some(program) = program else {
        return Vec::new();
    };
    let named: Vec<(Box<[u8]>, InstalledRoutine)> = which(interp)
        .get(&program)
        .into_iter()
        .flatten()
        .map(|(name, installed)| (name.clone(), *installed))
        .collect();
    // **Each object is rooted as it is built**, because building the next one
    // allocates: `Interp::routine_object` materialises the declaring
    // program's `.ROUTINES` table on a miss, and a `Routine` produced by an
    // earlier turn of this loop is reachable from nothing but this `Vec`,
    // which the collector does not walk. Measured under
    // `run_program_collect_every_alloc`: without the temp,
    // `corpus/lang/package_tables.rex` panics at `Interp::not_in_arena`.
    let mut entries = Vec::with_capacity(named.len());
    for (name, installed) in named {
        let Some(object) = interp.routine_object(installed) else {
            continue;
        };
        interp.roots.push_temp(object);
        entries.push((name, object));
    }
    entries
}

/// `PackageClass::getMethodsRexx`: the package's unattached `::METHOD`s,
/// which is the same table `.METHODS` answers.
fn defined_methods(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let entries = package_table_entries(interp, receiver, PackageTable::UnattachedMethods)?;
    Ok(Some(interp.string_table_of(entries)))
}

/// `PackageClass::getResourcesRexx`: the package's `::RESOURCE` bodies, the
/// same table `.RESOURCES` answers.
fn resources(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let entries = package_table_entries(interp, receiver, PackageTable::Resources)?;
    Ok(Some(interp.string_table_of(entries)))
}

/// `PackageClass::getResourceRexx`: one `::RESOURCE`'s `Array`, or `.nil`.
fn resource(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let entries = package_table_entries(interp, receiver, PackageTable::Resources)?;
    let Some(name) = args.first().copied().flatten() else {
        return Err(Raised::missing_named_argument("name").into());
    };
    let name = required_string_named_argument(interp, name, "name")?;
    let name = interp.to_text(name).to_ascii_uppercase();
    Ok(Some(
        entries
            .into_iter()
            .find(|(held, _)| **held == *name)
            .map_or(ObjRef::NIL, |(_, value)| value),
    ))
}

/// One of the package tables `.METHODS`/`.ROUTINES`/`.RESOURCES` also
/// answers, as (name, value) pairs.
fn package_table_entries(
    interp: &mut Interp,
    receiver: ObjRef,
    kind: PackageTable,
) -> Result<Vec<(Box<[u8]>, ObjRef)>, Failure> {
    let Some(program) = package_of(interp, receiver)? else {
        return Ok(Vec::new());
    };
    let Some(table) = interp.package_string_table(program, kind) else {
        return Ok(Vec::new());
    };
    Ok(table_pairs(interp, table))
}

/// Every (key, value) a `Body::Native` table holds.
fn table_pairs(interp: &Interp, table: ObjRef) -> Vec<(Box<[u8]>, ObjRef)> {
    let Some(object) = interp.heap.get(table) else {
        return Vec::new();
    };
    let Body::Native(native) = &object.body else {
        return Vec::new();
    };
    native
        .keys()
        .into_iter()
        .filter_map(|key| native.entry(&key).map(|value| (key, value)))
        .collect()
}

/// `PackageClass::getNamespacesRexx`: the qualifiers this package's
/// `::REQUIRES ... NAMESPACE` directives and `~addPackage` registered, each
/// answering the package it names.
fn namespaces(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let named: Vec<(Box<[u8]>, Package)> = match package_of(interp, receiver)? {
        Some(program) => cloned(interp.package_namespaces.get(&program)),
        None => Vec::new(),
    };
    let entries = named
        .into_iter()
        .map(|(name, package)| {
            let object = interp.package_object(package);
            (name, object)
        })
        .collect();
    Ok(Some(interp.string_table_of(entries)))
}

/// `PackageClass::getImportedPackagesRexx`: the packages this one has
/// imported, in the order they were added, as an `Array`.
fn imported_packages(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let imported: Vec<Package> = match package_of(interp, receiver)? {
        Some(program) => interp
            .package_imports
            .get(&program)
            .cloned()
            .unwrap_or_default(),
        None => Vec::new(),
    };
    let items = imported
        .into_iter()
        .map(|package| interp.package_object(package))
        .collect();
    Ok(Some(interp.object_array(items)))
}

/// `PackageClass::findClassRexx`: the whole environment-symbol search order,
/// starting at this package.
fn find_class(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let program = package_of(interp, receiver)?;
    let name = upper_name_argument(interp, args)?;
    Ok(Some(interp.package_find_class(program, &name)))
}

/// `PackageClass::findPublicClassRexx`: this package's own public classes,
/// then its imports, then the REXX package's.
fn find_public_class(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let program = package_of(interp, receiver)?;
    let name = upper_name_argument(interp, args)?;
    Ok(Some(interp.package_find_public_class(program, &name)))
}

/// `PackageClass::findRoutineRexx`: the package's own routines, then the
/// public ones its `::REQUIRES` brought in.
fn find_routine(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let program = package_of(interp, receiver)?;
    let name = upper_name_argument(interp, args)?;
    Ok(Some(interp.package_find_routine(program, &name)))
}

/// `PackageClass::findPublicRoutineRexx`.
fn find_public_routine(
    interp: &mut Interp,
    cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    find_routine(interp, cleared, receiver, args)
}

/// `PackageClass::findNamespaceRexx`: one of this package's registered
/// qualifiers, or the REXX package for the name `REXX`.
fn find_namespace(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let program = package_of(interp, receiver)?;
    let name = upper_name_argument(interp, args)?;
    if name == b"REXX" {
        return Ok(Some(interp.package_object(Package::Rexx)));
    }
    let found = program
        .and_then(|program| interp.package_namespaces.get(&program))
        .and_then(|held| held.get(name.as_slice()))
        .copied();
    Ok(Some(match found {
        Some(package) => interp.package_object(package),
        None => ObjRef::NIL,
    }))
}

/// `PackageClass::findProgramRexx`: the file `name` resolves to from this
/// package's own directory, as a **String**, or `.nil`.
fn find_program(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let program = package_of(interp, receiver)?;
    let Some(name) = args.first().copied().flatten() else {
        return Err(Raised::missing_named_argument("name").into());
    };
    let name = required_string_named_argument(interp, name, "name")?;
    let name = interp.to_text(name).to_vec();
    Ok(Some(
        match interp.resolve_program_name(program, &name, false) {
            Some(path) => interp.text(path.as_bytes()),
            None => ObjRef::NIL,
        },
    ))
}

/// The `name` argument every `find*` reader takes, upcased the way each of
/// their tables is keyed.
fn upper_name_argument(interp: &mut Interp, args: &[Option<ObjRef>]) -> Result<Vec<u8>, Failure> {
    let Some(name) = args.first().copied().flatten() else {
        return Err(Raised::missing_named_argument("name").into());
    };
    let name = required_string_named_argument(interp, name, "name")?;
    Ok(interp.to_text(name).to_ascii_uppercase())
}

/// `PackageClass::addRoutineRexx`: files a `Routine` under `name` in this
/// package's own routine table.
fn add_routine(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    install_routine(interp, receiver, args, false)
}

/// `PackageClass::addPublicRoutineRexx`, which files into **both** tables:
/// `addInstalledRoutine(name, routine, true)` writes `routines` and then
/// `publicRoutines` (`classes/PackageClass.cpp:1432`). Measured, oracle rc 0:
/// after `p~addPublicRoutine('PUBR', r)` the name is in `~routines` as well
/// as in `~publicRoutines`.
fn add_public_routine(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    install_routine(interp, receiver, args, true)
}

/// The body both `~addRoutine` rows share, since a second implementation is
/// where the two could come to disagree.
fn install_routine(
    interp: &mut Interp,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
    public: bool,
) -> Result<Option<ObjRef>, Failure> {
    let Some(name) = args.first().copied().flatten() else {
        return Err(Raised::missing_named_argument("name").into());
    };
    let name = required_string_named_argument(interp, name, "name")?;
    let name: Box<[u8]> = interp.to_text(name).to_ascii_uppercase().into();
    let Some(routine) = args.get(1).copied().flatten() else {
        return Err(Raised::missing_named_argument("routine").into());
    };
    let Some(installed) = interp
        .executable_sources
        .get(&routine)
        .and_then(|record| record.routine)
        .filter(|_| interp.class_of_value(routine) == Some(interp.routine_class()))
    else {
        return Err(Raised::argument_not_an_instance("routine", "Routine").into());
    };
    let program = writable_package_of(interp, receiver)?;
    let installed = InstalledRoutine {
        program: installed.0,
        directive: installed.1,
    };
    interp.routine_objects.insert(installed, routine);
    interp
        .routines
        .entry(program)
        .or_default()
        .insert(name.clone(), installed);
    if public {
        interp
            .package_public_routines
            .entry(program)
            .or_default()
            .insert(name.clone(), installed);
    }
    // `.ROUTINES` answers this same table, so the write has to reach it --
    // measured, oracle rc 0, `.routines~allIndexes` carries a name added by
    // `~addRoutine`.
    if let Some(table) = interp.package_string_table_for_write(program, PackageTable::Routines)
        && let Some(object) = interp.heap.get_mut(table)
        && let Body::Native(native) = &mut object.body
    {
        native.set_entry(&name, routine);
    }
    Ok(Some(receiver))
}

/// `PackageClass::addPackageRexx`: imports another package into this one,
/// optionally under a namespace qualifier.
fn add_package(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let Some(added) = args.first().copied().flatten() else {
        return Err(Raised::missing_named_argument("package").into());
    };
    let Some(added) = interp.which_package(added) else {
        return Err(Raised::argument_not_an_instance("package", "Package").into());
    };
    let namespace = match args.get(1).copied().flatten() {
        None => None,
        Some(namespace) => {
            let namespace = required_string_named_argument(interp, namespace, "namespace")?;
            Some(interp.to_text(namespace).to_ascii_uppercase())
        }
    };
    let program = writable_package_of(interp, receiver)?;
    if interp.add_imported_package(program, added) {
        interp.merge_package(program, added);
    }
    if let Some(namespace) = namespace {
        interp
            .package_namespaces
            .entry(program)
            .or_default()
            .insert(namespace.into(), added);
    }
    Ok(Some(receiver))
}

/// `PackageClass::loadPackageRexx`: loads a file the way a `::REQUIRES`
/// would, imports it, and answers the loaded package.
fn load_package(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    // `stringArgument(name, 1)`, which numbers the argument where every other
    // reader here names it: measured at rc 163, `p~loadPackage` is 93.903
    // `argument 1 is required` and `p~loadPackage(.nil)` is 88.909
    // `Argument 1 must have a string value.`
    let Some(name) = args.first().copied().flatten() else {
        return Err(Raised::missing_method_argument(1).into());
    };
    let name = super::required_string_argument(interp, name, 1)?;
    let name = interp.to_text(name).to_vec();
    // `checkRexxPackage` stands between the name and the source array, so the
    // REXX package refuses even the form this crate declines: measured at
    // rc 158, `.Class~package~loadPackage('X', 'Y')` is 98.984.
    let program = writable_package_of(interp, receiver)?;
    if args.get(1).copied().flatten().is_some() {
        return Err(Loud::package_from_source().into());
    }
    let loaded = interp.load_package(program, &name)?;
    if interp.add_imported_package(program, Package::Program(loaded)) {
        interp.merge_required(program, loaded);
    }
    Ok(Some(interp.package_object(Package::Program(loaded))))
}

/// `PackageClass::newRexx`: a package built from a file, or from an `Array`
/// of source lines.
fn package_new(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let Some(name) = args.first().copied().flatten() else {
        return Err(Raised::missing_named_argument("name").into());
    };
    let name = required_string_named_argument(interp, name, "name")?;
    let name = interp.to_text(name).to_vec();
    if let Some(context) = args.get(2).copied().flatten() {
        // A third argument that is none of the three the oracle accepts is
        // its own 93.953, before anything is compiled: measured at rc 163,
        // `.Package~new('X', 'Y', 'Z')`.
        let class = interp.class_of_value(context);
        if class != Some(interp.package_class())
            && class != Some(interp.routine_class())
            && class != Some(interp.method_class())
        {
            return Err(
                Raised::argument_not_convertible(3, "Method, Routine, or Package object").into(),
            );
        }
        return Err(Loud::executable_context().into());
    }
    if args.len() > 3 {
        return Err(Loud::executable_context().into());
    }
    let program = match args.get(1).copied().flatten() {
        None => {
            let from = interp.running_program().unwrap_or(ProgramId(0));
            interp.load_package(from, &name)?
        }
        Some(source) => {
            let lines = super::method_source_lines(interp, source, "source")?;
            interp.package_from_source(&name, &lines)?
        }
    };
    Ok(Some(interp.package_object(Package::Program(program))))
}

/// Every (key, value) of one of `Interp`'s per-program tables, owned.
fn cloned<K: Clone, V: Copy, S>(held: Option<&std::collections::HashMap<K, V, S>>) -> Vec<(K, V)> {
    held.into_iter()
        .flatten()
        .map(|(key, value)| (key.clone(), *value))
        .collect()
}
