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
    Free {
        next: Option<u32>,
        generation: u32,
    },
    Live {
        object: Object,
        generation: u32,
    },
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
}

pub struct Heap {
    slots: Vec<Slot>,
    free_head: Option<u32>,
    live: usize,
    marks: Vec<bool>,
}

impl Heap {
    pub fn new() -> Self {
        Heap { slots: Vec::new(), free_head: None, live: 0, marks: Vec::new() }
    }

    pub fn slot_capacity(&self) -> usize {
        self.slots.len()
    }

    /// Marks from the roots, then sweeps everything unmarked.
    pub fn collect(&mut self, roots: &RootSet) -> CollectStats {
        self.marks.clear();
        // Resized every time: the heap grows between collections.
        self.marks.resize(self.slots.len(), false);

        let mut work: Vec<ObjRef> = roots.iter().collect();
        let mut reached = Vec::new();
        while let Some(r) = work.pop() {
            let Some(slot) = self.resolve(r) else { continue };
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
                    let free = Slot::Free { next: self.free_head, generation: next };
                    self.free_head = Some(slot as u32);
                    free
                }
                _ => Slot::Free { next: None, generation },
            };
        }
        CollectStats { swept, live: self.live }
    }

    pub fn alloc(&mut self, body: Body) -> ObjRef {
        self.alloc_with(BehaviourId::OBJECT, body)
    }

    pub fn alloc_with(&mut self, behaviour: BehaviourId, body: Body) -> ObjRef {
        let object = Object { behaviour, body };
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
                self.slots.push(Slot::Live { object, generation: 0 });
                ObjRef::heap(slot, 0)
            }
        }
    }

    /// Resolves a handle, or `None` if it names no slot, a free slot, or a
    /// slot whose generation has moved on.
    fn resolve(&self, r: ObjRef) -> Option<usize> {
        let Decoded::Heap { slot, generation } = r.decode() else { return None };
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
        let r = heap.alloc(Body::String("old".into()));
        let Decoded::Heap { slot, .. } = r.decode() else { panic!("heap handle") };
        if let Slot::Live { generation, .. } = &mut heap.slots[slot as usize] {
            *generation = GENERATION_MAX;
        }
        let stale = ObjRef::heap(slot, GENERATION_MAX);
        heap.collect(&roots);
        let next = heap.alloc(Body::String("new".into()));
        assert_eq!(heap.slot_capacity(), 2, "the retired slot must not be reused");
        assert!(heap.get(stale).is_none(), "the stale handle still misses");
        assert!(heap.get(next).is_some());
    }
}
