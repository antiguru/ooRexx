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

//! The numeric builtins: `ABS`, `FORMAT`, `MAX`, `MIN`, `RANDOM`, `SIGN` and
//! `TRUNC`.
//! ```text
//! max('a',1,3)   93.943   MAX method target must be a number; found "a".
//! max(1,'a',3)   93.904   Method argument 1 must be a number; found "a".
//! max(1,2,'a')   93.904   Method argument 2 must be a number; found "a".
//! ```
//! ```rexx
//! numeric digits 5 ; x = max(123456789,1) ; numeric digits 12 ; say x
//!   -> 1.2346E+8, not 123456789
//! numeric form engineering ; x = max(1e10,1) ; numeric form scientific ; say x
//!   -> 10E+9, not 1E+10
//! numeric digits 3 ; x = format(1.23456,,4) ; numeric digits 9 ; say x
//!   -> 1.2300, which is text and could not have moved
//! ```

use std::cmp::Ordering;

use rexx_core::ObjRef;
use rexx_num::{CompareOp, DivOp, Form, Number, compare_decoded};

use super::{Args, arg, fresh_buffer, required_string, whole_number};
use crate::Interp;
use crate::error::{Failure, Raised};

/// The `NUMERIC` settings this family reads, fetched once per call.
struct Numeric {
    digits: u64,
    fuzz: u64,
    form: Form,
}

fn current(interp: &Interp) -> Numeric {
    let settings = &interp.activation().settings;
    Numeric {
        digits: settings.digits(),
        fuzz: settings.fuzz(),
        form: settings.form(),
    }
}

/// Argument 1 as a number, or the 93.943 that names this builtin.
fn target_number(
    interp: &mut Interp,
    name: &'static [u8],
    args: Args<'_>,
) -> Result<Number, Failure> {
    let value = arg(args, 1).expect("check_arity admitted the required first argument");
    match interp.to_number(value) {
        Ok(number) => Ok(number),
        Err(_) => {
            let found = required_string(interp, args, 1);
            Err(Raised::method_target_not_a_number(name, &found).into())
        }
    }
}

/// The range half of an optional non-negative argument, after
/// [`super::whole_number`] has already done the type half.
fn non_negative(value: Option<i64>, method_position: usize) -> Result<Option<i64>, Failure> {
    match value {
        Some(value) if value < 0 => Err(Raised::argument_not_non_negative(
            method_position,
            value.to_string().as_bytes(),
        )
        .into()),
        other => Ok(other),
    }
}

/// A width the result is padded out to, as the `u32` `rexx-num` takes, having
/// first asked the allocator for it.
fn padding_width(value: i64) -> Result<u32, Failure> {
    let width = u32::try_from(value).map_err(|_| Failure::from(Raised::system_resources()))?;
    fresh_buffer(width as usize)?;
    Ok(width)
}

// ---- ABS and SIGN ----

/// `ABS(number)`.
pub(crate) fn abs(
    interp: &mut Interp,
    name: &'static [u8],
    args: Args<'_>,
) -> Result<ObjRef, Failure> {
    let value = target_number(interp, name, args)?;
    Ok(abs_of(interp, &value))
}

/// [`abs`]'s answer once its target is a `Number`, shared with `String~abs`
/// the way [`sign_of`] is shared with `String~sign`.
pub(crate) fn abs_of(interp: &mut Interp, value: &Number) -> ObjRef {
    let Numeric { digits, form, .. } = current(interp);
    interp.number(value.abs().into_round(digits), saturate(digits), form)
}

/// The `NUMERIC DIGITS` in force, for a caller that needs it to ask a
/// question of a `Number` rather than to compute one.
pub(crate) fn digits_setting(interp: &Interp) -> u64 {
    current(interp).digits
}

/// `String~FLOOR`, `~CEILING` and `~ROUND`, which differ only in `take`.
pub(crate) fn integer_of(
    interp: &mut Interp,
    value: &Number,
    take: fn(&Number, u64) -> String,
) -> ObjRef {
    let Numeric { digits, .. } = current(interp);
    interp.integer_text(take(value, digits).into_bytes(), digits)
}

/// `String~MODULO`'s answer once its target and divisor are both in hand:
/// the `//` remainder, with the divisor added back when that came out
/// negative.
pub(crate) fn modulo_over(
    interp: &mut Interp,
    value: &Number,
    divisor: &Number,
) -> Result<ObjRef, Failure> {
    let Numeric { digits, form, .. } = current(interp);
    let remainder = value
        .div(divisor, digits, DivOp::Remainder)
        .map_err(Raised::from)?;
    let answer = if remainder.signum() < 0 {
        remainder.add(divisor, digits).map_err(Raised::from)?
    } else {
        remainder
    };
    Ok(interp.number(answer, saturate(digits), form))
}

