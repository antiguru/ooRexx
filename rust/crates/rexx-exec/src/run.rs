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
//!
//! `run_activation`, `step`, `step_in_temps_frame` and `run_fragment` moved
//! here from Task 3's spike (`lib.rs`), `Flow` alongside them. This is where
//! the design's "The borrow shape" is actually written down: clone the `Rc`
//! into a local on entry, and derive every `&CodeBody` and `&Expr` from that
//! local rather than from `self`. `run_activation`'s own doc comment carries
//! the argument in full, including the version that does not compile, and
//! none of it changed in the move -- `lib.rs` keeps `Code` itself, `Interp`,
//! the entry point and the loud-failure catalogue, which is what makes the
//! move safe: every field and every sibling method this file's functions
//! reach into is already visible here, exactly as it was in the crate root.
//!
//! Task 9 is the first task to extend `step` rather than only prove its
//! shape. `SAY` and `Assignment`'s `Variable` target, `EXIT` with no result,
//! and `INTERPRET` under the spike flag are the spike's own witnesses and
//! move unchanged. New here: `Assignment`'s `Stem`/`Compound` targets, all of
//! `DROP`, all six spellings of `NUMERIC`, `EXIT` with a result, and `LABEL`/
//! `NOP` as the no-ops they are.
//!
//! Task 10 is the first to make `Flow::Goto` live, for `IF`/`SELECT`/
//! `SELECT CASE`. **`If` and `Select` each resolve their whole construct
//! inside their own `step` arm**, running the winning branch through a
//! bounded local loop (`run_bounded`, shaped like `run_fragment`'s) rather
//! than leaving the outer `run_activation` loop to fall through the flat
//! instruction list on its own. That is not a style choice: Phase 3 elides
//! the C++'s synthetic end-of-branch markers, so `false_target` on an `If`
//! lands exactly on its `Else` (never past it), and a matched branch's own
//! completion, via plain `pc += 1`, lands on that exact same index -- the
//! true and false arrivals are indistinguishable from `(instruction, pc)`
//! alone, and only one of the two is supposed to enter the `Else`'s body.
//! `SELECT`/`WHEN` has the same defect with no marker to disambiguate with
//! at all. `run_bounded`'s doc comment carries the resolution; `Then`,
//! `Else`, `Otherwise`, `When` and `WhenCase` accordingly step as pure
//! no-ops (like `Label`) -- they are never independently dispatched, only
//! ever read as data by `If`/`Select` or walked over inside a bounded loop.

use crate::error::Raised;
use crate::eval::logical_value;
use crate::{Code, Failure, Interp, Loud};
use rexx_core::ObjRef;
use rexx_num::SettingsError;
use rexx_parse::{
    EndStyle, Expr, ExprKind, Fragment, Instruction, InstructionKind, NumericSetting,
    ProgramSource, VariableRef, compound_parts, parse_interpret,
};
use std::rc::Rc;

/// Where control goes after one instruction (the design's "Control flow").
enum Flow {
    Next,
    /// Unreachable in a fragment for a measured reason: a label inside
    /// `INTERPRET` text is error 47.1 (Task 1), so a fragment's `labels` is
    /// always empty and a fragment can never jump -- `run_fragment`'s own
    /// `unreachable!` on this variant still holds. Live everywhere else since
    /// Task 10: `If`/`Select` each resolve to one `Goto` that skips straight
    /// to their construct's true resume point, and `run_bounded`'s own
    /// internal loop applies one whenever a nested construct's target lands
    /// inside its range.
    Goto(usize),
    Exit(Option<ObjRef>),
}

impl Interp {
    // ---- the instruction loop, which is what this spike is for ----

    /// Runs the current activation's body to completion.
    ///
    /// **This function is the architectural claim.** The discipline is: clone
    /// the `Rc` into a local on entry, and derive every `&CodeBody` and
    /// `&Expr` from that local. It compiles for exactly one reason, and the
    /// reason is that `code` borrows `program` and `plan`, which are locals,
    /// rather than borrowing `self` -- so `self.step(…)`, which takes
    /// `&mut self`, has nothing to collide with.
    ///
    /// The version that does **not** compile, kept because the next phase to
    /// touch this will want to know which shape is wrong. Reaching the body
    /// through the activation and then stepping:
    ///
    /// ```text
    /// fn run_activation_wrong(&mut self) -> Result<Option<ObjRef>, Loud> {
    ///     let body = &self.activations.last().expect("a live activation").program.main;
    ///     while let Some(instruction) = body.instructions.get(self.activation().pc) {
    ///         self.step_wrong(body, instruction)?;
    ///     }
    ///     Ok(None)
    /// }
    /// ```
    ///
    /// The block below was captured by hand: the wrong version was written
    /// into this file, built, and deleted again, and this is what rustc 1.96.1
    /// printed. **Nothing re-checks it.** If the borrow checker's wording or
    /// its choice of underline changes, this text goes stale and no test
    /// fails, which is exactly why the doctests further down exist. Read it as
    /// a record of what was seen once, not as an assertion about what rustc
    /// does now:
    ///
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
    ///
    /// Worth reading the second underline rather than the third: the borrow
    /// that survives is not the argument, it is the **loop condition**.
    /// Compiled here too, rather than reasoned about: replacing the body of
    /// the loop with a `self.step_wrong_noargs()` that takes no arguments at
    /// all gives the identical `E0502`, with `while body.instructions.get(…)`
    /// underlined as the later use. So handing the body to `step` some other
    /// way is not the fix. `Rc::clone` is the fix, because what has to change
    /// is where `body` is rooted, not who it is passed to.
    ///
    /// The `Rc::clone` is what removes it, and it is not a workaround for the
    /// borrow checker being conservative: the checker is right. An activation
    /// can be replaced under a running loop, and a `&CodeBody` reached through
    /// the activation would then point into a program the activation no longer
    /// holds. The `Rc` in the local is what makes that impossible rather than
    /// merely unlikely.
    ///
    /// # The pair of doctests below, and what they are worth
    ///
    /// **What keeps the function honest is the compiler.** This function would
    /// not build if the discipline broke, which is the whole reason the
    /// discipline is worth having and is a stronger guarantee than any test.
    /// **What the pair below keeps honest is the documentation**, which is the
    /// part with no compiler behind it: everything above is prose, and nothing
    /// in the tree would notice if it stopped describing anything real.
    ///
    /// So the two snippets are a miniature of the same borrow, once in the
    /// shape that compiles and once in the shape that does not. `cargo test`
    /// runs both. A precondition that nothing states and that the pair
    /// silently depends on: **rustdoc does collect doctests on private
    /// items.** Confirmed rather than assumed, by putting a deliberately
    /// failing snippet on a private method and watching it run. If it did not,
    /// both of these would not exist rather than fail, and the pair would be
    /// decoration.
    ///
    /// **Read them for what they prove and not more.** `compile_fail` proves
    /// only "this does not compile", not "this fails with `E0502`". The
    /// `compile_fail,E0502` spelling looks like it pins the code and does not:
    /// measured on rustc 1.96.1, a doctest annotated `compile_fail,E0502`
    /// whose body is `let x: u32 = "not a u32";` passes, and that is `E0308`.
    /// A `compile_fail` snippet with a typo in it therefore passes for the
    /// wrong reason, which is the standard trap with this attribute.
    ///
    /// What narrows it is the **first** snippet, which must compile. The two
    /// differ only in how `body` is obtained, two lines against one, and share
    /// everything else, so any breakage in the shared part fails the passing
    /// twin instead of silently satisfying the failing one. Checked by
    /// mutation rather than assumed: rewriting the passing snippet's
    /// `Rc::clone` line into the failing snippet's shape makes it fail, and it
    /// fails with `E0502`. One test says "the fix works", the other says "the
    /// shape it fixes is still broken", and neither is worth much alone.
    ///
    /// Two residuals, the second larger than the first and worth stating
    /// because the pair is easy to over-read.
    ///
    /// A typo confined to the failing snippet's own `let body = …` line passes
    /// for the wrong reason and nothing here closes that.
    ///
    /// And **the miniature can drift away from this function.** It models
    /// `Rc<CodeBody>` over a `Vec<u32>` with a two-argument `step`, where the
    /// real thing has `Rc<Program>`, a three-field `Code<'a>` and a different
    /// `step`. Nothing ties the two together. A future rewrite of
    /// `run_activation` into a shape the miniature does not model leaves both
    /// doctests green while the prose above them describes a function that no
    /// longer exists. That is a real limit and not a reason to drop the pair:
    /// the compiler is still what stops the *function* going wrong, and the
    /// pair is still what stops this *comment* claiming a borrow error that
    /// the language no longer produces.
    ///
    /// Compiles, because `body` borrows the local `program`:
    ///
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
    ///
    /// Does not compile, because `body` borrows `self`. One line different:
    ///
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
    pub(crate) fn run_activation(&mut self) -> Result<Option<ObjRef>, Failure> {
        // `code` is bound to the activation on top of the stack at entry,
        // while every `pc` read and write below goes to whatever is on top
        // *now*. Those are the same frame only because `step` leaves the
        // activation stack as it found it, which is true in 4a because
        // nothing here pushes one, and true for a fragment because it runs
        // inside the creating activation rather than pushing its own.
        //
        // **4b breaks that and will not be told so by the compiler.** A
        // `CALL` pushes an activation inside `step`, and if it ever returned
        // with the callee still on the stack, this loop would carry on
        // reading the callee's `pc` while executing the caller's body: a
        // wrong answer, not a borrow error, because both are plain field
        // accesses on `self`. The assertion below is what turns that into a
        // failure at the first instruction instead of a debugging session.
        //
        // The body is `program.main` and the activation does not record which
        // body it is running, which is the other half of the same assumption.
        // 4b's activation for a `::routine` needs that field; without it, such
        // an activation would re-run the main body here. See `Activation`.
        let program = Rc::clone(&self.activation().program);
        let plan = Rc::clone(&self.activation().plan);
        let code = Code {
            body: &program.main,
            symbols: &program.symbols,
            slots: &plan.by_symbol,
        };
        let depth = self.activations.len();

        while let Some(instruction) = code.body.instructions.get(self.activation().pc) {
            let index = self.activation().pc;
            // The failing clause's site, if any escapes, is resolved inside
            // `step_in_temps_frame` itself (Task 10's own doc comment there):
            // this call may nest arbitrarily deep through `If`/`Select`'s own
            // `run_bounded`, and only the *innermost* one has the failing
            // instruction in hand. `run` pops this activation on the way out,
            // so a site resolved any higher up than that would have nothing
            // left to resolve against.
            //
            // A condition raised inside an `INTERPRET` fragment arrives here
            // too, and records the enclosing `INTERPRET` clause rather than
            // the fragment's own, because `run_fragment` passes `None` for
            // `source` and deliberately does not record: its spans index the
            // fragment's own source, not this one.
            //
            // The oracle prints **both**, innermost first, each carrying the
            // enclosing clause's line number (measured, `interpret "say 2 &
            // 1"` on line 2):
            //
            // ```text
            //      2 *-* say 2 & 1
            //      2 *-* interpret "say 2 & 1"
            // ```
            //
            // so this reproduces the second of those lines and not the
            // first. A known gap rather than something to fix here: stacking
            // one echo per nesting level changes `Raised::report`'s shape,
            // and 4a's only nesting is the fragment spike that 4b deletes.
            let flow =
                self.step_in_temps_frame(&code, index, instruction, Some(&program.source))?;
            match flow {
                Flow::Next => self.activation_mut().pc += 1,
                Flow::Goto(target) => self.activation_mut().pc = target,
                Flow::Exit(value) => return Ok(value),
            }
            debug_assert_eq!(
                self.activations.len(),
                depth,
                "step left the activation stack changed, so this loop's `code` and its `pc` \
                 no longer describe the same frame"
            );
        }
        Ok(None)
    }

