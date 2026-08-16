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

//! The executor.
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

use rexx_classes::{ClassKind, MethodId};
use rexx_core::{Heap, ObjRef, RootSet, SlotRef};
use rexx_parse::{
    AnnotationTarget, AttributeDirective, AttributeStyle, ClassDirective, CodeBody, ConstantValue,
    Directive, DirectiveKind, Expr, ExprKind, InstructionKind, MethodDirective, Operator, Program,
    SymbolId, SymbolTable, compound_parts, parse_program,
};
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

// The value model: `text`/`number`/`to_text`/`to_number` on `Interp`, and the
// two rules D15 exists to enforce (a number's rendering is fixed at creation,
// and a `SmallInt` is admissible only within the DIGITS that produced it).
mod value;

// Stems and compound variables (D15a): tail resolution, the tombstone rule,
// and the "replace the object, mutate a tail in place" split.
mod stem;

// The in-process external data queue (I15) `PUSH`/`QUEUE` write to and
// `PULL`/`PARSE PULL` read back.
mod queue;
use queue::Queue;

// What a command line supplies to a top-level program: the one argument string
// it can carry, how a list of words becomes that string, and where `.input`
// reads from.
mod invocation;
pub use invocation::{Engine, Invocation, ProgramInput, join_command_line};

// `.input`: one line position, shared by every construct that reads a line,
// and the queue-first rule `PULL` follows on top of it.
mod input;
use input::Input;

// The per-body resolution plan (D16): `Plan`, `BodyKey`, `ProgramId`, the
// plan cache, and the full name-resolution order (plan, then `extra`, then
// growth).
mod plan;
use plan::{BodyKey, CompoundName, Plan, ProgramId};

// One activation: everything about the frame currently executing (D16).
mod activation;
use activation::{Activation, ActivationId};
use clause::ClauseState;

// `Raised` (the payload of a real Rexx condition) and `Failure` (either a
// `Loud` not-implemented marker or a `Raised` condition, the one type
// `step` and everything above it propagate).
mod error;
use error::{ClauseSite, Failure, FailureSite, Raised};

// Expression evaluation (`eval`/`eval_node`): terms, arithmetic and
// concatenation.
mod eval;

// The builtin functions: the name set (read from `rexx_inventory`, never
// copied), the per-name arity, and the `dispatch` `Interp::invoke_call`
// reaches for a name `resolve_call` answered `Resolved::Builtin` for.
mod builtin;

// The instruction loop (D16's "Control flow"): `Flow`, and `step` and its two
// callers, together with the borrow discipline `run_activation` is written
// down to prove (Task 3's spike). Extended task by task with the branches and
// calls later tasks add; Task 9 is the first to extend it, with the seven
// instructions that do not branch.
mod clause;
mod run;
use run::Ended;

// The `PARSE` template engine: the movement cursor (source-independent, one
// struct, unit-tested against measured oracle bytes) and the driver that
// evaluates trigger operands, traces, and assigns the targets.
mod parse_template;

// `TRACE` (D17): the mode, the nine reachable prefixes' own byte formatting,
// and the classification a `TRACE`/`TRACE VALUE` setting goes through to
// become one. `run.rs`'s `step_in_temps_frame` and its loop drivers, and
// `eval.rs`'s `eval`, own *when* to call into this module; this module owns
// only the bytes.
mod trace;
use trace::ChunkTrace;

// The register-based instruction stream (Phase 4e): `Op`, `Chunk`, and
// `compile`, the one pass that turns a body into a `Chunk`. `Interp::chunk_for`
// (`plan.rs`, beside `plan_for`) is the cache that makes compiling a body once
// rather than once per entry.
mod ir;

// Message dispatch (Phase 5a): the object model a send resolves against, the
// `resolve`/`invoke` pair D24 asks for, and the one security chokepoint D45
// asks for.
mod dispatch;

// `.environment`, `.local`, `.context` and `.methods` as objects (D33), the
// order `PackageClass::findClass` resolves a `.NAME` in, and the one directory
// chokepoint D45 asks for.
mod environment;

/// The exit code for a construct this crate does not implement.
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
/// against, and it needed no external bisection to begin with, only a
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

/// The arena size below which no ordinary run ever collects, and the floor
/// every later growth allowance is raised to (see `Interp::collect_at`).
///
/// An arena of this many slots costs about 6 MB, at the 96 bytes per `Slot`
/// `phase-4d-retention.md` measured, plus each object's own payload. Below it
/// there is nothing worth reclaiming and a collection is pure cost: a program
/// that allocates a few hundred values -- which is most of the corpus --
/// never collects at all, and pays one branch per allocation for the
/// trigger's existence.
///
/// The number is a round power of two rather than a tuned one. It is the
/// floor `phase-4d-retention.md`'s prototype used, kept so that this crate's
/// measured landing can be read against that document's prediction rather
/// than against a threshold chosen after seeing the result.
const COLLECT_FLOOR: usize = 65_536;

/// The globals entry [`Interp::root_exit_value`] writes. A name rather than an
/// index because `RootSet::add_global` is keyed by name and replaces in place,
/// which is the behaviour wanted here.
const EXIT_VALUE_ROOT: &str = "the program's exit value";

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
/// streaming. Nothing in the corpus does that, and a streaming model would
/// replace this whole shape.
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
    /// How many times the run declined to compile a body because it does not
    /// fit the compiled stream's index widths, and ran it on the tree-walker
    /// instead.
    ///
    /// **Refusals, not bodies.** A refused body is not remembered -- there is
    /// no negative cache -- so one entered a thousand times is compiled,
    /// refused and counted a thousand times.
    ///
    /// Always `0` under [`Engine::TreeWalker`](crate::Engine::TreeWalker),
    /// which compiles nothing. Under [`Engine::Ir`](crate::Engine::Ir) it is
    /// what stops the fallback being silent: the dual-engine harness asserts
    /// it is zero across the whole population, so a compiler that starts
    /// refusing ordinary bodies goes red rather than quietly running
    /// everything on the tree-walker and passing.
    pub chunks_refused: usize,
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
///
/// **This is the one thing in `Outcome` the two engines can disagree about, and
/// the disagreement is the definition working rather than a defect.** It counts
/// `eval` recursion, and the compiled stream produces some values without
/// recursing at all: `crate::ir::Op::Const` builds a literal's value from the
/// chunk's own constant table and `crate::ir::Op::Load` reads a bare symbol's
/// out of a frame slot, neither of them entering `eval`. So `say 'x'` reports
/// `max_depth` 1 on the tree-walker and 0 on the compiled stream -- one level of
/// `eval` against none -- and 0 is the honest answer for a run that recursed
/// nowhere. **Nothing asserts it either way**: `tests/ir_dual.rs` compares
/// stdout, stderr and exit status, and this field reaches no oracle comparison
/// at all (`tests/support/oracle.rs`'s own doc says the oracle process never
/// measures it). An operator chain, which is what anything sizing a stack from
/// this measures, recurses identically on both engines, because an operator is
/// evaluated by `eval` whichever engine reached it. **A caller must not read
/// `max_depth` as a count of expressions evaluated.**
#[derive(Copy, Clone, Debug, Default)]
pub struct StackSpan {
    /// The deepest `eval` recursion the run reached. Zero if it never
    /// evaluated an expression at all, and zero also for a run whose only
    /// values came from ops that do not enter `eval` -- see this type's own doc
    /// comment.
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

/// A construct this crate does not implement, on its way to becoming an exit
/// code and a line on stderr.
///
/// Not a Rexx condition and never convertible into one: the whole point of
/// `NOT_IMPLEMENTED_EXIT` is that an implementation gap cannot produce a
/// passing differential test. Real errors have their own type, `Raised`, which
/// is a different thing entirely.
#[derive(Debug)]
struct Loud {
    message: String,
}

impl Loud {
    /// An instruction this crate does not execute. `keyword()` is `None` for the four
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

    /// An expression form this crate does not evaluate.
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

    /// A binary operator that no family of `Interp::apply_binary` claims -- an
    /// internal inconsistency, never a program error.
    ///
    /// `Operator::Backslash` is the operator this is about: it is the prefix
    /// `\` token, and the parser builds an `ExprKind::Prefix` from it rather
    /// than an `ExprKind::Binary`, so no program reaches this by writing one.
    /// An operator added to `rexx_parse::Operator` and left out of
    /// `eval::is_native_binary` would reach it too.
    ///
    /// Loud rather than an `unreachable!` for the reason [`Loud::instruction`]'s
    /// own doc gives: a guarantee the parser makes is not one the type system
    /// enforces, and an abort is precisely the outcome the failing-loudly rule
    /// exists to exclude.
    ///
    /// The variant's own name rather than its spelling, because
    /// `Operator::Abuttal`'s spelling is the empty string and a message naming
    /// it would name nothing. `Operator`'s derived `Debug` is one word with no
    /// tree behind it, unlike [`Loud::expression`]'s subject.
    ///
    /// [`Loud::instruction`]: Loud::instruction
    fn binary_operator(op: Operator) -> Loud {
        Loud {
            message: format!("binary operator {op:?} has no implementation"),
        }
    }

    /// A call that resolved to a **builtin this crate runs nothing for**.
    ///
    /// **Not "a name that resolved to nothing"**, which is what this used to
    /// answer and no longer does. A named call resolves in four steps --
    /// internal label, builtin, `::ROUTINE`, then an external Rexx file --
    /// and a name matching none of them is the oracle's own Error 43.1
    /// (`Raised::routine_not_found`), because the only step this crate skips
    /// is the file search and that answers nothing for a program with no such
    /// file beside it. External routine resolution is **Phase 7's**, with
    /// both transcripts in `phase-4-exclusions.txt`.
    ///
    /// What is left for this constructor is the second step's own gap: the
    /// fifteen builtins Phase 4 excludes outright (`builtin::
    /// is_excluded_builtin`), where the oracle answers normally and this
    /// crate has no code. A condition would be the wrong answer there --
    /// a program *expecting* 43.1 would pass against a gap -- which is why
    /// the excluded-builtin step sits in front of the `::ROUTINE` lookup
    /// rather than falling through it. `builtin::dispatch`'s own
    /// belt-and-braces arm is the other caller.
    ///
    /// The message keeps `owned_message`'s exact shape, `"routine \"NAME\"
    /// is not implemented (4c)"`, because that trailing shape is a contract
    /// `loud.rs` pins with an `ends_with`, not a formatting preference -- a
    /// second spelling here would be a second thing to keep in sync for
    /// nothing. The owner is `4c` for every name that reaches here, whichever
    /// phase actually owns the builtin; `corpus/keyword-exempt.txt`'s own
    /// header records that imprecision and which rows it affects.
    ///
    /// **Truncated**, which now costs nothing: every name reaching here is a
    /// builtin's, so it is short by construction. The truncation is kept
    /// because it is free and because nothing in the type stops a future
    /// caller handing over a `Call::Dynamic` target, which is an arbitrary
    /// run-time value.
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

    /// A message sent to a value whose class this phase does not build, so
    /// there is no behaviour to resolve the name against at all.
    ///
    /// The reachable case is a stem: `rexx_classes::deferred_classes` leaves
    /// `.Stem` out because its `Setup.cpp` block hides the comparison
    /// methods and `MethodDict` models no removal. The oracle answers a send
    /// to one -- `a. = 'dflt'; say a.~length` is `4`, forwarded through
    /// `StemClass`'s own `UNKNOWN` -- so 97.1 would be a wrong answer that a
    /// program could trap, which is why this is loud instead.
    ///
    /// `kind` names the value's shape rather than a class, because the whole
    /// point is that no class object was found for it.
    fn receiver_class(kind: &str) -> Loud {
        Loud {
            message: owned_message(&format!("a message send to {kind}"), Some("Phase 5")),
        }
    }

    /// An operator whose **left** operand is an object this phase can build
    /// but send no message to: a class object, or one of the interpreter's
    /// own (`.environment`, `.local`, `.methods`, `.context`).
    ///
    /// **The oracle sends the operator to the left operand as a message**, so
    /// what it answers is that object's own method and not a comparison of
    /// renderings. Measured: `.array + 1` and `.array > .array` are 97.1;
    /// `.methods == .routines` is `0` where both render `a StringTable`;
    /// `.array == "The Array class"` is `0` against the very text the object
    /// renders as. Handing those operands to the string and numeric operators
    /// answers `1`, `1` and `1` -- three wrong answers at rc 0, where this
    /// crate refused the whole program before `.NAME` resolved at all.
    ///
    /// **An operator's right operand is not this**, measured the same way:
    /// `1 + .array` is 41.1 quoting `"The Array class"` and
    /// `"a StringTable" == .methods` is `1`, both of which this crate already
    /// answers identically, because the oracle sends the operator to the
    /// *left* operand and that operand converts the right one through
    /// `stringValue()` exactly as this crate does. **That is a fact about
    /// operators and not a rule about right-hand operands**: a controlled `DO`
    /// header rounds every position through a unary operator of its own, so
    /// [`Loud::object_position`] refuses `do i = 1 to .array` too.
    ///
    /// `op` is spelled by the caller rather than taken as an `Operator`,
    /// because a prefix operator and a binary one are different types with
    /// the same need.
    fn operator_operand(op: &str, kind: &str) -> Loud {
        Loud {
            message: owned_message(
                &format!("the operator `{op}` applied to {kind}"),
                Some("Phase 5"),
            ),
        }
    }

    /// One of the objects [`Loud::operator_operand`] refuses, in a position
    /// that is not an operator's operand: a `DO` header's value, `DO OVER`'s
    /// target, a controlled loop's own control variable at the increment, or
    /// a `RAISE SYNTAX` clause's `ADDITIONAL` value.
    ///
    /// **A separate constructor because the program contains no operator to
    /// name**, and naming one would send a reader looking for something that
    /// is not there. `position` is the whole phrase rather than a keyword, so
    /// each site says where it is in its own words.
    ///
    /// What the oracle does at each, measured:
    ///
    /// * a controlled header's `initial`/`TO`/`BY`, and the control variable
    ///   the increment adds to, are rounded through what is a real unary `+`,
    ///   so each answers 97.1;
    /// * `DO OVER`'s target is handed to `requestArray`, which is 98.913 for a
    ///   class and an iteration of the entries for a directory or a string
    ///   table -- neither of which this crate can produce;
    /// * a `RAISE` clause's `ADDITIONAL` value reaches `requestArray` only
    ///   under a `SYNTAX` condition (`RaiseInstruction.cpp:280-286`). Under
    ///   any other condition, and for the `ARRAY (...)` form whose value is
    ///   already an array, the elements are rendered by `stringValue()` and
    ///   this crate matches -- which is why the refusal carries that guard.
    ///
    /// **`do i = 1 to .array` is why "the right operand always agrees" is not
    /// a rule.** It held for the binary operators, where the operand the
    /// operator was sent to converts the other through `stringValue()`; it
    /// does not hold here, where every position converts on its own.
    fn object_position(position: &str, kind: &str) -> Loud {
        Loud {
            message: owned_message(&format!("{kind} as {position}"), Some("Phase 5")),
        }
    }

    /// A `.NAME` the oracle's `.environment` or `.local` answers and this
    /// crate builds nothing for.
    ///
    /// Loud rather than the dotted-text fallback, which is what an
    /// **unresolved** name renders as on both sides: taking the fallback for a
    /// name the oracle resolves is a silent wrong answer at rc 0, and it is
    /// the shape `phase-4-exclusions.txt` recorded against `VALUE`'s
    /// one-argument form before this phase closed it.
    ///
    /// `owner` differs by which directory holds the name --
    /// `environment.rs`'s own bootstrap has the split and its reason.
    fn environment_symbol(name: &[u8], owner: &'static str) -> Loud {
        let shown = String::from_utf8_lossy(name);
        Loud {
            message: owned_message(&format!("environment symbol \"{shown}\""), Some(owner)),
        }
    }

