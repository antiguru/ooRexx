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

//! A host over a bare heap that serves the callback tables, for tests that
//! call a slot directly.

use std::borrow::Cow;

use rexx_core::{
    BehaviourHandle, Body, BufferState, Bytes, Decoded, Heap, NativeState, ObjRef, ScopePools,
};

use super::Surface;
use crate::handles::Table;
use crate::layout::POINTER;
use crate::values::{Class, Constants, Host, Numeric, Raised};

/// Stands in for the interpreter: strings are text bodies, numbers are read
/// from their text, and an array is an array body.
pub(crate) struct FakeHost {
    pub(crate) heap: Heap,
    pub(crate) locals: Table,
    /// How often [`Surface::clear_condition`] ran.
    pub(crate) cleared: usize,
    /// Whether [`Host::surface`] answers this host.
    pub(crate) serves: bool,
    /// The condition held: a `SYNTAX` number and its substitution array, or
    /// a condition's name and its description, additional and result.
    pub(crate) held: Option<Held>,
    /// What [`Surface::condition_object`] answers while a condition is held.
    pub(crate) condition: ObjRef,
    /// The entries [`Surface::directory_entry`] answers, by directory.
    pub(crate) entries: Vec<(ObjRef, Vec<u8>, ObjRef)>,
    /// What [`Surface::display_condition`] answers.
    pub(crate) displayed: isize,
    /// The calling activation's variables, by name as the extension spelled
    /// it.
    pub(crate) variables: Vec<(Vec<u8>, ObjRef)>,
    /// Every object [`Surface::global_reference`] was asked to hold.
    pub(crate) globals: Vec<ObjRef>,
}

/// A condition a [`FakeHost`] holds.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Held {
    Syntax(usize, Option<ObjRef>),
    Named(Vec<u8>, [Option<ObjRef>; 3]),
}

impl FakeHost {
    pub(crate) fn new() -> FakeHost {
        FakeHost {
            heap: Heap::new(),
            locals: Table::new(),
            cleared: 0,
            serves: true,
            held: None,
            condition: ObjRef::NIL,
            entries: Vec::new(),
            displayed: 0,
            variables: Vec::new(),
            globals: Vec::new(),
        }
    }

    pub(crate) fn text(&mut self, bytes: &[u8]) -> ObjRef {
        self.heap.alloc(Body::Text {
            bytes: Bytes::from_slice(bytes),
            num: None,
        })
    }

    /// `object`'s bytes, where it is a text body.
    pub(crate) fn bytes(&self, object: ObjRef) -> Option<Vec<u8>> {
        self.string_bytes(object).map(Cow::into_owned)
    }

    /// The items of an array body.
    pub(crate) fn items(&self, object: ObjRef) -> Option<Vec<Option<ObjRef>>> {
        match &self.heap.get(object)?.body {
            Body::Array { slots, .. } => Some(slots.clone()),
            _ => None,
        }
    }

    fn number<T: std::str::FromStr>(&self, object: ObjRef) -> Option<T> {
        if let Decoded::SmallInt(small) = object.decode() {
            return small.to_string().parse().ok();
        }
        String::from_utf8(self.bytes(object)?)
            .ok()?
            .trim()
            .parse()
            .ok()
    }
}

impl Host for FakeHost {
    fn is_method(&self) -> bool {
        true
    }

    fn string_value(&mut self, object: ObjRef) -> Result<Option<ObjRef>, Raised> {
        Ok(Some(object))
    }

