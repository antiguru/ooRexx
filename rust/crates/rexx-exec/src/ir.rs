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

//! The register-based instruction stream (Phase 4e): `Op`, `Chunk`, and the
//! shapes later tasks extend rather than reshape.

use std::cell::Cell;

use rexx_core::{FrameId, ObjRef};
use rexx_parse::{Operator, PrefixOp, SymbolId};

use crate::error::Raised;
use crate::eval::SymbolRead;
use crate::run::{
    HeaderRole, QueueKeyword, Resolved, ReturnKeyword, raised_if_not_logical,
    raised_when_not_logical,
};
use crate::trace::ChunkTrace;

mod compile;
pub(crate) mod drive;
pub(crate) use compile::compile;
pub(crate) use golden::render_annotated;

// `render` serialises an op stream back to text. Nothing the interpreter does
// at run time reads a chunk that way; its callers are the golden tests and
// `crate::render_ir`, which the `rexx-ir` binary prints.
mod golden;

#[cfg(test)]
mod golden_tests;

// The corpus-wide statement of the same promotion set `golden_tests` pins per
// construct: an invariant derived from each body's own instructions rather
// than a committed stream, so it holds over every corpus program without
// anything to regenerate. Its module doc has what it cannot see.
#[cfg(test)]
mod corpus_shape_tests;

/// **The stream's width, asserted so that a change to it is a compile error.**
/// Every op in every chunk is as wide as the widest variant, so the width is a
/// fact about the whole array rather than about the variant that sets it, and a
/// variant that outgrows this number grows every op there is. That is what this
/// line catches, at the moment it happens, rather than leaving it to a
/// measurement somebody has to take again. The widest payload has tail padding
/// for the discriminant to sit in, which is why sixteen is an equality and not
/// a bound.
const _: () = assert!(size_of::<Op>() == 16);

impl Op {
    /// The `src` an [`Op::PushArg`] or [`Op::TraceArgument`] carries for an
    /// omitted argument position.
    pub(crate) const ARG_OMITTED: u16 = u16::MAX;
}

