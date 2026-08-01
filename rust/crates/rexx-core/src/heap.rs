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
    /// **Until Task 16, nothing in the interpreter called this at all, and
    /// one known under-rooted window was waiting for the day something
    /// did.** `alloc_with_uncollected` never collects on its own, so
    /// `rexx-exec` has a value that lives past the temps frame that rooted
    /// it: `EXIT`'s result, from the wrapper's `pop_frame` through to
    /// `exit_code_for`. It is written up at that site in `run.rs`, and the
    /// fix is a root that outlives a clause rather than the one-clause
    /// `push_temp` every other instruction result gets. Task 16 gave
    /// `rexx-exec` an opt-in stress mode (`Interp::alloc_with`, `lib.rs`)
    /// that calls this after every allocation when enabled; it is still off
    /// by default, so this paragraph's "nothing calls this" becomes "nothing
    /// calls this unless asked to" rather than going stale outright.
    ///
    /// The pointer is here, rather than only there, because the person who
    /// wires a collector into the interpreter permanently is the one who
    /// turns that window into a use-after-free, and this is the function
    /// they will be looking at. Sweep `rexx-exec` for the same shape before
    /// doing it: an unrooted window is invisible to the compiler and shows
    /// up as a wrong value rather than a crash.
    ///
    /// **That window does not block 4a's collect-on-every-allocation gate
    /// criterion, and this comment used to imply it did.** Nothing between
    /// that pop and `exit_code_for` calls `alloc_with_uncollected`: the
    /// conversion fills a `Number` in place or parses onto the Rust heap.
    /// So a faithful collect-on-every-allocation mode never fires inside the
    /// window, and the criterion can be met with the window still open. It
    /// is a debt against a future *permanent* collector, not an obstacle to
    /// the gate.
    pub fn collect(&mut self, roots: &RootSet) -> CollectStats {
        self.collections += 1;
        self.marks.clear();
        // Resized every time: the heap grows between collections.
        self.marks.resize(self.slots.len(), false);

        let mut work: Vec<ObjRef> = roots.iter().collect();
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
    use crate::{Decoded, GENERATION_MAX, RootSet};

    #[test]
    fn a_slot_at_generation_max_is_retired_not_reused() {
        let mut heap = Heap::new();
        let roots = RootSet::new();
        let r = heap.alloc(Body::Text {
            bytes: b"old".to_vec(),
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
            bytes: b"new".to_vec(),
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
