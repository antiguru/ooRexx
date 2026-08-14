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

//! A [`Number`]'s decimal digits, held inline while they are few.
//!
//! One byte per decimal digit is what the interpreter also stores, but it
//! stores them *in* the object: `NumberString` ends in `char numberDigits[4]`,
//! a trailing flexible array, and arithmetic runs in a `char
//! resultBufFast[FAST_BUFFER]` stack buffer with `FAST_BUFFER = 48`, so a
//! small number costs no separate allocation at all. A `Vec<u8>` per `Number`
//! costs one, plus the `free` that matches it and a fresh pair for every
//! clone.
//!
//! [`Digits`] is that answer in safe Rust: an inline array for the small case
//! and a `Vec<u8>` for the rest. It derefs to `[u8]`, so every read is the
//! slice operation it always was; only construction and the few in-place
//! edits go through methods here.
//!
//! [`Number`]: crate::Number

/// How many decimal digits fit without a heap allocation.
///
/// **Two bounds from the language set this, and it is above both.**
///
/// * `NUMERIC DIGITS` defaults to 9. Every operator truncates its operands to
///   `working_length(digits)` -- `digits + 1`, so ten -- and an addition can
///   carry into an eleventh digit, so eleven is the widest intermediate the
///   default setting produces.
/// * [`Number::from_i64`] is the door a machine integer takes into this
///   crate, and an `i64` is at most nineteen decimal digits. That function's
///   own reservation already writes nineteen down, rounded to twenty.
///
/// **What bounds it above is layout, not a benchmark.** Measured with
/// `size_of` over candidate capacities: at twenty, `Digits` is 32 bytes and
/// `Number` 40; at thirty-one, `Number` reaches 48. `rexx-core`'s `Body::Num`
/// holds a `Number` beside a `u32`, a `Form` and an `Option<Vec<u8>>`, and
/// `Body`'s width is set by its widest variant -- so a `Number` past 40 bytes
/// widens every arena slot, and one at 40 does not. `the_capacity_is_the_one
/// _the_language_asks_for` holds both ends of that.
///
/// [`Number::from_i64`]: crate::Number::from_i64
/// **Thirty rather than twenty, and the difference is free.** `Digits` is an
/// enum over a 24-byte `Vec`, so its width is set by that arm until the
/// inline buffer passes it: measured with `size_of`, every capacity from
/// twenty to thirty gives `Digits` 32 bytes and `Number` 40, and thirty-one
/// is where `Number` reaches 48. Twenty spilled every intermediate that ran
/// one digit past it, and at `NUMERIC DIGITS 20` -- which works at
/// `digits + 1` -- that is every kept product and every division working
/// value.
pub(crate) const INLINE_DIGITS: usize = 30;

/// Most significant digit first, each value 0..=9 -- the same contract
/// [`Number::digits`] always had, with the storage decided by length.
///
/// `Inline`'s `len` is the number of live bytes at the front of `buf`;
/// nothing reads past it, and every method that extends the live region
/// writes the bytes it exposes rather than trusting what was left there.
///
/// [`Number::digits`]: crate::Number
#[derive(Clone)]
pub(crate) enum Digits {
    Inline { len: u8, buf: [u8; INLINE_DIGITS] },
    Heap(Vec<u8>),
}

impl Digits {
    /// No digits at all -- a builder's starting point, never a `Number`'s
    /// stored state.
    pub(crate) const fn new() -> Self {
        Digits::Inline {
            len: 0,
            buf: [0; INLINE_DIGITS],
        }
    }

    /// A copy of `source`, inline when it fits.
    pub(crate) fn from_slice(source: &[u8]) -> Self {
        if source.len() <= INLINE_DIGITS {
            let mut buf = [0u8; INLINE_DIGITS];
            buf[..source.len()].copy_from_slice(source);
            Digits::Inline {
                len: source.len() as u8,
                buf,
            }
        } else {
            Digits::Heap(source.to_vec())
        }
    }

    /// `count` zero digits, which is what an aligned addition or a
    /// multiplication accumulator starts from.
    pub(crate) fn zeros(count: usize) -> Self {
        if count <= INLINE_DIGITS {
            Digits::Inline {
                len: count as u8,
                buf: [0; INLINE_DIGITS],
            }
        } else {
            Digits::Heap(vec![0; count])
        }
    }

    /// One digit, which is every canonical zero and every single-digit
    /// mantissa.
    pub(crate) fn single(digit: u8) -> Self {
        let mut buf = [0u8; INLINE_DIGITS];
        buf[0] = digit;
        Digits::Inline { len: 1, buf }
    }

    pub(crate) fn as_slice(&self) -> &[u8] {
        match self {
            Digits::Inline { len, buf } => &buf[..*len as usize],
            Digits::Heap(v) => v,
        }
    }

