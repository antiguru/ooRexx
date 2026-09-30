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

use std::rc::Rc;

use crate::ObjRef;
use crate::frame::{FrameArena, FrameBlock};

/// A position in the temporary stack that a frame will unwind to.
#[derive(Copy, Clone, Debug)]
pub struct FrameId(usize);

/// A handle to one activation's segment of local-variable slots (D16).
/// `push_slots`/`pop_slots` bracket its lifetime; `frame_slot`,
/// `set_frame_slot` and `grow_slots_of` address within it by an offset from
/// the segment's start, which only the segment's record holds.
///
/// `serial` is the activation's identity: [`RootSet::push_slots`] numbers
/// every segment it opens, across every activity, and the record keeps the
/// number, so a handle resolved against a record that is not its own is
/// caught (a debug assertion). `depth` is where that record sits in its
/// activity's stack of records.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct SlotFrame {
    depth: usize,
    serial: u64,
}

/// One variable's storage, with any alias already followed: what
/// `PROCEDURE EXPOSE` and `USE ARG >name` bind a callee's slot *to*, and
/// what a `>name` reference names.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct SlotRef(Target);

/// Where a [`SlotRef`] or an alias entry leads: an offset in one
/// activation's segment, or a cell.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum Target {
    Slot { frame: SlotFrame, index: usize },
    Cell(usize),
}

/// One frame's alias entries, saved across a park by
/// [`ActivityRoots::take_frame_aliases`] and put back by
/// [`ActivityRoots::put_frame_aliases`].
pub struct FrameAliases(Vec<Option<Target>>);

/// Where one open frame's segment starts in `ActivityRoots::slots`, and whose
/// segment it is.
#[derive(Copy, Clone, Debug)]
struct Segment {
    start: usize,
    serial: u64,
}

/// Everything the collector starts from.
pub struct RootSet {
    globals: Vec<(String, ObjRef)>,
    /// Storage for variables a `>name` reference has been taken to, outside
    /// every frame and never truncated.
    cells: Vec<Option<ObjRef>>,
    /// The next [`SlotFrame::serial`].
    next_serial: u64,
    /// The running activity's roots.
    activity: ActivityRoots,
}

/// One activity's roots: its temporaries, register frames, slots and
/// parked values.
pub struct ActivityRoots {
    temps: Vec<ObjRef>,
    /// The driver's register frames.
    frames: Rc<FrameArena>,
    /// Local-variable slots for every open frame, one segment each, in push
    /// order: a frame's segment runs from its record's `start` to the next
    /// record's. `None` is an unassigned (or `DROP`ped) variable, not a
    /// missing one -- unlike `temps`, whose entries are always live values,
    /// a slot must be able to say "no value" without that colliding with
    /// `ObjRef::NIL`, which is itself a legal Rexx value (`x = .nil`).
    slots: Vec<Option<ObjRef>>,
    /// Parallel to a prefix of `slots`: `Some(target)` at position `p` means
    /// that slot is an **alias** for `target`, and every read and write
    /// addressed to it is served by `target` instead. `None`, or a position
    /// past the end, is a slot that is its own storage.
    aliases: Vec<Option<Target>>,
    /// One record per open frame, in push order, so a frame's `depth`
    /// indexes its own.
    segments: Vec<Segment>,
    /// The top record's `start`.
    top_start: usize,
    /// How many entries of `aliases` are `Some`, plus one for each
    /// [`ActivityRoots::begin_indirect`] not yet ended. While it is zero no
    /// slot redirects and every frame addressed is the top one.
    indirect: usize,
    /// Values an activation that is **not** on any stack still owns.
    parked: Vec<Option<Vec<ObjRef>>>,
    /// Indices of `parked` that are `None`, so a park after a release reuses
    /// an entry instead of extending the vector for the life of the process.
    parked_free: Vec<usize>,
}

/// A handle to one parked set of values, issued by [`ActivityRoots::park`] and spent
/// by [`ActivityRoots::release`].
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Parked(usize);

