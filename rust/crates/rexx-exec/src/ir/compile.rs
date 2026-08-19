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

//! `compile`: the one pass that turns a body into a [`Chunk`] (the plan's
//! Decisions section: "compilation is whole-body and lazy: one body at a
//! time, on first entry, cached").

use std::collections::HashMap;

use rexx_parse::{
    Call, CodeBody, Expr, ExprKind, Instruction, InstructionKind, LoopKind, ParseSource, SymbolId,
};

use super::{
    Calls, Chunk, ChunkTooLarge, ClausePosition, ConditionKeyword, Hints, NodePath, Op, PlanSlot,
};
use crate::eval::{SymbolRead, is_arithmetic, is_native_binary};
use crate::plan::Plan;
use crate::run::{
    HeaderPlan, QueueKeyword, ReturnKeyword, if_targets, loop_header_plan, loop_header_slot,
    otherwise_range, when_targets,
};
use crate::trace::ChunkTrace;

/// The compile-time register stack (the plan's Decisions section: "register
/// allocation is a compile-time stack, and the chunk records its high-water
/// mark").
///
/// `mark` takes the current top, `alloc` hands out the next index, and
/// `release` puts the top back to a mark. A promoted clause takes a mark when
/// its [`Op::Clause`] is emitted and releases to it at the clause's `end`, so
/// two sibling clauses reuse the same registers and a clause nested inside
/// another's region cannot reclaim the enclosing one's. A construct whose
/// state outlives its member clauses allocates in the *enclosing* scope,
/// before those clauses are emitted, which is what puts its registers below
/// every mark they take.
///
/// The withdrawn alternative was the spike's whole-chunk monotonic counter: it
/// allocates a register per assignment and never reuses one, so a long body
/// reserves a region proportional to its length.
struct Registers {
    /// The next index `alloc` hands out.
    next: u16,
    /// The largest `next` ever reached, which is what [`Chunk::registers`]
    /// records -- the size of the region the driver has to reserve for a run
    /// of this chunk to address every register it names.
    high_water: u16,
}

/// A saved register top, handed back to [`Registers::release`].
///
/// A newtype rather than a bare `u16` so a register index and a mark cannot
/// be passed to each other's function: both are positions in the same stack
/// and the compiler is otherwise the only thing keeping them apart.
#[derive(Clone, Copy)]
struct Mark(u16);

impl Registers {
    fn new() -> Registers {
        Registers {
            next: 0,
            high_water: 0,
        }
    }

    fn mark(&self) -> Mark {
        Mark(self.next)
    }

    /// The next register, or the refusal a body needing more than `u16::MAX`
    /// live at once earns (the plan's Decisions section: "register indices
    /// are `u16`").
    fn alloc(&mut self) -> Result<u16, ChunkTooLarge> {
        let index = self.next;
        self.next = self
            .next
            .checked_add(1)
            .ok_or(ChunkTooLarge { what: "registers" })?;
        self.high_water = self.high_water.max(self.next);
        Ok(index)
    }

    /// Puts the top back to `mark`, so every register allocated since is
    /// handed out again.
    ///
    /// The high-water mark is untouched: it is what the region has to be
    /// *sized* to, and a register released is still one the run needed.
    fn release(&mut self, mark: Mark) {
        self.next = mark.0;
    }

    fn high_water(&self) -> u16 {
        self.high_water
    }
}

/// The constant table [`Op::Const`] indexes, built as the pass goes and
/// **interned**: a literal written twice gets one entry.
///
/// **The interning is what the table is for, not a refinement of it.** Every
/// entry is a heap allocation, so an un-interned table costs one allocation
/// per literal *occurrence* -- which on a straight-line body is one per
/// clause, paid at compile time -- and the mechanics spike measured that
/// artifact swamping the difference it existed to measure.
///
/// `index` borrows the body's own literal bytes rather than owning a second
/// copy of each key, so a repeated literal costs a hash and a comparison and
/// no allocation at all. That is the whole reason this is a struct with a
/// lifetime instead of a `HashMap<Box<[u8]>, u32>`: the owning form allocates
/// once per *distinct* literal for the key as well as for the entry.
struct Constants<'a> {
    /// One entry per distinct literal, in the order they were first seen,
    /// which is what [`Chunk::consts`] becomes.
    ///
    /// [`Chunk::consts`]: super::Chunk
    values: Vec<Box<[u8]>>,
    /// Which entry a literal's bytes already have.
    index: HashMap<&'a [u8], u32>,
}

impl<'a> Constants<'a> {
    fn new() -> Constants<'a> {
        Constants {
            values: Vec::new(),
            index: HashMap::new(),
        }
    }

    /// The entry `bytes` has, adding one if this is the first occurrence.
    ///
    /// The refusal is the constant half of the one error `compile` has (the
    /// plan's Decisions section: "op, constant and instruction indices are
    /// `u32`").
    fn intern(&mut self, bytes: &'a [u8]) -> Result<u32, ChunkTooLarge> {
        if let Some(&at) = self.index.get(bytes) {
            return Ok(at);
        }
        let at =
            u32::try_from(self.values.len()).map_err(|_| ChunkTooLarge { what: "constants" })?;
        self.values.push(Box::from(bytes));
        self.index.insert(bytes, at);
        Ok(at)
    }
}

/// A jump whose target is an instruction index the forward pass has not
/// reached yet, and the op that has to be rewritten once it has.
///
/// **The pass is single and forward, so every branch target is a
/// backpatch.** An `IF`'s false path lands on an instruction after it and its
/// true path resumes past the `ELSE`, and neither op index exists when the
/// `IF` itself is compiled. Recording the pair and resolving it after the
/// loop is what keeps the pass single, and resolving it through `op_of` is
/// what keeps a target an *instruction* position everywhere else: `Flow::Goto`
/// carries instruction indices too, so both spaces meet in exactly one table.
struct Patch {
    /// The op to rewrite.
    op: u32,
    /// The instruction it continues at.
    target: usize,
    /// Which of the two ops an instruction can be entered at this jump wants.
    kind: PatchKind,
}

/// An op that has to be emitted in front of an instruction's own first op,
/// so that arriving at that instruction from elsewhere runs it and falling
/// into the instruction from the op before it does not.
///
/// This is the [`Chunk::op_of`]/`first_op_of` split, from the emitting side:
/// `op_of` names whichever of these is in front, and `first_op_of` names the
/// instruction's own op. Both entries in the table below are cases where an
/// arrival has to do something the instruction itself does not.
///
/// [`Chunk::op_of`]: super::Chunk
enum Before {
    /// The end of an `IF`'s true branch: the clause boundary the tree-walker's
    /// own wrapper runs there ([`Op::EndBranch`]), and the jump past the `ELSE`
    /// when there is one to skip.
    ///
    /// **Every `IF` registers one**, because the boundary is owed whether or
    /// not there is anything to jump over. `resume` is `None` for the `IF`
    /// whose true branch falls straight through to where the false path
    /// lands, which is every `IF` without an `ELSE`.
    ThenEnd { resume: Option<usize> },
    /// A `SELECT`'s [`Op::EnterOtherwise`], in front of the `OTHERWISE` marker
    /// whose branch it opens a frame over. The `SELECT` is at this index.
    EnterOtherwise(usize),
    /// A `SELECT` branch runs out in front of this instruction
    /// ([`Op::EndWhen`]): a listed `WHEN`'s own `false_target`, or the end of
    /// the `OTHERWISE` body.
    ///
    /// **One per branch that can end here rather than one per instruction**,
    /// because nested `SELECT`s can run out at the same instruction and each
    /// owes its own frame a close.
    SelectBranchEnd,
}

/// What a listed `WHEN` needs from the `SELECT` that collected it, recorded
/// when that `SELECT` compiles and read when the `WHEN` itself does.
///
/// A table indexed by instruction rather than a lookup from the `WHEN` back to
/// its `SELECT`: the pass is forward and a `SELECT` always compiles before its
/// own `whens`, so the answer is already known by the time it is wanted, and
/// nothing has to search for it.
struct WhenInfo {
    /// The `SELECT` this `WHEN` belongs to.
    select: u32,
    /// The register that `SELECT`'s `CASE` value is in, and `None` for a plain
    /// `SELECT`.
    case: Option<u16>,
    /// Where the scan goes when this `WHEN` does not hold: the next listed
    /// `WHEN`, the `OTHERWISE`, or the `END` whose 7.3 is what "no `WHEN`
    /// matched and there is no `OTHERWISE`" means.
    false_target: usize,
    /// Which of that instruction's two entries the scan wants. It is
    /// [`PatchKind::Resume`] for an `OTHERWISE`, whose `op_of` is the
    /// [`Op::EnterOtherwise`] that opens the frame, and [`PatchKind::Enter`]
    /// for the other two, which have nothing in front of them.
    false_kind: PatchKind,
}

/// Which op a jump to an instruction means, for the one instruction where
/// the two differ: the `ELSE` a branch-end jump sits in front of.
///
/// **Arriving at an `ELSE` means two different things and the tree-walker
/// tells them apart by which loop is running.** `run_bounded`'s own doc
/// comment states it: "the true path (fall through A, land on `Else` by
/// `pc += 1`) and the false path (`Goto` straight to `Else`) arrive at the
/// identical `(instruction, pc)`, and only one of the two arrivals is
/// supposed to enter B". A flat stream has two *op* positions there instead,
/// which is what lets it answer both without a second engine -- and this is
/// the field that says which one a jump wants.
#[derive(Clone, Copy)]
enum PatchKind {
    /// The instruction's own first op: run the instruction.
    ///
    /// The `IF`'s own false path, and nothing else. It is the one arrival
    /// that must enter the `ELSE` marker.
    Enter,
    /// Where control resumes when it arrives at this instruction from
    /// anywhere else, which is `Chunk::op_of`'s own answer: the branch-end
    /// jump when there is one in front, so that a true branch finishing --
    /// by falling off its end or by a nested construct's `Flow::Goto` landing
    /// exactly on the boundary -- skips the `ELSE` rather than running it.
    Resume,
}

