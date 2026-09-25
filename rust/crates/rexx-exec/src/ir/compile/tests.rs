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

use rexx_parse::{SymbolId, SymbolTable};

use super::{
    Op, PlanSlot, Registers, SymbolRead, assert_exec_regions_hold_nothing_else,
    assert_literal_echoes_follow_their_load, assert_read_echoes_follow_their_load,
    assert_region_ops_name_their_clause, assert_trace_ops_open_a_clause_region,
};

/// Two sibling clauses reuse the same registers, and a clause nested
/// inside another's mark does not.
#[test]
fn a_released_register_is_handed_out_again_and_a_nested_one_is_not() {
    let mut registers = Registers::new();

    let outer = registers.mark();
    assert_eq!(registers.alloc().expect("in range"), 0);
    let inner = registers.mark();
    assert_eq!(
        registers.alloc().expect("in range"),
        1,
        "a register allocated while an enclosing one is live must not reuse it"
    );
    registers.release(inner);
    assert_eq!(
        registers.alloc().expect("in range"),
        1,
        "releasing to the inner mark hands the same register out again"
    );
    registers.release(outer);
    assert_eq!(
        registers.alloc().expect("in range"),
        0,
        "releasing to the outer mark hands out the enclosing register too"
    );
}

/// The high-water mark is the deepest the stack ever reached, not the
/// number of registers handed out and not what is live at the end.
#[test]
fn the_high_water_mark_is_the_deepest_the_stack_reached() {
    let mut registers = Registers::new();
    assert_eq!(registers.high_water(), 0, "an empty body reserves nothing");

    for _ in 0..2 {
        let mark = registers.mark();
        registers.alloc().expect("in range");
        registers.alloc().expect("in range");
        registers.release(mark);
    }

    assert_eq!(registers.high_water(), 2);
}

/// A body needing more than `u16::MAX` registers live at once is refused
/// rather than wrapped, which is the register half of the one error
/// `compile` has (the plan's Decisions section: "the compiler has one
/// error, and it is a machine width rather than a language construct").
#[test]
fn a_register_file_wider_than_u16_is_refused() {
    let mut registers = Registers::new();
    for expected in 0..u16::MAX {
        assert_eq!(registers.alloc().expect("in range"), expected);
    }
    assert_eq!(registers.high_water(), u16::MAX);
    assert_eq!(
        registers.alloc().expect_err("one past the last index").what,
        "registers"
    );
}

/// A trace op that is inside a region but not at its head, which is the
/// half of [`Op::TraceClause`]'s contract that a check for "inside a
/// region" alone would miss.
#[test]
#[should_panic(expected = "not the first op of a Clause region")]
fn a_trace_op_after_the_regions_first_op_is_refused() {
    assert_trace_ops_open_a_clause_region(&[
        Op::Clause { index: 0, end: 4 },
        Op::EvalExpr {
            index: 0,
            slot: 0,
            dst: 0,
        },
        Op::TraceClause { index: 0 },
        Op::JumpUnless { reg: 0, target: 4 },
    ]);
}

/// A trace op with no region open at all, the other way the contract is
/// broken: the echo would read whatever value indent the last clause left
/// behind.
#[test]
#[should_panic(expected = "not the first op of a Clause region")]
fn a_trace_op_outside_a_clause_region_is_refused() {
    assert_trace_ops_open_a_clause_region(&[Op::Exec { index: 0 }, Op::TraceClause { index: 1 }]);
}

/// A literal echo in **front** of the load it reads, which is the
/// arrangement that prints whatever the register held before the literal
/// reached it -- and the only line of the transcript it moves is the one
/// carrying the value.
#[test]
#[should_panic(expected = "does not follow the load of the register it reads")]
fn a_literal_echo_in_front_of_its_load_is_refused() {
    assert_literal_echoes_follow_their_load(&[
        Op::Clause { index: 0, end: 4 },
        Op::TraceLiteral { src: 0 },
        Op::Const { dst: 0, konst: 0 },
        Op::Say {
            index: 0,
            src: Some(0),
        },
    ]);
}

/// A literal echo that reads a register the op in front of it did not
/// write, which no ordering check alone would see: the line lands in the
/// right place with the wrong value in it.
#[test]
#[should_panic(expected = "does not follow the load of the register it reads")]
fn a_literal_echo_reading_another_register_is_refused() {
    assert_literal_echoes_follow_their_load(&[
        Op::Clause { index: 0, end: 4 },
        Op::Const { dst: 0, konst: 0 },
        Op::TraceLiteral { src: 1 },
        Op::Say {
            index: 0,
            src: Some(0),
        },
    ]);
}

