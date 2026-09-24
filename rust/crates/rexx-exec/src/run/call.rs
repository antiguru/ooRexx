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

//! `CALL` and function calls: resolving the name, running the arguments, entering the callee.

use super::{
    Activation, BehaviourId, Body, BodyKey, CallContext, CallType, Code, Ended, Expr, ExprKind,
    Failure, Flow, Inherited, InstalledRoutine, Interp, Loud, ObjRef, Raised, Rc, Resolved,
    VarRefHome, body_of, builtin,
};

/// A call's target as it stands when its arguments are about to run.
#[derive(Clone, Copy)]
pub(crate) enum CallResolution<'a> {
    /// Decided before the arguments: a label or a builtin, a namespace-qualified
    /// routine, or a `findRoutine` answer a call site keeps.
    Settled(Resolved),
    /// Searched for once the arguments have run, as `externalCall` is
    /// (`expression/ExpressionFunction.cpp:185-210`). `kept` is a call site's
    /// answer that [`Resolved::kept_until_routines_change`] names, with the
    /// `Interp::routine_generation` it was read under, used again where the
    /// arguments moved no generation. `site` keeps the answer taken.
    AfterArguments {
        kept: Option<(Resolved, u32)>,
        site: Option<crate::ir::CallSiteSlot<'a>>,
    },
}

/// Which of the two activation-pushing outcomes a resolved call took, kept
/// past the push so the decisions that follow it can read it.
#[derive(Copy, Clone)]
pub(super) enum Entered {
    Label(usize),
    Routine(InstalledRoutine),
}

/// How the callee was reached: written in the program, or delivered to it.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) enum CallEntry {
    /// A `CALL`, an internal function call, or either form reaching a
    /// `::ROUTINE` -- a call the program's own text asked for.
    Written,
    /// A `CALL ON` handler, run because a condition was delivered.
    Trap,
}

/// The receiver the callee's calling convention carries (D24), given the
/// caller's own.
pub(super) fn entered_receiver(
    entered: Entered,
    entry: CallEntry,
    caller: Option<ObjRef>,
) -> Option<ObjRef> {
    match (entered, entry) {
        (Entered::Label(_), CallEntry::Written) => caller,
        (Entered::Label(_), CallEntry::Trap) | (Entered::Routine(_), _) => None,
    }
}

/// How many activations may be live at once before `CALL` raises 11.1
/// ("Insufficient control stack space", `Raised::insufficient_stack`).
/// ```text
/// enclosing DO blocks per activation | deepest surviving (debug) | (release)
///                                  0 |                    22,534 |   133,150
///                                  1 |                    14,062 |    94,518
///                                  5 |                     5,616 |         -
///                                 25 |                     1,403 |         -
/// ```
pub(crate) const MAX_ACTIVATION_DEPTH: usize = 10_000;

impl Interp {
    /// Resolves `name` to the thing a call of it runs, with no argument
    /// evaluated and nothing entered: [`Interp::resolve_fixed_call`], then
    /// [`Interp::resolve_routine_call`].
    pub(crate) fn resolve_call(
        &self,
        name: &[u8],
        search_labels: bool,
    ) -> Result<Resolved, Failure> {
        match self.resolve_fixed_call(name, search_labels)? {
            Some(resolved) => Ok(resolved),
            None => self.resolve_routine_call(name),
        }
    }

    /// The half of a call's resolution the oracle decides before its
    /// arguments run: a label (`RexxExpressionFunction::resolve`,
    /// `expression/ExpressionFunction.cpp:154`) or a builtin (the parser's
    /// `builtinIndex`), or the refusal of a builtin Phase 4 excludes. `None`
    /// for a name that is none of those.
    pub(crate) fn resolve_fixed_call(
        &self,
        name: &[u8],
        search_labels: bool,
    ) -> Result<Option<Resolved>, Failure> {
        // **Resolved against the running *activation's* body, not against the
        // body a caller is walking, and the two differ inside an `INTERPRET`
        // fragment.** A fragment's `labels` is always empty -- a label in
        // interpreted text is error 47.1 -- so searching the walked body would
        // make every `CALL` inside a fragment unresolvable. Measured on the
        // oracle: `interpret "call sub"` runs the enclosing program's `sub:`.
        // Found by running the composition rather than by reading the code:
        // the first version of this searched the walked body and passed every
        // test that had no `INTERPRET` in it.
        let program = Rc::clone(&self.activation().program);
        let selector = self.activation().body;
        let Some(activation_body) = body_of(&program, selector) else {
            return Err(Loud::missing_body().into());
        };
        let label = if search_labels {
            activation_body.labels.get(name).copied()
        } else {
            None
        };
        let resolved = match label {
            Some(target) => Resolved::Label(target),
            // **`resolve` rather than `is_builtin`, and it answers the same
            // question.** Every row's name is in scope
            // (`every_implemented_row_names_an_in_scope_builtin`) and a name in
            // scope with no row comes back `Gap`, so `resolve(name).is_some()`
            // and `is_builtin(name)` agree on every name. It costs the same
            // one lookup and keeps which builtin it found.
            None if let Some(target) = builtin::resolve(name) => Resolved::Builtin(target),
            // A builtin Phase 4 excludes outright is still a builtin, so it
            // sits here rather than behind the routine lookup -- see
            // `builtin::is_excluded_builtin`'s own doc for why neither the
            // routine step nor 43.1 is an acceptable answer for one.
            None if builtin::is_excluded_builtin(name) => {
                return Err(Loud::unresolved_call(name).into());
            }
            None => return Ok(None),
        };
        Ok(Some(resolved))
    }

