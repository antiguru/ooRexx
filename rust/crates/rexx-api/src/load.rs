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

use crate::ffi::{CallContext, Contexts, MethodContext, ThreadContext};
use crate::invoke::MAX_NATIVE_ARGUMENTS;
use crate::layout::{
    PackageHook, RexxCallContext_, RexxMethodContext_, RexxMethodEntry, RexxPackageEntry,
    RexxRoutineEntry, RexxThreadContext_, ValueDescriptor, ValueUnion,
};
use crate::values::{ARGUMENT_TERMINATOR, ResultRead, Written};
use std::cell::{Cell, RefCell};
use std::ffi::{CStr, c_char, c_int, c_void};
use std::path::{Path, PathBuf};
use std::rc::Rc;

/// The C signature of the stub the `RexxMethodN` macros generate
/// (`api/oorexxapi.h:4286`).
///
/// A null `arguments` is the signature request and answers the static type
/// array; an array is the call, and answers null (`:4282`).
pub(crate) type NativeMethod = Stub<RexxMethodContext_>;

/// The C signature of the stub the `RexxRoutineN` macros generate
/// (`api/oorexxapi.h:4562`), answering as [`NativeMethod`] does.
pub(crate) type NativeRoutine = Stub<RexxCallContext_>;

/// A generated stub over the context struct `C`.
type Stub<C> = unsafe extern "C" fn(*mut C, *mut ValueDescriptor) -> *mut u16;

/// `ROUTINE_TYPED_STYLE` (`api/oorexxapi.h:200`).
pub const ROUTINE_TYPED_STYLE: c_int = 1;

/// `ROUTINE_CLASSIC_STYLE` (`api/oorexxapi.h:201`).
pub const ROUTINE_CLASSIC_STYLE: c_int = 2;

/// The interpreter version an extension's `requiredVersion` is measured
/// against, `REXX_CURRENT_INTERPRETER_VERSION` (`api/oorexxapi.h:242`).
///
/// Two hex digits each for major, minor and revision (`:229`).
/// `tests/load.rs` re-reads the header and asserts this value against it.
pub const CURRENT_INTERPRETER_VERSION: c_int = 0x0005_0300;

/// `REXX_CURRENT_LANGUAGE_LEVEL` (`api/oorexxapi.h:249`), which the instance
/// table's `LanguageLevel` answers (`interpreter/api/InterpreterInstanceStubs.cpp:84`,
/// through `Interpreter::getLanguageLevel`). `tests/load.rs` re-reads the
/// header and asserts this value against it.
pub const CURRENT_LANGUAGE_LEVEL: usize = 0x0606;

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

/// The mapping a library's addresses point into, shared by the library and
/// every row copied out of it, so that closing it stops every one of them
/// calling in.
struct Mapping {
    /// `None` for this image's own stubs, which nothing closes, and once
    /// closed.
    handle: RefCell<Option<libloading::Library>>,
    open: Cell<bool>,
    /// Calls into the mapping in flight, which a close waits out by refusing.
    calls: Cell<usize>,
}

impl Mapping {
    fn new(handle: Option<libloading::Library>) -> Rc<Mapping> {
        Rc::new(Mapping {
            handle: RefCell::new(handle),
            open: Cell::new(true),
            calls: Cell::new(0),
        })
    }

    /// Runs `call`, counted as in flight.
    fn hold<R>(&self, call: impl FnOnce() -> R) -> R {
        struct InFlight<'a>(&'a Cell<usize>);
        impl Drop for InFlight<'_> {
            fn drop(&mut self) {
                self.0.set(self.0.get() - 1);
            }
        }
        self.calls.set(self.calls.get() + 1);
        let _in_flight = InFlight(&self.calls);
        call()
    }

    /// Closes the mapping, or answers `false` where a call into it is in
    /// flight.
    fn close(&self) -> bool {
        if self.calls.get() > 0 {
            return false;
        }
        self.open.set(false);
        drop(self.handle.borrow_mut().take());
        true
    }
}

impl std::fmt::Debug for Mapping {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Mapping")
            .field("open", &self.open.get())
            .finish_non_exhaustive()
    }
}

/// One mapping is equal only to itself.
impl PartialEq for Mapping {
    fn eq(&self, other: &Mapping) -> bool {
        std::ptr::eq(self, other)
    }
}

