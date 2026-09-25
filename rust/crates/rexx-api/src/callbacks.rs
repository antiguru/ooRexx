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

use crate::layout::{RexxObjectPtr, refuse};
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