/// The neighbouring arrangement that must stay accepted, without which both
/// refusals above are satisfied by a check that refuses every stream
/// carrying a literal at all -- which would make every such body a refusal.
#[test]
fn a_literal_echo_behind_its_own_load_is_accepted() {
    assert_literal_echoes_follow_their_load(&[
        Op::Clause { index: 0, end: 4 },
        Op::Const { dst: 0, konst: 0 },
        Op::TraceLiteral { src: 0 },
        Op::Say {
            index: 0,
            src: Some(0),
        },
    ]);
}

/// A read echo in **front** of the load it reads, which prints whatever the
/// register held before the read reached it.
#[test]
#[should_panic(expected = "does not follow the load of the symbol and register it names")]
fn a_read_echo_in_front_of_its_load_is_refused() {
    let (zv, _zw) = two_symbols();
    assert_read_echoes_follow_their_load(&[
        Op::Clause { index: 0, end: 4 },
        Op::TraceRead {
            symbol: zv,
            read: SymbolRead::Simple,
            src: 0,
        },
        Op::Load {
            symbol: zv,
            read: SymbolRead::Simple,
            at: PlanSlot::UNRESOLVED,
            dst: 0,
        },
        Op::Store {
            index: 0,
            at: PlanSlot::UNRESOLVED,
            src: 0,
        },
    ]);
}

/// A read echo reading a register the op in front of it did not write: the
/// line lands in the right place with the wrong value in it.
#[test]
#[should_panic(expected = "does not follow the load of the symbol and register it names")]
fn a_read_echo_reading_another_register_is_refused() {
    let (zv, _zw) = two_symbols();
    assert_read_echoes_follow_their_load(&[
        Op::Clause { index: 0, end: 4 },
        Op::Load {
            symbol: zv,
            read: SymbolRead::Simple,
            at: PlanSlot::UNRESOLVED,
            dst: 0,
        },
        Op::TraceRead {
            symbol: zv,
            read: SymbolRead::Simple,
            src: 1,
        },
        Op::Store {
            index: 0,
            at: PlanSlot::UNRESOLVED,
            src: 0,
        },
    ]);
}

/// A read echo naming a **different symbol** from the load in front of it,
/// which is the failure a literal's echo cannot have and which neither
/// ordering nor the register would see: `>V>` is tagged with the name, so
/// the line prints the right value under the wrong one.
#[test]
#[should_panic(expected = "does not follow the load of the symbol and register it names")]
fn a_read_echo_naming_another_symbol_is_refused() {
    let (zv, zw) = two_symbols();
    assert_read_echoes_follow_their_load(&[
        Op::Clause { index: 0, end: 4 },
        Op::Load {
            symbol: zv,
            read: SymbolRead::Simple,
            at: PlanSlot::UNRESOLVED,
            dst: 0,
        },
        Op::TraceRead {
            symbol: zw,
            read: SymbolRead::Simple,
            src: 0,
        },
        Op::Store {
            index: 0,
            at: PlanSlot::UNRESOLVED,
            src: 0,
        },
    ]);
}

/// The same for the read kind, which decides whether the `>C>` line in
/// front of `>V>` is emitted at all.
#[test]
#[should_panic(expected = "does not follow the load of the symbol and register it names")]
fn a_read_echo_of_another_kind_is_refused() {
    let (zv, _zw) = two_symbols();
    assert_read_echoes_follow_their_load(&[
        Op::Clause { index: 0, end: 4 },
        Op::Load {
            symbol: zv,
            read: SymbolRead::Simple,
            at: PlanSlot::UNRESOLVED,
            dst: 0,
        },
        Op::TraceRead {
            symbol: zv,
            read: SymbolRead::Compound,
            src: 0,
        },
        Op::Store {
            index: 0,
            at: PlanSlot::UNRESOLVED,
            src: 0,
        },
    ]);
}

/// The neighbouring arrangement that must stay accepted, without which all
/// four refusals above are satisfied by a check that refuses every stream
/// carrying a read at all -- which would make every such body a refusal.
#[test]
fn a_read_echo_behind_its_own_load_is_accepted() {
    let (zv, _zw) = two_symbols();
    assert_read_echoes_follow_their_load(&[
        Op::Clause { index: 0, end: 4 },
        Op::Load {
            symbol: zv,
            read: SymbolRead::Simple,
            at: PlanSlot::of(1),
            dst: 0,
        },
        Op::TraceRead {
            symbol: zv,
            read: SymbolRead::Simple,
            src: 0,
        },
        Op::Store {
            index: 0,
            at: PlanSlot::UNRESOLVED,
            src: 0,
        },
    ]);
}

