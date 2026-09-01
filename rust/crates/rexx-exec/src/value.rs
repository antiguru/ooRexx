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

//! The value model (D15): four functions every later task manipulates a
//! value through, and the two rules the oracle makes observable everywhere.
//!
//! 1. **A number's rendering is fixed when the number is created**, under the
//!    `DIGITS`/`FORM` in force at that moment, and never afterwards. So
//!    `number` takes `created_digits`/`created_form` as explicit arguments
//!    rather than reading them from anywhere ambient: the operation that
//!    produces a `Number` is what supplies the pair, and `to_text` formats
//!    through the value's own captured pair, never a current one. There is
//!    deliberately no "current settings" in this module at all.
//! 2. **`ObjRef::SmallInt` is admissible only when the exact result is
//!    whole, inside the tag's range, and no wider than the `DIGITS` that
//!    produced it**, checked once at creation in `number` and never
//!    re-derived.
//!
//! `text`/`number` construct a value; `to_text`/`to_number` read one back.
//! Every conversion is total: `to_text` always produces bytes (`.nil` has a
//! string value, `The NIL object`), and `to_number`'s only failure is
//! `NotNumeric` -- one marker, because nothing observable distinguishes why a
//! byte string is not a number (`rexx-core`'s own doc comment on
//! `NotNumeric`).

use crate::Interp;
use rexx_core::{
    BehaviourId, Body, Bytes, Decoded, INLINE_BYTES, InlineText, NotNumeric, ObjRef, SMALL_INT_MAX,
    SMALL_INT_MIN,
};
use rexx_num::{Form, Number};
use std::borrow::Cow;

/// Parses already answered for values whose bytes live in the handle.
///
/// **`Body::Text` carries a `num` cache and a handle-inline string has
/// nowhere to put one** -- `rexx_core::InlineText`'s own doc comment states
/// that limitation, and this is the home it lacks: on the interpreter rather
/// than on the value, because the value is a `Copy` sixty-four bits with no
/// object behind it.
///
/// **An entry cannot go stale, and that is a property of the key rather than
/// a discipline.** A handle carrying inline text *is* its bytes -- the tag,
/// the length and the bytes themselves are the whole of it, with no slot or
/// generation to be recycled -- so two handles comparing equal name the same
/// byte string, and the same byte string parses to the same `Number` forever.
/// `NUMERIC DIGITS` does not enter: [`Interp::to_number`] never rounds, which
/// is what lets one parse answer at every precision.
///
/// Direct-mapped and fixed-size, so a lookup is a mix, a load and a compare,
/// and a collision costs only the parse that would have happened anyway.
/// Measured on `samples/rexxcps.rex`, which converts a handle-inline string
/// to a `Number` 2,790,002 times over 24 distinct strings: thirty-two entries
/// answer 2,399,989 of those without parsing.
pub(crate) struct TextNumbers {
    /// [`ObjRef::NIL`] marks an empty slot. No key stored here can equal it:
    /// every one carries the inline-text tag, which `.nil` does not.
    keys: [ObjRef; TEXT_NUMBERS],
    parsed: [Result<Number, NotNumeric>; TEXT_NUMBERS],
}

/// How many parses [`TextNumbers`] remembers. A power of two, so the slot is
/// a mask rather than a division.
const TEXT_NUMBERS: usize = 32;

impl TextNumbers {
    pub(crate) fn new() -> Self {
        TextNumbers {
            keys: [ObjRef::NIL; TEXT_NUMBERS],
            parsed: std::array::from_fn(|_| Err(NotNumeric)),
        }
    }

    /// Which slot `value` maps to.
    ///
    /// A multiply-xor-multiply finaliser over the handle's bits, whose low
    /// bits then index. The bits themselves index badly: a handle's tag and
    /// length sit in the low byte and short strings differ only in the bytes
    /// above them. Measured over `rexxcps`' own conversions, taking the top
    /// bits of a single multiply answers 1,679,994 of them from the table
    /// where this answers 2,399,989.
    fn slot(value: ObjRef) -> usize {
        let mut mixed = value.bits().wrapping_mul(0xff51_afd7_ed55_8ccd);
        mixed ^= mixed >> 33;
        mixed = mixed.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
        (mixed as usize) & (TEXT_NUMBERS - 1)
    }
}

/// How wide [`Interp::text_scratch`] is.
///
/// Twenty because that is `i64::MIN`'s rendering, sign included, which is the
/// widest thing written there; the assertions below are what say so, rather
/// than this sentence. `INLINE_TEXT` is the other writer and is far shorter.
pub(crate) const TEXT_SCRATCH: usize = 20;
const _: () = assert!(TEXT_SCRATCH >= rexx_core::INLINE_TEXT);

/// Writes `value`'s decimal spelling into the tail of `buffer`, answering the
/// index it starts at.
///
/// **The one spelling of a tagged integer's rendering**, and it must stay
/// that way: [`Interp::write_text`] appends it, [`Interp::to_text`] leaves it
/// in the interpreter's scratch, and [`Interp::render`] puts it in the
/// `Rendered` it hands back. Those three must agree byte for byte or a value
/// would read differently depending on which accessor asked.
///
/// Right-aligned because that is the direction the digits come out in, and
/// `unsigned_abs` rather than a negation because `i64::MIN`'s magnitude has
/// no positive form. `a_tagged_integer_renders_as_its_own_display_does` pins
/// the result against `to_string` at both ends of the tagged range.
///
/// Written here rather than through `i64`'s `Display`: that would be the
/// contract stated directly, but it reaches this buffer only through
/// `core::fmt`, and measured on samples/rexxcps.rex a `write!` into a
/// fixed-size `fmt::Write` sink cost 0.85% more instructions than the
/// `to_string` allocation it was replacing -- `to_string` has a specialised
/// integer path that `write!` does not take. The test is what keeps the two
/// spellings honest instead.
/// A heap number's display bytes, filled into the object's own cache on the
/// first ask and reused forever after -- the `created_digits`/`created_form`
/// pair is fixed at creation, so the rendering is a pure function of the
/// object and cannot go stale.
///
/// **The one spelling of a heap number's rendering**, for the same reason
/// [`write_small_int`] is the one spelling of a tagged integer's:
/// [`Interp::to_text`] borrows these bytes out and [`Interp::text_len`]
/// measures them, and a second reading of the precision/form pair is a
/// number that could display differently depending on which accessor asked.
/// The two cannot be held apart by a test, either -- whichever ran first
/// would fill the cache the other then agrees with, which is measured: with
/// the two written out separately, mutating `text_len`'s copy to ignore
/// `created_digits` left `text_len_agrees_with_to_text` green.
fn num_rendering<'a>(
    value: &Number,
    created_digits: u32,
    created_form: Form,
    text: &'a mut Option<Vec<u8>>,
) -> &'a [u8] {
    text.get_or_insert_with(|| {
        value
            .format_form(u64::from(created_digits), created_form)
            .into_bytes()
    })
}

fn write_small_int(buffer: &mut [u8; TEXT_SCRATCH], value: i64) -> usize {
    let mut at = buffer.len();
    let mut magnitude = value.unsigned_abs();
    loop {
        at -= 1;
        buffer[at] = b'0' + (magnitude % 10) as u8;
        magnitude /= 10;
        if magnitude == 0 {
            break;
        }
    }
    if value < 0 {
        at -= 1;
        buffer[at] = b'-';
    }
    at
}

impl Interp {
    /// Creates a text value: D15's "a value whose identity is its bytes".
    ///
    /// The `num` cache starts at `None`, meaning "not yet asked" and nothing
    /// stronger -- it must not be read as "not a number" for a value nobody
    /// has converted yet, which is the whole reason the cache is a tri-state
    /// rather than a plain `Option<Number>`.
    ///
    /// Builds the `Bytes` from the slice directly rather than going through
    /// [`text_owned`]: a short string is copied into the slot and never
    /// allocates at all, where a `to_vec` first would allocate a buffer only
    /// to free it again.
    ///
    /// **The handle test comes before the `Bytes`, not after it.**
    /// [`text_bytes`] makes the same test, but by then the bytes have been
    /// copied into a `Bytes` whose inline arm is `INLINE_BYTES` wide, and a
    /// value that fits in the handle needs no storage at all -- so that copy
    /// is written and discarded. [`text_bytes`]' own doc has how often: the
    /// handle takes about two thirds to three quarters of the strings this
    /// interpreter creates. [`text_owned`] makes the same test in front of its
    /// own `Bytes`, so [`text_bytes`] asserts the answer rather than asking
    /// for it a third time.
    ///
    /// [`text_owned`]: Interp::text_owned
    /// [`text_bytes`]: Interp::text_bytes
    #[inline]
    pub(crate) fn text(&mut self, bytes: &[u8]) -> ObjRef {
        if let Some(inline) = ObjRef::inline_text(bytes) {
            return inline;
        }
        self.text_bytes(Bytes::from_slice(bytes))
    }

    /// A source literal's value, inlined as a tagged integer when the
    /// literal's own bytes are already exactly what that integer renders as,
    /// and a heap string otherwise.
    ///
    /// A Rexx literal keeps its source spelling -- `05` prints `05` and `5.0`
    /// prints `5.0` -- so only the canonical rendering is eligible, which is
    /// what [`canonical_small_int`] decides.
    ///
    /// This is not a new kind of value. `number` already inlines every
    /// arithmetic result that renders as a small integer, so a `SmallInt`
    /// reaches every consumer in the crate already; what changes here is only
    /// which of two existing representations a literal starts in.
    pub(crate) fn literal(&mut self, bytes: &[u8]) -> ObjRef {
        // Allocation is the thing being avoided, so the eligibility test may
        // not render a candidate to compare it -- rendering allocates the
        // string this exists to skip. The round trip is therefore stated as a
        // byte pattern and checked, rather than performed.
        if let Some(value) = canonical_small_int(bytes)
            && let Some(handle) = ObjRef::small_int(value)
        {
            return handle;
        }
        self.text(bytes)
    }

    /// [`literal`], for a caller that will keep the handle and read it for the
    /// rest of the run.
    ///
    /// **The value is allocated where the collector cannot take it and is
    /// shared by every later read**, so the caller stores the handle instead
    /// of rebuilding the value. The two arms `literal` answers with need
    /// nothing done to them -- a tagged small integer and a handle-inline
    /// string are the value, with no object behind either -- so only the arm
    /// that would allocate is interned.
    ///
    /// **Sharing one object across every execution of a constant is safe, and
    /// the reason is narrow enough to state.** The only in-place mutation of a
    /// `Body::Text` in this crate is its lazy `num` cache, filled by
    /// `Number::parse_bytes` from the object's own bytes: the same input gives
    /// the same answer, and `NUMERIC DIGITS` is not among its inputs --
    /// rounding happens where the arithmetic does, against the settings in
    /// force there. So the sharing is invisible except that the parse now
    /// happens once for the whole program rather than once per execution.
    ///
    /// [`literal`]: Interp::literal
    pub(crate) fn interned_literal(&mut self, bytes: &[u8]) -> ObjRef {
        if let Some(value) = canonical_small_int(bytes)
            && let Some(handle) = ObjRef::small_int(value)
        {
            return handle;
        }
        if let Some(inline) = ObjRef::inline_text(bytes) {
            return inline;
        }
        self.alloc_immortal_with(
            BehaviourId::STRING,
            Body::Text {
                bytes: Bytes::from_slice(bytes),
                num: None,
            },
        )
    }

    /// A builtin's counted result -- a length, a position, an index or a
    /// count -- as the inline tagged integer rather than as a heap string of
    /// its own decimal digits.
    ///
    /// The two are the same value in the two representations D15 already has,
    /// for the reason [`literal`] gives: a `SmallInt` renders through `i64`'s
    /// `Display`, which is byte for byte what `value.to_string()` produces, so
    /// the heap string this replaces was a second spelling of what the tag
    /// says. Unlike [`number`] there is no `DIGITS` admissibility test to
    /// make, and that is not an omission: a count is exact and integral by
    /// construction, nothing rounded it, and the interpreter renders such a
    /// result in full whatever `DIGITS` is in force -- measured, `numeric
    /// digits 3 ; say length(copies('a',1234))` is `1234` and only
    /// `length(...) + 0` is `1.23E+3`.
    ///
    /// What it saves is the render, the copy and the slot; what it saves at
    /// the *consumer* is larger, because [`Interp::arith_small_int`] needs
    /// both operands tagged and a heap operand alone sends the clause down the
    /// general decimal path.
    ///
    /// The fallback cannot be reached by a count derived from a byte length --
    /// `SMALL_INT_MAX` is 2^61 - 1 -- and is written rather than asserted
    /// because a total function is cheaper here than a proof.
    ///
    /// [`literal`]: Interp::literal
    /// [`number`]: Interp::number
    pub(crate) fn counted(&mut self, value: usize) -> ObjRef {
        if let Ok(value) = i64::try_from(value)
            && let Some(handle) = ObjRef::small_int(value)
        {
            return handle;
        }
        self.counted_text(value)
    }

