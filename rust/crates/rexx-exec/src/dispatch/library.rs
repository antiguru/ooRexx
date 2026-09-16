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

//! Running a method whose body is a procedure of a loaded shared library, and
//! the [`Host`] the boundary reaches this interpreter through.
//!
//! `NativeActivation::run` (`interpreter/execution/NativeActivation.cpp:1264`)
//! is the reference for the call, and the `NativeActivation` that runs it is
//! also what serves the context, which is why one frame carries both the
//! receiver's pool and the local references.

use std::borrow::Cow;
use std::rc::Rc;

use rexx_api::ffi::Contexts;
use rexx_api::handles::Table;
use rexx_api::invoke;
use rexx_api::layout::POINTER;
use rexx_api::values::{
    Activation, CStringPool, Class, Constants, Conversion, Failure as Refused, Host, Numeric,
    Raised as Condition,
};
use rexx_core::{BehaviourId, Body, Decoded, ObjRef};
use rexx_num::{DIGITS64, Number};

use super::Resolution;
use crate::builtin::datatype::{SymbolKind, classify};
use crate::error::Raised;
use crate::{Failure, Interp, LibraryBinding, Loud, NativeFrame};

/// The bytes a small integer or an inline string renders as, for a reader
/// that cannot allocate into the interpreter.
fn rendered(value: ObjRef) -> Option<Vec<u8>> {
    match value.decode() {
        Decoded::SmallInt(number) => Some(number.to_string().into_bytes()),
        Decoded::Text(inline) => Some(inline.to_vec()),
        _ => None,
    }
}

/// The pool name an extension's spelling addresses, or `None` for one
/// `getVariableRetriever` (`execution/VariableDictionary.cpp:738`) answers
/// nothing for: the write then does nothing, which is all the API can see of
/// the refusal.
fn pool_variable_name(name: &[u8]) -> Option<Vec<u8>> {
    let first = *name.first()?;
    if first.is_ascii_digit() || name.contains(&b'.') {
        return None;
    }
    Some(name.to_ascii_uppercase())
}

impl Interp {
    /// Runs `binding`'s procedure against `args` with `receiver` as its self,
    /// and answers what the extension returned.
    ///
    /// # Errors
    /// The condition the extension raised, whatever converting an argument or
    /// the result refuses, and [`Loud`] for a conversion this phase has not
    /// written.
    pub(super) fn run_library_method(
        &mut self,
        binding: &LibraryBinding,
        resolution: Resolution,
        receiver: ObjRef,
        name: &[u8],
        args: &[Option<ObjRef>],
    ) -> Result<Option<ObjRef>, Failure> {
        let owner = self.pool_owner(receiver)?;
        // Cloned out of the binding before the interpreter is borrowed as the
        // host: the row is borrowed from the library, and the library has to
        // outlive that borrow without being reachable through `self`.
        let library = Rc::clone(&binding.library);
        let Some(entry) = library.method(&binding.procedure) else {
            return Err(Loud::library_procedure_gone().into());
        };
        self.push_native_frame(owner, resolution.scope, Some(receiver), name, args);
        let mut strings = CStringPool::new();
        let (answered, pending) = {
            let activation = Activation::new(Conversion {
                host: self,
                strings: &mut strings,
            });
            let mut contexts = Contexts::new(&activation);
            let answered = invoke::method(entry, &contexts.method(), &activation, args);
            (answered, activation.pending())
        };
        let (raised, method) = self.pop_native_frame();

        // The condition first, because the oracle raises it in the caller's
        // frame once the call has returned (`NativeActivation::checkConditions`,
        // `:1787`) and before it does anything with the value the extension
        // wrote. Measured, oracle: `.MyRe~new('[')` over `RegExp_Init` is
        // `Error 38 ... Invalid template or pattern.` at rc 218, and the
        // extension returned zero on that path.
        if let Some(number) = pending {
            return Err(condition_of(number));
        }
        let packaged = self.external_package_path(resolution.method).is_some();
        self.settle_native_call(answered, raised, method, packaged)
    }

    /// Runs the routine a [`Interp::package_routine`] slot names, as the call
    /// named `name` passed it.
    ///
    /// # Errors
    /// As [`Interp::run_library_routine`].
    pub(crate) fn run_package_routine(
        &mut self,
        slot: usize,
        name: &[u8],
        args: &[Option<ObjRef>],
    ) -> Result<Option<ObjRef>, Failure> {
        let code = self.package_routine_code(slot);
        self.run_library_routine(code, &name.to_ascii_uppercase(), args)
    }

    /// Runs the library routine [`Interp::library_codes`] row `code` is
    /// against `args`, and answers what the extension returned.
    ///
    /// `name` is the name the traceback's `Compiled routine` line gives.
    ///
    /// # Errors
    /// The condition the extension raised, whatever converting an argument or
    /// the result refuses, and [`Loud`] for a conversion or a routine style
    /// this crate does not implement.
    pub(crate) fn run_library_routine(
        &mut self,
        code: usize,
        name: &[u8],
        args: &[Option<ObjRef>],
    ) -> Result<Option<ObjRef>, Failure> {
        let key = self.library_code_key(code).clone();
        let Some(library) = self.libraries.get(&key.library).map(Rc::clone) else {
            return Err(Loud::library_procedure_gone().into());
        };
        let Some(entry) = library.routine(&key.procedure) else {
            return Err(Loud::library_procedure_gone().into());
        };
        self.push_native_frame(ObjRef::NIL, ObjRef::NIL, None, name, args);
        let mut strings = CStringPool::new();
        let (answered, pending) = {
            let activation = Activation::new(Conversion {
                host: self,
                strings: &mut strings,
            });
            let mut contexts = Contexts::new(&activation);
            let answered = invoke::routine(entry, &contexts.call(), &activation, args);
            (answered, activation.pending())
        };
        let (raised, method) = self.pop_native_frame();
        let package = self.library_code_package_path(code);
        let outcome = match pending {
            Some(number) => Err(condition_of(number)),
            None => self.settle_native_call(answered, raised, method, package.is_some()),
        };
        if outcome.is_err() {
            self.blame_native_routine(name, package);
        }
        outcome
    }

    /// What a native call answers once its frame is off the stack: the value,
    /// the condition its string conversion raised, or the boundary's own
    /// refusal. `packaged` is whether the code reports a package, which is
    /// what a refusal before the call is reported against with no line; one
    /// that reports none is reported against the caller's clause
    /// (`Activity::generateProgramInformation`, `interpreter/concurrency/Activity.cpp:1093-1113`).
    fn settle_native_call(
        &mut self,
        answered: Result<Option<ObjRef>, Refused>,
        raised: Option<Failure>,
        method: bool,
        packaged: bool,
    ) -> Result<Option<ObjRef>, Failure> {
        match answered {
            Err(Refused::Raised) => {
                Err(raised.expect("a host answering Raised holds the condition it raised"))
            }
            Ok(value) => Ok(value),
            Err(refused) => Err(self.refusal(refused, packaged, method)),
        }
    }

