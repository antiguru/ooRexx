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

//! The arena heap: a vector of slots addressed by generation-checked handles.

use crate::body::{BehaviourId, Body, Object};
use crate::roots::RootSet;
use crate::{Decoded, ObjRef};

/// A slot carries its generation whether occupied or not, so that a handle
/// minted before a sweep cannot read the slot's next occupant.
enum Slot {
    Free { next: Option<u32>, generation: u32 },
    Live { object: Object, generation: u32 },
}

/// **The arena holds one of these per object**, so this is the number a
/// footprint claim is actually about -- `Body`'s own bound in `body.rs` is
/// the term that moves it. Measured at 96 across a change that took
/// `rexx_num::Number` from 32 bytes to 40 without either width shifting.
const _: () = assert!(size_of::<Slot>() <= 96);

impl Slot {
    fn generation(&self) -> u32 {
        match self {
            Slot::Free { generation, .. } | Slot::Live { generation, .. } => *generation,
        }
    }
}

/// What one collection did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CollectStats {
    pub swept: usize,
    pub live: usize,
    /// Objects that were unreachable but define `UNINIT`. They have been kept
    /// alive so the finalizer does not observe a half-collected graph.
    pub pending_uninit: Vec<ObjRef>,
    /// The class objects this collection freed, as they were before their
    /// slots' generations moved on.
    pub freed_classes: Vec<ObjRef>,
    /// The [`Body::held_bytes`] of every object that survived.
    pub live_bytes: usize,
}

pub struct Heap {
    slots: Vec<Slot>,
    free_head: Option<u32>,
    live: usize,
    marks: Vec<bool>,
    /// The slots this heap will never sweep, in allocation order.
    immortal: Vec<ObjRef>,
    /// Every handle [`Heap::set_uninit`] has flagged and no collection has
    /// since found gone or cleared.
    uninit: Vec<ObjRef>,
    /// How many times `collect` has run, ever. Exists for Task 16's
    /// collect-on-every-allocation gate criterion (4a exit gate, criterion
    /// 4): the mode has to *prove* it collected rather than merely claim to,
    /// and a caller that never sees this move above zero has not tested
    /// what it says it tested. Not reset by anything; a fresh count needs a
    /// fresh `Heap`.
    collections: u64,
    /// Body bytes charged by [`Heap::charge_body_bytes`] since the last
    /// collection.
    bytes_since: usize,
    /// The body bytes the last collection found live.
    live_bytes: usize,
    /// The body bytes every live object holds now: what was charged or held
    /// less what was released and what the sweeps freed.
    held_bytes: usize,
    /// The most body bytes held at once as of the last collection.
    peak_bytes: usize,
    #[cfg(feature = "sharing")]
    sharing: Sharing,
}

/// The sharing-fraction instrument (spec 2026-09-29 section 5): each slot's
/// object is tagged with the last activity that resolved it through
/// [`Heap::get`], [`Heap::get_mut`] or [`Heap::body_text`], or else made it.
#[cfg(feature = "sharing")]
#[derive(Default)]
struct Sharing {
    tags: Vec<std::cell::Cell<Tag>>,
    current: std::cell::Cell<u32>,
    /// While above zero, a resolution is not a touch.
    paused: std::cell::Cell<u32>,
    serials: u32,
    /// Whether objects made from here on are the program's.
    program: bool,
    /// Objects made, and of those touched by more than one activity, by
    /// whether the program made them.
    counts: std::cell::Cell<[SharingCount; 2]>,
}

/// One slot's tag.
#[cfg(feature = "sharing")]
#[derive(Copy, Clone)]
struct Tag {
    activity: u32,
    shared: bool,
    program: bool,
}

/// Objects made, and of those touched by more than one activity.
#[cfg(feature = "sharing")]
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct SharingCount {
    pub objects: u64,
    pub shared: u64,
}

