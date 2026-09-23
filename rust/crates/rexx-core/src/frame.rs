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

//! Register frames carved from blocks that never move while their arena
//! lives, read and written through a base pointer with no bound check.
//!
//! Every access is in bounds whatever index or release order the caller
//! uses, by properties this module alone establishes:
//!
//! 1. a block is one zeroed allocation of `cells + GUARD` cells, freed only by
//!    `FrameArena`'s `Drop`;
//! 2. a frame starts at an offset `<= cells` of its block (`FrameArena::reserve`
//!    opens a fresh block otherwise);
//! 3. an index is a `u16`, so below `GUARD`, and `start + index < cells + GUARD`;
//! 4. a `RegFrame<'a>` borrows its arena for `'a`, so it cannot be used once the
//!    arena is dropped;
//! 5. `FrameArena::iter` clamps every used length it walks to `cells + GUARD`,
//!    so bookkeeping a misordered release corrupted cannot widen its slices.
//!
//! Staying inside a frame's own `len` is a logical property, not a safety one;
//! the chunk validator in `rexx-exec` establishes it.

// Granted by Moritz, 2026-09-23: the invariant above is the whole argument.
#![allow(unsafe_code)]

use std::alloc::{Layout, alloc_zeroed, dealloc, handle_alloc_error};
use std::cell::{Cell, RefCell};
use std::marker::PhantomData;
use std::ptr::NonNull;

use crate::ObjRef;

/// Cells every block carries past the last offset a frame may start at: one
/// more than the largest `u16` index.
const GUARD: usize = u16::MAX as usize + 1;

/// How many cells of a block new frames may start in, validated when built.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct FrameBlock(usize);

impl FrameBlock {
    /// The smallest accepted size.
    pub const MIN: usize = 1;
    /// The largest accepted size.
    pub const MAX: usize = 1 << 24;
    /// The size the interpreter uses unless an embedding picks another.
    pub const DEFAULT: FrameBlock = FrameBlock(1 << 16);

    /// A block size of `cells`, or `None` outside `MIN..=MAX`.
    pub fn new(cells: usize) -> Option<FrameBlock> {
        (Self::MIN..=Self::MAX)
            .contains(&cells)
            .then_some(FrameBlock(cells))
    }

    pub fn cells(self) -> usize {
        self.0
    }

    fn layout(self) -> Layout {
        Layout::array::<Cell<ObjRef>>(self.0 + GUARD).expect("MAX keeps the layout in range")
    }
}

impl Default for FrameBlock {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// One activation's register frame, usable while its arena is borrowed.
#[derive(Copy, Clone, Debug)]
pub struct RegFrame<'a> {
    base: NonNull<Cell<ObjRef>>,
    len: u16,
    /// Whether reserving this frame opened its block.
    opened: bool,
    arena: PhantomData<&'a FrameArena>,
}

impl RegFrame<'_> {
    /// Reads register `index`.
    #[inline(always)]
    pub fn get(self, index: u16) -> ObjRef {
        debug_assert!(index < self.len);
        // SAFETY: properties 1-4 of the module doc put `base + index` inside a
        // live, zeroed block, and every bit pattern is a valid `ObjRef`. Cells
        // are only ever reached through `Cell`, so this shared access cannot
        // alias a `&mut`.
        unsafe { (*self.base.as_ptr().add(index as usize)).get() }
    }

    /// Writes register `index`.
    #[inline(always)]
    pub fn set(self, index: u16, value: ObjRef) {
        debug_assert!(index < self.len);
        // SAFETY: as in `get`.
        unsafe { (*self.base.as_ptr().add(index as usize)).set(value) }
    }

    /// How many registers this frame holds.
    pub fn len(self) -> u16 {
        self.len
    }

    pub fn is_empty(self) -> bool {
        self.len == 0
    }
}

/// The stack of live register frames, reserved and released in LIFO order.
///
/// A frame cannot outlive the arena it came from:
///
/// ```compile_fail,E0597
/// use rexx_core::{FrameArena, FrameBlock};
/// let escaped = {
///     let arena = FrameArena::new(FrameBlock::DEFAULT);
///     arena.reserve(1)
/// };
/// escaped.get(0);
/// ```
///
/// while the same use inside the arena's scope compiles:
///
/// ```
/// use rexx_core::{FrameArena, FrameBlock};
/// let arena = FrameArena::new(FrameBlock::DEFAULT);
/// let frame = arena.reserve(1);
/// assert_eq!(frame.get(0), rexx_core::ObjRef::NIL);
/// arena.release(frame);
/// ```
pub struct FrameArena {
    size: FrameBlock,
    /// Every block allocated so far, each of `size.layout()`.
    blocks: RefCell<Vec<NonNull<Cell<ObjRef>>>>,
    /// The used length of each block below `current`, saved when it was left.
    tops: RefCell<Vec<usize>>,
    current: Cell<usize>,
    /// `blocks[current]`, dangling before the first block.
    block: Cell<NonNull<Cell<ObjRef>>>,
    /// The used length of `block`; above `size` when the next frame must open
    /// a block, which includes the state before the first one.
    top: Cell<usize>,
}

