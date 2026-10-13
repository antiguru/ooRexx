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

//! The routines the oracle's two internal packages export, which are neither
//! builtins nor `::ROUTINE`s: `REXX` (`runtime/NativeFunctions.h`) and
//! `REXXUTIL` (`runtime/RexxUtilCommon.cpp`'s table plus the platform half in
//! `platform/unix/SysRexxUtilFunctions.h`). The oracle consults them after a
//! program's own routines and before the external file search, so a name here
//! that this crate has no body for must refuse rather than fall through to
//! 43.1 -- a "routine not found" a program cannot tell from its own typo.
//! `tests/internal_routines.rs` re-derives this table from those three C++
//! files.

use rexx_core::{Deferral, Owner};

/// Which internal package exports the routine.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) enum Package {
    /// `runtime/NativeFunctions.h`, loaded as the `REXX` package.
    Rexx,
    /// `runtime/RexxUtilCommon.cpp`, loaded as the `REXXUTIL` package.
    RexxUtil,
}

impl Package {
    /// The package's name, as the C++ registers it.
    pub(crate) fn label(self) -> &'static str {
        match self {
            Package::Rexx => "REXX",
            Package::RexxUtil => "REXXUTIL",
        }
    }
}

/// What one internal routine runs: the same shape a builtin's body has, since
/// both are entered with their arguments already evaluated and neither pushes
/// an activation.
pub(crate) type InternalBody = fn(
    &mut crate::Interp,
    &'static [u8],
    &[Option<rexx_core::ObjRef>],
) -> Result<rexx_core::ObjRef, crate::Failure>;

/// One routine an internal package exports.
pub(crate) struct InternalRoutine {
    /// The name as the C++ registers it; the lookup is caseless, so the
    /// spelling is for a reader.
    pub(crate) name: &'static str,
    pub(crate) package: Package,
    /// The phase owing a body, or `None` for a row this crate runs. **Not a
    /// field beside `body`**: a row has one or the other, and an owner on a
    /// row that runs is a phase name nothing can print.
    pub(crate) owner: Option<Owner>,
    /// The code, once some phase has written it. A row with `None` refuses,
    /// naming [`InternalRoutine::owner`].
    pub(crate) body: Option<InternalBody>,
}

/// A `REXX`-package row this crate runs.
const fn rexx_run(name: &'static str, body: InternalBody) -> InternalRoutine {
    InternalRoutine {
        name,
        package: Package::Rexx,
        owner: None,
        body: Some(body),
    }
}

/// A `REXXUTIL` row this crate runs.
const fn util_run(name: &'static str, body: InternalBody) -> InternalRoutine {
    InternalRoutine {
        name,
        package: Package::RexxUtil,
        owner: None,
        body: Some(body),
    }
}

const fn util(name: &'static str, owner: Owner) -> InternalRoutine {
    InternalRoutine {
        name,
        package: Package::RexxUtil,
        owner: Some(owner),
        body: None,
    }
}

