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
//! compiles to `Op::Generic`, which delegates the clause back to the
//! tree-walker's own clause unit, so the two engines produce identical bytes
//! on every program -- a selection test comparing output would stay green
//! with selection deleted. What separates them is whether `run_chunk` ran at
//! all, which is what [`super::run_chunk_entries`] counts.
//!
//! **Both counting tests name their engine and neither reads a default**, so
//! what they assert stays true whatever the default becomes. Which engine the
//! default *is* belongs to `Invocation`, and `invocation.rs` asserts it there.

use super::run_chunk_entries;
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
