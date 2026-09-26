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

//! The instruction loop: `Flow`, `step`, and the two functions that run it.

use crate::activation::{
    Activation, CallType, Entry, Inherited, InstanceVar, ReplyState, TraceEntry, Trap,
    TrappedCondition, body_of,
};
use crate::builtin;
use crate::clause::{ClauseEntry, ClauseOutcome, ClauseValue, HandlerExit};
use crate::error::{FailureSite, Raised, Search};
use crate::eval::logical_value;
use crate::ir::{BodyEngine, NodePath};
use crate::plan::BodyKey;
use crate::trace::{
    Announced, is_whole_number, raised_invalid_trace_letter, raised_numeric_trace_interactive_only,
};
use crate::value::{exact_small_int, within_digits};
use crate::{
    ActiveCondition, CallContext, Code, Failure, InstalledRoutine, Interp, Loud, Novalue,
    PendingTrap, VarHome,
};
use rexx_core::{
    BehaviourId, Body, Decoded, FrameId, ObjRef, RegFrame, ScopePools, SlotFrame, VarRefHome,
};
use rexx_num::{ArithError, CompareOp, Form, Number, SettingsError, compare_decoded};
use rexx_parse::{
    CodeBody, ConditionTrap, ControlExpr, DirectiveKind, EndStyle, Expr, ExprKind, Forward,
    Fragment, Guard, Instruction, InstructionKind, Loop, LoopConditional, LoopKind, NumericSetting,
    ProgramSource, Raise, SymbolId, Trace, Use, UseTarget, VariableRef, parse_interpret,
};
use std::borrow::Cow;
use std::rc::Rc;

// Each clause's static nesting indent, and the printed indent built on it.
mod indent;
pub(crate) use indent::{all_indents, static_indent};

// The `Raised` constructors for the conditions the instruction loop raises.
mod raised;
use raised::{
    raise_syntax_condition, raised_digit_led, raised_dot_led, raised_for_count_not_whole,
    raised_guard_not_logical, raised_iterate_no_loop, raised_iterate_no_match,
    raised_iterate_wrong_kind, raised_leave_no_loop, raised_leave_no_match,
    raised_naming_the_operand, raised_repetition_count_not_whole, raised_select_no_when,
    raised_symbol_expected, raised_until_not_logical, raised_while_not_logical,
};
pub(crate) use raised::{
    raised_for_code, raised_from_settings, raised_if_not_logical, raised_when_not_logical,
};

// `SELECT`'s own clauses, and where `IF`, `WHEN` and `SELECT` send control.
mod select;
use select::skip_else;
pub(crate) use select::{
    Absorbed, SelectEscape, SelectResume, absorb, if_targets, otherwise_range, otherwise_resume,
    select_escape, select_parts, when_resume, when_targets,
};

// `INTERPRET` fragments, `DROP`, and the interactive debug pause.
mod interpret;

// `ADDRESS`, `NUMERIC` and `TRACE`, and the `>I>`/`<I<` invocation lines.
mod settings;

// Condition traps and their delivery, `RAISE`, and `SIGNAL`.
mod condition;

// `CALL` and function calls: resolving the name, running the arguments, entering the callee.
mod call;
pub(crate) use call::{CallEntry, CallResolution, MAX_ACTIVATION_DEPTH};
#[cfg(test)]
use call::{Entered, entered_receiver};

// `DO`/`LOOP`: the header, each kind of repetition, and the flat-loop path the op driver takes.
mod loops;
#[cfg(test)]
use loops::numeric_less;
pub(crate) use loops::{
    FlatLoop, FlatStart, FlatStep, HeaderPlan, HeaderRole, LoopHeaderValues, loop_header_plan,
    loop_header_slot,
};

/// Where control goes after one instruction (the design's "Control flow").
pub(crate) enum Flow {
    Next,
    /// Live since Task 10: `If`/`Select` each resolve to one `Goto` that
    /// skips straight to their construct's true resume point, and
    /// `run_bounded`'s own internal loop applies one whenever a nested
    /// construct's target lands inside its range.
    Goto(usize),
    Exit(Option<ObjRef>),
    /// `RETURN`, with the expression's value or `None` for the bare form.
    /// Live since Task 3.
    Return(Option<ObjRef>),
    /// `LEAVE`, bare (`None`) or by name. Live since Task 11.
    Leave(Option<SymbolId>, Box<LeaveOrigin>),
    /// `ITERATE`, bare or by name. See `Leave`'s own doc comment; the two
    /// variants are handled by nearly identical logic in `Do`/`Select`'s own
    /// arms, differing only in which of the oracle's measured asymmetries
    /// applies (`Select` never consumes a bare `Iterate` at all, and a named
    /// one that matches its own label but is not a loop is 28.5, not simply
    /// "not mine, keep looking").
    Iterate(Option<SymbolId>, Box<LeaveOrigin>),
    /// `SIGNAL label` and `SIGNAL VALUE`, once the target resolves to an
    /// instruction index.
    Signal(usize),
}

/// How one activation finished, which is not the same question as what value
/// it produced.
pub(crate) enum Ended {
    /// `RETURN`: the caller resumes at its next clause, with this value in
    /// `RESULT`.
    Returned(Option<ObjRef>),
    /// `EXIT`, **or the body running out of instructions.** The whole program
    /// stops. Falling off the end belongs here rather than with `Returned`
    /// and that is measured, not assumed: a callee whose label is the last
    /// thing in the file ends the program -- `trace r` / `call sub` / `say
    /// 'after'` / `exit` / `sub:` / `hh = 1` echoes the callee's clauses, then
    /// stops at rc 0 with `after` neither printed nor echoed.
    Exited(Option<ObjRef>),
}

impl Ended {
    /// The value, whichever way the activation finished -- what the *top*
    /// level wants, where the distinction carries no information.
    pub(crate) fn value(self) -> Option<ObjRef> {
        match self {
            Ended::Returned(value) | Ended::Exited(value) => value,
        }
    }
}

/// Who emits a stepped clause's own `*-*` line
/// ([`crate::ir::Op::Clause`]).
pub(crate) enum Echo {
    /// The clause unit asks [`Interp::tracing_clause`] and echoes if the
    /// answer is yes: a clause whose chunk was compiled under a setting that
    /// is no longer in force.
    Gated,
    /// The clause unit emits nothing: this clause's chunk already decided,
    /// and carries the decision as [`crate::ir::Op::TraceClause`] or as the
    /// absence of it.
    Compiled,
}

/// A stepped clause that is open: [`Interp::enter_stepped_clause`] makes one
/// and [`Interp::leave_stepped_clause`] spends it.
#[must_use]
pub(crate) struct SteppedClause {
    /// The clause boundary this entry opened.
    entry: ClauseEntry,
    /// The GC temps frame the clause's own work pushes into.
    frame: FrameId,
    /// `RootSet::temps_len` as the clause was opened.
    temps_at_entry: usize,
}

