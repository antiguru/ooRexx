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

//! The Phase 4a executor, at the size Task 3's borrow-shape spike needs it.
//!
//! This crate is a spike, and the thing it exists to prove is one sentence
//! from the design's "The borrow shape": **the instruction loop clones the
//! `Rc` into a local on entry, and every `&CodeBody` and `&Expr` derives from
//! that local.** `Interp` owns the heap, the root set, the activation stack,
//! the plan cache and the two sinks, and does not own the AST. `run_activation`
//! (`run.rs`) is where that discipline is written down, together with the
//! version of it that does not compile.
//!
//! **What it executes grows task by task, and this doc deliberately does not
//! list it.** The enumeration that stood here was true of Task 3's spike and
//! false by Task 7, and the two `Loud` messages carried the same list and went
//! stale the same way; `Loud::expression`'s doc records why the cure is
//! deleting the list rather than correcting it. What holds instead: any
//! construct not yet implemented fails loudly with `NOT_IMPLEMENTED_EXIT`
//! rather than silently, which is a gate criterion.
//!
//! The per-concept modules the design's crate layout names have landed beside
//! this file (`value.rs`, `stem.rs`, `plan.rs`, `activation.rs`, `error.rs`,
//! `eval.rs`, `run.rs`). What stays here is the interpreter itself, the
//! loud-failure path, and the entry point and thread setup that exercise the
//! borrow discipline `run.rs` writes down -- `Code<'a>`, the type that
//! discipline is expressed through, stays here too, because every module that
//! evaluates or steps through one needs it equally and none of them is a
//! better owner than the crate root.

use rexx_core::{Heap, ObjRef, RootSet};
use rexx_parse::{
    CodeBody, ExprKind, InstructionKind, ParseError, PrefixOp, Program, SymbolId, SymbolTable,
    parse_program,
};
use std::collections::HashMap;
use std::rc::Rc;

// The value model: `text`/`number`/`to_text`/`to_number` on `Interp`, and the
// two rules D15 exists to enforce (a number's rendering is fixed at creation,
// and a `SmallInt` is admissible only within the DIGITS that produced it).
mod value;

// Stems and compound variables (D15a): tail resolution, the tombstone rule,
// and the "replace the object, mutate a tail in place" split.
mod stem;

// The per-body resolution plan (D16): `Plan`, `BodyKey`, `ProgramId`, the
// plan cache, and the full name-resolution order (plan, then `extra`, then
// growth).
mod plan;
use plan::{BodyKey, Plan, ProgramId};

// One activation: everything about the frame currently executing (D16).
mod activation;
use activation::Activation;

// `Raised` (the payload of a real Rexx condition) and `Failure` (either a
// `Loud` not-implemented marker or a `Raised` condition, the one type
// `step` and everything above it propagate).
mod error;
use error::{ClauseSite, Failure, FailureSite};

// Expression evaluation (`eval`/`eval_node`): terms, arithmetic and
// concatenation.
mod eval;

// The instruction loop (D16's "Control flow"): `Flow`, and `step` and its two
// callers, together with the borrow discipline `run_activation` is written
// down to prove (Task 3's spike). Extended task by task with the branches and
// calls later tasks add; Task 9 is the first to extend it, with the seven
// instructions that do not branch.
mod run;

// `TRACE` (D17): the mode, the nine reachable prefixes' own byte formatting,
// and the classification a `TRACE`/`TRACE VALUE` setting goes through to
// become one. `run.rs`'s `step_in_temps_frame` and its loop drivers, and
// `eval.rs`'s `eval`, own *when* to call into this module; this module owns
// only the bytes.
mod trace;
use trace::TraceMode;

/// The exit code for a construct Phase 4a does not implement.
///
/// It has to sit outside 157..=253, where a Rexx error's `256 - major` lives,
/// or a not-implemented failure is indistinguishable from error 11 and a
/// program *expecting* error 11 would pass. 120 also sits below 126, so it
/// cannot be read as a shell's `128 + signal` encoding either, and it is not
/// 0, 1, 2, 126 or 127.
///
/// **No value in 0..=255 is collision-free, and pretending otherwise is the
/// mistake to avoid here.** A program can name its own exit code, and once
/// Task 9 implements `EXIT` with a result it can name this one: measured,
/// `exit 120` gives rc 120 under the oracle, so a corpus program could produce
/// this code legitimately. What makes the choice safe is not the number, it is
/// **the harness treating this code as a hard failure whatever the oracle
/// did** (criterion 5), rather than comparing it against an expectation. The
/// 157..=253 exclusion is still worth having, because a code inside that band
/// would be wrong in a second and worse way: it would look like a *condition*
/// the interpreter raised, and a program expecting that condition would pass.
///
/// **Task 12 settled this at 120 by leaving it alone**, having built the
/// `256 - major` band it has to avoid, so the spike's choice is now the final
/// one. This constant remains the single place to change it.
pub const NOT_IMPLEMENTED_EXIT: i32 = 120;