    /// A count as a *text* value, for a caller that wants the digits rather
    /// than the tagged integer [`Interp::counted`] prefers.
    ///
    /// **Rendered into a stack buffer**, so the digits cost no allocation of
    /// their own: `to_string` would build a `String`, copy it into the value
    /// and free it again, which for a value this short is the whole cost.
    /// The width [`write_small_int`] needs is what `TEXT_SCRATCH` is sized
    /// for.
    pub(crate) fn counted_text(&mut self, value: usize) -> ObjRef {
        match i64::try_from(value) {
            Ok(value) => {
                let mut digits = [0u8; TEXT_SCRATCH];
                let at = write_small_int(&mut digits, value);
                self.text(&digits[at..])
            }
            // Unreachable on a machine whose `usize` is 64 bits wide, and
            // written rather than asserted for `counted`'s own reason.
            Err(_) => self.text(value.to_string().as_bytes()),
        }
    }

    /// [`text`], for a caller that already owns the bytes.
    ///
    /// **The copy `text` makes is a second allocation of the result's full
    /// size, and for a result whose size comes from an argument that is the
    /// difference between running and aborting.** A `Vec` that cannot be
    /// allocated aborts the process rather than returning an error, so a
    /// builtin sizing its result from user input has to reserve fallibly
    /// (`builtin::buffer`) *and* hand the reservation over
    /// rather than have it copied. Measured at the project's own
    /// `ulimit -v 1048576`: `say length(copies('a',400000000))` needs 400 MB
    /// once, which fits, and twice, which does not.
    ///
    /// **A result at or under `INLINE_BYTES` is copied into the slot and the
    /// caller's buffer freed here**, which is one allocation released early
    /// rather than one added; the guarantee above is about the results that
    /// are large, which are exactly the ones that stay on the heap arm.
    ///
    /// [`text`]: Interp::text
    /// Takes the shared builtin-result buffer, empty and ready to build into.
    pub(crate) fn take_result_buffer(&self) -> Vec<u8> {
        let mut buffer = self.result_buffer.take();
        buffer.clear();
        buffer
    }

    /// Hands the result buffer back for the next builtin.
    pub(crate) fn give_result_buffer(&self, buffer: Vec<u8>) {
        self.result_buffer.set(buffer);
    }

    /// A builtin's finished result, keeping the buffer it was built in when
    /// keeping it is free.
    ///
    /// **The branch is `Bytes::from_vec`'s own.** At `INLINE_BYTES` or less it
    /// copies into the object and drops the `Vec`, so the buffer is handed
    /// back here instead and the next builtin builds in it. Above that the
    /// `Vec` becomes the object's own storage, so it is given up and the next
    /// taker allocates -- which is the same trade `text_owned` always made.
    /// A builtin's finished result when the interpreter builds an *integer
    /// object* for it: the `SmallInt` tag when those exact bytes are what the
    /// tag renders back, and [`Interp::text_built`]'s own answer otherwise.
    ///
    /// **Which class a builtin builds is a comparison's business, not just its
    /// rendering's.** `RexxInteger::comp`
    /// (`interpreter/classes/IntegerClass.cpp:1191`) compares two integer
    /// objects inside `NUMERIC DIGITS` exactly and never consults `NUMERIC
    /// FUZZ`, so a builtin whose result is one answers differently from a
    /// string spelling the same digits. Measured at `DIGITS 9 FUZZ 8`,
    /// `trunc(100000000.4) = 100000001` is `0` while
    /// `format(100000000) = 100000001`, whose result the interpreter builds as
    /// a string, is `1`.
    ///
    /// This crate's tag is admitted on the rendering instead (D15), so the
    /// same set is reached by asking whether the bytes round-trip through
    /// `i64`: that refuses `+5`, `007` and anything with a point or an
    /// exponent, none of which the tag could render back.
    pub(crate) fn integer_text(&mut self, bytes: Vec<u8>, digits: u64) -> ObjRef {
        if let Some(handle) = rendered_small_int(&bytes, digits) {
            self.give_result_buffer(bytes);
            return handle;
        }
        self.text_built(bytes)
    }

    pub(crate) fn text_built(&mut self, bytes: Vec<u8>) -> ObjRef {
        if bytes.len() <= INLINE_BYTES {
            let value = self.text(&bytes);
            self.give_result_buffer(bytes);
            return value;
        }
        self.text_owned(bytes)
    }

    pub(crate) fn text_owned(&mut self, bytes: Vec<u8>) -> ObjRef {
        // **The handle test in front of the `Bytes`**, for the reason
        // [`Interp::text`] gives: a value short enough to live in the handle
        // needs no storage, and `Bytes::from_vec` would copy it into a
        // 54-byte inline buffer that is then discarded. The median string
        // this interpreter builds is three bytes long.
        if let Some(inline) = ObjRef::inline_text(&bytes) {
            return inline;
        }
        self.text_bytes(Bytes::from_vec(bytes))
    }

    /// The one place a `Body::Text` is built, so the `num` cache's initial
    /// state is stated once.
    ///
    /// **A string short enough never reaches here.** `ObjRef::inline_text`
    /// carries up to `INLINE_TEXT` bytes in the handle, so those values cost
    /// no slot, no mark byte and no sweep. Measured over the benchmark axes,
    /// that is about two thirds to three quarters of every string this
    /// interpreter creates -- the length distribution is bimodal, a median of
    /// three bytes with the next cluster far above the inline capacity.
    /// **Both callers make that test in front of the `Bytes` rather than
    /// behind it**, which is what this function no longer repeating it says:
    /// a `Bytes` built for a value that fits the handle copies the bytes into
    /// a 54-byte inline buffer and then discards the whole thing.
    ///
    /// The one thing given up is the `num` cache: a handle is `Copy`, so
    /// there is no shared mutable home for a lazy parse. Measured on the only
    /// axis that asks a string for a number at all, that cache was serving
    /// 22,400 repeat asks against 94,805 first ones, so what is lost is a
    /// fifth of a quarter of the calls on one axis and nothing on four.
    ///
    /// **`#[inline]`, because a `Bytes` is 56 bytes and both callers build one
    /// at the call site.** Out of line it is written to the caller's stack,
    /// copied across the call, copied into `Body::Text` and copied again into
    /// the slot; inlined, the construction and the `Body` literal have one
    /// destination between them.
    #[inline]
    fn text_bytes(&mut self, bytes: Bytes) -> ObjRef {
        debug_assert!(
            ObjRef::inline_text(&bytes).is_none(),
            "a value that fits the handle built a Bytes on the way here"
        );
        self.alloc_with(BehaviourId::STRING, Body::Text { bytes, num: None })
    }

    /// Creates a number value, applying D15's `SmallInt` admissibility rule.
    ///
    /// `created_digits`/`created_form` are the `DIGITS`/`FORM` in force at
    /// the *operation* that produced `value`, supplied by the caller rather
    /// than read from anywhere on `self` -- there is nothing to read, on
    /// purpose. If `value` is not admissible as a `SmallInt`, it becomes a
    /// heap `Body::Num` carrying the same pair, so the rendering rule is the
    /// same fact stated twice, on the fast path and the general one, never
    /// two different rules that happen to agree today.
    pub(crate) fn number(
        &mut self,
        value: Number,
        created_digits: u32,
        created_form: Form,
    ) -> ObjRef {
        if let Some(small) = small_int_for(&value, created_digits) {
            return ObjRef::small_int(small)
                .expect("small_int_for already checked SMALL_INT_MIN/MAX");
        }
        self.alloc_with(
            BehaviourId::STRING,
            Body::Num {
                value,
                created_digits,
                created_form,
                text: None,
            },
        )
    }

    /// Converts any value to text.
    ///
    /// `.nil` has a string value with no backing bytes at all (D15), and a
    /// `SmallInt` has none stored either, so both are rendered fresh here
    /// rather than looked up. A heap `Body::Num` goes through
    /// [`num_rendering`], which uses `created_digits`/`created_form` and
    /// **never** `settings.digits()`/`settings.form()` -- there is no
    /// `Settings` in scope to reach for by mistake.
    ///
    /// `&mut self` and not `&self`, against the naming convention `to_*`
    /// usually implies, because filling that cache mutates the heap object
    /// the first time this is asked about it. The name is the one the
    /// design's interface list gives every later task to call.
    ///
    /// A `Body::Stem` (D15a, Task 5) renders through its own `default` if
    /// `Some`, else its own `name` -- measured, `w. = 'wd' ; say w.` is
    /// `wd`, while `say q.` on a stem nobody has ever assigned a default to
    /// is `Q.`. The `default` may itself be any value at all, including
    /// (through `stem_assign`'s object-sharing rule, see `stem.rs`) another
    /// `Body::Stem`, so rendering it is a fresh, separate call to this same
    /// function rather than something the match arm below can produce
    /// inline -- the two calls cannot overlap their borrows of `self.heap`,
    /// which is why the redirect is decided and the first borrow dropped
    /// before the second call is ever made.
    /// Appends `value`'s rendering to `out`, without the intermediate
    /// allocation [`to_text`] would need for a value whose bytes are not
    /// already in the heap.
    ///
    /// **The case this exists for is the tagged small integer.** `to_text`
    /// answers `Cow::Borrowed` for `Body::Text` and for `Body::Num` (whose
    /// rendering it caches on the object), so those cost nothing to copy from
    /// -- but a small integer carries no object to borrow from, and `to_text`
    /// has to build a `String` for it and hand it back owned. A caller that
    /// only wants the bytes appended somewhere then allocates that `String`,
    /// copies it, and drops it. Measured with `heaptrack` on
    /// `samples/rexxcps.rex`, whose compound tails are small integers: that
    /// was the whole of the tail-key path's allocation.
    ///
    /// [`to_text`]: Interp::to_text
    pub(crate) fn write_text(&mut self, value: ObjRef, out: &mut Vec<u8>) {
        if let Decoded::SmallInt(n) = value.decode() {
            let mut buffer = [0u8; TEXT_SCRATCH];
            let at = write_small_int(&mut buffer, n);
            out.extend_from_slice(&buffer[at..]);
            return;
        }
        out.extend_from_slice(&self.to_text(value));
    }

    /// How many bytes [`to_text`] would answer, for a caller that wants the
    /// count and not the bytes.
    ///
    /// **Every arm renders exactly what [`to_text`] renders**; what this
    /// saves is what a caller pays to be handed those bytes rather than their
    /// count. `to_text` returns a `Cow<[u8]>`, and reaches it for a tagged
    /// integer or a handle-inline string by writing the value into
    /// `text_scratch` first -- so a caller taking `.len()` of the result pays
    /// for a buffer it never reads. Priced by difference against a control
    /// loop of identical shape, 3,000,000 calls each: `LENGTH` costs 40 fewer
    /// instructions per call on a handle-inline string, 16 fewer on a tagged
    /// integer and 19 fewer on a heap string than the `to_text(..).len()` it
    /// replaced.
    ///
    /// **Answering a `Body::Num` from its shape instead of rendering it was
    /// measured and declined.** `rexx-num` can size a rendering without
    /// writing it, and doing that here made `LENGTH` of a freshly created
    /// number 550 instructions per call cheaper when the number is then
    /// discarded, and 1579 cheaper when it renders exponentially. But it cost
    /// 199 per call whenever something renders the same number anyway, since
    /// the width then buys nothing and the cache is not filled --
    /// `samples/rexxcps.rex` takes `length(j)` of a loop control variable one
    /// clause before string-comparing the same `j`, and paid +0.659% for it
    /// against `strings` -0.201% and `alloc4c` -0.332%. Break-even is around
    /// 27% of number measurements being discards, and nothing says the real
    /// mix is above it.
    ///
    /// Mirroring [`to_text`]'s arms is a duplication, and
    /// `text_len_agrees_with_to_text` plus the assertion below are what
    /// police it -- except for `Body::Num`, which a test cannot police and
    /// which therefore shares [`num_rendering`] rather than restating it.
    /// Routing the rest through `to_text` instead would put a second tag test
    /// on the path of every value: measured, specialising `LENGTH` for the
    /// tagged shapes ahead of a `to_text` call cost +5 instructions on every
    /// arena value and regressed the bench suite.
    ///
    /// [`to_text`]: Interp::to_text
    pub(crate) fn text_len(&mut self, value: ObjRef) -> usize {
        let answer = self.text_len_inner(value);
        // The tripwire that makes the duplication above worth having: it runs
        // on **every** value the gate ever measures, including the arms a
        // unit test cannot easily construct, and it is compiled out of
        // release -- where calling `to_text` is exactly the cost this
        // function exists to avoid.
        debug_assert_eq!(
            answer,
            self.to_text(value).len(),
            "text_len disagrees with to_text"
        );
        answer
    }