/// Compiles `body` into a [`Chunk`], once, whole.
///
/// Every instruction in `body.instructions` compiles (D21: "every
/// instruction compiles, nothing refuses" is about instructions). A `DO` or
/// `LOOP` becomes a [`Op::Clause`] region holding its header's own evaluation
/// and ending in [`Op::LoopRun`], whose body clauses the driver steps; an `IF`
/// becomes a [`Op::Clause`] region that evaluates its condition and jumps; a
/// `SELECT` becomes one such region per listed `WHEN` as well as for its own
/// header, laid out as a scan chain with a frame opened over whichever branch
/// wins; an `Assignment` and a `SAY` each become a region that produces one
/// value and then writes or prints it; every other instruction becomes
/// [`Op::Generic`].
///
/// **`plan` is read for one thing: the slot a promoted read or write resolves
/// to.** The rule a compiled assignment's target has to keep is that a stem, a
/// compound tail and the `>=>` line stay one implementation -- `Op::Store` goes
/// through `Interp::assign_expr_target`, which is what `step`'s own arm calls,
/// and a store path in the driver that wrote a slot itself would be the second
/// one. What that rule forbids is the *dispatch*, not the resolution: the slot
/// travels as an argument into that one function, which uses it in the arm
/// where it means anything and ignores it in the two where it does not. A
/// promoted read is the same shape one function over, `Interp::read_at` taking
/// the slot it would otherwise resolve from this same map.
///
/// The one error is a machine width, not a language construct (the plan's
/// Decisions section: "the compiler has one error, and it is a machine
/// width"). Op indices are `u32` and register indices `u16`, so a body whose
/// stream or register file would exceed either is refused rather than wrapped
/// -- unreachable in practice at this task, since nothing produces four
/// billion instructions in a test and an `IF` allocates one register it then
/// releases, but the check is the contract [`ChunkTooLarge`] documents.
///
/// **`trace` is an input to the result, not a hint** (D23). It decides which
/// promoted clauses get an [`Op::TraceClause`] of their own, so two chunks for
/// one body can differ and `Interp::chunk_for` keys on it as well as on the
/// body. A [`ChunkTrace`] rather than a whole `TraceMode`, because the
/// argument *is* the key: a compiler that could read a field the key does not
/// carry would cache a chunk under a name that does not identify it.
pub(crate) fn compile(
    body: &CodeBody,
    plan: &Plan,
    trace: ChunkTrace,
) -> Result<Chunk, ChunkTooLarge> {
    #[cfg(test)]
    count_compile_call();

    let len = body.instructions.len();
    let mut ops: Vec<Op> = Vec::with_capacity(len);
    let mut op_of = Vec::with_capacity(len + 1);
    let mut registers = Registers::new();
    let mut consts = Constants::new();
    let mut hints = Hints::new();
    let mut calls = Calls::new();
    let mut patches: Vec<Patch> = Vec::new();
    // Indexed by instruction: the op that goes in front of that instruction's
    // own, emitted at its `op_of` entry. A `Vec` keyed by the instruction the
    // op sits in front of, rather than a stack, because each entry belongs to
    // exactly one construct -- an `ELSE` to one `THEN`, an `OTHERWISE` to one
    // `SELECT` -- and the `debug_assert`s below are what say so rather than
    // assuming it.
    // A list per instruction, not one entry: nested `IF`s whose branches end
    // at the same instruction each owe a boundary of their own, which the
    // oracle runs as one synthetic instruction per branch. `len + 1` entries,
    // because a branch can end at the body's own end and still owe one.
    let mut before: Vec<Vec<Before>> = Vec::new();
    before.resize_with(len + 1, Vec::new);
    // Indexed by instruction: what a listed `WHEN` compiling at that index
    // needs from the `SELECT` that collected it.
    let mut when_info: Vec<Option<WhenInfo>> = Vec::new();
    when_info.resize_with(len, || None);
    // Indexed by instruction: a register top to put back on reaching it. A
    // `SELECT CASE`'s own value is what needs one -- it is allocated in the
    // enclosing scope so that every member clause's release leaves it alone,
    // and this is where that allocation ends.
    let mut release_at: Vec<Option<Mark>> = vec![None; len];

    // **SPIKE.** Indexed by instruction: for an `END` that closes a repeating
    // `DO`/`LOOP`, that loop's own instruction index. Filled in at the header,
    // which is where the kind and the `END`'s position are both known, and read
    // when the walk reaches the `END` itself.
    let mut loop_of_end: Vec<Option<u32>> = vec![None; len];

    // Indexed by instruction: the instruction's own first op, which is
    // `op_of`'s entry except where a `Before` op sits in front of it.
    // Compile-time only -- the driver never wants it, because `PatchKind`'s
    // own doc comment has why the ops that do are emitted here.
    let mut first_op_of = Vec::with_capacity(len);

    for (index, instruction) in body.instructions.iter().enumerate() {
        op_of.push(op_index(&ops)?);
        if let Some(mark) = release_at[index] {
            registers.release(mark);
        }
        emit_before(&mut ops, &mut patches, &mut before[index])?;
        first_op_of.push(op_index(&ops)?);
        match &instruction.kind {
            // `DO` and `LOOP` are the same construct under two spellings
            // (`step`'s own arm matches them together), so they compile the
            // same way: the header's own clause region, then the op that runs
            // the construct from what that region evaluated.
            InstructionKind::Do(body_node) | InstructionKind::Loop(body_node) => {
                // Allocated in the *enclosing* scope and released past the
                // whole loop, because these registers are what roots the
                // header's values while the loop runs: a `DO OVER`'s target
                // lives in `LoopState` for the construct's lifetime, and a
                // register released at a body clause's own boundary would be
                // handed out again and overwritten while it is still in use.
                let outer = registers.mark();
                // Named apart from `plan`, which this arm reads too: a slot
                // that compiles natively resolves its reads against it,
                // exactly as an assignment's value does.
                let header_plan = loop_header_plan(body_node);
                let roles = header_plan.as_ref().map_or(&[][..], HeaderPlan::roles);
                let mut header = Vec::with_capacity(roles.len());
                for &role in roles {
                    header.push((role, registers.alloc()?));
                }
                let header_top = registers.mark();
                let at = op_index(&ops)?;
                let echo = echoes(trace, instruction);
                // One group per header expression, and `LoopRun` inside the
                // region behind them: the whole loop runs inside the `DO` clause,
                // exactly as it does on the tree-walker. `close_region` below is
                // what fixes the region's end, from what was actually pushed.
                ops.push(Op::Clause {
                    index: instruction_index(index)?,
                    end: 0,
                });
                push_echo(&mut ops, echo, instruction_index(index)?);
                for (slot, &(role, dst)) in header.iter().enumerate() {
                    let slot = instruction_index(slot)?;
                    // **Each slot decides for itself**, so a header holding one
                    // expression outside the native set keeps native ops for
                    // its others. A slot `loop_header_slot` has no expression
                    // for takes the same answer as one `native_shape` declines,
                    // and they are one arm because they are one answer: the
                    // slot stays a whole `Op::EvalExpr`, which is what it was
                    // before any of it compiled. `run.rs`'s own
                    // `eval_chunk_expr` is where a slot with no expression then
                    // ends up, and it is loud there.
                    match loop_header_slot(body_node, slot) {
                        Some(expr) if native_shape(expr, Some(NodePath::ROOT)) => push_native(
                            &mut ops,
                            &mut consts,
                            &mut registers,
                            &mut hints,
                            &mut calls,
                            plan,
                            expr,
                            instruction_index(index)?,
                            slot,
                            Some(NodePath::ROOT),
                            dst,
                        )?,
                        _ => ops.push(Op::EvalExpr {
                            index: instruction_index(index)?,
                            slot,
                            dst,
                        }),
                    }
                    if role.keyword().is_some() {
                        ops.push(Op::TraceKeyword { role, src: dst });
                    }
                    ops.push(Op::LoopHeaderValue { role, src: dst });
                }
                // **The header's own registers, exactly: neither more nor
                // fewer.** An equality rather than a bound, because the two
                // directions are different defects and only one of them is
                // harmful.
                //
                // Above `header_top` is a register a slot allocated and did not
                // hand back. Nothing reuses it until the `END`, so it is waste
                // rather than corruption -- measured, leaking one per slot with
                // this line removed leaves the `ir_dual` suite green and moves
                // golden streams and nothing else.
                //
                // Below `header_top` is a header value's own register handed
                // back early, which `LoopState` still reads for the rest of the
                // construct: that is the hazard the enclosing-scope allocation
                // above exists to prevent, and it is what the equality guards
                // against a future change. `push_native` cannot produce it
                // today, because it releases only to a mark it took itself.
                debug_assert_eq!(
                    registers.mark().0,
                    header_top.0,
                    "the register top after the header is not the header's own: above it is a \
                     register nothing reuses until the END, below it is a header value handed \
                     to the body while the loop still reads it"
                );
                ops.push(Op::LoopRun {
                    index: instruction_index(index)?,
                });
                close_region(&mut ops, at)?;
                // **SPIKE.** A repeating loop's `END` carries the op that ends
                // a pass. `Simple` is left alone: it does not repeat, so it has
                // no pass to end, and its `END` stays the `Generic` that no
                // range ever reaches.
                if let Some(end) = body_node.end
                    && end < len
                    && !matches!(body_node.kind, LoopKind::Simple)
                {
                    loop_of_end[end] = Some(instruction_index(index)?);
                }
                // Past the `END`, so nothing between here and there can reuse a
                // register the running loop still reads. A loop whose `END` is
                // the body's last instruction has nothing to hang the release
                // on and needs none: there is nothing after it to hand the
                // registers to.
                if let Some(end) = body_node.end
                    && end + 1 < len
                {
                    release_to(&mut release_at[end + 1], outer);
                }
            }
            // The `IF` clause is its condition and nothing else -- the branch
            // it chooses is not inside it, which is the boundary
            // `run.rs`'s own `If` arm measured (`SIGL` reports the `IF`'s line,
            // not the branch's). So the region is the condition's ops and the
            // jump that reads them, and the register the condition lands in is
            // released at its end.
            //
            // **`push_value` is deliberately not used here, because the
            // fallback is not the same op.** A condition `native_shape`
            // declines stays one `Op::EvalExpr` doing the whole job,
            // validation and `>>>` included, through `eval_chunk_expr`'s own
            // `If` arm -- so an `Op::Condition` behind it would trace the value
            // twice and validate it twice.
            InstructionKind::If {
                condition,
                false_target,
                ..
            } => {
                let targets = if_targets(&body.instructions, *false_target);
                let mark = registers.mark();
                let dst = registers.alloc()?;
                let at = op_index(&ops)?;
                let echo = echoes(trace, instruction);
                ops.push(Op::Clause {
                    index: instruction_index(index)?,
                    end: 0,
                });
                push_echo(&mut ops, echo, instruction_index(index)?);
                if native_shape(condition, Some(NodePath::ROOT)) {
                    push_native(
                        &mut ops,
                        &mut consts,
                        &mut registers,
                        &mut hints,
                        &mut calls,
                        plan,
                        condition,
                        instruction_index(index)?,
                        0,
                        Some(NodePath::ROOT),
                        dst,
                    )?;
                    ops.push(Op::Condition {
                        index: instruction_index(index)?,
                        reg: dst,
                        keyword: ConditionKeyword::If,
                    });
                } else {
                    ops.push(Op::EvalExpr {
                        index: instruction_index(index)?,
                        slot: 0,
                        dst,
                    });
                }
                let jump = op_index(&ops)?;
                ops.push(Op::JumpUnless {
                    reg: dst,
                    target: 0,
                });
                close_region(&mut ops, at)?;
                patches.push(Patch {
                    op: jump,
                    target: targets.false_target,
                    kind: PatchKind::Enter,
                });
                registers.release(mark);
                // Only when the false path lands on an `ELSE`: without one,
                // the true branch's fallthrough is already the resume, and a
                // jump to where control was going anyway is an op the driver
                // would execute for nothing.
                // The boundary is owed either way; the jump only when the
                // false path lands on an `ELSE`, since without one the true
                // branch's fallthrough is already the resume and a jump to
                // where control was going anyway is an op the driver would run
                // for nothing.
                let resume = (targets.resume != targets.false_target).then_some(targets.resume);
                before[targets.false_target].push(Before::ThenEnd { resume });
            }
            // The `SELECT` clause is its `CASE` expression and nothing else,
            // which is the boundary `run.rs`'s own `Select` arm measured
            // (`SIGL` reports the `SELECT`'s line, not the first `WHEN`'s), and
            // a plain `SELECT` has an empty region rather than none, because
            // the clause and its boundary are owed either way.
            InstructionKind::Select {
                case,
                whens,
                otherwise,
                end,
                ..
            } => {
                let select_end = otherwise_range(len, *end);
                // Allocated in the *enclosing* scope, before any member
                // clause's mark is taken, because the last `WHEN`'s test comes
                // after every earlier `WHEN`'s branch has already run: a
                // register released at a clause boundary inside the construct
                // would be handed out again while this one is still live.
                let outer = registers.mark();
                let case_reg = match case {
                    Some(_) => Some(registers.alloc()?),
                    None => None,
                };
                let at = op_index(&ops)?;
                let echo = echoes(trace, instruction);
                ops.push(Op::Clause {
                    index: instruction_index(index)?,
                    end: 0,
                });
                push_echo(&mut ops, echo, instruction_index(index)?);
                if let Some(dst) = case_reg {
                    ops.push(Op::EvalExpr {
                        index: instruction_index(index)?,
                        slot: 0,
                        dst,
                    });
                }
                // Closed here, so that `SelectCaseText` below is past the
                // region's end -- setting the case text is the construct's
                // business rather than the header clause's, and its own doc
                // comment has why.
                close_region(&mut ops, at)?;
                ops.push(Op::SelectCaseText {
                    index: instruction_index(index)?,
                    case: case_reg,
                });
                // Past the whole construct, so nothing between here and the
                // `END` can reuse the register. A `select_end` of `len` has no
                // instruction to hang the release on and needs none: there is
                // nothing after it to hand the register to.
                if select_end < len {
                    release_to(&mut release_at[select_end], outer);
                }
                // Where the scan goes once no listed `WHEN` is left, which is
                // also where it starts when there is no `WHEN` at all.
                let no_match = match otherwise {
                    Some(otherwise_index) => {
                        debug_assert!(
                            !before[*otherwise_index]
                                .iter()
                                .any(|before| matches!(before, Before::EnterOtherwise(_))),
                            "two SELECTs want a frame opened in front of instruction \
                             {otherwise_index}, so an OTHERWISE belongs to more than one SELECT"
                        );
                        before[*otherwise_index].push(Before::EnterOtherwise(index));
                        // The `OTHERWISE` body's own end, which is the
                        // `SELECT`'s: `otherwise_frame` gives that frame this
                        // same range.
                        before[select_end].push(Before::SelectBranchEnd);
                        (*otherwise_index, PatchKind::Resume)
                    }
                    // Landing on the `END` is what makes 7.3 the `END`'s own
                    // clause rather than this `SELECT`'s, exactly as the
                    // tree-walker's `Goto(end)` does.
                    None => (select_end, PatchKind::Enter),
                };
                for (position, &when_index) in whens.iter().enumerate() {
                    let (false_target, false_kind) = match whens.get(position + 1) {
                        Some(&next) => (next, PatchKind::Enter),
                        None => no_match,
                    };
                    when_info[when_index] = Some(WhenInfo {
                        select: instruction_index(index)?,
                        case: case_reg,
                        false_target,
                        false_kind,
                    });
                }
                // The first listed `WHEN` is the instruction after this one in
                // every program that parses, so the scan is reached by falling
                // through and a jump to it would be an op the driver runs for
                // nothing. Emitted only when it is not -- a `SELECT` with no
                // listed `WHEN` at all is the case that reaches this.
                if whens.first() != Some(&(index + 1)) {
                    let (target, kind) = match whens.first() {
                        Some(&first) => (first, PatchKind::Enter),
                        None => no_match,
                    };
                    let op = op_index(&ops)?;
                    ops.push(Op::Jump { target: 0 });
                    patches.push(Patch { op, target, kind });
                }
            }
            // A **listed** `WHEN`/`WHEN CASE`: one whose `SELECT` collected it,
            // which is what `when_info` holds an entry for. An *absorbed* one
            // -- itself another `WHEN`'s consequence, never collected -- has
            // none, and falls to `Generic` below, where `step`'s own arm
            // evaluates it and branches exactly as it does for the tree-walker.
            //
            // The clause is the condition and nothing else, same as an `IF`'s,
            // and `EnterWhen` sits past the region because opening the frame is
            // the branch's business rather than the clause's.
            InstructionKind::When { .. } | InstructionKind::WhenCase { .. }
                if when_info[index].is_some() =>
            {
                let info = when_info[index]
                    .take()
                    .expect("the guard above just observed one");
                // **Where this `WHEN`'s branch runs out, taken from the
                // instruction rather than from the `SELECT`'s scan chain.**
                // `Interp::when_frame` reads `when_targets` for the frame's
                // own `op_end`, and the two part company: a `WHEN` absorbed as
                // another's `THEN` is not in the list the scan walks, so the
                // next *listed* `WHEN` is not where this branch ends.
                before[when_targets(&instruction.kind, len).body_end].push(Before::SelectBranchEnd);
                let mark = registers.mark();
                let dst = registers.alloc()?;
                let at = op_index(&ops)?;
                let echo = echoes(trace, instruction);
                ops.push(Op::Clause {
                    index: instruction_index(index)?,
                    end: 0,
                });
                push_echo(&mut ops, echo, instruction_index(index)?);
                // **Only a plain `WHEN`'s condition promotes.** A `WhenCase`
                // holds a list of values compared against the enclosing
                // `SELECT CASE`'s own text through `Interp::test_case_when`,
                // which is not a condition at all: it traces two `>>>` lines
                // per value and raises nothing for a value that is not
                // `0`/`1`. So it stays on `Op::WhenTest`, which is the op that
                // does that whole job, and so does a `When` whose condition
                // `native_shape` declines.
                match &instruction.kind {
                    InstructionKind::When { condition, .. }
                        if native_shape(condition, Some(NodePath::ROOT)) =>
                    {
                        push_native(
                            &mut ops,
                            &mut consts,
                            &mut registers,
                            &mut hints,
                            &mut calls,
                            plan,
                            condition,
                            instruction_index(index)?,
                            0,
                            Some(NodePath::ROOT),
                            dst,
                        )?;
                        ops.push(Op::Condition {
                            index: instruction_index(index)?,
                            reg: dst,
                            keyword: ConditionKeyword::When,
                        });
                    }
                    _ => ops.push(Op::WhenTest {
                        index: instruction_index(index)?,
                        case: info.case,
                        dst,
                    }),
                }
                let jump = op_index(&ops)?;
                ops.push(Op::JumpUnless {
                    reg: dst,
                    target: 0,
                });
                // Closed before `EnterWhen`, which opens the branch's frame and
                // is the branch's business rather than the clause's.
                close_region(&mut ops, at)?;
                patches.push(Patch {
                    op: jump,
                    target: info.false_target,
                    kind: info.false_kind,
                });
                registers.release(mark);
                ops.push(Op::EnterWhen {
                    select: info.select,
                    when: instruction_index(index)?,
                });
            }
            // An assignment's clause is its value expression and the write,
            // both inside the region: the write is what the clause *is*, and
            // the boundary that follows it is where a `CALL ON` handler queued
            // by the value expression runs -- measured, the handler sees the
            // assignment already done.
            InstructionKind::Assignment { target, value } => {
                let mark = registers.mark();
                let dst = registers.alloc()?;
                let at = op_index(&ops)?;
                let echo = echoes(trace, instruction);
                ops.push(Op::Clause {
                    index: instruction_index(index)?,
                    end: 0,
                });
                push_echo(&mut ops, echo, instruction_index(index)?);
                push_value(
                    &mut ops,
                    &mut consts,
                    &mut registers,
                    &mut hints,
                    &mut calls,
                    plan,
                    value,
                    instruction_index(index)?,
                    0,
                    dst,
                )?;
                ops.push(Op::Store {
                    index: instruction_index(index)?,
                    at: write_slot(plan, target),
                    src: dst,
                });
                close_region(&mut ops, at)?;
                registers.release(mark);
            }
            // A `SAY` is the same shape with the print in place of the write,
            // and one register fewer when it has no expression: the bare form
            // prints a blank line and traces the null string, which is a
            // decision `Op::Say` carries rather than an empty register.
            InstructionKind::Say { expression } => {
                let mark = registers.mark();
                let at = op_index(&ops)?;
                let echo = echoes(trace, instruction);
                ops.push(Op::Clause {
                    index: instruction_index(index)?,
                    end: 0,
                });
                push_echo(&mut ops, echo, instruction_index(index)?);
                let src = match expression {
                    Some(expression) => {
                        let dst = registers.alloc()?;
                        push_value(
                            &mut ops,
                            &mut consts,
                            &mut registers,
                            &mut hints,
                            &mut calls,
                            plan,
                            expression,
                            instruction_index(index)?,
                            0,
                            dst,
                        )?;
                        Some(dst)
                    }
                    None => None,
                };
                ops.push(Op::Say {
                    index: instruction_index(index)?,
                    src,
                });
                close_region(&mut ops, at)?;
                registers.release(mark);
            }
            // `SIGNAL`, all three forms. The `SAY` shape, with the expression
            // present only for `VALUE` -- and it goes through `push_value`
            // like any other, so a `SIGNAL VALUE` whose expression is a
            // literal or a bare symbol reaches the same native op an
            // assignment's value would.
            InstructionKind::Signal(signal) => {
                let mark = registers.mark();
                let at = op_index(&ops)?;
                let echo = echoes(trace, instruction);
                ops.push(Op::Clause {
                    index: instruction_index(index)?,
                    end: 0,
                });
                push_echo(&mut ops, echo, instruction_index(index)?);
                let src = match &**signal {
                    rexx_parse::Signal::Value(expression) => {
                        let dst = registers.alloc()?;
                        push_value(
                            &mut ops,
                            &mut consts,
                            &mut registers,
                            &mut hints,
                            &mut calls,
                            plan,
                            expression,
                            instruction_index(index)?,
                            0,
                            dst,
                        )?;
                        Some(dst)
                    }
                    rexx_parse::Signal::Label(_) | rexx_parse::Signal::Trap(_) => None,
                };
                ops.push(Op::Signal {
                    index: instruction_index(index)?,
                    src,
                });
                close_region(&mut ops, at)?;
                registers.release(mark);
            }
            // `PARSE`, `ARG` and `PULL`, which are one instruction with three
            // spellings and reach one `exec_parse`. The `SAY` shape, with the
            // expression present only for `PARSE VALUE expr WITH` -- every
            // other source reads a variable, an argument list or a line, none
            // of which is an `Expr` a register could hold. `Op::Parse`'s own
            // doc has why the template's `(expr)` patterns stay behind.
            InstructionKind::Parse(parse)
            | InstructionKind::Arg(parse)
            | InstructionKind::Pull(parse) => {
                let mark = registers.mark();
                let at = op_index(&ops)?;
                let echo = echoes(trace, instruction);
                ops.push(Op::Clause {
                    index: instruction_index(index)?,
                    end: 0,
                });
                push_echo(&mut ops, echo, instruction_index(index)?);
                let src = match &parse.source {
                    ParseSource::Value(Some(expression)) => {
                        let dst = registers.alloc()?;
                        push_value(
                            &mut ops,
                            &mut consts,
                            &mut registers,
                            &mut hints,
                            &mut calls,
                            plan,
                            expression,
                            instruction_index(index)?,
                            0,
                            dst,
                        )?;
                        Some(dst)
                    }
                    _ => None,
                };
                ops.push(Op::Parse {
                    index: instruction_index(index)?,
                    src,
                });
                close_region(&mut ops, at)?;
                registers.release(mark);
            }
            // `RETURN` and `EXIT` are the `SAY` shape with a `Flow` in place
            // of the print, and one arm rather than two because the keyword is
            // all that differs -- read off the kind here exactly as `step`'s
            // own `PUSH`/`QUEUE` arm reads its end of the queue.
            //
            // **The op is the last of the region and control does not come
            // back to it**, so nothing follows it inside the region and
            // `close_region` marks the end the driver settles the `Flow`
            // against.
            InstructionKind::Return { expression } | InstructionKind::Exit { expression } => {
                let keyword = if matches!(instruction.kind, InstructionKind::Return { .. }) {
                    ReturnKeyword::Return
                } else {
                    ReturnKeyword::Exit
                };
                let mark = registers.mark();
                let at = op_index(&ops)?;
                let echo = echoes(trace, instruction);
                ops.push(Op::Clause {
                    index: instruction_index(index)?,
                    end: 0,
                });
                push_echo(&mut ops, echo, instruction_index(index)?);
                let src = match expression {
                    Some(expression) => {
                        let dst = registers.alloc()?;
                        push_value(
                            &mut ops,
                            &mut consts,
                            &mut registers,
                            &mut hints,
                            &mut calls,
                            plan,
                            expression,
                            instruction_index(index)?,
                            0,
                            dst,
                        )?;
                        Some(dst)
                    }
                    None => None,
                };
                ops.push(Op::Return {
                    index: instruction_index(index)?,
                    src,
                    keyword,
                });
                close_region(&mut ops, at)?;
                registers.release(mark);
            }
            // `PUSH` and `QUEUE` are the `SAY` shape with the queue in place
            // of the print, one arm for the reason the pair above is one.
            InstructionKind::Push { expression } | InstructionKind::Queue { expression } => {
                let keyword = if matches!(instruction.kind, InstructionKind::Push { .. }) {
                    QueueKeyword::Push
                } else {
                    QueueKeyword::Queue
                };
                let mark = registers.mark();
                let at = op_index(&ops)?;
                let echo = echoes(trace, instruction);
                ops.push(Op::Clause {
                    index: instruction_index(index)?,
                    end: 0,
                });
                push_echo(&mut ops, echo, instruction_index(index)?);
                let src = match expression {
                    Some(expression) => {
                        let dst = registers.alloc()?;
                        push_value(
                            &mut ops,
                            &mut consts,
                            &mut registers,
                            &mut hints,
                            &mut calls,
                            plan,
                            expression,
                            instruction_index(index)?,
                            0,
                            dst,
                        )?;
                        Some(dst)
                    }
                    None => None,
                };
                ops.push(Op::Queue {
                    index: instruction_index(index)?,
                    src,
                    keyword,
                });
                close_region(&mut ops, at)?;
                registers.release(mark);
            }
            // A `CALL name`/`CALL "name"` clause is the call and nothing else,
            // and it takes **no register**: an argument is not an `ObjRef` --
            // a `>name` reference carries the caller's own slot with it -- so
            // the arguments stay `Interp::invoke_call`'s, together with every
            // `>A>` line and every intermediate their expressions emit.
            //
            // The other three `Call` forms fall to `Generic` below.
            // `CALL ON`/`OFF` resolves no name at all, `CALL (expr)` learns
            // its name at run time and `CALL ns:name` is Phase 5's loud gap;
            // the first two have their own witnesses in `golden_tests.rs`.
            InstructionKind::Call(call) if matches!(&**call, Call::Named { .. }) => {
                let at = op_index(&ops)?;
                let echo = echoes(trace, instruction);
                ops.push(Op::Clause {
                    index: instruction_index(index)?,
                    end: 0,
                });
                push_echo(&mut ops, echo, instruction_index(index)?);
                ops.push(Op::Call {
                    index: instruction_index(index)?,
                    site: calls.reserve()?,
                });
                close_region(&mut ops, at)?;
            }
            // A message send as a whole clause: the plain form, the `~~`
            // form, and the message-assignment form. No register and no
            // expression slot -- `Interp::exec_message` evaluates the term
            // itself, exactly as the tree-walker's own arm does, so what this
            // promotion decides is the clause region around it and nothing
            // about the send.
            InstructionKind::Message { .. } => {
                let at = op_index(&ops)?;
                let echo = echoes(trace, instruction);
                ops.push(Op::Clause {
                    index: instruction_index(index)?,
                    end: 0,
                });
                push_echo(&mut ops, echo, instruction_index(index)?);
                ops.push(Op::Message {
                    index: instruction_index(index)?,
                });
                close_region(&mut ops, at)?;
            }
            // `EXPOSE`. No register and no expression slot: the names come
            // out of the instruction, and a `VariableRef::Indirect` reads its
            // selector through `Interp::read` exactly as the tree-walker's
            // own arm does, so what this promotion decides is the clause
            // region around it and nothing about the binding.
            InstructionKind::Expose { .. } => {
                let at = op_index(&ops)?;
                let echo = echoes(trace, instruction);
                ops.push(Op::Clause {
                    index: instruction_index(index)?,
                    end: 0,
                });
                push_echo(&mut ops, echo, instruction_index(index)?);
                ops.push(Op::Expose {
                    index: instruction_index(index)?,
                });
                close_region(&mut ops, at)?;
            }
            // `LEAVE`/`ITERATE`. No register and no expression slot: the name
            // is in the instruction and the origin is captured at run time
            // from the clause the region has open, exactly as the
            // tree-walker's own two arms capture it.
            InstructionKind::Leave { .. } | InstructionKind::Iterate { .. } => {
                let at = op_index(&ops)?;
                let echo = echoes(trace, instruction);
                ops.push(Op::Clause {
                    index: instruction_index(index)?,
                    end: 0,
                });
                push_echo(&mut ops, echo, instruction_index(index)?);
                ops.push(Op::Escape {
                    index: instruction_index(index)?,
                });
                close_region(&mut ops, at)?;
            }
            // The marker instructions: `step` answers `Ok(Flow::Next)` for
            // each and computes nothing, so the clause region is the whole of
            // what they do and it holds no op but its own echo
            // ([`Op::Clause`]'s own doc comment).
            //
            // **A `LABEL` is one of them, and it is the one with machinery
            // around it.** It is a jump target, so `Chunk::op_of` has to keep
            // naming it -- which it does, because a region opens at the
            // instruction's own entry exactly where its `Generic` op sat. It
            // echoes under `TRACE L` where an ordinary clause does not, which
            // [`echoes`] already asks through `ChunkTrace::echoes` and
            // `Interp::echo_compiled_clause` already answers by passing
            // `is_label` through. And it must not spend the activation's
            // first-instruction permission, which is
            // `Interp::grant_procedure_permission`'s own label arm: it never
            // grants for a label, so `procedure_permitted` is false when this
            // region takes it and `first_instruction_pending` survives to the
            // instruction after -- which is what makes `sub:` followed by
            // `PROCEDURE` legal.
            //
            // **`ELSE` and `OTHERWISE` execute the same way and are not
            // here.** Each is reached through machinery this arm does not
            // touch -- an `ELSE` through the `IF`'s own false target, an
            // `OTHERWISE` through the [`Op::EnterOtherwise`] that sits at its
            // entry -- so each is its own promotion with its own witnesses.
            //
            // **`END` is not here, and that is a measurement.** It is not a
            // marker: its `EndStyle::Select` arm raises 7.3. It is also barely
            // executed, because every construct returns a `Flow` that resumes
            // *past* its own `END` -- counted under a scratch build, zero
            // `END` clauses are stepped over a whole run of
            // `samples/rexxcps.rex` and six over the whole corpus, against
            // 700,002 `THEN` on `rexxcps` alone.
            InstructionKind::Nop | InstructionKind::Then | InstructionKind::Label { .. } => {
                let at = op_index(&ops)?;
                let echo = echoes(trace, instruction);
                ops.push(Op::Clause {
                    index: instruction_index(index)?,
                    end: 0,
                });
                push_echo(&mut ops, echo, instruction_index(index)?);
                close_region(&mut ops, at)?;
            }
            // **SPIKE.** The `END` of a repeating loop, which the flattened
            // form reaches by falling out of the body rather than by a range
            // check. An `END` closing anything else keeps the `Generic` below.
            InstructionKind::End { .. } if loop_of_end[index].is_some() => {
                ops.push(Op::LoopNext {
                    index: loop_of_end[index].expect("the guard just observed it"),
                });
            }
            _ => ops.push(Op::Generic {
                index: instruction_index(index)?,
            }),
        }
    }
    // A branch that ends at the body's own end still owes its boundary, and
    // there is no instruction iteration left to emit it in.
    emit_before(&mut ops, &mut patches, &mut before[len])?;
    // One entry past the last instruction, pushed after the loop above:
    // `run_bounded`'s absorption guard is inclusive, so a construct's
    // resume point can be `end`, one past its last instruction, and a table
    // that stopped at `len - 1` would panic there instead of failing loudly
    // at compile.
    let end = op_index(&ops)?;
    op_of.push(end);
    first_op_of.push(end);

    for patch in patches {
        // In range for every target: both tables have an entry per
        // instruction plus the end one, and `if_targets` clamps `None` to
        // `len`.
        let target = match patch.kind {
            PatchKind::Enter => first_op_of[patch.target],
            PatchKind::Resume => op_of[patch.target],
        };
        match &mut ops[patch.op as usize] {
            Op::Jump { target: slot } | Op::JumpUnless { target: slot, .. } => *slot = target,
            _ => unreachable!("a patch names the op it was recorded beside"),
        }
    }

    assert_clause_regions_hold_no_generic_op(&ops);
    assert_trace_ops_open_a_clause_region(&ops);
    assert_literal_echoes_follow_their_load(&ops);
    assert_read_echoes_follow_their_load(&ops);
    assert_operator_echoes_follow_their_op(&ops);
    assert_prefix_echoes_follow_their_op(&ops);
    assert_call_echoes_follow_their_op(&ops);
    assert_keyword_echoes_precede_their_value(&ops);
    assert_region_ops_name_their_clause(&ops);

    let consts = consts.values;
    // Sized from the stream rather than from the program's symbol table, which
    // `compile` does not have: the widest constant symbol this body actually
    // names, and nothing for a body that names none. `Chunk::interned_symbols`
    // has why the gap in front of a high id is the right trade.
    let widest_constant_symbol = ops
        .iter()
        .filter_map(|op| match op {
            Op::LoadConstant { symbol, .. } => Some(symbol.index()),
            _ => None,
        })
        .max();
    Ok(Chunk {
        trace,
        ops,
        op_of,
        registers: registers.high_water(),
        positions: clause_positions(body, plan),
        interned: vec![std::cell::Cell::new(rexx_core::ObjRef::NIL); consts.len()],
        interned_symbols: vec![
            std::cell::Cell::new(rexx_core::ObjRef::NIL);
            widest_constant_symbol.map_or(0, |widest| widest + 1)
        ],
        consts,
        hints,
        calls,
    })
}

