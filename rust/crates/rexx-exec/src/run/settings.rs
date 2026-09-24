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

//! `ADDRESS`, `NUMERIC` and `TRACE`, and the `>I>`/`<I<` invocation lines.

use super::{
    Announced, Code, DirectiveKind, Expr, Failure, Form, Interp, Loud, NumericSetting, ObjRef,
    Raised, Trace, TraceEntry, is_whole_number, raised_from_settings, raised_invalid_trace_letter,
    raised_naming_the_operand, raised_numeric_trace_interactive_only,
};

/// The longest `ADDRESS` environment name accepted, beyond which the
/// instruction raises 29.1.
const MAX_ADDRESS_NAME_LENGTH: usize = 250;

impl Interp {
    /// `debugSkip`: how many further pauses to skip, from a numeric `TRACE`
    /// typed at one. A negative count also suppresses the echo -- measured,
    /// `trace 2` traces the two clauses it skips the pause for and
    /// `trace -2` traces neither.
    fn set_debug_skip(&mut self, count: i64) -> Result<(), Failure> {
        if !self.debug_pause {
            return Err(raised_numeric_trace_interactive_only().into());
        }
        self.activation_mut().debug.skip = count.abs();
        if count < 0 {
            // **Through `set_trace_mode`, which keeps the cache
            // `Interp::trace_mode` reads.** Writing the activation's field
            // directly left the cache holding the un-suppressed setting, and
            // only the accessor's own `debug_assert` would have said so.
            let current = self.trace_mode();
            self.activation_mut().debug.saved = Some(current);
            self.set_trace_mode(crate::trace::TraceMode {
                debug: true,
                letter: current.letter,
                ..crate::trace::TraceMode::OFF
            });
        }
        // The setting changed from inside the pause, so the pause ends
        // without reading another line.
        self.activation_mut().debug.bypass = true;
        Ok(())
    }

    /// One parsed `TRACE` setting, merged with the setting in force.
    ///
    /// **A `TRACE` instruction is ignored while interactive debug is on**,
    /// and traced like any other clause: measured, a program that reaches
    /// `trace ?` and then `trace off` under `?R` still answers `?R` from
    /// `TRACE()` and keeps tracing. The builtin has no such guard, which is
    /// why it lives here rather than in `set_trace_mode`.
    fn apply_trace_request(&mut self, request: &crate::trace::TraceRequest) {
        // **`inDebug()` is `isDebug() && !debugPause`**, so the same
        // instruction the program cannot use is what a line typed at the
        // pause uses to end debug -- measured, `trace off` at a prompt ends
        // the session and the pause with it.
        if self.trace_mode().debug && !self.debug_pause {
            return;
        }
        let merged = crate::trace::applied(self.trace_mode(), request);
        self.set_trace_mode(merged);
    }

