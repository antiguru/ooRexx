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

use rexx_parse::{CodeBody, Instruction, InstructionKind};

use super::{Chunk, ChunkTooLarge, Op};
use crate::plan::Plan;
use crate::run::{if_targets, otherwise_range};
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
/// `LOOP` becomes [`Op::Loop`], whose body clauses the driver steps; an `IF`
/// becomes a [`Op::Clause`] region that evaluates its condition and jumps; a
/// `SELECT` becomes one such region per listed `WHEN` as well as for its own
/// header, laid out as a scan chain with a frame opened over whichever branch
/// wins; every other instruction becomes [`Op::Generic`]. `plan` is not yet
/// read:
/// nothing compiled here needs a name-to-slot answer, but a task that
/// promotes an assignment reads it to place the assignment's own `EvalExpr`.
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
    _plan: &Plan,
    trace: ChunkTrace,
) -> Result<Chunk, ChunkTooLarge> {
    #[cfg(test)]
    count_compile_call();

    let len = body.instructions.len();
    let mut ops: Vec<Op> = Vec::with_capacity(len);
    let mut op_of = Vec::with_capacity(len + 1);
    let mut registers = Registers::new();
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
            // same way.
            InstructionKind::Do(_) | InstructionKind::Loop(_) => ops.push(Op::Loop {
                index: instruction_index(index)?,
            }),
            // The `IF` clause is its condition and nothing else -- the branch
            // it chooses is not inside it, which is the boundary
            // `run.rs`'s own `If` arm measured (`SIGL` reports the `IF`'s line,
            // not the branch's). So the region is exactly two ops and the
            // register the condition lands in is released at its end.
            InstructionKind::If { false_target, .. } => {
                let targets = if_targets(&body.instructions, *false_target);
                let mark = registers.mark();
                let dst = registers.alloc()?;
                let at = op_index(&ops)?;
                let echo = echoes(trace, instruction);
                ops.push(Op::Clause {
                    index: instruction_index(index)?,
                    end: at + 3 + u32::from(echo),
                });
                push_echo(&mut ops, echo, instruction_index(index)?);
                ops.push(Op::EvalExpr {
                    index: instruction_index(index)?,
                    slot: 0,
                    dst,
                });
                let jump = op_index(&ops)?;
                ops.push(Op::JumpUnless {
                    reg: dst,
                    target: 0,
                });
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
                let region = if case_reg.is_some() { 2 } else { 1 };
                ops.push(Op::Clause {
                    index: instruction_index(index)?,
                    end: at + region + u32::from(echo),
                });
                push_echo(&mut ops, echo, instruction_index(index)?);
                if let Some(dst) = case_reg {
                    ops.push(Op::EvalExpr {
                        index: instruction_index(index)?,
                        slot: 0,
                        dst,
                    });
                }
                ops.push(Op::SelectCaseText {
                    index: instruction_index(index)?,
                    case: case_reg,
                });
                // Past the whole construct, so nothing between here and the
                // `END` can reuse the register. A `select_end` of `len` has no
                // instruction to hang the release on and needs none: there is
                // nothing after it to hand the register to.
                if select_end < len {
                    release_at[select_end] = Some(outer);
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
                let mark = registers.mark();
                let dst = registers.alloc()?;
                let at = op_index(&ops)?;
                let echo = echoes(trace, instruction);
                ops.push(Op::Clause {
                    index: instruction_index(index)?,
                    end: at + 3 + u32::from(echo),
                });
                push_echo(&mut ops, echo, instruction_index(index)?);
                ops.push(Op::WhenTest {
                    index: instruction_index(index)?,
                    case: info.case,
                    dst,
                });
                let jump = op_index(&ops)?;
                ops.push(Op::JumpUnless {
                    reg: dst,
                    target: 0,
                });
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

    assert_clause_regions_hold_no_clause_op(&ops);

    Ok(Chunk {
        trace,
        ops,
        op_of,
        registers: registers.high_water(),
    })
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
            Before::EnterOtherwise(select) => ops.push(Op::EnterOtherwise {
                select: instruction_index(*select)?,
            }),
        }
    }
    Ok(())
}

/// The index the next op will be pushed at, refused rather than wrapped.
fn op_index(ops: &[Op]) -> Result<u32, ChunkTooLarge> {
    u32::try_from(ops.len()).map_err(|_| ChunkTooLarge { what: "op stream" })
}

/// An instruction index as an op payload, refused rather than wrapped.
fn instruction_index(index: usize) -> Result<u32, ChunkTooLarge> {
    u32::try_from(index).map_err(|_| ChunkTooLarge { what: "op stream" })
}

/// **No `Generic` or `Loop` op sits inside a [`Op::Clause`] region.**
///
/// Both run a whole clause through `Interp::step_in_temps_frame`, which
/// echoes the clause itself -- and the echo is not idempotent, so a clause
/// already opened by a `Clause` op would echo twice. The compiler is where
/// this can be checked at all: the driver sees one op at a time and cannot
/// tell an op it reached by falling into a region from one it jumped to.
///
/// An unconditional `assert!` rather than a `debug_assert!`, so the release
/// build carries the same guarantee. It is one linear scan per body, once,
/// against a compile that has already walked the same list.
fn assert_clause_regions_hold_no_clause_op(ops: &[Op]) {
    for (at, op) in ops.iter().enumerate() {
        let Op::Clause { end, .. } = op else {
            continue;
        };
        for inside in ops[at + 1..(*end as usize).min(ops.len())].iter() {
            assert!(
                !matches!(inside, Op::Generic { .. } | Op::Loop { .. }),
                "a Clause region at op {at} holds an op that opens a clause of its own, \
                 so the clause would be echoed twice"
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
    use super::{Op, Registers, assert_clause_regions_hold_no_clause_op};

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
        assert_clause_regions_hold_no_clause_op(&[
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
        assert_clause_regions_hold_no_clause_op(&[
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
}
