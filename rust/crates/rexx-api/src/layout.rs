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

// The `allow` is for the frozen header's own spellings (D5): the interface
// tables, the context structs, `RexxCondition` and `ValueDescriptor` carry the
// member names `api/oorexxapi.h` gives them. The entry structs moved here from
// `load.rs` and keep the Rust spellings they arrived with.
#![allow(non_camel_case_types, non_snake_case)]

//! The `#[repr(C)]` surface an extension sees, and the tables it calls
//! through.

use std::cell::Cell;
use std::ffi::{c_char, c_int, c_void};

/// `wholenumber_t` (`api/rexx.h:237`).
pub type wholenumber_t = isize;
/// `stringsize_t` (`api/rexx.h:236`).
pub type stringsize_t = usize;
/// `logical_t` (`api/rexx.h:238`).
pub type logical_t = usize;
/// `CSTRING` (`api/rexx.h:78`).
pub type CSTRING = *const c_char;
/// `POINTER` (`api/rexx.h:79`).
pub type POINTER = *mut c_void;
/// `REXXPFN` (`api/platform/unix/rexxapitypes.h:64`).
pub type REXXPFN = *mut c_void;

/// `_RexxObjectPtr` and `RexxObjectPtr` (`api/rexx.h:117-159`).
#[repr(C)]
pub struct RexxObjectPtr_ {
    _opaque: [u8; 0],
}
/// A handle to an object of that shape; opaque to an extension.
pub type RexxObjectPtr = *mut RexxObjectPtr_;

/// `_RexxStringObject` and `RexxStringObject` (`api/rexx.h:117-159`).
#[repr(C)]
pub struct RexxStringObject_ {
    _opaque: [u8; 0],
}
/// A handle to an object of that shape; opaque to an extension.
pub type RexxStringObject = *mut RexxStringObject_;

/// `_RexxBufferStringObject` and `RexxBufferStringObject` (`api/rexx.h:117-159`).
#[repr(C)]
pub struct RexxBufferStringObject_ {
    _opaque: [u8; 0],
}
/// A handle to an object of that shape; opaque to an extension.
pub type RexxBufferStringObject = *mut RexxBufferStringObject_;

/// `_RexxArrayObject` and `RexxArrayObject` (`api/rexx.h:117-159`).
#[repr(C)]
pub struct RexxArrayObject_ {
    _opaque: [u8; 0],
}
/// A handle to an object of that shape; opaque to an extension.
pub type RexxArrayObject = *mut RexxArrayObject_;

/// `_RexxBufferObject` and `RexxBufferObject` (`api/rexx.h:117-159`).
#[repr(C)]
pub struct RexxBufferObject_ {
    _opaque: [u8; 0],
}
/// A handle to an object of that shape; opaque to an extension.
pub type RexxBufferObject = *mut RexxBufferObject_;

/// `_RexxPointerObject` and `RexxPointerObject` (`api/rexx.h:117-159`).
#[repr(C)]
pub struct RexxPointerObject_ {
    _opaque: [u8; 0],
}
/// A handle to an object of that shape; opaque to an extension.
pub type RexxPointerObject = *mut RexxPointerObject_;

/// `_RexxMethodObject` and `RexxMethodObject` (`api/rexx.h:117-159`).
#[repr(C)]
pub struct RexxMethodObject_ {
    _opaque: [u8; 0],
}
/// A handle to an object of that shape; opaque to an extension.
pub type RexxMethodObject = *mut RexxMethodObject_;

/// `_RexxRoutineObject` and `RexxRoutineObject` (`api/rexx.h:117-159`).
#[repr(C)]
pub struct RexxRoutineObject_ {
    _opaque: [u8; 0],
}
/// A handle to an object of that shape; opaque to an extension.
pub type RexxRoutineObject = *mut RexxRoutineObject_;

/// `_RexxPackageObject` and `RexxPackageObject` (`api/rexx.h:117-159`).
#[repr(C)]
pub struct RexxPackageObject_ {
    _opaque: [u8; 0],
}
/// A handle to an object of that shape; opaque to an extension.
pub type RexxPackageObject = *mut RexxPackageObject_;

/// `_RexxClassObject` and `RexxClassObject` (`api/rexx.h:117-159`).
#[repr(C)]
pub struct RexxClassObject_ {
    _opaque: [u8; 0],
}
/// A handle to an object of that shape; opaque to an extension.
pub type RexxClassObject = *mut RexxClassObject_;

/// `_RexxDirectoryObject` and `RexxDirectoryObject` (`api/rexx.h:117-159`).
#[repr(C)]
pub struct RexxDirectoryObject_ {
    _opaque: [u8; 0],
}
/// A handle to an object of that shape; opaque to an extension.
pub type RexxDirectoryObject = *mut RexxDirectoryObject_;

/// `_RexxStringTableObject` and `RexxStringTableObject` (`api/rexx.h:117-159`).
#[repr(C)]
pub struct RexxStringTableObject_ {
    _opaque: [u8; 0],
}
/// A handle to an object of that shape; opaque to an extension.
pub type RexxStringTableObject = *mut RexxStringTableObject_;

/// `_RexxSupplierObject` and `RexxSupplierObject` (`api/rexx.h:117-159`).
#[repr(C)]
pub struct RexxSupplierObject_ {
    _opaque: [u8; 0],
}
/// A handle to an object of that shape; opaque to an extension.
pub type RexxSupplierObject = *mut RexxSupplierObject_;

/// `_RexxStemObject` and `RexxStemObject` (`api/rexx.h:117-159`).
#[repr(C)]
pub struct RexxStemObject_ {
    _opaque: [u8; 0],
}
/// A handle to an object of that shape; opaque to an extension.
pub type RexxStemObject = *mut RexxStemObject_;

/// `_RexxMutableBufferObject` and `RexxMutableBufferObject` (`api/rexx.h:117-159`).
#[repr(C)]
pub struct RexxMutableBufferObject_ {
    _opaque: [u8; 0],
}
/// A handle to an object of that shape; opaque to an extension.
pub type RexxMutableBufferObject = *mut RexxMutableBufferObject_;

