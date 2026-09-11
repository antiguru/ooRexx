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

//! A [`Body::Text`]'s bytes, held inline while they are few.

use std::fmt;
use std::mem::MaybeUninit;
use std::ops::Deref;

/// How many bytes fit without a heap allocation.
pub const INLINE_BYTES: usize = 54;

/// **This is the payload width the capacity above is chosen against**, and it
/// is asserted rather than described because a layout claim defended by a
/// sentence has rotted in this repository before. `Body`'s and `Slot`'s own
/// bounds are the two that matter to the arena; this one says which capacity
/// produced them.
const _: () = assert!(size_of::<Bytes>() <= 56);

/// An immutable byte string, stored inline when it is at most
/// [`INLINE_BYTES`] long.
#[derive(Clone)]
pub struct Bytes(Repr);

/// What a debug build writes past `len`, so that a read which should never
/// happen produces something a test can recognise. `0xAB` is not a byte any
/// test source here produces.
#[cfg(debug_assertions)]
const POISON: u8 = 0xAB;

#[derive(Clone)]
enum Repr {
    Inline {
        len: u8,
        buf: [MaybeUninit<u8>; INLINE_BYTES],
    },
    Heap(Vec<u8>),
}

impl Bytes {
    /// A copy of `source`, inline when it fits.
    pub fn from_slice(source: &[u8]) -> Bytes {
        if source.len() <= INLINE_BYTES {
            let mut buf = [MaybeUninit::<u8>::uninit(); INLINE_BYTES];
            for (slot, byte) in buf.iter_mut().zip(source) {
                slot.write(*byte);
            }
            // See the type's doc: this exists so the invariant has a test.
            #[cfg(debug_assertions)]
            for slot in &mut buf[source.len()..] {
                slot.write(POISON);
            }
            Bytes(Repr::Inline {
                len: source.len() as u8,
                buf,
            })
        } else {
            Bytes(Repr::Heap(source.to_vec()))
        }
    }

    /// [`from_slice`], for a caller that already owns the bytes.
    pub fn from_vec(source: Vec<u8>) -> Bytes {
        if source.len() <= INLINE_BYTES {
            Bytes::from_slice(&source)
        } else {
            Bytes(Repr::Heap(source))
        }
    }

    /// **The only reader of `Repr::Inline`'s buffer**, which is the other half
    /// of what makes the invariant local: every comparison, hash, `Debug` and
    /// deref a caller performs arrives here first.
    pub fn as_slice(&self) -> &[u8] {
        match &self.0 {
            Repr::Inline { len, buf } => {
                let len = *len as usize;
                // Not a redundant bound: it is what says the slice below stays
                // inside the array, stated where the `unsafe` can see it
                // rather than left to `len`'s type.
                debug_assert!(len <= INLINE_BYTES, "an inline length past the buffer");
                // SAFETY: `from_slice` is the only constructor of
                // `Repr::Inline` and it writes exactly `len` bytes before
                // setting `len`, so the first `len` elements of `buf` are
                // initialised. `len` is a `u8` and `INLINE_BYTES` is 54, and
                // the assertion above holds it inside the array on every debug
                // run of the suite. Nothing between construction and here can
                // shorten the array or lengthen `len`: `Repr` is private to
                // this file and neither field is written anywhere else.
                unsafe { std::slice::from_raw_parts(buf.as_ptr().cast::<u8>(), len) }
            }
            Repr::Heap(v) => v,
        }
    }

    /// Whether these bytes are held inline rather than in a separate
    /// allocation.
    pub fn is_inline(&self) -> bool {
        matches!(self.0, Repr::Inline { .. })
    }
}

impl Deref for Bytes {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        self.as_slice()
    }
}

