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
///
/// An upper bound rather than an equality, for the reason `Body`'s carries.
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
}

pub struct Heap {
    slots: Vec<Slot>,
    free_head: Option<u32>,
    live: usize,
    marks: Vec<bool>,
    /// The slots this heap will never sweep, in allocation order.
    ///
    /// **A root the heap holds itself**, which is what "immortal" means here.
    /// The alternative shapes were a slot range of its own, as
    /// [`crate::CLASS_SLOT_BASE`] gives a class identity, and a per-slot flag
    /// the sweeper reads. A separate range would need a second store behind
    /// [`Heap::get`], which is the hottest read in the interpreter and would
    /// pay a branch on every value; a flag would need the marker to trace
    /// these anyway, so that an immortal object's *children* survive with it.
    /// Seeding the mark phase from here does both jobs at once: they are
    /// marked, so the sweeper skips them by the rule it already has, and
    /// whatever they reach is marked with them.
    ///
    /// Nothing removes an entry. That is the contract, not an omission: an
    /// interned constant is reachable from a compiled stream that lives as
    /// long as the program does.
    immortal: Vec<ObjRef>,
    /// How many times `collect` has run, ever. Exists for Task 16's
    /// collect-on-every-allocation gate criterion (4a exit gate, criterion
    /// 4): the mode has to *prove* it collected rather than merely claim to,
    /// and a caller that never sees this move above zero has not tested
    /// what it says it tested. Not reset by anything; a fresh count needs a
    /// fresh `Heap`.
    collections: u64,
}

