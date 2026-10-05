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

//! The driver: what runs a compiled [`Chunk`] for the activation on top of
//! the stack.

use std::rc::Rc;

use rexx_core::{Decoded, FrameId, ObjRef, ParkedFrame, RegFrame};
use rexx_parse::{Call, ExprKind, Instruction, InstructionKind, Program, ProgramSource, SymbolId};

use super::{BodyEngine, Chunk, ConditionKeyword, Op, SendForm};
use crate::activation::{CallType, body_of};
use crate::clause::{ClauseOutcome, ClauseValue};
use crate::eval::{SymbolRead, call_target_name};
use crate::plan::Plan;
use crate::run::{
    Absorbed, CallEntry, CallResolution, ConditionTrace, Echo, Ended, Flow, LoopHeaderValues,
    QueueKeyword, ReturnKeyword, SelectEscape, SelectResume, Started, SteppedClause, absorb,
    otherwise_range, otherwise_resume, select_escape, select_parts, subroutine_started,
    when_resume, when_targets,
};
use crate::scheduler::{ExecOutcome, Scheduler};
use crate::{Code, Failure, Interp, Loud, Raised};

#[cfg(test)]
use super::counters::{
    arith_hint_skips, call_site_hits, clause_op_entries, const_builds, count_arith_hint_skip,
    count_call_site_hit, count_clause_op_entry, count_const_build, count_exec_split,
    count_load_constant_build, count_run_chunk_entry, count_stackless_entry, count_trace_op_echo,
    exec_splits, frame_floor_high_water, load_constant_builds, record_frame_floor,
    run_chunk_entries, stackless_entries, trace_op_echoes,
};
#[cfg(test)]
pub(crate) use super::counters::{counting, resume_counters, suspend_counters};

/// What a body that runs off its own end answers.
const END_OF_BODY: Ended = Ended::Exited(None);

/// The grant position of an [`Interp::ops_loop`] with no clause to grant the
/// first-instruction permission to.
const NO_GRANT: u32 = u32::MAX;

/// One activation's compiled body, owned so that a parked level keeps it.
#[derive(Clone)]
pub(crate) struct Level {
    pub(crate) program: Rc<Program>,
    pub(crate) plan: Rc<Plan>,
    pub(crate) chunk: Rc<Chunk>,
    pub(crate) selector: Option<usize>,
}

/// Where [`Interp::drive_from`] starts.
pub(crate) enum DriveStart {
    /// A body, from its activation's own `pc`.
    Level(Level),
    /// The level a park left at `floor`, with what the parked send answered.
    Woken {
        floor: usize,
        sent: Result<Option<ObjRef>, Failure>,
    },
    /// The level a slice left at `floor`, from the clause at op `at`.
    Sliced { floor: usize, at: u32 },
}

/// How [`Interp::drive_from`] stopped.
pub(crate) enum Driven {
    Ended(Ended),
    /// A primitive method parked the activity, whose levels down to this floor
    /// are parked on it.
    Parked(usize),
    /// The activity's slice ended before the clause at op `at` began, with
    /// its levels down to `floor` parked.
    Sliced {
        floor: usize,
        at: u32,
    },
}

/// A level [`Interp::drive`] left for a callee: its body, its register frame
/// and temps, and the floor of the constructs it has open.
pub(crate) struct ParkedLevel {
    /// `None` for a level whose callee runs the same body from the same
    /// chunk, and so holds it.
    level: Option<Level>,
    registers: ParkedFrame,
    temps: FrameId,
    base: usize,
}

/// The replying level a `REPLY`'s continuation resumes, its registers rooted
/// as parked values until the continuation first runs: a pending
/// continuation opens no arena block.
pub(crate) struct RepliedLevel {
    level: Level,
    registers: rexx_core::Parked,
    temps: FrameId,
}

/// A clause region stopped at a call op whose callee has yet to run.
pub(crate) struct ParkedCall {
    /// The region's own `Op::Clause`.
    clause_pc: u32,
    /// The op after the call, where the region resumes.
    at: u32,
    /// One past the region's last op.
    end: u32,
    index: usize,
    entry: SteppedClause,
    stale: bool,
    debugging: bool,
    /// A `DO`/`LOOP` header's values the region had accumulated.
    header: Option<Box<LoopHeaderValues>>,
    deliver: Deliver,
}

/// Where a parked call's answer goes.
#[derive(Clone, Copy)]
enum Deliver {
    /// A function call's value, into this register.
    Register(u16),
    /// A `CALL`'s `RESULT`, settled at this indent, and the `Flow` that ends
    /// its clause.
    Flow(usize),
    /// Where the [`Op::Send`] in front of the resume point says.
    Send,
    /// Nowhere: the [`Op::Exec`] in front of the resume point runs its
    /// instruction again.
    Exec,
}

/// Where [`Interp::drive`] goes on with the level it holds.
#[derive(Clone, Copy)]
enum Next {
    /// The body, at its activation's own `pc`.
    Body,
    /// The body, from this op, past the granting instance.
    At(u32),
    /// The call it parked, whose callee has ended.
    Resume,
    /// The send it parked, whose activity has woken.
    Woken,
    /// The body, from the clause at this op, which a slice left unopened.
    Sliced(u32),
}

/// Why [`Interp::drive`] let go of the body it holds.
enum Left {
    /// A call op's callee runs this other one.
    Entered(Level),
    /// The callee ended, and its caller runs this one.
    Resumed(Level),
    /// A primitive method parked the activity.
    Parked,
    /// The activity's slice ended before the clause at this op.
    Sliced(u32),
}

/// How a pass of [`Interp::ops_loop`] ended.
enum Exit {
    /// A `Flow` the range does not absorb.
    Flow(Flow),
    /// The rest of the range, from this op, for the instance that never asks
    /// for the permission.
    At(u32),
    /// A call op parked its clause region on
    /// [`Activity::parked_calls`](crate::activity::Activity).
    Parked,
    /// The activity's slice ended before the clause at this op began.
    Slice(u32),
}

/// What a call op that entered its callee leaves its region to park.
struct Park {
    /// The op after the call.
    at: u32,
    header: Option<Box<LoopHeaderValues>>,
    deliver: Deliver,
}

/// One construct the driver has open: a `SELECT` branch that is running, and
/// everything an escaping `Flow` needs in order to leave it.
pub(crate) struct SelectFrame {
    /// The `SELECT` instruction this branch belongs to, which is the position
    /// `pop_search_frame` resets a forwarded `LEAVE`'s indent to.
    select: usize,
    /// `SELECT LABEL name`'s own label.
    label: Option<SymbolId>,
    /// That `SELECT`'s own `OTHERWISE` marker, for [`select_escape`].
    otherwise: Option<usize>,
    /// The branch's own instruction range, which an escaping `Flow` is
    /// absorbed against here.
    start: usize,
    end: usize,
    /// Where `leave_select` resumes this branch, which is two answers rather
    /// than one ([`SelectResume`]).
    resume: SelectResume,
    /// The `SELECT`'s own `END`, as the node records it, for
    /// [`Interp::leave_otherwise`] to build the same answer from.
    select_end: Option<usize>,
    /// One past the branch's last op. **Reaching it is the branch running off
    /// its own end**, and it is an op position rather than an instruction one
    /// because that is the space the counter walks.
    op_end: u32,
    /// Which branch this is, which decides how it is left.
    branch: Branch,
}

/// One construct this level has open: a `SELECT`'s branch, or a
/// flattened `DO`/`LOOP`.
pub(crate) struct Frame {
    /// One past this frame's last op, which reaching means the frame's own
    /// range ran out.
    op_end: u32,
    /// The instruction range an escaping `Flow` is absorbed against.
    start: usize,
    end: usize,
    kind: FrameKind,
}

pub(crate) enum FrameKind {
    Select(SelectFrame),
    /// Boxed so that the stack's element stays near a `SelectFrame`'s width:
    /// a loop's state holds the header's `Number`s and is several times that.
    /// The state is on `Activity::flat_loops`, innermost last, which is the same
    /// order this stack is in -- so the loop a frame belongs to is that stack's
    /// top when the frame is the innermost one, and no index is needed.
    Loop,
}

impl Frame {
    fn select(frame: SelectFrame) -> Self {
        Frame {
            op_end: frame.op_end,
            start: frame.start,
            end: frame.end,
            kind: FrameKind::Select(frame),
        }
    }

    fn loop_pass(body_start: usize, end_index: usize) -> Self {
        Frame {
            op_end: u32::MAX,
            start: body_start,
            end: end_index,
            kind: FrameKind::Loop,
        }
    }
}

/// Which of a `SELECT`'s two kinds of branch a [`SelectFrame`] is open over.
enum Branch {
    /// A matched listed `WHEN`'s branch.
    When,
    /// The `OTHERWISE` branch.
    Otherwise,
}

/// Where a `Flow` leaves the counter, once every open frame and the range
/// itself have had their say.
enum Settled {
    /// Continue at this op.
    At(u32),
    /// Nothing absorbed it: it is the whole range's answer.
    Escaped(Flow),
}

/// What a promoted clause's own ops answered: where they left the program
/// counter, or the `Flow` a construct they resolved produced.
enum RegionEnd {
    /// Continue at this op.
    At(u32),
    /// The construct this clause resolves inside itself answered this `Flow`,
    /// which the enclosing range settles.
    Flowed(Flow),
}

impl ClauseValue for RegionEnd {
    fn rooted(&self) -> Option<ObjRef> {
        match self {
            RegionEnd::At(_) => None,
            RegionEnd::Flowed(flow) => flow.rooted(),
        }
    }
}

/// The name [`Loud::op_not_driven`] reports for an op the driver's own loop
/// has no arm for.
#[cold]
fn undriven_op_name(op: &Op) -> &'static str {
    match op {
        Op::TraceKeyword { .. } => "TraceKeyword",
        Op::LoopHeaderValue { .. } => "LoopHeaderValue",
        Op::LoopRun { .. } => "LoopRun",
        Op::LoopNext { .. } => "LoopNext",
        Op::Clause { .. } => "Clause",
        Op::CallingClause { .. } => "CallingClause",
        Op::TraceClause { .. } => "TraceClause",
        Op::EvalExpr { .. } => "EvalExpr",
        Op::CallExpr { .. } => "CallExpr",
        Op::PushArg { .. } => "PushArg",
        Op::TraceArgument { .. } => "TraceArgument",
        Op::CallArgs { .. } => "CallArgs",
        Op::TraceFunction { .. } => "TraceFunction",
        Op::SelectCaseText { .. } => "SelectCaseText",
        Op::WhenTest { .. } => "WhenTest",
        Op::EndBranch => "EndBranch",
        Op::EndWhen => "EndWhen",
        Op::EnterWhen { .. } => "EnterWhen",
        Op::EnterOtherwise { .. } => "EnterOtherwise",
        Op::Const { .. } => "Const",
        Op::LoadConstant { .. } => "LoadConstant",
        Op::TraceLiteral { .. } => "TraceLiteral",
        Op::Load { .. } => "Load",
        Op::TraceRead { .. } => "TraceRead",
        Op::Arith { .. } => "Arith",
        Op::Binary { .. } => "Binary",
        Op::TraceOperator { .. } => "TraceOperator",
        Op::Prefix { .. } => "Prefix",
        Op::TracePrefix { .. } => "TracePrefix",
        Op::Store { .. } => "Store",
        Op::Say { .. } => "Say",
        Op::Signal { .. } => "Signal",
        Op::Parse { .. } => "Parse",
        Op::Return { .. } => "Return",
        Op::Queue { .. } => "Queue",
        Op::Call { .. } => "Call",
        Op::CallNamed { .. } => "CallNamed",
        Op::Message { .. } => "Message",
        Op::Expose { .. } => "Expose",
        Op::Exec { .. } => "Exec",
        Op::Escape { .. } => "Escape",
        Op::Jump { .. } => "Jump",
        Op::JumpUnless { .. } => "JumpUnless",
        Op::ConditionJump { .. } => "ConditionJump",
        Op::Send { .. } => "Send",
        Op::List { .. } => "List",
    }
}

/// What [`Interp::leave_ended_select_branch`] found.
enum BranchEnd {
    /// No frame ended here.
    None,
    /// A frame ended and the driver resumes at this op.
    At(u32),
    /// A frame ended and its `Flow` escaped the range being driven.
    Escaped(Flow),
}

