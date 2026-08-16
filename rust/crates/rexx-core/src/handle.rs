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
/// Short byte strings, stored in the handle itself.
///
/// **The last free tag.** `0b11` named nothing before this: the encoding had
/// three kinds and four tag values. Nothing is taken from the slot or the
/// generation to reach it, so neither budget moves -- see [`GENERATION_MAX`]
/// and [`ObjRef::heap`] for what those budgets are.
const TAG_TEXT: u64 = 0b11;

/// How many bytes fit in a handle.
///
/// **Fixed by the arithmetic and not by a measurement.** Two bits go to the
/// tag and three to the length, which is the fewest that can count `0..=7`,
/// leaving fifty-six for bytes. An eighth byte would need sixty-four bits of
/// payload plus a four-bit length in a sixty-four bit word, so seven is the
/// ceiling for any encoding of this shape rather than a tuning knob.
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

/// The first slot index reserved for a class object's identity, which the
/// arena may never allocate.
///
/// **A class object is not in the arena and its identity is still an
/// `ObjRef`** (D29: "a class's identity is an `ObjRef`; `BehaviourId` stays a
/// `u16` index for the primitive fast path"). Both counters used to start at
/// zero, so the first class and the first heap slot were the same sixty-four
/// bits -- and the failure mode is silent rather than loud: a class handle
/// arriving where a value is expected resolves to whatever object holds that
/// slot, so a send would answer from an unrelated value's class and the
/// collector would be handed a root the program never named.
///
/// Splitting the slot space is what separates them, and the tag space cannot:
/// all four tag values are taken. The arena counts up from zero and
/// [`crate::Heap`] asserts it never reaches here; classes count up from here.
/// Nothing is taken from the generation budget, and the arena's half is still
/// far past what an allocation of `Slot`s could exhaust.
pub const CLASS_SLOT_BASE: u32 = 1 << 31;

/// Inclusive bounds of the inline integer range.
pub const SMALL_INT_MAX: i64 = (1 << 61) - 1;
pub const SMALL_INT_MIN: i64 = -(1 << 61);

/// Whether a decoded `Decoded::Heap`'s parts name a class identity rather than
/// an arena slot -- [`ObjRef::class_id`]'s own test, and the same expression it
/// uses.
///
/// A free function so a caller holding an already-decoded handle asks without
/// decoding a second time. `Interp::operator_operand_gap` is the reason: it
/// has just matched the handle's `Decoded` to find out whether either shape it
/// refuses is even possible, and `ObjRef::class_id` would decode again to
/// answer the same question.
pub const fn is_class_slot(slot: u32, generation: u32) -> bool {
    generation == 0 && slot >= CLASS_SLOT_BASE
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct ObjRef(u64);

/// A byte string held in the handle, with no heap object behind it.
///
/// Derefs to `[u8]`, so a reader treats it as the slice it is. **It is
/// `Copy` and owns its bytes**, which is what makes the encoding worth
/// having and is also its one limitation: there is no shared mutable home
/// for a lazy parse cache the way [`crate::Body::Text`] has one, so a value
/// held this way answers `try_text` with `None` and is materialised through
/// `render` instead.
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
    ///
    /// The bytes are copied in; nothing outside is referenced afterwards,
    /// which is what lets the result outlive its source with no lifetime.
    pub fn inline_text(bytes: &[u8]) -> Option<Self> {
        if bytes.len() > INLINE_TEXT {
            return None;
        }
        let mut buf = [0u8; 8];
        buf[..bytes.len()].copy_from_slice(bytes);
        let data = u64::from_le_bytes(buf);
        Some(ObjRef(
            (data << TEXT_DATA_SHIFT) | ((bytes.len() as u64) << TEXT_LEN_SHIFT) | TAG_TEXT,
        ))
    }

    /// The `id`th class object's identity, or `None` for an `id` the reserved
    /// range does not hold.
    ///
    /// Generation zero, always: a class identity names no slot, so it has no
    /// generation to move on, and fixing it at zero is what lets
    /// [`ObjRef::class_id`] read the range back exactly.
    pub const fn class(id: u32) -> Option<Self> {
        if id >= CLASS_SLOT_BASE {
            return None;
        }
        Some(ObjRef::heap(CLASS_SLOT_BASE | id, 0))
    }

    /// This handle's own class index, or `None` for anything the arena could
    /// have produced.
    ///
    /// **The one predicate that tells a class identity from a value**, and
    /// what a receiver's decode has to ask before it reaches for the arena.
    pub const fn class_id(self) -> Option<u32> {
        match self.decode() {
            Decoded::Heap { slot, generation } if is_class_slot(slot, generation) => {
                Some(slot - CLASS_SLOT_BASE)
            }
            _ => None,
        }
    }

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
