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

//! **The corpus-wide net over what `compile` emitted, as an invariant rather
//! than a transcript.**
//!
//! `golden_tests.rs` pins the op stream of a handful of hand-written programs
//! exactly; the corpus differential pins that every body *compiled*. Neither
//! sees a promotion that stops firing for a construct in some shape the
//! golden set does not spell out, because the corpus harness compares the two
//! engines' output and a construct that falls back to `Op::Generic` produces
//! the same output by delegating to the tree-walker.
//!
//! So this module states the promotion set as a **derived expectation**: for
//! every body of every corpus program, what construct each instruction is
//! decides what its compiled clause must look like, and the compiled stream is
//! checked against that. Nothing is committed, so nothing churns when an
//! unrelated op is added.
//!
//! **What this cannot see, stated plainly.** It reads an op's *kind* and never
//! its operands, so a register number, a jump target, a region end, a constant
//! index or a `SymbolId` that is wrong is invisible here -- a transcript would
//! catch those and this does not. It says nothing about the ops it does not
//! name: a dropped `Op::TraceLiteral`, `Op::TraceRead`, `Op::TraceOperator`,
//! `Op::EndBranch` or `Op::Jump` passes. And it is a claim about what
//! compilation emitted, not about what running it does. The exact streams in
//! `golden_tests.rs` cover contents over a few programs; this covers kind over
//! every corpus program, and neither contains the other.
//!
//! **The expectations are computed here rather than asked of `compile`**, and
//! that is the whole of why this can fail. [`root_of`] restates what
//! `native_shape` and `push_native` decide, and [`promoted_as`] restates which
//! instruction arms `compile` has; a version of either that called into
//! `compile` would move with the code it is checking and could never redden.
//! The price is that the two can drift, and the drift is loud in both
//! directions: an operator dropped from the promoted set leaves an
//! `Op::EvalExpr` where a computing op is expected, and one added leaves a
//! computing op where an `Op::EvalExpr` is expected. Either reddens and names
//! the program.
//!
//! **What this catches that the rest of the suite does not, measured rather
//! than argued.** Each mutation below was applied to `compile` and the whole
//! workspace run under it with `--no-fail-fast`.
//!
//! * Making `CALL name` fall through to `Op::Generic` -- a whole promotion
//!   ceasing to fire -- reddens this, and reddens tests in `golden_tests.rs`
//!   and `drive/tests.rs` as well, and aborts `tests/spike.rs` on a stack
//!   overflow. **For that mutation this file adds nothing**; it is a more
//!   direct signal rather than a new one.
//! * Bounding `native_shape` at depth 8 -- a promotion that keeps firing for a
//!   shallow shape and stops firing for a deeper one -- reddens **this and
//!   `golden_tests.rs`'s
//!   `a_call_nested_past_the_paths_width_leaves_the_slot_general`, and nothing
//!   else in the workspace**. That is the class this file exists for, and the
//!   depth bound has two halves that those two witnesses split between them.
//!   Measured 2026-08-12, on two mutations run over the whole workspace with
//!   `--no-fail-fast`: refusing *every node* past depth 8 reddens both, while
//!   refusing only an *address* past depth 8 reddens the golden test alone and
//!   leaves this file green. `corpus/lang/deep_nested_expr.rex` is why -- a
//!   single assignment whose header says it nests three thousand terms on
//!   purpose and holds no call anywhere in them, so it reaches a bound on
//!   nodes and cannot reach one on addresses. The golden test reaches both,
//!   by generating a nesting and putting a call at the bottom of it.
//!   `push_native` recurses once per operator, so such a bound is a change
//!   Phase 4f might reasonably want, which is what makes the class live rather
//!   than hypothetical.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use rexx_parse::{
    Call, CodeBody, DirectiveKind, Expr, ExprKind, InstructionKind, Operator, parse_program,
};

use super::golden::render;
use super::{Chunk, Op};
use crate::plan::Plan;
use crate::trace::{ChunkTrace, TraceMode};

