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

//! Which `TRACE` setting is in force at each instruction of a body, as a
//! forward dataflow analysis in Kildall's framework ("A Unified Approach to
//! Global Program Optimization", 1973): a meet-semilattice of pools, an
//! optimizing function per node, and a worklist iterated to a fixpoint.
//!
//! The edge set is a **superset** of the body's own control flow, which is
//! what makes a missing edge impossible; [`successors`] and [`Blocks`] state
//! each edge and what it stands in for, and [`analyse`] states the
//! preconditions that make that superset enough.

use rexx_parse::{CodeBody, Instruction, InstructionKind};

use crate::plan::Plan;
use crate::trace::{ChunkTrace, TraceEvent};

/// One pool: the `TRACE` setting in force at an instruction, as far as the
/// source fixes it.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub(crate) enum Setting {
    /// The lattice's top: no path to this instruction has been seen yet.
    Unreached,
    /// The setting every path reaching this instruction leaves in force.
    Known(ChunkTrace),
    /// The lattice's bottom: two paths here disagree, or one of them ran
    /// something whose setting the source does not fix.
    Unknown,
}

impl Setting {
    /// The meet: agreement gives the setting, disagreement gives bottom, and
    /// top meets anything to that thing.
    fn meet(self, other: Setting) -> Setting {
        match (self, other) {
            (Setting::Unreached, answer) | (answer, Setting::Unreached) => answer,
            (Setting::Known(one), Setting::Known(two)) if one == two => Setting::Known(one),
            _ => Setting::Unknown,
        }
    }

    /// Whether a clause under this answer carries the value-echo ops
    /// (`>L>`/`>V>`/`>O>`/`>P>`/`>A>`/`>F>`). An instruction the analysis
    /// could not settle keeps them and keeps its run-time gate.
    pub(crate) fn echoes_values(self) -> bool {
        match self {
            Setting::Known(trace) => trace.intermediates(),
            Setting::Unreached | Setting::Unknown => true,
        }
    }

    /// The same for a `DO`/`LOOP` header's `>K>` op, which asks
    /// [`ChunkTrace::results`] rather than the bit above.
    pub(crate) fn echoes_keyword(self) -> bool {
        match self {
            Setting::Known(trace) => trace.results(),
            Setting::Unreached | Setting::Unknown => true,
        }
    }
}

/// The virtual node every `LABEL` is entered from, at index `len`.
///
/// A label is reachable from outside this body's own edges in ways no walk of
/// the instructions enumerates -- `SIGNAL` and `SIGNAL VALUE`, a condition
/// trap firing at an arbitrary clause, an internal routine call entering the
/// chunk at the label, and a `SIGNAL` inside an `INTERPRET`ed fragment. Every
/// instruction is given an edge to this node and this node an edge to every
/// label, so a label's answer is the meet over the whole body and the entry
/// pool -- which is what every one of those routes would contribute.
struct Graph {
    /// The `LABEL` instructions, which is what the virtual node's own edges
    /// are.
    labels: Box<[usize]>,
    /// The edges [`Blocks`] found, by node: a loop's back edge and zero-trip
    /// edge, and a `LEAVE`/`ITERATE`'s reach.
    extra: Box<[Box<[usize]>]>,
    /// One past the last instruction, which is the virtual node's index.
    virtual_node: usize,
}

/// The nodes `node`'s output pool reaches: the fallthrough, the edges
/// [`Blocks`] found for it, and the virtual label node.
///
/// `node + 1` is given to every instruction, including the ones that never
/// fall through -- a spurious edge only lowers an answer.
///
/// **A forward branch needs no edge of its own.** It skips only instructions
/// inside the arm it is skipping, so the chain of fallthrough edges across
/// them carries exactly what the branch edge would have -- which rests on
/// [`analyse`]'s second precondition, that no instruction inside an `IF` or a
/// `SELECT` changes the setting.
fn successors(graph: &Graph, node: usize, out: &mut Vec<usize>) {
    out.clear();
    if node == graph.virtual_node {
        out.extend_from_slice(&graph.labels);
        return;
    }
    if node + 1 < graph.virtual_node {
        out.push(node + 1);
    }
    out.extend_from_slice(&graph.extra[node]);
    if !graph.labels.is_empty() {
        out.push(graph.virtual_node);
    }
}

