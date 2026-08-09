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

//! `compile`: the one pass that turns a body into a [`Chunk`] (the plan's
//! Decisions section: "compilation is whole-body and lazy: one body at a
//! time, on first entry, cached").

use rexx_parse::{CodeBody, InstructionKind};

use super::{Chunk, ChunkTooLarge, Op};
use crate::plan::Plan;

/// Compiles `body` into a [`Chunk`], once, whole.
///
/// Every instruction in `body.instructions` compiles (D21: "every
/// instruction compiles, nothing refuses" is about instructions). A `DO` or
/// `LOOP` becomes [`Op::Loop`], whose body clauses the driver steps; every
/// other instruction becomes [`Op::Generic`]. `plan` is not yet read: nothing
/// compiled here needs a name-to-slot answer, but a task that promotes an
/// assignment or a branch reads it to place the `EvalExpr`/`Clause` ops this
/// file only declares.
///
/// The one error is a machine width, not a language construct (the plan's
/// Decisions section: "the compiler has one error, and it is a machine
/// width"). Op indices are `u32`, so a body whose op stream would exceed
/// `u32::MAX` is refused rather than wrapped -- unreachable in practice at
/// this task, since `ops` and `body.instructions` are the same length and
/// nothing produces four billion instructions in a test, but the check is
/// the contract [`ChunkTooLarge`] documents, not a defence against a case
/// this task can exercise.
pub(crate) fn compile(body: &CodeBody, _plan: &Plan) -> Result<Chunk, ChunkTooLarge> {
    #[cfg(test)]
    count_compile_call();

    let mut ops = Vec::with_capacity(body.instructions.len());
    let mut op_of = Vec::with_capacity(body.instructions.len() + 1);
    for instruction in &body.instructions {
        let op_index = u32::try_from(ops.len()).map_err(|_| ChunkTooLarge { what: "op stream" })?;
        op_of.push(op_index);
        ops.push(match &instruction.kind {
            // `DO` and `LOOP` are the same construct under two spellings
            // (`step`'s own arm matches them together), so they compile the
            // same way. Everything else is still `Generic`.
            InstructionKind::Do(_) | InstructionKind::Loop(_) => Op::Loop,
            _ => Op::Generic,
        });
    }
    // One entry past the last instruction, pushed after the loop above:
    // `run_bounded`'s absorption guard is inclusive, so a construct's
    // resume point can be `end`, one past its last instruction, and a table
    // that stopped at `len - 1` would panic there instead of failing loudly
    // at compile.
    let end = u32::try_from(ops.len()).map_err(|_| ChunkTooLarge { what: "op stream" })?;
    op_of.push(end);

    Ok(Chunk {
        ops,
        op_of,
        registers: 0,
    })
}

// Test-only instrumentation: how many times `compile` has actually run. The
// chunk-cache test (`golden_tests.rs`) needs this to tell "the chunk cache
// compiled the body once" apart from "the second lookup happened not to
// fail" -- an implementation that recompiles on every call and returns an
// equal-looking `Chunk` would still pass a check that only inspects the
// result, and a `thread_local` counter is what lets the test inspect the
// call count instead.
#[cfg(test)]
thread_local! {
    static COMPILE_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
fn count_compile_call() {
    COMPILE_CALLS.with(|calls| calls.set(calls.get() + 1));
}

#[cfg(test)]
pub(crate) fn compile_calls() -> usize {
    COMPILE_CALLS.with(|calls| calls.get())
}