impl Heap {
    pub fn new() -> Self {
        Heap {
            slots: Vec::new(),
            free_head: None,
            live: 0,
            marks: Vec::new(),
            immortal: Vec::new(),
            uninit: Vec::new(),
            collections: 0,
            bytes_since: 0,
            live_bytes: 0,
            held_bytes: 0,
            peak_bytes: 0,
            #[cfg(feature = "sharing")]
            sharing: Sharing::default(),
        }
    }

    pub fn slot_capacity(&self) -> usize {
        self.slots.len()
    }

    /// How many times `collect` has run against this heap. See the field's
    /// own doc comment.
    pub fn collections_performed(&self) -> u64 {
        self.collections
    }

    /// Records `bytes` of body allocated outside the slot toward the next
    /// collection, and answers the bytes charged since the last one. What a
    /// live object holds is recorded apart, by [`Heap::hold_body_bytes`].
    #[inline]
    pub fn charge_body_bytes(&mut self, bytes: usize) -> usize {
        self.bytes_since += bytes;
        self.bytes_since
    }

    /// Records `bytes` of body a live object holds outside its slot.
    #[inline]
    pub fn hold_body_bytes(&mut self, bytes: usize) {
        self.held_bytes += bytes;
    }

    /// Records that a live object no longer holds `bytes` of body outside
    /// its slot.
    #[inline]
    pub fn release_body_bytes(&mut self, bytes: usize) {
        debug_assert!(
            bytes <= self.held_bytes,
            "{bytes} body bytes released of {} held",
            self.held_bytes
        );
        self.held_bytes = self.held_bytes.saturating_sub(bytes);
    }

    /// Records that a live object's body went from holding `before` bytes
    /// outside its slot to holding `after`.
    #[inline]
    pub fn rehold_body_bytes(&mut self, before: usize, after: usize) {
        if after >= before {
            self.hold_body_bytes(after - before);
        } else {
            self.release_body_bytes(before - after);
        }
    }

    /// The body bytes charged since the last collection.
    pub fn bytes_since(&self) -> usize {
        self.bytes_since
    }

    /// The most body bytes this heap has held at once: the bytes the last
    /// collection found live plus everything charged since, at its largest.
    /// An upper bound, since a body charged and dropped before a collection
    /// still counts.
    pub fn peak_body_bytes(&self) -> usize {
        self.peak_bytes.max(self.live_bytes + self.bytes_since)
    }

