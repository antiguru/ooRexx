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

/// One slot's absolute position in the arena, with any alias already
/// followed: what `PROCEDURE EXPOSE` and `USE ARG >name` bind a callee's
/// slot *to*.
///
/// A newtype rather than a bare `usize` because the two are not
/// interchangeable at a call site: every other index in this file is
/// relative to a `SlotFrame`, and an absolute one passed where a relative
/// one belongs addresses a real slot in some other activation's range
/// rather than failing. Only [`RootSet::slot_ref`] produces one, and it
/// chases before returning, so a `SlotRef` is by construction a final
/// destination and never itself an alias.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct SlotRef(usize);

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
    /// Exactly parallel to `slots`: `Some(target)` at absolute position `p`
    /// means position `p` is an **alias** for absolute position `target`,
    /// and every read and write addressed to `p` is served by `target`
    /// instead. `None` is the ordinary case, a slot that is its own storage.
    ///
    /// This is where `PROCEDURE EXPOSE` lives (4b's Task 5). A `PROCEDURE`
    /// callee gets a frame of its own, so its variables are isolated; an
    /// exposed name's slot in that frame is aliased to the entry it was
    /// exposed *from*, so reads and writes through it reach the other
    /// activation's storage. `USE ARG >name` binds one slot the same way.
    ///
    /// **A parallel vector rather than a per-frame map**, because the
    /// resolution is on the hot path -- variable lookup is 8.1%/32.2% of
    /// runtime -- and this shape costs one indexed load and one branch with
    /// no hashing and no second bounds regime. It is a `Vec<Option<usize>>`
    /// and not a bitset-plus-one-target-frame, and that is measured rather
    /// than a preference: one `PROCEDURE` can expose two names that resolve
    /// to two *different* frames. Measured on the oracle, `a` calling `b:
    /// procedure expose n` calling `c: procedure expose n m`, with `c`
    /// writing both -- `b` sees both writes, `a` sees only `n`'s, because
    /// `m` was `b`'s own local and `n` was chased through `b`'s alias to
    /// `a`. A single target frame per callee cannot represent that pair.
    ///
    /// **Aliases are recorded per slot and already chased** ([`SlotRef`]),
    /// so resolution here follows exactly one link and never loops. The
    /// chase happens once, at bind time, where the intermediate frame is
    /// still addressable.
    aliases: Vec<Option<usize>>,
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
            aliases: Vec::new(),
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

    /// How many temporaries are currently rooted.
    ///
    /// For a **debug tripwire only**, and specifically for the one
    /// `rexx-exec`'s `step_in_temps_frame` carries: comparing this before and
    /// after a step is how that function checks nothing popped below its own
    /// watermark. `pop_frame`'s own doc explains why the balance cannot be
    /// asserted *here*, in the general case, without first making six
    /// `rexx-exec` sites pop on their own error path; the caller's tripwire is
    /// narrower (one call site, `Ok` path only) and needs no such change.
    ///
    /// Not a capacity, a budget, or anything a decision should be made from.
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
    ///
    /// **For asserting that frames are released, and nothing else.** A
    /// `PROCEDURE` callee allocates a frame and `resolve_and_run_call`
    /// releases it on both the ordinary and the error path; a missing
    /// release is invisible in a program's output -- the run stays correct
    /// and simply holds one frame per call forever, which `do 100000; call
    /// sub; end` turns into 100,000 rooted frames. This is what lets a test
    /// see that directly instead of inferring it.
    ///
    /// Counts frames rather than slots on purpose. A slot count also moves
    /// for reasons that are not leaks -- the first `CALL` in a program that
    /// never writes `RESULT` grows the top frame by one to hold it -- so a
    /// test written against slots has to model those too, and gets a
    /// different baseline on the ordinary and the error path because the
    /// error path never reaches the `RESULT` write. The frame count has no
    /// such confounder: it is one per live activation-with-a-pool, whatever
    /// happened inside them.
    ///
    /// Not a capacity or a budget, exactly like `temps_len` beside it.
    pub fn live_frames(&self) -> usize {
        self.frame_starts.len()
    }

    /// How many slots `frame` currently holds, its own growth included.
    ///
    /// What a `PROCEDURE` callee's frame is sized from: the exposed names
    /// were resolved to indices in the *caller's* frame, so the callee's
    /// frame has to be at least as long for those same indices to address
    /// anything at all. Sizing it from the plan alone would be one slot
    /// short for every name the caller grew at run time.
    ///
    /// A frame ends where the next one begins, and the top frame ends at
    /// the end of the arena -- which is the same fact `grow_slots` relies
    /// on, read here instead of assumed.
    pub fn frame_len(&self, frame: SlotFrame) -> usize {
        let end = self
            .frame_starts
            .get(frame.depth + 1)
            .copied()
            .unwrap_or(self.slots.len());
        end - frame.start
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
        self.aliases.truncate(frame.start);
    }

    /// The absolute position slot `index` of `frame` finally resolves to.
    ///
    /// The only producer of a [`SlotRef`], and it chases: if the slot is
    /// itself an alias, the answer is what it aliases, so the result is
    /// always a final destination. That single step is what makes exposure
    /// **transitive** -- measured on the oracle, `a` exposing `n` to `b` and
    /// `b` exposing the same `n` to `c` leaves `c`'s write visible in `a`,
    /// which only happens if binding `c` resolves through `b`'s alias to
    /// `a`'s storage rather than stopping at `b`'s frame.
    ///
    /// One step suffices for all depths precisely because every alias this
    /// type records was produced from a `SlotRef` and so was chased when it
    /// was made; there is no chain here to walk, by induction on the order
    /// the frames were pushed.
    pub fn slot_ref(&self, frame: SlotFrame, index: usize) -> SlotRef {
        SlotRef(self.resolve(frame, index))
    }

    /// Makes slot `index` of `frame` an alias for `target`: every later
    /// read and write addressed to it is served by `target`'s storage
    /// instead of its own.
    ///
    /// `PROCEDURE EXPOSE` and `USE ARG >name` are the two callers, and both
    /// establish that the slot holds nothing worth reaching before calling:
    /// `PROCEDURE` aliases into a frame it has just pushed, whose slots are
    /// all `None`, and `USE ARG >name` refuses a target that is not
    /// uninitialised (error 98.995, `run.rs`'s `target_is_uninitialised`).
    /// The aliased slot's own storage then stops being reachable by name.
    ///
    /// It is **not** guaranteed to stay `None`, and the one exception is
    /// deliberate: `USE ARG >q.` accepts a stem slot holding a vivified but
    /// empty `Body::Stem`, because a bare stem read and a `DROP` both leave
    /// one and the oracle treats the variable as uninitialised in both cases.
    /// So an aliasing slot's own entry is either `None` or an empty stem that
    /// nothing can now reach. See [`iter`] for what that costs.
    ///
    /// [`iter`]: RootSet::iter
    pub fn alias_slot(&mut self, frame: SlotFrame, index: usize, target: SlotRef) {
        self.aliases[frame.start + index] = Some(target.0);
    }

    /// `frame`'s slot `index` as an absolute position, following an alias if
    /// one is in force. The one place the redirect is applied, so that
    /// `slot`/`set_slot`/`clear_slot` cannot come apart on it.
    fn resolve(&self, frame: SlotFrame, index: usize) -> usize {
        let position = frame.start + index;
        // `unwrap_or` and not a loop: see `slot_ref`.
        self.aliases[position].unwrap_or(position)
    }

    /// Reads slot `index` within `frame`: `None` for an unassigned or
    /// `DROP`ped variable, which is a legal outcome and not an error.
    pub fn slot(&self, frame: SlotFrame, index: usize) -> Option<ObjRef> {
        self.slots[self.resolve(frame, index)]
    }

    pub fn set_slot(&mut self, frame: SlotFrame, index: usize, value: ObjRef) {
        let position = self.resolve(frame, index);
        self.slots[position] = Some(value);
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
        let position = self.resolve(frame, index);
        self.slots[position] = None;
    }

    /// Grows `frame` by one slot for a name its plan never saw -- `DROP (v)`
    /// naming its target at run time, measured: `v = 'X'; x = 1; drop (v);
    /// say x` prints `X`, so a name resolving to no existing slot must be
    /// able to allocate one. Returns the new slot's index within `frame`.
    ///
    /// Only the top frame may grow. **4b made the decision this invariant
    /// was waiting on, and the invariant stands unchanged.**
    ///
    /// It held in 4a because 4a has exactly one frame, and for `INTERPRET`,
    /// which runs inside the activation that created it rather than pushing
    /// its own. The case that looked like it would break it is a callee
    /// writing into its caller's pool -- measured, `sub: procedure expose
    /// zzz` with `zzz = 5` set in the callee makes the *caller* print 5
    /// after `return`, while the callee's frame sits on top of the caller's.
    /// The two answers open to 4b were to grow a non-top frame or to bind
    /// the exposed name to a slot in the caller's frame; **4b's Task 5 took
    /// the second**, and neither half of it needs a non-top grow:
    ///
    /// * An exposed name is bound by [`alias_slot`], which writes into the
    ///   *callee's* own frame and only reads the target's position. Nothing
    ///   is allocated in the caller at all.
    /// * A name the plan never saw -- a computed `expose (v)` naming a
    ///   symbol that appears in no instruction -- is resolved **before** the
    ///   callee's frame is pushed, while the caller's frame is still the top
    ///   one, so the grow it may need is a top-frame grow. That ordering is
    ///   the reason `PROCEDURE` allocates the callee's frame itself instead
    ///   of `CALL` allocating it in advance.
    ///
    /// So a panic here is not a placeholder awaiting a later relaxation: it
    /// is the check that the ordering above is still being observed. A
    /// silent wrong answer would be a variable landing in another routine's
    /// pool, discovered by chasing a wrong result instead of a message that
    /// already says why.
    ///
    /// [`alias_slot`]: RootSet::alias_slot
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
    ///
    /// **Aliased slots need no filtering either, and that is a property of
    /// how they are written rather than of this loop.** A write through an
    /// alias lands in the target's storage (`set_slot` resolves first), so an
    /// aliasing slot never accumulates values of its own and the exposed
    /// value is yielded exactly once, from the frame that really holds it --
    /// not twice, which would merely be wasted work, and not zero times,
    /// which would collect a live object.
    ///
    /// **An aliasing slot's own entry is not always `None`, though**, and an
    /// earlier version of this paragraph said it was. `alias_slot`'s callers
    /// guarantee the slot holds nothing *reachable*, not that it is empty:
    /// `USE ARG >q.` may alias over a vivified but empty `Body::Stem`, which
    /// this loop then yields. The cost is bounded and is over-retention only
    /// -- one empty stem per such alias, kept alive until the frame pops --
    /// never a collected live object, which is the direction that would
    /// matter.
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
