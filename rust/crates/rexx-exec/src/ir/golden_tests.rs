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
use crate::trace::{ChunkTrace, TraceMode};

/// Parses `source`, builds its plan and compiles it **under the setting every
/// activation starts at** -- the three steps `Interp::chunk_for` otherwise
/// runs one at a time, collapsed for tests that only want the resulting
/// `Chunk`. `#[cfg(test)]` only: this is not a crate entry point.
fn compile_for_test(source: &[u8]) -> Result<Chunk, ChunkTooLarge> {
    compile_for_test_under(source, ChunkTrace::of(TraceMode::NORMAL))
}

/// [`compile_for_test`] under a named trace setting, which is an input to
/// what `compile` emits (D23).
fn compile_for_test_under(source: &[u8], trace: ChunkTrace) -> Result<Chunk, ChunkTooLarge> {
    let program = parse_program(source.to_vec()).expect("test program parses");
    let plan = Plan::build(&program.main, &program.symbols);
    super::compile(&program.main, &plan, trace)
}

/// The setting `TRACE R` puts in force, as far as compilation can see it.
///
/// Through `mode_from_setting` rather than a hand-built `TraceMode`, so a case
/// below is compiling for the setting a `trace r` clause would actually
/// produce.
fn traced() -> ChunkTrace {
    ChunkTrace::of(crate::trace::mode_from_setting(b"r").expect("R is a valid TRACE setting"))
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
/// `say 'c'`. Three ops are the `IF`'s own clause -- the region that evaluates
/// the condition and branches on it -- and two more close the true branch,
/// sitting between its last op and the `ELSE`'s own op so that neither
/// instruction's entry in `op_of` moves.
///
/// The three things a reader should check by eye are the two jump targets and
/// what sits between them: `JumpUnless` goes to op 7, the `ELSE` marker, and
/// `Jump` goes to op 9, `say 'c'`, past the whole `ELSE` branch, with
/// `EndBranch` at op 5 -- the boundary a promoted construct owes once the
/// branch it chose has finished.
///
/// **The `JumpUnless` target is past that `EndBranch`, and that is measured**:
/// the oracle closes a *taken* branch with a synthetic instruction and jumps
/// over it on the false path, where an `IF` whose condition is false runs no
/// such boundary at all.
///
/// **`op_of[3]` is 5 and not 7**, which is the other half of the same
/// mechanism: the `ELSE`'s entry in the resume table is the pair of ops that
/// close the branch in front of it, so a `Flow::Goto(3)` -- a nested `DO`
/// block resuming at
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
         2: JumpUnless reg=0 target=7\n\
         3: Generic index=1\n\
         4: Generic index=2\n\
         5: EndBranch\n\
         6: Jump target=9\n\
         7: Generic index=3\n\
         8: Generic index=4\n\
         9: Generic index=5\n"
    );
    // One register, allocated for the condition and released at the clause's
    // own end -- so a body with two `IF`s reserves one, not two.
    assert_eq!(chunk.registers, 1);
    assert_eq!(chunk.op_of, vec![0, 3, 4, 5, 8, 9, 10]);
}

/// Without an `ELSE` **no jump is emitted at all**: the two targets are the
/// same instruction, and an op that jumps to where control was already going
/// is one the driver would run for nothing.
///
/// The end-of-branch boundary is still emitted, and is still the true path's
/// alone: the branch falls into `EndBranch` at op 5 and the `JumpUnless` goes
/// past it to op 6.
#[test]
fn an_if_with_no_else_emits_no_branch_end_jump() {
    let chunk = compile_for_test(b"if 1 = 0 then say 'a'\nsay 'b'\n").expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=3\n\
         1: EvalExpr index=0 slot=0 dst=0\n\
         2: JumpUnless reg=0 target=6\n\
         3: Generic index=1\n\
         4: Generic index=2\n\
         5: EndBranch\n\
         6: Generic index=3\n"
    );
    assert_eq!(chunk.registers, 1);
}

