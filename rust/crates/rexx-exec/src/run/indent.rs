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

//! Each clause's static nesting indent, and the printed indent built on it.

use super::{Code, Instruction, InstructionKind, Interp};

impl Interp {
    /// `target`'s own **absolute printed indent**: its lexical
    /// `static_indent`, plus the activation base it is running under, plus
    /// any escape elevation currently in force.
    pub(crate) fn printed_indent(&self, code: &Code<'_>, target: usize) -> usize {
        // The table when this body has one, and the walk when it does not --
        // an `INTERPRET` fragment is the case with none, and its instruction
        // list is short enough that the walk is what it always was.
        let base = match code.plan {
            Some(plan) => plan.indent_of(&code.body.instructions, target),
            None => static_indent(&code.body.instructions, target),
        };
        base + self.activation_indent + self.indent_offset
    }
}

/// How many spaces of nesting depth `target`'s own clause sits at --
/// Task 11's whole indentation feature, and the design decision at its
/// centre: **computed fresh from the flat instruction list every time,
/// never carried on a running `Interp` counter.**
fn fill_indents(
    instructions: &[Instruction],
    start: usize,
    end: usize,
    base: usize,
    out: &mut [usize],
) {
    let len = instructions.len();
    let mut pc = start;
    while pc < end {
        out[pc] = base;
        match &instructions[pc].kind {
            InstructionKind::If { false_target, .. } => {
                let false_target = false_target.unwrap_or(len);
                let then_start = pc + 1;
                fill_indents(
                    instructions,
                    then_start,
                    false_target.min(len),
                    base + 4,
                    out,
                );
                if then_start < len {
                    out[then_start] = base + 2;
                }
                match instructions.get(false_target).map(|i| &i.kind) {
                    Some(InstructionKind::Else { then_exit }) => {
                        let else_end = then_exit.unwrap_or(len);
                        fill_indents(
                            instructions,
                            false_target + 1,
                            else_end.min(len),
                            base + 4,
                            out,
                        );
                        out[false_target] = base + 2;
                        pc = else_end;
                    }
                    _ => pc = false_target,
                }
                continue;
            }
            InstructionKind::Do(body) | InstructionKind::Loop(body) => {
                let body_start = pc + 1;
                let end_index = body.end.expect(
                    "an unclosed DO/LOOP is error 14.1/14.5, so a body that parsed has this set",
                );
                fill_indents(instructions, body_start, end_index.min(len), base + 2, out);
                // `END`'s own position: `indent_in_range` skips past it
                // (`pc = end_index + 1`) and so answers for it from the
                // enclosing level, which is what aligns an `END` with its
                // `DO`.
                if end_index < len {
                    out[end_index] = base;
                }
                pc = end_index + 1;
                continue;
            }
            InstructionKind::Select {
                whens,
                otherwise,
                end: select_end,
                ..
            } => {
                let select_end = select_end.unwrap_or(len);
                // The arm's own fallback, written first so the shapes below
                // overwrite it: a position inside a `SELECT` that matches
                // none of them keeps the enclosing level rather than
                // asserting anything about how it got there.
                for slot in out.iter_mut().take(select_end.min(len)).skip(pc + 1) {
                    *slot = base;
                }
                if let Some(otherwise_index) = otherwise {
                    fill_indents(
                        instructions,
                        otherwise_index + 1,
                        select_end.min(len),
                        base + 4,
                        out,
                    );
                    out[*otherwise_index] = base + 2;
                }
                for &when_index in whens.iter().rev() {
                    let (body_start, body_end) = match &instructions[when_index].kind {
                        InstructionKind::When { false_target, .. }
                        | InstructionKind::WhenCase { false_target, .. } => {
                            (when_index + 1, false_target.unwrap_or(len))
                        }
                        _ => continue,
                    };
                    fill_indents(instructions, body_start, body_end.min(len), base + 6, out);
                    if body_start < len {
                        out[body_start] = base + 4;
                    }
                    out[when_index] = base + 2;
                }
                pc = select_end;
                continue;
            }
            _ => {}
        }
        pc += 1;
    }
}

/// Every position's static clause indent, in one walk.
pub(crate) fn all_indents(instructions: &[Instruction]) -> Box<[usize]> {
    let mut out = vec![0usize; instructions.len()];
    fill_indents(instructions, 0, instructions.len(), 0, &mut out);
    out.into_boxed_slice()
}

