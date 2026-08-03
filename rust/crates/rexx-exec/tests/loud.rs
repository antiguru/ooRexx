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
//! # Arm-grained ownership (Task 0's Step 2)
//!
//! `InstructionKind::Call` and `InstructionKind::Signal` are each one row in
//! `owners.rs`'s `INSTRUCTION_TAGS` (`"Call"` -> `"4b"`, `"Signal"` ->
//! `"4b"`), which is right for criterion 1's parse-only coverage question
//! ("was some `Call`/`Signal` constructed") but wrong for this file's own
//! question, because both wrap an inner enum (`rexx_parse::Call`,
//! `rexx_parse::Signal`) whose *arms* do not all become in-scope, or even
//! all belong to the same owner, at the same moment: `Call::Named` and
//! `Call::Dynamic` are both in scope from Task 3; `Call::Qualified` is
//! genuinely Phase 5's (a namespace-qualified `CALL`, mirroring `ExprKind::
//! QualifiedCall`'s own ownership); `Call::Trap` (`CALL ON`/`CALL OFF`) and
//! `Signal::Trap` (`SIGNAL ON`/`SIGNAL OFF`) are not in scope until Task 7,
//! after `Signal::Value`/`Signal::Label` already are (Task 6). A single
//! `"Call"`/`"Signal"` witness row cannot survive that: the moment Task 3
//! implements `Call::Named`, the witness naming it has to be deleted, and a
//! single shared row would take `Call::Qualified` and `Call::Trap`'s own
//! loudness coverage down with it, with nothing left to catch that `CALL
//! ON`/`CALL OFF` and `CALL ns:name` are still unimplemented.
//!
//! So `INSTRUCTION_WITNESSES` below carries **one row per arm** for the two
//! split variants -- `"Call::Named"`, `"Call::Dynamic"`, `"Call::Qualified"`,
//! `"Call::Trap"`, and `"Signal"` (still combined, for `Value`/`Label`
//! together, since both move in scope in the same task) plus `"Signal::
//! Trap"` -- rather than the coarse `"Call"`/`"Signal"` `assert_constructs`
//! would otherwise check against. [`instruction_arm`] is what resolves a
//! witness's tag against the program it parses: the coarse `owners::
//! instruction_tag` for every variant except these two, and the inner arm's
//! own name for them. [`assert_witness_set_is_complete`]'s own expected set
//! is built the same way, through [`expand_for_witnesses`] -- a hand-
//! maintained expansion of `owners.rs`'s coarse tag list, the same "pin it
//! as a literal, not as whatever the code currently computes" device
//! `EXPECTED_OUT_OF_SCOPE` already uses one level up, because nothing here
//! can enumerate `rexx_parse::Call`'s/`Signal`'s own arms at compile time.
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
//! **`ExprKind::VariableReference` (`>x`/`<x`) needs no `CALL` at all --
//! `say >x` reaches the arm directly.** An earlier version of this
//! paragraph claimed the opposite (that a call or message argument list is
//! `VariableReference`'s only legal position, citing `ast.rs`'s 20.930),
//! and that was wrong: 20.930 is about which *token* may follow `>`/`<`
//! (a variable or a stem, not a literal or a number, `expr.rs`'s own
//! `parseVariableReferenceTerm` doc), not about which instruction context
//! the whole reference may sit in. `eval.rs`'s own module doc already says
//! `VariableReference` fails loudly on *any* evaluation. Measured: `say
//! >x\n` gives `rexx-exec: a variable reference is not implemented (4b)`,
//! with no `CALL` anywhere in the program.

use std::path::Path;

use rexx_exec::{NOT_IMPLEMENTED_EXIT, run_program};
use rexx_parse::{ExprKind, InstructionKind, parse_program};

#[path = "owners.rs"]
mod owners;
use owners::{EXPR_TAGS, INSTRUCTION_TAGS, Owner, SPLIT_TABLE_PHASES, expr_tag, instruction_tag};

/// The fully arm-qualified tag for one `InstructionKind`: the coarse
/// `owners::instruction_tag` for every variant except `Call` and `Signal`,
/// whose own inner arm this returns instead. See the module doc's "Arm-
/// grained ownership" section for why those two specifically need this and
/// the rest do not.
fn instruction_arm(kind: &InstructionKind) -> String {
    match kind {
        InstructionKind::Call(c) => match &**c {
            rexx_parse::Call::Named { .. } => "Call::Named".to_string(),
            rexx_parse::Call::Dynamic { .. } => "Call::Dynamic".to_string(),
            rexx_parse::Call::Qualified { .. } => "Call::Qualified".to_string(),
            rexx_parse::Call::Trap(_) => "Call::Trap".to_string(),
        },
        InstructionKind::Signal(s) => match &**s {
            rexx_parse::Signal::Trap(_) => "Signal::Trap".to_string(),
            rexx_parse::Signal::Value(_) | rexx_parse::Signal::Label(_) => "Signal".to_string(),
        },
        other => instruction_tag(other).0.to_string(),
    }
}