/// One walk of a body's block structure: which instructions sit inside an
/// `IF` or a `SELECT`, and the edges a `DO`/`LOOP` and a `LEAVE`/`ITERATE`
/// contribute.
struct Blocks {
    /// Whether the instruction sits anywhere inside an `IF` or a `SELECT`.
    /// **Initialised to `true`**, so an instruction [`Blocks::scan`] does not
    /// reach keeps the answer that refuses the body.
    branchy: Box<[bool]>,
    /// The extra successor edges, by node, built by [`Blocks::edges`].
    extra: Vec<Vec<usize>>,
}

impl Blocks {
    /// Walks `instructions` once and answers both.
    ///
    /// The dispatch on `IF`/`SELECT`/`DO`/`LOOP` is `run::static_indent`'s,
    /// reading the same fields to walk the same ranges.
    fn of(instructions: &[Instruction]) -> Blocks {
        let len = instructions.len();
        let mut blocks = Blocks {
            branchy: vec![true; len].into_boxed_slice(),
            extra: vec![Vec::new(); len],
        };
        let mut enclosing = Vec::new();
        blocks.scan(instructions, 0, len, false, &mut enclosing);
        blocks
    }

    /// `[start, end)` at one nesting, marking each instruction and recording
    /// each block's own edges.
    fn scan(
        &mut self,
        instructions: &[Instruction],
        start: usize,
        end: usize,
        branchy: bool,
        enclosing: &mut Vec<(usize, usize)>,
    ) {
        let len = instructions.len();
        let mut pc = start;
        while pc < end {
            self.branchy[pc] = branchy;
            match &instructions[pc].kind {
                InstructionKind::If { false_target, .. } => {
                    let false_target = false_target.unwrap_or(len);
                    self.scan(instructions, pc + 1, false_target, true, enclosing);
                    pc = match instructions.get(false_target).map(|at| &at.kind) {
                        Some(InstructionKind::Else { then_exit }) => {
                            let else_end = then_exit.unwrap_or(len);
                            self.scan(instructions, false_target, else_end, true, enclosing);
                            else_end
                        }
                        _ => false_target,
                    };
                }
                InstructionKind::Do(loop_) | InstructionKind::Loop(loop_) => {
                    // An unclosed `DO` is error 14.1/14.5, so a body that
                    // parsed has this set; a body that somehow does not keeps
                    // the `true` every entry was initialised to.
                    let Some(end_index) = loop_.end else { return };
                    // **The back edge and the zero-trip edge**, which are what
                    // lets a `TRACE` sit inside a loop. The header is reached
                    // both from before the loop and from the `END`, so it
                    // answers the meet; the instruction past the `END` is
                    // reached both from the `END` and -- when the loop runs no
                    // passes at all -- from the header, so it answers the meet
                    // too. `DO FOREVER` and `DO UNTIL` cannot trip zero times
                    // and are given the edge anyway: a spurious edge only
                    // lowers an answer, and deciding which loops can be empty
                    // is the reasoning this edge set exists to avoid.
                    if end_index < len {
                        self.extra[end_index].push(pc);
                        if end_index + 1 < len {
                            self.extra[pc].push(end_index + 1);
                        }
                    }
                    enclosing.push((pc, end_index));
                    self.scan(instructions, pc + 1, end_index, branchy, enclosing);
                    enclosing.pop();
                    if end_index < len {
                        self.branchy[end_index] = branchy;
                    }
                    pc = end_index + 1;
                }
                InstructionKind::Select {
                    end: select_end, ..
                } => {
                    let select_end = select_end.unwrap_or(len);
                    enclosing.push((pc, select_end));
                    self.scan(instructions, pc + 1, select_end, true, enclosing);
                    enclosing.pop();
                    pc = select_end + 1;
                }
                // **Every enclosing block, not the innermost one**, and both
                // of its ends: a named `LEAVE` or `ITERATE` reaches any block
                // it is inside, and which one is a run-time match against the
                // name. The `ITERATE` target is the header and the `LEAVE`
                // target the instruction past the `END`; giving both to both
                // is the same spurious-edge trade as above.
                InstructionKind::Leave { .. } | InstructionKind::Iterate { .. } => {
                    for &(header, block_end) in enclosing.iter() {
                        self.extra[pc].push(header);
                        if block_end + 1 < len {
                            self.extra[pc].push(block_end + 1);
                        }
                    }
                    pc += 1;
                }
                _ => pc += 1,
            }
        }
    }
}