    /// The half of a call's resolution the oracle searches once the
    /// arguments have run (`RexxActivation::externalCall`,
    /// `execution/RexxActivation.cpp:3062-3105`), for a name
    /// [`Interp::resolve_fixed_call`] answered `None` for.
    pub(crate) fn resolve_routine_call(&self, name: &[u8]) -> Result<Resolved, Failure> {
        let resolved = match self.package_routine_lookup(name) {
            Some(resolved) => resolved,
            // **Ahead of the external file search and behind
            // everything above it.** `Setup.cpp` resolves
            // `CoreClasses.orx`'s two `CALL`s against the interpreter's
            // own directory, which is neither a label, a builtin nor a
            // `::ROUTINE`; this crate embeds those files instead. Gated
            // on the bootstrap running, so the names mean nothing to a
            // program.
            None if self.library_bootstrap
                && let Some(program) = rexx_lib::lookup(&String::from_utf8_lossy(name)) =>
            {
                Resolved::Library(program)
            }
            // **A routine of the oracle's own internal packages**, which
            // it consults here -- after the running package's routines and
            // before the external file search. Answering 43.1 for one of
            // these says "no such routine" for a name the oracle does
            // have, which a program cannot tell from its own typo.
            None if let Some(row) = crate::internal_routines::lookup(name) => match row.body {
                Some(_) => Resolved::Internal(row),
                None => {
                    let owner = row
                        .owner
                        .expect("a row with no body names the phase that owes it");
                    return Err(Loud::internal_routine(name, owner).into());
                }
            },
            // **A routine a loaded library exports**, whichever load
            // registered it. The oracle keeps these in the table the
            // internal packages' routines are in, where a library routine
            // replaces an internal one of the same name; here the internal
            // one is found first.
            None if let Some(slot) = self.package_routine(name) => Resolved::LibraryRoutine(slot),
            // **The external file search, which the oracle performs
            // before answering 43.1.** Only whether it resolves is
            // decided here; the path is searched for again where the
            // file is entered, because a `Resolved` is `Copy` and this
            // resolver is `&self`.
            None if self.external_program(name).is_some() => Resolved::External,
            // 43.1, once the search above has found nothing, and
            // raised by the consumer rather than here. Measured in a
            // clean directory with nothing of that name beside the
            // program: `call zorkolo` gives 43.1 rc 213 `Could not find
            // routine "ZORKOLO".`
            None => Resolved::Unresolved,
        };
        Ok(resolved)
    }

    /// The routine `name` reaches from the running package, through
    /// [`Interp::find_routine`].
    ///
    /// **The parent step is what code built by a send resolves through**:
    /// `Routine~newFile` and `Method~newFile` record the context's package or,
    /// with none, the caller's (`new_file_executable`), and `Package~new`
    /// records a context's (`package_from_source`). A file reached by a
    /// *call* records none, so it still sees its own package alone --
    /// measured, the same body finds the caller's `::ROUTINE` through
    /// `Routine~newFile` and raises 43.1 through `call`.
    fn package_routine_lookup(&self, name: &[u8]) -> Option<Resolved> {
        let program = self.running_activation()?.program_id;
        self.find_routine(program, &name.to_ascii_uppercase())
            .map(crate::MergedRoutine::resolved)
    }

    /// Takes the shared value buffer **with the caller's run intact**, and the
    /// depth to build above it.
    pub(crate) fn take_value_buffer(&mut self) -> (Vec<Option<ObjRef>>, usize) {
        let buffer = std::mem::take(&mut self.value_buffer);
        let mark = buffer.len();
        (buffer, mark)
    }

    /// Hands the value buffer back with this run removed, whichever way the
    /// send ended.
    pub(crate) fn give_value_buffer(&mut self, mut buffer: Vec<Option<ObjRef>>, mark: usize) {
        buffer.truncate(mark);
        self.value_buffer = buffer;
    }

    /// One builtin call: its arguments evaluated in the caller, then the row
    /// run over them.
    pub(crate) fn invoke_builtin_call(
        &mut self,
        code: &Code<'_>,
        target: crate::builtin::BuiltinTarget,
        name: &[u8],
        args: &[Option<Expr>],
    ) -> Result<ObjRef, Failure> {
        // **Pushed onto the shared stack one at a time**, rather than into a
        // buffer lent for the whole loop: each argument's evaluation calls
        // back into `&mut self`, so nothing may hold the `Vec` across it, and
        // a run that is only ever appended to needs no borrow between pushes.
        // The compiled path's `Op::PushArg` writes the same stack.
        let mark = self.value_buffer.len();
        for arg in args {
            let value = match arg {
                None => {
                    self.trace_argument(self.clause_state.current_value_indent, b"");
                    None
                }
                Some(expr) if self.leaf_argument(expr) => {
                    Some(self.eval_leaf_argument(code, expr)?)
                }
                Some(expr) => Some(self.eval_traced_argument(code, expr)?),
            };
            self.value_buffer.push(value);
        }
        self.run_over_pushed_args(mark, |interp, values| {
            builtin::run(interp, name, target, values)
        })
    }