    /// `NUMERIC DIGITS`/`FUZZ`/`FORM`, in every spelling the parser produces
    /// (`NumericSetting`, `rexx-parse`'s own `instruction.rs::numeric`).
    pub(super) fn exec_trace(&mut self, code: &Code<'_>, setting: &Trace) -> Result<(), Failure> {
        match setting {
            Trace::Default => {
                // `setTraceNormal`, which is silent here and is *not*
                // `TRACE OFF` -- measured, `trace r` then bare `trace` then
                // `trace()` gives `N`, where `trace off` gives `O`.
                self.set_trace_mode(crate::trace::TraceMode::NORMAL);
            }
            Trace::Setting(bytes) => {
                let request = crate::trace::parse_trace_request(bytes)
                    .expect("rexx-parse's check_trace_setting already validated this byte");
                self.apply_trace_request(&request);
            }
            // `debugSkip` (`RexxActivation.cpp:932`-`945`): valid only from
            // a pause, and 24.901 anywhere else -- measured, `trace 0` in a
            // program raises it exactly like `trace 5`.
            Trace::Skip(count) => {
                self.set_debug_skip(*count)?;
            }
            // `TRACE VALUE expr`: computed at run time, then classified
            // exactly like a literal `TRACE` setting would have been --
            // measured, `trace value 5` raises 24.901 like `trace 5`, and
            // `trace value 'R'` behaves exactly like `trace r` (this
            // task's report has both transcripts). A whole number is a
            // skip count checked *before* trying it as a letter, matching
            // `rexx-parse`'s own `trace` parser's order (`instruction.rs`'s
            // `whole_number` attempt precedes its `check_trace_setting`
            // fallback).
            Trace::Value(expression) => {
                let value = self.eval(code, expression)?;
                self.roots.push_temp(value);
                // `result->requestString()` (`instructions/TraceInstruction
                // .cpp:172`), and 24.1 names the conversion: measured, oracle
                // rc 232, `trace value .K` with a class-side `makeString`
                // returning `'ZZ'` reports `found "Z"`.
                let value = self.required_string_value(value)?;
                let text = self.to_text(value).to_vec();
                if is_whole_number(&text) {
                    let count = String::from_utf8_lossy(&text)
                        .trim()
                        .parse::<i64>()
                        .unwrap_or(0);
                    self.set_debug_skip(count)?;
                    return Ok(());
                }
                let request = crate::trace::parse_trace_request(&text)
                    .map_err(raised_invalid_trace_letter)?;
                self.apply_trace_request(&request);
            }
        }
        // `RexxActivation::setTrace` calls `traceEntry()` right after
        // installing the new flags (`RexxActivation.cpp:1024`), which is the
        // route every 4c-reachable `>I>` takes. `Trace::Skip` never gets here
        // -- it returned above -- and the C++ agrees: its own arm sets no
        // flags and calls nothing.
        self.trace_invocation_entry();
        Ok(())
    }

    /// `>I>`, if this activation is a `::ROUTINE` still on its first
    /// instruction and the setting just installed traces labels.
    pub(crate) fn trace_invocation_entry(&mut self) {
        if !self.activation().trace_entry.may_announce() {
            return;
        }
        let Some(subject) = self.invocation_subject() else {
            return;
        };
        if !self.trace_mode().labels {
            return;
        }
        self.activation_mut().trace_entry = TraceEntry::Done;
        let package = self.program_path.clone().into_bytes();
        self.trace_invocation(">I>", &subject, &package);
    }

    /// `>I>` for a body whose package's `::OPTIONS TRACE` put a
    /// label-tracing setting in force before its first clause.
    pub(crate) fn trace_package_invocation_entry(&mut self) {
        if !self.trace_mode().labels || self.activation().trace_entry != TraceEntry::Pending {
            return;
        }
        let Some(subject) = self.invocation_subject() else {
            return;
        };
        self.activation_mut().trace_entry = TraceEntry::Done;
        let package = self.program_path.clone().into_bytes();
        self.trace_invocation(">I>", &subject, &package);
    }

    /// `<I<`, on every way a routine activation can end.
    pub(crate) fn trace_invocation_exit(&mut self) {
        if self.activation().trace_entry != TraceEntry::Done {
            return;
        }
        let Some(subject) = self.invocation_subject() else {
            return;
        };
        if !self.trace_mode().labels {
            return;
        }
        let package = self.program_path.clone().into_bytes();
        self.trace_invocation("<I<", &subject, &package);
    }

    /// What the running activation announces itself as, or `None` when it
    /// announces nothing at all -- which is the `isMethodOrRoutine()` half of
    /// the gate, expressed as the lookup that would supply the substitutions.
    pub(crate) fn invocation_subject(&self) -> Option<Announced> {
        if let Some(identity) = &self.activation().method_identity {
            return Some(Announced::Method {
                name: identity.name.to_vec(),
                scope: self.class_id_text(identity.scope).as_bytes().to_vec(),
            });
        }
        let index = self.activation().body?;
        match &self.activation().program.directives.get(index)?.kind {
            DirectiveKind::Routine(routine) => Some(Announced::Routine {
                name: routine.name.to_vec(),
            }),
            _ => None,
        }
    }

