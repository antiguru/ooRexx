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

//! The single owner table `coverage.rs` (criterion 1, parse coverage) and
//! `loud.rs` (criterion 5, loud failures) both read, instead of each
//! hand-maintaining its own copy (inherited item I36).
//!
//! An integration test cannot `mod` another test binary's directory, so the
//! sharing goes through `#[path = "owners.rs"] mod owners;` -- see
//! `coverage.rs`'s and `loud.rs`'s own top for that line, and
//! [`the_two_harnesses_include_this_exact_file`] below for the regression
//! guard that a divergent private copy cannot silently reappear.
//!
//! This file is itself a normal, cargo-discovered integration test (it lives
//! directly under `tests/`, like `coverage.rs` and `loud.rs`), so its own
//! tests below run under `cargo test --test owners` in addition to running a
//! second and third time as part of `coverage`'s and `loud`'s own binaries
//! (each `#[path]`-including this same source). That tripling is deliberate:
//! it is what makes `cargo test -p rexx-exec --test coverage` alone (without
//! also running `owners`) still verify this file's own invariants.
//!
//! # `#![allow(dead_code)]`, file-wide
//!
//! Three independent binaries compile this file (`owners`, `coverage`,
//! `loud`), and no single one of them calls every item below -- `Coverage::
//! unwitnessed`, for instance, is `coverage.rs`'s own and `loud.rs` never
//! calls it, while `loud.rs`'s own consumers need `EXPECTED_OUT_OF_SCOPE`
//! only through `coverage.rs`. Denying `dead_code` per item would need a
//! different `#[allow]`/`#[expect]` shape in each of the three
//! compilations for the same source line, and `#[expect]` in particular
//! would be *wrong* here: it demands the lint actually fire, and whether it
//! fires for a given item depends on which of the three binaries is
//! compiling it. A file-wide `allow` is the honest statement of what this
//! file is -- a shared library of data and functions meant to be used
//! selectively by its callers, not a self-contained test module.
#![allow(dead_code)]

use std::collections::HashSet;

use rexx_parse::{ExprKind, InstructionKind, LoopKind, Operator, PrefixOp, Trace};

/// Who is responsible for a variant this file enumerates.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) enum Owner {
    /// Implemented here: must be witnessed by at least one program in the
    /// subset.
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
/// criterion 1 and criterion 5 both need ownership from one invocation for
/// the same reason Phase 3's needed the tag alone from one.
///
/// # The `split` form
///
/// A variant whose arms do not share one owner takes the trailing `split
/// PATTERN in (EXPR) { .. }` section, which expands to a nested `match` on
/// `EXPR` and contributes one row per arm to `$list`. Both matches stay
/// wildcard-free, so a new variant of either enum is a compile error here.
///
/// `split` rather than an ordinary row because the arms live behind a `Box`
/// (`InstructionKind::Call(Box<Call>)`) and box patterns are unstable, so
/// `InstructionKind::Call(Call::Named { .. })` is not a pattern that can be
/// written; the arms are only reachable by dereferencing into a second
/// `match`. The section is trailing, and the rows it contributes therefore
/// land at the end of `$list`, because a matcher cannot alternate two row
/// shapes inside one repetition without a token muncher. Every consumer
/// sorts or counts, so `$list`'s order carries nothing.
macro_rules! tags {
    ($fn_name:ident, $list:ident, $ty:ty, { $($pat:pat => ($name:literal, $owner:expr)),+ $(,)? }) => {
        pub(crate) fn $fn_name(k: &$ty) -> (&'static str, Owner) {
            match k {
                $($pat => ($name, $owner)),+
            }
        }
        pub(crate) const $list: &[(&str, Owner)] = &[$(($name, $owner)),+];
    };
    ($fn_name:ident, $list:ident, $ty:ty,
     { $($pat:pat => ($name:literal, $owner:expr)),+ $(,)? },
     split $outer:pat in ($inner:expr) {
         $($arm:pat => ($arm_name:literal, $arm_owner:expr)),+ $(,)?
     }) => {
        pub(crate) fn $fn_name(k: &$ty) -> (&'static str, Owner) {
            match k {
                $($pat => ($name, $owner),)+
                $outer => match $inner {
                    $($arm => ($arm_name, $arm_owner)),+
                },
            }
        }
        pub(crate) const $list: &[(&str, Owner)] =
            &[$(($name, $owner),)+ $(($arm_name, $arm_owner)),+];
    };
}

