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

//! `DO`/`LOOP`: the header, each kind of repetition, and the flat-loop path the op driver takes.

use super::{
    ArithError, BehaviourId, Body, BodyEngine, ClauseOutcome, ClauseValue, Code, CompareOp,
    ConditionTrace, ControlExpr, Cow, Decoded, Expr, ExprKind, Failure, Flow, Instruction,
    InstructionKind, Interp, Loop, LoopConditional, LoopKind, Loud, NameShape, Novalue, Number,
    ObjRef, ProgramSource, Raised, RegFrame, SymbolId, compare_decoded, exact_small_int,
    raised_available_not_logical, raised_for_count_not_whole, raised_iterate_wrong_kind,
    raised_repetition_count_not_whole, raised_until_not_logical, raised_while_not_logical,
    shape_of, within_digits,
};
use crate::clause::DeadlineCounted;

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

/// One repeating loop being driven from the op
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
    pub(crate) do_indent: usize,
    pub(crate) loop_indent: usize,
    do_line: usize,
    end_line: usize,
    header_clause: HeaderClause,
    /// The echo site `header_clause` names when it is `Iterate`.
    iterate_site: IterateSite,
    /// What a pass boundary tests beyond the advance. **The condition's own
    /// node is not held here**, because this outlives the borrow of `code` a
    /// reference to it would need; the node is read back off the `DO`
    /// instruction at the one point per pass that tests it.
    test: PassTest,
    counter: Option<LoopCounter>,
    state: LoopState,
}

/// A flat loop's per-pass tests: `UNTIL` after the body, and whether the
/// header takes [`Interp::flat_loop_header_general`] for a `WHILE` or a
/// `COUNTER`. The two are bits of one byte, so a loop with neither pays one
/// test of it per pass.
#[derive(Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
enum PassTest {
    Plain = 0,
    Until = 1,
    General = 2,
    UntilGeneral = 3,
}

impl PassTest {
    fn of(body: &Loop) -> PassTest {
        match (&body.conditional, body.counter.is_some()) {
            (Some(cond), counted) if cond.until => {
                if counted {
                    PassTest::UntilGeneral
                } else {
                    PassTest::Until
                }
            }
            (Some(_), _) | (None, true) => PassTest::General,
            (None, false) => PassTest::Plain,
        }
    }

    fn until(self) -> bool {
        self as u8 & 1 != 0
    }

    fn general(self) -> bool {
        self as u8 & 2 != 0
    }
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
            test: PassTest::Plain,
            counter: None,
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

/// What a header clause's own answer means: `Some(flow)` is the
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

/// The `WHILE`/`UNTIL` of the `DO`/`LOOP` at `index`, read back off
/// the instruction because a [`FlatLoop`] outlives any borrow of `code`.
fn loop_conditional_of<'a>(code: &'a Code<'_>, index: usize) -> Option<&'a LoopConditional> {
    match &code.body.instructions.get(index)?.kind {
        InstructionKind::Do(body) | InstructionKind::Loop(body) => body.conditional.as_ref(),
        _ => None,
    }
}

/// What `Interp::flat_loop_start` decided.
pub(crate) enum FlatStart {
    /// Driven from the driver's frame. The state is on `Activity::flat_loops`;
    /// this is the body's own instruction range, which the driver's frame
    /// absorbs an escaping `Flow` against.
    Flat { body_start: usize, end_index: usize },
    /// The header said zero passes, so the construct is already over.
    Ended(Flow),
    /// An unlabelled `DO; ... END` block whose `DO` clause has run: the body
    /// runs inline and its `END` is an ordinary clause, with nothing pushed.
    Block,
    /// Not a shape the flat path drives: take the nested path, with the header
    /// values handed back so that the nested path can move them rather than
    /// this one copying them.
    Fallback(LoopHeaderValues),
}

/// What one pass boundary decided.
pub(crate) enum FlatStep {
    /// One more pass, from this op.
    Body(u32),
    /// The construct is over and this is its answer.
    Done(Flow),
}

/// What drives one repeating `DO`/`LOOP`'s own iteration, once its header
/// has already been evaluated and validated -- everything `LoopKind` can be
/// except `Simple` (a block, never repeats, and `run_loop_with_header`'s own
/// `Simple` arm never builds one of these at all).
enum LoopState {
    Forever,
    /// `DO expr`: a fixed repeat count, decremented to zero.
    Count {
        remaining: u64,
    },
    /// `DO name OVER expr`: binds `control` to each of `items` in turn. A
    /// stem target iterates its tails in this crate's order rather than the
    /// oracle's (Deviation 1, `phase-4-exclusions.txt`).
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
    /// `DO WITH INDEX i ITEM v OVER expr`: steps the supplier the target
    /// answered `SUPPLIER` with, `WithLoop::checkIteration`
    /// (`instructions/DoBlockComponents.cpp`).
    With(Box<WithState>),
    /// A controlled loop whose header value or control variable is an
    /// object: driven by message, as `DoBlock::checkControl` drives every
    /// controlled loop.
    ObjectControlled(Box<ObjectControl>),
}

/// A controlled loop driven by the operator sends themselves: `+` with `BY`
/// at the increment and the `TO` comparison on what the control holds, each
/// answer kept as it is.
struct ObjectControl {
    control: SymbolId,
    /// [`control_slot`], taken once when this loop was entered.
    at: Option<usize>,
    shape: NameShape,
    /// The first pass's value, until that pass binds it.
    initial: Option<ObjRef>,
    to: Option<ObjRef>,
    by: ObjRef,
    /// Whether the comparison is `<` (a negative `BY`) rather than `>`.
    descending: bool,
    for_remaining: Option<u64>,
    stepped: bool,
}

/// A controlled header's `+` answers where one of its values is an object.
#[derive(Default)]
pub(crate) struct ObjectHeader {
    initial: Option<ObjRef>,
    to: Option<ObjRef>,
    by: Option<ObjRef>,
    descending: bool,
}

impl FlatLoop {
    /// The key [`Activity::loop_objects`] holds this loop's objects under
    /// where it is driven by message: its boxed state's address, which stays
    /// put for the loop's life.
    ///
    /// [`Activity::loop_objects`]: crate::activity::Activity::loop_objects
    pub(crate) fn object_key(&self) -> Option<usize> {
        match &self.state {
            LoopState::ObjectControlled(ctl) => Some(object_key(ctl)),
            _ => None,
        }
    }
}

/// [`FlatLoop::object_key`] for the boxed state itself.
fn object_key(ctl: &ObjectControl) -> usize {
    std::ptr::from_ref(ctl) as usize
}

/// A `DO WITH` loop's supplier and the names it binds.
struct WithState {
    index: Option<SymbolId>,
    item: Option<SymbolId>,
    supplier: ObjRef,
    /// Whether the next advance is the first, which reads the supplier's
    /// current pair rather than stepping past it.
    first: bool,
    remaining: Option<u64>,
}

/// A loop's `COUNTER` variable and the passes entered so far
/// (`DoBlock::setCounter`).
#[derive(Clone, Copy)]
struct LoopCounter {
    name: SymbolId,
    /// [`control_slot`], taken once when this loop was entered.
    at: Option<usize>,
    shape: NameShape,
    passes: u64,
}

impl LoopCounter {
    fn of(code: &Code<'_>, body: &Loop) -> Option<LoopCounter> {
        let name = body.counter?;
        Some(LoopCounter {
            name,
            at: control_slot(code, name),
            shape: shape_of(code.symbols.name(name).as_bytes()),
            passes: 0,
        })
    }
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
    /// `DO WITH ... OVER expr`'s target, echoed under the `WITH` tag.
    With,
    /// A `COUNTER` variable's reset to `0`, ahead of every header
    /// expression (`DoBlock::DoBlock`). Its slot has no expression.
    Counter,
    /// The `FOR` count of `DO name OVER expr FOR expr` and of `DO WITH ...
    /// OVER expr FOR expr`, **echoed under the `FOR` tag** (measured against
    /// the oracle: `do qq over zs for 1` prints `>K>   "FOR" => "1"`, on both
    /// `trace i` and `trace r` -- the same tag a controlled loop's own `FOR`
    /// and a bare `DO`'s repeat count carry; `do_with_trace.rex` measures the
    /// `DO WITH` form).
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
            HeaderRole::With => Some("WITH"),
            HeaderRole::Counter => Some("COUNTER"),
        }
    }
}

