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
            reached.clear();
            object.body.trace(&mut reached);
            resurrect.extend(reached.iter().copied());
        }

        let mut swept = 0;
        let mut freed_classes = Vec::new();
        for slot in 0..self.slots.len() {
            if self.marks[slot] || matches!(self.slots[slot], Slot::Free { .. }) {
                continue;
            }
            let generation = self.slots[slot].generation();
            // Reported so the caller can drop the rows it keys by this class.
            // The handle is still the live one here; after the assignment
            // below its generation has moved on and it would name nothing.
            if matches!(
                self.slots[slot],
                Slot::Live {
                    object: Object {
                        body: Body::Class { .. },
                        ..
                    },
                    ..
                }
            ) {
                freed_classes.push(ObjRef::heap(slot as u32, generation));
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
        CollectStats {
            swept,
            live: self.live,
            pending_uninit,
            freed_classes,
        }
    }

    pub fn alloc(&mut self, body: Body) -> ObjRef {
        self.alloc_with_uncollected(BehaviourId::OBJECT, body)
    }

    /// Allocates an object this heap will never sweep.
    pub fn mint_class(&mut self) -> ObjRef {
        self.alloc_immortal(BehaviourId::OBJECT, Body::Class { owned: Vec::new() })
    }

    pub fn alloc_immortal(&mut self, behaviour: BehaviourId, body: Body) -> ObjRef {
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
            .filter(|&r| self.get(r).is_some_and(crate::Object::has_uninit))
            .collect();
    }

    /// How many objects this heap has interned as immortal.
    pub fn immortal_count(&self) -> usize {
        self.immortal.len()
    }

    /// Allocates without ever collecting, whatever else is enabled.
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
                    },
                    generation,
                };
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
                    },
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
