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

//! `INTERPRET` fragments, `DROP`, and the interactive debug pause.

use super::{
    BodyEngine, Code, Failure, Flow, Fragment, Instruction, InstructionKind, Interp, Loud,
    NameShape, ObjRef, ProgramSource, Rc, TraceEntry, VariableRef, raised_iterate_no_loop,
    raised_iterate_no_match, raised_leave_no_loop, raised_leave_no_match, shape_of,
    split_indirect_words, validate_indirect_word,
};

impl Interp {
    /// Parses `text` as an `INTERPRET` fragment and runs it **inside the
    /// current activation**.
    /// ```text
    ///      2 *-*   leave outer
    ///      2 *-*   interpret "leave outer"
    /// ```
    pub(super) fn run_fragment(&mut self, text: Vec<u8>) -> Result<Flow, Failure> {
        let source = rexx_parse::ProgramSource::new(text, rexx_parse::SourceKind::Interpret);
        let fragment: Rc<Fragment> = match rexx_parse::fragment_from(source) {
            Ok(fragment) => Rc::new(fragment),
            Err(rejected) => return Err(self.raise_parse_failure(&rejected, None)),
        };

        // An owned `Fragment` would do here, since nothing but this loop reads
        // it. It is an `Rc` because an `INTERPRET` inside a fragment makes this
        // function reentrant and each level anchors its own.
        if let Some(level) = self.activity.fragments.last_mut() {
            level.fragment = Some(Rc::clone(&fragment));
        }
        let (slots, fragment_plan) = self.fragment_plan(&fragment);
        let code = Code {
            body: &fragment.body,
            symbols: &fragment.symbols,
            slots: &slots,
            // **A fragment carries its own plan, with every slot translated
            // into the enclosing frame** (`Interp::fragment_plan`). It used to
            // carry none, so its clause indents and compound splits were
            // computed the way they were before either table existed; it needs
            // one now because a fragment compiles, and a chunk's `Op::Load`
            // and `Op::Store` carry plan slots.
            plan: Some(&fragment_plan),
        };

        // `exit` inside `INTERPRET` ends the program, not the fragment, so
        // it has to propagate rather than stop here -- `run_bounded`'s own
        // catch-all does exactly that for anything it does not own, `Exit`
        // included, with nothing fragment-specific to add.
        let chunk = match crate::ir::compile(&fragment.body, &fragment_plan, self.chunk_trace()) {
            Ok(chunk) => chunk,
            Err(_) => {
                self.seal_site_level();
                return Err(Loud::chunk_refused().into());
            }
        };
        // Truncated on the way out, so a temp this chunk leaks does not outlive it.
        let temps = self.roots.activity_mut().push_frame();
        let arena = self.roots.activity().frames();
        let registers = arena.reserve(chunk.registers);
        let ran = self.run_bounded(
            &code,
            0,
            code.body.instructions.len(),
            Some(&fragment.source),
            BodyEngine::Chunk {
                chunk: &chunk,
                registers,
            },
        );
        // Released on both paths, exactly as `Interp::drive` does: a frame left
        // behind would keep its registers rooted for the rest of the run.
        arena.release(registers);
        self.roots.activity_mut().pop_frame(temps);
        let flow = match ran {
            Ok(flow) => flow,
            Err(failure) => {
                self.capture_fragment_frame(&failure);
                self.seal_site_level();
                return Err(failure);
            }
        };

        // The exhausted search, at the fragment's own boundary rather than
        // the program's: measured, the oracle's `LEAVE`/`ITERATE` search
        // does not cross an `INTERPRET` (this function's own doc comment has
        // the six transcripts). Byte-identical in shape to
        // `run_activation`'s own four arms, deliberately -- same four
        // constructors, same `record_leave_failure` call -- because it is
        // the same event happening at a different boundary.
        match flow {
            Flow::Leave(name, origin) => {
                self.record_leave_failure(&origin);
                let failure = match name {
                    None => raised_leave_no_loop(),
                    Some(id) => raised_leave_no_match(fragment.symbols.name(id).as_bytes()),
                }
                .into();
                self.capture_fragment_frame(&failure);
                self.seal_site_level();
                Err(failure)
            }
            Flow::Iterate(name, origin) => {
                self.record_leave_failure(&origin);
                let failure = match name {
                    None => raised_iterate_no_loop(),
                    Some(id) => raised_iterate_no_match(fragment.symbols.name(id).as_bytes()),
                }
                .into();
                self.capture_fragment_frame(&failure);
                self.seal_site_level();
                Err(failure)
            }
            other => Ok(other),
        }
    }