/// Shows the live bytes and never the padding behind them: the bytes past
/// `len` are not part of the value, and printing them would make two equal
/// values look different.
impl fmt::Debug for Bytes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_slice(), f)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The capacity is a boundary and both sides of it have to answer the
    /// same bytes, including the two lengths that bracket it exactly.
    #[test]
    fn both_arms_answer_the_bytes_they_were_given() {
        for len in 0..=(INLINE_BYTES + 8) {
            let source: Vec<u8> = (0..len).map(|i| (i % 251) as u8).collect();

            let from_slice = Bytes::from_slice(&source);
            assert_eq!(from_slice.as_slice(), &source[..], "from_slice at {len}");
            assert_eq!(&*from_slice, &source[..], "deref at {len}");
            assert_eq!(from_slice.len(), len, "len at {len}");

            let from_vec = Bytes::from_vec(source.clone());
            assert_eq!(from_vec.as_slice(), &source[..], "from_vec at {len}");

            let cloned = from_slice.clone();
            assert_eq!(cloned.as_slice(), &source[..], "clone at {len}");

            assert_eq!(
                from_slice.is_inline(),
                from_vec.is_inline(),
                "the two constructors disagree about the arm at {len}"
            );
        }
    }

    /// The capacity claim itself: `INLINE_BYTES` is inline and one more is
    /// not, through both constructors.
    #[test]
    fn the_capacity_is_where_the_constant_says_it_is() {
        let fits = vec![b'x'; INLINE_BYTES];
        let one_over = vec![b'x'; INLINE_BYTES + 1];

        assert!(Bytes::from_slice(&fits).is_inline());
        assert!(Bytes::from_vec(fits).is_inline());
        assert!(!Bytes::from_slice(&one_over).is_inline());
        assert!(!Bytes::from_vec(one_over).is_inline());

        assert!(Bytes::from_slice(b"").is_inline(), "the empty string");
    }

    /// The guarantee a fallibly-reserved buffer rests on, asserted on the
    /// allocation's own address because nothing else distinguishes the two:
    /// the bytes come out equal whether the buffer was taken or copied.
    #[test]
    fn a_buffer_past_the_capacity_is_taken_rather_than_copied() {
        let source = vec![b'y'; INLINE_BYTES + 1];
        let address = source.as_ptr();
        let bytes = Bytes::from_vec(source);
        assert_eq!(bytes.as_slice().as_ptr(), address);
    }

    /// A value that crosses the boundary by growth: the caller grows its own
    /// buffer, because a `Bytes` never does, and each length lands on the arm
    /// its length asks for.
    #[test]
    fn a_buffer_grown_across_the_capacity_changes_arm_where_it_should() {
        let mut source = Vec::new();
        for len in 0..=(INLINE_BYTES + 4) {
            let bytes = Bytes::from_slice(&source);
            assert_eq!(bytes.as_slice(), &source[..]);
            assert_eq!(
                bytes.is_inline(),
                len <= INLINE_BYTES,
                "wrong arm at length {len}"
            );
            source.push(b'a');
        }
    }

    /// **What [`POISON`] is for, and it is not extra mutation coverage.**
    /// Three mutations were applied and all three redden this file:
    /// `as_slice` answering `INLINE_BYTES` bytes, `as_slice` answering
    /// `len + 1`, and `from_slice` writing one byte too few. Every one of them
    /// is caught by the value assertions that were already here, so this test
    /// finds nothing they do not.
    #[cfg(debug_assertions)]
    #[test]
    fn the_bytes_past_len_are_never_part_of_the_value() {
        for len in 0..=INLINE_BYTES {
            let source: Vec<u8> = (0..len).map(|i| (i % 97) as u8).collect();
            assert!(
                !source.contains(&POISON),
                "the source itself carries the poison byte at {len}"
            );
            let bytes = Bytes::from_slice(&source);
            assert!(bytes.is_inline(), "not the arm under test at {len}");
            assert_eq!(bytes.as_slice(), &source[..], "bytes at {len}");
            assert!(
                !bytes.as_slice().contains(&POISON),
                "a read went past len at {len}: {:?}",
                bytes.as_slice()
            );
        }
    }

    /// Two constructors reaching the inline arm answer the same value, and
    /// the tails they leave behind are not allowed to make them differ.
    #[test]
    fn the_two_constructors_agree_whatever_is_behind_the_live_bytes() {
        for len in 0..=INLINE_BYTES {
            let source: Vec<u8> = (0..len).map(|i| (i % 251) as u8).collect();
            let a = Bytes::from_slice(&source);
            let b = Bytes::from_vec(source.clone());
            assert_eq!(a.as_slice(), b.as_slice(), "value at {len}");
            assert_eq!(format!("{a:?}"), format!("{b:?}"), "Debug at {len}");
        }
    }

    #[test]
    fn debug_shows_the_live_bytes_and_not_the_padding() {
        assert_eq!(format!("{:?}", Bytes::from_slice(b"ab")), "[97, 98]");
        assert_eq!(format!("{:?}", Bytes::from_slice(b"")), "[]");
    }
}
