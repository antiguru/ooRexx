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
/// `RootSet` (D16). `push_slots`/`pop_slots` bracket its lifetime.
/// `frame_slot`, `set_frame_slot` and `grow_slots` address within it.
/// `depth` is the frame stack's length at the moment this frame was pushed,
/// and is how `grow_slots` recognises "the top frame" even when two frames
/// happen to start at the same offset (both pushed with `initial_len` 0).
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct SlotFrame {
    start: usize,
    depth: usize,
}

/// One variable's storage, with any alias already followed: what
/// `PROCEDURE EXPOSE` and `USE ARG >name` bind a callee's slot *to*, and
/// what a `>name` reference names.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct SlotRef(usize);

/// Set on the tagged `usize` a [`SlotRef`] and an `aliases` entry carry when
/// it names a cell rather than a position in `slots`.
const CELL_TAG: usize = 1 << (usize::BITS - 1);

/// One frame's alias entries, saved across a park by
/// [`RootSet::take_frame_aliases`] and put back by
/// [`RootSet::put_frame_aliases`].
pub struct FrameAliases(Vec<Option<usize>>);

impl SlotRef {
    fn is_cell(self) -> bool {
        self.0 & CELL_TAG != 0
    }
}

/// Everything the collector starts from.
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
    /// Exactly parallel to `slots`: `Some(target)` at absolute position `p`
    /// means position `p` is an **alias** for absolute position `target`,
    /// and every read and write addressed to `p` is served by `target`
    /// instead. `None` is the ordinary case, a slot that is its own storage.
    aliases: Vec<Option<usize>>,
    /// How many entries of `aliases` are `Some`.
    alias_count: usize,
    /// Storage for variables a `>name` reference has been taken to, outside
    /// every frame and never truncated.
    cells: Vec<Option<ObjRef>>,
    /// The starting offset of every currently pushed frame, in push order.
    /// Its length is also every live frame's `depth` plus one, which is how
    /// `grow_slots` and `pop_slots` recognise the top frame.
    frame_starts: Vec<usize>,
    /// Values an activation that is **not** on any stack still owns.
    parked: Vec<Option<Vec<ObjRef>>>,
    /// Indices of `parked` that are `None`, so a park after a release reuses
    /// an entry instead of extending the vector for the life of the process.
    parked_free: Vec<usize>,
}

/// A handle to one parked set of values, issued by [`RootSet::park`] and spent
/// by [`RootSet::release`].
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Parked(usize);

impl RootSet {
    pub fn new() -> Self {
        RootSet {
            globals: Vec::new(),
            temps: Vec::new(),
            slots: Vec::new(),
            aliases: Vec::new(),
            alias_count: 0,
            cells: Vec::new(),
            frame_starts: Vec::new(),
            parked: Vec::new(),
            parked_free: Vec::new(),
        }
    }

    /// Roots `values` until the handle is spent, independently of every stack
    /// in this type.
    pub fn park(&mut self, values: Vec<ObjRef>) -> Parked {
        match self.parked_free.pop() {
            Some(index) => {
                self.parked[index] = Some(values);
                Parked(index)
            }
            None => {
                self.parked.push(Some(values));
                Parked(self.parked.len() - 1)
            }
        }
    }

    /// Drops what `parked` was rooting.
    pub fn release(&mut self, parked: Parked) {
        assert!(
            self.parked[parked.0].take().is_some(),
            "release on a parked entry that was already released"
        );
        self.parked_free.push(parked.0);
    }