/// The ops of one clause region, `$ops`, in order, as `Op::Clause`'s arm
/// runs them from `$from`. The region leaves through `break $region` with the
/// op the counter goes to, `break $cold` with its `Result<RegionEnd,
/// Failure>`, or, where `$top` holds, `break $park` with the [`Park`] of a call
/// op that entered its callee.
macro_rules! region_ops {
    (
        $self:ident,
        $code:ident,
        $chunk:ident,
        $registers:ident,
        $source:ident,
        $clause:ident,
        $index:ident,
        $stale:ident,
        $header:ident,
        $end:ident,
        $ops:ident,
        $from:expr,
        $top:expr,
        $cold:lifetime,
        $region:lifetime,
        $park:lifetime
    ) => {
        'region_loop: for region_op in $ops {
            let deliver: Deliver = 'entered: {
            match region_op {
                // **No gate**: this op exists only in a chunk
                // compiled under a setting that echoes, which is
                // the decision. `stale` is the one thing that
                // can withdraw it, and then the clause unit has
                // already asked the current setting instead.
                Op::TraceClause { $index } => {
                    debug_assert_names_the_clause(
                        $code,
                        *$index,
                        $clause,
                        "TraceClause",
                    );
                    if !$stale {
                        #[cfg(test)]
                        count_trace_op_echo();
                        // The indent this clause's own entry
                        // computed, read back rather than
                        // recomputed: `enter_stepped_clause`
                        // sets this field to `printed_indent`
                        // for the clause it is opening and
                        // nothing between there and here writes
                        // it, so the two engines cannot come to
                        // print an echo at two different indents
                        // for one clause.
                        let indent =
                            $self.activity.clause_state.current_value_indent;
                        $self.echo_compiled_clause($source, $clause, indent);
                    }
                }
                // **The one thing this does that `EvalExpr`
                // does not is start from the site's kept
                // resolution.** The argument loop, the `>A>`
                // lines, the activation bookkeeping and the
                // three `Ended` arms are the same functions
                // `eval.rs` calls on the same node.
                Op::CallExpr {
                    $index,
                    slot,
                    path,
                    site,
                    dst,
                } => {
                    debug_assert!(
                        $chunk.holds_register(*dst),
                        "op writes register {dst} outside the region the chunk \
                     reserved"
                    );
                    debug_assert_names_the_clause(
                        $code, *$index, $clause, "CallExpr",
                    );
                    if $top {
                        match $self.begin_call_expr($code, $chunk, $clause, *slot, *path, *site)
                        {
                            Ok(Started::Ran(value)) => $registers.set(*dst, value),
                            Ok(Started::Entered) => {
                            break 'entered Deliver::Register(*dst);
                            }
                            Err(failure) => break $cold Err(failure),
                        }
                        continue 'region_loop;
                    }
                    match $self.run_call_expr($code, $chunk, $clause, *slot, *path, *site) {
                        Ok(value) => {
                            $registers.set(*dst, value);
                        }
                        Err(failure) => break $cold Err(failure),
                    }
                }
                // **Every one of these bodies is behind
                // a call.** What they do is small, but
                // what they *contain* is not -- a `Vec`
                // push carries its own growth path and an
                // argument echo builds a rendering -- and
                // inlining either into the driver's frame
                // is paid by every op in the stream.
                // Measured with the bodies written out
                // here, `varlookup` -- which never
                // executes one -- retired 5.97% more
                // instructions, `compound` 4.14%.
                Op::PushArg { src } => {
                    $self.push_call_arg($registers, *src);
                }
                Op::TraceArgument { src } => {
                    if !$self.tracing_intermediates() {
                        continue 'region_loop;
                    }
                    $self.trace_call_arg($registers, *src);
                }
                Op::CallArgs {
                    slot,
                    path,
                    site,
                    argc,
                    dst,
                } => {
                    debug_assert!(
                        $chunk.holds_register(*dst),
                        "op writes register {dst} outside the region the chunk \
                     reserved"
                    );
                    // **The body is behind a call and not
                    // written here**, which is layout
                    // rather than style: a call's
                    // resolution, its argument run and its
                    // outcome are wide, and inlining them
                    // into the driver's own frame raises
                    // the register pressure every *other*
                    // op then pays. Measured: with this
                    // body inline, `varlookup` -- which
                    // calls nothing at all -- retired 4.9%
                    // more instructions.
                    if $top {
                        match $self.begin_call_args(
                            $code, $chunk, $clause, *slot, *path, *site, *argc,
                        ) {
                            Ok(Started::Ran(value)) => $registers.set(*dst, value),
                            Ok(Started::Entered) => {
                            break 'entered Deliver::Register(*dst);
                            }
                            Err(failure) => break $cold Err(failure),
                        }
                        continue 'region_loop;
                    }
                    match $self.run_call_args(
                        $code, $chunk, $clause, *slot, *path, *site, *argc,
                    ) {
                        Ok(value) => {
                            $registers.set(*dst, value);
                        }
                        Err(failure) => break $cold Err(failure),
                    }
                }
                // The `>F>` line the op above owes, emitted
                // behind it because `eval`'s own hook is
                // post-order and this is the same line.
                Op::TraceFunction {
                    $index,
                    slot,
                    path,
                    src,
                } => {
                    debug_assert_names_the_clause(
                        $code,
                        *$index,
                        $clause,
                        "TraceFunction",
                    );
                    // **The gate in front of the descent, not
                    // only inside `trace_intermediate`.** That
                    // function returns immediately under the
                    // same condition, so this changes no
                    // output; what it changes is that an
                    // untraced run walks no path to reach a
                    // node it is not going to print.
                    if !$self.tracing_intermediates() {
                        continue 'region_loop;
                    }
                    let Some(expr) =
                        Interp::chunk_node_at($clause, *slot, *path)
                    else {
                        break $cold Err(Loud::call_op_off_its_node().into());
                    };
                    let value = $registers.get(*src);
                    $self.trace_intermediate($code, expr, value);
                }
                // A send whose receiver and arguments ops of this region
                // computed. Its body is behind a call for the reason
                // `Op::CallArgs`'s arm gives.
                Op::Send { recv, dst, .. } => {
                    debug_assert!(
                        $chunk.holds_register(*recv) && $chunk.holds_register(*dst),
                        "op reads or writes a register outside the region the chunk \
                     reserved"
                    );
                    if $top {
                        match $self.begin_send_op($chunk, $registers, region_op) {
                            Ok(Started::Ran(value)) => $registers.set(*dst, value),
                            Ok(Started::Entered) => {
                            break 'entered Deliver::Send;
                            }
                            Err(failure) => break $cold Err(failure),
                        }
                        continue 'region_loop;
                    }
                    if let Err(failure) =
                        $self.run_send_op($chunk, $registers, region_op)
                    {
                        break $cold Err(failure);
                    }
                }
                Op::List { argc, dst } => {
                    debug_assert!(
                        $chunk.holds_register(*dst),
                        "op writes register {dst} outside the region the chunk \
                     reserved"
                    );
                    if let Err(failure) = $self.build_list($registers, *argc, *dst) {
                        break $cold Err(failure);
                    }
                }
                Op::EvalExpr { $index, slot, dst } => {
                    debug_assert!(
                        $chunk.holds_register(*dst),
                        "op writes register {dst} outside the region the chunk \
                     reserved"
                    );
                    debug_assert_names_the_clause(
                        $code, *$index, $clause, "EvalExpr",
                    );
                    let value = match pinned!(
                        $self,
                        crate::pinning::PinKind::TreeEval,
                        $self.eval_chunk_expr($code, $clause, *slot)
                    ) {
                        Ok(value) => value,
                        Err(failure) => break $cold Err(failure),
                    };
                    $registers.set(*dst, value);
                }
                // **The phase's first native expression op**:
                // the literal's value, built from the chunk's
                // own interned bytes through the same
                // `Interp::literal` that `eval_node`'s `Literal`
                // arm calls, with `eval.rs` not entered at all.
                // It emits nothing -- `Op::TraceLiteral` below
                // is the line that loading a literal owes.
                Op::Const { dst, konst } => {
                    debug_assert!(
                        $chunk.holds_register(*dst),
                        "op writes register {dst} outside the region the chunk \
                     reserved"
                    );
                    // **Built once for the whole run.** The
                    // value is interned into the chunk on the
                    // first execution and read from it after
                    // that, which is what stops this op
                    // allocating a fresh object for a literal
                    // longer than the handle carries inline --
                    // 560,000 times on the pinned `rexxcps`
                    // before this. `Chunk::interned`'s own doc
                    // has why the shared value is safe and why
                    // `NIL` is the empty state.
                    let mut value = $chunk.interned_konst(*konst);
                    if value == ObjRef::NIL {
                        let Some(bytes) = $chunk.konst(*konst) else {
                            break $cold Err(
                                Loud::constant_out_of_range().into()
                            );
                        };
                        #[cfg(test)]
                        count_const_build();
                        value = $self.interned_literal(bytes);
                        debug_assert_ne!(
                            value,
                            ObjRef::NIL,
                            "a constant interned to the handle that means \
                         'not built yet', so it would be rebuilt on every \
                         execution and the cache would be dead code"
                        );
                        $chunk.remember_konst(*konst, value);
                    }
                    $registers.set(*dst, value);
                }
                // A constant symbol's own value: its upcased
                // spelling, built through the same
                // `Interp::literal` `Op::Const` above uses and
                // read out of the symbol table `code` carries,
                // which is what `eval_node`'s own `Constant`
                // arm does. It emits nothing -- the
                // `Op::TraceLiteral` below is the line it owes
                // too, because both print `>L>`.
                Op::LoadConstant { symbol, dst } => {
                    debug_assert!(
                        $chunk.holds_register(*dst),
                        "op writes register {dst} outside the region the chunk \
                     reserved"
                    );
                    // Interned exactly as `Op::Const`'s value
                    // is, and keyed by the symbol's own id --
                    // so the symbol table is read once for the
                    // whole run rather than on every execution.
                    let mut value = $chunk.interned_symbol(*symbol);
                    if value == ObjRef::NIL {
                        #[cfg(test)]
                        count_load_constant_build();
                        value = $self.interned_literal(
                            $code.symbols.name(*symbol).as_bytes(),
                        );
                        debug_assert_ne!(
                            value,
                            ObjRef::NIL,
                            "a constant symbol interned to the handle that means \
                         'not built yet', so it would be rebuilt on every \
                         execution and the cache would be dead code"
                        );
                        $chunk.remember_symbol(*symbol, value);
                    }
                    $registers.set(*dst, value);
                }
                // The `>L>` line of one literal. **Its own op**,
                // because the load emits nothing and `eval.rs`
                // emits this as a side effect of evaluating --
                // so a promoted clause with no such op drops the
                // line while every line after it still matches.
                Op::TraceLiteral { src } => {
                    debug_assert!(
                        $chunk.holds_register(*src),
                        "op reads register {src} outside the region the chunk \
                     reserved"
                    );
                    // **The gate in front of the register read,
                    // not only inside `echo_literal`.** That
                    // function returns immediately under the
                    // same condition, so this changes no output;
                    // what it changes is that an untraced run
                    // does not reach for a value it is not going
                    // to print, and a register read is a load
                    // per echo op in a stream that
                    // carries one behind every literal, read
                    // and operator. Measured over this arm and
                    // the other value-echo arms carrying the
                    // same gate, with the read moved behind it:
                    // `bench-programs/varlookup.rex` 23.751 to
                    // 21.737 billion user instructions and the
                    // pinned `rexxcps` 11.427 to 11.060
                    // billion.
                    if !$self.tracing_intermediates() {
                        continue 'region_loop;
                    }
                    let value = $registers.get(*src);
                    $self.echo_literal(value);
                }
                // **The second native expression op**: one bare
                // symbol's own value, through the same
                // `Interp::read_symbol` that `eval_node`'s
                // `Variable`/`Stem`/`Compound` arms enter, with
                // `eval.rs` itself not entered at all. It emits
                // nothing -- `Op::TraceRead` below is what
                // reading a symbol owes.
                Op::Load {
                    symbol,
                    read,
                    at,
                    dst,
                } => {
                    debug_assert!(
                        $chunk.holds_register(*dst),
                        "op writes register {dst} outside the region the chunk \
                     reserved"
                    );
                    let at = at.resolved();
                    // **The tripwire for a chunk run against a
                    // plan that is not the one it was compiled
                    // from.** `at` was read out of that plan's
                    // `by_symbol`, which is the map `code.slots`
                    // is a view of, so a mismatch here is two
                    // different plans for one body -- and it
                    // would read somebody else's slot rather
                    // than fail, which is a wrong value found by
                    // chasing it.
                    debug_assert!(
                        at.is_none() || $code.slot_for(*symbol) == at,
                        "a compiled read names a slot this body's plan does not \
                     give its symbol"
                    );
                    // **The simple read is resolved here rather
                    // than through `read_symbol`, and that is a
                    // measurement.** A simple read is a slot
                    // index; reaching it through the shared
                    // function costs an out-of-line call, a
                    // match on `SymbolRead`, a tuple return and
                    // a `Result` return, per read. Measured with
                    // the marginal method -- a body run at N and
                    // 2N iterations, differenced -- `z = a` costs
                    // 479 user instructions per execution through
                    // `read_symbol` and 431 resolved here, while
                    // `z = 1` is unmoved at 367, which is the
                    // control saying the change reached the read
                    // and nothing else.
                    let value = match read {
                        SymbolRead::Simple => {
                            let (value, novalue) = match at {
                                Some(slot) => {
                                    $self.read_slot($code, *symbol, slot)
                                }
                                None => $self.read_at($code, *symbol, None),
                            };
                            if let Err(failure) =
                                $self.novalue_check(novalue, value)
                            {
                                break $cold Err(failure);
                            }
                            value
                        }
                        SymbolRead::Stem | SymbolRead::Compound | SymbolRead::Environment => {
                            match $self.read_symbol($code, *read, *symbol, at) {
                                Ok(value) => value,
                                Err(failure) => break $cold Err(failure),
                            }
                        }
                    };
                    $registers.set(*dst, value);
                }
                // The `>V>` line one read owes, and the `>C>`
                // line in front of it when the read is a
                // compound. **Its own op**, because the load
                // emits nothing and `eval.rs` emits these as a
                // side effect of evaluating -- so a promoted
                // clause with no such op drops them while every
                // line after them still matches.
                Op::TraceRead { symbol, src, .. } => {
                    debug_assert!(
                        $chunk.holds_register(*src),
                        "op reads register {src} outside the region the chunk \
                     reserved"
                    );
                    // The gate in front of the register read,
                    // for `Op::TraceLiteral`'s own reason.
                    if !$self.tracing_intermediates() {
                        continue 'region_loop;
                    }
                    let value = $registers.get(*src);
                    $self.echo_symbol_read($code, *symbol, value);
                }
                // **A native expression op**: one arithmetic
                // operator applied to two registers, through
                // the same `Interp::arith_small_int` and
                // `Interp::arith_general` that
                // `eval_arithmetic` enters, with `eval.rs`
                // itself not entered at all -- for the operands
                // either, which is what the ops in front of
                // this one are. It emits nothing --
                // `Op::TraceOperator` below is what applying an
                // operator owes.
                Op::Arith {
                    op,
                    hint,
                    lhs,
                    rhs,
                    dst,
                } => {
                    debug_assert!(
                        $chunk.holds_register(*lhs)
                            && $chunk.holds_register(*rhs),
                        "op reads registers {lhs}/{rhs} outside the region the \
                     chunk reserved"
                    );
                    debug_assert!(
                        $chunk.holds_register(*dst),
                        "op writes register {dst} outside the region the chunk \
                     reserved"
                    );
                    // **Both read before either is written**,
                    // which is what makes `lhs == dst` -- the
                    // shape a chain compiles to -- safe.
                    let left = $registers.get(*lhs);
                    let right = $registers.get(*rhs);
                    // The operands are rooted by the registers
                    // they came from, which is what
                    // `arith_general` requires of a caller and
                    // is why no frame is pushed here where
                    // `eval_arithmetic` pushes one: it has to
                    // root values held in Rust locals across
                    // the evaluation of the operand after them,
                    // and this op's operands were rooted before
                    // it ran.
                    let mut quick = None;
                    if $chunk.tries_small_int(*hint) {
                        quick = $self.arith_small_int(*op, left, right);
                        // The one state change a site makes,
                        // and it makes it at most once: a site
                        // that has fallen through skips the
                        // attempt from here on, so the store
                        // never repeats and the line the table
                        // sits on is not dirtied again.
                        if quick.is_none() {
                            $chunk.saw_general(*hint);
                        }
                    } else {
                        #[cfg(test)]
                        count_arith_hint_skip();
                    }
                    let value = match quick {
                        Some(value) => value,
                        None => match $self.arith_general(*op, left, right) {
                            Ok(value) => value,
                            Err(failure) => break $cold Err(failure),
                        },
                    };
                    $registers.set(*dst, value);
                }
                // **Every other binary operator**: one
                // concatenation, comparison or logical
                // operator applied to two registers, through
                // the same `Interp::apply_binary` that
                // `eval_node`'s own binary arm enters, with
                // `eval.rs` itself not entered at all -- for
                // the operands either, which is what the ops in
                // front of this one are. It emits nothing --
                // `Op::TraceOperator` below is what applying an
                // operator owes.
                Op::Binary { op, lhs, rhs, dst } => {
                    debug_assert!(
                        $chunk.holds_register(*lhs)
                            && $chunk.holds_register(*rhs),
                        "op reads registers {lhs}/{rhs} outside the region the \
                     chunk reserved"
                    );
                    debug_assert!(
                        $chunk.holds_register(*dst),
                        "op writes register {dst} outside the region the chunk \
                     reserved"
                    );
                    // **Both read before either is written**,
                    // which is what makes `lhs == dst` -- the
                    // shape a chain compiles to -- safe.
                    let left = $registers.get(*lhs);
                    let right = $registers.get(*rhs);
                    // The operands are rooted by the registers
                    // they came from, which is what
                    // `apply_binary` requires of a caller and
                    // is why no frame is pushed here where
                    // `eval_node`'s own arm pushes one: it has
                    // to root values held in Rust locals across
                    // the evaluation of the operand after them,
                    // and this op's operands were rooted before
                    // it ran.
                    let value = match $self.apply_binary(*op, left, right) {
                        Ok(value) => value,
                        Err(failure) => break $cold Err(failure),
                    };
                    $registers.set(*dst, value);
                }
                // The `>O>` line one operator owes. **Its own
                // op**, because the operation emits nothing and
                // `eval.rs` emits this as a side effect of
                // evaluating -- so a promoted clause with no
                // such op drops the line while every line after
                // it still matches.
                Op::TraceOperator { op, src } => {
                    debug_assert!(
                        $chunk.holds_register(*src),
                        "op reads register {src} outside the region the chunk \
                     reserved"
                    );
                    // The gate in front of the register read,
                    // for `Op::TraceLiteral`'s own reason.
                    if !$self.tracing_intermediates() {
                        continue 'region_loop;
                    }
                    let value = $registers.get(*src);
                    $self.echo_operator(*op, value);
                }
                // **A prefix operator**: `+`, `-` or `\`
                // applied to one register, through the same
                // `Interp::apply_prefix` that
                // `Interp::eval_prefix` enters, with `eval.rs`
                // itself not entered at all -- for the operand
                // either, which is what the ops in front of
                // this one are. It emits nothing --
                // `Op::TracePrefix` below is what applying an
                // operator owes.
                Op::Prefix { op, src, dst } => {
                    debug_assert!(
                        $chunk.holds_register(*src),
                        "op reads register {src} outside the region the chunk \
                     reserved"
                    );
                    debug_assert!(
                        $chunk.holds_register(*dst),
                        "op writes register {dst} outside the region the chunk \
                     reserved"
                    );
                    // **Read before the destination is
                    // written**, which is what makes `src ==
                    // dst` -- the shape a prefix compiles to --
                    // safe.
                    let value = $registers.get(*src);
                    // The operand is rooted by the register it
                    // came from, which is what `apply_prefix`
                    // requires of a caller and is why no frame
                    // is pushed here where `eval_prefix` pushes
                    // one: it has to root a value held in a
                    // Rust local across the operator's own
                    // allocation, and this op's operand was
                    // rooted before it ran.
                    let value = match $self.apply_prefix(*op, value) {
                        Ok(value) => value,
                        Err(failure) => break $cold Err(failure),
                    };
                    $registers.set(*dst, value);
                }
                // The `>P>` line one prefix operator owes.
                // **Its own op**, for the reason
                // `Op::TraceOperator` above is, and a different
                // op from it because `>P>` is a different line
                // from `>O>`.
                Op::TracePrefix { op, src } => {
                    debug_assert!(
                        $chunk.holds_register(*src),
                        "op reads register {src} outside the region the chunk \
                     reserved"
                    );
                    // The gate in front of the register read,
                    // for `Op::TraceLiteral`'s own reason.
                    if !$self.tracing_intermediates() {
                        continue 'region_loop;
                    }
                    let value = $registers.get(*src);
                    $self.echo_prefix_op(*op, value);
                }
                // The write, through `Interp::assign_evaluated`
                // -- the whole of what `step`'s own
                // `Assignment` arm does past the evaluation, so
                // the `>>>`/`>C>`/`>=>` lines and the stem and
                // compound dispatch are that arm's rather than a
                // second copy.
                Op::Store { $index, at, src } => {
                    debug_assert!(
                        $chunk.holds_register(*src),
                        "op reads register {src} outside the region the chunk \
                     reserved"
                    );
                    debug_assert_names_the_clause(
                        $code, *$index, $clause, "Store",
                    );
                    let at = at.resolved();
                    let value = $registers.get(*src);
                    // **A resolved slot is a simple target's slot write**, taken
                    // here rather than through `assign_evaluated`, which roots the
                    // value, materialises the `Option` its `>>>` line would want,
                    // reads the clause indent, and then matches the target's shape
                    // to reach the same store. `write_slot` resolves a slot only
                    // for a `Variable` target, so the fast path does not read the
                    // clause at all. An untraced run renders nothing on either
                    // path: `result_text` and `trace_result` gate on
                    // `trace_mode().results` themselves.
                    if let Some(slot) = at
                        && !$self.trace_mode().results
                    {
                        // `Op::Load`'s own tripwire, on the writing side: `at`
                        // came out of the plan this chunk was compiled from, and a
                        // mismatch is two different plans for one body -- which
                        // would write into somebody else's slot rather than fail.
                        debug_assert!(
                            matches!(
                                &$clause.kind,
                                InstructionKind::Assignment { target, .. }
                                    if matches!(
                                        &target.kind,
                                        ExprKind::Variable(id) if $code.slot_for(*id) == at
                                    )
                            ),
                            "a compiled write names a slot this body's plan does not give \
                             its target"
                        );
                        let frame = $self.activation().frame;
                        $self.set_variable(frame, slot, value);
                    } else {
                        let InstructionKind::Assignment { target, .. } =
                            &$clause.kind
                        else {
                            break $cold Err(
                                Loud::store_op_off_its_node().into()
                            );
                        };
                        if let Err(failure) =
                            $self.assign_evaluated($code, target, value, at)
                        {
                            break $cold Err(failure);
                        }
                    }
                }
                // The print, through `Interp::say_evaluated`,
                // for the same reason `Op::Store` goes through
                // `assign_evaluated`.
                // **`SIGNAL`, all three forms**, each doing
                // what `step`'s own arm does and nothing else:
                // the two transferring forms answer
                // `Flow::Signal` and end the region, the trap
                // form edits the activation's table and falls
                // through with whatever `exec_condition_trap`
                // answers.
                Op::Signal { $index, src } => {
                    debug_assert_names_the_clause(
                        $code, *$index, $clause, "Signal",
                    );
                    let InstructionKind::Signal(signal) = &$clause.kind else {
                        break $cold Err(Loud::instruction(&$clause.kind).into());
                    };
                    let flow = match (&**signal, src) {
                        (rexx_parse::Signal::Label(name), None) => {
                            match $self.signal_to_label(name) {
                                Ok(flow) => flow,
                                Err(failure) => break $cold Err(failure),
                            }
                        }
                        (rexx_parse::Signal::Value(_), Some(register)) => {
                            debug_assert!(
                                $chunk.holds_register(*register),
                                "op reads register {register} outside the region                                                  the chunk reserved"
                            );
                            let value = $registers.get(*register);
                            match $self.signal_to_value(value) {
                                Ok(flow) => flow,
                                Err(failure) => break $cold Err(failure),
                            }
                        }
                        (rexx_parse::Signal::Trap(trap), None) => {
                            match $self.exec_condition_trap(trap, false) {
                                Ok(flow) => flow,
                                Err(failure) => break $cold Err(failure),
                            }
                        }
                        // A form whose operand does not match
                        // the op's own: loud rather than a
                        // guess about which of the two is right.
                        _ => {
                            break $cold Err(
                                Loud::signal_op_off_its_node().into()
                            );
                        }
                    };
                    break $cold Ok(RegionEnd::Flowed(flow));
                }
                // `PARSE`/`ARG`/`PULL`. The whole instruction is
                // `exec_parse`, which both engines enter; what
                // this op adds is the source expression already
                // evaluated, for the one source that has one.
                Op::Parse { $index, src } => {
                    debug_assert_names_the_clause(
                        $code, *$index, $clause, "Parse",
                    );
                    let (InstructionKind::Parse(parse)
                    | InstructionKind::Arg(parse)
                    | InstructionKind::Pull(parse)) = &$clause.kind
                    else {
                        break $cold Err(Loud::instruction(&$clause.kind).into());
                    };
                    let evaluated = src.map(|register| {
                    debug_assert!(
                        $chunk.holds_register(register),
                        "op reads register {register} outside the region the \
                         chunk reserved"
                    );
                    $registers.get(register)
                });
                    if let Err(failure) =
                        $self.exec_parse($code, parse, evaluated)
                    {
                        break $cold Err(failure);
                    }
                    break $cold Ok(RegionEnd::Flowed(Flow::Next));
                }
                Op::Say { $index, src } => {
                    debug_assert_names_the_clause($code, *$index, $clause, "Say");
                    debug_assert!(
                        matches!(
                            &$clause.kind,
                            InstructionKind::Say { expression }
                                if expression.is_some() == src.is_some()
                        ),
                        "a Say op names an instruction that is not a SAY of \
                     matching arity"
                    );
                    let value = src.map(|register| {
                    debug_assert!(
                        $chunk.holds_register(register),
                        "op reads register {register} outside the region the \
                         chunk reserved"
                    );
                    $registers.get(register)
                });
                    if let Err(failure) = $self.say_evaluated(value) {
                        break $cold Err(failure);
                    }
                }
                // The end of the activation, through
                // `Interp::returned_value`, for the same
                // reason `Op::Say` goes through
                // `say_evaluated`: the `>>>` line, the root
                // and the choice between `Flow::Return` and
                // `Flow::Exit` are that function's rather than
                // a second copy.
                Op::Return {
                    $index,
                    src,
                    keyword,
                } => {
                    debug_assert_names_the_clause(
                        $code, *$index, $clause, "Return",
                    );
                    debug_assert!(
                        matches!(
                            &$clause.kind,
                            InstructionKind::Return { expression }
                                | InstructionKind::Exit { expression }
                                if expression.is_some() == src.is_some()
                        ),
                        "a Return op names an instruction that is not a RETURN or \
                     an EXIT of matching arity"
                    );
                    debug_assert!(
                        matches!(
                            (&$clause.kind, keyword),
                            (
                                InstructionKind::Return { .. },
                                ReturnKeyword::Return
                            ) | (
                                InstructionKind::Exit { .. },
                                ReturnKeyword::Exit
                            )
                        ),
                        "a Return op's keyword names the other half of the pair \
                     from the clause it ends"
                    );
                    let value = src.map(|register| {
                    debug_assert!(
                        $chunk.holds_register(register),
                        "op reads register {register} outside the region the \
                         chunk reserved"
                    );
                    $registers.get(register)
                });
                    // `break 'cold Err` rather than `?`,
                    // which is what puts this failure
                    // through `leave_stepped_clause` below
                    // and so gets the clause echoed. A
                    // `RETURN`/`EXIT` carrying a value
                    // after a `REPLY` is the failure this
                    // op can produce; measured, `?` here
                    // reported `0 *-* <no failing clause
                    // recorded>` instead of the `return`
                    // clause.
                    let flow = match $self.returned_value(value, *keyword) {
                        Ok(flow) => flow,
                        Err(failure) => break $cold Err(failure),
                    };
                    break $cold Ok(RegionEnd::Flowed(flow));
                }
                // The queue write and its `>>>`, through
                // `Interp::queue_evaluated`. This one does not
                // end the region: a `PUSH` and a `QUEUE`
                // answer `Flow::Next`, so the region ends
                // where a `SAY`'s does.
                Op::Queue {
                    $index,
                    src,
                    keyword,
                } => {
                    debug_assert_names_the_clause(
                        $code, *$index, $clause, "Queue",
                    );
                    debug_assert!(
                        matches!(
                            &$clause.kind,
                            InstructionKind::Push { expression }
                                | InstructionKind::Queue { expression }
                                if expression.is_some() == src.is_some()
                        ),
                        "a Queue op names an instruction that is not a PUSH or a \
                     QUEUE of matching arity"
                    );
                    debug_assert!(
                        matches!(
                            (&$clause.kind, keyword),
                            (InstructionKind::Push { .. }, QueueKeyword::Push)
                                | (
                                    InstructionKind::Queue { .. },
                                    QueueKeyword::Queue
                                )
                        ),
                        "a Queue op's keyword names the other half of the pair \
                     from the clause it writes for"
                    );
                    let value = src.map(|register| {
                    debug_assert!(
                        $chunk.holds_register(register),
                        "op reads register {register} outside the region the \
                         chunk reserved"
                    );
                    $registers.get(register)
                });
                    if let Err(failure) = $self.queue_evaluated(value, *keyword)
                    {
                        break $cold Err(failure);
                    }
                }
                // The call, through the same `begin_invoke_call`
                // and `settle_call_result` that `step`'s own `Call`
                // arm reaches. This op emits nothing itself and owes
                // no echo op, because it took no line away from
                // `eval.rs`: `Op::Call`'s own doc comment has
                // the argument and the measurement behind it.
                Op::Call { $index, site } => {
                    debug_assert_names_the_clause($code, *$index, $clause, "Call");
                    if $top {
                        match $self.begin_call_tree($code, $chunk, $clause, *site) {
                            Ok((Started::Ran(ended), base_indent)) => {
                                break $cold $self
                                    .settle_call_result(ended, base_indent)
                                    .map(RegionEnd::Flowed);
                            }
                            Ok((Started::Entered, base_indent)) => {
                            break 'entered Deliver::Flow(base_indent);
                            }
                            Err(failure) => break $cold Err(failure),
                        }
                    }
                    match $self.run_call_tree($code, $chunk, $clause, *site) {
                        Ok(flow) => break $cold Ok(RegionEnd::Flowed(flow)),
                        Err(failure) => break $cold Err(failure),
                    }
                }
                // The same clause with its arguments
                // already computed by ops of this region.
                // The body is behind a call for the reason
                // `Op::CallArgs`'s arm gives.
                Op::CallNamed { $index, site, argc } => {
                    debug_assert_names_the_clause(
                        $code,
                        *$index,
                        $clause,
                        "CallNamed",
                    );
                    if $top {
                        match $self.begin_call_named($clause, $chunk, *site, *argc) {
                            Ok((Started::Ran(ended), base_indent)) => {
                                break $cold $self
                                    .settle_call_result(ended, base_indent)
                                    .map(RegionEnd::Flowed);
                            }
                            Ok((Started::Entered, base_indent)) => {
                            break 'entered Deliver::Flow(base_indent);
                            }
                            Err(failure) => break $cold Err(failure),
                        }
                    }
                    match $self.run_call_named($clause, $chunk, *site, *argc) {
                        Ok(flow) => break $cold Ok(RegionEnd::Flowed(flow)),
                        Err(failure) => break $cold Err(failure),
                    }
                }
                // A message send that is a whole clause,
                // whichever form it was written in.
                // `exec_message` is arm,
                // entered here with the fields it reads: the
                // term, and the message-assignment form's
                // value.
                Op::Message { $index } => {
                    debug_assert_names_the_clause(
                        $code, *$index, $clause, "Message",
                    );
                    let InstructionKind::Message { term, value } = &$clause.kind
                    else {
                        break $cold Err(Loud::instruction(&$clause.kind).into());
                    };
                    let flow = match pinned!(
                        $self,
                        crate::pinning::PinKind::TreeSend,
                        $self.exec_message($code, term, value.as_ref())
                    ) {
                        Ok(flow) => flow,
                        Err(failure) => break $cold Err(failure),
                    };
                    break $cold Ok(RegionEnd::Flowed(flow));
                }
                // `EXPOSE`, through `exec_expose` with the one
                // field it reads.
                Op::Expose { $index } => {
                    debug_assert_names_the_clause(
                        $code, *$index, $clause, "Expose",
                    );
                    let InstructionKind::Expose { variables } = &$clause.kind
                    else {
                        break $cold Err(Loud::instruction(&$clause.kind).into());
                    };
                    if let Err(failure) = $self.exec_expose($code, variables) {
                        break $cold Err(failure);
                    }
                    break $cold Ok(RegionEnd::Flowed(Flow::Next));
                }
                // The instruction's own work, from
                // `Interp::exec_instruction`.
                Op::Exec { $index: at } => {
                    debug_assert_names_the_clause($code, *at, $clause, "Exec");
                    match pinned!(
                        $self,
                        crate::pinning::PinKind::OpExec,
                        $self.exec_instruction(
                            $code,
                            $index,
                            $clause,
                            $source,
                            $self.activity.region_procedure_permitted,
                        )
                    ) {
                        Ok(ExecOutcome::Done(flow)) => {
                            break $cold Ok(RegionEnd::Flowed(flow));
                        }
                        Ok(outcome) => match $self.exec_suspends(
                            outcome, $top, $code, $index, $clause, $source,
                        ) {
                            Some(region) => break $cold region,
                            None => break 'entered Deliver::Exec,
                        },
                        Err(failure) => break $cold Err(failure),
                    }
                }
                // `LEAVE`/`ITERATE`, through `leave_origin`
                // with the clause this region has open; which `Flow`
                // carries it is the only thing the two keywords
                // differ in, and the clause is what says which.
                Op::Escape { $index: at } => {
                    debug_assert_names_the_clause($code, *at, $clause, "Escape");
                    let origin = Box::new(
                        $self.leave_origin($code, $index, $source, $clause),
                    );
                    let flow = match $clause.kind {
                        InstructionKind::Leave { name } => {
                            Flow::Leave(name, origin)
                        }
                        InstructionKind::Iterate { name } => {
                            Flow::Iterate(name, origin)
                        }
                        _ => {
                            break $cold Err(
                                Loud::instruction(&$clause.kind).into()
                            );
                        }
                    };
                    break $cold Ok(RegionEnd::Flowed(flow));
                }
                // An `IF`'s or a plain `WHEN`'s condition
                // validation and its `>>>` line, through the
                // same `Interp::condition_value`
                // -- so the trace, the temps frame, the
                // readback and the raiser are that function's
                // rather than a second copy. One arm for both
                // keywords, because the raiser is the only
                // thing that differs and the op carries it.
                // The validated value is branched on here
                // rather than written back to `reg`: the
                // branch was its only reader, so `reg` keeps
                // the unvalidated value it came in with and
                // nothing downstream looks at it.
                Op::ConditionJump {
                    $index,
                    reg,
                    keyword,
                    target,
                } => {
                    debug_assert!(
                        $chunk.holds_register(*reg),
                        "op reads register {reg} outside the region the chunk \
                     reserved"
                    );
                    debug_assert_names_the_clause(
                        $code,
                        *$index,
                        $clause,
                        "ConditionJump",
                    );
                    debug_assert!(
                        matches!(
                            (&$clause.kind, keyword),
                            (InstructionKind::If { .. }, ConditionKeyword::If)
                                | (
                                    InstructionKind::When { .. },
                                    ConditionKeyword::When
                                )
                        ),
                        "a ConditionJump op's keyword does not name the clause whose \
                     condition it is validating"
                    );
                    let value = $registers.get(*reg);
                    // Read live rather than compiled in, for
                    // the reason `eval_if_condition` reads it
                    // live: a nested activation moves it.
                    let indent =
                        $self.activity.clause_state.current_value_indent;
                    // **A value that is already a logical
                    // needs neither a frame nor the general
                    // test.** `condition_value` opens a temps
                    // frame and roots the value, which its own
                    // doc explains is for `WHILE`/`UNTIL`: they
                    // re-test once per pass inside the
                    // enclosing `DO`'s single frame. An
                    // `IF`/`WHEN` tests once, and its value is
                    // already rooted in the register it came
                    // from.
                    let quick = if value == crate::eval::LOGICAL_TRUE {
                        Some(true)
                    } else if value == crate::eval::LOGICAL_FALSE {
                        Some(false)
                    } else {
                        match value.decode() {
                            rexx_core::Decoded::SmallInt(1) => Some(true),
                            rexx_core::Decoded::SmallInt(0) => Some(false),
                            _ => None,
                        }
                    };
                    let holds = match quick {
                        Some(holds) if !$self.trace_mode().results => holds,
                        _ => match $self.condition_value(
                            value,
                            ConditionTrace::Result(indent),
                            false,
                            keyword.raiser(),
                        ) {
                            Ok(holds) => holds,
                            Err(failure) => break $cold Err(failure),
                        },
                    };
                    if !holds {
                        break $region *target;
                    }
                }
                Op::JumpUnless { reg, target } => {
                    debug_assert!(
                        $chunk.holds_register(*reg),
                        "op reads register {reg} outside the region the chunk \
                     reserved"
                    );
                    match $self.register_holds($registers, *reg) {
                        Ok(true) => {}
                        Ok(false) => break $region *target,
                        Err(failure) => break $cold Err(failure),
                    }
                }
                Op::WhenTest { $index, case, dst } => {
                    debug_assert!(
                        $chunk.holds_register(*dst),
                        "op writes register {dst} outside the region the chunk \
                     reserved"
                    );
                    let case_text = match case {
                        Some(register) => {
                            debug_assert!(
                                $chunk.holds_register(*register),
                                "op reads register {register} outside the region \
                             the chunk reserved"
                            );
                            let value = $registers.get(*register);
                            Some($self.to_text(value).to_vec())
                        }
                        None => None,
                    };
                    debug_assert_names_the_clause(
                        $code, *$index, $clause, "WhenTest",
                    );
                    let holds = match $self.scan_when(
                        $code,
                        $clause,
                        case_text.as_deref(),
                    ) {
                        Ok(holds) => holds,
                        Err(failure) => break $cold Err(failure),
                    };
                    // In range unconditionally: `SMALL_INT_MAX`
                    // is far above one. Stored as the logical
                    // value `Op::JumpUnless` reads back, exactly
                    // as an `IF`'s condition is.
                    let value = ObjRef::small_int(i64::from(holds))
                        .unwrap_or(ObjRef::NIL);
                    $registers.set(*dst, value);
                }
                // The `>K>` line of one header value. **The
                // emission is its own op**, which is what lets
                // the stream reproduce the order the oracle
                // evaluates and echoes a loop header in:
                // evaluate `TO`, echo it, evaluate `BY`, echo
                // it.
                Op::TraceKeyword { role, src } => {
                    debug_assert!(
                        $chunk.holds_register(*src),
                        "op reads register {src} outside the region the chunk \
                     reserved"
                    );
                    let value = $registers.get(*src);
                    $self.echo_header_value(*role, value);
                }
                // One header value's own validation, in front of
                // the next value's evaluation because that order
                // is observable (`Op::LoopHeaderValue`'s own doc
                // comment).
                Op::LoopHeaderValue { role, src } => {
                    debug_assert!(
                        $chunk.holds_register(*src),
                        "op reads register {src} outside the region the chunk \
                     reserved"
                    );
                    let value = $registers.get(*src);
                    let values =
                        $header.get_or_insert_with(LoopHeaderValues::default);
                    if let Err(failure) =
                        $self.accept_header_value(*role, value, values)
                    {
                        break $cold Err(failure);
                    }
                    // Which register roots a flattened
                    // `DO OVER`'s snapshot, recorded here
                    // because this op is the only place
                    // that knows it -- see
                    // `LoopState::OverItems`.
                    if matches!(role, crate::run::HeaderRole::Over) {
                        values.over_register = Some(*src);
                    }
                }
                // The construct itself, from the values the ops
                // above filed, through `run_loop_with_header`.
                // `BodyEngine::Chunk` says the body's clauses
                // come from this chunk.
                Op::LoopRun { $index } => {
                    debug_assert_names_the_clause(
                        $code, *$index, $clause, "LoopRun",
                    );
                    let $index = *$index as usize;
                    let (InstructionKind::Do(body)
                    | InstructionKind::Loop(body)) = &$clause.kind
                    else {
                        break $cold Err(Loud::loop_op_off_its_node().into());
                    };
                    let values = $header.take().unwrap_or_default();
                    // Flattened when this is a
                    // shape the flat path drives: the frame goes
                    // on the stack, the region ends, and the
                    // counter falls into the body's first op,
                    // which is the op after this region.
                    let values = match $self.flat_loop_start(
                        $code, $index, $clause, body, $source, values, $end,
                        $registers,
                    ) {
                        Ok(crate::run::FlatStart::Flat {
                            body_start,
                            end_index,
                        }) => {
                            $self.activity
                                .frames
                                .push(Frame::loop_pass(body_start, end_index));
                            break $region $end;
                        }
                        Ok(crate::run::FlatStart::Ended(flow)) => {
                            break $cold Ok(RegionEnd::Flowed(flow));
                        }
                        Ok(crate::run::FlatStart::Block) => break $region $end,
                        Ok(crate::run::FlatStart::Fallback(values)) => values,
                        Err(failure) => break $cold Err(failure),
                    };
                    let flow = match pinned!(
                        $self,
                        crate::pinning::PinKind::NestedLoop,
                        $self.run_loop_with_header(
                            $code,
                            $index,
                            $clause,
                            body,
                            $source,
                            BodyEngine::Chunk { $chunk, $registers },
                            values,
                        )
                    ) {
                        Ok(flow) => flow,
                        Err(failure) => break $cold Err(failure),
                    };
                    break $cold Ok(RegionEnd::Flowed(flow));
                }
                Op::Jump { target } => {
                    break $region *target;
                }
                Op::LoopNext { .. } => {
                    break $cold Err(Loud::op_not_driven("LoopNext").into());
                }
                Op::Clause { .. } => {
                    break $cold Err(Loud::op_not_driven("Clause").into());
                }
                Op::CallingClause { .. } => {
                    break $cold Err(Loud::op_not_driven("CallingClause").into());
                }
                Op::SelectCaseText { .. } => {
                    break $cold Err(
                        Loud::op_not_driven("SelectCaseText").into()
                    );
                }
                Op::EnterWhen { .. } => {
                    break $cold Err(Loud::op_not_driven("EnterWhen").into());
                }
                Op::EnterOtherwise { .. } => {
                    break $cold Err(
                        Loud::op_not_driven("EnterOtherwise").into()
                    );
                }
                Op::EndBranch => {
                    break $cold Err(Loud::op_not_driven("EndBranch").into());
                }
                Op::EndWhen => {
                    break $cold Err(Loud::op_not_driven("EndWhen").into());
                }
            }
            continue 'region_loop;
            };
            break $park Park {
                at: op_after($ops, region_op, $from),
                $header: $header.take().map(Box::new),
                deliver,
            };
        }
    };
}

