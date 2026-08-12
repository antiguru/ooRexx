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
//!
//! The interpreter stores a string's bytes *in* the object: `RexxString` ends
//! in `char stringData[4]`, a trailing flexible array, so a string is one
//! allocation with its header and its bytes contiguous. A `Vec<u8>` beside the
//! arena slot costs a second allocation for every string however short, plus
//! the `free` that matches it and a dependent load to read a byte.
//!
//! [`Bytes`] is that answer in safe Rust: an inline array for the short case
//! and a `Vec<u8>` for the rest. It derefs to `[u8]`, so every read is the
//! slice operation it always was, and only construction goes through a method
//! here.
//!
//! **A Rexx string is immutable**, which is what makes this shape simple: no
//! value ever grows in place, so there is no spill path and no in-place edit
//! to get wrong. A `Bytes` is decided once, at construction, from the length
//! of what it is given.
//!
//! [`Body::Text`]: crate::Body

use std::fmt;
use std::ops::Deref;

/// How many bytes fit without a heap allocation.
///
/// **This comes from the layout ceiling and from nothing else.** `Body`'s
/// width is set by its widest variant, which is `Body::Stem`, and `Body::Text`
/// has headroom against it. Measured with `size_of` over candidate capacities:
/// `size_of::<Body>()` is 80 and `size_of::<Slot>()` is 96 for every capacity
/// up to fifty-four, and fifty-five is the first that widens both by eight. So
/// fifty-four is the largest capacity that costs the arena nothing, and one
/// byte more costs every object in the heap, including the ones that hold no
/// text at all.
///
/// **It is deliberately not chosen from any measured string population**, and
/// the trap is worth naming because a benchmark sits right on it:
/// `bench-programs/strings.rex` builds a 43-byte pangram and a 46-byte
/// concatenation, so any capacity from 46 upwards inlines the whole of that
/// axis. A capacity of 46 would measure the same as this one and would be the
/// representation fitted to the benchmark. The three assertions -- here, on
/// `Body` and on `Slot` -- are what hold the real bound.
pub const INLINE_BYTES: usize = 54;

/// **This is the payload width the capacity above is chosen against**, and it
/// is asserted rather than described because a layout claim defended by a
/// sentence has rotted in this repository before. `Body`'s and `Slot`'s own
/// bounds are the two that matter to the arena; this one says which capacity
/// produced them.
///
/// An upper bound rather than an equality, for the reason `Body`'s carries:
/// the claim is that nothing widened, and shrinking needs no decision.
const _: () = assert!(size_of::<Bytes>() <= 56);

/// An immutable byte string, stored inline when it is at most
/// [`INLINE_BYTES`] long.
///
/// The representation is private: `Inline`'s `len` counts the live bytes at
/// the front of `buf` and nothing may read past it, so a caller able to write
/// that field could produce a slice over bytes no one ever set. Everything
/// outside this module goes through [`Bytes::from_slice`] or
/// [`Bytes::from_vec`], which is why the invariant is checkable by reading
/// this file alone.
#[derive(Clone)]
pub struct Bytes(Repr);

#[derive(Clone)]
enum Repr {
    Inline { len: u8, buf: [u8; INLINE_BYTES] },
    Heap(Vec<u8>),
}

impl Bytes {
    /// A copy of `source`, inline when it fits.
    pub fn from_slice(source: &[u8]) -> Bytes {
        if source.len() <= INLINE_BYTES {
            let mut buf = [0u8; INLINE_BYTES];
            buf[..source.len()].copy_from_slice(source);
            Bytes(Repr::Inline {
                len: source.len() as u8,
                buf,
            })
        } else {
            Bytes(Repr::Heap(source.to_vec()))
        }
    }

    /// [`from_slice`], for a caller that already owns the bytes.
    ///
    /// **A buffer longer than [`INLINE_BYTES`] is taken, never copied**, which
    /// is the guarantee `rexx-exec`'s `Interp::text_owned` is built on: a
    /// builtin sizing its result from user input reserves fallibly and then
    /// hands the reservation over, and a copy would be a second allocation of
    /// the same size that no `try_reserve` can catch. A buffer that fits
    /// inline is copied and freed here, which is one allocation released
    /// early rather than one added.
    ///
    /// [`from_slice`]: Bytes::from_slice
    pub fn from_vec(source: Vec<u8>) -> Bytes {
        if source.len() <= INLINE_BYTES {
            Bytes::from_slice(&source)
        } else {
            Bytes(Repr::Heap(source))
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        match &self.0 {
            Repr::Inline { len, buf } => &buf[..*len as usize],
            Repr::Heap(v) => v,
        }
    }

    /// Whether these bytes are held inline rather than in a separate
    /// allocation.
    ///
    /// The representation is not observable through any other method on
    /// purpose -- two `Bytes` holding the same bytes on different arms are the
    /// same value. This exists so a test can assert *which* arm a length
    /// reaches, which is the whole content of the capacity claim and is
    /// otherwise invisible.
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
    ///
    /// **Without this the whole change is invisible to the suite**: every
    /// other assertion passes with the inline arm deleted, because a `Bytes`
    /// that is always `Heap` answers every byte correctly and is exactly
    /// today's `Vec<u8>`.
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

    #[test]
    fn debug_shows_the_live_bytes_and_not_the_padding() {
        assert_eq!(format!("{:?}", Bytes::from_slice(b"ab")), "[97, 98]");
        assert_eq!(format!("{:?}", Bytes::from_slice(b"")), "[]");
    }
}