    /// Runs one instruction.
    ///
    /// `code` is the caller's, so everything reached through it outlives every
    /// `&mut self` call in here. The `Assignment` arm is the clearest case:
    /// `name` is a `&[u8]` pulled out of `code.symbols` and it stays valid
    /// across `self.eval(…)`, which the same slice read out of `self` would
    /// not.
    ///
    /// **Every `eval` result is rooted before anything else runs.** `eval`
    /// hands back an unrooted handle by design, so its caller owns the moment
    /// it becomes a root, and here that is a `push_temp` on the line after
    /// each call. The instruction loops open a temps frame around this call
    /// and close it after, so what is pushed here lives exactly one clause.
    /// Not doing it would happen to work today, because `Heap::alloc_with`
    /// never collects, and would become a use-after-free the day it does,
    /// found by chasing a wrong value rather than by a compiler message.
    ///
    /// `index` is `instruction`'s own position in `code.body.instructions`,
    /// added for Task 10: `If` and `Select` need their own position to
    /// compute a branch's start (`index + 1`, past the `Then`/`When` node
    /// itself), and nothing else in `self.activation().pc` can stand in for
    /// it here, because a nested call (from inside `run_bounded`) is
    /// stepping an instruction the activation's own `pc` is not pointing at.
    ///
    /// `source`, also added for Task 10, is threaded through to `If`/
    /// `Select`'s own `run_bounded` calls purely so `step_in_temps_frame`
    /// can resolve *its own* clause when an error escapes from inside one --
    /// without it, an error raised several `run_bounded` levels deep would
    /// only ever be attributed to the outermost `If`/`SELECT` that
    /// `run_activation` itself was stepping, which is wrong for exactly the
    /// same reason `run_activation`'s own doc comment gives for popping this
    /// activation before resolving a site: the last place the failing
    /// instruction is in hand has to be the one that resolves it. `None`
    /// preserves `run_fragment`'s existing, deliberate choice not to resolve
    /// a site at all for an instruction inside an `INTERPRET` fragment (its
    /// spans index the fragment's own source, not this one).
    fn step(
        &mut self,
        code: &Code<'_>,
        index: usize,
        instruction: &Instruction,
        source: Option<&ProgramSource>,
    ) -> Result<Flow, Failure> {
        match &instruction.kind {
            InstructionKind::Say { expression } => {
                let line = match expression {
                    Some(expression) => {
                        let value = self.eval(code, expression)?;
                        self.roots.push_temp(value);
                        self.to_text(value).to_vec()
                    }
                    // `say` with no expression is a blank line.
                    None => Vec::new(),
                };
                self.out.extend_from_slice(&line);
                self.out.push(b'\n');
                Ok(Flow::Next)
            }

            InstructionKind::Assignment { target, value } => {
                // `addVariable` builds only `Variable`, `Stem` or `Compound`
                // targets (`ast.rs`'s own doc comment on `Assignment`), so the
                // `other` arm below is unreachable through any program that
                // parsed. Kept anyway, and exercised through `Loud::expression`
                // rather than `unreachable!`, for the same reason `Loud`'s own
                // exhaustive matches in `lib.rs` are: a guarantee the grammar
                // makes is not one the type system enforces, and this crate's
                // own rule is to fail loudly rather than trust it blindly.
                let value = self.eval(code, value)?;
                self.roots.push_temp(value);
                match &target.kind {
                    ExprKind::Variable(id) => {
                        let name = code.symbols.name(*id).as_bytes();
                        let slot = self.slot_of(name);
                        let frame = self.activation().frame;
                        self.roots.set_slot(frame, slot, value);
                    }
                    // `stem. = expr`: replace-and-rebind (D15a), through the
                    // library `stem_assign` already builds -- this arm is the
                    // dispatch Task 9 owns, not new stem logic.
                    ExprKind::Stem(id) => {
                        let name = code.symbols.name(*id).as_bytes();
                        self.stem_assign(name, value);
                    }
                    // `a.b = expr`: resolve the tail key the same way reading
                    // `a.b` would (`eval_node`'s own `Compound` arm), then
                    // mutate that one tail in place through `stem_set`.
                    ExprKind::Compound(id) => {
                        let (stem_name, _tails) = compound_parts(code.symbols.name(*id));
                        let key = self.tail_key(code, *id);
                        self.stem_set(stem_name.as_bytes(), &key, value);
                    }
                    other => return Err(Loud::expression(other).into()),
                }
                Ok(Flow::Next)
            }

            // A simple variable back to unset (`RootSet::clear_slot`, added
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

            // `EXIT` with a result: the spike had only the bare form
            // (`expression: None` matched literally, nothing else reaching
            // this arm at all). The value crosses out of the instruction loop
            // as `Flow::Exit`, unconverted -- `Interp::exit_code_for` (`lib.rs`)
            // is what turns it into a process exit code, and it runs once in
            // `execute` rather than here, because a `Flow::Exit` can also
            // come from inside a fragment (`run_fragment`'s own propagating
            // arm, below), and the conversion needs nothing this loop knows
            // that `execute` does not already have.
            InstructionKind::Exit { expression } => {
                let value = match expression {
                    Some(expression) => {
                        let value = self.eval(code, expression)?;
                        // Rooted for exactly one clause, like every other
                        // `eval` result -- and that is shorter than this
                        // value actually needs. `step_in_temps_frame` pops
                        // this temp unconditionally right after `step`
                        // returns, before `Flow::Exit` ever reaches
                        // `run_activation`, so from that pop through `run`'s
                        // activation teardown and into `execute`'s
                        // `exit_code_for` call, this `ObjRef` is an
                        // **under-rooted** value -- longer and later than
                        // any other window in this crate. Benign only
                        // because nothing on that path allocates or
                        // collects (`Heap::alloc_with` never collects, and
                        // `to_number`/`to_text` read an existing object
                        // rather than making one); once a collector exists,
                        // this needs a root that survives past the
                        // temps-frame pop -- a global, or a dedicated field
                        // on `Interp` -- rather than the one-clause
                        // `push_temp` every other instruction result gets.
                        self.roots.push_temp(value);
                        Some(value)
                    }
                    None => None,
                };
                Ok(Flow::Exit(value))
            }

            // A label is a traced no-op: the C++'s own `execute` on a label
            // instruction only traces it (nothing in 4a writes to the trace
            // sink yet -- Task 13's own construct) and does nothing else.
            // `SIGNAL`/`CALL` reach a label by jumping to the instruction
            // after it; nothing ever executes the label node for its own
            // effect.
            InstructionKind::Label { .. } => Ok(Flow::Next),

            InstructionKind::Nop => Ok(Flow::Next),

            // 4a builds the fragment machinery and 4b builds the keyword on
            // top of it, so through `run_program` this is not implemented.
            InstructionKind::Interpret { expression } if self.interpret_spike => {
                let value = self.eval(code, expression)?;
                self.roots.push_temp(value);
                let text = self.to_text(value).to_vec();
                self.run_fragment(text)
            }

            // `IF`/`THEN`/`ELSE`. This arm resolves the whole construct
            // itself rather than leaving the outer loop to fall through the
            // flat list -- see `run_bounded`'s doc comment for why that is
            // not optional. `false_target`'s own doc comment: "the ELSE if
            // there is one, otherwise the instruction after the THEN
            // branch" -- confirmed by tracing `block.rs` by hand, it is the
            // `Else` instruction's own index when there is one, landing
            // *on* it rather than past it.
            InstructionKind::If {
                condition,
                false_target,
            } => {
                let len = code.body.instructions.len();
                let false_target = false_target.unwrap_or(len);
                if self.eval_condition(code, condition, raised_if_not_logical)? {
                    let resume = self.skip_else(code, false_target);
                    match self.run_bounded(code, index + 1, false_target, source)? {
                        Flow::Next => Ok(Flow::Goto(resume)),
                        other => Ok(other),
                    }
                } else {
                    // Nothing to skip on the false path: whether
                    // `false_target` names an `Else` (whose own body then
                    // runs, and only traces on the way in, per its own doc
                    // comment) or is simply where control resumes with no
                    // `ELSE` at all, the outer loop's ordinary fallthrough
                    // already lands in the right place with no ambiguity --
                    // that is only true of the false path; see
                    // `run_bounded`'s doc comment for why the true path
                    // cannot rely on the same thing.
                    Ok(Flow::Goto(false_target))
                }
            }

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
            // is a `SELECT CASE`), then tests each of its own `whens` in
            // source order by reading the `When`/`WhenCase` node directly as
            // data (`condition`/`values`, `false_target`, `exit`) rather
            // than dispatching through `step_in_temps_frame` -- a
            // `When`/`WhenCase` node must never be independently stepped for
            // a decision of its own, only ever run past inside a bounded
            // sub-loop (see the `When`/`WhenCase` arm, below, for why).
            InstructionKind::Select {
                case,
                whens,
                otherwise,
                end,
                ..
            } => {
                let len = code.body.instructions.len();
                let case_text = match case {
                    Some(case_expr) => {
                        let value = self.eval(code, case_expr)?;
                        self.roots.push_temp(value);
                        Some(self.to_text(value).to_vec())
                    }
                    None => None,
                };
                for &when_index in whens {
                    let when_instruction = &code.body.instructions[when_index];
                    let outcome = match &when_instruction.kind {
                        InstructionKind::When {
                            condition,
                            false_target,
                            exit,
                        } => {
                            let holds =
                                self.eval_condition(code, condition, raised_when_not_logical)?;
                            holds.then(|| (false_target.unwrap_or(len), exit.unwrap_or(len)))
                        }
                        InstructionKind::WhenCase {
                            values,
                            false_target,
                            exit,
                        } => {
                            let case_text = case_text.as_deref().expect(
                                "a WhenCase's enclosing Select always carries a case expression",
                            );
                            let matched = self.test_case_when(code, values, case_text)?;
                            matched.then(|| (false_target.unwrap_or(len), exit.unwrap_or(len)))
                        }
                        other => panic!("a SELECT's whens holds only When/WhenCase, not {other:?}"),
                    };
                    if let Some((body_end, resume)) = outcome {
                        return match self.run_bounded(code, when_index + 1, body_end, source)? {
                            Flow::Next => Ok(Flow::Goto(resume)),
                            other => Ok(other),
                        };
                    }
                }
                match otherwise {
                    // `OTHERWISE` is always the last clause before `END`, so
                    // its own fallthrough into `END` is already correct with
                    // no sibling to accidentally re-enter -- unlike a
                    // matched `WHEN`, this needs no bounded sub-loop.
                    Some(otherwise_index) => Ok(Flow::Goto(*otherwise_index)),
                    // Landing exactly on `END` is deliberate: that is what
                    // makes 7.3's clause echo the `END`'s and not the
                    // `SELECT`'s (`End`'s own arm, below, is where it
                    // raises).
                    None => Ok(Flow::Goto(end.unwrap_or(len))),
                }
            }

            // Pure markers, exactly like `Then`/`Else`: real dispatch lives
            // entirely in `Select`'s own arm above, which reads a
            // `When`/`WhenCase` node as data rather than stepping it. A
            // `When`/`WhenCase` reached here at all is either inside a
            // bounded sub-loop (the winning branch's body, where it is
            // inert filler) or is the absorbed-`WHEN` shape -- a `WHEN`
            // whose own `THEN` consequence is itself a `WHEN` clause, which
            // `Select.whens` never collects (`ast.rs`'s own doc comment on
            // `whens`, `select_when_absorption.rex`) -- and in neither case
            // does it get to decide anything on its own: measured against
            // the oracle, `select / when 1 = 1 then / when 2 = 2 then n = 42
            // / otherwise / n = 99 / end / say n` prints `0`, not `42`, so
            // the absorbed `WHEN`'s own condition being true must not run
            // its own consequence.
            InstructionKind::When { .. } | InstructionKind::WhenCase { .. } => Ok(Flow::Next),
            InstructionKind::Otherwise => Ok(Flow::Next),

            // `END`. `Do`/`Loop` closings are Task 11's and fail loudly;
            // `Select`'s two non-7.3 closings (`OTHERWISE` present) are
            // reached only by that `OTHERWISE`'s own ordinary body
            // fallthrough and do nothing. `EndStyle::Select`'s own doc
            // comment: "Reaching this END at run time is error 7.3, because
            // every WHEN was false" -- which is exactly the one path that
            // lands here, since `Select`'s own arm above sends every other
            // path around this instruction entirely.
            InstructionKind::End { closes, .. } => {
                let closes = closes
                    .as_ref()
                    .expect("an End's closes is only None while its body is still being assembled");
                match closes.style {
                    EndStyle::Select => Err(raised_select_no_when().into()),
                    EndStyle::Otherwise | EndStyle::LabeledOtherwise => Ok(Flow::Next),
                    EndStyle::Do | EndStyle::LabeledDo | EndStyle::Loop => {
                        Err(Loud::instruction(&instruction.kind).into())
                    }
                }
            }

            other => Err(Loud::instruction(other).into()),
        }
    }