/// The ops that leave expression `slot` of instruction `index` in register
/// `dst`.
///
/// **What compiles natively is [`native_shape`]'s answer, and nothing here
/// restates it** -- that function is the enumeration, and a second copy of the
/// list in prose is one that stops agreeing with it. Everything it declines --
/// a `.name` is one such expression -- is evaluated by `eval.rs`
/// through [`Op::EvalExpr`], which is trace-identical to what the tree-walker
/// does with the same expression because it is the same call, and stays
/// identical because nothing this region emits sits between one evaluation and
/// the next.
///
/// **`expr` is the whole of the instruction's expression at `slot`, and the
/// choice is taken for the whole of it.** An expression holding one node with
/// no op falls to `EvalExpr` entire, however much of the rest of it would have
/// compiled.
///
/// **[`NodePath::ROOT`] is where the address starts**, because `expr` *is*
/// slot `slot`'s own expression: the route down to it has no steps in it, and
/// every step below is one [`native_shape`] and [`push_native`] take together
/// as they descend.
#[expect(
    clippy::too_many_arguments,
    reason = "the emission sinks, the plan a read resolves its slot against, and the address an \
              EvalExpr or a call op has to name"
)]
fn push_value<'a>(
    ops: &mut Vec<Op>,
    consts: &mut Constants<'a>,
    registers: &mut Registers,
    hints: &mut Hints,
    calls: &mut Calls,
    plan: &Plan,
    expr: &'a Expr,
    index: u32,
    slot: u32,
    dst: u16,
) -> Result<(), ChunkTooLarge> {
    if native_shape(expr, Some(NodePath::ROOT)) {
        push_native(
            ops,
            consts,
            registers,
            hints,
            calls,
            plan,
            expr,
            index,
            slot,
            Some(NodePath::ROOT),
            dst,
        )
    } else {
        ops.push(Op::EvalExpr { index, slot, dst });
        Ok(())
    }
}