/// **One `EndBranch` per `IF`, inner before outer.** Control leaves the inner
/// branch first, and the oracle runs one synthetic end-of-branch instruction
/// per branch rather than one per position -- so a handler delivered at the
/// inner one that queues again has the outer one left to deliver it. Both
/// `JumpUnless` targets are past both, because neither false path ran a
/// branch.
///
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
         2: JumpUnless reg=0 target=11\n\
         3: Generic index=1\n\
         4: Clause index=2 end=7\n\
         5: EvalExpr index=2 slot=0 dst=0\n\
         6: JumpUnless reg=0 target=11\n\
         7: Generic index=3\n\
         8: Generic index=4\n\
         9: EndBranch\n\
         10: EndBranch\n\
         11: Generic index=5\n"
    );
    assert_eq!(
        chunk.registers, 1,
        "the inner IF reuses the register the outer one released"
    );
}

/// The compiled `SELECT` with an `OTHERWISE`: one clause region for the header
/// and one per listed `WHEN`, chained by the `JumpUnless` each `WHEN` ends
/// with, and a frame opened over whichever branch wins.
///
/// The eleven instructions are `SELECT`, `WHEN`, `THEN`, `say 'a'`, `WHEN`,
/// `THEN`, `say 'b'`, `OTHERWISE`, `say 'o'`, `END`, `say 'after'`.
///
/// The three things a reader should check by eye are the jump targets:
///
/// * the first `WHEN`'s `JumpUnless` goes to op 8, the **second `WHEN`'s own
///   clause region**, which is the scan continuing;
/// * the second `WHEN`'s goes to op 14, the `EnterOtherwise` in front of the
///   `OTHERWISE` marker, which is the scan running out;
/// * and nothing jumps past a branch, because a branch is left by the frame
///   `EnterWhen` opened rather than by an op -- reaching op 14 by falling out
///   of the second `WHEN`'s branch is that branch's `op_end`, and the driver
///   closes the frame there instead of running the op.
///
/// **`op_of[7]` is 14 and not 15**, which is the other half of the same
/// mechanism: an absorbed `WHEN CASE`'s escape landing exactly on the
/// `OTHERWISE` marker has to open the frame the marker's branch runs under,
/// and that is what putting `EnterOtherwise` at the resume entry does.
#[test]
fn a_select_with_an_otherwise_compiles_to_a_scan_chain_and_two_frames() {
    let chunk = compile_for_test(
        b"select\n  when 1 = 0 then say 'a'\n  when 2 = 2 then say 'b'\n  otherwise say 'o'\n\
          end\nsay 'after'\n",
    )
    .expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=1\n\
         1: SelectCaseText index=0 case=-\n\
         2: Clause index=1 end=5\n\
         3: WhenTest index=1 case=- dst=0\n\
         4: JumpUnless reg=0 target=8\n\
         5: EnterWhen select=0 when=1\n\
         6: Generic index=2\n\
         7: Generic index=3\n\
         8: Clause index=4 end=11\n\
         9: WhenTest index=4 case=- dst=0\n\
         10: JumpUnless reg=0 target=14\n\
         11: EnterWhen select=0 when=4\n\
         12: Generic index=5\n\
         13: Generic index=6\n\
         14: EnterOtherwise select=0\n\
         15: Generic index=7\n\
         16: Generic index=8\n\
         17: Generic index=9\n\
         18: Generic index=10\n"
    );
    assert_eq!(
        chunk.registers, 1,
        "the second WHEN reuses the register the first one released"
    );
    assert_eq!(chunk.op_of, vec![0, 2, 6, 7, 8, 12, 13, 14, 16, 17, 18, 19]);
}

