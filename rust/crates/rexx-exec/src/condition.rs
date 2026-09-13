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

//! The condition object `CONDITION('O')` answers: the Directory
//! `Activity::createConditionObject` builds, with the entries the raised
//! condition's own kind carries.

use rexx_core::ObjRef;

use crate::Interp;
use crate::error::{Failure, Raised};

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
}

impl Interp {
    /// The condition object for `raised`, as the handler will read it back.
    ///
    /// `call` is whether a `CALL ON` trap took it, which is what
    /// `INSTRUCTION` names -- **the trapping instruction, not the raising
    /// clause**, measured `CALL` under `CALL ON` and `SIGNAL` under `SIGNAL
    /// ON` for the same failing command.
    ///
    /// **Which indexes appear depends on the condition**, measured on four
    /// kinds. Nine are always there; `RC` joins them wherever the raise
    /// carries one; `RESULT` only on the command path; and a `SYNTAX`
    /// condition adds `ADDITIONAL`, `CODE`, `ERRORTEXT` and `MESSAGE`. A
    /// directory that carried all of them for every kind would answer
    /// plausibly and wrongly.
    pub(crate) fn build_condition_object(
        &mut self,
        raised: &Raised,
        call: bool,
    ) -> Result<ObjRef, Failure> {
        let class = self
            .classes()
            .lookup("Directory")
            .expect("Directory is a native class");
        let object = self.native_instance(class);
        self.roots.push_temp(object);
        let frame = self.roots.push_frame();

        let syntax = raised.condition == "SYNTAX";
        let (frames, traceback, frame_line) = self.condition_frames()?;

        let mut entries: Vec<(&[u8], ObjRef)> = Vec::new();
        let condition = self.text(raised.condition.as_bytes());
        entries.push((key::CONDITION, condition));
        let description = self.text(raised.description.as_deref().unwrap_or(b""));
        entries.push((key::DESCRIPTION, description));
        let instruction = self.text(if call { b"CALL" } else { b"SIGNAL" });
        entries.push((key::INSTRUCTION, instruction));
        let package = self.condition_package();
        entries.push((key::PACKAGE, package));
        let position = if raised.position != 0 {
            // Captured at the raise, which is the only correct source when
            // the raising activation has since unwound.
            self.counted(raised.position as usize)
        } else {
            match frame_line {
                Some(line) => line,
                // No frame at all, which a raise from outside any activation
                // would be; the clause state is the only line there is.
                None => self.counted(self.clause_state.line()),
            }
        };
        entries.push((key::POSITION, position));
        let program = self.text(self.program_path.clone().as_bytes());
        entries.push((key::PROGRAM, program));
        // Always `0` here: this crate re-raises through `RAISE PROPAGATE`
        // without rebuilding the object, so nothing sets it to 1 yet.
        let propagated = self.text(b"0");
        entries.push((key::PROPAGATED, propagated));
        entries.push((key::STACKFRAMES, frames));
        entries.push((key::TRACEBACK, traceback));

        if let Some(rc) = raised.rc.as_deref() {
            let rc = self.text(rc);
            entries.push((key::RC, rc));
        }
        // The same bytes as `RC`, which is why `Raised` carries a flag
        // rather than the value.
        if raised.result_is_rc
            && let Some(rc) = raised.rc.as_deref()
        {
            let result = self.text(rc);
            entries.push((key::RESULT, result));
        }
        if syntax {
            let code = self.text(format!("{}.{}", raised.number, raised.sub).as_bytes());
            entries.push((key::CODE, code));
            let errortext = raised.message(raised.number, 0);
            let errortext = self.text_built(errortext);
            entries.push((key::ERRORTEXT, errortext));
            let message = raised.message(raised.number, raised.sub);
            let message = self.text_built(message);
            entries.push((key::MESSAGE, message));
        }
        // **Rendered bytes, not the object the raise named.** `Raised`
        // carries its substitutions as bytes, so a `RAISE ... ADDITIONAL (an
        // array)` rebuilds an Array of that array's strings rather than
        // handing back the original. Right for `SYNTAX`, whose substitutions
        // are text to begin with; an approximation for `USER`.
        match self.pending_additional.take() {
            // The raise's own object, whatever its class.
            Some(object) => entries.push((key::ADDITIONAL, object)),
            // A `SYNTAX` condition always carries the entry, as the Array its
            // substitutions make -- empty for one raised with none, measured
            // as an `Array` of no items rather than `.NIL`.
            None if syntax => {
                let additional = self.additional_array(&raised.additional);
                entries.push((key::ADDITIONAL, additional));
            }
            None => {}
        }

        for (name, value) in entries {
            let index = self.text(name);
            self.roots.push_temp(index);
            let caller = self.caller();
            self.send_message(object, b"PUT", None, &[Some(value), Some(index)], caller)?;
        }
        self.roots.pop_frame(frame);
        self.roots.push_temp(object);
        Ok(object)
    }

    /// The `STACKFRAMES` and `TRACEBACK` pair, innermost first. Both walk the
    /// same frames: the first holds the `StackFrame` objects and the second
    /// the trace line each one renders as, which is why they print alike and
    /// answer different classes.
    fn condition_frames(&mut self) -> Result<(ObjRef, ObjRef, Option<ObjRef>), Failure> {
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
        for depth in 0..self.frames().count() {
            let frame = crate::dispatch::context::build_frame(self, depth)?;
            self.roots.push_temp(frame);
            let caller = self.caller();
            self.send_message(frames, b"APPEND", None, &[Some(frame)], caller)?;
            let caller = self.caller();
            let line = self
                .send_message(frame, b"TRACELINE", None, &[], caller)?
                .unwrap_or(ObjRef::NIL);
            self.roots.push_temp(line);
            let caller = self.caller();
            self.send_message(lines, b"APPEND", None, &[Some(line)], caller)?;
            if depth == 0 {
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
        self.roots.push_temp(list);
        Ok(list)
    }

    /// One entry of a condition object, or `None` where it has none.
    /// `CONDITION('A')` reads `ADDITIONAL` back through this rather than
    /// re-deriving it, so the two cannot disagree.
    pub(crate) fn condition_entry(&mut self, object: ObjRef, name: &[u8]) -> Option<ObjRef> {
        let index = self.text(name);
        self.roots.push_temp(index);
        let caller = self.caller();
        self.send_message(object, b"AT", None, &[Some(index)], caller)
            .ok()
            .flatten()
            .filter(|answer| *answer != ObjRef::NIL)
    }

    /// `PACKAGE`: the running program's own, or the `REXX` package when
    /// nothing is running.
    fn condition_package(&mut self) -> ObjRef {
        // `plan::Package` and not `internal_routines::Package`: this crate has
        // two enums of that name and only one of them is what a package
        // object stands for.
        match self.running_program() {
            Some(program) => self.package_object(crate::plan::Package::Program(program)),
            None => self.package_object(crate::plan::Package::Rexx),
        }
    }

    /// `ADDITIONAL`: an Array of the raise's substitutions, empty for a
    /// `SYNTAX` condition that carried none -- measured, `1/0` answers an
    /// Array with no items rather than `.NIL`.
    fn additional_array(&mut self, values: &[Vec<u8>]) -> ObjRef {
        let frame = self.roots.push_frame();
        let mut slots = Vec::with_capacity(values.len());
        for value in values {
            let item = self.text(value);
            self.roots.push_temp(item);
            slots.push(Some(item));
        }
        let array = self.alloc_with(rexx_core::BehaviourId::ARRAY, rexx_core::Body::array(slots));
        self.roots.pop_frame(frame);
        self.roots.push_temp(array);
        array
    }
}
