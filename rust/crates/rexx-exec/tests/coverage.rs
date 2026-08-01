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

//! The 4a exit gate's criterion 1, coverage half: every `InstructionKind`,
//! `ExprKind`, `LoopKind`, `PrefixOp`, `EndStyle`, `Trace` and `Operator`
//! variant is either constructed by a program in `rust/corpus/phase-4a.txt`,
//! or carries the phase that owns it. The differential half -- the subset
//! runs with zero divergences against the oracle -- is `tests/corpus.rs`.
//!
//! # Variant identity, never `keyword()`
//!
//! `InstructionKind::keyword()` maps both `When` and `WhenCase` to `"WHEN"`
//! (`ast.rs:912`), because they are the same clause under two grammar
//! productions. A coverage test keyed on that string would let any `WHEN`
//! satisfy `WhenCase` too, and the gap analysis that produced this file's
//! witness list made exactly that mistake on its first run (see
//! `criterion-1-coverage-gap.md`). Every tag below therefore comes from the
//! `match` pattern on the variant itself, through the [`tags!`] macro, and
//! never from a keyword table.
//!
//! # The owner arm is not free-form
//!
//! A variant outside 4a's scope does not get a witness; it gets an owner
//! string instead. Left unchecked that is an escape hatch -- a variant that
//! turns out hard to implement could be relabelled someone else's rather than
//! given a witness -- so two things are enforced here rather than assumed:
//!
//! * The owner string must be one of `"4b"`, `"4c"`, `"Phase 5"` or
//!   `"Phase 7"`, spelled exactly as the split table
//!   (`docs/superpowers/specs/2026-07-30-phase-4a-executor-design.md`, "The
//!   split") spells them. [`Owner::Phase`] is the only constructor that can
//!   hold a string at all, and [`assert_owner_strings_are_split_table_phases`]
//!   checks every one actually used.
//! * The **set** of out-of-4a variants is pinned against a hardcoded literal
//!   ([`EXPECTED_OUT_OF_SCOPE`]) rather than merely "whatever the `tags!`
//!   tables currently say", so relabelling a variant shows up as a diff
//!   against a committed expectation and not as a silent pass. This is the
//!   same device `phase-4-exclusions.txt` uses for the builtin exclusion set,
//!   one level down.
//!
//! `ExprKind`'s six out-of-scope variants split five ways across four phases,
//! and five of those six assignments are not in the design spec at all --
//! the spec names only `Message` outright. The other five (`Call`,
//! `QualifiedCall`, `ClassResolver`, `List`, `VariableReference`) were a
//! judgement call made at Task 16 gate time by the team lead ("main"), on
//! request, because the spec's only other relevant sentence ("argument
//! attachment inside Call, QualifiedCall, Message, List and VariableReference
//! is exercised by 4b and 4c") names two phases jointly for five variants,
//! which is not an owner a `match` arm can return. The reasoning for each is
//! recorded in `docs/superpowers/plans/phase-4-exclusions.txt`'s "EXPRKIND
//! OWNERSHIP" section; this file's [`tags!`] invocation for `ExprKind` must
//! stay in sync with that section by hand, the same relationship
//! `tests/assertions.rs`'s `EXEMPT` list has with the exclusions file's own
//! builtin set.
//!
//! `Call` is the one genuinely split variant: 4b delivers the internal-routine
//! resolution half, 4c the builtin half, and `"4b"` is named here because that
//! is the phase after which the variant stops failing loudly for *some* call
//! target (an internal-routine call), not because 4c has no claim on it. A
//! reader who reaches the end of 4b and finds a builtin-named call still loud
//! is seeing exactly this, not a regression.
//!
//! # `Operator::Backslash` is not owed to anyone
//!
//! It cannot appear in an `ExprKind::Binary` node by construction -- `\` is
//! prefix-only, and one in a dyadic position is error 35.1, in **both**
//! implementations. That is not a gap 4b or 4c will close, so it does not get
//! a phase string: [`Owner::Unreachable`] says so explicitly, and
//! [`only_backslash_is_unreachable`] pins that it is the only variant in any
//! of the seven enums marked that way. Demanding a witness for it would
//! demand a program that cannot exist, which is the same shape as
//! `LoopKind::With` needing `SUPPLIER` -- except `With` really is owed to
//! Phase 5, and `Backslash` is owed to nobody.
//!
//! # Method
//!
//! Parse-only, like Phase 3's `variants.rs`. This criterion is about what the
//! subset's *programs* construct, not about running them -- the differential
//! half in `tests/corpus.rs` is what proves they execute correctly. The walk
//! below is `rexx-parse/tests/gate_walk`'s shared module, trimmed to what the
//! 4a subset actually contains (no directives -- `assert_program_has_no_directives`
//! guards that assumption rather than silently ignoring one) and reproduced
//! here rather than imported, because an integration test cannot reach
//! another crate's `tests/` module and this crate's own `Cargo.toml`
//! deliberately keeps `rexx-parse` as a normal, not dev, dependency for
//! reasons unrelated to this file.
//!
//! # The builtin exclusion set, owed to `phase-4-exclusions.txt` by Task 16
//!
//! Unrelated to the seven enums above, but the file's own gate item ("The
//! exclusions file") asks for a set assertion so its 15 whole and 3 partial
//! builtin exclusions cannot drift from what `BuiltinFunctions.cpp`'s table
//! actually contains, the way the enum owner sets above cannot drift from
//! the split table. `rexx-inventory` already generates
//! [`rexx_inventory::builtins::NAMES`] from that table at build time (81
//! entries, table order), so [`the_builtin_exclusion_set_matches_the_committed_file`]
//! checks the file's 18 names against it directly rather than trusting the
//! file's own count. This is the one thing in this file that is not about
//! `InstructionKind`/`ExprKind`/etc; it lives here because coverage.rs is
//! this task's only permitted file that can hold a `cargo test`.

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use rexx_parse::{
    EndTarget, Expr, ExprKind, Instruction, InstructionKind, Loop, LoopKind, Operator, PrefixOp,
    Program, Trace, parse_program,
};

