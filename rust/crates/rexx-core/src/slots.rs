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

//! An array's slots with the index of its last item kept beside them.

use std::collections::TryReserveError;
use std::ops::Deref;

use crate::ObjRef;

/// An array's slots in index order, `None` for a slot that holds no object,
/// and the 1-based index of the last occupied slot, 0 when none is
/// (`ArrayClass::lastItem`). Reads go through `Deref`. Every write goes
/// through a method here, which keeps the index.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ArraySlots {
    items: Vec<Option<ObjRef>>,
    last: usize,
}

impl ArraySlots {
    /// Slots over `items`.
    pub fn new(items: Vec<Option<ObjRef>>) -> ArraySlots {
        let last = last_within(&items, items.len());
        ArraySlots { items, last }
    }

    /// The 1-based index of the last occupied slot, or 0.
    #[inline]
    pub fn last_item(&self) -> usize {
        debug_assert_eq!(
            self.last,
            last_within(&self.items, self.items.len()),
            "the cached last item disagrees with the slots"
        );
        self.last
    }

    /// Writes slot `at`.
    ///
    /// # Panics
    ///
    /// If `at` is past the end.
    #[inline]
    pub fn set(&mut self, at: usize, item: Option<ObjRef>) {
        self.items[at] = item;
        self.wrote(at, item.is_some());
    }

    /// Writes slot `at` if there is one, and answers whether there was.
    #[inline]
    pub fn set_within(&mut self, at: usize, item: Option<ObjRef>) -> bool {
        match self.items.get_mut(at) {
            Some(slot) => {
                *slot = item;
                self.wrote(at, item.is_some());
                true
            }
            None => false,
        }
    }

    /// Writes every slot.
    pub fn fill(&mut self, item: Option<ObjRef>) {
        self.items.fill(item);
        self.last = if item.is_some() { self.items.len() } else { 0 };
    }

    /// Writes slots `0..` from `items`, stopping at whichever ends first.
    pub fn overwrite(&mut self, items: impl IntoIterator<Item = ObjRef>) {
        let mut written = 0;
        for (slot, item) in self.items.iter_mut().zip(items) {
            *slot = Some(item);
            written += 1;
        }
        self.last = self.last.max(written);
    }

    /// Appends one slot.
    pub fn push(&mut self, item: Option<ObjRef>) {
        self.items.push(item);
        if item.is_some() {
            self.last = self.items.len();
        }
    }

    /// Removes the final slot and answers what it held.
    pub fn pop(&mut self) -> Option<Option<ObjRef>> {
        let popped = self.items.pop();
        if self.last > self.items.len() {
            self.last = last_within(&self.items, self.items.len());
        }
        popped
    }

    /// Inserts a slot at `at`, shifting the later ones up.
    ///
    /// # Panics
    ///
    /// If `at` is past the end.
    pub fn insert(&mut self, at: usize, item: Option<ObjRef>) {
        self.items.insert(at, item);
        if at < self.last {
            self.last += 1;
        } else if item.is_some() {
            self.last = at + 1;
        }
    }

    /// Removes slot `at`, shifting the later ones down, and answers what it
    /// held.
    ///
    /// # Panics
    ///
    /// If `at` is past the end.
    pub fn remove(&mut self, at: usize) -> Option<ObjRef> {
        let removed = self.items.remove(at);
        if at + 1 < self.last {
            self.last -= 1;
        } else if at + 1 == self.last {
            self.last = last_within(&self.items, at);
        }
        removed
    }

    /// Grows or shrinks to `len` slots, the added ones empty.
    pub fn resize(&mut self, len: usize) {
        self.items.resize(len, None);
        if self.last > len {
            self.last = last_within(&self.items, len);
        }
    }

    /// Removes every slot.
    pub fn clear(&mut self) {
        self.items.clear();
        self.last = 0;
    }

    /// Replaces every slot with `items` and answers the old ones.
    pub fn replace(&mut self, items: Vec<Option<ObjRef>>) -> Vec<Option<ObjRef>> {
        std::mem::replace(self, ArraySlots::new(items)).items
    }

    /// `Vec::try_reserve_exact` over the slots.
    pub fn try_reserve_exact(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.items.try_reserve_exact(additional)
    }

    /// How many slots the allocation holds.
    pub fn capacity(&self) -> usize {
        self.items.capacity()
    }

    /// The slots themselves.
    pub fn into_vec(self) -> Vec<Option<ObjRef>> {
        self.items
    }

    /// Moves the index after slot `at` was written, `occupied` saying whether
    /// it now holds an object.
    #[inline]
    fn wrote(&mut self, at: usize, occupied: bool) {
        if occupied {
            self.last = self.last.max(at + 1);
        } else if at + 1 == self.last {
            self.last = last_within(&self.items, at);
        }
    }
}

impl Deref for ArraySlots {
    type Target = [Option<ObjRef>];

    #[inline]
    fn deref(&self) -> &[Option<ObjRef>] {
        &self.items
    }
}

impl From<Vec<Option<ObjRef>>> for ArraySlots {
    fn from(items: Vec<Option<ObjRef>>) -> ArraySlots {
        ArraySlots::new(items)
    }
}

/// The 1-based index of the last occupied slot among `items[..bound]`, or 0.
fn last_within(items: &[Option<ObjRef>], bound: usize) -> usize {
    items[..bound]
        .iter()
        .rposition(Option::is_some)
        .map_or(0, |at| at + 1)
}

#[cfg(test)]
mod tests {
    use super::ArraySlots;
    use crate::ObjRef;

    fn some(n: i64) -> Option<ObjRef> {
        let item = ObjRef::small_int(n);
        assert!(item.is_some(), "{n} is a small integer");
        item
    }

    /// Every write method, each followed by the accessor, whose debug
    /// assertion compares the index with the slots.
    #[test]
    fn every_write_keeps_the_last_item() {
        let mut slots = ArraySlots::new(vec![None, some(1), None, None]);
        assert_eq!(slots.last_item(), 2);
        slots.set(3, some(2));
        assert_eq!(slots.last_item(), 4);
        slots.set(3, None);
        assert_eq!(slots.last_item(), 2);
        assert!(slots.set_within(0, some(3)));
        assert!(!slots.set_within(9, some(3)));
        assert_eq!(slots.last_item(), 2);
        slots.set_within(1, None);
        assert_eq!(slots.last_item(), 1);
        slots.insert(0, None);
        assert_eq!(slots.last_item(), 2);
        slots.insert(4, some(4));
        assert_eq!(slots.last_item(), 5);
        assert_eq!(slots.remove(0), None);
        assert_eq!(slots.last_item(), 4);
        assert_eq!(slots.remove(3), some(4));
        assert_eq!(slots.last_item(), 1);
        slots.push(None);
        assert_eq!(slots.last_item(), 1);
        slots.push(some(5));
        assert_eq!(slots.last_item(), slots.len());
        assert_eq!(slots.pop(), Some(some(5)));
        assert_eq!(slots.last_item(), 1);
        slots.resize(10);
        assert_eq!(slots.last_item(), 1);
        slots.resize(0);
        assert_eq!(slots.last_item(), 0);
        slots.resize(3);
        slots.fill(some(6));
        assert_eq!(slots.last_item(), 3);
        slots.fill(None);
        assert_eq!(slots.last_item(), 0);
        slots.overwrite([some(7), some(8)].into_iter().flatten());
        assert_eq!(slots.last_item(), 2);
        assert_eq!(slots.replace(vec![some(9), None]).len(), 3);
        assert_eq!(slots.last_item(), 1);
        slots.clear();
        assert_eq!(slots.last_item(), 0);
    }
}
