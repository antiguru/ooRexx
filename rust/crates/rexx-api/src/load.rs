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

// D-U1, 2026-09-14
#![allow(unsafe_code)]

//! The outbound FFI boundary: loading a library and resolving symbols.

use std::ffi::{CStr, c_char, c_int, c_void};
use std::path::Path;

/// The interpreter version an extension's `requiredVersion` is measured
/// against, `REXX_CURRENT_INTERPRETER_VERSION` (`api/oorexxapi.h:242`).
///
/// Two hex digits each for major, minor and revision (`:229`).
/// `tests/load.rs` re-reads the header and asserts this value against it.
pub const CURRENT_INTERPRETER_VERSION: c_int = 0x0005_0300;

/// The symbol `OOREXX_GET_PACKAGE` publishes (`api/oorexxapi.h:253-256`).
const GET_PACKAGE_SYMBOL: &[u8] = b"RexxGetPackage\0";

/// `SysLibrary::load` refuses a longer name before it reaches `dlopen`
/// (`common/platform/unix/SysLibrary.cpp:56`, `:83-86`).
const MAX_LIBRARY_NAME_LENGTH: usize = 250;

/// Where the name search looks after the undecorated attempt
/// (`common/platform/unix/SysLibrary.cpp:94`).
const SECOND_ATTEMPT_DIRECTORY: &str = "/usr/lib";

/// A condition the load path raises, for the caller to render.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Failure {
    /// `Error_Execution_library_version`, raised where the package entry asks
    /// for a newer interpreter than this one
    /// (`interpreter/package/LibraryPackage.cpp:232-235`). The payload is the
    /// library name the message interpolates.
    LibraryVersion(String),
}

impl Failure {
    /// The major and minor error numbers this failure reports.
    #[must_use]
    pub fn code(&self) -> (u16, u16) {
        match self {
            Failure::LibraryVersion(_) => (98, 982),
        }
    }
}

/// `RexxPackageLoader` and `RexxPackageUnloader` (`api/oorexxapi.h:259-260`).
///
/// The argument is the thread context Task 3 defines; nothing here calls
/// either hook.
pub type PackageHook = unsafe extern "C" fn(*mut c_void);

/// `RexxRoutineEntry` (`api/oorexxapi.h:190-198`).
#[repr(C)]
#[derive(Debug)]
pub struct RexxRoutineEntry {
    pub style: c_int,
    pub reserved1: c_int,
    pub name: *const c_char,
    pub entry_point: *mut c_void,
    pub reserved2: c_int,
    pub reserved3: c_int,
}

/// `RexxMethodEntry` (`api/oorexxapi.h:211-219`).
#[repr(C)]
#[derive(Debug)]
pub struct RexxMethodEntry {
    pub style: c_int,
    pub reserved1: c_int,
    pub name: *const c_char,
    pub entry_point: *mut c_void,
    pub reserved2: c_int,
    pub reserved3: c_int,
}

/// `RexxPackageEntry` (`api/oorexxapi.h:262-273`).
#[repr(C)]
#[derive(Debug)]
pub struct RexxPackageEntry {
    pub size: c_int,
    pub api_version: c_int,
    pub required_version: c_int,
    pub package_name: *const c_char,
    pub package_version: *const c_char,
    pub loader: Option<PackageHook>,
    pub unloader: Option<PackageHook>,
    pub routines: *mut RexxRoutineEntry,
    pub methods: *mut RexxMethodEntry,
}

/// One row of an extension's method table, copied out of the library.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeMethodEntry {
    /// `METHOD_TYPED_STYLE` (`api/oorexxapi.h:221`); never zero, which is the
    /// terminator.
    pub style: c_int,
    pub name: Vec<u8>,
    /// Valid only while the [`Library`] this row came from is alive.
    pub entry_point: *mut c_void,
}

/// One row of an extension's routine table, copied out of the library.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NativeRoutineEntry {
    /// `ROUTINE_TYPED_STYLE` or `ROUTINE_CLASSIC_STYLE`
    /// (`api/oorexxapi.h:200-201`); never zero, which is the terminator.
    pub style: c_int,
    pub name: Vec<u8>,
    /// Valid only while the [`Library`] this row came from is alive.
    pub entry_point: *mut c_void,
}

/// An opened shared library together with the package entry it published.
#[derive(Debug)]
pub struct Library {
    name: Option<Vec<u8>>,
    version: Option<Vec<u8>>,
    methods: Vec<NativeMethodEntry>,
    routines: Vec<NativeRoutineEntry>,
    /// Last field, so it is dropped last: every `entry_point` above points
    /// into this mapping and dies with it. Held for that alone, which is a
    /// use `dead_code` does not count.
    #[expect(dead_code)]
    handle: libloading::Library,
}

