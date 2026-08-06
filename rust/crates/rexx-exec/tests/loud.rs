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
//!
//! # How this differs from `coverage.rs`
//!
//! `coverage.rs` (criterion 1) is parse-only: it proves the *subset*'s
//! programs construct every in-scope variant, and records an owner string for
//! everything else. This file *executes* one small program per out-of-scope
//! variant through [`rexx_exec::run_program`] and checks the actual exit
//! code. Nothing else in this gate runs a single out-of-scope construct
//! through the executor at all -- `tests/corpus.rs`'s subset is defined to
//! contain none of them, by its subset files' own headers -- so this is
//! genuinely the wider surface the design spec calls out ("one criterion
//! closes a surface larger than 4a's own").
//!
//! For the in-scope variants, this file does not re-run a program: `this
//! crate executes it` is already established, more thoroughly, by `corpus.rs`'s
//! byte-for-byte differential run against the oracle. Re-deriving that here
//! with a one-line snippet would be strictly weaker evidence covering the
//! same code path, not new evidence. What this file checks for the in-scope
//! side is only that `owners.rs`'s two tables hold the in-scope totals
//! [`in_scope_counts_match_the_audited_split`] asserts, so an in-scope
//! variant cannot go unlisted by omission.
//!
//! # The owner table lives in `owners.rs`
//!
//! `Owner`, the seven `*_TAGS` tables and their tag functions all live in
//! `owners.rs` now, `#[path]`-included below as `mod owners` -- `coverage.rs`
//! includes the identical file the same way, so the two can no longer
//! diverge by hand-editing only one (item I36; see `owners.rs`'s own module
//! doc). See `coverage.rs`'s module doc for the full reasoning behind each
//! owner string, in particular the five `ExprKind` assignments that are a
//! Task 16 gate-time judgement call rather than a spec citation, recorded in
//! `docs/superpowers/plans/phase-4-exclusions.txt`'s "EXPRKIND OWNERSHIP"
//! section.
//!
//! # Arm-grained ownership, and why this file no longer reconciles anything
//!
//! `InstructionKind::Call` wraps an inner enum (`rexx_parse::Call`) whose
//! arms do not share an owner: a namespace-qualified `CALL ns:name` needs
//! the object model and is Phase 5's, mirroring `ExprKind::QualifiedCall`,
//! while the other three arms are this crate's. `owners.rs`'s
//! `INSTRUCTION_TAGS` therefore gives each arm its own row, through that
//! file's `tags!` `split` section, and `owners::instruction_tag` answers
//! `"Call::Qualified"` rather than `"Call"` for the one that is still loud.
//!
//! **Everything below is keyed by exactly the tags that table produces.**
//! There is no expansion step, no second grain and no owner string written
//! down here: [`table_owner`] reads the owner out of `owners.rs`, and
//! [`assert_witness_set_is_complete`] holds this file's witness tags equal
//! to that table's phase-owned rows as literal sets. That is what lets
//! [`every_out_of_scope_variant_fails_loudly`] compare `src/lib.rs`'s
//! `instruction_owner`/`expr_owner` against `owners.rs` directly, rather
//! than against a hand-maintained reconciliation of it -- which would only
//! move the duplication into the reconciler.
//!
//! # Witness programs
//!
//! Each is the smallest program found that both (a) parses under
//! `rexx-parse` into the exact target variant, checked here rather than
//! assumed, and (b) reaches that instruction or expression through ordinary
//! straight-line execution with no preceding label, call or condition to
//! satisfy. Some out-of-scope instructions are conventionally written after
//! a label reached by `CALL` (`EXPOSE`, `GUARD`, `REPLY`, `FORWARD`) --
//! nothing in the grammar requires that context, so each is written as a
//! bare top-level clause instead, which also avoids depending on `CALL`
//! (itself out of scope) ever succeeding.
//!
//! **`ExprKind::VariableReference` (`>x`/`<x`) has no row, and the reason is
//! worth keeping because it was got wrong once:** `ast.rs`'s 20.930 is about
//! which *token* may follow `>`/`<` (a variable or a stem, not a literal or a
//! number, `expr.rs`'s own `parseVariableReferenceTerm` doc), **not** about
//! which instruction context the whole reference may sit in -- so `say >x` is
//! legal on its own and needs no `CALL` around it, and it prints the
//! referenced variable's value, measured against the oracle. One position
//! where `>` is *not* a reference, also measured: `say 'text' >x` is a
//! comparison, because a `>` following a complete term is the operator.

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
///
/// **No owner field.** A witness names a tag and `owners.rs` says who owns
/// it; see [`table_owner`].
struct Witness {
    tag: &'static str,
    source: &'static str,
    category: Category,
}

/// The phase `owners.rs` records for `witness`'s own tag.
///
/// This is the only place an owner string enters this file, and that is the
/// point rather than tidiness: the string it returns is what
/// [`every_out_of_scope_variant_fails_loudly`] then requires `src/lib.rs` to
/// have emitted, so the two tables are compared to each other rather than
/// each to a copy kept here. A witness that named its own owner could agree
/// with `lib.rs` while both disagreed with `owners.rs`, and nothing would
/// say so.
///
/// Panics rather than returns an `Option` for a tag the table does not
/// carry, or carries as in-scope: both mean this file and `owners.rs`
/// disagree about what is loud, which is a harness defect and not a result.
/// [`assert_witness_set_is_complete`] is what makes that unreachable in
/// practice, by pinning the two tag sets equal.
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
    Expr,
}