/// `SIGN(number)`.
pub(crate) fn sign(
    interp: &mut Interp,
    name: &'static [u8],
    args: Args<'_>,
) -> Result<ObjRef, Failure> {
    let value = target_number(interp, name, args)?;
    Ok(sign_of(interp, &value))
}

/// [`sign`]'s answer once its target is a `Number`, shared with `String~sign`
/// so the builtin and the method cannot come to disagree.
pub(crate) fn sign_of(interp: &mut Interp, value: &Number) -> ObjRef {
    let Numeric { digits, form, .. } = current(interp);
    let answer = Number::parse(&value.signum().to_string()).expect("-1, 0 and 1 are all numbers");
    interp.number(answer, saturate(digits), form)
}

// ---- TRUNC and FORMAT ----

/// `TRUNC(number, decimals)`.
/// ```text
/// trunc('AB.CD','V')   40.12   TRUNC argument 2 must be a whole number; found "V".
/// trunc('AB.CD',-1)    93.943  TRUNC method target must be a number; found "AB.CD".
/// trunc(1.5,-1)        93.906  Method argument 1 must be zero or a positive whole number.
/// ```
pub(crate) fn trunc(
    interp: &mut Interp,
    name: &'static [u8],
    args: Args<'_>,
) -> Result<ObjRef, Failure> {
    let places = whole_number(interp, name, args, 2)?;
    let value = target_number(interp, name, args)?;
    trunc_of(interp, &value, non_negative(places, 1)?)
}

/// [`trunc`]'s answer once its target and its places are in hand, shared with
/// `String~trunc`.
pub(crate) fn trunc_of(
    interp: &mut Interp,
    value: &Number,
    places: Option<i64>,
) -> Result<ObjRef, Failure> {
    let Numeric { digits, .. } = current(interp);
    let places = padding_width(places.unwrap_or(0))?;
    Ok(interp.integer_text(value.trunc(digits, places).into_bytes(), digits))
}

/// `FORMAT(number, before, after, expp, expt)`.
pub(crate) fn format(
    interp: &mut Interp,
    name: &'static [u8],
    args: Args<'_>,
) -> Result<ObjRef, Failure> {
    let before = whole_number(interp, name, args, 2)?;
    let after = whole_number(interp, name, args, 3)?;
    let expp = whole_number(interp, name, args, 4)?;
    let expt = whole_number(interp, name, args, 5)?;
    let value = target_number(interp, name, args)?;

    format_over(
        interp,
        &value,
        non_negative(before, 1)?,
        non_negative(after, 2)?,
        non_negative(expp, 3)?,
        non_negative(expt, 4)?,
    )
}

/// [`format`]'s answer once its target and its four widths are in hand, shared
/// with `String~format`.
pub(crate) fn format_over(
    interp: &mut Interp,
    value: &Number,
    before: Option<i64>,
    after: Option<i64>,
    expp: Option<i64>,
    expt: Option<i64>,
) -> Result<ObjRef, Failure> {
    let Numeric { digits, form, .. } = current(interp);
    // `before` and `after` are always materialised -- the interpreter's
    // `leadingSpaces` and `trailingDecimalZeros` are computed on every path
    // -- so both are reserved from the allocator before anything is built.
    let before = before.map(padding_width).transpose()?;
    let after = after.map(padding_width).transpose()?;

    // `expt` is a *threshold* and never occupies a byte, so a value past
    // `u32::MAX` saturates there rather than being refused: every exponent it
    // is compared against is bounded by `MAX_EXPONENT` (999,999,999), so
    // `u32::MAX` and nine quintillion decide every comparison identically.
    // Measured, `format(1,,,,999999999999999999)` is `1`.
    let expt = expt.map(|value| u32::try_from(value).unwrap_or(u32::MAX));

    // `expp` is the one width the interpreter does not always write: it pads
    // an exponent, and there is not always an exponent. So it is reserved
    // only once `rexx-num` says the field will be written, which is the same
    // line the oracle draws -- measured, at the identical width,
    // `format(1,,,3000000000)` is `1` at rc 0 while `format(1,,,3000000000,0)`
    // is `System resources exhausted.` at rc 251.
    let displayed = value
        .format_exponent(
            digits,
            form,
            after,
            expp.map(|width| u32::try_from(width).unwrap_or(u32::MAX)),
            expt,
        )
        .is_some();
    let expp = match expp {
        None => None,
        // Not a width at all: zero is the sentinel that suppresses
        // exponential form, and it has to survive to `rexx-num`.
        Some(0) => Some(0),
        Some(width) if displayed => Some(padding_width(width)?),
        // A width that is never written changes nothing about the result, so
        // it is dropped rather than narrowed. `rexx-num` reads `expp` only
        // through that zero test and through the exponential branch, and
        // `format_exponent` has just said the second is not taken.
        Some(_) => None,
    };

    let text = value
        .format_with(digits, form, before, after, expp, expt)
        .map_err(|error| Failure::from(Raised::from(error)))?;
    Ok(interp.text_built(text.into_bytes()))
}