/// The op a promoted clause's value expression must end in.
///
/// One variant per arm of `push_native`, plus the fallback: `EvalExpr` is what
/// an expression outside the native set compiles to, and naming it here is
/// what makes the check bidirectional rather than a one-way "something
/// promoted".
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
enum Root {
    Const,
    LoadConstant,
    Load,
    Arith,
    Binary,
    Prefix,
    CallExpr,
    EvalExpr,
}

impl Root {
    /// The `Root` an emitted op is, or `None` for an op that produces no
    /// value.
    ///
    /// Exhaustive with no catch-all, so an op variant added to [`Op`] has to
    /// be classified here rather than silently counting as "not a value".
    fn of(op: &Op) -> Option<Root> {
        match op {
            Op::Const { .. } => Some(Root::Const),
            Op::LoadConstant { .. } => Some(Root::LoadConstant),
            Op::Load { .. } => Some(Root::Load),
            Op::Arith { .. } => Some(Root::Arith),
            Op::Binary { .. } => Some(Root::Binary),
            Op::Prefix { .. } => Some(Root::Prefix),
            Op::EvalExpr { .. } => Some(Root::EvalExpr),
            Op::CallExpr { .. } => Some(Root::CallExpr),
            Op::Generic { .. }
            | Op::TraceKeyword { .. }
            | Op::LoopHeaderValue { .. }
            | Op::LoopRun { .. }
            | Op::Clause { .. }
            | Op::TraceClause { .. }
            | Op::SelectCaseText { .. }
            | Op::WhenTest { .. }
            | Op::TraceLiteral { .. }
            | Op::TraceRead { .. }
            | Op::TraceOperator { .. }
            | Op::TracePrefix { .. }
            | Op::Store { .. }
            | Op::Say { .. }
            | Op::Call { .. }
            | Op::TraceFunction { .. }
            | Op::EndBranch
            | Op::EnterWhen { .. }
            | Op::EnterOtherwise { .. }
            | Op::Jump { .. }
            | Op::JumpUnless { .. } => None,
        }
    }
}

/// Which construct of the minimum promotion set an instruction is, or `None`
/// for one outside it.
///
/// `listed` is the set of instruction indices some `SELECT` collected as its
/// own `WHEN`s: an *absorbed* `WHEN` -- itself another `WHEN`'s consequence --
/// is nobody's listed branch and compiles to `Op::Generic`, so it is not in
/// the set. Computed from the `SELECT` nodes rather than from `compile`'s own
/// `when_info`, for the reason the module doc gives.
///
/// **The `None` arm carries no claim.** An instruction outside the set may
/// compile to anything at all, so promoting a construct this does not name
/// cannot redden the assertions below. What it may not do is stop firing for
/// one that is named.
fn promoted_as(kind: &InstructionKind, index: usize, listed: &[usize]) -> Option<&'static str> {
    match kind {
        InstructionKind::Do(_) => Some("DO"),
        InstructionKind::Loop(_) => Some("LOOP"),
        InstructionKind::If { .. } => Some("IF"),
        InstructionKind::Select { .. } => Some("SELECT"),
        InstructionKind::When { .. } if listed.contains(&index) => Some("WHEN"),
        InstructionKind::WhenCase { .. } if listed.contains(&index) => Some("WHEN CASE"),
        InstructionKind::Assignment { .. } => Some("assignment"),
        InstructionKind::Say { .. } => Some("SAY"),
        InstructionKind::Call(call) if matches!(&**call, Call::Named { .. }) => Some("CALL name"),
        _ => None,
    }
}

