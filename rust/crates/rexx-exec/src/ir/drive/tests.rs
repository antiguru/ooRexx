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

//! Engine selection: which bodies the driver actually runs.
//!
//! **Output cannot answer this and these tests do not ask it to.** Every op
//! resolves its clause through the same functions the tree-walker's own loop
//! calls, so the two engines produce identical bytes on every program -- a
//! selection test comparing output would stay green with selection deleted.
//! What separates them is what the driver did: which bodies it drove
//! ([`super::run_chunk_entries`]) and which clauses it stepped
//! ([`super::clause_op_entries`]).
//!
//! **Both counting tests name their engine and neither reads a default**, so
//! what they assert stays true whatever the default becomes. Which engine the
//! default *is* belongs to `Invocation`, and `invocation.rs` asserts it there.

use super::{clause_op_entries, run_chunk_entries};
use crate::{Engine, Invocation, Outcome, execute, run_program};

/// The path these programs are reported under. Nothing reads it back: no
/// program below raises, so it never reaches a report.
const TEST_PATH: &str = "/nonexistent/ir-drive-test.rex";

/// A program that enters three bodies: its own main body, the internal label
/// `sub` a `CALL` transfers to, and the `::ROUTINE` body a second `CALL`
/// resolves to.
///
/// The main body is entered by the program starting; the other two are
/// entered by a call being resolved. So a count of three separates "the IR
/// engine runs the body a program starts in" from "the IR engine runs every
/// body an activation is ever pushed for", which is the distinction engine
/// selection has to get right at both.
const THREE_BODIES: &[u8] = b"\
call sub
call rtn
exit
sub:
  return
::routine rtn
  return
";

/// Runs [`THREE_BODIES`] on `engine`, on **this** thread.
///
/// `execute` rather than `run_program`, and that is what makes the count
/// exact: `run_program` runs the interpreter on a thread of its own, and
/// `run_chunk`'s counter is per thread. Everything `run_program` does to an
/// `Invocation` happens here too, so the `Engine` still travels the whole
/// production route from `Invocation` to `Interp::engine`. The stack
/// `run_program` sizes is not needed for a program three shallow bodies deep.
fn drive(engine: Engine) -> (Outcome, usize) {
    let before = run_chunk_entries();
    let outcome = execute(
        TEST_PATH,
        THREE_BODIES.to_vec(),
        false,
        Invocation::none().with_engine(engine),
    );
    (outcome, run_chunk_entries() - before)
}

#[test]
fn the_ir_engine_drives_every_body_the_program_enters() {
    let (outcome, driven) = drive(Engine::Ir);
    assert_eq!(outcome.exit_code, 0, "stderr: {:?}", outcome.stderr);
    assert_eq!(
        driven, 3,
        "the IR engine drove {driven} chunks where the program has three \
         bodies: its main body, the CALLed label, and the ::ROUTINE. Fewer \
         means a way of entering an activation reached the tree-walker \
         instead, which every later promotion would then silently skip"
    );
}

/// The negative control for the test above, and the reason it is a separate
/// test rather than a second assertion: a counter that never moved would
/// satisfy "the tree-walker drives nothing" on its own, and only the pair
/// says the count tracks the engine rather than the program.
#[test]
fn the_tree_walker_drives_no_chunk_at_all() {
    let (outcome, driven) = drive(Engine::TreeWalker);
    assert_eq!(outcome.exit_code, 0, "stderr: {:?}", outcome.stderr);
    assert_eq!(
        driven, 0,
        "the tree-walker drove {driven} chunks, so the engine choice no longer \
         decides which driver runs a body"
    );
}

