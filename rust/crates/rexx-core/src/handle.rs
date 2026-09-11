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

const TAG_BITS: u32 = 2;
const TAG_MASK: u64 = 0b11;
const TAG_HEAP: u64 = 0b00;
const TAG_INT: u64 = 0b01;
const TAG_NIL: u64 = 0b10;
/// Short byte strings, stored in the handle itself.
const TAG_TEXT: u64 = 0b11;

/// How many bytes fit in a handle.
pub const INLINE_TEXT: usize = 7;

const TEXT_LEN_SHIFT: u32 = TAG_BITS;
const TEXT_LEN_MASK: u64 = 0b111;
/// Bytes start at a byte boundary so that decoding is one shift and one
/// `to_le_bytes`, rather than a shift per byte. The three bits between the
/// length and the first byte are unused and always zero.
const TEXT_DATA_SHIFT: u32 = 8;

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

/// A byte string held in the handle, with no heap object behind it.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct InlineText {
    len: u8,
    buf: [u8; INLINE_TEXT],
}

impl std::ops::Deref for InlineText {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        &self.buf[..self.len as usize]
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Decoded {
    Heap {
        slot: u32,
        generation: u32,
    },
    SmallInt(i64),
    /// Bytes carried in the handle. See [`InlineText`].
    Text(InlineText),
    Nil,
}

impl ObjRef {
    pub const NIL: ObjRef = ObjRef(TAG_NIL);

    /// The handle's own bits, for a caller that has to *index* by identity
    /// rather than compare two handles.
    pub const fn bits(self) -> u64 {
        self.0
    }

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

    /// A handle carrying `bytes` itself, or `None` if there are too many.
    #[inline]
    pub fn inline_text(bytes: &[u8]) -> Option<Self> {
        if bytes.len() > INLINE_TEXT {
            return None;
        }
        let mut buf = [0u8; 8];
        for (slot, byte) in buf.iter_mut().zip(bytes) {
            *slot = *byte;
        }
        let data = u64::from_le_bytes(buf);
        Some(ObjRef(
            (data << TEXT_DATA_SHIFT) | ((bytes.len() as u64) << TEXT_LEN_SHIFT) | TAG_TEXT,
        ))
    }

    /// The inline-text handle for a single byte, as a constant.
    pub const fn inline_byte(byte: u8) -> Self {
        ObjRef(((byte as u64) << TEXT_DATA_SHIFT) | (1 << TEXT_LEN_SHIFT) | TAG_TEXT)
    }

    /// **`always` rather than a hint**, and the difference was measured
    /// rather than assumed. Under this workspace's `lto = "fat"` and
    /// `codegen-units = 1`, a plain `#[inline]` left the out-of-line symbol
    /// in the binary and moved `instructions:u` on every benchmark program
    /// by 0.000%; the callers this matters in are large enough that LLVM's
    /// cost model declines them. With `always`, `bench-programs/emptyloop.
    /// rex` is -3.23% and `varlookup.rex` -1.43%, and `.text` does not grow.
    #[inline(always)]
    pub const fn decode(self) -> Decoded {
        match self.0 & TAG_MASK {
            TAG_HEAP => Decoded::Heap {
                slot: ((self.0 >> SLOT_SHIFT) & SLOT_MASK) as u32,
                generation: (self.0 >> GEN_SHIFT) as u32,
            },
            TAG_INT => Decoded::SmallInt((self.0 as i64) >> TAG_BITS),
            TAG_TEXT => {
                let len = ((self.0 >> TEXT_LEN_SHIFT) & TEXT_LEN_MASK) as u8;
                let full = (self.0 >> TEXT_DATA_SHIFT).to_le_bytes();
                Decoded::Text(InlineText {
                    len,
                    buf: [
                        full[0], full[1], full[2], full[3], full[4], full[5], full[6],
                    ],
                })
            }
            _ => Decoded::Nil,
        }
    }
}

#[cfg(test)]
mod inline_byte_tests {
    use super::*;

    /// The `const` route and the slice route must agree, or every caller that
    /// compares against a constant logical silently stops matching.
    #[test]
    fn a_single_byte_inlines_the_same_way_either_route() {
        for byte in 0u8..=255 {
            assert_eq!(
                Some(ObjRef::inline_byte(byte)),
                ObjRef::inline_text(&[byte]),
                "byte {byte} disagrees between the const and the slice route"
            );
        }
    }
}
