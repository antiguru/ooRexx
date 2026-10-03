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

//! The pin depth, and the pinned-park counter: at each would-be park point,
//! the pinned re-entry frames (spec 2026-09-29 section 2.1) between the driver
//! and that point. Every build counts the depth on the running activity; the
//! `pinning` feature adds the kind of each frame.

/// `$body`, run with `$kind` pushed as a pinned frame.
#[cfg(feature = "pinning")]
macro_rules! pinned {
    ($interp:expr, $kind:expr, $body:expr) => {{
        $interp.activity.pin_depth += 1;
        $interp.activity.pins.enter($kind);
        let answer = $body;
        $interp.activity.pins.leave();
        $interp.activity.pin_depth -= 1;
        answer
    }};
}

#[cfg(not(feature = "pinning"))]
macro_rules! pinned {
    ($interp:expr, $kind:expr, $body:expr) => {{
        $interp.activity.pin_depth += 1;
        let answer = $body;
        $interp.activity.pin_depth -= 1;
        answer
    }};
}

/// Pushes `$kind` as a pinned frame; [`pin_leave!`] pops it.
#[cfg(feature = "pinning")]
macro_rules! pin_enter {
    ($interp:expr, $kind:expr) => {{
        $interp.activity.pin_depth += 1;
        $interp.activity.pins.enter($kind)
    }};
}

#[cfg(not(feature = "pinning"))]
macro_rules! pin_enter {
    ($interp:expr, $kind:expr) => {
        $interp.activity.pin_depth += 1
    };
}

#[cfg(feature = "pinning")]
macro_rules! pin_leave {
    ($interp:expr) => {{
        $interp.activity.pins.leave();
        $interp.activity.pin_depth -= 1;
    }};
}

#[cfg(not(feature = "pinning"))]
macro_rules! pin_leave {
    ($interp:expr) => {
        $interp.activity.pin_depth -= 1
    };
}

/// Records an arrival at the park point `$kind`, an `Option` or a kind.
#[cfg(feature = "pinning")]
macro_rules! park_point {
    ($interp:expr, $kind:expr) => {
        $interp.pinning.park(&$interp.activity.pins, $kind)
    };
}

#[cfg(not(feature = "pinning"))]
macro_rules! park_point {
    ($interp:expr, $kind:expr) => {};
}

/// Records an inverted wait for the park reason `$reason`.
#[cfg(feature = "pinning")]
macro_rules! inverted_wait {
    ($interp:expr, $reason:expr) => {
        $interp.pinning.inverted(&$interp.activity.pins, $reason)
    };
}

#[cfg(not(feature = "pinning"))]
macro_rules! inverted_wait {
    ($interp:expr, $reason:expr) => {};
}

/// Records a pinned wait for the park reason `$reason`, which [`pinned_unpark!`]
/// ends.
#[cfg(feature = "pinning")]
macro_rules! pinned_park {
    ($interp:expr, $reason:expr) => {
        $interp.pinning.pinned_park(&$interp.activity.pins, $reason)
    };
}

#[cfg(not(feature = "pinning"))]
macro_rules! pinned_park {
    ($interp:expr, $reason:expr) => {};
}

#[cfg(feature = "pinning")]
macro_rules! pinned_unpark {
    ($interp:expr) => {
        $interp.activity.pins.unpark()
    };
}

#[cfg(not(feature = "pinning"))]
macro_rules! pinned_unpark {
    ($interp:expr) => {};
}

/// Records that a loop above the pinned wait of the idle activity `$idle`
/// found it ready and set it aside: a late wake.
#[cfg(feature = "pinning")]
macro_rules! late_wake {
    ($interp:expr, $idle:expr) => {
        $interp.pinning.late_wake(&$idle.pins)
    };
}

/// Records a `SLICE` deferred because the activity is pinned.
#[cfg(feature = "pinning")]
macro_rules! deferred_slice {
    ($interp:expr) => {
        $interp.pinning.deferred_slice(&$interp.activity.pins)
    };
}

#[cfg(not(feature = "pinning"))]
macro_rules! deferred_slice {
    ($interp:expr) => {};
}

/// Records a pinned yield: a slice that found the activity still pinned.
#[cfg(feature = "pinning")]
macro_rules! pinned_yield {
    ($interp:expr) => {
        $interp.pinning.pinned_yield(&$interp.activity.pins)
    };
}

#[cfg(not(feature = "pinning"))]
macro_rules! pinned_yield {
    ($interp:expr) => {};
}

/// Records an inverted yield: a pinned yield whose round set aside a
/// buried activity that was ready.
#[cfg(feature = "pinning")]
macro_rules! inverted_yield {
    ($interp:expr) => {
        $interp.pinning.inverted_yield(&$interp.activity.pins)
    };
}