    /// [`text_len`] itself, with the assertion lifted off it so that the
    /// recursive `Body::Stem` arm below does not re-run it at every hop.
    ///
    /// [`text_len`]: Interp::text_len
    fn text_len_inner(&mut self, value: ObjRef) -> usize {
        match value.decode() {
            Decoded::Nil => return b"The NIL object".len(),
            Decoded::SmallInt(n) => {
                let at = write_small_int(&mut self.text_scratch, n);
                return TEXT_SCRATCH - at;
            }
            Decoded::Text(inline) => return inline.len(),
            Decoded::Heap { .. } => {}
        }

        let redirect = {
            let Some(object) = self.heap.get(value) else {
                return self.not_in_arena(value).len();
            };
            Redirect::of(&object.body)
        };
        match redirect {
            Redirect::StemDefault(default) => return self.text_len_inner(default),
            // Built rather than measured off the object, because an array's
            // string value exists nowhere until something asks for it -- the
            // same position a tagged integer's digits are in, reached one
            // indirection later.
            Redirect::Array => return self.array_string_of(value).len(),
            // Derived the same way and for the same reason, through the same
            // function `to_text` renders it with.
            Redirect::InstanceDefault(class) => {
                return crate::environment::default_object_name(self.classes().id_string(class))
                    .len();
            }
            Redirect::None => {}
        }

        let object = self.heap.get_mut(value).expect("a live value");
        match &mut object.body {
            Body::Text { bytes, .. } => bytes.len(),
            // Renders and caches, through the same function `to_text` reads.
            // Answering the width from the number's shape instead, so that a
            // number measured and then discarded never renders, was measured
            // and declined -- see this function's doc comment.
            Body::Num {
                value: number,
                created_digits,
                created_form,
                text,
            } => num_rendering(number, *created_digits, *created_form, text).len(),
            Body::Stem { name, .. } => name.len(),
            Body::Native(native) => native.rendered().len(),
            // Reached only with a name set, the arm `to_text` takes in the
            // same position: the redirect above answers for an instance that
            // has none.
            Body::Instance { name, .. } => match name {
                Some(bytes) => bytes.len(),
                None => unreachable!("Redirect::InstanceDefault answers an unnamed instance"),
            },
            other => unreachable!(
                "the value model only creates Text, Num, Stem, Array, Native and Instance, \
                 got {other:?}"
            ),
        }
    }

    /// An array's items, each rendered as [`string_value_text`] renders it and
    /// joined by `separator` -- `ArrayClass::toString`
    /// (`classes/ArrayClass.cpp:1856`), which `ArrayClass::makeString`
    /// forwards to with no arguments (`classes/ArrayClass.cpp:1841`) and which
    /// is therefore also what a string context asks an array for, at the
    /// platform line ending.
    ///
    /// **An empty slot contributes neither a rendering nor a separator.** The
    /// C++ compacts through `makeArray()` before the join and then skips a
    /// null anyway, so the separator sits between rendered items and not
    /// between slots -- measured, `say '<'||(1,,3)||'>'` is `<1` and `3>` on
    /// two lines, `(1,,3)~makeString('C')` is `13`, and
    /// `(1,,3)~makeString('L','-')` is `1-3`.
    ///
    /// The oracle asks each item for `stringValue()` at that loop rather than
    /// `requestString()`, and its own comment there says what the difference
    /// is: an array held inside an array renders as its default name instead
    /// of being joined in turn. Measured, `say '<'||((1,2),3)||'>'` is
    /// `<an Array` and `3>`.
    ///
    /// Measured too, three descriptors: `a = .Array~superClasses; say 'A['a']B'`
    /// renders `A[The Object class` and `The OrderedCollection class]B` on two
    /// lines, and `.Object~superClasses` -- which holds nothing -- renders
    /// empty.
    ///
    /// [`string_value_text`]: Interp::string_value_text
    pub(crate) fn array_string(&mut self, items: &[Option<ObjRef>], separator: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        let mut first = true;
        for item in items.iter().filter_map(|item| *item) {
            if !first {
                out.extend_from_slice(separator);
            }
            let bytes = self.string_value_text(item);
            out.extend_from_slice(&bytes);
            first = false;
        }
        out
    }

    /// `RexxInternalObject::stringValue()`: the rendering of a value as an
    /// object rather than as text, which is what every `TRACE` value line
    /// prints and what an array's own items are joined out of.
    ///
    /// **An array is the one value kind where this and [`to_text`] part.** A
    /// string context asks an array for `makeString` and gets its items
    /// joined; `stringValue` answers the default name. Measured, three
    /// descriptors: `trace i` over `a = (1,,3)` prints `>>>   "an Array"` and
    /// `>=>   A <= "an Array"` where `say '<'||(1,,3)||'>'` prints `<1` and
    /// `3>` on two lines, and a nested array renders `an Array` inside its
    /// parent's join (`say '<'||((1,2),3)||'>'` is `<an Array` and `3>`).
    ///
    /// **Out of line, and that is a measurement.** Every caller is a trace
    /// site behind a `TRACE` gate, so nothing here runs on an untraced pass --
    /// but inlined into [`Interp::intermediate_text`] and
    /// [`Interp::result_text`], both `#[inline(always)]`, it costs the
    /// `strings` axis 29 instructions per pass on the tree-walker arm and the
    /// `varlookup` axis 11, on programs that trace nothing. This is the same
    /// gate-then-outline shape `Interp::echo_symbol_read` already has.
    ///
    /// **Both are marginal readings and the two causes are not additive.**
    /// Each is `b-committed-f4b21eadb` minus `d-no-trace-gates` in
    /// `bench-baselines/phase-5a-arms.tsv`'s `11-bisection` rows, where every
    /// build is named for the single change it carries; `Interp::eval_cold`'s
    /// doc carries the other cause and the residual neither places.
    ///
    /// [`to_text`]: Interp::to_text
    #[inline(never)]
    pub(crate) fn string_value_text(&mut self, value: ObjRef) -> Vec<u8> {
        if matches!(
            self.heap.get(value).map(|object| &object.body),
            Some(Body::Array(_))
        ) {
            return crate::dispatch::ARRAY_DEFAULT_NAME.to_vec();
        }
        self.to_text(value).to_vec()
    }

    /// An array's own slots, borrowed, or `None` for a value that is not an
    /// array.
    ///
    /// Borrowed rather than cloned, so a caller that wants one slot or a count
    /// pays for one slot or a count. A caller that renders the slots has to
    /// clone anyway, because rendering needs `&mut self` and this borrow is
    /// live until it does.
    pub(crate) fn array_slots(&self, value: ObjRef) -> Option<&[Option<ObjRef>]> {
        match &self.heap.get(value)?.body {
            Body::Array(slots) => Some(slots),
            _ => None,
        }
    }

    /// [`Interp::array_slots`] as an owned copy, for a caller that goes on to
    /// use `interp`.
    pub(crate) fn array_slots_of(&self, value: ObjRef) -> Option<Vec<Option<ObjRef>>> {
        Some(self.array_slots(value)?.to_vec())
    }

    /// [`array_string`] for an array named by its handle, whose items it looks
    /// up itself.
    ///
    /// The second lookup [`Redirect::Array`] costs is here, and it is on the
    /// array path alone -- see that variant's own doc for what carrying the
    /// items instead cost every heap string.
    ///
    /// [`array_string`]: Interp::array_string
    fn array_string_of(&mut self, value: ObjRef) -> Vec<u8> {
        // Unreachable as `None`: every caller has just read `Redirect::Array`
        // off this handle's own body, or -- `heap_to_number`'s, which does not
        // go through `Redirect` at all -- has just matched `Body::Array`. An
        // array of no items renders empty, which is what that answers.
        let items = self.array_slots_of(value).unwrap_or_default();
        self.array_string(&items, b"\n")
    }

