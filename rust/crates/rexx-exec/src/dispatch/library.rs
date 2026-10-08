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

use rexx_api::handles::Table;
use rexx_api::invoke::{self, Completion, NativeCall};
use rexx_api::layout::POINTER;
use rexx_api::load::{CommandHandler, HeldMethod, HeldRoutine, Hook, Library};
use rexx_api::redirect::Redirector;
use rexx_api::values::{
    Activation, CStringPool, Class, Constants, Conversion, Failure as Refused, Host, HostRef,
    Numeric, Raised as Condition,
};
use rexx_core::{BehaviourId, Body, Decoded, ObjRef};
use rexx_num::{DIGITS64, Number};

use super::Resolution;
use crate::builtin::datatype::{SymbolKind, classify};
use crate::error::Raised;
use crate::island::{InterpBaton, Islanded, Lent};
use crate::run::Started;
use crate::scheduler::{ActivityId, Posted, Recall, Recaller, Scheduler};
use crate::sync::Arc;
use crate::timer::Inbox;
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
    /// and answers what the extension returned, or `Entered` with the
    /// activity parked for the call's driver exit
    /// ([`Interp::leaves_driver`]).
    ///
    /// # Errors
    /// The condition the extension raised, whatever converting an argument or
    /// the result refuses, and [`Loud`] for a conversion this phase has not
    /// written.
    /// `reserved` says the send holds the method's guard lock, which the
    /// call's frame then holds and releases where it still does.
    pub(super) fn run_library_method(
        &mut self,
        binding: &LibraryBinding,
        resolution: Resolution,
        receiver: ObjRef,
        name: &[u8],
        args: &[Option<ObjRef>],
        reserved: bool,
    ) -> Result<Started<Option<ObjRef>>, Failure> {
        let owner = self.pool_owner(receiver)?;
        let Some(entry) = binding
            .method
            .and_then(|method| binding.library.method_at(method))
        else {
            return Err(Loud::library_procedure_gone().into());
        };
        let held = HeldCall::Method(entry.held(), resolution.method);
        let exits = self.leaves_driver();
        self.push_native_frame(owner, resolution.scope, Some(receiver), name, args, None);
        pin_enter!(self, crate::pinning::PinKind::NativeApiCallback);
        let packaged = self.external_package_path(resolution.method).is_some();
        self.native_frame_mut().packaged = packaged;
        self.native_frame_mut().reserved = reserved;
        let Some((answered, pending)) =
            self.native_call(held, args, exits, receiver, resume_library_method)
        else {
            return Ok(Started::Entered);
        };
        self.end_library_method(answered, pending, receiver, resolution.scope, packaged)
            .map(Started::Ran)
    }

    /// What a library method answers once its stub has returned `answered`,
    /// with the call's frame still pushed and pinned: its frame popped, its
    /// guard lock released where the frame still held it, and then its
    /// condition or its value.
    #[inline(always)]
    fn end_library_method(
        &mut self,
        answered: Result<Option<ObjRef>, Refused>,
        pending: Option<usize>,
        receiver: ObjRef,
        scope: ObjRef,
        packaged: bool,
    ) -> Result<Option<ObjRef>, Failure> {
        let trapped = self.call_trapped_native_condition();
        let popped = self.pop_native_frame();
        pin_leave!(self);
        if popped.reserved {
            self.release_guard(crate::guards::GuardKey {
                object: receiver,
                scope,
            });
        }
        let trapped = trapped?;

        // The condition first, because the oracle raises it in the caller's
        // frame once the call has returned (`NativeActivation::checkConditions`,
        // `:1787`) and before it does anything with the value the extension
        // wrote. Measured, oracle: `.MyRe~new('[')` over `RegExp_Init` is
        // `Error 38 ... Invalid template or pattern.` at rc 218, and the
        // extension returned zero on that path.
        if let Some(number) = pending {
            self.activity.native_reraise = true;
            return Err(condition_of(number));
        }
        self.settle_native_call(answered, popped, packaged, trapped)
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
    ) -> Result<Started<Option<ObjRef>>, Failure> {
        let code = self.package_routine_code(slot);
        self.run_library_routine(code, &name.to_ascii_uppercase(), args)
    }

    /// Runs the library routine [`Interp::library_codes`] row `code` is
    /// against `args`, and answers what the extension returned, or `Entered`
    /// as [`Interp::run_library_method`] does.
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
    ) -> Result<Started<Option<ObjRef>>, Failure> {
        let Some(entry) = self
            .library_code_routine(code)
            .and_then(|(library, index)| library.routine_at(index))
        else {
            return Err(Loud::library_procedure_gone().into());
        };
        let held = HeldCall::Routine(entry.held());
        let exits = self.leaves_driver();
        self.push_native_frame(ObjRef::NIL, ObjRef::NIL, None, name, args, Some(code));
        pin_enter!(self, crate::pinning::PinKind::NativeApiCallback);
        let program = self.library_code_program(code);
        self.native_frame_mut().packaged = program.is_some();
        let Some((answered, pending)) =
            self.native_call(held, args, exits, ObjRef::NIL, resume_library_routine)
        else {
            return Ok(Started::Entered);
        };
        self.end_library_routine(answered, pending, name, program, args)
            .map(Started::Ran)
    }

    /// What a library routine answers once its stub has returned `answered`,
    /// as [`Interp::end_library_method`] does for a method, with the routine
    /// blamed for a failure.
    #[inline(always)]
    fn end_library_routine(
        &mut self,
        answered: Result<Option<ObjRef>, Refused>,
        pending: Option<usize>,
        name: &[u8],
        program: Option<crate::ProgramId>,
        args: &[Option<ObjRef>],
    ) -> Result<Option<ObjRef>, Failure> {
        let trapped = self.call_trapped_native_condition();
        let popped = self.pop_native_frame();
        pin_leave!(self);
        let outcome = match (pending, trapped) {
            (Some(number), _) => {
                self.activity.native_reraise = true;
                Err(condition_of(number))
            }
            (None, Err(failure)) => Err(failure),
            (None, Ok(trapped)) => {
                self.settle_native_call(answered, popped, program.is_some(), trapped)
            }
        };
        if outcome.is_err() {
            self.blame_native_routine(name, program, args);
        }
        outcome
    }

    /// The native call `held` against `args`, its frame pushed and pinned:
    /// its answer and the condition it holds, run back to back on the baton;
    /// or, where it `exits`, `None` with the call prepared on the activity's
    /// record and the activity parked, its continuation `resume` on
    /// `receiver`, and the frame unpinned.
    #[inline(always)]
    fn native_call(
        &mut self,
        held: HeldCall,
        args: &[Option<ObjRef>],
        exits: bool,
        receiver: ObjRef,
        resume: super::NativeResume,
    ) -> Option<(Result<Option<ObjRef>, Refused>, Option<usize>)> {
        let mut strings = CStringPool::new();
        let thread = self.thread_context();
        let frame = self.native_token();
        if !exits {
            let stress = self.stress_collect;
            let activation = Activation::new(Conversion {
                host: self.into(),
                strings: &mut strings,
            });
            activation.set_frame(frame);
            activation.set_between_halves(stress);
            let answered = thread.enter(&activation, |contexts| match &held {
                HeldCall::Method(held, _) => {
                    invoke::held_method(held, &contexts.method(), &activation, args)
                }
                HeldCall::Routine(held) => {
                    invoke::held_routine(held, &contexts.call(), &activation, args)
                }
            });
            return Some((answered, activation.pending()));
        }
        let mut native = NativeCall::empty();
        let (prepared, pending) = {
            let activation = Activation::new(Conversion {
                host: self.into(),
                strings: &mut strings,
            });
            activation.set_frame(frame);
            let prepared = thread.enter(&activation, |contexts| {
                let signature = match &held {
                    HeldCall::Method(held, _) => invoke::held_signature(held, &contexts.method()),
                    HeldCall::Routine(held) => {
                        invoke::held_routine_signature(held, &contexts.call())
                    }
                };
                signature.and_then(|signature| {
                    invoke::prepare(&mut native, &signature, &activation, args)
                })
            });
            (prepared, activation.pending())
        };
        if let Err(refused) = prepared {
            return Some((Err(refused), pending));
        }
        pin_leave!(self);
        self.activity.native_call = Some(Box::new(NativeInFlight {
            call: Some(Box::new(OffBaton {
                held,
                native,
                strings,
            })),
            frame,
            pending,
            stage: Stage::Prepared,
            park: None,
        }));
        self.activity.native_park = Some(Box::new(super::NativePark {
            reason: crate::scheduler::ParkReason::Native,
            resume,
            receiver,
            thens: Vec::new(),
        }));
        None
    }

    /// Whether a native call made now leaves its driver (spec 2026-09-29
    /// P6-3): from a resumable entry nothing pins, and where another activity
    /// is alive or a test mode asks for every exit. A lone activity's call
    /// runs on the baton, which costs a single-activity program nothing
    /// (ruling P43).
    pub(crate) fn leaves_driver(&self) -> bool {
        self.activity.pin_depth == 0
            && !self.activity.resuming
            && (self.switch.is_some() || self.stress_collect || self.others_live())
    }

    /// The running activity's native call, its completion drained: its
    /// record, with the call's frame pinned again and, under the collect
    /// stress mode, a collection before `finish` reads it.
    fn resume_native_call(&mut self) -> Result<(Box<NativeInFlight>, Completion), Failure> {
        let mut call =
            self.activity.native_call.take().ok_or_else(|| {
                Loud::scheduler_inconsistency("a native call resumed with no record")
            })?;
        let Stage::Completed(completion) = std::mem::replace(&mut call.stage, Stage::Prepared)
        else {
            return Err(
                Loud::scheduler_inconsistency("a native call resumed before it completed").into(),
            );
        };
        pin_enter!(self, crate::pinning::PinKind::NativeApiCallback);
        if self.stress_collect {
            self.collect_now();
        }
        Ok((call, completion))
    }

    /// `finish` for `call` as the activity that made it, and the condition
    /// it then holds.
    fn finish_native_call(
        &mut self,
        call: &NativeInFlight,
        completion: Completion,
    ) -> (Result<Option<ObjRef>, Refused>, Option<usize>) {
        let mut strings = CStringPool::new();
        let activation = Activation::new(Conversion {
            host: self.into(),
            strings: &mut strings,
        });
        activation.set_frame(call.frame);
        if let Some(number) = call.pending {
            activation.raise(number);
        }
        let answered = invoke::finish(&activation, completion);
        (answered, activation.pending())
    }

    /// Drops the running activity's native call whose wait was ended by
    /// `failure` in place of its completion: its frame popped and its guard
    /// lock released. The native may still be running, so the frame keeps
    /// what it holds for the native -- its handles' objects, kept strings and
    /// lent buffers -- until the completion is drained
    /// ([`Interp::end_abandoned_call`]).
    pub(crate) fn abandon_native_call(&mut self, receiver: ObjRef) {
        if self.activity.native_call.take().is_none() {
            return;
        }
        let scope = self.native_frame().scope;
        let token = self.native_token();
        let (popped, frame) = self.take_native_frame();
        self.activities.keep_abandoned(token, frame);
        if popped.reserved {
            self.release_guard(crate::guards::GuardKey {
                object: receiver,
                scope,
            });
        }
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
        pin_enter!(self, crate::pinning::PinKind::LibraryEntry);
        let mut strings = CStringPool::new();
        let thread = self.thread_context();
        let (ran, pending) = {
            let frame = self.native_token();
            let activation = Activation::new(Conversion {
                host: self.into(),
                strings: &mut strings,
            });
            activation.set_frame(frame);
            let ran = thread.enter(&activation, |contexts| {
                invoke::hook(library, hook, contexts, &activation)
            });
            (ran, activation.pending())
        };
        let popped = self.pop_native_frame();
        pin_leave!(self);
        if let Some(number) = pending {
            return Err(condition_of(number));
        }
        let settled = self.settle_native_call(ran.map(|()| None), popped, true, None);
        // A hook is no level of the failure's, so nothing re-raises it.
        self.activity.native_reraise = false;
        settled.map(|_| ())
    }

    /// Runs the registered command handler `handler` for `command`, issued to
    /// `environment`, in a native frame of its own, as `Activity::run` runs a
    /// `ContextCommandHandlerDispatcher` (`concurrency/CommandHandler.cpp:231-241`).
    ///
    /// # Errors
    /// A condition a string conversion raised, and [`Loud`] for an interface
    /// member the handler reached that this phase has not written. A
    /// condition the handler raised is answered, not returned as an error.
    pub(crate) fn run_command_handler(
        &mut self,
        handler: &CommandHandler,
        environment: &[u8],
        command: &[u8],
        redirector: &Redirector,
    ) -> Result<HandledCommand, Failure> {
        let address = self.text(environment);
        self.roots.activity_mut().push_temp(address);
        let issued = self.text(command);
        self.roots.activity_mut().push_temp(issued);
        self.push_native_frame(ObjRef::NIL, ObjRef::NIL, None, b"", &[], None);
        pin_enter!(self, crate::pinning::PinKind::NativeApiCallback);
        let mut strings = CStringPool::new();
        let thread = self.thread_context();
        let (answered, pending) = {
            let frame = self.native_token();
            let activation = Activation::new(Conversion {
                host: self.into(),
                strings: &mut strings,
            });
            activation.set_frame(frame);
            let answered = thread.enter(&activation, |contexts| {
                invoke::command(handler, contexts, &activation, address, issued, redirector)
            });
            (answered, activation.pending())
        };
        let popped = self.pop_native_frame();
        pin_leave!(self);
        if let Some(number) = pending {
            return Err(condition_of(number));
        }
        let value = match answered {
            Ok(value) => value,
            Err(Refused::Raised) => {
                return Err(popped
                    .raised
                    .expect("a host answering Raised holds the condition it raised"));
            }
            Err(refused) => return Err(self.refusal(refused, true, false)),
        };
        if let Some(value) = value {
            self.roots.activity_mut().push_temp(value);
        }
        Ok(HandledCommand {
            value,
            raised: popped.raised,
            additional: popped.additional,
            result: popped.result,
        })
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
                    // A registered handler's code may be in this library, and
                    // nothing ties the two together.
                    if library.close() {
                        self.command_handlers.clear();
                    }
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
    ///
    /// `trapped` is the object [`Interp::call_trapped_native_condition`]
    /// built before the frame came off.
    fn settle_native_call(
        &mut self,
        answered: Result<Option<ObjRef>, Refused>,
        popped: Popped,
        packaged: bool,
        trapped: Option<ObjRef>,
    ) -> Result<Option<ObjRef>, Failure> {
        match answered {
            Err(Refused::Raised) => Err(popped
                .raised
                .expect("a host answering Raised holds the condition it raised")),
            Ok(value) => match popped.raised {
                None => Ok(value),
                Some(raised) => {
                    self.raise_held_condition(raised, popped.additional, popped.result, trapped)
                }
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
        trapped: Option<ObjRef>,
    ) -> Result<Option<ObjRef>, Failure> {
        let condition = match &raised {
            Failure::Raised(held) if held.condition != "SYNTAX" => held.condition.to_string(),
            _ => {
                if additional.is_some() {
                    self.activity.pending_additional = additional;
                }
                self.activity.native_reraise = true;
                return Err(raised);
            }
        };
        let Failure::Raised(held) = &raised else {
            unreachable!("matched as a raise above")
        };
        match self.trap_for(condition.as_bytes()) {
            Some(trap) if trap.call => {
                let object = match trapped {
                    Some(object) => object,
                    None => {
                        self.activity.pending_additional = additional;
                        self.activity.pending_result = result;
                        self.build_condition_object(held, Some(true))?
                    }
                };
                self.activity.pending_traps.push_back(PendingTrap {
                    condition: condition.as_bytes().into(),
                    rc: None,
                    description: held.description.clone(),
                    object: Some(object),
                    activation: self.activation().id,
                    queued_during_delivery: false,
                    request: false,
                    fragment_depth: self.activity.fragment_depth,
                });
                Ok(result)
            }
            Some(_) => {
                self.activity.pending_additional = additional;
                self.activity.pending_result = result;
                Err(raised)
            }
            None => Ok(result),
        }
    }

    /// The object a `CALL ON` trap takes for the condition other than
    /// `SYNTAX` that the innermost native call holds, built while that call's
    /// frame still leads the stack, as `Activity::createConditionObject`
    /// builds it at the raise (`interpreter/concurrency/Activity.cpp:722-749`).
    fn call_trapped_native_condition(&mut self) -> Result<Option<ObjRef>, Failure> {
        let Some(Failure::Raised(held)) = &self.native_frame().raised else {
            return Ok(None);
        };
        if held.condition == "SYNTAX"
            || !self
                .trap_for(held.condition.as_bytes())
                .is_some_and(|trap| trap.call)
        {
            return Ok(None);
        }
        let held = held.clone();
        let frame = self.native_frame();
        (
            self.activity.pending_additional,
            self.activity.pending_result,
        ) = (frame.additional, frame.result);
        let object = self.build_trapped_native_condition_object(&held)?;
        self.roots.activity_mut().push_temp(object);
        Ok(Some(object))
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
        let mut frame = self
            .activity
            .native_spares
            .pop()
            .unwrap_or_else(|| NativeFrame {
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
                kept: rustc_hash::FxHashSet::default(),
                lent: Vec::new(),
                caller: None,
                id: 0,
                packaged: false,
                reserved: false,
            });
        frame.owner = owner;
        frame.scope = scope;
        frame.method = receiver.is_some();
        frame.receiver = receiver.unwrap_or(ObjRef::NIL);
        frame.code = code;
        frame.caller = self.running_activation().map(|activation| activation.id);
        frame.id = self.next_native_id();
        frame.packaged = false;
        frame.reserved = false;
        frame.name.extend_from_slice(name);
        frame.arguments.extend_from_slice(args);
        self.activity.native_handles.push(frame);
    }

    /// Pops the innermost native frame, answering the condition it holds and
    /// whether it was a method's, and keeps its cleared buffers for the next
    /// call to refill. The held condition's objects stay rooted as temps.
    fn pop_native_frame(&mut self) -> Popped {
        let (answer, mut frame) = self.take_native_frame();
        self.end_native_frame(&mut frame);
        self.activity.native_spares.push(frame);
        answer
    }

    /// Releases what the native frame of an abandoned call, named by the
    /// frame token `token`, holds, once that call's completion is drained.
    pub(crate) fn end_abandoned_call(&mut self, token: u64) {
        if let Some(mut frame) = self.activities.take_abandoned(token) {
            self.end_native_frame(&mut frame);
        }
    }

    /// Pops the innermost native frame and takes its held condition out of
    /// it, leaving the rest held.
    fn take_native_frame(&mut self) -> (Popped, crate::NativeFrame) {
        let mut frame = self
            .activity
            .native_handles
            .pop()
            .expect("the frame pushed for this call is still the innermost");
        let answer = Popped {
            raised: frame.raised.take(),
            method: frame.method,
            reserved: std::mem::take(&mut frame.reserved),
            additional: frame.additional.take(),
            result: frame.result.take(),
        };
        frame.condition = None;
        for object in [answer.additional, answer.result].into_iter().flatten() {
            self.roots.activity_mut().push_temp(object);
        }
        (answer, frame)
    }

    /// Clears `frame` for reuse, ending its hold on its kept strings and lent
    /// buffers.
    fn end_native_frame(&mut self, frame: &mut crate::NativeFrame) {
        frame.name.clear();
        frame.arguments.clear();
        frame.argument_list = None;
        frame.locals.clear();
        for object in frame.kept.drain() {
            self.release_kept(object);
        }
        for buffer in frame.lent.drain(..) {
            if let Some(state) = self.buffer_mut(buffer) {
                state.bytes.release();
            }
        }
    }

    /// Drops the kept copy of every handle-carried value no call in flight
    /// and no global reference holds, as a collection does. These copies
    /// grow without the heap growing, so a run of calls that allocates
    /// nothing would otherwise never meet a collection; this runs whenever
    /// their number passes twice what the last pass left, and 4096.
    pub(crate) fn drop_loose_kept_strings(&mut self) {
        let (holders, globals) = (&self.kept_holders, &self.global_references);
        self.kept_strings.retain(|object, _| {
            matches!(object.decode(), rexx_core::Decoded::Heap { .. })
                || holders.contains_key(object)
                || globals.holds(*object)
        });
        self.kept_carried = self
            .kept_strings
            .keys()
            .filter(|object| !matches!(object.decode(), rexx_core::Decoded::Heap { .. }))
            .count();
        self.kept_carried_limit = (self.kept_carried * 2).max(4096);
    }

    /// One native call's end for a handle-carried value's kept copy, which
    /// the next collection may then drop.
    fn release_kept(&mut self, object: ObjRef) {
        let std::collections::hash_map::Entry::Occupied(mut holders) =
            self.kept_holders.entry(object)
        else {
            return;
        };
        *holders.get_mut() -= 1;
        if *holders.get() == 0 {
            holders.remove();
        }
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
                        rexx_api::layout::refusal_owner(entry),
                    ),
                }
                .into();
            }
            // No phase owes a stale handle: a local reference protects its
            // object only until the call that made it ends
            // (`NativeActivation::createLocalReference`,
            // `interpreter/execution/NativeActivation.cpp:1179-1185`), so an
            // extension using one past that call is outside the API.
            Refused::StaleHandle | Refused::Raised => {
                return Loud {
                    message: crate::owned_message(&format!("{refused}"), None),
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

/// What a registered command handler left once its frame is off the stack:
/// the object it returned and the condition it raised, each rooted as a temp.
pub(crate) struct HandledCommand {
    pub(crate) value: Option<ObjRef>,
    pub(crate) raised: Option<Failure>,
    pub(crate) additional: Option<ObjRef>,
    pub(crate) result: Option<ObjRef>,
}

/// What [`Interp::pop_native_frame`] answers.
struct Popped {
    raised: Option<Failure>,
    method: bool,
    /// Whether the frame still held its method's guard lock.
    reserved: bool,
    additional: Option<ObjRef>,
    result: Option<ObjRef>,
}

/// A native call that leaves its driver, on its activity's record from
/// `prepare` until `finish`.
pub(crate) struct NativeInFlight {
    /// What the call reads and writes, taken by the thread that runs it.
    call: Option<Box<OffBaton>>,
    /// The call's native frame, as [`Interp::native_token`] names it.
    frame: u64,
    /// The condition a host without a surface recorded during the call.
    pending: Option<usize>,
    stage: Stage,
    /// The activity's park while the call runs, so that a park recorded by
    /// Rexx code a callback runs is told apart from it.
    park: Option<Box<super::NativePark>>,
}

/// The part of a native call its run off the baton takes.
pub(crate) struct OffBaton {
    held: HeldCall,
    native: NativeCall,
    /// The call's C strings, which its descriptors point into.
    strings: CStringPool,
}

/// The row a [`NativeInFlight`] calls.
enum HeldCall {
    /// A method's row, with the resolved method a failure blames.
    Method(HeldMethod, super::MethodId),
    Routine(HeldRoutine),
}

/// Where a [`NativeInFlight`] has got to.
enum Stage {
    /// Prepared, its driver not yet exited.
    Prepared,
    /// Left its driver; its completion not yet drained.
    Left,
    Completed(Completion),
}

impl NativeInFlight {
    /// Whether the call waits for its driver exit.
    pub(crate) fn awaits_exit(&self) -> bool {
        matches!(self.stage, Stage::Prepared)
    }

    /// The call's native frame, as [`Interp::native_token`] names it.
    pub(crate) fn frame(&self) -> u64 {
        self.frame
    }

    /// Readies the call to leave its driver, keeping `park`, the
    /// activity's, for the call's length, and answers what the call reads
    /// and writes, its frame, and the condition recorded so far.
    pub(crate) fn leave(
        &mut self,
        park: Option<Box<super::NativePark>>,
    ) -> (Box<OffBaton>, u64, Option<usize>) {
        self.park = park;
        self.stage = Stage::Left;
        let call = self
            .call
            .take()
            .expect("a prepared call holds what it runs");
        (call, self.frame, self.pending)
    }

    /// Takes back what [`NativeInFlight::leave`] answered once the call has
    /// run, with the condition a host without a surface recorded during it,
    /// and answers the activity's park.
    pub(crate) fn back(
        &mut self,
        call: Box<OffBaton>,
        pending: Option<usize>,
    ) -> Option<Box<super::NativePark>> {
        self.call = Some(call);
        self.pending = pending;
        self.park.take()
    }

    /// The activity's park, kept here while the call runs on a pool thread,
    /// for a failure that ends it first.
    pub(crate) fn take_park(&mut self) -> Option<Box<super::NativePark>> {
        self.park.take()
    }

    pub(crate) fn object_roots(&self, out: &mut Vec<ObjRef>) {
        if let Some(park) = &self.park {
            park.object_roots(out);
        }
    }

    /// Records the call's drained completion.
    pub(crate) fn complete(&mut self, completion: Completion) {
        self.stage = Stage::Completed(completion);
    }
}

/// A library method's continuation once its call has completed: `finish`,
/// then its end and its blame, as [`Interp::run_other`] blames.
fn resume_library_method(interp: &mut Interp, receiver: ObjRef) -> Result<Option<ObjRef>, Failure> {
    let (mut call, completion) = interp.resume_native_call()?;
    let frame = interp.native_frame();
    let (scope, name, args, packaged) = (
        frame.scope,
        frame.name.clone(),
        frame.arguments.clone(),
        frame.packaged,
    );
    let (answered, pending) = interp.finish_native_call(&call, completion);
    let Some(HeldCall::Method(_, method)) = call.call.take().map(|call| call.held) else {
        return Err(Loud::scheduler_inconsistency("a method resumed from a routine's call").into());
    };
    drop(call);
    let outcome = interp.end_library_method(answered, pending, receiver, scope, packaged);
    if outcome.is_err() {
        let scope_name = interp.scope_id(scope);
        let reraised = std::mem::take(&mut interp.activity.native_reraise);
        interp.blame_external_method(&name, &scope_name, method, receiver, &args, reraised);
    }
    outcome
}

/// A library routine's continuation once its call has completed, as
/// [`resume_library_method`] is a method's.
fn resume_library_routine(
    interp: &mut Interp,
    _receiver: ObjRef,
) -> Result<Option<ObjRef>, Failure> {
    let (call, completion) = interp.resume_native_call()?;
    let frame = interp.native_frame();
    let (name, args, code) = (frame.name.clone(), frame.arguments.clone(), frame.code);
    let program = code.and_then(|code| interp.library_code_program(code));
    let (answered, pending) = interp.finish_native_call(&call, completion);
    drop(call);
    interp.end_library_routine(answered, pending, &name, program, &args)
}

impl OffBaton {
    /// Runs the call on the thread holding `baton`, which keeps it: a thread
    /// may hold it by a lend, which only a give-back ends. `host` is reached
    /// through a [`HostRef::guarded`], as the native frame `frame` holding the
    /// condition `pending`; the completion goes to `post`. Answers the
    /// condition then held.
    pub(crate) fn run(
        &mut self,
        frame: u64,
        pending: Option<usize>,
        host: &mut Interp,
        baton: &InterpBaton,
        thread: &rexx_api::ffi::ThreadContext,
        post: impl FnOnce(Completion),
    ) -> Option<usize> {
        let OffBaton {
            held,
            native,
            strings,
        } = self;
        let activation = Activation::new(Conversion {
            host: HostRef::guarded(host, baton),
            strings,
        });
        activation.set_frame(frame);
        if let Some(number) = pending {
            activation.raise(number);
        }
        thread.enter(&activation, |contexts| post(held.call(native, contexts)));
        activation.pending()
    }
}

impl HeldCall {
    /// Calls the row's stub with `native`'s descriptors.
    fn call(
        &self,
        native: &mut NativeCall,
        contexts: &mut rexx_api::ffi::Contexts<'_, '_>,
    ) -> Completion {
        match self {
            HeldCall::Method(held, _) => invoke::call_held_method(native, held, &contexts.method()),
            HeldCall::Routine(held) => invoke::call_held_routine(native, held, &contexts.call()),
        }
    }
}

/// A native call a pool thread runs (spec 2026-09-29 2.2): the baton's holder
/// lends it the baton to enter the call, and it recalls the baton for each
/// callback and for the call's end, so that each runs as the call's activity.
pub(crate) struct PooledCall {
    work: Islanded<(Box<OffBaton>, rexx_api::ffi::ThreadContext)>,
    frame: u64,
    pending: Option<usize>,
    activity: ActivityId,
    baton: Arc<InterpBaton>,
    inbox: Arc<Inbox<Posted>>,
}

impl PooledCall {
    /// The call `call` in the native frame `frame` holding the condition
    /// `pending`, with `thread` as `activity`'s thread context.
    pub(crate) fn new(
        call: Box<OffBaton>,
        thread: rexx_api::ffi::ThreadContext,
        frame: u64,
        pending: Option<usize>,
        activity: ActivityId,
        baton: &Arc<InterpBaton>,
        inbox: Arc<Inbox<Posted>>,
    ) -> PooledCall {
        PooledCall {
            work: Islanded::new((call, thread), baton),
            frame,
            pending,
            activity,
            baton: Arc::clone(baton),
            inbox,
        }
    }

    /// On the pool thread: enters the call under the lend the holder makes,
    /// gives the baton back, calls the stub, and once it returns recalls the
    /// baton to leave the call, put its parts back on the activity's record
    /// and post its completion. A stub that panics recalls it to unwind.
    pub(crate) fn run(self) {
        let probe = 0u8;
        let recalling = Recalling {
            baton: &self.baton,
            inbox: &self.inbox,
            activity: self.activity,
            frame: self.frame,
            base: &raw const probe as usize,
        };
        let entered = Lent::wait(&self.baton);
        let (mut call, thread) = self.work.take(&self.baton);
        let (completion, mut ended, pending) = {
            let OffBaton {
                held,
                native,
                strings,
            } = &mut *call;
            let activation = Activation::new(Conversion {
                host: HostRef::through(&recalling),
                strings,
            });
            activation.set_frame(self.frame);
            if let Some(number) = self.pending {
                activation.raise(number);
            }
            let (completion, ended) = thread.enter(&activation, |contexts| {
                drop(entered);
                let completion = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    held.call(native, contexts)
                }));
                let ended = recalling.recall();
                match completion {
                    Ok(completion) => (completion, ended),
                    // Unwound under the lend: the call's state drops on the
                    // baton.
                    Err(payload) => std::panic::resume_unwind(payload),
                }
            });
            (completion, ended, activation.pending())
        };
        let interp = ended.interp();
        #[cfg(test)]
        if let Some(path) = &interp.panic_at_call_end {
            use std::io::Write;
            let mut file = std::fs::OpenOptions::new()
                .append(true)
                .create(true)
                .open(path)
                .expect("the test's file");
            writeln!(file, "end").expect("a line written");
            panic!("a call's end panicked under a lend");
        }
        if let Some(record) = interp
            .activity
            .native_call
            .as_mut()
            .filter(|record| record.frame == self.frame)
        {
            interp.activity.native_park = record.back(call, pending);
        }
        drop(thread);
        Interp::post_completion(&self.inbox, self.activity, self.frame, completion);
    }
}

