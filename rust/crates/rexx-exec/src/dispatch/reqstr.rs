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

//! The required-string protocol: `request("STRING")`, `makeString`, and the
//! fallbacks behind them.

use super::{Body, Decoded, Failure, Interp, ObjRef, Primitive, Raised};

/// What the protocol's conversion limbs answered, before anything is built.
enum RequiredString {
    Object(ObjRef),
    Bytes(Vec<u8>),
}

/// The message `Object~request("STRING")` looks for, and the one limb of the
/// required-string protocol a program can write.
pub(crate) const MAKESTRING: &[u8] = b"MAKESTRING";

/// What `RexxObject::stringValue` sends (`classes/ObjectClass.cpp:1157`).
pub(crate) const OBJECTNAME: &[u8] = b"OBJECTNAME";

/// What `RexxObject::objectName` sends for an object nothing has named and
/// whose class is not a base class (`classes/ObjectClass.cpp:1712`).
pub(crate) const DEFAULTNAME: &[u8] = b"DEFAULTNAME";

/// The required-string protocol's last limb, past every conversion
/// (`RexxInternalObject::requiredString`, `classes/ObjectClass.cpp:1353`).
pub(crate) const STRING: &[u8] = b"STRING";

/// What a value's own string value is, before the protocol's fallbacks: the
/// value itself, the object a `makeString` answered, bytes no object holds, or
/// nothing.
enum StringConversion {
    /// The object that *is* the string value: the value itself, or a stem's
    /// own default value, or what a `makeString` answered.
    Object(ObjRef),
    /// The string value as bytes no object holds -- an array's items joined.
    Bytes(Vec<u8>),
    /// The receiver's behaviour has a `makeString` to send.
    MakeString,
    /// None of those, which is `.nil` from `requestString`'s point of view.
    None,
}

impl Interp {
    /// The required-string protocol, D52: `request("STRING")`, then the
    /// receiver's `makeString`, then the NOSTRING condition if trapped, else
    /// `defaultName`. **This is the value every context `provide.xml`
    /// `reqstr` lists renders, and rendering it without coming through here
    /// is the silent wrong answer the protocol exists to prevent.**
    #[inline]
    pub(crate) fn required_string_value(&mut self, value: ObjRef) -> Result<ObjRef, Failure> {
        // **A string and a small integer are their own string value**, and
        // that is the answer whether the latch is armed or clear:
        // `classify_string_conversion`'s first arm returns
        // `StringConversion::Object(value)` for both, before it touches
        // `self`, allocates, or looks at the heap. Answering here rather than
        // three calls deeper is what keeps an armed interpreter from paying
        // `required_string_dispatch` + `required_string_answer` +
        // `classify_string_conversion` for every operand of every comparison.
        if matches!(value.decode(), Decoded::SmallInt(_) | Decoded::Text(_)) {
            return Ok(value);
        }
        if self.reqstr_armed {
            // **A heap string or number is its own string value as much as an
            // inline one is** -- the same `classify_string_conversion` arm
            // answers `Object(value)` for `Body::Text` and `Body::Num`. It
            // costs the heap lookup that arm makes anyway, and it keeps the
            // `push_temp`, because unlike the two decodings above this value
            // does live in the arena and the caller's rooting of it is not
            // this function's to assume.
            if matches!(
                self.heap.get(value).map(|object| &object.body),
                Some(Body::Text { .. } | Body::Num { .. })
            ) {
                self.roots.push_temp(value);
                return Ok(value);
            }
            return self.required_string_dispatch(value);
        }
        debug_assert!(
            self.required_string_latch_holds(value),
            "the required-string latch is off where the protocol would answer differently \
             or raise"
        );
        Ok(value)
    }

