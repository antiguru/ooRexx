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

//! The `Raised` constructors for the conditions the instruction loop raises.

use super::{Number, Raised, SettingsError};

/// 20.928: a subsidiary-list word is not a legal symbol at all (contains a
/// byte outside `is_symbol_byte`'s set, which is also what a parenthesised
/// entry like `"(w)"` fails on).
pub(super) fn raised_symbol_expected(found: &[u8]) -> Raised {
    Raised::syntax(20, 928, vec![found.to_vec()])
}

/// 31.2: a subsidiary-list word starts with a digit.
pub(super) fn raised_digit_led(found: &[u8]) -> Raised {
    Raised::syntax(31, 2, vec![found.to_vec()])
}

/// 31.3: a subsidiary-list word starts with a period.
pub(super) fn raised_dot_led(found: &[u8]) -> Raised {
    Raised::syntax(31, 3, vec![found.to_vec()])
}

/// 34.1: a single (non-list) `IF` condition is not exactly `0` or `1`.
/// `Error_Logical_value_if`, catalogue text "Value of expression following
/// IF keyword must be exactly \"0\" or \"1\"; found \"...\"", one
/// substitution, the operand's own rendered text.
pub(crate) fn raised_if_not_logical(found: &[u8]) -> Raised {
    Raised::syntax(34, 1, vec![found.to_vec()])
}

/// 34.902: a `GUARD ... WHEN` condition is not exactly `0` or `1`.
/// `Error_Logical_value_guard`, catalogue text "Value of expression following
/// GUARD keyword must be exactly \"0\" or \"1\"; found \"...\"", one
/// substitution, the operand's own rendered text. `truthValue(Error_Logical_
/// value_guard)` at `instructions/GuardInstruction.cpp:167` is what selects
/// this sub-number over `IF`'s and `WHEN`'s.
pub(super) fn raised_guard_not_logical(found: &[u8]) -> Raised {
    Raised::syntax(34, 902, vec![found.to_vec()])
}

/// 34.2: a single (non-list) `WHEN` condition is not exactly `0` or `1`.
/// `Error_Logical_value_when`, the same shape as `raised_if_not_logical`
/// with `WHEN`'s own sub-number -- a plain `WHEN`'s comma list is the
/// opposite case (`WhenCase`'s doc comment) and never reaches this raiser:
/// [`crate::Interp::eval_condition`] hands a list over already `checked`, and
/// `crate::ir::compile`'s `native_shape` declines `ExprKind::Logical`, so a
/// list never becomes a `crate::ir::Op::ConditionJump` either.
pub(crate) fn raised_when_not_logical(found: &[u8]) -> Raised {
    Raised::syntax(34, 2, vec![found.to_vec()])
}

/// 7.3: a `SELECT` reached its `END` with every `WHEN` false and no
/// `OTHERWISE`. `Error_When_expected_nootherwise`, catalogue text "All WHEN
/// expressions of SELECT are false; OTHERWISE expected.", no substitutions
/// (measured against `interpreter/messages/rexxmsg.xml`'s own `<Text>` for
/// major 7 sub 003, which carries no `<Sub>` tag).
pub(super) fn raised_select_no_when() -> Raised {
    Raised::syntax(7, 3, Vec::new())
}

/// Converts a `rexx-num` settings failure into a `Raised`.
/// ```text
/// raise syntax 40.4       -> 40.4       the catalogue entry
/// raise syntax 40         -> 40.0       ditto, major line only (sub 0)
/// raise syntax 40.001     -> 40.1       ".001" is the integer 1
/// raise syntax '4E1'      -> 40.0       each half is a Rexx number, not an int literal
/// raise syntax '40.1E2'   -> 98.941     found "40100"
/// raise syntax 40.10      -> 98.941     found "40010"
/// raise syntax 1          -> 98.941     found "1.0"
/// raise syntax 3.5        -> 98.941     found "3005"
/// raise syntax 0 / 100 / 999 / 'abc' / 40.1000 / '40.'  -> 33.904
/// ```
pub(super) fn raise_syntax_condition(text: &[u8], additional: Vec<Vec<u8>>) -> Raised {
    /// One half of the argument as a Rexx number: `numberValue`, then a
    /// whole-number check. `None` for anything that is not a whole number,
    /// which the caller turns into 33.904.
    fn whole(text: &str) -> Option<i64> {
        Number::parse(text)?.whole_value(rexx_num::DEFAULT_DIGITS as usize)
    }

    let text = String::from_utf8_lossy(text);
    let (major, sub) = match text.split_once('.') {
        // A decimal point with an empty tail is rejected rather than read as
        // zero: `'40.'` is 33.904 where `40` is the `(40, 0)` entry.
        Some((_, "")) => return Raised::syntax(33, 904, Vec::new()),
        Some((major, sub)) => (whole(major), whole(sub)),
        None => (whole(text.as_ref()), Some(0)),
    };
    let (Some(major), Some(sub)) = (major, sub) else {
        return Raised::syntax(33, 904, Vec::new());
    };
    if !(1..=99).contains(&major) || !(0..=999).contains(&sub) {
        return Raised::syntax(33, 904, Vec::new());
    }
    // Both bounds are checked above, so neither narrowing can lose anything.
    raised_for_code(major as u64 * 1000 + sub as u64, additional)
}