/// The opening half of an [`Op::Clause`] or [`Op::CallingClause`] arm: the
/// clause `$op_index` names entered, and the region's state assigned to the
/// idents given.
macro_rules! open_clause {
    (
        $self:ident,
        $code:ident,
        $chunk:ident,
        $source:ident,
        $pc:ident,
        $op_index:ident,
        $op_end:ident,
        $granting:ident,
        $granting_instance:expr,
        $grants:expr,
        $clause_pc:ident,
        $index:ident,
        $region_end:ident,
        $clause:ident,
        $stale:ident,
        $debugging:ident,
        $entry:ident,
        $from:ident
    ) => {
        if $granting_instance && !$granting {
            return Ok(Exit::At($pc));
        }
        // Counted ahead of everything the clause's opening does, so a slice
        // that ends here leaves it to be opened afresh from `$pc`.
        let counted = match $self.count_clause_against_deadline($grants) {
            Ok(counted) => counted,
            Err(Failure::Slice) => return Ok(Exit::Slice($pc)),
            Err(failure) => return Err(failure),
        };
        #[cfg(test)]
        count_clause_op_entry();
        // Where this region starts, which is where `=` at an
        // interactive-debug pause sends the counter back to.
        $clause_pc = $pc;
        $index = *$op_index as usize;
        $region_end = *$op_end;
        // **The instruction this whole region names**, fetched once
        // and read by every index-bearing op inside the region
        // rather than each resolving its own `index` against the
        // body, which is a bounds-checked lookup of the same
        // instruction per op that would do it.
        // `compile::invariants::assert_region_ops_name_their_clause` is what
        // makes the two the same instruction by checking rather
        // than by assuming, and [`debug_assert_names_the_clause`]
        // is the same check per op in debug.
        $clause = match $code.body.instructions.get($index) {
            Some(found) => found,
            None => return Err(Loud::chunk_map_too_short().into()),
        };
        if $granting {
            $self.grant_procedure_permission($clause);
            // **A label leaves the permission alone**, so a label
            // reading `false` here is not the permission being
            // spent: `sub: procedure expose zg` grants at the
            // `PROCEDURE`, one clause after the label.
            $granting = $self.activity.procedure_permitted
                || matches!($clause.kind, InstructionKind::Label { .. });
        }
        // **Whether the setting in force is still the one this
        // chunk's trace ops were emitted for**, and the whole of
        // what makes a compiled-in emission decision safe.
        // **One read of the setting answers both questions.**
        // Asking `self` again after the clause for the debug flag
        // cost `bench-programs/emptyloop.rex` 1.52% and
        // `dispatch.rex` 1.14% in `instructions:u`, measured.
        let sink = $self.chunk_trace();
        // **The analysis, checked at the clause it answered for.**
        // A `Known` answer is a claim that this clause always runs
        // under exactly that setting, and the claim is what a
        // compile-time emission decision rests on; a missing
        // control-flow edge shows up here as a mismatch rather
        // than as a program that silently stops tracing.
        // `debug_pause` is excluded because
        // [`Interp::traced_mode`] answers `OFF` under it by
        // design, which is not the program's own setting.
        #[cfg(debug_assertions)]
        if !$self.activity.debug_pause
            && let Some(crate::ir::trace_flow::Setting::Known(claimed)) = $chunk.setting_at($index)
        {
            debug_assert_eq!(
                claimed, sink,
                "the trace analysis answered {claimed:?} for instruction {}, and \
         the setting in force when it ran is not that one",
                $index
            );
        }
        $stale = $chunk.trace().clause_echoes() != sink.clause_echoes();
        $debugging = sink.debugging();
        // **`stale` moves the clause echo from the stream back to
        // the run-time gate, in both directions at once.** The
        // region's own [`Op::TraceClause`] is skipped and
        // [`Echo::Gated`] is passed instead, so a chunk compiled to
        // echo under a setting that no longer does prints nothing,
        // and one compiled silent under a setting that now echoes
        // prints the line the setting now asks for. The two have
        // to be one decision: doing only the first would leave a
        // `TRACE R` inside a body invisible to every promoted
        // clause after it, and only the second would leave `TRACE
        // N` unable to switch one off.
        let echo = if $stale { Echo::Gated } else { Echo::Compiled };
        // **The clause unit, entered by its two halves rather than
        // by its closure form**, which is what puts the region's
        // ops in this function's own frame instead of a callee's.
        // There is one implementation of the clause boundary --
        // `Interp::enter_stepped_clause` and
        // `Interp::leave_stepped_clause`, which
        // `crate::ir::Op::Clause` is itself defined in
        // terms of -- so a promoted clause and an unpromoted one
        // discharge the same list from the same code.
        $entry = $self.enter_stepped_clause(
            echo,
            $code,
            $index,
            $clause,
            $source,
            $chunk.position_at($index),
            counted,
        );
        // Taken on entry exactly as `step` takes it, because a
        // promoted clause is a clause and the permission is spent
        // by whichever clause the activation granted it to.
        // Outside a body the driver runs nothing has granted it, so
        // there is nothing to take.
        $self.activity.region_procedure_permitted = if $grants {
            std::mem::take(&mut $self.activity.procedure_permitted)
        } else {
            debug_assert!(
                !$self.activity.procedure_permitted,
                "a permission granted outside a driven body"
            );
            false
        };
        $from = $pc + 1;
    };
}