    #[allow(
        clippy::wrong_self_convention,
        reason = "the interface name is D15's, and `&mut self` is load-bearing \
                   for the lazy cache fill, not a style slip"
    )]
    pub(crate) fn to_text(&mut self, value: ObjRef) -> Cow<'_, [u8]> {
        match value.decode() {
            Decoded::Nil => return Cow::Borrowed(b"The NIL object"),
            // Rendered into the same scratch field the inline arm below uses,
            // for the same reason: `to_string` would allocate a `Vec` per
            // read, and a tagged integer is a value that exists precisely so
            // that it need not be stored anywhere.
            Decoded::SmallInt(n) => {
                let at = write_small_int(&mut self.text_scratch, n);
                return Cow::Borrowed(&self.text_scratch[at..]);
            }
            // Copied into a scratch field and borrowed back out, rather than
            // returned as an owned `Vec`, because a short string is the most
            // common value there is and an allocation here would give back
            // everything inlining them saves.
            //
            // **One buffer is enough, and `&mut self` is what makes that
            // sound.** The borrow this returns holds `self` exclusively, so
            // no second call can run to overwrite it while it is live -- the
            // signature that makes this function awkward to call is the same
            // one that makes a single slot safe.
            Decoded::Text(inline) => {
                let len = inline.len();
                self.text_scratch[..len].copy_from_slice(&inline);
                return Cow::Borrowed(&self.text_scratch[..len]);
            }
            Decoded::Heap { .. } => {}
        }

        let redirect = {
            // **The class arm rides the `None` this lookup already
            // produces**, and costs nothing when it does not fire --
            // [`Interp::not_in_arena`] has the measurement and the reason.
            let Some(object) = self.heap.get(value) else {
                return Cow::Borrowed(self.not_in_arena(value));
            };
            Redirect::of(&object.body)
        };
        match redirect {
            // `Cow::Owned`: the borrow this recursive call returns is tied
            // to `self`, not to `value`'s own object, so the two Cows
            // cannot share one lifetime.
            Redirect::StemDefault(default) => {
                return Cow::Owned(self.to_text(default).into_owned());
            }
            // An array's string value is built here and stored nowhere, so it
            // is `Cow::Owned` and `try_text` answers `None` for one.
            Redirect::Array => return Cow::Owned(self.array_string_of(value)),
            // Derived from the class id and stored nowhere, the position an
            // array's own string value is in.
            //
            // **The oracle sends `DEFAULTNAME` here and this does not.** The
            // paths that can send -- `Object~objectName`, `Object~string` and
            // the required-string protocol -- do; what is left is every
            // rendering reached through an infallible function, which is
            // `TRACE`'s value lines and this crate's own error-message
            // substitutions. Measured, oracle rc 0: under `trace i`, an
            // instance of a class overriding `defaultName` traces as that
            // method's answer where this renders the class id.
            Redirect::InstanceDefault(class) => {
                let id = crate::environment::default_object_name(self.classes().id_string(class));
                return Cow::Owned(id.into_bytes());
            }
            Redirect::None => {}
        }

        let object = self.heap.get_mut(value).expect("a live value");
        match &mut object.body {
            Body::Text { bytes, .. } => Cow::Borrowed(bytes.as_slice()),
            Body::Num {
                value: number,
                created_digits,
                created_form,
                text,
            } => Cow::Borrowed(num_rendering(number, *created_digits, *created_form, text)),
            // Reached for a `Body::Stem` with `default: None` too (the
            // `stem_default` check above only short-circuits the `Some`
            // case), rendering the object's own name.
            //
            // `&**name` and not `name.as_ref()`: the latter fails to borrow-
            // check here (E0515, "cannot return value referencing local
            // variable `name`"), even though `name` is match-ergonomically
            // `&mut Box<[u8]>` here exactly as `text` above is `&mut
            // Vec<u8>`. The explicit double-deref sidesteps whatever `AsRef`
            // impl `.as_ref()`'s method resolution was picking.
            Body::Stem { name, .. } => Cow::Borrowed(&**name),
            // `~objectName`, which the object carries because the two
            // directories were given theirs by the prologue and the rest
            // derive theirs from a class id -- `environment.rs` builds both.
            Body::Native(native) => Cow::Borrowed(native.rendered()),
            // Reached only with a name set: the redirect above answers for
            // an instance that has none.
            Body::Instance { name, .. } => match name {
                Some(bytes) => Cow::Borrowed(bytes),
                None => unreachable!("Redirect::InstanceDefault answers an unnamed instance"),
            },
            other => unreachable!(
                "the value model only creates Text, Num, Stem, Array, Native and Instance, \
                 got {other:?}"
            ),
        }
    }

    /// The name `~objectName=` gave an instance, or `None` for one that still
    /// answers its class's default -- and `None` for every other value kind.
    pub(crate) fn instance_name(&self, value: ObjRef) -> Option<Vec<u8>> {
        match &self.heap.get(value)?.body {
            Body::Instance { name, .. } => name.as_ref().map(|bytes| bytes.to_vec()),
            _ => None,
        }
    }

    /// The bytes of `value`, borrowed from the value itself, or `None` when
    /// this value has no bytes anywhere for a shared borrow to reach.
    ///
    /// **The point of this function is its `&self`, and the cost it removes
    /// is not its own body.** [`to_text`] takes `&mut self` because it fills
    /// two lazy caches, so a caller that wants two operands' bytes at once,
    /// or one operand's bytes and then any further call on the interpreter,
    /// cannot hold what `to_text` returns -- and has to buy its way out with
    /// an owned copy. Those copies are invisible in review, because each one
    /// is locally correct and locally explained. A shared borrow composes,
    /// so callers that only *read* bytes stop paying for the borrow shape.
    ///
    /// Every cause of `None` is "there is nothing to borrow", never "this
    /// value has no text":
    ///
    /// * a tagged small integer, whose digits are stored nowhere at all --
    ///   the value *is* the integer, and its rendering is computed fresh
    ///   every time it is asked for;
    /// * a `Body::Num` whose `text` cache is still empty, which needs the
    ///   `&mut` fill only [`to_text`] can do;
    /// * a `Body::Array`, whose string value is joined out of its items on
    ///   demand and cached nowhere.
    ///
    /// [`render`] turns the second cause into the first and the first into
    /// an owned buffer, so the two together let a caller take shared borrows
    /// of several values at once. Use them as a pair; on its own this
    /// function's `None` arm is a fallback every caller would have to write
    /// itself.
    ///
    /// **A stem's `default` redirect is chased here**, where [`to_text`] has
    /// to answer it with a `Cow::Owned` copy: two shared borrows of `self`
    /// can coexist, so the recursive call's result can be returned directly.
    /// That is the same mechanism this function exists for, applied to the
    /// value model's own internals.
    ///
    /// [`to_text`]: Interp::to_text
    /// [`render`]: Interp::render
    pub(crate) fn try_text(&self, value: ObjRef) -> Option<&[u8]> {
        match value.decode() {
            Decoded::Nil => return Some(b"The NIL object"),
            Decoded::SmallInt(_) => return None,
            // A third cause of `None`, and the same one as a small integer:
            // the bytes live in the handle, which is a `Copy` local here, so
            // there is nothing outliving this call to borrow from.
            Decoded::Text(_) => return None,
            Decoded::Heap { .. } => {}
        }

        // The class arm `to_text` takes, in the same place and for the same
        // reason.
        let Some(object) = self.heap.get(value) else {
            return Some(self.not_in_arena(value));
        };
        match &object.body {
            Body::Text { bytes, .. } => Some(bytes.as_slice()),
            Body::Num { text, .. } => text.as_deref(),
            Body::Stem {
                default: Some(default),
                ..
            } => self.try_text(*default),
            Body::Stem { name, .. } => Some(name),
            Body::Native(native) => Some(native.rendered()),
            // Joined on demand by `to_text` and held nowhere, which is one of
            // the causes of `None` this function's doc names.
            Body::Array(_) => None,
            // The same cause for an instance nothing has named: `to_text`
            // derives those bytes from the class id and stores them nowhere.
            Body::Instance { name, .. } => name.as_deref(),
            other => unreachable!(
                "the value model only creates Text, Num, Stem, Array, Native and Instance, \
                 got {other:?}"
            ),
        }
    }

    /// Makes `value`'s bytes reachable by [`try_text`], and carries the ones
    /// that can never be.
    ///
    /// The `Rendered` this hands back is empty in the case that matters: it
    /// means "a later `try_text` on this value returns `Some`", and a caller
    /// reading several values calls this for every one of them first, then
    /// takes all its shared borrows at once. It is non-empty for an inline
    /// string, a tagged small integer, or a stem resolving to a value whose
    /// bytes are themselves a copy -- and only the last of those allocates.
    ///
    /// **The two arms in front of the `Cow` match are not a shortcut past
    /// it.** `to_text` answers `Cow::Borrowed` both when it has left the bytes
    /// in the heap and when it has put them in the interpreter's single
    /// scratch slot, and only the first of those may become
    /// [`Carried::Borrowable`] -- a `Rendered` outlives the next `to_text`
    /// call, which overwrites that slot. The arms take the two values that
    /// use the slot before the match can confuse them for heap bytes.
    ///
    /// [`try_text`]: Interp::try_text
    /// [`to_text`]: Interp::to_text
    pub(crate) fn render(&mut self, value: ObjRef) -> Rendered {
        // Both of these are checked before `try_text`, because their bytes are
        // neither borrowable nor worth an allocation: they are written into
        // the `Rendered` itself, which the caller already owns.
        match value.decode() {
            Decoded::Text(inline) => {
                return Rendered {
                    value,
                    carried: Carried::Inline(inline),
                };
            }
            // A tagged integer's bytes live nowhere until something asks for
            // them, so `to_text` would leave them in the interpreter's single
            // scratch slot -- a `Cow::Borrowed` that the next call overwrites,
            // which is exactly what a `Rendered` may not hold. Rendered here
            // instead, into storage this value owns.
            Decoded::SmallInt(value_int) => {
                let mut buffer = [0u8; TEXT_SCRATCH];
                let at = write_small_int(&mut buffer, value_int);
                return Rendered {
                    value,
                    carried: Carried::Scratch {
                        buffer,
                        at: at as u8,
                    },
                };
            }
            _ => {}
        }
        if self.try_text(value).is_some() {
            return Rendered {
                value,
                carried: Carried::Borrowable,
            };
        }
        let carried = match self.to_text(value) {
            Cow::Borrowed(_) => Carried::Borrowable,
            Cow::Owned(bytes) => Carried::Owned(bytes),
        };
        Rendered { value, carried }
    }

    /// Converts any value to a `Number`, or `NotNumeric` if it can never be
    /// one.
    ///
    /// `.nil` is never numeric here: real arithmetic on `.nil` fails earlier
    /// still, at message dispatch (measured, `.nil + 1` is error 97.1,
    /// "does not understand message +"), because `.nil`'s class defines no
    /// `+` method at all. 4a has no general message dispatch (Phase 5's), so
    /// this function cannot reproduce that error and does not try to --
    /// `NotNumeric` is the honest answer for what this layer alone can see.
    /// Turning it into a real 41.1 condition is `eval.rs`'s job (Task 7,
    /// `Raised::nonnumeric`): this function only reports "not a number",
    /// never why arithmetic wanted one or what number to raise.
    ///
    /// A `Body::Text`'s `num` cache holds the exact parse and is filled at
    /// most once, by `Number::parse_bytes`. There is no UTF-8 step in front
    /// of it: a Rexx number's characters are ASCII by definition, so a parse
    /// over bytes refuses everything a `from_utf8` guard would have refused
    /// and answers the one `NotNumeric` either way. Nothing here rounds the parse to any
    /// `DIGITS` -- rounding belongs to the operation that reads the result,
    /// which is what lets the same cached parse answer `1.2346` at `DIGITS
    /// 5` and the full value at `DIGITS 20`.
    ///
    /// A `Body::Stem` (D15a) converts through the same redirect `to_text`
    /// uses: its own `default` if `Some`, else its own `name` parsed as a
    /// number (`NotNumeric` when the name does not parse, e.g. the default
    /// derived name `Q.`, which is not a number). This is F5 from the branch
    /// review: `to_text` already had this arm, `to_number` did not, so any
    /// arithmetic, comparison, or numeric-context read of a bare stem
    /// (`a. = 5; say a. + 1`) hit the `unreachable!` below and aborted the
    /// process. Measured against the oracle: `a. = 5; say a. + 1` is `6`;
    /// `say q. + 1` on a stem nobody ever assigned a default to raises 41.1
    /// (non-numeric name `Q.`), which is exactly `NotNumeric` here turning
    /// into a condition one layer up (`eval.rs`'s `Raised::nonnumeric`).
    // `&mut self` for the same reason `to_text` gives: this lazily fills
    // `Body::Text`'s `num` cache in place, which `to_*` alone would not imply.
    #[allow(
        clippy::wrong_self_convention,
        reason = "the interface name is D15's, and `&mut self` is load-bearing \
                   for the lazy cache fill, not a style slip"
    )]
    #[inline]
    pub(crate) fn to_number(&mut self, value: ObjRef) -> Result<Number, NotNumeric> {
        match value.decode() {
            Decoded::Nil => Err(NotNumeric),
            Decoded::SmallInt(n) => Ok(Number::from_i64(n)),
            Decoded::Text(inline) => self.inline_text_to_number(value, &inline),
            Decoded::Heap { .. } => self.heap_to_number(value),
        }
    }

    /// [`to_number`]'s arm for a value whose bytes are in its own handle,
    /// answered from [`TextNumbers`] when that table has been asked before.
    ///
    /// **Cloning what the table holds is a copy and not an allocation.** A
    /// handle carries at most `INLINE_TEXT` bytes, so a `Number` parsed out
    /// of one has at most that many digits, well inside the inline digit
    /// capacity `rexx-num`'s own `Digits` fixes at thirty.
    ///
    /// **Outlined for the reason [`heap_to_number`] is.** [`to_number`] is
    /// inlined into the expression evaluator's own hot loop, and the arms
    /// worth inlining there are a tag test and one call each.
    ///
    /// [`to_number`]: Interp::to_number
    /// [`heap_to_number`]: Interp::heap_to_number
    #[inline(never)]
    fn inline_text_to_number(
        &mut self,
        value: ObjRef,
        inline: &[u8],
    ) -> Result<Number, NotNumeric> {
        let slot = TextNumbers::slot(value);
        if self.text_numbers.keys[slot] == value {
            return self.text_numbers.parsed[slot].clone();
        }
        // Stored before it is handed back, so the parse is moved into the
        // table and only the answer is copied -- filling it the other way
        // round would clone twice for one miss.
        self.text_numbers.keys[slot] = value;
        self.text_numbers.parsed[slot] = Number::parse_bytes(inline).ok_or(NotNumeric);
        self.text_numbers.parsed[slot].clone()
    }

    /// [`to_number`]'s arm for a value that lives in the arena.
    ///
    /// **Outlined so that [`to_number`] is small enough to inline.** The
    /// three arms it leaves behind are a tag test and one call each; this
    /// one is a heap lookup, a stem redirect that recurses, and four body
    /// shapes. Inlining that at every `to_number` call site would cost far
    /// more than the arms worth inlining save. Not `#[cold]`: a `Body::Num`
    /// reaches this on any value produced by earlier arithmetic, which is
    /// ordinary rather than exceptional.
    ///
    /// [`to_number`]: Interp::to_number
    #[inline(never)]
    fn heap_to_number(&mut self, value: ObjRef) -> Result<Number, NotNumeric> {
        // **One lookup, and the stem redirect leaves through it rather than
        // behind it.** The redirect needs its `&mut self` back before it can
        // recurse; copying the default handle out of the arm is what ends the
        // borrow, and it ends it as completely as a separate lookup would.
        // Deciding the redirect first instead costs every value that is *not*
        // a stem with a default -- which is nearly all of them, `Body::Num`
        // above all -- a second walk of the arena to be told so.
        let Some(object) = self.heap.get_mut(value) else {
            // A class object is not a number. The call is for the
            // tripwire it carries, not for the bytes.
            let _ = self.not_in_arena(value);
            return Err(NotNumeric);
        };
        match &mut object.body {
            Body::Num { value, .. } => Ok(value.clone()),
            // Mirrors `to_text`'s own stem redirect above.
            Body::Stem {
                default: Some(default),
                ..
            } => {
                let default = *default;
                self.to_number(default)
            }
            Body::Text { bytes, num } => {
                let bytes = bytes.as_slice();
                let cached = num.get_or_insert_with(|| {
                    Number::parse_bytes(bytes).map(Box::new).ok_or(NotNumeric)
                });
                match cached {
                    Ok(number) => Ok((**number).clone()),
                    Err(marker) => Err(*marker),
                }
            }
            // A `Body::Stem` with `default: None`, the arm the redirect
            // above leaves behind: parses the object's own name, the
            // same fallback `to_text` renders. No cache field
            // exists on `Body::Stem` to hold the parse the way
            // `Body::Text`'s `num` does, so this reparses on every
            // call rather than memoising -- a stem's derived name
            // almost never parses as a number, so there is nothing
            // costly to memoise in the common case, and a cache
            // field added just for this would be new state on
            // `Body::Stem` no other rule needs.
            Body::Stem { name, .. } => Number::parse_bytes(name).ok_or(NotNumeric),
            // A directory's rendering is `a Directory`-shaped text
            // and never numeric, so this answers the marker rather
            // than parsing what `to_text` would produce.
            Body::Native(_) => Err(NotNumeric),
            // **The marker whatever they render as**, because the oracle
            // sends an operator to its left operand as a message and neither
            // answers one: measured, oracle rc 159, `a = (1,); say a + 1` and
            // `o~objectName = '123'; say o + 1` are both
            // `97.1 ... does not understand message "+".` even though each
            // renders as a number, and the comparisons `say (a = 1)` and
            // `say (o = 123)` are both `0` where parsing that rendering gives
            // `1`. `Interp::operator_operand_gap` is what reports them, and
            // it is reached only where this answers `NotNumeric` -- the
            // premise `a_value_the_operator_gap_names_parses_as_no_number`
            // holds.
            Body::Array(_) | Body::Instance { .. } => Err(NotNumeric),
            other => {
                unreachable!(
                    "the value model only creates Text, Num, Stem, Array, Native and Instance, \
                     got {other:?}"
                )
            }
        }
    }
}

