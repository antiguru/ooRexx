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

//! `RexxInfo`'s readers -- `classes/RexxInfoClass.cpp`, bound by
//! `memory/Setup.cpp:1252`-`:1279`.

use super::{Arity, Cleared, Failure, Interp, Loud, NativeMethod, ObjRef, Raised};
use crate::plan::Package;
use crate::version;

/// `RexxInfo`'s instance methods. Chained into `ObjectModel::build` beside
/// [`super::NATIVE_METHODS`].
pub(super) static NATIVE_METHODS: &[(&str, &str, Arity, NativeMethod)] = &[
    ("RexxInfo", "ARCHITECTURE", Arity::Fixed(0), architecture),
    (
        "RexxInfo",
        "CASESENSITIVEFILES",
        Arity::Fixed(0),
        case_sensitive_files,
    ),
    ("RexxInfo", "COPY", Arity::Fixed(0), copy),
    ("RexxInfo", "DATE", Arity::Fixed(0), date),
    ("RexxInfo", "DEBUG", Arity::Fixed(0), debug),
    ("RexxInfo", "DIGITS", Arity::Fixed(0), digits),
    (
        "RexxInfo",
        "DIRECTORYSEPARATOR",
        Arity::Fixed(0),
        directory_separator,
    ),
    ("RexxInfo", "ENDOFLINE", Arity::Fixed(0), end_of_line),
    ("RexxInfo", "EXECUTABLE", Arity::Fixed(0), executable),
    ("RexxInfo", "FORM", Arity::Fixed(0), form),
    ("RexxInfo", "FUZZ", Arity::Fixed(0), fuzz),
    (
        "RexxInfo",
        "INTERNALDIGITS",
        Arity::Fixed(0),
        internal_digits,
    ),
    (
        "RexxInfo",
        "INTERNALMAXNUMBER",
        Arity::Fixed(0),
        internal_max_number,
    ),
    (
        "RexxInfo",
        "INTERNALMINNUMBER",
        Arity::Fixed(0),
        internal_min_number,
    ),
    ("RexxInfo", "LANGUAGELEVEL", Arity::Fixed(0), language_level),
    ("RexxInfo", "LIBRARYPATH", Arity::Fixed(0), library_path),
    ("RexxInfo", "MAJORVERSION", Arity::Fixed(0), major_version),
    ("RexxInfo", "MAXARRAYSIZE", Arity::Fixed(0), max_array_size),
    ("RexxInfo", "MAXEXPONENT", Arity::Fixed(0), max_exponent),
    (
        "RexxInfo",
        "MAXPATHLENGTH",
        Arity::Fixed(0),
        max_path_length,
    ),
    ("RexxInfo", "MINEXPONENT", Arity::Fixed(0), min_exponent),
    ("RexxInfo", "MODIFICATION", Arity::Fixed(0), modification),
    ("RexxInfo", "NAME", Arity::Fixed(0), name),
    ("RexxInfo", "PACKAGE", Arity::Fixed(0), package),
    ("RexxInfo", "PATHSEPARATOR", Arity::Fixed(0), path_separator),
    ("RexxInfo", "PLATFORM", Arity::Fixed(0), platform),
    ("RexxInfo", "RELEASE", Arity::Fixed(0), release),
    ("RexxInfo", "REVISION", Arity::Fixed(0), revision),
    ("RexxInfo", "VERSION", Arity::Fixed(0), version),
];

/// `linux/limits.h`'s `PATH_MAX`, which `platform/unix/SysFileSystem.hpp:65`
/// takes `MAXIMUM_PATH_LENGTH` from.
const PATH_MAX: usize = 4096;

/// `ORX_BLD`: the source-control revision the interpreter was configured at.
const BUILD_LEVEL: i64 = 0;

/// `Numerics::MAX_WHOLENUMBER`: the largest value a whole-number conversion
/// can produce, and the ceiling on `NUMERIC DIGITS` itself.
const MAX_WHOLENUMBER: i64 = 10i64.pow(rexx_num::ARGUMENT_DIGITS as u32) - 1;

/// A whole number as the object a `new_integer` answers, falling back to the
/// digits when it is wider than a tagged small integer.
fn whole(interp: &mut Interp, value: i64) -> ObjRef {
    ObjRef::small_int(value).unwrap_or_else(|| interp.text_built(value.to_string().into_bytes()))
}