/// The rest of an [`Op::Clause`] or [`Op::CallingClause`] arm, from the
/// region's state: its ops from `$from`, and the clause boundary. Continues
/// `$ops_label` or returns where the arm does, and otherwise answers the
/// `Flow` and the op it is settled from.
macro_rules! clause_region {
    (
        $self:ident,
        $code:ident,
        $chunk:ident,
        $registers:ident,
        $source:ident,
        $pc:ident,
        $clause_pc:ident,
        $index:ident,
        $clause:ident,
        $stale:ident,
        $debugging:ident,
        $entry:ident,
        $end:ident,
        $from:ident,
        $header:expr,
        $top:expr,
        $ops_label:lifetime
    ) => {
        // The ops of this clause region, `[from, end)`, and where
        // they leave the counter.
        'arm: {
            let park: Park = 'park: {
                let ran: Result<RegionEnd, Failure> = 'cold: {
                    // **The region answers an op index and nothing else.**
                    // Where a clause leaves the counter is the whole of what
                    // an ordinary one has to say, and carrying that in a
                    // `Result<RegionEnd, Failure>` costs a 24-byte value
                    // built and moved per clause. The two answers that do
                    // need one -- a `Flow` the enclosing range settles, and a
                    // failure -- leave through `'cold` instead, so the
                    // discriminant rides the program counter on the path
                    // every clause takes and the value exists only on the
                    // paths that have something to put in it.
                    let next: u32 = 'region: {
                        // A `DO`/`LOOP` header's values, accumulated across this
                        // region's own ops because they are not `ObjRef`s and so
                        // have no register to live in: a bound is a `Number` and
                        // a budget is a count.
                        let mut header: Option<LoopHeaderValues> = $header;
                        let Some(ops) = $chunk.ops_in($from, $end) else {
                            break 'cold Err(Loud::chunk_map_too_short().into());
                        };
                        region_ops!(
                            $self, $code, $chunk, $registers, $source, $clause, $index, $stale,
                            header, $end, ops, $from, $top, 'cold, 'region, 'park
                        );
                        $end
                    };
                    // **The hot exit, and the whole point of the split.**
                    // `leave_clause`'s own fast path is this same question,
                    // so asking it here reaches the same answer without
                    // building the value that answer would travel in.
                    // `finish_plain_clause` discharges what is left of the
                    // boundary.
                    if $self.activity.pending_traps.is_empty() {
                        $self.finish_plain_clause($entry);
                        // **Asked of the setting and not of `stale`.**
                        // Staleness heals -- the chunk is recompiled under
                        // the setting now in force -- so a skip count set
                        // at a pause would stop running down after the
                        // first clause. Measured: `trace -2` then suppressed
                        // every later clause instead of two.
                        if $debugging && $self.debug_pause_after_clause()? {
                            $pc = $clause_pc;
                            continue $ops_label;
                        }
                        $pc = next;
                        continue $ops_label;
                    }
                    Ok(RegionEnd::At(next))
                };
                break 'arm match $self
                    .leave_stepped_clause($entry, $code, $index, $clause, $source, ran)?
                {
                    ClauseOutcome::Ran(region) => match region? {
                        // A promoted clause produces no `Flow` of its own:
                        // where it leaves the counter *is* its answer, which
                        // is what a jump op is for. Only its boundary can
                        // end the activation, and that is the `Ended` below.
                        RegionEnd::At(next) => {
                            $pc = next;
                            continue $ops_label;
                        }
                        // Settled against this range from the op past the
                        // region, which is where an absorbed `Flow::Next`
                        // continues -- the same position `pc + 1` is for an
                        // op that runs one clause and no more.
                        RegionEnd::Flowed(flow) => (flow, $end),
                    },
                    ClauseOutcome::Ended(exit) => (Flow::Exit(exit.value()), $clause_pc),
                };
            };
            $self.park_call(park, $entry, $clause_pc, $end, $index, $stale, $debugging);
            return Ok(Exit::Parked);
        }
    };
}

impl Interp {
    /// One [`crate::ir::Op::PushArg`]: the register's value, or an omitted
    /// position, onto the argument stack.
    #[inline(never)]
    fn push_call_arg(&mut self, registers: RegFrame<'_>, src: u16) {
        let value = if src == Op::ARG_OMITTED {
            None
        } else {
            Some(registers.get(src))
        };
        self.activity.value_buffer.push(value);
    }

    /// One [`crate::ir::Op::TraceArgument`]'s `>A>` line, its gate already
    /// answered by the caller.
    #[inline(never)]
    fn trace_call_arg(&mut self, registers: RegFrame<'_>, src: u16) {
        let indent = self.activity.clause_state.current_value_indent;
        if src == Op::ARG_OMITTED {
            self.trace_argument(indent, b"");
            return;
        }
        let value = registers.get(src);
        if let Some(rendered) = self.intermediate_text(value) {
            self.trace_argument(indent, &rendered);
        }
    }

