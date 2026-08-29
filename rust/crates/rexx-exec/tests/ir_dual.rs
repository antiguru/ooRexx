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

//! The dual-engine comparison: every program of this crate's existing
//! populations, run twice -- once on the tree-walker and once on the
//! register-based instruction stream -- with stdout, stderr and exit status
//! required to be byte-identical.
//!
//! # What this proves, and what it does not
//!
//! **The one thing it cannot see is work the two engines share.** An
//! instruction the compiler has not promoted delegates its clause back to the
//! tree-walker's own clause unit, and a construct that both arms resolve by
//! calling the same `Interp` function answers identically on both by
//! construction. Every promotion that shares its tail instead of
//! re-implementing it widens that blind spot rather than narrowing it, so
//! this comparison is not evidence that any of *those* is right; that is what
//! the oracle harnesses are for, and the two halves compose.
//!
//! **That is a property to keep rather than a weakness to fix**: a promotion
//! that shares its semantics cannot diverge, and one that re-implements them
//! can, which is what this comparison is here to catch.
//!
//! **What it does see is anything a compiled op does that the shared path does
//! not, and that is no longer hypothetical.** The stream carries ops whose
//! emission is not delegated at all: `crate::ir::Op::Const` produces a
//! literal's value without entering `eval.rs`, so the `>L>` line `eval.rs`
//! emits as a *side effect* of evaluating a literal has to be re-emitted by an
//! op of its own -- and **this file is what catches that line going missing.**
//! Measured, by making that op's emission a no-op:
//! [`both_engines_agree_across_every_population`] reddens, and stays red with
//! the purpose-written case file for those instructions held out of the
//! directory entirely.
//!
//! A bare-symbol read is the second such op and was falsified the same way,
//! with a result worth writing down because it is not the same one. Making
//! `crate::ir::Op::TraceRead`'s emission a no-op reddens
//! [`both_engines_agree_on_every_case_file`], and it stays red with
//! `ir_dual_cases/variable-reads` held out -- the `>V>` line a simple variable
//! owes is already exercised by rows written for something else. What that file
//! is the only catcher for is narrower: dropping the echo for a **bare stem**
//! alone leaves every dual-engine test in the workspace green without it, and
//! reddens this one with it.
//!
//! **The oracle differential cannot see the same defect**, because
//! `tests/support/mod.rs` normalises the region such a line sits in. This
//! comparison diffs raw stderr between the arms, so it can. Neither harness
//! substitutes for the other and the pair is stronger than either: this one
//! says a promoted expression still emits what its evaluation used to, and the
//! oracle harnesses say what that ought to be.
//!
//! What it proves beyond that is everything *around* the delegation, which is
//! the whole of what the driver adds and is not shared with `run_activation`:
//!
//! * the outer clause loop terminates where the tree-walker's does, on the
//!   same instruction, for every program in the population;
//! * the `pc` stays an instruction index across `Flow::Goto` and
//!   `Flow::Signal`, so `SIGNAL`, `IF`, `SELECT`, `DO` and `LEAVE` resume at
//!   the same clause under both engines;
//! * the trap offer sits where `run_activation` puts it, so a condition
//!   trapped by `SIGNAL ON`/`CALL ON` is offered once per activation, not
//!   once per nested construct;
//! * `grant_procedure_permission` is granted and spent identically, so
//!   `PROCEDURE` and `USE LOCAL` see the same first-instruction answer;
//! * the register region is opened and closed without disturbing the
//!   temporaries stack any clause depends on;
//! * every body in the population compiles -- `chunks_refused`, which counts
//!   refusals rather than distinct bodies, is asserted zero on both arms, so
//!   a compiler that started refusing ordinary bodies could not pass by
//!   quietly running everything on the tree-walker;
//! * a promoted expression re-emits every intermediate trace line its
//!   evaluation used to produce as a side effect, for every traced program in
//!   the population.
//!
//! Which *engine* actually ran, and how much of a program it drove, is not
//! observable from that program's output, and this file makes no attempt to
//! infer it from one. That question is answered where it can be answered
//! honestly, by counting what the driver did: `src/ir/drive/tests.rs`.
//!
//! # There is no REPORT mode here
//!
//! `corpus.rs`, `assertions.rs`, `bif_assertions.rs` and
//! `keyword_assertions.rs` each default to an always-green progress report
//! and gate only under their own environment variable
//! (`REXX_CORPUS_GATE`, `REXX_ASSERTIONS_GATE`, `REXX_BIF_GATE`,
//! `REXX_KEYWORD_GATE`). That is right for them: each measures how far this
//! crate has got against an external expectation, and how far that is
//! changes with every task.
//!
//! It would be wrong here. A divergence between two engines running the same
//! program is a defect at any point in the phase, with nothing to forgive
//! and no denominator to report -- so **this file asserts unconditionally
//! and reads no environment variable at all.** Setting the four gate
//! variables changes nothing about what it checks, which is strictly
//! stronger than honouring them: there is no mode in which it exits 0 having
//! found a divergence.
//!
//! What those four variables still matter for is the *other* half of the
//! argument. This file says the two engines agree; it says nothing about
//! whether either is right. That comes from the four harnesses above, run in
//! STRICT, on the default engine. The two halves compose and neither
//! substitutes for the other.
//!
//! # The populations, and why an arm cannot silently skip one
//!
//! Both arms run from **one** list. [`populations`] builds every case once
//! and [`compare`] runs that same case twice, so "the two arms saw the same
//! programs" is a property of the code rather than something asserted about
//! two separately built lists.
//!
//! What is asserted is that the list is complete, and every such assertion
//! compares against something outside this file:
//! [`the_dual_harness_reads_every_phase_subset_file`] pins the corpus half
//! against the corpus directory itself (the shape `corpus.rs`'s own
//! `the_differential_reads_every_phase_subset_file` uses),
//! [`the_sweep_runs_every_ootest_suite_a_sibling_harness_runs`] pins the
//! other half against the suite roots the sibling harnesses in `tests/` name,
//! and [`every_population_the_tree_calls_for_is_present_and_non_empty`]
//! requires each to have found programs. A population that silently
//! extracted nothing, or was deleted along with the line declaring it, would
//! otherwise pass with a shrunken denominator and no output saying so.

use std::fs;
use std::path::{Path, PathBuf};

use rexx_exec::{Engine, Invocation, Outcome, run_program};
use rexx_extract::bif::extract_bif;
use rexx_extract::keyword::extract_keyword;
use rexx_extract::{AssertionRow, Form, extract_assertions, find_test_groups};

/// The brief's own first test: a two-clause program, both engines, byte for
/// byte.
///
/// Kept beside the population sweep rather than folded into it because it is
/// the one case a reader can check by eye, and because it is the smallest
/// program that fails if engine selection or the driver's outer loop is
/// broken outright.
#[test]
fn both_engines_agree_on_an_all_generic_program() {
    let text = b"n1 = 2\nsay n1 + 3\n".to_vec();
    let tw = run(text.clone(), Engine::TreeWalker);
    let ir = run(text, Engine::Ir);
    assert_eq!(tw.stdout, ir.stdout);
    assert_eq!(tw.stderr, ir.stderr);
    assert_eq!(tw.exit_code, ir.exit_code);
    assert_eq!(tw.stdout, b"5\n", "the tree-walker's own answer moved");
}

fn run(text: Vec<u8>, engine: Engine) -> Outcome {
    run_program(INLINE_PATH, text, Invocation::none().with_engine(engine))
}

/// One inline program with the answer the tree-walker gives for it.
///
/// The expected bytes are half of what each case is worth and the two-engine
/// comparison is the other half, because neither half alone is enough here.
/// The comparison says the two engines agree and says nothing about what
/// either does; the expected bytes say what the program does and would stay
/// green if the IR engine were never selected at all. A case carries both.
struct InlineCase {
    name: &'static str,
    program: &'static str,
    stdout: &'static str,
    /// The trace sink, empty for a case that sets no `TRACE`. Written out in
    /// full for the traced cases rather than summarised: a construct's trace
    /// is where a re-implementation diverges first, because a `DO`/`END` pair
    /// re-echoes per pass and an `IF`'s `THEN`/`ELSE` markers echo at lines
    /// and indents no `SAY` can observe.
    ///
    /// `<PATH>` stands for [`INLINE_PATH`], which a raised condition's middle
    /// line prints and which no `&'static str` can spell without repeating
    /// the constant; [`compare_inline_cases`] substitutes it.
    stderr: &'static str,
    /// The exit status the tree-walker reports, so a case whose whole point
    /// is a raised condition pins the status as well as the message.
    exit_code: i32,
}

