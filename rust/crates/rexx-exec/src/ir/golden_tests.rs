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

/// A call compiles to [`super::Op::CallExpr`] wherever an address reaches it
/// inside a slot that compiles natively, and the address the op carries is the
/// route down to it: the same call at the root, as a left operand and as a
/// right operand gets the same op under a different address each time.
///
/// **It is not that a call promotes wherever it is written**, which is the
/// half this test cannot show: the choice is taken for the whole slot, so a
/// sibling term with no op takes the call down with it, and
/// `the_value_shapes_outside_the_native_set_stay_general` is where that row
/// is.
///
/// **The cases together are the test and no one of them alone.** The root case
/// is satisfied by a compiler that promotes only a call that *is* the slot's
/// expression; the left one is satisfied by one that writes a fixed address
/// into every nested call's op; and the right one is what separates the
/// address from the descent that produced it, because a compiler stepping into
/// the left child whatever the operand would render `root.L` under both.
///
/// The `site` field is expected as `0`, which is the first reservation in the
/// chunk -- an assertion about the reservation being dense over call ops, and
/// it would redden if a site were reserved for something that emitted no call
/// op.
///
/// **The register count is the last case and it is not decoration.** A nested
/// call writes the `dst` its parent gave it and takes no register of its own,
/// so two calls over one operator cost what `zb + zc` costs; a call that
/// allocated a register for itself would render the same ops and reserve more.
#[test]
fn a_call_promotes_at_the_root_and_below_it() {
    let root = compile_for_test(b"zz = length('abc')").expect("the chunk fits");
    assert_eq!(
        render(&root),
        "0: Clause index=0 end=4\n\
         1: CallExpr index=0 slot=0 path=root site=0 dst=0\n\
         2: TraceFunction index=0 slot=0 path=root src=0\n\
         3: Store index=0 at=0 src=0\n"
    );

    let left = compile_for_test(b"zz = length('abc') + 1").expect("the chunk fits");
    assert_eq!(
        render(&left),
        "0: Clause index=0 end=8\n\
         1: CallExpr index=0 slot=0 path=root.L site=0 dst=0\n\
         2: TraceFunction index=0 slot=0 path=root.L src=0\n\
         3: LoadConstant dst=1\n\
         4: TraceLiteral src=1\n\
         5: Arith op=+ hint=0 lhs=0 rhs=1 dst=0\n\
         6: TraceOperator op=+ src=0\n\
         7: Store index=0 at=0 src=0\n"
    );

    let right = compile_for_test(b"zz = 1 + length('abc')").expect("the chunk fits");
    assert_eq!(
        render(&right),
        "0: Clause index=0 end=8\n\
         1: LoadConstant dst=0\n\
         2: TraceLiteral src=0\n\
         3: CallExpr index=0 slot=0 path=root.R site=0 dst=1\n\
         4: TraceFunction index=0 slot=0 path=root.R src=1\n\
         5: Arith op=+ hint=0 lhs=0 rhs=1 dst=0\n\
         6: TraceOperator op=+ src=0\n\
         7: Store index=0 at=0 src=0\n"
    );

    let two_calls = compile_for_test(b"zz = length('ab') + length('cd')").expect("the chunk fits");
    assert_eq!(
        two_calls.registers,
        2,
        "two calls over one operator reserved {} registers, where a nested call takes none of \
         its own\n{}",
        two_calls.registers,
        render(&two_calls)
    );
}

/// A call the address cannot reach leaves the whole slot general, which is the
/// answer it had before there was an address at all.
///
/// **The refusal is pinned by its neighbour.** The same shape one operator
/// shallower promotes, and its op spells out every step of the descent, so
/// what the deeper case fixes is `NodePath`'s own width rather than some other
/// bound sitting nearby -- without that half, a compiler giving up at any
/// depth whatever would pass.
///
/// **The width is taken from `NodePath` rather than written down here**, so
/// what this states is that `compile` promotes exactly as deep as an address
/// reaches and no deeper. The width's own number is pinned by
/// `super::tests::a_node_path_carries_thirty_one_steps_and_refuses_the_thirty_second`,
/// which is where a change to it belongs.
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
        rendered.contains(&format!("CallExpr index=0 slot=0 path={address} site=0 ")),
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

/// An instruction whose clause this compiler emits [`super::Op::Generic`] for
/// gets exactly one op, carrying its own index.
///
/// `NOP` and `DROP` are two such instructions, and the choice is load-bearing
/// rather than arbitrary: a program written out of promoted instructions says
/// nothing about `Generic` at all. **Promoting either of these is what should
/// redden this test**, and the answer then is a different unpromoted
/// instruction rather than a new expectation.
#[test]
fn every_instruction_of_an_all_generic_body_compiles_to_one_generic_op() {
    let chunk = compile_for_test(b"nop\nnop\ndrop n1\n").expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Generic index=0\n\
         1: Generic index=1\n\
         2: Generic index=2\n"
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
///
/// The three ops are the whole of what the tree-walker's own `Assignment` arm
/// does around `assign_expr_target`, split into the pieces a stream can order:
///
/// * `Const` loads the interned constant into a register and emits nothing;
/// * `TraceLiteral` is the `>L>` line `eval.rs` produces as a **side effect**
///   of evaluating a literal. It is a separate op for exactly that reason: a
///   constant load emits nothing, every line after the missing one still
///   matches, and only an exact stderr comparison sees it go;
/// * `Store` writes through the target, which is `assign_expr_target` -- the
///   same call `step`'s own arm makes, so a stem, a compound tail and the
///   `>=>` line are one implementation rather than two.
#[test]
fn an_assignment_of_a_literal_compiles_to_a_constant_load_and_a_store() {
    let chunk = compile_for_test(b"n1 = 'abc'\n").expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=4\n\
         1: Const dst=0 konst=0\n\
         2: TraceLiteral src=0\n\
         3: Store index=0 at=0 src=0\n"
    );
    // One register, released at the clause's own end, so a body of a hundred
    // assignments reserves one.
    assert_eq!(chunk.registers, 1);
    assert_eq!(chunk.consts, vec![Box::from(&b"abc"[..])]);
}