/// `_RexxVariableReferenceObject` and `RexxVariableReferenceObject` (`api/rexx.h:117-159`).
#[repr(C)]
pub struct RexxVariableReferenceObject_ {
    _opaque: [u8; 0],
}
/// A handle to an object of that shape; opaque to an extension.
pub type RexxVariableReferenceObject = *mut RexxVariableReferenceObject_;

/// `INSTANCE_INTERFACE_VERSION` (`api/oorexxapi.h:477`).
pub const INSTANCE_INTERFACE_VERSION: wholenumber_t = 101;
/// `THREAD_INTERFACE_VERSION` (`api/oorexxapi.h:499`).
pub const THREAD_INTERFACE_VERSION: wholenumber_t = 103;
/// `METHOD_INTERFACE_VERSION` (`api/oorexxapi.h:680`).
pub const METHOD_INTERFACE_VERSION: wholenumber_t = 102;
/// `CALL_INTERFACE_VERSION` (`api/oorexxapi.h:716`).
pub const CALL_INTERFACE_VERSION: wholenumber_t = 101;
/// `EXIT_INTERFACE_VERSION` (`api/oorexxapi.h:747`).
pub const EXIT_INTERFACE_VERSION: wholenumber_t = 101;
/// `REDIRECT_INTERFACE_VERSION` (`api/oorexxapi.h:768`).
pub const REDIRECT_INTERFACE_VERSION: wholenumber_t = 100;

/// `RexxCondition` (`api/oorexxapi.h:462-473`).
#[repr(C)]
pub struct RexxCondition {
    pub code: wholenumber_t,
    pub rc: wholenumber_t,
    pub position: usize,
    pub conditionName: RexxStringObject,
    pub message: RexxStringObject,
    pub errortext: RexxStringObject,
    pub program: RexxStringObject,
    pub description: RexxStringObject,
    pub additional: RexxArrayObject,
}

/// The anonymous union inside `ValueDescriptor` (`api/oorexxapi.h:287-364`).
///
/// Every member is a view of the same word, so which one is live is what
/// `ValueDescriptor::type` records.
#[repr(C)]
pub union ValueUnion {
    pub value_ARGLIST: RexxArrayObject,
    pub value_NAME: CSTRING,
    pub value_SCOPE: RexxObjectPtr,
    pub value_CSELF: POINTER,
    pub value_OSELF: RexxClassObject,
    pub value_SUPER: RexxClassObject,
    pub value_RexxObjectPtr: RexxObjectPtr,
    pub value_RexxClassObject: RexxClassObject,
    pub value_int: c_int,
    pub value_wholenumber_t: wholenumber_t,
    pub value_stringsize_t: stringsize_t,
    pub value_logical_t: logical_t,
    pub value_double: f64,
    pub value_CSTRING: CSTRING,
    pub value_POINTER: POINTER,
    pub value_RexxStringObject: RexxStringObject,
    pub value_float: f32,
    pub value_int8_t: i8,
    pub value_int16_t: i16,
    pub value_int32_t: i32,
    pub value___int32_t: i32,
    pub value_int64_t: i64,
    pub value___int64_t: i64,
    pub value_uint8_t: u8,
    pub value_uint16_t: u16,
    pub value_uint32_t: u32,
    pub value___uint32_t: u32,
    pub value_uint64_t: u64,
    pub value___uint64_t: u64,
    pub value_intptr_t: isize,
    pub value_uintptr_t: usize,
    pub value___uintptr_t: usize,
    pub value_size_t: usize,
    pub value_ssize_t: isize,
    pub value_RexxArrayObject: RexxArrayObject,
    pub value_RexxStemObject: RexxStemObject,
    pub value_POINTERSTRING: POINTER,
    pub value_RexxMutableBufferObject: RexxMutableBufferObject,
    pub value_RexxVariableReferenceObject: RexxVariableReferenceObject,
    pub value_positive_wholenumber_t: wholenumber_t,
    pub value_nonnegative_wholenumber_t: wholenumber_t,
    pub value_OPTIONAL_RexxObjectPtr: RexxObjectPtr,
    pub value_OPTIONAL_int: c_int,
    pub value_OPTIONAL_wholenumber_t: wholenumber_t,
    pub value_OPTIONAL_stringsize_t: stringsize_t,
    pub value_OPTIONAL_logical_t: logical_t,
    pub value_OPTIONAL_double: f64,
    pub value_OPTIONAL_CSTRING: CSTRING,
    pub value_OPTIONAL_RexxClassObject: RexxClassObject,
    pub value_OPTIONAL_POINTER: POINTER,
    pub value_OPTIONAL_RexxStringObject: RexxStringObject,
    pub value_OPTIONAL_float: f32,
    pub value_OPTIONAL_int8_t: i8,
    pub value_OPTIONAL_int16_t: i16,
    pub value_OPTIONAL_int32_t: i32,
    pub value_OPTIONAL_int64_t: i64,
    pub value_OPTIONAL_uint8_t: u8,
    pub value_OPTIONAL_uint16_t: u16,
    pub value_OPTIONAL_uint32_t: u32,
    pub value_OPTIONAL_uint64_t: u64,
    pub value_OPTIONAL_intptr_t: isize,
    pub value_OPTIONAL_uintptr_t: usize,
    pub value_OPTIONAL_ssize_t: isize,
    pub value_OPTIONAL_size_t: usize,
    pub value_OPTIONAL_RexxArrayObject: RexxArrayObject,
    pub value_OPTIONAL_RexxStemObject: RexxStemObject,
    pub value_OPTIONAL_POINTERSTRING: POINTER,
    pub value_OPTIONAL_RexxMutableBufferObject: RexxMutableBufferObject,
    pub value_OPTIONAL_positive_wholenumber_t: wholenumber_t,
    pub value_OPTIONAL_nonnegative_wholenumber_t: wholenumber_t,
}

/// `ValueDescriptor` (`api/oorexxapi.h:281-394`).
#[repr(C)]
pub struct ValueDescriptor {
    pub value: ValueUnion,
    pub r#type: u16,
    pub flags: u16,
}

/// `RexxPackageLoader` and `RexxPackageUnloader` (`api/oorexxapi.h:257-258`).
pub type PackageHook = unsafe extern "C" fn(*mut RexxThreadContext_);

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

/// `RexxInstance_` (`api/oorexxapi.h:788-791`).
#[repr(C)]
pub struct RexxInstance_ {
    pub functions: *mut RexxInstanceInterface,
    pub applicationData: *mut c_void,
}