    /// Drops one `DROP` target: a plain variable, a whole stem, one tail, or
    /// the `(v)` indirect form.
    /// ```text
    /// a=1; b=2; v='a b'      ; drop (v); say a; say b   ->  A / B  (both dropped)
    /// x=1;      v=' x '      ; drop (v); say x           ->  X      (trimmed)
    /// v='9'                  ; drop (v)                  ->  Error 31.2
    /// v='.x'                 ; drop (v)                  ->  Error 31.3
    /// w=1;      v='(w)'      ; drop (v)                  ->  Error 20.928
    /// a=1;      v='a'        ; drop (v); say a           ->  A      (agrees with the old reading)
    /// ```
    pub(super) fn drop_variable(
        &mut self,
        code: &Code<'_>,
        variable: &VariableRef,
    ) -> Result<(), Failure> {
        match variable {
            VariableRef::Direct(id) => {
                let name = code.symbols.name(*id);
                if shape_of(name.as_bytes()) == NameShape::Compound {
                    let (stem_name, stem_at) = code.stem(*id);
                    let key = self.tail_key(code, *id)?;
                    self.stem_drop_tail_at(stem_name, stem_at, &key);
                } else {
                    self.drop_by_name(name.as_bytes());
                }
            }
            VariableRef::Indirect(id) => {
                let (value, _novalue) = self.read(code, *id);
                let value = self.required_string_value(value)?;
                let text = self.to_text(value).into_owned();
                let mut names = Vec::new();
                for word in split_indirect_words(&text) {
                    names.push(validate_indirect_word(word)?);
                }
                for name in &names {
                    self.drop_by_name(name);
                }
            }
        }
        Ok(())
    }

    /// Drops the variable, whole stem, or one verbatim-keyed tail `name`'s
    /// own spelling names, dispatched by `shape_of`.
    pub(super) fn drop_by_name(&mut self, name: &[u8]) {
        match shape_of(name) {
            NameShape::Simple => {
                let slot = self.slot_of(name);
                let frame = self.activation().frame;
                self.clear_variable(frame, slot);
            }
            NameShape::Stem => self.stem_drop(name),
            NameShape::Compound => {
                let dot = name
                    .iter()
                    .position(|&b| b == b'.')
                    .expect("NameShape::Compound guarantees at least one period");
                let (stem_name, key) = name.split_at(dot + 1);
                self.stem_drop_tail(stem_name, key);
            }
        }
    }

    /// The pause owed once `clause` has done its own work under a setting
    /// that is interactive, `flow` being what it answered if it answered one.
    /// Answers whether the clause is to run again, which is what `=` asks for.
    /// A failure the pause raises is recorded against `clause`.
    ///
    /// The oracle decides per instruction class (`pauseInstruction`,
    /// `pauseLabel`) against flags `TraceSetting::setDebug` derives from the
    /// letter: an instruction pauses under `A`, `R` and `I` and a label
    /// wherever labels trace. A command pauses from
    /// [`Interp::debug_pause_after_command`] instead. A negative skip count
    /// suppresses the echo and not the pauses it counts, so the flags come
    /// from the setting it saved. A block header that answers a `Flow` ran
    /// its whole construct, a flow that leaves the instruction for good owes
    /// no pause, and an `ITERATE` pauses once the loop it reaches has
    /// stepped.
    #[cold]
    #[inline(never)]
    pub(crate) fn debug_pause_after_clause(
        &mut self,
        code: &Code<'_>,
        index: usize,
        clause: &Instruction,
        source: Option<&ProgramSource>,
        flow: Option<&Flow>,
    ) -> Result<bool, Failure> {
        let kind = &clause.kind;
        if let Some(flow) = flow
            && (matches!(
                flow,
                Flow::Exit(_) | Flow::Return(_) | Flow::Signal(_) | Flow::Iterate(..)
            ) || matches!(
                kind,
                InstructionKind::Do(_)
                    | InstructionKind::Loop(_)
                    | InstructionKind::If { .. }
                    | InstructionKind::Select { .. }
            ))
        {
            return Ok(false);
        }
        let mode = self.pausing_mode();
        let pausing = match kind {
            InstructionKind::Label { .. } => mode.labels,
            InstructionKind::Command { .. } => false,
            InstructionKind::Address(address) if address.command.is_some() => false,
            _ => mode.all && pauses_after(kind),
        };
        if !pausing {
            return Ok(false);
        }
        self.debug_pause_now()
            .inspect_err(|_| self.record_failure_site(code, index, source, clause))
    }