/// The header expressions of one `DO`/`LOOP`, in **the order they are
/// evaluated**, which is the order they were written in
/// (`Controlled::order`, recorded because an expression can have side
/// effects).
pub(crate) struct HeaderPlan {
    roles: [HeaderRole; 5],
    len: usize,
}

impl HeaderPlan {
    fn new() -> HeaderPlan {
        HeaderPlan {
            roles: [HeaderRole::Initial; 5],
            len: 0,
        }
    }

    fn push(&mut self, role: HeaderRole) {
        debug_assert!(
            self.len < self.roles.len(),
            "a DO/LOOP header has more slots than a COUNTER, TO, BY, FOR and one control value"
        );
        self.roles[self.len] = role;
        self.len += 1;
    }

    /// The roles in evaluation order.
    pub(crate) fn roles(&self) -> &[HeaderRole] {
        &self.roles[..self.len]
    }
}

/// The header of `body`.
pub(crate) fn loop_header_plan(body: &Loop) -> HeaderPlan {
    let mut plan = HeaderPlan::new();
    if body.counter.is_some() {
        plan.push(HeaderRole::Counter);
    }
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
        LoopKind::Over { for_count, .. } => {
            plan.push(HeaderRole::Over);
            if for_count.is_some() {
                plan.push(HeaderRole::OverFor);
            }
        }
        LoopKind::With { for_count, .. } => {
            plan.push(HeaderRole::With);
            if for_count.is_some() {
                plan.push(HeaderRole::OverFor);
            }
        }
    }
    plan
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
        (LoopKind::With { target, .. }, HeaderRole::With) => Some(target),
        (LoopKind::With { for_count, .. }, HeaderRole::OverFor) => for_count.as_ref(),
        _ => None,
    }
}

/// The expression of `body`'s header at `slot` -- the compiled stream's own
/// addressing, where a slot is a position in [`HeaderPlan::roles`].
pub(crate) fn loop_header_slot(body: &Loop, slot: u32) -> Option<&Expr> {
    let plan = loop_header_plan(body);
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
    /// The supplier a `DO WITH`'s target answered `SUPPLIER` with.
    supplier: Option<ObjRef>,
    /// The register `over` or the `DO WITH` target was evaluated into, `None`
    /// off the compiled engine and for every other loop.
    /// [`Interp::flat_loop_start`] writes the snapshot or the supplier back
    /// into it, which roots it while the loop runs.
    pub(crate) over_register: Option<u16>,
    /// A bare `DO expr`'s repeat count.
    count: Option<u64>,
    /// Where a controlled header's value is an object, every `+` answer.
    objects: Option<Box<ObjectHeader>>,
}

