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

//! The op-stream invariants `compile` asserts over every chunk it builds.

use super::super::Op;

/// **Every [`Op::TraceClause`] is the first op of a [`Op::Clause`] region**,
/// which is both halves of that op's own contract at once.
pub(super) fn assert_trace_ops_open_a_clause_region(ops: &[Op]) {
    for (at, op) in ops.iter().enumerate() {
        if !matches!(op, Op::TraceClause { .. }) {
            continue;
        }
        let opens_here = at
            .checked_sub(1)
            .and_then(|before| ops.get(before))
            .is_some_and(|before| matches!(before, Op::Clause { end, .. } if *end as usize > at));
        assert!(
            opens_here,
            "the trace op at {at} is not the first op of a Clause region, so it echoes against \
             another clause's indent or after a value line it has to precede"
        );
    }
}

/// **Every [`Op::TraceLiteral`] sits immediately behind the [`Op::Const`] or
/// [`Op::LoadConstant`] whose own register it reads**, which is both halves of
/// that op's contract at once.
pub(super) fn assert_literal_echoes_follow_their_load(ops: &[Op]) {
    for (at, op) in ops.iter().enumerate() {
        let Op::TraceLiteral { src } = op else {
            continue;
        };
        let loads_it = at
            .checked_sub(1)
            .and_then(|before| ops.get(before))
            .is_some_and(|before| {
                matches!(
                    before,
                    Op::Const { dst, .. } | Op::LoadConstant { dst, .. } if dst == src
                )
            });
        assert!(
            loads_it,
            "the literal echo at {at} does not follow the load of the register it reads, so it \
             echoes a value that op did not put there"
        );
    }
}

/// **Every [`Op::TraceRead`] sits immediately behind the [`Op::Load`] it
/// echoes**, reading that op's register and repeating its symbol and its read
/// kind -- all three halves of that op's contract at once.
pub(super) fn assert_read_echoes_follow_their_load(ops: &[Op]) {
    for (at, op) in ops.iter().enumerate() {
        let Op::TraceRead { symbol, read, src } = op else {
            continue;
        };
        let loads_it = at
            .checked_sub(1)
            .and_then(|before| ops.get(before))
            .is_some_and(|before| {
                matches!(
                    before,
                    Op::Load {
                        symbol: loaded,
                        read: kind,
                        dst,
                        ..
                    } if loaded == symbol && kind == read && dst == src
                )
            });
        assert!(
            loads_it,
            "the read echo at {at} does not follow the load of the symbol and register it \
             names, so it echoes a value or a name that op did not put there"
        );
    }
}

/// **Every [`Op::TraceOperator`] sits immediately behind the [`Op::Arith`] or
/// [`Op::Binary`] it echoes**, reading that op's destination register and
/// repeating its operator.
pub(super) fn assert_operator_echoes_follow_their_op(ops: &[Op]) {
    for (at, op) in ops.iter().enumerate() {
        let Op::TraceOperator { op: echoed, src } = op else {
            continue;
        };
        let computes_it = at
            .checked_sub(1)
            .and_then(|before| ops.get(before))
            .is_some_and(|before| {
                matches!(
                    before,
                    Op::Arith { op: applied, dst, .. } | Op::Binary { op: applied, dst, .. }
                        if applied == echoed && dst == src
                )
            });
        assert!(
            computes_it,
            "the operator echo at {at} does not follow the operation whose operator and register \
             it names, so it echoes a value or a sign that op did not put there"
        );
    }
}

/// **Every [`Op::TracePrefix`] sits immediately behind the [`Op::Prefix`] it
/// echoes**, reading that op's destination register and repeating its operator.
pub(super) fn assert_prefix_echoes_follow_their_op(ops: &[Op]) {
    for (at, op) in ops.iter().enumerate() {
        let Op::TracePrefix { op: echoed, src } = op else {
            continue;
        };
        let computes_it = at
            .checked_sub(1)
            .and_then(|before| ops.get(before))
            .is_some_and(|before| {
                matches!(
                    before,
                    Op::Prefix { op: applied, dst, .. } if applied == echoed && dst == src
                )
            });
        assert!(
            computes_it,
            "the prefix echo at {at} does not follow the operation whose operator and register \
             it names, so it echoes a value or a sign that op did not put there"
        );
    }
}