    fn as_mut_slice(&mut self) -> &mut [u8] {
        match self {
            Digits::Inline { len, buf } => &mut buf[..*len as usize],
            Digits::Heap(v) => v,
        }
    }

    /// Moves the live digits to the heap, reserving room for `extra` more.
    ///
    /// A `Digits` never comes back from the heap once it has been here: the
    /// allocation is already paid, and re-inlining a shrinking vector would
    /// free and re-allocate for a value that is about to grow again.
    fn spill(&mut self, extra: usize) {
        if let Digits::Inline { len, buf } = self {
            let live = &buf[..*len as usize];
            let mut v = Vec::with_capacity(live.len() + extra);
            v.extend_from_slice(live);
            *self = Digits::Heap(v);
        }
    }

    pub(crate) fn push(&mut self, digit: u8) {
        if let Digits::Inline { len, buf } = self
            && (*len as usize) < INLINE_DIGITS
        {
            buf[*len as usize] = digit;
            *len += 1;
            return;
        }
        self.spill(1);
        match self {
            Digits::Heap(v) => v.push(digit),
            // `spill` leaves every `Digits` on the heap, and the arm above
            // returned for every inline case with room.
            Digits::Inline { .. } => unreachable!("spill leaves the heap arm"),
        }
    }

    pub(crate) fn pop(&mut self) -> Option<u8> {
        match self {
            Digits::Inline { len, buf } => {
                let last = len.checked_sub(1)?;
                *len = last;
                Some(buf[last as usize])
            }
            Digits::Heap(v) => v.pop(),
        }
    }

    pub(crate) fn truncate(&mut self, keep: usize) {
        match self {
            Digits::Inline { len, .. } => {
                if keep < *len as usize {
                    *len = keep as u8;
                }
            }
            Digits::Heap(v) => v.truncate(keep),
        }
    }

    /// Prepends `digit`, which is the carry an all-nines rounding grows.
    pub(crate) fn insert_front(&mut self, digit: u8) {
        if let Digits::Inline { len, buf } = self
            && (*len as usize) < INLINE_DIGITS
        {
            buf.copy_within(..*len as usize, 1);
            buf[0] = digit;
            *len += 1;
            return;
        }
        self.spill(1);
        match self {
            Digits::Heap(v) => v.insert(0, digit),
            Digits::Inline { .. } => unreachable!("spill leaves the heap arm"),
        }
    }

    /// Drops `count` digits from the front, which is what stripping leading
    /// zeros does.
    pub(crate) fn drop_front(&mut self, count: usize) {
        match self {
            Digits::Inline { len, buf } => {
                let count = count.min(*len as usize);
                buf.copy_within(count..*len as usize, 0);
                *len -= count as u8;
            }
            Digits::Heap(v) => {
                v.drain(..count.min(v.len()));
            }
        }
    }

    /// Appends `count` zero digits, which is what aligning two operands on
    /// their least significant digit does to the shorter one.
    pub(crate) fn extend_zeros(&mut self, count: usize) {
        if let Digits::Inline { len, buf } = self
            && *len as usize + count <= INLINE_DIGITS
        {
            let end = *len as usize + count;
            // Written rather than assumed zero: a `truncate` or a `pop` can
            // have left a live digit at these positions.
            buf[*len as usize..end].fill(0);
            *len = end as u8;
            return;
        }
        self.spill(count);
        match self {
            Digits::Heap(v) => v.resize(v.len() + count, 0),
            Digits::Inline { .. } => unreachable!("spill leaves the heap arm"),
        }
    }
}

impl std::ops::Deref for Digits {
    type Target = [u8];

    fn deref(&self) -> &[u8] {
        self.as_slice()
    }
}

impl std::ops::DerefMut for Digits {
    fn deref_mut(&mut self) -> &mut [u8] {
        self.as_mut_slice()
    }
}

/// By value, never by storage: an inline `[1, 2]` and a heap `[1, 2]` are the
/// same digits, and `Number`'s own `PartialEq` -- which the crate's tests and
/// `rexx-exec` both use -- would otherwise answer on where the bytes live.
impl PartialEq for Digits {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl Eq for Digits {}

/// As the slice, for the same reason `PartialEq` is: `Number`'s derived
/// `Debug` reaches this, and which arm holds the digits is not part of the
/// value.
impl std::fmt::Debug for Digits {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(self.as_slice(), f)
    }
}

/// So that `for d in &digits` reads the way it did when this was a `Vec`.
/// `Deref` alone does not reach `IntoIterator`, which is resolved on the
/// receiver's own type rather than through an auto-deref.
impl<'a> IntoIterator for &'a Digits {
    type Item = &'a u8;
    type IntoIter = std::slice::Iter<'a, u8>;

    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().iter()
    }
}