impl RootSet {
    pub fn new() -> Self {
        RootSet {
            globals: Vec::new(),
            cells: Vec::new(),
            next_serial: 0,
            activity: ActivityRoots::new(),
        }
    }

    pub fn add_global(&mut self, name: &str, value: ObjRef) {
        match self.globals.iter_mut().find(|(n, _)| n == name) {
            Some(entry) => entry.1 = value,
            None => self.globals.push((name.to_string(), value)),
        }
    }

    /// The running activity's roots.
    #[inline(always)]
    pub fn activity(&self) -> &ActivityRoots {
        &self.activity
    }

    /// The running activity's roots, mutably.
    #[inline(always)]
    pub fn activity_mut(&mut self) -> &mut ActivityRoots {
        &mut self.activity
    }

    /// Opens a new slot frame of `initial_len` unassigned slots in the
    /// running activity, for an activation entering with a plan of that many
    /// resolved names (D16).
    pub fn push_slots(&mut self, initial_len: usize) -> SlotFrame {
        let serial = self.next_serial;
        self.next_serial += 1;
        self.activity.push_segment(initial_len, serial)
    }

    /// Moves slot `index` of `frame` into a cell and answers that cell, so
    /// that a reference to the variable survives the frame.
    pub fn promote(&mut self, frame: SlotFrame, index: usize) -> SlotRef {
        let target = self.activity.resolve(frame, index);
        let Target::Slot { frame, index } = target else {
            return SlotRef(target);
        };
        let position = self.activity.position(frame, index);
        self.cells.push(self.activity.slots[position].take());
        let cell = Target::Cell(self.cells.len() - 1);
        self.activity.set_alias(position, cell);
        SlotRef(cell)
    }

    /// Reads the storage `slot` names, `None` for an unassigned variable.
    pub fn slot_value(&self, slot: SlotRef) -> Option<ObjRef> {
        self.at(slot.0)
    }

    /// Writes the storage `slot` names.
    pub fn set_slot_value(&mut self, slot: SlotRef, value: ObjRef) {
        self.write(slot.0, Some(value));
    }

    /// Reads slot `index` within `frame`: `None` for an unassigned or
    /// `DROP`ped variable, which is a legal outcome and not an error.
    ///
    /// `frame` is the top one, or an [`ActivityRoots::begin_indirect`] is in
    /// force; [`RootSet::frame_slot_of`] reads any open frame.
    #[inline(always)]
    pub fn frame_slot(&self, frame: SlotFrame, index: usize) -> Option<ObjRef> {
        let activity = &self.activity;
        if activity.indirect == 0 {
            activity.debug_assert_top(frame);
            let position = activity.top_start + index;
            debug_assert!(
                activity.aliases.get(position).is_none_or(Option::is_none),
                "slot {position} redirects while the count says none does"
            );
            assert!(position < activity.slots.len());
            return activity.slots[position];
        }
        self.frame_slot_of(frame, index)
    }

    /// [`RootSet::frame_slot`] for a frame that need not be the top one.
    #[cold]
    #[inline(never)]
    pub fn frame_slot_of(&self, frame: SlotFrame, index: usize) -> Option<ObjRef> {
        self.at(self.activity.resolve(frame, index))
    }

    /// Writes slot `index` within `frame`, which is the top one or an
    /// [`ActivityRoots::begin_indirect`] is in force.
    #[inline(always)]
    pub fn set_frame_slot(&mut self, frame: SlotFrame, index: usize, value: ObjRef) {
        let activity = &mut self.activity;
        if activity.indirect == 0 {
            activity.debug_assert_top(frame);
            let position = activity.top_start + index;
            debug_assert!(
                activity.aliases.get(position).is_none_or(Option::is_none),
                "slot {position} redirects while the count says none does"
            );
            assert!(position < activity.slots.len());
            activity.slots[position] = Some(value);
            return;
        }
        self.set_frame_slot_of(frame, index, value);
    }