impl FrameArena {
    pub fn new(size: FrameBlock) -> Self {
        FrameArena {
            size,
            blocks: RefCell::new(Vec::new()),
            tops: RefCell::new(Vec::new()),
            current: Cell::new(0),
            block: Cell::new(NonNull::dangling()),
            top: Cell::new(usize::MAX),
        }
    }

    pub fn size(&self) -> FrameBlock {
        self.size
    }

    /// Carves a frame of `len` registers, all [`ObjRef::NIL`].
    #[inline]
    pub fn reserve(&self, len: u16) -> RegFrame<'_> {
        let mut start = self.top.get();
        let opened = start > self.size.0;
        if opened {
            start = self.open_block();
        }
        // SAFETY: `start <= size`, so this stays inside the block (property 2).
        let base = unsafe { self.block.get().add(start) };
        for index in 0..len as usize {
            // SAFETY: `index < GUARD` (property 3).
            unsafe { (*base.as_ptr().add(index)).set(ObjRef::NIL) }
        }
        self.top.set(start + len as usize);
        RegFrame {
            base,
            len,
            opened,
            arena: PhantomData,
        }
    }

    /// Moves to the next block, allocating it the first time, and answers the
    /// offset the new frame starts at.
    #[cold]
    #[inline(never)]
    fn open_block(&self) -> usize {
        let mut blocks = self.blocks.borrow_mut();
        if !blocks.is_empty() {
            self.tops.borrow_mut().push(self.top.get());
            self.current.set(self.current.get() + 1);
        }
        if self.current.get() == blocks.len() {
            let layout = self.size.layout();
            // SAFETY: `layout` is at least `GUARD` cells, so not zero-sized.
            let raw = unsafe { alloc_zeroed(layout) }.cast::<Cell<ObjRef>>();
            let Some(raw) = NonNull::new(raw) else {
                handle_alloc_error(layout)
            };
            blocks.push(raw);
        }
        self.block.set(blocks[self.current.get()]);
        self.top.set(0);
        0
    }

    /// Releases `frame`, which must be the most recently reserved live one.
    #[inline]
    pub fn release(&self, frame: RegFrame<'_>) {
        let start = self.top.get().wrapping_sub(frame.len as usize);
        debug_assert!(
            self.top.get() >= frame.len as usize
                && std::ptr::eq(
                    frame.base.as_ptr(),
                    self.block.get().as_ptr().wrapping_add(start)
                ),
            "frame arena released out of order"
        );
        self.top.set(start);
        if frame.opened {
            self.close_block();
        }
    }

    #[cold]
    #[inline(never)]
    fn close_block(&self) {
        // The first block's opener leaves that block current, at `top` 0.
        if let Some(top) = self.tops.borrow_mut().pop() {
            self.current.set(self.current.get() - 1);
            self.block.set(self.blocks.borrow()[self.current.get()]);
            self.top.set(top);
        }
    }

    /// How many blocks have been allocated.
    pub fn blocks(&self) -> usize {
        self.blocks.borrow().len()
    }

    /// Every register of every live frame.
    pub fn iter(&self) -> impl Iterator<Item = ObjRef> + '_ {
        let live = if self.blocks.borrow().is_empty() {
            0
        } else {
            self.current.get() + 1
        };
        (0..live).flat_map(move |index| {
            let block = self.blocks.borrow()[index];
            let used = if index == self.current.get() {
                self.top.get()
            } else {
                self.tops.borrow()[index]
            }
            .min(self.size.0 + GUARD);
            // SAFETY: `used <= size + GUARD` (property 5), the block's length,
            // and the block lives as long as `self` (property 1). The cells are
            // `Cell`s, so this shared slice cannot alias a `&mut`.
            let cells = unsafe { std::slice::from_raw_parts(block.as_ptr(), used) };
            cells.iter().map(Cell::get)
        })
    }
}

