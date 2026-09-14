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

//! The conversion table an extension's declared signature drives.
//!
//! `NativeActivation::processArguments` (`interpreter/execution/NativeActivation.cpp:219`)
//! is the reference for the Rexx-to-C direction and `valueToObject` (`:718`)
//! for the way back.

use std::borrow::Cow;
use std::cell::{Cell, RefCell, RefMut};
use std::ffi::{c_char, c_int};

use rexx_core::ObjRef;

use crate::handles::Table;
use crate::layout::{CSTRING, POINTER, RexxObjectPtr, ValueDescriptor, ValueUnion};

/// `REXX_ARGUMENT_TERMINATOR` (`api/oorexxapi.h:54`).
pub const ARGUMENT_TERMINATOR: u16 = 0;

/// `REXX_OPTIONAL_ARGUMENT` (`api/oorexxapi.h:100`).
pub const OPTIONAL_ARGUMENT: u16 = 0x8000;

/// `ARGUMENT_EXISTS` (`api/oorexxapi.h:278`).
pub const ARGUMENT_EXISTS: u16 = 0x01;

/// `SPECIAL_ARGUMENT` (`api/oorexxapi.h:280`).
pub const SPECIAL_ARGUMENT: u16 = 0x02;

/// `ARGUMENT_TYPE` (`api/oorexxapi.h:4272`): a signature word without its
/// optional bit.
pub const fn argument_type(declared: u16) -> u16 {
    declared & !OPTIONAL_ARGUMENT
}

/// `IS_OPTIONAL_ARGUMENT` (`api/oorexxapi.h:4273`).
pub const fn is_optional(declared: u16) -> bool {
    declared & OPTIONAL_ARGUMENT != 0
}

/// The `REXX_VALUE_*` codes (`api/oorexxapi.h:55-98`).
///
/// A `REXX_VALUE_OPTIONAL_*` name in the header is one of these with
/// [`OPTIONAL_ARGUMENT`] set, not a code of its own, so the table is keyed on
/// these alone.
pub mod code {
    pub const ARGLIST: u16 = 2;
    pub const NAME: u16 = 3;
    pub const SCOPE: u16 = 4;
    pub const CSELF: u16 = 5;
    pub const OSELF: u16 = 6;
    pub const SUPER: u16 = 7;
    pub const REXX_OBJECT_PTR: u16 = 11;
    pub const INT: u16 = 12;
    pub const WHOLENUMBER_T: u16 = 13;
    pub const DOUBLE: u16 = 14;
    pub const CSTRING: u16 = 15;
    pub const POINTER: u16 = 16;
    pub const REXX_STRING_OBJECT: u16 = 17;
    pub const STRINGSIZE_T: u16 = 18;
    pub const FLOAT: u16 = 19;
    pub const INT8_T: u16 = 20;
    pub const INT16_T: u16 = 21;
    pub const INT32_T: u16 = 22;
    pub const INT64_T: u16 = 23;
    pub const UINT8_T: u16 = 24;
    pub const UINT16_T: u16 = 25;
    pub const UINT32_T: u16 = 26;
    pub const UINT64_T: u16 = 27;
    pub const INTPTR_T: u16 = 28;
    pub const UINTPTR_T: u16 = 29;
    pub const LOGICAL_T: u16 = 30;
    pub const REXX_ARRAY_OBJECT: u16 = 31;
    pub const REXX_STEM_OBJECT: u16 = 32;
    pub const SIZE_T: u16 = 33;
    pub const SSIZE_T: u16 = 34;
    pub const POINTERSTRING: u16 = 35;
    pub const REXX_CLASS_OBJECT: u16 = 36;
    pub const REXX_MUTABLE_BUFFER_OBJECT: u16 = 37;
    pub const POSITIVE_WHOLENUMBER_T: u16 = 38;
    pub const NONNEGATIVE_WHOLENUMBER_T: u16 = 39;
    pub const REXX_VARIABLE_REFERENCE_OBJECT: u16 = 40;
}

/// Which way a conversion runs.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Direction {
    /// A Rexx object into the C value the signature declares.
    ToNative,
    /// A value the extension wrote back into a Rexx object.
    FromNative,
}

