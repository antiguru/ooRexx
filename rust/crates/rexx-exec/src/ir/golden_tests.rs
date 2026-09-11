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

use rexx_parse::{ExprKind, InstructionKind, parse_program};

use super::golden::render;
use super::{Chunk, ChunkTooLarge, NodePath};
use crate::Interp;
use crate::plan::{BodyKey, BodyKind, Plan, ProgramId};
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
    let plan = Plan::build(
        &program.main,
        &program.symbols,
        Some(&program.source),
        BodyKind::Plain,
    );
    super::compile(&program.main, &plan, trace)
}

/// The setting `TRACE R` puts in force, as far as compilation can see it.
fn intermediates() -> ChunkTrace {
    ChunkTrace::of(crate::trace::mode_from_setting(b"i").expect("I is a valid TRACE setting"))
}

fn traced() -> ChunkTrace {
    ChunkTrace::of(crate::trace::mode_from_setting(b"r").expect("R is a valid TRACE setting"))
}

/// A message send as a whole clause compiles to a `Clause` region ending in
/// an `Op::Message`, whichever form it was written in.
#[test]
fn a_message_send_clause_compiles_to_a_region_ending_in_one_message_op() {
    for source in [
        &b"'abc'~length"[..],
        &b"'abc'~~length"[..],
        &b"zz = 'abc'; zz[1] = 2"[..],
    ] {
        let chunk = compile_for_test(source).expect("the chunk fits");
        let rendered = render(&chunk);
        let last = rendered
            .lines()
            .next_back()
            .expect("the stream is not empty");
        let at = rendered.lines().count() - 1;
        assert_eq!(
            last,
            format!(
                "{at}: Message index={}",
                if source.starts_with(b"zz") { 1 } else { 0 }
            ),
            "the stream for {:?} is\n{rendered}",
            String::from_utf8_lossy(source)
        );
    }

    // The whole stream for the plain form, so that the region's own bounds and
    // the absence of any expression op are stated rather than implied.
    assert_eq!(
        render(&compile_for_test(b"'abc'~length").expect("the chunk fits")),
        "0: Clause index=0 end=2\n\
         1: Message index=0\n"
    );
}

/// A call compiles to [`super::Op::CallExpr`] wherever an address reaches it
/// inside a slot that compiles natively, and the address the op carries is the
/// route down to it: the same call at the root, as a left operand and as a
/// right operand gets the same op under a different address each time.
#[test]
fn a_call_promotes_at_the_root_and_below_it() {
    let root = compile_for_test(b"zz = length('abc')").expect("the chunk fits");
    assert_eq!(
        render(&root),
        "0: Clause index=0 end=6\n\
         1: Const dst=1 konst=0\n\
         2: PushArg src=1\n\
         3: CallArgs slot=0 path=root site=0 argc=1 dst=0\n\
         4: TraceFunction index=0 slot=0 path=root src=0\n\
         5: Store index=0 at=0 src=0\n"
    );

    let left = compile_for_test(b"zz = length('abc') + 1").expect("the chunk fits");
    assert_eq!(
        render(&left),
        "0: Clause index=0 end=8\n\
         1: Const dst=1 konst=0\n\
         2: PushArg src=1\n\
         3: CallArgs slot=0 path=root.L site=0 argc=1 dst=0\n\
         4: TraceFunction index=0 slot=0 path=root.L src=0\n\
         5: LoadConstant dst=1\n\
         6: Arith op=+ hint=0 lhs=0 rhs=1 dst=0\n\
         7: Store index=0 at=0 src=0\n"
    );

    let right = compile_for_test(b"zz = 1 + length('abc')").expect("the chunk fits");
    assert_eq!(
        render(&right),
        "0: Clause index=0 end=8\n\
         1: LoadConstant dst=0\n\
         2: Const dst=2 konst=0\n\
         3: PushArg src=2\n\
         4: CallArgs slot=0 path=root.R site=0 argc=1 dst=1\n\
         5: TraceFunction index=0 slot=0 path=root.R src=1\n\
         6: Arith op=+ hint=0 lhs=0 rhs=1 dst=0\n\
         7: Store index=0 at=0 src=0\n"
    );

    // Two calls over one operator: a register each for the two results, and
    // **one** shared by both arguments, because each call's argument register
    // is released once the call that reads it has been emitted. A compiler
    // that kept them would reserve four.
    let two_calls = compile_for_test(b"zz = length('ab') + length('cd')").expect("the chunk fits");
    assert_eq!(
        two_calls.registers,
        3,
        "two calls over one operator reserved {} registers, where the second call's argument \
         should reuse the first's\n{}",
        two_calls.registers,
        render(&two_calls)
    );
}

/// A call the address cannot reach leaves the whole slot general, which is the
/// answer it had before there was an address at all.
#[test]
fn a_call_nested_past_the_paths_width_leaves_the_slot_general() {
    // `zz = zb + (zb + (... length('abc') ...))`, right-nested so that the
    // call sits `depth` steps down and every one of those steps is the `R`
    // one.
    fn nested(depth: usize) -> Vec<u8> {
        let mut source = b"zz = ".to_vec();
        for _ in 0..depth {
            source.extend_from_slice(b"zb + (");
        }
        source.extend_from_slice(b"length('abc')");
        source.extend(std::iter::repeat_n(b')', depth));
        source
    }

    let mut path = NodePath::ROOT;
    let mut width = 0usize;
    while let Some(deeper) = path.child(true) {
        path = deeper;
        width += 1;
    }

    let at_the_width = compile_for_test(&nested(width)).expect("the chunk fits");
    let rendered = render(&at_the_width);
    let address = format!("root{}", ".R".repeat(width));
    assert!(
        rendered.contains(&format!("CallArgs slot=0 path={address} site=0 argc=1 ")),
        "the call at the deepest address a NodePath carries did not take one\n{rendered}"
    );
    assert!(
        !rendered.contains("EvalExpr"),
        "a slot whose every node has an op still fell back to eval.rs\n{rendered}"
    );

    let past_the_width = compile_for_test(&nested(width + 1)).expect("the chunk fits");
    assert_eq!(
        render(&past_the_width),
        "0: Clause index=0 end=3\n\
         1: EvalExpr index=0 slot=0 dst=0\n\
         2: Store index=0 at=0 src=0\n"
    );
}

/// An instruction whose whole execution is one `Interp::exec_instruction`
/// call gets a clause region of its own holding exactly one
/// [`super::Op::Exec`], carrying its own index.
#[test]
fn every_instruction_of_an_all_delegating_body_compiles_to_one_exec_op() {
    let chunk = compile_for_test(b"drop n1\ndrop n2\nnumeric digits 5\n").expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=2\n\
         1: Exec index=0\n\
         2: Clause index=1 end=4\n\
         3: Exec index=1\n\
         4: Clause index=2 end=6\n\
         5: Exec index=2\n"
    );
    // Nothing here addresses a register, so the chunk reserves none.
    assert_eq!(chunk.registers, 0);
    // And nothing here holds a literal, so the constant table is empty --
    // which is what says an entry comes from an emitted `Const` rather than
    // from every literal the body happens to contain.
    assert!(chunk.consts.is_empty());
}

