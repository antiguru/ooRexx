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

use crate::ffi::MethodContext;
use crate::invoke::MAX_NATIVE_ARGUMENTS;
use crate::layout::{
    RexxMethodContext_, RexxMethodEntry, RexxPackageEntry, RexxRoutineEntry, ValueDescriptor,
    ValueUnion,
};
use crate::values::{ARGUMENT_TERMINATOR, Repr, Value};
use std::ffi::{CStr, c_char, c_int, c_void};
use std::path::{Path, PathBuf};

/// The C signature of the stub the `RexxMethodN` macros generate
/// (`api/oorexxapi.h:4286`).
///
/// A null `arguments` is the signature request and answers the static type
/// array; an array is the call, and answers null (`:4282`).
pub(crate) type NativeMethod =
    unsafe extern "C" fn(*mut RexxMethodContext_, *mut ValueDescriptor) -> *mut u16;

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

/// A library whose package entry [`check_version`] refused, with the library as
/// `LibraryPackage::loadPackage` leaves it once the check has raised: loaded,
/// its method table resolving, and its routine table never read, because
/// `loadRoutines` follows the check (`interpreter/package/LibraryPackage.cpp:232-237`).
#[derive(Debug)]
pub struct Refused {
    pub failure: Failure,
    pub library: Box<Library>,
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

/// One row of an extension's method table, copied out of the library.
///
/// Neither `Clone` nor holder of a public address: both would hand safe code
/// something that outlives the mapping the address points into. A raw pointer
/// is `Copy` and carries no lifetime, so a borrowed row is not on its own
/// enough, and the field is private for that reason.
///
/// ```compile_fail
/// # use rexx_api::load;
/// let row = load::open_path(std::path::Path::new("librxregexp.so"), "rxregexp")
///     .unwrap()
///     .unwrap();
/// let address = row.method(b"RegExp_Parse").unwrap().entry_point;
/// ```
#[derive(Debug, PartialEq, Eq)]
pub struct NativeMethodEntry {
    /// `METHOD_TYPED_STYLE` (`api/oorexxapi.h:221`); never zero, which is the
    /// terminator.
    pub style: c_int,
    pub name: Vec<u8>,
    entry_point: *mut c_void,
}

impl NativeMethodEntry {
    /// Whether the row carries an address.
    #[must_use]
    pub fn has_entry_point(&self) -> bool {
        !self.entry_point.is_null()
    }

    /// The type signature the stub publishes, terminator excluded, or `None`
    /// where the row carries no address, the stub answers no array, or no
    /// terminator appears in the first `limit` words.
    ///
    /// At most `limit - 1` words are answered, which is what bounds the
    /// descriptor array the caller fills from them.
    pub(crate) fn signature(&self, context: &MethodContext<'_>, limit: usize) -> Option<Vec<u16>> {
        let stub = self.stub()?;
        // SAFETY: `stub` is the address the extension's own method table gave
        // for this row, so the code it names is the generated stub and stays
        // mapped as long as the `Library` this row is borrowed from. A null
        // `arguments` is the signature request, which returns the static
        // array without running any of the extension's own code
        // (`api/oorexxapi.h:4342-4350`).
        let types = unsafe { stub(context.as_ptr(), std::ptr::null_mut()) };
        if types.is_null() {
            return None;
        }
        let mut words = Vec::new();
        for offset in 0..limit {
            // SAFETY: the invariant is the macro's, not this loop's: the
            // array it emits ends in `REXX_ARGUMENT_TERMINATOR`
            // (`api/oorexxapi.h:4338`), so every offset up to the first
            // terminator is inside it. `limit` bounds how far a malformed
            // array is followed, which the oracle does not bound at all
            // (`NativeActivation.cpp:235`).
            let word = unsafe { *types.add(offset) };
            if word == ARGUMENT_TERMINATOR {
                return Some(words);
            }
            words.push(word);
        }
        None
    }