/// Whether `expr` compiles to native ops entire, with `eval.rs` not entered
/// for any part of it.
///
/// **Asked once, at the whole expression, and it is what licenses
/// [`push_native`]'s own unreachable arm.** The recursion below descends into
/// an operator's operands, and a subexpression has nowhere to fall back to: an
/// [`Op::EvalExpr`] names an expression *slot* of an instruction, so emitting
/// one for an operand would re-evaluate the whole instruction's expression
/// instead of that operand. So the decision is taken for the whole tree before
/// anything is emitted, and an expression with one unpromotable node anywhere
/// inside it stays one `EvalExpr`.
///
/// **`path` is `expr`'s own address**, the route [`push_native`] writes into
/// an op emitted for this node, and `None` is a node the width of a
/// [`NodePath`] does not reach. **Only the call arm reads it.** A call's op
/// names the node it sits at, so a call with no address has no op; every other
/// shape here is computed from registers and is addressed by nothing, so a
/// depth past the width costs it nothing and a call-free expression promotes
/// however deep it runs.
fn native_shape(expr: &Expr, path: Option<NodePath>) -> bool {
    match &expr.kind {
        ExprKind::Literal(_)
        | ExprKind::Constant(_)
        | ExprKind::Variable(_)
        | ExprKind::Stem(_)
        | ExprKind::Compound(_) => true,
        // A call needs an address, and a node past the width has none. The
        // whole slot then falls to `Op::EvalExpr`, which is the answer it had
        // before there was an address at all.
        ExprKind::Call { .. } => path.is_some(),
        ExprKind::Binary { op, left, right } => {
            is_native_binary(*op)
                && native_shape(left, descend(path, false))
                && native_shape(right, descend(path, true))
        }
        // No operator condition beside the operand's, unlike the arm above:
        // `Interp::apply_prefix` matches `PrefixOp` exhaustively, so a variant
        // added to that enum is a compile error there rather than an operator
        // silently promoted here.
        ExprKind::Prefix { operand, .. } => native_shape(operand, descend(path, false)),
        _ => false,
    }
}

