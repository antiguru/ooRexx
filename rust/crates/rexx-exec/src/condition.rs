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

//! The condition object a trap holds, whose copy `CONDITION('O')` answers:
//! the Directory `Activity::createConditionObject` builds, with the entries
//! the raised condition's own kind carries.

use rexx_core::{Body, ObjRef};

use crate::Interp;
use crate::dispatch::context::LiveLevel;
use crate::error::{Failure, FailureSite, Raised};
use crate::plan::Package;

/// What the levels a trapped failure left contribute to its condition object:
/// the frames `Activity::generateProgramInformation` takes at the raise, which
/// this crate can only take as each level ends.
pub(crate) struct Unwound<'a> {
    /// Innermost first, as [`Activity::failure_sites`] holds them.
    pub(crate) sites: &'a [FailureSite],
    /// The `StackFrame`s of those levels, innermost first; each is rooted by
    /// the caller.
    pub(crate) frames: Vec<ObjRef>,
    /// The innermost level with a package, and its line if it has one.
    pub(crate) origin: Option<(Package, Option<usize>)>,
    /// `PROPAGATED`.
    pub(crate) propagated: bool,
    /// Whether a level re-raised it, whose line replaces the raise's own.
    pub(crate) reraised: bool,
}

/// An unwinding failure's record on [`crate::activity::Activity`].
struct Unwinding {
    site: Option<FailureSite>,
    sites: Vec<FailureSite>,
    frame: Option<ObjRef>,
    frames: Vec<ObjRef>,
    origin: Option<(Package, Option<usize>)>,
    propagated: bool,
    reraised: bool,
    reraised_object: Option<ObjRef>,
}

/// The indexes a condition's directory is keyed by.
mod key {
    pub(super) const ADDITIONAL: &[u8] = b"ADDITIONAL";
    pub(super) const CODE: &[u8] = b"CODE";
    pub(super) const CONDITION: &[u8] = b"CONDITION";
    pub(super) const DESCRIPTION: &[u8] = b"DESCRIPTION";
    pub(super) const ERRORTEXT: &[u8] = b"ERRORTEXT";
    pub(super) const INSTRUCTION: &[u8] = b"INSTRUCTION";
    pub(super) const MESSAGE: &[u8] = b"MESSAGE";
    pub(super) const PACKAGE: &[u8] = b"PACKAGE";
    pub(super) const POSITION: &[u8] = b"POSITION";
    pub(super) const PROGRAM: &[u8] = b"PROGRAM";
    pub(super) const PROPAGATED: &[u8] = b"PROPAGATED";
    pub(super) const RC: &[u8] = b"RC";
    pub(super) const RESULT: &[u8] = b"RESULT";
    pub(super) const STACKFRAMES: &[u8] = b"STACKFRAMES";
    pub(super) const TRACEBACK: &[u8] = b"TRACEBACK";

    /// The order `Activity::createExceptionObject` puts a `SYNTAX`
    /// condition's entries in, then the trap's `INSTRUCTION`
    /// (`RexxActivation.cpp:2524`); iteration order depends on it.
    pub(super) const SYNTAX_ORDER: &[&[u8]] = &[
        CODE,
        RC,
        ERRORTEXT,
        ADDITIONAL,
        MESSAGE,
        DESCRIPTION,
        RESULT,
        STACKFRAMES,
        TRACEBACK,
        POSITION,
        PROGRAM,
        PACKAGE,
        CONDITION,
        PROPAGATED,
        INSTRUCTION,
    ];

    /// `Activity::createConditionObject`'s order, for every other condition.
    pub(super) const ORDER: &[&[u8]] = &[
        CONDITION,
        DESCRIPTION,
        PROPAGATED,
        RC,
        ADDITIONAL,
        RESULT,
        STACKFRAMES,
        TRACEBACK,
        POSITION,
        PROGRAM,
        PACKAGE,
        INSTRUCTION,
    ];

    /// The order for a condition object that has `CODE`, which only a
    /// `SYNTAX` condition has, or not.
    pub(super) fn order(syntax: bool) -> &'static [&'static [u8]] {
        if syntax { SYNTAX_ORDER } else { ORDER }
    }
}