/// The interpreter thread's stack, in bytes.
///
/// Chosen from a measurement rather than from taste, and the measurement is in
/// `tests/spike.rs::records_the_stack_cost_of_one_eval_frame`, which is the
/// test Task 11 re-runs when it sets the evaluation-depth limit. D19 requires
/// the limit to be **at least 100,000** (the oracle evaluates a 100,000-term
/// expression and exits 0) and **below what this stack survives**, so this
/// number and the per-frame cost together bracket it from both sides.
///
/// Measured on a 100,000-term left-deep expression, `x86_64-unknown-linux-gnu`,
/// rustc 1.96.1: **784 bytes per `eval` level in a debug build, 192 in
/// release**. Debug is the number that matters, because that is what `cargo
/// test` runs and what the in-process harnesses will therefore sit on. The
/// whole pipeline survives to roughly 685,000 levels here, against a limit
/// that has to be at least 100,000, so there is about six and a half times the
/// headroom a limit at the oracle's own maximum needs.
///
/// The budget this covers is larger than `eval` alone, and all four of its
/// users run on this same thread because the entry point owns everything from
/// `parse_program` onward:
///
/// * `eval` recursing once per term of a left-deep expression,
/// * `Plan::note` recursing over the same expression to assign its slots,
///   which is this crate's own and is the shallowest-per-level of the three
///   at about 160 bytes, but is still a recursion and could be given the same
///   explicit-worklist treatment `rexx-parse`'s walks got,
/// * dropping the AST, which `rexx-parse` now does iteratively. It used to
///   recurse once per `Box<Expr>` level, and was the thing that bound this
///   budget until it was fixed,
/// * and, since Task 10, `step` recursing through `run_bounded` once per
///   source *nesting* level of `IF` or `SELECT` (`run.rs`'s own module doc
///   comment and `run_bounded`'s doc comment have the full argument: a
///   nested `IF`/`SELECT` resolves itself inside its enclosing one's own
///   `step` call, through `step_in_temps_frame`, rather than returning to
///   this thread's outer loop first). Task 11 added `DO`/`LOOP` to this
///   same recursion (`run_loop`/`run_repeating` each drive their own
///   `run_bounded` calls one level deeper, per lexical nesting level, the
///   identical shape `IF`/`SELECT` already had): a nested `DO`, `LOOP` or
///   `SELECT WHEN`/`WHEN CASE` costs a level here exactly like a nested
///   `IF` does. **Unmeasured** -- unlike the other three, nothing
///   generates a program with thousands of *lexically* nested `IF`/
///   `SELECT`/`DO`/`LOOP` clauses the way a left-deep expression generates
///   deep `eval` recursion from one term count, so there is no natural knob
///   to bisect against, and no corpus or real program comes remotely close
///   to needing one: a 2,000-level synthetic nested-`IF` chain ran clean in
///   about 70ms as a sanity check, nothing more precise. Bounded by program
///   *text* rather than by data, so it is not the unbounded case D19's
///   limit exists for and needs no counter of its own -- but the next
///   person to move this figure should know it is a fourth consumer, not
///   only the three above, and that "unmeasured" is the honest state of it
///   rather than a number this comment is confident in.
///
/// **`eval` is the recursion that binds, and the figure to size against is its
/// own 784.** Measured by bisecting `rexx-run` on this stack, debug, to within
/// 2,000 levels, with the phases separated by choosing programs that reach
/// different ones (`exit` first, so the deep expression is parsed and dropped
/// but never evaluated; a bare command clause, which `Plan::build` skips too):
///
/// | what runs | deepest surviving | implied bytes per level |
/// |---|---|---|
/// | parse and drop only | over 4,000,000, no cliff found | under 134 |
/// | parse, plan and drop | 3,354,442, fails at 3,356,347 | about 160 |
/// | all four, including `eval` | 684,618, fails at 686,523 | about 783 |
///
/// The last row is an independent check on the probe inside `eval`, arrived at
/// by a different method entirely, and the two agree to within 0.2 per cent:
/// 783 bisected against 784.0 probed. So **roughly 685,000 levels**, and a
/// limit at the oracle's own maximum of 100,000 has about six and a half times
/// the headroom.
///
/// **This table replaced an earlier one that said the opposite, and the reason
/// is worth keeping.** Before `rexx-parse` made `Expr`'s `Drop`, `block.rs`'s
/// `visit_expr` and the gate walk iterative, the parse-and-drop shape cliffed
/// near 630,000 and `eval` did not move the cliff at all, so the binding
/// recursion was in `rexx-parse` and not here. That measurement was correct
/// when taken and describes a tree that no longer exists. Anyone re-deriving
/// these numbers should re-run the bisection rather than trust the table,
/// including this one.
///
/// The other lesson from that round: quote the bisected cliff, not a per-level
/// number divided out of a coarse bracket. A 100,000-wide bracket produced an
/// apparent 820-versus-860 split that had parse-and-drop costing *less* per
/// level than parse-plan-and-drop, which no model of sequential phases
/// produces, and the incoherence was the tell.
///
/// One consequence that survives the correction: a depth limit on `eval` still
/// **does not close the abort path in general**, because parsing, planning and
/// dropping happen outside any counter this crate owns. It is no longer
/// reachable in practice at these depths, since those phases now cost 160
/// bytes a level and under, but nothing enforces that.
///
/// **Re-measured after Task 7, and it moved: 1600 bytes per `eval` level in
/// debug, roughly double the 784 above.** `eval_node` grew from four match
/// arms to fifteen (`Stem`, `Compound`, `DotVariable`, `Prefix`, the seven
/// arithmetic operators, `Abuttal`/`Blank` beside `||`), and in an
/// unoptimised debug build the compiler does not appear to reuse stack slots
/// across mutually exclusive match arms as aggressively as it does in
/// release, so a dispatch function's own frame grows with how many forms it
/// names, not only with what the one taken arm does -- confirmed by
/// re-running `records_the_stack_cost_of_one_eval_frame` on the unchanged
/// `||`-only stress program, whose own logic did not change. This is
/// `bytes_per_frame`'s probe reading, **not a re-bisection**: the earlier
/// table's own two rows for `eval` (783 bisected, 784.0 probed) agreed to
/// within 0.2%, so the probe is a reasonable stand-in, but confirming that
/// still holds at this size is Task 11's to do when it sets the real depth
/// limit, not assumed here. Survivable depth at the new cost is
/// `512 MiB / 1600 ≈ 335,000` levels, still more than three times D19's
/// 100,000 minimum, so `INTERPRETER_STACK_BYTES` stays unchanged rather than
/// growing to chase a number every later task's new `ExprKind` arms will
/// keep moving. Expect this figure to keep drifting downward as Task 8
/// (comparison, logical) and later tasks add more forms, and re-measure
/// rather than trust it, the same instruction the row above already gives.
///
/// 512 MiB is reserved address space, not resident memory. Linux commits stack
/// pages on first touch, so a program that never recurses pays for the pages it
/// actually uses and not for this number.
///
/// **Re-measured again at Task 11: 1840 bytes per `eval` level in debug, up
/// from the 1600 above.** `eval`'s own D19 depth counter (`MAX_EVAL_DEPTH`,
/// `eval.rs`) adds a handful of bookkeeping bytes to every level, exactly the
/// mechanism the 784-to-1600 move already described -- a dispatch function's
/// frame grows with what it does on every call, not only on the arm actually
/// taken. Method: `cargo test -p rexx-exec --test spike
/// records_the_stack_cost_of_one_eval_frame -- --nocapture`, unchanged from
/// the row above, printing (byte for byte, this run):
///
/// ```text
/// interpreter stack: 536870912 bytes, eval depth reached: 100000, span: 183998160 bytes, per frame: 1840.0 bytes
/// ```
///
/// Survivable depth at this cost: `536,870,912 / 1840 ≈ 291,777` levels,
/// still comfortably over D19's 100,000 floor (about 2.9x headroom, down
/// from Task 7's ~3.35x at 335,000 -- the trend this table exists to show is
/// still shrinking, as expected, and still nowhere near the floor).
/// `INTERPRETER_STACK_BYTES` stays at 512 MiB; this is the fifth value this
/// figure has taken across this phase (820, 850, 783/784, 1600, now 1840),
/// and the point of keeping every row rather than overwriting the last one
/// is that each was correct for the code it measured -- re-measure again
/// rather than trust this one either, the same instruction every prior row
/// already gives.
///
/// **Task 11 set `eval.rs`'s `MAX_EVAL_DEPTH` to 100,000, and that closes off
/// the way every figure on this page was re-derived.** Everything above was
/// measured by letting `eval` recurse far past 100,000 -- the external
/// `rexx-run` bisection to a guard-page abort (684,618 survives, 700,000
/// aborts, rc 134) and this crate's own `records_the_stack_cost_of_one_eval_frame`
/// (`tests/spike.rs`) both depend on that being possible. `run_program` is
/// now the only public entry point that reaches `eval` on a sized stack at
/// all, and `eval` itself refuses anything past `MAX_EVAL_DEPTH`, so neither
/// method can be re-run through it any more: a program built to recurse
/// 700,000 levels now raises 11.1 at 100,001 and never reaches the guard
/// page, and `records_the_stack_cost_of_one_eval_frame`'s own 100,000-term
/// chain is the deepest such a program can now legally go.
///
/// That measurement, at exactly 100,000, is still real and still runs on
/// every `cargo test` -- it is what the two-sided bound above is checked
/// against today, and it needed no external bisection to begin with, only a
/// division. What is gone is the ability to go *past* 100,000 through the
/// public API to independently confirm the extrapolation still holds at
/// higher depths, the way the external bisection to ~685,000 once did.
/// **Whoever next revisits this figure and wants that confirmation has to
/// raise `MAX_EVAL_DEPTH` (or call `eval` directly, bypassing `run_program`,
/// from code temporarily built for the purpose) before bisecting again, and
/// must remember to put it back.** Recorded here rather than only in
/// `eval.rs`, because this constant's own two-sided justification is the
/// thing the change affects, not the counter itself.
pub const INTERPRETER_STACK_BYTES: usize = 512 * 1024 * 1024;

/// What one interpreter run produced.
///
/// `stdout` and `stderr` are the sinks themselves rather than a handle to
/// them, for two reasons that happen to agree. The design wants a test to
/// capture output without a subprocess, and this value is what crosses the
/// `join()` back from the interpreter thread, so it has to be `Send` -- which
/// `Vec<u8>` is and an `Rc`-flavoured sink would not be. D17's "because they
/// are separate descriptors their relative interleaving is not observable" is
/// what makes two independently buffered sinks safe rather than a shortcut.
///
/// The cost, recorded here rather than discovered by Task 14: a program that
/// prints and then runs for a long time buffers all of it instead of
/// streaming. Nothing in 4a's corpus does that, and Phase 7's stream model
/// replaces this whole shape.
#[derive(Debug)]
pub struct Outcome {
    pub exit_code: i32,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    /// What the run cost in stack. Task 11 reads this to set the
    /// evaluation-depth limit; see `StackSpan`.
    pub stack: StackSpan,
    /// How many times `Heap::collect` ran during this program. Always `0`
    /// under `run_program`, which does not enable Task 16's stress mode and
    /// nothing else in this crate calls
    /// `collect` at all -- a non-zero value here is only ever possible
    /// through `run_program_collect_every_alloc`, and criterion 4's gate
    /// asserts it is non-zero specifically so a mode that silently
    /// collected nothing cannot pass by being indistinguishable from one
    /// that collected correctly.
    pub collections: u64,
}

/// How deep evaluation went and how much stack it took to get there.
///
/// Measured from inside `eval` itself rather than from a replica of it, by
/// taking the address of a local at the first level and at the deepest one.
/// Both ends are inside the same function, so the fixed cost of the frames
/// *above* `eval` cancels and what is left is the per-level cost.
///
/// **Both ends come from the same call chain, and that is load-bearing rather
/// than incidental.** A program evaluates many separate expressions, each its
/// own chain from depth 1, and the frames above `eval` are not the same height
/// for all of them: a fragment's `eval` runs under `run_fragment` under `step`
/// under the enclosing `eval`, thousands of bytes deeper than a top-level one.
/// An earlier version rewrote the depth-1 address on every depth-1 entry while
/// recording the deepest only on a new maximum, so the two ends could come
/// from different chains and the "frames above cancel" argument silently
/// stopped holding. Measured: a 1000-term expression followed by `interpret
/// "say 'b'"` reported **782.16** bytes per level against the true 784.0,
/// because the fragment's shallow evaluation sits deeper and shortened the
/// span. The error ran in the **unsafe** direction, since a smaller
/// bytes-per-level implies more survivable levels than there are, and this
/// value is public and its doc tells Task 11 to size a limit from it. So the
/// depth-1 address is now held aside and copied in at the moment the maximum
/// is beaten, which pins both ends to one chain by construction.
///
/// The probe itself perturbs the frame it measures, by the width of the local
/// whose address it takes. That biases the answer upward by a few bytes per
/// level, which is the safe direction for sizing a stack.
#[derive(Copy, Clone, Debug, Default)]
pub struct StackSpan {
    /// The deepest `eval` recursion the run reached. Zero if it never
    /// evaluated an expression at all.
    pub max_depth: usize,
    /// Stack bytes between the first `eval` level and the deepest one, both
    /// on the chain that reached `max_depth`. Meaningless unless `max_depth`
    /// is above 1, which is what `bytes_per_frame` checks before dividing.
    pub bytes: usize,
}

impl StackSpan {
    /// Stack bytes one further level of `eval` costs, or `None` when the run
    /// never recursed and there is nothing to divide by.
    ///
    /// The divisor is `max_depth - 1` and not `max_depth`: `bytes` spans the
    /// gap *between* level 1 and the deepest level, so it counts one fewer
    /// step than there are levels.
    pub fn bytes_per_frame(self) -> Option<f64> {
        (self.max_depth > 1).then(|| self.bytes as f64 / (self.max_depth - 1) as f64)
    }
}

