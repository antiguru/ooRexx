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

//! The instruction loop: `Flow`, `step`, and the two functions that run it.

use crate::activation::{
    Activation, CallType, Entry, Inherited, InstanceVar, ReplyState, TraceEntry, Trap,
    TrappedCondition, body_of,
};
use crate::builtin;
use crate::clause::{ClauseEntry, ClauseOutcome, ClauseValue, HandlerExit};
use crate::error::{FailureSite, Raised, Search};
use crate::eval::logical_value;
use crate::ir::{BodyEngine, NodePath};
use crate::plan::BodyKey;
use crate::trace::{
    Announced, is_whole_number, mode_from_setting, raised_invalid_trace_letter,
    raised_numeric_trace_interactive_only,
};
use crate::value::{exact_small_int, within_digits};
use crate::{
    ActiveCondition, CallContext, Code, Failure, InstalledRoutine, Interp, Loud, Novalue,
    PendingTrap, VarHome,
};
use rexx_core::{BehaviourId, Body, Decoded, FrameId, ObjRef, ScopePools, SlotFrame, VarRefHome};
use rexx_num::{ArithError, CompareOp, Form, Number, SettingsError, compare_decoded};
use rexx_parse::{
    CodeBody, ConditionTrap, ControlExpr, DirectiveKind, EndStyle, Expr, ExprKind, Forward,
    Fragment, Guard, Instruction, InstructionKind, Loop, LoopConditional, LoopKind, NumericSetting,
    ProgramSource, Raise, SymbolId, Trace, Use, UseTarget, VariableRef, parse_interpret,
};
use std::borrow::Cow;
use std::rc::Rc;

/// Where control goes after one instruction (the design's "Control flow").
pub(crate) enum Flow {
    Next,
    /// Live since Task 10: `If`/`Select` each resolve to one `Goto` that
    /// skips straight to their construct's true resume point, and
    /// `run_bounded`'s own internal loop applies one whenever a nested
    /// construct's target lands inside its range.
    Goto(usize),
    Exit(Option<ObjRef>),
    /// `RETURN`, with the expression's value or `None` for the bare form.
    /// Live since Task 3.
    Return(Option<ObjRef>),
    /// `LEAVE`, bare (`None`) or by name. Live since Task 11.
    Leave(Option<SymbolId>, Box<LeaveOrigin>),
    /// `ITERATE`, bare or by name. See `Leave`'s own doc comment; the two
    /// variants are handled by nearly identical logic in `Do`/`Select`'s own
    /// arms, differing only in which of the oracle's measured asymmetries
    /// applies (`Select` never consumes a bare `Iterate` at all, and a named
    /// one that matches its own label but is not a loop is 28.5, not simply
    /// "not mine, keep looking").
    Iterate(Option<SymbolId>, Box<LeaveOrigin>),
    /// `SIGNAL label` and `SIGNAL VALUE`, once the target resolves to an
    /// instruction index.
    Signal(usize),
}

/// How one activation finished, which is not the same question as what value
/// it produced.
pub(crate) enum Ended {
    /// `RETURN`: the caller resumes at its next clause, with this value in
    /// `RESULT`.
    Returned(Option<ObjRef>),
    /// `EXIT`, **or the body running out of instructions.** The whole program
    /// stops. Falling off the end belongs here rather than with `Returned`
    /// and that is measured, not assumed: a callee whose label is the last
    /// thing in the file ends the program -- `trace r` / `call sub` / `say
    /// 'after'` / `exit` / `sub:` / `hh = 1` echoes the callee's clauses, then
    /// stops at rc 0 with `after` neither printed nor echoed.
    Exited(Option<ObjRef>),
}

impl Ended {
    /// The value, whichever way the activation finished -- what the *top*
    /// level wants, where the distinction carries no information.
    pub(crate) fn value(self) -> Option<ObjRef> {
        match self {
            Ended::Returned(value) | Ended::Exited(value) => value,
        }
    }
}

/// Who emits a stepped clause's own `*-*` line
/// ([`crate::ir::Op::Clause`]).
pub(crate) enum Echo {
    /// The clause unit asks [`Interp::tracing_clause`] and echoes if the
    /// answer is yes: a clause whose chunk was compiled under a setting that
    /// is no longer in force.
    Gated,
    /// The clause unit emits nothing: this clause's chunk already decided,
    /// and carries the decision as [`crate::ir::Op::TraceClause`] or as the
    /// absence of it.
    Compiled,
}

/// A stepped clause that is open: [`Interp::enter_stepped_clause`] makes one
/// and [`Interp::leave_stepped_clause`] spends it.
#[must_use]
pub(crate) struct SteppedClause {
    /// The clause boundary this entry opened.
    entry: ClauseEntry,
    /// The GC temps frame the clause's own work pushes into.
    frame: FrameId,
    /// `RootSet::temps_len` as the clause was opened.
    temps_at_entry: usize,
}

/// What a called name resolved to, decided in one place before any argument
/// is evaluated.
#[derive(Clone, Copy)]
pub(crate) enum Resolved {
    /// A label in the *running activation's* body, at this instruction index.
    Label(usize),
    /// A builtin function name, and **which** builtin -- a row index, not a
    /// copy of the row, so nothing here can drift from the arity check and
    /// the code that live on that row together.
    Builtin(crate::builtin::BuiltinTarget),
    /// A `::ROUTINE` this program installed. `InstalledRoutine::directive` is
    /// the same integer `Activation::body` and `BodyKey::directive` carry.
    Routine(InstalledRoutine),
    /// One of the interpreter's own embedded `.orx` sources, named by a
    /// `CALL` inside the library bootstrap -- `CoreClasses.orx:122` and
    /// `:124`.
    Library(&'static rexx_lib::Program),
    /// A routine of the `REXX` or `REXXUTIL` package that this crate has a
    /// body for. Like a builtin it runs no activation; unlike one it is not in
    /// `BuiltinFunctions.cpp`'s table, which is why its argument errors are
    /// the native-routine family.
    Internal(&'static crate::internal_routines::InternalRoutine),
}

/// Which of the two activation-pushing outcomes a resolved call took, kept
/// past the push so the decisions that follow it can read it.
#[derive(Copy, Clone)]
enum Entered {
    Label(usize),
    Routine(InstalledRoutine),
}

/// How the callee was reached: written in the program, or delivered to it.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) enum CallEntry {
    /// A `CALL`, an internal function call, or either form reaching a
    /// `::ROUTINE` -- a call the program's own text asked for.
    Written,
    /// A `CALL ON` handler, run because a condition was delivered.
    Trap,
}

/// The receiver the callee's calling convention carries (D24), given the
/// caller's own.
fn entered_receiver(entered: Entered, entry: CallEntry, caller: Option<ObjRef>) -> Option<ObjRef> {
    match (entered, entry) {
        (Entered::Label(_), CallEntry::Written) => caller,
        (Entered::Label(_), CallEntry::Trap) | (Entered::Routine(_), _) => None,
    }
}

/// How many activations may be live at once before `CALL` raises 11.1
/// ("Insufficient control stack space", `Raised::insufficient_stack`).
/// ```text
/// enclosing DO blocks per activation | deepest surviving (debug) | (release)
///                                  0 |                    22,534 |   133,150
///                                  1 |                    14,062 |    94,518
///                                  5 |                     5,616 |         -
///                                 25 |                     1,403 |         -
/// ```
pub(crate) const MAX_ACTIVATION_DEPTH: usize = 10_000;

/// The longest `ADDRESS` environment name accepted, beyond which the
/// instruction raises 29.1.
const MAX_ADDRESS_NAME_LENGTH: usize = 250;

/// Where a `LEAVE`/`ITERATE` instruction itself sits, captured the instant
/// it steps rather than reconstructed later -- see `Flow::Leave`'s own doc
/// comment for why eagerly.
pub(crate) struct LeaveOrigin {
    /// `None` only when `source` was `None` at the moment this instruction
    /// stepped, which **no caller produces**: `run_fragment` passes its own
    /// fragment source, so a `LEAVE`/`ITERATE` inside fragment text resolves a
    /// real site and becomes the report's innermost echo. See
    /// `Interp::clause_site`.
    site: Option<(usize, Vec<u8>)>,
    indent: usize,
    /// This `LEAVE`/`ITERATE` clause's own line, captured the same way and at
    /// the same moment as `site` and `indent`, and for the same reason.
    clause_line: usize,
}

/// What one pass of a `DO`/`LOOP` body just did, from `do_body_outcome`.
enum DoOutcome {
    /// The body ran off its end into the `END` clause.
    FellThrough,
    /// An `ITERATE` naming this construct cut the pass short. The line is
    /// that `ITERATE` clause's own -- what the loop's next re-test is
    /// attributed to, because the oracle re-enters the loop from inside
    /// `RexxActivation::iterate`, with the `ITERATE` still the current
    /// instruction. `site` is the same clause's echo site, for the case
    /// where that re-test fails -- see [`HeaderClause::Iterate`].
    Iterated {
        line: usize,
        site: Option<(usize, Vec<u8>)>,
    },
    /// Stop: this `Flow` is the whole construct's final answer.
    Escaped(Flow),
}

/// What a repeating `DO`/`LOOP`'s own header clause decided: run the body
/// once more, or stop.
enum HeaderOutcome {
    Continue,
    Stop,
}

impl HeaderOutcome {
    /// Whether the loop's block is still open once this header clause has
    /// finished -- what [`Interp::settle_block_indent`] takes.
    fn entered(&self) -> bool {
        matches!(self, HeaderOutcome::Continue)
    }
}

impl ClauseValue for HeaderOutcome {
    /// Nothing to root: a loop header produces a decision, never a value
    /// whose only root was this clause's own temps frame.
    fn rooted(&self) -> Option<ObjRef> {
        None
    }
}

/// Which clause a repeating `DO`/`LOOP`'s own header evaluation -- the
/// control advance, a `WHILE` test, an `UNTIL` test -- belongs to on this
/// pass.
#[derive(Clone, Copy)]
enum HeaderClause {
    /// The first pass: the `DO`/`LOOP` clause's own line.
    Do,
    /// The previous pass fell through to `END`.
    End,
    /// The previous pass ended in an `ITERATE`, whose own echo site is the
    /// [`IterateSite`] its holder carries beside this.
    Iterate {
        /// That `ITERATE` clause's own line -- `SIGL`'s quantity, which
        /// honours `clause_line_override` inside an `INTERPRET`.
        line: usize,
    },
}

/// The `(line, text)` echo site of the `ITERATE` a [`HeaderClause::Iterate`]
/// stands for, carried so that a re-test which *fails* can be blamed on it.
/// `LeaveOrigin` captured this the instant the `ITERATE` stepped; without
/// carrying it the pair is gone by the time the next header runs, and the
/// failure is misattributed to the `DO` clause (review round 1, F2).
/// **Not** `LeaveOrigin::indent`, which is that clause's own lexical one:
/// measured, an `ITERATE` nested two blocks deep inside the body echoes at
/// its own depth when it steps and at the *loop body's* depth on the failure
/// path.
type IterateSite = Option<(usize, Vec<u8>)>;

/// **SPIKE, not for commit.** One repeating loop being driven from the op
/// driver's own frame: the state `run_repeating` holds in locals, held here
/// instead because the pass loop is the driver's rather than its own.
pub(crate) struct FlatLoop {
    /// The first op of the body, where every pass starts.
    pub(crate) op_body: u32,
    /// The body's own instruction range, which an escaping `Flow` is absorbed
    /// against exactly as `run_bounded`'s own bounds absorb it today.
    pub(crate) body_start: usize,
    pub(crate) end_index: usize,
    pub(crate) do_index: usize,
    resume: usize,
    label: Option<SymbolId>,
    do_indent: usize,
    loop_indent: usize,
    do_line: usize,
    end_line: usize,
    header_clause: HeaderClause,
    /// The echo site `header_clause` names when it is `Iterate`.
    iterate_site: IterateSite,
    /// `Some(true)` for `UNTIL`, `Some(false)` for `WHILE`, `None` for a loop
    /// with neither. **The condition's own node is not held here**, because
    /// this outlives the borrow of `code` a reference to it would need; the
    /// node is read back off the `DO` instruction at the one point per pass
    /// that tests it, and this says whether that read is owed at all.
    conditional: Option<bool>,
    state: LoopState,
}

impl FlatLoop {
    /// A `FlatLoop` naming nothing, which exists only to give
    /// [`Interp::flat_loop_start`] a box to write a real one into when the
    /// spare pool is empty. Every field is overwritten before anything reads
    /// one.
    const fn vacant() -> FlatLoop {
        FlatLoop {
            op_body: 0,
            body_start: 0,
            end_index: 0,
            do_index: 0,
            resume: 0,
            label: None,
            do_indent: 0,
            loop_indent: 0,
            do_line: 0,
            end_line: 0,
            header_clause: HeaderClause::Do,
            iterate_site: None,
            conditional: None,
            state: LoopState::Forever,
        }
    }

    /// Which clause a header or `UNTIL` test on this pass belongs to.
    fn header_line(&self) -> usize {
        match self.header_clause {
            HeaderClause::Do => self.do_line,
            HeaderClause::End => self.end_line,
            HeaderClause::Iterate { line } => line,
        }
    }
}

/// **SPIKE.** What a header clause's own answer means: `Some(flow)` is the
/// loop finishing, `None` is one more pass.
fn flat_header_outcome(
    header: ClauseOutcome<bool>,
    resume: usize,
) -> Result<Option<Flow>, Failure> {
    match header {
        ClauseOutcome::Ended(exit) => Ok(Some(Flow::Exit(exit.value()))),
        ClauseOutcome::Ran(Err(failure)) => Err(failure),
        ClauseOutcome::Ran(Ok(false)) => Ok(Some(Flow::Goto(resume))),
        ClauseOutcome::Ran(Ok(true)) => Ok(None),
    }
}

/// **SPIKE.** The `WHILE`/`UNTIL` of the `DO`/`LOOP` at `index`, read back off
/// the instruction because a [`FlatLoop`] outlives any borrow of `code`.
fn loop_conditional_of<'a>(code: &'a Code<'_>, index: usize) -> Option<&'a LoopConditional> {
    match &code.body.instructions.get(index)?.kind {
        InstructionKind::Do(body) | InstructionKind::Loop(body) => body.conditional.as_ref(),
        _ => None,
    }
}

/// **SPIKE.** What `Interp::flat_loop_start` decided.
pub(crate) enum FlatStart {
    /// Driven from the driver's frame. The state is on `Interp::flat_loops`;
    /// this is the body's own instruction range, which the driver's frame
    /// absorbs an escaping `Flow` against.
    Flat { body_start: usize, end_index: usize },
    /// The header said zero passes, so the construct is already over.
    Ended(Flow),
    /// Not a shape this spike drives: take the nested path, with the header
    /// values handed back so that the nested path can move them rather than
    /// this one copying them.
    Fallback(LoopHeaderValues),
}

/// **SPIKE.** What one pass boundary decided.
pub(crate) enum FlatStep {
    /// One more pass, from this op.
    Body(u32),
    /// The construct is over and this is its answer.
    Done(Flow),
}

/// What drives one repeating `DO`/`LOOP`'s own iteration, once its header
/// has already been evaluated and validated -- everything `LoopKind` can be
/// except `Simple` (a block, never repeats, and `run_loop_with_header`'s own
/// `Simple` arm never builds one of these at all) and `With` (the loud path).
enum LoopState {
    Forever,
    /// `DO expr`: a fixed repeat count, decremented to zero.
    Count {
        remaining: u64,
    },
    /// `DO name OVER expr`, a **non-stem** target only (Deviation 1: a stem
    /// target takes the loud path in `run_loop_with_header` before one of
    /// these is ever built): binds `control` to each of `items` in turn.
    OverItems {
        control: SymbolId,
        /// [`control_slot`], taken once when this loop was entered.
        at: Option<usize>,
        snapshot: ObjRef,
        items: Vec<ObjRef>,
        next: usize,
        remaining: Option<u64>,
    },
    /// `DO i = initial TO to BY by FOR for_count`. `to`/`for_remaining` are
    /// `None` when that keyword was not written at all (an absent `TO`
    /// loops until `LEAVE` or `FOR` stops it, exactly like `FOREVER` with a
    /// control variable riding along); `by` is never absent here --
    /// `setup_controlled` already defaulted it to `1`.
    Controlled {
        control: SymbolId,
        /// [`control_slot`], taken once when this loop was entered.
        at: Option<usize>,
        current: ControlValue,
        to: Option<Number>,
        by: Number,
        for_remaining: Option<u64>,
        /// `TO`/`BY` as plain integers, and the precision they hold for.
        cached_digits: u64,
        to_int: Option<i64>,
        by_int: Option<i64>,
        /// Whether the control variable is spelled simple, stem or compound.
        shape: NameShape,
        /// Whether at least one candidate iteration has already been
        /// decided, which is exactly the oracle's own `!first` argument to
        /// `DoBlock::checkControl` (`ControlledDoInstruction.cpp:162`): it
        /// selects between "read the value the header computed" and
        /// "increment it, tracing on both sides of the addition". A `bool`
        /// on the state rather than a flag threaded through `run_repeating`
        /// because `loop_advance` is the only reader and the only writer,
        /// and because it has to survive an `ITERATE`, which re-enters that
        /// function without passing through the top of the driver's loop.
        stepped: bool,
    },
}

/// A controlled loop's running control value.
enum ControlValue {
    Small(i64),
    Wide(Number),
}

impl ControlValue {
    /// The value as a `Number`, borrowed when it already is one.
    fn number(&self) -> Cow<'_, Number> {
        match self {
            ControlValue::Small(value) => Cow::Owned(Number::from_i64(*value)),
            ControlValue::Wide(number) => Cow::Borrowed(number),
        }
    }

    /// The value as an integer the bound test may compare exactly, or `None`
    /// when the fuzzed comparison has to run instead. The caller supplies the
    /// other half of the interpreter's condition, that `NUMERIC FUZZ` is zero.
    fn small(&self, digits: u64) -> Option<i64> {
        match self {
            ControlValue::Small(value) => within_digits(*value, digits).then_some(*value),
            ControlValue::Wide(_) => None,
        }
    }
}

/// What one expression of a `DO`/`LOOP` header is for.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum HeaderRole {
    /// A controlled loop's own starting value. Echoed under no tag at all:
    /// measured, `trace i` over `do ii = 1 to 2` shows the initial value's
    /// `>L>` line and then `>K>   "TO"`, with no `>K>` of its own.
    Initial,
    To,
    By,
    /// A controlled loop's `FOR`.
    For,
    /// A bare `DO expr`'s repeat count, **echoed under the `FOR` tag**
    /// (measured: the oracle traces a bare repeat count under `FOR`, the
    /// same as an explicit `DO ... FOR n`), and validated against 26.2 where
    /// a `FOR` is 26.3.
    Count,
    /// `DO name OVER expr`'s target, echoed under the `OVER` tag.
    Over,
    /// `DO name OVER expr FOR expr`'s count, **echoed under the `FOR` tag**
    /// (measured against the oracle: `do qq over zs for 1` prints
    /// `>K>   "FOR" => "1"`, on both `trace i` and `trace r` -- the same tag
    /// a controlled loop's own `FOR` and a bare `DO`'s repeat count carry).
    OverFor,
}

impl HeaderRole {
    /// The `>K>` tag this value's own echo carries, or `None` for the one
    /// role the oracle echoes nothing for: `Initial`, a control variable's
    /// own starting value (its own doc has the measurement).
    pub(crate) fn keyword(self) -> Option<&'static str> {
        match self {
            HeaderRole::Initial => None,
            HeaderRole::To => Some("TO"),
            HeaderRole::By => Some("BY"),
            HeaderRole::For | HeaderRole::Count | HeaderRole::OverFor => Some("FOR"),
            HeaderRole::Over => Some("OVER"),
        }
    }

    /// How a loud failure names the position this value sits in.
    pub(crate) fn value_name(self) -> &'static str {
        match self {
            HeaderRole::Initial => "a DO header's initial value",
            HeaderRole::To => "a DO header's TO value",
            HeaderRole::By => "a DO header's BY value",
            HeaderRole::For | HeaderRole::OverFor => "a DO header's FOR value",
            HeaderRole::Count => "a DO header's repeat count",
            HeaderRole::Over => "a DO header's OVER target",
        }
    }
}

/// The header expressions of one `DO`/`LOOP`, in **the order they are
/// evaluated**, which is the order they were written in
/// (`Controlled::order`, recorded because an expression can have side
/// effects).
pub(crate) struct HeaderPlan {
    roles: [HeaderRole; 4],
    len: usize,
}

impl HeaderPlan {
    fn new() -> HeaderPlan {
        HeaderPlan {
            roles: [HeaderRole::Initial; 4],
            len: 0,
        }
    }

    fn push(&mut self, role: HeaderRole) {
        debug_assert!(
            self.len < self.roles.len(),
            "a DO/LOOP header has more expressions than TO, BY, FOR and one control value"
        );
        self.roles[self.len] = role;
        self.len += 1;
    }

    /// The roles in evaluation order.
    pub(crate) fn roles(&self) -> &[HeaderRole] {
        &self.roles[..self.len]
    }
}

/// The header of `body`, or `None` for a `DO`/`LOOP` this crate refuses
/// **before evaluating anything**.
pub(crate) fn loop_header_plan(body: &Loop) -> Option<HeaderPlan> {
    if body.counter.is_some() {
        return None;
    }
    let mut plan = HeaderPlan::new();
    match &body.kind {
        // A block and a `FOREVER` loop each have no header expression at all.
        LoopKind::Simple | LoopKind::Forever => {}
        // `count_loop`'s own parser always calls `opt_expr`, which can answer
        // `None`; nothing in this crate's tests reaches `DO` with truly
        // nothing after it and no recognised keyword either, because
        // `create_loop`'s own `at_end()` check catches a bare `DO` first and
        // builds `LoopKind::Simple` instead.
        LoopKind::Count(None) => {}
        LoopKind::Count(Some(_)) => plan.push(HeaderRole::Count),
        LoopKind::Controlled(ctrl) => {
            plan.push(HeaderRole::Initial);
            for entry in &ctrl.order {
                plan.push(match entry {
                    ControlExpr::To => HeaderRole::To,
                    ControlExpr::By => HeaderRole::By,
                    ControlExpr::For => HeaderRole::For,
                });
            }
        }
        LoopKind::Over {
            target, for_count, ..
        } => {
            if matches!(target.kind, ExprKind::Stem(_)) {
                return None;
            }
            plan.push(HeaderRole::Over);
            if for_count.is_some() {
                plan.push(HeaderRole::OverFor);
            }
        }
        LoopKind::With { .. } => return None,
    }
    Some(plan)
}

/// The expression `role` names in `kind`, or `None` when that kind has no
/// expression for it.
fn header_expr_for(kind: &LoopKind, role: HeaderRole) -> Option<&Expr> {
    match (kind, role) {
        (LoopKind::Count(expr), HeaderRole::Count) => expr.as_ref(),
        (LoopKind::Controlled(ctrl), HeaderRole::Initial) => Some(&ctrl.initial),
        (LoopKind::Controlled(ctrl), HeaderRole::To) => ctrl.to.as_ref(),
        (LoopKind::Controlled(ctrl), HeaderRole::By) => ctrl.by.as_ref(),
        (LoopKind::Controlled(ctrl), HeaderRole::For) => ctrl.for_count.as_ref(),
        (LoopKind::Over { target, .. }, HeaderRole::Over) => Some(target),
        (LoopKind::Over { for_count, .. }, HeaderRole::OverFor) => for_count.as_ref(),
        _ => None,
    }
}

/// The expression of `body`'s header at `slot` -- the compiled stream's own
/// addressing, where a slot is a position in [`HeaderPlan::roles`].
pub(crate) fn loop_header_slot(body: &Loop, slot: u32) -> Option<&Expr> {
    let plan = loop_header_plan(body)?;
    let role = *plan.roles().get(slot as usize)?;
    header_expr_for(&body.kind, role)
}

/// One `DO`/`LOOP` header's evaluated and validated values.
#[derive(Default)]
pub(crate) struct LoopHeaderValues {
    /// A controlled loop's starting value, rounded at the digits in force,
    /// **in the representation the header's own value was in** -- an integer
    /// stays one, so the first pass's bound test can be the exact comparison
    /// the interpreter makes for two integer objects.
    initial: Option<ControlValue>,
    to: Option<Number>,
    by: Option<Number>,
    /// A `FOR`'s own budget, from either a controlled loop's `FOR` or a
    /// `DO OVER`'s.
    for_remaining: Option<u64>,
    /// A `DO OVER`'s target value.
    over: Option<ObjRef>,
    /// The register `over` was evaluated into, `None` off the compiled engine
    /// and for every loop that is not a `DO OVER`. [`Interp::flat_loop_start`]
    /// writes the snapshot back into it; see [`LoopState::OverItems`].
    pub(crate) over_register: Option<u16>,
    /// A bare `DO expr`'s repeat count.
    count: Option<u64>,
}

/// What `eval_condition` should do with the value it just computed, beyond
/// answering the caller's `bool` -- a caller-chosen variant rather than a
/// decision `eval_condition` makes on its own, because the same function
/// serves `IF`/`WHEN` (their own `>>>`, measured) and `WHILE`/`UNTIL`
/// (their own `>K>` instead, never a bare `>>>` alongside it, also
/// measured) and the two are genuinely different oracle behaviours, not
/// two spellings of one.
pub(crate) enum ConditionTrace<'a> {
    /// `IF`/`WHEN`'s own `>>>`.
    Result(usize),
    /// `WHILE`/`UNTIL`'s own `>K>`, tagged `"WHILE"`/`"UNTIL"`.
    Keyword(usize, &'a str),
}

/// Which keyword ended the activation, for [`Interp::returned_value`].
#[derive(Clone, Copy)]
enum Conversion {
    /// An array answers itself.
    Array,
    /// `StringUtil::makearray` over the value's own text.
    Lines,
    /// `StemClass::tailArray`.
    Tails,
    /// `TheNilObject`, which `FORWARD` reports as 98.946.
    Refused,
    /// A conversion this crate does not build. The string is the noun
    /// [`Loud::object_position`] puts in the refusal.
    NotBuilt(&'static str),
}

/// `StringUtil::makearray` with the default separator
/// (`classes/support/StringUtil.cpp:545`-`:638`): a piece per line end, one
/// `\r` dropped from a piece that ends in one, and a trailing piece only
/// where the text does not end at a separator.
fn makearray_lines(text: &[u8]) -> Vec<&[u8]> {
    let mut pieces = Vec::new();
    let mut start = 0;
    while let Some(offset) = text[start..].iter().position(|byte| *byte == b'\n') {
        let separator = start + offset;
        let mut end = separator;
        if end > start && text[end - 1] == b'\r' {
            end -= 1;
        }
        pieces.push(&text[start..end]);
        start = separator + 1;
    }
    if start < text.len() {
        pieces.push(&text[start..]);
    }
    pieces
}

#[derive(Clone, Copy)]
pub(crate) enum ReturnKeyword {
    Return,
    Exit,
}

/// Which end of the queue a line lands on, for [`Interp::queue_evaluated`].
#[derive(Clone, Copy)]
pub(crate) enum QueueKeyword {
    Push,
    Queue,
}

impl Interp {
    // ---- the instruction loop, which is what this spike is for ----

    /// Runs the current activation's body to completion.
    /// ```text
    /// fn run_activation_wrong(&mut self) -> Result<Option<ObjRef>, Loud> {
    ///     let body = &self.activations.last().expect("a live activation").program.main;
    ///     while let Some(instruction) = body.instructions.get(self.activation().pc) {
    ///         self.step_wrong(body, instruction)?;
    ///     }
    ///     Ok(None)
    /// }
    /// ```
    /// ```text
    /// error[E0502]: cannot borrow `*self` as mutable because it is also borrowed as immutable
    ///    --> crates/rexx-exec/src/lib.rs:851:13
    ///     |
    /// 849 |         let body = &self.activations.last().expect("a live activation").program.main;
    ///     |                     ---------------- immutable borrow occurs here
    /// 850 |         while let Some(instruction) = body.instructions.get(self.activation().pc) {
    ///     |                                       ----------------- immutable borrow later used here
    /// 851 |             self.step_wrong(body, instruction)?;
    ///     |             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ mutable borrow occurs here
    /// ```
    /// ```
    /// use std::rc::Rc;
    /// struct CodeBody { instructions: Vec<u32> }
    /// struct Frame { program: Rc<CodeBody>, pc: usize }
    /// struct Interp { activations: Vec<Frame> }
    /// impl Interp {
    ///     fn step(&mut self, _instruction: &u32) {}
    ///     fn run(&mut self) {
    ///         let program = Rc::clone(&self.activations.last().unwrap().program);
    ///         let body = &program.instructions;
    ///         while let Some(instruction) = body.get(self.activations.last().unwrap().pc) {
    ///             self.step(instruction);
    ///             self.activations.last_mut().unwrap().pc += 1;
    ///         }
    ///     }
    /// }
    /// ```
    /// ```compile_fail
    /// use std::rc::Rc;
    /// struct CodeBody { instructions: Vec<u32> }
    /// struct Frame { program: Rc<CodeBody>, pc: usize }
    /// struct Interp { activations: Vec<Frame> }
    /// impl Interp {
    ///     fn step(&mut self, _instruction: &u32) {}
    ///     fn run(&mut self) {
    ///         let body = &self.activations.last().unwrap().program.instructions;
    ///         while let Some(instruction) = body.get(self.activations.last().unwrap().pc) {
    ///             self.step(instruction);
    ///             self.activations.last_mut().unwrap().pc += 1;
    ///         }
    ///     }
    /// }
    /// ```
    pub(crate) fn run_activation(&mut self) -> Result<Ended, Failure> {
        // `code` is bound to the activation on top of the stack at entry,
        // while every `pc` read and write below goes to whatever is on top
        // *now*. Those are the same frame only because `step` leaves the
        // activation stack as it found it -- true for a fragment, which runs
        // inside the creating activation rather than pushing its own, and
        // true for a `CALL` only because the `Call` arm pops the callee
        // before it returns.
        let arguments = Rc::clone(&self.call_context.arguments);
        // **The name is copied only where nothing else records it.** A
        // method activation already carries the message name it was entered
        // under on `Activation::method_identity`, and that is the hot path:
        // measured, `instructions:u` on `bench-programs/dispatch.rex`, one
        // no-argument send per iteration, +1.732% against BASE with an
        // `Rc<[u8]>` built here for every send and +0.093% with this test in
        // front of it.
        let name = (self.activation().entry != crate::activation::Entry::Method)
            .then(|| Rc::from(&self.call_context.name[..]));
        let activation = self.activation_mut();
        if let Some(name) = name {
            activation.call_name = Some(name);
        }
        activation.call_arguments = Some(arguments);
        let program = Rc::clone(&self.activation().program);
        let plan = Rc::clone(&self.activation().plan);
        let selector = self.activation().body;
        // A selector that resolves to nothing is an internal inconsistency
        // and not a program error: it can only be built by a resolution step
        // that already looked the body up. Loud rather than a panic, matching
        // this crate's standing rule -- an abort is precisely the outcome
        // that rule exists to exclude.
        let Some(body) = body_of(&program, selector) else {
            return Err(Loud::missing_body().into());
        };
        let code = Code {
            body,
            symbols: &program.symbols,
            slots: &plan.by_symbol,
            plan: Some(&plan),
        };
        // The key this body's plan was cached under, and the key its chunk is
        // cached under. **They have to be the same key**, and nothing but this
        // says so: `plan` is read off the activation while `key` is rebuilt
        // from the activation's program id and body selector, so a push that
        // paired a plan with the wrong id would put a body's chunk under
        // another body's name -- a wrong answer rather than a miss, exactly
        // what `BodyKey::directive`'s own doc says about the selector.
        let key = self.activation().body_key();
        debug_assert!(
            self.plans
                .get(&key)
                .is_some_and(|cached| Rc::ptr_eq(cached, &plan)),
            "the running activation's body key does not name the plan it is running with, so \
             its chunk would be cached under another body's name"
        );

        // **Every activation's body runs from a compiled chunk.** There is
        // no second engine and no selection left to make: `run_activation` is
        // the one function that runs a body, and this is where the stream is
        // entered.
        let Some(chunk) = self.chunk_for(key, self.chunk_trace(), body, &plan) else {
            return Err(Loud::chunk_refused().into());
        };
        self.run_chunk(&code, &chunk, Some(&program.source))
    }

    /// Transfers "is this the first instruction executed in this activation"
    /// to the step about to run, which `PROCEDURE` and `USE LOCAL` are the
    /// only readers of.
    pub(crate) fn grant_procedure_permission(&mut self, instruction: &Instruction) {
        if !matches!(instruction.kind, InstructionKind::Label { .. }) {
            self.procedure_permitted =
                std::mem::take(&mut self.activation_mut().first_instruction_pending);
        }
    }

    /// Applies one clause's `Flow` to this activation.
    pub(crate) fn apply_flow(
        &mut self,
        code: &Code<'_>,
        flow: Flow,
    ) -> Result<Option<Ended>, Failure> {
        match flow {
            Flow::Next => self.activation_mut().pc += 1,
            Flow::Goto(target) => self.activation_mut().pc = target,
            // `SIGNAL`, once its target has escaped every nested construct
            // and every `INTERPRET` fragment it fired from (`Flow::Signal`'s
            // own doc comment has why it cannot ride `Goto` to get here).
            // The only consumer, matching `Goto`'s own arm exactly: `target`
            // already resolved against this activation's own body
            // (`resolve_signal_target`), which is exactly the body `code` is
            // bound to.
            Flow::Signal(target) => {
                self.activation_mut().pc = target;
                self.activation_indent = 0;
                self.indent_offset = 0;
            }
            // **The one place an activation's value stops being a clause's
            // temporary**, which is why the root that outlives the temps
            // stack is taken here rather than at each of the half-dozen
            // constructs that can produce one. `EXIT`, a top-level `RETURN`,
            // a `RAISE` with an `EXIT` tail and a handler's own exit all
            // arrive as one of these two variants; every one of them can end
            // up as the value `execute` hands `exit_code_for`, and by then
            // the frame that rooted it has been popped. See
            // [`Interp::root_exit_value`] for the measurement.
            Flow::Exit(value) => {
                if let Some(value) = value {
                    self.root_exit_value(value);
                }
                return Ok(Some(Ended::Exited(value)));
            }
            // The activation boundary `Flow::Return` was added to reach.
            // Every construct between the `RETURN` and here forwarded it
            // untouched; this is the one consumer.
            Flow::Return(value) => {
                if let Some(value) = value {
                    self.root_exit_value(value);
                }
                return Ok(Some(Ended::Returned(value)));
            }
            // Task 11: a `LEAVE`/`ITERATE` that reached the very top of the
            // program -- nothing anywhere, at any nesting depth, ever
            // matched it. This is the exhausted-search family, 28.1 (bare
            // `LEAVE`)/28.2 (bare `ITERATE`)/28.3 (named `LEAVE`)/28.4
            // (named `ITERATE`). `origin.indent` already holds this family's
            // own answer by the time it gets here -- every `Select`/`Do`
            // frame the search walked through on the way up has already
            // reset it to its own `static_indent` as it forwarded past
            // (`LeaveOrigin`'s own doc comment has the rule, corrected after
            // review: it is **not** always zero, only when every popped
            // frame along the way happened to sit at top level). 28.5 (a
            // named `ITERATE` that *did* match something, just not a loop)
            // is a different family, raised where the match was found, in
            // `Select`/`Do`'s own arms, and never reaches here.
            Flow::Leave(name, origin) => {
                self.record_leave_failure(&origin);
                let raised = match name {
                    None => raised_leave_no_loop(),
                    Some(n) => raised_leave_no_match(code.symbols.name(n).as_bytes()),
                };
                return Err(raised.into());
            }
            Flow::Iterate(name, origin) => {
                self.record_leave_failure(&origin);
                let raised = match name {
                    None => raised_iterate_no_loop(),
                    Some(n) => raised_iterate_no_match(code.symbols.name(n).as_bytes()),
                };
                return Err(raised.into());
            }
        }
        Ok(None)
    }