    /// Pushes the frame of a native call: a method's when `receiver` is
    /// given, a routine's otherwise.
    fn push_native_frame(
        &mut self,
        owner: ObjRef,
        scope: ObjRef,
        receiver: Option<ObjRef>,
        name: &[u8],
        args: &[Option<ObjRef>],
    ) {
        let mut frame = self.native_spares.pop().unwrap_or_else(|| NativeFrame {
            owner: ObjRef::NIL,
            scope: ObjRef::NIL,
            method: false,
            receiver: ObjRef::NIL,
            name: Vec::new(),
            arguments: Vec::new(),
            argument_list: None,
            locals: Table::new(),
            raised: None,
        });
        frame.owner = owner;
        frame.scope = scope;
        frame.method = receiver.is_some();
        frame.receiver = receiver.unwrap_or(ObjRef::NIL);
        frame.name.extend_from_slice(name);
        frame.arguments.extend_from_slice(args);
        self.native_handles.push(frame);
    }

    /// Pops the innermost native frame, answering the condition it holds and
    /// whether it was a method's, and keeps its cleared buffers for the next
    /// call to refill.
    fn pop_native_frame(&mut self) -> (Option<Failure>, bool) {
        let mut frame = self
            .native_handles
            .pop()
            .expect("the frame pushed for this call is still the innermost");
        let answer = (frame.raised.take(), frame.method);
        frame.name.clear();
        frame.arguments.clear();
        frame.argument_list = None;
        frame.locals.clear();
        self.native_spares.push(frame);
        answer
    }

    /// What the boundary's own refusal reports.
    fn refusal(&mut self, refused: Refused, packaged: bool, method: bool) -> Failure {
        let mut raised = match refused {
            Refused::MissingArgument { position } => Raised::missing_native_argument(position),
            Refused::NoStringValue { position } => {
                Raised::native_argument_needs_a_string_value(position)
            }
            // `found` is the argument's `stringValue()`, which is not its string
            // conversion: measured, oracle, an array is `found "an Array".` for
            // 88.921 and 88.905 where its string conversion joins its items.
            // `NotLogical` below is the exception: its `found` is the string
            // the conversion answered, `found "1\n2"` for that same array.
            Refused::InvalidDouble { position, argument } => {
                let found = self.native_found(argument);
                Raised::native_argument_not_a_double(position, &found)
            }
            Refused::NotPositive { position, argument } => {
                let found = self.native_found(argument);
                Raised::native_argument_not_positive(position, &found)
            }
            Refused::NotNonnegative { position, argument } => {
                let found = self.native_found(argument);
                Raised::native_argument_not_nonnegative(position, &found)
            }
            Refused::OutOfRange {
                position,
                min,
                max,
                argument,
            } => {
                let found = self.native_found(argument);
                Raised::native_argument_outside_range(position, min, max, &found)
            }
            Refused::NotLogical { found } => {
                let found = self.native_found(found);
                Raised::native_argument_not_logical(&found)
            }
            Refused::NotArray { argument } => {
                let found = self.native_found(argument);
                Raised::native_argument_not_an_array(&found)
            }
            Refused::NotInstance { position, class } => {
                Raised::native_argument_not_an_instance(position, class.id())
            }
            Refused::NotPointerString { position, argument } => {
                let found = self.native_found(argument);
                Raised::native_argument_not_a_pointer(position, &found)
            }
            Refused::NoStem { position, argument } => {
                let found = self.native_found(argument);
                Raised::native_argument_not_a_stem(method, position, &found)
            }
            Refused::TooManyArguments { expected } => Raised::too_many_external_arguments(expected),
            Refused::Signature | Refused::ResultSignature => {
                let number = refused
                    .error_number(method)
                    .expect("a signature refusal carries its number");
                Raised::incorrect_native_signature(number, refused == Refused::Signature)
            }
            Refused::ClassicStyle => {
                return Loud {
                    message: crate::owned_message(&format!("{refused}"), Some("Phase 10")),
                }
                .into();
            }
            Refused::UnfilledSlot { .. } | Refused::StaleHandle | Refused::Raised => {
                return Loud {
                    message: crate::owned_message(&format!("{refused}"), Some("Phase 8")),
                }
                .into();
            }
        };
        if !packaged {
            raised.delivery.lineless = false;
        }
        raised.into()
    }
}

/// The condition an extension raised, whose number is the major and the minor
/// packed as `major * 1000 + minor` (`api/oorexxerrors.h`).
fn condition_of(number: usize) -> Failure {
    let major = u16::try_from(number / 1000).unwrap_or(u16::MAX);
    let minor = u16::try_from(number % 1000).unwrap_or(0);
    Raised::syntax(major, minor, Vec::new()).into()
}

impl Host for Interp {
    fn is_method(&self) -> bool {
        self.native_handles.last().is_some_and(|frame| frame.method)
    }

    fn string_value(&mut self, object: ObjRef) -> Result<Option<ObjRef>, Condition> {
        match self.blamed_string_conversion(object) {
            Ok(converted) => {
                if let Some(text) = converted {
                    self.roots.push_temp(text);
                }
                Ok(converted)
            }
            Err(failure) => Err(self.hold_native_condition(failure)),
        }
    }