/// **Two assignments in one body reserve one register between them**, which is
/// the release at a promoted clause's own end doing its job.
///
/// Its own test rather than a line in the one above, because a body with a
/// single assignment in it reserves one register whether or not anything is
/// ever released -- so the single-assignment case cannot tell the two apart.
/// Found by mutation: dropping the `registers.release(mark)` from the
/// assignment arm left the whole workspace green, because every other stream
/// that asserts a register count reaches it through a promoted `SAY` instead.
/// Under that mutation the count here is 2, and it would grow with a body's
/// length -- exactly the withdrawn whole-chunk monotonic counter the plan's
/// Decisions section rejects.
///
/// The `SAY` is here for the same reason one instruction over, so that the
/// pair is stated rather than left to whichever other test happens to cover
/// it.
#[test]
fn two_assignments_and_two_says_in_one_body_reuse_one_register() {
    let chunk = compile_for_test(b"n1 = 'a'\nn2 = 'b'\n").expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=4\n\
         1: Const dst=0 konst=0\n\
         2: TraceLiteral src=0\n\
         3: Store index=0 at=0 src=0\n\
         4: Clause index=1 end=8\n\
         5: Const dst=0 konst=1\n\
         6: TraceLiteral src=0\n\
         7: Store index=1 at=1 src=0\n"
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
///
/// The interning is measured rather than tidy: an entry is a heap allocation,
/// so a table keyed by occurrence costs one allocation per literal written --
/// on a straight-line body one per clause, paid at compile time -- and the
/// mechanics spike measured that artifact swamping the difference it existed to
/// measure.
///
/// **Both halves are asserted, and each answers a degenerate table the other
/// does not.** The table's length answers a table that never looks a literal
/// up; the two `konst=0` fields answer one that dedupes the *entries* and hands
/// the second occurrence an index nothing filled. The third literal is what
/// stops both being satisfied by a table with one entry in it: `'x'` is a
/// different literal and takes entry 1.
///
/// **And the adjacent success, which is the half a dedupe would pass by
/// accident:** two literals whose *values* differ get two entries, so the
/// interning keys on bytes rather than on being a literal at all.
#[test]
fn one_literal_written_twice_is_one_interned_constant() {
    let chunk = compile_for_test(b"say 'dup'\nsay 'dup'\nsay 'x'\n").expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=4\n\
         1: Const dst=0 konst=0\n\
         2: TraceLiteral src=0\n\
         3: Say index=0 src=0\n\
         4: Clause index=1 end=8\n\
         5: Const dst=0 konst=0\n\
         6: TraceLiteral src=0\n\
         7: Say index=1 src=0\n\
         8: Clause index=2 end=12\n\
         9: Const dst=0 konst=1\n\
         10: TraceLiteral src=0\n\
         11: Say index=2 src=0\n"
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
///
/// **`at` is where the compile-time slot resolution shows, and the three kinds
/// answer differently.** A simple variable and a bare stem each resolve to
/// their own slot -- `1` here, because the plan assigns slots in source order
/// and the assignment's target `ZW` is written first. A compound resolves to
/// none: what its read goes through is the *stem's* slot and a tail key worked
/// out at the read site, so `ZA.ZI`'s own symbol has no slot to carry and the
/// run-time path is what finds them.
#[test]
fn a_bare_symbol_compiles_to_a_native_read_in_each_of_its_three_kinds() {
    let simple = compile_for_test(b"zw = zv\n").expect("compiles");
    assert_eq!(
        render(&simple),
        "0: Clause index=0 end=4\n\
         1: Load read=Simple at=1 dst=0\n\
         2: TraceRead read=Simple src=0\n\
         3: Store index=0 at=0 src=0\n"
    );
    assert_eq!(simple.registers, 1);
    assert!(
        simple.consts.is_empty(),
        "a read interns nothing: its value is in a frame slot, not in the chunk"
    );

    let stem = compile_for_test(b"zw = zs.\n").expect("compiles");
    assert_eq!(
        render(&stem),
        "0: Clause index=0 end=4\n\
         1: Load read=Stem at=1 dst=0\n\
         2: TraceRead read=Stem src=0\n\
         3: Store index=0 at=0 src=0\n"
    );

    let compound = compile_for_test(b"zw = za.zi\n").expect("compiles");
    assert_eq!(
        render(&compound),
        "0: Clause index=0 end=4\n\
         1: Load read=Compound at=- dst=0\n\
         2: TraceRead read=Compound src=0\n\
         3: Store index=0 at=0 src=0\n"
    );
}

/// **The symbol a compiled read names is the one its own expression names**,
/// which the rendered stream cannot say -- `render`'s own comment has why it
/// prints no symbol index.
///
/// Asserted against the parsed expression rather than against a number, so
/// nothing here moves when the pre-seeded symbol table does. The compound is
/// the kind that needs it most: its `at` is `-`, so a compiler that loaded the
/// assignment's *target* instead would render identically.
#[test]
fn a_compiled_read_names_the_symbol_its_expression_does() {
    let program = parse_program(b"zw = za.zi\n".to_vec()).expect("test program parses");
    let plan = Plan::build(&program.main, &program.symbols);
    let chunk =
        super::compile(&program.main, &plan, ChunkTrace::of(TraceMode::NORMAL)).expect("compiles");

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
        "0: Clause index=0 end=4\n\
         1: Load read=Simple at=0 dst=0\n\
         2: TraceRead read=Simple src=0\n\
         3: Say index=0 src=0\n"
    );
}

/// **An expression that merely *contains* a symbol is not a read**, and the
/// symbol's own load is not the expression's value.
///
/// The adjacent success the three cases above need: without it they are
/// satisfied by a compiler that emits a load for any expression holding a
/// symbol anywhere, which would evaluate `zv + 1` as `zv` and lose the
/// arithmetic. What such an expression compiles to now is the read **and** the
/// operator applied to it, which is the shape that has to be told from the bare
/// read -- a `Load` alone would produce a wrong answer that traces almost
/// right.
///
/// `.NIL` and `>zv` are the two expressions that look like a bare symbol read
/// and are not one: the first traces `>E>` and the second `>O>`, so a `Load`
/// for either would emit a `>V>` line the oracle does not print. Neither is a
/// term `native_shape` accepts, so both stay on [`super::Op::EvalExpr`] entire.
#[test]
fn an_expression_that_only_contains_a_symbol_is_more_than_that_symbols_read() {
    let chunk = compile_for_test(b"zw = zv + 1\n").expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=8\n\
         1: Load read=Simple at=1 dst=0\n\
         2: TraceRead read=Simple src=0\n\
         3: LoadConstant dst=1\n\
         4: TraceLiteral src=1\n\
         5: Arith op=+ hint=0 lhs=0 rhs=1 dst=0\n\
         6: TraceOperator op=+ src=0\n\
         7: Store index=0 at=0 src=0\n"
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
///
/// `za + zb + zc + zd` is left-nested, so the innermost `+` is reached first
/// and every later one takes the register below it as `lhs` and the register
/// above it as `rhs`. **The register economy is the assertion**: a compiler
/// that gave every operand a register of its own renders the same ops with
/// `lhs` and `dst` climbing, and would reserve one register per operator in a
/// chain rather than two in total.
#[test]
fn a_chain_of_operators_reuses_the_destination_register() {
    let chunk = compile_for_test(b"zw = za + zb + zc + zd\n").expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=16\n\
         1: Load read=Simple at=1 dst=0\n\
         2: TraceRead read=Simple src=0\n\
         3: Load read=Simple at=2 dst=1\n\
         4: TraceRead read=Simple src=1\n\
         5: Arith op=+ hint=0 lhs=0 rhs=1 dst=0\n\
         6: TraceOperator op=+ src=0\n\
         7: Load read=Simple at=3 dst=1\n\
         8: TraceRead read=Simple src=1\n\
         9: Arith op=+ hint=1 lhs=0 rhs=1 dst=0\n\
         10: TraceOperator op=+ src=0\n\
         11: Load read=Simple at=4 dst=1\n\
         12: TraceRead read=Simple src=1\n\
         13: Arith op=+ hint=2 lhs=0 rhs=1 dst=0\n\
         14: TraceOperator op=+ src=0\n\
         15: Store index=0 at=0 src=0\n"
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
///
/// **The adjacent case the chain above needs**: without it, a compiler that
/// ignored precedence and folded left every time would still pass that one, and
/// would compute `(za + zb) * zc` here. The `>O>` lines follow the ops, so this
/// is also what puts the inner operator's echo before the outer one's.
#[test]
fn precedence_decides_which_operator_is_the_inner_one() {
    let chunk = compile_for_test(b"zw = za + zb * zc\n").expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=12\n\
         1: Load read=Simple at=1 dst=0\n\
         2: TraceRead read=Simple src=0\n\
         3: Load read=Simple at=2 dst=1\n\
         4: TraceRead read=Simple src=1\n\
         5: Load read=Simple at=3 dst=2\n\
         6: TraceRead read=Simple src=2\n\
         7: Arith op=* hint=0 lhs=1 rhs=2 dst=1\n\
         8: TraceOperator op=* src=1\n\
         9: Arith op=+ hint=1 lhs=0 rhs=1 dst=0\n\
         10: TraceOperator op=+ src=0\n\
         11: Store index=0 at=0 src=0\n"
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
///
/// `eval::is_native_binary` is the one enumeration of the promotable operator
/// set, asked by `eval_node`'s own dispatch and by the compiler, so what this
/// pins is the other side of it: a term with no native op keeps its whole
/// expression general, and widening the operator set does not widen the term
/// set with it. A `.NIL` read through `Op::Load` would name a frame slot no
/// environment symbol has.
///
/// `ExprKind::Logical`, the comma list, is out of the native set too and is
/// not a row here: it appears only in a condition, and a condition is compiled
/// to `Op::EvalExpr` whatever its shape, so a row for it would pass under
/// every implementation of this function's subject.
///
/// **One row holds a call, and that is what it is for.** A call has an op of
/// its own and an address that reaches it there, so it is the one term where
/// "the whole slot or none of it" can be got wrong in the direction of
/// promoting too much: an implementation emitting a call's op inside a slot it
/// then covers with an `Op::EvalExpr` would run the call twice and print its
/// `>F>` line twice. Measured 2026-08-12, that mutation applied to
/// `push_value` passes the **whole workspace** with this row held out and
/// reddens here with it.
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
///
/// **The pair is the test.** Either half alone is satisfied by a compiler
/// that emits one op for every operator; together they pin the split, which
/// is that only arithmetic carries a quickening hint.
#[test]
fn every_binary_operator_but_arithmetic_compiles_to_one_op() {
    // One operator per family, over the same two operands, so the streams
    // differ in the operator alone.
    for (source, spelling) in [
        (&b"za = zb || zc\n"[..], "||"),
        (&b"za = zb = zc\n"[..], "="),
        (&b"za = zb & zc\n"[..], "&"),
    ] {
        let chunk = compile_for_test(source).expect("compiles");
        assert_eq!(
            render(&chunk),
            format!(
                "0: Clause index=0 end=8\n\
                 1: Load read=Simple at=1 dst=0\n\
                 2: TraceRead read=Simple src=0\n\
                 3: Load read=Simple at=2 dst=1\n\
                 4: TraceRead read=Simple src=1\n\
                 5: Binary op={spelling} lhs=0 rhs=1 dst=0\n\
                 6: TraceOperator op={spelling} src=0\n\
                 7: Store index=0 at=0 src=0\n"
            ),
            "{} did not compile to one Binary op",
            String::from_utf8_lossy(source)
        );
    }

    let chunk = compile_for_test(b"za = zb + zc\n").expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=8\n\
         1: Load read=Simple at=1 dst=0\n\
         2: TraceRead read=Simple src=0\n\
         3: Load read=Simple at=2 dst=1\n\
         4: TraceRead read=Simple src=1\n\
         5: Arith op=+ hint=0 lhs=0 rhs=1 dst=0\n\
         6: TraceOperator op=+ src=0\n\
         7: Store index=0 at=0 src=0\n"
    );
}

/// A prefix operator compiles to its own op and its own echo, and the echo
/// carries the operator: `\` and `-` trace different lines from one value.
///
/// **`+`, `-` and `\` are one test.** A single row is satisfied by a compiler
/// that writes one fixed operator into every echo, and the tag is the half of
/// a `>P>` line that can be wrong while its position and its register are
/// right -- which is what the rows differing in the operator alone separate.
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
                "0: Clause index=0 end=6\n\
                 1: Load read=Simple at=1 dst=0\n\
                 2: TraceRead read=Simple src=0\n\
                 3: Prefix op={spelling} src=0 dst=0\n\
                 4: TracePrefix op={spelling} src=0\n\
                 5: Store index=0 at=0 src=0\n"
            ),
            "{} did not compile to one Prefix op and its own echo",
            String::from_utf8_lossy(source)
        );
    }
}

