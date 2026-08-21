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
//! # The owner table lives in `owners.rs`
//!
//! `Owner`, the `tags!` macro, the seven `*_TAGS` tables and their tag
//! functions, `Coverage`, `EXPECTED_OUT_OF_SCOPE` and `SPLIT_TABLE_PHASES`
//! all live in `owners.rs` now, `#[path]`-included below as `mod owners`,
//! rather than being defined here by hand. `loud.rs` includes the identical
//! file the same way. See `owners.rs`'s own module doc for why (item I36)
//! and for what still has to be kept in sync by hand regardless (Step 5's
//! five pinned items).
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
//! A variant this crate does not implement does not get a witness; it gets an owner
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
//! * The **set** of out-of-scope variants is pinned against a hardcoded literal
//!   ([`EXPECTED_OUT_OF_SCOPE`]) rather than merely "whatever the `tags!`
//!   tables currently say", so relabelling a variant shows up as a diff
//!   against a committed expectation and not as a silent pass. This is the
//!   same device `phase-4-exclusions.txt` uses for the builtin exclusion set,
//!   one level down.
//!
//! `ExprKind` had six out-of-scope variants at Task 16 gate time, five of
//! which were not in the design spec at all -- the spec names only `Message`
//! outright. The other five (`Call`, `QualifiedCall`, `ClassResolver`,
//! `List`, `VariableReference`) were a judgement call made at Task 16 gate
//! time by the team lead ("main"), on request, because the spec's only other
//! relevant sentence ("argument attachment inside Call, QualifiedCall,
//! Message, List and VariableReference is exercised by 4b and 4c") names two
//! phases jointly for five variants, which is not an owner a `match` arm can
//! return. The reasoning for each is recorded in `docs/superpowers/plans/
//! phase-4-exclusions.txt`'s "EXPRKIND OWNERSHIP" section; this file's
//! [`tags!`] invocation for `ExprKind` must stay in sync with that section by
//! hand, the same relationship `tests/assertions.rs`'s `EXEMPT` list has with
//! the exclusions file's own builtin set.
//!
//! **`ExprKind::Call` is in scope, and is not one of the four below.**
//! Unlike `InstructionKind::Call`, whose arms `owners.rs` gives a row each
//! because `Call::Qualified` alone is still loud, `ExprKind::Call`'s own
//! `CallTarget` has exactly two forms and this crate evaluates both -- there
//! is no later-phase arm hiding inside it, so the variant closes outright
//! rather than staying split. That is not a claim that every call target
//! runs: a builtin-named call still fails loudly, through
//! `Loud::unresolved_call` naming `4c`, which is a claim on the resolution
//! steps rather than on the variant -- see `eval_call`'s own doc
//! (`eval.rs`) for the order.
//!
//! Which variants remain, and who owns each, is `owners.rs`'s `EXPR_TAGS` and
//! [`EXPECTED_OUT_OF_SCOPE`]; both are asserted here rather than described.
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
//! subset actually contains (`::ROUTINE`, `::CLASS`, `::METHOD`, `::ATTRIBUTE`
//! and `::CONSTANT`, and no other directive -- `assert_program_has_only_admitted_directives`
//! guards that assumption rather than silently ignoring one, and
//! `each_instruction` walks the body of each kind that has one) and reproduced
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
    DirectiveKind, EndTarget, Expr, ExprKind, Instruction, InstructionKind, Loop, LoopKind,
    Program, Trace, parse_program,
};

#[path = "owners.rs"]
mod owners;
use owners::{
    Coverage, END_STYLE_TAGS, EXPR_TAGS, INSTRUCTION_TAGS, LOOP_TAGS, OPERATOR_TAGS,
    PREFIX_OP_TAGS, TRACE_TAGS, end_style_tag, expr_tag, instruction_tag, loop_tag, operator_tag,
    prefix_op_tag, trace_tag,
};