/// The `SYNTAX` condition for error `code`, `major * 1000 + sub`, or 98.941
/// where the catalogue has no text for it: naming the code as a whole number
/// where the major has text, and as `major.sub` where it has none
/// (`Activity::createExceptionObject` and `Activity::buildMessage`,
/// `interpreter/concurrency/Activity.cpp:1017`, `:1229`).
pub(crate) fn raised_for_code(code: u64, additional: Vec<Vec<u8>>) -> Raised {
    let (major, sub) = (code / 1000, code % 1000);
    let entry = |major: u64, sub: u64| {
        let (major, sub) = (u16::try_from(major).ok()?, u16::try_from(sub).ok()?);
        rexx_inventory::errors::lookup(major, sub).map(|_| (major, sub))
    };
    if let Some((major, sub)) = entry(major, sub) {
        return Raised::syntax(major, sub, additional);
    }
    let found = if entry(major, 0).is_some() {
        code.to_string()
    } else {
        format!("{major}.{sub}")
    };
    Raised::syntax(98, 941, vec![found.into_bytes()])
}

/// [`raised_from_settings`] with the operand's own rendering in place of the
/// string the required-string protocol converted it to.
pub(super) fn raised_naming_the_operand(error: SettingsError, operand: &[u8]) -> Raised {
    let names_the_operand = matches!(
        error,
        SettingsError::InvalidForm { .. }
            | SettingsError::DigitsNotWhole { .. }
            | SettingsError::FuzzNotWhole { .. }
    );
    let mut raised = raised_from_settings(error);
    if names_the_operand {
        raised.additional = vec![operand.to_vec()];
    }
    raised
}

pub(crate) fn raised_from_settings(error: SettingsError) -> Raised {
    let additional = crate::error::into_substitutions(error.additional());
    let (number, sub): (u16, u16) = match &error {
        SettingsError::InvalidForm { .. } => (25, 11),
        SettingsError::DigitsNotWhole { .. } => (26, 5),
        SettingsError::FuzzNotWhole { .. } => (26, 6),
        SettingsError::FuzzNotBelowDigits { .. } => (33, 1),
    };
    Raised::syntax(number, sub, additional)
}

/// 34.3: a single (non-list) `WHILE` condition is not exactly `0` or `1`.
/// Same shape as `raised_if_not_logical`/`raised_when_not_logical`, with
/// `WHILE`'s own sub-number; a comma-list condition never reaches this
/// raiser (34.6 instead, `eval_logical_list`'s own answer).
pub(super) fn raised_while_not_logical(found: &[u8]) -> Raised {
    Raised::syntax(34, 3, vec![found.to_vec()])
}

/// 34.4: `UNTIL`'s own version of `raised_while_not_logical`.
pub(super) fn raised_until_not_logical(found: &[u8]) -> Raised {
    Raised::syntax(34, 4, vec![found.to_vec()])
}

/// 26.2: a bare `DO`'s own repetition-count expression is not zero or a
/// positive whole number. `Error_Invalid_expression_do`, measured: `do
/// 'a'`/`do -1`/`do 2.5` all give this, `found` the operand's own
/// unmodified text (`"a"`/`"-1"`/`"2.5"`).
pub(super) fn raised_repetition_count_not_whole(found: &[u8]) -> Raised {
    Raised::syntax(26, 2, vec![found.to_vec()])
}

/// 26.3: a `DO`/`LOOP`'s `FOR` expression is not zero or a positive whole
/// number. Measured: `do i = 1 to 3 for 'x'`/`for -1`/`for 1.5`.
pub(super) fn raised_for_count_not_whole(found: &[u8]) -> Raised {
    Raised::syntax(26, 3, vec![found.to_vec()])
}

/// 28.1: a bare `LEAVE` found no repetitive loop or labeled block
/// instruction anywhere on the enclosing chain. No substitution.
pub(super) fn raised_leave_no_loop() -> Raised {
    Raised::syntax(28, 1, Vec::new())
}

/// 28.2: a bare `ITERATE` found no repetitive loop anywhere on the
/// enclosing chain. No substitution.
pub(super) fn raised_iterate_no_loop() -> Raised {
    Raised::syntax(28, 2, Vec::new())
}

/// 28.3: a named `LEAVE name` found nothing on the enclosing chain whose own
/// label (`DO LABEL`, or a controlled/`OVER` loop's own control variable)
/// matches `name` -- **an ordinary clause label never matches**, measured:
/// `outer: do i = 1 to 3` then `leave outer` is this, not a hit. `found` is
/// the symbol's own (already-upcased) spelling.
pub(super) fn raised_leave_no_match(found: &[u8]) -> Raised {
    Raised::syntax(28, 3, vec![found.to_vec()])
}

/// 28.4: `ITERATE`'s own version of `raised_leave_no_match`.
pub(super) fn raised_iterate_no_match(found: &[u8]) -> Raised {
    Raised::syntax(28, 4, vec![found.to_vec()])
}

/// 28.5: a named `ITERATE name` matched a block on the enclosing chain by
/// label, but that block is not a repetitive loop (a labelled `DO`/plain
/// block, or a `SELECT LABEL` -- `ITERATE` never accepts either, unlike
/// `LEAVE`). Measured: `do label x / say 1 / iterate x / end` gives this,
/// not 28.4, because `x` *did* match something.
pub(super) fn raised_iterate_wrong_kind(found: &[u8]) -> Raised {
    Raised::syntax(28, 5, vec![found.to_vec()])
}