/// **The phase's first native expression op.** An assignment whose value is a
/// literal compiles to a constant load, the literal's own value line, and the
/// write -- with no `EvalExpr` in it at all, which is what makes it native.
#[test]
fn an_assignment_of_a_literal_compiles_to_a_constant_load_and_a_store() {
    let chunk = compile_for_test(b"n1 = 'abc'\n").expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=3\n\
         1: Const dst=0 konst=0\n\
         2: Store index=0 at=0 src=0\n"
    );
    // One register, released at the clause's own end, so a body of a hundred
    // assignments reserves one.
    assert_eq!(chunk.registers, 1);
    assert_eq!(chunk.consts, vec![Box::from(&b"abc"[..])]);
}

/// **Two assignments in one body reserve one register between them**, which is
/// the release at a promoted clause's own end doing its job.
#[test]
fn two_assignments_and_two_says_in_one_body_reuse_one_register() {
    let chunk = compile_for_test(b"n1 = 'a'\nn2 = 'b'\n").expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=3\n\
         1: Const dst=0 konst=0\n\
         2: Store index=0 at=0 src=0\n\
         3: Clause index=1 end=6\n\
         4: Const dst=0 konst=1\n\
         5: Store index=1 at=1 src=0\n"
    );
    assert_eq!(
        chunk.registers, 1,
        "the second assignment reuses the register the first one released"
    );

    let says = compile_for_test(b"say 'a'\nsay 'b'\n").expect("compiles");
    assert_eq!(
        says.registers, 1,
        "the second SAY reuses the register the first one released"
    );
}

/// **One literal written twice is one entry in the constant table, and both
/// occurrences load it.**
#[test]
fn one_literal_written_twice_is_one_interned_constant() {
    let chunk = compile_for_test(b"say 'dup'\nsay 'dup'\nsay 'x'\n").expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=3\n\
         1: Const dst=0 konst=0\n\
         2: Say index=0 src=0\n\
         3: Clause index=1 end=6\n\
         4: Const dst=0 konst=0\n\
         5: Say index=1 src=0\n\
         6: Clause index=2 end=9\n\
         7: Const dst=0 konst=1\n\
         8: Say index=2 src=0\n"
    );
    assert_eq!(
        chunk.consts,
        vec![Box::from(&b"dup"[..]), Box::from(&b"x"[..])],
        "three literal occurrences of two distinct values are two entries"
    );
}

/// **A whole expression that is a bare symbol compiles to a native read**, in
/// each of the three kinds, with the `>V>` line that reading it owes behind
/// the load exactly as a literal's `>L>` line sits behind its `Const`.
#[test]
fn a_bare_symbol_compiles_to_a_native_read_in_each_of_its_three_kinds() {
    let simple = compile_for_test(b"zw = zv\n").expect("compiles");
    assert_eq!(
        render(&simple),
        "0: Clause index=0 end=3\n\
         1: Load read=Simple at=1 dst=0\n\
         2: Store index=0 at=0 src=0\n"
    );
    assert_eq!(simple.registers, 1);
    assert!(
        simple.consts.is_empty(),
        "a read interns nothing: its value is in a frame slot, not in the chunk"
    );

    let stem = compile_for_test(b"zw = zs.\n").expect("compiles");
    assert_eq!(
        render(&stem),
        "0: Clause index=0 end=3\n\
         1: Load read=Stem at=1 dst=0\n\
         2: Store index=0 at=0 src=0\n"
    );

    let compound = compile_for_test(b"zw = za.zi\n").expect("compiles");
    assert_eq!(
        render(&compound),
        "0: Clause index=0 end=3\n\
         1: Load read=Compound at=- dst=0\n\
         2: Store index=0 at=0 src=0\n"
    );
}

/// **The symbol a compiled read names is the one its own expression names**,
/// which the rendered stream cannot say -- `render`'s own comment has why it
/// prints no symbol index.
#[test]
fn a_compiled_read_names_the_symbol_its_expression_does() {
    let program = parse_program(b"zw = za.zi\n".to_vec()).expect("test program parses");
    let plan = Plan::build(
        &program.main,
        &program.symbols,
        Some(&program.source),
        BodyKind::Plain,
    );
    let chunk = super::compile(&program.main, &plan, intermediates()).expect("compiles");

    let InstructionKind::Assignment { value, .. } = &program.main.instructions[0].kind else {
        panic!("the program's one instruction is an assignment");
    };
    let ExprKind::Compound(id) = &value.kind else {
        panic!("its value is a compound");
    };
    assert_eq!(
        program.symbols.name(*id),
        "ZA.ZI",
        "the expression names the compound, not the target"
    );

    let named: Vec<_> = chunk
        .ops
        .iter()
        .filter_map(|op| match op {
            super::Op::Load { symbol, .. } | super::Op::TraceRead { symbol, .. } => Some(*symbol),
            _ => None,
        })
        .collect();
    assert_eq!(
        named,
        vec![*id, *id],
        "the load and its echo both name the symbol the expression does"
    );
}

/// A `SAY` whose whole expression is a bare symbol reads it natively too, so
/// the promotion is the expression's rather than the assignment's.
#[test]
fn a_say_of_a_bare_symbol_compiles_to_a_native_read() {
    let chunk = compile_for_test(b"say zv\n").expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=3\n\
         1: Load read=Simple at=0 dst=0\n\
         2: Say index=0 src=0\n"
    );
}

/// **An expression that merely *contains* a symbol is not a read**, and the
/// symbol's own load is not the expression's value.
#[test]
fn an_expression_that_only_contains_a_symbol_is_more_than_that_symbols_read() {
    let chunk = compile_for_test(b"zw = zv + 1\n").expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=5\n\
         1: Load read=Simple at=1 dst=0\n\
         2: LoadConstant dst=1\n\
         3: Arith op=+ hint=0 lhs=0 rhs=1 dst=0\n\
         4: Store index=0 at=0 src=0\n"
    );

    let dotvar = compile_for_test(b"zw = .nil\n").expect("compiles");
    assert_eq!(
        render(&dotvar),
        "0: Clause index=0 end=3\n\
         1: EvalExpr index=0 slot=0 dst=0\n\
         2: Store index=0 at=0 src=0\n"
    );
    let reference = compile_for_test(b"zw = >zv\n").expect("compiles");
    assert_eq!(
        render(&reference),
        "0: Clause index=0 end=3\n\
         1: EvalExpr index=0 slot=0 dst=0\n\
         2: Store index=0 at=0 src=0\n"
    );
}

/// A chain of operators reuses **two** registers however long it runs, because
/// each operator writes its result back into the register the next one reads as
/// its left operand.
#[test]
fn a_chain_of_operators_reuses_the_destination_register() {
    let chunk = compile_for_test(b"zw = za + zb + zc + zd\n").expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=9\n\
         1: Load read=Simple at=1 dst=0\n\
         2: Load read=Simple at=2 dst=1\n\
         3: Arith op=+ hint=0 lhs=0 rhs=1 dst=0\n\
         4: Load read=Simple at=3 dst=1\n\
         5: Arith op=+ hint=1 lhs=0 rhs=1 dst=0\n\
         6: Load read=Simple at=4 dst=1\n\
         7: Arith op=+ hint=2 lhs=0 rhs=1 dst=0\n\
         8: Store index=0 at=0 src=0\n"
    );
    assert_eq!(
        chunk.registers, 2,
        "a chain of three operators reserved {} registers where two are enough",
        chunk.registers
    );
}

