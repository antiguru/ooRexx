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
//!
//! **Output cannot answer this and these tests do not ask it to.** A construct
//! compiles to one region shape or another without changing a byte of what a
//! program prints, so a test reading output stays green across a change of
//! shape. What separates the shapes is the count: which bodies the driver
//! drove ([`super::run_chunk_entries`]) and which clauses it stepped
//! ([`super::clause_op_entries`]).

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
/// `Invocation` happens here too, so the invocation still travels the whole
/// production route. The stack `run_program` sizes is not needed for a
/// program three shallow bodies deep.
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
///
/// **This is the only observable that separates a promoted `DO`/`LOOP` from
/// an unpromoted one**, and that is why it is a count rather than a
/// comparison of output. The construct is resolved by the same
/// `run_loop_with_header` either way, so both engines print the same bytes on
/// every program; what changes is whether the body's clauses reach the
/// compiled stream at all.
/// With the body left on the tree-walker the count is 1 -- the `DO` clause
/// alone -- and every promotion after this one would then silently skip
/// anything written inside a loop.
///
/// The `END` clause is not in the count and is not missing from it:
/// `run_bounded`'s range stops before `END`, and the loop's own `Flow::Goto`
/// resumes one past it, so no engine ever steps it.
///
/// **The count did not move when the header was flattened, and it is derivable
/// that it could not.** A loop's `DO` clause is opened once either way: it used
/// to be an op that handed the whole instruction to the clause unit, and it is
/// now a `Clause` region whose ops evaluate the header and then run the
/// construct. Both open exactly one clause for the `DO`, and the header's own
/// re-evaluation on each later pass is not a clause of this body -- it is
/// `run_repeating`'s own `in_clause`, which opens no clause of an instruction
/// and steps none. What *would* move this count is the loop's per-pass header
/// becoming a region of its own.
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
///
/// **A separate test from the counted loop's, because `LoopKind::Simple` is a
/// separate arm of `run_loop_with_header` with a `run_bounded` call of its
/// own**, and the
/// counted loop's count cannot see it: leaving that one arm on the tree-walker
/// leaves every other test in the workspace green, including the dual-engine
/// sweep, which cannot tell the two drivers apart because both produce
/// identical bytes.
///
/// **At the top of the body, not inside an `IF`.** An enclosing `IF` puts its
/// own clause into the same count -- the next test is the one that measures
/// that -- so a block written as `if 1 = 1 then do ... end` would give a
/// number that is not the block's alone.
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
///
/// **The only observable that separates a promoted `IF` from an unpromoted
/// one**, and that is why it is a count. An `IF`'s own condition reaches the
/// same `Interp::condition_value` on either engine -- whether it gets there
/// through `eval_if_condition` or through the ops `crate::ir::Op::Condition`
/// ends -- so both engines print the same bytes for every program, and what
/// changes is whether the branch's clauses reach the compiled stream at all.
/// With the branch left on the tree-walker the count is 2 on each path: the
/// `IF` clause and the one clause after the whole construct, with everything
/// inside the branch reached through `run_bounded`'s tree-walker arm and
/// counted nowhere.
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
///
/// **The only observable that separates a promoted `SELECT` from an unpromoted
/// one**, for the reason the `IF` count above is a count: the construct's
/// targets and escapes are resolved through the same `when_targets`,
/// `select_escape` and `leave_select` either way, and a `WHEN`'s own condition
/// reaches the same `Interp::condition_value` on either engine -- whether it
/// gets there through `scan_when` or through the ops
/// `crate::ir::Op::Condition` ends. So both engines print the same bytes for
/// every program and nothing in the output says which drove it. With the whole
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
///
/// **This is the only observable that says the compiled emission decision is
/// reached in production**, and it is a count for a sharper reason than the
/// counts above are. There the two engines merely agree; here the driver
/// deliberately *falls back* to the same gate the tree-walker uses whenever
/// the setting in force is not the one the chunk was compiled under, and that
/// fallback prints the identical bytes. So an implementation that never
/// compiled a trace op, or compiled one and never ran it, produces byte-exact
/// correct output on every program in the tree. Two mutations do exactly that
/// -- asking `chunk_for` for a chunk under a constant setting rather than the
/// one in force, and answering "stale" for every clause -- and both leave the
/// whole workspace green without this.
///
/// **Entered with the setting in force, not setting it in the body**, and that
/// is what the `CALL` is for: a body compiles when it is first entered, so a
/// `TRACE` on the body's own first line runs *after* its chunk exists and the
/// chunk is the untraced one. The callee inherits `trace r` at the call, so
/// its chunk is compiled to echo.
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
///
/// **It answers the degenerate counter the pair above cannot.** A counter that
/// moved on every promoted clause regardless of setting would satisfy the
/// traced test on its own; this is what says the *setting* decides.
///
/// **There used to be a second control here, and it went with the second
/// engine.** It ran the traced program on the tree-walker, whose clauses never
/// came from a chunk and so could not echo from a trace op at all -- it
/// answered "a counter that never moved", which no engine choice can produce
/// now. What that leaves is the control that was always about the compiled
/// emission decision rather than about which engine ran.
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
///
/// **The precondition check the patch table is not allowed to remove.** A hint
/// says a site *has* taken the small-integer path, never that it may skip the
/// test -- so every arm that reads one re-decodes both operands and re-checks
/// them against `DIGITS`, and falls through when either fails.
///
/// The program runs one pair of sites four times, and every pass but the first
/// falls through for a **different** reason, each one preceded by a pass that
/// took the fast path at the identical site:
///
/// * `zn = 1`: `1 - 25` and `1 * 500` are both exact at `DIGITS 3`, so both
///   sites take the small-integer path and both are quickened;
/// * `zn = 1000`: an **operand** wider than `DIGITS`. This is the case four
///   hand-written probes missed, because Rexx rounds the operands before it
///   operates and not only the answer: measured on the oracle, `1000 - 25` at
///   `DIGITS 3` is `980` and not the exact `975`, so a guard that checked only
///   the result would answer `975` here and be wrong by a rounding nobody
///   would look for;
/// * `zn = 500`: operands both inside `DIGITS`, **result** outside it --
///   `500 * 500` is `2.50E+5`, so an `i64` that fits the tag is still not the
///   value the general path produces;
/// * `zn = 0.5`: an operand that is **not an integer at all**, so the decode
///   itself fails rather than the range check.
///
/// Every expected byte was measured against the C++ oracle.
///
/// **The skip count is the half that says the table was read**, and without it
/// this test is satisfied by a patch table nothing ever consults: both paths
/// answer identically for every operand, so the bytes alone cannot tell which
/// one ran. [`super::arith_hint_skips`]'s own comment has why that is the
/// observable and not the load.
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
    //
    // **Zero with the table switched off, and asserted rather than skipped**,
    // so that the exit criterion's own measuring build has a green suite and
    // this test still says which configuration it is in. The bytes above are
    // required either way, which is the half that is about the precondition
    // rather than about the table.
    let expected = if QUICKENING { 4 } else { 0 };
    assert_eq!(
        skipped, expected,
        "the two sites skipped the small-integer attempt {skipped} times where {expected} was \
         due, so the hint is not being read back"
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
    let outcome = run_program(TEST_PATH, THREE_BODIES.to_vec(), Invocation::none());
    assert_eq!(
        outcome.chunks_refused, 0,
        "the compiler refused a body of a three-body program {} times",
        outcome.chunks_refused
    );
}