/// A conversion that could not be made.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Failure {
    /// A required argument was not supplied.
    MissingArgument { position: usize },
    /// The argument has no string value.
    NoStringValue { position: usize },
    /// The signature itself cannot be honoured: a code the table does not
    /// know, a `Value` that does not match the declared code, or a special
    /// argument asked for outside a method.
    Signature,
    /// A row whose conversion this phase has not written.
    Unfilled {
        code: u16,
        name: &'static str,
        direction: Direction,
    },
    /// More arguments were supplied than the signature consumes.
    /// `expected` is the number it does consume.
    TooManyArguments { expected: usize },
    /// A handle the calling activation no longer holds (D5).
    StaleHandle,
}

impl Failure {
    /// The error the oracle raises, or `None` where the refusal is this
    /// implementation's gap rather than a condition a program can see.
    pub fn error_number(&self, method: bool) -> Option<u32> {
        match self {
            // Measured 2026-09-14 against the oracle: a required CSTRING left
            // off `RegularExpression~parse` answers "Error 88.901".
            Failure::MissingArgument { .. } => Some(88901),
            // Measured the same way, passing an instance of a class with no
            // string value: "Error 88.909".
            Failure::NoStringValue { .. } => Some(88909),
            Failure::Signature => Some(if method { 93968 } else { 40918 }),
            // Measured 2026-09-14 against the oracle: a third argument to
            // `RegularExpression~new` answers "Error 88.922".
            Failure::TooManyArguments { .. } => Some(88922),
            Failure::Unfilled { .. } | Failure::StaleHandle => None,
        }
    }
}

impl std::fmt::Display for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Failure::MissingArgument { position } => {
                write!(f, "argument {position} is required")
            }
            Failure::NoStringValue { position } => {
                write!(f, "argument {position} must have a string value")
            }
            Failure::Signature => write!(f, "incorrect signature"),
            Failure::TooManyArguments { expected } => {
                write!(f, "too many arguments in invocation; {expected} expected")
            }
            Failure::Unfilled {
                code,
                name,
                direction,
            } => write!(
                f,
                "Phase 8 owes the {direction:?} conversion for REXX_VALUE_{name} ({code})"
            ),
            Failure::StaleHandle => write!(f, "the handle is no longer held by this activation"),
        }
    }
}

/// One member of a `ValueDescriptor`'s union, chosen by the declared code.
///
/// There is one variant per [`Repr`], so a descriptor's live member can be
/// named without a second switch on the code.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Value {
    /// An optional argument nobody supplied.
    Omitted,
    Int(c_int),
    /// A pointer into the call's [`CStringPool`].
    CString(CSTRING),
    Pointer(POINTER),
    Object(RexxObjectPtr),
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    Uint8(u8),
    Uint16(u16),
    Uint32(u32),
    Uint64(u64),
    Isize(isize),
    Usize(usize),
    Double(f64),
    Float(f32),
}

impl Value {
    /// The union with this value's member written.
    ///
    /// An omitted argument is a zero word: the C++ writes `value_int64_t = 0`
    /// for the integer and pointer types and `0.0` for `double` and `float`,
    /// which are the same bytes.
    fn as_union(self) -> ValueUnion {
        match self {
            Value::Omitted => ValueUnion { value_int64_t: 0 },
            Value::Int(v) => ValueUnion { value_int: v },
            Value::CString(p) => ValueUnion { value_CSTRING: p },
            Value::Pointer(p) => ValueUnion { value_POINTER: p },
            Value::Object(h) => ValueUnion {
                value_RexxObjectPtr: h,
            },
            Value::Int8(v) => ValueUnion { value_int8_t: v },
            Value::Int16(v) => ValueUnion { value_int16_t: v },
            Value::Int32(v) => ValueUnion { value_int32_t: v },
            Value::Int64(v) => ValueUnion { value_int64_t: v },
            Value::Uint8(v) => ValueUnion { value_uint8_t: v },
            Value::Uint16(v) => ValueUnion { value_uint16_t: v },
            Value::Uint32(v) => ValueUnion { value_uint32_t: v },
            Value::Uint64(v) => ValueUnion { value_uint64_t: v },
            Value::Isize(v) => ValueUnion {
                value_wholenumber_t: v,
            },
            Value::Usize(v) => ValueUnion {
                value_stringsize_t: v,
            },
            Value::Double(v) => ValueUnion { value_double: v },
            Value::Float(v) => ValueUnion { value_float: v },
        }
    }
}

