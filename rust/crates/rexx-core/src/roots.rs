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
///
/// A newtype rather than a bare `usize` because the two are not
/// interchangeable at a call site: every other index in this file is
/// relative to a `SlotFrame`, and an absolute one passed where a relative
/// one belongs addresses a real slot in some other activation's range
/// rather than failing. Only [`RootSet::slot_ref`] and [`RootSet::promote`]
/// produce one, and both chase before returning, so a `SlotRef` is by
/// construction a final destination and never itself an alias.
///
/// **It addresses either the frame arena or a cell**, and a cell is the
/// storage a variable is moved into when a `>name` reference is taken to it:
/// cells are never truncated, so such a reference stays valid after the
/// frame that declared the variable is gone.
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
    /// so resolution here follows exactly one link **except across a
    /// [`RootSet::promote`]**, which redirects an already-aliased position to
    /// a cell after other slots have been bound to it; `resolve` therefore
    /// chases to a fixed point. Every link but the last addresses a lower
    /// position, so the walk terminates.
    aliases: Vec<Option<usize>>,
    /// How many entries of `aliases` are `Some`.
    ///
    /// **`resolve` runs on every variable read and every write, and this is
    /// what lets the common program skip the parallel vector entirely.** An
    /// alias exists only where `PROCEDURE EXPOSE` or `USE ARG >name` bound
    /// one, so a program using neither -- which is most of them -- never has
    /// a live entry here, and reading `aliases[position]` would be a second
    /// indexed load from a second vector, on a different cache line from the
    /// `slots` entry it guards. Measured by removing the load outright:
    /// `bench-programs/varlookup.rex` -3.83%, `compound.rex` -2.90%,
    /// `emptyloop.rex` -2.24%, a fixed-work rexxcps -0.98%.
    alias_count: usize,
    /// Storage for variables a `>name` reference has been taken to, outside
    /// every frame and never truncated.
    ///
    /// A reference is an ordinary value and may outlive the activation whose
    /// variable it names -- measured on the oracle, a `procedure` returning
    /// `>v` answers a reference whose `~value` still reads and writes after
    /// the return. So the storage moves here at the moment the reference is
    /// taken ([`RootSet::promote`]) and the frame slot becomes an alias for
    /// it, which keeps the variable and every reference to it on one cell.
    ///
    /// **Nothing frees a cell**, so a program taking a reference to a fresh
    /// local on each of many calls grows this vector without bound.
    cells: Vec<Option<ObjRef>>,
    /// The starting offset of every currently pushed frame, in push order.
    /// Its length is also every live frame's `depth` plus one, which is how
    /// `grow_slots` and `pop_slots` recognise the top frame.
    frame_starts: Vec<usize>,
    /// Values an activation that is **not** on any stack still owns.
    ///
    /// A slot frame nests: `pop_slots` asserts that the frame it closes is the
    /// top one, and the arena behind it is one contiguous `Vec`. So an
    /// activation whose body is to continue later cannot keep its frame open
    /// while the activations above it close -- its values are copied out, the
    /// frame is released, and this is where they stay reachable in the
    /// meantime. Each entry is one parked activation's whole set; `None` is an
    /// entry whose owner has resumed, and [`RootSet::park`] reuses it.
    parked: Vec<Option<Vec<ObjRef>>>,
    /// Indices of `parked` that are `None`, so a park after a release reuses
    /// an entry instead of extending the vector for the life of the process.
    parked_free: Vec<usize>,
}