    /// One [`crate::ir::Op::CallArgs`]: resolve the callee, then run it over
    /// the `argc` arguments its own ops left on the argument stack.
    #[inline(never)]
    #[allow(
        clippy::too_many_arguments,
        reason = "every parameter is one field of the op or the stream it stands in, and folding them into a struct would put the wide fields back in the driver's frame"
    )]
    fn run_call_args(
        &mut self,
        code: &Code<'_>,
        chunk: &Chunk,
        clause: &Instruction,
        slot: u16,
        path: super::NodePath,
        site: u16,
        argc: u16,
    ) -> Result<ObjRef, Failure> {
        match self.begin_call_args(code, chunk, clause, slot, path, site, argc)? {
            Started::Ran(value) => Ok(value),
            Started::Entered if self.activity.native_park.is_some() => {
                self.activity.depth -= 1;
                Err(self.refuse_unrooted_park())
            }
            Started::Entered => {
                let ended = self.run_activation();
                self.finish_function_op(ended)
            }
        }
    }

    /// [`Interp::run_call_args`] up to the point its callee's body would run.
    /// An entered callee keeps the evaluation depth it counted until
    /// [`Interp::finish_function_op`].
    #[inline(never)]
    #[allow(
        clippy::too_many_arguments,
        reason = "every parameter is one field of the op or the stream it stands in, and folding them into a struct would put the wide fields back in the driver's frame"
    )]
    fn begin_call_args(
        &mut self,
        code: &Code<'_>,
        chunk: &Chunk,
        clause: &Instruction,
        slot: u16,
        path: super::NodePath,
        site: u16,
        argc: u16,
    ) -> Result<Started<ObjRef>, Failure> {
        let Some(mark) = self.activity.value_buffer.len().checked_sub(argc as usize) else {
            return Err(Loud::call_op_off_its_node().into());
        };
        // **A kept builtin is the one resolution that runs without the
        // spelling**, which is what makes this op cheaper than `Op::CallExpr`:
        // it dispatches through the row the site holds. Every other resolution
        // passes the name on, to a traceback line or the security manager.
        let target = || match Interp::chunk_node_at(clause, slot, path).map(|node| &node.kind) {
            Some(ExprKind::Call { target, .. }) => Ok(call_target_name(code, target)),
            _ => Err(Failure::from(Loud::call_op_off_its_node())),
        };
        let mut spelling: &[u8] = b"";
        let resolved = match chunk.resolved_call(site, self.routine_generation) {
            Some(resolved) => {
                #[cfg(test)]
                count_call_site_hit();
                if !matches!(resolved, crate::run::Resolved::Builtin(_)) {
                    spelling = target()?.0;
                }
                resolved
            }
            None => {
                let (name, search_labels) = target()?;
                spelling = name;
                let resolved = self.resolve_call(name, search_labels)?;
                chunk.remember_call(site, resolved, self.routine_generation);
                resolved
            }
        };
        let probe = 0u8;
        self.enter_eval_node(&raw const probe)?;
        let started = self.begin_call_over_pushed_args(resolved, spelling, mark);
        if !matches!(started, Ok(Started::Entered)) {
            self.activity.depth -= 1;
        }
        started
    }

    /// One [`crate::ir::Op::CallExpr`], its callee's body run on this Rust
    /// stack.
    #[inline(never)]
    fn run_call_expr(
        &mut self,
        code: &Code<'_>,
        chunk: &Chunk,
        clause: &Instruction,
        slot: u16,
        path: super::NodePath,
        site: u16,
    ) -> Result<ObjRef, Failure> {
        match self.begin_call_expr(code, chunk, clause, slot, path, site)? {
            Started::Ran(value) => Ok(value),
            Started::Entered if self.activity.native_park.is_some() => {
                self.activity.depth -= 1;
                Err(self.refuse_unrooted_park())
            }
            Started::Entered => {
                let ended = self.run_activation();
                self.finish_function_op(ended)
            }
        }
    }

    /// One [`crate::ir::Op::CallExpr`] up to the point its callee's body would
    /// run, as [`Interp::begin_call_args`] is for its op.
    #[inline(never)]
    fn begin_call_expr(
        &mut self,
        code: &Code<'_>,
        chunk: &Chunk,
        clause: &Instruction,
        slot: u16,
        path: super::NodePath,
        site: u16,
    ) -> Result<Started<ObjRef>, Failure> {
        let Some(ExprKind::Call { target, args }) =
            Interp::chunk_node_at(clause, slot, path).map(|node| &node.kind)
        else {
            return Err(Loud::call_op_off_its_node().into());
        };
        let (name, search_labels) = call_target_name(code, target);
        let resolution = self.site_resolution_before_arguments(chunk, site, name, search_labels)?;
        let probe = 0u8;
        self.enter_eval_node(&raw const probe)?;
        let started = self.begin_eval_call(code, resolution, name, args);
        if !matches!(started, Ok(Started::Entered)) {
            self.activity.depth -= 1;
        }
        started
    }

    /// A function call op's value once its callee's body has ended `ended`,
    /// with the evaluation depth its start counted given back.
    fn finish_function_op(&mut self, ended: Result<Ended, Failure>) -> Result<ObjRef, Failure> {
        let value = self.finish_function(ended);
        self.activity.depth -= 1;
        value
    }

    /// One [`crate::ir::Op::CallNamed`]: resolve the callee off the site or
    /// the instruction's own name, then run it over the `argc` arguments its
    /// own ops left on the argument stack.
    #[inline(never)]
    fn run_call_named(
        &mut self,
        clause: &Instruction,
        chunk: &Chunk,
        site: u16,
        argc: u16,
    ) -> Result<Flow, Failure> {
        let (started, base_indent) = self.begin_call_named(clause, chunk, site, argc)?;
        self.complete_subroutine(started, base_indent)
    }

    /// [`Interp::run_call_named`] up to the point its callee's body would
    /// run, and the indent its `RESULT` is settled at.
    #[inline(never)]
    fn begin_call_named(
        &mut self,
        clause: &Instruction,
        chunk: &Chunk,
        site: u16,
        argc: u16,
    ) -> Result<(Started<Ended>, usize), Failure> {
        let InstructionKind::Call(call) = &clause.kind else {
            return Err(Loud::call_op_off_its_node().into());
        };
        let Call::Named { name, literal, .. } = &**call else {
            return Err(Loud::call_op_off_its_node().into());
        };
        let Some(mark) = self.activity.value_buffer.len().checked_sub(argc as usize) else {
            return Err(Loud::call_op_off_its_node().into());
        };
        // The site's own kept answer and the resolution when it has none --
        // `Op::Call`'s arm has the whole argument, and this is the same site
        // table read the same way. The name comes off the instruction here
        // rather than off a node, which is why this op needs no address where
        // `Op::CallArgs` does.
        let resolved = match chunk.resolved_call(site, self.routine_generation) {
            Some(resolved) => {
                #[cfg(test)]
                count_call_site_hit();
                resolved
            }
            None => {
                let resolved = self.resolve_call(name, !*literal)?;
                chunk.remember_call(site, resolved, self.routine_generation);
                resolved
            }
        };
        // Captured before the callee runs, which overwrites
        // `current_value_indent` with its own clauses'.
        let base_indent = self.activity.clause_state.current_value_indent;
        let started = self.begin_subroutine_over_pushed_args(resolved, name, mark)?;
        Ok((started, base_indent))
    }

    /// One [`crate::ir::Op::Call`], its callee's body run on this Rust stack.
    #[inline(never)]
    fn run_call_tree(
        &mut self,
        code: &Code<'_>,
        chunk: &Chunk,
        clause: &Instruction,
        site: u16,
    ) -> Result<Flow, Failure> {
        let (started, base_indent) = self.begin_call_tree(code, chunk, clause, site)?;
        self.complete_subroutine(started, base_indent)
    }

    /// One [`crate::ir::Op::Call`] up to the point its callee's body would
    /// run, and the indent its `RESULT` is settled at: the arguments run and
    /// the resolution settled after them, as [`Interp::invoke_named_call`]
    /// runs them.
    #[inline(never)]
    fn begin_call_tree(
        &mut self,
        code: &Code<'_>,
        chunk: &Chunk,
        clause: &Instruction,
        site: u16,
    ) -> Result<(Started<Ended>, usize), Failure> {
        let InstructionKind::Call(call) = &clause.kind else {
            return Err(Loud::call_op_off_its_node().into());
        };
        let Call::Named {
            name,
            literal,
            args,
        } = &**call
        else {
            return Err(Loud::call_op_off_its_node().into());
        };
        let resolution = self.site_resolution_before_arguments(chunk, site, name, !*literal);
        let resolution = self.resolved_after_arguments(code, resolution, args)?;
        let base_indent = self.activity.clause_state.current_value_indent;
        let begun = self.begin_invoke_call(
            code,
            resolution,
            name,
            args,
            CallType::Subroutine,
            CallEntry::Written,
        )?;
        Ok((subroutine_started(begun), base_indent))
    }

    /// One [`Op::Send`] up to the point a Rexx body it enters would run, or
    /// the value its `dst` takes where it answered without entering one. An
    /// entered body keeps the evaluation depth this counted until
    /// [`Interp::finish_send_op`].
    #[inline(never)]
    fn begin_send_op(
        &mut self,
        chunk: &Chunk,
        registers: RegFrame<'_>,
        op: &Op,
    ) -> Result<Started<ObjRef>, Failure> {
        let Op::Send {
            site, recv, argc, ..
        } = *op
        else {
            return Err(Loud::call_op_off_its_node().into());
        };
        let (Some(name), Some(mark)) = (
            chunk.send_name(site),
            self.activity
                .value_buffer
                .len()
                .checked_sub(usize::from(argc)),
        ) else {
            return Err(Loud::call_op_off_its_node().into());
        };
        let receiver = registers.get(recv);
        let probe = 0u8;
        self.enter_eval_node(&raw const probe)?;
        let caller = self.caller();
        let mut values = std::mem::take(&mut self.activity.value_buffer);
        let started = self.begin_send(receiver, name, None, &values[mark..], caller);
        values.truncate(mark);
        self.lend_stack(
            matches!(started, Ok(Started::Entered)) && self.activity.native_park.is_none(),
            values,
        );
        match started {
            Ok(Started::Entered) => Ok(Started::Entered),
            Ok(Started::Ran(sent)) => {
                self.activity.depth -= 1;
                self.deliver_sent(chunk, op, sent).map(Started::Ran)
            }
            Err(failure) => {
                self.activity.depth -= 1;
                Err(failure)
            }
        }
    }

    /// One [`Op::Send`], a Rexx body it enters run on this Rust stack.
    #[inline(never)]
    fn run_send_op(
        &mut self,
        chunk: &Chunk,
        registers: RegFrame<'_>,
        op: &Op,
    ) -> Result<(), Failure> {
        let value = match self.begin_send_op(chunk, registers, op)? {
            Started::Ran(value) => value,
            Started::Entered if self.activity.native_park.is_some() => {
                self.activity.depth -= 1;
                return Err(self.refuse_unrooted_park());
            }
            Started::Entered => {
                let ended = self.run_activation();
                return self.finish_sent(chunk, registers, op, ended);
            }
        };
        if let Op::Send { dst, .. } = *op {
            registers.set(dst, value);
        }
        Ok(())
    }

    /// The [`Op::Send`] in front of op `at`, once the body it entered has
    /// ended `ended`.
    #[inline(never)]
    fn finish_send_op(
        &mut self,
        chunk: &Chunk,
        registers: RegFrame<'_>,
        at: u32,
        ended: Result<Ended, Failure>,
    ) -> Result<(), Failure> {
        let op = at
            .checked_sub(1)
            .and_then(|send| chunk.ops_in(send, at))
            .and_then(<[Op]>::first);
        let Some(op) = op else {
            return Err(Loud::chunk_map_too_short().into());
        };
        self.finish_sent(chunk, registers, op, ended)
    }

    /// The [`Op::Send`] in front of op `at`, whose activity parked and has
    /// woken with `sent`.
    fn deliver_woken_send(
        &mut self,
        chunk: &Chunk,
        registers: RegFrame<'_>,
        at: u32,
        sent: Result<Option<ObjRef>, Failure>,
    ) -> Result<(), Failure> {
        let op = at
            .checked_sub(1)
            .and_then(|send| chunk.ops_in(send, at))
            .and_then(<[Op]>::first);
        let Some(op) = op else {
            return Err(Loud::chunk_map_too_short().into());
        };
        self.activity.depth -= 1;
        let value = self.deliver_sent(chunk, op, sent?)?;
        if let Op::Send { dst, .. } = *op {
            registers.set(dst, value);
        }
        Ok(())
    }

    /// [`Interp::finish_send_op`] with the op in hand.
    fn finish_sent(
        &mut self,
        chunk: &Chunk,
        registers: RegFrame<'_>,
        op: &Op,
        ended: Result<Ended, Failure>,
    ) -> Result<(), Failure> {
        let sent = self.finish_send(ended);
        self.activity.depth -= 1;
        let value = self.deliver_sent(chunk, op, sent?)?;
        if let Op::Send { dst, .. } = *op {
            registers.set(dst, value);
        }
        Ok(())
    }

    /// [`Op::Send`] `op`'s answer `sent` settled, and the `>M>` line it
    /// owes, and what its `dst` takes: the answer for a send in an
    /// expression, and `.nil` for the other forms, whose answer goes to
    /// `RESULT`. The line comes before `RESULT` is settled, as
    /// `Interp::message_term` emits it for `Interp::exec_message`.
    fn deliver_sent(
        &mut self,
        chunk: &Chunk,
        op: &Op,
        sent: Option<ObjRef>,
    ) -> Result<ObjRef, Failure> {
        let Op::Send { site, form, .. } = *op else {
            return Err(Loud::call_op_off_its_node().into());
        };
        if form == SendForm::Value && sent.is_none() {
            return Err(Raised::no_result(chunk.send_name(site).unwrap_or_default()).into());
        }
        if let Some(value) = sent
            && self.tracing_intermediates()
        {
            self.trace_sent(chunk.send_name(site).unwrap_or_default(), value);
        }
        if form == SendForm::Value {
            return Ok(sent.unwrap_or(ObjRef::NIL));
        }
        let slot = self.reserved_result_slot();
        let frame = self.activation().frame;
        // **A send that produced no value drops `RESULT`** rather than
        // leaving the previous one in place, as `Interp::exec_message` does.
        match sent {
            Some(value) => {
                self.roots.activity_mut().push_temp(value);
                self.set_variable(frame, slot, value);
            }
            None => self.clear_variable(frame, slot),
        }
        Ok(ObjRef::NIL)
    }

    /// The `>M>` line a send of `name` that answered `value` owes.
    #[cold]
    #[inline(never)]
    fn trace_sent(&mut self, name: &[u8], value: ObjRef) {
        // Rooted across the rendering, which can run a `STRING` method.
        self.roots.activity_mut().push_temp(value);
        if let Some(rendered) = self.intermediate_text(value) {
            let indent = self.activity.clause_state.current_value_indent;
            self.trace_message(indent, name, &rendered);
        }
    }

    /// One [`Op::List`]: the array of the `argc` values on top of the argument
    /// stack, into `dst`, and the `>>>` line it owes.
    #[inline(never)]
    fn build_list(&mut self, registers: RegFrame<'_>, argc: u16, dst: u16) -> Result<(), Failure> {
        let Some(mark) = self
            .activity
            .value_buffer
            .len()
            .checked_sub(usize::from(argc))
        else {
            return Err(Loud::call_op_off_its_node().into());
        };
        let slots = self.activity.value_buffer.split_off(mark);
        let array = self.alloc_with(rexx_core::BehaviourId::ARRAY, rexx_core::Body::array(slots));
        registers.set(dst, array);
        if let Some(rendered) = self.result_text(array) {
            let indent = self.activity.clause_state.current_value_indent;
            self.trace_result(indent, &rendered);
        }
        Ok(())
    }

    /// Parks the clause region `park` stopped at, for its callee to run:
    /// everything [`Interp::ops_loop`] needs to go on from `park.at`.
    #[expect(
        clippy::too_many_arguments,
        reason = "every argument is one the driver already holds for the clause it has open"
    )]
    #[inline(always)]
    fn park_call(
        &mut self,
        park: Park,
        entry: SteppedClause,
        clause_pc: u32,
        end: u32,
        index: usize,
        stale: bool,
        debugging: bool,
    ) {
        let Park {
            at,
            header,
            deliver,
        } = park;
        self.activity.parked_calls.push(ParkedCall {
            clause_pc,
            at,
            end,
            index,
            entry,
            stale,
            debugging,
            header,
            deliver,
        });
    }

    /// An [`Op::Exec`] outcome other than `Done`, as its region's answer, or
    /// `None` where the instruction parked its activity at a level of a root
    /// driver (`top`) that nothing pins: the region parks at the op, which
    /// runs the instruction again once the activity wakes. Anywhere else a
    /// park is a pinned wait, after which the instruction runs again. A
    /// split is recorded on the replying activation and served at the next
    /// countdown visit, which this makes the next clause boundary.
    #[cold]
    #[inline(never)]
    fn exec_suspends(
        &mut self,
        mut outcome: ExecOutcome,
        top: bool,
        code: &Code<'_>,
        index: usize,
        clause: &Instruction,
        source: Option<&ProgramSource>,
    ) -> Option<Result<RegionEnd, Failure>> {
        while let ExecOutcome::Park(reason) = outcome {
            if top && self.activity.pin_depth == 0 {
                self.park_instruction(reason);
                return None;
            }
            if let Some(failure) = self.pinned_wait(reason) {
                self.abandon_guard_exec();
                return Some(Err(failure));
            }
            outcome = match pinned!(
                self,
                crate::pinning::PinKind::OpExec,
                self.exec_instruction(code, index, clause, source, false)
            ) {
                Ok(outcome) => outcome,
                Err(failure) => return Some(Err(failure)),
            };
        }
        Some(match outcome {
            ExecOutcome::Done(flow) => Ok(RegionEnd::Flowed(flow)),
            ExecOutcome::Park(_) => unreachable!("the loop above runs every park"),
            ExecOutcome::Split(value) => {
                #[cfg(test)]
                count_exec_split();
                self.activation_mut().replied = Some(Box::new(crate::activation::Replied {
                    value,
                    continuation: None,
                }));
                self.activity.splits_owed += 1;
                self.clause_countdown = 1;
                Ok(RegionEnd::Flowed(Flow::Next))
            }
        })
    }

    /// Closes the innermost `SELECT` branch if it runs out at `pc`.
    #[expect(
        clippy::too_many_arguments,
        reason = "the range `settle` absorbs against, and every argument is one the driver \
                  already holds"
    )]
    fn leave_ended_select_branch(
        &mut self,
        code: &Code<'_>,
        chunk: &Chunk,
        base: usize,
        pc: u32,
        start: usize,
        end: usize,
        source: Option<&ProgramSource>,
    ) -> Result<BranchEnd, Failure> {
        if self.activity.frames.len() <= base {
            return Ok(BranchEnd::None);
        }
        let Some(frame) = self.activity.frames.pop_if(|frame| pc >= frame.op_end) else {
            return Ok(BranchEnd::None);
        };
        let FrameKind::Select(frame) = frame.kind else {
            return Err(Loud::op_not_driven("a loop frame ran off its own end").into());
        };
        let flow = self.leave_branch(code, &frame, Flow::Next)?;
        Ok(
            match self.settle(code, chunk, base, flow, pc, start, end, source)? {
                Settled::At(target) => BranchEnd::At(target),
                Settled::Escaped(other) => BranchEnd::Escaped(other),
            },
        )
    }
}

