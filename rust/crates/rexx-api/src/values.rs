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

use crate::callbacks::Surface;
use crate::handles::Table;
use crate::layout::{CSTRING, POINTER, RexxObjectPtr, ValueDescriptor, ValueUnion};

mod convert;
use convert::{
    arglist_to_native, array_to_native, class_to_native, cself_to_native, cstring_from_native,
    cstring_to_native, double_from_native, double_to_native, float_from_native, float_to_native,
    int_from_native, int_to_native, int8_from_native, int8_to_native, int16_from_native,
    int16_to_native, int32_from_native, int32_to_native, int64_from_native, int64_to_native,
    isize_from_native, logical_from_native, logical_to_native, mutable_buffer_to_native,
    name_to_native, nonnegative_whole_number_to_native, object_from_native, object_to_native,
    oself_to_native, pointer_from_native, pointer_string_from_native, pointer_string_to_native,
    pointer_to_native, positive_whole_number_to_native, scope_to_native, signed_word_to_native,
    stem_to_native, string_object_to_native, string_size_to_native, super_to_native,
    uint8_from_native, uint8_to_native, uint16_from_native, uint16_to_native, uint32_from_native,
    uint32_to_native, uint64_from_native, uint64_to_native, unsigned_word_to_native,
    usize_from_native, variable_reference_to_native, whole_number_to_native,
};

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

/// A conversion that could not be made.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Failure {
    /// A required argument was not supplied.
    MissingArgument { position: usize },
    /// The argument has no string value.
    NoStringValue { position: usize },
    /// The declared parameters cannot be honoured: a code the table does not
    /// know, or a special argument asked for outside a method.
    Signature,
    /// The declared return type cannot be converted back: a code the table
    /// does not know, one `valueToObject` refuses, or a `Value` that does not
    /// match it. The oracle raises
    /// this once the extension has run (`NativeActivation.cpp:1301-1310`).
    ResultSignature,
    /// The extension reached an interface member this phase has not written,
    /// named `Table.Member`; the member recorded itself and returned.
    UnfilledSlot { entry: &'static str },
    /// The argument has no value as a C `double`.
    InvalidDouble { position: usize, argument: ObjRef },
    /// The argument is not a whole number from one to [`MAX_WHOLENUMBER`].
    NotPositive { position: usize, argument: ObjRef },
    /// The argument is not a whole number from zero to [`MAX_WHOLENUMBER`].
    NotNonnegative { position: usize, argument: ObjRef },
    /// The argument is not a whole number from `min` to `max`.
    OutOfRange {
        position: usize,
        min: i128,
        max: i128,
        argument: ObjRef,
    },
    /// The argument's string value, `found`, is not exactly `0` or `1`.
    NotLogical { found: ObjRef },
    /// The argument has no single-dimensional array value.
    NotArray { argument: ObjRef },
    /// The argument is not an instance of the class `class` names.
    NotInstance { position: usize, class: Class },
    /// The argument's string value is not an address written after `0x`.
    NotPointerString { position: usize, argument: ObjRef },
    /// The argument is not a stem, nor in a call the name of one.
    NoStem { position: usize, argument: ObjRef },
    /// More arguments were supplied than the signature consumes.
    /// `expected` is the number it does consume.
    TooManyArguments { expected: usize },
    /// A `ROUTINE_CLASSIC_STYLE` routine, whose `RXSTRING` convention this
    /// boundary does not call.
    ClassicStyle,
    /// A handle the calling activation no longer holds (D5).
    StaleHandle,
    /// The interpreter raised a condition while serving the conversion, and
    /// holds it for the caller to raise in its place.
    Raised,
}

/// A condition the host raised while serving a conversion, which the host
/// holds rather than the boundary.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Raised;

