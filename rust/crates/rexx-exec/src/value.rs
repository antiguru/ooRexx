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

//! The value model (D15): four functions every later task manipulates a
//! value through, and the two rules the oracle makes observable everywhere.

use crate::Interp;
use rexx_core::{
    BehaviourId, Body, BufferState, Bytes, Decoded, INLINE_BYTES, InlineText, NotNumeric, ObjRef,
    SMALL_INT_MAX, SMALL_INT_MIN, VarRef, VarRefHome,
};
use rexx_num::{Form, Number};
use std::borrow::Cow;

/// Parses already answered for values whose bytes live in the handle.
pub(crate) struct TextNumbers {
    /// [`ObjRef::NIL`] marks an empty slot. No key stored here can equal it:
    /// every one carries the inline-text tag, which `.nil` does not.
    keys: [ObjRef; TEXT_NUMBERS],
    parsed: [Result<Number, NotNumeric>; TEXT_NUMBERS],
}

/// How many parses [`TextNumbers`] remembers. A power of two, so the slot is
/// a mask rather than a division.
const TEXT_NUMBERS: usize = 32;

impl TextNumbers {
    pub(crate) fn new() -> Self {
        TextNumbers {
            keys: [ObjRef::NIL; TEXT_NUMBERS],
            parsed: std::array::from_fn(|_| Err(NotNumeric)),
        }
    }

    /// Which slot `value` maps to.
    fn slot(value: ObjRef) -> usize {
        let mut mixed = value.bits().wrapping_mul(0xff51_afd7_ed55_8ccd);
        mixed ^= mixed >> 33;
        mixed = mixed.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
        (mixed as usize) & (TEXT_NUMBERS - 1)
    }
}

/// How wide [`Interp::text_scratch`] is.
pub(crate) const TEXT_SCRATCH: usize = 20;
const _: () = assert!(TEXT_SCRATCH >= rexx_core::INLINE_TEXT);

/// Writes `value`'s decimal spelling into the tail of `buffer`, answering the
/// index it starts at.
fn num_rendering<'a>(
    value: &Number,
    created_digits: u32,
    created_form: Form,
    text: &'a mut Option<Vec<u8>>,
) -> &'a [u8] {
    text.get_or_insert_with(|| {
        value
            .format_form(u64::from(created_digits), created_form)
            .into_bytes()
    })
}

fn write_small_int(buffer: &mut [u8; TEXT_SCRATCH], value: i64) -> usize {
    let mut at = buffer.len();
    let mut magnitude = value.unsigned_abs();
    loop {
        at -= 1;
        buffer[at] = b'0' + (magnitude % 10) as u8;
        magnitude /= 10;
        if magnitude == 0 {
            break;
        }
    }
    if value < 0 {
        at -= 1;
        buffer[at] = b'-';
    }
    at
}

impl Interp {
    /// Creates a text value: D15's "a value whose identity is its bytes".
    #[inline]
    pub(crate) fn text(&mut self, bytes: &[u8]) -> ObjRef {
        if let Some(inline) = ObjRef::inline_text(bytes) {
            return inline;
        }
        self.text_bytes(Bytes::from_slice(bytes))
    }

    /// A source literal's value, inlined as a tagged integer when the
    /// literal's own bytes are already exactly what that integer renders as,
    /// and a heap string otherwise.
    pub(crate) fn literal(&mut self, bytes: &[u8]) -> ObjRef {
        // Allocation is the thing being avoided, so the eligibility test may
        // not render a candidate to compare it -- rendering allocates the
        // string this exists to skip. The round trip is therefore stated as a
        // byte pattern and checked, rather than performed.
        if let Some(value) = canonical_small_int(bytes)
            && let Some(handle) = ObjRef::small_int(value)
        {
            return handle;
        }
        self.text(bytes)
    }

    /// [`literal`], for a caller that will keep the handle and read it for the
    /// rest of the run.
    pub(crate) fn interned_literal(&mut self, bytes: &[u8]) -> ObjRef {
        if let Some(value) = canonical_small_int(bytes)
            && let Some(handle) = ObjRef::small_int(value)
        {
            return handle;
        }
        if let Some(inline) = ObjRef::inline_text(bytes) {
            return inline;
        }
        self.alloc_immortal_with(
            BehaviourId::STRING,
            Body::Text {
                bytes: Bytes::from_slice(bytes),
                num: None,
            },
        )
    }

    /// A builtin's counted result -- a length, a position, an index or a
    /// count -- as the inline tagged integer rather than as a heap string of
    /// its own decimal digits.
    pub(crate) fn counted(&mut self, value: usize) -> ObjRef {
        if let Ok(value) = i64::try_from(value)
            && let Some(handle) = ObjRef::small_int(value)
        {
            return handle;
        }
        self.counted_text(value)
    }

    /// A count as a *text* value, for a caller that wants the digits rather
    /// than the tagged integer [`Interp::counted`] prefers.
    pub(crate) fn counted_text(&mut self, value: usize) -> ObjRef {
        match i64::try_from(value) {
            Ok(value) => {
                let mut digits = [0u8; TEXT_SCRATCH];
                let at = write_small_int(&mut digits, value);
                self.text(&digits[at..])
            }
            // Unreachable on a machine whose `usize` is 64 bits wide, and
            // written rather than asserted for `counted`'s own reason.
            Err(_) => self.text(value.to_string().as_bytes()),
        }
    }