/// Every name the two packages register on Linux.
pub(crate) static INTERNAL_ROUTINES: &[InternalRoutine] = &[
    rexx_run("Directory", crate::builtin::platform::directory),
    rexx_run("Filespec", crate::builtin::platform::filespec),
    rexx_run("Beep", crate::builtin::platform::beep),
    util("SysAddRexxMacro", Owner::Phase10(Deferral::Rxapi)),
    util("SysClearRexxMacroSpace", Owner::Phase10(Deferral::Rxapi)),
    util_run("SysCloseEventSem", crate::builtin::rexxutil::close_sem),
    util_run("SysCloseMutexSem", crate::builtin::rexxutil::close_sem),
    util("SysCls", Owner::Phase9),
    util_run(
        "SysCreateEventSem",
        crate::builtin::rexxutil::create_event_sem,
    ),
    util_run(
        "SysCreateMutexSem",
        crate::builtin::rexxutil::create_mutex_sem,
    ),
    util("SysCreatePipe", Owner::Phase9),
    util("SysDropFuncs", Owner::Phase9),
    util("SysDropRexxMacro", Owner::Phase10(Deferral::Rxapi)),
    util_run(
        "SysDumpVariables",
        crate::builtin::rexxutil::stems::dump_variables,
    ),
    util_run("SysFileCopy", crate::builtin::rexxutil::files::file_copy),
    util_run("SysFileDelete", crate::builtin::rexxutil::file_delete),
    util_run("SysFileExists", crate::builtin::rexxutil::file_exists),
    util_run("SysFileMove", crate::builtin::rexxutil::files::file_move),
    util_run(
        "SysFileSearch",
        crate::builtin::rexxutil::files::file_search,
    ),
    util_run("SysFileTree", crate::builtin::rexxutil::file_tree),
    util("SysFork", Owner::Phase9),
    util("SysFormatMessage", Owner::Phase9),
    util("SysGetErrorText", Owner::Phase9),
    util_run(
        "SysGetFileDateTime",
        crate::builtin::rexxutil::files::get_file_date_time,
    ),
    util("SysGetKey", Owner::Phase9),
    util("SysGetMessage", Owner::Phase9),
    util("SysGetMessageX", Owner::Phase9),
    util_run("SysIsFile", crate::builtin::rexxutil::is_file),
    util_run(
        "SysIsFileDirectory",
        crate::builtin::rexxutil::files::is_file_directory,
    ),
    util_run(
        "SysIsFileLink",
        crate::builtin::rexxutil::files::is_file_link,
    ),
    util_run("SysLinVer", crate::builtin::rexxutil::version),
    util("SysLoadFuncs", Owner::Phase9),
    util("SysLoadRexxMacroSpace", Owner::Phase10(Deferral::Rxapi)),
    util_run("SysMkDir", crate::builtin::rexxutil::mk_dir),
    util("SysOpenEventSem", Owner::Phase9),
    util("SysOpenMutexSem", Owner::Phase9),
    util_run("SysPostEventSem", crate::builtin::rexxutil::post_event_sem),
    util("SysQueryProcess", Owner::Phase9),
    util("SysQueryRexxMacro", Owner::Phase10(Deferral::Rxapi)),
    util_run(
        "SysReleaseMutexSem",
        crate::builtin::rexxutil::release_mutex_sem,
    ),
    util("SysReorderRexxMacro", Owner::Phase10(Deferral::Rxapi)),
    util_run("SysRequestMutexSem", crate::builtin::rexxutil::wait_sem),
    util_run(
        "SysResetEventSem",
        crate::builtin::rexxutil::reset_event_sem,
    ),
    util_run("SysRmDir", crate::builtin::rexxutil::rm_dir),
    util("SysSaveRexxMacroSpace", Owner::Phase10(Deferral::Rxapi)),
    util_run(
        "SysSearchPath",
        crate::builtin::rexxutil::files::search_path,
    ),
    util_run(
        "SysSetFileDateTime",
        crate::builtin::rexxutil::files::set_file_date_time,
    ),
    util("SysSetPriority", Owner::Phase9),
    util_run("SysSleep", crate::builtin::rexxutil::sleep),
    util_run("SysStemCopy", crate::builtin::rexxutil::stems::copy),
    util_run("SysStemDelete", crate::builtin::rexxutil::stems::delete),
    util_run("SysStemInsert", crate::builtin::rexxutil::stems::insert),
    util_run("SysStemSort", crate::builtin::rexxutil::stems::sort),
    util_run(
        "SysTempFileName",
        crate::builtin::rexxutil::files::temp_file_name,
    ),
    util("SysUtilVersion", Owner::Phase9),
    util_run("SysVersion", crate::builtin::rexxutil::version),
    util("SysWait", Owner::Phase9),
    util_run("SysWaitEventSem", crate::builtin::rexxutil::wait_sem),
];

/// The `REXX` package's row `name` names, found as
/// `LibraryPackage::resolveRoutine` finds a routine table entry
/// (`interpreter/package/LibraryPackage.cpp:410-435`): by exact spelling, then
/// without regard to case.
pub(crate) fn rexx_package_routine(name: &[u8]) -> Option<&'static InternalRoutine> {
    let rows = || {
        INTERNAL_ROUTINES
            .iter()
            .filter(|row| row.package == Package::Rexx)
    };
    rows()
        .find(|row| row.name.as_bytes() == name)
        .or_else(|| rows().find(|row| row.name.as_bytes().eq_ignore_ascii_case(name)))
}

/// The row `name` names, matched caselessly as the oracle's own lookup does.
pub(crate) fn lookup(name: &[u8]) -> Option<&'static InternalRoutine> {
    INTERNAL_ROUTINES
        .iter()
        .find(|row| row.name.as_bytes().eq_ignore_ascii_case(name))
}
