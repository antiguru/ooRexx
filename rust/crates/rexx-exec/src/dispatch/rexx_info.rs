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
//!
//! A child of [`super`] rather than a sibling, for the reason its `mod native`
//! comment gives: the rows point at `NativeMethod`s, whose parameter list
//! names types only that module can.
//!
//! # The receiver carries none of this
//!
//! `RexxInfo::initialize` fills every field of the C++ instance from a
//! compile-time constant or a platform call, at image-build time; nothing a
//! program does moves any of them, and there is one instance. So a body here
//! reads a constant of this crate's own, and the constraint that matters is
//! **which** constant: the answer must be the same fact the interpreter
//! enforces elsewhere, not a value copied out of a differential.
//!
//! # `digits`, `form` and `fuzz` are the defaults and not the settings in force
//!
//! `RexxInfo::getDigits` reads `Numerics::DEFAULT_DIGITS` and
//! `RexxInfo::getFuzz` reads `Numerics::DEFAULT_FUZZ`; neither looks at the
//! running activation. Measured, oracle rc 0: after `numeric digits 5;
//! numeric form engineering; numeric fuzz 2`, `.RexxInfo~digits
//! .RexxInfo~form .RexxInfo~fuzz` is `9 SCIENTIFIC 0` where `digits() form()
//! fuzz()` is `5 ENGINEERING 2`. `RexxContext`'s same-named methods are the
//! live ones.
//!
//! # The version fields have one source
//!
//! [`crate::parse_template::VERSION`] is `PARSE VERSION`'s string and is also
//! `RexxInfo~name` -- `RexxInfo::initialize` assigns
//! `Interpreter::getVersionString()`. `version`, `majorVersion`, `release`,
//! `modification`, `languageLevel` and `date` are cut out of it there.
//! `revision` is not: it is `ORX_BLD` and appears nowhere in the string.

use super::{Arity, Cleared, Failure, Interp, NativeMethod, ObjRef};
use crate::parse_template;
use crate::plan::Package;

/// `RexxInfo`'s instance methods. Chained into `ObjectModel::build` beside
/// [`super::NATIVE_METHODS`].
///
/// `Setup.cpp:1252`-`:1279` declares each of these with a parameter count of
/// zero, so the send refuses an argument before a body runs -- measured,
/// `.RexxInfo~digits(1)` is 93 on the oracle and here.
pub(super) static NATIVE_METHODS: &[(&str, &str, Arity, NativeMethod)] = &[
    ("RexxInfo", "ARCHITECTURE", Arity::Fixed(0), architecture),
    (
        "RexxInfo",
        "CASESENSITIVEFILES",
        Arity::Fixed(0),
        case_sensitive_files,
    ),
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
///
/// Rust's standard library exposes no such limit and this workspace has no
/// `libc` dependency, so the header's value is mirrored here.
const PATH_MAX: usize = 4096;

/// `ORX_BLD`: the source-control revision the interpreter was configured at.
///
/// `CMakeLists.txt:87` sets it to `0` and `:143` overrides it with the
/// working copy's last-changed revision when the source is an SVN checkout.
/// Nothing behind this crate is one, so the configured default is the answer
/// -- and it is separate from `modification`, which is `ORX_MOD` and moves
/// with the version string.
const BUILD_LEVEL: i64 = 0;

/// `Numerics::MAX_WHOLENUMBER`: the largest value a whole-number conversion
/// can produce, and the ceiling on `NUMERIC DIGITS` itself.
///
/// **Derived from [`rexx_num::ARGUMENT_DIGITS`] rather than written out.**
/// `runtime/Numerics.hpp:85`-`:98` sets the two together per pointer width and
/// its own comment says `ARGUMENT_DIGITS` is the digits setting chosen "to
/// allow for the full range": eighteen digits beside eighteen nines on a
/// 64-bit build, nine beside nine on a 32-bit one. Derived, this answers the
/// right one on either.
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
///
/// **This mirrors that function's fallback rather than its probe.** On Linux
/// the probe is `ioctl(FS_IOC_GETFLAGS)` looking for `FS_CASEFOLD_FL`
/// (`platform/unix/SysFileSystem.cpp:1321`-`:1335`), and where it is
/// unavailable the C++ takes its own documented "non-determined, just return
/// true" path at `:1335`. This workspace has no `libc` dependency and denies
/// `unsafe`, so the ioctl is not reachable and the fallback is what is left.
/// A casefolded ext4 directory would make the oracle answer `0` where this
/// answers `1`.
fn case_sensitive_files(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    Ok(Some(interp.counted(1)))
}

/// `RexxInfo::getInterpreterDate`: the build date, which is
/// [`parse_template::VERSION`]'s tail.
fn date(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    Ok(Some(interp.text(parse_template::BUILD_DATE)))
}

/// `RexxInfo::getDebug`: `#ifdef _DEBUG`.
///
/// **A constant, and not this build's own `debug_assertions`.** `_DEBUG` is
/// an MSVC construct: no `-D_DEBUG` appears in ooRexx's `CMakeLists.txt`, and
/// the oracle's `CMakeCache.txt` has `CMAKE_CXX_FLAGS_DEBUG:STRING=-g` with
/// nothing defining it, so `getDebug` answers `.false` for every Linux build
/// of the interpreter including a Debug one.
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
    Ok(Some(interp.text(parse_template::LANGUAGE_LEVEL)))
}

/// `RexxInfo::getMajorVersion`: `ORX_VER`.
fn major_version(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    Ok(Some(interp.text(parse_template::MAJOR_VERSION)))
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
    Ok(Some(interp.text(parse_template::MODIFICATION)))
}

/// `RexxInfo::getInterpreterName`: `Interpreter::getVersionString()`, the
/// same string `PARSE VERSION` answers and the same constant.
fn name(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    Ok(Some(interp.text(parse_template::VERSION)))
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
    Ok(Some(interp.text(parse_template::PLATFORM)))
}

/// `RexxInfo::getRelease`: `ORX_REL`.
fn release(
    interp: &mut Interp,
    _cleared: Cleared,
    _receiver: ObjRef,
    _args: &[Option<ObjRef>],
) -> Result<Option<ObjRef>, Failure> {
    Ok(Some(interp.text(parse_template::RELEASE)))
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
    Ok(Some(interp.text(parse_template::VERSION_NUMBER)))
}