impl Drop for FrameArena {
    fn drop(&mut self) {
        let layout = self.size.layout();
        for block in self.blocks.get_mut().drain(..) {
            // SAFETY: allocated in `open_block` with this same layout, and no
            // `RegFrame` borrowing `self` can still exist (property 4).
            unsafe { dealloc(block.as_ptr().cast(), layout) }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{FrameArena, FrameBlock, GUARD};
    use crate::ObjRef;

    fn offset(arena: &FrameArena, frame: super::RegFrame<'_>) -> usize {
        (frame.base.as_ptr() as usize - arena.block.get().as_ptr() as usize)
            / std::mem::size_of::<std::cell::Cell<ObjRef>>()
    }

    #[test]
    fn every_frame_starts_inside_its_blocks_admitted_range() {
        for cells in [1, 3, 64] {
            let arena = FrameArena::new(FrameBlock::new(cells).expect("in range"));
            let mut frames = Vec::new();
            for len in [0, 2, 5, 1, 2, 0, 7, 3].iter().cycle().take(40) {
                let frame = arena.reserve(*len);
                assert!(offset(&arena, frame) <= cells);
                frames.push(frame);
            }
            while let Some(frame) = frames.pop() {
                arena.release(frame);
            }
        }
    }

    #[test]
    fn a_frame_larger_than_the_block_holds_every_register() {
        let arena = FrameArena::new(FrameBlock::new(1).expect("in range"));
        let small = arena.reserve(1);
        let big = arena.reserve(u16::MAX);
        let value = ObjRef::small_int(7).expect("small");
        big.set(u16::MAX - 1, value);
        small.set(0, value);
        assert_eq!(big.get(u16::MAX - 1), value);
        assert_eq!(arena.iter().filter(|v| *v == value).count(), 2);
        assert_eq!(arena.blocks(), 1);
        assert_eq!(offset(&arena, big), 1);
        assert!(offset(&arena, big) + u16::MAX as usize <= 1 + GUARD);
        arena.release(big);
        arena.release(small);
        assert_eq!(arena.iter().count(), 0);
    }

    #[test]
    fn frames_spill_into_new_blocks_and_come_back() {
        let arena = FrameArena::new(FrameBlock::new(10).expect("in range"));
        let value = ObjRef::small_int(3).expect("small");
        let mut frames = Vec::new();
        for _ in 0..20 {
            let frame = arena.reserve(4);
            frame.set(3, value);
            frames.push(frame);
        }
        assert!(arena.blocks() > 2);
        assert_eq!(arena.iter().filter(|v| *v == value).count(), 20);
        while let Some(frame) = frames.pop() {
            assert_eq!(frame.get(3), value);
            arena.release(frame);
        }
        assert_eq!(arena.iter().count(), 0);
        let again = arena.reserve(2);
        assert_eq!(offset(&arena, again), 0);
        assert_eq!(again.get(1), ObjRef::NIL);
        arena.release(again);
    }

    #[test]
    fn empty_frames_nest_across_a_block_boundary() {
        let arena = FrameArena::new(FrameBlock::new(1).expect("in range"));
        let outer = arena.reserve(2);
        let opener = arena.reserve(0);
        let inner = arena.reserve(0);
        let held = arena.reserve(1);
        held.set(0, ObjRef::small_int(1).expect("small"));
        arena.release(held);
        arena.release(inner);
        arena.release(opener);
        outer.set(1, ObjRef::small_int(2).expect("small"));
        assert_eq!(arena.iter().count(), 2);
        arena.release(outer);
        assert_eq!(arena.iter().count(), 0);
    }

    #[test]
    fn a_block_size_outside_the_range_is_refused() {
        assert_eq!(FrameBlock::new(0), None);
        assert_eq!(FrameBlock::new(FrameBlock::MAX + 1), None);
        assert_eq!(
            FrameBlock::new(FrameBlock::MIN).map(FrameBlock::cells),
            Some(FrameBlock::MIN)
        );
        assert_eq!(
            FrameBlock::new(FrameBlock::MAX).map(FrameBlock::cells),
            Some(FrameBlock::MAX)
        );
    }

    #[test]
    #[cfg(debug_assertions)]
    #[should_panic(expected = "frame arena released out of order")]
    fn an_out_of_order_release_is_refused() {
        let arena = FrameArena::new(FrameBlock::DEFAULT);
        let first = arena.reserve(1);
        let _second = arena.reserve(1);
        arena.release(first);
    }

    #[test]
    #[cfg(not(debug_assertions))]
    fn a_double_release_leaves_the_walk_inside_its_blocks() {
        let arena = FrameArena::new(FrameBlock::new(4).expect("in range"));
        let frame = arena.reserve(2);
        arena.release(frame);
        arena.release(frame);
        let read = |arena: &FrameArena| arena.iter().filter(|v| *v != ObjRef::NIL).count();
        assert!(read(&arena) <= 4 + GUARD);
        let opener = arena.reserve(1);
        assert!(read(&arena) <= 2 * (4 + GUARD));
        arena.release(opener);
    }
}