/// The baton as a pool thread running a native call reaches it: a thread
/// that does not hold it recalls it from the holder, which lends it with the
/// call's activity running.
struct Recalling<'p> {
    baton: &'p InterpBaton,
    inbox: &'p Inbox<Posted>,
    activity: ActivityId,
    frame: u64,
    base: usize,
}

impl<'p> Recalling<'p> {
    /// Recalls the baton and waits until it is lent.
    fn recall(&self) -> Lent<'p> {
        self.inbox.post(Posted::Recall(Recall {
            by: Recaller::Call {
                activity: self.activity,
                frame: self.frame,
            },
            thread: std::thread::current().id(),
            base: Some(self.base),
        }));
        Lent::wait(self.baton)
    }
}

/// The baton as a thread running no native call of the interpreter's
/// reaches it (spec 2026-09-29 2.4): it recalls the baton as a pool thread
/// does, and the holder serves it at its next drain.
pub(crate) struct Requester {
    baton: Arc<InterpBaton>,
    inbox: Arc<Inbox<Posted>>,
}

impl Requester {
    pub(crate) fn new(baton: &Arc<InterpBaton>, inbox: Arc<Inbox<Posted>>) -> Requester {
        Requester {
            baton: Arc::clone(baton),
            inbox,
        }
    }
}

