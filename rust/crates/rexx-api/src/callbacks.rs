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

//! The callbacks an extension makes through its context tables, over handles
//! and bytes rather than raw pointers.
//!
//! Each answers the value the oracle's stub answers on its failure path where
//! the call cannot be served: a handle this activation does not hold, a
//! condition the interpreter raised while serving it, or a host with no
//! [`Surface`], which records the member as refused.

use rexx_core::ObjRef;

#[cfg(test)]
pub(crate) mod fake;

use crate::layout::{CSTRING, POINTER, RexxObjectPtr, refuse};
use crate::values::{
    self, ARGUMENT_TERMINATOR, Activation, Conversion, Source, Value, is_optional,
};

/// What the callback tables need from the interpreter beyond what the
/// conversions need.
pub trait Surface {
    /// Forgets the condition the running native call holds, which is what
    /// `NativeActivation::clearException` does.
    fn clear_condition(&mut self);

    /// A single-dimensional array of `items`, an absent item an empty slot.
    fn new_array(&mut self, items: &[Option<ObjRef>]) -> ObjRef;

    /// Holds a `SYNTAX` condition numbered `number` (`major * 1000 + minor`)
    /// for the running native call, whose substitutions are the items of the
    /// array `substitutions`, replacing any it held: `reportException`.
    fn raise_exception(&mut self, number: usize, substitutions: Option<ObjRef>);

    /// Holds the condition `name` for the running native call, replacing any
    /// it held: `Activity::raiseCondition`.
    fn raise_condition(
        &mut self,
        name: &[u8],
        description: Option<ObjRef>,
        additional: Option<ObjRef>,
        result: Option<ObjRef>,
    );

    /// Whether the running native call holds a condition.
    fn has_condition(&mut self) -> bool;

    /// The condition object of the condition the running native call holds,
    /// or `None`.
    fn condition_object(&mut self) -> Option<ObjRef>;

    /// `Activity::displayCondition`: writes the report of a held `SYNTAX`
    /// condition and answers its `RC`, or answers zero.
    fn display_condition(&mut self) -> isize;

    /// The entry `name` of the directory `directory`, or `None`.
    fn directory_entry(&mut self, directory: ObjRef, name: &[u8]) -> Option<ObjRef>;

    /// `requestString`: `object`'s string value as a string object, or
    /// `None` where the conversion raised a condition, which the host holds.
    fn request_string(&mut self, object: ObjRef) -> Option<ObjRef>;

    /// Whether `object`'s class is exactly the class `id` names, which is
    /// `isOfClass`.
    fn is_of_class(&mut self, object: ObjRef, id: &str) -> bool;

    /// `isString`: an instance of `String` that is not a number.
    fn is_string(&mut self, object: ObjRef) -> bool;

    /// `raw_string`: a string of `length` zero bytes, which
    /// [`Surface::finish_string`] fills.
    fn new_raw_string(&mut self, length: usize) -> ObjRef;

    /// Replaces the bytes of a string [`Surface::new_raw_string`] made.
    fn finish_string(&mut self, string: ObjRef, bytes: &[u8]);

    /// A `Buffer` of `length` zero bytes.
    fn new_buffer(&mut self, length: usize) -> ObjRef;

    /// The address of a `Buffer`'s bytes and their length, or `None` for an
    /// object that is not one.
    fn buffer_data(&mut self, buffer: ObjRef) -> Option<(POINTER, usize)>;

    /// `new MutableBuffer(length, length)`, empty.
    fn new_mutable_buffer(&mut self, capacity: usize) -> ObjRef;

    /// The address of a `MutableBuffer`'s bytes, its length and its capacity,
    /// or `None` for an object that is not one.
    fn mutable_buffer(&mut self, buffer: ObjRef) -> Option<(POINTER, usize, usize)>;

    /// `MutableBuffer::setDataLength`, answering the length set.
    fn set_mutable_buffer_length(&mut self, buffer: ObjRef, length: usize) -> Option<usize>;

    /// `MutableBuffer::setCapacity`, answering the address of the bytes.
    fn set_mutable_buffer_capacity(&mut self, buffer: ObjRef, capacity: usize) -> Option<POINTER>;

    /// `RexxObject::getCSelf`: the `.Pointer` or `Buffer` an object's `CSELF`
    /// variable holds, unwrapped. `scope` starts the search at that scope and
    /// walks its super scopes; `None` searches every pool, newest first.
    fn object_cself(&mut self, object: ObjRef, scope: Option<ObjRef>) -> Option<POINTER>;

    /// Sends `name` to `receiver`, answering what it answered, or `Err`
    /// where it raised a condition, which the host holds.
    #[allow(clippy::result_unit_err)]
    fn send(
        &mut self,
        receiver: ObjRef,
        name: &[u8],
        arguments: &[Option<ObjRef>],
    ) -> Result<Option<ObjRef>, ()>;

    /// The built-in class `id` names.
    fn class_object(&mut self, id: &str) -> Option<ObjRef>;
}

