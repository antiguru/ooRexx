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

//! The pinned-park counter: at each would-be park point, the pinned re-entry
//! frames (spec 2026-09-29 section 2.1) between the driver and that point.
//! Without the `pinning` feature its macros expand to their body or to nothing.

/// `$body`, run with `$kind` pushed as a pinned frame.
#[cfg(feature = "pinning")]
macro_rules! pinned {
    ($interp:expr, $kind:expr, $body:expr) => {{
        $interp.pinning.enter($kind);
        let answer = $body;
        $interp.pinning.leave();
        answer
    }};
}

#[cfg(not(feature = "pinning"))]
macro_rules! pinned {
    ($interp:expr, $kind:expr, $body:expr) => {
        $body
    };
}

/// Pushes `$kind` as a pinned frame; [`pin_leave!`] pops it.
#[cfg(feature = "pinning")]
macro_rules! pin_enter {
    ($interp:expr, $kind:expr) => {
        $interp.pinning.enter($kind)
    };
}

#[cfg(not(feature = "pinning"))]
macro_rules! pin_enter {
    ($interp:expr, $kind:expr) => {};
}

#[cfg(feature = "pinning")]
macro_rules! pin_leave {
    ($interp:expr) => {
        $interp.pinning.leave()
    };
}

#[cfg(not(feature = "pinning"))]
macro_rules! pin_leave {
    ($interp:expr) => {};
}

/// Records an arrival at the park point `$kind`, an `Option` or a kind.
#[cfg(feature = "pinning")]
macro_rules! park_point {
    ($interp:expr, $kind:expr) => {
        $interp.pinning.park($kind)
    };
}

#[cfg(not(feature = "pinning"))]
macro_rules! park_point {
    ($interp:expr, $kind:expr) => {};
}

#[cfg(feature = "pinning")]
pub(crate) use counter::Pinning;
#[cfg(feature = "pinning")]
pub use counter::{ParkKind, PinKind, PinReport};

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
        /// A native method that runs Rexx, by message name.
        Native(Box<str>),
    }

    impl PinKind {
        /// The frame a native method `name` pushes; `None` for the natives
        /// the spec makes resumable entries.
        pub(crate) fn native(name: &[u8]) -> Option<PinKind> {
            const RESUMABLE: &[&[u8]] = &[
                b"SEND",
                b"SENDWITH",
                b"START",
                b"STARTWITH",
                b"NEW",
                b"CALL",
                b"CALLWITH",
            ];
            (!RESUMABLE.contains(&name))
                .then(|| PinKind::Native(String::from_utf8_lossy(name).into()))
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
                ("Message", b"WAIT") => Some(ParkKind::MessageWait),
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
        /// Frames still pushed when the report was taken; zero unless a push
        /// has no matching pop.
        pub unbalanced: usize,
    }

    /// The per-interpreter counter.
    #[derive(Default)]
    pub(crate) struct Pinning {
        stack: RefCell<Vec<Option<PinKind>>>,
        report: RefCell<PinReport>,
    }

    impl Pinning {
        pub(crate) fn enter(&self, kind: impl Into<Option<PinKind>>) {
            self.stack.borrow_mut().push(kind.into());
        }

        pub(crate) fn leave(&self) {
            self.stack.borrow_mut().pop();
        }

        pub(crate) fn park(&self, kind: impl Into<Option<ParkKind>>) {
            let Some(kind) = kind.into() else {
                return;
            };
            let mut frames: Vec<PinKind> = self.stack.borrow().iter().flatten().cloned().collect();
            frames.dedup();
            *self
                .report
                .borrow_mut()
                .parks
                .entry((kind, frames))
                .or_default() += 1;
        }

        /// Forgets the arrivals so far, keeping the frames pushed.
        pub(crate) fn reset(&self) {
            self.report.borrow_mut().parks.clear();
        }

        pub(crate) fn take(&self) -> PinReport {
            let mut report = std::mem::take(&mut *self.report.borrow_mut());
            report.unbalanced = self.stack.borrow().len();
            report
        }
    }
}