/// What a called name resolved to: a label or a builtin before the arguments
/// run, anything else after them.
#[derive(Clone, Copy)]
pub(crate) enum Resolved {
    /// A label in the *running activation's* body, at this instruction index.
    Label(usize),
    /// A builtin function name, and **which** builtin -- a row index, not a
    /// copy of the row, so nothing here can drift from the arity check and
    /// the code that live on that row together.
    Builtin(crate::builtin::BuiltinTarget),
    /// A `::ROUTINE` this program installed. `InstalledRoutine::directive` is
    /// the same integer `Activation::body` and `BodyKey::directive` carry.
    Routine(InstalledRoutine),
    /// One of the interpreter's own embedded `.orx` sources, named by a
    /// `CALL` inside the library bootstrap -- `CoreClasses.orx:122` and
    /// `:124`.
    Library(&'static rexx_lib::Program),
    /// A routine of the `REXX` or `REXXUTIL` package that this crate has a
    /// body for. Like a builtin it runs no activation; unlike one it is not in
    /// `BuiltinFunctions.cpp`'s table, which is why its argument errors are
    /// the native-routine family.
    Internal(&'static crate::internal_routines::InternalRoutine),
    /// A routine a loaded library exports, as the index of its
    /// `Interp::package_routines` slot. Like [`Resolved::Internal`] it runs
    /// no activation.
    LibraryRoutine(usize),
    /// A library routine a `::REQUIRES ... LIBRARY` merged into the running
    /// package's own lookup, as its `Interp::library_codes` row: found where a
    /// `::ROUTINE` is, before the security manager is asked, and run under the
    /// name as the call wrote it.
    MergedLibraryRoutine(usize),
    /// An external Rexx file the search found for this name, entered the way
    /// [`Resolved::Library`] is: a whole program with its own directives and
    /// its own `ProgramId`.
    ///
    /// **It carries no path**, and the consumer searches again to recover it.
    /// A `Resolved` is kept per call site in a `Cell`, so it has to be
    /// `Copy`, and the resolver is `&self` and can neither intern a path nor
    /// hand out an index. The second search is stats only, against a call that
    /// re-reads and re-parses the file anyway.
    External,
    /// Nothing answers this name. **Not an error here**: the security
    /// manager's `CALL` checkpoint runs before 43.1 is reported
    /// (`RexxActivation::externalCall`, `execution/RexxActivation.cpp:3077`
    /// against `:3102`), and it needs the arguments. The raise happens where
    /// the checkpoint declines.
    Unresolved,
}

impl Resolved {
    /// Whether a call site keeps this resolution only until
    /// `Interp::routine_generation` moves, rather than for as long as the
    /// site's compiled body lives.
    ///
    /// The oracle hands a routine back to the instruction from
    /// `externalCall`'s Step 2 alone (`findRoutine`,
    /// `execution/RexxActivation.cpp:3062-3070`), so a `::ROUTINE` or an
    /// imported routine stays at the instruction, a label and a builtin are
    /// fixed when the clause resolves, and everything later in the search (the
    /// REXX package's routines, a registered library routine, an external
    /// file) is looked up again on every call. Those are the ones answered
    /// here, used again only while the generation has not moved; it is one
    /// counter for the interpreter, so a routine-table write in any package
    /// or any library registering routines drops all of them.
    pub(crate) fn kept_until_routines_change(self) -> bool {
        matches!(
            self,
            Resolved::Library(_)
                | Resolved::Internal(_)
                | Resolved::LibraryRoutine(_)
                | Resolved::External
        )
    }
}

/// Where a `LEAVE`/`ITERATE` instruction itself sits, captured the instant
/// it steps rather than reconstructed later -- see `Flow::Leave`'s own doc
/// comment for why eagerly.
pub(crate) struct LeaveOrigin {
    /// `None` only when `source` was `None` at the moment this instruction
    /// stepped, which **no caller produces**: `run_fragment` passes its own
    /// fragment source, so a `LEAVE`/`ITERATE` inside fragment text resolves a
    /// real site and becomes the report's innermost echo. See
    /// `Interp::clause_site`.
    site: Option<(usize, Vec<u8>)>,
    indent: usize,
    /// This `LEAVE`/`ITERATE` clause's own line, captured the same way and at
    /// the same moment as `site` and `indent`, and for the same reason.
    clause_line: usize,
}

/// What `eval_condition` should do with the value it just computed, beyond
/// answering the caller's `bool` -- a caller-chosen variant rather than a
/// decision `eval_condition` makes on its own, because the same function
/// serves `IF`/`WHEN` (their own `>>>`, measured) and `WHILE`/`UNTIL`
/// (their own `>K>` instead, never a bare `>>>` alongside it, also
/// measured) and the two are genuinely different oracle behaviours, not
/// two spellings of one.
pub(crate) enum ConditionTrace<'a> {
    /// `IF`/`WHEN`'s own `>>>`.
    Result(usize),
    /// `WHILE`/`UNTIL`'s own `>K>`, tagged `"WHILE"`/`"UNTIL"`.
    Keyword(usize, &'a str),
}

/// Which keyword ended the activation, for [`Interp::returned_value`].
#[derive(Clone, Copy)]
enum Conversion {
    /// An array answers itself.
    Array,
    /// `StringUtil::makearray` over the value's own text.
    Lines,
    /// `StemClass::tailArray`.
    Tails,
    /// `TheNilObject`, which `FORWARD` reports as 98.946.
    Refused,
    /// A conversion this crate does not build. The string is the noun
    /// [`Loud::object_position`] puts in the refusal.
    NotBuilt(&'static str),
}

/// `StringUtil::makearray` with the default separator
/// (`classes/support/StringUtil.cpp:545`-`:638`): a piece per line end, one
/// `\r` dropped from a piece that ends in one, and a trailing piece only
/// where the text does not end at a separator.
fn makearray_lines(text: &[u8]) -> Vec<&[u8]> {
    let mut pieces = Vec::new();
    let mut start = 0;
    while let Some(offset) = text[start..].iter().position(|byte| *byte == b'\n') {
        let separator = start + offset;
        let mut end = separator;
        if end > start && text[end - 1] == b'\r' {
            end -= 1;
        }
        pieces.push(&text[start..end]);
        start = separator + 1;
    }
    if start < text.len() {
        pieces.push(&text[start..]);
    }
    pieces
}

#[derive(Clone, Copy)]
pub(crate) enum ReturnKeyword {
    Return,
    Exit,
}

/// Which end of the queue a line lands on, for [`Interp::queue_evaluated`].
#[derive(Clone, Copy)]
pub(crate) enum QueueKeyword {
    Push,
    Queue,
}

impl Interp {
    // ---- the instruction loop, which is what this spike is for ----

    /// Runs the current activation's body to completion.
    /// ```text
    /// fn run_activation_wrong(&mut self) -> Result<Option<ObjRef>, Loud> {
    ///     let body = &self.activations.last().expect("a live activation").program.main;
    ///     while let Some(instruction) = body.instructions.get(self.activation().pc) {
    ///         self.step_wrong(body, instruction)?;
    ///     }
    ///     Ok(None)
    /// }
    /// ```
    /// ```text
    /// error[E0502]: cannot borrow `*self` as mutable because it is also borrowed as immutable
    ///    --> crates/rexx-exec/src/lib.rs:851:13
    ///     |
    /// 849 |         let body = &self.activations.last().expect("a live activation").program.main;
    ///     |                     ---------------- immutable borrow occurs here
    /// 850 |         while let Some(instruction) = body.instructions.get(self.activation().pc) {
    ///     |                                       ----------------- immutable borrow later used here
    /// 851 |             self.step_wrong(body, instruction)?;
    ///     |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ mutable borrow occurs here
    /// ```
    /// ```
    /// use std::rc::Rc;
    /// struct CodeBody { instructions: Vec<u32> }
    /// struct Frame { program: Rc<CodeBody>, pc: usize }
    /// struct Interp { activations: Vec<Frame> }
    /// impl Interp {
    ///     fn step(&mut self, _instruction: &u32) {}
    ///     fn run(&mut self) {
    ///         let program = Rc::clone(&self.activations.last().unwrap().program);
    ///         let body = &program.instructions;
    ///         while let Some(instruction) = body.get(self.activations.last().unwrap().pc) {
    ///             self.step(instruction);
    ///             self.activations.last_mut().unwrap().pc += 1;
    ///         }
    ///     }
    /// }
    /// ```
    /// ```compile_fail
    /// use std::rc::Rc;
    /// struct CodeBody { instructions: Vec<u32> }
    /// struct Frame { program: Rc<CodeBody>, pc: usize }
    /// struct Interp { activations: Vec<Frame> }
    /// impl Interp {
    ///     fn step(&mut self, _instruction: &u32) {}
    ///     fn run(&mut self) {
    ///         let body = &self.activations.last().unwrap().program.instructions;
    ///         while let Some(instruction) = body.get(self.activations.last().unwrap().pc) {
    ///             self.step(instruction);
    ///             self.activations.last_mut().unwrap().pc += 1;
    ///         }
    ///     }
    /// }
    /// ```
    pub(crate) fn run_activation(&mut self) -> Result<Ended, Failure> {
        // `code` is bound to the activation on top of the stack at entry,
        // while every `pc` read and write below goes to whatever is on top
        // *now*. Those are the same frame only because `step` leaves the
        // activation stack as it found it -- true for a fragment, which runs
        // inside the creating activation rather than pushing its own, and
        // true for a `CALL` only because the `Call` arm pops the callee
        // before it returns.
        let arguments = Rc::clone(&self.call_context.arguments);
        // **The name is copied only where nothing else records it.** A
        // method activation already carries the message name it was entered
        // under on `Activation::method_identity`, and that is the hot path:
        // measured, `instructions:u` on `bench-programs/dispatch.rex`, one
        // no-argument send per iteration, +1.732% against BASE with an
        // `Rc<[u8]>` built here for every send and +0.093% with this test in
        // front of it.
        let name = (self.activation().entry != crate::activation::Entry::Method)
            .then(|| Rc::from(&self.call_context.name[..]));
        let activation = self.activation_mut();
        if let Some(name) = name {
            activation.call_name = Some(name);
        }
        activation.call_arguments = Some(arguments);
        let program = Rc::clone(&self.activation().program);
        let plan = Rc::clone(&self.activation().plan);
        let selector = self.activation().body;
        // A selector that resolves to nothing is an internal inconsistency
        // and not a program error: it can only be built by a resolution step
        // that already looked the body up. Loud rather than a panic, matching
        // this crate's standing rule -- an abort is precisely the outcome
        // that rule exists to exclude.
        let Some(body) = body_of(&program, selector) else {
            return Err(Loud::missing_body().into());
        };
        let code = Code {
            body,
            symbols: &program.symbols,
            slots: &plan.by_symbol,
            plan: Some(&plan),
        };
        // The key this body's plan was cached under, and the key its chunk is
        // cached under. **They have to be the same key**, and nothing but this
        // says so: `plan` is read off the activation while `key` is rebuilt
        // from the activation's program id and body selector, so a push that
        // paired a plan with the wrong id would put a body's chunk under
        // another body's name -- a wrong answer rather than a miss, exactly
        // what `BodyKey::directive`'s own doc says about the selector.
        let key = self.activation().body_key();
        debug_assert!(
            self.plans
                .get(&key)
                .is_some_and(|cached| Rc::ptr_eq(cached, &plan)),
            "the running activation's body key does not name the plan it is running with, so \
             its chunk would be cached under another body's name"
        );

        // **Every activation's body runs from a compiled chunk.** There is
        // no second engine and no selection left to make: `run_activation` is
        // the one function that runs a body, and this is where the stream is
        // entered.
        let Some(chunk) = self.chunk_for(key, self.chunk_trace(), body, &plan) else {
            return Err(Loud::chunk_refused().into());
        };
        self.run_chunk(&code, &chunk, Some(&program.source))
    }

    /// Transfers "is this the first instruction executed in this activation"
    /// to the step about to run, which `PROCEDURE` and `USE LOCAL` are the
    /// only readers of.
    pub(crate) fn grant_procedure_permission(&mut self, instruction: &Instruction) {
        if !matches!(instruction.kind, InstructionKind::Label { .. }) {
            self.procedure_permitted =
                std::mem::take(&mut self.activation_mut().first_instruction_pending);
        }
    }

    /// Applies one clause's `Flow` to this activation.
    pub(crate) fn apply_flow(
        &mut self,
        code: &Code<'_>,
        flow: Flow,
    ) -> Result<Option<Ended>, Failure> {
        match flow {
            Flow::Next => self.activation_mut().pc += 1,
            Flow::Goto(target) => self.activation_mut().pc = target,
            // `SIGNAL`, once its target has escaped every nested construct
            // and every `INTERPRET` fragment it fired from (`Flow::Signal`'s
            // own doc comment has why it cannot ride `Goto` to get here).
            // The only consumer, matching `Goto`'s own arm exactly: `target`
            // already resolved against this activation's own body
            // (`resolve_signal_target`), which is exactly the body `code` is
            // bound to.
            Flow::Signal(target) => {
                self.activation_mut().pc = target;
                self.activation_indent = 0;
                self.indent_offset = 0;
            }
            // **The one place an activation's value stops being a clause's
            // temporary**, which is why the root that outlives the temps
            // stack is taken here rather than at each of the half-dozen
            // constructs that can produce one. `EXIT`, a top-level `RETURN`,
            // a `RAISE` with an `EXIT` tail and a handler's own exit all
            // arrive as one of these two variants; every one of them can end
            // up as the value `execute` hands `exit_code_for`, and by then
            // the frame that rooted it has been popped. See
            // [`Interp::root_exit_value`] for the measurement.
            Flow::Exit(value) => {
                if let Some(value) = value {
                    self.root_exit_value(value);
                }
                return Ok(Some(Ended::Exited(value)));
            }
            // The activation boundary `Flow::Return` was added to reach.
            // Every construct between the `RETURN` and here forwarded it
            // untouched; this is the one consumer.
            Flow::Return(value) => {
                if let Some(value) = value {
                    self.root_exit_value(value);
                }
                return Ok(Some(Ended::Returned(value)));
            }
            // Task 11: a `LEAVE`/`ITERATE` that reached the very top of the
            // program -- nothing anywhere, at any nesting depth, ever
            // matched it. This is the exhausted-search family, 28.1 (bare
            // `LEAVE`)/28.2 (bare `ITERATE`)/28.3 (named `LEAVE`)/28.4
            // (named `ITERATE`). `origin.indent` already holds this family's
            // own answer by the time it gets here -- every `Select`/`Do`
            // frame the search walked through on the way up has already
            // reset it to its own `static_indent` as it forwarded past
            // (`LeaveOrigin`'s own doc comment has the rule, corrected after
            // review: it is **not** always zero, only when every popped
            // frame along the way happened to sit at top level). 28.5 (a
            // named `ITERATE` that *did* match something, just not a loop)
            // is a different family, raised where the match was found, in
            // `Select`/`Do`'s own arms, and never reaches here.
            Flow::Leave(name, origin) => {
                self.record_leave_failure(&origin);
                let raised = match name {
                    None => raised_leave_no_loop(),
                    Some(n) => raised_leave_no_match(code.symbols.name(n).as_bytes()),
                };
                return Err(raised.into());
            }
            Flow::Iterate(name, origin) => {
                self.record_leave_failure(&origin);
                let raised = match name {
                    None => raised_iterate_no_loop(),
                    Some(n) => raised_iterate_no_match(code.symbols.name(n).as_bytes()),
                };
                return Err(raised.into());
            }
        }
        Ok(None)
    }

    /// One instruction's own work, with the clause unit already discharged by
    /// whoever called: [`Interp::step`] takes the permission and enters from
    /// `Op::Clause`'s region, and [`crate::ir::Op::Exec`] enters from inside
    /// the [`crate::ir::Op::Clause`] region that already opened the clause.
    pub(crate) fn exec_instruction(
        &mut self,
        code: &Code<'_>,
        index: usize,
        instruction: &Instruction,
        source: Option<&ProgramSource>,
        first_instruction: bool,
    ) -> Result<Flow, Failure> {
        match &instruction.kind {
            InstructionKind::Say { expression } => {
                let value = match expression {
                    Some(expression) => Some(self.eval(code, expression)?),
                    None => None,
                };
                self.say_evaluated(value)?;
                Ok(Flow::Next)
            }

            InstructionKind::Assignment { target, value } => {
                let value = self.eval(code, value)?;
                // `None`: this engine resolves nothing ahead of time, so the
                // write resolves its own slot the way it always has.
                self.assign_evaluated(code, target, value, None)?;
                Ok(Flow::Next)
            }

            // A simple variable back to unset (`Interp::clear_variable`, added
            // expressly for this and never `ObjRef::NIL`, which is a value
            // and not an absence -- `x = .nil; say x` and `y = .nil; drop y;
            // say y` render differently, measured in `drop_variable`'s own
            // doc comment), a whole stem, one tail, or the `(v)` indirect
            // form. See `drop_variable`.
            InstructionKind::Drop { variables } => {
                for variable in variables {
                    self.drop_variable(code, variable)?;
                }
                Ok(Flow::Next)
            }

            // `NUMERIC DIGITS`/`FUZZ`/`FORM`, every spelling `NumericSetting`
            // has. See `exec_numeric`.
            InstructionKind::Numeric {
                setting,
                expression,
            } => {
                self.exec_numeric(code, setting, expression)?;
                Ok(Flow::Next)
            }

            // `TRACE` (D17): sets the running activation's own trace mode, or
            // raises 24.901 for
            // the interactive-only skip-count forms. See `exec_trace`.
            InstructionKind::Trace(setting) => {
                self.exec_trace(code, setting)?;
                Ok(Flow::Next)
            }

            // `EXIT`, bare or with a result: the spike had only the bare form
            // (`expression: None` matched literally, nothing else reaching
            // this arm at all). The value crosses out of the instruction loop
            // as `Flow::Exit`, unconverted -- `Interp::exit_code_for` (`lib.rs`)
            // is what turns it into a process exit code, and it runs once in
            // `execute` rather than here, because a `Flow::Exit` can also
            // come from inside a fragment (`run_fragment`'s own propagating
            // arm), and the conversion needs nothing this loop knows
            // that `execute` does not already have.
            InstructionKind::Exit { expression } => {
                let value = match expression {
                    Some(expression) => Some(self.eval(code, expression)?),
                    None => None,
                };
                self.returned_value(value, ReturnKeyword::Exit)
            }

            // A label is a traced no-op: the C++'s own `execute` on a label
            // instruction only traces it (Task 13's own construct -- a
            // `Label` clause is echoed here via `Op::Clause`'s region, same
            // as any other instruction) and does nothing else besides.
            // `SIGNAL`/`CALL` reach a label by jumping to the instruction
            // after it; nothing ever executes the label node for its own
            // effect.
            InstructionKind::Label { .. } => Ok(Flow::Next),

            InstructionKind::Nop => Ok(Flow::Next),

            // `INTERPRET expr`: evaluate to a string, parse it as a fragment,
            // run it against **this** activation, through `run_fragment`. The
            // arm is thin because `run_fragment` is where the work is.
            InstructionKind::Interpret { expression } => {
                let value = self.eval(code, expression)?;
                self.roots.push_temp(value);
                // `RexxInstructionExpression::evaluateStringExpression`
                // (`instructions/RexxInstruction.cpp:257`), the one
                // `requestString` every instruction that evaluates a single
                // string expression shares -- so the `>>>` below traces the
                // conversion, measured: `trace r` over `interpret .K` with a
                // class-side `makeString` returning `'nop'` prints
                // `>>>   "nop"` and then the fragment's own `*-* nop`.
                let value = self.required_string_value(value)?;
                let text = self.to_text(value).to_vec();
                // `>>>` on the interpreted text itself, before the fragment
                // runs -- the same `trace_result` every other value-producing
                // arm calls (`Say`, `Assignment`), at the same
                // `current_value_indent`. Review finding I1(a): the arm
                // shipped without this and was the only value-producing arm in
                // the crate that traced nothing. Measured (`trace r`, `zz =
                // 'nop'`, `interpret zz`), the oracle prints
                // ```text
                //      3 *-* interpret zz
                //        >>>   "nop"
                //      3 *-* nop
                // ```
                self.trace_result(self.clause_state.current_value_indent, &text);
                // **The fragment's level, with delta 0.** Measured: a
                // fragment's clauses print at the enclosing `INTERPRET`
                // clause's own absolute indent plus whatever nests them
                // *inside* the fragment -- `interpret "do jj = 1 to 1; say 2
                // & 1; end"` at top level echoes the inner clause at 2 and
                // the `INTERPRET` at 0, and the identical fragment two `DO`s
                // deep echoes them at 6 and 4. So the base is the enclosing
                // clause's printed indent exactly, with no bump of its own --
                // unlike a called routine's, which the same measurements put
                // two spaces further in (`call sub1` at printed indent 4 into
                // a flat routine echoes the callee's clause at 6), and which
                // is Task 3's to add.
                let base_indent = self.clause_state.current_value_indent;
                let base_line = self.clause_site(source, instruction).map(|(line, _)| line);
                let saved_base = std::mem::replace(&mut self.activation_indent, base_indent);
                let saved_offset = std::mem::take(&mut self.indent_offset);
                let saved_line = std::mem::replace(&mut self.clause_line_override, base_line);
                // **The fragment's own condition queue**, which the depth is
                // the key to rather than a second collection --
                // `Interp::fragment_depth` has why the oracle has one and what
                // was measured on either side of it. Incremented rather than
                // replaced the way the values saved above are, because a
                // nested fragment inherits each of those and needs a level of
                // its own here.
                self.fragment_depth += 1;
                // `saved_line` read before the replace above is also the
                // answer to "is a fragment already running", which is the one
                // extra thing `enter_fragment` needs and the only place it is
                // in hand. Nothing new is tracked for it.
                let saved_entry = self.enter_fragment(saved_line.is_some());
                let flow = self.run_fragment(text);
                // **A condition still queued at this depth dies with the
                // fragment**, on the failing path as well as this one, because
                // the activation whose queue it was in is what ends. Measured,
                // `interpret 'zq = raiser()'` with a handler that requeues: the
                // oracle runs the first handler and never runs the second, and
                // the enclosing clause's own boundary does not pick it up.
                // Dropping the entries also keeps a program that runs
                // `INTERPRET` in a loop from accumulating undeliverable ones.
                let depth = self.fragment_depth;
                self.pending_traps
                    .retain(|pending| pending.fragment_depth != depth);
                // **Nothing deeper than the fragment just left may survive
                // it**, which is the invariant that lets the delivery key be
                // an equality rather than a comparison: a deeper entry would
                // belong to a fragment that already ran this same discard on
                // its own way out, and a boundary out here would then have to
                // decide whether it inherits one. Asserted rather than
                // reasoned about, because the discard runs inside a
                // `deliver_pending_traps` that a handler can have re-entered.
                debug_assert!(
                    self.pending_traps
                        .iter()
                        .all(|pending| pending.fragment_depth < depth),
                    "a condition queued inside a fragment outlived that fragment's own exit"
                );
                self.fragment_depth -= 1;
                self.leave_fragment(saved_entry);
                self.activation_indent = saved_base;
                self.indent_offset = saved_offset;
                self.clause_line_override = saved_line;
                flow
            }

            // `IF`/`THEN`/`ELSE`. This arm resolves the whole construct
            // itself rather than leaving the outer loop to fall through the
            // flat list -- see `run_bounded`'s doc comment for why that is
            // not optional. `false_target`'s own doc comment: "the ELSE if
            // there is one, otherwise the instruction after the THEN
            // branch" -- confirmed by tracing `block.rs` by hand, it is the
            // `Else` instruction's own index when there is one, landing
            // *on* it rather than past it.

            // A pure marker: only ever reached inside `If`'s own bounded
            // sub-loop (the true branch, right after the `IF`) or via
            // ordinary fallthrough on the false path. Never independently
            // dispatched for a decision of its own.
            InstructionKind::Then => Ok(Flow::Next),

            // Also a pure marker (`ast.rs`'s own doc comment: "executing an
            // ELSE only traces"). Reached only by ordinary fallthrough on
            // the false path -- the true path's `Goto` in the `If` arm above
            // skips straight past it to `then_exit`, so this is never asked
            // to decide anything.
            InstructionKind::Else { .. } => Ok(Flow::Next),

            // `SELECT`/`SELECT CASE`. Evaluates `case` at most once (if this
            // is a `SELECT CASE`), then tests each of its own *listed*
            // `whens` in source order by reading the `When`/`WhenCase` node
            // directly as data (`condition`/`values`, `false_target`,
            // `exit`) rather than dispatching through `Op::Clause`'s region
            // -- a *listed* `When`/`WhenCase` node (one collected into this
            // `whens` list, `ast.rs`'s own doc comment) must never be
            // independently stepped for a decision of its own, only ever
            // run past inside a bounded sub-loop. An *absorbed* one (never
            // collected here at all, because it is itself another `When`/
            // `WhenCase`'s own `THEN`) is the exception, and is
            // independently stepped -- see the `When`/`WhenCase` arm,
            // below, for both halves.

            // **Fixed after review: this used to be a bare `Ok(Flow::Next)`,
            // and that was a silently wrong answer, not a formatting gap.**
            // A `When`/`WhenCase` is only ever reached here through the
            // absorbed-`WHEN` shape -- a `WHEN` whose own `THEN` consequence
            // is itself a `WHEN`/`WHEN CASE` clause, which the enclosing
            // `SELECT`'s own `whens` never collects (`ast.rs`'s own doc
            // comment on `whens`, `LanguageParser.cpp:1319`) -- since a
            // *listed* `When`/`WhenCase` is always fully handled by
            // `Select`'s own explicit arm, above, without ever calling
            // `step` on itself (its own body range never contains another
            // listed sibling's index).
            InstructionKind::When { condition, .. } => {
                self.eval_condition(
                    code,
                    condition,
                    ConditionTrace::Result(self.clause_state.current_value_indent),
                    raised_when_not_logical,
                )?;
                Ok(Flow::Next)
            }
            // `SELECT CASE`'s own absorbed form.
            InstructionKind::WhenCase {
                values,
                false_target,
                ..
            } => match self.current_case_text.clone() {
                Some(case_text) => {
                    let indent = self.clause_state.current_value_indent;
                    if self.test_case_when(code, values, &case_text, indent)? {
                        Ok(Flow::Next)
                    } else {
                        // **Corrected after a second re-verification found
                        // the first version of this line wrong under
                        // nesting.** `current_value_indent.saturating_sub
                        // (2)` (this line's own first attempt) gave the
                        // right answer at the top level by coincidence
                        // (`6 - 2 = 4`) and the *wrong* one nested one `DO`
                        // deeper (`8 - 2 = 6`, where the oracle still wants
                        // `4`) -- measured directly (`t13_f3_nested.rex`,
                        // then a `TRACE R` transcript one level deeper
                        // again, `i_trace_nested_escape.rex`/`j_trace_
                        // nested_otherwise.rex`, this task's report has
                        // all three). The offset is the **constant** `4`,
                        // not a function of how deep the absorbed
                        // condition itself sits: it is exactly two
                        // `indent()` bumps -- the enclosing, listed
                        // `WHEN`/`WHEN CASE`'s own marker, then its own
                        // body entry -- past wherever an *ordinary* `SELECT`
                        // -level construct (`END`, `OTHERWISE`) would sit,
                        // and that gap is the same two bumps regardless of
                        // how many other constructs enclose the whole
                        // `SELECT`. Confirmed at both nesting depths for
                        // all three landing shapes (`END`, `OTHERWISE`'s
                        // own marker, `OTHERWISE`'s own body) before
                        // trusting it a second time.
                        self.indent_offset = 4;
                        Ok(Flow::Goto(
                            false_target.unwrap_or(code.body.instructions.len()),
                        ))
                    }
                }
                None => {
                    for value in values {
                        let v = self.eval(code, value)?;
                        self.roots.push_temp(v);
                    }
                    Ok(Flow::Next)
                }
            },
            InstructionKind::Otherwise => Ok(Flow::Next),

            // `DO`/`LOOP`, every kind but `DO WITH` (the loud path,
            // `run_loop`'s own doc comment) -- Task 11. Resolves the whole
            // construct itself, every iteration, exactly the discipline
            // `If`/`Select` already established: see `Flow::Leave`'s own
            // doc comment for why `Do`'s own arm never returns until the
            // entire loop is over, one way or another.
            // `DO`/`LOOP`, `IF` and `SELECT` are **not** reachable here.
            InstructionKind::Do(_)
            | InstructionKind::Loop(_)
            | InstructionKind::If { .. }
            | InstructionKind::Select { .. } => Err(Loud::instruction(&instruction.kind).into()),

            // `LEAVE`/`ITERATE`, bare or by name -- Task 11. Resolves to
            // data, not a failure (`Flow::Leave`'s own doc comment): whether
            // this instruction's own name matches anything is answered by
            // whichever `Do`/`Select` (or `run_activation`'s own top level,
            // if none does) inspects the `Flow` this returns, never here.
            InstructionKind::Leave { name } => Ok(Flow::Leave(
                *name,
                Box::new(self.leave_origin(code, index, source, instruction)),
            )),
            InstructionKind::Iterate { name } => Ok(Flow::Iterate(
                *name,
                Box::new(self.leave_origin(code, index, source, instruction)),
            )),

            // `END`. `Select`'s two non-7.3 closings (`OTHERWISE` present)
            // are reached only by that `OTHERWISE`'s own ordinary body
            // fallthrough and do nothing. `EndStyle::Select`'s own doc
            // comment: "Reaching this END at run time is error 7.3, because
            // every WHEN was false" -- the ordinary way to land here, but
            // **not the only one since F3**: an absorbed `WhenCase`'s own
            // false-branch escape (`InstructionKind::WhenCase`'s own arm,
            // above) can also `Goto` straight onto this exact instruction,
            // carrying its own residual indent along in `pending_escape_
            // indent` for this arm's own 7.3 to be reported at rather than
            // this position's ordinary `static_indent`. An earlier version
            // of this comment said `Select`'s own arm "sends every other
            // path around this instruction entirely," which was true of
            // every path *it* controls directly and false of this one,
            // which escapes through it rather than being dispatched by it.
            InstructionKind::End { closes, .. } => {
                let closes = closes
                    .as_ref()
                    .expect("an End's closes is only None while its body is still being assembled");
                match closes.style {
                    EndStyle::Select => Err(raised_select_no_when().into()),
                    EndStyle::Otherwise
                    | EndStyle::LabeledOtherwise
                    | EndStyle::Do
                    | EndStyle::LabeledDo
                    | EndStyle::Loop => Ok(Flow::Next),
                }
            }

            // `CALL name`, `CALL "name"`, `CALL (expr)` and `CALL ON`/`CALL
            // OFF`. The one arm of `rexx_parse::Call` that stays loud keeps
            // its own owner (`instruction_owner`, `lib.rs`): `Qualified`
            // (`CALL ns:name`) is Phase 5's.
            InstructionKind::Call(call) => match &**call {
                // `name` arrives already upcased for the symbol form and
                // verbatim for the quoted one (`rexx-parse`'s own `Call`
                // doc). `literal` inverts into "may this search the label
                // table": measured, `call "SUB"` with `sub:` present is
                // Error 43.1 and not a call, so the quoted form bypasses the
                // search entirely rather than merely matching case-sensitively.
                rexx_parse::Call::Named {
                    name,
                    literal,
                    args,
                } => self.exec_call(code, name, !*literal, args),
                // `CALL (expr)`: the target is evaluated in the caller, its
                // value is traced, and the **verbatim** text is what the
                // label search sees. Both halves are measured and they pull
                // in opposite directions from the quoted form: `nm = 'SUB';
                // call (nm)` runs `sub:`, so this form *does* search labels,
                // while `nm = 'sub'; call (nm)` is Error 43.1 `Could not find
                // routine "sub"`, so the value is not upcased on the way in.
                rexx_parse::Call::Dynamic { target, args } => {
                    let value = self.eval(code, target)?;
                    self.roots.push_temp(value);
                    // `targetName = evaluatedTarget->requestString()`
                    // (`instructions/CallInstruction.cpp:296`), before the
                    // `>>>` below: measured, `trace r` over `call (.K)` with a
                    // class-side `makeString` returning `'MS'` traces
                    // `>>>   "MS"` and runs `MS:`.
                    let value = self.required_string_value(value)?;
                    let name = self.to_text(value).to_vec();
                    // Its own `>>>`, at the `CALL` clause's own indent, which
                    // `Call::Named` has no equivalent of -- measured, `call
                    // sub 1+1, 'q'` under `trace r` traces no value line at
                    // all while `call (nm)` traces one for the target.
                    self.trace_result(self.clause_state.current_value_indent, &name);
                    self.exec_call(code, &name, true, args)
                }
                // `CALL ON cond NAME label` / `CALL OFF cond`. Shares every
                // line of its implementation with `SIGNAL ON`/`OFF` except
                // the one `bool` that decides how the handler runs -- see
                // `exec_condition_trap`, and `Trap`'s own doc comment
                // (`activation.rs`) for the two measured behaviours that
                // `bool` selects between.
                rexx_parse::Call::Trap(trap) => self.exec_condition_trap(trap, true),
                // `CALL ns:name`, whose target is a public routine of that
                // namespace and nothing else --
                // `RexxInstructionQualifiedCall::resolve`
                // (`instructions/CallInstruction.cpp:443`). It settles
                // `RESULT` exactly as `Named` does, which is why it joins the
                // same second half; what it does not do is search labels or
                // builtins, so it has no `search_labels` to pass.
                rexx_parse::Call::Qualified {
                    namespace,
                    name,
                    args,
                } => {
                    let namespace = code.symbols.name(*namespace).as_bytes().to_vec();
                    let name = code.symbols.name(*name).as_bytes().to_vec();
                    let package = self.running_program().ok_or_else(Loud::missing_body)?;
                    let resolution = self.namespace_routine(package, &namespace, &name);
                    let resolved = self.resolved_after_arguments(code, resolution, args)?;
                    self.invoke_named_call(code, CallResolution::Settled(resolved), &name, args)
                }
            },

            // `RETURN`, bare or with a value. Unwinds to the activation
            // boundary; `Flow::Return`'s own doc comment has why none of the
            // other variants expresses that, and why the main body's own
            // `RETURN` ends the program.
            InstructionKind::Return { expression } => {
                let value = match expression {
                    Some(expression) => Some(self.eval(code, expression)?),
                    None => None,
                };
                self.returned_value(value, ReturnKeyword::Return)
            }

            // `SIGNAL label` and `SIGNAL VALUE`. `Signal::Trap` (`SIGNAL
            // ON`/`SIGNAL OFF`) stays loud, Task 7's own owner
            // (`instruction_owner`, `lib.rs`).
            InstructionKind::Signal(signal) => match &**signal {
                // `name` is already upcased for a bare symbol and verbatim
                // for a quoted one (`rexx-parse`'s own `Signal` doc), and
                // **both forms search the label table** -- unlike `CALL
                // "name"`, which never does, because `SIGNAL` has no
                // builtin/external fallback for a literal spelling to
                // deliberately bypass into. Measured: `signal "sub"` with
                // `sub:` present still raises 16.1 (case-sensitive against
                // the label's own upcased spelling, so the lowercase quoted
                // form does not match), while `signal Sub` (bare, mixed
                // case) and `signal "SUB"` both run it.
                rexx_parse::Signal::Label(name) => self.signal_to_label(name),
                // `SIGNAL VALUE expr`. Its own `>K>` -- `"VALUE" => text`, at
                // this clause's own indent with no `+2` the way `WHILE`/
                // `UNTIL` carry (measured one `DO` deep: `signal value
                // target` traces `>K>     "VALUE" => "THERE"` at the same
                // indent as its own clause echo, unlike those two, which are
                // evaluated as part of the *enclosing* `DO`/`LOOP`'s own
                // step). The rendered text is then searched exactly like
                // `Label`'s own bytes, with **no shape check on the value at
                // all** -- measured, a number, an empty string and an
                // ordinary non-label string all raise 16.1 naming that exact
                // text, none of them a different error.
                rexx_parse::Signal::Value(expr) => {
                    let value = self.eval(code, expr)?;
                    // Rooted here rather than inside `signal_to_value`, which
                    // the compiled stream reaches with the value already in a
                    // register: the temp is what roots it across the render
                    // there, and a second one would be a frame this clause
                    // does not own.
                    self.roots.push_temp(value);
                    self.signal_to_value(value)
                }
                // `SIGNAL ON cond NAME label` / `SIGNAL OFF cond`. Unlike the
                // two arms above it transfers no control of its own: it edits
                // this activation's trap table and falls through to the next
                // clause, and the transfer happens later, if the condition is
                // ever raised.
                rexx_parse::Signal::Trap(trap) => self.exec_condition_trap(trap, false),
            },

            // `PROCEDURE`, bare or with an `EXPOSE` list (D9r). Isolates the
            // callee's variable pool and aliases the exposed names back into
            // the pool they came from. See `exec_procedure`.
            InstructionKind::Procedure { variables } => {
                self.exec_procedure(code, variables, first_instruction)?;
                Ok(Flow::Next)
            }

            // `EXPOSE`: binds this method's names to the receiving object's
            // pool for the scope the method was declared in. See
            // `exec_expose`.
            InstructionKind::Expose { variables } => {
                self.exec_expose(code, variables)?;
                Ok(Flow::Next)
            }

            // `USE ARG`/`USE STRICT ARG`/`USE LOCAL`. See `exec_use`.
            InstructionKind::Use(use_) => {
                self.exec_use(code, use_, first_instruction)?;
                Ok(Flow::Next)
            }

            // `RAISE`, in all of its forms. See `exec_raise`, whose doc
            // comment carries the delivery table -- which is the whole of
            // this instruction and is not derivable from the grammar.
            InstructionKind::Raise(raise) => self.exec_raise(code, raise),

            // `PUSH`/`QUEUE line` (I15). One arm, not two copies that can
            // drift (review round 1's M4): the two spellings differ only in
            // which end of the queue the value lands on, decided below by
            // which variant matched. The rendering, the `>>>` line and the
            // write are `Interp::queue_evaluated`'s, shared with
            // `crate::ir::Op::Queue`.
            InstructionKind::Push { expression } | InstructionKind::Queue { expression } => {
                let value = match expression {
                    Some(expression) => Some(self.eval(code, expression)?),
                    None => None,
                };
                let keyword = if matches!(instruction.kind, InstructionKind::Push { .. }) {
                    QueueKeyword::Push
                } else {
                    QueueKeyword::Queue
                };
                self.queue_evaluated(value, keyword)?;
                Ok(Flow::Next)
            }

            // `PARSE`, in every source spelling, plus the two short forms that
            // are the same instruction with `UPPER` already set: `ARG
            // template` is `PARSE UPPER ARG template` and `PULL template` is
            // `PARSE UPPER PULL template`. One arm, because `rexx-parse`
            // builds the identical `Parse` body for all three (its
            // `parse_instruction_body` takes the implied source and sets
            // `upper` from it), so a second arm here would be a second copy of
            // the dispatch and nothing else.
            InstructionKind::Parse(parse)
            | InstructionKind::Arg(parse)
            | InstructionKind::Pull(parse) => {
                self.exec_parse(code, parse, None)?;
                Ok(Flow::Next)
            }

            // `ADDRESS`: the forms that only name an environment -- the
            // constant `ADDRESS env`, the computed `ADDRESS VALUE expr` and
            // the bare toggle -- go to `exec_address`. `ADDRESS env command`
            // runs one command against that name and writes neither half of
            // the activation's pair, measured. `WITH`'s redirection is what
            // is still owed.
            InstructionKind::Address(address) => {
                if let Some(command) = &address.command {
                    let environment = match (&address.environment, &address.dynamic) {
                        (Some(name), _) => name.to_vec(),
                        (None, Some(expression)) => {
                            let value = self.eval(code, expression)?;
                            self.roots.push_temp(value);
                            let value = self.required_string_value(value)?;
                            self.to_text(value).into_owned()
                        }
                        (None, None) => Vec::new(),
                    };
                    // **The configuration is this command's alone.** The
                    // arm that carries a command never calls `setAddress`,
                    // so it stores nothing under the name -- measured, a
                    // later command in the same program is not redirected
                    // and the stem it filled keeps its one line.
                    return self.exec_command(
                        code,
                        instruction,
                        source,
                        command,
                        Some(&environment),
                        address.io.as_deref(),
                    );
                }
                self.exec_address(code, address)?;
                Ok(Flow::Next)
            }

            // A message send as a whole clause: `q~append(1)`, `q~~append(1)`
            // and the message-assignment form `q[1] = 2`. See
            // `exec_message`.
            InstructionKind::Message { term, value } => {
                self.exec_message(code, term, value.as_ref())
            }

            // `GUARD ON`/`GUARD OFF`, with or without a `WHEN`. See
            // `exec_guard`.
            InstructionKind::Guard(guard) => self.exec_guard(code, guard),

            // `REPLY`, bare or with a value. See `exec_reply`.
            InstructionKind::Reply { expression } => {
                self.exec_reply(code, index, expression.as_ref())
            }

            // `FORWARD` and its options. See `exec_forward`.
            InstructionKind::Forward(forward) => self.exec_forward(code, forward),

            // A command clause: the string is evaluated, handed to the
            // `ADDRESS` environment in force, and its return code settles
            // `RC`, `.RS` and any condition. See `command.rs`.
            //
            // The expression is `Option` because `opt_expr` builds it, but a
            // clause reaching the command fallback has a term in it -- an
            // empty one is a null clause and is dropped before this. Measured
            // on the oracle: `;`, a blank line and `;;` each run nothing at
            // all, and the C++'s own `execute` would fault on the null it
            // would need. The assertion is the tripwire rather than a
            // sentence claiming it cannot happen.
            InstructionKind::Command { expression } => {
                debug_assert!(
                    expression.is_some(),
                    "a command clause reached execution with no expression"
                );
                match expression {
                    Some(expression) => {
                        self.exec_command(code, instruction, source, expression, None, None)
                    }
                    None => Ok(Flow::Next),
                }
            }

            other => Err(Loud::instruction(other).into()),
        }
    }

    /// A message send that is a clause of its own
    /// (`RexxInstructionMessage::execute`, `MessageInstruction.cpp:151`).
    pub(crate) fn exec_message(
        &mut self,
        code: &Code<'_>,
        term: &Expr,
        value: Option<&Expr>,
    ) -> Result<Flow, Failure> {
        let ExprKind::Message {
            target,
            name,
            super_class,
            args,
            cascade,
        } = &term.kind
        else {
            // `rexx-parse` builds this variant only from a message term
            // (`instruction.rs`'s `message`), so nothing else can arrive;
            // loud rather than a panic, on the standing rule that a parser
            // guarantee the type system does not carry must not abort.
            return Err(Loud::expression(&term.kind).into());
        };
        // **`message_term` directly rather than through `Interp::eval`**,
        // even for the form that is an ordinary expression: `eval`'s own
        // `ExprKind::Message` arm turns a valueless send into 91.999, which
        // is the expression position's error and not this one's -- measured,
        // a whole-clause `.K~m` on a method ending in a bare `return` is
        // rc 0.
        let mut assigned_name;
        let result = match value {
            None => {
                let probe = 0u8;
                self.enter_eval_node(&raw const probe)?;
                let sent = self.message_term(
                    code,
                    &crate::dispatch::MessageTerm {
                        target,
                        name,
                        super_class: super_class.as_deref(),
                        args,
                        cascade: *cascade,
                        assigned: None,
                    },
                );
                self.depth -= 1;
                sent?
            }
            Some(value) => {
                assigned_name = name.to_vec();
                assigned_name.push(b'=');
                self.message_term(
                    code,
                    &crate::dispatch::MessageTerm {
                        target,
                        name: &assigned_name,
                        super_class: super_class.as_deref(),
                        args,
                        // The oracle builds this form as `KEYWORD_MESSAGE`
                        // whatever the term's own tilde count, so a `~~`
                        // written here is not a cascade.
                        cascade: false,
                        assigned: Some(value),
                    },
                )?
            }
        };
        let slot = self.reserved_result_slot();
        let frame = self.activation().frame;
        // **A send that produced no value drops `RESULT`** rather than
        // leaving the previous one in place -- the same rule a bare `return`
        // from a `CALL` follows. Measured: `result = 'unset'` then `.K~m`
        // then `symbol('RESULT')` is `LIT`.
        match result {
            Some(result) => {
                self.roots.push_temp(result);
                self.set_variable(frame, slot, result);
            }
            None => self.clear_variable(frame, slot),
        }
        Ok(Flow::Next)
    }

    /// `PROCEDURE`, with or without an `EXPOSE` list (D9r).
    fn exec_procedure(
        &mut self,
        code: &Code<'_>,
        variables: &[VariableRef],
        first_instruction: bool,
    ) -> Result<(), Failure> {
        // 17.1 covers every shape but one: the first instruction executed
        // after an internal `CALL` or function invocation. Both halves are
        // needed -- every other entry fails the second, and anything after
        // another instruction in the same activation fails the first.
        let entered_by_internal_call = match self.activation().entry {
            Entry::InternalCall => true,
            Entry::TopLevel | Entry::Routine | Entry::Method => false,
        };
        if !(first_instruction && entered_by_internal_call) {
            return Err(Raised::procedure_out_of_place().into());
        }

        // The swap at the end of this function gives the callee a frame of
        // its own, and an activation that already owned one would be pushing
        // a second onto the same stack. That is the state
        // `RootSet::grow_slots` and `pop_slots` catch a step or two later,
        // by which time the instruction that caused it has returned, so the
        // invariant is asserted where it is established rather than where
        // the damage surfaces. `Activation::nested` builds the entry kind
        // admitted above, and it starts the callee sharing.
        assert!(
            !self.activation().owns_frame,
            "a PROCEDURE admitted in an activation that already owns its frame"
        );

        let names = self.expose_names(code, variables)?;

        // Resolved against the pool still in force, which is the caller's:
        // this activation has not swapped in a frame of its own yet.
        let outer = self.activation().frame;
        let mut bindings: Vec<(Box<[u8]>, usize, VarHome)> = Vec::with_capacity(names.len());
        for name in names {
            // Whole stems alias fine -- the stem object lives in one slot,
            // so aliasing that slot shares the object and every measured
            // stem transcript falls out of it. A single tail does not; see
            // `Loud::compound_expose`.
            if shape_of(&name) == NameShape::Compound {
                return Err(Loud::compound_expose("PROCEDURE EXPOSE", &name).into());
            }
            let slot = self.slot_of(&name);
            // A name the enclosing method exposed has no frame storage to
            // alias: its home is the object's pool, and the callee gets the
            // same home rather than a slot. Measured -- a class method
            // exposing `v` and calling `inner: procedure expose v`, which
            // assigns `v` -- the object variable is what changes.
            let target = match self.exposure(outer, slot) {
                Some(var) => VarHome::Instance(Box::new(var.clone())),
                None => VarHome::Slot(self.roots.slot_ref(outer, slot)),
            };
            bindings.push((name, slot, target));
        }

        // Any name that needed a fresh slot just grew the caller's frame and
        // was recorded in *this* activation's `extra` -- which is a clone of
        // the caller's, taken at the call. The caller has to learn about it,
        // because after the isolation below this map is replaced and the
        // return path deliberately does not write it back.
        let resolved = self.activation().extra.clone();
        if let Some(caller) = self.caller_activation_mut() {
            caller.extra = resolved;
        }

        // Sized from the caller's *current* frame length rather than from
        // `plan.len()`: an exposed name may sit at an index the caller grew
        // into, and that same index has to address something on this side of
        // the alias too.
        let len = self.roots.frame_len(outer);
        let inner = self.roots.push_slots(len);
        let mut exposed: Vec<(usize, InstanceVar)> = Vec::new();
        for (_, slot, target) in &bindings {
            match target {
                VarHome::Slot(target) => self.roots.alias_slot(inner, *slot, *target),
                VarHome::Instance(var) => exposed.push((*slot, (**var).clone())),
            }
        }

        // The callee's own run-time bindings start empty -- that is the
        // isolation -- except for exposed names the plan never saw, which
        // must keep resolving to the index the alias was installed at.
        let plan = Rc::clone(&self.activation().plan);
        let mut extra = rexx_core::NameMap::default();
        for (name, slot, _) in bindings {
            if plan.slot_of(&name).is_none() {
                extra.insert(name, slot);
            }
        }

        let activation = self.activation_mut();
        activation.frame = inner;
        activation.owns_frame = true;
        activation.extra = extra;
        // **Replaced, not extended.** This activation inherited the caller's
        // exposures when it was pushed, and a `PROCEDURE` isolates the pool:
        // a name the caller exposed and this list does not name is an
        // ordinary local here. Measured -- a class method exposing `v` and
        // calling `inner: procedure` with no list, which assigns `v` -- the
        // object variable is unchanged.
        activation.exposed = exposed;
        Ok(())
    }

    /// `EXPOSE`: bind every name it lists to the receiving object's variable
    /// pool for the scope the running method was declared in.
    pub(crate) fn exec_expose(
        &mut self,
        code: &Code<'_>,
        variables: &[VariableRef],
    ) -> Result<(), Failure> {
        let Some(identity) = self.activation().method_identity.as_ref() else {
            return Err(Raised::expose_outside_method().into());
        };
        let scope = identity.scope;
        let receiver = identity.receiver;
        let owner = self.pool_owner(receiver)?;
        for variable in variables {
            match variable {
                VariableRef::Direct(id) => {
                    let name = code.symbols.name(*id).as_bytes().into();
                    self.bind_exposed(owner, scope, name)?;
                }
                // **The selector's own name is bound before its value is
                // read**, and the order is observable rather than tidy.
                // Measured: with a class-scope `LISTER` holding `'BETA'` and a
                // class-scope `BETA` holding `'beta-value'`, `expose (lister)`
                // in a third method reads `[BETA][beta-value]` -- so `LISTER`
                // was read out of the object's pool and `BETA` was exposed
                // from it. Reading the selector first, out of the frame, gets
                // `[BETA][BETA]`: the frame's `LISTER` is unset, so its
                // derived name `LISTER` is what spells the list.
                VariableRef::Indirect(id) => {
                    let name = code.symbols.name(*id).as_bytes().into();
                    self.bind_exposed(owner, scope, name)?;
                    let (value, _novalue) = self.read(code, *id);
                    // `IndirectVariableReference::evaluate`'s own
                    // `value->requestString()`
                    // (`expression/IndirectVariableReference.cpp:132`), so a
                    // selector holding an object spells its list from the
                    // conversion. Measured, oracle rc 0: `zz = 1; x = .K;
                    // drop (x)` with a class-side `makeString` returning
                    // `'zz'` leaves `SYMBOL('ZZ')` at `LIT`.
                    let value = self.required_string_value(value)?;
                    let text = self.to_text(value).into_owned();
                    for word in split_indirect_words(&text) {
                        let word = validate_indirect_word(word)?;
                        self.bind_exposed(owner, scope, word.into())?;
                    }
                }
            }
        }
        Ok(())
    }

    /// Binds one name to `scope`'s pool on `owner`, for the rest of this
    /// activation.
    fn bind_exposed(
        &mut self,
        owner: ObjRef,
        scope: ObjRef,
        name: Box<[u8]>,
    ) -> Result<(), Failure> {
        // A whole stem is one value in one pool entry, so it binds like any
        // other name; a single tail is aliasing *inside* a stem object, which
        // this crate has no representation for. Measured on the oracle,
        // `expose a.1` in one class method assigning `a.1` and `a.2` and the
        // same in another reading them back: `[tail-one][A.2]`, so tail 1 is
        // shared and tail 2 is the method's own local. Exposing the whole stem
        // instead would be a silent wrong answer.
        if shape_of(&name) == NameShape::Compound {
            return Err(Loud::compound_expose("EXPOSE", &name).into());
        }
        let slot = self.slot_of(&name);
        let var = InstanceVar { owner, scope, name };
        let activation = self.activation_mut();
        // Replaced rather than appended: `expose v v` is legal and rc 0 on the
        // oracle, and two entries for one slot would leave every later read
        // deciding between them by list order.
        match activation.exposed.iter_mut().find(|(at, _)| *at == slot) {
            Some(bound) => bound.1 = var,
            None => activation.exposed.push((slot, var)),
        }
        Ok(())
    }

    /// The object whose [`rexx_core::ScopePools`] a send to `receiver` binds
    /// into.
    pub(crate) fn pool_owner(&mut self, receiver: ObjRef) -> Result<ObjRef, Failure> {
        if !self.heap.is_class(receiver) {
            if matches!(
                self.heap.get(receiver).map(|object| &object.body),
                Some(Body::Instance { .. })
            ) {
                return Ok(receiver);
            }
            return Err(Loud::expose_receiver().into());
        }
        if let Some(owner) = self.class_variables.get(&receiver) {
            return Ok(*owner);
        }
        // The pools of the class *object*, whose own class is its metaclass --
        // what `.K~class` answers. This object is storage and never a value a
        // program holds.
        let holds = self.classes().class_of(receiver);
        let behaviour = self.classes().instance_behaviour_handle(holds);
        let owner = self.alloc_with(
            BehaviourId::OBJECT,
            Body::Instance {
                class: holds,
                behaviour,
                name: None,
                pools: ScopePools::new(),
                own: None,
                native: None,
            },
        );
        // Held by the class before anything else can allocate, which is the
        // rule `.environment` and `.local` are created under.
        self.class_owns(receiver, owner);
        self.class_variables.insert(receiver, owner);
        Ok(owner)
    }

    /// Every name one `PROCEDURE EXPOSE` list names, in source order.
    fn expose_names(
        &mut self,
        code: &Code<'_>,
        variables: &[VariableRef],
    ) -> Result<Vec<Box<[u8]>>, Failure> {
        let mut names: Vec<Box<[u8]>> = Vec::new();
        for variable in variables {
            match variable {
                VariableRef::Direct(id) => names.push(code.symbols.name(*id).as_bytes().into()),
                VariableRef::Indirect(id) => {
                    // The selector itself, then the names its value spells.
                    names.push(code.symbols.name(*id).as_bytes().into());
                    let (value, _novalue) = self.read(code, *id);
                    let value = self.required_string_value(value)?;
                    let text = self.to_text(value).into_owned();
                    for word in split_indirect_words(&text) {
                        names.push(validate_indirect_word(word)?.into());
                    }
                }
            }
        }
        Ok(names)
    }

    /// `USE ARG`, `USE STRICT ARG` and `USE LOCAL`.
    /// ```text
    /// as the program's own first instruction    98.993, rc 158
    /// as a ::ROUTINE's own first instruction    98.993, rc 158
    /// anywhere else                             99.910, rc 157
    /// ```
    fn exec_use(
        &mut self,
        code: &Code<'_>,
        use_: &Use,
        first_instruction: bool,
    ) -> Result<(), Failure> {
        match use_ {
            Use::Local { .. } => {
                // 98.993 is "this is not a method invocation" and 99.910 is
                // "it is one, but this is not its first instruction", so
                // what decides between them is the entry kind rather than
                // whether there was a call. Measured on the oracle: `use
                // local` first in a `::ROUTINE` is 98.993 at rc 158, the
                // same answer the top-level shape gets.
                let method_invocation = match self.activation().entry {
                    Entry::TopLevel | Entry::InternalCall | Entry::Routine => false,
                    Entry::Method => true,
                };
                if first_instruction && method_invocation {
                    // The one shape the oracle **runs**: measured, `use
                    // local` as a `::METHOD`'s first instruction is rc 0.
                    // What it does is bind every name in its list as a
                    // local, which is `EXPOSE`'s own machinery seen from the
                    // other side, so it is loud until that lands rather than
                    // answering a condition the oracle does not raise.
                    Err(Loud::use_local_in_a_method().into())
                } else if first_instruction {
                    Err(Raised::use_local_outside_method().into())
                } else {
                    Err(Raised::use_local_not_first().into())
                }
            }
            Use::Arg {
                strict,
                allow_optionals,
                targets,
            } => self.exec_use_arg(code, *strict, *allow_optionals, targets),
        }
    }

    /// `USE ARG`/`USE STRICT ARG`: bind the call's arguments to this
    /// instruction's targets, positionally.
    fn exec_use_arg(
        &mut self,
        code: &Code<'_>,
        strict: bool,
        allow_optionals: bool,
        targets: &[Option<UseTarget>],
    ) -> Result<(), Failure> {
        let in_method = self.activation().entry == Entry::Method;
        if strict {
            let supplied = self.call_context.arguments.len();
            // The minimum is the position of the last target that must be
            // supplied -- one with no default of its own. A later target
            // carrying a default does not raise it, which is what makes `use
            // strict arg p, q = 'dflt'` legal with one argument.
            let minimum = targets
                .iter()
                .rposition(|target| {
                    target
                        .as_ref()
                        .is_none_or(|target| target.default.is_none())
                })
                .map_or(0, |index| index + 1);
            if supplied < minimum {
                return Err(if in_method {
                    Raised::not_enough_method_arguments(minimum).into()
                } else {
                    let name = self.call_context.name.clone();
                    Raised::not_enough_arguments(&name, minimum).into()
                });
            }
            if !allow_optionals && supplied > targets.len() {
                return Err(if in_method {
                    Raised::too_many_method_arguments(targets.len()).into()
                } else {
                    let name = self.call_context.name.clone();
                    Raised::too_many_arguments(&name, targets.len()).into()
                });
            }
        }

        for (index, target) in targets.iter().enumerate() {
            let Some(target) = target else { continue };
            // `get` past the end and a `None` inside the list are the same
            // thing to a target: nothing was supplied for this position.
            let argument = self.call_context.arguments.get(index).cloned().flatten();
            self.bind_use_target(code, index, target, argument, strict, in_method)?;
        }
        Ok(())
    }

    /// Binds one `USE ARG` target to one argument, or to its default, or to
    /// nothing.
    fn bind_use_target(
        &mut self,
        code: &Code<'_>,
        index: usize,
        target: &UseTarget,
        argument: Option<ObjRef>,
        strict: bool,
        in_method: bool,
    ) -> Result<(), Failure> {
        let position = index + 1;
        if target.alias {
            let Some(argument) = argument else {
                return Err(Raised::variable_reference_omitted(position).into());
            };
            let Some(bound) = self.as_variable_reference(argument) else {
                let found = self.to_text(argument).to_vec();
                return Err(Raised::not_a_variable_reference(position, &found).into());
            };
            let reference = bound.name.clone();
            let slot = bound.home.clone();
            let name = self.use_target_name(code, target)?;
            // **The kinds must match, and the check is before the
            // uninitialised one.** Measured: a target that is both
            // kind-mismatched and already assigned reports the kind error,
            // not 98.995. Compound is not a third kind to handle -- `>p.1`
            // and `>q.1` are both rejected by `rexx-parse` (20.930/20.931),
            // so each side is a simple variable or a stem and nothing else.
            let target_is_stem = shape_of(&name) == NameShape::Stem;
            let reference_is_stem = shape_of(&reference) == NameShape::Stem;
            if target_is_stem != reference_is_stem {
                // Both substitute the *caller's* name, unlike 98.995 just
                // below, which names the target. Measured with a variable
                // whose value differs from its name, so the two cannot be
                // confused: `p = 'value-not-name'` passed as `>p` reports
                // `found "P"`.
                return Err(if target_is_stem {
                    Raised::not_a_stem_variable_reference(position, &reference).into()
                } else {
                    Raised::not_a_simple_variable_reference(position, &reference).into()
                });
            }
            let index = self.slot_of(&name);
            let frame = self.activation().frame;
            // The target must be **currently unset**. `RootSet::slot`
            // resolves through any alias already in force, which is what the
            // repeat case needs: after one `use arg >q`, `Q` reads the
            // caller's variable, so it "has a value" and the second attempt
            // is refused.
            if !self.target_is_uninitialised(&name, frame, index) {
                return Err(Raised::variable_reference_not_uninitialised(&name).into());
            }
            match slot {
                VarRefHome::Cell(cell) => self.roots.alias_slot(frame, index, cell),
                // The same binding `EXPOSE` makes, on this activation's own
                // slot: the target names the caller's object variable rather
                // than any frame storage, so there is nothing to alias to.
                VarRefHome::Instance { owner, scope } => {
                    let var = InstanceVar {
                        owner,
                        scope,
                        name: reference.clone(),
                    };
                    let activation = self.activation_mut();
                    match activation.exposed.iter_mut().find(|(at, _)| *at == index) {
                        Some(bound) => bound.1 = var,
                        None => activation.exposed.push((index, var)),
                    }
                }
            }
            // `>R>`, the alias's own line and the **only** trace line this
            // branch emits: no `>>>` and no `>=>`, because nothing was
            // evaluated and nothing was assigned (`UseInstruction.cpp:164`-
            // `167`, `aliasVariable` then `traceVariableAlias`, and
            // `handleArgument` `return`s before its own `traceResult` for
            // this case). Caller's name first, target's second -- see
            // `trace_alias`.
            self.trace_alias(self.clause_state.current_value_indent, &reference, &name);
            return Ok(());
        }

        // Present: bind the value. Absent: the default if there is one, and
        // otherwise drop the target -- measured, an absent target does not
        // keep whatever it held before.
        let value = match argument {
            Some(argument) => Some(argument),
            None => match &target.default {
                Some(default) => {
                    let value = self.eval(code, default)?;
                    self.roots.push_temp(value);
                    Some(value)
                }
                None => None,
            },
        };
        let name = self.use_target_name(code, target)?;
        match value {
            Some(value) => {
                // `>>>` then `>=>`, in that order and both at this `USE`
                // clause's own indent -- `handleArgument`'s own
                // `traceResult(argument)` immediately before
                // `retriever->assign(context, argument)`, whose own
                // `traceAssignment` is the second line
                // (`UseInstruction.cpp:74`-`77`, and the default-value arm
                // ten lines below it does the identical pair). Measured
                // under `trace r`: `use arg a, b` on a two-argument call
                // traces `>>>     "1"` and `>>>     "2"` and no `>=>`, which
                // is the gating -- `>>>` is `results`, `>=>` is
                // `intermediates`, so the pair is not one line's worth of
                // conditional.
                let indent = self.clause_state.current_value_indent;
                // `results` and not `intermediates`, though the pair below
                // needs both: `results` is the weaker gate, true wherever
                // `intermediates` is, so this renders for either line and
                // drops neither.
                let rendered = self.result_text(value);
                if let Some(rendered) = &rendered {
                    self.trace_result(indent, rendered);
                }
                self.assign_by_name(&name, value);
                if let Some(rendered) = &rendered {
                    self.trace_assignment(indent, &name, rendered);
                }
                Ok(())
            }
            // `USE STRICT ARG` refuses an omitted position that has no
            // default of its own, where `USE ARG` drops the target
            // (`UseInstruction.cpp:97`-`:111`). The position is the target's,
            // not the last one supplied -- measured, oracle rc 216, `call r
            // , 2` into `use strict arg a, b` reports `argument 1`.
            None if strict => Err(if in_method {
                Raised::missing_method_argument(position).into()
            } else {
                let call = self.call_context.name.clone();
                Raised::missing_argument(&call, position).into()
            }),
            None => {
                self.drop_by_name(&name);
                Ok(())
            }
        }
    }

    /// Whether a `USE ARG >name` target is in the uninitialised state the
    /// oracle requires of it.
    fn target_is_uninitialised(&self, name: &[u8], frame: SlotFrame, index: usize) -> bool {
        match self.variable(frame, index) {
            None => true,
            Some(value) => shape_of(name) == NameShape::Stem && self.is_uninitialised_stem(value),
        }
    }

    /// One `USE ARG` target's variable name.
    fn use_target_name(&self, code: &Code<'_>, target: &UseTarget) -> Result<Vec<u8>, Failure> {
        match &target.target.kind {
            ExprKind::Variable(id) | ExprKind::Stem(id) | ExprKind::Compound(id) => {
                Ok(code.symbols.name(*id).as_bytes().to_vec())
            }
            other => Err(Loud::expression(other).into()),
        }
    }

    /// Everything one `SAY` does once its expression has been evaluated:
    /// `>>>`, then the line itself.
    #[inline]
    pub(crate) fn say_evaluated(&mut self, value: Option<ObjRef>) -> Result<(), Failure> {
        let line = match value {
            Some(value) => {
                self.roots.push_temp(value);
                let value = self.required_string_value(value)?;
                self.to_text(value).to_vec()
            }
            None => Vec::new(),
        };
        self.trace_result(self.clause_state.current_value_indent, &line);
        // `Activity::sayOutput` (`concurrency/Activity.cpp:3214`): `.OUTPUT`
        // gets the line as a `SAY` message and its reply is dropped; an entry
        // that is missing **or holds `.nil`** takes the buffer instead.
        //
        // The `.nil` test belongs here rather than in `resolve_stream`, and
        // the asymmetry is measured: `.local~output = .nil` then `say` then
        // `lineout(,)` prints the `SAY` line and *then* fails 97.1 on the
        // builtin, because `resolveStream` has no such test and sends to the
        // `.nil` it found.
        // **The direct write when nothing has redirected `.OUTPUT`**, which is
        // the same bytes the monitor would forward and none of the cost:
        // measured, delivering every `SAY` is 16,360 instructions a line and
        // puts `sayloop` at 9.63x. A match guard cannot ask, because a guard
        // may not borrow `self` mutably.
        match self.output_route()? {
            Some(route) => {
                let argument = self.text_built(line);
                // **Rooted before the send.** The line can be long enough to
                // take the owned-`Bytes` path, and the send allocates: without
                // this the collect-on-every-allocation gate fails `a live
                // value` at `dispatch.rs:1465`. Measured, adding it takes the
                // library suite from 788 to 790.
                self.roots.push_temp(argument);
                let caller = self.caller();
                self.send_message(route, crate::dispatch::SAY, None, &[Some(argument)], caller)?;
            }
            _ => {
                self.out.extend_from_slice(&line);
                self.out.push(b'\n');
            }
        }
        Ok(())
    }

    /// `GUARD ON`/`GUARD OFF`, with or without a `WHEN` expression
    /// (`RexxInstructionGuard::execute`, `instructions/GuardInstruction.cpp`).
    fn exec_guard(&mut self, code: &Code<'_>, guard: &Guard) -> Result<Flow, Failure> {
        if self.activation().method_identity.is_none() {
            return Err(Raised::guard_outside_method().into());
        }
        let Some(condition) = &guard.condition else {
            return Ok(Flow::Next);
        };
        let holds = self.eval_condition(
            code,
            condition,
            ConditionTrace::Keyword(self.clause_state.current_value_indent, "WHEN"),
            raised_guard_not_logical,
        )?;
        if holds {
            Ok(Flow::Next)
        } else {
            Err(Loud::guard_when_false().into())
        }
    }

    /// `REPLY`, bare or with a value (`RexxInstructionReply::execute`,
    /// `instructions/ReplyInstruction.cpp:66`).
    fn exec_reply(
        &mut self,
        code: &Code<'_>,
        index: usize,
        expression: Option<&Expr>,
    ) -> Result<Flow, Failure> {
        if self.activation().method_identity.is_none() {
            return Err(Raised::reply_outside_method().into());
        }
        if !top_level_clause(code.body, index) {
            return Err(Loud::reply_inside_construct().into());
        }
        let value = match expression {
            Some(expression) => Some(self.eval(code, expression)?),
            None => None,
        };
        // The rooting and the `>>>` line are `RETURN`'s, for the same reason
        // and at the same indent: `evaluateExpression` is the shared call in
        // the C++ and this is the shared call here.
        if let Some(value) = value {
            self.roots.push_temp(value);
            if let Some(rendered) = self.result_text(value) {
                self.trace_result(self.clause_state.current_value_indent, &rendered);
            }
        }
        if self.activation().reply != ReplyState::None {
            return Err(Raised::reply_twice().into());
        }
        let activation = self.activation_mut();
        activation.reply = ReplyState::Owed;
        activation.replied_a_value = value.is_some();
        activation.pc = index + 1;
        Ok(Flow::Return(value))
    }

    /// `FORWARD`, with any of `TO`, `MESSAGE`, `CLASS`, `ARGUMENTS`, `ARRAY`
    /// and `CONTINUE` (`RexxInstructionForward::execute`,
    /// `instructions/ForwardInstruction.cpp:128`).
    fn exec_forward(&mut self, code: &Code<'_>, forward: &Forward) -> Result<Flow, Failure> {
        let Some(identity) = self.activation().method_identity.as_ref() else {
            return Err(Raised::forward_outside_method().into());
        };
        let receiver = identity.receiver;
        let own_name = identity.name.clone();
        let indent = self.clause_state.current_value_indent;

        // The option order is the C++'s, and it is observable in the trace:
        // `TO`, `MESSAGE`, `CLASS`, then whichever of `ARGUMENTS` and `ARRAY`
        // is present. Measured under `trace i`, `forward to (t)
        // message('OTHER') array(1,2)` emits `>K> "TO"`, `>K> "MESSAGE"` and
        // `>K> "ARRAY"` in that order, with the `ARRAY` items' own `>A>`
        // lines ahead of its keyword line.
        let target = match &forward.to {
            None => receiver,
            Some(expr) => self.forward_keyword(code, expr, "TO")?,
        };
        let message = match &forward.message {
            None => own_name,
            Some(expr) => {
                let value = self.forward_keyword(code, expr, "MESSAGE")?;
                let text = self.required_string_value(value)?;
                self.to_text(text).to_ascii_uppercase().into_boxed_slice()
            }
        };
        let start_scope = match &forward.class {
            None => None,
            Some(expr) => {
                let value = self.eval(code, expr)?;
                self.roots.push_temp(value);
                // `_superClass->isInstanceOf(TheClassClass)`
                // (`ForwardInstruction.cpp:171`), reported with the same two
                // fixed substitutions a `~name:scope` override's own check
                // takes. Measured, `forward class (5) message('OTHER')` is
                // 88.914 at rc 168.
                if !self.heap.is_class(value) {
                    return Err(Raised::scope_override_not_a_class().into());
                }
                self.trace_forward_keyword("CLASS", value);
                Some(value)
            }
        };

        let (mut values, mark) = self.take_value_buffer();
        let evaluated = self.forward_arguments(code, forward, &mut values);
        // **After every option and before the send**, which is where
        // `RexxActivation::forward` asks it (`execution/RexxActivation.cpp:
        // 1367`-`:1369`): a non-continuing `FORWARD` answers the sender, and
        // a `REPLY` carrying a value has answered it already.
        let owed = evaluated.and_then(|()| self.forward_after_reply(forward));
        // `settings.setForwarded(true)` (`execution/RexxActivation.cpp:1372`):
        // after the 98.937 above and before the send below, so a condition
        // the send raises is not offered to this activation's own traps.
        // [`Activation::forwarded`] carries what that costs.
        if owed.is_ok() && !forward.continue_ {
            self.activation_mut().forwarded = true;
        }
        let caller = self.caller();
        let sent = owed
            .and_then(|()| self.validate_scope_override(target, start_scope))
            .and_then(|()| {
                self.send_message(target, &message, start_scope, &values[mark..], caller)
            });
        self.give_value_buffer(values, mark);
        let sent = sent?;

        if !forward.continue_ {
            return Ok(Flow::Return(sent));
        }
        let slot = self.reserved_result_slot();
        let frame = self.activation().frame;
        match sent {
            Some(value) => {
                self.roots.push_temp(value);
                if let Some(rendered) = self.result_text(value) {
                    self.trace_result(indent, &rendered);
                }
                self.set_variable(frame, slot, value);
            }
            None => self.clear_variable(frame, slot),
        }
        Ok(Flow::Next)
    }

    /// 98.937 for a non-continuing `FORWARD` under a `REPLY` that carried a
    /// value, which is the one legality question `FORWARD` asks that is not
    /// about `FORWARD` (`execution/RexxActivation.cpp:1367`-`:1369`).
    fn forward_after_reply(&self, forward: &Forward) -> Result<(), Failure> {
        if forward.continue_ || !self.activation().replied_a_value {
            return Ok(());
        }
        Err(Raised::exit_after_reply().into())
    }

    /// One `FORWARD` option that is a single expression: its value, rooted,
    /// with the `>K>` line the oracle's `traceKeywordResult` writes.
    fn forward_keyword(
        &mut self,
        code: &Code<'_>,
        expr: &Expr,
        keyword: &str,
    ) -> Result<ObjRef, Failure> {
        let value = self.eval(code, expr)?;
        self.roots.push_temp(value);
        self.trace_forward_keyword(keyword, value);
        Ok(value)
    }

    /// One `FORWARD` option's `>K>` line, for a caller that has to evaluate
    /// and trace at separate points.
    fn trace_forward_keyword(&mut self, keyword: &str, value: ObjRef) {
        let traced = self.string_value_text(value);
        self.trace_keyword(self.clause_state.current_value_indent, keyword, &traced);
    }

    /// The argument list a `FORWARD` sends, into a borrowed buffer so that
    /// [`Interp::exec_forward`] returns it on the failure path too.
    fn forward_arguments(
        &mut self,
        code: &Code<'_>,
        forward: &Forward,
        values: &mut Vec<Option<ObjRef>>,
    ) -> Result<(), Failure> {
        if let Some(expr) = &forward.arguments {
            let value = self.forward_keyword(code, expr, "ARGUMENTS")?;
            self.forward_arguments_converted(value, values)?;
            while values.last().is_some_and(Option::is_none) {
                values.pop();
            }
            return Ok(());
        }
        if let Some(items) = &forward.array {
            for item in items {
                match item {
                    None => {
                        self.trace_argument(self.clause_state.current_value_indent, b"");
                        values.push(None);
                    }
                    Some(expr) => {
                        values.push(Some(self.eval_traced_argument(code, expr)?));
                    }
                }
            }
            // The line renders the instruction's own array of expressions,
            // which has no evaluated counterpart here. Measured under
            // `trace i`, `forward message('OTHER') array(1,2)` writes
            // `>K>   "ARRAY" => "an Array"` after the items' own `>A>` lines.
            self.trace_keyword(
                self.clause_state.current_value_indent,
                "ARRAY",
                crate::dispatch::ARRAY_DEFAULT_NAME,
            );
            return Ok(());
        }
        values.extend(self.call_context.arguments.iter().copied());
        // Rooted here rather than relied on through `call_context`, which the
        // collector does not walk.
        for value in values.iter().flatten() {
            self.roots.push_temp(*value);
        }
        Ok(())
    }

    /// `ARGUMENTS expr`'s value through `requestArray`
    /// (`instructions/ForwardInstruction.cpp:182`), appended to `values`.
    fn forward_arguments_converted(
        &mut self,
        value: ObjRef,
        values: &mut Vec<Option<ObjRef>>,
    ) -> Result<(), Failure> {
        match self.forward_arguments_conversion(value) {
            Conversion::Array => {
                let slots = self.array_slots_of(value).unwrap_or_default();
                values.extend(slots);
            }
            Conversion::Lines => {
                let text = self.to_text(value).into_owned();
                for line in makearray_lines(&text) {
                    self.push_converted_argument(line, values);
                }
            }
            Conversion::Tails => {
                for tail in self.stem_assigned_tails(value) {
                    self.push_converted_argument(&tail, values);
                }
            }
            Conversion::Refused => return Err(Raised::forward_arguments().into()),
            Conversion::NotBuilt(kind) => {
                return Err(Loud::object_position("FORWARD ARGUMENTS", kind).into());
            }
        }
        Ok(())
    }

    /// Which arm of `requestArray` this value takes.
    fn forward_arguments_conversion(&mut self, value: ObjRef) -> Conversion {
        match value.decode() {
            Decoded::Nil => return Conversion::Refused,
            // `RexxInteger::makeArray` and `NumberString::makeArray` both
            // hand the work to their string value's.
            Decoded::SmallInt(_) | Decoded::Text(_) => return Conversion::Lines,
            Decoded::Heap { .. } => {}
        }
        match self.heap.get(value).map(|object| &object.body) {
            // A class object is a primitive with no `makeArray` of its own,
            // so `requestArray` stops at `TheNilObject`.
            Some(Body::Class { .. }) => Conversion::Refused,
            // A multi-dimensional array shares `TheNilObject`'s raise rather
            // than converting (`instructions/ForwardInstruction.cpp:189`-
            // `:191`).
            Some(Body::Array { .. }) => {
                if self.is_multi_dimensional_array(value) {
                    Conversion::Refused
                } else {
                    Conversion::Array
                }
            }
            Some(Body::Text { .. } | Body::Num { .. }) => Conversion::Lines,
            // `StemClass::makeArray` is `tailArray`, which is the assigned
            // tails and never the default: measured, `a. = 'dflt'` with no
            // tail assigned forwards no arguments at all.
            Some(Body::Stem { .. }) => Conversion::Tails,
            // `requestArray` sends `REQUEST('ARRAY')` for a non-primitive,
            // which looks `MAKEARRAY` up in the behaviour and sends it, and
            // otherwise answers `.nil` (`RexxObject::requestRexx`,
            // `classes/ObjectClass.cpp:1920`-`:1940`).
            Some(Body::Instance { .. }) => {
                if self.answers_message(value, "MAKEARRAY") {
                    Conversion::NotBuilt("an instance of a user class")
                } else {
                    Conversion::Refused
                }
            }
            // **Not the referent's conversion**, which is what
            // `~request('ARRAY')` answers and is a different route:
            // `requestArray` looks `MAKEARRAY` up in the receiver's own
            // behaviour rather than sending it, and a reference's behaviour
            // holds no such name, so the lookup fails and `TheNilObject` is
            // the answer. Measured, oracle rc 158: `forward arguments (>v)`
            // over a `v` holding `'val'` is `98.946`, where the same
            // instruction over `v` itself is rc 0 -- so this arm may not
            // chase the way the string conversion beside it does.
            Some(Body::VarRef(_)) => Conversion::Refused,
            Some(Body::Native(_) | Body::WeakRef(_)) | None => {
                Conversion::NotBuilt("one of the interpreter's own objects")
            }
        }
    }

    /// One converted `ARGUMENTS` item, rooted as it is appended.
    fn push_converted_argument(&mut self, bytes: &[u8], values: &mut Vec<Option<ObjRef>>) {
        let item = self.text(bytes);
        self.roots.push_temp(item);
        values.push(Some(item));
    }

    /// A stem's assigned tails, ordered by `CompoundVariableTail::compare`
    /// (`classes/support/CompoundVariableTail.hpp:170`), which sorts on
    /// length first and bytes second.
    fn stem_assigned_tails(&self, value: ObjRef) -> Vec<Vec<u8>> {
        let Some(Body::Stem { tails, .. }) = self.heap.get(value).map(|object| &object.body) else {
            return Vec::new();
        };
        let mut names: Vec<Vec<u8>> = tails
            .iter()
            .filter(|(_, (_, held))| held.is_some())
            .map(|(name, _)| name.clone())
            .collect();
        names.sort_by(|left, right| left.len().cmp(&right.len()).then_with(|| left.cmp(right)));
        names
    }

    /// Everything a `RETURN` or an `EXIT` does once its expression has been
    /// evaluated: its `>>>`, and the `Flow` that leaves the activation.
    pub(crate) fn returned_value(
        &mut self,
        value: Option<ObjRef>,
        keyword: ReturnKeyword,
    ) -> Result<Flow, Failure> {
        if let Some(value) = value {
            self.roots.push_temp(value);
            if let Some(rendered) = self.result_text(value) {
                self.trace_result(self.clause_state.current_value_indent, &rendered);
            }
        }
        // **LEGALITY, and Phase 6 keeps it.** A `REPLY` has already answered
        // the sender, so a value here has nobody to go to whatever activity
        // the rest of the body runs on: 98.936 for `RETURN`, 98.937 for
        // `EXIT`. Both are
        // asked **after** the trace line above, which is the order the C++
        // takes them in -- `RexxInstructionReturn::execute` evaluates through
        // `evaluateExpression` and only then calls `returnFrom`
        // (`instructions/ReturnInstruction.cpp:72`), whose check is at
        // `execution/RexxActivation.cpp:1074`; `exitFrom`'s own is at
        // `:1413`. The bare form of either is legal after a reply and is
        // measured: `reply 'v'` then `say 'tail'` then `return` is rc 0 with
        // both lines printed and an empty stderr.
        if value.is_some() && self.activation().reply != ReplyState::None {
            return Err(match keyword {
                ReturnKeyword::Return => Raised::return_after_reply(),
                ReturnKeyword::Exit => Raised::exit_after_reply(),
            }
            .into());
        }
        Ok(match keyword {
            ReturnKeyword::Return => Flow::Return(value),
            ReturnKeyword::Exit => Flow::Exit(value),
        })
    }

    /// Everything a `PUSH` or a `QUEUE` does once its expression has been
    /// evaluated: the `>>>` line, and the line itself onto one end of the
    /// queue.
    pub(crate) fn queue_evaluated(
        &mut self,
        value: Option<ObjRef>,
        keyword: QueueKeyword,
    ) -> Result<(), Failure> {
        let line = match value {
            Some(value) => {
                self.roots.push_temp(value);
                // `evaluateStringExpression` again, so the `>>>` below traces
                // the conversion: measured, `trace r` over `push .K` with a
                // class-side `makeString` returning `'pv'` prints
                // `>>>   "pv"` and `pull` reads `PV`.
                let value = self.required_string_value(value)?;
                self.to_text(value).to_vec()
            }
            None => Vec::new(),
        };
        self.trace_result(self.clause_state.current_value_indent, &line);
        match keyword {
            QueueKeyword::Push => self.queue.push(line),
            QueueKeyword::Queue => self.queue.queue(line),
        }
        Ok(())
    }

    /// Everything one assignment does once its value has been evaluated:
    /// `>>>`, then the write and the lines the write itself produces.
    #[inline]
    pub(crate) fn assign_evaluated(
        &mut self,
        code: &Code<'_>,
        target: &Expr,
        value: ObjRef,
        at: Option<usize>,
    ) -> Result<(), Failure> {
        self.roots.push_temp(value);
        // `>>>` fires before the assignment itself
        // (`RexxInstructionAssignment::execute`: evaluate, trace, *then*
        // assign), which matters only in that the traced value can never be
        // affected by the write it precedes.
        // Reads `current_value_indent` rather than recomputing
        // `static_indent(index)` independently -- the clause unit already
        // computed exactly this value (`indent_offset` included, F-EX1's own
        // correction to F3) for this same instruction right before the clause
        // ran, and a second computation of the identical quantity is how the
        // two drift, which is exactly what happened here before this fix: this
        // site's own copy never learned about the offset when the field was
        // added.
        let indent = self.clause_state.current_value_indent;
        // One render for both lines, and `results` is the gate because it is
        // the weaker of the two: `>>>` is gated on `results` and `>=>` on
        // `intermediates`, and `results` is true wherever `intermediates` is.
        // Guarding on `intermediates` instead would drop the `>>>` line under
        // `TRACE R`.
        let rendered = self.result_text(value);
        if let Some(rendered) = &rendered {
            // **The entry gate, not just the setting in force.** An assignment
            // whose expression turned tracing on owes no `>>>`, because the
            // oracle chose the path without one before it evaluated. See
            // `ClauseState::instructions_traced_at_entry`, which carries the
            // measurement.
            if self.clause_state.instructions_traced_at_entry {
                self.trace_result(indent, rendered);
            }
        }
        self.assign_expr_target(code, target, value, rendered.as_deref(), indent, at)
    }

    /// Writes `value` into the slot a plain-variable target is already bound
    /// to, and traces the write. `rendered` is the assigned value's own text
    /// where a `>=>` line will print it, and `None` where none will.
    ///
    /// Panics in a debug build when `id` does not name a simple variable: the
    /// slot written here is the whole name's, which a stem or a compound tail
    /// does not have.
    #[inline]
    pub(crate) fn assign_bound_variable(
        &mut self,
        code: &Code<'_>,
        id: SymbolId,
        slot: usize,
        value: ObjRef,
        rendered: Option<&[u8]>,
        indent: usize,
    ) {
        debug_assert!(
            matches!(
                shape_of(code.symbols.name(id).as_bytes()),
                NameShape::Simple
            ),
            "a bound slot was written for {}, which is not a simple variable",
            code.symbols.name(id),
        );
        let frame = self.activation().frame;
        self.set_variable(frame, slot, value);
        // The name is reached only where a line will print it, which is what
        // the general path below pays on every call.
        if let Some(rendered) = rendered {
            self.trace_assignment(indent, code.symbols.name(id).as_bytes(), rendered);
        }
    }

    /// Writes `value` through one assignment *target expression*, and traces
    /// the write.
    pub(crate) fn assign_expr_target(
        &mut self,
        code: &Code<'_>,
        target: &Expr,
        value: ObjRef,
        rendered: Option<&[u8]>,
        indent: usize,
        at: Option<usize>,
    ) -> Result<(), Failure> {
        match &target.kind {
            ExprKind::Variable(id) => {
                let name = code.symbols.name(*id).as_bytes();
                let slot = match at {
                    Some(slot) => slot,
                    None => self.slot_of(name),
                };
                let frame = self.activation().frame;
                self.set_variable(frame, slot, value);
                if let Some(rendered) = rendered {
                    self.trace_assignment(indent, name, rendered);
                }
            }
            // `stem. = expr`: replace-and-rebind (D15a), through the
            // library `stem_assign` already builds -- this arm is the
            // dispatch, not new stem logic.
            ExprKind::Stem(id) => {
                // The tripwire `crate::ir::drive`'s `Op::Store` arm carries
                // for the compiled side, here where **both** engines pass:
                // a slot handed to this arm was resolved against the symbol's
                // own id and is silently shadowed below, so a caller that
                // started supplying one would write through the entry's slot
                // and never learn that its own was ignored.
                debug_assert!(
                    at.is_none(),
                    "a stem write was handed a slot, and the slot it writes comes from the entry"
                );
                let name = code.symbols.name(*id).as_bytes();
                let at = code.compound(*id).and_then(|entry| entry.stem_at);
                self.stem_assign_at(name, at, value);
                if let Some(rendered) = rendered {
                    self.trace_assignment(indent, name, rendered);
                }
            }
            // `a.b = expr`: resolve the tail key the same way reading
            // `a.b` would (`eval_node`'s own `Compound` arm), then
            // mutate that one tail in place through `stem_set`.
            ExprKind::Compound(id) => {
                // `read_symbol`'s own compound tripwire, on the writing
                // side. A compound-shaped name **can** reach
                // `Plan::by_symbol`: `Plan::bind` puts every name it binds
                // whole there, and `note_loop` and `note_parse` both call it
                // with spellings that may be compound-shaped. So a caller
                // reaching for a slot by this symbol's id can find one, and it
                // is not the slot this arm writes.
                debug_assert!(
                    at.is_none(),
                    "a compound write was handed a slot, and the slot it writes is the stem's"
                );
                // **Borrowed, not copied.** `Code::stem` and `SymbolTable::
                // name` both answer with the lifetime of the `Code`, which is
                // the program rather than this `Interp`, so neither needs an
                // owned copy to survive the `&mut self` calls below. The
                // `Variable` arm above has always passed its name borrowed;
                // this arm copied both, and measured with `heaptrack` on
                // `samples/rexxcps.rex` that cost two allocations per compound
                // write.
                let tag = code.symbols.name(*id).as_bytes();
                let (stem_name, stem_at) = code.stem(*id);
                let mut key = self.take_key_buffer();
                if let Err(failure) = self.tail_key_into(code, *id, &mut key) {
                    self.give_key_buffer(key);
                    return Err(failure);
                }
                self.stem_set_at(stem_name, stem_at, &key, value);
                // **The resolved name is built only when a line will print
                // it.** `trace_compound_name` returns at once unless
                // intermediates are on, so joining the stem to the tail key
                // ahead of that check allocated a name to discard. This is the
                // rule `rendered` below already follows -- `trace.rs`'s own
                // doc comment gives the reasoning for the value half, and the
                // name half is the same argument.
                if self.tracing_intermediates() {
                    let mut resolved = stem_name.to_vec();
                    resolved.extend_from_slice(&key);
                    self.trace_compound_name(indent, tag, &resolved);
                }
                if let Some(rendered) = rendered {
                    self.trace_assignment(indent, tag, rendered);
                }
                self.give_key_buffer(key);
            }
            other => return Err(Loud::expression(other).into()),
        }
        Ok(())
    }

    /// Assigns `value` to the variable, whole stem, or one verbatim-keyed
    /// tail that `name`'s own spelling names.
    pub(crate) fn assign_by_name(&mut self, name: &[u8], value: ObjRef) {
        match shape_of(name) {
            NameShape::Simple => {
                let slot = self.slot_of(name);
                let frame = self.activation().frame;
                self.set_variable(frame, slot, value);
            }
            NameShape::Stem => self.stem_assign(name, value),
            NameShape::Compound => {
                let dot = name
                    .iter()
                    .position(|&b| b == b'.')
                    .expect("NameShape::Compound guarantees at least one period");
                let (stem_name, key) = name.split_at(dot + 1);
                self.stem_set(stem_name, key, value);
            }
        }
    }

    /// `SIGL`, set at the point of every control transfer -- `SIGNAL`'s own
    /// two `step` arms and `Interp::invoke_call` (`CALL`, and `ExprKind::
    /// Call` through `eval_call`, `eval.rs`) -- to `line`, always `self.
    /// current_clause_line` at the call site (`lib.rs`'s own doc comment on
    /// that field has why it is a field and not a parameter here).
    fn set_sigl(&mut self, line: usize) {
        let value = self.counted_text(line);
        // Not through `assign_by_name`: that reads the name's shape and then
        // hashes it, and `SIGL` is a simple name whose slot the plan already
        // holds. The fallback covers a plan with no name map at all.
        let slot = match self.activation().plan.sigl_slot {
            Some(slot) => slot,
            None => self.slot_of(b"SIGL"),
        };
        let frame = self.activation().frame;
        self.set_variable(frame, slot, value);
    }

    /// The slot `RESULT` lives in, from the plan when it has one.
    fn reserved_result_slot(&mut self) -> usize {
        match self.activation().plan.result_slot {
            Some(slot) => slot,
            None => self.slot_of(b"RESULT"),
        }
    }

    /// Opens a stepped clause of `code`: everything
    /// [`crate::ir::Op::Clause`] owes before the clause's own work
    /// runs.
    #[inline(always)]
    #[allow(
        clippy::too_many_arguments,
        reason = "the added parameter is a zero-sized proof rather than data, and the alternative -- letting this half take the deadline itself -- is the shape whose cost the Deadline doc's table rejects"
    )]
    pub(crate) fn enter_stepped_clause(
        &mut self,
        echo: Echo,
        code: &Code<'_>,
        index: usize,
        instruction: &Instruction,
        source: Option<&ProgramSource>,
        position: (bool, crate::ir::ClausePosition),
        counted: crate::clause::DeadlineCounted,
    ) -> SteppedClause {
        // `DATE`/`TIME`'s per-clause clock cache (`activation.rs`'s own doc
        // on `Activation::clock_stale`) is invalidated **unconditionally,
        // once per call, on whichever activation is executing right now**,
        // mirroring `RexxActivation::run`'s `settings.timeStamp.valid =
        // false` right after `nextInst->execute()` returns
        // (`RexxActivation.cpp:647`). This is that same one place every
        // flat instruction position -- including one nested inside `IF`/
        // `SELECT`/`DO`/`INTERPRET`, per this function's own doc, and
        // including a callee's own body reached through `resolve_and_run_
        // call` -- passes through, so a clause's first clock read gets a
        // fresh one, any further read of the same clause sees the value
        // that read cached, and a callee's own instructions invalidate only
        // the callee's own cache rather than reaching back into the
        // caller's. The cached *value* itself (`Activation::cached_clock`)
        // is left untouched here -- `TIME('R')`'s own lazy reset needs it
        // still readable one call later, and clearing it here is what an
        // earlier version of this did instead.
        let activation = self.activation_mut();
        activation.clock_stale = true;
        // `>I>`'s own "am I still on the first instruction" decay
        // (`TraceEntry`, `activation.rs`), spent **here** and not in
        // `run_activation`'s loop. `RexxActivation.cpp:657`-`659` clears the
        // C++'s flag at the bottom of its instruction loop, and an `IF`'s
        // then-clause, a `DO` body clause and a fragment's clauses are each a
        // separate instruction in that loop -- where here they are nested
        // inside their enclosing clause's step. This function is the one
        // place a clause is stepped at any depth, so it is the only site that
        // counts them all. Measured before the move: `if 1=1 then trace l` as
        // a routine's first clause announced the pair here and nothing on the
        // oracle.
        activation.trace_entry = activation.trace_entry.stepped();
        // `TRACE`'s own `*-*` clause echo (D17), and the single insertion
        // point for it -- exactly the analogue of `eval`'s own split from
        // `eval_node`, since this is the one place `run_bounded`'s loop
        // (this function's only non-test caller) visits every flat
        // instruction position it steps, markers (`Then`/`Else`/
        // `Otherwise`/`When`/`WhenCase`/`Label`) included, matching the
        // oracle's own `RexxInstruction::traceInstruction`, which every one
        // of those calls too from its own `execute`.
        let (tabled, position) = position;
        let shortcut = tabled && source.is_some() && self.clause_line_override.is_none();
        let indent = if shortcut {
            position.indent as usize + self.activation_indent + self.indent_offset
        } else {
            self.printed_indent(code, index)
        };
        debug_assert_eq!(
            indent,
            self.printed_indent(code, index),
            "the chunk's indent table disagrees with the plan's for instruction {index}"
        );
        self.clause_state.current_value_indent = indent;
        // Set here with the indent, and for the same reason that one is: this
        // is the one place every stepped instruction passes before its `step`
        // call runs, which is where the oracle reads it too. See the field.
        self.clause_state.instructions_traced_at_entry = self.trace_mode().all;
        // Set unconditionally, exactly like `current_value_indent` just
        // above and for the identical reason (that field's own doc comment):
        // `SIGL` (`lib.rs`'s doc on `current_clause_line`) has to stay
        // correct whether or not `TRACE` is on, and this is the one place
        // every stepped instruction, `SIGNAL`/`CALL` included, passes
        // through before its own `step` call runs.
        // `enter_clause` rather than a bare assignment: the clause line and
        // the clause boundary are one operation (`clause.rs`), and the
        // `ClauseEntry` this hands back is what `leave_stepped_clause` spends
        // on the matching half.
        let line = if shortcut {
            position.line as usize
        } else {
            self.clause_line_at(code, index, instruction, source)
                .unwrap_or_else(|| self.clause_state.line())
        };
        debug_assert_eq!(
            line,
            self.clause_line_at(code, index, instruction, source)
                .unwrap_or_else(|| self.clause_state.line()),
            "the chunk's line table disagrees with the plan's for instruction {index}"
        );
        // The frame readers' own view of this clause
        // ([`crate::activation::ClauseSnapshot`], which carries where this
        // has to be and why the line cannot come from `Activation::pc`).
        // **Skipped while a fragment is running**, so the enclosing
        // `INTERPRET` clause stays in force for this activation -- the
        // answer the oracle gives the frame beneath its own
        // `FRAME_INTERPRET` one.
        if self.clause_line_override.is_none() {
            self.clause_state.current_clause_index = index;
        }
        let entry = self.enter_clause(line, counted);
        // **`Echo::Gated` asks whether the setting in force echoes this
        // clause; `Echo::Compiled` is a clause whose chunk already answered
        // that**, and emits the echo as an op of its own
        // (`crate::ir::Op::TraceClause`) rather than here. The whole point of
        // the second arm is that a chunk compiled under a setting that does
        // not echo pays nothing at all for the decision -- no gate, no
        // `clause_site`, no op.
        if matches!(echo, Echo::Gated) {
            self.echo_stepped_clause(source, instruction, indent);
        }
        // The debug tripwire I22 asks for, alongside `RootSet::temps_len`, its
        // one prerequisite. `SteppedClause` carries the watermark and the
        // frame across to the matching half, which is where the check reads
        // them; that type's own doc comment has what it checks and why it is
        // there rather than in `pop_frame`.
        let temps_at_entry = self.roots.temps_len();
        let frame = self.roots.push_frame();
        SteppedClause {
            entry,
            frame,
            temps_at_entry,
        }
    }

    /// Closes the stepped clause `entry` opened, around `ran` -- everything
    /// [`crate::ir::Op::Clause`] owes once the clause's own work has
    /// run.
    #[inline(always)]
    pub(crate) fn leave_stepped_clause<T: ClauseValue>(
        &mut self,
        entry: SteppedClause,
        code: &Code<'_>,
        index: usize,
        instruction: &Instruction,
        source: Option<&ProgramSource>,
        ran: Result<T, Failure>,
    ) -> Result<ClauseOutcome<T>, Failure> {
        debug_assert!(
            ran.is_err() || self.roots.temps_len() >= entry.temps_at_entry,
            "step popped below its own temps watermark ({} -> {}), so it \
             discarded roots it did not push",
            entry.temps_at_entry,
            self.roots.temps_len()
        );
        self.roots.pop_frame(entry.frame);
        if ran.is_err() {
            self.record_failure_site(code, index, source, instruction);
        }
        // **A `DO`/`LOOP`'s step is not a clause the oracle has, so it owes no
        // boundary** -- `Interp::leave_clause_without_boundary` has the
        // mechanism and the transcript. The boundaries the construct does owe
        // are opened elsewhere: a plain `DO`'s header and `END` clauses in
        // `run_loop_with_header`'s own `LoopKind::Simple` arm, and a
        // repeating loop's clauses in `run_repeating`.
        let outcome = match &instruction.kind {
            InstructionKind::Do(_) | InstructionKind::Loop(_) => {
                self.leave_clause_without_boundary(entry.entry, ran)
            }
            _ => self.leave_clause(entry.entry, code, ran),
        };
        // The clause's *own* failure came back as `Ran(Err(_))` and was
        // recorded just above; this `Err` is the boundary's, and it is the
        // same clause that owes the site. Recording it twice is harmless --
        // `record_failure_at`'s first-wins guard makes the second call a
        // no-op -- and recording it in neither place is what the measurement
        // in `Op::Clause`'s region's doc comment describes.
        if outcome.is_err() {
            self.record_failure_site(code, index, source, instruction);
        }
        outcome
    }

    /// The stepped-clause boundary for a clause that produced **neither a
    /// `Flow` nor a failure**, with nothing queued to deliver.
    #[inline(always)]
    pub(crate) fn finish_plain_clause(&mut self, entry: SteppedClause) {
        debug_assert!(
            self.roots.temps_len() >= entry.temps_at_entry,
            "step popped below its own temps watermark ({} -> {}), so it \
             discarded roots it did not push",
            entry.temps_at_entry,
            self.roots.temps_len()
        );
        self.roots.pop_frame(entry.frame);
        self.spend_clause_entry(entry.entry);
    }

    /// One stepped clause's `*-*` echo, **if the setting in force echoes a
    /// clause of that kind**, at `indent`.
    #[inline(always)]
    pub(crate) fn echo_stepped_clause(
        &mut self,
        source: Option<&ProgramSource>,
        instruction: &Instruction,
        indent: usize,
    ) {
        let is_label = matches!(instruction.kind, InstructionKind::Label { .. });
        let is_command = matches!(instruction.kind, InstructionKind::Command { .. });
        if self.tracing_clause(is_label, is_command)
            && let Some((line, text)) = self.clause_site(source, instruction)
        {
            self.trace_stepped_clause(is_label, is_command, line, indent, &text);
        }
    }

    /// The same echo with **no gate at all**, for a caller that has already
    /// decided ([`crate::ir::Op::TraceClause`], whose presence in a chunk is
    /// that decision).
    pub(crate) fn echo_compiled_clause(
        &mut self,
        source: Option<&ProgramSource>,
        instruction: &Instruction,
        indent: usize,
    ) {
        // The banner the stepped path owes too, and this path owes it for the
        // same reason: it is the first traced clause of the activation that
        // carries it. Reached when the setting was already `?R` at compile
        // time -- `RXTRACE=ON` is the case, where a `TRACE ?R` instruction
        // leaves the chunk stale and takes the stepped path instead.
        self.trace_debug_source();
        if let Some((line, text)) = self.clause_site(source, instruction) {
            let start = self.trace.len();
            crate::trace::push_clause(&mut self.trace, line, indent, &text);
            self.route_trace_line(start);
        }
    }

    /// Resolves `instruction`'s own clause (and its statically-derived
    /// indent, `static_indent`) into `self.failure_site`, first call wins,
    /// when `source` is `Some`.
    pub(crate) fn record_failure_site(
        &mut self,
        code: &Code<'_>,
        index: usize,
        source: Option<&ProgramSource>,
        instruction: &Instruction,
    ) {
        let indent = self.printed_indent(code, index);
        self.record_failure_at(source, instruction, indent);
    }

    /// Assigns `blame`'s own clause to `self.failure_site` at exactly
    /// `indent` spaces, first call wins, when `source` is `Some`.
    fn record_failure_at(
        &mut self,
        source: Option<&ProgramSource>,
        blame: &Instruction,
        indent: usize,
    ) {
        if self.failure_site.is_some() {
            return;
        }
        if let Some((line, text)) = self.clause_site(source, blame) {
            self.failure_site = Some(match self.sourceless_site(line, indent) {
                Some(site) => site,
                None => match self.compiled_method_site(line, &text, indent) {
                    Some(site) => site,
                    None => match self.required_package_site(line, &text, indent) {
                        Some(site) => site,
                        None => FailureSite::Clause { line, text, indent },
                    },
                },
            });
        }
    }

    /// The frame a level in a package a `::REQUIRES` loaded contributes, or
    /// `None` for a level in the program the command line started.
    fn required_package_site(
        &self,
        line: usize,
        text: &[u8],
        indent: usize,
    ) -> Option<FailureSite> {
        let program_id = self.running_activation()?.program_id;
        let name = self.required_paths.get(&program_id)?.as_bytes().to_vec();
        Some(FailureSite::Named {
            line,
            indent,
            text: text.to_vec(),
            name,
        })
    }

    /// The frame a level inside a method compiled from source text
    /// contributes, or `None` for a level in a program's own body.
    fn compiled_method_site(&self, line: usize, text: &[u8], indent: usize) -> Option<FailureSite> {
        let program_id = self.running_activation()?.program_id;
        let name = self.compiled_method_names.get(&program_id)?.to_vec();
        Some(FailureSite::Named {
            line,
            indent,
            text: text.to_vec(),
            name,
        })
    }

    /// The frame a level in the interpreter's own library contributes, or
    /// `None` for a level in a program's own package.
    fn sourceless_site(&mut self, line: usize, indent: usize) -> Option<FailureSite> {
        let program_id = self.running_activation()?.program_id;
        if !self.library_programs.contains(&program_id) {
            return None;
        }
        // Copied out rather than borrowed: the scope's `~id` is read through
        // `self.classes()`, which takes `&mut self`.
        let identity = self
            .activation()
            .method_identity
            .as_ref()
            .map(|identity| (identity.name.to_vec(), identity.scope));
        let text = match identity {
            Some((name, scope)) => {
                let scope = self.classes().id_string(scope).to_string();
                Raised::sourceless_method_line(&name, &scope, crate::LIBRARY_PACKAGE_NAME)
            }
            // The library's own prologue, which is a program rather than a
            // method. Reachable only if the bootstrap itself fails, which
            // ends the interpreter -- rendered rather than left to the
            // clause echo so that path does not print a library source line
            // the oracle would not.
            None => Raised::sourceless_program_line(crate::LIBRARY_PACKAGE_NAME),
        };
        Some(FailureSite::Named {
            line,
            indent,
            text,
            name: crate::LIBRARY_PACKAGE_NAME.to_vec(),
        })
    }

    /// Captures a `LEAVE`/`ITERATE` instruction's own clause site and static
    /// indent the instant it steps, before any propagation -- see
    /// `Flow::Leave`'s own doc comment for why eagerly, and `LeaveOrigin`'s
    /// own doc comment for why `indent` is computed here rather than read
    /// back later. `clause_site` is the free function `record_failure_site`
    /// itself resolves through, shared rather than duplicated: this needs
    /// the same (line, text) pair, just held onto instead of assigned to
    /// `self.failure_site` immediately, since a `LEAVE`/`ITERATE` might
    /// still be consumed by an enclosing `Do`/`Select` rather than ever
    /// becoming a failure at all.
    pub(crate) fn leave_origin(
        &self,
        code: &Code<'_>,
        index: usize,
        source: Option<&ProgramSource>,
        instruction: &Instruction,
    ) -> LeaveOrigin {
        LeaveOrigin {
            site: self.clause_site(source, instruction),
            // `+ self.indent_offset` (F-EX1's own correction to F3,
            // `lib.rs`'s own doc comment): found missing here on the
            // *second* re-verification of F-EX1's own fix, not the first --
            // a `LEAVE`/`ITERATE` inside an escaped `OTHERWISE`'s own body
            // captures its own origin indent here, not through `step_in_
            // temps_frame`'s own computation at all (`Flow::Leave`'s own
            // doc comment: eagerly, before any propagation), so it needs
            // the identical addition independently, not by inheritance.
            indent: self.printed_indent(code, index),
            // Already this instruction's own line: `Op::Clause`'s region's
            // `in_clause` set it before dispatching this `step`, through the
            // same `clause_line` call `SIGL` reads.
            clause_line: self.clause_state.line(),
        }
    }

    /// Assigns `origin`'s own captured site to `self.failure_site`, first
    /// call wins, at `origin`'s own captured indent.
    fn record_leave_failure(&mut self, origin: &LeaveOrigin) {
        self.record_failure_site_at(origin.site.clone(), origin.indent);
    }

    /// The same first-wins record from an already-resolved `(line, text)`
    /// pair rather than from an `&Instruction`, for the one caller that has
    /// no instruction left to resolve: a loop re-test blamed on the
    /// `ITERATE` that transferred control back to it, whose site was
    /// captured a pass earlier (`HeaderClause::Iterate`). `indent` is the
    /// caller's, because that blame prints at the loop body's indent rather
    /// than at the `ITERATE`'s own -- see that variant's doc comment.
    fn record_failure_site_at(&mut self, site: Option<(usize, Vec<u8>)>, indent: usize) {
        if self.failure_site.is_some() {
            return;
        }
        if let Some((line, text)) = site {
            self.failure_site = Some(FailureSite::Clause { line, text, indent });
        }
    }

    /// Closes off the level that is unwinding now, so the level above it can
    /// record its own clause.
    pub(crate) fn seal_site_level(&mut self) {
        if let Some(site) = self.failure_site.take() {
            self.failure_sites.push(site);
        }
    }

    /// The clause boundary a promoted construct owes once the branch it chose
    /// has finished, which flattening removed.
    /// ```text
    /// call on user zx name h        /* h raises zy */
    /// call on user zy name g        /* g says SIGL */
    /// select
    /// when 1 = 1 then zq = raiser()
    /// end
    /// say 'after'
    /// ```
    pub(crate) fn end_promoted_branch(
        &mut self,
        code: &Code<'_>,
        flow: Flow,
    ) -> Result<Flow, Failure> {
        let line = self.clause_state.line();
        match self.in_clause(code, line, move |_| Ok(flow))? {
            ClauseOutcome::Ran(ran) => ran,
            ClauseOutcome::Ended(exit) => Ok(Flow::Exit(exit.value())),
        }
    }

    /// Runs `code.body.instructions[start..end]` in place, one instruction at
    /// a time through `Op::Clause`'s region, and answers what happened.
    fn run_bounded(
        &mut self,
        code: &Code<'_>,
        start: usize,
        end: usize,
        source: Option<&ProgramSource>,
        engine: BodyEngine<'_>,
    ) -> Result<Flow, Failure> {
        let BodyEngine::Chunk { chunk, registers } = engine;
        self.run_bounded_from_chunk(code, chunk, registers, start, end, source)
    }

    /// An `IF`'s own condition, evaluated as the whole of the `IF` clause's
    /// work.
    fn eval_if_condition(&mut self, code: &Code<'_>, condition: &Expr) -> Result<bool, Failure> {
        let indent = self.clause_state.current_value_indent;
        self.eval_condition(
            code,
            condition,
            ConditionTrace::Result(indent),
            raised_if_not_logical,
        )
    }

    /// The expression an op's address names: expression `slot` of
    /// `instruction`, then `path`'s steps down from that slot's root.
    pub(crate) fn chunk_node_at(
        instruction: &Instruction,
        slot: u16,
        path: NodePath,
    ) -> Option<&Expr> {
        let mut node = match (&instruction.kind, slot) {
            (InstructionKind::Assignment { value, .. }, 0) => value,
            // The one expression of an instruction that computes a value and
            // does something with it, slot `0`. A bare form holds no
            // expression at all, so it names no node and matches no arm here.
            (
                InstructionKind::Say {
                    expression: Some(expression),
                }
                | InstructionKind::Return {
                    expression: Some(expression),
                }
                | InstructionKind::Exit {
                    expression: Some(expression),
                }
                | InstructionKind::Push {
                    expression: Some(expression),
                }
                | InstructionKind::Queue {
                    expression: Some(expression),
                },
                0,
            ) => expression,
            // The two instructions whose slot `0` exists only in one of their
            // forms. Both are reachable: the expression is compiled through
            // `push_value` like any other, so a call inside it emits an
            // `Op::CallExpr` that addresses the node from here.
            (InstructionKind::Signal(signal), 0) => match &**signal {
                rexx_parse::Signal::Value(expression) => expression,
                rexx_parse::Signal::Label(_) | rexx_parse::Signal::Trap(_) => return None,
            },
            (
                InstructionKind::Parse(parse)
                | InstructionKind::Arg(parse)
                | InstructionKind::Pull(parse),
                0,
            ) => match &parse.source {
                rexx_parse::ParseSource::Value(Some(expression)) => expression,
                _ => return None,
            },
            (InstructionKind::If { condition, .. }, 0) => condition,
            // A **plain** `WHEN`'s condition. A `WhenCase`'s values are not a
            // condition and compile to no native op at all, so no address ever
            // names one and this arm does not answer for them.
            (InstructionKind::When { condition, .. }, 0) => condition,
            // A `DO`/`LOOP` header's `slot`th expression, resolved through the
            // same `loop_header_slot` the compiler emitted the slot's ops from
            // and the same one `eval_chunk_expr` evaluates a declining slot
            // with. Every slot of a header is addressable, not just one, which
            // is why this arm binds `slot` rather than matching a number.
            (InstructionKind::Do(body) | InstructionKind::Loop(body), slot) => {
                loop_header_slot(body, u32::from(slot))?
            }
            _ => return None,
        };
        for right in path.steps() {
            node = match (&node.kind, right) {
                (ExprKind::Binary { left, .. }, false) => left,
                (ExprKind::Binary { right, .. }, true) => right,
                (ExprKind::Prefix { operand, .. }, false) => operand,
                _ => return None,
            };
        }
        Some(node)
    }

    /// Expression `slot` of `instruction`, evaluated as the compiled stream's
    /// [`crate::ir::Op::EvalExpr`] asks.
    pub(crate) fn eval_chunk_expr(
        &mut self,
        code: &Code<'_>,
        instruction: &Instruction,
        slot: u32,
    ) -> Result<ObjRef, Failure> {
        match (&instruction.kind, slot) {
            (InstructionKind::If { condition, .. }, 0) => {
                let holds = self.eval_if_condition(code, condition)?;
                // In range unconditionally: `SMALL_INT_MAX` is far above one.
                Ok(ObjRef::small_int(i64::from(holds)).unwrap_or(ObjRef::NIL))
            }
            (
                InstructionKind::Select {
                    case: Some(case_expr),
                    ..
                },
                0,
            ) => self.select_case(code, case_expr),
            // A `DO`/`LOOP` header's `slot`th expression, in the order
            // `loop_header_plan` puts them in -- the order they were written,
            // which is the order they are evaluated. The expression is
            // evaluated and nothing else: its `>K>` echo and its validation are
            // ops of their own, because both have to happen before the next
            // expression is evaluated at all.
            (InstructionKind::Do(body) | InstructionKind::Loop(body), slot) => {
                let Some(expr) = loop_header_slot(body, slot) else {
                    return Err(Loud::instruction(&instruction.kind).into());
                };
                self.eval(code, expr)
            }
            // An `Assignment`'s value, and the one expression of a `SAY`, a
            // `RETURN`, an `EXIT`, a `PUSH` or a `QUEUE`, slot `0`: whatever
            // the expression came to, unvalidated and untagged. Each reaches
            // this arm only for an expression `compile` did not emit a native
            // op for -- a literal is `crate::ir::Op::Const` and a bare symbol
            // is `crate::ir::Op::Load` instead -- and each is trace-identical
            // to arm here because this is the same
            // `eval` call it makes.
            // `SIGNAL VALUE`'s own expression, for the shapes `compile` emits
            // no native op for. The `>K>` echo and the search are
            // `Op::Signal`'s, not this arm's, exactly as a loop header's
            // validation is its own op.
            (InstructionKind::Signal(signal), 0) => {
                let rexx_parse::Signal::Value(expr) = &**signal else {
                    return Err(Loud::instruction(&instruction.kind).into());
                };
                self.eval(code, expr)
            }
            // `PARSE VALUE expr WITH`'s own expression, on the same terms. The
            // `>K>` echo and the whole template walk are `Op::Parse`'s; only
            // the source is a register, and only this source has one --
            // `compile` emits no register for any other, so reaching here with
            // one is an op off its node.
            (
                InstructionKind::Parse(parse)
                | InstructionKind::Arg(parse)
                | InstructionKind::Pull(parse),
                0,
            ) => {
                let rexx_parse::ParseSource::Value(Some(expr)) = &parse.source else {
                    return Err(Loud::instruction(&instruction.kind).into());
                };
                self.eval(code, expr)
            }
            (InstructionKind::Assignment { value, .. }, 0) => self.eval(code, value),
            (
                InstructionKind::Say {
                    expression: Some(expression),
                }
                | InstructionKind::Return {
                    expression: Some(expression),
                }
                | InstructionKind::Exit {
                    expression: Some(expression),
                }
                | InstructionKind::Push {
                    expression: Some(expression),
                }
                | InstructionKind::Queue {
                    expression: Some(expression),
                },
                0,
            ) => self.eval(code, expression),
            (kind, _) => Err(Loud::instruction(kind).into()),
        }
    }

    /// Evaluates `condition` and answers whether it holds, for `IF`/`WHEN`.
    fn eval_condition(
        &mut self,
        code: &Code<'_>,
        condition: &Expr,
        trace: ConditionTrace<'_>,
        raise: fn(&[u8]) -> Raised,
    ) -> Result<bool, Failure> {
        let value = self.eval(code, condition)?;
        let checked = matches!(condition.kind, ExprKind::Logical(_));
        self.condition_value(value, trace, checked, raise)
    }

    /// Everything an evaluated condition value still owes: its `>>>` or `>K>`
    /// line, and the answer to whether it holds.
    pub(crate) fn condition_value(
        &mut self,
        value: ObjRef,
        trace: ConditionTrace<'_>,
        checked: bool,
        raise: fn(&[u8]) -> Raised,
    ) -> Result<bool, Failure> {
        // **The test's own temps frame, and it is a per-pass frame for the two
        // callers that are loop headers.** `WHILE` and `UNTIL` re-evaluate
        // their condition once per pass inside the enclosing `DO`
        // instruction's single frame, so the push below accumulates one
        // `ObjRef` per pass -- and one pinned heap object per pass whenever
        // the condition's result is one -- for the whole of the loop's run.
        // The value is never handed back (this answers a `bool`), so a frame
        // closed here releases it at the right time for every caller,
        // `IF`/`WHEN` included. The `?` path below leaves the pop to the outer
        // truncation, as `pop_frame`'s own doc describes.
        let frame = self.roots.push_frame();
        self.roots.push_temp(value);
        // **The owned copy is taken only when a line will print it.** Both
        // formatters below return at once unless `results` is on, and the copy
        // exists only because `to_text` borrows `self` while they need it
        // mutably -- so off that path the borrow is enough and the answer is
        // decided from it directly. Measured with `heaptrack` on
        // `samples/rexxcps.rex`: this was the largest single allocation site
        // in the interpreter, one copy per condition evaluated.
        let decided = if self.trace_mode().results {
            let text = self.to_text(value).to_vec();
            match trace {
                // `IF`/plain `WHEN`'s own `>>>` (`IfInstruction.cpp:140`, and
                // `select` / `when 1 = 1 then` measured to show the identical
                // shape) -- measured, `WHILE`/`UNTIL` never get this (this
                // task's report: `trace r` over a `DO WHILE` shows only `>K>
                // "WHILE" => ...`, no bare `>>>` alongside it), which is why
                // this is a variant a caller picks rather than something
                // `eval_condition` decides on its own. `SELECT CASE`'s own
                // `WHEN`/`WhenCase` comparison never reaches this function at
                // all -- see `test_case_when`'s own trace calls instead.
                ConditionTrace::Result(indent) => self.trace_result(indent, &text),
                // `WHILE`/`UNTIL`'s own `>K>` (`DoBlockComponents.cpp`'s
                // `traceKeywordResult(WHILE, ...)`/`(UNTIL, ...)`) -- re-fires
                // every pass because the oracle re-evaluates the condition every
                // pass too, which `run_repeating`'s own call site already does
                // without any change from this task.
                ConditionTrace::Keyword(indent, keyword) => {
                    self.trace_keyword(indent, keyword, &text);
                }
            }
            Self::condition_holds(checked, &text, raise)
        } else {
            // **Off the tracing path the bytes need not be moved anywhere.**
            // `to_text` would copy them into the interpreter's scratch slot
            // and hand back a borrow of it; for a value that carries its own
            // bytes that copy buys nothing. A condition's value is one byte --
            // `0` or `1` -- far more often than it is anything else, and one
            // byte always rides in the handle.
            match value.decode() {
                Decoded::Text(inline) => Self::condition_holds(checked, &inline, raise),
                // `to_text` renders these two as `"0"` and `"1"`, which both
                // arms of `condition_holds` then read as this same answer.
                // Every other integer falls to the arm below, so the failure
                // it raises still names the value as the oracle spells it.
                Decoded::SmallInt(number) if number == 0 || number == 1 => Ok(number == 1),
                _ => {
                    let text = self.to_text(value);
                    Self::condition_holds(checked, &text, raise)
                }
            }
        };
        // After the rendering above is out of scope, so the borrow it may hold
        // on `self` has ended. The decision itself touches neither `self` nor
        // the roots, so making it before this pop rather than after changes no
        // answer and no lifetime.
        self.roots.pop_frame(frame);
        decided
    }

    /// Whether a condition's rendered text holds, and the failure when it is
    /// neither `0` nor `1`.
    fn condition_holds(
        checked: bool,
        text: &[u8],
        raise: fn(&[u8]) -> Raised,
    ) -> Result<bool, Failure> {
        if checked {
            Ok(text == b"1")
        } else {
            logical_value(text).ok_or_else(|| raise(text).into())
        }
    }

    /// Whether any of a `WHEN CASE`'s `values` compares `==` (byte-for-byte,
    /// no padding, no numeric awareness) equal to the `SELECT CASE`'s own
    /// `case_text`, matching on the first that does (an OR of `==`, the
    /// opposite of a plain `WHEN`'s comma list, which is an AND checked for
    /// `0`/`1` -- `ast.rs`'s own doc comment on `WhenCase`).
    fn test_case_when(
        &mut self,
        code: &Code<'_>,
        values: &[Expr],
        case_text: &[u8],
        indent: usize,
    ) -> Result<bool, Failure> {
        for value in values {
            let value = self.eval(code, value)?;
            self.roots.push_temp(value);
            let text = self.to_text(value).to_vec();
            self.trace_result(indent, &text);
            let matched = text == case_text;
            self.trace_result(indent, if matched { b"1" } else { b"0" });
            if matched {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// `instruction`'s own clause text and the 1-based line to print it against,
    /// or `None` when `source` is `None`.
    pub(crate) fn clause_site(
        &self,
        source: Option<&ProgramSource>,
        instruction: &Instruction,
    ) -> Option<(usize, Vec<u8>)> {
        let line = self.clause_line(source, instruction)?;
        let source = source?;
        Some((
            line,
            source
                .join_span(instruction.clause_span.clone())
                .map_or_else(
                    // Visible rather than silent, matching `Raised::message`'s
                    // own reasoning for a catalogue miss: the error path is the
                    // worst place to turn a reportable condition into a crash or
                    // a blank line.
                    || b"<clause span outside the retained source>".to_vec(),
                    |bytes| bytes.into_owned(),
                ),
        ))
    }

    /// `clause_site`'s own line half, alone -- extracted so `current_clause_
    /// line` (`lib.rs`'s own doc comment on the field) can be kept fresh on
    /// every step without paying for `clause_site`'s own `join_span` text
    /// extraction, which nothing needs when only `SIGL`'s value is being
    /// computed. Identical rule, same `clause_line_override` honoured the
    /// same way, so a `SIGNAL`/`CALL` fired from inside an `INTERPRET`
    /// fragment reads the enclosing clause's own line here exactly as
    /// `clause_site` already gives the trace/error paths.
    pub(crate) fn clause_line(
        &self,
        source: Option<&ProgramSource>,
        instruction: &Instruction,
    ) -> Option<usize> {
        let source = source?;
        Some(
            self.clause_line_override
                .unwrap_or_else(|| source.line_of(instruction.clause_span.start)),
        )
    }

    /// [`Interp::clause_line`] for a caller that knows the instruction's
    /// **index**, which is what lets the answer come from `Plan::lines`
    /// rather than from a search.
    pub(crate) fn clause_line_at(
        &self,
        code: &Code<'_>,
        index: usize,
        instruction: &Instruction,
        source: Option<&ProgramSource>,
    ) -> Option<usize> {
        let source = source?;
        if let Some(line) = self.clause_line_override {
            return Some(line);
        }
        debug_assert!(
            code.body
                .instructions
                .get(index)
                .is_some_and(|at| std::ptr::eq(at, instruction)),
            "clause_line_at was given index {index} and an instruction that is not the one at \
             that index, so the table would be read for a different clause"
        );
        Some(match code.plan {
            Some(plan) => plan.line_at(instruction, source, index),
            None => source.line_of(instruction.clause_span.start),
        })
    }

    // `fragment_plan` and `slot_of` live in `plan.rs` (Task 6), beside `Plan`
    // itself; `stem_assign`/`stem_set`/`stem_drop`/`stem_drop_tail`/
    // `tail_key` live in `stem.rs` (Task 5), beside the rest of the D15a
    // library. `read` lives in `variables.rs`, beside `Interp`'s other variable
    // entry points.
}

/// Which of the three variable shapes `name`'s own spelling is, from an
/// already-interned (or already-upcased runtime) name alone.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) enum NameShape {
    Simple,
    Stem,
    Compound,
}

pub(crate) fn shape_of(name: &[u8]) -> NameShape {
    let dots = name.iter().filter(|&&b| b == b'.').count();
    if dots == 0 {
        NameShape::Simple
    } else if dots == 1 && name.last() == Some(&b'.') {
        NameShape::Stem
    } else {
        NameShape::Compound
    }
}

/// Splits an indirect wrapper's value into its subsidiary list's words --
/// `DROP (v)`, `EXPOSE (v)` and `PROCEDURE EXPOSE (v)` all spell the same
/// list and reach the same split.
fn split_indirect_words(text: &[u8]) -> impl Iterator<Item = &[u8]> {
    text.split(|&b| b == b' ' || b == b'\t')
        .filter(|word| !word.is_empty())
}

/// One legal symbol character, by the scanner's own character table
/// (`scanner.rs`'s `is_symbol_char`: `! . ? _ 0-9 A-Z a-z`, ASCII only --
/// `SymbolTable::intern`'s own doc says a non-ASCII byte cannot be part of a
/// symbol at all).
fn is_symbol_byte(b: u8) -> bool {
    matches!(b, b'!' | b'.' | b'?' | b'_' | b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z')
}

/// Validates one word of an indirect subsidiary list and answers its upcased
/// name, or the condition the oracle raises for it.
fn validate_indirect_word(word: &[u8]) -> Result<Vec<u8>, Failure> {
    if !word.iter().copied().all(is_symbol_byte) {
        return Err(raised_symbol_expected(word).into());
    }
    match word[0] {
        b'0'..=b'9' => return Err(raised_digit_led(word).into()),
        b'.' => return Err(raised_dot_led(word).into()),
        _ => {}
    }
    Ok(word.to_ascii_uppercase())
}

/// Whether `index` is a clause of `body`'s own top level -- reached by
/// falling from the instruction before it, with no `DO`, `LOOP`, `SELECT` or
/// `IF` construct enclosing it.
fn top_level_clause(body: &CodeBody, index: usize) -> bool {
    let instructions = &body.instructions;
    let len = instructions.len();
    let mut at = 0;
    while at < len {
        if at == index {
            return true;
        }
        at = match &instructions[at].kind {
            InstructionKind::Do(body) | InstructionKind::Loop(body) => {
                body.end.map_or(len, |end| end + 1)
            }
            InstructionKind::Select { end, .. } => end.map_or(len, |end| end + 1),
            InstructionKind::If { false_target, .. } => match false_target {
                Some(target) => skip_else(instructions, *target),
                None => len,
            },
            _ => at + 1,
        };
    }
    false
}

#[cfg(test)]
mod tests;

impl Interp {
    /// `RexxActivation::resolveStream`: the object a stream builtin sends its
    /// message to. An omitted or empty name and `STDIN`/`STDOUT`/`STDERR`
    /// (caselessly, with or without a trailing colon) are the standard
    /// streams; anything else is qualified and looked up in the owning
    /// activation's table by that qualified name, and a miss builds a
    /// `Stream` with the name **as written** -- `~string` answers what the
    /// program passed, while the key is the resolved path, so `a.txt` and
    /// `./a.txt` are one entry.
    ///
    /// `added` says whether a hit-or-miss may populate the table: every
    /// builtin passes it except `STATE`, `DESCRIPTION` and a `STREAM` command
    /// that is not OPEN, CLOSE or SEEK, which is why `stream(n,'S')` on an
    /// unknown name leaves nothing behind.
    pub(crate) fn resolve_stream(
        &mut self,
        name: &[u8],
        input: bool,
        added: bool,
    ) -> Result<ObjRef, Failure> {
        if let Some(standard) = standard_stream_name(name, input) {
            return self.dot_variable(standard);
        }
        let cwd = self.cwd_text();
        let qualified = crate::paths::normalize(&String::from_utf8_lossy(name), &cwd).into_bytes();
        if let Some(found) = self
            .stream_table()
            .and_then(|table| table.get(qualified.as_slice()).copied())
        {
            return Ok(found);
        }
        // **The manager sees the qualified name, and only a table miss.**
        // `.Stream~new` reaches none of this, so a program that builds its
        // own stream object is never checked.
        if let Some(replacement) = self.check_stream_access(&qualified)? {
            if let Some(table) = self.stream_table_mut() {
                table.insert(qualified.into_boxed_slice(), replacement);
            }
            return Ok(replacement);
        }
        let Some(class) = self.rexx_package_class(b"STREAM") else {
            // Only before `StreamClasses.orx` has installed, which is the
            // library bootstrap's own step.
            return Err(Loud::environment_symbol(b".STREAM", "Phase 5").into());
        };
        let argument = self.text(name);
        self.roots.push_temp(argument);
        let caller = self.caller();
        let built = self
            .send_message(class, b"NEW", None, &[Some(argument)], caller)?
            .ok_or_else(|| Failure::from(Raised::no_result(b"NEW")))?;
        if added && let Some(table) = self.stream_table_mut() {
            table.insert(qualified.into_boxed_slice(), built);
        }
        Ok(built)
    }
}

impl Interp {
    /// The security manager's `STREAM` checkpoint
    /// (`execution/SecurityManager.cpp:290`-`:308`): the object to use in
    /// place of a new `Stream`, or `None` where the manager declined or none
    /// is installed.
    ///
    /// **The manager's entry is `STREAM`, not `RESULT`.**
    fn check_stream_access(&mut self, qualified: &[u8]) -> Result<Option<ObjRef>, Failure> {
        if self.effective_security_manager().is_none() {
            return Ok(None);
        }
        let name = self.text(qualified);
        self.roots.push_temp(name);
        let entries = [(crate::security::key::NAME, name)];
        let Some(info) = self.security_check(crate::security::message::STREAM, &entries)? else {
            return Ok(None);
        };
        self.security_entry(info, crate::security::key::STREAM)
    }

    /// Drops `name`'s entry from the owning activation's table, which is what
    /// `LINEOUT(name)` with no string and a `CLOSE` command do after their
    /// send: the stream stays closed and the next builtin to name it builds a
    /// fresh one (`RexxActivation::removeFileName`).
    pub(crate) fn forget_stream(&mut self, name: &[u8]) {
        let cwd = self.cwd_text();
        let qualified = crate::paths::normalize(&String::from_utf8_lossy(name), &cwd).into_bytes();
        if let Some(table) = self.stream_table_mut() {
            table.remove(qualified.as_slice());
        }
    }
}

/// The `.local` name a stream name spells, or `None` for an ordinary one.
/// Matched caselessly with an optional trailing colon, and an omitted or empty
/// name is the default input or output rather than a file
/// (`RexxActivation.cpp:1938-2041`).
///
/// **The monitors, not the streams behind them.** `resolveStream` answers
/// `.INPUT`, `.OUTPUT` and `.ERROR`, so a redirected route is followed: measured
/// with `.output~destination(.stderr)` pushed, the oracle puts `SAY`,
/// `LINEOUT(,)` and `CHAROUT(,)` on stderr while `.stdout~lineout` stays on
/// stdout. Note `STDERR` reaches `.ERROR` rather than an `.STDERR` monitor,
/// which does not exist.
fn standard_stream_name(name: &[u8], input: bool) -> Option<&'static [u8]> {
    let bare = name.strip_suffix(b":").unwrap_or(name);
    // An omitted or empty name is the default input or output, and which one
    // depends on the caller -- measured with stdin redirected from a file,
    // `linein()` reads `l1` and `chars()` answers 9, while `charout( ,'W')`
    // writes `W` to stdout.
    if bare.is_empty() {
        return Some(if input { b".INPUT" } else { b".OUTPUT" });
    }
    if bare.eq_ignore_ascii_case(b"STDIN") {
        return Some(b".INPUT");
    }
    if bare.eq_ignore_ascii_case(b"STDOUT") {
        return Some(b".OUTPUT");
    }
    if bare.eq_ignore_ascii_case(b"STDERR") {
        return Some(b".ERROR");
    }
    None
}