/// The setting in force at each instruction of `body`, entered under `entry`.
///
/// Answers [`Setting::Unknown`] for every instruction rather than refusing,
/// which is the answer that emits exactly what the body emitted before this
/// analysis existed. It does so unless both preconditions hold:
///
/// * **`entry` is not in interactive debug.** A line typed at a pause changes
///   the setting from outside the body, and no analysis of the body sees it.
/// * **No instruction that changes the setting sits inside an `IF` or a
///   `SELECT`** ([`Blocks::branchy`]). This is what [`successors`] rests on:
///   a forward branch skipping a `TRACE` inside its own arm is an edge that
///   would have to be modelled, and this refuses the body instead. A `TRACE`
///   inside a loop needs no such refusal, because [`Blocks`] gives the loop
///   its two edges.
pub(crate) fn analyse(body: &CodeBody, plan: &Plan, entry: ChunkTrace) -> Box<[Setting]> {
    let len = body.instructions.len();
    let unknown = || vec![Setting::Unknown; len].into_boxed_slice();
    let events = plan.trace_events();
    if events.len() != len || entry.debugging() {
        return unknown();
    }
    let blocks = Blocks::of(&body.instructions);
    let settles_outside_a_branch = events
        .iter()
        .enumerate()
        .all(|(index, event)| *event == TraceEvent::Keeps || !blocks.branchy[index]);
    if !settles_outside_a_branch {
        return unknown();
    }

    let graph = Graph {
        labels: body
            .instructions
            .iter()
            .enumerate()
            .filter(|(_, instruction)| matches!(instruction.kind, InstructionKind::Label { .. }))
            .map(|(index, _)| index)
            .collect(),
        extra: blocks
            .extra
            .into_iter()
            .map(Vec::into_boxed_slice)
            .collect(),
        virtual_node: len,
    };

    // The entry pool reaches instruction 0 and, through the virtual node,
    // every label -- the two places a run of this chunk can start.
    let mut pools = vec![Setting::Unreached; len + 1];
    pools[graph.virtual_node] = Setting::Known(entry);
    let mut queued = vec![false; len + 1];
    let mut work = Vec::with_capacity(len + 1);
    let push = |node: usize, work: &mut Vec<usize>, queued: &mut Vec<bool>| {
        if !queued[node] {
            queued[node] = true;
            work.push(node);
        }
    };
    if len > 0 {
        pools[0] = Setting::Known(entry);
        push(0, &mut work, &mut queued);
    }
    push(graph.virtual_node, &mut work, &mut queued);

    let mut reached = Vec::new();
    while let Some(node) = work.pop() {
        queued[node] = false;
        let out = apply(events, graph.virtual_node, node, pools[node]);
        successors(&graph, node, &mut reached);
        for &next in &reached {
            let met = pools[next].meet(out);
            if met != pools[next] {
                pools[next] = met;
                push(next, &mut work, &mut queued);
            }
        }
    }

    // **A fixpoint, checked rather than assumed.** One more pass over every
    // node may not lower anything; a worklist that forgets to re-queue a node
    // it lowered leaves an answer that is too high, which is the one direction
    // that is unsound.
    #[cfg(debug_assertions)]
    for node in 0..=graph.virtual_node {
        let out = apply(events, graph.virtual_node, node, pools[node]);
        successors(&graph, node, &mut reached);
        for &next in &reached {
            debug_assert_eq!(
                pools[next].meet(out),
                pools[next],
                "node {node} still lowers {next}, so the worklist stopped short of a fixpoint"
            );
        }
    }

    pools.truncate(len);
    pools.into_boxed_slice()
}