    /// One instruction's own work, with the clause unit already discharged by
    /// whoever called: [`Interp::step`] takes the permission and enters from
    /// `Op::Clause`'s region, and [`crate::ir::Op::Exec`] enters from inside
    /// the [`crate::ir::Op::Clause`] region that already opened the clause.
    pub(crate) fn exec_instruction(
        &mut self,
        code: &Code<'_>,
        index: usize,
        instruction: &Instruction,
        source: Option<&ProgramSource>,
        first_instruction: bool,
    ) -> Result<Flow, Failure> {
        match &instruction.kind {
            InstructionKind::Say { expression } => {
                let value = match expression {
                    Some(expression) => Some(self.eval(code, expression)?),
                    None => None,
                };
                self.say_evaluated(value)?;
                Ok(Flow::Next)
            }

            InstructionKind::Assignment { target, value } => {
                let value = self.eval(code, value)?;
                // `None`: this engine resolves nothing ahead of time, so the
                // write resolves its own slot the way it always has.
                self.assign_evaluated(code, target, value, None)?;
                Ok(Flow::Next)
            }

            // A simple variable back to unset (`Interp::clear_variable`, added
            // expressly for this and never `ObjRef::NIL`, which is a value
            // and not an absence -- `x = .nil; say x` and `y = .nil; drop y;
            // say y` render differently, measured in `drop_variable`'s own
            // doc comment), a whole stem, one tail, or the `(v)` indirect
            // form. See `drop_variable`.
            InstructionKind::Drop { variables } => {
                for variable in variables {
                    self.drop_variable(code, variable)?;
                }
                Ok(Flow::Next)
            }

            // `NUMERIC DIGITS`/`FUZZ`/`FORM`, every spelling `NumericSetting`
            // has. See `exec_numeric`.
            InstructionKind::Numeric {
                setting,
                expression,
            } => {
                self.exec_numeric(code, setting, expression)?;
                Ok(Flow::Next)
            }

            // `TRACE` (D17): sets the running activation's own trace mode, or
            // raises 24.901 for
            // the interactive-only skip-count forms. See `exec_trace`.
            InstructionKind::Trace(setting) => {
                self.exec_trace(code, setting)?;
                Ok(Flow::Next)
            }

            // `EXIT`, bare or with a result: the spike had only the bare form
            // (`expression: None` matched literally, nothing else reaching
            // this arm at all). The value crosses out of the instruction loop
            // as `Flow::Exit`, unconverted -- `Interp::exit_code_for` (`lib.rs`)
            // is what turns it into a process exit code, and it runs once in
            // `execute` rather than here, because a `Flow::Exit` can also
            // come from inside a fragment (`run_fragment`'s own propagating
            // arm, below), and the conversion needs nothing this loop knows
            // that `execute` does not already have.
            InstructionKind::Exit { expression } => {
                let value = match expression {
                    Some(expression) => Some(self.eval(code, expression)?),
                    None => None,
                };
                self.returned_value(value, ReturnKeyword::Exit)
            }

            // A label is a traced no-op: the C++'s own `execute` on a label
            // instruction only traces it (Task 13's own construct -- a
            // `Label` clause is echoed here via `Op::Clause`'s region, same
            // as any other instruction) and does nothing else besides.
            // `SIGNAL`/`CALL` reach a label by jumping to the instruction
            // after it; nothing ever executes the label node for its own
            // effect.
            InstructionKind::Label { .. } => Ok(Flow::Next),

            InstructionKind::Nop => Ok(Flow::Next),

            // `INTERPRET expr`: evaluate to a string, parse it as a fragment,
            // run it against **this** activation, through `run_fragment`. The
            // arm is thin because `run_fragment` is where the work is.
            InstructionKind::Interpret { expression } => {
                let value = self.eval(code, expression)?;
                self.roots.push_temp(value);
                // `RexxInstructionExpression::evaluateStringExpression`
                // (`instructions/RexxInstruction.cpp:257`), the one
                // `requestString` every instruction that evaluates a single
                // string expression shares -- so the `>>>` below traces the
                // conversion, measured: `trace r` over `interpret .K` with a
                // class-side `makeString` returning `'nop'` prints
                // `>>>   "nop"` and then the fragment's own `*-* nop`.
                let value = self.required_string_value(value)?;
                let text = self.to_text(value).to_vec();
                // `>>>` on the interpreted text itself, before the fragment
                // runs -- the same `trace_result` every other value-producing
                // arm calls (`Say`, `Assignment`), at the same
                // `current_value_indent`. Review finding I1(a): the arm
                // shipped without this and was the only value-producing arm in
                // the crate that traced nothing. Measured (`trace r`, `zz =
                // 'nop'`, `interpret zz`), the oracle prints
                // ```text
                //      3 *-* interpret zz
                //        >>>   "nop"
                //      3 *-* nop
                // ```
                self.trace_result(self.clause_state.current_value_indent, &text);
                // **The fragment's level, with delta 0.** Measured: a
                // fragment's clauses print at the enclosing `INTERPRET`
                // clause's own absolute indent plus whatever nests them
                // *inside* the fragment -- `interpret "do jj = 1 to 1; say 2
                // & 1; end"` at top level echoes the inner clause at 2 and
                // the `INTERPRET` at 0, and the identical fragment two `DO`s
                // deep echoes them at 6 and 4. So the base is the enclosing
                // clause's printed indent exactly, with no bump of its own --
                // unlike a called routine's, which the same measurements put
                // two spaces further in (`call sub1` at printed indent 4 into
                // a flat routine echoes the callee's clause at 6), and which
                // is Task 3's to add.
                let base_indent = self.clause_state.current_value_indent;
                let base_line = self.clause_site(source, instruction).map(|(line, _)| line);
                let saved_base = std::mem::replace(&mut self.activation_indent, base_indent);
                let saved_offset = std::mem::take(&mut self.indent_offset);
                let saved_line = std::mem::replace(&mut self.clause_line_override, base_line);
                // **The fragment's own condition queue**, which the depth is
                // the key to rather than a second collection --
                // `Interp::fragment_depth` has why the oracle has one and what
                // was measured on either side of it. Incremented rather than
                // replaced the way the values saved above are, because a
                // nested fragment inherits each of those and needs a level of
                // its own here.
                self.fragment_depth += 1;
                // `saved_line` read before the replace above is also the
                // answer to "is a fragment already running", which is the one
                // extra thing `enter_fragment` needs and the only place it is
                // in hand. Nothing new is tracked for it.
                let saved_entry = self.enter_fragment(saved_line.is_some());
                let flow = self.run_fragment(text);
                // **A condition still queued at this depth dies with the
                // fragment**, on the failing path as well as this one, because
                // the activation whose queue it was in is what ends. Measured,
                // `interpret 'zq = raiser()'` with a handler that requeues: the
                // oracle runs the first handler and never runs the second, and
                // the enclosing clause's own boundary does not pick it up.
                // Dropping the entries also keeps a program that runs
                // `INTERPRET` in a loop from accumulating undeliverable ones.
                let depth = self.fragment_depth;
                self.pending_traps
                    .retain(|pending| pending.fragment_depth != depth);
                // **Nothing deeper than the fragment just left may survive
                // it**, which is the invariant that lets the delivery key be
                // an equality rather than a comparison: a deeper entry would
                // belong to a fragment that already ran this same discard on
                // its own way out, and a boundary out here would then have to
                // decide whether it inherits one. Asserted rather than
                // reasoned about, because the discard runs inside a
                // `deliver_pending_traps` that a handler can have re-entered.
                debug_assert!(
                    self.pending_traps
                        .iter()
                        .all(|pending| pending.fragment_depth < depth),
                    "a condition queued inside a fragment outlived that fragment's own exit"
                );
                self.fragment_depth -= 1;
                self.leave_fragment(saved_entry);
                self.activation_indent = saved_base;
                self.indent_offset = saved_offset;
                self.clause_line_override = saved_line;
                flow
            }

            // `IF`/`THEN`/`ELSE`. This arm resolves the whole construct
            // itself rather than leaving the outer loop to fall through the
            // flat list -- see `run_bounded`'s doc comment for why that is
            // not optional. `false_target`'s own doc comment: "the ELSE if
            // there is one, otherwise the instruction after the THEN
            // branch" -- confirmed by tracing `block.rs` by hand, it is the
            // `Else` instruction's own index when there is one, landing
            // *on* it rather than past it.

            // A pure marker: only ever reached inside `If`'s own bounded
            // sub-loop (the true branch, right after the `IF`) or via
            // ordinary fallthrough on the false path. Never independently
            // dispatched for a decision of its own.
            InstructionKind::Then => Ok(Flow::Next),

            // Also a pure marker (`ast.rs`'s own doc comment: "executing an
            // ELSE only traces"). Reached only by ordinary fallthrough on
            // the false path -- the true path's `Goto` in the `If` arm above
            // skips straight past it to `then_exit`, so this is never asked
            // to decide anything.
            InstructionKind::Else { .. } => Ok(Flow::Next),

            // `SELECT`/`SELECT CASE`. Evaluates `case` at most once (if this
            // is a `SELECT CASE`), then tests each of its own *listed*
            // `whens` in source order by reading the `When`/`WhenCase` node
            // directly as data (`condition`/`values`, `false_target`,
            // `exit`) rather than dispatching through `Op::Clause`'s region
            // -- a *listed* `When`/`WhenCase` node (one collected into this
            // `whens` list, `ast.rs`'s own doc comment) must never be
            // independently stepped for a decision of its own, only ever
            // run past inside a bounded sub-loop. An *absorbed* one (never
            // collected here at all, because it is itself another `When`/
            // `WhenCase`'s own `THEN`) is the exception, and is
            // independently stepped -- see the `When`/`WhenCase` arm,
            // below, for both halves.

            // **Fixed after review: this used to be a bare `Ok(Flow::Next)`,
            // and that was a silently wrong answer, not a formatting gap.**
            // A `When`/`WhenCase` is only ever reached here through the
            // absorbed-`WHEN` shape -- a `WHEN` whose own `THEN` consequence
            // is itself a `WHEN`/`WHEN CASE` clause, which the enclosing
            // `SELECT`'s own `whens` never collects (`ast.rs`'s own doc
            // comment on `whens`, `LanguageParser.cpp:1319`) -- since a
            // *listed* `When`/`WhenCase` is always fully handled by
            // `Select`'s own explicit arm, above, without ever calling
            // `step` on itself (its own body range never contains another
            // listed sibling's index).
            InstructionKind::When { condition, .. } => {
                self.eval_condition(
                    code,
                    condition,
                    ConditionTrace::Result(self.clause_state.current_value_indent),
                    raised_when_not_logical,
                )?;
                Ok(Flow::Next)
            }
            // `SELECT CASE`'s own absorbed form.
            InstructionKind::WhenCase {
                values,
                false_target,
                ..
            } => match self.current_case_text.clone() {
                Some(case_text) => {
                    let indent = self.clause_state.current_value_indent;
                    if self.test_case_when(code, values, &case_text, indent)? {
                        Ok(Flow::Next)
                    } else {
                        // **Corrected after a second re-verification found
                        // the first version of this line wrong under
                        // nesting.** `current_value_indent.saturating_sub
                        // (2)` (this line's own first attempt) gave the
                        // right answer at the top level by coincidence
                        // (`6 - 2 = 4`) and the *wrong* one nested one `DO`
                        // deeper (`8 - 2 = 6`, where the oracle still wants
                        // `4`) -- measured directly (`t13_f3_nested.rex`,
                        // then a `TRACE R` transcript one level deeper
                        // again, `i_trace_nested_escape.rex`/`j_trace_
                        // nested_otherwise.rex`, this task's report has
                        // all three). The offset is the **constant** `4`,
                        // not a function of how deep the absorbed
                        // condition itself sits: it is exactly two
                        // `indent()` bumps -- the enclosing, listed
                        // `WHEN`/`WHEN CASE`'s own marker, then its own
                        // body entry -- past wherever an *ordinary* `SELECT`
                        // -level construct (`END`, `OTHERWISE`) would sit,
                        // and that gap is the same two bumps regardless of
                        // how many other constructs enclose the whole
                        // `SELECT`. Confirmed at both nesting depths for
                        // all three landing shapes (`END`, `OTHERWISE`'s
                        // own marker, `OTHERWISE`'s own body) before
                        // trusting it a second time.
                        self.indent_offset = 4;
                        Ok(Flow::Goto(
                            false_target.unwrap_or(code.body.instructions.len()),
                        ))
                    }
                }
                None => {
                    for value in values {
                        let v = self.eval(code, value)?;
                        self.roots.push_temp(v);
                    }
                    Ok(Flow::Next)
                }
            },
            InstructionKind::Otherwise => Ok(Flow::Next),

            // `DO`/`LOOP`, every kind but `DO WITH` (the loud path,
            // `run_loop`'s own doc comment) -- Task 11. Resolves the whole
            // construct itself, every iteration, exactly the discipline
            // `If`/`Select` already established: see `Flow::Leave`'s own
            // doc comment for why `Do`'s own arm never returns until the
            // entire loop is over, one way or another.
            // `DO`/`LOOP`, `IF` and `SELECT` are **not** reachable here.
            InstructionKind::Do(_)
            | InstructionKind::Loop(_)
            | InstructionKind::If { .. }
            | InstructionKind::Select { .. } => Err(Loud::instruction(&instruction.kind).into()),

            // `LEAVE`/`ITERATE`, bare or by name -- Task 11. Resolves to
            // data, not a failure (`Flow::Leave`'s own doc comment): whether
            // this instruction's own name matches anything is answered by
            // whichever `Do`/`Select` (or `run_activation`'s own top level,
            // if none does) inspects the `Flow` this returns, never here.
            InstructionKind::Leave { name } => Ok(Flow::Leave(
                *name,
                Box::new(self.leave_origin(code, index, source, instruction)),
            )),
            InstructionKind::Iterate { name } => Ok(Flow::Iterate(
                *name,
                Box::new(self.leave_origin(code, index, source, instruction)),
            )),

            // `END`. `Select`'s two non-7.3 closings (`OTHERWISE` present)
            // are reached only by that `OTHERWISE`'s own ordinary body
            // fallthrough and do nothing. `EndStyle::Select`'s own doc
            // comment: "Reaching this END at run time is error 7.3, because
            // every WHEN was false" -- the ordinary way to land here, but
            // **not the only one since F3**: an absorbed `WhenCase`'s own
            // false-branch escape (`InstructionKind::WhenCase`'s own arm,
            // above) can also `Goto` straight onto this exact instruction,
            // carrying its own residual indent along in `pending_escape_
            // indent` for this arm's own 7.3 to be reported at rather than
            // this position's ordinary `static_indent`. An earlier version
            // of this comment said `Select`'s own arm "sends every other
            // path around this instruction entirely," which was true of
            // every path *it* controls directly and false of this one,
            // which escapes through it rather than being dispatched by it.
            InstructionKind::End { closes, .. } => {
                let closes = closes
                    .as_ref()
                    .expect("an End's closes is only None while its body is still being assembled");
                match closes.style {
                    EndStyle::Select => Err(raised_select_no_when().into()),
                    EndStyle::Otherwise
                    | EndStyle::LabeledOtherwise
                    | EndStyle::Do
                    | EndStyle::LabeledDo
                    | EndStyle::Loop => Ok(Flow::Next),
                }
            }

            // `CALL name`, `CALL "name"`, `CALL (expr)` and `CALL ON`/`CALL
            // OFF`. The one arm of `rexx_parse::Call` that stays loud keeps
            // its own owner (`instruction_owner`, `lib.rs`): `Qualified`
            // (`CALL ns:name`) is Phase 5's.
            InstructionKind::Call(call) => match &**call {
                // `name` arrives already upcased for the symbol form and
                // verbatim for the quoted one (`rexx-parse`'s own `Call`
                // doc). `literal` inverts into "may this search the label
                // table": measured, `call "SUB"` with `sub:` present is
                // Error 43.1 and not a call, so the quoted form bypasses the
                // search entirely rather than merely matching case-sensitively.
                rexx_parse::Call::Named {
                    name,
                    literal,
                    args,
                } => self.exec_call(code, name, !*literal, args),
                // `CALL (expr)`: the target is evaluated in the caller, its
                // value is traced, and the **verbatim** text is what the
                // label search sees. Both halves are measured and they pull
                // in opposite directions from the quoted form: `nm = 'SUB';
                // call (nm)` runs `sub:`, so this form *does* search labels,
                // while `nm = 'sub'; call (nm)` is Error 43.1 `Could not find
                // routine "sub"`, so the value is not upcased on the way in.
                rexx_parse::Call::Dynamic { target, args } => {
                    let value = self.eval(code, target)?;
                    self.roots.push_temp(value);
                    // `targetName = evaluatedTarget->requestString()`
                    // (`instructions/CallInstruction.cpp:296`), before the
                    // `>>>` below: measured, `trace r` over `call (.K)` with a
                    // class-side `makeString` returning `'MS'` traces
                    // `>>>   "MS"` and runs `MS:`.
                    let value = self.required_string_value(value)?;
                    let name = self.to_text(value).to_vec();
                    // Its own `>>>`, at the `CALL` clause's own indent, which
                    // `Call::Named` has no equivalent of -- measured, `call
                    // sub 1+1, 'q'` under `trace r` traces no value line at
                    // all while `call (nm)` traces one for the target.
                    self.trace_result(self.clause_state.current_value_indent, &name);
                    self.exec_call(code, &name, true, args)
                }
                // `CALL ON cond NAME label` / `CALL OFF cond`. Shares every
                // line of its implementation with `SIGNAL ON`/`OFF` except
                // the one `bool` that decides how the handler runs -- see
                // `exec_condition_trap`, and `Trap`'s own doc comment
                // (`activation.rs`) for the two measured behaviours that
                // `bool` selects between.
                rexx_parse::Call::Trap(trap) => self.exec_condition_trap(trap, true),
                // `CALL ns:name`, whose target is a public routine of that
                // namespace and nothing else --
                // `RexxInstructionQualifiedCall::resolve`
                // (`instructions/CallInstruction.cpp:443`). It settles
                // `RESULT` exactly as `Named` does, which is why it joins the
                // same second half; what it does not do is search labels or
                // builtins, so it has no `search_labels` to pass.
                rexx_parse::Call::Qualified {
                    namespace,
                    name,
                    args,
                } => {
                    let namespace = code.symbols.name(*namespace).as_bytes().to_vec();
                    let name = code.symbols.name(*name).as_bytes().to_vec();
                    let package = self.running_program().ok_or_else(Loud::missing_body)?;
                    let resolution = self
                        .namespace_routine(package, &namespace, &name)
                        .map(Resolved::Routine);
                    let resolved = self.resolved_after_arguments(code, resolution, args)?;
                    self.invoke_named_call(code, resolved, &name, args)
                }
            },

            // `RETURN`, bare or with a value. Unwinds to the activation
            // boundary; `Flow::Return`'s own doc comment has why none of the
            // other variants expresses that, and why the main body's own
            // `RETURN` ends the program.
            InstructionKind::Return { expression } => {
                let value = match expression {
                    Some(expression) => Some(self.eval(code, expression)?),
                    None => None,
                };
                self.returned_value(value, ReturnKeyword::Return)
            }

            // `SIGNAL label` and `SIGNAL VALUE`. `Signal::Trap` (`SIGNAL
            // ON`/`SIGNAL OFF`) stays loud, Task 7's own owner
            // (`instruction_owner`, `lib.rs`).
            InstructionKind::Signal(signal) => match &**signal {
                // `name` is already upcased for a bare symbol and verbatim
                // for a quoted one (`rexx-parse`'s own `Signal` doc), and
                // **both forms search the label table** -- unlike `CALL
                // "name"`, which never does, because `SIGNAL` has no
                // builtin/external fallback for a literal spelling to
                // deliberately bypass into. Measured: `signal "sub"` with
                // `sub:` present still raises 16.1 (case-sensitive against
                // the label's own upcased spelling, so the lowercase quoted
                // form does not match), while `signal Sub` (bare, mixed
                // case) and `signal "SUB"` both run it.
                rexx_parse::Signal::Label(name) => self.signal_to_label(name),
                // `SIGNAL VALUE expr`. Its own `>K>` -- `"VALUE" => text`, at
                // this clause's own indent with no `+2` the way `WHILE`/
                // `UNTIL` carry (measured one `DO` deep: `signal value
                // target` traces `>K>     "VALUE" => "THERE"` at the same
                // indent as its own clause echo, unlike those two, which are
                // evaluated as part of the *enclosing* `DO`/`LOOP`'s own
                // step). The rendered text is then searched exactly like
                // `Label`'s own bytes, with **no shape check on the value at
                // all** -- measured, a number, an empty string and an
                // ordinary non-label string all raise 16.1 naming that exact
                // text, none of them a different error.
                rexx_parse::Signal::Value(expr) => {
                    let value = self.eval(code, expr)?;
                    // Rooted here rather than inside `signal_to_value`, which
                    // the compiled stream reaches with the value already in a
                    // register: the temp is what roots it across the render
                    // there, and a second one would be a frame this clause
                    // does not own.
                    self.roots.push_temp(value);
                    self.signal_to_value(value)
                }
                // `SIGNAL ON cond NAME label` / `SIGNAL OFF cond`. Unlike the
                // two arms above it transfers no control of its own: it edits
                // this activation's trap table and falls through to the next
                // clause, and the transfer happens later, if the condition is
                // ever raised.
                rexx_parse::Signal::Trap(trap) => self.exec_condition_trap(trap, false),
            },

            // `PROCEDURE`, bare or with an `EXPOSE` list (D9r). Isolates the
            // callee's variable pool and aliases the exposed names back into
            // the pool they came from. See `exec_procedure`.
            InstructionKind::Procedure { variables } => {
                self.exec_procedure(code, variables, first_instruction)?;
                Ok(Flow::Next)
            }

            // `EXPOSE`: binds this method's names to the receiving object's
            // pool for the scope the method was declared in. See
            // `exec_expose`.
            InstructionKind::Expose { variables } => {
                self.exec_expose(code, variables)?;
                Ok(Flow::Next)
            }

            // `USE ARG`/`USE STRICT ARG`/`USE LOCAL`. See `exec_use`.
            InstructionKind::Use(use_) => {
                self.exec_use(code, use_, first_instruction)?;
                Ok(Flow::Next)
            }

            // `RAISE`, in all of its forms. See `exec_raise`, whose doc
            // comment carries the delivery table -- which is the whole of
            // this instruction and is not derivable from the grammar.
            InstructionKind::Raise(raise) => self.exec_raise(code, raise),

            // `PUSH`/`QUEUE line` (I15). One arm, not two copies that can
            // drift (review round 1's M4): the two spellings differ only in
            // which end of the queue the value lands on, decided below by
            // which variant matched. The rendering, the `>>>` line and the
            // write are `Interp::queue_evaluated`'s, shared with
            // `crate::ir::Op::Queue`.
            InstructionKind::Push { expression } | InstructionKind::Queue { expression } => {
                let value = match expression {
                    Some(expression) => Some(self.eval(code, expression)?),
                    None => None,
                };
                let keyword = if matches!(instruction.kind, InstructionKind::Push { .. }) {
                    QueueKeyword::Push
                } else {
                    QueueKeyword::Queue
                };
                self.queue_evaluated(value, keyword)?;
                Ok(Flow::Next)
            }

            // `PARSE`, in every source spelling, plus the two short forms that
            // are the same instruction with `UPPER` already set: `ARG
            // template` is `PARSE UPPER ARG template` and `PULL template` is
            // `PARSE UPPER PULL template`. One arm, because `rexx-parse`
            // builds the identical `Parse` body for all three (its
            // `parse_instruction_body` takes the implied source and sets
            // `upper` from it), so a second arm here would be a second copy of
            // the dispatch and nothing else.
            InstructionKind::Parse(parse)
            | InstructionKind::Arg(parse)
            | InstructionKind::Pull(parse) => {
                self.exec_parse(code, parse, None)?;
                Ok(Flow::Next)
            }

            // `ADDRESS`, the three forms that only name an environment: the
            // constant `ADDRESS env`, the computed `ADDRESS VALUE expr` (and
            // its parenthesised spelling), and the bare toggle. See
            // `exec_address`.
            InstructionKind::Address(address) => {
                if address.command.is_some() || address.io.is_some() {
                    return Err(Loud::instruction(&instruction.kind).into());
                }
                self.exec_address(code, address)?;
                Ok(Flow::Next)
            }

            // A message send as a whole clause: `q~append(1)`, `q~~append(1)`
            // and the message-assignment form `q[1] = 2`. See
            // `exec_message`.
            InstructionKind::Message { term, value } => {
                self.exec_message(code, term, value.as_ref())
            }

            // `GUARD ON`/`GUARD OFF`, with or without a `WHEN`. See
            // `exec_guard`.
            InstructionKind::Guard(guard) => self.exec_guard(code, guard),

            // `REPLY`, bare or with a value. See `exec_reply`.
            InstructionKind::Reply { expression } => {
                self.exec_reply(code, index, expression.as_ref())
            }

            // `FORWARD` and its options. See `exec_forward`.
            InstructionKind::Forward(forward) => self.exec_forward(code, forward),

            other => Err(Loud::instruction(other).into()),
        }
    }

    /// A message send that is a clause of its own
    /// (`RexxInstructionMessage::execute`, `MessageInstruction.cpp:151`).
    pub(crate) fn exec_message(
        &mut self,
        code: &Code<'_>,
        term: &Expr,
        value: Option<&Expr>,
    ) -> Result<Flow, Failure> {
        let ExprKind::Message {
            target,
            name,
            super_class,
            args,
            cascade,
        } = &term.kind
        else {
            // `rexx-parse` builds this variant only from a message term
            // (`instruction.rs`'s `message`), so nothing else can arrive;
            // loud rather than a panic, on the standing rule that a parser
            // guarantee the type system does not carry must not abort.
            return Err(Loud::expression(&term.kind).into());
        };
        // **`message_term` directly rather than through `Interp::eval`**,
        // even for the form that is an ordinary expression: `eval`'s own
        // `ExprKind::Message` arm turns a valueless send into 91.999, which
        // is the expression position's error and not this one's -- measured,
        // a whole-clause `.K~m` on a method ending in a bare `return` is
        // rc 0.
        let mut assigned_name;
        let result = match value {
            None => {
                let probe = 0u8;
                self.enter_eval_node(&raw const probe)?;
                let sent = self.message_term(
                    code,
                    &crate::dispatch::MessageTerm {
                        target,
                        name,
                        super_class: super_class.as_deref(),
                        args,
                        cascade: *cascade,
                        assigned: None,
                    },
                );
                self.depth -= 1;
                sent?
            }
            Some(value) => {
                assigned_name = name.to_vec();
                assigned_name.push(b'=');
                self.message_term(
                    code,
                    &crate::dispatch::MessageTerm {
                        target,
                        name: &assigned_name,
                        super_class: super_class.as_deref(),
                        args,
                        // The oracle builds this form as `KEYWORD_MESSAGE`
                        // whatever the term's own tilde count, so a `~~`
                        // written here is not a cascade.
                        cascade: false,
                        assigned: Some(value),
                    },
                )?
            }
        };
        let slot = self.reserved_result_slot();
        let frame = self.activation().frame;
        // **A send that produced no value drops `RESULT`** rather than
        // leaving the previous one in place -- the same rule a bare `return`
        // from a `CALL` follows. Measured: `result = 'unset'` then `.K~m`
        // then `symbol('RESULT')` is `LIT`.
        match result {
            Some(result) => {
                self.roots.push_temp(result);
                self.set_variable(frame, slot, result);
            }
            None => self.clear_variable(frame, slot),
        }
        Ok(Flow::Next)
    }

    /// `PROCEDURE`, with or without an `EXPOSE` list (D9r).
    fn exec_procedure(
        &mut self,
        code: &Code<'_>,
        variables: &[VariableRef],
        first_instruction: bool,
    ) -> Result<(), Failure> {
        // 17.1 covers every shape but one: the first instruction executed
        // after an internal `CALL` or function invocation. Both halves are
        // needed -- every other entry fails the second, and anything after
        // another instruction in the same activation fails the first.
        let entered_by_internal_call = match self.activation().entry {
            Entry::InternalCall => true,
            Entry::TopLevel | Entry::Routine | Entry::Method => false,
        };
        if !(first_instruction && entered_by_internal_call) {
            return Err(Raised::procedure_out_of_place().into());
        }

        // The swap at the end of this function gives the callee a frame of
        // its own, and an activation that already owned one would be pushing
        // a second onto the same stack. That is the state
        // `RootSet::grow_slots` and `pop_slots` catch a step or two later,
        // by which time the instruction that caused it has returned, so the
        // invariant is asserted where it is established rather than where
        // the damage surfaces. `Activation::nested` builds the entry kind
        // admitted above, and it starts the callee sharing.
        assert!(
            !self.activation().owns_frame,
            "a PROCEDURE admitted in an activation that already owns its frame"
        );

        let names = self.expose_names(code, variables)?;

        // Resolved against the pool still in force, which is the caller's:
        // this activation has not swapped in a frame of its own yet.
        let outer = self.activation().frame;
        let mut bindings: Vec<(Box<[u8]>, usize, VarHome)> = Vec::with_capacity(names.len());
        for name in names {
            // Whole stems alias fine -- the stem object lives in one slot,
            // so aliasing that slot shares the object and every measured
            // stem transcript falls out of it. A single tail does not; see
            // `Loud::compound_expose`.
            if shape_of(&name) == NameShape::Compound {
                return Err(Loud::compound_expose("PROCEDURE EXPOSE", &name).into());
            }
            let slot = self.slot_of(&name);
            // A name the enclosing method exposed has no frame storage to
            // alias: its home is the object's pool, and the callee gets the
            // same home rather than a slot. Measured -- a class method
            // exposing `v` and calling `inner: procedure expose v`, which
            // assigns `v` -- the object variable is what changes.
            let target = match self.exposure(outer, slot) {
                Some(var) => VarHome::Instance(Box::new(var.clone())),
                None => VarHome::Slot(self.roots.slot_ref(outer, slot)),
            };
            bindings.push((name, slot, target));
        }

        // Any name that needed a fresh slot just grew the caller's frame and
        // was recorded in *this* activation's `extra` -- which is a clone of
        // the caller's, taken at the call. The caller has to learn about it,
        // because after the isolation below this map is replaced and the
        // return path deliberately does not write it back.
        let resolved = self.activation().extra.clone();
        if let Some(caller) = self.caller_activation_mut() {
            caller.extra = resolved;
        }

        // Sized from the caller's *current* frame length rather than from
        // `plan.len()`: an exposed name may sit at an index the caller grew
        // into, and that same index has to address something on this side of
        // the alias too.
        let len = self.roots.frame_len(outer);
        let inner = self.roots.push_slots(len);
        let mut exposed: Vec<(usize, InstanceVar)> = Vec::new();
        for (_, slot, target) in &bindings {
            match target {
                VarHome::Slot(target) => self.roots.alias_slot(inner, *slot, *target),
                VarHome::Instance(var) => exposed.push((*slot, (**var).clone())),
            }
        }

        // The callee's own run-time bindings start empty -- that is the
        // isolation -- except for exposed names the plan never saw, which
        // must keep resolving to the index the alias was installed at.
        let plan = Rc::clone(&self.activation().plan);
        let mut extra = rexx_core::NameMap::default();
        for (name, slot, _) in bindings {
            if plan.slot_of(&name).is_none() {
                extra.insert(name, slot);
            }
        }

        let activation = self.activation_mut();
        activation.frame = inner;
        activation.owns_frame = true;
        activation.extra = extra;
        // **Replaced, not extended.** This activation inherited the caller's
        // exposures when it was pushed, and a `PROCEDURE` isolates the pool:
        // a name the caller exposed and this list does not name is an
        // ordinary local here. Measured -- a class method exposing `v` and
        // calling `inner: procedure` with no list, which assigns `v` -- the
        // object variable is unchanged.
        activation.exposed = exposed;
        Ok(())
    }

    /// `EXPOSE`: bind every name it lists to the receiving object's variable
    /// pool for the scope the running method was declared in.
    pub(crate) fn exec_expose(
        &mut self,
        code: &Code<'_>,
        variables: &[VariableRef],
    ) -> Result<(), Failure> {
        let Some(identity) = self.activation().method_identity.as_ref() else {
            return Err(Raised::expose_outside_method().into());
        };
        let scope = identity.scope;
        let receiver = identity.receiver;
        let owner = self.pool_owner(receiver)?;
        for variable in variables {
            match variable {
                VariableRef::Direct(id) => {
                    let name = code.symbols.name(*id).as_bytes().into();
                    self.bind_exposed(owner, scope, name)?;
                }
                // **The selector's own name is bound before its value is
                // read**, and the order is observable rather than tidy.
                // Measured: with a class-scope `LISTER` holding `'BETA'` and a
                // class-scope `BETA` holding `'beta-value'`, `expose (lister)`
                // in a third method reads `[BETA][beta-value]` -- so `LISTER`
                // was read out of the object's pool and `BETA` was exposed
                // from it. Reading the selector first, out of the frame, gets
                // `[BETA][BETA]`: the frame's `LISTER` is unset, so its
                // derived name `LISTER` is what spells the list.
                VariableRef::Indirect(id) => {
                    let name = code.symbols.name(*id).as_bytes().into();
                    self.bind_exposed(owner, scope, name)?;
                    let (value, _novalue) = self.read(code, *id);
                    // `IndirectVariableReference::evaluate`'s own
                    // `value->requestString()`
                    // (`expression/IndirectVariableReference.cpp:132`), so a
                    // selector holding an object spells its list from the
                    // conversion. Measured, oracle rc 0: `zz = 1; x = .K;
                    // drop (x)` with a class-side `makeString` returning
                    // `'zz'` leaves `SYMBOL('ZZ')` at `LIT`.
                    let value = self.required_string_value(value)?;
                    let text = self.to_text(value).into_owned();
                    for word in split_indirect_words(&text) {
                        let word = validate_indirect_word(word)?;
                        self.bind_exposed(owner, scope, word.into())?;
                    }
                }
            }
        }
        Ok(())
    }

    /// Binds one name to `scope`'s pool on `owner`, for the rest of this
    /// activation.
    fn bind_exposed(
        &mut self,
        owner: ObjRef,
        scope: ObjRef,
        name: Box<[u8]>,
    ) -> Result<(), Failure> {
        // A whole stem is one value in one pool entry, so it binds like any
        // other name; a single tail is aliasing *inside* a stem object, which
        // this crate has no representation for. Measured on the oracle,
        // `expose a.1` in one class method assigning `a.1` and `a.2` and the
        // same in another reading them back: `[tail-one][A.2]`, so tail 1 is
        // shared and tail 2 is the method's own local. Exposing the whole stem
        // instead would be a silent wrong answer.
        if shape_of(&name) == NameShape::Compound {
            return Err(Loud::compound_expose("EXPOSE", &name).into());
        }
        let slot = self.slot_of(&name);
        let var = InstanceVar { owner, scope, name };
        let activation = self.activation_mut();
        // Replaced rather than appended: `expose v v` is legal and rc 0 on the
        // oracle, and two entries for one slot would leave every later read
        // deciding between them by list order.
        match activation.exposed.iter_mut().find(|(at, _)| *at == slot) {
            Some(bound) => bound.1 = var,
            None => activation.exposed.push((slot, var)),
        }
        Ok(())
    }

    /// The object whose [`rexx_core::ScopePools`] a send to `receiver` binds
    /// into.
    pub(crate) fn pool_owner(&mut self, receiver: ObjRef) -> Result<ObjRef, Failure> {
        if !self.heap.is_class(receiver) {
            if matches!(
                self.heap.get(receiver).map(|object| &object.body),
                Some(Body::Instance { .. })
            ) {
                return Ok(receiver);
            }
            return Err(Loud::expose_receiver().into());
        }
        if let Some(owner) = self.class_variables.get(&receiver) {
            return Ok(*owner);
        }
        // The pools of the class *object*, whose own class is its metaclass --
        // what `.K~class` answers. This object is storage and never a value a
        // program holds.
        let holds = self.classes().class_of(receiver);
        let behaviour = self.classes().instance_behaviour_handle(holds);
        let owner = self.alloc_with(
            BehaviourId::OBJECT,
            Body::Instance {
                class: holds,
                behaviour,
                name: None,
                pools: ScopePools::new(),
                own: None,
                native: None,
            },
        );
        // Held by the class before anything else can allocate, which is the
        // rule `.environment` and `.local` are created under.
        self.class_owns(receiver, owner);
        self.class_variables.insert(receiver, owner);
        Ok(owner)
    }

    /// Every name one `PROCEDURE EXPOSE` list names, in source order.
    fn expose_names(
        &mut self,
        code: &Code<'_>,
        variables: &[VariableRef],
    ) -> Result<Vec<Box<[u8]>>, Failure> {
        let mut names: Vec<Box<[u8]>> = Vec::new();
        for variable in variables {
            match variable {
                VariableRef::Direct(id) => names.push(code.symbols.name(*id).as_bytes().into()),
                VariableRef::Indirect(id) => {
                    // The selector itself, then the names its value spells.
                    names.push(code.symbols.name(*id).as_bytes().into());
                    let (value, _novalue) = self.read(code, *id);
                    let value = self.required_string_value(value)?;
                    let text = self.to_text(value).into_owned();
                    for word in split_indirect_words(&text) {
                        names.push(validate_indirect_word(word)?.into());
                    }
                }
            }
        }
        Ok(names)
    }

    /// `USE ARG`, `USE STRICT ARG` and `USE LOCAL`.
    /// ```text
    /// as the program's own first instruction    98.993, rc 158
    /// as a ::ROUTINE's own first instruction    98.993, rc 158
    /// anywhere else                             99.910, rc 157
    /// ```
    fn exec_use(
        &mut self,
        code: &Code<'_>,
        use_: &Use,
        first_instruction: bool,
    ) -> Result<(), Failure> {
        match use_ {
            Use::Local { .. } => {
                // 98.993 is "this is not a method invocation" and 99.910 is
                // "it is one, but this is not its first instruction", so
                // what decides between them is the entry kind rather than
                // whether there was a call. Measured on the oracle: `use
                // local` first in a `::ROUTINE` is 98.993 at rc 158, the
                // same answer the top-level shape gets.
                let method_invocation = match self.activation().entry {
                    Entry::TopLevel | Entry::InternalCall | Entry::Routine => false,
                    Entry::Method => true,
                };
                if first_instruction && method_invocation {
                    // The one shape the oracle **runs**: measured, `use
                    // local` as a `::METHOD`'s first instruction is rc 0.
                    // What it does is bind every name in its list as a
                    // local, which is `EXPOSE`'s own machinery seen from the
                    // other side, so it is loud until that lands rather than
                    // answering a condition the oracle does not raise.
                    Err(Loud::use_local_in_a_method().into())
                } else if first_instruction {
                    Err(Raised::use_local_outside_method().into())
                } else {
                    Err(Raised::use_local_not_first().into())
                }
            }
            Use::Arg {
                strict,
                allow_optionals,
                targets,
            } => self.exec_use_arg(code, *strict, *allow_optionals, targets),
        }
    }

    /// `USE ARG`/`USE STRICT ARG`: bind the call's arguments to this
    /// instruction's targets, positionally.
    fn exec_use_arg(
        &mut self,
        code: &Code<'_>,
        strict: bool,
        allow_optionals: bool,
        targets: &[Option<UseTarget>],
    ) -> Result<(), Failure> {
        let in_method = self.activation().entry == Entry::Method;
        if strict {
            let supplied = self.call_context.arguments.len();
            // The minimum is the position of the last target that must be
            // supplied -- one with no default of its own. A later target
            // carrying a default does not raise it, which is what makes `use
            // strict arg p, q = 'dflt'` legal with one argument.
            let minimum = targets
                .iter()
                .rposition(|target| {
                    target
                        .as_ref()
                        .is_none_or(|target| target.default.is_none())
                })
                .map_or(0, |index| index + 1);
            if supplied < minimum {
                return Err(if in_method {
                    Raised::not_enough_method_arguments(minimum).into()
                } else {
                    let name = self.call_context.name.clone();
                    Raised::not_enough_arguments(&name, minimum).into()
                });
            }
            if !allow_optionals && supplied > targets.len() {
                return Err(if in_method {
                    Raised::too_many_method_arguments(targets.len()).into()
                } else {
                    let name = self.call_context.name.clone();
                    Raised::too_many_arguments(&name, targets.len()).into()
                });
            }
        }

        for (index, target) in targets.iter().enumerate() {
            let Some(target) = target else { continue };
            // `get` past the end and a `None` inside the list are the same
            // thing to a target: nothing was supplied for this position.
            let argument = self.call_context.arguments.get(index).cloned().flatten();
            self.bind_use_target(code, index, target, argument, strict, in_method)?;
        }
        Ok(())
    }