// ---------------------------------------------------------------------------
// The walk. Trimmed from `rexx-parse/tests/gate_walk/mod.rs` to what the
// subset actually contains: the bodies of `::ROUTINE`, `::METHOD` and
// `::ATTRIBUTE`, plus `::CLASS` and `::CONSTANT` admitted as directives that
// carry no body. `assert_program_has_only_admitted_directives` guards that
// assumption at every parse rather than silently under-walking a program
// that gained a directive kind this walker does not know about.
// ---------------------------------------------------------------------------

/// Whether `kind` is one of the directive kinds this walker admits without
/// panicking: `::ROUTINE`, `::METHOD` and `::ATTRIBUTE`, each walked into by
/// `each_instruction` below, plus `::CLASS` and `::CONSTANT`, which own no
/// body for it to walk.
///
/// **Exhaustive over `DirectiveKind`'s own nine variants**
/// (`rexx-parse/src/ast.rs:1353`-`1368`), not a string comparison against a
/// hand-typed table. A tenth variant added to that enum is a compile error
/// here until this match says whether it is admitted, and there is no string
/// for a typo to hide in: the previous version of this function compared
/// `d.kind.keyword()` against a `&[&str]` literal, which nothing checked was
/// even a real directive keyword, let alone in sync with the enum it meant
/// to track.
fn is_admitted_directive_kind(kind: &DirectiveKind) -> bool {
    match kind {
        DirectiveKind::Routine(_)
        | DirectiveKind::Class(_)
        | DirectiveKind::Method(_)
        | DirectiveKind::Attribute(_)
        | DirectiveKind::Constant(_) => true,
        DirectiveKind::Annotate(_)
        | DirectiveKind::Options(_)
        | DirectiveKind::Requires(_)
        | DirectiveKind::Resource(_) => false,
    }
}

fn assert_program_has_only_admitted_directives(path: &Path, p: &Program) {
    let others: Vec<&str> = p
        .directives
        .iter()
        .filter(|d| !is_admitted_directive_kind(&d.kind))
        .map(|d| d.kind.keyword())
        .collect();
    assert!(
        others.is_empty(),
        "{} has a `::` directive this walker does not admit ({others:?}) -- see \
         `is_admitted_directive_kind` for the set and `each_instruction` for \
         which of them carry a body it descends into; either the subset gained \
         a directive kind by mistake or this walker needs widening",
        path.display()
    );
}

/// Every one of `DirectiveKind`'s nine keywords, checked against
/// [`is_admitted_directive_kind`]'s real answer for a real parsed instance of
/// that kind -- not only `::ATTRIBUTE`, which `an_unadmitted_directive_still_panics`
/// below covers alone.
///
/// **What this catches that the single-keyword negative control does not.**
/// Measured: mutating `is_admitted_directive_kind` to also admit
/// `DirectiveKind::Constant(_)` (moving it into the `true` arm) left every
/// other test in this file green, `an_unadmitted_directive_still_panics`
/// included, because that test only ever constructs `::ATTRIBUTE`. This test
/// instead parses one minimal instance of every directive kind -- the same
/// nine literals `rexx-parse/src/directive/tests.rs`'s own
/// `every_directive_keyword_reaches_its_node` uses, reproduced here rather
/// than imported for the reason this file's own module doc gives (an
/// integration test cannot reach another crate's `tests/` module) -- and
/// checks each one against a committed true/false expectation, so moving
/// *any* one of the six refused variants into the admitted set (or vice
/// versa) reddens here specifically, with the wrong keyword named in the
/// failure.
#[test]
fn every_directive_keyword_is_correctly_admitted_or_refused() {
    let cases: &[(&str, &str, bool)] = &[
        ("::annotate package\n", "ANNOTATE", false),
        ("::attribute a\n", "ATTRIBUTE", true),
        ("::class c\n", "CLASS", true),
        ("::constant k 1\n", "CONSTANT", true),
        ("::method m\n  return 1\n", "METHOD", true),
        ("::options noprolog\n", "OPTIONS", false),
        ("::requires \"nosuch\"\n", "REQUIRES", false),
        ("::resource d\nbody\n::END\n", "RESOURCE", false),
        ("::routine r\n  return 1\n", "ROUTINE", true),
    ];
    for (text, expected_keyword, expected_admitted) in cases {
        let p = parse_program(text.as_bytes().to_vec())
            .unwrap_or_else(|e| panic!("{text:?} failed to parse: {e:?}"));
        assert_eq!(
            p.directives.len(),
            1,
            "{text:?} did not produce exactly one directive"
        );
        let kind = &p.directives[0].kind;
        assert_eq!(
            kind.keyword(),
            *expected_keyword,
            "{text:?} parsed to the wrong directive kind"
        );
        assert_eq!(
            is_admitted_directive_kind(kind),
            *expected_admitted,
            "{expected_keyword} admission disagrees with the committed expectation"
        );
    }
}

