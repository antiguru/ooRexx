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
//! `golden_tests.rs` alone. `render` is `pub(crate)` and not `pub`: nothing
//! outside this crate reads an op stream, and the only reader inside it
//! today is a test.

use super::{Chunk, Op};

/// Renders `chunk` as one line per op: `"{index}: {OpName}[ field=value]*"`.
///
/// Exhaustive over [`Op`] with no catch-all arm, so a new variant that
/// nobody teaches this function to render is a compile error here rather
/// than a silently blank line in a golden test's expected output.
#[allow(
    dead_code,
    reason = "called only from golden_tests.rs's #[test] functions"
)]
pub(crate) fn render(chunk: &Chunk) -> String {
    let mut out = String::new();
    for (index, op) in chunk.ops.iter().enumerate() {
        match op {
            Op::Generic => {
                out.push_str(&format!("{index}: Generic\n"));
            }
            Op::Clause { end } => {
                out.push_str(&format!("{index}: Clause end={end}\n"));
            }
            Op::EvalExpr {
                index: expr,
                slot,
                dst,
            } => {
                out.push_str(&format!(
                    "{index}: EvalExpr index={expr} slot={slot} dst={dst}\n"
                ));
            }
        }
    }
    out
}