impl Interp {
    /// Runs `root`'s body for the activation on top of the stack, and the
    /// body of every callee a call op of a driven region enters, on this one
    /// Rust frame.
    pub(crate) fn drive(&mut self, root: Level) -> Result<Ended, Failure> {
        // Once a second activity exists, a slice may end at any clause of
        // this driver, and only a pinned frame defers it.
        debug_assert!(
            self.activity.pin_depth > 0 || self.activities_spawned() == 0,
            "a nested driver runs under no pinned frame"
        );
        // The caller's pushed arguments are lent out: every clause of the
        // body empties the buffer.
        let lent = std::mem::take(&mut self.activity.value_buffer);
        let driven = self.drive_from(DriveStart::Level(root), false);
        self.activity.value_buffer = lent;
        match driven? {
            Driven::Ended(ended) => Ok(ended),
            Driven::Parked(_) | Driven::Sliced { .. } => Err(Loud::scheduler_inconsistency(
                "a wait outside every root driver and pinned frame",
            )
            .into()),
        }
    }

    /// [`Interp::drive`] from `start`. Where `parkable`, this is the running
    /// activity's root driver and a primitive method's park parks it.
    pub(crate) fn drive_from(
        &mut self,
        start: DriveStart,
        parkable: bool,
    ) -> Result<Driven, Failure> {
        let outer = std::mem::replace(&mut self.activity.driver_pins, self.activity.pin_depth);
        let driven = self.drive_levels(start, parkable);
        self.activity.driver_pins = outer;
        driven
    }