/// The shapes the loop promotion has to keep: one per `LoopKind` this crate
/// runs, both `LoopConditional` spellings, both `LEAVE` and `ITERATE`, and two
/// traced loops.
///
/// Every one of these passes with both engines delegating to the tree-walker's
/// clause unit, which is the point of adding them before the compiler emits
/// anything: a case written after a promotion cannot say whether it ever would
/// have failed.
const LOOP_CASES: &[InlineCase] = &[
    InlineCase {
        name: "controlled",
        program: "do i = 1 to 3\n  say i\nend\n",
        stdout: "1\n2\n3\n",
        stderr: "",
        exit_code: 0,
    },
    InlineCase {
        // The per-pass control-variable re-read, which is the behaviour a
        // reviewer once caught being called unreachable: the body writes the
        // control variable and the next pass reads `10` back, adds the `BY`,
        // and stops because `11 > 3`.
        name: "controlled, body writes the control variable",
        program: "do i = 1 to 3\n  i = 10\nend\nsay i\n",
        stdout: "11\n",
        stderr: "",
        exit_code: 0,
    },
    InlineCase {
        name: "controlled with BY and FOR",
        program: "do i = 1 to 10 by 3 for 2\n  say i\nend\nsay i\n",
        stdout: "1\n4\n7\n",
        stderr: "",
        exit_code: 0,
    },
    InlineCase {
        name: "bare repeat count",
        program: "do 3\n  say 'zz'\nend\n",
        stdout: "zz\nzz\nzz\n",
        stderr: "",
        exit_code: 0,
    },
    InlineCase {
        name: "while",
        program: "n1 = 0\ndo while n1 < 3\n  n1 = n1 + 1\n  say n1\nend\n",
        stdout: "1\n2\n3\n",
        stderr: "",
        exit_code: 0,
    },
    InlineCase {
        name: "until",
        program: "n1 = 0\ndo until n1 >= 3\n  n1 = n1 + 1\n  say n1\nend\n",
        stdout: "1\n2\n3\n",
        stderr: "",
        exit_code: 0,
    },
    InlineCase {
        name: "forever with leave",
        program: "n1 = 0\ndo forever\n  n1 = n1 + 1\n  if n1 = 3 then leave\nend\nsay n1\n",
        stdout: "3\n",
        stderr: "",
        exit_code: 0,
    },
    InlineCase {
        // The label search, across two frames: the `ITERATE` names the outer
        // loop from inside the inner one, so the inner loop is left and the
        // outer one re-tested.
        name: "nested, labelled iterate",
        program: "zz = 0\ndo label lbl outer = 1 to 3\n  do inner = 1 to 3\n    \
                  if inner = 2 then iterate lbl\n    zz = zz + 1\n  end\nend\nsay zz\n",
        stdout: "3\n",
        stderr: "",
        exit_code: 0,
    },
    InlineCase {
        // `DO OVER` in the single-iteration form this crate implements for a
        // non-stem target: the target yields itself, once.
        name: "do over a non-stem target",
        program: "do qq over 4.5\n  say qq\nend\n",
        stdout: "4.5\n",
        stderr: "",
        exit_code: 0,
    },
    InlineCase {
        // A block, not a loop: exactly one pass, and its `END` echoes once.
        name: "simple block",
        program: "do\n  say 'a'\n  say 'zz'\nend\nsay 'after'\n",
        stdout: "a\nzz\nafter\n",
        stderr: "",
        exit_code: 0,
    },
    InlineCase {
        // The same block reached through an `IF`'s true branch, which is a
        // different route into it: `If` steps its branch itself, so this
        // block's own clauses arrive from `If`'s driver rather than from the
        // enclosing body's.
        name: "simple block inside an if",
        program: "if 1 = 1 then do\n  say 'a'\n  say 'zz'\nend\n",
        stdout: "a\nzz\n",
        stderr: "",
        exit_code: 0,
    },
    InlineCase {
        // A labelled `Simple` block is leavable by name where an unlabelled
        // one is not, and it owns a search frame either way.
        name: "labelled simple block, left by name",
        program: "do label blk\n  say 'a'\n  leave blk\n  say 'never'\nend\nsay 'after'\n",
        stdout: "a\nafter\n",
        stderr: "",
        exit_code: 0,
    },
    InlineCase {
        // The `DO` clause re-echoes once per pass and `END` once per pass that
        // falls through to it, and the control step's four intermediate lines
        // straddle the `BY` addition.
        name: "controlled under trace i",
        program: "trace i\ndo i = 1 to 2\n  nop\nend\n",
        stdout: "",
        stderr: "     2 *-* do i = 1 to 2\n       >L>   \"1\"\n       >L>   \"2\"\n       \
                 >K>   \"TO\" => \"2\"\n       >=>   I <= \"1\"\n     3 *-*   nop\n     \
                 4 *-* end\n     2 *-* do i = 1 to 2\n       >V>     I => \"1\"\n       \
                 >>>     \"1\"\n       >>>     \"2\"\n       >=>     I <= \"2\"\n     \
                 3 *-*   nop\n     4 *-* end\n     2 *-* do i = 1 to 2\n       \
                 >V>     I => \"2\"\n       >>>     \"2\"\n       >>>     \"3\"\n       \
                 >=>     I <= \"3\"\n",
        exit_code: 0,
    },
    InlineCase {
        // `UNTIL` gets a second, unconditional `DO` re-echo of its own,
        // between `END` and the test, and no top-of-loop one.
        name: "until under trace r",
        program: "trace r\nn1 = 0\ndo until n1 >= 2\n  n1 = n1 + 1\nend\n",
        stdout: "",
        stderr: "     2 *-* n1 = 0\n       >>>   \"0\"\n     3 *-* do until n1 >= 2\n     \
                 4 *-*   n1 = n1 + 1\n       >>>     \"1\"\n     5 *-* end\n     \
                 3 *-* do until n1 >= 2\n       >K>     \"UNTIL\" => \"0\"\n     \
                 4 *-*   n1 = n1 + 1\n       >>>     \"2\"\n     5 *-* end\n     \
                 3 *-* do until n1 >= 2\n       >K>     \"UNTIL\" => \"1\"\n",
        exit_code: 0,
    },
];