    /// Binds one `USE ARG` target to one argument, or to its default, or to
    /// nothing.
    fn bind_use_target(
        &mut self,
        code: &Code<'_>,
        index: usize,
        target: &UseTarget,
        argument: Option<ObjRef>,
        strict: bool,
        in_method: bool,
    ) -> Result<(), Failure> {
        let position = index + 1;
        if target.alias {
            let Some(argument) = argument else {
                return Err(Raised::variable_reference_omitted(position).into());
            };
            let Some(bound) = self.as_variable_reference(argument) else {
                let found = self.to_text(argument).to_vec();
                return Err(Raised::not_a_variable_reference(position, &found).into());
            };
            let reference = bound.name.clone();
            let slot = bound.home.clone();
            let name = self.use_target_name(code, target)?;
            // **The kinds must match, and the check is before the
            // uninitialised one.** Measured: a target that is both
            // kind-mismatched and already assigned reports the kind error,
            // not 98.995. Compound is not a third kind to handle -- `>p.1`
            // and `>q.1` are both rejected by `rexx-parse` (20.930/20.931),
            // so each side is a simple variable or a stem and nothing else.
            let target_is_stem = shape_of(&name) == NameShape::Stem;
            let reference_is_stem = shape_of(&reference) == NameShape::Stem;
            if target_is_stem != reference_is_stem {
                // Both substitute the *caller's* name, unlike 98.995 just
                // below, which names the target. Measured with a variable
                // whose value differs from its name, so the two cannot be
                // confused: `p = 'value-not-name'` passed as `>p` reports
                // `found "P"`.
                return Err(if target_is_stem {
                    Raised::not_a_stem_variable_reference(position, &reference).into()
                } else {
                    Raised::not_a_simple_variable_reference(position, &reference).into()
                });
            }
            let index = self.slot_of(&name);
            let frame = self.activation().frame;
            // The target must be **currently unset**. `RootSet::slot`
            // resolves through any alias already in force, which is what the
            // repeat case needs: after one `use arg >q`, `Q` reads the
            // caller's variable, so it "has a value" and the second attempt
            // is refused.
            if !self.target_is_uninitialised(&name, frame, index) {
                return Err(Raised::variable_reference_not_uninitialised(&name).into());
            }
            match slot {
                VarRefHome::Cell(cell) => self.roots.alias_slot(frame, index, cell),
                // The same binding `EXPOSE` makes, on this activation's own
                // slot: the target names the caller's object variable rather
                // than any frame storage, so there is nothing to alias to.
                VarRefHome::Instance { owner, scope } => {
                    let var = InstanceVar {
                        owner,
                        scope,
                        name: reference.clone(),
                    };
                    let activation = self.activation_mut();
                    match activation.exposed.iter_mut().find(|(at, _)| *at == index) {
                        Some(bound) => bound.1 = var,
                        None => activation.exposed.push((index, var)),
                    }
                }
            }
            // `>R>`, the alias's own line and the **only** trace line this
            // branch emits: no `>>>` and no `>=>`, because nothing was
            // evaluated and nothing was assigned (`UseInstruction.cpp:164`-
            // `167`, `aliasVariable` then `traceVariableAlias`, and
            // `handleArgument` `return`s before its own `traceResult` for
            // this case). Caller's name first, target's second -- see
            // `trace_alias`.
            self.trace_alias(self.clause_state.current_value_indent, &reference, &name);
            return Ok(());
        }

        // Present: bind the value. Absent: the default if there is one, and
        // otherwise drop the target -- measured, an absent target does not
        // keep whatever it held before.
        let value = match argument {
            Some(argument) => Some(argument),
            None => match &target.default {
                Some(default) => {
                    let value = self.eval(code, default)?;
                    self.roots.push_temp(value);
                    Some(value)
                }
                None => None,
            },
        };
        let name = self.use_target_name(code, target)?;
        match value {
            Some(value) => {
                // `>>>` then `>=>`, in that order and both at this `USE`
                // clause's own indent -- `handleArgument`'s own
                // `traceResult(argument)` immediately before
                // `retriever->assign(context, argument)`, whose own
                // `traceAssignment` is the second line
                // (`UseInstruction.cpp:74`-`77`, and the default-value arm
                // ten lines below it does the identical pair). Measured
                // under `trace r`: `use arg a, b` on a two-argument call
                // traces `>>>     "1"` and `>>>     "2"` and no `>=>`, which
                // is the gating -- `>>>` is `results`, `>=>` is
                // `intermediates`, so the pair is not one line's worth of
                // conditional.
                let indent = self.clause_state.current_value_indent;
                // `results` and not `intermediates`, though the pair below
                // needs both: `results` is the weaker gate, true wherever
                // `intermediates` is, so this renders for either line and
                // drops neither.
                let rendered = self.result_text(value);
                if let Some(rendered) = &rendered {
                    self.trace_result(indent, rendered);
                }
                self.assign_by_name(&name, value);
                if let Some(rendered) = &rendered {
                    self.trace_assignment(indent, &name, rendered);
                }
                Ok(())
            }
            // `USE STRICT ARG` refuses an omitted position that has no
            // default of its own, where `USE ARG` drops the target
            // (`UseInstruction.cpp:97`-`:111`). The position is the target's,
            // not the last one supplied -- measured, oracle rc 216, `call r
            // , 2` into `use strict arg a, b` reports `argument 1`.
            None if strict => Err(if in_method {
                Raised::missing_method_argument(position).into()
            } else {
                let call = self.call_context.name.clone();
                Raised::missing_argument(&call, position).into()
            }),
            None => {
                self.drop_by_name(&name);
                Ok(())
            }
        }
    }

    /// Whether a `USE ARG >name` target is in the uninitialised state the
    /// oracle requires of it.
    fn target_is_uninitialised(&self, name: &[u8], frame: SlotFrame, index: usize) -> bool {
        match self.variable(frame, index) {
            None => true,
            Some(value) => shape_of(name) == NameShape::Stem && self.is_uninitialised_stem(value),
        }
    }

    /// One `USE ARG` target's variable name.
    fn use_target_name(&self, code: &Code<'_>, target: &UseTarget) -> Result<Vec<u8>, Failure> {
        match &target.target.kind {
            ExprKind::Variable(id) | ExprKind::Stem(id) | ExprKind::Compound(id) => {
                Ok(code.symbols.name(*id).as_bytes().to_vec())
            }
            other => Err(Loud::expression(other).into()),
        }
    }

    /// Everything one `SAY` does once its expression has been evaluated:
    /// `>>>`, then the line itself.
    #[inline]
    pub(crate) fn say_evaluated(&mut self, value: Option<ObjRef>) -> Result<(), Failure> {
        let line = match value {
            Some(value) => {
                self.roots.push_temp(value);
                let value = self.required_string_value(value)?;
                self.to_text(value).to_vec()
            }
            None => Vec::new(),
        };
        self.trace_result(self.clause_state.current_value_indent, &line);
        self.out.extend_from_slice(&line);
        self.out.push(b'\n');
        Ok(())
    }

    /// `GUARD ON`/`GUARD OFF`, with or without a `WHEN` expression
    /// (`RexxInstructionGuard::execute`, `instructions/GuardInstruction.cpp`).
    fn exec_guard(&mut self, code: &Code<'_>, guard: &Guard) -> Result<Flow, Failure> {
        if self.activation().method_identity.is_none() {
            return Err(Raised::guard_outside_method().into());
        }
        let Some(condition) = &guard.condition else {
            return Ok(Flow::Next);
        };
        let holds = self.eval_condition(
            code,
            condition,
            ConditionTrace::Keyword(self.clause_state.current_value_indent, "WHEN"),
            raised_guard_not_logical,
        )?;
        if holds {
            Ok(Flow::Next)
        } else {
            Err(Loud::guard_when_false().into())
        }
    }

    /// `REPLY`, bare or with a value (`RexxInstructionReply::execute`,
    /// `instructions/ReplyInstruction.cpp:66`).
    fn exec_reply(
        &mut self,
        code: &Code<'_>,
        index: usize,
        expression: Option<&Expr>,
    ) -> Result<Flow, Failure> {
        if self.activation().method_identity.is_none() {
            return Err(Raised::reply_outside_method().into());
        }
        if !top_level_clause(code.body, index) {
            return Err(Loud::reply_inside_construct().into());
        }
        let value = match expression {
            Some(expression) => Some(self.eval(code, expression)?),
            None => None,
        };
        // The rooting and the `>>>` line are `RETURN`'s, for the same reason
        // and at the same indent: `evaluateExpression` is the shared call in
        // the C++ and this is the shared call here.
        if let Some(value) = value {
            self.roots.push_temp(value);
            if let Some(rendered) = self.result_text(value) {
                self.trace_result(self.clause_state.current_value_indent, &rendered);
            }
        }
        if self.activation().reply != ReplyState::None {
            return Err(Raised::reply_twice().into());
        }
        let activation = self.activation_mut();
        activation.reply = ReplyState::Owed;
        activation.replied_a_value = value.is_some();
        activation.pc = index + 1;
        Ok(Flow::Return(value))
    }

    /// `FORWARD`, with any of `TO`, `MESSAGE`, `CLASS`, `ARGUMENTS`, `ARRAY`
    /// and `CONTINUE` (`RexxInstructionForward::execute`,
    /// `instructions/ForwardInstruction.cpp:128`).
    fn exec_forward(&mut self, code: &Code<'_>, forward: &Forward) -> Result<Flow, Failure> {
        let Some(identity) = self.activation().method_identity.as_ref() else {
            return Err(Raised::forward_outside_method().into());
        };
        let receiver = identity.receiver;
        let own_name = identity.name.clone();
        let indent = self.clause_state.current_value_indent;

        // The option order is the C++'s, and it is observable in the trace:
        // `TO`, `MESSAGE`, `CLASS`, then whichever of `ARGUMENTS` and `ARRAY`
        // is present. Measured under `trace i`, `forward to (t)
        // message('OTHER') array(1,2)` emits `>K> "TO"`, `>K> "MESSAGE"` and
        // `>K> "ARRAY"` in that order, with the `ARRAY` items' own `>A>`
        // lines ahead of its keyword line.
        let target = match &forward.to {
            None => receiver,
            Some(expr) => self.forward_keyword(code, expr, "TO")?,
        };
        let message = match &forward.message {
            None => own_name,
            Some(expr) => {
                let value = self.forward_keyword(code, expr, "MESSAGE")?;
                let text = self.required_string_value(value)?;
                self.to_text(text).to_ascii_uppercase().into_boxed_slice()
            }
        };
        let start_scope = match &forward.class {
            None => None,
            Some(expr) => {
                let value = self.eval(code, expr)?;
                self.roots.push_temp(value);
                // `_superClass->isInstanceOf(TheClassClass)`
                // (`ForwardInstruction.cpp:171`), reported with the same two
                // fixed substitutions a `~name:scope` override's own check
                // takes. Measured, `forward class (5) message('OTHER')` is
                // 88.914 at rc 168.
                if !self.heap.is_class(value) {
                    return Err(Raised::scope_override_not_a_class().into());
                }
                self.trace_forward_keyword("CLASS", value);
                Some(value)
            }
        };

        let (mut values, mark) = self.take_value_buffer();
        let evaluated = self.forward_arguments(code, forward, &mut values);
        // **After every option and before the send**, which is where
        // `RexxActivation::forward` asks it (`execution/RexxActivation.cpp:
        // 1367`-`:1369`): a non-continuing `FORWARD` answers the sender, and
        // a `REPLY` carrying a value has answered it already.
        let owed = evaluated.and_then(|()| self.forward_after_reply(forward));
        // `settings.setForwarded(true)` (`execution/RexxActivation.cpp:1372`):
        // after the 98.937 above and before the send below, so a condition
        // the send raises is not offered to this activation's own traps.
        // [`Activation::forwarded`] carries what that costs.
        if owed.is_ok() && !forward.continue_ {
            self.activation_mut().forwarded = true;
        }
        let caller = self.caller();
        let sent = owed
            .and_then(|()| self.validate_scope_override(target, start_scope))
            .and_then(|()| {
                self.send_message(target, &message, start_scope, &values[mark..], caller)
            });
        self.give_value_buffer(values, mark);
        let sent = sent?;

        if !forward.continue_ {
            return Ok(Flow::Return(sent));
        }
        let slot = self.reserved_result_slot();
        let frame = self.activation().frame;
        match sent {
            Some(value) => {
                self.roots.push_temp(value);
                if let Some(rendered) = self.result_text(value) {
                    self.trace_result(indent, &rendered);
                }
                self.set_variable(frame, slot, value);
            }
            None => self.clear_variable(frame, slot),
        }
        Ok(Flow::Next)
    }

    /// 98.937 for a non-continuing `FORWARD` under a `REPLY` that carried a
    /// value, which is the one legality question `FORWARD` asks that is not
    /// about `FORWARD` (`execution/RexxActivation.cpp:1367`-`:1369`).
    fn forward_after_reply(&self, forward: &Forward) -> Result<(), Failure> {
        if forward.continue_ || !self.activation().replied_a_value {
            return Ok(());
        }
        Err(Raised::exit_after_reply().into())
    }

    /// One `FORWARD` option that is a single expression: its value, rooted,
    /// with the `>K>` line the oracle's `traceKeywordResult` writes.
    fn forward_keyword(
        &mut self,
        code: &Code<'_>,
        expr: &Expr,
        keyword: &str,
    ) -> Result<ObjRef, Failure> {
        let value = self.eval(code, expr)?;
        self.roots.push_temp(value);
        self.trace_forward_keyword(keyword, value);
        Ok(value)
    }

    /// One `FORWARD` option's `>K>` line, for a caller that has to evaluate
    /// and trace at separate points.
    fn trace_forward_keyword(&mut self, keyword: &str, value: ObjRef) {
        let traced = self.string_value_text(value);
        self.trace_keyword(self.clause_state.current_value_indent, keyword, &traced);
    }

    /// The argument list a `FORWARD` sends, into a borrowed buffer so that
    /// [`Interp::exec_forward`] returns it on the failure path too.
    fn forward_arguments(
        &mut self,
        code: &Code<'_>,
        forward: &Forward,
        values: &mut Vec<Option<ObjRef>>,
    ) -> Result<(), Failure> {
        if let Some(expr) = &forward.arguments {
            let value = self.forward_keyword(code, expr, "ARGUMENTS")?;
            self.forward_arguments_converted(value, values)?;
            while values.last().is_some_and(Option::is_none) {
                values.pop();
            }
            return Ok(());
        }
        if let Some(items) = &forward.array {
            for item in items {
                match item {
                    None => {
                        self.trace_argument(self.clause_state.current_value_indent, b"");
                        values.push(None);
                    }
                    Some(expr) => {
                        values.push(Some(self.eval_traced_argument(code, expr)?));
                    }
                }
            }
            // The line renders the instruction's own array of expressions,
            // which has no evaluated counterpart here. Measured under
            // `trace i`, `forward message('OTHER') array(1,2)` writes
            // `>K>   "ARRAY" => "an Array"` after the items' own `>A>` lines.
            self.trace_keyword(
                self.clause_state.current_value_indent,
                "ARRAY",
                crate::dispatch::ARRAY_DEFAULT_NAME,
            );
            return Ok(());
        }
        values.extend(self.call_context.arguments.iter().copied());
        // Rooted here rather than relied on through `call_context`, which the
        // collector does not walk.
        for value in values.iter().flatten() {
            self.roots.push_temp(*value);
        }
        Ok(())
    }

    /// `ARGUMENTS expr`'s value through `requestArray`
    /// (`instructions/ForwardInstruction.cpp:182`), appended to `values`.
    fn forward_arguments_converted(
        &mut self,
        value: ObjRef,
        values: &mut Vec<Option<ObjRef>>,
    ) -> Result<(), Failure> {
        match self.forward_arguments_conversion(value) {
            Conversion::Array => {
                let slots = self.array_slots_of(value).unwrap_or_default();
                values.extend(slots);
            }
            Conversion::Lines => {
                let text = self.to_text(value).into_owned();
                for line in makearray_lines(&text) {
                    self.push_converted_argument(line, values);
                }
            }
            Conversion::Tails => {
                for tail in self.stem_assigned_tails(value) {
                    self.push_converted_argument(&tail, values);
                }
            }
            Conversion::Refused => return Err(Raised::forward_arguments().into()),
            Conversion::NotBuilt(kind) => {
                return Err(Loud::object_position("FORWARD ARGUMENTS", kind).into());
            }
        }
        Ok(())
    }

    /// Which arm of `requestArray` this value takes.
    fn forward_arguments_conversion(&mut self, value: ObjRef) -> Conversion {
        match value.decode() {
            Decoded::Nil => return Conversion::Refused,
            // `RexxInteger::makeArray` and `NumberString::makeArray` both
            // hand the work to their string value's.
            Decoded::SmallInt(_) | Decoded::Text(_) => return Conversion::Lines,
            Decoded::Heap { .. } => {}
        }
        match self.heap.get(value).map(|object| &object.body) {
            // A class object is a primitive with no `makeArray` of its own,
            // so `requestArray` stops at `TheNilObject`.
            Some(Body::Class { .. }) => Conversion::Refused,
            // A multi-dimensional array shares `TheNilObject`'s raise rather
            // than converting (`instructions/ForwardInstruction.cpp:189`-
            // `:191`).
            Some(Body::Array { .. }) => {
                if self.is_multi_dimensional_array(value) {
                    Conversion::Refused
                } else {
                    Conversion::Array
                }
            }
            Some(Body::Text { .. } | Body::Num { .. }) => Conversion::Lines,
            // `StemClass::makeArray` is `tailArray`, which is the assigned
            // tails and never the default: measured, `a. = 'dflt'` with no
            // tail assigned forwards no arguments at all.
            Some(Body::Stem { .. }) => Conversion::Tails,
            // `requestArray` sends `REQUEST('ARRAY')` for a non-primitive,
            // which looks `MAKEARRAY` up in the behaviour and sends it, and
            // otherwise answers `.nil` (`RexxObject::requestRexx`,
            // `classes/ObjectClass.cpp:1920`-`:1940`).
            Some(Body::Instance { .. }) => {
                if self.answers_message(value, "MAKEARRAY") {
                    Conversion::NotBuilt("an instance of a user class")
                } else {
                    Conversion::Refused
                }
            }
            // **Not the referent's conversion**, which is what
            // `~request('ARRAY')` answers and is a different route:
            // `requestArray` looks `MAKEARRAY` up in the receiver's own
            // behaviour rather than sending it, and a reference's behaviour
            // holds no such name, so the lookup fails and `TheNilObject` is
            // the answer. Measured, oracle rc 158: `forward arguments (>v)`
            // over a `v` holding `'val'` is `98.946`, where the same
            // instruction over `v` itself is rc 0 -- so this arm may not
            // chase the way the string conversion beside it does.
            Some(Body::VarRef(_)) => Conversion::Refused,
            Some(Body::Native(_) | Body::WeakRef(_)) | None => {
                Conversion::NotBuilt("one of the interpreter's own objects")
            }
        }
    }

    /// One converted `ARGUMENTS` item, rooted as it is appended.
    fn push_converted_argument(&mut self, bytes: &[u8], values: &mut Vec<Option<ObjRef>>) {
        let item = self.text(bytes);
        self.roots.push_temp(item);
        values.push(Some(item));
    }

    /// A stem's assigned tails, ordered by `CompoundVariableTail::compare`
    /// (`classes/support/CompoundVariableTail.hpp:170`), which sorts on
    /// length first and bytes second.
    fn stem_assigned_tails(&self, value: ObjRef) -> Vec<Vec<u8>> {
        let Some(Body::Stem { tails, .. }) = self.heap.get(value).map(|object| &object.body) else {
            return Vec::new();
        };
        let mut names: Vec<Vec<u8>> = tails
            .iter()
            .filter(|(_, (_, held))| held.is_some())
            .map(|(name, _)| name.clone())
            .collect();
        names.sort_by(|left, right| left.len().cmp(&right.len()).then_with(|| left.cmp(right)));
        names
    }

    /// Everything a `RETURN` or an `EXIT` does once its expression has been
    /// evaluated: its `>>>`, and the `Flow` that leaves the activation.
    pub(crate) fn returned_value(
        &mut self,
        value: Option<ObjRef>,
        keyword: ReturnKeyword,
    ) -> Result<Flow, Failure> {
        if let Some(value) = value {
            self.roots.push_temp(value);
            if let Some(rendered) = self.result_text(value) {
                self.trace_result(self.clause_state.current_value_indent, &rendered);
            }
        }
        // **LEGALITY, and Phase 6 keeps it.** A `REPLY` has already answered
        // the sender, so a value here has nobody to go to whatever activity
        // the rest of the body runs on: 98.936 for `RETURN`, 98.937 for
        // `EXIT`. Both are
        // asked **after** the trace line above, which is the order the C++
        // takes them in -- `RexxInstructionReturn::execute` evaluates through
        // `evaluateExpression` and only then calls `returnFrom`
        // (`instructions/ReturnInstruction.cpp:72`), whose check is at
        // `execution/RexxActivation.cpp:1074`; `exitFrom`'s own is at
        // `:1413`. The bare form of either is legal after a reply and is
        // measured: `reply 'v'` then `say 'tail'` then `return` is rc 0 with
        // both lines printed and an empty stderr.
        if value.is_some() && self.activation().reply != ReplyState::None {
            return Err(match keyword {
                ReturnKeyword::Return => Raised::return_after_reply(),
                ReturnKeyword::Exit => Raised::exit_after_reply(),
            }
            .into());
        }
        Ok(match keyword {
            ReturnKeyword::Return => Flow::Return(value),
            ReturnKeyword::Exit => Flow::Exit(value),
        })
    }

    /// Everything a `PUSH` or a `QUEUE` does once its expression has been
    /// evaluated: the `>>>` line, and the line itself onto one end of the
    /// queue.
    pub(crate) fn queue_evaluated(
        &mut self,
        value: Option<ObjRef>,
        keyword: QueueKeyword,
    ) -> Result<(), Failure> {
        let line = match value {
            Some(value) => {
                self.roots.push_temp(value);
                // `evaluateStringExpression` again, so the `>>>` below traces
                // the conversion: measured, `trace r` over `push .K` with a
                // class-side `makeString` returning `'pv'` prints
                // `>>>   "pv"` and `pull` reads `PV`.
                let value = self.required_string_value(value)?;
                self.to_text(value).to_vec()
            }
            None => Vec::new(),
        };
        self.trace_result(self.clause_state.current_value_indent, &line);
        match keyword {
            QueueKeyword::Push => self.queue.push(line),
            QueueKeyword::Queue => self.queue.queue(line),
        }
        Ok(())
    }

    /// Everything one assignment does once its value has been evaluated:
    /// `>>>`, then the write and the lines the write itself produces.
    #[inline]
    pub(crate) fn assign_evaluated(
        &mut self,
        code: &Code<'_>,
        target: &Expr,
        value: ObjRef,
        at: Option<usize>,
    ) -> Result<(), Failure> {
        self.roots.push_temp(value);
        // `>>>` fires before the assignment itself
        // (`RexxInstructionAssignment::execute`: evaluate, trace, *then*
        // assign), which matters only in that the traced value can never be
        // affected by the write it precedes.
        // Reads `current_value_indent` rather than recomputing
        // `static_indent(index)` independently -- the clause unit already
        // computed exactly this value (`indent_offset` included, F-EX1's own
        // correction to F3) for this same instruction right before the clause
        // ran, and a second computation of the identical quantity is how the
        // two drift, which is exactly what happened here before this fix: this
        // site's own copy never learned about the offset when the field was
        // added.
        let indent = self.clause_state.current_value_indent;
        // One render for both lines, and `results` is the gate because it is
        // the weaker of the two: `>>>` is gated on `results` and `>=>` on
        // `intermediates`, and `results` is true wherever `intermediates` is.
        // Guarding on `intermediates` instead would drop the `>>>` line under
        // `TRACE R`.
        let rendered = self.result_text(value);
        if let Some(rendered) = &rendered {
            // **The entry gate, not just the setting in force.** An assignment
            // whose expression turned tracing on owes no `>>>`, because the
            // oracle chose the path without one before it evaluated. See
            // `ClauseState::instructions_traced_at_entry`, which carries the
            // measurement.
            if self.clause_state.instructions_traced_at_entry {
                self.trace_result(indent, rendered);
            }
        }
        self.assign_expr_target(code, target, value, rendered.as_deref(), indent, at)
    }

    /// Writes `value` through one assignment *target expression*, and traces
    /// the write.
    pub(crate) fn assign_expr_target(
        &mut self,
        code: &Code<'_>,
        target: &Expr,
        value: ObjRef,
        rendered: Option<&[u8]>,
        indent: usize,
        at: Option<usize>,
    ) -> Result<(), Failure> {
        match &target.kind {
            ExprKind::Variable(id) => {
                let name = code.symbols.name(*id).as_bytes();
                let slot = match at {
                    Some(slot) => slot,
                    None => self.slot_of(name),
                };
                let frame = self.activation().frame;
                self.set_variable(frame, slot, value);
                if let Some(rendered) = rendered {
                    self.trace_assignment(indent, name, rendered);
                }
            }
            // `stem. = expr`: replace-and-rebind (D15a), through the
            // library `stem_assign` already builds -- this arm is the
            // dispatch, not new stem logic.
            ExprKind::Stem(id) => {
                // The tripwire `crate::ir::drive`'s `Op::Store` arm carries
                // for the compiled side, here where **both** engines pass:
                // a slot handed to this arm was resolved against the symbol's
                // own id and is silently shadowed below, so a caller that
                // started supplying one would write through the entry's slot
                // and never learn that its own was ignored.
                debug_assert!(
                    at.is_none(),
                    "a stem write was handed a slot, and the slot it writes comes from the entry"
                );
                let name = code.symbols.name(*id).as_bytes();
                let at = code.compound(*id).and_then(|entry| entry.stem_at);
                self.stem_assign_at(name, at, value);
                if let Some(rendered) = rendered {
                    self.trace_assignment(indent, name, rendered);
                }
            }
            // `a.b = expr`: resolve the tail key the same way reading
            // `a.b` would (`eval_node`'s own `Compound` arm), then
            // mutate that one tail in place through `stem_set`.
            ExprKind::Compound(id) => {
                // `read_symbol`'s own compound tripwire, on the writing
                // side. A compound-shaped name **can** reach
                // `Plan::by_symbol`: `Plan::bind` puts every name it binds
                // whole there, and `note_loop` and `note_parse` both call it
                // with spellings that may be compound-shaped. So a caller
                // reaching for a slot by this symbol's id can find one, and it
                // is not the slot this arm writes.
                debug_assert!(
                    at.is_none(),
                    "a compound write was handed a slot, and the slot it writes is the stem's"
                );
                // **Borrowed, not copied.** `Code::stem` and `SymbolTable::
                // name` both answer with the lifetime of the `Code`, which is
                // the program rather than this `Interp`, so neither needs an
                // owned copy to survive the `&mut self` calls below. The
                // `Variable` arm above has always passed its name borrowed;
                // this arm copied both, and measured with `heaptrack` on
                // `samples/rexxcps.rex` that cost two allocations per compound
                // write.
                let tag = code.symbols.name(*id).as_bytes();
                let (stem_name, stem_at) = code.stem(*id);
                let mut key = self.take_key_buffer();
                if let Err(failure) = self.tail_key_into(code, *id, &mut key) {
                    self.give_key_buffer(key);
                    return Err(failure);
                }
                self.stem_set_at(stem_name, stem_at, &key, value);
                // **The resolved name is built only when a line will print
                // it.** `trace_compound_name` returns at once unless
                // intermediates are on, so joining the stem to the tail key
                // ahead of that check allocated a name to discard. This is the
                // rule `rendered` below already follows -- `trace.rs`'s own
                // doc comment gives the reasoning for the value half, and the
                // name half is the same argument.
                if self.tracing_intermediates() {
                    let mut resolved = stem_name.to_vec();
                    resolved.extend_from_slice(&key);
                    self.trace_compound_name(indent, tag, &resolved);
                }
                if let Some(rendered) = rendered {
                    self.trace_assignment(indent, tag, rendered);
                }
                self.give_key_buffer(key);
            }
            other => return Err(Loud::expression(other).into()),
        }
        Ok(())
    }

    /// Assigns `value` to the variable, whole stem, or one verbatim-keyed
    /// tail that `name`'s own spelling names.
    fn assign_by_name(&mut self, name: &[u8], value: ObjRef) {
        match shape_of(name) {
            NameShape::Simple => {
                let slot = self.slot_of(name);
                let frame = self.activation().frame;
                self.set_variable(frame, slot, value);
            }
            NameShape::Stem => self.stem_assign(name, value),
            NameShape::Compound => {
                let dot = name
                    .iter()
                    .position(|&b| b == b'.')
                    .expect("NameShape::Compound guarantees at least one period");
                let (stem_name, key) = name.split_at(dot + 1);
                self.stem_set(stem_name, key, value);
            }
        }
    }

    /// `SIGL`, set at the point of every control transfer -- `SIGNAL`'s own
    /// two `step` arms and `Interp::invoke_call` (`CALL`, and `ExprKind::
    /// Call` through `eval_call`, `eval.rs`) -- to `line`, always `self.
    /// current_clause_line` at the call site (`lib.rs`'s own doc comment on
    /// that field has why it is a field and not a parameter here).
    fn set_sigl(&mut self, line: usize) {
        let value = self.counted_text(line);
        // Not through `assign_by_name`: that reads the name's shape and then
        // hashes it, and `SIGL` is a simple name whose slot the plan already
        // holds. The fallback covers a plan with no name map at all.
        let slot = match self.activation().plan.sigl_slot {
            Some(slot) => slot,
            None => self.slot_of(b"SIGL"),
        };
        let frame = self.activation().frame;
        self.set_variable(frame, slot, value);
    }

    /// The slot `RESULT` lives in, from the plan when it has one.
    fn reserved_result_slot(&mut self) -> usize {
        match self.activation().plan.result_slot {
            Some(slot) => slot,
            None => self.slot_of(b"RESULT"),
        }
    }

    /// `SIGNAL ON`/`OFF` and `CALL ON`/`OFF`, which are one instruction with
    /// one flag between them.
    pub(crate) fn exec_condition_trap(
        &mut self,
        trap: &ConditionTrap,
        call: bool,
    ) -> Result<Flow, Failure> {
        match &trap.label {
            Some(label) => {
                // The required-string protocol's other arming route: a
                // NOSTRING trap is what turns an object with no string value
                // from a rendering into a raise. `ANY` counts, measured --
                // `signal on any` over `say .environment` runs the handler
                // with `CONDITION('C')` `NOSTRING`. See
                // `Interp::reqstr_armed` for why this only ever sets.
                if matches!(&trap.condition[..], b"NOSTRING" | b"ANY") {
                    self.reqstr_armed = true;
                }
                let entry = Trap {
                    call,
                    label: std::rc::Rc::from(label.as_ref()),
                    delayed: false,
                };
                self.activation_mut()
                    .traps
                    .insert(trap.condition.clone(), entry);
                // `RexxActivation::trapOn`'s second half: arming a trap turns
                // off whatever `::OPTIONS ... SYNTAX` asked for the same
                // condition, so the trap takes it rather than a SYNTAX error
                // preempting the trap. `novalue_trapped` is false because the
                // guard is `trapOff`'s alone.
                self.activation_mut()
                    .condition_syntax
                    .disable_for(&trap.condition, !call, false);
            }
            None => {
                self.activation_mut().traps.remove(&trap.condition);
                // `RexxActivation::trapOff`'s second half, with the one guard
                // that arm carries and `trapOn`'s does not: a `NOVALUE` or
                // `ANY` trap still in the table keeps `NOVALUE`'s escalation
                // where it was.
                let traps = &self.activation().traps;
                let novalue_trapped = traps.get(b"NOVALUE".as_slice()).is_some()
                    || traps.get(b"ANY".as_slice()).is_some();
                self.activation_mut().condition_syntax.disable_for(
                    &trap.condition,
                    !call,
                    novalue_trapped,
                );
            }
        }
        Ok(Flow::Next)
    }

    /// The trap the running activation has enabled for `condition`, if any.
    pub(crate) fn trap_for(&self, condition: &[u8]) -> Option<Trap> {
        let traps = &self.trap_frame()?.traps;
        traps
            .get(condition)
            .or_else(|| traps.get(b"ANY".as_slice()))
            // A trap held while its own `CALL ON` handler runs does not fire
            // -- see `Trap::delayed`, which is what a handler that calls
            // something raising the same condition depends on.
            .filter(|trap| !trap.delayed)
            .cloned()
    }

    /// Turns an uninitialised variable read into a `NOVALUE` condition --
    /// but only when this activation has a `NOVALUE` trap that could take
    /// it.
    #[inline(always)]
    pub(crate) fn novalue_check(&self, novalue: Novalue, read: ObjRef) -> Result<(), Failure> {
        if novalue == Novalue::Set {
            return Ok(());
        }
        self.novalue_raised(read)
    }

    /// [`Interp::novalue_check`]'s uninitialised half: whether the setting in
    /// force turns this read into a raised `NOVALUE`, into
    /// `::OPTIONS NOVALUE SYNTAX`'s 98.986, or into neither -- the derived
    /// name `read_at` already produced.
    #[cold]
    #[inline(never)]
    fn novalue_raised(&self, read: ObjRef) -> Result<(), Failure> {
        if self.trap_for(b"NOVALUE").is_none_or(|trap| trap.call) {
            if self.condition_raises_syntax(b"NOVALUE") {
                return Err(Raised::unassigned_variable(&self.derived_name_text(read)).into());
            }
            return Ok(());
        }
        Err(Raised::condition(Cow::Borrowed("NOVALUE")).into())
    }

    /// The bytes of the derived name an uninitialised read answered.
    fn derived_name_text(&self, read: ObjRef) -> Vec<u8> {
        match read.decode() {
            rexx_core::Decoded::Text(inline) => inline.to_vec(),
            rexx_core::Decoded::Heap { .. } => match self.heap.get(read).map(|object| &object.body)
            {
                Some(rexx_core::Body::Text { bytes, .. }) => bytes.to_vec(),
                other => {
                    debug_assert!(
                        false,
                        "an uninitialised read answered something other than a string: {other:?}"
                    );
                    Vec::new()
                }
            },
            other => {
                debug_assert!(
                    false,
                    "an uninitialised read answered something other than a string: {other:?}"
                );
                Vec::new()
            }
        }
    }

    /// Raises 98.972 where `::OPTIONS LOSTDIGITS SYNTAX` is in force and
    /// `operand` really does carry more digits than the precision in force.
    #[inline(always)]
    pub(crate) fn lostdigits_check(
        &mut self,
        operand: &Number,
        value: ObjRef,
    ) -> Result<(), Failure> {
        if !self.lostdigits_armed {
            return Ok(());
        }
        self.lostdigits_raise(operand, value)
    }

    /// The two-operand form, so an operator pays the gate once.
    #[inline(always)]
    pub(crate) fn lostdigits_check2(
        &mut self,
        left: &Number,
        left_value: ObjRef,
        right: &Number,
        right_value: ObjRef,
    ) -> Result<(), Failure> {
        if !self.lostdigits_armed {
            return Ok(());
        }
        self.lostdigits_raise(left, left_value)?;
        self.lostdigits_raise(right, right_value)
    }

    /// [`Interp::lostdigits_check`]'s armed half: the per-activation setting,
    /// then the operand's own digit count, then the operand's bytes.
    #[cold]
    #[inline(never)]
    fn lostdigits_raise(&mut self, operand: &Number, value: ObjRef) -> Result<(), Failure> {
        if !self.condition_raises_syntax(b"LOSTDIGITS") {
            return Ok(());
        }
        let digits = usize::try_from(self.activation().settings.digits()).unwrap_or(usize::MAX);
        if operand.digit_count() > digits {
            let text = self.string_value_text(value);
            return Err(Raised::lostdigits(&text).into());
        }
        Ok(())
    }

    /// Offers a failure escaping the running activation to that activation's
    /// trap table, and either transfers control or hands the failure back to
    /// keep unwinding.
    pub(crate) fn offer_to_trap(
        &mut self,
        code: &Code<'_>,
        failure: Failure,
    ) -> Result<Flow, Failure> {
        let Failure::Raised(raised) = &failure else {
            return Err(failure);
        };
        match raised.delivery.search {
            Search::Here => {}
            // One level up, and this is that level's turn to decline. The
            // rewrite is what makes the *next* loop out offer it: without
            // it, `Caller` would skip every activation rather than one.
            Search::Caller => {
                let Failure::Raised(mut raised) = failure else {
                    unreachable!("matched Failure::Raised immediately above")
                };
                raised.delivery.search = Search::Here;
                return Err(Failure::Raised(raised));
            }
            // The outermost activation is the only one allowed to look.
            Search::Top if self.activation_depth() > 1 => return Err(failure),
            Search::Top => {}
            Search::Nobody => return Err(failure),
        }
        // A phantom does not trap. `RexxActivation::trap`
        // (`execution/RexxActivation.cpp:2450`) reads `isForwarded` before it
        // looks at any trap table, and this crate reaches the frame it drills
        // to by declining here and letting the failure leave this activation.
        // [`Interp::trap_for`] does the drilling for the callers that ask
        // whether a condition would be trapped at all, which is the same
        // question `RexxActivation::willTrap` answers.
        if self.running_activation().is_some_and(|a| a.forwarded) {
            return Err(failure);
        }
        let Some(trap) = self.trap_for(raised.condition.as_bytes()) else {
            return Err(failure);
        };
        if trap.call {
            return Err(failure);
        }
        // Resolved **before** anything is cleared or removed, because a
        // label that does not exist is not a trap that fired: measured,
        // `signal on syntax name nosuchlabel` with `say 1/0` on line 3
        // reports `Error 16.1 Label "NOSUCHLABEL" not found` against line 3
        // -- the raising clause's own site, which is still the one
        // `Op::Clause`'s region recorded a moment ago and which the clearing
        // below would have thrown away.
        let target = self.resolve_signal_target(&trap.label)?;
        let Failure::Raised(raised) = failure else {
            unreachable!("matched Failure::Raised immediately above")
        };
        // Removed when it fires (`Activation::traps`' own doc comment has
        // the two probes). Removed by the condition's *own* name and by
        // `ANY`, since `trap_for` may have matched either and leaving the
        // one that matched enabled would re-trap.
        let traps = &mut self.activation_mut().traps;
        traps.remove(raised.condition.as_bytes());
        traps.remove(b"ANY".as_slice());
        // **Inherited item I11, and the reason it is this task's.** Both
        // halves of the echo stack are dropped: a trapped condition prints
        // no report at all, so the sites it accumulated must not survive to
        // be printed against a *later*, untrapped one. Measured -- `say 1/0`
        // trapped on line 3 and `say 2/0` untrapped on line 8 inside the
        // handler reports line 8, alone, and a version that kept the first
        // site reports line 3.
        let site = self.failure_site.take();
        let sites = std::mem::take(&mut self.failure_sites);
        self.set_sigl(self.clause_state.line());
        if let Some(rc) = &raised.rc {
            let value = self.text(rc);
            self.assign_by_name(b"RC", value);
        }
        // What `CONDITION()` reports for the rest of this activation.
        // Written here and not onto `active_condition` below, because the
        // two have different lifetimes: this one dies with the activation
        // (`TrappedCondition`), while `active_condition` is the interpreter's
        // one slot for `RAISE PROPAGATE`.
        self.activation_mut().condition = Some(TrappedCondition {
            name: raised.condition.as_bytes().into(),
            // Only a `SYNTAX` condition has a `CODE` item at all
            // (`Activity::createExceptionObject` is the one place it is put
            // in), so the name and not `reportable()` is the test: measured,
            // a trapped `HALT` reports `E` as the null string, and `HALT` is
            // the one non-`SYNTAX` condition this crate numbers.
            code_sub: (raised.condition == "SYNTAX").then_some(raised.sub),
            call: false,
            description: raised.description.clone(),
        });
        // What a later `RAISE PROPAGATE` re-raises. See `exec_raise_
        // propagate` for what is and is not measured about it.
        self.active_condition = Some(ActiveCondition {
            raised: *raised,
            site,
            sites,
        });
        // **The failed clause's own boundary, and the one place it can
        // happen** (fix round 3). `Op::Clause`'s region ends a *completing*
        // clause; a clause that raised has not completed, and at that moment
        // nothing yet knows whether it will be trapped here or unwind the
        // activation. This is the point where that is decided in favour of
        // "trapped here, execution continues", so it is the point where a
        // `CALL ON` handler queued by the same clause is owed its run.
        if let Some(exit) = self.deliver_pending_traps(code)? {
            return Ok(Flow::Exit(exit.value()));
        }
        Ok(Flow::Signal(target))
    }