impl Eq for Mapping {}

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
    mapping: Rc<Mapping>,
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
        // for this row, in a mapping that is open and that a close leaves
        // open while this call is held, and `context` is live for the borrow.
        self.mapping
            .hold(|| unsafe { signature_of(stub, context.as_ptr(), limit) })
    }

    /// Call the stub with `arguments`, which it reads and writes its result
    /// into, and answer element zero read as `result` says, or `None` where
    /// `result` is. A row with no address calls nothing.
    ///
    /// The array is published on `context` for the call and taken off again
    /// afterwards, because it does not outlive this function.
    pub(crate) fn call(
        &self,
        context: &MethodContext<'_>,
        arguments: &mut [ValueDescriptor; MAX_NATIVE_ARGUMENTS],
        result: Option<ResultRead>,
    ) -> Option<Written> {
        let pointer = context.as_ptr();
        // SAFETY: `pointer` addresses the live struct `context` borrows, and
        // naming a field's address reads and writes nothing.
        let published = unsafe { &raw mut (*pointer).arguments };
        let stub = self.stub();
        // SAFETY: as `signature`, and `published` is `pointer`'s own field.
        self.mapping
            .hold(|| unsafe { call_stub(stub, pointer, published, arguments, result) })
    }

    /// The row's address as the callable it names, or `None` for a row that
    /// carries none or whose library has been closed.
    fn stub(&self) -> Option<NativeMethod> {
        if self.entry_point.is_null() || !self.mapping.open.get() {
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
        mapping: Mapping::new(None),
    }
}

/// A routine row for a stub this image itself defines, for a caller that
/// needs an entry point without a library.
#[cfg(test)]
pub(crate) fn stub_routine_entry(
    style: c_int,
    name: &[u8],
    stub: NativeRoutine,
) -> NativeRoutineEntry {
    NativeRoutineEntry {
        style,
        name: name.to_vec(),
        entry_point: stub as *mut c_void,
        mapping: Mapping::new(None),
    }
}

/// A library read from an in-memory package entry that declares `loader` and
/// `unloader` and nothing else, over the running image, for a test that
/// drives the interpreter's own load and termination paths without a shared
/// object.
///
/// The hooks are safe functions, so running one is sound whatever it is.
#[doc(hidden)]
#[cfg(unix)]
#[must_use]
pub fn hooks_only(
    loader: Option<extern "C" fn(*mut RexxThreadContext_)>,
    unloader: Option<extern "C" fn(*mut RexxThreadContext_)>,
) -> Library {
    let entry = RexxPackageEntry {
        size: 0,
        api_version: 0,
        required_version: 0,
        package_name: std::ptr::null(),
        package_version: std::ptr::null(),
        loader: loader.map(|hook| hook as PackageHook),
        unloader: unloader.map(|hook| hook as PackageHook),
        routines: std::ptr::null_mut(),
        methods: std::ptr::null_mut(),
    };
    // SAFETY: the entry names no table and no string.
    unsafe { library_of(&entry, libloading::os::unix::Library::this().into(), "") }
        .expect("an entry asking for no version is accepted")
        .expect("an entry is a package")
}