    /// [`text`], for a caller that already owns the bytes.
    pub(crate) fn take_result_buffer(&self) -> Vec<u8> {
        let mut buffer = self.result_buffer.take();
        buffer.clear();
        buffer
    }

    /// Hands the result buffer back for the next builtin.
    pub(crate) fn give_result_buffer(&self, buffer: Vec<u8>) {
        self.result_buffer.set(buffer);
    }

    /// A builtin's finished result, keeping the buffer it was built in when
    /// keeping it is free.
    pub(crate) fn integer_text(&mut self, bytes: Vec<u8>, digits: u64) -> ObjRef {
        if let Some(handle) = rendered_small_int(&bytes, digits) {
            self.give_result_buffer(bytes);
            return handle;
        }
        self.text_built(bytes)
    }

    pub(crate) fn text_built(&mut self, bytes: Vec<u8>) -> ObjRef {
        if bytes.len() <= INLINE_BYTES {
            let value = self.text(&bytes);
            self.give_result_buffer(bytes);
            return value;
        }
        self.text_owned(bytes)
    }

    pub(crate) fn text_owned(&mut self, bytes: Vec<u8>) -> ObjRef {
        // **The handle test in front of the `Bytes`**, for the reason
        // [`Interp::text`] gives: a value short enough to live in the handle
        // needs no storage, and `Bytes::from_vec` would copy it into a
        // 54-byte inline buffer that is then discarded. The median string
        // this interpreter builds is three bytes long.
        if let Some(inline) = ObjRef::inline_text(&bytes) {
            return inline;
        }
        self.text_bytes(Bytes::from_vec(bytes))
    }

    /// The one place a `Body::Text` is built, so the `num` cache's initial
    /// state is stated once.
    #[inline]
    pub(crate) fn text_bytes(&mut self, bytes: Bytes) -> ObjRef {
        debug_assert!(
            ObjRef::inline_text(&bytes).is_none(),
            "a value that fits the handle built a Bytes on the way here"
        );
        self.alloc_with(BehaviourId::STRING, Body::Text { bytes, num: None })
    }

    /// Creates a number value, applying D15's `SmallInt` admissibility rule.
    pub(crate) fn number(
        &mut self,
        value: Number,
        created_digits: u32,
        created_form: Form,
    ) -> ObjRef {
        if let Some(small) = small_int_for(&value, created_digits) {
            return ObjRef::small_int(small)
                .expect("small_int_for already checked SMALL_INT_MIN/MAX");
        }
        self.alloc_with(
            BehaviourId::STRING,
            Body::Num {
                value,
                created_digits,
                created_form,
                text: None,
            },
        )
    }

    /// Converts any value to text.
    pub(crate) fn write_text(&mut self, value: ObjRef, out: &mut Vec<u8>) {
        if let Decoded::SmallInt(n) = value.decode() {
            let mut buffer = [0u8; TEXT_SCRATCH];
            let at = write_small_int(&mut buffer, n);
            out.extend_from_slice(&buffer[at..]);
            return;
        }
        out.extend_from_slice(&self.to_text(value));
    }

    /// How many bytes [`to_text`] would answer, for a caller that wants the
    /// count and not the bytes.
    pub(crate) fn text_len(&mut self, value: ObjRef) -> usize {
        let answer = self.text_len_inner(value);
        // The tripwire that makes the duplication above worth having: it runs
        // on **every** value the gate ever measures, including the arms a
        // unit test cannot easily construct, and it is compiled out of
        // release -- where calling `to_text` is exactly the cost this
        // function exists to avoid.
        debug_assert_eq!(
            answer,
            self.to_text(value).len(),
            "text_len disagrees with to_text"
        );
        answer
    }

