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

use crate::ObjRef;

/// A position in the temporary stack that a frame will unwind to.
#[derive(Copy, Clone, Debug)]
pub struct FrameId(usize);

/// A handle to one activation's range of local-variable slots inside
/// `RootSet` (D16). `push_slots`/`pop_slots` bracket its lifetime. `slot`,
/// `set_slot` and `grow_slots` address within it. `depth` is the frame
/// stack's length at the moment this frame was pushed, and is how
/// `grow_slots` recognises "the top frame" even when two frames happen to
/// start at the same offset (both pushed with `initial_len` 0).
#[derive(Copy, Clone, Debug)]
pub struct SlotFrame {
    start: usize,
    depth: usize,
}

/// Everything the collector starts from.
///
/// The C++ implementation needs `ProtectedObject` at every allocation-crossing
/// site because raw pointers in C++ locals are invisible to it. Here the set
/// is small and explicit: globals, plus a stack of temporaries that expression
/// evaluation pushes into rather than holding values in Rust locals across an
/// allocation, plus the slot frames below.
pub struct RootSet {
    globals: Vec<(String, ObjRef)>,
    temps: Vec<ObjRef>,
    /// Local-variable slots for every currently active activation, flattened
    /// into one vector: each `SlotFrame` owns a contiguous range starting at
    /// its `start`. `None` is an unassigned (or `DROP`ped) variable, not a
    /// missing one -- unlike `temps`, whose entries are always live values,
    /// a slot must be able to say "no value" without that colliding with
    /// `ObjRef::NIL`, which is itself a legal Rexx value (`x = .nil`).
    slots: Vec<Option<ObjRef>>,
    /// The starting offset of every currently pushed frame, in push order.
    /// Its length is also every live frame's `depth` plus one, which is how
    /// `grow_slots` and `pop_slots` recognise the top frame.
    frame_starts: Vec<usize>,
}

impl RootSet {
    pub fn new() -> Self {
        RootSet {
            globals: Vec::new(),
            temps: Vec::new(),
            slots: Vec::new(),
            frame_starts: Vec::new(),
        }
    }

    pub fn add_global(&mut self, name: &str, value: ObjRef) {
        match self.globals.iter_mut().find(|(n, _)| n == name) {
            Some(entry) => entry.1 = value,
            None => self.globals.push((name.to_string(), value)),
        }
    }

    /// Marks the current top of the temporaries stack, to be handed back to
    /// `pop_frame`. Mutates nothing, so an unpopped `FrameId` costs nothing on
    /// its own.
    pub fn push_frame(&mut self) -> FrameId {
        FrameId(self.temps.len())
    }

    /// Discards every temporary pushed since `frame` was taken.
    ///
    /// **Truncating to a watermark, rather than popping one frame, is
    /// load-bearing and callers rely on it.** `rexx-exec` has six functions
    /// that open a frame and then use `?`, so a raised condition leaves their
    /// own `pop_frame` unreached; every one of those is healed here, because
    /// the enclosing `step_in_temps_frame` pops unconditionally with an outer
    /// watermark and this call unwinds the skipped inner frames with it. Pops
    /// are therefore idempotent and no corrupt state is representable. A stale
    /// `FrameId` has exactly two shapes and neither is unsound: one taken
    /// deeper than the current top truncates to a larger index, which is a
    /// silent no-op, and one taken shallower discards more than its owner
    /// meant to. Losing a root early is the direction that could bite, and it
    /// cannot happen from a handle this type issued, since every handle is a
    /// length this stack once had.
    ///
    /// **Do not add a balance assertion here** without first making those six
    /// sites pop on their own path. It would fire on the ordinary error path
    /// of a correct program. That is also why the slot side of this file
    /// asserts and this side deliberately does not; the asymmetry is a
    /// decision, not an oversight.
    pub fn pop_frame(&mut self, frame: FrameId) {
        self.temps.truncate(frame.0);
    }

    pub fn push_temp(&mut self, value: ObjRef) {
        self.temps.push(value);
    }

    /// Opens a new slot frame of `initial_len` unassigned slots, for an
    /// activation entering with a plan of that many resolved names (D16).
    pub fn push_slots(&mut self, initial_len: usize) -> SlotFrame {
        let start = self.slots.len();
        self.slots.resize(start + initial_len, None);
        let depth = self.frame_starts.len();
        self.frame_starts.push(start);
        SlotFrame { start, depth }
    }

    /// Closes `frame`, releasing its slots. Frames nest like any stack, so
    /// this must be the top one -- the same invariant `grow_slots` checks,
    /// stated there.
    pub fn pop_slots(&mut self, frame: SlotFrame) {
        assert_eq!(
            self.frame_starts.len(),
            frame.depth + 1,
            "pop_slots on a frame that is not the top one"
        );
        self.frame_starts.pop();
        self.slots.truncate(frame.start);
    }

