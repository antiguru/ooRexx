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
use rexx_core::{
    BehaviourId, Body, BufferState, Bytes, Decoded, NativeState, ObjRef, ScopePools, VarRef,
    VarRefHome,
};

use crate::Novalue;
use crate::builtin::datatype::{SymbolKind, classify};

use crate::error::{Raised, displayable};
use crate::plan::Package;
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
        self.kept_strings.remove(&string);
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
        let capacity = match bytes.try_reserve_exact(capacity) {
            Ok(()) => capacity,
            Err(_) => 0,
        };
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
        let (address, capacity) = state.writable();
        Some((address.cast(), state.bytes.len(), capacity))
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
        Some(state.writable().0.cast())
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
        scope: Option<ObjRef>,
        arguments: &[Option<ObjRef>],
    ) -> Result<Option<ObjRef>, ()> {
        let caller = self.caller();
        match self.send_message(receiver, name, scope, arguments, caller) {
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

    fn context_variable(&mut self, name: &[u8]) -> Option<ObjRef> {
        let upper = name.to_ascii_uppercase();
        match classify(&upper) {
            SymbolKind::Bad => None,
            // `isString(retriever)`: a constant symbol is its own value.
            SymbolKind::Numeric | SymbolKind::Literal => Some(self.text(&upper)),
            SymbolKind::LiteralDot => self.dot_variable(&upper).ok(),
            SymbolKind::Name => {
                let slot = self.slot_of(&upper);
                let frame = self.activation().frame;
                self.variable(frame, slot)
            }
            SymbolKind::Stem => Some(self.read_stem(&upper)),
            SymbolKind::CompoundName => {
                let (stem, key) = self.compound_parts(&upper);
                match self.stem_get(&stem, &key) {
                    (value, Novalue::Set) => Some(value),
                    _ => None,
                }
            }
        }
    }

    fn set_context_variable(&mut self, name: &[u8], value: ObjRef) {
        let upper = name.to_ascii_uppercase();
        match classify(&upper) {
            SymbolKind::Name => {
                let slot = self.slot_of(&upper);
                let frame = self.activation().frame;
                self.set_variable(frame, slot, value);
            }
            SymbolKind::Stem => self.stem_assign(&upper, value),
            SymbolKind::CompoundName => {
                let (stem, key) = self.compound_parts(&upper);
                self.stem_set(&stem, &key, value);
            }
            _ => {}
        }
    }

    fn drop_context_variable(&mut self, name: &[u8]) {
        let upper = name.to_ascii_uppercase();
        match classify(&upper) {
            SymbolKind::Name => {
                let slot = self.slot_of(&upper);
                let frame = self.activation().frame;
                self.clear_variable(frame, slot);
            }
            SymbolKind::Stem => self.stem_drop(&upper),
            SymbolKind::CompoundName => {
                let (stem, key) = self.compound_parts(&upper);
                self.stem_drop_tail(&stem, &key);
            }
            _ => {}
        }
    }

    fn context_variables(&mut self) -> Option<ObjRef> {
        match crate::dispatch::context::local_variables(self, 0) {
            Ok(directory) => Some(directory),
            Err(failure) => {
                self.hold_native_condition(failure);
                None
            }
        }
    }

    fn object_variable(&mut self, name: &[u8]) -> Option<ObjRef> {
        let frame = self.native_frame();
        let (owner, scope) = (frame.owner, frame.scope);
        let name = super::pool_variable_name(name)?;
        self.pools_of(owner)?.get(scope, &name)
    }

    fn variable_reference(&mut self, name: &[u8], object: bool) -> Option<ObjRef> {
        let upper = name.to_ascii_uppercase();
        if !matches!(classify(&upper), SymbolKind::Name | SymbolKind::Stem) {
            return None;
        }
        let home = if object {
            let frame = self.native_frame();
            VarRefHome::Instance {
                owner: frame.owner,
                scope: frame.scope,
            }
        } else {
            let slot = self.slot_of(&upper);
            let frame = self.activation().frame;
            match self.exposure(frame, slot) {
                Some(var) => VarRefHome::Instance {
                    owner: var.owner,
                    scope: var.scope,
                },
                None => VarRefHome::Cell(self.roots.promote(frame, slot)),
            }
        };
        let reference = self.alloc_with(
            BehaviourId::OBJECT,
            Body::VarRef(Box::new(VarRef {
                name: upper.into(),
                home,
            })),
        );
        self.roots.push_temp(reference);
        Some(reference)
    }

    fn find_class(&mut self, name: &[u8], executable: bool) -> Option<ObjRef> {
        let frame = self.native_frame();
        // A routine a library registered has no package of its own.
        let unpackaged = executable
            && !frame.method
            && frame
                .code
                .is_none_or(|code| self.library_code_package_path(code).is_none());
        if unpackaged {
            let found = self.system_symbol(name)?;
            return self.is_class_object(found).then_some(found);
        }
        self.class_named(name)
    }

    fn environment(&mut self, local: bool) -> Option<ObjRef> {
        let name: &[u8] = if local { b".LOCAL" } else { b".ENVIRONMENT" };
        self.dot_variable(name).ok()
    }

    fn executable(&mut self) -> Option<ObjRef> {
        let frame = self.native_frame();
        let (method, scope, name, code) =
            (frame.method, frame.scope, frame.name.clone(), frame.code);
        let object = if method {
            self.method_executable(scope, &name).ok()?
        } else {
            self.library_routine_object(code?)
        };
        self.roots.push_temp(object);
        Some(object)
    }

    fn caller_context(&mut self) -> Option<ObjRef> {
        self.context_object_at(0)
    }

    fn array_items(&mut self, array: ObjRef) -> Option<Vec<Option<ObjRef>>> {
        self.array_slots_of(array)
    }

    fn load_package(&mut self, name: &[u8]) -> Option<ObjRef> {
        let loaded = self.load_package_global(name);
        let program = self.held(loaded)?;
        Some(self.package_object(Package::Program(program)))
    }

    fn load_package_source(&mut self, name: &[u8], lines: &[Vec<u8>]) -> Option<ObjRef> {
        let loaded = self.package_from_source(name, lines, None);
        let program = self.held(loaded)?;
        Some(self.package_object(Package::Program(program)))
    }

    fn load_library(&mut self, name: &[u8]) -> bool {
        match self.resolve_library(name) {
            crate::LibraryLoad::Loaded(_) => true,
            crate::LibraryLoad::Missing => false,
            crate::LibraryLoad::Version => {
                let raised = Raised::library_version(name);
                self.hold_native_condition(raised.into());
                false
            }
            crate::LibraryLoad::Raised(failure) => {
                self.hold_native_condition(failure);
                false
            }
        }
    }

    fn call_program(&mut self, name: &[u8], arguments: &[Option<ObjRef>]) -> Option<ObjRef> {
        let Some(path) = self.resolve_program_name(None, name, false) else {
            // `Error_Program_unreadable_notfound`
            // (`concurrency/RexxStartDispatcher.cpp:233`).
            let raised = Raised::syntax(3, 901, vec![name.to_vec()]);
            self.hold_native_condition(raised.into());
            return None;
        };
        let routine = self.new_file_executable(path.as_bytes(), true, None);
        let routine = self.held(routine)?;
        self.roots.push_temp(routine);
        let (program, directive) = self.executable_sources.get(&routine)?.routine?;
        let installed = crate::InstalledRoutine { program, directive };
        let answered = self.run_routine_as_program(installed, arguments.to_vec());
        self.held(answered).flatten()
    }

    fn global_reference(&mut self, object: ObjRef) {
        self.global_references.register(object);
    }

    fn allocate_object_memory(&mut self, size: usize) -> Option<POINTER> {
        let buffer = Surface::new_buffer(self, size);
        let mut table = self.object_memory()?;
        table.push(Some(buffer));
        self.set_object_memory(table)?;
        self.native_state_mut(buffer)?.data_address()
    }

    fn free_object_memory(&mut self, pointer: POINTER) {
        let Some(mut table) = self.object_memory() else {
            return;
        };
        table.retain(|buffer| {
            buffer.and_then(|buffer| self.buffer_address(buffer)) != Some(pointer)
        });
        self.set_object_memory(table);
    }

    fn reallocate_object_memory(&mut self, pointer: POINTER, size: usize) -> Option<POINTER> {
        let table = self.object_memory()?;
        let old = table
            .iter()
            .flatten()
            .copied()
            .find(|buffer| self.buffer_address(*buffer) == Some(pointer))?;
        let bytes = self.native_state_mut(old)?.data()?.to_vec();
        if size <= bytes.len() {
            return Some(pointer);
        }
        let grown = Surface::allocate_object_memory(self, size)?;
        let buffer = self.object_memory()?.last().copied().flatten()?;
        if let Some(NativeState::Data(data)) = self.native_state_mut(buffer) {
            data[..bytes.len()].copy_from_slice(&bytes);
        }
        Surface::free_object_memory(self, pointer);
        Some(grown)
    }

    fn register_library(
        &mut self,
        name: &[u8],
        library: Result<Option<rexx_api::load::Library>, rexx_api::load::Refused>,
    ) -> bool {
        if self.libraries.get(name).is_some() {
            return false;
        }
        match self.settle_library(name, library) {
            crate::LibraryLoad::Loaded(_) => true,
            crate::LibraryLoad::Missing => false,
            crate::LibraryLoad::Version => {
                let raised = Raised::library_version(name);
                self.hold_native_condition(raised.into());
                false
            }
            crate::LibraryLoad::Raised(failure) => {
                self.hold_native_condition(failure);
                false
            }
        }
    }
}

impl Interp {
    /// The buffers the running method's receiver keeps for
    /// `AllocateObjectMemory`, which the oracle keeps in the receiver's
    /// `Object`-scope pool under the empty name (`classes/ObjectClass.cpp:2994`),
    /// or `None` outside a method.
    fn object_memory(&mut self) -> Option<Vec<Option<ObjRef>>> {
        let frame = self.native_frame();
        if !frame.method {
            return None;
        }
        let owner = frame.owner;
        let scope = self.classes().lookup("Object")?;
        match self.pools_of(owner).and_then(|pools| pools.get(scope, b"")) {
            Some(table) => self.array_slots_of(table),
            None => Some(Vec::new()),
        }
    }

    /// Replaces [`Interp::object_memory`]'s buffers.
    fn set_object_memory(&mut self, buffers: Vec<Option<ObjRef>>) -> Option<()> {
        let owner = self.native_frame().owner;
        let scope = self.classes().lookup("Object")?;
        let table = Surface::new_array(self, &buffers);
        self.roots.push_temp(table);
        self.set_pool_variable(owner, scope, b"", table);
        Some(())
    }

    /// The address of a `Buffer`'s bytes.
    fn buffer_address(&mut self, buffer: ObjRef) -> Option<POINTER> {
        self.native_state_mut(buffer)?.data_address()
    }

    /// The value `result` carries, or `None` with its condition held on the
    /// running native call.
    fn held<T>(&mut self, result: Result<T, Failure>) -> Option<T> {
        match result {
            Ok(value) => Some(value),
            Err(failure) => {
                self.hold_native_condition(failure);
                None
            }
        }
    }
}

impl Interp {
    /// The class `.name` resolves to from the calling activation, or `None`
    /// where it resolves to something that is not a class.
    fn class_named(&mut self, name: &[u8]) -> Option<ObjRef> {
        let mut dotted = Vec::with_capacity(name.len() + 1);
        dotted.push(b'.');
        dotted.extend_from_slice(name);
        let found = self.dot_variable(&dotted).ok()?;
        self.is_class_object(found).then_some(found)
    }

    /// A compound name's stem, with its period, and its tail resolved as
    /// `VALUE` resolves one: each piece that is a symbol replaced by the
    /// variable's value.
    fn compound_parts(&mut self, upper: &[u8]) -> (Vec<u8>, Vec<u8>) {
        let dot = upper
            .iter()
            .position(|&byte| byte == b'.')
            .expect("a compound name has a period");
        let stem = upper[..=dot].to_vec();
        let key = crate::builtin::datatype::resolve_compound_key(self, &upper[dot + 1..]);
        (stem, key)
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