impl From<Raised> for Failure {
    fn from(_: Raised) -> Failure {
        Failure::Raised
    }
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
            // Measured 2026-09-15 against the oracle through `RxCalcSqrt`:
            // `'abc'` answers "Error 88.921" and a precision of `0` "Error
            // 88.905".
            Failure::InvalidDouble { .. } => Some(88921),
            Failure::NotPositive { .. } => Some(88905),
            // Measured 2026-09-15 against the oracle through `orxmethod` and
            // `orxfunction`'s echo methods and routines.
            Failure::NotNonnegative { .. } => Some(88904),
            Failure::OutOfRange { .. } => Some(88907),
            Failure::NotLogical { .. } => Some(34901),
            Failure::NotArray { .. } => Some(98913),
            Failure::NotInstance { .. } => Some(88914),
            Failure::NotPointerString { .. } => Some(88919),
            Failure::NoStem { .. } => Some(if method { 93969 } else { 40919 }),
            // Measured 2026-09-15 against the oracle through a forged routine
            // library: a routine declaring `CSELF` answers "Error 40.918".
            Failure::Signature | Failure::ResultSignature => {
                Some(if method { 93968 } else { 40918 })
            }
            // Measured 2026-09-14 against the oracle: a third argument to
            // `RegularExpression~new` answers "Error 88.922".
            Failure::TooManyArguments { .. } => Some(88922),
            Failure::UnfilledSlot { .. }
            | Failure::StaleHandle
            | Failure::Raised
            | Failure::ClassicStyle => None,
        }
    }
}

/// The classes an argument row requires an instance of.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Class {
    Class,
    Pointer,
    MutableBuffer,
    VariableReference,
}

impl Class {
    /// The class's id, which is what 88.914 names.
    pub fn id(self) -> &'static str {
        match self {
            Class::Class => "Class",
            Class::Pointer => "Pointer",
            Class::MutableBuffer => "MutableBuffer",
            Class::VariableReference => "VariableReference",
        }
    }
}

/// `Numerics::MAX_WHOLENUMBER` (`interpreter/runtime/Numerics.hpp:86`).
pub const MAX_WHOLENUMBER: i64 = 999_999_999_999_999_999;

/// `Numerics::DEFAULT_DIGITS`, the precision a `double` or `float` result is
/// rendered at whatever `NUMERIC DIGITS` is in force: measured, oracle,
/// `TestDoubleArg('0.6666666666666666')` is `0.666666667` under `NUMERIC
/// DIGITS 5` and under `20`.
const RESULT_DIGITS: usize = 9;

