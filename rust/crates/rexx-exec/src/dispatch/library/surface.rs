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

//! The [`Surface`] an extension's callbacks reach this interpreter through.

use std::borrow::Cow;

use rexx_api::callbacks::Surface;
use rexx_api::layout::POINTER;
use rexx_core::{BehaviourId, Body, BufferState, Bytes, Decoded, NativeState, ObjRef, ScopePools};

use crate::error::{Raised, displayable};
use crate::{Failure, Interp};

impl Surface for Interp {
    fn clear_condition(&mut self) {
        let frame = self.native_frame_mut();
        frame.raised = None;
        frame.additional = None;
        frame.result = None;
        frame.condition = None;
    }

    fn new_array(&mut self, items: &[Option<ObjRef>]) -> ObjRef {
        self.alloc_with(
            BehaviourId::ARRAY,
            Body::Array {
                dimensions: None,
                slots: items.to_vec(),
            },
        )
    }

    fn raise_exception(&mut self, number: usize, substitutions: Option<ObjRef>) {
        // Each item's `stringValue()`, the default name where that raises,
        // and nothing for an empty slot, which is `messageSubstitution`'s
        // (`interpreter/concurrency/Activity.cpp:1246`).
        let items = substitutions
            .and_then(|array| self.array_slots_of(array))
            .unwrap_or_default();
        let texts: Vec<Vec<u8>> = items
            .into_iter()
            .map(|item| {
                item.map(|object| self.native_found(object))
                    .unwrap_or_default()
            })
            .collect();
        let major = u16::try_from(number / 1000).unwrap_or(u16::MAX);
        let minor = u16::try_from(number % 1000).unwrap_or(0);
        let frame = self.native_frame_mut();
        frame.raised = Some(Raised::syntax(major, minor, texts).into());
        frame.additional = substitutions;
        frame.result = None;
        frame.condition = None;
    }

    fn raise_condition(
        &mut self,
        name: &[u8],
        description: Option<ObjRef>,
        additional: Option<ObjRef>,
        result: Option<ObjRef>,
    ) {
        let description = description.map(|text| self.string_value_text(text));
        let raised = Raised {
            description,
            ..Raised::condition(Cow::Owned(String::from_utf8_lossy(name).into_owned()))
        };
        let frame = self.native_frame_mut();
        frame.raised = Some(raised.into());
        frame.additional = additional;
        frame.result = result;
        frame.condition = None;
    }

    fn has_condition(&mut self) -> bool {
        self.native_frame().raised.is_some()
    }

    fn condition_object(&mut self) -> Option<ObjRef> {
        let frame = self.native_frame();
        if let Some(object) = frame.condition {
            return Some(object);
        }
        let Some(Failure::Raised(raised)) = &frame.raised else {
            return None;
        };
        let (raised, additional, result) = (raised.clone(), frame.additional, frame.result);
        self.pending_additional = additional;
        self.pending_result = result;
        let object = self.build_native_condition_object(&raised).ok()?;
        self.native_frame_mut().condition = Some(object);
        Some(object)
    }

    fn display_condition(&mut self) -> isize {
        let Some(object) = self.condition_object() else {
            return 0;
        };
        let condition = self.condition_entry(object, b"CONDITION");
        if condition
            .map(|name| self.string_value_text(name))
            .as_deref()
            != Some(b"SYNTAX")
        {
            return 0;
        }
        let report = self.condition_report(object);
        self.write_trace_report(&report);
        // `Error_Interpretation / 1000` where `RC` is not a number.
        self.condition_entry(object, b"RC")
            .and_then(|rc| {
                std::str::from_utf8(&self.string_value_text(rc))
                    .ok()?
                    .trim()
                    .parse()
                    .ok()
            })
            .unwrap_or(49)
    }

    fn directory_entry(&mut self, directory: ObjRef, name: &[u8]) -> Option<ObjRef> {
        self.condition_entry(directory, name)
    }

    fn request_string(&mut self, object: ObjRef) -> Option<ObjRef> {
        let string = match self.required_string_value(object) {
            Ok(string) => string,
            Err(failure) => {
                self.hold_native_condition(failure);
                return None;
            }
        };
        if self.string_bytes_of(string) {
            return Some(string);
        }
        let bytes = self.to_text(string).into_owned();
        let string = self.text(&bytes);
        self.roots.push_temp(string);
        Some(string)
    }