/// What a branch promotion has to keep, in two kinds.
///
/// **Shapes**, which ask what a branch does: an `IF` with an `ELSE` on both
/// paths and one without on both paths, a nested `IF`, a null `THEN`
/// consequence, a `SELECT` that matches and two that do not (**7.3**, not the
/// 93.4 an earlier draft of the plan named -- measured against the oracle,
/// which reports "All WHEN expressions of SELECT are false; OTHERWISE
/// expected" at rc 249 for both `SELECT` and `SELECT CASE`), a condition that
/// is not a logical value, a condition that raises into a `SIGNAL ON` trap, a
/// branch inside a loop body that leaves it, a `DO` block as a whole true
/// branch, and a branch in a `CALL`ed label.
///
/// **Boundaries**, which ask whether the clause unit is discharged exactly
/// once where a promoted clause opens one: a `PROCEDURE` reached through a
/// true branch, which is 17.1 because a branch does not inherit the
/// first-instruction permission, and two `CALL ON` handlers that fail at an
/// `IF`'s own clause boundary.
///
/// **The second kind is the one a table of shapes does not reach**, and it is
/// here because a first version of this table had only the first kind and a
/// promoted `IF` shipped without recording its boundary's failure site: every
/// shape agreed on both engines and the divergence was in a clause obligation
/// no shape asks about. A table extended for the next promotion should grow
/// boundaries, not more shapes.
///
/// **`SELECT` opens a clause per listed `WHEN` as well as for its own header**,
/// so it has more of those boundaries than `IF` does. Its own boundary cases
/// are: a `CALL ON` handler queued by the `SELECT CASE` expression, which the
/// header's boundary must deliver before the first `WHEN` is tested; a handler
/// failing at a listed `WHEN`'s own clause boundary; a handler failing at a
/// matched `WHEN` body clause's boundary; and a `PROCEDURE` inside a matched
/// `WHEN` body, which is 17.1 for the same reason a branch's is.
///
/// Every one of these passes with both engines delegating to the tree-walker's
/// clause unit, which is the point of adding them before the compiler emits
/// anything: a case written after a promotion cannot say whether it ever would
/// have failed.
///
/// Every expected byte below was measured against the oracle before it was
/// written down, and the two agree on all of them.
const BRANCH_CASES: &[InlineCase] = &[
    InlineCase {
        name: "if with an else, true path",
        program: "if 1 = 1 then say 'then'\nelse say 'else'\nsay 'after'\n",
        stdout: "then\nafter\n",
        stderr: "",
        exit_code: 0,
    },
    InlineCase {
        // The path the compiled form changes most: the tree-walker leaves it
        // to the enclosing loop's fallthrough, and a jump has to land on the
        // same `ELSE`.
        name: "if with an else, false path",
        program: "if 1 = 0 then say 'then'\nelse say 'else'\nsay 'after'\n",
        stdout: "else\nafter\n",
        stderr: "",
        exit_code: 0,
    },
    InlineCase {
        name: "if with no else, false path",
        program: "if 1 = 0 then say 'then'\nsay 'after'\n",
        stdout: "after\n",
        stderr: "",
        exit_code: 0,
    },
    InlineCase {
        // Its own case rather than the mirror of the one above, because the
        // compiled form of a branch with no `ELSE` emits no branch-end jump:
        // the true path leaves by falling off its end, which is a different
        // mechanism from every other true path here.
        name: "if with no else, true path",
        program: "if 1 = 1 then say 'then'\nsay 'after'\n",
        stdout: "then\nafter\n",
        stderr: "",
        exit_code: 0,
    },
    InlineCase {
        // The outer `ELSE` has to be skipped by the inner branch's own
        // resume, which is the one target neither `false_target` gives.
        name: "nested if, both true",
        program: "if 1 = 1 then\n  if 2 = 2 then say 'inner'\n  else say 'inner else'\n\
                  else say 'outer else'\nsay 'after'\n",
        stdout: "inner\nafter\n",
        stderr: "",
        exit_code: 0,
    },
    InlineCase {
        // A null clause as the whole consequence: the true branch is the
        // `THEN` marker and nothing else.
        name: "empty then",
        program: "if 1 = 1 then;\nsay 'after'\n",
        stdout: "after\n",
        stderr: "",
        exit_code: 0,
    },
    InlineCase {
        name: "select with otherwise, second when matches",
        program: "select\n  when 1 = 0 then say 'a'\n  when 2 = 2 then say 'b'\n  \
                  otherwise say 'o'\nend\nsay 'after'\n",
        stdout: "b\nafter\n",
        stderr: "",
        exit_code: 0,
    },
    InlineCase {
        name: "select with no matching when",
        program: "select\n  when 1 = 0 then say 'a'\nend\nsay 'after'\n",
        stdout: "",
        stderr: "     3 *-* end\nError 7 running <PATH> line 3:  WHEN or OTHERWISE expected.\n\
                 Error 7.3:  All WHEN expressions of SELECT are false; OTHERWISE expected.\n",
        exit_code: 249,
    },
    InlineCase {
        name: "select case with no matching when",
        program: "select case 2\n  when 1 then say 'a'\nend\nsay 'after'\n",
        stdout: "",
        stderr: "     3 *-* end\nError 7 running <PATH> line 3:  WHEN or OTHERWISE expected.\n\
                 Error 7.3:  All WHEN expressions of SELECT are false; OTHERWISE expected.\n",
        exit_code: 249,
    },
    InlineCase {
        // The `>K>   "CASE"` line, the two `>>>` lines each `WHEN CASE`
        // comparison produces, and the `THEN` marker: a `SELECT CASE`'s whole
        // trace, which is where a re-implementation of the scan diverges first.
        name: "select case under trace r, second when matches",
        program: "trace r\nselect case 1 + 1\n  when 1 then say 'a'\n  when 2 then say 'b'\n  \
                  otherwise say 'o'\nend\nsay 'after'\n",
        stdout: "b\nafter\n",
        stderr: "     2 *-* select case 1 + 1\n       >K>   \"CASE\" => \"2\"\n     3 *-*   \
                 when 1 \n       >>>     \"1\"\n       >>>     \"0\"\n     4 *-*   when 2 \n       \
                 >>>     \"2\"\n       >>>     \"1\"\n     4 *-*     then\n     4 *-*       \
                 say 'b'\n       >>>         \"b\"\n     7 *-* say 'after'\n       \
                 >>>   \"after\"\n",
        exit_code: 0,
    },
    InlineCase {
        // The `OTHERWISE` marker echoes on its own line at the scan level, and
        // the `END` that closes an `OTHERWISE` echoes and does nothing --
        // neither is reached by any path a matched `WHEN` takes.
        name: "select with otherwise under trace r",
        program: "trace r\nselect\n  when 1 = 0 then say 'a'\n  otherwise say 'o'\nend\n\
                  say 'after'\n",
        stdout: "o\nafter\n",
        stderr: "     2 *-* select\n     3 *-*   when 1 = 0 \n       >>>     \"0\"\n     \
                 4 *-*   otherwise\n     4 *-*     say 'o'\n       >>>       \"o\"\n     \
                 5 *-* end\n     6 *-* say 'after'\n       >>>   \"after\"\n",
        exit_code: 0,
    },
    InlineCase {
        // An **absorbed** `WHEN CASE` -- one this `SELECT`'s `whens` never
        // collected, because it is the listed `WHEN`'s own `THEN` consequence
        // -- whose false path escapes the matched body and lands on the `END`.
        // Its residual indent rides along, so the 7.3 clause echoes four
        // columns further in than the `END`'s own position.
        name: "select case whose absorbed when case falls through to 7.3",
        program: "trace r\nselect case 2\n  when 2 then\n  when 3 then nop\nend\nsay 'after'\n",
        stdout: "",
        stderr: "     2 *-* select case 2\n       >K>   \"CASE\" => \"2\"\n     3 *-*   \
                 when 2 \n       >>>     \"2\"\n       >>>     \"1\"\n     3 *-*     then\n     \
                 4 *-*       when 3 \n       >>>         \"3\"\n       >>>         \"0\"\n     \
                 5 *-*     end\n     5 *-*     end\nError 7 running <PATH> line 5:  WHEN or \
                 OTHERWISE expected.\nError 7.3:  All WHEN expressions of SELECT are false; \
                 OTHERWISE expected.\n",
        exit_code: 249,
    },
    InlineCase {
        // F-EX1: the same absorbed escape landing exactly on this `SELECT`'s
        // own `OTHERWISE` marker, which has to run with this `SELECT`'s search
        // frame still standing -- the `leave s` is what says so, since a
        // redirect that lost the frame reports 28.3 instead.
        name: "select label whose absorbed when case escapes into otherwise",
        program: "select label s case 2\n  when 2 then\n  when 3 then nop\n  otherwise say 'O'\n  \
                  leave s\nend\nsay 'after'\n",
        stdout: "O\nafter\n",
        stderr: "",
        exit_code: 0,
    },
    InlineCase {
        // A `SELECT` is never a repetitive block, so an `ITERATE` naming one is
        // 28.5 even though the name matches -- the one flow a matching label
        // does not consume.
        name: "iterate naming a select is 28.5",
        program: "select label s\n  when 1 = 1 then iterate s\nend\nsay 'after'\n",
        stdout: "",
        stderr: "     2 *-*       iterate s\nError 28 running <PATH> line 2:  Invalid LEAVE or \
                 ITERATE.\nError 28.5:  Symbol following ITERATE (\"S\") does not match a \
                 repetitive block instruction.\n",
        exit_code: 228,
    },
    InlineCase {
        // A matching `LEAVE` is consumed and resumes past the whole `SELECT`,
        // from inside a `DO` block that is itself the matched branch -- so the
        // search walks the block's frame before it reaches the `SELECT`'s.
        name: "leave naming a select from inside a do block in its branch",
        program: "select label s\n  when 1 = 1 then do\n    say 'in'\n    leave s\n    \
                  say 'never'\n  end\nend\nsay 'after'\n",
        stdout: "in\nafter\n",
        stderr: "",
        exit_code: 0,
    },
    InlineCase {
        // A bare `LEAVE` from inside a matched `WHEN` is not the `SELECT`'s: it
        // is forwarded outward and consumed by the enclosing loop.
        name: "select inside a loop body, leaving the loop from a when",
        program: "zn = 0\ndo i = 1 to 5\n  select\n    when i = 3 then leave\n    \
                  otherwise zn = zn + 1\n  end\nend\nsay zn\n",
        stdout: "2\n",
        stderr: "",
        exit_code: 0,
    },
    InlineCase {
        // **A `SELECT` resumes at two different places, and this is the one
        // that is not the other.** A matching `LEAVE` from `OTHERWISE` resumes
        // *past* the `END`, where the same branch falling off its own end runs
        // the `END` (the pair below). Measured against the oracle under
        // `trace r`: no `5 *-* end` line here and one there. A single resume
        // for both echoes an `END` the oracle does not.
        name: "leave naming a select from inside its otherwise",
        program: "trace r\nselect label s\n  when 1 = 0 then nop\n  otherwise leave s\nend\n\
                  say 'after'\n",
        stdout: "after\n",
        stderr: "     2 *-* select label s\n     3 *-*   when 1 = 0 \n       >>>     \"0\"\n     \
                 4 *-*   otherwise\n     4 *-*     leave s\n     6 *-* say 'after'\n       \
                 >>>   \"after\"\n",
        exit_code: 0,
    },
    InlineCase {
        // The adjacent success for the case above: the same `OTHERWISE`
        // finishing normally *does* run the `END`, which is the arrival that
        // makes the two resumes different rather than one being wrong.
        name: "an otherwise that finishes normally runs its own end",
        program: "trace r\nselect label s\n  when 1 = 0 then nop\n  otherwise nop\nend\n\
                  say 'after'\n",
        stdout: "after\n",
        stderr: "     2 *-* select label s\n     3 *-*   when 1 = 0 \n       >>>     \"0\"\n     \
                 4 *-*   otherwise\n     4 *-*     nop\n     5 *-* end\n     6 *-* say 'after'\n  \
                 \x20    >>>   \"after\"\n",
        exit_code: 0,
    },
    InlineCase {
        // The escape elevation an absorbed `WHEN CASE` sets is restored once
        // `OTHERWISE`'s whole dispatch is over, and this is the program that
        // says so: the `END` and the clause after the `SELECT` both trace at
        // their own positions. Left in force they would each print four
        // columns further in, which no case whose `OTHERWISE` is the last
        // thing that traces can see.
        name: "the escape elevation is restored after an escaped otherwise",
        program: "trace r\nselect case 2\n  when 2 then\n  when 3 then nop\n  otherwise nop\nend\n\
                  say 'after'\n",
        stdout: "after\n",
        stderr: "     2 *-* select case 2\n       >K>   \"CASE\" => \"2\"\n     3 *-*   when 2 \n  \
                 \x20    >>>     \"2\"\n       >>>     \"1\"\n     3 *-*     then\n     \
                 4 *-*       when 3 \n       >>>         \"3\"\n       >>>         \"0\"\n     \
                 5 *-*       otherwise\n     5 *-*         nop\n     6 *-* end\n     \
                 7 *-* say 'after'\n       >>>   \"after\"\n",
        exit_code: 0,
    },
    InlineCase {
        // A `SELECT CASE` inside a matched `WHEN`'s branch of another: the
        // inner one's own value must not reclaim the register the outer one is
        // still comparing later `WHEN`s against, and the outer's must survive
        // the inner construct entirely.
        name: "select case nested in a matched when of another",
        program: "select case 2\n  when 1 then say 'no'\n  when 2 then do\n    \
                  select case 'zz'\n      when 'zz' then say 'inner'\n      \
                  otherwise say 'inner o'\n    end\n  end\n  when 3 then say 'no3'\n  \
                  otherwise say 'outer o'\nend\nsay 'after'\n",
        stdout: "inner\nafter\n",
        stderr: "",
        exit_code: 0,
    },
    InlineCase {
        // A `SELECT` inside an `INTERPRET` fragment, which compiles to no
        // chunk at all: the fragment's clauses stay on the tree-walker under
        // both engines, so this says the construct still works from the arm
        // the promotion left in place.
        name: "select inside an interpret fragment",
        program: "interpret \"select; when 1 = 1 then say 'frag'; end\"\nsay 'after'\n",
        stdout: "frag\nafter\n",
        stderr: "",
        exit_code: 0,
    },
    InlineCase {
        // A `LEAVE` naming nothing at all, forwarded past the `SELECT` and then
        // past the loop until the search runs out: the indent it is reported at
        // is the residual each forwarded-past frame resets, which is what
        // `pop_search_frame` decides.
        name: "leave naming nothing, forwarded out of a when",
        program: "do i = 1 to 1\n  select\n    when 1 = 1 then leave nope\n  end\nend\n",
        stdout: "",
        stderr: "     3 *-* leave nope\nError 28 running <PATH> line 3:  Invalid LEAVE or \
                 ITERATE.\nError 28.3:  Symbol following LEAVE (\"NOPE\") must either match the \
                 label of a current loop or block instruction.\n",
        exit_code: 228,
    },
    InlineCase {
        // The same from inside `OTHERWISE`, which reaches the forwarding
        // through a different dispatch than a matched `WHEN` does.
        name: "leave naming nothing, forwarded out of an otherwise",
        program: "do i = 1 to 1\n  select\n    when 1 = 0 then nop\n    otherwise leave nope\n  \
                  end\nend\n",
        stdout: "",
        stderr: "     4 *-* leave nope\nError 28 running <PATH> line 4:  Invalid LEAVE or \
                 ITERATE.\nError 28.3:  Symbol following LEAVE (\"NOPE\") must either match the \
                 label of a current loop or block instruction.\n",
        exit_code: 228,
    },
    InlineCase {
        // **A boundary case, not a shape.** The handler is queued by the
        // `SELECT CASE` expression and has to run at the *header's* own clause
        // boundary, before the first `WHEN` is tested -- the matched branch
        // reads the variable the handler set. A header that never ends its own
        // clause delivers late and prints `v unset`.
        name: "a call on handler queued by a select case expression",
        program: "call on user zx name h\nzv = 'unset'\nselect case raiser()\n  \
                  when 'V' then say 'v' zv\n  otherwise say 'o'\nend\nexit\nraiser:\n\
                  raise user zx return 'V'\nh:\nzv = 'set'\nreturn\n",
        stdout: "v set\n",
        stderr: "",
        exit_code: 0,
    },
    InlineCase {
        // **A boundary case, and the one a review found that this table's own
        // author had argued was unreachable.** The premise it was missing:
        // a delivered handler can *leave a new trap queued behind it*, and a
        // boundary drains only what was queued when it began, so that new trap
        // is not one it owes. So the last member clause's boundary is not the
        // last boundary with work, which is the assumption a flattened
        // construct's single boundary rests on.
        //
        // `h` runs at the body clause's boundary and its own `RAISE ... RETURN`
        // queues `zy`; the boundary that delivers `g` is the one the oracle
        // runs at the branch's synthetic end. Measured: `G ran 4` then
        // `after`. Without it `g` ran after `say 'after'` at the wrong `SIGL`,
        // and in debug the run tripped `clause.rs`'s own assertion.
        name: "a handler that queues again at a matched when's own branch end",
        program: "call on user zx name h\ncall on user zy name g\nselect\n\
                  when 1 = 1 then zq = raiser()\nend\nsay 'after'\nexit\nraiser:\n\
                  raise user zx return 'V'\nh:\nraise user zy return 1\ng:\nsay 'G ran' sigl\n\
                  return\n",
        stdout: "G ran 4\nafter\n",
        stderr: "",
        exit_code: 0,
    },
    InlineCase {
        // **One boundary per construct, not one per branch entered.** The
        // absorbed `WHEN CASE`'s own clause queues `zy` and its false path
        // escapes onto the `OTHERWISE` marker, which is F-EX1's redirect --
        // and a redirect is this `SELECT` still running, so the branch it left
        // owes no end-of-branch boundary. The delivery therefore lands at the
        // `OTHERWISE` marker's own clause, `SIGL` 6, where a `SELECT` that
        // closed the branch it was redirected out of would report 5.
        name: "a handler that queues again where a when is redirected to otherwise",
        program: "call on user zx name h\ncall on user zy name g\nselect case 2\nwhen 2 then\n\
                  when raiser() then nop\notherwise say 'O'\nend\nsay 'after'\nexit\nraiser:\n\
                  raise user zx return 'V'\nh:\nraise user zy return 1\ng:\nsay 'G ran' sigl\n\
                  return\n",
        stdout: "G ran 6\nO\nafter\n",
        stderr: "",
        exit_code: 0,
    },
    InlineCase {
        // The `IF` spelling of the case above, which is the same defect one
        // construct over and was already there before `SELECT` was promoted.
        // Measured: `G ran 3` then `after`.
        name: "a handler that queues again at an if's own branch end",
        program: "call on user zx name h\ncall on user zy name g\nif 1 = 1 then zq = raiser()\n\
                  say 'after'\nexit\nraiser:\nraise user zx return 'V'\nh:\n\
                  raise user zy return 1\ng:\nsay 'G ran' sigl\nreturn\n",
        stdout: "G ran 3\nafter\n",
        stderr: "",
        exit_code: 0,
    },
    InlineCase {
        // **A boundary case, not a shape, and the one that found a live
        // defect.** The handler is queued by a listed `WHEN`'s own condition
        // and delivered at *that* `WHEN`'s clause boundary, where it fails --
        // so the clause blamed is the `WHEN`, and the handler's own clauses
        // print at the `WHEN`'s indent plus two. Measured against the oracle:
        // `3 *-*   when raiser() = 'V'` at indent 2 and `11 *-*     say 1/0`
        // at indent 4. Before a `WHEN` was opened by the clause unit this
        // echoed `2 *-* select` at indent 0 and the handler two columns short,
        // with every other test in the workspace green.
        name: "a call on handler failing at a listed when's own boundary",
        program: "call on user zx name h\nselect\n  when raiser() = 'V' then say 'then'\n  \
                  otherwise say 'o'\nend\nsay 'after'\nexit\nraiser:\nraise user zx return 'V'\n\
                  h:\nsay 1/0\nreturn\n",
        stdout: "",
        stderr: "    11 *-*     say 1/0\n     3 *-*   when raiser() = 'V' \nError 42 running \
                 <PATH> line 11:  Arithmetic overflow/underflow.\nError 42.3:  Arithmetic \
                 overflow; divisor must not be zero.\n",
        exit_code: 214,
    },
    InlineCase {
        // **A boundary case, not a shape.** The handler is queued inside a
        // matched `WHEN`'s body clause and fails at *that* clause's boundary,
        // so the clause blamed is the body's own and not the `WHEN`'s or the
        // `SELECT`'s.
        name: "a call on handler failing at a when body clause's boundary",
        program: "call on user zx name h\nselect\n  when 1 = 1 then say raiser()\n  \
                  otherwise nop\nend\nsay 'after'\nexit\nraiser:\nraise user zx return 'V'\nh:\n\
                  say 1/0\nreturn\n",
        stdout: "V\n",
        stderr: "    11 *-*         say 1/0\n     3 *-*       say raiser()\nError 42 running \
                 <PATH> line 11:  Arithmetic overflow/underflow.\nError 42.3:  Arithmetic \
                 overflow; divisor must not be zero.\n",
        exit_code: 214,
    },
    InlineCase {
        // **A boundary case, not a shape.** A matched `WHEN`'s body does not
        // inherit the first-instruction permission any more than a branch
        // does, so a `PROCEDURE` there is 17.1.
        name: "procedure reached through a matched when",
        program: "call sub 1\nexit\nsub:\n  select\n    when arg(1) = 1 then procedure\n    \
                  otherwise nop\n  end\n  return\n",
        stdout: "",
        stderr: "     5 *-*         procedure\n     1 *-* call sub 1\nError 17 running <PATH> \
                 line 5:  Unexpected PROCEDURE.\nError 17.1:  PROCEDURE is valid only when it is \
                 the first instruction executed after an internal CALL or function \
                 invocation.\n",
        exit_code: 239,
    },
    InlineCase {
        // The `IF`'s own clause is what the failure is attributed to, and its
        // echo carries the clause text up to the `THEN`.
        name: "if condition is not a logical value",
        program: "if 'x' then say 'y'\n",
        stdout: "",
        stderr: "     1 *-* if 'x' \nError 34 running <PATH> line 1:  Logical value not 0 or 1.\n\
                 Error 34.1:  Value of expression following IF keyword must be exactly \"0\" \
                 or \"1\"; found \"x\".\n",
        exit_code: 222,
    },
    InlineCase {
        // The trap offer is made once, by the activation the condition
        // unwound, whether or not the `IF` resolves its branch inside its own
        // clause step.
        name: "if condition raises into a signal on trap",
        program: "signal on syntax\nif 1/0 = 1 then say 'y'\nsay 'unreached'\nexit\n\
                  syntax:\nsay 'trapped' rc\n",
        stdout: "trapped 42\n",
        stderr: "",
        exit_code: 0,
    },
    InlineCase {
        // The `THEN` marker echoes at the `IF`'s own line and the consequence
        // two columns further in, both of them clauses in their own right.
        name: "if under trace i, true path",
        program: "trace i\nif 1 = 1 then say 'y'\nelse say 'n'\n",
        stdout: "y\n",
        stderr: "     2 *-* if 1 = 1 \n       >L>   \"1\"\n       >L>   \"1\"\n       \
                 >O>   \"=\" => \"1\"\n       >>>   \"1\"\n     2 *-*   then\n     \
                 2 *-*     say 'y'\n       >L>       \"y\"\n       >>>       \"y\"\n",
        exit_code: 0,
    },
    InlineCase {
        // The `ELSE` marker echoes at its own line, which is the false path's
        // whole observable difference from the true one.
        name: "if under trace r, false path",
        program: "trace r\nif 1 = 0 then say 'y'\nelse say 'n'\n",
        stdout: "n\n",
        stderr: "     2 *-* if 1 = 0 \n       >>>   \"0\"\n     3 *-*   else\n     \
                 3 *-*     say 'n'\n       >>>       \"n\"\n",
        exit_code: 0,
    },
    InlineCase {
        // A `LEAVE` from inside a true branch inside a loop body: the flow
        // has to escape the branch and be consumed by the loop.
        name: "if inside a loop body, leaving it",
        program: "n1 = 0\ndo i = 1 to 5\n  if i = 3 then leave\n  n1 = n1 + 1\nend\nsay n1\n",
        stdout: "2\n",
        stderr: "",
        exit_code: 0,
    },
    InlineCase {
        // A `DO` block as the whole true branch, with an `ELSE` after it: the
        // block's own resume lands on the `ELSE`, which must not run.
        name: "if whose true branch is a do block, with an else",
        program: "if 1 = 1 then do\n  say 'a'\n  say 'b'\nend\nelse say 'c'\nsay 'after'\n",
        stdout: "a\nb\nafter\n",
        stderr: "",
        exit_code: 0,
    },
    InlineCase {
        name: "if in a called label",
        program: "call sub\nsay result\nexit\nsub:\n  if 1 = 1 then say 'in sub'\n  return 7\n",
        stdout: "in sub\n7\n",
        stderr: "",
        exit_code: 0,
    },
    InlineCase {
        // **A boundary case, not a shape.** The `CALL ON` handler is queued by
        // the `IF`'s own condition and delivered at the `IF`'s clause
        // boundary, where it raises -- so the clause the failure is attributed
        // to is the one whose boundary ran the handler. Measured against the
        // oracle: `2 *-* if raiser() = 'V'`. A promoted clause that records
        // its own failure but not its boundary's prints only the handler's
        // line here, and no shape case can see that, because every shape case
        // asks what a branch does rather than what its clause unit owes.
        name: "a call on handler failing at an if's own boundary",
        program: "call on user zx name h\nif raiser() = 'V' then say 'then'\nelse say 'else'\n\
                  say 'after'\nexit\nraiser:\nraise user zx return 'V'\nh:\nsay 1/0\nreturn\n",
        stdout: "",
        stderr: "     9 *-*   say 1/0\n     2 *-* if raiser() = 'V' \nError 42 running <PATH> \
                 line 9:  Arithmetic overflow/underflow.\nError 42.3:  Arithmetic overflow; \
                 divisor must not be zero.\n",
        exit_code: 214,
    },
    InlineCase {
        // The same boundary one construct deeper, where the failure to record
        // is not merely silent: the enclosing `DO`'s own site wins the
        // first-wins race instead, so the wrong clause is echoed rather than
        // none. Measured against the oracle: `3 *-* if raiser() = 'V'`, not
        // `2 *-* do i = 1 to 1`.
        name: "a call on handler failing at an if's boundary inside a loop",
        program: "call on user zx name h\ndo i = 1 to 1\n  if raiser() = 'V' then say 'then'\n\
                  end\nsay 'after'\nexit\nraiser:\nraise user zx return 'V'\nh:\nzq = 1/0\nreturn\n",
        stdout: "",
        stderr: "    10 *-*     zq = 1/0\n     3 *-*   if raiser() = 'V' \nError 42 running \
                 <PATH> line 10:  Arithmetic overflow/underflow.\nError 42.3:  Arithmetic \
                 overflow; divisor must not be zero.\n",
        exit_code: 214,
    },
    InlineCase {
        // A branch does not inherit the first-instruction permission, so a
        // `PROCEDURE` reached through one is 17.1 -- the observable that says
        // `grant_procedure_permission` is spent identically under both
        // engines.
        name: "procedure reached through a true branch",
        program: "call sub 1\nexit\nsub:\n  if arg(1) = 1 then procedure\n  return\n",
        stdout: "",
        stderr: "     4 *-*       procedure\n     1 *-* call sub 1\nError 17 running <PATH> \
                 line 4:  Unexpected PROCEDURE.\nError 17.1:  PROCEDURE is valid only when it \
                 is the first instruction executed after an internal CALL or function \
                 invocation.\n",
        exit_code: 239,
    },
];