impl Interp {
    /// The condition object for `raised`, as the handler will read it back.
    ///
    /// `call` is whether a `CALL ON` trap took it, which is what
    /// `INSTRUCTION` names -- **the trapping instruction, not the raising
    /// clause**, measured `CALL` under `CALL ON` and `SIGNAL` under `SIGNAL
    /// ON` for the same failing command. `None` is an object a native call
    /// asked for before any trap took it, which has no `INSTRUCTION`
    /// (measured through `GetConditionInfo`).
    ///
    /// **Which indexes appear depends on the condition**, measured on four
    /// kinds. Those every kind carries are always there, `INSTRUCTION` among
    /// them once a trap took it; `RC` joins them wherever the raise
    /// carries one; `RESULT` on the command path and where an extension's
    /// `RaiseCondition` named one; and a `SYNTAX` condition adds
    /// `ADDITIONAL`, `CODE`, `ERRORTEXT` and `MESSAGE`. A directory that
    /// carried all of them for every kind would answer plausibly and wrongly.
    pub(crate) fn build_condition_object(
        &mut self,
        raised: &Raised,
        call: Option<bool>,
    ) -> Result<ObjRef, Failure> {
        self.build_condition_object_from(raised, call, false, None)
    }

    /// [`Interp::build_condition_object`] for a condition a `SIGNAL ON` trap
    /// took after it left `unwound`'s levels, whose frames lead `STACKFRAMES`
    /// and `TRACEBACK` and whose innermost line is `POSITION`.
    pub(crate) fn build_trapped_condition_object(
        &mut self,
        raised: &Raised,
        unwound: &Unwound<'_>,
    ) -> Result<ObjRef, Failure> {
        self.build_condition_object_from(raised, Some(false), false, Some(unwound))
    }

    /// `Activity::reraiseException` over `object`: `POSITION`, `PROGRAM` and
    /// `PACKAGE` become those of the level it was re-raised in, `origin` when
    /// that level has since been left, and it is `PROPAGATED`.
    pub(crate) fn reraise_condition_object(
        &mut self,
        object: ObjRef,
        origin: Option<(Package, Option<usize>)>,
    ) -> Result<ObjRef, Failure> {
        self.roots.activity_mut().push_temp(object);
        let (package, line) = match origin {
            Some(origin) => origin,
            None => (
                self.running_program()
                    .map_or(Package::Rexx, Package::Program),
                Some(self.activity.clause_state.line()),
            ),
        };
        if let Some(line) = line {
            let position = self.counted(line);
            self.hash_entry_write(object, key::POSITION, position)?;
        }
        let program = match package {
            Package::Program(id) => self.program_display_name(id).to_vec(),
            Package::Rexx => crate::LIBRARY_PACKAGE_NAME.to_vec(),
        };
        let program = self.text_built(program);
        self.hash_entry_write(object, key::PROGRAM, program)?;
        let package = self.package_object(package);
        self.hash_entry_write(object, key::PACKAGE, package)?;
        let propagated = self.text(b"1");
        self.hash_entry_write(object, key::PROPAGATED, propagated)?;
        Ok(object)
    }

    /// Gives every message whose send the unwinding failure ended the
    /// condition object no trap built: the object of the levels it has left,
    /// or the kept object a level raised again.
    pub(crate) fn settle_failed_sends(&mut self, raised: &Raised) -> Result<(), Failure> {
        if self.activity.failed_sends.is_empty() {
            return Ok(());
        }
        let frame = self.roots.activity_mut().push_frame();
        let origin = self.activity.failure_origin;
        let object = match self.activity.reraised_object {
            Some(object) => self.reraise_condition_object(object, origin),
            None => {
                // The levels already left, as a trap takes them: a level still
                // running contributes its live frame instead.
                let sites = self.activity.failure_sites.clone();
                let unwound = Unwound {
                    sites: &sites,
                    frames: self.activity.failure_frames.clone(),
                    origin,
                    propagated: self.activity.failure_propagated,
                    reraised: self.activity.failure_reraised,
                };
                self.build_condition_object_from(raised, None, false, Some(&unwound))
            }
        };
        let attached = object.map(|object| self.attach_condition(object));
        if attached.is_err() {
            self.activity.failed_sends.clear();
        }
        self.roots.activity_mut().pop_frame(frame);
        attached
    }