/// A construct 4a does not implement, on its way to becoming an exit code and
/// a line on stderr.
///
/// Not a Rexx condition and never convertible into one: the whole point of
/// `NOT_IMPLEMENTED_EXIT` is that an implementation gap cannot produce a
/// passing differential test. Task 12 gives the real errors their own type,
/// `Raised`, which is a different thing entirely.
#[derive(Debug)]
struct Loud {
    message: String,
}

impl Loud {
    /// An instruction 4a does not execute. `keyword()` is `None` for the four
    /// clause shapes no keyword introduces, and their names come from the
    /// shape rather than from a keyword table.
    ///
    /// Two properties of the match below, both deliberate and both easy to
    /// undo by accident.
    ///
    /// It is **exhaustive with no `_` arm**, so adding an `InstructionKind`
    /// variant is a compile error here rather than a silent fallthrough. That
    /// is the same rule `form_name` follows, and it is why 36 variants are
    /// listed by name to reach one shared expression.
    ///
    /// It **cannot panic**. An earlier version ended `_ => unreachable!(…)`,
    /// which was true of the tree as it stood and broke the failing-loudly
    /// rule anyway: a new keywordless variant would have aborted the process
    /// instead of producing `NOT_IMPLEMENTED_EXIT` and a message naming the
    /// construct, and an abort is precisely the outcome that rule exists to
    /// exclude. The fallback is a string, not a panic, and the exhaustive
    /// match is what stops it ever being reached.
    fn instruction(kind: &InstructionKind) -> Loud {
        let name = match kind {
            // The four clause shapes no keyword introduces.
            InstructionKind::Assignment { .. } => "an assignment",
            InstructionKind::Label { .. } => "a label",
            InstructionKind::Message { .. } => "a message send",
            InstructionKind::Command { .. } => "a command",
            // Everything else is named by the keyword that introduced it.
            InstructionKind::Address { .. }
            | InstructionKind::Arg { .. }
            | InstructionKind::Call { .. }
            | InstructionKind::Do { .. }
            | InstructionKind::Drop { .. }
            | InstructionKind::Else { .. }
            | InstructionKind::End { .. }
            | InstructionKind::Exit { .. }
            | InstructionKind::Expose { .. }
            | InstructionKind::Forward { .. }
            | InstructionKind::Guard { .. }
            | InstructionKind::If { .. }
            | InstructionKind::Interpret { .. }
            | InstructionKind::Iterate { .. }
            | InstructionKind::Leave { .. }
            | InstructionKind::Loop { .. }
            | InstructionKind::Nop
            | InstructionKind::Numeric { .. }
            | InstructionKind::Options { .. }
            | InstructionKind::Otherwise
            | InstructionKind::Parse { .. }
            | InstructionKind::Procedure { .. }
            | InstructionKind::Pull { .. }
            | InstructionKind::Push { .. }
            | InstructionKind::Queue { .. }
            | InstructionKind::Raise { .. }
            | InstructionKind::Reply { .. }
            | InstructionKind::Return { .. }
            | InstructionKind::Say { .. }
            | InstructionKind::Select { .. }
            | InstructionKind::Signal { .. }
            | InstructionKind::Then
            | InstructionKind::Trace { .. }
            | InstructionKind::Use { .. }
            | InstructionKind::When { .. }
            | InstructionKind::WhenCase { .. } => kind.keyword().unwrap_or("an instruction"),
        };
        Loud {
            message: owned_message(name, instruction_owner(kind)),
        }
    }

    /// An expression form 4a does not evaluate.
    ///
    /// Names the **form** and never formats the node. An earlier version wrote
    /// `{kind:?}` and produced 364 KB of stderr for one clause of
    /// `corpus/lang/deep_nested_expr.rex`, because `ExprKind`'s derived `Debug`
    /// walks the whole tree and a tree is unbounded. Failing loudly is a gate
    /// criterion and every later task inherits this path, so the size of the
    /// message is part of the contract: the variant name is what a reader
    /// needs, and the differential harness has to compare whatever is emitted
    /// byte for byte.
    ///
    /// **Neither message lists what *is* implemented**, and both used to. The
    /// list was true of Task 3's spike and false by Task 7, which is the whole
    /// argument: every task that implements a form has to remember to edit a
    /// string in a file it is not otherwise touching, and none of the three
    /// that shipped between did. A message that can only go stale by being
    /// wrong about its own subject cannot rot this way, so the enumeration is
    /// gone rather than corrected.
    fn expression(kind: &ExprKind) -> Loud {
        Loud {
            message: owned_message(&form_name(kind), expr_owner(kind)),
        }
    }

    /// A fragment that did not parse.
    ///
    /// Stays loud for the reason `execute`'s parse arm spells out: Task 12's
    /// catalogue reports conditions a *running* program raises, and a syntax
    /// error supplies neither a `Raised` nor a clause that became an
    /// `Instruction`.
    ///
    /// **Reachable through `run_program` since 4b's Task 1, where before it
    /// needed the deleted spike entry point.** So this is now a live
    /// divergence rather than a latent one, and it is a real one: measured,
    /// `interpret "do forever then"` gives the oracle 27.901 at rc 229 with
    /// a two-line clause echo, and gives this `rexx-exec: INTERPRET text did
    /// not parse: 27.901: Invalid DO or LOOP syntax.` at rc 120. Loud rather
    /// than silent, which is what criterion 5 requires of it, and not
    /// byte-identical, which no parse error in this crate is (`execute`'s own
    /// parse arm: "wrong in the details on purpose and right in never being
    /// mistaken for success"). Closing it needs a `ParseError`-to-`Raised`
    /// conversion, which is the same machinery a *top-level* syntax error
    /// wants and should be built once for both, not here for one caller.
    fn parse(error: &ParseError) -> Loud {
        Loud {
            message: format!("INTERPRET text did not parse: {error}"),
        }
    }
}

/// Names an expression form in **bounded** text, for a loud failure to quote.
///
/// The bound is the whole point and it is a contract, not a preference: this is
/// called on nodes 4a cannot evaluate, and those nodes carry arbitrarily large
/// subtrees. Everything returned here is either a `&'static str` or one
/// `Operator::spelling`, which is also `&'static`, so no input can make the
/// answer long. **Never format a node into a message.**
///
/// The match is exhaustive with no `_` arm on purpose, so a new `ExprKind`
/// variant is a compile error here rather than a silent "unknown".
fn form_name(kind: &ExprKind) -> String {
    let name = match kind {
        ExprKind::Literal(_) => "a literal",
        ExprKind::Constant(_) => "a constant symbol",
        ExprKind::Variable(_) => "a simple variable",
        ExprKind::Stem(_) => "a stem",
        ExprKind::Compound(_) => "a compound variable",
        ExprKind::DotVariable(_) => "an environment symbol",
        // The two operator forms name the operator, because "a dyadic
        // operator is not implemented" does not tell a reader which one to
        // go and implement. Both spellings are `&'static`.
        ExprKind::Prefix { op, .. } => {
            return format!(
                "the prefix operator `{}`",
                match op {
                    PrefixOp::Plus => "+",
                    PrefixOp::Minus => "-",
                    PrefixOp::Not => "\\",
                }
            );
        }
        ExprKind::Binary { op, .. } => {
            return format!("the operator `{}`", op.spelling());
        }
        ExprKind::Call { .. } => "a function call",
        ExprKind::QualifiedCall { .. } => "a namespace-qualified call",
        ExprKind::ClassResolver { .. } => "a namespace-qualified class lookup",
        ExprKind::Message { .. } => "a message send",
        ExprKind::List(_) => "a parenthesised list",
        ExprKind::Logical(_) => "a comma list in a condition",
        ExprKind::VariableReference(_) => "a variable reference",
    };
    name.to_string()
}

/// Task 0's Step 3: appends the owner phase to a loud message, `"{name} is
/// not implemented ({owner})"`, or leaves it exactly as it was before this
/// task (`"{name} is not implemented"`) when [`instruction_owner`]/
/// [`expr_owner`] answer `None` -- meaning "this crate already implements
/// that variant", not "the owner is some particular phase".
///
/// **Review finding I6.** An earlier version keyed this on the literal
/// `"4a"`, which doubles a phase *name* as a sentinel for a different
/// property ("implemented here"), and that conflation is a forward trap:
/// the moment Task 3 implements `InstructionKind::Call`, the owner table
/// would have to either mislabel it `"4a"` (false -- 4b implemented it) or
/// leave it `"4b"` and let some other, still-unimplemented 4b-local site
/// print `rexx-exec: CALL is not implemented (4b)` for a *different*
/// reason -- the exact self-contradiction this carve-out exists to
/// prevent, with no carve-out left to catch it. `Option<&'static str>`
/// says what is actually meant and needs no phase name to mean it, so it
/// survives every later phase unchanged.
///
/// `None` is reachable here only through two documented edge cases in
/// `run.rs` (`run_loop`'s `DO`/`LOOP` COUNTER/`DO WITH` check, and its
/// stem-target `DO OVER` deviation) where the outer `InstructionKind`/
/// `ExprKind` is 4a's own but the specific reason that call happened is
/// not. Printing an owner there would read as self-contradictory -- the
/// construct plainly *is* implemented -- so this leaves the message
/// exactly as it was before this task on that path, and is the only
/// reason this function exists rather than a bare `format!` at each of the
/// two call sites. `run.rs`'s `do_with_takes_the_loud_path`,
/// `do_counter_takes_the_loud_path_regardless_of_which_other_kind_it_rides_on`,
/// `do_over_a_stem_target_takes_the_loud_path` and
/// `do_over_a_parenthesised_stem_target_is_also_caught` each assert the
/// exact unsuffixed message (I1) -- before those assertions existed,
/// deleting this carve-out left the whole suite green.
fn owned_message(name: &str, owner: Option<&'static str>) -> String {
    match owner {
        None => format!("{name} is not implemented"),
        Some(owner) => format!("{name} is not implemented ({owner})"),
    }
}