    /// Runs every handler this clause boundary owes, in the order the
    /// conditions were queued, and stops early only if one of them ends the
    /// program.
    pub(crate) fn deliver_pending_traps(
        &mut self,
        code: &Code<'_>,
    ) -> Result<Option<HandlerExit>, Failure> {
        // Snapshotted rather than re-read, so that anything a handler queues
        // lands beyond the prefix this boundary is answering for.
        let mut owed = self.pending_traps.len();
        while owed > 0 {
            // Only the activation whose trap table matched delivers, and only
            // once it is running again -- `PendingTrap::activation`'s own doc
            // comment has the three transcripts this identity check answers,
            // including the two a stack depth got wrong.
            let here = self.activation().id;
            // Both keys, and they answer different questions: the identity
            // says which activation's trap table matched, the depth says which
            // `INTERPRET` fragment's queue the condition is sitting in. A
            // fragment is an activation in the oracle and is not one here, so
            // the second is what the first cannot see.
            let depth = self.fragment_depth;
            let Some(at) =
                self.pending_traps.iter().take(owed).position(|pending| {
                    pending.activation == here && pending.fragment_depth == depth
                })
            else {
                return Ok(None);
            };
            let pending = self
                .pending_traps
                .remove(at)
                .expect("position answered an index inside the queue");
            owed -= 1;
            if let Some(exit) = self.deliver_one_pending_trap(code, pending)? {
                return Ok(Some(exit));
            }
        }
        Ok(None)
    }

    /// One queued condition's handler, run at the boundary that owes it.
    fn deliver_one_pending_trap(
        &mut self,
        code: &Code<'_>,
        pending: PendingTrap,
    ) -> Result<Option<HandlerExit>, Failure> {
        let Some(trap) = self.trap_for(&pending.condition) else {
            return Ok(None);
        };
        if !trap.call {
            // A `SIGNAL ON` trap never gets here: `exec_raise` throws for
            // that half instead, so the transfer happens where the raise is
            // rather than one clause later. Declining rather than asserting
            // keeps a future raiser that forgets the distinction from
            // silently running a `SIGNAL` handler as a call.
            return Ok(None);
        }
        self.set_sigl(self.clause_state.line());
        if let Some(rc) = &pending.rc {
            let value = self.text(rc);
            self.assign_by_name(b"RC", value);
        }
        // A `CALL ON` handler is running a condition too, and `RAISE
        // PROPAGATE` inside one asks for it -- measured, `raise propagate`
        // in a `CALL ON USER FOO` handler ends the program silently at rc 0,
        // where the same clause with no handler running at all is 98.918.
        // Recording nothing here would give the second answer for the first
        // program. No sites travel with it: nothing failed, so nothing was
        // cleared.
        let mut raised = Raised::condition(condition_name(&pending.condition));
        raised.rc = pending.rc.clone();
        // **Saved and restored, not cleared** (fix round 2's NEW 1). Round 1
        // set this back to `None` when the handler returned, which is right
        // only when nothing was active before -- and one clause can queue a
        // `CALL ON` condition *and* raise a `SIGNAL ON`-trapped one, so a
        // `SIGNAL` handler can be running when a `CALL` handler is delivered
        // inside it. Measured: `zq = sub() + 1/0` under both traps, with the
        // `SIGNAL` handler ending in `raise propagate`, is the original 42.3
        // at rc 214 on the oracle; clearing to `None` gave 98.918 at rc 158,
        // and never clearing at all gave silence at rc 0. Restoring gives the
        // oracle's answer in all three measured shapes, the "nothing was
        // active, restore `None`" one included.
        let enclosing = self.active_condition.take();
        self.active_condition = Some(ActiveCondition {
            raised,
            site: None,
            sites: Vec::new(),
        });
        let key: Box<[u8]> = pending.condition.clone();
        // **Delayed, not removed** (`Trap::delayed`). The two are the same
        // to every lookup that decides whether to trap, and different to
        // `CONDITION('S')` alone, which reports `DELAY` here and `OFF` for a
        // trap that is absent. Nothing else moves: an earlier version of
        // this comment claimed the flag also protects a handler's own `CALL
        // OFF`, and that is false -- the handler's table is a copy, so its
        // `CALL OFF` never reached this one to be undone.
        if let Some(trap) = self.activation_mut().traps.get_mut(&key) {
            trap.delayed = true;
        }
        // What `CONDITION()` answers inside the handler. Set on *this*
        // activation and restored afterwards, because the handler inherits
        // its copy at call time and the caller must be left as it was --
        // measured, `condition()` back in the caller after a `CALL ON`
        // handler returns is the null string.
        let enclosing_condition = self.activation_mut().condition.replace(TrappedCondition {
            name: key.clone(),
            // No condition reaching here has a `CODE`, and both halves of
            // that are measured. A `CALL ON` trap cannot name `SYNTAX`
            // directly -- `call on syntax` is a 25.1 translation error --
            // and `CALL ON ANY`, the one spelling that could smuggle it in,
            // does not catch a `SYNTAX` condition either: `call on any name
            // uh` with `say 1/0` is the ordinary fatal 42.3 at rc 214 on
            // both interpreters. `TrapHandler::canHandle` is the C++ side of
            // the same rule.
            code_sub: None,
            call: true,
            description: pending.description.clone(),
        });
        let queued_before = self.pending_traps.len();
        // `CallType::Subroutine` because a `CALL ON` handler is a `CALL`, and
        // that reaches `PARSE SOURCE` when the trap's name resolves to a
        // `::ROUTINE` rather than to a label. Measured: a trapped `USER`
        // condition whose handler is a `::routine` running `parse source`
        // answers `SUBROUTINE`, where the same handler written as a label
        // answers whatever the trapping activation answers.
        // **`CallEntry::Trap`, which is the one thing about a handler's own
        // activation that differs from an internal `CALL`'s**: the oracle's
        // `internalCallTrap` passes `OREF_NULL` where `internalCall` passes
        // the caller's receiver, so the handler's calling convention carries
        // none -- `entered_receiver` has the measurement for both.
        let ended = self.resolve_and_run_call(
            code,
            &trap.label,
            true,
            &[],
            CallType::Subroutine,
            CallEntry::Trap,
        );
        // A trap queued by the handler that just ran is not one the
        // interrupted clause owes, and `in_clause`'s tripwire has to be able
        // to tell the two apart -- see the field's own doc comment.
        for pending in self.pending_traps.iter_mut().skip(queued_before) {
            pending.queued_during_delivery = true;
        }
        self.activation_mut().condition = enclosing_condition;
        // `trapUndelay`. The `if let` mirrors the C++ testing the handler
        // for null before enabling it; nothing a Rexx program can do
        // removes the entry between here and the delay above, so the arm is
        // structural rather than a case anything reaches.
        if let Some(trap) = self.activation_mut().traps.get_mut(&key) {
            trap.delayed = false;
        }
        match ended {
            // The handler returned; execution resumes at the clause after
            // the one that finished.
            Ok(Ended::Returned(_)) => {
                self.active_condition = enclosing;
                Ok(None)
            }
            // The handler failed rather than returned. **Reachable but
            // unobservable, kept deliberately** (fix round 3). The
            // re-review's panic probe found four programs that take this
            // arm and no test that does, and established why nothing can see
            // it: every path that goes on to read `active_condition` passes
            // through `offer_to_trap` first, which overwrites the field
            // wholesale. So this line changes no output while that holds.
            Err(failure) => {
                self.active_condition = enclosing;
                Err(failure)
            }
            // `EXIT` inside the handler ends the program, exactly as it does
            // inside any other called routine. Nothing will read
            // `active_condition` again, so it is left as it is.
            Ok(exited @ Ended::Exited(_)) => Ok(HandlerExit::from_ended(exited)),
        }
    }

    /// The trap the running activation's **caller** has enabled, or `None`
    /// at top level.
    fn caller_trap_for(&self, condition: &[u8]) -> Option<Trap> {
        let traps = &self.caller_activation()?.traps;
        traps
            .get(condition)
            .or_else(|| traps.get(b"ANY".as_slice()))
            .cloned()
    }

    /// `RAISE`, in all of its forms.
    /// ```text
    /// RAISE SYNTAX n.m RETURN [e]   search from the raising activation outward
    /// RAISE SYNTAX n.m             \  the OUTERMOST activation's trap only;
    /// RAISE SYNTAX n.m EXIT [e]    /  every level in between skips its own
    /// RAISE other ... RETURN [e]      search from the raising activation's CALLER
    /// RAISE other ...              \  no trap at all -- the program ends, and
    /// RAISE other ... EXIT [e]     /  the condition's default action applies
    /// ```
    fn exec_raise(&mut self, code: &Code<'_>, raise: &Raise) -> Result<Flow, Failure> {
        if raise.propagate {
            return self.exec_raise_propagate();
        }
        // Each option traces a `>K>` line as it is evaluated, in source
        // order, at this clause's own indent. Measured, all five spellings:
        // ```text
        // raise syntax 40.4 description 'zdesc' additional 'zadd'
        //   >K>   "SYNTAX" => "40.4"
        //   >K>   "DESCRIPTION" => "zdesc"
        //   >K>   "ADDITIONAL" => "zadd"
        // raise syntax 40.4 array ('ZORKROUTINE', 7)
        //   >K>   "SYNTAX" => "40.4"
        //   >K>   "ARRAY" => "an Array"
        // raise user marker description 'zdesc' return 'zret'
        //   >K>   "DESCRIPTION" => "zdesc"
        //   >K>   "RESULT" => "zret"
        // ```
        let indent = self.clause_state.current_value_indent;
        let rc_text = match &raise.rc {
            Some(expr) => {
                let value = self.eval(code, expr)?;
                self.roots.push_temp(value);
                // **The line renders the object and the code is its string
                // value**, which `Interp::string_value_text`'s own doc has the
                // split for: `traceKeywordResult(conditionName, rc)`
                // (`instructions/RaiseInstruction.cpp:182`) is handed the
                // object and `rc->requestString()` (`:193`) is what becomes the
                // code. Measured, `trace i` over `raise syntax (1,2)`:
                // `>K>   "SYNTAX" => "an Array"`.
                let traced = self.string_value_text(value);
                let keyword = String::from_utf8_lossy(&raise.condition).into_owned();
                self.trace_keyword(indent, &keyword, &traced);
                Some(self.to_text(value).to_vec())
            }
            None => None,
        };
        // Kept, not only traced: a trapping handler reads it back through
        // `CONDITION('D')` -- measured, `raise syntax 40.4 description 'zd'`
        // trapped gives `zd` where the same raise without the clause gives
        // the null string.
        let mut description: Option<Vec<u8>> = None;
        if let Some(expr) = &raise.description {
            let value = self.eval(code, expr)?;
            self.roots.push_temp(value);
            // The object on the line and its string value in the condition,
            // the same split the `rc` keyword above takes. Measured,
            // `trace i` over `raise syntax 93.900 description (1,2)
            // additional 'x'`: `>K>   "DESCRIPTION" => "an Array"`.
            let traced = self.string_value_text(value);
            self.trace_keyword(indent, "DESCRIPTION", &traced);
            description = Some(self.to_text(value).to_vec());
        }
        // `ADDITIONAL expr` and `ARRAY (a, b)` produce the identical
        // substitution list -- measured, `additional ('MYROUTINE', 3)` and
        // `array ('MYROUTINE', 3)` give byte-identical reports -- so they
        // share one `Vec` here rather than being kept apart to no end. A
        // single non-array `ADDITIONAL` value is one substitution, also
        // measured: `additional 'JUSTONE'` fills `&1` and leaves `&2` as the
        // literal `&2`.
        let mut additional: Vec<Vec<u8>> = Vec::new();
        if let Some(expr) = &raise.additional {
            let value = self.eval(code, expr)?;
            self.roots.push_temp(value);
            // **A surface that is neither `stringValue()` nor an operator,
            // and only under one condition.** `RaiseInstruction::execute`
            // (`instructions/RaiseInstruction.cpp:270`-`290`) calls
            // `requestArray` on the additional information exactly once, and
            // only inside `if (errorCode->strCompare(SYNTAX))` -- so under
            // `USER` or any other condition the value is never
            // array-converted and this crate's rendering is the oracle's own
            // answer. Measured both ways: `raise syntax 40.1 additional
            // (.array)` is a 98 execution error at rc 158, `additional
            // (.environment)` substitutes `INPUTOUTPUTSTREAM` -- the first
            // entry of the array the directory converts to -- and `raise user
            // zork additional (.array)` under a trap is rc 0 on both sides.
            let converted = raise.condition.eq_ignore_ascii_case(b"SYNTAX");
            let slots = if converted {
                self.array_slots_of(value)
            } else {
                None
            };
            if converted
                && slots.is_none()
                && let Some(kind) = self.operator_operand_gap(value)
            {
                return Err(Loud::object_position("a RAISE ADDITIONAL value", kind).into());
            }
            // The object on the line, the same split the two keywords above
            // take. Measured, `trace i` over `raise syntax 93.900 additional
            // (1,2)`: `>K>   "ADDITIONAL" => "an Array"`.
            let traced = self.string_value_text(value);
            self.trace_keyword(indent, "ADDITIONAL", &traced);
            match slots {
                Some(slots) => {
                    for slot in slots {
                        additional.push(match slot {
                            Some(item) => self.string_value_text(item),
                            None => Vec::new(),
                        });
                    }
                }
                None => additional.push(self.to_text(value).to_vec()),
            }
        }
        if let Some(items) = &raise.array {
            // **The elements first, then the `>K>` line** -- corrected at
            // Task 9, which owns `>A>` and measured the ordering while
            // adding it. An earlier version of this arm traced the `>K>`
            // first and said so in a comment that claimed "the elements
            // produce no lines of their own"; both halves are false.
            // Measured, `trace i` / `raise syntax 40.4 array('R',,'X')`:
            // ```text
            //   >L>   "R"
            //   >A>   "R"
            //   >A>   "R"
            //   >A>   ""
            //   >L>   "X"
            //   >A>   "X"
            //   >A>   "X"
            //   >K>   "ARRAY" => "an Array"
            // ```
            for item in items {
                let Some(expr) = item else {
                    // An omitted position (`array (1,,3)`) **holds its
                    // place** in the substitution list rather than closing
                    // up, and substitutes as empty. Measured -- this used to
                    // `continue`, on a stated-as-unmeasured guess, and the
                    // guess was wrong: `raise syntax 40.4 array('R',,'X')`
                    // reports "maximum expected is ." on the oracle (`&2` is
                    // the hole) where closing up reported "maximum expected
                    // is X." here.
                    self.trace_argument(indent, b"");
                    additional.push(Vec::new());
                    continue;
                };
                let value = self.eval(code, expr)?;
                self.roots.push_temp(value);
                // **No gap check here, and the absence is the decision.**
                // `RaiseInstruction::execute` builds a real `ArrayClass` from
                // these elements (`RaiseInstruction.cpp:217`-`239`) and the
                // `requestArray` below it therefore gets an array already and
                // returns it unchanged -- the elements are never
                // array-converted, only rendered by the substitution
                // machinery. Measured: `array (.array)`, `array
                // (.environment)` and `array (.array, 'b')` are all rc 216
                // and byte-identical here. A check on this arm refused all
                // three.
                let rendered = self.string_value_text(value);
                self.trace_argument(indent, &rendered);
                self.trace_argument(indent, &rendered);
                additional.push(rendered);
            }
            // **`an Array`, verbatim and regardless of the elements** --
            // it is the Array class's own default string form, which is
            // what the oracle traces here (measured for `array
            // ('ZORKROUTINE', 7)`). This crate has no array object to render,
            // and building one purely to print a constant would be the
            // longer way to the same two words.
            self.trace_keyword(indent, "ARRAY", b"an Array");
        }
        // The `RETURN`/`EXIT` value, and its own `>K>` line -- measured for
        // both tails: `raise user foo return 'ONEVAL'` under `trace r`
        // traces `>K>     "RESULT" => "ONEVAL"`, and the same with `exit`
        // traces the identical line. `RETURN`'s own instruction arm traces
        // `>>>` instead, so this is not that path with a different indent.
        let result = match &raise.result {
            Some(result) => match &result.value {
                Some(expr) => {
                    let value = self.eval(code, expr)?;
                    self.roots.push_temp(value);
                    if let Some(rendered) = self.result_text(value) {
                        self.trace_keyword(indent, "RESULT", &rendered);
                    }
                    Some(value)
                }
                None => None,
            },
            None => None,
        };
        let returns = raise.result.as_ref().is_some_and(|result| !result.exit);

        if raise.condition.as_ref() == b"SYNTAX" {
            let mut raised = raise_syntax_condition(rc_text.as_deref().unwrap_or(b""), additional);
            raised.description = description;
            // **The delivery rule follows the tail even when the argument was
            // rejected**, which is measured rather than convenient: `raise
            // syntax 40.10` inside a routine, with the trap in the main body
            // and none in between, reports 98.941 fatally exactly as a
            // well-formed tail-less `RAISE SYNTAX` reports its own number
            // there. The substituted condition is still a `SYNTAX` condition
            // and travels like one.
            raised.delivery.search = if returns { Search::Here } else { Search::Top };
            return Err(raised.into());
        }

        // Every other condition. `HALT` is the one whose untrapped default
        // action reports; the rest are silent, and both halves are below.
        let halt = raise.condition.as_ref() == b"HALT";
        if raise.result.is_none() || !returns {
            // No tail, or `EXIT`: the program ends here and no trap is
            // consulted at any level. `Flow::Exit` carries `EXIT`'s own
            // value, which is `None` for the tail-less form.
            if halt {
                let mut raised = Raised::halt();
                raised.delivery.search = Search::Nobody;
                return Err(raised.into());
            }
            return Ok(Flow::Exit(result));
        }

        // `RETURN`: this routine returns `result`, and the condition is
        // offered to the caller.
        let name: Box<[u8]> = raise.condition.clone();
        // `RC` for `ERROR`/`FAILURE` is the raise's own argument, measured at
        // `rc= 5` for `raise error 5` trapped one level up. `SYNTAX`'s own
        // `RC` is the major and is filled in by `Raised::syntax` above.
        let rc = match raise.condition.as_ref() {
            b"ERROR" | b"FAILURE" => rc_text,
            _ => None,
        };
        match self.caller_trap_for(&name) {
            // A `CALL ON` trap resumes, so the condition waits for the
            // caller's current clause to finish -- `deliver_pending_traps`
            // has the two transcripts that pin the wait.
            Some(trap) if trap.call => {
                self.pending_traps.push_back(PendingTrap {
                    condition: name,
                    rc,
                    description: description.clone(),
                    // The caller's own identity -- this activation is about
                    // to be popped, and `caller_trap_for` above just read
                    // that same activation's table. See the field's own doc
                    // comment for the three transcripts behind it.
                    activation: self
                        .caller_activation()
                        .expect("a raise reaching here has a caller to queue against")
                        .id,
                    // Set by `deliver_pending_traps` if this turns out to have
                    // been queued while a handler was running, which is not
                    // knowable here: this is the raise, not the delivery.
                    queued_during_delivery: false,
                    // Which `INTERPRET` fragment's queue this joins. The
                    // raising activation is about to be popped and the depth
                    // is not its own -- a fragment does not push an activation
                    // here -- so it is read straight off `Interp`, where the
                    // `Interpret` arm maintains it.
                    fragment_depth: self.fragment_depth,
                });
                Ok(Flow::Return(result))
            }
            // A `SIGNAL ON` trap transfers, so the caller's clause is
            // abandoned rather than finished: measured, `say fun(1)` with a
            // trapped `raise user foo return 'RETVAL'` inside `fun` prints
            // nothing at all before the handler. That needs a real failure
            // unwinding this activation, not a value returned from it.
            Some(_) => {
                let mut raised = Raised::condition(condition_name(&name));
                raised.rc = rc;
                raised.description = description;
                raised.delivery.search = Search::Caller;
                Err(raised.into())
            }
            // Nothing traps it. `HALT` reports; everything else is ignored
            // outright and the routine simply returns its value -- measured,
            // `raise user foo return 'RETVAL-88'` with no trap anywhere
            // prints `RETVAL-88` and the caller carries on.
            None if halt => {
                let mut raised = Raised::halt();
                raised.delivery.search = Search::Nobody;
                Err(raised.into())
            }
            None => Ok(Flow::Return(result)),
        }
    }

    /// `RAISE PROPAGATE`: re-raise the condition whose handler is running.
    fn exec_raise_propagate(&mut self) -> Result<Flow, Failure> {
        let Some(active) = &self.active_condition else {
            return Err(Raised::syntax(98, 918, Vec::new()).into());
        };
        if !active.raised.reportable() {
            return Ok(Flow::Exit(None));
        }
        let mut raised = active.raised.clone();
        raised.delivery.search = Search::Nobody;
        raised.delivery.positionless = true;
        // The original condition's echo stack, put back exactly as it stood
        // when the trap cleared it. `record_failure_at` is first-wins, so
        // restoring a full `failure_site` is also what stops this `raise
        // propagate` clause recording itself over the clause that actually
        // raised -- measured, the oracle echoes line 8 (`say 1/0`), not line
        // 12 (`raise propagate`).
        self.failure_site = active.site.clone();
        self.failure_sites = active.sites.clone();
        Err(raised.into())
    }

    /// Resolves a `SIGNAL`/`SIGNAL VALUE` target against the running
    /// *activation's* own body -- not `code.body`, which differs inside an
    /// `INTERPRET` fragment (whose own `labels` is always empty, a label in
    /// interpreted text being 47.1). Mirrors `Interp::resolve_call`'s
    /// identical fix for `CALL`, immediately below (`run.rs:2153-2154` in
    /// the tree this task started from) -- found there by running the
    /// composition rather than reading the code, and true of `SIGNAL` for
    /// the same reason: measured, `interpret "signal there"` reaches an
    /// enclosing `there:`, and `call sub` into `sub:` containing `signal
    /// caller_label` reaches a label back in the caller's own text, because
    /// at this phase every internal `CALL` target shares its caller's exact
    /// body (no `::routine` directive gives it one of its own yet) -- not
    /// because `SIGNAL` reaches across an activation boundary on its own.
    pub(crate) fn signal_to_label(&mut self, name: &[u8]) -> Result<Flow, Failure> {
        let target = self.resolve_signal_target(name)?;
        // Set only once the target actually resolves -- an unresolved
        // `SIGNAL` (16.1) ends the program regardless, matching the oracle's
        // own `signalTo`, which a caller only ever invokes with an
        // already-resolved target.
        self.set_sigl(self.clause_state.line());
        Ok(Flow::Signal(target))
    }

    /// `SIGNAL VALUE expr`, past the expression: its `>K>` echo, then the
    /// same search and transfer a written label takes.
    pub(crate) fn signal_to_value(&mut self, value: ObjRef) -> Result<Flow, Failure> {
        let text = self.to_text(value).to_vec();
        // `>K>` names the object and the search reads the conversion, which
        // is `RexxInstructionDynamicSignal::execute`'s own order:
        // `traceKeywordResult(VALUE, result)` and then
        // `result->requestString()` (`instructions/SignalInstruction.cpp:217`
        // -`:219`). Measured, oracle rc 240: `signal value .K` with a
        // class-side `makeString` returning `'NOSUCH'` reports `Label
        // "NOSUCH" not found.`, so 16.1 names the conversion where NUMERIC's
        // own errors name the object.
        self.trace_keyword(self.clause_state.current_value_indent, "VALUE", &text);
        let converted = self.required_string_value(value)?;
        if converted == value {
            return self.signal_to_label(&text);
        }
        let text = self.to_text(converted).to_vec();
        self.signal_to_label(&text)
    }

    fn resolve_signal_target(&self, name: &[u8]) -> Result<usize, Failure> {
        let program = Rc::clone(&self.activation().program);
        let selector = self.activation().body;
        let Some(activation_body) = body_of(&program, selector) else {
            return Err(Loud::missing_body().into());
        };
        match activation_body.labels.get(name) {
            Some(target) => Ok(*target),
            None => Err(Raised::label_not_found(name).into()),
        }
    }

    /// Resolves `name` to the thing a call of it runs, with no argument
    /// evaluated and nothing entered.
    pub(crate) fn resolve_call(
        &self,
        name: &[u8],
        search_labels: bool,
    ) -> Result<Resolved, Failure> {
        // **Resolved against the running *activation's* body, not against the
        // body a caller is walking, and the two differ inside an `INTERPRET`
        // fragment.** A fragment's `labels` is always empty -- a label in
        // interpreted text is error 47.1 -- so searching the walked body would
        // make every `CALL` inside a fragment unresolvable. Measured on the
        // oracle: `interpret "call sub"` runs the enclosing program's `sub:`.
        // Found by running the composition rather than by reading the code:
        // the first version of this searched the walked body and passed every
        // test that had no `INTERPRET` in it.
        let program = Rc::clone(&self.activation().program);
        let selector = self.activation().body;
        let Some(activation_body) = body_of(&program, selector) else {
            return Err(Loud::missing_body().into());
        };
        let label = if search_labels {
            activation_body.labels.get(name).copied()
        } else {
            None
        };
        // **The whole resolution happens here, upstream of the argument loop
        // in `invoke_call`**, and the shape is load-bearing rather than tidy.
        // The builtin step needs its arguments already evaluated, so it cannot
        // sit where the raising return sits; putting the lookup between the
        // label miss and that return would have placed it upstream of the
        // evaluation it consumes. Deciding all four outcomes first is what
        // lets one argument loop serve three of them.
        let resolved = match label {
            Some(target) => Resolved::Label(target),
            // **`resolve` rather than `is_builtin`, and it answers the same
            // question.** Every row's name is in scope
            // (`every_implemented_row_names_an_in_scope_builtin`) and a name in
            // scope with no row comes back `Gap`, so `resolve(name).is_some()`
            // and `is_builtin(name)` agree on every name. It costs the same
            // one lookup and keeps which builtin it found.
            None if let Some(target) = builtin::resolve(name) => Resolved::Builtin(target),
            // A builtin Phase 4 excludes outright is still a builtin, so it
            // sits here rather than behind the routine lookup -- see
            // `builtin::is_excluded_builtin`'s own doc for why neither the
            // routine step nor 43.1 is an acceptable answer for one.
            None if builtin::is_excluded_builtin(name) => {
                return Err(Loud::unresolved_call(name).into());
            }
            None => match self.installed_routine(name) {
                Some(installed) => Resolved::Routine(installed),
                // **Ahead of the external file search, which is Phase 7's,
                // and behind everything above it.** `Setup.cpp` resolves
                // `CoreClasses.orx`'s two `CALL`s against the interpreter's
                // own directory, which is neither a label, a builtin nor a
                // `::ROUTINE`; this crate embeds those files instead. Gated
                // on the bootstrap running, so the names mean nothing to a
                // program.
                None if self.library_bootstrap
                    && let Some(program) = rexx_lib::lookup(&String::from_utf8_lossy(name)) =>
                {
                    Resolved::Library(program)
                }
                // **A routine of the oracle's own internal packages**, which
                // it consults here -- after the running package's routines and
                // before the external file search. Answering 43.1 for one of
                // these says "no such routine" for a name the oracle does
                // have, which a program cannot tell from its own typo.
                None if let Some(row) = crate::internal_routines::lookup(name) => match row.body {
                    Some(_) => Resolved::Internal(row),
                    None => return Err(Loud::internal_routine(name, row.owner).into()),
                },
                // **43.1, not this crate's loud gap**, and the difference is
                // one search: the oracle looks for an external Rexx file
                // named for the target before answering, and this crate does
                // not (Phase 7, `phase-4-exclusions.txt`). Measured in a
                // clean directory with nothing of that name beside the
                // program, the oracle's own answer is exactly this condition
                // -- `call zorkolo` gives 43.1 rc 213 `Could not find routine
                // "ZORKOLO".` -- so answering it here is right for every
                // program with no such file and wrong only for one that has
                // one, where the oracle runs the file at rc 0.
                None => return Err(Raised::routine_not_found(name).into()),
            },
        };
        Ok(resolved)
    }

    /// The `::ROUTINE` `name` reaches from the running package: one the
    /// package declared itself, then one a `::REQUIRES` imported.
    fn installed_routine(&self, name: &[u8]) -> Option<InstalledRoutine> {
        let program = self.running_activation()?.program_id;
        let upper = name.to_ascii_uppercase();
        if let Some(found) = self
            .routines
            .get(&program)
            .and_then(|table| table.get(&upper[..]))
        {
            return Some(*found);
        }
        self.merged_public_routines
            .get(&program)
            .and_then(|table| table.get(&upper[..]))
            .copied()
    }

    /// Takes the shared value buffer **with the caller's run intact**, and the
    /// depth to build above it.
    pub(crate) fn take_value_buffer(&mut self) -> (Vec<Option<ObjRef>>, usize) {
        let buffer = std::mem::take(&mut self.value_buffer);
        let mark = buffer.len();
        (buffer, mark)
    }

    /// Hands the value buffer back with this run removed, whichever way the
    /// send ended.
    pub(crate) fn give_value_buffer(&mut self, mut buffer: Vec<Option<ObjRef>>, mark: usize) {
        buffer.truncate(mark);
        self.value_buffer = buffer;
    }

    /// One builtin call: its arguments evaluated in the caller, then the row
    /// run over them.
    pub(crate) fn invoke_builtin_call(
        &mut self,
        code: &Code<'_>,
        target: crate::builtin::BuiltinTarget,
        name: &[u8],
        args: &[Option<Expr>],
    ) -> Result<ObjRef, Failure> {
        // **Pushed onto the shared stack one at a time**, rather than into a
        // buffer lent for the whole loop: each argument's evaluation calls
        // back into `&mut self`, so nothing may hold the `Vec` across it, and
        // a run that is only ever appended to needs no borrow between pushes.
        // The compiled path's `Op::PushArg` writes the same stack.
        let mark = self.value_buffer.len();
        for arg in args {
            let value = match arg {
                None => {
                    self.trace_argument(self.clause_state.current_value_indent, b"");
                    None
                }
                Some(expr) if self.leaf_argument(expr) => {
                    Some(self.eval_leaf_argument(code, expr)?)
                }
                Some(expr) => Some(self.eval_traced_argument(code, expr)?),
            };
            self.value_buffer.push(value);
        }
        self.run_over_pushed_args(mark, |interp, values| {
            builtin::run(interp, name, target, values)
        })
    }

    /// Evaluates the arguments of a call already resolved to `resolved` and
    /// runs it, in its own nested activation where it has one.
    pub(crate) fn invoke_call(
        &mut self,
        code: &Code<'_>,
        resolved: Resolved,
        name: &[u8],
        args: &[Option<Expr>],
        call_type: CallType,
        entry: CallEntry,
    ) -> Result<Ended, Failure> {
        // **Evaluated in the caller, before anything is pushed**, which is
        // where the argument expressions' own variables live. Observable
        // through failure, and the failure is real and measured: `call sub
        // 1/0` is Error 42.3
        // reported against the `CALL` clause, at rc 214, and a version that
        // skipped evaluation would run the callee instead.
        if let Resolved::Builtin(target) = resolved {
            return Ok(Ended::Returned(Some(
                self.invoke_builtin_call(code, target, name, args)?,
            )));
        }
        // An internal-package routine runs no activation either, so it takes
        // the same shortcut -- but its arguments are evaluated by the loop
        // below rather than by the builtin path's own, which is why it is not
        // folded into the arm above.
        if let Resolved::Internal(row) = resolved {
            let mut values: Vec<Option<ObjRef>> = Vec::with_capacity(args.len());
            for arg in args {
                match arg {
                    None => {
                        self.trace_argument(self.clause_state.current_value_indent, b"");
                        values.push(None);
                    }
                    Some(expr) if self.leaf_argument(expr) => {
                        values.push(Some(self.eval_leaf_argument(code, expr)?));
                    }
                    Some(expr) => values.push(Some(self.eval_traced_argument(code, expr)?)),
                }
            }
            return Ok(Ended::Returned(Some(self.run_internal(row, &values)?)));
        }

        // A fresh `Vec` and not a lent one: this path always hands the
        // arguments to the callee, which keeps them, so there is nothing to
        // give back and a pool would allocate on every call anyway.
        let mut arguments: Vec<Option<ObjRef>> = Vec::with_capacity(args.len());
        for arg in args {
            match arg {
                None => {
                    // An omitted position traces an **empty** value line, not
                    // no line: `traceArgument(GlobalNames::NULLSTRING)`,
                    // `RexxInstruction.cpp:161`, and measured above.
                    self.trace_argument(self.clause_state.current_value_indent, b"");
                    arguments.push(None);
                }
                Some(expr) if self.leaf_argument(expr) => {
                    arguments.push(Some(self.eval_leaf_argument(code, expr)?));
                }
                Some(expr) => arguments.push(Some(self.eval_traced_argument(code, expr)?)),
            }
        }
        self.invoke_call_over(resolved, name, arguments, call_type, entry)
    }

    /// One compiled call over the arguments its own ops already evaluated.
    pub(crate) fn call_over_pushed_args(
        &mut self,
        resolved: Resolved,
        name: &[u8],
        mark: usize,
    ) -> Result<ObjRef, Failure> {
        self.run_over_pushed_args(mark, |interp, values| {
            interp.call_over_values(resolved, name, values)
        })
    }

    /// Runs `body` over the argument run standing above `mark`, with the
    /// stack lent out for the duration and this run removed on the way back.
    fn run_over_pushed_args(
        &mut self,
        mark: usize,
        body: impl FnOnce(&mut Interp, &[Option<ObjRef>]) -> Result<ObjRef, Failure>,
    ) -> Result<ObjRef, Failure> {
        let mut values = std::mem::take(&mut self.value_buffer);
        let outcome = body(self, &values[mark..]);
        values.truncate(mark);
        self.value_buffer = values;
        outcome
    }

    /// One internal-package routine over its evaluated arguments. No
    /// activation, no `SIGL`, no depth guard -- the builtin discipline, since
    /// the oracle runs these as native code too.
    fn run_internal(
        &mut self,
        row: &'static crate::internal_routines::InternalRoutine,
        values: &[Option<ObjRef>],
    ) -> Result<ObjRef, Failure> {
        let body = row
            .body
            .expect("resolve_call only answers Internal with a body");
        body(self, row.name.as_bytes(), values)
    }

    /// [`Interp::call_over_pushed_args`] with the run in hand.
    fn call_over_values(
        &mut self,
        resolved: Resolved,
        name: &[u8],
        values: &[Option<ObjRef>],
    ) -> Result<ObjRef, Failure> {
        // The builtin path, which runs no activation -- the same shortcut
        // `eval_call_resolved` takes and for the same measured reason.
        if let Resolved::Builtin(target) = resolved {
            return builtin::run(self, name, target, values);
        }
        if let Resolved::Internal(row) = resolved {
            return self.run_internal(row, values);
        }
        match self.invoke_call_over(
            resolved,
            name,
            values.to_vec(),
            CallType::Function,
            CallEntry::Written,
        )? {
            Ended::Exited(value) => Err(Failure::Exited(value)),
            Ended::Returned(Some(value)) => Ok(value),
            Ended::Returned(None) => Err(Raised::no_data_returned(name).into()),
        }
    }

    /// One `::ROUTINE` entered from a native method body: `Routine~call`,
    /// `~callWith` and `~'[]'`.
    pub(crate) fn call_over_installed_routine(
        &mut self,
        installed: InstalledRoutine,
        arguments: Vec<Option<ObjRef>>,
    ) -> Result<Option<ObjRef>, Failure> {
        match self.invoke_call_over(
            Resolved::Routine(installed),
            b"CALL",
            arguments,
            CallType::Subroutine,
            CallEntry::Written,
        )? {
            Ended::Exited(value) => Err(Failure::Exited(value)),
            Ended::Returned(returned) => Ok(returned),
        }
    }

