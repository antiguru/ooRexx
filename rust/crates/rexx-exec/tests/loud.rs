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
//! variant either belongs to 4a's named set, or a program constructing it
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
//! contain none of them, by `phase-4a.txt`'s own header -- so this is
//! genuinely the wider surface the design spec calls out ("one criterion
//! closes a surface larger than 4a's own").
//!
//! For the in-scope variants, this file does not re-run a program: `4a
//! executes it` is already established, more thoroughly, by `corpus.rs`'s
//! byte-for-byte differential run against the oracle. Re-deriving that here
//! with a one-line snippet would be strictly weaker evidence covering the
//! same code path, not new evidence. What this file checks for the in-scope
//! side is only that the classification below -- the same `match` shape
//! `coverage.rs` uses -- has a total of 20 `InstructionKind` and 9 `ExprKind`
//! entries, so an in-scope variant cannot go unlisted by omission.
//!
//! # The owner table
//!
//! Duplicated from `coverage.rs` rather than shared through a common module:
//! an integration test file cannot `mod` another crate's `tests/` directory,
//! and this crate's own permitted-file list for this task does not include a
//! new shared module. The two tables must be kept in sync by hand; see
//! `coverage.rs`'s module doc for the full reasoning behind each owner
//! string, in particular the five `ExprKind` assignments that are a Task 16
//! gate-time judgement call rather than a spec citation, recorded in
//! `docs/superpowers/plans/phase-4-exclusions.txt`'s "EXPRKIND OWNERSHIP"
//! section.
//!
//! # Witness programs
//!
//! Each is the smallest program found that both (a) parses under
//! `rexx-parse` into the exact target variant, checked here rather than
//! assumed, and (b) reaches that instruction or expression through ordinary
//! straight-line execution with no preceding label, call or condition to
//! satisfy. Some out-of-scope instructions are conventionally written after
//! a label reached by `CALL` (`PROCEDURE`, `EXPOSE`, `GUARD`, `REPLY`,
//! `FORWARD`) -- nothing in the grammar requires that context, so each is
//! written as a bare top-level clause instead, which also avoids depending on
//! `CALL` (itself out of scope) ever succeeding.
//!
//! `ExprKind::VariableReference` (`>x`/`<x`) is the one exception: its only
//! legal position is a call or message argument list (`ast.rs`: "anything
//! else is error 20.930"), so its witness necessarily sits inside a `CALL`
//! instruction's argument list. That call is loud regardless of what its
//! arguments are, so the loudness observed is not confounded with anything
//! `VariableReference` itself would need to do differently -- both `Call`
//! (the instruction) and `VariableReference` are 4b's.

use std::path::Path;

use rexx_exec::{NOT_IMPLEMENTED_EXIT, run_program};
use rexx_parse::{ExprKind, InstructionKind, parse_program};

/// Who is responsible for a variant. Mirrors `coverage.rs`'s `Owner`; see
/// that file's module doc for why each `Phase` string is what it is.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum Owner {
    InScope,
    Phase(&'static str),
}

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
    InstructionKind::Call(_) => ("Call", Owner::Phase("4b")),
    InstructionKind::Return { .. } => ("Return", Owner::Phase("4b")),
    InstructionKind::Procedure { .. } => ("Procedure", Owner::Phase("4b")),
    InstructionKind::Use(_) => ("Use", Owner::Phase("4b")),
    InstructionKind::Signal(_) => ("Signal", Owner::Phase("4b")),
    InstructionKind::Raise(_) => ("Raise", Owner::Phase("4b")),
    InstructionKind::Interpret { .. } => ("Interpret", Owner::Phase("4b")),
    InstructionKind::Push { .. } => ("Push", Owner::Phase("4b")),
    InstructionKind::Queue { .. } => ("Queue", Owner::Phase("4b")),
    InstructionKind::Parse(_) => ("Parse", Owner::Phase("4c")),
    InstructionKind::Arg(_) => ("Arg", Owner::Phase("4c")),
    InstructionKind::Pull(_) => ("Pull", Owner::Phase("4c")),
    InstructionKind::Address(_) => ("Address", Owner::Phase("4c")),
    InstructionKind::Expose { .. } => ("Expose", Owner::Phase("Phase 5")),
    InstructionKind::Options { .. } => ("Options", Owner::Phase("Phase 5")),
    InstructionKind::Message { .. } => ("Message", Owner::Phase("Phase 5")),
    InstructionKind::Guard(_) => ("Guard", Owner::Phase("Phase 5")),
    InstructionKind::Reply { .. } => ("Reply", Owner::Phase("Phase 5")),
    InstructionKind::Forward(_) => ("Forward", Owner::Phase("Phase 5")),
});