/// A library with no tables whose package entry declares `loader` and
/// `unloader`, for a caller that needs hooks without a shared object.
#[cfg(test)]
#[cfg(unix)]
pub(crate) fn hooked_library(
    loader: Option<PackageHook>,
    unloader: Option<PackageHook>,
) -> Library {
    Library {
        name: None,
        version: None,
        methods: Vec::new(),
        routines: Vec::new(),
        loader,
        unloader,
        mapping: Mapping::new(Some(libloading::os::unix::Library::this().into())),
        thread: None,
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
    mapping: Rc<Mapping>,
}

impl NativeRoutineEntry {
    /// Whether the row carries an address.
    #[must_use]
    pub fn has_entry_point(&self) -> bool {
        !self.entry_point.is_null()
    }

    /// [`NativeMethodEntry::signature`] for a routine's stub. A row of any
    /// style but [`ROUTINE_TYPED_STYLE`] publishes none, since its address is
    /// a function of another signature.
    pub(crate) fn signature(&self, context: &CallContext<'_>, limit: usize) -> Option<Vec<u16>> {
        let stub = self.stub()?;
        // SAFETY: as `NativeMethodEntry::signature`; `stub` answers only for a
        // typed row.
        self.mapping
            .hold(|| unsafe { signature_of(stub, context.as_ptr(), limit) })
    }

    /// [`NativeMethodEntry::call`] for a routine's stub, which calls nothing
    /// for a row [`NativeRoutineEntry::signature`] publishes nothing for.
    pub(crate) fn call(
        &self,
        context: &CallContext<'_>,
        arguments: &mut [ValueDescriptor; MAX_NATIVE_ARGUMENTS],
        result: Option<ResultRead>,
    ) -> Option<Written> {
        let pointer = context.as_ptr();
        // SAFETY: as `NativeMethodEntry::call`.
        let published = unsafe { &raw mut (*pointer).arguments };
        let stub = self.stub();
        // SAFETY: as `NativeMethodEntry::call`; `stub` answers only for a
        // typed row.
        self.mapping
            .hold(|| unsafe { call_stub(stub, pointer, published, arguments, result) })
    }

    /// The row's address as the typed stub it names, or `None` for a row that
    /// carries none, for a row of any style but `ROUTINE_TYPED_STYLE`, and for
    /// a row whose library has been closed.
    ///
    /// A style that is neither classic nor typed is therefore refused as an
    /// unreadable signature, where the oracle calls any non-classic row as a
    /// typed routine (`interpreter/package/LibraryPackage.cpp:279-286`).
    fn stub(&self) -> Option<NativeRoutine> {
        if self.entry_point.is_null()
            || self.style != ROUTINE_TYPED_STYLE
            || !self.mapping.open.get()
        {
            return None;
        }
        // SAFETY: the row is `ROUTINE_TYPED_STYLE`, which `REXX_TYPED_ROUTINE`
        // fills with the stub the `RexxRoutineN` macro generated
        // (`api/oorexxapi.h:205`), whose C signature is `NativeRoutine`'s,
        // under the same D5 argument as `NativeMethodEntry::stub`.
        Some(unsafe { std::mem::transmute::<*mut c_void, NativeRoutine>(self.entry_point) })
    }
}

/// What `stub` publishes for a signature request, read as
/// [`NativeMethodEntry::signature`] describes.
///
/// # Safety
/// `stub` is a generated stub of this C signature in a library that stays
/// mapped for the call, and `context` addresses a live struct of its context
/// type.
unsafe fn signature_of<C>(stub: Stub<C>, context: *mut C, limit: usize) -> Option<Vec<u16>> {
    // SAFETY: the caller guarantees the stub and the context. A null
    // `arguments` is the signature request, which returns the static array
    // without running any of the extension's own code
    // (`api/oorexxapi.h:4342-4350`).
    let types = unsafe { stub(context, std::ptr::null_mut()) };
    if types.is_null() {
        return None;
    }
    let mut words = Vec::new();
    for offset in 0..limit {
        // SAFETY: the invariant is the macro's, not this loop's: the array
        // it emits ends in `REXX_ARGUMENT_TERMINATOR` (`api/oorexxapi.h:4338`),
        // so every offset up to the first terminator is inside it. `limit`
        // bounds how far a malformed array is followed, which the oracle does
        // not bound at all (`NativeActivation.cpp:235`).
        let word = unsafe { *types.add(offset) };
        if word == ARGUMENT_TERMINATOR {
            return Some(words);
        }
        words.push(word);
    }
    None
}

/// Call `stub`, where there is one, with `arguments` published on `context`
/// for the length of the call, and answer element zero read as `result` says.
///
/// # Safety
/// As [`signature_of`], and `published` is `context`'s own `arguments` field.
unsafe fn call_stub<C>(
    stub: Option<Stub<C>>,
    context: *mut C,
    published: *mut *mut ValueDescriptor,
    arguments: &mut [ValueDescriptor; MAX_NATIVE_ARGUMENTS],
    result: Option<ResultRead>,
) -> Option<Written> {
    // The whole word, before the stub can write a narrower member into it.
    arguments[0].value = ValueUnion { value_int64_t: 0 };
    if let Some(stub) = stub {
        let array = arguments.as_mut_ptr();
        // `argumentExists` reads the array through the context rather than
        // through the parameter (`api/oorexxapi.h:4276`), which is why the
        // oracle publishes it (`NativeActivation.cpp:1291`).
        //
        // SAFETY: the caller's context wrapper holds the only borrow of the
        // live struct for its lifetime and is neither `Clone` nor `Sync`, so
        // nothing else writes the field during this call.
        unsafe { *published = array };
        // SAFETY: the caller guarantees the stub. `array` is the caller's
        // live array, which nothing else touches for the duration of the
        // call, and the stub reads and writes only the elements its own
        // signature declares.
        unsafe { stub(context, array) };
        // SAFETY: as the write above.
        unsafe { *published = std::ptr::null_mut() };
    }
    Some(match result? {
        // SAFETY: element zero's word was written in full above, and a stub
        // writing a member into it leaves every byte initialised.
        ResultRead::Member(repr) => {
            Written::Member(unsafe { crate::ffi::value_of(&arguments[0], repr) })
        }
        ResultRead::Text => {
            // SAFETY: as the read above, for a pointer-sized member.
            let pointer = unsafe { arguments[0].value.value_CSTRING };
            if pointer.is_null() {
                Written::Text(None)
            } else {
                // SAFETY: a `CSTRING` result is a NUL-terminated string that
                // outlives the stub's return, which is the extension's side of
                // the header's contract and what `valueToObject` relies on when
                // it copies the string after the call
                // (`interpreter/execution/NativeActivation.cpp:825-833`). The
                // bytes are copied here, before anything else runs.
                let bytes = unsafe { CStr::from_ptr(pointer) }.to_bytes().to_vec();
                Written::Text(Some(bytes))
            }
        }
    })
}

/// Which of a package entry's hooks to run.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Hook {
    /// `loader`, which `LibraryPackage::loadPackage` runs once the routines
    /// have registered (`interpreter/package/LibraryPackage.cpp:237-245`).
    Loader,
    /// `unloader`, which `LibraryPackage::unload` runs at termination
    /// (`interpreter/package/LibraryPackage.cpp:166-174`).
    Unloader,
}