    /// [`Interp::drive_from`]'s loop.
    fn drive_levels(&mut self, start: DriveStart, parkable: bool) -> Result<Driven, Failure> {
        #[cfg(test)]
        count_run_chunk_entry();

        let arena = self.roots.activity().frames();
        let (floor, mut level, mut temps, mut registers, mut base, mut next, mut callee) =
            match start {
                DriveStart::Level(root) => {
                    // Truncated on the way out of each level, so a temp its
                    // chunk leaks does not outlive it.
                    let temps = self.roots.activity_mut().push_frame();
                    let registers = arena.reserve(root.chunk.registers);
                    (
                        self.activity.parked_levels.len(),
                        root,
                        temps,
                        registers,
                        self.activity.frames.len(),
                        Next::Body,
                        Ok(END_OF_BODY),
                    )
                }
                DriveStart::Woken { floor, sent } => {
                    let Some(ParkedLevel {
                        level: Some(level),
                        registers,
                        temps,
                        base,
                    }) = self.activity.parked_levels.pop()
                    else {
                        return Err(Loud::op_not_driven("a parked level").into());
                    };
                    (
                        floor,
                        level,
                        temps,
                        arena.unpark(registers),
                        base,
                        Next::Woken,
                        sent.map(Ended::Returned),
                    )
                }
                DriveStart::Sliced { floor, at } => {
                    let Some(ParkedLevel {
                        level: Some(level),
                        registers,
                        temps,
                        base,
                    }) = self.activity.parked_levels.pop()
                    else {
                        return Err(Loud::op_not_driven("a parked level").into());
                    };
                    (
                        floor,
                        level,
                        temps,
                        arena.unpark(registers),
                        base,
                        Next::Sliced(at),
                        Ok(END_OF_BODY),
                    )
                }
            };
        loop {
            let left = 'level: {
                let code = body_of(&level.program, level.selector).map(|body| Code {
                    body,
                    symbols: &level.program.symbols,
                    slots: &level.plan.by_symbol,
                    plan: Some(&level.plan),
                });
                let chunk: &Chunk = &level.chunk;
                let source = Some(&level.program.source);
                loop {
                    let ended: Result<Ended, Failure> = 'ended: {
                        let Some(code) = &code else {
                            break 'ended Err(Loud::missing_body().into());
                        };
                        let len = code.body.instructions.len();
                        loop {
                            let ran = match next {
                                Next::Body => {
                                    // The activation's own `pc` is where a body is entered at -- `0`
                                    // for a program, a label's index for a `CALL`, a handler's for a
                                    // trap -- and `apply_flow` is what moves it afterwards.
                                    let entry = self.activation().pc;
                                    if entry >= len {
                                        // A body with no clause serves its
                                        // method's pending reserve here.
                                        if !self.activity.guard_waits.is_empty() {
                                            match self.serve_guard_wait(parkable) {
                                                Ok(()) => {}
                                                Err(Failure::Slice) => {
                                                    let Some(at) = chunk.op_at(entry) else {
                                                        break 'ended Err(
                                                            Loud::chunk_map_too_short().into(),
                                                        );
                                                    };
                                                    break 'level Left::Sliced(at);
                                                }
                                                Err(failure) => break 'ended Err(failure),
                                            }
                                        }
                                        break 'ended Ok(END_OF_BODY);
                                    }
                                    let Some(at) = chunk.op_at(entry) else {
                                        break 'ended Err(Loud::chunk_map_too_short().into());
                                    };
                                    #[cfg(test)]
                                    record_frame_floor(base);
                                    // `[0, len]` is the whole body, so every `Goto` a clause of it
                                    // produces is absorbed here and only the flows that end or
                                    // redirect the activation come back.
                                    match self.grant_for(code, chunk, at) {
                                        Some(grant) => self.ops_loop_steady(
                                            code, chunk, registers, at, 0, len, source, base,
                                            grant, None,
                                        ),
                                        None => self.ops_loop_granting(
                                            code, chunk, registers, at, 0, len, source, base,
                                        ),
                                    }
                                }
                                Next::At(at) => self.ops_loop_steady(
                                    code, chunk, registers, at, 0, len, source, base, NO_GRANT,
                                    None,
                                ),
                                // As `Next::Body` enters, from the clause the
                                // slice left: the permission is still pending
                                // where that clause is the body's first.
                                Next::Sliced(at) => {
                                    // The clause was counted when the slice
                                    // ended at it.
                                    self.clause_countdown += 1;
                                    match self.grant_for(code, chunk, at) {
                                        Some(grant) => self.ops_loop_steady(
                                            code, chunk, registers, at, 0, len, source, base,
                                            grant, None,
                                        ),
                                        None => self.ops_loop_granting(
                                            code, chunk, registers, at, 0, len, source, base,
                                        ),
                                    }
                                }
                                Next::Resume => {
                                    let ended = std::mem::replace(&mut callee, Ok(END_OF_BODY));
                                    match self.activity.parked_calls.pop() {
                                        Some(parked) => match parked.deliver {
                                            Deliver::Flow(base_indent) => self.resume_call(
                                                code,
                                                chunk,
                                                source,
                                                base,
                                                len,
                                                parked,
                                                base_indent,
                                                ended,
                                            ),
                                            Deliver::Register(_)
                                            | Deliver::Send
                                            | Deliver::Exec => self.resume_region(
                                                code, chunk, registers, source, base, len, parked,
                                                ended,
                                            ),
                                        },
                                        None => Err(Loud::op_not_driven("a parked call").into()),
                                    }
                                }
                                Next::Woken => {
                                    let sent = std::mem::replace(&mut callee, Ok(END_OF_BODY))
                                        .map(Ended::value);
                                    match self.activity.parked_calls.pop() {
                                        Some(parked) => self.resume_woken(
                                            code, chunk, registers, source, base, len, parked, sent,
                                        ),
                                        None => Err(Loud::op_not_driven("a parked call").into()),
                                    }
                                }
                            };
                            let flow = match ran {
                                Ok(Exit::Flow(flow)) => flow,
                                Ok(Exit::At(at)) => {
                                    next = Next::At(at);
                                    continue;
                                }
                                Ok(Exit::Slice(at)) => {
                                    if self.split_owed() {
                                        break 'ended self
                                            .split_level(&level, registers, base, at)
                                            .map(|()| END_OF_BODY);
                                    }
                                    debug_assert!(
                                        parkable,
                                        "a slice ended in a driver no pinned frame counts"
                                    );
                                    if parkable {
                                        break 'level Left::Sliced(at);
                                    }
                                    next = Next::Sliced(at);
                                    continue;
                                }
                                Ok(Exit::Parked) if self.activity.native_park.is_some() => {
                                    if parkable {
                                        break 'level Left::Parked;
                                    }
                                    callee = Err(self.refuse_unrooted_park());
                                    next = Next::Woken;
                                    continue;
                                }
                                Ok(Exit::Parked) => match self.callee_level(&level) {
                                    Ok(None) => {
                                        self.activity.parked_levels.push(ParkedLevel {
                                            level: None,
                                            registers: arena.park(registers),
                                            temps,
                                            base,
                                        });
                                        temps = self.roots.activity_mut().push_frame();
                                        registers = arena.reserve(chunk.registers);
                                        base = self.activity.frames.len();
                                        next = Next::Body;
                                        #[cfg(test)]
                                        count_run_chunk_entry();
                                        #[cfg(test)]
                                        count_stackless_entry();
                                        continue;
                                    }
                                    Ok(Some(entered)) => break 'level Left::Entered(entered),
                                    // The callee's body never ran, and its caller
                                    // resumes with the failure.
                                    Err(failure) => {
                                        callee = Err(failure);
                                        next = Next::Resume;
                                        continue;
                                    }
                                },
                                // `offer_to_trap` answers `Flow::Signal` for a trap that
                                // fired and `Flow::Exit` for a handler that ended the
                                // program, never `Flow::Next`, so a trapped condition
                                // redirects this loop rather than ending it.
                                Err(failure) => {
                                    // A `Flow` that left through `settle` has already
                                    // popped every frame this level opened, so this
                                    // fires only where a `Failure` unwound past them.
                                    if self.activity.frames.len() > base {
                                        self.unwind_frames(base);
                                    }
                                    match self.offer_to_trap(code, failure) {
                                        Ok(flow) => flow,
                                        Err(failure) => break 'ended Err(failure),
                                    }
                                }
                            };
                            // `Flow::Next` is `ops_loop` reaching the end of its
                            // range, and the range here is the whole body -- so
                            // it is the end of the body itself rather than one
                            // clause finishing, and there is no `pc` to advance.
                            // Everything else is a flow the activation itself
                            // owns; a `Signal` writes the `pc` that `Next::Body`
                            // then reads back.
                            if matches!(flow, Flow::Next) {
                                break 'ended Ok(END_OF_BODY);
                            }
                            match self.apply_flow(code, flow) {
                                Ok(Some(ended)) => break 'ended Ok(ended),
                                Ok(None) => next = Next::Body,
                                Err(failure) => break 'ended Err(failure),
                            }
                        }
                    };
                    // Released on both paths rather than only on `Ok`: the loud
                    // and raised paths leave the activation for its caller to
                    // tear down, and a frame left behind would keep its
                    // registers rooted for the rest of the run.
                    arena.release(registers);
                    self.roots.activity_mut().pop_frame(temps);
                    if self.activity.parked_levels.len() == floor {
                        return ended.map(Driven::Ended);
                    }
                    let Some(parked) = self.activity.parked_levels.pop() else {
                        return Err(Loud::op_not_driven("a parked level").into());
                    };
                    registers = arena.unpark(parked.registers);
                    temps = parked.temps;
                    base = parked.base;
                    callee = ended;
                    next = Next::Resume;
                    if let Some(caller) = parked.level {
                        break 'level Left::Resumed(caller);
                    }
                }
            };
            match left {
                Left::Entered(entered) => {
                    self.activity.parked_levels.push(ParkedLevel {
                        level: Some(std::mem::replace(&mut level, entered)),
                        registers: arena.park(registers),
                        temps,
                        base,
                    });
                    temps = self.roots.activity_mut().push_frame();
                    registers = arena.reserve(level.chunk.registers);
                    base = self.activity.frames.len();
                    next = Next::Body;
                    #[cfg(test)]
                    count_run_chunk_entry();
                    #[cfg(test)]
                    count_stackless_entry();
                }
                Left::Resumed(caller) => level = caller,
                Left::Parked => {
                    if let Some(park) = self.activity.native_park.as_ref() {
                        let reason = park.reason();
                        self.park(reason);
                    }
                    self.activity.parked_levels.push(ParkedLevel {
                        level: Some(level),
                        registers: arena.park(registers),
                        temps,
                        base,
                    });
                    return Ok(Driven::Parked(floor));
                }
                Left::Sliced(at) => {
                    self.activity.parked_levels.push(ParkedLevel {
                        level: Some(level),
                        registers: arena.park(registers),
                        temps,
                        base,
                    });
                    return Ok(Driven::Sliced { floor, at });
                }
            }
        }
    }

    /// Moves the replying level to a new activity's record, parked to resume
    /// at the op `at`: its registers, the constructs it has open from `base`
    /// with their loops, and its level and clause state.
    /// Its activation and slot frame follow from
    /// [`Interp::release_method_activation`].
    #[cold]
    #[inline(never)]
    fn split_level(
        &mut self,
        level: &Level,
        registers: RegFrame<'_>,
        base: usize,
        at: u32,
    ) -> Result<(), Failure> {
        let spawner = self.spawner_of_running()?;
        let mut idle = self.new_activity();
        idle.activity.spawner = Some(spawner);
        let values = (0..registers.len())
            .map(|index| registers.get(index))
            .collect();
        let registers = idle.roots.park(values);
        let temps = idle.roots.push_frame();
        let continuation = &mut idle.activity;
        continuation.replied_level = Some(Box::new(RepliedLevel {
            level: level.clone(),
            registers,
            temps,
        }));
        continuation.sliced = Some((0, at));
        let loops = self.activity.frames[base..]
            .iter()
            .filter(|frame| matches!(frame.kind, FrameKind::Loop))
            .count();
        continuation.frames = self.activity.frames.split_off(base);
        if loops > 0 {
            let outer = self.activity.flat_loops.len() + 1 - loops;
            continuation.flat_loops = self.activity.flat_loops.split_off(outer);
            let enclosing = self.activity.flat_loops.pop();
            continuation.flat_top = std::mem::replace(&mut self.activity.flat_top, enclosing);
        }
        // A trap the `REPLY` clause queued was delivered at that clause's end.
        debug_assert!(
            !self
                .activity
                .pending_traps
                .iter()
                .any(|pending| pending.activation == self.activation().id),
            "a trap queued for a replying activation outlived its clause"
        );
        continuation.clause_state = self.save_clause_state().into_state();
        continuation.activation_indent = self.activity.activation_indent;
        continuation.indent_offset = self.activity.indent_offset;
        continuation.clause_line_override = self.activity.clause_line_override;
        if let Some(replied) = self.activation_mut().replied.as_mut() {
            replied.continuation = Some(idle);
        }
        self.activity.splits_owed -= 1;
        Ok(())
    }

    /// Parks the level [`Interp::split_level`] moved, its registers now in
    /// this activity's arena, for the continuation's first slice to resume.
    #[cold]
    #[inline(never)]
    pub(crate) fn open_replied_level(&mut self) {
        let Some(replied) = self.activity.replied_level.take() else {
            return;
        };
        let RepliedLevel {
            level,
            registers,
            temps,
        } = *replied;
        let values = self.roots.activity_mut().take_parked(registers);
        let arena = self.roots.activity().frames();
        let len = u16::try_from(values.len()).expect("a register frame's length is a u16");
        let moved = arena.reserve(len);
        for (index, value) in (0..len).zip(values) {
            moved.set(index, value);
        }
        self.activity.parked_levels.push(ParkedLevel {
            level: Some(level),
            registers: arena.park(moved),
            temps,
            base: 0,
        });
    }

    /// The body the callee a call op of `caller` entered runs: [`Interp::
    /// prepare_level`]'s, or `None` where that is `caller`'s own.
    fn callee_level(&mut self, caller: &Level) -> Result<Option<Level>, Failure> {
        self.record_call_convention();
        let running = self.activation();
        if running.body == caller.selector
            && Rc::ptr_eq(&running.plan, &caller.plan)
            && caller.chunk.trace() == self.chunk_trace()
        {
            return Ok(None);
        }
        self.running_level().map(Some)
    }

    /// A parked `CALL`, whose op ends its clause region: its `RESULT` settled
    /// at `base_indent`, and the clause boundary.
    #[expect(
        clippy::too_many_arguments,
        reason = "one caller, and every argument is a value it already holds"
    )]
    #[inline(never)]
    fn resume_call(
        &mut self,
        code: &Code<'_>,
        chunk: &Chunk,
        source: Option<&ProgramSource>,
        base: usize,
        len: usize,
        parked: ParkedCall,
        base_indent: usize,
        ended: Result<Ended, Failure>,
    ) -> Result<Exit, Failure> {
        let ran = self
            .finish_call(ended)
            .and_then(|ended| self.settle_call_result(ended, base_indent))
            .map(RegionEnd::Flowed);
        self.leave_parked(code, chunk, source, base, len, parked, ran)
    }

    /// The level [`Interp::drive`] has just taken back from
    /// [`Activity::parked_levels`](crate::activity::Activity), run on from
    /// `parked`, a function call or send op, with `ended`, what the callee's
    /// body answered: the answer delivered, then the rest of its clause
    /// region in [`Interp::ops_loop_steady`].
    #[expect(
        clippy::too_many_arguments,
        reason = "one caller, and every argument is a value it already holds"
    )]
    #[inline(never)]
    fn resume_region(
        &mut self,
        code: &Code<'_>,
        chunk: &Chunk,
        registers: RegFrame<'_>,
        source: Option<&ProgramSource>,
        base: usize,
        len: usize,
        parked: ParkedCall,
        ended: Result<Ended, Failure>,
    ) -> Result<Exit, Failure> {
        let delivered = match parked.deliver {
            Deliver::Register(dst) => self
                .finish_function_op(ended)
                .map(|value| registers.set(dst, value)),
            Deliver::Send => self.finish_send_op(chunk, registers, parked.at, ended),
            Deliver::Flow(_) => Err(Loud::op_not_driven("a parked CALL resumed mid-region").into()),
            Deliver::Exec => Err(Loud::op_not_driven("a parked instruction with a callee").into()),
        };
        match delivered {
            Ok(()) => self.ops_loop_steady(
                code,
                chunk,
                registers,
                parked.at,
                0,
                len,
                source,
                base,
                NO_GRANT,
                Some(parked),
            ),
            Err(failure) => self.leave_parked(code, chunk, source, base, len, parked, Err(failure)),
        }
    }

    /// 44.1 for the function call op in front of `parked`'s resume point,
    /// whose callee parked and answered nothing, named as its spelling is.
    #[cold]
    fn no_data_from_parked_call(
        &self,
        code: &Code<'_>,
        chunk: &Chunk,
        parked: &ParkedCall,
    ) -> Failure {
        let op = parked
            .at
            .checked_sub(1)
            .and_then(|call| chunk.ops_in(call, parked.at))
            .and_then(<[Op]>::first);
        let (Some(Op::CallArgs { slot, path, .. } | Op::CallExpr { slot, path, .. }), Some(clause)) =
            (op, code.body.instructions.get(parked.index))
        else {
            return Loud::call_op_off_its_node().into();
        };
        match Interp::chunk_node_at(clause, *slot, *path).map(|node| &node.kind) {
            Some(ExprKind::Call { target, .. }) => {
                Raised::no_data_returned(call_target_name(code, target).0).into()
            }
            _ => Loud::call_op_off_its_node().into(),
        }
    }

    /// [`Interp::resume_region`] for a call or send op whose activity parked
    /// and has woken with `sent`.
    #[expect(
        clippy::too_many_arguments,
        reason = "one caller, and every argument is a value it already holds"
    )]
    #[cold]
    #[inline(never)]
    fn resume_woken(
        &mut self,
        code: &Code<'_>,
        chunk: &Chunk,
        registers: RegFrame<'_>,
        source: Option<&ProgramSource>,
        base: usize,
        len: usize,
        parked: ParkedCall,
        sent: Result<Option<ObjRef>, Failure>,
    ) -> Result<Exit, Failure> {
        let delivered = match parked.deliver {
            Deliver::Send => self.deliver_woken_send(chunk, registers, parked.at, sent),
            Deliver::Exec => sent.map(|_| ()).inspect_err(|_| self.abandon_guard_exec()),
            Deliver::Register(dst) => {
                self.activity.depth -= 1;
                match sent {
                    Ok(Some(value)) => {
                        registers.set(dst, value);
                        Ok(())
                    }
                    Ok(None) => Err(self.no_data_from_parked_call(code, chunk, &parked)),
                    Err(failure) => Err(failure),
                }
            }
            Deliver::Flow(base_indent) => {
                let ran = sent
                    .and_then(|value| self.settle_call_result(Ended::Returned(value), base_indent))
                    .map(RegionEnd::Flowed);
                return self.leave_parked(code, chunk, source, base, len, parked, ran);
            }
        };
        let mut parked = parked;
        if matches!(parked.deliver, Deliver::Exec) {
            // The region goes on from the instruction's own op.
            parked.at -= 1;
        }
        match delivered {
            Ok(()) => self.ops_loop_steady(
                code,
                chunk,
                registers,
                parked.at,
                0,
                len,
                source,
                base,
                NO_GRANT,
                Some(parked),
            ),
            Err(failure) => self.leave_parked(code, chunk, source, base, len, parked, Err(failure)),
        }
    }

    /// `parked`'s clause left with `ran` rather than running on: the clause
    /// boundary, and where the counter goes after it.
    #[expect(
        clippy::too_many_arguments,
        reason = "every argument is a value each caller already holds"
    )]
    #[inline(always)]
    fn leave_parked(
        &mut self,
        code: &Code<'_>,
        chunk: &Chunk,
        source: Option<&ProgramSource>,
        base: usize,
        len: usize,
        parked: ParkedCall,
        ran: Result<RegionEnd, Failure>,
    ) -> Result<Exit, Failure> {
        let ParkedCall {
            clause_pc,
            end,
            index,
            entry,
            ..
        } = parked;
        let Some(clause) = code.body.instructions.get(index) else {
            return Err(Loud::chunk_map_too_short().into());
        };
        let (flow, next) =
            match self.leave_stepped_clause(entry, code, index, clause, source, ran)? {
                ClauseOutcome::Ran(region) => match region? {
                    RegionEnd::At(next) => return Ok(Exit::At(next)),
                    RegionEnd::Flowed(flow) => (flow, end),
                },
                ClauseOutcome::Ended(exit) => (Flow::Exit(exit.value()), clause_pc),
            };
        Ok(
            match self.settle(code, chunk, base, flow, next, 0, len, source)? {
                Settled::At(pc) => Exit::At(pc),
                Settled::Escaped(other) => Exit::Flow(other),
            },
        )
    }

    /// `run_bounded`'s chunk arm: the instructions of `[start, end)`, run from
    /// `chunk`'s ops.
    #[inline]
    pub(crate) fn run_bounded_from_chunk(
        &mut self,
        code: &Code<'_>,
        chunk: &Chunk,
        registers: RegFrame<'_>,
        start: usize,
        end: usize,
        source: Option<&ProgramSource>,
    ) -> Result<Flow, Failure> {
        let Some(at) = chunk.op_at(start) else {
            return Err(Loud::chunk_map_too_short().into());
        };
        self.run_ops(code, chunk, registers, at, start, end, source)
    }

    /// Runs `chunk`'s ops from op `at` until one of them produces a `Flow`
    /// that `[start, end]` does not absorb, and answers that `Flow`.
    #[expect(
        clippy::too_many_arguments,
        reason = "one caller, and every argument is a value it already holds"
    )]
    // Reproduced across two sittings and two builds, with branch misses at
    // 70,000 out of 6.3 billion branches either way, so it is not prediction.
    // No mechanism below "the layout changed" was found, and a change with that
    // profile is not worth 7 instructions.
    fn run_ops(
        &mut self,
        code: &Code<'_>,
        chunk: &Chunk,
        registers: RegFrame<'_>,
        at: u32,
        start: usize,
        end: usize,
        source: Option<&ProgramSource>,
    ) -> Result<Flow, Failure> {
        // The frames below this length belong to the levels that entered
        // before this one, and every check here stops at it. **This is the
        // whole of what makes one stack safe to share**: `settle` walks down
        // to it and no further, so an escaping `Flow` meets exactly the
        // constructs this level opened, in the order it opened them.
        let base = self.activity.frames.len();
        #[cfg(test)]
        record_frame_floor(base);
        let flow = self.run_ops_from(code, chunk, registers, at, start, end, source, base);
        // A `Flow` that left through `settle` has already popped every frame
        // this level opened, so this fires only where a `Failure` unwound past
        // them -- the point the `Vec` local to this call used to be dropped at.
        if self.activity.frames.len() > base {
            self.unwind_frames(base);
        }
        flow
    }

    /// [`Interp::run_ops`]' body, with the frame stack's floor already taken.
    #[expect(
        clippy::too_many_arguments,
        reason = "one caller, and every argument is a value it already holds"
    )]
    fn run_ops_from(
        &mut self,
        code: &Code<'_>,
        chunk: &Chunk,
        registers: RegFrame<'_>,
        at: u32,
        start: usize,
        end: usize,
        source: Option<&ProgramSource>,
        base: usize,
    ) -> Result<Flow, Failure> {
        match self.ops_loop::<false, false>(
            code, chunk, registers, at, start, end, source, base, NO_GRANT, None,
        )? {
            Exit::Flow(flow) => Ok(flow),
            Exit::At(_) | Exit::Parked | Exit::Slice(_) => {
                Err(Loud::op_not_driven("a handoff or a park outside a driven body").into())
            }
        }
    }

    /// [`Interp::ops_loop`]'s instance for a body [`Interp::drive`] runs.
    /// Its own function rather than inlined into `drive`: measured,
    /// `bench-programs/nop.rex` retires 5.2% fewer instructions this way.
    #[expect(
        clippy::too_many_arguments,
        reason = "every argument is a value each caller already holds"
    )]
    #[inline(never)]
    fn ops_loop_steady(
        &mut self,
        code: &Code<'_>,
        chunk: &Chunk,
        registers: RegFrame<'_>,
        at: u32,
        start: usize,
        end: usize,
        source: Option<&ProgramSource>,
        base: usize,
        grant: u32,
        resume: Option<ParkedCall>,
    ) -> Result<Exit, Failure> {
        self.ops_loop::<false, true>(
            code, chunk, registers, at, start, end, source, base, grant, resume,
        )
    }

    /// [`Interp::ops_loop`]'s granting instance, for a body whose entry
    /// [`Interp::grant_for`] cannot place the permission for.
    #[expect(
        clippy::too_many_arguments,
        reason = "one caller, and every argument is a value it already holds"
    )]
    #[cold]
    #[inline(never)]
    fn ops_loop_granting(
        &mut self,
        code: &Code<'_>,
        chunk: &Chunk,
        registers: RegFrame<'_>,
        at: u32,
        start: usize,
        end: usize,
        source: Option<&ProgramSource>,
        base: usize,
    ) -> Result<Exit, Failure> {
        self.ops_loop::<true, true>(
            code, chunk, registers, at, start, end, source, base, NO_GRANT, None,
        )
    }

    /// The op of the clause a body entered at op `at` opens first once past
    /// its labels, which the running activation's first-instruction
    /// permission goes to in [`Interp::ops_loop_steady`]: [`NO_GRANT`] where
    /// there is no permission, and `None` for an entry at anything but a run
    /// of clauses.
    fn grant_for(&self, code: &Code<'_>, chunk: &Chunk, at: u32) -> Option<u32> {
        if !self.activation().first_instruction_pending {
            return Some(NO_GRANT);
        }
        let mut pc = at;
        loop {
            let (index, end) = match chunk.ops_in(pc, pc + 1).and_then(<[Op]>::first) {
                Some(Op::Clause { index, end } | Op::CallingClause { index, end }) => {
                    (*index, *end)
                }
                _ => return None,
            };
            match code.body.instructions.get(index as usize) {
                Some(clause) if matches!(clause.kind, InstructionKind::Label { .. }) => pc = end,
                Some(_) => return Some(pc),
                None => return None,
            }
        }
    }

    /// Runs `chunk`'s ops from op `at` until one of them produces a `Flow`
    /// that `[start, end]` does not absorb. Under `TOP` the range is a body
    /// [`Interp::drive`] runs, and a call op parks its clause region for its
    /// callee; `grant` is the op of the clause the first-instruction
    /// permission goes to, and `resume` a parked region whose callee has
    /// ended and whose answer is delivered, run on first.
    #[expect(
        clippy::too_many_arguments,
        reason = "every argument is a value each caller already holds"
    )]
    #[inline(always)]
    fn ops_loop<const GRANTING: bool, const TOP: bool>(
        &mut self,
        code: &Code<'_>,
        chunk: &Chunk,
        registers: RegFrame<'_>,
        at: u32,
        start: usize,
        end: usize,
        source: Option<&ProgramSource>,
        base: usize,
        grant: u32,
        mut resume: Option<ParkedCall>,
    ) -> Result<Exit, Failure> {
        let Some(stop) = chunk.op_at(end) else {
            return Err(Loud::chunk_map_too_short().into());
        };
        // **The stream cut to this range's own bound, taken once.** The loop
        // below continues only while `pc < stop`, and this slice is `stop`
        // long, so every read of it under that guard is in range already and
        // the per-op check goes. A stream shorter than `stop` is the same
        // fault the per-op read used to report, found here instead.
        let Some(full) = chunk.ops_upto(stop) else {
            return Err(Loud::chunk_map_too_short().into());
        };
        // Cut short at the clause the permission goes to, so the bound check
        // every op already makes is what finds that clause too.
        let mut stream = full.get(..grant as usize).unwrap_or(full);
        let depth = self.activation_depth();
        // Whether the permission is still worth asking about. It is granted to
        // this activation's first instruction and cleared by the one after it;
        // the first clause that finds it spent hands the rest of the range to
        // the instance that never asks.
        let mut granting = GRANTING;
        // A parked region to resume starts the counter past `stop`, so the
        // bound check every op already makes is what finds it.
        let mut pc = if resume.is_some() { u32::MAX } else { at };
        'ops: loop {
            let (flow, next) = 'step: {
                // An `Op::CallingClause` region's state, opened here or resumed:
                // assigned in place on each path rather than built as one
                // value, which the region's hot path would then copy.
                let clause_pc;
                let index;
                let region_end;
                let clause;
                let stale;
                let debugging;
                let entry;
                let from;
                // A resumed region's `DO`/`LOOP` header values so far.
                let header: Option<Box<LoopHeaderValues>>;
                'calling: {
                    // **A branch that runs out exactly at this range's own end.**
                    // `stop` is `Chunk::op_of`'s entry for `end`, which is where the
                    // branch's own `Op::EndWhen` sits -- so the op is one past what
                    // this range drives and the boundary owes the close itself.
                    if pc as usize >= stream.len() {
                        if let Some(parked) = resume.take() {
                            clause = match code.body.instructions.get(parked.index) {
                                Some(clause) => clause,
                                None => return Err(Loud::chunk_map_too_short().into()),
                            };
                            clause_pc = parked.clause_pc;
                            index = parked.index;
                            region_end = parked.end;
                            stale = parked.stale;
                            debugging = parked.debugging;
                            entry = parked.entry;
                            header = parked.header;
                            from = parked.at;
                            break 'calling;
                        }
                        if stream.len() < full.len() {
                            stream = full;
                            if pc == grant {
                                self.activity.procedure_permitted = std::mem::take(
                                    &mut self.activation_mut().first_instruction_pending,
                                );
                            }
                            continue 'ops;
                        }
                        match self
                            .leave_ended_select_branch(code, chunk, base, pc, start, end, source)?
                        {
                            BranchEnd::At(target) => {
                                pc = target;
                                continue 'ops;
                            }
                            BranchEnd::Escaped(other) => return Ok(Exit::Flow(other)),
                            BranchEnd::None => return Ok(Exit::Flow(Flow::Next)),
                        }
                    }
                    // **The tripwire for the op that replaced the check that used to
                    // stand here.** A frame is overdue only at the position its own
                    // `Op::EndWhen` occupies; anywhere else means a branch outlived
                    // the op that closes it, which is a `SELECT` running on into what
                    // follows it and is silent in any program whose branches happen to
                    // agree.
                    #[cfg(debug_assertions)]
                    if !matches!(stream.get(pc as usize), Some(Op::EndWhen)) {
                        debug_assert!(
                            !(self.activity.frames.len() > base
                                && self
                                    .activity
                                    .frames
                                    .last()
                                    .is_some_and(|frame| pc >= frame.op_end)),
                            "the innermost SELECT frame ended before op {pc}, which is not its own EndWhen"
                        );
                    }
                    let op = &stream[pc as usize];
                    match op {
                        Op::Clause {
                            index: op_index,
                            end: op_end,
                        } => {
                            let clause_pc;
                            let index;
                            let region_end;
                            let clause;
                            let stale;
                            let debugging;
                            let entry;
                            let from;
                            open_clause!(
                                self, code, chunk, source, pc, op_index, op_end, granting,
                                GRANTING, TOP, clause_pc, index, region_end, clause, stale,
                                debugging, entry, from
                            );
                            let end = region_end;
                            break 'step clause_region!(
                                self, code, chunk, registers, source, pc, clause_pc, index, clause,
                                stale, debugging, entry, end, from, None, TOP, 'ops
                            );
                        }
                        Op::CallingClause {
                            index: op_index,
                            end: op_end,
                        } => {
                            open_clause!(
                                self, code, chunk, source, pc, op_index, op_end, granting,
                                GRANTING, TOP, clause_pc, index, region_end, clause, stale,
                                debugging, entry, from
                            );
                            header = None;
                        }
                        // **A jump past this range's end is loud rather than a
                        // stop.** `absorb` cannot check it -- a jump target is an op
                        // index and absorption is decided in instruction space -- so
                        // without this the loop's own `pc < stop` reads an escaping
                        // jump as "the range completed" and answers `Flow::Next`,
                        // which is a construct silently finishing where it should have
                        // propagated. Landing exactly on `stop` *is* completion, which
                        // is what a branch-end jump at a range boundary does, so the
                        // comparison is strict. A backward jump out of the range is
                        // not checked and is not emitted: it would re-run ops inside
                        // the range, which is a wrong answer rather than a silent one,
                        // and checking it costs a second `op_at` on the hot path.
                        Op::Jump { target } => {
                            if *target > stop {
                                return Err(Loud::jump_out_of_range().into());
                            }
                            pc = *target;
                            continue 'ops;
                        }
                        // Handing an absorbed `WHEN CASE` the text it compares against
                        // (`Op::SelectCaseText`'s own doc has why it is here rather
                        // than inside the header's clause region), and opening a frame
                        // over a branch. None of the three runs a clause or produces a
                        // `Flow`, so each continues straight to the next op.
                        Op::SelectCaseText { index, case } => {
                            let value = case.map(|register| {
                        debug_assert!(
                            chunk.holds_register(register),
                            "op reads register {register} outside the region the chunk reserved"
                        );
                        registers.get(register)
                    });
                            debug_assert!(
                                code.body.instructions.get(*index as usize).is_some(),
                                "a SelectCaseText op names an instruction outside its own body"
                            );
                            self.open_select_case(value);
                            pc += 1;
                            continue 'ops;
                        }
                        // The boundary wrapper around an
                        // `IF`'s whole arm runs, which a flattened construct has no
                        // wrapper to run. `Interp::end_promoted_branch`'s doc comment
                        // has the program that says it is not a spare one.
                        Op::EndBranch => {
                            break 'step (self.end_promoted_branch(code, Flow::Next)?, pc + 1);
                        }
                        Op::EndWhen => {
                            match self.leave_ended_select_branch(
                                code, chunk, base, pc, start, end, source,
                            )? {
                                BranchEnd::At(target) => {
                                    pc = target;
                                    continue 'ops;
                                }
                                BranchEnd::Escaped(other) => return Ok(Exit::Flow(other)),
                                // The scan's own landing place when the branch this
                                // ends was never entered.
                                BranchEnd::None => break 'step (Flow::Next, pc + 1),
                            }
                        }
                        Op::EnterWhen { select, when } => {
                            let frame =
                                self.when_frame(code, chunk, *select as usize, *when as usize)?;
                            self.activity.frames.push(Frame::select(frame));
                            pc += 1;
                            continue 'ops;
                        }
                        Op::EnterOtherwise { select } => {
                            let frame = self.otherwise_frame(code, chunk, *select as usize)?;
                            self.activity.frames.push(Frame::select(frame));
                            pc += 1;
                            continue 'ops;
                        }
                        // The bottom of a flattened pass, reached by the
                        // body falling out of its last clause into the `END`'s own op.
                        Op::LoopNext { index } => {
                            if self.activity.frames.len() <= base
                                || !matches!(
                                    self.activity.frames.last().map(|f| &f.kind),
                                    Some(FrameKind::Loop)
                                )
                            {
                                return Err(Loud::op_not_driven("LoopNext").into());
                            }
                            debug_assert_eq!(
                                self.activity.flat_top.as_ref().map(|flat| flat.do_index),
                                Some(*index as usize),
                                "a LoopNext op ended a pass of a loop other than the one it names"
                            );
                            // The pass's header clause, counted here so a
                            // slice that ends at it leaves this op to run
                            // afresh.
                            let counted = match self.count_clause_against_deadline(TOP) {
                                Ok(counted) => counted,
                                Err(Failure::Slice) => return Ok(Exit::Slice(pc)),
                                Err(failure) => return Err(failure),
                            };
                            match self
                                .flat_loop_step_top(code, source, Flow::Next, |_| Ok(counted))?
                            {
                                crate::run::FlatStep::Body(op_body) => {
                                    pc = op_body;
                                    continue 'ops;
                                }
                                crate::run::FlatStep::Done(flow) => {
                                    self.activity.frames.pop();
                                    break 'step (flow, pc);
                                }
                            }
                        }
                        // The ops below are only meaningful inside a `Clause` region,
                        // which the `Op::Clause` arm above walks: reaching one here
                        // means a jump landed in the middle of a region rather than on
                        // its `Clause`. Loud rather than a panic, which is this crate's
                        // standing rule for a state the type system admits and the
                        // compiler does not produce.
                        // Every op that belongs inside a clause region, which this
                        // loop does not drive -- `undriven_op_name` has why they are
                        // one arm rather than one each.
                        _ => return Err(Loud::op_not_driven(undriven_op_name(op)).into()),
                    }
                };
                let end = region_end;
                clause_region!(
                    self, code, chunk, registers, source, pc, clause_pc, index, clause, stale,
                    debugging, entry, end, from, header.map(|header| *header), TOP, 'ops
                )
            };
            // **Per clause, not per escaping flow.** A clause that left the
            // activation stack changed makes this loop's `code` describe a
            // frame it is no longer running, and every op after it is stepped
            // against the wrong body -- so the check has to sit where a clause
            // finishes rather than where control leaves this function.
            debug_assert_eq!(
                self.activation_depth(),
                depth,
                "a clause left the activation stack changed, so this loop's `code` and its `pc` \
                 no longer describe the same frame"
            );
            match self.settle(code, chunk, base, flow, next, start, end, source)? {
                Settled::At(target) => pc = target,
                Settled::Escaped(other) => return Ok(Exit::Flow(other)),
            }
        }
    }

    /// Drops every frame this level opened, and the loop state each open loop
    /// frame stands for.
    #[cold]
    #[inline(never)]
    fn unwind_frames(&mut self, base: usize) {
        while self.activity.frames.len() > base {
            let Some(frame) = self.activity.frames.pop() else {
                return;
            };
            if matches!(frame.kind, FrameKind::Loop) {
                // The innermost open loop is the one this frame stands for.
                // `None` is a pass boundary having taken it out and then
                // raised, which loses one box to the allocator and no state.
                if let Some(flat) = self.activity.flat_top.take() {
                    self.activity.flat_spares.push(flat);
                }
                // Uncovering the loop enclosing it is the same line
                // `Interp::flat_loop_step_top` runs for a loop that ends
                // normally, and it is what keeps `flat_top` meaning "the
                // innermost loop still open" without a repair pass after this
                // one. `None` is the last of them having gone.
                self.activity.flat_top = self.activity.flat_loops.pop();
            }
        }
    }

    /// Where `flow` leaves the counter, once every open frame and then the
    /// range itself have had their say.
    #[expect(
        clippy::too_many_arguments,
        reason = "two callers inside one loop, and every argument is a value that loop holds"
    )]
    #[inline(always)]
    fn settle(
        &mut self,
        code: &Code<'_>,
        chunk: &Chunk,
        base: usize,
        mut flow: Flow,
        next: u32,
        start: usize,
        end: usize,
        source: Option<&ProgramSource>,
    ) -> Result<Settled, Failure> {
        // `Flow::Next` settles at `next` whatever is open: `absorb`
        // answers `Advance` for it against every range, and every arm below
        // turns `Advance` into `At(next)`. So the walk over the open frames is
        // skipped for the one flow that every clause of every body produces,
        // which is what makes a frame affordable to hold open across a loop's
        // whole body rather than only across a `WHEN`'s branch.
        if matches!(flow, Flow::Next) {
            return Ok(Settled::At(next));
        }
        loop {
            // Copied out rather than held, because the `Escaped` arm pops the
            // frame this came from. **`base` and not emptiness** is what ends
            // the walk: the frames below it are another level's, and this one's
            // own range has the next say.
            let Some((frame_start, frame_end)) = (self.activity.frames.len() > base)
                .then(|| self.activity.frames.last().map(|f| (f.start, f.end)))
                .flatten()
            else {
                return Ok(match absorb(flow, start, end) {
                    Absorbed::Advance => Settled::At(next),
                    Absorbed::Resume(target) => Settled::At(op_at(chunk, target)?),
                    Absorbed::Escaped(other) => Settled::Escaped(other),
                });
            };
            match absorb(flow, frame_start, frame_end) {
                Absorbed::Advance => return Ok(Settled::At(next)),
                // `target` can be the branch's own `end`, which is one past its
                // last instruction: `op_at` answers this frame's `op_end`
                // there, and the loop's own check closes the frame on the next
                // pass rather than a second rule doing it here.
                Absorbed::Resume(target) => return Ok(Settled::At(op_at(chunk, target)?)),
                Absorbed::Escaped(other) => {
                    match self
                        .activity
                        .frames
                        .pop()
                        .expect("the check above just observed one")
                        .kind
                    {
                        FrameKind::Select(frame) => {
                            flow = self.leave_branch(code, &frame, other)?;
                        }
                        // A `LEAVE`/`ITERATE` that reached this
                        // loop, decided by the same `do_body_outcome` the
                        // nested form hands the answering `Flow` to. An
                        // `ITERATE` this loop consumes puts the frame back and
                        // resumes at the body's first op -- the same place
                        // `Op::LoopNext` resumes a pass that fell through.
                        FrameKind::Loop => {
                            match self.flat_loop_step_top(code, source, other, |it| {
                                it.count_clause_against_deadline(false)
                            })? {
                                crate::run::FlatStep::Body(op_body) => {
                                    self.activity.frames.push(Frame {
                                        op_end: u32::MAX,
                                        start: frame_start,
                                        end: frame_end,
                                        kind: FrameKind::Loop,
                                    });
                                    return Ok(Settled::At(op_body));
                                }
                                crate::run::FlatStep::Done(escape) => flow = escape,
                            }
                        }
                    }
                }
            }
        }
    }

    /// What a `SELECT` does with a `Flow` that left one of its branches:
    /// `select_escape` and `leave_select`, exactly what `step`'s own `Select`
    /// arm and `Op::EnterOtherwise` do with the same `Flow`, and then the boundary
    /// wrapper around the whole arm runs.
    fn leave_branch(
        &mut self,
        code: &Code<'_>,
        frame: &SelectFrame,
        flow: Flow,
    ) -> Result<Flow, Failure> {
        let flow = match frame.branch {
            Branch::When => match select_escape(frame.otherwise, flow) {
                // Still inside this `SELECT`: `EnterOtherwise` at the target's
                // own entry opens the next branch's frame, and this `WHEN`'s
                // branch did not finish -- control was redirected out of it,
                // so the end-of-branch boundary below is not owed.
                SelectEscape::Otherwise(target) => return Ok(Flow::Goto(target)),
                SelectEscape::Forward(flow) => flow,
            },
            // `Interp::leave_otherwise` is the whole of leaving this branch,
            // shared with `Op::EnterOtherwise`: the escape elevation is restored
            // and `leave_select` decides where control goes. **And no
            // end-of-branch boundary follows it**, because `OTHERWISE`'s
            // branch ends at the `END`, a real instruction with a boundary of
            // its own -- measured, a handler queued by the last clause of an
            // `OTHERWISE` and re-queued by its own handler is delivered at the
            // `END`'s line, not at the branch's.
            Branch::Otherwise => {
                return self.leave_otherwise(
                    code,
                    frame.select,
                    frame.label,
                    frame.end,
                    frame.select_end,
                    flow,
                );
            }
        };
        let flow = self.leave_select(code, frame.select, frame.label, frame.resume, flow)?;
        // A matched `WHEN`'s branch is one the oracle closes with a synthetic
        // instruction, so it owes that instruction's boundary.
        self.end_promoted_branch(code, flow)
    }

    /// The frame [`Op::EnterWhen`] opens: the matched branch of the listed
    /// `WHEN` at `when`, belonging to the `SELECT` at `select`.
    fn when_frame(
        &self,
        code: &Code<'_>,
        chunk: &Chunk,
        select: usize,
        when: usize,
    ) -> Result<SelectFrame, Failure> {
        let (Some(select_instruction), Some(when_instruction)) = (
            code.body.instructions.get(select),
            code.body.instructions.get(when),
        ) else {
            return Err(Loud::chunk_map_too_short().into());
        };
        let Some(parts) = select_parts(&select_instruction.kind) else {
            return Err(Loud::select_op_off_its_node().into());
        };
        let targets = when_targets(&when_instruction.kind, code.body.instructions.len());
        Ok(SelectFrame {
            select,
            label: parts.label,
            otherwise: parts.otherwise,
            select_end: parts.end,
            start: when + 1,
            end: targets.body_end,
            resume: when_resume(&targets),
            op_end: op_at(chunk, targets.body_end)?,
            branch: Branch::When,
        })
    }

    /// The frame [`Op::EnterOtherwise`] opens: the `OTHERWISE` branch of the
    /// `SELECT` at `select`.
    fn otherwise_frame(
        &self,
        code: &Code<'_>,
        chunk: &Chunk,
        select: usize,
    ) -> Result<SelectFrame, Failure> {
        let Some(select_instruction) = code.body.instructions.get(select) else {
            return Err(Loud::chunk_map_too_short().into());
        };
        let Some(parts) = select_parts(&select_instruction.kind) else {
            return Err(Loud::select_op_off_its_node().into());
        };
        let Some(otherwise) = parts.otherwise else {
            return Err(Loud::select_op_off_its_node().into());
        };
        let otherwise_end = otherwise_range(code.body.instructions.len(), parts.end);
        Ok(SelectFrame {
            select,
            label: parts.label,
            otherwise: parts.otherwise,
            select_end: parts.end,
            start: otherwise,
            end: otherwise_end,
            resume: otherwise_resume(code.body.instructions.len(), parts.end),
            op_end: op_at(chunk, otherwise_end)?,
            branch: Branch::Otherwise,
        })
    }

    /// Whether register `reg` holds the Rexx logical value `1`.
    fn register_holds(&self, registers: RegFrame<'_>, reg: u16) -> Result<bool, Failure> {
        let value = registers.get(reg);
        // The two handles a logical arrives in, compared as integers: the
        // constant a comparison answers with, and the small int
        // `Op::WhenTest` writes back for everything else.
        if value == crate::eval::LOGICAL_TRUE {
            return Ok(true);
        }
        if value == crate::eval::LOGICAL_FALSE {
            return Ok(false);
        }
        match value.decode() {
            Decoded::SmallInt(1) => Ok(true),
            Decoded::SmallInt(0) => Ok(false),
            _ => Err(Loud::register_not_logical().into()),
        }
    }
}