/// One step in a compiled stream.
pub(crate) enum Op {
    /// Opens the promoted clause of the instruction at `index`. `end` is the
    /// op index one past this clause's last op -- the mark the register
    /// allocator releases to when the clause finishes (the plan's Decisions
    /// section: "a promoted clause takes a mark when its `Clause` op is
    /// emitted and releases to it at `end`").
    Clause { index: u32, end: u32 },
    /// Echoes the `>K>` line of one `DO`/`LOOP` header value, from register
    /// `src`, under the tag [`HeaderRole`] gives it.
    TraceKeyword { role: HeaderRole, src: u16 },
    /// Validates the `DO`/`LOOP` header value in register `src` for the role
    /// it plays and files it for [`Op::LoopRun`].
    LoopHeaderValue { role: HeaderRole, src: u16 },
    /// Runs the `DO`/`LOOP` at `index` from the header values the ops before it
    /// filed, with **its body's clauses stepped from this chunk**.
    LoopRun { index: u32 },
    /// **SPIKE, not for commit.** The bottom of one pass of the flattened
    /// `DO`/`LOOP` at `index`: what the body just did, then the next pass's
    /// header test, then either back to the body or on past the `END`.
    LoopNext { index: u32 },
    /// Echoes the `*-*` line of the clause of the instruction at `index`.
    TraceClause { index: u32 },
    /// Evaluates expression `slot` of the instruction at `index` into
    /// register `dst`, still dispatched through `eval.rs` rather than
    /// reimplemented here (the plan's Decisions section: trace ops land
    /// before the first *native* expression op, so this stays
    /// trace-identical to the `eval.rs` call it wraps).
    EvalExpr { index: u32, slot: u32, dst: u16 },
    /// Runs the call at `path` inside expression `slot` of the instruction at
    /// `index`, into register `dst`, through the resolution site `site` keeps.
    CallExpr {
        index: u32,
        slot: u16,
        path: NodePath,
        site: u16,
        dst: u16,
    },
    /// Appends the value in `src` to the driver's argument stack, or an
    /// omitted position when `src` is [`Op::ARG_OMITTED`].
    PushArg { src: u16 },
    /// The `>A>` line one argument owes, for the value now in `src`, or the
    /// **empty** value line an omitted position owes when `src` is
    /// [`Op::ARG_OMITTED`] -- which is not the absence of a line
    /// (`RexxInstruction.cpp:161`).
    TraceArgument { src: u16 },
    /// A call over the `argc` arguments now on top of the driver's argument
    /// stack.
    CallArgs {
        slot: u16,
        path: NodePath,
        site: u16,
        argc: u16,
        dst: u16,
    },
    /// The `>F>` line [`Op::CallExpr`] owes, for the value now in `src`.
    TraceFunction {
        index: u32,
        slot: u16,
        path: NodePath,
        src: u16,
    },
    /// Hands the `SELECT` at `index` the text an **absorbed** `WHEN CASE`
    /// compares against, from register `case`, or clears it for a plain
    /// `SELECT` that has no `CASE` expression at all.
    SelectCaseText { index: u32, case: Option<u16> },
    /// Tests the listed `WHEN`/`WHEN CASE` at `index` and stores whether it
    /// holds in register `dst`, as the Rexx logical value [`Op::JumpUnless`]
    /// reads back.
    WhenTest {
        index: u32,
        case: Option<u16>,
        dst: u16,
    },
    /// Closes the branch control has just run off the end of: the clause
    /// boundary a promoted construct owes where wrapper
    /// around the whole arm runs one.
    EndBranch,
    /// Closes the [`Op::EnterWhen`]/[`Op::EnterOtherwise`] frame whose branch
    /// ends here, if one is open.
    EndWhen,
    /// Opens a frame over the branch of the listed `WHEN` at `when`, which
    /// belongs to the `SELECT` at `select`.
    EnterWhen { select: u32, when: u32 },
    /// Opens a frame over the `OTHERWISE` branch of the `SELECT` at `select`,
    /// which is [`Op::EnterWhen`]'s counterpart for the branch no `WHEN`
    /// owns.
    EnterOtherwise { select: u32 },
    /// Loads constant `konst` of [`Chunk::consts`] into register `dst`.
    Const { dst: u16, konst: u32 },
    /// Loads the value of the constant symbol `symbol` into register `dst`:
    /// its own upcased spelling, which is observable rather than incidental --
    /// `say 1e5` prints `1E5`.
    LoadConstant { symbol: SymbolId, dst: u16 },
    /// Echoes the `>L>` line of the literal or constant symbol in register
    /// `src`.
    TraceLiteral { src: u16 },
    /// Loads the value of the bare symbol `symbol` into register `dst`, by the
    /// read [`SymbolRead`] names.
    Load {
        symbol: SymbolId,
        read: SymbolRead,
        at: PlanSlot,
        dst: u16,
    },
    /// Echoes the `>V>` line of the read in register `src`, and the `>C>` line
    /// a compound read owes in front of it.
    TraceRead {
        symbol: SymbolId,
        read: SymbolRead,
        src: u16,
    },
    /// Computes `lhs op rhs` into register `dst`, for the operators
    /// `eval::is_arithmetic` names.
    Arith {
        op: Operator,
        /// This site's own slot in [`Chunk::hints`] -- **not** a position in
        /// the op stream. The table is dense over the ops that can specialise,
        /// keyed off the op the way [`Chunk::consts`] is, so a stream carries
        /// no entry for the ops that never quicken and the driver's own loop
        /// needs no counter walking beside it to find one.
        hint: u32,
        lhs: u16,
        rhs: u16,
        dst: u16,
    },
    /// Computes `lhs op rhs` into register `dst`, for the concatenation,
    /// comparison and logical operators -- every operator `eval::
    /// is_native_binary` names and `eval::is_arithmetic` does not.
    Binary {
        op: Operator,
        lhs: u16,
        rhs: u16,
        dst: u16,
    },
    /// Echoes the `>O>` line of the operator result in register `src`.
    TraceOperator { op: Operator, src: u16 },
    /// Computes `op src` into register `dst`, for the prefix operators `+`,
    /// `-` and `\`.
    Prefix { op: PrefixOp, src: u16, dst: u16 },
    /// Echoes the `>P>` line of the prefix-operator result in register `src`.
    TracePrefix { op: PrefixOp, src: u16 },
    /// Writes register `src` through the target of the `Assignment` at
    /// `index`, and traces the write.
    Store { index: u32, at: PlanSlot, src: u16 },
    /// Prints the `SAY` at `index` from register `src`, or a blank line when
    /// it has no expression at all.
    Say { index: u32, src: Option<u16> },
    /// The `SIGNAL` at `index`, in whichever of its three forms that
    /// instruction carries.
    Signal { index: u32, src: Option<u16> },
    /// The `PARSE`, `ARG` or `PULL` at `index`, with its template walked by
    /// `Interp::exec_parse`.
    Parse { index: u32, src: Option<u16> },
    /// Ends the activation at `index` with the value in register `src`, or
    /// with no value at all for the bare form, and answers the `Flow` its
    /// keyword calls for.
    Return {
        index: u32,
        src: Option<u16>,
        keyword: ReturnKeyword,
    },
    /// Queues the line in register `src` at the end its keyword names, or the
    /// null string for the bare form, for the `PUSH`/`QUEUE` at `index`.
    Queue {
        index: u32,
        src: Option<u16>,
        keyword: QueueKeyword,
    },
    /// Runs the `CALL name` at `index`, from the resolution this site has kept
    /// or a fresh one, and settles `RESULT` from what came back.
    Call { index: u32, site: u16 },
    /// [`Op::Call`] with its arguments already on the driver's argument
    /// stack, which is what [`Op::PushArg`] put there.
    CallNamed { index: u32, site: u16, argc: u16 },
    /// Runs the message-send clause at `index`: the receiver, the scope
    /// override, the arguments and their `>A>` lines, the send, the `>M>`
    /// line, and `RESULT`. All of it through `Interp::exec_message`, the
    /// same function `step` arm calls.
    Message { index: u32 },
    /// Binds the `EXPOSE` at `index` -- its names to the receiving object's
    /// pool for the running method's scope, through `Interp::exec_expose`,
    /// which is arm.
    Expose { index: u32 },
    /// Runs the instruction at `index` through the arm of
    /// `Interp::exec_instruction` that its kind names -- the shared
    /// implementation named, entered without `Op::Clause`'s region around it.
    Exec { index: u32 },
    /// Answers the `Flow` the `LEAVE`/`ITERATE` at `index` resolves to: the
    /// instruction's own name, if it has one, and the [`crate::run::
    /// LeaveOrigin`] captured here rather than reconstructed later.
    Escape { index: u32 },
    /// Continues at op `target`.
    Jump { target: u32 },
    /// Continues at op `target` unless register `reg` holds the logical value
    /// `1`, and at the next op if it does.
    JumpUnless { reg: u16, target: u32 },
    /// Validates the condition value in `reg`, emits the `>>>` line it owes,
    /// and leaves the logical value [`Op::JumpUnless`] reads back in that same
    /// register.
    Condition {
        index: u32,
        reg: u16,
        keyword: ConditionKeyword,
    },
}

