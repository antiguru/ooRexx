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

//! What the driver did: which bodies it drove and which clauses it stepped.

use super::super::{CALL_SITE_CACHE, QUICKENING};
use super::{
    arith_hint_skips, call_site_hits, clause_op_entries, const_builds, frame_floor_high_water,
    load_constant_builds, run_chunk_entries, trace_op_echoes,
};
use crate::{Invocation, Outcome, execute, run_program};

/// The path these programs are reported under. Nothing reads it back: no
/// program below raises, so it never reaches a report.
const TEST_PATH: &str = "/nonexistent/ir-drive-test.rex";

/// A program that enters three bodies: its own main body, the internal label
/// `sub` a `CALL` transfers to, and the `::ROUTINE` body a second `CALL`
/// resolves to.
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
fn drive() -> (Outcome, usize) {
    let before = run_chunk_entries();
    let outcome = execute(TEST_PATH, THREE_BODIES.to_vec(), false, Invocation::none());
    (outcome, run_chunk_entries() - before)
}

#[test]
fn the_ir_engine_drives_every_body_the_program_enters() {
    let (outcome, driven) = drive();
    assert_eq!(outcome.exit_code, 0, "stderr: {:?}", outcome.stderr);
    assert_eq!(
        driven, 3,
        "the IR engine drove {driven} chunks where the program has three \
         bodies: its main body, the CALLed label, and the ::ROUTINE. Fewer \
         means a way of entering an activation did not reach the compiled \
         stream, which every later promotion would then silently skip"
    );
}

