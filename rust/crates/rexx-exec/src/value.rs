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
    fn text_bytes(&mut self, bytes: Bytes) -> ObjRef {
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
            } => match state.buffer() {
                Some(buffer) => buffer.bytes.len(),
                None => match name {
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
            // `~objectName` the object carries -- the two directories were
            // given theirs by the prologue and the rest derive theirs from a
            // class id (`environment.rs` builds both). `NativeObject`'s own
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
            } => match state.buffer() {
                Some(buffer) => Cow::Borrowed(buffer.bytes.as_slice()),
                None => match name {
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
            // A buffer's own body holds the text, named or not.
            // A buffer's own body holds the text; a stream's does not -- its
            // string value is the name a Rexx `expose`d variable keeps, so it
            // takes the ordinary instance arms below.
            Body::Instance {
                native: Some(state),
                ..
            } if state.buffer().is_some() => Redirect::None,
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
mod tests {
    use super::*;
    use rexx_core::INLINE_TEXT;
    use rexx_num::DivOp;

    /// A buffer holding `bytes` at the constructor's default capacity.
    fn buffer_state(bytes: &[u8]) -> BufferState {
        BufferState {
            bytes: bytes.to_vec(),
            capacity: bytes.len().max(256),
            default_size: 256,
        }
    }

    /// `to_text` renders a tagged integer exactly as `i64`'s `Display` does,
    /// and the scratch it renders into is wide enough for every one of them.
    #[test]
    fn a_tagged_integer_renders_as_its_own_display_does() {
        let mut interp = Interp::new();
        for case in [
            0i64,
            1,
            -1,
            9,
            -9,
            10,
            -10,
            99,
            -100,
            1000,
            i64::from(i32::MIN),
            i64::from(u32::MAX),
            SMALL_INT_MAX,
            SMALL_INT_MIN,
        ] {
            let value = ObjRef::small_int(case).expect("inside the tagged range");
            assert_eq!(
                interp.to_text(value).as_ref(),
                case.to_string().as_bytes(),
                "{case}"
            );
        }
        assert_eq!(
            SMALL_INT_MIN.to_string().len(),
            TEXT_SCRATCH,
            "the widest tagged rendering is what fixes `TEXT_SCRATCH`; if this \
             moved, the buffer has slack or is about to overrun"
        );
    }

    /// A remembered parse answers what a fresh one answers, for every
    /// spelling a handle can carry -- numeric and not, at every slot the
    /// table has.
    #[test]
    fn a_remembered_parse_answers_what_a_fresh_one_does() {
        let spellings: Vec<Vec<u8>> = [
            "0", "1", "-1", "7", "05", "+5", " 5 ", "1.1", "2.2", "1.50", "99.7", "1e2", "1E-2",
            "67", "1234567", "-999999", "foobar", "Key", "?", "string", "", "1.2.3", "0x1f", "e",
            ".5", "-.5", "1.",
        ]
        .iter()
        .map(|s| s.as_bytes().to_vec())
        .collect();
        // Every one of them has to fit the handle, or it would reach
        // `heap_to_number` and this would be testing the wrong cache.
        for spelling in &spellings {
            assert!(
                spelling.len() <= INLINE_TEXT,
                "{spelling:?} does not fit a handle"
            );
        }

        let mut interp = Interp::new();
        let mut answered = 0usize;
        for _pass in 0..3 {
            for spelling in &spellings {
                for other in &spellings {
                    let handle =
                        ObjRef::inline_text(other).expect("checked to fit the handle above");
                    let _ = interp.to_number(handle);
                }
                let handle =
                    ObjRef::inline_text(spelling).expect("checked to fit the handle above");
                let expected = Number::parse_bytes(spelling).ok_or(NotNumeric);
                assert_eq!(
                    interp.to_number(handle),
                    expected,
                    "{:?}",
                    String::from_utf8_lossy(spelling)
                );
                answered += 1;
            }
        }
        // A floor, so a grid that stopped generating cases fails rather than
        // passes empty.
        assert_eq!(answered, spellings.len() * 3);
    }

    /// Two spellings that land in the same slot each keep their own answer.
    #[test]
    fn two_spellings_sharing_a_slot_each_keep_their_own_answer() {
        let candidates: Vec<Vec<u8>> = (0..4000u32)
            .map(|n| n.to_string().into_bytes())
            .filter(|s| s.len() <= INLINE_TEXT)
            .collect();
        let mut collision = None;
        'search: for (at, first) in candidates.iter().enumerate() {
            let first_handle = ObjRef::inline_text(first).expect("short enough");
            for second in &candidates[at + 1..] {
                let second_handle = ObjRef::inline_text(second).expect("short enough");
                if TextNumbers::slot(first_handle) == TextNumbers::slot(second_handle) {
                    collision = Some((first_handle, first.clone(), second_handle, second.clone()));
                    break 'search;
                }
            }
        }
        let (first_handle, first, second_handle, second) =
            collision.expect("two of four thousand spellings share one of thirty-two slots");

        let mut interp = Interp::new();
        for _round in 0..3 {
            assert_eq!(
                interp.to_number(first_handle),
                Number::parse_bytes(&first).ok_or(NotNumeric),
                "{:?}",
                String::from_utf8_lossy(&first)
            );
            assert_eq!(
                interp.to_number(second_handle),
                Number::parse_bytes(&second).ok_or(NotNumeric),
                "{:?}",
                String::from_utf8_lossy(&second)
            );
        }
    }

    /// Two reads in a row each answer their own value.
    #[test]
    fn a_second_tagged_read_does_not_show_the_first_ones_tail() {
        let mut interp = Interp::new();
        let long = ObjRef::small_int(SMALL_INT_MIN).expect("inside the tagged range");
        let short = ObjRef::small_int(7).expect("inside the tagged range");
        assert_eq!(
            interp.to_text(long).to_vec(),
            SMALL_INT_MIN.to_string().into_bytes()
        );
        assert_eq!(interp.to_text(short).to_vec(), b"7".to_vec());
    }

    /// `write_text` must append exactly the bytes `to_text` renders, for every
    /// value that reaches its hand-written formatter.
    #[test]
    fn write_text_writes_what_to_text_renders() {
        let mut interp = Interp::new();
        let cases: [i64; 13] = [
            0,
            1,
            -1,
            9,
            -9,
            10,
            -10,
            99,
            -100,
            i64::from(u32::MAX),
            -i64::from(u32::MAX),
            SMALL_INT_MAX,
            SMALL_INT_MIN,
        ];
        for case in cases {
            let value = ObjRef::small_int(case).expect("inside the tagged range");
            let expected = interp.to_text(value).to_vec();
            assert_eq!(
                expected,
                case.to_string().into_bytes(),
                "the case itself must render as its own decimal spelling: {case}"
            );
            let mut out = b"KEY.".to_vec();
            interp.write_text(value, &mut out);
            assert_eq!(
                out,
                [b"KEY.".as_slice(), &expected].concat(),
                "write_text must append what to_text renders, for {case}"
            );
        }
    }

    /// A test-only shorthand: every literal here is a number by construction,
    /// so a parse failure is this test's own bug, not a case to handle.
    fn n(text: &str) -> Number {
        Number::parse(text).expect("test literal parses")
    }

    /// `text_len` answers what `to_text` measures, for every value kind that
    /// can be built here.
    #[test]
    fn text_len_agrees_with_to_text() {
        let mut interp = Interp::new();
        let mut values: Vec<ObjRef> = vec![ObjRef::NIL];
        for case in [0i64, 1, -1, 9, -10, 1000, SMALL_INT_MAX, SMALL_INT_MIN] {
            values.push(ObjRef::small_int(case).expect("inside the tagged range"));
        }
        for spelling in ["", "a", "abcdef", "abcdefg"] {
            values.push(interp.text(spelling.as_bytes()));
        }
        // Past `INLINE_TEXT` and past `INLINE_BYTES`, so both a slot-inlined
        // and a heap-allocated `Body::Text` are measured.
        values.push(interp.text(&[b'q'; INLINE_TEXT + 1]));
        values.push(interp.text(&[b'q'; INLINE_BYTES + 1]));

        let mut numbers = 0usize;
        for spelling in [
            "0",
            "1.50",
            "-1.50",
            "0.05",
            "-0.05",
            "12.4",
            "99.6",
            "1E+9",
            "1E-9",
            "1E+30",
            "1E-30",
            "10E-19",
            "123456789012",
            "-123456789012",
            "0.000012345",
            "123456789012345678901234567890",
        ] {
            for digits in [1u32, 2, 5, 9, 18, 40] {
                for form in [Form::Scientific, Form::Engineering] {
                    let value = interp.number(n(spelling), digits, form);
                    // A `Body::Num`, not a tagged integer, is what this arm
                    // is about -- the tag path is already in the list above.
                    if matches!(value.decode(), Decoded::Heap { .. }) {
                        numbers += 1;
                    }
                    values.push(value);
                }
            }
        }

        // A stem with no default renders its own name, and one with a default
        // renders through it -- a second `text_len` hop, which is the only
        // recursive arm.
        let bare = interp.alloc_with(
            BehaviourId::STEM,
            Body::Stem {
                name: b"Q.".to_vec().into(),
                default: None,
                tails: rexx_core::NameMap::default(),
            },
        );
        values.push(bare);
        let default = interp.number(n("1.50"), 9, Form::Scientific);
        let defaulted = interp.alloc_with(
            BehaviourId::STEM,
            Body::Stem {
                name: b"W.".to_vec().into(),
                default: Some(default),
                tails: rexx_core::NameMap::default(),
            },
        );
        values.push(defaulted);

        // An array's string value is joined on demand and cached nowhere,
        // which is the redirect arm both functions take.
        let joined = interp.text(b"1");
        let array = interp.alloc_with(BehaviourId::ARRAY, Body::array(vec![Some(joined), None]));
        values.push(array);

        // Both shapes of instance, because `Redirect::of` answers only for
        // the unnamed one: the named one reaches the body match, which is the
        // half of the mirror a redirect arm cannot stand in for. A buffer in
        // each shape, because it takes neither: its own arm answers the
        // contents whether or not the instance is named.
        let class = interp
            .classes()
            .lookup("Object")
            .expect("the Object class is registered");
        let behaviour = interp.classes().instance_behaviour_handle(class);
        for name in [None, Some(b"123".to_vec().into_boxed_slice())] {
            for native in [
                None,
                Some(Box::new(rexx_core::NativeState::Buffer(buffer_state(
                    b"held for long enough to need a slot",
                )))),
            ] {
                let instance = interp.alloc_with(
                    BehaviourId::OBJECT,
                    Body::Instance {
                        class,
                        behaviour,
                        name: name.clone(),
                        pools: rexx_core::ScopePools::new(),
                        own: None,
                        native,
                    },
                );
                values.push(instance);
            }
        }

        // **`text_len` is asked first, before anything renders**, so that a
        // `Body::Num` reaches its arm with an empty `text` cache at least
        // once. Asking `to_text` first would fill that cache, after which
        // both functions read the same stored bytes and the rendering half of
        // the arm is never exercised -- measured, a mutation replacing its
        // `created_digits` with a constant left this test green when the two
        // calls were the other way round. The second pass is the cached
        // reading, deliberately, and it is the *second*.
        for pass in 0..2 {
            for value in &values {
                let got = interp.text_len(*value);
                let expected = interp.to_text(*value).len();
                assert_eq!(got, expected, "pass {pass}, {value:?}");
            }
        }
        // Floors, because every assertion above is inside a loop.
        assert!(
            values.len() > 100,
            "only {} values were built",
            values.len()
        );
        assert!(
            numbers > 50,
            "only {numbers} of the grid became a `Body::Num`"
        );
    }

    /// The tag decision is the rendering's, at the tag's own boundary.
    #[test]
    fn the_tag_test_agrees_with_parsing_the_same_bytes() {
        fn reference(bytes: &[u8]) -> Option<i64> {
            let (negative, digits) = match bytes.split_first() {
                Some((b'-', rest)) => (true, rest),
                _ => (false, bytes),
            };
            if digits.is_empty() || !digits.iter().all(u8::is_ascii_digit) {
                return None;
            }
            if digits[0] == b'0' && (digits.len() > 1 || negative) {
                return None;
            }
            let magnitude: i64 = std::str::from_utf8(digits).ok()?.parse().ok()?;
            let value = if negative { -magnitude } else { magnitude };
            (SMALL_INT_MIN..=SMALL_INT_MAX)
                .contains(&value)
                .then_some(value)
        }

        let mut cases: Vec<Vec<u8>> = vec![
            b"".to_vec(),
            b"-".to_vec(),
            b"0".to_vec(),
            b"-0".to_vec(),
            b"00".to_vec(),
            b"05".to_vec(),
            b"-05".to_vec(),
            b"1".to_vec(),
            b"-1".to_vec(),
            b"9".to_vec(),
            b"10".to_vec(),
            b"123456789".to_vec(),
            b"-123456789".to_vec(),
            b"+1".to_vec(),
            b"1.0".to_vec(),
            b"1e5".to_vec(),
            b"1a".to_vec(),
            b"a1".to_vec(),
            b" 1".to_vec(),
            b"1 ".to_vec(),
            b"1\xff".to_vec(),
            b"\xff1".to_vec(),
            b"99999999999999999999999999".to_vec(),
            b"-99999999999999999999999999".to_vec(),
        ];
        for edge in [
            i64::MAX,
            i64::MIN,
            SMALL_INT_MAX,
            SMALL_INT_MIN,
            SMALL_INT_MAX - 1,
            SMALL_INT_MIN + 1,
        ] {
            cases.push(edge.to_string().into_bytes());
        }
        // One past each tag limit, and one past `i64::MAX`, spelled out
        // because they are not themselves representable in the type.
        for spelled in [
            "2305843009213693952",
            "-2305843009213693953",
            "9223372036854775808",
            "-9223372036854775809",
        ] {
            cases.push(spelled.as_bytes().to_vec());
        }

        let mut answered = 0usize;
        for case in &cases {
            let got = canonical_small_int(case);
            assert_eq!(got, reference(case), "{:?}", String::from_utf8_lossy(case));
            if got.is_some() {
                answered += 1;
            }
        }
        assert!(
            answered > 0,
            "every case was refused, so the agreement above is vacuous"
        );
    }

    #[test]
    fn the_tag_decision_is_the_rendering_read_back() {
        fn by_rendering(value: &Number, created_digits: u32) -> Option<i64> {
            let rendered = value.format_form(u64::from(created_digits), Form::Scientific);
            if rendered.contains('.') || rendered.contains('E') {
                return None;
            }
            let whole: i64 = rendered.parse().ok()?;
            (SMALL_INT_MIN..=SMALL_INT_MAX)
                .contains(&whole)
                .then_some(whole)
        }

        let spellings = [
            "0",
            "0.00",
            "-0",
            "1",
            "-1",
            "5",
            "1.50",
            "150",
            "1.50E+2",
            "12.0",
            // Values whose *rounding* is the integer, which is the half of
            // the set `plain_integer` cannot see: dropped digit below and
            // above five, a carry, and both signs.
            "1.4",
            "2.5",
            "12.4",
            "12.5",
            "-12.4",
            "-12.5",
            "19.6",
            "99.4",
            "99.6",
            "9.996",
            "123.456",
            "1234.5678",
            "2305843009213693951.4",
            "2305843009213693952.4",
            "1.2E+3",
            "999",
            "999.5",
            "1000",
            "1E+9",
            "1E+18",
            "1E+19",
            "0.001",
            "1E-5",
            "-12345",
            "123456789012345678",
            // The tag's limits, either side, and one past every `i64`.
            "2305843009213693950",
            "2305843009213693951",
            "2305843009213693952",
            "-2305843009213693952",
            "-2305843009213693953",
            "9223372036854775807",
            "9999999999999999999999",
        ];
        let precisions: [u32; 12] = [1, 2, 3, 4, 5, 9, 18, 19, 20, 21, 22, 25];

        let mut tagged = 0usize;
        let mut tagged_beyond_plain_integer = 0usize;
        for spelling in spellings {
            for digits in precisions {
                let value = n(spelling);
                let decided = small_int_for(&value, digits);
                assert_eq!(
                    decided,
                    by_rendering(&value, digits),
                    "{spelling} at DIGITS {digits}"
                );
                if decided.is_some() {
                    tagged += 1;
                    if value.plain_integer(u64::from(digits)).is_none() {
                        tagged_beyond_plain_integer += 1;
                    }
                }
            }
        }
        assert!(tagged > 100, "only {tagged} of the grid was tagged at all");
        assert!(
            tagged_beyond_plain_integer > 10,
            "only {tagged_beyond_plain_integer} tagged cases are outside `plain_integer`, \
             so this asserts almost nothing about the values a narrower rule would drop"
        );
    }

    /// `text_owned` keeps the caller's buffer instead of copying it.
    #[test]
    fn text_owned_takes_the_buffer_rather_than_copying_it() {
        let long = vec![b'q'; INLINE_BYTES + 1];
        let mut interp = Interp::new();
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(long.len())
            .expect("the buffer is available");
        bytes.extend_from_slice(&long);
        let address = bytes.as_ptr();

        let value = interp.text_owned(bytes);
        assert_eq!(interp.to_text(value).as_ptr(), address);
        assert_eq!(&*interp.to_text(value), &long[..]);
    }

    /// The two construction entry points and the boundary between the arms,
    /// held at the interpreter's own surface rather than only inside `Bytes`.
    #[test]
    fn a_short_value_holds_its_bytes_in_the_slot_and_a_long_one_does_not() {
        for len in [
            INLINE_TEXT + 1,
            INLINE_BYTES - 1,
            INLINE_BYTES,
            INLINE_BYTES + 1,
            400,
        ] {
            let source: Vec<u8> = (0..len).map(|i| b'a' + (i % 26) as u8).collect();
            let mut interp = Interp::new();

            for value in [interp.text(&source), interp.text_owned(source.clone())] {
                assert_eq!(&*interp.to_text(value), &source[..], "bytes at {len}");
                let Decoded::Heap { slot, .. } = value.decode() else {
                    panic!("a heap string at {len}")
                };
                let inline = interp
                    .heap
                    .get(value)
                    .is_some_and(|object| match &object.body {
                        Body::Text { bytes, .. } => bytes.is_inline(),
                        other => panic!("a Body::Text at {len}, got {other:?}"),
                    });
                assert_eq!(inline, len <= INLINE_BYTES, "arm at {len}, slot {slot}");
            }
        }
    }

    // The plan's own draft sketched these five tests as calls to
    // `interp.eval_str(...)`/`interp.eval_with(...)`, which do not exist and
    // cannot without pulling Task 6's `Activation`/`Settings` and Task 7's
    // arithmetic forward into this task -- scaffolding Task 7 would then have
    // to unwind. So every `Number` below is built directly through
    // `rexx-num`, which Phase 2 already tested; what these tests check is
    // only what `text`/`number`/`to_text`/`to_number` do with it afterwards.
    // The oracle transcripts remain the source of truth and are re-run
    // against `build/bin/rexx` in the task report.

    #[test]
    fn a_numbers_rendering_is_fixed_when_it_is_created() {
        // numeric digits 9 ; y = 1 / 3 ; numeric digits 3 ; say y
        //   -> 0.333333333 (not 0.333)
        //                   z = 1 / 3 ; say z -> 0.333
        let mut interp = Interp::new();
        let y = interp.number(
            n("1").div(&n("3"), 9, DivOp::Divide).unwrap(),
            9,
            Form::Scientific,
        );
        let z = interp.number(
            n("1").div(&n("3"), 3, DivOp::Divide).unwrap(),
            3,
            Form::Scientific,
        );
        assert_eq!(&*interp.to_text(y), b"0.333333333");
        assert_eq!(&*interp.to_text(z), b"0.333");
        // Reading `y` again proves the cache is genuinely reused, not just
        // right the first time: nothing about creating or reading `z`
        // afterwards moved it.
        assert_eq!(&*interp.to_text(y), b"0.333333333");
    }

    #[test]
    fn numeric_form_is_captured_at_creation_too() {
        // numeric form engineering ; x = 1e10 + 0 ; say x -> 10E+9
        // numeric form scientific  ;               say x -> 10E+9 (unchanged)
        //                            y = 1e10 + 0 ; say y -> 1E+10
        let mut interp = Interp::new();
        let sum = n("1e10").add(&n("0"), 9).unwrap();
        let x = interp.number(sum.clone(), 9, Form::Engineering);
        assert_eq!(&*interp.to_text(x), b"10E+9");
        let y = interp.number(sum, 9, Form::Scientific);
        assert_eq!(&*interp.to_text(x), b"10E+9", "x's own form must not move");
        assert_eq!(&*interp.to_text(y), b"1E+10");
    }

    #[test]
    fn a_small_int_is_only_admissible_within_the_digits_of_its_own_operation() {
        // numeric digits 1 ; x = 15 + 0 ; x is 20, so x + 6 is 3E+1 while
        // 15 + 6 is 2E+1.
        let mut interp = Interp::new();
        let x_number = n("15").add(&n("0"), 1).unwrap();
        let x = interp.number(x_number.clone(), 1, Form::Scientific);
        assert_eq!(&*interp.to_text(x), b"2E+1");

        let sum = interp.number(x_number.add(&n("6"), 1).unwrap(), 1, Form::Scientific);
        assert_eq!(&*interp.to_text(sum), b"3E+1");

        let direct = interp.number(n("15").add(&n("6"), 1).unwrap(), 1, Form::Scientific);
        assert_eq!(&*interp.to_text(direct), b"2E+1");
    }

    #[test]
    fn small_int_admissibility_is_checked_once_against_the_producing_operations_digits() {
        // D15's own discriminating pair: two SmallInt-shaped results that
        // differ observably. Checking the actual `ObjRef` shape, not only the
        // rendered bytes, is the point -- a wrongly-admitted `SmallInt(20)`
        // and a correctly-refused one can render identically to `to_text` if
        // `to_text`'s two branches happen to agree, and only the tag itself
        // tells them apart.
        let mut interp = Interp::new();
        let a = interp.number(n("20"), 9, Form::Scientific);
        assert!(matches!(a.decode(), Decoded::SmallInt(20)));
        assert_eq!(&*interp.to_text(a), b"20");

        let b = interp.number(n("15").add(&n("0"), 1).unwrap(), 1, Form::Scientific);
        assert!(matches!(b.decode(), Decoded::Heap { .. }));
        assert_eq!(&*interp.to_text(b), b"2E+1");
    }

    #[test]
    fn text_keeps_its_own_spelling_and_caches_an_exact_parse() {
        // x = '007' ; say x -> 007 ; say x + 0 -> 7
        let mut interp = Interp::new();
        let x = interp.text(b"007");
        assert_eq!(&*interp.to_text(x), b"007");

        let parsed = interp.to_number(x).unwrap();
        let converted = interp.number(parsed.add(&n("0"), 9).unwrap(), 9, Form::Scientific);
        assert_eq!(&*interp.to_text(converted), b"7");
    }

    #[test]
    fn the_text_cache_holds_the_exact_parse_not_a_rounded_one() {
        // x = '1.234567890123456789'. The SAME stored parse renders 1.2346
        // at DIGITS 5 and the full nineteen-digit value at DIGITS 20.
        // Measured against the oracle: this is `x + 0` at each DIGITS, not
        // `say x` alone -- a `Body::Text`'s identity is its own bytes, so
        // `say x` never converts it at all and prints the literal unchanged
        // at every DIGITS (`t4b.rex`/`t4c.rex` in the task report).
        let mut interp = Interp::new();
        let x = interp.text(b"1.234567890123456789");

        let rounded_5 = interp.to_number(x).unwrap().add(&n("0"), 5).unwrap();
        let at_5 = interp.number(rounded_5, 5, Form::Scientific);
        assert_eq!(&*interp.to_text(at_5), b"1.2346");

        // Reads the cache a second time, under a completely different
        // DIGITS, to show it is still the exact original parse and not
        // whatever the first read happened to round it to.
        let rounded_20 = interp.to_number(x).unwrap().add(&n("0"), 20).unwrap();
        let at_20 = interp.number(rounded_20, 20, Form::Scientific);
        assert_eq!(&*interp.to_text(at_20), b"1.234567890123456789");
    }

    /// Every shape a value can take, read through the borrowing pair, gives
    /// the bytes `to_text` gives.
    #[test]
    fn the_borrowing_accessor_answers_what_to_text_answers() {
        let mut interp = Interp::new();

        let short = interp.text(b"abc");
        let long = interp.text(&[b'z'; INLINE_BYTES + 40]);
        let small = interp.number(n("7"), 9, Form::Scientific);
        let heap_number = interp.number(n("1.5"), 9, Form::Scientific);
        let rendered_number = interp.number(n("2.5"), 9, Form::Scientific);
        let _ = interp.to_text(rendered_number);

        let five = interp.number(n("5"), 9, Form::Scientific);
        let with_default = interp.alloc_with(
            BehaviourId::STEM,
            Body::Stem {
                name: b"A.".to_vec().into(),
                default: Some(five),
                tails: rexx_core::NameMap::default(),
            },
        );
        let bare = interp.alloc_with(
            BehaviourId::STEM,
            Body::Stem {
                name: b"Q.".to_vec().into(),
                default: None,
                tails: rexx_core::NameMap::default(),
            },
        );
        let aliasing = interp.alloc_with(
            BehaviourId::STEM,
            Body::Stem {
                name: b"B.".to_vec().into(),
                default: Some(with_default),
                tails: rexx_core::NameMap::default(),
            },
        );

        for value in [
            ObjRef::NIL,
            short,
            long,
            small,
            heap_number,
            rendered_number,
            with_default,
            bare,
            aliasing,
        ] {
            let rendered = interp.render(value);
            let borrowed = rendered.text(&interp).to_vec();
            let expected = interp.to_text(value).into_owned();
            assert_eq!(
                String::from_utf8_lossy(&borrowed),
                String::from_utf8_lossy(&expected),
                "{value:?}"
            );
        }
    }

    /// `try_text` answers `None` for exactly two causes, and both are "there
    /// is nothing to borrow" rather than "there is no text".
    #[test]
    fn try_text_answers_only_where_the_bytes_already_exist() {
        let mut interp = Interp::new();

        // Cause one: the digits of a tagged small integer are stored nowhere
        // at all, so nothing renders it into existence and `render` has to
        // carry them.
        let small = interp.number(n("7"), 9, Form::Scientific);
        assert!(matches!(small.decode(), Decoded::SmallInt(7)));
        assert_eq!(interp.try_text(small), None);
        let _ = interp.to_text(small);
        assert_eq!(
            interp.try_text(small),
            None,
            "rendering a SmallInt stores nothing"
        );

        // Cause two: a heap number's `text` cache is empty until something
        // fills it, and filling it is the `&mut` this pair exists to move.
        let number = interp.number(n("1.5"), 9, Form::Scientific);
        assert_eq!(interp.try_text(number), None);
        let _ = interp.to_text(number);
        assert_eq!(interp.try_text(number), Some(&b"1.5"[..]));

        // And a stem chasing a default is answered by the *default's* bytes,
        // which is what `to_text` has to copy and this does not -- so it
        // inherits the default's own answer, `None` included.
        let stem = interp.alloc_with(
            BehaviourId::STEM,
            Body::Stem {
                name: b"A.".to_vec().into(),
                default: Some(small),
                tails: rexx_core::NameMap::default(),
            },
        );
        assert_eq!(interp.try_text(stem), None, "the default is a SmallInt");
        // Long enough to have a slot: a shorter one lives in the handle and
        // so has nothing to borrow, which is the case asserted below.
        let text = interp.text(b"held for long enough to need a slot");
        let stem_of_text = interp.alloc_with(
            BehaviourId::STEM,
            Body::Stem {
                name: b"B.".to_vec().into(),
                default: Some(text),
                tails: rexx_core::NameMap::default(),
            },
        );
        assert_eq!(
            interp.try_text(stem_of_text),
            Some(&b"held for long enough to need a slot"[..])
        );

        // The third cause of `None`, and the one this encoding adds: the
        // bytes are in the handle, which is a `Copy` local, so a shared
        // borrow has nothing to point at. `to_text` still answers.
        let inline = interp.text(b"held");
        assert!(matches!(inline.decode(), Decoded::Text(_)));
        assert_eq!(interp.try_text(inline), None, "the bytes are the handle");
        assert_eq!(&*interp.to_text(inline), b"held");

        // A buffer's bytes are the object's own storage, borrowable whether
        // or not the instance is named -- the name never wins over them.
        let class = interp
            .classes()
            .lookup("Object")
            .expect("the Object class is registered");
        let behaviour = interp.classes().instance_behaviour_handle(class);
        for name in [None, Some(b"named".to_vec().into_boxed_slice())] {
            let buffer = interp.alloc_with(
                BehaviourId::OBJECT,
                Body::Instance {
                    class,
                    behaviour,
                    name,
                    pools: rexx_core::ScopePools::new(),
                    own: None,
                    native: Some(Box::new(rexx_core::NativeState::Buffer(buffer_state(
                        b"abcdef",
                    )))),
                },
            );
            assert_eq!(interp.try_text(buffer), Some(&b"abcdef"[..]));
            assert_eq!(&*interp.to_text(buffer), b"abcdef");
            assert_eq!(interp.text_len(buffer), 6);
        }
    }

    /// The other side of that boundary: a value short enough occupies no
    /// slot at all.
    #[test]
    fn a_value_short_enough_is_the_handle_and_has_no_slot() {
        for len in 0..=INLINE_TEXT {
            let source: Vec<u8> = (0..len).map(|i| b'a' + (i % 26) as u8).collect();
            let mut interp = Interp::new();
            let before = interp.heap.live_count();

            for value in [interp.text(&source), interp.text_owned(source.clone())] {
                assert!(
                    matches!(value.decode(), Decoded::Text(_)),
                    "carried in the handle at {len}, got {:?}",
                    value.decode()
                );
                assert_eq!(&*interp.to_text(value), &source[..], "bytes at {len}");
            }
            assert_eq!(
                interp.heap.live_count(),
                before,
                "no slot taken at {len}, on either entry point"
            );
        }

        // One byte more is a heap object again, which is what says the test
        // above is reading a boundary rather than a constant.
        let over: Vec<u8> = vec![b'z'; INLINE_TEXT + 1];
        let mut interp = Interp::new();
        let value = interp.text(&over);
        assert!(matches!(value.decode(), Decoded::Heap { .. }));
    }

    #[test]
    fn nil_has_a_string_value_and_the_booleans_are_plain_strings() {
        // say .nil -> The NIL object ; .true is "1" ; .false is "0"
        let mut interp = Interp::new();
        assert_eq!(&*interp.to_text(ObjRef::NIL), b"The NIL object");

        // `.true`/`.false` need no representation of their own (D15): they
        // are the one-byte strings "1" and "0", built the same way any other
        // text value is.
        let true_value = interp.text(b"1");
        let false_value = interp.text(b"0");
        assert_eq!(&*interp.to_text(true_value), b"1");
        assert_eq!(&*interp.to_text(false_value), b"0");
    }

    #[test]
    fn nonnumeric_text_and_nil_both_collapse_to_not_numeric() {
        let mut interp = Interp::new();
        let words = interp.text(b"not a number");
        assert_eq!(interp.to_number(words), Err(NotNumeric));
        assert_eq!(interp.to_number(ObjRef::NIL), Err(NotNumeric));
    }

    // Branch review F5 (Critical): `to_number` on a `Body::Stem` used to hit
    // the `unreachable!` fallback and abort the process (rc 101) --
    // `to_text` already redirected through a stem's default/name, `to_number`
    // never learned to. These three tests build a `Body::Stem` directly
    // (the same shape `stem.rs`'s own allocations use) and drive it straight
    // through `to_number`, with no live activation needed for that alone.

    #[test]
    fn to_number_on_a_stem_redirects_through_its_numeric_default() {
        // a. = 5 ; say a. + 1 -> 6 (measured against the oracle). Kills the
        // mutation that reverts this arm to `unreachable!`: that mutant
        // panics this test instead of returning `Ok(5)`.
        let mut interp = Interp::new();
        let five = interp.number(n("5"), 9, Form::Scientific);
        let stem = interp.alloc_with(
            BehaviourId::STEM,
            Body::Stem {
                name: b"A.".to_vec().into(),
                default: Some(five),
                tails: rexx_core::NameMap::default(),
            },
        );
        let value = interp.to_number(stem).unwrap();
        let sum = interp.number(value.add(&n("1"), 9).unwrap(), 9, Form::Scientific);
        assert_eq!(&*interp.to_text(sum), b"6");
    }

    #[test]
    fn to_number_on_a_defaultless_stem_parses_its_own_name_and_fails() {
        // say q. + 1 on a stem nobody ever assigned a default to: the
        // oracle raises 41.1, nonnumeric value "Q." (measured). Kills a
        // mutation that makes the `default: None` arm return some fixed
        // `Ok` (e.g. `Ok(0)`) instead of parsing the object's own name and
        // reporting `NotNumeric` when, as here, that name is not one.
        let mut interp = Interp::new();
        let stem = interp.alloc_with(
            BehaviourId::STEM,
            Body::Stem {
                name: b"Q.".to_vec().into(),
                default: None,
                tails: rexx_core::NameMap::default(),
            },
        );
        assert_eq!(interp.to_number(stem), Err(NotNumeric));
        // Rendering is untouched by this fix -- still the derived name.
        assert_eq!(&*interp.to_text(stem), b"Q.");
    }

    #[test]
    fn to_number_on_a_stem_aliasing_another_stem_chases_through_both() {
        // a. = 5 ; b. = a. ; say b. + 1 -> 6, the aliasing shape
        // `stem_assign` actually produces (`stem.rs`): `b.`'s default is
        // itself a `Body::Stem`, never a copy of `a.`'s value. Kills a
        // mutation that only redirects one level deep (e.g. matching
        // `Body::Num`/`Body::Text` inline instead of recursing through
        // `to_number` again), which would panic on the second hop.
        let mut interp = Interp::new();
        let five = interp.number(n("5"), 9, Form::Scientific);
        let a = interp.alloc_with(
            BehaviourId::STEM,
            Body::Stem {
                name: b"A.".to_vec().into(),
                default: Some(five),
                tails: rexx_core::NameMap::default(),
            },
        );
        let b = interp.alloc_with(
            BehaviourId::STEM,
            Body::Stem {
                name: b"B.".to_vec().into(),
                default: Some(a),
                tails: rexx_core::NameMap::default(),
            },
        );
        let value = interp.to_number(b).unwrap();
        let sum = interp.number(value.add(&n("1"), 9).unwrap(), 9, Form::Scientific);
        assert_eq!(&*interp.to_text(sum), b"6");
    }
}