/// Precedence decides the nesting, and the nesting decides the op order -- so
/// `za + zb * zc` multiplies first and needs a third register to hold the
/// product while `za` waits below it.
#[test]
fn precedence_decides_which_operator_is_the_inner_one() {
    let chunk = compile_for_test(b"zw = za + zb * zc\n").expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=7\n\
         1: Load read=Simple at=1 dst=0\n\
         2: Load read=Simple at=2 dst=1\n\
         3: Load read=Simple at=3 dst=2\n\
         4: Arith op=* hint=0 lhs=1 rhs=2 dst=1\n\
         5: Arith op=+ hint=1 lhs=0 rhs=1 dst=0\n\
         6: Store index=0 at=0 src=0\n"
    );
    assert_eq!(
        chunk.registers, 3,
        "a right-nested operator reserved {} registers where three are needed",
        chunk.registers
    );
}

/// **The value shapes that still do not promote**, which stay on
/// [`super::Op::EvalExpr`] entire however ordinary the rest of the expression
/// around them is.
#[test]
fn the_value_shapes_outside_the_native_set_stay_general() {
    for source in [
        // An environment symbol, which traces `>E>` rather than `>V>`.
        &b"zw = .nil\n"[..],
        &b"zw = .nil || za\n"[..],
        // `>name` in a value position, which is a node of its own around the
        // read rather than the read.
        &b"zw = >za\n"[..],
        // A call beside a term with no op: the address reaches the call and
        // the slot still goes general, because the choice is the slot's.
        &b"zw = .nil || length('a')\n"[..],
    ] {
        let chunk = compile_for_test(source).expect("compiles");
        assert_eq!(
            render(&chunk),
            "0: Clause index=0 end=3\n\
             1: EvalExpr index=0 slot=0 dst=0\n\
             2: Store index=0 at=0 src=0\n",
            "{} promoted a value shape outside the native set",
            String::from_utf8_lossy(source)
        );
    }

    // The adjacent success: the same concatenation with the environment symbol
    // replaced by an ordinary variable does promote, so the row above is about
    // the term rather than about the operator holding it.
    let promoted = compile_for_test(b"zw = zv || za\n").expect("compiles");
    assert!(
        render(&promoted).contains("Binary op=||"),
        "the same shape without the environment symbol did not promote either\n{}",
        render(&promoted)
    );
}

/// A concatenation, a comparison and a logical operator each compile to
/// `Op::Binary`, and arithmetic still compiles to `Op::Arith`.
#[test]
fn every_binary_operator_but_arithmetic_compiles_to_one_op() {
    // One operator per family, over the same two operands, so the streams
    // differ in the operator alone.
    for (source, spelling) in [
        (&b"za = zb || zc\n"[..], "||"),
        (&b"za = zb = zc\n"[..], "="),
        (&b"za = zb & zc\n"[..], "&"),
        (&b"za = zb zc\n"[..], "blank"),
    ] {
        let chunk = compile_for_test(source).expect("compiles");
        assert_eq!(
            render(&chunk),
            format!(
                "0: Clause index=0 end=5\n\
                 1: Load read=Simple at=1 dst=0\n\
                 2: Load read=Simple at=2 dst=1\n\
                 3: Binary op={spelling} lhs=0 rhs=1 dst=0\n\
                 4: Store index=0 at=0 src=0\n"
            ),
            "{} did not compile to one Binary op",
            String::from_utf8_lossy(source)
        );
    }

    let chunk = compile_for_test(b"za = zb + zc\n").expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=5\n\
         1: Load read=Simple at=1 dst=0\n\
         2: Load read=Simple at=2 dst=1\n\
         3: Arith op=+ hint=0 lhs=0 rhs=1 dst=0\n\
         4: Store index=0 at=0 src=0\n"
    );
}

/// A prefix operator compiles to its own op and its own echo, and the echo
/// carries the operator: `\` and `-` trace different lines from one value.
#[test]
fn a_prefix_operator_compiles_to_a_native_op_and_its_own_echo() {
    for (source, spelling) in [
        (&b"za = +zb\n"[..], "+"),
        (&b"za = -zb\n"[..], "-"),
        (&b"za = \\zb\n"[..], "\\"),
    ] {
        let chunk = compile_for_test(source).expect("compiles");
        assert_eq!(
            render(&chunk),
            format!(
                "0: Clause index=0 end=4\n\
                 1: Load read=Simple at=1 dst=0\n\
                 2: Prefix op={spelling} src=0 dst=0\n\
                 3: Store index=0 at=0 src=0\n"
            ),
            "{} did not compile to one Prefix op and its own echo",
            String::from_utf8_lossy(source)
        );
    }
}

/// A bare constant symbol is a native load of its own, the way a quoted literal
/// is -- and the two produce the same `>L>` echo, so one op serves both.
#[test]
fn a_constant_symbol_is_a_native_load() {
    let chunk = compile_for_test(b"zw = 1\n").expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=3\n\
         1: LoadConstant dst=0\n\
         2: Store index=0 at=0 src=0\n"
    );

    // A quoted literal is the other load, against the chunk's own interned
    // table, and it takes the identical echo.
    let quoted = compile_for_test(b"zw = '1'\n").expect("compiles");
    assert_eq!(
        render(&quoted),
        "0: Clause index=0 end=3\n\
         1: Const dst=0 konst=0\n\
         2: Store index=0 at=0 src=0\n"
    );
}

/// The compiled form of the plan's own example loop: the header is a clause
/// region of its own and the construct is the op that closes it.
#[test]
fn a_counted_loop_compiles_its_header_to_a_clause_region_and_its_body_to_a_marker() {
    let chunk = compile_for_test(b"do i = 1 to 3\n  nop\nend\n").expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=6\n\
         1: LoadConstant dst=0\n\
         2: LoopHeaderValue role=Initial src=0\n\
         3: LoadConstant dst=1\n\
         4: LoopHeaderValue role=To src=1\n\
         5: LoopRun index=0\n\
         6: Clause index=1 end=7\n\
         7: LoopNext index=0\n"
    );
    // One register per header expression, and they are **not** released at the
    // region's end: the loop runs from op 8 with the body's clauses stepped
    // between, so a register handed out again there would be overwritten while
    // the running loop still reads it.
    assert_eq!(chunk.registers, 2);
    assert_eq!(chunk.op_of, vec![0, 6, 7, 8]);
}

/// The same loop under `TRACE R`: the clause echo is an op of the region, and
/// so is the header's own `>K>` line.
#[test]
fn a_traced_counted_loop_echoes_its_do_clause_from_the_stream() {
    let chunk = compile_for_test_under(b"do i = 1 to 3\n  nop\nend\n", traced()).expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=8\n\
         1: TraceClause index=0\n\
         2: LoadConstant dst=0\n\
         3: LoopHeaderValue role=Initial src=0\n\
         4: LoadConstant dst=1\n\
         5: TraceKeyword role=To src=1\n\
         6: LoopHeaderValue role=To src=1\n\
         7: LoopRun index=0\n\
         8: Clause index=1 end=10\n\
         9: TraceClause index=1\n\
         10: LoopNext index=0\n"
    );
    assert_eq!(chunk.registers, 2);
}