    /// `MessageClass::error` (`classes/MessageClass.cpp:706`) for every
    /// message whose send the condition behind `object` ended.
    pub(crate) fn attach_condition(&mut self, object: ObjRef) {
        for message in std::mem::take(&mut self.activity.failed_sends) {
            self.set_native_entry(message, crate::dispatch::MESSAGE_CONDITION, object);
        }
    }

    /// The objects `message`'s `~notify` named sent `messageComplete` while
    /// the condition its send failed with unwinds, its record set aside
    /// meanwhile. A notifier's failure is answered with its own record in
    /// the activity's, and the set-aside one below it where `keep` is set:
    /// on the oracle a held send's notifier runs above the levels the
    /// condition was raised in, and a started one's after they have gone.
    pub(crate) fn notify_failed_send(
        &mut self,
        message: ObjRef,
        keep: bool,
    ) -> Result<(), Failure> {
        let frame = self.roots.activity_mut().push_frame();
        self.roots.activity_mut().push_temp(message);
        let unwinding = self.set_aside_unwinding();
        let notified = self.notify_parties(message);
        match &notified {
            Ok(()) => self.restore_unwinding(unwinding),
            Err(_) if keep => self.restore_unwinding_below(unwinding),
            Err(_) => {}
        }
        self.roots.activity_mut().pop_frame(frame);
        notified
    }

    /// The running activity's record of the failure now unwinding, taken
    /// off it with its objects on the temps.
    fn set_aside_unwinding(&mut self) -> Unwinding {
        let activity = &mut self.activity;
        let unwinding = Unwinding {
            site: activity.failure_site.take(),
            sites: std::mem::take(&mut activity.failure_sites),
            frame: activity.failure_frame.take(),
            frames: std::mem::take(&mut activity.failure_frames),
            origin: activity.failure_origin.take(),
            propagated: std::mem::take(&mut activity.failure_propagated),
            reraised: std::mem::take(&mut activity.failure_reraised),
            reraised_object: activity.reraised_object.take(),
        };
        let temps = self.roots.activity_mut();
        for &object in unwinding
            .frame
            .iter()
            .chain(&unwinding.frames)
            .chain(&unwinding.reraised_object)
        {
            temps.push_temp(object);
        }
        unwinding
    }

    /// Puts what [`Interp::set_aside_unwinding`] took below the record of
    /// the failure unwinding now: its levels outside this one's.
    fn restore_unwinding_below(&mut self, unwinding: Unwinding) {
        let activity = &mut self.activity;
        let site = activity.failure_site.take();
        activity.failure_sites.extend(site);
        activity.failure_sites.extend(unwinding.sites);
        activity.failure_site = unwinding.site;
        let frame = activity.failure_frame.take();
        activity.failure_frames.extend(frame);
        activity.failure_frames.extend(unwinding.frames);
        activity.failure_frame = unwinding.frame;
        if activity.failure_origin.is_none() {
            activity.failure_origin = unwinding.origin;
        }
        activity.failure_propagated |= unwinding.propagated;
    }

    /// Puts back what [`Interp::set_aside_unwinding`] took.
    fn restore_unwinding(&mut self, unwinding: Unwinding) {
        let activity = &mut self.activity;
        activity.failure_site = unwinding.site;
        activity.failure_sites = unwinding.sites;
        activity.failure_frame = unwinding.frame;
        activity.failure_frames = unwinding.frames;
        activity.failure_origin = unwinding.origin;
        activity.failure_propagated = unwinding.propagated;
        activity.failure_reraised = unwinding.reraised;
        activity.reraised_object = unwinding.reraised_object;
    }

    /// [`Interp::reraise_condition_object`] for a `SIGNAL ON` trap, which
    /// names its instruction in the object (`RexxActivation::trap`).
    pub(crate) fn trap_reraised_condition_object(
        &mut self,
        object: ObjRef,
        origin: Option<(Package, Option<usize>)>,
    ) -> Result<ObjRef, Failure> {
        let object = self.reraise_condition_object(object, origin)?;
        let instruction = self.text(b"SIGNAL");
        self.hash_entry_write(object, key::INSTRUCTION, instruction)?;
        Ok(object)
    }

    /// [`Interp::build_condition_object`] for a condition the innermost
    /// native call raised, whose own frame leads `STACKFRAMES` and
    /// `TRACEBACK` (`NativeActivation::createStackFrame`).
    pub(crate) fn build_native_condition_object(
        &mut self,
        raised: &Raised,
    ) -> Result<ObjRef, Failure> {
        self.build_condition_object_from(raised, None, true, None)
    }