/// Who is responsible for a variant this file enumerates.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum Owner {
    /// 4a's own: must be witnessed by at least one program in the subset.
    InScope,
    /// Owed to a later phase, spelled exactly as the split table spells it.
    Phase(&'static str),
    /// Structurally impossible in this position for either implementation.
    /// Nothing is owed. `Operator::Backslash` only -- see the module doc.
    Unreachable,
}

/// Expands to a tag-and-owner function whose `match` has no wildcard arm,
/// plus the list of every `(tag, owner)` pair it can produce. One invocation
/// is the source of both, so the compiled tag list and the checked owner
/// cannot drift apart the way two separate `match`es over the same enum
/// could. Phase 3's `tags!` (`rexx-parse/tests/variants.rs`) produced only
/// the tag; this is that macro widened to also carry ownership, because
/// criterion 1 here needs both from one invocation for the same reason
/// Phase 3's needed the tag alone from one.
macro_rules! tags {
    ($fn_name:ident, $list:ident, $ty:ty, { $($pat:pat => ($name:literal, $owner:expr)),+ $(,)? }) => {
        fn $fn_name(k: &$ty) -> (&'static str, Owner) {
            match k {
                $($pat => ($name, $owner)),+
            }
        }
        const $list: &[(&str, Owner)] = &[$(($name, $owner)),+];
    };
}

tags!(instruction_tag, INSTRUCTION_TAGS, InstructionKind, {
    // ---- 4a's own twenty ----
    InstructionKind::Assignment { .. } => ("Assignment", Owner::InScope),
    InstructionKind::Label { .. } => ("Label", Owner::InScope),
    InstructionKind::Command { .. } => ("Command", Owner::Phase("Phase 7")),
    InstructionKind::Do(_) => ("Do", Owner::InScope),
    InstructionKind::Loop(_) => ("Loop", Owner::InScope),
    InstructionKind::If { .. } => ("If", Owner::InScope),
    InstructionKind::Then => ("Then", Owner::InScope),
    InstructionKind::Else { .. } => ("Else", Owner::InScope),
    InstructionKind::Select { .. } => ("Select", Owner::InScope),
    InstructionKind::When { .. } => ("When", Owner::InScope),
    InstructionKind::WhenCase { .. } => ("WhenCase", Owner::InScope),
    InstructionKind::Otherwise => ("Otherwise", Owner::InScope),
    InstructionKind::Leave { .. } => ("Leave", Owner::InScope),
    InstructionKind::Iterate { .. } => ("Iterate", Owner::InScope),
    InstructionKind::End { .. } => ("End", Owner::InScope),
    InstructionKind::Drop { .. } => ("Drop", Owner::InScope),
    InstructionKind::Say { .. } => ("Say", Owner::InScope),
    InstructionKind::Exit { .. } => ("Exit", Owner::InScope),
    InstructionKind::Numeric { .. } => ("Numeric", Owner::InScope),
    InstructionKind::Trace(_) => ("Trace", Owner::InScope),
    InstructionKind::Nop => ("Nop", Owner::InScope),
    // ---- 4b's nine ----
    InstructionKind::Call(_) => ("Call", Owner::Phase("4b")),
    InstructionKind::Return { .. } => ("Return", Owner::Phase("4b")),
    InstructionKind::Procedure { .. } => ("Procedure", Owner::Phase("4b")),
    InstructionKind::Use(_) => ("Use", Owner::Phase("4b")),
    InstructionKind::Signal(_) => ("Signal", Owner::Phase("4b")),
    InstructionKind::Raise(_) => ("Raise", Owner::Phase("4b")),
    InstructionKind::Interpret { .. } => ("Interpret", Owner::Phase("4b")),
    InstructionKind::Push { .. } => ("Push", Owner::Phase("4b")),
    InstructionKind::Queue { .. } => ("Queue", Owner::Phase("4b")),
    // ---- 4c's four ----
    InstructionKind::Parse(_) => ("Parse", Owner::Phase("4c")),
    InstructionKind::Arg(_) => ("Arg", Owner::Phase("4c")),
    InstructionKind::Pull(_) => ("Pull", Owner::Phase("4c")),
    InstructionKind::Address(_) => ("Address", Owner::Phase("4c")),
    // ---- Phase 5's six ----
    InstructionKind::Expose { .. } => ("Expose", Owner::Phase("Phase 5")),
    InstructionKind::Options { .. } => ("Options", Owner::Phase("Phase 5")),
    InstructionKind::Message { .. } => ("Message", Owner::Phase("Phase 5")),
    InstructionKind::Guard(_) => ("Guard", Owner::Phase("Phase 5")),
    InstructionKind::Reply { .. } => ("Reply", Owner::Phase("Phase 5")),
    InstructionKind::Forward(_) => ("Forward", Owner::Phase("Phase 5")),
});