    fn string_bytes(&self, object: ObjRef) -> Option<Cow<'_, [u8]>> {
        if let Decoded::SmallInt(small) = object.decode() {
            return Some(Cow::Owned(small.to_string().into_bytes()));
        }
        match &self.heap.get(object)?.body {
            Body::Text { bytes, .. } => Some(Cow::Borrowed(bytes.as_slice())),
            _ => None,
        }
    }

    fn cself(&mut self) -> Option<POINTER> {
        None
    }

    fn constants(&mut self) -> Constants<ObjRef> {
        Constants {
            nil: ObjRef::NIL,
            true_object: ObjRef::small_int(1).expect("one is a small integer"),
            false_object: ObjRef::small_int(0).expect("zero is a small integer"),
            null_string: self.text(b""),
        }
    }

    fn set_object_variable(&mut self, _name: &[u8], _value: Option<ObjRef>) {}

    fn drop_object_variable(&mut self, _name: &[u8]) {}

    fn whole_number(&mut self, value: isize) -> ObjRef {
        match i64::try_from(value).ok().and_then(ObjRef::small_int) {
            Some(object) => object,
            None => self.text(value.to_string().as_bytes()),
        }
    }

    fn new_pointer(&mut self, _value: POINTER) -> ObjRef {
        unreachable!("no test here makes a pointer")
    }

    fn numeric(&self) -> Numeric {
        Numeric {
            digits: 9,
            fuzz: 0,
            engineering: false,
        }
    }

    fn double_value(&mut self, object: ObjRef) -> Result<Option<f64>, Raised> {
        Ok(self.number(object))
    }

    fn signed_integer(
        &mut self,
        object: ObjRef,
        min: i64,
        max: i64,
    ) -> Result<Option<i64>, Raised> {
        Ok(self
            .number(object)
            .filter(|number| (min..=max).contains(number)))
    }

    fn unsigned_integer(&mut self, object: ObjRef, max: u64) -> Result<Option<u64>, Raised> {
        Ok(self.number(object).filter(|number| *number <= max))
    }

    fn logical(&mut self, object: ObjRef) -> Result<Result<bool, ObjRef>, Raised> {
        Ok(match self.number::<i64>(object) {
            Some(0) => Ok(false),
            Some(1) => Ok(true),
            _ => Err(object),
        })
    }

    fn array_value(&mut self, object: ObjRef) -> Result<Option<ObjRef>, Raised> {
        Ok(self.items(object).map(|_| object))
    }

    fn is_stem(&self, _object: ObjRef) -> bool {
        false
    }

    fn context_stem(&mut self, _object: ObjRef) -> Result<Option<ObjRef>, Raised> {
        Ok(None)
    }

    fn is_instance_of(&mut self, _object: ObjRef, _class: Class) -> bool {
        false
    }

    fn pointer_value(&self, _object: ObjRef) -> Option<POINTER> {
        None
    }

    fn string_value_text(&mut self, object: ObjRef) -> Vec<u8> {
        self.bytes(object).unwrap_or_default()
    }

    fn receiver(&mut self) -> ObjRef {
        ObjRef::NIL
    }

    fn scope(&mut self) -> ObjRef {
        ObjRef::NIL
    }

    fn super_scope(&mut self) -> ObjRef {
        ObjRef::NIL
    }

    fn arguments(&mut self) -> ObjRef {
        self.new_array(&[])
    }

    fn message_name(&mut self) -> Vec<u8> {
        b"FAKE".to_vec()
    }

    fn unsigned_number(&mut self, value: u64) -> ObjRef {
        match i64::try_from(value).ok().and_then(ObjRef::small_int) {
            Some(object) => object,
            None => self.text(value.to_string().as_bytes()),
        }
    }

    fn new_string(&mut self, bytes: &[u8]) -> ObjRef {
        self.text(bytes)
    }

    fn double_object(&mut self, value: f64, precision: usize) -> ObjRef {
        self.text(format!("{value} at {precision}").as_bytes())
    }

    fn locals(&mut self) -> &mut Table {
        &mut self.locals
    }

    fn surface(&mut self) -> Option<&mut dyn Surface> {
        if self.serves { Some(self) } else { None }
    }
}

impl Surface for FakeHost {
    fn clear_condition(&mut self) {
        self.cleared += 1;
        self.held = None;
    }

    fn new_array(&mut self, items: &[Option<ObjRef>]) -> ObjRef {
        self.heap.alloc(Body::Array {
            dimensions: None,
            slots: items.to_vec(),
        })
    }

    fn raise_exception(&mut self, number: usize, substitutions: Option<ObjRef>) {
        self.held = Some(Held::Syntax(number, substitutions));
    }

    fn raise_condition(
        &mut self,
        name: &[u8],
        description: Option<ObjRef>,
        additional: Option<ObjRef>,
        result: Option<ObjRef>,
    ) {
        self.held = Some(Held::Named(
            name.to_vec(),
            [description, additional, result],
        ));
    }

    fn has_condition(&mut self) -> bool {
        self.held.is_some()
    }

    fn condition_object(&mut self) -> Option<ObjRef> {
        self.held.as_ref().map(|_| self.condition)
    }

    fn display_condition(&mut self) -> isize {
        self.displayed
    }

    fn directory_entry(&mut self, directory: ObjRef, name: &[u8]) -> Option<ObjRef> {
        self.entries
            .iter()
            .find(|(held, key, _)| *held == directory && key == name)
            .map(|(_, _, value)| *value)
    }

    fn request_string(&mut self, object: ObjRef) -> Option<ObjRef> {
        Some(object)
    }

    fn is_of_class(&mut self, _object: ObjRef, _id: &str) -> bool {
        false
    }

    fn is_string(&mut self, object: ObjRef) -> bool {
        matches!(
            self.heap.get(object).map(|held| &held.body),
            Some(Body::Text { .. })
        )
    }

    fn new_raw_string(&mut self, length: usize) -> ObjRef {
        self.text(&vec![0; length])
    }

    fn finish_string(&mut self, string: ObjRef, written: &[u8]) {
        if let Some(Body::Text { bytes, .. }) = self.heap.get_mut(string).map(|held| &mut held.body)
        {
            *bytes = Bytes::from_slice(written);
        }
    }

    fn new_buffer(&mut self, length: usize) -> ObjRef {
        self.native(NativeState::zeroed(length))
    }

    fn buffer_data(&mut self, buffer: ObjRef) -> Option<(POINTER, usize)> {
        let state = self.state(buffer)?;
        let length = state.data()?.len();
        Some((state.data_address()?, length))
    }