    /// [`Interp::build_native_condition_object`] for a `CALL ON` trap, which
    /// names its instruction.
    pub(crate) fn build_trapped_native_condition_object(
        &mut self,
        raised: &Raised,
    ) -> Result<ObjRef, Failure> {
        self.build_condition_object_from(raised, Some(true), true, None)
    }

    fn build_condition_object_from(
        &mut self,
        raised: &Raised,
        call: Option<bool>,
        native: bool,
        unwound: Option<&Unwound<'_>>,
    ) -> Result<ObjRef, Failure> {
        let frame = self.roots.activity_mut().push_frame();

        let syntax = raised.condition == "SYNTAX";
        let (frames, traceback, frame_line) = self.condition_frames(native, unwound)?;

        // Each value is rooted as it is made: the allocations after it can
        // collect any that are not.
        let mut entries: Vec<(&[u8], ObjRef)> = Vec::new();
        let condition = self.text(raised.condition.as_bytes());
        self.roots.activity_mut().push_temp(condition);
        entries.push((key::CONDITION, condition));
        let description = self.text(raised.description.as_deref().unwrap_or(b""));
        self.roots.activity_mut().push_temp(description);
        entries.push((key::DESCRIPTION, description));
        if let Some(call) = call {
            let instruction = self.text(if call { b"CALL" } else { b"SIGNAL" });
            self.roots.activity_mut().push_temp(instruction);
            entries.push((key::INSTRUCTION, instruction));
        }
        let unwound_origin = unwound.and_then(|unwound| unwound.origin);
        let origin = unwound_origin
            .map(|(package, _)| package)
            .or_else(|| self.running_program().map(Package::Program))
            .unwrap_or(Package::Rexx);
        // A failure that never had a Rexx level (no level left a line or a
        // package, and none is running): `generateProgramInformation` finds no
        // Rexx frame, so the object has no `PACKAGE` or `POSITION` and an
        // empty `PROGRAM`.
        let frameless = unwound.is_some_and(|unwound| {
            unwound.origin.is_none() && unwound.sites.iter().all(|site| site.line().is_none())
        }) && self.running_program().is_none();
        if !frameless {
            let package = self.package_object(origin);
            self.roots.activity_mut().push_temp(package);
            entries.push((key::PACKAGE, package));
        }
        // `generateProgramInformation` puts no `POSITION` where the frame it
        // takes the package from is native, which has no line.
        let reraised = unwound.is_some_and(|unwound| unwound.reraised);
        let packaged_native = native
            && self
                .activity
                .native_handles
                .last()
                .is_some_and(|frame| frame.packaged);
        let position = if packaged_native || frameless {
            None
        } else if raised.position != 0 && !reraised {
            // Captured at the raise, which is the only correct source when
            // the raising activation has since unwound.
            Some(self.counted(raised.position as usize))
        } else {
            match (unwound_origin, frame_line) {
                (Some((_, line)), _) => line.map(|line| self.counted(line)),
                (None, Some(line)) => Some(line),
                // No frame at all, which a raise from outside any activation
                // would be; the clause state is the only line there is.
                (None, None) => Some(self.counted(self.activity.clause_state.line())),
            }
        };
        if let Some(position) = position {
            self.roots.activity_mut().push_temp(position);
            entries.push((key::POSITION, position));
        }
        let program = match origin {
            _ if frameless => Vec::new(),
            Package::Program(id) if !self.library_programs.contains(&id) => {
                self.program_display_name(id).to_vec()
            }
            Package::Program(_) => self.program_path.clone().into_bytes(),
            Package::Rexx => crate::LIBRARY_PACKAGE_NAME.to_vec(),
        };
        let program = self.text_built(program);
        self.roots.activity_mut().push_temp(program);
        entries.push((key::PROGRAM, program));
        let propagated = match unwound.is_some_and(|unwound| unwound.propagated) {
            true => self.text(b"1"),
            false => self.text(b"0"),
        };
        self.roots.activity_mut().push_temp(propagated);
        entries.push((key::PROPAGATED, propagated));
        self.roots.activity_mut().push_temp(frames);
        entries.push((key::STACKFRAMES, frames));
        self.roots.activity_mut().push_temp(traceback);
        entries.push((key::TRACEBACK, traceback));

        if let Some(rc) = self.activity.pending_rc.take() {
            self.roots.activity_mut().push_temp(rc);
            entries.push((key::RC, rc));
        } else if let Some(rc) = raised.rc.as_deref() {
            let rc = self.text(rc);
            self.roots.activity_mut().push_temp(rc);
            entries.push((key::RC, rc));
        }
        // The same bytes as `RC`, which is why `Raised` carries a flag
        // rather than the value.
        if raised.result_is_rc
            && let Some(rc) = raised.rc.as_deref()
        {
            let result = self.text(rc);
            self.roots.activity_mut().push_temp(result);
            entries.push((key::RESULT, result));
        }
        if syntax {
            let code = self.text(format!("{}.{}", raised.number, raised.sub).as_bytes());
            self.roots.activity_mut().push_temp(code);
            entries.push((key::CODE, code));
            let errortext = raised.message(raised.number, 0);
            let errortext = self.text_built(errortext);
            self.roots.activity_mut().push_temp(errortext);
            entries.push((key::ERRORTEXT, errortext));
            let message = raised.message(raised.number, raised.sub);
            let message = self.text_built(message);
            self.roots.activity_mut().push_temp(message);
            entries.push((key::MESSAGE, message));
        }
        // **Rendered bytes, not the object the raise named.** `Raised`
        // carries its substitutions as bytes, so a `RAISE ... ADDITIONAL (an
        // array)` rebuilds an Array of that array's strings rather than
        // handing back the original. Right for `SYNTAX`, whose substitutions
        // are text to begin with; an approximation for `USER`.
        if let Some(result) = self.activity.pending_result.take() {
            self.roots.activity_mut().push_temp(result);
            entries.push((key::RESULT, result));
        }
        match self.activity.pending_additional.take() {
            // The raise's own object, whatever its class.
            Some(object) => {
                self.roots.activity_mut().push_temp(object);
                entries.push((key::ADDITIONAL, object));
            }
            // A `SYNTAX` condition always carries the entry, as the Array its
            // substitutions make -- empty for one raised with none, measured
            // as an `Array` of no items rather than `.NIL`.
            None if syntax => {
                let additional = self.additional_array(&raised.additional);
                self.roots.activity_mut().push_temp(additional);
                entries.push((key::ADDITIONAL, additional));
            }
            None => {}
        }

        // A native call's object is handed to the extension as it is, so it
        // gets the `Directory` a program can use; a trapped one is only ever
        // seen through `condition_copy`.
        let object = if native && call.is_none() {
            let order = key::order(syntax);
            entries.sort_by_key(|(name, _)| order.iter().position(|k| k == name));
            self.store_directory(&entries)?
        } else {
            let class = self
                .classes()
                .lookup("Directory")
                .expect("Directory is a native class");
            let object = self.native_instance(class);
            for (name, value) in entries {
                self.hash_entry_write(object, name, value)?;
            }
            object
        };
        self.roots.activity_mut().pop_frame(frame);
        self.roots.activity_mut().push_temp(object);
        Ok(object)
    }