/// `RexxThreadContext_` (`api/oorexxapi.h:825-828`).
#[repr(C)]
pub struct RexxThreadContext_ {
    pub instance: *mut RexxInstance_,
    pub functions: *mut RexxThreadInterface,
}

/// `RexxMethodContext_` (`api/oorexxapi.h:1666-1670`).
#[repr(C)]
pub struct RexxMethodContext_ {
    pub threadContext: *mut RexxThreadContext_,
    pub functions: *mut MethodContextInterface,
    pub arguments: *mut ValueDescriptor,
}

/// `RexxCallContext_` (`api/oorexxapi.h:2518-2522`).
#[repr(C)]
pub struct RexxCallContext_ {
    pub threadContext: *mut RexxThreadContext_,
    pub functions: *mut CallContextInterface,
    pub arguments: *mut ValueDescriptor,
}

/// `RexxExitContext_` (`api/oorexxapi.h:3349-3353`).
#[repr(C)]
pub struct RexxExitContext_ {
    pub threadContext: *mut RexxThreadContext_,
    pub functions: *mut ExitContextInterface,
    pub arguments: *mut ValueDescriptor,
}

/// `RexxIORedirectorContext_` (`api/oorexxapi.h:4138-4140`).
#[repr(C)]
pub struct RexxIORedirectorContext_ {
    pub functions: *mut IORedirectorInterface,
}

/// A context we handed out, together with the state that owns it.
///
/// `C` is first by value, so the address of the wrapper is the address of the
/// public struct and `ffi::owner_of` recovers `owner` by casting back
/// (`interpreter/concurrency/ActivationApiContexts.hpp:58-95`,
/// `interpreter/concurrency/Activity.hpp:503`).
#[repr(C)]
pub struct Owned<C, T> {
    pub context: C,
    pub owner: *mut T,
}

/// Each member a populated table still refuses, as `Table.Member`, with the
/// phase its refusal names.
pub const REFUSING_MEMBERS: &[(&str, &str)] = &[
    ("RexxInstanceInterface.Terminate", "Phase 9"),
    ("RexxInstanceInterface.Halt", "Phase 9"),
    ("RexxInstanceInterface.SetTrace", "Phase 9"),
    ("RexxInstanceInterface.AddCommandEnvironment", "Phase 8"),
    ("RexxThreadInterface.HaltThread", "Phase 9"),
    ("RexxThreadInterface.SetThreadTrace", "Phase 9"),
    ("MethodContextInterface.SetGuardOnWhenUpdated", "Phase 6"),
    ("MethodContextInterface.SetGuardOffWhenUpdated", "Phase 6"),
    ("MethodContextInterface.ThrowException0", "Phase 8"),
    ("MethodContextInterface.ThrowException1", "Phase 8"),
    ("MethodContextInterface.ThrowException2", "Phase 8"),
    ("MethodContextInterface.ThrowException", "Phase 8"),
    ("MethodContextInterface.ThrowCondition", "Phase 8"),
    ("CallContextInterface.ThrowException0", "Phase 8"),
    ("CallContextInterface.ThrowException1", "Phase 8"),
    ("CallContextInterface.ThrowException2", "Phase 8"),
    ("CallContextInterface.ThrowException", "Phase 8"),
    ("CallContextInterface.ThrowCondition", "Phase 8"),
];

/// The phase a refusal of `entry` names: its [`REFUSING_MEMBERS`] row, and
/// Phase 8 for a member of a table no task has populated.
#[must_use]
pub fn refusal_owner(entry: &str) -> &'static str {
    REFUSING_MEMBERS
        .iter()
        .find(|(member, _)| *member == entry)
        .map_or("Phase 8", |(_, owner)| owner)
}

/// Abandon the process, naming the entry the caller reached.
///
/// # Panics
/// Always. The stubs below call it, and each is an `extern "C"` frame, so the
/// panic aborts rather than unwinding into the extension's stack.
fn abort(entry: &str) -> ! {
    panic!("{entry} is not implemented ({})", refusal_owner(entry));
}

thread_local! {
    /// The first refusing slot the running native call or package hook
    /// reached.
    ///
    /// Set by a refusing stub and taken by [`recording_refusals`] around the
    /// one call it records, on the thread making that call, so it holds no
    /// state beyond that call: a nested call saves the outer record and puts it
    /// back. Every place extension code runs reads the record and refuses
    /// loudly on it: `invoke::run` for a method or routine, `invoke::hook` for
    /// a loader or unloader.
    static REFUSED: Cell<Option<&'static str>> = const { Cell::new(None) };
}

/// Records that the running call reached `entry`, keeping an earlier record.
pub(crate) fn refuse(entry: &'static str) {
    if REFUSED.get().is_none() {
        REFUSED.set(Some(entry));
    }
}

/// Runs `call` over a fresh record and answers what it answered with the first
/// refusing slot it reached, restoring the record in force before.
pub(crate) fn recording_refusals<R>(call: impl FnOnce() -> R) -> (R, Option<&'static str>) {
    let outer = REFUSED.replace(None);
    let answered = call();
    let refused = REFUSED.replace(outer);
    (answered, refused)
}

/// What a refusing stub answers after recording itself: the value the
/// oracle's own stubs answer on their failure paths, a null handle
/// (`interpreter/api/ThreadContextStubs.cpp:986-997`), a zero or `false`, with
/// any other value named at its member.
trait RefusedValue {
    const REFUSED: Self;
}

impl<T> RefusedValue for *mut T {
    const REFUSED: Self = std::ptr::null_mut();
}

impl RefusedValue for usize {
    const REFUSED: Self = 0;
}

impl RefusedValue for isize {
    const REFUSED: Self = 0;
}

/// The Rust type of one interface member.
///
/// A function member is `unsafe` to call, because it trusts raw pointers that
/// only its caller can vouch for.
macro_rules! entry_type {
    (value $ty:ty = $value:expr) => {
        $ty
    };
    ($(aborts)? call($($arg:ty),*)) => {
        unsafe extern "C" fn($($arg),*)
    };
    ($(aborts)? call($($arg:ty),*) -> $ret:ty $(, failing $value:expr)?) => {
        unsafe extern "C" fn($($arg),*) -> $ret
    };
}

