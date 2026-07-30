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

    pub fn push_frame(&mut self) -> FrameId {
        FrameId(self.temps.len())
    }

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