/// A block, and a `DO OVER`: the two ends of how much header a `DO`/`LOOP` can
/// have, and both still one clause region ending in the construct.
#[test]
fn a_block_has_an_empty_header_region_and_a_do_over_for_echoes_both_its_target_and_count() {
    let block = compile_for_test(b"do\n  nop\nend\n").expect("compiles");
    assert_eq!(
        render(&block),
        "0: Clause index=0 end=2\n\
         1: LoopRun index=0\n\
         2: Clause index=1 end=3\n\
         3: Clause index=2 end=5\n\
         4: Exec index=2\n"
    );
    assert_eq!(block.registers, 0, "a block evaluates nothing to hold");

    let over = compile_for_test(b"do qq over 4.5 for 2\n  nop\nend\n").expect("compiles");
    assert_eq!(
        render(&over),
        "0: Clause index=0 end=6\n\
         1: LoadConstant dst=0\n\
         2: LoopHeaderValue role=Over src=0\n\
         3: LoadConstant dst=1\n\
         4: LoopHeaderValue role=OverFor src=1\n\
         5: LoopRun index=0\n\
         6: Clause index=1 end=7\n\
         7: LoopNext index=0\n"
    );
    assert_eq!(over.registers, 2);
}

/// A header bound that is a bare symbol, and one that is a call.
#[test]
fn a_header_bound_that_is_a_symbol_and_one_that_is_a_call_take_their_own_ops() {
    let symbol = compile_for_test(b"do i = 1 to zn\n  nop\nend\n").expect("compiles");
    assert_eq!(
        render(&symbol),
        "0: Clause index=0 end=6\n\
         1: LoadConstant dst=0\n\
         2: LoopHeaderValue role=Initial src=0\n\
         3: Load read=Simple at=1 dst=1\n\
         4: LoopHeaderValue role=To src=1\n\
         5: LoopRun index=0\n\
         6: Clause index=1 end=7\n\
         7: LoopNext index=0\n"
    );

    let call = compile_for_test(b"do i = 1 to length(zs)\n  nop\nend\n").expect("compiles");
    assert_eq!(
        render(&call),
        "0: Clause index=0 end=9\n\
         1: LoadConstant dst=0\n\
         2: LoopHeaderValue role=Initial src=0\n\
         3: Load read=Simple at=1 dst=2\n\
         4: PushArg src=2\n\
         5: CallArgs slot=1 path=root site=0 argc=1 dst=1\n\
         6: TraceFunction index=0 slot=1 path=root src=1\n\
         7: LoopHeaderValue role=To src=1\n\
         8: LoopRun index=0\n\
         9: Clause index=1 end=10\n\
         10: LoopNext index=0\n"
    );
}

/// **One header expression outside the native set leaves the others native**,
/// because each slot is its own decision.
#[test]
fn a_header_slot_outside_the_native_set_leaves_the_other_slots_native() {
    let chunk = compile_for_test(b"do i = .nil to 3\n  nop\nend\n").expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=6\n\
         1: EvalExpr index=0 slot=0 dst=0\n\
         2: LoopHeaderValue role=Initial src=0\n\
         3: LoadConstant dst=1\n\
         4: LoopHeaderValue role=To src=1\n\
         5: LoopRun index=0\n\
         6: Clause index=1 end=7\n\
         7: LoopNext index=0\n"
    );
}

/// **A register `push_native` took for an operand inside a header goes back
/// before the body's clauses are emitted**, and the header's own registers do
/// not.
#[test]
fn a_header_operands_register_goes_back_to_the_body() {
    let chunk = compile_for_test(b"do i = 1 to zn + 1\n  zx = 5\nend\n").expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=8\n\
         1: LoadConstant dst=0\n\
         2: LoopHeaderValue role=Initial src=0\n\
         3: Load read=Simple at=1 dst=1\n\
         4: LoadConstant dst=2\n\
         5: Arith op=+ hint=0 lhs=1 rhs=2 dst=1\n\
         6: LoopHeaderValue role=To src=1\n\
         7: LoopRun index=0\n\
         8: Clause index=1 end=11\n\
         9: LoadConstant dst=2\n\
         10: Store index=1 at=2 src=2\n\
         11: LoopNext index=0\n"
    );
    assert_eq!(
        chunk.registers, 3,
        "the operand's register is the body's, so three is the high-water mark"
    );
}

/// **A nested loop's header registers sit above the enclosing loop's, and a
/// following loop's reuse them.** This is the plan's Decisions section in an
/// emitted stream: "a construct whose state outlives its member clauses
/// allocates in the enclosing scope, before emitting them, so those releases
/// cannot reclaim it."
#[test]
fn a_nested_loops_registers_sit_above_the_enclosing_loops_and_a_later_loops_reuse_them() {
    let nested = compile_for_test(b"do i = 1 to 2\n  do j = 1 to 2\n    nop\n  end\nend\n")
        .expect("compiles");
    assert_eq!(
        render(&nested),
        "0: Clause index=0 end=6\n\
         1: LoadConstant dst=0\n\
         2: LoopHeaderValue role=Initial src=0\n\
         3: LoadConstant dst=1\n\
         4: LoopHeaderValue role=To src=1\n\
         5: LoopRun index=0\n\
         6: Clause index=1 end=12\n\
         7: LoadConstant dst=2\n\
         8: LoopHeaderValue role=Initial src=2\n\
         9: LoadConstant dst=3\n\
         10: LoopHeaderValue role=To src=3\n\
         11: LoopRun index=1\n\
         12: Clause index=2 end=13\n\
         13: LoopNext index=1\n\
         14: LoopNext index=0\n"
    );
    assert_eq!(
        nested.registers, 4,
        "two loops are live at once, and never more"
    );

    let sequential = compile_for_test(b"do i = 1 to 2\n  nop\nend\ndo j = 1 to 2\n  nop\nend\n")
        .expect("compiles");
    assert_eq!(
        sequential.registers, 2,
        "the second loop reuses the registers the first one released past its END"
    );
    assert!(
        render(&sequential).contains(
            "11: LoadConstant dst=1\n12: LoopHeaderValue role=To src=1\n13: LoopRun index=3"
        ),
        "the second loop\'s own bound went somewhere other than register 1: {}",
        render(&sequential)
    );
}