    fn string_bytes(&self, object: ObjRef) -> Option<Cow<'_, [u8]>> {
        if let Some(bytes) = rendered(object) {
            return Some(Cow::Owned(bytes));
        }
        match &self.heap.get(object)?.body {
            Body::Text { bytes, .. } => Some(Cow::Borrowed(bytes.as_slice())),
            Body::Num { text, .. } => text.as_deref().map(Cow::Borrowed),
            _ => None,
        }
    }

    fn cself(&mut self) -> Option<POINTER> {
        let frame = self.native_handles.last()?;
        let (owner, scope) = (frame.owner, frame.scope);
        let held = self.pools_of(owner)?.get(scope, b"CSELF")?;
        match &self.heap.get(held)?.body {
            Body::Instance {
                native: Some(state),
                ..
            } => state.pointer(),
            _ => None,
        }
    }

    fn constants(&mut self) -> Constants<ObjRef> {
        Constants {
            nil: ObjRef::NIL,
            true_object: self.counted(1),
            false_object: self.counted(0),
            null_string: self.text(b""),
        }
    }

    fn set_object_variable(&mut self, name: &[u8], value: Option<ObjRef>) {
        let Some(frame) = self.native_handles.last() else {
            return;
        };
        let (owner, scope) = (frame.owner, frame.scope);
        let Some(name) = pool_variable_name(name) else {
            return;
        };
        match value {
            Some(value) => self.set_pool_variable(owner, scope, &name, value),
            None => self.clear_pool_variable(owner, scope, &name),
        }
    }

    fn drop_object_variable(&mut self, name: &[u8]) {
        self.set_object_variable(name, None);
    }

    fn whole_number(&mut self, value: isize) -> ObjRef {
        match i64::try_from(value).ok().and_then(ObjRef::small_int) {
            Some(object) => object,
            None => self.text(value.to_string().as_bytes()),
        }
    }

    fn new_pointer(&mut self, value: POINTER) -> ObjRef {
        let class = self
            .classes()
            .lookup("Pointer")
            .expect("Pointer is a native class");
        let behaviour = self.classes().instance_behaviour_handle(class);
        let body = Body::pointer(class, behaviour, value);
        let object = self.alloc_with(BehaviourId::OBJECT, body);
        self.roots.push_temp(object);
        object
    }

    fn numeric(&self) -> Numeric {
        let settings = &self.activation().settings;
        Numeric {
            digits: usize::try_from(settings.digits()).unwrap_or(usize::MAX),
            fuzz: usize::try_from(settings.fuzz()).unwrap_or(usize::MAX),
            engineering: settings.form() == rexx_num::Form::Engineering,
        }
    }

    fn double_value(&mut self, object: ObjRef) -> Result<Option<f64>, Condition> {
        if let Decoded::SmallInt(number) = object.decode() {
            #[expect(
                clippy::cast_precision_loss,
                reason = "`RexxInteger::doubleValue` is the same C conversion"
            )]
            return Ok(Some(number as f64));
        }
        let text = self.native_string_conversion(object)?;
        let bytes = self.to_text(text);
        if let Some(number) = Number::parse_bytes(&bytes) {
            return Ok(Some(double_of(&number)));
        }
        Ok(match &bytes[..] {
            b"nan" => Some(f64::NAN),
            b"+infinity" => Some(f64::INFINITY),
            b"-infinity" => Some(f64::NEG_INFINITY),
            _ => None,
        })
    }

    fn signed_integer(
        &mut self,
        object: ObjRef,
        min: i64,
        max: i64,
    ) -> Result<Option<i64>, Condition> {
        let value = match object.decode() {
            Decoded::SmallInt(number) => Some(number),
            _ => {
                let text = self.native_string_conversion(object)?;
                let bytes = self.to_text(text);
                Number::parse_bytes(&bytes).and_then(|number| number.int64_value(DIGITS64))
            }
        };
        Ok(value.filter(|number| (min..=max).contains(number)))
    }

    fn unsigned_integer(&mut self, object: ObjRef, max: u64) -> Result<Option<u64>, Condition> {
        let value = match object.decode() {
            Decoded::SmallInt(number) => u64::try_from(number).ok(),
            _ => {
                let text = self.native_string_conversion(object)?;
                let bytes = self.to_text(text);
                Number::parse_bytes(&bytes).and_then(|number| number.unsigned_int64_value(DIGITS64))
            }
        };
        Ok(value.filter(|number| *number <= max))
    }

    fn logical(&mut self, object: ObjRef) -> Result<Result<bool, ObjRef>, Condition> {
        if let Decoded::SmallInt(number) = object.decode() {
            return Ok(match number {
                0 => Ok(false),
                1 => Ok(true),
                _ => Err(object),
            });
        }
        let text = self.native_string_conversion(object)?;
        let bytes = self.to_text(text);
        Ok(crate::eval::logical_value(&bytes).ok_or(text))
    }

    fn array_value(&mut self, object: ObjRef) -> Result<Option<ObjRef>, Condition> {
        let converted = if self.array_slots_of(object).is_some() {
            Some(object)
        } else {
            match self.request_array_for_over(object) {
                Ok(converted) => converted,
                Err(failure) => return Err(self.hold_native_condition(failure)),
            }
        };
        let Some(array) = converted else {
            return Ok(None);
        };
        self.roots.push_temp(array);
        // `isMultiDimensional`: an array with a dimension list of any length
        // but one.
        Ok(match self.array_body(array) {
            Some((_, dimensions)) if dimensions.is_none_or(|shape| shape.len() == 1) => Some(array),
            _ => None,
        })
    }

    fn is_stem(&self, object: ObjRef) -> bool {
        matches!(
            self.heap.get(object).map(|found| &found.body),
            Some(Body::Stem { .. })
        )
    }

    fn context_stem(&mut self, object: ObjRef) -> Result<Option<ObjRef>, Condition> {
        let text = self.native_string_conversion(object)?;
        let mut name = self.to_text(text).to_ascii_uppercase();
        if name.last() != Some(&b'.') {
            name.push(b'.');
        }
        if classify(&name) != SymbolKind::Stem {
            return Ok(None);
        }
        Ok(Some(self.read_stem(&name)))
    }

    fn is_instance_of(&mut self, object: ObjRef, class: Class) -> bool {
        if class == Class::Class {
            return self.is_class_object(object);
        }
        let (Some(held), Some(wanted)) = (
            self.class_of_value(object),
            self.classes().lookup(class.id()),
        ) else {
            return false;
        };
        self.classes().is_a(held, wanted)
    }

    fn pointer_value(&self, object: ObjRef) -> Option<POINTER> {
        match &self.heap.get(object)?.body {
            Body::Instance {
                native: Some(state),
                ..
            } => state.pointer(),
            _ => None,
        }
    }

    fn string_value_text(&mut self, object: ObjRef) -> Vec<u8> {
        Interp::string_value_text(self, object)
    }

    fn receiver(&mut self) -> ObjRef {
        self.native_frame().receiver
    }

    fn scope(&mut self) -> ObjRef {
        self.native_frame().scope
    }

    fn super_scope(&mut self) -> ObjRef {
        let frame = self.native_frame();
        let (receiver, scope) = (frame.receiver, frame.scope);
        self.super_scope_of(receiver, scope).unwrap_or(ObjRef::NIL)
    }

    fn arguments(&mut self) -> ObjRef {
        let frame = self.native_frame();
        if let Some(list) = frame.argument_list {
            return list;
        }
        let slots = frame.arguments.clone();
        let list = self.alloc_with(
            BehaviourId::ARRAY,
            Body::Array {
                dimensions: None,
                slots,
            },
        );
        self.native_handles
            .last_mut()
            .expect("a native activation is running")
            .argument_list = Some(list);
        list
    }

    fn message_name(&mut self) -> Vec<u8> {
        self.native_frame().name.clone()
    }

    fn unsigned_number(&mut self, value: u64) -> ObjRef {
        let object = match i64::try_from(value).ok().and_then(ObjRef::small_int) {
            Some(object) => object,
            None => self.text(value.to_string().as_bytes()),
        };
        self.roots.push_temp(object);
        object
    }

    fn new_string(&mut self, bytes: &[u8]) -> ObjRef {
        let object = self.text(bytes);
        self.roots.push_temp(object);
        object
    }

    fn double_object(&mut self, value: f64, precision: usize) -> ObjRef {
        let object = self.text(&double_text(value, precision));
        self.roots.push_temp(object);
        object
    }

    fn locals(&mut self) -> &mut Table {
        &mut self
            .native_handles
            .last_mut()
            .expect("a native activation is running")
            .locals
    }
}

