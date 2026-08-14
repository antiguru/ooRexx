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
//! `NotNumeric`, collapsing "not UTF-8" and "not a number" into one marker
//! because nothing observable distinguishes the two (`rexx-core`'s own doc
//! comment on `NotNumeric`).

use crate::Interp;
use rexx_core::{
    BehaviourId, Body, Bytes, Decoded, NotNumeric, ObjRef, SMALL_INT_MAX, SMALL_INT_MIN,
};
use rexx_num::{Form, Number};
use std::borrow::Cow;

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
    /// [`text_owned`]: Interp::text_owned
    pub(crate) fn text(&mut self, bytes: &[u8]) -> ObjRef {
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
        self.text(value.to_string().as_bytes())
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
    pub(crate) fn text_owned(&mut self, bytes: Vec<u8>) -> ObjRef {
        self.text_bytes(Bytes::from_vec(bytes))
    }

    /// The one place a `Body::Text` is built, so the `num` cache's initial
    /// state is stated once.
    fn text_bytes(&mut self, bytes: Bytes) -> ObjRef {
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
    /// rather than looked up. A heap `Body::Num`'s `text` field is filled the
    /// first time it is asked for, through `format_form(created_digits,
    /// created_form)` and **never** `settings.digits()`/`settings.form()` --
    /// there is no `Settings` in scope to reach for by mistake -- and once
    /// filled it is reused forever: the pair is fixed at creation, so the
    /// rendering is a pure function of the object and cannot go stale.
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
            // Same digits `i64`'s own `Display` produces, including for
            // `i64::MIN`, whose magnitude has no positive form -- which is why
            // this goes through `unsigned_abs` rather than negating. Pinned
            // against `to_string` by `write_text_writes_what_to_text_renders`
            // below, over the boundary values and a spread between them.
            let mut buffer = [0u8; 20];
            let mut at = buffer.len();
            let mut magnitude = n.unsigned_abs();
            loop {
                at -= 1;
                buffer[at] = b'0' + (magnitude % 10) as u8;
                magnitude /= 10;
                if magnitude == 0 {
                    break;
                }
            }
            if n < 0 {
                at -= 1;
                buffer[at] = b'-';
            }
            out.extend_from_slice(&buffer[at..]);
            return;
        }
        out.extend_from_slice(&self.to_text(value));
    }

    #[allow(
        clippy::wrong_self_convention,
        reason = "the interface name is D15's, and `&mut self` is load-bearing \
                   for the lazy cache fill, not a style slip"
    )]
    pub(crate) fn to_text(&mut self, value: ObjRef) -> Cow<'_, [u8]> {
        match value.decode() {
            Decoded::Nil => return Cow::Borrowed(b"The NIL object"),
            Decoded::SmallInt(n) => return Cow::Owned(n.to_string().into_bytes()),
            Decoded::Heap { .. } => {}
        }

        let stem_default = {
            let object = self.heap.get(value).expect("a live value");
            match &object.body {
                Body::Stem {
                    default: Some(d), ..
                } => Some(*d),
                _ => None,
            }
        };
        if let Some(default) = stem_default {
            // `Cow::Owned`: the borrow this recursive call returns is tied
            // to `self`, not to `value`'s own object, so the two Cows
            // cannot share one lifetime.
            return Cow::Owned(self.to_text(default).into_owned());
        }

        let object = self.heap.get_mut(value).expect("a live value");
        match &mut object.body {
            Body::Text { bytes, .. } => Cow::Borrowed(bytes.as_slice()),
            Body::Num {
                value: number,
                created_digits,
                created_form,
                text,
            } => {
                let rendered = text.get_or_insert_with(|| {
                    number
                        .format_form(u64::from(*created_digits), *created_form)
                        .into_bytes()
                });
                Cow::Borrowed(rendered.as_slice())
            }
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
            other => unreachable!("the value model only creates Text, Num and Stem, got {other:?}"),
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
    /// `None` has exactly two causes and both are "there is nothing to
    /// borrow", never "this value has no text":
    ///
    /// * a tagged small integer, whose digits are stored nowhere at all --
    ///   the value *is* the integer, and its rendering is computed fresh
    ///   every time it is asked for;
    /// * a `Body::Num` whose `text` cache is still empty, which needs the
    ///   `&mut` fill only [`to_text`] can do.
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
            Decoded::Heap { .. } => {}
        }

        let object = self.heap.get(value).expect("a live value");
        match &object.body {
            Body::Text { bytes, .. } => Some(bytes.as_slice()),
            Body::Num { text, .. } => text.as_deref(),
            Body::Stem {
                default: Some(default),
                ..
            } => self.try_text(*default),
            Body::Stem { name, .. } => Some(name),
            other => unreachable!("the value model only creates Text, Num and Stem, got {other:?}"),
        }
    }

    /// Makes `value`'s bytes reachable by [`try_text`], and carries the ones
    /// that can never be.
    ///
    /// The `Rendered` this hands back is empty in the case that matters: it
    /// means "a later `try_text` on this value returns `Some`", and a caller
    /// reading several values calls this for every one of them first, then
    /// takes all its shared borrows at once. It is non-empty for a tagged
    /// small integer, or a stem resolving to one, and then costs exactly what
    /// [`to_text`] costs today -- which is what makes converting a call site
    /// a pure improvement rather than a trade.
    ///
    /// The `Cow` match below is the whole implementation and it is not a
    /// shortcut: `to_text` returns `Cow::Borrowed` precisely when it has left
    /// the bytes somewhere in the heap, and `Cow::Owned` precisely when it
    /// has not.
    ///
    /// [`try_text`]: Interp::try_text
    /// [`to_text`]: Interp::to_text
    pub(crate) fn render(&mut self, value: ObjRef) -> Rendered {
        if self.try_text(value).is_some() {
            return Rendered { value, owned: None };
        }
        let owned = match self.to_text(value) {
            Cow::Borrowed(_) => None,
            Cow::Owned(bytes) => Some(bytes),
        };
        Rendered { value, owned }
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
    /// most once: `std::str::from_utf8` then `Number::parse`, with both
    /// failures collapsing into `NotNumeric` because a Rexx number's
    /// characters are ASCII by definition, so "not UTF-8" and "not numeric
    /// text" are the same failure. Nothing here rounds the parse to any
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
    pub(crate) fn to_number(&mut self, value: ObjRef) -> Result<Number, NotNumeric> {
        match value.decode() {
            Decoded::Nil => Err(NotNumeric),
            Decoded::SmallInt(n) => Ok(Number::from_i64(n)),
            Decoded::Heap { .. } => {
                // Mirrors `to_text`'s own stem redirect above: decided, and
                // the borrow on `self.heap` dropped, before the recursive
                // call below, which cannot overlap it.
                let stem_default = {
                    let object = self.heap.get(value).expect("a live value");
                    match &object.body {
                        Body::Stem {
                            default: Some(d), ..
                        } => Some(*d),
                        _ => None,
                    }
                };
                if let Some(default) = stem_default {
                    return self.to_number(default);
                }

                let object = self.heap.get_mut(value).expect("a live value");
                match &mut object.body {
                    Body::Num { value, .. } => Ok(value.clone()),
                    Body::Text { bytes, num } => {
                        let bytes = bytes.as_slice();
                        let cached = num.get_or_insert_with(|| match std::str::from_utf8(bytes) {
                            Ok(text) => Number::parse(text).map(Box::new).ok_or(NotNumeric),
                            Err(_) => Err(NotNumeric),
                        });
                        match cached {
                            Ok(number) => Ok((**number).clone()),
                            Err(marker) => Err(*marker),
                        }
                    }
                    // Reached for a `Body::Stem` with `default: None` too
                    // (the `stem_default` check above only short-circuits
                    // the `Some` case): parses the object's own name, the
                    // same fallback `to_text` renders. No cache field
                    // exists on `Body::Stem` to hold the parse the way
                    // `Body::Text`'s `num` does, so this reparses on every
                    // call rather than memoising -- a stem's derived name
                    // almost never parses as a number, so there is nothing
                    // costly to memoise in the common case, and a cache
                    // field added just for this would be new state on
                    // `Body::Stem` no other rule needs.
                    Body::Stem { name, .. } => match std::str::from_utf8(name) {
                        Ok(text) => Number::parse(text).ok_or(NotNumeric),
                        Err(_) => Err(NotNumeric),
                    },
                    other => {
                        unreachable!(
                            "the value model only creates Text, Num and Stem, got {other:?}"
                        )
                    }
                }
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
fn canonical_small_int(bytes: &[u8]) -> Option<i64> {
    let (negative, digits) = match bytes.split_first() {
        Some((b'-', rest)) => (true, rest),
        _ => (false, bytes),
    };
    if digits.is_empty() || !digits.iter().all(u8::is_ascii_digit) {
        return None;
    }
    // `-0` is excluded by the same clause that excludes `05`: both render as
    // something other than their own source bytes.
    if digits[0] == b'0' && (digits.len() > 1 || negative) {
        return None;
    }
    let magnitude: i64 = std::str::from_utf8(digits).ok()?.parse().ok()?;
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
    owned: Option<Vec<u8>>,
}

impl Rendered {
    /// The bytes, borrowed from the interpreter's heap wherever
    /// [`Interp::render`] left them there.
    ///
    /// The shared borrow is the point: several of these can be live at once,
    /// which is what lets a caller read two operands without copying either.
    pub(crate) fn text<'a>(&'a self, interp: &'a Interp) -> &'a [u8] {
        match &self.owned {
            Some(bytes) => bytes,
            None => interp
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
    use rexx_core::{INLINE_BYTES, SMALL_INT_MAX, SMALL_INT_MIN};
    use rexx_num::DivOp;
    use std::collections::HashMap;

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
    #[test]
    fn a_short_value_holds_its_bytes_in_the_slot_and_a_long_one_does_not() {
        for len in [0, 1, INLINE_BYTES - 1, INLINE_BYTES, INLINE_BYTES + 1, 400] {
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
                tails: HashMap::new(),
            },
        );
        let bare = interp.alloc_with(
            BehaviourId::STEM,
            Body::Stem {
                name: b"Q.".to_vec().into(),
                default: None,
                tails: HashMap::new(),
            },
        );
        let aliasing = interp.alloc_with(
            BehaviourId::STEM,
            Body::Stem {
                name: b"B.".to_vec().into(),
                default: Some(with_default),
                tails: HashMap::new(),
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
                tails: HashMap::new(),
            },
        );
        assert_eq!(interp.try_text(stem), None, "the default is a SmallInt");
        let text = interp.text(b"held");
        let stem_of_text = interp.alloc_with(
            BehaviourId::STEM,
            Body::Stem {
                name: b"B.".to_vec().into(),
                default: Some(text),
                tails: HashMap::new(),
            },
        );
        assert_eq!(interp.try_text(stem_of_text), Some(&b"held"[..]));
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
                tails: HashMap::new(),
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
                tails: HashMap::new(),
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
                tails: HashMap::new(),
            },
        );
        let b = interp.alloc_with(
            BehaviourId::STEM,
            Body::Stem {
                name: b"B.".to_vec().into(),
                default: Some(a),
                tails: HashMap::new(),
            },
        );
        let value = interp.to_number(b).unwrap();
        let sum = interp.number(value.add(&n("1"), 9).unwrap(), 9, Form::Scientific);
        assert_eq!(&*interp.to_text(sum), b"6");
    }
}