    /// [`RootSet::set_frame_slot`] for a frame that need not be the top one.
    #[cold]
    #[inline(never)]
    pub fn set_frame_slot_of(&mut self, frame: SlotFrame, index: usize, value: ObjRef) {
        let target = self.activity.resolve(frame, index);
        self.write(target, Some(value));
    }

    /// Returns slot `index` within `frame` to the unset state, which is what
    /// `DROP` on a simple variable does.
    /// ```text
    /// a = 5     ; drop a ; say a   ->  A                 (unset: derived name)
    /// x = .nil            ; say x  ->  The NIL object    (`.nil` is a value)
    /// y = .nil  ; drop y  ; say y  ->  Y                 (unset, not NIL)
    /// ```
    ///
    /// `frame` is the top one, or an [`ActivityRoots::begin_indirect`] is in
    /// force.
    #[inline(always)]
    pub fn clear_frame_slot(&mut self, frame: SlotFrame, index: usize) {
        let activity = &mut self.activity;
        if activity.indirect == 0 {
            activity.debug_assert_top(frame);
            let position = activity.top_start + index;
            assert!(position < activity.slots.len());
            activity.slots[position] = None;
            return;
        }
        self.clear_frame_slot_of(frame, index);
    }

    /// [`RootSet::clear_frame_slot`] for a frame that need not be the top
    /// one.
    #[cold]
    #[inline(never)]
    pub fn clear_frame_slot_of(&mut self, frame: SlotFrame, index: usize) {
        let target = self.activity.resolve(frame, index);
        self.write(target, None);
    }

    /// Reads the storage `target` names: a frame's slot, or a cell.
    #[inline(always)]
    fn at(&self, target: Target) -> Option<ObjRef> {
        match target {
            Target::Slot { frame, index } => {
                self.activity.slots[self.activity.position(frame, index)]
            }
            Target::Cell(cell) => self.cells[cell],
        }
    }

    /// [`RootSet::at`]'s write, to the same storage.
    #[inline(always)]
    fn write(&mut self, target: Target, value: Option<ObjRef>) {
        match target {
            Target::Slot { frame, index } => {
                let position = self.activity.position(frame, index);
                self.activity.slots[position] = value;
            }
            Target::Cell(cell) => self.cells[cell] = value,
        }
    }

    /// Yields globals, temps, every live register, and every assigned slot
    /// across every currently active frame -- a popped frame's slots are
    /// already gone, truncated out of `slots` by `pop_slots`, so nothing here
    /// needs to filter them out again by frame.
    pub fn iter(&self) -> impl Iterator<Item = ObjRef> + '_ {
        self.globals
            .iter()
            .map(|(_, v)| *v)
            .chain(self.cells.iter().filter_map(|c| *c))
            .chain(self.activity.iter())
    }
}