    /// [`Interp::invoke_call`] past its argument evaluation: everything a
    /// callee needs once its arguments are values.
    pub(crate) fn invoke_call_over(
        &mut self,
        resolved: Resolved,
        name: &[u8],
        arguments: Vec<Option<ObjRef>>,
        call_type: CallType,
        entry: CallEntry,
    ) -> Result<Ended, Failure> {
        // **The builtin outcome ends here**, before `SIGL`, before the depth
        // guard and before any activation is pushed -- each of those three is
        // the label path's, and the oracle answers that the builtin path has
        // none of them (`builtin`'s own module doc carries the probe for
        // each). Every argument is still rooted by the loop above, so the
        // allocation a builtin's result costs happens with the inputs
        // reachable, and the value handed back is rooted by whichever caller
        // receives it exactly as a callee's `RETURN` value already is.
        // **The library outcome ends here too**, and for a reason unlike the
        // builtin's: an embedded `.orx` source is a whole program, with its
        // own directives to install and its own `ProgramId`, so it is
        // entered the way the command line's program is rather than the way
        // a routine is. `Entered` has no arm for it because none of the
        // decisions that enum exists to carry -- `SIGL`, the callee's pool,
        // the indent, the calling convention's receiver -- is a question
        // about it.
        if let Resolved::Library(program) = resolved {
            return self
                .enter_library_program(program, Some(arguments))
                .map(Ended::Returned);
        }

        let entered = match resolved {
            // Answered above, before the loop that just ran.
            Resolved::Builtin(_) => unreachable!("the builtin path returns before this"),
            Resolved::Internal(_) => unreachable!("the internal path returns before this"),
            Resolved::Label(target) => Entered::Label(target),
            Resolved::Routine(installed) => Entered::Routine(installed),
            Resolved::Library(_) => unreachable!("the library path returns just above"),
        };

        // The caller's own program and body selector, which a label callee
        // inherits: the same pair `resolve_call` searched, read again here
        // rather than threaded out of it, because what they are wanted for is
        // building the callee rather than finding it.
        let program = Rc::clone(&self.activation().program);
        let program_id = self.activation().program_id;
        let selector = self.activation().body;

        // `SIGL`, set here rather than before the argument loop above: the
        // oracle's own `internalCall` (`RexxActivation.cpp`, read directly)
        // receives its arguments already evaluated by its caller, so they
        // are evaluated under whatever `SIGL` was already in force, and only
        // then does the transfer overwrite it. Measured: `signal there` /
        // `there: call sub sigl` into `sub: use arg a` reports the argument
        // as `1` (the `SIGNAL`'s own line, still in force during evaluation)
        // and `sub`'s own `SIGL` as the `CALL`'s line -- a version setting
        // `SIGL` before evaluating arguments would report the argument as
        // the `CALL`'s own line instead.
        if matches!(entered, Entered::Label(_)) {
            self.set_sigl(self.clause_state.line());
        }

        // D19/I6: one Rust frame per activation, plus this counter, so an
        // unbounded recursion becomes a reportable condition instead of a
        // native abort. `Raised::insufficient_stack` already existed
        // (`error.rs`); measured, the oracle answers the same 11.1 at rc 245
        // for the same program, at its own depth of 27,314.
        if self.activation_depth() >= MAX_ACTIVATION_DEPTH {
            return Err(Raised::insufficient_stack().into());
        }

        let callee_id = self.next_activation_id();
        match entered {
            Entered::Label(target) => {
                // **D9r's default: a shared pool.** The callee reuses the
                // caller's `SlotFrame`, so it reads and writes the caller's
                // variables and its writes survive the return -- measured,
                // and `pop_slots` is deliberately not called on the way out
                // because the frame is not this activation's to free. Task
                // 5's `PROCEDURE` is what will ever push a frame of its own.
                let caller = self.activation();
                let plan = Rc::clone(&caller.plan);
                let frame = caller.frame;
                // The `call_type` parameter is deliberately not read here: an
                // internal label answers `PARSE SOURCE`'s second word from the
                // activation it was called out of, whichever form called it.
                // Measured, a `::routine` invoked as a function whose body
                // `CALL`s a label reads `FUNCTION` inside that label, not
                // `SUBROUTINE`.
                let caller_call_type = caller.call_type;
                let settings = caller.settings.clone();
                let trace_mode = caller.trace_mode;
                // Inherited with `settings` and for its reason: a `SIGNAL ON`
                // in the caller turned the package's escalation off for the
                // rest of that activation, and an internal call is still
                // inside it.
                let condition_syntax = caller.condition_syntax;
                let extra = caller.extra.clone();
                // Cloned with `extra`, and for the same reason: a label
                // reached without `PROCEDURE` shares the caller's pool, and an
                // exposed name is part of that pool. Measured -- a class
                // method exposing `v`, calling a label that assigns `v`, and a
                // second class method reading `v` back -- the assignment
                // reaches the object variable. Without this the label writes
                // the frame slot the exposure left empty and the write is
                // lost.
                let exposed = caller.exposed.clone();
                // Cloned in and never written back, exactly like `settings`
                // and `trace_mode` beside it -- `Activation::traps`' own doc
                // comment has the three probes that measure the inheritance
                // and its one-way direction.
                let traps = caller.traps.clone();
                // Both halves of the pair, not just the current one:
                // measured, a callee's own bare `ADDRESS` swaps to the
                // *caller's* alternate. `Activation::address`' own doc
                // comment has the transcript.
                let address = caller.address.clone();
                // Same one-way rule again: an internal call sees the caller's
                // `CONDITION()` answers and a reset inside the callee dies
                // with it. `TrappedCondition`'s own doc comment has the
                // four-line transcript.
                let condition = caller.condition.clone();
                let mut callee = Activation::nested(
                    callee_id,
                    program,
                    program_id,
                    selector,
                    plan,
                    frame,
                    target,
                    Inherited {
                        call_type: caller_call_type,
                        settings,
                        condition_syntax,
                        trace_mode,
                        address,
                        traps,
                        condition,
                    },
                );
                callee.extra = extra;
                callee.exposed = exposed;
                self.push_activation(callee);
            }
            // **A pool of its own, and not one of the five inheritances**
            // -- `Activation::routine` is where that is stated and
            // `Activation::nested`'s own doc carries the six probes. The
            // plan is the routine body's own, cached under its own
            // `BodyKey`, and it is what sizes the frame: a routine's names
            // are not the caller's, so a frame sized from the caller's plan
            // would be the wrong length.
            Entered::Routine(installed) => {
                // Reached through `programs` rather than through the running
                // activation's own `Rc`, so the plan's cache key and the
                // activation's program are the same program by construction
                // (`InstalledRoutine`'s own doc).
                let routine_program = Rc::clone(&self.programs[installed.program.0]);
                let Some(body) = body_of(&routine_program, Some(installed.directive)) else {
                    return Err(Loud::missing_body().into());
                };
                let plan = self.plan_for(
                    BodyKey {
                        program: installed.program,
                        directive: Some(installed.directive),
                    },
                    body,
                    &routine_program.symbols,
                    &routine_program.source,
                );
                let frame = self.roots.push_slots(plan.len());
                let mut callee = Activation::routine(
                    callee_id,
                    routine_program,
                    installed.program,
                    installed.directive,
                    plan,
                    frame,
                    call_type,
                );
                // The routine's own package, which is not always the caller's
                // -- `installed.program` is where the `::ROUTINE` was
                // declared. The caller's settings go with it for
                // `::OPTIONS NUMERIC INHERIT`, the one option that reads
                // them.
                self.start_from_package(&mut callee, Some(&self.activation().settings));
                self.push_activation(callee);
                self.trace_package_invocation_entry();
            }
        }

        // Level state for the callee, five pieces, saved here and restored
        // on both paths below. `Interpret`'s own arm is the model for four
        // of them, and one differs from it deliberately -- the fifth,
        // `clause_state`, is not level state for the callee at all, and is
        // saved and restored for a different reason stated where it is:
        let saved_clause_state = self.save_clause_state();
        let callee_indent = match entered {
            Entered::Label(_) => saved_clause_state.value_indent() + 2,
            Entered::Routine(_) => 0,
        };
        let saved_base = std::mem::replace(&mut self.activation_indent, callee_indent);
        let saved_offset = std::mem::take(&mut self.indent_offset);
        let saved_line = std::mem::take(&mut self.clause_line_override);
        let inherited = entered_receiver(entered, entry, self.call_context.receiver);
        let arguments = self.shared_arguments(&arguments);
        let saved_context = std::mem::replace(
            &mut self.call_context,
            CallContext {
                name: name.to_vec(),
                arguments,
                // Read out of the caller's own convention before this
                // replaces it, which is the only place it can be read from:
                // `entered_receiver` carries the rule and the measurement
                // for each of its arms.
                receiver: inherited,
            },
        );

        let ended = self.run_activation();

        // Before the pop, because both halves of `<I<`'s gate are the
        // callee's own -- its `trace_entry` state and its `TRACE` setting.
        // `RexxActivation::termination` is where the C++ puts it, which is
        // likewise inside the activation.
        self.trace_invocation_exit();

        // Popped on both paths, and unconditionally: `run_activation`'s own
        // loop asserts the activation stack is where it found it after every
        // step, so a `CALL` that returned with the callee still on it would
        // trip that assertion in the caller rather than quietly running the
        // wrong frame's `pc`.
        let mut callee = self.pop_activation().expect("the activation just pushed");
        // **The two halves of "was the pool shared" are one bool, and both
        // are needed.** A `PROCEDURE` callee pushed a frame of its own, so
        // that frame is popped here -- on the error path as well, which is
        // why this is not inside the `Ok` arm below. It also keeps its own
        // run-time name bindings, so they are *not* moved back: doing that
        // would overwrite the caller's `extra` with the callee's isolated
        // one. A shared-pool callee is the opposite on both counts, and its
        // `extra` write-back is what makes a name bound inside it survive
        // the return (measured, `interpret "zork = 42"` in a callee).
        if callee.owns_frame {
            self.roots.pop_slots(callee.frame);
        } else {
            // Taken rather than moved out, so the box stays whole and can be
            // parked: moving a field out of a `Box` moves the whole of it out
            // and frees the box, which is the allocation the pool exists to
            // keep. What is left behind is the empty map a fresh activation
            // starts with.
            self.activation_mut().extra = std::mem::take(&mut callee.extra);
        }
        self.recycle_activation(callee);
        self.activation_indent = saved_base;
        self.indent_offset = saved_offset;
        self.clause_line_override = saved_line;
        self.restore_clause_state(saved_clause_state);
        self.call_context = saved_context;

        // **`EXIT` inside a `::ROUTINE` ends the routine, not the program**,
        // where `EXIT` inside a `CALL`ed label ends the program. The C++'s
        // `implicitExit` sets `RETURNED` outright for an `isProgramLevelCall`
        // activation and only otherwise walks up through `exitFrom`
        // (`RexxActivation.cpp:1455`-`1469`), and a routine invocation is one
        // of those. Measured on the oracle, rc 0 every time, and the shapes
        // matter because the exit arrives here by three different routes:
        // ```text
        // call rtn / say result       ::routine rtn ; exit 5      ->  5, and main runs on
        // n1 = rtn() / say n1         ::routine rtn ; exit 7      ->  7
        // call rtn / say result       ::routine rtn ; exit        ->  RESULT, unset
        // call rtn / say 'after'      a label INSIDE the routine exits 9 -> "after" runs
        // call rtn / say 'after'      interpret "exit 4" in the routine  -> "after" runs
        // ```
        let ended = match ended {
            Ok(Ended::Exited(value)) | Err(Failure::Exited(value))
                if matches!(entered, Entered::Routine(_)) =>
            {
                return Ok(Ended::Returned(value));
            }
            other => other,
        };

        match ended {
            Ok(ended) => Ok(ended),
            Err(failure) => {
                // Seal before the failure leaves the callee, never after --
                // `seal_site_level`'s own rule, and the same one
                // `run_fragment` follows. Without it the callee's clause
                // would win `record_failure_at`'s first-wins race outright
                // and the call would never be echoed. Measured, the oracle
                // prints one echo per level, innermost first: a `say 1/0` in
                // a routine called from a routine called from a `DO` gives
                // three lines, at indents 6, 4 and 2.
                self.seal_site_level();
                Err(failure)
            }
        }
    }

    /// [`Interp::resolve_call`] followed by [`Interp::invoke_call`], with
    /// nothing remembered in between.
    pub(crate) fn resolve_and_run_call(
        &mut self,
        code: &Code<'_>,
        name: &[u8],
        search_labels: bool,
        args: &[Option<Expr>],
        call_type: CallType,
        entry: CallEntry,
    ) -> Result<Ended, Failure> {
        let resolved = self.resolve_call(name, search_labels)?;
        self.invoke_call(code, resolved, name, args, call_type, entry)
    }

    /// Builds the object a `>name` or `<name` term answers: the variable
    /// `inner` names, rather than the value it holds.
    pub(crate) fn variable_reference(
        &mut self,
        code: &Code<'_>,
        inner: &Expr,
    ) -> Result<ObjRef, Failure> {
        let id = match &inner.kind {
            ExprKind::Variable(id) | ExprKind::Stem(id) => *id,
            other => return Err(Loud::expression(other).into()),
        };
        let slot = match code.slot_for(id) {
            Some(slot) => slot,
            None => self.slot_of(code.symbols.name(id).as_bytes()),
        };
        let frame = self.activation().frame;
        // Resolved here, where the name's own home is: an alias this
        // activation holds is still addressable, which is what makes `>p`
        // work when its `p` came from *its* caller, and an `EXPOSE` binding
        // is on this activation, which is what makes it work on an object
        // variable. Chasing the slot for an exposed name would name the
        // empty slot the exposure left behind.
        let home = match self.exposure(frame, slot) {
            Some(var) => VarRefHome::Instance {
                owner: var.owner,
                scope: var.scope,
            },
            None => VarRefHome::Cell(self.roots.promote(frame, slot)),
        };
        let name = code.symbols.name(id).as_bytes().into();
        Ok(self.alloc_with(
            BehaviourId::OBJECT,
            Body::VarRef(Box::new(rexx_core::VarRef { name, home })),
        ))
    }

    /// One call argument, evaluated, rooted and traced -- the step both call
    /// paths share.
    #[inline(always)]
    fn leaf_argument(&self, expr: &Expr) -> bool {
        !self.tracing_intermediates()
            && matches!(
                expr.kind,
                ExprKind::Literal(_) | ExprKind::Constant(_) | ExprKind::Variable(_)
            )
    }

    /// One argument's value, for a shape whose evaluation produces that
    /// value and nothing else.
    fn eval_leaf_argument(&mut self, code: &Code<'_>, expr: &Expr) -> Result<ObjRef, Failure> {
        let anchor = 0u8;
        self.enter_eval_node(&raw const anchor)?;
        let value = self.eval_node(code, expr);
        self.depth -= 1;
        let value = value?;
        self.roots.push_temp(value);
        Ok(value)
    }

    pub(crate) fn eval_traced_argument(
        &mut self,
        code: &Code<'_>,
        expr: &Expr,
    ) -> Result<ObjRef, Failure> {
        let argument = self.eval(code, expr)?;
        self.roots.push_temp(argument);
        if let Some(rendered) = self.intermediate_text(argument) {
            self.trace_argument(self.clause_state.current_value_indent, &rendered);
        }
        Ok(argument)
    }

    /// A call's resolution, held back until its arguments have run.
    #[inline(always)]
    pub(crate) fn resolved_after_arguments(
        &mut self,
        code: &Code<'_>,
        resolution: Result<Resolved, Failure>,
        args: &[Option<Expr>],
    ) -> Result<Resolved, Failure> {
        match resolution {
            Ok(resolved) => Ok(resolved),
            Err(failure) => Err(self.arguments_before_failure(code, args, failure)),
        }
    }

    /// [`Interp::resolved_after_arguments`]' failing half: run the arguments
    /// for their trace lines and their own conditions, then report the
    /// resolution failure if none of them raised first.
    #[cold]
    #[inline(never)]
    fn arguments_before_failure(
        &mut self,
        code: &Code<'_>,
        args: &[Option<Expr>],
        failure: Failure,
    ) -> Failure {
        // The same three shapes `invoke_call`'s own loop has, for their trace
        // lines and their failures; the values themselves are dropped, since
        // there is no callee to hand them to.
        for arg in args {
            let raised = match arg {
                None => {
                    self.trace_argument(self.clause_state.current_value_indent, b"");
                    continue;
                }
                Some(expr) if self.leaf_argument(expr) => self.eval_leaf_argument(code, expr).err(),
                Some(expr) => self.eval_traced_argument(code, expr).err(),
            };
            if let Some(raised) = raised {
                return raised;
            }
        }
        failure
    }

    /// Runs one named `CALL`: `resolve_call`, then [`Interp::invoke_named_call`].
    fn exec_call(
        &mut self,
        code: &Code<'_>,
        name: &[u8],
        search_labels: bool,
        args: &[Option<Expr>],
    ) -> Result<Flow, Failure> {
        let resolution = self.resolve_call(name, search_labels);
        let resolved = self.resolved_after_arguments(code, resolution, args)?;
        self.invoke_named_call(code, resolved, name, args)
    }

    /// The `CALL` instruction past its resolution: [`Interp::invoke_call`],
    /// then settle `RESULT` and translate the outcome into this instruction's
    /// own `Flow`.
    pub(crate) fn invoke_named_call(
        &mut self,
        code: &Code<'_>,
        resolved: Resolved,
        name: &[u8],
        args: &[Option<Expr>],
    ) -> Result<Flow, Failure> {
        // Captured before `invoke_call` runs the callee, which overwrites
        // `current_value_indent` with its own clauses' -- this is the `CALL`
        // clause's own printed indent, needed below for the caller-side
        // `RESULT` trace.
        let base_indent = self.clause_state.current_value_indent;
        let ended = self.invoke_call(
            code,
            resolved,
            name,
            args,
            CallType::Subroutine,
            CallEntry::Written,
        )?;
        self.settle_call_result(ended, base_indent)
    }

    /// What a `CALL` does with the outcome its callee handed back: the
    /// caller's own `>>>` and `RESULT`.
    fn settle_call_result(&mut self, ended: Ended, base_indent: usize) -> Result<Flow, Failure> {
        let value = match ended {
            // `EXIT` inside the callee ends the program rather than the
            // call, and so does running off the end of the body -- measured
            // both ways. Forwarded unchanged; `RESULT` is never touched on
            // this path.
            Ended::Exited(value) => return Ok(Flow::Exit(value)),
            Ended::Returned(value) => value,
        };

        // **`RESULT` is settled on return and not at the call.** Measured:
        // a caller setting `result = 'before'` and calling a no-`PROCEDURE`
        // routine has the callee print `inside result= before`, so nothing
        // is cleared on the way in. After `return 42` the caller reads `42`;
        // after a bare `return` it reads the derived name `RESULT`, which is
        // what an unset variable renders as.
        let slot = self.reserved_result_slot();
        let frame = self.activation().frame;
        match value {
            Some(value) => {
                // Re-rooted in the caller: `Op::Clause`'s region popped the
                // callee's temps frame around every clause it ran, this one
                // included, so the `push_temp` the `RETURN` arm did is gone
                // by now. Same window `Flow::Exit`'s own arm documents,
                // closed here rather than left open, because unlike an exit
                // value this one goes on to be stored and read.
                self.roots.push_temp(value);
                // The caller's own `>>>`, at the `CALL` clause's indent --
                // `base_indent`, saved before the callee overwrote
                // `current_value_indent` with its own clauses'.
                if let Some(rendered) = self.result_text(value) {
                    self.trace_result(base_indent, &rendered);
                }
                self.set_variable(frame, slot, value);
            }
            None => self.clear_variable(frame, slot),
        }
        Ok(Flow::Next)
    }

    /// [`Interp::invoke_named_call`] over arguments the compiled stream has
    /// already evaluated onto the argument stack above `mark`.
    pub(crate) fn invoke_named_call_over_pushed_args(
        &mut self,
        resolved: Resolved,
        name: &[u8],
        mark: usize,
    ) -> Result<Flow, Failure> {
        let base_indent = self.clause_state.current_value_indent;
        let ended = self.subroutine_over_pushed_args(resolved, name, mark)?;
        self.settle_call_result(ended, base_indent)
    }

    /// The callee half of [`Interp::invoke_named_call_over_pushed_args`]: the
    /// argument run is lent out, turned into `Argument`s, and removed again on
    /// the way back whichever way the call ended.
    fn subroutine_over_pushed_args(
        &mut self,
        resolved: Resolved,
        name: &[u8],
        mark: usize,
    ) -> Result<Ended, Failure> {
        let mut values = std::mem::take(&mut self.value_buffer);
        let outcome = match resolved {
            // No activation, no `Argument`s built -- the shortcut
            // `call_over_values` takes, and the reason a `CALL` to a builtin
            // reaches `Ended::Returned(Some(_))` with a value to settle.
            Resolved::Builtin(target) => builtin::run(self, name, target, &values[mark..])
                .map(|value| Ended::Returned(Some(value))),
            // **Named rather than left to the arm below**, which has a
            // catch-all: an internal routine sent into `invoke_call_over`
            // would reach a `match` that has no arm for it.
            Resolved::Internal(row) => self
                .run_internal(row, &values[mark..])
                .map(|value| Ended::Returned(Some(value))),
            _ => self.invoke_call_over(
                resolved,
                name,
                values[mark..].to_vec(),
                CallType::Subroutine,
                CallEntry::Written,
            ),
        };
        values.truncate(mark);
        self.value_buffer = values;
        outcome
    }