/// The integer a literal's bytes spell, when those bytes are exactly that
/// integer's own rendering.
///
/// `Decoded::SmallInt` renders through `i64`'s `Display`, so the eligible
/// spellings are the ones `Display` produces: an optional `-`, then digits
/// with no leading zero unless the value is `0` itself. Everything else keeps
/// its source spelling and stays a string -- `05`, `5.0`, `+5`, `1E5` and
/// `-0` all render differently from the integer they denote, and a literal
/// that renders differently from its source is a wrong answer, not a faster
/// one.
///
/// Returns `None` rather than clamping when the value is outside the tag's
/// range, because the caller's fallback is a correct heap string.
/// **One pass over the digits, which validates and accumulates together.**
/// This used to check `is_ascii_digit` over the whole slice, then hand the
/// same bytes to `from_utf8` and `str::parse` -- three traversals, one of
/// them re-establishing that bytes already known to be ASCII digits are valid
/// UTF-8, and one of them the general `FromStr` machinery for a number whose
/// shape is already known. `checked_mul`/`checked_add` refuse the same
/// overflow `parse` refused, so the value that comes out is the same or there
/// is none; `the_tag_test_agrees_with_parsing_the_same_bytes` holds the two
/// spellings against each other rather than this paragraph doing it.
pub(crate) fn canonical_small_int(bytes: &[u8]) -> Option<i64> {
    let (negative, digits) = match bytes.split_first() {
        Some((b'-', rest)) => (true, rest),
        _ => (false, bytes),
    };
    let (&first, rest) = digits.split_first()?;
    if !first.is_ascii_digit() {
        return None;
    }
    // `-0` is excluded by the same clause that excludes `05`: both render as
    // something other than their own source bytes.
    if first == b'0' && (!rest.is_empty() || negative) {
        return None;
    }
    let mut magnitude = i64::from(first - b'0');
    for &byte in rest {
        if !byte.is_ascii_digit() {
            return None;
        }
        magnitude = magnitude
            .checked_mul(10)?
            .checked_add(i64::from(byte - b'0'))?;
    }
    let value = if negative { -magnitude } else { magnitude };
    (SMALL_INT_MIN..=SMALL_INT_MAX)
        .contains(&value)
        .then_some(value)
}

/// Powers of ten, up to the widest one an `i64` can hold.
///
/// Indexed by a `DIGITS` setting, so index `d` is the first magnitude that
/// needs more than `d` significant digits to write down.
const POW10: [u64; 19] = [
    1,
    10,
    100,
    1_000,
    10_000,
    100_000,
    1_000_000,
    10_000_000,
    100_000_000,
    1_000_000_000,
    10_000_000_000,
    100_000_000_000,
    1_000_000_000_000,
    10_000_000_000_000,
    100_000_000_000_000,
    1_000_000_000_000_000,
    10_000_000_000_000_000,
    100_000_000_000_000_000,
    1_000_000_000_000_000_000,
];

/// Whether `value` is written in at most `digits` significant digits, so
/// that rounding it to that precision is the identity.
///
/// `digits` at or above 19 admits every `i64`, since `i64::MIN` is itself 19
/// digits wide.
pub(crate) fn within_digits(value: i64, digits: u64) -> bool {
    digits >= POW10.len() as u64 || value.unsigned_abs() < POW10[digits as usize]
}

/// An exact integer result as a `SmallInt`, when that handle is the same
/// value [`Interp::number`] would have produced for it -- and `None` when it
/// is not, so the caller runs the general path instead.
///
/// Two conditions, both necessary:
///
/// * **`value` fits the tag.** [`ObjRef::small_int`] decides this; the
///   general path's own `Body::Num` holds everything wider.
/// * **`value` is [`within_digits`]**, so the rounding Rexx applies to
///   *every* arithmetic result leaves it alone. A result that needs rounding
///   renders exponentially: measured under `DIGITS 1`, `15 + 5` is `2E+1`
///   and not `20`.
///
/// **This is a condition on the result alone, and the result alone is not
/// enough to make an operation exact** -- the operands must each be
/// `within_digits` too, which is the caller's to check, because Rexx rounds
/// the operands before it operates on them and not only the answer
/// afterwards. Under `DIGITS 3`, `1000 - 25` is `980`: `1000` needs four
/// digits, so the `25` is aligned against a `1000` that has already lost its
/// last position, and the `5` falls off the end. The result, `980`, is
/// perfectly `within_digits` -- checking it and nothing else admits an
/// answer of `975`. (`ootest`'s `SUBTRACTION::test_147` and `test_151` are
/// that pair; they caught exactly this.)
pub(crate) fn exact_small_int(value: i64, digits: u64) -> Option<ObjRef> {
    within_digits(value, digits)
        .then(|| ObjRef::small_int(value))
        .flatten()
}

/// [`exact_small_int`] asked of a rendering rather than a value: the tag when
/// `rendered` is exactly what it renders back, and `None` when the two would
/// differ.
///
/// The round trip is the whole test. `i64::from_str` accepts spellings the
/// tag does not reproduce -- a leading `+`, leading zeros -- and comparing
/// `Display`'s bytes against the input refuses each without naming it.
fn rendered_small_int(rendered: &[u8], digits: u64) -> Option<ObjRef> {
    let value: i64 = str::from_utf8(rendered).ok()?.parse().ok()?;
    (value.to_string().as_bytes() == rendered)
        .then(|| exact_small_int(value, digits))
        .flatten()
}

/// Whether `value` qualifies for the inline `ObjRef::SmallInt` tag under
/// `created_digits`, and its payload if so (D15).
///
/// **The question is what the value renders as**, because a `SmallInt`
/// renders through `i64`'s `Display` and a heap `Body::Num` renders through
/// `format_form(created_digits, created_form)`, so the tag is admissible
/// exactly when those two agree. [`rendered_integer`] answers that from the
/// number's own exponent and digit vector, and its contract is stated as the
/// rendering: `rexx-num`'s `the_shape_predicate_answers_what_the_rendering_
/// says` asserts the two agree over a generated population, which is what
/// stops the two spellings of one rule drifting apart. The form the caller
/// is under does not enter it -- the two forms agree wherever a rendering is
/// plain, which is the only place this answers `Some`.
///
/// `value` is taken exactly as given, with **no rounding applied here**: the
/// arithmetic operation that produced it already rounded to `created_digits`
/// (`Number::add`/`sub`/`mul`/`div`/`pow` all end in `round_to(digits)`), and
/// the rendering re-rounds to the identical precision, so this never rounds a
/// second time to a *different* one.
///
/// A rendering is admissible only when it is a bare, optionally signed
/// decimal integer -- no `.` and no `E`:
///
/// * **No `.`:** a stored decimal place survives even when every digit after
///   the point is a literal `0` -- measured, `20.00 + 0` prints `20.00`, not
///   `20` -- and a `SmallInt` can only ever render as a bare integer, so a
///   value whose own rendering has a point, however trailing-zero it is, is
///   never eligible. `Number::whole_value` answers a related but different
///   question (whether a value *converts* to a whole number under some
///   precision, which it answers yes for `20.00`) and is the wrong function
///   to reach for here for exactly that reason.
/// * **No `E`:** exponential form means the value does not fit
///   `created_digits` in plain decimal -- exactly the condition under which a
///   bare-integer rendering would be wrong. Measured under `DIGITS 1`:
///   `15 + 0` rounds to `20`, which needs two plain digits and so renders
///   `2E+1`; inlining it as `SmallInt(20)` would print `20` instead.
/// * **Fits `SMALL_INT_MIN..=SMALL_INT_MAX`:** an `i64` already refuses
///   anything wider than 64 bits, and the range check narrows that further to
///   the tag's 61, because `created_digits` carries no ceiling of its own --
///   `NUMERIC DIGITS` can be set far wider than either.
///
/// [`rendered_integer`]: Number::rendered_integer
/// What [`Interp::render`] left behind for one value: nothing, because the
/// value's own bytes are now borrowable, or the bytes themselves, because
/// they live nowhere a borrow can reach.
///
/// **It carries the `ObjRef` it was made for on purpose.** In a converted
/// call site the two halves are deliberately far apart -- every `&mut` use of
/// the interpreter happens between them, which is the whole reason the split
/// exists -- so pairing one value's `Rendered` with another value's read is a
/// mistake that is available to make and would produce the wrong string
/// silently. Holding the `ObjRef` here means the call site never names it
/// twice.
pub(crate) struct Rendered {
    value: ObjRef,
    carried: Carried,
}

/// What [`Interp::render`] had to take a copy of, if anything.
enum Carried {
    /// Nothing: the bytes are where they were and `try_text` reaches them.
    Borrowable,
    /// A rendering that existed nowhere before and does not fit the scratch
    /// arm below -- a stem resolving to a value whose own bytes are already a
    /// copy.
    Owned(Vec<u8>),
    /// Bytes that live in the handle. Copied here rather than allocated,
    /// which is what keeps an inline string free at a call site that needs
    /// several operands' bytes at once.
    Inline(InlineText),
    /// A tagged integer's digits, written into this `Rendered` itself. The
    /// same trade [`Carried::Inline`] makes: the caller already owns this, so
    /// a value that exists only as a tag costs no allocation to read.
    Scratch { buffer: [u8; TEXT_SCRATCH], at: u8 },
}

/// What a value's text has to be read from somewhere other than the object
/// holding it.
///
/// Both [`Interp::to_text`] and [`Interp::text_len`] end their heap lookup
/// with this, and the reason is the borrow: the arms after it hand back a
/// borrow of the object, so the whole match is held for the caller's lifetime
/// and nothing inside it can reach the interpreter again. Deciding here, off
/// a shared borrow that ends, is what lets a stem chase its default and an
/// array join its items.
///
/// **`Array` carries no items, and that is a measured decision.** A variant
/// holding the `Vec` this type would then need drop glue for, and it is
/// returned by value on the path every heap string's rendering takes: carrying
/// one cost `strings` +1.061% of `instructions:u` on the compiled engine and
/// `alloc4c` +0.271%, against 1.000000 on the axes that name no string.
/// Looking the items up again in the arm costs a second `heap.get` on arrays
/// alone.
#[derive(Copy, Clone)]
enum Redirect {
    /// A stem with a default answers *as* that default.
    StemDefault(ObjRef),
    /// An array joins its items, which the arm reads back out of the object.
    Array,
    /// An instance nothing has named, which renders as its class's id with an
    /// article in front and stores those bytes nowhere.
    InstanceDefault(ObjRef),
    /// The object's own body holds the text.
    None,
}

impl Redirect {
    fn of(body: &Body) -> Redirect {
        match body {
            Body::Stem {
                default: Some(default),
                ..
            } => Redirect::StemDefault(*default),
            Body::Array(_) => Redirect::Array,
            Body::Instance {
                class, name: None, ..
            } => Redirect::InstanceDefault(*class),
            _ => Redirect::None,
        }
    }
}

impl Rendered {
    /// The bytes, borrowed from the interpreter's heap wherever
    /// [`Interp::render`] left them there.
    ///
    /// The shared borrow is the point: several of these can be live at once,
    /// which is what lets a caller read two operands without copying either.
    pub(crate) fn text<'a>(&'a self, interp: &'a Interp) -> &'a [u8] {
        match &self.carried {
            Carried::Owned(bytes) => bytes,
            Carried::Inline(inline) => inline,
            Carried::Scratch { buffer, at } => &buffer[usize::from(*at)..],
            Carried::Borrowable => interp
                .try_text(self.value)
                .expect("`render` carried no bytes, so it left them borrowable"),
        }
    }
}