#[cfg(not(feature = "pinning"))]
macro_rules! inverted_yield {
    ($interp:expr) => {};
}

/// Records a `REPLY` refused because Rust frames lie inside its method body.
#[cfg(feature = "pinning")]
macro_rules! immovable_reply {
    ($interp:expr) => {
        $interp.pinning.immovable_reply(&$interp.activity.pins)
    };
}

#[cfg(not(feature = "pinning"))]
macro_rules! immovable_reply {
    ($interp:expr) => {};
}

#[cfg(feature = "pinning")]
pub use counter::{ParkKind, PinKind, PinReport};
#[cfg(feature = "pinning")]
pub(crate) use counter::{PinStack, Pinning};

#[cfg(feature = "pinning")]
mod counter {
    use std::cell::RefCell;
    use std::collections::BTreeMap;

    /// A Rust frame that keeps an activity pinned.
    #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub enum PinKind {
        SortComparator,
        Conversion,
        Unknown,
        Forward,
        Delegate,
        Operator,
        TrapHandler,
        NativeApiCallback,
        LibraryEntry,
        StreamWrapper,
        PullWrapper,
        OutputWrapper,
        TraceWrapper,
        RedirectWrapper,
        SecurityManager,
        LoopHeader,
        NestedLoop,
        Interpret,
        TreeEval,
        TreeSend,
        OpExec,
        Program,
        /// An `UNINIT` method run by the collector's sweep or at termination.
        Uninit,
        /// A `messageComplete` sent to an object a `Message~notify` named.
        Notification,
        /// A native method that runs Rexx, by message name.
        Native(Box<str>),
    }

    impl PinKind {
        /// The frame a native method `name` pushes.
        pub(crate) fn native(name: &[u8]) -> PinKind {
            PinKind::Native(String::from_utf8_lossy(name).into())
        }
    }

    /// A would-be park point.
    #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub enum ParkKind {
        GuardOn,
        GuardWhen,
        Reply,
        MessageResult,
        MessageWait,
        SemaphoreWait,
        SysSemWait,
        SysSleep,
        Timer,
    }

    impl ParkKind {
        /// The park point a wait for `reason` reached.
        fn of(reason: crate::scheduler::ParkReason) -> ParkKind {
            match reason {
                crate::scheduler::ParkReason::Guard(_) => ParkKind::GuardOn,
                crate::scheduler::ParkReason::GuardWhen => ParkKind::GuardWhen,
                crate::scheduler::ParkReason::MessageResult(_) => ParkKind::MessageResult,
                crate::scheduler::ParkReason::MessageWait(_) => ParkKind::MessageWait,
                crate::scheduler::ParkReason::Sleep { .. } => ParkKind::SysSleep,
                crate::scheduler::ParkReason::Timer { .. } => ParkKind::Timer,
            }
        }

        /// The park point a send of `name` to a method of class `scope`
        /// with no body reaches.
        pub(crate) fn unimplemented_method(scope: &str, name: &[u8]) -> Option<ParkKind> {
            match (scope, name) {
                ("EventSemaphore", b"WAIT") | ("MutexSemaphore", b"ACQUIRE") => {
                    Some(ParkKind::SemaphoreWait)
                }
                _ => None,
            }
        }

        /// The park point the external routine `name` reaches.
        pub(crate) fn routine(name: &[u8]) -> Option<ParkKind> {
            [&b"SysWaitEventSem"[..], b"SysRequestMutexSem"]
                .iter()
                .any(|wait| wait.eq_ignore_ascii_case(name))
                .then_some(ParkKind::SysSemWait)
        }
    }

    /// Arrivals keyed by park kind and the pinned frames at the arrival,
    /// outermost first with consecutive repeats collapsed.
    #[derive(Clone, Debug, Default)]
    pub struct PinReport {
        pub parks: BTreeMap<(ParkKind, Vec<PinKind>), u64>,
        /// Pinned waits, keyed as `parks` is.
        pub pinned_parks: BTreeMap<(ParkKind, Vec<PinKind>), u64>,
        /// Pinned waits whose activity was ready while a loop above its own
        /// ran, which set it aside until that loop returned, keyed as `parks`
        /// is.
        pub late_wakes: BTreeMap<(ParkKind, Vec<PinKind>), u64>,
        /// Inverted pinned waits refused, keyed as `parks` is.
        pub inverted: BTreeMap<(ParkKind, Vec<PinKind>), u64>,
        /// Slices deferred to the next unpinned clause boundary, keyed by the
        /// pinned frames where each was first seen.
        pub deferred_slices: BTreeMap<Vec<PinKind>, u64>,
        /// Pinned yields, keyed by the pinned frames they ran under.
        pub pinned_yields: BTreeMap<Vec<PinKind>, u64>,
        /// Pinned yields whose round set aside a buried activity that was
        /// ready, keyed as `pinned_yields` is.
        pub inverted_yields: BTreeMap<Vec<PinKind>, u64>,
        /// `REPLY`s refused as immovable, keyed by the pinned frames at each.
        pub immovable_replies: BTreeMap<Vec<PinKind>, u64>,
        /// Frames still pushed when the report was taken; zero unless a push
        /// has no matching pop.
        pub unbalanced: usize,
    }

    /// The per-interpreter counter.
    #[derive(Default)]
    pub(crate) struct Pinning {
        report: RefCell<PinReport>,
    }

    /// One activity's pinned frames, innermost last, and its pinned waits.
    #[derive(Default)]
    pub(crate) struct PinStack {
        stack: RefCell<Vec<Option<PinKind>>>,
        waits: RefCell<Vec<ParkKind>>,
    }

    impl PinStack {
        pub(crate) fn unpark(&self) {
            self.waits.borrow_mut().pop();
        }

        pub(crate) fn enter(&self, kind: impl Into<Option<PinKind>>) {
            self.stack.borrow_mut().push(kind.into());
        }

        pub(crate) fn leave(&self) {
            self.stack.borrow_mut().pop();
        }
    }

    /// The pinned frames on `pins`, outermost first with consecutive repeats
    /// collapsed.
    fn frames_of(pins: &PinStack) -> Vec<PinKind> {
        let mut frames: Vec<PinKind> = pins.stack.borrow().iter().flatten().cloned().collect();
        frames.dedup();
        frames
    }

    impl Pinning {
        pub(crate) fn park(&self, pins: &PinStack, kind: impl Into<Option<ParkKind>>) {
            let Some(kind) = kind.into() else {
                return;
            };
            let mut frames: Vec<PinKind> = pins.stack.borrow().iter().flatten().cloned().collect();
            frames.dedup();
            *self
                .report
                .borrow_mut()
                .parks
                .entry((kind, frames))
                .or_default() += 1;
        }

        pub(crate) fn late_wake(&self, pins: &PinStack) {
            let Some(kind) = pins.waits.borrow().last().copied() else {
                return;
            };
            *self
                .report
                .borrow_mut()
                .late_wakes
                .entry((kind, frames_of(pins)))
                .or_default() += 1;
        }

        pub(crate) fn pinned_park(&self, pins: &PinStack, reason: crate::scheduler::ParkReason) {
            pins.waits.borrow_mut().push(ParkKind::of(reason));
            *self
                .report
                .borrow_mut()
                .pinned_parks
                .entry((ParkKind::of(reason), frames_of(pins)))
                .or_default() += 1;
        }

        pub(crate) fn inverted(&self, pins: &PinStack, reason: crate::scheduler::ParkReason) {
            *self
                .report
                .borrow_mut()
                .inverted
                .entry((ParkKind::of(reason), frames_of(pins)))
                .or_default() += 1;
        }

        pub(crate) fn deferred_slice(&self, pins: &PinStack) {
            *self
                .report
                .borrow_mut()
                .deferred_slices
                .entry(frames_of(pins))
                .or_default() += 1;
        }

        pub(crate) fn pinned_yield(&self, pins: &PinStack) {
            *self
                .report
                .borrow_mut()
                .pinned_yields
                .entry(frames_of(pins))
                .or_default() += 1;
        }

        pub(crate) fn inverted_yield(&self, pins: &PinStack) {
            *self
                .report
                .borrow_mut()
                .inverted_yields
                .entry(frames_of(pins))
                .or_default() += 1;
        }

        pub(crate) fn immovable_reply(&self, pins: &PinStack) {
            *self
                .report
                .borrow_mut()
                .immovable_replies
                .entry(frames_of(pins))
                .or_default() += 1;
        }

        /// Forgets the arrivals so far, keeping the frames pushed.
        pub(crate) fn reset(&self) {
            let mut report = self.report.borrow_mut();
            report.parks.clear();
            report.pinned_parks.clear();
            report.late_wakes.clear();
            report.inverted.clear();
            report.deferred_slices.clear();
            report.pinned_yields.clear();
            report.inverted_yields.clear();
            report.immovable_replies.clear();
        }

        pub(crate) fn take(&self, pins: &PinStack) -> PinReport {
            let mut report = std::mem::take(&mut *self.report.borrow_mut());
            report.unbalanced = pins.stack.borrow().len();
            report
        }
    }
}