/// `RexxInfo::getArchitecture`: `sizeof(void *) * 8`.
fn architecture(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    Ok(Some(interp.counted(size_of::<*const ()>() * 8)))
}

/// `RexxInfo::getCaseSensitiveFiles`: `SysFileSystem::isCaseSensitive("/")`.
fn case_sensitive_files(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    Ok(Some(interp.counted(1)))
}

/// `RexxInfo::copyRexx`, whose whole body is the 93.970 raise.
fn copy(
    interp: &mut Interp,
    _cleared: Cleared,
    receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let target = interp.string_value_text(receiver);
    Err(Raised::copy_not_supported(&target).into())
}

/// `RexxInfo::getRexxExecutable`: a `File` over the running executable's
/// path with its links resolved, or `.nil` where that cannot be determined.
fn executable(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let path = executable_path().map(|path| path.into_os_string().into_encoded_bytes());
    file_object(interp, path)
}

/// `RexxInfo::getRexxLibrary`: a `File` over the directory the interpreter's
/// code was loaded from, with a trailing separator. This crate's runtime is
/// linked into the executable, so that is the executable's own directory.
fn library_path(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let directory = executable_path().and_then(|path| {
        let mut bytes = path.parent()?.as_os_str().as_encoded_bytes().to_vec();
        bytes.push(std::path::MAIN_SEPARATOR as u8);
        Some(bytes)
    });
    file_object(interp, directory)
}

/// `SysProcess::getExecutableFullPath`: the running executable, resolved.
fn executable_path() -> Option<std::path::PathBuf> {
    std::env::current_exe().and_then(std::fs::canonicalize).ok()
}

/// `.File~new(path)`, or `.nil` for no path.
fn file_object(interp: &mut Interp, path: Option<Vec<u8>>) -> Result<Option<ObjRef>, Failure> {
    let Some(path) = path else {
        return Ok(Some(ObjRef::NIL));
    };
    let Some(class) = interp.rexx_package_class(b"FILE") else {
        return Err(Loud::environment_symbol(b".FILE", None).into());
    };
    let argument = interp.text_built(path);
    interp.roots.activity_mut().push_temp(argument);
    let caller = interp.caller();
    interp
        .send_message(class, b"NEW", None, &[Some(argument)], caller)?
        .ok_or_else(|| Failure::from(Raised::no_result(b"NEW")))
        .map(Some)
}

/// `RexxInfo::getInterpreterDate`: the build date, which is
/// [`version::VERSION`]'s tail.
fn date(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    Ok(Some(interp.text(version::BUILD_DATE)))
}

/// `RexxInfo::getDebug`: `#ifdef _DEBUG`.
fn debug(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    Ok(Some(interp.counted(0)))
}

/// `RexxInfo::getDigits`: `Numerics::DEFAULT_DIGITS`, read here off the
/// settings a program starts with rather than off the activation's own.
fn digits(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let digits = rexx_num::Settings::default().digits();
    Ok(Some(interp.counted(digits as usize)))
}

/// `RexxInfo::getDirectorySeparator`: `SysFileSystem::getSeparator`
/// (`platform/unix/SysFileSystem.cpp:1358`).
fn directory_separator(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    Ok(Some(interp.text(b"/")))
}

/// `RexxInfo::getFileEndOfLine`: `SysFileSystem::getLineEnd`
/// (`platform/unix/SysFileSystem.cpp:1380`).
fn end_of_line(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    Ok(Some(interp.text(b"\n")))
}

/// `RexxInfo::getForm`: `GlobalNames::SCIENTIFIC`, which that function
/// returns unconditionally -- its own comment is "always scientific". Not the
/// default setting's form, which would agree on this interpreter and stop
/// mirroring the C++ on one where they parted.
fn form(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    Ok(Some(interp.text(b"SCIENTIFIC")))
}

/// `RexxInfo::getFuzz`: `Numerics::DEFAULT_FUZZ`, read off the settings a
/// program starts with.
fn fuzz(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    let fuzz = rexx_num::Settings::default().fuzz();
    Ok(Some(interp.counted(fuzz as usize)))
}