/// A converted argument and the `flags` its descriptor carries.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Converted {
    pub value: Value,
    pub flags: u16,
}

/// The `ValueDescriptor` a converted argument fills in.
///
/// `declared` is the signature word the extension published. The optional bit
/// is stripped here, because what the descriptor carries is the type the
/// extension reads back (`NativeActivation.cpp:243`, `:246`).
pub fn descriptor(declared: u16, converted: Converted) -> ValueDescriptor {
    ValueDescriptor {
        value: converted.value.as_union(),
        r#type: argument_type(declared),
        flags: converted.flags,
    }
}

/// The terminated copies a call's `CSTRING` conversions point at.
///
/// A `CSTRING`'s lifetime is the call, and the oracle achieves that by rooting
/// the string object and pointing into it. That is not open to us: a Rexx
/// string carries no terminator, and the arena moves nothing but reallocates
/// the slot vector, so no address inside the heap survives an allocation. The
/// pool owns its copies instead, so a collection cannot free them, and each
/// one is a separate boxed slice, so growing the pool does not move a pointer
/// already handed out. Dropping the pool is what ends the lifetime, and that
/// is the end of the call.
#[derive(Default)]
pub struct CStringPool {
    /// Each copy with the object it was made for, `None` for a copy
    /// [`CStringPool::intern`] made, which is keyed by nothing.
    entries: Vec<(Option<ObjRef>, Box<[u8]>)>,
}

impl CStringPool {
    pub fn new() -> CStringPool {
        CStringPool::default()
    }

    /// A pointer to a terminated copy of `bytes`, valid until
    /// [`CStringPool::clear`] or the pool is dropped.
    ///
    /// Bytes are copied verbatim, so an embedded zero truncates the string an
    /// extension reads, which is what pointing into the oracle's string data
    /// does too. Each call answers a fresh copy at its own address.
    pub fn intern(&mut self, bytes: &[u8]) -> CSTRING {
        self.push(None, bytes)
    }

    /// [`CStringPool::intern`] keyed on the object the bytes came from, so
    /// that asking twice answers one address.
    ///
    /// `StringData` (`interpreter/api/ThreadContextStubs.cpp:1068`) hands out
    /// an interior pointer to a non-moving object, so the oracle answers the
    /// same address for the same object however often it is asked. `object`
    /// is what makes that hold here; the bytes are read again only when the
    /// pool has no copy for it yet.
    pub fn intern_for(&mut self, object: ObjRef, bytes: &[u8]) -> CSTRING {
        if let Some(found) = self.pointer_for(object) {
            return found;
        }
        self.push(Some(object), bytes)
    }

    /// The address this pool already answers for `object`, or `None`.
    fn pointer_for(&self, object: ObjRef) -> Option<CSTRING> {
        self.entries
            .iter()
            .find(|(key, _)| *key == Some(object))
            .map(|(_, entry)| entry.as_ptr().cast::<c_char>())
    }

    fn push(&mut self, key: Option<ObjRef>, bytes: &[u8]) -> CSTRING {
        let mut owned = Vec::with_capacity(bytes.len() + 1);
        owned.extend_from_slice(bytes);
        owned.push(0);
        self.entries.push((key, owned.into_boxed_slice()));
        let (_, entry) = self.entries.last().expect("just pushed");
        entry.as_ptr().cast::<c_char>()
    }

    /// The bytes behind a pointer this pool minted, without the terminator,
    /// or `None` for a pointer it did not mint or has since dropped.
    pub fn bytes_at(&self, pointer: CSTRING) -> Option<&[u8]> {
        self.entries
            .iter()
            .find(|(_, entry)| entry.as_ptr().cast::<c_char>() == pointer)
            .map(|(_, entry)| &entry[..entry.len() - 1])
    }

    /// Drops every copy, which is what the end of a call does.
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// What the table needs from the interpreter beyond the objects themselves.
pub trait Host {
    /// Whether this is a method invocation rather than a call.
    fn is_method(&self) -> bool;

    /// `object` forced to a string object, or `None` if it has no string
    /// value. `RexxInternalObject::requiredString`
    /// (`interpreter/classes/ObjectClass.cpp:1341`) is the reference, so a
    /// non-primitive receiver is sent `REQUEST("STRING")` and answering
    /// `.nil` is the `None` here.
    fn string_value(&mut self, object: ObjRef) -> Option<ObjRef>;