/// `Error_Incorrect_method_positive` (`api/oorexxerrors.h`), which `ArrayAt`
/// and `ArrayPut` raise for an index of zero.
const INCORRECT_METHOD_POSITIVE: usize = 93_907;

/// What `DecodeConditionInfo` writes into a `RexxCondition`
/// (`Interpreter::decodeConditionData`, `interpreter/runtime/Interpreter.cpp:541`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DecodedCondition {
    pub code: isize,
    pub rc: isize,
    pub position: usize,
    pub name: RexxObjectPtr,
    pub message: RexxObjectPtr,
    pub errortext: RexxObjectPtr,
    pub program: RexxObjectPtr,
    pub description: RexxObjectPtr,
    pub additional: RexxObjectPtr,
}

/// `NumberString::newInstanceFromDouble`'s precision for `DoubleToObject`,
/// which is the native activation's `digits()`, the default nine: measured,
/// oracle, `DoubleToObject(1/3)` is `0.333333333` under `NUMERIC DIGITS 20`.
const DOUBLE_DIGITS: usize = 9;

impl Activation<'_> {
    /// Runs `serve` against the host's [`Surface`], or records `slot` as
    /// refused and answers `refused` where the host has none.
    fn with_surface<R>(
        &self,
        slot: &'static str,
        refused: R,
        serve: impl FnOnce(&mut Conversion<'_>) -> R,
    ) -> R {
        let mut cx = self.conversion();
        if cx.host.surface().is_none() {
            refuse(slot);
            return refused;
        }
        serve(&mut cx)
    }

    /// The object `handle` names in this activation, or `None`.
    fn resolve(&self, handle: RexxObjectPtr) -> Option<ObjRef> {
        self.conversion().host.locals().resolve(handle)
    }

    /// `object` registered as a local reference.
    fn register(&self, object: ObjRef) -> RexxObjectPtr {
        self.conversion().host.locals().register(object)
    }

    /// `Int64ToObject`, `IntptrToObject` and `Int32ToObject`, which are
    /// `wholenumberToObject` for a value that fits a pointer.
    pub fn signed_object(&self, value: i64) -> RexxObjectPtr {
        let mut cx = self.conversion();
        let value = isize::try_from(value).expect("a pointer is 64 bits");
        let object = cx.host.whole_number(value);
        cx.host.locals().register(object)
    }

    /// `UnsignedInt64ToObject`, `UintptrToObject`, `StringSizeToObject` and
    /// `UnsignedInt32ToObject`.
    pub fn unsigned_object(&self, value: u64) -> RexxObjectPtr {
        let mut cx = self.conversion();
        let object = cx.host.unsigned_number(value);
        cx.host.locals().register(object)
    }

    /// `handle`'s value as a whole number from `min` to `max`, which is
    /// `objectToSignedInteger` and `objectToWholeNumber`, or `None`.
    pub fn signed_value(&self, handle: RexxObjectPtr, min: i64, max: i64) -> Option<i64> {
        let object = self.resolve(handle)?;
        let found = self.conversion().host.signed_integer(object, min, max);
        found
            .ok()
            .flatten()
            .filter(|number| (min..=max).contains(number))
    }

    /// `handle`'s value as a whole number from zero to `max`, which is
    /// `objectToUnsignedInteger` and `objectToStringSize`, or `None`.
    pub fn unsigned_value(&self, handle: RexxObjectPtr, max: u64) -> Option<u64> {
        let object = self.resolve(handle)?;
        let found = self.conversion().host.unsigned_integer(object, max);
        found.ok().flatten().filter(|number| *number <= max)
    }

    /// `ObjectToLogical`: `logicalValue`, or `None`.
    pub fn logical_value(&self, handle: RexxObjectPtr) -> Option<bool> {
        let object = self.resolve(handle)?;
        let found = self.conversion().host.logical(object);
        found.ok()?.ok()
    }

    /// `ObjectToDouble`: `doubleValue`, or `None`.
    pub fn double_value(&self, handle: RexxObjectPtr) -> Option<f64> {
        let object = self.resolve(handle)?;
        let found = self.conversion().host.double_value(object);
        found.ok().flatten()
    }

    /// `LogicalToObject`: `.true` for any value but zero.
    pub fn logical_object(&self, value: bool) -> RexxObjectPtr {
        let mut cx = self.conversion();
        let constants = cx.host.constants();
        let object = if value {
            constants.true_object
        } else {
            constants.false_object
        };
        cx.host.locals().register(object)
    }

    /// `DoubleToObject`.
    pub fn double_default_object(&self, value: f64) -> RexxObjectPtr {
        self.double_object(value, DOUBLE_DIGITS)
    }

    /// `NewString` and `NewStringFromAsciiz`.
    pub fn string_object(&self, bytes: &[u8]) -> RexxObjectPtr {
        let mut cx = self.conversion();
        let object = cx.host.new_string(bytes);
        cx.host.locals().register(object)
    }

    /// `valueToObject` over one descriptor an extension filled, whose
    /// `CSTRING` value, where it has one, is `text`.
    fn value_object(&self, declared: u16, value: Value, text: Option<&[u8]>) -> Option<ObjRef> {
        let mut cx = self.conversion();
        let value = match (value, text) {
            (Value::CString(_), Some(text)) => Value::CString(cx.strings.intern(text)),
            (value, _) => value,
        };
        values::from_native(&mut cx, declared, value).ok().flatten()
    }

    /// `ValueToObject`, null where the descriptor converts to nothing.
    pub fn value_to_object(
        &self,
        declared: u16,
        value: Value,
        text: Option<&[u8]>,
    ) -> RexxObjectPtr {
        match self.value_object(declared, value, text) {
            Some(object) => self.register(object),
            None => std::ptr::null_mut(),
        }
    }

    /// `ValuesToObject`: an array of each descriptor's object.
    pub fn values_to_object(&self, described: &[(u16, Value, Option<Vec<u8>>)]) -> RexxObjectPtr {
        // Each is registered as it is made, so that making the next cannot
        // collect it.
        let handles: Vec<RexxObjectPtr> = described
            .iter()
            .map(|(declared, value, text)| self.value_to_object(*declared, *value, text.as_deref()))
            .collect();
        let items: Vec<Option<ObjRef>> =
            handles.iter().map(|handle| self.resolve(*handle)).collect();
        self.with_surface(
            "RexxThreadInterface.ValuesToObject",
            std::ptr::null_mut(),
            |cx| {
                let surface = cx.host.surface().expect("checked by with_surface");
                let array = surface.new_array(&items);
                cx.host.locals().register(array)
            },
        )
    }

    /// `RaiseException0`: a host that serves the callback tables holds the
    /// condition itself, and one that does not records it here.
    pub fn raise_syntax(&self, number: usize) {
        let mut cx = self.conversion();
        match cx.host.surface() {
            Some(surface) => surface.raise_exception(number, None),
            None => {
                drop(cx);
                self.raise(number);
            }
        }
    }

    /// `RaiseException1` and `RaiseException2`: the substitutions as a new
    /// array, a handle this activation does not hold an empty slot.
    pub fn raise_with(&self, slot: &'static str, number: usize, substitutions: &[RexxObjectPtr]) {
        let items: Vec<Option<ObjRef>> = substitutions
            .iter()
            .map(|handle| self.resolve(*handle))
            .collect();
        self.with_surface(slot, (), |cx| {
            let surface = cx.host.surface().expect("checked by with_surface");
            let array = surface.new_array(&items);
            surface.raise_exception(number, Some(array));
        });
    }

    /// `RaiseException`: `substitutions` is the extension's own array.
    pub fn raise_with_array(&self, number: usize, substitutions: RexxObjectPtr) {
        let array = self.resolve(substitutions);
        self.with_surface("RexxThreadInterface.RaiseException", (), |cx| {
            let surface = cx.host.surface().expect("checked by with_surface");
            surface.raise_exception(number, array);
        });
    }

    /// `RaiseCondition`, with `name` upper-cased as `new_upper_string` does.
    pub fn raise_condition(
        &self,
        name: &[u8],
        description: RexxObjectPtr,
        additional: RexxObjectPtr,
        result: RexxObjectPtr,
    ) {
        let (description, additional, result) = (
            self.resolve(description),
            self.resolve(additional),
            self.resolve(result),
        );
        let name = name.to_ascii_uppercase();
        self.with_surface("RexxThreadInterface.RaiseCondition", (), |cx| {
            let surface = cx.host.surface().expect("checked by with_surface");
            surface.raise_condition(&name, description, additional, result);
        });
    }

    /// `CheckCondition`.
    pub fn check_condition(&self) -> bool {
        if self.pending().is_some() {
            return true;
        }
        self.with_surface("RexxThreadInterface.CheckCondition", false, |cx| {
            cx.host
                .surface()
                .expect("checked by with_surface")
                .has_condition()
        })
    }

    /// `ClearCondition`.
    pub fn clear_condition(&self) {
        self.clear_pending();
        self.with_surface("RexxThreadInterface.ClearCondition", (), |cx| {
            cx.host
                .surface()
                .expect("checked by with_surface")
                .clear_condition();
        });
    }

    /// `GetConditionInfo`, null where no condition is held.
    pub fn condition_info(&self) -> RexxObjectPtr {
        self.with_surface(
            "RexxThreadInterface.GetConditionInfo",
            std::ptr::null_mut(),
            |cx| {
                let surface = cx.host.surface().expect("checked by with_surface");
                match surface.condition_object() {
                    Some(object) => cx.host.locals().register(object),
                    None => std::ptr::null_mut(),
                }
            },
        )
    }

    /// `DisplayCondition`.
    pub fn display_condition(&self) -> isize {
        // `Error_Interpretation / 1000`, the stub's own answer where the
        // call fails (`interpreter/api/ThreadContextStubs.cpp:1948`).
        self.with_surface("RexxThreadInterface.DisplayCondition", 49, |cx| {
            cx.host
                .surface()
                .expect("checked by with_surface")
                .display_condition()
        })
    }

    /// `DecodeConditionInfo` over the directory `handle` names, or `None`
    /// for a handle this activation does not hold.
    pub fn decode_condition(&self, handle: RexxObjectPtr) -> Option<DecodedCondition> {
        let directory = self.resolve(handle)?;
        self.with_surface("RexxThreadInterface.DecodeConditionInfo", None, |cx| {
            let mut entry = |name: &[u8]| {
                let object = cx
                    .host
                    .surface()
                    .expect("checked by with_surface")
                    .directory_entry(directory, name)?;
                Some((object, cx.host.locals().register(object)))
            };
            let code = entry(b"CODE");
            let rc = entry(b"RC");
            let position = entry(b"POSITION");
            let handle = |found: Option<(ObjRef, RexxObjectPtr)>| {
                found.map_or(std::ptr::null_mut(), |(_, handle)| handle)
            };
            let decoded = DecodedCondition {
                code: 0,
                rc: 0,
                position: 0,
                name: handle(entry(b"CONDITION")),
                message: handle(entry(b"MESSAGE")),
                errortext: handle(entry(b"ERRORTEXT")),
                program: handle(entry(b"PROGRAM")),
                description: handle(entry(b"DESCRIPTION")),
                additional: handle(entry(b"ADDITIONAL")),
            };
            let text = |found: Option<(ObjRef, RexxObjectPtr)>, cx: &mut Conversion<'_>| {
                let (object, _) = found?;
                let text = cx.host.string_value_text(object);
                Some(text)
            };
            Some(DecodedCondition {
                code: text(code, cx).map_or(0, |code| message_number(&code)),
                rc: text(rc, cx).map_or(0, |rc| message_number(&rc) / 1000),
                position: text(position, cx)
                    .and_then(|position| std::str::from_utf8(&position).ok()?.parse().ok())
                    .unwrap_or(0),
                ..decoded
            })
        })
    }

    /// Runs `serve` against the host's [`Surface`] and the object `handle`
    /// names, answering `refused` for a handle this activation does not
    /// hold, or recording `slot` as refused where the host has no surface.
    fn with_object<R>(
        &self,
        slot: &'static str,
        handle: RexxObjectPtr,
        refused: R,
        serve: impl FnOnce(&mut Conversion<'_>, ObjRef) -> R,
    ) -> R {
        let Some(object) = self.resolve(handle) else {
            return refused;
        };
        self.with_surface(slot, refused, |cx| serve(cx, object))
    }

    /// `ObjectToString`.
    pub fn object_to_string(&self, handle: RexxObjectPtr) -> RexxObjectPtr {
        self.with_object(
            "RexxThreadInterface.ObjectToString",
            handle,
            std::ptr::null_mut(),
            |cx, object| {
                let surface = cx.host.surface().expect("checked by with_surface");
                match surface.request_string(object) {
                    Some(string) => cx.host.locals().register(string),
                    None => std::ptr::null_mut(),
                }
            },
        )
    }

    /// `ObjectToStringValue`: the string value's bytes, at one address per
    /// string for the length of the call.
    pub fn object_to_string_value(&self, handle: RexxObjectPtr) -> CSTRING {
        self.with_object(
            "RexxThreadInterface.ObjectToStringValue",
            handle,
            std::ptr::null(),
            |cx, object| {
                let surface = cx.host.surface().expect("checked by with_surface");
                let Some(string) = surface.request_string(object) else {
                    return std::ptr::null();
                };
                cx.host.locals().register(string);
                let bytes = cx.host.string_value_text(string);
                cx.strings.intern_for(string, &bytes)
            },
        )
    }

    /// A string object's bytes, or `None` for a handle this activation does
    /// not hold or an object that is not a string.
    pub fn string_bytes(&self, handle: RexxObjectPtr) -> Option<Vec<u8>> {
        let object = self.resolve(handle)?;
        let cx = self.conversion();
        cx.host
            .string_bytes(object)
            .map(std::borrow::Cow::into_owned)
    }

    /// `StringUpper` and `StringLower`: the string itself where the case
    /// changes nothing, as `RexxString::upper` answers `this`.
    pub fn string_case(&self, handle: RexxObjectPtr, upper: bool) -> RexxObjectPtr {
        let Some(bytes) = self.string_bytes(handle) else {
            return std::ptr::null_mut();
        };
        let changed = if upper {
            bytes.to_ascii_uppercase()
        } else {
            bytes.to_ascii_lowercase()
        };
        if changed == bytes {
            return handle;
        }
        self.string_object(&changed)
    }

    /// `IsString`.
    pub fn is_string(&self, handle: RexxObjectPtr) -> bool {
        self.with_object(
            "RexxThreadInterface.IsString",
            handle,
            false,
            |cx, object| {
                cx.host
                    .surface()
                    .expect("checked by with_surface")
                    .is_string(object)
            },
        )
    }

    /// `IsDirectory`, `IsStringTable`, `IsArray`, `IsStem`, `IsBuffer`,
    /// `IsPointer`, `IsMutableBuffer` and `IsVariableReference`: whether the
    /// object is an instance of exactly the class `id` names.
    pub fn is_of_class(&self, slot: &'static str, handle: RexxObjectPtr, id: &str) -> bool {
        self.with_object(slot, handle, false, |cx, object| {
            cx.host
                .surface()
                .expect("checked by with_surface")
                .is_of_class(object, id)
        })
    }

    /// `NewBufferString`: a string object to be written through
    /// [`Activation::buffer_string_data`] and then finished.
    pub fn new_buffer_string(&self, length: usize) -> RexxObjectPtr {
        self.with_surface(
            "RexxThreadInterface.NewBufferString",
            std::ptr::null_mut(),
            |cx| {
                let string = cx
                    .host
                    .surface()
                    .expect("checked by with_surface")
                    .new_raw_string(length);
                cx.strings.writable(string, length);
                cx.host.locals().register(string)
            },
        )
    }

    /// `BufferStringLength`: the length the string was made at.
    pub fn buffer_string_length(&self, handle: RexxObjectPtr) -> usize {
        let Some(object) = self.resolve(handle) else {
            return 0;
        };
        let cx = self.conversion();
        match cx.strings.writable_length(object) {
            Some(length) => length,
            None => cx.host.string_bytes(object).map_or(0, |bytes| bytes.len()),
        }
    }

    /// `BufferStringData`: the address the extension writes the string's
    /// bytes through, valid until the call ends.
    pub fn buffer_string_data(&self, handle: RexxObjectPtr) -> POINTER {
        let Some(object) = self.resolve(handle) else {
            return std::ptr::null_mut();
        };
        let mut cx = self.conversion();
        cx.strings
            .writable_address(object)
            .unwrap_or(std::ptr::null_mut())
    }

    /// `FinishBufferString`: the first `length` bytes written become the
    /// string's value, and the answer is the same string.
    pub fn finish_buffer_string(&self, handle: RexxObjectPtr, length: usize) -> RexxObjectPtr {
        let Some(object) = self.resolve(handle) else {
            return std::ptr::null_mut();
        };
        self.with_surface(
            "RexxThreadInterface.FinishBufferString",
            std::ptr::null_mut(),
            |cx| {
                let Some(written) = cx.strings.written(object, length) else {
                    return handle;
                };
                cx.host
                    .surface()
                    .expect("checked by with_surface")
                    .finish_string(object, &written);
                handle
            },
        )
    }

    /// `NewBuffer`.
    pub fn new_buffer(&self, length: usize) -> RexxObjectPtr {
        self.with_surface(
            "RexxThreadInterface.NewBuffer",
            std::ptr::null_mut(),
            |cx| {
                let buffer = cx
                    .host
                    .surface()
                    .expect("checked by with_surface")
                    .new_buffer(length);
                cx.host.locals().register(buffer)
            },
        )
    }

    /// `BufferData` and `BufferLength`.
    pub fn buffer_data(
        &self,
        slot: &'static str,
        handle: RexxObjectPtr,
    ) -> Option<(POINTER, usize)> {
        self.with_object(slot, handle, None, |cx, object| {
            cx.host
                .surface()
                .expect("checked by with_surface")
                .buffer_data(object)
        })
    }

    /// `NewMutableBuffer`.
    pub fn new_mutable_buffer(&self, capacity: usize) -> RexxObjectPtr {
        self.with_surface(
            "RexxThreadInterface.NewMutableBuffer",
            std::ptr::null_mut(),
            |cx| {
                let buffer = cx
                    .host
                    .surface()
                    .expect("checked by with_surface")
                    .new_mutable_buffer(capacity);
                cx.host.locals().register(buffer)
            },
        )
    }

    /// `MutableBufferData`, `MutableBufferLength` and `MutableBufferCapacity`.
    pub fn mutable_buffer(
        &self,
        slot: &'static str,
        handle: RexxObjectPtr,
    ) -> Option<(POINTER, usize, usize)> {
        self.with_object(slot, handle, None, |cx, object| {
            cx.host
                .surface()
                .expect("checked by with_surface")
                .mutable_buffer(object)
        })
    }

    /// `SetMutableBufferLength`.
    pub fn set_mutable_buffer_length(&self, handle: RexxObjectPtr, length: usize) -> usize {
        self.with_object(
            "RexxThreadInterface.SetMutableBufferLength",
            handle,
            0,
            |cx, object| {
                cx.host
                    .surface()
                    .expect("checked by with_surface")
                    .set_mutable_buffer_length(object, length)
                    .unwrap_or(0)
            },
        )
    }

    /// `SetMutableBufferCapacity`.
    pub fn set_mutable_buffer_capacity(&self, handle: RexxObjectPtr, capacity: usize) -> POINTER {
        self.with_object(
            "RexxThreadInterface.SetMutableBufferCapacity",
            handle,
            std::ptr::null_mut(),
            |cx, object| {
                cx.host
                    .surface()
                    .expect("checked by with_surface")
                    .set_mutable_buffer_capacity(object, capacity)
                    .unwrap_or(std::ptr::null_mut())
            },
        )
    }

    /// `GetCSelf`: the running method's `CSELF`, null outside a method.
    pub fn cself(&self) -> POINTER {
        let mut cx = self.conversion();
        if !cx.host.is_method() {
            return std::ptr::null_mut();
        }
        cx.host.cself().unwrap_or(std::ptr::null_mut())
    }

    /// `ObjectToCSelf` and `ObjectToCSelfScoped`.
    pub fn object_cself(&self, handle: RexxObjectPtr, scope: Option<RexxObjectPtr>) -> POINTER {
        let slot = if scope.is_some() {
            "RexxThreadInterface.ObjectToCSelfScoped"
        } else {
            "RexxThreadInterface.ObjectToCSelf"
        };
        let scope = match scope {
            Some(handle) => match self.resolve(handle) {
                Some(scope) => Some(scope),
                None => return std::ptr::null_mut(),
            },
            None => None,
        };
        self.with_object(slot, handle, std::ptr::null_mut(), |cx, object| {
            cx.host
                .surface()
                .expect("checked by with_surface")
                .object_cself(object, scope)
                .unwrap_or(std::ptr::null_mut())
        })
    }

    /// `PointerValue`: the address a `.Pointer` holds, null for anything
    /// else.
    pub fn pointer_value(&self, handle: RexxObjectPtr) -> POINTER {
        let Some(object) = self.resolve(handle) else {
            return std::ptr::null_mut();
        };
        self.conversion()
            .host
            .pointer_value(object)
            .unwrap_or(std::ptr::null_mut())
    }

    /// Sends `name` to the object `receiver` names with `arguments`, the
    /// objects handles and strings name, answering the object it answered,
    /// or `None` where it answered nothing or raised.
    fn send_to(
        &self,
        slot: &'static str,
        receiver: Option<ObjRef>,
        name: &[u8],
        arguments: &[Argument<'_>],
    ) -> Option<ObjRef> {
        let receiver = receiver?;
        let mut objects = Vec::with_capacity(arguments.len());
        for argument in arguments {
            objects.push(match argument {
                Argument::Handle(handle) => self.resolve(*handle),
                Argument::Text(bytes) => {
                    let mut cx = self.conversion();
                    let string = cx.host.new_string(bytes);
                    cx.host.locals().register(string);
                    Some(string)
                }
                Argument::Number(number) => {
                    let mut cx = self.conversion();
                    let value = isize::try_from(*number).unwrap_or(isize::MAX);
                    let object = cx.host.whole_number(value);
                    cx.host.locals().register(object);
                    Some(object)
                }
            });
        }
        self.with_surface(slot, None, |cx| {
            let answer = cx
                .host
                .surface()
                .expect("checked by with_surface")
                .send(receiver, name, &objects)
                .ok()
                .flatten()?;
            cx.host.locals().register(answer);
            Some(answer)
        })
    }

    /// [`Activation::send_to`] answering a handle, null for nothing.
    fn send_for_handle(
        &self,
        slot: &'static str,
        receiver: Option<ObjRef>,
        name: &[u8],
        arguments: &[Argument<'_>],
    ) -> RexxObjectPtr {
        match self.send_to(slot, receiver, name, arguments) {
            Some(object) => self.register(object),
            None => std::ptr::null_mut(),
        }
    }

    /// [`Activation::send_to`] answering a whole number, zero for nothing.
    fn send_for_count(
        &self,
        slot: &'static str,
        receiver: Option<ObjRef>,
        name: &[u8],
        arguments: &[Argument<'_>],
    ) -> usize {
        let Some(answer) = self.send_to(slot, receiver, name, arguments) else {
            return 0;
        };
        let found = self.conversion().host.unsigned_integer(answer, u64::MAX);
        found
            .ok()
            .flatten()
            .and_then(|count| usize::try_from(count).ok())
            .unwrap_or(0)
    }

    /// [`Activation::send_to`] answering its truth value, false for nothing.
    fn send_for_truth(
        &self,
        slot: &'static str,
        receiver: Option<ObjRef>,
        name: &[u8],
        arguments: &[Argument<'_>],
    ) -> bool {
        let Some(answer) = self.send_to(slot, receiver, name, arguments) else {
            return false;
        };
        let found = self.conversion().host.logical(answer);
        matches!(found, Ok(Ok(true)))
    }

    /// The built-in class `id` names.
    fn class(&self, slot: &'static str, id: &str) -> Option<ObjRef> {
        self.with_surface(slot, None, |cx| {
            cx.host
                .surface()
                .expect("checked by with_surface")
                .class_object(id)
        })
    }

    /// The item a collection holds at `index`, or `None` where it holds none,
    /// which the oracle's `get` answers as `OREF_NULL` where the message
    /// answers `.nil`.
    fn item_at(
        &self,
        slot: &'static str,
        collection: RexxObjectPtr,
        index: Argument<'_>,
    ) -> RexxObjectPtr {
        let collection = self.resolve(collection);
        if !self.send_for_truth(slot, collection, b"HASINDEX", &[index]) {
            return std::ptr::null_mut();
        }
        self.send_for_handle(slot, collection, b"AT", &[index])
    }

    /// The item a collection removes from `index`, null where it held none.
    fn item_removed(
        &self,
        slot: &'static str,
        collection: RexxObjectPtr,
        index: Argument<'_>,
    ) -> RexxObjectPtr {
        let collection = self.resolve(collection);
        if !self.send_for_truth(slot, collection, b"HASINDEX", &[index]) {
            return std::ptr::null_mut();
        }
        self.send_for_handle(slot, collection, b"REMOVE", &[index])
    }

    /// Holds `Error_Incorrect_method_positive` for the argument at `position`.
    fn raise_not_positive(&self, slot: &'static str, position: usize) {
        let position = {
            let mut cx = self.conversion();
            let object = cx.host.whole_number(isize::try_from(position).unwrap_or(0));
            cx.host.locals().register(object)
        };
        self.raise_with(slot, INCORRECT_METHOD_POSITIVE, &[position]);
    }

    /// `ArrayAt`: `safeGet`, null beyond the array's size.
    pub fn array_at(&self, array: RexxObjectPtr, index: usize) -> RexxObjectPtr {
        const SLOT: &str = "RexxThreadInterface.ArrayAt";
        if index == 0 {
            self.raise_not_positive(SLOT, 1);
            return std::ptr::null_mut();
        }
        self.item_at(SLOT, array, Argument::Number(index))
    }

    /// `ArrayPut`.
    pub fn array_put(&self, array: RexxObjectPtr, item: RexxObjectPtr, index: usize) {
        const SLOT: &str = "RexxThreadInterface.ArrayPut";
        if index == 0 {
            self.raise_not_positive(SLOT, 2);
            return;
        }
        let array = self.resolve(array);
        self.send_to(
            SLOT,
            array,
            b"PUT",
            &[Argument::Handle(item), Argument::Number(index)],
        );
    }

    /// `ArrayAppend` and `ArrayAppendString`: the index the item landed at.
    pub fn array_append(
        &self,
        slot: &'static str,
        array: RexxObjectPtr,
        item: Argument<'_>,
    ) -> usize {
        let array = self.resolve(array);
        self.send_for_count(slot, array, b"APPEND", &[item])
    }

    /// `ArraySize`, `ArrayItems` and `ArrayDimension`: what `SIZE`, `ITEMS`
    /// and `DIMENSION` answer.
    pub fn array_count(&self, slot: &'static str, array: RexxObjectPtr, name: &[u8]) -> usize {
        let array = self.resolve(array);
        self.send_for_count(slot, array, name, &[])
    }

    /// `NewArray`: an array of that size and no items.
    pub fn new_array_sized(&self, size: usize) -> RexxObjectPtr {
        const SLOT: &str = "RexxThreadInterface.NewArray";
        let class = self.class(SLOT, "Array");
        self.send_for_handle(SLOT, class, b"NEW", &[Argument::Number(size)])
    }

    /// `ArrayOfOne` to `ArrayOfFour`: a null handle an empty slot.
    pub fn array_of(&self, slot: &'static str, items: &[RexxObjectPtr]) -> RexxObjectPtr {
        let items: Vec<Option<ObjRef>> = items.iter().map(|item| self.resolve(*item)).collect();
        self.with_surface(slot, std::ptr::null_mut(), |cx| {
            let array = cx
                .host
                .surface()
                .expect("checked by with_surface")
                .new_array(&items);
            cx.host.locals().register(array)
        })
    }

    /// `NewDirectory`, `NewStringTable` and `NewStem`'s nameless form: the
    /// class's `NEW` with `arguments`.
    pub fn new_instance_of(
        &self,
        slot: &'static str,
        id: &str,
        arguments: &[Argument<'_>],
    ) -> RexxObjectPtr {
        let class = self.class(slot, id);
        self.send_for_handle(slot, class, b"NEW", arguments)
    }

    /// `DirectoryPut` and `StringTablePut`.
    pub fn table_put(
        &self,
        slot: &'static str,
        table: RexxObjectPtr,
        item: RexxObjectPtr,
        index: &[u8],
    ) {
        let table = self.resolve(table);
        self.send_to(
            slot,
            table,
            b"PUT",
            &[Argument::Handle(item), Argument::Text(index)],
        );
    }

    /// `DirectoryAt` and `StringTableAt`.
    pub fn table_at(
        &self,
        slot: &'static str,
        table: RexxObjectPtr,
        index: &[u8],
    ) -> RexxObjectPtr {
        self.item_at(slot, table, Argument::Text(index))
    }

    /// `DirectoryRemove` and `StringTableRemove`.
    pub fn table_remove(
        &self,
        slot: &'static str,
        table: RexxObjectPtr,
        index: &[u8],
    ) -> RexxObjectPtr {
        self.item_removed(slot, table, Argument::Text(index))
    }

    /// `SupplierItem` and `SupplierIndex`.
    pub fn supplier_object(
        &self,
        slot: &'static str,
        supplier: RexxObjectPtr,
        name: &[u8],
    ) -> RexxObjectPtr {
        let supplier = self.resolve(supplier);
        self.send_for_handle(slot, supplier, name, &[])
    }

    /// `SupplierAvailable`.
    pub fn supplier_available(&self, supplier: RexxObjectPtr) -> bool {
        let supplier = self.resolve(supplier);
        self.send_for_truth(
            "RexxThreadInterface.SupplierAvailable",
            supplier,
            b"AVAILABLE",
            &[],
        )
    }

    /// `SupplierNext`.
    pub fn supplier_next(&self, supplier: RexxObjectPtr) {
        let supplier = self.resolve(supplier);
        self.send_to("RexxThreadInterface.SupplierNext", supplier, b"NEXT", &[]);
    }

    /// `NewSupplier`.
    pub fn new_supplier(&self, values: RexxObjectPtr, names: RexxObjectPtr) -> RexxObjectPtr {
        const SLOT: &str = "RexxThreadInterface.NewSupplier";
        let class = self.class(SLOT, "Supplier");
        self.send_for_handle(
            SLOT,
            class,
            b"NEW",
            &[Argument::Handle(values), Argument::Handle(names)],
        )
    }

    /// `SetStemElement` and `SetStemArrayElement`: `tail` as written, which a
    /// compound variable's tail is not upper-cased from.
    pub fn stem_set(
        &self,
        slot: &'static str,
        stem: RexxObjectPtr,
        tail: Argument<'_>,
        value: RexxObjectPtr,
    ) {
        let stem = self.resolve(stem);
        self.send_to(slot, stem, b"[]=", &[Argument::Handle(value), tail]);
    }

    /// `GetStemElement` and `GetStemArrayElement`: null for a tail the stem
    /// holds no value for.
    pub fn stem_get(
        &self,
        slot: &'static str,
        stem: RexxObjectPtr,
        tail: Argument<'_>,
    ) -> RexxObjectPtr {
        self.item_at(slot, stem, tail)
    }

    /// `DropStemElement` and `DropStemArrayElement`.
    pub fn stem_drop(&self, slot: &'static str, stem: RexxObjectPtr, tail: Argument<'_>) {
        let stem = self.resolve(stem);
        self.send_to(slot, stem, b"REMOVE", &[tail]);
    }

    /// `GetAllStemElements` and `GetStemValue`: `TODIRECTORY` and the value
    /// `[]` answers with no tail.
    pub fn stem_whole(
        &self,
        slot: &'static str,
        stem: RexxObjectPtr,
        name: &[u8],
    ) -> RexxObjectPtr {
        let stem = self.resolve(stem);
        self.send_for_handle(slot, stem, name, &[])
    }

    /// `ObjectToValue`: `handle` converted as `declared` asks, or `None`
    /// where it does not convert, with any condition the conversion raised
    /// forgotten (`interpreter/api/ThreadContextStubs.cpp:730`).
    pub fn object_to_value(&self, handle: RexxObjectPtr, declared: u16) -> Option<Value> {
        let object = self.resolve(handle)?;
        // `objectToValue` switches on the word as the extension wrote it, so
        // an optional bit, a special row and an unknown code are its
        // `default:` (`interpreter/execution/NativeActivation.cpp:1170`).
        if is_optional(declared)
            || declared == ARGUMENT_TERMINATOR
            || values::source(declared) != Some(Source::Argument)
        {
            return None;
        }
        self.with_surface(
            "RexxThreadInterface.ObjectToValue",
            None,
            |cx| match values::to_native(cx, declared, Some(object), 1) {
                Ok(converted) => Some(converted.value),
                Err(_) => {
                    cx.host
                        .surface()
                        .expect("checked by with_surface")
                        .clear_condition();
                    None
                }
            },
        )
    }
}

/// `Interpreter::messageNumber` (`interpreter/runtime/Interpreter.cpp:679`):
/// `major.minor` as `major * 1000 + minor`, zero where it is not one.
fn message_number(text: &[u8]) -> isize {
    let text = std::str::from_utf8(text).unwrap_or_default();
    let (major, minor) = text.split_once('.').unwrap_or((text, "0"));
    match (major.parse::<isize>(), minor.parse::<isize>()) {
        (Ok(major), Ok(minor)) if (1..100).contains(&major) && (0..1000).contains(&minor) => {
            major * 1000 + minor
        }
        _ => 0,
    }
}

/// One argument of a message a callback sends on the extension's behalf.
#[derive(Clone, Copy, Debug)]
pub enum Argument<'a> {
    /// An object the extension passed, a null handle an omitted argument.
    Handle(RexxObjectPtr),
    /// A string made from the extension's bytes.
    Text(&'a [u8]),
    /// A whole number.
    Number(usize),
}