/// Every [`LOOP_CASES`] program on both engines, byte for byte, and against
/// the bytes the tree-walker produces for it.
#[test]
fn both_engines_agree_on_every_loop_shape() {
    compare_inline_cases(LOOP_CASES);
}

/// Every [`BRANCH_CASES`] program, the same way.
#[test]
fn both_engines_agree_on_every_branch_shape() {
    compare_inline_cases(BRANCH_CASES);
}

/// Where the case files this harness reads live.
const CASE_DIR: &str = "tests/ir_dual_cases";

/// The cases in [`CASE_DIR`], which are the ones written as data rather than
/// as a `const` table in this file (the plan's Tech Stack section: a data file
/// from Task 6 onward, and the three existing tables not retrofitted because
/// they hold witnesses nothing else catches).
///
/// Each stanza is one program, run on **both** engines, asserting the same two
/// halves `compare_inline_cases` does. The engines are compared against each
/// other inside the callback, because that is a property of the pair and has
/// no expected bytes to record; the tree-walker's own answer is what the
/// stanza's expected block holds, so a change to it shows as a diff rather
/// than as an assertion message.
///
/// **The expected bytes are the oracle's, so `REWRITE=1` must not reach
/// them.** That is `datadriven`'s whole idiom -- regenerate the expectation
/// from the implementation -- and it is exactly wrong here: it would turn an
/// oracle-measured expectation into a self-consistent one, in a diff that
/// looks like any other expectation update. The refusal below is the tripwire,
/// and the file's own header says how to regenerate a row instead.
///
/// A file whose subject is a deliberate deviation from the oracle says so in its
/// own header and names what its bytes are instead -- `loop-refusals`, whose
/// three forms the oracle implements and this crate declines. The refusal above
/// applies to it for the same reason: a regenerated expectation would agree with
/// whatever the code did.
#[test]
fn both_engines_agree_on_every_case_file() {
    assert!(
        std::env::var_os("REWRITE").is_none(),
        "REWRITE would replace this crate's oracle-measured expectations with whatever it \
         currently prints. Regenerate a row from the oracle instead -- {CASE_DIR}'s own header \
         has the command"
    );
    let mut cases = 0;
    datadriven::walk(CASE_DIR, |file| {
        file.run(|case| {
            cases += 1;
            assert_eq!(
                case.directive, "program",
                "unknown directive {:?}",
                case.directive
            );
            render_both_engines(&case.input)
        });
    });
    // A directory that produced no *stanzas* -- because it was emptied, renamed,
    // or because every stanza's directive stopped being recognised -- otherwise
    // passes this test with nothing run at all.
    assert!(
        cases > 0,
        "{CASE_DIR} produced no cases, so this test asserts nothing"
    );
}