pub(crate) fn static_indent(instructions: &[Instruction], target: usize) -> usize {
    indent_in_range(instructions, 0, instructions.len(), target)
}

/// `static_indent`'s own recursive worker, over one `[start, end)` range --
/// the same range shape `run_bounded` itself runs, so this function's
/// dispatch on `If`/`Select`/`Do`/`Loop` mirrors `step`'s own arms for them,
/// reading the identical fields, just never evaluating anything.
fn indent_in_range(instructions: &[Instruction], start: usize, end: usize, target: usize) -> usize {
    let len = instructions.len();
    let mut pc = start;
    while pc < end {
        if pc == target {
            // `target` is this range's own instruction at this position --
            // a plain clause, or a block-opener's own clause (a `DO`'s
            // control-setup expressions, a `SELECT`'s own `CASE` scrutinee),
            // with nothing further to add beyond whatever the caller already
            // contributed before recursing in here.
            return 0;
        }
        match &instructions[pc].kind {
            InstructionKind::If { false_target, .. } => {
                let false_target = false_target.unwrap_or(len);
                let then_start = pc + 1;
                // `then_start` is the `Then` marker's *own* index, not its
                // body's first instruction -- measured against the oracle
                // (`ThenInstruction.cpp`'s `execute`: `indent(); trace;
                // indent();`), a marker clause sits at exactly two spaces,
                // half of what its own body gets (four). Before this check
                // existed, `target == then_start` fell into the body branch
                // below and got the wrong answer (4, not 2) because that
                // branch's own recursive call happens to return 0 for the
                // very first position of its range -- silently, since
                // nothing before this task ever asked for a `Then`'s own
                // indent (a marker clause carries no expression, so it can
                // never be a `FailureSite`, only ever a `TRACE` echo).
                if target == then_start {
                    return 2;
                }
                if target > then_start && target < false_target {
                    return 4 + indent_in_range(instructions, then_start, false_target, target);
                }
                match instructions.get(false_target).map(|i| &i.kind) {
                    Some(InstructionKind::Else { then_exit }) => {
                        let else_end = then_exit.unwrap_or(len);
                        // Same shape as `Then`, and the same measurement
                        // (`ElseInstruction.cpp`'s `execute` is byte-for-byte
                        // the same two-`indent()`-calls dance). Before this
                        // check, `target == false_target` fell all the way
                        // through this whole arm (the body check below is
                        // strict `>`) to `pc = else_end; continue`, which
                        // advances `pc` *past* `target` in the enclosing
                        // walk -- the `Else` marker's own index was never
                        // revisited by anything, silently returning
                        // whatever the *enclosing* level happened to be
                        // (0 too shallow) rather than erroring.
                        if target == false_target {
                            return 2;
                        }
                        if target > false_target && target < else_end {
                            return 4 + indent_in_range(
                                instructions,
                                false_target + 1,
                                else_end,
                                target,
                            );
                        }
                        pc = else_end;
                    }
                    _ => pc = false_target,
                }
                continue;
            }
            InstructionKind::Do(body) | InstructionKind::Loop(body) => {
                let body_start = pc + 1;
                let end_index = body.end.expect(
                    "an unclosed DO/LOOP is error 14.1/14.5, so a body that parsed has this set",
                );
                if target > pc && target < end_index {
                    return 2 + indent_in_range(instructions, body_start, end_index, target);
                }
                pc = end_index + 1;
                continue;
            }
            InstructionKind::Select {
                whens,
                otherwise,
                end: select_end,
                ..
            } => {
                let select_end = select_end.unwrap_or(len);
                if target > pc && target < select_end {
                    // Inside this SELECT's own scan-through-dispatch range:
                    // two spaces on their own (measured: a WHEN's own
                    // condition, `target == when_index` below, sits at
                    // exactly this level), plus whichever branch's own
                    // extra applies.
                    for &when_index in whens {
                        if when_index == target {
                            return 2;
                        }
                        let (body_start, body_end) = match &instructions[when_index].kind {
                            InstructionKind::When { false_target, .. }
                            | InstructionKind::WhenCase { false_target, .. } => {
                                (when_index + 1, false_target.unwrap_or(len))
                            }
                            // This asserted unreachability too, on a claim
                            // that turned out to be no better-founded than
                            // the `OTHERWISE` one three lines of history
                            // below: "`whens` holds only `When`/`WhenCase`"
                            // is `rexx-parse`'s own invariant, not this
                            // function's, and this phase's invariants have
                            // not all held -- the absorbed-`WHEN` case
                            // (`when_absorbing_a_when_parses_and_runs_at_
                            // rc_0`, `run`'s own test) is exactly a
                            // `When` instruction executing while its
                            // enclosing `SELECT`'s own `whens` does not list
                            // it, which is the same shape of surprise. If a
                            // reader ever sees this, `whens` names an index
                            // whose own kind is not what built it -- a
                            // `rexx-parse` defect, not a formatting one, and
                            // nothing this function can correct. Skipping
                            // the entry (matching the outer fallback's own
                            // "nothing further to add" answer once the loop
                            // and the `OTHERWISE` check both come up empty)
                            // keeps the diagnostic path alive instead of
                            // trading a wrong indent for a dead process.
                            _ => continue,
                        };
                        // `body_start` is the WHEN's own `Then` marker,
                        // sharing `InstructionKind::Then` with `IF` (both go
                        // through `instruction.rs`'s `if_instruction`) --
                        // measured against the oracle exactly like `IF`'s
                        // own: the marker sits at half its body's indent
                        // (four, not six). Before this check, `target ==
                        // body_start` matched the `>=` below and returned
                        // six, the body's own value -- again invisible
                        // before `TRACE`, since a `Then` marker never raises.
                        if target == body_start {
                            return 4;
                        }
                        if target > body_start && target < body_end {
                            return 6 + indent_in_range(instructions, body_start, body_end, target);
                        }
                    }
                    if let Some(otherwise_index) = otherwise {
                        // `OTHERWISE` traces its own clause once (no double
                        // `indent()` -- `OtherwiseInstruction.cpp`'s
                        // `execute` is `trace; indent();`, not `indent();
                        // trace; indent();`) at the SELECT's own scan level,
                        // the same two spaces a `WHEN`'s condition gets, and
                        // only its body gets the further two. Before this
                        // check, `target == *otherwise_index` matched
                        // neither this arm's `>` check nor anything in the
                        // `whens` loop, and fell all the way to the
                        // `unreachable!` below -- **a live panic**, not
                        // merely a wrong number, confirmed by directly
                        // calling `static_indent` on `select\nwhen 1 = 0
                        // then nop\notherwise\nsay 'y'\nend`'s own
                        // `otherwise_index` before this fix existed.
                        if target == *otherwise_index {
                            return 2;
                        }
                        if target > *otherwise_index && target < select_end {
                            return 4 + indent_in_range(
                                instructions,
                                otherwise_index + 1,
                                select_end,
                                target,
                            );
                        }
                    }
                    // `target` is in range but matches none of the above.
                    // After the two equality cases just added, every
                    // reachable position inside a resolved SELECT is
                    // provably one of: a WHEN's own index, a WHEN's own
                    // `Then` marker, one WHEN's own body, `OTHERWISE`'s own
                    // marker, or `OTHERWISE`'s own body -- so a body that
                    // parsed should never reach here. It reached here once
                    // already, though (the `OTHERWISE`-marker case, before
                    // its equality check existed), and this exact arm is
                    // where that panic actually happened -- `unreachable!`
                    // asserted a claim about the code's own shape, and the
                    // claim was false for a case nothing had exercised yet.
                    // This crate's rule for the diagnostic path (`error.rs`'s
                    // message-catalogue miss renders a visible marker
                    // instead of aborting; `clause_site`'s own fallback, in
                    // `run.rs`, cites the identical reasoning) is that a
                    // formatting gap must never become a crash, and
                    // `static_indent` feeds both the error
                    // report and `TRACE` now -- so this returns the
                    // enclosing level (0 relative, "nothing further to add")
                    // rather than asserting unreachability a second time.
                    // If a reader ever sees indentation that looks too
                    // shallow by exactly the amount a `SELECT` construct
                    // should have contributed, this is where to look: it
                    // means some future `SELECT`-shaped clause position is
                    // not one of the five cases enumerated above.
                    return 0;
                }
                pc = select_end;
                continue;
            }
            _ => {}
        }
        pc += 1;
    }
    0
}