tags!(instruction_tag, INSTRUCTION_TAGS, InstructionKind, {
    // ---- implemented here ----
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
    InstructionKind::Interpret { .. } => ("Interpret", Owner::InScope),
    InstructionKind::Return { .. } => ("Return", Owner::InScope),
    // `PROCEDURE` isolates the callee's pool and aliases the exposed names;
    // `USE ARG`/`USE STRICT ARG` bind the call's arguments. `USE LOCAL` can
    // only ever fail here, since this crate has no method invocations -- but
    // it fails with the oracle's own 98.993/99.910, measured, which is an
    // implemented instruction answering the right bytes and not a gap.
    InstructionKind::Procedure { .. } => ("Procedure", Owner::InScope),
    InstructionKind::Use(_) => ("Use", Owner::InScope),
    // Whole rather than arm-grained the way `Call` is, both of them. All
    // three `Signal` arms are in scope, so nothing is left inside the
    // variant to split out.
    // `Raise` is whole for a subtler reason worth stating, since it is the
    // shape that would justify an arm-grained entry: `RAISE ... ADDITIONAL
    // (a, b)` does still fail loudly, but through `ExprKind::List`'s own
    // `Phase 5` owner -- the parenthesised list is an *expression* this
    // crate cannot evaluate, reported against that expression, and `RAISE
    // ... ARRAY (a, b)` reaches the identical oracle bytes with no gap at
    // all (measured, the two spellings' reports are byte-identical). So
    // there is no `RAISE` shape whose gap belongs to `RAISE`.
    InstructionKind::Signal(_) => ("Signal", Owner::InScope),
    InstructionKind::Raise(_) => ("Raise", Owner::InScope),
    // Both whole: `queue.rs` stores every line either writes and neither has
    // a shape this crate cannot express, so unlike `Call` there is nothing
    // left to split out.
    InstructionKind::Push { .. } => ("Push", Owner::InScope),
    InstructionKind::Queue { .. } => ("Queue", Owner::InScope),
    // ---- 4c's ----
    InstructionKind::Parse(_) => ("Parse", Owner::Phase("4c")),
    InstructionKind::Arg(_) => ("Arg", Owner::Phase("4c")),
    InstructionKind::Pull(_) => ("Pull", Owner::Phase("4c")),
    InstructionKind::Address(_) => ("Address", Owner::Phase("4c")),
    // ---- Phase 5's ----
    InstructionKind::Expose { .. } => ("Expose", Owner::Phase("Phase 5")),
    InstructionKind::Options { .. } => ("Options", Owner::Phase("Phase 5")),
    InstructionKind::Message { .. } => ("Message", Owner::Phase("Phase 5")),
    InstructionKind::Guard(_) => ("Guard", Owner::Phase("Phase 5")),
    InstructionKind::Reply { .. } => ("Reply", Owner::Phase("Phase 5")),
    InstructionKind::Forward(_) => ("Forward", Owner::Phase("Phase 5")),
},
// ---- `CALL`, arm-grained, because its arms do not share one owner ----
// The only variant in this table that needs the `split` form, and the
// language is what forces it rather than a preference: a qualified call
// resolves a public routine of a named namespace, which needs the object
// model, so `Call`'s arms cannot all land in one phase however the rest of
// `CALL` is implemented.
//
// This grain is what lets `src/lib.rs`'s `instruction_owner` -- which has
// always had to split `Call`, for the same reason -- be compared against
// this table row for row, with nothing hand-maintained in between.
// `loud.rs`'s `every_out_of_scope_variant_fails_loudly` is that comparison.
split InstructionKind::Call(c) in (&**c) {
    rexx_parse::Call::Named { .. } => ("Call::Named", Owner::InScope),
    rexx_parse::Call::Dynamic { .. } => ("Call::Dynamic", Owner::InScope),
    rexx_parse::Call::Trap(_) => ("Call::Trap", Owner::InScope),
    // `CALL ns:name`, mirroring `ExprKind::QualifiedCall`'s own ownership.
    rexx_parse::Call::Qualified { .. } => ("Call::Qualified", Owner::Phase("Phase 5")),
});