tags!(expr_tag, EXPR_TAGS, ExprKind, {
    ExprKind::Literal(_) => ("Literal", Owner::InScope),
    ExprKind::Constant(_) => ("Constant", Owner::InScope),
    ExprKind::Variable(_) => ("Variable", Owner::InScope),
    ExprKind::Stem(_) => ("Stem", Owner::InScope),
    ExprKind::Compound(_) => ("Compound", Owner::InScope),
    ExprKind::DotVariable(_) => ("DotVariable", Owner::InScope),
    ExprKind::Prefix { .. } => ("Prefix", Owner::InScope),
    ExprKind::Binary { .. } => ("Binary", Owner::InScope),
    ExprKind::Logical(_) => ("Logical", Owner::InScope),
    ExprKind::Call { .. } => ("Call", Owner::Phase("4b")),
    ExprKind::VariableReference(_) => ("VariableReference", Owner::Phase("4b")),
    ExprKind::QualifiedCall { .. } => ("QualifiedCall", Owner::Phase("Phase 5")),
    ExprKind::ClassResolver { .. } => ("ClassResolver", Owner::Phase("Phase 5")),
    ExprKind::List(_) => ("List", Owner::Phase("Phase 5")),
    ExprKind::Message { .. } => ("Message", Owner::Phase("Phase 5")),
});

/// One out-of-scope variant's witness: source text, which category owns the
/// tag it must construct, and the tag itself. Checked against the parsed AST
/// before it is ever run, so a snippet that silently parses into the wrong
/// shape cannot pass by accident.
struct Witness {
    tag: &'static str,
    owner: &'static str,
    source: &'static str,
    category: Category,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
enum Category {
    Instruction,
    Expr,
}

/// Every out-of-scope `InstructionKind`, one witness each. 20 entries, one
/// per `Owner::Phase` arm in `INSTRUCTION_TAGS` above -- `assert_witness_set_is_complete`
/// checks the two lists against each other so a variant cannot be silently
/// dropped from this list while staying in the tag table.
const INSTRUCTION_WITNESSES: &[Witness] = &[
    Witness {
        tag: "Command",
        owner: "Phase 7",
        // A clause that is only an expression, and not any other shape, is
        // dispatched as a command through the current ADDRESS.
        source: "'date'\n",
        category: Category::Instruction,
    },
    Witness {
        tag: "Call",
        owner: "4b",
        source: "call sub\n",
        category: Category::Instruction,
    },
    Witness {
        tag: "Return",
        owner: "4b",
        source: "return 1\n",
        category: Category::Instruction,
    },
    Witness {
        tag: "Procedure",
        owner: "4b",
        // Only legal as an internal routine's first instruction, but nothing
        // stops straight-line execution from reaching it: the label above it
        // is a traced no-op and control falls through.
        source: "sub: procedure\n",
        category: Category::Instruction,
    },
    Witness {
        tag: "Use",
        owner: "4b",
        source: "use arg x\n",
        category: Category::Instruction,
    },
    Witness {
        tag: "Signal",
        owner: "4b",
        source: "signal there\nthere: nop\n",
        category: Category::Instruction,
    },
    Witness {
        tag: "Raise",
        owner: "4b",
        source: "raise syntax 40.1\n",
        category: Category::Instruction,
    },
    Witness {
        tag: "Interpret",
        owner: "4b",
        source: "interpret \"say 1\"\n",
        category: Category::Instruction,
    },
    Witness {
        tag: "Push",
        owner: "4b",
        source: "push 'x'\n",
        category: Category::Instruction,
    },
    Witness {
        tag: "Queue",
        owner: "4b",
        source: "queue 'x'\n",
        category: Category::Instruction,
    },
    Witness {
        tag: "Parse",
        owner: "4c",
        source: "parse value 'a' with v\n",
        category: Category::Instruction,
    },
    Witness {
        tag: "Arg",
        owner: "4c",
        source: "arg x\n",
        category: Category::Instruction,
    },
    Witness {
        tag: "Pull",
        owner: "4c",
        source: "pull x\n",
        category: Category::Instruction,
    },
    Witness {
        tag: "Address",
        owner: "4c",
        source: "address cmd\n",
        category: Category::Instruction,
    },
    Witness {
        tag: "Expose",
        owner: "Phase 5",
        source: "expose x\n",
        category: Category::Instruction,
    },
    Witness {
        tag: "Options",
        owner: "Phase 5",
        source: "options 'x'\n",
        category: Category::Instruction,
    },
    Witness {
        tag: "Message",
        owner: "Phase 5",
        source: "a~b\n",
        category: Category::Instruction,
    },
    Witness {
        tag: "Guard",
        owner: "Phase 5",
        source: "guard on\n",
        category: Category::Instruction,
    },
    Witness {
        tag: "Reply",
        owner: "Phase 5",
        source: "reply 5\n",
        category: Category::Instruction,
    },
    Witness {
        tag: "Forward",
        owner: "Phase 5",
        source: "forward\n",
        category: Category::Instruction,
    },
];

/// Every out-of-scope `ExprKind`, one witness each. Wrapped in `SAY` (4a's
/// own) except `VariableReference`, whose only legal position is a call
/// argument list -- see the module doc.
const EXPR_WITNESSES: &[Witness] = &[
    Witness {
        tag: "Call",
        owner: "4b",
        source: "say foo(1)\n",
        category: Category::Expr,
    },
    Witness {
        tag: "QualifiedCall",
        owner: "Phase 5",
        source: "say ns:foo(1)\n",
        category: Category::Expr,
    },
    Witness {
        tag: "ClassResolver",
        owner: "Phase 5",
        source: "say ns:Bar\n",
        category: Category::Expr,
    },
    Witness {
        tag: "Message",
        owner: "Phase 5",
        source: "say a~b\n",
        category: Category::Expr,
    },
    Witness {
        tag: "List",
        owner: "Phase 5",
        source: "say (1, 2)\n",
        category: Category::Expr,
    },
    Witness {
        tag: "VariableReference",
        owner: "4b",
        // `Call` itself is 4b's too, so the loudness this produces is not
        // confounded: nothing here depends on `VariableReference` doing
        // anything differently for the exit code to be `NOT_IMPLEMENTED_EXIT`.
        source: "call sub >x\n",
        category: Category::Expr,
    },
];

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
         InstructionKind variant, no more and no fewer"
    );
    assert_eq!(expected_instructions.len(), 20);

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
    assert_eq!(expected_exprs.len(), 6);
}