impl Interp {
    /// A native refusal's `found` insert: the object's `stringValue()`, which
    /// for an instance sends `OBJECTNAME` (`interpreter/classes/ObjectClass.cpp:1157`)
    /// and so runs a class's own `defaultName`, and which falls back to the
    /// default name where that send raises, as message substitution does
    /// (`interpreter/concurrency/Activity.cpp:1300-1306`).
    fn native_found(&mut self, object: ObjRef) -> Vec<u8> {
        let sends = match self.heap.get(object).map(|held| &held.body) {
            // A state that renders its own `stringValue` answers without a
            // send, which is the arm `Interp::to_text` takes for one; every
            // other instance reaches `RexxObject::stringValue`.
            Some(Body::Instance {
                native: Some(state),
                ..
            }) => !state.renders_its_own_string_value(),
            Some(Body::Instance { .. }) => true,
            _ => false,
        };
        if sends {
            let caller = self.caller();
            if let Ok(Some(answered)) =
                self.send_message(object, super::OBJECTNAME, None, &[], caller)
            {
                self.roots.push_temp(answered);
                return self.string_value_text(answered);
            }
        }
        self.string_value_text(object)
    }

    /// `requestString` for a native argument, holding a raised condition on
    /// the running native frame as [`Host::string_value`] does.
    fn native_string_conversion(&mut self, object: ObjRef) -> Result<ObjRef, Condition> {
        match self.required_string_value(object) {
            Ok(text) => {
                self.roots.push_temp(text);
                Ok(text)
            }
            Err(failure) => Err(self.hold_native_condition(failure)),
        }
    }

    /// Holds `failure` on the running native frame for the call to raise once
    /// the boundary has answered [`Refused::Raised`].
    fn hold_native_condition(&mut self, failure: Failure) -> Condition {
        self.native_handles
            .last_mut()
            .expect("a native activation is running")
            .raised = Some(failure);
        Condition
    }

    /// [`Interp::native_frame`], mutably.
    ///
    /// # Panics
    /// As [`Interp::native_frame`].
    #[cfg(test)]
    fn native_frame_mut(&mut self) -> &mut NativeFrame {
        self.native_handles
            .last_mut()
            .expect("a native activation is running")
    }

    /// The running native activation.
    ///
    /// # Panics
    /// If no native activation is running, which is the only time the host is
    /// asked.
    fn native_frame(&self) -> &NativeFrame {
        self.native_handles
            .last()
            .expect("a native activation is running")
    }
}

/// A Rexx number as the C `double` `strtod` reads from its digits
/// (`NumberString::doubleValue`, `interpreter/classes/NumberStringClass.cpp:704`).
///
/// Rendered at as many digits as the number has, which keeps every digit.
/// `Number::format` takes the exponential form where the plain one would
/// pad the integer part with zeros or put more zeros after the point than
/// there are digits, so the text stays within about twice the digits.
fn double_of(number: &Number) -> f64 {
    let written = double_literal(number);
    written
        .parse()
        .unwrap_or_else(|_| unreachable!("a formatted Rexx number is a float literal: {written}"))
}

/// The float literal [`double_of`] reads.
fn double_literal(number: &Number) -> String {
    let digits = u64::try_from(number.digit_count()).unwrap_or(u64::MAX);
    number.format(digits)
}

/// `NumberString::newInstanceFromDouble(value, precision)`
/// (`interpreter/classes/NumberStringClass.cpp:4073`) as its string value:
/// `%.*g` at two digits past the precision, capped at sixteen, then rounded
/// to the precision and formatted at it.
fn double_text(value: f64, precision: usize) -> Vec<u8> {
    if value.is_nan() {
        return b"nan".to_vec();
    }
    if value == f64::INFINITY {
        return b"+infinity".to_vec();
    }
    if value == f64::NEG_INFINITY {
        return b"-infinity".to_vec();
    }
    let printed = percent_g(value, precision.min(16) + 2);
    let digits = u64::try_from(precision).unwrap_or(u64::MAX);
    Number::parse(&printed)
        .unwrap_or_else(|| unreachable!("%g prints a Rexx number: {printed}"))
        .into_round(digits)
        .format(digits)
        .into_bytes()
}

