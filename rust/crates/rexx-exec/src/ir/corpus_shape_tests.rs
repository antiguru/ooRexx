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

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use rexx_parse::{
    Call, CodeBody, DirectiveKind, Expr, ExprKind, InstructionKind, Operator, parse_program,
};

use super::golden::render;
use super::{Chunk, ConditionKeyword, Op};
use crate::plan::{BodyKind, Plan};
use crate::trace::{ChunkTrace, TraceMode};

/// The op a promoted clause's value expression must end in.
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
    /// The `Root` an emitted op is, or `None` for an op that is not one an
    /// expression's own value can end in.
    fn of(op: &Op) -> Option<Root> {
        match op {
            Op::Const { .. } => Some(Root::Const),
            Op::LoadConstant { .. } => Some(Root::LoadConstant),
            Op::Load { .. } => Some(Root::Load),
            Op::Arith { .. } => Some(Root::Arith),
            Op::Binary { .. } => Some(Root::Binary),
            Op::Prefix { .. } => Some(Root::Prefix),
            Op::EvalExpr { .. } => Some(Root::EvalExpr),
            Op::CallExpr { .. } | Op::CallArgs { .. } => Some(Root::CallExpr),
            Op::CallNamed { .. }
            | Op::PushArg { .. }
            | Op::TraceArgument { .. }
            | Op::TraceKeyword { .. }
            | Op::LoopHeaderValue { .. }
            | Op::LoopRun { .. }
            | Op::LoopNext { .. }
            | Op::Signal { .. }
            | Op::Parse { .. }
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
            | Op::Return { .. }
            | Op::Queue { .. }
            | Op::Call { .. }
            | Op::Message { .. }
            | Op::Expose { .. }
            | Op::Exec { .. }
            | Op::Escape { .. }
            | Op::TraceFunction { .. }
            | Op::EndBranch
            | Op::EndWhen
            | Op::EnterWhen { .. }
            | Op::EnterOtherwise { .. }
            | Op::Jump { .. }
            | Op::JumpUnless { .. }
            | Op::Condition { .. } => None,
        }
    }
}

/// Which construct of the minimum promotion set an instruction is, or `None`
/// for one outside it.
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
        InstructionKind::Return { .. } => Some("RETURN"),
        InstructionKind::Exit { .. } => Some("EXIT"),
        InstructionKind::Push { .. } => Some("PUSH"),
        InstructionKind::Queue { .. } => Some("QUEUE"),
        InstructionKind::Call(call) if matches!(&**call, Call::Named { .. }) => Some("CALL name"),
        InstructionKind::Message { .. } => Some("message send"),
        InstructionKind::Leave { .. } => Some("LEAVE"),
        InstructionKind::Iterate { .. } => Some("ITERATE"),
        InstructionKind::Nop => Some("NOP"),
        InstructionKind::Then => Some("THEN"),
        InstructionKind::Label { .. } => Some("label"),
        InstructionKind::Signal(_) => Some("SIGNAL"),
        InstructionKind::Parse(_) => Some("PARSE"),
        InstructionKind::Arg(_) => Some("ARG"),
        InstructionKind::Pull(_) => Some("PULL"),
        _ => None,
    }
}

/// The op the expression [`check_body`] hands this must end in.
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
const DEEPEST_ADDRESSED_CALL: usize = 31;

/// Whether every part of `expr` has a native op, which is what licenses the
/// whole tree compiling without `eval.rs` being entered. `depth` is how many
/// operators stand between `expr` and its slot's own root.
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
    /// How many `Op::Condition` ops the sweep saw, keyed by the keyword each
    /// is tagged with. Keyed rather than counted in one number, because a
    /// corpus holding a native `IF` condition and no native `WHEN` one would
    /// leave the `WHEN` row below vacuous while the total still looked
    /// healthy.
    native_conditions: BTreeMap<&'static str, usize>,
    /// How many `DO`/`LOOP` header slots compiled to something other than one
    /// `Op::EvalExpr`.
    native_header_values: usize,
}