    /// Opens a stepped clause of `code`: everything
    /// [`crate::ir::Op::Clause`] owes before the clause's own work
    /// runs.
    #[inline(always)]
    #[allow(
        clippy::too_many_arguments,
        reason = "the added parameter is a zero-sized proof rather than data, and the alternative -- letting this half take the deadline itself -- is the shape whose cost the Deadline doc's table rejects"
    )]
    pub(crate) fn enter_stepped_clause(
        &mut self,
        echo: Echo,
        code: &Code<'_>,
        index: usize,
        instruction: &Instruction,
        source: Option<&ProgramSource>,
        position: Option<crate::ir::ClausePosition>,
        counted: crate::clause::DeadlineCounted,
    ) -> SteppedClause {
        // `DATE`/`TIME`'s per-clause clock cache (`activation.rs`'s own doc
        // on `Activation::clock_stale`) is invalidated **unconditionally,
        // once per call, on whichever activation is executing right now**,
        // mirroring `RexxActivation::run`'s `settings.timeStamp.valid =
        // false` right after `nextInst->execute()` returns
        // (`RexxActivation.cpp:647`). This is that same one place every
        // flat instruction position -- including one nested inside `IF`/
        // `SELECT`/`DO`/`INTERPRET`, per this function's own doc, and
        // including a callee's own body reached through `resolve_and_run_
        // call` -- passes through, so a clause's first clock read gets a
        // fresh one, any further read of the same clause sees the value
        // that read cached, and a callee's own instructions invalidate only
        // the callee's own cache rather than reaching back into the
        // caller's. The cached *value* itself (`Activation::cached_clock`)
        // is left untouched here -- `TIME('R')`'s own lazy reset needs it
        // still readable one call later, and clearing it here is what an
        // earlier version of this did instead.
        let activation = self.activation_mut();
        activation.clock_stale = true;
        // `>I>`'s own "am I still on the first instruction" decay
        // (`TraceEntry`, `activation.rs`), spent **here** and not in
        // `run_activation`'s loop. `RexxActivation.cpp:657`-`659` clears the
        // C++'s flag at the bottom of its instruction loop, and an `IF`'s
        // then-clause, a `DO` body clause and a fragment's clauses are each a
        // separate instruction in that loop -- where here they are nested
        // inside their enclosing clause's step. This function is the one
        // place a clause is stepped at any depth, so it is the only site that
        // counts them all. Measured before the move: `if 1=1 then trace l` as
        // a routine's first clause announced the pair here and nothing on the
        // oracle.
        activation.trace_entry = activation.trace_entry.stepped();
        // `TRACE`'s own `*-*` clause echo (D17), and the single insertion
        // point for it -- exactly the analogue of `eval`'s own split from
        // `eval_node`, since this is the one place `run_bounded`'s loop
        // (this function's only non-test caller) visits every flat
        // instruction position it steps, markers (`Then`/`Else`/
        // `Otherwise`/`When`/`WhenCase`/`Label`) included, matching the
        // oracle's own `RexxInstruction::traceInstruction`, which every one
        // of those calls too from its own `execute`.
        let shortcut = match position {
            Some(position) if source.is_some() && self.clause_line_override.is_none() => {
                Some(position)
            }
            _ => None,
        };
        let indent = match shortcut {
            Some(position) => {
                position.indent as usize + self.activation_indent + self.indent_offset
            }
            None => self.printed_indent(code, index),
        };
        debug_assert_eq!(
            indent,
            self.printed_indent(code, index),
            "the chunk's indent table disagrees with the plan's for instruction {index}"
        );
        self.clause_state.current_value_indent = indent;
        // Set here with the indent, and for the same reason that one is: this
        // is the one place every stepped instruction passes before its `step`
        // call runs, which is where the oracle reads it too. See the field.
        self.clause_state.instructions_traced_at_entry = self.trace_mode().all;
        // Set unconditionally, exactly like `current_value_indent` just
        // above and for the identical reason (that field's own doc comment):
        // `SIGL` (`lib.rs`'s doc on `current_clause_line`) has to stay
        // correct whether or not `TRACE` is on, and this is the one place
        // every stepped instruction, `SIGNAL`/`CALL` included, passes
        // through before its own `step` call runs.
        // `enter_clause` rather than a bare assignment: the clause line and
        // the clause boundary are one operation (`clause.rs`), and the
        // `ClauseEntry` this hands back is what `leave_stepped_clause` spends
        // on the matching half.
        let line = match shortcut {
            Some(position) => position.line as usize,
            None => self
                .clause_line_at(code, index, instruction, source)
                .unwrap_or_else(|| self.clause_state.line()),
        };
        debug_assert_eq!(
            line,
            self.clause_line_at(code, index, instruction, source)
                .unwrap_or_else(|| self.clause_state.line()),
            "the chunk's line table disagrees with the plan's for instruction {index}"
        );
        // The frame readers' own view of this clause
        // ([`crate::activation::ClauseSnapshot`], which carries where this
        // has to be and why the line cannot come from `Activation::pc`).
        // **Skipped while a fragment is running**, so the enclosing
        // `INTERPRET` clause stays in force for this activation -- the
        // answer the oracle gives the frame beneath its own
        // `FRAME_INTERPRET` one.
        if self.clause_line_override.is_none() {
            self.clause_state.current_clause_index = index;
        }
        let entry = self.enter_clause(line, counted);
        // **`Echo::Gated` asks whether the setting in force echoes this
        // clause; `Echo::Compiled` is a clause whose chunk already answered
        // that**, and emits the echo as an op of its own
        // (`crate::ir::Op::TraceClause`) rather than here. The whole point of
        // the second arm is that a chunk compiled under a setting that does
        // not echo pays nothing at all for the decision -- no gate, no
        // `clause_site`, no op.
        if matches!(echo, Echo::Gated) {
            self.echo_stepped_clause(source, instruction, indent);
        }
        // The debug tripwire I22 asks for, alongside `RootSet::temps_len`, its
        // one prerequisite. `SteppedClause` carries the watermark and the
        // frame across to the matching half, which is where the check reads
        // them; that type's own doc comment has what it checks and why it is
        // there rather than in `pop_frame`.
        let temps_at_entry = self.roots.temps_len();
        let frame = self.roots.push_frame();
        SteppedClause {
            entry,
            frame,
            temps_at_entry,
        }
    }

    /// Closes the stepped clause `entry` opened, around `ran` -- everything
    /// [`crate::ir::Op::Clause`] owes once the clause's own work has
    /// run.
    #[inline(always)]
    pub(crate) fn leave_stepped_clause<T: ClauseValue>(
        &mut self,
        entry: SteppedClause,
        code: &Code<'_>,
        index: usize,
        instruction: &Instruction,
        source: Option<&ProgramSource>,
        ran: Result<T, Failure>,
    ) -> Result<ClauseOutcome<T>, Failure> {
        debug_assert!(
            ran.is_err() || self.roots.temps_len() >= entry.temps_at_entry,
            "step popped below its own temps watermark ({} -> {}), so it \
             discarded roots it did not push",
            entry.temps_at_entry,
            self.roots.temps_len()
        );
        self.roots.pop_frame(entry.frame);
        if ran.is_err() {
            self.record_failure_site(code, index, source, instruction);
        }
        // **A `DO`/`LOOP`'s step is not a clause the oracle has, so it owes no
        // boundary** -- `Interp::leave_clause_without_boundary` has the
        // mechanism and the transcript. The boundaries the construct does owe
        // are opened elsewhere: a plain `DO`'s header and `END` clauses in
        // `run_loop_with_header`'s own `LoopKind::Simple` arm, and a
        // repeating loop's clauses in `run_repeating`.
        let outcome = match &instruction.kind {
            InstructionKind::Do(_) | InstructionKind::Loop(_) => {
                self.leave_clause_without_boundary(entry.entry, ran)
            }
            _ => self.leave_clause(entry.entry, code, ran),
        };
        // The clause's *own* failure came back as `Ran(Err(_))` and was
        // recorded just above; this `Err` is the boundary's, and it is the
        // same clause that owes the site. Recording it twice is harmless --
        // `record_failure_at`'s first-wins guard makes the second call a
        // no-op -- and recording it in neither place is what the measurement
        // in `Op::Clause`'s region's doc comment describes.
        if outcome.is_err() {
            self.record_failure_site(code, index, source, instruction);
        }
        outcome
    }

    /// The stepped-clause boundary for a clause that produced **neither a
    /// `Flow` nor a failure**, with nothing queued to deliver.
    #[inline(always)]
    pub(crate) fn finish_plain_clause(&mut self, entry: SteppedClause) {
        debug_assert!(
            self.roots.temps_len() >= entry.temps_at_entry,
            "step popped below its own temps watermark ({} -> {}), so it \
             discarded roots it did not push",
            entry.temps_at_entry,
            self.roots.temps_len()
        );
        self.roots.pop_frame(entry.frame);
        self.spend_clause_entry(entry.entry);
    }

    /// One stepped clause's `*-*` echo, **if the setting in force echoes a
    /// clause of that kind**, at `indent`.
    #[inline(always)]
    pub(crate) fn echo_stepped_clause(
        &mut self,
        source: Option<&ProgramSource>,
        instruction: &Instruction,
        indent: usize,
    ) {
        let is_label = matches!(instruction.kind, InstructionKind::Label { .. });
        if self.tracing_clause(is_label)
            && let Some((line, text)) = self.clause_site(source, instruction)
        {
            self.trace_stepped_clause(is_label, line, indent, &text);
        }
    }

    /// The same echo with **no gate at all**, for a caller that has already
    /// decided ([`crate::ir::Op::TraceClause`], whose presence in a chunk is
    /// that decision).
    pub(crate) fn echo_compiled_clause(
        &mut self,
        source: Option<&ProgramSource>,
        instruction: &Instruction,
        indent: usize,
    ) {
        if let Some((line, text)) = self.clause_site(source, instruction) {
            crate::trace::push_clause(&mut self.trace, line, indent, &text);
        }
    }

    /// Resolves `instruction`'s own clause (and its statically-derived
    /// indent, `static_indent`) into `self.failure_site`, first call wins,
    /// when `source` is `Some`.
    pub(crate) fn record_failure_site(
        &mut self,
        code: &Code<'_>,
        index: usize,
        source: Option<&ProgramSource>,
        instruction: &Instruction,
    ) {
        let indent = self.printed_indent(code, index);
        self.record_failure_at(source, instruction, indent);
    }

    /// Assigns `blame`'s own clause to `self.failure_site` at exactly
    /// `indent` spaces, first call wins, when `source` is `Some`.
    fn record_failure_at(
        &mut self,
        source: Option<&ProgramSource>,
        blame: &Instruction,
        indent: usize,
    ) {
        if self.failure_site.is_some() {
            return;
        }
        if let Some((line, text)) = self.clause_site(source, blame) {
            self.failure_site = Some(match self.sourceless_site(line, indent) {
                Some(site) => site,
                None => match self.compiled_method_site(line, &text, indent) {
                    Some(site) => site,
                    None => match self.required_package_site(line, &text, indent) {
                        Some(site) => site,
                        None => FailureSite::Clause { line, text, indent },
                    },
                },
            });
        }
    }

    /// The frame a level in a package a `::REQUIRES` loaded contributes, or
    /// `None` for a level in the program the command line started.
    fn required_package_site(
        &self,
        line: usize,
        text: &[u8],
        indent: usize,
    ) -> Option<FailureSite> {
        let program_id = self.running_activation()?.program_id;
        let name = self.required_paths.get(&program_id)?.as_bytes().to_vec();
        Some(FailureSite::Named {
            line,
            indent,
            text: text.to_vec(),
            name,
        })
    }

    /// The frame a level inside a method compiled from source text
    /// contributes, or `None` for a level in a program's own body.
    fn compiled_method_site(&self, line: usize, text: &[u8], indent: usize) -> Option<FailureSite> {
        let program_id = self.running_activation()?.program_id;
        let name = self.compiled_method_names.get(&program_id)?.to_vec();
        Some(FailureSite::Named {
            line,
            indent,
            text: text.to_vec(),
            name,
        })
    }

    /// The frame a level in the interpreter's own library contributes, or
    /// `None` for a level in a program's own package.
    fn sourceless_site(&mut self, line: usize, indent: usize) -> Option<FailureSite> {
        let program_id = self.running_activation()?.program_id;
        if !self.library_programs.contains(&program_id) {
            return None;
        }
        // Copied out rather than borrowed: the scope's `~id` is read through
        // `self.classes()`, which takes `&mut self`.
        let identity = self
            .activation()
            .method_identity
            .as_ref()
            .map(|identity| (identity.name.to_vec(), identity.scope));
        let text = match identity {
            Some((name, scope)) => {
                let scope = self.classes().id_string(scope).to_string();
                Raised::sourceless_method_line(&name, &scope, crate::LIBRARY_PACKAGE_NAME)
            }
            // The library's own prologue, which is a program rather than a
            // method. Reachable only if the bootstrap itself fails, which
            // ends the interpreter -- rendered rather than left to the
            // clause echo so that path does not print a library source line
            // the oracle would not.
            None => Raised::sourceless_program_line(crate::LIBRARY_PACKAGE_NAME),
        };
        Some(FailureSite::Named {
            line,
            indent,
            text,
            name: crate::LIBRARY_PACKAGE_NAME.to_vec(),
        })
    }

    /// Captures a `LEAVE`/`ITERATE` instruction's own clause site and static
    /// indent the instant it steps, before any propagation -- see
    /// `Flow::Leave`'s own doc comment for why eagerly, and `LeaveOrigin`'s
    /// own doc comment for why `indent` is computed here rather than read
    /// back later. `clause_site` is the free function `record_failure_site`
    /// itself resolves through, shared rather than duplicated: this needs
    /// the same (line, text) pair, just held onto instead of assigned to
    /// `self.failure_site` immediately, since a `LEAVE`/`ITERATE` might
    /// still be consumed by an enclosing `Do`/`Select` rather than ever
    /// becoming a failure at all.
    pub(crate) fn leave_origin(
        &self,
        code: &Code<'_>,
        index: usize,
        source: Option<&ProgramSource>,
        instruction: &Instruction,
    ) -> LeaveOrigin {
        LeaveOrigin {
            site: self.clause_site(source, instruction),
            // `+ self.indent_offset` (F-EX1's own correction to F3,
            // `lib.rs`'s own doc comment): found missing here on the
            // *second* re-verification of F-EX1's own fix, not the first --
            // a `LEAVE`/`ITERATE` inside an escaped `OTHERWISE`'s own body
            // captures its own origin indent here, not through `step_in_
            // temps_frame`'s own computation at all (`Flow::Leave`'s own
            // doc comment: eagerly, before any propagation), so it needs
            // the identical addition independently, not by inheritance.
            indent: self.printed_indent(code, index),
            // Already this instruction's own line: `Op::Clause`'s region's
            // `in_clause` set it before dispatching this `step`, through the
            // same `clause_line` call `SIGL` reads.
            clause_line: self.clause_state.line(),
        }
    }

    /// Assigns `origin`'s own captured site to `self.failure_site`, first
    /// call wins, at `origin`'s own captured indent.
    fn record_leave_failure(&mut self, origin: &LeaveOrigin) {
        self.record_failure_site_at(origin.site.clone(), origin.indent);
    }

    /// The same first-wins record from an already-resolved `(line, text)`
    /// pair rather than from an `&Instruction`, for the one caller that has
    /// no instruction left to resolve: a loop re-test blamed on the
    /// `ITERATE` that transferred control back to it, whose site was
    /// captured a pass earlier (`HeaderClause::Iterate`). `indent` is the
    /// caller's, because that blame prints at the loop body's indent rather
    /// than at the `ITERATE`'s own -- see that variant's doc comment.
    fn record_failure_site_at(&mut self, site: Option<(usize, Vec<u8>)>, indent: usize) {
        if self.failure_site.is_some() {
            return;
        }
        if let Some((line, text)) = site {
            self.failure_site = Some(FailureSite::Clause { line, text, indent });
        }
    }

    /// The clause boundary a promoted construct owes once the branch it chose
    /// has finished, which flattening removed.
    /// ```text
    /// call on user zx name h        /* h raises zy */
    /// call on user zy name g        /* g says SIGL */
    /// select
    /// when 1 = 1 then zq = raiser()
    /// end
    /// say 'after'
    /// ```
    pub(crate) fn end_promoted_branch(
        &mut self,
        code: &Code<'_>,
        flow: Flow,
    ) -> Result<Flow, Failure> {
        let line = self.clause_state.line();
        match self.in_clause(code, line, move |_| Ok(flow))? {
            ClauseOutcome::Ran(ran) => ran,
            ClauseOutcome::Ended(exit) => Ok(Flow::Exit(exit.value())),
        }
    }

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
    fn pop_search_frame(
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

    /// Runs `code.body.instructions[start..end]` in place, one instruction at
    /// a time through `Op::Clause`'s region, and answers what happened.
    fn run_bounded(
        &mut self,
        code: &Code<'_>,
        start: usize,
        end: usize,
        source: Option<&ProgramSource>,
        engine: BodyEngine<'_>,
    ) -> Result<Flow, Failure> {
        let BodyEngine::Chunk { chunk, registers } = engine;
        self.run_bounded_from_chunk(code, chunk, registers, start, end, source)
    }

    /// One header value's own `>K>` line, at the `DO`/`LOOP` clause's own
    /// indent, for the roles the oracle echoes one for.
    pub(crate) fn echo_header_value(&mut self, role: HeaderRole, value: ObjRef) {
        let Some(keyword) = role.keyword() else {
            return;
        };
        // `trace_keyword` carries its own `results` gate, so this is not a second
        // decision about whether to *print*: it decides whether to render the
        // value into a `Vec` at all, which an untraced run has no use for. The
        // same shape and the same reason as `bind_control`'s own check, one level
        // down from where that one sits.
        if !self.trace_mode().results {
            return;
        }
        let text = self.string_value_text(value);
        self.trace_keyword(self.clause_state.current_value_indent, keyword, &text);
    }

    /// Validates one header value against whatever its role requires and files
    /// it in `values`.
    pub(crate) fn accept_header_value(
        &mut self,
        role: HeaderRole,
        value: ObjRef,
        values: &mut LoopHeaderValues,
    ) -> Result<(), Failure> {
        match role {
            // **The rounded value and the representation it arrived in.** The
            // rounding is the oracle's, and for an integer inside `DIGITS` it
            // is a no-op, so the two arms carry the same worth and differ only
            // in what [`ControlValue::small`] may then answer.
            HeaderRole::Initial => {
                let digits = self.activation().settings.digits();
                // **The `Number` is built only by the arm that keeps it.**
                // `ControlValue::Small` carries the `i64` the tag already
                // holds, so asking `header_number` first and then discarding
                // its answer builds a digit vector for every ordinary counted
                // loop and throws it away. What that call would have checked
                // -- an object in an operand position, a non-numeric value --
                // a tagged integer cannot fail.
                values.initial = Some(match value.decode() {
                    Decoded::SmallInt(small) if within_digits(small, digits) => {
                        ControlValue::Small(small)
                    }
                    _ => ControlValue::Wide(self.header_number(role, value)?),
                });
            }
            HeaderRole::To => values.to = Some(self.header_number(role, value)?),
            HeaderRole::By => values.by = Some(self.header_number(role, value)?),
            // **The rendering is built inside the failing arm**, because a
            // count is nearly always whole and the copy only ever reaches the
            // message: hoisting it renders and frees a string per loop header
            // to serve a branch that is not taken.
            // **`exprf` and `exprr` are the two `reqstr` contexts in a `DO`
            // header, and the three numeric positions above are not.**
            // `ForLoop::setup` and `DoWhile::setup` convert with
            // `result->requestString()->numberString()`
            // (`instructions/DoBlockComponents.cpp:88`) where
            // `ControlledLoop::setup` sends `callOperatorMethod(OPERATOR_PLUS)`
            // to the value instead (`:127`), which is a message to the object
            // and is 97.1 rather than a conversion -- `eval.rs`'s
            // `object_operand_tests` is that half.
            HeaderRole::For | HeaderRole::OverFor => {
                let converted = self.required_string_value(value)?;
                values.for_remaining = Some(match self.whole_nonneg(converted) {
                    Some(count) => count,
                    None => {
                        let found = self.string_value_text(value);
                        return Err(raised_for_count_not_whole(&found).into());
                    }
                });
            }
            HeaderRole::Count => {
                let converted = self.required_string_value(value)?;
                values.count = Some(match self.whole_nonneg(converted) {
                    Some(count) => count,
                    None => {
                        let found = self.string_value_text(value);
                        return Err(raised_repetition_count_not_whole(&found).into());
                    }
                });
            }
            // **`DO OVER` is not `stringValue()` and not an operator**, which
            // is why R12's other sites do not cover it: the oracle hands the
            // target to `requestArray`. Measured, `do e over .array` is
            // 98.913 at rc 158 and `do e over .environment` iterates the
            // directory's own entries, neither of which this crate answers.
            // An array and a string both do -- see [`Interp::over_snapshot`].
            HeaderRole::Over => {
                if let Some(kind) = self.over_target_gap(value) {
                    return Err(Loud::object_position(role.value_name(), kind).into());
                }
                values.over = Some(value);
            }
        }
        Ok(())
    }

    /// `OverLoop::setup`'s conversion
    /// (`instructions/DoBlockComponents.cpp:233`): the array whose items a
    /// `DO OVER` binds in turn.
    fn over_target_array(&mut self, value: ObjRef) -> Result<ObjRef, Failure> {
        if self.array_slots_of(value).is_some() {
            return Ok(value);
        }
        let converted = self.request_array_for_over(value)?;
        if let Some(converted) = converted
            && self.array_slots_of(converted).is_some()
        {
            return Ok(converted);
        }
        let found = self.string_value_text(value);
        Err(Raised::object_not_single_dimensional(&found).into())
    }

    /// [`Interp::over_target_array`]'s `requestArray` limb, which is a message
    /// send on one path and a direct call on the other.
    fn request_array_for_over(&mut self, value: ObjRef) -> Result<Option<ObjRef>, Failure> {
        let caller = self.caller();
        if self.is_base_class(value) {
            if self.lookup(value, b"MAKEARRAY", None).is_none() {
                return Ok(None);
            }
            return self.send_message(value, b"MAKEARRAY", None, &[], caller);
        }
        let wanted = self.text_built(b"ARRAY".to_vec());
        self.roots.push_temp(wanted);
        self.send_message(value, b"REQUEST", None, &[Some(wanted)], caller)
    }

    /// The one `DO OVER` target this crate still refuses: one of the
    /// interpreter's own directories.
    fn over_target_gap(&mut self, value: ObjRef) -> Option<&'static str> {
        if !matches!(
            self.heap.get(value).map(|object| &object.body),
            Some(Body::Native(_))
        ) {
            return None;
        }
        (self.receiver_class_id(value).as_deref() == Some("Directory"))
            .then_some("one of the interpreter's own objects")
    }

    /// Whether `value` is a `StringTable` -- `.methods`, `.routines`,
    /// `.resources`, or a package's `~publicClasses`.
    fn is_hash_collection(&mut self, value: ObjRef) -> bool {
        let Some(Body::Native(native)) = self.heap.get(value).map(|object| &object.body) else {
            return false;
        };
        let class = native.class();
        class == self.object_model().iterable_collection_class()
    }

    /// The values a `DO OVER` binds its control variable to: the **non-empty**
    /// slots of what [`Interp::over_target_array`] converts the target into.
    fn over_snapshot(&mut self, value: ObjRef) -> Result<(ObjRef, Vec<ObjRef>), Failure> {
        // The one collection whose order is this crate's rather than the
        // oracle's answers from its own walk -- see
        // [`Interp::hash_collection_indexes`] for why it is sorted and what
        // that costs. It cannot go through `MAKEARRAY` either: a
        // `Body::Native` receiver is not the store `hash.rs` owns.
        let items: Vec<ObjRef> = if self.is_hash_collection(value) {
            self.hash_collection_indexes(value)
        } else {
            let array = self.over_target_array(value)?;
            // Rooted before its slots are read out: a converted array is a
            // different object from the target the source named, and its items
            // are reachable only through it.
            if array != value {
                self.roots.push_temp(array);
            }
            match self.heap.get(array).map(|object| &object.body) {
                Some(Body::Array { slots, .. }) => slots.iter().flatten().copied().collect(),
                _ => vec![array],
            }
        };
        // Every item is already reachable here, which is what `alloc_with`
        // collecting before it allocates asks of this site.
        let snapshot = self.alloc_with(
            BehaviourId::ARRAY,
            Body::array(items.iter().copied().map(Some).collect()),
        );
        self.roots.push_temp(snapshot);
        debug_assert_eq!(
            self.array_slots(snapshot)
                .map(|slots| slots.iter().copied().flatten().collect::<Vec<_>>())
                .as_deref(),
            Some(items.as_slice()),
            "a DO OVER's cursor is not the snapshot's own slots, so the snapshot does not root it"
        );
        Ok((snapshot, items))
    }

    /// A `StringTable`'s indexes, as the values a `DO OVER` binds in turn.
    fn hash_collection_indexes(&mut self, table: ObjRef) -> Vec<ObjRef> {
        let Some(Body::Native(native)) = self.heap.get(table).map(|object| &object.body) else {
            return Vec::new();
        };
        let mut keys = native.keys();
        keys.sort_unstable();
        let mut items = Vec::with_capacity(keys.len());
        for key in &keys {
            let item = self.text(key);
            self.roots.push_temp(item);
            items.push(item);
        }
        items
    }

    /// One controlled-loop header value as the `Number` the loop runs on:
    /// numeric (41.1 if not) and rounded at the digits in force.
    fn header_number(&mut self, role: HeaderRole, value: ObjRef) -> Result<Number, Failure> {
        let entry_digits = self.activation().settings.digits();
        // **A tagged integer no wider than `DIGITS` is its own rounding**, so
        // the unary `+` below has nothing to do to it and the general path
        // only spends: `arith_operand` builds a `Number`, the addition copies
        // it into a zero-left fast path, rounds it, and copies it out again.
        // Neither of the two checks the general path makes can fail here --
        // the tag holds no object for `operator_operand_gap` to find, and an
        // integer is numeric -- so this answers directly.
        if let Decoded::SmallInt(small) = value.decode()
            && within_digits(small, entry_digits)
        {
            let shortcut = Number::from_i64(small);
            // The tripwire, not the test: a disagreement is invisible in the
            // answer for every program that stays inside the tag, which is
            // why it is asserted on every header of every program the debug
            // gate runs rather than probed once.
            debug_assert_eq!(
                Ok(&shortcut),
                round_via_unary_plus(&Number::from_i64(small), entry_digits).as_ref(),
                "a tagged integer within DIGITS {entry_digits} is not its own unary +"
            );
            return Ok(shortcut);
        }
        if let Some(kind) = self.operator_operand_gap(value) {
            return Err(Loud::object_position(role.value_name(), kind).into());
        }
        let result = self.header_number_body(value, entry_digits);
        // Blamed on any failure past the object-position check above, not
        // only `arith_operand`'s own conversion -- the same reason
        // `Interp::arith_general` (`eval.rs`) blames its own whole
        // computation rather than only `Interp::arith_left_operand`'s.
        // Measured: `numeric digits 1; s. = '9.9E999999999'; do i = s. to
        // 5` is 42.901 with the frame, past a conversion that already
        // succeeded -- `round_via_unary_plus`'s own range check is what
        // raises, still inside the same forwarded unary `+`.
        if result.is_err() {
            self.blame_stem_forwarded_operator(b"+", value);
        }
        result
    }

    /// [`Interp::header_number`]'s own computation for a position that
    /// reaches it (`Initial`, `To` and `By` -- see that function's own
    /// doc), wrapped by it so every failing step is blamed once.
    fn header_number_body(&mut self, value: ObjRef, entry_digits: u64) -> Result<Number, Failure> {
        let operand = self.arith_operand(value)?;
        // The header rounds through a unary `+`, so its three numeric
        // positions carry an operand exactly as an operator's do -- measured,
        // `do k = 1 to 123456789` and `... by 123456789` are both 98.972 at
        // DIGITS 3.
        self.lostdigits_check(&operand, value)?;
        Ok(round_via_unary_plus(&operand, entry_digits).map_err(Raised::from)?)
    }

    /// A `DO`/`LOOP` past its header: the construct itself, driven from the
    /// values whichever engine evaluated that header produced.
    #[allow(
        clippy::too_many_arguments,
        reason = "the same argument `run_repeating`'s own allow makes: every parameter is state one DO/LOOP needs"
    )]
    pub(crate) fn run_loop_with_header(
        &mut self,
        code: &Code<'_>,
        index: usize,
        instruction: &Instruction,
        body: &Loop,
        source: Option<&ProgramSource>,
        engine: BodyEngine<'_>,
        values: LoopHeaderValues,
    ) -> Result<Flow, Failure> {
        // **The refusal, here rather than at the driver's own entry.**
        // `loop_header_plan` answers `None` for
        // exactly the three forms this crate does not run, and every arm below
        // relies on that answer: `LoopKind::With` has no `LoopState`, and a
        // `COUNTER` or a stem `OVER` reaches an `expect` on a value nothing
        // evaluated. Measured, before this check existed: all three panicked on
        // the compiled stream, and no test in the workspace was red.
        if loop_header_plan(body).is_none() {
            return Err(Loud::instruction(&instruction.kind).into());
        }
        let body_start = index + 1;
        let end_index = body
            .end
            .expect("an unclosed DO/LOOP is error 14.1/14.5, so a body that parsed has this set");
        let resume = end_index + 1;
        let label = body.label;

        let state = match &body.kind {
            // A block, not a loop: exactly one pass, and `WHILE`/`UNTIL`
            // can never be present (`create_loop`'s own parser only reaches
            // `LoopKind::Simple` through the bare, at-end-of-clause `DO`
            // arm, before any conditional is even looked for). Still
            // leavable by an explicit `DO LABEL`, never by a bare `LEAVE`
            // (`do_body_outcome`'s own `is_loop: false`) -- measured, a
            // labelled simple block is leavable but an unlabelled one is
            // 28.1 on a bare `LEAVE` reaching it.
            LoopKind::Simple => {
                // Captured before the body runs, for the same reason
                // `run_repeating` captures it: this is the `DO`'s own indent,
                // and `current_value_indent` holds whatever the last body
                // clause left once `run_bounded` has returned.
                let do_indent = self.clause_state.current_value_indent;
                // **The block's own clause, opened and ended before any body
                // instruction runs.** The oracle's
                // `RexxInstructionSimpleDo::execute` traces the instruction,
                // opens the block and returns -- it never runs the body -- so
                // a condition queued before the `DO` is delivered here, at the
                // `DO`'s own line and with the block open. Measured, a handler
                // that requeues ahead of `do` / `say 'body'` / `end`: the
                // oracle runs the requeued handler with `SIGL` naming the `DO`
                // and prints its output ahead of `body`.
                let do_line = self
                    .clause_line_at(code, index, instruction, source)
                    .unwrap_or_else(|| self.clause_state.line());
                match self.in_clause(code, do_line, |it| {
                    // A block that opened, so the boundary's own handler runs
                    // one level in -- the `select case raiser()` row of
                    // `settle_block_indent`'s table, and the same call.
                    it.settle_block_indent(true, do_indent);
                    Ok(())
                })? {
                    ClauseOutcome::Ended(exit) => return Ok(Flow::Exit(exit.value())),
                    ClauseOutcome::Ran(ran) => ran?,
                }
                let flow = self.run_bounded(code, body_start, end_index, source, engine)?;
                return match self.do_body_outcome(code, index, label, false, resume, flow)? {
                    DoOutcome::Escaped(escape) => Ok(escape),
                    // Falls through to `END`, which `run_bounded`'s own
                    // `[body_start, end_index)` range never visits (the
                    // `Goto(resume)` below jumps straight past it) --
                    // unlike a repeating loop, a `Simple` block never runs
                    // this arm again, so one explicit echo here is the
                    // whole story, not a per-pass one the way
                    // `run_repeating`'s own is. Measured, Task 11's
                    // report: `if 1 = 1 then do / say 'x' / end` traces
                    // `end` on its own line even though the block never
                    // repeats.
                    DoOutcome::FellThrough | DoOutcome::Iterated { .. } => {
                        // **`END` is a clause of its own, and its boundary is
                        // where a condition queued by the block's last body
                        // clause is delivered.** The oracle executes `END` as
                        // an instruction, so a handler requeued inside the
                        // block runs there rather than after the whole
                        // construct. Measured, `do` / `end` with a handler
                        // that requeues twice: the oracle reports the second
                        // handler's `SIGL` as `END`'s line, ahead of the
                        // `SAY` that follows the block; without this clause
                        // it ran after that `SAY` had already printed, on
                        // both engines.
                        let end_instruction = &code.body.instructions[end_index];
                        let end_line = self
                            .clause_line_at(code, end_index, end_instruction, source)
                            .unwrap_or_else(|| self.clause_state.line());
                        // A fresh computation, not `current_value_indent` --
                        // `run_bounded`, just above, has already stepped this
                        // block's own body, so that field now holds whatever
                        // the *last* body instruction left it at, not this
                        // `DO`'s own. `printed_indent` for the same reason
                        // every other site on this page uses it: consistency
                        // if this `Simple` block's own `END` is ever itself
                        // the direct landing point of an escape (untested,
                        // but cheap to keep uniform rather than silently
                        // exempt).
                        let end_indent = self.printed_indent(code, index);
                        match self.in_clause(code, end_line, |it| {
                            if it.trace_mode().all
                                && let Some((line, text)) = it.clause_site(source, end_instruction)
                            {
                                it.trace_clause(line, end_indent, &text);
                            }
                            // The block is closed by the time this clause's
                            // boundary runs, so its handler sits back out at
                            // the `DO`'s own level.
                            it.settle_block_indent(false, do_indent);
                            Ok(())
                        })? {
                            ClauseOutcome::Ended(exit) => return Ok(Flow::Exit(exit.value())),
                            ClauseOutcome::Ran(ran) => ran?,
                        }
                        Ok(Flow::Goto(resume))
                    }
                };
            }
            LoopKind::Forever => LoopState::Forever,
            // A bare `DO` with no expression at all runs a single pass,
            // matching `Simple`'s own behaviour -- see `loop_header_plan`'s
            // own note on why nothing in this crate's tests reaches it.
            LoopKind::Count(_) => LoopState::Count {
                remaining: values.count.unwrap_or(1),
            },
            LoopKind::Controlled(ctrl) => LoopState::Controlled {
                control: ctrl.control,
                at: control_slot(code, ctrl.control),
                // The header's own value, in the representation the header
                // produced it in. The first re-test replaces it, computing its
                // own representation the same way.
                current: values
                    .initial
                    .expect("a controlled loop's plan always names its initial value"),
                to: values.to,
                by: match values.by {
                    Some(by) => by,
                    // No `round_via_unary_plus` needed on the default: a bare
                    // literal `1` is already whole at any width, so rounding
                    // it at the header's digits could only ever answer `1`
                    // again.
                    None => Number::one(),
                },
                for_remaining: values.for_remaining,
                cached_digits: u64::MAX,
                to_int: None,
                by_int: None,
                shape: shape_of(code.symbols.name(ctrl.control).as_bytes()),
                stepped: false,
            },
            LoopKind::Over { control, .. } => {
                let (snapshot, items) = self.over_snapshot(
                    values
                        .over
                        .expect("a DO OVER's plan always names its target"),
                )?;
                LoopState::OverItems {
                    control: *control,
                    at: control_slot(code, *control),
                    snapshot,
                    items,
                    next: 0,
                    remaining: values.for_remaining,
                }
            }
            LoopKind::With { .. } => unreachable!("DO WITH takes the loud path above"),
        };
        self.run_repeating(
            code,
            index,
            instruction,
            body_start,
            end_index,
            resume,
            label,
            body.conditional.as_ref(),
            source,
            state,
            engine,
        )
    }

    /// Leaves `current_value_indent` at the indent a **block instruction's**
    /// own clause **boundary** runs at: one level in when that instruction's
    /// block is still open once the clause has finished, the clause's own
    /// indent when it is not.
    /// ```text
    /// do zi = 1 to raiser()      raiser returns 1 -> 10 *-*     h:
    /// do zi = 1 to raiser()      raiser returns 0 -> 10 *-*   h:
    /// do while raiser() < 1      test false       -> 10 *-*   h:
    /// do while raiser() < 1      test true        -> 12 *-*     h:
    /// do until raiser() > 0      test true        -> 10 *-*   h:
    /// do until raiser() > 0      test false       -> 13 *-*     h:
    /// select case raiser()       always open      -> 12 *-*     h:
    /// ```
    fn settle_block_indent(&mut self, open: bool, clause_indent: usize) {
        self.clause_state.current_value_indent = if open {
            clause_indent + 2
        } else {
            clause_indent
        };
    }

    /// The shared driver for every repeating `LoopKind` (everything but
    /// `Simple`, which never repeats and runs through
    /// `run_loop_with_header`'s own arm directly):
    /// advance-test-run-test-advance, in the order the oracle is measured to
    /// use it in (`report`'s own transcripts) --
    /// `WHILE` tested before the body, `UNTIL` after, and a `LEAVE`/
    /// `ITERATE` handled identically to falling off the bottom of the body
    /// normally, because that is what the oracle's own `ITERATE` does
    /// (measured: `do until n = 1 / n = n + 1 / if n = 1 then iterate / ...`
    /// terminates immediately rather than skipping straight to the next
    /// pass's top, because `ITERATE` jumps to the loop's own
    /// bottom-of-iteration bookkeeping, which for an `UNTIL` loop includes
    /// testing `UNTIL` right there -- see the report for the full
    /// transcript).
    #[allow(
        clippy::too_many_arguments,
        reason = "every parameter is load-bearing state one repeating DO/LOOP needs; splitting it into a struct is Task 13's to consider if it too needs this shape"
    )]
    fn run_repeating(
        &mut self,
        code: &Code<'_>,
        do_index: usize,
        do_instruction: &Instruction,
        body_start: usize,
        end_index: usize,
        resume: usize,
        label: Option<SymbolId>,
        conditional: Option<&LoopConditional>,
        source: Option<&ProgramSource>,
        mut state: LoopState,
        engine: BodyEngine<'_>,
    ) -> Result<Flow, Failure> {
        // The loop's own two spaces of indent, added once here rather than
        // per-check: `static_indent(&code.body.instructions, do_index)` is
        // what a control-setup failure at `do_index` itself already
        // reports (measured: `do i = 1 to 3 for 1/0` is unindented at top
        // level), and `WHILE`/`UNTIL` both report two spaces *more* than
        // that (measured: `do while 1/0` at top level is indented two).
        // Captured from `current_value_indent` once, here, rather than
        // recomputed: `Op::Clause`'s region already set it to exactly this
        // value (`indent_offset` included) for this same `DO`/`LOOP`
        // instruction, and every caller into this function reaches it
        // through nothing but `self.eval` calls in between (never another
        // instruction step), so it has not moved.
        let do_indent = self.clause_state.current_value_indent;
        let loop_indent = do_indent + 2;
        // `TRACE`'s own per-iteration re-echo (D17, this task's report,
        // "Step 6"): the oracle's `DO`/`LOOP` instruction is re-executed
        // once per pass (`DoBlock::checkControl`, read directly), so its
        // own clause -- and `END`'s -- echo again on every pass, unlike
        // every other construct in this crate, which resolves its whole
        // repetition inside one `step` call and so is stepped, and echoed,
        // exactly once (`Op::Clause`'s own doc comment). `false`
        // on entry because the *first* pass's echo already happened there,
        // before `run_loop_with_header` ever called into this function.
        let is_until_loop = matches!(conditional, Some(cond) if cond.until);
        let mut first_pass = true;
        // Which clause the loop header's own evaluation belongs to on this
        // pass -- `HeaderClause`'s own doc comment has the oracle mechanism
        // and the three measured transcripts.
        let mut header_clause = HeaderClause::Do;
        let mut iterate_site: IterateSite = None;
        let end_line = self
            .clause_line_at(code, end_index, &code.body.instructions[end_index], source)
            .unwrap_or(0);
        // Hoisted out of the loop body (review round 1, F2): the header's own
        // failure path needs it one statement *before* the body that used to
        // bind it. Derived from `code`, so it borrows nothing this function
        // mutates.
        let end_instruction = &code.body.instructions[end_index];

        loop {
            if !first_pass
                && !is_until_loop
                && self.trace_mode().all
                && let Some((line, text)) = self.clause_site(source, do_instruction)
            {
                self.trace_clause(line, do_indent, &text);
            }
            first_pass = false;

            // **The `DO` clause, entered and ended like any other** (fix
            // round 3). The header's control expressions and a `WHILE` test
            // are Rexx clauses in their own right: measured, `do while zn <
            // sub()` reports `SIGL` 4 -- the `DO` clause's own line -- for
            // the first test, and delivers a `CALL ON` handler queued by
            // `sub()` right there rather than after the whole loop.
            let do_line = self
                .clause_line_at(code, do_index, do_instruction, source)
                .unwrap_or_else(|| self.clause_state.line());
            let header_line = match header_clause {
                HeaderClause::Do => do_line,
                HeaderClause::End => end_line,
                HeaderClause::Iterate { line } => line,
            };
            let header = self.in_clause(code, header_line, |it| {
                // **A header that fails is blamed on the clause that
                // transferred control back here, at the loop body's indent**
                // -- not on the `DO` clause, which is where the enclosing
                // `Op::Clause`'s region would put it (review round 1, F2).
                // Reachable since the control variable is genuinely re-read:
                // a body that leaves it non-numeric fails the `BY` addition
                // on the next re-test. Measured, three shapes, all `trace r`
                // with `ii = 'abc'` in a `do ii = 1 to 3` body:
                let outcome = 'header: {
                    let advanced = match it.loop_advance(code, &mut state, do_indent, loop_indent) {
                        Ok(advanced) => advanced,
                        Err(failure) => {
                            match header_clause {
                                HeaderClause::Do => {}
                                HeaderClause::End => {
                                    it.record_failure_at(source, end_instruction, loop_indent);
                                }
                                HeaderClause::Iterate { .. } => {
                                    it.record_failure_site_at(iterate_site.clone(), loop_indent);
                                }
                            }
                            return Err(failure);
                        }
                    };
                    if !advanced {
                        break 'header HeaderOutcome::Stop;
                    }
                    if let Some(cond) = conditional
                        && !cond.until
                    {
                        // Overrides `Op::Clause`'s region's own setting of
                        // `current_value_indent` (to `do_indent`, from stepping
                        // the `DO`/`LOOP` instruction itself) -- `WHILE`'s own
                        // condition is evaluated here, inside that same `step`
                        // call, never through a `Op::Clause`'s region of its own.
                        it.clause_state.current_value_indent = loop_indent;
                        match it.eval_condition(
                            code,
                            &cond.condition,
                            ConditionTrace::Keyword(loop_indent, "WHILE"),
                            raised_while_not_logical,
                        ) {
                            Ok(true) => {}
                            Ok(false) => break 'header HeaderOutcome::Stop,
                            Err(failure) => {
                                it.record_failure_at(source, do_instruction, loop_indent);
                                return Err(failure);
                            }
                        }
                    }
                    HeaderOutcome::Continue
                };
                it.settle_block_indent(outcome.entered(), do_indent);
                Ok(outcome)
            })?;
            match header {
                ClauseOutcome::Ended(exit) => return Ok(Flow::Exit(exit.value())),
                ClauseOutcome::Ran(Err(failure)) => return Err(failure),
                ClauseOutcome::Ran(Ok(HeaderOutcome::Stop)) => return Ok(Flow::Goto(resume)),
                ClauseOutcome::Ran(Ok(HeaderOutcome::Continue)) => {}
            }

            let flow = self.run_bounded(code, body_start, end_index, source, engine)?;
            match self.do_body_outcome(code, do_index, label, true, resume, flow)? {
                DoOutcome::Escaped(escape) => return Ok(escape),
                // **`END` is not reached at all when an `ITERATE` ended the
                // pass**, so it neither echoes nor owns the re-test. Measured
                // (fix round 4, found while measuring NEW-1's `SIGL`
                // divergence): under `trace r`, `do while zn < 2 / zn = zn +
                // 1 / iterate / end` echoes `iterate` and then the `do`
                // clause again, with no `end` line between them, where the
                // same loop without the `ITERATE` does echo `end`. The
                // oracle's reason is structural: `END`'s own `execute` is
                // what calls `reExecute` on a fall-through, and
                // `RexxActivation::iterate` is what calls it for an
                // `ITERATE` -- `END` is jumped straight over.
                DoOutcome::Iterated { line, site } => {
                    header_clause = HeaderClause::Iterate { line };
                    iterate_site = site;
                }
                // Reached only when the body fell off its end. **Not**
                // reached on a matched `LEAVE`, which returns above instead
                // -- measured, this task's report (`DO FOREVER` with a
                // `LEAVE` on the second pass): `END` never echoes for that
                // final pass, only for a pass that genuinely falls through
                // to it.
                DoOutcome::FellThrough => {
                    header_clause = HeaderClause::End;
                    if self.trace_mode().all
                        && let Some((line, text)) = self.clause_site(source, end_instruction)
                    {
                        self.trace_clause(line, do_indent, &text);
                    }
                }
            }

            if let Some(cond) = conditional
                && cond.until
            {
                // **F4's own sibling, found while re-verifying this task's
                // review fixes rather than assumed clean**: `UNTIL`'s own
                // check needs a *second*, unconditional re-echo of the
                // `DO`/`LOOP` clause here, not only the top-of-loop one
                // above. Measured: `do until n = 1 / n = n + 1 / end`
                // re-echoes `do until ...` a second time, after `END`,
                // before testing `UNTIL` at all -- even on the very first
                // test, which runs after the body's only pass and before
                // the top-of-loop re-echo (gated on `!first_pass`) would
                // ever fire again. The oracle's own `DO`/`LOOP` instruction
                // is re-entered to make *this* decision too, exactly like
                // it is to test `WHILE` or advance a `Controlled` loop
                // (`checkControl`, read directly, this task's report) --
                // `UNTIL`'s decision point is not the same event as the
                // top-of-loop one, so it needs its own echo unconditionally
                // rather than sharing `first_pass`'s gate.
                if self.trace_mode().all
                    && let Some((line, text)) = self.clause_site(source, do_instruction)
                {
                    self.trace_clause(line, do_indent, &text);
                }
                // Same override as `WHILE`'s own, above -- the re-echoed
                // `END` clause just before this point left
                // `current_value_indent` untouched (its own `trace_clause`
                // call does not set it), so without this `UNTIL`'s
                // intermediates would otherwise still read `do_indent`.
                self.clause_state.current_value_indent = loop_indent;
                // `UNTIL`'s test belongs to the same clause the *next*
                // top-of-loop re-test does, and for the same reason: in the
                // oracle they are one event, `reExecute` called by whichever
                // instruction transferred control back to the loop. Measured
                // with an `ITERATE` in the body, which is what tells the two
                // candidates apart -- `do until zs() >= 2` with `if zn = 1
                // then iterate` on line 4 reports `4` for the first test and
                // `6` (the `END` line) for the second.
                let until_line = match header_clause {
                    HeaderClause::Do => do_line,
                    HeaderClause::End => end_line,
                    HeaderClause::Iterate { line } => line,
                };
                // **This clause's boundary is unobservable on every probe
                // tried, and it is here because it cannot be separated from
                // the line.**
                // Round 3 shipped the line and the boundary as two calls, and
                // re-review 3 measured that replacing the boundary half alone
                // changed nothing on any of its 38 probes: between this test
                // and the next top-of-loop test no user clause runs and
                // nothing re-sets the clause line, so whichever of the two
                // boundaries fires first delivers at the same line. With
                // `in_clause` there is no half to remove -- the mutation
                // "keep the line, drop the boundary" is not expressible, and
                // dropping both is what `a_while_retest_belongs_to_the_do_
                // clause_then_to_the_end_clause` and `a_loop_retest_after_
                // an_iterate_belongs_to_the_iterate_clause` fail on.
                let tested = self.in_clause(code, until_line, |it| {
                    let held = it.eval_condition(
                        code,
                        &cond.condition,
                        ConditionTrace::Keyword(loop_indent, "UNTIL"),
                        raised_until_not_logical,
                    )?;
                    // An `UNTIL` that held ends the loop, so this clause's
                    // own boundary sits outside the block -- the mirror of
                    // the top-of-loop test's, and the same call.
                    it.settle_block_indent(!held, do_indent);
                    Ok(held)
                })?;
                match tested {
                    ClauseOutcome::Ended(exit) => return Ok(Flow::Exit(exit.value())),
                    ClauseOutcome::Ran(Ok(true)) => return Ok(Flow::Goto(resume)),
                    ClauseOutcome::Ran(Ok(false)) => {}
                    ClauseOutcome::Ran(Err(failure)) => {
                        self.record_failure_at(source, end_instruction, loop_indent);
                        return Err(failure);
                    }
                }
            }
            // Nothing happens at the bottom of a pass any more. A
            // `Controlled` loop's `BY` increment used to, as `loop_step`;
            // Task 9 moved it into `loop_advance`, where the oracle does it,
            // because the two `>>>` lines the oracle traces straddle that
            // addition and the value on the near side of it is gone by the
            // time the next `loop_advance` runs. `loop_advance`'s own doc
            // comment has the citation and why nothing else moved with it.
        }
    }

    /// What one repeating `Do`/`Loop`'s own body just produced, translated
    /// into what `run_repeating`/`run_loop_with_header`'s own `Simple` arm
    /// does next.
    fn do_body_outcome(
        &mut self,
        code: &Code<'_>,
        do_index: usize,
        label: Option<SymbolId>,
        is_loop: bool,
        resume: usize,
        flow: Flow,
    ) -> Result<DoOutcome, Failure> {
        let owns_frame = is_loop || label.is_some();
        match flow {
            Flow::Next => Ok(DoOutcome::FellThrough),
            Flow::Leave(name, origin) => {
                let matched = match name {
                    None => is_loop,
                    Some(n) => label == Some(n),
                };
                if matched {
                    Ok(DoOutcome::Escaped(Flow::Goto(resume)))
                } else if owns_frame {
                    Ok(DoOutcome::Escaped(Flow::Leave(
                        name,
                        self.pop_search_frame(code, do_index, origin),
                    )))
                } else {
                    Ok(DoOutcome::Escaped(Flow::Leave(name, origin)))
                }
            }
            Flow::Iterate(name, origin) => {
                let matched = match name {
                    None => is_loop,
                    Some(n) => label == Some(n),
                };
                if !matched {
                    return Ok(DoOutcome::Escaped(Flow::Iterate(
                        name,
                        if owns_frame {
                            self.pop_search_frame(code, do_index, origin)
                        } else {
                            origin
                        },
                    )));
                }
                if !is_loop {
                    let name = name.expect(
                        "is_loop is false and matched is true only through the named branch above",
                    );
                    self.record_leave_failure(&origin);
                    return Err(
                        raised_iterate_wrong_kind(code.symbols.name(name).as_bytes()).into(),
                    );
                }
                Ok(DoOutcome::Iterated {
                    line: origin.clause_line,
                    site: origin.site,
                })
            }
            other => Ok(DoOutcome::Escaped(other)),
        }
    }

    /// **SPIKE, not for commit.** Sets a repeating loop up to be driven from
    /// the op driver's own frame instead of a nested `run_ops` entry, or
    /// declines and leaves the caller to take the nested path.
    #[allow(clippy::too_many_arguments, reason = "spike")]
    pub(crate) fn flat_loop_start(
        &mut self,
        code: &Code<'_>,
        index: usize,
        instruction: &Instruction,
        body: &Loop,
        source: Option<&ProgramSource>,
        values: LoopHeaderValues,
        op_body: u32,
        registers: FrameId,
    ) -> Result<FlatStart, Failure> {
        // SPIKE: the switch is a run-time one so that both arms are the same
        // binary -- the per-op checks this spike adds to the driver's loop are
        // compiled in either way, so an arm with the flat path never taken
        // prices those checks on their own.
        static FLAT: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        // **`loop_header_plan` is the whole refusal**, exactly as it is for
        // `run_loop_with_header`: it answers `None` for a `COUNTER`, a stem
        // `OVER` and `DO WITH`, which this crate does not run on either
        // engine, and the nested path is where that becomes the loud error.
        if *FLAT.get_or_init(|| std::env::var_os("REXX_NO_FLAT").is_some())
            || loop_header_plan(body).is_none()
        {
            return Ok(FlatStart::Fallback(values));
        }
        let over_register = values.over_register;
        let state = match &body.kind {
            LoopKind::Forever => LoopState::Forever,
            LoopKind::Count(_) => LoopState::Count {
                remaining: values.count.unwrap_or(1),
            },
            LoopKind::Controlled(ctrl) => LoopState::Controlled {
                control: ctrl.control,
                at: control_slot(code, ctrl.control),
                current: values
                    .initial
                    .expect("a controlled loop's plan always names its initial value"),
                to: values.to,
                by: match values.by {
                    Some(by) => by,
                    None => Number::one(),
                },
                for_remaining: values.for_remaining,
                cached_digits: u64::MAX,
                to_int: None,
                by_int: None,
                shape: shape_of(code.symbols.name(ctrl.control).as_bytes()),
                stepped: false,
            },
            LoopKind::Over { control, .. } => {
                let (snapshot, items) = self.over_snapshot(
                    values
                        .over
                        .expect("a DO OVER's plan always names its target"),
                )?;
                LoopState::OverItems {
                    control: *control,
                    at: control_slot(code, *control),
                    snapshot,
                    items,
                    next: 0,
                    remaining: values.for_remaining,
                }
            }
            // A block, not a loop: one pass, its own trace shape, and
            // `run_loop_with_header`'s own arm resolves the whole of it
            // without ever reaching a pass boundary. `DO WITH` is refused
            // above and cannot arrive here.
            LoopKind::Simple | LoopKind::With { .. } => {
                return Ok(FlatStart::Fallback(values));
            }
        };
        // **The snapshot's only root once this answers `Flat`**: the driver
        // then closes this clause and pops the temps frame `over_snapshot`
        // pushed into, while the loop runs on. A loop header's registers are
        // allocated in the enclosing scope and released past the whole loop
        // (`ir/compile.rs`), and nothing reads the target again, so its
        // register holds the snapshot instead.
        if let LoopState::OverItems { snapshot, .. } = &state
            && let Some(register) = over_register
        {
            self.roots.set_temp(registers, register as usize, *snapshot);
        }
        let end_index = body
            .end
            .expect("an unclosed DO/LOOP is error 14.1/14.5, so a body that parsed has this set");
        let do_indent = self.clause_state.current_value_indent;
        let end_line = self
            .clause_line_at(code, end_index, &code.body.instructions[end_index], source)
            .unwrap_or(0);
        let do_line = self
            .clause_line_at(code, index, instruction, source)
            .unwrap_or_else(|| self.clause_state.line());
        // **Built where it will live, not on the stack and then moved
        // there.** A `FlatLoop` is 328 bytes, 200 of them the `LoopState` a
        // controlled loop's three `Number`s live in, and the version that
        // built one here and assigned it into the box afterwards wrote those
        // bytes twice per loop entry -- which for a nested loop is twice per
        // iteration of the loop above it. Measured on `do n = 1 to N ; do j =
        // 1 to 1 ; end ; end`: -5.871% retired instructions, with
        // `__memmove_avx_unaligned_erms` falling from 8.41% of the program's
        // cycles to 2.52%.
        let mut boxed = match self.flat_spares.pop() {
            Some(spare) => spare,
            None => Box::new(FlatLoop::vacant()),
        };
        *boxed = FlatLoop {
            op_body,
            body_start: index + 1,
            end_index,
            do_index: index,
            resume: end_index + 1,
            label: body.label,
            do_indent,
            loop_indent: do_indent + 2,
            end_line,
            do_line,
            header_clause: HeaderClause::Do,
            iterate_site: None,
            conditional: body.conditional.as_ref().map(|cond| cond.until),
            state,
        };
        let header = match self.flat_loop_header(code, source, &mut boxed) {
            Ok(header) => header,
            Err(failure) => {
                self.flat_spares.push(boxed);
                return Err(failure);
            }
        };
        match header {
            Some(flow) => {
                // The header ended the loop, so nothing will drive it and
                // this box is spare again rather than leaked back to the
                // allocator.
                self.flat_spares.push(boxed);
                Ok(FlatStart::Ended(flow))
            }
            None => {
                let range = (boxed.body_start, boxed.end_index);
                // The loop just entered becomes the innermost, and whatever
                // was innermost joins the ones enclosing it.
                if let Some(enclosing) = self.flat_top.replace(boxed) {
                    self.flat_loops.push(enclosing);
                }
                Ok(FlatStart::Flat {
                    body_start: range.0,
                    end_index: range.1,
                })
            }
        }
    }

    /// **SPIKE.** One pass boundary of the innermost flat loop: the state is
    /// taken out of `Interp::flat_top` so that this can hold a `&mut Interp`
    /// beside it, and put back when another pass follows.
    pub(crate) fn flat_loop_step_top(
        &mut self,
        code: &Code<'_>,
        source: Option<&ProgramSource>,
        arrival: Flow,
    ) -> Result<FlatStep, Failure> {
        let Some(mut top) = self.flat_top.take() else {
            return Err(Loud::op_not_driven("a pass boundary with no loop open").into());
        };
        match self.flat_loop_step(code, source, &mut top, arrival) {
            Ok(FlatStep::Body(op_body)) => {
                self.flat_top = Some(top);
                Ok(FlatStep::Body(op_body))
            }
            Ok(FlatStep::Done(flow)) => {
                self.flat_spares.push(top);
                // The loop that just ended uncovers the one enclosing it, and
                // `None` here is the outermost of a nest having ended.
                self.flat_top = self.flat_loops.pop();
                Ok(FlatStep::Done(flow))
            }
            // `top` is dropped rather than handed back, and this loop's own
            // frame is still standing: `unwind_frames` finds `flat_top`
            // already empty and takes the enclosing box for that frame, so one
            // box reaches the allocator instead of `flat_spares` and no state
            // is lost. Its own `if let Some` is what tolerates the last frame
            // finding nothing left, which is the same tolerance the pop off
            // `flat_loops` needed before this field existed.
            Err(failure) => Err(failure),
        }
    }

    /// **SPIKE.** One pass boundary: what the body just answered, then the
    /// next pass's header test.
    fn flat_loop_step(
        &mut self,
        code: &Code<'_>,
        source: Option<&ProgramSource>,
        flat: &mut FlatLoop,
        arrival: Flow,
    ) -> Result<FlatStep, Failure> {
        // **Read once per pass boundary and used for both echoes.** The two
        // events are one boundary and nothing between them can run a `TRACE`,
        // so the second read could only ever answer what the first did.
        let echoing = self.trace_mode().all;
        // **A pass that fell out of its body needs nothing decided.**
        // `do_body_outcome`'s own `Flow::Next` arm answers `FellThrough` and
        // reads none of its other arguments, and that is the arrival of every
        // pass that did not end in a `LEAVE`, an `ITERATE` or an escape -- so
        // asking is a call per pass for an answer the discriminant already
        // gives.
        let outcome = if matches!(arrival, Flow::Next) {
            DoOutcome::FellThrough
        } else {
            self.do_body_outcome(code, flat.do_index, flat.label, true, flat.resume, arrival)?
        };
        match outcome {
            DoOutcome::Escaped(escape) => return Ok(FlatStep::Done(escape)),
            DoOutcome::Iterated { line, site } => {
                flat.header_clause = HeaderClause::Iterate { line };
                flat.iterate_site = site;
            }
            DoOutcome::FellThrough => {
                flat.header_clause = HeaderClause::End;
                // `END` echoes for a pass that fell through to it and for no
                // other, exactly as `run_repeating`'s own arm does.
                if echoing
                    && let Some((line, text)) =
                        self.clause_site(source, &code.body.instructions[flat.end_index])
                {
                    self.trace_clause(line, flat.do_indent, &text);
                }
            }
        }
        // **`UNTIL`'s own test, and its own re-echo of the `DO`/`LOOP`
        // clause.** `run_repeating`'s own arm has the measurement: the oracle
        // re-enters the loop instruction to make this decision as much as to
        // test `WHILE` or advance a control variable, so the echo here is
        // unconditional rather than sharing the top-of-loop one below -- which
        // is why that one is not emitted at all for an `UNTIL` loop.
        if flat.conditional == Some(true) {
            if echoing
                && let Some((line, text)) =
                    self.clause_site(source, &code.body.instructions[flat.do_index])
            {
                self.trace_clause(line, flat.do_indent, &text);
            }
            if let Some(flow) = self.flat_loop_until(code, source, flat)? {
                return Ok(FlatStep::Done(flow));
            }
        } else if echoing
            // **The re-echo of the `DO`/`LOOP` clause itself, once per pass
            // after the first**, and it is asked here rather than at the
            // loop's entry because a `TRACE` in the body changes the answer:
            // measured, an `ir_recorded` case that switches tracing on inside the
            // body loses this line and `END`'s when the decision is made once
            // on the way in.
            && let Some((line, text)) =
                self.clause_site(source, &code.body.instructions[flat.do_index])
        {
            self.trace_clause(line, flat.do_indent, &text);
        }
        match self.flat_loop_header(code, source, flat)? {
            Some(flow) => Ok(FlatStep::Done(flow)),
            None => Ok(FlatStep::Body(flat.op_body)),
        }
    }

    /// **SPIKE.** An `UNTIL` loop's own bottom-of-pass test: `Some(flow)` is
    /// the loop finishing, `None` is carrying on to the header.
    fn flat_loop_until(
        &mut self,
        code: &Code<'_>,
        source: Option<&ProgramSource>,
        flat: &FlatLoop,
    ) -> Result<Option<Flow>, Failure> {
        let Some(cond) = loop_conditional_of(code, flat.do_index) else {
            return Err(Loud::instruction(&code.body.instructions[flat.do_index].kind).into());
        };
        let until_line = flat.header_line();
        let (do_indent, loop_indent, resume) = (flat.do_indent, flat.loop_indent, flat.resume);
        // The re-echoed `END` clause just above leaves `current_value_indent`
        // untouched -- `trace_clause` does not set it -- so without this the
        // `UNTIL`'s intermediates would read the `DO`'s own indent.
        self.clause_state.current_value_indent = loop_indent;
        let tested = self.in_clause(code, until_line, |it| {
            let held = it.eval_condition(
                code,
                &cond.condition,
                ConditionTrace::Keyword(loop_indent, "UNTIL"),
                raised_until_not_logical,
            )?;
            // An `UNTIL` that held ends the loop, so this clause's own
            // boundary sits outside the block.
            it.settle_block_indent(!held, do_indent);
            Ok(held)
        })?;
        match tested {
            ClauseOutcome::Ended(exit) => Ok(Some(Flow::Exit(exit.value()))),
            ClauseOutcome::Ran(Ok(true)) => Ok(Some(Flow::Goto(resume))),
            ClauseOutcome::Ran(Ok(false)) => Ok(None),
            ClauseOutcome::Ran(Err(failure)) => {
                let end = &code.body.instructions[flat.end_index];
                self.record_failure_at(source, end, loop_indent);
                Err(failure)
            }
        }
    }

    /// **SPIKE.** Who a failing header re-test is blamed on: the clause that
    /// transferred control back to the loop, at the body's indent.
    #[cold]
    #[inline(never)]
    fn blame_header_failure(
        &mut self,
        code: &Code<'_>,
        source: Option<&ProgramSource>,
        blame: HeaderClause,
        site: &IterateSite,
        end_index: usize,
        loop_indent: usize,
    ) {
        match blame {
            HeaderClause::Do => {}
            HeaderClause::End => {
                self.record_failure_at(source, &code.body.instructions[end_index], loop_indent);
            }
            HeaderClause::Iterate { .. } => {
                self.record_failure_site_at(site.clone(), loop_indent);
            }
        }
    }

    /// **SPIKE.** A `WHILE` that failed, blamed on the `DO`/`LOOP` clause at
    /// the body's indent. `#[cold]` for the reason above.
    #[cold]
    #[inline(never)]
    fn blame_while_failure(
        &mut self,
        code: &Code<'_>,
        source: Option<&ProgramSource>,
        do_index: usize,
        loop_indent: usize,
    ) {
        self.record_failure_at(source, &code.body.instructions[do_index], loop_indent);
    }

    /// **SPIKE.** The header re-test, in its own clause: `Some(flow)` is the
    /// loop finishing, `None` is one more pass.
    #[inline(always)]
    fn flat_loop_header(
        &mut self,
        code: &Code<'_>,
        source: Option<&ProgramSource>,
        flat: &mut FlatLoop,
    ) -> Result<Option<Flow>, Failure> {
        if flat.conditional == Some(false) {
            return self.flat_loop_header_while(code, source, flat);
        }
        let header_line = flat.header_line();
        let do_indent = flat.do_indent;
        let loop_indent = flat.loop_indent;
        let resume = flat.resume;
        let end_index = flat.end_index;
        let blame = flat.header_clause;
        let site = &flat.iterate_site;
        let state = &mut flat.state;
        let header = self.in_clause(code, header_line, |it| {
            let advanced = match it.loop_advance(code, state, do_indent, loop_indent) {
                Ok(advanced) => advanced,
                Err(failure) => {
                    it.blame_header_failure(code, source, blame, site, end_index, loop_indent);
                    return Err(failure);
                }
            };
            it.settle_block_indent(advanced, do_indent);
            Ok(advanced)
        })?;
        flat_header_outcome(header, resume)
    }

    /// **SPIKE.** [`Interp::flat_loop_header`] for a loop that carries a
    /// `WHILE`: the same advance, then the condition, both inside the one
    /// clause the oracle re-enters to make this decision.
    #[inline(never)]
    fn flat_loop_header_while(
        &mut self,
        code: &Code<'_>,
        source: Option<&ProgramSource>,
        flat: &mut FlatLoop,
    ) -> Result<Option<Flow>, Failure> {
        let Some(cond) = loop_conditional_of(code, flat.do_index) else {
            return Err(Loud::instruction(&code.body.instructions[flat.do_index].kind).into());
        };
        let header_line = flat.header_line();
        let do_indent = flat.do_indent;
        let loop_indent = flat.loop_indent;
        let resume = flat.resume;
        let (do_index, end_index) = (flat.do_index, flat.end_index);
        let blame = flat.header_clause;
        let site = &flat.iterate_site;
        let state = &mut flat.state;
        let header = self.in_clause(code, header_line, |it| {
            let advanced = match it.loop_advance(code, state, do_indent, loop_indent) {
                Ok(advanced) => advanced,
                Err(failure) => {
                    it.blame_header_failure(code, source, blame, site, end_index, loop_indent);
                    return Err(failure);
                }
            };
            if !advanced {
                it.settle_block_indent(false, do_indent);
                return Ok(false);
            }
            // Overrides what stepping the `DO`/`LOOP` instruction set:
            // `WHILE`'s condition is evaluated here, inside that same step,
            // never through a `Op::Clause`'s region of its own.
            it.clause_state.current_value_indent = loop_indent;
            let held = match it.eval_condition(
                code,
                &cond.condition,
                ConditionTrace::Keyword(loop_indent, "WHILE"),
                raised_while_not_logical,
            ) {
                Ok(held) => held,
                Err(failure) => {
                    it.blame_while_failure(code, source, do_index, loop_indent);
                    return Err(failure);
                }
            };
            it.settle_block_indent(held, do_indent);
            Ok(held)
        })?;
        flat_header_outcome(header, resume)
    }

    /// Decides whether one more candidate iteration of `state` should run,
    /// consuming whatever budget (`FOR`, a bare count) applies and binding
    /// a control variable **before** the decision is answered, not after --
    /// measured, `do i = 5 to 3 / say never / end / say i` prints `5`: the
    /// control variable is bound to its own value even for a loop that ends
    /// up running zero iterations.
    fn loop_advance(
        &mut self,
        code: &Code<'_>,
        state: &mut LoopState,
        do_indent: usize,
        loop_indent: usize,
    ) -> Result<bool, Failure> {
        match state {
            LoopState::Forever => Ok(true),
            LoopState::Count { remaining } => {
                if *remaining == 0 {
                    return Ok(false);
                }
                *remaining -= 1;
                Ok(true)
            }
            LoopState::OverItems {
                control,
                at,
                snapshot: _,
                items,
                next,
                remaining,
            } => {
                // **The item is bound before `FOR`'s budget is consulted**,
                // which is `RexxInstructionDoOverFor::iterate`'s own
                // `doblock->checkOver(context, stack) && doblock->checkFor()`
                // (`instructions/DoOverInstruction.cpp:279`): `checkOver`
                // assigns the control variable, and the `&&` reaches
                // `checkFor` afterwards. Measured, three descriptors: `a =
                // (10,20,30,40)` with `do e over a for 2` leaves `e` at `30`,
                // and `do e over 'abc' for 0` leaves it at `abc` with the body
                // never entered. Under `trace i` the terminating pass still
                // shows its own `>=>` line.
                let Some(value) = items.get(*next).copied() else {
                    return Ok(false);
                };
                *next += 1;
                self.bind_control(
                    code,
                    *control,
                    loop_indent,
                    value,
                    *at,
                    shape_of(code.symbols.name(*control).as_bytes()),
                )?;
                if let Some(r) = remaining {
                    if *r == 0 {
                        return Ok(false);
                    }
                    *r -= 1;
                }
                Ok(true)
            }
            LoopState::Controlled {
                control,
                at,
                current,
                to,
                by,
                for_remaining,
                cached_digits,
                to_int,
                by_int,
                shape,
                stepped,
            } => {
                // **The re-tested pass's own four lines** (Task 9, closing
                // the KNOWN GAP this arm used to disclose).
                // `DoBlock::checkControl` (`DoBlock.cpp:182`-`205`, read
                // directly) is called as `checkControl(context, stack,
                // !first)` (`ControlledDoInstruction.cpp:162`), and its
                // `increment` arm does four traceable things in this order:
                // read the control variable (`control->evaluate`, which
                // traces `>V>`), `traceResult` that value, add `BY`,
                // `traceResult` the sum, then `control->assign` (`>=>`).
                // The `!first` is exactly `stepped` here. `trace i` /
                // `do ii = 1 to 2`'s own second pass:
                // ```text
                //   >V>     II => "1"
                //   >>>     "1"
                //   >>>     "2"
                //   >=>     II <= "2"
                // ```
                let digits = self.activation().settings.digits();
                // **The ordinary counted pass, answered without reaching any
                // of the code below.** Every branch of the arm from here on
                // exists for a shape this test excludes: a control variable
                // that is not a plain name, a value or a bound too wide for
                // the tag, a `FOR` budget, a `TRACE` that has to render the
                // value, or the first pass, whose value the header already
                // computed. What is left is read the slot, add, write the
                // slot, compare -- and, because both handles are tagged
                // integers rather than heap objects, no temporaries frame
                // for the collector to walk.
                if *stepped
                    && *shape == NameShape::Simple
                    && *cached_digits == digits
                    && for_remaining.is_none()
                    && let Some(slot) = *at
                    && let Some(step) = *by_int
                    && let Some(bound) = *to_int
                    && let ControlValue::Small(_) = current
                {
                    let mode = self.trace_mode();
                    // `FUZZ` sits here rather than in the conjunction above so
                    // a pass that fails an earlier test never reads it: the
                    // arm below is the only one that answers the bound test
                    // without consulting it, and the general path reads it once
                    // on its own account.
                    if !mode.results
                        && !mode.intermediates
                        && self.activation().settings.fuzz() == 0
                    {
                        let frame = self.activation().frame;
                        // The read is `Interp::variable` rather than
                        // `read_at`: the shape is `Simple` and the slot is
                        // the loop's own, so the two agree, and an
                        // uninitialised slot answers `None` here instead of
                        // building a derived name -- which is one of the
                        // shapes this path declines, since it goes on to
                        // fail 41.1 below.
                        if let Some(previous) = self.variable(frame, slot)
                            && let Decoded::SmallInt(value) = previous.decode()
                            && within_digits(value, digits)
                            && let Some(sum) = value.checked_add(step)
                            && within_digits(sum, digits)
                            && let Some(handle) = exact_small_int(sum, digits)
                        {
                            debug_assert_eq!(
                                *shape,
                                shape_of(code.symbols.name(*control).as_bytes()),
                                "the loop's cached control-variable shape is not the one its name gives"
                            );
                            debug_assert_eq!(
                                *to_int,
                                to.as_ref().and_then(|bound| bound.plain_integer(digits)),
                                "the loop's cached TO disagrees with DIGITS {digits}"
                            );
                            debug_assert_eq!(
                                *by_int,
                                by.plain_integer(digits),
                                "the loop's cached BY disagrees with DIGITS {digits}"
                            );
                            *current = ControlValue::Small(sum);
                            self.set_variable(frame, slot, handle);
                            return Ok(if step < 0 { sum >= bound } else { sum <= bound });
                        }
                    }
                }
                // **The same shortcut for the control values the arm above
                // cannot take.** `Number::plain_integer` opens by rejecting a
                // negative exponent, so a bound written `1.0` is refused as
                // flatly as `1.1`: measured, `do j=1.0 to 3.0 by 1.0` and `do
                // j=1.1 to 3.3 by 1.1` differ by 594 instructions in 3.6
                // billion, because both reach the general path below. What
                // that path costs over this one is its own bookkeeping -- a GC
                // frame per pass, the `NOVALUE` check, two temps, the trace
                // renderings and `bind_control`'s dispatch on shape -- none of
                // which a simple untraced control variable needs.
                if *stepped
                    && *shape == NameShape::Simple
                    && *cached_digits == digits
                    && for_remaining.is_none()
                    && let Some(slot) = *at
                    && let Some(bound) = to.as_ref()
                {
                    let mode = self.trace_mode();
                    let fuzz = self.activation().settings.fuzz();
                    if !mode.results && !mode.intermediates && fuzz == 0 {
                        let frame = self.activation().frame;
                        if let Some(previous) = self.variable(frame, slot)
                            && matches!(previous.decode(), Decoded::Heap { .. })
                        {
                            let sum = self.controlled_step_wide(previous, by, digits)?;
                            let stepped_value = ControlValue::Wide(sum);
                            let within = Self::controlled_within_wide(
                                &stepped_value,
                                bound,
                                by,
                                digits,
                                fuzz,
                            )?;
                            let form = self.activation().settings.form();
                            let handle = self.controlled_value_wide(&stepped_value, digits, form);
                            self.set_variable(frame, slot, handle);
                            *current = stepped_value;
                            return Ok(within);
                        }
                    }
                }
                let fuzz = self.activation().settings.fuzz();
                let form = self.activation().settings.form();
                if *cached_digits != digits {
                    *cached_digits = digits;
                    *to_int = to.as_ref().and_then(|bound| bound.plain_integer(digits));
                    *by_int = by.plain_integer(digits);
                }
                // The cache is only ever as good as its invalidation, so the
                // debug gate re-derives both on every pass and compares. This
                // is the tripwire, not the test: a stale entry is invisible in
                // the answer for every program tried against the oracle, which
                // is exactly why it needs an assertion rather than a probe.
                debug_assert_eq!(
                    *to_int,
                    to.as_ref().and_then(|bound| bound.plain_integer(digits)),
                    "the loop's cached TO disagrees with DIGITS {digits}"
                );
                debug_assert_eq!(
                    *by_int,
                    by.plain_integer(digits),
                    "the loop's cached BY disagrees with DIGITS {digits}"
                );
                let (to_int, by_int) = (*to_int, *by_int);
                // The same tripwire for the control variable's shape, which
                // needs no invalidation at all: it is a property of a name
                // the parse fixed, so nothing can move it. The assertion is
                // what says so on every pass of every program the debug gate
                // runs, rather than a comment claiming it.
                debug_assert_eq!(
                    *shape,
                    shape_of(code.symbols.name(*control).as_bytes()),
                    "the loop's cached control-variable shape is not the one its name gives"
                );
                let shape = *shape;
                let re_tested = std::mem::replace(stepped, true);
                // **One pass's own temps frame, released before the next pass
                // opens one.** The enclosing `Op::Clause`'s region belongs to
                // the whole `DO` instruction, so without this every root
                // pushed below survives until the *loop* ends rather than
                // until the *pass* does -- one `ObjRef` per iteration, and
                // each one pins whatever heap object it names. Popped after
                // `bind_control` has written the new value into the control
                // variable's own storage, which is what roots it from there
                // on; the `?` paths below leave it to the outer truncation,
                // exactly as `pop_frame`'s own doc describes.
                let pass = self.roots.push_frame();
                if re_tested {
                    // **`read`, not `read_by_name`: this is an evaluation and
                    // it can raise `NOVALUE`** (review round 1 re-review,
                    // NEW-1 -- a defect this arm shipped with, not a
                    // pre-existing one). The oracle's own `control->evaluate`
                    // is a full expression evaluation, so a body that
                    // `DROP`s the control variable makes the next re-test
                    // raise `NOVALUE` rather than read a derived name.
                    // Measured, `signal on novalue name nv` around
                    // `do ii = 1 to 3 ; drop ii ; end`: the oracle runs the
                    // handler and exits 0, where `read_by_name` here gave a
                    // spurious 41.1 at rc 215. `read_by_name` reports
                    // nothing to its caller and cannot express that.
                    let (previous, novalue, resolved) = match shape {
                        NameShape::Simple => {
                            // `read_at` with the slot `control_slot` took when
                            // the loop was entered, which is the same
                            // resolution this read made for itself on every
                            // pass before -- `None` still makes it, so the two
                            // shapes below and a control this resolution does
                            // not reach are unaffected.
                            let (value, novalue) = self.read_at(code, *control, *at);
                            (value, novalue, None)
                        }
                        // A bare stem never raises `NOVALUE` on read (`eval_
                        // node`'s own `ExprKind::Stem` arm has the citation),
                        // so there is no fallible read to thread through.
                        // The slot comes off the entry rather than from the
                        // loop's kept `at`, which is the same source the write
                        // half of this pass uses and the reason `control_slot`
                        // declines a stem.
                        NameShape::Stem => {
                            let at = code.compound(*control).and_then(|entry| entry.stem_at);
                            let name = code.symbols.name(*control).as_bytes();
                            (self.read_stem_at(name, at), Novalue::Set, None)
                        }
                        NameShape::Compound => {
                            let (stem_name, stem_at) = code.stem(*control);
                            let key = self.tail_key(code, *control)?;
                            let (value, novalue) = self.stem_get_at(stem_name, stem_at, &key);
                            let mut resolved = stem_name.to_vec();
                            resolved.extend_from_slice(&key);
                            (value, novalue, Some(resolved))
                        }
                    };
                    self.novalue_check(novalue, previous)?;
                    self.roots.push_temp(previous);
                    // `>C>` before `>V>`, both self-gated on `intermediates`
                    // like every other value-bearing prefix -- `stem_get`'s
                    // own read announces the fully-resolved name it used
                    // before either of the value lines shows what is stored
                    // there, the same order `eval_node`'s `Compound` arm and
                    // its own tracing counterpart use for an ordinary read.
                    if let Some(resolved) = &resolved {
                        let name = code.symbols.name(*control).as_bytes();
                        self.trace_compound_name(loop_indent, name, resolved);
                    }
                    // `result_text` for the pair, not `intermediate_text`:
                    // `>V>` is `intermediates` and `>>>` is `results`, and
                    // `results` is the weaker of the two, so it renders for
                    // either and drops neither.
                    if let Some(rendered) = self.result_text(previous) {
                        let name = code.symbols.name(*control).as_bytes();
                        self.trace_variable(loop_indent, name, &rendered);
                        self.trace_result(loop_indent, &rendered);
                    }
                    // The increment, on integers when it can be. `previous`
                    // comes back out of the variable pool as a tagged small
                    // integer for every ordinary counted loop, and `BY` is
                    // whole; the guard is the same one `eval`'s own
                    // arithmetic fast path uses, and for the same reason --
                    // an operand too wide for `DIGITS` is rounded before the
                    // addition, so the exact `i64` sum would be the wrong
                    // answer.
                    let stepped = match (&*current, previous.decode()) {
                        (ControlValue::Small(_), Decoded::SmallInt(value))
                            if within_digits(value, digits) =>
                        {
                            by_int
                                .and_then(|step| value.checked_add(step))
                                .filter(|sum| within_digits(*sum, digits))
                        }
                        _ => None,
                    };
                    // **Written inside each arm rather than assigned from the
                    // `match`'s own value**, which is layout rather than
                    // style. A single assignment site has to write a whole
                    // `ControlValue`, and that is as wide as the `Number` its
                    // other arm carries, so the integer arm pays that width
                    // to deliver a tag and an `i64`; per arm, each writes
                    // only the bytes it has. Measured, 10 instructions a pass
                    // -- 190,000,000 across `bench-programs/varlookup.rex`.
                    match stepped {
                        Some(sum) => *current = ControlValue::Small(sum),
                        None => {
                            *current =
                                ControlValue::Wide(self.controlled_step_wide(previous, by, digits)?)
                        }
                    }
                }
                // The first pass takes the value the header already computed,
                // unincremented and with no line of its own beyond the `>=>`
                // below -- `checkControl`'s own `else` arm reads it with
                // `getValue`, whose comment says why: the initial assignment
                // was already traced during setup, and tracing here too
                // "prevents getting an extra add looking item traced".
                let value = if let ControlValue::Small(small) = current
                    && let Some(handle) = exact_small_int(*small, digits)
                {
                    handle
                } else {
                    self.controlled_value_wide(current, digits, form)
                };
                // Rooted before anything else can allocate: the render below
                // builds a `Vec`, and `bind_control`'s compound arm resolves a
                // tail key, which allocates in the arena. Nothing collected
                // between this allocation and the write before the trigger
                // existed, so this push closes a window that was inert rather
                // than absent.
                self.roots.push_temp(value);
                let bind_indent = if re_tested { loop_indent } else { do_indent };
                if re_tested && let Some(rendered) = self.result_text(value) {
                    self.trace_result(loop_indent, &rendered);
                }
                self.bind_control(code, *control, bind_indent, value, *at, shape)?;
                // The control variable's own storage now roots `value`, and
                // `previous` is dead, so the pass's frame goes here. The three
                // `return Ok(false)` paths below end the loop, whose enclosing
                // frame truncates past this one anyway.
                self.roots.pop_frame(pass);

                if let Some(r) = for_remaining
                    && *r == 0
                {
                    return Ok(false);
                }
                if let Some(to) = to {
                    // The bound test, on integers when it can be.
                    // `numeric_less` reaches it through a subtraction, which
                    // allocates twice per pass for what an `i64` comparison
                    // answers outright.
                    let within = match (current.small(digits), to_int, by_int) {
                        (Some(current), Some(to), Some(by)) if fuzz == 0 => {
                            if by < 0 {
                                current >= to
                            } else {
                                current <= to
                            }
                        }
                        _ => Self::controlled_within_wide(current, to, by, digits, fuzz)?,
                    };
                    if !within {
                        return Ok(false);
                    }
                }
                if let Some(r) = for_remaining {
                    *r -= 1;
                }
                Ok(true)
            }
        }
    }

    /// One controlled pass's step where the control variable is not an
    /// integer the tag holds, or the sum leaves what `DIGITS` admits.
    #[inline(never)]
    fn controlled_step_wide(
        &mut self,
        previous: ObjRef,
        by: &Number,
        digits: u64,
    ) -> Result<Number, Failure> {
        // The control variable is the **left** operand of the oracle's own
        // implicit `+`, so an object assigned to it inside the body is 97.1
        // there -- measured, `do i = 1 to 3; i = .array; end` prints one
        // iteration and then raises.
        if let Some(kind) = self.operator_operand_gap(previous) {
            return Err(Loud::object_position("a controlled DO's control variable", kind).into());
        }
        let read = self.arith_operand(previous)?;
        read.add(by, digits)
            .map_err(Raised::from)
            .map_err(Failure::from)
    }

    /// The handle a controlled pass binds when [`exact_small_int`] declines
    /// the control value. Out of line for the reason
    /// [`Interp::controlled_step_wide`] gives.
    #[inline(never)]
    fn controlled_value_wide(
        &mut self,
        current: &ControlValue,
        digits: u64,
        form: rexx_num::Form,
    ) -> ObjRef {
        let number = current.number().into_owned();
        self.number(number, crate::eval::saturate_digits(digits), form)
    }

    /// A controlled loop's `TO` test where either side is wider than an
    /// `i64` comparison answers. Out of line for the reason
    /// [`Interp::controlled_step_wide`] gives.
    #[inline(never)]
    fn controlled_within_wide(
        current: &ControlValue,
        to: &Number,
        by: &Number,
        digits: u64,
        fuzz: u64,
    ) -> Result<bool, Failure> {
        let current = current.number();
        let within = if by.signum() < 0 {
            !numeric_less(&current, to, digits, fuzz).map_err(Raised::from)?
        } else {
            !numeric_less(to, &current, digits, fuzz).map_err(Raised::from)?
        };
        Ok(within)
    }

    /// Writes `value` into `control`'s own variable, through whichever of
    /// the three shapes (`shape_of`) its own spelling is.
    fn bind_control(
        &mut self,
        code: &Code<'_>,
        control: SymbolId,
        indent: usize,
        value: ObjRef,
        at: Option<usize>,
        shape: NameShape,
    ) -> Result<(), Failure> {
        debug_assert_eq!(
            shape,
            shape_of(code.symbols.name(control).as_bytes()),
            "a control variable's write was told a shape its name does not give"
        );
        match shape {
            NameShape::Simple => {
                // The tripwire `crate::ir::Op::Load` and `Op::Store` each carry,
                // on the one write that keeps its slot across passes rather
                // than reading it out of an op: a kept index that is not the
                // one this body's plan gives the name would write into another
                // variable's slot rather than fail.
                debug_assert!(
                    at.is_none() || control_slot(code, control) == at,
                    "a loop's kept control slot is not the one this body's plan gives its name"
                );
                let slot = match at {
                    Some(slot) => slot,
                    // The name is looked up here rather than above the
                    // `match`, so that a loop whose slot was resolved when it
                    // was entered -- every counted loop -- never asks for its
                    // own name again on a pass.
                    None => self.slot_of(code.symbols.name(control).as_bytes()),
                };
                let frame = self.activation().frame;
                self.set_variable(frame, slot, value);
                // `trace_assignment` carries its own `intermediates` gate, so
                // the check below is not a second decision about whether to
                // *print*: it decides whether to *build* the two `Vec`s, and
                // it exists because this function runs once per loop pass.
                if self.tracing_intermediates() {
                    let name = code.symbols.name(control).as_bytes().to_vec();
                    let rendered = self.string_value_text(value);
                    self.trace_assignment(indent, &name, &rendered);
                }
                Ok(())
            }
            NameShape::Stem => {
                let target = Expr {
                    kind: ExprKind::Stem(control),
                    span: 0..0,
                };
                let rendered = self.intermediate_text(value);
                // `None`, and a literal one rather than the loop's kept
                // slot: `assign_expr_target`'s `Stem` arm finds this stem's
                // slot on its own entry, and a value here in place of the
                // constant costs every controlled loop 2 instructions a pass
                // (`control_slot`'s doc has the measurement).
                self.assign_expr_target(code, &target, value, rendered.as_deref(), indent, None)
            }
            NameShape::Compound => {
                let target = Expr {
                    kind: ExprKind::Compound(control),
                    span: 0..0,
                };
                let rendered = self.intermediate_text(value);
                // `None` because a compound writes one tail through a key
                // resolved on this pass, so the symbol's own slot is not what
                // is written and there is none to carry.
                self.assign_expr_target(code, &target, value, rendered.as_deref(), indent, None)
            }
        }
    }

    /// Validates `value` as "zero or a positive whole number" -- the rule a
    /// bare `DO`'s own repeat count and a `FOR` expression share (26.2/26.3
    /// respectively; the caller supplies which raiser applies, since that
    /// is the only way the two differ), and answers it as a `u64`, or
    /// `None` if it fails either check.
    pub(crate) fn whole_nonneg(&mut self, value: ObjRef) -> Option<u64> {
        let digits = usize::try_from(self.activation().settings.digits()).ok()?;
        // **A tagged integer already is the answer**, when it is small enough
        // that `whole_value` would round nothing -- which is what `whole_i64`
        // decides, and `builtin::whole_number` takes the same shortcut against
        // its own fixed width. Going the long way builds a `Number` out of the
        // tag, digit by digit, and takes the `i64` straight back out of it. A
        // `None` here means only that the rounding rule has to run, so the
        // general path below still does.
        if let Decoded::SmallInt(small) = value.decode()
            && let Some(whole) = rexx_num::whole_i64(small, digits)
        {
            return u64::try_from(whole).ok();
        }
        let number = self.to_number(value).ok()?;
        let whole = number.whole_value(digits)?;
        u64::try_from(whole).ok()
    }

    /// An `IF`'s own condition, evaluated as the whole of the `IF` clause's
    /// work.
    fn eval_if_condition(&mut self, code: &Code<'_>, condition: &Expr) -> Result<bool, Failure> {
        let indent = self.clause_state.current_value_indent;
        self.eval_condition(
            code,
            condition,
            ConditionTrace::Result(indent),
            raised_if_not_logical,
        )
    }

    /// The expression an op's address names: expression `slot` of
    /// `instruction`, then `path`'s steps down from that slot's root.
    pub(crate) fn chunk_node_at(
        instruction: &Instruction,
        slot: u16,
        path: NodePath,
    ) -> Option<&Expr> {
        let mut node = match (&instruction.kind, slot) {
            (InstructionKind::Assignment { value, .. }, 0) => value,
            // The one expression of an instruction that computes a value and
            // does something with it, slot `0`. A bare form holds no
            // expression at all, so it names no node and matches no arm here.
            (
                InstructionKind::Say {
                    expression: Some(expression),
                }
                | InstructionKind::Return {
                    expression: Some(expression),
                }
                | InstructionKind::Exit {
                    expression: Some(expression),
                }
                | InstructionKind::Push {
                    expression: Some(expression),
                }
                | InstructionKind::Queue {
                    expression: Some(expression),
                },
                0,
            ) => expression,
            // The two instructions whose slot `0` exists only in one of their
            // forms. Both are reachable: the expression is compiled through
            // `push_value` like any other, so a call inside it emits an
            // `Op::CallExpr` that addresses the node from here.
            (InstructionKind::Signal(signal), 0) => match &**signal {
                rexx_parse::Signal::Value(expression) => expression,
                rexx_parse::Signal::Label(_) | rexx_parse::Signal::Trap(_) => return None,
            },
            (
                InstructionKind::Parse(parse)
                | InstructionKind::Arg(parse)
                | InstructionKind::Pull(parse),
                0,
            ) => match &parse.source {
                rexx_parse::ParseSource::Value(Some(expression)) => expression,
                _ => return None,
            },
            (InstructionKind::If { condition, .. }, 0) => condition,
            // A **plain** `WHEN`'s condition. A `WhenCase`'s values are not a
            // condition and compile to no native op at all, so no address ever
            // names one and this arm does not answer for them.
            (InstructionKind::When { condition, .. }, 0) => condition,
            // A `DO`/`LOOP` header's `slot`th expression, resolved through the
            // same `loop_header_slot` the compiler emitted the slot's ops from
            // and the same one `eval_chunk_expr` evaluates a declining slot
            // with. Every slot of a header is addressable, not just one, which
            // is why this arm binds `slot` rather than matching a number.
            (InstructionKind::Do(body) | InstructionKind::Loop(body), slot) => {
                loop_header_slot(body, u32::from(slot))?
            }
            _ => return None,
        };
        for right in path.steps() {
            node = match (&node.kind, right) {
                (ExprKind::Binary { left, .. }, false) => left,
                (ExprKind::Binary { right, .. }, true) => right,
                (ExprKind::Prefix { operand, .. }, false) => operand,
                _ => return None,
            };
        }
        Some(node)
    }

    /// Expression `slot` of `instruction`, evaluated as the compiled stream's
    /// [`crate::ir::Op::EvalExpr`] asks.
    pub(crate) fn eval_chunk_expr(
        &mut self,
        code: &Code<'_>,
        instruction: &Instruction,
        slot: u32,
    ) -> Result<ObjRef, Failure> {
        match (&instruction.kind, slot) {
            (InstructionKind::If { condition, .. }, 0) => {
                let holds = self.eval_if_condition(code, condition)?;
                // In range unconditionally: `SMALL_INT_MAX` is far above one.
                Ok(ObjRef::small_int(i64::from(holds)).unwrap_or(ObjRef::NIL))
            }
            (
                InstructionKind::Select {
                    case: Some(case_expr),
                    ..
                },
                0,
            ) => self.select_case(code, case_expr),
            // A `DO`/`LOOP` header's `slot`th expression, in the order
            // `loop_header_plan` puts them in -- the order they were written,
            // which is the order they are evaluated. The expression is
            // evaluated and nothing else: its `>K>` echo and its validation are
            // ops of their own, because both have to happen before the next
            // expression is evaluated at all.
            (InstructionKind::Do(body) | InstructionKind::Loop(body), slot) => {
                let Some(expr) = loop_header_slot(body, slot) else {
                    return Err(Loud::instruction(&instruction.kind).into());
                };
                self.eval(code, expr)
            }
            // An `Assignment`'s value, and the one expression of a `SAY`, a
            // `RETURN`, an `EXIT`, a `PUSH` or a `QUEUE`, slot `0`: whatever
            // the expression came to, unvalidated and untagged. Each reaches
            // this arm only for an expression `compile` did not emit a native
            // op for -- a literal is `crate::ir::Op::Const` and a bare symbol
            // is `crate::ir::Op::Load` instead -- and each is trace-identical
            // to arm here because this is the same
            // `eval` call it makes.
            // `SIGNAL VALUE`'s own expression, for the shapes `compile` emits
            // no native op for. The `>K>` echo and the search are
            // `Op::Signal`'s, not this arm's, exactly as a loop header's
            // validation is its own op.
            (InstructionKind::Signal(signal), 0) => {
                let rexx_parse::Signal::Value(expr) = &**signal else {
                    return Err(Loud::instruction(&instruction.kind).into());
                };
                self.eval(code, expr)
            }
            // `PARSE VALUE expr WITH`'s own expression, on the same terms. The
            // `>K>` echo and the whole template walk are `Op::Parse`'s; only
            // the source is a register, and only this source has one --
            // `compile` emits no register for any other, so reaching here with
            // one is an op off its node.
            (
                InstructionKind::Parse(parse)
                | InstructionKind::Arg(parse)
                | InstructionKind::Pull(parse),
                0,
            ) => {
                let rexx_parse::ParseSource::Value(Some(expr)) = &parse.source else {
                    return Err(Loud::instruction(&instruction.kind).into());
                };
                self.eval(code, expr)
            }
            (InstructionKind::Assignment { value, .. }, 0) => self.eval(code, value),
            (
                InstructionKind::Say {
                    expression: Some(expression),
                }
                | InstructionKind::Return {
                    expression: Some(expression),
                }
                | InstructionKind::Exit {
                    expression: Some(expression),
                }
                | InstructionKind::Push {
                    expression: Some(expression),
                }
                | InstructionKind::Queue {
                    expression: Some(expression),
                },
                0,
            ) => self.eval(code, expression),
            (kind, _) => Err(Loud::instruction(kind).into()),
        }
    }

    /// Evaluates `condition` and answers whether it holds, for `IF`/`WHEN`.
    fn eval_condition(
        &mut self,
        code: &Code<'_>,
        condition: &Expr,
        trace: ConditionTrace<'_>,
        raise: fn(&[u8]) -> Raised,
    ) -> Result<bool, Failure> {
        let value = self.eval(code, condition)?;
        let checked = matches!(condition.kind, ExprKind::Logical(_));
        self.condition_value(value, trace, checked, raise)
    }

    /// Everything an evaluated condition value still owes: its `>>>` or `>K>`
    /// line, and the answer to whether it holds.
    pub(crate) fn condition_value(
        &mut self,
        value: ObjRef,
        trace: ConditionTrace<'_>,
        checked: bool,
        raise: fn(&[u8]) -> Raised,
    ) -> Result<bool, Failure> {
        // **The test's own temps frame, and it is a per-pass frame for the two
        // callers that are loop headers.** `WHILE` and `UNTIL` re-evaluate
        // their condition once per pass inside the enclosing `DO`
        // instruction's single frame, so the push below accumulates one
        // `ObjRef` per pass -- and one pinned heap object per pass whenever
        // the condition's result is one -- for the whole of the loop's run.
        // The value is never handed back (this answers a `bool`), so a frame
        // closed here releases it at the right time for every caller,
        // `IF`/`WHEN` included. The `?` path below leaves the pop to the outer
        // truncation, as `pop_frame`'s own doc describes.
        let frame = self.roots.push_frame();
        self.roots.push_temp(value);
        // **The owned copy is taken only when a line will print it.** Both
        // formatters below return at once unless `results` is on, and the copy
        // exists only because `to_text` borrows `self` while they need it
        // mutably -- so off that path the borrow is enough and the answer is
        // decided from it directly. Measured with `heaptrack` on
        // `samples/rexxcps.rex`: this was the largest single allocation site
        // in the interpreter, one copy per condition evaluated.
        let decided = if self.trace_mode().results {
            let text = self.to_text(value).to_vec();
            match trace {
                // `IF`/plain `WHEN`'s own `>>>` (`IfInstruction.cpp:140`, and
                // `select` / `when 1 = 1 then` measured to show the identical
                // shape) -- measured, `WHILE`/`UNTIL` never get this (this
                // task's report: `trace r` over a `DO WHILE` shows only `>K>
                // "WHILE" => ...`, no bare `>>>` alongside it), which is why
                // this is a variant a caller picks rather than something
                // `eval_condition` decides on its own. `SELECT CASE`'s own
                // `WHEN`/`WhenCase` comparison never reaches this function at
                // all -- see `test_case_when`'s own trace calls instead.
                ConditionTrace::Result(indent) => self.trace_result(indent, &text),
                // `WHILE`/`UNTIL`'s own `>K>` (`DoBlockComponents.cpp`'s
                // `traceKeywordResult(WHILE, ...)`/`(UNTIL, ...)`) -- re-fires
                // every pass because the oracle re-evaluates the condition every
                // pass too, which `run_repeating`'s own call site already does
                // without any change from this task.
                ConditionTrace::Keyword(indent, keyword) => {
                    self.trace_keyword(indent, keyword, &text);
                }
            }
            Self::condition_holds(checked, &text, raise)
        } else {
            // **Off the tracing path the bytes need not be moved anywhere.**
            // `to_text` would copy them into the interpreter's scratch slot
            // and hand back a borrow of it; for a value that carries its own
            // bytes that copy buys nothing. A condition's value is one byte --
            // `0` or `1` -- far more often than it is anything else, and one
            // byte always rides in the handle.
            match value.decode() {
                Decoded::Text(inline) => Self::condition_holds(checked, &inline, raise),
                // `to_text` renders these two as `"0"` and `"1"`, which both
                // arms of `condition_holds` then read as this same answer.
                // Every other integer falls to the arm below, so the failure
                // it raises still names the value as the oracle spells it.
                Decoded::SmallInt(number) if number == 0 || number == 1 => Ok(number == 1),
                _ => {
                    let text = self.to_text(value);
                    Self::condition_holds(checked, &text, raise)
                }
            }
        };
        // After the rendering above is out of scope, so the borrow it may hold
        // on `self` has ended. The decision itself touches neither `self` nor
        // the roots, so making it before this pop rather than after changes no
        // answer and no lifetime.
        self.roots.pop_frame(frame);
        decided
    }

    /// Whether a condition's rendered text holds, and the failure when it is
    /// neither `0` nor `1`.
    fn condition_holds(
        checked: bool,
        text: &[u8],
        raise: fn(&[u8]) -> Raised,
    ) -> Result<bool, Failure> {
        if checked {
            Ok(text == b"1")
        } else {
            logical_value(text).ok_or_else(|| raise(text).into())
        }
    }

    /// Whether any of a `WHEN CASE`'s `values` compares `==` (byte-for-byte,
    /// no padding, no numeric awareness) equal to the `SELECT CASE`'s own
    /// `case_text`, matching on the first that does (an OR of `==`, the
    /// opposite of a plain `WHEN`'s comma list, which is an AND checked for
    /// `0`/`1` -- `ast.rs`'s own doc comment on `WhenCase`).
    fn test_case_when(
        &mut self,
        code: &Code<'_>,
        values: &[Expr],
        case_text: &[u8],
        indent: usize,
    ) -> Result<bool, Failure> {
        for value in values {
            let value = self.eval(code, value)?;
            self.roots.push_temp(value);
            let text = self.to_text(value).to_vec();
            self.trace_result(indent, &text);
            let matched = text == case_text;
            self.trace_result(indent, if matched { b"1" } else { b"0" });
            if matched {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Parses `text` as an `INTERPRET` fragment and runs it **inside the
    /// current activation**.
    /// ```text
    ///      2 *-*   leave outer
    ///      2 *-*   interpret "leave outer"
    /// ```
    fn run_fragment(&mut self, text: Vec<u8>) -> Result<Flow, Failure> {
        let fragment: Rc<Fragment> = match parse_interpret(text) {
            Ok(fragment) => Rc::new(fragment),
            // **Step 5b: the oracle's own condition, not a loud refusal.**
            // Measured, `interpret "do forever then"` on line 2 raises 27.901
            // at rc 229; this used to be `Loud::parse`, `rexx-exec: INTERPRET
            // text did not parse: ...` at rc 120. `error.rs`'s own `impl
            // From<&ParseError> for Raised` has the transcript and states
            // exactly what the conversion cannot carry.
            Err(error) => return Err(Raised::from(&error).into()),
        };

        // An owned `Fragment` would do here, since nothing but this loop reads
        // it. It is an `Rc` because an `INTERPRET` inside a fragment makes this
        // function reentrant and each level anchors its own.
        let (slots, fragment_plan) = self.fragment_plan(&fragment);
        let code = Code {
            body: &fragment.body,
            symbols: &fragment.symbols,
            slots: &slots,
            // **A fragment carries its own plan, with every slot translated
            // into the enclosing frame** (`Interp::fragment_plan`). It used to
            // carry none, so its clause indents and compound splits were
            // computed the way they were before either table existed; it needs
            // one now because a fragment compiles, and a chunk's `Op::Load`
            // and `Op::Store` carry plan slots.
            plan: Some(&fragment_plan),
        };

        // `exit` inside `INTERPRET` ends the program, not the fragment, so
        // it has to propagate rather than stop here -- `run_bounded`'s own
        // catch-all does exactly that for anything it does not own, `Exit`
        // included, with nothing fragment-specific to add.
        let chunk = match crate::ir::compile(&fragment.body, &fragment_plan, self.chunk_trace()) {
            Ok(chunk) => chunk,
            Err(_) => {
                self.seal_site_level();
                return Err(Loud::chunk_refused().into());
            }
        };
        let registers = self.roots.reserve_temps(chunk.registers as usize);
        let ran = self.run_bounded(
            &code,
            0,
            code.body.instructions.len(),
            Some(&fragment.source),
            BodyEngine::Chunk {
                chunk: &chunk,
                registers,
            },
        );
        // Truncated on both paths, exactly as `run_chunk` does: a region left
        // behind would keep its registers rooted for the rest of the run.
        self.roots.pop_frame(registers);
        let flow = match ran {
            Ok(flow) => flow,
            Err(failure) => {
                self.seal_site_level();
                return Err(failure);
            }
        };

        // The exhausted search, at the fragment's own boundary rather than
        // the program's: measured, the oracle's `LEAVE`/`ITERATE` search
        // does not cross an `INTERPRET` (this function's own doc comment has
        // the six transcripts). Byte-identical in shape to
        // `run_activation`'s own four arms, deliberately -- same four
        // constructors, same `record_leave_failure` call -- because it is
        // the same event happening at a different boundary.
        match flow {
            Flow::Leave(name, origin) => {
                self.record_leave_failure(&origin);
                self.seal_site_level();
                let raised = match name {
                    None => raised_leave_no_loop(),
                    Some(id) => raised_leave_no_match(fragment.symbols.name(id).as_bytes()),
                };
                Err(raised.into())
            }
            Flow::Iterate(name, origin) => {
                self.record_leave_failure(&origin);
                self.seal_site_level();
                let raised = match name {
                    None => raised_iterate_no_loop(),
                    Some(id) => raised_iterate_no_match(fragment.symbols.name(id).as_bytes()),
                };
                Err(raised.into())
            }
            other => Ok(other),
        }
    }

    /// Closes off the level that is unwinding now, so the level above it can
    /// record its own clause.
    pub(crate) fn seal_site_level(&mut self) {
        if let Some(site) = self.failure_site.take() {
            self.failure_sites.push(site);
        }
    }

    /// Drops one `DROP` target: a plain variable, a whole stem, one tail, or
    /// the `(v)` indirect form.
    /// ```text
    /// a=1; b=2; v='a b'      ; drop (v); say a; say b   ->  A / B  (both dropped)
    /// x=1;      v=' x '      ; drop (v); say x           ->  X      (trimmed)
    /// v='9'                  ; drop (v)                  ->  Error 31.2
    /// v='.x'                 ; drop (v)                  ->  Error 31.3
    /// w=1;      v='(w)'      ; drop (v)                  ->  Error 20.928
    /// a=1;      v='a'        ; drop (v); say a           ->  A      (agrees with the old reading)
    /// ```
    fn drop_variable(&mut self, code: &Code<'_>, variable: &VariableRef) -> Result<(), Failure> {
        match variable {
            VariableRef::Direct(id) => {
                let name = code.symbols.name(*id);
                if shape_of(name.as_bytes()) == NameShape::Compound {
                    let (stem_name, stem_at) = code.stem(*id);
                    let key = self.tail_key(code, *id)?;
                    self.stem_drop_tail_at(stem_name, stem_at, &key);
                } else {
                    self.drop_by_name(name.as_bytes());
                }
            }
            VariableRef::Indirect(id) => {
                let (value, _novalue) = self.read(code, *id);
                let value = self.required_string_value(value)?;
                let text = self.to_text(value).into_owned();
                let mut names = Vec::new();
                for word in split_indirect_words(&text) {
                    names.push(validate_indirect_word(word)?);
                }
                for name in &names {
                    self.drop_by_name(name);
                }
            }
        }
        Ok(())
    }

    /// Drops the variable, whole stem, or one verbatim-keyed tail `name`'s
    /// own spelling names, dispatched by `shape_of`.
    fn drop_by_name(&mut self, name: &[u8]) {
        match shape_of(name) {
            NameShape::Simple => {
                let slot = self.slot_of(name);
                let frame = self.activation().frame;
                self.clear_variable(frame, slot);
            }
            NameShape::Stem => self.stem_drop(name),
            NameShape::Compound => {
                let dot = name
                    .iter()
                    .position(|&b| b == b'.')
                    .expect("NameShape::Compound guarantees at least one period");
                let (stem_name, key) = name.split_at(dot + 1);
                self.stem_drop_tail(stem_name, key);
            }
        }
    }

    /// `NUMERIC DIGITS`/`FUZZ`/`FORM`, in every spelling the parser produces
    /// (`NumericSetting`, `rexx-parse`'s own `instruction.rs::numeric`).
    fn exec_trace(&mut self, code: &Code<'_>, setting: &Trace) -> Result<(), Failure> {
        match setting {
            Trace::Default => {
                // `setTraceNormal`, which is silent here and is *not*
                // `TRACE OFF` -- measured, `trace r` then bare `trace` then
                // `trace()` gives `N`, where `trace off` gives `O`.
                self.set_trace_mode(crate::trace::TraceMode::NORMAL);
            }
            Trace::Setting(bytes) => {
                self.set_trace_mode(
                    mode_from_setting(bytes)
                        .expect("rexx-parse's check_trace_setting already validated this byte"),
                );
            }
            // 24.901, unconditional -- measured, `trace 0` raises it
            // exactly like `trace 5` (this task's report), because this
            // runtime has no interactive debugging for a nonzero skip
            // count to be valid *from* either way.
            Trace::Skip(_) => {
                return Err(raised_numeric_trace_interactive_only().into());
            }
            // `TRACE VALUE expr`: computed at run time, then classified
            // exactly like a literal `TRACE` setting would have been --
            // measured, `trace value 5` raises 24.901 like `trace 5`, and
            // `trace value 'R'` behaves exactly like `trace r` (this
            // task's report has both transcripts). A whole number is a
            // skip count checked *before* trying it as a letter, matching
            // `rexx-parse`'s own `trace` parser's order (`instruction.rs`'s
            // `whole_number` attempt precedes its `check_trace_setting`
            // fallback).
            Trace::Value(expression) => {
                let value = self.eval(code, expression)?;
                self.roots.push_temp(value);
                // `result->requestString()` (`instructions/TraceInstruction
                // .cpp:172`), and 24.1 names the conversion: measured, oracle
                // rc 232, `trace value .K` with a class-side `makeString`
                // returning `'ZZ'` reports `found "Z"`.
                let value = self.required_string_value(value)?;
                let text = self.to_text(value).to_vec();
                if is_whole_number(&text) {
                    return Err(raised_numeric_trace_interactive_only().into());
                }
                self.set_trace_mode(mode_from_setting(&text).map_err(raised_invalid_trace_letter)?);
            }
        }
        // `RexxActivation::setTrace` calls `traceEntry()` right after
        // installing the new flags (`RexxActivation.cpp:1024`), which is the
        // route every 4c-reachable `>I>` takes. `Trace::Skip` never gets here
        // -- it returned above -- and the C++ agrees: its own arm sets no
        // flags and calls nothing.
        self.trace_invocation_entry();
        Ok(())
    }

    /// `>I>`, if this activation is a `::ROUTINE` still on its first
    /// instruction and the setting just installed traces labels.
    pub(crate) fn trace_invocation_entry(&mut self) {
        if !self.activation().trace_entry.may_announce() {
            return;
        }
        let Some(subject) = self.invocation_subject() else {
            return;
        };
        if !self.trace_mode().labels {
            return;
        }
        self.activation_mut().trace_entry = TraceEntry::Done;
        let package = self.program_path.clone().into_bytes();
        self.trace_invocation(">I>", &subject, &package);
    }

    /// `>I>` for a body whose package's `::OPTIONS TRACE` put a
    /// label-tracing setting in force before its first clause.
    pub(crate) fn trace_package_invocation_entry(&mut self) {
        if !self.trace_mode().labels || self.activation().trace_entry != TraceEntry::Pending {
            return;
        }
        let Some(subject) = self.invocation_subject() else {
            return;
        };
        self.activation_mut().trace_entry = TraceEntry::Done;
        let package = self.program_path.clone().into_bytes();
        self.trace_invocation(">I>", &subject, &package);
    }

    /// `<I<`, on every way a routine activation can end.
    pub(crate) fn trace_invocation_exit(&mut self) {
        if self.activation().trace_entry != TraceEntry::Done {
            return;
        }
        let Some(subject) = self.invocation_subject() else {
            return;
        };
        if !self.trace_mode().labels {
            return;
        }
        let package = self.program_path.clone().into_bytes();
        self.trace_invocation("<I<", &subject, &package);
    }

    /// Gives a fragment its own `>I>` count, and answers with the enclosing
    /// state for [`Interp::leave_fragment`] to put back.
    fn enter_fragment(&mut self, nested: bool) -> TraceEntry {
        let enclosing = self.activation().trace_entry;
        let entry = if enclosing.may_announce() && !nested {
            TraceEntry::Pending
        } else {
            TraceEntry::Spent
        };
        self.activation_mut().trace_entry = entry;
        enclosing
    }

    /// Puts the enclosing state back after a fragment, **except** that a
    /// fragment which announced leaves the activation `Done`.
    fn leave_fragment(&mut self, enclosing: TraceEntry) {
        if self.activation().trace_entry != TraceEntry::Done {
            self.activation_mut().trace_entry = enclosing;
        }
    }

    /// What the running activation announces itself as, or `None` when it
    /// announces nothing at all -- which is the `isMethodOrRoutine()` half of
    /// the gate, expressed as the lookup that would supply the substitutions.
    fn invocation_subject(&self) -> Option<Announced> {
        if let Some(identity) = &self.activation().method_identity {
            return Some(Announced::Method {
                name: identity.name.to_vec(),
                scope: self.class_id_text(identity.scope).as_bytes().to_vec(),
            });
        }
        let index = self.activation().body?;
        match &self.activation().program.directives.get(index)?.kind {
            DirectiveKind::Routine(routine) => Some(Announced::Routine {
                name: routine.name.to_vec(),
            }),
            _ => None,
        }
    }

    /// `ADDRESS`'s three environment-naming forms. The caller has already
    /// turned the command and `WITH` forms away.
    /// ```text
    /// address envC                    ->  ENVC
    /// address 'LiTeRaL'               ->  LiTeRaL
    /// nm = 'mIxEd'; address value nm  ->  mIxEd
    /// nm = 'mIxEd'; address (nm)      ->  mIxEd
    /// ```
    fn exec_address(
        &mut self,
        code: &Code<'_>,
        address: &rexx_parse::Address,
    ) -> Result<(), Failure> {
        // `environment` before `dynamic`, mirroring the C++'s own `if
        // (environment != OREF_NULL)` ahead of its `ADDRESS VALUE` arm. The
        // parser never fills both, so the order decides nothing today.
        match (&address.environment, &address.dynamic) {
            (None, None) => {
                self.activation_mut().address.toggle();
                return Ok(());
            }
            (Some(environment), _) => {
                if environment.len() > MAX_ADDRESS_NAME_LENGTH {
                    return Err(Raised::environment_name_too_long(
                        MAX_ADDRESS_NAME_LENGTH,
                        environment,
                    )
                    .into());
                }
                self.activation_mut().address.set_bytes(environment);
            }
            (None, Some(expression)) => {
                let value = self.eval(code, expression)?;
                self.roots.push_temp(value);
                // `_address = result->requestString()` before the `>>>`
                // (`instructions/AddressInstruction.cpp:182`), so the trace
                // names the conversion: measured, `trace r` over
                // `address (.K)` with a class-side `makeString` returning
                // `'CMD'` prints `>>>   "CMD"` and `ADDRESS()` answers `CMD`.
                let value = self.required_string_value(value)?;
                // Built in the lent buffer: the rendering exists only to be
                // traced and then copied into the `Rc`, so an owned `Vec` of
                // its own would be an allocation and a free per `ADDRESS
                // VALUE` clause.
                let mut text = self.take_result_buffer();
                text.extend_from_slice(&self.to_text(value));
                self.trace_result(self.clause_state.current_value_indent, &text);
                if text.len() > MAX_ADDRESS_NAME_LENGTH {
                    let raised = Raised::environment_name_too_long(MAX_ADDRESS_NAME_LENGTH, &text);
                    self.give_result_buffer(text);
                    return Err(raised.into());
                }
                self.activation_mut().address.set_bytes(&text);
                self.give_result_buffer(text);
            }
        }
        Ok(())
    }

    fn exec_numeric(
        &mut self,
        code: &Code<'_>,
        setting: &NumericSetting,
        expression: &Option<Expr>,
    ) -> Result<(), Failure> {
        match setting {
            NumericSetting::Digits => {
                match self.numeric_operand(code, expression, "DIGITS")? {
                    Some((operand, text)) => {
                        let parsed = String::from_utf8_lossy(&text);
                        let outcome = self.activation_mut().settings.set_digits_str(&parsed);
                        self.give_result_buffer(text);
                        if let Err(error) = outcome {
                            let named = self.to_text(operand).to_vec();
                            return Err(raised_naming_the_operand(error, &named).into());
                        }
                    }
                    // The reset stores the default and makes the operand
                    // form's FUZZ check against it, which is the arm the
                    // interpreter runs: it can fail, and the value it names
                    // is the default rather than the setting in force.
                    None => {
                        let default = self.package_default_numeric().digits();
                        self.activation_mut()
                            .settings
                            .reset_digits(default)
                            .map_err(raised_from_settings)?;
                    }
                }
            }
            NumericSetting::Fuzz => match self.numeric_operand(code, expression, "FUZZ")? {
                Some((operand, text)) => {
                    let parsed = String::from_utf8_lossy(&text);
                    let outcome = self.activation_mut().settings.set_fuzz_str(&parsed);
                    self.give_result_buffer(text);
                    if let Err(error) = outcome {
                        let named = self.to_text(operand).to_vec();
                        return Err(raised_naming_the_operand(error, &named).into());
                    }
                }
                None => {
                    let default = self.package_default_numeric().fuzz();
                    self.activation_mut()
                        .settings
                        .reset_fuzz(default)
                        .map_err(raised_from_settings)?;
                }
            },
            // **`FormDefault` and `FormScientific` part company once
            // `::OPTIONS FORM ENGINEERING` moves the package default**:
            // measured, a bare `numeric form` in such a file answers
            // `ENGINEERING` where `numeric form scientific` answers
            // `SCIENTIFIC`.
            NumericSetting::FormDefault => {
                let default = self.package_default_numeric().form();
                self.activation_mut().settings.set_form(default);
            }
            NumericSetting::FormScientific => {
                self.activation_mut().settings.set_form(Form::Scientific);
            }
            NumericSetting::FormEngineering => {
                self.activation_mut().settings.set_form(Form::Engineering);
            }
            NumericSetting::FormValue => {
                // The parser only ever produces this with an expression: an
                // explicit `VALUE` with none is 35.917 at parse time
                // (`instruction.rs::numeric`), and the implicit spelling
                // (`NUMERIC FORM (expr)`) only takes this branch once a token
                // is already known to be there. Loud rather than a panic, on
                // this crate's own rule against aborting on a shape the
                // grammar rules out but the type does not.
                let Some(expression) = expression else {
                    return Err(Loud {
                        message: "NUMERIC FORM VALUE with no expression".to_string(),
                    }
                    .into());
                };
                // `set_form_str`'s own doc comment: the runtime `VALUE` path
                // does no uppercasing, no trimming and no abbreviation, unlike
                // the keyword spellings above -- measured, `numeric form
                // value 'engineering'` is 25.11, not accepted
                // case-insensitively.
                let value = self.eval(code, expression)?;
                self.roots.push_temp(value);
                let text = self.to_text(value).to_vec();
                // `>K>   "FORM" => "engineering"` (F2, branch review): fires
                // before `set_form_str`'s own validation, same as `DIGITS`/
                // `FUZZ` below and `setup_controlled`'s own `TO`/`BY`/`FOR`
                // -- measured, `numeric form value 'engineering'` under
                // `trace r` traces `>K>` and *then* raises 25.11, not the
                // reverse. Untranslated, matching the error report's own
                // `found "engineering"` substitution, which is also
                // unmodified case -- `set_form_str`'s own no-uppercasing
                // rule for this one path, unlike the two keyword spellings
                // above.
                self.trace_keyword(self.clause_state.current_value_indent, "FORM", &text);
                // The required-string protocol between the `>K>` and the
                // validation, which is where `requestString` sits
                // (`instructions/NumericInstruction.cpp:175`).
                let converted = self.required_string_value(value)?;
                let parsed = String::from_utf8_lossy(&self.to_text(converted)).into_owned();
                if let Err(error) = self.activation_mut().settings.set_form_str(&parsed) {
                    return Err(raised_naming_the_operand(error, &text).into());
                }
            }
        }
        Ok(())
    }

    /// Evaluates `expression`, or answers `default` when there is none
    /// (`NUMERIC DIGITS`/`FUZZ` alone).
    fn numeric_operand(
        &mut self,
        code: &Code<'_>,
        expression: &Option<Expr>,
        keyword: &str,
    ) -> Result<Option<(ObjRef, Vec<u8>)>, Failure> {
        let Some(expression) = expression else {
            return Ok(None);
        };
        let value = self.eval(code, expression)?;
        self.roots.push_temp(value);
        let mut text = self.take_result_buffer();
        text.extend_from_slice(&self.to_text(value));
        self.trace_keyword(self.clause_state.current_value_indent, keyword, &text);
        // **The required-string protocol runs after the `>K>` and its answer
        // replaces the bytes**, which is `requestUnsignedNumber`'s own
        // `requestString()` (`classes/ObjectClass.cpp:1077`). The trace above
        // names the object and the parse below reads the conversion --
        // measured, `numeric digits .K` with a class-side `makeString`
        // returning `12` traces `>K>   "DIGITS" => "The K class"` and then
        // answers `12` to `DIGITS()`.
        let converted = self.required_string_value(value)?;
        if converted != value {
            text.clear();
            text.extend_from_slice(&self.to_text(converted));
        }
        Ok(Some((value, text)))
    }

    /// `instruction`'s own clause text and the 1-based line to print it against,
    /// or `None` when `source` is `None`.
    pub(crate) fn clause_site(
        &self,
        source: Option<&ProgramSource>,
        instruction: &Instruction,
    ) -> Option<(usize, Vec<u8>)> {
        let line = self.clause_line(source, instruction)?;
        let source = source?;
        Some((
            line,
            source
                .join_span(instruction.clause_span.clone())
                .map_or_else(
                    // Visible rather than silent, matching `Raised::message`'s
                    // own reasoning for a catalogue miss: the error path is the
                    // worst place to turn a reportable condition into a crash or
                    // a blank line.
                    || b"<clause span outside the retained source>".to_vec(),
                    |bytes| bytes.into_owned(),
                ),
        ))
    }

    /// `clause_site`'s own line half, alone -- extracted so `current_clause_
    /// line` (`lib.rs`'s own doc comment on the field) can be kept fresh on
    /// every step without paying for `clause_site`'s own `join_span` text
    /// extraction, which nothing needs when only `SIGL`'s value is being
    /// computed. Identical rule, same `clause_line_override` honoured the
    /// same way, so a `SIGNAL`/`CALL` fired from inside an `INTERPRET`
    /// fragment reads the enclosing clause's own line here exactly as
    /// `clause_site` already gives the trace/error paths.
    pub(crate) fn clause_line(
        &self,
        source: Option<&ProgramSource>,
        instruction: &Instruction,
    ) -> Option<usize> {
        let source = source?;
        Some(
            self.clause_line_override
                .unwrap_or_else(|| source.line_of(instruction.clause_span.start)),
        )
    }

    /// [`Interp::clause_line`] for a caller that knows the instruction's
    /// **index**, which is what lets the answer come from `Plan::lines`
    /// rather than from a search.
    pub(crate) fn clause_line_at(
        &self,
        code: &Code<'_>,
        index: usize,
        instruction: &Instruction,
        source: Option<&ProgramSource>,
    ) -> Option<usize> {
        let source = source?;
        if let Some(line) = self.clause_line_override {
            return Some(line);
        }
        debug_assert!(
            code.body
                .instructions
                .get(index)
                .is_some_and(|at| std::ptr::eq(at, instruction)),
            "clause_line_at was given index {index} and an instruction that is not the one at \
             that index, so the table would be read for a different clause"
        );
        Some(match code.plan {
            Some(plan) => plan.line_at(instruction, source, index),
            None => source.line_of(instruction.clause_span.start),
        })
    }

    // `fragment_plan` and `slot_of` live in `plan.rs` (Task 6), beside `Plan`
    // itself; `stem_assign`/`stem_set`/`stem_drop`/`stem_drop_tail`/
    // `tail_key` live in `stem.rs` (Task 5), beside the rest of the D15a
    // library. `read` lives in `lib.rs`, beside `Interp`'s other value-model
    // entry points.
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
                            // rc_0`, this module's own test) is exactly a
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
                    // instead of aborting; `clause_site`'s own fallback, this
                    // file, cites the identical reasoning) is that a
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

/// Which of the three variable shapes `name`'s own spelling is, from an
/// already-interned (or already-upcased runtime) name alone.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) enum NameShape {
    Simple,
    Stem,
    Compound,
}

/// The frame slot a loop's control variable writes and re-reads, resolved
/// **once, when the loop is entered**, or `None` for a control this cannot
/// answer for.
fn control_slot(code: &Code<'_>, control: SymbolId) -> Option<usize> {
    match shape_of(code.symbols.name(control).as_bytes()) {
        NameShape::Simple => code.slot_for(control),
        NameShape::Stem | NameShape::Compound => None,
    }
}

pub(crate) fn shape_of(name: &[u8]) -> NameShape {
    let dots = name.iter().filter(|&&b| b == b'.').count();
    if dots == 0 {
        NameShape::Simple
    } else if dots == 1 && name.last() == Some(&b'.') {
        NameShape::Stem
    } else {
        NameShape::Compound
    }
}

/// Splits an indirect wrapper's value into its subsidiary list's words --
/// `DROP (v)`, `EXPOSE (v)` and `PROCEDURE EXPOSE (v)` all spell the same
/// list and reach the same split.
fn split_indirect_words(text: &[u8]) -> impl Iterator<Item = &[u8]> {
    text.split(|&b| b == b' ' || b == b'\t')
        .filter(|word| !word.is_empty())
}

/// One legal symbol character, by the scanner's own character table
/// (`scanner.rs`'s `is_symbol_char`: `! . ? _ 0-9 A-Z a-z`, ASCII only --
/// `SymbolTable::intern`'s own doc says a non-ASCII byte cannot be part of a
/// symbol at all).
fn is_symbol_byte(b: u8) -> bool {
    matches!(b, b'!' | b'.' | b'?' | b'_' | b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z')
}

/// Validates one word of an indirect subsidiary list and answers its upcased
/// name, or the condition the oracle raises for it.
fn validate_indirect_word(word: &[u8]) -> Result<Vec<u8>, Failure> {
    if !word.iter().copied().all(is_symbol_byte) {
        return Err(raised_symbol_expected(word).into());
    }
    match word[0] {
        b'0'..=b'9' => return Err(raised_digit_led(word).into()),
        b'.' => return Err(raised_dot_led(word).into()),
        _ => {}
    }
    Ok(word.to_ascii_uppercase())
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
fn skip_else(instructions: &[Instruction], target: usize) -> usize {
    match instructions.get(target).map(|i| &i.kind) {
        Some(InstructionKind::Else { then_exit }) => then_exit.unwrap_or(instructions.len()),
        _ => target,
    }
}

/// Whether `index` is a clause of `body`'s own top level -- reached by
/// falling from the instruction before it, with no `DO`, `LOOP`, `SELECT` or
/// `IF` construct enclosing it.
fn top_level_clause(body: &CodeBody, index: usize) -> bool {
    let instructions = &body.instructions;
    let len = instructions.len();
    let mut at = 0;
    while at < len {
        if at == index {
            return true;
        }
        at = match &instructions[at].kind {
            InstructionKind::Do(body) | InstructionKind::Loop(body) => {
                body.end.map_or(len, |end| end + 1)
            }
            InstructionKind::Select { end, .. } => end.map_or(len, |end| end + 1),
            InstructionKind::If { false_target, .. } => match false_target {
                Some(target) => skip_else(instructions, *target),
                None => len,
            },
            _ => at + 1,
        };
    }
    false
}

/// 20.928: a subsidiary-list word is not a legal symbol at all (contains a
/// byte outside `is_symbol_byte`'s set, which is also what a parenthesised
/// entry like `"(w)"` fails on).
fn raised_symbol_expected(found: &[u8]) -> Raised {
    Raised::syntax(20, 928, vec![found.to_vec()])
}

/// 31.2: a subsidiary-list word starts with a digit.
fn raised_digit_led(found: &[u8]) -> Raised {
    Raised::syntax(31, 2, vec![found.to_vec()])
}

/// 31.3: a subsidiary-list word starts with a period.
fn raised_dot_led(found: &[u8]) -> Raised {
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
fn raised_guard_not_logical(found: &[u8]) -> Raised {
    Raised::syntax(34, 902, vec![found.to_vec()])
}

/// 34.2: a single (non-list) `WHEN` condition is not exactly `0` or `1`.
/// `Error_Logical_value_when`, the same shape as `raised_if_not_logical`
/// with `WHEN`'s own sub-number -- a plain `WHEN`'s comma list is the
/// opposite case (`WhenCase`'s doc comment) and never reaches this raiser:
/// [`Interp::eval_condition`] hands a list over already `checked`, and
/// `crate::ir::compile`'s `native_shape` declines `ExprKind::Logical`, so a
/// list never becomes a `crate::ir::Op::Condition` either.
pub(crate) fn raised_when_not_logical(found: &[u8]) -> Raised {
    Raised::syntax(34, 2, vec![found.to_vec()])
}

/// 7.3: a `SELECT` reached its `END` with every `WHEN` false and no
/// `OTHERWISE`. `Error_When_expected_nootherwise`, catalogue text "All WHEN
/// expressions of SELECT are false; OTHERWISE expected.", no substitutions
/// (measured against `interpreter/messages/rexxmsg.xml`'s own `<Text>` for
/// major 7 sub 003, which carries no `<Sub>` tag).
fn raised_select_no_when() -> Raised {
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
fn raise_syntax_condition(text: &[u8], additional: Vec<Vec<u8>>) -> Raised {
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
    let (major, sub) = (major as u16, sub as u16);
    if rexx_inventory::errors::lookup(major, sub).is_some() {
        return Raised::syntax(major, sub, additional);
    }
    let found = if rexx_inventory::errors::lookup(major, 0).is_some() {
        (u32::from(major) * 1000 + u32::from(sub)).to_string()
    } else {
        format!("{major}.{sub}")
    };
    Raised::syntax(98, 941, vec![found.into_bytes()])
}

/// A `RAISE`'s condition name as `Raised::condition` carries it.
fn condition_name(name: &[u8]) -> Cow<'static, str> {
    Cow::Owned(String::from_utf8_lossy(name).into_owned())
}

/// [`raised_from_settings`] with the operand's own rendering in place of the
/// string the required-string protocol converted it to.
fn raised_naming_the_operand(error: SettingsError, operand: &[u8]) -> Raised {
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
fn raised_while_not_logical(found: &[u8]) -> Raised {
    Raised::syntax(34, 3, vec![found.to_vec()])
}

/// 34.4: `UNTIL`'s own version of `raised_while_not_logical`.
fn raised_until_not_logical(found: &[u8]) -> Raised {
    Raised::syntax(34, 4, vec![found.to_vec()])
}

/// 26.2: a bare `DO`'s own repetition-count expression is not zero or a
/// positive whole number. `Error_Invalid_expression_do`, measured: `do
/// 'a'`/`do -1`/`do 2.5` all give this, `found` the operand's own
/// unmodified text (`"a"`/`"-1"`/`"2.5"`).
fn raised_repetition_count_not_whole(found: &[u8]) -> Raised {
    Raised::syntax(26, 2, vec![found.to_vec()])
}

/// 26.3: a `DO`/`LOOP`'s `FOR` expression is not zero or a positive whole
/// number. Measured: `do i = 1 to 3 for 'x'`/`for -1`/`for 1.5`.
fn raised_for_count_not_whole(found: &[u8]) -> Raised {
    Raised::syntax(26, 3, vec![found.to_vec()])
}

/// 28.1: a bare `LEAVE` found no repetitive loop or labeled block
/// instruction anywhere on the enclosing chain. No substitution.
fn raised_leave_no_loop() -> Raised {
    Raised::syntax(28, 1, Vec::new())
}

/// 28.2: a bare `ITERATE` found no repetitive loop anywhere on the
/// enclosing chain. No substitution.
fn raised_iterate_no_loop() -> Raised {
    Raised::syntax(28, 2, Vec::new())
}

/// 28.3: a named `LEAVE name` found nothing on the enclosing chain whose own
/// label (`DO LABEL`, or a controlled/`OVER` loop's own control variable)
/// matches `name` -- **an ordinary clause label never matches**, measured:
/// `outer: do i = 1 to 3` then `leave outer` is this, not a hit. `found` is
/// the symbol's own (already-upcased) spelling.
fn raised_leave_no_match(found: &[u8]) -> Raised {
    Raised::syntax(28, 3, vec![found.to_vec()])
}

/// 28.4: `ITERATE`'s own version of `raised_leave_no_match`.
fn raised_iterate_no_match(found: &[u8]) -> Raised {
    Raised::syntax(28, 4, vec![found.to_vec()])
}

/// 28.5: a named `ITERATE name` matched a block on the enclosing chain by
/// label, but that block is not a repetitive loop (a labelled `DO`/plain
/// block, or a `SELECT LABEL` -- `ITERATE` never accepts either, unlike
/// `LEAVE`). Measured: `do label x / say 1 / iterate x / end` gives this,
/// not 28.4, because `x` *did* match something.
fn raised_iterate_wrong_kind(found: &[u8]) -> Raised {
    Raised::syntax(28, 5, vec![found.to_vec()])
}

/// Whether `a < b`, numerically, through `rexx-num`'s own `compare_decoded`
/// rather than a hand-rolled sign comparison -- this crate's standing rule
/// against a second copy of a comparison `rexx-num` already owns
/// (`compare_values`' own doc comment states it for the expression
/// operators; a controlled loop's own bound test and its `BY`'s sign are
/// the same rule applied to two `Number`s this crate already holds, not a
/// different one).
fn numeric_less(a: &Number, b: &Number, digits: u64, fuzz: u64) -> Result<bool, ArithError> {
    compare_decoded(b"", Some(a), b"", Some(b), digits, fuzz, CompareOp::Less)
}

/// Rounds `number` under `digits`, mirroring the oracle's own unary `+` at
/// a controlled loop's entry (F1, branch review, Important): `setup_
/// controlled` used to store `initial`/`to`/`by` as their exact parse, but
/// `ControlledLoop::setup` (`DoBlockComponents.cpp:126-166`, verified by
/// containment) rounds all three with `callOperatorMethod(OPERATOR_PLUS,
/// ...)` before the loop ever starts. Masked while `NUMERIC DIGITS` stays
/// constant (every later use re-rounds to the same width anyway) and wrong
/// the moment digits changes inside the loop body -- measured: `numeric
/// digits 3; do i = 1.23456 to 3; say i; numeric digits 9; end` is `1.23 /
/// 2.23 / 3.23` on the oracle (the header values were rounded once, at
/// entry, under digits 3, and stay that width even after digits widens);
/// this crate gave `1.23 / 2.23456 / 3.23456` before this fix (the exact
/// parse survived into the wider-digits passes untouched).
fn round_via_unary_plus(number: &Number, digits: u64) -> Result<Number, ArithError> {
    Number::zero().add(number, digits)
}

#[cfg(test)]
mod tests;
