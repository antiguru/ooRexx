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
    CodeBody, ExprKind, InstructionKind, PrefixOp, Program, SymbolId, SymbolTable, parse_program,
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
use run::Ended;

// `TRACE` (D17): the mode, the nine reachable prefixes' own byte formatting,
// and the classification a `TRACE`/`TRACE VALUE` setting goes through to
// become one. `run.rs`'s `step_in_temps_frame` and its loop drivers, and
// `eval.rs`'s `eval`, own *when* to call into this module; this module owns
// only the bytes.
mod trace;

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

    /// A named call this crate resolved to nothing it implements: not an
    /// internal label of the calling body, so the next steps are the builtin
    /// table and then external resolution, and **both are 4c's**.
    ///
    /// The message keeps `owned_message`'s exact shape, `"routine \"NAME\"
    /// is not implemented (4c)"`, because that trailing shape is a contract
    /// `loud.rs` pins with an `ends_with`, not a formatting preference -- a
    /// second spelling here would be a second thing to keep in sync for
    /// nothing.
    ///
    /// **Truncated, and that is the same contract `form_name`'s doc states.**
    /// A `Call::Named` target is a symbol or a quoted literal and so is
    /// bounded by the source, but a `Call::Dynamic` target is an arbitrary
    /// run-time value: `call (v)` with a megabyte in `v` would otherwise put
    /// a megabyte on stderr, which the differential harness then compares
    /// byte for byte. The oracle's own 43.1 does not truncate, so this is a
    /// deliberate difference on a path where the two already differ -- the
    /// oracle reports a condition here and this reports a gap.
    fn unresolved_call(name: &[u8]) -> Loud {
        const LIMIT: usize = 128;
        let shown = if name.len() > LIMIT {
            format!("{}...", String::from_utf8_lossy(&name[..LIMIT]))
        } else {
            String::from_utf8_lossy(name).into_owned()
        };
        Loud {
            message: owned_message(&format!("routine \"{shown}\""), Some("4c")),
        }
    }

    /// An activation's body selector named something that is not a routine
    /// body -- an internal inconsistency, never a program error.
    ///
    /// Unreachable through any program today, since nothing constructs a
    /// `Some(index)` selector at all (`Activation::body`'s own doc has the
    /// measured reason). Kept, and kept as a `Loud` rather than an
    /// `unreachable!`, for the same reason `Loud::instruction`'s own doc
    /// gives for not ending its match in a panic: a guarantee the resolution
    /// order makes is not one the type system enforces, and an abort is
    /// precisely the outcome the failing-loudly rule exists to exclude.
    /// Whoever first sets `Some(index)` is who makes this reachable.
    fn missing_body() -> Loud {
        Loud {
            message: "an activation's body selector names no routine body".to_string(),
        }
    }

    // **There is no `Loud::parse` any more, and its absence is the fix.** A
    // fragment that did not parse used to become a loud `INTERPRET text did
    // not parse: ...` at rc 120, with a doc comment recording that the
    // oracle raises 27.901 at rc 229 instead and that closing the gap needed
    // a `ParseError`-to-`Raised` conversion "built once for both, not here
    // for one caller". 4b's Task 2 built it -- `impl From<&ParseError> for
    // Raised` in `error.rs`, which `run_fragment` now uses. What the *top
    // level* could and could not take from it is written out at `execute`'s
    // own parse arm, below.
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
        // In scope since Task 3, alongside `Call::Named`/`Call::Dynamic`
        // below. A `RETURN` in the main body is not a gap either: measured,
        // it ends the program with its value exactly as `EXIT` does.
        | InstructionKind::Return { .. }
        | InstructionKind::Nop => None,
        // **Arm-grained, and two of the four arms are now `None`.**
        // `Call::Named` and `Call::Dynamic` are implemented (Task 3), so
        // "4b" would be a false statement in a table whose only job is to be
        // true -- `Loud::instruction` is no longer reached for either, and
        // an owner string nothing reads is exactly how the third copy of
        // this data drifts. A named call that resolves to no internal label
        // still fails loudly, through `Loud::unresolved_call`, naming `4c`:
        // the builtin and external steps behind the label search are that
        // phase's, not a residual claim on the `CALL` keyword itself.
        InstructionKind::Call(call) => match &**call {
            rexx_parse::Call::Named { .. } | rexx_parse::Call::Dynamic { .. } => None,
            rexx_parse::Call::Trap(_) => Some("4b"),
            rexx_parse::Call::Qualified { .. } => Some("Phase 5"),
        },
        InstructionKind::Procedure { .. }
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
    /// added (never replacing) inside `Interp::printed_indent` on every step
    /// while non-zero, and explicitly restored to `0` by `run_otherwise`
    /// once its own `run_bounded` call returns -- the one place that knows
    /// the elevated dispatch is now over. The `END`-only landing needs no
    /// explicit restore: 7.3 is fatal, so nothing runs afterward to see a
    /// stale value (`execute`, `lib.rs`, gives every run a fresh `Interp`).
    ///
    /// **This field means exactly one thing again, and briefly did not.**
    /// 4b's Task 2 was told to carry an `INTERPRET` fragment's activation
    /// base here as well, on the reasoning that both are "an indent added on
    /// top of `static_indent`". They are not the same quantity and the
    /// difference is lifetime: this one is transient and its two producers
    /// write it **absolutely** (`= 4` at the absorbed escape, `= 0` at
    /// `run_otherwise`), while an activation base lives for the whole life
    /// of the fragment. Measured, with no `CALL` anywhere -- `do z = 1 to 1`
    /// around `interpret "select; when 1 = 0 then nop; otherwise nop; end;
    /// say 1/0"` printed `say 1/0` at 0 where the oracle prints 2, because
    /// `run_otherwise`'s reset destroyed the base. The base now lives in
    /// [`Interp::activation_indent`], the two are added together, and each
    /// producer writes only its own.
    ///
    /// **The "narrower than the general case" disclosure this doc used to
    /// carry is gone because the narrowness is.** It said only
    /// `step_in_temps_frame` and `run_otherwise` added this offset, that the
    /// `WHEN` scan and the `WHILE`/`UNTIL` overrides did not, and bounded the
    /// consequence with "no corpus or spec example nests this deeply". The
    /// bound was false **before any fragment base existed**: a nested
    /// `SELECT` inside an escaped `OTHERWISE`, with no `INTERPRET` in the
    /// program, printed its inner `WHEN` at 6 where the oracle prints 10.
    /// A fragment base widened the reach -- a plain `SELECT` inside an
    /// `INTERPRET` inside one `DO` printed 2 against the oracle's 4 -- but it
    /// did not create the defect, and saying it did would be the same false
    /// bound one notch narrower. Every site that
    /// applies either offset now goes through `Interp::printed_indent`, the
    /// `WHEN` scan included, so there is no per-site list left to go stale.
    /// The `WHILE`/`UNTIL` sites were never really exceptions -- they read
    /// `current_value_indent`, which is a `printed_indent` result already.
    /// `pop_search_frame` is the one deliberate exclusion and says so at its
    /// own definition.
    indent_offset: usize,
    /// The absolute printed indent every clause of the **current activation
    /// level** starts from -- `0` for a program's own body, and an
    /// `INTERPRET` fragment's enclosing clause's own printed indent for the
    /// life of that fragment.
    ///
    /// Added to `static_indent` alongside [`Interp::indent_offset`] by
    /// `Interp::printed_indent`, which is the one place either is applied.
    /// **`0` throughout every 4a program**, which is what makes adding it at
    /// a site incapable of moving a 4a expectation: nothing but a fragment
    /// (and, next, `CALL`) ever sets it.
    ///
    /// **Measured delta 0 for a fragment, +2 for a called routine**, and
    /// that is why this is the activation's base rather than "one more
    /// level of nesting". `interpret "do jj = 1 to 1; say 2 & 1; end"` at
    /// top level echoes the inner clause at 2 and the `INTERPRET` at 0, and
    /// the identical fragment two `DO`s deep echoes them at 6 and 4 -- the
    /// fragment adds nothing of its own. A `CALL` at printed indent 4 into a
    /// flat routine echoes the callee's clause at 6, so Task 3 sets this to
    /// the calling clause's printed indent **plus two**.
    ///
    /// **Set, not added, and `indent_offset` is zeroed with it.** The
    /// `Interpret` arm saves both fields, sets this to the enclosing
    /// clause's `current_value_indent` and `indent_offset` to `0`, and
    /// restores both afterwards. The enclosing clause's own printed indent
    /// already contains whatever escape elevation was in force, so leaving
    /// `indent_offset` alone would count it twice -- measured on an
    /// `INTERPRET` inside an escaped `OTHERWISE`'s own body one `DO` deep,
    /// where the oracle prints the fragment's clause at 12 and the
    /// double-counting version prints 16. A fragment is a fresh level, so it
    /// starts with a fresh (zero) escape elevation; saving and restoring
    /// rather than clearing is what lets a fragment nest inside a fragment.
    activation_indent: usize,
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
    ///
    /// **First-wins *within one level*, and 4b's Task 2 left that unchanged
    /// on purpose (inherited item I11).** The `is_none()` guard in
    /// `record_failure_at` still means the most specific clause at this level
    /// wins, and it still never clears. Clearing it matters only once a trap
    /// can resume after a raise, which is **Task 7's** to own -- for both this
    /// field and [`Interp::failure_sites`] below, since a resumed trap would
    /// have to empty the stack as well as the slot.
    failure_site: Option<FailureSite>,
    /// The levels that have already finished failing, innermost first --
    /// `Raised::report`'s echo stack minus its last entry.
    ///
    /// **Why two fields and not one `Vec`.** `failure_site` is the level
    /// currently unwinding and is first-wins; this is the record of levels
    /// already sealed. `run.rs`'s `seal_site_level` moves one into the other
    /// and is called by exactly the constructs that open a level -- today
    /// `run_fragment`, and Task 3's `CALL` next. Keeping the two apart is
    /// what lets the guard stay a plain `is_none()` rather than a "did
    /// anything get recorded since the current level opened" watermark, and
    /// it is why the pre-Task-2 single-site behaviour falls out unchanged
    /// when nothing ever seals: this stays empty and `execute` builds a
    /// one-entry stack.
    ///
    /// Never resolved by walking `Interp::activations` instead: `run` pops
    /// the activation before `execute` sees the error, which is the whole
    /// reason `failure_site` exists rather than being reconstructed at the
    /// top.
    failure_sites: Vec<FailureSite>,
    /// The line number every clause echo prints while an `INTERPRET`
    /// fragment is running, overriding the clause's own line in its own
    /// source.
    ///
    /// A fragment's spans index the fragment's text, which is line 1 of a
    /// source of its own, and the oracle prints the **enclosing `INTERPRET`
    /// clause's** line for every clause inside it -- measured, `interpret
    /// "say 2 & 1"` on line 2 echoes both the fragment's clause and the
    /// `INTERPRET` as line 2, and a fragment inside a fragment (`interpret
    /// 'interpret "say 2 & 1"'` on line 3, inside two `DO`s) echoes all
    /// three at line 3. So the override is set once by the outermost
    /// `INTERPRET` and inherited unchanged inward, which falls out of setting
    /// it from the resolved line of the `INTERPRET` clause itself: by then
    /// that line has already been through any override in force.
    ///
    /// A field rather than a parameter for the same reason
    /// `current_value_indent` is one: `clause_site` is reached from four
    /// call sites across `step_in_temps_frame`, `record_failure_at`,
    /// `leave_origin` and `run_otherwise`, none of which otherwise has any
    /// business knowing a fragment is running. Saved and restored around
    /// `run_fragment` by the `Interpret` arm, not cleared afterwards, so a
    /// nested fragment cannot strand the outer one's value.
    clause_line_override: Option<usize>,
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
            current_value_indent: 0,
            current_case_text: None,
            indent_offset: 0,
            activation_indent: 0,
            failure_site: None,
            failure_sites: Vec::new(),
            clause_line_override: None,
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
    ///
    /// This is the *outermost* activation only. A `CALL` pushes and pops its
    /// own (`run.rs`'s `exec_call`), so the pop below is still matched with
    /// the push above it by the time control gets here -- `run_activation`'s
    /// own loop asserts exactly that after every step.
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

        // `Returned` and `Exited` are the same thing at the top: measured,
        // `return 5` in a main body with no active call exits 5, exactly like
        // `exit 5`, and a bare `return` there exits 0. `Ended` keeps the two
        // apart because a *callee* has to tell them apart, not because the
        // program's own exit value ever depends on which arrived.
        let exit = self.run_activation().map(Ended::value);

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
        // **A top-level parse failure stays loud, and 4b's Task 2 checked
        // whether it could stop being loud rather than assuming either way.**
        // Task 2 built the `ParseError`-to-`Raised` conversion the old
        // `Loud::parse` doc comment asked for and used it for `INTERPRET`
        // (`run_fragment`). This arm can have the *mapping* -- it is one
        // `impl From` and nothing about it is fragment-specific -- but not
        // the *report*, and the obstacle is concrete rather than a
        // preference: `Raised::report`'s major line names a source line, the
        // line comes from `ParseError::line(&source)`, and `parse_program`
        // takes `text` by value and returns only the `ParseError` on the
        // failure path, so by the time this arm runs the `ProgramSource` that
        // could answer has been built and dropped inside the parser. There is
        // no way back to it from here: `rexx-parse` exposes `ProgramSource::
        // new` and `scan`, but the composition that turns a `&ProgramSource`
        // into a `Program` is private, so the only route is to clone the
        // whole program text before every parse to serve a path that runs
        // only on syntax errors. Closing it properly is a `rexx-parse`
        // signature change -- hand the source back alongside the error, or
        // make the `parse(&ProgramSource)` composition public -- which is
        // outside the file list Task 2 was given, so it is written down here
        // rather than half-done. The second gap `INTERPRET` shares is the
        // clause echo: the failing clause never became an `Instruction`, and
        // `ParseError` carries the clause's *start* byte with no end, so
        // there is no span to echo at either level.
        //
        // Parse errors remain deliberately not reproduced byte for byte (the
        // number and sub match on a plausible line; message text and
        // substitutions are not gated), so this arm is wrong in the details
        // on purpose and right in never being mistaken for success.
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
    // The whole echo stack, innermost first: the levels `seal_site_level`
    // already closed, then the level that was still unwinding when the
    // condition reached the top. See `Interp::failure_sites` for why the two
    // are separate fields, and `Raised::report` for what the order means.
    let mut failure_sites = std::mem::take(&mut interp.failure_sites);
    failure_sites.extend(interp.failure_site.take());
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
            // `run_activation` records the site on the way out. An empty
            // stack here would mean a condition escaped without passing an
            // instruction loop, which nothing in 4a or 4b-so-far can do; it
            // renders visibly rather than panicking, on the error path's
            // standing rule that a reportable condition must never become a
            // crash.
            if failure_sites.is_empty() {
                failure_sites.push(FailureSite {
                    line: 0,
                    text: b"<no failing clause recorded>".to_vec(),
                    indent: 0,
                });
            }
            let site = ClauseSite {
                path,
                sites: &failure_sites,
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

    /// 4b Task 2: a condition raised inside an `INTERPRET` fragment reports
    /// **both** clauses, and this is the whole report, byte for byte, at the
    /// one level `run_program` can see it.
    ///
    /// Oracle, verbatim, for a program whose two `DO`s put the `INTERPRET` at
    /// printed indent 4 and whose fragment nests its failing clause one
    /// deeper (rc 222, empty stdout):
    ///
    /// ```text
    ///      3 *-*       say 2 & 1;
    ///      3 *-*     interpret "do jj = 1 to 1; say 2 & 1; end"
    /// Error 34 running <path> line 3:  Logical value not 0 or 1.
    /// Error 34.901:  Logical value must be exactly "0" or "1"; found "2".
    /// ```
    ///
    /// **Every field here discriminates a different wrong implementation.**
    /// Four mutations, each built and run rather than argued about:
    ///
    /// | mutation | this shape prints | `interpret ...` alone on line 1 |
    /// |---|---|---|
    /// | the level is never sealed | one echo, not two | one echo, not two |
    /// | no line override | inner echo at line **1** | **identical** |
    /// | a called routine's `+ 2` base | inner echo at **8** | inner at 4 |
    /// | no base at all | inner echo at **2** | **identical** |
    ///
    /// **That right-hand column is why the program is two `DO`s deep with the
    /// `INTERPRET` on line 3 and not one line at top level**, and the
    /// measurement corrected a claim written here first and checked
    /// afterwards. Two of the four survive the simplest shape, and for two
    /// unrelated reasons that both look like "it worked": at indent 0 the
    /// base is 0, so omitting it changes nothing, and with the `INTERPRET` on
    /// line 1 the enclosing line and the fragment's own line are both 1, so
    /// overriding it changes nothing either. Varying only the depth would
    /// have caught the first and not the second.
    ///
    /// `say 2 & 1;` keeps its semicolon because that is where the fragment's
    /// own clause span ends; trimming it diverges.
    ///
    /// `rust/corpus/lang/interpret_error_echo.rex` is the same shape as a
    /// live differential, and all four mutations above were confirmed against
    /// it as well. This test exists beside it because the corpus gate needs a
    /// built oracle and is skipped without one, and the property is too
    /// central to have no assertion on a machine that lacks it.
    #[test]
    fn a_raise_inside_a_fragment_reports_both_clauses() {
        let program = b"do kk = 1 to 1\n\
                        do mm = 1 to 1\n\
                        interpret \"do jj = 1 to 1; say 2 & 1; end\"\n\
                        end\n\
                        end\n";
        let outcome = run_program(TEST_PATH, program.to_vec());
        assert_eq!(outcome.exit_code, 222);
        assert_eq!(outcome.stdout, b"");
        assert_eq!(
            String::from_utf8(outcome.stderr).unwrap(),
            format!(
                concat!(
                    "     3 *-*       say 2 & 1;\n",
                    "     3 *-*     interpret \"do jj = 1 to 1; say 2 & 1; end\"\n",
                    "Error 34 running {path} line 3:  Logical value not 0 or 1.\n",
                    "Error 34.901:  Logical value must be exactly \"0\" or \"1\"; found \"2\".\n",
                ),
                path = TEST_PATH
            )
        );
    }

    /// Review round 1, F1 and its neighbours: the activation base survives
    /// every construct inside the fragment that writes an indent of its own.
    ///
    /// **F1 was a live divergence and this is the shape that found it.**
    /// `Interp::indent_offset` had two absolute writers (`= 4` at the
    /// absorbed `WhenCase` escape, `= 0` at the end of `run_otherwise`),
    /// which were correct while it carried only a transient escape
    /// elevation and destroyed the fragment base once it carried that too.
    /// Every row below was captured from the oracle before the fix and every
    /// one of the first three failed against it:
    ///
    /// | row | what writes an indent inside the fragment | was |
    /// |---|---|---|
    /// | `OTHERWISE` | `run_otherwise`'s `indent_offset = 0` | echo at 0, oracle 2 |
    /// | `LEAVE` out of a `DO` | `pop_search_frame`'s reset | echo at 0, oracle 2 |
    /// | escaped `OTHERWISE` around the `INTERPRET` | the `= 4` write | already right, and the row that keeps it right |
    ///
    /// The third row is the **regression guard on the fix itself**, not a
    /// third bug: the enclosing clause's own printed indent already contains
    /// the escape elevation, so a fix that set `activation_indent` without
    /// zeroing `indent_offset` counts the 4 twice and prints 16 where the
    /// oracle prints 12. It passed before this fix and it has to keep
    /// passing, which is the only reason it is here.
    ///
    /// Each row asserts the whole stderr, so a wrong indent on *either* echo
    /// fails rather than only the one being probed.
    #[test]
    fn a_fragments_activation_base_survives_every_indent_writer_inside_it() {
        // (program, expected stderr with `{path}` for the program's path)
        let rows: &[(&str, &str)] = &[
            (
                "do z = 1 to 1\n\
                 interpret \"select; when 1 = 0 then nop; otherwise nop; end; say 1/0\"\n\
                 end\n",
                concat!(
                    "     2 *-*   say 1/0\n",
                    "     2 *-*   interpret \"select; when 1 = 0 then nop; otherwise nop; \
                     end; say 1/0\"\n",
                    "Error 42 running {path} line 2:  Arithmetic overflow/underflow.\n",
                    "Error 42.3:  Arithmetic overflow; divisor must not be zero.\n",
                ),
            ),
            (
                "do z = 1 to 1\n\
                 interpret \"do jj = 1 to 1; leave zz; end\"\n\
                 end\n",
                concat!(
                    "     2 *-*   leave zz;\n",
                    "     2 *-*   interpret \"do jj = 1 to 1; leave zz; end\"\n",
                    "Error 28 running {path} line 2:  Invalid LEAVE or ITERATE.\n",
                    "Error 28.3:  Symbol following LEAVE (\"ZZ\") must either match the \
                     label of a current loop or block instruction.\n",
                ),
            ),
            (
                "do z = 1 to 1\n\
                 select case 2\n\
                 \x20 when 2 then\n\
                 \x20   when 3 then nop\n\
                 \x20 otherwise interpret \"do jj = 1 to 1; say 1/0; end\"\n\
                 end\n\
                 end\n",
                concat!(
                    "     5 *-*             say 1/0;\n",
                    "     5 *-*           interpret \"do jj = 1 to 1; say 1/0; end\"\n",
                    "Error 42 running {path} line 5:  Arithmetic overflow/underflow.\n",
                    "Error 42.3:  Arithmetic overflow; divisor must not be zero.\n",
                ),
            ),
        ];
        for (index, (program, expected)) in rows.iter().enumerate() {
            let outcome = run_program(TEST_PATH, program.as_bytes().to_vec());
            assert_eq!(
                String::from_utf8(outcome.stderr).unwrap(),
                expected.replace("{path}", TEST_PATH),
                "row {index}"
            );
        }
    }

    /// Review round 1, F2: the `WHEN` scan's own echo carries the offsets too.
    ///
    /// `Select`'s arm computed `when_indent` from `static_indent` alone --
    /// the one clause-echo indent in `run.rs` that added neither offset.
    /// **Already a live 4a divergence before any fragment base existed**: a
    /// nested `SELECT` inside an escaped `OTHERWISE` reaches it with no
    /// `INTERPRET` in the program at all, printing the inner `WHEN` at 6
    /// where the oracle prints 10. The fragment base only made it easy to
    /// hit -- a plain `SELECT` inside an `INTERPRET` inside one `DO` is not
    /// deep nesting either. The old doc bounded this with "no corpus or spec
    /// example nests this deeply", and that false bound is why nobody looked;
    /// "it only became live with a fragment base" would be the same trap one
    /// notch narrower.
    ///
    /// Oracle, verbatim and byte-identical below (rc 0, empty stdout). The
    /// `WHEN` line is the one that was wrong, at 2 instead of 4; the whole
    /// transcript is asserted so that fixing it by moving the error elsewhere
    /// fails too. A plain `do` block rather than `do z = 1 to 1` on purpose:
    /// a `Controlled` loop's re-tested pass omits two `>>>` lines this crate
    /// does not yet emit (the KNOWN GAP row on the re-tested pass), and this
    /// test is not the place to encode that.
    #[test]
    fn a_when_scan_inside_a_fragment_echoes_at_the_fragments_own_indent() {
        let program = "trace r\n\
                       do\n\
                       interpret \"select; when 1 = 1 then nop; end; nop\"\n\
                       end\n";
        let outcome = run_program(TEST_PATH, program.as_bytes().to_vec());
        assert_eq!(outcome.exit_code, 0);
        assert_eq!(outcome.stdout, b"");
        assert_eq!(
            String::from_utf8(outcome.stderr).unwrap(),
            concat!(
                "     2 *-* do\n",
                "     3 *-*   interpret \"select; when 1 = 1 then nop; end; nop\"\n",
                "       >>>     \"select; when 1 = 1 then nop; end; nop\"\n",
                "     3 *-*   select;\n",
                "     3 *-*     when 1 = 1 \n",
                "       >>>       \"1\"\n",
                "     3 *-*       then\n",
                "     3 *-*         nop;\n",
                "     3 *-*   nop\n",
                "     4 *-* end\n",
            )
        );
    }

    /// 4b Task 2, Step 5b: a fragment that does not parse raises the oracle's
    /// own condition instead of failing loudly.
    ///
    /// Measured: `interpret "do forever then"` is **27.901 at rc 229** on the
    /// oracle, and `interpret "if"` is 35.929 at rc 221. Through 4a and 4b's
    /// Task 1 both were `Loud::parse` at rc 120 -- correct-but-loud while
    /// `INTERPRET` was unreachable, and a live divergence once Task 1 made it
    /// reachable.
    ///
    /// **What is asserted and what is deliberately not.** The exit code and
    /// the enclosing clause echo are the oracle's exactly, so this fails for
    /// anything that kept `NOT_IMPLEMENTED_EXIT` or that dropped the echo.
    /// Two things still diverge and are asserted as they *are* rather than as
    /// the oracle has them, so the divergence cannot shrink unnoticed: the
    /// oracle prints a further echo of the failing fragment clause above this
    /// one (`     2 *-* do forever then`), which needs a clause span
    /// `ParseError` does not carry, and its sub-message reads `found "THEN"`
    /// where ours leaves the catalogue's `&1` unfilled, because `ParseError`
    /// carries no substitution values. `phase-4-exclusions.txt`'s amended
    /// KNOWN GAP row records both with their measurements.
    #[test]
    fn a_fragment_that_does_not_parse_raises_the_oracles_condition() {
        let outcome = run_program(
            TEST_PATH,
            b"say 1\ninterpret \"do forever then\"\n".to_vec(),
        );
        assert_eq!(outcome.exit_code, 229);
        assert_eq!(outcome.stdout, b"1\n");
        assert_eq!(
            String::from_utf8(outcome.stderr).unwrap(),
            format!(
                concat!(
                    "     2 *-* interpret \"do forever then\"\n",
                    "Error 27 running {path} line 2:  Invalid DO or LOOP syntax.\n",
                    "Error 27.901:  Incorrect data following FOREVER keyword on the loop; \
                     found \"&1\".\n",
                ),
                path = TEST_PATH
            )
        );

        let outcome = run_program(TEST_PATH, b"say 1\ninterpret \"if\"\n".to_vec());
        assert_eq!(outcome.exit_code, 221);
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