    /// Runs one instruction inside its own temps frame.
    ///
    /// The frame is opened and closed **here rather than inside `step`**,
    /// because `step` returns through a dozen `?` paths and a frame closed on
    /// only some of them is worse than none: it would leak on exactly the
    /// paths nobody tests. Closing it around the call covers every exit,
    /// including the loud failures.
    ///
    /// One clause is the right lifetime for a temporary. It is also what the
    /// C++ does, and it is why `step` can push freely without deciding when to
    /// let go.
    ///
    /// **Also resolves the failing clause's site, when one escapes and
    /// `source` is `Some`.** Moved here from `run_activation`'s own error
    /// path (Task 10), because `run_activation` only ever sees the outermost
    /// instruction it was stepping, and Task 10 nests `step_in_temps_frame`
    /// calls arbitrarily deep through `If`/`Select`'s own `run_bounded` --
    /// without this, an error raised inside a `WHEN`'s branch would be
    /// misattributed to the enclosing `SELECT`'s own clause (measured
    /// against the oracle before this existed: a `1/0` inside a matched
    /// `WHEN`'s `THEN` reported the `SELECT`'s line and text, not the
    /// failing clause's). `self.failure_site.is_none()` is the guard that
    /// makes the *innermost* call win: the deepest `step_in_temps_frame`
    /// that sees the error is always the first to run this check, and every
    /// enclosing one that the error then propagates through leaves an
    /// already-`Some` site alone. `source: None` (`run_fragment`'s own call
    /// into `run_bounded`) skips this entirely, preserving that function's
    /// existing, deliberate choice not to resolve a site for an instruction
    /// inside an `INTERPRET` fragment.
    fn step_in_temps_frame(
        &mut self,
        code: &Code<'_>,
        index: usize,
        instruction: &Instruction,
        source: Option<&ProgramSource>,
    ) -> Result<Flow, Failure> {
        let frame = self.roots.push_frame();
        let flow = self.step(code, index, instruction, source);
        self.roots.pop_frame(frame);
        if let (Err(_), Some(source)) = (&flow, source)
            && self.failure_site.is_none()
        {
            self.failure_site = Some((
                source.line_of(instruction.clause_span.start),
                source
                    .join_span(instruction.clause_span.clone())
                    .map_or_else(
                        // Visible rather than silent, matching
                        // `Raised::message`'s own reasoning for a
                        // catalogue miss: the error path is the worst
                        // place to turn a reportable condition into a
                        // crash or a blank line.
                        || b"<clause span outside the retained source>".to_vec(),
                        |bytes| bytes.into_owned(),
                    ),
            ));
        }
        flow
    }

    /// Runs `code.body.instructions[start..end]` in place, one instruction at
    /// a time through `step_in_temps_frame`, and answers what happened.
    ///
    /// **Why this exists at all.** Phase 3 elides the C++'s synthetic
    /// end-of-branch markers (`ast.rs`'s own "Why there is no node for the
    /// synthetic end of a branch"), so a flat instruction list has nowhere to
    /// hang "the THEN branch just finished, skip the ELSE" or "one true WHEN
    /// ends the whole SELECT" other than on the branch instruction itself.
    /// Concretely, traced by hand against `block.rs`: `if c then A else B`
    /// gives `If.false_target == Else`'s own index, so the true path (fall
    /// through `A`, land on `Else` by `pc += 1`) and the false path (`Goto`
    /// straight to `Else`) arrive at the *identical* `(instruction, pc)`, and
    /// only one of the two arrivals is supposed to enter `B`. `SELECT`/`WHEN`
    /// has the same defect with no marker at all: a matched `WHEN`'s body,
    /// left to fall through on its own, lands on the next `WHEN` and would
    /// test it again (`select_when_bodies.rex` is written to catch exactly
    /// that). Per-instruction dispatch on `(instruction, pc)` alone cannot
    /// tell these two arrivals apart, and a per-activation block stack would
    /// resolve it trivially but does not exist yet (`Activation`'s own doc
    /// comment: Task 11's).
    ///
    /// **The fix confines itself to a range instead.** `If`/`Select` compute
    /// the winning branch's `[start, end)` directly from data already on the
    /// node (`If`'s `false_target`, `Select`'s `whens`/`false_target`/`exit`)
    /// and run exactly that range here, then return one `Flow::Goto` past
    /// the whole construct -- so the ambiguous fallthrough this function
    /// would otherwise produce never reaches the outer loop at all. Nested
    /// constructs are safe under this with no extra bookkeeping, because
    /// `block.rs` assembles an inner branch's own jump targets to close
    /// before the outer one's does, so they always fall inside (or exactly
    /// at the boundary of) whichever range encloses them.
    ///
    /// **What this owns, and what it does not.** A `Flow::Next` advances the
    /// local `pc` by one. A `Flow::Goto(target)` is only "mine" when `target`
    /// is inside `[start, end]` (`end` inclusive: a nested construct's own
    /// resume point landing exactly on my own boundary is normal completion,
    /// not an escape) -- anything else, this returns immediately and
    /// unchanged, exactly as received. That covers `Flow::Exit` today and is
    /// written to keep covering whatever Task 11 adds for `LEAVE`/`ITERATE`:
    /// `leave sel` on a labelled `SELECT` has to unwind out of a nested Rust
    /// call the same way `Flow::Exit` already does here, and a `Flow`
    /// variant this function does not recognise is deliberately the same
    /// case as one whose `Goto` target falls outside the range -- both fall
    /// to the catch-all below and propagate outward rather than being
    /// matched (and silently mishandled) by name.
    ///
    /// Reaching `end` exactly, whether by `pc += 1` or by an in-range `Goto`,
    /// is the only way this returns `Ok(Flow::Next)`; every other exit
    /// returns the escaping `Flow` unchanged, and the caller (`If`/`Select`)
    /// must check which happened rather than assume the former.
    ///
    /// `source` is forwarded to `step_in_temps_frame` unchanged, purely so it
    /// can resolve the failing clause's own site rather than the caller's --
    /// see that function's own doc comment.
    fn run_bounded(
        &mut self,
        code: &Code<'_>,
        start: usize,
        end: usize,
        source: Option<&ProgramSource>,
    ) -> Result<Flow, Failure> {
        let mut pc = start;
        while pc < end {
            let instruction = &code.body.instructions[pc];
            match self.step_in_temps_frame(code, pc, instruction, source)? {
                Flow::Next => pc += 1,
                Flow::Goto(target) if target >= start && target <= end => pc = target,
                other => return Ok(other),
            }
        }
        Ok(Flow::Next)
    }