    /// [`text_len`] itself, with the assertion lifted off it so that the
    /// recursive `Body::Stem` arm below does not re-run it at every hop.
    fn text_len_inner(&mut self, value: ObjRef) -> usize {
        match value.decode() {
            Decoded::Nil => return b"The NIL object".len(),
            Decoded::SmallInt(n) => {
                let at = write_small_int(&mut self.text_scratch, n);
                return TEXT_SCRATCH - at;
            }
            Decoded::Text(inline) => return inline.len(),
            Decoded::Heap { .. } => {}
        }

        let redirect = {
            let Some(object) = self.heap.get(value) else {
                return self.not_in_arena(value).len();
            };
            if matches!(object.body, Body::Class { .. }) {
                return self.not_in_arena(value).len();
            }
            self.redirect_of(&object.body)
        };
        match redirect {
            Redirect::StemDefault(default) => return self.text_len_inner(default),
            Redirect::VarRefValue(referenced) => return self.string_value_text(referenced).len(),
            Redirect::VarRefUnset => {
                return match &self.heap.get(value).expect("a live value").body {
                    Body::VarRef(reference) => reference.name.len(),
                    other => {
                        unreachable!("Redirect::VarRefUnset answers a reference, got {other:?}")
                    }
                };
            }
            // Built rather than measured off the object, because an array's
            // string value exists nowhere until something asks for it -- the
            // same position a tagged integer's digits are in, reached one
            // indirection later.
            Redirect::Array => return self.array_string_of(value).len(),
            // Derived the same way and for the same reason, through the same
            // function `to_text` renders it with.
            Redirect::InstanceDefault(class) => {
                return crate::environment::default_object_name(self.classes().id_string(class))
                    .len();
            }
            Redirect::None => {}
        }

        let object = self.heap.get_mut(value).expect("a live value");
        match &mut object.body {
            Body::Text { bytes, .. } => bytes.len(),
            // Renders and caches, through the same function `to_text` reads.
            // Answering the width from the number's shape instead, so that a
            // number measured and then discarded never renders, was measured
            // and declined -- see this function's doc comment.
            Body::Num {
                value: number,
                created_digits,
                created_form,
                text,
            } => num_rendering(number, *created_digits, *created_form, text).len(),
            Body::Stem { name, .. } => name.len(),
            Body::Native(native) => native.rendered().len(),
            // The contents, the arm `to_text` takes in the same position, and
            // bound once for the borrow reason recorded there.
            Body::Instance {
                native: Some(state),
                name,
                ..
            } => match (state.buffer(), state.pointer()) {
                (Some(buffer), _) => buffer.bytes.len(),
                (_, Some(address)) => rexx_core::pointer_to_string(address).len(),
                _ => match name {
                    Some(bytes) => bytes.len(),
                    None => unreachable!("Redirect::InstanceDefault answers an unnamed instance"),
                },
            },
            // Reached only with a name set, the arm `to_text` takes in the
            // same position: the redirect above answers for an instance that
            // has none.
            Body::Instance { name, .. } => match name {
                Some(bytes) => bytes.len(),
                None => unreachable!("Redirect::InstanceDefault answers an unnamed instance"),
            },
            other => unreachable!(
                "the value model only creates Text, Num, Stem, Array, Native and Instance, \
                 got {other:?}"
            ),
        }
    }

    /// An array's items, each rendered as [`string_value_text`] renders it and
    /// joined by `separator` -- `ArrayClass::toString`
    /// (`classes/ArrayClass.cpp:1856`), which `ArrayClass::makeString`
    /// forwards to with no arguments (`classes/ArrayClass.cpp:1841`) and which
    /// is therefore also what a string context asks an array for, at the
    /// platform line ending.
    pub(crate) fn array_string(&mut self, items: &[Option<ObjRef>], separator: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        let mut first = true;
        for item in items.iter().filter_map(|item| *item) {
            if !first {
                out.extend_from_slice(separator);
            }
            let bytes = self.string_value_text(item);
            out.extend_from_slice(&bytes);
            first = false;
        }
        out
    }

    /// `RexxInternalObject::stringValue()`: the rendering of a value as an
    /// object rather than as text, which is what every `TRACE` value line
    /// prints and what an array's own items are joined out of.
    #[inline(never)]
    pub(crate) fn string_value_text(&mut self, value: ObjRef) -> Vec<u8> {
        if matches!(
            self.heap.get(value).map(|object| &object.body),
            Some(Body::Array { .. })
        ) {
            return crate::dispatch::ARRAY_DEFAULT_NAME.to_vec();
        }
        self.to_text(value).to_vec()
    }

    /// An array's own slots, borrowed, or `None` for a value that is not an
    /// array.
    pub(crate) fn array_slots(&self, value: ObjRef) -> Option<&[Option<ObjRef>]> {
        match &self.heap.get(value)?.body {
            Body::Array { slots, .. } => Some(slots),
            _ => None,
        }
    }

    /// An array's slots and its dimensions array, or `None` for a value that
    /// is not an array.
    pub(crate) fn array_body(
        &self,
        value: ObjRef,
    ) -> Option<(&[Option<ObjRef>], Option<&[usize]>)> {
        match &self.heap.get(value)?.body {
            Body::Array { slots, dimensions } => Some((slots, dimensions.as_deref())),
            _ => None,
        }
    }

    /// A `MutableBuffer`'s state, borrowed, or `None` for a value that is not
    /// one.
    pub(crate) fn buffer(&self, value: ObjRef) -> Option<&BufferState> {
        match &self.heap.get(value)?.body {
            Body::Instance {
                native: Some(state),
                ..
            } => state.buffer(),
            _ => None,
        }
    }

    /// [`Interp::buffer`] for a caller that changes the contents. While the
    /// borrow is live nothing on `self` can allocate, so nothing can collect
    /// under it.
    pub(crate) fn buffer_mut(&mut self, value: ObjRef) -> Option<&mut BufferState> {
        match &mut self.heap.get_mut(value)?.body {
            Body::Instance {
                native: Some(state),
                ..
            } => state.buffer_mut(),
            _ => None,
        }
    }

    /// A `Stream`'s state, borrowed, or `None` for a value that is not one.
    pub(crate) fn stream(&self, value: ObjRef) -> Option<&rexx_core::StreamState> {
        match &self.heap.get(value)?.body {
            Body::Instance {
                native: Some(state),
                ..
            } => state.stream(),
            _ => None,
        }
    }