    /// A message that **resolved** to a primitive method this crate has no
    /// code for.
    ///
    /// Loud rather than a condition, for the reason [`Loud::unresolved_call`]
    /// gives about excluded builtins: the oracle answers normally here, so a
    /// 97.1 would let a program expecting the oracle's answer pass against a
    /// gap. 97.1 is reserved for a name the receiver's behaviour genuinely
    /// does not answer, which is a different question and is decided before
    /// this constructor can be reached.
    ///
    /// `scope` is the class the definition came from, which is what makes the
    /// message actionable: the same name can be a different method on a
    /// different class.
    ///
    /// [`Loud::unresolved_call`]: Loud::unresolved_call
    fn native_method(name: &[u8], scope: &str) -> Loud {
        let shown = String::from_utf8_lossy(name);
        Loud {
            message: owned_message(
                &format!("method \"{shown}\" of class \"{scope}\""),
                Some("Phase 5"),
            ),
        }
    }

    /// `PROCEDURE EXPOSE` naming a single compound tail.
    ///
    /// **Both spellings reach here, and the second is easy to miss.** The
    /// direct one is `procedure expose a.1`; the indirect one is `v = 'A.1'`
    /// with `procedure expose (v)`, because `expose_names` expands the
    /// selector's value into ordinary names and a compound-shaped word among
    /// them arrives at the same check. Measured, the indirect form: oracle rc
    /// 0 printing `changed other`, this crate rc 120 with the message below.
    /// The gap is exactly as wide as the direct spelling suggests, not
    /// narrower.
    ///
    /// **A disclosed gap inside an otherwise implemented instruction, and
    /// loud rather than approximated because the near-miss is a silent wrong
    /// answer.** Measured: with the caller holding `a.1 = 'kept'` and `a.2 =
    /// 'other'`, `sub: procedure expose a.1` writing both tails leaves the
    /// caller printing `changed other` -- tail 1 is shared and tail 2 is the
    /// callee's own. So this is aliasing *inside* a stem object, at one tail,
    /// and this crate's exposure mechanism aliases whole slots: the stem lives in a
    /// slot and its tails do not. Exposing the whole stem instead would make
    /// `a.2` shared as well, which is a wrong answer found by chasing a wrong
    /// value rather than a message that says why.
    ///
    /// No owner string: unlike `unresolved_call`'s `4c`, the steps behind
    /// this are not another phase's to build -- nothing has been scheduled to
    /// build them. `owned_message` is deliberately not used, since its shape
    /// belongs to the variant-keyed owner tables (`instruction_owner`) and
    /// this is a sub-case within a variant those tables call implemented.
    fn compound_expose(name: &[u8]) -> Loud {
        Loud {
            message: format!(
                "PROCEDURE EXPOSE of the single compound tail \"{}\" is not implemented",
                String::from_utf8_lossy(name)
            ),
        }
    }

    /// A builtin's option letter whose answer this crate cannot produce.
    ///
    /// **A disclosed gap inside an otherwise delivered builtin**, the shape
    /// [`Loud::compound_expose`] established, and loud for the same reason:
    /// the near miss is a silent wrong answer.
    ///
    /// **Two different reasons reach it, and the name says "object" for
    /// only one of them.** `ARG(n,'A')` and `CONDITION('A')`/`CONDITION('O')`
    /// answer an `Array` or a `Directory` -- measured, `condition('A')~class`
    /// inside a `SIGNAL ON SYNTAX` handler is `The Array class` and
    /// `condition('O')~class` is a `Directory` with 14 items -- and this
    /// crate's value model has neither. `CONDITION('D')` for a `NOVALUE`
    /// condition answers an ordinary *string*, the variable's derived name,
    /// which this crate simply does not carry as far as the handler
    /// (`Raised::description`). Both are "the oracle has an answer here and
    /// we cannot build it", which is what this constructor is for; only the
    /// first is about an object.
    ///
    /// `why` names what is missing, so the message says that rather than
    /// only that something is.
    ///
    /// No owner string: like `compound_expose`, this is a sub-case within a
    /// builtin the status table calls implemented, and `owned_message`'s
    /// shape belongs to the variant-keyed owner tables.
    ///
    /// [`Loud::compound_expose`]: Loud::compound_expose
    fn builtin_option_object(routine: &str, option: u8, why: &str) -> Loud {
        Loud {
            message: format!(
                "{routine} option \"{}\" answers {why}, which is not implemented",
                option.escape_ascii()
            ),
        }
    }

    /// `VALUE`'s three-argument form: a *present* third argument selects an
    /// external pool rather than this crate's own local variables
    /// (`expression/BuiltinFunctions.cpp:1848`-`1913`).
    ///
    /// **Presence decides, not the selector's value.** Measured 2026-08-07:
    /// `value('myvar',,'')` still reaches this path, because an *empty*
    /// third argument is a present one and the oracle answers a lookup in
    /// `.environment`; only an *omitted* third argument stays on 4c's own
    /// local-pool read/write. A crate that ignores the third argument
    /// answers the local pool's value instead -- a wrong answer, not a loud
    /// one -- which is worse than the gap this declares.
    ///
    /// **One argument, three destinations, and this constructor declares
    /// all three unimplemented without claiming which one a given call
    /// would have reached.** The oracle's own dispatch on the selector's
    /// value (`BUILTIN(VALUE)`, cited above) is an empty selector reading
    /// or writing `.environment`; the literal `'ENVIRONMENT'` reading or
    /// writing the OS environment; and anything else trying a
    /// platform-defined selector and then the registered value exit. Only
    /// the third of these is Phase 7's; the first is Phase 5's, the same
    /// environment/`.local` subsystem a `docs/superpowers/plans/
    /// phase-4-exclusions.txt` KNOWN GAP row already names, and the second
    /// is neither. That row carries the per-path attribution; this
    /// constructor does not repeat it.
    ///
    /// No owner string: like [`Loud::builtin_option_object`], this is a
    /// sub-case within a builtin the status table calls implemented, and
    /// `owned_message`'s shape belongs to the variant-keyed owner tables.
    ///
    /// [`Loud::builtin_option_object`]: Loud::builtin_option_object
    fn value_selector() -> Loud {
        Loud {
            message: "VALUE's external-selector form is not implemented".to_string(),
        }
    }

    /// A `PARSE` template trigger that needs an operand and has none.
    ///
    /// Not reachable from a program that parsed: `parse_template`
    /// (`rexx-parse`'s own `instruction.rs`) fills `value` for every kind but
    /// `End`, and refuses a trigger with nothing after it at parse time
    /// (38.901, measured). Kept as a `Loud` rather than a defaulted position
    /// or an `unreachable!` for the reason [`Loud::instruction`]'s own doc
    /// gives: a guarantee the grammar makes is not one the type system
    /// enforces, and an abort is the outcome the failing-loudly rule exists to
    /// exclude.
    ///
    /// [`Loud::instruction`]: Loud::instruction
    fn parse_trigger_operand() -> Loud {
        Loud {
            message: "a PARSE template trigger carries no position operand".to_string(),
        }
    }

    /// An activation's body selector named something that is not a routine
    /// body -- an internal inconsistency, never a program error.
    ///
    /// Unreachable through any program, since nothing constructs a
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

    /// A chunk's instruction map is shorter than the body it was compiled
    /// from -- an internal inconsistency, never a program error.
    ///
    /// `compile` writes one entry per instruction plus a final one, so the
    /// driver's lookup is in range for every instruction index the body has.
    /// Loud rather than an indexing panic for the reason [`Loud::instruction`]
    /// gives: an abort is precisely the outcome the failing-loudly rule
    /// exists to exclude, and a guarantee one function makes is not one the
    /// type system enforces at the other.
    ///
    /// [`Loud::instruction`]: Loud::instruction
    fn chunk_map_too_short() -> Loud {
        Loud {
            message: "a compiled chunk has no op for an instruction of its own body".to_string(),
        }
    }

    /// The driver reached an op it has no arm for.
    ///
    /// `what` names the op, so the message says which one rather than only
    /// that one was reached.
    fn op_not_driven(what: &'static str) -> Loud {
        Loud {
            message: format!("a compiled {what} op has no driver arm"),
        }
    }

    /// A compiled jump names an op past the end of the range it is running in
    /// -- an internal inconsistency, never a program error.
    ///
    /// A construct's own jumps stay inside (or exactly at the boundary of) the
    /// range that encloses it, because `block.rs` closes an inner branch's
    /// targets before the outer one's. Loud rather than left to the loop's own
    /// bound, which would read the escape as the range having completed
    /// normally and answer `Flow::Next` for a construct that never finished.
    fn jump_out_of_range() -> Loud {
        Loud {
            message: "a compiled jump leaves the range it is running in".to_string(),
        }
    }

    /// A register a branch op reads holds something that is not a Rexx
    /// logical value -- an internal inconsistency, never a program error.
    ///
    /// A register a branch op reads is written by the op that decided the
    /// branch, always as the small integer of the `bool` that op answered, so
    /// anything else in it says the two came apart. Loud rather than a
    /// defaulted answer, which would take a branch on a value nothing chose.
    fn register_not_logical() -> Loud {
        Loud {
            message: "a compiled branch read a register holding no logical value".to_string(),
        }
    }

    /// A compiled `SELECT` op does not describe the `SELECT` it was emitted
    /// for -- an internal inconsistency, never a program error.
    ///
    /// `ir::compile` emits these ops only while compiling a `Select` node and
    /// fills their instruction indices from that node, so reaching this means
    /// an op and the body it indexes came apart. Loud rather than a panic, for
    /// the reason [`Loud::instruction`] gives.
    ///
    /// [`Loud::instruction`]: Loud::instruction
    fn select_op_off_its_node() -> Loud {
        Loud {
            message: "a compiled SELECT op does not name a SELECT of its own body".to_string(),
        }
    }

    /// A compiled `DO`/`LOOP` op does not describe the loop it was emitted for
    /// -- an internal inconsistency, never a program error.
    ///
    /// [`Loud::select_op_off_its_node`]'s reasoning exactly, one construct over.
    ///
    /// [`Loud::select_op_off_its_node`]: Loud::select_op_off_its_node
    fn loop_op_off_its_node() -> Loud {
        Loud {
            message: "a compiled DO/LOOP op does not name a DO/LOOP of its own body".to_string(),
        }
    }

    /// A compiled `Store` op does not describe the assignment it was emitted
    /// for -- an internal inconsistency, never a program error.
    ///
    /// [`Loud::select_op_off_its_node`]'s reasoning exactly, one instruction
    /// over.
    ///
    /// [`Loud::select_op_off_its_node`]: Loud::select_op_off_its_node
    fn store_op_off_its_node() -> Loud {
        Loud {
            message: "a compiled Store op does not name an assignment of its own body".to_string(),
        }
    }

    /// A compiled op that runs or traces a call does not describe the call it
    /// was emitted for -- an internal inconsistency, never a program error.
    ///
    /// [`Loud::select_op_off_its_node`]'s reasoning exactly, shared by the ops
    /// that reach a call:
    ///
    /// * `Op::Call`, which `ir::compile` emits only for a `CALL` instruction's
    ///   `Named` form, so reaching it there means the op names an instruction
    ///   that is not a `CALL` at all, or one whose `CALL` is a form that stays
    ///   `Op::Generic`;
    /// * `Op::CallExpr`, whose `slot` and `path` reach no node of the clause,
    ///   or reach one that is not a call;
    /// * `Op::TraceFunction`, whose echo walks that same address to the node
    ///   it describes and finds nothing at the end of it.
    ///
    /// [`Loud::select_op_off_its_node`]: Loud::select_op_off_its_node
    fn call_op_off_its_node() -> Loud {
        Loud {
            message: "a compiled call op does not name a call of its own body".to_string(),
        }
    }

    /// A compiled `Const` op names a constant its own chunk does not carry --
    /// an internal inconsistency, never a program error.
    ///
    /// `ir::compile` interns every constant it emits an op for into the same
    /// chunk, so reaching this means the op and the table came apart. Loud
    /// rather than an indexing panic, for the reason [`Loud::instruction`]
    /// gives.
    ///
    /// [`Loud::instruction`]: Loud::instruction
    fn constant_out_of_range() -> Loud {
        Loud {
            message: "a compiled Const op names no constant of its own chunk".to_string(),
        }
    }