/// Phase 5a Task 1's own demonstration: `::CLASS K` is an inline literal
/// here, not a `phase-5a.txt` corpus entry, because `::CLASS` is not
/// implemented and committing it would redden the corpus differential (see
/// `docs/superpowers/plans/2026-08-15-phase-5a-native-layer.md`'s Task 1
/// brief).
///
/// **Measured before this task widened the walker:** run against the
/// then-named `assert_program_has_only_routine_directives`, this exact
/// program panicked with `has a \`::\` directive this walker does not follow
/// into (["CLASS"])`. Task 1's report carries the full transcript.
#[test]
fn a_class_directive_is_admitted_without_panicking() {
    let p = parse_program(b"::CLASS K\n".to_vec()).expect("::CLASS K parses");
    assert_program_has_only_admitted_directives(Path::new("<phase-5a-task-1-demo>"), &p);
}

/// **[`each_instruction`] descends into a `::METHOD` and a `::ATTRIBUTE`
/// body**, which is what makes a construct written only inside one count
/// toward criterion 1.
///
/// Paired with a `::ROUTINE` in the same program, so the assertion cannot be
/// satisfied by a walk that visits every directive's body indiscriminately
/// and one that visits none reads differently from one that visits only the
/// routine: the three keywords are distinct, and all three must arrive.
#[test]
fn the_walker_descends_into_a_method_body_and_an_attribute_body() {
    let p = parse_program(
        b"nop\n\
          ::routine r\n  iterate\n\
          ::class K\n\
          ::method m class\n  leave\n\
          ::attribute a get\n  return 1\n"
            .to_vec(),
    )
    .expect("the program parses");
    let mut seen: Vec<&str> = Vec::new();
    each_instruction(&p, &mut |i| seen.push(instruction_tag(&i.kind).0));
    assert_eq!(
        seen,
        vec!["Nop", "Iterate", "Leave", "Return"],
        "the walk did not visit exactly the main body's instruction and one \
         from each directive body, in directive order"
    );
}

