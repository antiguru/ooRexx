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
}

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