    /// [`Interp::stream`] for a caller that opens, reads, writes or positions
    /// it.
    pub(crate) fn stream_mut(&mut self, value: ObjRef) -> Option<&mut rexx_core::StreamState> {
        match &mut self.heap.get_mut(value)?.body {
            Body::Instance {
                native: Some(state),
                ..
            } => state.stream_mut(),
            _ => None,
        }
    }

    /// Whether a fixed dimension list makes this array multi-dimensional --
    /// `ArrayClass::isMultiDimensional` (`classes/ArrayClass.hpp:312`), a
    /// dimension list of any length other than one.
    pub(crate) fn is_multi_dimensional_array(&self, value: ObjRef) -> bool {
        matches!(
            self.array_body(value),
            Some((_, Some(dimensions))) if dimensions.len() != 1
        )
    }

    /// [`Interp::array_slots`] as an owned copy, for a caller that goes on to
    /// use `interp`.
    pub(crate) fn array_slots_of(&self, value: ObjRef) -> Option<Vec<Option<ObjRef>>> {
        Some(self.array_slots(value)?.to_vec())
    }

    /// [`array_string`] for an array named by its handle, whose items it looks
    /// up itself.
    fn array_string_of(&mut self, value: ObjRef) -> Vec<u8> {
        // Unreachable as `None`: every caller has just read `Redirect::Array`
        // off this handle's own body, or -- `heap_to_number`'s, which does not
        // go through `Redirect` at all -- has just matched `Body::Array`. An
        // array of no items renders empty, which is what that answers.
        let items = self.array_slots_of(value).unwrap_or_default();
        self.array_string(&items, b"\n")
    }