    /// `ADDRESS`'s three environment-naming forms. The caller has already
    /// turned the command and `WITH` forms away.
    /// ```text
    /// address envC                    ->  ENVC
    /// address 'LiTeRaL'               ->  LiTeRaL
    /// nm = 'mIxEd'; address value nm  ->  mIxEd
    /// nm = 'mIxEd'; address (nm)      ->  mIxEd
    /// ```
    pub(super) fn exec_address(
        &mut self,
        code: &Code<'_>,
        address: &rexx_parse::Address,
    ) -> Result<(), Failure> {
        // `environment` before `dynamic`, mirroring the C++'s own `if
        // (environment != OREF_NULL)` ahead of its `ADDRESS VALUE` arm. The
        // parser never fills both, so the order decides nothing today.
        match (&address.environment, &address.dynamic) {
            (None, None) => {
                self.activation_mut().address.toggle();
                return Ok(());
            }
            (Some(environment), _) => {
                if environment.len() > MAX_ADDRESS_NAME_LENGTH {
                    return Err(Raised::environment_name_too_long(
                        MAX_ADDRESS_NAME_LENGTH,
                        environment,
                    )
                    .into());
                }
                self.activation_mut().address.set_bytes(environment);
                self.store_io_config(environment, address);
            }
            (None, Some(expression)) => {
                let value = self.eval(code, expression)?;
                self.roots.push_temp(value);
                // `_address = result->requestString()` before the `>>>`
                // (`instructions/AddressInstruction.cpp:182`), so the trace
                // names the conversion: measured, `trace r` over
                // `address (.K)` with a class-side `makeString` returning
                // `'CMD'` prints `>>>   "CMD"` and `ADDRESS()` answers `CMD`.
                let value = self.required_string_value(value)?;
                // Built in the lent buffer: the rendering exists only to be
                // traced and then copied into the `Rc`, so an owned `Vec` of
                // its own would be an allocation and a free per `ADDRESS
                // VALUE` clause.
                let mut text = self.take_result_buffer();
                text.extend_from_slice(&self.to_text(value));
                self.trace_result(self.clause_state.current_value_indent, &text);
                if text.len() > MAX_ADDRESS_NAME_LENGTH {
                    let raised = Raised::environment_name_too_long(MAX_ADDRESS_NAME_LENGTH, &text);
                    self.give_result_buffer(text);
                    return Err(raised.into());
                }
                self.activation_mut().address.set_bytes(&text);
                self.store_io_config(&text, address);
                self.give_result_buffer(text);
            }
        }
        Ok(())
    }

