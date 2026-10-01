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
        LoopHeader,
        NestedLoop,
        Interpret,
        TreeEval,
        TreeSend,
        OpExec,
        Program,
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

        /// The park point the `LIBRARY REXX` entry point `entry` reaches.
        pub(crate) fn entry_point(entry: &str) -> Option<ParkKind> {
            matches!(entry, "alarm_startTimer" | "ticker_waitTimer").then_some(ParkKind::Timer)
        }
    }

    /// Arrivals keyed by park kind and the pinned frames at the arrival,
    /// outermost first with consecutive repeats collapsed.
    #[derive(Clone, Debug, Default)]
    pub struct PinReport {
        pub parks: BTreeMap<(ParkKind, Vec<PinKind>), u64>,
        /// Inverted pinned waits refused, keyed as `parks` is.
        pub inverted: BTreeMap<(ParkKind, Vec<PinKind>), u64>,
        /// Frames still pushed when the report was taken; zero unless a push
        /// has no matching pop.
        pub unbalanced: usize,
    }

    /// The per-interpreter counter.
    #[derive(Default)]
    pub(crate) struct Pinning {
        report: RefCell<PinReport>,
    }

    /// One activity's pinned frames, innermost last.
    #[derive(Default)]
    pub(crate) struct PinStack {
        stack: RefCell<Vec<Option<PinKind>>>,
    }

    impl PinStack {
        pub(crate) fn enter(&self, kind: impl Into<Option<PinKind>>) {
            self.stack.borrow_mut().push(kind.into());
        }

        pub(crate) fn leave(&self) {
            self.stack.borrow_mut().pop();
        }
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

        pub(crate) fn inverted(&self, pins: &PinStack, reason: crate::scheduler::ParkReason) {
            let kind = match reason {
                crate::scheduler::ParkReason::Guard => ParkKind::GuardOn,
                crate::scheduler::ParkReason::MessageResult(_) => ParkKind::MessageResult,
            };
            let mut frames: Vec<PinKind> = pins.stack.borrow().iter().flatten().cloned().collect();
            frames.dedup();
            *self
                .report
                .borrow_mut()
                .inverted
                .entry((kind, frames))
                .or_default() += 1;
        }

        /// Forgets the arrivals so far, keeping the frames pushed.
        pub(crate) fn reset(&self) {
            let mut report = self.report.borrow_mut();
            report.parks.clear();
            report.inverted.clear();
        }

        pub(crate) fn take(&self, pins: &PinStack) -> PinReport {
            let mut report = std::mem::take(&mut *self.report.borrow_mut());
            report.unbalanced = pins.stack.borrow().len();
            report
        }
    }
}
