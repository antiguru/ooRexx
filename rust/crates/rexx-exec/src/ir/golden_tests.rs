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

//! The golden tests for `ir::compile`: what an all-`Generic` body renders
//! as, the `op_of` boundary `run_bounded`'s inclusive absorption guard
//! needs, and the chunk cache's "compiled once" guarantee.

use std::rc::Rc;

use rexx_parse::parse_program;

use super::golden::render;
use super::{Chunk, ChunkTooLarge};
use crate::Interp;
use crate::plan::{BodyKey, Plan, ProgramId};

/// Parses `source`, builds its plan and compiles it -- the three steps
/// `Interp::chunk_for` otherwise runs one at a time, collapsed for tests
/// that only want the resulting `Chunk`. `#[cfg(test)]` only: this is not a
/// crate entry point.
fn compile_for_test(source: &[u8]) -> Result<Chunk, ChunkTooLarge> {
    let program = parse_program(source.to_vec()).expect("test program parses");
    let plan = Plan::build(&program.main, &program.symbols);
    super::compile(&program.main, &plan)
}

#[test]
fn every_instruction_of_an_all_generic_body_compiles_to_one_generic_op() {
    let chunk = compile_for_test(b"say 1\nsay 2\nn1 = 3\n").expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Generic\n\
         1: Generic\n\
         2: Generic\n"
    );
    // Nothing allocates a register until Task 4's allocator exists (the
    // plan's Decisions section), so none of this task's own chunks ever
    // claim one.
    assert_eq!(chunk.registers, 0);
}

/// `run_bounded`'s absorption guard is inclusive, so a construct's resume
/// point can be `end`, which is one past its last instruction. A map that
/// stops at `len - 1` panics there rather than at compile.
///
/// The expected count comes from the parsed body's own `instructions.len()`,
/// not from anything `Chunk` computes: an earlier version of this test
/// compared `chunk.op_of.len()` against a `Chunk::instruction_count()` that
/// was itself defined as `op_of.len() - 1`, which reduces to `x == x` and
/// cannot fail regardless of whether `compile` pushes the final entry.
/// Verified by removing that final push in `compile.rs` and confirming this
/// version goes red where the old one did not (recorded in the task report).
#[test]
fn the_instruction_map_has_an_entry_one_past_the_last_instruction() {
    let source = b"if 1 = 1 then say 'a'\nsay 'b'\n";
    let program = parse_program(source.to_vec()).expect("test program parses");
    let plan = Plan::build(&program.main, &program.symbols);
    let chunk = super::compile(&program.main, &plan).expect("compiles");
    assert_eq!(
        chunk.op_of.len(),
        program.main.instructions.len() + 1,
        "one entry per instruction plus the end entry"
    );
}

/// A program calling one routine in a loop compiles that routine's body
/// once: `chunk_for` must answer every later lookup from the cache rather
/// than recompiling it. This counts `compile`'s own call count rather than
/// inspecting what a lookup returns, because an implementation that
/// recompiles on every call and happens to return an equal-looking `Chunk`
/// would pass a check that only looks at the result -- `Rc::ptr_eq` below
/// pins the cache's *identity* guarantee, which the call count alone does
/// not, but neither one alone rules out both degenerate implementations at
/// once.
#[test]
fn chunk_for_compiles_a_body_once_across_repeated_lookups() {
    let program = parse_program(b"say 1\n".to_vec()).expect("test program parses");
    let key = BodyKey {
        program: ProgramId(0),
        directive: None,
    };
    let plan = Plan::build(&program.main, &program.symbols);

    let mut interp = Interp::new();
    let before = super::compile::compile_calls();

    let first = interp
        .chunk_for(key, &program.main, &plan)
        .expect("compiles");
    let second = interp.chunk_for(key, &program.main, &plan).expect("cached");
    let third = interp.chunk_for(key, &program.main, &plan).expect("cached");

    assert_eq!(
        super::compile::compile_calls() - before,
        1,
        "compile ran exactly once across three lookups under the same key"
    );
    assert!(
        Rc::ptr_eq(&first, &second),
        "second lookup is the same chunk"
    );
    assert!(Rc::ptr_eq(&first, &third), "third lookup is the same chunk");
}