/// The compiled `IF` with an `ELSE`, which is the shape the whole promotion
/// is about: both paths are jumps in one stream where the tree-walker splits
/// them across two engines.
#[test]
fn an_if_with_an_else_compiles_to_a_clause_region_and_two_jumps() {
    let chunk =
        compile_for_test(b"if 1 = 1 then say 'a'\nelse say 'b'\nsay 'c'\n").expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=6\n\
         1: LoadConstant dst=0\n\
         2: LoadConstant dst=1\n\
         3: Binary op== lhs=0 rhs=1 dst=0\n\
         4: Condition index=0 reg=0 keyword=IF\n\
         5: JumpUnless reg=0 target=12\n\
         6: Clause index=1 end=7\n\
         7: Clause index=2 end=10\n\
         8: Const dst=0 konst=0\n\
         9: Say index=2 src=0\n\
         10: EndBranch\n\
         11: Jump target=16\n\
         12: Clause index=3 end=13\n\
         13: Clause index=4 end=16\n\
         14: Const dst=0 konst=1\n\
         15: Say index=4 src=0\n\
         16: Clause index=5 end=19\n\
         17: Const dst=0 konst=2\n\
         18: Say index=5 src=0\n"
    );
    // Two registers throughout: the comparison's right operand takes one of
    // its own beside the register the condition lands in, and gives it back
    // inside `push_native`'s own binary arm as soon as the operation has run.
    // The condition's own register is released at the `IF`'s clause end, so
    // each promoted `SAY` below gets register 0 back.
    assert_eq!(chunk.registers, 2);
    assert_eq!(chunk.op_of, vec![0, 6, 7, 10, 13, 16, 19]);
    // Three distinct literals, one entry each, in the order they were first
    // seen -- which is the order the `konst` fields above read.
    assert_eq!(
        chunk.consts,
        vec![
            Box::from(&b"a"[..]),
            Box::from(&b"b"[..]),
            Box::from(&b"c"[..])
        ]
    );
}

/// Without an `ELSE` **no jump is emitted at all**: the two targets are the
/// same instruction, and an op that jumps to where control was already going
/// is one the driver would run for nothing.
#[test]
fn an_if_with_no_else_emits_no_branch_end_jump() {
    let chunk = compile_for_test(b"if 1 = 0 then say 'a'\nsay 'b'\n").expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=6\n\
         1: LoadConstant dst=0\n\
         2: LoadConstant dst=1\n\
         3: Binary op== lhs=0 rhs=1 dst=0\n\
         4: Condition index=0 reg=0 keyword=IF\n\
         5: JumpUnless reg=0 target=11\n\
         6: Clause index=1 end=7\n\
         7: Clause index=2 end=10\n\
         8: Const dst=0 konst=0\n\
         9: Say index=2 src=0\n\
         10: EndBranch\n\
         11: Clause index=3 end=14\n\
         12: Const dst=0 konst=1\n\
         13: Say index=3 src=0\n"
    );
    assert_eq!(chunk.registers, 2);
}

/// **A condition outside the native set stays one [`super::Op::EvalExpr`] and
/// takes no `Condition` op**, which is the adjacent refusal to the two streams
/// above.
#[test]
fn a_condition_outside_the_native_set_stays_one_eval_expr() {
    let chunk = compile_for_test(b"if .nil then nop\n").expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=3\n\
         1: EvalExpr index=0 slot=0 dst=0\n\
         2: JumpUnless reg=0 target=6\n\
         3: Clause index=1 end=4\n\
         4: Clause index=2 end=5\n\
         5: EndBranch\n"
    );
    assert_eq!(chunk.registers, 1);
}

/// **One `EndBranch` per `IF`, inner before outer.** Control leaves the inner
/// branch first, and the oracle runs one synthetic end-of-branch instruction
/// per branch rather than one per position -- so a handler delivered at the
/// inner one that queues again has the outer one left to deliver it. Both
/// `JumpUnless` targets are past both, because neither false path ran a
/// branch.
#[test]
fn nested_ifs_reuse_their_registers() {
    let chunk =
        compile_for_test(b"if 1 = 1 then\n  if 2 = 2 then say 'a'\nsay 'b'\n").expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=6\n\
         1: LoadConstant dst=0\n\
         2: LoadConstant dst=1\n\
         3: Binary op== lhs=0 rhs=1 dst=0\n\
         4: Condition index=0 reg=0 keyword=IF\n\
         5: JumpUnless reg=0 target=19\n\
         6: Clause index=1 end=7\n\
         7: Clause index=2 end=13\n\
         8: LoadConstant dst=0\n\
         9: LoadConstant dst=1\n\
         10: Binary op== lhs=0 rhs=1 dst=0\n\
         11: Condition index=2 reg=0 keyword=IF\n\
         12: JumpUnless reg=0 target=19\n\
         13: Clause index=3 end=14\n\
         14: Clause index=4 end=17\n\
         15: Const dst=0 konst=0\n\
         16: Say index=4 src=0\n\
         17: EndBranch\n\
         18: EndBranch\n\
         19: Clause index=5 end=22\n\
         20: Const dst=0 konst=1\n\
         21: Say index=5 src=0\n"
    );
    assert_eq!(
        chunk.registers, 2,
        "the inner IF reuses the registers the outer one released"
    );
}

/// The compiled `SELECT` with an `OTHERWISE`: one clause region for the header
/// and one per listed `WHEN`, chained by the `JumpUnless` each `WHEN` ends
/// with, and a frame opened over whichever branch wins.
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
         2: Clause index=1 end=8\n\
         3: LoadConstant dst=0\n\
         4: LoadConstant dst=1\n\
         5: Binary op== lhs=0 rhs=1 dst=0\n\
         6: Condition index=1 reg=0 keyword=WHEN\n\
         7: JumpUnless reg=0 target=14\n\
         8: EnterWhen select=0 when=1\n\
         9: Clause index=2 end=10\n\
         10: Clause index=3 end=13\n\
         11: Const dst=0 konst=0\n\
         12: Say index=3 src=0\n\
         13: EndWhen\n\
         14: Clause index=4 end=20\n\
         15: LoadConstant dst=0\n\
         16: LoadConstant dst=1\n\
         17: Binary op== lhs=0 rhs=1 dst=0\n\
         18: Condition index=4 reg=0 keyword=WHEN\n\
         19: JumpUnless reg=0 target=25\n\
         20: EnterWhen select=0 when=4\n\
         21: Clause index=5 end=22\n\
         22: Clause index=6 end=25\n\
         23: Const dst=0 konst=1\n\
         24: Say index=6 src=0\n\
         25: EndWhen\n\
         26: EnterOtherwise select=0\n\
         27: Clause index=7 end=28\n\
         28: Clause index=8 end=31\n\
         29: Const dst=0 konst=2\n\
         30: Say index=8 src=0\n\
         31: EndWhen\n\
         32: Clause index=9 end=34\n\
         33: Exec index=9\n\
         34: Clause index=10 end=37\n\
         35: Const dst=0 konst=3\n\
         36: Say index=10 src=0\n"
    );
    assert_eq!(
        chunk.registers, 2,
        "the second WHEN reuses the registers the first one released, and a \
         comparison's right operand takes one beside the answer's own"
    );
    assert_eq!(
        chunk.op_of,
        vec![0, 2, 9, 10, 13, 21, 22, 25, 28, 31, 34, 37]
    );
}