/// Which keyword's condition an [`Op::Condition`] is validating.
#[derive(Clone, Copy)]
pub(crate) enum ConditionKeyword {
    If,
    When,
}

impl ConditionKeyword {
    /// The raiser for a condition value that is not exactly `0` or `1`.
    pub(crate) fn raiser(self) -> fn(&[u8]) -> Raised {
        match self {
            ConditionKeyword::If => raised_if_not_logical,
            ConditionKeyword::When => raised_when_not_logical,
        }
    }
}

/// Which driver steps the member clauses of a construct that resolves the
/// whole of itself inside one clause step -- a `DO`/`LOOP`'s body today.
#[derive(Clone, Copy)]
pub(crate) enum BodyEngine<'a> {
    /// The clauses are stepped from `chunk`, whose `op_of` indexes exactly the
    /// body `Code::body` names, with `registers` naming the region
    /// `Interp::run_chunk` reserved for it.
    Chunk {
        chunk: &'a Chunk,
        registers: FrameId,
    },
}

/// The route from an expression slot's root down to the node one op names.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct NodePath(u32);

impl NodePath {
    /// The slot's own expression: the sentinel alone, with no steps below it.
    pub(crate) const ROOT: NodePath = NodePath(1);

    /// One step further down, into the right child when `right` and the left
    /// or only child otherwise, or `None` when this width has no room for it.
    pub(crate) fn child(self, right: bool) -> Option<NodePath> {
        (self.0 >> (u32::BITS - 1) == 0).then(|| NodePath(self.0 << 1 | u32::from(right)))
    }

    /// The steps below the sentinel, **outermost first**: the order a descent
    /// from the slot's root walks them in.
    pub(crate) fn steps(self) -> impl Iterator<Item = bool> {
        // The sentinel is the highest set bit, so its position is how many
        // steps sit below it -- zero for `ROOT`, whose value is the sentinel
        // alone.
        let depth = u32::BITS - 1 - self.0.leading_zeros();
        (0..depth).rev().map(move |bit| self.0 >> bit & 1 == 1)
    }
}

