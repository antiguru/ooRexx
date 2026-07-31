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
use rexx_core::{BehaviourId, Body, Decoded, NotNumeric, ObjRef, SMALL_INT_MAX, SMALL_INT_MIN};
use rexx_num::{Form, Number};
use std::borrow::Cow;

impl Interp {
    /// Creates a text value: D15's "a value whose identity is its bytes".
    ///
    /// The `num` cache starts at `None`, meaning "not yet asked" and nothing
    /// stronger -- it must not be read as "not a number" for a value nobody
    /// has converted yet, which is the whole reason the cache is a tri-state
    /// rather than a plain `Option<Number>`.
    pub(crate) fn text(&mut self, bytes: &[u8]) -> ObjRef {
        self.heap.alloc_with(
            BehaviourId::STRING,
            Body::Text {
                bytes: bytes.to_vec(),
                num: None,
            },
        )
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
        self.heap.alloc_with(
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
            Decoded::SmallInt(n) => Ok(Number::parse(&n.to_string())
                .expect("an i64's decimal spelling is always a number")),
            Decoded::Heap { .. } => {
                let object = self.heap.get_mut(value).expect("a live value");
                match &mut object.body {
                    Body::Num { value, .. } => Ok(value.clone()),
                    Body::Text { bytes, num } => {
                        let cached = num.get_or_insert_with(|| match std::str::from_utf8(bytes) {
                            Ok(text) => Number::parse(text).map(Box::new).ok_or(NotNumeric),
                            Err(_) => Err(NotNumeric),
                        });
                        match cached {
                            Ok(number) => Ok((**number).clone()),
                            Err(marker) => Err(*marker),
                        }
                    }
                    other => {
                        unreachable!("the value model only creates Text and Num, got {other:?}")
                    }
                }
            }
        }
    }
}

/// Whether `value` qualifies for the inline `ObjRef::SmallInt` tag under
/// `created_digits`, and its payload if so (D15).
///
/// **Decides by rendering, not by inspecting `Number`'s own fields** --
/// `digits`/`exponent`/`negative` are `pub(crate)` to `rexx-num` and this
/// crate cannot reach them, but that constraint turns out to force the right
/// design rather than merely work around a wall: `format_form` is exactly
/// what `to_text` calls for a heap `Body::Num`, so asking it the same
/// question here **guarantees** a `SmallInt`'s rendering and a `Body::Num`'s
/// rendering can never drift apart, which two independent implementations of
/// "is this whole and narrow enough" could not promise. `Form::Scientific`
/// below is a probe, not a decision: D15 states the two forms agree on plain
/// (non-exponential) rendering, and an exponential rendering is refused
/// regardless of which form chose its exponent grouping, so the probe form
/// cannot bias the answer.
///
/// `value` is taken exactly as given, with **no rounding applied here**: the
/// arithmetic operation that produced it already rounded to `created_digits`
/// (`Number::add`/`sub`/`mul`/`div`/`pow` all end in `round_to(digits)`), and
/// `format_form(created_digits, ..)` re-rounds to the identical precision, so
/// this never rounds a second time to a *different* one.
///
/// The rendered string is admissible only when it is a bare, optionally
/// signed decimal integer -- no `.` and no `E`:
///
/// * **No `.`:** a stored decimal place survives even when every digit after
///   the point is a literal `0` -- measured, `20.00 + 0` prints `20.00`, not
///   `20` -- and a `SmallInt` can only ever render as a bare integer, so a
///   value whose own rendering has a point, however trailing-zero it is, is
///   never eligible. `Number::whole_value` answers a related but different
///   question (whether a value *converts* to a whole number under some
///   precision, which it answers yes for `20.00`) and is the wrong function
///   to reach for here for exactly that reason.
/// * **No `E`:** `format_form` chose exponential form, which means the value
///   does not fit `created_digits` in plain decimal -- exactly the condition
///   under which a bare-integer rendering would be wrong. Measured under
///   `DIGITS 1`: `15 + 0` rounds to `20`, which needs two plain digits and so
///   renders `2E+1`; inlining it as `SmallInt(20)` would print `20` instead.
/// * **Fits `SMALL_INT_MIN..=SMALL_INT_MAX`:** parsing the rendered digits
///   into `i64` already refuses anything wider than 64 bits, and the range
///   check narrows that further to the tag's 61, because `created_digits`
///   carries no ceiling of its own -- `NUMERIC DIGITS` can be set far wider
///   than either.
///
/// On a refusal, `number` below throws this rendering away rather than
/// seeding `Body::Num`'s `text` cache with it: the probe always renders in
/// `Scientific`, but a refusal by `E` means the value *is* exponential, where
/// `Scientific` and `Engineering` disagree (D15's own `1E+10`/`10E+9` pair).
/// Caching this string on an object whose `created_form` is `Engineering`
/// would seed the cache with the wrong one; `to_text`'s own lazy fill, keyed
/// off the object's real `created_form`, is what must produce it.
fn small_int_for(value: &Number, created_digits: u32) -> Option<i64> {
    let rendered = value.format_form(u64::from(created_digits), Form::Scientific);
    if rendered.contains('.') || rendered.contains('E') {
        return None;
    }
    let whole: i64 = rendered.parse().ok()?;
    (SMALL_INT_MIN..=SMALL_INT_MAX)
        .contains(&whole)
        .then_some(whole)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rexx_num::DivOp;

    /// A test-only shorthand: every literal here is a number by construction,
    /// so a parse failure is this test's own bug, not a case to handle.
    fn n(text: &str) -> Number {
        Number::parse(text).expect("test literal parses")
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
        let mut interp = Interp::new(false);
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
        let mut interp = Interp::new(false);
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
        let mut interp = Interp::new(false);
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
        let mut interp = Interp::new(false);
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
        let mut interp = Interp::new(false);
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
        let mut interp = Interp::new(false);
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

    #[test]
    fn nil_has_a_string_value_and_the_booleans_are_plain_strings() {
        // say .nil -> The NIL object ; .true is "1" ; .false is "0"
        let mut interp = Interp::new(false);
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
        let mut interp = Interp::new(false);
        let words = interp.text(b"not a number");
        assert_eq!(interp.to_number(words), Err(NotNumeric));
        assert_eq!(interp.to_number(ObjRef::NIL), Err(NotNumeric));
    }
}