    /// `conditionobj->copy()`, what `CONDITION('O')` answers: a new
    /// store-backed `Directory` holding the same items.
    pub(crate) fn condition_copy(&mut self, object: ObjRef) -> Result<ObjRef, Failure> {
        let Some(Body::Native(native)) = self.heap.get(object).map(|held| &held.body) else {
            let caller = self.caller();
            let copy = self
                .send_message(object, b"COPY", None, &[], caller)?
                .unwrap_or(ObjRef::NIL);
            self.roots.activity_mut().push_temp(copy);
            return Ok(copy);
        };
        let entries: Vec<(&[u8], ObjRef)> = key::order(native.entry(key::CODE).is_some())
            .iter()
            .filter_map(|name| Some((*name, native.entry(name)?)))
            .collect();
        let frame = self.roots.activity_mut().push_frame();
        let copy = self.store_directory(&entries)?;
        self.roots.activity_mut().pop_frame(frame);
        self.roots.activity_mut().push_temp(copy);
        Ok(copy)
    }

    /// A store-backed `Directory` holding `entries`, put in their order. The
    /// caller roots every value.
    fn store_directory(&mut self, entries: &[(&[u8], ObjRef)]) -> Result<ObjRef, Failure> {
        let class = self
            .classes()
            .lookup("Directory")
            .expect("Directory is a native class");
        let caller = self.caller();
        let object = self
            .send_message(class, b"NEW", None, &[], caller)?
            .expect("Directory~new answers an instance");
        self.roots.activity_mut().push_temp(object);
        for &(name, value) in entries {
            let index = self.text(name);
            self.roots.activity_mut().push_temp(index);
            crate::dispatch::hash::store_insert(self, object, index, value)?;
        }
        Ok(object)
    }

