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

//! An op stream whose every register operand is inside the chunk's register
//! count, checked once when the chunk is built. The only way to a `Chunk`'s
//! ops is through this type.

use super::{ChunkTooLarge, Op};

pub(crate) struct ValidOps(Vec<Op>);

impl ValidOps {
    /// Accepts `ops` only if every register it names is below `registers`.
    pub(crate) fn new(ops: Vec<Op>, registers: u16) -> Result<ValidOps, ChunkTooLarge> {
        let refused = ChunkTooLarge {
            what: "a register operand outside the chunk's register count",
        };
        let inside = |reg: u16| reg < registers;
        let arg = |reg: u16| reg == Op::ARG_OMITTED || reg < registers;
        let optional = |reg: Option<u16>| reg.is_none_or(inside);
        for op in &ops {
            let ok = match *op {
                Op::TraceKeyword { src, .. }
                | Op::LoopHeaderValue { src, .. }
                | Op::TraceFunction { src, .. }
                | Op::TraceLiteral { src }
                | Op::TraceRead { src, .. }
                | Op::TraceOperator { src, .. }
                | Op::TracePrefix { src, .. }
                | Op::Store { src, .. } => inside(src),
                Op::PushArg { src } | Op::TraceArgument { src } => arg(src),
                Op::EvalExpr { dst, .. }
                | Op::CallExpr { dst, .. }
                | Op::CallArgs { dst, .. }
                | Op::Const { dst, .. }
                | Op::LoadConstant { dst, .. }
                | Op::Load { dst, .. } => inside(dst),
                Op::SelectCaseText { case, .. } => optional(case),
                Op::WhenTest { case, dst, .. } => optional(case) && inside(dst),
                Op::Arith { lhs, rhs, dst, .. } | Op::Binary { lhs, rhs, dst, .. } => {
                    inside(lhs) && inside(rhs) && inside(dst)
                }
                Op::Prefix { src, dst, .. } => inside(src) && inside(dst),
                Op::Say { src, .. }
                | Op::Signal { src, .. }
                | Op::Parse { src, .. }
                | Op::Return { src, .. }
                | Op::Queue { src, .. } => optional(src),
                Op::JumpUnless { reg, .. } | Op::ConditionJump { reg, .. } => inside(reg),
                Op::Clause { .. }
                | Op::LoopRun { .. }
                | Op::LoopNext { .. }
                | Op::TraceClause { .. }
                | Op::EndBranch
                | Op::EndWhen
                | Op::EnterWhen { .. }
                | Op::EnterOtherwise { .. }
                | Op::Call { .. }
                | Op::CallNamed { .. }
                | Op::Message { .. }
                | Op::Expose { .. }
                | Op::Exec { .. }
                | Op::Escape { .. }
                | Op::Jump { .. } => true,
            };
            if !ok {
                return Err(refused);
            }
        }
        Ok(ValidOps(ops))
    }
}

impl std::ops::Deref for ValidOps {
    type Target = [Op];

    fn deref(&self) -> &[Op] {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::{Op, ValidOps};

    #[test]
    fn a_register_at_the_count_is_refused_and_one_below_it_is_accepted() {
        let op = |dst| Op::Const { dst, konst: 0 };
        assert!(ValidOps::new(vec![op(2)], 3).is_ok());
        assert!(ValidOps::new(vec![op(3)], 3).is_err());
        assert!(
            ValidOps::new(
                vec![Op::PushArg {
                    src: Op::ARG_OMITTED
                }],
                0
            )
            .is_ok()
        );
        assert!(ValidOps::new(vec![Op::PushArg { src: 0 }], 0).is_err());
    }
}