    // **There is no `Loud::parse`, and its absence is the fix.** A fragment
    // that does not parse raises the oracle's own 27.901 at rc 229, through
    // `impl From<&ParseError> for Raised` (`error.rs`), which `run_fragment`
    // uses. What the *top level* can and cannot take from it is written out
    // at `execute`'s own parse arm, below.
}

/// Names an expression form in **bounded** text, for a loud failure to quote.
///
/// The bound is the whole point and it is a contract, not a preference: this is
/// called on nodes this crate cannot evaluate, and those nodes carry arbitrarily large
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
        // go and implement. Each asks its own operator type for the spelling,
        // which is where the canonical bytes live and what the trace line for
        // that operator carries.
        ExprKind::Prefix { op, .. } => {
            return format!("the prefix operator `{}`", op.spelling());
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

/// Appends the owner phase to a loud message, `"{name} is not implemented
/// ({owner})"`, or leaves it unsuffixed (`"{name} is not implemented"`) when
/// [`instruction_owner`]/[`expr_owner`] answer `None` -- meaning "this crate
/// implements that variant", not "the owner is some particular phase".
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
/// `run.rs` (`run_loop_with_header`'s `DO`/`LOOP` COUNTER/`DO WITH` check,
/// and its stem-target `DO OVER` deviation) where the outer `InstructionKind`/
/// `ExprKind` is implemented but the specific reason that call happened is
/// not. Printing an owner there would read as self-contradictory -- the
/// construct plainly *is* implemented -- so this leaves the message
/// without a suffix on that path, and is the only
/// reason this function exists rather than a bare `format!` at each of the
/// two call sites. `run/tests.rs`'s `do_with_takes_the_loud_path`,
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

/// The gap a `::` directive declares at install time, or `None` for one this
/// crate can install.
///
/// **The predicate is what installing the directive *does*, and it has three
/// clauses**: a directive is a gap here when installing it runs code, changes
/// a package setting a Phase 4 construct can read, or resolves a name against
/// a table this crate does not have. `Interp::install_directives`' own doc
/// carries the measured transcript for every form on both sides of the line,
/// and `phase-4-exclusions.txt`'s directive section carries the two arms that
/// deliberately over-refuse and why.
///
/// A directive that installs cleanly is **ignored**, not implemented: 4c has
/// no object model, so a `::CLASS` that names nothing and a `::METHOD` with a
/// body are unreachable from any construct this phase runs. That is why they
/// are not a gap -- a program containing one and never using it produces the
/// oracle's own bytes.
fn directive_gap(kind: &DirectiveKind) -> Option<Loud> {
    let gap = |name: &str, owner: &'static str| {
        Some(Loud {
            message: owned_message(name, Some(owner)),
        })
    };
    match kind {
        // Loads a shared library and binds an entry point in it, before
        // `main` and whether or not the routine is ever called -- measured,
        // 98.903 rc 158 with stdout empty in both shapes. Phase 7 owns
        // library loading.
        DirectiveKind::Routine(routine) if routine.external.is_some() => {
            gap("::ROUTINE EXTERNAL", "Phase 7")
        }
        DirectiveKind::Method(method) if method.external.is_some() => {
            gap("::METHOD EXTERNAL", "Phase 7")
        }
        DirectiveKind::Attribute(attribute) if attribute.external.is_some() => {
            gap("::ATTRIBUTE EXTERNAL", "Phase 7")
        }
        // Loads a file and **runs its prolog** before `main`: measured, a
        // helper whose first clause is `say 'PROLOG RAN'` prints that line
        // above the requiring program's own output at rc 0. So presence is
        // use, and this refuses a program the oracle runs whenever the
        // prolog happens to be empty -- the trade `phase-4-exclusions.txt`
        // states, taken because the alternative is silently dropping both
        // that output and the public routines the file imports.
        DirectiveKind::Requires(_) => gap("::REQUIRES", "Phase 5"),
        // Applies package settings unconditionally, and there is no unused
        // form: measured, `::options digits 12` makes `digits()` report 12,
        // and `::options trace labels` makes every `::ROUTINE` in the file
        // emit its own `>I>`/`<I<` pair.
        DirectiveKind::Options(_) => gap("::OPTIONS", "Phase 5"),
        // Resolves a class name against the environment, which 4c has no
        // table for -- so this cannot tell `subclass object` (rc 0 on the
        // oracle) from `subclass zzznotaclass` (98.909 rc 158) and refuses
        // both.
        DirectiveKind::Class(class)
            if class.subclass.is_some()
                || class.metaclass.is_some()
                || !class.inherit.is_empty() =>
        {
            gap("::CLASS naming another class", "Phase 5")
        }
        // Resolves its target against the accumulated package: measured,
        // `::annotate routine nosuchrtn` is 99.945 rc 157. `::ANNOTATE
        // PACKAGE` names nothing and is ignored with the rest.
        DirectiveKind::Annotate(annotate)
            if !matches!(annotate.target, AnnotationTarget::Package) =>
        {
            gap("::ANNOTATE naming a target", "Phase 5")
        }
        DirectiveKind::Annotate(_)
        | DirectiveKind::Attribute(_)
        | DirectiveKind::Class(_)
        | DirectiveKind::Constant(_)
        | DirectiveKind::Method(_)
        | DirectiveKind::Resource(_)
        | DirectiveKind::Routine(_) => None,
    }
}

/// Who is responsible for an `InstructionKind` this crate does not implement,
/// spelled exactly as the split table spells it
/// (`docs/superpowers/specs/2026-07-30-phase-4a-executor-design.md`, "The
/// split") -- `None` for a variant this crate implements (see
/// [`owned_message`]'s doc for why that is `None` and not a `"4a"` string).
///
/// **A third copy of `tests/owners.rs`'s `INSTRUCTION_TAGS`, separate by
/// construction**: production code cannot depend on anything under
/// `tests/`, so the two cannot be merged the way `coverage.rs` and
/// `loud.rs` were. Separate, but not unchecked, and the difference is worth
/// stating because it decides how much care an edit here needs: a variant
/// that moves in scope or changes owner must be edited in both places, and
/// `loud.rs`'s `every_out_of_scope_variant_fails_loudly` fails if it is
/// not. That test requires each witness's emitted message to end with the
/// owner `owners.rs` records for that witness's tag, read out of the table
/// rather than restated in `loud.rs`, so this match is compared against
/// `owners.rs` itself. `owners.rs`'s own module doc names this function as
/// the fifth of its five pinned items.
///
/// **The comparison reaches the `Some` arms only, and almost nothing covers
/// the rest.** `Loud::instruction` is not called for a variant this crate
/// executes, so a phase written onto one is data no path reads: measured,
/// giving `InstructionKind::Say` an owner leaves the whole workspace suite
/// green. Nothing catches that and nothing needs to, the value being
/// unreachable -- but the reason is unreachability, not some other test
/// standing guard, and an edit here should not expect one to.
///
/// The exception is `Do`/`Loop`, because `run_loop_with_header` reaches this
/// function for them through the two edge cases described below, where the
/// instruction is implemented and only the specific reason is not. Measured:
/// giving that arm an owner turns `run/tests.rs`'s `do_with_takes_the_loud_path`,
/// `do_counter_takes_the_loud_path_regardless_of_which_other_kind_it_rides_on`,
/// `do_over_a_stem_target_takes_the_loud_path` and
/// `do_over_a_parenthesised_stem_target_is_also_caught` red, all four
/// asserting the exact unsuffixed message.
///
/// **Arm-grained for `InstructionKind::Call` and `InstructionKind::Address`,
/// matching `owners.rs`'s own `split` sections row for row.** Three of
/// `rexx_parse::Call`'s four arms are implemented and answer `None`, and
/// `Call::Qualified` is genuinely Phase 5's (a namespace-qualified `CALL`,
/// mirroring `ExprKind::QualifiedCall`'s own ownership below); `ADDRESS`
/// splits on whether the instruction carries a command or a `WITH`
/// redirection, which its own arm below writes out. The two splits differ in
/// shape because the language does: `Call`'s forms are enum arms and
/// `Address`'s are struct fields, so one matches a pattern and the other a
/// `bool`.
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
        | InstructionKind::Interpret { .. }
        // A `RETURN` in the main body is not a gap either: measured, it ends
        // the program with its value exactly as `EXIT` does.
        | InstructionKind::Return { .. }
        | InstructionKind::Nop => None,
        // **Arm-grained, and three of the four arms are `None`.**
        // `Call::Named`, `Call::Dynamic` and `Call::Trap` are all implemented,
        // so any owner string on them would be a false statement in a
        // table whose only job is to be true -- `Loud::instruction` is not
        // reached for any of the three, and an owner string nothing
        // reads is exactly how the third copy of this data drifts. A named
        // call that resolves to no internal label, no builtin and no
        // `::ROUTINE` raises the oracle's own 43.1 rather than failing
        // loudly; the one step behind those three that this crate skips is
        // the external file search, which is Phase 7's. So there is no
        // residual claim on the `CALL` keyword here at all.
        InstructionKind::Call(call) => match &**call {
            rexx_parse::Call::Named { .. }
            | rexx_parse::Call::Dynamic { .. }
            | rexx_parse::Call::Trap(_) => None,
            rexx_parse::Call::Qualified { .. } => Some("Phase 5"),
        },
        // `Use` is `None` even
        // though `USE LOCAL` can only ever fail here: it fails with the
        // oracle's own two errors (98.993/99.910), measured, which is an
        // implemented instruction answering the same bytes the oracle
        // answers -- not a gap. The one shape inside `Procedure` this crate
        // cannot express, `expose a.1`, fails loudly through
        // `Loud::compound_expose` rather than through this table, because it
        // is a sub-case of a variant and this table is per variant.
        InstructionKind::Procedure { .. } | InstructionKind::Use(_) => None,
        // All three `Signal` arms are implemented, so unlike
        // `Call` above this one needs no arm-grained match. `RAISE` is
        // likewise whole: its one shape that still fails loudly, `ADDITIONAL
        // (a, b)`, does so through `ExprKind::List`'s own `Phase 5` owner --
        // a sub-case of an *expression*, reported where that expression is,
        // not a residual claim on the `RAISE` keyword.
        InstructionKind::Signal(_) | InstructionKind::Raise(_) => None,
        // Both keywords are whole: `queue.rs`
        // stores every line either writes, and neither has a shape this
        // crate cannot express the way `Procedure`'s `expose a.1` does.
        InstructionKind::Push { .. } | InstructionKind::Queue { .. } => None,
        // All three spellings of the one instruction: `PARSE`, and the `ARG`
        // and `PULL` short forms, which `rexx-parse` gives variants of their
        // own but which share `exec_parse`. Every source is implemented,
        // including the two that read a line (`PARSE PULL`, `PARSE LINEIN`).
        InstructionKind::Parse(_) | InstructionKind::Arg(_) | InstructionKind::Pull(_) => None,
        // **Arm-grained, the second variant in this match that is.** The three
        // forms that only name an environment -- `ADDRESS env`, `ADDRESS VALUE
        // expr` and the bare toggle -- are implemented and answer `None`.
        // `ADDRESS env command` issues a command, and a `WITH` redirection
        // configures where a command's streams go; both need the command
        // dispatch `InstructionKind::Command` needs, so they carry that same
        // owner rather than one of their own (D18).
        //
        // The condition is a `bool` and not a pattern because these are struct
        // fields rather than enum arms; `owners.rs`'s `split` section for
        // `Address` matches on the identical expression, which is what keeps
        // the two tables comparable row for row.
        InstructionKind::Address(address) => {
            if address.command.is_some() || address.io.is_some() {
                Some("Phase 7")
            } else {
                None
            }
        }
        // A message send as a whole clause: `exec_message` (`run.rs`)
        // evaluates it and settles `RESULT`. `None` in the same sense
        // `DotVariable` is `None` below -- the variant is implemented, and
        // the sub-cases with no code here fail loudly through `Loud::
        // receiver_class`/`Loud::native_method` rather than answering.
        InstructionKind::Message { .. } => None,
        InstructionKind::Expose { .. }
        | InstructionKind::Options { .. }
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
        // **`ExprKind::Call` is `None`, not an owner string.** It has
        // exactly two `CallTarget` forms and this crate evaluates both --
        // unlike `InstructionKind::Call`, which stays arm-grained because
        // `Call::Qualified` is loud, this variant has
        // no later-phase arm hiding inside it, so it closes outright. A
        // name that resolves to no internal label (or a `CallTarget::
        // Literal`, which never searches labels at all), no builtin and no
        // `::ROUTINE` raises the oracle's own 43.1 -- exactly the same shape
        // `InstructionKind::Call`'s own comment above describes for `CALL`,
        // with the external file search behind those three being Phase 7's.
        // `>name`/`<name` decays to the referenced
        // variable's value in every ordinary position (measured, `say >p`
        // prints `p`'s value), and its one load-bearing use, as the argument
        // half of `USE ARG >name`, is handled at the call site by
        // `run.rs`'s `eval_argument` rather than here.
        //
        // **`ExprKind::Message` is `None` too**, for the reason its
        // `InstructionKind` twin above is: `dispatch.rs` resolves and invokes
        // the send, and what it has no code for is loud.
        ExprKind::Call { .. } | ExprKind::VariableReference(_) | ExprKind::Message { .. } => None,
        ExprKind::QualifiedCall { .. } | ExprKind::ClassResolver { .. } | ExprKind::List(_) => {
            Some("Phase 5")
        }
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
    slots: &'a [Option<usize>],
    /// The upfront pass's answers about **this** body: its clause indents
    /// (`Plan::indents`), the line each of its clauses sits on
    /// (`Plan::lines`), and how each of its compound names splits
    /// (`Plan::compounds`).
    ///
    /// A field here for the same reason `slots` is one: an `INTERPRET`
    /// fragment runs its own instruction list while the activation's plan
    /// still describes the enclosing body, so neither table can be read back
    /// off the activation without being the wrong body's.
    ///
    /// **`None` means there is no upfront pass to consult**, and each reader
    /// falls back to computing its own answer -- what `printed_indent`,
    /// `Interp::clause_line_at` and `Code::compound` each did before their
    /// own table existed. An `INTERPRET`
    /// fragment is that case, and it is `None` rather than the fragment's own
    /// local plan on purpose: `Interp::fragment_plan` does build one, but its
    /// slot numbers are local to the fragment and only its returned
    /// translation means anything in the enclosing frame, so handing the local
    /// plan over here would put a table whose slots are local beside a `slots`
    /// map whose slots are not.
    ///
    /// **`symbols` and this field come from the same body**, which is what
    /// makes indexing `Plan::compounds` by an id out of `symbols` mean
    /// anything at all -- `SymbolId::index`'s own doc comment has what
    /// happens when an id and a table are mismatched, and the quiet case is
    /// the one to design against.
    plan: Option<&'a Plan>,
}

impl<'a> Code<'a> {
    /// The slot this body's upfront pass bound `id` to, or `None` when it
    /// bound it none.
    ///
    /// **An indexed load where this was a `HashMap` lookup.** `SymbolId` is a
    /// dense, table-local, zero-based index (`SymbolId::index()`), and
    /// `Plan::by_symbol` is sized by the table it belongs to, so the id the
    /// AST already carries addresses the answer directly. What that removes is
    /// not the resolution -- the plan did that once, upfront -- but the
    /// `SipHash` of a `u32` that reading the answer used to cost, on a path
    /// every compiled read and write goes through.
    ///
    /// **`None` is safe and a wrong `Some` is not**, which is what the
    /// tripwire below checks and why it checks in one direction only. A `None`
    /// falls through to `Interp::slot_of`, which resolves the name and answers
    /// the same number. A `Some` is used as-is: were it another symbol's slot,
    /// every read and write of this name would silently address another
    /// variable, and no program that does not already know the answer could
    /// notice. `Plan::bind` fills this table and `Plan::names` from one
    /// `slot_for` call, so the name map is an independent recomputation of the
    /// same fact rather than a second copy of this one.
    ///
    /// The check is one-directional because the reverse does not hold and its
    /// failing is not a defect: `Plan::note_compound_name` puts a stem and its
    /// variable tail pieces in `names` without binding their ids here, so a
    /// name can carry a slot that no symbol's entry does.
    pub(crate) fn slot_for(&self, id: SymbolId) -> Option<usize> {
        let at = self.slots.get(id.index()).copied().flatten();
        debug_assert!(
            at.is_none()
                || self.plan.is_none_or(|plan| {
                    plan.names.get(self.symbols.name(id).as_bytes()).copied() == at
                }),
            "the plan's slot for {} is {at:?}, which its own name map does not agree with",
            self.symbols.name(id),
        );
        at
    }

    /// How the compound `id` names splits, from the upfront pass when this
    /// body had one, and by splitting the interned spelling when it did not.
    ///
    /// The returned borrow is `'a` and not the borrow of `self`, so a caller
    /// can hold the pieces across the `&mut Interp` calls that read a
    /// variable piece's value.
    fn compound(&self, id: SymbolId) -> Option<&'a CompoundName> {
        self.plan?.compound(id)
    }

    /// The stem half of the compound `id` names, its trailing period
    /// included -- the name whose slot holds the stem object a tail is read
    /// out of or written into -- **and that slot, when the entry carries
    /// one**.
    ///
    /// The name and the slot come back together rather than from separate
    /// methods, so that a caller reaching for the name has to say what it
    /// does about the slot.
    /// `None` is the answer for a body with no plan, and for the entry
    /// `Plan::bind` records for a **compound**-shaped name, whose stem half is
    /// a name of its own that the plan never bound; it means "resolve it the
    /// ordinary way", which is what every stem accessor did before the slot
    /// was kept. A stem-shaped name has no stem half distinct from itself, so
    /// `bind` does record that one.
    fn stem(&self, id: SymbolId) -> (&'a [u8], Option<usize>) {
        match self.compound(id) {
            Some(entry) => (&entry.stem, entry.stem_at),
            None => (compound_parts(self.symbols.name(id)).0.as_bytes(), None),
        }
    }
}

/// A program's main body as a `Code`, carrying the plan the upfront pass
/// builds for it -- which is what `run.rs` hands every step.
///
/// Here rather than in each test module because a hand-written `Code` with
/// `plan: None` takes `Code::compound`'s and `printed_indent`'s fallbacks
/// instead of the tables, and so exercises the path only an `INTERPRET`
/// fragment reaches. A test about what a compound resolves to wants the
/// production path unless it says otherwise.
#[cfg(test)]
fn planned_code<'a>(program: &'a Program, plan: &'a Plan) -> Code<'a> {
    Code {
        body: &program.main,
        symbols: &program.symbols,
        slots: &plan.by_symbol,
        plan: Some(plan),
    }
}

/// Whether a variable read found a value or derived one from the name.
///
/// D16 requires the read path to answer this from the start rather than gain
/// it later: `SIGNAL ON NOVALUE` changes what an uninitialised read
/// does, and retrofitting a raise into the hottest path is what naming it here
/// prevents. `Interp::novalue_check`
/// (`run.rs`) is the reader D16 was holding it for,
/// and the retrofit that would otherwise have been needed never happened.
///
/// **Both producers matter and they are not the same code.** `Interp::read`
/// answers it for a simple variable and `Interp::stem_get` for a compound;
/// `Interp::read_stem`, the bare-stem read, deliberately answers nothing at
/// all, because measured, `say zstem.` under `SIGNAL ON NOVALUE` does not
/// trap where `say zstem.1` does.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum Novalue {
    Set,
    Unset,
}