impl Library {
    /// The entry's `packageName`, or `None` where that pointer is null.
    #[must_use]
    pub fn name(&self) -> Option<&[u8]> {
        self.name.as_deref()
    }

    /// The entry's `packageVersion`; `NO_VERSION_YET` is a null pointer
    /// (`api/oorexxapi.h:243`) and answers `None`.
    #[must_use]
    pub fn version(&self) -> Option<&[u8]> {
        self.version.as_deref()
    }

    /// The exported method table, in the order the extension declares it.
    #[must_use]
    pub fn methods(&self) -> &[NativeMethodEntry] {
        &self.methods
    }

    /// The exported routine table, in the order the extension declares it.
    #[must_use]
    pub fn routines(&self) -> &[NativeRoutineEntry] {
        &self.routines
    }

    /// The method row `name` names, matched without regard to case as
    /// `LibraryPackage::locateMethodEntry` matches it
    /// (`interpreter/package/LibraryPackage.cpp:320-325`).
    ///
    /// The row's `entry_point` is valid only while `self` is alive.
    #[must_use]
    pub fn method(&self, name: &[u8]) -> Option<NativeMethodEntry> {
        self.methods
            .iter()
            .find(|row| row.name.eq_ignore_ascii_case(name))
            .cloned()
    }
}

/// Open the library `name` names, searching as `SysLibrary::load` searches
/// (`common/platform/unix/SysLibrary.cpp:75-102`).
///
/// `Ok(None)` is the answer where no library of that name loads, and where one
/// loads but publishes no `RexxGetPackage`. Neither is an error here: the C++
/// leaves that decision to the caller at both sites
/// (`interpreter/package/LibraryPackage.cpp:199-204`, `:211-215`).
///
/// # Errors
/// [`Failure::LibraryVersion`] where the package entry asks for a newer
/// interpreter than this one.
pub fn open(name: &str) -> Result<Option<Library>, Failure> {
    if name.len() > MAX_LIBRARY_NAME_LENGTH {
        return Ok(None);
    }
    let file = format!("lib{name}{}", std::env::consts::DLL_SUFFIX);
    let handle = dlopen(Path::new(&file))
        .or_else(|| dlopen(&Path::new(SECOND_ATTEMPT_DIRECTORY).join(&file)));
    let Some(handle) = handle else {
        return Ok(None);
    };
    package_of(handle, name)
}

/// Open the library file at `path`, skipping the name search.
///
/// Answers and errors as [`open`] does, and the error names `path` where
/// [`open`] names the library.
///
/// # Errors
/// [`Failure::LibraryVersion`] where the package entry asks for a newer
/// interpreter than this one.
pub fn open_path(path: &Path) -> Result<Option<Library>, Failure> {
    let Some(handle) = dlopen(path) else {
        return Ok(None);
    };
    package_of(handle, &path.display().to_string())
}

/// Whether the library at `path` loads at all, which is what separates
/// [`open_path`]'s two `Ok(None)` answers from one another.
#[must_use]
pub fn loads(path: &Path) -> bool {
    dlopen(path).is_some()
}

/// The version gate `LibraryPackage::loadPackage` applies before it registers
/// anything (`interpreter/package/LibraryPackage.cpp:232-235`).
///
/// `name` is the library name the raised message interpolates.
///
/// # Errors
/// [`Failure::LibraryVersion`] where `requiredVersion` is non-zero and above
/// [`CURRENT_INTERPRETER_VERSION`].
pub fn check_version(entry: &RexxPackageEntry, name: &str) -> Result<(), Failure> {
    if entry.required_version != 0 && entry.required_version > CURRENT_INTERPRETER_VERSION {
        return Err(Failure::LibraryVersion(name.to_owned()));
    }
    Ok(())
}

fn dlopen(path: &Path) -> Option<libloading::Library> {
    // `Library::new` is `dlopen(path, RTLD_LAZY | RTLD_LOCAL)`, and glibc
    // resolves the oracle's bare `RTLD_LAZY` (`SysLibrary.cpp:90`) to that
    // same pair.
    //
    // SAFETY: `dlopen` runs the library's initialisers, which is code this
    // process cannot vet, so no property of `path` makes the call sound. What
    // establishes it is the interpreter's own contract: a Rexx program that
    // names a library is authorised to load it, exactly the trust the oracle
    // extends at `SysLibrary.cpp:90`. D-U1 puts the `unsafe` here so that
    // `open`, `open_path` and `loads` stay safe for the rest of the tree.
    unsafe { libloading::Library::new(path) }.ok()
}