/// The op the value expression of a promoted `Assignment` or `SAY` must end
/// in.
///
/// A restatement of `native_shape` followed by `push_native`, over the parse
/// tree alone. Every operator is spelled out below rather than asked of
/// `eval::is_arithmetic` or `eval::is_native_binary`, which is what `compile`
/// asks: an expectation computed by the code under test is an expectation that
/// agrees with it whatever it does.
fn root_of(expr: &Expr) -> Root {
    match &expr.kind {
        // A call at the root is the whole slot's expression, so the route down
        // to it has no steps and every address reaches it. A call *below* the
        // root is an operand and never the last op, so it is [`native`]'s
        // business rather than this function's.
        ExprKind::Call { .. } => Root::CallExpr,
        ExprKind::Literal(_) => Root::Const,
        ExprKind::Constant(_) => Root::LoadConstant,
        ExprKind::Variable(_) | ExprKind::Stem(_) | ExprKind::Compound(_) => Root::Load,
        ExprKind::Binary { op, left, right } if native(left, 1) && native(right, 1) => {
            if arithmetic(*op) {
                Root::Arith
            } else if other_family(*op) {
                Root::Binary
            } else {
                Root::EvalExpr
            }
        }
        // No operator condition beside the operand's, unlike the arm above.
        // The expectation this file states is that a prefix promotes whatever
        // its operator, and one that did not would leave an `Op::EvalExpr`
        // where this arm calls for `Root::Prefix` -- which reddens rather than
        // passes, so the arm is the claim and not an assumption behind it.
        ExprKind::Prefix { operand, .. } if native(operand, 1) => Root::Prefix,
        _ => Root::EvalExpr,
    }
}

/// The deepest a call can sit below its slot's root and still be addressed by
/// an op of its own: one step per bit a `u32` holds below the sentinel bit
/// that marks where the route starts.
///
/// Written out here rather than read off `super::NodePath`, for the reason the
/// module doc gives about the operator sets: a bound taken from the code under
/// test moves with it and could never redden.
///
/// **Only a corpus program nesting a call deeper than this can falsify the
/// number, and that is the price of the independence.** A `NodePath` whose
/// width moved would redden
/// `super::tests::a_node_path_carries_thirty_one_steps_and_refuses_the_thirty_second`
/// and `golden_tests`'s
/// `a_call_nested_past_the_paths_width_leaves_the_slot_general` while this
/// number went on calling for `Root::EvalExpr` at the depths the wider address
/// had just reached. Whoever widens one widens this, and this sentence is what
/// says so.
const DEEPEST_ADDRESSED_CALL: usize = 31;

/// Whether every part of `expr` has a native op, which is what licenses the
/// whole tree compiling without `eval.rs` being entered. `depth` is how many
/// operators stand between `expr` and its slot's own root.
///
/// **Only a call reads `depth`.** A call's op carries the route down to the
/// node, so a call standing deeper than a route reaches has no op and takes
/// its whole slot general with it; every other shape here is computed into a
/// register the operator above names, is addressed by nothing, and promotes
/// however deep it stands.
fn native(expr: &Expr, depth: usize) -> bool {
    match &expr.kind {
        ExprKind::Literal(_)
        | ExprKind::Constant(_)
        | ExprKind::Variable(_)
        | ExprKind::Stem(_)
        | ExprKind::Compound(_) => true,
        ExprKind::Call { .. } => depth <= DEEPEST_ADDRESSED_CALL,
        ExprKind::Binary { op, left, right } => {
            (arithmetic(*op) || other_family(*op))
                && native(left, depth + 1)
                && native(right, depth + 1)
        }
        ExprKind::Prefix { operand, .. } => native(operand, depth + 1),
        _ => false,
    }
}

/// The operators `Interp::eval_arithmetic` computes, as this file's own
/// statement of the set.
fn arithmetic(op: Operator) -> bool {
    matches!(
        op,
        Operator::Plus
            | Operator::Subtract
            | Operator::Multiply
            | Operator::Divide
            | Operator::IntDiv
            | Operator::Remainder
            | Operator::Power
    )
}