/// A `SELECT CASE`'s own value is allocated in the **enclosing** scope, so the
/// register a `WHEN` takes for its own answer cannot reclaim it.
///
/// **This is the first emitted stream where `Mark` carrying a position is
/// observable**, which `nested_ifs_reuse_one_register` says it is not for an
/// `IF`. The `CASE` value is register 0 and outlives every member clause --
/// the second `WHEN` is tested after the first `WHEN`'s branch has already run
/// -- while the two `WHEN`s share register 1 between them. An allocator whose
/// `mark()` always answered `Mark(0)` would hand register 0 back to the first
/// `WHEN` and compare every later `WHEN` against whatever that left behind.
#[test]
fn a_select_cases_own_value_outlives_the_registers_its_whens_take() {
    let chunk = compile_for_test(
        b"select case 1 + 1\n  when 1 then say 'a'\n  when 2 then say 'b'\nend\nsay 'after'\n",
    )
    .expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=2\n\
         1: EvalExpr index=0 slot=0 dst=0\n\
         2: SelectCaseText index=0 case=0\n\
         3: Clause index=1 end=6\n\
         4: WhenTest index=1 case=0 dst=1\n\
         5: JumpUnless reg=1 target=9\n\
         6: EnterWhen select=0 when=1\n\
         7: Generic index=2\n\
         8: Generic index=3\n\
         9: Clause index=4 end=12\n\
         10: WhenTest index=4 case=0 dst=1\n\
         11: JumpUnless reg=1 target=15\n\
         12: EnterWhen select=0 when=4\n\
         13: Generic index=5\n\
         14: Generic index=6\n\
         15: Generic index=7\n\
         16: Generic index=8\n"
    );
    assert_eq!(
        chunk.registers, 2,
        "the CASE value and one WHEN answer are live at once, and never more"
    );
}

/// Without an `OTHERWISE` the scan runs out onto the `END`, whose own 7.3 is
/// what "every WHEN was false" means -- so the last `WHEN`'s `JumpUnless`
/// names the `END`'s **own** op and no frame is open when it runs.
///
/// The neighbouring case to the one above, and it is what says the
/// `EnterOtherwise` there belongs to the `OTHERWISE` rather than being emitted
/// for every `SELECT`: a driver that opened a frame here would have one still
/// standing when the `END` raised.
#[test]
fn a_select_with_no_otherwise_scans_out_onto_its_own_end() {
    let chunk = compile_for_test(b"select\n  when 1 = 0 then say 'a'\nend\nsay 'after'\n")
        .expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=1\n\
         1: SelectCaseText index=0 case=-\n\
         2: Clause index=1 end=5\n\
         3: WhenTest index=1 case=- dst=0\n\
         4: JumpUnless reg=0 target=8\n\
         5: EnterWhen select=0 when=1\n\
         6: Generic index=2\n\
         7: Generic index=3\n\
         8: Generic index=4\n\
         9: Generic index=5\n"
    );
}

/// The same `IF` compiled under `TRACE R`: the clause echo is an **op** of the
/// clause's own region, and the region is one op longer for it.
///
/// The neighbour of `an_if_with_an_else_compiles_to_a_clause_region_and_two_
/// jumps`, and the pair is the whole of D23's emission decision: one body, two
/// settings, two streams. Everything but the `TraceClause` and the indices it
/// shifts is identical, which is what says the setting decides *what is
/// emitted* rather than what any op does.
///
/// **The echo is the region's first op, not its last.** The tree-walker echoes
/// a clause before it computes anything, so the `>>>` line an `IF`'s condition
/// produces follows the `*-*` line; an echo emitted after the `EvalExpr` would
/// reverse them and no register or jump would move.
#[test]
fn a_traced_if_carries_its_clause_echo_as_an_op_of_the_region() {
    let chunk = compile_for_test_under(b"if 1 = 1 then say 'a'\nelse say 'b'\nsay 'c'\n", traced())
        .expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=4\n\
         1: TraceClause index=0\n\
         2: EvalExpr index=0 slot=0 dst=0\n\
         3: JumpUnless reg=0 target=8\n\
         4: Generic index=1\n\
         5: Generic index=2\n\
         6: EndBranch\n\
         7: Jump target=10\n\
         8: Generic index=3\n\
         9: Generic index=4\n\
         10: Generic index=5\n"
    );
    // The echo op addresses no register, so the extra op changes nothing the
    // driver has to reserve.
    assert_eq!(chunk.registers, 1);
    assert_eq!(chunk.op_of, vec![0, 4, 5, 6, 9, 10, 11]);
}