    /// Marks from the roots, then sweeps everything unmarked.
    pub fn collect(&mut self, roots: &RootSet) -> CollectStats {
        self.collections += 1;
        self.marks.clear();
        // Resized every time: the heap grows between collections.
        self.marks.resize(self.slots.len(), false);

        // The caller's roots and this heap's own. See the `immortal` field for
        // why an interned constant is a root here rather than a slot range or
        // a flag.
        let mut work: Vec<ObjRef> = roots.iter().chain(self.immortal.iter().copied()).collect();
        let mut reached = Vec::new();
        // The marked weak references, gathered here rather than by a walk of
        // the arena afterwards. The mark loop visits each marked slot exactly
        // once -- that is what the `replace` below guarantees -- so this ends
        // up holding precisely the slots such a walk would have selected, at
        // the cost of one discriminant test per marked object instead of a
        // read of every slot in the table. Whether a *target* survived cannot
        // be decided here, so the decision waits for the loop to finish.
        let mut weak_marked: Vec<u32> = Vec::new();
        // The survivors' body bytes, summed only to check the running figure
        // the sweep leaves in `held_bytes`.
        #[cfg(debug_assertions)]
        let mut survivor_bytes = 0;
        while let Some(r) = work.pop() {
            let Some(slot) = self.resolve(r) else {
                continue;
            };
            if std::mem::replace(&mut self.marks[slot], true) {
                continue; // already marked: this is what terminates cycles
            }
            let Slot::Live { object, .. } = &self.slots[slot] else {
                unreachable!("resolve rejects free slots")
            };
            if matches!(object.body, Body::WeakRef(_)) {
                weak_marked.push(slot as u32);
            }
            #[cfg(debug_assertions)]
            {
                survivor_bytes += object.body.held_bytes();
            }
            reached.clear();
            object.body.trace(&mut reached);
            work.extend(reached.iter().copied());
        }

        // Pass 1: clear weak references whose target did not survive.
        for slot in weak_marked {
            let slot = slot as usize;
            let Slot::Live { object, .. } = &self.slots[slot] else {
                unreachable!("the mark loop only records live slots")
            };
            let Body::WeakRef(target) = object.body else {
                unreachable!("the mark loop only records weak references")
            };
            // "Dead" includes unresolvable: a target whose slot was already
            // freed, or whose generation has moved on, died in an earlier
            // cycle and its reference must still clear.
            let target_alive = self.resolve(target).is_some_and(|t| self.marks[t]);
            if !target_alive {
                let Slot::Live { object, .. } = &mut self.slots[slot] else {
                    unreachable!()
                };
                object.body = Body::WeakRef(ObjRef::NIL);
            }
        }

        // Pass 2: resurrect unreachable objects that define UNINIT, marking
        // everything they reach so the finalizer never sees a half-collected
        // graph. They are reported, not swept; the caller clears the flag
        // once the finalizer has run, and the next collection takes them.
        let mut pending_uninit = Vec::new();
        let mut resurrect: Vec<ObjRef> = Vec::new();
        if !self.uninit.is_empty() {
            let mut registry = std::mem::take(&mut self.uninit);
            registry.retain(|&r| {
                let Some(slot) = self.resolve(r) else {
                    return false;
                };
                let Slot::Live { object, .. } = &self.slots[slot] else {
                    unreachable!("resolve rejects free slots")
                };
                if !object.has_uninit {
                    return false;
                }
                if !self.marks[slot] {
                    // Resurrected on every collection that finds it
                    // unreachable, because the flag is what keeps it alive;
                    // reported once, which is `setReadyForUninit`.
                    if !object.ready_for_uninit {
                        pending_uninit.push(r);
                    }
                    resurrect.push(r);
                }
                true
            });
            self.uninit = registry;
        }
        for &r in &pending_uninit {
            let Some(slot) = self.resolve(r) else {
                continue;
            };
            let Slot::Live { object, .. } = &mut self.slots[slot] else {
                unreachable!("resolve rejects free slots")
            };
            object.ready_for_uninit = true;
        }
        while let Some(r) = resurrect.pop() {
            let Some(slot) = self.resolve(r) else {
                continue;
            };
            if std::mem::replace(&mut self.marks[slot], true) {
                continue;
            }
            let Slot::Live { object, .. } = &self.slots[slot] else {
                unreachable!("resolve rejects free slots")
            };
            #[cfg(debug_assertions)]
            {
                survivor_bytes += object.body.held_bytes();
            }
            reached.clear();
            object.body.trace(&mut reached);
            resurrect.extend(reached.iter().copied());
        }

        let mut swept = 0;
        let mut freed_bytes = 0;
        let mut freed_classes = Vec::new();
        for slot in 0..self.slots.len() {
            if self.marks[slot] {
                continue;
            }
            let Slot::Live { object, generation } = &self.slots[slot] else {
                continue;
            };
            let generation = *generation;
            match &object.body {
                // Reported so the caller can drop the rows it keys by this
                // class. The handle is still the live one here; after the
                // assignment below its generation has moved on and it would
                // name nothing.
                Body::Class { .. } => freed_classes.push(ObjRef::heap(slot as u32, generation)),
                body => freed_bytes += body.held_bytes(),
            }
            swept += 1;
            self.live -= 1;
            // A slot whose generation would overflow is retired, not reused:
            // wrapping would let a stale handle alias a live object again,
            // which is the whole reason the generation exists.
            self.slots[slot] = match generation.checked_add(1) {
                Some(next) if next <= crate::GENERATION_MAX => {
                    let free = Slot::Free {
                        next: self.free_head,
                        generation: next,
                    };
                    self.free_head = Some(slot as u32);
                    free
                }
                _ => Slot::Free {
                    next: None,
                    generation,
                },
            };
        }
        self.release_body_bytes(freed_bytes);
        #[cfg(debug_assertions)]
        assert_eq!(
            survivor_bytes, self.held_bytes,
            "the survivors hold {survivor_bytes} body bytes, the running figure says {}",
            self.held_bytes
        );
        self.peak_bytes = self.peak_body_bytes();
        self.live_bytes = self.held_bytes;
        self.bytes_since = 0;
        CollectStats {
            swept,
            live: self.live,
            pending_uninit,
            freed_classes,
            live_bytes: self.held_bytes,
        }
    }