/// The value one interface member holds in a table nothing has built yet.
///
/// A function member records itself and returns, unless it is marked
/// `aborts`: those answer a data pointer the extension dereferences, or leave
/// the extension by a C++ throw, so no value they could return is safe.
macro_rules! entry_stub {
    ($entry:expr, value $ty:ty = $value:expr) => {
        $value
    };
    ($entry:expr, call($($arg:ty),*)) => {{
        extern "C" fn stub($(_: $arg),*) {
            refuse($entry)
        }
        stub
    }};
    ($entry:expr, call($($arg:ty),*) -> $ret:ty) => {{
        extern "C" fn stub($(_: $arg),*) -> $ret {
            refuse($entry);
            <$ret as RefusedValue>::REFUSED
        }
        stub
    }};
    ($entry:expr, call($($arg:ty),*) -> $ret:ty, failing $value:expr) => {{
        extern "C" fn stub($(_: $arg),*) -> $ret {
            refuse($entry);
            $value
        }
        stub
    }};
    ($entry:expr, aborts call($($arg:ty),*)) => {{
        extern "C" fn stub($(_: $arg),*) {
            abort($entry)
        }
        stub
    }};
    ($entry:expr, aborts call($($arg:ty),*) -> $ret:ty) => {{
        extern "C" fn stub($(_: $arg),*) -> $ret {
            abort($entry)
        }
        stub
    }};
}

/// A function member's address, `None` for a data member.
macro_rules! entry_address {
    ($member:expr, value $ty:ty = $value:expr) => {
        None
    };
    ($member:expr, $($kind:tt)*) => {
        Some($member as usize)
    };
}

/// Whether one interface member is marked `aborts`.
macro_rules! entry_aborts {
    (aborts $($rest:tt)*) => {
        true
    };
    ($($rest:tt)*) => {
        false
    };
}