impl FromIterator<u8> for Digits {
    fn from_iter<I: IntoIterator<Item = u8>>(iter: I) -> Self {
        let mut out = Digits::new();
        for digit in iter {
            out.push(digit);
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::{Digits, INLINE_DIGITS};
    use crate::{CompareOp, DivOp, Form, Number};

    /// The same value with its digits forced onto the heap.
    ///
    /// Which arm holds a number's digits is not supposed to be observable in
    /// any answer, and that is a claim about every operation rather than
    /// about the container -- so it is checked by running the operations
    /// twice rather than by inspecting the container twice.
    fn heaped(n: &Number) -> Number {
        Number {
            negative: n.negative,
            digits: Digits::Heap(n.digits.as_slice().to_vec()),
            exponent: n.exponent,
        }
    }

    /// Values chosen for where this representation can break: either side of
    /// the inline capacity, the all-nines carry that grows a digit, and the
    /// leading zero a subtraction's borrow leaves.
    fn population() -> Vec<Number> {
        let mantissas = [
            "0",
            "1",
            "9",
            "12",
            "123456789",
            "999999999",
            "1234567890123456789",
            "9999999999999999999",
            "12345678901234567890",
            "99999999999999999999",
            "123456789012345678901",
            "999999999999999999999",
            "1000000000000000000000000",
            "100000000000000000000000000000000009",
        ];
        let mut out = Vec::new();
        for m in mantissas {
            for exponent in ["", "E-25", "E-7", "E-1", "E+1", "E+7", "E+25"] {
                for sign in ["", "-"] {
                    if let Some(n) = Number::parse(&format!("{sign}{m}{exponent}")) {
                        out.push(n);
                    }
                }
            }
        }
        out
    }

    /// Every operation, on both arms, over the boundary population.
    ///
    /// This is the test the inline arm exists to be held to: it hand-shifts
    /// bytes where a `Vec` called into the standard library, and the failure
    /// that produces is a wrong digit rather than a crash.
    #[test]
    fn every_operation_answers_the_same_with_the_digits_on_the_heap() {
        let values = population();
        let precisions = [1u64, 2, 9, 19, 20, 21, 25, 40, 999];
        let mut checked = 0usize;
        let mut reached_heap = 0usize;

        for a in &values {
            let ha = heaped(a);
            if matches!(a.digits, Digits::Heap(_)) {
                reached_heap += 1;
            }
            for digits in precisions {
                assert_eq!(a.format(digits), ha.format(digits));
                assert_eq!(
                    a.format_form(digits, Form::Engineering),
                    ha.format_form(digits, Form::Engineering)
                );
                assert_eq!(a.round_to(digits), ha.round_to(digits));
                assert_eq!(a.plain_integer(digits), ha.plain_integer(digits));
                assert_eq!(a.rendered_integer(digits), ha.rendered_integer(digits));
                assert_eq!(a.abs(), ha.abs());
                assert_eq!(a.signum(), ha.signum());

                for b in &values {
                    let hb = heaped(b);
                    assert_eq!(a.add(b, digits), ha.add(&hb, digits));
                    assert_eq!(a.sub(b, digits), ha.sub(&hb, digits));
                    assert_eq!(a.mul(b, digits), ha.mul(&hb, digits));
                    for op in [DivOp::Divide, DivOp::IntegerDivide, DivOp::Remainder] {
                        assert_eq!(a.div(b, digits, op), ha.div(&hb, digits, op));
                    }
                    assert_eq!(
                        crate::compare_decoded(
                            b"",
                            Some(a),
                            b"",
                            Some(b),
                            digits,
                            0,
                            CompareOp::Less
                        ),
                        crate::compare_decoded(
                            b"",
                            Some(&ha),
                            b"",
                            Some(&hb),
                            digits,
                            0,
                            CompareOp::Less
                        )
                    );
                    checked += 1;
                }
            }
        }

        // `pow` separately and at small exponents: it squares and multiplies,
        // so a large one is the same code many times over at the cost of
        // making this test a benchmark.
        for a in &values {
            let ha = heaped(a);
            for e in ["0", "1", "2", "3", "-1", "-2"] {
                let exponent = Number::parse(e).expect("a literal exponent");
                for digits in [9u64, 20, 21] {
                    assert_eq!(a.pow(&exponent, digits), ha.pow(&exponent, digits));
                    checked += 1;
                }
            }
        }

        // Floors, so a population that stopped generating cases -- or one
        // that never left the inline arm -- fails rather than passes empty.
        assert!(checked > 100_000, "only {checked} case(s)");
        assert!(reached_heap > 0, "no case exercised the heap arm");
    }

    /// The boundary crossed *inside* one computation rather than between
    /// two: operands that fit inline whose product does not, and the way
    /// back down.
    #[test]
    fn a_computation_that_crosses_the_capacity_agrees_with_the_heap() {
        let wide = Number::parse("99999999999999999999").expect("twenty nines");
        let narrow = Number::parse("7").expect("a digit");
        for digits in [9u64, 19, 20, 21, 45] {
            let up = wide.mul(&wide, digits).expect("in range");
            let back = up.div(&wide, digits, DivOp::Divide).expect("in range");
            let h_up = heaped(&wide).mul(&heaped(&wide), digits).expect("in range");
            let h_back = h_up
                .div(&heaped(&wide), digits, DivOp::Divide)
                .expect("in range");
            assert_eq!(up, h_up);
            assert_eq!(back, h_back);
            assert_eq!(up.format(digits), h_up.format(digits));

            // Growing one digit at a time across the capacity, which is the
            // carry path `insert_front` serves.
            let mut acc = narrow.clone();
            let mut h_acc = heaped(&narrow);
            for _ in 0..30 {
                acc = acc
                    .mul(&Number::parse("10").expect("ten"), 999)
                    .expect("in range");
                acc = acc.add(&narrow, 999).expect("in range");
                h_acc = h_acc
                    .mul(&Number::parse("10").expect("ten"), 999)
                    .expect("in range");
                h_acc = h_acc.add(&heaped(&narrow), 999).expect("in range");
                assert_eq!(acc, h_acc);
                assert_eq!(acc.format(999), h_acc.format(999));
            }
        }
    }

    /// Both ends of the capacity choice, so that moving it is a deliberate
    /// act with a failing test behind it rather than an edit to a constant.
    #[test]
    fn the_capacity_is_the_one_the_language_asks_for() {
        // `NUMERIC DIGITS 9` works at `digits + 1` and can carry one more.
        let widest_default_intermediate = crate::working_length(crate::DEFAULT_DIGITS) + 1;
        assert!(INLINE_DIGITS >= widest_default_intermediate);
        // Every `i64` -- `from_i64` is how a tagged small integer arrives.
        assert!(INLINE_DIGITS >= i64::MIN.to_string().trim_start_matches('-').len());
        // The layout ceiling: past 40 bytes a `Number` widens `rexx-core`'s
        // `Body` and with it every arena slot.
        assert!(size_of::<Number>() <= 40);
        // The capacity sits at that ceiling rather than below it: `Digits`
        // is 32 bytes for every capacity up to thirty, so anything smaller
        // spills sooner for nothing.
        assert_eq!(size_of::<Digits>(), 32);
    }

    #[test]
    fn a_long_value_spills_to_the_heap_and_reads_back_the_same() {
        let long: Vec<u8> = (0..INLINE_DIGITS as u8 + 7).map(|i| i % 10).collect();
        let mut d = Digits::new();
        for &digit in &long {
            d.push(digit);
        }
        assert!(matches!(d, Digits::Heap(_)));
        assert_eq!(&*d, &long[..]);
        assert_eq!(d, Digits::from_slice(&long));
    }

    #[test]
    fn the_same_digits_compare_equal_whichever_arm_holds_them() {
        let short = [1u8, 2, 3];
        let inline = Digits::from_slice(&short);
        let heap = Digits::Heap(short.to_vec());
        assert!(matches!(inline, Digits::Inline { .. }));
        assert_eq!(inline, heap);
        assert_eq!(format!("{inline:?}"), format!("{heap:?}"));
    }

    /// The edits that move digits about, run on both arms with the same
    /// expected answer, because an inline arm that shifts bytes by hand is
    /// where this representation can differ from a `Vec`.
    #[test]
    fn the_in_place_edits_agree_with_a_vector_on_both_arms() {
        for start in [8usize, INLINE_DIGITS + 5] {
            let source: Vec<u8> = (0..start as u8).map(|i| i % 10).collect();

            let mut d = Digits::from_slice(&source);
            let mut v = source.clone();
            d.insert_front(7);
            v.insert(0, 7);
            assert_eq!(&*d, &v[..]);

            d.drop_front(3);
            v.drain(..3);
            assert_eq!(&*d, &v[..]);

            assert_eq!(d.pop(), v.pop());
            assert_eq!(&*d, &v[..]);

            d.truncate(2);
            v.truncate(2);
            assert_eq!(&*d, &v[..]);

            // After a truncate the bytes beyond `len` are still live in the
            // inline buffer, which is the case `extend_zeros` has to write
            // rather than assume.
            d.extend_zeros(4);
            v.resize(v.len() + 4, 0);
            assert_eq!(&*d, &v[..]);
        }
    }

    #[test]
    fn a_slice_at_the_boundary_inlines_and_one_past_it_does_not() {
        let fits = vec![1u8; INLINE_DIGITS];
        let over = vec![1u8; INLINE_DIGITS + 1];
        assert!(matches!(Digits::from_slice(&fits), Digits::Inline { .. }));
        assert!(matches!(Digits::from_slice(&over), Digits::Heap(_)));
    }
}