/// Runs one case's program on both engines, asserts they agree, and renders
/// the tree-walker's answer in the tagged form the case files record.
///
/// **Every line is tagged, so no line of an expected block can be empty.** A
/// blank line is what ends such a block, and a program's own output may
/// contain one; the tag also keeps a trailing blank -- which an `IF` header's
/// `*-*` echo ends in, and which is compared -- away from the end of a line
/// that would otherwise look empty.
fn render_both_engines(program: &str) -> String {
    let text = program.as_bytes().to_vec();
    let tw = run(text.clone(), Engine::TreeWalker);
    let ir = run(text, Engine::Ir);
    assert_eq!(
        String::from_utf8_lossy(&tw.stdout),
        String::from_utf8_lossy(&ir.stdout),
        "stdout differs between engines"
    );
    assert_eq!(
        String::from_utf8_lossy(&tw.stderr),
        String::from_utf8_lossy(&ir.stderr),
        "stderr differs between engines"
    );
    assert_eq!(tw.exit_code, ir.exit_code, "exit status differs");
    assert_eq!(
        ir.chunks_refused, 0,
        "the ir arm refused the body and ran it on the tree-walker"
    );

    let mut out = format!("rc> {}\n", tw.exit_code);
    for line in String::from_utf8_lossy(&tw.stdout).lines() {
        out.push_str(&format!("out> {line}\n"));
    }
    for line in String::from_utf8_lossy(&tw.stderr).lines() {
        out.push_str(&format!("err> {line}\n"));
    }
    out
}