    /// `conditionalPauseInstruction`: the pause an instruction takes from
    /// inside its own work, `INTERPRET`'s before its fragment, a `DO` whose
    /// header ends it before any pass, and `ITERATE`'s once the loop has
    /// stepped. Answers whether the instruction is to run again.
    pub(crate) fn debug_pause_instruction(&mut self) -> Result<bool, Failure> {
        let mode = self.pausing_mode();
        if !(mode.debug && mode.all) {
            return Ok(false);
        }
        self.debug_pause_now()
    }

    /// The pause `RexxActivation::command` takes once a command it traced
    /// has settled `RC` and raised its condition. Answers whether the command
    /// is to run again.
    pub(crate) fn debug_pause_after_command(
        &mut self,
        status: crate::command::ReturnStatus,
    ) -> Result<bool, Failure> {
        let mode = self.pausing_mode();
        let traced = mode.all
            || mode.commands
            || match status {
                crate::command::ReturnStatus::Error => mode.errors,
                crate::command::ReturnStatus::Failure => mode.failures,
                crate::command::ReturnStatus::Normal => false,
            };
        if !(mode.debug && traced) {
            return Ok(false);
        }
        self.debug_pause_now()
    }

    /// The setting whose flags decide which clauses pause.
    fn pausing_mode(&self) -> crate::trace::TraceMode {
        match self.activation().debug.saved {
            Some(saved) => saved,
            None => self.trace_mode(),
        }
    }

    /// `doDebugPause` (`RexxActivation.cpp:4199`-`4283`). Answers whether the
    /// clause is to run again.
    ///
    /// The prompt is printed once per activation, and a line that is neither
    /// empty nor `=` runs as an `INTERPRET` fragment; the pause ends when the
    /// fragment ended debug or changed a setting from inside it.
    pub(crate) fn debug_pause_now(&mut self) -> Result<bool, Failure> {
        if self.debug_pause() {
            return Ok(false);
        }
        if self.activation().debug.bypass {
            self.activation_mut().debug.bypass = false;
            return Ok(false);
        }
        let skip = self.activation().debug.skip;
        if skip > 0 {
            let left = skip - 1;
            self.activation_mut().debug.skip = left;
            if left == 0
                && let Some(saved) = self.activation_mut().debug.saved.take()
            {
                self.set_trace_mode(saved);
            }
            return Ok(false);
        }
        if !self.activation().debug.prompt_issued {
            self.activation_mut().debug.prompt_issued = true;
            let line = crate::error::Raised::debug_prompt_line();
            let start = self.trace.len();
            self.trace.extend_from_slice(&line);
            self.trace.push(b'\n');
            self.route_trace_line(start);
        }
        loop {
            let line = self.debug_input_line()?;
            if line.is_empty() {
                return Ok(false);
            }
            if line == b"=" {
                return Ok(true);
            }
            self.run_debug_fragment(line)?;
            if self.activation().debug.bypass {
                self.activation_mut().debug.bypass = false;
                return Ok(false);
            }
            if !self.trace_mode().debug {
                return Ok(false);
            }
        }
    }

    /// `Activity::traceInput` (`Activity.cpp:3240`-`3263`): `LINEIN` sent to
    /// `.local~DEBUGINPUT`, an absent entry or a `.nil` answer read as the
    /// null string. A condition the send raises is the pause's failure.
    fn debug_input_line(&mut self) -> Result<Vec<u8>, Failure> {
        let Some(route) = self.local_route(b"DEBUGINPUT")? else {
            return Ok(Vec::new());
        };
        let caller = self.caller();
        let answer = pinned!(
            self,
            crate::pinning::PinKind::TraceWrapper,
            self.send_message(route, crate::dispatch::LINEIN, None, &[], caller)
        )?;
        Ok(match answer {
            Some(value) if value != ObjRef::NIL => self.to_text(value).into_owned(),
            _ => Vec::new(),
        })
    }

