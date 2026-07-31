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

//! `Raised`: the payload of a real Rexx condition.
//!
//! **Only the payload exists here.** Task 12 ("Errors, the message
//! catalogue, and the exit code") owns the message catalogue, the
//! oracle's exact two-line stderr format, the clause echo, and the
//! `256 - number` exit-code mapping -- none of that is built in this
//! task. What exists is enough to assert *which* condition was raised
//! (the condition name, the number and sub-number, and the substitution
//! values), not to reproduce what the oracle prints for it. A test
//! against this type checks "did `1/0` raise 42.3", never "does stderr
//! read `Error 42.3: ...` and does the process exit 214".
//!
//! `Failure` is the other half: `step` and everything above it can fail
//! either because 4a does not implement a construct (`Loud`, `lib.rs`) or
//! because a real condition was raised (`Raised`), and a clause containing
//! an expression can do either, so the propagation type has to carry both.

use crate::Loud;
use rexx_num::ArithError;

/// A real Rexx condition raised during evaluation.
#[derive(Clone, Debug)]
pub(crate) struct Raised {
    /// The condition name a trapped Rexx program would see from
    /// `condition('c')`. Every raiser this task produces is `SYNTAX`, and
    /// it is carried as a field rather than hardcoded at each call site
    /// because the spec's own shape includes it and later tasks (4b's
    /// `NOVALUE`, `NOMETHOD`, ...) need to set it to something else.
    pub(crate) condition: &'static str,
    pub(crate) number: u16,
    pub(crate) sub: u16,
    pub(crate) additional: Vec<String>,
}

impl Raised {
    fn syntax(number: u16, sub: u16, additional: Vec<String>) -> Raised {
        Raised {
            condition: "SYNTAX",
            number,
            sub,
            additional,
        }
    }

    /// 41.1: a nonnumeric value used in arithmetic. `value` is the
    /// operand's own text, verbatim -- measured, `say 'abc' + 1` reports
    /// `Nonnumeric value ("abc")`, the operand as it renders, not upcased
    /// or otherwise transformed.
    pub(crate) fn nonnumeric(value: &[u8]) -> Raised {
        Raised::syntax(41, 1, vec![String::from_utf8_lossy(value).into_owned()])
    }

    /// 26.8: `**`'s right operand is not a whole number, **including not
    /// being a number at all**. Measured: `2 ** 'x'` and `2 ** 2.5` both
    /// give 26.8 ("found \"x\""/"found \"2.5\""), while the identical
    /// failure on the *left* operand is the ordinary 41.1 (`'y' ** 2` is
    /// 41.1, `'y' ** 'x'` is still 41.1 -- the base is checked first).
    /// This is deliberately not routed through `nonnumeric`: the oracle's
    /// own asymmetry between the two operands is the fact being
    /// reproduced, not an implementation shortcut. `found` is the
    /// exponent's own text; used when the exponent does not even parse as
    /// a number, so there is no `Number` for `rexx-num`'s own
    /// `ArithError::PowerExponentNotWhole` to carry.
    pub(crate) fn power_exponent_not_whole(found: &[u8]) -> Raised {
        Raised::syntax(26, 8, vec![String::from_utf8_lossy(found).into_owned()])
    }

    /// 34.901: the prefix `\` operator's operand is not a logical value.
    /// A logical value is *exactly* the one-byte string `0` or `1`, no
    /// coercion -- this is a text check, never a numeric one, which is
    /// why the caller passes the operand's own rendered text rather than
    /// anything from `to_number`. Measured: `say \'abc'` gives 34.901,
    /// `Logical value must be exactly "0" or "1"; found "abc"`.
    pub(crate) fn not_logical(found: &[u8]) -> Raised {
        Raised::syntax(34, 901, vec![String::from_utf8_lossy(found).into_owned()])
    }
}

/// Converts a `rexx-num` arithmetic failure into a `Raised`.
///
/// The `(major, sub)` pair comes from `ArithError::sub_code`, made `pub` in
/// `rexx-num` for exactly this caller (`4a320f1c`) rather than hand-copied
/// here: this task originally flagged that `sub_code` was private and
/// shipped a two-variant stopgap covering only what its own tests had
/// independently verified against the oracle (`DivideByZero`,
/// `PowerExponentNotWhole`), sub `0` elsewhere. The accessor landing
/// retires that stopgap -- every `ArithError` variant now gets its real
/// sub-number, not only the two this task happened to exercise.
impl From<ArithError> for Raised {
    fn from(error: ArithError) -> Raised {
        // `additional()` and `sub_code()` both borrow, so either can run
        // first; ordered to match the doc comment's own telling.
        let additional = error.additional();
        let (number, sub) = error.sub_code();
        Raised::syntax(number, sub, additional)
    }
}

/// Either kind of failure a clause can produce: a construct 4a does not
/// implement (`Loud`) or a real Rexx condition (`Raised`). `step` and
/// everything above it propagate this rather than either alone, since a
/// clause containing an expression can fail either way -- `eval`'s own
/// `ExprKind::Call` arm is `Loud` (not implemented), its `1 / 0` arm is
/// `Raised` (implemented, and this is what it does).
#[derive(Debug)]
pub(crate) enum Failure {
    Loud(Loud),
    Raised(Raised),
}

impl From<Loud> for Failure {
    fn from(loud: Loud) -> Failure {
        Failure::Loud(loud)
    }
}

impl From<Raised> for Failure {
    fn from(raised: Raised) -> Failure {
        Failure::Raised(raised)
    }
}