/// Who is responsible for an `InstructionKind` that is not (yet) 4a's own,
/// spelled exactly as the split table spells it
/// (`docs/superpowers/specs/2026-07-30-phase-4a-executor-design.md`, "The
/// split") -- `None` for a variant 4a already implements (see
/// [`owned_message`]'s doc for why that is `None` and not a `"4a"` string).
///
/// **A third copy of `tests/owners.rs`'s `INSTRUCTION_TAGS`, unavoidably**:
/// production code cannot depend on anything under `tests/`, so the two
/// cannot be merged the way `coverage.rs` and `loud.rs` were (Task 0's Step
/// 1). Any variant that moves in scope, or changes owner, has to be edited
/// in both places -- `owners.rs`'s own module doc names this function as
/// the fifth of its five pinned items for exactly that reason, and
/// `loud.rs`'s `every_out_of_scope_variant_fails_loudly` is what would
/// catch the two drifting apart: it asserts the emitted stderr contains
/// each witness's own declared owner.
///
/// **Arm-grained for `InstructionKind::Call`, matching `loud.rs`'s own
/// witness table (Step 2)**: every arm of `rexx_parse::Call` is `"4b"`
/// except `Call::Qualified`, which is genuinely Phase 5's (a namespace-
/// qualified `CALL`, mirroring `ExprKind::QualifiedCall`'s own ownership
/// below). Every other variant here stays coarse -- in particular
/// `InstructionKind::Signal` is `"4b"` regardless of arm, because `Signal::
/// Trap` is 4b's own too (Task 7), just a later task within it than
/// `Signal::Value`/`Label` (Task 6) -- so no nested match is needed there
/// the way `Call` needs one.
///
/// Exhaustive with no `_` arm, matching `Loud::instruction`'s own match: a
/// new `InstructionKind` variant is a compile error here, not a silent
/// omission from the loud message's owner.
fn instruction_owner(kind: &InstructionKind) -> Option<&'static str> {
    match kind {
        InstructionKind::Assignment { .. }
        | InstructionKind::Label { .. }
        | InstructionKind::Do(_)
        | InstructionKind::Loop(_)
        | InstructionKind::If { .. }
        | InstructionKind::Then
        | InstructionKind::Else { .. }
        | InstructionKind::Select { .. }
        | InstructionKind::When { .. }
        | InstructionKind::WhenCase { .. }
        | InstructionKind::Otherwise
        | InstructionKind::Leave { .. }
        | InstructionKind::Iterate { .. }
        | InstructionKind::End { .. }
        | InstructionKind::Drop { .. }
        | InstructionKind::Say { .. }
        | InstructionKind::Exit { .. }
        | InstructionKind::Numeric { .. }
        | InstructionKind::Trace(_)
        // In scope since 4b's Task 1: the fragment machinery was 4a's and the
        // keyword is this task's, so `Interpret` is `None` here (implemented
        // in this crate) rather than `Some("4b")`.
        | InstructionKind::Interpret { .. }
        | InstructionKind::Nop => None,
        InstructionKind::Call(call) => Some(match &**call {
            rexx_parse::Call::Named { .. }
            | rexx_parse::Call::Dynamic { .. }
            | rexx_parse::Call::Trap(_) => "4b",
            rexx_parse::Call::Qualified { .. } => "Phase 5",
        }),
        InstructionKind::Return { .. }
        | InstructionKind::Procedure { .. }
        | InstructionKind::Use(_)
        | InstructionKind::Signal(_)
        | InstructionKind::Raise(_)
        | InstructionKind::Push { .. }
        | InstructionKind::Queue { .. } => Some("4b"),
        InstructionKind::Parse(_)
        | InstructionKind::Arg(_)
        | InstructionKind::Pull(_)
        | InstructionKind::Address(_) => Some("4c"),
        InstructionKind::Expose { .. }
        | InstructionKind::Options { .. }
        | InstructionKind::Message { .. }
        | InstructionKind::Guard(_)
        | InstructionKind::Reply { .. }
        | InstructionKind::Forward(_) => Some("Phase 5"),
        InstructionKind::Command { .. } => Some("Phase 7"),
    }
}

/// [`instruction_owner`]'s counterpart for `ExprKind`. See that function's
/// own doc for why this is a third copy of `owners.rs`'s `EXPR_TAGS` (there,
/// `EXPR_TAGS`), and for the completeness guarantee the exhaustive match
/// below carries.
fn expr_owner(kind: &ExprKind) -> Option<&'static str> {
    match kind {
        ExprKind::Literal(_)
        | ExprKind::Constant(_)
        | ExprKind::Variable(_)
        | ExprKind::Stem(_)
        | ExprKind::Compound(_)
        | ExprKind::DotVariable(_)
        | ExprKind::Prefix { .. }
        | ExprKind::Binary { .. }
        | ExprKind::Logical(_) => None,
        ExprKind::Call { .. } | ExprKind::VariableReference(_) => Some("4b"),
        ExprKind::QualifiedCall { .. }
        | ExprKind::ClassResolver { .. }
        | ExprKind::Message { .. }
        | ExprKind::List(_) => Some("Phase 5"),
    }
}

/// The code a step is executing, all of it borrowed from the caller's local
/// `Rc` and none of it from `self`.
///
/// This is the design's `fn eval(&mut self, body: &CodeBody, expr: &Expr)`
/// with the two things a body needs beyond its instructions folded in: the
/// symbol table a `SymbolId` is meaningless without, and the slot each symbol
/// resolved to. Bundling them keeps the lifetime argument in one place --
/// every field of a `Code<'a>` outlives `&mut self` because `'a` is a local's,
/// so a `&[u8]` name pulled out of `symbols` survives a `&mut self` call that
/// a `&self.…` name would not.
struct Code<'a> {
    body: &'a CodeBody,
    symbols: &'a SymbolTable,
    /// Slot per `SymbolId` **of this code's own table**. For a fragment those
    /// ids are the fragment's, resolved against the enclosing frame, which is
    /// why this is a field of `Code` rather than something read back off the
    /// activation.
    slots: &'a HashMap<SymbolId, usize>,
}

/// Whether a variable read found a value or derived one from the name.
///
/// D16 requires the read path to answer this from the start rather than gain
/// it later: `SIGNAL ON NOVALUE` in 4b changes what an uninitialised read
/// does, and retrofitting a raise into the hottest path is what naming it here
/// prevents. The spike reads the flag and does nothing with it, which is the
/// correct amount of nothing.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum Novalue {
    Set,
    Unset,
}