    #[allow(
        clippy::wrong_self_convention,
        reason = "the interface name is D15's, and `&mut self` is load-bearing \
                   for the lazy cache fill, not a style slip"
    )]
    pub(crate) fn to_text(&mut self, value: ObjRef) -> Cow<'_, [u8]> {
        match value.decode() {
            Decoded::Nil => return Cow::Borrowed(b"The NIL object"),
            // Rendered into the same scratch field the inline arm below uses,
            // for the same reason: `to_string` would allocate a `Vec` per
            // read, and a tagged integer is a value that exists precisely so
            // that it need not be stored anywhere.
            Decoded::SmallInt(n) => {
                let at = write_small_int(&mut self.text_scratch, n);
                return Cow::Borrowed(&self.text_scratch[at..]);
            }
            // Copied into a scratch field and borrowed back out, rather than
            // returned as an owned `Vec`, because a short string is the most
            // common value there is and an allocation here would give back
            // everything inlining them saves.
            Decoded::Text(inline) => {
                let len = inline.len();
                self.text_scratch[..len].copy_from_slice(&inline);
                return Cow::Borrowed(&self.text_scratch[..len]);
            }
            Decoded::Heap { .. } => {}
        }

        let redirect = {
            let Some(object) = self.heap.get(value) else {
                return Cow::Borrowed(self.not_in_arena(value));
            };
            // A class renders its own name rather than anything the value
            // model holds. Since Phase 5j a class resolves like any other
            // object, so this is an arm and no longer rides the `None`.
            if matches!(object.body, Body::Class { .. }) {
                return Cow::Borrowed(self.not_in_arena(value));
            }
            self.redirect_of(&object.body)
        };
        match redirect {
            // `Cow::Owned`: the borrow this recursive call returns is tied
            // to `self`, not to `value`'s own object, so the two Cows
            // cannot share one lifetime.
            Redirect::StemDefault(default) => {
                return Cow::Owned(self.to_text(default).into_owned());
            }
            // **`string_value_text` and not `to_text`**, which is the
            // difference between the referent's `stringValue()` and its
            // string *conversion*, and it is measured rather than reasoned:
            // `RexxObject::requestString` looks `MAKESTRING` up in the
            // receiver's own behaviour and a reference's holds no such name,
            // so the conversion falls through to `stringValue()`. Oracle
            // rc 0, over a `zz` holding a two-item array: `say 'p' >zz` is
            // `p an Array` where `say 'p' zz` is `p one` and `two`, and over
            // an instance whose class defines `makeString`, `say 'p' >k` is
            // the default name where `say 'p' k` runs the method.
            Redirect::VarRefValue(referenced) => {
                return Cow::Owned(self.string_value_text(referenced));
            }
            // A variable holding nothing reads as its own derived name,
            // which for a simple or stem symbol is the reference's own
            // spelling -- measured, `o = >vr` over an unassigned `vr` prints
            // `VR`.
            Redirect::VarRefUnset => {
                return match &self.heap.get(value).expect("a live value").body {
                    Body::VarRef(reference) => Cow::Owned(reference.name.to_vec()),
                    other => {
                        unreachable!("Redirect::VarRefUnset answers a reference, got {other:?}")
                    }
                };
            }
            // An array's string value is built here and stored nowhere, so it
            // is `Cow::Owned` and `try_text` answers `None` for one.
            Redirect::Array => return Cow::Owned(self.array_string_of(value)),
            // Derived from the class id and stored nowhere, the position an
            // array's own string value is in.
            Redirect::InstanceDefault(class) => {
                let id = crate::environment::default_object_name(self.classes().id_string(class));
                return Cow::Owned(id.into_bytes());
            }
            Redirect::None => {}
        }

        let object = self.heap.get_mut(value).expect("a live value");
        match &mut object.body {
            Body::Text { bytes, .. } => Cow::Borrowed(bytes.as_slice()),
            Body::Num {
                value: number,
                created_digits,
                created_form,
                text,
            } => Cow::Borrowed(num_rendering(number, *created_digits, *created_form, text)),
            // Reached for a `Body::Stem` with `default: None` too (the
            // `stem_default` check above only short-circuits the `Some`
            // case), rendering the object's own name.
            Body::Stem { name, .. } => Cow::Borrowed(&**name),
            // `stringValue()`, which for every native class but one is the
            // `~objectName` the object carries. `NativeObject`'s own
            // `string_value` field carries where the two part and why.
            Body::Native(native) => Cow::Borrowed(native.string_value()),
            // `MutableBuffer::stringValue` (`classes/MutableBufferClass.cpp:740`):
            // the contents, whether or not `~objectName=` has named the buffer.
            // **One borrow of the state, not two**: a guard that asked
            // `state.buffer().is_some()` and an arm that asked again holds two
            // shared borrows across a `match` whose scrutinee a later arm
            // moves, which does not compile. A stream's state is not its
            // string value, so it takes the named-instance answer below.
            Body::Instance {
                native: Some(state),
                name,
                ..
            } => match (state.buffer(), state.pointer()) {
                (Some(buffer), _) => Cow::Borrowed(buffer.bytes.as_slice()),
                // `PointerClass::stringValue`
                // (`classes/PointerClass.cpp:152`), held nowhere, which is
                // why `try_text` answers `None` for one.
                (_, Some(address)) => Cow::Owned(rexx_core::pointer_to_string(address)),
                _ => match name {
                    Some(bytes) => Cow::Borrowed(bytes),
                    None => unreachable!("Redirect::InstanceDefault answers an unnamed instance"),
                },
            },
            // Reached only with a name set: the redirect above answers for
            // an instance that has none.
            Body::Instance { name, .. } => match name {
                Some(bytes) => Cow::Borrowed(bytes),
                None => unreachable!("Redirect::InstanceDefault answers an unnamed instance"),
            },
            other => unreachable!(
                "the value model only creates Text, Num, Stem, Array, Native and Instance, \
                 got {other:?}"
            ),
        }
    }

    /// The name `~objectName=` gave an instance, or `None` for one that still
    /// answers its class's default -- and `None` for every other value kind.
    pub(crate) fn instance_name(&self, value: ObjRef) -> Option<Vec<u8>> {
        match &self.heap.get(value)?.body {
            Body::Instance { name, .. } => name.as_ref().map(|bytes| bytes.to_vec()),
            _ => None,
        }
    }

    /// The bytes of `value`, borrowed from the value itself, or `None` when
    /// this value has no bytes anywhere for a shared borrow to reach.
    pub(crate) fn try_text(&self, value: ObjRef) -> Option<&[u8]> {
        match value.decode() {
            Decoded::Nil => return Some(b"The NIL object"),
            Decoded::SmallInt(_) => return None,
            // A third cause of `None`, and the same one as a small integer:
            // the bytes live in the handle, which is a `Copy` local here, so
            // there is nothing outliving this call to borrow from.
            Decoded::Text(_) => return None,
            Decoded::Heap { .. } => {}
        }

        // The class arm `to_text` takes, in the same place and for the same
        // reason.
        let Some(object) = self.heap.get(value) else {
            return Some(self.not_in_arena(value));
        };
        if matches!(object.body, Body::Class { .. }) {
            return Some(self.not_in_arena(value));
        }
        match &object.body {
            Body::Text { bytes, .. } => Some(bytes.as_slice()),
            Body::Num { text, .. } => text.as_deref(),
            Body::Stem {
                default: Some(default),
                ..
            } => self.try_text(*default),
            Body::Stem { name, .. } => Some(name),
            Body::Native(native) => Some(native.rendered()),
            // Joined on demand by `to_text` and held nowhere, which is one of
            // the causes of `None` this function's doc names.
            Body::Array { .. } => None,
            // A buffer's contents, as `to_text` answers them.
            Body::Instance {
                native: Some(state),
                ..
            } => state.buffer().map(|buffer| buffer.bytes.as_slice()),
            // The same cause for an instance nothing has named: `to_text`
            // derives those bytes from the class id and stores them nowhere.
            Body::Instance { name, .. } => name.as_deref(),
            // The variable's own bytes, chased here for the stem default's
            // reason one arm up; an unset variable borrows the reference's
            // own name, which is what it renders as.
            Body::VarRef(reference) => match self.referenced_value(reference) {
                Some(referenced) if self.array_slots(referenced).is_some() => None,
                Some(referenced) => self.try_text(referenced),
                None => Some(&reference.name),
            },
            other => unreachable!(
                "the value model only creates Text, Num, Stem, Array, Native and Instance, \
                 got {other:?}"
            ),
        }
    }

    /// Makes `value`'s bytes reachable by [`try_text`], and carries the ones
    /// that can never be.
    pub(crate) fn render(&mut self, value: ObjRef) -> Rendered {
        // Both of these are checked before `try_text`, because their bytes are
        // neither borrowable nor worth an allocation: they are written into
        // the `Rendered` itself, which the caller already owns.
        match value.decode() {
            Decoded::Text(inline) => {
                return Rendered {
                    value,
                    carried: Carried::Inline(inline),
                };
            }
            // A tagged integer's bytes live nowhere until something asks for
            // them, so `to_text` would leave them in the interpreter's single
            // scratch slot -- a `Cow::Borrowed` that the next call overwrites,
            // which is exactly what a `Rendered` may not hold. Rendered here
            // instead, into storage this value owns.
            Decoded::SmallInt(value_int) => {
                let mut buffer = [0u8; TEXT_SCRATCH];
                let at = write_small_int(&mut buffer, value_int);
                return Rendered {
                    value,
                    carried: Carried::Scratch {
                        buffer,
                        at: at as u8,
                    },
                };
            }
            _ => {}
        }
        if self.try_text(value).is_some() {
            return Rendered {
                value,
                carried: Carried::Borrowable,
            };
        }
        let carried = match self.to_text(value) {
            Cow::Borrowed(_) => Carried::Borrowable,
            Cow::Owned(bytes) => Carried::Owned(bytes),
        };
        Rendered { value, carried }
    }

    /// Converts any value to a `Number`, or `NotNumeric` if it can never be
    /// one.
    // `&mut self` for the same reason `to_text` gives: this lazily fills
    // `Body::Text`'s `num` cache in place, which `to_*` alone would not imply.
    #[allow(
        clippy::wrong_self_convention,
        reason = "the interface name is D15's, and `&mut self` is load-bearing \
                   for the lazy cache fill, not a style slip"
    )]
    #[inline]
    pub(crate) fn to_number(&mut self, value: ObjRef) -> Result<Number, NotNumeric> {
        match value.decode() {
            Decoded::Nil => Err(NotNumeric),
            Decoded::SmallInt(n) => Ok(Number::from_i64(n)),
            Decoded::Text(inline) => self.inline_text_to_number(value, &inline),
            Decoded::Heap { .. } => self.heap_to_number(value),
        }
    }

    /// [`to_number`]'s arm for a value whose bytes are in its own handle,
    /// answered from [`TextNumbers`] when that table has been asked before.
    #[inline(never)]
    fn inline_text_to_number(
        &mut self,
        value: ObjRef,
        inline: &[u8],
    ) -> Result<Number, NotNumeric> {
        let slot = TextNumbers::slot(value);
        if self.text_numbers.keys[slot] == value {
            return self.text_numbers.parsed[slot].clone();
        }
        // Stored before it is handed back, so the parse is moved into the
        // table and only the answer is copied -- filling it the other way
        // round would clone twice for one miss.
        self.text_numbers.keys[slot] = value;
        self.text_numbers.parsed[slot] = Number::parse_bytes(inline).ok_or(NotNumeric);
        self.text_numbers.parsed[slot].clone()
    }

    /// [`to_number`]'s arm for a value that lives in the arena.
    #[inline(never)]
    fn heap_to_number(&mut self, value: ObjRef) -> Result<Number, NotNumeric> {
        // **One lookup, and the stem redirect leaves through it rather than
        // behind it.** The redirect needs its `&mut self` back before it can
        // recurse; copying the default handle out of the arm is what ends the
        // borrow, and it ends it as completely as a separate lookup would.
        // Deciding the redirect first instead costs every value that is *not*
        // a stem with a default -- which is nearly all of them, `Body::Num`
        // above all -- a second walk of the arena to be told so.
        if self.heap.is_class(value) {
            // A class object is not a number. The call is for the tripwire
            // it carries, not for the bytes.
            let _ = self.not_in_arena(value);
            return Err(NotNumeric);
        }
        let Some(object) = self.heap.get_mut(value) else {
            let _ = self.not_in_arena(value);
            return Err(NotNumeric);
        };
        match &mut object.body {
            Body::Num { value, .. } => Ok(value.clone()),
            // Mirrors `to_text`'s own stem redirect above.
            Body::Stem {
                default: Some(default),
                ..
            } => {
                let default = *default;
                self.to_number(default)
            }
            Body::Text { bytes, num } => {
                let bytes = bytes.as_slice();
                let cached = num.get_or_insert_with(|| {
                    Number::parse_bytes(bytes).map(Box::new).ok_or(NotNumeric)
                });
                match cached {
                    Ok(number) => Ok((**number).clone()),
                    Err(marker) => Err(*marker),
                }
            }
            // `VariableReference::numberValue` is the referenced value's
            // (`classes/VariableReference.cpp:262`), which is why `>vr + 1`
            // is `6` and not 41.1 -- measured, oracle rc 0. Read before the
            // borrow is taken again, the way the stem redirect is.
            Body::VarRef(_) => {
                let referenced = match self.as_variable_reference(value) {
                    Some(reference) => self.referenced_value(reference),
                    None => None,
                };
                match referenced {
                    Some(referenced) => self.to_number(referenced),
                    // An unset variable reads as its own name, which parses
                    // as a number only if the name spells one -- it cannot,
                    // since a symbol beginning with a digit is a literal.
                    None => Err(NotNumeric),
                }
            }
            // A `Body::Stem` with `default: None`, the arm the redirect
            // above leaves behind: parses the object's own name, the
            // same fallback `to_text` renders. No cache field
            // exists on `Body::Stem` to hold the parse the way
            // `Body::Text`'s `num` does, so this reparses on every
            // call rather than memoising -- a stem's derived name
            // almost never parses as a number, so there is nothing
            // costly to memoise in the common case, and a cache
            // field added just for this would be new state on
            // `Body::Stem` no other rule needs.
            Body::Stem { name, .. } => Number::parse_bytes(name).ok_or(NotNumeric),
            // A directory's rendering is `a Directory`-shaped text
            // and never numeric, so this answers the marker rather
            // than parsing what `to_text` would produce.
            Body::Native(_) => Err(NotNumeric),
            // **The marker whatever they render as**, because the oracle
            // sends an operator to its left operand as a message and neither
            // answers one: measured, oracle rc 159, `a = (1,); say a + 1` and
            // `o~objectName = '123'; say o + 1` are both
            // `97.1 ... does not understand message "+".` even though each
            // renders as a number, and the comparisons `say (a = 1)` and
            // `say (o = 123)` are both `0` where parsing that rendering gives
            // `1`. `Interp::operator_operand_gap` is what reports them, and
            // it is reached only where this answers `NotNumeric` -- the
            // premise `a_value_the_operator_gap_names_parses_as_no_number`
            // holds.
            Body::Array { .. } | Body::Instance { .. } => Err(NotNumeric),
            other => {
                unreachable!(
                    "the value model only creates Text, Num, Stem, Array, Native and Instance, \
                     got {other:?}"
                )
            }
        }
    }
}

