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

use rexx_api::handles::Table;
use rexx_api::invoke;
use rexx_api::layout::POINTER;
use rexx_api::load::{Hook, Library};
use rexx_api::values::{
    Activation, CStringPool, Class, Constants, Conversion, Failure as Refused, Host, Numeric,
    Raised as Condition,
};
use rexx_core::{BehaviourId, Body, Decoded, ObjRef};
use rexx_num::{DIGITS64, Number};

use super::Resolution;
use crate::builtin::datatype::{SymbolKind, classify};
use crate::error::Raised;
use crate::{Failure, Interp, LibraryBinding, Loud, NativeFrame, PendingTrap};

/// The bytes a small integer or an inline string renders as, for a reader
/// that cannot allocate into the interpreter.
fn rendered(value: ObjRef) -> Option<Vec<u8>> {
    match value.decode() {
        Decoded::SmallInt(number) => Some(number.to_string().into_bytes()),
        Decoded::Text(inline) => Some(inline.to_vec()),
        _ => None,
    }
}

/// The pool name an extension's spelling addresses, a simple or a stem
/// name, or `None` for one `getObjectVariableRetriever`
/// (`execution/NativeActivation.cpp:3002`) answers nothing for: a constant
/// symbol or a compound name. The write then does nothing, which is all the
/// API can see of the refusal.
fn pool_variable_name(name: &[u8]) -> Option<Vec<u8>> {
    let first = *name.first()?;
    if first.is_ascii_digit()
        || first == b'.'
        || crate::run::shape_of(name) == crate::run::NameShape::Compound
    {
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
        self.push_native_frame(owner, resolution.scope, Some(receiver), name, args, None);
        let mut strings = CStringPool::new();
        let thread = self.thread.clone();
        let (answered, pending) = {
            let activation = Activation::new(Conversion {
                host: self,
                strings: &mut strings,
            });
            let answered = thread.enter(&activation, |contexts| {
                invoke::method(entry, &contexts.method(), &activation, args)
            });
            (answered, activation.pending())
        };
        let popped = self.pop_native_frame();

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
        self.settle_native_call(answered, popped, packaged)
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
        self.push_native_frame(ObjRef::NIL, ObjRef::NIL, None, name, args, Some(code));
        let mut strings = CStringPool::new();
        let thread = self.thread.clone();
        let (answered, pending) = {
            let activation = Activation::new(Conversion {
                host: self,
                strings: &mut strings,
            });
            let answered = thread.enter(&activation, |contexts| {
                invoke::routine(entry, &contexts.call(), &activation, args)
            });
            (answered, activation.pending())
        };
        let popped = self.pop_native_frame();
        let package = self.library_code_package_path(code);
        let outcome = match pending {
            Some(number) => Err(condition_of(number)),
            None => self.settle_native_call(answered, popped, package.is_some()),
        };
        if outcome.is_err() {
            self.blame_native_routine(name, package);
        }
        outcome
    }

    /// Runs `library`'s `hook` in a native frame of its own, as
    /// `Activity::run` runs a `CallbackDispatcher`
    /// (`interpreter/concurrency/Activity.cpp:3461-3474`).
    ///
    /// # Errors
    /// The condition the hook raised, and [`Loud`] for an interface member it
    /// reached that this phase has not written.
    pub(crate) fn run_package_hook(
        &mut self,
        library: &Library,
        hook: Hook,
    ) -> Result<(), Failure> {
        if !library.has_hook(hook) {
            return Ok(());
        }
        self.push_native_frame(ObjRef::NIL, ObjRef::NIL, None, b"", &[], None);
        let mut strings = CStringPool::new();
        let thread = self.thread.clone();
        let (ran, pending) = {
            let activation = Activation::new(Conversion {
                host: self,
                strings: &mut strings,
            });
            let ran = thread.enter(&activation, |contexts| {
                invoke::hook(library, hook, contexts, &activation)
            });
            (ran, activation.pending())
        };
        let popped = self.pop_native_frame();
        if let Some(number) = pending {
            return Err(condition_of(number));
        }
        self.settle_native_call(ran.map(|()| None), popped, true)
            .map(|_| ())
    }

    /// `Interpreter::terminateInterpreter`'s two steps that run extension or
    /// Rexx code (`runtime/Interpreter.cpp:279-281`): the last-chance
    /// `UNINIT`s, then the package unloaders. Answers the refusals they met.
    pub(crate) fn terminate(&mut self) -> Vec<Loud> {
        let mut refused = self.run_termination_uninits();
        refused.extend(self.run_package_unloaders());
        refused
    }

    /// `PackageManager::unload` (`interpreter/package/PackageManager.cpp:642-650`):
    /// each held library's unloader, in the order the oracle's table walks
    /// them, each library closed straight after its own
    /// (`LibraryPackage::unload`, `interpreter/package/LibraryPackage.cpp:166-181`),
    /// answering the refusal that ended the walk if one did.
    ///
    /// Measured, oracle: a condition raised inside an unloader ends the walk,
    /// reports nothing, leaves the exit status alone and leaves that library
    /// and every later one open. A refusal ends it too.
    fn run_package_unloaders(&mut self) -> Option<Loud> {
        for (_, library) in self.libraries.in_unload_order() {
            match self.run_package_hook(&library, Hook::Unloader) {
                Ok(()) => {
                    library.close();
                }
                Err(Failure::Loud(loud)) => return Some(*loud),
                Err(_) => return None,
            }
        }
        None
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
        popped: Popped,
        packaged: bool,
    ) -> Result<Option<ObjRef>, Failure> {
        match answered {
            Err(Refused::Raised) => Err(popped
                .raised
                .expect("a host answering Raised holds the condition it raised")),
            Ok(value) => match popped.raised {
                None => Ok(value),
                Some(raised) => self.raise_held_condition(raised, popped.additional, popped.result),
            },
            Err(refused) => Err(self.refusal(refused, packaged, popped.method)),
        }
    }

    /// A condition a callback raised and the extension did not clear, raised
    /// once the call has returned (`NativeActivation::checkConditions`,
    /// `:1787`): a `SYNTAX` condition as the call's failure; any other is
    /// offered to the caller's trap, and the call answers its `RESULT` where
    /// no `SIGNAL ON` takes it.
    fn raise_held_condition(
        &mut self,
        raised: Failure,
        additional: Option<ObjRef>,
        result: Option<ObjRef>,
    ) -> Result<Option<ObjRef>, Failure> {
        let condition = match &raised {
            Failure::Raised(held) if held.condition != "SYNTAX" => held.condition.to_string(),
            _ => {
                if additional.is_some() {
                    self.pending_additional = additional;
                }
                return Err(raised);
            }
        };
        let Failure::Raised(held) = &raised else {
            unreachable!("matched as a raise above")
        };
        match self.trap_for(condition.as_bytes()) {
            Some(trap) if trap.call => {
                self.pending_additional = additional;
                self.pending_result = result;
                let object = self.build_condition_object(held, Some(true))?;
                self.pending_traps.push_back(PendingTrap {
                    condition: condition.as_bytes().into(),
                    rc: None,
                    description: held.description.clone(),
                    object: Some(object),
                    activation: self.activation().id,
                    queued_during_delivery: false,
                    fragment_depth: self.fragment_depth,
                });
                Ok(result)
            }
            Some(_) => {
                self.pending_additional = additional;
                self.pending_result = result;
                Err(raised)
            }
            None => Ok(result),
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
        code: Option<usize>,
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
            additional: None,
            result: None,
            condition: None,
            code: None,
        });
        frame.owner = owner;
        frame.scope = scope;
        frame.method = receiver.is_some();
        frame.receiver = receiver.unwrap_or(ObjRef::NIL);
        frame.code = code;
        frame.name.extend_from_slice(name);
        frame.arguments.extend_from_slice(args);
        self.native_handles.push(frame);
    }

    /// Pops the innermost native frame, answering the condition it holds and
    /// whether it was a method's, and keeps its cleared buffers for the next
    /// call to refill. The held condition's objects stay rooted as temps.
    fn pop_native_frame(&mut self) -> Popped {
        let mut frame = self
            .native_handles
            .pop()
            .expect("the frame pushed for this call is still the innermost");
        let answer = Popped {
            raised: frame.raised.take(),
            method: frame.method,
            additional: frame.additional.take(),
            result: frame.result.take(),
        };
        frame.condition = None;
        for object in [answer.additional, answer.result].into_iter().flatten() {
            self.roots.push_temp(object);
        }
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
            Refused::UnfilledSlot { entry } => {
                return Loud {
                    message: crate::owned_message(
                        &format!("{refused}"),
                        Some(rexx_api::layout::refusal_owner(entry)),
                    ),
                }
                .into();
            }
            Refused::StaleHandle | Refused::Raised => {
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

/// What [`Interp::pop_native_frame`] answers.
struct Popped {
    raised: Option<Failure>,
    method: bool,
    additional: Option<ObjRef>,
    result: Option<ObjRef>,
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
            // Rendered as `num_rendering` renders it, without the cache, which
            // a shared borrow cannot fill.
            Body::Num {
                value,
                created_digits,
                created_form,
                text,
            } => Some(match text {
                Some(text) => Cow::Borrowed(text.as_slice()),
                None => Cow::Owned(
                    value
                        .format_form(u64::from(*created_digits), *created_form)
                        .into_bytes(),
                ),
            }),
            _ => None,
        }
    }

    fn cself(&mut self) -> Option<POINTER> {
        let frame = self.native_handles.last()?;
        let (receiver, scope) = (frame.receiver, frame.scope);
        // `NativeActivation::cself` (`execution/NativeActivation.cpp:2091`):
        // the receiver's, from the running method's scope upwards.
        rexx_api::callbacks::Surface::object_cself(self, receiver, Some(scope))
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
        let stem = crate::run::shape_of(&name) == crate::run::NameShape::Stem;
        match value {
            // `st. = value`'s rule: a stem is shared, anything else becomes a
            // fresh stem's default.
            Some(value) if stem => {
                let value = self.stem_assignment_value(&name, value);
                self.set_pool_variable(owner, scope, &name, value);
            }
            Some(value) => self.set_pool_variable(owner, scope, &name, value),
            // `VariableDictionary::dropStemVariable` (`:356`): a stem variable
            // always has a value, so a drop leaves a fresh one.
            None if stem => {
                let fresh = self.empty_stem(&name);
                self.set_pool_variable(owner, scope, &name, fresh);
            }
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

    fn resolve(&mut self, handle: rexx_api::layout::RexxObjectPtr) -> Option<ObjRef> {
        self.locals()
            .resolve(handle)
            .or_else(|| self.global_references.resolve(handle))
    }

    fn surface(&mut self) -> Option<&mut dyn rexx_api::callbacks::Surface> {
        Some(self)
    }

    fn kept_c_string(&mut self, object: ObjRef, bytes: &[u8]) -> Option<rexx_api::layout::CSTRING> {
        let kept = self
            .kept_strings
            .entry(object)
            .or_insert_with(|| terminated(bytes));
        Some(kept.as_ptr().cast())
    }

    fn kept_name(&mut self, name: &[u8]) -> Option<rexx_api::layout::CSTRING> {
        let kept = self
            .kept_names
            .entry(name.into())
            .or_insert_with(|| terminated(name));
        Some(kept.as_ptr().cast())
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
    if precision == 0 {
        return no_digit_text(value);
    }
    let printed = percent_g(value, precision.min(16) + 2);
    let digits = u64::try_from(precision).unwrap_or(u64::MAX);
    Number::parse(&printed)
        .unwrap_or_else(|| unreachable!("%g prints a Rexx number: {printed}"))
        .into_round(digits)
        .format(digits)
        .into_bytes()
}

/// `newInstanceFromDouble` at precision 0: `truncateToDigits` keeps no digit
/// and moves only the exponent, `mathRound` adds one to it where the first
/// digit dropped is 5 or more (`classes/NumberStringMath.cpp:315`, `:489`), and
/// `stringValue` renders the digitless number
/// (`classes/NumberStringClass.cpp:338`): the sign alone at exponent 0, else
/// `0` with any exponent past the first place.
fn no_digit_text(value: f64) -> Vec<u8> {
    if value == 0.0 {
        return b"0".to_vec();
    }
    let printed = percent_g(value, 2);
    let (sign, body) = match printed.strip_prefix('-') {
        Some(rest) => ("-", rest),
        None => ("", printed.as_str()),
    };
    let (mantissa, scale) = match body.split_once('e') {
        Some((mantissa, scale)) => (
            mantissa,
            scale
                .parse::<i64>()
                .unwrap_or_else(|_| unreachable!("%g prints a decimal exponent: {printed}")),
        ),
        None => (body, 0),
    };
    let (whole, fraction) = mantissa.split_once('.').unwrap_or((mantissa, ""));
    let digits = format!("{whole}{fraction}");
    let digits = digits.trim_start_matches('0');
    let width = |text: &str| i64::try_from(text.len()).unwrap_or(i64::MAX);
    let mut exponent = scale - width(fraction) + width(digits);
    if digits
        .as_bytes()
        .first()
        .is_some_and(|first| *first >= b'5')
    {
        exponent += 1;
    }
    if exponent == 0 {
        return sign.as_bytes().to_vec();
    }
    let shown = exponent - 1;
    let mut out = format!("{sign}0");
    if shown != 0 {
        let direction = if shown > 0 { '+' } else { '-' };
        out.push_str(&format!("E{direction}{}", shown.unsigned_abs()));
    }
    out.into_bytes()
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

mod surface;
#[cfg(test)]
mod tests;

/// `bytes` with a NUL after them.
fn terminated(bytes: &[u8]) -> Box<[u8]> {
    let mut owned = Vec::with_capacity(bytes.len() + 1);
    owned.extend_from_slice(bytes);
    owned.push(0);
    owned.into_boxed_slice()
}