/// **Every [`Op::TraceFunction`] sits immediately behind the [`Op::CallExpr`]
/// it echoes**, reading that op's destination register and repeating its whole
/// address.
pub(super) fn assert_call_echoes_follow_their_op(ops: &[Op]) {
    for (at, op) in ops.iter().enumerate() {
        let Op::TraceFunction {
            index: echoed,
            slot: echoed_slot,
            path: echoed_path,
            src,
        } = op
        else {
            continue;
        };
        let runs_it = at
            .checked_sub(1)
            .and_then(|before| ops.get(before))
            .is_some_and(|before| {
                match before {
                    Op::CallExpr {
                        index,
                        slot,
                        path,
                        dst,
                        ..
                    } => {
                        index == echoed && slot == echoed_slot && path == echoed_path && dst == src
                    }
                    // No `index` of its own to compare -- `Op::CallArgs`'s own
                    // doc has why it carries none, and this check is what
                    // gives it the echo's instead.
                    Op::CallArgs {
                        slot, path, dst, ..
                    } => slot == echoed_slot && path == echoed_path && dst == src,
                    _ => false,
                }
            });
        assert!(
            runs_it,
            "the function echo at {at} does not follow the call whose address and register it \
             names, so it echoes a value that call did not produce or reads a node that call \
             did not run"
        );
    }
}

/// **Every [`Op::TraceKeyword`] is immediately followed by the
/// [`Op::LoopHeaderValue`] that files the value it echoes**, reading that op's
/// register and naming that op's role.
pub(super) fn assert_keyword_echoes_precede_their_value(ops: &[Op]) {
    for (at, op) in ops.iter().enumerate() {
        let Op::TraceKeyword { role: echoed, src } = op else {
            continue;
        };
        let files_it = ops.get(at + 1).is_some_and(|next| {
            matches!(
                next,
                Op::LoopHeaderValue { role, src: filed } if role == echoed && filed == src
            )
        });
        assert!(
            files_it,
            "the keyword echo at {at} is not in front of the header value it names, so it echoes \
             a value or a keyword that op does not file"
        );
    }
}

/// Every [`Op::Exec`] region holds nothing but its own [`Op::Clause`], the
/// optional [`Op::TraceClause`] between them, and the `Exec` itself.
pub(super) fn assert_exec_regions_hold_nothing_else(ops: &[Op]) {
    for (at, op) in ops.iter().enumerate() {
        let Op::Clause { end, .. } = op else {
            continue;
        };
        let inside = &ops[at + 1..(*end as usize).min(ops.len())];
        if !inside.iter().any(|op| matches!(op, Op::Exec { .. })) {
            continue;
        }
        for (offset, op) in inside.iter().enumerate() {
            assert!(
                matches!(op, Op::Exec { .. } | Op::TraceClause { .. }),
                "op {} sits in the Exec region of clause {at}, which may hold                  nothing but its echo and the Exec itself",
                at + 1 + offset
            );
        }
    }
}

/// **Every index-bearing op inside a [`Op::Clause`] region names that region's
/// own clause.**
pub(super) fn assert_region_ops_name_their_clause(ops: &[Op]) {
    for (at, op) in ops.iter().enumerate() {
        let Op::Clause { index, end } = op else {
            continue;
        };
        for (inside, op) in ops[at + 1..(*end as usize).min(ops.len())]
            .iter()
            .enumerate()
        {
            let named = match op {
                Op::TraceClause { index }
                | Op::EvalExpr { index, .. }
                | Op::Store { index, .. }
                | Op::Say { index, .. }
                | Op::Return { index, .. }
                | Op::Queue { index, .. }
                | Op::WhenTest { index, .. }
                | Op::Call { index, .. }
                | Op::CallNamed { index, .. }
                | Op::Message { index }
                | Op::Expose { index }
                | Op::Exec { index }
                | Op::Escape { index }
                | Op::CallExpr { index, .. }
                | Op::TraceFunction { index, .. }
                | Op::ConditionJump { index, .. }
                | Op::LoopRun { index }
                | Op::LoopNext { index }
                | Op::Signal { index, .. }
                | Op::Parse { index, .. } => Some(*index),
                Op::TraceKeyword { .. }
                | Op::LoopHeaderValue { .. }
                | Op::Clause { .. }
                | Op::SelectCaseText { .. }
                | Op::EndBranch
                | Op::EndWhen
                | Op::EnterWhen { .. }
                | Op::EnterOtherwise { .. }
                | Op::Const { .. }
                | Op::LoadConstant { .. }
                | Op::TraceLiteral { .. }
                | Op::Load { .. }
                | Op::TraceRead { .. }
                | Op::Arith { .. }
                | Op::Binary { .. }
                | Op::TraceOperator { .. }
                | Op::Prefix { .. }
                | Op::TracePrefix { .. }
                | Op::Jump { .. }
                | Op::PushArg { .. }
                | Op::TraceArgument { .. }
                | Op::CallArgs { .. }
                | Op::JumpUnless { .. } => None,
            };
            assert!(
                named.is_none_or(|named| named == *index),
                "the op at {} names instruction {} inside the region of clause {index}, so the \
                 driver would hand it the wrong instruction",
                at + 1 + inside,
                named.unwrap_or_default()
            );
        }
    }
}
