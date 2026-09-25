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

use rexx_core::{Body, Bytes, Decoded, Heap, ObjRef};

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
}

impl FakeHost {
    pub(crate) fn new() -> FakeHost {
        FakeHost {
            heap: Heap::new(),
            locals: Table::new(),
            cleared: 0,
            serves: true,
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
    }

    fn new_array(&mut self, items: &[Option<ObjRef>]) -> ObjRef {
        self.heap.alloc(Body::Array {
            dimensions: None,
            slots: items.to_vec(),
        })
    }
}
