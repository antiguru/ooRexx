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
macro_rules! tags {
    ($fn_name:ident, $list:ident, $ty:ty,
     { $($pat:pat => ($name:literal, $owner:expr)),+ $(,)? }
     $(, split $outer:pat in ($inner:expr) {
         $($arm:pat => ($arm_name:literal, $arm_owner:expr)),+ $(,)?
     })* $(,)?) => {
        pub(crate) fn $fn_name(k: &$ty) -> (&'static str, Owner) {
            match k {
                $($pat => ($name, $owner),)+
                $($outer => match $inner {
                    $($arm => ($arm_name, $arm_owner)),+
                },)*
            }
        }
        pub(crate) const $list: &[(&str, Owner)] =
            &[$(($name, $owner),)+ $($(($arm_name, $arm_owner),)+)*];
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
    // `Raise` is whole in the sense `Expose` above is. The shape that would
    // have forced an arm-grained entry is `ADDITIONAL <array>`, and it answers:
    // measured on both engines, three descriptors, `raise syntax 40.4
    // additional (1,,3)` and `... array (1,,3)` are byte-identical to each
    // other and to the oracle. What has no code is an `ADDITIONAL` value under
    // a `SYNTAX` condition that is a class object or one of the interpreter's
    // own, and that is refused through `Loud::object_position` rather than
    // through this table.
    InstructionKind::Signal(_) => ("Signal", Owner::InScope),
    InstructionKind::Raise(_) => ("Raise", Owner::InScope),
    // Both whole: `queue.rs` stores every line either writes and neither has
    // a shape this crate cannot express, so unlike `Call` there is nothing
    // left to split out.
    InstructionKind::Push { .. } => ("Push", Owner::InScope),
    InstructionKind::Queue { .. } => ("Queue", Owner::InScope),
    // The one instruction's three spellings, all in scope with every source:
    // `PARSE`, and the `ARG`/`PULL` short forms that are the same instruction
    // with `UPPER` implied. They are separate `InstructionKind` variants and so
    // separate rows, but nothing distinguishes their ownership.
    InstructionKind::Parse(_) => ("Parse", Owner::InScope),
    InstructionKind::Arg(_) => ("Arg", Owner::InScope),
    InstructionKind::Pull(_) => ("Pull", Owner::InScope),
    // A message send as a whole clause -- `q~append(1)`, `q~~append(1)` and
    // the message-assignment form `q[1] = 2`. In scope in the same sense
    // `DotVariable` is: the variant evaluates, and the named sub-cases this
    // crate has no code for (a receiver whose class is deferred, a primitive
    // method with no implementation) fail loudly rather than silently.
    InstructionKind::Message { .. } => ("Message", Owner::InScope),
    // Binds its names to the receiving object's scope pool. In scope in the
    // same sense `Message` is: the variant executes, and the sub-cases with no
    // code -- a single compound tail, a receiver that is not a class object --
    // fail loudly rather than silently.
    InstructionKind::Expose { .. } => ("Expose", Owner::InScope),
    // Reserves and releases the receiver's scope. In scope in the same sense
    // `Expose` is: the variant executes -- a reservation nothing can contend
    // for is a no-op on one activity, and the method-invocation check is the
    // oracle's own 99.911 -- and the one sub-case with no code, a `WHEN`
    // expression that is false and so has to wait, fails loudly.
    InstructionKind::Guard(_) => ("Guard", Owner::InScope),
    // Hands its value to the sender and leaves the rest of the method body
    // owed. In scope in the same sense `Guard` is: the variant executes, and
    // the sub-case with no code -- a `REPLY` under a construct whose state an
    // instruction index cannot restore -- fails loudly.
    InstructionKind::Reply { .. } => ("Reply", Owner::InScope),
    // Re-sends the message the method was entered with, to whatever `TO`,
    // `MESSAGE`, `CLASS`, `ARGUMENTS` and `ARRAY` leave of the context. In
    // scope in the same sense `Guard` is: every option executes, and the
    // sub-case with no code -- an `ARGUMENTS` value whose conversion this
    // crate does not build -- fails loudly.
    InstructionKind::Forward(_) => ("Forward", Owner::InScope),
    // ---- Phase 5's ----
    InstructionKind::Options { .. } => ("Options", Owner::Phase("Phase 5")),
},
// ---- `CALL`, still arm-grained although every arm is now in scope ----
// The four resolve by four different rules -- a label search, a run-time
// target, a namespace's public routines, and a condition trap -- so a single
// row would say less than these four do about what has been checked.
split InstructionKind::Call(c) in (&**c) {
    rexx_parse::Call::Named { .. } => ("Call::Named", Owner::InScope),
    rexx_parse::Call::Dynamic { .. } => ("Call::Dynamic", Owner::InScope),
    rexx_parse::Call::Trap(_) => ("Call::Trap", Owner::InScope),
    // `CALL ns:name`, resolved against that namespace's public routines --
    // 98.987 for a namespace nothing registered and 43.902 for a routine it
    // does not export, both the oracle's own.
    rexx_parse::Call::Qualified { .. } => ("Call::Qualified", Owner::InScope),
},
// ---- `ADDRESS`, split on the same condition `instruction_owner` uses ----
// One keyword, two jobs. `ADDRESS env`, `ADDRESS VALUE expr` and the bare
// toggle only name an environment, which is per-activation state and needs
// nothing outside this crate. `ADDRESS env command` issues a command to that
// environment, and `WITH` says where a command's three streams go; both need
// the command dispatch `InstructionKind::Command` needs, and carry that same
// owner (D18).
split InstructionKind::Address(a) in (a.command.is_some() || a.io.is_some()) {
    true => ("Address::Command", Owner::Phase("Phase 7")),
    false => ("Address::Environment", Owner::InScope),
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
    // `>x`/`<x` answers a `VariableReference`, built by `eval.rs`'s own arm
    // over `run.rs`'s `Interp::variable_reference`. Every rendering and every
    // conversion of one answers as the variable it names, which is why
    // `say >p` still prints `p`'s value.
    ExprKind::VariableReference(_) => ("VariableReference", Owner::InScope),
    // `target~name(...)`, `target~~name(...)` and `target[...]`, resolved
    // and invoked through `dispatch.rs`. See `InstructionKind::Message`
    // above for what "in scope" claims and what it does not.
    ExprKind::Message { .. } => ("Message", Owner::InScope),
    // `(a, b, ...)`, which builds an `.Array` -- `eval.rs`'s `eval_list`.
    ExprKind::List(_) => ("List", Owner::InScope),
    // `ns:name(...)` and `ns:Name`, resolved through the running package's
    // own namespace table -- `eval.rs`'s `eval_cold` arms over
    // `Interp::namespace_routine` and `Interp::namespace_class`.
    ExprKind::QualifiedCall { .. } => ("QualifiedCall", Owner::InScope),
    ExprKind::ClassResolver { .. } => ("ClassResolver", Owner::InScope),
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
pub(crate) const EXPECTED_OUT_OF_SCOPE: &[(&str, &str, &str)] = &[
    ("InstructionKind", "Command", "Phase 7"),
    // The one arm-grained row. `ADDRESS`'s other form is in scope, and so is
    // every arm of `CALL`, so those appear in `INSTRUCTION_TAGS` and not here.
    ("InstructionKind", "Address::Command", "Phase 7"),
    ("InstructionKind", "Options", "Phase 5"),
    ("LoopKind", "With", "Phase 5"),
];

/// Every phase name the split table names, spelled exactly as it spells them.
/// `docs/superpowers/specs/2026-07-30-phase-4a-executor-design.md`, "The
/// split" table and its "assigned elsewhere" paragraph.
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
    assert_eq!(INSTRUCTION_TAGS.len(), 44);
    assert_eq!(
        INSTRUCTION_TAGS
            .iter()
            .filter(|(_, o)| *o == Owner::InScope)
            .count(),
        41
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
        0
    );
    assert_eq!(
        INSTRUCTION_TAGS
            .iter()
            .filter(|(_, o)| *o == Owner::Phase("Phase 5"))
            .count(),
        1
    );
    assert_eq!(
        INSTRUCTION_TAGS
            .iter()
            .filter(|(_, o)| *o == Owner::Phase("Phase 7"))
            .count(),
        2
    );

    assert_eq!(EXPR_TAGS.len(), 15);
    assert_eq!(
        EXPR_TAGS
            .iter()
            .filter(|(_, o)| *o == Owner::InScope)
            .count(),
        15
    );
    assert_eq!(
        EXPR_TAGS
            .iter()
            .filter(|(_, o)| matches!(o, Owner::Phase(_)))
            .count(),
        0
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