// ---- MAX and MIN ----

/// Which end of the range [`max_min`] is looking for.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Extreme {
    Max,
    Min,
}

impl Extreme {
    /// The comparison outcome that makes a candidate the new answer.
    fn wins(self, ordering: Ordering) -> bool {
        match self {
            Extreme::Max => ordering == Ordering::Greater,
            Extreme::Min => ordering == Ordering::Less,
        }
    }

    /// [`wins`] as a `rexx-num` predicate, for the general path, where the
    /// comparison honours `FUZZ` and so cannot be a bare `Ord`.
    fn op(self) -> CompareOp {
        match self {
            Extreme::Max => CompareOp::Greater,
            Extreme::Min => CompareOp::Less,
        }
    }
}

/// `Numerics::REXXINTEGER_DIGITS` (`runtime/Numerics.cpp:112`, which defines
/// it as `ARGUMENT_DIGITS`): the widest a value can be and still be held as
/// an integer object rather than a general number.
const REXX_INTEGER_DIGITS: usize = 18;

/// The value's own integer form, when the oracle would be holding it as a
/// `RexxInteger` rather than a `NumberString` or a `RexxString`.
fn integer_object(text: &[u8]) -> Option<i64> {
    let digits = text.strip_prefix(b"-").unwrap_or(text);
    if digits.is_empty() || digits.len() > REXX_INTEGER_DIGITS {
        return None;
    }
    if !digits.iter().all(u8::is_ascii_digit) {
        return None;
    }
    if digits[0] == b'0' && digits.len() > 1 {
        return None;
    }
    std::str::from_utf8(text).ok()?.parse().ok()
}

/// `Numerics::isValid(value, digits)` (`runtime/Numerics.hpp:191`): whether
/// an integer object still fits the precision in force, capped at
/// [`REXX_INTEGER_DIGITS`].
fn valid_under(value: i64, digits: u64) -> bool {
    let width = digits.min(REXX_INTEGER_DIGITS as u64) as u32;
    // `10i64.pow(18)` is the largest this can compute and fits comfortably.
    value.unsigned_abs() < 10u64.pow(width)
}

/// `MAX(number, ...)`.
pub(crate) fn max(
    interp: &mut Interp,
    name: &'static [u8],
    args: Args<'_>,
) -> Result<ObjRef, Failure> {
    max_min(interp, name, args, Extreme::Max)
}

/// `MIN(number, ...)`.
pub(crate) fn min(
    interp: &mut Interp,
    name: &'static [u8],
    args: Args<'_>,
) -> Result<ObjRef, Failure> {
    max_min(interp, name, args, Extreme::Min)
}

/// The body both of them share, and the one place in this file where the
/// answer depends on which *representation* the target has rather than only
/// on its value.
/// ```text
/// max(1,,3)     93.903  Missing argument in method; argument 0 is required.   rc 163
/// max(1.0,,3)   40.5    Missing argument in invocation of MAX; argument 1 ...  rc 216
/// ```
fn max_min(
    interp: &mut Interp,
    name: &'static [u8],
    args: Args<'_>,
    want: Extreme,
) -> Result<ObjRef, Failure> {
    // Only `digits` is read out here: the fast path below is the whole of
    // what this wrapper decides, and `fuzz` and `form` belong to the general
    // path, which now reads them for itself.
    let Numeric { digits, .. } = current(interp);
    let target = arg(args, 1).expect("check_arity admitted the required first argument");
    let rest = args.values_from(2);

    if let Some(answer) = integer_path(interp, target, rest, digits, want)? {
        return Ok(answer);
    }

    let best = target_number(interp, name, args)?;
    let best_text = required_string(interp, args, 1);
    let objects: Vec<Option<ObjRef>> = (0..rest.len()).map(|i| args.object(i + 2)).collect();
    max_min_over(interp, name, &best, best_text, rest, &objects, want)
}