impl std::fmt::Display for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Failure::MissingArgument { position } => {
                write!(f, "argument {position} is required")
            }
            Failure::NoStringValue { position } => {
                write!(f, "argument {position} must have a string value")
            }
            Failure::InvalidDouble { position, .. } => {
                write!(f, "argument {position} must be a valid double value")
            }
            Failure::NotPositive { position, .. } => {
                write!(f, "argument {position} must be a positive whole number")
            }
            Failure::NotNonnegative { position, .. } => {
                write!(
                    f,
                    "argument {position} must be zero or a positive whole number"
                )
            }
            Failure::OutOfRange {
                position, min, max, ..
            } => write!(f, "argument {position} must be in the range {min} to {max}"),
            Failure::NotLogical { .. } => write!(f, "logical value must be exactly 0 or 1"),
            Failure::NotArray { .. } => {
                write!(f, "the argument has no single-dimensional array value")
            }
            Failure::NotInstance { position, class } => write!(
                f,
                "argument {position} must be an instance of the {} class",
                class.id()
            ),
            Failure::NotPointerString { position, .. } => {
                write!(f, "argument {position} is not in valid pointer format")
            }
            Failure::NoStem { position, .. } => {
                write!(f, "argument {position} must have a stem object value")
            }
            Failure::ClassicStyle => write!(f, "a call to a ROUTINE_CLASSIC_STYLE routine"),
            Failure::Signature => write!(f, "incorrect signature"),
            Failure::ResultSignature => write!(f, "incorrect signature for the result"),
            Failure::TooManyArguments { expected } => {
                write!(f, "too many arguments in invocation; {expected} expected")
            }
            Failure::UnfilledSlot { entry } => write!(f, "{entry}"),
            Failure::StaleHandle => write!(f, "the handle is no longer held by this activation"),
            Failure::Raised => write!(f, "the interpreter raised a condition while converting"),
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
    /// The union with this value's member written over a zeroed word, which
    /// is how `processArguments` fills the result descriptor
    /// (`NativeActivation.cpp:229`), so every member reads initialised bytes.
    ///
    /// An omitted argument is the zero word itself: the C++ writes
    /// `value_int64_t = 0` for the integer and pointer types and `0.0` for
    /// `double` and `float`, which are the same bytes.
    fn as_union(self) -> ValueUnion {
        let mut word = ValueUnion { value_int64_t: 0 };
        match self {
            Value::Omitted => {}
            Value::Int(v) => word.value_int = v,
            Value::CString(p) => word.value_CSTRING = p,
            Value::Pointer(p) => word.value_POINTER = p,
            Value::Object(h) => word.value_RexxObjectPtr = h,
            Value::Int8(v) => word.value_int8_t = v,
            Value::Int16(v) => word.value_int16_t = v,
            Value::Int32(v) => word.value_int32_t = v,
            Value::Int64(v) => word.value_int64_t = v,
            Value::Uint8(v) => word.value_uint8_t = v,
            Value::Uint16(v) => word.value_uint16_t = v,
            Value::Uint32(v) => word.value_uint32_t = v,
            Value::Uint64(v) => word.value_uint64_t = v,
            Value::Isize(v) => word.value_wholenumber_t = v,
            Value::Usize(v) => word.value_stringsize_t = v,
            Value::Double(v) => word.value_double = v,
            Value::Float(v) => word.value_float = v,
        }
        word
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
    /// The bytes an extension writes a buffer string's value into, by string.
    /// A vector's elements are addressed through its own pointer, so moving
    /// the vector as this list grows does not move them.
    writable: Vec<(ObjRef, Vec<u8>)>,
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
        self.writable.clear();
    }

    /// Makes `length` zero bytes the extension writes `string`'s value into.
    pub fn writable(&mut self, string: ObjRef, length: usize) {
        self.writable.push((string, vec![0; length]));
    }

    /// The length [`CStringPool::writable`] made `string`'s bytes at.
    pub fn writable_length(&self, string: ObjRef) -> Option<usize> {
        self.writable
            .iter()
            .find(|(key, _)| *key == string)
            .map(|(_, bytes)| bytes.len())
    }

    /// The address of `string`'s writable bytes, valid until the pool is
    /// cleared.
    pub fn writable_address(&mut self, string: ObjRef) -> Option<POINTER> {
        self.writable
            .iter_mut()
            .find(|(key, _)| *key == string)
            .map(|(_, bytes)| bytes.as_mut_ptr().cast())
    }

    /// The first `length` bytes written for `string`, at most as many as
    /// were made, which becomes the area's length. The area stays where it
    /// is until the pool is cleared.
    pub fn written(&mut self, string: ObjRef, length: usize) -> Option<Vec<u8>> {
        let (_, bytes) = self.writable.iter_mut().find(|(key, _)| *key == string)?;
        bytes.truncate(length);
        Some(bytes.clone())
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
    ///
    /// # Errors
    /// [`Raised`] where that send raised a condition, which the host keeps.
    fn string_value(&mut self, object: ObjRef) -> Result<Option<ObjRef>, Raised>;

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
    /// (`interpreter/concurrency/Activity.cpp:1846-1849`) patches into the
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

    /// `Numerics::wholenumberToObject` (`interpreter/runtime/Numerics.cpp:184`).
    fn whole_number(&mut self, value: isize) -> ObjRef;

    /// A `.Pointer` wrapping `value`, which is `new_pointer`
    /// (`interpreter/classes/PointerClass.hpp:81`).
    fn new_pointer(&mut self, value: POINTER) -> ObjRef;

    /// The `NUMERIC` settings of the Rexx activation that made the call,
    /// which `NativeActivation::digits`, `fuzz` and `form` answer
    /// (`interpreter/execution/NativeActivation.cpp:2146-2198`).
    fn numeric(&self) -> Numeric;

    /// `object`'s value as a C `double`, or `None` where it has none:
    /// `RexxInternalObject::doubleValue` (`interpreter/classes/ObjectClass.cpp:1101`)
    /// converts the string `requestString` answers, which is a Rexx number or
    /// one of `nan`, `+infinity` and `-infinity` spelled exactly
    /// (`interpreter/classes/StringClass.cpp:510`).
    ///
    /// # Errors
    /// [`Raised`] where the string conversion raised a condition.
    fn double_value(&mut self, object: ObjRef) -> Result<Option<f64>, Raised>;

    /// `object` as a whole number from `min` to `max`, or `None` where it is
    /// not one: `Numerics::objectToSignedInteger`
    /// (`interpreter/runtime/Numerics.cpp:272`), whose string limb is
    /// `NumberString::int64Value` at `Numerics::SIZE_DIGITS`.
    ///
    /// # Errors
    /// [`Raised`] where the string conversion raised a condition.
    fn signed_integer(&mut self, object: ObjRef, min: i64, max: i64)
    -> Result<Option<i64>, Raised>;

    /// `object` as a whole number from zero to `max`, or `None` where it is
    /// not one: `Numerics::objectToUnsignedInteger` (`:364`).
    ///
    /// # Errors
    /// [`Raised`] where the string conversion raised a condition.
    fn unsigned_integer(&mut self, object: ObjRef, max: u64) -> Result<Option<u64>, Raised>;

    /// `object`'s truth value, or where it has none the object that was tested:
    /// `truthValue` (`interpreter/classes/StringClass.cpp:1475`), which tests
    /// the string the conversion answered and reports that string.
    ///
    /// # Errors
    /// [`Raised`] where the string conversion raised a condition.
    fn logical(&mut self, object: ObjRef) -> Result<Result<bool, ObjRef>, Raised>;

    /// `object` as a single-dimensional array, or `None` where it has none:
    /// `requestArray` and the dimension check `arrayArgument` makes
    /// (`interpreter/runtime/MethodArguments.hpp:675`).
    ///
    /// # Errors
    /// [`Raised`] where the conversion raised a condition.
    fn array_value(&mut self, object: ObjRef) -> Result<Option<ObjRef>, Raised>;

    /// Whether `object` is a stem.
    fn is_stem(&self, object: ObjRef) -> bool;

    /// The stem the calling Rexx activation holds under the name `object`'s
    /// string value spells, a period added where it has none, or `None` where
    /// that is not a stem's name: `NativeActivation::getContextStem`
    /// (`interpreter/execution/NativeActivation.cpp:2835`).
    ///
    /// # Errors
    /// [`Raised`] where the string conversion raised a condition.
    fn context_stem(&mut self, object: ObjRef) -> Result<Option<ObjRef>, Raised>;

    /// Whether `object` is an instance of `class` or of a subclass of it.
    fn is_instance_of(&mut self, object: ObjRef, class: Class) -> bool;

    /// The address `object` holds, or `None` where it is not a `.Pointer`.
    fn pointer_value(&self, object: ObjRef) -> Option<POINTER>;

    /// `object`'s `stringValue()`, which sends nothing: a string's bytes, and
    /// the default name for an object whose class has no primitive one.
    fn string_value_text(&mut self, object: ObjRef) -> Vec<u8>;

    /// The running method's receiver.
    fn receiver(&mut self) -> ObjRef;

    /// The running method's scope (`NativeActivation::getScope`, `:2791`).
    fn scope(&mut self) -> ObjRef;

    /// The scope above the running method's in its receiver, `.nil` above the
    /// topmost (`NativeActivation::getSuper`, `:2781`).
    fn super_scope(&mut self) -> ObjRef;

    /// The running call's arguments as an array, an omitted one an empty
    /// position, and the same array however often the call asks
    /// (`NativeActivation::getArguments`, `:2745`).
    fn arguments(&mut self) -> ObjRef;

    /// The name the running method was sent by, or the running routine was
    /// called by.
    fn message_name(&mut self) -> Vec<u8>;

    /// `Numerics::stringsizeToObject` and `uint64ToObject`
    /// (`interpreter/runtime/Numerics.cpp:163`, `:205`).
    fn unsigned_number(&mut self, value: u64) -> ObjRef;

    /// A string object holding `bytes`.
    fn new_string(&mut self, bytes: &[u8]) -> ObjRef;

    /// The number `value` rounded to `precision` digits, which is
    /// `NumberString::newInstanceFromDouble(value, precision)`
    /// (`interpreter/classes/NumberStringClass.cpp:4073`).
    fn double_object(&mut self, value: f64, precision: usize) -> ObjRef;

    /// The local-reference table of the native activation this host is
    /// serving, which is where a handle handed to an extension is registered
    /// and where the collector reads it back from.
    ///
    /// The table lives behind the host rather than beside it (ruling 19,
    /// Moritz 2026-09-14) so that a collection a callback triggers can see
    /// it, and so that an implementation holding the table inside itself
    /// needs no second mutable borrow of that same object. `NativeActivation`
    /// both serves the context and owns the save list
    /// (`interpreter/execution/NativeActivation.hpp:232`).
    fn locals(&mut self) -> &mut Table;

    /// The object `handle` names for the running native call: one of its
    /// local references, or for a host that keeps them, a global reference,
    /// which is the same handle for the same object.
    fn resolve(&mut self, handle: RexxObjectPtr) -> Option<ObjRef> {
        self.locals().resolve(handle)
    }

    /// What the callback tables reach beyond the conversions, or `None` for a
    /// host that serves only the conversions, whose callbacks then refuse.
    fn surface(&mut self) -> Option<&mut dyn Surface> {
        None
    }
}