/// Runs each case twice and asserts both halves: the two engines against each
/// other, and the tree-walker against the bytes recorded for it.
fn compare_inline_cases(cases: &[InlineCase]) {
    assert!(!cases.is_empty(), "an empty case table asserts nothing");
    for case in cases {
        let text = case.program.as_bytes().to_vec();
        let tw = run(text.clone(), Engine::TreeWalker);
        let ir = run(text, Engine::Ir);
        assert_eq!(
            String::from_utf8_lossy(&tw.stdout),
            String::from_utf8_lossy(&ir.stdout),
            "[{}] stdout",
            case.name
        );
        assert_eq!(
            String::from_utf8_lossy(&tw.stderr),
            String::from_utf8_lossy(&ir.stderr),
            "[{}] stderr",
            case.name
        );
        assert_eq!(tw.exit_code, ir.exit_code, "[{}] exit status", case.name);
        assert_eq!(
            ir.chunks_refused, 0,
            "[{}] the ir arm refused the body and ran it on the tree-walker",
            case.name
        );
        assert_eq!(
            String::from_utf8_lossy(&tw.stdout),
            case.stdout,
            "[{}] the tree-walker's own stdout moved",
            case.name
        );
        assert_eq!(
            String::from_utf8_lossy(&tw.stderr),
            case.stderr.replace("<PATH>", INLINE_PATH),
            "[{}] the tree-walker's own trace moved",
            case.name
        );
        assert_eq!(
            tw.exit_code, case.exit_code,
            "[{}] the tree-walker's own exit status moved",
            case.name
        );
    }
}

/// One program the two engines are **known** to answer differently, with what
/// each of them says and what the oracle says.
struct KnownDivergence {
    name: &'static str,
    program: &'static str,
    /// The tree-walker's stdout.
    tree_walker: &'static str,
    /// The compiled stream's stdout, which in both rows below is also the
    /// oracle's.
    ir: &'static str,
}

/// Where the two engines disagree today, why it is not fixed here, and the
/// bytes that make it impossible for either side to move quietly.
///
/// **This does not weaken the sweep above.** That asserts no divergence over
/// its populations, unconditionally, and nothing here is in any of them: a
/// divergence needs a `CALL ON` handler that itself raises a second trapped
/// condition, and no corpus program or `ootest` row does that. So the choice
/// is not between catching these and not catching them; it is between writing
/// them down and leaving them undiscoverable.
///
/// **Both rows are one mechanism.** The tree-walker resolves an `IF`'s or a
/// `SELECT`'s chosen branch *inside* that instruction's own `step`, so
/// `step_in_temps_frame` runs a clause boundary when the whole construct
/// finishes. Where the oracle ends a taken branch with a synthetic
/// instruction, that boundary is the right one and both engines have it (the
/// compiled stream's `Op::EndBranch`). Where the oracle has no such
/// instruction -- an `IF` whose condition was false ran no branch, and an
/// `OTHERWISE` branch ends at the real `END` -- the tree-walker runs a
/// boundary the oracle does not, and the compiled stream, having no wrapper,
/// does not. **The compiled stream is the one that matches the oracle in both
/// rows.**
///
/// **A third member of the same family belongs to neither engine, so it is not
/// a row here -- and this is where a fixer will look for it.** A `DO` block that
/// is a branch body, whose last body clause queues a handler that itself
/// `RAISE`s, reports the second delivery's `SIGL` **one clause early on both
/// engines**. Measured against the oracle: `if 1 = 1 then do` / `zq = raiser()`
/// / `end` with the requeueing handler prints `G ran 5` on the oracle and
/// `G ran 4` on both arms; putting an unpromoted `CALL raiser` in the same slot
/// prints `G ran 6` against `G ran 5`, also on both arms.
///
/// It is the same elided instruction one construct over: the oracle ends that
/// branch at the block's real `END`, which has a boundary of its own, where both
/// engines deliver at the last body clause instead. **So it is pre-existing and
/// not any promotion's** -- the unpromoted-`CALL` spelling is the control that
/// says so -- and no test asserts it, because the two arms agree and this table
/// only holds programs where they do not.
///
/// Not fixed here because suppressing it means letting an instruction opt out
/// of its own clause boundary, which is the exact thing `clause.rs` is built
/// to make impossible; the honest fix is for the tree-walker to stop resolving
/// branches inside its own step, which is a change to `IF`/`SELECT`'s own
/// design rather than to this phase's.
const KNOWN_DIVERGENCES: &[KnownDivergence] = &[
    KnownDivergence {
        // `h` queues a second trapped condition. The oracle jumps over the
        // synthetic end-of-branch instruction on a false condition, so the
        // delivery waits for the next real clause: `after` then `G ran 4`.
        name: "a handler that queues again where no branch was taken",
        program: "call on user zx name h\ncall on user zy name g\nif raiser() = 'X' then nop\n\
                  say 'after'\nexit\nraiser:\nraise user zx return 'V'\nh:\n\
                  raise user zy return 1\ng:\nsay 'G ran' sigl\nreturn\n",
        tree_walker: "G ran 3\nafter\n",
        ir: "after\nG ran 4\n",
    },
    KnownDivergence {
        // The same handler inside an `OTHERWISE`, whose branch the oracle ends
        // at the `END` -- so the delivery is at the `END`'s own line, 7.
        name: "a handler that queues again at the end of an otherwise",
        program: "call on user zx name h\ncall on user zy name g\nselect\n\
                  when 1 = 0 then nop\notherwise\n   zq = raiser()\nend\nsay 'after'\nexit\n\
                  raiser:\nraise user zx return 'V'\nh:\nraise user zy return 1\ng:\n\
                  say 'G ran' sigl\nreturn\n",
        tree_walker: "G ran 6\nafter\n",
        ir: "G ran 7\nafter\n",
    },
];

