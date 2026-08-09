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

use super::{Chunk, Op};

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
            Op::Loop { index: at } => {
                out.push_str(&format!("{index}: Loop index={at}\n"));
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