    /// One line typed at a pause, run as an `INTERPRET` fragment that traces
    /// nothing. A SYNTAX condition inside it is reported under the two
    /// `+++ Interactive trace.` lines and swallowed, and the pause goes on --
    /// measured, `zz = 1/0` at a pause reports 42.3 and the next line still
    /// runs.
    fn run_debug_fragment(&mut self, text: Vec<u8>) -> Result<(), Failure> {
        let saved_pause = self.replace_debug_pause(true);
        // A `SELECT CASE` typed at the pause opens and closes inside the
        // line; the construct the pause interrupted keeps its own value.
        let saved_case = self.activation().current_case;
        if let Some(case) = saved_case {
            self.roots.activity_mut().push_temp(case);
        }
        self.activity.fragment_depth += 1;
        let (line, indent) = (
            self.activity.clause_state.line(),
            self.activity.clause_state.current_value_indent,
        );
        self.enter_fragment_level(line, indent);
        let saved_entry = self.enter_fragment(self.activity.clause_line_override.is_some());
        let outcome = pinned!(
            self,
            crate::pinning::PinKind::Interpret,
            self.run_fragment(text)
        );
        let depth = self.activity.fragment_depth;
        self.activity
            .pending_traps
            .retain(|pending| pending.fragment_depth != depth);
        self.activity.fragment_depth -= 1;
        self.leave_fragment_level();
        self.leave_fragment(saved_entry);
        self.replace_debug_pause(saved_pause);
        self.activation_mut().current_case = saved_case;
        match outcome {
            Ok(_) => Ok(()),
            Err(Failure::Raised(raised)) => {
                self.report_debug_error(&raised);
                self.clear_failure_levels();
                Ok(())
            }
            Err(other) => Err(other),
        }
    }

    /// The two lines a failing pause line prints: the major and the sub, each
    /// behind `+++ Interactive trace.  Error`.
    fn report_debug_error(&mut self, raised: &crate::error::Raised) {
        for line in raised.debug_error_lines() {
            let start = self.trace.len();
            self.trace.extend_from_slice(&line);
            self.trace.push(b'\n');
            self.route_trace_line(start);
        }
    }

    /// Records a fragment entered from the running activation's clause at
    /// `line` and `indent`.
    pub(super) fn enter_fragment_level(&mut self, line: usize, indent: usize) {
        let owner = self.activation().id;
        self.activity.fragments.push(crate::FragmentLevel {
            owner,
            fragment: None,
            line,
            indent,
            outer_clause: self.activity.fragment_clause,
        });
    }

    /// Forgets the innermost fragment, whose enclosing one's clause is
    /// current again.
    pub(super) fn leave_fragment_level(&mut self) {
        if let Some(level) = self.activity.fragments.pop() {
            self.activity.fragment_clause = level.outer_clause;
        }
    }

    /// Gives a fragment its own `>I>` count, and answers with the enclosing
    /// state for [`Interp::leave_fragment`] to put back.
    pub(super) fn enter_fragment(&mut self, nested: bool) -> TraceEntry {
        let enclosing = self.activation().trace_entry;
        let entry = if enclosing.may_announce() && !nested {
            TraceEntry::Pending
        } else {
            TraceEntry::Spent
        };
        self.activation_mut().trace_entry = entry;
        enclosing
    }

    /// Puts the enclosing state back after a fragment, **except** that a
    /// fragment which announced leaves the activation `Done`.
    pub(super) fn leave_fragment(&mut self, enclosing: TraceEntry) {
        if self.activation().trace_entry != TraceEntry::Done {
            self.activation_mut().trace_entry = enclosing;
        }
    }
}

/// Whether a debug pause follows an instruction of `kind` once its own work
/// is done, as the oracle's `execute` ends in `pauseInstruction` or a block
/// header's in `conditionalPauseInstruction`. `INTERPRET` pauses before its
/// fragment instead. `REPLY` answers false: the oracle pauses on the
/// continuation's thread, and this crate does not pause there.
pub(crate) fn pauses_after(kind: &InstructionKind) -> bool {
    !matches!(
        kind,
        InstructionKind::Then
            | InstructionKind::Else { .. }
            | InstructionKind::Otherwise
            | InstructionKind::End { .. }
            | InstructionKind::Return { .. }
            | InstructionKind::Exit { .. }
            | InstructionKind::Signal(_)
            | InstructionKind::Raise(_)
            | InstructionKind::Guard(_)
            | InstructionKind::Interpret { .. }
            | InstructionKind::Reply { .. }
    )
}