    /// Call the stub with `arguments`, which it reads and writes its result
    /// into, and answer element zero read as `result`'s member, or `None`
    /// where `result` is. A row with no address calls nothing.
    ///
    /// The array is published on `context` for the call and taken off again
    /// afterwards, because it does not outlive this function.
    pub(crate) fn call(
        &self,
        context: &MethodContext<'_>,
        arguments: &mut [ValueDescriptor; MAX_NATIVE_ARGUMENTS],
        result: Option<Repr>,
    ) -> Option<Value> {
        // The whole word, before the stub can write a narrower member into it.
        arguments[0].value = ValueUnion { value_int64_t: 0 };
        if let Some(stub) = self.stub() {
            let array = arguments.as_mut_ptr();
            // `argumentExists` reads the array through the context rather
            // than through the parameter (`api/oorexxapi.h:4276`), which is
            // why the oracle publishes it (`NativeActivation.cpp:1291`).
            let pointer = context.as_ptr();
            // SAFETY: `context` holds the only borrow of a live
            // `RexxMethodContext_` for its lifetime, and it is neither `Clone`
            // nor `Sync`, so nothing else writes the struct during this call.
            unsafe { (*pointer).arguments = array };
            // SAFETY: `stub` names the generated stub, as above. `array` is
            // the caller's live array, which nothing else touches for the
            // duration of the call, and the stub reads and writes only the
            // elements its own signature declares.
            unsafe { stub(pointer, array) };
            // SAFETY: as the write above.
            unsafe { (*pointer).arguments = std::ptr::null_mut() };
        }
        // SAFETY: element zero's word was written in full above, and a stub
        // writing a member into it leaves every byte initialised.
        result.map(|repr| unsafe { crate::ffi::value_of(&arguments[0], repr) })
    }

    /// The row's address as the callable it names, or `None` for a row that
    /// carries none.
    fn stub(&self) -> Option<NativeMethod> {
        if self.entry_point.is_null() {
            return None;
        }
        // SAFETY: `REXX_METHOD_ENTRY` fills `entryPoint` with the stub the
        // `RexxMethodN` macro generated (`api/oorexxapi.h:223`), whose C
        // signature is `NativeMethod`'s. D5 freezes that header and makes it
        // the declaration both sides compile against, so the code at this
        // address has this type. The two are the same width, which
        // `transmute` checks.
        Some(unsafe { std::mem::transmute::<*mut c_void, NativeMethod>(self.entry_point) })
    }
}

/// `METHOD_TYPED_STYLE` (`api/oorexxapi.h:221`).
#[cfg(test)]
const METHOD_TYPED_STYLE: c_int = 1;

/// A method row for a stub this image itself defines, for a caller that needs
/// an entry point without a library.
#[cfg(test)]
pub(crate) fn stub_entry(name: &[u8], stub: NativeMethod) -> NativeMethodEntry {
    NativeMethodEntry {
        style: METHOD_TYPED_STYLE,
        name: name.to_vec(),
        entry_point: stub as *mut c_void,
    }
}

/// One row of an extension's routine table, copied out of the library.
///
/// Shaped as [`NativeMethodEntry`] and for the same reason.
#[derive(Debug, PartialEq, Eq)]
pub struct NativeRoutineEntry {
    /// `ROUTINE_TYPED_STYLE` or `ROUTINE_CLASSIC_STYLE`
    /// (`api/oorexxapi.h:200-201`); never zero, which is the terminator.
    pub style: c_int,
    pub name: Vec<u8>,
    entry_point: *mut c_void,
}

impl NativeRoutineEntry {
    /// Whether the row carries an address.
    #[must_use]
    pub fn has_entry_point(&self) -> bool {
        !self.entry_point.is_null()
    }
}

/// An opened shared library together with the package entry it published.
#[derive(Debug)]
pub struct Library {
    name: Option<Vec<u8>>,
    version: Option<Vec<u8>>,
    methods: Vec<NativeMethodEntry>,
    routines: Vec<NativeRoutineEntry>,
    /// The mapping every `entry_point` above addresses. No safe code outside
    /// this module can copy an address out of a row, which is what keeps one
    /// from outliving this field.
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
    /// The row is borrowed from `self`, which is what stops it outliving the
    /// mapping its address points into.
    ///
    /// ```no_run
    /// # use rexx_api::load;
    /// let library = load::open_path(std::path::Path::new("librxregexp.so"), "rxregexp")
    ///     .unwrap()
    ///     .unwrap();
    /// let row = library.method(b"RegExp_Parse").unwrap();
    /// assert!(row.has_entry_point());
    /// ```
    ///
    /// ```compile_fail
    /// # use rexx_api::load;
    /// let row = load::open_path(std::path::Path::new("librxregexp.so"), "rxregexp")
    ///     .unwrap()
    ///     .unwrap()
    ///     .method(b"RegExp_Parse")
    ///     .unwrap();
    /// assert!(row.has_entry_point());
    /// ```
    #[must_use]
    pub fn method(&self, name: &[u8]) -> Option<&NativeMethodEntry> {
        self.methods
            .iter()
            .find(|row| row.name.eq_ignore_ascii_case(name))
    }