/// The integer a literal's bytes spell, when those bytes are exactly that
/// integer's own rendering.
pub(crate) fn canonical_small_int(bytes: &[u8]) -> Option<i64> {
    let (negative, digits) = match bytes.split_first() {
        Some((b'-', rest)) => (true, rest),
        _ => (false, bytes),
    };
    let (&first, rest) = digits.split_first()?;
    if !first.is_ascii_digit() {
        return None;
    }
    // `-0` is excluded by the same clause that excludes `05`: both render as
    // something other than their own source bytes.
    if first == b'0' && (!rest.is_empty() || negative) {
        return None;
    }
    let mut magnitude = i64::from(first - b'0');
    for &byte in rest {
        if !byte.is_ascii_digit() {
            return None;
        }
        magnitude = magnitude
            .checked_mul(10)?
            .checked_add(i64::from(byte - b'0'))?;
    }
    let value = if negative { -magnitude } else { magnitude };
    (SMALL_INT_MIN..=SMALL_INT_MAX)
        .contains(&value)
        .then_some(value)
}

/// Powers of ten, up to the widest one an `i64` can hold.
const POW10: [u64; 19] = [
    1,
    10,
    100,
    1_000,
    10_000,
    100_000,
    1_000_000,
    10_000_000,
    100_000_000,
    1_000_000_000,
    10_000_000_000,
    100_000_000_000,
    1_000_000_000_000,
    10_000_000_000_000,
    100_000_000_000_000,
    1_000_000_000_000_000,
    10_000_000_000_000_000,
    100_000_000_000_000_000,
    1_000_000_000_000_000_000,
];

