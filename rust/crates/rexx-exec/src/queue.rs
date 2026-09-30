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

//! The in-process external data queue (I15) that `PUSH` and `QUEUE` write to,
//! through `Interp::queue_evaluated` (`run.rs`), which both engines enter.
//! One per `Interp`, held as a plain field rather than anything IPC-backed --
//! see "Why not cross-process" below.
//! ```text
//! C
//! A
//! B
//! ```

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
    pub(crate) fn pop(&mut self) -> Option<Vec<u8>> {
        self.lines.pop_front()
    }

    /// How many lines are waiting: what `QUEUED()` answers.
    pub(crate) fn len(&self) -> usize {
        self.lines.len()
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
        let program_id = ProgramId(interp.programs.len());
        interp.programs.push(Rc::clone(&program));
        let plan = interp.plan_for(
            BodyKey {
                program: program_id,
                directive: None,
            },
            &program.main,
            &program.symbols,
            &program.source,
        );
        let frame = interp.roots.push_slots(plan.len());
        let id = interp.next_activation_id();
        interp.push_activation(Activation::new(
            id,
            Rc::clone(&program),
            program_id,
            plan,
            frame,
        ));
        program
    }

    /// **I3 (review round 1): the reader `Queue`'s own tests above cannot
    /// be.** Both tests above construct a `Queue` and call its methods
    /// directly, so neither can see whether the running interpreter ever
    /// calls `Queue::push`/`Queue::queue` at all. Those calls are
    /// `Interp::queue_evaluated`'s (`run.rs`), which `step`'s own
    /// `Push`/`Queue` arm and `crate::ir::Op::Queue` both enter, so a
    /// deletion of them -- keeping the expression's evaluation and its trace,
    /// discarding only the rendered line -- takes the write away from both
    /// engines at once. Measured at review round 1, when the write sat in
    /// `step`'s arm alone: that deletion left `cargo test --workspace` at 978
    /// passed / 0 failed and the STRICT corpus at 39 of 39, because nothing
    /// ran a program through the interpreter and then read `Interp::queue`
    /// back afterward. This test is that reader: it runs the module doc's own
    /// probe -- the three `PUSH`/`QUEUE` clauses, without the `PULL`s --
    /// through `Interp::run_activation`, the same entry point `Interp::run`
    /// uses in production, and inspects the queue afterward through the same
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