    /// Whether the fast path is the right answer for `value` with the latch
    /// clear -- one test per limb the latch claims cannot fire.
    fn required_string_latch_holds(&mut self, value: ObjRef) -> bool {
        // Limb 3's route, and it is not about `value` at all: a trap that
        // could take NOSTRING while the latch is clear is wrong for every
        // rendered object, because `Interp::exec_condition_trap` is what arms
        // both and a clear latch here means it did not. The gate is the same
        // one `required_string_dispatch` applies, so this fires exactly where
        // that would have raised.
        if self.trap_for(b"NOSTRING").is_some_and(|trap| !trap.call)
            || self.condition_raises_syntax(b"NOSTRING")
        {
            return false;
        }
        // Limb 1's route: the conversion limbs, run in full, against the
        // rendering the fast path hands back instead.
        let expected = self.to_text(value).into_owned();
        let answered = match self.required_string_answer(value) {
            Ok(answered) => answered,
            Err(_) => return false,
        };
        let bytes = match answered {
            Some(RequiredString::Object(text)) => self.to_text(text).into_owned(),
            Some(RequiredString::Bytes(bytes)) => bytes,
            // `stringValue()`, which is what the fast path's own rendering of
            // an object with no string value comes to.
            None => self.string_value_text(value),
        };
        bytes == expected
    }

    /// **The answer is rooted here and not at the call sites.** A `makeString`
    /// can allocate and can collect, and both fallbacks below build a string
    /// of their own, so every value this returns is one the caller did not
    /// have a root for. Rooting once, on the arm that can produce a fresh
    /// object, is what keeps the gate above free of a temp push per rendered
    /// value.
    #[cold]
    #[inline(never)]
    fn required_string_dispatch(&mut self, value: ObjRef) -> Result<ObjRef, Failure> {
        match self.required_string_answer(value) {
            Ok(Some(RequiredString::Object(text))) => {
                self.roots.push_temp(text);
                return Ok(text);
            }
            Ok(Some(RequiredString::Bytes(bytes))) => {
                let text = self.text_built(bytes);
                self.roots.push_temp(text);
                return Ok(text);
            }
            Ok(None) => {}
            Err(failure) => {
                self.blame_request();
                return Err(failure);
            }
        }
        // `sendMessage(STRING)`. An instance's `STRING` can be a Rexx
        // method, so it is sent; the shortcut below is what the send answers
        // wherever `STRING` resolves to `native_string`. Measured, oracle
        // rc 0: with `::METHOD string` returning `from-string`, `say o`,
        // `'x' o` and `length(o)` all follow it.
        let readable = if matches!(self.receiver_kind(value), Ok(Primitive::Instance { .. })) {
            let caller = self.caller();
            match self.send_message(value, STRING, None, &[], caller)? {
                Some(answered) => self.string_value_text(answered),
                None => self.string_value_text(value),
            }
        } else {
            self.string_value_text(value)
        };
        // Gated on the trap rather than raised unconditionally, the shape
        // `Interp::novalue_raised` describes: an untrapped NOSTRING resumes
        // with the readable rendering, so a raise nothing can take would
        // build a condition per rendered object and then throw it away. A
        // `CALL ON` trap is excluded because `CALL ON NOSTRING` is a parse
        // error and `CALL ON ANY` is measured not to catch a condition with
        // no resumption point.
        if self.trap_for(b"NOSTRING").is_some_and(|trap| !trap.call) {
            return Err(Raised::nostring(&readable).into());
        }
        // `::OPTIONS NOSTRING SYNTAX`, and only where nothing trapped: the
        // trap wins, measured -- `signal on nostring` over `say .array` in
        // such a file runs the handler, and `signal on syntax` takes 98.973.
        if self.condition_raises_syntax(b"NOSTRING") {
            return Err(Raised::nostring_syntax(&readable).into());
        }
        let readable = self.text_built(readable);
        self.roots.push_temp(readable);
        Ok(readable)
    }

    /// Every supplied argument of one builtin call **except the positions
    /// `crate::builtin::raw_argument_positions` exempts for `name`**, in
    /// position order, through the required-string protocol -- `None` when the
    /// protocol cannot change any of them, so the caller passes its own list
    /// on.
    pub(crate) fn required_string_arguments(
        &mut self,
        name: &'static [u8],
        args: &[Option<ObjRef>],
    ) -> Result<Option<Vec<Option<ObjRef>>>, Failure> {
        if !self.reqstr_armed {
            return Ok(None);
        }
        // Looked up behind the latch, not in front of it: an ordinary call
        // pays one bool test here and nothing else, where a lookup at the call
        // site would scan the exemption table on every builtin call in every
        // program.
        let raw = crate::builtin::raw_argument_positions(name);
        // **Scanned before anything is built.** `None` means "the arguments
        // stand as they are", so a call whose every argument is already its
        // own string value can answer that instead of allocating a vector to
        // hold copies of what the caller already has. An armed interpreter
        // otherwise allocated one `Vec` per builtin call for the whole run,
        // and being armed is not rare: any instance of a Rexx class arms it.
        if !args.iter().enumerate().any(|(index, argument)| {
            argument.is_some_and(|value| {
                !raw.contains(&(index + 1))
                    && !matches!(value.decode(), Decoded::SmallInt(_) | Decoded::Text(_))
            })
        }) {
            return Ok(None);
        }
        let mut converted = Vec::with_capacity(args.len());
        for (index, argument) in args.iter().enumerate() {
            let position = index + 1;
            converted.push(match argument {
                None => None,
                Some(value) if raw.contains(&position) => Some(*value),
                Some(value) => Some(self.required_string_value(*value)?),
            });
        }
        Ok(Some(converted))
    }