    /// Evaluates `condition` and answers whether it holds, for `IF`/`WHEN`.
    ///
    /// **A comma list checks itself; a single expression does not, and this
    /// is the one place that gap gets closed.** `ExprKind::Logical` (a comma
    /// list) is evaluated through `eval`'s own dispatch to `eval_logical_list`
    /// exactly like any other expression, which already validates every
    /// element is exactly `0`/`1` and raises 34.6 on the first that is not --
    /// re-checking its result here would misreport that failure as 34.1/34.2.
    /// A single, non-list expression never passes through `eval_logical_list`
    /// at all (there is no list to iterate), so nothing has checked it yet;
    /// `raise` is the keyword-specific raiser for exactly that case (34.1
    /// `IF`, 34.2 `WHEN`) -- measured across both, `if 'x', 1 then` is 34.6
    /// (a list, regardless of which element failed) while `if 'x' then` is
    /// 34.1 (not a list at all).
    fn eval_condition(
        &mut self,
        code: &Code<'_>,
        condition: &Expr,
        raise: fn(&[u8]) -> Raised,
    ) -> Result<bool, Failure> {
        let value = self.eval(code, condition)?;
        self.roots.push_temp(value);
        let text = self.to_text(value).to_vec();
        if matches!(condition.kind, ExprKind::Logical(_)) {
            // `eval_logical_list` already validated every element and
            // answers exactly `b"0"`/`b"1"` (its own doc comment), so this
            // is a plain readback rather than a second check.
            Ok(text == b"1")
        } else {
            logical_value(&text).ok_or_else(|| raise(&text).into())
        }
    }