/// An opened shared library together with the package entry it published.
pub struct Library {
    name: Option<Vec<u8>>,
    version: Option<Vec<u8>>,
    methods: Vec<NativeMethodEntry>,
    routines: Vec<NativeRoutineEntry>,
    /// Private for the reason `entry_point` is. Never set on a library
    /// refused for its version, whose loader the oracle never reaches.
    loader: Option<PackageHook>,
    unloader: Option<PackageHook>,
    /// The mapping every address above points into. No safe code outside
    /// this module can copy an address out of a row, which is what keeps one
    /// from outliving it.
    mapping: Rc<Mapping>,
    /// Declared last, so that it is dropped after the close, whose
    /// destructors may call through a thread context the extension kept.
    thread: Option<ThreadContext>,
}

impl std::fmt::Debug for Library {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Library")
            .field("name", &self.name)
            .field("version", &self.version)
            .field("methods", &self.methods)
            .field("routines", &self.routines)
            .field("loader", &self.loader.is_some())
            .field("unloader", &self.unloader.is_some())
            .finish_non_exhaustive()
    }
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

    /// Whether the package entry declares `hook`.
    #[must_use]
    pub fn has_hook(&self, hook: Hook) -> bool {
        self.hook(hook).is_some()
    }

    fn hook(&self, hook: Hook) -> Option<PackageHook> {
        match hook {
            Hook::Loader => self.loader,
            Hook::Unloader => self.unloader,
        }
    }

    /// Runs `hook`, where the package entry declares it, handing it the
    /// thread context `contexts` links.
    pub(crate) fn run_hook(&self, hook: Hook, contexts: &Contexts<'_, '_>) {
        let Some(function) = self.hook(hook) else {
            return;
        };
        if !self.mapping.open.get() {
            return;
        }
        // SAFETY: the address is the package entry's own `loader` or
        // `unloader`, a `RexxPackageLoader`/`RexxPackageUnloader`
        // (`api/oorexxapi.h:257-258`), in a mapping that is open and that a
        // close leaves open while this call is held; the thread context is
        // the one `contexts` links, live for the call.
        self.mapping.hold(|| unsafe { function(contexts.thread()) });
    }

    /// `SysLibrary::unload`: closes the library, after which none of its rows
    /// or hooks calls in, and answers whether it closed, which it does not
    /// while a call into it is in flight.
    pub fn close(&self) -> bool {
        self.mapping.close()
    }

    /// Whether [`Library::close`] has not yet closed it.
    #[must_use]
    pub fn is_open(&self) -> bool {
        self.mapping.open.get()
    }

    /// Keeps `thread` allocated until this library has been closed.
    pub fn keep_thread_context(&mut self, thread: &ThreadContext) {
        self.thread = Some(thread.clone());
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

    /// The routine row `name` names, found as `LibraryPackage::resolveRoutine`
    /// finds it (`interpreter/package/LibraryPackage.cpp:410-435`), and
    /// borrowed from `self` for [`Library::method`]'s reason.
    ///
    /// The spelling is `name` byte for byte where a row has it, and otherwise
    /// that of the first row matching `name` without regard to case (`:420`,
    /// `:428`); of rows spelled alike, the answer is the last, which is the
    /// one `loadRoutines` leaves in its table (`:291`).
    #[must_use]
    pub fn routine(&self, name: &[u8]) -> Option<&NativeRoutineEntry> {
        let spelling = match self.routines.iter().find(|row| row.name == name) {
            Some(row) => &row.name,
            None => {
                &self
                    .routines
                    .iter()
                    .find(|row| row.name.eq_ignore_ascii_case(name))?
                    .name
            }
        };
        self.routines.iter().rev().find(|row| row.name == *spelling)
    }

    /// Each upper-cased routine name with the spelling whose row answers it
    /// through `PackageManager::packageRoutines`, in the order
    /// `LibraryPackage::loadRoutines` puts them
    /// (`interpreter/package/LibraryPackage.cpp:270-299`): a later row whose
    /// name upper-cases alike replaces an earlier one.
    #[must_use]
    pub fn package_routines(&self) -> Vec<(Vec<u8>, &[u8])> {
        let mut registered: Vec<(Vec<u8>, &[u8])> = Vec::new();
        for row in &self.routines {
            let upper = row.name.to_ascii_uppercase();
            match registered.iter_mut().find(|(name, _)| *name == upper) {
                Some(slot) => slot.1 = &row.name,
                None => registered.push((upper, &row.name)),
            }
        }
        registered
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
    // Being the library's own entry, each table pointer in it is null or the
    // array the extension declared, and every string a literal in the same
    // mapping.
    unsafe { library_of(&*entry, handle, name) }
}

/// The library `entry` describes, refused where its version check refuses.
///
/// # Safety
/// Each of `entry`'s table pointers is null or an array terminated by a
/// zero-`style` row, and every string it or a row names is null or
/// NUL-terminated; all of it outlives `handle`.
unsafe fn library_of(
    entry: &RexxPackageEntry,
    handle: libloading::Library,
    name: &str,
) -> Result<Option<Library>, Refused> {
    let refused = check_version(entry, name).err();
    let mapping = Mapping::new(Some(handle));

    // SAFETY: the caller guarantees the tables and strings.
    let (package_name, version, methods) = unsafe {
        (
            c_bytes(entry.package_name),
            c_bytes(entry.package_version),
            method_table(entry.methods, &mapping),
        )
    };
    let routines = match refused {
        Some(_) => Vec::new(),
        // SAFETY: as above.
        None => unsafe { routine_table(entry.routines, &mapping) },
    };
    let library = Library {
        name: package_name,
        version,
        methods,
        routines,
        loader: entry.loader.filter(|_| refused.is_none()),
        unloader: entry.unloader,
        mapping,
        thread: None,
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
unsafe fn method_table(
    table: *mut RexxMethodEntry,
    mapping: &Rc<Mapping>,
) -> Vec<NativeMethodEntry> {
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
            mapping: Rc::clone(mapping),
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
unsafe fn routine_table(
    table: *mut RexxRoutineEntry,
    mapping: &Rc<Mapping>,
) -> Vec<NativeRoutineEntry> {
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
            mapping: Rc::clone(mapping),
        });
        // SAFETY: `at` was not the terminator, so a further row follows it.
        at = unsafe { at.add(1) };
    }
}

#[cfg(test)]
mod tests {
    use super::{CURRENT_INTERPRETER_VERSION, Hook, Library, NativeRoutineEntry, library_of};
    use crate::layout::{RexxPackageEntry, RexxThreadContext_};

    extern "C" fn ignoring_hook(_thread: *mut RexxThreadContext_) {}

    /// A package entry with no tables asking for `required`, declaring both
    /// hooks.
    fn entry_asking_for(required: std::ffi::c_int) -> RexxPackageEntry {
        RexxPackageEntry {
            size: 0,
            api_version: 0,
            required_version: required,
            package_name: std::ptr::null(),
            package_version: std::ptr::null(),
            loader: Some(ignoring_hook),
            unloader: Some(ignoring_hook),
            routines: std::ptr::null_mut(),
            methods: std::ptr::null_mut(),
        }
    }

    /// A library refused for its version keeps its unloader and not its
    /// loader: measured, oracle, such a library's unloader runs at
    /// termination and its loader never does, since `loadPackage` raises
    /// before it (`interpreter/package/LibraryPackage.cpp:232-245`).
    #[test]
    #[cfg(unix)]
    #[cfg_attr(miri, ignore = "opens the running image")]
    fn a_library_refused_for_its_version_keeps_only_its_unloader() {
        let refused_entry = entry_asking_for(CURRENT_INTERPRETER_VERSION + 1);
        // SAFETY: the entry has no tables and no strings.
        let refused = unsafe {
            library_of(
                &refused_entry,
                libloading::os::unix::Library::this().into(),
                "forged",
            )
        }
        .expect_err("a newer version is refused");
        assert!(!refused.library.has_hook(Hook::Loader));
        assert!(refused.library.has_hook(Hook::Unloader));

        let accepted_entry = entry_asking_for(CURRENT_INTERPRETER_VERSION);
        // SAFETY: as above.
        let accepted = unsafe {
            library_of(
                &accepted_entry,
                libloading::os::unix::Library::this().into(),
                "forged",
            )
        }
        .expect("the current version is accepted")
        .expect("an entry is a package");
        assert!(accepted.has_hook(Hook::Loader));
        assert!(accepted.has_hook(Hook::Unloader));
    }

    /// `ROUTINE_TYPED_STYLE` (`api/oorexxapi.h:200`).
    const ROUTINE_TYPED_STYLE: std::ffi::c_int = 1;

    /// A routine row whose entry point is `tag`, which is never called.
    fn row(name: &[u8], tag: usize) -> NativeRoutineEntry {
        NativeRoutineEntry {
            style: ROUTINE_TYPED_STYLE,
            name: name.to_vec(),
            entry_point: std::ptr::without_provenance_mut(tag),
            mapping: super::Mapping::new(None),
        }
    }

    /// Which row `name` finds, by its tag.
    fn found(library: &Library, name: &[u8]) -> Option<usize> {
        library.routine(name).map(|row| row.entry_point.addr())
    }

    /// No extension the C++ tree builds has two routines whose names differ
    /// only in case, so the table is built here.
    #[test]
    #[cfg(unix)]
    #[cfg_attr(miri, ignore = "opens the running image")]
    fn a_routine_is_found_by_its_exact_spelling_before_its_case() {
        let library = Library {
            name: None,
            version: None,
            methods: Vec::new(),
            routines: vec![
                row(b"Foo", 1),
                row(b"FOO", 2),
                row(b"Foo", 3),
                row(b"bar", 4),
            ],
            loader: None,
            unloader: None,
            mapping: super::Mapping::new(Some(libloading::os::unix::Library::this().into())),
            thread: None,
        };
        assert_eq!(found(&library, b"FOO"), Some(2));
        assert_eq!(found(&library, b"Foo"), Some(3));
        assert_eq!(found(&library, b"foo"), Some(3));
        assert_eq!(found(&library, b"BAR"), Some(4));
        assert_eq!(found(&library, b"baz"), None);
    }

    /// The global table is keyed by the upper-cased name and the last row
    /// wins, so `FOO` answers the row spelled `Foo` that follows it, which is
    /// not the row [`Library::routine`] finds for the spelling `FOO`.
    #[test]
    #[cfg(unix)]
    #[cfg_attr(miri, ignore = "opens the running image")]
    fn the_global_table_keeps_the_last_row_of_each_upper_cased_name() {
        let library = Library {
            name: None,
            version: None,
            methods: Vec::new(),
            routines: vec![
                row(b"Foo", 1),
                row(b"FOO", 2),
                row(b"Foo", 3),
                row(b"bar", 4),
            ],
            loader: None,
            unloader: None,
            mapping: super::Mapping::new(Some(libloading::os::unix::Library::this().into())),
            thread: None,
        };
        assert_eq!(
            library.package_routines(),
            vec![
                (b"FOO".to_vec(), &b"Foo"[..]),
                (b"BAR".to_vec(), &b"bar"[..])
            ]
        );
        assert_eq!(found(&library, b"Foo"), Some(3));
    }
}
