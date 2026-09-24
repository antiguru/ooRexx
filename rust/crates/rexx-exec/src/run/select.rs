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

//! `SELECT`'s own clauses, and where `IF`, `WHEN` and `SELECT` send control.

use super::{
    Code, ConditionTrace, Expr, Failure, Flow, Instruction, InstructionKind, Interp, LeaveOrigin,
    ObjRef, SymbolId, raised_iterate_wrong_kind, raised_when_not_logical, static_indent,
};

impl Interp {
    /// A `SELECT CASE`'s own `CASE` expression: the whole of what the
    /// `SELECT` header clause does, and the value every `WHEN CASE` of that
    /// `SELECT` is compared against.
    pub(crate) fn select_case(
        &mut self,
        code: &Code<'_>,
        case_expr: &Expr,
    ) -> Result<ObjRef, Failure> {
        // The clause unit set this to the `SELECT`'s own printed indent on the
        // way in.
        let indent = self.clause_state.current_value_indent;
        let value = self.eval(code, case_expr)?;
        self.roots.push_temp(value);
        let text = self.to_text(value).to_vec();
        // `>K>` (`SelectInstruction.cpp:372`, `traceKeywordResult(CASE, ...)`),
        // at the `SELECT`'s own level -- measured, `>K>   "CASE" => "2"` sits
        // at the same indent as `select case ...` itself, not the `WHEN`-scan
        // level a `WhenCase`'s own comparison lines are indented to.
        self.trace_keyword(indent, "CASE", &text);
        // **The block opens after this**, so the header clause's own boundary
        // runs one level in -- `newBlockInstruction` is the next thing
        // `RexxInstructionSelectCase::execute` does once the scrutinee has
        // been evaluated and its `>K>` traced, and a `CALL ON` handler
        // delivered at that boundary is based on whatever the counter reads
        // then. Measured: the handler `h:` of a `select case raiser()` at top
        // level echoes at `12 *-*     h:` on the oracle, where the `SELECT`
        // clause itself echoes unindented.
        self.settle_block_indent(true, indent);
        Ok(value)
    }

    /// Hands a `SELECT` its case text: the value its `WHEN CASE`s compare
    /// against, and `Interp::current_case_text` for the **absorbed** ones that
    /// have no other way to reach it (`lib.rs`'s own doc comment on the
    /// field).
    pub(crate) fn open_select_case(&mut self, value: Option<ObjRef>) -> Option<Vec<u8>> {
        let text = value.map(|value| self.to_text(value).to_vec());
        self.current_case_text = text.clone();
        text
    }

    /// One listed `WHEN`/`WHEN CASE`'s own condition, and whether it holds.
    pub(crate) fn scan_when(
        &mut self,
        code: &Code<'_>,
        when_instruction: &Instruction,
        case_text: Option<&[u8]>,
    ) -> Result<bool, Failure> {
        // The clause unit set this to this `WHEN`'s own printed indent on the
        // way in, which is what its condition's `>>>` lines trace at.
        let indent = self.clause_state.current_value_indent;
        match &when_instruction.kind {
            InstructionKind::When { condition, .. } => self.eval_condition(
                code,
                condition,
                ConditionTrace::Result(indent),
                raised_when_not_logical,
            ),
            InstructionKind::WhenCase { values, .. } => match case_text {
                Some(case_text) => self.test_case_when(code, values, case_text, indent),
                // A listed `WhenCase` with no `case` expression: a plain
                // `SELECT` with no `CASE` at all, which the parser should
                // never produce for a `WhenCase` node (only `SELECT CASE`
                // ever builds one, `ast.rs`'s own doc comment) -- F-EX3, the
                // same unproven parser invariant the absorbed `WhenCase` arm
                // already refuses to crash on, and formerly an `.expect()`
                // here that did. Evaluates `values` for side effects and
                // never matches, the identical fallback.
                None => {
                    for value in values {
                        let v = self.eval(code, value)?;
                        self.roots.push_temp(v);
                    }
                    Ok(false)
                }
            },
            other => panic!("a SELECT's whens holds only When/WhenCase, not {other:?}"),
        }
    }

    /// Leaving a `SELECT`'s `OTHERWISE` branch: the escape elevation is
    /// restored now that the whole dispatch -- marker and body alike -- is
    /// finished reading it, and then `leave_select` decides where control
    /// goes.
    pub(crate) fn leave_otherwise(
        &mut self,
        code: &Code<'_>,
        index: usize,
        label: Option<SymbolId>,
        otherwise_end: usize,
        end: Option<usize>,
        flow: Flow,
    ) -> Result<Flow, Failure> {
        debug_assert_eq!(
            otherwise_end,
            otherwise_range(code.body.instructions.len(), end),
            "an OTHERWISE branch was run over a range that is not its own"
        );
        self.indent_offset = 0;
        let resume = otherwise_resume(code.body.instructions.len(), end);
        self.leave_select(code, index, label, resume, flow)
    }

