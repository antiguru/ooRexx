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

//! A reference to a Rexx object.
//!
//! Two low bits carry a tag. A `Heap` handle carries a 32-bit slot index and
//! a 30-bit generation; `SmallInt` carries a 62-bit signed value inline,
//! which removes the allocation the C++ implementation pays for via
//! `RexxInteger`. `.nil` is a singleton because Rexx code compares against it
//! by identity.
//!
//! Note that `.true` and `.false` need no encoding: in Rexx they are the
//! strings "1" and "0".
//!
//! Layout, low to high: `[tag: 2][slot: 32][generation: 30]`.
//!
//! The generation is not decoration. Slots are recycled through a free list,
//! so without it a handle held across a collection would silently name
//! whatever is allocated into that slot next -- memory-safe, but returning
//! the wrong object, which is the defect class this design exists to remove.
//! It matters most at the native-API boundary, where foreign code holds
//! references across GC points.

const TAG_BITS: u32 = 2;
const TAG_MASK: u64 = 0b11;
const TAG_HEAP: u64 = 0b00;
const TAG_INT: u64 = 0b01;
const TAG_NIL: u64 = 0b10;

const SLOT_SHIFT: u32 = TAG_BITS;
const SLOT_BITS: u32 = 32;
const SLOT_MASK: u64 = (1 << SLOT_BITS) - 1;
const GEN_SHIFT: u32 = SLOT_SHIFT + SLOT_BITS;
const GEN_BITS: u32 = 30;

/// The highest generation a slot can reach. A slot that would exceed this is
/// retired rather than reused, so a stale handle can never alias a live one.
pub const GENERATION_MAX: u32 = (1 << GEN_BITS) - 1;

/// Inclusive bounds of the inline integer range.
pub const SMALL_INT_MAX: i64 = (1 << 61) - 1;
pub const SMALL_INT_MIN: i64 = -(1 << 61);

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct ObjRef(u64);

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Decoded {
    Heap { slot: u32, generation: u32 },
    SmallInt(i64),
    Nil,
}

impl ObjRef {
    pub const NIL: ObjRef = ObjRef(TAG_NIL);

    pub const fn heap(slot: u32, generation: u32) -> Self {
        debug_assert!(generation <= GENERATION_MAX);
        ObjRef(((generation as u64) << GEN_SHIFT) | ((slot as u64) << SLOT_SHIFT) | TAG_HEAP)
    }

    pub const fn small_int(value: i64) -> Option<Self> {
        if value > SMALL_INT_MAX || value < SMALL_INT_MIN {
            return None;
        }
        Some(ObjRef((((value as u64) << TAG_BITS) & !TAG_MASK) | TAG_INT))
    }

    pub const fn decode(self) -> Decoded {
        match self.0 & TAG_MASK {
            TAG_HEAP => Decoded::Heap {
                slot: ((self.0 >> SLOT_SHIFT) & SLOT_MASK) as u32,
                generation: (self.0 >> GEN_SHIFT) as u32,
            },
            TAG_INT => Decoded::SmallInt((self.0 as i64) >> TAG_BITS),
            _ => Decoded::Nil,
        }
    }
}
