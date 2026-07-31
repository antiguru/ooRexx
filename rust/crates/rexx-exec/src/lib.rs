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
//! below is where that discipline is written down, together with the version
//! of it that does not compile.
//!
//! **What it executes grows task by task, and this doc deliberately does not
//! list it.** The enumeration that stood here was true of Task 3's spike and
//! false by Task 6, and the two `Loud` messages carried the same list and went
//! stale the same way; `Loud::expression`'s doc records why the cure is
//! deleting the list rather than correcting it. What holds instead: any
//! construct not yet implemented fails loudly with `NOT_IMPLEMENTED_EXIT`
//! rather than silently, which is a gate criterion.
//!
//! The per-concept modules the design's crate layout names have landed beside
//! this file (`value.rs`, `stem.rs`, `plan.rs`, `activation.rs`, `error.rs`,
//! `eval.rs`). What stays here is the interpreter itself, the loud-failure
//! path, and the borrow discipline, which is the one thing meant to survive as
//! it stands.

use rexx_core::{Heap, ObjRef, RootSet};
use rexx_parse::{
    CodeBody, ExprKind, Fragment, Instruction, InstructionKind, ParseError, PrefixOp, Program,
    SymbolId, SymbolTable, parse_interpret, parse_program,
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
use error::Failure;

// Expression evaluation (`eval`/`eval_node`): terms, arithmetic and
// concatenation.
mod eval;

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
/// **Task 12 owns the final value** and states it in `error.rs` alongside the
/// message catalogue. This constant is the spike's choice and the single place
/// to change it.
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
/// The budget this covers is larger than `eval` alone, and all three of its
/// users run on this same thread because the entry point owns everything from
/// `parse_program` onward:
///
/// * `eval` recursing once per term of a left-deep expression,
/// * `Plan::note` recursing over the same expression to assign its slots,
///   which is this crate's own and is the shallowest-per-level of the three
///   at about 160 bytes, but is still a recursion and could be given the same
///   explicit-worklist treatment `rexx-parse`'s walks got,
/// * and dropping the AST, which `rexx-parse` now does iteratively. It used to
///   recurse once per `Box<Expr>` level, and was the thing that bound this
///   budget until it was fixed.
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
            message: format!("{name} is not implemented"),
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
    /// list was true of Task 3's spike and false by Task 6, which is the whole
    /// argument: every task that implements a form has to remember to edit a
    /// string in a file it is not otherwise touching, and none of the three
    /// that shipped between did. A message that can only go stale by being
    /// wrong about its own subject cannot rot this way, so the enumeration is
    /// gone rather than corrected.
    fn expression(kind: &ExprKind) -> Loud {
        Loud {
            message: format!("{} is not implemented", form_name(kind)),
        }
    }

    /// A fragment that did not parse. Task 12 owns the real reporting; the
    /// spike only has to not swallow it.
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
    /// The trace sink, which becomes `Outcome::stderr` **and which nothing in
    /// this crate writes yet.**
    ///
    /// It exists because the design puts both sinks on `Interp` and D17 makes
    /// them separate for a measured reason: with `trace r` the `*-*` and `>>>`
    /// lines are on stderr while `SAY` is on stdout, and being separate
    /// descriptors is what makes their relative interleaving unobservable and
    /// two independently buffered sinks safe. Task 13 is the first to write to
    /// it. Keeping it now costs one field and means the loud-failure path
    /// already appends to the right buffer rather than being rerouted later,
    /// which is when a stray ordering difference would appear.
    trace: Vec<u8>,
    /// True when the caller is the fragment spike, in which case
    /// `InstructionKind::Interpret` runs its fragment instead of failing
    /// loudly.
    ///
    /// 4a builds the fragment machinery and **4b builds the `INTERPRET`
    /// instruction on top of it**, so through `run_program` the keyword is
    /// still not implemented and still exits `NOT_IMPLEMENTED_EXIT`. 4b's
    /// first move here is to delete this field and the branch that reads it.
    interpret_spike: bool,
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
    fn new(interpret_spike: bool) -> Interp {
        Interp {
            heap: Heap::new(),
            roots: RootSet::new(),
            activations: Vec::new(),
            programs: Vec::new(),
            plans: HashMap::new(),
            out: Vec::new(),
            trace: Vec::new(),
            interpret_spike,
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
    fn run_activation(&mut self) -> Result<Option<ObjRef>, Failure> {
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
            match self.step_in_temps_frame(&code, instruction)? {
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
                // here; the spike takes the first. Task 5 built the
                // `stem_assign`/`stem_set` library the other two need, but
                // not the dispatch itself: recognising a `Stem`/`Compound`
                // target and calling into it is Task 9's (the instruction
                // loop), which needs `eval_node` to evaluate those forms as
                // an expression first, and that is Task 7's.
                let ExprKind::Variable(id) = &target.kind else {
                    return Err(Loud::expression(&target.kind).into());
                };
                let name = code.symbols.name(*id).as_bytes();
                let value = self.eval(code, value)?;
                self.roots.push_temp(value);
                let slot = self.slot_of(name);
                let frame = self.activation().frame;
                self.roots.set_slot(frame, slot, value);
                Ok(Flow::Next)
            }

            // `EXIT` with a result is Task 9's, together with the mapping from
            // that result to a process exit code, so the spike takes only the
            // bare form.
            InstructionKind::Exit { expression: None } => Ok(Flow::Exit(None)),

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
pub fn run_program(text: Vec<u8>) -> Outcome {
    on_interpreter_thread(move || execute(text, false))
}

/// `run_program`, except that an `INTERPRET` instruction runs its fragment
/// instead of failing loudly.
///
/// **Task 3's spike surface, and 4b deletes it.** The `INTERPRET` *keyword* is
/// 4b's; what 4a owns is the machinery underneath it, and the lifetime that
/// machinery has to satisfy is only exercised by actually running a fragment
/// mid-instruction. An integration test sees only the public API, so proving
/// it needs an entry point that admits the fragment while `run_program` keeps
/// the loud failure that 4a's contract requires.
///
/// `#[doc(hidden)]` because it is `pub` only to reach `tests/`, and without it
/// it appears in the rendered docs beside `run_program` as though it were an
/// equal choice of entry point.
///
/// **The choice that created this surface, named so that whoever deletes it
/// knows what to weigh:** a `#[cfg(test)] mod tests` inside this file could
/// call the private `on_interpreter_thread` directly and prove the same
/// lifetime with **no public surface at all**. Picking an integration test
/// over a unit test is what forced a public entry point to exist. The
/// integration test was preferred because it exercises the crate the way every
/// later harness will, through the public API and on the sized thread, and
/// because a unit test with privileged access to private internals proves less
/// about the shape callers actually get. That is a defensible trade and not an
/// obvious one, so 4b should re-make it rather than inherit it.
///
/// The rejected alternative is worth recording: hooking the fragment onto an
/// innocent instruction such as `NOP` proves the same lifetime while lying
/// about which node owns it, and leaves nothing for 4b to delete.
#[doc(hidden)]
pub fn run_program_interpret_spike(text: Vec<u8>) -> Outcome {
    on_interpreter_thread(move || execute(text, true))
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
fn execute(text: Vec<u8>, interpret_spike: bool) -> Outcome {
    let program = match parse_program(text) {
        Ok(program) => program,
        // Task 12 owns the oracle's exact two-line syntax-error format and the
        // `256 - major` exit code. Until then a parse failure is loud, which
        // is wrong in the details and right in never being mistaken for
        // success.
        Err(error) => {
            return Outcome {
                exit_code: NOT_IMPLEMENTED_EXIT,
                stdout: Vec::new(),
                stderr: format!("rexx-exec: {error}\n").into_bytes(),
                stack: StackSpan::default(),
            };
        }
    };

    let mut interp = Interp::new(interpret_spike);
    let result = interp.run(program);
    let stack = interp.stack_span();
    let mut stderr = interp.trace;
    let exit_code = match result {
        Ok(_) => 0,
        Err(Failure::Loud(loud)) => {
            stderr.extend_from_slice(format!("rexx-exec: {}\n", loud.message).as_bytes());
            NOT_IMPLEMENTED_EXIT
        }
        // Task 12 owns the oracle's exact two-line stderr format and the
        // `256 - number` exit code; this only says WHICH condition fired,
        // without pretending to reproduce what the oracle prints for it --
        // the same "wrong in the details, right in never being mistaken
        // for success" rule the parse-error arm above already follows.
        Err(Failure::Raised(raised)) => {
            stderr.extend_from_slice(
                format!(
                    "rexx-exec: raised {} {}.{} (not yet rendered: Task 12), additional {:?}\n",
                    raised.condition, raised.number, raised.sub, raised.additional
                )
                .as_bytes(),
            );
            NOT_IMPLEMENTED_EXIT
        }
    };

    Outcome {
        exit_code,
        stdout: interp.out,
        stderr,
        stack,
    }
}

#[cfg(test)]
mod tests {
    use super::form_name;
    use rexx_parse::{Expr, ExprKind, Operator, PrefixOp};

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
}