/// A bare constant symbol is a native load of its own, the way a quoted literal
/// is -- and the two produce the same `>L>` echo, so one op serves both.
///
/// **The number in `zx + 1` is one of these and not an `ExprKind::Literal`**,
/// which is why arithmetic promotion needed it: with the constant left
/// unpromotable the whole expression falls to [`super::Op::EvalExpr`] and no
/// arithmetic op is emitted at all.
#[test]
fn a_constant_symbol_is_a_native_load() {
    let chunk = compile_for_test(b"zw = 1\n").expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=4\n\
         1: LoadConstant dst=0\n\
         2: TraceLiteral src=0\n\
         3: Store index=0 at=0 src=0\n"
    );

    // A quoted literal is the other load, against the chunk's own interned
    // table, and it takes the identical echo.
    let quoted = compile_for_test(b"zw = '1'\n").expect("compiles");
    assert_eq!(
        render(&quoted),
        "0: Clause index=0 end=4\n\
         1: Const dst=0 konst=0\n\
         2: TraceLiteral src=0\n\
         3: Store index=0 at=0 src=0\n"
    );
}

/// The compiled form of the plan's own example loop: the header is a clause
/// region of its own and the construct is the op that closes it.
///
/// The program is `DO`, `nop`, `END`, and the `DO`'s own region is laid out as
/// **one group per header expression, in the order the expressions were
/// written**:
///
/// * `1`-`3`: the control variable's starting value, computed and then
///   validated. No `TraceKeyword` in the group, because the oracle echoes no
///   `>K>` line for an initial value.
/// * `4`-`7`: the `TO` bound, computed, **echoed**, and then validated. The
///   echo sits between the two, which is the whole reason this construct waited
///   for the trace ops: `do i = 1 to 'a' by zf()` echoes `>K>  "TO" => "a"` and
///   raises before `zf` is called, so neither the echo nor the validation may
///   move past the next expression's evaluation.
/// * `8`: the construct itself.
///
/// **A header value is its expression's own native ops where `native_shape`
/// accepts the expression**, and one `Op::EvalExpr` where it does not. `1` and
/// `3` are constant symbols, so each is an `Op::LoadConstant` and the `>L>`
/// echo a load owes; `Op::LoopHeaderValue` is what the slot still owes either
/// way, which is why this promotion needed no op of its own.
///
/// The body clause and the `END` are still `Generic`. The `END` op is never
/// reached -- `run_bounded`'s range stops before it and the loop's own resume
/// is one past it -- and it is emitted anyway because `op_of` is indexed by
/// instruction, so an instruction without an op would shift every later entry.
#[test]
fn a_counted_loop_compiles_its_header_to_a_clause_region_and_its_body_to_generic() {
    let chunk = compile_for_test(b"do i = 1 to 3\n  nop\nend\n").expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=9\n\
         1: LoadConstant dst=0\n\
         2: TraceLiteral src=0\n\
         3: LoopHeaderValue role=Initial src=0\n\
         4: LoadConstant dst=1\n\
         5: TraceLiteral src=1\n\
         6: TraceKeyword role=To src=1\n\
         7: LoopHeaderValue role=To src=1\n\
         8: LoopRun index=0\n\
         9: Generic index=1\n\
         10: Generic index=2\n"
    );
    // One register per header expression, and they are **not** released at the
    // region's end: the loop runs from op 8 with the body's clauses stepped
    // between, so a register handed out again there would be overwritten while
    // the running loop still reads it.
    assert_eq!(chunk.registers, 2);
    assert_eq!(chunk.op_of, vec![0, 9, 10, 11]);
}

/// The same loop under `TRACE R`: the clause echo is an op of the region, and
/// the header's own groups are unchanged behind it.
///
/// The pair with the test above is what says the setting decides *what is
/// emitted* and nothing about the header's shape -- every group is the same
/// ops, one index further along.
#[test]
fn a_traced_counted_loop_echoes_its_do_clause_from_the_stream() {
    let chunk = compile_for_test_under(b"do i = 1 to 3\n  nop\nend\n", traced()).expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=10\n\
         1: TraceClause index=0\n\
         2: LoadConstant dst=0\n\
         3: TraceLiteral src=0\n\
         4: LoopHeaderValue role=Initial src=0\n\
         5: LoadConstant dst=1\n\
         6: TraceLiteral src=1\n\
         7: TraceKeyword role=To src=1\n\
         8: LoopHeaderValue role=To src=1\n\
         9: LoopRun index=0\n\
         10: Generic index=1\n\
         11: Generic index=2\n"
    );
    assert_eq!(chunk.registers, 2);
}

/// A block, and a `DO OVER`: the two ends of how much header a `DO`/`LOOP` can
/// have, and both still one clause region ending in the construct.
///
/// A `DO` block has **no header expression at all**, so its region is the
/// `LoopRun` op alone -- an empty region rather than none, because the clause
/// and its boundary are owed either way. A `DO OVER ... FOR` has an expression
/// for the target and one for the count, and emits a `TraceKeyword` for the
/// target alone, which is `HeaderRole::OverFor::keyword()` answering `None`.
///
/// **That answer diverges from the oracle**, and that doc comment carries the
/// measurement; the stream here is this crate's, not the oracle's, for that one
/// line.
#[test]
fn a_block_has_an_empty_header_region_and_a_do_over_echoes_only_its_target() {
    let block = compile_for_test(b"do\n  nop\nend\n").expect("compiles");
    assert_eq!(
        render(&block),
        "0: Clause index=0 end=2\n\
         1: LoopRun index=0\n\
         2: Generic index=1\n\
         3: Generic index=2\n"
    );
    assert_eq!(block.registers, 0, "a block evaluates nothing to hold");

    let over = compile_for_test(b"do qq over 4.5 for 2\n  nop\nend\n").expect("compiles");
    assert_eq!(
        render(&over),
        "0: Clause index=0 end=9\n\
         1: LoadConstant dst=0\n\
         2: TraceLiteral src=0\n\
         3: TraceKeyword role=Over src=0\n\
         4: LoopHeaderValue role=Over src=0\n\
         5: LoadConstant dst=1\n\
         6: TraceLiteral src=1\n\
         7: LoopHeaderValue role=OverFor src=1\n\
         8: LoopRun index=0\n\
         9: Generic index=1\n\
         10: Generic index=2\n"
    );
    assert_eq!(over.registers, 2);
}