impl Interp {
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
        self.trace_keyword(
            self.activity.clause_state.current_value_indent,
            keyword,
            &text,
        );
    }

    /// One `Op::LoopHeaderValue`: a `COUNTER`'s reset, or `value` validated
    /// and filed in `values` with the register `src` it came from where that
    /// register roots the running loop's snapshot or supplier.
    pub(crate) fn file_header_value(
        &mut self,
        code: &Code<'_>,
        instruction: &Instruction,
        role: HeaderRole,
        src: u16,
        value: ObjRef,
        values: &mut LoopHeaderValues,
    ) -> Result<(), Failure> {
        match role {
            HeaderRole::Counter => return self.reset_loop_counter(code, instruction, value),
            HeaderRole::Over | HeaderRole::With => values.over_register = Some(src),
            _ => {}
        }
        self.accept_header_value(role, value, values)
    }

    /// Validates one header value against whatever its role requires and files
    /// it in `values`.
    fn accept_header_value(
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
                    _ => match self.header_number(value)? {
                        Some(number) => ControlValue::Wide(number),
                        None => return self.accept_object_header(role, value, values),
                    },
                });
            }
            // An object goes by message; the other values stay numbers, which
            // [`Interp::object_control_state`] turns into the objects their
            // `+` would answer.
            HeaderRole::To | HeaderRole::By => match self.header_number(value)? {
                Some(number) if role == HeaderRole::To => values.to = Some(number),
                Some(number) => values.by = Some(number),
                None => self.accept_object_header(role, value, values)?,
            },
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
            // `DO OVER` hands its target to `requestArray` -- see
            // [`Interp::over_snapshot`].
            HeaderRole::Over => values.over = Some(value),
            HeaderRole::With => values.supplier = Some(self.with_supplier(value)?),
            // [`Interp::file_header_value`] answers this role itself.
            HeaderRole::Counter => {}
        }
        Ok(())
    }

    /// `WithLoop::setup`: the supplier `value` answers `SUPPLIER` with, 98.994
    /// when that is not a `Supplier`.
    fn with_supplier(&mut self, value: ObjRef) -> Result<ObjRef, Failure> {
        let caller = self.caller();
        let answered = pinned!(
            self,
            crate::pinning::PinKind::LoopHeader,
            self.send_message(value, b"SUPPLIER", None, &[], caller)
        )?;
        if let Some(supplier) = answered {
            self.roots.activity_mut().push_temp(supplier);
            let class = self.class_of_value(supplier);
            let wanted = self.classes().lookup("Supplier");
            if let (Some(class), Some(wanted)) = (class, wanted)
                && self.classes().is_a(class, wanted)
            {
                return Ok(supplier);
            }
        }
        let found = self.string_value_text(value);
        Err(Raised::syntax(98, 994, vec![found]).into())
    }

    /// A `COUNTER`'s reset to `value` and its `>K>` line, at the `DO`/`LOOP`
    /// clause's own indent, before any header expression runs.
    fn reset_loop_counter(
        &mut self,
        code: &Code<'_>,
        instruction: &Instruction,
        value: ObjRef,
    ) -> Result<(), Failure> {
        let (InstructionKind::Do(body) | InstructionKind::Loop(body)) = &instruction.kind else {
            return Err(Loud::loop_op_off_its_node().into());
        };
        let Some(counter) = LoopCounter::of(code, body) else {
            return Err(Loud::loop_op_off_its_node().into());
        };
        let indent = self.activity.clause_state.current_value_indent;
        self.bind_control(code, counter.name, indent, value, counter.at, counter.shape)?;
        self.echo_header_value(HeaderRole::Counter, value);
        Ok(())
    }

    /// One pass's `COUNTER` assignment and its `>K>` line, at the loop body's
    /// indent (`DoBlock::setCounter`).
    fn count_pass(
        &mut self,
        code: &Code<'_>,
        counter: &mut LoopCounter,
        loop_indent: usize,
    ) -> Result<(), Failure> {
        counter.passes += 1;
        let value = self.counted(usize::try_from(counter.passes).unwrap_or(usize::MAX));
        // A pass's own frame: the counter variable roots `value` once it is
        // bound, so nothing pushed here outlives the pass.
        let pass = self.roots.activity_mut().push_frame();
        self.roots.activity_mut().push_temp(value);
        self.bind_control(
            code,
            counter.name,
            loop_indent,
            value,
            counter.at,
            counter.shape,
        )?;
        if self.trace_mode().results {
            let text = self.string_value_text(value);
            self.trace_keyword(loop_indent, "COUNTER", &text);
        }
        self.roots.activity_mut().pop_frame(pass);
        Ok(())
    }

    /// `OverLoop::setup`'s conversion
    /// (`instructions/DoBlockComponents.cpp:233`): the array whose items a
    /// `DO OVER` binds in turn.
    fn over_target_array(&mut self, value: ObjRef) -> Result<ObjRef, Failure> {
        if self.array_slots_of(value).is_some() {
            return Ok(value);
        }
        let converted = self.request_array_value(value)?;
        if let Some(converted) = converted
            && self.array_slots_of(converted).is_some()
        {
            return Ok(converted);
        }
        let found = self.string_value_text(value);
        Err(Raised::object_not_single_dimensional(&found).into())
    }

    /// `RexxInternalObject::requestArray`'s conversion, which is a message
    /// send on one path and a direct call on the other: `None` where a
    /// primitive has no `MAKEARRAY`, and otherwise what the send answers.
    pub(crate) fn request_array_value(&mut self, value: ObjRef) -> Result<Option<ObjRef>, Failure> {
        let caller = self.caller();
        if self.is_base_class(value) {
            if self.lookup(value, b"MAKEARRAY", None).is_none() {
                return Ok(None);
            }
            return pinned!(
                self,
                crate::pinning::PinKind::Conversion,
                self.send_message(value, b"MAKEARRAY", None, &[], caller)
            );
        }
        let wanted = self.text_built(b"ARRAY".to_vec());
        self.roots.activity_mut().push_temp(wanted);
        pinned!(
            self,
            crate::pinning::PinKind::Conversion,
            self.send_message(value, b"REQUEST", None, &[Some(wanted)], caller)
        )
    }

    /// Whether `value` is a `StringTable` -- `.methods`, `.routines`,
    /// `.resources`, or a package's `~publicClasses` -- or a `Directory` built
    /// on the same map, a package's `~local`.
    fn is_hash_collection(&mut self, value: ObjRef) -> bool {
        let Some(Body::Native(native)) = self.heap.get(value).map(|object| &object.body) else {
            return false;
        };
        let class = native.class();
        class == self.object_model().iterable_collection_class()
            || self.receiver_class_id(value).as_deref() == Some("Directory")
    }

    /// The values a `DO OVER` binds its control variable to: the **non-empty**
    /// slots of what [`Interp::over_target_array`] converts the target into.
    fn over_snapshot(&mut self, value: ObjRef) -> Result<(ObjRef, Vec<ObjRef>), Failure> {
        // The collections whose order is this crate's rather than the
        // oracle's, a native `StringTable` or `Directory`, answer from their
        // own walk -- see
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
                self.roots.activity_mut().push_temp(array);
            }
            match self.heap.get(array).map(|object| &object.body) {
                Some(Body::Array { slots, .. }) => slots.iter().flatten().copied().collect(),
                _ => vec![array],
            }
        };
        // Every item is already reachable here, which is what `alloc_with`
        // collecting before it allocates asks of this site.
        let snapshot = self.alloc_charged(
            BehaviourId::ARRAY,
            Body::array(items.iter().copied().map(Some).collect()),
        );
        self.roots.activity_mut().push_temp(snapshot);
        debug_assert_eq!(
            unshared!(
                self,
                self.array_slots(snapshot).map(|slots| slots
                    .iter()
                    .copied()
                    .flatten()
                    .collect::<Vec<_>>())
            )
            .as_deref(),
            Some(items.as_slice()),
            "a DO OVER's cursor is not the snapshot's own slots, so the snapshot does not root it"
        );
        Ok((snapshot, items))
    }

    /// A native `StringTable`'s or `Directory`'s indexes, as the values a `DO
    /// OVER` binds in turn.
    fn hash_collection_indexes(&mut self, table: ObjRef) -> Vec<ObjRef> {
        let Some(Body::Native(native)) = self.heap.get(table).map(|object| &object.body) else {
            return Vec::new();
        };
        let mut keys = native.keys();
        keys.sort_unstable();
        let mut items = Vec::with_capacity(keys.len());
        for key in &keys {
            let item = self.text(key);
            self.roots.activity_mut().push_temp(item);
            items.push(item);
        }
        items
    }

    /// One controlled-loop header value as the `Number` the loop runs on:
    /// numeric (41.1 if not) and rounded at the digits in force.
    /// `None` for an operator receiver, which the header sends `+` instead.
    fn header_number(&mut self, value: ObjRef) -> Result<Option<Number>, Failure> {
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
            return Ok(Some(shortcut));
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

    /// A controlled header value once the header holds an object:
    /// `ControlledLoop::setup`'s `callOperatorMethod(OPERATOR_PLUS)`, a message
    /// to an object (97.1 where it has no `+`), whose answer is kept as it is.
    /// `BY`'s answer is sent the setup's `<` against 0 for the direction.
    #[cold]
    #[inline(never)]
    fn accept_object_header(
        &mut self,
        role: HeaderRole,
        value: ObjRef,
        values: &mut LoopHeaderValues,
    ) -> Result<(), Failure> {
        let answer = self.apply_prefix(rexx_parse::PrefixOp::Plus, value)?;
        self.roots.activity_mut().push_temp(answer);
        let descending = if role == HeaderRole::By {
            let zero = self.counted(0);
            let below = self.apply_binary(rexx_parse::Operator::LessThan, answer, zero)?;
            self.truth(below, Raised::not_logical)?
        } else {
            false
        };
        let objects = values.objects.get_or_insert_with(Box::default);
        match role {
            HeaderRole::Initial => objects.initial = Some(answer),
            HeaderRole::To => objects.to = Some(answer),
            _ => {
                objects.by = Some(answer);
                objects.descending = descending;
            }
        }
        Ok(())
    }

    /// The loop state for a controlled header that holds an object; a value
    /// the header converted as a number stays one.
    fn object_control_state(
        &mut self,
        code: &Code<'_>,
        control: SymbolId,
        values: &LoopHeaderValues,
        objects: &ObjectHeader,
    ) -> LoopState {
        let digits = self.activation().settings.digits();
        let initial = match (objects.initial, &values.initial) {
            (Some(initial), _) => initial,
            (None, Some(initial)) => self.control_number_object(initial.number().as_ref(), digits),
            (None, None) => ObjRef::NIL,
        };
        let to = match (objects.to, &values.to) {
            (Some(to), _) => Some(to),
            (None, Some(to)) => Some(self.control_number_object(to, digits)),
            (None, None) => None,
        };
        let (by, descending) = match (objects.by, &values.by) {
            (Some(by), _) => (by, objects.descending),
            (None, Some(by)) => (self.control_number_object(by, digits), by.signum() < 0),
            (None, None) => (self.counted(1), false),
        };
        self.hold_object_control(Box::new(ObjectControl {
            control,
            at: control_slot(code, control),
            shape: shape_of(code.symbols.name(control).as_bytes()),
            initial: Some(initial),
            to,
            by,
            descending,
            for_remaining: values.for_remaining,
            stepped: false,
        }))
    }

    /// Roots `ctl`'s objects in [`Activity::loop_objects`] for the loop's
    /// life, which a pass boundary needs: it takes the loop out of
    /// `flat_top` while user code may run.
    ///
    /// [`Activity::loop_objects`]: crate::activity::Activity::loop_objects
    fn hold_object_control(&mut self, ctl: Box<ObjectControl>) -> LoopState {
        let key = object_key(&ctl);
        let held = &mut self.activity.loop_objects;
        held.extend(ctl.initial.map(|object| (key, object)));
        held.extend(ctl.to.map(|object| (key, object)));
        held.push((key, ctl.by));
        LoopState::ObjectControlled(ctl)
    }

    /// Drops what [`Interp::hold_object_control`] held for `flat`, once its
    /// loop is over.
    #[cold]
    #[inline(never)]
    pub(crate) fn release_loop_objects(&mut self, flat: &FlatLoop) {
        if self.activity.loop_objects.is_empty() {
            return;
        }
        if let Some(key) = flat.object_key() {
            self.activity.loop_objects.retain(|(held, _)| *held != key);
        }
    }

    /// `number` as the object a header's `+` answers, rooted as a temp
    /// until [`Interp::hold_object_control`] holds it.
    fn control_number_object(&mut self, number: &Number, digits: u64) -> ObjRef {
        let form = self.activation().settings.form();
        let object = self.number(number.clone(), crate::eval::saturate_digits(digits), form);
        self.roots.activity_mut().push_temp(object);
        object
    }

    /// [`Interp::header_number`]'s own computation for a position that
    /// reaches it (`Initial`, `To` and `By` -- see that function's own
    /// doc), wrapped by it so every failing step is blamed once.
    fn header_number_body(
        &mut self,
        value: ObjRef,
        entry_digits: u64,
    ) -> Result<Option<Number>, Failure> {
        // An operator receiver never converts, so it is asked for only once
        // the conversion has failed.
        let operand = match self.to_number(value) {
            Ok(operand) => operand,
            Err(_) if self.operator_message_receiver(value).is_some() => return Ok(None),
            Err(_) => self.arith_operand(value)?,
        };
        // The header rounds through a unary `+`, so its three numeric
        // positions carry an operand exactly as an operator's do -- measured,
        // `do k = 1 to 123456789` and `... by 123456789` are both 98.972 at
        // DIGITS 3.
        self.lostdigits_check(&operand, value)?;
        Ok(Some(
            round_via_unary_plus(&operand, entry_digits).map_err(Raised::from)?,
        ))
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
                let do_indent = self.activity.clause_state.current_value_indent;
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
                    .unwrap_or_else(|| self.activity.clause_state.line());
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
                            .unwrap_or_else(|| self.activity.clause_state.line());
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
            // `flat_loop_start` answers every kind but a labelled block itself,
            // and this function is reached only from its `Fallback`, so a loop
            // driven by message sends never gets here to be held and left
            // unreleased.
            LoopKind::Controlled(_) if values.objects.is_some() => {
                unreachable!("only a labelled block falls back to the nested path")
            }
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
            LoopKind::With {
                index: with_index,
                item,
                ..
            } => self.with_state(*with_index, *item, &values),
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
            LoopCounter::of(code, body),
            engine,
        )
    }

    /// A `DO WITH`'s state, from the supplier its header filed.
    fn with_state(
        &self,
        index: Option<SymbolId>,
        item: Option<SymbolId>,
        values: &LoopHeaderValues,
    ) -> LoopState {
        LoopState::With(Box::new(WithState {
            index,
            item,
            supplier: values
                .supplier
                .expect("a DO WITH's plan always names its target"),
            first: true,
            remaining: values.for_remaining,
        }))
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
    pub(super) fn settle_block_indent(&mut self, open: bool, clause_indent: usize) {
        self.activity.clause_state.current_value_indent = if open {
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
        mut counter: Option<LoopCounter>,
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
        let do_indent = self.activity.clause_state.current_value_indent;
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
                .unwrap_or_else(|| self.activity.clause_state.line());
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
                        it.activity.clause_state.current_value_indent = loop_indent;
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
                    if let Some(counter) = counter.as_mut() {
                        it.count_pass(code, counter, loop_indent)?;
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
            // The pass boundary is an instruction of its own on the oracle
            // (`END`, or the `ITERATE` that jumped to it), and the clock is
            // stale after it (`RexxActivation.cpp:647`), so a loop with no
            // body clause still reads a fresh `TIME('E')` per test.
            self.activation_mut().clock_stale = true;
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
                self.activity.clause_state.current_value_indent = loop_indent;
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

    /// Sets a repeating loop up to be driven from
    /// the op driver's own frame instead of a nested `run_ops` entry, or
    /// declines and leaves the caller to take the nested path.
    #[allow(
        clippy::too_many_arguments,
        reason = "the same argument `run_loop_with_header`'s own allow makes: every parameter is state one DO/LOOP needs"
    )]
    pub(crate) fn flat_loop_start(
        &mut self,
        code: &Code<'_>,
        index: usize,
        instruction: &Instruction,
        body: &Loop,
        source: Option<&ProgramSource>,
        values: LoopHeaderValues,
        op_body: u32,
        registers: RegFrame<'_>,
    ) -> Result<FlatStart, Failure> {
        // A labelled block is left by a `LEAVE` naming it, which needs a
        // frame to absorb it, so it keeps the nested path.
        if matches!(body.kind, LoopKind::Simple) && body.label.is_none() {
            return self.flat_block_start(code, index, instruction, source);
        }
        let over_register = values.over_register;
        let state = match &body.kind {
            LoopKind::Forever => LoopState::Forever,
            LoopKind::Count(_) => LoopState::Count {
                remaining: values.count.unwrap_or(1),
            },
            LoopKind::Controlled(ctrl) if values.objects.is_some() => {
                let objects = values.objects.as_deref().expect("tested by the guard");
                self.object_control_state(code, ctrl.control, &values, objects)
            }
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
            LoopKind::With {
                index: with_index,
                item,
                ..
            } => self.with_state(*with_index, *item, &values),
            // A labelled block.
            LoopKind::Simple => return Ok(FlatStart::Fallback(values)),
        };
        // **The snapshot's only root once this answers `Flat`**: the driver
        // then closes this clause and pops the temps frame `over_snapshot`
        // pushed into, while the loop runs on. A loop header's registers are
        // allocated in the enclosing scope and released past the whole loop
        // (`ir/compile.rs`), and nothing reads the target again, so its
        // register holds the snapshot instead.
        if let Some(register) = over_register {
            match &state {
                LoopState::OverItems { snapshot, .. } => registers.set(register, *snapshot),
                LoopState::With(with) => registers.set(register, with.supplier),
                _ => {}
            }
        }
        let end_index = body
            .end
            .expect("an unclosed DO/LOOP is error 14.1/14.5, so a body that parsed has this set");
        let do_indent = self.activity.clause_state.current_value_indent;
        let end_line = self
            .clause_line_at(code, end_index, &code.body.instructions[end_index], source)
            .unwrap_or(0);
        let do_line = self
            .clause_line_at(code, index, instruction, source)
            .unwrap_or_else(|| self.activity.clause_state.line());
        // **Built where it will live, not on the stack and then moved
        // there.** A `FlatLoop` holds the `LoopState` a controlled loop's
        // three `Number`s live in, and the version that built one here and assigned it into the box afterwards wrote those
        // bytes twice per loop entry -- which for a nested loop is twice per
        // iteration of the loop above it. Measured on `do n = 1 to N ; do j =
        // 1 to 1 ; end ; end`: -5.871% retired instructions, with
        // `__memmove_avx_unaligned_erms` falling from 8.41% of the program's
        // cycles to 2.52%.
        let counted = self.count_clause_against_deadline(false)?;
        let mut boxed = match self.activity.flat_spares.pop() {
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
            test: PassTest::of(body),
            counter: LoopCounter::of(code, body),
            state,
        };
        let header = match self.flat_loop_header(code, source, &mut boxed, counted) {
            Ok(header) => header,
            Err(failure) => {
                self.release_loop_objects(&boxed);
                self.activity.flat_spares.push(boxed);
                return Err(failure);
            }
        };
        match header {
            Some(flow) => {
                // The header ended the loop, so nothing will drive it and
                // this box is spare again rather than leaked back to the
                // allocator.
                self.release_loop_objects(&boxed);
                self.activity.flat_spares.push(boxed);
                Ok(FlatStart::Ended(flow))
            }
            None => {
                let range = (boxed.body_start, boxed.end_index);
                // The loop just entered becomes the innermost, and whatever
                // was innermost joins the ones enclosing it.
                if let Some(enclosing) = self.activity.flat_top.replace(boxed) {
                    self.activity.flat_loops.push(enclosing);
                }
                Ok(FlatStart::Flat {
                    body_start: range.0,
                    end_index: range.1,
                })
            }
        }
    }

    /// The block's own clause, run inside the `DO`'s as `run_loop_with_header`'s
    /// `Simple` arm runs it, so a condition queued before the `DO` is
    /// delivered with the block open.
    fn flat_block_start(
        &mut self,
        code: &Code<'_>,
        index: usize,
        instruction: &Instruction,
        source: Option<&ProgramSource>,
    ) -> Result<FlatStart, Failure> {
        let do_indent = self.activity.clause_state.current_value_indent;
        let do_line = self
            .clause_line_at(code, index, instruction, source)
            .unwrap_or_else(|| self.activity.clause_state.line());
        match self.in_clause(code, do_line, |it| {
            it.settle_block_indent(true, do_indent);
            Ok(())
        })? {
            ClauseOutcome::Ended(exit) => Ok(FlatStart::Ended(Flow::Exit(exit.value()))),
            ClauseOutcome::Ran(ran) => ran.map(|()| FlatStart::Block),
        }
    }

    /// One pass boundary of the innermost flat loop: the state is
    /// taken out of `Activity::flat_top` so that this can hold a `&mut Interp`
    /// beside it, and put back when another pass follows.
    pub(crate) fn flat_loop_step_top(
        &mut self,
        code: &Code<'_>,
        source: Option<&ProgramSource>,
        arrival: Flow,
        counted: impl FnOnce(&mut Self) -> Result<DeadlineCounted, Failure>,
    ) -> Result<FlatStep, Failure> {
        let Some(mut top) = self.activity.flat_top.take() else {
            return Err(Loud::op_not_driven("a pass boundary with no loop open").into());
        };
        match self.flat_loop_step(code, source, &mut top, arrival, counted) {
            Ok(FlatStep::Body(op_body)) => {
                self.activity.flat_top = Some(top);
                Ok(FlatStep::Body(op_body))
            }
            Ok(FlatStep::Done(flow)) => {
                self.release_loop_objects(&top);
                self.activity.flat_spares.push(top);
                // The loop that just ended uncovers the one enclosing it, and
                // `None` here is the outermost of a nest having ended.
                self.activity.flat_top = self.activity.flat_loops.pop();
                Ok(FlatStep::Done(flow))
            }
            // `top` is dropped rather than handed back, and this loop's own
            // frame is still standing: `unwind_frames` finds `flat_top`
            // already empty and takes the enclosing box for that frame, so one
            // box reaches the allocator instead of `flat_spares` and no state
            // is lost. Its own `if let Some` is what tolerates the last frame
            // finding nothing left, which is the same tolerance the pop off
            // `flat_loops` needed before this field existed.
            Err(failure) => {
                self.release_loop_objects(&top);
                Err(failure)
            }
        }
    }

    /// Ends the innermost flat loop away from a pass boundary: its objects
    /// released, its box spare, and the loop enclosing it uncovered.
    pub(crate) fn close_flat_top(&mut self) {
        if let Some(top) = self.activity.flat_top.take() {
            self.release_loop_objects(&top);
            self.activity.flat_spares.push(top);
        }
        self.activity.flat_top = self.activity.flat_loops.pop();
    }

    /// One pass boundary: what the body just answered, then the
    /// next pass's header test, whose first clause `counted` counts.
    #[inline(always)]
    fn flat_loop_step(
        &mut self,
        code: &Code<'_>,
        source: Option<&ProgramSource>,
        flat: &mut FlatLoop,
        arrival: Flow,
        counted: impl FnOnce(&mut Self) -> Result<DeadlineCounted, Failure>,
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
        // The clock is stale after the pass boundary, as `run_repeating`'s
        // own store says.
        self.activation_mut().clock_stale = true;
        // **`UNTIL`'s own test, and its own re-echo of the `DO`/`LOOP`
        // clause.** `run_repeating`'s own arm has the measurement: the oracle
        // re-enters the loop instruction to make this decision as much as to
        // test `WHILE` or advance a control variable, so the echo here is
        // unconditional rather than sharing the top-of-loop one below -- which
        // is why that one is not emitted at all for an `UNTIL` loop.
        if flat.test.until() {
            if echoing
                && let Some((line, text)) =
                    self.clause_site(source, &code.body.instructions[flat.do_index])
            {
                self.trace_clause(line, flat.do_indent, &text);
            }
            let counted = counted(self)?;
            if let Some(flow) = self.flat_loop_until(code, source, flat, counted)? {
                return Ok(FlatStep::Done(flow));
            }
            let counted = self.count_clause_against_deadline(false)?;
            return match self.flat_loop_header(code, source, flat, counted)? {
                Some(flow) => Ok(FlatStep::Done(flow)),
                None => Ok(FlatStep::Body(flat.op_body)),
            };
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
        let counted = counted(self)?;
        match self.flat_loop_header(code, source, flat, counted)? {
            Some(flow) => Ok(FlatStep::Done(flow)),
            None => Ok(FlatStep::Body(flat.op_body)),
        }
    }

    /// An `UNTIL` loop's own bottom-of-pass test: `Some(flow)` is
    /// the loop finishing, `None` is carrying on to the header.
    fn flat_loop_until(
        &mut self,
        code: &Code<'_>,
        source: Option<&ProgramSource>,
        flat: &FlatLoop,
        counted: DeadlineCounted,
    ) -> Result<Option<Flow>, Failure> {
        let Some(cond) = loop_conditional_of(code, flat.do_index) else {
            return Err(Loud::instruction(&code.body.instructions[flat.do_index].kind).into());
        };
        let until_line = flat.header_line();
        let (do_indent, loop_indent, resume) = (flat.do_indent, flat.loop_indent, flat.resume);
        // The re-echoed `END` clause just above leaves `current_value_indent`
        // untouched -- `trace_clause` does not set it -- so without this the
        // `UNTIL`'s intermediates would read the `DO`'s own indent.
        self.activity.clause_state.current_value_indent = loop_indent;
        let tested = self.in_counted_clause(code, until_line, counted, |it| {
            let held = pinned!(
                it,
                crate::pinning::PinKind::LoopHeader,
                it.eval_condition(
                    code,
                    &cond.condition,
                    ConditionTrace::Keyword(loop_indent, "UNTIL"),
                    raised_until_not_logical,
                )
            )?;
            // An `UNTIL` that held ends the loop, so this clause's own
            // boundary sits outside the block.
            it.settle_block_indent(!held, do_indent);
            Ok(held)
        });
        let tested = match tested {
            Ok(tested) => tested,
            // The boundary's own failure, a requested `HALT`, is the `END`'s
            // as the condition's is.
            Err(failure) => {
                let end = &code.body.instructions[flat.end_index];
                self.record_failure_at(source, end, loop_indent);
                return Err(failure);
            }
        };
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

    /// Who a failing header re-test is blamed on: the clause that
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

    /// A `WHILE` that failed, blamed on the `DO`/`LOOP` clause at
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

    /// The header re-test, in its own clause: `Some(flow)` is the
    /// loop finishing, `None` is one more pass.
    #[inline(always)]
    fn flat_loop_header(
        &mut self,
        code: &Code<'_>,
        source: Option<&ProgramSource>,
        flat: &mut FlatLoop,
        counted: DeadlineCounted,
    ) -> Result<Option<Flow>, Failure> {
        if flat.test.general() {
            return self.flat_loop_header_general(code, source, flat, counted);
        }
        let header_line = flat.header_line();
        let do_indent = flat.do_indent;
        let loop_indent = flat.loop_indent;
        let resume = flat.resume;
        let end_index = flat.end_index;
        let blame = flat.header_clause;
        let site = &flat.iterate_site;
        let state = &mut flat.state;
        // `in_counted_clause` spelled out rather than given a closure: the
        // closure's inlining into the driver is a codegen decision that
        // unrelated changes to the flat path flip (+18.6% on `emptyloop`).
        let entry = self.enter_clause(header_line, counted);
        let ran = match self.loop_advance(code, state, do_indent, loop_indent) {
            Ok(advanced) => {
                self.settle_block_indent(advanced, do_indent);
                Ok(advanced)
            }
            Err(failure) => {
                if by_message_first_pass(blame, state) {
                    self.blame_while_failure(code, source, flat.do_index, loop_indent);
                }
                self.blame_header_failure(code, source, blame, site, end_index, loop_indent);
                Err(failure)
            }
        };
        let header = self.leave_clause(entry, code, ran);
        let header = match header {
            Ok(header) => header,
            // The boundary's own failure, a requested `HALT`, is blamed as the
            // header's is.
            Err(failure) => {
                self.blame_header_failure(code, source, blame, site, end_index, loop_indent);
                return Err(failure);
            }
        };
        flat_header_outcome(header, resume)
    }

    /// [`Interp::flat_loop_header`] for a loop that carries a `WHILE` or a
    /// `COUNTER`: the same advance, then the condition, then the counter, all
    /// inside the one clause the oracle re-enters to make this decision.
    #[inline(never)]
    fn flat_loop_header_general(
        &mut self,
        code: &Code<'_>,
        source: Option<&ProgramSource>,
        flat: &mut FlatLoop,
        counted: DeadlineCounted,
    ) -> Result<Option<Flow>, Failure> {
        let cond = loop_conditional_of(code, flat.do_index).filter(|cond| !cond.until);
        let header_line = flat.header_line();
        let do_indent = flat.do_indent;
        let loop_indent = flat.loop_indent;
        let resume = flat.resume;
        let (do_index, end_index) = (flat.do_index, flat.end_index);
        let blame = flat.header_clause;
        let site = &flat.iterate_site;
        let state = &mut flat.state;
        let counter = &mut flat.counter;
        let header = self.in_counted_clause(code, header_line, counted, |it| {
            let advanced = match it.loop_advance(code, state, do_indent, loop_indent) {
                Ok(advanced) => advanced,
                Err(failure) => {
                    if by_message_first_pass(blame, state) {
                        it.blame_while_failure(code, source, do_index, loop_indent);
                    }
                    it.blame_header_failure(code, source, blame, site, end_index, loop_indent);
                    return Err(failure);
                }
            };
            if !advanced {
                it.settle_block_indent(false, do_indent);
                return Ok(false);
            }
            if let Some(cond) = cond {
                // Overrides what stepping the `DO`/`LOOP` instruction set:
                // `WHILE`'s condition is evaluated here, inside that same step,
                // never through a `Op::Clause`'s region of its own.
                it.activity.clause_state.current_value_indent = loop_indent;
                let held = match pinned!(
                    it,
                    crate::pinning::PinKind::LoopHeader,
                    it.eval_condition(
                        code,
                        &cond.condition,
                        ConditionTrace::Keyword(loop_indent, "WHILE"),
                        raised_while_not_logical,
                    )
                ) {
                    Ok(held) => held,
                    Err(failure) => {
                        it.blame_while_failure(code, source, do_index, loop_indent);
                        return Err(failure);
                    }
                };
                if !held {
                    it.settle_block_indent(false, do_indent);
                    return Ok(false);
                }
            }
            if let Some(counter) = counter.as_mut() {
                it.count_pass(code, counter, loop_indent)?;
            }
            it.settle_block_indent(true, do_indent);
            Ok(true)
        });
        let header = match header {
            Ok(header) => header,
            // The boundary's own failure, a requested `HALT`, is blamed as the
            // header's is.
            Err(failure) => {
                self.blame_header_failure(code, source, blame, site, end_index, loop_indent);
                return Err(failure);
            }
        };
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
            LoopState::With(with) => self.with_advance(code, with, loop_indent),
            LoopState::ObjectControlled(ctl) => {
                self.object_control_advance(code, ctl, do_indent, loop_indent)
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
                            && let Some(sum) = self.controlled_step_wide(previous, by, digits)?
                        {
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
                let pass = self.roots.activity_mut().push_frame();
                let mut by_message = None;
                if re_tested {
                    let previous =
                        self.read_control_traced(code, *control, *at, shape, loop_indent)?;
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
                        // An object in the control variable: the loop goes on
                        // by message from here.
                        None => match self.controlled_step_wide(previous, by, digits)? {
                            Some(sum) => *current = ControlValue::Wide(sum),
                            None => by_message = Some(previous),
                        },
                    }
                }
                if let Some(previous) = by_message {
                    *state = self.object_control_from_numbers(
                        (*control, *at, shape),
                        to.as_ref(),
                        by,
                        *for_remaining,
                        digits,
                    );
                    let LoopState::ObjectControlled(ctl) = state else {
                        unreachable!("the state was written on the line above");
                    };
                    let more = self.object_control_pass(
                        code,
                        ctl,
                        Some(previous),
                        do_indent,
                        loop_indent,
                    )?;
                    self.roots.activity_mut().pop_frame(pass);
                    return Ok(more);
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
                self.roots.activity_mut().push_temp(value);
                let bind_indent = if re_tested { loop_indent } else { do_indent };
                if re_tested && let Some(rendered) = self.result_text(value) {
                    self.trace_result(loop_indent, &rendered);
                }
                self.bind_control(code, *control, bind_indent, value, *at, shape)?;
                // The control variable's own storage now roots `value`, and
                // `previous` is dead, so the pass's frame goes here. The three
                // `return Ok(false)` paths below end the loop, whose enclosing
                // frame truncates past this one anyway.
                self.roots.activity_mut().pop_frame(pass);

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

    /// `DoBlock::checkControl`'s read of the control variable at a re-test:
    /// an evaluation, so it can raise `NOVALUE`, traced `>C>`, `>V>` and
    /// `>>>`, and rooted.
    #[inline(always)]
    fn read_control_traced(
        &mut self,
        code: &Code<'_>,
        control: SymbolId,
        at: Option<usize>,
        shape: NameShape,
        loop_indent: usize,
    ) -> Result<ObjRef, Failure> {
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
                let (value, novalue) = self.read_at(code, control, at);
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
                let at = code.compound(control).and_then(|entry| entry.stem_at);
                let name = code.symbols.name(control).as_bytes();
                (self.read_stem_at(name, at), Novalue::Set, None)
            }
            NameShape::Compound => {
                let (stem_name, stem_at) = code.stem(control);
                let key = self.tail_key(code, control)?;
                let (value, novalue) = self.stem_get_at(stem_name, stem_at, &key);
                let mut resolved = stem_name.to_vec();
                resolved.extend_from_slice(&key);
                (value, novalue, Some(resolved))
            }
        };
        self.novalue_check(novalue, previous)?;
        self.roots.activity_mut().push_temp(previous);
        // `>C>` before `>V>`, both self-gated on `intermediates`
        // like every other value-bearing prefix -- `stem_get`'s
        // own read announces the fully-resolved name it used
        // before either of the value lines shows what is stored
        // there, the same order `eval_node`'s `Compound` arm and
        // its own tracing counterpart use for an ordinary read.
        if let Some(resolved) = &resolved {
            let name = code.symbols.name(control).as_bytes();
            self.trace_compound_name(loop_indent, name, resolved);
        }
        // `result_text` for the pair, not `intermediate_text`:
        // `>V>` is `intermediates` and `>>>` is `results`, and
        // `results` is the weaker of the two, so it renders for
        // either and drops neither.
        if let Some(rendered) = self.result_text(previous) {
            let name = code.symbols.name(control).as_bytes();
            self.trace_variable(loop_indent, name, &rendered);
            self.trace_result(loop_indent, &rendered);
        }
        Ok(previous)
    }

    /// One controlled pass's step where the control variable is not an
    /// integer the tag holds, or the sum leaves what `DIGITS` admits; `None`
    /// where it holds an operator receiver, which goes on by message.
    #[inline(never)]
    fn controlled_step_wide(
        &mut self,
        previous: ObjRef,
        by: &Number,
        digits: u64,
    ) -> Result<Option<Number>, Failure> {
        let read = match self.to_number(previous) {
            Ok(read) => read,
            Err(_) if self.operator_message_receiver(previous).is_some() => return Ok(None),
            Err(_) => self.arith_operand(previous)?,
        };
        read.add(by, digits)
            .map(Some)
            .map_err(Raised::from)
            .map_err(Failure::from)
    }

    /// [`Interp::object_control_pass`] with its own temps frame and, at a
    /// re-test, the control variable's read.
    #[cold]
    #[inline(never)]
    fn object_control_advance(
        &mut self,
        code: &Code<'_>,
        ctl: &mut ObjectControl,
        do_indent: usize,
        loop_indent: usize,
    ) -> Result<bool, Failure> {
        let pass = self.roots.activity_mut().push_frame();
        let previous = if ctl.stepped {
            Some(self.read_control_traced(code, ctl.control, ctl.at, ctl.shape, loop_indent)?)
        } else {
            None
        };
        let more = self.object_control_pass(code, ctl, previous, do_indent, loop_indent)?;
        self.roots.activity_mut().pop_frame(pass);
        Ok(more)
    }

    /// A numeric controlled loop's state carried over once its control
    /// variable holds an object: `TO` and `BY` become the objects their
    /// numbers are, once for the rest of the loop, and the direction comes from
    /// `BY`'s sign.
    #[cold]
    #[inline(never)]
    fn object_control_from_numbers(
        &mut self,
        (control, at, shape): (SymbolId, Option<usize>, NameShape),
        to: Option<&Number>,
        by: &Number,
        for_remaining: Option<u64>,
        digits: u64,
    ) -> LoopState {
        let to = to.map(|to| self.control_number_object(to, digits));
        let descending = by.signum() < 0;
        let by = self.control_number_object(by, digits);
        self.hold_object_control(Box::new(ObjectControl {
            control,
            at,
            shape,
            initial: None,
            to,
            by,
            descending,
            for_remaining,
            stepped: true,
        }))
    }

    /// One pass of an [`ObjectControl`] loop, `DoBlock::checkControl`: at a
    /// re-test `+` is sent to `previous` with `BY` and the answer traced; the
    /// value is bound as it is, then sent the `TO` comparison, and the loop
    /// ends where that answers true. An object without the method is 97.1.
    #[cold]
    #[inline(never)]
    fn object_control_pass(
        &mut self,
        code: &Code<'_>,
        ctl: &mut ObjectControl,
        previous: Option<ObjRef>,
        do_indent: usize,
        loop_indent: usize,
    ) -> Result<bool, Failure> {
        let (value, bind_indent) = match previous {
            Some(previous) => {
                let sum = self.arith_general(rexx_parse::Operator::Plus, previous, ctl.by)?;
                self.roots.activity_mut().push_temp(sum);
                if let Some(rendered) = self.result_text(sum) {
                    self.trace_result(loop_indent, &rendered);
                }
                (sum, loop_indent)
            }
            None => {
                let initial = ctl.initial.take().unwrap_or(ObjRef::NIL);
                self.roots.activity_mut().push_temp(initial);
                (initial, do_indent)
            }
        };
        ctl.stepped = true;
        self.bind_control(code, ctl.control, bind_indent, value, ctl.at, ctl.shape)?;
        if let Some(to) = ctl.to {
            let compare = if ctl.descending {
                rexx_parse::Operator::LessThan
            } else {
                rexx_parse::Operator::GreaterThan
            };
            let past = self.apply_binary(compare, value, to)?;
            if self.truth(past, Raised::not_logical)? {
                return Ok(false);
            }
        }
        if let Some(remaining) = &mut ctl.for_remaining {
            if *remaining == 0 {
                return Ok(false);
            }
            *remaining -= 1;
        }
        Ok(true)
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

    /// `WithLoop::checkIteration` and then the `FOR` budget: steps the
    /// supplier past the first pass, binds `INDEX` and then `ITEM`, and
    /// consults `FOR` only after both are bound.
    #[inline(never)]
    fn with_advance(
        &mut self,
        code: &Code<'_>,
        with: &mut WithState,
        loop_indent: usize,
    ) -> Result<bool, Failure> {
        let supplier = with.supplier;
        // A pass's own frame, as `count_pass`'s: the bound variables root
        // INDEX and ITEM once written. The `?` paths leave it to the outer
        // truncation.
        let pass = self.roots.activity_mut().push_frame();
        if !with.first {
            self.supplier_send(supplier, b"NEXT")?;
        }
        with.first = false;
        let available = self.supplier_send(supplier, b"AVAILABLE")?;
        self.roots.activity_mut().push_temp(available);
        if !self.truth(available, raised_available_not_logical)? {
            self.roots.activity_mut().pop_frame(pass);
            return Ok(false);
        }
        for (name, message) in [(with.index, &b"INDEX"[..]), (with.item, &b"ITEM"[..])] {
            if let Some(name) = name {
                let value = self.supplier_send(supplier, message)?;
                self.roots.activity_mut().push_temp(value);
                self.bind_control(
                    code,
                    name,
                    loop_indent,
                    value,
                    control_slot(code, name),
                    shape_of(code.symbols.name(name).as_bytes()),
                )?;
            }
        }
        self.roots.activity_mut().pop_frame(pass);
        if let Some(remaining) = with.remaining.as_mut() {
            if *remaining == 0 {
                return Ok(false);
            }
            *remaining -= 1;
        }
        Ok(true)
    }

    /// `message` sent to a `DO WITH`'s supplier, `.nil` for no result.
    fn supplier_send(&mut self, supplier: ObjRef, message: &[u8]) -> Result<ObjRef, Failure> {
        let caller = self.caller();
        let answered = pinned!(
            self,
            crate::pinning::PinKind::LoopHeader,
            self.send_message(supplier, message, None, &[], caller)
        )?;
        Ok(answered.unwrap_or(ObjRef::NIL))
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

/// Whether `a < b`, numerically, through `rexx-num`'s own `compare_decoded`
/// rather than a hand-rolled sign comparison -- this crate's standing rule
/// against a second copy of a comparison `rexx-num` already owns
/// (`compare_values`' own doc comment states it for the expression
/// operators; a controlled loop's own bound test and its `BY`'s sign are
/// the same rule applied to two `Number`s this crate already holds, not a
/// different one).
pub(super) fn numeric_less(
    a: &Number,
    b: &Number,
    digits: u64,
    fuzz: u64,
) -> Result<bool, ArithError> {
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

/// Whether a failed advance is a message-driven loop's first test, which the
/// oracle makes inside the block it has just opened: blamed on the `DO`
/// clause at the body's indent, as a `WHILE` is.
fn by_message_first_pass(blame: HeaderClause, state: &LoopState) -> bool {
    matches!(blame, HeaderClause::Do) && matches!(state, LoopState::ObjectControlled(_))
}