/// A three-pass loop over a one-clause body: the loop's `DO` clause and each
/// of the three body clauses are stepped from the chunk.
#[test]
fn the_ir_engine_steps_a_loop_body_from_the_chunk() {
    const COUNTED_LOOP: &[u8] = b"do i = 1 to 3\n  nop\nend\n";

    let before = clause_op_entries();
    let outcome = execute(TEST_PATH, COUNTED_LOOP.to_vec(), false, Invocation::none());
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
#[test]
fn the_ir_engine_steps_a_simple_blocks_body_from_the_chunk() {
    let before = clause_op_entries();
    let outcome = execute(
        TEST_PATH,
        b"do\n  nop\n  nop\nend\n".to_vec(),
        false,
        Invocation::none(),
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
#[test]
fn the_ir_engine_steps_an_ifs_chosen_branch_from_the_chunk() {
    // `IF`, `THEN`, `say 'a'`, `ELSE`, `say 'b'`, `say 'c'` -- four clauses
    // run on either path, and which four is what differs.
    for (condition, expected, path) in [("1 = 1", 4, "then"), ("1 = 0", 4, "else")] {
        let program = format!("if {condition} then say 'a'\nelse say 'b'\nsay 'c'\n");
        let before = clause_op_entries();
        let outcome = execute(TEST_PATH, program.into_bytes(), false, Invocation::none());
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
#[test]
fn the_ir_engine_steps_a_selects_chosen_branch_from_the_chunk() {
    // The matched path: the `SELECT` clause, both `WHEN` clauses, the winning
    // `THEN` marker and its body, and the clause after the whole construct.
    // The `END` is not among them and is not missing: one true `WHEN` resumes
    // past it.
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
            Invocation::none(),
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

/// A body entered with `TRACE R` already in force echoes its promoted clauses
/// from the chunk's own [`crate::ir::Op::TraceClause`], rather than from the
/// clause unit's run-time gate.
#[test]
fn a_body_entered_under_trace_r_echoes_its_promoted_clause_from_the_chunk() {
    const ENTERED_TRACED: &[u8] = b"\
trace r
call sub
exit
sub:
  if 1 = 1 then nop
  return
";
    let before = trace_op_echoes();
    let outcome = execute(
        TEST_PATH,
        ENTERED_TRACED.to_vec(),
        false,
        Invocation::none(),
    );
    let echoed = trace_op_echoes() - before;
    assert_eq!(outcome.exit_code, 0, "stderr: {:?}", outcome.stderr);
    assert_eq!(
        echoed, 5,
        "the compiled stream emitted {echoed} clause echoes from a trace op where the callee's \
         promoted clauses -- its label, its IF, its THEN, its NOP and its RETURN -- owe one each, \
         so either its chunk was not compiled for the setting it was entered under or the ops it \
         carries did not run"
    );
}

/// The negative control for the count above: the identical construct with
/// everything but `TRACE` unchanged emits no clause echo from a trace op.
#[test]
fn no_trace_op_echoes_without_the_setting() {
    const ENTERED_UNTRACED: &[u8] = b"call sub\nexit\nsub:\n  if 1 = 1 then nop\n  return\n";

    let before = trace_op_echoes();
    let outcome = execute(
        TEST_PATH,
        ENTERED_UNTRACED.to_vec(),
        false,
        Invocation::none(),
    );
    let echoed = trace_op_echoes() - before;
    assert_eq!(outcome.exit_code, 0, "stderr: {:?}", outcome.stderr);
    assert_eq!(
        echoed, 0,
        "{echoed} clause echoes came from a trace op where no TRACE is in force, so the count \
         no longer tracks the compiled emission decision"
    );
}

/// A quickened arithmetic site handed operands the small-integer path cannot
/// take answers what the general path answers.
#[test]
fn a_quickened_site_falls_through_to_the_general_path() {
    // A `SELECT` rather than a table of programs, because a fresh program is a
    // fresh chunk with a fresh patch table: what this test needs is one site
    // reached repeatedly, having already been quickened by an earlier pass.
    const QUICKENED_THEN_WIDENED: &[u8] = b"\
numeric digits 3
zn = 1
do zi = 1 to 4
  say zn - 25
  say zn * 500
  select
    when zi = 1 then zn = 1000
    when zi = 2 then zn = 500
    when zi = 3 then zn = 0.5
    otherwise nop
  end
end
";

    let before = arith_hint_skips();
    let outcome = execute(
        TEST_PATH,
        QUICKENED_THEN_WIDENED.to_vec(),
        false,
        Invocation::none(),
    );
    let skipped = arith_hint_skips() - before;
    assert_eq!(outcome.exit_code, 0, "stderr: {:?}", outcome.stderr);
    assert_eq!(
        String::from_utf8_lossy(&outcome.stdout),
        "-24\n500\n980\n5.00E+5\n475\n2.50E+5\n-24.5\n250\n",
        "a quickened arithmetic site answered something other than the general path's answer \
         for operands the small-integer path cannot take"
    );
    // Two sites, each falling through on the second pass and skipping the
    // attempt on the two after it.
    let expected = if QUICKENING { 4 } else { 0 };
    assert_eq!(
        skipped, expected,
        "the two sites skipped the small-integer attempt {skipped} times where {expected} was \
         due, so the hint is not being read back"
    );
}

/// Nothing in an ordinary program overflows the compiled stream's index
/// widths, so the counter `Interp::chunk_for` bumps on a refusal stays at zero.
#[test]
fn no_body_is_refused() {
    let outcome = run_program(TEST_PATH, THREE_BODIES.to_vec(), Invocation::none());
    assert_eq!(
        outcome.chunks_refused, 0,
        "the compiler refused a body of a three-body program {} times",
        outcome.chunks_refused
    );
}

/// A call site reached on every pass of a loop resolves **once** and answers
/// from what it kept on every pass after that.
#[test]
fn a_call_site_resolves_once_and_answers_from_what_it_kept() {
    const TWO_SITES_IN_A_LOOP: &[u8] = b"\
do zi = 1 to 4
  call zsub zi
  say result
  call length 'abcde'
  say result
end
exit
zsub:
  use arg zn
  return zn * 10
";

    let before = call_site_hits();
    let outcome = execute(
        TEST_PATH,
        TWO_SITES_IN_A_LOOP.to_vec(),
        false,
        Invocation::none(),
    );
    let hits = call_site_hits() - before;
    assert_eq!(outcome.exit_code, 0, "stderr: {:?}", outcome.stderr);
    assert_eq!(
        String::from_utf8_lossy(&outcome.stdout),
        "10\n5\n20\n5\n30\n5\n40\n5\n",
        "a call site answered from a resolution that was not its own"
    );
    // Two sites, each resolving on the first pass and answering from the site
    // on the three after it.
    let expected = if CALL_SITE_CACHE { 6 } else { 0 };
    assert_eq!(
        hits, expected,
        "the two sites answered from a kept resolution {hits} times where {expected} was due, so \
         the table is not being read back"
    );
}
/// **A slot resolved before the loop still names its own variable after the
/// frame has grown under it**, which is the wrong-answer risk a compiled write
/// and a kept control slot both carry.
#[test]
fn a_resolved_slot_still_names_its_variable_after_the_frame_grows() {
    const GROWS_UNDER_A_CACHED_SLOT: &[u8] = b"\
zc = 0
do zi = 1 to 4
  interpret 'zn' || zi || ' = ' || (zi * 10)
  zc = zc + zi
end
say zc zi
say zn1 zn2 zn3 zn4
call sub
say 'caller' zg
exit
sub: procedure expose zg
  zg = 2
  do zj = 1 to 3
    zg = zg + zj
  end
  interpret 'zfresh = 9'
  zg = zg + zfresh
  say 'callee' zg zj
  return
";

    let outcome = execute(
        TEST_PATH,
        GROWS_UNDER_A_CACHED_SLOT.to_vec(),
        false,
        Invocation::none(),
    );
    assert_eq!(outcome.exit_code, 0, "stderr: {:?}", outcome.stderr);
    assert_eq!(
        String::from_utf8_lossy(&outcome.stdout),
        "10 5\n10 20 30 40\ncallee 17 4\ncaller 17\n",
        "the compiled stream did not answer the oracle's own bytes for this program"
    );
}

/// The adjacent case the test above needs to be pinned rather than
/// coincidental: **a control variable whose slot must not be resolved early at
/// all.**
#[test]
fn a_compound_control_resolves_its_tail_on_every_pass() {
    const A_MOVING_TAIL: &[u8] = b"\
za. = 0
zi = 1
do za.zi = 1 to 3
  if zi > 7 then leave
  zi = zi + 1
end
say zi za.1 za.2 za.3 za.8
";

    let outcome = execute(TEST_PATH, A_MOVING_TAIL.to_vec(), false, Invocation::none());
    assert_eq!(outcome.exit_code, 0, "stderr: {:?}", outcome.stderr);
    assert_eq!(
        String::from_utf8_lossy(&outcome.stdout),
        "8 1 1 1 1\n",
        "a compound control's tail was resolved once instead of on every pass"
    );
}

/// A raise that escapes two open loops leaves the frame stack where it found
/// it, once per trap, however many times it happens.
#[test]
fn a_trapped_raise_gives_back_every_frame_the_loops_it_escaped_opened() {
    const TRAPS_OUT_OF_LOOPS: &[u8] = b"\
signal on syntax
zc = 0
retry:
do i = 1 to 2
  do k = 1 to 2
    zq = 1 / 0
  end
end
exit 1

syntax:
signal on syntax
zc = zc + 1
if zc > 4 then do
  say 'traps' zc
  exit 0
  end
signal retry
";

    let before = frame_floor_high_water();
    let outcome = execute(
        TEST_PATH,
        TRAPS_OUT_OF_LOOPS.to_vec(),
        false,
        Invocation::none(),
    );
    // The adjacent success: five traps fired and the handler ran to its own
    // `EXIT`, so the frames really were opened and really were escaped. Without
    // this the assertion below is satisfied by a driver that never pushes one.
    assert_eq!(outcome.exit_code, 0, "stderr: {:?}", outcome.stderr);
    assert_eq!(
        String::from_utf8_lossy(&outcome.stdout),
        "traps 5\n",
        "the program did not trap five times out of its loops, so what the \
         floor below says about frames is about some other program"
    );
    assert_eq!(
        frame_floor_high_water(),
        before,
        "an entry found frames another level had left open. Every `run_ops` \
         call in this program is the activation's own, so each starts at a \
         floor of zero unless a raise walked out over open frames without \
         giving them back"
    );
}

/// A constant is built once however many times its op runs.
#[test]
fn a_long_constant_is_built_once_however_many_passes_read_it() {
    const ONE_CONSTANT: &[u8] = b"\
do i = 1 to 3
  zs = 'a literal too long to live in the handle'
end
say zs
";
    const TWO_CONSTANTS: &[u8] = b"\
do i = 1 to 3
  zs = 'a literal too long to live in the handle'
  zt = 'a second literal, also too long for the handle'
end
say zs zt
";

    let before = const_builds();
    let outcome = execute(TEST_PATH, ONE_CONSTANT.to_vec(), false, Invocation::none());
    let one = const_builds() - before;
    assert_eq!(outcome.exit_code, 0, "stderr: {:?}", outcome.stderr);
    assert_eq!(
        String::from_utf8_lossy(&outcome.stdout),
        "a literal too long to live in the handle\n",
        "the program did not run its loop, so the build count is about some other program"
    );
    assert_eq!(
        one, 1,
        "one constant read on three passes was built {one} times, so `Op::Const` is \
         rebuilding its value instead of reading the one it interned"
    );

    let before = const_builds();
    let outcome = execute(TEST_PATH, TWO_CONSTANTS.to_vec(), false, Invocation::none());
    let two = const_builds() - before;
    assert_eq!(outcome.exit_code, 0, "stderr: {:?}", outcome.stderr);
    assert_eq!(
        String::from_utf8_lossy(&outcome.stdout),
        "a literal too long to live in the handle a second literal, also too long for the \
         handle\n",
        "the program did not run its loop, so the build count is about some other program"
    );
    assert_eq!(
        two, 2,
        "two distinct constants over three passes were built {two} times, where one build \
         each is the whole of what the cache promises"
    );
}

/// A constant symbol is built once however many times its op runs, and each
/// distinct symbol gets its own entry.
#[test]
fn a_constant_symbol_is_built_once_and_each_one_gets_its_own_entry() {
    const THREE_PASSES: &[u8] = b"\
do i = 1 to 3
  zs = 1.50000000000
end
say zs
";
    const THIRTY_PASSES: &[u8] = b"\
do i = 1 to 30
  zs = 1.50000000000
end
say zs
";
    const TWO_SYMBOLS: &[u8] = b"\
do i = 1 to 3
  zs = 1.50000000000
  zt = 2.50000000000
end
say zs zt
";

    fn builds(program: &[u8], expected: &str) -> usize {
        let before = load_constant_builds();
        let outcome = execute(TEST_PATH, program.to_vec(), false, Invocation::none());
        assert_eq!(outcome.exit_code, 0, "stderr: {:?}", outcome.stderr);
        assert_eq!(
            String::from_utf8_lossy(&outcome.stdout),
            expected,
            "the program did not produce its own values, so the build count below is \
             about some other program"
        );
        load_constant_builds() - before
    }

    let three = builds(THREE_PASSES, "1.50000000000\n");
    let thirty = builds(THIRTY_PASSES, "1.50000000000\n");
    let two = builds(TWO_SYMBOLS, "1.50000000000 2.50000000000\n");

    assert_eq!(
        three, thirty,
        "three passes built {three} constants and thirty built {thirty}, so \
         `Op::LoadConstant` is rebuilding its value per execution rather than reading \
         the one it interned"
    );
    assert_eq!(
        two,
        three + 1,
        "adding a second distinct constant symbol took the count from {three} to {two}, \
         where exactly one more build is what a table keyed by the symbol's own id \
         promises"
    );
}