/// A header bound that is a bare symbol, and one that is a call.
///
/// The symbol takes `Op::Load` and the `>V>` echo behind it, the call takes
/// `Op::CallExpr` and the `>F>` echo behind it -- the same ops an assignment's
/// value takes for the same expressions, because it is the same `push_native`
/// descent. What is a header's own is the address: the call op names slot `1`
/// of the `DO` and `root` below it, which is the address
/// `Interp::chunk_node_at` resolves back through `loop_header_slot`. Without
/// that resolution the op reaches `Loud::call_op_off_its_node` at run time
/// rather than at compile time.
#[test]
fn a_header_bound_that_is_a_symbol_and_one_that_is_a_call_take_their_own_ops() {
    let symbol = compile_for_test(b"do i = 1 to zn\n  nop\nend\n").expect("compiles");
    assert_eq!(
        render(&symbol),
        "0: Clause index=0 end=9\n\
         1: LoadConstant dst=0\n\
         2: TraceLiteral src=0\n\
         3: LoopHeaderValue role=Initial src=0\n\
         4: Load read=Simple at=1 dst=1\n\
         5: TraceRead read=Simple src=1\n\
         6: TraceKeyword role=To src=1\n\
         7: LoopHeaderValue role=To src=1\n\
         8: LoopRun index=0\n\
         9: Generic index=1\n\
         10: Generic index=2\n"
    );

    let call = compile_for_test(b"do i = 1 to length(zs)\n  nop\nend\n").expect("compiles");
    assert_eq!(
        render(&call),
        "0: Clause index=0 end=9\n\
         1: LoadConstant dst=0\n\
         2: TraceLiteral src=0\n\
         3: LoopHeaderValue role=Initial src=0\n\
         4: CallExpr index=0 slot=1 path=root site=0 dst=1\n\
         5: TraceFunction index=0 slot=1 path=root src=1\n\
         6: TraceKeyword role=To src=1\n\
         7: LoopHeaderValue role=To src=1\n\
         8: LoopRun index=0\n\
         9: Generic index=1\n\
         10: Generic index=2\n"
    );
}

/// **One header expression outside the native set leaves the others native**,
/// because each slot is its own decision.
///
/// `.nil` is an `ExprKind::DotVariable`, which `native_shape` declines, so slot
/// `0` is one `Op::EvalExpr` doing that slot's whole evaluation -- and slot `1`
/// beside it is still the constant symbol's own load. `Op::LoopHeaderValue`
/// follows either, which is what lets the two forms sit in one header at all.
#[test]
fn a_header_slot_outside_the_native_set_leaves_the_other_slots_native() {
    let chunk = compile_for_test(b"do i = .nil to 3\n  nop\nend\n").expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=8\n\
         1: EvalExpr index=0 slot=0 dst=0\n\
         2: LoopHeaderValue role=Initial src=0\n\
         3: LoadConstant dst=1\n\
         4: TraceLiteral src=1\n\
         5: TraceKeyword role=To src=1\n\
         6: LoopHeaderValue role=To src=1\n\
         7: LoopRun index=0\n\
         8: Generic index=1\n\
         9: Generic index=2\n"
    );
}

/// **A register `push_native` took for an operand inside a header goes back
/// before the body's clauses are emitted**, and the header's own registers do
/// not.
///
/// The two are allocated from one stack and released at opposite ends of the
/// loop: `zn + 1` needs a second register for its right operand, which
/// `push_native` hands back as soon as the `Op::Arith` has read it, while the
/// registers the header values land in are held past the `END`. Register 2 is
/// where that meets: the operand takes it, gives it back, and the body's
/// assignment is handed the same one.
///
/// **The stream pins both ways of getting that wrong, and they are opposite
/// failures.** Measured, each as its own mutation over the whole workspace:
///
/// * The header's own registers released at the region's end instead of past
///   the `END` puts the assignment at register 0 -- a value `LoopState` reads
///   for the rest of the construct, written over by a body clause, because
///   `Op::LoopRun` steps the body from inside this same region. The `ir_dual`
///   suite stays green, population sweep included; this test and
///   `a_nested_loops_registers_sit_above_the_enclosing_loops_and_a_later_loops_reuse_them`
///   move, and nothing else in the workspace.
/// * `push_native` never releasing the operand's register puts the assignment
///   at register 3, which wastes a register and overwrites nothing. **That
///   reading needs `compile`'s own `debug_assert_eq!` on the header's register
///   top removed as well**, because with it in place the assertion fires before
///   this test reaches `render` -- and the mutation is then loud rather than
///   quiet, reaching `both_engines_agree_across_every_population`, which fails
///   on the assertion's message. With both removed, this test and
///   `a_chain_of_operators_reuses_the_destination_register` move and nothing
///   else does.
///
/// **So the harmful direction is not visible to any differential harness in the
/// tree**, and the reason is that a header value's `ObjRef` is rooted by its
/// register only on the compiled engine, and nothing in the corpus collects
/// while a loop is running.
#[test]
fn a_header_operands_register_goes_back_to_the_body() {
    let chunk = compile_for_test(b"do i = 1 to zn + 1\n  zx = 5\nend\n").expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=13\n\
         1: LoadConstant dst=0\n\
         2: TraceLiteral src=0\n\
         3: LoopHeaderValue role=Initial src=0\n\
         4: Load read=Simple at=1 dst=1\n\
         5: TraceRead read=Simple src=1\n\
         6: LoadConstant dst=2\n\
         7: TraceLiteral src=2\n\
         8: Arith op=+ hint=0 lhs=1 rhs=2 dst=1\n\
         9: TraceOperator op=+ src=1\n\
         10: TraceKeyword role=To src=1\n\
         11: LoopHeaderValue role=To src=1\n\
         12: LoopRun index=0\n\
         13: Clause index=1 end=17\n\
         14: LoadConstant dst=2\n\
         15: TraceLiteral src=2\n\
         16: Store index=1 at=2 src=2\n\
         17: Generic index=2\n"
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
///
/// The inner loop takes registers 2 and 3 rather than 0 and 1, because the
/// outer loop is still running -- its `LoopState` reads the values registers 0
/// and 1 root for as long as the body it encloses is being stepped. An
/// allocator that released the outer loop's registers at its own region's end
/// would hand 0 and 1 to the inner loop and overwrite a running loop's bound.
///
/// **And the adjacent success, which is what stops that being satisfied by
/// never releasing at all:** the second of two loops written one after the
/// other does reuse 0 and 1, because by the instruction after the first loop's
/// `END` the first loop is over.
#[test]
fn a_nested_loops_registers_sit_above_the_enclosing_loops_and_a_later_loops_reuse_them() {
    let nested = compile_for_test(b"do i = 1 to 2\n  do j = 1 to 2\n    nop\n  end\nend\n")
        .expect("compiles");
    assert_eq!(
        render(&nested),
        "0: Clause index=0 end=9\n\
         1: LoadConstant dst=0\n\
         2: TraceLiteral src=0\n\
         3: LoopHeaderValue role=Initial src=0\n\
         4: LoadConstant dst=1\n\
         5: TraceLiteral src=1\n\
         6: TraceKeyword role=To src=1\n\
         7: LoopHeaderValue role=To src=1\n\
         8: LoopRun index=0\n\
         9: Clause index=1 end=18\n\
         10: LoadConstant dst=2\n\
         11: TraceLiteral src=2\n\
         12: LoopHeaderValue role=Initial src=2\n\
         13: LoadConstant dst=3\n\
         14: TraceLiteral src=3\n\
         15: TraceKeyword role=To src=3\n\
         16: LoopHeaderValue role=To src=3\n\
         17: LoopRun index=1\n\
         18: Generic index=2\n\
         19: Generic index=3\n\
         20: Generic index=4\n"
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
            "17: TraceKeyword role=To src=1\n18: LoopHeaderValue role=To src=1\n19: LoopRun index=3"
        ),
        "the second loop\'s own bound went somewhere other than register 1: {}",
        render(&sequential)
    );
}

