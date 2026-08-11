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

//! The golden op-stream serialiser, for the golden tests in
//! `golden_tests.rs` alone. `#[cfg(test)]` at the `mod` declaration
//! (`ir/mod.rs`), not a `#[allow(dead_code)]` here: no task in the plan ever
//! gives `render` a non-test caller, so an `allow` would be permanent rather
//! than a placeholder for one.

use super::{Chunk, Op, ReadSlot};

/// Renders `chunk` as one line per op: `"{index}: {OpName}[ field=value]*"`.
///
/// Exhaustive over [`Op`] with no catch-all arm, so a new variant that
/// nobody teaches this function to render is a compile error here rather
/// than a silently blank line in a golden test's expected output.
pub(crate) fn render(chunk: &Chunk) -> String {
    let mut out = String::new();
    for (index, op) in chunk.ops.iter().enumerate() {
        match op {
            Op::Generic { index: at } => {
                out.push_str(&format!("{index}: Generic index={at}\n"));
            }
            Op::TraceKeyword { role, src } => {
                out.push_str(&format!("{index}: TraceKeyword role={role:?} src={src}\n"));
            }
            Op::LoopHeaderValue { role, src } => {
                out.push_str(&format!(
                    "{index}: LoopHeaderValue role={role:?} src={src}\n"
                ));
            }
            Op::LoopRun { index: at } => {
                out.push_str(&format!("{index}: LoopRun index={at}\n"));
            }
            Op::Clause { index: at, end } => {
                out.push_str(&format!("{index}: Clause index={at} end={end}\n"));
            }
            Op::TraceClause { index: at } => {
                out.push_str(&format!("{index}: TraceClause index={at}\n"));
            }
            Op::EvalExpr {
                index: at,
                slot,
                dst,
            } => {
                out.push_str(&format!(
                    "{index}: EvalExpr index={at} slot={slot} dst={dst}\n"
                ));
            }
            Op::SelectCaseText { index: at, case } => {
                out.push_str(&format!(
                    "{index}: SelectCaseText index={at} case={}\n",
                    render_register(*case)
                ));
            }
            Op::WhenTest {
                index: at,
                case,
                dst,
            } => {
                out.push_str(&format!(
                    "{index}: WhenTest index={at} case={} dst={dst}\n",
                    render_register(*case)
                ));
            }
            Op::Const { dst, konst } => {
                out.push_str(&format!("{index}: Const dst={dst} konst={konst}\n"));
            }
            // The `symbol` field is not rendered, for the reason [`Op::Load`]'s
            // is not: a `SymbolId`'s index is into a table pre-seeded with
            // every keyword spelling, so the number says nothing about the
            // program. What identifies this one is the register it lands in and
            // the expression the test compiled.
            Op::LoadConstant { dst, .. } => {
                out.push_str(&format!("{index}: LoadConstant dst={dst}\n"));
            }
            Op::TraceLiteral { src } => {
                out.push_str(&format!("{index}: TraceLiteral src={src}\n"));
            }
            // **The `symbol` field is deliberately not rendered.** A
            // `SymbolId`'s index is
            // into the one symbol table `scan` pre-seeds with every keyword
            // spelling before it reads a byte of source, so the number a
            // program's own first symbol gets says nothing about that program
            // and would move under any change to those tables. What identifies
            // the symbol here without that is `at`, its own frame slot, which
            // the plan assigns from the program alone;
            // `a_compiled_read_names_the_symbol_its_expression_does` is what
            // pins the id itself, against the parsed expression rather than
            // against a number.
            Op::Load { read, at, dst, .. } => {
                out.push_str(&format!(
                    "{index}: Load read={read:?} at={} dst={dst}\n",
                    render_slot(*at)
                ));
            }
            Op::TraceRead { read, src, .. } => {
                out.push_str(&format!("{index}: TraceRead read={read:?} src={src}\n"));
            }
            // The operator by its own spelling rather than by its `Debug`
            // name, because the spelling is what the `>O>` line this op's
            // echo prints carries, and a golden that read `Plus` could not
            // say whether the tag would.
            Op::Arith { op, lhs, rhs, dst } => {
                out.push_str(&format!(
                    "{index}: Arith op={} lhs={lhs} rhs={rhs} dst={dst}\n",
                    op.spelling()
                ));
            }
            Op::TraceOperator { op, src } => {
                out.push_str(&format!(
                    "{index}: TraceOperator op={} src={src}\n",
                    op.spelling()
                ));
            }
            Op::Store { index: at, src } => {
                out.push_str(&format!("{index}: Store index={at} src={src}\n"));
            }
            Op::Say { index: at, src } => {
                out.push_str(&format!(
                    "{index}: Say index={at} src={}\n",
                    render_register(*src)
                ));
            }
            Op::EndBranch => {
                out.push_str(&format!("{index}: EndBranch\n"));
            }
            Op::EnterWhen { select, when } => {
                out.push_str(&format!("{index}: EnterWhen select={select} when={when}\n"));
            }
            Op::EnterOtherwise { select } => {
                out.push_str(&format!("{index}: EnterOtherwise select={select}\n"));
            }
            Op::Jump { target } => {
                out.push_str(&format!("{index}: Jump target={target}\n"));
            }
            Op::JumpUnless { reg, target } => {
                out.push_str(&format!("{index}: JumpUnless reg={reg} target={target}\n"));
            }
        }
    }
    out
}

/// An optional register operand: its index, or `-` for an op that has none.
///
/// A rendering rather than `Option`'s own `Debug`, so a golden expectation
/// reads as one word per field.
fn render_register(register: Option<u16>) -> String {
    match register {
        Some(register) => register.to_string(),
        None => "-".to_string(),
    }
}

/// A compiled read's own slot, or `-` for a read that resolves its own.
///
/// The same shape [`render_register`] uses, so an absent operand reads the
/// same way whichever field it is.
fn render_slot(at: ReadSlot) -> String {
    match at.resolved() {
        Some(at) => at.to_string(),
        None => "-".to_string(),
    }
}
