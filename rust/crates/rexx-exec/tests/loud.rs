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
//! `coverage.rs` uses -- has a total of 28 `InstructionKind` and 11
//! `ExprKind` entries (20 and 9 at the 4a gate; 4b's Task 1 moved
//! `Interpret` in scope, Task 3 moved `Return`, Task 4 moved
//! `ExprKind::Call`, Task 5 moved `Procedure`/`Use`/`ExprKind::
//! VariableReference`, Task 7 moved `Signal`/`Raise` and Task 8 moved
//! `Push`/`Queue`), so an in-scope variant cannot go unlisted by omission.
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
//! `InstructionKind::Call` is one row in `owners.rs`'s `INSTRUCTION_TAGS`
//! (`"Call"` -> `"Phase 5"`), which is right for criterion 1's parse-only
//! coverage question ("was some `Call` constructed") but wrong for this
//! file's own question, because it wraps an inner enum (`rexx_parse::Call`)
//! whose *arms* do not all become in-scope, or even all belong to the same
//! owner, at the same moment: `Call::Named` and `Call::Dynamic` are in scope
//! from Task 3 and `Call::Trap` (`CALL ON`/`CALL OFF`) from Task 7, while
//! `Call::Qualified` is genuinely Phase 5's (a namespace-qualified `CALL`,
//! mirroring `ExprKind::QualifiedCall`'s own ownership). A single `"Call"`
//! witness row could not survive that: the moment Task 3 implemented
//! `Call::Named`, the witness naming it had to be deleted, and a shared row
//! would have taken `Call::Qualified`'s own loudness coverage down with it,
//! with nothing left to catch that `CALL ns:name` is still unimplemented.
//!
//! **`InstructionKind::Signal` was the second such variant and no longer is.**
//! `Signal::Label`/`Signal::Value` moved in scope together at Task 6, leaving
//! `"Signal::Trap"` alone to catch that `SIGNAL ON`/`SIGNAL OFF` still was
//! not; Task 7 implemented that arm too, so the coarse tag is
//! `Owner::InScope` and nothing about `Signal` is arm-grained any more. The
//! history is kept here because it is the clearest illustration of why the
//! split exists at all: one row would have gone silently dead at Task 6.
//!
//! So `INSTRUCTION_WITNESSES` below carries **one row per still-loud arm**
//! for the one split variant left -- `"Call::Qualified"` alone.
//! [`instruction_arm`] is what resolves a witness's tag against the program
//! it parses: the coarse `owners::instruction_tag` for every variant except
//! `Call` and `Signal`, and the inner arm's own name for those two (it still
//! names `Signal`'s arms, harmlessly, since a tag it produces for an
//! in-scope variant is never looked up).
//! [`assert_witness_set_is_complete`]'s own expected set is built the same
//! way, through [`expand_for_witnesses`] -- a hand-maintained expansion of
//! `owners.rs`'s coarse tag list, the same "pin it as a literal, not as
//! whatever the code currently computes" device `EXPECTED_OUT_OF_SCOPE`
//! already uses one level up, because nothing here can enumerate
//! `rexx_parse::Call`'s own arms at compile time.
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
//! (itself out of scope) ever succeeding. `PROCEDURE` was on that list until
//! 4b's Task 5 moved it in scope and deleted its row; a bare top-level
//! `procedure` is now the oracle's error 17.1, not a gap.
//!
//! **`ExprKind::VariableReference` (`>x`/`<x`) also has no row since Task
//! 5.** The fact this paragraph existed to record is still worth keeping,
//! because it was got wrong once: `ast.rs`'s 20.930 is about which *token*
//! may follow `>`/`<` (a variable or a stem, not a literal or a number,
//! `expr.rs`'s own `parseVariableReferenceTerm` doc), **not** about which
//! instruction context the whole reference may sit in -- so `say >x` is
//! legal on its own and needs no `CALL` around it. What has changed is the
//! outcome: `say >x` now prints the referenced variable's value, measured
//! against the oracle, where it used to answer `rexx-exec: a variable
//! reference is not implemented (4b)`. One position where `>` is *not* a
//! reference, also measured: `say 'text' >x` is a comparison, because a `>`
//! following a complete term is the operator.

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
            rexx_parse::Signal::Label(_) => "Signal::Label".to_string(),
            rexx_parse::Signal::Value(_) => "Signal::Value".to_string(),
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
        // One, not four: Task 3 implemented `Call::Named` and
        // `Call::Dynamic` and Task 7 implemented `Call::Trap`, so three rows
        // are gone and this expansion shrank with them. The coarse tag stays
        // phase-owned in `owners.rs` -- as `Phase 5` now, not `4b` -- because
        // `Call::Qualified` is the arm still left.
        "Call" => vec!["Call::Qualified"],
        // **No `"Signal"` arm at all since Task 7.** `Signal::Trap` was the
        // last of the three, so the coarse tag is `Owner::InScope` in
        // `owners.rs` and never reaches this function: only phase-owned tags
        // are expanded. A stale `"Signal" => vec!["Signal::Trap"]` here
        // would be dead rather than wrong, which is exactly the kind of
        // entry that survives a rename and then misleads.
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