tags!(expr_tag, EXPR_TAGS, ExprKind, {
    // ---- 4a's own nine ----
    ExprKind::Literal(_) => ("Literal", Owner::InScope),
    ExprKind::Constant(_) => ("Constant", Owner::InScope),
    ExprKind::Variable(_) => ("Variable", Owner::InScope),
    ExprKind::Stem(_) => ("Stem", Owner::InScope),
    ExprKind::Compound(_) => ("Compound", Owner::InScope),
    ExprKind::DotVariable(_) => ("DotVariable", Owner::InScope),
    ExprKind::Prefix { .. } => ("Prefix", Owner::InScope),
    ExprKind::Binary { .. } => ("Binary", Owner::InScope),
    ExprKind::Logical(_) => ("Logical", Owner::InScope),
    // ---- the six that fail loudly; see the module doc's ownership section ----
    ExprKind::Call { .. } => ("Call", Owner::Phase("4b")),
    ExprKind::VariableReference(_) => ("VariableReference", Owner::Phase("4b")),
    ExprKind::QualifiedCall { .. } => ("QualifiedCall", Owner::Phase("Phase 5")),
    ExprKind::ClassResolver { .. } => ("ClassResolver", Owner::Phase("Phase 5")),
    ExprKind::List(_) => ("List", Owner::Phase("Phase 5")),
    ExprKind::Message { .. } => ("Message", Owner::Phase("Phase 5")),
});

tags!(loop_tag, LOOP_TAGS, LoopKind, {
    LoopKind::Simple => ("Simple", Owner::InScope),
    LoopKind::Forever => ("Forever", Owner::InScope),
    LoopKind::Count(_) => ("Count", Owner::InScope),
    LoopKind::Controlled(_) => ("Controlled", Owner::InScope),
    LoopKind::Over { .. } => ("Over", Owner::InScope),
    // `DO WITH ... OVER` sends SUPPLIER, which nothing in 4a answers.
    LoopKind::With { .. } => ("With", Owner::Phase("Phase 5")),
});

tags!(prefix_op_tag, PREFIX_OP_TAGS, PrefixOp, {
    PrefixOp::Plus => ("Plus", Owner::InScope),
    PrefixOp::Minus => ("Minus", Owner::InScope),
    PrefixOp::Not => ("Not", Owner::InScope),
});

tags!(end_style_tag, END_STYLE_TAGS, rexx_parse::EndStyle, {
    rexx_parse::EndStyle::Do => ("Do", Owner::InScope),
    rexx_parse::EndStyle::LabeledDo => ("LabeledDo", Owner::InScope),
    rexx_parse::EndStyle::Loop => ("Loop", Owner::InScope),
    rexx_parse::EndStyle::Select => ("Select", Owner::InScope),
    rexx_parse::EndStyle::Otherwise => ("Otherwise", Owner::InScope),
    rexx_parse::EndStyle::LabeledOtherwise => ("LabeledOtherwise", Owner::InScope),
});

tags!(trace_tag, TRACE_TAGS, Trace, {
    Trace::Default => ("Default", Owner::InScope),
    Trace::Setting(_) => ("Setting", Owner::InScope),
    Trace::Skip(_) => ("Skip", Owner::InScope),
    Trace::Value(_) => ("Value", Owner::InScope),
});

tags!(operator_tag, OPERATOR_TAGS, Operator, {
    Operator::Plus => ("Plus", Owner::InScope),
    Operator::Subtract => ("Subtract", Owner::InScope),
    Operator::Multiply => ("Multiply", Owner::InScope),
    Operator::Divide => ("Divide", Owner::InScope),
    Operator::IntDiv => ("IntDiv", Owner::InScope),
    Operator::Remainder => ("Remainder", Owner::InScope),
    Operator::Power => ("Power", Owner::InScope),
    Operator::Abuttal => ("Abuttal", Owner::InScope),
    Operator::Concatenate => ("Concatenate", Owner::InScope),
    Operator::Blank => ("Blank", Owner::InScope),
    Operator::Equal => ("Equal", Owner::InScope),
    Operator::BackslashEqual => ("BackslashEqual", Owner::InScope),
    Operator::GreaterThan => ("GreaterThan", Owner::InScope),
    Operator::BackslashGreaterThan => ("BackslashGreaterThan", Owner::InScope),
    Operator::LessThan => ("LessThan", Owner::InScope),
    Operator::BackslashLessThan => ("BackslashLessThan", Owner::InScope),
    Operator::GreaterThanEqual => ("GreaterThanEqual", Owner::InScope),
    Operator::LessThanEqual => ("LessThanEqual", Owner::InScope),
    Operator::StrictEqual => ("StrictEqual", Owner::InScope),
    Operator::StrictBackslashEqual => ("StrictBackslashEqual", Owner::InScope),
    Operator::StrictGreaterThan => ("StrictGreaterThan", Owner::InScope),
    Operator::StrictBackslashGreaterThan => ("StrictBackslashGreaterThan", Owner::InScope),
    Operator::StrictLessThan => ("StrictLessThan", Owner::InScope),
    Operator::StrictBackslashLessThan => ("StrictBackslashLessThan", Owner::InScope),
    Operator::StrictGreaterThanEqual => ("StrictGreaterThanEqual", Owner::InScope),
    Operator::StrictLessThanEqual => ("StrictLessThanEqual", Owner::InScope),
    Operator::LessThanGreaterThan => ("LessThanGreaterThan", Owner::InScope),
    Operator::GreaterThanLessThan => ("GreaterThanLessThan", Owner::InScope),
    Operator::And => ("And", Owner::InScope),
    Operator::Or => ("Or", Owner::InScope),
    Operator::Xor => ("Xor", Owner::InScope),
    // `\` is prefix-only; a dyadic one is error 35.1 in both implementations.
    Operator::Backslash => ("Backslash", Owner::Unreachable),
});