/// Pairs with the success above: a directive kind [`is_admitted_directive_kind`]
/// still marks `false` must still panic, or the widening silently removed the
/// guard rather than widening it.
#[test]
#[should_panic(expected = "has a `::` directive this walker does not admit")]
fn an_unadmitted_directive_still_panics() {
    let p = parse_program(b"::OPTIONS NOPROLOG\n".to_vec()).expect("::OPTIONS NOPROLOG parses");
    assert_program_has_only_admitted_directives(Path::new("<phase-5a-task-1-demo>"), &p);
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

/// Every instruction of the main body **and of every `::ROUTINE` body**.
///
/// Descending into a routine is what lets
/// `assert_program_has_only_admitted_directives` admit `::ROUTINE` without
/// also admitting a hole: the guard exists to stop a subset program hiding
/// constructs from criterion 1 inside a body nothing walks.
///
/// **Every directive kind that owns a `CodeBody` is descended into**, which
/// is the same set `rexx_exec`'s own `body_of` turns into a `&CodeBody`:
/// `::ROUTINE`, `::METHOD` and `::ATTRIBUTE`. `::CLASS` owns no body of its
/// own, and a `::CONSTANT`'s parenthesised value is an expression rather than
/// an instruction, so neither can carry an `Instruction` for this walk to
/// miss; both are admitted by the guard above and visited here as nothing.
///
/// Descending into a method body **cannot hide anything**, in either
/// direction: the only consumer, `every_in_scope_variant_is_witnessed_by_the_
/// phase_subsets`, reports `Coverage::unwitnessed`, so a wider walk can only
/// move a variant from unwitnessed to witnessed. What it changes is that a
/// construct written only inside a `::METHOD` body now counts as a witness,
/// where before the same program witnessed nothing at all.
fn each_instruction<'a>(p: &'a Program, visit: &mut impl FnMut(&'a Instruction)) {
    for i in &p.main.instructions {
        visit(i);
    }
    for directive in &p.directives {
        let body = match &directive.kind {
            DirectiveKind::Routine(routine) => routine.body.as_ref(),
            DirectiveKind::Method(method) => method.body.as_ref(),
            DirectiveKind::Attribute(attribute) => attribute.body.as_ref(),
            _ => None,
        };
        if let Some(body) = body {
            for i in &body.instructions {
                visit(i);
            }
        }
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

/// The union of every non-comment, non-blank line across `list_paths`, in
/// first-seen order, each entry appearing once even if two files name the
/// same corpus program.
///
/// A slice rather than one `&Path`, so a later phase's own subset file can run
/// *alongside* `phase-4a.txt` rather than replacing it -- every earlier-phase
/// witness stays exercised as later phases add their own subset files, instead
/// of each phase's own harness run choosing between the earlier programs and
/// its own and losing the other's coverage.
fn read_subset(list_paths: &[&Path]) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut union = Vec::new();
    for list_path in list_paths {
        let text = fs::read_to_string(list_path)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", list_path.display()));
        for line in text.lines().map(str::trim) {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if seen.insert(line.to_string()) {
                union.push(line.to_string());
            }
        }
    }
    union
}

/// Review finding I5: `read_subset`'s multi-file union and de-duplication
/// -- the only new runtime logic Task 0 shipped -- had no test. Two files
/// with an overlapping entry are the direct exercise of the union path:
/// first-seen order across files, and a name repeated in the second file
/// appearing only once, at its *first* position. Still worth keeping now
/// that the real call sites pass more than one file, and the reason is
/// measured: **no two phase subset files share an entry** -- checked across
/// all three by sorting their entries and looking for a repeat, none found --
/// so a run over the committed files never takes the `seen.insert` false
/// branch at all, whatever it passes. **Nothing enforces that disjointness**,
/// and this comment does not claim otherwise; it is a fact about the files as
/// they stand, which is exactly why the de-duplication half needs
/// hand-written overlapping inputs to reach at all. No union total is written
/// down here either, because a total goes stale the moment a phase adds a
/// program -- and one did.
#[test]
fn read_subset_unions_two_files_first_seen_order_deduplicated() {
    let dir = std::env::temp_dir();
    let a = dir.join("rexx-exec-read-subset-test-a.txt");
    let b = dir.join("rexx-exec-read-subset-test-b.txt");
    fs::write(&a, "# a comment\none.rex\ntwo.rex\n").expect("writing the first list file");
    fs::write(&b, "two.rex\nthree.rex\n").expect("writing the second list file");

    let union = read_subset(&[&a, &b]);
    assert_eq!(
        union,
        vec![
            "one.rex".to_string(),
            "two.rex".to_string(),
            "three.rex".to_string()
        ],
        "the union must be first-seen order across files (one.rex before \
         two.rex before three.rex) with two.rex's second, repeated \
         occurrence (in b) dropped rather than duplicated"
    );

    let _ = fs::remove_file(&a);
    let _ = fs::remove_file(&b);
}

fn corpus_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus")
}

/// The subset files the union-coverage run reads, in union order.
///
/// A named constant rather than a literal at the call site so that
/// [`the_union_reads_every_phase_subset_file`] can assert it against the
/// corpus directory itself.
///
/// **Dropping a file from this list is caught today, but only incidentally.**
/// It fails `every_in_scope_variant_is_witnessed_by_the_phase_subsets` --
/// measured, removing `phase-4c.txt` reports `4 in-scope variant(s)
/// unwitnessed: Parse, Arg, Pull, Address::Environment` -- and that depends on
/// the dropped phase still owning a variant no earlier phase witnesses. It is
/// a property of the corpus as it stands, not an invariant: the moment those
/// variants gain a witness elsewhere, the file can be dropped silently.
const SUBSET_FILES: &[&str] = &[
    "phase-4a.txt",
    "phase-4b.txt",
    "phase-4c.txt",
    "phase-5a.txt",
];

/// The phase subset files that exist in the corpus directory, sorted.
///
/// Read from the directory rather than listed a second time, so the assertion
/// below cannot be satisfied by a copy of [`SUBSET_FILES`] edited in the same
/// change, and so a subset file added later and never wired in here is red
/// rather than silently unread.
///
/// Duplicated from `corpus.rs` and `collect_stress.rs` rather than shared, for
/// the reason `read_subset` above is duplicated: these are three
/// integration-test binaries and none can `mod` another.
fn phase_subset_files_on_disk() -> Vec<String> {
    let dir = corpus_dir();
    let entries =
        fs::read_dir(&dir).unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()));
    let mut names: Vec<String> = entries
        .flatten()
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| name.starts_with("phase-") && name.ends_with(".txt"))
        .collect();
    names.sort();
    names
}