/// The frame slot a symbol resolves to, worked out when this chunk was
/// compiled, or the absence of one.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct PlanSlot(u32);

impl PlanSlot {
    /// No slot resolved: the read or write works its own out, which is what
    /// every one of them did before there was a compiler to do it earlier.
    pub(crate) const UNRESOLVED: PlanSlot = PlanSlot(u32::MAX);

    /// The slot `at`, or [`PlanSlot::UNRESOLVED`] when it does not fit this
    /// width.
    pub(crate) fn of(at: usize) -> PlanSlot {
        match u32::try_from(at) {
            Ok(at) if at != PlanSlot::UNRESOLVED.0 => PlanSlot(at),
            _ => PlanSlot::UNRESOLVED,
        }
    }

    /// The slot, or `None` when this op carries none.
    pub(crate) fn resolved(self) -> Option<usize> {
        (self != PlanSlot::UNRESOLVED).then_some(self.0 as usize)
    }
}

/// The one way [`compile`] can fail: a body that does not fit the index
/// widths this stream commits to, not a language construct it refuses (the
/// plan's Decisions section: "the compiler has one error, and it is a
/// machine width rather than a language construct"). Every instruction
/// still compiles; this reports that the *stream* built from them does not
/// fit some index's width.
#[derive(Debug)]
pub(crate) struct ChunkTooLarge {
    /// What overflowed: the `u32` op stream, or the `u16` register file.
    #[allow(dead_code, reason = "no production caller renders a refusal's reason")]
    pub(crate) what: &'static str,
}

/// Whether a compiled arithmetic site keeps a hint about which path to try
/// first.
const QUICKENING: bool = true;

/// The state a site starts in and stays in while the small-integer path keeps
/// answering: try that path first.
const TRY_SMALL_INT: u32 = 0;

/// The state a site moves to the first time the small-integer path answers
/// `None`: go straight to the general one.
const GENERAL: u32 = 1;

/// One arithmetic site's hint.
struct PatchSlot(Cell<u32>);

impl PatchSlot {
    fn new() -> PatchSlot {
        PatchSlot(Cell::new(TRY_SMALL_INT))
    }

    fn get(&self) -> u32 {
        self.0.get()
    }

    fn set(&self, state: u32) {
        self.0.set(state);
    }
}

/// One chunk's patch table: a hint per [`Op::Arith`], indexed by that op's own
/// `hint` field.
struct Hints {
    /// One slot per arithmetic op, or **empty** when [`QUICKENING`] is off.
    slots: Vec<PatchSlot>,
    /// How many arithmetic ops have taken a slot.
    next: u32,
}

impl Hints {
    fn new() -> Hints {
        Hints {
            slots: Vec::new(),
            next: 0,
        }
    }

    /// Reserves the slot for one arithmetic op, answering the index it
    /// carries.
    fn reserve(&mut self) -> Result<u32, ChunkTooLarge> {
        let at = self.next;
        self.next = at.checked_add(1).ok_or(ChunkTooLarge {
            what: "arithmetic sites past u32",
        })?;
        if QUICKENING {
            self.slots.push(PatchSlot::new());
        }
        Ok(at)
    }

    /// Whether site `at` is still worth trying the small-integer path for.
    fn tries_small_int(&self, at: u32) -> bool {
        !QUICKENING
            || self
                .slots
                .get(at as usize)
                .is_none_or(|slot| slot.get() == TRY_SMALL_INT)
    }

    /// Records that site `at` has had the small-integer path answer `None`.
    fn saw_general(&self, at: u32) {
        if QUICKENING && let Some(slot) = self.slots.get(at as usize) {
            slot.set(GENERAL);
        }
    }
}

/// Whether a compiled call site keeps the resolution it made.
const CALL_SITE_CACHE: bool = true;

/// One call site's kept resolution, or none yet.
struct CallSite(Cell<Option<Resolved>>);

impl CallSite {
    fn new() -> CallSite {
        CallSite(Cell::new(None))
    }

    fn get(&self) -> Option<Resolved> {
        self.0.get()
    }

    fn set(&self, resolved: Resolved) {
        self.0.set(Some(resolved));
    }
}