/// One category's seen-set against its full `(tag, owner)` list.
struct Coverage {
    category: &'static str,
    all: &'static [(&'static str, Owner)],
    seen: HashSet<&'static str>,
}

impl Coverage {
    fn new(category: &'static str, all: &'static [(&'static str, Owner)]) -> Self {
        Coverage {
            category,
            all,
            seen: HashSet::new(),
        }
    }

    /// In-scope variants with no witness -- the failure this criterion
    /// exists to catch.
    fn unwitnessed(&self) -> Vec<&'static str> {
        self.all
            .iter()
            .filter(|(name, owner)| *owner == Owner::InScope && !self.seen.contains(name))
            .map(|(name, _)| *name)
            .collect()
    }

    /// `(category, tag, phase)` for every out-of-scope variant, used to build
    /// the set this file pins against [`EXPECTED_OUT_OF_SCOPE`].
    fn out_of_scope(&self) -> Vec<(&'static str, &'static str, &'static str)> {
        self.all
            .iter()
            .filter_map(|(name, owner)| match owner {
                Owner::Phase(p) => Some((self.category, *name, *p)),
                Owner::InScope | Owner::Unreachable => None,
            })
            .collect()
    }
}

// ---------------------------------------------------------------------------
// The walk. Trimmed from `rexx-parse/tests/gate_walk/mod.rs` to what the 4a
// subset actually contains: no directives. `assert_program_has_no_directives`
// guards that assumption at every parse rather than silently under-walking a
// program that gained one.
// ---------------------------------------------------------------------------

fn assert_program_has_no_directives(path: &Path, p: &Program) {
    assert!(
        p.directives.is_empty(),
        "{} has a `::` directive, which this walker does not follow into -- \
         the 4a subset is defined to have none (see phase-4a.txt's own header); \
         either the subset gained one by mistake or this walker needs widening",
        path.display()
    );
}

/// Every direct child expression of `expr`, in source order. Exhaustive so a
/// new `ExprKind` variant is a compile error here, the same guarantee the
/// `tags!` tables above carry.
fn children_of<'a>(expr: &'a Expr, f: &mut impl FnMut(&'a Expr)) {
    match &expr.kind {
        ExprKind::Literal(_)
        | ExprKind::Constant(_)
        | ExprKind::Variable(_)
        | ExprKind::Stem(_)
        | ExprKind::Compound(_)
        | ExprKind::DotVariable(_)
        | ExprKind::ClassResolver { .. } => {}
        ExprKind::Prefix { operand, .. } => f(operand),
        ExprKind::Binary { left, right, .. } => {
            f(left);
            f(right);
        }
        ExprKind::Call { args, .. } | ExprKind::QualifiedCall { args, .. } => {
            for arg in args.iter().flatten() {
                f(arg);
            }
        }
        ExprKind::Message {
            target,
            super_class,
            args,
            ..
        } => {
            f(target);
            if let Some(super_class) = super_class {
                f(super_class);
            }
            for arg in args.iter().flatten() {
                f(arg);
            }
        }
        ExprKind::List(items) => {
            for item in items.iter().flatten() {
                f(item);
            }
        }
        ExprKind::Logical(items) => {
            for item in items {
                f(item);
            }
        }
        ExprKind::VariableReference(inner) => f(inner),
    }
}

/// Every top-level expression an instruction holds directly. Exhaustive over
/// `InstructionKind`.
fn exprs_of_instruction<'a>(kind: &'a InstructionKind, f: &mut impl FnMut(&'a Expr)) {
    let opt = |e: &'a Option<Expr>, f: &mut dyn FnMut(&'a Expr)| {
        if let Some(e) = e {
            f(e);
        }
    };
    match kind {
        InstructionKind::Assignment { target, value } => {
            f(target);
            f(value);
        }
        InstructionKind::Message { term, value } => {
            f(term);
            opt(value, f);
        }
        InstructionKind::Command { expression }
        | InstructionKind::Push { expression }
        | InstructionKind::Queue { expression }
        | InstructionKind::Say { expression }
        | InstructionKind::Return { expression }
        | InstructionKind::Exit { expression }
        | InstructionKind::Reply { expression }
        | InstructionKind::Numeric { expression, .. } => opt(expression, f),
        InstructionKind::Do(l) | InstructionKind::Loop(l) => exprs_of_loop(l, f),
        InstructionKind::If { condition, .. } | InstructionKind::When { condition, .. } => {
            f(condition)
        }
        InstructionKind::WhenCase { values, .. } => {
            for v in values {
                f(v);
            }
        }
        InstructionKind::Select { case, .. } => opt(case, f),
        InstructionKind::Label { .. }
        | InstructionKind::Then
        | InstructionKind::Else { .. }
        | InstructionKind::Otherwise
        | InstructionKind::Leave { .. }
        | InstructionKind::Iterate { .. }
        | InstructionKind::End { .. }
        | InstructionKind::Drop { .. }
        | InstructionKind::Expose { .. }
        | InstructionKind::Procedure { .. }
        | InstructionKind::Nop => {}
        InstructionKind::Parse(p) | InstructionKind::Arg(p) | InstructionKind::Pull(p) => {
            for trigger in p.template.iter().flatten() {
                if let Some(e) = &trigger.value {
                    f(e);
                }
                for target in trigger.targets.iter().flatten() {
                    f(target);
                }
            }
            if let rexx_parse::ParseSource::Value(Some(e)) = &p.source {
                f(e);
            }
        }
        InstructionKind::Call(c) => match &**c {
            rexx_parse::Call::Named { args, .. } | rexx_parse::Call::Qualified { args, .. } => {
                for arg in args.iter().flatten() {
                    f(arg);
                }
            }
            rexx_parse::Call::Dynamic { target, args } => {
                f(target);
                for arg in args.iter().flatten() {
                    f(arg);
                }
            }
            rexx_parse::Call::Trap(_) => {}
        },
        InstructionKind::Signal(s) => match &**s {
            rexx_parse::Signal::Value(e) => f(e),
            rexx_parse::Signal::Label(_) | rexx_parse::Signal::Trap(_) => {}
        },
        InstructionKind::Interpret { expression } | InstructionKind::Options { expression } => {
            f(expression)
        }
        InstructionKind::Guard(g) => opt(&g.condition, f),
        InstructionKind::Forward(fw) => {
            opt(&fw.to, f);
            opt(&fw.message, f);
            opt(&fw.class, f);
            opt(&fw.arguments, f);
            if let Some(items) = &fw.array {
                for item in items.iter().flatten() {
                    f(item);
                }
            }
        }
        InstructionKind::Raise(r) => {
            opt(&r.rc, f);
            opt(&r.description, f);
            opt(&r.additional, f);
            if let Some(items) = &r.array {
                for item in items.iter().flatten() {
                    f(item);
                }
            }
            if let Some(result) = &r.result {
                opt(&result.value, f);
            }
        }
        InstructionKind::Use(u) => match &**u {
            rexx_parse::Use::Arg { targets, .. } => {
                for t in targets.iter().flatten() {
                    f(&t.target);
                    opt(&t.default, f);
                }
            }
            rexx_parse::Use::Local { .. } => {}
        },
        InstructionKind::Address(a) => {
            opt(&a.dynamic, f);
            opt(&a.command, f);
            if let Some(io) = &a.io {
                for r in [&io.input, &io.output, &io.error] {
                    match r {
                        rexx_parse::Redirection::Stream(e) | rexx_parse::Redirection::Using(e) => {
                            f(e)
                        }
                        rexx_parse::Redirection::Default
                        | rexx_parse::Redirection::Normal
                        | rexx_parse::Redirection::Stem(_) => {}
                    }
                }
            }
        }
        InstructionKind::Trace(t) => {
            if let Trace::Value(e) = t {
                f(e);
            }
        }
    }
}