/// The coverage union reads **every** phase subset file the corpus has.
///
/// The pin on *which files* the union site reads. The three
/// `phase_*_subset_matches_the_committed_list` tests below pin each file's
/// contents and say nothing about whether anything reads it.
#[test]
fn the_union_reads_every_phase_subset_file() {
    assert_eq!(
        SUBSET_FILES,
        phase_subset_files_on_disk(),
        "the variant-coverage union does not read every phase subset file in \
         rust/corpus/. A file missing from SUBSET_FILES is a phase whose \
         programs contribute no variants, and the coverage property then holds \
         over a smaller union"
    );
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
    // The plan amendment this assertion exists to make visible. The program is
    // 25 plain `DO` blocks around a failing clause, and pins the error
    // report's 40-column indent saturation, which no other program in this
    // list nests deeply enough to reach.
    "lang/deep_nesting_indent_cap.rex",
    // A controlled loop's per-step rounding, and specifically the value the
    // *next* step adds to. No other program in this list advances a control
    // variable past its own DIGITS, so none of them can tell the rounded sum
    // from the exact one.
    "lang/loop_control_rounding.rex",
];

#[test]
fn phase_4a_subset_matches_the_committed_list() {
    let corpus_dir = corpus_dir();
    let subset = read_subset(&[&corpus_dir.join("phase-4a.txt")]);
    assert_eq!(
        subset, EXPECTED_SUBSET,
        "phase-4a.txt's entries drifted from EXPECTED_SUBSET -- adding or \
         removing a line from the L0 subset is a plan amendment, and must \
         change both the file and this list together, so a line cannot be \
         silently dropped (shrinking every measurement that reads the file) \
         or silently added (widening the subset with no witness review)"
    );
}