/// The operators `Interp::apply_binary` computes -- concatenation, comparison
/// and logical -- as this file's own statement of that set, spelled out for
/// the reason [`arithmetic`] is.
///
/// The prefix `\` is deliberately in neither list: the parser builds an
/// `ExprKind::Prefix` from it, so no expression this walks can hold one as a
/// binary operator, and a row for it here would be an expectation about a tree
/// shape the corpus cannot contain.
fn other_family(op: Operator) -> bool {
    matches!(
        op,
        Operator::Concatenate
            | Operator::Abuttal
            | Operator::Blank
            | Operator::Equal
            | Operator::BackslashEqual
            | Operator::GreaterThan
            | Operator::BackslashGreaterThan
            | Operator::LessThan
            | Operator::BackslashLessThan
            | Operator::GreaterThanEqual
            | Operator::LessThanEqual
            | Operator::StrictEqual
            | Operator::StrictBackslashEqual
            | Operator::StrictGreaterThan
            | Operator::StrictBackslashGreaterThan
            | Operator::StrictLessThan
            | Operator::StrictBackslashLessThan
            | Operator::StrictGreaterThanEqual
            | Operator::StrictLessThanEqual
            | Operator::LessThanGreaterThan
            | Operator::GreaterThanLessThan
            | Operator::And
            | Operator::Or
            | Operator::Xor
    )
}

/// The compiled region of each promoted clause, keyed by instruction index:
/// the ops strictly inside `Op::Clause`'s own `(here, end)`.
fn regions(chunk: &Chunk) -> BTreeMap<u32, &[Op]> {
    let mut out = BTreeMap::new();
    for (at, op) in chunk.ops.iter().enumerate() {
        if let Op::Clause { index, end } = op {
            let previous = out.insert(*index, &chunk.ops[at + 1..*end as usize]);
            assert!(
                previous.is_none(),
                "instruction {index} opened two Clause regions"
            );
        }
    }
    out
}

/// The instruction indices every `SELECT` in `body` collected as its own
/// `WHEN`s.
fn listed_whens(body: &CodeBody) -> Vec<usize> {
    let mut out = Vec::new();
    for instruction in &body.instructions {
        if let InstructionKind::Select { whens, .. } = &instruction.kind {
            out.extend(whens.iter().copied());
        }
    }
    out
}

/// Every construct and every expression root this sweep saw fire, so that a
/// row the corpus never reaches is a loud gap rather than a silently
/// satisfied assertion.
#[derive(Default)]
struct Seen {
    constructs: BTreeMap<&'static str, usize>,
    roots: BTreeMap<Root, usize>,
}

/// Checks one body's compiled stream against what its instructions call for,
/// and records what fired.
///
/// `where_` names the program and the body, and it is on every message: a
/// failure that did not say which of the corpus programs produced it would
/// leave the reader running the sweep by hand.
fn check_body(body: &CodeBody, symbols: &rexx_parse::SymbolTable, where_: &str, seen: &mut Seen) {
    let plan = Plan::build(body, symbols);
    let chunk = match super::compile(body, &plan, ChunkTrace::of(TraceMode::NORMAL)) {
        Ok(chunk) => chunk,
        // The one error `compile` has is a machine width, and no corpus
        // program is four billion instructions long. Loud rather than
        // skipped, because a refusal here would make every assertion below
        // vacuous for that body.
        Err(error) => panic!("{where_}: compile refused this body: {error:?}"),
    };
    let regions = regions(&chunk);
    let generic: Vec<u32> = chunk
        .ops
        .iter()
        .filter_map(|op| match op {
            Op::Generic { index } => Some(*index),
            _ => None,
        })
        .collect();
    let listed = listed_whens(body);

    for (index, instruction) in body.instructions.iter().enumerate() {
        let Some(construct) = promoted_as(&instruction.kind, index, &listed) else {
            continue;
        };
        let at = u32::try_from(index).expect("a corpus body is not four billion instructions");
        *seen.constructs.entry(construct).or_default() += 1;

        assert!(
            !generic.contains(&at),
            "{where_}: instruction {index} ({construct}) is in the minimum promotion set and \
             compiled to Op::Generic\n{}",
            render(&chunk)
        );
        let region = regions.get(&at).unwrap_or_else(|| {
            panic!(
                "{where_}: instruction {index} ({construct}) opened no Clause region\n{}",
                render(&chunk)
            )
        });

        // The value expression, for the two constructs that have one that can
        // compile natively. Every other promoted construct evaluates through
        // `Op::EvalExpr` unconditionally, so there is nothing here to state
        // about it that its presence in the stream has not already said.
        let value = match &instruction.kind {
            InstructionKind::Assignment { value, .. } => Some(value),
            InstructionKind::Say { expression } => expression.as_ref(),
            _ => continue,
        };
        let expected = value.map(root_of);
        let actual = region.iter().filter_map(Root::of).next_back();
        assert_eq!(
            actual,
            expected,
            "{where_}: instruction {index} ({construct}) has an expression calling for \
             {expected:?} and a compiled clause ending in {actual:?}\n{}",
            render(&chunk)
        );
        if let Some(root) = expected {
            *seen.roots.entry(root).or_default() += 1;
        }
    }
}