/// Two distinct `SymbolId`s, from a table of this test module's own so the
/// ids are the parser's rather than numbers invented here.
fn two_symbols() -> (SymbolId, SymbolId) {
    let mut symbols = SymbolTable::default();
    (symbols.intern("ZV"), symbols.intern("ZW"))
}

/// A slot too wide for a compiled read's own field is **not** a refusal:
/// the read still has a correct answer and the run-time path is what
/// computes it, so the whole body must not be refused for something that
/// is only an optimisation.
#[test]
fn a_slot_too_wide_for_a_compiled_read_is_unresolved_rather_than_refused() {
    assert_eq!(PlanSlot::of(0).resolved(), Some(0));
    let last = u32::MAX as usize - 1;
    assert_eq!(PlanSlot::of(last).resolved(), Some(last));
    assert_eq!(PlanSlot::of(u32::MAX as usize).resolved(), None);
    assert_eq!(PlanSlot::UNRESOLVED.resolved(), None);
}

/// The neighbouring arrangement that must stay accepted, without which
/// both refusals above are satisfied by a check that refuses every stream
/// carrying a trace op -- which would make every traced body a refusal.
#[test]
fn a_trace_op_at_the_head_of_a_clause_region_is_accepted() {
    assert_trace_ops_open_a_clause_region(&[
        Op::Clause { index: 0, end: 4 },
        Op::TraceClause { index: 0 },
        Op::EvalExpr {
            index: 0,
            slot: 0,
            dst: 0,
        },
        Op::JumpUnless { reg: 0, target: 4 },
    ]);
}

/// An op inside a region that names the instruction *next* to the region's
/// clause, which is the arrangement the driver cannot detect.
#[test]
#[should_panic(expected = "names instruction 1 inside the region of clause 0")]
fn a_region_op_naming_a_neighbouring_instruction_is_refused() {
    assert_region_ops_name_their_clause(&[
        Op::Clause { index: 0, end: 3 },
        Op::EvalExpr {
            index: 1,
            slot: 0,
            dst: 0,
        },
        Op::Store {
            index: 0,
            at: PlanSlot::UNRESOLVED,
            src: 0,
        },
    ]);
}

/// An op that drives clauses of its own, inside the region of an `Exec`.
#[test]
#[should_panic(expected = "op 2 sits in the Exec region of clause 0")]
fn an_exec_region_holding_an_op_that_drives_clauses_is_refused() {
    assert_exec_regions_hold_nothing_else(&[
        Op::Clause { index: 0, end: 4 },
        Op::TraceClause { index: 0 },
        Op::LoopRun { index: 0 },
        Op::Exec { index: 0 },
    ]);
}

/// The two arrangements that must stay accepted: the shape `compile`
/// actually emits, echoed and not, and a region with no `Exec` in it at
/// all, whose ops this check has no opinion about.
#[test]
fn the_emitted_exec_shapes_and_every_other_region_are_accepted() {
    assert_exec_regions_hold_nothing_else(&[
        Op::Clause { index: 0, end: 3 },
        Op::TraceClause { index: 0 },
        Op::Exec { index: 0 },
        Op::Clause { index: 1, end: 5 },
        Op::Exec { index: 1 },
        Op::Clause { index: 2, end: 8 },
        Op::LoopRun { index: 2 },
        Op::LoopNext { index: 2 },
    ]);
}

/// The same for the echo op, which carries an index of its own and reaches
/// a different function with it.
#[test]
#[should_panic(expected = "names instruction 2 inside the region of clause 1")]
fn a_trace_op_naming_a_neighbouring_instruction_is_refused() {
    assert_region_ops_name_their_clause(&[
        Op::Exec { index: 0 },
        Op::Clause { index: 1, end: 4 },
        Op::TraceClause { index: 2 },
        Op::EvalExpr {
            index: 1,
            slot: 0,
            dst: 0,
        },
    ]);
}

/// The neighbouring arrangement that must stay accepted: two regions, each
/// of whose ops names its own clause, with an index-bearing op *outside* any
/// region naming a third instruction.
#[test]
fn ops_naming_their_own_region_clause_are_accepted() {
    assert_region_ops_name_their_clause(&[
        Op::Clause { index: 0, end: 3 },
        Op::TraceClause { index: 0 },
        Op::Store {
            index: 0,
            at: PlanSlot::UNRESOLVED,
            src: 0,
        },
        Op::SelectCaseText {
            index: 7,
            case: None,
        },
        Op::Clause { index: 1, end: 6 },
        Op::Say {
            index: 1,
            src: Some(0),
        },
    ]);
}
