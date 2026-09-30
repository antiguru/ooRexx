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
    /// The activity parks at this instruction, which runs again on wake.
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

/// An activity, named to the scheduler.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ActivityId(u32);

impl ActivityId {
    /// The activity an interpreter starts with.
    pub(crate) const FIRST: ActivityId = ActivityId(0);
}

/// What the driver asks of whatever runs the interpreter's activities.
#[expect(
    dead_code,
    reason = "the seam methods the single-activity driver does not call"
)]
pub(crate) trait Scheduler {
    /// A new activity for a continuation, or `None` where none is made and
    /// the continuation stays with the current activity.
    fn spawn(&mut self) -> Option<ActivityId>;
    /// Runs ready activities until one is to continue on this driver, and
    /// answers it.
    fn run_until_park(&mut self) -> ActivityId;
    /// The current activity parks for `reason`.
    fn park(&mut self, reason: ParkReason);
    /// `activity` is ready again.
    fn unpark(&mut self, activity: ActivityId);
    /// The current activity's slice has ended.
    fn yield_at_slice(&mut self);
    /// The driver exits for a native call, releasing the baton.
    fn exit_for_native(&mut self);
    /// The driver exits for a blocking operation, releasing the baton.
    fn exit_for_block(&mut self);
    /// An off-baton operation of `activity` has finished.
    fn post_completion(&mut self, activity: ActivityId);
    /// A callback's thread asks for the baton.
    fn request_baton(&mut self);
    /// Runs `world` with every other activity stopped.
    fn stop_the_world<T>(&mut self, world: impl FnOnce() -> T) -> T;
}

/// One activity and no other: a parked activity is the one that continues,
/// and a continuation stays where it is.
#[derive(Default)]
pub(crate) struct SingleActivity;

impl Scheduler for SingleActivity {
    fn spawn(&mut self) -> Option<ActivityId> {
        None
    }

    fn run_until_park(&mut self) -> ActivityId {
        ActivityId::FIRST
    }

    fn park(&mut self, _reason: ParkReason) {}

    fn unpark(&mut self, _activity: ActivityId) {}

    fn yield_at_slice(&mut self) {}

    fn exit_for_native(&mut self) {}

    fn exit_for_block(&mut self) {}

    fn post_completion(&mut self, _activity: ActivityId) {}

    fn request_baton(&mut self) {}

    fn stop_the_world<T>(&mut self, world: impl FnOnce() -> T) -> T {
        world()
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