    /// How many parked entries are currently rooting anything.
    pub fn live_parked(&self) -> usize {
        self.parked.iter().flatten().count()
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
    pub fn pop_frame(&mut self, frame: FrameId) {
        self.temps.truncate(frame.0);
    }

    pub fn push_temp(&mut self, value: ObjRef) {
        self.temps.push(value);
    }

    /// Opens a region of `count` indexable temporaries, all [`ObjRef::NIL`],
    /// and returns the watermark below them.
    pub fn reserve_temps(&mut self, count: usize) -> FrameId {
        let frame = FrameId(self.temps.len());
        self.temps.resize(self.temps.len() + count, ObjRef::NIL);
        frame
    }

    /// Reads register `index` of the region opened at `frame`.
    pub fn temp_at(&self, frame: FrameId, index: usize) -> ObjRef {
        assert!(frame.0 + index < self.temps.len());
        self.temps[frame.0 + index]
    }

    /// Writes register `index` of the region opened at `frame`.
    pub fn set_temp(&mut self, frame: FrameId, index: usize, value: ObjRef) {
        assert!(frame.0 + index < self.temps.len());
        self.temps[frame.0 + index] = value;
    }

    /// How many temporaries are currently rooted.
    pub fn temps_len(&self) -> usize {
        self.temps.len()
    }

    /// Opens a new slot frame of `initial_len` unassigned slots, for an
    /// activation entering with a plan of that many resolved names (D16).
    pub fn push_slots(&mut self, initial_len: usize) -> SlotFrame {
        let start = self.slots.len();
        self.slots.resize(start + initial_len, None);
        self.aliases.resize(start + initial_len, None);
        let depth = self.frame_starts.len();
        self.frame_starts.push(start);
        SlotFrame { start, depth }
    }

    /// How many slot frames are currently open.
    pub fn live_frames(&self) -> usize {
        self.frame_starts.len()
    }

    /// How many slots `frame` currently holds, its own growth included.
    pub fn frame_len(&self, frame: SlotFrame) -> usize {
        let end = self
            .frame_starts
            .get(frame.depth + 1)
            .copied()
            .unwrap_or(self.slots.len());
        end - frame.start
    }

    /// How many of `frame`'s slots are aliases for storage somewhere else.
    pub fn frame_aliases(&self, frame: SlotFrame) -> usize {
        let end = frame.start + self.frame_len(frame);
        self.aliases[frame.start..end]
            .iter()
            .flatten()
            .filter(|target| !SlotRef(**target).is_cell())
            .count()
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
        // Truncated together with `slots`, never separately: the two are
        // parallel by construction, and an `aliases` left longer would give
        // the *next* frame pushed at this offset a set of stale redirects
        // pointing into a dead activation's storage.
        // The count goes with them. Only walked when there is something to
        // find, so a program that never aliased pays one test per frame pop.
        if self.alias_count != 0 {
            let dropped = self.aliases[frame.start..].iter().flatten().count();
            self.alias_count -= dropped;
        }
        self.aliases.truncate(frame.start);
    }

    /// The absolute position slot `index` of `frame` finally resolves to.
    pub fn slot_ref(&self, frame: SlotFrame, index: usize) -> SlotRef {
        SlotRef(self.resolve(frame, index))
    }

    /// Moves slot `index` of `frame` into a cell and answers that cell, so
    /// that a reference to the variable survives the frame.
    pub fn promote(&mut self, frame: SlotFrame, index: usize) -> SlotRef {
        let position = self.resolve(frame, index);
        if SlotRef(position).is_cell() {
            return SlotRef(position);
        }
        self.cells.push(self.slots[position]);
        let cell = (self.cells.len() - 1) | CELL_TAG;
        self.slots[position] = None;
        if self.aliases[position].is_none() {
            self.alias_count += 1;
        }
        self.aliases[position] = Some(cell);
        SlotRef(cell)
    }

    /// Copies out `frame`'s alias entries, for a caller that is about to
    /// release the frame and re-push it later.
    pub fn take_frame_aliases(&self, frame: SlotFrame) -> FrameAliases {
        let end = frame.start + self.frame_len(frame);
        FrameAliases(self.aliases[frame.start..end].to_vec())
    }

    /// Puts back what [`RootSet::take_frame_aliases`] copied out, into a
    /// frame pushed at the same length.
    pub fn put_frame_aliases(&mut self, frame: SlotFrame, saved: &FrameAliases) {
        for (index, entry) in saved.0.iter().enumerate() {
            let Some(target) = *entry else { continue };
            let at = &mut self.aliases[frame.start + index];
            if at.is_none() {
                self.alias_count += 1;
            }
            *at = Some(target);
        }
    }

    /// Reads the storage `slot` names, `None` for an unassigned variable.
    pub fn slot_value(&self, slot: SlotRef) -> Option<ObjRef> {
        self.at(slot.0)
    }

    /// Writes the storage `slot` names.
    pub fn set_slot_value(&mut self, slot: SlotRef, value: ObjRef) {
        self.write(slot.0, Some(value));
    }

    /// Makes slot `index` of `frame` an alias for `target`: every later
    /// read and write addressed to it is served by `target`'s storage
    /// instead of its own.
    pub fn alias_slot(&mut self, frame: SlotFrame, index: usize, target: SlotRef) {
        let entry = &mut self.aliases[frame.start + index];
        if entry.is_none() {
            self.alias_count += 1;
        }
        *entry = Some(target.0);
    }

    /// `frame`'s slot `index` as an absolute position, following an alias if
    /// one is in force. The one place the redirect is applied, so that
    /// `frame_slot`/`set_frame_slot`/`clear_frame_slot` cannot come apart on
    /// it.
    fn resolve(&self, frame: SlotFrame, index: usize) -> usize {
        let position = frame.start + index;
        // Nothing is aliased anywhere, so nothing can redirect: the parallel
        // vector is not read at all. See `alias_count` for what that is worth.
        if self.alias_count == 0 {
            debug_assert!(
                self.aliases[position].is_none(),
                "slot {position} redirects while the alias count says none does"
            );
            return position;
        }
        self.resolve_aliased(position)
    }

    /// [`RootSet::resolve`]'s slow half, for a caller that has already found
    /// an alias may be in force.
    #[inline(always)]
    fn resolve_aliased(&self, position: usize) -> usize {
        let mut at = position;
        while at & CELL_TAG == 0 {
            let Some(target) = self.aliases[at] else {
                break;
            };
            debug_assert!(
                target & CELL_TAG != 0 || target < at,
                "alias at {at} points at {target}, which does not descend"
            );
            at = target;
        }
        at
    }

    /// Reads slot `index` within `frame`: `None` for an unassigned or
    /// `DROP`ped variable, which is a legal outcome and not an error.
    #[inline(always)]
    pub fn frame_slot(&self, frame: SlotFrame, index: usize) -> Option<ObjRef> {
        let position = frame.start + index;
        if self.alias_count == 0 {
            debug_assert!(
                self.aliases[position].is_none(),
                "slot {position} redirects while the alias count says none does"
            );
            assert!(position < self.slots.len());
            return self.slots[position];
        }
        self.at(self.resolve_aliased(position))
    }

    #[inline(always)]
    pub fn set_frame_slot(&mut self, frame: SlotFrame, index: usize, value: ObjRef) {
        let position = frame.start + index;
        if self.alias_count == 0 {
            debug_assert!(
                self.aliases[position].is_none(),
                "slot {position} redirects while the alias count says none does"
            );
            assert!(position < self.slots.len());
            self.slots[position] = Some(value);
            return;
        }
        let position = self.resolve_aliased(position);
        self.write(position, Some(value));
    }

    /// Returns slot `index` within `frame` to the unset state, which is what
    /// `DROP` on a simple variable does.
    /// ```text
    /// a = 5     ; drop a ; say a   ->  A                 (unset: derived name)
    /// x = .nil            ; say x  ->  The NIL object    (`.nil` is a value)
    /// y = .nil  ; drop y  ; say y  ->  Y                 (unset, not NIL)
    /// ```
    pub fn clear_frame_slot(&mut self, frame: SlotFrame, index: usize) {
        let position = self.resolve(frame, index);
        self.write(position, None);
    }

    /// Reads the storage at a tagged position: the frame arena, or a cell.
    #[inline(always)]
    fn at(&self, position: usize) -> Option<ObjRef> {
        if position & CELL_TAG == 0 {
            assert!(position < self.slots.len());
            self.slots[position]
        } else {
            self.cells[position & !CELL_TAG]
        }
    }

    /// [`RootSet::at`]'s write, in the same position.
    #[inline(always)]
    fn write(&mut self, position: usize, value: Option<ObjRef>) {
        if position & CELL_TAG == 0 {
            assert!(position < self.slots.len());
            self.slots[position] = value;
        } else {
            self.cells[position & !CELL_TAG] = value;
        }
    }

    /// Grows `frame` by one slot for a name its plan never saw -- `DROP (v)`
    /// naming its target at run time, measured: `v = 'X'; x = 1; drop (v);
    /// say x` prints `X`, so a name resolving to no existing slot must be
    /// able to allocate one. Returns the new slot's index within `frame`.
    pub fn grow_slots(&mut self, frame: SlotFrame) -> usize {
        assert_eq!(
            self.frame_starts.len(),
            frame.depth + 1,
            "grow_slots on a frame that is not the top one (a 4a invariant, \
             kept: 4b binds exposed names with alias_slot and resolves a \
             computed expose (v) before the callee's frame is pushed, so \
             neither needs this -- see grow_slots's doc comment)"
        );
        let index = self.slots.len() - frame.start;
        self.slots.push(None);
        // Kept parallel; a new slot is its own storage, never an alias.
        self.aliases.push(None);
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
            .chain(self.cells.iter().filter_map(|c| *c))
            .chain(self.parked.iter().flatten().flatten().copied())
    }
}

impl Default for RootSet {
    fn default() -> Self {
        Self::new()
    }
}