    /// Whether any of a `WHEN CASE`'s `values` compares `==` (byte-for-byte,
    /// no padding, no numeric awareness) equal to the `SELECT CASE`'s own
    /// `case_text`, matching on the first that does (an OR of `==`, the
    /// opposite of a plain `WHEN`'s comma list, which is an AND checked for
    /// `0`/`1` -- `ast.rs`'s own doc comment on `WhenCase`).
    ///
    /// **Reasoned rather than routed through `eval_compare`'s own
    /// `Operator::StrictEqual`, and this is why.** The design's own
    /// "Expression evaluation" section states the strict family's rule in
    /// full: "there is no padding and the shorter string is less" -- for an
    /// *ordering* comparison. Equality has no "less" to fall back on, so
    /// under that same rule two strict operands are equal if and only if
    /// they are the same length and every byte matches, which is exactly
    /// `==` on the two `Vec<u8>`s below and needs no numeric awareness or
    /// `rexx-num` call to compute. Measured, matching D15's own example:
    /// `select case '007'` does not match `when 7`, because `"007"` and
    /// `"7"` are not byte-identical. Calling `eval_compare` would work too,
    /// but needs a second `eval.rs` visibility bump beyond the one already
    /// asked for and approved (`logical_value`); this way needs none.
    fn test_case_when(
        &mut self,
        code: &Code<'_>,
        values: &[Expr],
        case_text: &[u8],
    ) -> Result<bool, Failure> {
        for value in values {
            let value = self.eval(code, value)?;
            self.roots.push_temp(value);
            if &*self.to_text(value) == case_text {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// If `target` names an `Else` instruction, its own `then_exit`
    /// (defaulting to the end of the body when `None`, "the end of this
    /// body" per `ast.rs`'s own doc comment); otherwise `target` unchanged.
    ///
    /// Shared by both of `If`'s own arms: the true path calls this to learn
    /// where to resume once its bounded branch finishes, and the doc comment
    /// on the `If` arm is where the false path's own reasoning for *not*
    /// needing this lives.
    fn skip_else(&self, code: &Code<'_>, target: usize) -> usize {
        match code.body.instructions.get(target).map(|i| &i.kind) {
            Some(InstructionKind::Else { then_exit }) => {
                then_exit.unwrap_or(code.body.instructions.len())
            }
            _ => target,
        }
    }

    /// Parses `text` as an `INTERPRET` fragment and runs it **inside the
    /// current activation**.
    ///
    /// This is the case that stresses the lifetime, and the three things it
    /// proves are:
    ///
    /// * The fragment's `Rc` is a **local** that outlives the nested loop, the
    ///   same shape `run_activation` uses, one level down. The enclosing
    ///   loop's own local `Rc<Program>` is untouched and still anchors the
    ///   instruction that is mid-execution.
    /// * The nested loop's program counter is a **local `usize`**, not the
    ///   activation's, because the activation's is sitting on the `INTERPRET`
    ///   instruction and has to still be there afterwards. A `LEAVE`/
    ///   `ITERATE` naming an outer loop could not target anything inside a
    ///   fragment for the measured reason that a fragment can never have a
    ///   label at all: one inside `INTERPRET` text is error 47.1 (Task 1), so
    ///   `body.labels` is always empty. `IF`/`SELECT` need no label and can
    ///   still appear and jump *inside* a fragment, which is why this reuses
    ///   `run_bounded` (Task 10) rather than the hand-rolled loop this
    ///   function used to have: every jump such a construct computes stays
    ///   within `[0, code.body.instructions.len())` by construction
    ///   (`resolve_targets` clamps everything else to `None`, and `None`
    ///   defaults to the body's own length), so `run_bounded` always "owns"
    ///   it and never mistakes it for an escape.
    /// * No frame is pushed. The fragment's assignments land in the enclosing
    ///   frame's slots, which is what `fragment_plan` resolves them against,
    ///   and it is why `RootSet::grow_slots`'s top-frame assertion holds here.
    fn run_fragment(&mut self, text: Vec<u8>) -> Result<Flow, Failure> {
        let fragment: Rc<Fragment> = match parse_interpret(text) {
            Ok(fragment) => Rc::new(fragment),
            Err(error) => return Err(Loud::parse(&error).into()),
        };

        // An owned `Fragment` would do here, since nothing but this loop reads
        // it. It is an `Rc` because that is the shape 4b needs, where an
        // `INTERPRET` inside a fragment makes this function reentrant and each
        // level anchors its own.
        let slots = self.fragment_plan(&fragment);
        let code = Code {
            body: &fragment.body,
            symbols: &fragment.symbols,
            slots: &slots,
        };

        // `exit` inside `INTERPRET` ends the program, not the fragment, so
        // it has to propagate rather than stop here -- `run_bounded`'s own
        // catch-all does exactly that for anything it does not own, `Exit`
        // included, with nothing fragment-specific to add.
        self.run_bounded(&code, 0, code.body.instructions.len(), None)
    }

    /// Drops one `DROP` target: a plain variable, a whole stem, one tail, or
    /// the `(v)` indirect form.
    ///
    /// **`Direct` resolves a compound's tail pieces as variables; `Indirect`
    /// is a subsidiary list, not a single name.** `Direct(id)`'s name came
    /// through the scanner, so a compound-shaped spelling still has real
    /// tail pieces to resolve -- `tail_key`, exactly what a `Compound`
    /// expression's own read or `Assignment`'s own write already does.
    ///
    /// `Indirect(id)`'s value is **blank- or tab-separated list of variable
    /// symbols, each validated and dropped on its own**, not one verbatim
    /// name -- a fix-round correction to this function's first version,
    /// which treated the whole value as a single name and let three classes
    /// of bad input through silently. Measured, the six rows that pin it
    /// down (all against the oracle):
    ///
    /// ```text
    /// a=1; b=2; v='a b'      ; drop (v); say a; say b   ->  A / B  (both dropped)
    /// x=1;      v=' x '      ; drop (v); say x           ->  X      (trimmed)
    /// v='9'                  ; drop (v)                  ->  Error 31.2
    /// v='.x'                 ; drop (v)                  ->  Error 31.3
    /// w=1;      v='(w)'      ; drop (v)                  ->  Error 20.928
    /// a=1;      v='a'        ; drop (v); say a           ->  A      (agrees with the old reading)
    /// ```
    ///
    /// The last row is why every pre-fix test passed: a single-word,
    /// already-valid, unpadded value is exactly where "split, validate,
    /// resolve each" and "resolve the whole value" coincide. The `(w)` row
    /// is what rules out a recursive reading -- a parenthesised entry is
    /// 20.928, "Symbol expected as an indirect variable name", the same
    /// error any other not-a-symbol word gets (`a-b` gives the identical
    /// 20.928, `found "a-b"`), not a second round of indirection.
    ///
    /// **Validation runs over the whole list before any drop happens.**
    /// Measured: `a=1; b=2; v='a 9 b'; drop (v)` raises 31.2 on `"9"` and
    /// leaves *both* `a` and `b` at `1` and `2` -- `a` is never dropped even
    /// though it sits before the bad word. So this collects every word's
    /// validated, upcased name first (`validate_indirect_word`, which can
    /// fail) and only then drops each one (`drop_by_name`, which cannot),
    /// rather than interleaving the two.
    ///
    /// **The value is upcased only after validation, one word at a time,
    /// never as a whole.** Measured: `v = 'x'; x = 1; drop (v); say x`
    /// prints `X` -- each word is upcased exactly as the scanner would have
    /// upcased it had it been written directly, because `DROP (v)` never
    /// goes through the scanner at all. A `Direct` name is already upcased,
    /// by `SymbolTable::intern`, long before this ever runs.
    fn drop_variable(&mut self, code: &Code<'_>, variable: &VariableRef) -> Result<(), Failure> {
        match variable {
            VariableRef::Direct(id) => {
                let name = code.symbols.name(*id);
                if shape_of(name.as_bytes()) == NameShape::Compound {
                    let (stem_name, _tails) = compound_parts(name);
                    let key = self.tail_key(code, *id);
                    self.stem_drop_tail(stem_name.as_bytes(), &key);
                } else {
                    self.drop_by_name(name.as_bytes());
                }
            }
            VariableRef::Indirect(id) => {
                let (value, _novalue) = self.read(code, *id);
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
    ///
    /// Shared by `Direct`'s `Simple`/`Stem` cases (an already-upcased
    /// compile-time name) and by every word of an indirect subsidiary list
    /// (already validated and upcased by `validate_indirect_word`) -- both
    /// are "a plain string names a variable, resolve it with no further
    /// symbol lookup", the same operation `drop_variable`'s own doc comment
    /// says the two cases share. **Not** used for `Direct`'s `Compound` case:
    /// a source-level compound's tail pieces are still symbols to resolve
    /// (`tail_key`), which this function's uniform verbatim split at the
    /// first period does not do.
    fn drop_by_name(&mut self, name: &[u8]) {
        match shape_of(name) {
            NameShape::Simple => {
                let slot = self.slot_of(name);
                let frame = self.activation().frame;
                self.roots.clear_slot(frame, slot);
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

    /// `NUMERIC DIGITS`/`FUZZ`/`FORM`, in every spelling the parser produces
    /// (`NumericSetting`, `rexx-parse`'s own `instruction.rs::numeric`).
    ///
    /// `DIGITS`/`FUZZ` with no expression reset to the package default --
    /// measured, `numeric digits 3; numeric digits; y = 1/3; say y` gives
    /// `0.333333333`, the DIGITS-9 rendering, and the reset is reported
    /// exactly as if `"9"` had been typed rather than as some sentinel
    /// meaning "no change": `numeric digits 20; numeric fuzz 15; numeric
    /// digits` raises 33.1 with `("9")` as the rejected candidate. `FORM`
    /// alone (`FormDefault`) resets the same way, to `SCIENTIFIC` -- measured,
    /// `numeric form engineering; numeric form; say form()` gives
    /// `SCIENTIFIC`. 4a has no `::OPTIONS` to move the package default away
    /// from `Scientific`, which is why `FormDefault` and `FormScientific` do
    /// the identical thing below; a later phase's `::OPTIONS FORM` is what
    /// would make the two differ, and should split this arm rather than
    /// assume they stay equal.
    fn exec_numeric(
        &mut self,
        code: &Code<'_>,
        setting: &NumericSetting,
        expression: &Option<Expr>,
    ) -> Result<(), Failure> {
        match setting {
            NumericSetting::Digits => {
                let default = rexx_num::DEFAULT_DIGITS.to_string();
                let text = self.numeric_operand(code, expression, &default)?;
                self.activation_mut()
                    .settings
                    .set_digits_str(&text)
                    .map_err(raised_from_settings)?;
            }
            NumericSetting::Fuzz => {
                let text = self.numeric_operand(code, expression, "0")?;
                self.activation_mut()
                    .settings
                    .set_fuzz_str(&text)
                    .map_err(raised_from_settings)?;
            }
            NumericSetting::FormDefault | NumericSetting::FormScientific => {
                self.activation_mut()
                    .settings
                    .set_form_str("SCIENTIFIC")
                    .expect("a hardcoded valid spelling always validates");
            }
            NumericSetting::FormEngineering => {
                self.activation_mut()
                    .settings
                    .set_form_str("ENGINEERING")
                    .expect("a hardcoded valid spelling always validates");
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
                let text = String::from_utf8_lossy(&self.to_text(value)).into_owned();
                self.activation_mut()
                    .settings
                    .set_form_str(&text)
                    .map_err(raised_from_settings)?;
            }
        }
        Ok(())
    }

    /// Evaluates `expression`, or answers `default` when there is none
    /// (`NUMERIC DIGITS`/`FUZZ` alone).
    ///
    /// A `String` rather than the value's own bytes: `set_digits_str`/
    /// `set_fuzz_str` take `&str`, and a Rexx value's bytes are not
    /// guaranteed UTF-8. `from_utf8_lossy` is this crate's own standing choice
    /// for exactly that gap (`Raised::nonnumeric`'s substitution text,
    /// `error.rs`), and a lossy byte cannot parse as a valid DIGITS/FUZZ
    /// value either way, so the conversion still ends in the right error
    /// family rather than silently accepting mangled input.
    fn numeric_operand(
        &mut self,
        code: &Code<'_>,
        expression: &Option<Expr>,
        default: &str,
    ) -> Result<String, Failure> {
        match expression {
            Some(expression) => {
                let value = self.eval(code, expression)?;
                self.roots.push_temp(value);
                Ok(String::from_utf8_lossy(&self.to_text(value)).into_owned())
            }
            None => Ok(default.to_string()),
        }
    }

    // `fragment_plan` and `slot_of` live in `plan.rs` (Task 6), beside `Plan`
    // itself; `stem_assign`/`stem_set`/`stem_drop`/`stem_drop_tail`/
    // `tail_key` live in `stem.rs` (Task 5), beside the rest of the D15a
    // library. `read` lives in `lib.rs`, beside `Interp`'s other value-model
    // entry points.
}

/// Which of the three variable shapes `name`'s own spelling is, from an
/// already-interned (or already-upcased runtime) name alone.
///
/// Reproduces the scanner's own classification (`scanner.rs::scan_symbol`,
/// `SymbolClass::{Variable,Stem,Compound}`) as a pure function of the byte
/// string, which is all a `DROP` target has by the time it reaches here --
/// a direct target's name came from the scanner originally, but an indirect
/// one (`DROP (v)`) never did, so this cannot simply read a tag the AST
/// already carries. The rule is exactly the scanner's: no period is a simple
/// variable; exactly one period, and it is the last byte, is a stem;
/// anything else with a period -- two or more, or one not at the end -- is a
/// compound.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum NameShape {
    Simple,
    Stem,
    Compound,
}

fn shape_of(name: &[u8]) -> NameShape {
    let dots = name.iter().filter(|&&b| b == b'.').count();
    if dots == 0 {
        NameShape::Simple
    } else if dots == 1 && name.last() == Some(&b'.') {
        NameShape::Stem
    } else {
        NameShape::Compound
    }
}

/// Splits a `DROP (v)` wrapper's value into its subsidiary list's words.
///
/// A blank (`' '`) or a tab (`'\t'`) separates words, any run of either
/// counts as one separator, and an empty word never results -- measured,
/// `'a    b'` and a leading/trailing-blank `'  a  b  '` both give exactly
/// `["a", "b"]`, and a `'09'x` (tab) byte between two names splits them the
/// same way a space does. A newline or carriage return does **not**
/// separate: `'a' || '0a'x || 'b'` is one word, `"a\nb"`, which then fails
/// `validate_indirect_word`'s character check and raises 20.928 rather than
/// splitting. An all-blank or empty value yields no words at all, which is
/// why `drop (v)` on an empty or blanks-only `v` is a silent no-op on the
/// oracle rather than an error.
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

/// Validates one word of an indirect `DROP`'s subsidiary list and answers
/// its upcased name, or the condition the oracle raises for it.
///
/// Three ways a word can fail, checked in the order the oracle's own error
/// numbers imply (a character-set check before either shape check, since a
/// word with an illegal character is never inspected for its *first*
/// character at all -- measured, `'a-b'` and `'(w)'` both give 20.928, not
/// 31.2/31.3, even though neither starts with a digit or a period):
///
/// * any byte outside the symbol character set (`is_symbol_byte`) -- 20.928,
///   "Symbol expected as an indirect variable name"; this is also what rules
///   out treating a parenthesised entry as a nested indirect reference,
///   since `(`/`)` are not symbol characters and so are rejected the same
///   way any other stray punctuation is, not by a dedicated recursion guard;
/// * a leading digit -- 31.2, "Variable symbol must not start with a
///   number", matching `SymbolClass::Constant`'s own first-byte rule;
/// * a leading period -- 31.3, "Variable symbol must not start with a
///   '.'" (measured on a bare `"."` too, not only a longer dot-led word).
///
/// Every substitution is the word's **own, unmodified** bytes -- measured,
/// `'.X'`/`'9abc'` report `found ".X"`/`found "9abc"`, not the upcased form
/// -- so upcasing happens only on the success path, after every check.
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

/// 20.928: a subsidiary-list word is not a legal symbol at all (contains a
/// byte outside `is_symbol_byte`'s set, which is also what a parenthesised
/// entry like `"(w)"` fails on).
fn raised_symbol_expected(found: &[u8]) -> Raised {
    Raised {
        condition: "SYNTAX",
        number: 20,
        sub: 928,
        additional: vec![String::from_utf8_lossy(found).into_owned()],
    }
}

/// 31.2: a subsidiary-list word starts with a digit.
fn raised_digit_led(found: &[u8]) -> Raised {
    Raised {
        condition: "SYNTAX",
        number: 31,
        sub: 2,
        additional: vec![String::from_utf8_lossy(found).into_owned()],
    }
}

/// 31.3: a subsidiary-list word starts with a period.
fn raised_dot_led(found: &[u8]) -> Raised {
    Raised {
        condition: "SYNTAX",
        number: 31,
        sub: 3,
        additional: vec![String::from_utf8_lossy(found).into_owned()],
    }
}

/// 34.1: a single (non-list) `IF` condition is not exactly `0` or `1`.
/// `Error_Logical_value_if`, catalogue text "Value of expression following
/// IF keyword must be exactly \"0\" or \"1\"; found \"...\"", one
/// substitution, the operand's own rendered text.
fn raised_if_not_logical(found: &[u8]) -> Raised {
    Raised {
        condition: "SYNTAX",
        number: 34,
        sub: 1,
        additional: vec![String::from_utf8_lossy(found).into_owned()],
    }
}

/// 34.2: a single (non-list) `WHEN` condition is not exactly `0` or `1`.
/// `Error_Logical_value_when`, the same shape as `raised_if_not_logical`
/// with `WHEN`'s own sub-number -- a plain `WHEN`'s comma list is the
/// opposite case (`WhenCase`'s doc comment) and never reaches this raiser,
/// since `eval_condition` only calls it when `condition.kind` is not
/// `ExprKind::Logical`.
fn raised_when_not_logical(found: &[u8]) -> Raised {
    Raised {
        condition: "SYNTAX",
        number: 34,
        sub: 2,
        additional: vec![String::from_utf8_lossy(found).into_owned()],
    }
}

/// 7.3: a `SELECT` reached its `END` with every `WHEN` false and no
/// `OTHERWISE`. `Error_When_expected_nootherwise`, catalogue text "All WHEN
/// expressions of SELECT are false; OTHERWISE expected.", no substitutions
/// (measured against `interpreter/messages/rexxmsg.xml`'s own `<Text>` for
/// major 7 sub 003, which carries no `<Sub>` tag).
///
/// Raised from `End`'s own arm, not `Select`'s -- `EndStyle::Select`'s own
/// doc comment says reaching that `END` at run time *is* the error, and
/// `Select`'s arm sends every other outcome around this instruction
/// entirely (`Flow::Goto` past it on a match, or onto it exactly on no
/// match/no `OTHERWISE`), so the clause `run_activation`'s failure path
/// echoes is the `END`'s own, matching the oracle (measured, rc 249).
fn raised_select_no_when() -> Raised {
    Raised {
        condition: "SYNTAX",
        number: 7,
        sub: 3,
        additional: Vec::new(),
    }
}

/// Converts a `rexx-num` settings failure into a `Raised`.
///
/// `ArithError` has a `sub_code` accessor `rexx-num` made `pub` expressly for
/// `error.rs`'s own `From` impl (that impl's doc comment says so);
/// `SettingsError`'s equivalent is still private, and nothing asked for it to
/// change for this one caller. The `(major, sub)` pairs below are copied from
/// `settings.rs`'s own doc comments on each variant rather than read through
/// an accessor that does not exist yet. `Raised`'s fields are `pub(crate)`,
/// which is what lets this build one directly, with no constructor of its
/// own needed in `error.rs`.
fn raised_from_settings(error: SettingsError) -> Raised {
    let additional = error.additional();
    let (number, sub): (u16, u16) = match &error {
        SettingsError::InvalidForm { .. } => (25, 11),
        SettingsError::DigitsNotWhole { .. } => (26, 5),
        SettingsError::FuzzNotWhole { .. } => (26, 6),
        SettingsError::FuzzNotBelowDigits { .. } => (33, 1),
    };
    Raised {
        condition: "SYNTAX",
        number,
        sub,
        additional,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Activation;
    use crate::plan::{BodyKey, ProgramId};
    use rexx_parse::{Program, parse_program};
    use std::collections::HashMap;

    /// Pushes a fresh top-level activation for `program`, the same setup
    /// `Interp::run` does, so a test can drive `step` through a live
    /// activation without the full instruction loop. Copied rather than
    /// shared, matching every other test module in this crate (`eval.rs`,
    /// `plan.rs`, `stem.rs` each keep their own).
    fn activate(interp: &mut Interp, program: Program) -> Rc<Program> {
        let program = Rc::new(program);
        let id = ProgramId(interp.programs.len());
        interp.programs.push(Rc::clone(&program));
        let plan = interp.plan_for(
            BodyKey {
                program: id,
                directive: None,
            },
            &program.main,
            &program.symbols,
        );
        let frame = interp.roots.push_slots(plan.len());
        interp
            .activations
            .push(Activation::new(Rc::clone(&program), plan, frame));
        program
    }

    /// Parses `source`, activates it, and runs its whole body -- a miniature
    /// `run_activation`, through `run_bounded` rather than a hand-rolled
    /// `for` loop since Task 10: this module's tests now include `IF`/
    /// `SELECT` programs that branch, and `run_bounded(code, 0, len)` is
    /// exactly `run_activation`'s own loop shape (Task 10's own doc comment
    /// on it explains why a plain `for` cannot follow a `Goto`). Every call
    /// here still reaches `step` only through `step_in_temps_frame`, since
    /// that is what `run_bounded` itself does -- this helper never calls
    /// `step` directly, matching the one-non-test-caller rule. `slots` is an
    /// empty map throughout: `read`/`slot_of`'s fallback chain answers
    /// correctly without the real plan's fast path, the same choice
    /// `eval.rs`'s own test helpers make.
    fn run_source(interp: &mut Interp, source: &[u8]) -> Result<Option<ObjRef>, Failure> {
        let program = parse_program(source.to_vec()).expect("test program parses");
        let program = activate(interp, program);
        let code = Code {
            body: &program.main,
            symbols: &program.symbols,
            slots: &HashMap::new(),
        };
        match interp.run_bounded(
            &code,
            0,
            code.body.instructions.len(),
            Some(&program.source),
        )? {
            Flow::Next => Ok(None),
            Flow::Exit(value) => Ok(value),
            Flow::Goto(_) => {
                unreachable!("run_bounded never escapes a top-level program's own [0, len) range")
            }
        }
    }

    fn say_output(interp: &mut Interp, source: &[u8]) -> Vec<u8> {
        run_source(interp, source).expect("test program runs");
        std::mem::take(&mut interp.out)
    }

    // ---- assignment ----

    #[test]
    fn assignment_to_a_variable_a_stem_and_a_compound() {
        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(&mut interp, b"x = 5\nsay x"),
            b"5\n".to_vec(),
            "a simple variable"
        );

        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(&mut interp, b"a. = 'wd'\nsay a.1"),
            b"wd\n".to_vec(),
            "a bare stem assignment, read through an unset tail"
        );

        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(&mut interp, b"a.1 = 'one'\nsay a.1\nsay a.2"),
            b"one\nA.2\n".to_vec(),
            "a compound assignment mutates one tail and leaves the rest deriving its name"
        );
    }

    // ---- SAY ----

    #[test]
    fn say_of_each_value_kind_and_of_an_omitted_expression() {
        let mut interp = Interp::new(false);
        assert_eq!(say_output(&mut interp, b"say 'abc'"), b"abc\n".to_vec());
        assert_eq!(say_output(&mut interp, b"say 1 + 2"), b"3\n".to_vec());
        assert_eq!(
            say_output(&mut interp, b"say .nil"),
            b"The NIL object\n".to_vec()
        );
        // No expression at all: a blank line, not nothing.
        assert_eq!(say_output(&mut interp, b"say"), b"\n".to_vec());
    }

    // ---- DROP ----

    #[test]
    fn drop_of_a_variable_returns_it_to_unset() {
        // a = 5; drop a; say a -> A (the derived name, not left over from
        // before -- and never `.nil`, which is a value and not an absence).
        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(&mut interp, b"a = 5\ndrop a\nsay a"),
            b"A\n".to_vec()
        );

        // The `.nil`-versus-dropped distinction `RootSet::clear_slot` exists
        // for: `x = .nil` renders "The NIL object"; a dropped variable
        // derives its own name instead, never that string.
        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(&mut interp, b"y = .nil\ndrop y\nsay y"),
            b"Y\n".to_vec()
        );
    }

    #[test]
    fn drop_of_a_tail_tombstones_it_without_taking_the_default() {
        // u. = 'd'; u.1 = 'one'; drop u.1; say u.1 -> U.1 (tombstoned, not
        // falling back to the default); say u.2 -> d (an untouched tail
        // still does).
        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(
                &mut interp,
                b"u. = 'd'\nu.1 = 'one'\ndrop u.1\nsay u.1\nsay u.2"
            ),
            b"U.1\nd\n".to_vec()
        );
    }

    #[test]
    fn drop_of_a_whole_stem_leaves_it_looking_untouched() {
        // x. = 'd'; x.1 = 'one'; drop x.; say x.1; say x. -> X.1, X. (exactly
        // what a never-touched stem would give).
        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(
                &mut interp,
                b"x. = 'd'\nx.1 = 'one'\ndrop x.\nsay x.1\nsay x."
            ),
            b"X.1\nX.\n".to_vec()
        );
    }

    #[test]
    fn drop_of_the_indirect_form() {
        // A simple variable named by another's (upcased) value.
        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(&mut interp, b"v = 'x'\nx = 1\ndrop (v)\nsay x"),
            b"X\n".to_vec(),
            "the wrapper's value is upcased before it names a variable"
        );

        // A whole stem, named indirectly.
        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(
                &mut interp,
                b"v = 'A.'\na. = 'wd'\na.1 = 'one'\ndrop (v)\nsay a.1\nsay a."
            ),
            b"A.1\nA.\n".to_vec()
        );

        // One tail, named indirectly, joined-dots key taken verbatim rather
        // than re-resolved as source -- the discriminating transcript from
        // `drop_variable`'s own doc comment.
        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(
                &mut interp,
                b"v = 'A.1.2'\na.1.2 = 'x'\ndrop (v)\nsay a.1.2"
            ),
            b"A.1.2\n".to_vec()
        );
    }

    /// Fix-round: the indirect form's value is a **subsidiary list**, not a
    /// single verbatim name. Every case here uses *set* targets (the trap
    /// the review named: an unset target's cleared slot and its own derived
    /// name render identically, so a test built on unset targets cannot
    /// tell "resolved the whole value as one name" apart from "split,
    /// validated and dropped two names" -- only a set value discriminates).
    #[test]
    fn drop_of_the_indirect_form_is_a_subsidiary_list_of_words() {
        // Two names, blank-separated, both set and both dropped.
        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(
                &mut interp,
                b"a = 1\nb = 2\nv = 'a b'\ndrop (v)\nsay a\nsay b"
            ),
            b"A\nB\n".to_vec(),
            "a blank-separated list drops every word, not one name literally spelled 'A B'"
        );

        // A run of blanks between words, and leading/trailing blanks, both
        // collapse -- still exactly two words.
        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(
                &mut interp,
                b"a = 1\nb = 2\nv = '  a    b  '\ndrop (v)\nsay a\nsay b"
            ),
            b"A\nB\n".to_vec()
        );

        // A tab (`'09'x`) separates two words exactly like a blank does.
        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(
                &mut interp,
                b"a = 1\nb = 2\nv = 'a'||'09'x||'b'\ndrop (v)\nsay a\nsay b"
            ),
            b"A\nB\n".to_vec(),
            "a tab byte separates words the same way a blank does"
        );

        // A mix of shapes in one list: a whole stem and a simple variable.
        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(
                &mut interp,
                b"a. = 'd'\na.1 = 'one'\nx = 5\nv = 'a. x'\ndrop (v)\nsay a.1\nsay x"
            ),
            b"A.1\nX\n".to_vec()
        );
    }

    #[test]
    fn drop_of_the_indirect_form_validates_every_word() {
        // A digit-led word: 31.2.
        let mut interp = Interp::new(false);
        let failure = run_source(&mut interp, b"v = '9'\ndrop (v)").unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (31, 2));
        assert_eq!(raised.additional, vec!["9".to_string()]);

        // A dot-led word: 31.3.
        let mut interp = Interp::new(false);
        let failure = run_source(&mut interp, b"v = '.x'\ndrop (v)").unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (31, 3));
        assert_eq!(raised.additional, vec![".x".to_string()]);

        // A parenthesised word: 20.928, not a second round of indirection --
        // proves the list is not recursive.
        let mut interp = Interp::new(false);
        let failure = run_source(&mut interp, b"w = 1\nv = '(w)'\ndrop (v)").unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (20, 928));
        assert_eq!(raised.additional, vec!["(w)".to_string()]);

        // **A newline does not separate.** It is not whitespace for this
        // purpose: the whole of `a`, the newline and `b` form ONE word, which
        // then fails the character-set check, and the reported name carries
        // the raw newline. Measured byte for byte against the oracle,
        // substitution included.
        //
        // This assertion is why `split_indirect_words` tests space and tab
        // explicitly instead of calling `is_ascii_whitespace`. Without it a
        // mutant that used `is_ascii_whitespace` passed all 79 tests, because
        // no other case here carries a newline, and the distinction rested
        // entirely on an end-to-end diff nobody re-runs. Carriage return,
        // form feed and vertical tab behave as the newline does.
        let mut interp = Interp::new(false);
        let failure = run_source(&mut interp, b"v = 'a'||'0a'x||'b'\ndrop (v)").unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (20, 928));
        assert_eq!(raised.additional, vec!["a\nb".to_string()]);
    }

    #[test]
    fn drop_of_the_indirect_form_validates_before_dropping_any_of_it() {
        // a=1; b=2; v='a 9 b'; drop (v) -> Error 31.2, and NEITHER a nor b
        // is dropped, even though `a` sits before the bad word -- measured
        // against the oracle (SIGNAL ON SYNTAX recovery there shows both
        // untouched). The whole list validates before any drop runs.
        let mut interp = Interp::new(false);
        let failure = run_source(&mut interp, b"a = 1\nb = 2\nv = 'a 9 b'\ndrop (v)").unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (31, 2));
        assert_eq!(raised.additional, vec!["9".to_string()]);

        // The activation is still on the stack (`run_source` does not pop
        // on error), so `a`'s slot is still directly inspectable.
        let a_slot = interp.slot_of(b"A");
        let frame = interp.activation().frame;
        let a_value = interp
            .roots
            .slot(frame, a_slot)
            .expect("a must still be set");
        assert_eq!(
            &*interp.to_text(a_value),
            b"1",
            "a must not have been dropped before the list's third word failed validation"
        );
    }

    #[test]
    fn drop_of_the_indirect_form_on_an_empty_or_blanks_only_value_is_a_no_op() {
        // Measured: `v=''`/`v='   '` both run clean under the oracle -- zero
        // words, nothing to validate or drop.
        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(&mut interp, b"v = ''\ndrop (v)\nsay 'after'"),
            b"after\n".to_vec()
        );

        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(&mut interp, b"v = '   '\ndrop (v)\nsay 'after'"),
            b"after\n".to_vec()
        );
    }

    // ---- NUMERIC ----

    #[test]
    fn numeric_digits_changes_rounding_and_resets_to_9_with_no_expression() {
        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(&mut interp, b"numeric digits 3\nsay 1/3"),
            b"0.333\n".to_vec()
        );

        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(&mut interp, b"numeric digits 3\nnumeric digits\nsay 1/3"),
            b"0.333333333\n".to_vec(),
            "NUMERIC DIGITS alone resets to the package default, 9"
        );
    }

    #[test]
    fn numeric_digits_reset_reports_a_conflict_exactly_as_if_9_were_typed() {
        // numeric digits 20; numeric fuzz 15; numeric digits -> 33.1, ("9")
        // rejected against the still-15 fuzz -- measured against the oracle.
        let mut interp = Interp::new(false);
        let failure = run_source(
            &mut interp,
            b"numeric digits 20\nnumeric fuzz 15\nnumeric digits",
        )
        .unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (33, 1));
        assert_eq!(raised.additional, vec!["9".to_string(), "15".to_string()]);
    }

    #[test]
    fn numeric_fuzz_changes_comparison_and_resets_to_0_with_no_expression() {
        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(
                &mut interp,
                b"numeric digits 5\nnumeric fuzz 3\nsay (1.001 = 1)"
            ),
            b"1\n".to_vec()
        );

        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(
                &mut interp,
                b"numeric digits 5\nnumeric fuzz 3\nnumeric fuzz\nsay (1.001 = 1)"
            ),
            b"0\n".to_vec(),
            "NUMERIC FUZZ alone resets to the package default, 0"
        );
    }

    #[test]
    fn numeric_form_scientific_engineering_and_default() {
        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(&mut interp, b"numeric form engineering\nsay 1e10 + 0"),
            b"10E+9\n".to_vec()
        );

        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(&mut interp, b"numeric form scientific\nsay 1e10 + 0"),
            b"1E+10\n".to_vec()
        );

        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(
                &mut interp,
                b"numeric form engineering\nnumeric form\nsay 1e10 + 0"
            ),
            b"1E+10\n".to_vec(),
            "NUMERIC FORM alone resets to the package default, SCIENTIFIC"
        );
    }

    #[test]
    fn numeric_form_value_spellings() {
        // The keyword VALUE and the implicit `(expr)` spelling both set the
        // same setting.
        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(
                &mut interp,
                b"numeric form value 'ENGINEERING'\nsay 1e10 + 0"
            ),
            b"10E+9\n".to_vec()
        );

        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(&mut interp, b"numeric form ('ENGINEERING')\nsay 1e10 + 0"),
            b"10E+9\n".to_vec()
        );
    }

    #[test]
    fn numeric_form_value_is_not_case_insensitive() {
        // set_form_str's own rule: the runtime VALUE path does no
        // uppercasing -- measured, `numeric form value 'engineering'` is
        // 25.11 under the oracle.
        let mut interp = Interp::new(false);
        let failure = run_source(&mut interp, b"numeric form value 'engineering'").unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (25, 11));
    }

    // ---- EXIT ----

    #[test]
    fn exit_with_and_without_an_expression() {
        let mut interp = Interp::new(false);
        let value = run_source(&mut interp, b"exit").expect("bare exit runs");
        assert_eq!(value, None);

        let mut interp = Interp::new(false);
        let value = run_source(&mut interp, b"exit 42").expect("exit with a result runs");
        let value = value.expect("EXIT 42 carries a result");
        assert_eq!(&*interp.to_text(value), b"42");
    }

    #[test]
    fn exit_code_for_converts_the_result_the_way_the_oracle_does() {
        // The whole transcript this task's report re-verifies against the
        // oracle: a bare EXIT and a huge, fractional or non-numeric result
        // all leave the code at 0; a literal in i32 range converts exactly,
        // and an arithmetic result (EXIT's own unary minus counts) is
        // already rounded to the active NUMERIC DIGITS by the time it gets
        // here, which is where the negative-vs-positive asymmetry the report
        // measured against the oracle comes from.
        let mut interp = Interp::new(false);
        assert_eq!(interp.exit_code_for(None), 0, "a bare EXIT");

        let value = run_source(&mut interp, b"exit 2147483647")
            .unwrap()
            .expect("a result");
        assert_eq!(
            interp.exit_code_for(Some(value)),
            2147483647,
            "INT32_MAX exactly"
        );

        let value = run_source(&mut interp, b"exit 2147483648")
            .unwrap()
            .expect("a result");
        assert_eq!(
            interp.exit_code_for(Some(value)),
            0,
            "one past INT32_MAX falls back to 0"
        );

        let value = run_source(&mut interp, b"exit 5.9")
            .unwrap()
            .expect("a result");
        assert_eq!(interp.exit_code_for(Some(value)), 0, "fractional");

        let value = run_source(&mut interp, b"exit 5.0")
            .unwrap()
            .expect("a result");
        assert_eq!(
            interp.exit_code_for(Some(value)),
            5,
            "a whole number spelled with a point"
        );

        let value = run_source(&mut interp, b"exit 'abc'")
            .unwrap()
            .expect("a result");
        assert_eq!(interp.exit_code_for(Some(value)), 0, "non-numeric");

        // The asymmetry: -2147483647 is 0 - 2147483647, rounded to the
        // *active* DIGITS (9 by default) the moment it is created, landing
        // one past INT32_MIN; raising DIGITS before the subtraction removes
        // the rounding and the failure with it.
        let value = run_source(&mut interp, b"exit -2147483647")
            .unwrap()
            .expect("a result");
        assert_eq!(
            interp.exit_code_for(Some(value)),
            0,
            "rounded to 9 digits at creation, one past INT32_MIN"
        );

        let value = run_source(&mut interp, b"numeric digits 20\nexit -2147483647")
            .unwrap()
            .expect("a result");
        assert_eq!(
            interp.exit_code_for(Some(value)),
            -2147483647,
            "no rounding at DIGITS 20, exact"
        );
    }

    // ---- LABEL and NOP ----

    #[test]
    fn a_label_is_a_traced_no_op() {
        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(&mut interp, b"here: say 'hit'"),
            b"hit\n".to_vec(),
            "the label itself produces no output of its own"
        );
    }

    #[test]
    fn nop_is_a_no_op() {
        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(&mut interp, b"nop\nsay 'after'"),
            b"after\n".to_vec()
        );
    }

    // ---- IF/THEN/ELSE ----

    /// The discriminating shape for a wrong `false_target`/`then_exit`: a
    /// true condition whose `ELSE` branch has a **different** side effect
    /// than the `THEN` branch. A version that lets the true path fall
    /// through into the `ELSE` marker without skipping it (the naive
    /// fallthrough this task's whole design exists to avoid -- see
    /// `run_bounded`'s doc comment) runs *both* assignments and prints
    /// `abXc`; a version that never runs the `ELSE` branch on the false path
    /// at all prints `aXc` for the companion case below. Only the correct
    /// wiring prints `abc` and `aXc` respectively.
    #[test]
    fn if_then_else_runs_exactly_one_branch() {
        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(
                &mut interp,
                b"a = 'a'\nif 1 = 1 then a = a || 'b'\nelse a = a || 'X'\na = a || 'c'\nsay a"
            ),
            b"abc\n".to_vec(),
            "true: runs the THEN branch and skips the ELSE branch entirely"
        );

        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(
                &mut interp,
                b"a = 'a'\nif 1 = 0 then a = a || 'b'\nelse a = a || 'X'\na = a || 'c'\nsay a"
            ),
            b"aXc\n".to_vec(),
            "false: runs the ELSE branch and skips the THEN branch entirely"
        );
    }

    #[test]
    fn if_then_with_no_else_falls_through_on_false() {
        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(
                &mut interp,
                b"a = 'a'\nif 1 = 0 then a = a || 'b'\na = a || 'c'\nsay a"
            ),
            b"ac\n".to_vec()
        );
    }

    /// The `ast.rs`/`block.rs`-derived discriminating case: an `IF`/`ELSE
    /// IF`/`ELSE` chain (no `DO`, which Task 11 owns) where each link's
    /// `false_target` is the next link's own condition and the whole
    /// chain's resume point sits after all of them. Run once per branch so
    /// a wrong `false_target` (skipping or re-testing a link) and a wrong
    /// `then_exit`/resume (landing inside a later link, matching
    /// `if_else_chain.rex`'s own framing) are both visible.
    #[test]
    fn if_else_if_chain_takes_exactly_one_link() {
        let source = |n: &str| -> Vec<u8> {
            format!(
                "n = {n}\na = ''\nif n = 1 then a = a || 'one'\nelse if n = 2 then a = a || 'two'\nelse if n = 3 then a = a || 'three'\nelse a = a || 'other'\na = a || '-after'\nsay a"
            )
            .into_bytes()
        };
        for (n, expected) in [
            ("1", "one-after\n"),
            ("2", "two-after\n"),
            ("3", "three-after\n"),
            ("4", "other-after\n"),
        ] {
            let mut interp = Interp::new(false);
            assert_eq!(
                say_output(&mut interp, &source(n)),
                expected.as_bytes().to_vec(),
                "n = {n}"
            );
        }
    }

    #[test]
    fn if_condition_that_is_not_0_or_1_raises_34_1() {
        let mut interp = Interp::new(false);
        let failure = run_source(&mut interp, b"if 'x' then nop").unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (34, 1));
        assert_eq!(raised.additional, vec!["x".to_string()]);
    }

    /// A comma list is 34.6 regardless of which element fails, never 34.1 --
    /// re-checking `eval_logical_list`'s own result would misreport it.
    /// Measured against the oracle (brief's own transcript).
    #[test]
    fn if_condition_that_is_a_comma_list_raises_34_6_not_34_1() {
        let mut interp = Interp::new(false);
        let failure = run_source(&mut interp, b"if 'x', 1 then nop").unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (34, 6));
        assert_eq!(raised.additional, vec!["x".to_string()]);
    }

    #[test]
    fn a_true_comma_list_condition_is_an_and_of_its_parts() {
        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(&mut interp, b"if 1, 1 then say 'hit'\nelse say 'miss'"),
            b"hit\n".to_vec()
        );
        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(&mut interp, b"if 1, 0 then say 'hit'\nelse say 'miss'"),
            b"miss\n".to_vec()
        );
    }

    // ---- SELECT / WHEN ----

    #[test]
    fn select_when_runs_exactly_the_first_matching_when() {
        let source = |n: &str| -> Vec<u8> {
            format!(
                "n = {n}\nr = ''\nselect\n  when n = 1 then r = r || 'one'\n  when n = 2 then r = r || 'two'\n  when n = 3 then r = r || 'three'\n  otherwise r = r || 'other'\nend\nr = r || '-after'\nsay r"
            )
            .into_bytes()
        };
        for (n, expected) in [
            ("1", "one-after\n"),
            ("2", "two-after\n"),
            ("3", "three-after\n"),
            ("4", "other-after\n"),
        ] {
            let mut interp = Interp::new(false);
            assert_eq!(
                say_output(&mut interp, &source(n)),
                expected.as_bytes().to_vec(),
                "n = {n}"
            );
        }
    }

    /// The `select_when_bodies.rex` discriminating shape, reproduced without
    /// `DO` (Task 11's): each `WHEN`'s consequence is a nested `SELECT`
    /// spanning several flat instructions of its own
    /// (`Select`/`When`/`Then`/two assignments/`End`), so a wrong exit that
    /// lands even one instruction into a *later* `WHEN`'s span, rather than
    /// cleanly past the whole outer `SELECT`, shows up as an extra or
    /// missing accumulator update. Run for every `n` so a wrong exit wired
    /// to (for instance) always resume after the *first* `WHEN` would still
    /// be caught on `n = 2` or `n = 3`.
    #[test]
    fn select_when_wrong_exit_would_land_in_a_later_whens_multi_instruction_body() {
        let source = |n: &str| -> Vec<u8> {
            format!(
                "n = {n}\nr = ''\nselect\n  when n = 1 then select\n    when 1 = 1 then r = r || 'w1a'\n    otherwise nop\n  end\n  when n = 2 then select\n    when 1 = 1 then r = r || 'w2a'\n    otherwise nop\n  end\n  when n = 3 then select\n    when 1 = 1 then r = r || 'w3a'\n    otherwise nop\n  end\n  otherwise r = r || 'w4a'\nend\nr = r || '-done'\nsay r"
            )
            .into_bytes()
        };
        for (n, expected) in [
            ("1", "w1a-done\n"),
            ("2", "w2a-done\n"),
            ("3", "w3a-done\n"),
            ("4", "w4a-done\n"),
        ] {
            let mut interp = Interp::new(false);
            assert_eq!(
                say_output(&mut interp, &source(n)),
                expected.as_bytes().to_vec(),
                "n = {n}: exactly one nested SELECT's body ran, and nothing else did"
            );
        }
    }

    /// `when 1 = 1 then` followed immediately by `when 2 = 2 then n = 42` is
    /// the second `WHEN`'s absorption into the first's (empty) consequence
    /// (`ast.rs`'s own doc comment on `Select::whens`): the second `WHEN` is
    /// never collected, and its own `exit` is permanently `None`. Measured
    /// against the oracle: prints `0`, rc 0 -- the absorbed `WHEN`'s own
    /// condition being true (`2 = 2`) must not run `n = 42`, and `OTHERWISE`
    /// must not run either, because one true `WHEN` (the outer one) already
    /// ended the whole `SELECT`.
    #[test]
    fn an_absorbed_when_runs_neither_its_own_consequence_nor_otherwise() {
        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(
                &mut interp,
                b"n = 0\nselect\n  when 1 = 1 then\n    when 2 = 2 then n = 42\n  otherwise\n    n = 99\nend\nsay n"
            ),
            b"0\n".to_vec()
        );
    }

    /// The true-condition-variant is accepted at rc 0 (`ast.rs:776`, and the
    /// brief's own transcript) -- the *false*-condition variant is a
    /// separate, upstream oracle segfault (SF #2018) and is deliberately not
    /// probed here.
    #[test]
    fn when_absorbing_a_when_parses_and_runs_at_rc_0() {
        let mut interp = Interp::new(false);
        assert_eq!(
            run_source(
                &mut interp,
                b"select\n  when 1 = 1 then\n    when 2 = 2 then nop\nend"
            )
            .expect("accepted, rc 0"),
            None
        );
    }

    #[test]
    fn select_with_no_when_true_and_no_otherwise_raises_7_3() {
        let mut interp = Interp::new(false);
        let failure = run_source(&mut interp, b"select\n  when 1 = 0 then nop\nend").unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (7, 3));
        assert_eq!(raised.additional, Vec::<String>::new());
    }

    #[test]
    fn select_with_no_when_true_and_an_otherwise_runs_it_without_error() {
        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(
                &mut interp,
                b"select\n  when 1 = 0 then nop\n  otherwise say 'lo'\nend"
            ),
            b"lo\n".to_vec()
        );
    }

    #[test]
    fn when_condition_that_is_not_0_or_1_raises_34_2() {
        let mut interp = Interp::new(false);
        let failure = run_source(&mut interp, b"select\n  when 'x' then nop\nend").unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (34, 2));
        assert_eq!(raised.additional, vec!["x".to_string()]);
    }

    // ---- SELECT CASE / WhenCase ----

    /// **The central `WhenCase` rule.** `select case 2` / `when 1, 2 then`
    /// matches (an OR of `==`), while a plain `select` / `when 1, 2` on the
    /// same non-logical value is 34.6 (an AND, each element checked for
    /// `0`/`1`) -- the two commas parse into the same-looking node and mean
    /// opposites (`ast.rs:801-815`, and the brief's own framing).
    #[test]
    fn whencase_comma_is_an_or_of_equals_the_opposite_of_a_plain_whens_and() {
        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(
                &mut interp,
                b"select case 2\n  when 1, 2 then say 'hit'\n  otherwise say 'miss'\nend"
            ),
            b"hit\n".to_vec(),
            "SELECT CASE: an OR of == comparisons"
        );

        let mut interp = Interp::new(false);
        let failure = run_source(&mut interp, b"select\n  when 1, 2 then nop\nend").unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!(
            (raised.number, raised.sub),
            (34, 6),
            "plain SELECT: 2 is not a logical value, an AND-with-a-check, not an OR-of-=="
        );
        assert_eq!(raised.additional, vec!["2".to_string()]);
    }

    /// `==` is strict: byte-for-byte, no padding, no numeric awareness.
    /// Measured (D15's own example, restated in the brief): `'007'` does not
    /// match `when 7`, because the two are not byte-identical.
    #[test]
    fn select_case_compares_with_strict_equality_not_numeric_equality() {
        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(
                &mut interp,
                b"select case '007'\n  when 7 then say 'hit'\n  otherwise say 'miss'\nend"
            ),
            b"miss\n".to_vec()
        );

        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(
                &mut interp,
                b"select case 7\n  when 7 then say 'hit'\n  otherwise say 'miss'\nend"
            ),
            b"hit\n".to_vec()
        );
    }

    #[test]
    fn select_case_evaluates_its_own_expression_and_runs_exactly_the_matching_when() {
        let source = |v: &str| -> Vec<u8> {
            format!("select case {v}\n  when 1 then say 'one'\n  when 2 then say 'two'\n  otherwise say 'other'\nend")
                .into_bytes()
        };
        for (v, expected) in [("1", "one\n"), ("2", "two\n"), ("3", "other\n")] {
            let mut interp = Interp::new(false);
            assert_eq!(
                say_output(&mut interp, &source(v)),
                expected.as_bytes().to_vec(),
                "v = {v}"
            );
        }
    }

    // ---- SELECT with a LABEL, and a nested SELECT/IF ----

    #[test]
    fn a_labelled_select_with_otherwise_runs_normally() {
        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(
                &mut interp,
                b"select label s\n  when 1 = 0 then nop\n  otherwise say 'lo'\nend"
            ),
            b"lo\n".to_vec()
        );
    }

    #[test]
    fn a_select_nested_inside_an_ifs_then_branch_is_fully_resolved_before_resuming() {
        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(
                &mut interp,
                b"a = 'a'\nif 1 = 1 then select\n  when 1 = 1 then a = a || 'b'\nend\na = a || 'c'\nsay a"
            ),
            b"abc\n".to_vec()
        );
    }
}