/// The compiled `IF` with an `ELSE`, which is the shape the whole promotion
/// is about: both paths are jumps in one stream where the tree-walker splits
/// them across two engines.
///
/// The six instructions are `IF`, `THEN`, `say 'a'`, `ELSE`, `say 'b'`,
/// `say 'c'`. The `IF`'s own clause is the region that computes the condition
/// and branches on it: `1 = 1` is native, so it is the two constant loads,
/// the comparison and their three echo lines, then `Condition` -- the
/// validation and the `>>>` that the condition still owes -- and the jump that
/// reads the register `Condition` left its answer in. Two more ops close the
/// true branch, sitting between its last op and the `ELSE`'s own op so that
/// neither instruction's entry in `op_of` moves.
///
/// The three things a reader should check by eye are the two jump targets and
/// what sits between them: `JumpUnless` goes to op 16, the `ELSE` marker, and
/// `Jump` goes to op 21, `say 'c'`'s own clause, past the whole `ELSE` branch,
/// with `EndBranch` at op 14 -- the boundary a promoted construct owes once the
/// branch it chose has finished.
///
/// **The `JumpUnless` target is past that `EndBranch`, and that is measured**:
/// the oracle closes a *taken* branch with a synthetic instruction and jumps
/// over it on the false path, where an `IF` whose condition is false runs no
/// such boundary at all.
///
/// **`op_of[3]` is 14 and not 16**, which is the other half of the same
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
        "0: Clause index=0 end=9\n\
         1: LoadConstant dst=0\n\
         2: TraceLiteral src=0\n\
         3: LoadConstant dst=1\n\
         4: TraceLiteral src=1\n\
         5: Binary op== lhs=0 rhs=1 dst=0\n\
         6: TraceOperator op== src=0\n\
         7: Condition index=0 reg=0 keyword=IF\n\
         8: JumpUnless reg=0 target=16\n\
         9: Generic index=1\n\
         10: Clause index=2 end=14\n\
         11: Const dst=0 konst=0\n\
         12: TraceLiteral src=0\n\
         13: Say index=2 src=0\n\
         14: EndBranch\n\
         15: Jump target=21\n\
         16: Generic index=3\n\
         17: Clause index=4 end=21\n\
         18: Const dst=0 konst=1\n\
         19: TraceLiteral src=0\n\
         20: Say index=4 src=0\n\
         21: Clause index=5 end=25\n\
         22: Const dst=0 konst=2\n\
         23: TraceLiteral src=0\n\
         24: Say index=5 src=0\n"
    );
    // Two registers throughout: the comparison's right operand takes one of
    // its own beside the register the condition lands in, and gives it back
    // inside `push_native`'s own binary arm as soon as the operation has run.
    // The condition's own register is released at the `IF`'s clause end, so
    // each promoted `SAY` below gets register 0 back.
    assert_eq!(chunk.registers, 2);
    assert_eq!(chunk.op_of, vec![0, 9, 10, 14, 17, 21, 25]);
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
///
/// The end-of-branch boundary is still emitted, and is still the true path's
/// alone: the branch falls into `EndBranch` at op 14 and the `JumpUnless` goes
/// past it to op 15.
#[test]
fn an_if_with_no_else_emits_no_branch_end_jump() {
    let chunk = compile_for_test(b"if 1 = 0 then say 'a'\nsay 'b'\n").expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=9\n\
         1: LoadConstant dst=0\n\
         2: TraceLiteral src=0\n\
         3: LoadConstant dst=1\n\
         4: TraceLiteral src=1\n\
         5: Binary op== lhs=0 rhs=1 dst=0\n\
         6: TraceOperator op== src=0\n\
         7: Condition index=0 reg=0 keyword=IF\n\
         8: JumpUnless reg=0 target=15\n\
         9: Generic index=1\n\
         10: Clause index=2 end=14\n\
         11: Const dst=0 konst=0\n\
         12: TraceLiteral src=0\n\
         13: Say index=2 src=0\n\
         14: EndBranch\n\
         15: Clause index=3 end=19\n\
         16: Const dst=0 konst=1\n\
         17: TraceLiteral src=0\n\
         18: Say index=3 src=0\n"
    );
    assert_eq!(chunk.registers, 2);
}

/// **A condition outside the native set stays one [`super::Op::EvalExpr`] and
/// takes no `Condition` op**, which is the adjacent refusal to the two streams
/// above.
///
/// The fallback is not the same op with a piece missing: that `EvalExpr` runs
/// `Interp::eval_if_condition`, which evaluates the expression *and* validates
/// it *and* emits its `>>>`, so a `Condition` behind it would trace the value
/// a second time and validate it a second time. `.nil` is
/// `ExprKind::DotVariable`, which `native_shape` has no arm for.
#[test]
fn a_condition_outside_the_native_set_stays_one_eval_expr() {
    let chunk = compile_for_test(b"if .nil then nop\n").expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=3\n\
         1: EvalExpr index=0 slot=0 dst=0\n\
         2: JumpUnless reg=0 target=6\n\
         3: Generic index=1\n\
         4: Generic index=2\n\
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
///
/// Two `IF`s in one body reuse the same registers, which is what the
/// allocator's stack discipline buys over the spike's withdrawn monotonic
/// counter: under that counter the inner `IF` would take registers 2 and 3 and
/// the high-water mark would grow with a body's length rather than with its
/// depth.
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
fn nested_ifs_reuse_their_registers() {
    let chunk =
        compile_for_test(b"if 1 = 1 then\n  if 2 = 2 then say 'a'\nsay 'b'\n").expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=9\n\
         1: LoadConstant dst=0\n\
         2: TraceLiteral src=0\n\
         3: LoadConstant dst=1\n\
         4: TraceLiteral src=1\n\
         5: Binary op== lhs=0 rhs=1 dst=0\n\
         6: TraceOperator op== src=0\n\
         7: Condition index=0 reg=0 keyword=IF\n\
         8: JumpUnless reg=0 target=26\n\
         9: Generic index=1\n\
         10: Clause index=2 end=19\n\
         11: LoadConstant dst=0\n\
         12: TraceLiteral src=0\n\
         13: LoadConstant dst=1\n\
         14: TraceLiteral src=1\n\
         15: Binary op== lhs=0 rhs=1 dst=0\n\
         16: TraceOperator op== src=0\n\
         17: Condition index=2 reg=0 keyword=IF\n\
         18: JumpUnless reg=0 target=26\n\
         19: Generic index=3\n\
         20: Clause index=4 end=24\n\
         21: Const dst=0 konst=0\n\
         22: TraceLiteral src=0\n\
         23: Say index=4 src=0\n\
         24: EndBranch\n\
         25: EndBranch\n\
         26: Clause index=5 end=30\n\
         27: Const dst=0 konst=1\n\
         28: TraceLiteral src=0\n\
         29: Say index=5 src=0\n"
    );
    assert_eq!(
        chunk.registers, 2,
        "the inner IF reuses the registers the outer one released"
    );
}