/// One witness per phase-owned row of `owners.rs`'s `INSTRUCTION_TAGS`.
///
/// **The count is not written here**, in either grain, because
/// [`assert_witness_set_is_complete`] asserts this list's tag set equal to
/// that table's phase-owned rows and a number beside it would be a second,
/// unchecked statement of the same thing. It went stale three times as
/// exactly that. The rule the assertion enforces in both directions: a
/// variant that moves in scope must have its row *deleted* here, and a row
/// here with no phase-owned tag fails just as loudly as a missing one.
const INSTRUCTION_WITNESSES: &[Witness] = &[
    Witness {
        tag: "Command",
        // A clause that is only an expression, and not any other shape, is
        // dispatched as a command through the current ADDRESS.
        source: "'date'\n",
        category: Category::Instruction,
    },
    // The one arm-grained tag; see the module doc.
    Witness {
        tag: "Call::Qualified",
        // `CALL ns:name args`, restricted to public routines of that
        // namespace.
        source: "call ns:sub\n",
        category: Category::Instruction,
    },
    // Which variants need a row here is `owners.rs`'s to say, and the
    // assertion below reads it: a variant this crate implements must not
    // carry one, because the row would assert a loud failure that does not
    // happen.
    Witness {
        tag: "Address",
        source: "address cmd\n",
        category: Category::Instruction,
    },
    Witness {
        tag: "Expose",
        source: "expose x\n",
        category: Category::Instruction,
    },
    Witness {
        tag: "Options",
        source: "options 'x'\n",
        category: Category::Instruction,
    },
    Witness {
        tag: "Message",
        source: "a~b\n",
        category: Category::Instruction,
    },
    Witness {
        tag: "Guard",
        source: "guard on\n",
        category: Category::Instruction,
    },
    Witness {
        tag: "Reply",
        source: "reply 5\n",
        category: Category::Instruction,
    },
    Witness {
        tag: "Forward",
        source: "forward\n",
        category: Category::Instruction,
    },
];

/// Every out-of-scope `ExprKind`, one witness each, every one wrapped in
/// `SAY`, which is implemented, so the wrapper is never itself the gap -- see
/// the module doc's note on `VariableReference`.
const EXPR_WITNESSES: &[Witness] = &[
    Witness {
        tag: "QualifiedCall",
        source: "say ns:foo(1)\n",
        category: Category::Expr,
    },
    Witness {
        tag: "ClassResolver",
        source: "say ns:Bar\n",
        category: Category::Expr,
    },
    Witness {
        tag: "Message",
        source: "say a~b\n",
        category: Category::Expr,
    },
    Witness {
        tag: "List",
        source: "say (1, 2)\n",
        category: Category::Expr,
    },
];

/// Confirms `path`'s program actually constructs `witness.tag` in the
/// category it claims, before running it. A snippet that parses into the
/// wrong shape would otherwise let a passing exit-code check mean nothing.
///
/// `owners::instruction_tag` needs no help telling a `Call` arm from its
/// variant: that table is arm-grained where `lib.rs` is, so the tag it
/// answers for a parsed node is already the tag a witness names.
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
         InstructionKind variant (per arm, for Call), no more and no \
         fewer"
    );
    assert_eq!(expected_instructions.len(), 9);

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
    assert_eq!(expected_exprs.len(), 4);
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
        34
    );
    assert_eq!(
        EXPR_TAGS
            .iter()
            .filter(|(_, o)| *o == Owner::InScope)
            .count(),
        11
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
        //
        // What it does **not** cover: the `None` arms. `Loud::instruction`
        // is only reached for a variant the executor declines, so an owner
        // wrongly written as `None` for something loud shows up here, while
        // a phase wrongly written *onto* an implemented variant is data no
        // path reads. **Nothing covers that, and this comment does not
        // point anywhere claiming otherwise**, because a disclaimer naming
        // a test that is not in fact watching is worse than none -- it
        // stops the next reader looking. Measured: giving
        // `InstructionKind::Say` an owner leaves the whole workspace suite
        // green. The one exception is `Do`/`Loop`, which `run_loop` does
        // reach here, and whose four `run.rs` tests on the exact unsuffixed
        // message go red; `lib.rs`'s `instruction_owner` names them.
        //
        // **Pins the exact trailing shape, not merely the owner's presence
        // (review finding I2).** An earlier version checked
        // `stderr.contains(..)`, which a message reading `[4b] CALL:
        // unimplemented` also satisfies -- `contains` cannot tell "names the
        // owner in the documented shape" from "mentions the owner's bytes
        // somewhere". `ends_with` on the exact suffix `owned_message`
        // (`lib.rs`) produces is what actually pins the shape later tasks
        // are told to rely on (`" is not implemented (OWNER)"`, `trim_end`
        // first since every message ends in `\n`).
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