/// Every [`KNOWN_DIVERGENCES`] row still says exactly what it claims.
///
/// Red if either engine's answer moves, in either direction -- including a
/// fix, which is what should delete the row rather than update it.
#[test]
fn the_known_engine_divergences_still_diverge_exactly_as_recorded() {
    assert!(
        !KNOWN_DIVERGENCES.is_empty(),
        "an empty table asserts nothing; delete the test with the last row"
    );
    for case in KNOWN_DIVERGENCES {
        let text = case.program.as_bytes().to_vec();
        let tw = run(text.clone(), Engine::TreeWalker);
        let ir = run(text, Engine::Ir);
        assert_eq!(
            String::from_utf8_lossy(&tw.stdout),
            case.tree_walker,
            "[{}] the tree-walker's own answer moved",
            case.name
        );
        assert_eq!(
            String::from_utf8_lossy(&ir.stdout),
            case.ir,
            "[{}] the compiled stream's own answer moved",
            case.name
        );
        assert_ne!(
            case.tree_walker, case.ir,
            "[{}] the two answers recorded here are the same, so this row \
             records no divergence at all",
            case.name
        );
    }
}

/// The path a program with no file behind it is reported under -- the rows
/// and bodies extracted from `ootest/`, and the inline program above. A
/// label, not a location, in the same spirit as `assertions.rs`'s `ROW_PATH`.
const INLINE_PATH: &str = "/nonexistent/ir-dual-case.rex";

/// One program, run once per engine.
struct Case {
    /// What a divergence report names this case by.
    name: String,
    /// The path the program is reported under, which reaches output through
    /// a raised condition's middle line and so has to be identical on both
    /// arms.
    path: String,
    text: Vec<u8>,
}

/// A named group of cases, drawn from one source.
struct Population {
    name: &'static str,
    cases: Vec<Case>,
}

/// The `ootest/ooRexx/base` suites this sweep draws programs from, sorted.
///
/// A literal, checked against the tree by
/// [`the_sweep_runs_every_ootest_suite_a_sibling_harness_runs`]: the sibling
/// harnesses in `tests/` name the suites they run, on disk, outside this
/// file, so a suite dropped from here is red rather than a quietly smaller
/// sweep. That is [`SUBSET_FILES`]'s arrangement exactly, one literal against
/// one external enumeration, and it is the arrangement a second in-repo list
/// does not have -- deleting a population and its name from a list beside it
/// is one edit, not two.
const OOTEST_SUITES: &[&str] = &["bif", "expressions", "keyword"];

/// The name of the population that is not an `ootest` suite.
const CORPUS_POPULATION: &str = "corpus";

/// The subset files the corpus population reads, in union order.
///
/// The same list `corpus.rs` keeps, for the same reason and pinned the same
/// way -- see [`the_dual_harness_reads_every_phase_subset_file`]. Duplicated
/// rather than shared because these are separate integration-test binaries
/// and neither can `mod` the other.
///
/// A literal here and a directory listing on the other side of the
/// assertion, never two literals: that asymmetry is the whole of what the pin
/// is worth.
const SUBSET_FILES: &[&str] = &[
    "phase-4a.txt",
    "phase-4b.txt",
    "phase-4c.txt",
    "phase-5a.txt",
    "phase-5b.txt",
];

fn corpus_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../corpus")
}

fn ootest_dir(suite: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../ootest/ooRexx/base")
        .join(suite)
}

/// The phase subset files that exist in the corpus directory, sorted.
///
/// Read from the directory rather than listed a second time, so the
/// assertion cannot be satisfied by a copy of [`SUBSET_FILES`] edited in the
/// same change.
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

/// Every corpus program named by a phase subset file, each once.
fn corpus_cases() -> Vec<Case> {
    let dir = corpus_dir();
    let mut seen = std::collections::HashSet::new();
    let mut cases = Vec::new();
    for name in SUBSET_FILES {
        let list_path = dir.join(name);
        let text = fs::read_to_string(&list_path)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", list_path.display()));
        for line in text.lines().map(str::trim) {
            if line.is_empty() || line.starts_with('#') || !seen.insert(line.to_string()) {
                continue;
            }
            let path = dir.join(line);
            // Canonicalised for the same reason `corpus.rs`'s runner passes
            // an absolute path: a raised condition's report names the
            // program, so a relative path would reach stderr. Both arms get
            // the identical string either way, so this is about the
            // comparison being run on realistic bytes rather than about the
            // two agreeing.
            let path = fs::canonicalize(&path)
                .unwrap_or_else(|e| panic!("cannot canonicalise {}: {e}", path.display()));
            let text =
                fs::read(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
            cases.push(Case {
                name: format!("corpus {line}"),
                path: path.to_string_lossy().into_owned(),
                text,
            });
        }
    }
    cases
}

/// The `NUMERIC FORM` keyword for a row's [`Form`].
fn form_keyword(form: Form) -> &'static str {
    match form {
        Form::Scientific => "SCIENTIFIC",
        Form::Engineering => "ENGINEERING",
    }
}

/// A program that establishes one row's state and then `SAY`s each clause.
///
/// The same shape `assertions.rs` and `bif_assertions.rs` build, and for the
/// same reason: `NUMERIC DIGITS`/`FORM` first, always, because a row
/// evaluated at the wrong precision exercises different code in both arms
/// rather than the code the row is about.
fn row_program(digits: u32, form: Form, prelude: &[String], clauses: &[&str]) -> Vec<u8> {
    let mut text = String::new();
    text.push_str(&format!("numeric digits {digits}\n"));
    text.push_str(&format!("numeric form {}\n", form_keyword(form)));
    for line in prelude {
        text.push_str(line);
        text.push('\n');
    }
    for clause in clauses {
        text.push_str("say ");
        text.push_str(clause);
        text.push('\n');
    }
    text.into_bytes()
}