/// Checks one body's compiled stream against what its instructions call for,
/// and records what fired.
fn check_body(
    body: &CodeBody,
    symbols: &rexx_parse::SymbolTable,
    source: &rexx_parse::ProgramSource,
    where_: &str,
    seen: &mut Seen,
) {
    let plan = Plan::build(body, symbols, Some(source), BodyKind::Plain);
    let chunk = match super::compile(body, &plan, ChunkTrace::of(TraceMode::NORMAL)) {
        Ok(chunk) => chunk,
        // The one error `compile` has is a machine width, and no corpus
        // program is four billion instructions long. Loud rather than
        // skipped, because a refusal here would make every assertion below
        // vacuous for that body.
        Err(error) => panic!("{where_}: compile refused this body: {error:?}"),
    };
    let regions = regions(&chunk);
    let listed = listed_whens(body);

    for (index, instruction) in body.instructions.iter().enumerate() {
        let Some(construct) = promoted_as(&instruction.kind, index, &listed) else {
            continue;
        };
        let at = u32::try_from(index).expect("a corpus body is not four billion instructions");
        *seen.constructs.entry(construct).or_default() += 1;

        // **The `Op::Exec` check that stood here is gone with the op.** It
        // asserted that a construct in the minimum promotion set had not
        // fallen back; nothing can fall back now, so the question is
        // unaskable. What it was really protecting is the line below: the
        // instruction opened a region of its own.
        let region = regions.get(&at).unwrap_or_else(|| {
            panic!(
                "{where_}: instruction {index} ({construct}) opened no Clause region\n{}",
                render(&chunk)
            )
        });

        // Whether this clause carries a compiled condition, and for which
        // keyword. Counted because the `If` and `When` rows below say nothing
        // at all on a corpus whose every condition is outside the native set:
        // `root_of` would call for `Root::EvalExpr`, the stream would hold
        // what a declining condition compiles to, and both rows would pass
        // without the promotion ever having fired.
        for op in region.iter() {
            if let Op::Condition { keyword, .. } = op {
                *seen
                    .native_conditions
                    .entry(match keyword {
                        ConditionKeyword::If => "IF",
                        ConditionKeyword::When => "WHEN",
                    })
                    .or_default() += 1;
            }
        }

        // **A `DO`/`LOOP` header is a list of expressions rather than one**, so
        // its expectation is per slot. Each `Op::LoopHeaderValue` ends one
        // slot's group, and a group starts where the one before it ended, or at
        // the region's own start for the first. The last `Root` in a group is
        // what that slot's expression must end in: neither `Op::TraceKeyword`
        // nor `Op::LoopHeaderValue` is a `Root`, so the scan back inside a
        // group finds the expression's own op and nothing else.
        if let InstructionKind::Do(loop_) | InstructionKind::Loop(loop_) = &instruction.kind {
            let mut slot = 0u32;
            let mut group = 0;
            for (op_at, op) in region.iter().enumerate() {
                if !matches!(op, Op::LoopHeaderValue { .. }) {
                    continue;
                }
                let expected =
                    crate::run::loop_header_slot(loop_, slot).map_or(Root::EvalExpr, root_of);
                let actual = region[group..op_at].iter().filter_map(Root::of).next_back();
                assert_eq!(
                    actual,
                    Some(expected),
                    "{where_}: instruction {index} ({construct}) has a header slot {slot} \
                     calling for {expected:?} and a compiled group ending in {actual:?}\n{}",
                    render(&chunk)
                );
                *seen.roots.entry(expected).or_default() += 1;
                if expected != Root::EvalExpr {
                    seen.native_header_values += 1;
                }
                group = op_at + 1;
                slot += 1;
            }
            continue;
        }

        // The expression a promoted clause's region must end in, for the
        // constructs whose expression `compile` offers to `push_native` as a
        // whole slot. A `SELECT`'s own expression is offered to no such thing
        // and evaluates through `Op::EvalExpr` unconditionally; a `WHEN
        // CASE`'s values are compared inside `Op::WhenTest`. Neither is a slot
        // this file has anything to state about that the op's presence in the
        // stream has not already said.
        let expected = match &instruction.kind {
            InstructionKind::Assignment { value, .. } => Some(root_of(value)),
            // A `SAY`, a `RETURN`, an `EXIT`, a `PUSH` and a `QUEUE` are one
            // row: each offers its whole expression to `push_native` and each
            // bare form holds none, so `None` here is a clause whose region
            // ends in no `Root` at all.
            InstructionKind::Say { expression }
            | InstructionKind::Return { expression }
            | InstructionKind::Exit { expression }
            | InstructionKind::Push { expression }
            | InstructionKind::Queue { expression } => expression.as_ref().map(root_of),
            InstructionKind::If { condition, .. } => Some(root_of(condition)),
            InstructionKind::When { condition, .. } => match root_of(condition) {
                Root::EvalExpr => None,
                root => Some(root),
            },
            _ => continue,
        };
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
            check_body(
                body,
                &program.symbols,
                &program.source,
                &format!("{name} {what}"),
                &mut seen,
            );
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
    for construct in [
        "IF",
        "SELECT",
        "WHEN",
        "WHEN CASE",
        "assignment",
        "SAY",
        "RETURN",
        "EXIT",
        "PUSH",
        "QUEUE",
        "CALL name",
        "message send",
        "LEAVE",
        "ITERATE",
        "NOP",
        "THEN",
        "label",
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
    for keyword in ["IF", "WHEN"] {
        assert!(
            seen.native_conditions.contains_key(keyword),
            "no corpus {keyword} compiled its condition to native ops, so the {keyword} rows \
             above hold only because every condition declined"
        );
    }
    assert!(
        seen.native_header_values > 0,
        "no corpus DO/LOOP header slot compiled to native ops, so the header rows above hold \
         only because every header expression declined"
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
