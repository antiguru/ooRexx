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

//! The in-process external data queue (I15) that `PUSH` and `QUEUE` write to
//! (`run.rs`'s own arms for both). One per `Interp`, held as a plain field
//! rather than anything IPC-backed -- see "Why not cross-process" below.
//!
//! `PULL` and `PARSE PULL` read it back, through `Interp::pull_line`
//! (`input.rs`), which is where the "queue first, then `.input`" rule lives
//! rather than here: this type knows the order its own two writers produce and
//! nothing about the console.
//!
//! This module's own tests read the stored order back through the private
//! `lines` field as well as through [`Queue::pop`], the same way `stem.rs`'s
//! tests reach into `Body::Stem` directly -- both the type-level tests below,
//! which construct a `Queue` directly, and
//! `tests::push_and_queue_actually_write_into_the_running_interpreters_queue`,
//! which runs a program through a real `Interp` and reads `Interp::queue`
//! back the same way. The second exists because the first two cannot see
//! whether `run.rs`'s `step` arms ever call `Queue::push`/`Queue::queue` at
//! all -- review round 1's I3 found that deleting just those two call sites
//! (keeping the evaluation and the trace) left every other gate green,
//! because nothing ran a program through the interpreter and then read the
//! queue back.
//!
//! # LIFO and FIFO, and which end is which
//!
//! `PUSH` inserts at the head; `QUEUE` appends at the tail. Both share the
//! oracle's `RexxInstructionQueue` (`QueueInstruction.cpp`) and differ only
//! in `Activity::QueueOrder` (`QUEUE_LIFO` for `PUSH`, `QUEUE_FIFO` for
//! `QUEUE`) -- `ast.rs`'s own doc comment on `InstructionKind::Push`/`Queue`
//! already says the two keywords share one C++ class. The head is the next
//! line [`Queue::pop`] removes, which is what makes `PUSH` (to the head) the
//! LIFO order and `QUEUE` (to the tail) the FIFO one relative to it.
//!
//! Measured against the oracle, `push "a"` then `queue "b"` then
//! `push "c"`, then three bare `PULL`s into `v1`/`v2`/`v3`, each `SAY`n:
//!
//! ```text
//! C
//! A
//! B
//! ```
//!
//! `PULL` upcases what it reads (it is `PARSE UPPER PULL`), so the queue's
//! own stored order -- before that transform -- is `c`, `a`, `b`: `push "c"`
//! landed ahead of both earlier entries (LIFO, at the head), and `queue "b"`
//! landed behind `push "a"` rather than ahead of it (FIFO, at the tail).
//! Quoted literals were used rather than the brief's own bare `a`/`b`/`c`
//! shorthand so the measurement can tell "order" and "the upcase-at-`PULL`
//! transform" apart: an unquoted symbol already reads as its own upcased
//! name, so a probe built from bare symbols could not distinguish the queue
//! storing values verbatim from the queue upcasing them itself at `PUSH`/
//! `QUEUE` time. This probe can, and does: the values reappear as `C`/`A`/
//! `B` only through `PULL`'s own transform, not because `queue.rs` folded
//! their case on the way in.
//!
//! **The "head is what `PULL` removes" half of that used to be an assumption
//! nothing enforced (review round 1, M5), and [`Queue::pop`] is what closed
//! it.** A `pop_back` implementation would leave both type-level tests in
//! this file green -- they assert the stored order, which `pop_back` does not
//! change -- while printing `B`, `A`, `C` for the probe above. What
//! distinguishes the two is a program that pushes and then pulls, which is
//! `corpus/lang/pull_queue.rex` differentially and
//! `interleaved_push_and_queue_survive_a_round_trip` below in crate.
//!
//! # Why not cross-process
//!
//! The oracle's own queue is `rxapi`-backed and, per the design's scoping
//! document (I15), was recorded as making a cross-process differential run
//! of `QUEUED`/`PULL` impossible "live rather than theoretical" on the
//! strength of `rxapi` being confirmed running. That reasoning conflated two
//! different questions -- whether a daemon is reachable, and whether two
//! separate process invocations actually observe each other's pushes.
//! Measured directly instead: a program whose whole body is `push "X"` /
//! `queue "Y"` exits 0 with empty stdout and empty stderr (nothing in 4b
//! reads the queue, so there is nothing to print), and a **second, separate**
//! `rexx` process run afterward reports `queued()` as `0`. So on this host,
//! right now, a plain queue push does not survive past the process that made
//! it -- the single-program rule this crate's tests already follow is still
//! correct, but for that measured reason, not for "a live daemon's state
//! permanently diverges from ours". Being host/session-state dependent
//! rather than architectural, a different host or an explicit named queue
//! could behave differently, and the rule should be re-measured rather than
//! assumed if that ever matters (`phase-4-exclusions.txt`'s own KNOWN GAP
//! row for this task has the fuller record).