/// One interface table, and the member names a test measures it against.
///
/// `populated` additionally builds `REFUSING`, in which no member is a null
/// pointer: a function member's type is not nullable, and the literal must
/// name every field.
macro_rules! interface {
    (@shape $(#[$meta:meta])* $Name:ident { $($field:ident : { $($kind:tt)* }),* $(,)? }) => {
        $(#[$meta])*
        #[repr(C)]
        pub struct $Name {
            $( pub $field: entry_type!($($kind)*), )*
        }

        impl $Name {
            /// The member names the frozen header declares, in order.
            pub const FIELDS: &'static [&'static str] = &[$(stringify!($field)),*];

            /// For each of [`Self::FIELDS`], whether its refusing stub aborts
            /// rather than records and returns.
            pub const ABORTS: &'static [bool] = &[$(entry_aborts!($($kind)*)),*];

            /// For each of [`Self::FIELDS`], the Rust type of the member.
            #[must_use]
            pub fn member_types() -> Vec<&'static str> {
                vec![$(std::any::type_name::<entry_type!($($kind)*)>()),*]
            }

            /// For each function member, its name and the address it holds.
            #[must_use]
            pub fn addresses(&self) -> Vec<(&'static str, usize)> {
                [$( (stringify!($field), entry_address!(self.$field, $($kind)*)) ),*]
                    .into_iter()
                    .filter_map(|(name, address)| Some((name, address?)))
                    .collect()
            }
        }
    };
    (@refusing $Name:ident { $($field:ident : { $($kind:tt)* }),* $(,)? }) => {
        impl $Name {
            /// Every function member refusing loudly, for a table whose
            /// bodies later tasks write.
            ///
            /// The object members are null until an interpreter instance
            /// fills them, which is not this task's.
            pub const REFUSING: Self = Self {
                $( $field: entry_stub!(
                    concat!(stringify!($Name), ".", stringify!($field)),
                    $($kind)*
                ), )*
            };
        }
    };
    ($(#[$meta:meta])* $Name:ident, populated { $($body:tt)* }) => {
        interface!(@shape $(#[$meta])* $Name { $($body)* });
        interface!(@refusing $Name { $($body)* });
    };
    ($(#[$meta:meta])* $Name:ident, unpopulated { $($body:tt)* }) => {
        interface!(@shape $(#[$meta])* $Name { $($body)* });
    };
}

interface! {
    /// `RexxInstanceInterface` (`api/oorexxapi.h:482-493`).
    RexxInstanceInterface, populated {
        interfaceVersion: { value wholenumber_t = INSTANCE_INTERFACE_VERSION },
        Terminate: { call(*mut RexxInstance_) },
        AttachThread: { call(*mut RexxInstance_, *mut *mut RexxThreadContext_) -> logical_t },
        InterpreterVersion: { call(*mut RexxInstance_) -> usize },
        LanguageLevel: { call(*mut RexxInstance_) -> usize },
        Halt: { call(*mut RexxInstance_) },
        SetTrace: { call(*mut RexxInstance_, logical_t) },
        AddCommandEnvironment: { call(*mut RexxInstance_, *const c_char, REXXPFN, c_int) },
    }
}

interface! {
    /// `RexxThreadInterface` (`api/oorexxapi.h:503-674`).
    RexxThreadInterface, populated {
        interfaceVersion: { value wholenumber_t = THREAD_INTERFACE_VERSION },
        DetachThread: { call(*mut RexxThreadContext_) },
        HaltThread: { call(*mut RexxThreadContext_) },
        SetThreadTrace: { call(*mut RexxThreadContext_, logical_t) },
        RequestGlobalReference: { call(*mut RexxThreadContext_, RexxObjectPtr) -> RexxObjectPtr },
        ReleaseGlobalReference: { call(*mut RexxThreadContext_, RexxObjectPtr) },
        ReleaseLocalReference: { call(*mut RexxThreadContext_, RexxObjectPtr) },
        SendMessage: { call(*mut RexxThreadContext_, RexxObjectPtr, CSTRING, RexxArrayObject) -> RexxObjectPtr },
        SendMessage0: { call(*mut RexxThreadContext_, RexxObjectPtr, CSTRING) -> RexxObjectPtr },
        SendMessage1: { call(*mut RexxThreadContext_, RexxObjectPtr, CSTRING, RexxObjectPtr) -> RexxObjectPtr },
        SendMessage2: { call(*mut RexxThreadContext_, RexxObjectPtr, CSTRING, RexxObjectPtr, RexxObjectPtr) -> RexxObjectPtr },
        GetLocalEnvironment: { call(*mut RexxThreadContext_) -> RexxDirectoryObject },
        GetGlobalEnvironment: { call(*mut RexxThreadContext_) -> RexxDirectoryObject },
        IsInstanceOf: { call(*mut RexxThreadContext_, RexxObjectPtr, RexxClassObject) -> logical_t },
        IsOfType: { call(*mut RexxThreadContext_, RexxObjectPtr, CSTRING) -> logical_t },
        HasMethod: { call(*mut RexxThreadContext_, RexxObjectPtr, CSTRING) -> logical_t },
        LoadPackage: { call(*mut RexxThreadContext_, CSTRING) -> RexxPackageObject },
        LoadPackageFromData: { call(*mut RexxThreadContext_, CSTRING, CSTRING, usize) -> RexxPackageObject },
        LoadLibrary: { call(*mut RexxThreadContext_, CSTRING) -> logical_t },
        RegisterLibrary: { call(*mut RexxThreadContext_, CSTRING, *mut RexxPackageEntry) -> logical_t },
        FindClass: { call(*mut RexxThreadContext_, CSTRING) -> RexxClassObject },
        FindPackageClass: { call(*mut RexxThreadContext_, RexxPackageObject, CSTRING) -> RexxClassObject },
        GetPackageRoutines: { call(*mut RexxThreadContext_, RexxPackageObject) -> RexxDirectoryObject },
        GetPackagePublicRoutines: { call(*mut RexxThreadContext_, RexxPackageObject) -> RexxDirectoryObject },
        GetPackageClasses: { call(*mut RexxThreadContext_, RexxPackageObject) -> RexxDirectoryObject },
        GetPackagePublicClasses: { call(*mut RexxThreadContext_, RexxPackageObject) -> RexxDirectoryObject },
        GetPackageMethods: { call(*mut RexxThreadContext_, RexxPackageObject) -> RexxDirectoryObject },
        CallRoutine: { call(*mut RexxThreadContext_, RexxRoutineObject, RexxArrayObject) -> RexxObjectPtr },
        CallProgram: { call(*mut RexxThreadContext_, CSTRING, RexxArrayObject) -> RexxObjectPtr },
        NewMethod: { call(*mut RexxThreadContext_, CSTRING, CSTRING, usize) -> RexxMethodObject },
        NewRoutine: { call(*mut RexxThreadContext_, CSTRING, CSTRING, usize) -> RexxRoutineObject },
        IsRoutine: { call(*mut RexxThreadContext_, RexxObjectPtr) -> logical_t },
        IsMethod: { call(*mut RexxThreadContext_, RexxObjectPtr) -> logical_t },
        GetRoutinePackage: { call(*mut RexxThreadContext_, RexxRoutineObject) -> RexxPackageObject },
        GetMethodPackage: { call(*mut RexxThreadContext_, RexxMethodObject) -> RexxPackageObject },
        ObjectToCSelf: { aborts call(*mut RexxThreadContext_, RexxObjectPtr) -> POINTER },
        WholeNumberToObject: { call(*mut RexxThreadContext_, wholenumber_t) -> RexxObjectPtr },
        UintptrToObject: { call(*mut RexxThreadContext_, usize) -> RexxObjectPtr },
        IntptrToObject: { call(*mut RexxThreadContext_, isize) -> RexxObjectPtr },
        ValueToObject: { call(*mut RexxThreadContext_, *mut ValueDescriptor) -> RexxObjectPtr },
        ValuesToObject: { call(*mut RexxThreadContext_, *mut ValueDescriptor, usize) -> RexxArrayObject },
        ObjectToValue: { call(*mut RexxThreadContext_, RexxObjectPtr, *mut ValueDescriptor) -> logical_t },
        StringSizeToObject: { call(*mut RexxThreadContext_, stringsize_t) -> RexxObjectPtr },
        ObjectToWholeNumber: { call(*mut RexxThreadContext_, RexxObjectPtr, *mut wholenumber_t) -> logical_t },
        ObjectToStringSize: { call(*mut RexxThreadContext_, RexxObjectPtr, *mut stringsize_t) -> logical_t },
        Int64ToObject: { call(*mut RexxThreadContext_, i64) -> RexxObjectPtr },
        UnsignedInt64ToObject: { call(*mut RexxThreadContext_, u64) -> RexxObjectPtr },
        ObjectToInt64: { call(*mut RexxThreadContext_, RexxObjectPtr, *mut i64) -> logical_t },
        ObjectToUnsignedInt64: { call(*mut RexxThreadContext_, RexxObjectPtr, *mut u64) -> logical_t },
        Int32ToObject: { call(*mut RexxThreadContext_, i32) -> RexxObjectPtr },
        UnsignedInt32ToObject: { call(*mut RexxThreadContext_, u32) -> RexxObjectPtr },
        ObjectToInt32: { call(*mut RexxThreadContext_, RexxObjectPtr, *mut i32) -> logical_t },
        ObjectToUnsignedInt32: { call(*mut RexxThreadContext_, RexxObjectPtr, *mut u32) -> logical_t },
        ObjectToUintptr: { call(*mut RexxThreadContext_, RexxObjectPtr, *mut usize) -> logical_t },
        ObjectToIntptr: { call(*mut RexxThreadContext_, RexxObjectPtr, *mut isize) -> logical_t },
        ObjectToLogical: { call(*mut RexxThreadContext_, RexxObjectPtr, *mut logical_t) -> logical_t },
        LogicalToObject: { call(*mut RexxThreadContext_, logical_t) -> RexxObjectPtr },
        DoubleToObject: { call(*mut RexxThreadContext_, f64) -> RexxObjectPtr },
        DoubleToObjectWithPrecision: { call(*mut RexxThreadContext_, f64, usize) -> RexxObjectPtr },
        ObjectToDouble: { call(*mut RexxThreadContext_, RexxObjectPtr, *mut f64) -> logical_t },
        ObjectToString: { call(*mut RexxThreadContext_, RexxObjectPtr) -> RexxStringObject },
        ObjectToStringValue: { aborts call(*mut RexxThreadContext_, RexxObjectPtr) -> CSTRING },
        StringGet: { call(*mut RexxThreadContext_, RexxStringObject, usize, POINTER, usize) -> usize },
        StringLength: { call(*mut RexxThreadContext_, RexxStringObject) -> usize },
        StringData: { aborts call(*mut RexxThreadContext_, RexxStringObject) -> CSTRING },
        NewString: { call(*mut RexxThreadContext_, CSTRING, usize) -> RexxStringObject },
        NewStringFromAsciiz: { call(*mut RexxThreadContext_, CSTRING) -> RexxStringObject },
        StringUpper: { call(*mut RexxThreadContext_, RexxStringObject) -> RexxStringObject },
        StringLower: { call(*mut RexxThreadContext_, RexxStringObject) -> RexxStringObject },
        IsString: { call(*mut RexxThreadContext_, RexxObjectPtr) -> logical_t },
        NewBufferString: { call(*mut RexxThreadContext_, usize) -> RexxBufferStringObject },
        BufferStringLength: { call(*mut RexxThreadContext_, RexxBufferStringObject) -> usize },
        BufferStringData: { aborts call(*mut RexxThreadContext_, RexxBufferStringObject) -> POINTER },
        FinishBufferString: { call(*mut RexxThreadContext_, RexxBufferStringObject, usize) -> RexxStringObject },
        DirectoryPut: { call(*mut RexxThreadContext_, RexxDirectoryObject, RexxObjectPtr, CSTRING) },
        DirectoryAt: { call(*mut RexxThreadContext_, RexxDirectoryObject, CSTRING) -> RexxObjectPtr },
        DirectoryRemove: { call(*mut RexxThreadContext_, RexxDirectoryObject, CSTRING) -> RexxObjectPtr },
        NewDirectory: { call(*mut RexxThreadContext_) -> RexxDirectoryObject },
        IsDirectory: { call(*mut RexxThreadContext_, RexxObjectPtr) -> logical_t },
        ArrayAt: { call(*mut RexxThreadContext_, RexxArrayObject, usize) -> RexxObjectPtr },
        ArrayPut: { call(*mut RexxThreadContext_, RexxArrayObject, RexxObjectPtr, usize) },
        ArrayAppend: { call(*mut RexxThreadContext_, RexxArrayObject, RexxObjectPtr) -> usize },
        ArrayAppendString: { call(*mut RexxThreadContext_, RexxArrayObject, CSTRING, usize) -> usize },
        ArraySize: { call(*mut RexxThreadContext_, RexxArrayObject) -> usize },
        ArrayItems: { call(*mut RexxThreadContext_, RexxArrayObject) -> usize },
        ArrayDimension: { call(*mut RexxThreadContext_, RexxArrayObject) -> usize },
        NewArray: { call(*mut RexxThreadContext_, usize) -> RexxArrayObject },
        ArrayOfOne: { call(*mut RexxThreadContext_, RexxObjectPtr) -> RexxArrayObject },
        ArrayOfTwo: { call(*mut RexxThreadContext_, RexxObjectPtr, RexxObjectPtr) -> RexxArrayObject },
        ArrayOfThree: { call(*mut RexxThreadContext_, RexxObjectPtr, RexxObjectPtr, RexxObjectPtr) -> RexxArrayObject },
        ArrayOfFour: { call(*mut RexxThreadContext_, RexxObjectPtr, RexxObjectPtr, RexxObjectPtr, RexxObjectPtr) -> RexxArrayObject },
        IsArray: { call(*mut RexxThreadContext_, RexxObjectPtr) -> logical_t },
        BufferData: { aborts call(*mut RexxThreadContext_, RexxBufferObject) -> POINTER },
        BufferLength: { call(*mut RexxThreadContext_, RexxBufferObject) -> usize },
        NewBuffer: { call(*mut RexxThreadContext_, usize) -> RexxBufferObject },
        IsBuffer: { call(*mut RexxThreadContext_, RexxObjectPtr) -> logical_t },
        PointerValue: { aborts call(*mut RexxThreadContext_, RexxPointerObject) -> POINTER },
        NewPointer: { call(*mut RexxThreadContext_, POINTER) -> RexxPointerObject },
        IsPointer: { call(*mut RexxThreadContext_, RexxObjectPtr) -> logical_t },
        SupplierItem: { call(*mut RexxThreadContext_, RexxSupplierObject) -> RexxObjectPtr },
        SupplierIndex: { call(*mut RexxThreadContext_, RexxSupplierObject) -> RexxObjectPtr },
        SupplierAvailable: { call(*mut RexxThreadContext_, RexxSupplierObject) -> logical_t },
        SupplierNext: { call(*mut RexxThreadContext_, RexxSupplierObject) },
        NewSupplier: { call(*mut RexxThreadContext_, RexxArrayObject, RexxArrayObject) -> RexxSupplierObject },
        NewStem: { call(*mut RexxThreadContext_, CSTRING) -> RexxStemObject },
        SetStemElement: { call(*mut RexxThreadContext_, RexxStemObject, CSTRING, RexxObjectPtr) },
        GetStemElement: { call(*mut RexxThreadContext_, RexxStemObject, CSTRING) -> RexxObjectPtr },
        DropStemElement: { call(*mut RexxThreadContext_, RexxStemObject, CSTRING) },
        SetStemArrayElement: { call(*mut RexxThreadContext_, RexxStemObject, usize, RexxObjectPtr) },
        GetStemArrayElement: { call(*mut RexxThreadContext_, RexxStemObject, usize) -> RexxObjectPtr },
        DropStemArrayElement: { call(*mut RexxThreadContext_, RexxStemObject, usize) },
        GetAllStemElements: { call(*mut RexxThreadContext_, RexxStemObject) -> RexxDirectoryObject },
        GetStemValue: { call(*mut RexxThreadContext_, RexxStemObject) -> RexxObjectPtr },
        IsStem: { call(*mut RexxThreadContext_, RexxObjectPtr) -> logical_t },
        RaiseException0: { call(*mut RexxThreadContext_, usize) },
        RaiseException1: { call(*mut RexxThreadContext_, usize, RexxObjectPtr) },
        RaiseException2: { call(*mut RexxThreadContext_, usize, RexxObjectPtr, RexxObjectPtr) },
        RaiseException: { call(*mut RexxThreadContext_, usize, RexxArrayObject) },
        RaiseCondition: { call(*mut RexxThreadContext_, CSTRING, RexxStringObject, RexxObjectPtr, RexxObjectPtr) },
        CheckCondition: { call(*mut RexxThreadContext_) -> logical_t },
        GetConditionInfo: { call(*mut RexxThreadContext_) -> RexxDirectoryObject },
        DecodeConditionInfo: { call(*mut RexxThreadContext_, RexxDirectoryObject, *mut RexxCondition) },
        ClearCondition: { call(*mut RexxThreadContext_) },
        RexxNil: { value RexxObjectPtr = std::ptr::null_mut() },
        RexxTrue: { value RexxObjectPtr = std::ptr::null_mut() },
        RexxFalse: { value RexxObjectPtr = std::ptr::null_mut() },
        RexxNullString: { value RexxStringObject = std::ptr::null_mut() },
        ObjectToCSelfScoped: { aborts call(*mut RexxThreadContext_, RexxObjectPtr, RexxObjectPtr) -> POINTER },
        // `Error_Interpretation/1000` (`interpreter/api/ThreadContextStubs.cpp:1948`), where
        // `Error_Interpretation` is 49000 (`interpreter/messages/RexxErrorCodes.h:456`).
        DisplayCondition: { call(*mut RexxThreadContext_) -> wholenumber_t, failing 49 },
        MutableBufferData: { aborts call(*mut RexxThreadContext_, RexxMutableBufferObject) -> POINTER },
        MutableBufferLength: { call(*mut RexxThreadContext_, RexxMutableBufferObject) -> usize },
        SetMutableBufferLength: { call(*mut RexxThreadContext_, RexxMutableBufferObject, usize) -> usize },
        NewMutableBuffer: { call(*mut RexxThreadContext_, usize) -> RexxMutableBufferObject },
        IsMutableBuffer: { call(*mut RexxThreadContext_, RexxObjectPtr) -> logical_t },
        MutableBufferCapacity: { call(*mut RexxThreadContext_, RexxMutableBufferObject) -> usize },
        SetMutableBufferCapacity: { aborts call(*mut RexxThreadContext_, RexxMutableBufferObject, usize) -> POINTER },
        VariableReferenceName: { call(*mut RexxThreadContext_, RexxVariableReferenceObject) -> RexxStringObject },
        VariableReferenceValue: { call(*mut RexxThreadContext_, RexxVariableReferenceObject) -> RexxObjectPtr },
        SetVariableReferenceValue: { call(*mut RexxThreadContext_, RexxVariableReferenceObject, RexxObjectPtr) },
        IsVariableReference: { call(*mut RexxThreadContext_, RexxObjectPtr) -> logical_t },
        StringTablePut: { call(*mut RexxThreadContext_, RexxStringTableObject, RexxObjectPtr, CSTRING) },
        StringTableAt: { call(*mut RexxThreadContext_, RexxStringTableObject, CSTRING) -> RexxObjectPtr },
        StringTableRemove: { call(*mut RexxThreadContext_, RexxStringTableObject, CSTRING) -> RexxObjectPtr },
        NewStringTable: { call(*mut RexxThreadContext_) -> RexxStringTableObject },
        IsStringTable: { call(*mut RexxThreadContext_, RexxObjectPtr) -> logical_t },
        SendMessageScoped: { call(*mut RexxThreadContext_, RexxObjectPtr, CSTRING, RexxClassObject, RexxArrayObject) -> RexxObjectPtr },
        GetInterpreterInstance: { aborts call(*mut RexxThreadContext_) -> *mut RexxInstance_ },
    }
}

interface! {
    /// `MethodContextInterface` (`api/oorexxapi.h:682-712`).
    MethodContextInterface, populated {
        interfaceVersion: { value wholenumber_t = METHOD_INTERFACE_VERSION },
        GetArguments: { call(*mut RexxMethodContext_) -> RexxArrayObject },
        GetArgument: { call(*mut RexxMethodContext_, usize) -> RexxObjectPtr },
        GetMessageName: { aborts call(*mut RexxMethodContext_) -> CSTRING },
        GetMethod: { call(*mut RexxMethodContext_) -> RexxMethodObject },
        GetSelf: { call(*mut RexxMethodContext_) -> RexxObjectPtr },
        GetSuper: { call(*mut RexxMethodContext_) -> RexxClassObject },
        GetScope: { call(*mut RexxMethodContext_) -> RexxObjectPtr },
        SetObjectVariable: { call(*mut RexxMethodContext_, CSTRING, RexxObjectPtr) },
        GetObjectVariable: { call(*mut RexxMethodContext_, CSTRING) -> RexxObjectPtr },
        DropObjectVariable: { call(*mut RexxMethodContext_, CSTRING) },
        ForwardMessage: { call(*mut RexxMethodContext_, RexxObjectPtr, CSTRING, RexxClassObject, RexxArrayObject) -> RexxObjectPtr },
        SetGuardOn: { call(*mut RexxMethodContext_) },
        SetGuardOff: { call(*mut RexxMethodContext_) },
        FindContextClass: { call(*mut RexxMethodContext_, CSTRING) -> RexxClassObject },
        GetCSelf: { aborts call(*mut RexxMethodContext_) -> POINTER },
        AllocateObjectMemory: { aborts call(*mut RexxMethodContext_, usize) -> POINTER },
        FreeObjectMemory: { call(*mut RexxMethodContext_, POINTER) },
        ReallocateObjectMemory: { aborts call(*mut RexxMethodContext_, POINTER, usize) -> POINTER },
        GetObjectVariableReference: { call(*mut RexxMethodContext_, CSTRING) -> RexxVariableReferenceObject },
        SetGuardOnWhenUpdated: { call(*mut RexxMethodContext_, CSTRING) -> RexxObjectPtr },
        SetGuardOffWhenUpdated: { call(*mut RexxMethodContext_, CSTRING) -> RexxObjectPtr },
        ThrowException0: { aborts call(*mut RexxMethodContext_, usize) },
        ThrowException1: { aborts call(*mut RexxMethodContext_, usize, RexxObjectPtr) },
        ThrowException2: { aborts call(*mut RexxMethodContext_, usize, RexxObjectPtr, RexxObjectPtr) },
        ThrowException: { aborts call(*mut RexxMethodContext_, usize, RexxArrayObject) },
        ThrowCondition: { aborts call(*mut RexxMethodContext_, CSTRING, RexxStringObject, RexxObjectPtr, RexxObjectPtr) },
    }
}

interface! {
    /// `CallContextInterface` (`api/oorexxapi.h:718-743`).
    CallContextInterface, populated {
        interfaceVersion: { value wholenumber_t = CALL_INTERFACE_VERSION },
        GetArguments: { call(*mut RexxCallContext_) -> RexxArrayObject },
        GetArgument: { call(*mut RexxCallContext_, usize) -> RexxObjectPtr },
        GetRoutineName: { aborts call(*mut RexxCallContext_) -> CSTRING },
        GetRoutine: { call(*mut RexxCallContext_) -> RexxRoutineObject },
        SetContextVariable: { call(*mut RexxCallContext_, CSTRING, RexxObjectPtr) },
        GetContextVariable: { call(*mut RexxCallContext_, CSTRING) -> RexxObjectPtr },
        DropContextVariable: { call(*mut RexxCallContext_, CSTRING) },
        GetAllContextVariables: { call(*mut RexxCallContext_) -> RexxDirectoryObject },
        ResolveStemVariable: { call(*mut RexxCallContext_, RexxObjectPtr) -> RexxStemObject },
        InvalidRoutine: { call(*mut RexxCallContext_) },
        GetContextDigits: { call(*mut RexxCallContext_) -> stringsize_t },
        GetContextFuzz: { call(*mut RexxCallContext_) -> stringsize_t },
        GetContextForm: { call(*mut RexxCallContext_) -> logical_t },
        GetCallerContext: { call(*mut RexxCallContext_) -> RexxObjectPtr },
        FindContextClass: { call(*mut RexxCallContext_, CSTRING) -> RexxClassObject },
        GetContextVariableReference: { call(*mut RexxCallContext_, CSTRING) -> RexxVariableReferenceObject },
        ThrowException0: { aborts call(*mut RexxCallContext_, usize) },
        ThrowException1: { aborts call(*mut RexxCallContext_, usize, RexxObjectPtr) },
        ThrowException2: { aborts call(*mut RexxCallContext_, usize, RexxObjectPtr, RexxObjectPtr) },
        ThrowException: { aborts call(*mut RexxCallContext_, usize, RexxArrayObject) },
        ThrowCondition: { aborts call(*mut RexxCallContext_, CSTRING, RexxStringObject, RexxObjectPtr, RexxObjectPtr) },
    }
}

interface! {
    /// `ExitContextInterface` (`api/oorexxapi.h:749-763`).
    ExitContextInterface, unpopulated {
        interfaceVersion: { value wholenumber_t = EXIT_INTERFACE_VERSION },
        SetContextVariable: { call(*mut RexxExitContext_, CSTRING, RexxObjectPtr) },
        GetContextVariable: { call(*mut RexxExitContext_, CSTRING) -> RexxObjectPtr },
        DropContextVariable: { call(*mut RexxExitContext_, CSTRING) },
        GetAllContextVariables: { call(*mut RexxExitContext_) -> RexxDirectoryObject },
        GetCallerContext: { call(*mut RexxExitContext_) -> RexxObjectPtr },
        GetContextVariableReference: { call(*mut RexxExitContext_, CSTRING) -> RexxVariableReferenceObject },
        ThrowException0: { call(*mut RexxExitContext_, usize) },
        ThrowException1: { call(*mut RexxExitContext_, usize, RexxObjectPtr) },
        ThrowException2: { call(*mut RexxExitContext_, usize, RexxObjectPtr, RexxObjectPtr) },
        ThrowException: { call(*mut RexxExitContext_, usize, RexxArrayObject) },
        ThrowCondition: { call(*mut RexxExitContext_, CSTRING, RexxStringObject, RexxObjectPtr, RexxObjectPtr) },
    }
}

interface! {
    /// `IORedirectorInterface` (`api/oorexxapi.h:769-784`).
    IORedirectorInterface, unpopulated {
        interfaceVersion: { value wholenumber_t = REDIRECT_INTERFACE_VERSION },
        ReadInput: { call(*mut RexxIORedirectorContext_, *mut CSTRING, *mut usize) },
        ReadInputBuffer: { call(*mut RexxIORedirectorContext_, *mut CSTRING, *mut usize) },
        WriteOutput: { call(*mut RexxIORedirectorContext_, CSTRING, usize) },
        WriteError: { call(*mut RexxIORedirectorContext_, CSTRING, usize) },
        WriteOutputBuffer: { call(*mut RexxIORedirectorContext_, CSTRING, usize) },
        WriteErrorBuffer: { call(*mut RexxIORedirectorContext_, CSTRING, usize) },
        IsInputRedirected: { call(*mut RexxIORedirectorContext_) -> logical_t },
        IsOutputRedirected: { call(*mut RexxIORedirectorContext_) -> logical_t },
        IsErrorRedirected: { call(*mut RexxIORedirectorContext_) -> logical_t },
        AreOutputAndErrorSameTarget: { call(*mut RexxIORedirectorContext_) -> logical_t },
        IsRedirectionRequested: { call(*mut RexxIORedirectorContext_) -> logical_t },
    }
}

/// The method-context table an extension calls through, at one address.
pub static METHOD_CONTEXT_INTERFACE: MethodContextInterface = MethodContextInterface::REFUSING;

// The thread table has no `static` beside it: its object members are raw
// pointers, so the type is not `Sync`, and the C++ fills them once the constant
// objects exist (`interpreter/concurrency/Activity.cpp:1844-1849`). Whatever
// hands out a thread context owns the table, which is also what keeps this
// phase from mutating a global.

/// The exit-context interface, which nothing reaches yet: what a method or call
/// context addresses is the thread table, its instance and its own context
/// table.
///
/// # Panics
/// Always.
pub fn exit_context_interface() -> ExitContextInterface {
    abort("ExitContextInterface");
}

/// The I/O-redirector interface, unreached for the reason
/// [`exit_context_interface`] gives.
///
/// # Panics
/// Always.
pub fn io_redirector_interface() -> IORedirectorInterface {
    abort("IORedirectorInterface");
}