/// The interpreter. Owns the heap, the root set, the activation stack, the
/// plan cache and the two sinks, and **does not own the AST**.
struct Interp {
    heap: Heap,
    roots: RootSet,
    activations: Vec<Activation>,
    /// Every program the loader has issued an id for, indexed by that id.
    ///
    /// This is what makes a `ProgramId` a durable identity rather than a
    /// number that outlives its program: a plan cached under
    /// `BodyKey { program: ProgramId(0), .. }` stays correct because
    /// `ProgramId(0)`'s program is still here.
    programs: Vec<Rc<Program>>,
    plans: HashMap<BodyKey, Rc<Plan>>,
    /// The output sink. `SAY` writes here and `Outcome::stdout` is what it
    /// becomes.
    out: Vec<u8>,
    /// The trace sink, which becomes `Outcome::stderr`.
    ///
    /// It exists because the design puts both sinks on `Interp` and D17 makes
    /// them separate for a measured reason: with `trace r` the `*-*` and `>>>`
    /// lines are on stderr while `SAY` is on stdout, and being separate
    /// descriptors is what makes their relative interleaving unobservable and
    /// two independently buffered sinks safe. Task 13 was the first to write
    /// to it (`trace.rs` and a dozen call sites in `run.rs` now do); keeping
    /// the field from the start meant the loud-failure path already appended
    /// to the right buffer rather than being rerouted later, which is when a
    /// stray ordering difference would have appeared.
    trace: Vec<u8>,
    /// The current `TRACE` setting's visible-output shape (D17). Lives on
    /// `Interp` rather than per-`Activation`'s `Settings`, unlike the design's
    /// own stated rule that `TRACE` "behaves the same way" as `Settings` and
    /// is restored across a call -- **a deliberate 4a-only simplification,
    /// not a rediscovery of that rule**: 4a has exactly one frame (the design
    /// says so in the same breath, "which is exactly why this must be written
    /// down now rather than discovered by 4b"), so there is no call for a
    /// callee's `TRACE OFF` to fail to survive past, and putting this on
    /// `Activation` today would be the same throwaway-scaffolding shape the
    /// `eval_str` correction and Task 6's `Vec<Block>` deferral both ruled
    /// out -- `Activation` already carries its own `Settings` for exactly
    /// this per-frame inheritance, and 4b's `CALL` is what makes a second
    /// frame exist for `TRACE` to need the same treatment. The move this
    /// field still owes is onto `Activation`, deleting it from here, and it
    /// is the task that lands `CALL` that owes it -- **not** 4b's Task 1,
    /// which was the other half of this note when the `interpret_spike`
    /// field sat just below it, and which introduces no second frame:
    /// `INTERPRET` runs its fragment inside the creating activation.
    trace_mode: TraceMode,
    /// The indent (Task 11's `static_indent` quantity, spaces already
    /// doubled) an intermediate value line traces at right now -- the one
    /// piece of state `eval`'s own single insertion point needs that
    /// `eval`'s signature does not otherwise carry, mirroring the oracle's
    /// own `settings.traceIndent` (a persistent field every `traceValue`/
    /// `traceVariable`/... call reads, never threaded as a parameter
    /// through `evaluate`). Set once per traced clause, at whichever call
    /// site already computed that clause's own indent for the `*-*` echo
    /// or a `>K>` line -- `run.rs`'s own doc comments name each site.
    ///
    /// **Why a field and not an `eval` parameter.** Threading an indent
    /// through `eval`/`eval_node`'s entire recursive call graph would touch
    /// every arm in `eval.rs`, exactly the "eighteen arms" retrofit the
    /// design's own withdrawn note wrongly predicted for the value events
    /// themselves -- reading the oracle's own field-not-parameter design
    /// avoids inventing that threading here instead.
    current_value_indent: usize,
    /// **F3, found by review.** The innermost `SELECT CASE`'s own evaluated
    /// `case` text, or `None` inside a plain `SELECT` (or before any
    /// `SELECT`/`SELECT CASE` has run at all) -- the one piece of state an
    /// **absorbed** `WhenCase` needs that nothing else threads to it: a
    /// *listed* `WhenCase` gets `case_text` handed to it directly by
    /// `Select`'s own explicit arm (`run.rs`), but an absorbed one (a
    /// `WhenCase` reached only through ordinary `step_in_temps_frame`
    /// stepping, because it is itself the `THEN` consequence of a
    /// preceding `WHEN`/`WHEN CASE`, `ast.rs`'s own doc comment on
    /// `whens`) has no such hand-off -- it is stepped like any other
    /// instruction, with nothing carrying its enclosing `SELECT CASE`'s
    /// own comparison value along.
    ///
    /// Set by `Select`'s own arm, unconditionally, every time (mirroring
    /// `current_value_indent`'s own field-not-parameter shape) -- **not
    /// saved and restored across a nested `SELECT`/`SELECT CASE`**, which
    /// is a real, narrow, disclosed limitation: an absorbed `WhenCase`
    /// belonging to an *outer* `SELECT CASE`, reached only *after* a
    /// *nested* `SELECT CASE` inside the same outer body has already run
    /// and overwritten this field, would read the nested one's `case`
    /// text instead of its own. No corpus or spec example nests `SELECT
    /// CASE` around an absorbed `WHEN CASE` this way; `run.rs`'s own
    /// `WhenCase` arm names the same limitation again at its own read
    /// site.
    current_case_text: Option<Vec<u8>>,
    /// **F3's own perimeter, found by review -- and corrected twice more,
    /// each correction found by re-verifying the previous one rather than
    /// trusting it.** When an absorbed `WhenCase` (`run.rs`'s own doc
    /// comment on that arm) takes its `Flow::Goto(false_target)` branch,
    /// whatever it lands on -- `END`'s own 7.3, or (F-EX1) `OTHERWISE`'s
    /// own marker *and its whole body*, redirected through `run_
    /// otherwise` -- reports every indent it computes **`self` spaces
    /// higher** than its own ordinary `static_indent` would give, for as
    /// long as this stays non-zero.
    ///
    /// **The value is the constant `4`, always -- not a function of the
    /// absorbed condition's own depth, which the field's second version
    /// wrongly used.** That second version (`current_value_indent - 2`,
    /// an *additive* offset rather than the first version's absolute
    /// replacement) was right at the top level (`6 - 2 = 4`) and wrong one
    /// `DO` deeper (`8 - 2 = 6`, where the oracle still wants `4`) --
    /// caught only because F-EX1's own fix was re-verified at a second
    /// nesting depth rather than trusted from the first. The real
    /// invariant: the absorbed condition always sits exactly two
    /// `indent()` bumps past an *ordinary* `SELECT`-level construct's own
    /// position -- the enclosing, listed `WHEN`/`WHEN CASE`'s own marker,
    /// then its own body entry -- and that gap (`4` spaces) does not grow
    /// with how many other constructs enclose the whole `SELECT`, because
    /// both the absorbed condition's own depth *and* `END`'s/`OTHERWISE`'s
    /// own ordinary depth grow by the identical amount together. Measured
    /// at two nesting depths for all three landing shapes before trusting
    /// it a second time (this task's report has the full transcripts):
    ///
    /// | landing shape | top-level ordinary / actual | one `DO` deeper |
    /// |---|---|---|
    /// | `END`'s own 7.3 | `0` / `4` | `2` / `6` |
    /// | `OTHERWISE`'s own marker | `2` / `6` | `4` / `8` |
    /// | `OTHERWISE`'s own body | `4` / `8` | `6` / `10` |
    ///
    /// every row's own `actual - ordinary` is `4`. `4` is the identical
    /// "marker is half its body" arithmetic `static_indent`'s own doc
    /// comment already states for `THEN`/`ELSE`/`OTHERWISE`, so the
    /// *number* is still the one rule this task keeps reusing; it is
    /// simply a fixed constant here, not `current_value_indent`-derived.
    ///
    /// **Why a field, not `Flow::Goto` growing a payload.** `Flow::Goto`
    /// is the ordinary resume mechanism *every* `If`/`Select`/`Do` match
    /// uses (`Ok(Flow::Goto(resume))`, dozens of sites), none of which has
    /// any residual indent to carry -- only this one escape does. Giving
    /// every one of those sites an indent to thread, to serve the single
    /// site that needs one, is the restructuring the coordinator asked to
    /// be told about rather than done; this field is the same shape
    /// `current_value_indent`/`current_case_text` already are, applied to
    /// a third, narrower quantity, not a new idiom.
    ///
    /// **Why persistent rather than consumed after one step, unlike the
    /// field's own first version.** `OTHERWISE`'s own body can be more
    /// than one clause (`say 'O'` *and* `leave s`, in the measured case,
    /// both needing the offset), so a `.take()` at the top of the very
    /// next `step_in_temps_frame` call -- right for `END`'s own one-clause
    /// landing -- would have zeroed it before the second body clause ever
    /// read it. `0` in the overwhelmingly common case (nothing escaping
    /// right now); set by the absorbed `WhenCase`'s own false branch,
    /// added (never replacing) inside `step_in_temps_frame`'s own indent
    /// computation on every step while non-zero, and explicitly restored
    /// to `0` by `run_otherwise` once its own `run_bounded` call returns --
    /// the one place that knows the elevated dispatch is now over. The
    /// `END`-only landing needs no explicit restore: 7.3 is fatal, so
    /// nothing runs afterward to see a stale value (`execute`, `lib.rs`,
    /// gives every run a fresh `Interp`).
    ///
    /// **Narrower than the general case, disclosed rather than chased
    /// further under this task's own time budget.** Only `step_in_temps_
    /// frame`'s own indent computation and `run_otherwise`'s own explicit
    /// marker computation add this offset; the `WHEN`-scan's and `WHILE`/
    /// `UNTIL`'s own explicit `current_value_indent` overrides do not, so a
    /// `SELECT`/`DO`/`WHILE` *nested inside* an escaped `OTHERWISE`'S own
    /// body would not have every one of *its own* explicit-override sites
    /// inherit the elevation, only whatever reaches `step_in_temps_frame`
    /// ordinarily. No corpus or spec example nests this deeply; this is
    /// the same class of disclosure as `current_case_text`'s own nested-
    /// clobber limitation just above, not a silently assumed correctness
    /// claim.
    indent_offset: usize,
    /// The clause a `Raised` condition escaped from, as the 1-based line and
    /// the bytes `TRACE` would echo, or `None` if nothing raised.
    ///
    /// Resolved here rather than carried on `Raised`, and the reason is that
    /// only an instruction loop knows **which source** an
    /// `Instruction::clause_span` indexes into: the main loop's spans are the
    /// program's, a fragment's are its own. Storing a bare span would leave
    /// `execute` guessing between them.
    ///
    /// Written once, by the first loop to see the failure escape, and read by
    /// `execute` after `run` has already popped the activation the site came
    /// from. That teardown is why the site cannot simply be reconstructed at
    /// the top: by then the frame is gone.
    failure_site: Option<FailureSite>,
    /// Task 16's collect-on-every-allocation gate criterion (4a exit gate,
    /// criterion 4): when true, [`Interp::alloc_with`] calls `Heap::collect`
    /// after every allocation instead of never. Off by default, and the off
    /// path is untouched by this field's existence -- `alloc_with` reads it
    /// once, in an `if`, and does nothing else differently; nothing upstream
    /// of that one check changed at all. Named for what it does rather than
    /// for the criterion, since a later, permanent collector would want the
    /// same flag and should not have to rename it away from a gate task's
    /// number.
    stress_collect: bool,
    /// Current `eval` recursion depth, and the deepest it has reached.
    ///
    /// Task 11 turns `depth` into D19's guard by comparing it against a limit
    /// and raising 11.1. Here it only feeds the measurement, because the limit
    /// is set from numbers this spike is what produces.
    depth: usize,
    max_depth: usize,
    /// The depth-1 address of the chain currently being evaluated, kept aside
    /// until that chain turns out to be the deepest one.
    ///
    /// Scratch, not a result. It is overwritten by every new top-level
    /// evaluation, which is exactly why it is not `stack_first`: see
    /// `StackSpan` for the measurement that showed what happens when the two
    /// ends of the span come from different chains.
    stack_entry: usize,
    /// The two ends of the span, both from the chain that reached
    /// `max_depth`, written together so they can never disagree.
    ///
    /// Zero before anything is measured. `stack_span` subtracts one from the
    /// other saturatingly, so the unmeasured state answers zero rather than
    /// needing a sentinel to test for.
    stack_first: usize,
    stack_deepest: usize,
}