impl ActivityRoots {
    pub fn new() -> Self {
        ActivityRoots {
            temps: Vec::new(),
            frames: Rc::new(FrameArena::new(FrameBlock::DEFAULT)),
            slots: Vec::new(),
            aliases: Vec::new(),
            segments: Vec::new(),
            top_start: 0,
            indirect: 0,
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

    /// The register frame arena, whose frames are roots while reserved. A
    /// frame borrows the handle this answers, not `self`.
    pub fn frames(&self) -> Rc<FrameArena> {
        Rc::clone(&self.frames)
    }

    /// Replaces the register frame arena with an empty one of `size`.
    ///
    /// # Panics
    ///
    /// If the current arena has ever reserved a frame or is held elsewhere.
    pub fn set_frame_block(&mut self, size: FrameBlock) {
        assert!(
            self.frames.blocks() == 0 && Rc::strong_count(&self.frames) == 1,
            "the frame block size is set before the first frame"
        );
        self.frames = Rc::new(FrameArena::new(size));
    }

    /// How many temporaries are currently rooted.
    pub fn temps_len(&self) -> usize {
        self.temps.len()
    }

    /// Opens a segment of `initial_len` unassigned slots for the frame
    /// numbered `serial`.
    fn push_segment(&mut self, initial_len: usize, serial: u64) -> SlotFrame {
        let start = self.slots.len();
        self.slots.resize(start + initial_len, None);
        let depth = self.segments.len();
        self.segments.push(Segment { start, serial });
        self.top_start = start;
        SlotFrame { depth, serial }
    }

    /// `frame`'s record.
    ///
    /// # Panics
    ///
    /// If `frame` is closed; in a debug build, also if its depth now holds
    /// another frame's record.
    #[inline(always)]
    fn segment(&self, frame: SlotFrame) -> Segment {
        let segment = self.segments[frame.depth];
        debug_assert_eq!(
            segment.serial, frame.serial,
            "slot frame {frame:?} resolved against another frame's record"
        );
        segment
    }

    /// Where `frame`'s segment ends.
    fn end_of(&self, frame: SlotFrame) -> usize {
        self.segments
            .get(frame.depth + 1)
            .map_or(self.slots.len(), |next| next.start)
    }

    /// The position in `slots` of slot `index` of `frame`.
    #[inline(always)]
    fn position(&self, frame: SlotFrame, index: usize) -> usize {
        let position = self.segment(frame).start + index;
        debug_assert!(
            position < self.end_of(frame),
            "slot {index} is past the end of {frame:?}"
        );
        position
    }

    #[inline(always)]
    fn debug_assert_top(&self, frame: SlotFrame) {
        debug_assert!(
            self.segments.len() == frame.depth + 1 && self.segment(frame).start == self.top_start,
            "slot frame {frame:?} addressed as the top one"
        );
    }

    /// How many slot frames are currently open.
    pub fn live_frames(&self) -> usize {
        self.segments.len()
    }

    /// How many slots `frame` currently holds, its own growth included.
    pub fn frame_len(&self, frame: SlotFrame) -> usize {
        self.end_of(frame) - self.segment(frame).start
    }

    /// `frame`'s alias entries, as far as `aliases` reaches into its segment.
    fn aliases_of(&self, frame: SlotFrame) -> &[Option<Target>] {
        let end = self.end_of(frame).min(self.aliases.len());
        let start = self.segment(frame).start.min(end);
        &self.aliases[start..end]
    }

    /// How many of `frame`'s slots are aliases for storage somewhere else.
    pub fn frame_aliases(&self, frame: SlotFrame) -> usize {
        self.aliases_of(frame)
            .iter()
            .flatten()
            .filter(|target| matches!(target, Target::Slot { .. }))
            .count()
    }

    /// Closes `frame`, releasing its slots. Frames nest like any stack, so
    /// this must be the top one.
    pub fn pop_slots(&mut self, frame: SlotFrame) {
        assert_eq!(
            self.segments.len(),
            frame.depth + 1,
            "pop_slots on a frame that is not the top one"
        );
        let start = self.segment(frame).start;
        self.segments.pop();
        self.top_start = self.segments.last().map_or(0, |below| below.start);
        self.slots.truncate(start);
        if self.aliases.len() > start {
            let dropped = self.aliases[start..].iter().flatten().count();
            self.indirect -= dropped;
            self.aliases.truncate(start);
        }
    }

    /// The storage slot `index` of `frame` finally resolves to.
    pub fn slot_ref(&self, frame: SlotFrame, index: usize) -> SlotRef {
        SlotRef(self.resolve(frame, index))
    }

    /// Copies out `frame`'s alias entries, for a caller that is about to
    /// release the frame and re-push it later.
    pub fn take_frame_aliases(&self, frame: SlotFrame) -> FrameAliases {
        FrameAliases(self.aliases_of(frame).to_vec())
    }

    /// Puts back what [`ActivityRoots::take_frame_aliases`] copied out, into a
    /// frame pushed at the same length.
    pub fn put_frame_aliases(&mut self, frame: SlotFrame, saved: &FrameAliases) {
        for (index, entry) in saved.0.iter().enumerate() {
            if let Some(target) = *entry {
                let position = self.position(frame, index);
                self.set_alias(position, target);
            }
        }
    }

    /// Makes slot `index` of `frame` an alias for `target`: every later
    /// read and write addressed to it is served by `target`'s storage
    /// instead of its own.
    pub fn alias_slot(&mut self, frame: SlotFrame, index: usize, target: SlotRef) {
        let position = self.position(frame, index);
        self.set_alias(position, target.0);
    }

    fn set_alias(&mut self, position: usize, target: Target) {
        if self.aliases.len() <= position {
            self.aliases.resize(position + 1, None);
        }
        if self.aliases[position].replace(target).is_none() {
            self.indirect += 1;
        }
    }

    /// Opens a stretch in which frames other than the top one may be
    /// addressed through [`RootSet::frame_slot`] and its writes, each through
    /// its own record. Ended by [`ActivityRoots::end_indirect`].
    pub fn begin_indirect(&mut self) {
        self.indirect += 1;
    }

    /// Ends the innermost [`ActivityRoots::begin_indirect`].
    pub fn end_indirect(&mut self) {
        self.indirect -= 1;
    }

    /// `frame`'s slot `index`, following an alias if one is in force. The
    /// one place the redirect is applied, so that
    /// `frame_slot`/`set_frame_slot`/`clear_frame_slot` cannot come apart on
    /// it.
    fn resolve(&self, frame: SlotFrame, index: usize) -> Target {
        if self.indirect == 0 {
            debug_assert!(
                self.aliases_of(frame).iter().all(Option::is_none),
                "{frame:?} redirects while the count says nothing does"
            );
            return Target::Slot { frame, index };
        }
        self.resolve_aliased(frame, index)
    }

    /// [`ActivityRoots::resolve`]'s slow half, for a caller that has already found
    /// an alias may be in force.
    #[inline(always)]
    fn resolve_aliased(&self, frame: SlotFrame, index: usize) -> Target {
        let mut at = Target::Slot { frame, index };
        while let Target::Slot { frame, index } = at {
            let position = self.position(frame, index);
            let Some(Some(target)) = self.aliases.get(position).copied() else {
                break;
            };
            debug_assert!(
                match target {
                    Target::Slot {
                        frame: to,
                        index: to_index,
                    } => (to.depth, to_index) < (frame.depth, index),
                    Target::Cell(_) => true,
                },
                "alias at {at:?} points at {target:?}, which does not descend"
            );
            at = target;
        }
        at
    }

    /// Grows `frame` by `n` slots, for names its plan never saw -- `DROP (v)`
    /// naming its target at run time, measured: `v = 'X'; x = 1; drop (v);
    /// say x` prints `X`, so a name resolving to no existing slot must be
    /// able to allocate one. Returns the first new slot's index within
    /// `frame`.
    ///
    /// `frame` need not be the top one: the frames above it keep their
    /// offsets and their records move.
    pub fn grow_slots_of(&mut self, frame: SlotFrame, n: usize) -> usize {
        let start = self.segment(frame).start;
        let end = self.end_of(frame);
        self.slots.splice(end..end, std::iter::repeat_n(None, n));
        if self.aliases.len() > end {
            self.aliases.splice(end..end, std::iter::repeat_n(None, n));
        }
        for above in &mut self.segments[frame.depth + 1..] {
            above.start += n;
        }
        self.top_start = self.segments.last().map_or(0, |top| top.start);
        end - start
    }

    /// Yields this activity's temps, live registers, assigned slots and
    /// parked values.
    pub fn iter(&self) -> impl Iterator<Item = ObjRef> + '_ {
        self.temps
            .iter()
            .copied()
            .chain(self.frames.iter())
            .chain(self.slots.iter().filter_map(|s| *s))
            .chain(self.parked.iter().flatten().flatten().copied())
    }
}

impl Default for RootSet {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ActivityRoots {
    fn default() -> Self {
        Self::new()
    }
}
