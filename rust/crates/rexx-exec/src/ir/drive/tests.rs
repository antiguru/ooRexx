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

use std::sync::{Mutex, MutexGuard};

use super::run_chunk_entries;
use crate::{Engine, Invocation, run_program};

/// The path these programs are reported under. Nothing reads it back: no
/// program below raises, so it never reaches a report.
const TEST_PATH: &str = "/nonexistent/ir-drive-test.rex";

/// Serialises every test in this module.
///
/// The counter is process-wide (see its own comment for why it cannot be a
/// `thread_local`), so a test taking a delta while another runs a program
/// under [`Engine::Ir`] sees both. **Every test here holds this, including
/// the ones that never read the counter**: what has to be excluded is a
/// concurrent *run*, not a concurrent read, and the two tests that only run
/// programs are exactly the ones that would corrupt someone else's delta.
/// Nothing outside this module runs under `Engine::Ir`, which is what keeps
/// the lock sufficient as well as necessary.
static COUNTER: Mutex<()> = Mutex::new(());

fn counter_lock() -> MutexGuard<'static, ()> {
    COUNTER
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// A program that enters three bodies: its own main body, the internal label
/// `sub` a `CALL` transfers to, and the `::ROUTINE` body a second `CALL`
/// resolves to.
///
/// The three are the two production paths into `run_activation` --
/// `Interp::run` for the first and `resolve_and_run_call` for the other two
/// -- so the count separates "the IR engine runs the main body" from "the IR
/// engine runs every body".
const THREE_BODIES: &[u8] = b"\
call sub
call rtn
exit
sub:
  return
::routine rtn
  return
";

#[test]
fn the_ir_engine_drives_every_body_the_program_enters() {
    let _guard = counter_lock();
    let before = run_chunk_entries();
    let outcome = run_program(
        TEST_PATH,
        THREE_BODIES.to_vec(),
        Invocation::none().with_engine(Engine::Ir),
    );
    assert_eq!(outcome.exit_code, 0, "stderr: {:?}", outcome.stderr);
    assert_eq!(
        run_chunk_entries() - before,
        3,
        "the IR engine drove {} bodies where the program has three: its main \
         body, the CALLed label, and the ::ROUTINE. Fewer means an entry point \
         into `run_activation` reached the tree-walker instead, which every \
         later promotion would then silently skip",
        run_chunk_entries() - before
    );
}

/// The negative control for the test above, and the reason it is a separate
/// test rather than a second assertion: a counter that never moved would
/// satisfy "the tree-walker drives nothing" on its own, and only the pair
/// says the count tracks the engine rather than the program.
#[test]
fn the_tree_walker_drives_no_chunk_at_all() {
    let _guard = counter_lock();
    let before = run_chunk_entries();
    let outcome = run_program(TEST_PATH, THREE_BODIES.to_vec(), Invocation::none());
    assert_eq!(outcome.exit_code, 0, "stderr: {:?}", outcome.stderr);
    assert_eq!(
        run_chunk_entries() - before,
        0,
        "the default engine drove a chunk, so `Engine::TreeWalker` is no longer \
         the default this phase's later tasks measure against"
    );
}

/// Nothing in an ordinary program overflows the compiled stream's index
/// widths, so the fallback path is never taken and the counter it bumps
/// stays at zero.
///
/// Asserted on both engines: under the tree-walker nothing is compiled at
/// all, and under the IR engine every body compiled. A non-zero count either
/// way would mean bodies were quietly running on the tree-walker while a
/// dual-engine comparison passed.
#[test]
fn no_body_is_refused_by_either_engine() {
    let _guard = counter_lock();
    for engine in [Engine::TreeWalker, Engine::Ir] {
        let outcome = run_program(
            TEST_PATH,
            THREE_BODIES.to_vec(),
            Invocation::none().with_engine(engine),
        );
        assert_eq!(
            outcome.chunks_refused, 0,
            "{engine:?} refused a body of a three-body program"
        );
    }
}