/// The compiled `SELECT` with an `OTHERWISE`: one clause region for the header
/// and one per listed `WHEN`, chained by the `JumpUnless` each `WHEN` ends
/// with, and a frame opened over whichever branch wins.
///
/// The instructions are `SELECT`, `WHEN`, `THEN`, `say 'a'`, `WHEN`, `THEN`,
/// `say 'b'`, `OTHERWISE`, `say 'o'`, `END`, `say 'after'`.
///
/// **Each listed `WHEN`'s region is its condition compiled, exactly as an
/// `IF`'s is**: `1 = 0` is native, so the region is the two constant loads,
/// the comparison, their echo lines, then `Condition` -- the validation and
/// the `>>>` the condition still owes -- and the jump that reads the register
/// `Condition` left its answer in. `keyword=WHEN` is what selects 34.2 over an
/// `IF`'s 34.1 for a value that is not exactly `0`/`1`.
///
/// The three things a reader should check by eye are the jump targets:
///
/// * the first `WHEN`'s `JumpUnless` goes to op 17, the **second `WHEN`'s own
///   clause region**, which is the scan continuing;
/// * the second `WHEN`'s goes to op 32, the `EnterOtherwise` in front of the
///   `OTHERWISE` marker, which is the scan running out;
/// * and nothing jumps past a branch, because a branch is left by the frame
///   `EnterWhen` opened rather than by an op -- reaching op 32 by falling out
///   of the second `WHEN`'s branch is that branch's `op_end`, and the driver
///   closes the frame there instead of running the op.
///
/// **`op_of[7]` is 32 and not 33**, which is the other half of the same
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
         2: Clause index=1 end=11\n\
         3: LoadConstant dst=0\n\
         4: TraceLiteral src=0\n\
         5: LoadConstant dst=1\n\
         6: TraceLiteral src=1\n\
         7: Binary op== lhs=0 rhs=1 dst=0\n\
         8: TraceOperator op== src=0\n\
         9: Condition index=1 reg=0 keyword=WHEN\n\
         10: JumpUnless reg=0 target=17\n\
         11: EnterWhen select=0 when=1\n\
         12: Generic index=2\n\
         13: Clause index=3 end=17\n\
         14: Const dst=0 konst=0\n\
         15: TraceLiteral src=0\n\
         16: Say index=3 src=0\n\
         17: Clause index=4 end=26\n\
         18: LoadConstant dst=0\n\
         19: TraceLiteral src=0\n\
         20: LoadConstant dst=1\n\
         21: TraceLiteral src=1\n\
         22: Binary op== lhs=0 rhs=1 dst=0\n\
         23: TraceOperator op== src=0\n\
         24: Condition index=4 reg=0 keyword=WHEN\n\
         25: JumpUnless reg=0 target=32\n\
         26: EnterWhen select=0 when=4\n\
         27: Generic index=5\n\
         28: Clause index=6 end=32\n\
         29: Const dst=0 konst=1\n\
         30: TraceLiteral src=0\n\
         31: Say index=6 src=0\n\
         32: EnterOtherwise select=0\n\
         33: Generic index=7\n\
         34: Clause index=8 end=38\n\
         35: Const dst=0 konst=2\n\
         36: TraceLiteral src=0\n\
         37: Say index=8 src=0\n\
         38: Generic index=9\n\
         39: Clause index=10 end=43\n\
         40: Const dst=0 konst=3\n\
         41: TraceLiteral src=0\n\
         42: Say index=10 src=0\n"
    );
    assert_eq!(
        chunk.registers, 2,
        "the second WHEN reuses the registers the first one released, and a \
         comparison's right operand takes one beside the answer's own"
    );
    assert_eq!(
        chunk.op_of,
        vec![0, 2, 12, 13, 17, 27, 28, 32, 34, 38, 39, 43]
    );
}

/// A `SELECT CASE`'s own value is allocated in the **enclosing** scope, so the
/// register a `WHEN` takes for its own answer cannot reclaim it.
///
/// **This is the first emitted stream where `Mark` carrying a position is
/// observable**, which `nested_ifs_reuse_their_registers` says it is not for an
/// `IF`. The `CASE` value is register 0 and outlives every member clause --
/// the second `WHEN` is tested after the first `WHEN`'s branch has already run
/// -- while the two `WHEN`s share register 1 between them. An allocator whose
/// `mark()` always answered `Mark(0)` would hand register 0 back to the first
/// `WHEN` and compare every later `WHEN` against whatever that left behind.
///
/// **And the adjacent success, in the same stream:** the promoted `SAY` after
/// the whole `SELECT` is back on register 0, because the release at the
/// construct's `END` reaches the enclosing mark. An allocator that never
/// released it would leave that `SAY` on register 1 instead.
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
         7: Generic index=2\n\
         8: Clause index=3 end=12\n\
         9: Const dst=1 konst=0\n\
         10: TraceLiteral src=1\n\
         11: Say index=3 src=1\n\
         12: Clause index=4 end=15\n\
         13: WhenTest index=4 case=0 dst=1\n\
         14: JumpUnless reg=1 target=21\n\
         15: EnterWhen select=0 when=4\n\
         16: Generic index=5\n\
         17: Clause index=6 end=21\n\
         18: Const dst=1 konst=1\n\
         19: TraceLiteral src=1\n\
         20: Say index=6 src=1\n\
         21: Generic index=7\n\
         22: Clause index=8 end=26\n\
         23: Const dst=0 konst=2\n\
         24: TraceLiteral src=0\n\
         25: Say index=8 src=0\n"
    );
    assert_eq!(
        chunk.registers, 2,
        "the CASE value and one WHEN answer are live at once, and never more"
    );
}

/// Without an `OTHERWISE` the scan runs out onto the `END`, whose own 7.3 is
/// what "every WHEN was false" means -- so the last `WHEN`'s `JumpUnless`
/// names the `END`'s **own** op (17, the `Generic`) and no frame is open when
/// it runs.
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
         2: Clause index=1 end=11\n\
         3: LoadConstant dst=0\n\
         4: TraceLiteral src=0\n\
         5: LoadConstant dst=1\n\
         6: TraceLiteral src=1\n\
         7: Binary op== lhs=0 rhs=1 dst=0\n\
         8: TraceOperator op== src=0\n\
         9: Condition index=1 reg=0 keyword=WHEN\n\
         10: JumpUnless reg=0 target=17\n\
         11: EnterWhen select=0 when=1\n\
         12: Generic index=2\n\
         13: Clause index=3 end=17\n\
         14: Const dst=0 konst=0\n\
         15: TraceLiteral src=0\n\
         16: Say index=3 src=0\n\
         17: Generic index=4\n\
         18: Clause index=5 end=22\n\
         19: Const dst=0 konst=1\n\
         20: TraceLiteral src=0\n\
         21: Say index=5 src=0\n"
    );
}

/// **A `WHEN`'s condition outside the native set stays one
/// [`super::Op::WhenTest`] and takes no `Condition` op**, which is the
/// adjacent refusal to the two `SELECT` streams above and the sibling of
/// `a_condition_outside_the_native_set_stays_one_eval_expr`.
///
/// The fallback is not the same op with a piece missing: that `WhenTest` runs
/// `Interp::scan_when`, which evaluates the condition *and* validates it *and*
/// emits its `>>>`, so a `Condition` behind it would trace the value a second
/// time and validate it a second time. `.nil` is `ExprKind::DotVariable`,
/// which `native_shape` has no arm for.
#[test]
fn a_when_condition_outside_the_native_set_stays_one_when_test() {
    let chunk = compile_for_test(b"select\n  when .nil then nop\nend\n").expect("compiles");
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
         8: Generic index=4\n"
    );
}

/// A call inside a `WHEN`'s condition takes an [`super::Op::CallExpr`]
/// addressed at slot `0` and the route down to it, which is what
/// `Interp::chunk_node_at`'s own `When` arm resolves. `root.L` is that route:
/// the `>` is the condition's own root and the call is its left operand.
///
/// The address is the whole of what makes the op runnable: without that arm
/// the descent answers `None` and the driver reaches
/// `Loud::call_op_off_its_node` instead of calling anything.
#[test]
fn a_call_in_a_whens_condition_is_addressed_at_the_conditions_slot() {
    let chunk = compile_for_test(b"zs = 'abcd'\nselect\n  when length(zs) > 3 then nop\nend\n")
        .expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=4\n\
         1: Const dst=0 konst=0\n\
         2: TraceLiteral src=0\n\
         3: Store index=0 at=0 src=0\n\
         4: Clause index=1 end=5\n\
         5: SelectCaseText index=1 case=-\n\
         6: Clause index=2 end=15\n\
         7: CallExpr index=2 slot=0 path=root.L site=0 dst=0\n\
         8: TraceFunction index=2 slot=0 path=root.L src=0\n\
         9: LoadConstant dst=1\n\
         10: TraceLiteral src=1\n\
         11: Binary op=> lhs=0 rhs=1 dst=0\n\
         12: TraceOperator op=> src=0\n\
         13: Condition index=2 reg=0 keyword=WHEN\n\
         14: JumpUnless reg=0 target=18\n\
         15: EnterWhen select=1 when=2\n\
         16: Generic index=3\n\
         17: Generic index=4\n\
         18: Generic index=5\n"
    );
}

