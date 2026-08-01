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
//! at all. `run_bounded`'s doc comment carries the resolution. `Then`,
//! `Else`, `Otherwise`, `When` and `WhenCase` accordingly step as pure
//! no-ops (like `Label`) -- they are never independently dispatched, only
//! ever read as data by `If`/`Select` or walked over inside a bounded loop.

use crate::error::{FailureSite, Raised};
use crate::eval::logical_value;
use crate::trace::{
    is_whole_number, mode_from_setting, raised_invalid_trace_letter,
    raised_numeric_trace_interactive_only,
};
use crate::{Code, Failure, Interp, Loud};
use rexx_core::ObjRef;
use rexx_num::{ArithError, CompareOp, Number, SettingsError, compare_decoded};
use rexx_parse::{
    ControlExpr, Controlled, EndStyle, Expr, ExprKind, Fragment, Instruction, InstructionKind,
    Loop, LoopConditional, LoopKind, NumericSetting, ProgramSource, SymbolId, Trace, VariableRef,
    compound_parts, parse_interpret,
};
use std::rc::Rc;

/// Where control goes after one instruction (the design's "Control flow").
enum Flow {
    Next,
    /// Live since Task 10: `If`/`Select` each resolve to one `Goto` that
    /// skips straight to their construct's true resume point, and
    /// `run_bounded`'s own internal loop applies one whenever a nested
    /// construct's target lands inside its range.
    ///
    /// **This includes inside a fragment.** A label inside `INTERPRET` text
    /// is still error 47.1 (Task 1), so a fragment's `labels` is still
    /// always empty and nothing there can jump to a *label* -- but Task 10
    /// makes `IF`/`SELECT` able to appear (and jump) anywhere a `Code` body
    /// can be stepped, a fragment's own included, with no label involved at
    /// all. `run_fragment` no longer has an `unreachable!` on this variant.
    /// It now runs through `run_bounded` for exactly this reason, and
    /// `run_fragment`'s own doc comment carries the argument for why every
    /// jump such a construct computes stays inside the fragment's own range.
    Goto(usize),
    Exit(Option<ObjRef>),
    /// `LEAVE`, bare (`None`) or by name. Live since Task 11.
    ///
    /// **Why this is a `Flow` variant and not an immediate `Err`:** `LEAVE`
    /// finding its own target is not a failure, so it has to travel as data
    /// through exactly the same channel `Goto`/`Exit` already do --
    /// `run_bounded`'s own catch-all (`other => return Ok(other)`, its doc
    /// comment names this variant by name as the reason it exists) forwards
    /// it outward through any nested `IF` untouched, and `Do`/`Select`'s own
    /// arms are the only two that ever inspect it, matching the oracle's own
    /// rule that only a `SELECT`/`DO`/`LOOP` block ever participates in the
    /// search (`RexxActivation::leaveLoop`, read directly and cited in the
    /// report -- `IF`/`THEN`/`ELSE` never push a frame at all, so they are
    /// transparent by construction, not by a case this crate has to add).
    ///
    /// **The invariant this crate's whole `Do`/`Select` design holds so this
    /// variant is safe to use at all**: unlike `Goto`, a `Do`'s own
    /// repetition is never expressed as a `Goto` back to its own body's top.
    /// The trap that would create is exactly `run_bounded`'s own doc comment
    /// warns about for a future `LEAVE`/`ITERATE` variant -- a `Goto` whose
    /// target lands inside an *enclosing* `IF`/`SELECT`'s own range (a `DO`
    /// nested in an `IF`'s `THEN`, iterating) is absorbed by that enclosing
    /// `run_bounded` directly, never seen by the `DO`'s own arm again, which
    /// would re-enter it as a first entry with its own state reset. `Do`'s
    /// own arm therefore never returns to its caller until the *entire*
    /// loop -- every iteration -- is over, one way or another: it drives its
    /// own `run_bounded(body)` calls in an internal `loop {}` and only ever
    /// returns a `Flow` once there is truly nothing left for it to decide.
    /// `leave_and_iterate_survive_a_goto_absorbing_enclosing_if` (this
    /// file's own tests) pins exactly this shape: a `DO` with an `ITERATE`
    /// in its body, nested inside an `IF`'s `THEN`, run enough times that a
    /// version which instead returned a re-entry `Goto` to the loop's own
    /// top would either loop forever (the `IF`'s own `run_bounded` silently
    /// re-entering the `DO` as a fresh first pass on every `Goto`) or lose
    /// the loop's own running total, depending on exactly how such a bug
    /// were shaped -- either way, not the correct, small, printed result the
    /// test asserts.
    ///
    /// The payload eagerly resolves the `LEAVE`/`ITERATE` instruction's own
    /// clause and static indent (`LeaveOrigin`) at the moment it steps,
    /// before any propagation: **28.1-28.4 (the "found nothing at all"
    /// family) and 28.5 (the "found a name match, but it names something
    /// that is not a loop" family) report at two different, both
    /// oracle-measured, indentations that no longer-lived state can recover
    /// once the search has moved on** -- see the report's own transcripts.
    Leave(Option<SymbolId>, LeaveOrigin),
    /// `ITERATE`, bare or by name. See `Leave`'s own doc comment; the two
    /// variants are handled by nearly identical logic in `Do`/`Select`'s own
    /// arms, differing only in which of the oracle's measured asymmetries
    /// applies (`Select` never consumes a bare `Iterate` at all, and a named
    /// one that matches its own label but is not a loop is 28.5, not simply
    /// "not mine, keep looking").
    Iterate(Option<SymbolId>, LeaveOrigin),
}

/// Where a `LEAVE`/`ITERATE` instruction itself sits, captured the instant
/// it steps rather than reconstructed later -- see `Flow::Leave`'s own doc
/// comment for why eagerly.
///
/// **`indent`'s own rule, corrected after review.** `indent` starts as
/// `static_indent` applied to the `LEAVE`/`ITERATE`'s own index -- its full
/// lexical depth, computed once when it steps -- and from there is the
/// search's own running **residual**, updated (not merely read) as the
/// `Flow` this is attached to propagates outward: every `SELECT` (always)
/// and every `DO`/`LOOP` that either `is_loop` or carries an explicit
/// `LABEL` (i.e. every one **except** an unlabelled `Simple` block) "owns a
/// search frame," and when such a construct examines this `Flow` and does
/// **not** consume it, it resets `indent` to *its own* `static_indent`
/// (`pop_search_frame`) before forwarding -- mirroring the oracle's own
/// `popBlockInstruction`, which restores `traceIndent` to the value saved
/// when the frame it is popping was pushed. A construct that *does*
/// consume the `Flow` (a match, successful or 28.5) does **not** reset
/// anything itself; whatever `indent` already holds at that point is the
/// answer. An unlabelled `Simple` block owns no frame and is fully
/// transparent, exactly like `IF` (which never even sees this `Flow` at
/// all, since it is not a block instruction and forwards everything
/// through ordinary fallthrough).
///
/// This first shipped as two hardcoded shapes -- 28.1-28.4 always zero,
/// 28.5 always the origin's own unmodified full lexical depth -- which
/// happened to match every probe behind it because none of them mixed an
/// `IF`/unlabelled-`Simple` intervenor with a `SELECT`/`DO`/`LOOP` one. A
/// reviewer's fourteen-point probe (nine of theirs, plus this task's own
/// five re-measured and added afterward) falsified seven of the fourteen
/// under that rule and fits all fourteen under this one; the report has
/// every transcript.
struct LeaveOrigin {
    /// `None` only when `source` was `None` at the moment this instruction
    /// stepped (inside an `INTERPRET` fragment, `run_fragment`'s own
    /// established convention of not resolving a site there at all).
    site: Option<(usize, Vec<u8>)>,
    indent: usize,
}

/// What drives one repeating `DO`/`LOOP`'s own iteration, once its header
/// has already been evaluated and validated -- everything `LoopKind` can be
/// except `Simple` (a block, never repeats, and `run_loop`'s own `Simple`
/// arm never builds one of these at all) and `With` (the loud path).
///
/// `Count`, `OverOnce` and `Controlled` all decrement whatever budget the
/// oracle is measured to decrement once per candidate iteration, including
/// one an `ITERATE` cuts short (measured: a `FOR 3` loop with an `ITERATE`
/// on its first pass still stops after exactly three iterations, not four).
enum LoopState {
    Forever,
    /// `DO expr`: a fixed repeat count, decremented to zero.
    Count {
        remaining: u64,
    },
    /// `DO name OVER expr`, a **non-stem** target only (Deviation 1: a stem
    /// target takes the loud path in `run_loop` before one of these is ever
    /// built): iterates exactly once, binding `control` to `value` itself
    /// (measured, the brief's own framing: "a string and a number each
    /// iterate once, yielding themselves"). `remaining` is `FOR`'s own
    /// budget, already validated, independent of `done`.
    OverOnce {
        control: SymbolId,
        value: ObjRef,
        done: bool,
        remaining: Option<u64>,
    },
    /// `DO i = initial TO to BY by FOR for_count`. `to`/`for_remaining` are
    /// `None` when that keyword was not written at all (an absent `TO`
    /// loops until `LEAVE` or `FOR` stops it, exactly like `FOREVER` with a
    /// control variable riding along); `by` is never absent here --
    /// `setup_controlled` already defaulted it to `1`.
    Controlled {
        control: SymbolId,
        current: Number,
        to: Option<Number>,
        by: Number,
        for_remaining: Option<u64>,
    },
}