tags!(expr_tag, EXPR_TAGS, ExprKind, {
    // ---- implemented here ----
    ExprKind::Literal(_) => ("Literal", Owner::InScope),
    ExprKind::Constant(_) => ("Constant", Owner::InScope),
    ExprKind::Variable(_) => ("Variable", Owner::InScope),
    ExprKind::Stem(_) => ("Stem", Owner::InScope),
    ExprKind::Compound(_) => ("Compound", Owner::InScope),
    ExprKind::DotVariable(_) => ("DotVariable", Owner::InScope),
    ExprKind::Prefix { .. } => ("Prefix", Owner::InScope),
    ExprKind::Binary { .. } => ("Binary", Owner::InScope),
    ExprKind::Logical(_) => ("Logical", Owner::InScope),
    // `ExprKind::Call`'s own `CallTarget` has exactly two forms and this crate
    // evaluates both, so unlike `InstructionKind::Call` there is no
    // later-phase arm left hiding inside it -- see `eval_call`'s own doc
    // (`eval.rs`) for the resolution order a name still falls through to the
    // loud fallback for.
    ExprKind::Call { .. } => ("Call", Owner::InScope),
    // `>x`/`<x` evaluates to the referenced variable's value in every ordinary
    // position (measured, `say >p` prints `p`'s value), and its load-bearing
    // use is as the argument half of `USE ARG >name`, which `run.rs`'s
    // `eval_argument` handles at the call site.
    ExprKind::VariableReference(_) => ("VariableReference", Owner::InScope),
    // ---- the ones that still fail loudly; see coverage.rs's module doc's ownership section ----
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
    // `DO WITH ... OVER` sends SUPPLIER, which nothing in this crate answers.
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
///
/// `seen` is `pub(crate)` rather than accessed only through a method:
/// `coverage.rs`'s own corpus walk inserts into it directly (`instructions.
/// seen.insert(...)`), from the parent module this struct is now shared
/// into rather than defined in, and `pub(crate)` is exactly as wide as that
/// access already was when this struct lived inside `coverage.rs` itself.
pub(crate) struct Coverage {
    pub(crate) category: &'static str,
    pub(crate) all: &'static [(&'static str, Owner)],
    pub(crate) seen: HashSet<&'static str>,
}