/// Every `.testGroup` under `ootest/ooRexx/base/<suite>`, read once.
fn suite_sources(suite: &str) -> Vec<(String, String)> {
    let dir = ootest_dir(suite);
    let mut groups = find_test_groups(&dir);
    groups.sort();
    assert!(
        !groups.is_empty(),
        "no .testGroup files under {} -- the ootest checkout is missing base/{suite}",
        dir.display()
    );
    groups
        .iter()
        .map(|path| {
            let bytes =
                fs::read(path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
            let group = path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("group")
                .to_string();
            (group, String::from_utf8_lossy(&bytes).into_owned())
        })
        .collect()
}

fn case_of(row: &AssertionRow, index: usize) -> Case {
    // Both texts in one program, where `assertions.rs` says the expected
    // value only for a value row. The two arms are compared against each
    // other rather than against the row's own claim, so the extra clause
    // costs nothing and exercises a second expression per case.
    let clauses: Vec<&str> = if row.expect_raise.is_some() {
        vec![row.expr.as_str()]
    } else {
        vec![row.expr.as_str(), row.expected.as_str()]
    };
    Case {
        name: format!("{}::{}#{index}", row.group, row.method),
        path: INLINE_PATH.to_string(),
        text: row_program(row.digits, row.form, &row.prelude, &clauses),
    }
}

/// Every case, one population per entry of [`OOTEST_SUITES`] plus the
/// corpus.
///
/// The `match` has no catch-all: a suite named in that list with no arm here
/// cannot be turned into programs, and saying so loudly is the only honest
/// answer -- silently running three populations where four were declared is
/// the shrunken denominator this file exists to prevent.
fn populations() -> Vec<Population> {
    let mut out = vec![Population {
        name: CORPUS_POPULATION,
        cases: corpus_cases(),
    }];
    for suite in OOTEST_SUITES {
        let cases = match *suite {
            "expressions" => expression_cases(suite),
            "bif" => bif_cases(suite),
            "keyword" => keyword_cases(suite),
            other => panic!(
                "ootest suite base/{other} is declared in OOTEST_SUITES and nothing here turns \
                 its .testGroup files into programs"
            ),
        };
        out.push(Population { name: suite, cases });
    }
    out
}

fn expression_cases(suite: &str) -> Vec<Case> {
    let mut cases = Vec::new();
    for (group, source) in suite_sources(suite) {
        for (index, row) in extract_assertions(&group, &source).rows.iter().enumerate() {
            cases.push(case_of(row, index));
        }
    }
    cases
}

fn bif_cases(suite: &str) -> Vec<Case> {
    let mut cases = Vec::new();
    for (group, source) in suite_sources(suite) {
        let extraction = extract_bif(&group, &source);
        for (index, row) in extraction.rows.iter().enumerate() {
            cases.push(case_of(row, index));
        }
        for (index, row) in extraction.raises.iter().enumerate() {
            let operands: Vec<&str> = row.operands.iter().map(String::as_str).collect();
            cases.push(Case {
                name: format!("{}::{} raise #{index}", row.group, row.method),
                path: INLINE_PATH.to_string(),
                text: row_program(row.digits, row.form, &row.prelude, &operands),
            });
        }
    }
    cases
}

fn keyword_cases(suite: &str) -> Vec<Case> {
    let mut cases = Vec::new();
    for (group, source) in suite_sources(suite) {
        for body in extract_keyword(&group, &source).bodies {
            cases.push(Case {
                name: format!("{}::{}", body.group, body.method),
                path: INLINE_PATH.to_string(),
                text: body.program.clone().into_bytes(),
            });
        }
    }
    cases
}

/// Runs one case on both engines and describes the first difference, if any.
///
/// One `Case` and two runs, so the two arms cannot be given different
/// programs. Stdout, stderr and exit status are compared **unnormalised**:
/// there is no oracle here, so `corpus.rs`'s DEVIATION 0 does not apply and
/// a trace line's own indentation is required to match exactly.
fn compare(case: &Case) -> Option<String> {
    let tw = run_program(
        &case.path,
        case.text.clone(),
        Invocation::none().with_engine(Engine::TreeWalker),
    );
    let ir = run_program(
        &case.path,
        case.text.clone(),
        Invocation::none().with_engine(Engine::Ir),
    );

    if tw.exit_code != ir.exit_code {
        return Some(format!(
            "exit status: tree-walker {} vs ir {}",
            tw.exit_code, ir.exit_code
        ));
    }
    if tw.stdout != ir.stdout {
        return Some(format!(
            "stdout:\n  tree-walker {:?}\n  ir          {:?}",
            excerpt(&tw.stdout),
            excerpt(&ir.stdout)
        ));
    }
    if tw.stderr != ir.stderr {
        return Some(format!(
            "stderr:\n  tree-walker {:?}\n  ir          {:?}",
            excerpt(&tw.stderr),
            excerpt(&ir.stderr)
        ));
    }
    // Not a divergence between the arms, and that is exactly why it is
    // checked here rather than left to the comparison above: a body the
    // compiler refuses runs on the tree-walker under *both* arms, so the two
    // agree and the population passes while the engine under test never ran.
    if ir.chunks_refused != 0 {
        return Some(format!(
            "the ir arm refused a body {} times, running it on the tree-walker \
             instead",
            ir.chunks_refused
        ));
    }
    if tw.chunks_refused != 0 {
        return Some(format!(
            "the tree-walker arm counted {} refusals, and it compiles nothing \
             to refuse",
            tw.chunks_refused
        ));
    }
    None
}

/// Bounds a byte string to a readable excerpt, so a divergence stays
/// diagnosable without reprinting a program's whole output.
fn excerpt(bytes: &[u8]) -> String {
    const LIMIT: usize = 300;
    let text = String::from_utf8_lossy(bytes);
    if text.len() <= LIMIT {
        return text.into_owned();
    }
    format!("{}… ({} bytes)", &text[..LIMIT], bytes.len())
}

/// The dual harness reads **every** phase subset file the corpus has.
///
/// `corpus.rs`'s own `the_differential_reads_every_phase_subset_file` has the
/// argument: a file missing from the list is a phase whose programs are never
/// run, the sweep stays green over whatever is left, and nothing else here
/// can see it happen.
#[test]
fn the_dual_harness_reads_every_phase_subset_file() {
    assert_eq!(
        SUBSET_FILES,
        phase_subset_files_on_disk(),
        "the dual-engine sweep does not read every phase subset file in \
         rust/corpus/"
    );
}

/// Every `ootest` suite a sibling harness in `tests/` runs programs from,
/// read out of those files rather than listed here a second time.
///
/// The marker is the path the harnesses build their suite root from, which is
/// a literal in each of them. `ir_dual.rs` itself joins the suite name on
/// separately and so contributes nothing to this set, which is what stops the
/// answer being a copy of the question -- but the file is skipped by name as
/// well, so a later edit that spelled the path out here could not quietly
/// satisfy the pin either.
fn ootest_suites_sibling_harnesses_read() -> Vec<String> {
    const MARKER: &str = "ootest/ooRexx/base/";
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let entries =
        fs::read_dir(&dir).unwrap_or_else(|e| panic!("cannot read {}: {e}", dir.display()));
    let mut suites = std::collections::BTreeSet::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_none_or(|e| e != "rs")
            || path.file_name().is_some_and(|n| n == "ir_dual.rs")
        {
            continue;
        }
        let text = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
        for (offset, _) in text.match_indices(MARKER) {
            let tail = &text[offset + MARKER.len()..];
            let end = tail
                .find(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                .unwrap_or(tail.len());
            if end > 0 {
                suites.insert(tail[..end].to_string());
            }
        }
    }
    suites.into_iter().collect()
}

/// The sweep runs every `ootest` suite some other harness in this crate runs.
///
/// **What this rules out is a two-sided deletion**, which is the one that
/// happens: a population removed from [`populations`] *and* from the list
/// beside it leaves every other test in this file green over a sweep missing
/// thousands of programs. Pinning the list against a second list in the same
/// file does not rule that out, because both are one edit away. The sibling
/// harness sources are not.
#[test]
fn the_sweep_runs_every_ootest_suite_a_sibling_harness_runs() {
    let on_disk = ootest_suites_sibling_harnesses_read();
    assert!(
        !on_disk.is_empty(),
        "no sibling harness in tests/ names an ootest suite root, so this pin \
         found nothing to compare against and would accept any sweep at all"
    );
    assert_eq!(
        OOTEST_SUITES, on_disk,
        "the dual-engine sweep and this crate's other harnesses do not run the \
         same ootest suites. A suite only they run is one the two engines are \
         never compared on"
    );
}

/// Every population the tree calls for is built, and every one of them found
/// programs.
///
/// The expectation is derived, not restated: the corpus population is
/// required because `rust/corpus/` has phase subset files in it, and each
/// suite population because a sibling harness runs that suite. So deleting
/// either kind from [`populations`] is red without a second edit anywhere
/// being able to hide it.
///
/// Non-emptiness is separate from presence and catches the other shape: an
/// extractor pointed at the wrong directory, or a subset file naming nothing,
/// builds a population that exists and runs no programs.
#[test]
fn every_population_the_tree_calls_for_is_present_and_non_empty() {
    let mut required = vec![CORPUS_POPULATION.to_string()];
    assert!(
        !phase_subset_files_on_disk().is_empty(),
        "rust/corpus/ has no phase subset file, so nothing here requires the \
         corpus population to exist"
    );
    required.extend(ootest_suites_sibling_harnesses_read());
    required.sort();

    let populations = populations();
    let mut names: Vec<String> = populations.iter().map(|p| p.name.to_string()).collect();
    names.sort();
    assert_eq!(
        names, required,
        "a population the tree calls for is missing"
    );

    for population in &populations {
        assert!(
            !population.cases.is_empty(),
            "the {} population is empty, so the sweep asserts nothing about it",
            population.name
        );
    }
}

/// The sweep: every case of every population, both engines, byte for byte.
#[test]
fn both_engines_agree_across_every_population() {
    let populations = populations();
    let mut divergences = Vec::new();
    let mut compared = 0usize;
    for population in &populations {
        for case in &population.cases {
            compared += 1;
            if let Some(reason) = compare(case) {
                divergences.push(format!("[{}] {}: {reason}", population.name, case.name));
            }
        }
    }
    assert!(
        compared > 0,
        "the sweep compared nothing at all, which is a harness defect and not \
         an empty pass"
    );
    assert!(
        divergences.is_empty(),
        "{} of {compared} programs behave differently on the two engines:\n{}",
        divergences.len(),
        divergences.join("\n")
    );
}