/// What `eval_condition` should do with the value it just computed, beyond
/// answering the caller's `bool` -- a caller-chosen variant rather than a
/// decision `eval_condition` makes on its own, because the same function
/// serves `IF`/`WHEN` (their own `>>>`, measured) and `WHILE`/`UNTIL`
/// (their own `>K>` instead, never a bare `>>>` alongside it, also
/// measured) and the two are genuinely different oracle behaviours, not
/// two spellings of one.
enum ConditionTrace<'a> {
    /// `IF`/`WHEN`'s own `>>>`.
    Result(usize),
    /// `WHILE`/`UNTIL`'s own `>K>`, tagged `"WHILE"`/`"UNTIL"`.
    Keyword(usize, &'a str),
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
                // Task 11: a `LEAVE`/`ITERATE` that reached the very top of
                // the program -- nothing anywhere, at any nesting depth,
                // ever matched it. This is the exhausted-search family,
                // 28.1 (bare `LEAVE`)/28.2 (bare `ITERATE`)/28.3 (named
                // `LEAVE`)/28.4 (named `ITERATE`). `origin.indent` already
                // holds this family's own answer by the time it gets here
                // -- every `Select`/`Do` frame the search walked through on
                // the way up has already reset it to its own `static_indent`
                // as it forwarded past (`LeaveOrigin`'s own doc comment has
                // the rule, corrected after review: it is **not** always
                // zero, only when every popped frame along the way happened
                // to sit at top level). 28.5 (a named `ITERATE` that *did*
                // match something, just not a loop) is a different family,
                // raised where the match was found, in `Select`/`Do`'s own
                // arms, and never reaches here.
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
                    // `say` with no expression is a blank line. Still
                    // traced (`RexxInstructionExpression::
                    // evaluateStringExpression`'s own `else` arm:
                    // `traceResult(GlobalNames::NULLSTRING)`), as an empty
                    // string, not skipped.
                    None => Vec::new(),
                };
                self.trace_result(static_indent(&code.body.instructions, index), &line);
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
                // `>>>` fires before the assignment itself
                // (`RexxInstructionAssignment::execute`: evaluate, trace,
                // *then* assign), which matters only in that the traced
                // value can never be affected by the write it precedes.
                let indent = static_indent(&code.body.instructions, index);
                let rendered = self.to_text(value).to_vec();
                self.trace_result(indent, &rendered);
                match &target.kind {
                    ExprKind::Variable(id) => {
                        let name = code.symbols.name(*id).as_bytes().to_vec();
                        let slot = self.slot_of(&name);
                        let frame = self.activation().frame;
                        self.roots.set_slot(frame, slot, value);
                        self.trace_assignment(indent, &name, &rendered);
                    }
                    // `stem. = expr`: replace-and-rebind (D15a), through the
                    // library `stem_assign` already builds -- this arm is the
                    // dispatch Task 9 owns, not new stem logic.
                    ExprKind::Stem(id) => {
                        let name = code.symbols.name(*id).as_bytes().to_vec();
                        self.stem_assign(&name, value);
                        self.trace_assignment(indent, &name, &rendered);
                    }
                    // `a.b = expr`: resolve the tail key the same way reading
                    // `a.b` would (`eval_node`'s own `Compound` arm), then
                    // mutate that one tail in place through `stem_set`.
                    //
                    // `>C>` before `>=>` (`RexxActivation.cpp:4791`-`4802`'s
                    // own order for a *read*; measured, this task's report,
                    // that a *write* through `ExpressionCompoundVariable::
                    // assign` announces the same resolved name first too):
                    // the tag is the compound's own **source spelling**
                    // (`a.i` stays `A.I` regardless of `i`'s value), and the
                    // resolved name is `stem_name` (the read site's own,
                    // matching `stem_set`'s own convention) concatenated
                    // with `key`.
                    ExprKind::Compound(id) => {
                        let tag = code.symbols.name(*id).as_bytes().to_vec();
                        let (stem_name, _tails) = compound_parts(code.symbols.name(*id));
                        let stem_name = stem_name.as_bytes().to_vec();
                        let key = self.tail_key(code, *id);
                        self.stem_set(&stem_name, &key, value);
                        let mut resolved = stem_name;
                        resolved.extend_from_slice(&key);
                        self.trace_compound_name(indent, &tag, &resolved);
                        self.trace_assignment(indent, &tag, &rendered);
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

            // `TRACE` (D17): sets `self.trace_mode`, or raises 24.901 for
            // the interactive-only skip-count forms. See `exec_trace`.
            InstructionKind::Trace(setting) => {
                self.exec_trace(code, setting)?;
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
                let indent = static_indent(&code.body.instructions, index);
                if self.eval_condition(
                    code,
                    condition,
                    ConditionTrace::Result(indent),
                    raised_if_not_logical,
                )? {
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
                    // that is only true of the false path. See
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
                label,
                case,
                whens,
                otherwise,
                end,
            } => {
                let len = code.body.instructions.len();
                let select_indent = static_indent(&code.body.instructions, index);
                let case_text = match case {
                    Some(case_expr) => {
                        let value = self.eval(code, case_expr)?;
                        self.roots.push_temp(value);
                        let text = self.to_text(value).to_vec();
                        // `>K>` (`SelectInstruction.cpp:372`,
                        // `traceKeywordResult(CASE, ...)`), at the
                        // `SELECT`'s own level -- measured, this task's
                        // report, `>K>   "CASE" => "2"` sits at the same
                        // indent as `select case ...` itself, not the
                        // `WHEN`-scan level `WhenCase`'s own comparison
                        // lines (below) are indented to.
                        self.trace_keyword(select_indent, "CASE", &text);
                        Some(text)
                    }
                    None => None,
                };
                for &when_index in whens {
                    let when_instruction = &code.body.instructions[when_index];
                    // `When`/`WhenCase`'s own clause echo, explicit for the
                    // same reason `record_failure_site`'s own call below is:
                    // that instruction's `step` arm is a pure no-op (never
                    // independently dispatched, only ever read as data
                    // here), so nothing else ever calls `step_in_temps_
                    // frame` for it and its `*-*` line would otherwise never
                    // appear at all -- measured, `select` / `when 1 = 1
                    // then ...` echoes the `WHEN`'s own clause on its own
                    // line before anything about its condition.
                    let when_indent = static_indent(&code.body.instructions, when_index);
                    // Overrides the enclosing `SELECT`'s own
                    // `current_value_indent` (`step_in_temps_frame` set it
                    // to `select_indent` before this arm even started) for
                    // exactly the same reason the explicit clause echo just
                    // above is explicit: this condition is evaluated
                    // outside any `step_in_temps_frame` call of its own.
                    self.current_value_indent = when_indent;
                    if self.trace_mode.all
                        && let Some((line, text)) = clause_site(source, when_instruction)
                    {
                        self.trace_clause(line, when_indent, &text);
                    }
                    // Every fallible call below is matched explicitly,
                    // never through `?`, so a failure can be attributed to
                    // `when_instruction` -- the `When`/`WhenCase` whose
                    // condition is actually being evaluated -- before it
                    // propagates. Nothing here goes through
                    // `step_in_temps_frame` at all: `When`/`WhenCase`'s own
                    // `step` arm is a no-op (see its own doc comment), so
                    // without this a raise here would still be attributed
                    // to this `SELECT` instruction, which is exactly the
                    // defect `record_failure_site`'s own doc comment
                    // describes. Measured: `select` / `when 'x' then nop` /
                    // `end` must report the `WHEN`'s own line and clause,
                    // not the `SELECT`'s.
                    let outcome = match &when_instruction.kind {
                        InstructionKind::When {
                            condition,
                            false_target,
                            exit,
                        } => {
                            let holds = match self.eval_condition(
                                code,
                                condition,
                                ConditionTrace::Result(when_indent),
                                raised_when_not_logical,
                            ) {
                                Ok(holds) => holds,
                                Err(failure) => {
                                    self.record_failure_site(
                                        code,
                                        when_index,
                                        source,
                                        when_instruction,
                                    );
                                    return Err(failure);
                                }
                            };
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
                            let matched =
                                match self.test_case_when(code, values, case_text, when_indent) {
                                    Ok(matched) => matched,
                                    Err(failure) => {
                                        self.record_failure_site(
                                            code,
                                            when_index,
                                            source,
                                            when_instruction,
                                        );
                                        return Err(failure);
                                    }
                                };
                            matched.then(|| (false_target.unwrap_or(len), exit.unwrap_or(len)))
                        }
                        other => panic!("a SELECT's whens holds only When/WhenCase, not {other:?}"),
                    };
                    if let Some((body_end, resume)) = outcome {
                        let flow = self.run_bounded(code, when_index + 1, body_end, source)?;
                        return self.leave_select(code, index, *label, resume, flow);
                    }
                }
                match otherwise {
                    // Task 11: **used to be** a plain `Goto` onto the
                    // `OTHERWISE` marker, left for the outer loop's own
                    // ordinary fallthrough to run -- correct for clause
                    // attribution (nothing here ever needed a bounded
                    // sub-loop to get *that* right, and the fallthrough
                    // still lands exactly on `END` with nothing to skip,
                    // same as before), but wrong for `LEAVE`/`ITERATE`: this
                    // `SELECT` never gets a chance to recognise its own
                    // label from inside a branch it does not itself call
                    // `run_bounded` over. So `OTHERWISE`'s own body is now
                    // `[otherwise_index + 1, end)`, run the same way a
                    // matched `WHEN`'s is -- attribution is unaffected
                    // (`step_in_temps_frame`'s own resolution does not care
                    // which loop dispatched it), and a `LEAVE`/`ITERATE`
                    // naming this `SELECT`'s own label from inside
                    // `OTHERWISE` is now caught here too.
                    Some(otherwise_index) => {
                        let otherwise_end = end.unwrap_or(len);
                        // `OTHERWISE`'s own clause echo, explicit for the
                        // same reason `WHEN`'s own is, above: its `step`
                        // arm is a no-op and its own index is never inside
                        // any `run_bounded` range (the body below starts
                        // *after* it), so nothing else ever visits it.
                        // Measured, this task's report: `otherwise` traces
                        // on its own line, at the `SELECT`'s own scan
                        // level, before its body.
                        let otherwise_instruction = &code.body.instructions[*otherwise_index];
                        // `static_indent`'s own fixed answer for
                        // `*otherwise_index` (this task's earlier fix, not
                        // `select_indent`): the marker sits at the scan
                        // level (2 at top level), the same as a `WHEN`'s
                        // own condition, not the `SELECT`'s own level.
                        let otherwise_indent =
                            static_indent(&code.body.instructions, *otherwise_index);
                        self.current_value_indent = otherwise_indent;
                        if self.trace_mode.all
                            && let Some((line, text)) = clause_site(source, otherwise_instruction)
                        {
                            self.trace_clause(line, otherwise_indent, &text);
                        }
                        let flow =
                            self.run_bounded(code, otherwise_index + 1, otherwise_end, source)?;
                        self.leave_select(code, index, *label, otherwise_end, flow)
                    }
                    // Landing exactly on `END` is deliberate: that is what
                    // makes 7.3's clause echo the `END`'s and not the
                    // `SELECT`'s (`End`'s own arm, below, is where it
                    // raises). No body ran, so there is nothing for a
                    // `LEAVE`/`ITERATE` to have escaped from here.
                    None => Ok(Flow::Goto(end.unwrap_or(len))),
                }
            }

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
            //
            // The old comment's own measurement was real but its conclusion
            // was wrong: `select / when 1=1 then / when 2=2 then n=42 /
            // otherwise / n=99 / end / say n` prints `0`, and that alone is
            // consistent with *never evaluating* condition B just as much
            // as with *evaluating it and discarding the answer*. The
            // measurement that tells the two apart is a raising absorbed
            // condition: `select / when 1=1 then / when 1/0 then nop / end`
            // is rc **214**, `Error 42.3`, on the oracle -- so B's condition
            // *is* evaluated for real, and simply never gets to take its
            // own branch (confirmed the other direction too: `when 2=2
            // then say 'x'` with no `otherwise` prints nothing but `after`,
            // not `x` -- true or false, the absorbed consequence never
            // runs). `Select`'s own arm already reasons this exactly right
            // for a *listed* `WHEN`'s condition (raise first, decide
            // second); this arm was the one place that reasoning did not
            // reach, because nothing before this task's own review probed
            // a raising absorbed condition -- every prior probe used a
            // side-effect-free true one, which cannot distinguish the two
            // models.
            //
            // `current_value_indent` is already correct here without any
            // extra work: this instruction *is* being stepped through the
            // ordinary `step_in_temps_frame` path (unlike a listed `WHEN`,
            // which `Select`'s own arm overrides it for explicitly), so the
            // clause echo and this trace both land at this absorbed
            // clause's own static indent.
            InstructionKind::When { condition, .. } => {
                self.eval_condition(
                    code,
                    condition,
                    ConditionTrace::Result(self.current_value_indent),
                    raised_when_not_logical,
                )?;
                Ok(Flow::Next)
            }
            // `SELECT CASE`'s own absorbed form. Evaluates every `values`
            // expression, for the identical reason (side effects, and a
            // raise must escape) -- **known gap, not attempted**: with no
            // access to the enclosing `SELECT CASE`'s own `case` text here
            // (nothing threads it to an absorbed node, unlike a listed
            // `WhenCase`, which `Select`'s own arm already has it in hand
            // for), this cannot reproduce `test_case_when`'s own two-line
            // `>>>` comparison pair for the absorbed case, only evaluate
            // and trace each value on its own. No corpus or spec example
            // exercises an absorbed `WHEN CASE`; this is deliberately
            // proportionate to that, not a claim that it is fully correct.
            InstructionKind::WhenCase { values, .. } => {
                for value in values {
                    let v = self.eval(code, value)?;
                    self.roots.push_temp(v);
                }
                Ok(Flow::Next)
            }
            InstructionKind::Otherwise => Ok(Flow::Next),

            // `DO`/`LOOP`, every kind but `DO WITH` (the loud path,
            // `run_loop`'s own doc comment) -- Task 11. Resolves the whole
            // construct itself, every iteration, exactly the discipline
            // `If`/`Select` already established: see `Flow::Leave`'s own
            // doc comment for why `Do`'s own arm never returns until the
            // entire loop is over, one way or another.
            InstructionKind::Do(body) | InstructionKind::Loop(body) => {
                self.run_loop(code, index, instruction, body, source)
            }

            // `LEAVE`/`ITERATE`, bare or by name -- Task 11. Resolves to
            // data, not a failure (`Flow::Leave`'s own doc comment): whether
            // this instruction's own name matches anything is answered by
            // whichever `Do`/`Select` (or `run_activation`'s own top level,
            // if none does) inspects the `Flow` this returns, never here.
            InstructionKind::Leave { name } => Ok(Flow::Leave(
                *name,
                self.leave_origin(code, index, source, instruction),
            )),
            InstructionKind::Iterate { name } => Ok(Flow::Iterate(
                *name,
                self.leave_origin(code, index, source, instruction),
            )),

            // `END`. `Select`'s two non-7.3 closings (`OTHERWISE` present)
            // are reached only by that `OTHERWISE`'s own ordinary body
            // fallthrough and do nothing. `EndStyle::Select`'s own doc
            // comment: "Reaching this END at run time is error 7.3, because
            // every WHEN was false" -- which is exactly the one path that
            // lands here, since `Select`'s own arm above sends every other
            // path around this instruction entirely.
            //
            // **`EndStyle::Do`/`LabeledDo`/`Loop` used to fail loudly here,
            // and no longer do.** Task 11's `Do`/`Loop` arm now resolves its
            // own construct exactly the way `If`/`Select` already do,
            // returning a `Goto` past this exact instruction on every exit
            // path -- normal completion, a consumed `LEAVE`, or `UNTIL`
            // finally holding. So this is reached only inside a bounded
            // sub-loop, as inert filler, precisely like `Then`/`Else`/
            // `When`/`Otherwise` above; nothing independently dispatches it
            // for a decision of its own, and a plain no-op is what those
            // four already do in that position.
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
    /// failing clause's).
    ///
    /// **This alone is not enough for a `WHEN`/`WhenCase` whose own
    /// *condition* raises**, and a second, real defect this task's own
    /// review found: `Select`'s own arm evaluates a `When`/`WhenCase`'s
    /// condition directly, as data, and never opens a `step_in_temps_frame`
    /// for the `When`/`WhenCase` instruction itself (that instruction's own
    /// `step` arm is a pure no-op, per the `When`/`WhenCase` arm's own doc
    /// comment) -- so a raise there has no inner wrapper call to be the
    /// "innermost" one, and would still be attributed to the `SELECT`.
    /// Measured: `select` / `when 'x' then nop` / `end` reported the
    /// `SELECT`'s own line and clause, not the `WHEN`'s. `Select`'s own arm
    /// calls `record_failure_site` directly, past `code.body.instructions[
    /// when_index]`, on exactly that path -- see its own call sites there.
    ///
    /// `self.failure_site.is_none()` is the guard, in both callers, that
    /// makes the *first* resolution win, which is always the most specific
    /// one available: the deepest `step_in_temps_frame` call, or `Select`'s
    /// own direct call for a `When`/`WhenCase` condition, always runs before
    /// any enclosing propagation reaches an outer wrapper. `source: None`
    /// (`run_fragment`'s own call into `run_bounded`) skips this entirely,
    /// preserving that function's existing, deliberate choice not to
    /// resolve a site for an instruction inside an `INTERPRET` fragment.
    fn step_in_temps_frame(
        &mut self,
        code: &Code<'_>,
        index: usize,
        instruction: &Instruction,
        source: Option<&ProgramSource>,
    ) -> Result<Flow, Failure> {
        // `TRACE`'s own `*-*` clause echo (D17), and the single insertion
        // point for it -- exactly the analogue of `eval`'s own split from
        // `eval_node`, since this is the one place `run_bounded`'s loop
        // (this function's only non-test caller) visits every flat
        // instruction position it steps, markers (`Then`/`Else`/
        // `Otherwise`/`When`/`WhenCase`/`Label`) included, matching the
        // oracle's own `RexxInstruction::traceInstruction`, which every one
        // of those calls too from its own `execute`.
        //
        // **Not the whole story for a `DO`/`LOOP`.** The oracle's own `DO`
        // instruction is re-executed once per iteration, so its own clause
        // (and `END`'s) echoes again on every pass -- this call site fires
        // exactly once, when the `DO`/`LOOP` instruction is first stepped,
        // because `run_loop`/`run_repeating` resolve every iteration inside
        // *this* one `step` call rather than returning between passes
        // (`Flow::Leave`'s own doc comment has the reason: a `Goto`-shaped
        // re-entry risks the absorption trap that design avoids). The
        // per-iteration re-echo is `run_repeating`'s own, separate call
        // into `trace_clause` -- see its doc comment.
        // `current_value_indent` (`lib.rs`'s own doc comment on the field)
        // is set here **unconditionally**, not only when `trace_mode.all`
        // -- `INTERMEDIATES` implies `all` (`TraceMode`'s own doc comment),
        // never the reverse, so anything that reads this field is already
        // gated by its own `intermediates` check; setting it plainly is
        // cheaper than a second `if` that would just repeat that gate.
        let indent = static_indent(&code.body.instructions, index);
        self.current_value_indent = indent;
        if self.trace_mode.all
            && let Some((line, text)) = clause_site(source, instruction)
        {
            self.trace_clause(line, indent, &text);
        }
        let frame = self.roots.push_frame();
        let flow = self.step(code, index, instruction, source);
        self.roots.pop_frame(frame);
        if flow.is_err() {
            self.record_failure_site(code, index, source, instruction);
        }
        flow
    }

    /// Resolves `instruction`'s own clause (and its statically-derived
    /// indent, `static_indent`) into `self.failure_site`, first call wins
    /// (`self.failure_site.is_none()`), when `source` is `Some`.
    ///
    /// The shared half of `step_in_temps_frame`'s own resolution (its doc
    /// comment has the full argument for why the *first* caller to run this
    /// is always the right one) -- factored out so `Select`'s own arm can
    /// call it directly for a `When`/`WhenCase` whose *condition* raises,
    /// which never goes through `step_in_temps_frame` at all since that
    /// instruction's own `step` arm never runs for a decision of its own.
    ///
    /// `index` is `instruction`'s own position in `code.body.instructions`,
    /// needed (beyond what `step_in_temps_frame` already required it for)
    /// so `static_indent` has something to walk the flat instruction list
    /// up to.
    fn record_failure_site(
        &mut self,
        code: &Code<'_>,
        index: usize,
        source: Option<&ProgramSource>,
        instruction: &Instruction,
    ) {
        self.record_failure_at(
            source,
            instruction,
            static_indent(&code.body.instructions, index),
        );
    }

    /// Assigns `blame`'s own clause to `self.failure_site` at exactly
    /// `indent` spaces, first call wins, when `source` is `Some`.
    ///
    /// The common tail `record_failure_site` itself uses (computing
    /// `indent` from `blame`'s own position first) and that `Do`'s own
    /// `WHILE`/`UNTIL` checks call directly with a *different* indent --
    /// neither corresponds to a flat instruction position `static_indent`
    /// resolves correctly on its own (`static_indent`'s own doc comment has
    /// the full argument), so `Do`'s own arm computes `WHILE`'s/`UNTIL`'s
    /// indent itself and hands it straight to this function rather than
    /// asking `record_failure_site` to guess between two different, both
    /// correct, answers for the same instruction index.
    fn record_failure_at(
        &mut self,
        source: Option<&ProgramSource>,
        blame: &Instruction,
        indent: usize,
    ) {
        if self.failure_site.is_some() {
            return;
        }
        if let Some((line, text)) = clause_site(source, blame) {
            self.failure_site = Some(FailureSite { line, text, indent });
        }
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
    fn leave_origin(
        &self,
        code: &Code<'_>,
        index: usize,
        source: Option<&ProgramSource>,
        instruction: &Instruction,
    ) -> LeaveOrigin {
        LeaveOrigin {
            site: clause_site(source, instruction),
            indent: static_indent(&code.body.instructions, index),
        }
    }

    /// Assigns `origin`'s own captured site to `self.failure_site`, first
    /// call wins, at `origin`'s own captured indent.
    ///
    /// **Corrected after review.** This crate's first cut of the
    /// LEAVE/ITERATE indent family hardcoded the exhausted-search family
    /// (28.1-28.4) to zero and reported `origin.indent` unmodified for
    /// 28.5, on the theory that those were the only two shapes the
    /// oracle's own indent could take. A reviewer's fourteen-point probe
    /// falsified that in seven cases (re-measured independently against
    /// the oracle before changing anything -- see the report): the actual
    /// rule is that `origin.indent` is the search's own *residual*, updated
    /// every time a frame the search examines is popped rather than
    /// matched, and this function's only job now is to report whatever
    /// `origin.indent` already holds by the time either family reaches its
    /// own resolution point -- there is exactly one caller-facing function
    /// for both families, because the difference between them was never in
    /// how the site gets recorded, only in how far the search walked before
    /// giving up. See `Do`'s and `Select`'s own arms (`do_body_outcome`,
    /// `leave_select`) for where the residual is actually updated, and
    /// `LeaveOrigin`'s own doc comment for the rule in full.
    fn record_leave_failure(&mut self, origin: &LeaveOrigin) {
        if self.failure_site.is_some() {
            return;
        }
        if let Some((line, text)) = &origin.site {
            self.failure_site = Some(FailureSite {
                line: *line,
                text: text.clone(),
                indent: origin.indent,
            });
        }
    }

    /// Turns the `Flow` a `SELECT`'s own matched `WHEN` or `OTHERWISE` body
    /// produced into this `SELECT`'s own answer.
    ///
    /// `Flow::Next` becomes `Goto(resume)`, exactly the shape every branch
    /// gave before Task 11. A `LEAVE`/`ITERATE` naming this `SELECT`'s own
    /// `label` (`Some` only for `SELECT LABEL name` -- an ordinary clause
    /// label in front of a `SELECT` is a separate `Label` instruction and
    /// never reaches `label` at all, measured 28.3/28.4 exactly as for an
    /// unlabelled loop) is consumed here: a matching `LEAVE` resumes past
    /// the whole `SELECT`; a matching `ITERATE` is **28.5**, because
    /// `SELECT` is never a repetitive loop (`RexxInstructionSelect::isLoop`
    /// answers `false` unconditionally, read directly in the report) --
    /// measured, `ITERATE` never accepts a non-loop target even when the
    /// name matches. Everything else -- an unnamed `LEAVE`/`ITERATE` (a
    /// `SELECT` is never a bare target either, same reason), or one naming
    /// something else -- is **not matched, but not untouched either**: a
    /// `SELECT` always owns a search frame (unconditionally, labelled or
    /// not -- unlike `Do`'s own unlabelled-`Simple` exception), so
    /// forwarding it outward resets `origin.indent` to this `SELECT`'s own
    /// `static_indent` first (`LeaveOrigin`'s own doc comment has the full
    /// rule and the oracle transcripts that pin it). `Exit` and a `Goto`
    /// that escaped `run_bounded`'s own range pass through with nothing
    /// touched, same as always.
    fn leave_select(
        &mut self,
        code: &Code<'_>,
        index: usize,
        label: Option<SymbolId>,
        resume: usize,
        flow: Flow,
    ) -> Result<Flow, Failure> {
        match flow {
            Flow::Next => Ok(Flow::Goto(resume)),
            Flow::Leave(Some(name), _) if label == Some(name) => Ok(Flow::Goto(resume)),
            Flow::Iterate(Some(name), origin) if label == Some(name) => {
                self.record_leave_failure(&origin);
                Err(raised_iterate_wrong_kind(code.symbols.name(name).as_bytes()).into())
            }
            // Not consumed: this SELECT is being "popped" by the search,
            // so its own indent becomes the new residual before the flow
            // continues outward.
            Flow::Leave(name, origin) => Ok(Flow::Leave(
                name,
                self.pop_search_frame(code, index, origin),
            )),
            Flow::Iterate(name, origin) => Ok(Flow::Iterate(
                name,
                self.pop_search_frame(code, index, origin),
            )),
            other => Ok(other),
        }
    }

    /// Resets `origin.indent` to `index`'s own `static_indent`, for a
    /// `SELECT`/`DO`/`LOOP` that owns a search frame and is being forwarded
    /// past (not matched) -- the update `LeaveOrigin`'s own doc comment
    /// describes as "restoring the indent to the value saved when that
    /// frame was pushed." Shared by `leave_select` (always calls it, since
    /// a `SELECT` always owns a frame) and `do_body_outcome` (calls it only
    /// when the `Do`/`Loop` in question owns one, i.e. skips an unlabelled
    /// `Simple` block).
    fn pop_search_frame(&self, code: &Code<'_>, index: usize, origin: LeaveOrigin) -> LeaveOrigin {
        LeaveOrigin {
            site: origin.site,
            indent: static_indent(&code.body.instructions, index),
        }
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
    /// is the only way this returns `Ok(Flow::Next)`. Every other exit
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

    /// `DO`/`LOOP`, every kind. Resolves the whole construct -- header
    /// validation, every iteration, `LEAVE`/`ITERATE`, `WHILE`/`UNTIL` --
    /// inside this one call, exactly the discipline `If`/`Select` already
    /// hold: see `Flow::Leave`'s own doc comment for why a `Do` must never
    /// return to its caller mid-loop.
    ///
    /// **`COUNTER` and `DO WITH` both take the loud path, checked first and
    /// unconditionally.** The brief this task started from names both
    /// explicitly and asks for a decision, not a silent fallthrough:
    /// `COUNTER`'s own running-count bookkeeping is Phase-5-shaped extra
    /// state that no other of `DO`/`LOOP`'s 21 other forms needs, and
    /// `DO WITH` sends `SUPPLIER` a message, which nothing in 4a answers
    /// (no message dispatch at all yet). Checked ahead of any header
    /// evaluation, so `do counter c with index i over x` -- both keywords
    /// at once -- fails loudly without evaluating `x` either.
    fn run_loop(
        &mut self,
        code: &Code<'_>,
        index: usize,
        instruction: &Instruction,
        body: &Loop,
        source: Option<&ProgramSource>,
    ) -> Result<Flow, Failure> {
        if body.counter.is_some() || matches!(body.kind, LoopKind::With { .. }) {
            return Err(Loud::instruction(&instruction.kind).into());
        }

        let body_start = index + 1;
        let end_index = body
            .end
            .expect("an unclosed DO/LOOP is error 14.1/14.5, so a body that parsed has this set");
        let resume = end_index + 1;
        let label = body.label;

        match &body.kind {
            // A block, not a loop: exactly one pass, and `WHILE`/`UNTIL`
            // can never be present (`create_loop`'s own parser only reaches
            // `LoopKind::Simple` through the bare, at-end-of-clause `DO`
            // arm, before any conditional is even looked for). Still
            // leavable by an explicit `DO LABEL`, never by a bare `LEAVE`
            // (`do_body_outcome`'s own `is_loop: false`) -- measured, a
            // labelled simple block is leavable but an unlabelled one is
            // 28.1 on a bare `LEAVE` reaching it.
            LoopKind::Simple => {
                let flow = self.run_bounded(code, body_start, end_index, source)?;
                match self.do_body_outcome(code, index, label, false, resume, flow)? {
                    Some(escape) => Ok(escape),
                    // Falls through to `END`, which `run_bounded`'s own
                    // `[body_start, end_index)` range never visits (the
                    // `Goto(resume)` below jumps straight past it) --
                    // unlike a repeating loop, a `Simple` block never runs
                    // this arm again, so one explicit echo here is the
                    // whole story, not a per-pass one the way
                    // `run_repeating`'s own is. Measured, this task's
                    // report: `if 1 = 1 then do / say 'x' / end` traces
                    // `end` on its own line even though the block never
                    // repeats.
                    None => {
                        if self.trace_mode.all
                            && let Some((line, text)) =
                                clause_site(source, &code.body.instructions[end_index])
                        {
                            self.trace_clause(
                                line,
                                static_indent(&code.body.instructions, index),
                                &text,
                            );
                        }
                        Ok(Flow::Goto(resume))
                    }
                }
            }
            LoopKind::Forever => self.run_repeating(
                code,
                index,
                instruction,
                body_start,
                end_index,
                resume,
                label,
                body.conditional.as_ref(),
                source,
                LoopState::Forever,
            ),
            LoopKind::Count(count_expr) => {
                let remaining = match count_expr {
                    Some(expr) => {
                        let value = self.eval(code, expr)?;
                        self.roots.push_temp(value);
                        let text = self.to_text(value).to_vec();
                        // `>K>   "FOR" => "2"`, once, at the `DO`'s own
                        // level -- measured, this task's own report (F1,
                        // found by review): the oracle traces a bare
                        // repeat count under the `FOR` tag, the same as
                        // an explicit `DO ... FOR n`'s own. Fires before
                        // the `whole_nonneg` check below, matching every
                        // other `>K>` site in this function (the value is
                        // traced as evaluated, not as validated).
                        self.trace_keyword(
                            static_indent(&code.body.instructions, index),
                            "FOR",
                            &text,
                        );
                        self.whole_nonneg(value)
                            .ok_or_else(|| raised_repetition_count_not_whole(&text))?
                    }
                    // Defensive, not measured: `count_loop`'s own parser
                    // (`instruction.rs`) always calls `opt_expr`, which can
                    // answer `None`, but nothing in this crate's own tests
                    // reaches `DO` with truly nothing after it and no
                    // recognised keyword either -- `create_loop`'s own
                    // `at_end()` check catches a bare `DO` first and builds
                    // `LoopKind::Simple` instead. A single pass, matching
                    // `Simple`'s own behaviour, is the least surprising
                    // answer if this is ever reached.
                    None => 1,
                };
                self.run_repeating(
                    code,
                    index,
                    instruction,
                    body_start,
                    end_index,
                    resume,
                    label,
                    body.conditional.as_ref(),
                    source,
                    LoopState::Count { remaining },
                )
            }
            LoopKind::Controlled(ctrl) => {
                let indent = static_indent(&code.body.instructions, index);
                let state = self.setup_controlled(code, ctrl, indent)?;
                self.run_repeating(
                    code,
                    index,
                    instruction,
                    body_start,
                    end_index,
                    resume,
                    label,
                    body.conditional.as_ref(),
                    source,
                    state,
                )
            }
            LoopKind::Over {
                control,
                target,
                for_count,
            } => {
                // Deviation 1 (`phase-4-exclusions.txt`): a stem target's
                // own tail order does not reproduce the oracle's (a
                // balanced tree against our hash map), and no corpus
                // program may contain one. Detected from `target`'s own
                // *syntax*, never by evaluating it: `over a.` parses `a.`
                // through the ordinary expression grammar, which recognises
                // a bare trailing-dot token as `ExprKind::Stem` the same
                // way a plain stem read anywhere else does, so a target
                // that is a stem never needs evaluating to know it is out
                // of scope.
                //
                // **Corrected after review**: an earlier version of this
                // comment said a stem reached indirectly (`over (a.)`) is
                // "not detected here". Measured (`do_over_a_parenthesised_
                // stem_target_is_also_caught`), it *is*: a single
                // parenthesised sub-expression collapses to that
                // sub-expression's own `ExprKind` rather than being wrapped
                // in `ExprKind::List`, so `(a.)` is already
                // `ExprKind::Stem` by the time the `matches!` below runs,
                // with nothing extra needed to catch it. What genuinely
                // escapes this check is a stem reached through something
                // that does not collapse this way -- a function call
                // returning one, for instance -- and that gap is real, not
                // a mistaken claim: no test may write one either way, so
                // nothing observable depends on catching it, but the
                // previous wording overstated the gap to include a case
                // this check already closes.
                if matches!(target.kind, ExprKind::Stem(_)) {
                    return Err(Loud::instruction(&instruction.kind).into());
                }
                let value = self.eval(code, target)?;
                self.roots.push_temp(value);
                // `>K>   "OVER" => "abc"`, once, at the `DO`'s own level --
                // measured, this task's report: fires on the first pass
                // only, exactly like `TO`/`BY`/`FOR`, because `target` is
                // evaluated once at loop entry here too.
                let over_indent = static_indent(&code.body.instructions, index);
                let over_text = self.to_text(value).to_vec();
                self.trace_keyword(over_indent, "OVER", &over_text);
                let remaining = match for_count {
                    Some(expr) => {
                        let count_value = self.eval(code, expr)?;
                        self.roots.push_temp(count_value);
                        let text = self.to_text(count_value).to_vec();
                        Some(
                            self.whole_nonneg(count_value)
                                .ok_or_else(|| raised_for_count_not_whole(&text))?,
                        )
                    }
                    None => None,
                };
                self.run_repeating(
                    code,
                    index,
                    instruction,
                    body_start,
                    end_index,
                    resume,
                    label,
                    body.conditional.as_ref(),
                    source,
                    LoopState::OverOnce {
                        control: *control,
                        value,
                        done: false,
                        remaining,
                    },
                )
            }
            LoopKind::With { .. } => unreachable!("DO WITH takes the loud path above"),
        }
    }

    /// The shared driver for every repeating `LoopKind` (everything but
    /// `Simple`, which never repeats and runs through `run_loop`'s own
    /// arm directly): advance-test-run-test-advance, in the order the
    /// oracle is measured to use it in (`report`'s own transcripts) --
    /// `WHILE` tested before the body, `UNTIL` after, and a `LEAVE`/
    /// `ITERATE` handled identically to falling off the bottom of the body
    /// normally, because that is what the oracle's own `ITERATE` does
    /// (measured: `do until n = 1 / n = n + 1 / if n = 1 then iterate / ...`
    /// terminates immediately rather than skipping straight to the next
    /// pass's top, because `ITERATE` jumps to the loop's own
    /// bottom-of-iteration bookkeeping, which for an `UNTIL` loop includes
    /// testing `UNTIL` right there -- see the report for the full
    /// transcript).
    ///
    /// `do_index`/`do_instruction` are the `DO`/`LOOP` instruction's own
    /// position and node, needed only for `WHILE`'s own attribution
    /// (`end_index` is `END`'s own, for `UNTIL`'s) -- neither corresponds
    /// to a flat position `static_indent` resolves on its own, so both
    /// indents are computed here rather than asked of it (`record_failure_at`'s
    /// own doc comment).
    #[allow(
        clippy::too_many_arguments,
        reason = "every parameter is load-bearing state one repeating DO/LOOP needs; splitting it into a struct is Task 13's to consider if it too needs this shape"
    )]
    fn run_repeating(
        &mut self,
        code: &Code<'_>,
        do_index: usize,
        do_instruction: &Instruction,
        body_start: usize,
        end_index: usize,
        resume: usize,
        label: Option<SymbolId>,
        conditional: Option<&LoopConditional>,
        source: Option<&ProgramSource>,
        mut state: LoopState,
    ) -> Result<Flow, Failure> {
        // The loop's own two spaces of indent, added once here rather than
        // per-check: `static_indent(&code.body.instructions, do_index)` is
        // what a control-setup failure at `do_index` itself already
        // reports (measured: `do i = 1 to 3 for 1/0` is unindented at top
        // level), and `WHILE`/`UNTIL` both report two spaces *more* than
        // that (measured: `do while 1/0` at top level is indented two).
        let do_indent = static_indent(&code.body.instructions, do_index);
        let loop_indent = do_indent + 2;
        // `TRACE`'s own per-iteration re-echo (D17, this task's report,
        // "Step 6"): the oracle's `DO`/`LOOP` instruction is re-executed
        // once per pass (`DoBlock::checkControl`, read directly), so its
        // own clause -- and `END`'s -- echo again on every pass, unlike
        // every other construct in this crate, which resolves its whole
        // repetition inside one `step` call and so is stepped, and echoed,
        // exactly once (`step_in_temps_frame`'s own doc comment). `false`
        // on entry because the *first* pass's echo already happened there,
        // before `run_loop` ever called into this function.
        let mut first_pass = true;

        loop {
            if !first_pass
                && self.trace_mode.all
                && let Some((line, text)) = clause_site(source, do_instruction)
            {
                self.trace_clause(line, do_indent, &text);
            }
            first_pass = false;

            if !self.loop_advance(code, &mut state)? {
                return Ok(Flow::Goto(resume));
            }
            if let Some(cond) = conditional
                && !cond.until
            {
                // Overrides `step_in_temps_frame`'s own setting of
                // `current_value_indent` (to `do_indent`, from stepping the
                // `DO`/`LOOP` instruction itself) -- `WHILE`'s own
                // condition is evaluated here, inside that same `step`
                // call, never through a `step_in_temps_frame` of its own.
                self.current_value_indent = loop_indent;
                match self.eval_condition(
                    code,
                    &cond.condition,
                    ConditionTrace::Keyword(loop_indent, "WHILE"),
                    raised_while_not_logical,
                ) {
                    Ok(true) => {}
                    Ok(false) => return Ok(Flow::Goto(resume)),
                    Err(failure) => {
                        self.record_failure_at(source, do_instruction, loop_indent);
                        return Err(failure);
                    }
                }
            }

            let flow = self.run_bounded(code, body_start, end_index, source)?;
            if let Some(escape) = self.do_body_outcome(code, do_index, label, true, resume, flow)? {
                return Ok(escape);
            }

            // Reached only when the pass is about to continue (fell
            // through, or a matched `ITERATE` -- `do_body_outcome`
            // returning `Ok(None)` is exactly that set, per its own doc
            // comment). **Not** reached on a matched `LEAVE`, which
            // returns above instead -- measured, this task's report
            // (`DO FOREVER` with a `LEAVE` on the second pass): `END`
            // never echoes for that final pass, only for a pass that
            // genuinely falls through to it.
            let end_instruction = &code.body.instructions[end_index];
            if self.trace_mode.all
                && let Some((line, text)) = clause_site(source, end_instruction)
            {
                self.trace_clause(line, do_indent, &text);
            }

            if let Some(cond) = conditional
                && cond.until
            {
                // Same override as `WHILE`'s own, above -- the re-echoed
                // `END` clause just before this point left
                // `current_value_indent` untouched (its own `trace_clause`
                // call does not set it), so without this `UNTIL`'s
                // intermediates would otherwise still read `do_indent`.
                self.current_value_indent = loop_indent;
                match self.eval_condition(
                    code,
                    &cond.condition,
                    ConditionTrace::Keyword(loop_indent, "UNTIL"),
                    raised_until_not_logical,
                ) {
                    Ok(true) => return Ok(Flow::Goto(resume)),
                    Ok(false) => {}
                    Err(failure) => {
                        self.record_failure_at(source, end_instruction, loop_indent);
                        return Err(failure);
                    }
                }
            }

            self.loop_step(&mut state)?;
        }
    }

    /// What one repeating `Do`/`Loop`'s own body just produced, translated
    /// into what `run_repeating`/`run_loop`'s own `Simple` arm does next.
    ///
    /// `Ok(None)`: the body ran to completion, or an `ITERATE` naming this
    /// construct was consumed -- proceed to whatever bottom-of-iteration
    /// test/advance comes next (`run_repeating`'s own doc comment has the
    /// argument for why the two are the identical next step). `Ok(Some(f))`:
    /// stop, and `f` is this construct's own final answer -- either
    /// `Goto(resume)` (a consumed `LEAVE`) or an unconsumed `Flow` to
    /// propagate outward unchanged (`Exit`, a `Goto` that escaped
    /// `run_bounded`'s own range, or a `LEAVE`/`ITERATE` naming something
    /// else). `Err`: a named `ITERATE` matched `label`, but `is_loop` is
    /// `false` -- 28.5, `ITERATE` never accepts a labelled block, only a
    /// loop (measured).
    ///
    /// `is_loop` is `false` only for `LoopKind::Simple` (`run_loop`'s own
    /// `Simple` arm passes it); every `LoopState` variant `run_repeating`
    /// drives is a real, repetitive loop and passes `true`.
    ///
    /// **Whether this construct "owns a search frame" (`LeaveOrigin`'s own
    /// doc comment has the rule and the oracle transcripts) is `is_loop ||
    /// label.is_some()`, not `is_loop` alone.** A labelled `Simple` block
    /// does not repeat, but it is still leavable by name and still resets
    /// the search's own residual indent when a `LEAVE`/`ITERATE` naming
    /// something else is forwarded past it -- only an *unlabelled* `Simple`
    /// block is fully transparent, touching nothing as a `LEAVE`/`ITERATE`
    /// passes through.
    ///
    /// `do_index` is this `Do`/`Loop` instruction's own position, needed
    /// only to compute that reset (`pop_search_frame`); it is *not* used
    /// for clause attribution here, since nothing in this function raises
    /// against this instruction's own clause.
    fn do_body_outcome(
        &mut self,
        code: &Code<'_>,
        do_index: usize,
        label: Option<SymbolId>,
        is_loop: bool,
        resume: usize,
        flow: Flow,
    ) -> Result<Option<Flow>, Failure> {
        let owns_frame = is_loop || label.is_some();
        match flow {
            Flow::Next => Ok(None),
            Flow::Leave(name, origin) => {
                let matched = match name {
                    None => is_loop,
                    Some(n) => label == Some(n),
                };
                if matched {
                    Ok(Some(Flow::Goto(resume)))
                } else if owns_frame {
                    Ok(Some(Flow::Leave(
                        name,
                        self.pop_search_frame(code, do_index, origin),
                    )))
                } else {
                    Ok(Some(Flow::Leave(name, origin)))
                }
            }
            Flow::Iterate(name, origin) => {
                let matched = match name {
                    None => is_loop,
                    Some(n) => label == Some(n),
                };
                if !matched {
                    return Ok(Some(Flow::Iterate(
                        name,
                        if owns_frame {
                            self.pop_search_frame(code, do_index, origin)
                        } else {
                            origin
                        },
                    )));
                }
                if !is_loop {
                    let name = name.expect(
                        "is_loop is false and matched is true only through the named branch above",
                    );
                    self.record_leave_failure(&origin);
                    return Err(
                        raised_iterate_wrong_kind(code.symbols.name(name).as_bytes()).into(),
                    );
                }
                Ok(None)
            }
            other => Ok(Some(other)),
        }
    }

    /// Decides whether one more candidate iteration of `state` should run,
    /// consuming whatever budget (`FOR`, a bare count) applies and binding
    /// a control variable **before** the decision is answered, not after --
    /// measured, `do i = 5 to 3 / say never / end / say i` prints `5`: the
    /// control variable is bound to its own value even for a loop that ends
    /// up running zero iterations.
    fn loop_advance(&mut self, code: &Code<'_>, state: &mut LoopState) -> Result<bool, Failure> {
        match state {
            LoopState::Forever => Ok(true),
            LoopState::Count { remaining } => {
                if *remaining == 0 {
                    return Ok(false);
                }
                *remaining -= 1;
                Ok(true)
            }
            LoopState::OverOnce {
                control,
                value,
                done,
                remaining,
            } => {
                if *done {
                    return Ok(false);
                }
                if let Some(r) = remaining {
                    if *r == 0 {
                        *done = true;
                        return Ok(false);
                    }
                    *r -= 1;
                }
                *done = true;
                self.bind_control(code, *control, *value);
                Ok(true)
            }
            LoopState::Controlled {
                control,
                current,
                to,
                by,
                for_remaining,
            } => {
                // **KNOWN GAP, disclosed rather than fixed under this
                // task's own time budget.** `DoBlock::checkControl`
                // (`DoBlock.cpp:182`, read directly) traces two `>>>`
                // lines on every pass after the first: the control
                // variable's own pre-increment value, then `value + by`,
                // both via plain `traceResult` -- measured, this task's
                // report, `do i = 1 to 2`'s own second pass shows `>>>
                // "1"` then `>>>   "2"` with nothing else around them.
                // This function does not reproduce that pair: `current`
                // here already holds `loop_step`'s own pre-computed
                // `current + by` from the *previous* pass (this arm only
                // binds and tests it), so the pre-increment value is gone
                // by the time this runs, and recovering it would mean
                // moving the increment out of `loop_step` and into this
                // arm -- restructuring already-reviewed, working control
                // flow (`the_corrected_28x_indent_rule_matches_all_
                // fourteen_probed_shapes` and the whole `Flow::Leave`/
                // `run_bounded` absorption discipline both sit downstream
                // of this exact split) for a formatting concern, under
                // less time than that would need to be done safely. Every
                // `>K>` (`TO`/`BY`/`FOR`, fires once, matches the oracle
                // exactly because these headers are evaluated once here
                // too) and every `WHILE`/`UNTIL` `>K>` (fires every pass,
                // matches because the condition is genuinely re-evaluated
                // every pass here too) is unaffected -- this gap is
                // narrowly the two-line pair for a `Controlled`
                // (`TO`-style) loop's own re-tested pass, nothing else.
                // This task's report names it explicitly rather than
                // choosing a witness that cannot see it.
                //
                // **Bound before the decision, not after** -- measured
                // against the oracle, `do i = 5 to 3 / say never / end /
                // say i` prints `5`: the control variable takes its own
                // header value even for a loop that goes on to run zero
                // iterations, for *either* reason a candidate iteration can
                // fail below (an exhausted `FOR` budget or the `TO` bound).
                let digits = self.activation().settings.digits();
                let fuzz = self.activation().settings.fuzz();
                let form = self.activation().settings.form();
                let value =
                    self.number(current.clone(), crate::eval::saturate_digits(digits), form);
                self.bind_control(code, *control, value);

                if let Some(r) = for_remaining
                    && *r == 0
                {
                    return Ok(false);
                }
                if let Some(to) = to {
                    let by_negative =
                        numeric_less(by, &Number::zero(), digits, fuzz).map_err(Raised::from)?;
                    let within = if by_negative {
                        !numeric_less(current, to, digits, fuzz).map_err(Raised::from)?
                    } else {
                        !numeric_less(to, current, digits, fuzz).map_err(Raised::from)?
                    };
                    if !within {
                        return Ok(false);
                    }
                }
                if let Some(r) = for_remaining {
                    *r -= 1;
                }
                Ok(true)
            }
        }
    }

    /// Advances `state` for the *next* candidate iteration, once this one
    /// has fully finished (fallen through, or an `ITERATE` was consumed) --
    /// only `Controlled` has anything to do here: `current = current + by`,
    /// under the settings active *now*, matching every other arithmetic
    /// operation in this crate (`eval_arithmetic`'s own doc comment).
    fn loop_step(&mut self, state: &mut LoopState) -> Result<(), Failure> {
        if let LoopState::Controlled { current, by, .. } = state {
            let digits = self.activation().settings.digits();
            *current = current.add(by, digits).map_err(Raised::from)?;
        }
        Ok(())
    }

    /// Evaluates a `Controlled` loop's header: `initial` first, then
    /// whichever of `TO`/`BY`/`FOR` were written, in the order they were
    /// written (`ctrl.order`, recorded because the expressions can have
    /// side effects) -- never a fixed `TO`-then-`BY`-then-`FOR` order.
    ///
    /// `initial`/`to`/`by` need only be *numeric* (41.1 if not, via
    /// `arith_operand` -- the same check ordinary arithmetic already
    /// makes), never *whole*: measured, `do i = 1.5 to 3` is legal and
    /// steps by fractional values. `by` defaults to `1` when absent.
    /// `for_count` is the one exception, checked against `whole_nonneg`
    /// (26.3) exactly like a bare `DO`'s own repeat count is (26.2).
    /// `indent`: the `DO`/`LOOP` instruction's own `static_indent`, for
    /// `>K>`'s own `TO`/`BY`/`FOR` lines -- **not** `loop_indent`
    /// (`+2`, `WHILE`/`UNTIL`'s own level): measured, `>K>   "TO" => "2"`
    /// sits at the same indent as `do i = 1 to 2` itself, because these
    /// header expressions are evaluated once at loop entry, before the
    /// body's own frame exists at all (`control_setup_expressions_are_
    /// unindented_unlike_the_loop_body_they_precede`, this file's own test
    /// from Task 11, makes the identical point about a *raise* at this
    /// same point).
    fn setup_controlled(
        &mut self,
        code: &Code<'_>,
        ctrl: &Controlled,
        indent: usize,
    ) -> Result<LoopState, Failure> {
        let initial_value = self.eval(code, &ctrl.initial)?;
        self.roots.push_temp(initial_value);
        let current = self.arith_operand(initial_value)?;

        let mut to = None;
        let mut by = None;
        let mut for_remaining = None;
        for entry in &ctrl.order {
            match entry {
                ControlExpr::To => {
                    let expr = ctrl
                        .to
                        .as_ref()
                        .expect("ctrl.order names To only when ctrl.to is Some");
                    let value = self.eval(code, expr)?;
                    self.roots.push_temp(value);
                    let text = self.to_text(value).to_vec();
                    self.trace_keyword(indent, "TO", &text);
                    to = Some(self.arith_operand(value)?);
                }
                ControlExpr::By => {
                    let expr = ctrl
                        .by
                        .as_ref()
                        .expect("ctrl.order names By only when ctrl.by is Some");
                    let value = self.eval(code, expr)?;
                    self.roots.push_temp(value);
                    let text = self.to_text(value).to_vec();
                    self.trace_keyword(indent, "BY", &text);
                    by = Some(self.arith_operand(value)?);
                }
                ControlExpr::For => {
                    let expr = ctrl
                        .for_count
                        .as_ref()
                        .expect("ctrl.order names For only when ctrl.for_count is Some");
                    let value = self.eval(code, expr)?;
                    self.roots.push_temp(value);
                    let text = self.to_text(value).to_vec();
                    self.trace_keyword(indent, "FOR", &text);
                    for_remaining = Some(
                        self.whole_nonneg(value)
                            .ok_or_else(|| raised_for_count_not_whole(&text))?,
                    );
                }
            }
        }
        let by = match by {
            Some(by) => by,
            None => Number::parse("1").expect("the literal 1 always parses"),
        };
        Ok(LoopState::Controlled {
            control: ctrl.control,
            current,
            to,
            by,
            for_remaining,
        })
    }

    /// Writes `value` into `control`'s own slot -- the same read-the-name,
    /// resolve-a-slot, write path `Assignment`'s `Variable` target already
    /// uses (`step`'s own `Assignment` arm), reused rather than duplicated.
    fn bind_control(&mut self, code: &Code<'_>, control: SymbolId, value: ObjRef) {
        let name = code.symbols.name(control).as_bytes();
        let slot = self.slot_of(name);
        let frame = self.activation().frame;
        self.roots.set_slot(frame, slot, value);
    }

    /// Validates `value` as "zero or a positive whole number" -- the rule a
    /// bare `DO`'s own repeat count and a `FOR` expression share (26.2/26.3
    /// respectively; the caller supplies which raiser applies, since that
    /// is the only way the two differ), and answers it as a `u64`, or
    /// `None` if it fails either check. `rexx_num::ARGUMENT_DIGITS` is
    /// `exit_code_for`'s own choice of width for exactly this
    /// current-`DIGITS`-independent kind of conversion (`lib.rs`'s own doc
    /// comment on it), reused here rather than invented: a loop bound is no
    /// more digits-limited than `EXIT`'s own result is.
    fn whole_nonneg(&mut self, value: ObjRef) -> Option<u64> {
        let number = self.to_number(value).ok()?;
        let whole = number.whole_value(rexx_num::ARGUMENT_DIGITS)?;
        u64::try_from(whole).ok()
    }

    /// Evaluates `condition` and answers whether it holds, for `IF`/`WHEN`.
    ///
    /// **A comma list checks itself, but a single expression does not, and
    /// this is the one place that gap gets closed.** `ExprKind::Logical` (a
    /// comma list) is evaluated through `eval`'s own dispatch to
    /// `eval_logical_list` exactly like any other expression, which already
    /// validates every element is exactly `0`/`1` and raises 34.6 on the
    /// first that is not -- re-checking its result here would misreport
    /// that failure as 34.1/34.2. A single, non-list expression never
    /// passes through `eval_logical_list` at all (there is no list to
    /// iterate), so nothing has checked it yet. `raise` is the
    /// keyword-specific raiser for exactly that case (34.1 `IF`, 34.2
    /// `WHEN`) -- measured across both, `if 'x', 1 then` is 34.6 (a list,
    /// regardless of which element failed) while `if 'x' then` is 34.1 (not
    /// a list at all).
    fn eval_condition(
        &mut self,
        code: &Code<'_>,
        condition: &Expr,
        trace: ConditionTrace<'_>,
        raise: fn(&[u8]) -> Raised,
    ) -> Result<bool, Failure> {
        let value = self.eval(code, condition)?;
        self.roots.push_temp(value);
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
    /// asked for and approved for `logical_value`. This way needs none.
    /// `indent` traces two `>>>` lines per value tested, up to and including
    /// whichever one matches (`WhenCaseInstruction.cpp:154`/`158`:
    /// `traceResult(compareValue)` then `traceResult(result)`, "result"
    /// being the comparison's own `0`/`1` outcome, not a second copy of the
    /// value) -- measured, `select case 1 + 1 / when 2 then ...`:
    /// `>>>   "2"` (the one `values` entry evaluated) then `>>>   "1"`
    /// (matched). Stops at the first match, mirroring the early `return
    /// Ok(true)` below -- an oracle transcript with more than one `values`
    /// entry and no match on the first would need its own probe to confirm
    /// every untested entry gets the same pair, which this task did not run.
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

    /// If `target` names an `Else` instruction, its own `then_exit`
    /// (defaulting to the end of the body when `None`, "the end of this
    /// body" per `ast.rs`'s own doc comment). Otherwise `target` unchanged.
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
    /// `TRACE`'s four forms (D17). `Trace::Default` (bare `TRACE`) and a
    /// `Trace::Setting` letter that recognises but has nothing visible to
    /// show in this crate's scope (`C`/`L`/`E`/`F`/`N`/`O`) both land on
    /// `TraceMode::OFF` -- `mode_from_setting` draws no distinction between
    /// them because this crate cannot observe one (measured: `trace` alone
    /// and `trace value 'N'` are both silent, this task's own report has
    /// the transcript).
    ///
    /// `Trace::Setting`'s own bytes were already validated by `rexx-parse`'s
    /// `check_trace_setting` at parse time, so `.expect()` rather than
    /// propagating the `Err` arm -- a `Trace::Setting` this crate ever sees
    /// cannot carry an unrecognised letter. `Trace::Value`'s text has no
    /// such guarantee (it is computed at run time from an arbitrary Rexx
    /// expression), which is the one path that can reach `mode_from_
    /// setting`'s `Err` for real, and does through `raised_invalid_trace_
    /// letter`.
    fn exec_trace(&mut self, code: &Code<'_>, setting: &Trace) -> Result<(), Failure> {
        match setting {
            Trace::Default => {
                self.trace_mode = crate::trace::TraceMode::OFF;
            }
            Trace::Setting(bytes) => {
                self.trace_mode = mode_from_setting(bytes)
                    .expect("rexx-parse's check_trace_setting already validated this byte");
            }
            // 24.901, unconditional -- measured, `trace 0` raises it
            // exactly like `trace 5` (this task's report), because this
            // runtime has no interactive debugging for a nonzero skip
            // count to be valid *from* either way.
            Trace::Skip(_) => {
                return Err(raised_numeric_trace_interactive_only().into());
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
                let text = self.to_text(value).to_vec();
                if is_whole_number(&text) {
                    return Err(raised_numeric_trace_interactive_only().into());
                }
                self.trace_mode = mode_from_setting(&text).map_err(raised_invalid_trace_letter)?;
            }
        }
        Ok(())
    }

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

/// `instruction`'s own 1-based line and clause text, or `None` when `source`
/// is `None` (`run_fragment`'s own established convention of not resolving a
/// site for an instruction inside an `INTERPRET` fragment at all).
///
/// A free function, not a method: needs no `&self`, and both
/// `record_failure_site` and `leave_origin` resolve the identical pair from
/// it rather than each carrying its own copy.
fn clause_site(
    source: Option<&ProgramSource>,
    instruction: &Instruction,
) -> Option<(usize, Vec<u8>)> {
    let source = source?;
    Some((
        source.line_of(instruction.clause_span.start),
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

/// How many spaces of nesting depth `target`'s own clause sits at --
/// Task 11's whole indentation feature, and the design decision at its
/// centre: **computed fresh from the flat instruction list every time,
/// never carried on a running `Interp` counter.**
///
/// Task 10's own report concluded the depth is derivable from the AST
/// statically, with no runtime block stack, and this task's own oracle
/// probes confirm it (the report has the full transcripts): a clause's
/// indentation never depends on which iteration of an enclosing loop is
/// currently running, only on how many `DO`/`LOOP` bodies, matched `IF`
/// branches and `SELECT` scans lexically enclose it -- exactly the
/// information `If`'s `false_target`, `Select`'s `whens`/`otherwise`/`end`
/// and `Loop`'s `end` already carry, with nothing further to add.
///
/// **A mutable counter was the design first tried here, and it was dropped
/// once it became clear what it would cost to keep correct.** It would need
/// to be incremented and decremented in exact lockstep on *every* exit path
/// out of *every* `IF`/`SELECT`/`DO` arm, including every `?`-propagated
/// error path and the `Goto`-absorption case `Flow::Leave`'s own doc comment
/// describes -- precisely the shape of defect this crate's own skipped-
/// `pop_frame` discussion elsewhere warns is easy to introduce and hard to
/// notice, because the symptom is two spaces of wrong stderr that no
/// existing test asserts on. A pure function of `(instructions, target)`
/// cannot desync from anything, because there is no state to desync: asking
/// it twice for the same `target` on the same body always gives the same
/// answer, computed the same way, whether the failure happens on a loop's
/// first pass or its thousandth. `the_indent_after_a_loop_has_already_exited_
/// is_not_left_over_from_it` (this file's own tests) is the test that would
/// have caught a live counter's most likely failure mode -- a raise reached
/// *after* a loop's own body has already run and exited, at a shallower
/// lexical depth, where a counter not perfectly unwound on every path out of
/// the loop would over-indent and a purely static answer cannot.
///
/// **The one place this recomputes something rather than reading it back**
/// is `WHILE`/`UNTIL`: neither corresponds to a *distinct* flat instruction
/// position with the right semantics (`WHILE` shares the `DO`/`LOOP`
/// instruction's own clause, tested *inside* the loop's own frame; `UNTIL`
/// shares the `END`'s, likewise inside), so `Do`'s own arm adds the loop's
/// own two spaces on top of `static_indent(instructions, do_index)` directly
/// at its two call sites rather than asking this function to guess which of
/// two different, correct answers a `DO`/`LOOP` instruction's *own* index
/// means (measured: `do i = 1 to 3 for 1/0`'s control-setup failure is
/// unindented at that same index, while `do while 1/0` is indented two).
///
/// Recurses into whichever construct's own range contains `target`, adding
/// that construct's contribution before descending -- see the report for
/// the additive model (two per `DO`/`LOOP`, four per matched `IF` branch,
/// two for a `SELECT`'s own scan plus four more for a matched `WHEN`'s
/// `THEN` or two more for `OTHERWISE`) and the oracle transcripts that pin
/// each number, including the two the brief this task started from did not
/// state: a `WHEN`'s own condition sits at the `SELECT`'s own two, not zero,
/// and `OTHERWISE`'s own body is two more, not the `WHEN`-`THEN` shape's
/// four more.
fn static_indent(instructions: &[Instruction], target: usize) -> usize {
    indent_in_range(instructions, 0, instructions.len(), target)
}

/// `static_indent`'s own recursive worker, over one `[start, end)` range --
/// the same range shape `run_bounded` itself runs, so this function's
/// dispatch on `If`/`Select`/`Do`/`Loop` mirrors `step`'s own arms for them,
/// reading the identical fields, just never evaluating anything.
fn indent_in_range(instructions: &[Instruction], start: usize, end: usize, target: usize) -> usize {
    let len = instructions.len();
    let mut pc = start;
    while pc < end {
        if pc == target {
            // `target` is this range's own instruction at this position --
            // a plain clause, or a block-opener's own clause (a `DO`'s
            // control-setup expressions, a `SELECT`'s own `CASE` scrutinee),
            // with nothing further to add beyond whatever the caller already
            // contributed before recursing in here.
            return 0;
        }
        match &instructions[pc].kind {
            InstructionKind::If { false_target, .. } => {
                let false_target = false_target.unwrap_or(len);
                let then_start = pc + 1;
                // `then_start` is the `Then` marker's *own* index, not its
                // body's first instruction -- measured against the oracle
                // (`ThenInstruction.cpp`'s `execute`: `indent(); trace;
                // indent();`), a marker clause sits at exactly two spaces,
                // half of what its own body gets (four). Before this check
                // existed, `target == then_start` fell into the body branch
                // below and got the wrong answer (4, not 2) because that
                // branch's own recursive call happens to return 0 for the
                // very first position of its range -- silently, since
                // nothing before this task ever asked for a `Then`'s own
                // indent (a marker clause carries no expression, so it can
                // never be a `FailureSite`, only ever a `TRACE` echo).
                if target == then_start {
                    return 2;
                }
                if target > then_start && target < false_target {
                    return 4 + indent_in_range(instructions, then_start, false_target, target);
                }
                match instructions.get(false_target).map(|i| &i.kind) {
                    Some(InstructionKind::Else { then_exit }) => {
                        let else_end = then_exit.unwrap_or(len);
                        // Same shape as `Then`, and the same measurement
                        // (`ElseInstruction.cpp`'s `execute` is byte-for-byte
                        // the same two-`indent()`-calls dance). Before this
                        // check, `target == false_target` fell all the way
                        // through this whole arm (the body check below is
                        // strict `>`) to `pc = else_end; continue`, which
                        // advances `pc` *past* `target` in the enclosing
                        // walk -- the `Else` marker's own index was never
                        // revisited by anything, silently returning
                        // whatever the *enclosing* level happened to be
                        // (0 too shallow) rather than erroring.
                        if target == false_target {
                            return 2;
                        }
                        if target > false_target && target < else_end {
                            return 4 + indent_in_range(
                                instructions,
                                false_target + 1,
                                else_end,
                                target,
                            );
                        }
                        pc = else_end;
                    }
                    _ => pc = false_target,
                }
                continue;
            }
            InstructionKind::Do(body) | InstructionKind::Loop(body) => {
                let body_start = pc + 1;
                let end_index = body
                    .end
                    .expect("an End's closes is only None while its body is still being assembled");
                if target > pc && target < end_index {
                    return 2 + indent_in_range(instructions, body_start, end_index, target);
                }
                pc = end_index + 1;
                continue;
            }
            InstructionKind::Select {
                whens,
                otherwise,
                end: select_end,
                ..
            } => {
                let select_end = select_end.unwrap_or(len);
                if target > pc && target < select_end {
                    // Inside this SELECT's own scan-through-dispatch range:
                    // two spaces on their own (measured: a WHEN's own
                    // condition, `target == when_index` below, sits at
                    // exactly this level), plus whichever branch's own
                    // extra applies.
                    for &when_index in whens {
                        if when_index == target {
                            return 2;
                        }
                        let (body_start, body_end) = match &instructions[when_index].kind {
                            InstructionKind::When { false_target, .. }
                            | InstructionKind::WhenCase { false_target, .. } => {
                                (when_index + 1, false_target.unwrap_or(len))
                            }
                            // This asserted unreachability too, on a claim
                            // that turned out to be no better-founded than
                            // the `OTHERWISE` one three lines of history
                            // below: "`whens` holds only `When`/`WhenCase`"
                            // is `rexx-parse`'s own invariant, not this
                            // function's, and this phase's invariants have
                            // not all held -- the absorbed-`WHEN` case
                            // (`when_absorbing_a_when_parses_and_runs_at_
                            // rc_0`, this module's own test) is exactly a
                            // `When` instruction executing while its
                            // enclosing `SELECT`'s own `whens` does not list
                            // it, which is the same shape of surprise. If a
                            // reader ever sees this, `whens` names an index
                            // whose own kind is not what built it -- a
                            // `rexx-parse` defect, not a formatting one, and
                            // nothing this function can correct. Skipping
                            // the entry (matching the outer fallback's own
                            // "nothing further to add" answer once the loop
                            // and the `OTHERWISE` check both come up empty)
                            // keeps the diagnostic path alive instead of
                            // trading a wrong indent for a dead process.
                            _ => continue,
                        };
                        // `body_start` is the WHEN's own `Then` marker,
                        // sharing `InstructionKind::Then` with `IF` (both go
                        // through `instruction.rs`'s `if_instruction`) --
                        // measured against the oracle exactly like `IF`'s
                        // own: the marker sits at half its body's indent
                        // (four, not six). Before this check, `target ==
                        // body_start` matched the `>=` below and returned
                        // six, the body's own value -- again invisible
                        // before `TRACE`, since a `Then` marker never raises.
                        if target == body_start {
                            return 4;
                        }
                        if target > body_start && target < body_end {
                            return 6 + indent_in_range(instructions, body_start, body_end, target);
                        }
                    }
                    if let Some(otherwise_index) = otherwise {
                        // `OTHERWISE` traces its own clause once (no double
                        // `indent()` -- `OtherwiseInstruction.cpp`'s
                        // `execute` is `trace; indent();`, not `indent();
                        // trace; indent();`) at the SELECT's own scan level,
                        // the same two spaces a `WHEN`'s condition gets, and
                        // only its body gets the further two. Before this
                        // check, `target == *otherwise_index` matched
                        // neither this arm's `>` check nor anything in the
                        // `whens` loop, and fell all the way to the
                        // `unreachable!` below -- **a live panic**, not
                        // merely a wrong number, confirmed by directly
                        // calling `static_indent` on `select\nwhen 1 = 0
                        // then nop\notherwise\nsay 'y'\nend`'s own
                        // `otherwise_index` before this fix existed.
                        if target == *otherwise_index {
                            return 2;
                        }
                        if target > *otherwise_index && target < select_end {
                            return 4 + indent_in_range(
                                instructions,
                                otherwise_index + 1,
                                select_end,
                                target,
                            );
                        }
                    }
                    // `target` is in range but matches none of the above.
                    // After the two equality cases just added, every
                    // reachable position inside a resolved SELECT is
                    // provably one of: a WHEN's own index, a WHEN's own
                    // `Then` marker, one WHEN's own body, `OTHERWISE`'s own
                    // marker, or `OTHERWISE`'s own body -- so a body that
                    // parsed should never reach here. It reached here once
                    // already, though (the `OTHERWISE`-marker case, before
                    // its equality check existed), and this exact arm is
                    // where that panic actually happened -- `unreachable!`
                    // asserted a claim about the code's own shape, and the
                    // claim was false for a case nothing had exercised yet.
                    // This crate's rule for the diagnostic path (`error.rs`'s
                    // message-catalogue miss renders a visible marker
                    // instead of aborting; `run.rs:536` cites the same
                    // reasoning) is that a formatting gap must never become
                    // a crash, and `static_indent` feeds both the error
                    // report and `TRACE` now -- so this returns the
                    // enclosing level (0 relative, "nothing further to add")
                    // rather than asserting unreachability a second time.
                    // If a reader ever sees indentation that looks too
                    // shallow by exactly the amount a `SELECT` construct
                    // should have contributed, this is where to look: it
                    // means some future `SELECT`-shaped clause position is
                    // not one of the five cases enumerated above.
                    return 0;
                }
                pc = select_end;
                continue;
            }
            _ => {}
        }
        pc += 1;
    }
    0
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

/// 34.3: a single (non-list) `WHILE` condition is not exactly `0` or `1`.
/// Same shape as `raised_if_not_logical`/`raised_when_not_logical`, with
/// `WHILE`'s own sub-number; a comma-list condition never reaches this
/// raiser (34.6 instead, `eval_logical_list`'s own answer).
fn raised_while_not_logical(found: &[u8]) -> Raised {
    Raised {
        condition: "SYNTAX",
        number: 34,
        sub: 3,
        additional: vec![String::from_utf8_lossy(found).into_owned()],
    }
}

/// 34.4: `UNTIL`'s own version of `raised_while_not_logical`.
fn raised_until_not_logical(found: &[u8]) -> Raised {
    Raised {
        condition: "SYNTAX",
        number: 34,
        sub: 4,
        additional: vec![String::from_utf8_lossy(found).into_owned()],
    }
}

/// 26.2: a bare `DO`'s own repetition-count expression is not zero or a
/// positive whole number. `Error_Invalid_expression_do`, measured: `do
/// 'a'`/`do -1`/`do 2.5` all give this, `found` the operand's own
/// unmodified text (`"a"`/`"-1"`/`"2.5"`).
fn raised_repetition_count_not_whole(found: &[u8]) -> Raised {
    Raised {
        condition: "SYNTAX",
        number: 26,
        sub: 2,
        additional: vec![String::from_utf8_lossy(found).into_owned()],
    }
}

/// 26.3: a `DO`/`LOOP`'s `FOR` expression is not zero or a positive whole
/// number. Measured: `do i = 1 to 3 for 'x'`/`for -1`/`for 1.5`.
fn raised_for_count_not_whole(found: &[u8]) -> Raised {
    Raised {
        condition: "SYNTAX",
        number: 26,
        sub: 3,
        additional: vec![String::from_utf8_lossy(found).into_owned()],
    }
}

/// 28.1: a bare `LEAVE` found no repetitive loop or labeled block
/// instruction anywhere on the enclosing chain. No substitution.
fn raised_leave_no_loop() -> Raised {
    Raised {
        condition: "SYNTAX",
        number: 28,
        sub: 1,
        additional: Vec::new(),
    }
}

/// 28.2: a bare `ITERATE` found no repetitive loop anywhere on the
/// enclosing chain. No substitution.
fn raised_iterate_no_loop() -> Raised {
    Raised {
        condition: "SYNTAX",
        number: 28,
        sub: 2,
        additional: Vec::new(),
    }
}

/// 28.3: a named `LEAVE name` found nothing on the enclosing chain whose own
/// label (`DO LABEL`, or a controlled/`OVER` loop's own control variable)
/// matches `name` -- **an ordinary clause label never matches**, measured:
/// `outer: do i = 1 to 3` then `leave outer` is this, not a hit. `found` is
/// the symbol's own (already-upcased) spelling.
fn raised_leave_no_match(found: &[u8]) -> Raised {
    Raised {
        condition: "SYNTAX",
        number: 28,
        sub: 3,
        additional: vec![String::from_utf8_lossy(found).into_owned()],
    }
}

/// 28.4: `ITERATE`'s own version of `raised_leave_no_match`.
fn raised_iterate_no_match(found: &[u8]) -> Raised {
    Raised {
        condition: "SYNTAX",
        number: 28,
        sub: 4,
        additional: vec![String::from_utf8_lossy(found).into_owned()],
    }
}

/// 28.5: a named `ITERATE name` matched a block on the enclosing chain by
/// label, but that block is not a repetitive loop (a labelled `DO`/plain
/// block, or a `SELECT LABEL` -- `ITERATE` never accepts either, unlike
/// `LEAVE`). Measured: `do label x / say 1 / iterate x / end` gives this,
/// not 28.4, because `x` *did* match something.
fn raised_iterate_wrong_kind(found: &[u8]) -> Raised {
    Raised {
        condition: "SYNTAX",
        number: 28,
        sub: 5,
        additional: vec![String::from_utf8_lossy(found).into_owned()],
    }
}

/// Whether `a < b`, numerically, through `rexx-num`'s own `compare_decoded`
/// rather than a hand-rolled sign comparison -- this crate's standing rule
/// against a second copy of a comparison `rexx-num` already owns
/// (`eval_compare`'s own doc comment states it for the twelve expression
/// operators; a controlled loop's own bound test and its `BY`'s sign are
/// the same rule applied to two `Number`s this crate already holds, not a
/// different one).
///
/// **The empty byte slices are provably unused, not a placeholder standing
/// in for something forgotten.** `compare_decoded`'s own body only reads
/// its `bytes` arguments when at least one side's `Option<Number>` is
/// `None` (the string-fallback and strict families) -- every call here
/// passes `Some` on both sides and a non-strict `CompareOp`, so the branch
/// that would read `a`/`b` is never taken. Passing real text would cost an
/// unwanted `Number::format` round-trip (rendering, then reparsing, which
/// is not exactly what a fresh comparison of the already-held `Number`s
/// would give at the boundary of a value too wide for `digits` to hold
/// exactly) for bytes the function does not use.
fn numeric_less(a: &Number, b: &Number, digits: u64, fuzz: u64) -> Result<bool, ArithError> {
    compare_decoded(b"", Some(a), b"", Some(b), digits, fuzz, CompareOp::Less)
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
            // A miniature of `run_activation`'s own top-level conversion
            // (its doc comment on the same two arms has the full argument):
            // nothing anywhere in this test program's own body consumed
            // the `LEAVE`/`ITERATE`, so it becomes the exhausted-search
            // error, at whatever residual indent the search's own walk
            // back up already left in `origin`.
            Flow::Leave(name, origin) => {
                interp.record_leave_failure(&origin);
                let raised = match name {
                    None => raised_leave_no_loop(),
                    Some(n) => raised_leave_no_match(code.symbols.name(n).as_bytes()),
                };
                Err(raised.into())
            }
            Flow::Iterate(name, origin) => {
                interp.record_leave_failure(&origin);
                let raised = match name {
                    None => raised_iterate_no_loop(),
                    Some(n) => raised_iterate_no_match(code.symbols.name(n).as_bytes()),
                };
                Err(raised.into())
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
    /// `abXc`. A version that never runs the `ELSE` branch on the false path
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

    /// **Critical, found by review, not by this task's own probes.** The
    /// mutation this kills is exactly the one the old `Ok(Flow::Next)`
    /// arm *was*: treating an absorbed `WHEN`'s own condition as never
    /// evaluated at all, rather than evaluated and discarded. A version
    /// that reverts `InstructionKind::When`'s arm to a bare `Ok(Flow::
    /// Next)` makes this test hang around `run_source` returning `Ok`
    /// (prints `after`, rc 0) where the oracle raises 42.3 at rc 214 --
    /// `an_absorbed_when_runs_neither_its_own_consequence_nor_otherwise`,
    /// just above, cannot distinguish the two models (both give `n = 0`
    /// for a side-effect-free true condition), which is exactly why every
    /// probe before this review used one and missed this.
    #[test]
    fn an_absorbed_whens_raising_condition_escapes_even_though_its_own_consequence_never_runs() {
        let mut interp = Interp::new(false);
        let failure = run_source(
            &mut interp,
            b"select\n  when 1 = 1 then\n    when 1 / 0 then nop\n  otherwise nop\nend\nsay 'after'",
        )
        .unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (42, 3));
    }

    /// The companion half: a **true** absorbed condition with a printable
    /// side effect still never runs its own consequence (matching the
    /// existing `n = 0` test, restated with `SAY` so a wrong "the
    /// absorbed WHEN's branch is taken" model would be caught by output
    /// content rather than only by a variable's final value) -- measured,
    /// this task's own report: `ABSORBED-RAN` never prints, only `after`.
    #[test]
    fn an_absorbed_whens_true_condition_still_never_runs_its_own_consequence() {
        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(
                &mut interp,
                b"select\n  when 1 = 1 then\n    when 2 = 2 then say 'ABSORBED-RAN'\nend\nsay 'after'"
            ),
            b"after\n".to_vec(),
            "a reverted fix would print ABSORBED-RAN first"
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

    /// **The coordinator's own finding, fixed after the first round.** A
    /// `WHEN`'s own `step` arm is a pure no-op (`Select`'s own arm reads it
    /// as data instead), so a raise while evaluating its *condition* never
    /// went through a `step_in_temps_frame` call for the `WHEN` itself --
    /// the first version of this task attributed it to the enclosing
    /// `SELECT`, wrong clause *and* wrong line, measured against the
    /// oracle. `record_failure_site`'s own doc comment on `Select`'s call
    /// sites has the fix. Checks `failure_site` directly (line and clause
    /// text), which `raised.number`/`.sub` alone -- every other test in
    /// this file -- cannot: both were already unaffected by the bug, since
    /// the bug is entirely in *which clause* gets echoed, not in which
    /// condition failed.
    #[test]
    fn a_when_conditions_own_failure_is_attributed_to_the_when_not_the_select() {
        let mut interp = Interp::new(false);
        run_source(&mut interp, b"select\nwhen 'x' then nop\nend").unwrap_err();
        let FailureSite { line, text, .. } = interp
            .failure_site
            .expect("a raised condition always resolves a site when source is Some");
        assert_eq!(line, 2, "the WHEN's own line, not the SELECT's (line 1)");
        assert_eq!(
            text,
            b"when 'x' ".to_vec(),
            "the WHEN's own clause text, not \"select\""
        );
    }

    /// The line has to move with the failing `WHEN`, not merely differ from
    /// the `SELECT`'s -- a test whose expected line is the first `WHEN`'s
    /// cannot tell a correct resolution from one that defaults to
    /// "whichever `WHEN` this loop happens to be looking at first".
    #[test]
    fn the_second_of_two_whens_own_failure_moves_the_line_with_it() {
        let mut interp = Interp::new(false);
        run_source(
            &mut interp,
            b"select\nwhen 1 = 0 then nop\nwhen 'x' then nop\nend",
        )
        .unwrap_err();
        let FailureSite { line, text, .. } = interp.failure_site.expect("a site was resolved");
        assert_eq!(line, 3, "the second WHEN's own line, not the first's (2)");
        assert_eq!(text, b"when 'x' ".to_vec());
    }

    /// A `SELECT CASE` expression that itself raises is the `SELECT`'s own
    /// clause -- confirmed against the oracle rather than assumed, since
    /// `case` is evaluated directly inside `Select`'s own `step` call and
    /// the coordinator asked this be checked, not taken on faith.
    #[test]
    fn a_select_cases_own_expression_failure_is_attributed_to_the_select() {
        let mut interp = Interp::new(false);
        run_source(&mut interp, b"select case (1/0)\nwhen 1 then nop\nend").unwrap_err();
        let FailureSite { line, text, .. } = interp.failure_site.expect("a site was resolved");
        assert_eq!(line, 1);
        assert_eq!(text, b"select case (1/0)".to_vec());
    }

    /// A `WhenCase` value expression that raises is the `WHEN`'s own
    /// clause, the same rule as a plain `WHEN`'s condition -- both go
    /// through `Select`'s own explicit-match-and-record path, never
    /// through `step_in_temps_frame` for the `When`/`WhenCase` node itself.
    #[test]
    fn a_whencase_values_own_failure_is_attributed_to_the_when_not_the_select() {
        let mut interp = Interp::new(false);
        run_source(&mut interp, b"select case 1\nwhen (1/0) then nop\nend").unwrap_err();
        let FailureSite { line, text, .. } = interp.failure_site.expect("a site was resolved");
        assert_eq!(line, 2);
        assert_eq!(text, b"when (1/0) ".to_vec());
    }

    /// A raise inside an `OTHERWISE` branch was already correct before this
    /// round's fix (`OTHERWISE`'s own body runs through the outer loop's
    /// ordinary `step_in_temps_frame`, never through `Select`'s own
    /// explicit-match path), and stays that way -- checked because the
    /// coordinator asked for it explicitly, not assumed from the `WHEN` fix.
    #[test]
    fn a_raise_inside_an_otherwise_branch_is_attributed_to_its_own_clause() {
        let mut interp = Interp::new(false);
        run_source(
            &mut interp,
            b"select\nwhen 1 = 0 then nop\notherwise\n  say 1/0\nend",
        )
        .unwrap_err();
        let FailureSite { line, text, .. } = interp.failure_site.expect("a site was resolved");
        assert_eq!(line, 4);
        assert_eq!(text, b"say 1/0".to_vec());
    }

    /// A raise inside a matched `WHEN`'s **body** (not its condition) has to
    /// go through `run_bounded`'s own `step_in_temps_frame` calls with
    /// `source` actually threaded through, which is a different path from
    /// every other test in this section: those all check a condition/value
    /// expression `Select`'s own arm evaluates directly, and
    /// `a_raise_inside_an_otherwise_branch_is_attributed_to_its_own_clause`
    /// runs through the *outer* loop's `step_in_temps_frame`, never through a
    /// `run_bounded` nested inside `If`/`Select`'s own arm. That last
    /// distinction matters and an earlier wording of it was wrong: in the
    /// test harness `run_source` routes everything through its own top-level
    /// `run_bounded` with `source` supplied directly, so "never through
    /// `run_bounded` at all" is true of the production outer loop only.
    ///
    /// This is round 1's own defect class -- an error escaping a nested
    /// `run_bounded` call misattributed to the enclosing construct -- and no
    /// existing test exercises the path that would regress if `source`
    /// stopped being threaded into `run_bounded`. Confirmed by mutation, not
    /// assumed: passing `None` in place of `source` at the two call sites,
    /// which are `If`'s and `Select`'s and not two of `Select`'s own, made
    /// this test and the `IF` one below fail while leaving all 102
    /// pre-existing tests green. Restored immediately after.
    #[test]
    fn a_raise_inside_a_matched_whens_body_is_attributed_to_its_own_clause() {
        let mut interp = Interp::new(false);
        run_source(&mut interp, b"select\nwhen 1 = 1 then\n  say 1/0\nend").unwrap_err();
        let FailureSite { line, text, .. } = interp.failure_site.expect("a site was resolved");
        assert_eq!(
            line, 3,
            "the WHEN's own body clause, not the SELECT's (line 1)"
        );
        assert_eq!(text, b"say 1/0".to_vec());
    }

    /// The `IF` analogue of the `WHEN`-body test above: a raise inside the
    /// matched `THEN` branch's own body, which likewise only ever reaches
    /// `step_in_temps_frame` through `run_bounded`. Confirmed by the same
    /// mutation (`None` for `source` at `If`'s own `run_bounded` call site
    /// made this fail too, alongside the `WHEN`-body test, both restored
    /// after).
    #[test]
    fn a_raise_inside_an_ifs_then_body_is_attributed_to_its_own_clause() {
        let mut interp = Interp::new(false);
        run_source(&mut interp, b"if 1 = 1 then\n  say 1/0").unwrap_err();
        let FailureSite { line, text, .. } = interp.failure_site.expect("a site was resolved");
        assert_eq!(
            line, 2,
            "the THEN branch's own body clause, not the IF's (line 1)"
        );
        assert_eq!(text, b"say 1/0".to_vec());
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

    // ---- DO/LOOP: every LoopKind ----

    #[test]
    fn a_simple_do_block_runs_its_body_exactly_once_with_no_control() {
        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(&mut interp, b"do\nsay 'once'\nend"),
            b"once\n".to_vec()
        );
    }

    /// `LOOP` alone is `DO FOREVER`; `DO` alone is a block, not a loop
    /// (`ast.rs`'s own doc comment on `LoopKind::Simple`/`Forever`) --
    /// proven here by the fact that only the `LOOP` form is stopped by a
    /// bare `LEAVE`.
    #[test]
    fn loop_alone_is_forever_and_do_alone_is_a_block_not_a_loop() {
        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(
                &mut interp,
                b"n = 0\nloop\nn = n + 1\nif n = 3 then leave\nend\nsay n"
            ),
            b"3\n".to_vec()
        );
    }

    #[test]
    fn do_forever_runs_until_a_bare_leave() {
        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(
                &mut interp,
                b"n = 0\ndo forever\nn = n + 1\nif n = 3 then leave\nend\nsay n"
            ),
            b"3\n".to_vec()
        );
    }

    #[test]
    fn do_count_repeats_exactly_the_evaluated_number_of_times() {
        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(&mut interp, b"n = 0\ndo 3\nn = n + 1\nend\nsay n"),
            b"3\n".to_vec()
        );
        // The repeat count is an expression, evaluated once.
        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(&mut interp, b"n = 0\ndo 1 + 2\nn = n + 1\nend\nsay n"),
            b"3\n".to_vec()
        );
        // Zero repetitions is legal and runs the body no times at all.
        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(&mut interp, b"n = 0\ndo 0\nn = n + 1\nend\nsay n"),
            b"0\n".to_vec()
        );
    }

    /// F1, found by review: a bare count `DO n` traced no `>K>` line at
    /// all, where the oracle traces it tagged `FOR` -- the same tag an
    /// explicit `DO ... FOR n` gets, measured (this task's own report).
    /// The mutation this kills is exactly the gap: deleting the new
    /// `trace_keyword` call in `run_loop`'s `LoopKind::Count` arm makes
    /// `interp.trace` empty instead of carrying the `>K>` line, with
    /// every other assertion in this file untouched -- this is the one
    /// test that would have caught the omission, since the report's own
    /// verification claimed `>K>` was checked while never actually
    /// running a bare-count program through it.
    #[test]
    fn a_bare_repeat_count_traces_as_for_the_same_as_an_explicit_one() {
        let mut interp = Interp::new(false);
        interp.trace_mode = mode_from_setting(b"r").expect("R is a valid TRACE setting");
        say_output(&mut interp, b"do 2\nnop\nend");
        // `>K>` fires exactly once, on the first pass, matching every
        // other single-evaluation control-setup keyword (`TO`/`BY`/
        // `OVER`) -- the third `do 2` re-echo is the exit-check pass
        // `run_repeating`'s own per-pass re-echo already covers, verified
        // by running this exact assertion once and reading the bytes
        // back rather than hand-composing them (`this file's own report
        // has the trap: a hand-guessed expectation here was short by
        // exactly this one pass on the first attempt).
        assert_eq!(
            interp.trace,
            b"     1 *-* do 2\n       >K>   \"FOR\" => \"2\"\n     2 *-*   nop\n     \
              3 *-* end\n     1 *-* do 2\n     2 *-*   nop\n     3 *-* end\n     1 *-* do 2\n"
                .to_vec()
        );
    }

    #[test]
    fn do_with_takes_the_loud_path() {
        let mut interp = Interp::new(false);
        let failure = run_source(&mut interp, b"do with index i over 'x'\nsay i\nend").unwrap_err();
        let Failure::Loud(_) = failure else {
            panic!("expected Loud, got {failure:?}");
        };
    }

    #[test]
    fn do_counter_takes_the_loud_path_regardless_of_which_other_kind_it_rides_on() {
        let mut interp = Interp::new(false);
        let failure = run_source(&mut interp, b"do counter c i = 1 to 3\nnop\nend").unwrap_err();
        let Failure::Loud(_) = failure else {
            panic!("expected Loud, got {failure:?}");
        };
    }

    // ---- DO i = TO/BY/FOR (controlled), and DO OVER ----

    #[test]
    fn a_controlled_loop_runs_to_then_by_then_stops() {
        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(&mut interp, b"do i = 1 to 3\nsay i\nend"),
            b"1\n2\n3\n".to_vec()
        );
        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(&mut interp, b"do i = 10 to 1 by -4\nsay i\nend"),
            b"10\n6\n2\n".to_vec(),
            "measured against the oracle, do_loop_forms.rex's own transcript"
        );
    }

    /// A non-whole control value is legal (Step 2's own table): `do i = 1.5
    /// to 3` steps by fractional values, not an error.
    #[test]
    fn a_controlled_loops_own_values_need_only_be_numeric_not_whole() {
        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(&mut interp, b"do i = 1.5 to 3\nsay i\nend"),
            b"1.5\n2.5\n".to_vec()
        );
    }

    /// `BY 0` loops forever -- behaviour to reproduce, not an error
    /// (Step 2's own table) -- bounded here by a `LEAVE` so the test itself
    /// terminates.
    #[test]
    fn a_controlled_loop_with_by_0_loops_forever() {
        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(
                &mut interp,
                b"i = 0\ndo j = 1 by 0 to 3\ni = i + 1\nif i > 5 then leave\nend\nsay i"
            ),
            b"6\n".to_vec(),
            "measured against the oracle"
        );
    }

    #[test]
    fn a_controlled_loops_own_for_caps_the_iteration_count_independent_of_to() {
        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(
                &mut interp,
                b"n = 0\ndo i = 1 to 100 for 3\nn = n + 1\nend\nsay n"
            ),
            b"3\n".to_vec()
        );
    }

    /// The control variable is bound to its own header value **before** the
    /// loop's own bound test, even for a loop that ends up running zero
    /// iterations -- measured against the oracle.
    #[test]
    fn the_control_variable_is_bound_even_when_the_loop_never_runs_its_body() {
        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(&mut interp, b"do i = 5 to 3\nsay 'never'\nend\nsay i"),
            b"5\n".to_vec()
        );
    }

    /// `DO name OVER expr` on a **non-stem** target: a string and a number
    /// each iterate exactly once, yielding themselves -- Deviation 1 keeps
    /// a stem target out of scope, and this test uses none.
    #[test]
    fn do_over_a_non_stem_target_iterates_once_yielding_itself() {
        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(&mut interp, b"do x over 'hello'\nsay x\nend"),
            b"hello\n".to_vec()
        );
        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(&mut interp, b"do x over 42\nsay x\nend"),
            b"42\n".to_vec()
        );
    }

    /// `DO OVER ... FOR` on a non-stem target: `FOR 0` skips the one
    /// iteration entirely; any `FOR` at least `1` still runs it exactly
    /// once (there is only ever one item). Not independently measured
    /// against the oracle (no oracle transcript pins `OVER ... FOR` on a
    /// non-stem target specifically); implemented as the direct, minimal
    /// extension of `FOR`'s own general "caps the iteration count" rule,
    /// and named as a judgement call in the report.
    #[test]
    fn do_over_for_0_skips_the_single_non_stem_iteration() {
        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(
                &mut interp,
                b"do x over 'hello' for 0\nsay x\nend\nsay 'after'"
            ),
            b"after\n".to_vec()
        );
        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(&mut interp, b"do x over 'hello' for 5\nsay x\nend"),
            b"hello\n".to_vec()
        );
    }

    /// Deviation 1 (`phase-4-exclusions.txt`): `DO OVER` on a stem does not
    /// reproduce the oracle's traversal order, and takes the loud path
    /// instead -- detected from `target`'s own syntax (a bare `NAME.`
    /// parses as `ExprKind::Stem`), never by evaluating it.
    #[test]
    fn do_over_a_stem_target_takes_the_loud_path() {
        let mut interp = Interp::new(false);
        let failure = run_source(&mut interp, b"a.1 = 'x'\ndo v over a.\nsay v\nend").unwrap_err();
        let Failure::Loud(_) = failure else {
            panic!("expected Loud, got {failure:?}");
        };
    }

    /// A stem target wrapped in parens is **also** caught -- corrected after
    /// review, which found this task's own comment on the `Over` arm
    /// claimed the opposite (`over (a.)` "is not detected"). It is detected:
    /// a single parenthesised sub-expression collapses to that
    /// sub-expression's own `ExprKind` rather than wrapping it in
    /// `ExprKind::List`, so `(a.)` is already `ExprKind::Stem` by the time
    /// `run_loop`'s own `matches!` check sees it, with nothing extra
    /// needed. The safe direction either way (loud, never a silent
    /// divergence), but the comment was wrong about which one it is.
    #[test]
    fn do_over_a_parenthesised_stem_target_is_also_caught() {
        let mut interp = Interp::new(false);
        let failure =
            run_source(&mut interp, b"a.1 = 'x'\ndo v over (a.)\nsay v\nend").unwrap_err();
        let Failure::Loud(_) = failure else {
            panic!("expected Loud, got {failure:?}");
        };
    }

    // ---- DO/LOOP header errors (Step 2's own table, re-measured) ----

    #[test]
    fn a_non_numeric_control_value_raises_41_1() {
        for (source, found) in [
            (&b"do i = 'a' to 3\nnop\nend"[..], "a"),
            (&b"do i = 1 to 'x'\nnop\nend"[..], "x"),
            (&b"do i = 1 by 'x'\nnop\nend"[..], "x"),
        ] {
            let mut interp = Interp::new(false);
            let failure = run_source(&mut interp, source).unwrap_err();
            let Failure::Raised(raised) = failure else {
                panic!("expected Raised, got {failure:?}");
            };
            assert_eq!((raised.number, raised.sub), (41, 1), "{source:?}");
            assert_eq!(raised.additional, vec![found.to_string()], "{source:?}");
        }
    }

    #[test]
    fn a_bad_for_expression_raises_26_3() {
        for (source, found) in [
            (&b"do i = 1 to 3 for 'x'\nnop\nend"[..], "x"),
            (&b"do i = 1 to 3 for -1\nnop\nend"[..], "-1"),
            (&b"do i = 1 to 3 for 1.5\nnop\nend"[..], "1.5"),
        ] {
            let mut interp = Interp::new(false);
            let failure = run_source(&mut interp, source).unwrap_err();
            let Failure::Raised(raised) = failure else {
                panic!("expected Raised, got {failure:?}");
            };
            assert_eq!((raised.number, raised.sub), (26, 3), "{source:?}");
            assert_eq!(raised.additional, vec![found.to_string()], "{source:?}");
        }
    }

    #[test]
    fn a_bad_repetition_count_raises_26_2() {
        for (source, found) in [
            (&b"do 'a'\nnop\nend"[..], "a"),
            (&b"do -1\nnop\nend"[..], "-1"),
            (&b"do 2.5\nnop\nend"[..], "2.5"),
        ] {
            let mut interp = Interp::new(false);
            let failure = run_source(&mut interp, source).unwrap_err();
            let Failure::Raised(raised) = failure else {
                panic!("expected Raised, got {failure:?}");
            };
            assert_eq!((raised.number, raised.sub), (26, 2), "{source:?}");
            assert_eq!(raised.additional, vec![found.to_string()], "{source:?}");
        }
    }

    // ---- WHILE/UNTIL ----

    #[test]
    fn do_while_tests_the_condition_before_the_body_and_do_until_after() {
        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(&mut interp, b"i = 0\ndo while i < 2\ni = i + 1\nsay i\nend"),
            b"1\n2\n".to_vec()
        );
        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(
                &mut interp,
                b"i = 5\ndo until i >= 7\ni = i + 1\nsay i\nend"
            ),
            b"6\n7\n".to_vec(),
            "measured against the oracle, do_loop_forms.rex's own transcript"
        );
    }

    #[test]
    fn a_while_condition_that_is_not_0_or_1_raises_34_3_not_34_1() {
        let mut interp = Interp::new(false);
        let failure = run_source(&mut interp, b"do while 'x'\nnop\nend").unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (34, 3));
        assert_eq!(raised.additional, vec!["x".to_string()]);
    }

    #[test]
    fn an_until_condition_that_is_not_0_or_1_raises_34_4() {
        let mut interp = Interp::new(false);
        let failure = run_source(&mut interp, b"do until 'x'\nnop\nend").unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (34, 4));
        assert_eq!(raised.additional, vec!["x".to_string()]);
    }

    /// A comma-list `WHILE`/`UNTIL` condition is 34.6, never 34.3/34.4 --
    /// `eval_condition`'s own rule, reused unchanged from `IF`/`WHEN`.
    #[test]
    fn a_comma_list_while_condition_raises_34_6_not_34_3() {
        let mut interp = Interp::new(false);
        let failure = run_source(&mut interp, b"do while 'x', 1\nnop\nend").unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (34, 6));
    }

    /// **The discriminating measurement for this whole section**: `UNTIL`
    /// is tested at the *bottom* of the loop, so its own failure is
    /// attributed to the `END`'s own clause, not the `DO`'s -- a loop that
    /// evaluated `UNTIL` eagerly, at the top like `WHILE`, would still
    /// produce the same raised number and the same exit code, and only the
    /// echoed clause reveals the difference. Measured against the oracle:
    /// `do until 'x' / end` echoes `end`, `do while 'x' / end` echoes the
    /// `do` line.
    #[test]
    fn until_is_attributed_to_the_end_clause_while_while_is_attributed_to_the_do_clause() {
        let mut interp = Interp::new(false);
        run_source(&mut interp, b"do until 'x'\nnop\nend").unwrap_err();
        let FailureSite { line, text, .. } = interp.failure_site.expect("a site was resolved");
        assert_eq!(line, 3, "the END's own line");
        assert_eq!(text, b"end".to_vec(), "the END's own clause, not the DO's");

        let mut interp = Interp::new(false);
        run_source(&mut interp, b"do while 'x'\nnop\nend").unwrap_err();
        let FailureSite { line, text, .. } = interp.failure_site.expect("a site was resolved");
        assert_eq!(line, 1, "the DO's own line");
        assert_eq!(text, b"do while 'x'".to_vec(), "the DO's own clause");
    }

    /// **`ITERATE` jumps to the loop's own bottom-of-iteration bookkeeping,
    /// not to the top of the next pass** -- measured against the oracle,
    /// this exact program terminates immediately with `n` still `1`, rather
    /// than looping forever. A design that instead skipped `UNTIL` for the
    /// interrupted iteration would hang on this program (`n` would keep
    /// advancing past `1` and `UNTIL n = 1` would never hold again), which
    /// is the mutation this test is built to catch -- not run, for the
    /// obvious reason a hang cannot be asserted against, but predicted and
    /// confirmed absent by construction: `do_body_outcome`'s own `Iterate`
    /// arm answers `Ok(None)`, the *same* value `Flow::Next` answers, so
    /// `run_repeating`'s own `UNTIL` check below it runs on both paths
    /// identically.
    #[test]
    fn iterate_reaches_untils_own_bottom_of_iteration_test_rather_than_skipping_it() {
        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(
                &mut interp,
                b"n = 0\ndo until n = 1\nn = n + 1\nif n = 1 then iterate\nsay 'unreached'\nend\nsay 'done' n"
            ),
            b"done 1\n".to_vec()
        );
    }

    // ---- LEAVE/ITERATE: the block-stack rules ----

    #[test]
    fn bare_leave_stops_the_innermost_loop() {
        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(
                &mut interp,
                b"do i = 1 to 5\nif i = 3 then leave\nsay i\nend"
            ),
            b"1\n2\n".to_vec()
        );
    }

    #[test]
    fn bare_iterate_skips_the_rest_of_the_current_pass() {
        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(
                &mut interp,
                b"do i = 1 to 4\nif i = 2 then iterate\nsay i\nend"
            ),
            b"1\n3\n4\n".to_vec()
        );
    }

    /// A bare `LEAVE`/`ITERATE` skips **transparently past** an unlabelled
    /// `DO` block on its way to the nearest enclosing loop -- measured
    /// against the oracle: this program prints `1` then `after`, not
    /// `unreached`, because the innermost `DO` (a block, not a loop) never
    /// intercepts the bare `LEAVE` at all.
    #[test]
    fn bare_leave_passes_transparently_through_an_unlabelled_simple_block() {
        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(
                &mut interp,
                b"do i = 1 to 3\ndo\nsay i\nleave\nsay 'unreached'\nend\nend\nsay 'after'"
            ),
            b"1\nafter\n".to_vec()
        );
    }

    /// **Bare `LEAVE` in a simple `DO` block is 28.1** (not a loop, and
    /// unlabelled, so nothing on the enclosing chain ever matches) -- but a
    /// **labelled** simple block is leavable by its own explicit name,
    /// which the next test pins.
    #[test]
    fn bare_leave_in_an_unlabelled_simple_block_with_nothing_enclosing_it_raises_28_1() {
        let mut interp = Interp::new(false);
        let failure = run_source(&mut interp, b"do\nleave\nend").unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (28, 1));
    }

    #[test]
    fn a_labelled_simple_block_is_leavable_by_its_own_name() {
        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(
                &mut interp,
                b"do label blk\nsay 'a'\nleave blk\nsay 'b'\nend\nsay 'after'"
            ),
            b"a\nafter\n".to_vec()
        );
    }

    /// **An ordinary clause label does not name a loop or a block.**
    /// `outer:` here is a plain `Label` instruction, entirely separate from
    /// the `DO`'s own `label` field (which is `i`, the control variable,
    /// since no `LABEL` keyword was written) -- measured, `leave outer` is
    /// 28.3, not a hit, exactly as `leave_nested_outer.rex`'s own comment
    /// states.
    #[test]
    fn an_ordinary_clause_label_does_not_name_the_loop_it_precedes() {
        let mut interp = Interp::new(false);
        let failure =
            run_source(&mut interp, b"outer: do i = 1 to 3\nleave outer\nend").unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (28, 3));
        assert_eq!(raised.additional, vec!["OUTER".to_string()]);

        let mut interp = Interp::new(false);
        let failure =
            run_source(&mut interp, b"outer: do i = 1 to 3\niterate outer\nend").unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (28, 4));
        assert_eq!(raised.additional, vec!["OUTER".to_string()]);
    }

    /// **What does name a loop for `LEAVE`/`ITERATE`**: a controlled loop's
    /// own control variable, as an automatic label, with no `LABEL` keyword
    /// needed at all. `leave i`/`iterate i` from inside a *nested* loop
    /// reaches the outer one and unwinds the inner one on the way --
    /// `leave_nested_outer.rex`'s own transcript, reproduced here.
    #[test]
    fn a_controlled_loops_own_control_variable_is_an_automatic_label() {
        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(
                &mut interp,
                b"do outer = 1 to 3\ndo inner = 1 to 3\nif inner = 2 then leave outer\nsay 'inner' outer inner\nend\nsay 'after inner' outer\nend\nsay 'after outer'"
            ),
            b"inner 1 1\nafter outer\n".to_vec()
        );
    }

    #[test]
    fn iterate_naming_the_outer_loops_control_variable_cuts_every_outer_pass_short() {
        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(
                &mut interp,
                b"do outer = 1 to 3\ndo inner = 1 to 3\nif inner = 2 then iterate outer\nsay 'l' outer inner\nend\nsay 'outer-after' outer\nend"
            ),
            b"l 1 1\nl 2 1\nl 3 1\n".to_vec(),
            "outer-after never prints for any pass, since every one is cut short at inner = 2"
        );
    }

    /// **`leave sel` exits a `SELECT` only when it was written
    /// `SELECT LABEL sel`.** An ordinary clause label in front of a
    /// `SELECT` does not make it leavable (28.3, exactly like a loop).
    #[test]
    fn leave_by_name_exits_a_select_only_when_it_was_given_that_label() {
        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(
                &mut interp,
                b"s: select label s\nwhen 1 = 1 then\ndo\nleave s\nend\notherwise\nnop\nend\nsay 'after'"
            ),
            b"after\n".to_vec()
        );

        let mut interp = Interp::new(false);
        let failure = run_source(
            &mut interp,
            b"select\nwhen 1 = 1 then\ndo\nleave sel\nend\notherwise\nnop\nend",
        )
        .unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (28, 3));
        assert_eq!(raised.additional, vec!["SEL".to_string()]);
    }

    /// A named `LEAVE` reaching a `SELECT LABEL` from inside its
    /// `OTHERWISE` branch -- the fix that routes `OTHERWISE`'s own body
    /// through `Select`'s own `run_bounded`/`leave_select`, not the plain
    /// `Goto` it used before Task 11 (`Select`'s own arm has the full
    /// argument). Kills a version that still lets `OTHERWISE` fall through
    /// to the outer loop directly: that version would propagate this
    /// `LEAVE` all the way to `run_activation`'s own top level and raise
    /// 28.3 instead of the clean exit this test asserts.
    #[test]
    fn leave_by_name_reaches_a_select_label_from_inside_its_otherwise_branch() {
        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(
                &mut interp,
                b"select label s\nwhen 1 = 0 then nop\notherwise\nsay 'o'\nleave s\nsay 'unreached'\nend\nsay 'after'"
            ),
            b"o\nafter\n".to_vec()
        );
    }

    /// **`ITERATE` never accepts a labelled block or a labelled `SELECT`,
    /// only a loop -- 28.5, not silently skipped, when the name matches but
    /// the kind is wrong.**
    #[test]
    fn a_named_iterate_matching_a_labelled_simple_block_raises_28_5_not_28_4() {
        let mut interp = Interp::new(false);
        let failure = run_source(&mut interp, b"do label x\nsay 1\niterate x\nend").unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (28, 5));
        assert_eq!(raised.additional, vec!["X".to_string()]);
    }

    #[test]
    fn a_named_iterate_matching_a_select_label_raises_28_5() {
        let mut interp = Interp::new(false);
        let failure = run_source(
            &mut interp,
            b"select label s\nwhen 1 = 1 then\ndo\niterate s\nend\notherwise\nnop\nend",
        )
        .unwrap_err();
        let Failure::Raised(raised) = failure else {
            panic!("expected Raised, got {failure:?}");
        };
        assert_eq!((raised.number, raised.sub), (28, 5));
        assert_eq!(raised.additional, vec!["S".to_string()]);
    }

    #[test]
    fn iterate_from_inside_a_select_nested_in_a_loop_skips_only_the_current_pass() {
        // iterate_from_select.rex's own transcript: an ITERATE inside a
        // SELECT's WHEN body must skip straight to the loop's own next
        // pass, past both the SELECT's own resume point and back up to the
        // enclosing DO, printing exactly one "skip"/"keep" pair per
        // iteration and never both for the same i.
        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(
                &mut interp,
                b"do i = 1 to 4\nselect\nwhen i = 2 then do\nsay 'skip' i\niterate\nend\notherwise nop\nend\nsay 'keep' i\nend"
            ),
            b"keep 1\nskip 2\nkeep 3\nkeep 4\n".to_vec()
        );
    }

    /// Two real, unnamed loops nested two deep, neither matching `zz`: both
    /// own a search frame (`is_loop` true for a `Controlled` loop), so both
    /// get popped and both reset the residual to their own `static_indent`
    /// -- the outer one last, to `0` (top level), which is what survives to
    /// the exhausted-search report. This is **not** "28.1-28.4 always
    /// report zero" (that rule was wrong -- see `n1`/`n2`/`n3` below, and
    /// `LeaveOrigin`'s own doc comment for the corrected one): it is zero
    /// here specifically because the outermost popped frame happens to sit
    /// at top level, the same way it did in every one of this task's own
    /// original probes, which is exactly how the wrong rule looked right
    /// for as long as it did.
    #[test]
    fn leave_no_match_through_two_real_loops_resets_to_the_outer_ones_own_indent() {
        let mut interp = Interp::new(false);
        run_source(
            &mut interp,
            b"do i = 1 to 3\ndo j = 1 to 3\nleave zz\nend\nend",
        )
        .unwrap_err();
        let FailureSite { indent, .. } = interp.failure_site.expect("a site was resolved");
        assert_eq!(indent, 0);
    }

    /// The one intervening construct is an *unlabelled* `Simple` block,
    /// which owns no search frame and is fully transparent -- so nothing
    /// ever resets the residual, and it stays at the `ITERATE`'s own full
    /// lexical depth all the way to the match (the outer labelled block,
    /// which does not reset on a match either). Two real loops in
    /// `leave_no_match_through_two_real_loops_resets_to_the_outer_ones_own_indent`
    /// above land on the *same* final answer as this test for an entirely
    /// different reason -- neither test tells the two rules apart, which is
    /// this task's own original mistake and why the corrected rule below has
    /// its own dedicated tests.
    #[test]
    fn iterate_wrong_kind_through_a_transparent_unlabelled_block_reports_full_lexical_depth() {
        let mut interp = Interp::new(false);
        run_source(&mut interp, b"do label x\ndo\niterate x\nend\nend").unwrap_err();
        let FailureSite { indent, .. } = interp.failure_site.expect("a site was resolved");
        assert_eq!(indent, 4, "two DO frames deep, matching the oracle");
    }

    /// **The corrected 28.x indent rule, all fourteen points**: nine from
    /// the reviewer's own probe (`p1`/`p2`/`p5`/`p8`/`p9`/`p10`/`p11`/`p12`/
    /// `p13`, their own naming, kept so the report's cross-reference to
    /// this table still resolves) plus five re-measured independently by
    /// this task against the oracle before touching any code (`n1`-`n3`
    /// entirely new shapes, `p1`/`p11` re-run to confirm the two that
    /// already matched still do). Every row was captured with `cat -A`
    /// against `build/bin/rexx`, byte for byte, not inferred.
    ///
    /// **The rule the whole table obeys**: start at the `LEAVE`/`ITERATE`'s
    /// own full lexical depth; walk outward; every `SELECT` (always) or
    /// `DO`/`LOOP` (unless an unlabelled `Simple` block) that is examined
    /// and does *not* match resets the residual to *its own*
    /// `static_indent`; a match stops the walk without resetting anything
    /// itself; report whatever the residual is at that point. `p11`/`p1`
    /// are the two rows where the very first frame examined is the match,
    /// so nothing ever resets and the reported value is the origin's own
    /// unmodified full depth -- the case the original, wrong rule
    /// generalised from.
    #[test]
    fn the_corrected_28x_indent_rule_matches_all_fourteen_probed_shapes() {
        for (name, source, expect) in [
            ("p5", &b"if 1=1 then leave"[..], 4),
            ("p9", &b"if 1=1 then do\nleave\nend"[..], 6),
            ("p12", &b"do\nleave\nend"[..], 2),
            ("p8", &b"if 1=1 then do i=1 to 3\nleave zz\nend"[..], 4),
            (
                "p2",
                &b"do label x\nselect\nwhen 1=1 then iterate x\notherwise nop\nend\nend"[..],
                2,
            ),
            (
                "p10",
                &b"do label x\nselect\nwhen 1=1 then do\niterate x\nend\notherwise nop\nend\nend"[..],
                2,
            ),
            (
                "p13",
                &b"do label x\ndo label y\niterate x\nend\nend"[..],
                2,
            ),
            (
                "p1",
                &b"do label x\nif 1=1 then do\niterate x\nend\nend"[..],
                8,
            ),
            (
                "p11",
                &b"select label s\nwhen 1=1 then iterate s\notherwise nop\nend"[..],
                6,
            ),
            // Independently added by this task, not in the reviewer's own
            // table: a bare LEAVE reaching only an unlabelled SELECT
            // (SELECT owns a frame unconditionally, even unlabelled, so
            // the pop still happens and still resets to its own indent).
            (
                "n1",
                &b"select\nwhen 1=1 then leave\notherwise nop\nend"[..],
                0,
            ),
            // Three real loops nested three deep, none matching: each pop
            // resets in turn, the outermost (top level) wins.
            (
                "n2",
                &b"do i=1 to 3\ndo j=1 to 3\ndo k=1 to 3\nleave zz\nend\nend\nend"[..],
                0,
            ),
            // A named ITERATE crossing one unlabelled real loop and one
            // unlabelled SELECT, matching neither: both pop, the outer
            // loop's own indent (0, top level) is what survives.
            (
                "n3",
                &b"do i=1 to 3\nselect\nwhen 1=1 then iterate zz\notherwise nop\nend\nend"[..],
                0,
            ),
        ] {
            let mut interp = Interp::new(false);
            run_source(&mut interp, source).unwrap_err();
            let FailureSite { indent, .. } = interp.failure_site.expect("a site was resolved");
            assert_eq!(indent, expect, "{name}: {source:?}");
        }
    }

    /// **The `run_bounded` `Goto`-absorption trap, named in `Flow::Leave`'s
    /// own doc comment.** A `DO` with an `ITERATE` in its body, nested
    /// inside an `IF`'s `THEN`, run enough times that a version which
    /// implemented repetition as a `Goto` back to the loop's own top
    /// (rather than an internal `run_repeating` loop that never returns
    /// mid-construct) would have that `Goto`'s target absorbed by the
    /// enclosing `IF`'s own `run_bounded` -- which owns the whole `THEN`
    /// range the `DO` sits inside -- and re-enter the `DO`'s own arm as a
    /// fresh first pass, with its running total reset to `Simple`/`Count`'s
    /// own initial state every time. That bug's own observable signature
    /// is `n` staying stuck at whatever the first `ITERATE`d pass produced,
    /// forever, or (depending on exactly how the re-entry is shaped)
    /// hanging outright, rather than the small, exact total this test
    /// asserts.
    #[test]
    fn leave_and_iterate_survive_a_do_nested_in_an_ifs_then_iterating_repeatedly() {
        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(
                &mut interp,
                b"n = 0\nif 1 = 1 then do i = 1 to 5\nif i // 2 = 0 then iterate\nn = n + i\nend\nsay n"
            ),
            b"9\n".to_vec(),
            "1 + 3 + 5, the odd i's only -- a Goto-based re-entry would not \
             accumulate this total across the DO's own five iterations"
        );
    }

    // ---- Task 11's own indentation quantity ----

    #[test]
    fn one_two_and_three_enclosing_dos_indent_by_two_four_and_six() {
        for (source, spaces) in [
            (&b"do i = 1 to 3\nsay 1/0\nend"[..], 2),
            (&b"do i = 1 to 3\ndo j = 1 to 3\nsay 1/0\nend\nend"[..], 4),
            (
                &b"do i = 1 to 3\ndo j = 1 to 3\ndo k = 1 to 3\nsay 1/0\nend\nend\nend"[..],
                6,
            ),
        ] {
            let mut interp = Interp::new(false);
            run_source(&mut interp, source).unwrap_err();
            let FailureSite { indent, .. } = interp.failure_site.expect("a site was resolved");
            assert_eq!(indent, spaces, "{source:?}");
        }
    }

    /// **The test the coordinator's own design review asked for by name**:
    /// a raise *after* a loop has already exited, at a shallower lexical
    /// depth than the loop's own body -- exactly the shape a live,
    /// imperfectly-unwound `Interp` counter would over-indent and a purely
    /// static function of `(instructions, index)` cannot, because there is
    /// no state left over from the loop's own three completed iterations
    /// for anything to have failed to unwind.
    #[test]
    fn the_indent_after_a_loop_has_already_exited_is_not_left_over_from_it() {
        let mut interp = Interp::new(false);
        run_source(&mut interp, b"do i = 1 to 3\nsay i\nend\nsay 1/0").unwrap_err();
        let FailureSite { indent, text, .. } = interp.failure_site.expect("a site was resolved");
        assert_eq!(
            indent, 0,
            "top level, after the loop, not the loop's own two"
        );
        assert_eq!(text, b"say 1/0".to_vec());
    }

    /// `DO`'s own control-setup expressions (`TO`/`BY`/`FOR`/the repeat
    /// count/`OVER`'s target) are evaluated **before** the loop's own body
    /// is entered, and are unindented even though the `DO` clause they
    /// belong to sits at the same lexical position the body's own two
    /// spaces would apply to -- the case the brief's own bullet 4 names
    /// (`do i = 1 to 'x'` gets none), re-measured across the sibling forms
    /// the brief did not enumerate.
    #[test]
    fn control_setup_expressions_are_unindented_unlike_the_loop_body_they_precede() {
        for source in [
            &b"do i = 1 to 'x'\nsay 1\nend"[..],
            &b"do 1/0\nsay 1\nend"[..],
            &b"do i = 1 to 3 for 1/0\nsay 1\nend"[..],
            &b"do i = 1 to 3 by 1/0\nsay 1\nend"[..],
        ] {
            let mut interp = Interp::new(false);
            run_source(&mut interp, source).unwrap_err();
            let FailureSite { indent, .. } = interp.failure_site.expect("a site was resolved");
            assert_eq!(indent, 0, "{source:?}");
        }
    }

    /// `WHILE`/`UNTIL` are tested **inside** the loop's own frame, unlike
    /// the header's control-setup expressions -- measured, `do while 1/0`
    /// is indented two at top level, not zero.
    #[test]
    fn while_and_until_are_indented_inside_the_loops_own_frame() {
        let mut interp = Interp::new(false);
        run_source(&mut interp, b"do while 1/0\nsay 1\nend").unwrap_err();
        let FailureSite { indent, .. } = interp.failure_site.expect("a site was resolved");
        assert_eq!(indent, 2);

        let mut interp = Interp::new(false);
        run_source(&mut interp, b"do until 1/0\nsay 1\nend").unwrap_err();
        let FailureSite { indent, .. } = interp.failure_site.expect("a site was resolved");
        assert_eq!(indent, 2);
    }

    /// A `SELECT`'s own scan (evaluating each `WHEN`'s condition) is
    /// indented two, present even before any `WHEN` matches -- the finding
    /// this task made that the brief did not state (measured: `select /
    /// when 1/0 then nop / end` indents the failing `WHEN`'s own condition
    /// by two, not zero).
    #[test]
    fn a_whens_own_condition_is_indented_at_the_selects_own_two_spaces() {
        let mut interp = Interp::new(false);
        run_source(&mut interp, b"select\nwhen 1/0 then nop\nend").unwrap_err();
        let FailureSite { indent, .. } = interp.failure_site.expect("a site was resolved");
        assert_eq!(indent, 2);
    }

    /// A matched `WHEN`'s own `THEN` body indents six (`SELECT`'s two, plus
    /// the `WHEN`-`THEN` shape's own four, the same as an `IF`'s matched
    /// branch); `OTHERWISE`'s own body indents only four, **not** six --
    /// the second finding this task made that the brief did not state,
    /// because `OTHERWISE` is not built from the same double-frame `IF`-
    /// shaped machinery a `WHEN`'s `THEN` is.
    #[test]
    fn a_matched_whens_then_body_indents_six_but_otherwises_body_indents_only_four() {
        let mut interp = Interp::new(false);
        run_source(&mut interp, b"select\nwhen 1 = 1 then say 1/0\nend").unwrap_err();
        let FailureSite { indent, .. } = interp.failure_site.expect("a site was resolved");
        assert_eq!(indent, 6);

        let mut interp = Interp::new(false);
        run_source(
            &mut interp,
            b"select\nwhen 1 = 0 then nop\notherwise\nsay 1/0\nend",
        )
        .unwrap_err();
        let FailureSite { indent, .. } = interp.failure_site.expect("a site was resolved");
        assert_eq!(
            indent, 4,
            "OTHERWISE's own body, one frame, not the WHEN-THEN shape's two"
        );
    }

    /// An `IF`'s matched branch is four spaces, for **both** `THEN` and a
    /// plain `ELSE` (not only the "else if" chain shape the brief's own
    /// example used) -- and an "else if" chain nests two such branches,
    /// giving eight.
    #[test]
    fn an_ifs_matched_then_or_else_branch_indents_four_and_an_else_if_chain_indents_eight() {
        let mut interp = Interp::new(false);
        run_source(&mut interp, b"if 1 = 1 then say 1/0").unwrap_err();
        let FailureSite { indent, .. } = interp.failure_site.expect("a site was resolved");
        assert_eq!(indent, 4, "THEN");

        let mut interp = Interp::new(false);
        run_source(&mut interp, b"if 1 = 0 then say 2\nelse say 1/0").unwrap_err();
        let FailureSite { indent, .. } = interp.failure_site.expect("a site was resolved");
        assert_eq!(indent, 4, "a plain ELSE, not part of an else-if chain");

        let mut interp = Interp::new(false);
        run_source(
            &mut interp,
            b"if 1 = 0 then say 2\nelse if 1 = 1 then say 1/0",
        )
        .unwrap_err();
        let FailureSite { indent, .. } = interp.failure_site.expect("a site was resolved");
        assert_eq!(
            indent, 8,
            "the outer ELSE's own four, plus the inner IF's THEN's own four"
        );
    }

    /// Task 13's own four `static_indent` fixes, found while building
    /// `TRACE`'s clause echo -- **not a second computation of the
    /// indentation quantity**, a bug in the existing one, for a case
    /// Task 10/11 structurally could not exercise: none of `THEN`/`ELSE`/
    /// `OTHERWISE`/a `WHEN`'s own `THEN` carries an expression, so none of
    /// them can ever raise a condition and become a `FailureSite` -- the
    /// only way anything before this task ever asked `static_indent` a
    /// question. `TRACE` echoes every stepped instruction, markers
    /// included, which is what finally asks.
    ///
    /// Each expected number is the oracle's, read with `cat -A` against
    /// `build/bin/rexx` under `trace r` (the report has the full
    /// transcripts): a marker clause sits at exactly half the indent its
    /// own body gets, all the way down through nesting -- confirmed by
    /// `ThenInstruction.cpp`/`ElseInstruction.cpp`'s own `execute`
    /// (`indent(); trace; indent();`, so the marker traces after the first
    /// bump and the body after the second) and by `OtherwiseInstruction.cpp`
    /// (`trace; indent();`, one bump, so `OTHERWISE`'s own clause sits at
    /// the `SELECT`'s scan level, the same as a `WHEN`'s own condition).
    ///
    /// This calls `static_indent` directly rather than through a raise,
    /// because a marker clause cannot raise -- there is no `FailureSite` to
    /// read one back from.
    #[test]
    fn a_then_else_when_then_or_otherwise_markers_own_clause_indents_half_its_bodys() {
        // `IF`'s own `THEN`: `then_start` used to fall into the body's `+4`
        // branch (the branch's own recursive call returns 0 for the very
        // first position of its range) and answer 4, not 2.
        let instructions = instructions_of(b"if 1 = 1 then say 'x'");
        let then_start = if_then_start(&instructions, 0);
        assert_eq!(static_indent(&instructions, then_start), 2, "IF's own THEN");

        // `IF`'s own `ELSE`: `target == false_target` used to fall through
        // this whole arm entirely (`pc = else_end; continue`, which passes
        // straight over the marker's own index in the enclosing walk) and
        // answer 0, the enclosing level, not 2.
        let instructions = instructions_of(b"if 1 = 0 then say 'x'\nelse say 'y'");
        let if_index = 0;
        let InstructionKind::If { false_target, .. } = &instructions[if_index].kind else {
            panic!("index 0 is the IF")
        };
        let else_index = false_target.expect("this IF has an ELSE");
        assert_eq!(static_indent(&instructions, else_index), 2, "IF's own ELSE");

        // A `WHEN`'s own `THEN`, sharing `InstructionKind::Then` with `IF`
        // (`instruction.rs`'s `if_instruction` builds both): `target ==
        // body_start` used to match the loop's own `>=` and answer 6, the
        // body's value, not 4.
        let instructions = instructions_of(b"select\nwhen 1 = 1 then say 'x'\nend");
        let when_index = 1;
        let then_start = if_then_start(&instructions, when_index);
        assert_eq!(
            static_indent(&instructions, then_start),
            4,
            "WHEN's own THEN"
        );

        // `OTHERWISE`'s own clause -- **this one used to abort the process**,
        // not merely answer a wrong number: `target == *otherwise_index`
        // matched neither the `whens` loop nor the body check below it, and
        // fell to `unreachable!("a resolved SELECT's own range holds only
        // its WHENs and OTHERWISE")`. Reproduced against the tree before
        // this fix (`cargo test` aborted this exact test with that message,
        // `run.rs`'s panic site named in the fix's own commit) rather than
        // inferred.
        let instructions = instructions_of(b"select\nwhen 1 = 0 then nop\notherwise\nsay 'y'\nend");
        let otherwise_index = instructions
            .iter()
            .find_map(|i| match &i.kind {
                InstructionKind::Select { otherwise, .. } => *otherwise,
                _ => None,
            })
            .expect("this SELECT has an OTHERWISE");
        assert_eq!(
            static_indent(&instructions, otherwise_index),
            2,
            "OTHERWISE's own marker clause"
        );
    }

    /// `if_instruction` (`rexx-parse`'s `instruction.rs`) gives both `IF`
    /// and a `WHEN`'s own `THEN` the identical shape: the `Then` marker
    /// sits immediately after the condition-bearing instruction at
    /// `condition_index`. A free function rather than inlined three times
    /// in the test above, since all three call sites need the exact same
    /// index arithmetic and nothing else about `Instruction` to find it.
    fn if_then_start(instructions: &[Instruction], condition_index: usize) -> usize {
        assert!(
            matches!(
                instructions[condition_index].kind,
                InstructionKind::If { .. }
                    | InstructionKind::When { .. }
                    | InstructionKind::WhenCase { .. }
            ),
            "condition_index must be an IF, a WHEN or a WHEN CASE"
        );
        condition_index + 1
    }

    /// Parses `source` and returns its top-level instruction list, owned --
    /// the marker-index tests above need to read `InstructionKind` fields
    /// directly (`false_target`, `otherwise`) rather than only run the
    /// program to a raise, since a marker clause cannot raise at all.
    fn instructions_of(source: &[u8]) -> Vec<Instruction> {
        parse_program(source.to_vec())
            .expect("test program parses")
            .main
            .instructions
    }

    /// Nesting a `SELECT` (with a matched `WHEN`) inside a `DO`: two plus
    /// two plus four -- confirming the additive model composes across
    /// different construct kinds, not only same-kind nesting.
    #[test]
    fn a_select_nested_inside_a_do_composes_the_two_constructs_own_contributions() {
        let mut interp = Interp::new(false);
        run_source(
            &mut interp,
            b"do i = 1 to 3\nselect\nwhen 1 = 1 then say 1/0\notherwise nop\nend\nend",
        )
        .unwrap_err();
        let FailureSite { indent, .. } = interp.failure_site.expect("a site was resolved");
        assert_eq!(indent, 8);
    }

    // ---- End arm ----

    #[test]
    fn a_do_loops_own_end_is_a_pure_marker_never_independently_dispatched() {
        // Reached at all only if something jumped straight onto it, which
        // nothing in this crate does -- covered indirectly by every DO/LOOP
        // test above completing without the loud failure this arm used to
        // give before Task 11. Direct coverage: a labelled LOOP closes
        // cleanly (EndStyle::LabeledDo is exercised on the same code path
        // EndStyle::Do/Loop are).
        let mut interp = Interp::new(false);
        assert_eq!(
            say_output(&mut interp, b"do label x\nsay 'ok'\nend"),
            b"ok\n".to_vec()
        );
    }
}
