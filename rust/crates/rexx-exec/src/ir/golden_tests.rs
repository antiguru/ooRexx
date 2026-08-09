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

//! The golden tests for `ir::compile`: what an unpromoted body and a
//! compiled loop each render as, the `op_of` boundary `run_bounded`'s
//! inclusive absorption guard needs, and the chunk cache's "compiled once"
//! guarantee.

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
        "0: Generic index=0\n\
         1: Generic index=1\n\
         2: Generic index=2\n"
    );
    // Nothing here addresses a register, so the chunk reserves none.
    assert_eq!(chunk.registers, 0);
}

/// The compiled form of the plan's own example loop.
///
/// Three instructions and three ops: the `DO` is [`super::Op::Loop`], and the
/// body clause and the `END` are still `Generic`. The `END` op is never
/// reached -- `run_bounded`'s range stops before it and the loop's own resume
/// is one past it -- and it is emitted anyway because `op_of` is indexed by
/// instruction, so an instruction without an op would shift every later entry.
#[test]
fn a_counted_loop_compiles_its_do_to_a_loop_op_and_its_body_to_generic() {
    let chunk = compile_for_test(b"do i = 1 to 3\n  nop\nend\n").expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Loop index=0\n\
         1: Generic index=1\n\
         2: Generic index=2\n"
    );
    // The loop is driven by `run_loop`, which holds its control value in a
    // `LoopState` of its own rather than in the chunk's register region, so
    // nothing here allocates one.
    assert_eq!(chunk.registers, 0);
}

/// The compiled `IF` with an `ELSE`, which is the shape the whole promotion
/// is about: both paths are jumps in one stream where the tree-walker splits
/// them across two engines.
///
/// The six instructions are `IF`, `THEN`, `say 'a'`, `ELSE`, `say 'b'`,
/// `say 'c'`. Three of the nine ops are the `IF`'s own clause -- the region
/// that evaluates the condition and branches on it -- and one more is the
/// jump that ends the true branch, which sits between the last op of the
/// true branch and the `ELSE`'s own op so that neither instruction's entry
/// in `op_of` moves.
///
/// The two jump targets are the two things a reader should check by eye:
/// `JumpUnless` goes to op 6, the `ELSE` marker, and `Jump` goes to op 8,
/// `say 'c'`, past the whole `ELSE` branch.
///
/// **`op_of[3]` is 5 and not 6**, which is the other half of the same
/// mechanism: the `ELSE`'s entry in the resume table is the branch-end jump
/// in front of it, so a `Flow::Goto(3)` -- a nested `DO` block resuming at
/// exactly the branch's boundary -- skips the `ELSE` the way falling off the
/// end of the branch does. The false path is the one arrival that must run
/// it, and that is the `JumpUnless` above, resolved against the `ELSE`'s own
/// first op instead.
#[test]
fn an_if_with_an_else_compiles_to_a_clause_region_and_two_jumps() {
    let chunk =
        compile_for_test(b"if 1 = 1 then say 'a'\nelse say 'b'\nsay 'c'\n").expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=3\n\
         1: EvalExpr index=0 slot=0 dst=0\n\
         2: JumpUnless reg=0 target=6\n\
         3: Generic index=1\n\
         4: Generic index=2\n\
         5: Jump target=8\n\
         6: Generic index=3\n\
         7: Generic index=4\n\
         8: Generic index=5\n"
    );
    // One register, allocated for the condition and released at the clause's
    // own end -- so a body with two `IF`s reserves one, not two.
    assert_eq!(chunk.registers, 1);
    assert_eq!(chunk.op_of, vec![0, 3, 4, 5, 7, 8, 9]);
}

/// Without an `ELSE` the true branch falls straight through to where the
/// false path lands, so **no jump is emitted at all**: the two targets are
/// the same instruction, and an op that jumps to where control was already
/// going is one the driver would run for nothing.
#[test]
fn an_if_with_no_else_emits_no_branch_end_jump() {
    let chunk = compile_for_test(b"if 1 = 0 then say 'a'\nsay 'b'\n").expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=3\n\
         1: EvalExpr index=0 slot=0 dst=0\n\
         2: JumpUnless reg=0 target=5\n\
         3: Generic index=1\n\
         4: Generic index=2\n\
         5: Generic index=3\n"
    );
    assert_eq!(chunk.registers, 1);
}

/// Two `IF`s in one body reuse the same register, which is what the
/// allocator's stack discipline buys over the spike's withdrawn monotonic
/// counter: under that counter the inner `IF` would take register 1 and the
/// high-water mark would grow with a body's length rather than with its depth.
///
/// **What this does not pin, stated because an earlier version of this comment
/// claimed it did.** It says nothing about a `Mark` carrying a *position*: a
/// `mark()` that always answered `Mark(0)` leaves this test green, because
/// nothing this compiler emits yet allocates in an enclosing scope and so
/// every mark taken here really is zero. The nesting below is nesting of `IF`s
/// in the source, not of live registers. What pins the mark's position is
/// `compile::tests::a_released_register_is_handed_out_again_and_a_nested_one_is_not`,
/// which drives the allocator directly and does redden under that mutation --
/// and the first construct to allocate in an enclosing scope, which the plan's
/// Decisions section names as a loop's control value, is what will make it
/// observable in an emitted stream.
#[test]
fn nested_ifs_reuse_one_register() {
    let chunk =
        compile_for_test(b"if 1 = 1 then\n  if 2 = 2 then say 'a'\nsay 'b'\n").expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=3\n\
         1: EvalExpr index=0 slot=0 dst=0\n\
         2: JumpUnless reg=0 target=9\n\
         3: Generic index=1\n\
         4: Clause index=2 end=7\n\
         5: EvalExpr index=2 slot=0 dst=0\n\
         6: JumpUnless reg=0 target=9\n\
         7: Generic index=3\n\
         8: Generic index=4\n\
         9: Generic index=5\n"
    );
    assert_eq!(
        chunk.registers, 1,
        "the inner IF reuses the register the outer one released"
    );
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