fn small_int_for(value: &Number, created_digits: u32) -> Option<i64> {
    let whole = value.rendered_integer(u64::from(created_digits))?;
    (SMALL_INT_MIN..=SMALL_INT_MAX)
        .contains(&whole)
        .then_some(whole)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rexx_core::INLINE_TEXT;
    use rexx_num::DivOp;

    /// `to_text` renders a tagged integer exactly as `i64`'s `Display` does,
    /// and the scratch it renders into is wide enough for every one of them.
    ///
    /// Both halves matter and neither implies the other. A renderer writing
    /// into too small a buffer panics rather than truncating, so the widest
    /// values are the ones that would find it -- `SMALL_INT_MIN` is nineteen
    /// digits and a sign, which is `TEXT_SCRATCH` exactly, and it is in the
    /// grid for that reason rather than for its value. A renderer wide enough
    /// but spelling a number differently is what the byte equality catches.
    #[test]
    fn a_tagged_integer_renders_as_its_own_display_does() {
        let mut interp = Interp::new();
        for case in [
            0i64,
            1,
            -1,
            9,
            -9,
            10,
            -10,
            99,
            -100,
            1000,
            i64::from(i32::MIN),
            i64::from(u32::MAX),
            SMALL_INT_MAX,
            SMALL_INT_MIN,
        ] {
            let value = ObjRef::small_int(case).expect("inside the tagged range");
            assert_eq!(
                interp.to_text(value).as_ref(),
                case.to_string().as_bytes(),
                "{case}"
            );
        }
        assert_eq!(
            SMALL_INT_MIN.to_string().len(),
            TEXT_SCRATCH,
            "the widest tagged rendering is what fixes `TEXT_SCRATCH`; if this \
             moved, the buffer has slack or is about to overrun"
        );
    }

    /// A remembered parse answers what a fresh one answers, for every
    /// spelling a handle can carry -- numeric and not, at every slot the
    /// table has.
    ///
    /// **The interleaving is the test, not the repetition.** Every value is
    /// asked about twice with every *other* value asked in between, so a
    /// table that returned a neighbour's answer, or that let a collision
    /// overwrite the key without overwriting the parse beside it, gives a
    /// wrong `Number` rather than a stale one. `Number::parse_bytes` on the
    /// same bytes is the reference, and it is what the interpreter would
    /// have done with no table at all.
    #[test]
    fn a_remembered_parse_answers_what_a_fresh_one_does() {
        let spellings: Vec<Vec<u8>> = [
            "0", "1", "-1", "7", "05", "+5", " 5 ", "1.1", "2.2", "1.50", "99.7", "1e2", "1E-2",
            "67", "1234567", "-999999", "foobar", "Key", "?", "string", "", "1.2.3", "0x1f", "e",
            ".5", "-.5", "1.",
        ]
        .iter()
        .map(|s| s.as_bytes().to_vec())
        .collect();
        // Every one of them has to fit the handle, or it would reach
        // `heap_to_number` and this would be testing the wrong cache.
        for spelling in &spellings {
            assert!(
                spelling.len() <= INLINE_TEXT,
                "{spelling:?} does not fit a handle"
            );
        }

        let mut interp = Interp::new();
        let mut answered = 0usize;
        for _pass in 0..3 {
            for spelling in &spellings {
                for other in &spellings {
                    let handle =
                        ObjRef::inline_text(other).expect("checked to fit the handle above");
                    let _ = interp.to_number(handle);
                }
                let handle =
                    ObjRef::inline_text(spelling).expect("checked to fit the handle above");
                let expected = Number::parse_bytes(spelling).ok_or(NotNumeric);
                assert_eq!(
                    interp.to_number(handle),
                    expected,
                    "{:?}",
                    String::from_utf8_lossy(spelling)
                );
                answered += 1;
            }
        }
        // A floor, so a grid that stopped generating cases fails rather than
        // passes empty.
        assert_eq!(answered, spellings.len() * 3);
    }

    /// Two spellings that land in the same slot each keep their own answer.
    ///
    /// **Found rather than assumed.** The pair is searched for at run time
    /// through the table's own slot function, so this stays a test of
    /// eviction however that function is later changed -- a hand-picked pair
    /// would silently stop colliding and leave the test passing without ever
    /// exercising the case.
    #[test]
    fn two_spellings_sharing_a_slot_each_keep_their_own_answer() {
        let candidates: Vec<Vec<u8>> = (0..4000u32)
            .map(|n| n.to_string().into_bytes())
            .filter(|s| s.len() <= INLINE_TEXT)
            .collect();
        let mut collision = None;
        'search: for (at, first) in candidates.iter().enumerate() {
            let first_handle = ObjRef::inline_text(first).expect("short enough");
            for second in &candidates[at + 1..] {
                let second_handle = ObjRef::inline_text(second).expect("short enough");
                if TextNumbers::slot(first_handle) == TextNumbers::slot(second_handle) {
                    collision = Some((first_handle, first.clone(), second_handle, second.clone()));
                    break 'search;
                }
            }
        }
        let (first_handle, first, second_handle, second) =
            collision.expect("two of four thousand spellings share one of thirty-two slots");

        let mut interp = Interp::new();
        for _round in 0..3 {
            assert_eq!(
                interp.to_number(first_handle),
                Number::parse_bytes(&first).ok_or(NotNumeric),
                "{:?}",
                String::from_utf8_lossy(&first)
            );
            assert_eq!(
                interp.to_number(second_handle),
                Number::parse_bytes(&second).ok_or(NotNumeric),
                "{:?}",
                String::from_utf8_lossy(&second)
            );
        }
    }

    /// Two reads in a row each answer their own value.
    ///
    /// The scratch is one slot, so a renderer that left a longer previous
    /// rendering behind it -- or that returned a borrow outliving the next
    /// write -- would show up as the first value's tail hanging off the
    /// second. The pair is deliberately long-then-short.
    #[test]
    fn a_second_tagged_read_does_not_show_the_first_ones_tail() {
        let mut interp = Interp::new();
        let long = ObjRef::small_int(SMALL_INT_MIN).expect("inside the tagged range");
        let short = ObjRef::small_int(7).expect("inside the tagged range");
        assert_eq!(
            interp.to_text(long).to_vec(),
            SMALL_INT_MIN.to_string().into_bytes()
        );
        assert_eq!(interp.to_text(short).to_vec(), b"7".to_vec());
    }

    /// `write_text` must append exactly the bytes `to_text` renders, for every
    /// value that reaches its hand-written formatter.
    ///
    /// **The formatter is the risk, not the dispatch.** `write_text` writes
    /// `i64` digits itself rather than calling `Display`, so this pins it
    /// against `to_string` at both ends of the tagged range, at both ends of
    /// `i64` itself, and across the sign and single-digit boundaries -- and
    /// it appends to a non-empty buffer, because appending is what every
    /// caller does and a formatter that overwrote instead would pass a test
    /// that started from empty.
    #[test]
    fn write_text_writes_what_to_text_renders() {
        let mut interp = Interp::new();
        let cases: [i64; 13] = [
            0,
            1,
            -1,
            9,
            -9,
            10,
            -10,
            99,
            -100,
            i64::from(u32::MAX),
            -i64::from(u32::MAX),
            SMALL_INT_MAX,
            SMALL_INT_MIN,
        ];
        for case in cases {
            let value = ObjRef::small_int(case).expect("inside the tagged range");
            let expected = interp.to_text(value).to_vec();
            assert_eq!(
                expected,
                case.to_string().into_bytes(),
                "the case itself must render as its own decimal spelling: {case}"
            );
            let mut out = b"KEY.".to_vec();
            interp.write_text(value, &mut out);
            assert_eq!(
                out,
                [b"KEY.".as_slice(), &expected].concat(),
                "write_text must append what to_text renders, for {case}"
            );
        }
    }

    /// A test-only shorthand: every literal here is a number by construction,
    /// so a parse failure is this test's own bug, not a case to handle.
    fn n(text: &str) -> Number {
        Number::parse(text).expect("test literal parses")
    }

    /// `text_len` answers what `to_text` measures, for every value kind that
    /// can be built here.
    ///
    /// **The risk is the mirror, not any one arm**: the two functions match
    /// on the same tags and the same `Body` variants, and nothing but this
    /// stops one of them being widened without the other. So the grid is one
    /// value of every kind, plus a wide spread of `Body::Num` -- both `FORM`s
    /// and a precision that rounds -- because that is the arm whose answer
    /// depends on more than the bytes already sitting in the object.
    #[test]
    fn text_len_agrees_with_to_text() {
        let mut interp = Interp::new();
        let mut values: Vec<ObjRef> = vec![ObjRef::NIL];
        for case in [0i64, 1, -1, 9, -10, 1000, SMALL_INT_MAX, SMALL_INT_MIN] {
            values.push(ObjRef::small_int(case).expect("inside the tagged range"));
        }
        for spelling in ["", "a", "abcdef", "abcdefg"] {
            values.push(interp.text(spelling.as_bytes()));
        }
        // Past `INLINE_TEXT` and past `INLINE_BYTES`, so both a slot-inlined
        // and a heap-allocated `Body::Text` are measured.
        values.push(interp.text(&[b'q'; INLINE_TEXT + 1]));
        values.push(interp.text(&[b'q'; INLINE_BYTES + 1]));

        let mut numbers = 0usize;
        for spelling in [
            "0",
            "1.50",
            "-1.50",
            "0.05",
            "-0.05",
            "12.4",
            "99.6",
            "1E+9",
            "1E-9",
            "1E+30",
            "1E-30",
            "10E-19",
            "123456789012",
            "-123456789012",
            "0.000012345",
            "123456789012345678901234567890",
        ] {
            for digits in [1u32, 2, 5, 9, 18, 40] {
                for form in [Form::Scientific, Form::Engineering] {
                    let value = interp.number(n(spelling), digits, form);
                    // A `Body::Num`, not a tagged integer, is what this arm
                    // is about -- the tag path is already in the list above.
                    if matches!(value.decode(), Decoded::Heap { .. }) {
                        numbers += 1;
                    }
                    values.push(value);
                }
            }
        }

        // A stem with no default renders its own name, and one with a default
        // renders through it -- a second `text_len` hop, which is the only
        // recursive arm.
        let bare = interp.alloc_with(
            BehaviourId::STEM,
            Body::Stem {
                name: b"Q.".to_vec().into(),
                default: None,
                tails: rexx_core::NameMap::default(),
            },
        );
        values.push(bare);
        let default = interp.number(n("1.50"), 9, Form::Scientific);
        let defaulted = interp.alloc_with(
            BehaviourId::STEM,
            Body::Stem {
                name: b"W.".to_vec().into(),
                default: Some(default),
                tails: rexx_core::NameMap::default(),
            },
        );
        values.push(defaulted);

        // An array's string value is joined on demand and cached nowhere,
        // which is the redirect arm both functions take.
        let joined = interp.text(b"1");
        let array = interp.alloc_with(BehaviourId::ARRAY, Body::Array(vec![Some(joined), None]));
        values.push(array);

        // Both shapes of instance, because `Redirect::of` answers only for
        // the unnamed one: the named one reaches the body match, which is the
        // half of the mirror a redirect arm cannot stand in for.
        let class = interp
            .classes()
            .lookup("Object")
            .expect("the Object class is registered");
        let behaviour = interp.classes().instance_behaviour_handle(class);
        for name in [None, Some(b"123".to_vec().into_boxed_slice())] {
            let instance = interp.alloc_with(
                BehaviourId::OBJECT,
                Body::Instance {
                    class,
                    behaviour,
                    name,
                    pools: rexx_core::ScopePools::new(),
                    own: None,
                },
            );
            values.push(instance);
        }

        // **`text_len` is asked first, before anything renders**, so that a
        // `Body::Num` reaches its arm with an empty `text` cache at least
        // once. Asking `to_text` first would fill that cache, after which
        // both functions read the same stored bytes and the rendering half of
        // the arm is never exercised -- measured, a mutation replacing its
        // `created_digits` with a constant left this test green when the two
        // calls were the other way round. The second pass is the cached
        // reading, deliberately, and it is the *second*.
        for pass in 0..2 {
            for value in &values {
                let got = interp.text_len(*value);
                let expected = interp.to_text(*value).len();
                assert_eq!(got, expected, "pass {pass}, {value:?}");
            }
        }
        // Floors, because every assertion above is inside a loop.
        assert!(
            values.len() > 100,
            "only {} values were built",
            values.len()
        );
        assert!(
            numbers > 50,
            "only {numbers} of the grid became a `Body::Num`"
        );
    }

    /// The tag decision is the rendering's, at the tag's own boundary.
    ///
    /// `rexx-num`'s `the_shape_predicate_answers_what_the_rendering_says`
    /// holds `rendered_integer` to the rendering it names; what is this
    /// crate's own is the narrowing on top of it, where a value that renders
    /// as a plain integer is still refused for being wider than 61 bits. So
    /// the oracle here is the whole decision written the way it used to be --
    /// render, refuse a `.` or an `E`, parse back, then range-check -- and
    /// the grid crosses the tag's limits with the precisions that reach them.
    ///
    /// The count floors are what stop it passing vacuously, and the second is
    /// the one that matters: a decision equal to `plain_integer` plus the
    /// range check satisfies every assertion below without it.
    /// The single-pass accumulation answers what the validate-then-`parse`
    /// spelling it replaced answered, on every input either of them can see.
    ///
    /// The reference below is that previous spelling, kept whole. Holding the
    /// two against each other is the whole test: a grid of expected answers
    /// written out by hand would be a third opinion, and the one thing that
    /// matters is that this rewrite changed no answer.
    ///
    /// The grid is chosen for the edges the rewrite could plausibly move:
    /// both `i64` limits and one past each, both tag limits and one past
    /// each, the leading-zero and `-0` refusals, a non-digit in the first
    /// position and in a later one (the two now handled by different
    /// branches), and the empty and bare-sign inputs.
    #[test]
    fn the_tag_test_agrees_with_parsing_the_same_bytes() {
        fn reference(bytes: &[u8]) -> Option<i64> {
            let (negative, digits) = match bytes.split_first() {
                Some((b'-', rest)) => (true, rest),
                _ => (false, bytes),
            };
            if digits.is_empty() || !digits.iter().all(u8::is_ascii_digit) {
                return None;
            }
            if digits[0] == b'0' && (digits.len() > 1 || negative) {
                return None;
            }
            let magnitude: i64 = std::str::from_utf8(digits).ok()?.parse().ok()?;
            let value = if negative { -magnitude } else { magnitude };
            (SMALL_INT_MIN..=SMALL_INT_MAX)
                .contains(&value)
                .then_some(value)
        }

        let mut cases: Vec<Vec<u8>> = vec![
            b"".to_vec(),
            b"-".to_vec(),
            b"0".to_vec(),
            b"-0".to_vec(),
            b"00".to_vec(),
            b"05".to_vec(),
            b"-05".to_vec(),
            b"1".to_vec(),
            b"-1".to_vec(),
            b"9".to_vec(),
            b"10".to_vec(),
            b"123456789".to_vec(),
            b"-123456789".to_vec(),
            b"+1".to_vec(),
            b"1.0".to_vec(),
            b"1e5".to_vec(),
            b"1a".to_vec(),
            b"a1".to_vec(),
            b" 1".to_vec(),
            b"1 ".to_vec(),
            b"1\xff".to_vec(),
            b"\xff1".to_vec(),
            b"99999999999999999999999999".to_vec(),
            b"-99999999999999999999999999".to_vec(),
        ];
        for edge in [
            i64::MAX,
            i64::MIN,
            SMALL_INT_MAX,
            SMALL_INT_MIN,
            SMALL_INT_MAX - 1,
            SMALL_INT_MIN + 1,
        ] {
            cases.push(edge.to_string().into_bytes());
        }
        // One past each tag limit, and one past `i64::MAX`, spelled out
        // because they are not themselves representable in the type.
        for spelled in [
            "2305843009213693952",
            "-2305843009213693953",
            "9223372036854775808",
            "-9223372036854775809",
        ] {
            cases.push(spelled.as_bytes().to_vec());
        }

        let mut answered = 0usize;
        for case in &cases {
            let got = canonical_small_int(case);
            assert_eq!(got, reference(case), "{:?}", String::from_utf8_lossy(case));
            if got.is_some() {
                answered += 1;
            }
        }
        assert!(
            answered > 0,
            "every case was refused, so the agreement above is vacuous"
        );
    }

    #[test]
    fn the_tag_decision_is_the_rendering_read_back() {
        fn by_rendering(value: &Number, created_digits: u32) -> Option<i64> {
            let rendered = value.format_form(u64::from(created_digits), Form::Scientific);
            if rendered.contains('.') || rendered.contains('E') {
                return None;
            }
            let whole: i64 = rendered.parse().ok()?;
            (SMALL_INT_MIN..=SMALL_INT_MAX)
                .contains(&whole)
                .then_some(whole)
        }

        let spellings = [
            "0",
            "0.00",
            "-0",
            "1",
            "-1",
            "5",
            "1.50",
            "150",
            "1.50E+2",
            "12.0",
            // Values whose *rounding* is the integer, which is the half of
            // the set `plain_integer` cannot see: dropped digit below and
            // above five, a carry, and both signs.
            "1.4",
            "2.5",
            "12.4",
            "12.5",
            "-12.4",
            "-12.5",
            "19.6",
            "99.4",
            "99.6",
            "9.996",
            "123.456",
            "1234.5678",
            "2305843009213693951.4",
            "2305843009213693952.4",
            "1.2E+3",
            "999",
            "999.5",
            "1000",
            "1E+9",
            "1E+18",
            "1E+19",
            "0.001",
            "1E-5",
            "-12345",
            "123456789012345678",
            // The tag's limits, either side, and one past every `i64`.
            "2305843009213693950",
            "2305843009213693951",
            "2305843009213693952",
            "-2305843009213693952",
            "-2305843009213693953",
            "9223372036854775807",
            "9999999999999999999999",
        ];
        let precisions: [u32; 12] = [1, 2, 3, 4, 5, 9, 18, 19, 20, 21, 22, 25];

        let mut tagged = 0usize;
        let mut tagged_beyond_plain_integer = 0usize;
        for spelling in spellings {
            for digits in precisions {
                let value = n(spelling);
                let decided = small_int_for(&value, digits);
                assert_eq!(
                    decided,
                    by_rendering(&value, digits),
                    "{spelling} at DIGITS {digits}"
                );
                if decided.is_some() {
                    tagged += 1;
                    if value.plain_integer(u64::from(digits)).is_none() {
                        tagged_beyond_plain_integer += 1;
                    }
                }
            }
        }
        assert!(tagged > 100, "only {tagged} of the grid was tagged at all");
        assert!(
            tagged_beyond_plain_integer > 10,
            "only {tagged_beyond_plain_integer} tagged cases are outside `plain_integer`, \
             so this asserts almost nothing about the values a narrower rule would drop"
        );
    }

    /// `text_owned` keeps the caller's buffer instead of copying it.
    ///
    /// **Asserted on the allocation's own address, because that is the only
    /// thing that distinguishes the two.** The bytes come out equal either
    /// way; what a caller sizing a result from user input needs is that the
    /// buffer it reserved *fallibly* is the buffer the value ends up holding,
    /// and a copy would be a second allocation of the same size that no
    /// `try_reserve` can catch. A `text_owned` rewritten as
    /// `self.text(&bytes)` fails here and nowhere else.
    ///
    /// **The buffer is longer than `INLINE_BYTES` on purpose**, and its length
    /// is derived from that constant rather than written out, so this cannot
    /// quietly stop testing the guarantee if the capacity moves. A result that
    /// fits inline *is* copied into the slot, and that is not a breach of the
    /// promise: the results whose size comes from user input are the large
    /// ones, and those are exactly the ones the inline arm never takes.
    #[test]
    fn text_owned_takes_the_buffer_rather_than_copying_it() {
        let long = vec![b'q'; INLINE_BYTES + 1];
        let mut interp = Interp::new();
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(long.len())
            .expect("the buffer is available");
        bytes.extend_from_slice(&long);
        let address = bytes.as_ptr();

        let value = interp.text_owned(bytes);
        assert_eq!(interp.to_text(value).as_ptr(), address);
        assert_eq!(&*interp.to_text(value), &long[..]);
    }

    /// The two construction entry points and the boundary between the arms,
    /// held at the interpreter's own surface rather than only inside `Bytes`.
    ///
    /// A value reads back the same bytes on either arm, and a length that
    /// fits is *not* a separate allocation -- checked by asking whether the
    /// bytes `to_text` hands back lie inside the arena slot they came from,
    /// which is the observable difference and the whole point of the change.
    ///
    /// **The sweep starts above `INLINE_TEXT`**, because below it there is no
    /// slot to be in: those values are the handle. The boundary that rule
    /// owns is pinned by
    /// `a_value_short_enough_is_the_handle_and_has_no_slot` beside this, so
    /// the two lengths are asserted separately rather than one test quietly
    /// covering whichever arm the constants happen to select.
    #[test]
    fn a_short_value_holds_its_bytes_in_the_slot_and_a_long_one_does_not() {
        for len in [
            INLINE_TEXT + 1,
            INLINE_BYTES - 1,
            INLINE_BYTES,
            INLINE_BYTES + 1,
            400,
        ] {
            let source: Vec<u8> = (0..len).map(|i| b'a' + (i % 26) as u8).collect();
            let mut interp = Interp::new();

            for value in [interp.text(&source), interp.text_owned(source.clone())] {
                assert_eq!(&*interp.to_text(value), &source[..], "bytes at {len}");
                let Decoded::Heap { slot, .. } = value.decode() else {
                    panic!("a heap string at {len}")
                };
                let inline = interp
                    .heap
                    .get(value)
                    .is_some_and(|object| match &object.body {
                        Body::Text { bytes, .. } => bytes.is_inline(),
                        other => panic!("a Body::Text at {len}, got {other:?}"),
                    });
                assert_eq!(inline, len <= INLINE_BYTES, "arm at {len}, slot {slot}");
            }
        }
    }

    // The plan's own draft sketched these five tests as calls to
    // `interp.eval_str(...)`/`interp.eval_with(...)`, which do not exist and
    // cannot without pulling Task 6's `Activation`/`Settings` and Task 7's
    // arithmetic forward into this task -- scaffolding Task 7 would then have
    // to unwind. So every `Number` below is built directly through
    // `rexx-num`, which Phase 2 already tested; what these tests check is
    // only what `text`/`number`/`to_text`/`to_number` do with it afterwards.
    // The oracle transcripts remain the source of truth and are re-run
    // against `build/bin/rexx` in the task report.

    #[test]
    fn a_numbers_rendering_is_fixed_when_it_is_created() {
        // numeric digits 9 ; y = 1 / 3 ; numeric digits 3 ; say y
        //   -> 0.333333333 (not 0.333)
        //                   z = 1 / 3 ; say z -> 0.333
        let mut interp = Interp::new();
        let y = interp.number(
            n("1").div(&n("3"), 9, DivOp::Divide).unwrap(),
            9,
            Form::Scientific,
        );
        let z = interp.number(
            n("1").div(&n("3"), 3, DivOp::Divide).unwrap(),
            3,
            Form::Scientific,
        );
        assert_eq!(&*interp.to_text(y), b"0.333333333");
        assert_eq!(&*interp.to_text(z), b"0.333");
        // Reading `y` again proves the cache is genuinely reused, not just
        // right the first time: nothing about creating or reading `z`
        // afterwards moved it.
        assert_eq!(&*interp.to_text(y), b"0.333333333");
    }

    #[test]
    fn numeric_form_is_captured_at_creation_too() {
        // numeric form engineering ; x = 1e10 + 0 ; say x -> 10E+9
        // numeric form scientific  ;               say x -> 10E+9 (unchanged)
        //                            y = 1e10 + 0 ; say y -> 1E+10
        let mut interp = Interp::new();
        let sum = n("1e10").add(&n("0"), 9).unwrap();
        let x = interp.number(sum.clone(), 9, Form::Engineering);
        assert_eq!(&*interp.to_text(x), b"10E+9");
        let y = interp.number(sum, 9, Form::Scientific);
        assert_eq!(&*interp.to_text(x), b"10E+9", "x's own form must not move");
        assert_eq!(&*interp.to_text(y), b"1E+10");
    }

    #[test]
    fn a_small_int_is_only_admissible_within_the_digits_of_its_own_operation() {
        // numeric digits 1 ; x = 15 + 0 ; x is 20, so x + 6 is 3E+1 while
        // 15 + 6 is 2E+1.
        let mut interp = Interp::new();
        let x_number = n("15").add(&n("0"), 1).unwrap();
        let x = interp.number(x_number.clone(), 1, Form::Scientific);
        assert_eq!(&*interp.to_text(x), b"2E+1");

        let sum = interp.number(x_number.add(&n("6"), 1).unwrap(), 1, Form::Scientific);
        assert_eq!(&*interp.to_text(sum), b"3E+1");

        let direct = interp.number(n("15").add(&n("6"), 1).unwrap(), 1, Form::Scientific);
        assert_eq!(&*interp.to_text(direct), b"2E+1");
    }

    #[test]
    fn small_int_admissibility_is_checked_once_against_the_producing_operations_digits() {
        // D15's own discriminating pair: two SmallInt-shaped results that
        // differ observably. Checking the actual `ObjRef` shape, not only the
        // rendered bytes, is the point -- a wrongly-admitted `SmallInt(20)`
        // and a correctly-refused one can render identically to `to_text` if
        // `to_text`'s two branches happen to agree, and only the tag itself
        // tells them apart.
        let mut interp = Interp::new();
        let a = interp.number(n("20"), 9, Form::Scientific);
        assert!(matches!(a.decode(), Decoded::SmallInt(20)));
        assert_eq!(&*interp.to_text(a), b"20");

        let b = interp.number(n("15").add(&n("0"), 1).unwrap(), 1, Form::Scientific);
        assert!(matches!(b.decode(), Decoded::Heap { .. }));
        assert_eq!(&*interp.to_text(b), b"2E+1");
    }

    #[test]
    fn text_keeps_its_own_spelling_and_caches_an_exact_parse() {
        // x = '007' ; say x -> 007 ; say x + 0 -> 7
        let mut interp = Interp::new();
        let x = interp.text(b"007");
        assert_eq!(&*interp.to_text(x), b"007");

        let parsed = interp.to_number(x).unwrap();
        let converted = interp.number(parsed.add(&n("0"), 9).unwrap(), 9, Form::Scientific);
        assert_eq!(&*interp.to_text(converted), b"7");
    }

    #[test]
    fn the_text_cache_holds_the_exact_parse_not_a_rounded_one() {
        // x = '1.234567890123456789'. The SAME stored parse renders 1.2346
        // at DIGITS 5 and the full nineteen-digit value at DIGITS 20.
        // Measured against the oracle: this is `x + 0` at each DIGITS, not
        // `say x` alone -- a `Body::Text`'s identity is its own bytes, so
        // `say x` never converts it at all and prints the literal unchanged
        // at every DIGITS (`t4b.rex`/`t4c.rex` in the task report).
        let mut interp = Interp::new();
        let x = interp.text(b"1.234567890123456789");

        let rounded_5 = interp.to_number(x).unwrap().add(&n("0"), 5).unwrap();
        let at_5 = interp.number(rounded_5, 5, Form::Scientific);
        assert_eq!(&*interp.to_text(at_5), b"1.2346");

        // Reads the cache a second time, under a completely different
        // DIGITS, to show it is still the exact original parse and not
        // whatever the first read happened to round it to.
        let rounded_20 = interp.to_number(x).unwrap().add(&n("0"), 20).unwrap();
        let at_20 = interp.number(rounded_20, 20, Form::Scientific);
        assert_eq!(&*interp.to_text(at_20), b"1.234567890123456789");
    }

    /// Every shape a value can take, read through the borrowing pair, gives
    /// the bytes `to_text` gives.
    ///
    /// This is the whole correctness claim for `try_text`/`render`: they are
    /// an accessor with a different borrow, not a different answer. The
    /// table is the value model's own variant list rather than a sample --
    /// `Nil`, a tagged small integer, a short and a long `Body::Text`, a
    /// `Body::Num` both before and after anything has rendered it, and the
    /// three stem shapes -- because a `None` that quietly fell through to a
    /// wrong-but-plausible answer is exactly what a sample would miss.
    ///
    /// Reading `to_text` **second** on each row is deliberate: it fills the
    /// caches, so a `try_text` that only ever answered for already-rendered
    /// values would pass a test that asked in the other order.
    #[test]
    fn the_borrowing_accessor_answers_what_to_text_answers() {
        let mut interp = Interp::new();

        let short = interp.text(b"abc");
        let long = interp.text(&[b'z'; INLINE_BYTES + 40]);
        let small = interp.number(n("7"), 9, Form::Scientific);
        let heap_number = interp.number(n("1.5"), 9, Form::Scientific);
        let rendered_number = interp.number(n("2.5"), 9, Form::Scientific);
        let _ = interp.to_text(rendered_number);

        let five = interp.number(n("5"), 9, Form::Scientific);
        let with_default = interp.alloc_with(
            BehaviourId::STEM,
            Body::Stem {
                name: b"A.".to_vec().into(),
                default: Some(five),
                tails: rexx_core::NameMap::default(),
            },
        );
        let bare = interp.alloc_with(
            BehaviourId::STEM,
            Body::Stem {
                name: b"Q.".to_vec().into(),
                default: None,
                tails: rexx_core::NameMap::default(),
            },
        );
        let aliasing = interp.alloc_with(
            BehaviourId::STEM,
            Body::Stem {
                name: b"B.".to_vec().into(),
                default: Some(with_default),
                tails: rexx_core::NameMap::default(),
            },
        );

        for value in [
            ObjRef::NIL,
            short,
            long,
            small,
            heap_number,
            rendered_number,
            with_default,
            bare,
            aliasing,
        ] {
            let rendered = interp.render(value);
            let borrowed = rendered.text(&interp).to_vec();
            let expected = interp.to_text(value).into_owned();
            assert_eq!(
                String::from_utf8_lossy(&borrowed),
                String::from_utf8_lossy(&expected),
                "{value:?}"
            );
        }
    }

    /// `try_text` answers `None` for exactly two causes, and both are "there
    /// is nothing to borrow" rather than "there is no text".
    ///
    /// Pinned separately from the agreement test above because the `None`
    /// arm is the one a caller has to handle: an accessor that answered
    /// `Some` here -- with the empty slice, say, or with a `Body::Num`'s
    /// unfilled cache read as blank -- would pass every agreement check that
    /// rendered first, and silently give the wrong string to the callers
    /// that do not.
    #[test]
    fn try_text_answers_only_where_the_bytes_already_exist() {
        let mut interp = Interp::new();

        // Cause one: the digits of a tagged small integer are stored nowhere
        // at all, so nothing renders it into existence and `render` has to
        // carry them.
        let small = interp.number(n("7"), 9, Form::Scientific);
        assert!(matches!(small.decode(), Decoded::SmallInt(7)));
        assert_eq!(interp.try_text(small), None);
        let _ = interp.to_text(small);
        assert_eq!(
            interp.try_text(small),
            None,
            "rendering a SmallInt stores nothing"
        );

        // Cause two: a heap number's `text` cache is empty until something
        // fills it, and filling it is the `&mut` this pair exists to move.
        let number = interp.number(n("1.5"), 9, Form::Scientific);
        assert_eq!(interp.try_text(number), None);
        let _ = interp.to_text(number);
        assert_eq!(interp.try_text(number), Some(&b"1.5"[..]));

        // And a stem chasing a default is answered by the *default's* bytes,
        // which is what `to_text` has to copy and this does not -- so it
        // inherits the default's own answer, `None` included.
        let stem = interp.alloc_with(
            BehaviourId::STEM,
            Body::Stem {
                name: b"A.".to_vec().into(),
                default: Some(small),
                tails: rexx_core::NameMap::default(),
            },
        );
        assert_eq!(interp.try_text(stem), None, "the default is a SmallInt");
        // Long enough to have a slot: a shorter one lives in the handle and
        // so has nothing to borrow, which is the case asserted below.
        let text = interp.text(b"held for long enough to need a slot");
        let stem_of_text = interp.alloc_with(
            BehaviourId::STEM,
            Body::Stem {
                name: b"B.".to_vec().into(),
                default: Some(text),
                tails: rexx_core::NameMap::default(),
            },
        );
        assert_eq!(
            interp.try_text(stem_of_text),
            Some(&b"held for long enough to need a slot"[..])
        );

        // The third cause of `None`, and the one this encoding adds: the
        // bytes are in the handle, which is a `Copy` local, so a shared
        // borrow has nothing to point at. `to_text` still answers.
        let inline = interp.text(b"held");
        assert!(matches!(inline.decode(), Decoded::Text(_)));
        assert_eq!(interp.try_text(inline), None, "the bytes are the handle");
        assert_eq!(&*interp.to_text(inline), b"held");
    }

    /// The other side of that boundary: a value short enough occupies no
    /// slot at all.
    ///
    /// **Asserted on the arena's own count, not only on the tag.** A tag
    /// check alone would pass for an encoding that also allocated, which is
    /// the failure this change exists to avoid; `live_count` standing still
    /// across the construction is the claim that matters.
    #[test]
    fn a_value_short_enough_is_the_handle_and_has_no_slot() {
        for len in 0..=INLINE_TEXT {
            let source: Vec<u8> = (0..len).map(|i| b'a' + (i % 26) as u8).collect();
            let mut interp = Interp::new();
            let before = interp.heap.live_count();

            for value in [interp.text(&source), interp.text_owned(source.clone())] {
                assert!(
                    matches!(value.decode(), Decoded::Text(_)),
                    "carried in the handle at {len}, got {:?}",
                    value.decode()
                );
                assert_eq!(&*interp.to_text(value), &source[..], "bytes at {len}");
            }
            assert_eq!(
                interp.heap.live_count(),
                before,
                "no slot taken at {len}, on either entry point"
            );
        }

        // One byte more is a heap object again, which is what says the test
        // above is reading a boundary rather than a constant.
        let over: Vec<u8> = vec![b'z'; INLINE_TEXT + 1];
        let mut interp = Interp::new();
        let value = interp.text(&over);
        assert!(matches!(value.decode(), Decoded::Heap { .. }));
    }

    #[test]
    fn nil_has_a_string_value_and_the_booleans_are_plain_strings() {
        // say .nil -> The NIL object ; .true is "1" ; .false is "0"
        let mut interp = Interp::new();
        assert_eq!(&*interp.to_text(ObjRef::NIL), b"The NIL object");

        // `.true`/`.false` need no representation of their own (D15): they
        // are the one-byte strings "1" and "0", built the same way any other
        // text value is.
        let true_value = interp.text(b"1");
        let false_value = interp.text(b"0");
        assert_eq!(&*interp.to_text(true_value), b"1");
        assert_eq!(&*interp.to_text(false_value), b"0");
    }

    #[test]
    fn nonnumeric_text_and_nil_both_collapse_to_not_numeric() {
        let mut interp = Interp::new();
        let words = interp.text(b"not a number");
        assert_eq!(interp.to_number(words), Err(NotNumeric));
        assert_eq!(interp.to_number(ObjRef::NIL), Err(NotNumeric));
    }

    // Branch review F5 (Critical): `to_number` on a `Body::Stem` used to hit
    // the `unreachable!` fallback and abort the process (rc 101) --
    // `to_text` already redirected through a stem's default/name, `to_number`
    // never learned to. These three tests build a `Body::Stem` directly
    // (the same shape `stem.rs`'s own allocations use) and drive it straight
    // through `to_number`, with no live activation needed for that alone.

    #[test]
    fn to_number_on_a_stem_redirects_through_its_numeric_default() {
        // a. = 5 ; say a. + 1 -> 6 (measured against the oracle). Kills the
        // mutation that reverts this arm to `unreachable!`: that mutant
        // panics this test instead of returning `Ok(5)`.
        let mut interp = Interp::new();
        let five = interp.number(n("5"), 9, Form::Scientific);
        let stem = interp.alloc_with(
            BehaviourId::STEM,
            Body::Stem {
                name: b"A.".to_vec().into(),
                default: Some(five),
                tails: rexx_core::NameMap::default(),
            },
        );
        let value = interp.to_number(stem).unwrap();
        let sum = interp.number(value.add(&n("1"), 9).unwrap(), 9, Form::Scientific);
        assert_eq!(&*interp.to_text(sum), b"6");
    }

    #[test]
    fn to_number_on_a_defaultless_stem_parses_its_own_name_and_fails() {
        // say q. + 1 on a stem nobody ever assigned a default to: the
        // oracle raises 41.1, nonnumeric value "Q." (measured). Kills a
        // mutation that makes the `default: None` arm return some fixed
        // `Ok` (e.g. `Ok(0)`) instead of parsing the object's own name and
        // reporting `NotNumeric` when, as here, that name is not one.
        let mut interp = Interp::new();
        let stem = interp.alloc_with(
            BehaviourId::STEM,
            Body::Stem {
                name: b"Q.".to_vec().into(),
                default: None,
                tails: rexx_core::NameMap::default(),
            },
        );
        assert_eq!(interp.to_number(stem), Err(NotNumeric));
        // Rendering is untouched by this fix -- still the derived name.
        assert_eq!(&*interp.to_text(stem), b"Q.");
    }

    #[test]
    fn to_number_on_a_stem_aliasing_another_stem_chases_through_both() {
        // a. = 5 ; b. = a. ; say b. + 1 -> 6, the aliasing shape
        // `stem_assign` actually produces (`stem.rs`): `b.`'s default is
        // itself a `Body::Stem`, never a copy of `a.`'s value. Kills a
        // mutation that only redirects one level deep (e.g. matching
        // `Body::Num`/`Body::Text` inline instead of recursing through
        // `to_number` again), which would panic on the second hop.
        let mut interp = Interp::new();
        let five = interp.number(n("5"), 9, Form::Scientific);
        let a = interp.alloc_with(
            BehaviourId::STEM,
            Body::Stem {
                name: b"A.".to_vec().into(),
                default: Some(five),
                tails: rexx_core::NameMap::default(),
            },
        );
        let b = interp.alloc_with(
            BehaviourId::STEM,
            Body::Stem {
                name: b"B.".to_vec().into(),
                default: Some(a),
                tails: rexx_core::NameMap::default(),
            },
        );
        let value = interp.to_number(b).unwrap();
        let sum = interp.number(value.add(&n("1"), 9).unwrap(), 9, Form::Scientific);
        assert_eq!(&*interp.to_text(sum), b"6");
    }
}