    /// Turns the `Flow` a `SELECT`'s own matched `WHEN` or `OTHERWISE` body
    /// produced into this `SELECT`'s own answer.
    pub(crate) fn leave_select(
        &mut self,
        code: &Code<'_>,
        index: usize,
        label: Option<SymbolId>,
        resume: SelectResume,
        flow: Flow,
    ) -> Result<Flow, Failure> {
        match flow {
            Flow::Next => Ok(Flow::Goto(resume.done)),
            Flow::Leave(Some(name), _) if label == Some(name) => Ok(Flow::Goto(resume.left)),
            Flow::Iterate(Some(name), origin) if label == Some(name) => {
                self.record_leave_failure(&origin);
                Err(raised_iterate_wrong_kind(code.symbols.name(name).as_bytes()).into())
            }
            // Not consumed: this SELECT is being "popped" by the search,
            // so its own indent becomes the new residual before the flow
            // continues outward.
            Flow::Leave(name, origin) => Ok(Flow::Leave(
                name,
                self.pop_search_frame(code, index, origin),
            )),
            Flow::Iterate(name, origin) => Ok(Flow::Iterate(
                name,
                self.pop_search_frame(code, index, origin),
            )),
            other => Ok(other),
        }
    }

    /// Resets `origin.indent` to `index`'s own `static_indent`, for a
    /// `SELECT`/`DO`/`LOOP` that owns a search frame and is being forwarded
    /// past (not matched) -- the update `LeaveOrigin`'s own doc comment
    /// describes as "restoring the indent to the value saved when that
    /// frame was pushed." Shared by `leave_select` (always calls it, since
    /// a `SELECT` always owns a frame) and `do_body_outcome` (calls it only
    /// when the `Do`/`Loop` in question owns one, i.e. skips an unlabelled
    /// `Simple` block).
    pub(super) fn pop_search_frame(
        &self,
        code: &Code<'_>,
        index: usize,
        mut origin: Box<LeaveOrigin>,
    ) -> Box<LeaveOrigin> {
        origin.indent = static_indent(&code.body.instructions, index) + self.activation_indent;
        // `site` and `clause_line` are left alone: this resets the *indent*
        // the search reports at, and the clause line stays the
        // `LEAVE`/`ITERATE`'s own however many frames it is forwarded past --
        // measured, `iterate lab` inside an inner loop attributes the outer
        // loop's re-test to the `ITERATE`'s line, not to anything about the
        // frames in between.
        origin
    }
}

/// What [`Interp::run_bounded`]'s absorption rule says about one clause's
/// `Flow`, in a range bounded by `[start, end]`.
pub(crate) enum Absorbed {
    /// Nothing to redirect: continue after the clause that produced it.
    Advance,
    /// An in-range `Flow::Goto`: continue at this instruction.
    Resume(usize),
    /// Not this range's: hand it back to the caller unchanged.
    Escaped(Flow),
}

/// [`Absorbed`] for `flow` in the range `[start, end]`.
pub(crate) fn absorb(flow: Flow, start: usize, end: usize) -> Absorbed {
    match flow {
        Flow::Next => Absorbed::Advance,
        Flow::Goto(target) if target >= start && target <= end => Absorbed::Resume(target),
        other => Absorbed::Escaped(other),
    }
}

/// Where an `IF` sends control on each of its two paths.
pub(crate) struct IfTargets {
    /// Where control goes when the condition is false: the `ELSE` when there
    /// is one, otherwise the instruction after the `THEN` branch.
    pub(crate) false_target: usize,
    /// Where control resumes once the *true* branch has finished, which is
    /// past the `ELSE` branch when there is one and identical to
    /// `false_target` when there is not.
    pub(crate) resume: usize,
}

/// [`IfTargets`] for an `If` whose parsed `false_target` is `raw`, against the
/// body it belongs to.
pub(crate) fn if_targets(instructions: &[Instruction], raw: Option<usize>) -> IfTargets {
    // `None` is the end of this body (`InstructionKind::If`'s own doc), which
    // is one past the last instruction and exactly what an empty range there
    // needs.
    let false_target = raw.unwrap_or(instructions.len());
    IfTargets {
        false_target,
        resume: skip_else(instructions, false_target),
    }
}

/// Where a listed `WHEN` sends control once its own condition holds.
pub(crate) struct WhenTargets {
    /// One past the last instruction of this `WHEN`'s own branch: the next
    /// listed `WHEN`, the `OTHERWISE`, or the enclosing `SELECT`'s `END`.
    pub(crate) body_end: usize,
    /// Where control resumes once that branch has finished, which is past the
    /// whole `SELECT`, because one true `WHEN` ends it.
    pub(crate) resume: usize,
}