/// `phase-4b.txt`'s exact line list, the same device [`EXPECTED_SUBSET`] is
/// for `phase-4a.txt`, and added for the same reason.
///
/// **What its absence costs was measured, and it was not
/// theoretical.** Removing **one entry at a time** from `phase-4b.txt` and
/// running this file: for **nine of the twelve, that one deletion leaves it
/// green**, because [`every_in_scope_variant_is_witnessed_by_the_phase_subsets`]
/// only needs *some* program to construct each variant, and by the end of 4b
/// most variants have several witnesses. Only `call_expression`,
/// `use_arg_forms` and `push_queue` construct something nothing else in the
/// union does.
///
/// **Distributively, not collectively**: removing all nine at once *does* fail
/// this file (`3 in-scope variant(s) unwitnessed: Interpret, Signal, Raise`).
/// The single silent deletion is what this pin is for.
///
/// The worst case is `lang/condition_traps.rex`, the 4b gate's criterion 8
/// witness and the declared corpus catcher for **one** of `mutate-4b.sh`'s
/// mutations (row 6, `SIGL` off by one -- each corpus-catching mutation
/// diverges on exactly one program, so none can claim more): with its line
/// removed the corpus reported `41 of 41 matching` at exit 0, this file
/// passed, and `collect_stress` passed -- three criteria reporting MET with
/// one criterion's entire subject deleted, and the headline count shrinking
/// silently. A "N of N matching" harness cannot notice a missing program;
/// only a committed list can.
const EXPECTED_SUBSET_4B: &[&str] = &[
    "lang/interpret_dynamic.rex",
    "lang/interpret_error_echo.rex",
    "lang/call_return.rex",
    "lang/call_expression.rex",
    "lang/call_procedure_expose.rex",
    "lang/use_arg_forms.rex",
    "lang/signal_forms.rex",
    "lang/condition_traps.rex",
    "lang/push_queue.rex",
    "lang/raise_array_substitution.rex",
    "lang/loop_retest_blame.rex",
    "lang/call_on_trap_rearms.rex",
];

/// `phase-4c.txt`'s exact line list, the same device [`EXPECTED_SUBSET`] and
/// [`EXPECTED_SUBSET_4B`] are for the earlier phases, and added for the same
/// reason: a "N of N" harness cannot notice a missing program, and every check
/// that reads the file shrinks silently when a line goes.
const EXPECTED_SUBSET_4C: &[&str] = &[
    "lang/parse_triggers.rex",
    "lang/parse_sources.rex",
    "lang/parse_template.rex",
    "lang/pull_queue.rex",
    "lang/address_env.rex",
    "lang/state_builtins.rex",
    "lang/builtin_argument_range.rex",
    "lang/routine_dispatch.rex",
    "lang/pos_window.rex",
    "lang/procedure_entry_rule.rex",
    "lang/do_clause_boundaries.rex",
    "lang/condition_queue_drain.rex",
];

#[test]
fn phase_4c_subset_matches_the_committed_list() {
    let corpus_dir = corpus_dir();
    let subset = read_subset(&[&corpus_dir.join("phase-4c.txt")]);
    assert_eq!(
        subset, EXPECTED_SUBSET_4C,
        "phase-4c.txt's entries drifted from EXPECTED_SUBSET_4C -- adding or \
         removing a line from the 4c subset is a plan amendment, and must \
         change both the file and this list together"
    );
}

#[test]
fn phase_4b_subset_matches_the_committed_list() {
    let corpus_dir = corpus_dir();
    let subset = read_subset(&[&corpus_dir.join("phase-4b.txt")]);
    assert_eq!(
        subset, EXPECTED_SUBSET_4B,
        "phase-4b.txt's entries drifted from EXPECTED_SUBSET_4B -- adding or \
         removing a line from the 4b subset is a plan amendment, and must \
         change both the file and this list together. Nine of the twelve are \
         witnessed by no other check at all: see EXPECTED_SUBSET_4B's own doc \
         comment for the measurement, and for the case where deleting one \
         program left three gate criteria still reporting MET"
    );
}