/// One chunk's resolution table: a slot per call op -- [`Op::Call`] and
/// [`Op::CallExpr`] alike -- indexed by that op's own `site` field.
struct Calls {
    /// One slot per call op, or **empty** when [`CALL_SITE_CACHE`] is off.
    slots: Vec<CallSite>,
    /// How many call ops have taken a slot, kept separately from `slots.len()`
    /// so that the indices the ops carry are the same whether or not the table
    /// exists -- [`Hints::next`]'s own reason, so that a golden op stream reads
    /// identically under either setting.
    next: u16,
}

impl Calls {
    fn new() -> Calls {
        Calls {
            slots: Vec::new(),
            next: 0,
        }
    }

    /// Reserves the slot for one call op, answering the index it carries.
    fn reserve(&mut self) -> Result<u16, ChunkTooLarge> {
        let at = self.next;
        self.next = at.checked_add(1).ok_or(ChunkTooLarge {
            what: "call sites past u16",
        })?;
        if CALL_SITE_CACHE {
            self.slots.push(CallSite::new());
        }
        Ok(at)
    }

    /// What site `at` resolved to last time, or `None` for a site that has not
    /// resolved yet or has no slot at all -- which is what makes
    /// [`CALL_SITE_CACHE`] off behave as no table rather than as a table that
    /// answers wrongly.
    fn resolved(&self, at: u16) -> Option<Resolved> {
        self.slots.get(at as usize).and_then(CallSite::get)
    }

    /// Records what site `at` resolved to.
    fn remember(&self, at: u16, resolved: Resolved) {
        if let Some(slot) = self.slots.get(at as usize) {
            slot.set(resolved);
        }
    }
}

/// One body's compiled instruction stream, cached on `Interp` under its
/// `Plan`'s `BodyKey` **and the [`ChunkTrace`] it was compiled under**
/// (`Interp::chunk_for`, in `plan.rs`).
/// A clause's source line and its nesting indent, both fixed by the source
/// text and so decided once, where the chunk is compiled.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) struct ClausePosition {
    /// `Plan::lines`' own entry: the source line the clause starts on.
    pub(crate) line: u32,
    /// `Plan::indents`' own entry: the nesting indent **before**
    /// `Interp::activation_indent` and `Interp::indent_offset` are added,
    /// which is what `Interp::printed_indent` adds to it.
    pub(crate) indent: u32,
}

pub(crate) struct Chunk {
    /// The trace setting this stream's ops were emitted for, which is half of
    /// the key it is cached under and the thing the driver compares the
    /// setting in force against.
    trace: ChunkTrace,
    /// One entry per instruction, in instruction order -- D21's "every
    /// instruction compiles, nothing refuses" is a claim about instructions,
    /// not about promotion, so an instruction no task has promoted still gets
    /// an op.
    ops: Vec<Op>,
    /// Instruction index -> **the op control resumes at** when it arrives at
    /// that instruction. One entry per instruction, in order, plus one final
    /// entry at `ops.len()`: `run_bounded`'s absorption guard is inclusive,
    /// so a construct's resume point can be one past its last instruction,
    /// and a map that stopped at `len - 1` would panic there instead of
    /// failing loudly at compile.
    op_of: Vec<u32>,
    /// The register allocator's high-water mark (the plan's Decisions
    /// section: "the chunk records its high-water mark").
    /// `Interp::run_chunk` reserves this many registers before running a
    /// chunk and truncates them away on the way out, and `Interp::run_fragment`
    /// does the same for the chunk an `INTERPRET` compiles.
    pub(crate) registers: u16,
    /// One entry per instruction, in instruction order, or **empty when this
    /// body's plan cannot supply every entry** -- a body planned without
    /// source has no line table, and a clause line or indent too wide for a
    /// `u32` has no entry either. Empty means the driver reads the tables as
    /// it always did, so this is a shortcut and never the only route to an
    /// answer.
    positions: Box<[ClausePosition]>,
    /// The literal values [`Op::Const`] loads, **one entry per distinct
    /// literal** rather than one per occurrence.
    consts: Vec<Box<[u8]>>,
    /// The value each entry of [`Chunk::consts`] was interned to, or
    /// [`ObjRef::NIL`] for one not built yet.
    interned: Vec<Cell<ObjRef>>,
    /// The same for [`Op::LoadConstant`], indexed by the constant symbol's own
    /// `SymbolId` rather than by a slot `compile` assigned.
    interned_symbols: Vec<Cell<ObjRef>>,
    /// The quickening hints [`Op::Arith`] reads, one per such op.
    hints: Hints,
    /// The resolutions [`Op::Call`] reads and writes, one per such op.
    calls: Calls,
}

