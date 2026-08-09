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
            Op::EvalExpr {
                index: at,
                slot,
                dst,
            } => {
                out.push_str(&format!(
                    "{index}: EvalExpr index={at} slot={slot} dst={dst}\n"
                ));
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