/// A three-pass loop over a one-clause body: the loop's `DO` clause and each
/// of the three body clauses are stepped from the chunk.
///
/// **This is the only observable that separates a promoted `DO`/`LOOP` from
/// an unpromoted one**, and that is why it is a count rather than a
/// comparison of output. The construct is resolved by the same `run_loop`
/// either way, so both engines print the same bytes on every program; what
/// changes is whether the body's clauses reach the compiled stream at all.
/// With the body left on the tree-walker the count is 1 -- the `DO` clause
/// alone -- and every promotion after this one would then silently skip
/// anything written inside a loop.
///
/// The `END` clause is not in the count and is not missing from it:
/// `run_bounded`'s range stops before `END`, and the loop's own `Flow::Goto`
/// resumes one past it, so no engine ever steps it.
#[test]
fn the_ir_engine_steps_a_loop_body_from_the_chunk() {
    const COUNTED_LOOP: &[u8] = b"do i = 1 to 3\n  nop\nend\n";

    let before = clause_op_entries();
    let outcome = execute(
        TEST_PATH,
        COUNTED_LOOP.to_vec(),
        false,
        Invocation::none().with_engine(Engine::Ir),
    );
    let stepped = clause_op_entries() - before;
    assert_eq!(outcome.exit_code, 0, "stderr: {:?}", outcome.stderr);
    assert_eq!(
        stepped, 4,
        "the IR engine stepped {stepped} clauses from the chunk where a \
         three-pass loop over a one-clause body has four: its own DO clause \
         and one body clause per pass"
    );
}

/// A `DO ... END` block's two body clauses are stepped from the chunk too.
///
/// **A separate test from the counted loop's, because `LoopKind::Simple` is a
/// separate arm of `run_loop` with a `run_bounded` call of its own**, and the
/// counted loop's count cannot see it: leaving that one arm on the tree-walker
/// leaves every other test in the workspace green, including the dual-engine
/// sweep, which cannot tell the two drivers apart because both produce
/// identical bytes.
///
/// **At the top of the body, not inside an `IF`.** `If` steps its own branch
/// through the tree-walker (that promotion is not this task's), so a block
/// written as `if 1 = 1 then do ... end` never reaches the stream at all and a
/// count taken over one would be satisfied by the `IF`, not by the block.
#[test]
fn the_ir_engine_steps_a_simple_blocks_body_from_the_chunk() {
    let before = clause_op_entries();
    let outcome = execute(
        TEST_PATH,
        b"do\n  nop\n  nop\nend\n".to_vec(),
        false,
        Invocation::none().with_engine(Engine::Ir),
    );
    let stepped = clause_op_entries() - before;
    assert_eq!(outcome.exit_code, 0, "stderr: {:?}", outcome.stderr);
    assert_eq!(
        stepped, 3,
        "the IR engine stepped {stepped} clauses from the chunk where a block \
         over two clauses has three: its own DO clause and both body clauses"
    );
}

/// An `IF`'s chosen branch is stepped from the chunk, on both paths.
///
/// **The only observable that separates a promoted `IF` from an unpromoted
/// one**, and that is why it is a count. Both engines print the same bytes
/// for every program -- the condition is evaluated by the same
/// `eval_if_condition` either way -- so what changes is whether the branch's
/// clauses reach the compiled stream at all. With the branch left on the
/// tree-walker the count is 2 on each path: the `IF` clause and the one
/// clause after the whole construct, with everything inside the branch
/// reached through `run_bounded`'s tree-walker arm and counted nowhere.
///
/// Both paths, because they are different mechanisms: the true path falls
/// into the branch and leaves it by the branch-end jump, and the false path
/// gets there by the `JumpUnless`. A count taken on one alone is satisfied by
/// an implementation that only flattened the other.
#[test]
fn the_ir_engine_steps_an_ifs_chosen_branch_from_the_chunk() {
    // `IF`, `THEN`, `say 'a'`, `ELSE`, `say 'b'`, `say 'c'` -- four clauses
    // run on either path, and which four is what differs.
    for (condition, expected, path) in [("1 = 1", 4, "then"), ("1 = 0", 4, "else")] {
        let program = format!("if {condition} then say 'a'\nelse say 'b'\nsay 'c'\n");
        let before = clause_op_entries();
        let outcome = execute(
            TEST_PATH,
            program.into_bytes(),
            false,
            Invocation::none().with_engine(Engine::Ir),
        );
        let stepped = clause_op_entries() - before;
        assert_eq!(outcome.exit_code, 0, "stderr: {:?}", outcome.stderr);
        assert_eq!(
            stepped, expected,
            "the IR engine stepped {stepped} clauses from the chunk on the {path} path, where \
             the IF's own clause, the branch marker, the branch body and the clause after the \
             whole construct are four"
        );
    }
}