/// `MAX`'s and `MIN`'s general path, shared with `String~max` and `String~min`.
pub(crate) fn max_min_over(
    interp: &mut Interp,
    name: &'static [u8],
    target: &Number,
    target_text: Vec<u8>,
    rest: &[Option<ObjRef>],
    objects: &[Option<ObjRef>],
    want: Extreme,
) -> Result<ObjRef, Failure> {
    let Numeric { digits, fuzz, form } = current(interp);
    let mut best = target.clone().into_round(digits);
    let mut best_text = target_text;
    for (index, slot) in rest.iter().enumerate() {
        let Some(candidate) = slot else {
            return Err(Raised::missing_argument(name, index + 1).into());
        };
        let found = interp.to_text(*candidate).into_owned();
        let Ok(number) = interp.to_number(*candidate) else {
            // The object rather than the conversion, which is every
            // argument message's rule -- `whole_number`'s own doc has the
            // measurement behind it.
            let named = match objects.get(index).copied().flatten() {
                Some(object) => interp.string_value_text(object),
                None => found,
            };
            return Err(Raised::method_argument_not_a_number(index + 1, &named).into());
        };
        let number = number.into_round(digits);
        // Both sides are already parsed, so the byte slices are never read
        // for these two non-strict operators; they are the real renderings
        // anyway rather than a placeholder that would become wrong if a
        // later `CompareOp` needed them.
        let takes_over = compare_decoded(
            &found,
            Some(&number),
            &best_text,
            Some(&best),
            digits,
            fuzz,
            want.op(),
        )
        .map_err(|error| Failure::from(Raised::from(error)))?;
        if takes_over {
            best = number;
            best_text = found;
        }
    }
    Ok(interp.number(best, saturate(digits), form))
}

/// `RexxInteger::Max`/`::Min`'s fast path, or `None` when the oracle would
/// have fallen through to `NumberString::maxMin` with the whole argument list.
fn integer_path(
    interp: &mut Interp,
    target: ObjRef,
    rest: &[Option<ObjRef>],
    digits: u64,
    want: Extreme,
) -> Result<Option<ObjRef>, Failure> {
    let text = interp.to_text(target).into_owned();
    let Some(value) = integer_object(&text) else {
        return Ok(None);
    };
    if want == Extreme::Min && rest.is_empty() {
        return Ok(Some(target));
    }
    if !valid_under(value, digits) {
        return Ok(None);
    }
    if rest.is_empty() {
        return Ok(Some(target));
    }

    let mut best = value;
    let mut best_object = target;
    for (index, slot) in rest.iter().enumerate() {
        // The omission check runs before this argument's own type is looked
        // at, which is `requiredArgument(argument, arg)`'s position in the
        // C++ loop, and `index` is passed to it unincremented -- so the
        // message really does say "argument 0" for the call's argument 2.
        let Some(candidate) = slot else {
            return Err(Raised::missing_method_argument(index).into());
        };
        let candidate_text = interp.to_text(*candidate).into_owned();
        let Some(other) = integer_object(&candidate_text) else {
            // One non-integer sends the *whole* list back to the general
            // path, which rescans from the first argument -- so an omission
            // later in the list is never reached. Measured:
            // `max(1,'a',,4)` is 93.904 naming method argument 1, not the
            // 40.5 the omission at position 3 would have given.
            return Ok(None);
        };
        if want.wins(other.cmp(&best)) {
            best = other;
            best_object = *candidate;
        }
    }
    Ok(Some(best_object))
}

// ---- RANDOM ----

/// `RexxActivation::RANDOM_FACTOR` and `RANDOM_ADDER`
/// (`execution/RexxActivation.hpp:593`).
const RANDOM_FACTOR: u64 = 25_214_903_917;
const RANDOM_ADDER: u64 = 11;

/// `DefaultRandomMin`/`DefaultRandomMax`/`MaxRandomRange`
/// (`execution/RexxActivation.hpp:601`).
const DEFAULT_MIN: i64 = 0;
const DEFAULT_MAX: i64 = 999;
const MAX_RANGE: i64 = 999_999_999;

/// `RANDOMIZE` (`execution/RexxActivation.hpp:596`), wrapping because the
/// C++ multiplies `uint64_t`.
fn randomize(seed: u64) -> u64 {
    seed.wrapping_mul(RANDOM_FACTOR).wrapping_add(RANDOM_ADDER)
}