/// The address of one child of the node addressed by `path`: the right child
/// when `right`, and the left or only one otherwise.
///
/// `None` for a child the width does not reach, and `None` propagated from a
/// parent that had no address either -- the two are the same answer here,
/// because what a caller does with it is the same.
///
/// The `right` flag is the one `Interp::chunk_node_at` reads back, and the
/// children it names are the ones that descent walks: a binary operator's two
/// operands and a prefix operator's only one.
fn descend(path: Option<NodePath>, right: bool) -> Option<NodePath> {
    path?.child(right)
}

/// The ops that leave `expr` -- which [`native_shape`] has already accepted --
/// in register `dst`.
///
/// **Each split is what the expression is rather than what is convenient.** A
/// literal's value is bytes the node already carries, so it becomes a native
/// [`Op::Const`] against the interned table plus the `>L>` line that loading it
/// owes. A bare symbol's value is in a frame slot, so it becomes a native
/// [`Op::Load`] plus the `>V>` line that reading it owes. A binary operator's
/// value is computed from its two operands' registers, so it becomes their ops
/// followed by [`Op::Arith`] or [`Op::Binary`] plus the `>O>` line that
/// applying it owes, and a prefix operator's is the same with one operand,
/// [`Op::Prefix`] and the `>P>` line that operator's own trace is. A call's
/// value is what running it answers, so it becomes [`Op::CallExpr`] plus the
/// `>F>` line that owes -- and that op is addressed rather than computed,
/// which is why `index`, `slot` and `path` come down here at all.
#[expect(
    clippy::too_many_arguments,
    reason = "the emission sinks, the plan a read resolves its slot against, and the address a \
              call op has to name"
)]
fn push_native<'a>(
    ops: &mut Vec<Op>,
    consts: &mut Constants<'a>,
    registers: &mut Registers,
    hints: &mut Hints,
    calls: &mut Calls,
    plan: &Plan,
    expr: &'a Expr,
    index: u32,
    slot: u32,
    path: Option<NodePath>,
    dst: u16,
) -> Result<(), ChunkTooLarge> {
    match &expr.kind {
        ExprKind::Literal(bytes) => {
            ops.push(Op::Const {
                dst,
                konst: consts.intern(bytes)?,
            });
            // Behind the load rather than in front of it, because `eval.rs`
            // emits this line post-order, with the value in hand.
            ops.push(Op::TraceLiteral { src: dst });
        }
        // A constant symbol's value is its own upcased spelling, which lives
        // in the symbol table rather than in the node -- so the symbol travels
        // in the op and the spelling is read at run time, where the table is
        // in hand. The echo behind it is the literal's, because the line is the
        // same `>L>`.
        ExprKind::Constant(id) => {
            ops.push(Op::LoadConstant { symbol: *id, dst });
            ops.push(Op::TraceLiteral { src: dst });
        }
        // The three bare-symbol reads. **`ExprKind::DotVariable` and
        // `ExprKind::VariableReference` are not among them and are not
        // reads**: they trace `>E>` and `>O>` rather than `>V>`.
        ExprKind::Variable(id) => push_read(ops, plan, SymbolRead::Simple, *id, dst),
        ExprKind::Stem(id) => push_read(ops, plan, SymbolRead::Stem, *id, dst),
        ExprKind::Compound(id) => push_read(ops, plan, SymbolRead::Compound, *id, dst),
        ExprKind::Binary { op, left, right } => {
            // **The left operand lands in `dst` itself and only the right one
            // takes a register of its own**, which is what keeps a chain's
            // register cost at two however long it runs: `za + zb + zc + zd`
            // is left-nested, so each operator writes its result back into the
            // register the next one reads as its left. Both sources are read
            // before the destination is written, which is the whole of what
            // makes the aliasing safe.
            push_native(
                ops,
                consts,
                registers,
                hints,
                calls,
                plan,
                left,
                index,
                slot,
                descend(path, false),
                dst,
            )?;
            let mark = registers.mark();
            let rhs = registers.alloc()?;
            push_native(
                ops,
                consts,
                registers,
                hints,
                calls,
                plan,
                right,
                index,
                slot,
                descend(path, true),
                rhs,
            )?;
            // **Only arithmetic takes a hint slot**, because only arithmetic
            // has a second path for one to choose. `Chunk::hints` is dense over
            // the ops that can specialise, so a slot reserved for an operator
            // that never reads one would shift every later arithmetic site's
            // index by one and hand it somebody else's decision.
            ops.push(if is_arithmetic(*op) {
                Op::Arith {
                    op: *op,
                    hint: hints.reserve()?,
                    lhs: dst,
                    rhs,
                    dst,
                }
            } else {
                Op::Binary {
                    op: *op,
                    lhs: dst,
                    rhs,
                    dst,
                }
            });
            // Behind the operation rather than in front of it, because
            // `eval.rs` emits this line post-order, with the value in hand --
            // so an inner operator's line precedes the outer one's.
            ops.push(Op::TraceOperator { op: *op, src: dst });
            // Held until the operation has run: the right operand's register
            // is live right up to it, and a release any earlier would hand it
            // out to the next operand of an enclosing operator.
            registers.release(mark);
        }
        ExprKind::Prefix { op, operand } => {
            // **The operand lands in `dst` itself and no register is taken**,
            // which is the one-operand case of the discipline the binary arm
            // above states: the operand is read before the destination is
            // written, so a chain of prefixes costs the registers its innermost
            // term does and no more.
            push_native(
                ops,
                consts,
                registers,
                hints,
                calls,
                plan,
                operand,
                index,
                slot,
                descend(path, false),
                dst,
            )?;
            ops.push(Op::Prefix {
                op: *op,
                src: dst,
                dst,
            });
            // Behind the operation rather than in front of it, because
            // `eval.rs` emits this line post-order, with the value in hand --
            // so an inner operator's line precedes the outer one's.
            //
            // `Op::TracePrefix` and not `Op::TraceOperator`: a prefix operator
            // traces `>P>`, which is a different line from the `>O>` every
            // binary operator traces.
            ops.push(Op::TracePrefix { op: *op, src: dst });
        }
        // **A call takes its own op wherever it sits**, which is
        // [`Op::CallExpr`], and the only thing that op changes about running
        // the call is that the resolution comes from a site instead of being
        // made afresh. What it needs and no other arm here does is an address:
        // the value is not computed from operand registers, so the driver goes
        // back to the node to find the target and the arguments.
        ExprKind::Call { .. } => {
            let slot = u16::try_from(slot).map_err(|_| ChunkTooLarge {
                what: "expression slots past u16",
            })?;
            let path = path.expect("native_shape accepts a call only where an address reaches it");
            ops.push(Op::CallExpr {
                index,
                slot,
                path,
                site: calls.reserve()?,
                dst,
            });
            // Behind the call rather than in front of it, because `eval.rs`
            // emits this line post-order, with the value in hand.
            ops.push(Op::TraceFunction {
                index,
                slot,
                path,
                src: dst,
            });
        }
        _ => unreachable!("this descends only into an expression native_shape accepted"),
    }
    Ok(())
}

/// The slot one assignment *target* resolves to, or [`PlanSlot::UNRESOLVED`]
/// for a target that does not write a slot by name.
///
/// **A simple variable only, and the two it declines are declined for
/// different reasons.**
/// A compound target resolves a tail key at the write site and mutates one
/// tail through `stem_set`, which writes the *stem's* slot and not the
/// symbol's own -- the same asymmetry [`PlanSlot`]'s own doc comment records
/// for a compound *read* -- so the number this could answer with is one
/// `Interp::assign_expr_target`'s compound arm has no use for, and that arm
/// asserts it was not given one.
/// A bare stem target **does** write the symbol's own slot, and the number is
/// available: `Plan::bind` binds a stem-shaped spelling whole, so
/// `by_symbol[id]` holds it. It is `UNRESOLVED` here because `Plan::bind`
/// records that same slot on the entry `Code::compound` hands back, and
/// `assign_expr_target`'s stem arm reads it from there -- which answers on
/// the tree-walker as well, where an op answers only for the compiled engine.
/// Carrying it here as well would be a second source for one number.
///
/// The map is the plan's `by_symbol`, which is what `Code::slots` is a view of
/// at run time, so the compiled answer and `Interp::slot_of`'s are one
/// resolution made at two times ([`push_read`]'s own doc comment has the
/// argument in full).
///
/// [`PlanSlot`]: super::PlanSlot
fn write_slot(plan: &Plan, target: &Expr) -> PlanSlot {
    match &target.kind {
        ExprKind::Variable(id) => plan
            .slot_for_symbol(*id)
            .map_or(PlanSlot::UNRESOLVED, PlanSlot::of),
        _ => PlanSlot::UNRESOLVED,
    }
}