fn exprs_of_loop<'a>(l: &'a Loop, f: &mut impl FnMut(&'a Expr)) {
    match &l.kind {
        LoopKind::Simple | LoopKind::Forever => {}
        LoopKind::Count(e) => {
            if let Some(e) = e {
                f(e);
            }
        }
        LoopKind::Controlled(c) => {
            f(&c.initial);
            for e in [&c.to, &c.by, &c.for_count].into_iter().flatten() {
                f(e);
            }
        }
        LoopKind::Over {
            target, for_count, ..
        }
        | LoopKind::With {
            target, for_count, ..
        } => {
            f(target);
            if let Some(e) = for_count {
                f(e);
            }
        }
    }
    if let Some(cond) = &l.conditional {
        f(&cond.condition);
    }
}

fn each_instruction<'a>(p: &'a Program, visit: &mut impl FnMut(&'a Instruction)) {
    for i in &p.main.instructions {
        visit(i);
    }
}

fn each_expr<'a>(p: &'a Program, visit: &mut impl FnMut(&'a Expr)) {
    // An explicit stack, not recursion through `children_of`: `deep_nested_expr.rex`'s
    // 3,000-term chain is in this subset specifically to exercise this walker, and
    // Phase 3's `variants.rs` already measured that a naive recursive walk aborts on
    // it. See that file's own comment for the full argument.
    fn walk<'a>(root: &'a Expr, visit: &mut impl FnMut(&'a Expr)) {
        let mut stack = vec![root];
        while let Some(e) = stack.pop() {
            visit(e);
            let mut children: Vec<&'a Expr> = Vec::new();
            children_of(e, &mut |child| children.push(child));
            stack.extend(children.into_iter().rev());
        }
    }
    each_instruction(p, &mut |i| {
        exprs_of_instruction(&i.kind, &mut |e| walk(e, visit));
    });
}