    pub(super) fn exec_numeric(
        &mut self,
        code: &Code<'_>,
        setting: &NumericSetting,
        expression: &Option<Expr>,
    ) -> Result<(), Failure> {
        match setting {
            NumericSetting::Digits => {
                match self.numeric_operand(code, expression, "DIGITS")? {
                    Some((operand, text)) => {
                        let parsed = String::from_utf8_lossy(&text);
                        let outcome = self.activation_mut().settings.set_digits_str(&parsed);
                        self.give_result_buffer(text);
                        if let Err(error) = outcome {
                            let named = self.to_text(operand).to_vec();
                            return Err(raised_naming_the_operand(error, &named).into());
                        }
                    }
                    // The reset stores the default and makes the operand
                    // form's FUZZ check against it, which is the arm the
                    // interpreter runs: it can fail, and the value it names
                    // is the default rather than the setting in force.
                    None => {
                        let default = self.package_default_numeric().digits();
                        self.activation_mut()
                            .settings
                            .reset_digits(default)
                            .map_err(raised_from_settings)?;
                    }
                }
            }
            NumericSetting::Fuzz => match self.numeric_operand(code, expression, "FUZZ")? {
                Some((operand, text)) => {
                    let parsed = String::from_utf8_lossy(&text);
                    let outcome = self.activation_mut().settings.set_fuzz_str(&parsed);
                    self.give_result_buffer(text);
                    if let Err(error) = outcome {
                        let named = self.to_text(operand).to_vec();
                        return Err(raised_naming_the_operand(error, &named).into());
                    }
                }
                None => {
                    let default = self.package_default_numeric().fuzz();
                    self.activation_mut()
                        .settings
                        .reset_fuzz(default)
                        .map_err(raised_from_settings)?;
                }
            },
            // **`FormDefault` and `FormScientific` part company once
            // `::OPTIONS FORM ENGINEERING` moves the package default**:
            // measured, a bare `numeric form` in such a file answers
            // `ENGINEERING` where `numeric form scientific` answers
            // `SCIENTIFIC`.
            NumericSetting::FormDefault => {
                let default = self.package_default_numeric().form();
                self.activation_mut().settings.set_form(default);
            }
            NumericSetting::FormScientific => {
                self.activation_mut().settings.set_form(Form::Scientific);
            }
            NumericSetting::FormEngineering => {
                self.activation_mut().settings.set_form(Form::Engineering);
            }
            NumericSetting::FormValue => {
                // The parser only ever produces this with an expression: an
                // explicit `VALUE` with none is 35.917 at parse time
                // (`instruction.rs::numeric`), and the implicit spelling
                // (`NUMERIC FORM (expr)`) only takes this branch once a token
                // is already known to be there. Loud rather than a panic, on
                // this crate's own rule against aborting on a shape the
                // grammar rules out but the type does not.
                let Some(expression) = expression else {
                    return Err(Loud {
                        message: "NUMERIC FORM VALUE with no expression".to_string(),
                    }
                    .into());
                };
                // `set_form_str`'s own doc comment: the runtime `VALUE` path
                // does no uppercasing, no trimming and no abbreviation, unlike
                // the keyword spellings above -- measured, `numeric form
                // value 'engineering'` is 25.11, not accepted
                // case-insensitively.
                let value = self.eval(code, expression)?;
                self.roots.push_temp(value);
                let text = self.to_text(value).to_vec();
                // `>K>   "FORM" => "engineering"` (F2, branch review): fires
                // before `set_form_str`'s own validation, same as `DIGITS`/
                // `FUZZ` below and `setup_controlled`'s own `TO`/`BY`/`FOR`
                // -- measured, `numeric form value 'engineering'` under
                // `trace r` traces `>K>` and *then* raises 25.11, not the
                // reverse. Untranslated, matching the error report's own
                // `found "engineering"` substitution, which is also
                // unmodified case -- `set_form_str`'s own no-uppercasing
                // rule for this one path, unlike the two keyword spellings
                // above.
                self.trace_keyword(self.clause_state.current_value_indent, "FORM", &text);
                // The required-string protocol between the `>K>` and the
                // validation, which is where `requestString` sits
                // (`instructions/NumericInstruction.cpp:175`).
                let converted = self.required_string_value(value)?;
                let parsed = String::from_utf8_lossy(&self.to_text(converted)).into_owned();
                if let Err(error) = self.activation_mut().settings.set_form_str(&parsed) {
                    return Err(raised_naming_the_operand(error, &text).into());
                }
            }
        }
        Ok(())
    }

    /// Evaluates `expression`, or answers `default` when there is none
    /// (`NUMERIC DIGITS`/`FUZZ` alone).
    fn numeric_operand(
        &mut self,
        code: &Code<'_>,
        expression: &Option<Expr>,
        keyword: &str,
    ) -> Result<Option<(ObjRef, Vec<u8>)>, Failure> {
        let Some(expression) = expression else {
            return Ok(None);
        };
        let value = self.eval(code, expression)?;
        self.roots.push_temp(value);
        let mut text = self.take_result_buffer();
        text.extend_from_slice(&self.to_text(value));
        self.trace_keyword(self.clause_state.current_value_indent, keyword, &text);
        // **The required-string protocol runs after the `>K>` and its answer
        // replaces the bytes**, which is `requestUnsignedNumber`'s own
        // `requestString()` (`classes/ObjectClass.cpp:1077`). The trace above
        // names the object and the parse below reads the conversion --
        // measured, `numeric digits .K` with a class-side `makeString`
        // returning `12` traces `>K>   "DIGITS" => "The K class"` and then
        // answers `12` to `DIGITS()`.
        let converted = self.required_string_value(value)?;
        if converted != value {
            text.clear();
            text.extend_from_slice(&self.to_text(converted));
        }
        Ok(Some((value, text)))
    }
}