    /// Allocates `body` and holds the bytes it holds outside its slot.
    pub fn alloc(&mut self, body: Body) -> ObjRef {
        self.hold_body_bytes(body.held_bytes());
        self.alloc_with_uncollected(BehaviourId::OBJECT, body)
    }

    /// Allocates an object this heap will never sweep.
    pub fn mint_class(&mut self) -> ObjRef {
        self.alloc_immortal(BehaviourId::OBJECT, Body::Class { owned: Vec::new() })
    }

    /// Allocates an object this heap will never sweep, and holds the bytes
    /// its body holds outside its slot.
    pub fn alloc_immortal(&mut self, behaviour: BehaviourId, body: Body) -> ObjRef {
        self.hold_body_bytes(body.held_bytes());
        let handle = self.alloc_with_uncollected(behaviour, body);
        self.immortal.push(handle);
        handle
    }

    /// Records that `r`'s object defines `UNINIT`, so the collector
    /// resurrects and reports it rather than sweeping it. Answers whether the
    /// handle named a live object.
    pub fn set_uninit(&mut self, r: ObjRef) -> bool {
        let Some(slot) = self.resolve(r) else {
            return false;
        };
        let Slot::Live { object, .. } = &mut self.slots[slot] else {
            unreachable!("resolve rejects free slots")
        };
        if !object.has_uninit {
            object.has_uninit = true;
            self.uninit.push(r);
        }
        true
    }

    /// Every handle still flagged, oldest first, with the flag cleared and
    /// the registry emptied, for a caller running every pending finalizer.
    pub fn take_uninit_flagged(&mut self) -> Vec<ObjRef> {
        let registry = std::mem::take(&mut self.uninit);
        let mut flagged = Vec::new();
        for r in registry {
            let Some(slot) = self.resolve(r) else {
                continue;
            };
            let Slot::Live { object, .. } = &mut self.slots[slot] else {
                unreachable!("resolve rejects free slots")
            };
            if object.has_uninit {
                object.has_uninit = false;
                object.ready_for_uninit = false;
                flagged.push(r);
            }
        }
        flagged
    }

    /// Undoes [`Heap::set_uninit`] for a batch, for the caller reporting that
    /// their finalizers have run. The next collection sweeps them like any
    /// other object.
    pub fn clear_uninit_all(&mut self, objects: &[ObjRef]) {
        for &r in objects {
            let Some(slot) = self.resolve(r) else {
                continue;
            };
            let Slot::Live { object, .. } = &mut self.slots[slot] else {
                unreachable!("resolve rejects free slots")
            };
            object.has_uninit = false;
            object.ready_for_uninit = false;
        }
        let registry = std::mem::take(&mut self.uninit);
        self.uninit = registry
            .into_iter()
            .filter(|&r| {
                self.resolve(r).is_some_and(|slot| {
                    matches!(&self.slots[slot], Slot::Live { object, .. } if object.has_uninit)
                })
            })
            .collect();
    }

    /// How many objects this heap has interned as immortal.
    pub fn immortal_count(&self) -> usize {
        self.immortal.len()
    }