impl rexx_api::values::Baton for Requester {
    fn take_unless_held(&self) -> bool {
        self.take_for(0)
    }

    fn take_for(&self, context: usize) -> bool {
        if self.baton.held_here() {
            return false;
        }
        #[cfg(test)]
        self.baton.count_take();
        self.inbox.post(Posted::Recall(Recall {
            by: Recaller::Context(context),
            thread: std::thread::current().id(),
            base: None,
        }));
        std::mem::forget(Lent::wait(&self.baton));
        true
    }

    fn release(&self) {
        self.baton.give_back();
    }

    fn held_here(&self) -> bool {
        self.baton.held_here()
    }

    fn host(&self) -> Option<std::ptr::NonNull<dyn rexx_api::values::Host>> {
        self.baton.lent().map(crate::island::Island::host)
    }
}

impl rexx_api::values::Baton for Recalling<'_> {
    fn take_unless_held(&self) -> bool {
        if self.baton.held_here() {
            return false;
        }
        #[cfg(test)]
        self.baton.count_take();
        std::mem::forget(self.recall());
        true
    }

    fn release(&self) {
        self.baton.give_back();
    }

    fn held_here(&self) -> bool {
        self.baton.held_here()
    }

    fn host(&self) -> Option<std::ptr::NonNull<dyn rexx_api::values::Host>> {
        self.baton.lent().map(crate::island::Island::host)
    }
}

