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
//! What it executes is deliberately almost nothing: `SAY`, assignment to a
//! simple variable, concatenation with `||`, and an `INTERPRET` fragment when
//! the spike entry point asks for one. Every other construct fails loudly with
//! `NOT_IMPLEMENTED_EXIT`. Tasks 4 to 13 replace this file with the per-concept
//! modules the design's crate layout names (`value.rs`, `plan.rs`,
//! `activation.rs`, `eval.rs`, `run.rs`, and the rest); nothing here is meant
//! to survive as it stands except the borrow discipline itself.

use rexx_core::{BehaviourId, Body, Heap, ObjRef, RootSet, SlotFrame};
use rexx_parse::{
    CodeBody, Expr, ExprKind, Fragment, Instruction, InstructionKind, Operator, ParseError,
    PrefixOp, Program, SymbolId, SymbolTable, parse_interpret, parse_program,
};
use std::collections::HashMap;
use std::rc::Rc;

/// The exit code for a construct Phase 4a does not implement.
///
/// It has to sit outside 157..=253, where a Rexx error's `256 - major` lives,
/// or a not-implemented failure is indistinguishable from error 11 and a
/// program *expecting* error 11 would pass. 120 also sits below 126, so it
/// cannot be read as a shell's `128 + signal` encoding either, and it is not
/// 0, 1, 2, 126 or 127.
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
/// whole pipeline survives to roughly 620,000 levels here, against a limit
/// that has to be at least 100,000, so there is about six times the headroom a
/// limit at the oracle's own maximum needs.
///
/// The budget this covers is larger than `eval` alone, and all three of its
/// users run on this same thread because the entry point owns everything from
/// `parse_program` onward:
///
/// * `eval` recursing once per term of a left-deep expression,
/// * `Plan::note` recursing over the same expression to assign its slots,
/// * and `Drop` for the AST, which recurses once per `Box<Expr>` level when
///   the `Rc<Program>` goes. That third one is easy to forget and is a real
///   stack consumer: nothing in the source says "recursive", the derive does.
///
/// **The 784 figure covers only the first of those three, and the first is not
/// the one that binds.** Measured by bisecting `rexx-run` on this stack, debug,
/// with the three phases separated by choosing programs that reach different
/// ones (`exit` first, so the deep expression is parsed and dropped but never
/// evaluated; a bare command clause, which `Plan::build` also skips):
///
/// | what runs | deepest surviving | implied bytes per level |
/// |---|---|---|
/// | parse and drop only | 600,000 to 700,000 | about 820 |
/// | parse, plan and drop | 600,000 to 640,000 | about 860 |
/// | all four, including eval | 600,000 to 640,000 | about 860 |
///
/// Adding `eval` does not move the cliff, because the phases are sequential
/// and each unwinds before the next, so what binds is the **`Drop`** at about
/// 820 bytes per level, slightly more than `eval`'s 784. Sizing against
/// `eval`'s number alone is optimistic by roughly ten per cent; the honest
/// budget is about 860 bytes per level, or roughly 620,000 levels here.
///
/// One consequence Task 11 should not have to rediscover: a depth limit on
/// `eval` **does not close the abort path**. `exit` followed by a 700,000-term
/// expression aborts in the drop with no evaluation at all, and no counter in
/// this crate is in a position to see it.
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
/// The probe itself perturbs the frame it measures, by the width of the local
/// whose address it takes. That biases the answer upward by a few bytes per
/// level, which is the safe direction for sizing a stack.
#[derive(Copy, Clone, Debug, Default)]
pub struct StackSpan {
    /// The deepest `eval` recursion the run reached. Zero if it never
    /// evaluated an expression at all.
    pub max_depth: usize,
    /// Stack bytes between the first `eval` level and the deepest one.
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
    /// An instruction 4a does not execute. `keyword` is `None` for the four
    /// clause shapes no keyword introduces, and their names come from the
    /// shape rather than from a keyword table.
    fn instruction(kind: &InstructionKind) -> Loud {
        let name = match kind.keyword() {
            Some(keyword) => keyword,
            None => match kind {
                InstructionKind::Assignment { .. } => "an assignment",
                InstructionKind::Label { .. } => "a label",
                InstructionKind::Message { .. } => "a message send",
                InstructionKind::Command { .. } => "a command",
                _ => unreachable!("keyword() answers Some for every other kind"),
            },
        };
        Loud {
            message: format!(
                "{name} is not implemented: Task 3's spike executes SAY, EXIT, assignment to a \
                 simple variable, and `||` only"
            ),
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
    fn expression(kind: &ExprKind) -> Loud {
        Loud {
            message: format!(
                "{} is not implemented: Task 3's spike evaluates a literal, a constant, a simple \
                 variable and `||` only",
                form_name(kind)
            ),
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

/// A loaded program's identity.
///
/// A small integer the loader hands out, never a pointer: D16 requires that
/// the plan cache's key cannot be reused by a different program, and an
/// address can be, once an `Rc` drops and the allocator reuses the block.
/// `Interp::programs` holds an `Rc` for every id it has issued, so an id
/// outlives every plan keyed against it by construction.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
struct ProgramId(usize);

/// Which code body of which loaded program a cached plan belongs to (D16).
///
/// There is deliberately **no fragment arm**, and that is a finding rather
/// than an omission. D16 says a fragment's plan is keyed by `(enclosing body,
/// fragment id)`, but a fragment is re-parsed on every execution of its
/// `INTERPRET` and its text can differ per iteration, so a "fragment id" can
/// only be a counter handed out per parse. Every lookup against such a key
/// misses and every insert stays forever, so `do 1000000; interpret s; end`
/// would accumulate a million plans that are each read zero times. The
/// durable part of a fragment's resolution is not its plan but the
/// name-to-slot bindings it adds to the enclosing activation, and those live
/// on `Activation::extra`. So a fragment plan is built, used, and dropped with
/// the fragment. See `Interp::fragment_plan`.
#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
struct BodyKey {
    program: ProgramId,
    /// `None` is the program's main body. 4b is the first to need
    /// `Some(index)` for `directives[index]`'s body.
    directive: Option<usize>,
}

/// One body's variable-resolution plan, built by one upfront pass at first
/// execution (D16).
///
/// Two views of the same assignment. `names` is the map D16 specifies, keyed
/// by upcased name, and it is what a name resolved at *run time* goes through:
/// `DROP (v)`, and a fragment's names. `by_symbol` is what evaluation goes
/// through, so the hot path is a lookup by the id the AST already carries
/// rather than a byte-string hash.
///
/// `by_symbol` is a `HashMap` and D16's shape wants an array index. It cannot
/// be one yet: `rexx_parse::SymbolId` is a newtype over a private `u32` with
/// no accessor, so nothing outside `rexx-parse` can turn one into a `Vec`
/// index. Task 6 either adds that accessor or keeps the hash; recorded here
/// so the choice is made rather than inherited.
#[derive(Debug, Default)]
struct Plan {
    names: HashMap<Box<[u8]>, usize>,
    by_symbol: HashMap<SymbolId, usize>,
}

impl Plan {
    /// Walks `body` once and returns a finished table (D16: "built by one
    /// upfront pass", not populated lazily one name at a time).
    fn build(body: &CodeBody, symbols: &SymbolTable) -> Plan {
        let mut plan = Plan::default();
        for instruction in &body.instructions {
            match &instruction.kind {
                InstructionKind::Assignment { target, value } => {
                    plan.note(target, symbols);
                    plan.note(value, symbols);
                }
                InstructionKind::Say {
                    expression: Some(expression),
                } => plan.note(expression, symbols),
                InstructionKind::Interpret { expression } => plan.note(expression, symbols),
                // Every other instruction fails loudly before evaluation ever
                // reaches it, so a variable inside one can never be read and
                // needs no slot. Task 6 makes this pass exhaustive over
                // `InstructionKind`, which is the point at which the omission
                // would start to matter.
                _ => {}
            }
        }
        plan
    }

    /// Assigns slots to every variable `expr` names, in source order.
    ///
    /// Recursive, and that recursion is on the interpreter thread's stack
    /// budget alongside `eval`'s: a left-deep 100,000-term expression is
    /// walked here as deeply as it is later evaluated.
    fn note(&mut self, expr: &Expr, symbols: &SymbolTable) {
        match &expr.kind {
            ExprKind::Variable(id) => self.bind(*id, symbols.name(*id).as_bytes()),
            ExprKind::Prefix { operand, .. } => self.note(operand, symbols),
            ExprKind::Binary { left, right, .. } => {
                self.note(left, symbols);
                self.note(right, symbols);
            }
            // A literal and a constant name no variable; every remaining form
            // fails loudly in `eval` before its names could be read. Task 6
            // covers `Stem`, `Compound` and the rest, where D16's rule that a
            // tail piece lands on the *same* slot as a same-named variable is
            // what `names` exists for.
            _ => {}
        }
    }

    /// Binds `name` to a slot, and `id` to the same one.
    ///
    /// Both views are updated together because they are one assignment seen
    /// two ways: a second symbol spelling the same name must land on the slot
    /// the first one got, which is why the slot number comes from `names` and
    /// never from `by_symbol`'s length.
    fn bind(&mut self, id: SymbolId, name: &[u8]) {
        let next = self.names.len();
        let slot = *self.names.entry(name.into()).or_insert(next);
        self.by_symbol.insert(id, slot);
    }

    fn len(&self) -> usize {
        self.names.len()
    }
}

/// One activation: everything about the frame currently executing.
///
/// `Settings` is per activation and not one field on `Interp` (measured, in
/// the design's "The borrow shape"), so 4b's frame carries its own. 4a has one
/// frame, and the spike has no `NUMERIC` instruction, so there is nothing to
/// carry yet and the field arrives with Task 9.
struct Activation {
    /// The program this frame is running.
    ///
    /// **A liveness anchor, and never borrowed through.** Nothing takes
    /// `&self.activations.last().program.…` and then calls a `&mut self`
    /// method: that is the `E0502` written out in `run_activation`. This field
    /// exists so that the `Rc` the instruction loop clones into its local has
    /// something to be cloned from, and so that a frame keeps its program
    /// alive independently of `Interp::programs`.
    program: Rc<Program>,
    plan: Rc<Plan>,
    /// Names bound after `plan` was built, and the reason this field exists is
    /// the whole answer to "does a fragment's plan work against the enclosing
    /// plan's name map".
    ///
    /// It does for reads, and it cannot for writes. A fragment that introduces
    /// a name the enclosing body never mentions has to bind that name to a
    /// slot, and the binding has to outlive the fragment. Measured on the
    /// oracle:
    ///
    /// ```text
    /// interpret "zork = 42"      /* ZORK is in no instruction of this body */
    /// interpret "say zork"       /* prints 42 */
    /// ```
    ///
    /// The enclosing `plan` is an `Rc` the activation holds a clone of, so it
    /// is not uniquely owned and cannot be extended; `RootSet::grow_slots`
    /// hands out the *slot* but records no *name* for it. This map is where
    /// the name goes. `DROP (v)` has the identical hole, so this is not a
    /// fragment-only mechanism.
    extra: HashMap<Box<[u8]>, usize>,
    frame: SlotFrame,
    pc: usize,
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
    out: Vec<u8>,
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
    /// Stack addresses of a local inside `eval`, at the first level and at the
    /// deepest one. `usize::MAX`/0 are the "nothing measured yet" values, and
    /// `stack_span` reports nothing rather than a difference against them.
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
            stack_first: 0,
            stack_deepest: usize::MAX,
        }
    }

    // ---- loading and the activation stack ----

    /// Loads `program`, runs its main body in a fresh activation, and tears
    /// the activation down again.
    fn run(&mut self, program: Program) -> Result<Option<ObjRef>, Loud> {
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
        self.activations.push(Activation {
            program: Rc::clone(&program),
            plan,
            extra: HashMap::new(),
            frame,
            pc: 0,
        });

        let exit = self.run_activation();

        // Popped whether or not the body raised, so the root set is left the
        // way it was found even on the failure path.
        let activation = self.activations.pop().expect("the frame just pushed");
        self.roots.pop_slots(activation.frame);
        exit
    }

    /// The plan for one body, from the cache or built and cached (D16:
    /// "cached on `Interp`, not on the body", because an `Rc<Program>` gives
    /// shared immutable access and nothing can be written into a `CodeBody`
    /// reached through one).
    fn plan_for(&mut self, key: BodyKey, body: &CodeBody, symbols: &SymbolTable) -> Rc<Plan> {
        if let Some(plan) = self.plans.get(&key) {
            return Rc::clone(plan);
        }
        let plan = Rc::new(Plan::build(body, symbols));
        self.plans.insert(key, Rc::clone(&plan));
        plan
    }

    fn activation(&self) -> &Activation {
        self.activations.last().expect("a live activation")
    }

    fn activation_mut(&mut self) -> &mut Activation {
        self.activations.last_mut().expect("a live activation")
    }

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
    /// That is not a paraphrase. It was compiled in this file, and this is
    /// what rustc 1.96.1 said about it:
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
    /// Everything above is prose. It was true when it was written, and nothing
    /// in the tree would notice if it stopped being true, so the two snippets
    /// that follow are the checkable form of it: a miniature of the same
    /// borrow, once in the shape that compiles and once in the shape that does
    /// not. `cargo test` runs both.
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
    /// differ by one line and share everything else, so any breakage in the
    /// shared part fails the passing twin instead of silently satisfying the
    /// failing one. Checked by mutation rather than assumed: rewriting the
    /// passing snippet's `Rc::clone` line into the failing snippet's shape
    /// makes it fail, and it fails with `E0502`. One test says "the fix
    /// works", the other says "the shape it fixes is still broken", and
    /// neither is worth much alone.
    ///
    /// What the pair still cannot catch is a typo confined to the failing
    /// snippet's own one different line. Nothing available here closes that,
    /// and it is smaller than what is closed.
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
    fn run_activation(&mut self) -> Result<Option<ObjRef>, Loud> {
        let program = Rc::clone(&self.activation().program);
        let plan = Rc::clone(&self.activation().plan);
        let code = Code {
            body: &program.main,
            symbols: &program.symbols,
            slots: &plan.by_symbol,
        };

        while let Some(instruction) = code.body.instructions.get(self.activation().pc) {
            match self.step(&code, instruction)? {
                Flow::Next => self.activation_mut().pc += 1,
                Flow::Goto(target) => self.activation_mut().pc = target,
                Flow::Exit(value) => return Ok(value),
            }
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
    fn step(&mut self, code: &Code<'_>, instruction: &Instruction) -> Result<Flow, Loud> {
        match &instruction.kind {
            InstructionKind::Say { expression } => {
                let line = match expression {
                    Some(expression) => {
                        let value = self.eval(code, expression)?;
                        self.text_of(value).to_vec()
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
                // here; the spike takes the first and Task 5 takes the others.
                let ExprKind::Variable(id) = &target.kind else {
                    return Err(Loud::expression(&target.kind));
                };
                let name = code.symbols.name(*id).as_bytes();
                let value = self.eval(code, value)?;
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
                let text = self.text_of(value).to_vec();
                self.run_fragment(text)
            }

            other => Err(Loud::instruction(other)),
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
    ///   instruction and has to still be there afterwards. A local is enough
    ///   for the measured reason that a fragment can never jump: a label
    ///   inside `INTERPRET` text is error 47.1 (Task 1), so `body.labels` is
    ///   always empty.
    /// * No frame is pushed. The fragment's assignments land in the enclosing
    ///   frame's slots, which is what `fragment_plan` resolves them against,
    ///   and it is why `RootSet::grow_slots`'s top-frame assertion holds here.
    fn run_fragment(&mut self, text: Vec<u8>) -> Result<Flow, Loud> {
        let fragment: Rc<Fragment> = match parse_interpret(text) {
            Ok(fragment) => Rc::new(fragment),
            Err(error) => return Err(Loud::parse(&error)),
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
            match self.step(&code, instruction)? {
                Flow::Next => pc += 1,
                Flow::Goto(_) => unreachable!("a fragment has no labels, so it cannot jump (47.1)"),
                // `exit` inside `INTERPRET` ends the program, not the
                // fragment, so this propagates rather than stopping here.
                Flow::Exit(value) => return Ok(Flow::Exit(value)),
            }
        }
        Ok(Flow::Next)
    }

    /// Resolves a fragment's own `SymbolId`s to slots in the **enclosing**
    /// frame.
    ///
    /// This is D16's "its plan is built against the enclosing plan's name map"
    /// and it goes through `slot_of`, which is that name map plus the two
    /// things D16 does not mention: the activation's `extra` bindings, and
    /// growth for a name nobody has bound yet. A fragment's ids are its own,
    /// and `parse_interpret` builds a fresh `SymbolTable` every call, so id 7
    /// in the fragment and id 7 in the program name unrelated symbols -- the
    /// join has to be through the text, `fragment.symbols.name(id)`, and this
    /// is the only place that matters.
    ///
    /// The result is returned rather than cached, for the reason `BodyKey`
    /// gives.
    fn fragment_plan(&mut self, fragment: &Fragment) -> HashMap<SymbolId, usize> {
        // The same upfront pass, run against the fragment's own body, which
        // numbers its names 0..n in walk order. Those numbers are local to the
        // fragment and mean nothing to the enclosing frame; the loop below is
        // what translates them.
        let local = Plan::build(&fragment.body, &fragment.symbols);

        // Walk order, recovered from the local numbering rather than from
        // iterating the map, because a `HashMap`'s order varies run to run and
        // the enclosing frame's slots would then be allocated in a different
        // order each time. Nothing observable depends on that order today,
        // which is the reason to fix it now rather than after something does.
        let mut by_local: Vec<&[u8]> = vec![b""; local.len()];
        for (name, slot) in &local.names {
            by_local[*slot] = name;
        }
        let enclosing: Vec<usize> = by_local.iter().map(|name| self.slot_of(name)).collect();

        local
            .by_symbol
            .iter()
            .map(|(id, local_slot)| (*id, enclosing[*local_slot]))
            .collect()
    }

    // ---- the variable pool ----

    /// The slot `name` resolves to in the current frame, allocating one if it
    /// resolves to none.
    ///
    /// Three sources in order, and the third is the one D16 leaves out. The
    /// plan's name map is the upfront pass's answer. `extra` is every binding
    /// made since, which is where a fragment's new names and `DROP (v)`'s
    /// run-time target land. Growth is what happens when neither has it:
    /// `RootSet::grow_slots` extends the frame, and the name is recorded
    /// **here**, because the plan is an `Rc` and cannot be extended.
    fn slot_of(&mut self, name: &[u8]) -> usize {
        let activation = self.activation();
        if let Some(slot) = activation.plan.names.get(name) {
            return *slot;
        }
        if let Some(slot) = activation.extra.get(name) {
            return *slot;
        }
        let frame = activation.frame;
        let slot = self.roots.grow_slots(frame);
        self.activation_mut().extra.insert(name.into(), slot);
        slot
    }

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
                let derived = code.symbols.name(id).as_bytes().to_vec();
                (self.alloc_text(derived), Novalue::Unset)
            }
        }
    }

    // ---- values ----

    fn alloc_text(&mut self, bytes: Vec<u8>) -> ObjRef {
        self.heap
            .alloc_with(BehaviourId::STRING, Body::Text { bytes, num: None })
    }

    fn text_of(&self, value: ObjRef) -> &[u8] {
        match &self.heap.get(value).expect("a live value").body {
            Body::Text { bytes, .. } => bytes,
            other => unreachable!("the spike allocates only Body::Text, got {other:?}"),
        }
    }

    // ---- expression evaluation ----

    /// Evaluates one expression node, and keeps the depth bookkeeping D19
    /// needs.
    ///
    /// Split from `eval_node` so that the depth is decremented on every exit
    /// path including the `?` ones, without a guard type that would need to
    /// hold a borrow of `self` across the recursive call. Task 11 adds the
    /// limit check to this function, which is why it is the one that owns the
    /// counter.
    ///
    /// The stack probe: the address of a local here, recorded at the first
    /// level and at the deepest. Taking a raw pointer and casting it to
    /// `usize` is safe code, so this needs no `unsafe`, and measuring the real
    /// function rather than a replica of it is the whole reason to do it here.
    fn eval(&mut self, code: &Code<'_>, expr: &Expr) -> Result<ObjRef, Loud> {
        let probe = 0u8;
        let here = &probe as *const u8 as usize;

        self.depth += 1;
        if self.depth == 1 {
            self.stack_first = here;
        }
        if self.depth > self.max_depth {
            self.max_depth = self.depth;
            self.stack_deepest = here;
        }

        let value = self.eval_node(code, expr);
        self.depth -= 1;
        value
    }

    fn eval_node(&mut self, code: &Code<'_>, expr: &Expr) -> Result<ObjRef, Loud> {
        match &expr.kind {
            ExprKind::Literal(bytes) => Ok(self.alloc_text(bytes.to_vec())),
            // A constant's value is its own upcased spelling, which is
            // observable rather than incidental: `say 1e5` prints `1E5`.
            ExprKind::Constant(id) => {
                let bytes = code.symbols.name(*id).as_bytes().to_vec();
                Ok(self.alloc_text(bytes))
            }
            ExprKind::Variable(id) => {
                let (value, _novalue) = self.read(code, *id);
                Ok(value)
            }

            // One operator, chosen because it needs no arithmetic and so no
            // `rexx-num` dependency yet, while still being a genuinely
            // recursive left-deep chain: `'' || '' || …` is the same tree
            // shape as D19's `1 + 1 + …`.
            ExprKind::Binary {
                op: Operator::Concatenate,
                left,
                right,
            } => {
                // The temps discipline, established here rather than
                // retrofitted: a value held only in a Rust local is invisible
                // to the collector, so it is pushed before anything that can
                // allocate runs. `Heap::alloc` does not collect on its own
                // today, which makes this belt and braces at the moment and
                // load-bearing the day allocation triggers a collection.
                let frame = self.roots.push_frame();
                let left_value = self.eval(code, left)?;
                self.roots.push_temp(left_value);
                let right_value = self.eval(code, right)?;
                self.roots.push_temp(right_value);

                let mut bytes = self.text_of(left_value).to_vec();
                bytes.extend_from_slice(self.text_of(right_value));
                let joined = self.alloc_text(bytes);

                // `joined` is unrooted from here to the caller's own
                // `push_temp`, and nothing between the two allocates.
                self.roots.pop_frame(frame);
                Ok(joined)
            }

            other => Err(Loud::expression(other)),
        }
    }

    fn stack_span(&self) -> StackSpan {
        StackSpan {
            max_depth: self.max_depth,
            bytes: self.stack_first.saturating_sub(self.stack_deepest),
        }
    }
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
/// The rejected alternative is worth recording: hooking the fragment onto an
/// innocent instruction such as `NOP` proves the same lifetime while lying
/// about which node owns it, and leaves nothing for 4b to delete.
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
        Err(loud) => {
            stderr.extend_from_slice(format!("rexx-exec: {}\n", loud.message).as_bytes());
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