    /// Allocates without ever collecting, whatever else is enabled. The
    /// caller holds the bytes the body holds outside its slot
    /// ([`Heap::hold_body_bytes`]).
    #[inline]
    pub fn alloc_with_uncollected(&mut self, behaviour: BehaviourId, body: Body) -> ObjRef {
        self.live += 1;
        // **`Object` is built inside each arm rather than once above the
        // match**, so that the body lands in the slot it will live in instead
        // of being written to a stack temporary and copied. A slot is wide --
        // see this module's `size_of::<Slot>()` assertion -- and the copy is
        // its full width. Measured on the allocating benchmark axes.
        match self.free_head {
            Some(slot) => {
                let Slot::Free { next, generation } = self.slots[slot as usize] else {
                    unreachable!("the free list only threads free slots")
                };
                self.free_head = next;
                self.slots[slot as usize] = Slot::Live {
                    object: Object {
                        behaviour,
                        body,
                        has_uninit: false,
                        ready_for_uninit: false,
                        watched: false,
                    },
                    generation,
                };
                #[cfg(feature = "sharing")]
                self.made(slot);
                ObjRef::heap(slot, generation)
            }
            None => {
                let slot = u32::try_from(self.slots.len()).expect("heap exceeds 2^32 slots");
                self.slots.push(Slot::Live {
                    object: Object {
                        behaviour,
                        body,
                        has_uninit: false,
                        ready_for_uninit: false,
                        watched: false,
                    },
                    generation: 0,
                });
                #[cfg(feature = "sharing")]
                self.made(slot);
                ObjRef::heap(slot, 0)
            }
        }
    }

    /// Resolves a handle, or `None` if it names no slot, a free slot, or a
    /// slot whose generation has moved on.
    fn resolve(&self, r: ObjRef) -> Option<usize> {
        let Decoded::Heap { slot, generation } = r.decode() else {
            return None;
        };
        let entry = self.slots.get(slot as usize)?;
        (entry.generation() == generation && matches!(entry, Slot::Live { .. }))
            .then_some(slot as usize)
    }

    /// A live `Body::Text`'s bytes, and `None` for every other slot state and
    /// every other body -- one indexing and one match, where [`Heap::get`]
    /// followed by a body match is two of each. For a caller that reads the
    /// same handle many times over.
    pub fn body_text(&self, r: ObjRef) -> Option<&[u8]> {
        let Decoded::Heap { slot, generation } = r.decode() else {
            return None;
        };
        match self.slots.get(slot as usize)? {
            Slot::Live {
                object:
                    Object {
                        body: Body::Text { bytes, .. },
                        ..
                    },
                generation: live,
            } if *live == generation => {
                #[cfg(feature = "sharing")]
                self.resolved(slot as usize);
                Some(bytes.as_slice())
            }
            _ => None,
        }
    }

    pub fn get(&self, r: ObjRef) -> Option<&Object> {
        let slot = self.resolve(r)?;
        #[cfg(feature = "sharing")]
        self.resolved(slot);
        match &self.slots[slot] {
            Slot::Live { object, .. } => Some(object),
            Slot::Free { .. } => unreachable!("resolve rejects free slots"),
        }
    }

    /// [`Heap::get`] for a walk of the interpreter's own tables rather than
    /// a program's resolution: the sharing instrument does not count it.
    pub fn peek(&self, r: ObjRef) -> Option<&Object> {
        let slot = self.resolve(r)?;
        match &self.slots[slot] {
            Slot::Live { object, .. } => Some(object),
            Slot::Free { .. } => unreachable!("resolve rejects free slots"),
        }
    }

    /// [`Heap::get_mut`] for a walk of the interpreter's own tables, which
    /// the sharing instrument does not count, as [`Heap::peek`].
    pub fn peek_mut(&mut self, r: ObjRef) -> Option<&mut Object> {
        let slot = self.resolve(r)?;
        match &mut self.slots[slot] {
            Slot::Live { object, .. } => Some(object),
            Slot::Free { .. } => unreachable!("resolve rejects free slots"),
        }
    }

    /// The live object `r` names, for a caller that writes it. A caller that
    /// changes what the body holds outside its slot ([`Body::held_bytes`]),
    /// by growing, shrinking or replacing it, records the change with
    /// [`Heap::hold_body_bytes`], [`Heap::release_body_bytes`] or
    /// [`Heap::rehold_body_bytes`] before any step that can fail. The same
    /// holds for [`Heap::peek_mut`].
    pub fn get_mut(&mut self, r: ObjRef) -> Option<&mut Object> {
        let slot = self.resolve(r)?;
        #[cfg(feature = "sharing")]
        self.resolved(slot);
        match &mut self.slots[slot] {
            Slot::Live { object, .. } => Some(object),
            Slot::Free { .. } => unreachable!("resolve rejects free slots"),
        }
    }