/// `RexxInfo::getInternalDigits`: [`rexx_num::ARGUMENT_DIGITS`], which mirrors
/// `Numerics::ARGUMENT_DIGITS` and is the precision an argument is converted
/// to a machine integer under.
fn internal_digits(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    Ok(Some(interp.counted(rexx_num::ARGUMENT_DIGITS)))
}

/// `RexxInfo::getInternalMaxNumber`: [`MAX_WHOLENUMBER`].
fn internal_max_number(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    Ok(Some(whole(interp, MAX_WHOLENUMBER)))
}

/// `RexxInfo::getInternalMinNumber`: `Numerics::MIN_WHOLENUMBER`, which
/// `Numerics.hpp:87` and `:95` are [`MAX_WHOLENUMBER`] negated.
fn internal_min_number(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    Ok(Some(whole(interp, -MAX_WHOLENUMBER)))
}

/// `RexxInfo::getLanguageLevel`: `Interpreter::getLanguageLevelString()`,
/// which is also the word `PARSE VERSION` carries.
fn language_level(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    Ok(Some(interp.text(version::LANGUAGE_LEVEL)))
}

/// `RexxInfo::getMajorVersion`: `ORX_VER`.
fn major_version(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    Ok(Some(interp.text(version::MAJOR_VERSION)))
}

/// `RexxInfo::getMaxArraySize`: `ArrayClass::MaxFixedArraySize`, the same
/// constant [`super::MAX_FIXED_ARRAY_SIZE`] bounds `.Array~new` by. Measured,
/// oracle rc 0 under a trap: `.Array~new(that + 1)` is
/// `Error 93.959: An array cannot contain more than 100000000000000000
/// elements.`
fn max_array_size(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    Ok(Some(interp.counted(super::MAX_FIXED_ARRAY_SIZE)))
}

/// `RexxInfo::getMaxExponent`: `Numerics::MAX_EXPONENT`, the bound this
/// crate's arithmetic reports error 42 past.
fn max_exponent(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    Ok(Some(whole(interp, i64::from(rexx_num::MAX_EXPONENT))))
}

/// `RexxInfo::getMaxPathLength`: `SysFileSystem::MaximumPathLength - 1`,
/// which `platform/unix/SysFileSystem.hpp:65` makes `PATH_MAX + 1`.
fn max_path_length(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    Ok(Some(interp.counted(PATH_MAX)))
}

/// `RexxInfo::getMinExponent`: `Numerics::MIN_EXPONENT`.
fn min_exponent(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    Ok(Some(whole(interp, i64::from(rexx_num::MIN_EXPONENT))))
}

/// `RexxInfo::getModification`: `ORX_MOD`.
fn modification(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    Ok(Some(interp.text(version::MODIFICATION)))
}

/// `RexxInfo::getInterpreterName`: `Interpreter::getVersionString()`, the
/// same string `PARSE VERSION` answers and the same constant.
fn name(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    Ok(Some(interp.text(version::VERSION)))
}

/// `RexxInfo::getPackage`: `TheRexxPackage`, the package the primitive
/// classes belong to and the one `.Array~package` already answers.
fn package(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    Ok(Some(interp.package_object(Package::Rexx)))
}

/// `RexxInfo::getPathSeparator`: `SysFileSystem::getPathSeparator`
/// (`platform/unix/SysFileSystem.cpp:1369`).
fn path_separator(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    Ok(Some(interp.text(b":")))
}

/// `RexxInfo::getPlatform`: `SystemInterpreter::getPlatformName()`, which is
/// `ORX_SYS_STR` -- the same constant `PARSE SOURCE`'s first word reads.
fn platform(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    Ok(Some(interp.text(version::PLATFORM)))
}

/// `RexxInfo::getRelease`: `ORX_REL`.
fn release(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    Ok(Some(interp.text(version::RELEASE)))
}

/// `RexxInfo::getRevision`: `ORX_BLD` -- see [`BUILD_LEVEL`].
fn revision(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    Ok(Some(whole(interp, BUILD_LEVEL)))
}

/// `RexxInfo::getInterpreterVersion`: `ORX_VER.ORX_REL.ORX_MOD`.
fn version(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    Ok(Some(interp.text(version::VERSION_NUMBER)))
}