/// A `SELECT`'s own header, every listed `WHEN` it tests, and the branch that
/// wins are all stepped from the chunk.
///
/// **The only observable that separates a promoted `SELECT` from an unpromoted
/// one**, for the reason the `IF` count above is a count: the construct is
/// resolved through the same `scan_when`, `when_targets`, `select_escape` and
/// `leave_select` either way, so both engines print the same bytes for every
/// program and nothing in the output says which drove it. With the whole
/// construct left on the tree-walker the count is 2 on either path -- the
/// `SELECT`'s own clause and the one clause after it -- with the scan, the
/// branch and the `END` reached through `run_bounded`'s tree-walker arm and
/// counted nowhere.
///
/// Both paths, because they are different mechanisms and different ops: a
/// matched `WHEN` falls into its branch past an `EnterWhen` and leaves by its
/// frame's own end, and `OTHERWISE` is reached by the last `WHEN`'s
/// `JumpUnless` landing on an `EnterOtherwise`. A count taken on one alone is
/// satisfied by an implementation that only flattened the other.
#[test]
fn the_ir_engine_steps_a_selects_chosen_branch_from_the_chunk() {
    // The matched path: the `SELECT` clause, both `WHEN` clauses, the winning
    // `THEN` marker and its body, and the clause after the whole construct.
    // The `END` is not among them and is not missing: one true `WHEN` resumes
    // past it.
    //
    // The `OTHERWISE` path: the `SELECT` clause, the one `WHEN` clause, the
    // `OTHERWISE` marker and its body, the `END` -- which this path *does*
    // reach, and does nothing at -- and the clause after.
    for (program, expected, path) in [
        (
            "select\n  when 1 = 0 then nop\n  when 2 = 2 then nop\n  otherwise nop\nend\nnop\n",
            6,
            "matched when",
        ),
        (
            "select\n  when 1 = 0 then nop\n  otherwise nop\nend\nnop\n",
            6,
            "otherwise",
        ),
    ] {
        let before = clause_op_entries();
        let outcome = execute(
            TEST_PATH,
            program.as_bytes().to_vec(),
            false,
            Invocation::none().with_engine(Engine::Ir),
        );
        let stepped = clause_op_entries() - before;
        assert_eq!(outcome.exit_code, 0, "stderr: {:?}", outcome.stderr);
        assert_eq!(
            stepped, expected,
            "the IR engine stepped {stepped} clauses from the chunk on the {path} path, where \
             the SELECT's own clause, each listed WHEN's clause, the chosen branch's clauses and \
             the clause after the whole construct are {expected}"
        );
    }
}

/// The negative control for the test above: the tree-walker steps nothing
/// from a chunk, so a count that never moved would satisfy it on its own.
#[test]
fn the_tree_walker_steps_no_clause_from_a_chunk() {
    let before = clause_op_entries();
    let outcome = execute(
        TEST_PATH,
        b"do i = 1 to 3\n  nop\nend\n".to_vec(),
        false,
        Invocation::none().with_engine(Engine::TreeWalker),
    );
    let stepped = clause_op_entries() - before;
    assert_eq!(outcome.exit_code, 0, "stderr: {:?}", outcome.stderr);
    assert_eq!(
        stepped, 0,
        "the tree-walker stepped {stepped} clauses from a chunk, so the engine \
         choice no longer decides which driver steps a clause"
    );
}

/// Nothing in an ordinary program overflows the compiled stream's index
/// widths, so the refusal path is never taken and the counter it bumps stays
/// at zero.
///
/// Asserted on both engines: under the tree-walker nothing is compiled at
/// all, and under the IR engine every body compiled. A non-zero count either
/// way would mean bodies were quietly running on the tree-walker while a
/// dual-engine comparison passed.
///
/// Through `run_program`, unlike the two above, because what it reads is the
/// public `Outcome` field rather than the per-thread counter -- so this also
/// covers the descent from `run_program` through its own thread, which
/// `execute` alone does not.
#[test]
fn no_body_is_refused_by_either_engine() {
    for engine in [Engine::TreeWalker, Engine::Ir] {
        let outcome = run_program(
            TEST_PATH,
            THREE_BODIES.to_vec(),
            Invocation::none().with_engine(engine),
        );
        assert_eq!(
            outcome.chunks_refused, 0,
            "{engine:?} refused a body of a three-body program {} times",
            outcome.chunks_refused
        );
    }
}