    /// The `STACKFRAMES` and `TRACEBACK` pair, innermost first. Both walk the
    /// same frames: the first holds the `StackFrame` objects and the second
    /// the trace line each one renders as, which is why they print alike and
    /// answer different classes.
    fn condition_frames(
        &mut self,
        native: bool,
        unwound: Option<&Unwound<'_>>,
    ) -> Result<(ObjRef, ObjRef, Option<ObjRef>), Failure> {
        let list_class = self
            .classes()
            .lookup("List")
            .expect("List is a native class");
        // **`NEW` rather than `native_instance`.** A `List` is a
        // `Body::Instance` carrying the class's behaviour, which is what
        // `is_list`'s `class_of_value`/`is_a` pair asks about; a
        // `Body::Native` of class `List` matches no `Primitive` arm and every
        // send to it is refused as "one of the interpreter's own objects".
        // Measured: building them that way refused before the program touched
        // the object at all.
        let frames = self.new_list(list_class)?;
        let lines = self.new_list(list_class)?;
        // `POSITION` is the **innermost frame's** line, not
        // `clause_state.line()`. By the time a raise is trapped from a called
        // routine the clause state has moved to the caller -- measured, the
        // oracle answers 4 for a `RAISE ERROR 5` inside a routine where the
        // caller's own clause is line 2. The frame already carries the right
        // number, so it is read back rather than derived a second way.
        let mut position = None;
        if let Some(unwound) = unwound {
            for &frame in &unwound.frames {
                let caller = self.caller();
                self.send_message(frames, b"APPEND", None, &[Some(frame)], caller)?;
            }
            for site in unwound.sites {
                let mut text = Vec::new();
                site.push_trace_line(&mut text);
                let line = self.text_built(text);
                self.roots.activity_mut().push_temp(line);
                let caller = self.caller();
                self.send_message(lines, b"APPEND", None, &[Some(line)], caller)?;
            }
        }
        let mut levels = crate::dispatch::context::live_levels(self);
        // A native call's own condition leads with its frame even where no
        // activation called it.
        if native
            && let Some(row) = self.activity.native_handles.len().checked_sub(1)
            && !levels.contains(&LiveLevel::Native(row))
        {
            levels.insert(0, LiveLevel::Native(row));
        }
        for level in levels {
            let frame = crate::dispatch::context::build_live_frame(self, level)?;
            self.roots.activity_mut().push_temp(frame);
            let caller = self.caller();
            self.send_message(frames, b"APPEND", None, &[Some(frame)], caller)?;
            let caller = self.caller();
            let line = self
                .send_message(frame, b"TRACELINE", None, &[], caller)?
                .unwrap_or(ObjRef::NIL);
            self.roots.activity_mut().push_temp(line);
            let caller = self.caller();
            self.send_message(lines, b"APPEND", None, &[Some(line)], caller)?;
            if level == LiveLevel::Activation(0) {
                let caller = self.caller();
                position = self.send_message(frame, b"LINE", None, &[], caller)?;
            }
        }
        Ok((frames, lines, position))
    }

    /// An empty `List`, built the way a program's `.List~new` builds one so
    /// that its body kind and behaviour are the ones the collection methods
    /// expect.
    fn new_list(&mut self, list_class: ObjRef) -> Result<ObjRef, Failure> {
        let caller = self.caller();
        let list = self
            .send_message(list_class, b"NEW", None, &[], caller)?
            .expect("List~new answers an instance");
        self.roots.activity_mut().push_temp(list);
        Ok(list)
    }