/// Every out-of-scope `InstructionKind`, one witness each. **12 entries**, one
/// per `Owner::Phase` arm after `Call` expands -- 12 coarse phase-owned tags
/// in `INSTRUCTION_TAGS` above, and `Call`'s own single tag still expands to
/// its one remaining loud arm (`Call::Qualified`), so coarse and expanded
/// counts are equal. (Review finding M1: this line said "19 entries" and,
/// before that, "20", both of which counted the coarse tags while describing
/// the expanded list -- the two numbers are different quantities and the
/// arm-grained section of the module doc above is where the distinction is
/// set out. **This line went stale the identical way twice more after
/// that.** Task 5 moved `Procedure` and `Use` in scope and nothing here
/// followed, sitting at "20 entries -- 18 coarse" through Tasks 4 and 5
/// while `assert_witness_set_is_complete`'s own copy of this same
/// arithmetic, a few dozen lines down, was correctly kept at 16/18 --
/// Task 6 fixed both numbers here, together with `Signal`'s own new count
/// ("17 entries -- 16 coarse... `Call` becomes two rows and `Signal`
/// becomes one"). **Then Task 7 moved `Signal` and `Raise` in scope and
/// moved `Call::Trap` in scope alongside `Signal`'s own arm, dropping
/// `Call`'s own expansion from two rows to one and leaving coarse and
/// expanded counts equal for the first time (16/17 to 14/14), and this line
/// sat at Task 6's stale "17/16/two rows" text through that whole task**,
/// uncorrected -- exactly the drift the parenthetical above already warned
/// about, found only now, by Task 8, which fixes it a third time alongside
/// its own change: `Push` and `Queue` move in scope, 14 to 12 both ways.)
///
/// 12 coarse tags since 4b's Task 8 moved `Push` and `Queue` in scope (14
/// after Task 7's `Signal`/`Raise`, 16 after Task 5's `Procedure`/`Use`, 18
/// after Task 3's `Return`, 19 after Task 1's `Interpret`, 20 at the 4a gate)
/// -- pinned item 4 in `owners.rs`'s own list: a witness for a variant that
/// moved in scope must be *deleted*, not left stale, or
/// `assert_witness_set_is_complete` fails the other way.
/// `assert_witness_set_is_complete` checks the two lists against each other so
/// a variant cannot be silently dropped from this list while staying in the tag
/// table.
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
        tag: "Call::Qualified",
        owner: "Phase 5",
        // `CALL ns:name args`, restricted to public routines of that
        // namespace -- genuinely Phase 5's, unlike `Call::Named`/`Dynamic`/
        // `Trap`, all "4b".
        source: "call ns:sub\n",
        category: Category::Instruction,
    },
    // **No `Call::Trap` row since 4b's Task 7.** `CALL ON`/`CALL OFF` is
    // implemented, so a row here would assert a loud failure that correctly
    // no longer happens -- the same way `Procedure`'s and `Use`'s rows had
    // to go at Task 5.
    // **No `Procedure` or `Use` row since 4b's Task 5.** Both moved into
    // scope (`owners.rs`'s own `INSTRUCTION_TAGS`). The `Procedure` row's
    // own witness, `sub: procedure` reached by falling through the label,
    // is now the oracle's error 17.1 at rc 239 rather than a loud gap --
    // measured, and `exec_procedure`'s doc has the four-shape table -- so
    // keeping the row would assert a loud failure that correctly no longer
    // happens.
    // **No `Signal` row of any grain since 4b's Task 7, and no `Raise` row.**
    // `Signal::Label`/`Signal::Value` moved into scope at Task 6 and
    // `Signal::Trap` here, so all three arms are implemented and the coarse
    // tag is `Owner::InScope`; `RAISE` is implemented whole (`owners.rs`'s
    // own entry has why its one remaining loud shape belongs to
    // `ExprKind::List` rather than to `RAISE`).
    // **No `Push` or `Queue` row since 4b's Task 8 (I15).** Both moved into
    // scope: `queue.rs` stores every line either writes. This is 4b's own
    // last pair of rows -- no `owner: "4b"` witness remains anywhere in
    // this list, and `SPLIT_TABLE_PHASES` keeping `"4b"` as a valid phase
    // name is not stale, since a phase can still owe nothing right now and
    // owe something again if a later task's audit finds otherwise.
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
///
/// **No `Call` row since 4b's Task 4.** `ExprKind::Call` moved fully into
/// scope (`owners.rs`'s own `EXPR_TAGS`, and see `eval_call`'s doc,
/// `eval.rs`, for the resolution order): a name that is not an internal
/// label -- `foo` in this row's own former witness, `say foo(1)` -- now
/// runs through `eval_call` and fails loudly naming `4c`, not `4b`, so the
/// row this list used to carry would assert the wrong owner rather than
/// disappearing quietly.
const EXPR_WITNESSES: &[Witness] = &[
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
    // `flat_map(expand_for_witnesses)`, not a bare `.map`, and since 4b's
    // Task 7 there is exactly one coarse tag left that expands to anything
    // other than itself (Step 2, the module doc's "Arm-grained ownership").
    // 12 coarse phase-owned `InstructionKind` tags since Task 8 moved `Push`
    // and `Queue` in scope (14 after Task 7's `Signal`/`Raise`, 16 after
    // Task 5's `Procedure`/`Use`, 18 after Task 3's `Return`, 19 after Task
    // 1's `Interpret`, 20 at the 4a gate); `Call`'s one row becomes **one**
    // (`Qualified` alone, its other three arms being Tasks 3 and 7's), net
    // +0, so 12 expected witness tags. The `flat_map` stays rather than
    // collapsing to `.map`: it is still doing real work for `Call`, and it
    // is what a later task adding a second split variant needs already in
    // place.
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
         InstructionKind variant (per arm, for Call), no more and no \
         fewer"
    );
    assert_eq!(expected_instructions.len(), 12);

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
    // 4 since 4b's Task 5 moved `ExprKind::VariableReference` in scope and
    // deleted its row (5 after Task 4 did the same for `ExprKind::Call`, 6
    // before that).
    assert_eq!(expected_exprs.len(), 4);
}

#[test]
fn in_scope_counts_match_the_audited_split() {
    // 28 since 4b's Task 8 moved `Push` and `Queue` in scope (26 after Task
    // 7's `Signal`/`Raise`, 24 after Task 5's `Procedure`/`Use`, 22 after
    // Task 3's `Return`, 21 after Task 1's `Interpret`); see `owners.rs`'s
    // own `variant_counts_match_the_audited_split` for the full split,
    // including `Call`'s sideways move from 4b's column into Phase 5's,
    // which changes no count here.
    assert_eq!(
        INSTRUCTION_TAGS
            .iter()
            .filter(|(_, o)| *o == Owner::InScope)
            .count(),
        28
    );
    // 11 since 4b's Task 5 moved `ExprKind::VariableReference` in scope (10
    // after Task 4 moved `ExprKind::Call`, 9 before that).
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