/// Whether `value` is written in at most `digits` significant digits, so
/// that rounding it to that precision is the identity.
pub(crate) fn within_digits(value: i64, digits: u64) -> bool {
    digits >= POW10.len() as u64 || value.unsigned_abs() < POW10[digits as usize]
}

/// An exact integer result as a `SmallInt`, when that handle is the same
/// value [`Interp::number`] would have produced for it -- and `None` when it
/// is not, so the caller runs the general path instead.
pub(crate) fn exact_small_int(value: i64, digits: u64) -> Option<ObjRef> {
    within_digits(value, digits)
        .then(|| ObjRef::small_int(value))
        .flatten()
}

/// [`exact_small_int`] asked of a rendering rather than a value: the tag when
/// `rendered` is exactly what it renders back, and `None` when the two would
/// differ.
fn rendered_small_int(rendered: &[u8], digits: u64) -> Option<ObjRef> {
    let value: i64 = str::from_utf8(rendered).ok()?.parse().ok()?;
    (value.to_string().as_bytes() == rendered)
        .then(|| exact_small_int(value, digits))
        .flatten()
}

/// Whether `value` qualifies for the inline `ObjRef::SmallInt` tag under
/// `created_digits`, and its payload if so (D15).
pub(crate) struct Rendered {
    value: ObjRef,
    carried: Carried,
}