    /// The protocol's conversion limbs, answered without building anything --
    /// [`Interp::string_conversion`]'s own body with the materialisation left
    /// to the caller.
    fn required_string_answer(&mut self, value: ObjRef) -> Result<Option<RequiredString>, Failure> {
        Ok(match self.classify_string_conversion(value) {
            StringConversion::Object(text) => Some(RequiredString::Object(text)),
            StringConversion::Bytes(bytes) => Some(RequiredString::Bytes(bytes)),
            StringConversion::None => None,
            StringConversion::MakeString => {
                // `string_value = string_value->primitiveMakeString()`
                // (`:1257`, `:1353`): what `makeString` answered has to be a
                // real string, and an object that is not one converts here or
                // counts as no answer at all. Measured, oracle rc 0 on
                // `say .K` with a class-side `makeString`: `return 5` prints
                // `5`, `return .Object~superClasses` prints the empty line
                // an empty array joins to, and `return .array` prints
                // `The K class` -- the third fell back.
                match self.send_make_string(value)? {
                    Some(answered) => match self.classify_string_conversion(answered) {
                        StringConversion::Object(text) => Some(RequiredString::Object(text)),
                        StringConversion::Bytes(bytes) => Some(RequiredString::Bytes(bytes)),
                        StringConversion::MakeString | StringConversion::None => None,
                    },
                    None => None,
                }
            }
        })
    }

    /// `RexxInternalObject::requiredString()` (`classes/ObjectClass.cpp:1341`):
    /// the protocol's conversion limbs alone, with **no** `~string` fallback
    /// and **no** NOSTRING condition. `None` is the oracle's `.nil`.
    pub(crate) fn string_conversion(&mut self, value: ObjRef) -> Result<Option<ObjRef>, Failure> {
        Ok(match self.required_string_answer(value)? {
            Some(RequiredString::Object(text)) => Some(text),
            Some(RequiredString::Bytes(bytes)) => Some(self.text_built(bytes)),
            None => None,
        })
    }

    /// [`Interp::string_conversion`] with the `REQUEST` traceback line on it,
    /// for a caller inside a native method's own activation -- see
    /// [`Interp::blame_request`] for why the two frames stack there.
    pub(super) fn blamed_string_conversion(
        &mut self,
        value: ObjRef,
    ) -> Result<Option<ObjRef>, Failure> {
        let converted = self.string_conversion(value);
        if converted.is_err() {
            self.blame_request();
        }
        converted
    }