    fn is_of_class(&mut self, object: ObjRef, id: &str) -> bool {
        let Some(class) = self.classes().lookup(id) else {
            return false;
        };
        self.class_of_value(object) == Some(class)
    }

    fn is_string(&mut self, object: ObjRef) -> bool {
        // A number is a `RexxInteger` or a `NumberString` in the oracle, which
        // `isString` does not count: measured, `IsString(1+1)` is `0` where
        // `IsString('abc')` is `1`.
        let text = match object.decode() {
            Decoded::Text(_) => true,
            Decoded::Heap { .. } => matches!(
                self.heap.get(object).map(|held| &held.body),
                Some(Body::Text { .. })
            ),
            _ => false,
        };
        text && self.is_of_class(object, "String")
    }

    fn new_raw_string(&mut self, length: usize) -> ObjRef {
        let string = self.alloc_with(
            BehaviourId::STRING,
            Body::Text {
                bytes: Bytes::from_slice(&vec![0; length]),
                num: None,
            },
        );
        self.roots.push_temp(string);
        string
    }

    fn finish_string(&mut self, string: ObjRef, written: &[u8]) {
        if let Some(Body::Text { bytes, num }) =
            self.heap.get_mut(string).map(|held| &mut held.body)
        {
            *bytes = Bytes::from_slice(written);
            *num = None;
        }
    }

    fn new_buffer(&mut self, length: usize) -> ObjRef {
        self.native_state_instance("Buffer", NativeState::Data(vec![0; length]))
    }

    fn buffer_data(&mut self, buffer: ObjRef) -> Option<(POINTER, usize)> {
        let state = self.native_state_mut(buffer)?;
        let length = state.data()?.len();
        Some((state.data_address()?, length))
    }

    fn new_mutable_buffer(&mut self, capacity: usize) -> ObjRef {
        let mut bytes = Vec::new();
        // An allocation this large fails here as `new_buffer` fails in the
        // oracle; the empty buffer is what a failed reservation leaves.
        let _ = bytes.try_reserve_exact(capacity);
        self.native_state_instance(
            "MutableBuffer",
            NativeState::Buffer(BufferState {
                bytes,
                capacity,
                default_size: capacity,
            }),
        )
    }

    fn mutable_buffer(&mut self, buffer: ObjRef) -> Option<(POINTER, usize, usize)> {
        let state = self.buffer_mut(buffer)?;
        Some((
            state.bytes.as_mut_ptr().cast(),
            state.bytes.len(),
            state.capacity,
        ))
    }

    fn set_mutable_buffer_length(&mut self, buffer: ObjRef, length: usize) -> Option<usize> {
        let state = self.buffer_mut(buffer)?;
        // `MutableBuffer::setDataLength`
        // (`interpreter/classes/MutableBufferClass.cpp:264`): capped at the
        // capacity, and padded with NULs where it grows.
        let length = length.min(state.capacity);
        state.bytes.resize(length, 0);
        Some(length)
    }

    fn set_mutable_buffer_capacity(&mut self, buffer: ObjRef, capacity: usize) -> Option<POINTER> {
        let state = self.buffer_mut(buffer)?;
        // `MutableBuffer::setCapacity` (`:292`), which asks `ensureCapacity`
        // for the difference over the capacity and not over the length.
        if capacity > state.capacity {
            let added = capacity - state.capacity;
            let _ = state.ensure_capacity(added);
        }
        Some(state.bytes.as_mut_ptr().cast())
    }

    fn object_cself(&mut self, object: ObjRef, scope: Option<ObjRef>) -> Option<POINTER> {
        let owner = self.pool_owner(object).ok()?;
        let held = match scope {
            None => self.pools_of(owner)?.find(b"CSELF"),
            Some(mut scope) => loop {
                if scope == ObjRef::NIL {
                    break None;
                }
                if let Some(held) = self
                    .pools_of(owner)
                    .and_then(|pools| pools.get(scope, b"CSELF"))
                {
                    break Some(held);
                }
                scope = self.super_scope_of(object, scope)?;
            },
        }?;
        let state = self.native_state_mut(held)?;
        state.pointer().or_else(|| state.data_address())
    }