impl Interp {
    /// `stress_collect` always starts `false` here rather than as a second
    /// parameter: `Interp::new` has well over a hundred callers, almost all
    /// of them unit tests in files this task is not permitted to touch
    /// (`run.rs`, `eval.rs`, `plan.rs`, `trace.rs`), so widening its
    /// signature would force edits far outside this task's granted scope
    /// for a flag only two callers (`execute`, below) ever need to set.
    /// [`Interp::stress_collect`] flips it after construction instead,
    /// which is exactly as inert for every existing caller as adding a
    /// field with a fixed default already is.
    ///
    /// **This took an `interpret_spike: bool` until 4b's Task 1**, and the
    /// hundred-plus callers that argument's removal touched are the direct
    /// cost the paragraph above was weighing. `INTERPRET` is implemented
    /// now, so there is no mode left to select between: every caller passed
    /// `false` except the two spike tests, and all of them now say
    /// `Interp::new()`.
    fn new() -> Interp {
        Interp {
            heap: Heap::new(),
            roots: RootSet::new(),
            activations: Vec::new(),
            programs: Vec::new(),
            plans: HashMap::new(),
            out: Vec::new(),
            trace: Vec::new(),
            trace_mode: TraceMode::OFF,
            current_value_indent: 0,
            current_case_text: None,
            indent_offset: 0,
            failure_site: None,
            stress_collect: false,
            depth: 0,
            max_depth: 0,
            stack_entry: 0,
            stack_first: 0,
            stack_deepest: 0,
        }
    }

    // ---- loading and the activation stack ----

    /// Loads `program`, runs its main body in a fresh activation, and tears
    /// the activation down again.
    fn run(&mut self, program: Program) -> Result<Option<ObjRef>, Failure> {
        let program = Rc::new(program);
        let id = ProgramId(self.programs.len());
        self.programs.push(Rc::clone(&program));

        // Note what does *not* happen here: the plan is looked up through
        // `&program.main`, a borrow of the local `Rc`, while `self` is
        // borrowed mutably by `plan_for`. Reaching the same body through
        // `self.programs[id.0].main` instead would be the `E0502` that
        // `run_activation` writes out.
        let plan = self.plan_for(
            BodyKey {
                program: id,
                directive: None,
            },
            &program.main,
            &program.symbols,
        );

        let frame = self.roots.push_slots(plan.len());
        self.activations
            .push(Activation::new(Rc::clone(&program), plan, frame));

        let exit = self.run_activation();

        // Popped whether or not the body raised, so the root set is left the
        // way it was found even on the failure path.
        let activation = self.activations.pop().expect("the frame just pushed");
        self.roots.pop_slots(activation.frame);
        exit
    }

    // `plan_for` and `activation`/`activation_mut` live in `plan.rs`/
    // `activation.rs` (Task 6), beside the types they operate on.

    // `fragment_plan` and `slot_of` live in `plan.rs` (Task 6), beside
    // `Plan` itself.

    /// Reads a variable, by the slot the plan already resolved its id to.
    ///
    /// Falls back to `slot_of` when the plan never saw the id, which the
    /// spike's non-exhaustive `Plan::build` can produce and which Task 6's
    /// exhaustive pass should not. The fallback is not dead weight even then:
    /// it is the same path a name bound at run time takes.
    fn read(&mut self, code: &Code<'_>, id: SymbolId) -> (ObjRef, Novalue) {
        let slot = match code.slots.get(&id) {
            Some(slot) => *slot,
            None => self.slot_of(code.symbols.name(id).as_bytes()),
        };
        let frame = self.activation().frame;
        match self.roots.slot(frame, slot) {
            Some(value) => (value, Novalue::Set),
            // An uninitialised read yields the derived name, which for a
            // simple variable is its own upcased spelling.
            None => {
                let derived = code.symbols.name(id).as_bytes();
                (self.text(derived), Novalue::Unset)
            }
        }
    }

    /// Converts `EXIT`'s result into the raw exit code, before `rexx-run`'s
    /// own 8-bit truncation (`bin/rexx-run.rs`) narrows it to a process exit
    /// status.
    ///
    /// `None` -- a bare `EXIT`, or falling off the end of the body -- is 0,
    /// matching the oracle. `Some(value)` needs `value` to be a whole number
    /// that fits a signed 32-bit integer (`Numerics::objectToSignedInteger`'s
    /// own bound, `INT32_MIN..=INT32_MAX`, both inclusive); anything else --
    /// fractional, non-numeric, or simply too wide -- leaves the exit code at
    /// 0, which is where it already sits on every path 4a can reach. Measured:
    /// `exit 5.9` and `exit 'abc'` and `exit 2147483648` (one past
    /// `INT32_MAX`) all give rc 0.
    ///
    /// **Not a fixed-width check on its own -- it inherits one for free from
    /// D15's own rule that a number's precision is fixed at creation.** A
    /// bare literal like `exit 2147483647` never passes through arithmetic, so
    /// `to_number` hands back the exact value with nothing rounded, and the
    /// only bound left is the `i32` one. A value built by arithmetic -- even
    /// `EXIT`'s own unary minus, `-2147483647` -- was already rounded to the
    /// *active* `NUMERIC DIGITS` (9 by default) the moment it was created
    /// (`eval_prefix`, `eval.rs`), and this function never re-rounds it. That
    /// is the entire explanation for an asymmetry that looks, at first, like
    /// a sign bug: measured, `exit 2147483647` gives rc 255 while `exit
    /// -2147483647` gives rc 0, because the second is `0 - 2147483647`
    /// rounded to 9 digits at creation (`2147483650`, one past `INT32_MAX`),
    /// not because negative values are bounded differently. Raising the
    /// active DIGITS before the subtraction removes the rounding and the
    /// asymmetry with it: measured, `numeric digits 20; exit -2147483647`
    /// gives rc 1, `-2147483647 mod 256`.
    ///
    /// `rexx_num::ARGUMENT_DIGITS` (18) is `whole_value`'s own precision
    /// argument here, deliberately not the activation's current `NUMERIC
    /// DIGITS`: the oracle's own conversion (`NumberString::int64Value`) uses
    /// a fixed width of its own, independent of the setting in force --
    /// measured, `numeric digits 3; exit 2147483647` still gives rc 255. Any
    /// width at least the ten digits `INT32_MAX` needs gives the identical
    /// answer here, since a value wide enough to need rounding at 18 digits is
    /// already wide enough to fail the `i32` bound regardless of how it was
    /// rounded; 18 is used rather than invented because it is already
    /// `rexx-num`'s own public constant for exactly this kind of
    /// current-DIGITS-independent conversion (`::OPTIONS DIGITS`'s own reason
    /// for reaching for it).
    fn exit_code_for(&mut self, value: Option<ObjRef>) -> i32 {
        let Some(value) = value else { return 0 };
        let Ok(number) = self.to_number(value) else {
            return 0;
        };
        match number.whole_value(rexx_num::ARGUMENT_DIGITS) {
            Some(whole) => i32::try_from(whole).unwrap_or(0),
            None => 0,
        }
    }

    /// Turns on Task 16's collect-on-every-allocation stress mode. Only
    /// `execute`'s `collect_every_alloc` arm calls this, right after
    /// construction and before `run`; nothing else needs to flip it, and
    /// nothing can un-flip it once a run has started.
    fn enable_stress_collect(&mut self) {
        self.stress_collect = true;
    }

