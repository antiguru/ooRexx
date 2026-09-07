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
//! `ExprKind` had six out-of-scope variants at Task 16 gate time and has
//! none now. Five of the six were not in the design spec at all -- the spec
//! names only `Message`
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
//! **Every `ExprKind` variant is in scope now.** That is not a claim that
//! every call target runs: a builtin-named call still fails loudly, through
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
/// `each_instruction` below, plus the ones that own no body for it to walk.
///
/// The list is the match below rather than this sentence; the test beside
/// this function walks every one of `DirectiveKind`'s variants and holds each
/// against its own arm.
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
        | DirectiveKind::Constant(_)
        | DirectiveKind::Options(_)
        | DirectiveKind::Annotate(_)
        | DirectiveKind::Requires(_) => true,
        DirectiveKind::Resource(_) => false,
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
/// **any** variant across [`is_admitted_directive_kind`]'s arms in either
/// direction reddens here specifically, with the wrong keyword named in the
/// failure. Which variants sit on which side is that function's own match and
/// is not restated here: `every_directive_keyword_reaches_its_node`'s literal
/// list and this test's own `cases` table together enumerate the kinds, and
/// the assertion below is what holds them equal.
#[test]
fn every_directive_keyword_is_correctly_admitted_or_refused() {
    let cases: &[(&str, &str, bool)] = &[
        ("::annotate package\n", "ANNOTATE", true),
        ("::attribute a\n", "ATTRIBUTE", true),
        ("::class c\n", "CLASS", true),
        ("::constant k 1\n", "CONSTANT", true),
        ("::method m\n  return 1\n", "METHOD", true),
        ("::options noprolog\n", "OPTIONS", true),
        ("::requires \"nosuch\"\n", "REQUIRES", true),
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
    let p = parse_program(b"::RESOURCE d\nbody\n::END\n".to_vec()).expect("::RESOURCE parses");
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
    "phase-5b.txt",
    "phase-5c.txt",
    "phase-5d.txt",
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
    // Fix round 2, finding 1: the same forwarded frame reached from a
    // failing step past the receiver's own conversion.
    "lang/operator_frame_stem_divide_by_zero.rex",
    "lang/operator_frame_stem_power_exponent_range.rex",
    "lang/operator_frame_stem_logical_and.rex",
    "lang/operator_frame_stem_prefix_not.rex",
    "lang/operator_frame_stem_do_exponent_range.rex",
    // Fix round 3, finding 1: the same forwarded frame reached through a
    // non-strict comparison's own numeric conversion.
    "lang/operator_frame_stem_compare_overflow.rex",
    // Phase 5a (2026-08-17 plan) Task 7: `::CLASS ... MIXINCLASS` and
    // `::CLASS ... INHERIT`, and the refusal ladder `INHERIT` brings with it.
    "lang/class_mixinclass.rex",
    "lang/class_inherit_order.rex",
    "lang/class_inherit_not_found.rex",
    "lang/class_inherit_trailing_keyword.rex",
    "lang/class_inherit_not_a_mixin.rex",
    "lang/class_inherit_base_class.rex",
    "lang/class_inherit_recursive.rex",
    "lang/class_inherit_cycle.rex",
    "lang/class_metaclass_cycle.rex",
    // Phase 5a (2026-08-17 plan) Task 8: `::CLASS ... METACLASS`, the
    // refusals it brings, and ABSTRACT on a metaclass.
    "lang/class_metaclass.rex",
    "lang/class_metaclass_class_method_does_not_donate.rex",
    "lang/class_metaclass_superclass_wins.rex",
    "lang/class_metaclass_not_found.rex",
    "lang/class_metaclass_not_a_metaclass.rex",
    "lang/class_abstract_metaclass.rex",
    "lang/class_abstract_metaclass_subclass.rex",
    "lang/class_abstract_metaclass_after_inherit.rex",
    // Phase 5a (2026-08-17 plan) Task 9: the Object and Class reflection
    // protocol, `~method`'s own-dictionary rule, the argument refusals of the
    // methods that take one, and what a program can do with the array
    // `~superClasses` answers.
    "lang/class_reflection.rex",
    "lang/class_package.rex",
    "lang/class_method_own_dictionary.rex",
    "lang/class_method_class_side_raises.rex",
    "lang/class_reflection_argument_ladder.rex",
    "lang/array_make_string.rex",
    "lang/array_make_string_refusals.rex",
    "lang/array_unknown_method.rex",
    "lang/array_list_expression.rex",
    "lang/array_do_over.rex",
    "lang/directory_at_and_put.rex",
    "lang/array_index_refusals.rex",
    "lang/directory_index_refusals.rex",
    "lang/array_trace.rex",
    "lang/raise_additional_array.rex",
    "lang/raise_array_spelling.rex",
    "lang/raise_keyword_object_traces.rex",
    "lang/raise_array_nested.rex",
    "lang/raise_additional_nested.rex",
    // Phase 5a (2026-08-17 plan) Task 12: the UNKNOWN forward and the
    // NOMETHOD condition beneath it, which are what the documented search
    // order has after the class chain.
    "lang/message_send_unknown_forward.rex",
    "lang/condition_nomethod.rex",
    "lang/method_access_private.rex",
    "lang/method_access_private_refused.rex",
    "lang/method_access_package_and_protected.rex",
    "lang/method_access_private_attribute.rex",
    // Phase 5a (2026-08-17 plan) Task 14: the required-string protocol -- every
    // context `provide.xml` `reqstr` lists that this phase can reach, with a
    // `makeString` and without one, the NOSTRING condition, the messages the
    // protocol answers in its own right, and the argument, operator and
    // traceback surfaces the conversion changes.
    "lang/required_string_contexts.rex",
    "lang/required_string_default_name.rex",
    "lang/required_string_nostring.rex",
    "lang/required_string_face.rex",
    "lang/required_string_builtin_arguments.rex",
    "lang/required_string_nostring_no_directive.rex",
    "lang/required_string_builtin_raw_argument.rex",
    "lang/required_string_operator_argument.rex",
    "lang/required_string_method_argument.rex",
    "lang/required_string_make_string_raises.rex",
    "lang/required_string_argument_make_string_raises.rex",
    // Task 15: the generated accessor pair, and ABSTRACT's send-time refusal.
    // The value program, and the refusals a program can read out of the pair,
    // which are separate programs because a program can only fail once.
    "lang/method_attribute_generated.rex",
    "lang/method_attribute_generated_no_result.rex",
    "lang/method_attribute_generated_getter_arguments.rex",
    "lang/method_attribute_generated_setter_arguments.rex",
    "lang/method_attribute_generated_setter_omitted.rex",
    "lang/method_abstract_send.rex",
    // Task 16: the GUARD instruction, and REPLY inside a method. The guard's
    // spellings in one program; REPLY's own transcript, whose whole point is
    // exit status 0 with a traceback; the legality refusals, one program each
    // because a program can only fail once -- which is not the same as each
    // being fatal, since a second REPLY raises from the body the first one
    // left owed and so is itself rc 0; what an owed body's raise does to the
    // exit status; and a reply chained out of an owed body.
    "lang/method_guard_instruction.rex",
    "lang/method_reply.rex",
    "lang/method_guard_outside_method.rex",
    "lang/method_reply_outside_method.rex",
    "lang/method_reply_twice.rex",
    "lang/method_reply_no_result.rex",
    "lang/method_reply_exit_status.rex",
    "lang/method_reply_chain.rex",
    // GUARD's own value check: a WHEN expression naming an exposed variable
    // whose value is not exactly `0` or `1` is 34.902, not IF's or WHEN's.
    "lang/method_guard_when_not_logical.rex",
    // Task 17: the environment search order this phase can observe, the
    // Directory entry-method mechanism on the shipped entry and on one a
    // program stores, and the `.METHODS` and `.ROUTINES` tables --
    // `.RESOURCES` is `dispatch.rs`'s own test, because a `::RESOURCE` body is
    // source lines that no clause span covers and `rexx-parse`'s
    // `every_corpus_program_tiles` requires every byte of a corpus program to
    // be tiled by one.
    "lang/environment_local_shadows_the_environment.rex",
    "lang/environment_directory_entry_method.rex",
    "lang/environment_methods_join.rex",
    "lang/environment_routines_table.rex",
    // Task 18: the install passes and class-object initialization. The
    // INIT/ACTIVATE pair, which discriminates only together; a constant
    // expression naming a class declared later; the construction order every
    // pass walks; what each form of a `::CONSTANT` answers; the 97.4 an
    // expression form gets while its own class is still being built; and the
    // class a failing `ACTIVATE` is blamed against.
    "lang/class_init_activate_inherit_merge.rex",
    "lang/class_constant_expression_later_class.rex",
    "lang/class_init_activate_order.rex",
    "lang/class_constant_values.rex",
    "lang/class_constant_uninitialized.rex",
    "lang/class_activate_failure_blames_the_last_installed_class.rex",
    // Task 18, fix round 1: the duplicate-member refusals. One row per code,
    // because a program can only fail once -- `::METHOD`'s own, `::METHOD`
    // colliding with a `::CONSTANT` that occupies both dictionaries,
    // `::ATTRIBUTE`'s own reached through a setter key nothing in the file
    // spells, `::CONSTANT`'s own, and the `CLASS` keyword with no `::CLASS`
    // above it -- and then the negative control, whose every name is written
    // twice and which the oracle runs.
    "lang/class_duplicate_method.rex",
    "lang/class_duplicate_constant_and_method.rex",
    "lang/class_duplicate_attribute.rex",
    "lang/class_duplicate_constant.rex",
    "lang/class_member_class_keyword_needs_class.rex",
    "lang/class_member_names_per_side.rex",
    // Task 18, fix round 3: the rest of the duplicate-directive family. The
    // `::CLASS` refusal, whose pair is spelled two ways and is not adjacent,
    // with a `::CLASS` naming an unresolvable superclass below it; and the
    // control that says each directive kind keeps its own table of names.
    // `::RESOURCE` has no row and cannot -- `every_corpus_program_tiles`
    // rejects a `::RESOURCE` body byte by byte -- so `run/tests.rs`'s
    // `a_duplicate_resource_name_is_refused_and_a_distinct_one_is_not` is its
    // sole instrument.
    "lang/class_duplicate_class.rex",
    "lang/class_directive_names_are_their_own_table.rex",
    // Task 19: `::CONSTANT`'s instance-side getter, read through `~method`
    // with a class-only `::METHOD` beside it in the same class; the
    // expression's own activation, whose `SELF`, `SUPER` and receiver the two
    // classes and the `PRIVATE` class method separate; and the failure path
    // through that activation, whose traceback carries a method clause.
    "lang/class_constant_instance_method.rex",
    "lang/class_constant_expression_self.rex",
    "lang/class_constant_expression_method_failure.rex",
    // Task 20: ::ANNOTATE's six targets and the readback -- the targets and
    // their handles, the live annotation table, and the refusal whose target
    // is declared below the ::ANNOTATE.
    "lang/directive_annotate_targets.rex",
    "lang/directive_annotate_table_is_live.rex",
    "lang/directive_annotate_missing_target.rex",
    // Task 21: the REXX_DEFINED lock, one row per mutator so that the frame
    // line names each of the five it refuses.
    "lang/class_rexx_defined_define.rex",
    "lang/class_rexx_defined_define_methods.rex",
    "lang/class_rexx_defined_delete.rex",
    "lang/class_rexx_defined_inherit.rex",
    "lang/class_rexx_defined_uninherit.rex",
    // Task 21: the same five on a class the file declares, where they
    // succeed -- the identity split between `~define` and `~defineMethods`,
    // the omitted argument's tombstone against `.nil`'s removal,
    // `~inherit`'s position, and the refusals.
    "lang/class_mutators_user_class.rex",
    "lang/class_mutator_define_nil_removes.rex",
    "lang/class_mutator_delete_absent_name.rex",
    "lang/class_mutator_inherit_position.rex",
    "lang/class_mutator_inherit_position_not_inherited.rex",
    "lang/class_mutator_refusals.rex",
    "lang/class_mutator_uninherit_not_inherited.rex",
    "lang/class_mutator_define_methods_supplier.rex",
    // Task 21: Setup.cpp's own removal and hiding, which `~method` reads
    // apart, each removal against the same name on the class it was donated
    // from.
    "lang/class_native_hiding.rex",
    "lang/class_native_removal_sort.rex",
    "lang/class_native_removal_make_string.rex",
    // Task 21: the Package object -- `RexxContext~package`, the two class
    // tables, and the refusal the REXX package gives an addition. Beside them
    // the context object's own rows, which pin that object's identity rather
    // than the package's: one per activation, surviving a forced collection
    // while its activation is running or suspended, and surviving one while
    // it is parked by a REPLY.
    "lang/class_context_package.rex",
    "lang/class_context_identity.rex",
    "lang/class_context_gc.rex",
    "lang/class_context_reply.rex",
    "lang/class_package_classes.rex",
    "lang/class_package_addition_refused.rex",
    // `::METHOD ... EXTERNAL 'LIBRARY REXX name'`: the bind, its eager
    // failure, the entry point's own argument check, and where the
    // resolution sits in the install walk, pinned from either side. The last
    // is gate table D's own `::METHOD EXTERNAL` probe, whose row this task
    // made `agree`.
    "lang/directive_method_external_bind.rex",
    "lang/directive_method_external_missing.rex",
    "lang/directive_method_external_arguments.rex",
    "lang/directive_method_external_duplicate_wins.rex",
    "lang/directive_method_external_source_order.rex",
    "lang/directive_method_external_before_duplicate.rex",
    "lang/directive_method_external_not_a_staged_gap.rex",
    "gate-tables/directives/method__external__subkeyword.rex",
    "lang/message_send_scope_override_chain.rex",
    "lang/message_send_scope_override_not_a_scope.rex",
    "lang/library_bootstrap_state.rex",
    "lang/library_bootstrap_setup_methods_gone.rex",
    "lang/do_over_string_table.rex",
    "lang/string_upper.rex",
    "lang/library_method_traceback.rex",
    "lang/library_method_traceback_nested.rex",
    // Task 23 fix round 3: the `REXX_DEFINED` lock on the classes the library
    // declares. The `class_rexx_defined_*` rows above are each `.Array`, a
    // class `Setup.cpp` builds, so they see one half of the flagged set.
    "lang/class_rexx_defined_library_inherit.rex",
    "lang/class_rexx_defined_library_define.rex",
    "lang/class_rexx_defined_library_no_mutation.rex",
    // `::ATTRIBUTE ... EXTERNAL 'LIBRARY REXX name'` and the `::METHOD ...
    // ATTRIBUTE EXTERNAL` spelling of it, with gate table D's own probe for
    // the row last, exactly as the `::METHOD EXTERNAL` block above ends.
    "lang/directive_attribute_external_bind.rex",
    "lang/directive_attribute_external_missing.rex",
    "lang/directive_attribute_external_get_third_word.rex",
    "lang/directive_attribute_external_set_default.rex",
    "lang/directive_attribute_external_arguments.rex",
    "lang/directive_method_attribute_external_missing.rex",
    "gate-tables/directives/attribute__external__subkeyword.rex",
    // `Method~scope` on the routes to a method object beyond the directive
    // pair `gate-tables/concepts/xscope.rex` asks about.
    "lang/method_scope.rex",
    // `Class~subclass` and `Class~mixinClass`: what the factory builds, and
    // the argument ladder whose order puts the metaclass before the class id.
    "lang/class_subclass_factory.rex",
    "lang/class_subclass_refusals.rex",
    // `.RexxInfo`: the environment entry that is an instance, and the class
    // object behind it that no environment name reaches.
    "lang/rexxinfo_entry.rex",
    // A method compiled from source text, through each of the two callers
    // that compile one, ending on the position string that separates their
    // reports.
    "lang/method_from_source.rex",
    "lang/method_from_source_table.rex",
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

/// `phase-5b.txt`'s exact line list, the same device [`EXPECTED_SUBSET_5A`]
/// is for `phase-5a.txt`, and added for the same reason: committed empty by
/// Task 0, with each later 5b task appending its own witness and this list
/// together.
const EXPECTED_SUBSET_5B: &[&str] = &[
    // Task 1: the concept-row probes `~new` makes agree, and the witnesses
    // of what those rows do not send.
    "gate-tables/concepts/creo.rex",
    "gate-tables/concepts/abscla.rex",
    "lang/instance_naming.rex",
    "lang/instance_naming_overrides.rex",
    "lang/instance_naming_raises.rex",
    "lang/instance_self_reassigned.rex",
    // Task 1 fix round: the operand positions a named instance answers in.
    "lang/instance_named_operands.rex",
    // Task 5: the concept row a class-side `UNINIT` makes agree, the two
    // instance deliveries, the two inherited class arms, the sweep order, and
    // the delivery points -- a registration outliving its method, an
    // allocating finalizer, and a collection driven from inside a finalizer
    // at each of the interlock's copies.
    "gate-tables/concepts/obdes.rex",
    "lang/uninit_instance_collected.rex",
    "lang/uninit_instance_retained.rex",
    "lang/uninit_class_inherited.rex",
    "lang/uninit_class_mixin.rex",
    "lang/uninit_class_inherit_runtime.rex",
    "lang/uninit_class_sweep_order.rex",
    "lang/uninit_class_uninherit.rex",
    "lang/uninit_allocating_finalizer.rex",
    "lang/uninit_nested_collection.rex",
    "lang/uninit_nested_collection_at_exit.rex",
    // Task 2: the concept row the behaviour snapshot makes agree, the two
    // rebuild-in-place arms, the copying family's third member, and the
    // subclass shape where the two families agree.
    "gate-tables/concepts/objcla.rex",
    "lang/class_behaviour_snapshot_inherit.rex",
    "lang/class_behaviour_snapshot_uninherit.rex",
    "lang/class_behaviour_snapshot_delete.rex",
    "lang/class_behaviour_snapshot_subclass.rex",
    // Task 3: the concept row per-object methods make agree, and the shapes
    // that row cannot see. `usesem.rex` defines its one-off nowhere else, so
    // it cannot see the search order, the scope argument, either
    // restricted-private refusal, or the no-method form.
    "gate-tables/concepts/usesem.rex",
    "lang/setmethod_precedence.rex",
    "lang/setmethod_float_scope.rex",
    "lang/setmethod_object_scope.rex",
    "lang/setmethod_hidden.rex",
    "lang/setmethod_uninit.rex",
    "lang/setmethod_private_refusal.rex",
    "lang/setmethod_restricted_refusal.rex",
    "lang/setmethod_restricted_allowed.rex",
    "lang/enhanced_scope.rex",
    "lang/method_source_reported_name.rex",
    // Task 3 fix round: the level `Class~enhanced`'s methods sit at, which
    // `unsetMethod` reveals rather than removes.
    "lang/enhanced_unset.rex",
    // Task 4: the two table D rows `DELEGATE` makes agree, and the arms
    // neither row can see -- which variable a delegate reads, at which
    // scope, and that it leaves no traceback frame of its own.
    "gate-tables/directives/method__delegate__subkeyword.rex",
    "gate-tables/directives/attribute__delegate__subkeyword.rex",
    "lang/delegate_variable.rex",
    "lang/delegate_no_frame.rex",
    "lang/delegate_private.rex",
    // Task 4: `FORWARD`, which the `DELEGATE` rows do not reach at all --
    // each option, the defaults, `CONTINUE`'s two outcomes, the two refusals
    // no other instruction raises, and the traceback frame that is the
    // difference from `DELEGATE`.
    "lang/forward_class_super.rex",
    "lang/forward_options.rex",
    "lang/forward_continue.rex",
    "lang/forward_outside_method.rex",
    "lang/forward_arguments_not_an_array.rex",
    "lang/forward_frame.rex",
    "lang/forward_after_reply.rex",
    // Task 4 fix round: what the send does to the forwarding activation, and
    // what `ARGUMENTS` does to its value. `forward_phantom_trap.rex` is the
    // condition rule -- a trap armed in the forwarding method does not see a
    // condition the send raises and the caller's does -- with the four
    // adjacent successes that bound it to a non-continuing FORWARD's own
    // send. `forward_class_scope.rex` is the scope-override validation the
    // send performs, whose placement decides both the error number and
    // whether the forwarding method can trap it, and which names the TO
    // target rather than SELF. `forward_arguments_converted.rex` is
    // `requestArray` over a stem and over a string's lines, where the
    // predicate that stood there answered a different question.
    // `forward_class_trace.rex` is the `>K>` line an invalid CLASS does not
    // write, which is the one thing about `FORWARD`'s options that only a
    // trace transcript can see.
    "lang/forward_phantom_trap.rex",
    "lang/forward_class_scope.rex",
    "lang/forward_arguments_converted.rex",
    "lang/forward_class_trace.rex",
    // Task 7: the four paths, and the refusal rows a witness of the answers
    // cannot see.
    "lang/object_copy.rex",
    "lang/object_copy_class_refusal.rex",
    "lang/object_run.rex",
    "lang/object_run_refusals.rex",
    "lang/object_send.rex",
    "lang/object_send_refusals.rex",
    "lang/object_send_name_refusal.rex",
    "lang/object_start.rex",
    // Task 8: the concept row multidimensional `Array` makes agree, what that
    // row does not reach -- the shapes `~new` builds, the reshape a write
    // past a dimension performs, and the single-dimension `[]=` that extends
    // instead -- and the error surface, where the same rejected subscript is
    // 93.924 or 93.907 by which index kind it belongs to.
    "gate-tables/concepts/methodsbyclass.rex",
    "lang/array_multidimensional.rex",
    "lang/array_multidimensional_refusals.rex",
    // Task 9: the array size the allocator refuses, which is the one array
    // error `MaxFixedArraySize` does not decide, beside that limit's own
    // 93.959 and a size in reach.
    "lang/array_allocation_refused.rex",
    // Review fix round: the argument slots `arrayArgument` guards, each handed
    // a multi-dimensional array, with the single-dimensional neighbour that
    // answers ahead of it. The named overload's 88.913 under two argument
    // names, the positional overload's 98.913, and `FORWARD`'s own 98.946.
    "lang/object_send_multidimensional.rex",
    "lang/object_start_multidimensional.rex",
    "lang/forward_arguments_multidimensional.rex",
    "lang/object_run_multidimensional.rex",
];

#[test]
fn phase_5b_subset_matches_the_committed_list() {
    let corpus_dir = corpus_dir();
    let subset = read_subset(&[&corpus_dir.join("phase-5b.txt")]);
    assert_eq!(
        subset, EXPECTED_SUBSET_5B,
        "phase-5b.txt's entries drifted from EXPECTED_SUBSET_5B -- adding or \
         removing a line from the 5b subset is a plan amendment, and must \
         change both the file and this list together, or a deleted line \
         shrinks every measurement that reads phase-5b.txt with nothing to \
         notice it"
    );
}

/// `phase-5c.txt`'s exact line list, the same device [`EXPECTED_SUBSET_5A`]
/// and [`EXPECTED_SUBSET_5B`] are for their own files.
///
/// Written whole rather than grown a task at a time: 5c's witnesses waited in
/// test binaries of their own until its method rows could be owned per class,
/// and Phase 5d's Task 1 is where they land.
const EXPECTED_SUBSET_5C: &[&str] = &[
    // The three interim witnesses, each moved here from its own binary.
    "lang/variable_reference.rex",
    "lang/stem_object.rex",
    "lang/string_makearray.rex",
    // `::OPTIONS`, whose whole family 5c delivered.
    "lang/directive_options.rex",
    "lang/directive_options_call_on_keeps_it.rex",
    "lang/directive_options_digits_below_fuzz.rex",
    "lang/directive_options_internal_call.rex",
    "lang/directive_options_nostring_syntax.rex",
    "lang/directive_options_novalue_syntax.rex",
    "lang/directive_options_novalue_trap_wins.rex",
    "lang/directive_options_numeric_inherit.rex",
    "lang/directive_options_signal_off.rex",
    "lang/directive_options_trace.rex",
    "lang/directive_options_trace_method.rex",
    "lang/directive_options_trace_reply.rex",
    // The Phase 3 and Phase 4 programs no subset file had ever named, filed
    // here because this is the file that exists rather than because 5c wrote
    // them. Each was compared against the oracle before being filed.
    "lang/call_procedure.rex",
    "lang/condition_syntax.rex",
    "lang/do_variants.rex",
    "lang/gate_variants.rex",
    "lang/keyword_as_variable.rex",
    "lang/source_arg.rex",
    "lang/string_builtins.rex",
    "lang/whitespace_significant.rex",
    // The 5c follow-up's `MutableBuffer` witnesses, filed as each task lands.
    "lang/mutablebuffer_state.rex",
    "lang/mutablebuffer_instance.rex",
    "lang/mutablebuffer_readers.rex",
    "lang/mutablebuffer_mutators.rex",
    "lang/mutablebuffer_caseless.rex",
    "lang/mutablebuffer_conversion.rex",
    // Phase 5f's `String` witnesses, filed as each family lands.
    "lang/string_search.rex",
    "lang/string_search_refusals.rex",
    "lang/string_counts.rex",
    "lang/string_counts_refusals.rex",
    "lang/string_match.rex",
    "lang/string_match_refusals.rex",
    "lang/string_extract.rex",
    "lang/string_extract_refusals.rex",
    "lang/string_words.rex",
    "lang/string_words_refusals.rex",
    "lang/string_edits.rex",
    "lang/string_edits_refusals.rex",
    "lang/string_rewrites.rex",
    "lang/string_rewrites_refusals.rex",
    "lang/string_rounding.rex",
    "lang/string_rounding_refusals.rex",
    "lang/string_base64.rex",
    "lang/string_base64_refusals.rex",
    "lang/object_hashcode.rex",
    "lang/object_hashcode_refusals.rex",
    "lang/string_operators.rex",
    "lang/string_operators_refusals.rex",
    "lang/string_choice.rex",
    "lang/string_choice_refusals.rex",
    "lang/string_pad.rex",
    "lang/string_pad_refusals.rex",
    "lang/string_strip.rex",
    "lang/string_strip_refusals.rex",
    "lang/string_compare.rex",
    "lang/string_compare_refusals.rex",
    "lang/string_convert.rex",
    "lang/string_convert_refusals.rex",
    "lang/string_bits.rex",
    "lang/string_bits_refusals.rex",
    "lang/string_datatype.rex",
    "lang/string_datatype_refusals.rex",
    "lang/string_numeric.rex",
    "lang/string_numeric_refusals.rex",
    "lang/string_extremes.rex",
    "lang/string_extremes_refusals.rex",
    "lang/string_caseless.rex",
    "lang/string_caseless_refusals.rex",
    "lang/array_enumeration.rex",
    "lang/array_item_argument.rex",
    "lang/array_index_argument.rex",
    "lang/array_extra_argument.rex",
    "lang/supplier_iteration.rex",
    "lang/array_navigation.rex",
    "lang/array_structure.rex",
    "lang/array_structure_refusals.rex",
    "lang/array_sorting.rex",
    "lang/queue_operations.rex",
    "lang/list_operations.rex",
    "lang/collection_subclasses.rex",
    "lang/collection_of.rex",
    "lang/array_append_index.rex",
    "lang/queue_bounds.rex",
    "lang/collection_item_equality.rex",
    "lang/queue_extent.rex",
    "lang/collection_callback_mutates.rex",
    "lang/nil_comparison.rex",
    "lang/list_index_conversion.rex",
    "lang/list_empty_entry.rex",
    "lang/list_empty_and_section.rex",
    "lang/supplier_bounds.rex",
    "lang/collection_construction.rex",
    "lang/sort_comparisons.rex",
    "lang/array_dimensions_append.rex",
    "lang/hash_table_store.rex",
    "lang/set_operations.rex",
    "lang/relation_multivalue.rex",
    "lang/directory_string_keys.rex",
    "lang/stem_collection.rex",
    "lang/collection_copy.rex",
    "lang/directory_set_method.rex",
    "lang/method_new.rex",
    "lang/stem_request_and_directory.rex",
    "lang/do_over_request_array.rex",
];

#[test]
fn phase_5c_subset_matches_the_committed_list() {
    let corpus_dir = corpus_dir();
    let subset = read_subset(&[&corpus_dir.join("phase-5c.txt")]);
    assert_eq!(
        subset, EXPECTED_SUBSET_5C,
        "phase-5c.txt's entries drifted from EXPECTED_SUBSET_5C -- adding or \
         removing a line from the 5c subset is a plan amendment, and must \
         change both the file and this list together, or a deleted line \
         shrinks every measurement that reads phase-5c.txt with nothing to \
         notice it"
    );
}

/// `phase-5d.txt`'s exact line list, the same device [`EXPECTED_SUBSET_5A`],
/// [`EXPECTED_SUBSET_5B`] and [`EXPECTED_SUBSET_5C`] are for their own files.
const EXPECTED_SUBSET_5D: &[&str] = &[
    // Task 3's operator-method witness.
    "lang/operator_methods.rex",
    // Task 4's `::REQUIRES` witness and Task 5's namespace one, each with two
    // `.cls` helpers beside it that no corpus scan sees.
    "lang/package_requires.rex",
    "lang/package_namespace.rex",
];

#[test]
fn phase_5d_subset_matches_the_committed_list() {
    let corpus_dir = corpus_dir();
    let subset = read_subset(&[&corpus_dir.join("phase-5d.txt")]);
    assert_eq!(
        subset, EXPECTED_SUBSET_5D,
        "phase-5d.txt's entries drifted from EXPECTED_SUBSET_5D -- adding or \
         removing a line from the 5d subset is a plan amendment, and must \
         change both the file and this list together, or a deleted line \
         shrinks every measurement that reads phase-5d.txt with nothing to \
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