#[cfg(test)]
thread_local! {
    /// How many times a callback took the baton of an interpreter this
    /// thread ran.
    static CALLBACK_TAKES: std::cell::Cell<u64> = const { std::cell::Cell::new(0) };
}

/// [`CALLBACK_TAKES`]'s count.
#[cfg(test)]
pub(crate) fn callback_takes() -> u64 {
    CALLBACK_TAKES.with(std::cell::Cell::get)
}

/// Adds `takes` to [`CALLBACK_TAKES`].
#[cfg(test)]
pub(crate) fn note_callback_takes(takes: u64) {
    CALLBACK_TAKES.with(|counted| counted.set(counted.get() + takes));
}

impl rexx_api::values::Baton for InterpBaton {
    fn take_unless_held(&self) -> bool {
        let took = crate::baton::Baton::take_unless_held(self);
        #[cfg(test)]
        if took {
            self.count_take();
        }
        took
    }

    fn release(&self) {
        crate::baton::Baton::release(self);
    }

    fn held_here(&self) -> bool {
        crate::baton::Baton::held_here(self)
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
        self.activity
            .native_handles
            .last()
            .is_some_and(|frame| frame.method)
    }

    fn string_value(&mut self, object: ObjRef) -> Result<Option<ObjRef>, Condition> {
        match self.blamed_string_conversion(object) {
            Ok(converted) => {
                if let Some(text) = converted {
                    self.roots.activity_mut().push_temp(text);
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
        let frame = self.activity.native_handles.last()?;
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
        let Some(frame) = self.activity.native_handles.last() else {
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
        self.roots.activity_mut().push_temp(object);
        object
    }

    fn numeric(&self) -> Numeric {
        numeric_of(&self.activation().settings)
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
            match self.request_array_value(object) {
                Ok(converted) => converted,
                Err(failure) => return Err(self.hold_native_condition(failure)),
            }
        };
        let Some(array) = converted else {
            return Ok(None);
        };
        self.roots.activity_mut().push_temp(array);
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
        self.activity
            .native_handles
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
        self.roots.activity_mut().push_temp(object);
        object
    }

    fn new_string(&mut self, bytes: &[u8]) -> ObjRef {
        let object = self.text(bytes);
        self.roots.activity_mut().push_temp(object);
        object
    }

    fn double_object(&mut self, value: f64, precision: usize) -> ObjRef {
        let object = self.text(&double_text(value, precision));
        self.roots.activity_mut().push_temp(object);
        object
    }

    fn locals(&mut self) -> &mut Table {
        &mut self
            .activity
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
        if !matches!(object.decode(), rexx_core::Decoded::Heap { .. }) {
            let frame = self.activity.native_handles.last_mut()?;
            if frame.kept.insert(object) {
                *self.kept_holders.entry(object).or_default() += 1;
            }
            if !self.kept_strings.contains_key(&object) {
                self.kept_carried += 1;
                if self.kept_carried > self.kept_carried_limit {
                    self.drop_loose_kept_strings();
                }
            }
        }
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

    fn between_halves(&mut self) {
        if self.stress_collect {
            self.collect_now();
        }
    }

    fn runs_frame(&self, frame: u64) -> bool {
        self.runs_native_frame(frame)
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
                self.roots.activity_mut().push_temp(answered);
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
                self.roots.activity_mut().push_temp(text);
                Ok(text)
            }
            Err(failure) => Err(self.hold_native_condition(failure)),
        }
    }

    /// Holds `failure` on the running native frame for the call to raise once
    /// the boundary has answered [`Refused::Raised`].
    fn hold_native_condition(&mut self, failure: Failure) -> Condition {
        let refused = match &failure {
            Failure::Raised(raised) => self.settle_failed_sends(raised).err(),
            _ => None,
        };
        let failure = refused.unwrap_or(failure);
        self.activity
            .native_handles
            .last_mut()
            .expect("a native activation is running")
            .raised = Some(failure);
        Condition
    }

    /// [`Interp::native_frame`], mutably.
    ///
    /// # Panics
    /// As [`Interp::native_frame`].
    pub(crate) fn native_frame_mut(&mut self) -> &mut NativeFrame {
        self.activity
            .native_handles
            .last_mut()
            .expect("a native activation is running")
    }

    /// The running native activation's name for rexx-api: its row in
    /// [`Activity::native_handles`] above its number, so no two live frames
    /// share one.
    ///
    /// # Panics
    /// As [`Interp::native_frame`].
    fn native_token(&self) -> u64 {
        let row = self.activity.native_handles.len() - 1;
        (row as u64) << 32 | u64::from(self.activity.native_handles[row].id)
    }

    /// The running native activation.
    ///
    /// # Panics
    /// If no native activation is running, which is the only time the host is
    /// asked.
    pub(crate) fn native_frame(&self) -> &NativeFrame {
        self.activity
            .native_handles
            .last()
            .expect("a native activation is running")
    }
}

/// The `NUMERIC` settings a call context reports for an activation's
/// `settings`.
pub(super) fn numeric_of(settings: &rexx_num::Settings) -> Numeric {
    Numeric {
        digits: usize::try_from(settings.digits()).unwrap_or(usize::MAX),
        fuzz: usize::try_from(settings.fuzz()).unwrap_or(usize::MAX),
        engineering: settings.form() == rexx_num::Form::Engineering,
    }
}

/// A Rexx number as the C `double` `strtod` reads from its digits
/// (`NumberString::doubleValue`, `interpreter/classes/NumberStringClass.cpp:704`).
///
/// Rendered at as many digits as the number has, which keeps every digit.
/// `Number::format` takes the exponential form where the plain one would
/// pad the integer part with zeros or put more zeros after the point than
/// there are digits, so the text stays within about twice the digits.
pub(super) fn double_of(number: &Number) -> f64 {
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