/// The condition a running handler was entered for, kept so `RAISE
/// PROPAGATE` can re-raise it.
///
/// **The echo stack travels with it**, which is the part that is measured
/// rather than obvious. A trapped condition clears `failure_site` and
/// `failure_sites` (inherited item I11), so by the time a handler runs there
/// is nothing left to echo -- yet the oracle's report for a `RAISE
/// PROPAGATE` from inside that handler names the *original* raising clause,
/// not the `raise propagate` clause:
///
/// ```text
///      8 *-*   say 1/0          <- line 8 raised; line 12 propagated
///      3 *-* call fun
/// Error 42:  Arithmetic overflow/underflow.
/// ```
///
/// So the two fields are saved here at the moment they are cleared and put
/// back if `PROPAGATE` ever asks. Putting `site` back full is also what
/// keeps the `raise propagate` clause itself out of the report, since
/// `record_failure_at` is first-wins.
struct ActiveCondition {
    raised: Raised,
    site: Option<FailureSite>,
    sites: Vec<FailureSite>,
}

/// A condition waiting for the current clause to finish before its `CALL ON`
/// handler runs.
///
/// Carries only what the handler needs that cannot be looked up again at
/// delivery time: the condition's name is the trap-table key, and `rc` is the
/// `RAISE ERROR n`/`RAISE FAILURE n` argument, held as its rendered text
/// rather than as an `ObjRef` because the raising clause's temps frame is
/// popped long before the handler runs and a handle into it would be
/// unrooted. `set_sigl` already takes the same approach for the same reason.
struct PendingTrap {
    condition: Box<[u8]>,
    rc: Option<Vec<u8>>,
    /// `RAISE ... DESCRIPTION`'s rendered value, held for the same reason
    /// `rc` is: the raising clause's temps frame is long gone by the time
    /// the handler reads it back through `CONDITION('D')`.
    description: Option<Vec<u8>>,
    /// The activation this may be delivered to: the raising activation's
    /// **caller**, which is the one whose trap table matched.
    ///
    /// **A depth first, then an identity, and both changes came from
    /// measurement.** The original had no such field at all and delivered at
    /// the next clause boundary reached by *any* activation, which is a
    /// different thing the moment the raising clause goes on to call
    /// something else: `say 'a' one(1) two(2)`, with `one` raising, ran the
    /// handler inside `two` -- `SIGL` 8, the `two:` label's own line, against
    /// the oracle's 3 -- and printed it before the `SAY` rather than after.
    /// A depth closed that and left a second hole, because a depth is only
    /// unique while its activation is live: `call aa` then `call cc`, with
    /// `aa` raising, ran the handler inside `cc`, and a pending condition
    /// whose activation is unwound by an error the caller traps was delivered
    /// into the next routine where the oracle drops it. [`ActivationId`] is
    /// unique across a pop, which is what both holes needed.
    ///
    /// Pinned by `a_call_trap_waits_for_the_raising_clause_to_finish`,
    /// `a_pending_trap_is_delivered_when_the_trapping_clause_is_a_return` and
    /// `a_pending_trap_whose_activation_is_gone_is_never_delivered`.
    activation: ActivationId,
    /// Whether this was queued by a **handler** running at a clause boundary
    /// rather than by that clause's own work.
    ///
    /// **It exempts exactly one thing: `Interp::in_clause`'s own tripwire.**
    /// That assertion reads a condition still waiting when a later clause
    /// begins as a construct having run an instruction without ending its
    /// header clause first, and for a trap the clause itself queued that is
    /// what it means. A trap queued *during* a delivery is not: the boundary
    /// running that delivery drains only the entries that were queued when it
    /// began, so this trap lands beyond the prefix that boundary owes, and the
    /// oracle defers it to the next boundary too. Measured with no construct
    /// anywhere in the program --
    /// `zq = raiser()` on line 3 whose handler itself raises a second trapped
    /// condition -- the oracle prints `after` and then the second handler's
    /// `SIGL` 4, and this crate agrees; the assertion fired on it regardless.
    ///
    /// Nothing else reads it. Delivery order, the identity check and `SIGL`
    /// are all unchanged by it.
    queued_during_delivery: bool,
    /// [`Interp::fragment_depth`] as it stood when this was queued: which
    /// `INTERPRET` fragment, if any, was running.
    ///
    /// **A condition is delivered only at a boundary reached at the same
    /// depth**, which is this crate's stand-in for the oracle running a
    /// fragment in an activation whose condition queue is its own. Both
    /// directions are measured, on a `CALL ON USER` handler that requeues a
    /// second trapped condition:
    ///
    /// * queued *before* the fragment, `interpret 'do; say ''body''; end'`:
    ///   the oracle prints `body` and then the second handler, so the
    ///   fragment's own clauses offer that condition no boundary and the
    ///   enclosing `INTERPRET` clause's boundary is where it lands;
    /// * queued *inside* the fragment, `interpret 'zq = raiser(); do; say
    ///   ''body''; end'`: the oracle prints the second handler and *then*
    ///   `body`, so a fragment clause's boundary does take it.
    ///
    /// One field answers both, where suppressing the boundary itself answers
    /// only the first and moves the second handler after `body`.
    fragment_depth: usize,
}