/// A `SELECT CASE`'s own value is allocated in the **enclosing** scope, so the
/// register a `WHEN` takes for its own answer cannot reclaim it.
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
         5: JumpUnless reg=1 target=12\n\
         6: EnterWhen select=0 when=1\n\
         7: Clause index=2 end=8\n\
         8: Clause index=3 end=11\n\
         9: Const dst=1 konst=0\n\
         10: Say index=3 src=1\n\
         11: EndWhen\n\
         12: Clause index=4 end=15\n\
         13: WhenTest index=4 case=0 dst=1\n\
         14: JumpUnless reg=1 target=21\n\
         15: EnterWhen select=0 when=4\n\
         16: Clause index=5 end=17\n\
         17: Clause index=6 end=20\n\
         18: Const dst=1 konst=1\n\
         19: Say index=6 src=1\n\
         20: EndWhen\n\
         21: Clause index=7 end=23\n\
         22: Exec index=7\n\
         23: Clause index=8 end=26\n\
         24: Const dst=0 konst=2\n\
         25: Say index=8 src=0\n"
    );
    assert_eq!(
        chunk.registers, 2,
        "the CASE value and one WHEN answer are live at once, and never more"
    );
}

/// **A `WHEN` whose body is an `IF` owes two boundaries at one instruction,
/// and the branch's own goes first.** The `IF`'s [`super::Op::EndBranch`] and
/// the `WHEN`'s [`super::Op::EndWhen`] both sit in front of the `OTHERWISE`
/// here. Leaving the branch sends control to the `SELECT`'s own end, so with
/// `EndWhen` in front the `EndBranch` is never reached -- which is what the
/// driver did before the branch end was an op, when it tested for an ended
/// frame ahead of fetching anything.
#[test]
fn a_whens_branch_end_is_emitted_in_front_of_an_ifs() {
    let chunk =
        compile_for_test(b"select\n  when 1 = 1 then if 1 = 1 then nop\n  otherwise nop\nend\n")
            .expect("compiles");
    let stream = render(&chunk);
    let end_when = stream
        .find(": EndWhen")
        .expect("the WHEN's branch end is emitted");
    let end_branch = stream
        .find(": EndBranch")
        .expect("the IF's branch end is emitted");
    assert!(
        end_when < end_branch,
        "the IF's EndBranch is emitted in front of the WHEN's EndWhen, so leaving the branch \
         runs a clause boundary the tree-walker does not: {stream}"
    );
}

/// Without an `OTHERWISE` the scan runs out onto the `END`, whose own 7.3 is
/// what "every WHEN was false" means -- so the last `WHEN`'s `JumpUnless`
/// names the `END`'s **own** op (18, the `Op::Clause` that opens its region)
/// and no frame is open when it runs.
#[test]
fn a_select_with_no_otherwise_scans_out_onto_its_own_end() {
    let chunk = compile_for_test(b"select\n  when 1 = 0 then say 'a'\nend\nsay 'after'\n")
        .expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=1\n\
         1: SelectCaseText index=0 case=-\n\
         2: Clause index=1 end=8\n\
         3: LoadConstant dst=0\n\
         4: LoadConstant dst=1\n\
         5: Binary op== lhs=0 rhs=1 dst=0\n\
         6: Condition index=1 reg=0 keyword=WHEN\n\
         7: JumpUnless reg=0 target=14\n\
         8: EnterWhen select=0 when=1\n\
         9: Clause index=2 end=10\n\
         10: Clause index=3 end=13\n\
         11: Const dst=0 konst=0\n\
         12: Say index=3 src=0\n\
         13: EndWhen\n\
         14: Clause index=4 end=16\n\
         15: Exec index=4\n\
         16: Clause index=5 end=19\n\
         17: Const dst=0 konst=1\n\
         18: Say index=5 src=0\n"
    );
}

/// **A `WHEN`'s condition outside the native set stays one
/// [`super::Op::WhenTest`] and takes no `Condition` op**, which is the
/// adjacent refusal to the two `SELECT` streams above and the sibling of
/// `a_condition_outside_the_native_set_stays_one_eval_expr`.
#[test]
fn a_when_condition_outside_the_native_set_stays_one_when_test() {
    let chunk = compile_for_test(b"select\n  when .nil then nop\nend\n").expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=1\n\
         1: SelectCaseText index=0 case=-\n\
         2: Clause index=1 end=5\n\
         3: WhenTest index=1 case=- dst=0\n\
         4: JumpUnless reg=0 target=9\n\
         5: EnterWhen select=0 when=1\n\
         6: Clause index=2 end=7\n\
         7: Clause index=3 end=8\n\
         8: EndWhen\n\
         9: Clause index=4 end=11\n\
         10: Exec index=4\n"
    );
}

/// A call inside a `WHEN`'s condition takes an [`super::Op::CallExpr`]
/// addressed at slot `0` and the route down to it, which is what
/// `Interp::chunk_node_at`'s own `When` arm resolves. `root.L` is that route:
/// the `>` is the condition's own root and the call is its left operand.
#[test]
fn a_call_in_a_whens_condition_is_addressed_at_the_conditions_slot() {
    let chunk = compile_for_test(b"zs = 'abcd'\nselect\n  when length(zs) > 3 then nop\nend\n")
        .expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=3\n\
         1: Const dst=0 konst=0\n\
         2: Store index=0 at=0 src=0\n\
         3: Clause index=1 end=4\n\
         4: SelectCaseText index=1 case=-\n\
         5: Clause index=2 end=14\n\
         6: Load read=Simple at=0 dst=1\n\
         7: PushArg src=1\n\
         8: CallArgs slot=0 path=root.L site=0 argc=1 dst=0\n\
         9: TraceFunction index=2 slot=0 path=root.L src=0\n\
         10: LoadConstant dst=1\n\
         11: Binary op=> lhs=0 rhs=1 dst=0\n\
         12: Condition index=2 reg=0 keyword=WHEN\n\
         13: JumpUnless reg=0 target=18\n\
         14: EnterWhen select=1 when=2\n\
         15: Clause index=3 end=16\n\
         16: Clause index=4 end=17\n\
         17: EndWhen\n\
         18: Clause index=5 end=20\n\
         19: Exec index=5\n"
    );
}

/// An **absorbed** `WHEN` -- one that is itself another `WHEN`'s consequence
/// rather than a branch its `SELECT` collected -- compiles to a region of its
/// own holding a single [`super::Op::Exec`].
#[test]
fn an_absorbed_when_compiles_to_its_own_region() {
    let chunk = compile_for_test(b"select\n  when 1 = 1 then when 2 = 2 then nop\nend\n")
        .expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=1\n\
         1: SelectCaseText index=0 case=-\n\
         2: Clause index=1 end=8\n\
         3: LoadConstant dst=0\n\
         4: LoadConstant dst=1\n\
         5: Binary op== lhs=0 rhs=1 dst=0\n\
         6: Condition index=1 reg=0 keyword=WHEN\n\
         7: JumpUnless reg=0 target=15\n\
         8: EnterWhen select=0 when=1\n\
         9: Clause index=2 end=10\n\
         10: Clause index=3 end=12\n\
         11: Exec index=3\n\
         12: EndWhen\n\
         13: Clause index=4 end=14\n\
         14: Clause index=5 end=15\n\
         15: Clause index=6 end=17\n\
         16: Exec index=6\n"
    );
}