/// The two ops one bare-symbol read is: the load, and the `>V>`/`>C>` line
/// that reading it owes.
///
/// **The slot comes from the plan's own `by_symbol` map**, which is the map
/// `Code::slots` is a view of at run time, so the compiled answer and the
/// run-time one are one resolution made at two times rather than two
/// resolutions. `Plan::build` is exhaustive over the body, so a symbol read by
/// an instruction of it is bound -- but that is a property of another function,
/// and a read whose symbol is not in the map simply resolves its own slot the
/// way every read did before this op existed.
///
/// **A compound is never resolved here**, and it is the case that makes the
/// unresolved arm ordinary rather than defensive: what a compound read goes
/// through is the *stem's* slot and a tail key worked out at the read site, and
/// `Plan::note_compound_name` binds those by name with no `SymbolId` to hang
/// them on ([`PlanSlot`]'s own doc comment).
///
/// [`PlanSlot`]: super::PlanSlot
fn push_read(ops: &mut Vec<Op>, plan: &Plan, read: SymbolRead, symbol: SymbolId, dst: u16) {
    let at = match read {
        SymbolRead::Simple | SymbolRead::Stem => plan
            .slot_for_symbol(symbol)
            .map_or(PlanSlot::UNRESOLVED, PlanSlot::of),
        SymbolRead::Compound => PlanSlot::UNRESOLVED,
    };
    ops.push(Op::Load {
        symbol,
        read,
        at,
        dst,
    });
    // Behind the load rather than in front of it, because `eval.rs` emits
    // these lines post-order, with the value in hand.
    ops.push(Op::TraceRead {
        symbol,
        read,
        src: dst,
    });
}

/// Whether a promoted clause of `instruction` echoes under `trace`.
///
/// The same question `Interp::tracing_clause` asks at run time, through the
/// same [`ChunkTrace::echoes`], so the compiled answer and the run-time one
/// cannot disagree. `is_label` is read off the instruction rather than assumed
/// false: nothing here says a `LABEL` can never be promoted, and a rule that
/// held only because of what happens to be promoted today is the kind that
/// stops holding without anything going red.
fn echoes(trace: ChunkTrace, instruction: &Instruction) -> bool {
    trace.echoes(matches!(instruction.kind, InstructionKind::Label { .. }))
}

/// Pushes the clause echo op, if `echo`, as the **first** op of the region
/// that follows -- the position the tree-walker's own clause unit echoes at,
/// before anything the clause computes.
fn push_echo(ops: &mut Vec<Op>, echo: bool, index: u32) {
    if echo {
        ops.push(Op::TraceClause { index });
    }
}

/// Emits the ops that go in front of one instruction, innermost first.
///
/// **Reversed, because registration order is outermost first.** A construct
/// registers its own entry when *it* compiles, and an enclosing construct
/// compiles before the one nested inside it; control leaves the inner branch
/// first, so the inner boundary runs first. The one entry that can carry a
/// jump is the outermost, which reversal puts last -- and it has to be last,
/// since every op after a jump at the same position is unreachable.
fn emit_before(
    ops: &mut Vec<Op>,
    patches: &mut Vec<Patch>,
    before: &mut [Before],
) -> Result<(), ChunkTooLarge> {
    before.reverse();
    // **A `SELECT` branch's close goes in front of everything else here**,
    // which is where the driver used to ask the question: it tested for an
    // ended frame before fetching the op at all, so a `WHEN` whose body is an
    // `IF` left the branch -- and jumped to the `SELECT`'s own end -- without
    // ever reaching that `IF`'s [`Op::EndBranch`]. Emitting the close second
    // runs that boundary first, which is a clause boundary that did not run
    // before. The sort is stable, so nested branches keep the innermost-first
    // order the reverse above gives them.
    before.sort_by_key(|entry| match entry {
        Before::SelectBranchEnd => 0,
        Before::ThenEnd { .. } | Before::EnterOtherwise(_) => 1,
    });
    debug_assert!(
        before
            .iter()
            .rev()
            .skip(1)
            .all(|entry| !matches!(entry, Before::ThenEnd { resume: Some(_) })),
        "an op in front of an instruction carries a jump with more ops behind it, which \
         nothing would reach"
    );
    for entry in before.iter() {
        match entry {
            Before::ThenEnd { resume } => {
                ops.push(Op::EndBranch);
                if let Some(resume) = resume {
                    let op = op_index(ops)?;
                    ops.push(Op::Jump { target: 0 });
                    patches.push(Patch {
                        op,
                        target: *resume,
                        kind: PatchKind::Resume,
                    });
                }
            }
            Before::SelectBranchEnd => ops.push(Op::EndWhen),
            Before::EnterOtherwise(select) => ops.push(Op::EnterOtherwise {
                select: instruction_index(*select)?,
            }),
        }
    }
    Ok(())
}

/// Records that `mark` is dead by the time the instruction this slot belongs to
/// is reached, keeping **the lowest** mark any construct wants released there.
///
/// Two constructs can end at the same instruction -- a `DO` block closing one
/// instruction before the `SELECT` whose `WHEN` holds it -- and by that point
/// both have finished, so every register above the lower of the two marks is
/// dead. Keeping the higher one instead would leave the outer construct's
/// registers allocated for the rest of the body, which costs reservation
/// without being wrong; keeping the lower one is what actually hands them back.
fn release_to(slot: &mut Option<Mark>, mark: Mark) {
    let lowest = match *slot {
        Some(existing) if existing.0 <= mark.0 => existing,
        _ => mark,
    };
    *slot = Some(lowest);
}

/// Rewrites the [`Op::Clause`] at `at` so that its `end` is the position one
/// past everything pushed since -- the region's own end.
///
/// **A region is opened, emitted, and then closed from the stream's own length**,
/// rather than opened with a length worked out ahead of the ops it describes. A
/// length computed ahead is the emitted shape written twice inside one arm, once
/// as pushes and once as arithmetic, and for a region whose ops depend on the
/// node -- a `DO`/`LOOP` header, which is one group per expression written -- the
/// second copy is a second traversal that has to agree with the first. Closing
/// from the length cannot disagree with what was pushed.
fn close_region(ops: &mut [Op], at: u32) -> Result<(), ChunkTooLarge> {
    let end = op_index(ops)?;
    match &mut ops[at as usize] {
        Op::Clause { end: slot, .. } => *slot = end,
        _ => unreachable!("close_region is given the index of the Clause op it closes"),
    }
    Ok(())
}

/// Every clause's [`ClausePosition`], or an empty table where the plan cannot
/// supply one for each.
///
/// **All or nothing, so the driver's own check is one test rather than one
/// per clause.** A plan built without source has no line table at all, and a
/// line or indent past `u32` has no entry that would fit; either way the
/// driver reads the plan as it did before this table existed.
fn clause_positions(body: &CodeBody, plan: &Plan) -> Box<[ClausePosition]> {
    let len = body.instructions.len();
    if plan.lines.len() != len || plan.indents.len() != len {
        return Box::default();
    }
    let mut positions = Vec::with_capacity(len);
    for (line, indent) in plan.lines.iter().zip(plan.indents.iter()) {
        let (Ok(line), Ok(indent)) = (u32::try_from(*line), u32::try_from(*indent)) else {
            return Box::default();
        };
        positions.push(ClausePosition { line, indent });
    }
    positions.into_boxed_slice()
}

/// The index the next op will be pushed at, refused rather than wrapped.
fn op_index(ops: &[Op]) -> Result<u32, ChunkTooLarge> {
    u32::try_from(ops.len()).map_err(|_| ChunkTooLarge { what: "op stream" })
}

/// An instruction index as an op payload, refused rather than wrapped.
fn instruction_index(index: usize) -> Result<u32, ChunkTooLarge> {
    u32::try_from(index).map_err(|_| ChunkTooLarge { what: "op stream" })
}

/// **No `Generic` op sits inside a [`Op::Clause`] region.**
///
/// It runs a whole clause through `Interp::step_in_temps_frame`, which echoes
/// the clause itself -- and the echo is not idempotent, so a clause already
/// opened by a `Clause` op would echo twice. The compiler is where this can be
/// checked at all: the driver sees one op at a time and cannot tell an op it
/// reached by falling into a region from one it jumped to.
///
/// An op that runs clauses belonging to something other than this region's own
/// instruction is not one of these and is deliberately not checked for:
/// [`Op::LoopRun`]'s are its construct's body, stepped by a nested driver entry,
/// and [`Op::Call`]'s are a nested activation's, stepped by a driver of its
/// own.
///
/// **`Generic` and not "any op that opens a clause", which the name used to
/// say.** A nested [`Op::Clause`] inside a region is ordinary and not a
/// defect: an `IF`'s region spans its branches, and each branch's clauses are
/// regions of their own. So this scan names the one op whose presence is
/// wrong, and the two exclusions above say why the others are not it.
///
/// An unconditional `assert!` rather than a `debug_assert!`, so the release
/// build carries the same guarantee. It is one linear scan per body, once,
/// against a compile that has already walked the same list.
fn assert_clause_regions_hold_no_generic_op(ops: &[Op]) {
    for (at, op) in ops.iter().enumerate() {
        let Op::Clause { end, .. } = op else {
            continue;
        };
        for inside in ops[at + 1..(*end as usize).min(ops.len())].iter() {
            assert!(
                !matches!(inside, Op::Generic { .. }),
                "a Clause region at op {at} holds an op that opens a clause of its own, \
                 so the clause would be echoed twice"
            );
        }
    }
}