/// The interpreter. Owns the heap, the root set, the activation stack, the
/// plan cache and the two sinks, and **does not own the AST**.
struct Interp {
    heap: Heap,
    roots: RootSet,
    /// A buffer lent out for building a compound's tail key, and handed back.
    ///
    /// **A tail key is built, read once, and dropped**, once per compound
    /// reference. Measured with `heaptrack` on `samples/rexxcps.rex`, whose
    /// inner loop references `acompound.key1.loop`, lending this buffer
    /// removed 1,939,997 allocations of 8 bytes and 1,119,998 of 16 -- a key
    /// built by pushing into an empty `Vec` takes both, growing through the
    /// first capacity into the second.
    ///
    /// **Lent and returned rather than borrowed in place**, because every
    /// caller uses the key while calling back into `&mut self` -- to read a
    /// stem, to set one -- which a live borrow of a field would forbid.
    /// [`Interp::take_key_buffer`] moves it out and
    /// [`Interp::give_key_buffer`] moves it back.
    ///
    /// **Losing it is safe and costs only the reuse.** A caller that returns
    /// early between the two leaves this empty, and the next taker allocates a
    /// fresh one; nothing observes the difference. That property is what makes
    /// this sound under nesting too, which is why no reentrancy guard is
    /// needed: an inner build gets its own buffer rather than corrupting an
    /// outer one.
    key_buffer: Vec<u8>,
    /// A buffer lent out for a builtin call's evaluated argument values.
    ///
    /// **Every builtin call allocated one of these and freed it on the way
    /// out.** Measured with `heaptrack` on `bench-programs/strings.rex`, whose
    /// loop makes four builtin calls, that was eight of the ten allocations
    /// the program made per iteration -- this buffer and a second one, since
    /// the arguments were built as `Argument`s first and copied into values to
    /// hand over. The builtin path builds the values directly now, so this is
    /// the only one left.
    ///
    /// **Lent and returned rather than borrowed in place**, for
    /// [`Interp::key_buffer`]'s reason: the argument loop calls back into
    /// `&mut self` for every argument it evaluates, which a live borrow of a
    /// field would forbid. Losing it is safe and costs only the reuse -- a
    /// call that returns early through `?` leaves this empty and the next
    /// taker allocates.
    value_buffer: Vec<Option<ObjRef>>,
    /// Where an inline string's bytes are put so that [`Interp::to_text`]
    /// can hand back a borrow of them.
    ///
    /// **A `Cow::Owned` here would undo the encoding it serves.** An inline
    /// string is the commonest value there is, and `to_text` is how most of
    /// them are read; allocating a `Vec` per read would give back everything
    /// carrying the bytes in the handle saves.
    ///
    /// **One slot is enough because `to_text` takes `&mut self`.** The borrow
    /// it returns holds the interpreter exclusively, so no second call can
    /// run to overwrite this while the first result is live -- the compiler
    /// enforces the invariant rather than a convention doing it. A reader
    /// that wants two values' bytes at once goes through `render`, which
    /// copies into its own `Rendered` and never touches this.
    text_scratch: [u8; rexx_core::INLINE_TEXT],
    /// A buffer lent out for building a builtin's result, and handed back.
    ///
    /// **The same lending as [`Interp::key_buffer`], for a waste of the same
    /// shape.** A builtin builds its answer into an owned `Vec` and hands it
    /// to `text_owned`, and `Bytes::from_vec` copies anything of
    /// `INLINE_BYTES` or less into the object's own inline buffer and drops
    /// the `Vec`. So every short result allocated, filled, copied, and freed.
    /// Measured with `heaptrack` on `samples/rexxcps.rex`, whose inner loop
    /// asks for `substr` and `word`: the 1, 2, 3 and 7-byte widths together
    /// were about a quarter of every allocation the interpreter made.
    ///
    /// Losing it is safe and costs only the reuse, exactly as for the key
    /// buffer: a builtin that returns early through `?` between the take and
    /// the give leaves this empty and the next taker allocates.
    ///
    /// **A `Cell` where the key buffer is a plain field**, and that is not a
    /// style choice. A builtin reads its argument's bytes out of the heap
    /// first, which borrows the `Interp`, and then wants the buffer -- so a
    /// take that needed `&mut self` would be refused while that borrow is
    /// live. `Cell::take` needs only `&self` and leaves the default behind,
    /// which is exactly the lending this wants.
    result_buffer: std::cell::Cell<Vec<u8>>,
    activations: Vec<Activation>,
    /// The next [`ActivationId`] to hand out. Monotonic, never reset, never
    /// reused -- see that type for the two defects that needed an identity a
    /// stack depth could not supply.
    next_activation_id: u64,
    /// Every program the loader has issued an id for, indexed by that id.
    ///
    /// This is what makes a `ProgramId` a durable identity rather than a
    /// number that outlives its program: a plan cached under
    /// `BodyKey { program: ProgramId(0), .. }` stays correct because
    /// `ProgramId(0)`'s program is still here.
    programs: Vec<Rc<Program>>,
    plans: HashMap<BodyKey, Rc<Plan>>,
    /// Which engine `run_activation` runs a body on, from the
    /// [`Invocation`] `execute` was handed.
    ///
    /// On the interpreter rather than threaded through every call, exactly
    /// like `stress_collect` beside it: it is a property of the whole run,
    /// chosen once by the caller and never varying between activations.
    engine: Engine,
    /// The chunk cache (Phase 4e): D16's discipline applied to a second cache
    /// rather than invented afresh for it, under `plans`' own `BodyKey`
    /// **paired with the trace setting the chunk was compiled under**.
    ///
    /// The pair rather than the `BodyKey` alone because the setting is an
    /// input to compilation (D23), so one body has one plan and can have more
    /// than one chunk. See `Interp::chunk_for`, in `plan.rs`, for why a
    /// narrower key is a wrong-output defect.
    chunks: HashMap<(BodyKey, ChunkTrace), Rc<crate::ir::Chunk>>,
    /// How many times `chunk_for` has refused a body because it does not fit
    /// the index widths the compiled stream commits to (`ChunkTooLarge`) --
    /// never because a body contains a construct the compiler does not
    /// know, which does not exist (D21: every instruction compiles).
    ///
    /// One per refusal rather than one per body, because `chunks` holds only
    /// successes: a refused body is recompiled and refused again on every
    /// entry. `Interp::chunk_for`'s own doc says what stops this being a
    /// silent fallback to the tree-walker.
    chunks_refused: usize,
    /// Every `::ROUTINE` the running program installs, keyed by its
    /// **upcased** name and holding its index in `Program::directives`.
    ///
    /// Upcased on both sides, which is the lookup rule and not a convenience:
    /// measured, `::routine 'zork'` is reached by `call zork`, `call 'zork'`
    /// and `call 'ZORK'` alike, and `::routine MiXeD` by `call 'mixed'`.
    /// `CodeBody::labels` is the opposite -- a quoted target never searches it
    /// at all -- so the two tables cannot share a key rule.
    ///
    /// Filled by [`Interp::install_directives`] before the main body's first
    /// clause, matching the oracle, which resolves every directive at
    /// translation or install time: measured, a `::ROUTINE` naming a library
    /// it cannot load reports 98.903 with **empty stdout** whether or not the
    /// program ever calls it.
    routines: HashMap<Box<[u8]>, InstalledRoutine>,
    /// The object model message dispatch resolves against: `Setup.cpp`'s
    /// native classes, whatever `::CLASS`/`::METHOD`/`::ATTRIBUTE` have
    /// installed beside them, and the primitive methods this crate
    /// implements. `rexx-classes`' own
    /// [`rexx_classes::ClassRegistry`] is the one class model here (R9), not
    /// one of two.
    ///
    /// **`None` until something needs it**, because building the native set
    /// is measured at 5.5 ms and a program that neither declares a class nor
    /// sends a message must not pay it. [`Interp::classes`] and
    /// [`Interp::object_model`] (`dispatch.rs`) are the accessors that force
    /// it.
    object_model: Option<dispatch::ObjectModel>,
    /// `.environment`, `.local` and what `.context`/`.methods` are built from
    /// (D33). `None` until a `.NAME` is resolved, for the reason
    /// [`Interp::object_model`] is: building it forces the native class set.
    environment: Option<environment::EnvironmentModel>,
    /// The classes each program's own `::CLASS` directives installed, keyed by
    /// the uppercased name -- `PackageClass`'s installed-class table, which
    /// `.NAME` resolution consults ahead of `.local` and `.environment`.
    ///
    /// Per program rather than global, because that is what makes the first
    /// step of the order mean anything: two packages may each declare a class
    /// of one name.
    package_classes: HashMap<ProgramId, HashMap<Box<[u8]>, ObjRef>>,
    /// Which `(program, directive)` a [`rexx_classes::MethodId`] `install_directives`
    /// minted names -- the "bodies are stored" half of R9, addressed by the
    /// same identity `ClassRegistry::add_instance_method`/`add_class_method`
    /// already returns.
    ///
    /// Keyed rather than positional: the registry mints an id for every
    /// native method before a directive can install one, so a
    /// [`rexx_classes::MethodId`]'s own `u32` is not an index into the
    /// directives this program declared. [`Interp::record_method_body`] runs
    /// immediately after each mint a directive makes, and asserts the key is
    /// fresh.
    method_bodies: HashMap<MethodId, InstalledMethodBody>,
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
    /// `current_value_indent` and `current_clause_line`, bundled -- see
    /// `ClauseState`'s own doc comment for what the two share, the property
    /// that decides what belongs alongside them, and why they are one field
    /// rather than two.
    clause_state: ClauseState,
    /// A condition raised by `RAISE` whose `CALL ON` handler has not run yet.
    ///
    /// **Deliberately not part of `ClauseState`**, checked against that
    /// struct's own membership rule rather than placed by analogy: this is
    /// not set per clause by `step_in_temps_frame`, it is set once by a
    /// `RAISE` and *consumed* at the next clause boundary, so a nested
    /// activation overwriting it is not the hazard `ClauseState` exists to
    /// close.
    ///
    /// **The real hazard is delivery into the wrong activation, and it took
    /// two goes to close** (fix round 1's finding 7 corrects the sentence
    /// that used to assert it was already closed). Delivery happens at a
    /// clause boundary, on every path out of a clause -- a `RETURN` or an
    /// `EXIT` cannot skip it, because `Flow::Return` means the clause *was* a
    /// `RETURN`, not that it did not happen -- and it delivers only to the
    /// activation named by [`PendingTrap::activation`], an identity rather
    /// than a stack depth. The first property makes the condition reach its
    /// activation; the second stops it reaching anyone else's.
    ///
    /// **"Every clause" means every clause, and four rounds each named a set
    /// of sites that was short by one.** The lists are gone: the boundary is
    /// now inseparable from the clause's own line, in `clause.rs`'s
    /// [`Interp::in_clause`], whose closure body *is* the clause. Where the
    /// call sites are is a derived question rather than an enumerated one --
    /// the oracle's boundary sits after every instruction of the
    /// activation's flat list, and this crate diverges only where `IF`,
    /// `SELECT`, `DO`/`LOOP` and `INTERPRET` resolve other instructions
    /// inside their own `step`. `in_clause`'s own `debug_assert` is what
    /// makes a fifth such construct announce itself; `clause.rs`'s module doc
    /// has the rule, what the shape does and does not guarantee, and the
    /// residual.
    ///
    /// **A queue, drained at a boundary in the order the conditions were
    /// queued.** The oracle takes everything a clause left pending, not one:
    /// measured, `zr = ra() + rb()` with both trapped runs `ra`'s handler and
    /// then `rb`'s, both reporting the raising clause's own line, and the
    /// three-condition version runs all three in that same order. A single
    /// slot answered the last one only and lost the rest outright.
    ///
    /// **What a handler queues while it runs is owed to the next boundary,
    /// not this one.** Measured over a two-pass loop whose body requeues: the
    /// oracle defers the requeue past the boundary that delivered it and takes
    /// it at the following clause -- which is what [`PendingTrap::
    /// queued_during_delivery`] records, and why the drain is bounded to the
    /// entries present when the boundary began.
    ///
    /// **Entries for another activation are stepped over, not blocking.**
    /// Each activation owns its own queue in the oracle; here they share one
    /// and [`PendingTrap::activation`] is what separates them.
    ///
    /// **One shape in this family has no oracle answer:** a clause queuing the
    /// same condition name twice segfaults the oracle once the drain reaches
    /// the second copy. `corpus/oracle-crashes.txt` carries the program and
    /// the warning; nothing here may be described as agreeing with the oracle
    /// on it.
    pending_traps: VecDeque<PendingTrap>,
    /// The condition whose handler is running, for `RAISE PROPAGATE` to
    /// re-raise.
    ///
    /// Set when a trap fires and never cleared -- `run.rs`'s own
    /// `exec_raise_propagate` states what that costs and what is measured
    /// either side of it.
    active_condition: Option<ActiveCondition>,
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
    /// **This field means exactly one thing, and the mistake it invites is
    /// carrying an `INTERPRET` fragment's activation base here as well**, on
    /// the reasoning that both are "an indent added on
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
    /// **`0` for every program with no fragment and no call**, which is what
    /// makes adding it at a site incapable of moving such a program's
    /// expectation: nothing but a fragment or a `CALL` ever sets it.
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
    /// **First-wins *within one level* (inherited item I11).** The
    /// early-return guard at the top of `record_failure_at` means the most
    /// specific clause at this level wins.
    ///
    /// **A trap is what clears it**, and it empties
    /// [`Interp::failure_sites`] alongside it: a trapped condition prints no
    /// report, so the sites it accumulated must not be printed against a
    /// later, untrapped one. `run.rs`'s `offer_to_trap` is the one place that
    /// clears either, and `a_second_raise_after_a_trapped_one_reports_its_
    /// own_site` is the transcript.
    failure_site: Option<FailureSite>,
    /// The levels that have already finished failing, innermost first --
    /// `Raised::report`'s echo stack minus its last entry.
    ///
    /// **Why two fields and not one `Vec`.** `failure_site` is the level
    /// currently unwinding and is first-wins; this is the record of levels
    /// already sealed. `run.rs`'s `seal_site_level` moves one into the other
    /// and is called by exactly the constructs that open a level --
    /// `run_fragment` and `Interp::invoke_call`. Keeping the two apart is
    /// what lets the guard stay a plain `is_none()` rather than a "did
    /// anything get recorded since the current level opened" watermark, and
    /// it is why the single-site behaviour falls out unchanged
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
    /// How many `INTERPRET` fragments are running, counted from zero outside
    /// any of them.
    ///
    /// **The key a clause boundary matches a queued condition against**
    /// ([`PendingTrap::fragment_depth`], which has the transcripts): the
    /// oracle runs a fragment in an activation whose condition queue is its
    /// own, so a condition queued outside the fragment gets no boundary
    /// inside it and one queued inside gets no boundary outside.
    ///
    /// A depth rather than a "some fragment is running" flag, because a
    /// fragment inside a fragment is a third queue again. Measured against a
    /// built flag design, with a condition queued in the **outer** fragment and
    /// an inner fragment running while it waits: `interpret 'zq = raiser();
    /// interpret "say 1; say 2"; say 3'` prints `1`, `2`, the second handler,
    /// then `3` on the oracle and here, where the flag delivers after `1`.
    /// `ir_dual_cases/interpret-condition-queue` holds that program, and beside
    /// it the neighbouring shape that does **not** separate the two designs.
    ///
    /// Separate from `clause_line_override`, which the `Interpret` arm sets
    /// beside it: that one is *inherited* by a nested fragment (the oracle
    /// prints the outermost `INTERPRET`'s line for every clause inside), where
    /// this one has to distinguish the nesting levels it deliberately
    /// flattens. Not saved across a call either, where `clause_line_override`
    /// is cleared: a callee's clauses run in the fragment that called them,
    /// and a condition the callee raises is owed to the caller's own next
    /// boundary, which is inside that fragment.
    fragment_depth: usize,
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
    /// The arena size at which [`Interp::alloc_with`] collects, and half of
    /// this crate's trigger policy. The other half is `Heap::will_grow`.
    ///
    /// **Collect when the arena is about to grow AND it has reached this
    /// many slots; afterwards set it to twice what survived, floored at
    /// [`COLLECT_FLOOR`].** The two halves answer different questions and
    /// both are needed:
    ///
    /// * **`will_grow` is the pressure event.** It is the moment the process
    ///   would ask for more memory, which is what the oracle triggers on --
    ///   see that method's own doc for why its allocation-failure signal does
    ///   not port and this branch is the analogue that does. While swept
    ///   slots remain there is nothing to gain by collecting again, and this
    ///   is what stops it happening.
    /// * **This watermark is the growth allowance.** A fresh heap has no free
    ///   list, so `will_grow` alone would collect on every allocation
    ///   forever, finding nothing. Doubling it from the survivors is the
    ///   oracle's `adjustMemorySize` step: the collection's own result sets
    ///   the next threshold rather than a constant somebody picked.
    ///
    /// Three properties come out of the pair, and nothing else was asked of
    /// it:
    ///
    /// * **The peak is bounded by the live set rather than by the program's
    ///   total allocation.** Before this existed nothing collected at all, so
    ///   a loop's peak resident set was everything it had ever allocated --
    ///   `strings.rex` reached 2.5 GB with a live set of a few values.
    /// * **The collector's total work is bounded by a constant times the
    ///   program's total allocation.** Between two collections the program
    ///   must allocate at least as many objects as the first one left alive,
    ///   so a marking pass over `n` survivors is paid for by `n` allocations,
    ///   whatever `n` is. That is what keeps a program with a genuinely large
    ///   live set -- `alloc4c.rex`'s growing compound table -- from
    ///   re-marking it on a schedule the live set itself sets.
    /// * **A transient does not go on being paid for.** Measured, a program
    ///   building 200,000 live tails, dropping them, then making 2,000,000
    ///   short-lived values: 11 collections against the 35 a watermark on the
    ///   *live count* alone performs, for the same peak resident set and the
    ///   same output. The count-based form collects while thousands of swept
    ///   slots sit unused, because live has fallen back to the floor.
    ///
    /// **The blind spot, named rather than left to be discovered: this counts
    /// slots, not bytes.** A slot is 96 bytes of arena; the object's payload
    /// -- a string's bytes, a `Number`'s digit vector, a stem's map -- is a
    /// separate `malloc` the heap does not size. So a few very large strings
    /// are few slots and a great deal of memory, and neither half of this
    /// policy reacts to them. Byte accounting would need payload sizes
    /// `Body` does not carry today.
    ///
    /// It is a **policy** and policies invite tuning, so the numbers in it
    /// are fixed and stated rather than searched: a threshold adjusted until
    /// a benchmark number looked right would not survive a different
    /// workload.
    collect_at: usize,
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
    /// Whether the instruction about to be stepped is allowed to be a
    /// `PROCEDURE` -- and, read the other way, whether it is the first
    /// instruction executed in its activation.
    ///
    /// **Set only by `run_activation`, and taken at the top of `step`.**
    /// That pairing is the whole mechanism, and it is what makes the
    /// permission stop at exactly one instruction: any nested stepping --
    /// an `INTERPRET` fragment, an `IF`/`SELECT` branch through
    /// `run_bounded` -- reaches `step` again after the outer `step` has
    /// already taken the flag, so it sees `false` without any of those paths
    /// having to know this field exists. Measured, and the reason it is
    /// taken on the way in rather than cleared on the way out: `sub:
    /// interpret "procedure"` is error 17.1, so a fragment must not inherit
    /// its host clause's permission.
    ///
    /// A field rather than a parameter for the reason `current_value_indent`
    /// gives for the same choice: `step` is reached from several callers
    /// that have no business knowing about `PROCEDURE`.
    procedure_permitted: bool,
    /// The call that entered the running activation: what `USE ARG` reads.
    ///
    /// **Saved and restored around every call, alongside the four pieces of
    /// level state `Interp::invoke_call` already saves.** That is the same
    /// discipline Task 4's own review finding was about -- a fifth piece of
    /// per-activation state added without a restore is invisible until two
    /// activations per clause are reachable, and then wrong. Everything a
    /// call sets here is set in one place and put back in one place.
    ///
    /// On `Interp` and not on `Activation` because it is filled *before* the
    /// callee's activation exists: the arguments are evaluated in the caller,
    /// which is where the argument expressions' own variables live.
    ///
    /// **The top-level program is a call too, and its caller is the command
    /// line.** `execute` fills this from the [`Invocation`] before running the
    /// main body, with the program's own path as the name and the command
    /// line's one argument string -- or nothing at all, when there was no
    /// argument. Measured, `use arg p` as a program's own first clause: with
    /// no argument `p` reads as `P`, with `rexx p.rex hello` it reads as
    /// `hello`, and with `rexx p.rex ""` it reads as the null string. So an
    /// empty context here is the right answer for one specific invocation and
    /// the wrong answer for the other two.
    call_context: CallContext,
    /// The in-process external data queue (I15): every line
    /// `PUSH`/`QUEUE` has written and `PULL`/`PARSE PULL` have not yet
    /// removed. See `queue.rs`'s own module doc for the LIFO/FIFO split, and
    /// `Interp::pull_line` (`input.rs`) for the queue-first-then-`.input`
    /// rule that reads it.
    queue: Queue,
    /// `.input`'s position: the one line cursor `PULL`, `PARSE PULL` and
    /// `PARSE LINEIN` all advance. See `input.rs` for the shared-position
    /// measurement and the line rule; `ProgramInput`'s own doc has why the
    /// process's real standard input is never what this holds by default.
    input: Input,
    /// `RANDOM`'s generator state: the seed the next call will scramble, or
    /// `None` before any call has drawn one.
    ///
    /// **One field for the whole interpreter, where the oracle keeps it per
    /// activation and forwards.** `RexxActivation::getRandomSeed`
    /// (`execution/RexxActivation.cpp:3453`) opens with `if
    /// (isInternalLevelCall()) return parent->getRandomSeed(seed)`, so an
    /// internal routine and an `INTERPRET` share their caller's stream rather
    /// than starting one of their own -- which is what
    /// `bif/RANDOM.testGroup`'s stream test needs, since it seeds once and
    /// then makes 99 further unseeded calls and requires the whole run to
    /// repeat. Holding it here reproduces that sharing directly.
    ///
    /// `None` rather than a fixed starting value, because the oracle's first
    /// unseeded number is genuinely process-random: measured, the same
    /// program's first `random()` was 894, 152 and 414 on three consecutive
    /// runs, while every number *after* a seeded call repeated exactly. So
    /// this is drawn once, from the clock and the process id, and never from
    /// a constant that would make an unseeded program reproducible where the
    /// oracle's is not.
    random_seed: Option<u64>,
    /// `TIME('E')`/`TIME('R')`'s anchor: the clock reading (`builtin::
    /// datetime`'s microseconds-since-0001-01-01 unit) elapsed time is
    /// measured from, or `None` before any `E`/`R` call has run.
    ///
    /// **Lazily initialised to the first call's own reading, not to zero.**
    /// `RexxActivation::getElapsed` does the same lazy fill
    /// (`execution/RexxActivation.cpp:3424`), which is what makes a
    /// program's *first* `TIME('E')` or `TIME('R')` read exactly `0` rather
    /// than elapsed-since-process-start.
    ///
    /// **The reset itself is applied lazily, not immediately, matching the
    /// oracle rather than simplifying it.** `TIME('R')` does not overwrite
    /// this field the instant it runs; it sets [`pending_elapsed_reset`]
    /// instead, and `builtin::datetime::now_base_time`'s own cache-miss
    /// path is what actually moves this anchor, to the *stale* value still
    /// sitting in [`Activation::cached_clock`] from the clause the reset
    /// ran in -- exactly `RexxActivation::getTime`'s own order
    /// (`execution/RexxActivation.cpp:3400`-`3406`): capture the timestamp
    /// before overwriting it, then refresh. An earlier version of this
    /// applied the reset immediately and got exactly one measured case
    /// wrong: two `TIME('R')` calls inside **one** clause read the same
    /// value on the oracle (the reset has not taken effect yet when the
    /// second call reads the still-cached timestamp), where the immediate
    /// version read `0` for the second -- `builtin::datetime`'s own
    /// `time_r_resets_relative_to_the_last_reset_not_program_start` pins
    /// the corrected behaviour with a real burn rather than a fabricated
    /// clock.
    ///
    /// **One field for the whole interpreter, where the oracle keeps it per
    /// activation** (`ActivationSettings::elapsedTime`) -- the same
    /// divergence [`random_seed`]'s own doc takes, but **not** for the
    /// reason an earlier revision of this comment gave. The oracle does
    /// not reset this to zero on every `CALL`: `putSettings` copies the
    /// *whole* settings block into a callee by value
    /// (`execution/RexxActivation.cpp:225`, the same inheritance
    /// [`trace_mode`]/[`traps`] already document), so a callee's own
    /// `TIME('E')` reads the caller's own elapsed time correctly inherited
    /// -- and this one field reproduces exactly that, measured directly.
    /// The real, opposite divergence is the *write-back*: the oracle's own
    /// copy-in is guarded to never copy back out except for an `INTERPRET`
    /// fragment (`isInterpret()`, `:686`), so a callee's own reset dies
    /// with its frame there and leaks into the caller here, because both
    /// read and write the identical field. Measured:
    ///
    /// ```text
    /// zz=time('E'); call burn; call sub; say 'after' time('E')   [sub does n2 = time('R')]
    ///   oracle:  inside 0.725271  inside-after-R 0.000004  after 0.725387
    ///   crate:   inside 33.889432 inside-after-R 0.000005  after 0.000013
    /// ```
    ///
    /// `inside` (the callee's own inherited reading) matches; `after` (the
    /// caller's reading once the callee has returned) does not, because
    /// the callee's own `R` reset this field out from under the caller.
    /// `builtin::datetime`'s own
    /// `a_callees_own_time_r_leaks_into_the_caller_after_it_returns` pins
    /// this as a declared divergence rather than a silent one.
    ///
    /// [`random_seed`]: Interp::random_seed
    /// [`pending_elapsed_reset`]: Interp::pending_elapsed_reset
    /// [`Activation::cached_clock`]: crate::activation::Activation::cached_clock
    /// [`trace_mode`]: crate::activation::Activation::trace_mode
    /// [`traps`]: crate::activation::Activation::traps
    elapsed_anchor: Option<i64>,
    /// Whether a `TIME('R')` (or a clock read going backward) is waiting
    /// to move [`elapsed_anchor`] the next time the clock cache next
    /// refreshes -- `RexxActivation`'s own `elapsedReset` state flag
    /// (`execution/ActivationSettings.hpp:121`), consumed by
    /// `builtin::datetime::now_base_time`'s cache-miss path. See
    /// [`elapsed_anchor`]'s own doc for why the reset is lazy at all.
    ///
    /// [`elapsed_anchor`]: Interp::elapsed_anchor
    pending_elapsed_reset: bool,
    /// The running program's own location, as `PARSE SOURCE`'s third word.
    ///
    /// The same string `run_program` was handed and `Raised::report`'s
    /// position line prints -- absolute and dot-normalised, which is how the
    /// oracle spells it (`run_program`'s own doc has the measurement). Held
    /// here rather than derived from `programs` because a program's bytes
    /// cannot know where they came from, and filled by `execute`; every other
    /// construction path leaves it empty, so a unit test that wants a path
    /// sets one.
    program_path: String,
}