    /// The bytes of a string object, or `None` if `object` is not one. The
    /// return is owned rather than borrowed for a string the handle itself
    /// carries, which has no heap object to borrow from.
    fn string_bytes(&self, object: ObjRef) -> Option<Cow<'_, [u8]>>;

    /// The `CSELF` object variable's pointer, `None` when the receiver has
    /// none. `NativeActivation::cself` (`:2091`) is the reference: it takes
    /// the guard lock through `methodVariables` and unwraps the `.Pointer`
    /// the variable holds.
    fn cself(&mut self) -> Option<POINTER>;

    /// The constant objects `Activity::initializeThreadContext`
    /// (`interpreter/concurrency/Activity.cpp:1841-1849`) patches into the
    /// thread table once they exist.
    fn constants(&mut self) -> Constants<ObjRef>;

    /// Binds `name` in the running method's own scope pool, or returns it to
    /// the uninitialised state for a `value` of `None`.
    ///
    /// `name` is the extension's own spelling: `getVariableRetriever`
    /// (`interpreter/execution/VariableDictionary.cpp:738`) upper-cases it
    /// and refuses one that is not a variable name, and both belong to the
    /// interpreter rather than to the boundary. The scope is the method's,
    /// not the receiver's class (`NativeActivation.cpp:1878`) -- measured
    /// 2026-09-14 against the oracle, a subclass method's `expose CSELF`
    /// reads an unset variable where the defining class's reads the pointer.
    fn set_object_variable(&mut self, name: &[u8], value: Option<ObjRef>);

    /// [`Host::set_object_variable`]'s scope and name rules, applied to
    /// `NativeActivation::dropObjectVariable` (`:3071`).
    fn drop_object_variable(&mut self, name: &[u8]);

    /// `Numerics::wholenumberToObject` (`interpreter/runtime/Numerics.cpp:855`).
    fn whole_number(&mut self, value: isize) -> ObjRef;

    /// A `.Pointer` wrapping `value`, which is `new_pointer`
    /// (`interpreter/classes/PointerClass.hpp:114`).
    fn new_pointer(&mut self, value: POINTER) -> ObjRef;
}

/// The four objects the thread table carries as data rather than as
/// functions, over whatever names them.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Constants<T> {
    pub nil: T,
    pub true_object: T,
    pub false_object: T,
    pub null_string: T,
}

/// What one native call's conversions read and write.
pub struct Conversion<'a> {
    pub host: &'a mut dyn Host,
    pub locals: &'a mut Table,
    pub strings: &'a mut CStringPool,
}

/// One native call's whole interpreter-facing state: what the conversions
/// read and write, and the condition an extension raised into it.
///
/// Held by shared reference so that the context an extension is given and the
/// protocol driving that call reach the same state without either holding a
/// unique borrow across the other. Every entry takes the cell for the length
/// of one operation, so an extension calling back while a conversion is in
/// flight is a panic and not a silent aliasing.
pub struct Activation<'a> {
    conversion: RefCell<Conversion<'a>>,
    pending: Cell<Option<usize>>,
}