/// `phase-5a.txt`'s exact line list, the same device [`EXPECTED_SUBSET`],
/// [`EXPECTED_SUBSET_4B`] and [`EXPECTED_SUBSET_4C`] are for the earlier
/// phases, and added for the same reason: this file was itself the
/// still-empty phase subset those three files' own review finding (H2) was
/// measured against, and Task 4 is the first to give it content -- landing
/// six entries with no pin at all would repeat exactly the gap H2 found,
/// on the first commit that could show it.
const EXPECTED_SUBSET_5A: &[&str] = &[
    "lang/directive_class_installs.rex",
    "lang/directive_method_installs.rex",
    "lang/directive_attribute_installs.rex",
    "lang/directive_constant_expression_installs.rex",
    "lang/directive_constant_expression_fails.rex",
    "lang/directive_constant_expression_needs_class.rex",
    "lang/directive_constant_expression_blames_the_last_class.rex",
    "lang/message_send_native.rex",
    "lang/message_send_inherited_scope.rex",
    "lang/message_send_unknown_method.rex",
    "lang/message_send_unknown_method_on_nil.rex",
    "lang/message_send_unknown_method_on_a_number.rex",
    "lang/message_send_scope_override.rex",
    "lang/message_send_too_many_arguments.rex",
    "lang/message_send_missing_argument.rex",
    "lang/message_send_argument_not_a_string.rex",
    "lang/message_instruction.rex",
    "lang/message_assignment_form.rex",
    "lang/environment_symbols.rex",
    "lang/environment_symbols_through_value.rex",
    "lang/environment_package_class.rex",
    "lang/environment_package_class_shadows_the_environment.rex",
    "lang/environment_special_dot_variables_are_not_resolved.rex",
    "lang/environment_methods_table_unattached.rex",
    "lang/environment_methods_table_attached.rex",
    "lang/environment_object_operands.rex",
    "lang/environment_object_in_a_loop_header.rex",
    "lang/environment_object_in_a_raise.rex",
    "lang/method_class_body.rex",
    "lang/method_returns_no_value.rex",
    "lang/method_no_result_is_an_error.rex",
    "lang/method_exit_returns_to_the_sender.rex",
    "lang/method_body_raises.rex",
    "lang/method_class_side_lookup.rex",
    "lang/method_attribute_body.rex",
    "lang/method_trace_invocation.rex",
    "lang/method_trace_nested.rex",
    "lang/method_attribute_set_body.rex",
    "lang/method_parse_source.rex",
    "lang/message_send_argument_object_not_a_string.rex",
    // Task 8: EXPOSE and the scope-keyed variable pool behind it.
    "lang/expose_class_variable.rex",
    "lang/expose_stem.rex",
    "lang/expose_internal_call.rex",
    "lang/expose_indirect_list.rex",
    "lang/expose_outside_a_method.rex",
    "lang/expose_object_value.rex",
    // Task 8, ruling R26: `::CLASS ... SUBCLASS`, and the two-scope program
    // it makes reachable.
    "lang/class_subclass.rex",
    "lang/class_subclass_not_found.rex",
    "lang/class_subclass_cycle.rex",
    "lang/expose_two_scopes.rex",
    // Task 8 fix round 1: the failing-::CONSTANT blame target under a
    // dependency-ordered class install.
    "lang/directive_constant_blames_the_last_installed_class.rex",
    // Phase 5a (2026-08-17 plan) Task 6: an operator forwarded through a
    // stem to its default value's own native method carries the
    // `Compiled method` traceback frame the oracle emits, scope "String".
    "lang/operator_frame_stem_plus.rex",
    "lang/operator_frame_stem_power.rex",
    "lang/operator_frame_stem_prefix_minus.rex",
    // Fix round 1, finding 3: the same forwarded frame reached from a
    // controlled DO header's numeric position, one program per position.
    "lang/operator_frame_stem_do_initial.rex",
    "lang/operator_frame_stem_do_to.rex",
    "lang/operator_frame_stem_do_by.rex",
];

#[test]
fn phase_5a_subset_matches_the_committed_list() {
    let corpus_dir = corpus_dir();
    let subset = read_subset(&[&corpus_dir.join("phase-5a.txt")]);
    assert_eq!(
        subset, EXPECTED_SUBSET_5A,
        "phase-5a.txt's entries drifted from EXPECTED_SUBSET_5A -- adding or \
         removing a line from the 5a subset is a plan amendment, and must \
         change both the file and this list together, or a deleted line \
         shrinks every measurement that reads phase-5a.txt with nothing to \
         notice it"
    );
}