    /// Evaluates a call's arguments, settles `resolution` once they have run,
    /// and runs the call, in its own nested activation where it has one.
    pub(crate) fn invoke_call(
        &mut self,
        code: &Code<'_>,
        resolution: CallResolution<'_>,
        name: &[u8],
        args: &[Option<Expr>],
        call_type: CallType,
        entry: CallEntry,
    ) -> Result<Ended, Failure> {
        // **Evaluated in the caller, before anything is pushed**, which is
        // where the argument expressions' own variables live. Observable
        // through failure, and the failure is real and measured: `call sub
        // 1/0` is Error 42.3
        // reported against the `CALL` clause, at rc 214, and a version that
        // skipped evaluation would run the callee instead.
        if let CallResolution::Settled(Resolved::Builtin(target)) = resolution {
            return Ok(Ended::Returned(Some(
                self.invoke_builtin_call(code, target, name, args)?,
            )));
        }
        // A fresh `Vec` and not a lent one: this path always hands the
        // arguments to the callee, which keeps them, so there is nothing to
        // give back and a pool would allocate on every call anyway.
        let mut arguments: Vec<Option<ObjRef>> = Vec::with_capacity(args.len());
        for arg in args {
            match arg {
                None => {
                    // An omitted position traces an **empty** value line, not
                    // no line: `traceArgument(GlobalNames::NULLSTRING)`,
                    // `RexxInstruction.cpp:161`, and measured above.
                    self.trace_argument(self.clause_state.current_value_indent, b"");
                    arguments.push(None);
                }
                Some(expr) if self.leaf_argument(expr) => {
                    arguments.push(Some(self.eval_leaf_argument(code, expr)?));
                }
                Some(expr) => arguments.push(Some(self.eval_traced_argument(code, expr)?)),
            }
        }
        let resolved = self.settle_after_arguments(resolution, name)?;
        // An internal-package routine runs no activation, and neither does a
        // library routine, so they take the builtin discipline over the values.
        if let Resolved::Internal(_) | Resolved::LibraryRoutine(_) = resolved {
            if let Some(handled) = self.call_checkpoint(name, &arguments)? {
                return Ok(Ended::Returned(handled));
            }
            return match resolved {
                Resolved::Internal(row) => {
                    Ok(Ended::Returned(Some(self.run_internal(row, &arguments)?)))
                }
                Resolved::LibraryRoutine(slot) => self
                    .run_package_routine(slot, name, &arguments)
                    .map(Ended::Returned),
                _ => unreachable!("only the two arms above reach here"),
            };
        }
        self.invoke_call_over(resolved, name, arguments, call_type, entry)
    }

    /// What a call runs, once its arguments have run.
    fn settle_after_arguments(
        &self,
        resolution: CallResolution<'_>,
        name: &[u8],
    ) -> Result<Resolved, Failure> {
        let (kept, site) = match resolution {
            CallResolution::Settled(resolved) => return Ok(resolved),
            CallResolution::AfterArguments { kept, site } => (kept, site),
        };
        if let Some((resolved, generation)) = kept
            && generation == self.routine_generation
        {
            return Ok(resolved);
        }
        let resolved = self.resolve_routine_call(name)?;
        if let Some(site) = site {
            site.remember(resolved, self.routine_generation);
        }
        Ok(resolved)
    }