/// The `NUMERIC` settings a call context reports.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Numeric {
    pub digits: usize,
    pub fuzz: usize,
    /// `NUMERIC FORM ENGINEERING`, the `true` `GetContextForm` answers.
    pub engineering: bool,
}

/// The objects the thread table carries as data rather than as functions,
/// over whatever names them.
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
        let object = cx.host.resolve(value);
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
        cx.host.locals().register(object)
    }

    /// `DoubleToObjectWithPrecision`, registered as
    /// [`Activation::whole_number`]'s answer is.
    pub fn double_object(&self, value: f64, precision: usize) -> RexxObjectPtr {
        let mut cx = self.conversion();
        let object = cx.host.double_object(value, precision);
        cx.host.locals().register(object)
    }

    /// `GetContextDigits`, `GetContextFuzz` and `GetContextForm`.
    pub fn numeric(&self) -> Numeric {
        self.conversion().host.numeric()
    }

    /// `NewPointer`, registered as [`Activation::whole_number`]'s answer is.
    pub fn new_pointer(&self, value: POINTER) -> RexxObjectPtr {
        let mut cx = self.conversion();
        let object = cx.host.new_pointer(value);
        cx.host.locals().register(object)
    }

    /// `StringData`: one address per object for as long as the call lasts, or
    /// a null pointer for a handle this activation does not hold or an object
    /// with no bytes, which is the `NULL` the stub answers on an exception.
    pub fn string_data(&self, handle: RexxObjectPtr) -> CSTRING {
        let mut cx = self.conversion();
        let Some(object) = cx.host.resolve(handle) else {
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
        let mut cx = self.conversion();
        let Some(object) = cx.host.resolve(handle) else {
            return 0;
        };
        cx.host.string_bytes(object).map_or(0, |bytes| bytes.len())
    }

    /// The thread table's data members, each registered as a local reference
    /// so that the handle names a rooted object.
    pub fn constants(&self) -> Constants<RexxObjectPtr> {
        let mut cx = self.conversion();
        let objects = cx.host.constants();
        Constants {
            nil: cx.host.locals().register(objects.nil),
            true_object: cx.host.locals().register(objects.true_object),
            false_object: cx.host.locals().register(objects.false_object),
            null_string: cx.host.locals().register(objects.null_string),
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
    /// [`Source::Special`] for the whole argument list, which a call may then
    /// supply more of than its other rows consume (`usedArglist`,
    /// `NativeActivation.cpp:680`).
    Arguments,
}

/// What an optional argument nobody supplied writes
/// (`NativeActivation.cpp:615`).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Absent {
    Zero,
    Signature,
}

/// How a call's result word is read out of element zero.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ResultRead {
    /// The union member the row's [`Repr`] names.
    Member(Repr),
    /// The bytes the `CSTRING` member points at, copied before the call's
    /// state is used again, or nothing where it is null.
    Text,
}

