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
//! what makes a missing edge impossible; [`successors`] states each edge and
//! what it stands in for, and [`analyse`] states the two preconditions that
//! make that superset enough.

use rexx_parse::{CodeBody, InstructionKind};

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
    /// One past the last instruction, which is the virtual node's index.
    virtual_node: usize,
}

/// The nodes `node`'s output pool reaches: the fallthrough, and the virtual
/// label node.
///
/// `node + 1` is given to every instruction, including the ones that never
/// fall through -- a spurious edge only lowers an answer.
///
/// **The two edge kinds a structured body also has need no entry of their
/// own**, and both arguments rest on [`analyse`]'s second precondition, that
/// no instruction inside a construct changes the setting:
///
/// * A **forward branch** skips only instructions inside the arm it is
///   skipping, so the chain of fallthrough edges across them carries exactly
///   what the branch edge would have.
/// * A **loop's backward edge** carries the pool at the `END` to the header.
///   Everything between them passes its pool through, so the value arriving
///   is the header's own output -- which the header already has, if it is not
///   an event, and which it ignores, if it is.
fn successors(graph: &Graph, node: usize, out: &mut Vec<usize>) {
    out.clear();
    if node == graph.virtual_node {
        out.extend_from_slice(&graph.labels);
        return;
    }
    if node + 1 < graph.virtual_node {
        out.push(node + 1);
    }
    if !graph.labels.is_empty() {
        out.push(graph.virtual_node);
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
/// * **Every instruction that changes the setting sits at the body's own top
///   level** (`Plan::indents` answers 0 for it). This is what [`successors`]
///   rests on: a forward branch skipping a `TRACE` inside its own arm is an
///   edge that would have to be modelled, and this refuses the body instead.
pub(crate) fn analyse(body: &CodeBody, plan: &Plan, entry: ChunkTrace) -> Box<[Setting]> {
    let len = body.instructions.len();
    let unknown = || vec![Setting::Unknown; len].into_boxed_slice();
    let events = plan.trace_events();
    if events.len() != len || entry.debugging() {
        return unknown();
    }
    let settles_at_top_level = events.iter().enumerate().all(|(index, event)| {
        *event == TraceEvent::Keeps || plan.indent_of(&body.instructions, index) == 0
    });
    if !settles_at_top_level {
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

    /// A `TRACE` anywhere but the body's own top level is the shape
    /// [`successors`] does not model, because the branch or loop edge around
    /// it would have to carry the setting from before it: the body is refused
    /// whole, in both of the two shapes that reach it.
    #[test]
    fn a_trace_below_the_top_level_refuses_the_body() {
        for source in [
            &b"trace i\nif 0 then trace off\nsay 'x'"[..],
            &b"do zi = 1 to 3\n  trace off\n  say zi\nend\nsay 'x'"[..],
        ] {
            let settings = settings_of(source, TraceMode::NORMAL);
            assert!(
                settings.iter().all(|setting| *setting == Setting::Unknown),
                "{settings:?}"
            );
        }
        // The control: the same `TRACE OFF`, moved out to the top level,
        // settles the whole body. Without this the two above would also pass
        // for an analysis that refused every body carrying a `TRACE`.
        assert_eq!(
            settings_of(
                b"trace off\ndo zi = 1 to 3\n  say zi\nend",
                TraceMode::NORMAL
            ),
            vec![off(); 4]
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