/// **Every [`Op::TraceClause`] is the first op of a [`Op::Clause`] region**,
/// which is both halves of that op's own contract at once.
///
/// The echo reads `Interp::clause_state`'s value indent and the instruction
/// the enclosing region opened, so an echo the driver reached outside a region
/// prints against whatever the last clause left there. And the tree-walker
/// echoes a clause before it computes anything, so an echo anywhere but first
/// puts the clause's `*-*` line after a value line it must precede.
///
/// Checking it needs only the op before: a `Clause` whose `end` is past this
/// position is a region that has just opened and has emitted nothing else yet.
///
/// An unconditional `assert!` for [`assert_clause_regions_hold_no_generic_op`]'s
/// reason, and it is the same linear scan's worth of work.
fn assert_trace_ops_open_a_clause_region(ops: &[Op]) {
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
///
/// Either load, because the two produce the same `>L>` line and this op is the
/// echo for both.
///
/// `eval.rs` emits a literal's `>L>` line post-order, with the value in hand,
/// so an echo in front of its load prints whatever the register held before --
/// and an echo further behind it prints after value lines that the oracle puts
/// after this one. Reading a *different* register is the third way the pair
/// comes apart, and it is the one no ordering check would see: the line would
/// be in the right place with the wrong value in it.
///
/// An unconditional `assert!` for [`assert_clause_regions_hold_no_generic_op`]'s
/// reason, and it is the same linear scan's worth of work.
fn assert_literal_echoes_follow_their_load(ops: &[Op]) {
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
///
/// [`assert_literal_echoes_follow_their_load`]'s two failures, plus one a
/// literal's echo cannot have: the tag. `>V>` names the symbol that was read,
/// so an echo carrying another op's symbol prints the right value under the
/// wrong name, and a compound's `>C>` line resolves *that* symbol's tail --
/// which means a `read` that disagreed with the load's would emit a line the
/// oracle prints nowhere, or drop one it prints.
///
/// An unconditional `assert!` for [`assert_clause_regions_hold_no_generic_op`]'s
/// reason, and it is the same linear scan's worth of work.
fn assert_read_echoes_follow_their_load(ops: &[Op]) {
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
///
/// [`assert_read_echoes_follow_their_load`]'s three failures in this op's own
/// terms. The position is what puts the line where `eval.rs` puts it -- and a
/// chain emits one operation/`TraceOperator` pair per operator, so an echo one
/// place out prints the inner operator's line after the outer one's. The
/// register is what makes it the right value, since a chain reuses `dst` for
/// every operator in it. The operator is the tag: `>O>` names the operator that
/// was applied, so an echo carrying another op's prints the right value under
/// the wrong sign.
///
/// An unconditional `assert!` for [`assert_clause_regions_hold_no_generic_op`]'s
/// reason, and it is the same linear scan's worth of work.
fn assert_operator_echoes_follow_their_op(ops: &[Op]) {
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
///
/// The failures [`assert_operator_echoes_follow_their_op`] checks for -- the
/// position, the register and the tag -- in this op's own terms. The tag is the
/// one worth spelling out again: `>P>` carries the prefix operator's own
/// spelling, so an echo behind the wrong [`Op::Prefix`], or one carrying a
/// `PrefixOp` that operation does not, lands in the right place with the wrong
/// sign in it.
///
/// An unconditional `assert!` for [`assert_clause_regions_hold_no_generic_op`]'s
/// reason, and it is the same linear scan's worth of work.
fn assert_prefix_echoes_follow_their_op(ops: &[Op]) {
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
///
/// The failures [`assert_operator_echoes_follow_their_op`] checks for -- the
/// position, the register and the tag -- in this op's own terms, and the tag
/// is a different thing here. `>F>` carries no operator; what it carries is
/// the node, because `trace_intermediate` reads the expression at the address
/// to decide the tag it prints under. So the pair has to agree on `index`,
/// `slot` **and** `path`, and an echo addressing some other node lands the
/// right value on the wrong line -- or, where that node is not a call at all,
/// raises `Loud::call_op_off_its_node` at run time from an op `compile` was
/// content with.
///
/// **The width of a call op's address is what makes this worth asserting.** An
/// address that is only ever [`NodePath::ROOT`] agrees by construction; one
/// that carries a route agrees only because [`push_native`]'s call arm was
/// written to push the same one twice.
///
/// An unconditional `assert!` for [`assert_clause_regions_hold_no_generic_op`]'s
/// reason, and it is the same linear scan's worth of work.
fn assert_call_echoes_follow_their_op(ops: &[Op]) {
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
                matches!(
                    before,
                    Op::CallExpr { index, slot, path, dst, .. }
                        if index == echoed && slot == echoed_slot && path == echoed_path
                            && dst == src
                )
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
///
/// The other echo ops here look one place *back*, at the op that computed their
/// register. This one cannot: a header value is its expression's own ops, so
/// what sits in front of a `>K>` is the last of those -- a load, an operation,
/// a call, each with its own echo behind it, or one [`Op::EvalExpr`] where the
/// slot declined. What is fixed is the pair on the other side: the echo and the
/// validation of one header value, in that order and on one register.
///
/// Both halves of that pairing are measured on the oracle and recorded in
/// `ir_dual_cases/loop-header-boundaries`: `do i = 1 to 'a' by 2` under
/// `trace r` prints `>K>   "TO" => "a"` for the very value that then raises
/// 41.1, so the echo precedes the validation; and `do i = 1 to 'a' by zf()`
/// never calls `zf`, so the validation precedes the next value's evaluation.
/// An echo separated from its value by anything is one of those two orders
/// broken.
///
/// **What this adds is the shape of the failure, not coverage, and that is
/// measured rather than assumed.** Emitting the echo in front of the slot's own
/// ops instead of behind them, with this check removed, moves the population,
/// loop-shape and case-file sweeps and `trace_oracle`'s control-variable
/// transcripts -- because the echo then reads a register nothing has written
/// and prints `>K>   "TO" => "The NIL object"`. What this turns that into is a
/// refusal at compile time naming the op.
///
/// An unconditional `assert!` for [`assert_clause_regions_hold_no_generic_op`]'s
/// reason, and it is the same linear scan's worth of work.
fn assert_keyword_echoes_precede_their_value(ops: &[Op]) {
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

/// **Every index-bearing op inside a [`Op::Clause`] region names that region's
/// own clause.**
///
/// That is what lets the driver read the instruction off the region once and
/// hand it to every op inside it, instead of looking each op's own `index` up
/// again -- measured, three bounds-checked lookups of the same instruction per
/// promoted clause where the tree-walker makes one. Each op keeps its `index`,
/// because that is what says which instruction the op belongs to and it is what
/// a golden stream is read against; nothing at run time resolves it.
///
/// The property holds by construction -- every one of these ops is emitted from
/// the arm of the instruction whose region it is -- and that is exactly the kind
/// of claim that stops holding without anything going red.
///
/// **What this adds is the shape of the failure, not coverage, and that is
/// measured rather than assumed.** Making an assignment's value op name the
/// instruction after it reddens six tests with this check removed -- the
/// dual-engine population sweep, the branch, loop and case-file harnesses and the
/// known-divergence table all notice -- so the suite already sees a mis-indexed
/// op. What it sees is a divergence in a program's output; what this turns that
/// into is a refusal at compile time naming the op and both instructions.
///
/// An unconditional `assert!` for [`assert_clause_regions_hold_no_generic_op`]'s
/// reason, and it is the same linear scan's worth of work.
fn assert_region_ops_name_their_clause(ops: &[Op]) {
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
                | Op::Message { index }
                | Op::Expose { index }
                | Op::Escape { index }
                | Op::CallExpr { index, .. }
                | Op::TraceFunction { index, .. }
                | Op::Condition { index, .. }
                | Op::LoopRun { index }
                | Op::LoopNext { index }
                | Op::Signal { index, .. }
                | Op::Parse { index, .. } => Some(*index),
                Op::Generic { .. }
                | Op::TraceKeyword { .. }
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

// Test-only instrumentation: how many times `compile` has actually run. The
// chunk-cache test (`golden_tests.rs`) needs this to tell "the chunk cache
// compiled the body once" apart from "the second lookup happened not to
// fail" -- an implementation that recompiles on every call and returns an
// equal-looking `Chunk` would still pass a check that only inspects the
// result, and a `thread_local` counter is what lets the test inspect the
// call count instead.
#[cfg(test)]
thread_local! {
    static COMPILE_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
fn count_compile_call() {
    COMPILE_CALLS.with(|calls| calls.set(calls.get() + 1));
}

#[cfg(test)]
pub(crate) fn compile_calls() -> usize {
    COMPILE_CALLS.with(|calls| calls.get())
}

#[cfg(test)]
mod tests {
    use rexx_parse::{SymbolId, SymbolTable};

    use super::{
        Op, PlanSlot, Registers, SymbolRead, assert_clause_regions_hold_no_generic_op,
        assert_literal_echoes_follow_their_load, assert_read_echoes_follow_their_load,
        assert_region_ops_name_their_clause, assert_trace_ops_open_a_clause_region,
    };

    /// Two sibling clauses reuse the same registers, and a clause nested
    /// inside another's mark does not.
    ///
    /// This is the discipline the plan's Decisions section fixes, and the one
    /// the withdrawn whole-chunk monotonic counter fails: under that counter
    /// the second sibling would get register 2, and the high-water mark would
    /// grow with the body's length rather than with its depth.
    #[test]
    fn a_released_register_is_handed_out_again_and_a_nested_one_is_not() {
        let mut registers = Registers::new();

        let outer = registers.mark();
        assert_eq!(registers.alloc().expect("in range"), 0);
        let inner = registers.mark();
        assert_eq!(
            registers.alloc().expect("in range"),
            1,
            "a register allocated while an enclosing one is live must not reuse it"
        );
        registers.release(inner);
        assert_eq!(
            registers.alloc().expect("in range"),
            1,
            "releasing to the inner mark hands the same register out again"
        );
        registers.release(outer);
        assert_eq!(
            registers.alloc().expect("in range"),
            0,
            "releasing to the outer mark hands out the enclosing register too"
        );
    }

    /// The high-water mark is the deepest the stack ever reached, not the
    /// number of registers handed out and not what is live at the end.
    ///
    /// Both halves matter: a `Chunk::registers` taken from the live count
    /// would reserve nothing for a body whose every clause released, and one
    /// taken from the number of `alloc` calls would reserve four here.
    #[test]
    fn the_high_water_mark_is_the_deepest_the_stack_reached() {
        let mut registers = Registers::new();
        assert_eq!(registers.high_water(), 0, "an empty body reserves nothing");

        for _ in 0..2 {
            let mark = registers.mark();
            registers.alloc().expect("in range");
            registers.alloc().expect("in range");
            registers.release(mark);
        }

        assert_eq!(registers.high_water(), 2);
    }

    /// A body needing more than `u16::MAX` registers live at once is refused
    /// rather than wrapped, which is the register half of the one error
    /// `compile` has (the plan's Decisions section: "the compiler has one
    /// error, and it is a machine width rather than a language construct").
    ///
    /// Driven through the allocator directly: no program this crate parses
    /// reaches that many live registers, so a test that tried to write one
    /// would be measuring the parser instead.
    ///
    /// The last index handed out is `u16::MAX - 1` rather than `u16::MAX`,
    /// because it is the *count* that has to fit: a region of `u16::MAX + 1`
    /// registers is what a chunk could not record the size of.
    #[test]
    fn a_register_file_wider_than_u16_is_refused() {
        let mut registers = Registers::new();
        for expected in 0..u16::MAX {
            assert_eq!(registers.alloc().expect("in range"), expected);
        }
        assert_eq!(registers.high_water(), u16::MAX);
        assert_eq!(
            registers.alloc().expect_err("one past the last index").what,
            "registers"
        );
    }

    /// The assertion `compile` runs over what it emitted, shown firing on the
    /// arrangement it forbids.
    ///
    /// Built here rather than by mutating the compiler, so the witness stays
    /// in the tree: a `Generic` op inside a `Clause` region would open a
    /// second clause for an instruction whose clause is already open, and
    /// `step_in_temps_frame` echoes the clause on the way in, so the echo
    /// would appear twice.
    #[test]
    #[should_panic(expected = "holds an op that opens a clause of its own")]
    fn a_generic_op_inside_a_clause_region_is_refused() {
        assert_clause_regions_hold_no_generic_op(&[
            Op::Clause { index: 0, end: 3 },
            Op::EvalExpr {
                index: 0,
                slot: 0,
                dst: 0,
            },
            Op::Generic { index: 1 },
        ]);
    }

    /// The neighbouring arrangement that must stay accepted: the same ops
    /// with the `Generic` one *past* the region's end.
    ///
    /// Without this the assertion above is satisfied by a check that refuses
    /// every stream, which is the shape that would make every `IF` a refusal
    /// and every body a tree-walker body.
    #[test]
    fn a_generic_op_after_a_clause_region_is_accepted() {
        assert_clause_regions_hold_no_generic_op(&[
            Op::Clause { index: 0, end: 3 },
            Op::EvalExpr {
                index: 0,
                slot: 0,
                dst: 0,
            },
            Op::JumpUnless { reg: 0, target: 3 },
            Op::Generic { index: 1 },
        ]);
    }

    /// A trace op that is inside a region but not at its head, which is the
    /// half of [`Op::TraceClause`]'s contract that a check for "inside a
    /// region" alone would miss.
    ///
    /// The arrangement is exactly what emitting the echo after the condition
    /// produces, and it prints the clause's `*-*` line after the `>>>` line
    /// that condition traces.
    #[test]
    #[should_panic(expected = "not the first op of a Clause region")]
    fn a_trace_op_after_the_regions_first_op_is_refused() {
        assert_trace_ops_open_a_clause_region(&[
            Op::Clause { index: 0, end: 4 },
            Op::EvalExpr {
                index: 0,
                slot: 0,
                dst: 0,
            },
            Op::TraceClause { index: 0 },
            Op::JumpUnless { reg: 0, target: 4 },
        ]);
    }

    /// A trace op with no region open at all, the other way the contract is
    /// broken: the echo would read whatever value indent the last clause left
    /// behind.
    #[test]
    #[should_panic(expected = "not the first op of a Clause region")]
    fn a_trace_op_outside_a_clause_region_is_refused() {
        assert_trace_ops_open_a_clause_region(&[
            Op::Generic { index: 0 },
            Op::TraceClause { index: 1 },
        ]);
    }

    /// A literal echo in **front** of the load it reads, which is the
    /// arrangement that prints whatever the register held before the literal
    /// reached it -- and the only line of the transcript it moves is the one
    /// carrying the value.
    #[test]
    #[should_panic(expected = "does not follow the load of the register it reads")]
    fn a_literal_echo_in_front_of_its_load_is_refused() {
        assert_literal_echoes_follow_their_load(&[
            Op::Clause { index: 0, end: 4 },
            Op::TraceLiteral { src: 0 },
            Op::Const { dst: 0, konst: 0 },
            Op::Say {
                index: 0,
                src: Some(0),
            },
        ]);
    }

    /// A literal echo that reads a register the op in front of it did not
    /// write, which no ordering check alone would see: the line lands in the
    /// right place with the wrong value in it.
    #[test]
    #[should_panic(expected = "does not follow the load of the register it reads")]
    fn a_literal_echo_reading_another_register_is_refused() {
        assert_literal_echoes_follow_their_load(&[
            Op::Clause { index: 0, end: 4 },
            Op::Const { dst: 0, konst: 0 },
            Op::TraceLiteral { src: 1 },
            Op::Say {
                index: 0,
                src: Some(0),
            },
        ]);
    }

    /// The neighbouring arrangement that must stay accepted, without which both
    /// refusals above are satisfied by a check that refuses every stream
    /// carrying a literal at all -- which would make every such body a refusal.
    #[test]
    fn a_literal_echo_behind_its_own_load_is_accepted() {
        assert_literal_echoes_follow_their_load(&[
            Op::Clause { index: 0, end: 4 },
            Op::Const { dst: 0, konst: 0 },
            Op::TraceLiteral { src: 0 },
            Op::Say {
                index: 0,
                src: Some(0),
            },
        ]);
    }

    /// A read echo in **front** of the load it reads, which prints whatever the
    /// register held before the read reached it.
    #[test]
    #[should_panic(expected = "does not follow the load of the symbol and register it names")]
    fn a_read_echo_in_front_of_its_load_is_refused() {
        let (zv, _zw) = two_symbols();
        assert_read_echoes_follow_their_load(&[
            Op::Clause { index: 0, end: 4 },
            Op::TraceRead {
                symbol: zv,
                read: SymbolRead::Simple,
                src: 0,
            },
            Op::Load {
                symbol: zv,
                read: SymbolRead::Simple,
                at: PlanSlot::UNRESOLVED,
                dst: 0,
            },
            Op::Store {
                index: 0,
                at: PlanSlot::UNRESOLVED,
                src: 0,
            },
        ]);
    }

    /// A read echo reading a register the op in front of it did not write: the
    /// line lands in the right place with the wrong value in it.
    #[test]
    #[should_panic(expected = "does not follow the load of the symbol and register it names")]
    fn a_read_echo_reading_another_register_is_refused() {
        let (zv, _zw) = two_symbols();
        assert_read_echoes_follow_their_load(&[
            Op::Clause { index: 0, end: 4 },
            Op::Load {
                symbol: zv,
                read: SymbolRead::Simple,
                at: PlanSlot::UNRESOLVED,
                dst: 0,
            },
            Op::TraceRead {
                symbol: zv,
                read: SymbolRead::Simple,
                src: 1,
            },
            Op::Store {
                index: 0,
                at: PlanSlot::UNRESOLVED,
                src: 0,
            },
        ]);
    }

    /// A read echo naming a **different symbol** from the load in front of it,
    /// which is the failure a literal's echo cannot have and which neither
    /// ordering nor the register would see: `>V>` is tagged with the name, so
    /// the line prints the right value under the wrong one.
    #[test]
    #[should_panic(expected = "does not follow the load of the symbol and register it names")]
    fn a_read_echo_naming_another_symbol_is_refused() {
        let (zv, zw) = two_symbols();
        assert_read_echoes_follow_their_load(&[
            Op::Clause { index: 0, end: 4 },
            Op::Load {
                symbol: zv,
                read: SymbolRead::Simple,
                at: PlanSlot::UNRESOLVED,
                dst: 0,
            },
            Op::TraceRead {
                symbol: zw,
                read: SymbolRead::Simple,
                src: 0,
            },
            Op::Store {
                index: 0,
                at: PlanSlot::UNRESOLVED,
                src: 0,
            },
        ]);
    }

    /// The same for the read kind, which decides whether the `>C>` line in
    /// front of `>V>` is emitted at all.
    #[test]
    #[should_panic(expected = "does not follow the load of the symbol and register it names")]
    fn a_read_echo_of_another_kind_is_refused() {
        let (zv, _zw) = two_symbols();
        assert_read_echoes_follow_their_load(&[
            Op::Clause { index: 0, end: 4 },
            Op::Load {
                symbol: zv,
                read: SymbolRead::Simple,
                at: PlanSlot::UNRESOLVED,
                dst: 0,
            },
            Op::TraceRead {
                symbol: zv,
                read: SymbolRead::Compound,
                src: 0,
            },
            Op::Store {
                index: 0,
                at: PlanSlot::UNRESOLVED,
                src: 0,
            },
        ]);
    }

    /// The neighbouring arrangement that must stay accepted, without which all
    /// four refusals above are satisfied by a check that refuses every stream
    /// carrying a read at all -- which would make every such body a refusal.
    #[test]
    fn a_read_echo_behind_its_own_load_is_accepted() {
        let (zv, _zw) = two_symbols();
        assert_read_echoes_follow_their_load(&[
            Op::Clause { index: 0, end: 4 },
            Op::Load {
                symbol: zv,
                read: SymbolRead::Simple,
                at: PlanSlot::of(1),
                dst: 0,
            },
            Op::TraceRead {
                symbol: zv,
                read: SymbolRead::Simple,
                src: 0,
            },
            Op::Store {
                index: 0,
                at: PlanSlot::UNRESOLVED,
                src: 0,
            },
        ]);
    }

    /// Two distinct `SymbolId`s, from a table of this test module's own so the
    /// ids are the parser's rather than numbers invented here.
    fn two_symbols() -> (SymbolId, SymbolId) {
        let mut symbols = SymbolTable::default();
        (symbols.intern("ZV"), symbols.intern("ZW"))
    }

    /// A slot too wide for a compiled read's own field is **not** a refusal:
    /// the read still has a correct answer and the run-time path is what
    /// computes it, so the whole body must not fall back to the tree-walker for
    /// something that is only an optimisation.
    ///
    /// The reserved value is one below the width's own maximum, which is what
    /// separates "no slot" from "the last slot that fits".
    #[test]
    fn a_slot_too_wide_for_a_compiled_read_is_unresolved_rather_than_refused() {
        assert_eq!(PlanSlot::of(0).resolved(), Some(0));
        let last = u32::MAX as usize - 1;
        assert_eq!(PlanSlot::of(last).resolved(), Some(last));
        assert_eq!(PlanSlot::of(u32::MAX as usize).resolved(), None);
        assert_eq!(PlanSlot::UNRESOLVED.resolved(), None);
    }

    /// The neighbouring arrangement that must stay accepted, without which
    /// both refusals above are satisfied by a check that refuses every stream
    /// carrying a trace op -- which would make every traced body a refusal.
    #[test]
    fn a_trace_op_at_the_head_of_a_clause_region_is_accepted() {
        assert_trace_ops_open_a_clause_region(&[
            Op::Clause { index: 0, end: 4 },
            Op::TraceClause { index: 0 },
            Op::EvalExpr {
                index: 0,
                slot: 0,
                dst: 0,
            },
            Op::JumpUnless { reg: 0, target: 4 },
        ]);
    }

    /// An op inside a region that names the instruction *next* to the region's
    /// clause, which is the arrangement the driver cannot detect.
    ///
    /// The driver reads the instruction off the region once and hands it to every
    /// op inside it, so an op naming a neighbour is not a lookup that fails: it
    /// silently evaluates the wrong instruction's expression, or writes through
    /// the wrong assignment's target, and every line around it still matches.
    #[test]
    #[should_panic(expected = "names instruction 1 inside the region of clause 0")]
    fn a_region_op_naming_a_neighbouring_instruction_is_refused() {
        assert_region_ops_name_their_clause(&[
            Op::Clause { index: 0, end: 3 },
            Op::EvalExpr {
                index: 1,
                slot: 0,
                dst: 0,
            },
            Op::Store {
                index: 0,
                at: PlanSlot::UNRESOLVED,
                src: 0,
            },
        ]);
    }

    /// The same for the echo op, which carries an index of its own and reaches
    /// a different function with it.
    #[test]
    #[should_panic(expected = "names instruction 2 inside the region of clause 1")]
    fn a_trace_op_naming_a_neighbouring_instruction_is_refused() {
        assert_region_ops_name_their_clause(&[
            Op::Generic { index: 0 },
            Op::Clause { index: 1, end: 4 },
            Op::TraceClause { index: 2 },
            Op::EvalExpr {
                index: 1,
                slot: 0,
                dst: 0,
            },
        ]);
    }

    /// The neighbouring arrangement that must stay accepted: two regions, each
    /// of whose ops names its own clause, with an index-bearing op *outside* any
    /// region naming a third instruction.
    ///
    /// Without this the refusals above are satisfied by a check that refuses
    /// every stream holding more than one instruction index -- and the ops past a
    /// region's end genuinely do name other instructions, which is what
    /// `Op::SelectCaseText` is.
    #[test]
    fn ops_naming_their_own_region_clause_are_accepted() {
        assert_region_ops_name_their_clause(&[
            Op::Clause { index: 0, end: 3 },
            Op::TraceClause { index: 0 },
            Op::Store {
                index: 0,
                at: PlanSlot::UNRESOLVED,
                src: 0,
            },
            Op::SelectCaseText {
                index: 7,
                case: None,
            },
            Op::Clause { index: 1, end: 6 },
            Op::Say {
                index: 1,
                src: Some(0),
            },
        ]);
    }
}