    /// Whether `r` names a class object.
    pub fn is_class(&self, r: ObjRef) -> bool {
        matches!(
            self.get(r).map(|object| &object.body),
            Some(Body::Class { .. })
        )
    }

    pub fn live_count(&self) -> usize {
        self.live
    }

    /// Whether the next allocation would **grow** the arena rather than reuse
    /// a slot an earlier collection freed.
    pub fn will_grow(&self) -> bool {
        self.free_head.is_none()
    }

    /// A tag for a new activity, distinct from every earlier one and from
    /// the tag a new heap starts with.
    #[cfg(feature = "sharing")]
    pub fn sharing_tag(&mut self) -> u32 {
        self.sharing.serials += 1;
        self.sharing.serials
    }

    /// Resolves and makes objects as `tag` from here on.
    #[cfg(feature = "sharing")]
    pub fn share_as(&self, tag: u32) {
        self.sharing.current.set(tag);
    }

    /// Counts objects made from here on as the program's.
    #[cfg(feature = "sharing")]
    pub fn sharing_program_starts(&mut self) {
        self.sharing.program = true;
    }

    /// Stops counting resolutions as touches, for reads the program does not
    /// make, until a matching `sharing_pause(false)`; pauses nest.
    #[cfg(feature = "sharing")]
    pub fn sharing_pause(&self, paused: bool) {
        let depth = self.sharing.paused.get();
        self.sharing
            .paused
            .set(if paused { depth + 1 } else { depth - 1 });
    }

    /// The counts for objects made before [`Heap::sharing_program_starts`],
    /// and for those made after it.
    #[cfg(feature = "sharing")]
    pub fn sharing_counts(&self) -> [SharingCount; 2] {
        self.sharing.counts.get()
    }

    #[cfg(feature = "sharing")]
    fn made(&mut self, slot: u32) {
        let program = self.sharing.program;
        let tag = Tag {
            activity: self.sharing.current.get(),
            shared: false,
            program,
        };
        let slot = slot as usize;
        if slot == self.sharing.tags.len() {
            self.sharing.tags.push(std::cell::Cell::new(tag));
        } else {
            self.sharing.tags[slot].set(tag);
        }
        let mut counts = self.sharing.counts.get();
        counts[usize::from(program)].objects += 1;
        self.sharing.counts.set(counts);
    }

    #[cfg(feature = "sharing")]
    fn resolved(&self, slot: usize) {
        let current = self.sharing.current.get();
        let cell = &self.sharing.tags[slot];
        let tag = cell.get();
        if tag.activity != current && self.sharing.paused.get() == 0 {
            if !tag.shared {
                let mut counts = self.sharing.counts.get();
                counts[usize::from(tag.program)].shared += 1;
                self.sharing.counts.set(counts);
            }
            cell.set(Tag {
                activity: current,
                shared: true,
                ..tag
            });
        }
    }
}

impl Default for Heap {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod retire_tests {
    //! The retirement branch cannot be reached from the public API -- no test
    //! can allocate 2^30 times -- so it is exercised from inside the module,
    //! where the slot's generation can be forced directly.
    use super::*;
    use crate::bytes::Bytes;
    use crate::{Decoded, GENERATION_MAX, RootSet};