/// The answer **emission** may use at instruction `index`, given the pool
/// [`analyse`] answered for it.
///
/// An instruction that can change the setting can change it part-way through
/// its own clause, and the ops after that point in the clause run under the
/// new setting rather than under the pool the clause was entered with.
/// Measured: `trace off` and then `zg = trace('i') 'tail'` echoes
/// `>L>   "tail"` and `>O>   " " => "O tail"`, the concatenation's own value
/// lines, under the `I` that the call in the same clause has just installed
/// -- and emitting that clause for the `OFF` it was entered under loses both.
pub(crate) fn for_emission(plan: &Plan, index: usize, pool: Setting) -> Setting {
    match plan.trace_events().get(index) {
        Some(TraceEvent::Keeps) => pool,
        _ => Setting::Unknown,
    }
}

/// The optimizing function: an instruction that does not change the setting
/// passes its pool through, one whose own text fixes a setting produces that
/// setting, and one that can change it to something the source does not fix
/// produces bottom. The virtual label node passes its pool through.
fn apply(events: &[TraceEvent], virtual_node: usize, node: usize, pool: Setting) -> Setting {
    if node == virtual_node {
        return pool;
    }
    match events[node] {
        TraceEvent::Keeps => pool,
        TraceEvent::Sets(trace) => Setting::Known(trace),
        TraceEvent::Unknown => Setting::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::trace::TraceMode;

    /// The analysis over one program's main body, entered under `entry`.
    fn settings_of(source: &[u8], entry: TraceMode) -> Vec<Setting> {
        let program = rexx_parse::parse_program(source.to_vec()).expect("a program that parses");
        let plan = Plan::build(
            &program.main,
            &program.symbols,
            Some(&program.source),
            crate::plan::BodyKind::Plain,
        );
        analyse(&program.main, &plan, ChunkTrace::of(entry)).to_vec()
    }

    /// `TRACE OFF`'s own answer, which is also `TRACE N`'s: the two differ
    /// only in `TraceMode::failures`, which no chunk's emission reads.
    fn off() -> Setting {
        Setting::Known(ChunkTrace::of(TraceMode::OFF))
    }

    /// A body with no `TRACE` in it answers the entry setting everywhere,
    /// which is what `Plan::never_retraces` already decided for it -- so the
    /// analysis cannot be less precise than what it replaces.
    #[test]
    fn a_body_that_never_retraces_is_known_at_every_clause() {
        let settings = settings_of(b"zw = 1\nzv = zw + 1\nsay zv", TraceMode::NORMAL);
        assert_eq!(settings, vec![off(); 3]);
        let intermediates = Setting::Known(ChunkTrace::of(TraceMode::INTERMEDIATES));
        assert_eq!(
            settings_of(b"zw = 1\nzv = zw + 1\nsay zv", TraceMode::INTERMEDIATES),
            vec![intermediates; 3]
        );
    }

    /// `TRACE OFF` under the setting every program starts in: the whole body
    /// is known, because `NORMAL` and `OFF` are one `ChunkTrace`. This is the
    /// case the item exists for.
    #[test]
    fn trace_off_from_normal_leaves_the_whole_body_known() {
        assert_eq!(
            settings_of(b"trace off\nzw = 1\nzv = zw + 1\nsay zv", TraceMode::NORMAL),
            vec![off(); 4]
        );
    }

    /// `TRACE OFF` while `TRACE I` is in force splits the body: the `TRACE`
    /// clause itself still runs under the setting it is replacing, and every
    /// clause after it runs under the one it sets.
    #[test]
    fn trace_off_from_intermediates_settles_what_follows_it() {
        let intermediates = Setting::Known(ChunkTrace::of(TraceMode::INTERMEDIATES));
        assert_eq!(
            settings_of(b"trace off\nsay 'a'\nsay 'b'", TraceMode::INTERMEDIATES),
            vec![intermediates, off(), off()]
        );
    }

    /// `TRACE VALUE` computes its setting at run time, so it is bottom --
    /// and bottom reaches the clauses after it and no others. The first two
    /// entries are the control: without them this would also pass for an
    /// analysis that answered bottom for every body containing one.
    #[test]
    fn trace_value_leaves_what_follows_it_unknown() {
        assert_eq!(
            settings_of(b"say 'a'\ntrace value zv\nsay 'b'", TraceMode::NORMAL),
            vec![off(), off(), Setting::Unknown]
        );
    }

    /// An `INTERPRET` can run a `TRACE` in this very activation -- measured
    /// against the oracle, a `trace i` inside one traces the clauses after it
    /// in the enclosing body -- so it is bottom.
    #[test]
    fn interpret_leaves_what_follows_it_unknown() {
        assert_eq!(
            settings_of(b"trace off\ninterpret zv\nsay 'b'", TraceMode::NORMAL),
            vec![off(), off(), Setting::Unknown]
        );
    }

    /// The `TRACE()` builtin changes the setting in the activation that calls
    /// it -- measured -- so a call that could be it is bottom.
    #[test]
    fn the_trace_builtin_leaves_what_follows_it_unknown() {
        assert_eq!(
            settings_of(b"trace off\nzg = trace('i')\nsay 'b'", TraceMode::NORMAL),
            vec![off(), off(), Setting::Unknown]
        );
    }

    /// A `TRACE` inside an `IF` or a `SELECT` is the shape [`successors`]
    /// does not model, because the branch edge around it would have to carry
    /// the setting from before it: the body is refused whole.
    #[test]
    fn a_trace_inside_a_branch_refuses_the_body() {
        for source in [
            &b"trace i\nif 0 then trace off\nsay 'x'"[..],
            &b"trace i\nif 0 then nop\nelse trace off\nsay 'x'"[..],
            &b"trace i\nselect\n  when 0 then trace off\n  otherwise nop\nend\nsay 'x'"[..],
            &b"trace i\nselect\n  when 0 then nop\n  otherwise trace off\nend\nsay 'x'"[..],
        ] {
            let settings = settings_of(source, TraceMode::NORMAL);
            assert!(
                settings.iter().all(|setting| *setting == Setting::Unknown),
                "{settings:?}"
            );
        }
        // The control: the same `TRACE OFF`, moved out to the top level,
        // settles the whole body. Without this the four above would also pass
        // for an analysis that refused every body carrying a `TRACE`.
        assert_eq!(
            settings_of(
                b"trace off\ndo zi = 1 to 3\n  say zi\nend",
                TraceMode::NORMAL
            ),
            vec![off(); 4]
        );
    }

    /// A `TRACE` inside a loop settles the clauses after it **in the loop**,
    /// and the loop's own two edges are what keep the header and the
    /// instruction past the `END` honest.
    ///
    /// `TRACE OFF` under a `NORMAL` entry is the case where every one of them
    /// agrees, because the two are one [`ChunkTrace`]: the whole body settles.
    #[test]
    fn a_trace_off_inside_a_loop_settles_the_whole_body() {
        assert_eq!(
            settings_of(
                b"do zi = 1 to 3\n  trace off\n  say zi\nend\nsay 'x'",
                TraceMode::NORMAL
            ),
            vec![off(); 5]
        );
    }

    /// The same shape with a setting the entry disagrees with, which is what
    /// says the two edges are load-bearing rather than decoration.
    ///
    /// * the **header** is bottom: it runs under the entry setting on the
    ///   first pass and under `I` on every later one, which is the back edge;
    /// * the clauses **after** the `TRACE`, inside the loop, are `Known(I)` --
    ///   and they are the hot half;
    /// * the instruction **past the `END`** is bottom, because a loop that
    ///   runs no passes at all leaves the entry setting in force, which is the
    ///   zero-trip edge.
    #[test]
    fn a_trace_i_inside_a_loop_settles_the_body_and_not_its_ends() {
        let intermediates = Setting::Known(ChunkTrace::of(TraceMode::INTERMEDIATES));
        assert_eq!(
            settings_of(
                b"do zi = 1 to 3\n  trace i\n  say zi\nend\nsay 'x'",
                TraceMode::NORMAL
            ),
            vec![
                Setting::Unknown,
                Setting::Unknown,
                intermediates,
                intermediates,
                Setting::Unknown,
            ]
        );
    }

    /// A `LEAVE` reaches the instruction past its loop's `END` without
    /// running what lies between it and the `END`, so that instruction
    /// answers the meet of the setting at the `LEAVE` and the one the `END`
    /// was reached under.
    ///
    /// Measured against the oracle on this very body, rc 0: the `LEAVE` fires
    /// on the second pass under the `I` the clause above it installed, and
    /// `say 'x'` echoes `>L>   "x"` and `>>>   "x"`. Without the `LEAVE`'s
    /// edge the answer there is `Known(OFF)`, which drops both lines.
    #[test]
    fn a_leave_carries_the_setting_from_before_the_trace_after_it() {
        let settings = settings_of(
            b"do zi = 1 to 3\n  trace i\n  if zi = 2 then leave\n  trace off\nend\nsay 'x'",
            TraceMode::NORMAL,
        );
        assert_eq!(
            settings.last().copied(),
            Some(Setting::Unknown),
            "the instruction past the END, which the LEAVE reaches: {settings:?}"
        );
        // The control: the same body with the `LEAVE` replaced by a `NOP`
        // settles there, so the bottom above is the `LEAVE`'s edge doing it
        // rather than anything else in the shape.
        assert_eq!(
            settings_of(
                b"do zi = 1 to 3\n  trace i\n  if zi = 2 then nop\n  trace off\nend\nsay 'x'",
                TraceMode::NORMAL
            )
            .last()
            .copied(),
            Some(off())
        );
    }

    /// A label is entered from outside this body's own edges, so it answers
    /// the meet of the entry pool and every instruction's output: known where
    /// they agree, and bottom as soon as one of them does not.
    #[test]
    fn a_label_answers_the_meet_over_the_whole_body() {
        assert_eq!(
            settings_of(
                b"trace off\nsay 'a'\nexit\nzsub:\nsay 'b'\nreturn",
                TraceMode::NORMAL
            ),
            vec![off(); 6],
            "every setting this body puts in force is the entry one"
        );

        let disagreeing = settings_of(
            b"trace i\nsay 'a'\nexit\nzsub:\nsay 'b'\nreturn",
            TraceMode::NORMAL,
        );
        assert_eq!(
            disagreeing[3..],
            [Setting::Unknown; 3],
            "the label and what follows it, against an entry the body disagrees with"
        );
        assert_eq!(
            disagreeing[1],
            Setting::Known(ChunkTrace::of(TraceMode::INTERMEDIATES)),
            "and the clause after the TRACE I, which the label does not reach"
        );
    }

    /// Interactive debug changes the setting from a place the body does not
    /// contain, so a chunk entered under it is refused whole.
    #[test]
    fn a_debug_entry_setting_refuses_the_body() {
        let debugging = TraceMode {
            debug: true,
            ..TraceMode::RESULTS
        };
        assert_eq!(
            settings_of(b"trace off\nsay 'a'", debugging),
            vec![Setting::Unknown; 2]
        );
        // The same body entered without the flag, which is what says the
        // refusal above is the flag's doing and not the body's.
        assert_eq!(
            settings_of(b"trace off\nsay 'a'", TraceMode::RESULTS),
            vec![Setting::Known(ChunkTrace::of(TraceMode::RESULTS)), off()]
        );
    }
}