    /// Which limb of the protocol answers for `value`.
    fn classify_string_conversion(&mut self, value: ObjRef) -> StringConversion {
        let redirect = match value.decode() {
            Decoded::Nil => return self.make_string_or_none(value),
            // `RexxString::primitiveMakeString` and
            // `RexxInteger::primitiveMakeString`: a string and a number are
            // their own string value.
            Decoded::SmallInt(_) | Decoded::Text(_) => {
                return StringConversion::Object(value);
            }
            Decoded::Heap { .. } if self.heap.is_class(value) => {
                return self.make_string_or_none(value);
            }
            Decoded::Heap { .. } => match self.heap.get(value) {
                // A handle whose slot is gone, which `Interp::to_text` turns
                // into its own tripwire. Answering the value keeps that the
                // one report rather than adding a second.
                None => return StringConversion::Object(value),
                Some(object) => match &object.body {
                    Body::Text { .. } | Body::Num { .. } => {
                        return StringConversion::Object(value);
                    }
                    // `StemClass::makeString` forwards to the default value,
                    // and a stem holding none is its own derived name, which
                    // is what `to_text` renders for it.
                    Body::Stem { default: None, .. } => {
                        return StringConversion::Object(value);
                    }
                    Body::Stem {
                        default: Some(default),
                        ..
                    } => Some(*default),
                    // `ArrayClass::makeString` (`classes/ArrayClass.cpp:1841`),
                    // the items joined by a newline -- **not**
                    // `stringValue()`, which is `an Array`.
                    Body::Array { .. } => None,
                    // **The referent's `stringValue()`, not its
                    // conversion.** `requestString` looks `MAKESTRING` up in
                    // the receiver's own behaviour rather than sending it,
                    // and a reference's behaviour holds no such name, so
                    // limb 4 answers `VariableReference::stringValue`
                    // (`classes/VariableReference.cpp:249`), which is the
                    // referent's. Measured, oracle rc 0: `say 'p' >zz` over a
                    // two-item array is `p an Array` where `say 'p' zz` joins
                    // the items. `~request('STRING')` is a different route
                    // and does forward -- `native_reference_request`.
                    Body::VarRef(_) => {
                        let text = crate::value::string_value_of_reference(self, value);
                        return StringConversion::Bytes(text);
                    }
                    _ => return self.make_string_or_none(value),
                },
            },
        };
        match redirect {
            Some(default) => self.classify_string_conversion(default),
            None => StringConversion::Bytes(self.string_conversion_array_text(value)),
        }
    }

    /// [`Interp::classify_string_conversion`]'s array arm, split out so the
    /// borrow of the heap object above ends before the join runs.
    fn string_conversion_array_text(&mut self, value: ObjRef) -> Vec<u8> {
        let items = self.array_slots_of(value).unwrap_or_default();
        self.array_string(&items, b"\n")
    }

    /// Whether the receiver's own behaviour answers `MAKESTRING`.
    fn make_string_or_none(&mut self, value: ObjRef) -> StringConversion {
        match self.lookup(value, MAKESTRING, None) {
            Some(_) => StringConversion::MakeString,
            None => StringConversion::None,
        }
    }

    /// Sends `makeString` on the receiver's behalf.
    fn send_make_string(&mut self, receiver: ObjRef) -> Result<Option<ObjRef>, Failure> {
        // The sending side is the frame the conversion happens in, not the
        // conversion itself: `checkPrivate` asks
        // `getTopStackFrame()->getReceiver()`, and `requestString` runs no
        // frame of its own that could answer that question differently.
        let caller = self.caller();
        self.send_message(receiver, MAKESTRING, None, &[], caller)
    }

    /// The traceback line the oracle's own `REQUEST` activation contributes
    /// when a `makeString` reached through the protocol fails.
    fn blame_request(&mut self) {
        self.blame_native_method(b"REQUEST", "Object");
    }
}

/// `RexxInternalObject::requiredString(position)`
/// (`classes/ObjectClass.cpp:1373`): a method argument the method needs as
/// text, converted through the required-string protocol, or 88.909 for a
/// value that has no string value at all.
/// ```text
/// 'abc'~hasMethod(5)                oracle `0` rc 0    a number has one
/// 'abc'~hasMethod(a.)               oracle `0` rc 0    an unset stem is its own name
/// 'abc'~hasMethod(.nil)             oracle 88.909 rc 168
/// 'abc'~hasMethod(.String)          oracle 88.909 rc 168
/// 'abc'~hasMethod(.environment)     oracle 88.909 rc 168
/// a. = .array; 'abc'~hasMethod(a.)  oracle 88.909 rc 168
/// a. = .nil;   'abc'~hasMethod(a.)  oracle 88.909 rc 168
/// ```
pub(super) fn required_string_argument(
    interp: &mut Interp,
    value: ObjRef,
    position: usize,
) -> Result<ObjRef, Failure> {
    match interp.blamed_string_conversion(value)? {
        Some(text) => Ok(text),
        None => Err(Raised::argument_needs_a_string_value(position).into()),
    }
}

/// [`required_string_argument`] for an argument the oracle's 88.909 names
/// rather than numbers.
pub(super) fn required_string_named_argument(
    interp: &mut Interp,
    value: ObjRef,
    argument: &'static str,
) -> Result<ObjRef, Failure> {
    match interp.blamed_string_conversion(value)? {
        Some(text) => Ok(text),
        None => Err(Raised::named_argument_needs_a_string_value(argument).into()),
    }
}