    /// Reads slot `index` within `frame`: `None` for an unassigned or
    /// `DROP`ped variable, which is a legal outcome and not an error.
    pub fn slot(&self, frame: SlotFrame, index: usize) -> Option<ObjRef> {
        self.slots[frame.start + index]
    }

    pub fn set_slot(&mut self, frame: SlotFrame, index: usize, value: ObjRef) {
        self.slots[frame.start + index] = Some(value);
    }

    /// Returns slot `index` within `frame` to the unset state, which is what
    /// `DROP` on a simple variable does.
    ///
    /// **A separate operation rather than `set_slot` taking an
    /// `Option<ObjRef>`**, and the choice is about call sites rather than
    /// about this file. `DROP` is a construct the language has, so a caller
    /// that spells it `clear_slot(frame, i)` says what it means, while
    /// `set_slot(frame, i, None)` reads at a glance like a caller that forgot
    /// to compute a value. The read side already carries the `Option` shape,
    /// since `slot` returns one, so nothing is hidden by keeping the common
    /// write monomorphic. It also leaves every existing `set_slot` call
    /// untouched, where the alternative was a mechanical `Some(...)` wrap
    /// across the crate for no gain.
    ///
    /// The other half of the operation is that a cleared slot **stops being a
    /// root**, which `iter` gets right because it filters on the `Option` and
    /// this writes `None` rather than some in-band marker. That is the half
    /// worth stating: a clearing operation that only changed what `slot`
    /// answers, while `iter` went on yielding the old value, would keep an
    /// unreachable object alive with nothing failing until a collection
    /// happened to land later, somewhere unrelated.
    ///
    /// `ObjRef::NIL` cannot stand in for the unset state, which is the reason
    /// this exists at all. Measured on `build/bin/rexx`:
    ///
    /// ```text
    /// a = 5     ; drop a ; say a   ->  A                 (unset: derived name)
    /// x = .nil            ; say x  ->  The NIL object    (`.nil` is a value)
    /// y = .nil  ; drop y  ; say y  ->  Y                 (unset, not NIL)
    /// ```
    ///
    /// The third line settles it: a variable holding `.nil` and a dropped one
    /// render differently, so a bare `ObjRef` slot has no spare value left to
    /// mean "no value". D16 rejected storing slots in `temps` on the same
    /// argument.
    ///
    /// Clearing a slot that is already unset is a no-op rather than an error,
    /// because `DROP` on a never-assigned variable is legal Rexx and does
    /// nothing.
    pub fn clear_slot(&mut self, frame: SlotFrame, index: usize) {
        self.slots[frame.start + index] = None;
    }

    /// Grows `frame` by one slot for a name its plan never saw -- `DROP (v)`
    /// naming its target at run time, measured: `v = 'X'; x = 1; drop (v);
    /// say x` prints `X`, so a name resolving to no existing slot must be
    /// able to allocate one. Returns the new slot's index within `frame`.
    ///
    /// Only the top frame may grow. **This is a 4a invariant, not a general
    /// one, and 4b must revisit it before it has more than one live frame.**
    /// It holds in 4a because 4a has exactly one frame, and for `INTERPRET`,
    /// which runs inside the activation that created it rather than pushing
    /// its own. It is already false once a call can be in progress: measured,
    /// `sub: procedure expose zzz` with `zzz = 5` set in the callee makes the
    /// *caller* print 9 after `return`, so a callee write lands in the
    /// caller's pool while the callee's frame sits on top of it. 4b either
    /// grows a non-top frame or binds an exposed name to a slot in the
    /// caller's frame at call time. Deciding which is 4b's. A panic here is
    /// the right shape until that decision is made: a silent wrong answer
    /// would be a variable landing in another routine's pool, discovered by
    /// chasing a wrong result instead of a message that already says why.
    pub fn grow_slots(&mut self, frame: SlotFrame) -> usize {
        assert_eq!(
            self.frame_starts.len(),
            frame.depth + 1,
            "grow_slots on a frame that is not the top one (a 4a invariant, \
             see grow_slots's doc comment: 4b must bind exposed names \
             differently rather than relax this)"
        );
        let index = self.slots.len() - frame.start;
        self.slots.push(None);
        index
    }

    /// Yields globals, temps, and every assigned slot across every currently
    /// active frame -- a popped frame's slots are already gone, truncated
    /// out of `slots` by `pop_slots`, so nothing here needs to filter them
    /// out again by frame.
    pub fn iter(&self) -> impl Iterator<Item = ObjRef> + '_ {
        self.globals
            .iter()
            .map(|(_, v)| *v)
            .chain(self.temps.iter().copied())
            .chain(self.slots.iter().filter_map(|s| *s))
    }
}

impl Default for RootSet {
    fn default() -> Self {
        Self::new()
    }
}