    /// The one allocation entry point every value/stem constructor in this
    /// crate goes through, so that Task 16's stress mode has exactly one
    /// place to hook rather than one per call site.
    ///
    /// **Off by default, and provably inert when off**: with
    /// `stress_collect` false (the constructed default, and the only value
    /// `run_program` ever leaves it at), this
    /// is `self.heap.alloc_with_uncollected(behaviour, body)` and nothing
    /// else -- one call, one `if` that does not take its branch, no new
    /// allocation, no new borrow of `self.roots`. Every existing caller
    /// (`value.rs`'s `text`/`number`, `stem.rs`'s two stem constructors) was
    /// renamed from `self.heap.alloc_with` to `self.alloc_with` with no
    /// other change at the call site, so the behaviour those four sites saw
    /// before this task is exactly what they see now with the mode off.
    ///
    /// **On, it is `Heap::collect(&self.roots)` followed by
    /// `Heap::alloc_with_uncollected`.** `self.heap` and `self.roots` are
    /// sibling fields, so this borrows each independently and needs no
    /// interior mutability or unsafe cell to call one method with a
    /// borrow of the other in scope.
    ///
    /// **Collecting BEFORE the allocation, not after, and this was not the
    /// first thing tried.** An earlier version of this method collected
    /// *after* allocating, on the reasoning that a fresh object is the one
    /// thing this mode should be checking. That reasoning is backwards: the
    /// caller has not had a chance to root the value this call is about to
    /// return -- `self.text(bytes)` cannot call `push_temp` on its own
    /// result before handing it back -- so a collect fused into the
    /// allocation that produced it can only ever find it unreached and
    /// sweep it, on every single allocation, unconditionally. Measured: with
    /// that order, `run_program_collect_every_alloc("say 1")` panicked
    /// (`value.rs`'s `to_text`, "a live value") and so did all 29 of
    /// `phase-4a.txt`'s programs, including the ones with no rooting
    /// question at stake at all -- a mode that fails everything tests
    /// nothing, the same shape `/bin/true` failed criterion 6 for. Collecting
    /// *before* asks the right question instead: is everything the caller
    /// already holds -- rooted by an *earlier* call's `push_temp`, which by
    /// now has had every chance to run -- still reachable at the moment a
    /// *new* allocation is requested. `eval_arithmetic`'s own shape
    /// (`eval.rs`) is what makes this the faithful test: `push_temp
    /// (left_value)` runs immediately after `left_value` is produced and
    /// strictly before `right_value`'s own evaluation can allocate anything,
    /// so by the time this method's pre-allocation collect runs for
    /// `right_value`'s own allocation, `left_value` has already had its
    /// chance to be rooted -- and the negative control below is what
    /// confirms the mode actually notices when that chance was skipped.
    pub(crate) fn alloc_with(
        &mut self,
        behaviour: rexx_core::BehaviourId,
        body: rexx_core::Body,
    ) -> ObjRef {
        if self.stress_collect {
            self.heap.collect(&self.roots);
        }
        self.heap.alloc_with_uncollected(behaviour, body)
    }

    // ---- values ----
    //
    // `text`, `number`, `to_text` and `to_number` live in `value.rs` (Task 4),
    // as `impl Interp` methods in a sibling module rather than here: `Interp`
    // and its fields are defined in this module (the crate root), and a
    // private item is visible to its defining module's descendants, so
    // `value.rs` reaches `self.heap` directly with no `pub(crate)` needed on
    // the fields themselves. The methods are marked `pub(crate)` there
    // because visibility does not run the other way -- this module could not
    // otherwise call them.

    // `eval`/`eval_node`/`stack_span` live in `eval.rs` (Task 7), beside the
    // operators they evaluate. `depth`/`max_depth`/`stack_entry`/
    // `stack_first`/`stack_deepest` stay here, on `Interp`'s own struct
    // definition, exactly like every other field a sibling module's
    // `impl Interp` block reaches into.
}

// ---- the public entry point ----

/// Runs a Rexx program and returns what it produced.
///
/// **This entry point owns everything from `parse_program` onward** and does
/// it on a thread with an explicitly sized stack (D19). Not the `rexx-run`
/// binary: the L0 harness and the assertion-table harness both run in process,
/// and a `cargo test` thread's default stack is far smaller than the one the
/// depth limit is calibrated against, so putting the thread in the binary
/// alone would leave every in-process caller on exactly the cliff the depth
/// policy exists to keep them off.
///
/// The parse is inside the thread and not outside it because `Rc<Program>` is
/// `!Send`: a program parsed on the caller's thread cannot be handed across.
/// That is a compile error on day one rather than a subtle bug, and it is why
/// `text` crosses as `Vec<u8>` and an `Outcome` comes back.
///
/// `path` is the program's location **as the oracle prints it**, which is the
/// absolute, dot-normalised form: measured, running `./sub/../sub/rel.rex` and
/// `rel.rex` from two different working directories both report the same
/// canonical path. It is a parameter rather than something derived from `text`
/// because a program's own bytes cannot know where they came from, and it is
/// separate from `ProgramSource` because the parser has no use for it. The
/// caller that read the file is the one that knows; `rexx-run` canonicalises
/// before calling.
pub fn run_program(path: &str, text: Vec<u8>) -> Outcome {
    let path = path.to_string();
    on_interpreter_thread(move || execute(&path, text, false))
}

/// `run_program`, except that `Heap::collect` runs after every allocation
/// instead of never. The 4a exit gate's criterion 4: the named L0 subset
/// has to pass again under this mode, with the mode proved to have actually
/// collected (`Outcome::collections` non-zero) rather than merely having
/// been requested.
///
/// `#[doc(hidden)]` because it is `pub` only so `tests/` can reach it, not a
/// second front-door choice beside `run_program`. (Task 3's own
/// `run_program_interpret_spike` carried the identical note until 4b's Task 1
/// deleted it, `INTERPRET` being implemented; this is now the crate's only
/// hidden entry point.)
///
/// **This mode was built at Task 16 gate time, not during the phase, and
/// this is the first time anything has run the L0 subset under it.** The
/// design spec's criterion 4 asked to run the subset under collect-on-every-
/// allocation as though the mode already existed; it did not -- `alloc_with`
/// never collected and `Heap::collect` had no caller outside `rexx-core`'s
/// own tests. So a pass here is real evidence about this run, gathered for
/// the first time on the day it was gathered, not a re-confirmation of
/// something exercised throughout the phase. Say that plainly wherever this
/// criterion's result is reported, rather than letting a passing gate imply
/// otherwise.
#[doc(hidden)]
pub fn run_program_collect_every_alloc(path: &str, text: Vec<u8>) -> Outcome {
    let path = path.to_string();
    on_interpreter_thread(move || execute(&path, text, true))
}

/// Runs `body` on a thread with `INTERPRETER_STACK_BYTES` of stack.
///
/// A panic on that thread is resumed on the caller's rather than converted
/// into an `Outcome`: a panic is a bug in this crate, not a Rexx condition,
/// and swallowing it into an exit code would make it indistinguishable from a
/// construct that failed loudly on purpose.
///
/// A *stack overflow* is the one failure this cannot report. Rust's guard page
/// prints "has overflowed its stack" and aborts the process, which is exactly
/// the silent death D19's depth limit exists to prevent, and the reason the
/// limit has to be below what this stack survives rather than merely large.
fn on_interpreter_thread(body: impl FnOnce() -> Outcome + Send + 'static) -> Outcome {
    let interpreter = std::thread::Builder::new()
        .name("rexx-interp".to_string())
        .stack_size(INTERPRETER_STACK_BYTES)
        .spawn(body)
        .expect("spawning the interpreter thread");
    match interpreter.join() {
        Ok(outcome) => outcome,
        Err(panic) => std::panic::resume_unwind(panic),
    }
}

/// Everything that happens on the interpreter thread: parse, run, report.
fn execute(path: &str, text: Vec<u8>, collect_every_alloc: bool) -> Outcome {
    let program = match parse_program(text) {
        Ok(program) => program,
        // **A parse failure stays loud, and Task 12 did not change that.** It
        // built the catalogue and the three-line format for conditions a
        // *running* program raises, which is what the arms below now use. A
        // syntax error needs two things that path does not supply: the major
        // and sub extracted from a `ParseError` rather than from a `Raised`,
        // and a clause echo for a clause that by definition did not parse into
        // an `Instruction` with a `clause_span`. Parse errors are also
        // deliberately not reproduced byte for byte here (the number and sub
        // match on a plausible line; message text and substitutions are not
        // gated), so this arm is wrong in the details on purpose and right in
        // never being mistaken for success.
        Err(error) => {
            return Outcome {
                exit_code: NOT_IMPLEMENTED_EXIT,
                stdout: Vec::new(),
                stderr: format!("rexx-exec: {error}\n").into_bytes(),
                stack: StackSpan::default(),
                collections: 0,
            };
        }
    };

    let mut interp = Interp::new();
    if collect_every_alloc {
        interp.enable_stress_collect();
    }
    let result = interp.run(program);
    let stack = interp.stack_span();
    let collections = interp.heap.collections_performed();
    let failure_site = interp.failure_site.take();
    // `exit_code_for` needs `&mut interp` (`to_number` fills a lazy cache),
    // so this has to run before `interp.trace`/`interp.out` move out of
    // `interp` below -- a partial move of one field ends `interp`'s usability
    // as a whole value, and every other call above this one only reads or
    // takes a single field, never the whole struct.
    let exit_code = match result {
        Ok(value) => interp.exit_code_for(value),
        Err(Failure::Loud(loud)) => {
            interp
                .trace
                .extend_from_slice(format!("rexx-exec: {}\n", loud.message).as_bytes());
            NOT_IMPLEMENTED_EXIT
        }
        Err(Failure::Raised(raised)) => {
            // `run_activation` records the site on the way out. `None` here
            // would mean a condition escaped without passing an instruction
            // loop, which nothing in 4a can do; it renders visibly rather than
            // panicking, on the error path's standing rule that a reportable
            // condition must never become a crash.
            let failure_site = failure_site.unwrap_or_else(|| FailureSite {
                line: 0,
                text: b"<no failing clause recorded>".to_vec(),
                indent: 0,
            });
            let site = ClauseSite {
                path,
                line: failure_site.line,
                text: &failure_site.text,
                indent: failure_site.indent,
            };
            interp.trace.extend_from_slice(&raised.report(&site));
            raised.exit_code()
        }
    };

    Outcome {
        exit_code,
        stdout: interp.out,
        stderr: interp.trace,
        stack,
        collections,
    }
}