    /// The routine row `name` names, matched as
    /// `LibraryPackage::locateRoutineEntry` matches it
    /// (`interpreter/package/LibraryPackage.cpp:344-356`), and borrowed from
    /// `self` for [`Library::method`]'s reason.
    #[must_use]
    pub fn routine(&self, name: &[u8]) -> Option<&NativeRoutineEntry> {
        self.routines
            .iter()
            .find(|row| row.name.eq_ignore_ascii_case(name))
    }
}

/// Open the library `name` names, searching as `SysLibrary::load` searches
/// (`common/platform/unix/SysLibrary.cpp:75-102`).
///
/// `search` is tried, in order, before the undecorated name. The loader reads
/// `LD_LIBRARY_PATH` once at process start, so a directory the interpreter
/// learned of afterwards is unreachable through the undecorated attempt and
/// has to be spelled out here; the caller supplies the directories that
/// variable names.
///
/// `Ok(None)` is the answer where no library of that name loads, and where one
/// loads but publishes no `RexxGetPackage`. Neither is an error here: the C++
/// leaves that decision to the caller at both sites
/// (`interpreter/package/LibraryPackage.cpp:199-204`, `:211-215`).
///
/// # Errors
/// [`Refused`] where the package entry asks for a newer interpreter than this
/// one.
pub fn open(name: &str, search: &[PathBuf]) -> Result<Option<Library>, Refused> {
    if name.len() > MAX_LIBRARY_NAME_LENGTH {
        return Ok(None);
    }
    let file = format!("lib{name}{}", std::env::consts::DLL_SUFFIX);
    let handle = search
        .iter()
        .find_map(|directory| dlopen(&directory.join(&file)))
        .or_else(|| dlopen(Path::new(&file)))
        .or_else(|| dlopen(&Path::new(SECOND_ATTEMPT_DIRECTORY).join(&file)));
    let Some(handle) = handle else {
        return Ok(None);
    };
    package_of(handle, name)
}

/// Open the library file at `path`, skipping the name search.
///
/// Answers as [`open`] does. `name` is what a raised [`Failure`] interpolates,
/// which for a `::REQUIRES` is the spelling in the source rather than `path`.
///
/// # Errors
/// [`Refused`] where the package entry asks for a newer interpreter than this
/// one.
pub fn open_path(path: &Path, name: &str) -> Result<Option<Library>, Refused> {
    let Some(handle) = dlopen(path) else {
        return Ok(None);
    };
    package_of(handle, name)
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

fn package_of(handle: libloading::Library, name: &str) -> Result<Option<Library>, Refused> {
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

    let refused = check_version(entry, name).err();

    // SAFETY: `entry` is the library's own package entry, so each table
    // pointer is null or the array the extension declared, terminated by a
    // zero-`style` row, and every string in it is a literal in the same
    // mapping. All of it outlives `handle`.
    let (package_name, version, methods) = unsafe {
        (
            c_bytes(entry.package_name),
            c_bytes(entry.package_version),
            method_table(entry.methods),
        )
    };
    let routines = match refused {
        Some(_) => Vec::new(),
        // SAFETY: as above.
        None => unsafe { routine_table(entry.routines) },
    };
    let library = Library {
        name: package_name,
        version,
        methods,
        routines,
        handle,
    };
    match refused {
        Some(failure) => Err(Refused {
            failure,
            library: Box::new(library),
        }),
        None => Ok(Some(library)),
    }
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