    /// The resolution a call without a site of its own starts from: what
    /// [`Interp::resolve_fixed_call`] decided, or a search once the arguments
    /// have run.
    pub(crate) fn unkept_resolution(fixed: Option<Resolved>) -> CallResolution<'static> {
        match fixed {
            Some(resolved) => CallResolution::Settled(resolved),
            None => CallResolution::AfterArguments {
                kept: None,
                site: None,
            },
        }
    }

    /// One compiled call over the arguments its own ops already evaluated.
    pub(crate) fn call_over_pushed_args(
        &mut self,
        resolved: Resolved,
        name: &[u8],
        mark: usize,
    ) -> Result<ObjRef, Failure> {
        self.run_over_pushed_args(mark, |interp, values| {
            interp.call_over_values(resolved, name, values)
        })
    }

    /// Runs `body` over the argument run standing above `mark`, with the
    /// stack lent out for the duration and this run removed on the way back.
    fn run_over_pushed_args(
        &mut self,
        mark: usize,
        body: impl FnOnce(&mut Interp, &[Option<ObjRef>]) -> Result<ObjRef, Failure>,
    ) -> Result<ObjRef, Failure> {
        let mut values = std::mem::take(&mut self.value_buffer);
        let outcome = body(self, &values[mark..]);
        values.truncate(mark);
        self.value_buffer = values;
        outcome
    }

    /// The security manager's `CALL` checkpoint
    /// (`execution/SecurityManager.cpp:203`-`:225`), asked for every name
    /// that gets past the running package's own `::ROUTINE`s -- including
    /// one nothing answers, which is why it runs before 43.1.
    ///
    /// `Some(result)` where the manager handled the call, nothing then
    /// running; `None` where it declined or none is installed.
    fn call_checkpoint(
        &mut self,
        name: &[u8],
        values: &[Option<ObjRef>],
    ) -> Result<Option<Option<ObjRef>>, Failure> {
        if self.effective_security_manager().is_none() {
            return Ok(None);
        }
        let called = self.text(name);
        self.roots.push_temp(called);
        let arguments = self.security_arguments_array(values);
        let entries = [
            (crate::security::key::NAME, called),
            (crate::security::key::ARGUMENTS, arguments),
        ];
        let Some(info) = self.security_check(crate::security::message::CALL, &entries)? else {
            return Ok(None);
        };
        let result = self.security_entry(info, crate::security::key::RESULT)?;
        Ok(Some(result))
    }

    /// One internal-package routine over its evaluated arguments. No
    /// activation, no `SIGL`, no depth guard -- the builtin discipline, since
    /// the oracle runs these as native code too.
    fn run_internal(
        &mut self,
        row: &'static crate::internal_routines::InternalRoutine,
        values: &[Option<ObjRef>],
    ) -> Result<ObjRef, Failure> {
        self.run_internal_as(row, None, values)
    }

    /// [`Interp::run_internal`] for a call that passes the routine `name`
    /// rather than its upcased row name, which is what a `::ROUTINE` bound to
    /// the row and `Routine~call` pass.
    pub(crate) fn run_internal_as(
        &mut self,
        row: &'static crate::internal_routines::InternalRoutine,
        name: Option<&[u8]>,
        values: &[Option<ObjRef>],
    ) -> Result<ObjRef, Failure> {
        let Some(body) = row.body else {
            let owner = row
                .owner
                .expect("a row with no body names the phase that owes it");
            return Err(Loud::internal_routine(row.name.as_bytes(), owner).into());
        };
        let outcome = body(self, row.name.as_bytes(), values);
        if outcome.is_err() {
            // The oracle reports one of these under its own routine line,
            // whether the argument marshalling or the body raised -- measured,
            // `filespec('D')` and `SysSleep('abc')` both carry it.
            match name {
                Some(name) => self.blame_internal_routine(name),
                None => {
                    let upper = row.name.to_ascii_uppercase();
                    self.blame_internal_routine(upper.as_bytes());
                }
            }
        }
        outcome
    }

    /// [`Interp::call_over_pushed_args`] with the run in hand.
    fn call_over_values(
        &mut self,
        resolved: Resolved,
        name: &[u8],
        values: &[Option<ObjRef>],
    ) -> Result<ObjRef, Failure> {
        // The builtin path, which runs no activation -- the same shortcut
        // `eval_call_resolved` takes and for the same measured reason.
        if let Resolved::Builtin(target) = resolved {
            return builtin::run(self, name, target, values);
        }
        if let Resolved::Internal(row) = resolved {
            if let Some(handled) = self.call_checkpoint(name, values)? {
                return handled.ok_or_else(|| Raised::no_data_returned(name).into());
            }
            return self.run_internal(row, values);
        }
        if let Resolved::LibraryRoutine(slot) = resolved {
            let answered = match self.call_checkpoint(name, values)? {
                Some(handled) => handled,
                None => self.run_package_routine(slot, name, values)?,
            };
            return answered.ok_or_else(|| Raised::no_data_returned(name).into());
        }
        match self.invoke_call_over(
            resolved,
            name,
            values.to_vec(),
            CallType::Function,
            CallEntry::Written,
        )? {
            Ended::Exited(value) => Err(Failure::Exited(value)),
            Ended::Returned(Some(value)) => Ok(value),
            Ended::Returned(None) => Err(Raised::no_data_returned(name).into()),
        }
    }

    /// One `::ROUTINE` entered from a native method body: `Routine~call`,
    /// `~callWith` and `~'[]'`.
    pub(crate) fn call_over_installed_routine(
        &mut self,
        installed: InstalledRoutine,
        arguments: Vec<Option<ObjRef>>,
    ) -> Result<Option<ObjRef>, Failure> {
        match self.invoke_call_over(
            Resolved::Routine(installed),
            b"CALL",
            arguments,
            CallType::Subroutine,
            CallEntry::Written,
        )? {
            Ended::Exited(value) => Err(Failure::Exited(value)),
            Ended::Returned(returned) => Ok(returned),
        }
    }

    /// [`Interp::invoke_call`] past its argument evaluation: everything a
    /// callee needs once its arguments are values.
    pub(crate) fn invoke_call_over(
        &mut self,
        resolved: Resolved,
        name: &[u8],
        arguments: Vec<Option<ObjRef>>,
        call_type: CallType,
        entry: CallEntry,
    ) -> Result<Ended, Failure> {
        // **The builtin outcome ends here**, before `SIGL`, before the depth
        // guard and before any activation is pushed -- each of those three is
        // the label path's, and the oracle answers that the builtin path has
        // none of them (`builtin`'s own module doc carries the probe for
        // each). Every argument is still rooted by the loop above, so the
        // allocation a builtin's result costs happens with the inputs
        // reachable, and the value handed back is rooted by whichever caller
        // receives it exactly as a callee's `RETURN` value already is.
        // **The library outcome ends here too**, and for a reason unlike the
        // builtin's: an embedded `.orx` source is a whole program, with its
        // own directives to install and its own `ProgramId`, so it is
        // entered the way the command line's program is rather than the way
        // a routine is. `Entered` has no arm for it because none of the
        // decisions that enum exists to carry -- `SIGL`, the callee's pool,
        // the indent, the calling convention's receiver -- is a question
        // about it.
        // **Every arm below the running package's own `::ROUTINE`s passes
        // the manager first**, an unresolvable name included.
        if matches!(
            resolved,
            Resolved::Library(_) | Resolved::External | Resolved::Unresolved
        ) && let Some(handled) = self.call_checkpoint(name, &arguments)?
        {
            return Ok(Ended::Returned(handled));
        }
        if let Resolved::Unresolved = resolved {
            return Err(Raised::routine_not_found(name).into());
        }
        if let Resolved::Library(program) = resolved {
            return self
                .enter_library_program(program, Some(arguments))
                .map(Ended::Returned);
        }
        // **And the external file ends here for the same reason**, which is
        // the whole of why it is not an `Entered`: a file the search found is
        // a program, entered the way the command line's is.
        if let Resolved::External = resolved {
            return self
                .enter_external_program(name, arguments, call_type)
                .map(Ended::Returned);
        }

        // A library routine merged into the package, and a `::ROUTINE` bound
        // to native code, run no activation either, and are found before the
        // manager is asked.
        if let Resolved::MergedLibraryRoutine(code) = resolved {
            return self
                .run_library_routine(code, name, &arguments)
                .map(Ended::Returned);
        }
        if let Resolved::Routine(installed) = resolved {
            if let Some(code) = self.library_routine_code(installed) {
                return self
                    .run_library_routine(code, name, &arguments)
                    .map(Ended::Returned);
            }
            if let Some(row) = self.rexx_routine_row(installed) {
                return self
                    .run_internal_as(row, Some(name), &arguments)
                    .map(|value| Ended::Returned(Some(value)));
            }
        }

        let entered = match resolved {
            // Answered above, before the loop that just ran.
            Resolved::Builtin(_) => unreachable!("the builtin path returns before this"),
            Resolved::Internal(_) => unreachable!("the internal path returns before this"),
            Resolved::LibraryRoutine(_) | Resolved::MergedLibraryRoutine(_) => {
                unreachable!("a library routine is run before this")
            }
            Resolved::Label(target) => Entered::Label(target),
            Resolved::Routine(installed) => Entered::Routine(installed),
            Resolved::Library(_) => unreachable!("the library path returns just above"),
            Resolved::External => unreachable!("the external path returns just above"),
            Resolved::Unresolved => unreachable!("an unresolved name raises just above"),
        };

        // The caller's own program and body selector, which a label callee
        // inherits: the same pair `resolve_call` searched, read again here
        // rather than threaded out of it, because what they are wanted for is
        // building the callee rather than finding it.
        let program = Rc::clone(&self.activation().program);
        let program_id = self.activation().program_id;
        let selector = self.activation().body;

        // `SIGL`, set here rather than before the argument loop above: the
        // oracle's own `internalCall` (`RexxActivation.cpp`, read directly)
        // receives its arguments already evaluated by its caller, so they
        // are evaluated under whatever `SIGL` was already in force, and only
        // then does the transfer overwrite it. Measured: `signal there` /
        // `there: call sub sigl` into `sub: use arg a` reports the argument
        // as `1` (the `SIGNAL`'s own line, still in force during evaluation)
        // and `sub`'s own `SIGL` as the `CALL`'s line -- a version setting
        // `SIGL` before evaluating arguments would report the argument as
        // the `CALL`'s own line instead.
        if matches!(entered, Entered::Label(_)) {
            self.set_sigl(self.clause_state.line());
        }

        // D19/I6: one Rust frame per activation, plus this counter, so an
        // unbounded recursion becomes a reportable condition instead of a
        // native abort. `Raised::insufficient_stack` already existed
        // (`error.rs`); measured, the oracle answers the same 11.1 at rc 245
        // for the same program, at its own depth of 27,314.
        if self.activation_depth() >= MAX_ACTIVATION_DEPTH {
            return Err(Raised::insufficient_stack().into());
        }

        let callee_id = self.next_activation_id();
        match entered {
            Entered::Label(target) => {
                // **D9r's default: a shared pool.** The callee reuses the
                // caller's `SlotFrame`, so it reads and writes the caller's
                // variables and its writes survive the return -- measured,
                // and `pop_slots` is deliberately not called on the way out
                // because the frame is not this activation's to free. Task
                // 5's `PROCEDURE` is what will ever push a frame of its own.
                let caller = self.activation();
                let plan = Rc::clone(&caller.plan);
                let frame = caller.frame;
                // The `call_type` parameter is deliberately not read here: an
                // internal label answers `PARSE SOURCE`'s second word from the
                // activation it was called out of, whichever form called it.
                // Measured, a `::routine` invoked as a function whose body
                // `CALL`s a label reads `FUNCTION` inside that label, not
                // `SUBROUTINE`.
                let caller_call_type = caller.call_type;
                let caller_rs = caller.rs;
                let settings = caller.settings.clone();
                let trace_mode = caller.trace_mode;
                // Inherited with `settings` and for its reason: a `SIGNAL ON`
                // in the caller turned the package's escalation off for the
                // rest of that activation, and an internal call is still
                // inside it.
                let condition_syntax = caller.condition_syntax;
                let extra = caller.extra.clone();
                // Cloned with `extra`, and for the same reason: a label
                // reached without `PROCEDURE` shares the caller's pool, and an
                // exposed name is part of that pool. Measured -- a class
                // method exposing `v`, calling a label that assigns `v`, and a
                // second class method reading `v` back -- the assignment
                // reaches the object variable. Without this the label writes
                // the frame slot the exposure left empty and the write is
                // lost.
                let exposed = caller.exposed.clone();
                // Cloned in and never written back, exactly like `settings`
                // and `trace_mode` beside it -- `Activation::traps`' own doc
                // comment has the three probes that measure the inheritance
                // and its one-way direction.
                let traps = caller.traps.clone();
                // Both halves of the pair, not just the current one:
                // measured, a callee's own bare `ADDRESS` swaps to the
                // *caller's* alternate. `Activation::address`' own doc
                // comment has the transcript.
                let address = caller.address.clone();
                let caller_io_configs = caller.io_configs.clone();
                // Same one-way rule again: an internal call sees the caller's
                // `CONDITION()` answers and a reset inside the callee dies
                // with it. `TrappedCondition`'s own doc comment has the
                // four-line transcript.
                let condition = caller.condition.clone();
                let mut callee = Activation::nested(
                    callee_id,
                    program,
                    program_id,
                    selector,
                    plan,
                    frame,
                    target,
                    Inherited {
                        call_type: caller_call_type,
                        rs: caller_rs,
                        settings,
                        condition_syntax,
                        trace_mode,
                        address,
                        // The refcount, not the table: a write in the callee
                        // clones first, which is how the oracle keeps the
                        // inheritance one-way (`checkIOConfigTable`).
                        io_configs: caller_io_configs,
                        traps,
                        condition,
                    },
                );
                callee.extra = extra;
                callee.exposed = exposed;
                self.push_activation(callee);
            }
            // **A pool of its own, and not one of the five inheritances**
            // -- `Activation::routine` is where that is stated and
            // `Activation::nested`'s own doc carries the six probes. The
            // plan is the routine body's own, cached under its own
            // `BodyKey`, and it is what sizes the frame: a routine's names
            // are not the caller's, so a frame sized from the caller's plan
            // would be the wrong length.
            Entered::Routine(installed) => {
                // Reached through `programs` rather than through the running
                // activation's own `Rc`, so the plan's cache key and the
                // activation's program are the same program by construction
                // (`InstalledRoutine`'s own doc).
                let routine_program = Rc::clone(&self.programs[installed.program.0]);
                let Some(body) = body_of(&routine_program, Some(installed.directive)) else {
                    return Err(Loud::missing_body().into());
                };
                let plan = self.plan_for(
                    BodyKey {
                        program: installed.program,
                        directive: Some(installed.directive),
                    },
                    body,
                    &routine_program.symbols,
                    &routine_program.source,
                );
                let frame = self.roots.push_slots(plan.len());
                let mut callee = Activation::routine(
                    callee_id,
                    routine_program,
                    installed.program,
                    installed.directive,
                    plan,
                    frame,
                    call_type,
                );
                // The routine's own package, which is not always the caller's
                // -- `installed.program` is where the `::ROUTINE` was
                // declared. The caller's settings go with it for
                // `::OPTIONS NUMERIC INHERIT`, the one option that reads
                // them.
                self.start_from_package(&mut callee, Some(&self.activation().settings));
                self.push_activation(callee);
                self.trace_package_invocation_entry();
            }
        }

        // Level state for the callee, five pieces, saved here and restored
        // on both paths below. `Interpret`'s own arm is the model for four
        // of them, and one differs from it deliberately -- the fifth,
        // `clause_state`, is not level state for the callee at all, and is
        // saved and restored for a different reason stated where it is:
        let saved_clause_state = self.save_clause_state();
        let callee_indent = match entered {
            Entered::Label(_) => saved_clause_state.value_indent() + 2,
            Entered::Routine(_) => 0,
        };
        let saved_base = std::mem::replace(&mut self.activation_indent, callee_indent);
        let saved_offset = std::mem::take(&mut self.indent_offset);
        let saved_line = std::mem::take(&mut self.clause_line_override);
        let inherited = entered_receiver(entered, entry, self.call_context.receiver);
        let arguments = self.shared_arguments(&arguments);
        let saved_context = std::mem::replace(
            &mut self.call_context,
            CallContext {
                name: name.to_vec(),
                arguments,
                // Read out of the caller's own convention before this
                // replaces it, which is the only place it can be read from:
                // `entered_receiver` carries the rule and the measurement
                // for each of its arms.
                receiver: inherited,
            },
        );

        let ended = self.run_activation();

        // Before the pop, because both halves of `<I<`'s gate are the
        // callee's own -- its `trace_entry` state and its `TRACE` setting.
        // `RexxActivation::termination` is where the C++ puts it, which is
        // likewise inside the activation.
        self.trace_invocation_exit();

        // Popped on both paths, and unconditionally: `run_activation`'s own
        // loop asserts the activation stack is where it found it after every
        // step, so a `CALL` that returned with the callee still on it would
        // trip that assertion in the caller rather than quietly running the
        // wrong frame's `pc`.
        let mut callee = self.pop_activation().expect("the activation just pushed");
        // **The two halves of "was the pool shared" are one bool, and both
        // are needed.** A `PROCEDURE` callee pushed a frame of its own, so
        // that frame is popped here -- on the error path as well, which is
        // why this is not inside the `Ok` arm below. It also keeps its own
        // run-time name bindings, so they are *not* moved back: doing that
        // would overwrite the caller's `extra` with the callee's isolated
        // one. A shared-pool callee is the opposite on both counts, and its
        // `extra` write-back is what makes a name bound inside it survive
        // the return (measured, `interpret "zork = 42"` in a callee).
        if callee.owns_frame {
            self.roots.pop_slots(callee.frame);
        } else {
            // Taken rather than moved out, so the box stays whole and can be
            // parked: moving a field out of a `Box` moves the whole of it out
            // and frees the box, which is the allocation the pool exists to
            // keep. What is left behind is the empty map a fresh activation
            // starts with.
            self.activation_mut().extra = std::mem::take(&mut callee.extra);
        }
        self.recycle_activation(callee);
        self.activation_indent = saved_base;
        self.indent_offset = saved_offset;
        self.clause_line_override = saved_line;
        self.restore_clause_state(saved_clause_state);
        self.call_context = saved_context;

        // **`EXIT` inside a `::ROUTINE` ends the routine, not the program**,
        // where `EXIT` inside a `CALL`ed label ends the program. The C++'s
        // `implicitExit` sets `RETURNED` outright for an `isProgramLevelCall`
        // activation and only otherwise walks up through `exitFrom`
        // (`RexxActivation.cpp:1455`-`1469`), and a routine invocation is one
        // of those. Measured on the oracle, rc 0 every time, and the shapes
        // matter because the exit arrives here by three different routes:
        // ```text
        // call rtn / say result       ::routine rtn ; exit 5      ->  5, and main runs on
        // n1 = rtn() / say n1         ::routine rtn ; exit 7      ->  7
        // call rtn / say result       ::routine rtn ; exit        ->  RESULT, unset
        // call rtn / say 'after'      a label INSIDE the routine exits 9 -> "after" runs
        // call rtn / say 'after'      interpret "exit 4" in the routine  -> "after" runs
        // ```
        let ended = match ended {
            Ok(Ended::Exited(value)) | Err(Failure::Exited(value))
                if matches!(entered, Entered::Routine(_)) =>
            {
                return Ok(Ended::Returned(value));
            }
            other => other,
        };

        match ended {
            Ok(ended) => Ok(ended),
            Err(failure) => {
                // Seal before the failure leaves the callee, never after --
                // `seal_site_level`'s own rule, and the same one
                // `run_fragment` follows. Without it the callee's clause
                // would win `record_failure_at`'s first-wins race outright
                // and the call would never be echoed. Measured, the oracle
                // prints one echo per level, innermost first: a `say 1/0` in
                // a routine called from a routine called from a `DO` gives
                // three lines, at indents 6, 4 and 2.
                self.seal_site_level();
                Err(failure)
            }
        }
    }

    /// [`Interp::resolve_fixed_call`] followed by [`Interp::invoke_call`], with
    /// nothing remembered.
    pub(crate) fn resolve_and_run_call(
        &mut self,
        code: &Code<'_>,
        name: &[u8],
        search_labels: bool,
        args: &[Option<Expr>],
        call_type: CallType,
        entry: CallEntry,
    ) -> Result<Ended, Failure> {
        let fixed = self.resolve_fixed_call(name, search_labels)?;
        self.invoke_call(
            code,
            Self::unkept_resolution(fixed),
            name,
            args,
            call_type,
            entry,
        )
    }

    /// Builds the object a `>name` or `<name` term answers: the variable
    /// `inner` names, rather than the value it holds.
    pub(crate) fn variable_reference(
        &mut self,
        code: &Code<'_>,
        inner: &Expr,
    ) -> Result<ObjRef, Failure> {
        let id = match &inner.kind {
            ExprKind::Variable(id) | ExprKind::Stem(id) => *id,
            other => return Err(Loud::expression(other).into()),
        };
        let slot = match code.slot_for(id) {
            Some(slot) => slot,
            None => self.slot_of(code.symbols.name(id).as_bytes()),
        };
        let frame = self.activation().frame;
        // Resolved here, where the name's own home is: an alias this
        // activation holds is still addressable, which is what makes `>p`
        // work when its `p` came from *its* caller, and an `EXPOSE` binding
        // is on this activation, which is what makes it work on an object
        // variable. Chasing the slot for an exposed name would name the
        // empty slot the exposure left behind.
        let home = match self.exposure(frame, slot) {
            Some(var) => VarRefHome::Instance {
                owner: var.owner,
                scope: var.scope,
            },
            None => VarRefHome::Cell(self.roots.promote(frame, slot)),
        };
        let name = code.symbols.name(id).as_bytes().into();
        Ok(self.alloc_with(
            BehaviourId::OBJECT,
            Body::VarRef(Box::new(rexx_core::VarRef { name, home })),
        ))
    }

    /// One call argument, evaluated, rooted and traced -- the step both call
    /// paths share.
    #[inline(always)]
    fn leaf_argument(&self, expr: &Expr) -> bool {
        !self.tracing_intermediates()
            && matches!(
                expr.kind,
                ExprKind::Literal(_) | ExprKind::Constant(_) | ExprKind::Variable(_)
            )
    }

    /// One argument's value, for a shape whose evaluation produces that
    /// value and nothing else.
    fn eval_leaf_argument(&mut self, code: &Code<'_>, expr: &Expr) -> Result<ObjRef, Failure> {
        let anchor = 0u8;
        self.enter_eval_node(&raw const anchor)?;
        let value = self.eval_node(code, expr);
        self.depth -= 1;
        let value = value?;
        self.roots.push_temp(value);
        Ok(value)
    }

    pub(crate) fn eval_traced_argument(
        &mut self,
        code: &Code<'_>,
        expr: &Expr,
    ) -> Result<ObjRef, Failure> {
        let argument = self.eval(code, expr)?;
        self.roots.push_temp(argument);
        if let Some(rendered) = self.intermediate_text(argument) {
            self.trace_argument(self.clause_state.current_value_indent, &rendered);
        }
        Ok(argument)
    }

    /// A call's resolution, held back until its arguments have run: the oracle
    /// evaluates a call's arguments before its external search
    /// (`expression/ExpressionFunction.cpp:184`, `instructions/CallInstruction.cpp:166`).
    #[inline(always)]
    pub(crate) fn resolved_after_arguments<T>(
        &mut self,
        code: &Code<'_>,
        resolution: Result<T, Failure>,
        args: &[Option<Expr>],
    ) -> Result<T, Failure> {
        match resolution {
            Ok(resolved) => Ok(resolved),
            Err(failure) => Err(self.arguments_before_failure(code, args, failure)),
        }
    }

    /// [`Interp::resolved_after_arguments`]' failing half: run the arguments
    /// for their trace lines and their own conditions, then report the
    /// resolution failure if none of them raised first.
    #[cold]
    #[inline(never)]
    fn arguments_before_failure(
        &mut self,
        code: &Code<'_>,
        args: &[Option<Expr>],
        failure: Failure,
    ) -> Failure {
        // The same three shapes `invoke_call`'s own loop has, for their trace
        // lines and their failures; the values themselves are dropped, since
        // there is no callee to hand them to.
        for arg in args {
            let raised = match arg {
                None => {
                    self.trace_argument(self.clause_state.current_value_indent, b"");
                    continue;
                }
                Some(expr) if self.leaf_argument(expr) => self.eval_leaf_argument(code, expr).err(),
                Some(expr) => self.eval_traced_argument(code, expr).err(),
            };
            if let Some(raised) = raised {
                return raised;
            }
        }
        failure
    }

    /// Runs one named `CALL`: `resolve_fixed_call`, then
    /// [`Interp::invoke_named_call`].
    pub(super) fn exec_call(
        &mut self,
        code: &Code<'_>,
        name: &[u8],
        search_labels: bool,
        args: &[Option<Expr>],
    ) -> Result<Flow, Failure> {
        let fixed = self.resolve_fixed_call(name, search_labels);
        let fixed = self.resolved_after_arguments(code, fixed, args)?;
        self.invoke_named_call(code, Self::unkept_resolution(fixed), name, args)
    }

    /// The `CALL` instruction past its resolution: [`Interp::invoke_call`],
    /// then settle `RESULT` and translate the outcome into this instruction's
    /// own `Flow`.
    pub(crate) fn invoke_named_call(
        &mut self,
        code: &Code<'_>,
        resolution: CallResolution<'_>,
        name: &[u8],
        args: &[Option<Expr>],
    ) -> Result<Flow, Failure> {
        // Captured before `invoke_call` runs the callee, which overwrites
        // `current_value_indent` with its own clauses' -- this is the `CALL`
        // clause's own printed indent, needed below for the caller-side
        // `RESULT` trace.
        let base_indent = self.clause_state.current_value_indent;
        let ended = self.invoke_call(
            code,
            resolution,
            name,
            args,
            CallType::Subroutine,
            CallEntry::Written,
        )?;
        self.settle_call_result(ended, base_indent)
    }

    /// What a `CALL` does with the outcome its callee handed back: the
    /// caller's own `>>>` and `RESULT`.
    fn settle_call_result(&mut self, ended: Ended, base_indent: usize) -> Result<Flow, Failure> {
        let value = match ended {
            // `EXIT` inside the callee ends the program rather than the
            // call, and so does running off the end of the body -- measured
            // both ways. Forwarded unchanged; `RESULT` is never touched on
            // this path.
            Ended::Exited(value) => return Ok(Flow::Exit(value)),
            Ended::Returned(value) => value,
        };

        // **`RESULT` is settled on return and not at the call.** Measured:
        // a caller setting `result = 'before'` and calling a no-`PROCEDURE`
        // routine has the callee print `inside result= before`, so nothing
        // is cleared on the way in. After `return 42` the caller reads `42`;
        // after a bare `return` it reads the derived name `RESULT`, which is
        // what an unset variable renders as.
        let slot = self.reserved_result_slot();
        let frame = self.activation().frame;
        match value {
            Some(value) => {
                // Re-rooted in the caller: `Op::Clause`'s region popped the
                // callee's temps frame around every clause it ran, this one
                // included, so the `push_temp` the `RETURN` arm did is gone
                // by now. Same window `Flow::Exit`'s own arm documents,
                // closed here rather than left open, because unlike an exit
                // value this one goes on to be stored and read.
                self.roots.push_temp(value);
                // The caller's own `>>>`, at the `CALL` clause's indent --
                // `base_indent`, saved before the callee overwrote
                // `current_value_indent` with its own clauses'.
                if let Some(rendered) = self.result_text(value) {
                    self.trace_result(base_indent, &rendered);
                }
                self.set_variable(frame, slot, value);
            }
            None => self.clear_variable(frame, slot),
        }
        Ok(Flow::Next)
    }

    /// [`Interp::invoke_named_call`] over arguments the compiled stream has
    /// already evaluated onto the argument stack above `mark`.
    pub(crate) fn invoke_named_call_over_pushed_args(
        &mut self,
        resolved: Resolved,
        name: &[u8],
        mark: usize,
    ) -> Result<Flow, Failure> {
        let base_indent = self.clause_state.current_value_indent;
        let ended = self.subroutine_over_pushed_args(resolved, name, mark)?;
        self.settle_call_result(ended, base_indent)
    }

    /// The callee half of [`Interp::invoke_named_call_over_pushed_args`]: the
    /// argument run is lent out, turned into `Argument`s, and removed again on
    /// the way back whichever way the call ended.
    fn subroutine_over_pushed_args(
        &mut self,
        resolved: Resolved,
        name: &[u8],
        mark: usize,
    ) -> Result<Ended, Failure> {
        let mut values = std::mem::take(&mut self.value_buffer);
        let outcome = match resolved {
            // No activation, no `Argument`s built -- the shortcut
            // `call_over_values` takes, and the reason a `CALL` to a builtin
            // reaches `Ended::Returned(Some(_))` with a value to settle.
            Resolved::Builtin(target) => builtin::run(self, name, target, &values[mark..])
                .map(|value| Ended::Returned(Some(value))),
            // **Named rather than left to the arm below**, which has a
            // catch-all: an internal routine sent into `invoke_call_over`
            // would reach a `match` that has no arm for it.
            Resolved::Internal(row) => match self.call_checkpoint(name, &values[mark..]) {
                Ok(Some(handled)) => Ok(Ended::Returned(handled)),
                Ok(None) => self
                    .run_internal(row, &values[mark..])
                    .map(|value| Ended::Returned(Some(value))),
                Err(failure) => Err(failure),
            },
            Resolved::LibraryRoutine(slot) => match self.call_checkpoint(name, &values[mark..]) {
                Ok(Some(handled)) => Ok(Ended::Returned(handled)),
                Ok(None) => self
                    .run_package_routine(slot, name, &values[mark..])
                    .map(Ended::Returned),
                Err(failure) => Err(failure),
            },
            _ => self.invoke_call_over(
                resolved,
                name,
                values[mark..].to_vec(),
                CallType::Subroutine,
                CallEntry::Written,
            ),
        };
        values.truncate(mark);
        self.value_buffer = values;
        outcome
    }
}
