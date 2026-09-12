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
    /// The phase owing a body, for as long as there is none.
    pub(crate) owner: &'static str,
    /// The code, once some phase has written it. A row with `None` refuses,
    /// naming [`InternalRoutine::owner`].
    pub(crate) body: Option<InternalBody>,
}

/// A `REXX`-package row this phase implements.
const fn rexx_run(name: &'static str, body: InternalBody) -> InternalRoutine {
    InternalRoutine {
        name,
        package: Package::Rexx,
        owner: "Phase 7",
        body: Some(body),
    }
}

const fn util(name: &'static str, owner: &'static str) -> InternalRoutine {
    InternalRoutine {
        name,
        package: Package::RexxUtil,
        owner,
        body: None,
    }
}

/// Every name the two packages register on Linux.
pub(crate) static INTERNAL_ROUTINES: &[InternalRoutine] = &[
    rexx_run("Directory", crate::builtin::platform::directory),
    rexx_run("Filespec", crate::builtin::platform::filespec),
    rexx_run("Beep", crate::builtin::platform::beep),
    util("SysAddRexxMacro", "Phase 10"),
    util("SysClearRexxMacroSpace", "Phase 10"),
    util("SysCloseEventSem", "Phase 6"),
    util("SysCloseMutexSem", "Phase 6"),
    util("SysCls", "Phase 10"),
    util("SysCreateEventSem", "Phase 6"),
    util("SysCreateMutexSem", "Phase 6"),
    util("SysCreatePipe", "Phase 10"),
    util("SysDropFuncs", "Phase 10"),
    util("SysDropRexxMacro", "Phase 10"),
    util("SysDumpVariables", "Phase 10"),
    util("SysFileCopy", "Phase 10"),
    util("SysFileDelete", "Phase 7"),
    util("SysFileExists", "Phase 7"),
    util("SysFileMove", "Phase 10"),
    util("SysFileSearch", "Phase 10"),
    util("SysFileTree", "Phase 7"),
    util("SysFork", "Phase 10"),
    util("SysFormatMessage", "Phase 10"),
    util("SysGetErrorText", "Phase 10"),
    util("SysGetFileDateTime", "Phase 10"),
    util("SysGetKey", "Phase 10"),
    util("SysGetMessage", "Phase 10"),
    util("SysGetMessageX", "Phase 10"),
    util("SysIsFile", "Phase 7"),
    util("SysIsFileDirectory", "Phase 10"),
    util("SysIsFileLink", "Phase 10"),
    util("SysLinVer", "Phase 7"),
    util("SysLoadFuncs", "Phase 10"),
    util("SysLoadRexxMacroSpace", "Phase 10"),
    util("SysMkDir", "Phase 7"),
    util("SysOpenEventSem", "Phase 6"),
    util("SysOpenMutexSem", "Phase 6"),
    util("SysPostEventSem", "Phase 6"),
    util("SysQueryProcess", "Phase 10"),
    util("SysQueryRexxMacro", "Phase 10"),
    util("SysReleaseMutexSem", "Phase 6"),
    util("SysReorderRexxMacro", "Phase 10"),
    util("SysRequestMutexSem", "Phase 6"),
    util("SysResetEventSem", "Phase 6"),
    util("SysRmDir", "Phase 7"),
    util("SysSaveRexxMacroSpace", "Phase 10"),
    util("SysSearchPath", "Phase 10"),
    util("SysSetFileDateTime", "Phase 10"),
    util("SysSetPriority", "Phase 10"),
    util("SysSleep", "Phase 7"),
    util("SysStemCopy", "Phase 10"),
    util("SysStemDelete", "Phase 10"),
    util("SysStemInsert", "Phase 10"),
    util("SysStemSort", "Phase 10"),
    util("SysTempFileName", "Phase 10"),
    util("SysUtilVersion", "Phase 10"),
    util("SysVersion", "Phase 7"),
    util("SysWait", "Phase 10"),
    util("SysWaitEventSem", "Phase 6"),
];

/// The row `name` names, matched caselessly as the oracle's own lookup does.
pub(crate) fn lookup(name: &[u8]) -> Option<&'static InternalRoutine> {
    INTERNAL_ROUTINES
        .iter()
        .find(|row| row.name.as_bytes().eq_ignore_ascii_case(name))
}
