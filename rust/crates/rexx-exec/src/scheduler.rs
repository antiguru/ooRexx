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

//! The scheduler seam (spec 2026-09-29 section 2.1), and the one
//! implementation there is: a single activity.

use crate::run::Flow;

/// What an instruction run by [`crate::ir::Op::Exec`] answers the driver.
pub(crate) enum ExecOutcome {
    /// It ran, and this is its `Flow`.
    Done(Flow),
    /// The instruction would park the activity, which the driver refuses
    /// loudly.
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "constructed only by the test-only arm")
    )]
    Park(ParkReason),
    /// Its continuation goes to a new activity, and the current one goes on
    /// with this `Flow`.
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "constructed only by the test-only arm")
    )]
    Split(Flow),
}

/// Why an activity parks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ParkReason {
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "constructed only by the test-only arm")
    )]
    Guard,
}

/// What the driver asks of whatever runs the interpreter's activities.
pub(crate) trait Scheduler {
    /// Makes a new activity for a continuation; `false` where none is made
    /// and the continuation stays with the current activity.
    fn spawn(&mut self) -> bool;
}

/// One activity and no other.
#[derive(Default)]
pub(crate) struct SingleActivity;

impl Scheduler for SingleActivity {
    /// No second activity exists to take a continuation.
    fn spawn(&mut self) -> bool {
        false
    }
}

/// An outcome [`crate::Interp::exec_instruction`] answers in place of running
/// its instruction.
#[cfg(test)]
#[derive(Clone, Copy, Debug)]
pub(crate) enum Scripted {
    /// `Park`, without running the instruction.
    Park,
    /// `Split`, with the instruction's own `Flow`.
    Split,
}

#[cfg(test)]
thread_local! {
    static SCRIPT: std::cell::RefCell<std::collections::VecDeque<Scripted>> =
        const { std::cell::RefCell::new(std::collections::VecDeque::new()) };
}

/// Queues `outcomes` for the next `Op::Exec` instructions this thread runs.
#[cfg(test)]
pub(crate) fn script(outcomes: &[Scripted]) {
    SCRIPT.with(|script| script.borrow_mut().extend(outcomes.iter().copied()));
}

/// The next queued outcome, outside the library bootstrap.
#[cfg(test)]
pub(crate) fn take_scripted() -> Option<Scripted> {
    if !crate::ir::drive::counting() {
        return None;
    }
    SCRIPT.with(|script| script.borrow_mut().pop_front())
}