/// Criterion 1's coverage property, read against the **union** of every
/// phase's subset file rather than `phase-4a.txt` alone.
///
/// A later phase's witness cannot live in `phase-4a.txt`, whose own header
/// excludes those constructs by definition -- `INTERPRET` is the first such
/// case. Reading the union rather than swapping the file
/// is what keeps every earlier witness exercised as later phases add their own
/// subsets, which is why `read_subset` takes a slice at all.
///
/// [`EXPECTED_SUBSET`]'s own test deliberately does **not** widen with this
/// one: it pins `phase-4a.txt`'s exact line list, and a union would destroy
/// that.
#[test]
fn every_in_scope_variant_is_witnessed_by_the_phase_subsets() {
    let mut instructions = Coverage::new("InstructionKind", INSTRUCTION_TAGS);
    let mut exprs = Coverage::new("ExprKind", EXPR_TAGS);
    let mut loops = Coverage::new("LoopKind", LOOP_TAGS);
    let mut prefix_ops = Coverage::new("PrefixOp", PREFIX_OP_TAGS);
    let mut end_styles = Coverage::new("EndStyle", END_STYLE_TAGS);
    let mut traces = Coverage::new("Trace", TRACE_TAGS);
    let mut operators = Coverage::new("Operator", OPERATOR_TAGS);

    let corpus_dir = corpus_dir();
    let paths: Vec<PathBuf> = SUBSET_FILES
        .iter()
        .map(|name| corpus_dir.join(name))
        .collect();
    let subset = read_subset(&paths.iter().map(PathBuf::as_path).collect::<Vec<_>>());
    assert!(
        !subset.is_empty(),
        "the phase subset files named no programs -- that is a corpus defect, \
         not an empty pass"
    );

    for rel_path in &subset {
        let abs = corpus_dir.join(rel_path);
        let text = fs::read(&abs).unwrap_or_else(|e| panic!("cannot read {}: {e}", abs.display()));
        let p = parse_program(text)
            .unwrap_or_else(|e| panic!("{} failed to parse: {e:?}", abs.display()));
        assert_program_has_only_admitted_directives(&abs, &p);

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
                "{}: {} in-scope variant(s) unwitnessed by the phase subsets: {}",
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
/// (`VALUE`, `ADDRESS`, `QUEUED`), written as a literal rather than parsed
/// from the prose file -- the same choice `tests/assertions.rs`'s `EXEMPT`
/// list makes against that file's builtin set. Changing a name there without
/// changing the file (or vice versa) is exactly the drift this test exists to
/// catch.
///
/// **The literal lives in `rexx_inventory::builtins`, not here.** A private
/// `const` in this file is unreachable from any other test binary and from
/// `src/`, and the same list is what a builtin dispatch consults and what
/// `tests/builtin_status.rs` derives its `excluded` rows from. One copy, in
/// the crate that already holds the generated builtin table, is what keeps
/// those three readers agreeing.
use rexx_inventory::builtins::EXCLUDED as EXCLUDED_BUILTINS;

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

    // Every partial row must also be an excluded row, or `wholly_excluded`
    // subtracts a name that was never in the set and silently returns 16.
    for partial in rexx_inventory::builtins::PARTIALLY_EXCLUDED {
        assert!(
            EXCLUDED_BUILTINS.contains(partial),
            "{partial} is a partial exclusion but is not in EXCLUDED at all"
        );
    }
    assert_eq!(
        rexx_inventory::builtins::wholly_excluded().len(),
        15,
        "the whole exclusions are EXCLUDED less the partial rows"
    );

    // Derived, not asserted: this is the number phase-4-exclusions.txt's own
    // header reads out ("66 of the 81 builtins ... are in scope, three of
    // them partially"), and this line is how that phrasing stays synced to
    // the actual table rather than to a copy-pasted figure.
    let in_scope = names.len() - rexx_inventory::builtins::wholly_excluded().len();
    assert_eq!(
        in_scope, 66,
        "66 of the 81 builtins are in scope, three of them partially -- see \
         phase-4-exclusions.txt"
    );
    assert_eq!(
        rexx_inventory::builtins::in_scope().len(),
        in_scope,
        "in_scope() must enumerate exactly the names that arithmetic counts"
    );
}