/// A traced `SELECT CASE`: **one echo per promoted clause**, the header's and
/// each listed `WHEN`'s, and none anywhere else.
///
/// Three things this pins that the `IF` pair does not:
///
/// * the header's echo sits **before** its `EvalExpr`, so the `>K>  "CASE"`
///   line the expression produces follows the `*-*` line rather than preceding
///   it;
/// * `SelectCaseText` stays **outside** the region, one op further along than
///   it was untraced -- it is not part of the clause and the echo must not
///   have pulled it in;
/// * a `THEN` marker, a branch body and the `END` get no echo op at all. They
///   are `Generic`, so their echo comes from the tree-walker's own clause unit
///   and a second one here would print every such clause twice.
#[test]
fn a_traced_select_echoes_its_header_and_each_listed_when() {
    let chunk = compile_for_test_under(
        b"select case 1 + 1\n  when 1 then say 'a'\n  when 2 then say 'b'\nend\nsay 'after'\n",
        traced(),
    )
    .expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=3\n\
         1: TraceClause index=0\n\
         2: EvalExpr index=0 slot=0 dst=0\n\
         3: SelectCaseText index=0 case=0\n\
         4: Clause index=1 end=8\n\
         5: TraceClause index=1\n\
         6: WhenTest index=1 case=0 dst=1\n\
         7: JumpUnless reg=1 target=11\n\
         8: EnterWhen select=0 when=1\n\
         9: Generic index=2\n\
         10: Generic index=3\n\
         11: Clause index=4 end=15\n\
         12: TraceClause index=4\n\
         13: WhenTest index=4 case=0 dst=1\n\
         14: JumpUnless reg=1 target=18\n\
         15: EnterWhen select=0 when=4\n\
         16: Generic index=5\n\
         17: Generic index=6\n\
         18: Generic index=7\n\
         19: Generic index=8\n"
    );
    assert_eq!(
        chunk.registers, 2,
        "the CASE value and one WHEN answer are live at once, and never more"
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
    let chunk =
        super::compile(&program.main, &plan, ChunkTrace::of(TraceMode::NORMAL)).expect("compiles");
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

    let untraced = ChunkTrace::of(TraceMode::NORMAL);
    let first = interp
        .chunk_for(key, untraced, &program.main, &plan)
        .expect("compiles");
    let second = interp
        .chunk_for(key, untraced, &program.main, &plan)
        .expect("cached");
    let third = interp
        .chunk_for(key, untraced, &program.main, &plan)
        .expect("cached");

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

/// **One body, two settings, two chunks.** The trace setting is an input to
/// compilation (D23), so a cache keyed on `BodyKey` alone hands the body
/// entered under the second setting a stream compiled for the first one.
///
/// The three assertions answer three different degenerate caches, and the
/// third is the one the key change is for:
///
/// * `compile` ran twice, so the second setting was compiled for rather than
///   answered from the first setting's entry -- this is what a key that
///   ignores the setting fails;
/// * the two chunks are distinct `Rc`s, so it is not one chunk handed back
///   under two names;
/// * and asking again under the *first* setting gives the *first* chunk back,
///   which is what says the second lookup added an entry rather than replacing
///   one. A cache that evicted on a setting change passes the first two and
///   fails this, and a program that toggles `TRACE` in a loop is what that
///   costs.
#[test]
fn one_body_under_two_trace_settings_is_two_cached_chunks() {
    let program = parse_program(b"if 1 = 1 then say 1\n".to_vec()).expect("test program parses");
    let key = BodyKey {
        program: ProgramId(0),
        directive: None,
    };
    let plan = Plan::build(&program.main, &program.symbols);
    let untraced = ChunkTrace::of(TraceMode::NORMAL);

    let mut interp = Interp::new();
    let before = super::compile::compile_calls();

    let silent = interp
        .chunk_for(key, untraced, &program.main, &plan)
        .expect("compiles");
    let echoing = interp
        .chunk_for(key, traced(), &program.main, &plan)
        .expect("compiles");

    assert_eq!(
        super::compile::compile_calls() - before,
        2,
        "the second setting was answered from the first setting's cached chunk"
    );
    assert!(
        !Rc::ptr_eq(&silent, &echoing),
        "both settings got the same chunk, so one of them is running the other's stream"
    );
    assert!(
        render(&silent) != render(&echoing),
        "the two settings compiled to the same stream, so the setting decided nothing"
    );
    assert!(
        Rc::ptr_eq(
            &silent,
            &interp
                .chunk_for(key, untraced, &program.main, &plan)
                .expect("cached")
        ),
        "the first setting's chunk was evicted rather than kept beside the second's"
    );
}