/// [`SelectResume`] for a matched listed `WHEN`: one answer twice, because one
/// true `WHEN` ends the whole `SELECT` whether its branch finished or was left
/// by name.
pub(crate) fn when_resume(targets: &WhenTargets) -> SelectResume {
    SelectResume {
        done: targets.resume,
        left: targets.resume,
    }
}

/// [`SelectResume`] for the `OTHERWISE` branch: falling off its end runs the
/// `END`, and a `LEAVE` naming this `SELECT` resumes past it.
pub(crate) fn otherwise_resume(len: usize, end: Option<usize>) -> SelectResume {
    SelectResume {
        done: otherwise_range(len, end),
        left: select_exit(len, end),
    }
}

/// [`WhenTargets`] for a listed `When`/`WhenCase` node, against the body it
/// belongs to.
pub(crate) fn when_targets(kind: &InstructionKind, len: usize) -> WhenTargets {
    match kind {
        InstructionKind::When {
            false_target, exit, ..
        }
        | InstructionKind::WhenCase {
            false_target, exit, ..
        } => WhenTargets {
            body_end: false_target.unwrap_or(len),
            resume: exit.unwrap_or(len),
        },
        other => panic!("a SELECT's whens holds only When/WhenCase, not {other:?}"),
    }
}

/// What a `SELECT` node tells whoever is running one of its branches.
pub(crate) struct SelectParts {
    /// `SELECT LABEL name`'s own label, which a `LEAVE`/`ITERATE` may name.
    pub(crate) label: Option<SymbolId>,
    /// This `SELECT`'s own `OTHERWISE` marker, if it has one.
    pub(crate) otherwise: Option<usize>,
    /// The `END` that closes it.
    pub(crate) end: Option<usize>,
}

/// [`SelectParts`] for a `Select` node, and `None` for anything else.
pub(crate) fn select_parts(kind: &InstructionKind) -> Option<SelectParts> {
    match kind {
        InstructionKind::Select {
            label,
            otherwise,
            end,
            ..
        } => Some(SelectParts {
            label: *label,
            otherwise: *otherwise,
            end: *end,
        }),
        _ => None,
    }
}

/// One past the last instruction of a `SELECT`'s `OTHERWISE` branch, which is
/// also where control resumes once that branch has finished: its own `END`,
/// where the `EndStyle::Otherwise` arm does nothing.
pub(crate) fn otherwise_range(len: usize, end: Option<usize>) -> usize {
    end.unwrap_or(len)
}

/// Where a `LEAVE` naming a `SELECT` resumes: past the `END` that closes it.
pub(crate) fn select_exit(len: usize, end: Option<usize>) -> usize {
    match end {
        Some(end) => (end + 1).min(len),
        None => len,
    }
}

/// Where a `SELECT` sends control when one of its branches is over, which is
/// **two answers and not one**.
#[derive(Clone, Copy)]
pub(crate) struct SelectResume {
    /// A branch that ran off its own end. `OTHERWISE`'s falls through onto the
    /// `END`, which executes and does nothing; a matched `WHEN`'s resumes past
    /// the `END`, because one true `WHEN` ends the whole `SELECT`.
    pub(crate) done: usize,
    /// A `LEAVE` naming this `SELECT`, which resumes past the `END` from
    /// **either** branch. Measured under `trace r`: `select label s` /
    /// `when 1 = 0 then nop` / `otherwise leave s` / `end` echoes no `end`
    /// clause at all, where the same `OTHERWISE` falling through echoes one.
    pub(crate) left: usize,
}

/// What a `SELECT` does with a `Flow` that escaped the branch it was running.
pub(crate) enum SelectEscape {
    /// The flow lands exactly on this `SELECT`'s own `OTHERWISE` marker, so
    /// that branch runs **with this `SELECT`'s search frame still standing**.
    Otherwise(usize),
    /// Anything else, `leave_select`'s to resolve.
    Forward(Flow),
}

/// [`SelectEscape`] for `flow` against a `SELECT` whose `OTHERWISE` is at
/// `otherwise`.
pub(crate) fn select_escape(otherwise: Option<usize>, flow: Flow) -> SelectEscape {
    match flow {
        Flow::Goto(target) if otherwise == Some(target) => SelectEscape::Otherwise(target),
        other => SelectEscape::Forward(other),
    }
}

/// Where the *true* branch resumes, given where the false one goes.
pub(super) fn skip_else(instructions: &[Instruction], target: usize) -> usize {
    match instructions.get(target).map(|i| &i.kind) {
        Some(InstructionKind::Else { then_exit }) => then_exit.unwrap_or(instructions.len()),
        _ => target,
    }
}