impl Coverage {
    pub(crate) fn new(category: &'static str, all: &'static [(&'static str, Owner)]) -> Self {
        Coverage {
            category,
            all,
            seen: HashSet::new(),
        }
    }

    /// In-scope variants with no witness -- the failure criterion 1
    /// (`coverage.rs`) exists to catch.
    pub(crate) fn unwitnessed(&self) -> Vec<&'static str> {
        self.all
            .iter()
            .filter(|(name, owner)| *owner == Owner::InScope && !self.seen.contains(name))
            .map(|(name, _)| *name)
            .collect()
    }

    /// `(category, tag, phase)` for every out-of-scope variant, used to build
    /// the set pinned against [`EXPECTED_OUT_OF_SCOPE`].
    pub(crate) fn out_of_scope(&self) -> Vec<(&'static str, &'static str, &'static str)> {
        self.all
            .iter()
            .filter_map(|(name, owner)| match owner {
                Owner::Phase(p) => Some((self.category, *name, *p)),
                Owner::InScope | Owner::Unreachable => None,
            })
            .collect()
    }
}

/// The out-of-scope variant set this file's `tags!` tables are allowed to
/// produce, as a literal rather than "whatever the tables say" -- the same
/// device `phase-4-exclusions.txt` uses for the builtin set. Any edit to an
/// owner arm above that is not also made here is a test failure, which is
/// the point: relabelling a variant is a plan amendment, not a drive-by
/// `match` edit.
///
/// **Pinned literal 1 of 5** this file's own module doc (below, "What is
/// pinned here") tracks for Step 5's own purposes.
pub(crate) const EXPECTED_OUT_OF_SCOPE: &[(&str, &str, &str)] = &[
    ("InstructionKind", "Command", "Phase 7"),
    // The one arm-grained row: `CALL`'s other three arms are in scope, so
    // they appear in `INSTRUCTION_TAGS` and not here.
    ("InstructionKind", "Call::Qualified", "Phase 5"),
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
    ("ExprKind", "QualifiedCall", "Phase 5"),
    ("ExprKind", "ClassResolver", "Phase 5"),
    ("ExprKind", "List", "Phase 5"),
    ("ExprKind", "Message", "Phase 5"),
    ("LoopKind", "With", "Phase 5"),
];

/// Every phase name the split table names, spelled exactly as it spells them.
/// `docs/superpowers/specs/2026-07-30-phase-4a-executor-design.md`, "The
/// split" table and its "assigned elsewhere" paragraph.
///
/// **A name stays here because the split table names it, not because some row
/// above currently uses it.** This is the set of spellings an owner is allowed
/// to have; a phase that owes nothing right now can owe something again if a
/// later audit reassigns a variant, and dropping its name would turn that
/// reassignment into a test failure with no defect behind it.
pub(crate) const SPLIT_TABLE_PHASES: &[&str] = &["4b", "4c", "Phase 5", "Phase 7"];

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
    // The design spec's criterion 1 counts, re-derived here rather than
    // trusted. These are the *implemented* counts: a task that moves a
    // variant across the line edits both the column it left and the column
    // it joined, and this test is what makes that a pair rather than a
    // choice.
    //
    // **`INSTRUCTION_TAGS` is one row per separately owned unit, not one per
    // variant**: a variant, or an arm where a variant is split. So its
    // length exceeds `InstructionKind`'s 40 variants by the 3 extra rows
    // `Call`'s `split` section contributes. `only_backslash_is_unreachable`
    // and `out_of_scope_set_matches_the_committed_expectation` are what
    // police the rows themselves; these are the totals.
    //
    // "In scope" means this crate answers the same bytes the oracle answers,
    // not that every spelling of the keyword runs. `USE LOCAL` can only ever
    // fail here, because this crate has no method invocations for it to be
    // legal in -- but it fails with the oracle's own 98.993/99.910,
    // measured, which is the same distinction `Procedure` draws for a
    // misplaced `PROCEDURE`: error 17.1, and not a gap.
    assert_eq!(INSTRUCTION_TAGS.len(), 43);
    assert_eq!(
        INSTRUCTION_TAGS
            .iter()
            .filter(|(_, o)| *o == Owner::InScope)
            .count(),
        31
    );
    assert_eq!(
        INSTRUCTION_TAGS
            .iter()
            .filter(|(_, o)| *o == Owner::Phase("4b"))
            .count(),
        0
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
        7
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
        11
    );
    assert_eq!(
        EXPR_TAGS
            .iter()
            .filter(|(_, o)| matches!(o, Owner::Phase(_)))
            .count(),
        4
    );

    assert_eq!(LOOP_TAGS.len(), 6);
    assert_eq!(PREFIX_OP_TAGS.len(), 3);
    assert_eq!(END_STYLE_TAGS.len(), 6);
    assert_eq!(TRACE_TAGS.len(), 4);
    assert_eq!(OPERATOR_TAGS.len(), 32);
}

