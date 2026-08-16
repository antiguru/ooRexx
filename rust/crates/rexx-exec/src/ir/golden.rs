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
//! (`ir.rs`), not a `#[allow(dead_code)]` here: no task in the plan ever
//! gives `render` a non-test caller, so an `allow` would be permanent rather
//! than a placeholder for one.

use super::{Chunk, ConditionKeyword, NodePath, Op, PlanSlot};
use crate::run::{QueueKeyword, ReturnKeyword};

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
            Op::CallExpr {
                index: at,
                slot,
                path,
                site,
                dst,
            } => {
                out.push_str(&format!(
                    "{index}: CallExpr index={at} slot={slot} path={} site={site} dst={dst}\n",
                    render_path(*path)
                ));
            }
            Op::TraceFunction {
                index: at,
                slot,
                path,
                src,
            } => {
                out.push_str(&format!(
                    "{index}: TraceFunction index={at} slot={slot} path={} src={src}\n",
                    render_path(*path)
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
            // The `hint` field is rendered, unlike [`Op::Load`]'s `symbol`,
            // because it is an index this compiler hands out from the program
            // alone: a golden reading `hint=1` for the second arithmetic op is
            // what says the table is dense over these ops rather than over the
            // stream.
            Op::Arith {
                op,
                hint,
                lhs,
                rhs,
                dst,
            } => {
                out.push_str(&format!(
                    "{index}: Arith op={} hint={hint} lhs={lhs} rhs={rhs} dst={dst}\n",
                    op.spelling()
                ));
            }
            // No `hint` field to render, which is what a golden reading this
            // line beside an `Arith` one says: the two ops are the same shape
            // apart from the slot only arithmetic takes.
            Op::Binary { op, lhs, rhs, dst } => {
                out.push_str(&format!(
                    "{index}: Binary op={} lhs={lhs} rhs={rhs} dst={dst}\n",
                    op.spelling()
                ));
            }
            Op::TraceOperator { op, src } => {
                out.push_str(&format!(
                    "{index}: TraceOperator op={} src={src}\n",
                    op.spelling()
                ));
            }
            // The operator by its own spelling, for the reason
            // [`Op::Arith`]'s is: it is the tag the `>P>` line this op's echo
            // prints carries, and a golden reading `Minus` could not say
            // whether the tag would.
            Op::Prefix { op, src, dst } => {
                out.push_str(&format!(
                    "{index}: Prefix op={} src={src} dst={dst}\n",
                    op.spelling()
                ));
            }
            Op::TracePrefix { op, src } => {
                out.push_str(&format!(
                    "{index}: TracePrefix op={} src={src}\n",
                    op.spelling()
                ));
            }
            Op::Store {
                index: node,
                at,
                src,
            } => {
                out.push_str(&format!(
                    "{index}: Store index={node} at={} src={src}\n",
                    render_slot(*at)
                ));
            }
            Op::Say { index: at, src } => {
                out.push_str(&format!(
                    "{index}: Say index={at} src={}\n",
                    render_register(*src)
                ));
            }
            // The keyword is rendered for [`Op::Condition`]'s reason: it
            // decides which `Flow` the op answers, and a golden expectation
            // reading `RETURN` or `EXIT` is what makes that visible in the
            // stream rather than only in what a called label does.
            Op::Return {
                index: at,
                src,
                keyword,
            } => {
                out.push_str(&format!(
                    "{index}: Return index={at} src={} keyword={}\n",
                    render_register(*src),
                    render_return_keyword(*keyword)
                ));
            }
            Op::Queue {
                index: at,
                src,
                keyword,
            } => {
                out.push_str(&format!(
                    "{index}: Queue index={at} src={} keyword={}\n",
                    render_register(*src),
                    render_queue_keyword(*keyword)
                ));
            }
            // The `site` field is rendered, for the reason [`Op::Arith`]'s
            // `hint` is: it is an index this compiler hands out from the
            // program alone, so a golden reading `site=1` for the second call
            // op is what says the table is dense over these ops rather than
            // over the stream.
            Op::Call { index: at, site } => {
                out.push_str(&format!("{index}: Call index={at} site={site}\n"));
            }
            // No `site` to render, unlike `Op::Call` above: a send resolves
            // afresh every time (D28), so this op carries nothing but the
            // clause it runs.
            Op::Message { index: at } => {
                out.push_str(&format!("{index}: Message index={at}\n"));
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
            Op::Condition {
                index: at,
                reg,
                keyword,
            } => {
                out.push_str(&format!(
                    "{index}: Condition index={at} reg={reg} keyword={}\n",
                    render_condition_keyword(*keyword)
                ));
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

/// Which keyword an [`Op::Condition`] validates for, as the keyword itself:
/// the tag decides which raiser answers a value that is not `0`/`1`, and a
/// golden expectation reading `IF` or `WHEN` is what makes that visible in the
/// stream rather than only in a program's stderr.
fn render_condition_keyword(keyword: ConditionKeyword) -> &'static str {
    match keyword {
        ConditionKeyword::If => "IF",
        ConditionKeyword::When => "WHEN",
    }
}

/// Which keyword an [`Op::Return`] ends the activation for, as the keyword
/// itself: the tag decides whether a called label's clause resumes its caller
/// or ends the program.
fn render_return_keyword(keyword: ReturnKeyword) -> &'static str {
    match keyword {
        ReturnKeyword::Return => "RETURN",
        ReturnKeyword::Exit => "EXIT",
    }
}

/// Which end of the queue an [`Op::Queue`] writes to, as the keyword itself.
fn render_queue_keyword(keyword: QueueKeyword) -> &'static str {
    match keyword {
        QueueKeyword::Push => "PUSH",
        QueueKeyword::Queue => "QUEUE",
    }
}

/// The route an op's address takes down from its slot's root: `root` for the
/// slot's own expression, and a step per child below it, `L` into a binary
/// operator's left or a prefix operator's operand and `R` into a binary
/// operator's right.
///
/// Spelled out rather than rendered as the encoding's own integer, because a
/// golden expectation is read by a person: `root.L.R` says where the op sits
/// and the bits behind it do not.
fn render_path(path: NodePath) -> String {
    let mut out = "root".to_string();
    for right in path.steps() {
        out.push('.');
        out.push(if right { 'R' } else { 'L' });
    }
    out
}

/// A compiled read's or write's own slot, or `-` for one that resolves its
/// own.
///
/// The same shape [`render_register`] uses, so an absent operand reads the
/// same way whichever field it is.
fn render_slot(at: PlanSlot) -> String {
    match at.resolved() {
        Some(at) => at.to_string(),
        None => "-".to_string(),
    }
}