    fn send(
        &mut self,
        receiver: ObjRef,
        name: &[u8],
        arguments: &[Option<ObjRef>],
    ) -> Result<Option<ObjRef>, ()> {
        let caller = self.caller();
        match self.send_message(receiver, name, None, arguments, caller) {
            Ok(answer) => {
                if let Some(answer) = answer {
                    self.roots.push_temp(answer);
                }
                Ok(answer)
            }
            Err(failure) => {
                self.hold_native_condition(failure);
                Err(())
            }
        }
    }

    fn class_object(&mut self, id: &str) -> Option<ObjRef> {
        self.classes().lookup(id)
    }
}

impl Interp {
    /// Whether `string` is a string's own value, which a tagged integer, an
    /// inline string and a string or number body are.
    fn string_bytes_of(&self, string: ObjRef) -> bool {
        match string.decode() {
            Decoded::SmallInt(_) | Decoded::Text(_) => true,
            Decoded::Heap { .. } => matches!(
                self.heap.get(string).map(|held| &held.body),
                Some(Body::Text { .. } | Body::Num { .. })
            ),
            _ => false,
        }
    }

    /// An instance of the native class `id` carrying `state`, made as the
    /// C++ constructors make one, with no `INIT` sent.
    fn native_state_instance(&mut self, id: &str, state: NativeState) -> ObjRef {
        let class = self
            .classes()
            .lookup(id)
            .unwrap_or_else(|| unreachable!("{id} is a native class"));
        let behaviour = self.classes().instance_behaviour_handle(class);
        let object = self.alloc_with(
            BehaviourId::OBJECT,
            Body::Instance {
                class,
                behaviour,
                name: None,
                pools: ScopePools::new(),
                own: None,
                native: Some(Box::new(state)),
            },
        );
        self.roots.push_temp(object);
        // As `new_instance` arms it for every instance.
        self.reqstr_armed = true;
        object
    }

    /// The native state `object` carries, for a caller that writes it.
    pub(super) fn native_state_mut(&mut self, object: ObjRef) -> Option<&mut NativeState> {
        match &mut self.heap.get_mut(object)?.body {
            Body::Instance {
                native: Some(state),
                ..
            } => Some(state),
            _ => None,
        }
    }
}

impl Interp {
    /// `Activity::display` (`interpreter/concurrency/Activity.cpp:1414`):
    /// the condition object's traceback lines, then the error lines its
    /// entries spell.
    fn condition_report(&mut self, object: ObjRef) -> Vec<u8> {
        let mut report = Vec::new();
        if let Some(traceback) = self.condition_entry(object, b"TRACEBACK") {
            let caller = self.caller();
            let lines = self
                .send_message(traceback, b"MAKEARRAY", None, &[], caller)
                .ok()
                .flatten()
                .and_then(|array| self.array_slots_of(array))
                .unwrap_or_default();
            for line in lines.into_iter().flatten() {
                if line != ObjRef::NIL {
                    report.extend_from_slice(&self.string_value_text(line));
                    report.push(b'\n');
                }
            }
        }
        let entry = |interp: &mut Interp, name: &[u8]| {
            interp
                .condition_entry(object, name)
                .map(|value| interp.string_value_text(value))
        };
        let rc = entry(self, b"RC").unwrap_or_default();
        report.extend_from_slice(b"Error ");
        report.extend_from_slice(&rc);
        if let Some(program) = entry(self, b"PROGRAM").filter(|program| !program.is_empty()) {
            report.extend_from_slice(b" running ");
            report.extend_from_slice(&program);
            if let Some(position) = entry(self, b"POSITION") {
                report.extend_from_slice(b" line ");
                report.extend_from_slice(&position);
            }
        }
        report.extend_from_slice(b":  ");
        report.extend_from_slice(&entry(self, b"ERRORTEXT").unwrap_or_default());
        report.push(b'\n');
        if let Some(message) = entry(self, b"MESSAGE") {
            report.extend_from_slice(b"Error ");
            report.extend_from_slice(&entry(self, b"CODE").unwrap_or_default());
            report.extend_from_slice(b":  ");
            report.extend_from_slice(&message);
            report.push(b'\n');
        }
        displayable(&mut report);
        report
    }
}