/// Every body of `program`: the main one and each directive's.
///
/// Exhaustive over `DirectiveKind` with no catch-all, so a directive form that
/// gains a body has to be routed here rather than dropping out of the sweep.
fn bodies(program: &rexx_parse::Program) -> Vec<(&CodeBody, String)> {
    let mut out = vec![(&program.main, "main".to_string())];
    for directive in &program.directives {
        let (body, what) = match &directive.kind {
            DirectiveKind::Method(method) => (method.body.as_ref(), "::METHOD"),
            DirectiveKind::Attribute(attribute) => (attribute.body.as_ref(), "::ATTRIBUTE"),
            DirectiveKind::Routine(routine) => (routine.body.as_ref(), "::ROUTINE"),
            DirectiveKind::Annotate(_)
            | DirectiveKind::Class(_)
            | DirectiveKind::Constant(_)
            | DirectiveKind::Options(_)
            | DirectiveKind::Requires(_)
            | DirectiveKind::Resource(_) => (None, ""),
        };
        if let Some(body) = body {
            out.push((body, what.to_string()));
        }
    }
    out
}

fn corpus_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus")
}

/// Every corpus program named by a phase subset file, each once, sorted.
///
/// The subset files are read from the directory rather than listed here.
/// `ir_dual.rs` keeps a literal pinned against the same listing, and a third
/// copy of that literal is a third thing to keep in step; taking the listing
/// directly is one fewer, and a phase subset file added later is swept without
/// anyone remembering this file.
fn corpus_programs() -> Vec<String> {
    let dir = corpus_dir();
    let entries =
        fs::read_dir(&dir).unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()));
    let mut lists: Vec<String> = entries
        .flatten()
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| name.starts_with("phase-") && name.ends_with(".txt"))
        .collect();
    lists.sort();
    assert!(
        !lists.is_empty(),
        "no phase subset files under {} -- this sweep would run on nothing",
        dir.display()
    );

    let mut names: Vec<String> = Vec::new();
    for list in lists {
        let path = dir.join(&list);
        let text = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
        for line in text.lines().map(str::trim) {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            names.push(line.to_string());
        }
    }
    names.sort();
    names.dedup();
    names
}