/// The regression guard for item I36 itself: `coverage.rs` and `loud.rs`
/// must both include *this exact file*, not a private copy that happens to
/// agree with it. Checked at the source level, not the value level,
/// because the two consumers compile into two separate test binaries with
/// no way for one process to inspect another's constants at run time --
/// `#[path = "owners.rs"] mod owners;`, appearing verbatim in both files'
/// own source text, is what makes divergence structurally impossible rather
/// than merely absent right now, and this test is what would catch a future
/// edit that quietly reverted one of the two back to a hand-copied table.
#[test]
fn the_two_harnesses_include_this_exact_file() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    const NEEDLE: &str = "#[path = \"owners.rs\"]";
    for consumer in ["coverage.rs", "loud.rs"] {
        let path = dir.join(consumer);
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
        assert!(
            text.contains(NEEDLE),
            "{} does not contain {NEEDLE:?} -- it must `#[path]`-include this \
             exact file rather than hand-maintaining its own owner table \
             (item I36)",
            path.display()
        );
    }
}

// ---------------------------------------------------------------------------
// What is pinned here: any task moving an `InstructionKind`,
// `ExprKind` or `LoopKind` variant into scope (or otherwise changing
// which phase owns it) must update every one of the five items below in the
// same change, or one of the tests above (or in `coverage.rs`/`loud.rs`)
// fails.
//
// 1. **`EXPECTED_OUT_OF_SCOPE`**, above: the pinned `(category, tag, phase)`
//    set every out-of-scope variant must appear in exactly once. Checked by
//    `out_of_scope_set_matches_the_committed_expectation`.
// 2. **`coverage.rs`'s `EXPECTED_SUBSET`**: the exact line list of
//    `phase-4a.txt`, checked by that file's own
//    `phase_4a_subset_matches_the_committed_list`. Unrelated to variant
//    ownership directly, but a task that widens the L0 subset (adding a
//    program) has to extend this list in the same change, or the test
//    fails on the new, unlisted line.
// 3. **This file's own [`variant_counts_match_the_audited_split`]**: the
//    hardcoded `InstructionKind` phase counts and the two `ExprKind` ones.
//    A variant moving in scope changes the `InScope` count and whichever
//    phase count it left, and both sides of that move must be edited
//    together. `loud.rs`'s own `in_scope_counts_match_the_audited_split`
//    carries a copy of the `InScope` figure and moves with it.
// 4. **`loud.rs`'s `INSTRUCTION_WITNESSES`/`EXPR_WITNESSES`**: one witness
//    row per phase-owned row of this file's tables, at this file's grain --
//    so `Call`'s witness names an arm, because the `split` section gives
//    each arm its own row here. The moment a variant or arm moves in scope,
//    its witness must be *deleted*, not merely left stale, or
//    `assert_witness_set_is_complete` fails the other way (an extra witness
//    with no matching phase-owned tag).
// 5. **`src/lib.rs`'s `instruction_owner`/`expr_owner`**: the third copy of
//    this same ownership data, separate by construction because production
//    code cannot depend on anything under `tests/`. **Separate is no longer
//    unchecked.** A variant moving in scope, or changing owner, must move
//    there too, and `loud.rs`'s `every_out_of_scope_variant_fails_loudly`
//    is what fails if it does not: the owner it requires the emitted
//    message to end with is read straight out of the tables above, so the
//    two are compared to each other rather than each to a copy. That
//    comparison covers the phase-owned rows, which are the reachable ones.
//    An owner written onto a variant this crate implements is data no
//    execution path reads, and **nothing covers it** -- measured, giving
//    `InstructionKind::Say` an owner leaves the whole suite green. It needs
//    no cover, being unreachable, but an edit there should not expect any.
//    `lib.rs`'s own doc has the one exception, `Do`/`Loop`, and names the
//    four `run.rs` tests that pin it.