    fn new_mutable_buffer(&mut self, capacity: usize) -> ObjRef {
        self.native(NativeState::Buffer(BufferState {
            bytes: Vec::with_capacity(capacity),
            capacity,
            default_size: capacity,
        }))
    }

    fn mutable_buffer(&mut self, buffer: ObjRef) -> Option<(POINTER, usize, usize)> {
        let state = self.state(buffer)?.buffer_mut()?;
        let (address, capacity) = state.writable();
        Some((address.cast(), state.bytes.len(), capacity))
    }

    fn set_mutable_buffer_length(&mut self, buffer: ObjRef, length: usize) -> Option<usize> {
        let state = self.state(buffer)?.buffer_mut()?;
        let length = length.min(state.capacity);
        state.bytes.resize(length, 0);
        Some(length)
    }

    fn set_mutable_buffer_capacity(&mut self, buffer: ObjRef, capacity: usize) -> Option<POINTER> {
        let state = self.state(buffer)?.buffer_mut()?;
        if capacity > state.capacity {
            state.ensure_capacity(capacity - state.capacity).ok()?;
        }
        Some(state.writable().0.cast())
    }

    fn object_cself(&mut self, _object: ObjRef, _scope: Option<ObjRef>) -> Option<POINTER> {
        None
    }

    fn send(
        &mut self,
        _receiver: ObjRef,
        _name: &[u8],
        _scope: Option<ObjRef>,
        _arguments: &[Option<ObjRef>],
    ) -> Result<Option<ObjRef>, ()> {
        Err(())
    }

    fn class_object(&mut self, _id: &str) -> Option<ObjRef> {
        None
    }

    fn context_variable(&mut self, name: &[u8]) -> Option<ObjRef> {
        self.variables
            .iter()
            .find(|(bound, _)| bound.as_slice() == name)
            .map(|(_, value)| *value)
    }

    fn set_context_variable(&mut self, name: &[u8], value: ObjRef) {
        self.drop_context_variable(name);
        self.variables.push((name.to_vec(), value));
    }

    fn drop_context_variable(&mut self, name: &[u8]) {
        self.variables.retain(|(bound, _)| bound.as_slice() != name);
    }

    fn context_variables(&mut self) -> Option<ObjRef> {
        None
    }

    fn object_variable(&mut self, _name: &[u8]) -> Option<ObjRef> {
        None
    }

    fn variable_reference(&mut self, _name: &[u8], _object: bool) -> Option<ObjRef> {
        None
    }

    fn find_class(&mut self, _name: &[u8], _executable: bool) -> Option<ObjRef> {
        None
    }

    fn environment(&mut self, _local: bool) -> Option<ObjRef> {
        None
    }

    fn executable(&mut self) -> Option<ObjRef> {
        None
    }

    fn caller_context(&mut self) -> Option<ObjRef> {
        None
    }

    fn array_items(&mut self, array: ObjRef) -> Option<Vec<Option<ObjRef>>> {
        self.items(array)
    }

    fn load_package(&mut self, _name: &[u8]) -> Option<ObjRef> {
        None
    }

    fn load_package_source(&mut self, _name: &[u8], _lines: &[Vec<u8>]) -> Option<ObjRef> {
        None
    }

    fn load_library(&mut self, _name: &[u8]) -> bool {
        false
    }

    fn call_program(&mut self, _name: &[u8], _arguments: &[Option<ObjRef>]) -> Option<ObjRef> {
        None
    }

    fn global_reference(&mut self, object: ObjRef) {
        self.globals.push(object);
    }

    fn allocate_object_memory(&mut self, _size: usize) -> Option<POINTER> {
        None
    }

    fn free_object_memory(&mut self, _pointer: POINTER) {}

    fn reallocate_object_memory(&mut self, _pointer: POINTER, _size: usize) -> Option<POINTER> {
        None
    }

    fn register_library(
        &mut self,
        _name: &[u8],
        _library: Result<Option<crate::load::Library>, crate::load::Refused>,
    ) -> bool {
        false
    }
}

impl FakeHost {
    /// An instance carrying `state`.
    fn native(&mut self, state: NativeState) -> ObjRef {
        self.heap.alloc(Body::Instance {
            class: ObjRef::NIL,
            behaviour: BehaviourHandle::new(0),
            name: None,
            pools: ScopePools::new(),
            own: None,
            native: Some(Box::new(state)),
        })
    }

    /// A copy of `object`, as `Object~copy` makes one: its body cloned.
    pub(crate) fn copy(&mut self, object: ObjRef) -> Option<ObjRef> {
        let body = self.heap.get(object)?.body.clone();
        Some(self.heap.alloc(body))
    }

    /// The state an instance carries.
    pub(crate) fn state(&mut self, object: ObjRef) -> Option<&mut NativeState> {
        match &mut self.heap.get_mut(object)?.body {
            Body::Instance {
                native: Some(state),
                ..
            } => Some(state),
            _ => None,
        }
    }
}