impl<'a> Activation<'a> {
    pub fn new(conversion: Conversion<'a>) -> Activation<'a> {
        Activation {
            conversion: RefCell::new(conversion),
            pending: Cell::new(None),
        }
    }

    /// The conversion state, for the length of one operation.
    ///
    /// # Panics
    /// If the caller already holds it, which is an extension entering the
    /// interpreter while the interpreter is inside a conversion.
    pub fn conversion(&self) -> RefMut<'_, Conversion<'a>> {
        self.conversion.borrow_mut()
    }

    /// `RaiseException0` (`interpreter/api/ThreadContextStubs.cpp:1863`):
    /// records the condition and returns, leaving the extension to run on.
    /// A second raise overwrites the first, as `setConditionInfo`
    /// (`interpreter/execution/NativeActivation.cpp:2678`) does.
    pub fn raise(&self, number: usize) {
        self.pending.set(Some(number));
    }

    /// The condition the call has to raise once it returns
    /// (`NativeActivation::checkConditions`, `:1787`), or `None`.
    pub fn pending(&self) -> Option<usize> {
        self.pending.get()
    }

    /// Forgets a recorded condition, which is what raising it does.
    pub fn clear_pending(&self) {
        self.pending.set(None);
    }

    /// `SetObjectVariable`: `value` is the handle the extension passed, and a
    /// handle this activation does not hold -- a null one included -- is the
    /// `OREF_NULL` the oracle would store (D5).
    pub fn set_object_variable(&self, name: &[u8], value: RexxObjectPtr) {
        let mut cx = self.conversion();
        let object = cx.locals.resolve(value);
        cx.host.set_object_variable(name, object);
    }

    /// `DropObjectVariable`.
    pub fn drop_object_variable(&self, name: &[u8]) {
        let mut cx = self.conversion();
        cx.host.drop_object_variable(name);
    }

    /// `WholeNumberToObject`, registered as a local reference the way
    /// `ApiContext::ret` registers one.
    pub fn whole_number(&self, value: isize) -> RexxObjectPtr {
        let mut cx = self.conversion();
        let object = cx.host.whole_number(value);
        cx.locals.register(object)
    }

    /// `NewPointer`, registered as [`Activation::whole_number`]'s answer is.
    pub fn new_pointer(&self, value: POINTER) -> RexxObjectPtr {
        let mut cx = self.conversion();
        let object = cx.host.new_pointer(value);
        cx.locals.register(object)
    }

    /// `StringData`: one address per object for as long as the call lasts, or
    /// a null pointer for a handle this activation does not hold or an object
    /// with no bytes, which is the `NULL` the stub answers on an exception.
    pub fn string_data(&self, handle: RexxObjectPtr) -> CSTRING {
        let mut cx = self.conversion();
        let Some(object) = cx.locals.resolve(handle) else {
            return std::ptr::null();
        };
        let Some(bytes) = cx.host.string_bytes(object) else {
            return std::ptr::null();
        };
        let bytes = bytes.into_owned();
        cx.strings.intern_for(object, &bytes)
    }

    /// `StringLength`, or zero where [`Activation::string_data`] answers a
    /// null pointer.
    pub fn string_length(&self, handle: RexxObjectPtr) -> usize {
        let cx = self.conversion();
        let Some(object) = cx.locals.resolve(handle) else {
            return 0;
        };
        cx.host.string_bytes(object).map_or(0, |bytes| bytes.len())
    }

    /// The four data members of the thread table, each registered as a local
    /// reference so that the handle names a rooted object.
    pub fn constants(&self) -> Constants<RexxObjectPtr> {
        let mut cx = self.conversion();
        let objects = cx.host.constants();
        Constants {
            nil: cx.locals.register(objects.nil),
            true_object: cx.locals.register(objects.true_object),
            false_object: cx.locals.register(objects.false_object),
            null_string: cx.locals.register(objects.null_string),
        }
    }
}

/// Which member of a `ValueDescriptor`'s union a code's value occupies
/// (`api/oorexxapi.h:287-364`).
///
/// The header's own `ARGUMENT_TYPE_<name>` defines (`:4196-4239`) are what say
/// which member a code uses, and the test derives this mapping from them.
/// Reading a union is `unsafe` and lives past the boundary in `ffi.rs` (D-U1),
/// so this is how the table tells that side what to read without a second
/// switch on the code.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Repr {
    /// Any object-typed member, all of them `RexxObjectPtr`-shaped handles.
    Object,
    CString,
    /// `value_POINTER` and `value_POINTERSTRING`.
    Pointer,
    Int,
    Int8,
    Int16,
    Int32,
    Int64,
    Uint8,
    Uint16,
    Uint32,
    Uint64,
    /// `wholenumber_t`, `ssize_t` and `intptr_t`.
    Isize,
    /// `stringsize_t`, `size_t`, `uintptr_t` and `logical_t`.
    Usize,
    Double,
    Float,
}

/// Whether a row takes its value from the argument list or from the context.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Source {
    /// The row consumes one argument and its descriptor carries
    /// [`ARGUMENT_EXISTS`].
    Argument,
    /// The row consumes no argument and its descriptor carries
    /// [`SPECIAL_ARGUMENT`] as well.
    Special,
}

/// What an optional argument nobody supplied writes
/// (`NativeActivation.cpp:615`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Absent {
    Zero,
    Signature,
}