/// One path per non-comment, non-blank line of `phase-4a.txt`.
fn read_subset(list_path: &Path) -> Vec<String> {
    let text = fs::read_to_string(list_path)
        .unwrap_or_else(|e| panic!("cannot read {}: {e}", list_path.display()));
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_string)
        .collect()
}

fn corpus_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus")
}

/// The out-of-4a variant set this file's `tags!` tables are allowed to
/// produce, as a literal rather than "whatever the tables say" -- the same
/// device `phase-4-exclusions.txt` uses for the builtin set. Any edit to an
/// owner arm above that is not also made here is a test failure, which is
/// the point: relabelling a variant is a plan amendment, not a drive-by
/// `match` edit.
const EXPECTED_OUT_OF_SCOPE: &[(&str, &str, &str)] = &[
    ("InstructionKind", "Command", "Phase 7"),
    ("InstructionKind", "Call", "4b"),
    ("InstructionKind", "Return", "4b"),
    ("InstructionKind", "Procedure", "4b"),
    ("InstructionKind", "Use", "4b"),
    ("InstructionKind", "Signal", "4b"),
    ("InstructionKind", "Raise", "4b"),
    ("InstructionKind", "Interpret", "4b"),
    ("InstructionKind", "Push", "4b"),
    ("InstructionKind", "Queue", "4b"),
    ("InstructionKind", "Parse", "4c"),
    ("InstructionKind", "Arg", "4c"),
    ("InstructionKind", "Pull", "4c"),
    ("InstructionKind", "Address", "4c"),
    ("InstructionKind", "Expose", "Phase 5"),
    ("InstructionKind", "Options", "Phase 5"),
    ("InstructionKind", "Message", "Phase 5"),
    ("InstructionKind", "Guard", "Phase 5"),
    ("InstructionKind", "Reply", "Phase 5"),
    ("InstructionKind", "Forward", "Phase 5"),
    ("ExprKind", "Call", "4b"),
    ("ExprKind", "VariableReference", "4b"),
    ("ExprKind", "QualifiedCall", "Phase 5"),
    ("ExprKind", "ClassResolver", "Phase 5"),
    ("ExprKind", "List", "Phase 5"),
    ("ExprKind", "Message", "Phase 5"),
    ("LoopKind", "With", "Phase 5"),
];

/// Every phase name the split table names, spelled exactly as it spells them.
/// `docs/superpowers/specs/2026-07-30-phase-4a-executor-design.md`, "The
/// split" table and its "assigned elsewhere" paragraph.
const SPLIT_TABLE_PHASES: &[&str] = &["4b", "4c", "Phase 5", "Phase 7"];

#[test]
fn assert_owner_strings_are_split_table_phases() {
    for (category, all) in [
        ("InstructionKind", INSTRUCTION_TAGS),
        ("ExprKind", EXPR_TAGS),
        ("LoopKind", LOOP_TAGS),
        ("PrefixOp", PREFIX_OP_TAGS),
        ("EndStyle", END_STYLE_TAGS),
        ("Trace", TRACE_TAGS),
        ("Operator", OPERATOR_TAGS),
    ] {
        for (name, owner) in all {
            if let Owner::Phase(p) = owner {
                assert!(
                    SPLIT_TABLE_PHASES.contains(p),
                    "{category}::{name} names owner {p:?}, which is not one of \
                     the split table's phases {SPLIT_TABLE_PHASES:?} -- an owner \
                     string outside that set is an unpoliced escape, exactly \
                     what this assertion exists to close off"
                );
            }
        }
    }
}

#[test]
fn only_backslash_is_unreachable() {
    for (category, all) in [
        ("InstructionKind", INSTRUCTION_TAGS),
        ("ExprKind", EXPR_TAGS),
        ("LoopKind", LOOP_TAGS),
        ("PrefixOp", PREFIX_OP_TAGS),
        ("EndStyle", END_STYLE_TAGS),
        ("Trace", TRACE_TAGS),
        ("Operator", OPERATOR_TAGS),
    ] {
        for (name, owner) in all {
            if *owner == Owner::Unreachable {
                assert_eq!(
                    (category, *name),
                    ("Operator", "Backslash"),
                    "only Operator::Backslash is structurally unreachable; a \
                     second Unreachable arm needs its own justification, not \
                     a copy of this one"
                );
            }
        }
    }
}

#[test]
fn out_of_scope_set_matches_the_committed_expectation() {
    let mut actual: Vec<(&str, &str, &str)> = Vec::new();
    for cov in [
        Coverage::new("InstructionKind", INSTRUCTION_TAGS),
        Coverage::new("ExprKind", EXPR_TAGS),
        Coverage::new("LoopKind", LOOP_TAGS),
        Coverage::new("PrefixOp", PREFIX_OP_TAGS),
        Coverage::new("EndStyle", END_STYLE_TAGS),
        Coverage::new("Trace", TRACE_TAGS),
        Coverage::new("Operator", OPERATOR_TAGS),
    ] {
        actual.extend(cov.out_of_scope());
    }
    actual.sort();
    let mut expected = EXPECTED_OUT_OF_SCOPE.to_vec();
    expected.sort();
    assert_eq!(
        actual, expected,
        "the set of out-of-4a variants drifted from EXPECTED_OUT_OF_SCOPE -- \
         relabelling a variant's owner (or adding/removing one) is a plan \
         amendment and must be made in both places, the same rule \
         phase-4-exclusions.txt applies to the builtin set"
    );
}