    #[test]
    fn a_slot_at_generation_max_is_retired_not_reused() {
        let mut heap = Heap::new();
        let roots = RootSet::new();
        let r = heap.alloc(Body::Text {
            bytes: Bytes::from_slice(b"old"),
            num: None,
        });
        let Decoded::Heap { slot, .. } = r.decode() else {
            panic!("heap handle")
        };
        if let Slot::Live { generation, .. } = &mut heap.slots[slot as usize] {
            *generation = GENERATION_MAX;
        }
        let stale = ObjRef::heap(slot, GENERATION_MAX);
        heap.collect(&roots);
        let next = heap.alloc(Body::Text {
            bytes: Bytes::from_slice(b"new"),
            num: None,
        });
        assert_eq!(
            heap.slot_capacity(),
            2,
            "the retired slot must not be reused"
        );
        assert!(heap.get(stale).is_none(), "the stale handle still misses");
        assert!(heap.get(next).is_some());
    }
}

#[cfg(test)]
mod body_text_tests {
    //! [`Heap::body_text`] is a shortcut through [`Heap::get`] and a body
    //! match, so every case is asserted against that pair rather than against
    //! a written-down expectation.
    use super::*;
    use crate::RootSet;
    use crate::bytes::Bytes;

    /// What the shortcut is a shortcut *for*, spelled out.
    fn the_long_way(heap: &Heap, r: ObjRef) -> Option<&[u8]> {
        match heap.get(r).map(|object| &object.body) {
            Some(Body::Text { bytes, .. }) => Some(bytes.as_slice()),
            _ => None,
        }
    }

    #[test]
    fn it_answers_what_get_and_a_body_match_answer() {
        let mut heap = Heap::new();
        let text = heap.alloc(Body::Text {
            bytes: Bytes::from_slice(b"a parse source"),
            num: None,
        });
        let other = heap.alloc(Body::WeakRef(ObjRef::NIL));
        for candidate in [text, other, ObjRef::NIL, ObjRef::inline_byte(b'x')] {
            assert_eq!(
                heap.body_text(candidate),
                the_long_way(&heap, candidate),
                "{candidate:?}"
            );
        }
        assert_eq!(heap.body_text(text), Some(&b"a parse source"[..]));
        assert_eq!(heap.body_text(other), None);

        // A swept slot **taken by a live `Body::Text` again**, which is the
        // state an unrooted parse source reaches and the one the shortcut has
        // to answer `None` for rather than the next occupant's bytes. Every
        // freed slot is refilled, because the free list hands them back in an
        // order this test does not get to choose.
        let capacity = heap.slot_capacity();
        heap.collect(&RootSet::new());
        let occupants: Vec<ObjRef> = (0..capacity)
            .map(|_| {
                heap.alloc(Body::Text {
                    bytes: Bytes::from_slice(b"the next occupant"),
                    num: None,
                })
            })
            .collect();
        assert_eq!(heap.slot_capacity(), capacity, "the slots were not reused");
        for occupant in occupants {
            assert_eq!(heap.body_text(occupant), Some(&b"the next occupant"[..]));
        }
        assert_eq!(heap.body_text(text), None, "a swept handle");
        assert_eq!(heap.body_text(text), the_long_way(&heap, text));
    }
}

#[cfg(test)]
mod body_bytes_tests {
    use super::*;
    use crate::RootSet;
    use crate::bytes::Bytes;

    fn text(heap: &mut Heap, len: usize) -> ObjRef {
        heap.charge_body_bytes(len);
        heap.alloc(Body::Text {
            bytes: Bytes::from_slice(&vec![b'x'; len]),
            num: None,
        })
    }

    /// The survivors' bytes include a resurrected `UNINIT` object's, and an
    /// inline body holds no bytes.
    #[test]
    fn a_collection_sums_the_survivors_body_bytes() {
        let mut heap = Heap::new();
        let mut roots = RootSet::new();
        let kept = text(&mut heap, 100);
        let _dead = text(&mut heap, 200);
        let finalized = text(&mut heap, 300);
        let inline = heap.alloc(Body::Text {
            bytes: Bytes::from_slice(b"short"),
            num: None,
        });
        roots.add_global("kept", kept);
        roots.add_global("inline", inline);
        assert!(heap.set_uninit(finalized));
        let stats = heap.collect(&roots);
        assert_eq!(stats.live_bytes, 400);
        assert_eq!(heap.peak_body_bytes(), 600);
        heap.charge_body_bytes(50);
        assert_eq!(heap.peak_body_bytes(), 600);
        heap.charge_body_bytes(500);
        assert_eq!(heap.peak_body_bytes(), 950);
    }
}
