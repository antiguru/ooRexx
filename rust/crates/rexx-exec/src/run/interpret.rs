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
    BodyEngine, Code, Failure, Flow, Fragment, Interp, Loud, NameShape, Raised, Rc, TraceEntry,
    VariableRef, parse_interpret, raised_iterate_no_loop, raised_iterate_no_match,
    raised_leave_no_loop, raised_leave_no_match, shape_of, split_indirect_words,
    validate_indirect_word,
};

impl Interp {
    /// Parses `text` as an `INTERPRET` fragment and runs it **inside the
    /// current activation**.
    /// ```text
    ///      2 *-*   leave outer
    ///      2 *-*   interpret "leave outer"
    /// ```
    pub(super) fn run_fragment(&mut self, text: Vec<u8>) -> Result<Flow, Failure> {
        let fragment: Rc<Fragment> = match parse_interpret(text) {
            Ok(fragment) => Rc::new(fragment),
            // **Step 5b: the oracle's own condition, not a loud refusal.**
            // Measured, `interpret "do forever then"` on line 2 raises 27.901
            // at rc 229; this used to be `Loud::parse`, `rexx-exec: INTERPRET
            // text did not parse: ...` at rc 120. `error.rs`'s own `impl
            // From<&ParseError> for Raised` has the transcript and states
            // exactly what the conversion cannot carry.
            Err(error) => return Err(Raised::from(&error).into()),
        };

        // An owned `Fragment` would do here, since nothing but this loop reads
        // it. It is an `Rc` because an `INTERPRET` inside a fragment makes this
        // function reentrant and each level anchors its own.
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
        let temps = self.roots.push_frame();
        let arena = self.roots.frames();
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
        // Released on both paths, exactly as `run_chunk` does: a frame left
        // behind would keep its registers rooted for the rest of the run.
        arena.release(registers);
        self.roots.pop_frame(temps);
        let flow = match ran {
            Ok(flow) => flow,
            Err(failure) => {
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
                self.seal_site_level();
                let raised = match name {
                    None => raised_leave_no_loop(),
                    Some(id) => raised_leave_no_match(fragment.symbols.name(id).as_bytes()),
                };
                Err(raised.into())
            }
            Flow::Iterate(name, origin) => {
                self.record_leave_failure(&origin);
                self.seal_site_level();
                let raised = match name {
                    None => raised_iterate_no_loop(),
                    Some(id) => raised_iterate_no_match(fragment.symbols.name(id).as_bytes()),
                };
                Err(raised.into())
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

    /// `doDebugPause` (`RexxActivation.cpp:4199`-`4283`), after a clause the
    /// setting in force traced. Answers whether the clause is to run again,
    /// which is what `=` asks for.
    ///
    /// The prompt is printed once per activation, and a line that is neither
    /// empty nor `=` runs as an `INTERPRET` fragment; the pause ends when the
    /// fragment ended debug or changed a setting from inside it.
    pub(crate) fn debug_pause_after_clause(&mut self) -> Result<bool, Failure> {
        if self.debug_pause {
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
            let Some(line) = self.input_line() else {
                return Ok(false);
            };
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

    /// One line typed at a pause, run as an `INTERPRET` fragment that traces
    /// nothing. A SYNTAX condition inside it is reported under the two
    /// `+++ Interactive trace.` lines and swallowed, and the pause goes on --
    /// measured, `zz = 1/0` at a pause reports 42.3 and the next line still
    /// runs.
    fn run_debug_fragment(&mut self, text: Vec<u8>) -> Result<(), Failure> {
        let saved_pause = self.replace_debug_pause(true);
        self.fragment_depth += 1;
        let saved_entry = self.enter_fragment(self.clause_line_override.is_some());
        let outcome = self.run_fragment(text);
        let depth = self.fragment_depth;
        self.pending_traps
            .retain(|pending| pending.fragment_depth != depth);
        self.fragment_depth -= 1;
        self.leave_fragment(saved_entry);
        self.replace_debug_pause(saved_pause);
        match outcome {
            Ok(_) => Ok(()),
            Err(Failure::Raised(raised)) => {
                self.report_debug_error(&raised);
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
