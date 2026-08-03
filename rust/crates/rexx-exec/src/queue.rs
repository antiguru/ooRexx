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
//! **Nothing in 4b reads this queue.** `PULL`, `PARSE PULL` and `QUEUED()`
//! are all 4c's, so this module has no removal method yet: adding one before
//! anything calls it would be speculative API nobody has measured a caller
//! for. The type's own unit tests below read the stored order through its
//! private field instead, the same way `stem.rs`'s tests reach into
//! `Body::Stem` directly rather than through a round trip nothing wires up
//! yet.
//!
//! # LIFO and FIFO, and which end is which
//!
//! `PUSH` inserts at the head; `QUEUE` appends at the tail. Both share the
//! oracle's `RexxInstructionQueue` (`QueueInstruction.cpp`) and differ only
//! in `Activity::QueueOrder` (`QUEUE_LIFO` for `PUSH`, `QUEUE_FIFO` for
//! `QUEUE`) -- `ast.rs`'s own doc comment on `InstructionKind::Push`/`Queue`
//! already says the two keywords share one C++ class. The head is "the next
//! line `PULL` will remove" once 4c wires that up, which is what makes
//! `PUSH` (to the head) the LIFO order and `QUEUE` (to the tail) the FIFO
//! one relative to it.
//!
//! Measured against the oracle with a 4c-shaped probe this crate cannot yet
//! run itself (`PULL` is not implemented), `push "a"` then `queue "b"` then
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

/// One `Interp`'s queue: every line `PUSH`/`QUEUE` has written, oldest
/// `PULL` target at the front. `Vec<u8>` per line rather than an `ObjRef`,
/// matching `Interp::out`/`Interp::trace`'s own sinks: `PUSH`/`QUEUE` store
/// the already-rendered string form (`evaluateStringExpression`'s
/// `requestString`, mirrored by `run.rs`'s arms calling `Interp::to_text`
/// the same way `SAY`'s own arm does), not a heap value, so there is nothing
/// here for the collector to trace and no `ObjRef` to keep rooted between a
/// write and whatever later `PULL` reads it back.
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
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The module doc's own 4c-shaped probe (`push "a"`, `queue "b"`,
    /// `push "c"`), asserted against the queue type directly rather than
    /// through the instruction loop -- `step`'s own `Push`/`Queue` arms
    /// (`run.rs`) are exercised separately by the loud-witness removal in
    /// `tests/loud.rs`, but nothing in 4b can observe *order* through the
    /// executor, since `PULL` is 4c's. This is the whole of Task 8's
    /// coverage for the property that would make a degenerate `Push | Queue
    /// => Ok(Flow::Next)` wrong: deleting either `push_front` or
    /// `push_back` above (collapsing both to the same end) or dropping the
    /// argument entirely (an empty queue) both fail this assertion.
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
}
