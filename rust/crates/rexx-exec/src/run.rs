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

use crate::error::Raised;
use crate::{Code, Failure, Interp, Loud};
use rexx_core::ObjRef;
use rexx_num::SettingsError;
use rexx_parse::{
    Expr, ExprKind, Fragment, Instruction, InstructionKind, NumericSetting, VariableRef,
    compound_parts, parse_interpret,
};
use std::rc::Rc;

/// Where control goes after one instruction (the design's "Control flow").
enum Flow {
    Next,
    /// Unreachable in the spike, which has no jump, and unreachable in a
    /// fragment for a measured reason: a label inside `INTERPRET` text is
    /// error 47.1 (Task 1), so a fragment's `labels` is always empty and a
    /// fragment can never jump.
    #[allow(dead_code, reason = "Tasks 10 and 11 build the jumps that produce it")]
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
            let flow = match self.step_in_temps_frame(&code, instruction) {
                Ok(flow) => flow,
                Err(failure) => {
                    // The last place the failing instruction is in hand.
                    // `run` pops this activation on the way out, so a site
                    // resolved any higher up would have nothing left to
                    // resolve against.
                    //
                    // A condition raised inside an `INTERPRET` fragment
                    // arrives here too, and records the enclosing `INTERPRET`
                    // clause rather than the fragment's own, because the
                    // fragment loop deliberately does not record: its spans
                    // index the fragment's source and not this one, so the
                    // line they resolve to would be the fragment's.
                    //
                    // The oracle prints **both**, innermost first, each
                    // carrying the enclosing clause's line number (measured,
                    // `interpret "say 2 & 1"` on line 2):
                    //
                    // ```text
                    //      2 *-* say 2 & 1
                    //      2 *-* interpret "say 2 & 1"
                    // ```
                    //
                    // so this reproduces the second of those lines and not the
                    // first. A known gap rather than something to fix here:
                    // stacking one echo per nesting level changes
                    // `Raised::report`'s shape, and 4a's only nesting is the
                    // fragment spike that 4b deletes.
                    let source = &program.source;
                    self.failure_site = Some((
                        source.line_of(instruction.clause_span.start),
                        source
                            .join_span(instruction.clause_span.clone())
                            .map_or_else(
                                // Visible rather than silent, for the same
                                // reason `Raised::message` renders a catalogue
                                // miss instead of panicking: the error path is
                                // the worst place to turn a reportable
                                // condition into a crash or into a blank line.
                                || b"<clause span outside the retained source>".to_vec(),
                                |bytes| bytes.into_owned(),
                            ),
                    ));
                    return Err(failure);
                }
            };
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
    fn step(&mut self, code: &Code<'_>, instruction: &Instruction) -> Result<Flow, Failure> {
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
    fn step_in_temps_frame(
        &mut self,
        code: &Code<'_>,
        instruction: &Instruction,
    ) -> Result<Flow, Failure> {
        let frame = self.roots.push_frame();
        let flow = self.step(code, instruction);
        self.roots.pop_frame(frame);
        flow
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
    ///   instruction and has to still be there afterwards. A local is enough
    ///   for the measured reason that a fragment can never jump: a label
    ///   inside `INTERPRET` text is error 47.1 (Task 1), so `body.labels` is
    ///   always empty.
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

        let mut pc = 0;
        while let Some(instruction) = code.body.instructions.get(pc) {
            match self.step_in_temps_frame(&code, instruction)? {
                Flow::Next => pc += 1,
                Flow::Goto(_) => unreachable!("a fragment has no labels, so it cannot jump (47.1)"),
                // `exit` inside `INTERPRET` ends the program, not the
                // fragment, so this propagates rather than stopping here.
                Flow::Exit(value) => return Ok(Flow::Exit(value)),
            }
        }
        Ok(Flow::Next)
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

    /// Parses `source`, activates it, and runs every one of its instructions
    /// through `step_in_temps_frame` in order -- a miniature `run_activation`
    /// for a program that never branches, which is every program this
    /// module's tests write. Goes through the wrapper and not `step`
    /// directly: `step_in_temps_frame` is the chokepoint that unconditionally
    /// closes each instruction's temps frame, including on the failure path,
    /// and calling `step` around it would leave this the one caller in the
    /// crate that skips it -- exactly the shape a future non-test caller
    /// could copy by example. `slots` is an empty map throughout:
    /// `read`/`slot_of`'s fallback chain answers correctly without the real
    /// plan's fast path, the same choice `eval.rs`'s own test helpers make.
    fn run_source(interp: &mut Interp, source: &[u8]) -> Result<Option<ObjRef>, Failure> {
        let program = parse_program(source.to_vec()).expect("test program parses");
        let program = activate(interp, program);
        let code = Code {
            body: &program.main,
            symbols: &program.symbols,
            slots: &HashMap::new(),
        };
        for instruction in &code.body.instructions {
            match interp.step_in_temps_frame(&code, instruction)? {
                Flow::Next => {}
                Flow::Goto(_) => unreachable!("no test program in this module branches"),
                Flow::Exit(value) => return Ok(value),
            }
        }
        Ok(None)
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
}