use std::collections::VecDeque;

/// One `Interp`'s queue: every line `PUSH`/`QUEUE` has written and
/// [`Queue::pop`] has not removed, the head being the next line it will --
/// see the module doc's own "LIFO and FIFO" section for which end that is for
/// each keyword.
/// `Vec<u8>` per line rather than an `ObjRef`, matching `Interp::out`/
/// `Interp::trace`'s own sinks: `PUSH`/`QUEUE` store the already-rendered
/// string form (`evaluateStringExpression`'s `requestString`, mirrored by
/// `run.rs`'s arms calling `Interp::to_text` the same way `SAY`'s own arm
/// does), not a heap value, so there is nothing here for the collector to
/// trace and no `ObjRef` to keep rooted between a write and whatever later
/// `PULL` reads it back.
pub(crate) struct Queue {
    lines: VecDeque<Vec<u8>>,
}

impl Queue {
    pub(crate) fn new() -> Queue {
        Queue {
            lines: VecDeque::new(),
        }
    }

    /// `PUSH line`: inserted at the head, so it is the next line `PULL` will
    /// remove -- LIFO relative to every earlier `PUSH`/`QUEUE`.
    pub(crate) fn push(&mut self, line: Vec<u8>) {
        self.lines.push_front(line);
    }

    /// `QUEUE line`: appended at the tail, behind everything already
    /// queued -- FIFO relative to every earlier `PUSH`/`QUEUE`.
    pub(crate) fn queue(&mut self, line: Vec<u8>) {
        self.lines.push_back(line);
    }