/// Where one installed `::ROUTINE` lives: which loaded program, and which of
/// its directives.
///
/// The program is carried rather than assumed to be the running one, because
/// the two spellings the resolution needs -- the `BodyKey` a plan is cached
/// under and the `Rc<Program>` the activation holds -- must name the same
/// program or a routine would run under another program's plan. Both come
/// from this one field, so they cannot come apart.
#[derive(Copy, Clone)]
struct InstalledRoutine {
    program: ProgramId,
    directive: usize,
}

/// Where one installed `::METHOD`/`::ATTRIBUTE` accessor's own body lives --
/// the same shape as [`InstalledRoutine`], for the same reason: a
/// [`rexx_classes::MethodId`] alone names neither the program nor the
/// directive, and both are needed to find the [`rexx_parse::CodeBody`]
/// (or its absence, for a generated accessor or an `ABSTRACT`/`DELEGATE`
/// method) later.
///
/// `#[allow(dead_code)]`: nothing outside this crate's own tests reads
/// either field yet, because reading one back needs a resolved message send
/// (Task 5's) to find the right entry and a method-body invocation (Task
/// 7's) to do anything with it. Both fields are read by
/// `tests::method_bodies_names_each_directive_in_minting_order`, so this is
/// not an unread struct, only one with no production reader before Task 7.
#[derive(Copy, Clone)]
#[allow(dead_code)]
struct InstalledMethodBody {
    program: ProgramId,
    directive: usize,
}

/// The name and arguments of one call in progress.
///
/// One struct rather than two `Interp` fields so that the save-and-restore
/// in `Interp::invoke_call` is a single `mem::replace`: two fields would be
/// two places to forget, which is precisely the defect shape this is
/// modelled to avoid.
#[derive(Default)]
struct CallContext {
    /// The resolved routine name, as errors 40.3 and 40.4 spell it --
    /// measured, `Not enough arguments in invocation of SUB2`, the label's
    /// own upcased spelling.
    ///
    /// At the top level it is the **program's own path**, not a label: with no
    /// argument supplied, `use strict arg p` as a program's first clause is
    /// measured as `Error 40.3: Not enough arguments in invocation of
    /// /abs/path/p.rex; minimum expected is 1.`, and `use strict arg` with an
    /// argument supplied is the matching 40.4 naming the same path.
    name: Vec<u8>,
    /// The arguments in source order, an omitted position (`call sub 1,,3`)
    /// left as `None` rather than closed up. Measured: that call into `use
    /// arg p, q, r` gives `[1] [Q] [3]`, so an omission holds its place.
    arguments: Vec<Option<Argument>>,
}

/// One evaluated call argument.
///
/// Two variants and not a bare `ObjRef`, because `USE ARG >name` needs
/// something an ordinary value cannot carry: which of the *caller's* slots
/// the argument named, so the callee's own variable can be aliased to it.
/// Measured -- `call sub2 >p` into `use arg >q` makes the callee's `q =
/// 'aliased'` visible as the caller's `p`, while the same call into a plain
/// `use arg q` merely copies the value.
///
/// `Reference` carries a value as well as a slot, and that is not
/// redundancy: a variable reference used as an ordinary argument **decays
/// to the referenced variable's value**, measured -- `say >p` prints `p`'s
/// value, and `call sub2 >p` into a plain `use arg q` binds that value.
/// So every argument has a value and only some have a slot.
///
/// `Reference` also carries the referenced variable's **name**, which is
/// there for two distinct jobs and neither is cosmetic. Its *shape* is the
/// reference's kind, and `USE ARG >name` refuses a kind mismatch -- a simple
/// reference into a stem target is error 88.929 and the reverse is 88.930,
/// measured. Its *text* is what those two errors substitute: measured with a
/// variable whose value differs from its name, `p = 'value-not-name'` passed
/// as `>p` into `use arg >q.` reports `found "P"`, the caller's name, where
/// 88.928 in the same position reports the argument's value. The two
/// families disagree about what they name, so the name has to be carried
/// rather than reconstructed from the value.
///
/// Not `Copy`, only `Clone`, because of that owned name. The one read site
/// (`exec_use_arg`) clones per target, which is one small allocation per
/// `USE ARG >` position and nothing at all for an ordinary argument.
#[derive(Clone)]
enum Argument {
    Value(ObjRef),
    Reference {
        target: SlotRef,
        value: ObjRef,
        /// The referenced variable's own spelling, upcased as the scanner
        /// interned it, including a stem's trailing period (`P`, `P.`).
        name: Box<[u8]>,
    },
}

impl Argument {
    /// The argument's value, which every form has. `USE ARG` without `>`
    /// and `ARG()` both want only this.
    fn value(&self) -> ObjRef {
        match self {
            Argument::Value(value) | Argument::Reference { value, .. } => *value,
        }
    }
}