impl Chunk {
    /// The op instruction `index` starts at, or `None` when the map is
    /// shorter than the body it belongs to.
    fn op_at(&self, index: usize) -> Option<u32> {
        self.op_of.get(index).copied()
    }

    /// The clause at instruction index `index`'s own [`ClausePosition`], or
    /// `None` where this chunk carries no table.
    pub(crate) fn position_at(&self, index: usize) -> Option<ClausePosition> {
        self.positions.get(index).copied()
    }

    /// The op at op index `at`.
    fn op_at_index(&self, at: u32) -> Option<&Op> {
        self.ops.get(at as usize)
    }

    /// The ops of `[at, end)`, or `None` when that is not a range of this
    /// stream.
    fn ops_in(&self, at: u32, end: u32) -> Option<&[Op]> {
        self.ops.get(at as usize..end as usize)
    }

    /// The setting this chunk's trace ops were emitted for.
    fn trace(&self) -> ChunkTrace {
        self.trace
    }

    /// The bytes of constant `at`, or `None` when the index is outside the
    /// table this chunk was compiled with.
    fn konst(&self, at: u32) -> Option<&[u8]> {
        self.consts.get(at as usize).map(|bytes| &bytes[..])
    }

    /// The value constant `at` was interned to, or [`ObjRef::NIL`] when it has
    /// not been built yet or the index names no entry.
    fn interned_konst(&self, at: u32) -> ObjRef {
        match self.interned.get(at as usize) {
            Some(cell) => cell.get(),
            None => ObjRef::NIL,
        }
    }

    /// Records what constant `at` was interned to.
    fn remember_konst(&self, at: u32, value: ObjRef) {
        if let Some(cell) = self.interned.get(at as usize) {
            cell.set(value);
        }
    }

    /// The value constant symbol `symbol` was interned to, or [`ObjRef::NIL`]
    /// when it has not been built yet.
    fn interned_symbol(&self, symbol: SymbolId) -> ObjRef {
        match self.interned_symbols.get(symbol.index()) {
            Some(cell) => cell.get(),
            None => ObjRef::NIL,
        }
    }

    /// Records what constant symbol `symbol` was interned to.
    fn remember_symbol(&self, symbol: SymbolId, value: ObjRef) {
        if let Some(cell) = self.interned_symbols.get(symbol.index()) {
            cell.set(value);
        }
    }

    /// Whether `reg` is inside the region `run_chunk` reserves for this
    /// chunk.
    fn holds_register(&self, reg: u16) -> bool {
        reg < self.registers
    }

    /// Whether arithmetic site `at` is still worth trying the small-integer
    /// path for ([`Hints::tries_small_int`]).
    fn tries_small_int(&self, at: u32) -> bool {
        self.hints.tries_small_int(at)
    }

    /// Records that arithmetic site `at` has fallen through to the general
    /// path ([`Hints::saw_general`]).
    fn saw_general(&self, at: u32) {
        self.hints.saw_general(at);
    }

    /// What call site `at` resolved to last time ([`Calls::resolved`]).
    fn resolved_call(&self, at: u16) -> Option<Resolved> {
        self.calls.resolved(at)
    }

    /// Records what call site `at` resolved to ([`Calls::remember`]).
    fn remember_call(&self, at: u16, resolved: Resolved) {
        self.calls.remember(at, resolved);
    }
}

#[cfg(test)]
mod tests {
    use super::NodePath;

    /// A path round-trips the steps it was built from, and refuses the step
    /// past its width rather than silently dropping the sentinel.
    #[test]
    fn a_node_path_carries_thirty_one_steps_and_refuses_the_thirty_second() {
        let mut path = NodePath::ROOT;
        assert_eq!(path.steps().count(), 0);
        for step in 0..31 {
            path = path.child(step % 2 == 0).expect("thirty-one steps fit");
        }
        assert_eq!(path.steps().count(), 31);
        assert!(path.child(false).is_none());
    }

    /// The steps come back outermost first, which is the order the descent
    /// walks them in -- a path read innermost first would land on the wrong
    /// node in every asymmetric expression.
    #[test]
    fn a_node_paths_steps_come_back_outermost_first() {
        let path = NodePath::ROOT
            .child(true)
            .and_then(|path| path.child(false))
            .expect("two steps fit");
        assert_eq!(path.steps().collect::<Vec<_>>(), [true, false]);
    }
}