    /// One entry of a condition object, or `None` where it has none.
    /// `CONDITION('A')` reads `ADDITIONAL` back through this rather than
    /// re-deriving it, so the two cannot disagree.
    pub(crate) fn condition_entry(&mut self, object: ObjRef, name: &[u8]) -> Option<ObjRef> {
        if let Some(Body::Native(native)) = self.heap.get(object).map(|held| &held.body) {
            return native.entry(name).filter(|answer| *answer != ObjRef::NIL);
        }
        let index = self.text(name);
        self.roots.activity_mut().push_temp(index);
        let caller = self.caller();
        self.send_message(object, b"AT", None, &[Some(index)], caller)
            .ok()
            .flatten()
            .filter(|answer| *answer != ObjRef::NIL)
    }

    /// `ADDITIONAL`: an Array of the raise's substitutions, empty for a
    /// `SYNTAX` condition that carried none -- measured, `1/0` answers an
    /// Array with no items rather than `.NIL`.
    fn additional_array(&mut self, values: &[Vec<u8>]) -> ObjRef {
        let frame = self.roots.activity_mut().push_frame();
        let mut slots = Vec::with_capacity(values.len());
        for value in values {
            let item = self.text(value);
            self.roots.activity_mut().push_temp(item);
            slots.push(Some(item));
        }
        let array =
            self.alloc_charged(rexx_core::BehaviourId::ARRAY, rexx_core::Body::array(slots));
        self.roots.activity_mut().pop_frame(frame);
        self.roots.activity_mut().push_temp(array);
        array
    }
}

#[cfg(test)]
mod tests {
    /// **A SYNTAX condition object's entries survive the allocations that
    /// build it.** Under a collection at every allocation, `MESSAGE`,
    /// `ERRORTEXT` and `PROGRAM` read back as under none; the oracle prints
    /// the same line. Each of those is long enough to live in the arena, where
    /// `CODE` is not, which is why reading `CODE` alone could not see a
    /// missing root.
    #[test]
    fn a_syntax_condition_objects_entries_survive_a_collection_at_every_allocation() {
        let text = b"signal on syntax\n\
            y = 1/0\n\
            exit\n\
            syntax:\n\
            c = condition('O')\n\
            say c~code '|' c~message '|' c~errortext '|' (c~program == .context~package~name)\n"
            .to_vec();
        let name = "/tmp/condition_entries.rex";
        let plain = crate::run_program(name, text.clone(), crate::Invocation::none());
        assert_eq!(
            plain.exit_code,
            0,
            "{}",
            String::from_utf8_lossy(&plain.stderr)
        );
        assert_eq!(
            String::from_utf8_lossy(&plain.stdout),
            "42.3 | Arithmetic overflow; divisor must not be zero. | Arithmetic overflow/underflow. \
             | 1\n"
        );
        let swept = crate::run_program_collect_every_alloc(name, text, crate::Invocation::none());
        assert_eq!(swept.exit_code, plain.exit_code);
        assert_eq!(swept.stdout, plain.stdout);
        assert_eq!(swept.stderr, plain.stderr);
        assert!(swept.collections > 0, "the stress mode did not collect");
    }

    /// **A `CALL ON` handler's condition object is a root for as long as the
    /// handler runs**, a routine it calls included, once the trap queue has
    /// handed it to the handler's activation. The stdout is the oracle's; the
    /// output before the nested call is what gives a collection somewhere to
    /// fall between the handover and the read.
    #[test]
    fn a_call_on_handlers_condition_object_survives_a_collection_at_every_allocation() {
        let text = b"call on error name onerror\n\
            \"sh -c 'exit 3'\"\n\
            exit\n\
            onerror:\n\
            say 'a command condition'\n\
            call report\n\
            return\n\
            report:\n\
            o = condition('O')\n\
            say o~class~id\n\
            return\n"
            .to_vec();
        let name = "/tmp/handler_condition.rex";
        let plain = crate::run_program(name, text.clone(), crate::Invocation::none());
        assert_eq!(
            plain.exit_code,
            0,
            "{}",
            String::from_utf8_lossy(&plain.stderr)
        );
        assert_eq!(
            String::from_utf8_lossy(&plain.stdout),
            "a command condition\nDirectory\n"
        );
        let swept = crate::run_program_collect_every_alloc(name, text, crate::Invocation::none());
        assert_eq!(swept.exit_code, plain.exit_code);
        assert_eq!(swept.stdout, plain.stdout);
        assert_eq!(swept.stderr, plain.stderr);
        assert!(swept.collections > 0, "the stress mode did not collect");
    }
}