#[test]
fn variant_counts_match_the_audited_split() {
    // Re-derived here rather than trusted: 40 InstructionKind variants (20 in
    // 4a, 9 in 4b, 4 in 4c, 6 in Phase 5, 1 in Phase 7) and 15 ExprKind (9 in
    // scope, 6 failing loudly), per the design spec's criterion 1.
    assert_eq!(INSTRUCTION_TAGS.len(), 40);
    assert_eq!(
        INSTRUCTION_TAGS
            .iter()
            .filter(|(_, o)| *o == Owner::InScope)
            .count(),
        20
    );
    assert_eq!(
        INSTRUCTION_TAGS
            .iter()
            .filter(|(_, o)| *o == Owner::Phase("4b"))
            .count(),
        9
    );
    assert_eq!(
        INSTRUCTION_TAGS
            .iter()
            .filter(|(_, o)| *o == Owner::Phase("4c"))
            .count(),
        4
    );
    assert_eq!(
        INSTRUCTION_TAGS
            .iter()
            .filter(|(_, o)| *o == Owner::Phase("Phase 5"))
            .count(),
        6
    );
    assert_eq!(
        INSTRUCTION_TAGS
            .iter()
            .filter(|(_, o)| *o == Owner::Phase("Phase 7"))
            .count(),
        1
    );

    assert_eq!(EXPR_TAGS.len(), 15);
    assert_eq!(
        EXPR_TAGS
            .iter()
            .filter(|(_, o)| *o == Owner::InScope)
            .count(),
        9
    );
    assert_eq!(
        EXPR_TAGS
            .iter()
            .filter(|(_, o)| matches!(o, Owner::Phase(_)))
            .count(),
        6
    );

    assert_eq!(LOOP_TAGS.len(), 6);
    assert_eq!(PREFIX_OP_TAGS.len(), 3);
    assert_eq!(END_STYLE_TAGS.len(), 6);
    assert_eq!(TRACE_TAGS.len(), 4);
    assert_eq!(OPERATOR_TAGS.len(), 32);
}

/// `phase-4a.txt`'s exact line list, one entry per non-comment, non-blank
/// line, in file order. A branch review (`branch-review-harness.md`, H2)
/// found that nothing pinned this: `corpus.rs`, `collect_stress.rs` and
/// this file's own coverage test all report "N of N" or "witnessed by the
/// subset" against whatever the file happens to contain, so deleting a
/// line shrinks every measurement silently and `cargo test -p rexx-exec`
/// stays fully green. Measured in that review: deleting exactly the three
/// `mutation_*` entries below leaves the whole suite green while
/// `mutate-4a.sh` falls from 9 of 9 caught to 5 of 9, because three of the
/// nine mutations have no other witness. `phase_4a_subset_matches_the_
/// committed_list` closes it the same way `EXPECTED_OUT_OF_SCOPE` above
/// and `tests/assertions.rs`'s `EXEMPT` already do: a literal, checked by
/// equality rather than by length, so removing *or* adding a line is a
/// test failure here, and the file's own set assertion in
/// `docs/superpowers/plans/phase-4-exclusions.txt`'s spirit -- adding a
/// witness is not free, but making one silently stop counting must not
/// be either.
const EXPECTED_SUBSET: &[&str] = &[
    "lang/arith_digits.rex",
    "lang/no_trailing_newline.rex",
    "lang/select_when.rex",
    "lang/stem_compound.rex",
    "lang/trace_output.rex",
    "num/comparison.rex",
    "num/notation_thresholds.rex",
    "lang/do_loop_forms.rex",
    "lang/do_label.rex",
    "lang/leave_nested_outer.rex",
    "lang/iterate_from_select.rex",
    "lang/if_else_chain.rex",
    "lang/select_when_bodies.rex",
    "lang/select_when_absorption.rex",
    "lang/leave_iterate_variants.rex",
    "lang/drop_stem_tail.rex",
    "lang/stem_aliasing.rex",
    "lang/exit_with_value.rex",
    "lang/exit_no_value.rex",
    "lang/number_identity.rex",
    "lang/comparison_families.rex",
    "lang/deep_nested_expr.rex",
    "lang/trace_results.rex",
    "lang/prefix_dotvar_logical_over_label.rex",
    "lang/comparison_operators_remaining.rex",
    "lang/trace_numeric_request.rex",
    "lang/mutation_digits_at_render.rex",
    "lang/mutation_form_at_render.rex",
    "lang/mutation_controlled_order.rex",
];

#[test]
fn phase_4a_subset_matches_the_committed_list() {
    let corpus_dir = corpus_dir();
    let subset = read_subset(&corpus_dir.join("phase-4a.txt"));
    assert_eq!(
        subset, EXPECTED_SUBSET,
        "phase-4a.txt's entries drifted from EXPECTED_SUBSET -- adding or \
         removing a line from the L0 subset is a plan amendment, and must \
         change both the file and this list together, so a line cannot be \
         silently dropped (shrinking every measurement that reads the file) \
         or silently added (widening the subset with no witness review)"
    );
}