/// The same `IF` compiled under `TRACE R`: the clause echo is an **op** of the
/// clause's own region, and the region is one op longer for it.
#[test]
fn a_traced_if_carries_its_clause_echo_as_an_op_of_the_region() {
    let chunk = compile_for_test_under(b"if 1 = 1 then say 'a'\nelse say 'b'\nsay 'c'\n", traced())
        .expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=7\n\
         1: TraceClause index=0\n\
         2: LoadConstant dst=0\n\
         3: LoadConstant dst=1\n\
         4: Binary op== lhs=0 rhs=1 dst=0\n\
         5: Condition index=0 reg=0 keyword=IF\n\
         6: JumpUnless reg=0 target=15\n\
         7: Clause index=1 end=9\n\
         8: TraceClause index=1\n\
         9: Clause index=2 end=13\n\
         10: TraceClause index=2\n\
         11: Const dst=0 konst=0\n\
         12: Say index=2 src=0\n\
         13: EndBranch\n\
         14: Jump target=21\n\
         15: Clause index=3 end=17\n\
         16: TraceClause index=3\n\
         17: Clause index=4 end=21\n\
         18: TraceClause index=4\n\
         19: Const dst=0 konst=1\n\
         20: Say index=4 src=0\n\
         21: Clause index=5 end=25\n\
         22: TraceClause index=5\n\
         23: Const dst=0 konst=2\n\
         24: Say index=5 src=0\n"
    );
    // The echo op addresses no register, so the extra op changes nothing the
    // driver has to reserve.
    assert_eq!(chunk.registers, 2);
    assert_eq!(chunk.op_of, vec![0, 7, 9, 13, 17, 21, 25]);
}

/// A traced `SELECT CASE`: **one echo per promoted clause, and exactly one**.
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
         7: JumpUnless reg=1 target=16\n\
         8: EnterWhen select=0 when=1\n\
         9: Clause index=2 end=11\n\
         10: TraceClause index=2\n\
         11: Clause index=3 end=15\n\
         12: TraceClause index=3\n\
         13: Const dst=1 konst=0\n\
         14: Say index=3 src=1\n\
         15: EndWhen\n\
         16: Clause index=4 end=20\n\
         17: TraceClause index=4\n\
         18: WhenTest index=4 case=0 dst=1\n\
         19: JumpUnless reg=1 target=28\n\
         20: EnterWhen select=0 when=4\n\
         21: Clause index=5 end=23\n\
         22: TraceClause index=5\n\
         23: Clause index=6 end=27\n\
         24: TraceClause index=6\n\
         25: Const dst=1 konst=1\n\
         26: Say index=6 src=1\n\
         27: EndWhen\n\
         28: Clause index=7 end=31\n\
         29: TraceClause index=7\n\
         30: Exec index=7\n\
         31: Clause index=8 end=35\n\
         32: TraceClause index=8\n\
         33: Const dst=0 konst=2\n\
         34: Say index=8 src=0\n"
    );
    assert_eq!(
        chunk.registers, 2,
        "the CASE value and one WHEN answer are live at once, and never more"
    );
}

/// **Two constructs ending at the same instruction both hand their registers
/// back there.** A `DO` block closing one instruction before the `SELECT CASE`
/// whose `WHEN` holds it wants the same release point, and the point releases to
/// the *lower* of the two marks -- by that instruction both constructs are over,
/// so every register above the lower one is dead.
#[test]
fn two_constructs_ending_at_one_instruction_release_to_the_lower_mark() {
    let chunk = compile_for_test(
        b"select case 1\n  when 1 then do i = 1 to 2\n    nop\n  end\nend\ndo j = 1 to 2\n            nop\nend\n",
    )
    .expect("compiles");
    let stream = render(&chunk);
    // **Located by the last `EndWhen` rather than by op index.** The indices
    // here move whenever anything before them changes how many ops an
    // instruction takes -- promoting the `END` off `Op::Generic` shifted them
    // by one -- and an index in the pattern makes this test report a register
    // that was handed back correctly as one that was not.
    let after_select = stream
        .rsplit_once("EndWhen\n")
        .expect("the SELECT closes a branch")
        .1;
    assert!(
        after_select.contains("LoadConstant dst=0\n")
            && after_select.contains("LoopHeaderValue role=Initial src=0\n"),
        "the loop after the whole SELECT did not get register 0 back: {stream}"
    );
    assert_eq!(
        chunk.registers, 3,
        "the SELECT CASE value and the inner loop's two bounds are live at once: {stream}"
    );
}

/// `run_bounded`'s absorption guard is inclusive, so a construct's resume
/// point can be `end`, which is one past its last instruction. A map that
/// stops at `len - 1` panics there rather than at compile.
#[test]
fn the_instruction_map_has_an_entry_one_past_the_last_instruction() {
    let source = b"if 1 = 1 then say 'a'\nsay 'b'\n";
    let program = parse_program(source.to_vec()).expect("test program parses");
    let plan = Plan::build(
        &program.main,
        &program.symbols,
        Some(&program.source),
        BodyKind::Plain,
    );
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
    let plan = Plan::build(
        &program.main,
        &program.symbols,
        Some(&program.source),
        BodyKind::Plain,
    );

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
#[test]
fn one_body_under_two_trace_settings_is_two_cached_chunks() {
    let program = parse_program(b"if 1 = 1 then say 1\n".to_vec()).expect("test program parses");
    let key = BodyKey {
        program: ProgramId(0),
        directive: None,
    };
    let plan = Plan::build(
        &program.main,
        &program.symbols,
        Some(&program.source),
        BodyKind::Plain,
    );
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

/// A `CALL` compiles to a clause region whose arguments are ops of their own,
/// ending in [`super::Op::CallNamed`].
#[test]
fn a_call_compiles_its_arguments_to_ops_and_one_call_op() {
    let chunk = compile_for_test(b"call zsub 1\n").expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=4\n\
         1: LoadConstant dst=0\n\
         2: PushArg src=0\n\
         3: CallNamed index=0 site=0 argc=1\n"
    );
    assert_eq!(chunk.registers, 1);
}

/// **The two argument shapes that keep a `CALL` on [`super::Op::Call`]**, each
/// beside the promoted form above so that what separates them is the argument
/// and nothing else.
#[test]
fn an_argument_that_is_not_a_plain_value_keeps_the_whole_call_unpromoted() {
    let nested = compile_for_test(b"call zsub length('x')\n").expect("compiles");
    assert_eq!(
        render(&nested),
        "0: Clause index=0 end=2\n\
         1: Call index=0 site=0\n"
    );
    assert_eq!(nested.registers, 0);
    // Nothing interned either: the argument's literal is evaluated from its
    // own node by `invoke_call`, not loaded from this chunk's table.
    assert!(nested.consts.is_empty());

    let reference = compile_for_test(b"zp = 1\ncall zsub >zp\n").expect("compiles");
    assert!(
        render(&reference).contains("Call index=1 site=0\n"),
        "a `>name` argument compiled to something other than Op::Call:\n{}",
        render(&reference)
    );
    assert!(
        !render(&reference).contains("CallNamed"),
        "a `>name` argument reached Op::CallNamed, which carries values and \
         not variable homes:\n{}",
        render(&reference)
    );
}

/// **Each call site takes a site index of its own, dense over the call ops.**
#[test]
fn each_call_site_takes_a_resolution_site_of_its_own() {
    let chunk = compile_for_test(b"call zsub 1\ncall zsub 2\n").expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=4\n\
         1: LoadConstant dst=0\n\
         2: PushArg src=0\n\
         3: CallNamed index=0 site=0 argc=1\n\
         4: Clause index=1 end=8\n\
         5: LoadConstant dst=0\n\
         6: PushArg src=0\n\
         7: CallNamed index=1 site=1 argc=1\n"
    );
}