#[cfg(test)]
mod tests {
    use super::{form_name, run_program};
    use rexx_parse::{Expr, ExprKind, Operator, PrefixOp};

    /// The path these tests report programs under.
    ///
    /// They build programs from bytes rather than from files, so there is no
    /// real path here to canonicalise, and nothing below reads it back: it
    /// reaches output only through a raised condition's middle line, which
    /// none of these programs produces. `tests/spike.rs`'s own copy of this
    /// constant carries the fuller note, beside the test that does assert it.
    const TEST_PATH: &str = "/nonexistent/lib-test-program.rex";

    fn literal() -> Expr {
        Expr::new(ExprKind::Literal(Box::from(&b"1"[..])), 0..1)
    }

    fn nest(depth: usize) -> Expr {
        let mut node = literal();
        for _ in 0..depth {
            node = Expr::new(
                ExprKind::Binary {
                    op: Operator::Plus,
                    left: Box::new(node),
                    right: Box::new(literal()),
                },
                0..1,
            );
        }
        node
    }

    /// `Loud::expression`'s size contract, tested on the two arms that can
    /// break it.
    ///
    /// Thirteen of `form_name`'s fifteen arms return a `&'static str` and
    /// cannot grow with anything. `Prefix` and `Binary` are the two that call
    /// `format!`, so they are the two where a regression to `{kind:?}` -- the
    /// 364 KB stderr the doc on `Loud::expression` records -- could actually
    /// land. Both are checked here against a subtree two hundred levels deep,
    /// which is the direct form of the property: the message is a function of
    /// the operator alone, and the children are not read.
    ///
    /// **This lives here rather than in `tests/spike.rs` because the runtime
    /// test cannot reach these two arms for long.** A test that observes a
    /// loud failure needs a form the executor does not evaluate, and every
    /// operator, prefix and dyadic, is implemented within Phase 4a -- the
    /// spike's witness had to move from `+` to `=` when Task 7 landed and
    /// would have moved again for Task 8. Calling `form_name` directly needs
    /// no unimplemented form at all, so nothing a later task does can take
    /// this coverage away.
    #[test]
    fn the_two_formatting_arms_do_not_grow_with_the_subtree() {
        let deep = nest(200);
        let shallow = nest(1);
        assert_eq!(form_name(&deep.kind), form_name(&shallow.kind));
        assert_eq!(form_name(&deep.kind), "the operator `+`");

        let deep = Expr::new(
            ExprKind::Prefix {
                op: PrefixOp::Minus,
                operand: Box::new(nest(200)),
            },
            0..1,
        );
        let shallow = Expr::new(
            ExprKind::Prefix {
                op: PrefixOp::Minus,
                operand: Box::new(literal()),
            },
            0..1,
        );
        assert_eq!(form_name(&deep.kind), form_name(&shallow.kind));
        assert_eq!(form_name(&deep.kind), "the prefix operator `-`");
    }

    // ---- the fragment's lifetime (moved here from `tests/spike.rs`, I7) ----
    //
    // These three arrived with Task 3's borrow-shape spike and ran through
    // `run_program_interpret_spike`, a `pub` entry point that existed for one
    // reason: `INTERPRET` was not implemented, so an *integration* test could
    // not reach a fragment at all through the public API. That entry point's
    // own doc comment named the trade it was making -- a `#[cfg(test)] mod
    // tests` in this file could prove the same lifetime with no public
    // surface -- and asked 4b to re-make it rather than inherit it.
    //
    // Re-made here, and the argument that once favoured the integration test
    // now settles it the other way. That argument was "a unit test with
    // privileged access to private internals proves less about the shape
    // callers actually get". These tests need no privileged access: they call
    // `run_program`, the same public entry point on the same sized thread
    // that `tests/spike.rs` would use, because `INTERPRET` is implemented and
    // a fragment is reachable through the front door. So the public spike
    // surface is deleted and nothing about what these tests exercise changed
    // -- only which file they live in, and that they no longer need a second
    // `pub fn` to exist.

    /// Step 4's test, and the one property `INTERPRET` has that no other
    /// instruction does: a name bound inside fragment text outlives the
    /// fragment, so a *later, separate* fragment reads it back.
    ///
    /// Measured on the oracle in 4a: the binding outlives the fragment.
    #[test]
    fn interpret_binds_a_name_the_enclosing_body_never_mentions() {
        let outcome = run_program(
            TEST_PATH,
            b"interpret \"zork = 42\"\ninterpret \"say zork\"\n".to_vec(),
        );
        assert_eq!(outcome.exit_code, 0, "stderr: {:?}", outcome.stderr);
        assert_eq!(outcome.stdout, b"42\n");
    }

    /// Step 2, and the reason the spike exists in the shape it does.
    ///
    /// Three separate things are being asserted by one transcript, and they are
    /// listed here because a single `assert_eq!` hides which one broke:
    ///
    /// 1. A fragment created mid-instruction **reads** a name the enclosing body
    ///    bound (`zzz`).
    /// 2. A fragment **introduces** a name that appears in no instruction of the
    ///    enclosing body (`zork`), and a *later, separate* fragment reads it back.
    ///    This is the case that forces the enclosing activation to own a mutable
    ///    name-to-slot map, because the enclosing plan is an `Rc` and cannot be
    ///    extended.
    /// 3. The enclosing body carries on afterwards with its own slots intact, and
    ///    a third fragment sees the updated value.
    ///
    /// Oracle, verbatim:
    ///
    /// ```text
    /// zzz = 'from the enclosing frame'
    /// interpret "say zzz"
    /// interpret "zork = 42"
    /// interpret "say zork"
    /// zzz = zzz || '!'
    /// interpret "say zzz"
    /// ```
    ///
    /// ```text
    /// from the enclosing frame
    /// 42
    /// from the enclosing frame!
    /// ```
    ///
    /// rc 0.
    #[test]
    fn a_fragment_shares_the_enclosing_frames_variable_pool() {
        let program = b"zzz = 'from the enclosing frame'\n\
                        interpret \"say zzz\"\n\
                        interpret \"zork = 42\"\n\
                        interpret \"say zork\"\n\
                        zzz = zzz || '!'\n\
                        interpret \"say zzz\"\n";
        let outcome = run_program(TEST_PATH, program.to_vec());
        assert_eq!(outcome.exit_code, 0, "stderr: {:?}", outcome.stderr);
        assert_eq!(
            outcome.stdout,
            b"from the enclosing frame\n42\nfrom the enclosing frame!\n"
        );
    }

    /// `EXIT` inside a fragment ends the *program*, not the fragment, so control
    /// leaves the nested loop and the enclosing one together and both `Rc` locals
    /// drop in order.
    ///
    /// Oracle, verbatim:
    ///
    /// ```text
    /// say 'before'
    /// interpret "say 'inside'"
    /// interpret "exit"
    /// say 'after'
    /// ```
    ///
    /// gives `before\ninside\n`, rc 0. `after` is not printed.
    #[test]
    fn an_exit_inside_a_fragment_ends_the_program() {
        let program = b"say 'before'\n\
                        interpret \"say 'inside'\"\n\
                        interpret \"exit\"\n\
                        say 'after'\n";
        let outcome = run_program(TEST_PATH, program.to_vec());
        assert_eq!(outcome.exit_code, 0, "stderr: {:?}", outcome.stderr);
        assert_eq!(outcome.stdout, b"before\ninside\n");
    }

    /// The reported span comes from one call chain, so what else the program
    /// evaluated cannot change it.
    ///
    /// A regression test for a defect the review found by measuring rather than by
    /// reading. `eval` records a depth-1 address and a deepest address; if the
    /// first is rewritten by *every* top-level evaluation while the second moves
    /// only on a new maximum, the two can end up describing different chains, and
    /// the frames above `eval` then no longer cancel. A fragment's evaluation runs
    /// under `run_fragment` under `step` under the enclosing `eval`, about 2 KB
    /// deeper than a top-level one, so appending one `INTERPRET` to a program was
    /// enough: measured before the fix, this pair reported **784.0** and
    /// **782.158**, and the second is the dangerous direction, since a smaller
    /// per-level cost implies more survivable levels than there are.
    ///
    /// The assertion is equality of the two spans rather than a bound on either,
    /// because the property is "the span does not depend on what else ran" and a
    /// bound would pass for both the fixed and the broken version.
    #[test]
    fn the_stack_span_does_not_depend_on_what_else_the_program_evaluated() {
        let mut alone = b"say 'a'".to_vec();
        for _ in 1..1_000 {
            alone.extend_from_slice(b"||''");
        }
        alone.push(b'\n');

        let mut then_a_fragment = alone.clone();
        then_a_fragment.extend_from_slice(b"interpret \"say 'b'\"\n");

        let alone = run_program(TEST_PATH, alone);
        let then_a_fragment = run_program(TEST_PATH, then_a_fragment);

        assert_eq!(alone.exit_code, 0, "stderr: {:?}", alone.stderr);
        assert_eq!(
            then_a_fragment.exit_code, 0,
            "stderr: {:?}",
            then_a_fragment.stderr
        );
        assert_eq!(
            alone.stack.max_depth, then_a_fragment.stack.max_depth,
            "the fragment's own evaluation is shallow, so it must not move the maximum"
        );
        assert_eq!(
            alone.stack.bytes, then_a_fragment.stack.bytes,
            "the span must come from the chain that reached the maximum, not from the last \
             top-level evaluation to start"
        );
    }
}