/// What [`Interp::render`] had to take a copy of, if anything.
enum Carried {
    /// Nothing: the bytes are where they were and `try_text` reaches them.
    Borrowable,
    /// A rendering that existed nowhere before and does not fit the scratch
    /// arm below -- a stem resolving to a value whose own bytes are already a
    /// copy.
    Owned(Vec<u8>),
    /// Bytes that live in the handle. Copied here rather than allocated,
    /// which is what keeps an inline string free at a call site that needs
    /// several operands' bytes at once.
    Inline(InlineText),
    /// A tagged integer's digits, written into this `Rendered` itself. The
    /// same trade [`Carried::Inline`] makes: the caller already owns this, so
    /// a value that exists only as a tag costs no allocation to read.
    Scratch { buffer: [u8; TEXT_SCRATCH], at: u8 },
}

/// What a value's text has to be read from somewhere other than the object
/// holding it.
#[derive(Copy, Clone)]
enum Redirect {
    /// A stem with a default answers *as* that default.
    StemDefault(ObjRef),
    /// A `>name` reference answers *as* the variable it names, whose value
    /// is read at the moment of the ask rather than captured -- measured on
    /// the oracle, `o = >vr` then `vr = 'changed'` leaves `o~value` reading
    /// `changed`.
    VarRefValue(ObjRef),
    /// The same for a reference to a variable holding nothing, which reads
    /// as its own derived name.
    VarRefUnset,
    /// An array joins its items, which the arm reads back out of the object.
    Array,
    /// An instance nothing has named, which renders as its class's id with an
    /// article in front and stores those bytes nowhere.
    InstanceDefault(ObjRef),
    /// The object's own body holds the text.
    None,
}

impl Interp {
    /// Which value `body` answers *as*, rather than out of its own storage.
    fn redirect_of(&self, body: &Body) -> Redirect {
        match body {
            Body::Stem {
                default: Some(default),
                ..
            } => Redirect::StemDefault(*default),
            Body::Array { .. } => Redirect::Array,
            // A buffer's own body holds the text; a stream's does not -- its
            // string value is the name a Rexx `expose`d variable keeps, so it
            // takes the ordinary instance arms below.
            Body::Instance {
                native: Some(state),
                ..
            } if state.renders_its_own_string_value() => Redirect::None,
            Body::Instance {
                class, name: None, ..
            } => Redirect::InstanceDefault(*class),
            Body::VarRef(reference) => match self.referenced_value(reference) {
                Some(value) => Redirect::VarRefValue(value),
                None => Redirect::VarRefUnset,
            },
            _ => Redirect::None,
        }
    }

    /// The value the variable a reference names currently holds, `None` for
    /// one holding nothing.
    pub(crate) fn referenced_value(&self, reference: &VarRef) -> Option<ObjRef> {
        match reference.home {
            VarRefHome::Cell(cell) => self.roots.slot_value(cell),
            VarRefHome::Instance { owner, scope } => {
                match self.heap.get(owner).map(|object| &object.body) {
                    Some(Body::Instance { pools, .. }) => pools.get(scope, &reference.name),
                    _ => None,
                }
            }
        }
    }

    /// [`Interp::referenced_value`] for a reference named by its handle, with
    /// an unset variable's derived name materialised as a string.
    pub(crate) fn referenced_object(&mut self, value: ObjRef) -> Option<ObjRef> {
        let reference = self.as_variable_reference(value)?;
        match self.referenced_value(reference) {
            Some(referenced) => Some(referenced),
            None => {
                let name = reference.name.to_vec();
                Some(self.text_built(name))
            }
        }
    }

    /// The bytes `value`, a `>name` reference, renders as -- the referenced
    /// variable's `stringValue()`.
    pub(crate) fn as_variable_reference(&self, value: ObjRef) -> Option<&VarRef> {
        match &self.heap.get(value)?.body {
            Body::VarRef(reference) => Some(reference),
            _ => None,
        }
    }
}

impl Rendered {
    /// The bytes, borrowed from the interpreter's heap wherever
    /// [`Interp::render`] left them there.
    pub(crate) fn text<'a>(&'a self, interp: &'a Interp) -> &'a [u8] {
        match &self.carried {
            Carried::Owned(bytes) => bytes,
            Carried::Inline(inline) => inline,
            Carried::Scratch { buffer, at } => &buffer[usize::from(*at)..],
            Carried::Borrowable => interp
                .try_text(self.value)
                .expect("`render` carried no bytes, so it left them borrowable"),
        }
    }
}

/// The bytes a `>name` reference renders as: the referenced variable's
/// `stringValue()`.
pub(crate) fn string_value_of_reference(interp: &mut Interp, value: ObjRef) -> Vec<u8> {
    match interp.referenced_object(value) {
        Some(referenced) => interp.string_value_text(referenced),
        // Unreachable as `None`: the one caller has just matched
        // `Body::VarRef` on this handle.
        None => Vec::new(),
    }
}

fn small_int_for(value: &Number, created_digits: u32) -> Option<i64> {
    let whole = value.rendered_integer(u64::from(created_digits))?;
    (SMALL_INT_MIN..=SMALL_INT_MAX)
        .contains(&whole)
        .then_some(whole)
}

#[cfg(test)]
mod tests;