/// **Every instruction of every corpus body that the minimum promotion set
/// covers compiles to that construct's own ops**, and every promoted value
/// expression ends in the op its shape calls for.
///
/// The net the phase-4e gate's criterion 6 asks for. It is what would go red
/// if a promotion silently stopped firing for a construct in a shape
/// `golden_tests.rs` does not spell out -- and the corpus differential would
/// not, because an instruction that falls back to `Op::Generic` produces
/// byte-identical output by delegating to the tree-walker.
///
/// One test over the whole population rather than one per program: the
/// population is read at run time from a directory, so there is no list to
/// generate a test per entry from, and a failure names its program in the
/// message.
#[test]
fn every_corpus_body_compiles_the_minimum_promotion_set_to_its_own_ops() {
    // **On the stack the interpreter compiles bodies on, and that is not a
    // precaution.** `compile`'s expression walk takes a frame per operator,
    // and `corpus/lang/deep_nested_expr.rex` is one assignment nesting three
    // thousand of them. Calling `compile` from a libtest thread is ordinary --
    // `golden_tests` does it throughout -- and what is not ordinary is doing
    // it on a body this deep, which is why this sweep is the one that
    // overflowed and why it is this sweep that asks for a stack rather than
    // the callers around it. `INTERPRETER_STACK_BYTES` is the size because
    // that is what `on_interpreter_thread` gives the same walk in production.
    // Measured 2026-08-12 with the sweep called inline instead: it aborts the
    // whole test binary at `RUST_MIN_STACK=2621440` and passes at `2883584`,
    // against a libtest thread's own 2 MiB.
    //
    // A stack overflow is not a test failure -- Rust's guard page aborts the
    // process, taking every other test in the binary with it -- which is why
    // this is a stack size rather than a depth the sweep watches.
    //
    // A panic is resumed on this thread rather than turned into a failure of
    // its own, so an assertion below still reports its own message.
    let sweep = std::thread::Builder::new()
        .stack_size(crate::INTERPRETER_STACK_BYTES)
        .spawn(sweep_every_corpus_body)
        .expect("spawning the sweep thread");
    if let Err(panic) = sweep.join() {
        std::panic::resume_unwind(panic);
    }
}

/// The sweep itself, split out so that the test above is the thread it runs
/// on and nothing else.
fn sweep_every_corpus_body() {
    let dir = corpus_dir();
    let programs = corpus_programs();
    let mut seen = Seen::default();
    let mut bodies_checked = 0usize;

    for name in &programs {
        let path = dir.join(name);
        let text =
            fs::read(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
        // Loud rather than skipped: a corpus program is a program the oracle
        // runs, so one this crate cannot parse is a defect and not a case to
        // step over. A skip here would also shrink the population without
        // anything saying so.
        let program = parse_program(text)
            .unwrap_or_else(|e| panic!("{name}: a corpus program does not parse: {e:?}"));
        for (body, what) in bodies(&program) {
            check_body(body, &program.symbols, &format!("{name} {what}"), &mut seen);
            bodies_checked += 1;
        }
    }

    // The population is not empty and the bodies are not empty, because every
    // assertion above is inside two loops and an empty one of either passes
    // without checking anything.
    assert!(
        programs.len() > 1,
        "the phase subset files named {} programs",
        programs.len()
    );
    assert!(bodies_checked >= programs.len());

    // **The anti-vacuity control.** Every construct of the minimum promotion
    // set, and every expression root a promoted clause can end in, is reached
    // by this population at least once -- so a row of `promoted_as` or of
    // `root_of` that nothing exercises is red here rather than a check that
    // happens to hold over an empty set.
    //
    // `DO` and `LOOP` are one row: they are the same construct under two
    // spellings and a corpus that writes only one of them is not a gap.
    for construct in [
        "IF",
        "SELECT",
        "WHEN",
        "WHEN CASE",
        "assignment",
        "SAY",
        "CALL name",
    ] {
        assert!(
            seen.constructs.contains_key(construct),
            "no corpus body contains a promoted {construct}, so this sweep says nothing about it"
        );
    }
    assert!(
        seen.constructs.contains_key("DO") || seen.constructs.contains_key("LOOP"),
        "no corpus body contains a promoted DO or LOOP"
    );
    for root in [
        Root::CallExpr,
        Root::Const,
        Root::LoadConstant,
        Root::Load,
        Root::Arith,
        Root::Binary,
        Root::Prefix,
        Root::EvalExpr,
    ] {
        assert!(
            seen.roots.contains_key(&root),
            "no promoted corpus clause has a {root:?} expression, so this sweep says nothing \
             about that shape"
        );
    }
}