/// Under a setting that echoes, the region carries its clause echo op in front
/// of the call -- the position the tree-walker's own clause unit echoes at,
/// before anything the clause does.
#[test]
fn a_traced_call_carries_its_clause_echo_in_front_of_the_call() {
    let chunk = compile_for_test_under(b"call zsub 1\n", traced()).expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=5\n\
         1: TraceClause index=0\n\
         2: LoadConstant dst=0\n\
         3: PushArg src=0\n\
         4: CallNamed index=0 site=0 argc=1\n"
    );
}

/// **`CALL ON`/`CALL OFF` and `CALL (expr)` carry no resolved call site**,
/// and each carries none for a reason of its own rather than by oversight.
#[test]
fn a_trap_call_and_a_dynamic_call_keep_no_resolved_site() {
    let chunk = compile_for_test(b"call on error name zh\ncall (zn)\n").expect("compiles");
    let rendered = render(&chunk);
    assert_eq!(
        rendered,
        "0: Clause index=0 end=2\n\
         1: Exec index=0\n\
         2: Clause index=1 end=4\n\
         3: Exec index=1\n"
    );
    // The half the stream above states only by absence, asserted rather than
    // left to be read out of it: neither form emits the op that carries a
    // resolution, so neither can carry one.
    assert!(
        !rendered.contains("CallNamed"),
        "a call with nothing to resolve compiled to a resolved call\n{rendered}"
    );
    assert_eq!(chunk.registers, 0);
}

/// **A compiled write carries the plan's slot for a simple target and carries
/// none for the other two**, which is the whole of what `write_slot` decides.
#[test]
fn a_compiled_write_names_a_slot_only_for_a_simple_target() {
    let simple = compile_for_test(b"zw = 'v'\n").expect("compiles");
    assert_eq!(
        render(&simple),
        "0: Clause index=0 end=3\n\
         1: Const dst=0 konst=0\n\
         2: Store index=0 at=0 src=0\n"
    );

    for source in [&b"zs. = 'v'\n"[..], &b"zs.zk = 'v'\n"[..]] {
        let chunk = compile_for_test(source).expect("compiles");
        assert_eq!(
            render(&chunk),
            "0: Clause index=0 end=3\n\
             1: Const dst=0 konst=0\n\
             2: Store index=0 at=- src=0\n",
            "{} resolved a slot for a target that does not read one",
            String::from_utf8_lossy(source)
        );
    }
}

/// The slot a write carries is the one the **plan** gives that name, not the
/// order the writes appear in.
#[test]
fn a_compiled_writes_slot_comes_from_the_plan_rather_than_from_its_position() {
    let chunk = compile_for_test(b"say zb za\nza = 1\nzb = 2\n").expect("compiles");
    assert!(
        render(&chunk).contains("Store index=1 at=1 src=0\n"),
        "the first write did not take the plan's slot for its own name: {}",
        render(&chunk)
    );
    assert!(
        render(&chunk).contains("Store index=2 at=0 src=0\n"),
        "the second write did not take the plan's slot for its own name: {}",
        render(&chunk)
    );
}

/// **A `RETURN` and an `EXIT` compile to one op tagged with their keyword**,
/// with the value's own ops in front of it exactly as a `SAY`'s are, and
/// `src=-` for the bare form.
#[test]
fn a_return_and_an_exit_compile_to_one_op_tagged_with_their_keyword() {
    let chunk = compile_for_test(b"return 1\n").expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=3\n\
         1: LoadConstant dst=0\n\
         2: Return index=0 src=0 keyword=RETURN\n"
    );

    let bare = compile_for_test(b"return\n").expect("compiles");
    assert_eq!(
        render(&bare),
        "0: Clause index=0 end=2\n\
         1: Return index=0 src=- keyword=RETURN\n"
    );

    let exit = compile_for_test(b"exit 1\n").expect("compiles");
    assert_eq!(
        render(&exit),
        "0: Clause index=0 end=3\n\
         1: LoadConstant dst=0\n\
         2: Return index=0 src=0 keyword=EXIT\n"
    );

    let bare_exit = compile_for_test(b"exit\n").expect("compiles");
    assert_eq!(
        render(&bare_exit),
        "0: Clause index=0 end=2\n\
         1: Return index=0 src=- keyword=EXIT\n"
    );
}

/// **A `PUSH` and a `QUEUE` compile to one op tagged with the end of the queue
/// the line lands on**, the bare form included.
#[test]
fn a_push_and_a_queue_compile_to_one_op_tagged_with_their_end() {
    let chunk = compile_for_test(b"push 'a'\nqueue 'b'\npush\nqueue\n").expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=3\n\
         1: Const dst=0 konst=0\n\
         2: Queue index=0 src=0 keyword=PUSH\n\
         3: Clause index=1 end=6\n\
         4: Const dst=0 konst=1\n\
         5: Queue index=1 src=0 keyword=QUEUE\n\
         6: Clause index=2 end=8\n\
         7: Queue index=2 src=- keyword=PUSH\n\
         8: Clause index=3 end=10\n\
         9: Queue index=3 src=- keyword=QUEUE\n"
    );
}

/// **A `RETURN` whose expression is outside the native set keeps its own op**,
/// with one [`super::Op::EvalExpr`] in front of it -- which is the `SAY`
/// fallback and **not** the `IF` one.
#[test]
fn a_return_expression_outside_the_native_set_keeps_its_return_op() {
    let chunk = compile_for_test(b"return .nil\n").expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=3\n\
         1: EvalExpr index=0 slot=0 dst=0\n\
         2: Return index=0 src=0 keyword=RETURN\n"
    );
}

/// A call that is the whole of a `RETURN`'s expression takes an
/// [`super::Op::CallExpr`] addressed at slot `0` with no descent, which is
/// what `Interp::chunk_node_at`'s own arm for these instructions resolves.
#[test]
fn a_call_in_a_returns_expression_is_addressed_at_the_returns_slot() {
    let chunk = compile_for_test(b"return length(zs)\n").expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=6\n\
         1: Load read=Simple at=0 dst=1\n\
         2: PushArg src=1\n\
         3: CallArgs slot=0 path=root site=0 argc=1 dst=0\n\
         4: TraceFunction index=0 slot=0 path=root src=0\n\
         5: Return index=0 src=0 keyword=RETURN\n"
    );
}

/// A promoted `EXIT` compiled under a setting that echoes carries its clause
/// echo as a [`super::Op::TraceClause`] of its own, in front of the value's
/// ops.
#[test]
fn a_traced_exit_carries_its_clause_echo_op() {
    let chunk = compile_for_test_under(b"exit 2\n", traced()).expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=4\n\
         1: TraceClause index=0\n\
         2: LoadConstant dst=0\n\
         3: Return index=0 src=0 keyword=EXIT\n"
    );
}
