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
    EndTarget, Expr, ExprKind, Instruction, InstructionKind, Loop, LoopKind, Program, Trace,
    parse_program,
};

#[path = "owners.rs"]
mod owners;
use owners::{
    Coverage, END_STYLE_TAGS, EXPR_TAGS, INSTRUCTION_TAGS, LOOP_TAGS, OPERATOR_TAGS,
    PREFIX_OP_TAGS, TRACE_TAGS, end_style_tag, expr_tag, instruction_tag, loop_tag, operator_tag,
    prefix_op_tag, trace_tag,
};

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

/// The union of every non-comment, non-blank line across `list_paths`, in
/// first-seen order, each entry appearing once even if two files name the
/// same corpus program.
///
/// **Task 0's Step 4.** Was a single-file reader (`&Path`); widened to `&[&Path]`
/// so a later task's own subset file (4b's, say) can run *alongside*
/// `phase-4a.txt` rather than replacing it -- every earlier-phase witness
/// stays exercised as later phases add their own subset files, instead of
/// each phase's own harness run choosing between "4a's programs" and "my
/// own programs" and losing the other's coverage. Today's callers all pass
/// a one-element slice containing only `phase-4a.txt`, so the union is that
/// file's own content unchanged -- this task ships no behaviour change.
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

fn corpus_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus")
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
    let subset = read_subset(&[&corpus_dir.join("phase-4a.txt")]);
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