/// Asserts, in debug, that the op naming instruction `index` from inside a
/// clause region names that region's own clause.
fn debug_assert_names_the_clause(code: &Code<'_>, index: u32, clause: &Instruction, op: &str) {
    debug_assert_eq!(
        code.body
            .instructions
            .get(index as usize)
            .map(std::ptr::from_ref),
        Some(std::ptr::from_ref(clause)),
        "a {op} op names an instruction that is not the clause of the region it sits in"
    );
}

/// The position of the op after `op` in its chunk's stream, for `op` one of
/// `ops`, which start at position `from`.
fn op_after(ops: &[Op], op: &Op, from: u32) -> u32 {
    let offset =
        (std::ptr::from_ref(op) as usize - ops.as_ptr() as usize) / std::mem::size_of::<Op>();
    debug_assert!(
        std::ptr::eq(&ops[offset], op),
        "an op outside the region it was found in"
    );
    from + offset as u32 + 1
}

/// The op instruction `target` resumes at, or the loud failure a chunk whose
/// map is shorter than its own body earns.
fn op_at(chunk: &Chunk, target: usize) -> Result<u32, Failure> {
    chunk
        .op_at(target)
        .ok_or_else(|| Loud::chunk_map_too_short().into())
}

impl Interp {
    /// A call op's resolution before its arguments run, for an op that
    /// evaluates them itself: the site's kept answer, or the part of the
    /// resolution the oracle decides before the arguments, kept when it
    /// decides one.
    ///
    /// **A raise is deliberately not recorded**: `resolve_fixed_call` answers
    /// `Err` for a name it refuses, and a site that raised asks again. A kept
    /// answer of a kind the oracle searches for on every call is handed on
    /// with the generation it was read under, since the arguments can write a
    /// routine table before it is used.
    fn site_resolution_before_arguments<'c>(
        &self,
        chunk: &'c Chunk,
        site: u16,
        name: &[u8],
        search_labels: bool,
    ) -> Result<CallResolution<'c>, Failure> {
        let generation = self.routine_generation;
        if let Some(resolved) = chunk.resolved_call(site, generation) {
            #[cfg(test)]
            count_call_site_hit();
            if !resolved.kept_until_routines_change() {
                return Ok(CallResolution::Settled(resolved));
            }
            return Ok(CallResolution::AfterArguments {
                kept: Some((resolved, generation)),
                site: Some(chunk.call_site_slot(site)),
            });
        }
        Ok(match self.resolve_fixed_call(name, search_labels)? {
            Some(resolved) => {
                chunk.remember_call(site, resolved, generation);
                CallResolution::Settled(resolved)
            }
            None => CallResolution::AfterArguments {
                kept: None,
                site: Some(chunk.call_site_slot(site)),
            },
        })
    }
}

#[cfg(test)]
mod tests;