    /// The head line, removed, or `None` when the queue is empty.
    ///
    /// The **front**, which is the whole of the LIFO/FIFO split: see the
    /// module doc for the measured `C`, `A`, `B` this end produces and for
    /// what a `pop_back` here would print instead. `None` rather than a null
    /// string, so that `Interp::pull_line` (`input.rs`) can tell an empty queue
    /// from a queued empty line -- it has to, because only the first sends the
    /// read on to `.input`.
    pub(crate) fn pop(&mut self) -> Option<Vec<u8>> {
        self.lines.pop_front()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plan::{BodyKey, ProgramId};
    use crate::{Activation, Interp};
    use rexx_parse::{Program, parse_program};
    use std::rc::Rc;

    /// The module doc's own probe (`push "a"`, `queue "b"`, `push "c"`),
    /// asserted against the queue type directly rather than through the
    /// instruction loop.
    ///
    /// **What this pins, and what it does not (review round 1, I1/I2/M2
    /// corrected this comment's earlier, wrong claim about the split).**
    /// This test constructs a `Queue` and calls its methods directly, so it
    /// can only ever prove `push_front`/`push_back` are wired to the right
    /// keyword -- collapsing either to the other end, or dropping the
    /// argument entirely, fails it. It says nothing about whether `run.rs`'s
    /// `step` arms actually call these methods (that is
    /// `push_and_queue_actually_write_into_the_running_interpreters_queue`,
    /// below) or whether the expression reaching them was evaluated and
    /// traced correctly (that is `corpus/lang/push_queue.rex`, under
    /// `REXX_CORPUS_GATE`). `tests/loud.rs` proves neither: it runs a
    /// program only for an *out-of-scope* variant, and `Push`/`Queue` moved
    /// in scope this same task.
    #[test]
    fn interleaved_push_and_queue_match_the_oracle_order() {
        let mut queue = Queue::new();
        queue.push(b"a".to_vec());
        queue.queue(b"b".to_vec());
        queue.push(b"c".to_vec());
        assert_eq!(
            queue.lines,
            VecDeque::from([b"c".to_vec(), b"a".to_vec(), b"b".to_vec()])
        );
    }

    /// The adjacent success `PUSH`/`QUEUE` sharing a codepath needs
    /// (CLAUDE.md's "pair a refusal with its adjacent success"): a queue
    /// touched only by `QUEUE`, never `PUSH`, is a plain FIFO with no head
    /// insertion to get right, and this is what would stay green if
    /// `push`'s own `push_front` quietly became a second `push_back` --
    /// the interleaved test above would still fail that mutation (order
    /// would come out `a`, `b`, `c` rather than `c`, `a`, `b`), but pinning
    /// pure FIFO order separately is what shows *this* half of the type is
    /// right for the ordinary reason and not by an accident of the other
    /// half's mistake cancelling out.
    ///
    /// The same reasoning applies to [`Queue::pop`], which is why
    /// `interleaved_push_and_queue_survive_a_round_trip` exists beside it:
    /// this test's own expectation is unchanged by which end `pop` takes from.
    #[test]
    fn queue_alone_is_plain_fifo() {
        let mut queue = Queue::new();
        queue.queue(b"a".to_vec());
        queue.queue(b"b".to_vec());
        queue.queue(b"c".to_vec());
        assert_eq!(
            queue.lines,
            VecDeque::from([b"a".to_vec(), b"b".to_vec(), b"c".to_vec()])
        );
    }

    /// Which end [`Queue::pop`] takes from, which neither test above can see.
    ///
    /// Both of them assert the *stored* order, and `pop_back` does not change
    /// that -- so both stay green under it while every `PULL` in the language
    /// answers in reverse. The module doc's measured `C`, `A`, `B` is what
    /// this compares against, before `PULL`'s own upcasing: the values come
    /// back `c`, `a`, `b`.
    #[test]
    fn interleaved_push_and_queue_survive_a_round_trip() {
        let mut queue = Queue::new();
        queue.push(b"a".to_vec());
        queue.queue(b"b".to_vec());
        queue.push(b"c".to_vec());
        assert_eq!(queue.pop().as_deref(), Some(&b"c"[..]));
        assert_eq!(queue.pop().as_deref(), Some(&b"a"[..]));
        assert_eq!(queue.pop().as_deref(), Some(&b"b"[..]));
        // Emptiness has to be distinguishable from a queued empty line, since
        // only the first sends `PULL` on to `.input`.
        assert_eq!(queue.pop(), None);
        queue.queue(Vec::new());
        assert_eq!(queue.pop().as_deref(), Some(&b""[..]));
        assert_eq!(queue.pop(), None);
    }

    /// Pushes a fresh top-level activation for `program`, the minimal setup
    /// `Interp::run` does. Copied rather than shared, matching every other
    /// test module in this crate -- `run.rs`'s own copy of this same
    /// function has why: `eval.rs`, `stem.rs` and `plan.rs` each keep their
    /// own rather than exporting one for every caller to share.
    fn activate(interp: &mut Interp, program: Program) -> Rc<Program> {
        let program = Rc::new(program);
        let id = ProgramId(interp.programs.len());
        interp.programs.push(Rc::clone(&program));
        let plan = interp.plan_for(
            BodyKey {
                program: id,
                directive: None,
            },
            &program.main,
            &program.symbols,
        );
        let frame = interp.roots.push_slots(plan.len());
        let id = interp.next_activation_id();
        interp
            .activations
            .push(Activation::new(id, Rc::clone(&program), plan, frame));
        program
    }

    /// **I3 (review round 1): the reader `Queue`'s own tests above cannot
    /// be.** Both tests above construct a `Queue` and call its methods
    /// directly, so neither can see whether `step`'s `Push`/`Queue` arms
    /// (`run.rs`) ever call `Queue::push`/`Queue::queue` at all. Measured:
    /// deleting just those two call sites -- keeping the expression's
    /// evaluation and its trace, discarding only the rendered line --
    /// left `cargo test --workspace` at 978 passed / 0 failed and the
    /// STRICT corpus at 39 of 39, because nothing ran a program through the
    /// interpreter and then read `Interp::queue` back afterward. This test
    /// is that reader: it runs the module doc's own 4c-shaped probe (minus
    /// the three `PULL`s 4c has not implemented yet) through
    /// `Interp::run_activation`, the same entry point `Interp::run` uses in
    /// production, and inspects the queue afterward through the same
    /// private field the type-level tests above use.
    #[test]
    fn push_and_queue_actually_write_into_the_running_interpreters_queue() {
        let mut interp = Interp::new();
        let program = parse_program(b"push \"a\"\nqueue \"b\"\npush \"c\"\n".to_vec())
            .expect("test program parses");
        activate(&mut interp, program);
        interp.run_activation().expect("push/queue never fail");
        assert_eq!(
            interp.queue.lines,
            VecDeque::from([b"c".to_vec(), b"a".to_vec(), b"b".to_vec()])
        );
    }
}