/// An **absorbed** `WHEN` -- one that is itself another `WHEN`'s consequence
/// rather than a branch its `SELECT` collected -- compiles to
/// [`super::Op::Generic`] and takes no region, so nothing this promotion emits
/// reaches it.
///
/// The instructions are `SELECT`, `WHEN 1 = 1`, `THEN`, `WHEN 2 = 2`, `THEN`,
/// `nop`, `END`, so instruction 3 is the absorbed one and op 13 is its whole
/// compiled form. There is no `Clause index=3` anywhere in the stream.
///
/// The pair with the streams above is the point: a listed `WHEN` and an
/// absorbed one are the same instruction kind, and what tells them apart is
/// whether some `SELECT` named the index.
#[test]
fn an_absorbed_when_compiles_to_generic() {
    let chunk = compile_for_test(b"select\n  when 1 = 1 then when 2 = 2 then nop\nend\n")
        .expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=1\n\
         1: SelectCaseText index=0 case=-\n\
         2: Clause index=1 end=11\n\
         3: LoadConstant dst=0\n\
         4: TraceLiteral src=0\n\
         5: LoadConstant dst=1\n\
         6: TraceLiteral src=1\n\
         7: Binary op== lhs=0 rhs=1 dst=0\n\
         8: TraceOperator op== src=0\n\
         9: Condition index=1 reg=0 keyword=WHEN\n\
         10: JumpUnless reg=0 target=16\n\
         11: EnterWhen select=0 when=1\n\
         12: Generic index=2\n\
         13: Generic index=3\n\
         14: Generic index=4\n\
         15: Generic index=5\n\
         16: Generic index=6\n"
    );
}

/// The same `IF` compiled under `TRACE R`: the clause echo is an **op** of the
/// clause's own region, and the region is one op longer for it.
///
/// The neighbour of `an_if_with_an_else_compiles_to_a_clause_region_and_two_
/// jumps`, and the pair is the whole of D23's emission decision: one body, two
/// settings, two streams. Everything but the echo ops and the indices they
/// shift is identical, which is what says the setting decides *what is emitted*
/// rather than what any op does.
///
/// **The echo is each region's first op, not its last.** The tree-walker echoes
/// a clause before it computes anything, so every line the condition produces
/// -- its operands' `>L>`, its operator's `>O>`, and the `>>>` of the value it
/// settles on -- follows the `*-*` line; an echo emitted after the condition's
/// ops would reverse them and no register or jump would move.
///
/// **The whole traced order of a promoted `SAY` is readable off ops 11 to 15**:
/// the clause echo, the constant load, the literal's own `>L>` line, and the
/// print with its `>>>`. Three of those four lines come from three different
/// ops, and the oracle prints them in exactly that order
/// (`tests/ir_dual_cases/assignment-and-say`, "SAY of a literal under trace
/// i").
#[test]
fn a_traced_if_carries_its_clause_echo_as_an_op_of_the_region() {
    let chunk = compile_for_test_under(b"if 1 = 1 then say 'a'\nelse say 'b'\nsay 'c'\n", traced())
        .expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=10\n\
         1: TraceClause index=0\n\
         2: LoadConstant dst=0\n\
         3: TraceLiteral src=0\n\
         4: LoadConstant dst=1\n\
         5: TraceLiteral src=1\n\
         6: Binary op== lhs=0 rhs=1 dst=0\n\
         7: TraceOperator op== src=0\n\
         8: Condition index=0 reg=0 keyword=IF\n\
         9: JumpUnless reg=0 target=18\n\
         10: Generic index=1\n\
         11: Clause index=2 end=16\n\
         12: TraceClause index=2\n\
         13: Const dst=0 konst=0\n\
         14: TraceLiteral src=0\n\
         15: Say index=2 src=0\n\
         16: EndBranch\n\
         17: Jump target=24\n\
         18: Generic index=3\n\
         19: Clause index=4 end=24\n\
         20: TraceClause index=4\n\
         21: Const dst=0 konst=1\n\
         22: TraceLiteral src=0\n\
         23: Say index=4 src=0\n\
         24: Clause index=5 end=29\n\
         25: TraceClause index=5\n\
         26: Const dst=0 konst=2\n\
         27: TraceLiteral src=0\n\
         28: Say index=5 src=0\n"
    );
    // The echo op addresses no register, so the extra op changes nothing the
    // driver has to reserve.
    assert_eq!(chunk.registers, 2);
    assert_eq!(chunk.op_of, vec![0, 10, 11, 16, 19, 24, 29]);
}

/// A traced `SELECT CASE`: **one echo per promoted clause, and exactly one**.
///
/// Three things this pins that the `IF` pair does not:
///
/// * the header's echo sits **before** its `EvalExpr`, so the `>K>  "CASE"`
///   line the expression produces follows the `*-*` line rather than preceding
///   it;
/// * `SelectCaseText` stays **outside** the region, one op further along than
///   it was untraced -- it is not part of the clause and the echo must not
///   have pulled it in;
/// * a `THEN` marker and the `END` get no echo op at all. They are `Generic`,
///   so their echo comes from the tree-walker's own clause unit and a second
///   one here would print every such clause twice. Each branch body is its own
///   promoted clause and so does carry one, which is the pair that says the op
///   follows the region rather than the construct.
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
         7: JumpUnless reg=1 target=15\n\
         8: EnterWhen select=0 when=1\n\
         9: Generic index=2\n\
         10: Clause index=3 end=15\n\
         11: TraceClause index=3\n\
         12: Const dst=1 konst=0\n\
         13: TraceLiteral src=1\n\
         14: Say index=3 src=1\n\
         15: Clause index=4 end=19\n\
         16: TraceClause index=4\n\
         17: WhenTest index=4 case=0 dst=1\n\
         18: JumpUnless reg=1 target=26\n\
         19: EnterWhen select=0 when=4\n\
         20: Generic index=5\n\
         21: Clause index=6 end=26\n\
         22: TraceClause index=6\n\
         23: Const dst=1 konst=1\n\
         24: TraceLiteral src=1\n\
         25: Say index=6 src=1\n\
         26: Generic index=7\n\
         27: Clause index=8 end=32\n\
         28: TraceClause index=8\n\
         29: Const dst=0 konst=2\n\
         30: TraceLiteral src=0\n\
         31: Say index=8 src=0\n"
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
///
/// The `SELECT CASE`'s own value is register 0 and the inner loop's two are 1 and
/// 2, so the release at the `SELECT`'s `END` has to reach 0 and the loop after
/// the whole thing gets 0 and 1 back. Releasing to the inner loop's own mark
/// instead leaves register 0 allocated for the rest of the body and hands that
/// loop 1 and 2.
///
/// **The reserved count does not tell the two apart and the assertion on it is
/// context rather than the witness.** Three registers are live at the deepest
/// point either way, and a release only decides which indices come *next*; the
/// first version of this test asserted the count alone and passed under both.
#[test]
fn two_constructs_ending_at_one_instruction_release_to_the_lower_mark() {
    let chunk = compile_for_test(
        b"select case 1\n  when 1 then do i = 1 to 2\n    nop\n  end\nend\ndo j = 1 to 2\n            nop\nend\n",
    )
    .expect("compiles");
    let stream = render(&chunk);
    assert!(
        stream.contains(
            "21: LoadConstant dst=0\n22: TraceLiteral src=0\n23: LoopHeaderValue role=Initial src=0"
        ),
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
/// **Each assertion below answers a degenerate cache the others do not**,
/// which is why they are written out rather than folded together:
///
/// * `compile` ran twice, so the second setting was compiled for rather than
///   answered from the first setting's entry -- this is what a key that
///   ignores the setting fails;
/// * the two chunks are distinct `Rc`s, so it is not one chunk handed back
///   under two names;
/// * their rendered streams differ, so the setting decided something rather
///   than producing the same ops twice;
/// * and asking again under the *first* setting gives the *first* chunk back,
///   which is what says the second lookup added an entry rather than replacing
///   one. A cache that evicted on a setting change passes the rest and fails
///   this, and a program that toggles `TRACE` in a loop is what that costs.
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

/// A `CALL` compiles to a clause region holding one call op, and **no
/// register**.
///
/// The two ops are the whole of what the tree-walker's own `Call` arm is,
/// split at the one seam a stream can use: the clause boundary, which
/// [`super::Op::Clause`] owns, and the call itself, which
/// [`super::Op::Call`] runs through the same `Interp::resolve_call` and
/// `Interp::invoke_named_call` that `step`'s own arm reaches.
///
/// **The register count is the assertion that says where this promotion
/// stops.** A call's arguments are expressions, and every other promoted
/// instruction evaluates its expression into a register -- this one does not,
/// because an argument is not an `ObjRef`: a `>name` reference carries the
/// caller's slot with it, which no register holds. So the arguments stay
/// `invoke_call`'s, and with them every `>A>` line and every intermediate the
/// argument expressions emit.
#[test]
fn a_call_compiles_to_a_clause_region_and_one_call_op() {
    let chunk = compile_for_test(b"call zsub 1\n").expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=2\n\
         1: Call index=0 site=0\n"
    );
    assert_eq!(chunk.registers, 0);
    // And nothing is interned: the argument's literal is evaluated from its
    // own node by `invoke_call`, not loaded from this chunk's table.
    assert!(chunk.consts.is_empty());
}

/// **Each call site takes a site index of its own, dense over the call ops.**
///
/// The same shape [`super::Op::Arith`]'s `hint` uses and for the same reason:
/// the table a site's resolution is remembered in is indexed off the op rather
/// than off the op's position, so a stream carries no entry for the ops that
/// never resolve anything and the driver's loop needs no counter beside it.
///
/// Two calls of the **same** name, which is what makes this about the site
/// rather than about the name: a table keyed by name would hand both the same
/// entry, and the numbering below is what says it does not.
#[test]
fn each_call_site_takes_a_resolution_site_of_its_own() {
    let chunk = compile_for_test(b"call zsub 1\ncall zsub 2\n").expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=2\n\
         1: Call index=0 site=0\n\
         2: Clause index=1 end=4\n\
         3: Call index=1 site=1\n"
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
        "0: Clause index=0 end=3\n\
         1: TraceClause index=0\n\
         2: Call index=0 site=0\n"
    );
}