/// C's `%.*g` with `significant` digits: the style `%e` would take where its
/// exponent is below -4 or not below `significant`, the style `%f` takes
/// otherwise, and in either no trailing zero after the point.
fn percent_g(value: f64, significant: usize) -> String {
    let scientific = format!("{:.*e}", significant - 1, value);
    let (mantissa, exponent) = scientific
        .split_once('e')
        .unwrap_or_else(|| unreachable!("{{:e}} prints an exponent: {scientific}"));
    let exponent: i64 = exponent
        .parse()
        .unwrap_or_else(|_| unreachable!("{{:e}} prints a decimal exponent: {scientific}"));
    let (sign, mantissa) = match mantissa.strip_prefix('-') {
        Some(rest) => ("-", rest),
        None => ("", mantissa),
    };
    let digits: String = mantissa.chars().filter(char::is_ascii_digit).collect();
    let width = i64::try_from(significant).unwrap_or(i64::MAX);
    if exponent < -4 || exponent >= width {
        let kept = digits.trim_end_matches('0');
        let kept = if kept.is_empty() { "0" } else { kept };
        let (first, rest) = kept.split_at(1);
        let point = if rest.is_empty() { "" } else { "." };
        return format!("{sign}{first}{point}{rest}e{exponent}");
    }
    if exponent >= 0 {
        let (whole, fraction) = digits.split_at(usize::try_from(exponent + 1).unwrap_or(0));
        let fraction = fraction.trim_end_matches('0');
        if fraction.is_empty() {
            return format!("{sign}{whole}");
        }
        return format!("{sign}{whole}.{fraction}");
    }
    let zeros = "0".repeat(usize::try_from(-exponent - 1).unwrap_or(0));
    format!("{sign}0.{zeros}{}", digits.trim_end_matches('0'))
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use std::rc::Rc;

    use rexx_api::values::Failure as Refused;

    use rexx_num::Number;

    use super::{double_literal, double_of, double_text, percent_g, pool_variable_name};
    use crate::{Failure, Interp, LibraryLoad};

    /// This worktree's own `build/lib`, not the oracle checkout's: a second
    /// build of the extensions from the same sources, which the oracle never
    /// runs (D5's amendment, `docs/superpowers/specs/2026-09-14-phase-8-native-api.md`).
    /// Its libraries are loaded, never rebuilt.
    fn worktree_library_directory() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../build/lib")
            .canonicalize()
            .expect("the worktree's build/lib is three directories above this crate")
    }

    /// The oracle's `librxregexp.so`, opened by path.
    fn open_rxregexp() -> rexx_api::load::Library {
        rexx_api::load::open_path(
            &worktree_library_directory().join("librxregexp.so"),
            "rxregexp",
        )
        .expect("the extension asks for 4.0.0, which is below this interpreter")
        .expect("the extension publishes RexxGetPackage")
    }

    /// An interpreter started with an `LD_LIBRARY_PATH` naming that
    /// directory. The process variable is not touched: this is the
    /// interpreter's copy, which is the one the search reads.
    fn interp_that_can_see_rxregexp() -> Interp {
        let mut interp = Interp::new();
        interp.adopt_environment(vec![(
            b"LD_LIBRARY_PATH".to_vec(),
            worktree_library_directory()
                .into_os_string()
                .into_encoded_bytes(),
        )]);
        interp
    }

    /// **One resolution path, and the assertion is on the side effect rather
    /// than on the answer.** Two `Loaded` answers would look alike whether or
    /// not the second one re-opened the library, and so would two answers
    /// holding the same `Rc`, because a non-replacing hold hands the first one
    /// back; the attempt count is what says the `dlopen` and the package read
    /// happened once.
    #[test]
    fn a_library_named_twice_is_opened_once() {
        let mut interp = interp_that_can_see_rxregexp();
        let LibraryLoad::Loaded(first) = interp.resolve_library(b"rxregexp") else {
            panic!("librxregexp.so did not load from the worktree's build directory");
        };
        let LibraryLoad::Loaded(second) = interp.resolve_library(b"rxregexp") else {
            panic!("the second resolve did not load");
        };
        assert_eq!(
            interp.library_open_attempts, 1,
            "the same name was opened twice"
        );
        assert!(Rc::ptr_eq(&first, &second));

        // The control that says the count sees an attempt at all: a name
        // nothing is held for is looked for every time it is asked, whether or
        // not anything loads.
        assert!(matches!(
            interp.resolve_library(b"zorkolib"),
            LibraryLoad::Missing
        ));
        assert!(matches!(
            interp.resolve_library(b"zorkolib"),
            LibraryLoad::Missing
        ));
        assert_eq!(interp.library_open_attempts, 3);
    }

    /// The search directories come from the interpreter's own environment,
    /// which is what makes the name reachable at all: without the variable the
    /// same name answers nothing.
    #[test]
    fn the_search_path_is_what_makes_the_name_resolve() {
        let mut bare = Interp::new();
        bare.adopt_environment(Vec::new());
        assert!(
            matches!(bare.resolve_library(b"rxregexp"), LibraryLoad::Missing),
            "librxregexp.so is on the process loader's own path, so the test above \
             is not reading the directory it supplies"
        );
        let mut seeing = interp_that_can_see_rxregexp();
        assert!(matches!(
            seeing.resolve_library(b"rxregexp"),
            LibraryLoad::Loaded(_)
        ));
    }

    /// The search is the one the interpreter started with: a directory written
    /// into its `LD_LIBRARY_PATH` afterwards, as `VALUE(..., 'ENVIRONMENT')`
    /// writes it, is not searched. The control is the same directory handed in
    /// at the start.
    #[test]
    fn a_search_directory_written_after_the_start_is_not_searched() {
        let directory = worktree_library_directory()
            .into_os_string()
            .into_encoded_bytes();
        let mut late = Interp::new();
        late.adopt_environment(Vec::new());
        late.env_set(b"LD_LIBRARY_PATH", Some(directory));
        assert!(matches!(
            late.resolve_library(b"rxregexp"),
            LibraryLoad::Missing
        ));
        let mut early = interp_that_can_see_rxregexp();
        assert!(matches!(
            early.resolve_library(b"rxregexp"),
            LibraryLoad::Loaded(_)
        ));
    }

    /// A name that resolved to nothing is not held, so the next ask opens
    /// again: `PackageManager::loadLibrary` removes a package whose load
    /// answered false (`interpreter/package/PackageManager.cpp:240-244`).
    #[test]
    fn a_name_that_resolves_to_nothing_is_not_held() {
        let mut interp = Interp::new();
        assert!(matches!(
            interp.settle_library(b"zorkolib", Ok(None)),
            LibraryLoad::Missing
        ));
        assert!(interp.libraries.get(b"zorkolib").is_none());
    }

    /// A library whose version check refused raises on the ask that opened it
    /// and is held, so every later ask answers it loaded without opening
    /// anything: measured on the oracle with a forged package entry asking
    /// for 6.0.0, `loadLibrary` is 98.982 and then `1`, and a method of the
    /// library binds and runs afterwards.
    #[test]
    fn a_version_refused_library_raises_once_and_is_held() {
        let mut interp = Interp::new();
        let refused = rexx_api::load::Refused {
            failure: rexx_api::load::Failure::LibraryVersion("forgever".to_owned()),
            library: Box::new(open_rxregexp()),
        };
        assert!(matches!(
            interp.settle_library(b"forgever", Err(refused)),
            LibraryLoad::Version
        ));
        // No `forgever` is on any search path, so an ask that opened again
        // would answer `Missing`.
        let LibraryLoad::Loaded(held) = interp.resolve_library(b"forgever") else {
            panic!("the refused library was not held");
        };
        assert!(held.method(b"RegExp_Parse").is_some());
        assert_eq!(
            interp.library_open_attempts, 0,
            "the held library was opened again"
        );
    }

    /// **The end-to-end witness answers the same under a collection at every
    /// allocation as it does under none.** The stress harness cannot say this:
    /// `collect_stress` reads `phase-8.txt` last and aborts on a pre-existing
    /// panic before it gets there.
    ///
    /// **What it does not say**, measured: with `Interp::object_roots`
    /// dropping `native_handles` entirely this still passes, because the
    /// receiver is rooted by the send's own temps and `Host::new_pointer`
    /// pushes what it mints as a temp before handing it back. Whether the
    /// frame's own rooting is live is
    /// `a_native_activations_local_references_are_roots_only_while_it_lives`,
    /// which does fail under that edit.
    #[test]
    fn a_library_call_answers_the_same_under_a_collection_at_every_allocation() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../corpus/lang/library_method_external.rex");
        let text = std::fs::read(&path).expect("the corpus witness is readable");
        let invocation = || {
            crate::Invocation::none().with_environment(vec![(
                b"LD_LIBRARY_PATH".to_vec(),
                worktree_library_directory()
                    .into_os_string()
                    .into_encoded_bytes(),
            )])
        };
        let name = path.to_string_lossy().into_owned();

        let plain = crate::run_program(&name, text.clone(), invocation());
        // The value the comparison rests on, so that two identical failures
        // cannot pass for agreement.
        assert_eq!(
            plain.exit_code,
            0,
            "{}",
            String::from_utf8_lossy(&plain.stderr)
        );
        assert_eq!(
            String::from_utf8_lossy(&plain.stdout),
            "parse 0\npos 3\nmatch 1\nlastpos 3\nreparse 0\nmatch 1\nminimal 0\nfind 3\nfindpos 5\n"
        );

        let swept = crate::run_program_collect_every_alloc(&name, text, invocation());
        assert_eq!(swept.exit_code, plain.exit_code);
        assert_eq!(swept.stdout, plain.stdout);
        assert_eq!(swept.stderr, plain.stderr);
    }

    /// **The routine half answers the same under a collection at every
    /// allocation as under none**: a merged and a loaded routine, a double
    /// the extension builds, an argument refused after its string conversion,
    /// and a `loadExternalMethod` answer a class defines. The stdout is the
    /// oracle's, measured.
    #[test]
    fn a_library_routine_answers_the_same_under_a_collection_at_every_allocation() {
        let text = b"say 'merged' RxCalcSqrt(2) RxCalcPower(10, 10) RxCalcSqrt('nan')\n\
            signal on syntax\n\
            say RxCalcSqrt(.object~new)\n\
            syntax:\n\
            say 'refused' condition('O')~code\n\
            r = .Routine~loadExternalRoutine('r', 'LIBRARY rxmath RxCalcPi')\n\
            say 'loaded' r~call(12) r~callWith(.array~of(3))\n\
            say 'found' .context~package~findRoutine('RXCALCSQRT')~call(81)\n\
            .K~define('DOES', .Method~loadExternalMethod('m', 'LIBRARY rxregexp RegExp_Match'))\n\
            .K~define('INIT', .Method~loadExternalMethod('i', 'LIBRARY rxregexp RegExp_Init'))\n\
            say 'defined' .K~new('a*b')~does('aab')\n\
            ::requires 'rxmath' LIBRARY\n\
            ::class K\n"
            .to_vec();
        let invocation = || {
            crate::Invocation::none().with_environment(vec![(
                b"LD_LIBRARY_PATH".to_vec(),
                worktree_library_directory()
                    .into_os_string()
                    .into_encoded_bytes(),
            )])
        };
        let name = "/tmp/routine_stress.rex";

        let plain = crate::run_program(name, text.clone(), invocation());
        assert_eq!(
            plain.exit_code,
            0,
            "{}",
            String::from_utf8_lossy(&plain.stderr)
        );
        assert_eq!(
            String::from_utf8_lossy(&plain.stdout),
            "merged 1.41421356 1.00000000E+10 nan\nrefused 88.921\nloaded 3.14159265359 3.14\n\
             found 9\ndefined 1\n"
        );

        let swept = crate::run_program_collect_every_alloc(name, text, invocation());
        assert_eq!(swept.exit_code, plain.exit_code);
        assert_eq!(swept.stdout, plain.stdout);
        assert_eq!(swept.stderr, plain.stderr);
        assert!(swept.collections > 0, "the stress mode did not collect");
    }

    /// **The conversion rows answer the same under a collection at every
    /// allocation as under none**: integers at their ends, a float, a double,
    /// a logical, a `CSTRING` both ways, the `size_t` result, an array made by
    /// conversion, a class, pointers, stems by object and by name, the special
    /// arguments, and a refusal. The stdout is the oracle's, measured. It reads
    /// a refusal's `code` and not its `message`: `signal on syntax`, `y = 1/0`,
    /// then `say condition('O')~message` in the syntax handler panics in this
    /// mode with no native call, at the assertion `collect_stress`'s L0 test
    /// fails at.
    #[test]
    fn the_conversion_rows_answer_the_same_under_a_collection_at_every_allocation() {
        let text = b"t = .T~new\n\
            say 'int' t~int8(-128) t~uint64('18446744073709551615') t~int64('-9223372036854775808') t~size('1E19')\n\
            say 'num' t~float(1.1) t~double(2/3) t~logical(1) t~cstring('abc') t~version\n\
            say 'obj' t~array('abc')~items t~array(.list~of(1, 2))~items t~classarg(.string) t~object(t)~class\n\
            say 'ptr' t~pointerarg(t~pointervalue) t~nullpointerstring t~pointerstringarg(t~pointerstringvalue)\n\
            y.7 = 'seven'\n\
            say 'stem' TestStemArg('y')[7] t~stem(y.)[7]\n\
            drop qq.\n\
            say 'unset' TestStemArg('qq')~class symbol('QQ.')\n\
            al = t~arglist(1, , 3)\n\
            say 'special' al~size al~items (t~oself == t) t~scope t~super t~name TestNameArg() TestArglistArg(1, 2)~items\n\
            say 'rxmath ['MathLoadFuncs()']'\n\
            signal on syntax\n\
            say t~int8(128)\n\
            exit\n\
            syntax:\n\
            say 'refused' condition('O')~code\n\
            ::requires 'orxfunction' LIBRARY\n\
            ::requires 'rxmath' LIBRARY\n\
            ::class Base\n\
            ::class T subclass Base\n\
            ::method int8 external \"LIBRARY orxmethod TestInt8Arg\"\n\
            ::method uint64 external \"LIBRARY orxmethod TestUint64Arg\"\n\
            ::method int64 external \"LIBRARY orxmethod TestInt64Arg\"\n\
            ::method size external \"LIBRARY orxmethod TestSizeArg\"\n\
            ::method float external \"LIBRARY orxmethod TestFloatArg\"\n\
            ::method double external \"LIBRARY orxmethod TestDoubleArg\"\n\
            ::method logical external \"LIBRARY orxmethod TestLogicalArg\"\n\
            ::method cstring external \"LIBRARY orxmethod TestCstringArg\"\n\
            ::method version external \"LIBRARY orxmethod TestInterpreterVersion\"\n\
            ::method array external \"LIBRARY orxmethod TestArrayArg\"\n\
            ::method classarg external \"LIBRARY orxmethod TestClassArg\"\n\
            ::method object external \"LIBRARY orxmethod TestObjectArg\"\n\
            ::method pointervalue external \"LIBRARY orxmethod TestPointerValue\"\n\
            ::method pointerarg external \"LIBRARY orxmethod TestPointerArg\"\n\
            ::method pointerstringvalue external \"LIBRARY orxmethod TestPointerStringValue\"\n\
            ::method pointerstringarg external \"LIBRARY orxmethod TestPointerStringArg\"\n\
            ::method nullpointerstring external \"LIBRARY orxmethod TestNullPointerStringValue\"\n\
            ::method stem external \"LIBRARY orxmethod TestStemArg\"\n\
            ::method arglist external \"LIBRARY orxmethod TestArglistArg\"\n\
            ::method oself external \"LIBRARY orxmethod TestOSelfArg\"\n\
            ::method scope external \"LIBRARY orxmethod TestScopeArg\"\n\
            ::method super external \"LIBRARY orxmethod TestSuperArg\"\n\
            ::method name external \"LIBRARY orxmethod TestNameArg\"\n\
            "
        .to_vec();
        let invocation = || {
            crate::Invocation::none().with_environment(vec![(
                b"LD_LIBRARY_PATH".to_vec(),
                worktree_library_directory()
                    .into_os_string()
                    .into_encoded_bytes(),
            )])
        };
        let name = "/tmp/conversion_stress.rex";

        let plain = crate::run_program(name, text.clone(), invocation());
        assert_eq!(
            plain.exit_code,
            0,
            "{}",
            String::from_utf8_lossy(&plain.stderr)
        );
        assert_eq!(
            String::from_utf8_lossy(&plain.stdout),
            "int -128 18446744073709551615 -9223372036854775808 10000000000000000000\n\
             num 1.10000002 0.666666667 1 abc 328448\n\
             obj 1 2 The String class The T class\n\
             ptr 1 0x0 1\n\
             stem seven seven\n\
             unset The Stem class VAR\n\
             special 3 2 1 The T class The BASE class NAME TESTNAMEARG 2\n\
             rxmath []\n\
             refused 88.907\n"
        );

        let swept = crate::run_program_collect_every_alloc(name, text, invocation());
        assert_eq!(swept.exit_code, plain.exit_code);
        assert_eq!(swept.stdout, plain.stdout);
        assert_eq!(swept.stderr, plain.stderr);
        assert!(swept.collections > 0, "the stress mode did not collect");
    }

    /// A native call's frame is reused by the next call with nothing of the
    /// last one left in it: a handle the last call registered resolves to
    /// nothing, and its arguments, argument array, name and condition are
    /// gone.
    #[test]
    fn a_reused_native_frame_holds_nothing_of_the_call_before() {
        let mut interp = Interp::new();
        let object = interp.text(b"held by the first call");
        interp.push_native_frame(
            rexx_core::ObjRef::NIL,
            rexx_core::ObjRef::NIL,
            Some(object),
            b"FIRST",
            &[Some(object), None],
        );
        let handle = interp.native_frame_mut().locals.register(object);
        interp.native_frame_mut().argument_list = Some(object);
        interp.native_frame_mut().raised = Some(crate::Loud::library_procedure_gone().into());
        let (raised, method) = interp.pop_native_frame();
        assert!(raised.is_some() && method);

        interp.push_native_frame(
            rexx_core::ObjRef::NIL,
            rexx_core::ObjRef::NIL,
            None,
            b"SECOND",
            &[],
        );
        assert_eq!(
            interp.native_spares.len(),
            0,
            "the spare frame was not reused"
        );
        let frame = interp.native_frame_mut();
        assert_eq!(frame.locals.resolve(handle), None);
        assert_eq!(frame.name, b"SECOND");
        assert!(frame.arguments.is_empty());
        assert_eq!(frame.argument_list, None);
        assert!(frame.raised.is_none());
        assert!(!frame.method);
        assert_eq!(frame.receiver, rexx_core::ObjRef::NIL);
    }

    /// A name that has loaded keeps the library it loaded, which is what stops
    /// a second write dropping the interpreter's own reference to it.
    #[test]
    fn a_held_library_is_not_replaced_by_a_later_answer() {
        let (first, second) = (Rc::new(open_rxregexp()), Rc::new(open_rxregexp()));
        let mut libraries = crate::Libraries::new();
        assert!(Rc::ptr_eq(
            &libraries.hold(b"rxregexp", Rc::clone(&first)),
            &first
        ));
        assert!(Rc::ptr_eq(
            &libraries.hold(b"rxregexp", Rc::clone(&second)),
            &first
        ));
        assert!(Rc::ptr_eq(
            libraries.get(b"rxregexp").expect("a library was held"),
            &first
        ));
        // The control that says the second library was refused rather than
        // never built: under a name nothing is held for it is taken.
        assert!(Rc::ptr_eq(
            &libraries.hold(b"zorkolib", Rc::clone(&second)),
            &second
        ));
    }

    /// A loaded library survives a collection and the finalizer sweep, which
    /// is what an object still holding a `CSELF` at termination needs: its
    /// `UNINIT` is an address inside the mapping.
    #[test]
    fn a_loaded_library_outlives_the_finaliser_sweep() {
        let mut interp = interp_that_can_see_rxregexp();
        let LibraryLoad::Loaded(library) = interp.resolve_library(b"rxregexp") else {
            panic!("librxregexp.so did not load from the worktree's build directory");
        };
        let watch = std::rc::Rc::downgrade(&library);
        drop(library);
        interp.collect_now();
        assert!(interp.run_termination_uninits().is_empty());
        assert!(
            watch.upgrade().is_some(),
            "the interpreter gave up its library across a collection and the sweep"
        );

        // The control, and what says the assertion above is about the
        // interpreter's hold rather than some other one: the interpreter is
        // the only holder, so the watch reports a release as soon as it goes.
        drop(interp);
        assert!(
            watch.upgrade().is_none(),
            "something outside the interpreter holds the library, so the watch \
             above cannot see a release"
        );
    }

    /// **A finalizer allocates**, and nothing about the sweep it runs in
    /// forbids that: a collection only readies an object, and the sweep is
    /// reached afterwards from `GC('force')` and from termination. The
    /// allocating part of this witness is the Rexx subclass finalizer around
    /// the native one, which sends and says; `RegExp_Uninit` itself allocates
    /// nothing. The witness answers the same with a collection at every
    /// allocation as with none.
    #[test]
    fn the_native_finaliser_answers_the_same_under_a_collection_at_every_allocation() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../corpus/lang/library_uninit_collected.rex");
        let text = std::fs::read(&path).expect("the corpus witness is readable");
        let invocation = || {
            crate::Invocation::none().with_environment(vec![(
                b"LD_LIBRARY_PATH".to_vec(),
                worktree_library_directory()
                    .into_os_string()
                    .into_encoded_bytes(),
            )])
        };
        let name = path.to_string_lossy().into_owned();

        let plain = crate::run_program(&name, text.clone(), invocation());
        assert_eq!(
            plain.exit_code,
            0,
            "{}",
            String::from_utf8_lossy(&plain.stderr)
        );
        // The oracle's own bytes, so that two identical failures cannot pass
        // for agreement: `sub after String` is `DropObjectVariable` reaching
        // the pool, which only the extension's own code writes.
        assert_eq!(
            String::from_utf8_lossy(&plain.stdout),
            "live Pointer\nsub before Pointer\nsub after String\ndone\n"
        );

        let swept = crate::run_program_collect_every_alloc(&name, text, invocation());
        assert_eq!(swept.exit_code, plain.exit_code);
        assert_eq!(swept.stdout, plain.stdout);
        assert_eq!(swept.stderr, plain.stderr);
        assert!(swept.collections > 0, "the stress mode did not collect");
    }

    /// Which refusals are reported against the declaring package with no
    /// line. Measured, oracle rc 163 and 168: 93.968 for a parameter code and
    /// 88.909 for an argument with no string value name the package; 93.968
    /// for a return word carrying `OPTIONAL` names the sending clause's line.
    #[test]
    fn a_refusal_before_the_call_is_lineless_and_one_after_it_is_not() {
        let mut interp = Interp::new();
        let mut lineless =
            |refused: Refused, packaged: bool| match interp.refusal(refused, packaged, true) {
                Failure::Raised(raised) => raised.delivery.lineless,
                _ => panic!("every condition-shaped refusal is a raise"),
            };
        assert!(lineless(Refused::MissingArgument { position: 1 }, true));
        assert!(lineless(Refused::NoStringValue { position: 1 }, true));
        assert!(lineless(Refused::TooManyArguments { expected: 0 }, true));
        assert!(lineless(Refused::Signature, true));
        assert!(!lineless(Refused::ResultSignature, true));
        // Measured, oracle rc 168: `RxCalcSqrt()` through a routine no
        // directive has bound is `Error 88 running <the caller> line 2`.
        assert!(!lineless(Refused::MissingArgument { position: 1 }, false));
        assert!(!lineless(Refused::TooManyArguments { expected: 2 }, false));
    }

    /// A signature refusal is 93.968 in a method and 40.918 in a routine.
    /// Measured through a forged routine library, oracle rc 216: a routine
    /// declaring `CSELF` is `Error 40.918:  Invalid native function signature
    /// specification.` with no line where a directive bound it, and a result
    /// word carrying the optional bit the same with the sending clause's line.
    #[test]
    fn a_signature_refusal_is_numbered_for_a_method_or_a_routine() {
        let mut interp = Interp::new();
        let mut numbered = |refused: Refused, method: bool| match interp.settle_native_call(
            Err(refused),
            None,
            method,
            true,
        ) {
            Err(Failure::Raised(raised)) => (raised.number, raised.sub, raised.delivery.lineless),
            _ => panic!("a signature refusal is a raise"),
        };
        assert_eq!(numbered(Refused::Signature, true), (93, 968, true));
        assert_eq!(numbered(Refused::Signature, false), (40, 918, true));
        assert_eq!(numbered(Refused::ResultSignature, true), (93, 968, false));
        assert_eq!(numbered(Refused::ResultSignature, false), (40, 918, false));
    }

    /// A number past either end of a double's range converts to what
    /// `strtod` answers without being written out digit by digit: measured,
    /// oracle, `RxCalcSqrt('1E+999999999')` is `+infinity` and
    /// `RxCalcSqrt('1E-999999999')` is `0` under a 1 GiB address-space cap.
    #[test]
    fn a_double_argument_is_read_from_its_digits_and_exponent() {
        let read = |text: &str| double_of(&Number::parse(text).expect("a Rexx number"));
        assert_eq!(read("1E+999999999"), f64::INFINITY);
        assert_eq!(read("-9.99E+999999999"), f64::NEG_INFINITY);
        assert_eq!(read("1E-999999999"), 0.0);
        assert_eq!(read("4.9E-324"), 4.9e-324);
        assert_eq!(read("1.7976931348623157E+308"), f64::MAX);
        assert_eq!(
            read("123456789012345678901234567890E-10"),
            1.2345678901234567e19
        );
        assert_eq!(read("0.000000000000000000000000000000000000001"), 1e-39);
        assert_eq!(read("2.25"), 2.25);
        assert_eq!(read("-16"), -16.0);
        let written = double_literal(&Number::parse("1E+999999999").expect("a Rexx number"));
        assert_eq!(written, "1E+999999999");
    }

    /// `%g`'s two styles and its trailing zeros, against what glibc prints.
    #[test]
    fn percent_g_prints_what_c_prints() {
        assert_eq!(percent_g(4.0, 11), "4");
        assert_eq!(percent_g(2f64.sqrt(), 11), "1.4142135624");
        assert_eq!(percent_g(1e10, 11), "10000000000");
        assert_eq!(percent_g(1e10, 5), "1e10");
        assert_eq!(percent_g(1e-5, 11), "1e-5");
        assert_eq!(percent_g(1e-4, 11), "0.0001");
        assert_eq!(percent_g(-2.5, 3), "-2.5");
        assert_eq!(percent_g(9.99, 2), "10");
        assert_eq!(percent_g(0.0, 11), "0");
    }

    /// Measured on the oracle through `rxmath`, which hands its precision
    /// straight to `DoubleToObjectWithPrecision`.
    #[test]
    fn a_double_renders_as_the_oracle_renders_it() {
        let rendered = |value: f64, precision: usize| {
            String::from_utf8(double_text(value, precision)).expect("ASCII")
        };
        assert_eq!(rendered(4.0, 9), "4");
        assert_eq!(rendered(2f64.sqrt(), 9), "1.41421356");
        assert_eq!(rendered(2f64.sqrt(), 3), "1.41");
        assert_eq!(rendered(2f64.sqrt(), 16), "1.414213562373095");
        assert_eq!(rendered(1e10, 9), "1.00000000E+10");
        assert_eq!(rendered(1e10, 3), "1E+10");
        assert_eq!(rendered(1e9, 9), "1.00000000E+9");
        assert_eq!(rendered(1e8, 9), "100000000");
        assert_eq!(rendered(1e-5, 9), "0.00001");
        assert_eq!(rendered(9.99, 2), "10");
        assert_eq!(rendered(123_456_789.0, 5), "1.2346E+8");
        assert_eq!(rendered(f64::NAN, 9), "nan");
        assert_eq!(rendered(f64::INFINITY, 9), "+infinity");
        assert_eq!(rendered(0.0, 9), "0");
    }

    /// The spellings `getVariableRetriever` answers nothing for, beside
    /// `CSELF` and `!POS`, which `rxregexp` really writes.
    #[test]
    fn a_pool_name_is_upcased_and_a_compound_one_is_refused() {
        assert_eq!(pool_variable_name(b"CSELF").as_deref(), Some(&b"CSELF"[..]));
        assert_eq!(pool_variable_name(b"!POS").as_deref(), Some(&b"!POS"[..]));
        assert_eq!(pool_variable_name(b"cself").as_deref(), Some(&b"CSELF"[..]));
        assert_eq!(pool_variable_name(b""), None);
        assert_eq!(pool_variable_name(b"a.b"), None);
        assert_eq!(pool_variable_name(b"1x"), None);
    }
}