#[test]
fn every_in_scope_variant_is_witnessed_by_the_phase_4a_subset() {
    let mut instructions = Coverage::new("InstructionKind", INSTRUCTION_TAGS);
    let mut exprs = Coverage::new("ExprKind", EXPR_TAGS);
    let mut loops = Coverage::new("LoopKind", LOOP_TAGS);
    let mut prefix_ops = Coverage::new("PrefixOp", PREFIX_OP_TAGS);
    let mut end_styles = Coverage::new("EndStyle", END_STYLE_TAGS);
    let mut traces = Coverage::new("Trace", TRACE_TAGS);
    let mut operators = Coverage::new("Operator", OPERATOR_TAGS);

    let corpus_dir = corpus_dir();
    let subset = read_subset(&corpus_dir.join("phase-4a.txt"));
    assert!(
        !subset.is_empty(),
        "phase-4a.txt named no programs -- that is a corpus defect, not an \
         empty pass"
    );

    for rel_path in &subset {
        let abs = corpus_dir.join(rel_path);
        let text = fs::read(&abs).unwrap_or_else(|e| panic!("cannot read {}: {e}", abs.display()));
        let p = parse_program(text)
            .unwrap_or_else(|e| panic!("{} failed to parse: {e:?}", abs.display()));
        assert_program_has_no_directives(&abs, &p);

        each_instruction(&p, &mut |i| {
            instructions.seen.insert(instruction_tag(&i.kind).0);
            match &i.kind {
                InstructionKind::Do(l) | InstructionKind::Loop(l) => {
                    loops.seen.insert(loop_tag(&l.kind).0);
                }
                InstructionKind::End {
                    closes: Some(EndTarget { style, .. }),
                    ..
                } => {
                    end_styles.seen.insert(end_style_tag(style).0);
                }
                InstructionKind::Trace(t) => {
                    traces.seen.insert(trace_tag(t).0);
                }
                _ => {}
            }
        });
        each_expr(&p, &mut |e| {
            exprs.seen.insert(expr_tag(&e.kind).0);
            match &e.kind {
                ExprKind::Prefix { op, .. } => {
                    prefix_ops.seen.insert(prefix_op_tag(op).0);
                }
                ExprKind::Binary { op, .. } => {
                    operators.seen.insert(operator_tag(op).0);
                }
                _ => {}
            }
        });
    }

    let mut report = String::new();
    for cov in [
        &instructions,
        &exprs,
        &loops,
        &prefix_ops,
        &end_styles,
        &traces,
        &operators,
    ] {
        let missing = cov.unwitnessed();
        if !missing.is_empty() {
            use std::fmt::Write as _;
            writeln!(
                report,
                "{}: {} in-scope variant(s) unwitnessed by phase-4a.txt: {}",
                cov.category,
                missing.len(),
                missing.join(", ")
            )
            .unwrap();
        }
    }
    assert!(
        report.is_empty(),
        "criterion 1's coverage property fails:\n{report}"
    );
}

/// `phase-4-exclusions.txt`'s 15 whole exclusions plus the 3 partial rows
/// (`VALUE`, `ADDRESS`, `QUEUED`), copied here as a literal rather than
/// parsed from the prose file -- the same choice `tests/assertions.rs`'s
/// `EXEMPT` list makes against that file's builtin set. Changing a name here
/// without changing the file (or vice versa) is exactly the drift this test
/// exists to catch.
const EXCLUDED_BUILTINS: &[&str] = &[
    // Phase 7, streams and platform.
    "CHARIN",
    "CHAROUT",
    "CHARS",
    "LINEIN",
    "LINEOUT",
    "LINES",
    "STREAM",
    "QUALIFY",
    "USERID",
    "SETLOCAL",
    "ENDLOCAL",
    // Phase 10, RXAPI.
    "RXQUEUE",
    "RXFUNCADD",
    "RXFUNCDROP",
    "RXFUNCQUERY",
    // Partial: in scope in one form, excluded in another.
    "VALUE",
    "ADDRESS",
    "QUEUED",
];

#[test]
fn the_builtin_exclusion_set_matches_the_committed_file() {
    let names = rexx_inventory::builtins::NAMES;
    assert_eq!(
        names.len(),
        81,
        "BuiltinFunctions.cpp's table moved off 81 entries -- phase-4-exclusions.txt's \
         \"66 of the 81\" line and this test's own derivation below both need revisiting"
    );

    let mut seen = HashSet::new();
    for excluded in EXCLUDED_BUILTINS {
        assert!(
            names.contains(excluded),
            "{excluded} is listed in phase-4-exclusions.txt but is not in \
             BuiltinFunctions.cpp's builtin table at all"
        );
        assert!(
            seen.insert(*excluded),
            "{excluded} is listed twice in EXCLUDED_BUILTINS"
        );
    }
    assert_eq!(
        EXCLUDED_BUILTINS.len(),
        18,
        "15 whole exclusions plus 3 partial rows"
    );

    // Derived, not asserted: this is the number phase-4-exclusions.txt's own
    // header reads out ("66 of the 81 builtins ... are in scope, three of
    // them partially"), and this line is how that phrasing stays synced to
    // the actual table rather than to a copy-pasted figure.
    let in_scope = names.len() - (EXCLUDED_BUILTINS.len() - 3);
    assert_eq!(
        in_scope, 66,
        "66 of the 81 builtins are in scope, three of them partially -- see \
         phase-4-exclusions.txt"
    );
}