/// **`CALL ON`/`CALL OFF` and `CALL (expr)` are not promoted**, and each is
/// unpromoted for a reason of its own rather than by oversight.
///
/// `CALL ON` resolves no name at all: it edits the activation's trap table,
/// which is `exec_condition_trap`'s and has nothing a call site could
/// remember. `CALL (expr)` learns its name at run time, so a site cannot hold
/// a resolution for it without a guard comparing the name it was resolved
/// for -- and a guarded cache is exactly the shape this task has no evidence
/// about (D24, amended).
#[test]
fn a_trap_call_and_a_dynamic_call_stay_generic() {
    let chunk = compile_for_test(b"call on error name zh\ncall (zn)\n").expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Generic index=0\n\
         1: Generic index=1\n"
    );
    assert_eq!(chunk.registers, 0);
}

/// **A compiled write carries the plan's slot for a simple target and carries
/// none for the other two**, which is the whole of what `write_slot` decides.
///
/// The pairing is what pins it: a simple target reads `at=0`, and the stem and
/// compound targets beside it read `at=-` with the identical stream around
/// them. An implementation that resolved every target's own symbol would give
/// all three a number, and one that resolved none would give all three a dash
/// -- and each of those is what one half of this test alone would still admit.
///
/// The stem and compound arms of `Interp::assign_expr_target` write through a
/// name and through a tail key resolved at the write site; neither writes the
/// symbol's own frame slot, so a number here would be a slot they do not use.
#[test]
fn a_compiled_write_names_a_slot_only_for_a_simple_target() {
    let simple = compile_for_test(b"zw = 'v'\n").expect("compiles");
    assert_eq!(
        render(&simple),
        "0: Clause index=0 end=4\n\
         1: Const dst=0 konst=0\n\
         2: TraceLiteral src=0\n\
         3: Store index=0 at=0 src=0\n"
    );

    for source in [&b"zs. = 'v'\n"[..], &b"zs.zk = 'v'\n"[..]] {
        let chunk = compile_for_test(source).expect("compiles");
        assert_eq!(
            render(&chunk),
            "0: Clause index=0 end=4\n\
             1: Const dst=0 konst=0\n\
             2: TraceLiteral src=0\n\
             3: Store index=0 at=- src=0\n",
            "{} resolved a slot for a target that does not write one",
            String::from_utf8_lossy(source)
        );
    }
}

/// The slot a write carries is the one the **plan** gives that name, not the
/// order the writes appear in.
///
/// Two names written in the reverse order they were first read in: `zb` is
/// read first, so the plan binds it to the lower slot, and the writes below
/// carry `1` then `0` rather than `0` then `1`. A compiler numbering its own
/// stores would print them the other way round and the interpreter would then
/// write each value into the other variable's slot.
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
///
/// The tag is the whole of what tells the two apart in the stream: the ops
/// around it are identical, and what differs is the `Flow` the driver answers,
/// which decides whether a called label's clause resumes its caller or ends
/// the program.
///
/// **`src=-` is not `src` holding a null string**, and the two forms differ in
/// what the *caller* sees: a bare `RETURN` leaves `RESULT` unset where `RETURN
/// ''` sets it, so a register holding an empty value would answer the wrong
/// thing for one of them. Neither bare clause emits a value op at all, which
/// is what the two-op region says.
#[test]
fn a_return_and_an_exit_compile_to_one_op_tagged_with_their_keyword() {
    let chunk = compile_for_test(b"return 1\n").expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=4\n\
         1: LoadConstant dst=0\n\
         2: TraceLiteral src=0\n\
         3: Return index=0 src=0 keyword=RETURN\n"
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
        "0: Clause index=0 end=4\n\
         1: LoadConstant dst=0\n\
         2: TraceLiteral src=0\n\
         3: Return index=0 src=0 keyword=EXIT\n"
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
///
/// The four clauses are one program so that the register reuse is visible: the
/// second clause writes register `0` again, which the first released at its
/// own region's end, and the two bare clauses take none.
#[test]
fn a_push_and_a_queue_compile_to_one_op_tagged_with_their_end() {
    let chunk = compile_for_test(b"push 'a'\nqueue 'b'\npush\nqueue\n").expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=4\n\
         1: Const dst=0 konst=0\n\
         2: TraceLiteral src=0\n\
         3: Queue index=0 src=0 keyword=PUSH\n\
         4: Clause index=1 end=8\n\
         5: Const dst=0 konst=1\n\
         6: TraceLiteral src=0\n\
         7: Queue index=1 src=0 keyword=QUEUE\n\
         8: Clause index=2 end=10\n\
         9: Queue index=2 src=- keyword=PUSH\n\
         10: Clause index=3 end=12\n\
         11: Queue index=3 src=- keyword=QUEUE\n"
    );
}

/// **A `RETURN` whose expression is outside the native set keeps its own op**,
/// with one [`super::Op::EvalExpr`] in front of it -- which is the `SAY`
/// fallback and **not** the `IF` one.
///
/// The difference is what the declining op still owes. An `IF`'s
/// `Op::EvalExpr` evaluates the condition *and* validates it *and* traces it,
/// so nothing follows it; here the `EvalExpr` only evaluates, and the `>>>`
/// line and the `Flow` are still owed by the op behind it. `.nil` is
/// `ExprKind::DotVariable`, which `native_shape` has no arm for.
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
///
/// Without that arm the descent answers `None` and the driver reaches
/// `Loud::call_op_off_its_node` instead of calling anything.
#[test]
fn a_call_in_a_returns_expression_is_addressed_at_the_returns_slot() {
    let chunk = compile_for_test(b"return length(zs)\n").expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=4\n\
         1: CallExpr index=0 slot=0 path=root site=0 dst=0\n\
         2: TraceFunction index=0 slot=0 path=root src=0\n\
         3: Return index=0 src=0 keyword=RETURN\n"
    );
}

/// A promoted `EXIT` compiled under a setting that echoes carries its clause
/// echo as a [`super::Op::TraceClause`] of its own, in front of the value's
/// ops.
///
/// The sibling of the `SAY` and `IF` rows for the same decision (D23), and
/// what it adds is that the ops this task emits sit inside a region whose echo
/// is compiled in rather than gated at run time.
#[test]
fn a_traced_exit_carries_its_clause_echo_op() {
    let chunk = compile_for_test_under(b"exit 2\n", traced()).expect("compiles");
    assert_eq!(
        render(&chunk),
        "0: Clause index=0 end=5\n\
         1: TraceClause index=0\n\
         2: LoadConstant dst=0\n\
         3: TraceLiteral src=0\n\
         4: Return index=0 src=0 keyword=EXIT\n"
    );
}