impl Heap {
    pub fn new() -> Self {
        Heap {
            slots: Vec::new(),
            free_head: None,
            live: 0,
            marks: Vec::new(),
            immortal: Vec::new(),
            collections: 0,
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

    /// Marks from the roots, then sweeps everything unmarked.
    ///
    /// **`rexx-exec` calls this on its own now**, from `Interp::alloc_with`
    /// (`lib.rs`) when the live-object count reaches that module's watermark,
    /// as well as on every allocation under the opt-in stress mode
    /// `run_program_collect_every_alloc` turns on. So every allocation site in
    /// that crate is a collection point, and a value held only in a Rust local
    /// across one is a use-after-free rather than a cost.
    ///
    /// **A missed root does not show as a crash, and the instrument that finds
    /// one is the stress mode.** It collects strictly more often than any
    /// watermark can, so a program that survives it survives the production
    /// trigger. A stale handle *misses* rather than aliasing -- a swept slot's
    /// generation is incremented, which is what the generation is for -- so
    /// the failure is a loud `a live value` or a wrong answer, never a silent
    /// read of another object.
    ///
    /// The one window this crate's own callers used to leave open is closed:
    /// `EXIT`'s result outlives the temps frame that rooted it, all the way to
    /// `exit_code_for`, and `Interp::root_exit_value` is the root that spans
    /// it. Its doc comment carries the measurement.
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
            reached.clear();
            object.body.trace(&mut reached);
            work.extend(reached.iter().copied());
        }

        // Pass 1: clear weak references whose target did not survive.
        //
        // This runs BEFORE the uninit resurrection below, matching the oracle
        // -- MemoryObject::markObjects at RexxMemory.cpp:426-433 calls
        // checkWeakReferences() then checkUninit(), and the comment at :422
        // gives the reason: "so that the uninit list doesn't mark any of the
        // weakly referenced items. We don't want an object placed on the
        // uninit queue to end up strongly referenced later."
        //
        // Swapping these two passes is observable: a weak reference to an
        // unreachable but uninit-pending object reads .nil under this order
        // and reads the live object under the other one.
        for slot in 0..self.slots.len() {
            if !self.marks[slot] {
                continue;
            }
            let Slot::Live { object, .. } = &self.slots[slot] else {
                continue;
            };
            let Body::WeakRef(target) = object.body else {
                continue;
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
        // graph. They are reported, not swept; the caller clears has_uninit
        // once the finalizer has run, and the next collection takes them.
        let mut pending_uninit = Vec::new();
        let mut resurrect: Vec<ObjRef> = Vec::new();
        for slot in 0..self.slots.len() {
            if self.marks[slot] {
                continue;
            }
            let Slot::Live { object, generation } = &self.slots[slot] else {
                continue;
            };
            if object.has_uninit {
                let r = ObjRef::heap(slot as u32, *generation);
                pending_uninit.push(r);
                resurrect.push(r);
            }
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
            reached.clear();
            object.body.trace(&mut reached);
            resurrect.extend(reached.iter().copied());
        }

        let mut swept = 0;
        for slot in 0..self.slots.len() {
            if self.marks[slot] || matches!(self.slots[slot], Slot::Free { .. }) {
                continue;
            }
            let generation = self.slots[slot].generation();
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
        CollectStats {
            swept,
            live: self.live,
            pending_uninit,
        }
    }

    pub fn alloc(&mut self, body: Body) -> ObjRef {
        self.alloc_with_uncollected(BehaviourId::OBJECT, body)
    }

    /// Allocates an object this heap will never sweep.
    ///
    /// For a value that is interned once and read for the rest of the run --
    /// a compiled stream's string constants. **The handle needs no root of the
    /// caller's**, which is the whole point: it can be stored in a structure
    /// the collector does not walk, and handed out for as long as that
    /// structure lives.
    ///
    /// Not a way to avoid thinking about lifetime. Every allocation made here
    /// is held for the life of the `Heap`, so a caller that interned per
    /// *execution* rather than per distinct constant would leak steadily and
    /// the collector could not tell it was happening.
    pub fn alloc_immortal(&mut self, behaviour: BehaviourId, body: Body) -> ObjRef {
        let handle = self.alloc_with_uncollected(behaviour, body);
        self.immortal.push(handle);
        handle
    }

    /// How many objects this heap has interned as immortal.
    ///
    /// The instrument for the paragraph above: a program's count is the number
    /// of *distinct* constants its compiled streams hold, and a count that
    /// tracks executions instead is the leak that would otherwise be invisible.
    pub fn immortal_count(&self) -> usize {
        self.immortal.len()
    }

    /// Allocates without ever collecting, whatever else is enabled.
    ///
    /// Named `_uncollected` rather than plain `alloc_with`, on request, so
    /// that a **new** allocation site written the natural way announces at
    /// the call site that it is bypassing the stress hook, rather than
    /// silently matching the four existing `rexx-exec` sites that already
    /// went through the friendly-named wrapper before this rename existed. A
    /// fifth allocation site added later that keeps calling this name
    /// directly would not fail anything -- the stress mode would just
    /// quietly collect less often than it claims to, which is exactly the
    /// vacuity shape this project keeps finding in its own instruments. The
    /// obviously-correct choice for production code is
    /// `rexx_exec::Interp::alloc_with` (`lib.rs`), which calls this and then
    /// decides whether to collect; this method stays `pub` because
    /// `rexx-core`'s own tests allocate directly with no `Interp` in scope
    /// at all, and forcing every one of those through a heavier entry point
    /// would buy this rule nothing they can bypass just as easily.
    pub fn alloc_with_uncollected(&mut self, behaviour: BehaviourId, body: Body) -> ObjRef {
        let object = Object {
            behaviour,
            body,
            has_uninit: false,
        };
        self.live += 1;
        match self.free_head {
            Some(slot) => {
                let Slot::Free { next, generation } = self.slots[slot as usize] else {
                    unreachable!("the free list only threads free slots")
                };
                self.free_head = next;
                self.slots[slot as usize] = Slot::Live { object, generation };
                ObjRef::heap(slot, generation)
            }
            None => {
                let slot = u32::try_from(self.slots.len()).expect("heap exceeds 2^32 slots");
                // **The arena never reaches the range a class identity lives
                // in**, which is what makes a class handle and a value handle
                // distinguishable at all -- see `CLASS_SLOT_BASE`. Asserted
                // rather than argued from the arithmetic: an arena that grew
                // this far would start handing out handles equal to class
                // identities, and the failure mode is a wrong answer rather
                // than a crash.
                assert!(
                    slot < crate::CLASS_SLOT_BASE,
                    "the arena reached the slot range reserved for class identities"
                );
                self.slots.push(Slot::Live {
                    object,
                    generation: 0,
                });
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

    pub fn get(&self, r: ObjRef) -> Option<&Object> {
        let slot = self.resolve(r)?;
        match &self.slots[slot] {
            Slot::Live { object, .. } => Some(object),
            Slot::Free { .. } => unreachable!("resolve rejects free slots"),
        }
    }

    pub fn get_mut(&mut self, r: ObjRef) -> Option<&mut Object> {
        let slot = self.resolve(r)?;
        match &mut self.slots[slot] {
            Slot::Live { object, .. } => Some(object),
            Slot::Free { .. } => unreachable!("resolve rejects free slots"),
        }
    }

    pub fn live_count(&self) -> usize {
        self.live
    }

    /// Whether the next allocation would **grow** the arena rather than reuse
    /// a slot an earlier collection freed.
    ///
    /// The pressure signal a collector wants, and the reason it is exposed
    /// rather than derived from `live_count` and `slot_capacity`: it is the
    /// branch `alloc_with_uncollected` is about to take, so a caller testing
    /// it is asking the allocator's own question and not reconstructing it.
    ///
    /// **It is this crate's stand-in for the oracle's allocation failure.**
    /// `NormalSegmentSet::handleAllocationFailure`
    /// (`interpreter/memory/MemorySegment.cpp:1135`) collects, then calls
    /// `adjustMemorySize` -- "now that we have good GC data, decide if we
    /// need to adjust the heap size" -- then retries. The oracle can trigger
    /// on failure because it owns its segments; this crate takes each
    /// object's payload from `malloc`, which does not fail, it grows the
    /// process until the OOM killer arrives. There is no failure event here,
    /// and this branch is the moment that means the same thing: the arena is
    /// about to ask for more.
    ///
    /// It answers `true` for a fresh heap, which has no free list yet, so a
    /// caller needs a growth allowance of its own as well -- exactly the
    /// second half of the oracle's shape.
    pub fn will_grow(&self) -> bool {
        self.free_head.is_none()
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