/// What the stub left in element zero, read as [`ResultRead`] says.
#[derive(Clone, PartialEq, Debug)]
pub enum Written {
    Member(Value),
    Text(Option<Vec<u8>>),
}

type ToNative = fn(&mut Conversion<'_>, ObjRef, usize) -> Result<Value, Failure>;
type FromNative = fn(&mut Conversion<'_>, Value) -> Result<Option<ObjRef>, Failure>;

/// How a row's result comes back.
#[derive(Clone, Copy)]
enum Back {
    /// `valueToObject`'s `default:` (`NativeActivation.cpp:855`), which reads
    /// nothing.
    Refused,
    Member(FromNative),
    Text(FromNative),
}

/// One `REXX_VALUE_*` code's conversion.
struct Row {
    code: u16,
    /// The header's spelling after `REXX_VALUE_`, which the coverage test
    /// names.
    name: &'static str,
    source: Source,
    absent: Absent,
    repr: Repr,
    to_native: ToNative,
    back: Back,
}

const fn argument(
    code: u16,
    name: &'static str,
    repr: Repr,
    to_native: ToNative,
    from_native: FromNative,
) -> Row {
    Row {
        code,
        name,
        source: Source::Argument,
        absent: Absent::Zero,
        repr,
        to_native,
        back: Back::Member(from_native),
    }
}

const fn special(
    code: u16,
    name: &'static str,
    source: Source,
    repr: Repr,
    to_native: ToNative,
) -> Row {
    Row {
        code,
        name,
        source,
        absent: Absent::Signature,
        repr,
        to_native,
        back: Back::Refused,
    }
}

static TABLE: &[Row] = &[
    special(
        code::ARGLIST,
        "ARGLIST",
        Source::Arguments,
        Repr::Object,
        arglist_to_native,
    ),
    special(
        code::NAME,
        "NAME",
        Source::Special,
        Repr::CString,
        name_to_native,
    ),
    special(
        code::SCOPE,
        "SCOPE",
        Source::Special,
        Repr::Object,
        scope_to_native,
    ),
    special(
        code::CSELF,
        "CSELF",
        Source::Special,
        Repr::Pointer,
        cself_to_native,
    ),
    special(
        code::OSELF,
        "OSELF",
        Source::Special,
        Repr::Object,
        oself_to_native,
    ),
    special(
        code::SUPER,
        "SUPER",
        Source::Special,
        Repr::Object,
        super_to_native,
    ),
    argument(
        code::REXX_OBJECT_PTR,
        "RexxObjectPtr",
        Repr::Object,
        object_to_native,
        object_from_native,
    ),
    argument(code::INT, "int", Repr::Int, int_to_native, int_from_native),
    argument(
        code::WHOLENUMBER_T,
        "wholenumber_t",
        Repr::Isize,
        whole_number_to_native,
        isize_from_native,
    ),
    argument(
        code::DOUBLE,
        "double",
        Repr::Double,
        double_to_native,
        double_from_native,
    ),
    Row {
        code: code::CSTRING,
        name: "CSTRING",
        source: Source::Argument,
        absent: Absent::Zero,
        repr: Repr::CString,
        to_native: cstring_to_native,
        back: Back::Text(cstring_from_native),
    },
    argument(
        code::POINTER,
        "POINTER",
        Repr::Pointer,
        pointer_to_native,
        pointer_from_native,
    ),
    argument(
        code::REXX_STRING_OBJECT,
        "RexxStringObject",
        Repr::Object,
        string_object_to_native,
        object_from_native,
    ),
    argument(
        code::STRINGSIZE_T,
        "stringsize_t",
        Repr::Usize,
        string_size_to_native,
        usize_from_native,
    ),
    argument(
        code::FLOAT,
        "float",
        Repr::Float,
        float_to_native,
        float_from_native,
    ),
    argument(
        code::INT8_T,
        "int8_t",
        Repr::Int8,
        int8_to_native,
        int8_from_native,
    ),
    argument(
        code::INT16_T,
        "int16_t",
        Repr::Int16,
        int16_to_native,
        int16_from_native,
    ),
    argument(
        code::INT32_T,
        "int32_t",
        Repr::Int32,
        int32_to_native,
        int32_from_native,
    ),
    argument(
        code::INT64_T,
        "int64_t",
        Repr::Int64,
        int64_to_native,
        int64_from_native,
    ),
    argument(
        code::UINT8_T,
        "uint8_t",
        Repr::Uint8,
        uint8_to_native,
        uint8_from_native,
    ),
    argument(
        code::UINT16_T,
        "uint16_t",
        Repr::Uint16,
        uint16_to_native,
        uint16_from_native,
    ),
    argument(
        code::UINT32_T,
        "uint32_t",
        Repr::Uint32,
        uint32_to_native,
        uint32_from_native,
    ),
    argument(
        code::UINT64_T,
        "uint64_t",
        Repr::Uint64,
        uint64_to_native,
        uint64_from_native,
    ),
    argument(
        code::INTPTR_T,
        "intptr_t",
        Repr::Isize,
        signed_word_to_native,
        isize_from_native,
    ),
    argument(
        code::UINTPTR_T,
        "uintptr_t",
        Repr::Usize,
        unsigned_word_to_native,
        usize_from_native,
    ),
    argument(
        code::LOGICAL_T,
        "logical_t",
        Repr::Usize,
        logical_to_native,
        logical_from_native,
    ),
    argument(
        code::REXX_ARRAY_OBJECT,
        "RexxArrayObject",
        Repr::Object,
        array_to_native,
        object_from_native,
    ),
    argument(
        code::REXX_STEM_OBJECT,
        "RexxStemObject",
        Repr::Object,
        stem_to_native,
        object_from_native,
    ),
    argument(
        code::SIZE_T,
        "size_t",
        Repr::Usize,
        unsigned_word_to_native,
        usize_from_native,
    ),
    argument(
        code::SSIZE_T,
        "ssize_t",
        Repr::Isize,
        signed_word_to_native,
        isize_from_native,
    ),
    argument(
        code::POINTERSTRING,
        "POINTERSTRING",
        Repr::Pointer,
        pointer_string_to_native,
        pointer_string_from_native,
    ),
    argument(
        code::REXX_CLASS_OBJECT,
        "RexxClassObject",
        Repr::Object,
        class_to_native,
        object_from_native,
    ),
    argument(
        code::REXX_MUTABLE_BUFFER_OBJECT,
        "RexxMutableBufferObject",
        Repr::Object,
        mutable_buffer_to_native,
        object_from_native,
    ),
    argument(
        code::POSITIVE_WHOLENUMBER_T,
        "positive_wholenumber_t",
        Repr::Isize,
        positive_whole_number_to_native,
        isize_from_native,
    ),
    argument(
        code::NONNEGATIVE_WHOLENUMBER_T,
        "nonnegative_wholenumber_t",
        Repr::Isize,
        nonnegative_whole_number_to_native,
        isize_from_native,
    ),
    // The header marks this one as never optional (`api/oorexxapi.h:98`) and
    // the C++ leaves it out of the absent switch, so an omitted one is a
    // signature error rather than a zero.
    Row {
        code: code::REXX_VARIABLE_REFERENCE_OBJECT,
        name: "RexxVariableReferenceObject",
        source: Source::Argument,
        absent: Absent::Signature,
        repr: Repr::Object,
        to_native: variable_reference_to_native,
        back: Back::Member(object_from_native),
    },
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

/// How a declared result word's value is read, or `None` for a word
/// `valueToObject` refuses without reading. The word is read as declared,
/// optional bit included, because `valueToObject` switches on it unstripped
/// (`NativeActivation.cpp:720`, `:855-858`).
pub fn result_read(declared: u16) -> Option<ResultRead> {
    let row = row(declared)?;
    match row.back {
        Back::Refused => None,
        Back::Member(_) => Some(ResultRead::Member(row.repr)),
        Back::Text(_) => Some(ResultRead::Text),
    }
}

/// Where the code in `declared` takes its value from, or `None` for a code
/// the table does not know.
pub fn source(declared: u16) -> Option<Source> {
    row(argument_type(declared)).map(|row| row.source)
}

/// Whether the code in `declared` takes its value from the argument list.
///
/// A code the table does not know does: it is `processArguments`' `default:`,
/// which consumes an argument (`NativeActivation.cpp:325-327`, `:672`).
pub fn consumes_argument(declared: u16) -> bool {
    row(argument_type(declared)).is_none_or(|row| row.source == Source::Argument)
}

/// Whether the code in `declared` hands the extension the whole argument
/// list, which lifts the check for arguments the signature does not consume.
pub fn takes_argument_list(declared: u16) -> bool {
    row(argument_type(declared)).is_some_and(|row| row.source == Source::Arguments)
}

/// `argument` converted into the C value `declared` asks for.
///
/// `argument` is the Rexx object at this position, or `None` when the caller
/// supplied nothing there; a row that consumes no argument ignores it either
/// way. `position` is one-based, as the oracle's error inserts are.
pub fn to_native(
    cx: &mut Conversion<'_>,
    declared: u16,
    argument: Option<ObjRef>,
    position: usize,
) -> Result<Converted, Failure> {
    let row = row(argument_type(declared));
    // The absent cases are settled before the per-type conversion, as they
    // are in the C++: a missing required argument and an omitted optional one
    // are both answered without entering the type switch, and a code the
    // table does not know is an argument position like any other until then
    // (`NativeActivation.cpp:607-612`).
    let object = match row.map(|row| row.source) {
        Some(Source::Special | Source::Arguments) => ObjRef::NIL,
        Some(Source::Argument) | None => match argument {
            Some(object) => object,
            None if !is_optional(declared) => {
                return Err(Failure::MissingArgument { position });
            }
            None => {
                return match row.map(|row| row.absent) {
                    Some(Absent::Zero) => Ok(Converted {
                        value: Value::Omitted,
                        flags: 0,
                    }),
                    Some(Absent::Signature) | None => Err(Failure::Signature),
                };
            }
        },
    };
    let Some(row) = row else {
        return Err(Failure::Signature);
    };
    let flags = match row.source {
        Source::Special | Source::Arguments => ARGUMENT_EXISTS | SPECIAL_ARGUMENT,
        Source::Argument => ARGUMENT_EXISTS,
    };
    Ok(Converted {
        value: (row.to_native)(cx, object, position)?,
        flags,
    })
}

/// The Rexx object `value` describes under `declared`, or `None` where the
/// oracle answers `OREF_NULL`.
///
/// A `CSTRING` value is a pointer into `cx`'s pool, where the call put the
/// bytes [`Written::Text`] carried.
pub fn from_native(
    cx: &mut Conversion<'_>,
    declared: u16,
    value: Value,
) -> Result<Option<ObjRef>, Failure> {
    // `valueToObject` treats a zero type as an omitted value rather than a bad
    // one, which is what makes a partly filled argument list convertible. The
    // word is not stripped of its optional bit: the switch reads it as
    // declared (`NativeActivation.cpp:720`).
    if declared == ARGUMENT_TERMINATOR {
        return Ok(None);
    }
    let Some(row) = row(declared) else {
        return Err(Failure::ResultSignature);
    };
    match row.back {
        Back::Refused => Err(Failure::ResultSignature),
        Back::Member(convert) | Back::Text(convert) => convert(cx, value),
    }
}

/// What `sscanf(text, "0x%p", &pointer)` reads
/// (`NativeActivation::pointerString`, `:2049`), or `None` where it converts
/// nothing.
///
/// glibc reads `%p` as `%x` does, through `strtoul`: blanks, a sign, an
/// optional `0x` or `0X`, then hex digits; a value past the range reads as the
/// largest, and a `-` negates any other. It also spells the null pointer
/// `(nil)` and reads that spelling back. Measured, oracle, over the address
/// `TestPointerStringValue` answers: `0x0x<hex>`, `0x-<complement>` and
/// trailing junk convert to it; `0X<hex>`, a leading blank, `0x0x` alone and
/// `0x--1` refuse; `0x0` converts to something else. Measured, `sscanf`
/// itself: `0x(nil)`, `0x (nil)`, `0x(NIL)`, `0x(nil)zz`, `0x0x(nil)` and
/// `0x0X(nil)` read a null pointer, where `0x-(nil)`, `0x+(nil)`, `0x(nil`,
/// `0x( nil)`, `0x(nil )` and `0x0x (nil)` convert nothing.
pub fn pointer_string(text: &[u8]) -> Option<usize> {
    let text = text.split(|byte| *byte == 0).next().unwrap_or_default();
    let mut rest = text.strip_prefix(b"0x")?;
    while let [b' ' | b'\t' | b'\n' | 0x0b | 0x0c | b'\r', tail @ ..] = rest {
        rest = tail;
    }
    let signed = matches!(rest, [b'-' | b'+', ..]);
    let negative = match rest {
        [b'-', tail @ ..] => {
            rest = tail;
            true
        }
        [b'+', tail @ ..] => {
            rest = tail;
            false
        }
        _ => false,
    };
    if !signed && is_nil_spelling(rest) {
        return Some(0);
    }
    if let [b'0', b'x' | b'X', tail @ ..] = rest {
        rest = tail;
        if !rest.first().is_some_and(u8::is_ascii_hexdigit) {
            if !signed && is_nil_spelling(rest) {
                return Some(0);
            }
            return None;
        }
    }
    let digits: Vec<u8> = rest
        .iter()
        .copied()
        .take_while(u8::is_ascii_hexdigit)
        .collect();
    if digits.is_empty() {
        return None;
    }
    let read = digits.iter().try_fold(0usize, |value, digit| {
        let digit = match digit {
            b'0'..=b'9' => digit - b'0',
            b'a'..=b'f' => digit - b'a' + 10,
            _ => digit - b'A' + 10,
        };
        value.checked_mul(16)?.checked_add(usize::from(digit))
    });
    Some(match read {
        None => usize::MAX,
        Some(value) if negative => value.wrapping_neg(),
        Some(value) => value,
    })
}

/// Whether `text` opens with the `(nil)` glibc prints for a null pointer,
/// which it matches without regard to case.
fn is_nil_spelling(text: &[u8]) -> bool {
    matches!(text.get(..b"(nil)".len()), Some(head) if head.eq_ignore_ascii_case(b"(nil)"))
}