/// A handle to one parked set of values, issued by [`RootSet::park`] and spent
/// by [`RootSet::release`].
///
/// A newtype rather than a bare `usize` for [`SlotRef`]'s reason: every other
/// index in this file addresses a slot, and one passed where a slot index
/// belongs would read a real slot rather than failing.
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
    ///
    /// **SCHEDULING, and a Phase 6 reader of this file may delete the whole
    /// parked group** -- this, [`RootSet::release`], [`RootSet::live_parked`]
    /// and [`RootSet::frame_aliases`], with `parked` and `parked_free`. Its
    /// only caller is `rexx-exec`'s `Interp::park_reply`, which exists because
    /// a `REPLY` in a single-activity interpreter has to suspend the body it
    /// left owed and a suspended activation cannot hold a slot frame open. A
    /// design where the body continues on another activity, with its frame
    /// still open, needs none of it.
    ///
    /// The caller keeps its own copy of whatever it needs to restore; this is
    /// reachability and nothing else, which is why it takes a flat vector
    /// rather than a shape. A caller with values in several places hands over
    /// the union of them.
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
    ///
    /// Panics on a handle already spent: a double release would put one index
    /// on the free list twice, and the second park to draw it would hand two
    /// owners the same entry.
    pub fn release(&mut self, parked: Parked) {
        assert!(
            self.parked[parked.0].take().is_some(),
            "release on a parked entry that was already released"
        );
        self.parked_free.push(parked.0);
    }

    /// How many parked entries are currently rooting anything.
    ///
    /// For asserting that a park is matched by a release, exactly as
    /// [`RootSet::live_frames`] is for a frame. Not a capacity or a budget.
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
    ///
    /// **Truncating to a watermark, rather than popping one frame, is
    /// load-bearing and callers rely on it.** `rexx-exec` opens a frame and
    /// then evaluates through `?`, so a raised condition leaves that frame's
    /// own `pop_frame` unreached; every such frame is healed here, because
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
    /// **Do not add a balance assertion here** without first making every such
    /// site pop on its own path. It would fire on the ordinary error
    /// path of a correct program. That is also why the slot side of this file
    /// asserts and this side deliberately does not; the asymmetry is a
    /// decision, not an oversight.
    pub fn pop_frame(&mut self, frame: FrameId) {
        self.temps.truncate(frame.0);
    }

    pub fn push_temp(&mut self, value: ObjRef) {
        self.temps.push(value);
    }

    /// Opens a region of `count` indexable temporaries, all [`ObjRef::NIL`],
    /// and returns the watermark below them.
    ///
    /// An IR chunk's register file. Registers are roots the
    /// collector must walk, and the two other places to put them both fail:
    /// a `SlotFrame` of their own makes `grow_slots` on the frame beneath
    /// panic (its assertion, deliberately not a placeholder), and extending
    /// the activation's own `push_slots` request reaches neither `CALL
    /// label` nor `INTERPRET` nor a trap handler, none of which push a frame
    /// at all.
    ///
    /// The temporaries stack has neither problem: it is independent of slot
    /// frames, so a variable growing underneath is not its business, and it
    /// nests, which is what a fragment chunk compiled and run inside a
    /// running chunk needs.
    pub fn reserve_temps(&mut self, count: usize) -> FrameId {
        let frame = FrameId(self.temps.len());
        self.temps.resize(self.temps.len() + count, ObjRef::NIL);
        frame
    }

    /// Reads register `index` of the region opened at `frame`.
    ///
    /// **The `assert!` is not a second check and it is not belt and braces.**
    /// It states the same condition the indexing below states, so LLVM keeps
    /// one comparison and drops `Index`'s -- and with it the call to
    /// `panic_bounds_check`, which is the expensive half, because that
    /// function takes the index, the length and a `Location` and so pins all
    /// three live across the access. An `assert!` reaches a cold path that
    /// takes nothing. Counted in the binary: the six asserts in this file
    /// remove 58 `panic_bounds_check` call sites, 527 to 469.
    ///
    /// Measured as retired instructions, interleaved, all six together:
    /// `varlookup` -5.2%, `rexxcps` -1.6%, `emptyloop` -1.1%, `arith` -0.8%.
    /// Replacing the indexing with `get_unchecked` instead buys a further
    /// 2.4% on `varlookup` and is the whole of what the remaining comparison
    /// costs -- not taken, because the comparison is what makes a compiler
    /// defect a panic here rather than a silent read of another frame.
    ///
    /// **No message, and that is measured too.** A formatted one
    /// (`"register {index} is outside .."`) puts the values back where
    /// `panic_bounds_check` had them and builds a `fmt::Arguments` besides:
    /// `varlookup` +11.1% and `rexxcps` +4.8% against this tree, far worse
    /// than the checks it was meant to replace. Even a `&'static str` costs
    /// 2 instructions per `emptyloop` iteration. The stringified condition
    /// an argument-less `assert!` panics with names the invariant anyway.
    pub fn temp_at(&self, frame: FrameId, index: usize) -> ObjRef {
        assert!(frame.0 + index < self.temps.len());
        self.temps[frame.0 + index]
    }

    /// Writes register `index` of the region opened at `frame`.
    ///
    /// The `assert!` is [`RootSet::temp_at`]'s, for its reason.
    pub fn set_temp(&mut self, frame: FrameId, index: usize, value: ObjRef) {
        assert!(frame.0 + index < self.temps.len());
        self.temps[frame.0 + index] = value;
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

    /// How many of `frame`'s slots are aliases for storage somewhere else.
    ///
    /// For asserting that a frame's contents can be *copied* -- which is what
    /// `rexx-exec` parks a suspended activation's variables with, so this is
    /// SCHEDULING with the rest of the parked group ([`RootSet::park`] names
    /// it). Copying an
    /// alias's value out and writing it back as a plain slot would silently
    /// break the sharing, so a caller that copies has to know there is none.
    ///
    /// **A redirect to a cell does not count**, because [`RootSet::park`]'s
    /// caller saves and restores those with
    /// [`RootSet::take_frame_aliases`]; a cell outlives every frame, so
    /// putting the redirect back rebuilds the sharing a copy would break.
    ///
    /// Not a capacity or a budget, like [`RootSet::live_frames`] beside it.
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
    /// the frames were pushed. [`RootSet::promote`] is the one operation that
    /// redirects a position other slots may already name, which is why
    /// `resolve` walks rather than steps.
    pub fn slot_ref(&self, frame: SlotFrame, index: usize) -> SlotRef {
        SlotRef(self.resolve(frame, index))
    }

    /// Moves slot `index` of `frame` into a cell and answers that cell, so
    /// that a reference to the variable survives the frame.
    ///
    /// Idempotent: a variable already living in a cell answers the same one,
    /// which is what keeps two references to one variable sharing storage.
    /// The redirect is written at the *resolved* position, so an exposed name
    /// promotes the storage it was exposed from rather than its own slot.
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
    ///
    /// A [`RootSet::promote`] redirect is the reason this exists: it is the
    /// one binding whose target survives `pop_slots`, so putting it back is
    /// what keeps the variable and the references to it on one cell across a
    /// park.
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
        //
        // The two directions of drift are not symmetric. Above the truth
        // costs only the load this was meant to save. Below it stops
        // honouring a live alias, which `corpus/lang/use_arg_forms.rex`
        // catches against the oracle -- measured, dropping `alias_slot`'s
        // increment makes that program print the aliased variable's own
        // value where the oracle prints what was written through the alias.
        // The assertion is here because it names the slot at the read that
        // relied on the claim, rather than leaving a line of output to
        // disagree somewhere later.
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
    ///
    /// **Split out so that the three frame accessors can address `slots`
    /// directly when nothing is aliased.** Their storage test -- frame arena
    /// or cell -- is on the answer this returns, and a program with neither
    /// `PROCEDURE EXPOSE` nor `>name` must not pay it: see `alias_count` for
    /// what one avoided load on this path is worth.
    ///
    /// A loop and not `unwrap_or`, for the one case [`SlotRef`]'s induction
    /// does not cover: [`RootSet::promote`] redirects a position other slots
    /// may already be bound to. Every slot-to-slot link addresses a lower
    /// position and a cell is terminal, so this walk ends.
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
    ///
    /// **Frame storage, which is not the same thing as a Rexx variable**, and
    /// the three accessors here say `frame` in their names for that reason. A
    /// name an `EXPOSE` bound to a scope pool on the receiving object lives in
    /// that object's `Body::Instance` and never in a slot, so a caller that
    /// wants the variable rather than the slot goes through `rexx-exec`'s own
    /// `Interp::variable`/`set_variable`/`clear_variable`, which decide
    /// between the two. Asking this function for such a name answers `None`
    /// for as long as the binding lasts -- an uninitialised variable, which
    /// is a wrong answer nothing announces. The names are what makes a caller
    /// choose.
    /// `always` for [`ObjRef::decode`]'s measured reason: a plain
    /// `#[inline]` leaves the symbol standing. This accessor and the write
    /// below are together -3.48% on `bench-programs/emptyloop.rex` and
    /// -2.65% on `varlookup.rex`.
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
    ///
    /// **A separate operation rather than `set_frame_slot` taking an
    /// `Option<ObjRef>`**, and the choice is about call sites rather than
    /// about this file. `DROP` is a construct the language has, so a caller
    /// that spells it `clear_frame_slot(frame, i)` says what it means, while
    /// `set_frame_slot(frame, i, None)` reads at a glance like a caller that
    /// forgot to compute a value. The read side already carries the `Option`
    /// shape, since `frame_slot` returns one, so nothing is hidden by keeping
    /// the common write monomorphic, and the alternative would be a
    /// mechanical `Some(...)` wrap at every write in the crate for no gain.
    ///
    /// The other half of the operation is that a cleared slot **stops being a
    /// root**, which `iter` gets right because it filters on the `Option` and
    /// this writes `None` rather than some in-band marker. That is the half
    /// worth stating: a clearing operation that only changed what `frame_slot`
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
    pub fn clear_frame_slot(&mut self, frame: SlotFrame, index: usize) {
        let position = self.resolve(frame, index);
        self.write(position, None);
    }

    /// Reads the storage at a tagged position: the frame arena, or a cell.
    ///
    /// Reached only where an alias may be in force -- see
    /// [`RootSet::resolve_aliased`] for why the accessors above do not.
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
    /// alias lands in the target's storage (`set_frame_slot` resolves
    /// first), so an aliasing slot never accumulates values of its own and
    /// the exposed value is yielded exactly once, from the frame that really
    /// holds it --
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
            .chain(self.cells.iter().filter_map(|c| *c))
            .chain(self.parked.iter().flatten().flatten().copied())
    }
}

impl Default for RootSet {
    fn default() -> Self {
        Self::new()
    }
}