fn package_of(handle: libloading::Library, name: &str) -> Result<Option<Library>, Failure> {
    type GetPackage = unsafe extern "C" fn() -> *mut RexxPackageEntry;

    let get_package = {
        // SAFETY: the symbol is read at the signature `OOREXX_GET_PACKAGE`
        // expands to, a nullary function answering a `RexxPackageEntry *`
        // (`api/oorexxapi.h:255`). D5 freezes that header and makes it the
        // declaration both sides compile against, so an extension exporting
        // the name at another signature is outside the contract.
        let symbol: libloading::Symbol<'_, GetPackage> =
            match unsafe { handle.get(GET_PACKAGE_SYMBOL) } {
                Ok(symbol) => symbol,
                // A library without the exporter is a classic registration
                // rather than a failure (`LibraryPackage.cpp:211-215`).
                Err(_) => return Ok(None),
            };
        *symbol
    };

    // SAFETY: the address came from `dlsym` on a library `handle` still holds,
    // so the code it names stays mapped across the call. The function answers
    // the address of a static in that library and does nothing else
    // (`api/oorexxapi.h:255`).
    let entry = unsafe { get_package() };
    if entry.is_null() {
        // `LibraryPackage::load` treats a null table as "not a package"
        // (`LibraryPackage.cpp:148-155`).
        return Ok(None);
    }

    // SAFETY: `entry` is non-null and, by the contract above, is the address
    // of the `RexxPackageEntry` static the library defines, which lives as
    // long as the mapping `handle` holds. Its layout is the frozen header's.
    let entry = unsafe { &*entry };

    check_version(entry, name)?;

    // SAFETY: `entry` is the library's own package entry, so each table
    // pointer is null or the array the extension declared, terminated by a
    // zero-`style` row, and every string in it is a literal in the same
    // mapping. All of it outlives `handle`.
    let (name, version, methods, routines) = unsafe {
        (
            c_bytes(entry.package_name),
            c_bytes(entry.package_version),
            method_table(entry.methods),
            routine_table(entry.routines),
        )
    };

    Ok(Some(Library {
        name,
        version,
        methods,
        routines,
        handle,
    }))
}

/// The bytes a C string holds, or `None` where `ptr` is null.
///
/// # Safety
/// A non-null `ptr` addresses a NUL-terminated string that outlives the call.
unsafe fn c_bytes(ptr: *const c_char) -> Option<Vec<u8>> {
    if ptr.is_null() {
        return None;
    }
    // SAFETY: the caller guarantees a non-null `ptr` is a live
    // NUL-terminated string.
    Some(unsafe { CStr::from_ptr(ptr) }.to_bytes().to_vec())
}

/// The rows of a `RexxMethodEntry[]`, read to its `REXX_LAST_METHOD`
/// terminator.
///
/// # Safety
/// A non-null `table` addresses an array holding a row whose `style` is zero,
/// with every earlier row's `name` a live NUL-terminated string; all of it
/// outlives the call.
unsafe fn method_table(table: *mut RexxMethodEntry) -> Vec<NativeMethodEntry> {
    let mut rows = Vec::new();
    if table.is_null() {
        return rows;
    }
    let mut at = table;
    loop {
        // SAFETY: `at` starts at the caller's array and advances only past a
        // row that was not the terminator, so it stays inside it.
        let row = unsafe { &*at };
        // The oracle ends on `style == 0` and not on a null name
        // (`LibraryPackage.cpp:320`).
        if row.style == 0 {
            return rows;
        }
        rows.push(NativeMethodEntry {
            style: row.style,
            // SAFETY: `REXX_METHOD_ENTRY` stringizes the name
            // (`api/oorexxapi.h:223`), so a row the terminator check admits
            // carries a non-null one, held in the library's own image.
            name: unsafe { CStr::from_ptr(row.name) }.to_bytes().to_vec(),
            entry_point: row.entry_point,
        });
        // SAFETY: `at` was not the terminator, so a further row follows it.
        at = unsafe { at.add(1) };
    }
}

/// The rows of a `RexxRoutineEntry[]`, read to its `REXX_LAST_ROUTINE`
/// terminator.
///
/// # Safety
/// As [`method_table`].
unsafe fn routine_table(table: *mut RexxRoutineEntry) -> Vec<NativeRoutineEntry> {
    let mut rows = Vec::new();
    if table.is_null() {
        return rows;
    }
    let mut at = table;
    loop {
        // SAFETY: `at` starts at the caller's array and advances only past a
        // row that was not the terminator, so it stays inside it.
        let row = unsafe { &*at };
        // `loadRoutines` ends on `style == 0` (`LibraryPackage.cpp:270`).
        if row.style == 0 {
            return rows;
        }
        rows.push(NativeRoutineEntry {
            style: row.style,
            // SAFETY: `REXX_ROUTINE` stringizes the name
            // (`api/oorexxapi.h:203`), so a row the terminator check admits
            // carries a non-null one, held in the library's own image.
            name: unsafe { CStr::from_ptr(row.name) }.to_bytes().to_vec(),
            entry_point: row.entry_point,
        });
        // SAFETY: `at` was not the terminator, so a further row follows it.
        at = unsafe { at.add(1) };
    }
}
