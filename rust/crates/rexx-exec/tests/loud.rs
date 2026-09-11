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

//! The 4a exit gate's criterion 5: every `InstructionKind` and `ExprKind`
//! variant either belongs to the set this crate implements (`owners.rs`), or a
//! program constructing it
//! exits [`NOT_IMPLEMENTED_EXIT`] rather than succeeding, crashing, or
//! producing a plausible Rexx condition. "An implementation gap must never be
//! able to produce a passing test" is the design spec's own statement of what
//! this closes (`2026-07-30-phase-4a-executor-design.md`, "Failing loudly").

use std::path::Path;

use rexx_exec::{NOT_IMPLEMENTED_EXIT, run_program};
use rexx_parse::{ExprKind, InstructionKind, parse_program};

#[path = "owners.rs"]
mod owners;
use owners::{EXPR_TAGS, INSTRUCTION_TAGS, Owner, expr_tag, instruction_tag};

/// One out-of-scope variant's witness: source text, which category owns the
/// tag it must construct, and the tag itself. Checked against the parsed AST
/// before it is ever run, so a snippet that silently parses into the wrong
/// shape cannot pass by accident.
struct Witness {
    tag: &'static str,
    source: &'static str,
    category: Category,
}

/// The phase `owners.rs` records for `witness`'s own tag.
fn table_owner(witness: &Witness) -> &'static str {
    let table = match witness.category {
        Category::Instruction => INSTRUCTION_TAGS,
        Category::Expr => EXPR_TAGS,
    };
    let (_, owner) = table
        .iter()
        .find(|(name, _)| *name == witness.tag)
        .unwrap_or_else(|| {
            panic!(
                "witness {:?} names no row in owners.rs's table for {:?}",
                witness.tag, witness.category
            )
        });
    match owner {
        Owner::Phase(p) => p,
        Owner::InScope | Owner::Unreachable => panic!(
            "witness {:?} is owned by {owner:?} in owners.rs, so it must not \
             have a witness row here at all -- a row asserting a loud failure \
             for something this crate implements would fail for the right \
             reason and the wrong cause",
            witness.tag
        ),
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum Category {
    Instruction,
    /// **Unconstructed, because no `ExprKind` variant is out of scope.**
    /// `#[expect]` rather than `#[allow]` so that the attribute itself goes red
    /// the moment a task re-owns one and adds the `EXPR_WITNESSES` row that
    /// `assert_witness_set_is_complete` would then demand.
    #[expect(dead_code, reason = "no ExprKind variant is phase-owned today")]
    Expr,
}

/// One witness per phase-owned row of `owners.rs`'s `INSTRUCTION_TAGS`.
const INSTRUCTION_WITNESSES: &[Witness] = &[
    Witness {
        tag: "Command",
        // A clause that is only an expression, and not any other shape, is
        // dispatched as a command through the current ADDRESS.
        source: "'date'\n",
        category: Category::Instruction,
    },
    // The one arm-grained tag; see the module doc.
    // Which variants need a row here is `owners.rs`'s to say, and the
    // assertion below reads it: a variant this crate implements must not
    // carry one, because the row would assert a loud failure that does not
    // happen.
    Witness {
        tag: "Address::Command",
        source: "address cmd ''\n",
        category: Category::Instruction,
    },
    Witness {
        tag: "Options",
        source: "options 'x'\n",
        category: Category::Instruction,
    },
];

/// Every out-of-scope `ExprKind`, one witness each, every one wrapped in
/// `SAY`, which is implemented, so the wrapper is never itself the gap -- see
/// the module doc's note on `VariableReference`.
const EXPR_WITNESSES: &[Witness] = &[];

/// Confirms `path`'s program actually constructs `witness.tag` in the
/// category it claims, before running it. A snippet that parses into the
/// wrong shape would otherwise let a passing exit-code check mean nothing.
fn assert_constructs(witness: &Witness) {
    let program = parse_program(witness.source.as_bytes().to_vec())
        .unwrap_or_else(|e| panic!("witness for {} failed to parse: {e:?}", witness.tag));
    let found = match witness.category {
        Category::Instruction => program
            .main
            .instructions
            .iter()
            .any(|i| instruction_tag(&i.kind).0 == witness.tag),
        Category::Expr => {
            let mut found = false;
            for i in &program.main.instructions {
                walk_exprs(&i.kind, &mut |e| {
                    if expr_tag(&e.kind).0 == witness.tag {
                        found = true;
                    }
                });
            }
            found
        }
    };
    assert!(
        found,
        "witness for {} did not construct that variant: {:?}",
        witness.tag, witness.source
    );
}

/// A small, non-exhaustive expression walk sufficient for this file's own
/// witnesses: it only needs to find one target node inside a `SAY` or a
/// `CALL`'s arguments, not every position every instruction can hold one in.
/// `coverage.rs` has the exhaustive version this file does not need.
fn walk_exprs<'a>(kind: &'a InstructionKind, f: &mut impl FnMut(&'a rexx_parse::Expr)) {
    fn walk<'a>(e: &'a rexx_parse::Expr, f: &mut impl FnMut(&'a rexx_parse::Expr)) {
        f(e);
        match &e.kind {
            ExprKind::Prefix { operand, .. } => walk(operand, f),
            ExprKind::Binary { left, right, .. } => {
                walk(left, f);
                walk(right, f);
            }
            ExprKind::Call { args, .. } | ExprKind::QualifiedCall { args, .. } => {
                for a in args.iter().flatten() {
                    walk(a, f);
                }
            }
            ExprKind::Message { target, args, .. } => {
                walk(target, f);
                for a in args.iter().flatten() {
                    walk(a, f);
                }
            }
            ExprKind::List(items) => {
                for i in items.iter().flatten() {
                    walk(i, f);
                }
            }
            ExprKind::Logical(items) => {
                for i in items {
                    walk(i, f);
                }
            }
            ExprKind::VariableReference(inner) => walk(inner, f),
            ExprKind::Literal(_)
            | ExprKind::Constant(_)
            | ExprKind::Variable(_)
            | ExprKind::Stem(_)
            | ExprKind::Compound(_)
            | ExprKind::DotVariable(_)
            | ExprKind::ClassResolver { .. } => {}
        }
    }
    match kind {
        InstructionKind::Say {
            expression: Some(e),
        } => walk(e, f),
        InstructionKind::Call(c) => {
            if let rexx_parse::Call::Named { args, .. } = &**c {
                for a in args.iter().flatten() {
                    walk(a, f);
                }
            }
        }
        _ => {}
    }
}

#[test]
fn assert_witness_set_is_complete() {
    // A plain `.map`: `owners.rs`'s table is arm-grained, so its phase-owned
    // rows are already the tags this file's witnesses are keyed by, one for
    // one. Nothing expands, and the absence of an expansion step is what
    // makes `table_owner`'s lookup below a comparison against `owners.rs`
    // rather than against a reconciliation of it.
    let expected_instructions: Vec<&str> = INSTRUCTION_TAGS
        .iter()
        .filter(|(_, o)| matches!(o, Owner::Phase(_)))
        .map(|(name, _)| *name)
        .collect();
    let mut got_instructions: Vec<&str> = INSTRUCTION_WITNESSES.iter().map(|w| w.tag).collect();
    got_instructions.sort();
    let mut expected_sorted = expected_instructions.clone();
    expected_sorted.sort();
    assert_eq!(
        got_instructions, expected_sorted,
        "INSTRUCTION_WITNESSES must have exactly one entry per out-of-scope \
         InstructionKind variant (per arm, for Call and Address), no more \
         and no fewer"
    );
    assert_eq!(expected_instructions.len(), 3);

    let expected_exprs: Vec<&str> = EXPR_TAGS
        .iter()
        .filter(|(_, o)| matches!(o, Owner::Phase(_)))
        .map(|(name, _)| *name)
        .collect();
    let mut got_exprs: Vec<&str> = EXPR_WITNESSES.iter().map(|w| w.tag).collect();
    got_exprs.sort();
    let mut expected_exprs_sorted = expected_exprs.clone();
    expected_exprs_sorted.sort();
    assert_eq!(
        got_exprs, expected_exprs_sorted,
        "EXPR_WITNESSES must have exactly one entry per out-of-scope ExprKind \
         variant, no more and no fewer"
    );
    assert_eq!(expected_exprs.len(), 0);
}

#[test]
fn in_scope_counts_match_the_audited_split() {
    // The in-scope halves of `owners.rs`'s two tables, so that an in-scope
    // variant cannot go unlisted by omission. `owners.rs`'s own
    // `variant_counts_match_the_audited_split` carries the full split these
    // two figures are a part of; the rows themselves, including `Call`'s
    // arm-grained three, are policed there.
    assert_eq!(
        INSTRUCTION_TAGS
            .iter()
            .filter(|(_, o)| *o == Owner::InScope)
            .count(),
        41
    );
    assert_eq!(
        EXPR_TAGS
            .iter()
            .filter(|(_, o)| *o == Owner::InScope)
            .count(),
        15
    );
}

#[test]
fn every_out_of_scope_variant_fails_loudly() {
    let mut failures = String::new();
    for witness in INSTRUCTION_WITNESSES.iter().chain(EXPR_WITNESSES.iter()) {
        // This loop does not check the owner against `SPLIT_TABLE_PHASES`,
        // because `owners.rs`'s own
        // `assert_owner_strings_are_split_table_phases` holds every row of
        // every one of its seven tables to that set -- including these, and
        // including the rows this file never asks about.
        let owner = table_owner(witness);
        assert_constructs(witness);

        let outcome = run_program(
            Path::new("/tmp/loud-witness.rex").to_str().unwrap(),
            witness.source.as_bytes().to_vec(),
            rexx_exec::Invocation::none(),
        );
        if outcome.exit_code != NOT_IMPLEMENTED_EXIT {
            use std::fmt::Write as _;
            writeln!(
                failures,
                "{} ({}): expected exit {NOT_IMPLEMENTED_EXIT}, got {} \
                 (stdout {:?}, stderr {:?})",
                witness.tag,
                owner,
                outcome.exit_code,
                String::from_utf8_lossy(&outcome.stdout),
                String::from_utf8_lossy(&outcome.stderr)
            )
            .unwrap();
            continue;
        }
        // **This line is the equality between `src/lib.rs`'s
        // `instruction_owner`/`expr_owner` and `owners.rs`'s tables.**
        // `owner` came out of `owners.rs`; the suffix came out of the
        // running executor, which built it from `lib.rs`'s own answer. The
        // third copy of the ownership data is therefore checked against the
        // first on every run, per row, with no hand-maintained table
        // between them -- which is what makes `lib.rs`'s match a derived
        // fact rather than one more place to remember to edit.
        let stderr = String::from_utf8_lossy(&outcome.stderr);
        let want_suffix = format!(" is not implemented ({owner})");
        if !stderr.trim_end().ends_with(&want_suffix) {
            use std::fmt::Write as _;
            writeln!(
                failures,
                "{} ({}): stderr does not end with {want_suffix:?}: {stderr:?}",
                witness.tag, owner
            )
            .unwrap();
        }
    }
    assert!(
        failures.is_empty(),
        "an out-of-scope variant did not fail loudly, or failed loudly \
         without naming its owner -- an implementation gap must never be \
         able to produce a passing test, and a loud message that does not \
         name who owns it is exactly the property Step 3 exists to add:\n{failures}"
    );
}