/// `RANDOM(minimum, maximum, seed)`.
pub(crate) fn random(
    interp: &mut Interp,
    name: &'static [u8],
    args: Args<'_>,
) -> Result<ObjRef, Failure> {
    let minimum = whole_number(interp, name, args, 1)?;
    let maximum = whole_number(interp, name, args, 2)?;
    let seed = whole_number(interp, name, args, 3)?;

    // The seed is validated and *applied* before the range is looked at,
    // which is `RexxActivation::random`'s own first statement. A call that
    // then fails its range check has still moved the stream on.
    if let Some(seed) = seed
        && seed < 0
    {
        let found = seed.to_string();
        return Err(Raised::argument_not_non_negative_call(name, 3, found.as_bytes()).into());
    }
    let scrambled = next_seed(interp, seed);

    // `RANDOM`'s three arguments do not mean the same thing in every
    // combination, and the seed is what decides: with no maximum and no
    // seed, a lone argument is the *maximum*, but with a seed it is the
    // minimum. Measured, `random(0)` is always `0` while `random(1,,2)` --
    // the same lone argument with a seed behind it -- answered `313`, which
    // is above it.
    let (mut low, high) = match (minimum, maximum, seed) {
        (Some(low), None, None) => (DEFAULT_MIN, low),
        (Some(low), None, Some(_)) => (low, DEFAULT_MAX),
        (Some(low), Some(high), _) => (low, high),
        (None, Some(high), _) => (DEFAULT_MIN, high),
        (None, None, _) => (DEFAULT_MIN, DEFAULT_MAX),
    };

    // An argument the call omitted substitutes as the null string, because
    // the C++ passes `OREF_NULL` -- measured, `random(-1)` reports
    // `argument 1 ("-1")` and `argument 2 ("")`.
    let written = |value: Option<i64>| value.map_or(Vec::new(), |v| v.to_string().into_bytes());
    if high < low {
        return Err(Raised::random_bounds_reversed(&written(minimum), &written(maximum)).into());
    }
    if high - low > MAX_RANGE {
        return Err(Raised::random_range_too_wide(&written(minimum), &written(maximum)).into());
    }

    if low != high {
        let spread = u64::try_from(high - low + 1).expect("the range check bounded this");
        low += i64::try_from(reverse_bits(scrambled) % spread).expect("a value below the spread");
    }
    // **Text, not a number.** `RexxActivation::random` answers
    // `new_integer(minimum)`, a `RexxInteger`, whose spelling is its own
    // decimal digits and is never re-rendered under a later `DIGITS` or
    // `FORM`. Measured: `numeric digits 3 ; say random(12345,12345)` is
    // `12345`, where a value carrying the D15 pair would render `1.23E+4` --
    // and `say random(12345,12345) + 0` at the same setting *is* `1.23E+4`,
    // because that is the addition's own result rather than this one.
    Ok(interp.text_built(low.to_string().into_bytes()))
}

/// `RexxActivation::getRandomSeed`: install a supplied seed, then advance the
/// stream once and answer the new state.
fn next_seed(interp: &mut Interp, seed: Option<i64>) -> u64 {
    if let Some(seed) = seed {
        // "flipping all of the bits gives us a better spread", then thirteen
        // scrambles, then the unconditional fourteenth below.
        let mut state = !(seed as u64);
        for _ in 0..13 {
            state = randomize(state);
        }
        interp.random_seed = Some(state);
    }
    let state = interp.random_seed.get_or_insert_with(initial_seed);
    *state = randomize(*state);
    *state
}

/// The starting state for a program that never supplies a seed.
fn initial_seed() -> u64 {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |elapsed| elapsed.subsec_nanos());
    u64::from(nanos) << 32 ^ u64::from(std::process::id())
}

/// The bit reversal `RexxActivation::random` does before taking the modulus:
/// the seed's bits, most significant first.
fn reverse_bits(seed: u64) -> u64 {
    let mut work = 0u64;
    let mut seed = seed;
    for _ in 0..u64::BITS {
        work = (work << 1) | (seed & 1);
        seed >>= 1;
    }
    work
}

/// The `DIGITS` in force, narrowed the way every other value-producing site
/// narrows it (`eval::saturate_digits`).
fn saturate(digits: u64) -> u32 {
    crate::eval::saturate_digits(digits)
}

#[cfg(test)]
mod tests;