#[test]
fn in_scope_counts_match_the_audited_split() {
    assert_eq!(
        INSTRUCTION_TAGS
            .iter()
            .filter(|(_, o)| *o == Owner::InScope)
            .count(),
        20
    );
    assert_eq!(
        EXPR_TAGS
            .iter()
            .filter(|(_, o)| *o == Owner::InScope)
            .count(),
        9
    );
}

const SPLIT_TABLE_PHASES: &[&str] = &["4b", "4c", "Phase 5", "Phase 7"];

#[test]
fn every_out_of_scope_variant_fails_loudly() {
    let mut failures = String::new();
    for witness in INSTRUCTION_WITNESSES.iter().chain(EXPR_WITNESSES.iter()) {
        assert!(
            SPLIT_TABLE_PHASES.contains(&witness.owner),
            "{} names owner {:?}, which is not one of the split table's \
             phases {SPLIT_TABLE_PHASES:?}",
            witness.tag,
            witness.owner
        );
        assert_constructs(witness);

        let outcome = run_program(
            Path::new("/tmp/loud-witness.rex").to_str().unwrap(),
            witness.source.as_bytes().to_vec(),
        );
        if outcome.exit_code != NOT_IMPLEMENTED_EXIT {
            use std::fmt::Write as _;
            writeln!(
                failures,
                "{} ({}): expected exit {NOT_IMPLEMENTED_EXIT}, got {} \
                 (stdout {:?}, stderr {:?})",
                witness.tag,
                witness.owner,
                outcome.exit_code,
                String::from_utf8_lossy(&outcome.stdout),
                String::from_utf8_lossy(&outcome.stderr)
            )
            .unwrap();
        }
    }
    assert!(
        failures.is_empty(),
        "an out-of-scope variant did not fail loudly -- an implementation gap \
         must never be able to produce a passing test:\n{failures}"
    );
}