/// A call site reached on every pass of a loop resolves **once** and answers
/// from what it kept on every pass after that.
///
/// **The hit count is the half that says the table was read**, and without it
/// this test is satisfied by a table nothing ever consults: a kept resolution
/// and a fresh one are the same answer -- which is what makes keeping one safe
/// -- so the bytes alone cannot tell which ran. [`super::call_site_hits`]'s own
/// comment has why that is the observable.
///
/// **Two sites, resolving to two different things**, so the count is not one
/// site's alone and a table that handed every site the first answer it stored
/// would produce the wrong bytes rather than a smaller count: `zsub` is an
/// internal label and `length` is a builtin, and a shared entry would call one
/// of them four times.
///
/// The expected bytes were measured against the C++ oracle.
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
    //
    // **Zero with the table switched off, and asserted rather than skipped**,
    // so that a measuring build has a green suite and this test still says
    // which configuration it is in -- `a_quickened_site_falls_through_to_the_
    // general_path`'s own shape, one table over.
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
///
/// Three growth events in one program, because a cached index survives or fails
/// for one reason and each of these reaches it differently: an `INTERPRET`
/// binding a name the plan never saw grows the frame **inside** the loop whose
/// control slot was taken before it; `PROCEDURE EXPOSE` turns a slot of the
/// callee's own frame into an alias of the caller's, so a write through the
/// cached index has to land in the caller's storage; and the callee's own
/// `INTERPRET` grows a frame that already holds an alias.
///
/// **The expected bytes are the oracle's**, measured on this program directly:
/// `10 5` / `10 20 30 40` / `callee 17 4` / `caller 17`. `zc` and `zg` are
/// compiled writes, `zi` and `zj` are control variables, and every one of them
/// is read back after the growth that could have displaced it.
///
/// Run on **both engines**, which is what makes it the witness for the control
/// slot as well as for the write. `Interp::run_loop_with_header` is shared, so
/// a control slot resolved wrongly is wrong on both arms identically and the
/// dual-engine sweep structurally cannot see it; only bytes pinned to the
/// oracle can.
///
/// **It can fail, and it is not the only thing that would notice a wrong slot**
/// -- measured, by making `control_slot` answer one past the plan's: this test
/// reddens, and so do dozens of existing loop tests that never grow a frame at
/// all. What is unique to it is the growth, not the slot.
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
///
/// A compound control names a different tail on every pass -- the body's own
/// `zi = zi + 1` moves which tail `za.zi` is -- so the loop's bound keeps
/// comparing against a fresh, still-default `0`, and it is the `LEAVE` that
/// ends it at `8`. An implementation that resolved `za.zi` once and reused it
/// prints `4`. The oracle's own answer, measured on this program: `8 1 1 1 1`.
///
/// **Neither of the two "answer for a compound as well" widenings reddens this
/// test or anything else in the suite**, both measured by running them. They
/// are unobservable for different reasons, and only one of them is
/// unobservable in the code rather than in this suite's contents.
///
/// `control_slot` answering for a compound control is unobservable in the
/// code: `bind_control` and the loop's own re-test each select their arms by
/// the same `shape_of`, and neither compound arm reads `at`, so the slot would
/// be resolved and discarded whatever program ran.
///
/// `write_slot` answering for a compound assignment target is **not**. A
/// compound-shaped name does reach `Plan::by_symbol`: `Plan::bind` puts every
/// name it binds whole there, and `note_loop` and `note_parse` both call it
/// with spellings that may be compound-shaped. So the widened arm has a number
/// to answer with whenever such a spelling is also an assignment target.
/// Measured, in a debug build: `zb = 1; do za.zb = 1 to 2; end; za.zb = 5`
/// panics under `REXX_ENGINE=ir` at rc 101, and the `DO ... OVER` and `PARSE
/// VAR` spellings of the same shape do too, through the tripwire `Op::Store`'s
/// own arm carries. The
/// tree-walker does not, because that tripwire guards an op it never reads.
/// `parse value 'seven' with za.zb` does not either, because a `PARSE` target
/// reaches the plan through `note_compound_name` and never gets a `by_symbol`
/// entry. **No program in the suite has that shape**, which is why the
/// mutation comes back green; it is a fact about the suite, not about the
/// code.
///
/// `Interp::assign_expr_target`'s compound arm asserts that no caller handed
/// it a slot, which is the check that does not depend on which engine ran or
/// on which programs the suite holds.
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
///
/// **The property a shared frame stack needs and a `Vec` local to each level
/// got for free.** There, a `Failure` unwound past the open frames and the
/// `Vec` went with it. Here the stack outlives the level, so the entry
/// boundary has to put it back, and nothing about the program's output says
/// whether it did: leftovers sit below the next level's floor, are never read
/// again, and the interpreter goes on printing the right bytes while the stack
/// grows for as long as the program keeps trapping. `FRAME_FLOOR_HIGH_WATER`
/// exists for exactly this, and carries the memory measurement.
///
/// The loops are what make it a test of the unwind rather than of the trap:
/// two are open at the raise, both flattened, so each trap has two frames and
/// two `flat_loops` entries to discard, and a leak shows as a floor climbing
/// by two per trap rather than staying at zero.
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
///
/// **The only observable the interning has.** A value rebuilt per execution
/// and a value built once produce identical bytes, identical trace and the
/// same exit status -- that is what makes sharing one object safe, and it is
/// also why no output comparison in this workspace can tell whether the cache
/// is doing anything. `CONST_BUILDS` counts where the value is built, so a
/// cache written and never read shows one build per pass here.
///
/// The literal is longer than a handle carries inline, which is the case that
/// used to take an arena slot every time; a short one would be answered from
/// the handle by `Interp::interned_literal` before the cache was consulted at
/// all, and would say nothing about interning.
///
/// **The pair is what makes the count mean "per distinct constant".** One
/// literal over three passes is one build, and a second, different literal in
/// the same body makes it two -- so a cache that ignored its index and
/// answered every constant with the first one would fail the second assertion
/// while passing the first.
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
///
/// The sibling of `a_long_constant_is_built_once_however_many_passes_read_it`,
/// and separate from it because the two ops key their caches differently: a
/// literal's bytes get a slot `compile` assigns, and a constant symbol is keyed
/// by its own `SymbolId`, on the strength of those being dense and zero-based.
///
/// **A constant symbol is a numeric one**, which the shape of these programs
/// depends on: an unquoted word like `abc` is a *variable* whose value defaults
/// to its own upcased spelling, and compiles to `Op::Load`. The values here are
/// written with a decimal point so they are not canonical small integers, and
/// long enough not to fit the handle inline -- either would be answered before
/// the cache was consulted and would say nothing about it.
///
/// **Neither count is asserted against a fixed number**, because a loop header
/// contributes constant symbols of its own -- `1` and `3` in `do i = 1 to 3`
/// are two more `Op::LoadConstant`s, and pinning a total would be pinning that.
/// The two properties are stated directly instead: the count does not move with
/// the pass count, and one more distinct symbol is one more build.
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