/// `owners::INSTRUCTION_TAGS`'s coarse, phase-owned tag names, expanded to
/// the fine-grained witness tags `INSTRUCTION_WITNESSES` actually carries
/// for `Call` and `Signal` (`instruction_arm`'s own inverse, in effect) --
/// every other coarse tag expands to itself. Hand-maintained rather than
/// derived from `rexx_parse::Call`/`Signal`, which nothing here can
/// enumerate at compile time -- the same device `EXPECTED_OUT_OF_SCOPE`
/// already uses for a different set, one level up.
fn expand_for_witnesses(coarse: &'static str) -> Vec<&'static str> {
    match coarse {
        "Call" => vec![
            "Call::Named",
            "Call::Dynamic",
            "Call::Qualified",
            "Call::Trap",
        ],
        "Signal" => vec!["Signal", "Signal::Trap"],
        other => vec![other],
    }
}

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
    // ---- `Call`, arm-grained (Step 2): see the module doc ----
    Witness {
        tag: "Call::Named",
        owner: "4b",
        source: "call sub\n",
        category: Category::Instruction,
    },
    Witness {
        tag: "Call::Dynamic",
        owner: "4b",
        // `CALL (expr) args`, whose target is only known at run time.
        source: "call (x)\n",
        category: Category::Instruction,
    },
    Witness {
        tag: "Call::Qualified",
        owner: "Phase 5",
        // `CALL ns:name args`, restricted to public routines of that
        // namespace -- genuinely Phase 5's, unlike `Call::Named`/`Dynamic`/
        // `Trap`, all "4b".
        source: "call ns:sub\n",
        category: Category::Instruction,
    },
    Witness {
        tag: "Call::Trap",
        owner: "4b",
        // `CALL ON`/`CALL OFF`. The `OFF` form needs no `NAME` clause, the
        // shortest way to construct this arm.
        source: "call off error\n",
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
    // ---- `Signal`, arm-grained (Step 2) for `Trap` only: `Value`/`Label`
    // both move in scope in the same task (6), so they stay one row; `Trap`
    // (`SIGNAL ON`/`SIGNAL OFF`) does not move until Task 7 -- see the
    // module doc ----
    Witness {
        tag: "Signal",
        owner: "4b",
        source: "signal there\nthere: nop\n",
        category: Category::Instruction,
    },
    Witness {
        tag: "Signal::Trap",
        owner: "4b",
        // `SIGNAL ON`/`SIGNAL OFF`. The `OFF` form needs no `NAME` clause,
        // mirroring `Call::Trap`'s own witness above.
        source: "signal off error\n",
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

/// Every out-of-scope `ExprKind`, one witness each, every one wrapped in
/// `SAY` (4a's own) -- see the module doc's corrected note on
/// `VariableReference`.
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
        // `say >x` reaches this arm directly -- see the module doc's
        // corrected note. No `CALL` needed.
        source: "say >x\n",
        category: Category::Expr,
    },
];

/// Confirms `path`'s program actually constructs `witness.tag` in the
/// category it claims, before running it. A snippet that parses into the
/// wrong shape would otherwise let a passing exit-code check mean nothing.
///
/// `instruction_arm`, not the coarse `owners::instruction_tag`, for the
/// `Category::Instruction` half: a witness's `tag` may name a specific
/// `Call`/`Signal` arm (Step 2), and only `instruction_arm` can tell those
/// apart.
fn assert_constructs(witness: &Witness) {
    let program = parse_program(witness.source.as_bytes().to_vec())
        .unwrap_or_else(|e| panic!("witness for {} failed to parse: {e:?}", witness.tag));
    let found = match witness.category {
        Category::Instruction => program
            .main
            .instructions
            .iter()
            .any(|i| instruction_arm(&i.kind) == witness.tag),
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
    // `flat_map(expand_for_witnesses)`, not a bare `.map`: `Call` and
    // `Signal` each expand to more than one expected tag (Step 2, the
    // module doc's "Arm-grained ownership"), so the coarse 20 phase-owned
    // `InstructionKind` tags become 24 expected witness tags -- `Call`'s
    // one row becomes four (`Named`/`Dynamic`/`Qualified`/`Trap`) and
    // `Signal`'s one becomes two (`Signal`/`Signal::Trap`), net +3 +1 = +4.
    let expected_instructions: Vec<&str> = INSTRUCTION_TAGS
        .iter()
        .filter(|(_, o)| matches!(o, Owner::Phase(_)))
        .flat_map(|(name, _)| expand_for_witnesses(name))
        .collect();
    let mut got_instructions: Vec<&str> = INSTRUCTION_WITNESSES.iter().map(|w| w.tag).collect();
    got_instructions.sort();
    let mut expected_sorted = expected_instructions.clone();
    expected_sorted.sort();
    assert_eq!(
        got_instructions, expected_sorted,
        "INSTRUCTION_WITNESSES must have exactly one entry per out-of-scope \
         InstructionKind variant (per arm, for Call/Signal), no more and no \
         fewer"
    );
    assert_eq!(expected_instructions.len(), 24);

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
            continue;
        }
        // Task 0's Step 3: the loud message now names an owner
        // (`src/lib.rs`'s `instruction_owner`/`expr_owner`), and this is
        // where that is proved rather than merely formatted -- a stderr
        // that does not name the witness's own owner would mean production
        // and this table drifted apart, exactly the drift Step 5's fifth
        // pinned item warns about.
        //
        // **Pins the exact trailing shape, not merely the owner's presence
        // (review finding I2).** An earlier version checked
        // `stderr.contains(witness.owner)`, which a message reading
        // `[4b] CALL: unimplemented` also satisfies -- `contains` cannot
        // tell "names the owner in the documented shape" from "mentions the
        // owner's bytes somewhere". `ends_with` on the exact suffix
        // `owned_message` (`lib.rs`) produces is what actually pins the
        // shape later tasks are told to rely on
        // (`" is not implemented (OWNER)"`, `trim_end` first since every
        // message ends in `\n`).
        let stderr = String::from_utf8_lossy(&outcome.stderr);
        let want_suffix = format!(" is not implemented ({})", witness.owner);
        if !stderr.trim_end().ends_with(&want_suffix) {
            use std::fmt::Write as _;
            writeln!(
                failures,
                "{} ({}): stderr does not end with {want_suffix:?}: {stderr:?}",
                witness.tag, witness.owner
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