type ToNative = fn(&mut Conversion<'_>, ObjRef, usize) -> Result<Value, Failure>;
type FromNative = fn(&mut Conversion<'_>, Value) -> Result<Option<ObjRef>, Failure>;

/// One `REXX_VALUE_*` code's conversion.
struct Row {
    code: u16,
    /// The header's spelling after `REXX_VALUE_`, which is what
    /// [`Failure::Unfilled`] and the coverage test name.
    name: &'static str,
    source: Source,
    absent: Absent,
    repr: Repr,
    to_native: Option<ToNative>,
    from_native: Option<FromNative>,
}

/// A row whose conversions are not written yet.
const fn stub(code: u16, name: &'static str, source: Source, absent: Absent, repr: Repr) -> Row {
    Row {
        code,
        name,
        source,
        absent,
        repr,
        to_native: None,
        from_native: None,
    }
}

static TABLE: &[Row] = &[
    stub(
        code::ARGLIST,
        "ARGLIST",
        Source::Special,
        Absent::Signature,
        Repr::Object,
    ),
    stub(
        code::NAME,
        "NAME",
        Source::Special,
        Absent::Signature,
        Repr::CString,
    ),
    stub(
        code::SCOPE,
        "SCOPE",
        Source::Special,
        Absent::Signature,
        Repr::Object,
    ),
    Row {
        code: code::CSELF,
        name: "CSELF",
        source: Source::Special,
        absent: Absent::Signature,
        repr: Repr::Pointer,
        to_native: Some(cself_to_native),
        from_native: None,
    },
    stub(
        code::OSELF,
        "OSELF",
        Source::Special,
        Absent::Signature,
        Repr::Object,
    ),
    stub(
        code::SUPER,
        "SUPER",
        Source::Special,
        Absent::Signature,
        Repr::Object,
    ),
    stub(
        code::REXX_OBJECT_PTR,
        "RexxObjectPtr",
        Source::Argument,
        Absent::Zero,
        Repr::Object,
    ),
    Row {
        code: code::INT,
        name: "int",
        source: Source::Argument,
        absent: Absent::Zero,
        repr: Repr::Int,
        to_native: None,
        from_native: Some(int_from_native),
    },
    stub(
        code::WHOLENUMBER_T,
        "wholenumber_t",
        Source::Argument,
        Absent::Zero,
        Repr::Isize,
    ),
    stub(
        code::DOUBLE,
        "double",
        Source::Argument,
        Absent::Zero,
        Repr::Double,
    ),
    Row {
        code: code::CSTRING,
        name: "CSTRING",
        source: Source::Argument,
        absent: Absent::Zero,
        repr: Repr::CString,
        to_native: Some(cstring_to_native),
        from_native: None,
    },
    Row {
        code: code::POINTER,
        name: "POINTER",
        source: Source::Argument,
        absent: Absent::Zero,
        repr: Repr::Pointer,
        to_native: None,
        from_native: Some(pointer_from_native),
    },
    Row {
        code: code::REXX_STRING_OBJECT,
        name: "RexxStringObject",
        source: Source::Argument,
        absent: Absent::Zero,
        repr: Repr::Object,
        to_native: Some(string_object_to_native),
        from_native: Some(object_from_native),
    },
    stub(
        code::STRINGSIZE_T,
        "stringsize_t",
        Source::Argument,
        Absent::Zero,
        Repr::Usize,
    ),
    stub(
        code::FLOAT,
        "float",
        Source::Argument,
        Absent::Zero,
        Repr::Float,
    ),
    stub(
        code::INT8_T,
        "int8_t",
        Source::Argument,
        Absent::Zero,
        Repr::Int8,
    ),
    stub(
        code::INT16_T,
        "int16_t",
        Source::Argument,
        Absent::Zero,
        Repr::Int16,
    ),
    stub(
        code::INT32_T,
        "int32_t",
        Source::Argument,
        Absent::Zero,
        Repr::Int32,
    ),
    stub(
        code::INT64_T,
        "int64_t",
        Source::Argument,
        Absent::Zero,
        Repr::Int64,
    ),
    stub(
        code::UINT8_T,
        "uint8_t",
        Source::Argument,
        Absent::Zero,
        Repr::Uint8,
    ),
    stub(
        code::UINT16_T,
        "uint16_t",
        Source::Argument,
        Absent::Zero,
        Repr::Uint16,
    ),
    stub(
        code::UINT32_T,
        "uint32_t",
        Source::Argument,
        Absent::Zero,
        Repr::Uint32,
    ),
    stub(
        code::UINT64_T,
        "uint64_t",
        Source::Argument,
        Absent::Zero,
        Repr::Uint64,
    ),
    stub(
        code::INTPTR_T,
        "intptr_t",
        Source::Argument,
        Absent::Zero,
        Repr::Isize,
    ),
    stub(
        code::UINTPTR_T,
        "uintptr_t",
        Source::Argument,
        Absent::Zero,
        Repr::Usize,
    ),
    stub(
        code::LOGICAL_T,
        "logical_t",
        Source::Argument,
        Absent::Zero,
        Repr::Usize,
    ),
    stub(
        code::REXX_ARRAY_OBJECT,
        "RexxArrayObject",
        Source::Argument,
        Absent::Zero,
        Repr::Object,
    ),
    stub(
        code::REXX_STEM_OBJECT,
        "RexxStemObject",
        Source::Argument,
        Absent::Zero,
        Repr::Object,
    ),
    stub(
        code::SIZE_T,
        "size_t",
        Source::Argument,
        Absent::Zero,
        Repr::Usize,
    ),
    stub(
        code::SSIZE_T,
        "ssize_t",
        Source::Argument,
        Absent::Zero,
        Repr::Isize,
    ),
    stub(
        code::POINTERSTRING,
        "POINTERSTRING",
        Source::Argument,
        Absent::Zero,
        Repr::Pointer,
    ),
    stub(
        code::REXX_CLASS_OBJECT,
        "RexxClassObject",
        Source::Argument,
        Absent::Zero,
        Repr::Object,
    ),
    stub(
        code::REXX_MUTABLE_BUFFER_OBJECT,
        "RexxMutableBufferObject",
        Source::Argument,
        Absent::Zero,
        Repr::Object,
    ),
    stub(
        code::POSITIVE_WHOLENUMBER_T,
        "positive_wholenumber_t",
        Source::Argument,
        Absent::Zero,
        Repr::Isize,
    ),
    stub(
        code::NONNEGATIVE_WHOLENUMBER_T,
        "nonnegative_wholenumber_t",
        Source::Argument,
        Absent::Zero,
        Repr::Isize,
    ),
    // The header marks this one as never optional (`api/oorexxapi.h:98`) and
    // the C++ leaves it out of the absent switch, so an omitted one is a
    // signature error rather than a zero.
    stub(
        code::REXX_VARIABLE_REFERENCE_OBJECT,
        "RexxVariableReferenceObject",
        Source::Argument,
        Absent::Signature,
        Repr::Object,
    ),
];

/// The union member `declared`'s value occupies, or `None` for a code the
/// table does not know.
pub fn repr(declared: u16) -> Option<Repr> {
    row(argument_type(declared)).map(|row| row.repr)
}

/// Every code the table has a row for, with the header's spelling of its
/// name.
pub fn rows() -> impl Iterator<Item = (u16, &'static str)> {
    TABLE.iter().map(|row| (row.code, row.name))
}

/// The row for `code`, or `None` for a code the header does not define.
fn row(code: u16) -> Option<&'static Row> {
    TABLE.iter().find(|row| row.code == code)
}

/// Whether the code in `declared` takes its value from the argument list.
///
/// Returns `None` for a code the table does not know.
pub fn consumes_argument(declared: u16) -> Option<bool> {
    row(argument_type(declared)).map(|row| row.source == Source::Argument)
}

/// `argument` converted into the C value `declared` asks for.
///
/// `argument` is the Rexx object at this position, or `None` when the caller
/// supplied nothing there; a [`Source::Special`] row ignores it either way.
/// `position` is one-based, as the oracle's error inserts are.
pub fn to_native(
    cx: &mut Conversion<'_>,
    declared: u16,
    argument: Option<ObjRef>,
    position: usize,
) -> Result<Converted, Failure> {
    let code = argument_type(declared);
    let Some(row) = row(code) else {
        return Err(Failure::Signature);
    };
    // The absent cases are settled before the per-type conversion, as they
    // are in the C++: a missing required argument and an omitted optional one
    // are both answered without entering the type switch.
    let object = match row.source {
        Source::Special => ObjRef::NIL,
        Source::Argument => match argument {
            Some(object) => object,
            None if !is_optional(declared) => {
                return Err(Failure::MissingArgument { position });
            }
            None => {
                return match row.absent {
                    Absent::Zero => Ok(Converted {
                        value: Value::Omitted,
                        flags: 0,
                    }),
                    Absent::Signature => Err(Failure::Signature),
                };
            }
        },
    };
    let convert = row.to_native.ok_or(Failure::Unfilled {
        code,
        name: row.name,
        direction: Direction::ToNative,
    })?;
    let flags = match row.source {
        Source::Special => ARGUMENT_EXISTS | SPECIAL_ARGUMENT,
        Source::Argument => ARGUMENT_EXISTS,
    };
    Ok(Converted {
        value: convert(cx, object, position)?,
        flags,
    })
}

/// The Rexx object `value` describes under `declared`, or `None` where the
/// oracle answers `OREF_NULL`.
pub fn from_native(
    cx: &mut Conversion<'_>,
    declared: u16,
    value: Value,
) -> Result<Option<ObjRef>, Failure> {
    let code = argument_type(declared);
    // `valueToObject` treats a zero type as an omitted value rather than a bad
    // one, which is what makes a partly filled argument list convertible.
    if code == ARGUMENT_TERMINATOR {
        return Ok(None);
    }
    let Some(row) = row(code) else {
        return Err(Failure::Signature);
    };
    let convert = row.from_native.ok_or(Failure::Unfilled {
        code,
        name: row.name,
        direction: Direction::FromNative,
    })?;
    convert(cx, value)
}

/// `REXX_VALUE_CSELF` (`NativeActivation.cpp:294`).
fn cself_to_native(
    cx: &mut Conversion<'_>,
    _argument: ObjRef,
    _position: usize,
) -> Result<Value, Failure> {
    if !cx.host.is_method() {
        return Err(Failure::Signature);
    }
    Ok(Value::Pointer(
        cx.host.cself().unwrap_or(std::ptr::null_mut()),
    ))
}

/// `REXX_VALUE_CSTRING` (`NativeActivation.cpp:467`).
fn cstring_to_native(
    cx: &mut Conversion<'_>,
    argument: ObjRef,
    position: usize,
) -> Result<Value, Failure> {
    let string = cx
        .host
        .string_value(argument)
        .ok_or(Failure::NoStringValue { position })?;
    let bytes = cx
        .host
        .string_bytes(string)
        .ok_or(Failure::NoStringValue { position })?;
    Ok(Value::CString(cx.strings.intern(&bytes)))
}

/// `REXX_VALUE_RexxStringObject` (`NativeActivation.cpp:473`).
fn string_object_to_native(
    cx: &mut Conversion<'_>,
    argument: ObjRef,
    position: usize,
) -> Result<Value, Failure> {
    let string = cx
        .host
        .string_value(argument)
        .ok_or(Failure::NoStringValue { position })?;
    // The C++ registers a local reference only for a string the conversion
    // had to create. Minting a handle is registering it, so here every one is
    // rooted and the "was it created" question does not arise.
    Ok(Value::Object(cx.locals.register(string)))
}

/// `valueToObject` for the object codes (`NativeActivation.cpp:723`).
fn object_from_native(cx: &mut Conversion<'_>, value: Value) -> Result<Option<ObjRef>, Failure> {
    let Value::Object(handle) = value else {
        return Err(Failure::Signature);
    };
    if handle.is_null() {
        return Ok(None);
    }
    cx.locals
        .resolve(handle)
        .map(Some)
        .ok_or(Failure::StaleHandle)
}

/// `valueToObject` for `REXX_VALUE_POINTER` (`NativeActivation.cpp:840`),
/// which wraps the address in a `.Pointer` and does not register it: the
/// answer is the call's result, which the caller roots.
fn pointer_from_native(cx: &mut Conversion<'_>, value: Value) -> Result<Option<ObjRef>, Failure> {
    let Value::Pointer(address) = value else {
        return Err(Failure::Signature);
    };
    Ok(Some(cx.host.new_pointer(address)))
}

/// `valueToObject` for `REXX_VALUE_int` (`NativeActivation.cpp:733`).
///
/// # Panics
/// Never for a `c_int`, whose whole range is a small integer.
fn int_from_native(_cx: &mut Conversion<'_>, value: Value) -> Result<Option<ObjRef>, Failure> {
    let Value::Int(number) = value else {
        return Err(Failure::Signature);
    };
    Ok(Some(
        ObjRef::small_int(i64::from(number)).expect("a c_int is a small integer"),
    ))
}