impl Interp {
    /// **Every field a caller might want to vary starts at a fixed value
    /// here and is set after construction**, so this signature does not grow
    /// a parameter for each one. `stress_collect` and `engine` are both of
    /// that shape: `execute` sets each from what it was handed, and every
    /// other caller -- the unit tests throughout this crate, which is nearly
    /// all of them -- gets the value below.
    ///
    /// **`engine` starting at `TreeWalker` is therefore not the default
    /// engine**, which is [`Engine::DEFAULT`] and reaches an `Interp` through
    /// `execute`. What this value decides is the arm those unit tests run on,
    /// and it is the tree-walker: measured, by setting it to `Engine::Ir` and
    /// counting what the driver did -- 1 chunk driven and 2 clauses stepped
    /// from it against 0 and 0 -- with the whole suite green either way.
    fn new() -> Interp {
        Interp {
            heap: Heap::new(),
            roots: RootSet::new(),
            key_buffer: Vec::new(),
            value_buffer: Vec::new(),
            text_scratch: [0; rexx_core::INLINE_TEXT],
            result_buffer: std::cell::Cell::new(Vec::new()),
            activations: Vec::new(),
            programs: Vec::new(),
            plans: HashMap::new(),
            engine: Engine::TreeWalker,
            chunks: HashMap::new(),
            chunks_refused: 0,
            routines: HashMap::new(),
            object_model: None,
            environment: None,
            package_classes: HashMap::new(),
            method_bodies: HashMap::new(),
            out: Vec::new(),
            trace: Vec::new(),
            clause_state: ClauseState::new(),
            pending_traps: VecDeque::new(),
            active_condition: None,
            next_activation_id: 0,
            current_case_text: None,
            indent_offset: 0,
            activation_indent: 0,
            failure_site: None,
            failure_sites: Vec::new(),
            clause_line_override: None,
            fragment_depth: 0,
            stress_collect: false,
            collect_at: COLLECT_FLOOR,
            depth: 0,
            max_depth: 0,
            stack_entry: 0,
            stack_first: 0,
            stack_deepest: 0,
            procedure_permitted: false,
            call_context: CallContext::default(),
            queue: Queue::new(),
            // Nothing to read, which is what makes it impossible for a unit
            // test to reach the harness's own standard input: only `execute`
            // ever replaces this, and only from an `Invocation` that named a
            // source. `ProgramInput`'s own doc has the argument.
            input: Input::new(ProgramInput::Nothing),
            random_seed: None,
            elapsed_anchor: None,
            pending_elapsed_reset: false,
            program_path: String::new(),
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
        let program_id = ProgramId(self.programs.len());
        self.programs.push(Rc::clone(&program));

        // **Before the first clause, and its failures print nothing on
        // stdout.** That is the oracle's own shape rather than a choice
        // here: measured, `say 'main ran'` followed by `::requires
        // 'no_such_file_zz.rex'` is 43.901 at rc 213 with stdout EMPTY, and
        // `::class foo subclass zzznotaclass` is 98.909 at rc 158, likewise
        // empty. A program whose directives all install runs its main body
        // exactly as one with no directives does.
        self.install_directives(program_id, &program)?;

        // Note what does *not* happen here: the plan is looked up through
        // `&program.main`, a borrow of the local `Rc`, while `self` is
        // borrowed mutably by `plan_for`. Reaching the same body through
        // `self.programs[id.0].main` instead would be the `E0502` that
        // `run_activation` writes out.
        let plan = self.plan_for(
            BodyKey {
                program: program_id,
                directive: None,
            },
            &program.main,
            &program.symbols,
            &program.source,
        );

        let frame = self.roots.push_slots(plan.len());
        let id = self.next_activation_id();
        self.activations.push(Activation::new(
            id,
            Rc::clone(&program),
            program_id,
            plan,
            frame,
        ));

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

    /// Resolves every `::` directive of `program`, filling [`Interp::routines`]
    /// and refusing the ones this crate cannot resolve.
    ///
    /// **A directive that fails to resolve refuses the program; a directive
    /// that merely *exists* does not.** Both halves are measured, and they
    /// fail in opposite directions, which is why the rule is stated rather
    /// than approximated by "any directive is a gap". Programs the oracle
    /// runs, all rc 0 printing `main ran`:
    ///
    /// ```text
    /// ::class foo
    /// ::class foo + ::method bar
    /// ::class foo + ::attribute baz
    /// ::constant kk 5
    /// ::resource foo ... ::END
    /// ::annotate package author 'me'
    /// a loose ::method with no ::class
    /// ```
    ///
    /// Programs the oracle refuses before `main`, stdout EMPTY in every case:
    ///
    /// ```text
    /// ::class foo subclass zzznotaclass     98.909 rc 158
    /// ::class foo metaclass zzznotaclass    98.908 rc 158
    /// ::class bar inherit zzznotaclass      98.909 rc 158
    /// ::requires 'no_such_file_zz.rex'      43.901 rc 213
    /// ::routine z external "LIBRARY nosuchlib nosuchfn"   98.903 rc 158
    /// ::method m external "LIBRARY nosuchlib nosuchfn"    98.903 rc 158
    /// ::annotate routine nosuchrtn          99.945 rc 157
    /// duplicate ::routine of the same name  99.903 rc 157
    /// ```
    ///
    /// So the predicate below is: **a directive installs here when installing
    /// it neither runs code, changes a setting a Phase 4 construct can read,
    /// nor resolves a name against a table this crate does not have.** Each
    /// arm's own comment says which of the three it trips.
    ///
    /// **`::CONSTANT`'s parenthesised expression form is the one exception to
    /// "installing does not run code".** The oracle evaluates it right here,
    /// before `program.main`'s first clause, so a failing expression refuses
    /// the program exactly the way the other install-time gaps above do --
    /// measured, `::class K` then `::constant c (1/0)` is rc 214 with stdout
    /// EMPTY, both engines, while the same directive with `(2+3)` is rc 0 with
    /// `program.main`'s own output intact. A well-formed expression's *value*
    /// is discarded: nothing in this crate can read it back yet, since
    /// dispatching to a `::CONSTANT` accessor is later work.
    fn install_directives(&mut self, id: ProgramId, program: &Rc<Program>) -> Result<(), Failure> {
        // **One pass for both translation-time refusals**, because the oracle
        // finds each by reading the directive list rather than by installing
        // anything: a duplicate `::ROUTINE` name, and a parenthesised
        // `::CONSTANT` with no `::CLASS` anywhere before it. Measured,
        // `::constant sep (1+2)` alone in a file is rc 157 with `Error
        // 99.906`, where the identical directive under a preceding `::CLASS`
        // reaches the install-time evaluation below instead.
        let mut saw_class = false;
        for (index, directive) in program.directives.iter().enumerate() {
            match &directive.kind {
                DirectiveKind::Class(_) => saw_class = true,
                DirectiveKind::Constant(constant) => {
                    if matches!(constant.value, ConstantValue::Expression(_)) && !saw_class {
                        self.blame_directive(program, directive);
                        return Err(Raised::constant_needs_class().into());
                    }
                }
                DirectiveKind::Routine(routine) => {
                    // Resolves nothing outside this file and runs nothing:
                    // the body is already assembled in the AST, so installing
                    // it is recording a name.
                    let name: Box<[u8]> = routine.name.to_ascii_uppercase().into();
                    let installed = InstalledRoutine {
                        program: id,
                        directive: index,
                    };
                    if self.routines.insert(name, installed).is_some() {
                        // A *translation* error on the oracle, not an install
                        // one: measured, two `::routine zork` directives give
                        // `Error 99.903: Duplicate ::ROUTINE directive
                        // instruction.` at rc 157, echoing the second
                        // directive's own clause. `rexx-parse` does not
                        // detect it, so it is detected here, where the
                        // accumulated table is what answers.
                        self.blame_directive(program, directive);
                        return Err(Raised::duplicate_routine().into());
                    }
                }
                _ => {}
            }
        }

        // **A second pass, because the oracle's own translation-time
        // refusals above happen before every install-time one below**
        // (98.9xx/43.901/the `::CONSTANT` expression evaluation), so a
        // program with both gets the translation error -- which is what
        // running the whole first pass before any of this reproduces.
        //
        // **The failing-`::CONSTANT` blame target is the LAST `::CLASS`
        // directive in the whole file, not the nearest preceding one, and
        // that is measured rather than assumed.** `::class A` / `::constant
        // x (1/0)` / `::class B` blames `B`, and adding a third `::class C`
        // after that blames `C` -- the oracle's blame does not depend on
        // which class the constant is lexically under at all. Computed once,
        // up front, rather than tracked positionally the way
        // `current_class_id` below is for R9's registry attachment, which is
        // a genuinely different rule: the `Method` and `Attribute` arms
        // attach to the class positionally nearest above them, and only the
        // constant-failure echo uses this file-wide rule.
        let last_class_directive = program
            .directives
            .iter()
            .rev()
            .find(|d| matches!(d.kind, DirectiveKind::Class(_)));

        // R9: the class a `::METHOD`/`::ATTRIBUTE` attaches to, tracked
        // positionally (the most recently installed `::CLASS`) -- ordinary
        // object-model attachment, unrelated to `last_class_directive` above.
        let mut current_class_id: Option<ObjRef> = None;
        for (index, directive) in program.directives.iter().enumerate() {
            if let Some(loud) = directive_gap(&directive.kind) {
                return Err(loud.into());
            }
            match &directive.kind {
                DirectiveKind::Class(class) => {
                    current_class_id = Some(self.install_class(id, class));
                }
                DirectiveKind::Method(method) => {
                    if let Some(class_id) = current_class_id {
                        self.install_method(id, index, class_id, method);
                    }
                    // A loose `::METHOD` with no preceding `::CLASS` installs
                    // on the oracle (measured, rc 0 "main ran") and has
                    // nothing here to attach to; recording nothing for it is
                    // the R9 boundary, not a gap -- there is no dispatch yet
                    // to observe the difference.
                }
                DirectiveKind::Attribute(attribute) => {
                    if let Some(class_id) = current_class_id {
                        self.install_attribute(id, index, class_id, attribute);
                    }
                }
                DirectiveKind::Constant(constant) => {
                    let ConstantValue::Expression(expr) = &constant.value else {
                        continue;
                    };
                    if let Err(failure) = self.eval_constant_expression(id, program, expr) {
                        // Two clause echoes, innermost first, matching the
                        // oracle's own report exactly (measured, `::class K`
                        // / `::constant c (1/0)`):
                        //
                        // ```text
                        //      4 *-* ::constant c (1/0)
                        //      3 *-* ::class K
                        // ```
                        //
                        // `blame_directive` sets `self.failure_site`;
                        // `seal_site_level` is the same mechanism
                        // `invoke_call`/`run_fragment` use to move a level's
                        // site into `self.failure_sites` before the next,
                        // enclosing level sets its own -- there is no real
                        // activation nesting here, only the two directives'
                        // own clauses standing in for it.
                        self.blame_directive(program, directive);
                        self.seal_site_level();
                        self.blame_directive(
                            program,
                            last_class_directive.expect(
                                "the first pass already refused an expression with no \
                                 preceding ::CLASS",
                            ),
                        );
                        return Err(failure);
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// `::CLASS`'s own R9 install: a class object in [`Interp::classes`],
    /// carrying the name as written, and an entry in the running package's own
    /// class table under the uppercased one.
    ///
    /// **Every `::CLASS` naming a `SUBCLASS`/`METACLASS`/`INHERIT` is still a
    /// `directive_gap` above and never reaches here** (Phase 5, this crate
    /// resolves no class-name expression yet), so a class that does reach
    /// this point is the bare form: `subclass Object`, `metaclass Class`,
    /// which is what `RexxClass::subclass`'s own defaults give it
    /// (`ClassClass.cpp:1562`, and `Setup.cpp`'s every `StartClassDefinition`
    /// block passes the same pair).
    ///
    /// `String::from_utf8_lossy` rather than a hard requirement: a class
    /// name is close to always ASCII in practice and `ClassRegistry`'s API
    /// takes `&str`, so a non-UTF-8 literal name (legal Rexx, rare in
    /// practice) has its id recorded lossily rather than being rejected -- a
    /// known, narrow limitation. The id is observable now that `say .Foo`
    /// renders `The Foo class` through it, so a class whose declared name is
    /// not UTF-8 renders replacement characters where the oracle renders the
    /// bytes. The package table's key below is taken from the directive's own
    /// bytes and is not lossy.
    fn install_class(&mut self, program: ProgramId, class: &ClassDirective) -> ObjRef {
        let name = String::from_utf8_lossy(&class.name).into_owned();
        let (object, metaclass) = self.root_and_metaclass();
        // `define_unregistered_class`, not `define_class`: an installed class
        // does not go into `.environment`, and registering it there would let
        // `::class array` displace the environment's own `Array` for every
        // later lookup rather than only for this package's.
        let id = self.classes().define_unregistered_class(
            &name,
            Some(object),
            ClassKind::Regular,
            metaclass,
        );
        // The package's own installed-class table, which is what `.NAME`
        // resolution reads first -- see `environment.rs`'s
        // `record_package_class` for why the registry's flat table is not
        // that.
        self.record_package_class(program, &class.name, id);
        id
    }

    /// `::METHOD`'s own R9 install: the name lands in `class`'s instance
    /// dictionary, or its class dictionary for `::METHOD ... CLASS`, and
    /// [`Interp::record_method_body`] records which directive to read its
    /// body from later (Task 7's, not entered here).
    ///
    /// **The dictionary key is the upcased name**, which is what the oracle
    /// installs under: `LanguageParser::methodDirective` builds
    /// `internalname = commonString(name->upper())` and hands *that* to
    /// `addMethod`, exactly as `attributeDirective` does for the accessors
    /// [`Interp::install_attribute`] generates. `MethodDirective::name` keeps
    /// the as-written spelling, which is the method object's own name and a
    /// different thing from its lookup key.
    ///
    /// **Nothing observes the difference yet**, and no test pins it:
    /// `MethodDict::add_method` upcases its own key, so `::method "abc"`
    /// answers `~abc` either way and the stored key is the same string either
    /// way. The upcasing here is alignment with the oracle's own parser
    /// ahead of the first reader that can tell -- the method object's own
    /// name, which is the task that enters a method body.
    fn install_method(
        &mut self,
        program: ProgramId,
        directive: usize,
        class: ObjRef,
        method: &MethodDirective,
    ) {
        let name = String::from_utf8_lossy(&method.name.to_ascii_uppercase()).into_owned();
        let method_id = if method.class_method {
            self.classes().add_class_method(class, &name)
        } else {
            self.classes().add_instance_method(class, &name)
        };
        self.record_method_body(method_id, program, directive);
    }

    /// `::ATTRIBUTE`'s own R9 install: one or two accessor names, per
    /// [`AttributeStyle`] -- the plain name for a getter, the name with `=`
    /// appended for a setter, both for the default (neither `GET` nor `SET`)
    /// style -- landing in `class`'s instance or class dictionary the same
    /// way [`Interp::install_method`] does.
    ///
    /// One [`InstalledMethodBody`] per accessor, even for the `Both` style's
    /// two generated names sharing one directive: a later reader asking
    /// "does this method have a written Rexx body" reads
    /// `attribute.body`, which is `None` for both in that style and `Some`
    /// for the one style ([`AttributeStyle::Get`]/[`AttributeStyle::Set`])
    /// that admits a body at all -- [`Interp::record_method_body`] does not
    /// need to know which.
    ///
    /// The names are upcased for the same reason
    /// [`Interp::install_method`]'s is: `attributeDirective` derives both
    /// accessors from `internalname = commonString(name->upper())`, the
    /// setter as that name with `=` concatenated.
    fn install_attribute(
        &mut self,
        program: ProgramId,
        directive: usize,
        class: ObjRef,
        attribute: &AttributeDirective,
    ) {
        let upper = attribute.name.to_ascii_uppercase();
        let names: Vec<Vec<u8>> = match attribute.style {
            AttributeStyle::Both => {
                let mut setter = upper.clone();
                setter.push(b'=');
                vec![upper, setter]
            }
            AttributeStyle::Get => vec![upper],
            AttributeStyle::Set => {
                let mut setter = upper;
                setter.push(b'=');
                vec![setter]
            }
        };
        for name in names {
            let name = String::from_utf8_lossy(&name).into_owned();
            let method_id = if attribute.class_method {
                self.classes().add_class_method(class, &name)
            } else {
                self.classes().add_instance_method(class, &name)
            };
            self.record_method_body(method_id, program, directive);
        }
    }

    /// Records which `(program, directive)` a just-minted
    /// [`rexx_classes::MethodId`] names, immediately after the call that
    /// minted it -- see [`Interp::method_bodies`]'s own doc for what the key
    /// is.
    fn record_method_body(&mut self, method: MethodId, program: ProgramId, directive: usize) {
        let previous = self
            .method_bodies
            .insert(method, InstalledMethodBody { program, directive });
        debug_assert!(
            previous.is_none(),
            "a MethodId was recorded twice, so one of the two bodies is lost"
        );
    }

    /// Evaluates a `::CONSTANT` directive's parenthesised expression at
    /// install time: a throwaway activation with default `NUMERIC` settings
    /// and `TRACE` off, no bindings of its own, torn down immediately after.
    /// Mirrors the oracle's own moment for this (before `program.main`'s
    /// first clause), and is engine-agnostic: it runs once, before either
    /// engine's own instruction loop starts, and reaches the same shared
    /// [`Interp::eval`] both loops call, so `REXX_ENGINE=tree-walker` and the
    /// default IR engine evaluate it identically.
    ///
    /// `slots: &[]` and `plan: None` on the [`Code`] below are correct rather
    /// than merely convenient: the expression is evaluated with no enclosing
    /// frame at all, so every name in it -- there are none in this task's own
    /// corpus witnesses -- falls through [`Code::slot_for`] to
    /// [`Interp::slot_of`]'s ordinary resolution exactly as an `INTERPRET`
    /// fragment's untranslated names do (`run_fragment`, `run.rs`).
    ///
    /// **The `Plan::default()` handed to `Activation::new` is a different,
    /// unrelated placeholder, not a second copy of the one above.**
    /// `Activation::new`'s signature requires an `Rc<Plan>` field to exist.
    /// [`Interp::slot_of`] does read it, but every read goes through
    /// `Plan::slot_of`, which answers `None` for a name it does not carry,
    /// so an empty plan sends each name down the unresolved path. Building
    /// one from the body here would compute nothing this call could reach:
    /// the body it would walk has no instructions, so the map comes out
    /// empty either way.
    fn eval_constant_expression(
        &mut self,
        id: ProgramId,
        program: &Rc<Program>,
        expr: &Expr,
    ) -> Result<(), Failure> {
        let empty_body = CodeBody::default();
        let plan = Rc::new(Plan::default());
        let frame = self.roots.push_slots(0);
        let activation_id = self.next_activation_id();
        self.activations.push(Activation::new(
            activation_id,
            Rc::clone(program),
            id,
            plan,
            frame,
        ));
        let code = Code {
            body: &empty_body,
            symbols: &program.symbols,
            slots: &[],
            plan: None,
        };
        let result = self.eval(&code, expr);
        self.activations.pop();
        self.roots.pop_slots(frame);
        result.map(|_| ())
    }

    /// Records `directive`'s own clause as the site a directive-time
    /// condition is reported against, so its report carries the same echo
    /// line the oracle prints above the two `Error` lines.
    ///
    /// Indent 0 unconditionally: a directive is never nested inside anything,
    /// and the oracle's own echo for one is flush left (measured, `     2 *-*
    /// ::class foo subclass zzznotaclass`).
    fn blame_directive(&mut self, program: &Rc<Program>, directive: &Directive) {
        let line = program.source.line_of(directive.clause_span.start);
        let text = program
            .source
            .join_span(directive.clause_span.clone())
            .map_or_else(
                || b"<clause span outside the retained source>".to_vec(),
                |bytes| bytes.into_owned(),
            );
        self.failure_site = Some(FailureSite::Clause {
            line,
            text,
            indent: 0,
        });
    }

    // `plan_for` and `activation`/`activation_mut` live in `plan.rs`/
    // `activation.rs` (Task 6), beside the types they operate on.

    // `fragment_plan` and `slot_of` live in `plan.rs` (Task 6), beside
    // `Plan` itself.

    /// Reads a variable, resolving its slot here.
    fn read(&mut self, code: &Code<'_>, id: SymbolId) -> (ObjRef, Novalue) {
        self.read_at(code, id, None)
    }

    /// Reads a variable, by the slot the plan already resolved its id to, or
    /// by `at` when a compiler resolved the same thing earlier.
    ///
    /// Falls back to `slot_of` when the plan never saw the id, which a
    /// non-exhaustive `Plan::build` can produce and which the exhaustive pass
    /// should not. The fallback is not dead weight even then: it is the same
    /// path a name bound at run time takes.
    ///
    /// **`at` is that same resolution made at compile time**, from the `Plan`
    /// whose `by_symbol` map `Code::slots` is a view of, so the two cannot be
    /// different slots for one activation -- `Interp::run_ops` asserts that in
    /// debug at the op that carries one.
    pub(crate) fn read_at(
        &mut self,
        code: &Code<'_>,
        id: SymbolId,
        at: Option<usize>,
    ) -> (ObjRef, Novalue) {
        let slot = match at {
            Some(slot) => slot,
            None => match code.slot_for(id) {
                Some(slot) => slot,
                None => self.slot_of(code.symbols.name(id).as_bytes()),
            },
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
    /// 0, which is where it already sits on every path here. Measured:
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

    /// Roots a value that has to outlive the clause that produced it, all the
    /// way to [`Interp::exit_code_for`].
    ///
    /// **The one value in this crate whose lifetime the temps stack cannot
    /// express.** `EXIT`'s result is pushed as an ordinary one-clause temp,
    /// and `leave_stepped_clause` pops that frame before `Flow::Exit` has even
    /// reached `run_activation` -- so from there through the activation
    /// teardown and into `execute`'s conversion, nothing on the temps stack
    /// names it. `add_global` is the root that survives, because nothing
    /// truncates the globals list.
    ///
    /// **Measured, which is why this exists rather than a comment arguing the
    /// window is benign.** A `Heap::collect` placed immediately before
    /// `exit_code_for` swept the value and panicked on `a live value` in four
    /// harnesses; with this root taken, the same probe leaves the whole
    /// workspace green. The trigger itself never fires inside the window --
    /// the workspace is also green with collect-on-every-allocation forced on
    /// for every run -- but that says only that nothing on today's path
    /// allocates, which is a property of the code rather than an invariant of
    /// the design.
    ///
    /// One name, replaced rather than accumulated, so a routine whose own
    /// `EXIT` becomes a `RETURN` to its caller (`run.rs`'s `Entered::Routine`
    /// arm) can run any number of times and hold one value at a time.
    fn root_exit_value(&mut self, value: ObjRef) {
        self.roots.add_global(EXIT_VALUE_ROOT, value);
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
    /// **Two things can make it collect first, and they are different
    /// questions.** `stress_collect` collects on *every* allocation and is a
    /// test instrument (`run_program_collect_every_alloc`). The other is the
    /// production trigger, and it is two tests: the arena is about to grow,
    /// and it has reached `collect_at` slots. `collect_at`'s own doc comment
    /// defines the policy; the cost when it does not fire is an `Option`
    /// discriminant test and a `usize` comparison, both against state the
    /// heap already maintains.
    ///
    /// **Every allocation is a collection point, and that is what makes the
    /// rooting discipline load-bearing rather than advisory.** Before this
    /// trigger existed, a value held only in a Rust local across another
    /// allocation was inert -- nothing swept, so nothing noticed. Now it is a
    /// use-after-free that shows up as a wrong answer. `push_temp` the value
    /// before anything else can allocate; the instrument that finds the ones
    /// that were missed is `run_program_collect_every_alloc`, which collects
    /// strictly more often than any watermark can.
    ///
    /// `self.heap` and `self.roots` are sibling fields, so this borrows each
    /// independently and needs no interior mutability or unsafe cell to call
    /// one method with a borrow of the other in scope.
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
        if self.stress_collect
            || (self.heap.will_grow() && self.heap.slot_capacity() >= self.collect_at)
        {
            let stats = self.heap.collect(&self.roots);
            // `pending_uninit` is what the collector resurrected so a finalizer
            // could run against a whole graph. Nothing in this crate sets
            // `Object::has_uninit`, because `UNINIT` needs a class to define
            // it and message sends are Phase 5, so the list is empty and there
            // is nothing to deliver. The day something sets that flag, this is
            // the site that owes the delivery -- which is why the value is
            // named here rather than dropped at the call.
            debug_assert!(
                stats.pending_uninit.is_empty(),
                "an object was resurrected for UNINIT and nothing here runs a finalizer"
            );
            // **Not raised for the stress mode**, which collects on every
            // allocation by definition and must not have its watermark moved
            // out from under it.
            if !self.stress_collect {
                self.collect_at = COLLECT_FLOOR.max(stats.live.saturating_mul(2));
            }
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
///
/// `invocation` is what the command line supplied -- see [`Invocation`], whose
/// own doc carries the measured argument model. A caller with nothing to
/// supply passes [`Invocation::none`], which is what a program run as
/// `rexx p.rex` gets and is not the same as one run as `rexx p.rex ""`.
///
/// **A third parameter rather than a sibling entry point.** The alternative
/// considered was leaving this signature alone and adding
/// `run_program_with_arguments` beside it, which is cheaper to land and wrong
/// for the reason `run_program_collect_every_alloc`'s own doc states about
/// itself: this crate has one front door, and a second one is a second thing
/// for every future caller to choose between. Widening the parameter list once
/// costs a mechanical edit at every existing call site and nothing afterward.
pub fn run_program(path: &str, text: Vec<u8>, invocation: Invocation) -> Outcome {
    let path = path.to_string();
    on_interpreter_thread(move || execute(&path, text, false, invocation))
}

/// `run_program`, except that `Heap::collect` runs after every allocation
/// instead of never. The 4a exit gate's criterion 4: the named L0 subset
/// has to pass again under this mode, with the mode proved to have actually
/// collected (`Outcome::collections` non-zero) rather than merely having
/// been requested.
///
/// `#[doc(hidden)]` because it is `pub` only so `tests/` can reach it, not a
/// second front-door choice beside `run_program`. It is the crate's only
/// hidden entry point.
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
///
/// It takes an [`Invocation`] for the same reason it takes a `path` and a
/// `text`: it is a mode of `execute`, not a narrower entry point, and a mode
/// that could not run a program the ordinary front door can run would be
/// unable to stress exactly the programs whose rooting is least obvious --
/// the argument string is an `ObjRef` this crate has to keep reachable for the
/// whole run, which is the kind of thing this mode exists to check.
#[doc(hidden)]
pub fn run_program_collect_every_alloc(
    path: &str,
    text: Vec<u8>,
    invocation: Invocation,
) -> Outcome {
    let path = path.to_string();
    on_interpreter_thread(move || execute(&path, text, true, invocation))
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
fn execute(
    path: &str,
    text: Vec<u8>,
    collect_every_alloc: bool,
    invocation: Invocation,
) -> Outcome {
    let program = match parse_program(text) {
        Ok(program) => program,
        // **A top-level parse failure stays loud, and that was checked rather
        // than assumed either way.** The `ParseError`-to-`Raised` conversion
        // exists and `INTERPRET` uses it
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
                chunks_refused: 0,
            };
        }
    };

    let mut interp = Interp::new();
    interp.program_path = path.to_string();
    if collect_every_alloc {
        interp.enable_stress_collect();
    }
    // The command line is the top-level program's caller, so what it supplied
    // goes into the same `call_context` a `CALL` fills -- see that field's own
    // doc for what reads it and for the three measured invocations that tell
    // "no argument" from "one empty argument" apart.
    interp.call_context.name = path.as_bytes().to_vec();
    let (argument, program_input, engine) = invocation.into_parts();
    interp.input = Input::new(program_input);
    interp.engine = engine;
    if let Some(argument) = argument {
        let value = interp.text(&argument);
        // Rooted with a `push_temp` taken before `run`, which is what makes it
        // outlive every clause: `step_in_temps_frame` truncates the
        // temporaries stack back to a watermark it takes on entry, and every
        // such watermark sits above this push. This is the same mechanism
        // `Interp::invoke_call` uses to keep a call's own arguments reachable
        // (`run.rs`, the `push_temp(argument.value())` beside the argument
        // list it builds); `call_context` itself is not walked by the
        // collector, so without this the value is unreachable the first time
        // anything allocates.
        interp.roots.push_temp(value);
        interp.call_context.arguments = vec![Some(Argument::Value(value))];
    }
    let result = interp.run(program);
    let stack = interp.stack_span();
    let collections = interp.heap.collections_performed();
    let chunks_refused = interp.chunks_refused;
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
        // `Failure::Exited` is not a failure -- it is `EXIT` (or falling off
        // the routine's own end) reached through `ExprKind::Call`'s
        // expression form, tunnelled here through `Err`/`?` only because
        // `eval`'s own return type has no `Flow` to carry it through instead
        // (`Failure::Exited`'s own doc, `error.rs`, has the full argument).
        // Treated exactly like an ordinary `Ok(value)`: same exit-code rule,
        // no stderr report, because it is not one.
        Ok(value) | Err(Failure::Exited(value)) => interp.exit_code_for(value),
        Err(Failure::Loud(loud)) => {
            interp
                .trace
                .extend_from_slice(format!("rexx-exec: {}\n", loud.message).as_bytes());
            NOT_IMPLEMENTED_EXIT
        }
        Err(Failure::Raised(raised)) => {
            // `run_activation` records the site on the way out. An empty
            // stack here would mean a condition escaped without passing an
            // instruction loop, which nothing in this crate can do; it
            // renders visibly rather than panicking, on the error path's
            // standing rule that a reportable condition must never become a
            // crash.
            if failure_sites.is_empty() {
                failure_sites.push(FailureSite::Clause {
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
        chunks_refused,
    }
}

#[cfg(test)]
mod tests {
    use super::{Interp, ProgramId, form_name, parse_program, run_program};
    use rexx_parse::{DirectiveKind, Expr, ExprKind, Operator, PrefixOp, Program};
    use std::rc::Rc;

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
    /// operator, prefix and dyadic, is implemented here -- a witness picked
    /// from the operators has to move each time one lands. Calling
    /// `form_name` directly needs no unimplemented form at all, so nothing a
    /// later task does can take this coverage away.
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

    // ---- the fragment's lifetime (I7) ----
    //
    // Here rather than in `tests/spike.rs`, and the usual argument against a
    // unit test does not apply: "a unit test with privileged access to
    // private internals proves less about the shape callers actually get".
    // These need no privileged access. They call `run_program`, the same
    // public entry point on the same sized thread an integration test would
    // use, because a fragment is reachable through the front door.

    /// Step 4's test, and the one property `INTERPRET` has that no other
    /// instruction does: a name bound inside fragment text outlives the
    /// fragment, so a *later, separate* fragment reads it back.
    ///
    /// Measured on the oracle: the binding outlives the fragment.
    #[test]
    fn interpret_binds_a_name_the_enclosing_body_never_mentions() {
        let outcome = run_program(
            TEST_PATH,
            b"interpret \"zork = 42\"\ninterpret \"say zork\"\n".to_vec(),
            crate::Invocation::none(),
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
        let outcome = run_program(TEST_PATH, program.to_vec(), crate::Invocation::none());
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
        let outcome = run_program(TEST_PATH, program.to_vec(), crate::Invocation::none());
        assert_eq!(outcome.exit_code, 0, "stderr: {:?}", outcome.stderr);
        assert_eq!(outcome.stdout, b"before\ninside\n");
    }

    /// A condition raised inside an `INTERPRET` fragment reports
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
        let outcome = run_program(TEST_PATH, program.to_vec(), crate::Invocation::none());
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
            let outcome = run_program(
                TEST_PATH,
                program.as_bytes().to_vec(),
                crate::Invocation::none(),
            );
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
    /// **The divergence does not need a fragment base**: a
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
    /// a `Controlled` loop's re-tested pass traces its own control-variable
    /// value lines, which this test is not the place to encode: this test is
    /// about a fragment's own indent.
    #[test]
    fn a_when_scan_inside_a_fragment_echoes_at_the_fragments_own_indent() {
        let program = "trace r\n\
                       do\n\
                       interpret \"select; when 1 = 1 then nop; end; nop\"\n\
                       end\n";
        let outcome = run_program(
            TEST_PATH,
            program.as_bytes().to_vec(),
            crate::Invocation::none(),
        );
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

    /// A fragment that does not parse raises the oracle's own condition
    /// instead of failing loudly.
    ///
    /// Measured: `interpret "do forever then"` is **27.901 at rc 229** on the
    /// oracle, and `interpret "if"` is 35.929 at rc 221.
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
            crate::Invocation::none(),
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

        let outcome = run_program(
            TEST_PATH,
            b"say 1\ninterpret \"if\"\n".to_vec(),
            crate::Invocation::none(),
        );
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
    ///
    /// **On the tree-walker, because the recursion this measures is `eval`'s.**
    /// `ir::compile` promotes a chain of native operators to ops that reach the
    /// operator with its operands in registers, so on the compiled engine this
    /// chain does not enter `eval` at all and both spans would be the depth of
    /// nothing -- equal, and equal for the wrong reason. `eval::tests`'
    /// `depth_limited` carries the measurement behind that.
    #[test]
    fn the_stack_span_does_not_depend_on_what_else_the_program_evaluated() {
        let mut alone = b"say 'a'".to_vec();
        for _ in 1..1_000 {
            alone.extend_from_slice(b"||''");
        }
        alone.push(b'\n');

        let mut then_a_fragment = alone.clone();
        then_a_fragment.extend_from_slice(b"interpret \"say 'b'\"\n");

        let on_eval = || crate::Invocation::none().with_engine(crate::Engine::TreeWalker);
        let alone = run_program(TEST_PATH, alone, on_eval());
        let then_a_fragment = run_program(TEST_PATH, then_a_fragment, on_eval());

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

    // ---- R9: ::CLASS/::METHOD/::ATTRIBUTE create and record in
    // `Interp::classes`. No corpus program can see this yet -- reading a
    // class object back needs a message send, and `ExprKind::Message` still
    // fails loudly (Task 5's), so this is witnessed here, against the
    // registry `install_directives` leaves behind, instead. ----

    /// Parses `text` and installs its directives against a fresh `Interp`,
    /// under `ProgramId(0)`, handing both back so a test can read the
    /// registry state left behind.
    fn installed(text: &[u8]) -> (Interp, Rc<Program>) {
        let program = Rc::new(parse_program(text.to_vec()).expect("test program parses"));
        let mut interp = Interp::new();
        interp
            .install_directives(ProgramId(0), &program)
            .expect("test program installs without a raised condition");
        (interp, program)
    }

    /// The class `::CLASS name` installed, read out of the package's own
    /// table.
    ///
    /// Not `ClassRegistry::lookup`: that table is `.environment`'s class
    /// entries and an installed class is deliberately absent from it, so a
    /// lookup there would answer the environment's class of the same name or
    /// nothing at all.
    fn installed_class(interp: &Interp, name: &str) -> rexx_core::ObjRef {
        interp.package_classes[&ProgramId(0)][name.as_bytes()]
    }

    /// Had `::CLASS` not created anything, the package's own table would hold
    /// nothing and the index below would panic.
    #[test]
    fn a_bare_class_directive_creates_a_class_object() {
        let (interp, _program) = installed(b"say 'main ran'\n::class Foo\n");
        let id = installed_class(&interp, "FOO");
        assert!(id.class_id().is_some(), "a class identity, not a value");
    }

    /// **An installed class is not an environment entry**, which is the whole
    /// of why `install_class` does not register the name.
    ///
    /// `::class array` declares a class whose id is `ARRAY`, and
    /// `.environment` must still answer the native `Array` for that name --
    /// the package's own table is what shadows it, and only for this package.
    /// Had the directive registered the name, the environment's snapshot
    /// would be built from a table the directive had already overwritten.
    #[test]
    fn an_installed_class_does_not_displace_the_environments_own_entry() {
        let (mut interp, _program) = installed(b"say 'main ran'\n::class array\n");
        let installed = installed_class(&interp, "ARRAY");
        let native = interp.classes().lookup("Array").expect("Array is native");
        assert_ne!(installed, native);
        // Upcased, because the directive names the class with a symbol and
        // the scanner interns a symbol upcased -- which is why the oracle
        // prints `The ARRAY class` for it and `The Array class` for the
        // environment's own.
        assert_eq!(interp.classes().id_string(installed), "ARRAY");
        assert_eq!(interp.classes().id_string(native), "Array");
    }

    /// A bare `::CLASS` -- the only shape that reaches this far, since
    /// `SUBCLASS`/`METACLASS`/`INHERIT` are still `directive_gap` above --
    /// is `subclass Object`, `metaclass Class`, which is what
    /// `RexxClass::subclass`'s own defaults give it. Had `install_class`
    /// left the superclass out, or named some other class, this would
    /// catch it.
    #[test]
    fn a_bare_class_directive_subclasses_object() {
        let (mut interp, _program) = installed(b"say 'main ran'\n::class Foo\n");
        let id = installed_class(&interp, "FOO");
        let object = interp.classes().lookup("Object").unwrap();
        let class = interp.classes().lookup("Class").unwrap();
        assert_eq!(interp.classes().superclass(id), Some(object));
        assert_eq!(interp.classes().metaclass(id), class);
    }

    /// The other half of the same install: a user class inherits `.Object`'s
    /// own instance methods through the flattened cascade, which is what a
    /// send to one of its instances would resolve against. Had
    /// `install_class` recorded no superclass, this set would hold `BAR`
    /// alone.
    #[test]
    fn a_user_class_inherits_objects_instance_methods() {
        let (mut interp, _program) =
            installed(b"say 'main ran'\n::class Foo\n::method bar\n  return 1\n");
        let id = installed_class(&interp, "FOO");
        let names = interp.classes().instance_method_names(id);
        assert!(names.contains("BAR"));
        assert!(names.contains("HASMETHOD"));
    }

    /// Had `::METHOD` landed in the class dictionary instead of the instance
    /// one (or nowhere), one side of this pair would be wrong.
    ///
    /// `own_*_method_names` rather than the flattened sets the previous test
    /// reads: `.Object`'s and `.Class`'s own methods are in both flattened
    /// sets now, and a name absent from one of them is only evidence if the
    /// set is this class's own.
    #[test]
    fn a_method_directive_lands_in_the_classs_instance_dictionary() {
        let (mut interp, _program) =
            installed(b"say 'main ran'\n::class Foo\n::method bar\n  return 1\n");
        let id = installed_class(&interp, "FOO");
        assert!(
            interp
                .classes()
                .own_instance_method_names(id)
                .contains("BAR")
        );
        assert!(!interp.classes().own_class_method_names(id).contains("BAR"));
    }

    /// `::METHOD ... CLASS`'s own side of the same pair.
    #[test]
    fn a_class_method_directive_lands_in_the_classs_class_dictionary() {
        let (mut interp, _program) =
            installed(b"say 'main ran'\n::class Foo\n::method bar class\n  return 1\n");
        let id = installed_class(&interp, "FOO");
        assert!(interp.classes().own_class_method_names(id).contains("BAR"));
        assert!(
            !interp
                .classes()
                .own_instance_method_names(id)
                .contains("BAR")
        );
    }

    /// Neither `GET` nor `SET`: both accessor names install. Had the `=`
    /// suffix been on the wrong name, or missing, one side of this pair
    /// would fail.
    #[test]
    fn an_attribute_with_no_style_installs_both_accessor_names() {
        let (mut interp, _program) = installed(b"say 'main ran'\n::class Foo\n::attribute baz\n");
        let id = installed_class(&interp, "FOO");
        let names = interp.classes().own_instance_method_names(id);
        assert!(names.contains("BAZ"));
        assert!(names.contains("BAZ="));
    }

    /// `GET` alone installs only the getter -- had `install_attribute`
    /// always installed both names regardless of style, `BAZ=` would be
    /// present here too.
    #[test]
    fn an_attribute_get_installs_only_the_getter() {
        let (mut interp, _program) =
            installed(b"say 'main ran'\n::class Foo\n::attribute baz get\n");
        let id = installed_class(&interp, "FOO");
        let names = interp.classes().own_instance_method_names(id);
        assert!(names.contains("BAZ"));
        assert!(!names.contains("BAZ="));
    }

    /// A loose `::METHOD` with no preceding `::CLASS` installs on the oracle
    /// (measured, rc 0 "main ran") and has nothing here to attach to -- had
    /// `install_directives` recorded it against a stale or default class id
    /// instead of skipping it, `method_bodies` would be non-empty here.
    #[test]
    fn a_loose_method_with_no_preceding_class_is_not_recorded() {
        let (interp, _program) = installed(b"say 'main ran'\n::method bar\n  return 1\n");
        assert!(interp.method_bodies.is_empty());
    }

    /// The "bodies are stored" half of R9: `method_bodies` names the exact
    /// directive a method's own body came from, keyed by the identity the
    /// registry minted for it. Had `record_method_body` recorded the wrong
    /// directive index, or keyed the two methods the same way, this would
    /// catch it; had a mint been recorded twice, the `debug_assert!` inside
    /// `record_method_body` catches that first, in every debug build
    /// including the workspace's own gate.
    #[test]
    fn method_bodies_names_each_directive_by_its_minted_identity() {
        let (mut interp, program) = installed(
            b"say 'main ran'\n::class Foo\n::method bar\n  return 1\n::method baz class\n  return 2\n",
        );
        let index_of = |name: &[u8]| -> usize {
            program
                .directives
                .iter()
                .position(|d| matches!(&d.kind, DirectiveKind::Method(m) if &*m.name == name))
                .expect("the ::METHOD directive is in the program")
        };
        let id = installed_class(&interp, "FOO");
        // A bare symbol's own name is already upcased by the scanner (a
        // quoted literal is the shape that would keep the source case), so
        // `bar`/`baz class` in the source above are `BAR`/`BAZ` here.
        let (_, bar) = interp
            .classes()
            .lookup_instance_method(id, "BAR")
            .expect("::method bar resolves");
        let (_, baz) = interp
            .classes()
            .lookup_class_method(id, "BAZ")
            .expect("::method baz class resolves");
        assert_eq!(interp.method_bodies.len(), 2);
        assert_eq!(interp.method_bodies[&bar].program, ProgramId(0));
        assert_eq!(interp.method_bodies[&bar].directive, index_of(b"BAR"));
        assert_eq!(interp.method_bodies[&baz].program, ProgramId(0));
        assert_eq!(interp.method_bodies[&baz].directive, index_of(b"BAZ"));
    }
}
